// SPDX-License-Identifier: GPL-3.0-or-later
//! Public Releases only: no GitHub account, token, Git or build tools required.
use crate::core::{Log, Result};
use crate::i18n::{tf, tr};
use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

const API: &str = "https://api.github.com/repos/legluondunet/zelda3/releases/latest";
const MAX_DOWNLOAD: u64 = 512 * 1024 * 1024;
#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}
#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}
fn client() -> Result<Client> {
    Ok(Client::builder()
        .user_agent("Z3-Launcher")
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(300))
        .build()?)
}
fn bytes(client: &Client, url: &str) -> Result<Vec<u8>> {
    if !url.starts_with("https://github.com/") && !url.starts_with("https://api.github.com/") {
        return Err(tr("download.invalid_url").into());
    }
    let response = client.get(url).send()?.error_for_status()?;
    if response.content_length().is_some_and(|n| n > MAX_DOWNLOAD) {
        return Err(tr("download.too_large").into());
    }
    let mut output = Vec::new();
    response.take(MAX_DOWNLOAD + 1).read_to_end(&mut output)?;
    if output.len() as u64 > MAX_DOWNLOAD {
        return Err(tr("download.too_large").into());
    }
    Ok(output)
}
fn package_name() -> Result<&'static str> {
    if std::env::consts::ARCH != "x86_64" {
        return Err(tr("download.unsupported").into());
    }
    if cfg!(windows) {
        Ok("zelda3-windows-x86_64.zip")
    } else if cfg!(target_os = "linux") {
        Ok("zelda3-linux-x86_64.zip")
    } else {
        Err(tr("download.unsupported").into())
    }
}
fn verify(bytes: &[u8], sums: &str, name: &str) -> Result<()> {
    let expected = sums
        .lines()
        .find_map(|line| {
            let mut parts = line.split_whitespace();
            let hash = parts.next()?;
            let file = parts.next()?.trim_start_matches('*');
            (file == name).then_some(hash)
        })
        .ok_or(tr("download.checksum_missing"))?;
    if expected != format!("{:x}", Sha256::digest(bytes)) {
        return Err(tr("download.checksum_failed").into());
    }
    Ok(())
}
pub fn game(destination: &Path, log: &Log) -> Result<String> {
    let name = package_name()?;
    log(tr("download.lookup").into());
    let client = client()?;
    let response = client.get(API).send()?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(tr("download.no_release").into());
    }
    let release: Release = response.error_for_status()?.json()?;
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == name)
        .ok_or(tr("download.no_package"))?;
    let sums = release
        .assets
        .iter()
        .find(|a| a.name == "SHA256SUMS")
        .ok_or(tr("download.checksum_missing"))?;
    log(tf(
        "download.package",
        &[("version", release.tag_name.clone()), ("name", name.into())],
    ));
    let package = bytes(&client, &asset.browser_download_url)?;
    let checksums = bytes(&client, &sums.browser_download_url)?;
    verify(&package, std::str::from_utf8(&checksums)?, name)?;
    unpack(&package, destination, false)?;
    validate_package(destination)?;
    executable(&destination.join(crate::platform::GAME_BINARY))?;
    executable(&destination.join(extractor_name()))?;
    // PyInstaller shared libraries also need executable mode; upload-artifact drops modes.
    Ok(release.tag_name)
}
fn validate_package(destination: &Path) -> Result<()> {
    // A release may only replace the executable, DLLs, extractor and package notices.
    // Reject unexpected top-level entries before touching an existing installation.
    for entry in fs::read_dir(destination)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let allowed = name == crate::platform::GAME_BINARY
            || name == "extractor"
            || name == "zelda3.ini"
            || name == "LICENSE.txt"
            || name == "LICENSE.upstream.txt"
            || name == "COPYING"
            || name == "VERSION"
            || name == "BUILD-INFO.txt"
            || name == "README.txt"
            || name == "APPIMAGE-README.txt"
            || (cfg!(windows) && name.ends_with(".dll"));
        if !allowed {
            return Err(tr("download.invalid_archive").into());
        }
    }
    for required in [crate::platform::GAME_BINARY, extractor_name(), "zelda3.ini"] {
        if !destination.join(required).is_file() {
            return Err(tf("download.missing_file", &[("name", required.into())]).into());
        }
    }
    Ok(())
}
pub fn extractor_name() -> &'static str {
    if cfg!(windows) {
        "extractor/zelda3-extractor.exe"
    } else {
        "extractor/zelda3-extractor"
    }
}
pub fn executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}
fn relative(name: &str) -> Result<PathBuf> {
    if name.starts_with('/')
        || name.contains('\\')
        || name.contains(':')
        || name.split('/').any(|p| p == "..")
    {
        return Err(tr("download.invalid_archive").into());
    }
    Ok(PathBuf::from(name))
}
fn unpack(data: &[u8], destination: &Path, strip_root: bool) -> Result<()> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(data))?;
    let mut size = 0u64;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index)?;
        let mut path = relative(entry.name())?;
        if entry.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000) {
            return Err(tr("download.invalid_archive").into());
        }
        if strip_root {
            path = path.components().skip(1).collect();
        }
        if path.as_os_str().is_empty() {
            continue;
        }
        size = size
            .checked_add(entry.size())
            .ok_or(tr("download.too_large"))?;
        if size > 2 * 1024 * 1024 * 1024 {
            return Err(tr("download.too_large").into());
        }
        let target = destination.join(path);
        if entry.is_dir() {
            fs::create_dir_all(target)?;
            continue;
        }
        fs::create_dir_all(target.parent().ok_or(tr("download.invalid_archive"))?)?;
        let mut output = fs::File::create(&target)?;
        std::io::copy(&mut entry, &mut output)?;
        output.flush()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                &target,
                fs::Permissions::from_mode(entry.unix_mode().unwrap_or(0o644) & 0o777),
            )?;
        }
    }
    Ok(())
}
pub fn shaders(destination: &Path, log: &Log) -> Result<()> {
    if ["stock.glsl", "nearest.glslp", "bilinear.glslp"]
        .iter()
        .all(|n| destination.join(n).is_file())
    {
        return Ok(());
    }
    log(tf(
        "shaders.check",
        &[("path", destination.display().to_string())],
    ));
    let parent = destination.parent().ok_or(tr("download.invalid_archive"))?;
    fs::create_dir_all(parent)?;
    let temp = tempfile::tempdir_in(parent)?;
    let staged = temp.path().join("shaders");
    fs::create_dir(&staged)?;
    if destination.exists() {
        copy_tree(destination, &staged)?;
    }
    unpack(
        &bytes(
            &client()?,
            "https://github.com/libretro/glsl-shaders/archive/refs/heads/master.zip",
        )?,
        &staged,
        true,
    )?;
    for name in ["stock.glsl", "nearest.glslp", "bilinear.glslp"] {
        if !staged.join(name).is_file() {
            return Err(tf("download.missing_file", &[("name", name.into())]).into());
        }
    }
    let old = temp.path().join("old");
    if destination.exists() {
        fs::rename(destination, &old)?;
    }
    if let Err(e) = fs::rename(&staged, destination) {
        if old.exists() {
            fs::rename(old, destination)?;
        }
        return Err(e.into());
    }
    Ok(())
}
pub fn copy_tree(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_symlink() {
            return Err(tr("download.invalid_archive").into());
        }
        let dest = target.join(entry.file_name());
        if ty.is_dir() {
            copy_tree(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), dest)?;
        }
    }
    Ok(())
}
/// Replace only files/directories owned by the package. Saves, MSU and config survive.
/// If any rename fails (e.g. a running Windows game), undo all replacements.
pub fn install(staged: &Path, destination: &Path, backup: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    fs::create_dir_all(backup)?;
    let mut moved: Vec<(PathBuf, bool)> = Vec::new();
    let operation = (|| -> Result<()> {
        let mut entries = fs::read_dir(staged)?.collect::<std::io::Result<Vec<_>>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let name = entry.file_name();
            if name == "zelda3.ini" && destination.join(&name).exists() {
                continue;
            }
            let exists = destination.join(&name).exists();
            if exists {
                fs::rename(destination.join(&name), backup.join(&name))?;
            }
            moved.push((PathBuf::from(&name), exists));
            fs::rename(entry.path(), destination.join(&name))?;
        }
        Ok(())
    })();
    if let Err(error) = operation {
        for (name, existed) in moved.into_iter().rev() {
            let path = destination.join(&name);
            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            } else if path.exists() {
                fs::remove_file(&path)?;
            }
            if existed {
                fs::rename(backup.join(name), path)?;
            }
        }
        return Err(error);
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn release_notices_are_accepted_but_unexpected_files_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("extractor")).unwrap();
        for name in [crate::platform::GAME_BINARY, extractor_name(), "zelda3.ini",
            "LICENSE.txt", "LICENSE.upstream.txt", "COPYING", "VERSION", "BUILD-INFO.txt",
            "README.txt", "APPIMAGE-README.txt"] {
            fs::write(dir.path().join(name), "test").unwrap();
        }
        assert!(validate_package(dir.path()).is_ok());
        fs::write(dir.path().join("zelda3_assets.dat"), "unexpected").unwrap();
        assert!(validate_package(dir.path()).is_err());
    }
    #[test]
    fn checksum_and_paths_are_validated() {
        assert!(verify(
            b"ok",
            &format!("{:x}  p.zip\n", Sha256::digest(b"ok")),
            "p.zip"
        )
        .is_ok());
        assert!(verify(
            b"bad",
            &format!("{:x}  p.zip\n", Sha256::digest(b"ok")),
            "p.zip"
        )
        .is_err());
        for p in ["../escape", "/escape", "a/../b", "C:/file", "a\\b"] {
            assert!(relative(p).is_err());
        }
    }
    #[test]
    fn installation_preserves_configuration_saves_and_resources() {
        let temp = tempfile::tempdir().unwrap();
        let staged = temp.path().join("stage");
        let dest = temp.path().join("game");
        fs::create_dir(&staged).unwrap();
        fs::create_dir(&dest).unwrap();
        fs::write(staged.join("zelda3.ini"), "default").unwrap();
        fs::write(staged.join("program"), "new").unwrap();
        fs::write(dest.join("zelda3.ini"), "custom").unwrap();
        fs::write(dest.join("save.srm"), "save").unwrap();
        install(&staged, &dest, &temp.path().join("backup")).unwrap();
        assert_eq!(
            fs::read_to_string(dest.join("zelda3.ini")).unwrap(),
            "custom"
        );
        assert_eq!(fs::read_to_string(dest.join("save.srm")).unwrap(), "save");
    }
    #[test]
    fn failed_replacement_restores_already_replaced_files() {
        let temp = tempfile::tempdir().unwrap();
        let staged = temp.path().join("stage");
        let dest = temp.path().join("game");
        let backup = temp.path().join("backup");
        fs::create_dir(&staged).unwrap();
        fs::create_dir(&dest).unwrap();
        fs::create_dir_all(backup.join("b")).unwrap();
        for name in ["a", "b"] {
            fs::write(staged.join(name), "new").unwrap();
            fs::write(dest.join(name), "old").unwrap();
        }
        assert!(install(&staged, &dest, &backup).is_err());
        for name in ["a", "b"] {
            assert_eq!(fs::read_to_string(dest.join(name)).unwrap(), "old");
        }
    }
    #[test]
    fn zip_traversal_is_rejected() {
        let mut data = std::io::Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut data);
            zip.start_file("../escape", zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(b"bad").unwrap();
            zip.finish().unwrap();
        }
        let dir = tempfile::tempdir().unwrap();
        assert!(unpack(data.get_ref(), dir.path(), false).is_err());
    }
}
