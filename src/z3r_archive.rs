// SPDX-License-Identifier: GPL-3.0-or-later
//! Strict extraction of official Z3R release packages into an EMPTY staging directory.
//! Integrity must have been checked with z3r_source::verify_package before calling.
use crate::core::Result;
use flate2::read::GzDecoder;
use std::{collections::HashSet, fs, io::{self, Read, Write}, path::Path};

const MAX_UNCOMPRESSED: u64 = 128 * 1024 * 1024;

fn allowed(name: &str, windows_package: bool) -> bool {
    if windows_package {
        matches!(name, "zelda3.exe" | "SDL2.dll" | "zelda3.ini" | "readme.md" | "LICENSE.txt")
    } else {
        matches!(name, "zelda3" | "zelda3.ini" | "readme.md" | "LICENSE.txt")
    }
}

fn check_staging(destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    if fs::read_dir(destination)?.next().is_some() {
        return Err("Z3R extraction requires an empty staging directory".into());
    }
    Ok(())
}

fn verify_contents(dir: &Path, windows_package: bool) -> Result<()> {
    let executable = if windows_package { "zelda3.exe" } else { "zelda3" };
    for name in [executable, "zelda3.ini", "LICENSE.txt"] {
        if !dir.join(name).is_file() {
            return Err(format!("Z3R package is missing {name}").into());
        }
    }
    #[cfg(unix)]
    if !windows_package {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(dir.join(executable), fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

fn normalize_entry(name: &str) -> Result<Option<&str>> {
    let name = name.strip_prefix("./").unwrap_or(name);
    if name.is_empty() || name == "." {
        return Ok(None);
    }
    if name.contains('/') || name.contains('\\') || name.contains(':')
        || name == ".." || name.starts_with('.')
    {
        return Err("Z3R archive contains a non-top-level or unsafe path".into());
    }
    Ok(Some(name))
}

pub fn unpack_zip(package: &[u8], destination: &Path) -> Result<()> {
    check_staging(destination)?;
    let mut archive = zip::ZipArchive::new(io::Cursor::new(package))?;
    let mut names = HashSet::new();
    let mut expanded = 0u64;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.is_dir() {
            if normalize_entry(entry.name().trim_end_matches('/'))?.is_none() {
                continue;
            }
            return Err("Unexpected Z3R directory in ZIP".into());
        }
        if entry.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000) {
            return Err("Z3R ZIP symlinks are not allowed".into());
        }
        let name = normalize_entry(entry.name())?
            .ok_or("Unexpected root file in Z3R ZIP")?;
        if !allowed(name, true) || !names.insert(name.to_owned()) {
            return Err(format!("Unexpected or duplicate Z3R ZIP file: {name}").into());
        }
        expanded = expanded.checked_add(entry.size()).ok_or("Z3R archive size overflow")?;
        if expanded > MAX_UNCOMPRESSED {
            return Err("Z3R archive exceeds extraction limit".into());
        }
        let mut output = fs::File::create(destination.join(name))?;
        let written = io::copy(&mut entry, &mut output)?;
        if written != entry.size() {
            return Err("Truncated Z3R ZIP entry".into());
        }
        output.flush()?;
    }
    verify_contents(destination, true)
}

fn tar_string(bytes: &[u8]) -> Result<&str> {
    let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
    Ok(std::str::from_utf8(&bytes[..end])?)
}

fn octal(bytes: &[u8]) -> Result<u64> {
    let s = tar_string(bytes)?.trim();
    if s.is_empty() { return Ok(0); }
    Ok(u64::from_str_radix(s, 8)?)
}

pub fn unpack_tar_gz(package: &[u8], destination: &Path) -> Result<()> {
    check_staging(destination)?;
    let mut reader = GzDecoder::new(package);
    let mut names = HashSet::new();
    let mut expanded = 0u64;
    let mut members = 0usize;
    loop {
        let mut header = [0u8; 512];
        reader.read_exact(&mut header)?;
        if header.iter().all(|b| *b == 0) { break; }
        members += 1;
        if members > 16 {
            return Err("Z3R TAR contains too many entries".into());
        }
        // Restrict to POSIX/GNU ustar entries that fit in a single safe path.
        if !tar_string(&header[345..500])?.is_empty() {
            return Err("Z3R TAR directory prefixes are not allowed".into());
        }
        let raw = tar_string(&header[..100])?;
        let name = normalize_entry(raw.trim_end_matches('/'))?;
        let size = octal(&header[124..136])?;
        let flag = header[156];
        let regular = flag == b'0' || flag == 0;
        if name.is_none() && flag == b'5' && size == 0 {
            continue;
        }
        if !regular || name.is_none() {
            return Err("Unsupported Z3R TAR entry type".into());
        }
        let name = name.unwrap();
        if !allowed(name, false) || !names.insert(name.to_owned()) {
            return Err(format!("Unexpected or duplicate Z3R TAR file: {name}").into());
        }
        expanded = expanded.checked_add(size).ok_or("Z3R archive size overflow")?;
        if expanded > MAX_UNCOMPRESSED {
            return Err("Z3R archive exceeds extraction limit".into());
        }
        let mut output = fs::File::create(destination.join(name))?;
        let copied = io::copy(&mut reader.by_ref().take(size), &mut output)?;
        if copied != size {
            return Err("Truncated Z3R TAR file".into());
        }
        output.flush()?;
        let padding = (512 - (size % 512)) % 512;
        if padding != 0 {
            let skipped = io::copy(&mut reader.by_ref().take(padding), &mut io::sink())?;
            if skipped != padding {
                return Err("Truncated Z3R TAR alignment padding".into());
            }
        }
    }
    verify_contents(destination, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_flat_expected_filenames_are_accepted() {
        for p in ["../escape", "/absolute", "a/b", "C:\\evil", ".hidden"] {
            assert!(normalize_entry(p).is_err());
        }
        assert_eq!(normalize_entry("./zelda3").unwrap(), Some("zelda3"));
        assert!(allowed("zelda3", false));
        assert!(!allowed("zelda3.exe", false));
        assert!(allowed("SDL2.dll", true));
        assert!(!allowed("unapproved.sh", true));
    }
    #[test]
    fn empty_staging_is_required() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("existing"), b"preserve").unwrap();
        assert!(unpack_tar_gz(b"", dir.path()).is_err());
        assert_eq!(fs::read(dir.path().join("existing")).unwrap(), b"preserve");
    }
    #[test]
    fn extracts_expected_windows_zip_files() {
        use zip::write::SimpleFileOptions;
        let mut cursor = io::Cursor::new(Vec::new());
        {
            let mut writer = zip::ZipWriter::new(&mut cursor);
            for (name, contents) in [
                ("zelda3.exe", b"exe".as_slice()),
                ("zelda3.ini", b"[General]".as_slice()),
                ("LICENSE.txt", b"MIT".as_slice()),
                ("SDL2.dll", b"dll".as_slice()),
            ] {
                writer.start_file(name, SimpleFileOptions::default()).unwrap();
                writer.write_all(contents).unwrap();
            }
            writer.finish().unwrap();
        }
        let dir = tempfile::tempdir().unwrap();
        unpack_zip(cursor.get_ref(), dir.path()).unwrap();
        assert_eq!(fs::read(dir.path().join("zelda3.exe")).unwrap(), b"exe");
        assert!(dir.path().join("SDL2.dll").exists());
    }

    fn append_tar_file(tar: &mut Vec<u8>, name: &str, bytes: &[u8]) {
        let mut header = [0u8; 512];
        header[..name.len()].copy_from_slice(name.as_bytes());
        header[100..108].copy_from_slice(b"0000755\0");
        header[108..116].copy_from_slice(b"0000000\0");
        header[116..124].copy_from_slice(b"0000000\0");
        header[124..136].copy_from_slice(format!("{:011o}\0", bytes.len()).as_bytes());
        header[136..148].copy_from_slice(b"00000000000\0");
        header[148..156].fill(b' ');
        header[156] = b'0';
        header[257..263].copy_from_slice(b"ustar\0");
        header[263..265].copy_from_slice(b"00");
        let checksum: u32 = header.iter().map(|&v| v as u32).sum();
        header[148..156].copy_from_slice(format!("{:06o}\0 ", checksum).as_bytes());
        tar.extend_from_slice(&header);
        tar.extend_from_slice(bytes);
        tar.resize((tar.len() + 511) & !511, 0);
    }

    #[test]
    fn extracts_expected_linux_tar_gz_files() {
        use flate2::{Compression, write::GzEncoder};
        let mut tar = Vec::new();
        for (name, bytes) in [
            ("./zelda3", b"elf".as_slice()),
            ("./zelda3.ini", b"[General]".as_slice()),
            ("./LICENSE.txt", b"MIT".as_slice()),
        ] {
            append_tar_file(&mut tar, name, bytes);
        }
        tar.extend_from_slice(&[0u8; 1024]);
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&tar).unwrap();
        let package = encoder.finish().unwrap();
        let dir = tempfile::tempdir().unwrap();
        unpack_tar_gz(&package, dir.path()).unwrap();
        assert_eq!(fs::read(dir.path().join("zelda3")).unwrap(), b"elf");
        assert!(dir.path().join("zelda3.ini").exists());
    }

    #[test]
    #[ignore = "Requires official release archives supplied by the CI workflow"]
    fn official_z3r_release_archives_are_valid_and_extractable() {
        let linux = fs::read(std::env::var("Z3R_TEST_LINUX_TAR_GZ").unwrap()).unwrap();
        let windows = fs::read(std::env::var("Z3R_TEST_WINDOWS_ZIP").unwrap()).unwrap();
        let asset = |name: &str, sha256: &str| crate::z3r_source::Asset {
            name: name.into(),
            browser_download_url: format!(
                "https://github.com/xander-haj/Z3R/releases/download/v0.2.4/{name}"
            ),
            digest: Some(format!("sha256:{sha256}")),
        };
        crate::z3r_source::verify_package(
            &linux,
            &asset(
                "Z3R-linux-x64.tar.gz",
                "e5ccc6eab9732c9bc3d022b938b46a6cd891ad4c85b817477346e6f8f1a71321",
            ),
        ).unwrap();
        crate::z3r_source::verify_package(
            &windows,
            &asset(
                "Z3R-windows-x64.zip",
                "3a44ee3688ffe4e78ed51af25b547f70428d53388394aa032a079a4d45b7ceb7",
            ),
        ).unwrap();
        let linux_dir = tempfile::tempdir().unwrap();
        unpack_tar_gz(&linux, linux_dir.path()).unwrap();
        assert!(linux_dir.path().join("zelda3").is_file());
        let windows_dir = tempfile::tempdir().unwrap();
        unpack_zip(&windows, windows_dir.path()).unwrap();
        assert!(windows_dir.path().join("zelda3.exe").is_file());
    }

    #[test]
    fn mismatched_or_invalid_archives_are_rejected() {
        let a = tempfile::tempdir().unwrap();
        assert!(unpack_tar_gz(b"not gzip", a.path()).is_err());
        let b = tempfile::tempdir().unwrap();
        assert!(unpack_zip(b"not zip", b.path()).is_err());
    }
}
