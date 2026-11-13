# SPDX-FileCopyrightText: 2026 legluondunet
# SPDX-License-Identifier: GPL-3.0-or-later
"""Standalone Z3R resource extractor built from pinned upstream scripts.
No ROM, extracted Nintendo resources or saves are bundled.
"""
import argparse
from pathlib import Path
import runpy
import shutil
import sys

def main():
    parser = argparse.ArgumentParser(description="Extract Z3R resources from a user-owned ROM")
    parser.add_argument("--workspace", required=True)
    parser.add_argument("--check-language")
    args, rest = parser.parse_known_args()
    workspace = Path(args.workspace).resolve()
    workspace.mkdir(parents=True, exist_ok=True)
    assets = workspace / "assets"
    assets.mkdir(parents=True, exist_ok=True)
    (assets / "sprites").mkdir(exist_ok=True)

    base = Path(getattr(sys, "_MEIPASS", Path(__file__).resolve().parent.parent))
    source = base / "assets"
    if not (source / "restool.py").is_file():
        parser.error("Bundled Z3R resource tools are missing")
    for path in source.iterdir():
        if path.is_file() and path.suffix in (".py", ".bin"):
            shutil.copyfile(path, assets / path.name)
    other = base / "other" / "3x5_font.png"
    if other.is_file():
        target = workspace / "other"
        target.mkdir(exist_ok=True)
        shutil.copyfile(other, target / other.name)

    sys.path.insert(0, str(assets))
    if args.check_language:
        try:
            rom = rest[rest.index("--rom") + 1]
        except (ValueError, IndexError):
            parser.error("--check-language requires --rom FILE")
        import util
        loaded = util.load_rom(rom, True)
        if loaded.language != args.check_language:
            parser.error(
                f"ROM language mismatch: expected {args.check_language}, "
                f"got {loaded.language}"
            )
    sys.argv = [str(assets / "restool.py"), *rest]
    runpy.run_path(str(assets / "restool.py"), run_name="__main__")

if __name__ == "__main__":
    main()
