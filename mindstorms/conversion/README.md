# Converting Mindstorms to Markdown

`../mindstorms.md` was made from the scanned PDF's own OCR text layer, then
checked page by page against the page images.

1. `pdftotext -layout`, one page at a time, into `pages/NNN.txt`.
2. `draft.py` makes `draft.md`: drops running heads and page numbers, joins
   lines into paragraphs, rejoins words broken across lines (keeping the
   hyphen of compounds), moves footnotes after their paragraph, and marks
   each printed page with `<!-- p. N -->`.
3. `index.py` extracts the two-column index one column at a time.
4. `apply.py` applies `fixes/*.py` to the draft, giving `mindstorms.md`.
   Each fix is an exact `(old, new)` pair that must match exactly once (or a
   `re:` regex, or `all:N:` for N occurrences), so nothing changes by
   accident. Every fix was made from the page image.
5. `verify.py A` compares the OCR text with the draft and `verify.py B` the
   draft with the result, word by word. Every difference was reviewed: A
   shows only rejoined words and dropped heads; B only the fixes.

Figures were cropped from 150 dpi renders (`pdftoppm -r 150`), using
`bands.sh` to find their rows. The scripts expect `pages/` and the renders
beside them, as in the working directory they ran in.
