//! The gallery (`.scratch/gallery/`, ADR 0030): sections of examples to
//! run or load into the editor, from `content/gallery.kdl` at the top of the
//! repository. Each entry points at an example where it already lives (a
//! help example, a lesson's program, or a program in `examples/gallery/`),
//! so nothing is copied. Everything is built into the app.

use std::sync::OnceLock;

use kdl::{KdlDocument, KdlNode};
use pendown_logo::reference::reference;

use crate::{source::Source, tutorial};

const INDEX: &str = include_str!("../../../content/gallery.kdl");

/// The gallery's own programs, by their path from the top of the
/// repository. A test checks this is every `.logo` file in
/// `examples/gallery/`, and that `content/gallery.kdl` uses each.
pub const FILES: [(&str, &str); 10] = [
    (
        "examples/gallery/snowflake.logo",
        include_str!("../../../examples/gallery/snowflake.logo"),
    ),
    (
        "examples/gallery/spirograph.logo",
        include_str!("../../../examples/gallery/spirograph.logo"),
    ),
    (
        "examples/gallery/blossom.logo",
        include_str!("../../../examples/gallery/blossom.logo"),
    ),
    (
        "examples/gallery/city.logo",
        include_str!("../../../examples/gallery/city.logo"),
    ),
    (
        "examples/gallery/garden.logo",
        include_str!("../../../examples/gallery/garden.logo"),
    ),
    (
        "examples/gallery/spiderweb.logo",
        include_str!("../../../examples/gallery/spiderweb.logo"),
    ),
    (
        "examples/gallery/ghost.logo",
        include_str!("../../../examples/gallery/ghost.logo"),
    ),
    (
        "examples/gallery/lightning.logo",
        include_str!("../../../examples/gallery/lightning.logo"),
    ),
    (
        "examples/gallery/pumpkin.logo",
        include_str!("../../../examples/gallery/pumpkin.logo"),
    ),
    (
        "examples/gallery/cauldron.logo",
        include_str!("../../../examples/gallery/cauldron.logo"),
    ),
];

pub struct Section {
    pub title: String,
    pub entries: Vec<Entry>,
}

pub struct Entry {
    pub title: String,
    pub source: Source,
}

/// Where an entry comes from, for its card: the help page or lesson it
/// links to, or the gallery's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum From {
    /// A help example: the command's index in the reference.
    Help(usize),
    /// A lesson's program: the lesson, counting from 0.
    Lesson(usize),
    /// Written for the gallery.
    New,
}

impl Entry {
    pub fn from(&self) -> From {
        match self.source {
            Source::Example { command, .. } => From::Help(command),
            Source::Lesson { lesson, .. } => From::Lesson(lesson),
            Source::File(_) => From::New,
        }
    }

    /// What its card says it's from: "Help: REPEAT", "Lesson 6" or "New".
    pub fn from_label(&self) -> String {
        match self.from() {
            From::Help(command) => format!("Help: {}", reference().commands[command].title),
            From::Lesson(lesson) => format!("Lesson {}", lesson + 1),
            From::New => "New".to_string(),
        }
    }
}

/// The gallery, parsed the first time it's needed. An entry that points at
/// nothing is left out; the test says which.
pub fn gallery() -> &'static [Section] {
    static GALLERY: OnceLock<Vec<Section>> = OnceLock::new();
    GALLERY.get_or_init(|| parse(INDEX).0)
}

/// The title a gallery entry gives this source, if one does.
pub fn title_of(source: Source) -> Option<&'static str> {
    gallery()
        .iter()
        .flat_map(|s| &s.entries)
        .find(|e| e.source == source)
        .map(|e| e.title.as_str())
}

/// The sections in `text`, and a message for each entry that points at
/// nothing.
pub fn parse(text: &str) -> (Vec<Section>, Vec<String>) {
    let mut problems = Vec::new();
    let document: KdlDocument = match text.parse() {
        Ok(d) => d,
        Err(e) => {
            return (
                Vec::new(),
                vec![format!("content/gallery.kdl doesn't parse: {e:?}")],
            );
        }
    };
    let mut sections = Vec::new();
    for node in document.nodes() {
        if node.name().value() != "section" {
            problems.push(format!("unknown node `{}`", node.name().value()));
            continue;
        }
        let title = string_arg(node, 0).unwrap_or_default();
        let mut entries = Vec::new();
        for entry in node.children().map(|c| c.nodes()).unwrap_or_default() {
            match resolve(entry) {
                Ok(e) => entries.push(e),
                Err(why) => problems.push(format!("in “{title}”: {}: {why}", entry)),
            }
        }
        sections.push(Section { title, entries });
    }
    (sections, problems)
}

fn string_arg(node: &KdlNode, i: usize) -> Option<String> {
    node.entries()
        .iter()
        .filter(|e| e.name().is_none())
        .nth(i)?
        .value()
        .as_string()
        .map(str::to_string)
}

