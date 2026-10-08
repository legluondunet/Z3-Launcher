#!/usr/bin/env bash
# Build on the oldest supported glibc distribution, e.g. Ubuntu 22.04 x86_64.
set -euo pipefail
project="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || { echo 'This script supports Linux x86_64.' >&2; exit 1; }
for tool in cargo python3 curl file ldconfig; do
  command -v "$tool" >/dev/null || { echo "Missing build tool: $tool" >&2; exit 1; }
done
cd -- "$project"
python3 tools/check-translations.py
cargo build --release
binary="$project/target/release/z3-launcher"
file "$binary" | grep -q 'ELF 64-bit.*x86-64' || { echo 'Expected a Linux x86_64 binary.' >&2; exit 1; }
version="$(python3 -c 'import re; print(re.search(r"^version\s*=\s*\"([^\"]+)\"", open("Cargo.toml").read(), re.M)[1])')"
mkdir -p -- "$project/dist" "$project/tools/appimage"
linuxdeploy="${LINUXDEPLOY:-$project/tools/appimage/linuxdeploy-x86_64.AppImage}"
if [[ ! -f $linuxdeploy ]]; then
  [[ ! ${LINUXDEPLOY+x} ]] || { echo "LINUXDEPLOY does not exist: $linuxdeploy" >&2; exit 1; }
  curl --fail --location --retry 3 https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage -o "$linuxdeploy.download"
  mv -- "$linuxdeploy.download" "$linuxdeploy"
fi
chmod +x -- "$linuxdeploy"
build_dir="$(mktemp -d "$project/dist/appimage-build.XXXXXX")"
trap 'rm -rf -- "$build_dir"' EXIT
appdir="$build_dir/AppDir"
mkdir -p "$appdir/usr/bin/locales" "$appdir/usr/share/doc/z3-launcher"
cp -- "$project"/locales/*.json "$appdir/usr/bin/locales/"
cp -- "$project/LICENSE" "$project/LICENSE-MIT-original" "$project/LICENSING.md" "$project/APPIMAGE.md" "$project/assets/DejaVu-LICENSE.txt" "$appdir/usr/share/doc/z3-launcher/"
# eframe loads these libraries dynamically, so ldd alone cannot discover them.
cache="$(ldconfig -p)"
libraries=()
for library in libX11.so.6 libXcursor.so.1 libXi.so.6 libXrandr.so.2 libwayland-client.so.0 libwayland-cursor.so.0 libwayland-egl.so.1 libxkbcommon.so.0; do
  path="$(awk -v name="$library" '$1 == name && /x86-64/ { print $NF; exit }' <<< "$cache")"
  [[ -f $path ]] || { echo "Missing packaging library: $library" >&2; exit 1; }
  libraries+=(--library "$path")
done
# SDL2 compatibility implementations may load SDL3 dynamically.
path="$(awk '$1 == "libSDL3.so.0" && /x86-64/ { print $NF; exit }' <<< "$cache")"
if [[ -f $path ]]; then libraries+=(--library "$path"); fi
export ARCH=x86_64
export APPIMAGE_EXTRACT_AND_RUN=1
export LINUXDEPLOY_OUTPUT_VERSION="$version"
export LDAI_OUTPUT="$project/dist/Z3-Launcher-$version-x86_64.AppImage"
"$linuxdeploy" --appdir "$appdir" --executable "$binary" \
  --desktop-file "$project/packaging/z3-launcher.desktop" \
  --icon-file "$project/packaging/z3-launcher.svg" \
  --custom-apprun "$project/packaging/AppRun" "${libraries[@]}" --output appimage
[[ -s $LDAI_OUTPUT ]] || { echo 'AppImage output is missing.' >&2; exit 1; }
chmod +x -- "$LDAI_OUTPUT"
sha256sum -- "$LDAI_OUTPUT" > "$LDAI_OUTPUT.sha256"
printf 'AppImage: %s\n' "$LDAI_OUTPUT"
