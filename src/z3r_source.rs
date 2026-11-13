// SPDX-License-Identifier: GPL-3.0-or-later
//! Z3R-only GitHub release source. Installation/extraction integration follows
//! once the fork-specific resource pipeline has been validated.
//! Never substitute an arbitrary GitHub repository for this trusted source.
use crate::core::Result;
use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{io::Read, time::Duration};

pub const REPOSITORY: &str = "xander-haj/Z3R";
pub const RELEASES_API: &str = "https://api.github.com/repos/xander-haj/Z3R/releases/latest";
const MAX_PACKAGE: u64 = 128 * 1024 * 1024;

#[derive(Debug, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub assets: Vec<Asset>,
}
#[derive(Debug, Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
    // GitHub Releases publishes sha256:<64 lowercase hex characters>.
    pub digest: Option<String>,
}

pub fn package_name() -> Result<&'static str> {
    if std::env::consts::ARCH != "x86_64" {
        return Err("Z3R packages currently require x86_64".into());
    }
    if cfg!(target_os = "linux") {
        Ok("Z3R-linux-x64.tar.gz")
    } else if cfg!(windows) {
        Ok("Z3R-windows-x64.zip")
    } else {
        Err("Z3R has no supported package for this launcher platform".into())
    }
}

pub fn client() -> Result<Client> {
    Ok(Client::builder()
        .user_agent("Z3-Launcher")
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(300))
        .build()?)
}

pub fn latest(client: &Client) -> Result<Release> {
    let response = client.get(RELEASES_API).send()?.error_for_status()?;
    Ok(response.json()?)
}

pub fn select_asset<'a>(release: &'a Release, name: &str) -> Result<&'a Asset> {
    let asset = release.assets.iter().find(|asset| asset.name == name)
        .ok_or_else(|| format!("Z3R release {} lacks {}", release.tag_name, name))?;
    // Only the official GitHub Release attachment is eligible for download.
    let prefix = format!(
        "https://github.com/{REPOSITORY}/releases/download/{}/",
        release.tag_name
    );
    if !asset.browser_download_url.starts_with(&prefix)
        || asset.browser_download_url.len() <= prefix.len()
        || !valid_digest(asset.digest.as_deref())
    {
        return Err(format!("Z3R release asset {name} has no trusted SHA-256 or URL").into());
    }
    Ok(asset)
}

