//! Templates and the higher-order commands: `APPLY`, `INVOKE`, `FOREACH`,
//! `MAP`, `MAP.SE`, `FILTER`, `FIND` and `REDUCE` (ADR 0021).
//!
//! A template is one of UCBLogo's four forms:
//! - a procedure's name, such as `"SUM`;
//! - an explicit-slot list, such as `[? * 2]` or `[?1 + ?2]`;
//! - a lambda, `[[x] :x * 2]`;
//! - procedure text, `[[x] [PRINT :x] [OUTPUT :x * 2]]`.
//!
//! It's prepared once for each call of a higher-order command, so a list
//! template is parsed once, not once for each member.

use std::rc::Rc;

use super::{Args, control::rest_of, data::members};
use crate::{
    interp::{Flow, Interpreter, Proc, Slots},
    names::Sym,
    parse::{Arity, Block, Target},
    value::{List, Value},
};

type Out = Result<Option<Value>, Flow>;

enum Template {
    Named { target: Target, name: Rc<str>, arity: Arity },
    Slots(Rc<Block>),
    Lambda { inputs: Vec<Sym>, body: Rc<Block> },
    Text(Rc<Proc>),
}

/// A data input's members, and whether it was a word, so `?REST` can be
/// put back together as the same kind of thing.
pub struct Data {
    members: Vec<Value>,
    word: bool,
}

impl Data {
    fn rest(&self, after: usize) -> Value {
        let rest = &self.members[after + 1..];
        if self.word {
            Value::Word(rest.iter().map(Value::print_form).collect::<String>().into())
        } else {
            Value::List(Rc::new(List::new(rest.to_vec())))
        }
    }
}

fn data(a: &Args, i: usize) -> Result<Data, Flow> {
    Ok(Data { members: members(a, i)?, word: !matches!(a.vals[i], Value::List(_)) })
}

fn prepare(it: &mut Interpreter, a: &Args, i: usize) -> Result<Template, Flow> {
    match &a.vals[i] {
        Value::List(list) => match list.items.first() {
            Some(Value::List(params)) => {
                let mut inputs = Vec::with_capacity(params.items.len());
                for p in &params.items {
                    match p {
                        Value::Word(w) => inputs.push(it.define(w.strip_prefix(':').unwrap_or(w))),
                        _ => return Err(a.doesnt_like(i)),
                    }
                }
                let rest = &list.items[1..];
                if !rest.is_empty() && rest.iter().all(|v| matches!(v, Value::List(_))) {
                    Ok(Template::Text(it.make_proc("template", inputs, rest, a.call.span)?))
                } else {
                    let body = it.parse_block(&rest_of(list, 1), a.call.span)?;
                    Ok(Template::Lambda { inputs, body })
                }
            }
            _ => Ok(Template::Slots(it.parse_block(list, a.call.span)?)),
        },
        Value::Array(_) => Err(a.doesnt_like(i)),
        _ => {
            let name = a.word(i)?;
            match it.procedure_named(&name) {
                Some((target, arity)) => Ok(Template::Named { target, name, arity }),
                None => Err(a.error(format!("I don't know how to {}", name.to_uppercase()))),
            }
        }
    }
}

fn check_inputs(a: &Args, n: usize, min: usize, max: Option<usize>, name: &str) -> Result<(), Flow> {
    if n < min {
        return Err(a.error(format!("Not enough inputs to {}", name.to_uppercase())));
    }
    if max.is_some_and(|max| n > max) {
        return Err(a.error(format!("Too many inputs to {}", name.to_uppercase())));
    }
    Ok(())
}

