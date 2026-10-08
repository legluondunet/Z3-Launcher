// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 legluondunet — https://github.com/legluondunet
//! Windows tools run in the MSYS2 UCRT64 environment; the game runs natively.
use std::{path::{Path, PathBuf}, process::{Command, Stdio, ExitStatus}, time::{Duration, Instant}, io};
use crate::{core::{Log, Result}, i18n::{tr, tf}};

pub fn installation_root() -> PathBuf {
    std::env::var_os("MSYS2_ROOT").map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\msys64"))
}
fn msys_root() -> Result<PathBuf> {
    let root = installation_root();
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
            let mut paths = vec![root.join("ucrt64/bin"), root.join("usr/bin")];
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


fn probe_status(command: &mut Command, timeout: Duration) -> io::Result<ExitStatus> {
    use std::os::windows::process::CommandExt;
    command.creation_flags(0x0800_0000)
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    let mut child = command.spawn()?;
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) if start.elapsed() < timeout => std::thread::sleep(Duration::from_millis(25)),
            result => {
                let _ = child.kill();
                let _ = child.wait();
                return match result {
                    Err(error) => Err(error),
                    _ => Err(io::Error::new(io::ErrorKind::TimedOut, tr("windows.probe_timeout"))),
                };
            }
        }
    }
}

pub fn missing_packages(log: &Log) -> Result<Vec<String>> {
    log(tr("windows.check").into());
    let root = match msys_root() {
        Ok(root) => root,
        Err(error) => {
            log(error.to_string()); log("https://www.msys2.org/".into());
            return Err(error);
        }
    };
    let cwd = std::env::current_dir()?;
    let probes: &[(&str, &str, &[&str], &str, &[&str])] = &[
        ("Git", "git", &["--version"], "git", &["usr/bin/git.exe"]),
        ("Make", "make", &["--version"], "make", &["usr/bin/make.exe"]),
        ("GCC", "gcc", &["--version"], "mingw-w64-ucrt-x86_64-gcc", &["ucrt64/bin/gcc.exe"]),
        ("windres", "windres", &["--version"], "mingw-w64-ucrt-x86_64-binutils", &["ucrt64/bin/windres.exe"]),
        ("Python", "python3", &["--version"], "mingw-w64-ucrt-x86_64-python", &["ucrt64/bin/python.exe", "ucrt64/bin/python3.exe"]),
        ("Pillow", "python3", &["-c", "import PIL"], "mingw-w64-ucrt-x86_64-python-pillow", &["ucrt64/bin/python.exe", "ucrt64/bin/python3.exe"]),
        ("PyYAML", "python3", &["-c", "import yaml"], "mingw-w64-ucrt-x86_64-python-yaml", &["ucrt64/bin/python.exe", "ucrt64/bin/python3.exe"]),
        ("SDL2", "sdl2-config", &["--version"], "mingw-w64-ucrt-x86_64-SDL2", &["ucrt64/bin/sdl2-config"]),
    ];
    let mut missing = std::collections::BTreeSet::new();
    let mut python_ok = false;
    for (name, program, args, package, candidates) in probes {
        log(tf("windows.probing", &[("name", (*name).into())]));
        let executable = candidates.iter().map(|p| root.join(p)).find(|p| p.is_file());
        let error = if (*name == "Pillow" || *name == "PyYAML") && !python_ok {
            Some(tr("windows.python_unavailable").to_owned())
        } else if let Some(executable) = executable {
            // Resolve Python directly in UCRT64, never through Windows Store aliases.
            let mut cmd = if *program == "sdl2-config" {
                command(program, args, &cwd)?
            } else {
                command(executable.to_str().ok_or(tr("text.the_executable_path_is_not_utf_8"))?, args, &cwd)?
            };
            cmd.env("PATH", std::env::join_paths([root.join("ucrt64/bin"), root.join("usr/bin")])?);
            match probe_status(&mut cmd, Duration::from_secs(5)) {
                Ok(status) if status.success() => None,
                Ok(status) => Some(tf("windows.probe_failed", &[("status", status.to_string())])),
                Err(error) => Some(error.to_string()),
            }
        } else {
            Some(tr("windows.tool_not_installed").to_owned())
        };
        if *name == "Python" { python_ok = error.is_none(); }
        if let Some(error) = error {
            missing.insert(*package);
            log(tf("windows.missing", &[("name", (*name).into()), ("error", error)]));
        } else { log(tf("windows.ok", &[("name", (*name).into())])); }
    }
    if missing.is_empty() { return Ok(Vec::new()); }
    log(tr("windows.install").into());
    let packages=missing.into_iter().map(str::to_owned).collect::<Vec<_>>();
    log(format!("pacman -S --needed {}", packages.join(" ")));
    Ok(packages)
}

pub fn check(log: &Log) -> Result<()> {
    if missing_packages(log)?.is_empty() { Ok(()) }
    else { Err(tr("text.some_dependencies_are_missing_or_unusable_see_the_report").into()) }
}
pub fn install_plan(log: &Log) -> Result<Option<crate::core::DependencyPlan>> {
    let bootstrap=msys_root().is_err();
    let packages=if bootstrap {
        log(tr("windows.msys_missing").into());
        vec!["git", "make", "mingw-w64-ucrt-x86_64-gcc", "mingw-w64-ucrt-x86_64-binutils",
            "mingw-w64-ucrt-x86_64-python", "mingw-w64-ucrt-x86_64-python-pillow",
            "mingw-w64-ucrt-x86_64-python-yaml", "mingw-w64-ucrt-x86_64-SDL2"]
            .into_iter().map(str::to_owned).collect()
    } else { missing_packages(log)? };
    if packages.is_empty() { return Ok(None); }
    let mut description=tf("install.windows_plan", &[("path", installation_root().display().to_string())]);
    if bootstrap { description.push_str("\n"); description.push_str(tr("install.msys_bootstrap")); }
    Ok(Some(crate::core::DependencyPlan { description, packages,
        family:crate::dependencies::Family::Unknown, bootstrap_msys:bootstrap }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn msys_installer_script_has_valid_powershell_syntax() {
        let mut cmd=Command::new("powershell.exe");
        cmd.args(["-NoProfile", "-NonInteractive", "-Command", "$ErrorActionPreference='Stop'; [void][scriptblock]::Create($env:Z3_TEST_INSTALL_SCRIPT)"])
            .env("Z3_TEST_INSTALL_SCRIPT", include_str!("../tools/install-msys2.ps1"));
        assert!(probe_status(&mut cmd, Duration::from_secs(10)).unwrap().success());
    }
    #[test]
    fn dependency_probe_reports_exit_status() {
        let mut cmd = Command::new("cmd.exe");
        cmd.args(["/C", "exit", "7"]);
        assert_eq!(probe_status(&mut cmd, Duration::from_secs(5)).unwrap().code(), Some(7));
    }
    #[test]
    fn dependency_probe_terminates_an_unresponsive_process() {
        let mut cmd = Command::new("powershell.exe");
        cmd.args(["-NoProfile", "-NonInteractive", "-Command", "Start-Sleep -Seconds 30"]);
        let started = Instant::now();
        let error = probe_status(&mut cmd, Duration::from_millis(200)).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(started.elapsed() < Duration::from_secs(10));
    }
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
