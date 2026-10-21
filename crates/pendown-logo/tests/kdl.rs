//! `pendown-logo.kdl` is the source of truth for which commands exist (see
//! `implementation-plan.md`), so it must parse, and the interpreter must
//! implement exactly the commands it marks as phases 1 and 2.

use std::collections::BTreeSet;

use kdl::{KdlDocument, KdlNode};

fn document() -> KdlDocument {
    let text = include_str!("../../../pendown-logo.kdl");
    text.parse().unwrap_or_else(|e| panic!("pendown-logo.kdl doesn't parse: {e:?}"))
}

/// Every command node: one with an `eg` child. Chapters and sections hold
/// them at different depths.
fn commands(nodes: &[KdlNode], out: &mut Vec<KdlNode>) {
    for node in nodes {
        let Some(children) = node.children() else { continue };
        if children.get("eg").is_some() {
            out.push(node.clone());
        } else {
            commands(children.nodes(), out);
        }
    }
}

fn value<'a>(node: &'a KdlNode, name: &str) -> Option<&'a kdl::KdlValue> {
    node.children()?.get(name)?.entries().first().map(|e| e.value())
}

/// The names a command is called by: the first word of each line of its
/// `eg`, such as `FORWARD` and `FD`. Lines that show an infix form, such as
/// `num1 + num2`, or a variable, such as `:quoted.varname`, are skipped.
fn names(node: &KdlNode) -> Vec<String> {
    const OPERATORS: [&str; 10] = ["+", "-", "*", "/", "=", "<", ">", "<=", ">=", "<>"];
    let eg = value(node, "eg").and_then(|v| v.as_string()).unwrap_or_default();
    eg.lines()
        .filter_map(|line| {
            let words: Vec<_> = line.trim().trim_start_matches('(').split_whitespace().collect();
            let first = *words.first()?;
            let infix = words.get(1).is_some_and(|w| OPERATORS.contains(w));
            let operator = OPERATORS.contains(&first);
            (!infix && !operator && !first.starts_with([':', '"'])).then(|| first.to_lowercase())
        })
        .collect()
}

#[test]
fn pendown_logo_kdl_parses() {
    assert!(!document().nodes().is_empty());
}

fn phase(node: &KdlNode) -> Option<i128> {
    value(node, "phase").and_then(|v| v.as_integer())
}

/// Special variables, such as `CASEIGNOREDP`, are documented like commands,
/// with `(variable)` in their `eg`.
fn is_variable(node: &KdlNode) -> bool {
    value(node, "eg").and_then(|v| v.as_string()).is_some_and(|eg| eg.contains("(variable)"))
}

#[test]
fn the_interpreter_implements_exactly_the_phase_1_and_2_commands() {
    let mut all = Vec::new();
    commands(document().nodes(), &mut all);
    assert_eq!(all.iter().filter(|n| phase(n) == Some(1)).count(), 90, "the plan's count for phase 1");
    assert_eq!(all.iter().filter(|n| phase(n) == Some(2)).count(), 76, "the plan's count for phase 2");
    let supported: Vec<_> = all.iter().filter(|n| matches!(phase(n), Some(1 | 2))).collect();
    let (variables, commands): (Vec<_>, Vec<_>) = supported.into_iter().partition(|n| is_variable(n));

    let specials: BTreeSet<String> = pendown_logo::special_variables().iter().map(|n| n.to_lowercase()).collect();
    let documented: BTreeSet<String> = variables.iter().flat_map(|n| names(n)).collect();
    assert_eq!(specials, documented, "the special variables");

    let implemented: BTreeSet<String> =
        pendown_logo::command_names().flat_map(|names| names.iter().map(|n| n.to_string())).collect();
    // TO and END are part of the syntax rather than commands.
    let syntax = ["to", "end"];
    let mut missing = Vec::new();
    let mut documented = BTreeSet::new();
    for node in &commands {
        for name in names(node) {
            documented.insert(name.clone());
            if !implemented.contains(&name) && !syntax.contains(&name.as_str()) {
                missing.push(name);
            }
        }
    }
    assert!(missing.is_empty(), "phase 1 and 2 commands the interpreter doesn't have: {missing:?}");
    // `#` is REPCOUNT's short form, which the manual gives in REPCOUNT's
    // description. `?` and `?REST` are template slots, described with MAP.
    let described = ["#", "?", "?rest"];
    let extra: Vec<_> = implemented.iter().filter(|n| !documented.contains(*n) && !described.contains(&n.as_str())).collect();
    assert!(extra.is_empty(), "commands the interpreter has that aren't phase 1 or 2 in the kdl: {extra:?}");
}

/// The example programs in `examples/phase-1/` use every phase-1 command at
/// least once, under any of its names. `pendown-cli`'s picture tests run
/// them.
#[test]
fn the_examples_use_every_phase_1_command() {
    use pendown_logo::syntax::{Kind, highlight};
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/phase-1");
    let mut used = BTreeSet::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "logo") {
            continue;
        }
        let text = std::fs::read_to_string(path).unwrap();
        for (range, kind) in highlight(&text) {
            if matches!(kind, Kind::Command | Kind::Keyword) {
                used.insert(text[range].to_lowercase());
            }
        }
    }
    let mut all = Vec::new();
    commands(document().nodes(), &mut all);
    let unused: Vec<_> = all
        .iter()
        .filter(|n| value(n, "phase").and_then(|v| v.as_integer()) == Some(1))
        .filter(|n| !names(n).iter().any(|name| used.contains(name)))
        .map(|n| n.name().value().to_string())
        .collect();
    assert!(unused.is_empty(), "phase 1 commands no example uses: {unused:?}");
}
