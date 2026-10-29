#!/bin/sh
# Makes crates/pendown/windows/pendown.ico, the icon of pendown.exe, from
# pendown-icon.svg, at the sizes Windows shows. Needs Inkscape and
# ImageMagick. Run it again after changing the SVG, and commit the .ico.
set -eu
dir="$(dirname "$0")/../crates/pendown/windows"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
for size in 16 20 24 32 40 48 64 256; do
    inkscape "$dir/pendown-icon.svg" --export-type=png \
        --export-width="$size" --export-height="$size" \
        --export-filename="$tmp/$size.png" 2>/dev/null
done
convert "$tmp/16.png" "$tmp/20.png" "$tmp/24.png" "$tmp/32.png" "$tmp/40.png" \
    "$tmp/48.png" "$tmp/64.png" "$tmp/256.png" "$dir/pendown.ico"
