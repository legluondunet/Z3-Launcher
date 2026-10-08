// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 legluondunet — https://github.com/legluondunet
//! Windows tools run in the MSYS2 UCRT64 environment; the game runs natively.
use std::{path::{Path, PathBuf}, process::Command};
use crate::{core::{Log, Result}, i18n::{tr, tf}};

fn msys_root() -> Result<PathBuf> {
    let root = std::env::var_os("MSYS2_ROOT").map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\msys64"));
    if root.join("usr/bin/bash.exe").is_file() { Ok(root) }
    else { Err(tr("windows.msys_missing").into()) }
}

pub fn command(program: &str, args: &[&str], dir: &Path) -> Result<Command> {
    if Path::new(program).is_absolute() {
        let mut cmd = Command::new(program);
        hide_console_for_gui(&mut cmd);
        cmd.args(args).current_dir(dir);
        // UCRT64 GCC runtime DLLs can be needed by the compiled game.
        if let Ok(root) = msys_root() {
            let mut paths = vec![root.join("ucrt64/bin")];
            if let Some(path) = std::env::var_os("PATH") { paths.extend(std::env::split_paths(&path)); }
            cmd.env("PATH", std::env::join_paths(paths)?);
        }
        return Ok(cmd);
    }
    let root = msys_root()?;
    let mut cmd = Command::new(root.join("usr/bin/bash.exe"));
    hide_console_for_gui(&mut cmd);
    // Arguments remain separate: ROM filenames and Python code are never evaluated as shell code.
    cmd.args(["--noprofile", "--norc", "-c",
        "export PATH=/ucrt64/bin:/usr/bin:$PATH; exec \"$@\"", "Z3-Launcher"])
        .arg(program).args(args).current_dir(dir)
        .env("MSYSTEM", "UCRT64").env("MSYSTEM_PREFIX", "/ucrt64")
        .env("OS", "Windows_NT");
    Ok(cmd)
}

pub fn check(log: &Log) -> Result<()> {
    log(tr("windows.check").into());
    if let Err(error) = msys_root() {
        log(error.to_string()); log("https://www.msys2.org/".into());
        return Err(error);
    }
    let cwd = std::env::current_dir()?;
    let probes: &[(&str, &str, &[&str], &str)] = &[
        ("Git", "git", &["--version"], "git"),
        ("Make", "make", &["--version"], "make"),
        ("GCC", "gcc", &["--version"], "mingw-w64-ucrt-x86_64-gcc"),
        ("windres", "windres", &["--version"], "mingw-w64-ucrt-x86_64-binutils"),
        ("Python", "python3", &["--version"], "mingw-w64-ucrt-x86_64-python"),
        ("Pillow", "python3", &["-c", "import PIL"], "mingw-w64-ucrt-x86_64-python-pillow"),
        ("PyYAML", "python3", &["-c", "import yaml"], "mingw-w64-ucrt-x86_64-python-yaml"),
        ("SDL2", "sdl2-config", &["--version"], "mingw-w64-ucrt-x86_64-SDL2"),
    ];
    let mut missing = std::collections::BTreeSet::new();
    for (name, program, args, package) in probes {
        let result = command(program, args, &cwd)?.output();
        let error = match result {
            Ok(output) if output.status.success() => None,
            Ok(output) => Some(String::from_utf8_lossy(&output.stderr).trim().to_owned()),
            Err(error) => Some(error.to_string()),
        };
        if let Some(error) = error {
            missing.insert(*package);
            log(tf("windows.missing", &[("name", (*name).into()), ("error", error)]));
        } else { log(tf("windows.ok", &[("name", (*name).into())])); }
    }
    if missing.is_empty() { return Ok(()); }
    log(tr("windows.install").into());
    log(format!("pacman -S --needed {}", missing.into_iter().collect::<Vec<_>>().join(" ")));
    Err(tr("text.some_dependencies_are_missing_or_unusable_see_the_report").into())
}

#[cfg(feature = "gui")]
pub fn attach_parent_console_for_cli() {
    if std::env::args_os().nth(1).is_none() { return; }
    #[link(name = "kernel32")]
    extern "system" {
        fn AttachConsole(process_id: u32) -> i32;
    }
    // ATTACH_PARENT_PROCESS: reuse an existing terminal; never create a new console.
    // This call has no pointer arguments. Failure just means no parent console exists.
    unsafe { AttachConsole(u32::MAX); }
}

/// Prevent tool and game subprocesses from opening consoles during a GUI session.
/// stdout/stderr pipes remain available for the launcher's journal.
pub fn hide_console_for_gui(command: &mut Command) {
    if cfg!(feature = "gui") && std::env::args_os().nth(1).is_none() {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
}
