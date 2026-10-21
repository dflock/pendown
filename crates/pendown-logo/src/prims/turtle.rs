//! Turtle motion, queries and visibility. Positions are in the turtle
//! area's coordinates, with y up, and headings are degrees clockwise from
//! north (ADR 0009). The turtle's state lives here in the interpreter, so
//! queries never ask the GUI.

use std::rc::Rc;

use super::Args;
use crate::{
    event::{Event, Pen, Point},
    geometry::TurtleMode,
    interp::{Flow, Interpreter},
    number::sin_cos_degrees,
    value::{List, Value},
};

type Out = Result<Option<Value>, Flow>;

/// Moves the turtle in a straight line to `to`, drawing if the pen is
/// down. In `WRAP` mode the move is split where it crosses an edge, and in
/// `FENCE` mode it stops at the edge with an error.
pub(super) fn move_to(it: &mut Interpreter, a: &Args, to: Point) -> Out {
    let path = it.area.line(it.turtle.pos, to, it.turtle.mode);
    let pen = it.turtle.pen;
    for (from, to) in path.pieces {
        it.emit(Event::Move { from, to, pen })?;
    }
    it.turtle.pos = path.end;
    if path.fenced {
        return Err(a.error("The turtle can't go past the edge in FENCE mode. Use WRAP or WINDOW to let it.".into()));
    }
    Ok(None)
}

fn walk(it: &mut Interpreter, a: &Args, distance: f64) -> Out {
    let (sin, cos) = sin_cos_degrees(it.turtle.heading);
    let from = it.turtle.pos;
    move_to(it, a, Point { x: from.x + distance * sin, y: from.y + distance * cos })
}

/// Turns the turtle by `by` degrees, clockwise if positive. The event
/// carries the signed angle, so `RT 270` visibly turns 270° (ADR 0009).
pub(super) fn turn(it: &mut Interpreter, by: f64) -> Out {
    let from = it.turtle.heading;
    it.turtle.heading = (from + by).rem_euclid(360.0);
    it.emit(Event::Turn { from, by })?;
    Ok(None)
}

/// Turns to an absolute heading, the shorter way round.
fn turn_to(it: &mut Interpreter, heading: f64) -> Out {
    let mut by = (heading - it.turtle.heading).rem_euclid(360.0);
    if by > 180.0 {
        by -= 360.0;
    }
    if by == 0.0 {
        return Ok(None);
    }
    turn(it, by)
}

pub(super) fn forward(it: &mut Interpreter, a: &Args) -> Out {
    walk(it, a, a.num(0)?)
}

pub(super) fn back(it: &mut Interpreter, a: &Args) -> Out {
    walk(it, a, -a.num(0)?)
}

pub(super) fn right(it: &mut Interpreter, a: &Args) -> Out {
    turn(it, a.num(0)?)
}

pub(super) fn left(it: &mut Interpreter, a: &Args) -> Out {
    turn(it, -a.num(0)?)
}

pub(super) fn setpos(it: &mut Interpreter, a: &Args) -> Out {
    let to = a.point(0)?;
    move_to(it, a, to)
}

pub(super) fn setxy(it: &mut Interpreter, a: &Args) -> Out {
    let to = Point { x: a.num(0)?, y: a.num(1)? };
    move_to(it, a, to)
}

pub(super) fn setx(it: &mut Interpreter, a: &Args) -> Out {
    let to = Point { x: a.num(0)?, y: it.turtle.pos.y };
    move_to(it, a, to)
}

pub(super) fn sety(it: &mut Interpreter, a: &Args) -> Out {
    let to = Point { x: it.turtle.pos.x, y: a.num(0)? };
    move_to(it, a, to)
}

pub(super) fn setheading(it: &mut Interpreter, a: &Args) -> Out {
    turn_to(it, a.num(0)?.rem_euclid(360.0))
}

/// `HOME` is a move like any other, so it draws when the pen is down.
pub(super) fn home(it: &mut Interpreter, a: &Args) -> Out {
    move_to(it, a, Point { x: 0.0, y: 0.0 })?;
    turn_to(it, 0.0)
}

fn point_value(p: Point) -> Value {
    Value::List(Rc::new(List::new(vec![Value::Num(tidy(p.x)), Value::Num(tidy(p.y))])))
}

