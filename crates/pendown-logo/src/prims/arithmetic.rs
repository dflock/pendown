//! Arithmetic, comparisons and logic. Numbers are `f64` (ADR 0002); results
//! that aren't finite are errors, and comparisons round to 15 significant
//! digits.

use std::cmp::Ordering;

use super::Args;
use crate::{
    interp::{Flow, Interpreter},
    number::{self, sin_cos_degrees},
    value::Value,
};

type Out = Result<Option<Value>, Flow>;

pub(super) fn sum(_: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    let mut total = 0.0;
    for i in 0..a.vals.len() {
        total += a.num(i)?;
    }
    a.result(total)
}

pub(super) fn difference(_: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    a.result(a.num(0)? - a.num(1)?)
}

pub(super) fn product(_: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    let mut total = 1.0;
    for i in 0..a.vals.len() {
        total *= a.num(i)?;
    }
    a.result(total)
}

/// With one input, `(QUOTIENT x)` is the reciprocal.
pub(super) fn quotient(_: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    let (x, y) = if a.vals.len() == 1 {
        (1.0, a.num(0)?)
    } else {
        (a.num(0)?, a.num(1)?)
    };
    if y == 0.0 {
        return Err(a.error("Can't divide by zero".into()));
    }
    a.result(x / y)
}

pub(super) fn minus(_: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    a.result(-a.num(0)?)
}

pub(super) fn equalp(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    Ok(Some(it.bool(it.equal(&a.vals[0], &a.vals[1]))))
}

pub(super) fn notequalp(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    Ok(Some(it.bool(!it.equal(&a.vals[0], &a.vals[1]))))
}

fn compare(
    it: &mut Interpreter,
    a: &Args,
    test: fn(Ordering) -> bool,
) -> Result<Option<Value>, Flow> {
    Ok(Some(it.bool(test(number::compare(a.num(0)?, a.num(1)?)))))
}

pub(super) fn lessp(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    compare(it, a, Ordering::is_lt)
}

pub(super) fn greaterp(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    compare(it, a, Ordering::is_gt)
}

pub(super) fn lessequalp(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    compare(it, a, Ordering::is_le)
}

pub(super) fn greaterequalp(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    compare(it, a, Ordering::is_ge)
}

/// Both inputs must be whole numbers, and the result has the sign of the
/// first, as C's `%` does.
pub(super) fn remainder(_: &mut Interpreter, a: &Args) -> Out {
    let (x, y) = (a.int(0)?, a.int(1)?);
    if y == 0 {
        return Err(a.error("Can't divide by zero".into()));
    }
    a.result((x % y) as f64)
}

/// Like `REMAINDER`, but the result has the sign of the second input.
pub(super) fn modulo(_: &mut Interpreter, a: &Args) -> Out {
    let (x, y) = (a.int(0)?, a.int(1)?);
    if y == 0 {
        return Err(a.error("Can't divide by zero".into()));
    }
    a.result((((x % y) + y) % y) as f64)
}

/// Drops the fractional part, towards zero.
pub(super) fn int(_: &mut Interpreter, a: &Args) -> Out {
    a.result(a.num(0)?.trunc())
}

/// The nearest whole number, with halves rounded away from zero.
pub(super) fn round(_: &mut Interpreter, a: &Args) -> Out {
    a.result(a.num(0)?.round())
}

pub(super) fn sqrt(_: &mut Interpreter, a: &Args) -> Out {
    let x = a.num(0)?;
    if x < 0.0 {
        return Err(a.doesnt_like(0));
    }
    a.result(x.sqrt())
}

/// A negative number can only be raised to a whole power.
pub(super) fn power(_: &mut Interpreter, a: &Args) -> Out {
    let (x, y) = (a.num(0)?, a.num(1)?);
    if x < 0.0 && y.fract() != 0.0 {
        return Err(a.doesnt_like(1));
    }
    a.result(x.powf(y))
}

/// In degrees, and exact at multiples of 90.
pub(super) fn sin(_: &mut Interpreter, a: &Args) -> Out {
    a.result(sin_cos_degrees(a.num(0)?).0)
}

pub(super) fn cos(_: &mut Interpreter, a: &Args) -> Out {
    a.result(sin_cos_degrees(a.num(0)?).1)
}

/// In degrees. `(ARCTAN x y)` is the angle of the point `x, y`, from -180 to
/// 180, which is 90 or -90 when `x` is 0.
pub(super) fn arctan(_: &mut Interpreter, a: &Args) -> Out {
    let degrees = if a.vals.len() == 2 {
        a.num(1)?.atan2(a.num(0)?)
    } else {
        a.num(0)?.atan()
    }
    .to_degrees();
    a.result(degrees)
}

/// `RANDOM n` is a whole number from 0 to n - 1. `(RANDOM start end)` is one
/// from `start` to `end`, inclusive.
pub(super) fn random(it: &mut Interpreter, a: &Args) -> Out {
    let (low, high) = if a.vals.len() == 2 {
        let (low, high) = (a.int(0)?, a.int(1)?);
        if high < low {
            return Err(a.doesnt_like(1));
        }
        (low, high)
    } else {
        let n = a.int(0)?;
        if n <= 0 {
            return Err(a.doesnt_like(0));
        }
        (0, n - 1)
    };
    let span = (high - low) as u64 + 1;
    a.result((low + it.random.below(span) as i64) as f64)
}

