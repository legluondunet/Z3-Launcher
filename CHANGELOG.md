# Changelog

All notable Z3-Launcher changes, from the original source import to the latest documented change, are recorded here.

## Scope and version numbering

This history covers the original C#/WinForms source supplied for the port, the archived Rust development notes, and every Git commit reachable from `feature/validated-forest-ui` through [efbad1a](https://github.com/legluondunet/Z3-Launcher/commit/efbad1a83f12694c79ba1ebacef9028040cb358d).

The milestone numbers below follow the project's proposed progression. They organize development history; they do **not** imply that matching release tags or binaries were published:

| Milestone | Meaning |
| --- | --- |
| 0.1.0 | Original launcher import, Rust rewrite, native Linux/Windows support and AppImage packaging |
| 0.2.0 | Interface localization, completed with five bundled languages |
| 0.3.0 | Prebuilt game downloads replace dependency installation and local game compilation |
| 1.0.0 — Unreleased | Complete forest-interface redesign and release preparation |

Earlier preparation builds used numbers such as 0.6, 0.7 and 0.11.16. These are retained in the legacy history below. Some milestones overlap: features were refined across several development stages. This document does not rewrite old Git tags, alter Cargo's version, or publish a release.

## 0.1.0 — Source import and Rust foundation

### Original project and rewrite

- Started from **Zelda 3 Launcher**, the C#/WinForms Windows application by Anthony Johns (RadzPrower), originally licensed under MIT.
- Used the original installation, configuration and input-mapping workflow as the reference for the Rust implementation.
- Reimplemented the launcher in Rust with an egui/eframe desktop interface, a native file picker and SDL2 controller support. The external Zelda3 game remained a separate C project.
- Added native Linux support with X11/Wayland integration and retained native Windows support.
- Added a non-GUI command-line path and workspace selection with `--dir`.
- Added ROM selection by file picker and drag-and-drop, accepting `.sfc` and `.smc`, including paths with spaces and accents.
- Verified the supported headerless US ROM by SHA-256 before extraction.
- Ran installation/build operations in background threads and streamed process output to the interface and `launcher.log`.

### Original source-based installation workflow

- Added recursive Git checkout, submodule synchronization, resource extraction and Make-based game compilation.
- Updated existing checkouts with `git pull --ff-only` rather than destructive resets.
- Added complete rebuilding through `make clean_obj`, preserving configuration, saves and imported language resources.
- Added dependency checks for Git, Python, Pillow, PyYAML, Make, C compilation and SDL2 development files.
- Detected Linux distribution families through `/etc/os-release`, including Debian/Ubuntu, Arch/Manjaro, Fedora/Nobara and openSUSE.
- Reported all missing dependencies and deduplicated suggested package-install commands; avoided unsupported commands for known immutable distributions.
- Added Windows MSYS2/UCRT64 tool discovery, bounded dependency probes, an official bootstrap installer and confirmed dependency installation.
- Fixed MSYS2 installer waiting and made system tools available to package installation hooks.
- Changed the dependency button from verification to confirmed installation when dependencies were missing.
- These game-build and dependency-installation features were subsequently removed in milestone 0.3.0.

### Configuration and controls

- Recreated gameplay, display, renderer, audio, MSU/OPUZ, keyboard, controller and shortcut settings.
- Preserved INI comments, unknown keys, sections and line endings during edits.
- Added validated saves, temporary-file replacement and `zelda3.ini.bak` backups.
- Added independent keyboard/controller mappings for the twelve SNES controls and additional game shortcuts.
- Added QWERTY, AZERTY and QWERTZ presets, single-input capture, sequential assignment, clearing and controller-default restoration.
- Added Ctrl/Alt/Shift combinations, capture cancellation with Escape and a 15-second capture timeout.
- Added SDL controller detection, hotplug handling, logical button names and trigger capture; manual selection remains available without a connected controller.
- Added separate keyboard/controller shortcuts for game actions, cheats, rendering, replays and ten load/save/replay slots.
- Added compatible game-language ROM import, language/hash verification, dialogue/font extraction, reimport and resource regeneration.
- Kept the US ROM as the game base, retained previously imported languages and rolled back failed imports.
- Added ZSPR sprite selection and shader selection.

### Platform, packaging and project identity

