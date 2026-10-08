// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 legluondunet — https://github.com/legluondunet
use crate::i18n::{tr, tf};
use std::{fs, io::{BufRead, BufReader, Write}, path::{Path, PathBuf}, process::{Command, Stdio}, sync::{Arc, Mutex}, thread};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub const ROM_HASH: &str = "66871d66be19ad2c34c927d6b14cd8eb6fc3181965b6e517cb361f7316009cfb";
pub type Log = Arc<dyn Fn(String) + Send + Sync>;
#[derive(Clone)]
pub struct DependencyPlan {
    pub description: String,
    pub packages: Vec<String>,
    pub family: crate::dependencies::Family,
    pub bootstrap_msys: bool,
}
#[derive(Clone)]
pub struct Launcher { pub root: PathBuf }
impl Launcher {
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let portable_root = crate::platform::portable_dir();
        let root = portable_root.as_deref().unwrap_or_else(|| root.as_ref());
        let root = if root.is_absolute() { root.to_path_buf() } else { std::env::current_dir()?.join(root) };
        fs::create_dir_all(&root)?;
        Ok(Self { root: crate::platform::canonicalize(root)? })
    }
    pub fn repo(&self) -> PathBuf { self.root.join("zelda3") }
    pub fn log(&self, callback: Log) -> Result<Log> {
        let file = Arc::new(Mutex::new(fs::OpenOptions::new().create(true).append(true).open(self.root.join("launcher.log"))?));
        Ok(Arc::new(move |line| {
            if let Ok(mut f) = file.lock() { let _ = writeln!(f, "{line}"); }
            callback(line);
        }))
    }
    pub fn command(&self, program: &str, args: &[&str], dir: &Path, log: &Log) -> Result<()> {
        log(tf("process.command", &[("program", program.to_owned()), ("args", format!("{args:?}")), ("path", dir.display().to_string())]));
        let command = Self::process(program, args, dir)?;
        run_logged(command, program, log)
    }
    fn process(program: &str, args: &[&str], dir: &Path) -> Result<Command> {
        #[cfg(windows)]
        { crate::windows::command(program, args, dir) }
        #[cfg(not(windows))]
        { let mut cmd = crate::platform::host_command(program); cmd.args(args).current_dir(dir); Ok(cmd) }
    }
    pub fn check(&self, log: &Log) -> Result<()> {
        #[cfg(windows)]
        { return crate::windows::check(log); }
        #[cfg(not(windows))]
        {
        let report = crate::dependencies::inspect();
        for line in report.lines() { log(line.to_owned()); }
        if report.missing.is_empty() { Ok(()) } else { Err(tr("text.some_dependencies_are_missing_or_unusable_see_the_report").into()) }
        }
    }
    pub fn dependency_plan(&self, log: &Log) -> Result<Option<DependencyPlan>> {
        #[cfg(windows)]
        { crate::windows::install_plan(log) }
        #[cfg(not(windows))]
        {
            let report=crate::dependencies::inspect();
            for line in report.lines() { log(line); }
            if report.missing.is_empty() { return Ok(None); }
            if report.distribution.family == crate::dependencies::Family::Unknown {
                return Err(tr("text.unsupported_distribution_install_these_dependencies_using_its_package_manager").into());
            }
            let description=crate::dependencies::install_command(report.distribution.family, &report.missing)
                .ok_or(tr("install.unavailable"))?;
            Ok(Some(DependencyPlan { description,
                packages:crate::dependencies::package_names(report.distribution.family, &report.missing),
                family:report.distribution.family, bootstrap_msys:false }))
        }
    }
    pub fn install_dependencies(&self, plan: &DependencyPlan, log: &Log) -> Result<()> {
        if plan.packages.is_empty() { return Ok(()); }
        log(tr("install.started").into());
        #[cfg(windows)]
        {
            let root=crate::windows::installation_root();
            if plan.bootstrap_msys {
                if root.exists() { return Err(tr("install.msys_existing").into()); }
                log(tf("install.bootstrap_progress", &[("path", root.display().to_string())]));
                let script=include_str!("../tools/install-msys2.ps1");
                // Pass the destination via an environment variable rather than shell interpolation.
                let mut command=crate::platform::host_command("powershell.exe");
                command.args(["-NoProfile", "-NonInteractive", "-Command", script])
                    .env("Z3_MSYS2_ROOT", &root);
                run_logged(command, "MSYS2 installer", log)?;
                if !root.join("usr/bin/pacman.exe").is_file() { return Err(tr("windows.msys_missing").into()); }
            }
            let pacman=root.join("usr/bin/pacman.exe");
            let program=pacman.to_str().ok_or(tr("text.the_executable_path_is_not_utf_8"))?;
            // Core MSYS2 updates may require a second invocation after the first exits.
            self.command(program, &["-Syu", "--noconfirm"], &self.root, log)?;
            self.command(program, &["-Syu", "--noconfirm"], &self.root, log)?;
            let mut args=vec!["-S", "--needed", "--noconfirm"];
            args.extend(plan.packages.iter().map(String::as_str));
            self.command(program, &args, &self.root, log)?;
        }
        #[cfg(not(windows))]
        {
            // pkexec delegates authentication to the desktop; the launcher never reads a password.
            let script=match plan.family {
                crate::dependencies::Family::Debian => "/usr/bin/apt-get update && exec /usr/bin/apt-get install -y \"$@\"",
                crate::dependencies::Family::Arch => "exec /usr/bin/pacman -S --needed --noconfirm \"$@\"",
                crate::dependencies::Family::Fedora => "exec /usr/bin/dnf -y install \"$@\"",
                crate::dependencies::Family::Suse => "exec /usr/bin/zypper --non-interactive install \"$@\"",
                crate::dependencies::Family::Unknown => return Err(tr("install.unavailable").into()),
            };
            let mut args=vec!["/bin/sh", "-c", script, "Z3-Launcher"];
            args.extend(plan.packages.iter().map(String::as_str));
            self.command("pkexec", &args, &self.root, log)?;
        }
        log(tr("install.completed").into());
        Ok(())
    }
    pub fn clone_repo(&self, log: &Log) -> Result<()> {
        if self.repo().join(".git").exists() { return Ok(()); }
        if self.repo().exists() { return Err(tr("text.the_zelda3_directory_exists_but_is_not_a_git").into()); }
        self.command("git", &["clone", "--recursive", "https://github.com/snesrev/zelda3.git", "zelda3"], &self.root, log)
    }
    pub fn assets(&self) -> Result<PathBuf> {
        for name in ["assets", "tables"] { let d = self.repo().join(name); if d.join("restool.py").is_file() { return Ok(d); } }
        Err(tr("text.restool_py_was_not_found_in_assets_or_tables").into())
    }
    pub fn verify_rom(path: &Path) -> Result<()> {
        let path = crate::platform::canonicalize(path)?;
        #[cfg(windows)]
        {
            use sha2::{Digest, Sha256};
            let hash = format!("{:x}", Sha256::digest(fs::read(&path)?));
            if hash != ROM_HASH { return Err(tf("rom.incompatible", &[("hash", ROM_HASH.to_owned())]).into()); }
        }
        #[cfg(not(windows))]
        {
        let output = crate::platform::host_command("sha256sum").arg("--").arg(&path).output()?;
        if !output.status.success() { return Err(tr("text.could_not_calculate_the_rom_sha256").into()); }
        if String::from_utf8_lossy(&output.stdout).split_whitespace().next() != Some(ROM_HASH) {
            return Err(tf("rom.incompatible", &[("hash", ROM_HASH.to_owned())]).into());
        }
        }
        Ok(())
    }
    pub fn install_rom(&self, path: &Path, log: &Log) -> Result<()> {
        Self::verify_rom(path)?;
        let dest = self.repo().join("zelda3.sfc");
        if crate::platform::canonicalize(path)? != dest { fs::copy(path, &dest)?; }
        // Support historical tables/ and current assets/ scripts.
        let local = self.assets()?.join("zelda3.sfc");
        if local != dest { fs::copy(&dest, local)?; }
        log(tr("text.us_rom_verified_and_installed").into()); Ok(())
    }
    pub fn has_language_resources(&self, code: &str) -> bool {
        self.assets().is_ok_and(|a| a.join(format!("dialogue_{code}.txt")).is_file() && a.join(format!("font_{code}.png")).is_file())
    }
    fn resource_languages(&self) -> Result<String> {
        let assets = self.assets()?;
        Ok(["de", "fr", "fr-c", "en", "es", "pl", "pt", "redux", "nl", "sv"].into_iter()
            .filter(|code| assets.join(format!("dialogue_{code}.txt")).is_file() && assets.join(format!("font_{code}.png")).is_file())
            .collect::<Vec<_>>().join(","))
    }
    pub fn import_language(&self, code: &str, rom: Option<&Path>, log: &Log) -> Result<()> {
        if !["de", "fr", "fr-c", "en", "es", "pl", "pt", "redux", "nl", "sv"].contains(&code) {
            return Err(tr("game_language.unsupported").into());
        }
        Self::verify_rom(&self.repo().join("zelda3.sfc"))?;
        let assets = self.assets()?;
        let paths = [assets.join(format!("dialogue_{code}.txt")), assets.join(format!("font_{code}.png")), self.repo().join("zelda3_assets.dat")];
        let backups: Vec<Option<Vec<u8>>> = paths.iter().map(|p| {
            if p.exists() { fs::read(p).map(Some) } else { Ok(None) }
        }).collect::<std::io::Result<_>>()?;
        let operation = (|| -> Result<()> {
            if let Some(rom) = rom {
                let rom = crate::platform::canonicalize(rom)?;
                let rom = rom.to_str().ok_or(tr("text.the_rom_path_must_be_utf_8"))?;
                // Use the repository's own hashes, including supported fan translations.
                let verify = "import sys,util\nr=util.load_rom(sys.argv[1], True)\nif r.language != sys.argv[2]: raise ValueError('ROM language mismatch: expected '+sys.argv[2]+', got '+str(r.language))";
                self.command("python3", &["-c", verify, rom, code], &assets, log)?;
                self.command("python3", &["restool.py", "--extract-dialogue", "--rom", rom], &assets, log)?;
            }
            if !self.has_language_resources(code) { return Err(tr("game_language.resources_missing").into()); }
            let flag = format!("--languages={}", self.resource_languages()?);
            self.command("python3", &["restool.py", &flag], &assets, log)?;
            if !self.repo().join("zelda3_assets.dat").is_file() { return Err(tr("game_language.resources_missing").into()); }
            log(tf("game_language.imported", &[("language", code.to_owned())]));
            Ok(())
        })();
        if let Err(error) = operation {
            for (path, backup) in paths.iter().zip(backups) {
                let rollback = match backup { Some(bytes) => fs::write(path, bytes), None if path.exists() => fs::remove_file(path), None => Ok(()) };
                if let Err(rollback_error) = rollback {
                    return Err(tf("game_language.rollback_failed", &[("error", error.to_string()), ("rollback", rollback_error.to_string())]).into());
                }
            }
            return Err(error);
        }
        Ok(())
    }
    pub fn build(&self, log: &Log) -> Result<()> {
        Self::verify_rom(&self.repo().join("zelda3.sfc"))?;
        let assets = self.assets()?;
        fs::copy(self.repo().join("zelda3.sfc"), assets.join("zelda3.sfc"))?;
        let languages = self.resource_languages()?;
        let language_flag = format!("--languages={languages}");
        let mut arguments = vec!["restool.py", "--extract-from-rom"];
        if !languages.is_empty() { arguments.push(&language_flag); }
        self.command("python3", &arguments, &assets, log)?;
        let jobs = format!("-j{}", thread::available_parallelism().map(|n| n.get()).unwrap_or(1));
        #[cfg(not(windows))]
        self.command("make", &[&jobs], &self.repo(), log)?;
        #[cfg(windows)]
        self.command("make", &[&jobs, "CC=gcc", "OS=Windows_NT", "TARGET_EXEC=zelda3.exe"], &self.repo(), log)?;
        if !self.repo().join(crate::platform::GAME_BINARY).is_file() { return Err(tr("text.make_finished_but_the_zelda3_executable_was_not_found").into()); }
        Ok(())
    }
    fn ensure_shaders(&self, log: &Log) -> Result<()> {
        let destination = crate::platform::shaders_dir();
        log(tf("shaders.check", &[("path", destination.display().to_string())]));
        let parent = destination.parent().ok_or("Shader directory has no parent")?;
        fs::create_dir_all(parent)?;
        let script = include_str!("../tools/download-shaders.py");
        self.command("python3", &["-c", script, destination.to_str().ok_or(tr("text.the_rom_path_must_be_utf_8"))?], parent, log)
    }
    fn sync_repo(&self, log: &Log) -> Result<()> {
        log(tr("setup.check_updates").into());
        // Fast-forward only: never reset or overwrite divergent/local changes.
        self.command("git", &["pull", "--ff-only"], &self.repo(), log)?;
        self.command("git", &["submodule", "update", "--init", "--recursive"], &self.repo(), log)
    }
    fn clean_compiled(&self, log: &Log) -> Result<()> {
        log(tr("setup.clean_compiled").into());
        // Do not use `make clean`: clean_gen also deletes localized dialogue/fonts.
        #[cfg(not(windows))]
        self.command("make", &["clean_obj"], &self.repo(), log)?;
        #[cfg(windows)]
        self.command("make", &["clean_obj", "CC=gcc", "OS=Windows_NT", "TARGET_EXEC=zelda3.exe"], &self.repo(), log)?;
        #[cfg(windows)]
        {
            // clean_obj does not remove the compiled Windows resource in upstream's Makefile.
            let resource = self.repo().join("zelda3.res");
            if resource.exists() { fs::remove_file(resource)?; }
        }
        Ok(())
    }
    pub fn update(&self, log: &Log) -> Result<()> {
        if !self.repo().join(".git").exists() { return Err(tr("text.install_the_repository_first").into()); }
        self.sync_repo(log)?;
        self.build(log)
    }
    pub fn run(&self, log: &Log) -> Result<()> {
        let bin = self.repo().join(crate::platform::GAME_BINARY);
        if !bin.is_file() { return Err(tr("text.build_the_game_first").into()); }
        self.command(bin.to_str().ok_or(tr("text.the_executable_path_is_not_utf_8"))?, &[], &self.repo(), log)
    }
    pub fn action(&self, action: &str, rom: Option<&Path>, log: &Log) -> Result<()> {
        match action {
            "check" => self.check(log),
            "setup" => {
                let rom = rom.ok_or(tr("text.select_a_us_rom"))?;
                Self::verify_rom(rom)?;
                self.check(log)?;
                if self.repo().join(".git").exists() { self.sync_repo(log)?; }
                else { self.clone_repo(log)?; }
                self.ensure_shaders(log)?;
                self.install_rom(rom, log)?;
                self.clean_compiled(log)?;
                self.build(log)
            },
            "build" => self.build(log), "update" => self.update(log), "run" => self.run(log),
            "status" => { log(tf("launcher.status", &[("path", self.root.display().to_string()), ("repo", self.repo().join(".git").exists().to_string()), ("rom", self.repo().join("zelda3.sfc").is_file().to_string()), ("binary", self.repo().join(crate::platform::GAME_BINARY).is_file().to_string())])); Ok(()) },
            _ => Err(tf("cli.unknown_command", &[("action", action.to_owned())]).into())
        }
    }
}
fn run_logged(mut command: Command, program: &str, log: &Log) -> Result<()> {
    let mut child=command.env("PYTHONUNBUFFERED", "1").stdin(Stdio::null())
        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    let out=child.stdout.take().ok_or(tr("text.stdout_is_unavailable"))?;
    let err=child.stderr.take().ok_or(tr("text.stderr_is_unavailable"))?;
    let l=log.clone();
    let a=thread::spawn(move || { for line in BufReader::new(out).lines() { match line { Ok(line)=>l(line), Err(_)=>break } } });
    let l=log.clone();
    let b=thread::spawn(move || { for line in BufReader::new(err).lines() { match line { Ok(line)=>l(line), Err(_)=>break } } });
    let status=child.wait()?; let _=a.join(); let _=b.join();
    if !status.success() { return Err(tf("process.failed", &[("program", program.to_owned()), ("status", status.to_string())]).into()); }
    Ok(())
}
pub fn save_ini(path: &Path, text: &str) -> Result<()> {
    if !path.is_file() { return Err(tr("text.load_an_existing_ini_file_first").into()); }
    fs::copy(path, path.with_extension("ini.bak"))?;
    let temp = path.with_extension("ini.tmp");
    fs::write(&temp, text)?;
    fs::rename(temp, path)?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn temp() -> PathBuf { std::env::temp_dir().join(format!("zelda-test-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())) }
    #[test] fn ini_backup_preserves_original() {
        let d = temp(); fs::create_dir_all(&d).unwrap(); let p = d.join("zelda3.ini");
        fs::write(&p, "# comment\n[Graphics]\nFullscreen=0\n").unwrap();
        save_ini(&p, "[Graphics]\nFullscreen=1\n").unwrap();
        assert!(fs::read_to_string(p.with_extension("ini.bak")).unwrap().contains("# comment"));
        assert!(fs::read_to_string(p).unwrap().contains("Fullscreen=1")); fs::remove_dir_all(d).unwrap();
    }
    #[test] fn refuses_non_repository_folder() {
        let d = temp(); let l = Launcher::new(&d).unwrap(); fs::create_dir(l.repo()).unwrap();
        let log: Log = Arc::new(|_| {}); assert!(l.clone_repo(&log).is_err()); fs::remove_dir_all(d).unwrap();
    }
    #[cfg(unix)]
    #[test] fn command_failure_is_reported_and_both_streams_are_drained() {
        let d = temp(); let l = Launcher::new(&d).unwrap();
        let messages = Arc::new(Mutex::new(Vec::new())); let m = messages.clone();
        let log: Log = Arc::new(move |s| m.lock().unwrap().push(s));
        assert!(l.command("sh", &["-c", "echo out; echo err >&2; exit 3"], &d, &log).is_err());
        let m = messages.lock().unwrap(); assert!(m.iter().any(|s| s == "out")); assert!(m.iter().any(|s| s == "err"));
        fs::remove_dir_all(d).unwrap();
    }
}

