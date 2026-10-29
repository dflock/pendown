//! The language reference, from `pendown-logo.kdl` (ADR 0018): every
//! command's names, usage, our description and examples, by chapter and
//! section, and the syntax that isn't a command. The help panel shows it,
//! and the interpreter uses it to say why an obsolete command isn't here.
//!
//! The file is embedded, and parsed once, the first time it's needed.

use std::{collections::HashMap, sync::OnceLock};

use kdl::{KdlDocument, KdlNode, KdlValue};

const SOURCE: &str = include_str!("../../../pendown-logo.kdl");

/// The infix operators, which `eg` shows as `num1 + num2`.
const OPERATORS: [&str; 10] = ["+", "-", "*", "/", "=", "<", ">", "<=", ">=", "<>"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Command,
    /// A special variable, such as `CASEIGNOREDP`.
    Variable,
    /// Syntax that isn't a command, such as `:name` or `[...]`.
    Syntax,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Example {
    pub title: Option<String>,
    pub code: String,
    /// It stops with an error on purpose, as `ERRACT`'s does.
    pub fails: bool,
}

#[derive(Clone, Debug)]
pub struct Command {
    /// The node's name in the kdl, such as `forward`.
    pub key: String,
    /// What a page is titled: the first name, such as `FORWARD`, or a syntax
    /// entry's title.
    pub title: String,
    /// The names it's called by, upper case, the main one first: `FORWARD`
    /// and `FD`. Syntax entries' names are what the cursor can be on.
    pub names: Vec<String>,
    /// Infix operators that do the same, such as `+` for `SUM`.
    pub operators: Vec<String>,
    pub kind: Kind,
    /// How to call it, a line for each form: the kdl's `eg`.
    pub usage: String,
    /// The manual's description, which describes UCBLogo.
    pub desc: String,
    /// Our own description, in the help panel's markup.
    pub help: Option<String>,
    pub examples: Vec<Example>,
    /// More commands for "See also" than its section gives, by name.
    pub see: Vec<String>,
    pub phase: Option<u8>,
    /// Ours, not UCBLogo's.
    pub extension: bool,
    /// Why it isn't in Pendown, if it's obsolete: a sentence that follows
    /// "EDIT isn't in Pendown: ".
    pub obsolete: Option<String>,
    /// The interpreter has it.
    pub implemented: bool,
    pub chapter: usize,
    pub section: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct Section {
    pub key: String,
    pub title: String,
    /// Indexes into `Reference::commands`.
    pub commands: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct Chapter {
    pub key: String,
    pub title: String,
    pub sections: Vec<Section>,
    /// Commands directly in the chapter, outside any section, such as the
    /// special variables.
    pub commands: Vec<usize>,
}

impl Chapter {
    /// Every command in the chapter, in order.
    pub fn all(&self) -> impl Iterator<Item = usize> + '_ {
        self.commands.iter().copied().chain(
            self.sections
                .iter()
                .flat_map(|s| s.commands.iter().copied()),
        )
    }
}

pub struct Reference {
    /// The syntax first, then the manual's chapters.
    pub chapters: Vec<Chapter>,
    pub commands: Vec<Command>,
    /// Lower-case names and operators to commands.
    index: HashMap<String, usize>,
}

/// The reference, parsed the first time it's asked for.
pub fn reference() -> &'static Reference {
    static REFERENCE: OnceLock<Reference> = OnceLock::new();
    REFERENCE.get_or_init(|| {
        Reference::parse(SOURCE).unwrap_or_else(|e| panic!("pendown-logo.kdl doesn't parse: {e}"))
    })
}

/// `turtle-motion` as a title: "Turtle motion".
fn title(key: &str) -> String {
    let words = key.replace('-', " ");
    let mut chars = words.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

fn child<'a>(node: &'a KdlNode, name: &str) -> Option<&'a KdlValue> {
    node.children()?
        .get(name)?
        .entries()
        .iter()
        .find(|e| e.name().is_none())
        .map(|e| e.value())
}

fn string(node: &KdlNode, name: &str) -> Option<String> {
    child(node, name)
        .and_then(|v| v.as_string())
        .map(str::to_string)
}

/// Every unnamed string a child node holds, in order, from every node of
/// that name: `see "back" "left"`.
fn strings(node: &KdlNode, name: &str) -> Vec<String> {
    let Some(children) = node.children() else {
        return Vec::new();
    };
    children
        .nodes()
        .iter()
        .filter(|n| n.name().value() == name)
        .flat_map(|n| {
            n.entries()
                .iter()
                .filter(|e| e.name().is_none())
                .filter_map(|e| e.value().as_string())
        })
        .map(str::to_string)
        .collect()
}

