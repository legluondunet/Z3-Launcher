# Z3-Launcher 2.0.1 — Wayland startup compatibility

This maintenance release addresses [issue #34](https://github.com/legluondunet/Z3-Launcher/issues/34), where Z3-Launcher could fail to start in certain Linux Wayland environments with:

```text
Found no glutin configs matching the template
Error: provided display handle is not supported
```

## Fix

- Try **native Wayland first** as before; no change on systems where it works.
- If eframe/glutin reports that **specific display-handle initialization failure**, and an X11/XWayland display is available, **retry the GUI once under XWayland** in a new process, matching the workaround confirmed by the reporter.
- Limit the retry to one attempt to avoid restart loops. Do not silently retry unrelated errors.
- An opt-out for troubleshooting is available: `Z3_DISABLE_X11_FALLBACK=1`.
- The launcher remains exclusive to **Z3R v0.2.4**, with its native Linux/Windows game downloads, bundled resource extractor, user settings, saves, portable mode and separate launcher/game logs unchanged.

The workaround depends on an available XWayland display; it does not fix the underlying host EGL/driver incompatibility. The original reporter's **manual XWayland workaround is confirmed**, but testing the automatic retry on that machine is still requested.

## Packages

- Linux x86_64 AppImage, with accompanying SHA-256 checksum
- Linux x86_64 native TAR.GZ
- Windows x86_64 ZIP
- Experimental Linux Flatpak

## Diagnostic fallback

If your environment still cannot launch the AppImage, try:

```bash
env -u WAYLAND_DISPLAY ./Z3-Launcher-*.AppImage
```

Please provide details of your desktop, GPU and any error output in [issue #34](https://github.com/legluondunet/Z3-Launcher/issues/34). Logs remain `Z3-Launcher.log` and `zelda3.log` in the configured workspace.

No Nintendo ROM, copyrighted Nintendo assets or extracted game resources are bundled. Z3-Launcher remains GPL-3.0-or-later and preserves original and Z3R upstream license notices.

legluondunet assisted by ChatGPT (OpenAI)
