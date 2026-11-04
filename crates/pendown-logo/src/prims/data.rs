//! Words, lists and arrays. Numbers are words (ADR 0002), so `FIRST 345` is
//! `3`. Arrays are mutable and shared (ADR 0021): `SETITEM` changes the
//! array wherever it's used.

use std::rc::Rc;

use super::Args;
use crate::{
    interp::{Flow, Interpreter},
    number::format_number,
    value::{Array, List, Value},
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
        Value::List(_) | Value::Array(_) => None,
    }
}

/// The letters of a word, the members of a list, or an array.
enum Parts {
    Letters(Vec<char>),
    Items(Rc<List>),
    Array(Rc<Array>),
}

fn parts(v: &Value) -> Parts {
    match v {
        Value::List(list) => Parts::Items(list.clone()),
        Value::Array(array) => Parts::Array(array.clone()),
        v => Parts::Letters(text(v).expect("a word").chars().collect()),
    }
}

fn word(letters: &[char]) -> Value {
    Value::Word(letters.iter().collect::<String>().into())
}

/// The members of a word or list as values: a word's letters are one-letter
/// words. Arrays aren't accepted.
pub(super) fn members(a: &Args, i: usize) -> Result<Vec<Value>, Flow> {
    match parts(&a.vals[i]) {
        Parts::Letters(l) => Ok(l.iter().map(|c| word(&[*c])).collect()),
        Parts::Items(l) => Ok(l.items.clone()),
        Parts::Array(_) => Err(a.wanted(i, "a word or a list")),
    }
}

