use std::{
    cell::{OnceCell, RefCell},
    fmt,
    rc::Rc,
};

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
    /// Arrays are mutable: `SETITEM` changes the array wherever it's used.
    Array(Rc<Array>),
}

impl Value {
    /// Whether two values are certainly the same without looking inside:
    /// equal numbers, or the same word or list. An array can change in
    /// place, so it's never the same.
    pub fn same(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => a == b,
            (Value::Word(a), Value::Word(b)) => Rc::ptr_eq(a, b),
            (Value::List(a), Value::List(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

/// An array's members, numbered from `origin`.
pub struct Array {
    pub items: RefCell<Vec<Value>>,
    pub origin: i64,
}

impl Array {
    pub fn new(items: Vec<Value>, origin: i64) -> Array {
        Array { items: RefCell::new(items), origin }
    }

    /// The position in `items` of the member numbered `index`, if there is one.
    pub fn position(&self, index: i64) -> Option<usize> {
        let i = index.checked_sub(self.origin)?;
        (0..self.items.borrow().len() as i64).contains(&i).then_some(i as usize)
    }

    /// Whether `value` is this array or contains it, at any depth.
    pub fn is_in(self: &Rc<Array>, value: &Value) -> bool {
        match value {
            Value::Array(a) => Rc::ptr_eq(a, self) || a.items.borrow().iter().any(|v| self.is_in(v)),
            Value::List(l) => l.items.iter().any(|v| self.is_in(v)),
            _ => false,
        }
    }
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
            Value::List(_) | Value::Array(_) => None,
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
        self.show_styled(Style::default())
    }

    /// How `PRINT` prints it: a list loses its outer brackets.
    pub fn print_form(&self) -> String {
        self.print_styled(Style::default())
    }

    pub fn show_styled(&self, style: Style) -> String {
        let mut out = String::new();
        self.write(&mut out, true, style, 0);
        out
    }

    pub fn print_styled(&self, style: Style) -> String {
        let mut out = String::new();
        self.write(&mut out, false, style, 0);
        out
    }

    /// `depth` is how many lists or arrays this one is inside.
    fn write(&self, out: &mut String, brackets: bool, style: Style, depth: usize) {
        let items = |out: &mut String, items: &[Value]| {
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                if style.width.is_some_and(|width| i >= width) || style.length.is_some_and(|length| out.len() >= length) {
                    out.push_str("...");
                    break;
                }
                item.write(out, true, style, depth + 1);
            }
        };
        let too_deep = style.depth.is_some_and(|limit| depth >= limit);
        match self {
            Value::Num(n) => out.push_str(&format_number(*n)),
            Value::Word(w) if w.is_empty() && style.full => out.push_str("||"),
            Value::Word(w) => out.push_str(w),
            Value::List(_) | Value::Array(_) if too_deep => out.push_str("..."),
            Value::List(list) => {
                if brackets {
                    out.push('[');
                }
                items(out, &list.items);
                if brackets {
                    out.push(']');
                }
            }
            // Arrays keep their braces even at the top of `PRINT`.
            Value::Array(array) => {
                out.push('{');
                items(out, &array.items.borrow());
                out.push('}');
                if array.origin != 1 {
                    out.push_str(&format!("@{}", array.origin));
                }
            }
        }
    }
}

/// How `PRINT`, `TYPE` and `SHOW` print, from the special variables
/// `PRINTDEPTHLIMIT`, `PRINTWIDTHLIMIT` and `FULLPRINTP`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Style {
    /// Lists nested this deep print as `...`.
    pub depth: Option<usize>,
    /// Members of a list after this many print as `...`.
    pub width: Option<usize>,
    /// The empty word prints as `||`.
    pub full: bool,
    /// Stop with `...` once this many bytes are written, for the variables
    /// panel. `PRINT` has no such limit.
    pub length: Option<usize>,
}

/// `EQUALP`: numbers compare numerically, rounded to 15 significant digits,
/// so `2 = 2.0` and `0.1 + 0.2 = 0.3`; words ignore case if `ignore_case`,
/// which is `CASEIGNOREDP`; lists compare item by item; arrays are equal only
/// if they're the same array.
pub fn equal(a: &Value, b: &Value, ignore_case: bool) -> bool {
    if let (Some(x), Some(y)) = (a.as_num(), b.as_num()) {
        return compare(x, y).is_eq();
    }
    match (a, b) {
        (Value::List(x), Value::List(y)) => {
            x.items.len() == y.items.len()
                && x.items.iter().zip(&y.items).all(|(a, b)| equal(a, b, ignore_case))
        }
        (Value::Array(x), Value::Array(y)) => Rc::ptr_eq(x, y),
        (Value::List(_) | Value::Array(_), _) | (_, Value::List(_) | Value::Array(_)) => false,
        _ if ignore_case => a.print_form().to_lowercase() == b.print_form().to_lowercase(),
        _ => a.print_form() == b.print_form(),
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
