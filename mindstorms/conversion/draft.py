"""Makes a first Markdown draft of Mindstorms from pdftotext's per-page text.

Removes running headers and page numbers, joins lines into paragraphs (a
paragraph starts with an indented line), rejoins words hyphenated across
lines, and marks where each book page starts. Headings, figures, code and
OCR slips are left for review against the page images.
"""
import re, sys, collections
from pathlib import Path

HERE = Path(__file__).parent
PAGES = HERE / "pages"
FIRST, LAST = 7, 239  # Foreword to the end of the index

# PDF page -> book page: the Foreword is vi-ix, then the book's numbers are
# 9 behind the PDF's.
def book_page(p):
    if p <= 10:
        return ["vi", "vii", "viii", "ix"][p - 7]
    return str(p - 9)

# Where each part starts, and its heading.
STARTS = {
    7: "Foreword: The Gears of My Childhood",
    12: "Introduction: Computers for Children",
    28: "Chapter 1: Computers and Computer Cultures",
    47: "Chapter 2: Mathophobia: The Fear of Learning",
    64: "Chapter 3: Turtle Geometry: A Mathematics Made for Learning",
    104: "Chapter 4: Languages for Computers and for People",
    129: "Chapter 5: Microworlds: Incubators for Knowledge",
    144: "Chapter 6: Powerful Ideas in Mind-Size Bites",
    165: "Chapter 7: LOGO's Roots: Piaget and AI",
    186: "Chapter 8: Images of the Learning Society",
    199: "Epilogue: The Mathematical Unconscious",
    217: "Afterword and Acknowledgments",
    226: "Notes",
    234: "Index",
}

HEADER = re.compile(r"^\s*(M\W?[I/l1]NDSTORMS|Foreword|Introduction|Notes|Index|"
                    r"Computers and Computer Cultures|Mathophobia.*|Turtle Geometry.*|"
                    r"Languages ?for Computers.*|Microworlds.*|Powerful Ideas.*|LOGO's Roots.*|"
                    r"Images of the Learning.*|The ?M ?a ?t ?h.*|Afterword and.*)\s*$")
NUMBER = re.compile(r"^\s*([0-9]{1,3}|[ivx]{1,5})\s*$")

raw = {p: (PAGES / f"{p:03d}.txt").read_text().replace("\f", "").split("\n") for p in range(FIRST, LAST + 1)}

def body(p):
    lines = [l.rstrip() for l in raw[p]]
    while lines and not lines[0].strip():
        lines.pop(0)
    while lines and not lines[-1].strip():
        lines.pop()
    if p not in STARTS and lines and HEADER.match(lines[0]):
        lines.pop(0)
    if lines and NUMBER.match(lines[-1]):
        lines.pop()
    while lines and not lines[0].strip():
        lines.pop(0)
    while lines and not lines[-1].strip():
        lines.pop()
    return lines

# Words written whole, or with a hyphen, somewhere other than a line end.
whole, hyphened = collections.Counter(), collections.Counter()
for p in raw:
    for line in body(p):
        words = line.split()
        for w in words[:-1] if line.rstrip().endswith("-") else words:
            w = re.sub(r"^\W+|\W+$", "", w).lower()
            (hyphened if "-" in w else whole)[w] += 1

# Compounds broken at their own hyphen, which appear only once, so the
# counts above can't tell them from a broken word. Found by listing joins
# that aren't dictionary words but whose halves are, each checked against
# the page image.
COMPOUNDS = {"object-to", "head-on", "self-sabotage", "two-wheeler", "step-by", "mind-sized", "eye-following",
             "too-quantitative", "and-butter", "six-year", "half-humored", "pleasure-giving", "a-boo", "out-of", "of-motion"}
changed = []

def join(a, b):
    """Joins line `a`, ending in a hyphen, to line `b`."""
    first = re.sub(r"^.*?(\w+)-$", r"\1", a)
    rest = re.match(r"^(\w+)", b)
    if rest:
        joined = (first + rest.group(1)).lower()
        dashed = (first + "-" + rest.group(1)).lower()
        if hyphened[dashed] > whole[joined]:
            return a + b
        if dashed in COMPOUNDS:
            changed.append(dashed)
            return a + b
    return a[:-1] + b

