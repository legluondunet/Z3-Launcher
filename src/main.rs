// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 legluondunet — https://github.com/legluondunet
#[cfg(windows)]
mod windows;
use crate::i18n::{tr, tf};
mod core;
mod i18n;
mod platform;
mod dependencies;
mod ini;
#[cfg(feature = "gui")]
mod ui;
#[cfg(feature = "gui")]
mod options;
use std::{path::PathBuf, sync::Arc};
fn main() {
    i18n::init();
    if let Err(e) = start() { eprintln!("{}", tf("status.error", &[("error", e.to_string())])); std::process::exit(1); }
}
fn start() -> core::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        #[cfg(feature = "gui")]
        { return ui::start().map_err(|e| e.to_string().into()); }
        #[cfg(not(feature = "gui"))]
        { help(); return Ok(()); }
    }
    if args[0] == "--help" || args[0] == "-h" { help(); return Ok(()); }
    let action = &args[0]; let mut root = default_root(); let mut rom = None; let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--dir" => { i += 1; root = PathBuf::from(args.get(i).ok_or(tr("text.missing_value_for_dir"))?); },
            "--rom" => { i += 1; rom = Some(PathBuf::from(args.get(i).ok_or(tr("text.missing_value_for_rom"))?)); },
            arg => return Err(tf("cli.unknown_argument", &[("argument", arg.to_owned())]).into()),
        }
        i += 1;
    }
    let launcher = core::Launcher::new(root)?;
    let log = launcher.log(Arc::new(|line| println!("{line}")))?;
    launcher.action(action, rom.as_deref(), &log)
}
fn default_root() -> PathBuf { platform::data_dir() }
fn help() { println!("{}", tr("cli.help")); }

#[cfg(feature = "gui")]
mod theme;
