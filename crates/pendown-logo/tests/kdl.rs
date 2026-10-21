//! `pendown-logo.kdl` is the source of truth for which commands exist (see
//! `implementation-plan.md`), so it must always parse.

use kdl::KdlDocument;

#[test]
fn pendown_logo_kdl_parses() {
    let text = include_str!("../../../pendown-logo.kdl");
    let doc: KdlDocument = text.parse().unwrap_or_else(|e| panic!("pendown-logo.kdl doesn't parse: {e:?}"));
    assert!(!doc.nodes().is_empty());
}
