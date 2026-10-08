# Z3-Launcher

Desktop launcher for Zelda3, for Windows and Linux x86_64. The launcher downloads the latest public release from [legluondunet/zelda3](https://github.com/legluondunet/zelda3/releases/latest). It does not compile the game or install build dependencies.

1. Select your headerless US Zelda: A Link to the Past ROM (`.sfc` / `.smc`).
2. Click **Download and install**.
3. Click **Launch game**.

The ROM must have SHA-256 `66871d66be19ad2c34c927d6b14cd8eb6fc3181965b6e517cb361f7316009cfb`. The ROM stays on your computer. No ROM or Nintendo resources are distributed or uploaded.

The release includes the game and a standalone resource extractor with its own Python runtime, Pillow and PyYAML. Git, Make, GCC, MSYS2 and a system Python are unnecessary. Linux uses the game's AppImage (bundled SDL2; FUSE is not required). Windows includes the game's DLLs.

The extractor generates `zelda3_assets.dat` locally and places it next to `zelda3.exe` or `zelda3.AppImage` in the workspace's `zelda3/` folder. The launcher starts the game from that folder. The US ROM is retained there to regenerate resources on update or when importing game languages.

**Update game** downloads the latest release and regenerates resources with its matching extractor. Existing `zelda3.ini`, saves, MSU packs and imported language resources are preserved. A failed download/extraction leaves the existing installation intact; file replacement failures are rolled back. Old source checkouts are retained during migration.

Public downloads require no GitHub account or token. The download is checked against the release's `SHA256SUMS`. Until the companion release workflow has run, the launcher reports that no public release is available.

Interface languages: English, French, Italian, Spanish and German. Game languages can still be imported from supported ROMs independently of the interface language. The GLSL shader archive is downloaded without external tools; a shader download failure does not prevent installation of the game.

## Terminal

```bash
z3-launcher setup --rom /path/to/zelda3.sfc
z3-launcher update
z3-launcher run
z3-launcher status
```

Use `--dir DIRECTORY` for another workspace. Portable mode continues to follow `portable.txt` or the AppImage's `.home` directory.

## Building the launcher (developers only)

Install Rust and the platform GUI development libraries (SDL2, X11/Wayland/OpenGL on Linux), then run `cargo test` and `cargo build --release`. `cargo test --no-default-features` checks the non-GUI launcher. `build-appimage.sh` packages the Linux launcher. These tools are for building the launcher itself, not prerequisites for users.

The project is GPL-3.0-or-later; see `LICENSING.md` for retained upstream notices.