/// Puts members back together as the same kind of thing as input `i`: a list,
/// or a word if every member is a word.
fn rebuild(a: &Args, i: usize, members: Vec<Value>) -> Result<Value, Flow> {
    if matches!(a.vals[i], Value::List(_)) {
        return Ok(list(members));
    }
    let mut out = String::new();
    for m in &members {
        out.push_str(&text(m).ok_or_else(|| a.doesnt_like(i))?);
    }
    Ok(Value::Word(out.into()))
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

/// The first letter or member. For an array, the manual says it's the
/// array's origin, the index of its first member.
pub(super) fn first(_: &mut Interpreter, a: &Args) -> Out {
    match parts(&a.vals[0]) {
        Parts::Letters(l) if !l.is_empty() => Ok(Some(word(&l[..1]))),
        Parts::Items(l) if !l.items.is_empty() => Ok(Some(l.items[0].clone())),
        Parts::Array(array) => Ok(Some(Value::Num(array.origin as f64))),
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
        Parts::Items(l) if !l.items.is_empty() => {
            Ok(Some(list(l.items[..l.items.len() - 1].to_vec())))
        }
        _ => Err(a.doesnt_like(0)),
    }
}

/// `ITEM index thing`, counting from 1, or from an array's origin.
pub(super) fn item(_: &mut Interpreter, a: &Args) -> Out {
    let index = a.int(0)?;
    let at = |len: usize| (index >= 1 && index as usize <= len).then(|| index as usize - 1);
    match parts(&a.vals[1]) {
        Parts::Letters(l) => at(l.len()).map(|i| word(&l[i..=i])),
        Parts::Items(l) => at(l.items.len()).map(|i| l.items[i].clone()),
        Parts::Array(array) => array
            .position(index)
            .map(|i| array.items.borrow()[i].clone()),
    }
    .map(Some)
    .ok_or_else(|| a.doesnt_like(0))
}

pub(super) fn count(_: &mut Interpreter, a: &Args) -> Out {
    let n = match parts(&a.vals[0]) {
        Parts::Letters(l) => l.len(),
        Parts::Items(l) => l.items.len(),
        Parts::Array(array) => array.items.borrow().len(),
    };
    Ok(Some(Value::Num(n as f64)))
}

pub(super) fn emptyp(it: &mut Interpreter, a: &Args) -> Out {
    let empty = match parts(&a.vals[0]) {
        Parts::Letters(l) => l.is_empty(),
        Parts::Items(l) => l.items.is_empty(),
        Parts::Array(array) => array.items.borrow().is_empty(),
    };
    Ok(Some(it.bool(empty)))
}

// Constructors.

/// A one-letter word, for `FPUT` and `LPUT` onto a word.
fn letter(a: &Args, i: usize) -> Result<Rc<str>, Flow> {
    match text(&a.vals[i]) {
        Some(w) if w.chars().count() == 1 => Ok(w),
        _ => Err(a.doesnt_like(i)),
    }
}

pub(super) fn fput(_: &mut Interpreter, a: &Args) -> Out {
    match &a.vals[1] {
        Value::List(l) => {
            let mut items = Vec::with_capacity(l.items.len() + 1);
            items.push(a.vals[0].clone());
            items.extend(l.items.iter().cloned());
            Ok(Some(list(items)))
        }
        Value::Array(_) => Err(a.doesnt_like(1)),
        w => Ok(Some(Value::Word(
            format!("{}{}", letter(a, 0)?, text(w).expect("a word")).into(),
        ))),
    }
}

pub(super) fn lput(_: &mut Interpreter, a: &Args) -> Out {
    match &a.vals[1] {
        Value::List(l) => {
            let mut items = l.items.clone();
            items.push(a.vals[0].clone());
            Ok(Some(list(items)))
        }
        Value::Array(_) => Err(a.doesnt_like(1)),
        w => Ok(Some(Value::Word(
            format!("{}{}", text(w).expect("a word"), letter(a, 0)?).into(),
        ))),
    }
}

/// `WORD` onto a word, `FPUT` onto a list.
pub(super) fn combine(it: &mut Interpreter, a: &Args) -> Out {
    match &a.vals[1] {
        Value::List(_) => fput(it, a),
        Value::Array(_) => Err(a.doesnt_like(1)),
        w => Ok(Some(Value::Word(
            format!("{}{}", a.word(0)?, text(w).expect("a word")).into(),
        ))),
    }
}

/// A list's members, or a word's letters, in reverse order.
pub(super) fn reverse(_: &mut Interpreter, a: &Args) -> Out {
    let mut m = members(a, 0)?;
    m.reverse();
    Ok(Some(rebuild(a, 0, m)?))
}

// Selectors.

/// The `FIRST`, or `BUTFIRST`, of each member of a list.
fn each(it: &mut Interpreter, a: &Args, f: fn(&mut Interpreter, &Args) -> Out) -> Out {
    let Value::List(l) = &a.vals[0] else {
        return Err(a.doesnt_like(0));
    };
    let mut out = Vec::with_capacity(l.items.len());
    for item in &l.items {
        let one = [item.clone()];
        let inner = Args {
            call: a.call,
            vals: &one,
        };
        // An empty member: the error names the whole input.
        match f(it, &inner) {
            Ok(v) => out.push(v.expect("selectors output")),
            Err(_) => return Err(a.doesnt_like(0)),
        }
    }
    Ok(Some(list(out)))
}

pub(super) fn firsts(it: &mut Interpreter, a: &Args) -> Out {
    each(it, a, first)
}

pub(super) fn butfirsts(it: &mut Interpreter, a: &Args) -> Out {
    each(it, a, butfirst)
}

/// A member of a list, or a letter of a word, chosen at random.
pub(super) fn pick(it: &mut Interpreter, a: &Args) -> Out {
    let m = members(a, 0)?;
    if m.is_empty() {
        return Err(a.doesnt_like(0));
    }
    let i = it.random.below(m.len() as u64) as usize;
    Ok(Some(m[i].clone()))
}

/// `REMOVE thing list`: the list without any members equal to `thing`.
pub(super) fn remove(it: &mut Interpreter, a: &Args) -> Out {
    let m = members(a, 1)?;
    let kept = m.into_iter().filter(|v| !it.equal(v, &a.vals[0])).collect();
    Ok(Some(rebuild(a, 1, kept)?))
}

/// Without duplicates: of equal members, the rightmost stays.
pub(super) fn remdup(it: &mut Interpreter, a: &Args) -> Out {
    let m = members(a, 0)?;
    let mut kept = Vec::with_capacity(m.len());
    for (i, v) in m.iter().enumerate() {
        if !m[i + 1..].iter().any(|later| it.equal(v, later)) {
            kept.push(v.clone());
        }
    }
    Ok(Some(rebuild(a, 0, kept)?))
}

/// A word with a quotation mark before it; a list as it is.
pub(super) fn quoted(_: &mut Interpreter, a: &Args) -> Out {
    match &a.vals[0] {
        Value::List(_) | Value::Array(_) => Ok(Some(a.vals[0].clone())),
        w => Ok(Some(Value::Word(
            format!("\"{}", text(w).expect("a word")).into(),
        ))),
    }
}

// Arrays.

impl Args<'_> {
    /// An origin: input `i` if there is one, or 1.
    fn origin(&self, i: usize) -> Result<i64, Flow> {
        if self.vals.len() > i {
            self.int(i)
        } else {
            Ok(1)
        }
    }

    fn array(&self, i: usize) -> Result<Rc<Array>, Flow> {
        match &self.vals[i] {
            Value::Array(array) => Ok(array.clone()),
            _ => Err(self.doesnt_like(i)),
        }
    }

    /// A list of whole numbers, as `MDARRAY` and `MDITEM` take.
    fn ints(&self, i: usize) -> Result<Vec<i64>, Flow> {
        let Value::List(l) = &self.vals[i] else {
            return Err(self.doesnt_like(i));
        };
        let ints: Option<Vec<i64>> = l
            .items
            .iter()
            .map(|v| {
                v.as_num()
                    .filter(|n| n.fract() == 0.0 && n.abs() < 2f64.powi(53))
                    .map(|n| n as i64)
            })
            .collect();
        ints.filter(|ints| !ints.is_empty())
            .ok_or_else(|| self.doesnt_like(i))
    }
}