/// Runs a template with `values` in its slots. `data` and `index` are for
/// `?REST`: the data inputs, and which member is in the slots.
fn apply(it: &mut Interpreter, t: &Template, values: Vec<Value>, data: &Rc<Vec<Data>>, index: usize, a: &Args) -> Out {
    match t {
        Template::Named { target, name, arity } => {
            check_inputs(a, values.len(), arity.min, arity.max, name)?;
            it.call_with(*target, name.clone(), values, a.call.span)
        }
        Template::Slots(block) => {
            it.slots.push(Slots { values, data: data.clone(), index });
            let result = it.run_block_maybe(block);
            it.slots.pop();
            result
        }
        Template::Lambda { inputs, body } => {
            check_inputs(a, values.len(), inputs.len(), Some(inputs.len()), "the template")?;
            let body = body.clone();
            it.with_locals(inputs, values, &a.call.name, a.call.span, |it| it.run_block_maybe(&body))
        }
        Template::Text(proc) => {
            check_inputs(a, values.len(), proc.inputs.len(), Some(proc.inputs.len()), "the template")?;
            it.call_proc(proc.clone(), values, a.call.span)
        }
    }
}

/// Runs a template once with each member of the data inputs, with `#` and
/// `REPCOUNT` giving the position, and hands each result to `each`.
fn for_each_member(
    it: &mut Interpreter,
    a: &Args,
    template: usize,
    data_inputs: std::ops::Range<usize>,
    mut each: impl FnMut(&mut Interpreter, Option<Value>, &Value) -> Result<bool, Flow>,
) -> Result<(), Flow> {
    let t = prepare(it, a, template)?;
    let data: Vec<Data> = data_inputs.map(|i| data(a, i)).collect::<Result<_, _>>()?;
    let len = data[0].members.len();
    if data.iter().any(|d| d.members.len() != len) {
        return Err(a.error(format!("{}'s data inputs must be the same length", a.call.name.to_uppercase())));
    }
    let data = Rc::new(data);
    it.counting(&a.call.name, Some(len as i64), true, |it| {
        for i in 0..len {
            let values: Vec<Value> = data.iter().map(|d| d.members[i].clone()).collect();
            let first = values[0].clone();
            it.tick(a.call.span)?;
            it.set_count(i as i64 + 1);
            let result = apply(it, &t, values, &data, i, a);
            if !each(it, result?, &first)? {
                break;
            }
        }
        Ok(())
    })
}

/// What a template output, which it must have.
fn output_of(a: &Args, v: Option<Value>, template: usize) -> Result<Value, Flow> {
    v.ok_or_else(|| a.error(format!("{} didn't output to {}", a.vals[template].show(), a.call.name.to_uppercase())))
}

/// `TRUE` or `FALSE` from a template, for `FILTER` and `FIND`.
fn truth(a: &Args, v: Option<Value>) -> Result<bool, Flow> {
    let v = output_of(a, v, 0)?;
    v.as_bool().ok_or_else(|| a.error(format!("{} doesn't like {} from the template", a.call.name.to_uppercase(), v.show())))
}

fn no_data() -> Rc<Vec<Data>> {
    Rc::new(Vec::new())
}

/// `APPLY template inputlist`.
pub(super) fn apply_(it: &mut Interpreter, a: &Args) -> Out {
    let t = prepare(it, a, 0)?;
    let Value::List(inputs) = &a.vals[1] else { return Err(a.doesnt_like(1)) };
    apply(it, &t, inputs.items.clone(), &no_data(), 0, a)
}

/// `INVOKE template input`, or `(INVOKE template input1 input2 ...)`.
pub(super) fn invoke(it: &mut Interpreter, a: &Args) -> Out {
    let t = prepare(it, a, 0)?;
    apply(it, &t, a.vals[1..].to_vec(), &no_data(), 0, a)
}

/// `FOREACH data template`, or `(FOREACH data1 data2 ... template)`.
pub(super) fn foreach(it: &mut Interpreter, a: &Args) -> Out {
    let template = a.vals.len() - 1;
    for_each_member(it, a, template, 0..template, |_, v, _| match v {
        Some(v) => Err(a.error(format!("You don't say what to do with {}", v.show()))),
        None => Ok(true),
    })?;
    Ok(None)
}

