//! Where a program that runs on the canvas, or loads into the editor,
//! comes from: a help example, a lesson's program, or one of the gallery's
//! own programs (`.scratch/gallery/`). The help panel, the tutorial, the
//! gallery and the banner over the canvas all name programs this way.

use pendown_logo::reference::reference;

use crate::{gallery, tutorial};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Source {
    /// A help example: the command's index in the reference, and the
    /// example's among its examples.
    Example { command: usize, example: usize },
    /// A lesson's program, both counting from 0.
    Lesson { lesson: usize, program: usize },
    /// One of the gallery's own programs, in `examples/gallery/`, by its
    /// index in [`gallery::FILES`].
    File(usize),
}

impl Source {
    /// Its code.
    pub fn code(&self) -> String {
        match *self {
            Source::Example { command, example } => {
                reference().commands[command].examples[example].code.clone()
            }
            Source::Lesson { lesson, program } => tutorial::lessons()[lesson]
                .programs()
                .nth(program)
                .unwrap_or_default()
                .to_string(),
            Source::File(i) => gallery::FILES[i].1.to_string(),
        }
    }

    /// What the banner over the canvas says it is.
    pub fn describe(&self) -> String {
        match *self {
            Source::Example { command, example } => {
                let command = &reference().commands[command];
                match &command.examples[example].title {
                    Some(title) => format!("“{title}”, from {}", command.title),
                    None => format!("From {}", command.title),
                }
            }
            Source::Lesson { lesson, .. } => {
                format!("From the lesson “{}”", tutorial::lessons()[lesson].title)
            }
            Source::File(_) => match gallery::title_of(*self) {
                Some(title) => format!("“{title}”, from the gallery"),
                None => "From the gallery".to_string(),
            },
        }
    }
}
