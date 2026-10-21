//! The built-in procedures. A real version would take its names and arities
//! from `pendown-logo.kdl`.

use std::{cmp::Ordering, collections::HashMap, rc::Rc, sync::OnceLock};

use crate::{
    error::Error,
    event::{Event, Point},
    interp::{Flow, Interpreter},
    number,
    parse::{Arity, Call, Op},
    value::{Value, equal},
};

pub type PrimId = usize;

/// A primitive's own code: it gets its evaluated inputs, and outputs a value
/// or nothing.
pub type PrimFn = fn(&mut Interpreter, &Args) -> Result<Option<Value>, Flow>;

pub struct Prim {
    /// The name the manual uses first, then its aliases.
    pub names: &'static [&'static str],
    pub arity: Arity,
    pub imp: Imp,
}

/// Control structures are run by the evaluator itself, because they run
/// instruction lists and take part in tail calls (ADR 0007).
#[derive(Clone, Copy)]
pub enum Imp {
    F(PrimFn),
    If,
    IfElse,
    Repeat,
    Forever,
    Stop,
    Output,
}

const fn some(min: usize, default: usize, max: Option<usize>) -> Arity {
    Arity { min, default, max }
}

const ONE: Arity = Arity::fixed(1);
const TWO: Arity = Arity::fixed(2);

pub static PRIMS: &[Prim] = &[
    Prim { names: &["forward", "fd"], arity: ONE, imp: Imp::F(forward) },
    Prim { names: &["back", "bk"], arity: ONE, imp: Imp::F(back) },
    Prim { names: &["right", "rt"], arity: ONE, imp: Imp::F(right) },
    Prim { names: &["left", "lt"], arity: ONE, imp: Imp::F(left) },
    Prim { names: &["heading"], arity: Arity::fixed(0), imp: Imp::F(heading) },
    Prim { names: &["repeat"], arity: TWO, imp: Imp::Repeat },
    Prim { names: &["repcount"], arity: Arity::fixed(0), imp: Imp::F(repcount) },
    Prim { names: &["forever"], arity: ONE, imp: Imp::Forever },
    Prim { names: &["if"], arity: some(2, 2, Some(3)), imp: Imp::If },
    Prim { names: &["ifelse"], arity: Arity::fixed(3), imp: Imp::IfElse },
    Prim { names: &["stop"], arity: Arity::fixed(0), imp: Imp::Stop },
    Prim { names: &["output", "op"], arity: ONE, imp: Imp::Output },
    Prim { names: &["make"], arity: TWO, imp: Imp::F(make) },
    Prim { names: &["thing"], arity: ONE, imp: Imp::F(thing) },
    Prim { names: &["print", "pr"], arity: some(0, 1, None), imp: Imp::F(print) },
    Prim { names: &["type"], arity: some(0, 1, None), imp: Imp::F(type_) },
    Prim { names: &["show"], arity: some(0, 1, None), imp: Imp::F(show) },
    Prim { names: &["word"], arity: some(0, 2, None), imp: Imp::F(word) },
    Prim { names: &["sum"], arity: some(0, 2, None), imp: Imp::F(sum) },
    Prim { names: &["difference"], arity: TWO, imp: Imp::F(difference) },
    Prim { names: &["product"], arity: some(0, 2, None), imp: Imp::F(product) },
    Prim { names: &["quotient"], arity: TWO, imp: Imp::F(quotient) },
    Prim { names: &["minus"], arity: ONE, imp: Imp::F(minus) },
    Prim { names: &["equalp", "equal?"], arity: TWO, imp: Imp::F(equalp) },
    Prim { names: &["notequalp", "notequal?"], arity: TWO, imp: Imp::F(notequalp) },
    Prim { names: &["lessp", "less?"], arity: TWO, imp: Imp::F(lessp) },
    Prim { names: &["greaterp", "greater?"], arity: TWO, imp: Imp::F(greaterp) },
    Prim { names: &["lessequalp", "lessequal?"], arity: TWO, imp: Imp::F(lessequalp) },
    Prim { names: &["greaterequalp", "greaterequal?"], arity: TWO, imp: Imp::F(greaterequalp) },
];

pub fn lookup(name: &str) -> Option<PrimId> {
    static MAP: OnceLock<HashMap<&'static str, PrimId>> = OnceLock::new();
    let map = MAP.get_or_init(|| {
        let mut map = HashMap::new();
        for (id, prim) in PRIMS.iter().enumerate() {
            for name in prim.names {
                map.insert(*name, id);
            }
        }
        map
    });
    map.get(name.to_lowercase().as_str()).copied()
}

/// The primitive an infix operator stands for.
pub fn infix_prim(op: Op) -> &'static Prim {
    let name = match op {
        Op::Sum => "sum",
        Op::Difference => "difference",
        Op::Product => "product",
        Op::Quotient => "quotient",
        Op::Equal => "equalp",
        Op::Less => "lessp",
        Op::Greater => "greaterp",
        Op::LessEqual => "lessequalp",
        Op::GreaterEqual => "greaterequalp",
        Op::NotEqual => "notequalp",
    };
    &PRIMS[lookup(name).expect("every operator has a primitive")]
}

/// A primitive's evaluated inputs, with the call they came from, for errors.
pub struct Args<'a> {
    pub call: &'a Call,
    pub vals: &'a [Value],
}

