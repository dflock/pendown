//! `ARC` (ADR 0016) and the shapes (ADR 0017).

use super::{
    Args,
    turtle::{move_to, turn},
};
use crate::{
    event::{Event, Point},
    geometry::{TurtleMode, arc_center, arc_chords},
    interp::{Flow, Interpreter},
    value::Value,
};

type Out = Result<Option<Value>, Flow>;

impl Args<'_> {
    /// A number that's zero or more: a size or a radius.
    fn size(&self, i: usize) -> Result<f64, Flow> {
        let n = self.num(i)?;
        if n >= 0.0 { Ok(n) } else { Err(self.doesnt_like(i)) }
    }
}

/// `ARC angle radius`: the turtle walks along an arc of a circle with that
/// radius, turning through `angle` degrees as it goes, clockwise if
/// positive.
pub(super) fn arc(it: &mut Interpreter, a: &Args) -> Out {
    let angle = a.num(0)?;
    let radius = a.size(1)?;
    if angle == 0.0 {
        return Ok(None);
    }
    if radius == 0.0 {
        // No arc at all: just the turn.
        return turn(it, angle);
    }
    let (start, heading) = (it.turtle.pos, it.turtle.heading);
    let center = arc_center(start, heading, radius, angle);
    let points = arc_chords(center, radius, heading, angle);
    let inside = points.iter().all(|p| it.area.contains(*p));
    if it.turtle.mode == TurtleMode::Window || inside {
        let pen = it.turtle.pen;
        it.emit(Event::Arc { center, radius, start_heading: heading, angle, pen })?;
        it.turtle.pos = *points.last().expect("at least two points");
        it.turtle.heading = (heading + angle).rem_euclid(360.0);
        return Ok(None);
    }
    // The arc crosses an edge, so it goes as short straight moves, a degree
    // at a time, which wrap or stop at the fence like any other move. A move
    // that wraps shifts the rest of the arc with it.
    let step = angle / (points.len() - 1) as f64;
    let mut offset = Point { x: 0.0, y: 0.0 };
    for to in &points[1..] {
        let target = Point { x: to.x + offset.x, y: to.y + offset.y };
        move_to(it, a, target)?;
        offset = Point { x: it.turtle.pos.x - to.x, y: it.turtle.pos.y - to.y };
        turn(it, step)?;
    }
    Ok(None)
}

fn ellipse_event(it: &mut Interpreter, radii: (f64, f64)) -> Out {
    let t = &it.turtle;
    let event = Event::Ellipse { center: t.pos, radii, heading: t.heading, pen: t.pen, fill: None };
    it.emit(event)?;
    Ok(None)
}

fn rect_event(it: &mut Interpreter, a: &Args, size: (f64, f64), corner_input: usize) -> Out {
    let corner = if a.vals.len() > corner_input { a.size(corner_input)? } else { 0.0 };
    // A corner can't be rounder than half the shorter side.
    let corner = corner.min(size.0.min(size.1) / 2.0);
    let t = &it.turtle;
    let event = Event::Rect { center: t.pos, size, corner, heading: t.heading, pen: t.pen, fill: None };
    it.emit(event)?;
    Ok(None)
}

pub(super) fn circle(it: &mut Interpreter, a: &Args) -> Out {
    let r = a.size(0)?;
    ellipse_event(it, (r, r))
}

pub(super) fn ellipse(it: &mut Interpreter, a: &Args) -> Out {
    let radii = (a.size(0)?, a.size(1)?);
    ellipse_event(it, radii)
}

pub(super) fn square(it: &mut Interpreter, a: &Args) -> Out {
    let side = a.size(0)?;
    rect_event(it, a, (side, side), 1)
}

pub(super) fn rectangle(it: &mut Interpreter, a: &Args) -> Out {
    let size = (a.size(0)?, a.size(1)?);
    rect_event(it, a, size, 2)
}
