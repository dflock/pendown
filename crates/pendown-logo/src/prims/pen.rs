//! The pen and the background (ADR 0015).

use std::rc::Rc;

use super::Args;
use crate::{
    colour::{self, Rgba},
    error::Error,
    event::Event,
    interp::{Flow, Interpreter},
    value::{List, Value},
};

type Out = Result<Option<Value>, Flow>;

impl Args<'_> {
    /// A colour in any of its forms (ADR 0015).
    pub fn colour(&self, it: &Interpreter, i: usize) -> Result<Rgba, Flow> {
        colour::parse_with(&self.vals[i], &it.palette).map_err(|e| match e.message() {
            None => self.doesnt_like(i),
            Some(message) => Flow::Error(Box::new(Error::new(message, Some(self.call.args[i].span())).about(&self.call.name))),
        })
    }
}

pub(super) fn penup(it: &mut Interpreter, _: &Args) -> Out {
    it.turtle.pen.down = false;
    Ok(None)
}

pub(super) fn pendown(it: &mut Interpreter, _: &Args) -> Out {
    it.turtle.pen.down = true;
    Ok(None)
}

pub(super) fn pendownp(it: &mut Interpreter, _: &Args) -> Out {
    Ok(Some(it.bool(it.turtle.pen.down)))
}

pub(super) fn setpencolor(it: &mut Interpreter, a: &Args) -> Out {
    it.turtle.pen.color = a.colour(it, 0)?;
    it.turtle.pen_colour = a.vals[0].clone();
    Ok(None)
}

/// The colour in the form it was set, so `SETPC PC` always works.
pub(super) fn pencolor(it: &mut Interpreter, _: &Args) -> Out {
    Ok(Some(it.turtle.pen_colour.clone()))
}

/// A positive number, or `[width height]` as in UCBLogo. The pen is always
/// square, so the height is ignored.
pub(super) fn setpensize(it: &mut Interpreter, a: &Args) -> Out {
    let size = match &a.vals[0] {
        Value::List(list) => match list.items.as_slice() {
            [w, _] => w.as_num(),
            _ => None,
        },
        v => v.as_num(),
    };
    match size {
        Some(size) if size > 0.0 => {
            it.turtle.pen.size = size;
            Ok(None)
        }
        _ => Err(a.doesnt_like(0)),
    }
}

/// `[width height]`, as in UCBLogo. They're always equal.
pub(super) fn pensize(it: &mut Interpreter, _: &Args) -> Out {
    let size = Value::Num(it.turtle.pen.size);
    Ok(Some(Value::List(Rc::new(List::new(vec![size.clone(), size])))))
}

/// The background is always solid, so any opacity is ignored.
pub(super) fn setbackground(it: &mut Interpreter, a: &Args) -> Out {
    let [r, g, b, _] = a.colour(it, 0)?;
    it.background = ([r, g, b, 255], a.vals[0].clone());
    it.emit(Event::SetBackground { color: it.background.0 })?;
    Ok(None)
}

pub(super) fn background(it: &mut Interpreter, _: &Args) -> Out {
    Ok(Some(it.background.1.clone()))
}

/// `SETPALETTE number colour`: changes colour numbers 8 to 15, as UCBLogo
/// keeps 0 to 7 the same. The colour can be any form but a number. Colours
/// already in use don't change.
pub(super) fn setpalette(it: &mut Interpreter, a: &Args) -> Out {
    let n = a.int(0)?;
    if !(8..16).contains(&n) {
        return Err(a.doesnt_like(0));
    }
    if a.vals[1].as_num().is_some() {
        return Err(a.doesnt_like(1));
    }
    it.palette[n as usize] = a.colour(it, 1)?;
    Ok(None)
}

/// `PALETTE number`: an RGB list, or RGBA if it isn't opaque.
pub(super) fn palette(it: &mut Interpreter, a: &Args) -> Out {
    let n = a.int(0)?;
    if !(0..16).contains(&n) {
        return Err(a.doesnt_like(0));
    }
    let [r, g, b, alpha] = it.palette[n as usize];
    let mut items: Vec<Value> = [r, g, b].iter().map(|c| Value::Num(*c as f64)).collect();
    if alpha != 255 {
        items.push(Value::Num(alpha as f64));
    }
    Ok(Some(Value::List(Rc::new(List::new(items)))))
}

/// `SETFILLCOLOR colour` (ADR 0017): shapes are filled with it, even with
/// the pen up. `SETFILLCOLOR "none` turns filling off.
pub(super) fn setfillcolor(it: &mut Interpreter, a: &Args) -> Out {
    it.turtle.fill = match &a.vals[0] {
        Value::Word(w) if w.eq_ignore_ascii_case("none") => None,
        _ => Some(a.colour(it, 0)?),
    };
    it.turtle.fill_colour = a.vals[0].clone();
    Ok(None)
}

/// The fill colour in the form it was set, or `none`.
pub(super) fn fillcolor(it: &mut Interpreter, _: &Args) -> Out {
    Ok(Some(it.turtle.fill_colour.clone()))
}