impl Args<'_> {
    pub fn error(&self, message: String) -> Flow {
        Flow::Error(Box::new(Error::new(message, Some(self.call.span))))
    }

    pub fn doesnt_like(&self, i: usize) -> Flow {
        let message = format!("{} doesn't like {} as input", self.call.name.to_uppercase(), self.vals[i].show());
        Flow::Error(Box::new(Error::new(message, Some(self.call.args[i].span()))))
    }

    pub fn num(&self, i: usize) -> Result<f64, Flow> {
        self.vals[i].as_num().ok_or_else(|| self.doesnt_like(i))
    }

    /// A number with no fractional part (ADR 0002).
    pub fn int(&self, i: usize) -> Result<i64, Flow> {
        let n = self.num(i)?;
        if n.fract() == 0.0 && n.abs() < 2f64.powi(53) { Ok(n as i64) } else { Err(self.doesnt_like(i)) }
    }

    pub fn word(&self, i: usize) -> Result<Rc<str>, Flow> {
        match &self.vals[i] {
            Value::Word(w) => Ok(w.clone()),
            Value::Num(n) => Ok(crate::number::format_number(*n).into()),
            Value::List(_) => Err(self.doesnt_like(i)),
        }
    }

    /// Arithmetic results must be finite (ADR 0002).
    fn result(&self, x: f64) -> Result<Option<Value>, Flow> {
        if x.is_finite() { Ok(Some(Value::Num(x))) } else { Err(self.error("That number is too big".into())) }
    }
}

fn walk(it: &mut Interpreter, distance: f64) -> Result<Option<Value>, Flow> {
    let t = &mut it.turtle;
    let from = t.pos;
    let radians = t.heading.to_radians();
    t.pos = Point { x: from.x + distance * radians.sin(), y: from.y + distance * radians.cos() };
    let event = Event::Move { from, to: t.pos, pen: t.pen };
    it.emit(event)?;
    Ok(None)
}

fn turn(it: &mut Interpreter, by: f64) -> Result<Option<Value>, Flow> {
    let from = it.turtle.heading;
    it.turtle.heading = (from + by).rem_euclid(360.0);
    it.emit(Event::Turn { from, by })?;
    Ok(None)
}

fn forward(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    walk(it, a.num(0)?)
}

fn back(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    walk(it, -a.num(0)?)
}

fn right(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    turn(it, a.num(0)?)
}

fn left(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    turn(it, -a.num(0)?)
}

fn heading(it: &mut Interpreter, _: &Args) -> Result<Option<Value>, Flow> {
    Ok(Some(Value::Num(it.turtle.heading)))
}

fn repcount(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    match it.repcounts.last() {
        Some(&n) => Ok(Some(Value::Num(n as f64))),
        None => Err(a.error("REPCOUNT can only be used inside REPEAT or FOREVER".into())),
    }
}

fn make(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    let name = a.word(0)?;
    let sym = it.intern(&name);
    it.vars[sym as usize] = Some(a.vals[1].clone());
    Ok(None)
}

fn thing(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    let name = a.word(0)?;
    let sym = it.intern(&name);
    match &it.vars[sym as usize] {
        Some(v) => Ok(Some(v.clone())),
        None => Err(a.error(format!("{name} has no value"))),
    }
}

fn joined(a: &Args, form: fn(&Value) -> String) -> String {
    a.vals.iter().map(form).collect::<Vec<_>>().join(" ")
}

fn print(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    it.emit(Event::Print(joined(a, Value::print_form) + "\n"))?;
    Ok(None)
}

fn type_(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    it.emit(Event::Print(a.vals.iter().map(Value::print_form).collect()))?;
    Ok(None)
}

fn show(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    it.emit(Event::Print(joined(a, Value::show) + "\n"))?;
    Ok(None)
}

fn word(_: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    let mut out = String::new();
    for i in 0..a.vals.len() {
        out.push_str(&a.word(i)?);
    }
    Ok(Some(Value::Word(out.into())))
}

fn sum(_: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    let mut total = 0.0;
    for i in 0..a.vals.len() {
        total += a.num(i)?;
    }
    a.result(total)
}

fn difference(_: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    a.result(a.num(0)? - a.num(1)?)
}

fn product(_: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    let mut total = 1.0;
    for i in 0..a.vals.len() {
        total *= a.num(i)?;
    }
    a.result(total)
}

fn quotient(_: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    let (x, y) = (a.num(0)?, a.num(1)?);
    if y == 0.0 {
        return Err(a.error("Can't divide by zero".into()));
    }
    a.result(x / y)
}

fn minus(_: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    a.result(-a.num(0)?)
}

fn equalp(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    Ok(Some(it.bool(equal(&a.vals[0], &a.vals[1]))))
}

fn notequalp(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    Ok(Some(it.bool(!equal(&a.vals[0], &a.vals[1]))))
}

fn compare(it: &mut Interpreter, a: &Args, test: fn(Ordering) -> bool) -> Result<Option<Value>, Flow> {
    Ok(Some(it.bool(test(number::compare(a.num(0)?, a.num(1)?)))))
}

fn lessp(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    compare(it, a, Ordering::is_lt)
}

fn greaterp(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    compare(it, a, Ordering::is_gt)
}

fn lessequalp(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    compare(it, a, Ordering::is_le)
}

fn greaterequalp(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    compare(it, a, Ordering::is_ge)
}
