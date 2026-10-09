// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 legluondunet — https://github.com/legluondunet
use std::{path::{Path,PathBuf}, sync::OnceLock};
static PORTABLE: OnceLock<Option<PathBuf>> = OnceLock::new();
fn appimage_home_at(image: &Path) -> Option<PathBuf> {
    let mut path = image.as_os_str().to_os_string();
    path.push(".home");
    let path = PathBuf::from(path);
    path.is_dir().then_some(path)
}
fn appimage_home() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    if std::env::var_os("Z3_APPIMAGE_ENV").is_some() {
        return std::env::var_os("APPIMAGE").and_then(|image| appimage_home_at(Path::new(&image)));
    }
    None
}
fn home() -> PathBuf { std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".")) }
fn portable_at(executable: &Path) -> Option<PathBuf> {
    let parent = executable.parent()?;
    if parent.join("portable.txt").is_file() { Some(parent.join("Z3-Launcher")) } else { None }
}
pub fn portable_dir() -> Option<PathBuf> {
    PORTABLE.get_or_init(|| {
        // .home takes priority and follows the image when moved or renamed.
        if let Some(home) = appimage_home() { return Some(home.join(".local/share/Z3-Launcher")); }
        // AppImage mounts are read-only. Resolve portable.txt next to the actual AppImage.
        #[cfg(target_os = "linux")]
        if std::env::var_os("Z3_APPIMAGE_ENV").is_some() {
            if let Some(image) = std::env::var_os("APPIMAGE") { return portable_at(Path::new(&image)); }
        }
        std::env::current_exe().ok().and_then(|exe| portable_at(&exe))
    }).clone()
}
pub fn is_portable() -> bool { portable_dir().is_some() }
pub fn data_dir() -> PathBuf {
    #[cfg(windows)]
    { return portable_dir().unwrap_or_else(|| windows_home().join("Z3-Launcher")); }
    #[cfg(not(windows))]
    portable_dir().unwrap_or_else(||std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".local/share")).join("Z3-Launcher"))
}
pub fn config_dir() -> PathBuf {
    if let Some(home) = appimage_home() { return home.join(".config/Z3-Launcher"); }
    #[cfg(windows)]
    { return portable_dir().map(|root|root.join("config")).unwrap_or_else(|| windows_home().join("Z3-Launcher/config")); }
    #[cfg(not(windows))]
    portable_dir().map(|root|root.join("config")).unwrap_or_else(||std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".config")).join("Z3-Launcher"))
}
#[cfg(not(windows))]
pub const GAME_BINARY: &str = "zelda3.AppImage";
#[cfg(windows)]
pub const GAME_BINARY: &str = "zelda3.exe";
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn appimage_home_requires_a_directory_and_keeps_the_full_filename() {
        let dir=std::env::temp_dir().join(format!("appimage-home-test-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&dir).unwrap();
        let image=dir.join("Launcher été.AppImage");
        let portable_home=dir.join("Launcher été.AppImage.home");
        assert_eq!(appimage_home_at(&image),None);
        std::fs::write(&portable_home,"").unwrap();
        assert_eq!(appimage_home_at(&image),None);
        std::fs::remove_file(&portable_home).unwrap();
        std::fs::create_dir(&portable_home).unwrap();
        assert_eq!(appimage_home_at(&image),Some(portable_home));
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test] fn marker_requires_a_file_and_data_follows_the_executable() {
        let dir=std::env::temp_dir().join(format!("portable-test-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&dir).unwrap();let exe=dir.join("Z3-Launcher");
        assert_eq!(portable_at(&exe),None);
        std::fs::create_dir(dir.join("portable.txt")).unwrap();assert_eq!(portable_at(&exe),None);
        std::fs::remove_dir(dir.join("portable.txt")).unwrap();std::fs::write(dir.join("portable.txt"),"").unwrap();
        assert_eq!(portable_at(&exe),Some(dir.join("Z3-Launcher")));
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[cfg(windows)]
fn windows_home() -> PathBuf { std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from).unwrap_or_else(home).join("AppData/Local")) }

pub fn canonicalize(path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    #[cfg(windows)]
    { dunce::canonicalize(path) }
    #[cfg(not(windows))]
    { std::fs::canonicalize(path) }
}

/// Keep packaged GUI libraries out of the game and standalone resource extractor.
pub fn host_command(program: &str) -> std::process::Command {
    let mut command = std::process::Command::new(program);
    #[cfg(windows)]
    crate::windows::hide_console_for_gui(&mut command);
    if std::env::var_os("Z3_APPIMAGE_ENV").is_some() {
        if let Some(path) = std::env::var_os("Z3_HOST_LIBRARY_PATH") {
            command.env("LD_LIBRARY_PATH", path);
        } else { command.env_remove("LD_LIBRARY_PATH"); }
        if std::env::var_os("Z3_XKB_PRELOAD").is_some() {
            if let Some(preload) = std::env::var_os("Z3_HOST_LD_PRELOAD") {
                command.env("LD_PRELOAD", preload);
            } else { command.env_remove("LD_PRELOAD"); }
        }
        command.env_remove("Z3_APPIMAGE_ENV").env_remove("Z3_HOST_LIBRARY_PATH")
            .env_remove("Z3_XKB_PRELOAD").env_remove("Z3_HOST_LD_PRELOAD");
    }
    command
}

pub fn shaders_dir() -> PathBuf { data_dir().join("shaders") }
