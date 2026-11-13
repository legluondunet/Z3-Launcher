# SPDX-FileCopyrightText: 2026 legluondunet
# SPDX-License-Identifier: GPL-3.0-or-later
"""Standalone Z3R resource builder, bundling Z3R v0.2.4 asset scripts.

The extractor uses the matching Z3R upstream sources, Pillow and PyYAML.
Neither a ROM nor generated copyrighted assets are bundled.
"""
import argparse
import importlib
from pathlib import Path
import runpy
import shutil
import sys


def bundled_sources():
    source_root = Path(getattr(sys, "_MEIPASS", Path(__file__).resolve().parent.parent))
    source_assets = source_root / "assets"
    return source_root, source_assets


def self_test():
    """Exercise packaged Python imports and required data without a ROM.

    Unlike --help this detects missing PyInstaller modules and data files.
    This intentionally does not claim to verify asset generation or gameplay.
    """
    root, assets = bundled_sources()
    required_scripts = (
        "restool.py", "util.py", "tables.py", "extract_resources.py",
        "compile_resources.py", "sprite_sheets.py", "sprite_sheet_info.py",
        "extract_music.py", "compile_music.py", "text_compression.py",
    )
    for name in required_scripts:
        if not (assets / name).is_file():
            raise FileNotFoundError(f"Missing bundled Z3R asset script: {name}")
    for path in (assets / "palette_usage.bin", root / "other" / "3x5_font.png"):
        if not path.is_file() or not path.stat().st_size:
            raise FileNotFoundError(f"Missing bundled Z3R resource: {path}")
    # Compile all bundled sources to catch missing/corrupted scripts without
    # executing code that requires a ROM. compile_resources.py references
    # util.ROM at import time and cannot be imported before ROM loading.
    for name in required_scripts:
        script = assets / name
        compile(script.read_bytes(), str(script), "exec")

    sys.path.insert(0, str(assets))
    for name in ("util", "tables", "sprite_sheet_info", "sprite_sheets", "extract_resources"):
        importlib.import_module(name)
    print("Z3R resource builder self-test passed (imports, syntax and bundled data).")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--workspace")
    parser.add_argument("--check-language")
    parser.add_argument("--self-test", action="store_true",
                        help="Verify bundled asset modules and data without a ROM")
    args, forwarded = parser.parse_known_args()

    if args.self_test:
        self_test()
        return
    if not args.workspace:
        parser.error("--workspace is required except with --self-test")

    workspace = Path(args.workspace).resolve()
    assets = workspace / "assets"
    assets.mkdir(parents=True, exist_ok=True)
    (assets / "sprites").mkdir(exist_ok=True)

    root, source_assets = bundled_sources()
    if not (source_assets / "restool.py").is_file():
        raise FileNotFoundError("Bundled Z3R asset tools are missing")
    for item in source_assets.iterdir():
        if item.is_file() and item.suffix in (".py", ".bin"):
            shutil.copyfile(item, assets / item.name)

    sys.path.insert(0, str(assets))
    font = root / "other" / "3x5_font.png"
    if font.is_file():
        other = workspace / "other"
        other.mkdir(exist_ok=True)
        shutil.copyfile(font, other / font.name)

    if args.check_language:
        if "--rom" not in forwarded or forwarded.index("--rom") + 1 >= len(forwarded):
            parser.error("--check-language requires --rom")
        import util
        path = forwarded[forwarded.index("--rom") + 1]
        rom = util.load_rom(path, True)
        if rom.language != args.check_language:
            raise ValueError(
                f"ROM language mismatch: expected {args.check_language}, got {rom.language}"
            )

    sys.argv = [str(assets / "restool.py"), *forwarded]
    runpy.run_path(str(assets / "restool.py"), run_name="__main__")


if __name__ == "__main__":
    main()
