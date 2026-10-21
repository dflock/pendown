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
    pub fn colour(&self, i: usize) -> Result<Rgba, Flow> {
        colour::parse(&self.vals[i]).map_err(|e| match e.message() {
            None => self.doesnt_like(i),
            Some(message) => Flow::Error(Box::new(Error::new(message, Some(self.call.args[i].span())))),
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
    it.turtle.pen.color = a.colour(0)?;
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
    let [r, g, b, _] = a.colour(0)?;
    it.background = ([r, g, b, 255], a.vals[0].clone());
    it.emit(Event::SetBackground { color: it.background.0 })?;
    Ok(None)
}

pub(super) fn background(it: &mut Interpreter, _: &Args) -> Out {
    Ok(Some(it.background.1.clone()))
}
