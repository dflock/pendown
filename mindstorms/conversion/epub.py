"""Builds ../mindstorms.epub from ../mindstorms.md, with pandoc.

Each `##` section (Foreword, Introduction, the chapters, Epilogue, Afterword,
Notes and Index) becomes a chapter of its own, listed in the table of
contents. The book's details go into the EPUB's metadata rather than the
text, the scan's title page is the cover, and the page markers, which mean
nothing in an e-book, are left out.
"""
import re
import subprocess
from pathlib import Path

HERE = Path(__file__).parent
BOOK = HERE.parent
BUILD = HERE / "build"
BUILD.mkdir(exist_ok=True)

text = (BOOK / "mindstorms.md").read_text()

# The title and front matter come from the metadata; the text starts at the
# Foreword, after a page for the frontispiece.
text = text[text.index("## Foreword"):]
text = re.sub(r"<!--.*?-->\n?", "", text, flags=re.S)
text = re.sub(r"\n{3,}", "\n\n", text)
frontispiece = (
    "## Frontispiece {.unlisted .unnumbered}\n\n"
    "![Frontispiece: LOGO Turtle. Two children with a floor Turtle, its button box, "
    "and the shapes it has drawn on paper.](images/frontispiece.png)\n\n"
    "*Frontispiece: LOGO Turtle.*\n\n"
)
text = frontispiece + text
# The Afterword's signature, set to the right.
text = re.sub(r'<p align="right">(.*?)</p>',
              lambda m: "::: signature\n" + re.sub(r"<i>(.*?)</i>", r"*\1*", m.group(1)).replace("<br>", "\\\n") + "\n:::",
              text)
(BUILD / "mindstorms.md").write_text(text)

(BUILD / "metadata.yaml").write_text("""\
title: "Mindstorms"
subtitle: "Children, Computers, and Powerful Ideas"
author: Seymour Papert
publisher: "Basic Books, Inc."
date: "1980"
lang: en-US
rights: "Copyright © 1980 by Basic Books, Inc."
identifier:
  - scheme: ISBN-10
    text: "0-465-04627-4"
description: >
  First edition, converted from a scan. Designed by Vincent Torre.
""")

(BUILD / "epub.css").write_text("""\
body { font-family: serif; line-height: 1.4; }
h1 { page-break-before: always; text-align: left; margin-top: 2em; }
h2 { margin-top: 1.5em; }
p { margin: 0.4em 0; text-align: justify; }
img { max-width: 100%; height: auto; display: block; margin: 1em auto; }
blockquote { margin: 1em 1.5em; font-size: 0.95em; }
pre { font-size: 0.85em; white-space: pre-wrap; margin: 1em 0; }
code { font-family: monospace; }
sup { font-size: 0.7em; line-height: 0; }
.signature { text-align: right; margin-top: 1.5em; }
ol, ul { margin: 0.5em 0 0.5em 1.5em; }
""")

subprocess.run(
    [
        "pandoc", str(BUILD / "mindstorms.md"),
        "--from", "markdown+pipe_tables-implicit_figures",
        "--to", "epub3",
        "--output", str(BOOK / "mindstorms.epub"),
        "--metadata-file", str(BUILD / "metadata.yaml"),
        "--css", str(BUILD / "epub.css"),
        "--epub-cover-image", str(BOOK / "images" / "cover.png"),
        # The book's sections are ## headings: as chapters they're level 1.
        "--shift-heading-level-by=-1",
        "--toc", "--toc-depth=1",
        "--split-level=1",
        "--resource-path", str(BOOK),
    ],
    check=True,
)
print("wrote", BOOK / "mindstorms.epub")
