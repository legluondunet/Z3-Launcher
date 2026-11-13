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
    fn published_asset_schema_can_be_deserialized() {
        let json = r#"{"tag_name":"v0.2.4","assets":[{"name":"Z3R-windows-x64.zip","browser_download_url":"https://github.com/xander-haj/Z3R/releases/download/v0.2.4/Z3R-windows-x64.zip","digest":"sha256:3a44ee3688ffe4e78ed51af25b547f70428d53388394aa032a079a4d45b7ceb7"}]}"#;
        let release: Release = serde_json::from_str(json).unwrap();
        assert!(select_asset(&release, "Z3R-windows-x64.zip").is_ok());
    }
}