fn new_array(size: i64, origin: i64) -> Value {
    Value::Array(Rc::new(Array::new(
        vec![list(Vec::new()); size as usize],
        origin,
    )))
}

/// `ARRAY size`, or `(ARRAY size origin)`: each member starts as `[]`.
pub(super) fn array(_: &mut Interpreter, a: &Args) -> Out {
    let size = a.int(0)?;
    if size < 1 {
        return Err(a.doesnt_like(0));
    }
    if size > super::arithmetic::MAX_MEMBERS {
        return Err(super::arithmetic::too_many(a));
    }
    Ok(Some(new_array(size, a.origin(1)?)))
}

/// An array of arrays, one level for each size.
pub(super) fn mdarray(_: &mut Interpreter, a: &Args) -> Out {
    let sizes = a.ints(0)?;
    if sizes.iter().any(|&s| s < 1) {
        return Err(a.doesnt_like(0));
    }
    if sizes
        .iter()
        .try_fold(1i64, |total, &s| total.checked_mul(s))
        .is_none_or(|total| total > super::arithmetic::MAX_MEMBERS)
    {
        return Err(super::arithmetic::too_many(a));
    }
    let origin = a.origin(1)?;
    fn build(sizes: &[i64], origin: i64) -> Value {
        match sizes {
            [size] => new_array(*size, origin),
            [size, rest @ ..] => {
                let items = (0..*size).map(|_| build(rest, origin)).collect();
                Value::Array(Rc::new(Array::new(items, origin)))
            }
            [] => unreachable!("checked not empty"),
        }
    }
    Ok(Some(build(&sizes, origin)))
}

pub(super) fn listtoarray(_: &mut Interpreter, a: &Args) -> Out {
    let Value::List(l) = &a.vals[0] else {
        return Err(a.doesnt_like(0));
    };
    Ok(Some(Value::Array(Rc::new(Array::new(
        l.items.clone(),
        a.origin(1)?,
    )))))
}

pub(super) fn arraytolist(_: &mut Interpreter, a: &Args) -> Out {
    Ok(Some(list(a.array(0)?.items.borrow().clone())))
}

