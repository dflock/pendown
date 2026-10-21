//! The built-in procedures. The table here gives each one's names, inputs
//! and code; the code is in a module per area, as `pendown-logo.kdl` groups
//! them.

mod arithmetic;
mod control;
mod data;
mod pen;
mod shapes;
mod templates;
mod turtle;

use std::{collections::HashMap, rc::Rc, sync::OnceLock};

use arithmetic::*;
use control::*;
use data::*;
use pen::*;
use shapes::*;
use templates::*;
use turtle::*;

pub use templates::Data;

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
    Prim { names: &["setpalette"], arity: TWO, imp: Imp::F(setpalette) },
    Prim { names: &["palette"], arity: ONE, imp: Imp::F(palette) },
    Prim { names: &["setfillcolor", "setfc"], arity: ONE, imp: Imp::F(setfillcolor) },
    Prim { names: &["fillcolor", "fc"], arity: NONE, imp: Imp::F(fillcolor) },
    Prim { names: &["repeat"], arity: TWO, imp: Imp::Repeat },
    Prim { names: &["repcount", "#"], arity: NONE, imp: Imp::F(repcount) },
    Prim { names: &["forever"], arity: ONE, imp: Imp::Forever },
    Prim { names: &["if"], arity: some(2, 2, Some(3)), imp: Imp::If },
    Prim { names: &["ifelse"], arity: Arity::fixed(3), imp: Imp::IfElse },
    Prim { names: &["stop"], arity: Arity::fixed(0), imp: Imp::Stop },
    Prim { names: &["wait"], arity: ONE, imp: Imp::F(wait) },
    Prim { names: &["setspeed"], arity: ONE, imp: Imp::F(setspeed) },
    Prim { names: &["speed"], arity: NONE, imp: Imp::F(speed) },
    Prim { names: &["output", "op"], arity: ONE, imp: Imp::Output },
    Prim { names: &["run"], arity: ONE, imp: Imp::F(run) },
    Prim { names: &["runresult"], arity: ONE, imp: Imp::F(runresult) },
    Prim { names: &["ignore"], arity: ONE, imp: Imp::F(ignore) },
    Prim { names: &["for"], arity: TWO, imp: Imp::F(for_) },
    Prim { names: &["while"], arity: TWO, imp: Imp::F(while_) },
    Prim { names: &["until"], arity: TWO, imp: Imp::F(until) },
    Prim { names: &["do.while"], arity: TWO, imp: Imp::F(do_while) },
    Prim { names: &["do.until"], arity: TWO, imp: Imp::F(do_until) },
    Prim { names: &["case"], arity: TWO, imp: Imp::F(case) },
    Prim { names: &["cond"], arity: ONE, imp: Imp::F(cond) },
    Prim { names: &["catch"], arity: TWO, imp: Imp::F(catch) },
    Prim { names: &["throw"], arity: some(1, 1, Some(2)), imp: Imp::F(throw) },
    Prim { names: &["error"], arity: NONE, imp: Imp::F(error) },
    // Templates (ADR 0021).
    Prim { names: &["apply"], arity: TWO, imp: Imp::F(apply_) },
    Prim { names: &["invoke"], arity: some(1, 2, None), imp: Imp::F(invoke) },
    Prim { names: &["foreach"], arity: some(2, 2, None), imp: Imp::F(foreach) },
    Prim { names: &["map"], arity: some(2, 2, None), imp: Imp::F(map) },
    Prim { names: &["map.se"], arity: some(2, 2, None), imp: Imp::F(map_se) },
    Prim { names: &["filter"], arity: TWO, imp: Imp::F(filter) },
    Prim { names: &["find"], arity: TWO, imp: Imp::F(find) },
    Prim { names: &["reduce"], arity: TWO, imp: Imp::F(reduce) },
    Prim { names: &["?"], arity: some(0, 0, Some(1)), imp: Imp::F(slot) },
    Prim { names: &["?rest"], arity: some(0, 0, Some(1)), imp: Imp::F(slot_rest) },
    Prim { names: &["make"], arity: TWO, imp: Imp::F(make) },
    Prim { names: &["local"], arity: some(1, 1, None), imp: Imp::F(local) },
    Prim { names: &["localmake"], arity: TWO, imp: Imp::F(localmake) },
    Prim { names: &["thing"], arity: ONE, imp: Imp::F(thing) },
    Prim { names: &["name"], arity: TWO, imp: Imp::F(name) },
    Prim { names: &["global"], arity: some(1, 1, None), imp: Imp::F(global) },
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
    Prim { names: &["fput"], arity: TWO, imp: Imp::F(fput) },
    Prim { names: &["lput"], arity: TWO, imp: Imp::F(lput) },
    Prim { names: &["combine"], arity: TWO, imp: Imp::F(combine) },
    Prim { names: &["reverse"], arity: ONE, imp: Imp::F(reverse) },
    Prim { names: &["firsts"], arity: ONE, imp: Imp::F(firsts) },
    Prim { names: &["butfirsts", "bfs"], arity: ONE, imp: Imp::F(butfirsts) },
    Prim { names: &["pick"], arity: ONE, imp: Imp::F(pick) },
    Prim { names: &["remove"], arity: TWO, imp: Imp::F(remove) },
    Prim { names: &["remdup"], arity: ONE, imp: Imp::F(remdup) },
    Prim { names: &["quoted"], arity: ONE, imp: Imp::F(quoted) },
    Prim { names: &["push"], arity: TWO, imp: Imp::F(push) },
    Prim { names: &["pop"], arity: ONE, imp: Imp::F(pop) },
    Prim { names: &["queue"], arity: TWO, imp: Imp::F(queue) },
    Prim { names: &["dequeue"], arity: ONE, imp: Imp::F(pop) },
    Prim { names: &["wordp", "word?"], arity: ONE, imp: Imp::F(wordp) },
    Prim { names: &["listp", "list?"], arity: ONE, imp: Imp::F(listp) },
    Prim { names: &["arrayp", "array?"], arity: ONE, imp: Imp::F(arrayp) },
    Prim { names: &["numberp", "number?"], arity: ONE, imp: Imp::F(numberp) },
    Prim { names: &["beforep", "before?"], arity: TWO, imp: Imp::F(beforep) },
    Prim { names: &["memberp", "member?"], arity: TWO, imp: Imp::F(memberp) },
    Prim { names: &["substringp", "substring?"], arity: TWO, imp: Imp::F(substringp) },
    Prim { names: &["member"], arity: TWO, imp: Imp::F(member) },
    Prim { names: &["ascii"], arity: ONE, imp: Imp::F(ascii) },
    Prim { names: &["char"], arity: ONE, imp: Imp::F(char) },
    Prim { names: &["lowercase"], arity: ONE, imp: Imp::F(lowercase) },
    Prim { names: &["uppercase"], arity: ONE, imp: Imp::F(uppercase) },
    // Arrays (ADR 0021).
    Prim { names: &["array"], arity: some(1, 1, Some(2)), imp: Imp::F(array) },
    Prim { names: &["mdarray"], arity: some(1, 1, Some(2)), imp: Imp::F(mdarray) },
    Prim { names: &["listtoarray"], arity: some(1, 1, Some(2)), imp: Imp::F(listtoarray) },
    Prim { names: &["arraytolist"], arity: ONE, imp: Imp::F(arraytolist) },
    Prim { names: &["setitem"], arity: Arity::fixed(3), imp: Imp::F(setitem) },
    Prim { names: &["mditem"], arity: TWO, imp: Imp::F(mditem) },
    Prim { names: &["mdsetitem"], arity: Arity::fixed(3), imp: Imp::F(mdsetitem) },
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
    Prim { names: &["exp"], arity: ONE, imp: Imp::F(exp) },
    Prim { names: &["log10"], arity: ONE, imp: Imp::F(log10) },
    Prim { names: &["ln"], arity: ONE, imp: Imp::F(ln) },
    Prim { names: &["radsin"], arity: ONE, imp: Imp::F(radsin) },
    Prim { names: &["radcos"], arity: ONE, imp: Imp::F(radcos) },
    Prim { names: &["radarctan"], arity: some(1, 1, Some(2)), imp: Imp::F(radarctan) },
    Prim { names: &["iseq"], arity: TWO, imp: Imp::F(iseq) },
    Prim { names: &["rseq"], arity: Arity::fixed(3), imp: Imp::F(rseq) },
    Prim { names: &["rerandom"], arity: some(0, 0, Some(1)), imp: Imp::F(rerandom) },
    Prim { names: &["form"], arity: Arity::fixed(3), imp: Imp::F(form) },
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
            Value::List(_) | Value::Array(_) => Err(self.doesnt_like(i)),
        }
    }

    /// Arithmetic results must be finite (ADR 0002).
    pub(super) fn result(&self, x: f64) -> Result<Option<Value>, Flow> {
        if x.is_finite() { Ok(Some(Value::Num(x))) } else { Err(self.error("That number is too big".into())) }
    }
}

