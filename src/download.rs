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

const GAME_API: &str = "https://api.github.com/repos/xander-haj/Z3R/releases/latest";
const EXTRACTOR_API: &str =
    "https://api.github.com/repos/legluondunet/zelda3/releases/tags/v0.1.0-dad1b14";
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
    digest: Option<String>,
}
fn client() -> Result<Client> {
    Ok(Client::builder()
        .user_agent("Z3-Launcher")
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(300))
        .build()?)
}
fn bytes(client: &Client, url: &str) -> Result<Vec<u8>> {
    if !url.starts_with("https://github.com/")
        && !url.starts_with("https://api.github.com/")
    {
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
fn release(client: &Client, api: &str) -> Result<Release> {
    let response = client.get(api).send()?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(tr("download.no_release").into());
    }
    Ok(response.error_for_status()?.json()?)
}
fn select_asset<'a>(release: &'a Release, name: &str) -> Result<&'a Asset> {
    release.assets.iter().find(|a| a.name == name)
        .ok_or_else(|| tr("download.no_package").into())
}
fn verify_github_digest(data: &[u8], digest: Option<&str>) -> Result<()> {
    let digest = digest.and_then(|d| d.strip_prefix("sha256:"))
        .ok_or(tr("download.checksum_missing"))?;
    if digest.len() != 64 || !digest.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(tr("download.checksum_missing").into());
    }
    if !format!("{:x}", Sha256::digest(data)).eq_ignore_ascii_case(digest) {
        return Err(tr("download.checksum_failed").into());
    }
    Ok(())
}
fn package_name() -> Result<&'static str> {
    if std::env::consts::ARCH != "x86_64" {
        return Err(tr("download.unsupported").into());
    }
    if cfg!(windows) {
        Ok("Z3R-windows-x64.zip")
    } else if cfg!(target_os = "linux") {
        Ok("Z3R-linux-x64.tar.gz")
    } else {
        Err(tr("download.unsupported").into())
    }
}
pub fn game(destination: &Path, log: &Log) -> Result<String> {
    let name = package_name()?;
    log(tr("download.lookup").into());
    let client = client()?;
    let release = release(&client, GAME_API)?;
    let asset = select_asset(&release, name)?;
    log(tf("download.package",
        &[("version", release.tag_name.clone()), ("name", name.into())]));
    let package = bytes(&client, &asset.browser_download_url)?;
    verify_github_digest(&package, asset.digest.as_deref())?;
    if cfg!(windows) {
        unpack(&package, destination, false)?;
    } else {
        unpack_tar_gz(&package, destination)?;
    }
    validate_game_package(destination)?;
    executable(&destination.join(crate::platform::GAME_BINARY))?;
    Ok(release.tag_name)
}
/// Z3R's published binary archives have no extractor. Until the Z3R
/// extractor is independently packaged, use the verified standalone
/// legacy extractor as a compatibility bridge, never the legacy game binary.
pub fn extractor(destination: &Path, log: &Log) -> Result<()> {
    let name = if cfg!(windows) {
        "zelda3-windows-x86_64.zip"
    } else {
        "zelda3-linux-x86_64.zip"
    };
    log(tr("download.lookup").into());
    let client = client()?;
    let source = release(&client, EXTRACTOR_API)?;
    let asset = select_asset(&source, name)?;
    let package = bytes(&client, &asset.browser_download_url)?;
    verify_github_digest(&package, asset.digest.as_deref())?;
    let scratch = tempfile::tempdir_in(destination.parent().ok_or(tr("download.invalid_archive"))?)?;
    unpack(&package, scratch.path(), false)?;
    let tool_dir = scratch.path().join("extractor");
    let tool = tool_dir.join(if cfg!(windows) { "zelda3-extractor.exe" } else { "zelda3-extractor" });
    if !tool.is_file() {
        return Err(tr("download.extractor_missing").into());
    }
    copy_tree(&tool_dir, &destination.join("extractor"))?;
    executable(&destination.join(extractor_name()))?;
    Ok(())
}
fn validate_game_package(destination: &Path) -> Result<()> {
    for entry in fs::read_dir(destination)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let allowed = name == crate::platform::GAME_BINARY
            || name == "zelda3.ini"
            || name == "LICENSE.txt"
            || name == "readme.md"
            || (cfg!(windows) && name == "SDL2.dll");
        if !allowed || !entry.file_type()?.is_file() {
            return Err(tr("download.invalid_archive").into());
        }
    }
    for required in [crate::platform::GAME_BINARY, "zelda3.ini"] {
        if !destination.join(required).is_file() {
            return Err(tf("download.missing_file", &[("name", required.into())]).into());
        }
    }
    Ok(())
}
pub fn extractor_name() -> &'static str {
    if cfg!(windows) { "extractor/zelda3-extractor.exe" }
    else { "extractor/zelda3-extractor" }
}
/// Extract only the expected root-level files of the upstream Z3R tar.gz.
/// Reject unexpected members, links, devices, duplicates, and unsafe paths.
fn unpack_tar_gz(data: &[u8], destination: &Path) -> Result<()> {
    let mut stream = flate2::read::GzDecoder::new(data);
    let mut total = 0u64;
    let mut seen = std::collections::HashSet::new();
    loop {
        let mut header = [0u8; 512];
        stream.read_exact(&mut header)?;
        if header.iter().all(|&byte| byte == 0) { break; }
        let octal = |field: &[u8]| -> Result<u64> {
            let text = std::str::from_utf8(field)?
                .trim_matches(|c: char| c == '\0' || c == ' ');
            Ok(u64::from_str_radix(text, 8)?)
        };
        let expected = octal(&header[148..156])?;
        let checksum: u64 = header.iter().enumerate().map(|(i, &b)| {
            if (148..156).contains(&i) { b' ' as u64 } else { b as u64 }
        }).sum();
        if checksum != expected { return Err(tr("download.invalid_archive").into()); }
        let name_bytes = header[..100].split(|&b| b == 0).next().unwrap_or_default();
        let raw_name = std::str::from_utf8(name_bytes)?;
        let name = raw_name.strip_prefix("./").unwrap_or(raw_name);
        let size = octal(&header[124..136])?;
        let kind = header[156];
        if (name.is_empty() || name == ".") && kind == b'5' && size == 0 {
            continue;
        }
        if kind != b'0' && kind != 0 {
            return Err(tr("download.invalid_archive").into());
        }
        if !["zelda3", "zelda3.ini", "LICENSE.txt", "readme.md"].contains(&name)
            || !seen.insert(name.to_owned())
        {
            return Err(tr("download.invalid_archive").into());
        }
        total = total.checked_add(size).ok_or(tr("download.too_large"))?;
        if total > 2 * 1024 * 1024 * 1024 {
            return Err(tr("download.too_large").into());
        }
        let mut output = fs::File::create(destination.join(name))?;
        if std::io::copy(&mut stream.by_ref().take(size), &mut output)? != size {
            return Err(tr("download.invalid_archive").into());
        }
        let padding = (512 - size % 512) % 512;
        if padding > 0 {
            let mut padding_bytes = [0u8; 512];
            stream.read_exact(&mut padding_bytes[..padding as usize])?;
        }
    }
    Ok(())
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
    fn z3r_package_accepts_only_expected_files() {
        let dir = tempfile::tempdir().unwrap();
        for name in [crate::platform::GAME_BINARY, "zelda3.ini", "LICENSE.txt", "readme.md"] {
            fs::write(dir.path().join(name), "test").unwrap();
        }
        assert!(validate_game_package(dir.path()).is_ok());
        fs::write(dir.path().join("zelda3_assets.dat"), "unexpected").unwrap();
        assert!(validate_game_package(dir.path()).is_err());
    }
    #[test]
    fn github_digest_and_paths_are_validated() {
        let digest = format!("sha256:{:x}", Sha256::digest(b"ok"));
        assert!(verify_github_digest(b"ok", Some(&digest)).is_ok());
        assert!(verify_github_digest(b"bad", Some(&digest)).is_err());
        assert!(verify_github_digest(b"ok", None).is_err());
        assert!(verify_github_digest(b"ok", Some("sha256:invalid")).is_err());
        for p in ["../escape", "/escape", "a/../b", "C:/file", "a\\b"] {
            assert!(relative(p).is_err());
        }
    }
    #[test]
    fn z3r_tarball_is_extracted_without_allowing_unknown_files() {
        fn tar_member(name: &str, data: &[u8]) -> Vec<u8> {
            let mut header = [0u8; 512];
            header[..name.len()].copy_from_slice(name.as_bytes());
            header[124..136].copy_from_slice(format!("{:011o}\0", data.len()).as_bytes());
            header[148..156].fill(b' ');
            header[156] = b'0';
            let checksum: u64 = header.iter().map(|&b| b as u64).sum();
            header[148..156].copy_from_slice(format!("{:06o}\0 ", checksum).as_bytes());
            let mut result = header.to_vec();
            result.extend_from_slice(data);
            result.resize((result.len() + 511) / 512 * 512, 0);
            result
        }
        fn gz(items: &[(&str, &[u8])]) -> Vec<u8> {
            let mut plain = Vec::new();
            for (name, data) in items {
                plain.extend(tar_member(name, data));
            }
            plain.extend([0u8; 1024]);
            let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(&plain).unwrap();
            encoder.finish().unwrap()
        }
        let root = tempfile::tempdir().unwrap();
        let good = gz(&[("./zelda3", b"game"), ("./zelda3.ini", b"ini")]);
        unpack_tar_gz(&good, root.path()).unwrap();
        assert_eq!(fs::read(root.path().join("zelda3")).unwrap(), b"game");
        assert_eq!(fs::read(root.path().join("zelda3.ini")).unwrap(), b"ini");
        for bad_name in ["../escape", "/escape", "unexpected", "nested/file"] {
            let bad = gz(&[(bad_name, b"bad")]);
            assert!(unpack_tar_gz(&bad, root.path()).is_err());
        }
        let duplicate = gz(&[("zelda3", b"one"), ("zelda3", b"two")]);
        assert!(unpack_tar_gz(&duplicate, root.path()).is_err());
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
