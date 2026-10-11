#!/usr/bin/env python3
"""Keep AppStream's current release in sync with Cargo.toml (Python 3.11+)."""
import argparse
from pathlib import Path
import tomllib
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
CARGO = ROOT / "Cargo.toml"
META = ROOT / "packaging/flatpak/io.github.legluondunet.Z3Launcher.metainfo.xml"

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="Fail if AppStream version differs from Cargo.toml")
    args = parser.parse_args()
    with CARGO.open("rb") as f:
        version = tomllib.load(f)["package"]["version"]
    tree = ET.parse(META)
    release = tree.find("./releases/release")
    if release is None:
        raise SystemExit("Missing AppStream release element")
    current = release.get("version")
    if current == version:
        print(f"AppStream version matches Cargo.toml: {version}")
        return
    if args.check:
        raise SystemExit(f"AppStream version {current!r} does not match Cargo.toml {version!r}")
    release.set("version", version)
    tree.write(META, encoding="unicode", xml_declaration=True)
    print(f"Updated AppStream version: {current} -> {version}")

if __name__ == "__main__":
    main()
