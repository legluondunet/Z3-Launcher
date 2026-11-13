# SPDX-FileCopyrightText: 2026 legluondunet
# SPDX-License-Identifier: GPL-3.0-or-later
"""Self-contained Z3R asset builder.

The PyInstaller bundle includes Z3R's MIT-licensed Python asset tools,
Pillow, and PyYAML. It never includes a ROM or generated game resources.
"""
import argparse
import os
from pathlib import Path
import runpy
import shutil
import sys

parser = argparse.ArgumentParser()
parser.add_argument("--workspace", required=True)
parser.add_argument("--check-language")
args, forwarded = parser.parse_known_args()
workspace = Path(args.workspace).resolve()
assets = workspace / "assets"
assets.mkdir(parents=True, exist_ok=True)
(assets / "sprites").mkdir(exist_ok=True)

# PyInstaller places --add-data assets under _MEIPASS.
source_root = Path(getattr(sys, "_MEIPASS", Path(__file__).resolve().parent.parent))
source_assets = source_root / "assets"
if not (source_assets / "restool.py").is_file():
    raise FileNotFoundError("Bundled Z3R asset tools are missing")
for item in source_assets.iterdir():
    if item.is_file() and item.suffix in (".py", ".bin"):
        shutil.copyfile(item, assets / item.name)

sys.path.insert(0, str(assets))
font = source_root / "other" / "3x5_font.png"
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
