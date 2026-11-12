# AppImage — Linux x86_64

## Build

The published AppImage is built on Ubuntu 22.04 to target systems with glibc 2.35 or later. Native host graphics drivers, OpenGL, and a working desktop environment are still required. It is not a universal musl/Alpine package.

To build locally, install Rust (stable) and the necessary native build packages on Ubuntu 22.04:

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config cmake curl file python3 libsdl2-dev libxkbcommon-dev libwayland-dev libx11-dev libxcursor-dev libxi-dev libxrandr-dev libgl1-mesa-dev
bash ./build-appimage.sh
```

The script tests translation catalogs, builds Z3-Launcher in release mode, bundles the required libraries, translations and licenses, and writes `dist/Z3-Launcher-<version>-x86_64.AppImage` and the matching `.sha256` file. CI stamps the Cargo version with the short commit hash, for example `1.2.0+abcdef0`. The script downloads linuxdeploy from its official release if it is not cached; to use a specific version, set `LINUXDEPLOY` to an existing absolute path.

## Run

```bash
chmod +x Z3-Launcher-1.2.0+abcdef0-x86_64.AppImage
./Z3-Launcher-1.2.0+abcdef0-x86_64.AppImage
```

If FUSE is unavailable, try `--appimage-extract-and-run`. A compatible user-provided ROM is required to generate Zelda 3 resources. The launcher downloads a prebuilt Zelda 3 game package, validates its published SHA-256, and generates resources locally; it does **not** compile the game from source, and its user does **not** need Rust or Make.

## Portable mode

- **`portable.txt`:** Create this empty file beside the *original AppImage*. Game files, settings, shaders and `Z3-Launcher.log` are stored beside that file (or in its subdirectories), rather than in an extra `Z3-Launcher/` workspace.
- **AppImage `.home` mode:** Without `portable.txt`, a folder named exactly `<full AppImage filename>.home` redirects HOME and XDG data/configuration to that folder. Renaming or moving the AppImage requires renaming or moving the matching `.home` directory.
- If both are present, **`portable.txt` takes priority**. Mode changes require a restart and do not automatically migrate prior data.

## Custom background and ROM discovery

- To override the embedded forest image without rebuilding, place `config/background.png` or `background.png` beside the original AppImage. A normal per-user configuration path is a further fallback. Invalid images are skipped, and the embedded image remains the final fallback. The image is tiled (repeated), not stretched.
- A compatible US `.sfc` or `.smc` beside the original AppImage is detected by SHA-256 and preselected. This does not require `portable.txt`.

## Validation before publication

Use the `main` GitHub Actions workflow to verify build and unit tests on Linux, Windows, AppImage and experimental Flatpak. Download the **current commit's** AppImage artifact for Linux runtime checks: launching on X11/Wayland, ROM selection, installation/update, controllers, language changes, window resizing, portable mode, external backgrounds, logging, and paths with spaces or accented characters. For checksum verification, use the adjacent `.sha256` file. A successful compilation alone does not validate runtime behavior on every distribution.