fn examples(node: &KdlNode) -> Vec<Example> {
    let Some(children) = node.children() else {
        return Vec::new();
    };
    children
        .nodes()
        .iter()
        .filter(|n| n.name().value() == "example")
        .filter_map(|n| {
            let code = n
                .entries()
                .iter()
                .find(|e| e.name().is_none())?
                .value()
                .as_string()?
                .to_string();
            let title = n
                .get("title")
                .and_then(|v| v.as_string())
                .map(str::to_string);
            let fails = n.get("fails").and_then(|v| v.as_bool()) == Some(true);
            Some(Example { title, code, fails })
        })
        .collect()
}

/// The names and operators in a command's usage: the first word of each
/// line, such as `FORWARD` and `FD`, and the operator in an infix line such
/// as `num1 + num2`. Lines that show a variable, such as `:quoted.varname`,
/// give none.
fn names_in(usage: &str) -> (Vec<String>, Vec<String>) {
    let (mut names, mut operators): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
    for line in usage.lines() {
        let words: Vec<_> = line
            .trim()
            .trim_start_matches('(')
            .split_whitespace()
            .collect();
        // `(EDIT)` is EDIT with no inputs.
        let Some(first) = words.first().map(|w| w.trim_end_matches(')')) else {
            continue;
        };
        let name = if OPERATORS.contains(&first) {
            Err(first)
        } else if let Some(op) = words.get(1).filter(|w| OPERATORS.contains(w)) {
            Err(*op)
        } else if first.starts_with([':', '"']) {
            continue;
        } else {
            Ok(first)
        };
        match name {
            Ok(n) if !names.iter().any(|m| m.eq_ignore_ascii_case(n)) => {
                names.push(n.to_uppercase())
            }
            Err(op) if !operators.iter().any(|o| o == op) => operators.push(op.to_string()),
            _ => {}
        }
    }
    (names, operators)
}

/// Whether the interpreter has the command or special variable with this
/// key, its main name in the kdl. Not any of its names: a usage can show
/// other commands, as `USUAL`'s shows `TO` and `END`.
fn implemented(key: &str, kind: Kind) -> bool {
    match kind {
        Kind::Syntax => true,
        Kind::Variable => crate::special_variables()
            .iter()
            .any(|v| v.eq_ignore_ascii_case(key)),
        // TO and END are syntax to the interpreter, not primitives.
        Kind::Command => {
            crate::command_names().any(|ns| ns.iter().any(|n| n.eq_ignore_ascii_case(key)))
                || key == "to"
                || key == "end"
        }
    }
}

impl Reference {
    pub fn parse(source: &str) -> Result<Reference, String> {
        let document: KdlDocument = source.parse().map_err(|e| format!("{e:?}"))?;
        let mut reference = Reference {
            chapters: Vec::new(),
            commands: Vec::new(),
            index: HashMap::new(),
        };
        if let Some(syntax) = document.get("syntax") {
            let chapter = Chapter {
                key: "syntax".into(),
                title: "Syntax".into(),
                sections: Vec::new(),
                commands: Vec::new(),
            };
            reference.chapters.push(chapter);
            for node in syntax.children().map(|c| c.nodes()).unwrap_or_default() {
                let i = reference.add(node, Kind::Syntax, 0, None);
                reference.chapters[0].commands.push(i);
            }
        }
        let logo = document.get("logo").ok_or("no logo node")?;
        for chapter_node in logo.children().map(|c| c.nodes()).unwrap_or_default() {
            let c = reference.chapters.len();
            let key = chapter_node.name().value().to_string();
            reference.chapters.push(Chapter {
                title: title(&key),
                key,
                sections: Vec::new(),
                commands: Vec::new(),
            });
            for node in chapter_node
                .children()
                .map(|c| c.nodes())
                .unwrap_or_default()
            {
                if child(node, "eg").is_some() {
                    let i = reference.add(node, Kind::Command, c, None);
                    reference.chapters[c].commands.push(i);
                    continue;
                }
                let s = reference.chapters[c].sections.len();
                let key = node.name().value().to_string();
                reference.chapters[c].sections.push(Section {
                    title: title(&key),
                    key,
                    commands: Vec::new(),
                });
                for command in node.children().map(|c| c.nodes()).unwrap_or_default() {
                    let i = reference.add(command, Kind::Command, c, Some(s));
                    reference.chapters[c].sections[s].commands.push(i);
                }
            }
        }
        Ok(reference)
    }