/// Replaces a member of `array`, unless that would put the array inside
/// itself.
fn set_member(
    a: &Args,
    array: &Rc<Array>,
    index: i64,
    value: &Value,
    index_input: usize,
) -> Result<(), Flow> {
    let Some(i) = array.position(index) else {
        return Err(a.doesnt_like(index_input));
    };
    if array.is_in(value) {
        return Err(a.error(format!(
            "{} can't put an array inside itself",
            a.call.name.to_uppercase()
        )));
    }
    array.items.borrow_mut()[i] = value.clone();
    Ok(())
}

/// `SETITEM index array value`.
pub(super) fn setitem(_: &mut Interpreter, a: &Args) -> Out {
    let array = a.array(1)?;
    set_member(a, &array, a.int(0)?, &a.vals[2], 0)?;
    Ok(None)
}

/// Follows all but the last index of `indices` through nested arrays.
fn inner_array(a: &Args, indices: &[i64], array: Rc<Array>) -> Result<Rc<Array>, Flow> {
    let mut array = array;
    for &index in indices {
        let next = match array
            .position(index)
            .map(|i| array.items.borrow()[i].clone())
        {
            Some(Value::Array(inner)) => inner,
            _ => return Err(a.doesnt_like(0)),
        };
        array = next;
    }
    Ok(array)
}

/// `MDITEM [i j ...] array`.
pub(super) fn mditem(_: &mut Interpreter, a: &Args) -> Out {
    let indices = a.ints(0)?;
    let (last, path) = indices.split_last().expect("not empty");
    let array = inner_array(a, path, a.array(1)?)?;
    let item = array
        .position(*last)
        .map(|i| array.items.borrow()[i].clone());
    item.map(Some).ok_or_else(|| a.doesnt_like(0))
}

/// `MDSETITEM [i j ...] array value`.
pub(super) fn mdsetitem(_: &mut Interpreter, a: &Args) -> Out {
    let indices = a.ints(0)?;
    let (last, path) = indices.split_last().expect("not empty");
    let outer = a.array(1)?;
    let array = inner_array(a, path, outer.clone())?;
    if outer.is_in(&a.vals[2]) {
        return Err(a.error("MDSETITEM can't put an array inside itself".into()));
    }
    set_member(a, &array, *last, &a.vals[2], 0)?;
    Ok(None)
}

// Stacks and queues: the first input names a variable holding a list.

fn named_list(it: &mut Interpreter, a: &Args) -> Result<(crate::names::Sym, Rc<List>), Flow> {
    let name = a.word(0)?;
    let sym = it.intern(&name);
    match &it.vars[sym as usize] {
        Some(Value::List(l)) => Ok((sym, l.clone())),
        Some(v) => Err(a.error(format!(
            "{} doesn't like {} as the value of {name}",
            a.call.name.to_uppercase(),
            v.show()
        ))),
        None => Err(a.error(format!("{name} has no value"))),
    }
}

/// `PUSH "stack thing`: adds to the front.
pub(super) fn push(it: &mut Interpreter, a: &Args) -> Out {
    let (sym, l) = named_list(it, a)?;
    let mut items = Vec::with_capacity(l.items.len() + 1);
    items.push(a.vals[1].clone());
    items.extend(l.items.iter().cloned());
    it.vars[sym as usize] = Some(list(items));
    Ok(None)
}

/// `QUEUE "queue thing`: adds to the back.
pub(super) fn queue(it: &mut Interpreter, a: &Args) -> Out {
    let (sym, l) = named_list(it, a)?;
    let mut items = l.items.clone();
    items.push(a.vals[1].clone());
    it.vars[sym as usize] = Some(list(items));
    Ok(None)
}

/// `POP "stack` and `DEQUEUE "queue` both take from the front, since `PUSH`
/// adds there and `QUEUE` adds at the back.
pub(super) fn pop(it: &mut Interpreter, a: &Args) -> Out {
    let (sym, l) = named_list(it, a)?;
    let Some((first, rest)) = l.items.split_first() else {
        return Err(a.error(format!("{} is empty", a.vals[0].print_form())));
    };
    let first = first.clone();
    it.vars[sym as usize] = Some(list(rest.to_vec()));
    Ok(Some(first))
}

// Predicates.