fn int_arg(node: &KdlNode, i: usize) -> Option<i128> {
    node.entries()
        .iter()
        .filter(|e| e.name().is_none())
        .nth(i)?
        .value()
        .as_integer()
}

fn title_property(node: &KdlNode) -> Result<String, String> {
    node.get("title")
        .and_then(|v| v.as_string())
        .map(str::to_string)
        .ok_or_else(|| "it needs a title=\"...\"".to_string())
}

/// Finds what an entry points at.
fn resolve(node: &KdlNode) -> Result<Entry, String> {
    match node.name().value() {
        "help" => {
            let (Some(key), Some(title)) = (string_arg(node, 0), string_arg(node, 1)) else {
                return Err("it needs a command and an example's title".into());
            };
            let r = reference();
            let command = r
                .commands
                .iter()
                .position(|c| c.key == key)
                .ok_or_else(|| format!("there's no command `{key}`"))?;
            if !r.commands[command].implemented {
                return Err(format!("`{key}` isn't implemented"));
            }
            let example = r.commands[command]
                .examples
                .iter()
                .position(|e| e.title.as_deref() == Some(title.as_str()))
                .ok_or_else(|| format!("`{key}` has no example “{title}”"))?;
            if r.commands[command].examples[example].fails {
                return Err(format!("“{title}” stops with an error on purpose"));
            }
            Ok(Entry {
                title,
                source: Source::Example { command, example },
            })
        }
        "lesson" => {
            let lesson = int_arg(node, 0).ok_or("it needs a lesson number")?;
            let name = string_arg(node, 1)
                .ok_or("it needs a program's name, after its fence's language")?;
            let lessons = tutorial::lessons();
            if lesson < 1 || lesson as usize > lessons.len() {
                return Err(format!("there's no lesson {lesson}"));
            }
            let lesson = lesson as usize - 1;
            let program = lessons[lesson]
                .program_named(&name)
                .ok_or_else(|| format!("lesson {} has no program “{name}”", lesson + 1))?;
            Ok(Entry {
                title: title_property(node)?,
                source: Source::Lesson { lesson, program },
            })
        }
        "file" => {
            let path = string_arg(node, 0).ok_or("it needs a path")?;
            let i = FILES
                .iter()
                .position(|(p, _)| *p == path)
                .ok_or_else(|| format!("`{path}` isn't in gallery::FILES"))?;
            Ok(Entry {
                title: title_property(node)?,
                source: Source::File(i),
            })
        }
        other => Err(format!("unknown entry `{other}`")),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn every_entry_points_at_an_example() {
        let (sections, problems) = parse(INDEX);
        assert_eq!(problems, Vec::<String>::new());
        let entries = sections.iter().map(|s| s.entries.len()).sum::<usize>();
        assert!(entries >= 30, "{entries} entries");
    }

    #[test]
    fn no_example_is_in_twice() {
        let mut seen = HashSet::new();
        for entry in gallery().iter().flat_map(|s| &s.entries) {
            assert!(seen.insert(entry.source), "{} is in twice", entry.title);
        }
    }

    #[test]
    fn every_gallery_program_is_built_in_and_used() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/gallery");
        let mut on_disk: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".logo"))
            .map(|n| format!("examples/gallery/{n}"))
            .collect();
        on_disk.sort();
        let mut built_in: Vec<String> = FILES.iter().map(|(p, _)| p.to_string()).collect();
        built_in.sort();
        assert_eq!(on_disk, built_in, "gallery::FILES and examples/gallery/");
        for (i, (path, _)) in FILES.iter().enumerate() {
            assert!(
                title_of(Source::File(i)).is_some(),
                "content/gallery.kdl doesn't use {path}"
            );
        }
    }

    #[test]
    fn a_broken_entry_says_why() {
        let (_, problems) = parse(
            "section \"S\" {\n  help \"repeat\" \"No such example\"\n  lesson 6 \"bush\" title=\"X\"\n  file \"examples/gallery/none.logo\" title=\"Y\"\n}",
        );
        assert_eq!(problems.len(), 3, "{problems:?}");
        assert!(problems[0].contains("has no example “No such example”"));
        assert!(problems[1].contains("lesson 6 has no program “bush”"));
        assert!(problems[2].contains("isn't in gallery::FILES"));
    }

    #[test]
    fn labels_say_where_entries_come_from() {
        let labels: Vec<String> = gallery()
            .iter()
            .flat_map(|s| &s.entries)
            .map(|e| e.from_label())
            .collect();
        assert!(labels.contains(&"Help: REPEAT".to_string()));
        assert!(labels.contains(&"Lesson 6".to_string()));
        assert!(labels.contains(&"New".to_string()));
    }
}
