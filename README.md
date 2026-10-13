# Z3-Launcher
**A graphical Zelda 3 installer, updater and configuration manager for Linux and Windows.**

[![Native builds](https://github.com/legluondunet/Z3-Launcher/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/legluondunet/Z3-Launcher/actions/workflows/build.yml)
[![License: GPL-3.0-or-later](https://img.shields.io/badge/license-GPL--3.0--or--later-blue.svg)](LICENSE)

Z3-Launcher is a desktop application for [legluondunet/zelda3](https://github.com/legluondunet/zelda3), based on the [Zelda 3 Launcher](https://github.com/RadzPrower/Zelda-3-Launcher) project code to manage [snesrev/zelda3](https://github.com/snesrev/zelda3) game. It downloads prebuilt Zelda 3 releases, extracts the required assets from **your own ROM**, and provides a graphical interface for configuring and launching the game.

<img width="1251" height="863" alt="image" src="https://github.com/user-attachments/assets/a6901e13-d0d5-4623-80f5-80348b4523a0" />

## Features

- **Install and play:** select a compatible US ROM, download the latest game release, extract assets locally, and launch Zelda 3.
- **Safe game updates:** verify downloads with SHA-256 checksums, regenerate game resources, and preserve existing configuration, save files, MSU music packs and imported languages. Failed file replacements are rolled back.
- **Graphical settings:** adjust gameplay, graphics, audio, keyboard bindings, gamepad controls and shortcuts without manually editing the INI file.
- **Built-in INI editor:** edit `zelda3.ini` directly. When leaving the tab with unsaved changes, choose whether to save, discard or continue editing. The editor provides syntax highlighting, line numbers and status feedback.
- **Automatic settings saves:** valid changes made through the graphical settings controls are saved automatically and applied on the next game launch.
- **Input configuration:** assign keyboard shortcuts and SDL gamepad bindings, capture inputs, and restore default controller bindings.
- **Graphics customization:** choose display and fullscreen modes, rendering options, Link sprites and compatible OpenGL `.glsl` / `.glslp` shaders.
- **Multilingual interface:** English, French, German, Spanish and Italian, with automatic system-language detection on first launch and English fallback. Importing a supported game-language ROM is a separate feature and does not change the interface language.
- **Portable installations:** keep game files and preferences alongside the launcher when portable mode is enabled.
- **Forest-inspired interface:** pixel-art background, translucent panels, gold-accented controls and scrollbars.

## Supported platforms

| Platform | Distribution |
| --- | --- |
| Windows x86_64 | Standalone `z3-launcher.exe` |
| Linux x86_64 | Native binary |
| Linux x86_64 | AppImage |
| Linux x86_64 | Flatpak (experimental) |


Download the published **[latest Z3-Launcher release](https://github.com/legluondunet/Z3-Launcher/releases/latest)** (Windows ZIP, Linux native TAR.GZ, AppImage or experimental Flatpak). Release archives include the launcher; a compatible user-provided ROM is still required to install the game.

## Installation and portable mode

Download the package for your system from the [latest release](https://github.com/legluondunet/Z3-Launcher/releases/latest).

### Flatpak (Linux, experimental)

Install Flatpak first using your distribution's package manager, then install the downloaded bundle (replace the filename with the one you downloaded):

```bash
flatpak install --user ./Z3-Launcher-x86_64.flatpak
flatpak run io.github.legluondunet.Z3Launcher
```

The bundle is an experimental distribution format; the separately downloaded Zelda 3 game may require additional Flatpak sandbox permissions depending on your system.

### Portable mode (Windows)

Extract the Windows ZIP into a writable folder. Create an **empty file** named `portable.txt` next to `z3-launcher.exe` (make sure Windows does not name it `portable.txt.txt`). Restart the launcher. It will use the adjacent `Z3-Launcher` directory for game data and configuration:

```text
Z3-Launcher/
├── z3-launcher.exe
├── portable.txt
└── Z3-Launcher/
    └── config/
```

Portable mode does not automatically move data from an existing non-portable installation. See [WINDOWS.md](WINDOWS.md).

### Portable mode (Linux AppImage)

Place an empty `portable.txt` next to the `.AppImage` file to use an adjacent `Z3-Launcher` data directory. Alternatively, create a directory whose name is the **full AppImage filename followed by `.home`** (for example `Z3-Launcher-1.1.0-x86_64.AppImage.home`): this uses AppImage portable-home mode, which takes precedence over `portable.txt`. Keep the AppImage and its matching `.home` directory together when moving them. Restart the launcher after changing modes. Existing data is not migrated automatically. See [APPIMAGE.md](APPIMAGE.md).

## Quick start

1. Download Z3-Launcher for your operating system.
2. Start the launcher and select your **headerless US Zelda: A Link to the Past ROM** (`.sfc` or `.smc`).
3. Select **Download and install**. The launcher retrieves the game package and generates `zelda3_assets.dat` locally.
4. Select **Play Zelda 3**.
5. Customize the game in the **Game**, **Graphics**, **Audio**, **Controls**, **Shortcuts**, or **zelda3.ini** tabs.

### ROM requirements

The supported US ROM has SHA-256:

```text
66871d66be19ad2c34c927d6b14cd8eb6fc3181965b6e517cb361f7316009cfb
```

The ROM is processed **on your computer** and is never uploaded. This repository and its releases do not contain Nintendo ROMs or copyrighted game assets.

The game release includes a standalone resource extractor with its own Python runtime and dependencies. The launcher generates `zelda3_assets.dat` in the installed game's directory and retains the ROM there for subsequent resource regeneration.

## Updating the game

Use **Update game** to download the latest compatible [zelda3 release](https://github.com/legluondunet/zelda3/releases/latest) and regenerate its assets. The updater checks the published `SHA256SUMS` file, preserves user data and protects the existing installation when an update fails.

Public downloads do not require a GitHub account or API token. A game release must be available before installation can proceed.

## Command-line interface

The graphical launcher also supports:

```bash
z3-launcher setup --rom /path/to/zelda3.sfc
z3-launcher update
z3-launcher run
z3-launcher status
```

Use `--dir DIRECTORY` to choose a different workspace. See [WINDOWS.md](WINDOWS.md) and [APPIMAGE.md](APPIMAGE.md) for platform-specific information.

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
