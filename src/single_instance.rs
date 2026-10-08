// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 legluondunet — https://github.com/legluondunet
//! Hold an OS lock for the lifetime of the graphical launcher.
use std::io;

#[cfg(windows)]
pub struct Guard(*mut std::ffi::c_void);
#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn CreateMutexW(attributes: *const std::ffi::c_void, owner: i32, name: *const u16) -> *mut std::ffi::c_void;
    fn GetLastError() -> u32;
    fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
}
#[cfg(windows)]
impl Drop for Guard {
    fn drop(&mut self) {
        // This guard owns exactly one valid kernel handle.
        unsafe { CloseHandle(self.0); }
    }
}
#[cfg(windows)]
fn acquire_named(name: &str) -> io::Result<Option<Guard>> {
    let name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    // The name is terminated and remains alive for the call; no handle inheritance.
    let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
    let error = unsafe { GetLastError() };
    if handle.is_null() { return Err(io::Error::from_raw_os_error(error as i32)); }
    let guard = Guard(handle);
    if error == 183 { drop(guard); Ok(None) } else { Ok(Some(guard)) }
}
#[cfg(windows)]
pub fn acquire() -> io::Result<Option<Guard>> {
    // Shared by installed and portable copies in the same Windows session.
    let guard = acquire_named(r"Local\Z3-Launcher.GUI.Singleton")?;
    if guard.is_none() { activate_existing_window(); }
    Ok(guard)
}

