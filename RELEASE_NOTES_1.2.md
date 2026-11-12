# Z3-Launcher 1.2.0

This release improves portable use of Z3-Launcher and polishes its forest-themed interface on Linux and Windows.

- **Portable workspace:** an empty `portable.txt` beside the executable or original AppImage keeps game data, configuration, shaders and logs alongside the application. The redundant nested `Z3-Launcher/` directory is no longer used. For AppImage, this marker takes precedence over the matching `.home` directory.
- **Automatic ROM selection:** on startup, find and SHA-256-validate a supported headerless US `.sfc`/`.smc` ROM beside the executable/AppImage. Leave the original untouched; copy it into the installed game directory only when needed for installation.
- **Custom background:** override the embedded forest artwork with `config/background.png` (or `background.png`) beside the executable/AppImage, without recompiling or enabling portable mode. The background repeats as tiles, and invalid images fall back safely.
- **UI refinements:** a translucent compact header improves app-name/version legibility; backgrounds are slightly dimmed for contrast, form controls are better aligned, and the journal can be resized vertically. The unnecessary embedded-background startup message was removed.
- **Journal naming:** the log is now `Z3-Launcher.log`. Older `launcher.log` entries are preserved during migration, without overwriting an existing new-format log.
- **Documentation:** clarify AppImage portable-home priority, adjacent-ROM discovery and external background configuration.

**Packages:** Windows x86_64 ZIP, Linux x86_64 TAR.GZ, AppImage (+ SHA-256 checksum), and experimental Flatpak.

Z3-Launcher remains GPL-3.0-or-later with preserved upstream licensing notices. A compatible ROM supplied by the user is required; no Nintendo ROM or game assets are distributed.

legluondunet assisted by ChatGPT (OpenAI)
