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
    fn mismatched_or_invalid_archives_are_rejected() {
        let a = tempfile::tempdir().unwrap();
        assert!(unpack_tar_gz(b"not gzip", a.path()).is_err());
        let b = tempfile::tempdir().unwrap();
        assert!(unpack_zip(b"not zip", b.path()).is_err());
    }
}
