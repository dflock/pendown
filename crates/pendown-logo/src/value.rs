use std::{cell::OnceCell, fmt, rc::Rc};

use crate::{
    number::{compare, format_number, parse_number},
    parse::Block,
    span::Span,
};

/// A Logo value. Numbers are words (ADR 0002): a `Word` that looks like a
/// number can be used as one, and a `Num` can be used as a word. A number
/// literal is stored as `Num` only when it's typed in its printed form, so
/// `1.50` stays the word `1.50`, but `1.50 + 0` is `1.5`.
#[derive(Clone)]
pub enum Value {
    Num(f64),
    Word(Rc<str>),
    List(Rc<List>),
}

pub struct List {
    pub items: Vec<Value>,
    /// The span of each item, for lists written in the source. Lists built
    /// while the program runs have none.
    pub spans: Option<Vec<Span>>,
    /// The list parsed as instructions, the first time it's run (ADR 0001).
    pub(crate) parsed: OnceCell<Rc<Block>>,
}

impl List {
    pub fn new(items: Vec<Value>) -> List {
        List { items, spans: None, parsed: OnceCell::new() }
    }

    pub(crate) fn with_spans(items: Vec<Value>, spans: Vec<Span>) -> List {
        List { items, spans: Some(spans), parsed: OnceCell::new() }
    }
}

impl Value {
    pub fn word(text: &str) -> Value {
        Value::Word(text.into())
    }

    /// The value as a number, if it is one or is a word that looks like one.
    pub fn as_num(&self) -> Option<f64> {
        match self {
            Value::Num(n) => Some(*n),
            Value::Word(w) => parse_number(w),
            Value::List(_) => None,
        }
    }

    /// `TRUE` or `FALSE`, in any case.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Word(w) if w.eq_ignore_ascii_case("true") => Some(true),
            Value::Word(w) if w.eq_ignore_ascii_case("false") => Some(false),
            _ => None,
        }
    }

    /// How `SHOW` prints it: lists keep their brackets.
    pub fn show(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, true);
        out
    }

    /// How `PRINT` prints it: a list loses its outer brackets.
    pub fn print_form(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, false);
        out
    }

    fn write(&self, out: &mut String, brackets: bool) {
        match self {
            Value::Num(n) => out.push_str(&format_number(*n)),
            Value::Word(w) => out.push_str(w),
            Value::List(list) => {
                if brackets {
                    out.push('[');
                }
                for (i, item) in list.items.iter().enumerate() {
                    if i > 0 {
                        out.push(' ');
                    }
                    item.write(out, true);
                }
                if brackets {
                    out.push(']');
                }
            }
        }
    }
}

/// `EQUALP`: numbers compare numerically, rounded to 15 significant digits,
/// so `2 = 2.0` and `0.1 + 0.2 = 0.3`; words ignore case
/// (`CASEIGNOREDP` is always true until phase 2); lists compare item by item.
pub fn equal(a: &Value, b: &Value) -> bool {
    if let (Some(x), Some(y)) = (a.as_num(), b.as_num()) {
        return compare(x, y).is_eq();
    }
    match (a, b) {
        (Value::List(x), Value::List(y)) => {
            x.items.len() == y.items.len()
                && x.items.iter().zip(&y.items).all(|(a, b)| equal(a, b))
        }
        (Value::List(_), _) | (_, Value::List(_)) => false,
        _ => a.print_form().to_lowercase() == b.print_form().to_lowercase(),
    }
}

impl fmt::Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(&self.items).finish()
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.show())
    }
}
