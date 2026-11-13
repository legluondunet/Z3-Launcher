# Z3R migration — Z3-Launcher 2.0.0

Z3-Launcher 2.0.0 **exclusively supports [Z3R](https://github.com/xander-haj/Z3R)**. It replaces the earlier launcher integration with `legluondunet/zelda3` and keeps older installations separate. This document records the migration's implementation and validation boundaries; it is not an alternative installation guide.

## Implementation

- **Supported upstream release:** Z3R `v0.2.4`, at commit `dd44234a6baaaafa1bdee0a71e1006fc8877ab20`. The exact commit supplies the bundled resource extractor. Later upstream releases remain blocked until tested.
- **Downloads:** official GitHub release artifacts with SHA-256 digests from the Releases API; `Z3R-linux-x64.tar.gz` (native Linux binary) and `Z3R-windows-x64.zip`.
- **Resources:** a standalone PyInstaller extractor containing the upstream Z3R asset scripts, Pillow and PyYAML. ROM extraction and generation of `zelda3_assets.dat` take place locally and require a compatible user-supplied US ROM. No game assets or ROM are distributed.
- **Migration safety:** the new installation resides in `<workspace>/Z3R/`; old `<workspace>/zelda3/` installations are never overwritten or silently migrated.
- **Configuration:** Z3R's bundled `zelda3.ini` supplies defaults; `zelda3.user.ini` holds editable user overrides. Saves, language resources and personal settings are preserved on game updates.
- **Logs:** `Z3-Launcher.log` and `zelda3.log` remain together at the workspace root, including in portable mode.

## Validation at release preparation

- [x] Z3R migration build/test CI passed on Linux, Windows, AppImage and experimental Flatpak.
- [x] Bundled extractor self-tests verify required Python imports, script syntax, palette lookup data and font assets on supported build platforms.
- [x] The user confirmed that the migrated launcher and Z3R work in their own test environment.
- [ ] Comprehensive runtime tests are still recommended for all package formats, distributions, controllers, MSU audio, language imports and cross-platform save-file handling.
- [ ] Native Wayland startup behavior on the affected system in [issue #34](https://github.com/legluondunet/Z3-Launcher/issues/34) still requires separate confirmation.

Passing CI and the no-ROM extractor self-test does not guarantee runtime compatibility on every system. The user must provide their own compatible ROM and extracted assets; report problems using `Z3-Launcher.log` and `zelda3.log`.

## Licensing

The launcher is GPL-3.0-or-later, with preserved MIT attribution for the original launcher. The bundled Z3R resource scripts retain their upstream MIT license and notices. See [LICENSING.md](LICENSING.md).
