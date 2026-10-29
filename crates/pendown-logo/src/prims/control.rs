//! `RUN`, the loops, `CASE` and `COND`, and `CATCH` and `THROW` (ADR 0021).
//! They're ordinary built-in commands that run lists through the evaluator,
//! so `STOP` and `OUTPUT` inside them leave the procedure they're in, as in
//! UCBLogo.

use std::rc::Rc;

use super::Args;
use crate::{
    interp::{Flow, Interpreter},
    span::Span,
    value::{List, Value},
};

type Out = Result<Option<Value>, Flow>;

impl Args<'_> {
    /// A list to run, or a word, which runs as a list of one word.
    pub(super) fn runnable(&self, i: usize) -> Result<Rc<List>, Flow> {
        match &self.vals[i] {
            Value::List(list) => Ok(list.clone()),
            Value::Array(_) => Err(self.doesnt_like(i)),
            word => Ok(Rc::new(List::new(vec![word.clone()]))),
        }
    }

    pub(super) fn list(&self, i: usize) -> Result<Rc<List>, Flow> {
        match &self.vals[i] {
            Value::List(list) => Ok(list.clone()),
            _ => Err(self.doesnt_like(i)),
        }
    }
}

/// The members of a list from `from` on, keeping their spans, so errors in
/// them point at the code.
pub(super) fn rest_of(list: &List, from: usize) -> List {
    let items = list.items[from..].to_vec();
    match &list.spans {
        Some(spans) => List::with_spans(items, spans[from..].to_vec()),
        None => List::new(items),
    }
}

/// `RUN list`: outputs if the list's last instruction does.
pub(super) fn run(it: &mut Interpreter, a: &Args) -> Out {
    let list = a.runnable(0)?;
    it.run_maybe(&list, a.call.span)
}

/// `RUNRESULT list`: `[]`, or a list of what the list output.
pub(super) fn runresult(it: &mut Interpreter, a: &Args) -> Out {
    let list = a.runnable(0)?;
    let items = it.run_maybe(&list, a.call.span)?.into_iter().collect();
    Ok(Some(Value::List(Rc::new(List::new(items)))))
}

pub(super) fn ignore(_: &mut Interpreter, _: &Args) -> Out {
    Ok(None)
}

/// Runs input `i`, which must output `TRUE` or `FALSE`, as the loops'
/// conditions do.
fn condition(it: &mut Interpreter, a: &Args, i: usize) -> Result<bool, Flow> {
    let list = a.runnable(i)?;
    match it.run_maybe(&list, a.call.span)? {
        Some(v) => v.as_bool().ok_or_else(|| {
            a.error(format!(
                "{} doesn't like {} as a condition",
                a.call.name.to_uppercase(),
                v.show()
            ))
        }),
        None => Err(a.error(format!(
            "{} didn't output TRUE or FALSE to {}",
            a.vals[i].show(),
            a.call.name.to_uppercase()
        ))),
    }
}

/// `WHILE`, `UNTIL`, `DO.WHILE` and `DO.UNTIL`: the condition is input
/// `test`, the body input `body`, and the loop goes on while the condition
/// is `go_on`.
fn while_loop(
    it: &mut Interpreter,
    a: &Args,
    test: usize,
    body: usize,
    go_on: bool,
    body_first: bool,
) -> Out {
    let list = a.list(body)?;
    if body_first {
        it.run_commands(&list, a.call.span)?;
    }
    while condition(it, a, test)? == go_on {
        it.tick(a.call.span)?;
        it.run_commands(&list, a.call.span)?;
    }
    Ok(None)
}

pub(super) fn while_(it: &mut Interpreter, a: &Args) -> Out {
    while_loop(it, a, 0, 1, true, false)
}

pub(super) fn until(it: &mut Interpreter, a: &Args) -> Out {
    while_loop(it, a, 0, 1, false, false)
}

pub(super) fn do_while(it: &mut Interpreter, a: &Args) -> Out {
    while_loop(it, a, 1, 0, true, true)
}

pub(super) fn do_until(it: &mut Interpreter, a: &Args) -> Out {
    while_loop(it, a, 1, 0, false, true)
}

/// -1, 0 or 1.
fn sign(x: f64) -> i8 {
    if x > 0.0 {
        1
    } else if x < 0.0 {
        -1
    } else {
        0
    }
}

/// `FOR [var start limit step] list`. The start, limit and step are each
/// worked out as by `RUN`, so they can be expressions such as `:n*2`. The
/// variable is local to the `FOR`, and the next value is worked out from
/// its value after the body runs.
pub(super) fn for_(it: &mut Interpreter, a: &Args) -> Out {
    let control = a.list(0)?;
    let body = a.list(1)?;
    let (3 | 4) = control.items.len() else {
        return Err(a.doesnt_like(0));
    };
    let name = match &control.items[0] {
        Value::Word(w) => w.clone(),
        _ => return Err(a.doesnt_like(0)),
    };
    let number = |it: &mut Interpreter, i: usize| -> Result<f64, Flow> {
        let member = &control.items[i];
        if let Some(n) = member.as_num() {
            return Ok(n);
        }
        let list = match member {
            Value::List(inner) => inner.clone(),
            _ => {
                let mut one = rest_of(&control, i);
                one.items.truncate(1);
                if let Some(spans) = &mut one.spans {
                    spans.truncate(1);
                }
                Rc::new(one)
            }
        };
        match it
            .run_maybe(&list, a.call.span)?
            .as_ref()
            .and_then(Value::as_num)
        {
            Some(n) => Ok(n),
            None => Err(a.doesnt_like(0)),
        }
    };
    let start = number(it, 1)?;
    let limit = number(it, 2)?;
    let step = if control.items.len() == 4 {
        number(it, 3)?
    } else if limit >= start {
        1.0
    } else {
        -1.0
    };
    let sym = it.define(&name);
    // How many passes there will be, unless the body changes the variable.
    let passes = (step != 0.0).then(|| ((limit - start) / step).floor().max(-1.0) as i64 + 1);
    it.with_locals(
        &[sym],
        vec![Value::Num(start)],
        &a.call.name,
        a.call.span,
        |it| {
            it.counting(&a.call.name, passes, false, |it| {
                let mut current = start;
                let mut pass = 0;
                while sign(current - limit) != sign(step) {
                    it.tick(a.call.span)?;
                    pass += 1;
                    it.set_count(pass);
                    it.vars[sym as usize] = Some(Value::Num(current));
                    it.run_commands(&body, a.call.span)?;
                    let now = it.vars[sym as usize]
                        .as_ref()
                        .and_then(Value::as_num)
                        .unwrap_or(current);
                    current = now + step;
                }
                Ok(None)
            })
        },
    )
}

