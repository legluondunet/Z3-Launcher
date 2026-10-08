// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 legluondunet — https://github.com/legluondunet
//! Detect the current Linux distribution without evaluating os-release as shell code.
use crate::i18n::{tr, tf};
use std::{collections::BTreeSet, fs};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family { Debian, Arch, Fedora, Suse, Unknown }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dependency { Git, Python, Pillow, Yaml, Make, Compiler, Sdl, Sha }
impl Dependency {
    fn label(self) -> &'static str { match self {Self::Git=>"Git",Self::Python=>"Python 3",Self::Pillow=>tr("text.python_pillow"),Self::Yaml=>tr("text.python_pyyaml"),Self::Make=>"make",Self::Compiler=>tr("text.c_compiler_cc"),Self::Sdl=>tr("text.sdl2_sdl2_config"),Self::Sha=>"sha256sum"} }
    fn probe(self) -> (&'static str, Vec<&'static str>) {
        match self {
            Self::Git=>("git",vec!["--version"]),Self::Python=>("python3",vec!["--version"]),
            Self::Pillow=>("python3",vec!["-c","import PIL"]),Self::Yaml=>("python3",vec!["-c","import yaml"]),
            Self::Make=>("make",vec!["--version"]),Self::Compiler=>("cc",vec!["--version"]),
            Self::Sdl=>("sdl2-config",vec!["--version"]),Self::Sha=>("sha256sum",vec!["--version"]),
        }
    }
}
#[derive(Debug)]
pub struct Distribution { pub name: String, pub family: Family }
impl Distribution {
    pub fn parse(text: &str) -> Self {
        let value = |name:&str| -> String {
            text.lines().filter_map(|line|line.split_once('=')).find(|(k,_)|k.trim()==name)
                .map(|(_,v)|v.trim().trim_matches('"').trim_matches('\'').to_owned()).unwrap_or_default()
        };
        let id=value("ID");let like=value("ID_LIKE");let pretty=value("PRETTY_NAME");let variant=value("VARIANT_ID");
        let classify=|id:&str| match id {
            "debian"|"ubuntu"|"linuxmint"|"pop"|"elementary"|"zorin"|"kali"=>Family::Debian,
            "arch"|"manjaro"|"endeavouros"|"garuda"=>Family::Arch,
            "fedora"|"nobara"=>Family::Fedora,
            "opensuse"|"opensuse-leap"|"opensuse-tumbleweed"|"opensuse-slowroll"=>Family::Suse,
            _=>Family::Unknown,
        };
        let immutable = ["silverblue", "kinoite", "coreos", "sericea", "onyx", "sway-atomic", "budgie-atomic"].contains(&variant.as_str());
        let mut family=if immutable { Family::Unknown } else { classify(&id) };
        // Immutable distributions and enterprise variants require their own instructions.
        if !immutable && !["rhel","centos","rocky","almalinux","silverblue","kinoite","bazzite","steamos"].contains(&id.as_str()) && family==Family::Unknown {
            family=like.split_whitespace().map(classify).find(|f|*f!=Family::Unknown).unwrap_or(Family::Unknown);
        }
        Self {name:if pretty.is_empty(){if id.is_empty(){tr("text.distribution_not_detected").into()}else{id}}else{pretty},family}
    }
    fn detect() -> Self { Self::parse(&fs::read_to_string("/etc/os-release").or_else(|_|fs::read_to_string("/usr/lib/os-release")).unwrap_or_default()) }
}
pub struct Report { pub distribution: Distribution, pub missing: Vec<Dependency>, rows: Vec<String> }
impl Report {
    pub fn lines(&self) -> std::vec::IntoIter<String> {
        let mut lines=vec![tf("dependencies.distribution", &[("name", self.distribution.name.clone())])];lines.extend(self.rows.clone());
        if self.missing.is_empty() {lines.push(tr("text.all_required_dependencies_are_available").into());}
        else {
            lines.push(tf("dependencies.missing_list", &[("list", self.missing.iter().map(|d|d.label()).collect::<Vec<_>>().join(", "))]));
            if let Some(command)=install_command(self.distribution.family,&self.missing) {
                lines.push(tr("text.example_command_to_enter_in_a_terminal").into());lines.push(command);
                lines.push(tr("text.nothing_is_installed_automatically_run_check_dependencies_again_after").into());
                lines.push(tr("text.if_python_runs_in_a_virtual_environment_system_packages").into());
            } else {lines.push(tr("text.unsupported_distribution_install_these_dependencies_using_its_package_manager").into());}
        }
        lines.into_iter()
    }
}
pub fn inspect() -> Report {
    let distribution=Distribution::detect();let mut missing=Vec::new();let mut rows=Vec::new();
    for dependency in [Dependency::Git,Dependency::Python,Dependency::Pillow,Dependency::Yaml,Dependency::Make,Dependency::Compiler,Dependency::Sdl,Dependency::Sha] {
        let (program,args)=dependency.probe();
        match crate::platform::host_command(program).args(args).output() {
            Ok(output) if output.status.success()=>rows.push(tf("dependencies.ok", &[("name", dependency.label().to_owned())])),
            Ok(output)=>{
                missing.push(dependency);rows.push(tf("dependencies.missing", &[("name", dependency.label().to_owned())]));
                let detail=String::from_utf8_lossy(&output.stderr).trim().to_owned();if !detail.is_empty(){rows.push(detail);}
            },
            Err(e)=>{missing.push(dependency);rows.push(tf("dependencies.missing_detail", &[("name", dependency.label().to_owned()), ("error", e.to_string())]));},
        }
    }
    Report {distribution,missing,rows}
}
pub fn package_names(family: Family, missing: &[Dependency]) -> Vec<String> {
    if family == Family::Unknown { return Vec::new(); }
    let packages:BTreeSet<&str>=missing.iter().map(|d|match family {
        Family::Debian=>match d {Dependency::Git=>"git",Dependency::Python=>"python3",Dependency::Pillow=>"python3-pil",Dependency::Yaml=>"python3-yaml",Dependency::Make|Dependency::Compiler=>"build-essential",Dependency::Sdl=>"libsdl2-dev",Dependency::Sha=>"coreutils"},
        Family::Arch=>match d {Dependency::Git=>"git",Dependency::Python=>"python",Dependency::Pillow=>"python-pillow",Dependency::Yaml=>"python-yaml",Dependency::Make|Dependency::Compiler=>"base-devel",Dependency::Sdl=>"sdl2-compat",Dependency::Sha=>"coreutils"},
        Family::Fedora=>match d {Dependency::Git=>"git",Dependency::Python=>"python3",Dependency::Pillow=>"python3-pillow",Dependency::Yaml=>"python3-pyyaml",Dependency::Make=>"make",Dependency::Compiler=>"gcc",Dependency::Sdl=>"sdl2-compat-devel",Dependency::Sha=>"coreutils"},
        Family::Suse=>match d {Dependency::Git=>"git",Dependency::Python=>"python3",Dependency::Pillow=>"python3-Pillow",Dependency::Yaml=>"python3-PyYAML",Dependency::Make=>"make",Dependency::Compiler=>"gcc",Dependency::Sdl=>"libSDL2-devel",Dependency::Sha=>"coreutils"},
        Family::Unknown=>unreachable!(),
    }).collect();
    packages.into_iter().map(str::to_owned).collect()
}
pub fn install_command(family:Family,missing:&[Dependency]) -> Option<String> {
    if missing.is_empty() || family==Family::Unknown {return None;}
    let packages=package_names(family, missing).join(" ");
    Some(match family {
        Family::Debian=>format!("sudo apt update && sudo apt install {packages}"),
        Family::Arch=>format!("sudo pacman -S --needed {packages}"),
        Family::Fedora=>format!("sudo dnf install {packages}"),
        Family::Suse=>format!("sudo zypper install {packages}"),Family::Unknown=>unreachable!()
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn detects_known_distributions_and_derivatives() {
        for (id,family) in [("ubuntu",Family::Debian),("linuxmint",Family::Debian),("manjaro",Family::Arch),("fedora",Family::Fedora),("opensuse-tumbleweed",Family::Suse)] {assert_eq!(Distribution::parse(&format!("ID={id}\n")).family,family);}
        assert_eq!(Distribution::parse("ID=custom\nID_LIKE=\"ubuntu debian\"\n").family,Family::Debian);
        assert_eq!(Distribution::parse("ID=fedora\nVARIANT_ID=silverblue\n").family,Family::Unknown);
        assert_eq!(Distribution::parse("ID=bazzite\nID_LIKE=fedora\n").family,Family::Unknown);
    }
    #[test] fn commands_only_contain_needed_packages_and_deduplicate() {
        let command=install_command(Family::Debian,&[Dependency::Make,Dependency::Compiler,Dependency::Yaml]).unwrap();
        assert_eq!(command,"sudo apt update && sudo apt install build-essential python3-yaml");
        assert!(!command.contains("libsdl2"));assert!(install_command(Family::Unknown,&[Dependency::Git]).is_none());
    }
}

