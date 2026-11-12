# Windows

Download the compiled Z3-Launcher from GitHub Actions, then run `z3-launcher.exe`. No MSYS2, Git, Python, Make or GCC installation is required to use it.

Choose the headerless US ROM and click **Download and install**. The launcher downloads the latest Windows x86_64 ZIP from `legluondunet/zelda3` Releases, verifies its SHA-256, and runs the included standalone extractor locally. The generated `zelda3_assets.dat` sits beside the game executable and its DLLs.

**Update game** preserves configuration, saves, MSU packs and imported game languages. Close the running game before updating. The graphical launcher and its subprocesses do not open console windows.

For a portable workspace, place an empty `portable.txt` beside `z3-launcher.exe`. The game, configuration, shaders and `Z3-Launcher.log` are stored in that same directory or its subdirectories, with no additional nested `Z3-Launcher` directory. An adjacent compatible US `.sfc` or `.smc` ROM is SHA-256 checked and preselected automatically, even without portable mode.

Developers building the launcher can use `cargo test --no-default-features` and `cargo build --release`; SDL2 is bundled during the Windows build. GitHub Actions builds the distributed launcher.