fn valid_digest(digest: Option<&str>) -> bool {
    let Some(hex) = digest.and_then(|v| v.strip_prefix("sha256:")) else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub fn verify_package(package: &[u8], asset: &Asset) -> Result<()> {
    if !valid_digest(asset.digest.as_deref()) {
        return Err("Z3R package has no valid SHA-256 digest".into());
    }
    let actual = format!("sha256:{:x}", Sha256::digest(package));
    if asset.digest.as_deref() != Some(&actual) {
        return Err(format!("Z3R package {} failed SHA-256 verification", asset.name).into());
    }
    Ok(())
}

/// Fetch and stage the official Z3R game package. Never touches an installed game.
pub fn game(destination: &std::path::Path, log: &crate::core::Log) -> Result<String> {
    let name = package_name()?;
    log(crate::i18n::tr("download.lookup").into());
    let client = client()?;
    let release = latest(&client)?;
    let asset = select_asset(&release, name)?;
    log(crate::i18n::tf("download.package", &[
        ("version", release.tag_name.clone()), ("name", name.to_owned())
    ]));
    let package = download_package(&client, asset)?;
    if cfg!(windows) {
        crate::z3r_archive::unpack_zip(&package, destination)?;
    } else {
        crate::z3r_archive::unpack_tar_gz(&package, destination)?;
    }
    Ok(release.tag_name)
}

pub fn download_package(client: &Client, asset: &Asset) -> Result<Vec<u8>> {
    if !valid_digest(asset.digest.as_deref())
        || !asset.browser_download_url.starts_with(&format!(
            "https://github.com/{REPOSITORY}/releases/download/"
        ))
    {
        return Err("Untrusted Z3R release asset".into());
    }
    let response = client.get(&asset.browser_download_url).send()?.error_for_status()?;
    if response.content_length().is_some_and(|n| n > MAX_PACKAGE) {
        return Err("Z3R package exceeds download size limit".into());
    }
    let mut output = Vec::new();
    response.take(MAX_PACKAGE + 1).read_to_end(&mut output)?;
    if output.len() as u64 > MAX_PACKAGE {
        return Err("Z3R package exceeds download size limit".into());
    }
    verify_package(&output, asset)?;
    Ok(output)
}

// Verified against the exact kAssets_Sig and kNumberOfAssets constants
// in xander-haj/Z3R src/assets.h at migration time. Equal signatures are
// necessary, but not sufficient, to establish gameplay compatibility.
const ASSET_COUNT: usize = 165;
const ASSET_SIGNATURE: [u8; 48] = [
    90, 101, 108, 100, 97, 51, 95, 118, 48, 32, 32, 32, 32, 32, 10, 0,
    27, 174, 233, 45, 74, 174, 252, 50, 49, 27, 153, 197, 27, 43, 216, 197,
    132, 101, 173, 169, 36, 108, 15, 155, 176, 169, 57, 131, 174, 101, 51, 207,
];

pub fn validate_assets_file(data: &[u8]) -> Result<()> {
    let header_len = 88usize + ASSET_COUNT * 4;
    if data.len() < header_len || data.get(..48) != Some(ASSET_SIGNATURE.as_slice()) {
        return Err("Z3R resource file has an incompatible signature".into());
    }
    let read_u32 = |offset: usize| -> Result<usize> {
        let value = data.get(offset..offset + 4)
            .ok_or("Z3R resource file has a truncated header")?;
        Ok(u32::from_le_bytes(value.try_into()?) as usize)
    };
    if read_u32(80)? != ASSET_COUNT {
        return Err("Z3R resource file has an unexpected asset count".into());
    }
    let key_len = read_u32(84)?;
    let mut offset = header_len.checked_add(key_len)
        .ok_or("Z3R resource file has an invalid key table length")?;
    if offset > data.len() {
        return Err("Z3R resource file has a truncated key table".into());
    }
    for index in 0..ASSET_COUNT {
        let size = read_u32(88 + index * 4)?;
        offset = offset.checked_add(3)
            .map(|n| n & !3)
            .ok_or("Z3R resource file has invalid alignment")?;
        offset = offset.checked_add(size)
            .ok_or("Z3R resource file has an invalid asset size")?;
        if offset > data.len() {
            return Err("Z3R resource file has truncated asset data".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release() -> Release {
        let checksum = format!("sha256:{:x}", Sha256::digest(b"z3r package"));
        Release {
            tag_name: "v0.2.4".into(),
            assets: vec![Asset {
                name: "Z3R-linux-x64.tar.gz".into(),
                browser_download_url:
                    "https://github.com/xander-haj/Z3R/releases/download/v0.2.4/Z3R-linux-x64.tar.gz".into(),
                digest: Some(checksum),
            }],
        }
    }

    #[test]
    fn only_expected_z3r_release_assets_are_accepted() {
        let mut release = release();
        assert!(select_asset(&release, "Z3R-linux-x64.tar.gz").is_ok());
        assert!(select_asset(&release, "zelda3-linux-x86_64.zip").is_err());
        release.assets[0].browser_download_url =
            "https://github.com/other/fork/releases/download/v0.2.4/Z3R-linux-x64.tar.gz".into();
        assert!(select_asset(&release, "Z3R-linux-x64.tar.gz").is_err());
        release.assets[0].browser_download_url =
            "https://github.com/xander-haj/Z3R/releases/download/v0.2.4/Z3R-linux-x64.tar.gz".into();
        release.assets[0].digest = None;
        assert!(select_asset(&release, "Z3R-linux-x64.tar.gz").is_err());
    }

    #[test]
    fn package_verification_rejects_modified_or_invalid_content() {
        let release = release();
        let asset = select_asset(&release, "Z3R-linux-x64.tar.gz").unwrap();
        assert!(verify_package(b"z3r package", asset).is_ok());
        assert!(verify_package(b"modified", asset).is_err());
        assert!(!valid_digest(Some("sha256:xyz")));
        assert!(!valid_digest(Some("md5:aaaaaaaa")));
    }

    #[test]
    fn asset_layout_validation_rejects_incompatible_or_truncated_payloads() {
        let mut data = vec![0u8; 88 + ASSET_COUNT * 4];
        data[..48].copy_from_slice(&ASSET_SIGNATURE);
        data[80..84].copy_from_slice(&(ASSET_COUNT as u32).to_le_bytes());
        assert!(validate_assets_file(&data).is_ok());

        data[47] ^= 0xff;
        assert!(validate_assets_file(&data).is_err());
        data[47] ^= 0xff;
        data[80..84].copy_from_slice(&164u32.to_le_bytes());
        assert!(validate_assets_file(&data).is_err());
        data[80..84].copy_from_slice(&(ASSET_COUNT as u32).to_le_bytes());
        data[84..88].copy_from_slice(&1u32.to_le_bytes());
        assert!(validate_assets_file(&data).is_err());
        data[84..88].copy_from_slice(&0u32.to_le_bytes());
        data[88..92].copy_from_slice(&1u32.to_le_bytes());
        assert!(validate_assets_file(&data).is_err());
    }

    #[test]
    fn published_asset_schema_can_be_deserialized() {
        let json = r#"{"tag_name":"v0.2.4","assets":[{"name":"Z3R-windows-x64.zip","browser_download_url":"https://github.com/xander-haj/Z3R/releases/download/v0.2.4/Z3R-windows-x64.zip","digest":"sha256:3a44ee3688ffe4e78ed51af25b547f70428d53388394aa032a079a4d45b7ceb7"}]}"#;
        let release: Release = serde_json::from_str(json).unwrap();
        assert!(select_asset(&release, "Z3R-windows-x64.zip").is_ok());
    }
}
