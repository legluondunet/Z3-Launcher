# Z3-Launcher
**A graphical Zelda 3 installer, updater and configuration manager for Linux and Windows.**

[![Native builds](https://github.com/legluondunet/Z3-Launcher/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/legluondunet/Z3-Launcher/actions/workflows/build.yml)
[![License: GPL-3.0-or-later](https://img.shields.io/badge/license-GPL--3.0--or--later-blue.svg)](LICENSE)

Z3-Launcher is a desktop application for [legluondunet/zelda3](https://github.com/legluondunet/zelda3), based on the [Zelda 3 Launcher](https://github.com/RadzPrower/Zelda-3-Launcher) project code to manage [snesrev/zelda3](https://github.com/snesrev/zelda3) game. It downloads prebuilt Zelda 3 releases, extracts the required assets from **your own ROM**, and provides a graphical interface for configuring and launching the game.

<img width="1251" height="863" alt="image" src="https://github.com/user-attachments/assets/a6901e13-d0d5-4623-80f5-80348b4523a0" />


### Custom launcher background

To replace the built-in forest background **without recompiling and without enabling portable mode**, place a PNG called `background.png` in a `config/` directory next to the actual executable or AppImage. Restart the launcher after replacing it.

For example:

```text
Z3-Launcher-1.2.0+abcdef0-x86_64.AppImage
config/
  background.png
```

The launcher searches for an override in this order:

1. `config/background.png` beside the executable or original AppImage.
2. `background.png` directly beside the executable or original AppImage.
3. `background.png` in the platform's normal Z3-Launcher configuration directory (including the AppImage `.home/.config/Z3-Launcher` location when applicable).

The first valid image takes priority. Invalid images are skipped, and the built-in background is used if none can be loaded. Startup diagnostics in `Z3-Launcher.log` and the interface journal report the selected source or an image-loading error. The override is drawn as repeating tiles, not stretched; for invisible joins, the image itself must be seamlessly tileable on all four edges.

## Features

- **Install and play:** select a compatible US ROM, download the latest game release, extract assets locally, and launch Zelda 3.
- **Safe game updates:** verify downloads with SHA-256 checksums, regenerate game resources, and preserve existing configuration, save files, MSU music packs and imported languages. Failed file replacements are rolled back.
- **Graphical settings:** adjust gameplay, graphics, audio, keyboard bindings, gamepad controls and shortcuts without manually editing the INI file.
- **Built-in INI editor:** edit `zelda3.ini` directly. When leaving the tab with unsaved changes, choose whether to save, discard or continue editing. The editor provides syntax highlighting, line numbers and status feedback.
- **Automatic settings saves:** valid changes made through the graphical settings controls are saved automatically and applied on the next game launch.
- **Input configuration:** assign keyboard shortcuts and SDL gamepad bindings, capture inputs, and restore default controller bindings.
- **Graphics customization:** choose display and fullscreen modes, rendering options, Link sprites and compatible OpenGL `.glsl` / `.glslp` shaders.
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

Extract the ZIP into a writable folder. Create an **empty file** named `portable.txt` next to `z3-launcher.exe` (make sure Windows does not add another `.txt`). Restart the launcher. It stores the game, configuration, shaders and `Z3-Launcher.log` alongside the executable, in that directory or its subdirectories; there is no extra nested `Z3-Launcher/` workspace:

```text
Z3-Launcher/
├── z3-launcher.exe
├── portable.txt
├── Z3-Launcher.log
├── zelda3.log
├── config/
└── zelda3/
```

Portable mode does not automatically migrate existing non-portable data. See [WINDOWS.md](WINDOWS.md).

### Portable mode (native Linux binary)

Extract the Linux `.tar.gz` archive into a writable directory and create an empty `portable.txt` alongside the `z3-launcher` binary:

```bash
touch portable.txt
./z3-launcher
```

The marker must be next to the executable, not just in the terminal's working directory. The launcher stores game files, `config/`, shaders and its log in the executable directory or its subdirectories. Without the marker, normal XDG user data and configuration locations apply; existing data is not moved automatically.

### Portable mode (Linux AppImage)

Create an empty `portable.txt` next to the **original AppImage file**, not in its temporary mount. This gives the launcher a workspace alongside the AppImage and **takes priority** over an AppImage portable-home `.home` directory, when both are present.

Alternatively, without `portable.txt`, create a directory matching the **entire AppImage filename followed by `.home`** (for example `Z3-Launcher-1.2.0+abcdef0-x86_64.AppImage.home`); that uses the AppImage portable-home environment. Keep the AppImage and matching directory together if you move or rename them. Restart after changing modes; data is never migrated automatically. See [APPIMAGE.md](APPIMAGE.md).

### Adjacent ROM and portable mode

At startup, the launcher checks for a compatible headerless US `.sfc` or `.smc` ROM next to its executable or the original AppImage. It verifies the ROM's SHA-256 hash and preselects the matching file. Your original ROM remains untouched: on installation, a copy named `zelda3.sfc` is written inside the installed `zelda3/` directory only after resource generation succeeds.

The adjacent-ROM detection and external `background.png` override work **without** portable mode. Use `portable.txt` only when you also want launcher-generated files to live beside the executable.

### Separate launcher and game journals

Installation, update and launcher messages are written to `Z3-Launcher.log`. When the game runs, its standard output and error output are captured separately in `zelda3.log`, **right beside the launcher log in the workspace root**, rather than inside the `zelda3/` folder. The journal panel offers two views: **Z3-Launcher** and **Zelda 3**. Both files accumulate output across sessions.

With `portable.txt`, both logs are directly next to the launcher executable or original AppImage. Without portable mode, both logs are kept together in the selected user-writable workspace rather than relying on permission to write beside a system-installed executable.

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