/// `MAP template data`: a word if the data is a word, or a list.
pub(super) fn map(it: &mut Interpreter, a: &Args) -> Out {
    let mut out = Vec::new();
    for_each_member(it, a, 0, 1..a.vals.len(), |_, v, _| {
        out.push(output_of(a, v, 0)?);
        Ok(true)
    })?;
    if matches!(a.vals[1], Value::List(_)) {
        return Ok(Some(Value::List(Rc::new(List::new(out)))));
    }
    let mut word = String::new();
    for v in &out {
        match v {
            Value::List(_) | Value::Array(_) => return Err(a.error(format!("MAP can't put {} in a word", v.show()))),
            v => word.push_str(&v.print_form()),
        }
    }
    Ok(Some(Value::Word(word.into())))
}

/// `MAP.SE template data`: the results joined as by `SENTENCE`.
pub(super) fn map_se(it: &mut Interpreter, a: &Args) -> Out {
    let mut out = Vec::new();
    for_each_member(it, a, 0, 1..a.vals.len(), |_, v, _| {
        match output_of(a, v, 0)? {
            Value::List(l) => out.extend(l.items.iter().cloned()),
            v => out.push(v),
        }
        Ok(true)
    })?;
    Ok(Some(Value::List(Rc::new(List::new(out)))))
}

/// `FILTER tftemplate data`: the members for which the template outputs
/// `TRUE`.
pub(super) fn filter(it: &mut Interpreter, a: &Args) -> Out {
    let mut kept = Vec::new();
    for_each_member(it, a, 0, 1..2, |_, v, member| {
        if truth(a, v)? {
            kept.push(member.clone());
        }
        Ok(true)
    })?;
    if matches!(a.vals[1], Value::List(_)) {
        return Ok(Some(Value::List(Rc::new(List::new(kept)))));
    }
    Ok(Some(Value::Word(kept.iter().map(Value::print_form).collect::<String>().into())))
}

/// `FIND tftemplate data`: the first member for which the template outputs
/// `TRUE`, or `[]`.
pub(super) fn find(it: &mut Interpreter, a: &Args) -> Out {
    let mut found = None;
    for_each_member(it, a, 0, 1..2, |_, v, member| {
        if truth(a, v)? {
            found = Some(member.clone());
            return Ok(false);
        }
        Ok(true)
    })?;
    Ok(Some(found.unwrap_or_else(|| Value::List(Rc::new(List::new(Vec::new()))))))
}

/// `REDUCE template data`: from the right, so `REDUCE "DIFFERENCE [10 3 2]`
/// is `10 - (3 - 2)`.
pub(super) fn reduce(it: &mut Interpreter, a: &Args) -> Out {
    let t = prepare(it, a, 0)?;
    let m = members(a, 1)?;
    let Some((last, rest)) = m.split_last() else { return Err(a.doesnt_like(1)) };
    let mut acc = last.clone();
    for v in rest.iter().rev() {
        let result = apply(it, &t, vec![v.clone(), acc], &no_data(), 0, a)?;
        acc = output_of(a, result, 0)?;
    }
    Ok(Some(acc))
}

/// `?`, `?2` or `(? 2)`: what's in a slot of the template running.
pub(super) fn slot(it: &mut Interpreter, a: &Args) -> Out {
    let n = if a.vals.is_empty() { 1 } else { a.int(0)? };
    let Some(slots) = it.slots.last() else { return Err(a.error("? can only be used inside a template".into())) };
    match usize::try_from(n).ok().and_then(|n| n.checked_sub(1)).and_then(|i| slots.values.get(i)) {
        Some(v) => Ok(Some(v.clone())),
        None => Err(a.error(format!("This template has no ?{n}"))),
    }
}

/// `?REST` or `(?REST 2)`: the rest of a data input after the member in
/// the slot.
pub(super) fn slot_rest(it: &mut Interpreter, a: &Args) -> Out {
    let n = if a.vals.is_empty() { 1 } else { a.int(0)? };
    let Some(slots) = it.slots.last() else { return Err(a.error("?REST can only be used inside a template".into())) };
    match usize::try_from(n).ok().and_then(|n| n.checked_sub(1)).and_then(|i| slots.data.get(i)) {
        Some(d) => Ok(Some(d.rest(slots.index))),
        None => Err(a.error(format!("This template has no ?REST {n}"))),
    }
}
