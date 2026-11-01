//! The help text in `pendown-logo.kdl` is good markup, and its links go
//! somewhere (phase 5 issue 03).

use pendown::markup::{self, Block};
use pendown_logo::reference::reference;

#[test]
fn every_link_in_the_help_resolves() {
    let r = reference();
    let mut broken = Vec::new();
    for c in r.commands.iter().filter(|c| c.implemented) {
        let blocks = markup::parse(c.help.as_deref().unwrap_or_default());
        for link in markup::links(&blocks) {
            if !r.find(link).is_some_and(|target| target.implemented) {
                broken.push(format!("{}: [{link}]", c.key));
            }
        }
    }
    assert!(broken.is_empty(), "{broken:#?}");
}

#[test]
fn help_has_no_code_blocks() {
    // Programs go in examples, which have Run and pictures.
    for c in reference().commands.iter().filter(|c| c.implemented) {
        let blocks = markup::parse(c.help.as_deref().unwrap_or_default());
        assert!(
            !blocks.iter().any(|b| matches!(b, Block::Code { .. })),
            "{}",
            c.key
        );
    }
}

#[test]
fn every_link_in_the_tutorial_resolves() {
    let r = reference();
    for (n, lesson) in pendown::tutorial::lessons().iter().enumerate() {
        for link in markup::links(&lesson.blocks) {
            assert!(
                r.find(link).is_some_and(|c| c.implemented),
                "lesson {}: [{link}]",
                n + 1
            );
        }
        assert!(
            lesson.programs().count() >= 3,
            "lesson {} has few programs",
            n + 1
        );
    }
    assert_eq!(pendown::tutorial::lessons().len(), 8);
}
