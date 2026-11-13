# Z3R-only migration: validation checklist

**Status: development-only, not ready for release.** The published launcher
v1.2.0 continues to manage legluondunet/zelda3. This branch switches to
[xander-haj/Z3R](https://github.com/xander-haj/Z3R) exclusively.

- Game releases come from `xander-haj/Z3R`.
- Linux: `Z3R-linux-x64.tar.gz` (native `zelda3`), Windows:
  `Z3R-windows-x64.zip` (`zelda3.exe`).
- The GitHub release asset `sha256:` digest must be present and match.
- A standalone PyInstaller resource extractor is built from the upstream
  Z3R v0.2.4 sources, pinned to Git commit
  `dd44234a6baaaafa1bdee0a71e1006fc8877ab20`.
  No ROM or generated Nintendo data is bundled.
- Installations are isolated under `<workspace>/Z3R/`. An existing
  `<workspace>/zelda3/` directory is not modified or migrated.
- Keep existing Z3R saves, configuration and resource data on updates.
- The launcher and game journals remain `Z3-Launcher.log` and
  `zelda3.log` in the workspace root.

## Required pre-release checks

- [ ] CI builds and tests pass: Linux, Windows, AppImage, Flatpak.
- [ ] Fresh installation using a compatible US ROM generates
      `Z3R/zelda3_assets.dat` and starts Z3R.
- [ ] Verify the generated asset file with Z3R's *matching* v0.2.4 binary:
      both 4:3 and widescreen scenes must load.
- [ ] Linux native package works when SDL2 is available.
- [ ] Windows package works with bundled SDL2.dll.
- [ ] AppImage works on X11 and Wayland / XWayland fallback.
- [ ] Flatpak extractor can run inside the sandbox.
- [ ] Language import, INI editing, controllers, MSU, save files and
      shaders verified; Z3R-specific settings examined.
- [ ] Updating Z3R preserves user files, without touching legacy
      `zelda3/` installations.
- [ ] Update README and docs only after the migration is validated.

Note: the Z3R standalone releases do not include asset extraction scripts,
so the launcher builds an independent resource extractor from the pinned
Z3R source. That extracted code remains under Z3R's upstream MIT license;
the original copyright and notices are preserved with its bundle.

A successful compiler run does not confirm that ROM-derived data and the
native Z3R executable are compatible. Do not merge until a runtime test passes.
