#!/usr/bin/env python3
"""Validate JSON translations and named placeholders using only Python's standard library."""
import argparse
from collections import Counter
import json
from pathlib import Path
import re
import sys


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"Duplicate JSON key: {key}")
        result[key] = value
    return result


def load(path):
    with path.open(encoding="utf-8") as stream:
        data = json.load(stream, object_pairs_hook=unique_object)
    if not isinstance(data, dict) or not isinstance(data.get("language"), dict) or not isinstance(data.get("messages"), dict):
        raise ValueError("Expected language and messages objects")
    language = data["language"]
    code = language.get("code")
    if not isinstance(code, str) or not re.fullmatch(r"[A-Za-z0-9_-]+", code):
        raise ValueError("Invalid language code")
    if not isinstance(language.get("name"), str) or not language["name"].strip():
        raise ValueError("Missing native language name")
    if code != path.stem:
        raise ValueError("Language code must match the JSON filename")
    if any(not isinstance(k, str) or not isinstance(v, str) or not v for k, v in data["messages"].items()):
        raise ValueError("Messages must have nonempty string values")
    return data


def placeholders(text):
    return Counter(re.findall(r"\{([A-Za-z0-9_]+)\}", text))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", nargs="?", type=Path, default=Path(__file__).resolve().parents[1] / "locales")
    parser.add_argument("--allow-missing", action="store_true", help="Allow incomplete translations (English fallback)")
    args = parser.parse_args()
    try:
        baseline = load(args.directory / "en.json")["messages"]
    except (OSError, ValueError) as error:
        print(f"ERROR: English reference: {error}")
        return 1
    failed = False
    codes = set()
    for path in sorted(args.directory.glob("*.json")):
        try:
            data = load(path)
            code = data["language"]["code"]
            if code in codes:
                raise ValueError(f"Duplicate language code: {code}")
            codes.add(code)
            messages = data["messages"]
            missing = sorted(baseline.keys() - messages.keys())
            unknown = sorted(messages.keys() - baseline.keys())
            invalid = sorted(k for k in baseline.keys() & messages.keys() if placeholders(baseline[k]) != placeholders(messages[k]))
            for label, keys in [("MISSING", missing), ("UNKNOWN", unknown), ("PLACEHOLDERS", invalid)]:
                if keys:
                    print(f"{path.name}: {label}: {', '.join(keys)}")
            if unknown or invalid or (missing and not args.allow_missing):
                failed = True
            else:
                print(f"{path.name}: OK ({len(messages)} messages)")
        except (OSError, ValueError) as error:
            print(f"{path.name}: ERROR: {error}")
            failed = True
    return int(failed)


if __name__ == "__main__":
    sys.exit(main())
