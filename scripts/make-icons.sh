#!/bin/sh
# Makes Pendown's icons: the Windows pendown.ico, at the sizes Windows shows,
# and the PNG the window shows on Linux. Sizes from 32 up are Kame's picture,
# content/tutorial/kame-turtle.svg, centred in a transparent square because
# it's wider than it is tall. Smaller sizes are the simpler drawing in
# crates/pendown/images/pendown-icon.svg, which stays clear when that small.
# Needs Inkscape and ImageMagick. Run it again after changing either SVG, and
# commit what it makes.
set -eu
root="$(dirname "$0")/.."
crate="$root/crates/pendown"
small="$crate/images/pendown-icon.svg"
large="$root/content/tutorial/kame-turtle.svg"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
small_png() {
    inkscape "$small" --export-type=png --export-width="$1" --export-height="$1" \
        --export-filename="$2" 2>/dev/null
}
large_png() {
    inkscape "$large" --export-type=png --export-width="$1" \
        --export-filename="$tmp/wide.png" 2>/dev/null
    convert "$tmp/wide.png" -background none -gravity center \
        -extent "$1x$1" "$2"
}
for size in 16 20 24; do
    small_png "$size" "$tmp/$size.png"
done
for size in 32 40 48 64 256; do
    large_png "$size" "$tmp/$size.png"
done
convert "$tmp/16.png" "$tmp/20.png" "$tmp/24.png" "$tmp/32.png" "$tmp/40.png" \
    "$tmp/48.png" "$tmp/64.png" "$tmp/256.png" "$crate/windows/pendown.ico"
large_png 128 "$crate/linux/pendown.png"