fn is_else(v: &Value) -> bool {
    matches!(v, Value::Word(w) if w.eq_ignore_ascii_case("else"))
}

/// The clauses of `CASE` or `COND`: a list of non-empty lists.
fn clauses(a: &Args, i: usize) -> Result<Vec<Rc<List>>, Flow> {
    let list = a.list(i)?;
    list.items
        .iter()
        .map(|c| match c {
            Value::List(clause) if !clause.items.is_empty() => Ok(clause.clone()),
            _ => Err(a.doesnt_like(i)),
        })
        .collect()
}

/// `CASE value clauses`: runs the rest of the first clause whose list of
/// values includes `value`, or that starts with `ELSE`.
pub(super) fn case(it: &mut Interpreter, a: &Args) -> Out {
    for clause in clauses(a, 1)? {
        let chosen = match &clause.items[0] {
            v if is_else(v) => true,
            Value::List(values) => values.items.iter().any(|v| it.equal(v, &a.vals[0])),
            _ => return Err(a.doesnt_like(1)),
        };
        if chosen {
            return it.run_maybe(&rest_of(&clause, 1), a.call.span);
        }
    }
    Ok(None)
}

/// `COND clauses`: runs the rest of the first clause whose condition is
/// `TRUE`, or that starts with `ELSE`.
pub(super) fn cond(it: &mut Interpreter, a: &Args) -> Out {
    for clause in clauses(a, 0)? {
        let chosen = match &clause.items[0] {
            v if is_else(v) => true,
            Value::List(test) => match it.run_maybe(test, a.call.span)? {
                Some(v) => v.as_bool().ok_or_else(|| {
                    a.error(format!("COND doesn't like {} as a condition", v.show()))
                })?,
                None => {
                    return Err(a.error(format!(
                        "{} didn't output TRUE or FALSE to COND",
                        clause.items[0].show()
                    )));
                }
            },
            _ => return Err(a.doesnt_like(0)),
        };
        if chosen {
            return it.run_maybe(&rest_of(&clause, 1), a.call.span);
        }
    }
    Ok(None)
}

/// `ERROR`'s list for an error: `[code message procedure line]`. Pendown's
/// errors aren't numbered, so the code is always 0, and the line is a line
/// number, or `[]` when there isn't one.
fn error_list(message: &str, procedure: Option<Rc<str>>, span: Option<Span>) -> Value {
    let empty = || Value::List(Rc::new(List::new(Vec::new())));
    let items = vec![
        Value::Num(0.0),
        Value::Word(message.into()),
        procedure.map_or_else(empty, Value::Word),
        span.map_or_else(empty, |s| Value::Num(s.line as f64 + 1.0)),
    ];
    Value::List(Rc::new(List::new(items)))
}

/// `CATCH tag list`. `CATCH "ERROR` catches errors as well as `THROW
/// "ERROR`, but never Stop.
pub(super) fn catch(it: &mut Interpreter, a: &Args) -> Out {
    let tag = a.word(0)?;
    let list = a.list(1)?;
    let errors = tag.eq_ignore_ascii_case("error");
    it.catching_errors += usize::from(errors);
    let result = it.run_maybe(&list, a.call.span);
    it.catching_errors -= usize::from(errors);
    match result {
        Err(Flow::Throw {
            tag: thrown,
            value,
            span,
        }) if thrown.eq_ignore_ascii_case(&tag) => {
            if !errors {
                return Ok(value);
            }
            // `THROW "ERROR` makes an error, so the CATCH doesn't output.
            let message = value.map_or_else(|| "THROW \"ERROR".to_string(), |v| v.print_form());
            it.caught = Some(error_list(&message, it.current_procedure(), Some(span)));
            Ok(None)
        }
        Err(Flow::Error(e)) if errors => {
            let procedure = e
                .calls
                .first()
                .map(|c| Rc::from(c.procedure.as_str()))
                .or_else(|| it.current_procedure());
            it.caught = Some(error_list(&e.message, procedure, e.span));
            Ok(None)
        }
        other => other,
    }
}

/// `THROW tag` or `(THROW tag value)`.
pub(super) fn throw(_: &mut Interpreter, a: &Args) -> Out {
    let tag = a.word(0)?;
    Err(Flow::Throw {
        tag,
        value: a.vals.get(1).cloned(),
        span: a.call.span,
    })
}

/// The error `CATCH "ERROR` caught last, which it then forgets, or `[]`.
pub(super) fn error(it: &mut Interpreter, _: &Args) -> Out {
    Ok(Some(it.caught.take().unwrap_or_else(|| {
        Value::List(Rc::new(List::new(Vec::new())))
    })))
}
