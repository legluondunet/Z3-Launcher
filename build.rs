// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 legluondunet
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=GITHUB_SHA");
    println!("cargo:rerun-if-changed=.git/HEAD");
    let git_sha = Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|sha| sha.trim().to_owned())
        .filter(|sha| !sha.is_empty());
    let sha = git_sha.or_else(|| {
        std::env::var("GITHUB_SHA").ok().map(|sha| sha.chars().take(7).collect())
    }).unwrap_or_else(|| "unknown".to_owned());
    println!("cargo:rustc-env=Z3_BUILD_COMMIT={sha}");
}