- Added portable mode through a `portable.txt` file next to the executable.
- Added AppImage `.home` support, redirecting HOME and XDG directories to the matching portable folder.
- Kept data and preferences independent of the terminal's working directory; did not silently move existing user data.
- Added Linux AppImage build scripts, desktop integration, bundled dynamic libraries, fonts and translations.
- Restored the host library environment for tools and the game launched from the AppImage.
- Added GitHub Actions jobs for Windows, Linux and AppImage builds and tests; moved the workflow to the recognized `.github/workflows/` location.
- Hid the console for the Windows GUI while preserving command-line operation.
- Added a gold three-triangle icon for the header, window, taskbar, Linux desktop entry and AppImage; aligned the Wayland application ID.
- Added single-instance protection on Linux and Windows. A second launch activates and restores the existing window instead of displaying an error dialog.
- Renamed the application, executable, artifacts and data/configuration directories to **Z3-Launcher**.
- Applied GPL-3.0-or-later to the Rust fork, attributed it to legluondunet, and preserved the original MIT notice and DejaVu license.
- Added platform, translation, licensing and build documentation.

## 0.2.0 — Multilingual interface

- Added an interface-language selector, initially English and French.
- Applied language changes immediately and persisted the choice independently of the game's language and INI file.
- Embedded catalogs so translations remain available without external files.
- Added external JSON catalog overrides and new-language support without recompilation.
- Added fallback to embedded English for missing messages or invalid placeholders.
- Added safe placeholder substitution and catalog-validation tooling, including checks for duplicated JSON keys and inconsistent parameters.
- Localized launcher actions, progress, errors, settings, controller capture and contextual help.
- Added 77 contextual help messages adapted from the original launcher.
- Completed bundled **English, French, German, Spanish and Italian** translations.
- Added and corrected translations for INI unsaved changes and successful automatic saves during the subsequent interface redesign.
- Left external program output in its original language and retained earlier log entries when changing the interface language.

## 0.3.0 — Prebuilt game installation and updates

### Installation architecture

- Replaced local source checkout, dependency installation and C compilation with downloads from **legluondunet/zelda3** public GitHub releases.
- Selected the supported x86_64 package for Linux or Windows.
- Removed the dependency-check/install and game-build actions from the normal launcher workflow.
- Used a bundled standalone resource extractor, removing the need for system Python, Pillow or PyYAML to install the distributed game.
- Continued extracting `zelda3_assets.dat` locally from the user's verified US ROM and installed it beside the game executable.
- Retained a local ROM copy for asset regeneration during subsequent updates.
- Preserved imported dialogue/font resources when migrating older source-based installations.
- Recorded the installed release tag in `.release-version`.
- Simplified the CLI to installation, update, run and status actions.

### Download and update protection

- Verified release packages against their published `SHA256SUMS`.
- Limited downloaded and expanded archive sizes.
- Rejected unsafe archive paths, symbolic links and unexpected top-level package entries.
- Checked that the executable, extractor and default configuration were present.
- Restored executable permissions where required.
- Staged updates before replacing the installed package and rolled back already replaced files if installation failed.
- Preserved existing configuration, saves, MSU music and imported languages.
- Protected long installation/import operations from accidental window closure.
- Kept public downloads independent of a GitHub account or token.

### Rendering, shaders and update notifications

- Made **OpenGL** the default renderer for new installations and missing renderer settings, while preserving existing explicit choices.
- Restricted shaders to compatible `.glsl` and `.glslp` files; rejected unsupported `.slangp` presets after an earlier experimental filter had admitted them.
- Enabled shader controls only for OpenGL/OpenGL ES without deleting stored shader paths when another renderer is selected.
- Added retrieval of the Libretro GLSL shader collection, staged installation, reuse of complete installations and preservation of additional user files.
- Added background startup checks for game and launcher releases with bounded network timeouts.
- Displayed available updates as links in the persistent status bar.
- Compared the game release against the installed tag and checked package compatibility; compared launcher versions numerically.
- Handled missing releases and network failures without preventing startup.

### Compatibility fixes

- Accepted `LICENSE.upstream.txt`, `COPYING`, `VERSION` and `BUILD-INFO.txt` in the updated game packages. Previously these legitimate files triggered the misleading unsafe-archive error.
- Preferred the host's x86_64 `libxkbcommon.so.0` when launching an AppImage to address Escape/Alt closures observed with the bundled library.
- Retained the bundled keyboard library as fallback and preserved existing preloads.
- Restored the original `LD_PRELOAD` for child programs, alongside the existing `LD_LIBRARY_PATH` restoration.

## 1.0.0 — Unreleased — Forest interface redesign

### Theme and navigation

- Introduced a forest-green and gold theme with parchment-colored DejaVu Serif text and an embedded background.
- Adopted the validated forest artwork and refined the palette to match the approved mockup.
- Added translucent green panels, gold borders, rounded corners, gold headings and column dividers.
- Styled the selected tab with a gold fill; distributed tabs across the window and adjusted their font size, height and spacing.
- Removed the nested Options page and placed settings directly in the main tab bar.
- Renamed display and sound pages to Graphics and Audio.
- Merged Keyboard and Gamepad into **Controls**, with distinct sections.
- Renamed the first Graphics frame to **General**.
- Adjusted tab indices and preserved validation when leaving the INI editor.

