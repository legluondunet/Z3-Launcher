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
    acquire_named(r"Local\Z3-Launcher.GUI.Singleton")
}

#[cfg(unix)]
pub struct Guard { _file: std::fs::File }
#[cfg(unix)]
extern "C" { fn flock(fd: std::os::raw::c_int, operation: std::os::raw::c_int) -> std::os::raw::c_int; }
#[cfg(unix)]
fn acquire_file(path: &std::path::Path) -> io::Result<Option<Guard>> {
    use std::os::unix::{fs::OpenOptionsExt, io::AsRawFd};
    let file = std::fs::OpenOptions::new().read(true).write(true).create(true)
        .truncate(false).mode(0o600).open(path)?;
    // LOCK_EX | LOCK_NB. The descriptor stays open in Guard; closing releases the lock.
    if unsafe { flock(file.as_raw_fd(), 2 | 4) } == 0 { return Ok(Some(Guard { _file: file })); }
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
    acquire_file(&directory.join("launcher.lock"))
}

#[cfg(test)]
mod tests {
    use super::*;
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