pub(super) fn wordp(it: &mut Interpreter, a: &Args) -> Out {
    Ok(Some(it.bool(text(&a.vals[0]).is_some())))
}

pub(super) fn listp(it: &mut Interpreter, a: &Args) -> Out {
    Ok(Some(it.bool(matches!(a.vals[0], Value::List(_)))))
}

pub(super) fn arrayp(it: &mut Interpreter, a: &Args) -> Out {
    Ok(Some(it.bool(matches!(a.vals[0], Value::Array(_)))))
}

pub(super) fn numberp(it: &mut Interpreter, a: &Args) -> Out {
    Ok(Some(it.bool(a.vals[0].as_num().is_some())))
}

/// A word, lowercased if `CASEIGNOREDP` is true.
fn folded(it: &Interpreter, w: &str) -> String {
    if it.ignore_case() {
        w.to_lowercase()
    } else {
        w.to_string()
    }
}

/// `BEFOREP word1 word2`: in character order, so `BEFOREP 3 12` is false.
pub(super) fn beforep(it: &mut Interpreter, a: &Args) -> Out {
    let (x, y) = (a.word(0)?, a.word(1)?);
    Ok(Some(it.bool(folded(it, &x) < folded(it, &y))))
}

/// Where `thing` is in `within`, a list, array or word, if it's there. In a
/// word, `thing` must be one letter.
fn find_member(it: &Interpreter, thing: &Value, within: &Value) -> Option<usize> {
    match within {
        Value::List(l) => l.items.iter().position(|v| it.equal(thing, v)),
        Value::Array(array) => array.items.borrow().iter().position(|v| it.equal(thing, v)),
        w => {
            let letter = text(thing)?;
            if letter.chars().count() != 1 {
                return None;
            }
            let letter = folded(it, &letter);
            text(w)?
                .chars()
                .position(|c| folded(it, &c.to_string()) == letter)
        }
    }
}

pub(super) fn memberp(it: &mut Interpreter, a: &Args) -> Out {
    Ok(Some(
        it.bool(find_member(it, &a.vals[0], &a.vals[1]).is_some()),
    ))
}

/// `SUBSTRINGP thing1 thing2`: false for lists and arrays.
pub(super) fn substringp(it: &mut Interpreter, a: &Args) -> Out {
    let found = match (text(&a.vals[0]), text(&a.vals[1])) {
        (Some(part), Some(whole)) => folded(it, &whole).contains(&folded(it, &part)),
        _ => false,
    };
    Ok(Some(it.bool(found)))
}

// Queries.

/// The rest of `thing2` from the first `thing1` in it, or an empty word or
/// list.
pub(super) fn member(it: &mut Interpreter, a: &Args) -> Out {
    let found = find_member(it, &a.vals[0], &a.vals[1]);
    match &a.vals[1] {
        Value::Array(_) => Err(a.doesnt_like(1)),
        Value::List(l) => Ok(Some(list(
            found.map_or(Vec::new(), |i| l.items[i..].to_vec()),
        ))),
        w => {
            let letters: Vec<char> = text(w).expect("a word").chars().collect();
            Ok(Some(word(found.map_or(&[], |i| &letters[i..]))))
        }
    }
}

/// The character's Unicode code point, which is its ASCII code for ASCII.
pub(super) fn ascii(_: &mut Interpreter, a: &Args) -> Out {
    let w = a.word(0)?;
    let mut chars = w.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Ok(Some(Value::Num(c as u32 as f64))),
        _ => Err(a.doesnt_like(0)),
    }
}

pub(super) fn char(_: &mut Interpreter, a: &Args) -> Out {
    let n = a.int(0)?;
    match u32::try_from(n).ok().and_then(char::from_u32) {
        Some(c) => Ok(Some(word(&[c]))),
        None => Err(a.doesnt_like(0)),
    }
}

pub(super) fn lowercase(_: &mut Interpreter, a: &Args) -> Out {
    Ok(Some(Value::Word(a.word(0)?.to_lowercase().into())))
}

pub(super) fn uppercase(_: &mut Interpreter, a: &Args) -> Out {
    Ok(Some(Value::Word(a.word(0)?.to_uppercase().into())))
}