### Frames, forms and layout

- Stacked Game and Audio frames vertically and centered them at 85% width, with two-column contents.
- Applied matching frames to Graphics, Controls and Shortcuts.
- Distributed graphical settings, controller/keyboard commands and shortcut slots across two internal columns.
- Refined frame widths to match the approved layout and equalized section widths.
- Added framed, translucent sections to the launcher's General page.
- Centered gold dividers between actual content edges, balanced gutters and added clearance for left-column controls.
- Extended the MSU divider and made its path field more compact.
- Gave each settings tab its own scroll position.
- Improved label, field and button alignment; reserved fixed-width controller dropdown slots and standardized capture/action buttons.
- Made shortcut/input rows adapt to available space and corrected row spacing.
- Improved form readability, field contrast and translucent input backgrounds.

### Log, status bar and INI editor

- Made the log selectable and read-only, with Ctrl+C and context-menu copy for either the selection or the full log.
- Preserved selection on right-click; retained copy/clear actions.
- Improved log text contrast and reduced empty-log space while keeping footer actions visible.
- Moved action messages and activity indication into a persistent bottom status bar.
- Added automatic saving of valid form changes; removed redundant Save/Reload controls and repeated notices.
- Kept the last valid configuration intact when an edit or write failed; added retry/discard handling.
- Added one translated confirmation after an automatic save and a notice that settings apply on the next game launch.
- Replaced external INI editing with a dedicated integrated tab.
- Saved and refreshed forms when leaving the editor; kept invalid drafts open for correction and guarded closure when changes could not be saved.
- Added unsaved-change feedback in the status bar and moved editing guidance there.
- Framed the INI editor, placed the save explanation below the configuration path and expanded the frame to fill the available tab height.

### Implementation fixes and documentation

- Explicitly typed gold-border stroke widths to avoid newer compiler warnings.
- Fixed conflicting egui borrows while drawing dividers.
- Corrected cursor positioning to use a rectangle, then replaced a private cursor method with supported layout APIs.
- Fixed malformed newline character literals in shortcut and binding editors.
- Rechecked new status messages across all five translations.
- Rewrote the GitHub README in English for the current installer/updater/configuration workflow, platform support, ROM requirements, portable mode, licensing and developer instructions.
- Added the forest artwork to the README. At the covered revision it is identified as artwork; a full application screenshot is still pending.
- Kept final release publication separate from these development changes.

## Legacy preparation builds

The archived pre-import README records the following original development labels. These are historical labels, not additional releases created by this changelog.

| Original label | Recorded change |
| --- | --- |
| 0.1 | Rust/Linux prototype; user-confirmed build and US-ROM resource extraction |
| 0.4 | Expanded settings/input implementation and source tests; execution was not verified in the preparation environment |
| 0.6 | Distribution-aware dependency reports and package suggestions; interim Windows/macOS changes were reverted at this stage |
| 0.7 | English/French catalogs, immediate language switching, external overrides and validation; fixed module-comment placement |
| 0.8 | Restored game-language ROM import, validation, resource regeneration and rollback |
| 0.9 | Portable marker, fixed portable workspace and portable preferences |
| 0.11.3 | Direct main tabs and automatic saving |
| 0.11.4 | Selectable log; source update and full rebuild while preserving imported resources |
| 0.11.5 | Simplified visible actions; temporary expansion of the shader filter |
| 0.11.6 | Main-action naming and contextual help |
| 0.11.7 | External INI editor and persistent GLSL shader library; removed unsupported Slang presets |
| 0.11.8 | Renderer-dependent shader controls |
| 0.11.9 | Embedded forest/gold theme and font |
| 0.11.10 | Explicit float stroke types |
| 0.11.11 | Integrated INI editor and invalid-draft preservation |
| 0.11.12 | Revised log layout, tab styling and persistent status bar |
| 0.11.13 | More compact settings and framed gameplay sections |
| 0.11.15 | GPL-3.0-or-later attribution and Z3-Launcher identity |
| 0.11.16 | Updated data/configuration paths and portable folder naming |

The archived notes use 0.11.4 and 0.11.15 for more than one change. Labels absent from those notes are not assigned invented changes.

## Complete Git chronology

The ledger below includes all 104 commits reachable from the covered branch revision, including imports, merges, translation-only commits and corrective follow-ups. Dates use UTC. Earlier work bundled into the initial source upload is described above rather than given fabricated commit dates.