/// An input to `AND`, `OR` or `NOT`: `TRUE` or `FALSE`, or a list of
/// instructions that outputs one, run only when it's needed.
fn truth(it: &mut Interpreter, a: &Args, i: usize) -> Result<bool, Flow> {
    let value = match &a.vals[i] {
        Value::List(list) => it.run_expression(list, a.call)?,
        v => v.clone(),
    };
    value.as_bool().ok_or_else(|| a.doesnt_like(i))
}

/// Stops at the first `FALSE`, so later list inputs aren't run.
pub(super) fn and(it: &mut Interpreter, a: &Args) -> Out {
    for i in 0..a.vals.len() {
        if !truth(it, a, i)? {
            return Ok(Some(it.bool(false)));
        }
    }
    Ok(Some(it.bool(true)))
}

/// Stops at the first `TRUE`.
pub(super) fn or(it: &mut Interpreter, a: &Args) -> Out {
    for i in 0..a.vals.len() {
        if truth(it, a, i)? {
            return Ok(Some(it.bool(true)));
        }
    }
    Ok(Some(it.bool(false)))
}

pub(super) fn not(it: &mut Interpreter, a: &Args) -> Out {
    let b = truth(it, a, 0)?;
    Ok(Some(it.bool(!b)))
}

pub(super) fn exp(_: &mut Interpreter, a: &Args) -> Out {
    a.result(a.num(0)?.exp())
}

/// A logarithm of a number above zero.
fn log(a: &Args, f: fn(f64) -> f64) -> Out {
    let x = a.num(0)?;
    if x <= 0.0 {
        return Err(a.doesnt_like(0));
    }
    a.result(f(x))
}

pub(super) fn ln(_: &mut Interpreter, a: &Args) -> Out {
    log(a, f64::ln)
}

pub(super) fn log10(_: &mut Interpreter, a: &Args) -> Out {
    log(a, f64::log10)
}

pub(super) fn radsin(_: &mut Interpreter, a: &Args) -> Out {
    a.result(a.num(0)?.sin())
}

pub(super) fn radcos(_: &mut Interpreter, a: &Args) -> Out {
    a.result(a.num(0)?.cos())
}

/// Like `ARCTAN`, in radians.
pub(super) fn radarctan(_: &mut Interpreter, a: &Args) -> Out {
    a.result(if a.vals.len() == 2 {
        a.num(1)?.atan2(a.num(0)?)
    } else {
        a.num(0)?.atan()
    })
}

/// The most members `ISEQ`, `RSEQ` and `ARRAY` will make, so a typing
/// mistake like `ISEQ 1 1e12` is an error rather than running out of memory.
pub(super) const MAX_MEMBERS: i64 = 10_000_000;

pub(super) fn too_many(a: &Args) -> Flow {
    a.error(format!(
        "{} would make more than {MAX_MEMBERS} members",
        a.call.name.to_uppercase()
    ))
}

fn numbers(items: impl Iterator<Item = f64>) -> Out {
    Ok(Some(Value::List(std::rc::Rc::new(
        crate::value::List::new(items.map(Value::Num).collect()),
    ))))
}

/// `ISEQ from to`: the whole numbers from `from` to `to`, counting down if
/// `to` is smaller.
pub(super) fn iseq(_: &mut Interpreter, a: &Args) -> Out {
    let (from, to) = (a.int(0)?, a.int(1)?);
    if (to - from).abs() >= MAX_MEMBERS {
        return Err(too_many(a));
    }
    if from <= to {
        numbers((from..=to).map(|n| n as f64))
    } else {
        numbers((to..=from).rev().map(|n| n as f64))
    }
}

/// `RSEQ from to count`: `count` numbers evenly spaced from `from` to `to`.
pub(super) fn rseq(_: &mut Interpreter, a: &Args) -> Out {
    let (from, to, count) = (a.num(0)?, a.num(1)?, a.int(2)?);
    if count < 1 {
        return Err(a.doesnt_like(2));
    }
    if count > MAX_MEMBERS {
        return Err(too_many(a));
    }
    if count == 1 {
        return numbers(std::iter::once(from));
    }
    let step = (to - from) / (count - 1) as f64;
    numbers((0..count).map(|i| {
        if i == count - 1 {
            to
        } else {
            from + i as f64 * step
        }
    }))
}

/// `RERANDOM` starts `RANDOM`'s sequence again from the start, and
/// `(RERANDOM seed)` picks one of many repeatable sequences.
pub(super) fn rerandom(it: &mut Interpreter, a: &Args) -> Out {
    let seed = if a.vals.is_empty() { 0 } else { a.int(0)? };
    it.random = crate::random::Random::new(seed as u64);
    Ok(None)
}

/// `FORM num width precision`: `num` with exactly `precision` digits after
/// the point, padded with spaces on the left to at least `width`
/// characters. UCBLogo's printf formats, `(FORM num -1 format)`, aren't
/// supported.
pub(super) fn form(_: &mut Interpreter, a: &Args) -> Out {
    let x = a.num(0)?;
    let width = a.int(1)?;
    if width < 0 {
        return Err(
            a.error("FORM's printf formats aren't supported: give a width of 0 or more".into())
        );
    }
    let precision = a.int(2)?;
    if !(0..=100).contains(&precision) {
        return Err(a.doesnt_like(2));
    }
    if width > 1000 {
        return Err(a.doesnt_like(1));
    }
    let text = format!(
        "{x:>width$.precision$}",
        width = width as usize,
        precision = precision as usize
    );
    Ok(Some(Value::Word(text.into())))
}
