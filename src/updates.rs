// SPDX-License-Identifier: GPL-3.0-or-later
//! Startup checks are independent of downloads and never block the UI.
use reqwest::blocking::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Default)]
pub struct Updates {
    pub game: Option<String>,
    pub launcher: Option<String>,
}
impl Updates {
    pub fn game_available(&self, installed: Option<&str>) -> bool {
        match (&self.game, installed) { (Some(latest), Some(old)) => latest != old.trim(), _ => false }
    }
}
#[derive(Deserialize)]
struct Release { tag_name: String, assets: Vec<Asset> }
#[derive(Deserialize)]
struct Asset { name: String }
fn latest(client: &Client, repo: &str) -> Option<Release> {
    client.get(format!("https://api.github.com/repos/legluondunet/{repo}/releases/latest"))
        .send().ok()?.error_for_status().ok()?.json().ok()
}
pub fn check() -> Updates {
    let Ok(client) = Client::builder().user_agent("Z3-Launcher")
        .connect_timeout(Duration::from_secs(5)).timeout(Duration::from_secs(15)).build()
        else { return Updates::default(); };
    let package = if cfg!(windows) { "zelda3-windows-x86_64.zip" } else { "zelda3-linux-x86_64.zip" };
    let game = latest(&client, "zelda3").filter(|r| std::env::consts::ARCH == "x86_64"
        && r.assets.iter().any(|a| a.name == package)).map(|r| r.tag_name);
    let launcher = latest(&client, "Z3-Launcher").filter(|r| newer(&r.tag_name, env!("CARGO_PKG_VERSION")))
        .map(|r| r.tag_name);
    Updates { game, launcher }
}
fn version(tag: &str) -> Option<(u64,u64,u64)> {
    let mut parts=tag.trim().trim_start_matches('v').split('.');
    let major=parts.next()?.parse().ok()?;
    let minor=parts.next()?.parse().ok()?;
    let patch=parts.next()?.split('+').next()?.parse().ok()?;
    if parts.next().is_some() { return None; }
    Some((major,minor,patch))
}
fn newer(remote: &str, installed: &str) -> bool {
    match (version(remote),version(installed)) { (Some(r),Some(i)) => r>i, _=>false }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn compares_versions_numerically_and_ignores_invalid_tags() {
        assert!(newer("v0.12.0", "0.11.16"));
        assert!(newer("0.11.17", "0.11.9"));
        for tag in ["0.11.16", "0.11.15", "build-deadbeef", "0.12.0-beta", "1.2.3.4"] {
            assert!(!newer(tag,"0.11.16"));
        }
    }
    #[test] fn game_builds_require_an_installed_different_release() {
        let updates=Updates { game: Some("build-new".into()), launcher: None };
        assert!(!updates.game_available(None));
        assert!(!updates.game_available(Some("build-new\n")));
        assert!(updates.game_available(Some("build-old")));
        assert!(!Updates::default().game_available(Some("build-old")));
    }
}
