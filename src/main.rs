// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 legluondunet — https://github.com/legluondunet
#![cfg_attr(all(windows, feature = "gui"), windows_subsystem = "windows")]

#[cfg(windows)]
mod windows;
use crate::i18n::{tr, tf};
mod core;
mod single_instance;
mod i18n;
mod platform;
mod download;
mod updates;
mod ini;
#[cfg(feature = "gui")]
mod ui;
#[cfg(feature = "gui")]
mod options;
use std::{path::PathBuf, sync::Arc};
fn main() {
    #[cfg(all(windows, feature = "gui"))]
    windows::attach_parent_console_for_cli();
    i18n::init();
    if let Err(e) = start() { eprintln!("{}", tf("status.error", &[("error", e.to_string())])); std::process::exit(1); }
}
fn start() -> core::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        #[cfg(feature = "gui")]
        {
            let Some(instance) = single_instance::acquire()? else { return Ok(()); };
            return ui::start(instance).map_err(|e| e.to_string().into());
        }
        #[cfg(not(feature = "gui"))]
        { help(); return Ok(()); }
    }
    if matches!(args[0].as_str(), "--help" | "-h") {
        help();
        return Ok(());
    }
    if matches!(args[0].as_str(), "--version" | "-V") {
        println!("{}", version_string());
        return Ok(());
    }

    let Some(cli) = parse_cli(&args)? else {
        help();
        return Ok(());
    };
    let launcher = core::Launcher::new(cli.root)?;
    let log = launcher.log(Arc::new(|line| println!("{line}")))?;
    if cli.verbose {
        log(format!("[verbose] {}", version_string()));
        log(format!(
            "[verbose] Platform: {} / {}",
            std::env::consts::OS, std::env::consts::ARCH
        ));
        log(format!("[verbose] Command: {}", cli.action));
        log(format!("[verbose] Portable mode: {}", crate::platform::is_portable()));
        log(format!("[verbose] Workspace: {}", launcher.root.display()));
        log(format!("[verbose] Game directory: {}", launcher.repo().display()));
        log(format!(
            "[verbose] Game executable: {}",
            launcher.repo().join(crate::platform::GAME_BINARY).display()
        ));
        if let Some(ref rom) = cli.rom {
            log(format!("[verbose] ROM: {}", rom.display()));
        }
        log(format!(
            "[verbose] Launcher journal: {}",
            launcher.root.join(core::LOG_FILENAME).display()
        ));
        log(format!(
            "[verbose] Game journal: {}",
            launcher.root.join(core::GAME_LOG_FILENAME).display()
        ));
    }
    launcher.action_with_verbosity(
        &cli.action, cli.rom.as_deref(), &log, cli.verbose,
    )
}

struct Cli {
    action: String,
    root: PathBuf,
    rom: Option<PathBuf>,
    verbose: bool,
}

/// Accept --verbose before or after the command, and keep --dir/--rom semantics.
fn parse_cli(args: &[String]) -> core::Result<Option<Cli>> {
    let mut cli = Cli {
        action: String::new(),
        root: default_root(),
        rom: None,
        verbose: false,
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--verbose" => cli.verbose = true,
            "--dir" => {
                index += 1;
                cli.root = PathBuf::from(
                    args.get(index).ok_or(tr("text.missing_value_for_dir"))?
                );
            }
            "--rom" => {
                index += 1;
                cli.rom = Some(PathBuf::from(
                    args.get(index).ok_or(tr("text.missing_value_for_rom"))?
                ));
            }
            action if cli.action.is_empty() && !action.starts_with('-') => {
                cli.action = action.to_owned();
            }
            arg => return Err(tf(
                "cli.unknown_argument", &[("argument", arg.to_owned())],
            ).into()),
        }
        index += 1;
    }
    Ok((!cli.action.is_empty()).then_some(cli))
}

fn version_string() -> String {
    let version = env!("CARGO_PKG_VERSION");
    if version.contains('+') {
        format!("Z3-Launcher {version}")
    } else {
        format!("Z3-Launcher {version}+{}", env!("Z3_BUILD_COMMIT"))
    }
}

fn default_root() -> PathBuf { platform::data_dir() }
fn help() { println!("{}", tr("cli.help")); }

#[cfg(test)]
mod cli_tests {
    use super::*;

    fn args(strings: &[&str]) -> Vec<String> {
        strings.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn version_includes_package_version_and_commit() {
        let value = version_string();
        assert!(value.starts_with(&format!("Z3-Launcher {}", env!("CARGO_PKG_VERSION"))));
        assert!(!value.trim().is_empty());
    }

    #[test]
    fn verbose_is_accepted_before_or_after_the_action() {
        for flags in [
            args(&["--verbose", "status"]),
            args(&["status", "--verbose"]),
            args(&["status", "--dir", "Test Folder", "--verbose"]),
        ] {
            let cli = parse_cli(&flags).unwrap().unwrap();
            assert_eq!(cli.action, "status");
            assert!(cli.verbose);
        }
    }

    #[test]
    fn setup_preserves_rom_and_working_directory() {
        let cli = parse_cli(&args(&[
            "setup", "--rom", "My ROM.sfc", "--verbose", "--dir", "My Games",
        ])).unwrap().unwrap();
        assert_eq!(cli.action, "setup");
        assert!(cli.verbose);
        assert_eq!(cli.rom.as_deref(), Some(std::path::Path::new("My ROM.sfc")));
        assert_eq!(cli.root, PathBuf::from("My Games"));
    }

    #[test]
    fn unknown_arguments_and_missing_values_are_errors() {
        assert!(parse_cli(&args(&["run", "--verify"])).is_err());
        assert!(parse_cli(&args(&["setup", "--rom"])).is_err());
        assert!(parse_cli(&args(&["run", "--dir"])).is_err());
    }
}

#[cfg(feature = "gui")]
mod theme;