/// `WAIT time`, in 60ths of a second.
fn wait(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    let sixtieths = a.num(0)?;
    if sixtieths < 0.0 {
        return Err(a.doesnt_like(0));
    }
    if sixtieths > 0.0 {
        it.emit(Event::Wait(std::time::Duration::from_secs_f64(sixtieths / 60.0)))?;
    }
    Ok(None)
}

/// `SETSPEED level`, from 1 to 11, where 11 is Instant (ADR 0011).
fn setspeed(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    let level = a.int(0)?;
    if !(1..=11).contains(&level) {
        return Err(a.doesnt_like(0));
    }
    it.control().speed.store(level as u8, std::sync::atomic::Ordering::Relaxed);
    it.emit(Event::SetSpeed(level as u8))?;
    Ok(None)
}

/// The level the speed slider shows, which `SETSPEED` changes too.
fn speed(it: &mut Interpreter, _: &Args) -> Result<Option<Value>, Flow> {
    let level = it.control().speed.load(std::sync::atomic::Ordering::Relaxed);
    Ok(Some(Value::Num(if level == 0 { 11.0 } else { level as f64 })))
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

/// `NAME value varname`: `MAKE` with its inputs the other way round.
fn name(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    let name = a.word(1)?;
    let sym = it.define(&name);
    it.vars[sym as usize] = Some(a.vals[0].clone());
    Ok(None)
}

/// `GLOBAL`: in UCBLogo, this lets the setter notation (`SETXYZ`) work for
/// a variable with no value yet. Pendown has no setter notation, and `MAKE`
/// creates globals anyway, so it only checks its inputs and records the
/// names' spelling.
fn global(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    for name in names(a)? {
        it.define(&name);
    }
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

fn joined(a: &Args, form: impl Fn(&Value) -> String) -> String {
    a.vals.iter().map(form).collect::<Vec<_>>().join(" ")
}

fn print(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    let style = it.print_style();
    it.emit(Event::Print(joined(a, |v| v.print_styled(style)) + "\n"))?;
    Ok(None)
}

fn type_(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    let style = it.print_style();
    it.emit(Event::Print(a.vals.iter().map(|v| v.print_styled(style)).collect()))?;
    Ok(None)
}

fn show(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    let style = it.print_style();
    it.emit(Event::Print(joined(a, |v| v.show_styled(style)) + "\n"))?;
    Ok(None)
}

fn word(_: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    let mut out = String::new();
    for i in 0..a.vals.len() {
        out.push_str(&a.word(i)?);
    }
    Ok(Some(Value::Word(out.into())))
}

