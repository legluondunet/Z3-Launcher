# AppImage — Linux x86_64

## Build

The published AppImage is built on Ubuntu 22.04 to target systems with glibc 2.35 or later. Native host graphics drivers, OpenGL, and a working desktop environment are still required. It is not a universal musl/Alpine package.

To build locally, install Rust (stable) and the necessary native build packages on Ubuntu 22.04:

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config cmake curl file python3 libsdl2-dev libxkbcommon-dev libwayland-dev libx11-dev libxcursor-dev libxi-dev libxrandr-dev libgl1-mesa-dev
bash ./build-appimage.sh
```

The script tests translation catalogs, builds Z3-Launcher in release mode, bundles the required libraries, translations and licenses, and writes `dist/Z3-Launcher-<version>-x86_64.AppImage` and the matching `.sha256` file. CI stamps the Cargo version with the short commit hash, for example `2.0.1+abcdef0`. The script downloads linuxdeploy from its official release if it is not cached; to use a specific version, set `LINUXDEPLOY` to an existing absolute path.

## Run

```bash
chmod +x Z3-Launcher-2.0.1+abcdef0-x86_64.AppImage
./Z3-Launcher-2.0.1+abcdef0-x86_64.AppImage
```

If FUSE is unavailable, try `--appimage-extract-and-run`. A compatible user-provided ROM is required to generate Z3R resources. The launcher downloads a prebuilt native Z3R Linux package, validates its GitHub SHA-256 digest, and generates resources locally with its bundled Z3R-specific extractor. It does **not** compile the game from source, and its user does **not** need Rust, Python or Make. Z3R's native Linux executable requires a working SDL2 runtime.

## Automatic fallback from Wayland to XWayland

When the native Wayland OpenGL/EGL startup fails with the specific glutin error `provided display handle is not supported`, Z3-Launcher automatically retries **once** under X11/XWayland, provided both `WAYLAND_DISPLAY` and `DISPLAY` are present. Native Wayland remains the default when it works. Other startup errors are not retried, and an X11/XWayland display must be available for the fallback.

This reproduces the workaround confirmed in [issue #34](https://github.com/legluondunet/Z3-Launcher/issues/34):

```bash
env -u WAYLAND_DISPLAY ./Z3-Launcher-*.AppImage
```

To disable automatic retry for diagnosis, launch with `Z3_DISABLE_X11_FALLBACK=1`. The retry runs in a separate process because the windowing event loop cannot safely be recreated in the same process. This is a compatibility workaround, not a general correction of the host EGL drivers.

## Command-line diagnostics

Run `./Z3-Launcher-<version>-x86_64.AppImage --version` to print its embedded version and build commit. Add `--verbose` to an action such as `status`, `setup`, `update` or `run` for additional diagnostics. `run --verbose` also prints game messages to the terminal, while `zelda3.log` stays separate from `Z3-Launcher.log` on disk.

## Portable mode

- **`portable.txt`:** Create this empty file beside the *original AppImage*. Game files, settings, shaders and `Z3-Launcher.log` are stored beside that file (or in its subdirectories), rather than in an extra `Z3-Launcher/` workspace.
- **AppImage `.home` mode:** Without `portable.txt`, a folder named exactly `<full AppImage filename>.home` redirects HOME and XDG data/configuration to that folder. Renaming or moving the AppImage requires renaming or moving the matching `.home` directory.
- If both are present, **`portable.txt` takes priority**. Mode changes require a restart and do not automatically migrate prior data.

## Custom background and ROM discovery

- To override the embedded forest image without rebuilding, place `config/background.png` or `background.png` beside the original AppImage. A normal per-user configuration path is a further fallback. Invalid images are skipped, and the embedded image remains the final fallback. The image is tiled (repeated), not stretched.
- A compatible US `.sfc` or `.smc` beside the original AppImage is detected by SHA-256 and preselected. This does not require `portable.txt`.

## Separate logs

The launcher writes `Z3-Launcher.log` and saves Z3R runtime output to `zelda3.log` beside it, at the workspace root. With `portable.txt`, both logs are next to the original AppImage. Without portable mode, both stay together in the user's data directory. Use the two journal views inside Z3-Launcher to inspect them.

## Validation before publication

Use the `main` GitHub Actions workflow to verify build and unit tests on Linux, Windows, AppImage and experimental Flatpak. Download the **current commit's** AppImage artifact for Linux runtime checks: launching on X11/Wayland, ROM selection, installation/update, controllers, language changes, window resizing, portable mode, external backgrounds, logging, and paths with spaces or accented characters. For checksum verification, use the adjacent `.sha256` file. A successful compilation alone does not validate runtime behavior on every distribution.
