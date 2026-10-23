//! The tutorial (phase 5): lessons with Kame, in `tutorial/` at the top of
//! the repository, in the help panel's markup. Each starts with its title
//! as a heading. Their code blocks are programs, which load into the
//! editor and run.

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

/// A picture of Kame, for the top of each lesson.
pub const KAME: &[u8] = include_bytes!("../../../tutorial/kame-turtle.svg");

pub struct Lesson {
    pub title: String,
    /// Everything after the title.
    pub blocks: Vec<Block>,
}

impl Lesson {
    /// The programs in it, in order.
    pub fn programs(&self) -> impl Iterator<Item = &str> {
        self.blocks.iter().filter_map(|b| if let Block::Code { code, .. } = b { Some(code.as_str()) } else { None })
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
