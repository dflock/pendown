# Index, pp. 225-230: the OCR read the two columns across, so the index is
# extracted again one column at a time, by index.py, and checked against PDF
# pages 234-239. Page numbers are the printed ones, as in the page markers.
from pathlib import Path

INDEX = (Path(__file__).parent.parent / "index.md").read_text()

FIXES = [
    (r"re:## Index\n\n<!-- p\. 225 -->.*\Z",
     "## Index\n\n<!-- p. 225 -->\n\n<!-- Extracted one column at a time; the page markers of pp. 226-230 are left out of the list. -->\n\n" + INDEX),
]
