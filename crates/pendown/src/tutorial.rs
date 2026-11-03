//! The tutorial (phase 5): lessons with Kame, in `content/tutorial/` at the top of
//! the repository, in the help panel's markup. Each starts with its title
//! as a heading. Their code blocks are programs, which load into the
//! editor and run; a block marked `picture` shows only what its program
//! draws, so a lesson can show what Kame meant to draw without the program.

use std::sync::OnceLock;

use crate::markup::{self, Block};

/// The lessons' files, in order.
const SOURCES: [&str; 8] = [
    include_str!("../../../content/tutorial/01-moving-and-turning.md"),
    include_str!("../../../content/tutorial/02-repeat.md"),
    include_str!("../../../content/tutorial/03-procedures.md"),
    include_str!("../../../content/tutorial/04-help-kame-fix-the-bug.md"),
    include_str!("../../../content/tutorial/05-inputs-and-variables.md"),
    include_str!("../../../content/tutorial/06-recursion.md"),
    include_str!("../../../content/tutorial/07-colours-and-shapes.md"),
    include_str!("../../../content/tutorial/08-exploring-l-systems.md"),
];

/// The language that marks a code block as a picture: only what its
/// program draws is shown, with the rest of the fence line as its caption.
pub const PICTURE: &str = "picture";

/// The language that marks a code block as text: shown as it is, with no
/// Load into editor or Run, as for a rule's letters.
pub const TEXT: &str = "text";

/// A picture of Kame, for the top of each lesson.
pub const KAME: &[u8] = include_bytes!("../../../content/tutorial/kame-turtle.svg");

pub struct Lesson {
    pub title: String,
    /// Everything after the title.
    pub blocks: Vec<Block>,
}

impl Lesson {
    /// The programs in it, with Load into editor and Run, in order.
    pub fn programs(&self) -> impl Iterator<Item = &str> {
        self.code(Kind::Program)
    }

    /// Where the program named `name` (after the fence's language, as in
    /// ```` ```logo tree ````) is among [`Lesson::programs`].
    pub fn program_named(&self, name: &str) -> Option<usize> {
        self.blocks
            .iter()
            .filter(|b| kind(b) == Some(Kind::Program))
            .position(|b| matches!(b, Block::Code { name: Some(n), .. } if n == name))
    }

    /// The programs of its pictures, in order.
    pub fn pictures(&self) -> impl Iterator<Item = &str> {
        self.code(Kind::Picture)
    }

    fn code(&self, wanted: Kind) -> impl Iterator<Item = &str> {
        self.blocks.iter().filter_map(move |b| match b {
            Block::Code { code, .. } if kind(b) == Some(wanted) => Some(code.as_str()),
            _ => None,
        })
    }
}

/// What a code block in a lesson is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    /// A program, with Load into editor and Run.
    Program,
    /// Only what its program draws, with a caption.
    Picture,
    /// Text, shown as it is.
    Text,
}

/// What `block` is, if it's a code block.
pub fn kind(block: &Block) -> Option<Kind> {
    match block {
        Block::Code { language, .. } if language == PICTURE => Some(Kind::Picture),
        Block::Code { language, .. } if language == TEXT => Some(Kind::Text),
        Block::Code { .. } => Some(Kind::Program),
        _ => None,
    }
}

pub fn lessons() -> &'static [Lesson] {
    static LESSONS: OnceLock<Vec<Lesson>> = OnceLock::new();
    LESSONS.get_or_init(|| {
        SOURCES
            .iter()
            .map(|source| {
                let mut blocks = markup::parse(source);
                let title = match blocks.first() {
                    Some(Block::Heading(1, inlines)) => markup::plain(inlines),
                    _ => panic!("a lesson starts with its title"),
                };
                blocks.remove(0);
                Lesson { title, blocks }
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pictures_are_apart_from_programs() {
        let lesson = Lesson {
            title: "Bugs".into(),
            blocks: markup::parse(
                "Kame meant this:\n\n```picture\nREPEAT 4 [FD 100 RT 90]\n```\n\n```logo\nREPEAT 4 [FD 100 RT 80]\n```\n\n```\nFD 10\n```\n",
            ),
        };
        assert_eq!(
            lesson.programs().collect::<Vec<_>>(),
            ["REPEAT 4 [FD 100 RT 80]", "FD 10"]
        );
        assert_eq!(
            lesson.pictures().collect::<Vec<_>>(),
            ["REPEAT 4 [FD 100 RT 90]"]
        );
    }

    #[test]
    fn text_is_neither_a_program_nor_a_picture() {
        let lesson = Lesson {
            title: "Rules".into(),
            blocks: markup::parse(
                "```logo\nFD 1\n```\n\n```text\nF-F++F-F\n```\n\n```picture A bump\nFD 2\n```\n\n```logo bump\nFD 3\n```\n",
            ),
        };
        assert_eq!(lesson.programs().collect::<Vec<_>>(), ["FD 1", "FD 3"]);
        assert_eq!(lesson.pictures().collect::<Vec<_>>(), ["FD 2"]);
        assert_eq!(lesson.program_named("bump"), Some(1));
        assert_eq!(kind(&lesson.blocks[1]), Some(Kind::Text));
    }

    #[test]
    fn no_two_programs_in_a_lesson_share_a_name() {
        for lesson in lessons() {
            let mut names = std::collections::HashSet::new();
            for block in lesson
                .blocks
                .iter()
                .filter(|b| kind(b) == Some(Kind::Program))
            {
                if let Block::Code {
                    name: Some(name), ..
                } = block
                {
                    assert!(names.insert(name), "“{}” has two “{name}”s", lesson.title);
                }
            }
        }
    }

    #[test]
    fn a_program_is_found_by_its_name() {
        let lesson = Lesson {
            title: "Trees".into(),
            blocks: markup::parse(
                "```picture tree\nFD 1\n```\n\n```logo\nFD 2\n```\n\n```logo tree\nFD 3\n```\n",
            ),
        };
        assert_eq!(lesson.program_named("tree"), Some(1));
        assert_eq!(lesson.program_named("bush"), None);
    }
}
