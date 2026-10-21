//! The pen: up or down, colour and size (ADR 0015). Issue 07 adds the rest
//! of the colour forms and the queries.

use super::Args;
use crate::{
    interp::{Flow, Interpreter},
    value::Value,
};

pub(super) fn penup(it: &mut Interpreter, _: &Args) -> Result<Option<Value>, Flow> {
    it.turtle.pen.down = false;
    Ok(None)
}

pub(super) fn pendown(it: &mut Interpreter, _: &Args) -> Result<Option<Value>, Flow> {
    it.turtle.pen.down = true;
    Ok(None)
}

/// UCBLogo's default palette (ADR 0015).
const PALETTE: [[u8; 3]; 16] = [
    [0, 0, 0],
    [0, 0, 255],
    [0, 255, 0],
    [0, 255, 255],
    [255, 0, 0],
    [255, 0, 255],
    [255, 255, 0],
    [255, 255, 255],
    [155, 96, 59],
    [197, 136, 18],
    [100, 162, 64],
    [120, 187, 187],
    [255, 149, 119],
    [144, 113, 208],
    [255, 163, 0],
    [183, 183, 183],
];

/// A palette number or an `[r g b]` list from 0 to 255. The other colour
/// forms come with phase 1's issue 07.
pub(super) fn setpencolor(it: &mut Interpreter, a: &Args) -> Result<Option<Value>, Flow> {
    let [r, g, b] = match &a.vals[0] {
        Value::List(list) if list.items.len() == 3 => {
            let mut rgb = [0u8; 3];
            for (c, item) in rgb.iter_mut().zip(&list.items) {
                match item.as_num() {
                    Some(n) if (0.0..=255.0).contains(&n) => *c = n.round() as u8,
                    _ => return Err(a.doesnt_like(0)),
                }
            }
            rgb
        }
        v => match v.as_num() {
            Some(n) if n.fract() == 0.0 && (0.0..16.0).contains(&n) => PALETTE[n as usize],
            _ => return Err(a.doesnt_like(0)),
        },
    };
    it.turtle.pen.color = [r, g, b, 255];
    Ok(None)
}

