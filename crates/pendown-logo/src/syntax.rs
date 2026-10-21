//! What the editor needs from the interpreter while the user types: syntax
//! colours from the same tokens the parser uses, and parse errors (ADRs
//! 0001, 0005 and 0008).

use std::{collections::HashSet, ops::Range};

use crate::{
    error::Error,
    names::Names,
    parse::{self, Block, Cx, Expr, Target, Tok, Token},
    prims::{self, Imp, PRIMS},
    span::{SourceId, Span},
    value::Value,
};

/// What a piece of source text is, for colouring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Comment,
    /// `[`, `]`, `{`, `}`, `(` and `)`.
    Bracket,
    Number,
    /// A quoted word, such as `"red`.
    Quoted,
    /// `:name`.
    Variable,
    /// An infix operator, or a minus sign.
    Operator,
    /// `TO` and `END`.
    Keyword,
    /// A built-in command.
    Command,
    /// A procedure defined in the text.
    Procedure,
    /// A word that's none of those, which won't run.
    Unknown,
}

/// The words, lists and comments of `text`, as the reader sees them.
enum Piece {
    Word(Range<usize>),
    Bracket(Range<usize>),
    Comment(Range<usize>),
}

fn pieces(text: &str) -> Vec<Piece> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b' ' | b'\t' | b'\r' | b'\n' => i += 1,
            b'[' | b']' | b'{' => {
                out.push(Piece::Bracket(i..i + 1));
                i += 1;
            }
            // An array's origin is part of its closing brace: `}@0`.
            b'}' => {
                let start = i;
                i += 1;
                if b.get(i) == Some(&b'@') {
                    i += 1;
                    while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'-') {
                        i += 1;
                    }
                }
                out.push(Piece::Bracket(start..i));
            }
            b';' => {
                let start = i;
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
                out.push(Piece::Comment(start..i));
            }
            _ => {
                let start = i;
                while i < b.len() && !matches!(b[i], b' ' | b'\t' | b'\r' | b'\n' | b'[' | b']' | b'{' | b'}' | b';') {
                    i += 1;
                }
                out.push(Piece::Word(start..i));
            }
        }
    }
    out
}

/// The names `text` defines with `TO`, in lowercase.
fn defined(text: &str, pieces: &[Piece]) -> HashSet<String> {
    let mut names = HashSet::new();
    for pair in pieces.windows(2) {
        if let [Piece::Word(to), Piece::Word(name)] = pair
            && text[to.clone()].eq_ignore_ascii_case("to")
        {
            names.insert(text[name.clone()].to_lowercase());
        }
    }
    names
}

/// The colour of every token in `text`, in order.
pub fn highlight(text: &str) -> Vec<(Range<usize>, Kind)> {
    let pieces = pieces(text);
    let procedures = defined(text, &pieces);
    let span = |r: &Range<usize>| Span { source: SourceId::Editor, start: r.start as u32, end: r.end as u32, line: 0 };
    let mut out = Vec::new();
    for piece in &pieces {
        let word = match piece {
            Piece::Comment(r) => {
                out.push((r.clone(), Kind::Comment));
                continue;
            }
            Piece::Bracket(r) => {
                out.push((r.clone(), Kind::Bracket));
                continue;
            }
            Piece::Word(r) => r,
        };
        let mut tokens: Vec<Token> = Vec::new();
        parse::split_word(&text[word.clone()], Some(span(word)), span(word), &mut tokens);
        for Token { tok, span } in tokens {
            let range = span.range();
            let kind = match tok {
                Tok::Lit(_) => Kind::Number,
                Tok::Quoted(_) => Kind::Quoted,
                Tok::Var(_) => Kind::Variable,
                Tok::Op(_) | Tok::Neg => Kind::Operator,
                Tok::LParen | Tok::RParen => Kind::Bracket,
                Tok::List(_) => continue,
                Tok::Word(w) if w.eq_ignore_ascii_case("to") || w.eq_ignore_ascii_case("end") => Kind::Keyword,
                Tok::Word(w) if prims::lookup(&w).is_some() || parse::slot_number(&w).is_some() => Kind::Command,
                Tok::Word(w) if procedures.contains(&w.to_lowercase()) => Kind::Procedure,
                Tok::Word(_) => Kind::Unknown,
            };
            out.push((range, kind));
        }
    }
    out
}

/// Parses `text` as the editor's program would be on Run, without running
/// it, and returns every error: those of the parse itself, and those in the
/// instruction lists and templates given to commands such as `REPEAT`,
/// `IF`, `FOR` and `MAP`, which otherwise only show up when they run (ADR
/// 0001).
pub fn check(text: &str) -> Vec<Error> {
    let mut names = Names::default();
    let no_procedures = |_| None;
    let mut cx = Cx { names: &mut names, known: &no_procedures, defining: Default::default() };
    let (program, mut errors) = parse::parse_program_partly(text, SourceId::Editor, &mut cx);
    check_block(&program.top, &mut cx, &mut errors);
    for def in &program.procs {
        check_block(&def.body, &mut cx, &mut errors);
    }
    errors
}

fn check_block(block: &Block, cx: &mut Cx, errors: &mut Vec<Error>) {
    for instr in block {
        check_expr(&instr.expr, cx, errors);
    }
}