# A footnote, marked * (or • by the OCR), runs from its line to the end of
# the page.
FOOTNOTE = re.compile(r"^\s{1,6}[*•] ?[A-Z]")

def indent(line):
    return len(line) - len(line.lstrip()) if line.strip() else -1

def quotes(lines):
    """For each line, whether it's part of an indented quotation, and whether
    it starts a paragraph there. A quotation is a run of three or more lines
    all indented at least two spaces, as body text isn't; within it, a line
    indented further than the rest starts a paragraph."""
    out = [(False, False)] * len(lines)
    n = 0
    while n < len(lines):
        if indent(lines[n]) >= 2:
            end = n
            while end < len(lines) and indent(lines[end]) >= 2:
                end += 1
            if end - n >= 3:
                least = min(indent(l) for l in lines[n:end])
                for k in range(n, end):
                    out[k] = (True, k == n or indent(lines[k]) > least + 1)
            n = end
        else:
            n += 1
    return out

out = []
para = ""
notes = []  # footnotes waiting for their paragraph to end

def flush():
    global para
    if para:
        out.append(para + "\n\n")
        para = ""
    for note in notes:
        out.append("> " + note + "\n\n")
    notes.clear()

for p in range(FIRST, LAST + 1):
    lines = body(p)
    marker = f"<!-- p. {book_page(p)} -->"
    incoming = []  # this page's footnotes, which wait for its text to start
    for n, line in enumerate(lines):
        if n > len(lines) / 2 and FOOTNOTE.match(line):
            note = ""
            for l in lines[n:]:
                l = l.strip()
                note = note[:-1] + l if note.endswith("-") else (note + " " + l).strip()
            incoming.append(re.sub(r"^[*•] ?", "\\* ", note))
            lines = lines[:n]
            break
    if p in STARTS:
        flush()
        out.append(f"\n\n## {STARTS[p]}\n\n{marker}\n\n")
        # The printed title's lines come before the first full line.
        while lines and len(lines[0].strip()) < 40:
            lines.pop(0)
        marker = ""
    quoting = False
    marks = quotes(lines)
    for n, line in enumerate(lines):
        if not line.strip():
            flush()
            continue
        quote, starts = marks[n]
        if quote != quoting or (quote and starts):
            flush()
            quoting = quote
            if quote:
                para = ">"
        indented = line.startswith("  ") and not line.startswith("          ") and not quote
        text = line.strip()
        if indented and para:
            flush()
        if "*" in text:
            # The line with a footnote's mark: the note follows its paragraph.
            notes.extend(incoming)
            incoming = []
        rejoined = para.endswith("-") and not para.endswith("--")
        before = para
        if rejoined:
            para = join(para, text)
        else:
            para = (para + " " + text) if para else text
        if marker:
            # The page starts with this line: after the word carried over
            # from the last page, if one was rejoined, or else before it.
            if rejoined:
                _, _, tail = text.partition(" ")
                cut = len(para) - len(tail)
                para = para[:cut].rstrip() + " " + marker + " " + para[cut:] if tail else para + " " + marker
            else:
                cut = len(before)
                para = (para[:cut] + " " + marker + " " + para[cut:].lstrip()).lstrip()
            marker = ""
    if quoting:
        flush()
    notes.extend(incoming)
flush()

text = "".join(out)
text = text.replace("---", "—")
text = re.sub(r"[ \t]+\n", "\n", text)
text = re.sub(r"\n{3,}", "\n\n", text)
(HERE / "draft.md").write_text("# Mindstorms: Children, Computers, and Powerful Ideas\n\nSeymour Papert, 1980\n" + text.strip() + "\n")
print(len(text.split()), "words")
(HERE / "compounds.txt").write_text("\n".join(changed) + "\n")
