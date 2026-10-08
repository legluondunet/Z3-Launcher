#!/usr/bin/env bash
set -u
missing=0
for tool in cargo rustc git python3 make cc sdl2-config sha256sum pkg-config; do
  if command -v "$tool" >/dev/null 2>&1; then printf 'OK : %s\n' "$tool"; else printf 'MISSING : %s\n' "$tool"; missing=1; fi
done
if python3 -c 'import PIL, yaml' 2>/dev/null; then echo 'OK : Pillow and PyYAML'; else echo 'MISSING : Pillow / PyYAML'; missing=1; fi
exit "$missing"