fn check_expr(expr: &Expr, cx: &mut Cx, errors: &mut Vec<Error>) {
    match expr {
        Expr::Neg(e, _) => check_expr(e, cx, errors),
        Expr::Call(call) => {
            // The inputs that are instruction lists or templates, for the
            // commands that run them. `FOREACH`'s template comes last.
            let last = call.args.len().saturating_sub(1);
            let lists: &[usize] = match call.target {
                Target::Prim(id) => match (PRIMS[id].imp, PRIMS[id].names[0]) {
                    (Imp::Repeat, _) => &[1],
                    (Imp::Forever, _) => &[0],
                    (Imp::If | Imp::IfElse, _) => &[1, 2],
                    (_, "run" | "runresult" | "apply" | "invoke" | "map" | "map.se" | "filter" | "find" | "reduce") => &[0],
                    (_, "for" | "catch") => &[1],
                    (_, "while" | "until" | "do.while" | "do.until") => &[0, 1],
                    (_, "foreach") => std::slice::from_ref(&last),
                    _ => &[],
                },
                _ => &[],
            };
            for (i, arg) in call.args.iter().enumerate() {
                match arg {
                    Expr::Lit(Value::List(list), span) if lists.contains(&i) => match parse::parse_list(list, *span, cx) {
                        Ok(block) => check_block(&block, cx, errors),
                        Err(e) => errors.push(e),
                    },
                    arg => check_expr(arg, cx, errors),
                }
            }
        }
        Expr::Lit(..) | Expr::Var(..) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(text: &str) -> Vec<(&str, Kind)> {
        highlight(text).into_iter().map(|(r, k)| (&text[r], k)).collect()
    }

    #[test]
    fn colours() {
        use Kind::*;
        assert_eq!(
            kinds("TO BOX :n ; a square\n REPEAT 4 [FD :n RT 90]\nEND\nBOX 50 SETPC \"red FOO -5"),
            [
                ("TO", Keyword),
                ("BOX", Procedure),
                (":n", Variable),
                ("; a square", Comment),
                ("REPEAT", Command),
                ("4", Number),
                ("[", Bracket),
                ("FD", Command),
                (":n", Variable),
                ("RT", Command),
                ("90", Number),
                ("]", Bracket),
                ("END", Keyword),
                ("BOX", Procedure),
                ("50", Number),
                ("SETPC", Command),
                ("\"red", Quoted),
                ("FOO", Unknown),
                ("-", Operator),
                ("5", Number),
            ]
        );
        assert_eq!(kinds("SHOW {a [b]}@0 ?2"), [
            ("SHOW", Command),
            ("{", Bracket),
            ("a", Unknown),
            ("[", Bracket),
            ("b", Unknown),
            ("]", Bracket),
            ("}@0", Bracket),
            ("?2", Command),
        ]);
        assert_eq!(kinds("(SUM :a-1 2)"), [
            ("(", Bracket),
            ("SUM", Command),
            (":a", Variable),
            ("-", Operator),
            ("1", Number),
            ("2", Number),
            (")", Bracket),
        ]);
    }

    fn messages(text: &str) -> Vec<(String, String)> {
        check(text)
            .into_iter()
            .map(|e| (e.message, e.span.map_or(String::new(), |s| text[s.range()].to_string())))
            .collect()
    }

    #[test]
    fn errors_while_typing() {
        assert_eq!(messages("FD 10\nFOOBAR 3"), [("I don't know how to FOOBAR".into(), "FOOBAR".into())]);
        assert_eq!(messages("REPEAT 4 [FD]"), [("Not enough inputs to FD".into(), "FD".into())]);
        assert_eq!(messages("IF 1 < 2 [PRINT 1] IFELSE \"true [BOX] [FD 1]"), [("I don't know how to BOX".into(), "BOX".into())]);
        assert_eq!(messages("TO SPIN\n FOREVER [RT 1 REPEAT 2 [FD]]\nEND"), [("Not enough inputs to FD".into(), "FD".into())]);
        assert_eq!(messages("REPEAT 4 [BOX 10]\nTO BOX :n\nFD :n\nEND"), [], "definitions anywhere in the text count");
        // Lists that aren't instructions aren't checked.
        assert_eq!(messages("MAKE \"x [FD] SETPOS [1 2]"), []);
        assert_eq!(messages("PRINT [1 2"), [("This [ has no matching ]".into(), "[".into())]);
        // Phase 2's lists and templates.
        assert_eq!(messages("FOR [i 1 3] [FD]"), [("Not enough inputs to FD".into(), "FD".into())]);
        assert_eq!(messages("WHILE [FOO] [FD 1]"), [("I don't know how to FOO".into(), "FOO".into())]);
        assert_eq!(messages("SHOW MAP [SQRT] [1 2]"), [("Not enough inputs to SQRT".into(), "SQRT".into())]);
        assert_eq!(messages("FOREACH [1 2] [PRINT BAR ?]"), [("I don't know how to BAR".into(), "BAR".into())]);
        assert_eq!(messages("SHOW MAP [[x] :x * 2] [1] SHOW MAP \"FIRST [ab] CATCH \"error [FD 1]"), []);
        assert_eq!(messages("SHOW {1 2}@0 PRINT ?1"), []);
        assert!(check("").is_empty());
    }
}

#[cfg(test)]
mod timing {
    use super::*;

    /// Spike 01's generated 500-line program.
    fn big_program() -> String {
        let mut s = String::from("; A generated program for timing\n");
        let mut n = 0;
        while s.lines().count() < 500 {
            n += 1;
            s.push_str(&format!("TO SHAPE{n} :size\n  REPEAT {} [FD :size RT {}] ; side {n}\n  SETPC \"gold\nEND\n", 3 + n % 5, 360 / (3 + n % 5)));
        }
        s
    }

    #[test]
    fn fast_enough_to_run_on_every_keystroke() {
        let text = big_program();
        let started = std::time::Instant::now();
        let runs = 20;
        for _ in 0..runs {
            assert!(check(&text).is_empty());
            assert!(!highlight(&text).is_empty());
        }
        let each = started.elapsed() / runs;
        eprintln!("check and highlight a 500-line program: {each:?}");
        assert!(each < std::time::Duration::from_millis(30), "{each:?}");
    }
}
