//! Prints the built-in commands and special variables for tree-sitter-logo's
//! `queries/highlights.scm`. Its output goes between that file's `; BEGIN
//! GENERATED` and `; END GENERATED` lines, and the file is then copied to
//! zed-logo's `languages/logo/highlights.scm`.
//!
//! With those repositories next to this one, `UPDATE_HIGHLIGHTS=1 cargo test
//! -p pendown-logo --test tree_sitter` does all of this instead.

fn main() {
    print!(
        "{}",
        pendown_logo::reference::reference().highlight_patterns()
    );
}
