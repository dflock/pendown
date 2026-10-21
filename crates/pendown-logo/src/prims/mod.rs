//! The built-in procedures. The table here gives each one's names, inputs
//! and code; the code is in a module per area, as `pendown-logo.kdl` groups
//! them.

mod arithmetic;
mod data;
mod pen;
mod shapes;
mod turtle;

use std::{collections::HashMap, rc::Rc, sync::OnceLock};

use arithmetic::*;
use data::*;
use pen::*;
use shapes::*;
use turtle::*;

use crate::{
    error::Error,
    event::{Event, Point},
    interp::{Flow, Interpreter},
    parse::{Arity, Call, Op},
    value::Value,
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

const NONE: Arity = Arity::fixed(0);
const ONE: Arity = Arity::fixed(1);
const TWO: Arity = Arity::fixed(2);

pub static PRIMS: &[Prim] = &[
    // Turtle motion and queries.
    Prim { names: &["forward", "fd"], arity: ONE, imp: Imp::F(forward) },
    Prim { names: &["back", "bk"], arity: ONE, imp: Imp::F(back) },
    Prim { names: &["right", "rt"], arity: ONE, imp: Imp::F(right) },
    Prim { names: &["left", "lt"], arity: ONE, imp: Imp::F(left) },
    Prim { names: &["setpos"], arity: ONE, imp: Imp::F(setpos) },
    Prim { names: &["setxy"], arity: TWO, imp: Imp::F(setxy) },
    Prim { names: &["setx"], arity: ONE, imp: Imp::F(setx) },
    Prim { names: &["sety"], arity: ONE, imp: Imp::F(sety) },
    Prim { names: &["setheading", "seth"], arity: ONE, imp: Imp::F(setheading) },
    Prim { names: &["home"], arity: NONE, imp: Imp::F(home) },
    Prim { names: &["pos"], arity: NONE, imp: Imp::F(pos) },
    Prim { names: &["xcor"], arity: NONE, imp: Imp::F(xcor) },
    Prim { names: &["ycor"], arity: NONE, imp: Imp::F(ycor) },
    Prim { names: &["heading"], arity: NONE, imp: Imp::F(heading) },
    Prim { names: &["towards"], arity: ONE, imp: Imp::F(towards) },
    Prim { names: &["arc"], arity: TWO, imp: Imp::F(arc) },
    // Shapes (our extension, ADR 0017).
    Prim { names: &["circle"], arity: ONE, imp: Imp::F(circle) },
    Prim { names: &["ellipse"], arity: TWO, imp: Imp::F(ellipse) },
    Prim { names: &["square"], arity: some(1, 1, Some(2)), imp: Imp::F(square) },
    Prim { names: &["rectangle"], arity: some(2, 2, Some(3)), imp: Imp::F(rectangle) },
    // Turtle and window control.
    Prim { names: &["showturtle", "st"], arity: NONE, imp: Imp::F(showturtle) },
    Prim { names: &["hideturtle", "ht"], arity: NONE, imp: Imp::F(hideturtle) },
    Prim { names: &["shownp", "shown?"], arity: NONE, imp: Imp::F(shownp) },
    Prim { names: &["clean"], arity: NONE, imp: Imp::F(clean) },
    Prim { names: &["clearscreen", "cs"], arity: NONE, imp: Imp::F(clearscreen) },
    Prim { names: &["wrap"], arity: NONE, imp: Imp::F(wrap) },
    Prim { names: &["window"], arity: NONE, imp: Imp::F(window) },
    Prim { names: &["fence"], arity: NONE, imp: Imp::F(fence) },
    Prim { names: &["turtlemode"], arity: NONE, imp: Imp::F(turtlemode) },
    // Pen and background.
    Prim { names: &["penup", "pu"], arity: NONE, imp: Imp::F(penup) },
    Prim { names: &["pendown", "pd"], arity: NONE, imp: Imp::F(pendown) },
    Prim { names: &["pendownp", "pendown?"], arity: NONE, imp: Imp::F(pendownp) },
    Prim { names: &["setpencolor", "setpc"], arity: ONE, imp: Imp::F(setpencolor) },
    Prim { names: &["pencolor", "pc"], arity: NONE, imp: Imp::F(pencolor) },
    Prim { names: &["setpensize"], arity: ONE, imp: Imp::F(setpensize) },
    Prim { names: &["pensize"], arity: NONE, imp: Imp::F(pensize) },
    Prim { names: &["setbackground", "setbg"], arity: ONE, imp: Imp::F(setbackground) },
    Prim { names: &["background", "bg"], arity: NONE, imp: Imp::F(background) },
    Prim { names: &["repeat"], arity: TWO, imp: Imp::Repeat },
    Prim { names: &["repcount", "#"], arity: NONE, imp: Imp::F(repcount) },
    Prim { names: &["forever"], arity: ONE, imp: Imp::Forever },
    Prim { names: &["if"], arity: some(2, 2, Some(3)), imp: Imp::If },
    Prim { names: &["ifelse"], arity: Arity::fixed(3), imp: Imp::IfElse },
    Prim { names: &["stop"], arity: Arity::fixed(0), imp: Imp::Stop },
    Prim { names: &["output", "op"], arity: ONE, imp: Imp::Output },
    Prim { names: &["make"], arity: TWO, imp: Imp::F(make) },
    Prim { names: &["local"], arity: some(1, 1, None), imp: Imp::F(local) },
    Prim { names: &["localmake"], arity: TWO, imp: Imp::F(localmake) },
    Prim { names: &["thing"], arity: ONE, imp: Imp::F(thing) },
    Prim { names: &["print", "pr"], arity: some(0, 1, None), imp: Imp::F(print) },
    Prim { names: &["type"], arity: some(0, 1, None), imp: Imp::F(type_) },
    Prim { names: &["show"], arity: some(0, 1, None), imp: Imp::F(show) },
    // Words and lists.
    Prim { names: &["word"], arity: some(0, 2, None), imp: Imp::F(word) },
    Prim { names: &["list"], arity: some(0, 2, None), imp: Imp::F(list_) },
    Prim { names: &["sentence", "se"], arity: some(0, 2, None), imp: Imp::F(sentence) },
    Prim { names: &["first"], arity: ONE, imp: Imp::F(first) },
    Prim { names: &["last"], arity: ONE, imp: Imp::F(last) },
    Prim { names: &["butfirst", "bf"], arity: ONE, imp: Imp::F(butfirst) },
    Prim { names: &["butlast", "bl"], arity: ONE, imp: Imp::F(butlast) },
    Prim { names: &["item"], arity: TWO, imp: Imp::F(item) },
    Prim { names: &["count"], arity: ONE, imp: Imp::F(count) },
    Prim { names: &["emptyp", "empty?"], arity: ONE, imp: Imp::F(emptyp) },
    Prim { names: &["sum"], arity: some(0, 2, None), imp: Imp::F(sum) },
    Prim { names: &["difference"], arity: TWO, imp: Imp::F(difference) },
    Prim { names: &["product"], arity: some(0, 2, None), imp: Imp::F(product) },
    Prim { names: &["quotient"], arity: some(1, 2, Some(2)), imp: Imp::F(quotient) },
    Prim { names: &["remainder"], arity: TWO, imp: Imp::F(remainder) },
    Prim { names: &["modulo"], arity: TWO, imp: Imp::F(modulo) },
    Prim { names: &["minus"], arity: ONE, imp: Imp::F(minus) },
    Prim { names: &["int"], arity: ONE, imp: Imp::F(int) },
    Prim { names: &["round"], arity: ONE, imp: Imp::F(round) },
    Prim { names: &["sqrt"], arity: ONE, imp: Imp::F(sqrt) },
    Prim { names: &["power"], arity: TWO, imp: Imp::F(power) },
    Prim { names: &["sin"], arity: ONE, imp: Imp::F(sin) },
    Prim { names: &["cos"], arity: ONE, imp: Imp::F(cos) },
    Prim { names: &["arctan"], arity: some(1, 1, Some(2)), imp: Imp::F(arctan) },
    Prim { names: &["random"], arity: some(1, 1, Some(2)), imp: Imp::F(random) },
    Prim { names: &["and"], arity: some(0, 2, None), imp: Imp::F(and) },
    Prim { names: &["or"], arity: some(0, 2, None), imp: Imp::F(or) },
    Prim { names: &["not"], arity: ONE, imp: Imp::F(not) },
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
        let shown = match &self.vals[i] {
            Value::Word(w) if w.is_empty() => "the empty word".to_string(),
            v => v.show(),
        };
        let message = format!("{} doesn't like {shown} as input", self.call.name.to_uppercase());
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

    /// A position: a list of two numbers.
    pub fn point(&self, i: usize) -> Result<Point, Flow> {
        if let Value::List(list) = &self.vals[i]
            && let [x, y] = list.items.as_slice()
            && let (Some(x), Some(y)) = (x.as_num(), y.as_num())
        {
            return Ok(Point { x, y });
        }
        Err(self.doesnt_like(i))
    }

    pub fn word(&self, i: usize) -> Result<Rc<str>, Flow> {
        match &self.vals[i] {
            Value::Word(w) => Ok(w.clone()),
            Value::Num(n) => Ok(crate::number::format_number(*n).into()),
            Value::List(_) => Err(self.doesnt_like(i)),
        }
    }

    /// Arithmetic results must be finite (ADR 0002).
    pub(super) fn result(&self, x: f64) -> Result<Option<Value>, Flow> {
        if x.is_finite() { Ok(Some(Value::Num(x))) } else { Err(self.error("That number is too big".into())) }
    }
}

fn repcount(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    match it.repcounts.last() {
        Some(&n) => Ok(Some(Value::Num(n as f64))),
        None => Err(a.error("REPCOUNT can only be used inside REPEAT or FOREVER".into())),
    }
}

fn make(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    let name = a.word(0)?;
    let sym = it.define(&name);
    it.vars[sym as usize] = Some(a.vals[1].clone());
    Ok(None)
}

/// The names `LOCAL` gets: words, or lists of words.
fn names(a: &Args) -> Result<Vec<Rc<str>>, Flow> {
    let mut names = Vec::new();
    for (i, v) in a.vals.iter().enumerate() {
        match v {
            Value::List(list) => {
                for item in &list.items {
                    match item {
                        Value::Word(w) => names.push(w.clone()),
                        _ => return Err(a.doesnt_like(i)),
                    }
                }
            }
            _ => names.push(a.word(i)?),
        }
    }
    Ok(names)
}

fn local(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    for name in names(a)? {
        let sym = it.define(&name);
        if !it.make_local(sym) {
            return Err(a.error(format!("{} can only be used inside a procedure", a.call.name.to_uppercase())));
        }
    }
    Ok(None)
}

/// `LOCAL` then `MAKE`.
fn localmake(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    let name = a.word(0)?;
    let sym = it.define(&name);
    if !it.make_local(sym) {
        return Err(a.error("LOCALMAKE can only be used inside a procedure".into()));
    }
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