    fn add(&mut self, node: &KdlNode, kind: Kind, chapter: usize, section: Option<usize>) -> usize {
        let usage = string(node, "eg").unwrap_or_default();
        // A syntax entry's usage shows operators and such, which are other
        // pages: it's found only by its `names`.
        let (mut names, operators) = if kind == Kind::Syntax {
            Default::default()
        } else {
            names_in(&usage)
        };
        let kind = if kind == Kind::Command && usage.contains("(variable)") {
            Kind::Variable
        } else {
            kind
        };
        for extra in strings(node, "names") {
            if !names.iter().any(|n| n.eq_ignore_ascii_case(&extra)) {
                names.push(extra.to_uppercase());
            }
        }
        let key = node.name().value().to_string();
        let obsolete = child(node, "obsolete").and_then(|v| v.as_bool()) == Some(true);
        let command = Command {
            title: string(node, "title")
                .or_else(|| names.first().cloned())
                .unwrap_or_else(|| key.to_uppercase()),
            implemented: implemented(&key, kind),
            key,
            operators,
            kind,
            desc: string(node, "desc").unwrap_or_default(),
            help: string(node, "help"),
            examples: examples(node),
            see: strings(node, "see"),
            phase: child(node, "phase")
                .and_then(|v| v.as_integer())
                .map(|p| p as u8),
            extension: child(node, "extension").and_then(|v| v.as_bool()) == Some(true),
            obsolete: if obsolete {
                string(node, "obsolete_reason")
            } else {
                None
            },
            chapter,
            section,
            usage,
            names,
        };
        let i = self.commands.len();
        // A name's first entry wins, so syntax comes before the commands,
        // and implemented commands before ones that aren't.
        for name in command.names.iter().chain(&command.operators) {
            let name = name.to_lowercase();
            let replace = self
                .index
                .get(&name)
                .is_some_and(|&j| !self.commands[j].implemented && command.implemented);
            if replace || !self.index.contains_key(&name) {
                self.index.insert(name, i);
            }
        }
        self.commands.push(command);
        i
    }

    /// The command or syntax entry with this name, alias or operator, in any
    /// case.
    pub fn find(&self, name: &str) -> Option<&Command> {
        self.find_index(name).map(|i| &self.commands[i])
    }

    /// By name, alias or operator, or failing that by the kdl's key, which
    /// is how a link names a syntax entry, as in `[parentheses]`.
    pub fn find_index(&self, name: &str) -> Option<usize> {
        let name = name.to_lowercase();
        self.index
            .get(&name)
            .copied()
            .or_else(|| self.commands.iter().position(|c| c.key == name))
    }

    /// Commands with a name containing `query`: an exact match first, then
    /// names starting with it, then names with it anywhere, each shorter
    /// names first, then alphabetically. Only implemented ones, and obsolete
    /// ones if asked for.
    pub fn search(&self, query: &str, obsolete: bool) -> Vec<usize> {
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }
        let mut hits: Vec<(usize, usize, &str)> = Vec::new();
        for (i, command) in self.commands.iter().enumerate() {
            if !command.implemented && !(obsolete && command.obsolete.is_some()) {
                continue;
            }
            let best = command
                .names
                .iter()
                .chain(&command.operators)
                .filter_map(|n| {
                    let lower = n.to_lowercase();
                    let rank = if lower == query {
                        0
                    } else if lower.starts_with(&query) {
                        1
                    } else if lower.contains(&query) {
                        2
                    } else {
                        return None;
                    };
                    Some((rank, n.len(), n.as_str()))
                })
                .min();
            if let Some((rank, _, name)) = best {
                hits.push((rank, i, name));
            }
        }
        hits.sort_by(|a, b| (a.0, a.2.len(), a.2).cmp(&(b.0, b.2.len(), b.2)));
        hits.into_iter().map(|(_, i, _)| i).collect()
    }

    /// The other commands to suggest after this one: its `see` list, then
    /// the rest of its section, implemented ones only.
    pub fn see_also(&self, i: usize) -> Vec<usize> {
        let command = &self.commands[i];
        let chapter = &self.chapters[command.chapter];
        let section: &[usize] = match command.section {
            Some(s) => &chapter.sections[s].commands,
            None => &chapter.commands,
        };
        let mut out: Vec<usize> = command
            .see
            .iter()
            .filter_map(|n| self.find_index(n))
            .collect();
        out.extend(section.iter().copied());
        let mut seen = std::collections::HashSet::new();
        out.retain(|&j| j != i && self.commands[j].implemented && seen.insert(j));
        out
    }

    /// Why a name isn't in Pendown, if it's an obsolete command.
    pub fn obsolete_reason(&self, name: &str) -> Option<&str> {
        let command = &self.commands[self.find_index(name)?];
        if command.implemented {
            None
        } else {
            command.obsolete.as_deref()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_come_from_the_usage() {
        assert_eq!(
            names_in("FORWARD dist\nFD dist"),
            (vec!["FORWARD".into(), "FD".into()], vec![])
        );
        assert_eq!(
            names_in("SUM num1 num2\n(SUM num1 num2 num3 ...)\nnum1 + num2"),
            (vec!["SUM".into()], vec!["+".into()])
        );
        assert_eq!(
            names_in("MINUS num\n- num"),
            (vec!["MINUS".into()], vec!["-".into()])
        );
        assert_eq!(
            names_in("EDIT contentslist\nED contentslist\n(EDIT)\n(ED)").0,
            ["EDIT", "ED"]
        );
    }

    #[test]
    fn titles() {
        assert_eq!(title("turtle-motion"), "Turtle motion");
        assert_eq!(title("and"), "And");
    }
}
