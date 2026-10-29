"""Applies the reviewed fixes, in fixes/*.py, to draft.md, giving mindstorms.md.

Each fix is (old, new): `old` must occur exactly once in the text as it is
by then, so a fix can't land in the wrong place. Each file defines FIXES.
"""
import sys, runpy
from pathlib import Path

HERE = Path(__file__).parent
text = (HERE / "draft.md").read_text()

# Rules for the OCR's systematic slips, before the fixes for single places.
import re
# Poincaré's accented e, read as 6 or ~.
text = re.sub(r"Poincar[6~]", "Poincaré", text)
# An em dash read as a tilde, between two words.
text = re.sub(r"(?<=\w)~(?=\w)", "—", text)
# Letter-spaced capitals, as the book prints Logo's words: R I G H T.
text = re.sub(r"\b(?:[A-Z] ){2,}[A-Z]\b", lambda m: m.group(0).replace(" ", ""), text)
# A footnote's mark, after punctuation, so Markdown doesn't read it as
# emphasis.
text = re.sub(r'(?<=[.,;:"\'])\*(?=\s|$)', r"\\*", text)

failed = 0
for path in sorted((HERE / "fixes").glob("*.py")):
    for old, new in runpy.run_path(str(path))["FIXES"]:
        # `re:` starts a regex, for text too garbled to quote, such as what
        # the OCR made of a figure. It too must match exactly once.
        if old.startswith("re:"):
            matches = list(re.finditer(old[3:], text, re.S))
            if len(matches) != 1:
                print(f"{path.name}: {len(matches)} matches for {old[:70]!r}")
                failed += 1
                continue
            m = matches[0]
            text = text[:m.start()] + new + text[m.end():]
            continue
        # `all:N:` replaces every occurrence, of which there must be N.
        if old.startswith("all:"):
            _, want, old = old.split(":", 2)
            n = text.count(old)
            if n != int(want):
                print(f"{path.name}: {n} matches, not {want}, for {old[:70]!r}")
                failed += 1
                continue
            text = text.replace(old, new)
            continue
        n = text.count(old)
        if n != 1:
            print(f"{path.name}: {n} matches for {old[:70]!r}")
            failed += 1
            continue
        text = text.replace(old, new)
(HERE / "mindstorms.md").write_text(text)
print("failed" if failed else "ok", failed)
sys.exit(1 if failed else 0)