/// Positions are reported without the tiny errors that sines and cosines
/// leave, such as `1e-14` for 0, so `POS` after `FD 100 RT 90 FD 100 RT 90
/// FD 100` is `[100 0]`. Anything within 1e-9 of a whole number is that
/// number.
fn tidy(x: f64) -> f64 {
    let whole = x.round();
    if (x - whole).abs() < 1e-9 { whole + 0.0 } else { x }
}

pub(super) fn pos(it: &mut Interpreter, _: &Args) -> Out {
    Ok(Some(point_value(it.turtle.pos)))
}

pub(super) fn xcor(it: &mut Interpreter, _: &Args) -> Out {
    Ok(Some(Value::Num(tidy(it.turtle.pos.x))))
}

pub(super) fn ycor(it: &mut Interpreter, _: &Args) -> Out {
    Ok(Some(Value::Num(tidy(it.turtle.pos.y))))
}

pub(super) fn heading(it: &mut Interpreter, _: &Args) -> Out {
    Ok(Some(Value::Num(tidy(it.turtle.heading).rem_euclid(360.0))))
}

/// The heading that points from the turtle to the given position.
pub(super) fn towards(it: &mut Interpreter, a: &Args) -> Out {
    let to = a.point(0)?;
    let from = it.turtle.pos;
    let heading = (to.x - from.x).atan2(to.y - from.y).to_degrees().rem_euclid(360.0);
    Ok(Some(Value::Num(tidy(heading).rem_euclid(360.0))))
}

pub(super) fn showturtle(it: &mut Interpreter, _: &Args) -> Out {
    it.turtle.shown = true;
    it.emit(Event::ShowTurtle)?;
    Ok(None)
}

pub(super) fn hideturtle(it: &mut Interpreter, _: &Args) -> Out {
    it.turtle.shown = false;
    it.emit(Event::HideTurtle)?;
    Ok(None)
}

pub(super) fn shownp(it: &mut Interpreter, _: &Args) -> Out {
    Ok(Some(it.bool(it.turtle.shown)))
}

/// Clears the drawing. The turtle stays where it is.
pub(super) fn clean(it: &mut Interpreter, _: &Args) -> Out {
    it.emit(Event::Clean)?;
    Ok(None)
}

/// `CLEAN` and `HOME` together, except that the move home never draws:
/// the turtle goes home with the pen up, then the pen is as it was.
pub(super) fn clearscreen(it: &mut Interpreter, a: &Args) -> Out {
    clean(it, a)?;
    let down = it.turtle.pen.down;
    it.turtle.pen.down = false;
    // Home is always inside the area, so this never wraps or fences.
    let r = move_to(it, a, Point { x: 0.0, y: 0.0 });
    it.turtle.pen.down = down;
    r?;
    turn_to(it, 0.0)
}

/// `WRAP` and `FENCE` first bring a turtle that's outside the area back in,
/// wrapping its position, as UCBLogo does. The jump isn't drawn.
fn set_mode(it: &mut Interpreter, mode: TurtleMode) -> Out {
    if mode != TurtleMode::Window && !it.area.contains(it.turtle.pos) {
        let from = it.turtle.pos;
        let to = it.area.wrap_point(from);
        it.turtle.pos = to;
        let pen = Pen { down: false, ..it.turtle.pen };
        it.emit(Event::Move { from, to, pen })?;
    }
    it.turtle.mode = mode;
    Ok(None)
}

pub(super) fn wrap(it: &mut Interpreter, _: &Args) -> Out {
    set_mode(it, TurtleMode::Wrap)
}

pub(super) fn window(it: &mut Interpreter, _: &Args) -> Out {
    set_mode(it, TurtleMode::Window)
}

pub(super) fn fence(it: &mut Interpreter, _: &Args) -> Out {
    set_mode(it, TurtleMode::Fence)
}

pub(super) fn turtlemode(it: &mut Interpreter, _: &Args) -> Out {
    let name = match it.turtle.mode {
        TurtleMode::Wrap => "WRAP",
        TurtleMode::Window => "WINDOW",
        TurtleMode::Fence => "FENCE",
    };
    Ok(Some(Value::word(name)))
}
