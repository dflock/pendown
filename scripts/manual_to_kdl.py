"""Convert the Berkeley Logo Reference Manual (texinfo HTML) into a KDL v2 file.

Usage (from the repo root):

    python3 -I scripts/manual_to_kdl.py refs/logo/Berkeley-Logo-Reference-Manual.html berkeley-logo.kdl

Structure: logo > chapter > [section >] entry
"""

import re
import sys
from html.parser import HTMLParser

SKIP_CHAPTERS = {"introduction", "index"}  # no commands in these


def kebab(title):
    title = re.sub(r"^[\d.]+\s+", "", title).strip().lower()
    return re.sub(r"[^a-z0-9.]+", "-", title).strip("-")


# ---------------------------------------------------------------- tokenising
# Turn the HTML into a flat list of events we care about.


class Flatten(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.events = []  # (kind, payload)
        self.stack = []
        self.text = []
        self.skip = 0  # inside header div / menu table / style etc.
        self.pre = False
        self.heading = None
        self.link = None

    def flush(self, kind="text"):
        if self.text:
            self.events.append((kind, "".join(self.text)))
            self.text = []

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        cls = a.get("class", "")
        if self.skip:
            if tag in ("div", "table", "style", "head", "script"):
                self.skip += 1
            return
        if (tag == "div" and cls == "header") or (tag == "table" and cls in ("menu", "index-entries")) or tag in ("style", "head", "script"):
            self.flush()
            self.skip = 1
            return
        if tag in ("h1", "h2", "h3", "h4"):
            self.flush()
            self.heading = (tag, cls)
            return
        if tag == "hr":
            self.flush()
            self.events.append(("hr", None))
        elif tag == "pre":
            self.flush()
            self.pre = True
        elif tag in ("p", "li", "tr", "br"):
            self.flush()
            self.events.append(("break", tag))
        elif tag == "a" and "href" in a and a["href"].startswith("#"):
            self.link = []
            self.href = a["href"][1:]
        elif tag == "a" and "name" in a:
            self.flush()
            self.events.append(("anchor", a["name"]))

    def handle_endtag(self, tag):
        if self.skip:
            if tag in ("div", "table", "style", "head", "script"):
                self.skip -= 1
            return
        if tag in ("h1", "h2", "h3", "h4") and self.heading:
            lvl, cls = self.heading
            self.events.append(("heading", (lvl, cls, "".join(self.text).strip())))
            self.text = []
            self.heading = None
        elif tag == "pre":
            self.events.append(("pre", "".join(self.text)))
            self.text = []
            self.pre = False
        elif tag == "a" and self.link is not None:
            self.events.append(("link", ("".join(self.link), self.href)))
            self.link = None
        elif tag in ("p", "li", "ul", "table"):
            self.flush()
            self.events.append(("break", "/" + tag))

    def handle_data(self, data):
        if self.skip:
            return
        if self.link is not None:
            self.link.append(data)
        self.text.append(data)


# ---------------------------------------------------------------- structuring


def parse(path):
    f = Flatten()
    f.feed(open(path, encoding="utf-8").read())
    f.flush()
    ev = f.events

    chapters = []  # [{name, sections:[{name, entries}], entries:[...]}]
    chapter = section = entry = None
    node_anchors = []  # anchors seen since the last <hr>
    anchor_to_name = {}

    for i, (kind, payload) in enumerate(ev):
        if kind == "hr":
            node_anchors = []
        elif kind == "anchor" and entry is None:
            node_anchors.append(payload)
        if kind == "heading":
            lvl, cls, title = payload
            # Entries are subsections, except that the manual marks a few up as
            # sections: AND, .MACRO and ALLOWGETSET (unnumbered) and 3.6 USUAL.
            is_entry = lvl == "h4" or (lvl == "h3" and (cls == "unnumberedsec" or kebab(title) == "usual"))
            if lvl == "h2":
                chapter = {"name": kebab(title), "children": []}
                chapters.append(chapter)
                section = entry = None
            elif lvl == "h3" and not is_entry:
                section = {"name": kebab(title), "entries": []}
                chapter["children"].append(("section", section))
                entry = None
            elif is_entry:
                name = re.sub(r"^[\d.]+\s+", "", title)
                name = {"\u2018": "`"}.get(name, name)  # texinfo turned the backquote into a curly quote
                entry = {"name": name, "blocks": [], "see": []}
                for a in node_anchors:
                    anchor_to_name[a] = name
                if lvl == "h4" and section:
                    section["entries"].append(entry)
                else:
                    chapter["children"].append(("entry", entry))
            continue
        if entry is None:
            continue
        if kind == "hr":
            entry = None
        elif kind == "pre":
            entry["blocks"].append(("pre", payload))
        elif kind == "break":
            entry["blocks"].append(("break", payload))
        elif kind == "text":
            entry["blocks"].append(("text", payload))
        elif kind == "link":
            entry["blocks"].append(("link", payload))

    global ANCHORS
    ANCHORS = anchor_to_name
    return [c for c in chapters if c["name"] not in SKIP_CHAPTERS]


def render_entry(entry):
    """Split an entry's blocks into (eg, desc, see)."""
    blocks = entry["blocks"]
    eg = None
    if blocks and any(k == "pre" for k, _ in blocks):
        # The usage block is the first <pre>, if nothing but whitespace precedes it.
        first = next(i for i, (k, _) in enumerate(blocks) if k == "pre")
        if all(k != "text" or not v.strip() for k, v in blocks[:first]):
            # Tab runs only align annotations like "(library procedure)".
            eg = "\n".join(re.sub(r"(\s*\t\s*|\s{3,})(?=\([a-z ]+\)$)", "  ", re.sub(r"\s*\t\s*", "  ", l).rstrip()) for l in blocks[first][1].strip("\n").split("\n"))
            blocks = blocks[first + 1 :]

    paras = []  # list of str (prose) or ("pre", str)
    cur = []
    links = []
    in_li = False

    def end_para():
        nonlocal cur, links
        text = re.sub(r"\s+", " ", "".join(cur)).strip()
        # "See FOO , BAR ." paragraphs become structured references.
        if text.startswith("See ") and links:
            rest = text
            for label, _ in links:
                rest = rest.replace(label.strip(), "", 1)
            if not re.sub(r"See|[\s,.;]", "", rest):
                entry["see"].extend(ANCHORS.get(href, label.strip().lower()) for label, href in links)
                text = ""
        if text:
            paras.append(("- " if in_li else "") + text)
        cur, links = [], []

    for kind, v in blocks:
        if kind == "text":
            cur.append(v)
        elif kind == "link":
            links.append(v)
        elif kind == "break":
            end_para()
            if v == "li":
                in_li = True
            elif v in ("/li", "/ul"):
                in_li = False
        elif kind == "pre":
            end_para()
            paras.append(("pre", "\n".join(l.expandtabs(8).rstrip() for l in v.strip("\n").split("\n"))))
    end_para()

    out = []
    for p in paras:
        if isinstance(p, tuple):
            out.append("\n".join("    " + l if l.strip() else "" for l in p[1].split("\n")))
        else:
            out.append(p)
    # Keep list items together, separate everything else with a blank line.
    desc = ""
    for i, p in enumerate(out):
        if i:
            desc += "\n" if p.startswith("- ") and out[i - 1].startswith("- ") else "\n\n"
        desc += p
    return eg, desc, entry["see"]


# ---------------------------------------------------------------- KDL output

KEYWORDS = {"inf", "-inf", "nan", "true", "false", "null"}


def ident(name):
    """A bare KDL v2 identifier if possible, otherwise a quoted string."""
    bare = (
        name
        and name not in KEYWORDS
        and not re.search(r'[(){}\[\]/\\"#;=\s]', name)
        and not re.match(r"[0-9]|[+-]?\.?[0-9]", name)
    )
    return name if bare else quote(name)


def quote(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"').replace("\t", "\\t") + '"'


def multiline(s, indent):
    s = s.replace("\\", "\\\\").replace('"""', '\\"""')
    lines = [(indent + l) if l.strip() else "" for l in s.split("\n")]
    return '"""\n' + "\n".join(lines) + "\n" + indent + '"""'


def string(s, indent):
    return multiline(s, indent) if "\n" in s else quote(s)


def emit_entry(entry, depth, out):
    pad = "  " * depth
    eg, desc, see = render_entry(entry)
    out.append(f"{pad}{ident(entry['name'])} {{")
    if eg is not None:
        out.append(f"{pad}  eg {string(eg, pad + '  ')}")
    if desc:
        out.append(f"{pad}  desc {string(desc, pad + '  ')}")
    if see:
        out.append(f"{pad}  see " + " ".join(quote(s) for s in dict.fromkeys(see)))
    out.append(f"{pad}  supported #true")
    out.append(f"{pad}}}")


def main(src, dst):
    chapters = parse(src)
    out = ["// Generated from refs/logo/Berkeley-Logo-Reference-Manual.html", "logo {"]
    count = 0
    for c in chapters:
        children = [(k, v) for k, v in c["children"] if k == "entry" or v["entries"]]
        if not children:
            continue
        out.append(f"  {ident(c['name'])} {{")
        for kind, child in children:
            if kind == "entry":
                emit_entry(child, 2, out)
                count += 1
                continue
            out.append(f"    {ident(child['name'])} {{")
            for e in child["entries"]:
                emit_entry(e, 3, out)
                count += 1
            out.append("    }")
        out.append("  }")
    out.append("}")
    open(dst, "w", encoding="utf-8").write("\n".join(out) + "\n")
    print(f"{count} entries written to {dst}", file=sys.stderr)


if __name__ == "__main__":
    main(*sys.argv[1:3])
