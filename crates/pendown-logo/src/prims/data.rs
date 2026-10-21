//! Words and lists: the constructors and selectors drawing needs, mainly for
//! `POS` and `SETPOS`. Numbers are words (ADR 0002), so `FIRST 345` is `3`.

use std::rc::Rc;

use super::Args;
use crate::{
    interp::{Flow, Interpreter},
    number::format_number,
    value::{List, Value},
};

type Out = Result<Option<Value>, Flow>;

fn list(items: Vec<Value>) -> Value {
    Value::List(Rc::new(List::new(items)))
}

/// A word's text, whether it's held as a word or a number.
fn text(v: &Value) -> Option<Rc<str>> {
    match v {
        Value::Word(w) => Some(w.clone()),
        Value::Num(n) => Some(format_number(*n).into()),
        Value::List(_) => None,
    }
}

/// The letters of a word, or the members of a list.
enum Parts {
    Letters(Vec<char>),
    Items(Rc<List>),
}

fn parts(v: &Value) -> Parts {
    match v {
        Value::List(list) => Parts::Items(list.clone()),
        v => Parts::Letters(text(v).expect("not a list").chars().collect()),
    }
}

fn word(letters: &[char]) -> Value {
    Value::Word(letters.iter().collect::<String>().into())
}

/// `LIST a b`, or `(LIST a b c ...)`: a list of its inputs.
pub(super) fn list_(_: &mut Interpreter, a: &Args) -> Out {
    Ok(Some(list(a.vals.to_vec())))
}

/// `SENTENCE a b`: like `LIST`, but lists are joined rather than nested.
pub(super) fn sentence(_: &mut Interpreter, a: &Args) -> Out {
    let mut items = Vec::new();
    for v in a.vals {
        match v {
            Value::List(l) => items.extend(l.items.iter().cloned()),
            v => items.push(v.clone()),
        }
    }
    Ok(Some(list(items)))
}

pub(super) fn first(_: &mut Interpreter, a: &Args) -> Out {
    match parts(&a.vals[0]) {
        Parts::Letters(l) if !l.is_empty() => Ok(Some(word(&l[..1]))),
        Parts::Items(l) if !l.items.is_empty() => Ok(Some(l.items[0].clone())),
        _ => Err(a.doesnt_like(0)),
    }
}

pub(super) fn last(_: &mut Interpreter, a: &Args) -> Out {
    match parts(&a.vals[0]) {
        Parts::Letters(l) if !l.is_empty() => Ok(Some(word(&l[l.len() - 1..]))),
        Parts::Items(l) if !l.items.is_empty() => Ok(Some(l.items[l.items.len() - 1].clone())),
        _ => Err(a.doesnt_like(0)),
    }
}

pub(super) fn butfirst(_: &mut Interpreter, a: &Args) -> Out {
    match parts(&a.vals[0]) {
        Parts::Letters(l) if !l.is_empty() => Ok(Some(word(&l[1..]))),
        Parts::Items(l) if !l.items.is_empty() => Ok(Some(list(l.items[1..].to_vec()))),
        _ => Err(a.doesnt_like(0)),
    }
}

pub(super) fn butlast(_: &mut Interpreter, a: &Args) -> Out {
    match parts(&a.vals[0]) {
        Parts::Letters(l) if !l.is_empty() => Ok(Some(word(&l[..l.len() - 1]))),
        Parts::Items(l) if !l.items.is_empty() => Ok(Some(list(l.items[..l.items.len() - 1].to_vec()))),
        _ => Err(a.doesnt_like(0)),
    }
}

/// `ITEM index thing`, counting from 1.
pub(super) fn item(_: &mut Interpreter, a: &Args) -> Out {
    let index = a.int(0)?;
    let at = |len: usize| (index >= 1 && index as usize <= len).then(|| index as usize - 1);
    match parts(&a.vals[1]) {
        Parts::Letters(l) => at(l.len()).map(|i| word(&l[i..=i])),
        Parts::Items(l) => at(l.items.len()).map(|i| l.items[i].clone()),
    }
    .map(Some)
    .ok_or_else(|| a.doesnt_like(0))
}

pub(super) fn count(_: &mut Interpreter, a: &Args) -> Out {
    let n = match parts(&a.vals[0]) {
        Parts::Letters(l) => l.len(),
        Parts::Items(l) => l.items.len(),
    };
    Ok(Some(Value::Num(n as f64)))
}

pub(super) fn emptyp(it: &mut Interpreter, a: &Args) -> Out {
    let empty = match parts(&a.vals[0]) {
        Parts::Letters(l) => l.is_empty(),
        Parts::Items(l) => l.items.is_empty(),
    };
    Ok(Some(it.bool(empty)))
}
