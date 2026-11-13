# Z3-Launcher
**A graphical installer, updater and configuration manager for Zelda 3 Reborn (Z3R) on Linux and Windows.**

[![Native builds](https://github.com/legluondunet/Z3-Launcher/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/legluondunet/Z3-Launcher/actions/workflows/build.yml)
[![License: GPL-3.0-or-later](https://img.shields.io/badge/license-GPL--3.0--or--later-blue.svg)](LICENSE)

Z3-Launcher is being adapted exclusively for **[Z3R — Zelda 3 Reborn](https://github.com/xander-haj/Z3R)**, a fork of [snesrev/zelda3](https://github.com/snesrev/zelda3). Based on the original [Zelda 3 Launcher](https://github.com/RadzPrower/Zelda-3-Launcher) by RadzPrower, it downloads verified Z3R game releases and generates the required assets from **your own compatible ROM**.

> **Development branch:** Z3R installation and resource compatibility require end-to-end validation. Z3-Launcher 1.2.0 (the published release) continues to use the previous fork. Existing installations are not migrated automatically.

<img width="1251" height="863" alt="Z3-Launcher main window with game installation controls, configuration tabs, and journal panel" src="https://github.com/user-attachments/assets/0d97a976-0465-49c0-b5a8-6f93f79b094b" />


## Features

- **Install and play:** select a compatible US ROM, download the latest Z3R release, extract assets locally, and launch Zelda 3.
- **Safe game updates:** verify downloads with SHA-256 checksums, regenerate game resources, and preserve existing configuration, save files, MSU music packs and imported languages. Failed file replacements are rolled back.
- **Game settings:** configure gameplay, graphics, audio, keyboard/gamepad bindings, and shortcuts through the interface. Valid changes are saved automatically for the next launch; gamepad input capture and default binding restoration are supported.
- **Built-in INI editor:** edit `zelda3.ini` with syntax highlighting, line numbers, and save/discard prompts.
- **Advanced graphics:** select window and fullscreen modes, rendering options, Link sprites, and compatible OpenGL `.glsl` / `.glslp` shaders.
- **Multilingual interface:** English, French, German, Spanish and Italian, with automatic system-language detection on first launch and English fallback. Importing a supported game-language ROM is a separate feature and does not change the interface language.
- **Portable installations:** keep game files, preferences, shaders and logs in the executable/AppImage directory when `portable.txt` is present, without an extra nested workspace. Both `Z3-Launcher.log` and `zelda3.log` are stored at the workspace root.
- **Forest-inspired interface:** pixel-art background, translucent panels, gold-accented controls and scrollbars.

## Supported platforms

| Platform | Distribution |
| --- | --- |
| Windows x86_64 | Standalone `z3-launcher.exe` |
| Linux x86_64 | Native binary |
| Linux x86_64 | AppImage |
| Linux x86_64 | Flatpak (experimental) |


Download the published **[latest Z3-Launcher release](https://github.com/legluondunet/Z3-Launcher/releases/latest)** (Windows ZIP, Linux native TAR.GZ, AppImage or experimental Flatpak). Release archives include the launcher; a compatible user-provided ROM is still required to install the game.

## Quick start

1. Download Z3-Launcher for your operating system.
2. Start the launcher. A matching ROM beside the executable/AppImage is selected automatically; otherwise select your **headerless US Zelda: A Link to the Past ROM** (`.sfc` or `.smc`).
3. Select **Download and install**. The launcher retrieves the game package and generates `zelda3_assets.dat` locally.
4. Select **Play Zelda 3**.
5. Customize the game in the **Game**, **Graphics**, **Audio**, **Controls**, **Shortcuts**, or **zelda3.ini** tabs.

### ROM requirements

The supported US ROM has SHA-256:

```text
66871d66be19ad2c34c927d6b14cd8eb6fc3181965b6e517cb361f7316009cfb
```

The ROM is processed **on your computer** and is never uploaded. This repository and its releases do not contain Nintendo ROMs or copyrighted game assets.

Z3R releases do not include an asset extractor. During the transition, Z3-Launcher downloads a pinned, SHA-256-verified standalone extractor from our previous fork; no system Python is needed. Compatibility with all Z3R-specific resource changes remains to be tested. The launcher generates `zelda3_assets.dat` in the installed Z3R directory and retains the ROM there for subsequent resource regeneration.

## Installation and portable mode

Download the package for your platform from the [latest release](https://github.com/legluondunet/Z3-Launcher/releases/latest). On Windows, extract the ZIP and run `z3-launcher.exe`. On Linux, run the AppImage or extract the native `.tar.gz` and run `z3-launcher`. The distributed launcher does not require Rust, Python, or other build tools.

### Flatpak (Linux, experimental)

Install Flatpak with your distribution's package manager, then install and launch the downloaded bundle:

```bash
flatpak install --user ./Z3-Launcher-x86_64.flatpak
flatpak run io.github.legluondunet.Z3Launcher
```

This distribution format is experimental; launching the separately downloaded game may require additional sandbox permissions.

### Portable workspace

Create an empty `portable.txt` beside `z3-launcher.exe`, the native `z3-launcher` binary, or the **original AppImage**, then restart the launcher. Game files, settings, shaders, and both log files will be kept in that directory (or its subdirectories), without an extra nested workspace:

```text
Z3-Launcher/
├── z3-launcher.exe (or z3-launcher / Z3-Launcher-*.AppImage)
├── portable.txt
├── Z3-Launcher.log
├── zelda3.log
├── config/
└── zelda3/
```

On AppImage, `portable.txt` takes priority over a matching `<full AppImage filename>.home` directory. Without portable mode, user data and configuration follow the normal platform directories. Switching modes does not migrate existing data. See [WINDOWS.md](WINDOWS.md) and [APPIMAGE.md](APPIMAGE.md) for platform-specific instructions.

### Automatic ROM discovery

The launcher checks for a supported headerless US `.sfc` or `.smc` ROM next to the executable or original AppImage, verifies its SHA-256 hash, and preselects it. The original is not modified; a copy is stored inside the installed `Z3R/` directory when installing. ROM discovery works without portable mode.

### Separate launcher and game journals

Installation, update and launcher messages are written to `Z3-Launcher.log`. When the game runs, its standard output and error output are captured separately in `zelda3.log`, **right beside the launcher log in the workspace root**, rather than inside the `zelda3/` folder. The journal panel offers two views: **Z3-Launcher** and **Zelda 3**. Both files accumulate output across sessions.

With `portable.txt`, both logs are directly next to the launcher executable or original AppImage. Without portable mode, both logs are kept together in the selected user-writable workspace rather than relying on permission to write beside a system-installed executable.

## Updating the game

Use **Update game** to download the latest [Z3R release](https://github.com/xander-haj/Z3R/releases/latest) and regenerate its assets. The updater verifies GitHub's published SHA-256 asset digest, preserves user data and rolls back failed file replacements. Z3R is installed in its own `Z3R/` directory: older installations in `zelda3/` are left untouched.

Public downloads do not require a GitHub account or API token. A game release must be available before installation can proceed.

## Command-line interface

The graphical launcher also supports:

```bash
z3-launcher setup --rom /path/to/zelda3.sfc
z3-launcher update
z3-launcher run
z3-launcher status
z3-launcher --version
z3-launcher status --verbose
z3-launcher run --verbose
```

Use `--dir DIRECTORY` to choose a different workspace and `--rom US_ROM` with `setup`. The `--version` flag prints the installed launcher version and short build commit without launching the GUI or changing any files.

The optional `--verbose` flag works with `setup`, `update`, `run` and `status`, before or after the command. It adds version, platform, workspace and path diagnostics to the terminal and `Z3-Launcher.log`. For `run --verbose`, Zelda 3 stdout/stderr are also shown in the terminal with a `[zelda3]` prefix, while the actual game output remains **only** in `zelda3.log` on disk. Without `--verbose`, the terminal remains concise and game output is still captured in its own log.

The existing `status` command reports installation status; there is no separate `--verify` flag. See [WINDOWS.md](WINDOWS.md) and [APPIMAGE.md](APPIMAGE.md) for platform-specific information.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for the development history and release notes. The experimental Flatpak includes AppStream release notes, with its version checked against `Cargo.toml` by `tools/sync-flatpak-version.py`.

## Translations

The interface includes English, French, German, Spanish and Italian. External translation catalogs can extend or override the built-in messages without recompiling the launcher. See [TRANSLATING.md](TRANSLATING.md) to contribute translations.

## Building from source (developers)

Z3-Launcher itself is written in Rust. To build it, install Rust and the relevant platform development libraries (SDL2, X11/Wayland and OpenGL on Linux).

```bash
cargo test
cargo build --release
```

For non-GUI tests, run `cargo test --no-default-features`. To build the Linux AppImage, run `bash ./build-appimage.sh`.

**These dependencies are only required to develop or build Z3-Launcher. They are not required to install or play Zelda 3 through a distributed launcher.**

## License and credits

Z3-Launcher is distributed under **GPL-3.0-or-later**. See [LICENSE](LICENSE) and [LICENSING.md](LICENSING.md) for license details and preserved upstream notices.

Game project: [legluondunet/zelda3](https://github.com/legluondunet/zelda3) · Original Zelda 3 reverse-engineering project: [snesrev/zelda3](https://github.com/snesrev/zelda3).
