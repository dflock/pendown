#!/bin/sh
# Makes Pendown's icons from crates/pendown/images/pendown-icon.svg: the
# Windows pendown.ico, at the sizes Windows shows, and the PNG the window
# shows on Linux. Needs Inkscape and ImageMagick. Run it again after
# changing the SVG, and commit what it makes.
set -eu
crate="$(dirname "$0")/../crates/pendown"
svg="$crate/images/pendown-icon.svg"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
png() {
    inkscape "$svg" --export-type=png --export-width="$1" --export-height="$1" \
        --export-filename="$2" 2>/dev/null
}
for size in 16 20 24 32 40 48 64 256; do
    png "$size" "$tmp/$size.png"
done
convert "$tmp/16.png" "$tmp/20.png" "$tmp/24.png" "$tmp/32.png" "$tmp/40.png" \
    "$tmp/48.png" "$tmp/64.png" "$tmp/256.png" "$crate/windows/pendown.ico"
png 128 "$crate/linux/pendown.png"
