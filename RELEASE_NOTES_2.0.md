# Z3-Launcher 2.0.0 — Z3R edition

**Major release:** Z3-Launcher now exclusively installs and manages [Z3R (Zelda 3 Reborn)](https://github.com/xander-haj/Z3R), replacing support for the former `legluondunet/zelda3` game fork.

## What's changed

- **Official Z3R binaries:** download the tested Z3R **v0.2.4** release from `xander-haj/Z3R`. Use the native Linux `Z3R-linux-x64.tar.gz` or Windows `Z3R-windows-x64.zip` package.
- **Verified downloads:** compare downloaded game archives with their SHA-256 digests supplied by the GitHub Releases API. The launcher intentionally restricts Z3R to the tested version until a newer game's asset pipeline has been validated.
- **Self-contained asset creation:** generate `zelda3_assets.dat` on the user's computer with a bundled resource extractor built from the exact upstream Z3R v0.2.4 sources. End users do not need Git, Python, PyInstaller or a C compiler.
- **Non-destructive migration:** Z3R is installed in a separate `Z3R/` directory. Existing `zelda3/` installations from Z3-Launcher 1.x are neither overwritten nor silently imported.
- **User configuration:** preserve `zelda3.user.ini` overrides while keeping Z3R's distributed `zelda3.ini` defaults. The configuration editor and game setting controls target the user override.
- **Preserved user data:** updates retain user configuration, saves, MSU files and imported language resources, while continuing to use staged replacement and rollback.
- **Unchanged launcher conveniences:** portable mode (`portable.txt`), automatic selection of a compatible US ROM beside the launcher, separate `Z3-Launcher.log` and `zelda3.log` journals, and `--version` / `--verbose` remain available.
- **Platform packages:** Windows x86_64 ZIP, native Linux x86_64 TAR.GZ, Linux AppImage (plus SHA-256 checksum), and experimental Flatpak.

## Important upgrade information

- **A user-supplied compatible US Zelda: A Link to the Past ROM is required.** No Nintendo ROM or generated copyrighted game assets are distributed.
- **No automatic migration from 1.x:** the new `Z3R/` directory is separate from the old `zelda3/` directory. Keep the old installation if you need its data; manually review any save/config transfer.
- This release supports **Z3R v0.2.4 only**. If a newer Z3R version is published, it must be validated with our extractor before it can be installed.
- On Linux, the downloaded native Z3R game requires a working SDL2 runtime. The AppImage bundles the launcher and asset extractor, not the game executable.
- A native Wayland/OpenGL startup issue affecting some launcher AppImage environments remains tracked in [issue #34](https://github.com/legluondunet/Z3-Launcher/issues/34); the previously confirmed manual XWayland workaround is `env -u WAYLAND_DISPLAY ./Z3-Launcher-*.AppImage` where XWayland is available.

## Licenses

Z3-Launcher is GPL-3.0-or-later. The original RadzPrower launcher retains its MIT attribution and the packaged Z3R extractor retains its upstream MIT license. See [LICENSING.md](https://github.com/legluondunet/Z3-Launcher/blob/main/LICENSING.md). This launcher is an independent project, not the official Z3R launcher.
