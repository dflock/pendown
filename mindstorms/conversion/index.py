"""Turns the index's columns, extracted one at a time, into a Markdown list.

An entry starts at the margin, a subentry two spaces in, and a line indented
further continues the line before it.
"""
from pathlib import Path

HERE = Path(__file__).parent
entries = []  # (level, text)
columns = (HERE / "index-raw.txt").read_text().split("@@ col")
lines = []
for column in columns:
    # Page numbers and running heads aren't entries; and a column's own
    # margin is its indent zero.
    rows = [l.rstrip() for l in column.split("\n") if l.strip() and l.strip() != "Index"]
    if rows and rows[-1].strip().isdigit() and int(rows[-1]) >= 225:
        rows.pop()
    margin = min((len(l) - len(l.lstrip()) for l in rows), default=0)
    lines += [l[margin:] for l in rows]
for line in lines:
    # An entry continued from the last page is the same entry.
    if line.strip().endswith("(continued)"):
        continue
    indent = len(line) - len(line.lstrip())
    text = line.strip()
    if indent >= 4 and entries:
        level, last = entries[-1]
        joined = last[:-1] + text if last.endswith("-") and last[-2:-1].isalpha() else (last + text if last.endswith("-") else last + " " + text)
        entries[-1] = (level, joined)
    else:
        entries.append((1 if indent >= 2 else 0, text))

out = []
for level, text in entries:
    out.append(("  " if level else "") + "- " + text)
(HERE / "index.md").write_text("\n".join(out) + "\n")
print(len(entries), "entries")

# Slips in the OCR, checked against PDF pages 234-239, and the book's
# italics: see, see also, roman page numbers and groupements.
import re
FIXES = [
    ("182-! 83", "182-183"), ("Ren6", "René"), ("local nature Of,", "local nature of,"),
    ("L~vi-Strauss", "Lévi-Strauss"), ("179-I 83", "179-183"), ("Pe~sonal", "Personal"),
    ("Poincar6, Raymond, 190-197, 201,203", "Poincaré, Raymond, 190-197, 201, 203"),
    ("(Poincar6)", "(Poincaré)"), ("130--131", "130-131"), ("52, 221, nl (Ch. 3)", "52, 221, n1 (Ch. 3)"),
    ("samba, s e e samba school(s)", "samba, *see* samba school(s)"),
    ("see a l s o Piagetian", "*see also* Piagetian"),
    ("- Groupements, Piaget's", "- *Groupements,* Piaget's"), ("  - groupements theory", "  - *groupements* theory"),
]
text = (HERE / "index.md").read_text()
for old, new in FIXES:
    assert text.count(old) == 1, old
    text = text.replace(old, new)
text = text.replace("221 n 1", "221n 1")
text = re.sub(r"; see also\b", "; *see also*", text)
text = re.sub(r", see\b(?! also)", ", *see*", text)
text = re.sub(r"\b(vii-viii|viii|vii)\b", r"*\1*", text)
(HERE / "index.md").write_text(text)