- 2026-10-08 — [`d6373b9`](https://github.com/legluondunet/Z3-Launcher/commit/d6373b94ce2e7be383f3367656f39f050a698951) — Add files via upload
- 2026-10-08 — [`b83ca1b`](https://github.com/legluondunet/Z3-Launcher/commit/b83ca1b9d192fd83ba444230c7f6ffdb47642706) — Add files via upload
- 2026-10-08 — [`e454219`](https://github.com/legluondunet/Z3-Launcher/commit/e454219a938d9da8ad38b505ef0e5b37aea81d9b) — Delete Z3-Launcher.zip
- 2026-10-08 — [`ba55994`](https://github.com/legluondunet/Z3-Launcher/commit/ba5599427de19bcb1dca91d2552573d5421869ad) — Add files via upload
- 2026-10-08 — [`f21ddc9`](https://github.com/legluondunet/Z3-Launcher/commit/f21ddc965f61e2ab5e28c24c18ba19a588793e4f) — Rename workflows/build.yml to .github/workflows/build.yml
- 2026-10-08 — [`18c6568`](https://github.com/legluondunet/Z3-Launcher/commit/18c6568efc94e2009cce51e9a7936ff50f4f87d0) — Update build.yml
- 2026-10-08 — [`806b48f`](https://github.com/legluondunet/Z3-Launcher/commit/806b48f2c8dff5f8cb8c7c103057e4e835b1dacd) — Hide Windows console in graphical launcher: main
- 2026-10-08 — [`f47bc86`](https://github.com/legluondunet/Z3-Launcher/commit/f47bc868f86be5aba28d9967fa93124aa1303b4c) — Hide Windows console in graphical launcher: platform
- 2026-10-08 — [`4e6cbcc`](https://github.com/legluondunet/Z3-Launcher/commit/4e6cbcc084a787c6c8f1e9526586cf9128601f74) — Hide Windows console in graphical launcher: windows
- 2026-10-08 — [`33c4711`](https://github.com/legluondunet/Z3-Launcher/commit/33c4711c7d1149d345f8fa26632255c1d636b420) — Generate native window icon from the header's gold triangle emblem
- 2026-10-08 — [`ee02b99`](https://github.com/legluondunet/Z3-Launcher/commit/ee02b99c47286e6aa11c64f747c206504f642535) — Set the launcher window and taskbar icon
- 2026-10-08 — [`7b88d8b`](https://github.com/legluondunet/Z3-Launcher/commit/7b88d8b43b5aab74f0c7fd90d59937c45e0fda9d) — Use the same three gold triangles for the Linux desktop and AppImage icon
- 2026-10-08 — [`86f313a`](https://github.com/legluondunet/Z3-Launcher/commit/86f313ab59573180dd6f161623081bad16828868) — Match the Wayland application ID to the Linux desktop entry
- 2026-10-08 — [`a5007c8`](https://github.com/legluondunet/Z3-Launcher/commit/a5007c88945b89adeb70a1985b167e11a783273f) — Distribute tabs across the window and increase labels to 12 points
- 2026-10-08 — [`d3b54de`](https://github.com/legluondunet/Z3-Launcher/commit/d3b54de8b8806eeb0da209b1274b7ae183f9606a) — Bound Windows dependency probes and resolve tools inside MSYS2
- 2026-10-08 — [`567540b`](https://github.com/legluondunet/Z3-Launcher/commit/567540b86645a95da5dc5a0fdc0c49bad8fb0852) — Clarify dependency verification and localize progress and timeout errors
- 2026-10-08 — [`689cb78`](https://github.com/legluondunet/Z3-Launcher/commit/689cb78cd5fd1dc6cb77bc2ce368b49301630e86) — Clarify dependency verification and localize progress and timeout errors
- 2026-10-08 — [`aef1998`](https://github.com/legluondunet/Z3-Launcher/commit/aef1998b93fe9e143134f25f24f3a93ebe3f0f47) — Add verified official MSYS2 bootstrap installer
- 2026-10-08 — [`4d8d7db`](https://github.com/legluondunet/Z3-Launcher/commit/4d8d7dbfa5dec54ec965a9499d582344070d5f6a) — Add confirmed dependency installation: locales/en.json
- 2026-10-08 — [`c517d8e`](https://github.com/legluondunet/Z3-Launcher/commit/c517d8eb9d44c690ba700969d1efcd3fd5d18155) — Add confirmed dependency installation: locales/fr.json
- 2026-10-08 — [`2c4f01f`](https://github.com/legluondunet/Z3-Launcher/commit/2c4f01fea7119b12e8611ca97f965d4877062809) — Add confirmed dependency installation: src/dependencies.rs
- 2026-10-08 — [`e1ddebd`](https://github.com/legluondunet/Z3-Launcher/commit/e1ddebd3fbf4e1aef2efc65b661529a09b19f1a2) — Add confirmed dependency installation: src/windows.rs
- 2026-10-08 — [`dc005ce`](https://github.com/legluondunet/Z3-Launcher/commit/dc005ce9995065072c2b9fa990d24ab92fab7930) — Add confirmed dependency installation: src/core.rs
- 2026-10-08 — [`fb7c952`](https://github.com/legluondunet/Z3-Launcher/commit/fb7c95230b3493be524a1e6c39b8067aa1dc312a) — Add confirmed dependency installation: src/ui.rs
- 2026-10-08 — [`b12f503`](https://github.com/legluondunet/Z3-Launcher/commit/b12f50302fc62a8a0ecbfdbc7f1e9dd46ae99933) — Add confirmed dependency installation: WINDOWS.md
- 2026-10-08 — [`c7b294d`](https://github.com/legluondunet/Z3-Launcher/commit/c7b294d076bcfb066531efa9047817e952295389) — Expose MSYS2 system tools to pacman installation hooks
- 2026-10-08 — [`e41f402`](https://github.com/legluondunet/Z3-Launcher/commit/e41f4023285cd22605c31eb8cd02b1719121c7d1) — Wait for the MSYS2 GUI installer to exit before initialization
- 2026-10-08 — [`94688cc`](https://github.com/legluondunet/Z3-Launcher/commit/94688cc16fa5ed9f36e53aceaafc5897cd158415) — Merge pull request #1 from legluondunet/fix/windows-gui-console
- 2026-10-08 — [`a9aa587`](https://github.com/legluondunet/Z3-Launcher/commit/a9aa587ef9221dc30f61a5a400fb18f8c90087fd) — Merge pull request #2 from legluondunet/feat/window-triangle-icon
- 2026-10-08 — [`2c7acff`](https://github.com/legluondunet/Z3-Launcher/commit/2c7acfffd9bdf55f38dd6e9f0c161e5c7d895b90) — Merge pull request #3 from legluondunet/ui/full-width-tabs
- 2026-10-08 — [`450e702`](https://github.com/legluondunet/Z3-Launcher/commit/450e7025dcdf2078d6dfb47cf6743a46877c6de8) — Merge main into dependency installation branch, retaining Windows console helpers
- 2026-10-08 — [`c3e2c3e`](https://github.com/legluondunet/Z3-Launcher/commit/c3e2c3e216ca6b4d1b76a235ddac3be680907337) — Merge pull request #4 from legluondunet/fix/windows-dependency-check
- 2026-10-08 — [`82fd251`](https://github.com/legluondunet/Z3-Launcher/commit/82fd2516163327edb9ff2a814edb4a1cf714ab98) — Prevent simultaneous graphical launcher instances on Windows and Linux
- 2026-10-08 — [`611e7f3`](https://github.com/legluondunet/Z3-Launcher/commit/611e7f3e729437023355d6b13e14be7aff427038) — Activate and restore the existing launcher instead of showing a dialog
- 2026-10-08 — [`03b815a`](https://github.com/legluondunet/Z3-Launcher/commit/03b815ac2c50c4b689b2fc1182db7c53c88c9072) — Switch the dependency check button to confirmed installation when needed
- 2026-10-08 — [`0de024d`](https://github.com/legluondunet/Z3-Launcher/commit/0de024da22173a7d47a45d923ed669830d6f1dd5) — Arrange graphics and audio settings in two columns and rename tabs
- 2026-10-08 — [`1333ae0`](https://github.com/legluondunet/Z3-Launcher/commit/1333ae0f130b601edc0e72905cba3cb1063f1862) — Add complete embedded Italian, Spanish and German interface translations
- 2026-10-08 — [`9038f80`](https://github.com/legluondunet/Z3-Launcher/commit/9038f808cf527b881d9989c2b6e8ecc00aa3e8bf) — Merge pull request #5 from legluondunet/fix/single-gui-instance
- 2026-10-08 — [`b4de30e`](https://github.com/legluondunet/Z3-Launcher/commit/b4de30ecda6323737712474d530e278eac398176) — Merge branch 'main' into ui/adaptive-dependency-button
- 2026-10-08 — [`3795de4`](https://github.com/legluondunet/Z3-Launcher/commit/3795de4816799ff1eeae786d04cf8db237ced2c5) — Merge pull request #6 from legluondunet/ui/adaptive-dependency-button
- 2026-10-08 — [`2d6a30d`](https://github.com/legluondunet/Z3-Launcher/commit/2d6a30da6364e0c0ef857427b58e32cc21c0a68e) — Merge branch 'main' into ui/graphics-audio-columns
- 2026-10-08 — [`d803d90`](https://github.com/legluondunet/Z3-Launcher/commit/d803d907903309604ce6845e01472c1dde28efcf) — Merge pull request #7 from legluondunet/ui/graphics-audio-columns
- 2026-10-08 — [`6fa3593`](https://github.com/legluondunet/Z3-Launcher/commit/6fa3593cbaff7fb6073121051e0306f4af4b68aa) — Merge pull request #8 from legluondunet/feat/italian-spanish-german
- 2026-10-08 — [`239e838`](https://github.com/legluondunet/Z3-Launcher/commit/239e838daec29e82aa0fb4a8255a8138aaee8b3c) — Download prebuilt Zelda3 releases instead of installing tools and compiling
- 2026-10-08 — [`23060fd`](https://github.com/legluondunet/Z3-Launcher/commit/23060fdac64dce4050f41468db72dcf2a4aba549) — Merge pull request #9 from legluondunet/codex/download-prebuilt-game
- 2026-10-08 — [`8dfb442`](https://github.com/legluondunet/Z3-Launcher/commit/8dfb442bfebf66dcd24501e85ff4197db48cc3ce) — Adjust tab and content font sizes and frame graphical options
- 2026-10-08 — [`32e9aaa`](https://github.com/legluondunet/Z3-Launcher/commit/32e9aaa8f13b852a9764745af8c1369494b4508f) — Use OpenGL as the default game renderer
- 2026-10-08 — [`c878afa`](https://github.com/legluondunet/Z3-Launcher/commit/c878afad7407f1b78a8744368716a7ccf4c442c3) — Merge pull request #10 from legluondunet/codex/tab-fonts-graphics-frames
- 2026-10-08 — [`40e3145`](https://github.com/legluondunet/Z3-Launcher/commit/40e314583214577967d07ae2e17eb425500f389d) — Stack Game and Audio sections and make settings frames translucent
- 2026-10-08 — [`cc275c1`](https://github.com/legluondunet/Z3-Launcher/commit/cc275c17de7fbf0f37d72a0552b3c83e57f4d85e) — Merge keyboard and gamepad settings into Controls tab
- 2026-10-08 — [`7abd426`](https://github.com/legluondunet/Z3-Launcher/commit/7abd42640d173694e265a041f6368b926d96c642) — Unify stacked translucent frames across settings tabs
- 2026-10-08 — [`ce426e1`](https://github.com/legluondunet/Z3-Launcher/commit/ce426e14ee971a69a6d3481da42ac025c26e9db4) — Arrange settings in two columns and check releases at startup
- 2026-10-09 — [`5e7276e`](https://github.com/legluondunet/Z3-Launcher/commit/5e7276e2a11b640df4bb8fd4ab06a03b748c2295) — Accept license and build metadata in Zelda3 release packages
- 2026-10-09 — [`6d1f56a`](https://github.com/legluondunet/Z3-Launcher/commit/6d1f56a4bb3436d46c874e739bee2646b6b095de) — Merge pull request #11 from legluondunet/codex/stacked-translucent-settings
- 2026-10-09 — [`8a54868`](https://github.com/legluondunet/Z3-Launcher/commit/8a54868416bbd8f5e321d919a42c921dcb620ae0) — Merge pull request #12 from legluondunet/codex/merge-keyboard-gamepad-tabs
- 2026-10-09 — [`5d56bad`](https://github.com/legluondunet/Z3-Launcher/commit/5d56bad47884ac9d3949dc770b590082062d0fc3) — Merge pull request #13 from legluondunet/codex/fix-release-package-notices
- 2026-10-09 — [`5907d6d`](https://github.com/legluondunet/Z3-Launcher/commit/5907d6df77223d9ff405c492a36623dc4c0f9ff0) — Prefer host xkbcommon for AppImage keyboard handling
- 2026-10-09 — [`070e174`](https://github.com/legluondunet/Z3-Launcher/commit/070e174842f6fd4160142b7769e2e377a50f1d32) — Restore original preload environment for child programs
- 2026-10-09 — [`753dfda`](https://github.com/legluondunet/Z3-Launcher/commit/753dfda4f7c9e660772fcffcf1dc0d285f1fa1c7) — Merge pull request #14 from legluondunet/codex/fix-appimage-xkbcommon
- 2026-10-09 — [`367238d`](https://github.com/legluondunet/Z3-Launcher/commit/367238dd1c80d8e5feaf88db93430a220280c613) — Rename graphics settings frame to General
- 2026-10-09 — [`081526d`](https://github.com/legluondunet/Z3-Launcher/commit/081526dc35182e477da83395c5646d8362825b3e) — Merge pull request #15 from legluondunet/codex/rename-graphics-general-frame
- 2026-10-09 — [`a9ff41f`](https://github.com/legluondunet/Z3-Launcher/commit/a9ff41fc0a6ecb4be84df4905953889595103894) — style: align forest palette and gold controls with approved mockup
- 2026-10-09 — [`d8b06d9`](https://github.com/legluondunet/Z3-Launcher/commit/d8b06d9b4678d47b851ca2a9cafd3af1e3f694f1) — style: refine tabs and footer for approved forest layout
- 2026-10-09 — [`9397b07`](https://github.com/legluondunet/Z3-Launcher/commit/9397b07ba725a0d730784c2a93d9221cd19666ea) — style: widen translucent settings panels to match approved layout
- 2026-10-09 — [`4d61efd`](https://github.com/legluondunet/Z3-Launcher/commit/4d61efdf9984e388f0d84226473219711af276d0) — style: add gold column dividers and restore translucent panels
- 2026-10-09 — [`98e764f`](https://github.com/legluondunet/Z3-Launcher/commit/98e764f5f7b02fa8e8492f5e5be48f2a2ba0a8f0) — Add files via upload
- 2026-10-09 — [`b835d4f`](https://github.com/legluondunet/Z3-Launcher/commit/b835d4ff3fa767b5d0f1b0346509df7ef69c3d29) — fix: avoid conflicting egui borrow when drawing column dividers
- 2026-10-09 — [`1d4df18`](https://github.com/legluondunet/Z3-Launcher/commit/1d4df18c88fae792660aa3d770c6ed273ffccb4c) — style: give left-column controls clearance from centered gold dividers
- 2026-10-09 — [`19a3b20`](https://github.com/legluondunet/Z3-Launcher/commit/19a3b200d7d7772fbf7a411cfbcb5ec5299cfe8b) — style: enlarge tabs and refine spacing below launcher header
- 2026-10-09 — [`3d351c0`](https://github.com/legluondunet/Z3-Launcher/commit/3d351c093386b82a44adac1c9b0eb5ef63dc2b06) — style: center gold dividers with balanced gutters and improve panel readability
- 2026-10-09 — [`5262b5b`](https://github.com/legluondunet/Z3-Launcher/commit/5262b5b095a61c7994c8a6cfaf023816ac074882) — fix: pass a rectangle to egui cursor positioning
- 2026-10-09 — [`7f89c3e`](https://github.com/legluondunet/Z3-Launcher/commit/7f89c3ea0d727f62283d0b1bc3c55ce60065e8b8) — style: improve form text readability and shrink empty log panel
- 2026-10-09 — [`c30cf5e`](https://github.com/legluondunet/Z3-Launcher/commit/c30cf5ef85eb98543ab706d2d8bb20c461e80d1b) — style: align controller binding fields and use uniform action buttons
- 2026-10-09 — [`b08c8d9`](https://github.com/legluondunet/Z3-Launcher/commit/b08c8d933dacd8fba2d1ad2ba4ccefef6d7df199) — fix: replace private egui set_cursor call with supported layout API
- 2026-10-09 — [`1c71999`](https://github.com/legluondunet/Z3-Launcher/commit/1c71999e1608ebe6db28f04c2f0e2036df0672d3) — style: center gold divider between actual content edges and isolate tab scroll positions
- 2026-10-09 — [`c7bcb5b`](https://github.com/legluondunet/Z3-Launcher/commit/c7bcb5b38043f9be9d61397a4f9a1aa3285d5a89) — layout: responsive shortcut rows and compact MSU path field; remove repeated settings notice
- 2026-10-09 — [`ed35e7e`](https://github.com/legluondunet/Z3-Launcher/commit/ed35e7e1c8d76ad4719372dff28f29604a8c9bb9) — style: improve spacing in keyboard and gamepad binding rows
- 2026-10-09 — [`ed94875`](https://github.com/legluondunet/Z3-Launcher/commit/ed948757cfe7a9521861e465702d0bdd28564add) — status: show restart notice alongside successful option autosave
- 2026-10-09 — [`0ca4d0e`](https://github.com/legluondunet/Z3-Launcher/commit/0ca4d0e1207f0b02165a2bf7aff1bbe8bff13f60) — fix: use valid Rust newline character literal in shortcut editor
- 2026-10-09 — [`b49c611`](https://github.com/legluondunet/Z3-Launcher/commit/b49c6111a07df89d46f88425e3a40348daf946be) — layout: equalize frame widths, extend MSU divider, align gamepad labels
- 2026-10-09 — [`b2a7421`](https://github.com/legluondunet/Z3-Launcher/commit/b2a74217e5bd8942f77d84dadd8b90079064d75b) — style: add translucent gold-framed panels to launcher general tab
- 2026-10-09 — [`c318796`](https://github.com/legluondunet/Z3-Launcher/commit/c31879601e8d67c49507df1ac5c3fe8d5eb3d5cf) — status: show a single confirmation after automatic settings save
- 2026-10-09 — [`658d17d`](https://github.com/legluondunet/Z3-Launcher/commit/658d17db5ac5a6601b92730ea8c2e5f747069eb2) — layout: reserve fixed-width slots for gamepad dropdowns and align capture buttons
- 2026-10-09 — [`6bae969`](https://github.com/legluondunet/Z3-Launcher/commit/6bae96936219dc1c077b7e739b20de3c74487f9d) — style: improve contrast of text fields and use translucent forest-green input backgrounds
- 2026-10-09 — [`8a0c888`](https://github.com/legluondunet/Z3-Launcher/commit/8a0c888679cd72e1dbd4639e46f77c9ee6822524) — style: render journal text in high-contrast off-white
- 2026-10-09 — [`4058fc6`](https://github.com/legluondunet/Z3-Launcher/commit/4058fc681670f038c982a25963f04b7572e56254) — ui: move INI editing guidance from editor header to status bar
- 2026-10-09 — [`a3edede`](https://github.com/legluondunet/Z3-Launcher/commit/a3edede904ea8b38107920130043d5bf7192ea22) — ui: align gamepad mapping rows with fixed-width controls
- 2026-10-09 — [`448fb42`](https://github.com/legluondunet/Z3-Launcher/commit/448fb424d5a8afd6b700aa207dfaad08328ba377) — ui: frame INI editor and move autosave explanation below file path
- 2026-10-09 — [`e07ea19`](https://github.com/legluondunet/Z3-Launcher/commit/e07ea19e93f67ab9bbc9a3e0772b53b6789fd92b) — i18n: add INI unsaved changes status
- 2026-10-09 — [`d65902a`](https://github.com/legluondunet/Z3-Launcher/commit/d65902a42aa5daf027ec7b7493f159e512a1acfd) — i18n: add INI unsaved changes status
- 2026-10-09 — [`e18c2f7`](https://github.com/legluondunet/Z3-Launcher/commit/e18c2f7aba5ce19d2ec27a85ed61773952038c0d) — status: flag unsaved INI changes while editing
- 2026-10-09 — [`32b55a3`](https://github.com/legluondunet/Z3-Launcher/commit/32b55a391990531c306cd9b40c4397292ba1f8b9) — ui: connect INI editor dirty state to status bar
- 2026-10-09 — [`e3f501e`](https://github.com/legluondunet/Z3-Launcher/commit/e3f501e2a2ab2fac83f8621bbe5aee7c910fa140) — fix: use valid newline character literal in binding editor
- 2026-10-09 — [`44fd1b9`](https://github.com/legluondunet/Z3-Launcher/commit/44fd1b96a425ec2c0c2620d8a57dda3c51496dea) — i18n: add missing INI unsaved status in de
- 2026-10-09 — [`39e5197`](https://github.com/legluondunet/Z3-Launcher/commit/39e5197f739c2ca8de481c36d96289af60b4c494) — i18n: add missing INI unsaved status in es
- 2026-10-09 — [`8a4577e`](https://github.com/legluondunet/Z3-Launcher/commit/8a4577ed16310847b407cee1df1a794ce91502d2) — i18n: add missing INI unsaved status in it
- 2026-10-09 — [`834cdee`](https://github.com/legluondunet/Z3-Launcher/commit/834cdee3ae59b7947b5d85fce4920d2211a14e7b) — i18n: translate automatic save confirmation (en)
- 2026-10-09 — [`dffe1e6`](https://github.com/legluondunet/Z3-Launcher/commit/dffe1e6678479996fdb636e0f189baf561fc6ac5) — i18n: translate automatic save confirmation (fr)
- 2026-10-09 — [`f3627c5`](https://github.com/legluondunet/Z3-Launcher/commit/f3627c55f4a4ea19d37dcdb8b1a93812feb11b93) — i18n: translate automatic save confirmation (de)
- 2026-10-09 — [`4f5d43b`](https://github.com/legluondunet/Z3-Launcher/commit/4f5d43b886df5b2bd33bd8dfff168b93417c78a1) — i18n: translate automatic save confirmation (es)
- 2026-10-09 — [`12e1e1f`](https://github.com/legluondunet/Z3-Launcher/commit/12e1e1fb0f57e0ce4ebb1c4af885b1dec1693a04) — i18n: translate automatic save confirmation (it)
- 2026-10-09 — [`12898fe`](https://github.com/legluondunet/Z3-Launcher/commit/12898fea2a60350a384996b99dba3beed575a86b) — i18n: use translated status for successful automatic save
- 2026-10-09 — [`ea600e3`](https://github.com/legluondunet/Z3-Launcher/commit/ea600e39b140706459fe7cfc728340b9a1bdb43f) — ui: expand INI editor frame to fill available tab height
- 2026-10-09 — [`efbad1a`](https://github.com/legluondunet/Z3-Launcher/commit/efbad1a83f12694c79ba1ebacef9028040cb358d) — docs: refresh README for current launcher features and forest theme