#[cfg(unix)]
pub struct Guard {
    _file: std::fs::File,
    socket: Option<std::os::unix::net::UnixDatagram>,
    socket_path: Option<std::path::PathBuf>,
    activation: std::sync::Arc<std::sync::atomic::AtomicBool>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
#[cfg(unix)]
impl Drop for Guard {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Release);
        if let Some(path) = &self.socket_path { let _ = std::fs::remove_file(path); }
    }
}
#[cfg(unix)]
extern "C" { fn flock(fd: std::os::raw::c_int, operation: std::os::raw::c_int) -> std::os::raw::c_int; }
#[cfg(unix)]
fn acquire_file(path: &std::path::Path) -> io::Result<Option<Guard>> {
    use std::os::unix::{fs::OpenOptionsExt, io::AsRawFd};
    let file = std::fs::OpenOptions::new().read(true).write(true).create(true)
        .truncate(false).mode(0o600).open(path)?;
    // LOCK_EX | LOCK_NB. The descriptor stays open in Guard; closing releases the lock.
    if unsafe { flock(file.as_raw_fd(), 2 | 4) } == 0 { return Ok(Some(Guard { _file: file, socket: None, socket_path: None, activation: Default::default(), stop: Default::default() })); }
    let error = io::Error::last_os_error();
    if error.kind() == io::ErrorKind::WouldBlock { Ok(None) } else { Err(error) }
}
#[cfg(unix)]
pub fn acquire() -> io::Result<Option<Guard>> {
    use std::os::unix::fs::{DirBuilderExt, MetadataExt};
    extern "C" { fn getuid() -> u32; }
    let uid = unsafe { getuid() };
    // Keep the lock independent of portable HOME, workspace and executable location.
    let runtime = std::path::PathBuf::from(format!("/run/user/{uid}"));
    let directory = if runtime.is_dir() { runtime.join("Z3-Launcher") }
        else { std::path::PathBuf::from(format!("/tmp/Z3-Launcher-{uid}")) };
    match std::fs::DirBuilder::new().mode(0o700).create(&directory) {
        Ok(()) => {},
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {},
        Err(error) => return Err(error),
    }
    let metadata = std::fs::symlink_metadata(&directory)?;
    if !metadata.is_dir() || metadata.uid() != uid {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Invalid instance lock directory"));
    }
    let socket_path = directory.join("activate.sock");
    let Some(mut guard) = acquire_file(&directory.join("launcher.lock"))? else {
        let sender = std::os::unix::net::UnixDatagram::unbound()?;
        // The first instance may still be creating its socket after taking the lock.
        for _ in 0..40 {
            if sender.send_to(b"activate", &socket_path).is_ok() { return Ok(None); }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        return Err(io::Error::new(io::ErrorKind::NotConnected, "Cannot activate the running launcher"));
    };
    guard.bind_activation(&socket_path)?;
    Ok(Some(guard))
}

#[cfg(unix)]
impl Guard {
    fn bind_activation(&mut self, socket_path: &std::path::Path) -> io::Result<()> {
        // The caller holds the lock, so a stale socket can safely be removed.
        match std::fs::remove_file(socket_path) {
            Ok(()) => {},
            Err(error) if error.kind() == io::ErrorKind::NotFound => {},
            Err(error) => return Err(error),
        }
        let socket = std::os::unix::net::UnixDatagram::bind(socket_path)?;
        socket.set_read_timeout(Some(std::time::Duration::from_millis(200)))?;
        self.socket = Some(socket); self.socket_path = Some(socket_path.to_path_buf());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    #[test]
    fn activation_socket_receives_requests_and_recovers_a_stale_socket() {
        let directory = std::env::temp_dir().join(format!("z3-activate-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let socket_path = directory.join("activate.sock");
        // A crashed instance can leave its socket pathname behind.
        drop(std::os::unix::net::UnixDatagram::bind(&socket_path).unwrap());
        let mut guard = acquire_file(&directory.join("launcher.lock")).unwrap().unwrap();
        guard.bind_activation(&socket_path).unwrap();
        let sender = std::os::unix::net::UnixDatagram::unbound().unwrap();
        sender.send_to(b"activate", &socket_path).unwrap();
        let mut buffer = [0; 32];
        let n = guard.socket.as_ref().unwrap().recv(&mut buffer).unwrap();
        assert_eq!(&buffer[..n], b"activate");
        drop(guard);
        assert!(!socket_path.exists());
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn excludes_second_instance_and_releases_on_drop() {
        let unique = format!("z3-single-test-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        #[cfg(windows)]
        let claim = || acquire_named(&format!(r"Local\{unique}"));
        #[cfg(unix)]
        let path = std::env::temp_dir().join(unique);
        #[cfg(unix)]
        let claim = || acquire_file(&path);
        let first = claim().unwrap().expect("first instance");
        assert!(claim().unwrap().is_none());
        drop(first);
        let restarted = claim().unwrap().expect("restart after close");
        drop(restarted);
        #[cfg(unix)]
        std::fs::remove_file(path).unwrap();
    }
}

#[cfg(windows)]
fn activate_existing_window() {
    type Window = *mut std::ffi::c_void;
    #[link(name = "user32")]
    extern "system" {
        fn FindWindowW(class: *const u16, title: *const u16) -> Window;
        fn IsIconic(window: Window) -> i32;
        fn ShowWindowAsync(window: Window, command: i32) -> i32;
        fn SetForegroundWindow(window: Window) -> i32;
    }
    let title: Vec<u16> = "Z3-Launcher".encode_utf16().chain(Some(0)).collect();
    // A freshly launched process can activate the existing window in response to the user.
    // Retry briefly if the first process has its mutex but has not created its window yet.
    for _ in 0..40 {
        let window = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) };
        if !window.is_null() {
            unsafe {
                if IsIconic(window) != 0 { ShowWindowAsync(window, 9); } // SW_RESTORE
                SetForegroundWindow(window);
            }
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

#[cfg(feature = "gui")]
impl Guard {
    pub fn watch_activation(&self, ctx: &eframe::egui::Context) -> io::Result<()> {
        #[cfg(unix)]
        if let Some(socket) = &self.socket {
            let socket = socket.try_clone()?;
            let stop = self.stop.clone(); let activation = self.activation.clone(); let ctx = ctx.clone();
            std::thread::spawn(move || {
                let mut buffer = [0; 32];
                while !stop.load(std::sync::atomic::Ordering::Acquire) {
                    match socket.recv(&mut buffer) {
                        Ok(n) if &buffer[..n] == b"activate" => {
                            activation.store(true, std::sync::atomic::Ordering::Release);
                            ctx.request_repaint();
                        },
                        Ok(_) => {},
                        Err(e) if matches!(e.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut | io::ErrorKind::Interrupted) => {},
                        Err(_) => break,
                    }
                }
            });
        }
        #[cfg(windows)]
        let _ = ctx;
        Ok(())
    }
    pub fn take_activation(&self) -> bool {
        #[cfg(unix)]
        { self.activation.swap(false, std::sync::atomic::Ordering::AcqRel) }
        #[cfg(windows)]
        { false }
    }
}
