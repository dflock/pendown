"""Compares word sequences, ignoring case, punctuation and markup, and lists
every place where they differ: A, the OCR text against the draft; B, the
draft against the final Markdown."""
import re, sys, difflib
from pathlib import Path

HERE = Path(__file__).parent

def words(text, markdown):
    if markdown:
        text = re.sub(r"<!--.*?-->", " ", text, flags=re.S)
        text = re.sub(r"!\[[^\]]*\]\([^)]*\)", " ", text)
        text = re.sub(r"</?[a-z]+[^<>]*>", "", text)
    return re.findall(r"[a-z0-9]+", text.lower())

# Words of the running heads, which the draft drops with the page numbers.
HEADS = set(words("MINDSTORMS Foreword Introduction Notes Index Computers and Computer Cultures "
                  "Mathophobia The Fear of Learning Turtle Geometry A Mathematics Made for Learning "
                  "Languages for Computers and for People Microworlds Incubators for Knowledge "
                  "Powerful Ideas in Mind Size Bites LOGO's Roots Piaget and AI Images of the Learning Society "
                  "The Mathematical Unconscious Afterword and Acknowledgments vi vii viii ix", False))

def expected(a, b):
    """A difference the draft is meant to make: words rejoined across a line
    end, or running heads and page numbers dropped from between them."""
    kept = [w for w in a if not (w in HEADS or (w.isdigit() and int(w) <= 239))]
    return "".join(kept) == "".join(b)

def diff(a, b, name, limit, filtered=False):
    sm = difflib.SequenceMatcher(None, a, b, autojunk=False)
    ops = [op for op in sm.get_opcodes() if op[0] != "equal"]
    if filtered:
        ops = [op for op in ops if not expected(a[op[1]:op[2]], b[op[3]:op[4]])]
    changed = sum(max(i2 - i1, j2 - j1) for _, i1, i2, j1, j2 in ops)
    print(f"{name}: {len(a)} words against {len(b)}, {len(ops)} differences, {changed} words")
    for tag, i1, i2, j1, j2 in ops[:limit]:
        print(f"  {tag:7} {' '.join(a[i1:i2])[:90]!r} -> {' '.join(b[j1:j2])[:90]!r}")
    return ops

which = sys.argv[1]
if which == "A":
    raw = "\n".join((HERE / "pages" / f"{p:03d}.txt").read_text() for p in range(7, 240))
    diff(words(raw, False), words((HERE / "draft.md").read_text(), True), "A", int(sys.argv[2]), filtered=True)
else:
    diff(words((HERE / "draft.md").read_text(), True), words((HERE / "mindstorms.md").read_text(), True), "B", int(sys.argv[2]))
