//! The tutorial (phase 5): lessons with Kame, in `tutorial/` at the top of
//! the repository, in the help panel's markup. Each starts with its title
//! as a heading. Their code blocks are programs, which load into the
//! editor and run; a block marked `picture` shows only what its program
//! draws, so a lesson can show what Kame meant to draw without the program.

use std::sync::OnceLock;

use crate::markup::{self, Block};

/// The lessons' files, in order.
const SOURCES: [&str; 6] = [
    include_str!("../../../tutorial/01-moving-and-turning.md"),
    include_str!("../../../tutorial/02-repeat.md"),
    include_str!("../../../tutorial/03-procedures.md"),
    include_str!("../../../tutorial/04-inputs-and-variables.md"),
    include_str!("../../../tutorial/05-recursion.md"),
    include_str!("../../../tutorial/06-colours-and-shapes.md"),
];

/// The language that marks a code block as a picture: only what its
/// program draws is shown.
pub const PICTURE: &str = "picture";

/// A picture of Kame, for the top of each lesson.
pub const KAME: &[u8] = include_bytes!("../../../tutorial/kame-turtle.svg");

pub struct Lesson {
    pub title: String,
    /// Everything after the title.
    pub blocks: Vec<Block>,
}

impl Lesson {
    /// The programs in it, with Load into editor and Run, in order.
    pub fn programs(&self) -> impl Iterator<Item = &str> {
        self.code(false)
    }

    /// The programs of its pictures, in order.
    pub fn pictures(&self) -> impl Iterator<Item = &str> {
        self.code(true)
    }

    fn code(&self, pictures: bool) -> impl Iterator<Item = &str> {
        self.blocks.iter().filter_map(move |b| match b {
            Block::Code { language, code } if (language == PICTURE) == pictures => {
                Some(code.as_str())
            }
            _ => None,
        })
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
}
