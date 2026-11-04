//! The tree-sitter grammar's highlights list Pendown's built-in commands and
//! special variables, generated from `content/reference.kdl`, and zed-logo has
//! a copy of the grammar's queries. These tests keep both up to date, when
//! those repositories are checked out next to this one, and pass with a note
//! when they aren't.
//!
//! After adding commands, run
//! `UPDATE_HIGHLIGHTS=1 cargo test -p pendown-logo --test tree_sitter`.
//! Without the checkouts, `cargo run -p pendown-logo --example highlights`
//! prints the list.

use std::path::PathBuf;

use pendown_logo::reference::reference;

const BEGIN: &str = "; BEGIN GENERATED\n";
const END: &str = "; END GENERATED\n";

/// A repository checked out next to this one, if it's there.
fn sibling(name: &str) -> Option<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(name);
    if !dir.is_dir() {
        eprintln!("Not checked: {name} isn't next to this repository");
        return None;
    }
    Some(dir)
}

#[test]
fn the_highlights_list_the_built_in_commands() {
    let Some(dir) = sibling("tree-sitter-logo") else {
        return;
    };
    let path = dir.join("queries/highlights.scm");
    let text = std::fs::read_to_string(&path).unwrap();
    let start = text.find(BEGIN).expect("the generated part's start") + BEGIN.len();
    let end = text.find(END).expect("the generated part's end");
    let expected = reference().highlight_patterns();
    if std::env::var_os("UPDATE_HIGHLIGHTS").is_some() {
        std::fs::write(
            &path,
            format!("{}{expected}{}", &text[..start], &text[end..]),
        )
        .unwrap();
        return;
    }
    assert!(
        text[start..end] == expected,
        "tree-sitter-logo/queries/highlights.scm is out of date: run UPDATE_HIGHLIGHTS=1 cargo test -p pendown-logo --test tree_sitter"
    );
}

/// Zed's extension has its own copy of the queries, which must be the same.
#[test]
fn the_zed_extension_has_the_same_queries() {
    let (Some(grammar), Some(zed)) = (sibling("tree-sitter-logo"), sibling("zed-logo")) else {
        return;
    };
    for name in ["highlights", "brackets", "outline", "indents", "folds"] {
        let queries = std::fs::read_to_string(grammar.join(format!("queries/{name}.scm"))).unwrap();
        let path = zed.join(format!("languages/logo/{name}.scm"));
        if std::env::var_os("UPDATE_HIGHLIGHTS").is_some() {
            std::fs::write(&path, &queries).unwrap();
        }
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            queries,
            "zed-logo/languages/logo/{name}.scm: run with UPDATE_HIGHLIGHTS=1"
        );
    }
}
