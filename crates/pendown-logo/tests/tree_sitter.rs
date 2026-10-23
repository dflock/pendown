//! The tree-sitter grammar's highlights list Pendown's built-in commands
//! and special variables, generated from `pendown-logo.kdl`, and this test
//! keeps them up to date (phase 5 issue 10). After adding commands, run
//! `UPDATE_HIGHLIGHTS=1 cargo test -p pendown-logo --test tree_sitter`.

use std::path::PathBuf;

use pendown_logo::reference::{Kind, reference};

const BEGIN: &str = "; BEGIN GENERATED\n";
const END: &str = "; END GENERATED\n";

/// A name in a tree-sitter `#match?` pattern, which is a Rust regex in a
/// query string: the string takes one backslash off, so a regex's `\?` is
/// written `\\?`.
fn escape(name: &str) -> String {
    name.chars().flat_map(|c| if ".?*+()[]{}|^$\\#".contains(c) { vec!['\\', '\\', c] } else { vec![c] }).collect()
}

fn generated() -> String {
    let r = reference();
    let names = |kind: Kind| {
        let mut names: Vec<String> = r
            .commands
            .iter()
            .filter(|c| c.implemented && c.kind == kind)
            .flat_map(|c| c.names.iter())
            .filter(|n| n.chars().next().is_some_and(|c| c.is_alphabetic() || c == '.'))
            .map(|n| escape(&n.to_lowercase()))
            .collect();
        names.sort();
        names.dedup();
        names.join("|")
    };
    let (commands, variables) = (names(Kind::Command), names(Kind::Variable));
    // Each list twice: what matches is built in, and everything else is a
    // procedure of the program, or a variable, so no two patterns match the
    // same node.
    format!(
        "((word) @function.builtin\n  (#match? @function.builtin \"^(?i)({commands})$\"))\n\n\
         ((word) @function.call\n  (#not-match? @function.call \"^(?i)({commands})$\"))\n\n\
         ((variable) @variable.builtin\n  (#match? @variable.builtin \"^:(?i)({variables})$\"))\n\n\
         ((variable) @variable\n  (#not-match? @variable \"^:(?i)({variables})$\"))\n"
    )
}

#[test]
fn the_highlights_list_the_built_in_commands() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tree-sitter-logo/queries/highlights.scm");
    let text = std::fs::read_to_string(&path).unwrap();
    let start = text.find(BEGIN).expect("the generated part's start") + BEGIN.len();
    let end = text.find(END).expect("the generated part's end");
    let expected = generated();
    if std::env::var_os("UPDATE_HIGHLIGHTS").is_some() {
        std::fs::write(&path, format!("{}{expected}{}", &text[..start], &text[end..])).unwrap();
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
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tree-sitter-logo");
    for name in ["highlights", "brackets", "outline", "indents", "folds"] {
        let queries = std::fs::read_to_string(dir.join(format!("queries/{name}.scm"))).unwrap();
        let path = dir.join(format!("zed/languages/logo/{name}.scm"));
        if std::env::var_os("UPDATE_HIGHLIGHTS").is_some() {
            std::fs::write(&path, &queries).unwrap();
        }
        assert_eq!(std::fs::read_to_string(&path).unwrap(), queries, "zed/languages/logo/{name}.scm: run with UPDATE_HIGHLIGHTS=1");
    }
}
