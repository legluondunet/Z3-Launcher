// SPDX-License-Identifier: GPL-3.0-or-later
use crate::i18n::{tf, tr};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub const ROM_HASH: &str = "66871d66be19ad2c34c927d6b14cd8eb6fc3181965b6e517cb361f7316009cfb";
pub type Log = Arc<dyn Fn(String) + Send + Sync>;
#[derive(Clone)]
pub struct Launcher {
    pub root: PathBuf,
}
const LANGUAGES: &[&str] = &[
    "de", "fr", "fr-c", "en", "es", "pl", "pt", "redux", "nl", "sv",
];
impl Launcher {
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let portable = crate::platform::portable_dir();
        let root = portable.as_deref().unwrap_or_else(|| root.as_ref());
        let root = if root.is_absolute() {
            root.to_owned()
        } else {
            std::env::current_dir()?.join(root)
        };
        fs::create_dir_all(&root)?;
        Ok(Self {
            root: crate::platform::canonicalize(root)?,
        })
    }
    pub fn repo(&self) -> PathBuf {
        self.root.join("zelda3")
    }
    pub fn log(&self, callback: Log) -> Result<Log> {
        let file = Arc::new(Mutex::new(
            fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(self.root.join("launcher.log"))?,
        ));
        Ok(Arc::new(move |line| {
            if let Ok(mut f) = file.lock() {
                let _ = writeln!(f, "{line}");
            }
            callback(line);
        }))
    }
    pub fn verify_rom(path: &Path) -> Result<()> {
        if format!("{:x}", Sha256::digest(fs::read(path)?)) != ROM_HASH {
            return Err(tf("rom.incompatible", &[("hash", ROM_HASH.into())]).into());
        }
        Ok(())
    }
    pub fn assets(&self) -> Result<PathBuf> {
        Ok(self.repo().join(".resources/assets"))
    }
    pub fn has_language_resources(&self, code: &str) -> bool {
        self.assets().is_ok_and(|a| {
            a.join(format!("dialogue_{code}.txt")).is_file()
                && a.join(format!("font_{code}.png")).is_file()
        })
    }
    fn extract(&self, package: &Path, workspace: &Path, args: &[&str], log: &Log) -> Result<()> {
        let tool = package.join(crate::download::extractor_name());
        if !tool.is_file() {
            return Err(tr("download.extractor_missing").into());
        }
        let mut command = crate::platform::host_command(
            tool.to_str()
                .ok_or(tr("text.the_executable_path_is_not_utf_8"))?,
        );
        command
            .args([
                "--workspace",
                workspace
                    .to_str()
                    .ok_or(tr("text.the_rom_path_must_be_utf_8"))?,
            ])
            .args(args)
            .current_dir(package)
            .env_remove("_PYI_APPLICATION_HOME_DIR")
            .env("PYINSTALLER_RESET_ENVIRONMENT", "1")
            .env("PYTHONUTF8", "1");
        log(tr("download.extracting").into());
        run_logged(command, "zelda3-extractor", log)
    }
    fn generate(&self, package: &Path, workspace: &Path, rom: &Path, log: &Log) -> Result<()> {
        Self::verify_rom(rom)?;
        let assets = workspace.join("assets");
        fs::create_dir_all(&assets)?;
        let rom = crate::platform::canonicalize(rom)?;
        let rom = rom.to_str().ok_or(tr("text.the_rom_path_must_be_utf_8"))?;
        let languages = LANGUAGES
            .iter()
            .filter(|code| {
                assets.join(format!("dialogue_{code}.txt")).is_file()
                    && assets.join(format!("font_{code}.png")).is_file()
            })
            .copied()
            .collect::<Vec<_>>()
            .join(",");
        let flag = format!("--languages={languages}");
        let mut args = vec!["--extract-from-rom", "--rom", rom];
        if !languages.is_empty() {
            args.push(&flag);
        }
        self.extract(package, workspace, &args, log)?;
        let file = workspace.join("zelda3_assets.dat");
        if !file.is_file() || fs::metadata(&file)?.len() < 1000 {
            return Err(tr("game_language.resources_missing").into());
        }
        Ok(())
    }
    fn install_game(&self, rom: &Path, log: &Log) -> Result<()> {
        Self::verify_rom(rom)?;
        let temporary = tempfile::tempdir_in(&self.root)?;
        let staged = temporary.path().join("package");
        fs::create_dir(&staged)?;
        let version = crate::download::game(&staged, log)?;
        let resources = staged.join(".resources");
        // Preserve both current and older source-based localized resources on migration.
        let previous = self.repo().join(".resources");
        if previous.is_dir() {
            crate::download::copy_tree(&previous, &resources)?;
        } else if self.repo().join("assets").is_dir() {
            crate::download::copy_tree(&self.repo().join("assets"), &resources.join("assets"))?;
        }
        self.generate(&staged, &resources, rom, log)?;
        fs::rename(
            resources.join("zelda3_assets.dat"),
            staged.join("zelda3_assets.dat"),
        )?;
        fs::copy(rom, staged.join("zelda3.sfc"))?;
        fs::write(staged.join(".release-version"), version)?;
        crate::download::install(&staged, &self.repo(), &temporary.path().join("backup"))?;
        if let Err(error) = crate::download::shaders(&crate::platform::shaders_dir(), log) {
            log(tf(
                "download.shader_warning",
                &[("error", error.to_string())],
            ));
        }
        log(tr("download.installed").into());
        Ok(())
    }
    pub fn update(&self, log: &Log) -> Result<()> {
        self.install_game(&self.repo().join("zelda3.sfc"), log)
    }
    pub fn import_language(&self, code: &str, rom: Option<&Path>, log: &Log) -> Result<()> {
        if !LANGUAGES.contains(&code) {
            return Err(tr("game_language.unsupported").into());
        }
        let us = self.repo().join("zelda3.sfc");
        Self::verify_rom(&us)?;
        let temporary = tempfile::tempdir_in(&self.root)?;
        let staged = temporary.path().join("stage");
        fs::create_dir(&staged)?;
        let resources = staged.join(".resources");
        crate::download::copy_tree(&self.repo().join(".resources"), &resources)?;
        if let Some(rom) = rom {
            let rom = crate::platform::canonicalize(rom)?;
            self.extract(
                &self.repo(),
                &resources,
                &[
                    "--check-language",
                    code,
                    "--extract-dialogue",
                    "--rom",
                    rom.to_str().ok_or(tr("text.the_rom_path_must_be_utf_8"))?,
                ],
                log,
            )?;
        }
        if !resources
            .join("assets")
            .join(format!("dialogue_{code}.txt"))
            .is_file()
        {
            return Err(tr("game_language.resources_missing").into());
        }
        self.generate(&self.repo(), &resources, &us, log)?;
        fs::rename(
            resources.join("zelda3_assets.dat"),
            staged.join("zelda3_assets.dat"),
        )?;
        crate::download::install(&staged, &self.repo(), &temporary.path().join("backup"))?;
        log(tf("game_language.imported", &[("language", code.into())]));
        Ok(())
    }
    pub fn run(&self, log: &Log) -> Result<()> {
        let bin = self.repo().join(crate::platform::GAME_BINARY);
        if !bin.is_file() || !self.repo().join("zelda3_assets.dat").is_file() {
            return Err(tr("download.install_first").into());
        }
        let mut command = crate::platform::host_command(
            bin.to_str()
                .ok_or(tr("text.the_executable_path_is_not_utf_8"))?,
        );
        command.current_dir(self.repo());
        #[cfg(target_os = "linux")]
        command.env("APPIMAGE_EXTRACT_AND_RUN", "1");
        run_logged(command, crate::platform::GAME_BINARY, log)
    }
    pub fn action(&self, action: &str, rom: Option<&Path>, log: &Log) -> Result<()> {
        match action {
            "setup" => self.install_game(rom.ok_or(tr("text.select_a_us_rom"))?, log),
            "update" => self.update(log),
            "run" => self.run(log),
            "status" => {
                log(tf(
                    "launcher.status",
                    &[
                        ("path", self.root.display().to_string()),
                        (
                            "repo",
                            self.repo().join(".release-version").exists().to_string(),
                        ),
                        ("rom", self.repo().join("zelda3.sfc").is_file().to_string()),
                        (
                            "binary",
                            self.repo()
                                .join(crate::platform::GAME_BINARY)
                                .is_file()
                                .to_string(),
                        ),
                    ],
                ));
                Ok(())
            }
            _ => Err(tf("cli.unknown_command", &[("action", action.to_owned())]).into()),
        }
    }
}
fn run_logged(mut command: Command, program: &str, log: &Log) -> Result<()> {
    let mut child = command
        .env("PYTHONUNBUFFERED", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let out = child
        .stdout
        .take()
        .ok_or(tr("text.stdout_is_unavailable"))?;
    let err = child
        .stderr
        .take()
        .ok_or(tr("text.stderr_is_unavailable"))?;
    let l = log.clone();
    let a = thread::spawn(move || {
        for line in BufReader::new(out)
            .lines()
            .map_while(std::result::Result::ok)
        {
            l(line);
        }
    });
    let l = log.clone();
    let b = thread::spawn(move || {
        for line in BufReader::new(err)
            .lines()
            .map_while(std::result::Result::ok)
        {
            l(line);
        }
    });
    let status = child.wait()?;
    let _ = a.join();
    let _ = b.join();
    if !status.success() {
        return Err(tf(
            "process.failed",
            &[("program", program.into()), ("status", status.to_string())],
        )
        .into());
    }
    Ok(())
}
pub fn save_ini(path: &Path, text: &str) -> Result<()> {
    if !path.is_file() {
        return Err(tr("text.load_an_existing_ini_file_first").into());
    }
    fs::copy(path, path.with_extension("ini.bak"))?;
    let temp = path.with_extension("ini.tmp");
    fs::write(&temp, text)?;
    fs::rename(temp, path)?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_rom_before_network_access() {
        let t = tempfile::tempdir().unwrap();
        let rom = t.path().join("bad.sfc");
        fs::write(&rom, b"bad").unwrap();
        let l = Launcher::new(t.path()).unwrap();
        let log: Log = Arc::new(|_| {});
        assert!(l.action("setup", Some(&rom), &log).is_err());
        assert!(!l.repo().exists());
        assert!(l.action("build", None, &log).is_err());
        assert!(l.action("check", None, &log).is_err());
    }
}
