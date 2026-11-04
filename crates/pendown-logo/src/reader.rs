//! Splits source text into words, lists and arrays, like UCBLogo's
//! `parser_iterate`. Only whitespace, brackets and braces end a word here;
//! parentheses and infix operators are split later, by `parse::runparse`, so
//! `[a+b]` is a list of one word, as in UCBLogo.

use std::rc::Rc;

use crate::{
    error::Error,
    span::{SourceId, Span},
    value::{Array, List, Value},
};

/// Reads the whole source as one list of items, each with its span.
/// Comments run from `;` to the end of the line. Arrays are typed as
/// `{a b c}`, with `{a b c}@0` for an origin other than 1.
pub fn read(text: &str, source: SourceId) -> Result<Rc<List>, Error> {
    struct Open {
        items: Vec<Value>,
        spans: Vec<Span>,
        start: Span,
        array: bool,
    }

    let span = |start: usize, end: usize, line: u32| Span {
        source,
        start: start as u32,
        end: end as u32,
        line,
    };
    let mut stack = vec![Open {
        items: vec![],
        spans: vec![],
        start: span(0, 0, 0),
        array: false,
    }];
    let mut line = 0;
    let mut word_start: Option<usize> = None;
    let bytes = text.as_bytes();
    let mut i = 0;

    while i <= bytes.len() {
        let c = bytes.get(i).copied();
        let ends_word = matches!(
            c,
            None | Some(b' ' | b'\t' | b'\r' | b'\n' | b'[' | b']' | b'{' | b'}' | b';')
        );
        if ends_word && let Some(start) = word_start.take() {
            let top = stack.last_mut().expect("the outer list is never popped");
            top.items.push(Value::word(&text[start..i]));
            top.spans.push(span(start, i, line));
        }
        match c {
            None => break,
            Some(b'\n') => line += 1,
            Some(b';') => {
                while i + 1 < bytes.len() && bytes[i + 1] != b'\n' {
                    i += 1;
                }
            }
            Some(open @ (b'[' | b'{')) => {
                stack.push(Open {
                    items: vec![],
                    spans: vec![],
                    start: span(i, i + 1, line),
                    array: open == b'{',
                });
            }
            Some(close @ (b']' | b'}')) => {
                let array = close == b'}';
                if stack.len() == 1 || stack.last().is_some_and(|top| top.array != array) {
                    let message = if array {
                        "This } has no matching {"
                    } else {
                        "This ] has no matching ["
                    };
                    return Err(Error::new(message, Some(span(i, i + 1, line))));
                }
                let open = stack.pop().expect("checked above");
                let mut end = i + 1;
                let value = if array {
                    // An origin straight after the brace: `}@0` or `}@-1`.
                    let mut origin = 1;
                    if bytes.get(end) == Some(&b'@') {
                        let digits = end + 1;
                        let mut j = digits + usize::from(bytes.get(digits) == Some(&b'-'));
                        while bytes.get(j).is_some_and(u8::is_ascii_digit) {
                            j += 1;
                        }
                        match text[digits..j].parse() {
                            Ok(n) => origin = n,
                            Err(_) => {
                                return Err(Error::new(
                                    "An array's origin must be a whole number, as in {a b c}@0",
                                    Some(span(i, j.max(end + 1), line)),
                                ));
                            }
                        }
                        end = j;
                    }
                    Value::Array(Rc::new(Array::new(open.items, origin)))
                } else {
                    Value::List(Rc::new(List::with_spans(open.items, open.spans)))
                };
                let top = stack.last_mut().expect("the outer list is never popped");
                top.items.push(value);
                top.spans.push(open.start.to(span(i, end, line)));
                i = end;
                continue;
            }
            Some(_) if !ends_word && word_start.is_none() => word_start = Some(i),
            Some(_) => {}
        }
        i += 1;
    }

    if stack.len() > 1 {
        let open = stack.pop().expect("checked above");
        let message = if open.array {
            "This { has no matching }"
        } else {
            "This [ has no matching ]"
        };
        return Err(Error::new(message, Some(open.start)));
    }
    let top = stack.pop().expect("the outer list");
    Ok(Rc::new(List::with_spans(top.items, top.spans)))
}
