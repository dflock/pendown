//! The Colours page's arithmetic, without a window: hue, saturation and
//! brightness, the wheel, and colours as Logo text in each form (ADR 0015).

use pendown_logo::{COLOUR_NAMES, PALETTE, Rgba};

/// A colour as the picker keeps it: hue from 0 to 360, saturation and
/// brightness from 0 to 1, and opacity from 0 to 255. Kept this way while
/// picking, so dragging the brightness to black and back keeps the hue.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hsva {
    pub h: f64,
    pub s: f64,
    pub v: f64,
    pub a: u8,
}

impl Hsva {
    pub fn from_rgba([r, g, b, a]: Rgba) -> Hsva {
        let (r, g, b) = (r as f64 / 255., g as f64 / 255., b as f64 / 255.);
        let max = r.max(g).max(b);
        let range = max - r.min(g).min(b);
        let h = if range == 0. {
            0.
        } else if max == r {
            60. * ((g - b) / range).rem_euclid(6.)
        } else if max == g {
            60. * ((b - r) / range + 2.)
        } else {
            60. * ((r - g) / range + 4.)
        };
        let s = if max == 0. { 0. } else { range / max };
        Hsva { h, s, v: max, a }
    }

    pub fn rgba(&self) -> Rgba {
        let h = self.h.rem_euclid(360.) / 60.;
        let c = self.v * self.s;
        let x = c * (1. - (h % 2. - 1.).abs());
        let (r, g, b) = match h as u32 {
            0 => (c, x, 0.),
            1 => (x, c, 0.),
            2 => (0., c, x),
            3 => (0., x, c),
            4 => (x, 0., c),
            _ => (c, 0., x),
        };
        let m = self.v - c;
        let byte = |n: f64| ((n + m) * 255.).round().clamp(0., 255.) as u8;
        [byte(r), byte(g), byte(b), self.a]
    }
}

/// Where a hue and saturation are on the wheel, in a disc of radius 1
/// centred on 0, 0, with y down: red at the top, then clockwise through
/// yellow, green, cyan, blue and magenta, white in the middle.
pub fn wheel_point(h: f64, s: f64) -> (f64, f64) {
    let angle = h.to_radians();
    (s * angle.sin(), -s * angle.cos())
}

/// The hue and saturation at a point on the wheel, as `wheel_point` places
/// them. A point outside the disc gives the colour at its edge.
pub fn wheel_colour(x: f64, y: f64) -> (f64, f64) {
    let s = x.hypot(y).min(1.);
    let h = x.atan2(-y).to_degrees().rem_euclid(360.);
    (h, s)
}

/// How a colour is written in a program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    Palette(u8),
    Name(&'static str),
    Rgb,
    Rgba,
    Hex,
}

impl Form {
    /// Its label on the form buttons.
    pub fn label(&self) -> &'static str {
        match self {
            Form::Palette(_) => "Palette",
            Form::Name(_) => "Name",
            Form::Rgb => "RGB",
            Form::Rgba => "RGBA",
            Form::Hex => "Hex",
        }
    }
}

/// A colour as Logo text, as `SETPC` takes it: `4`, `"tomato`,
/// `[255 99 71]`, `[255 99 71 128]`, or `"#ff6347`, with two more hex
/// digits only when it isn't solid.
pub fn text(form: Form, [r, g, b, a]: Rgba) -> String {
    match form {
        Form::Palette(n) => n.to_string(),
        Form::Name(name) => format!("\"{name}"),
        Form::Rgb => format!("[{r} {g} {b}]"),
        Form::Rgba => format!("[{r} {g} {b} {a}]"),
        Form::Hex if a == 255 => format!("\"#{r:02x}{g:02x}{b:02x}"),
        Form::Hex => format!("\"#{r:02x}{g:02x}{b:02x}{a:02x}"),
    }
}

fn name_rgba(rgb: u32) -> Rgba {
    [(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 255]
}

/// The first name, alphabetically, that is exactly this colour, if any.
pub fn name_of(rgba: Rgba) -> Option<&'static str> {
    COLOUR_NAMES
        .iter()
        .find(|(_, rgb)| name_rgba(*rgb) == rgba)
        .map(|(name, _)| *name)
}

/// The default palette number that is exactly this colour, if any.
pub fn palette_of(rgba: Rgba) -> Option<u8> {
    PALETTE.iter().position(|c| *c == rgba).map(|n| n as u8)
}

/// The forms a colour can be copied in: RGB, RGBA and hex always, and its
/// name and palette number when it's exactly one.
pub fn forms(rgba: Rgba) -> Vec<Form> {
    let mut forms = Vec::new();
    forms.extend(palette_of(rgba).map(Form::Palette));
    forms.extend(name_of(rgba).map(Form::Name));
    forms.extend([Form::Rgb, Form::Rgba, Form::Hex]);
    forms
}

/// What goes in the editor for `text` when the character before the
/// cursor is `before`: a space first, unless it follows a space, a `[`, or
/// the start of a line or the program.
pub fn insertion(before: Option<char>, text: &str) -> String {
    match before {
        None => text.to_string(),
        Some(c) if c.is_whitespace() || c == '[' => text.to_string(),
        Some(_) => format!(" {text}"),
    }
}

/// What's typed in the R, G, B or A field, called `label`.
pub fn parse_channel(text: &str, label: &str) -> Result<u8, String> {
    match text.trim().parse::<f64>() {
        Ok(n) if (0.0..=255.0).contains(&n) => Ok(n.round() as u8),
        _ => Err(format!("{label} goes from 0 to 255.")),
    }
}

/// What's typed in the hex field, with or without its `#`.
pub fn parse_hex(text: &str) -> Result<Rgba, String> {
    let digits = text.trim().trim_start_matches('"').trim_start_matches('#');
    if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Hex has only 0 to 9 and a to f after the #.".to_string());
    }
    let nibble = |i: usize| u8::from_str_radix(&digits[i..i + 1], 16).unwrap() * 17;
    let byte = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).unwrap();
    match digits.len() {
        3 => Ok([nibble(0), nibble(1), nibble(2), 255]),
        4 => Ok([nibble(0), nibble(1), nibble(2), nibble(3)]),
        6 => Ok([byte(0), byte(2), byte(4), 255]),
        8 => Ok([byte(0), byte(2), byte(4), byte(6)]),
        _ => Err("Hex has 3, 4, 6 or 8 digits after the #.".to_string()),
    }
}

/// The colour names in colour order: greys by lightness first, then the
/// rest by hue, in twelve slices of 30°, lightest first in each.
pub fn names_by_colour() -> Vec<(&'static str, Rgba)> {
    let mut names: Vec<_> = COLOUR_NAMES
        .iter()
        .map(|(name, rgb)| (*name, name_rgba(*rgb)))
        .collect();
    let key = |rgba: Rgba| {
        let [r, g, b, _] = rgba.map(|c| c as f64 / 255.);
        let (max, min) = (r.max(g).max(b), r.min(g).min(b));
        let lightness = (max + min) / 2.;
        let saturation = if max == min {
            0.
        } else {
            (max - min) / (1. - (2. * lightness - 1.).abs())
        };
        let grey = saturation < 0.12 || lightness < 0.06 || lightness > 0.97;
        let slice = (Hsva::from_rgba(rgba).h / 30.).round() as i64 % 12;
        let lightness = (lightness * 1000.).round() as i64;
        if grey {
            (0, lightness, 0)
        } else {
            (1, slice, -lightness)
        }
    };
    names.sort_by_key(|(name, rgba)| (key(*rgba), *name));
    names
}

/// The names with `filter` anywhere in them, in colour order.
pub fn names_matching(filter: &str) -> Vec<(&'static str, Rgba)> {
    let filter = filter.trim().to_lowercase();
    names_by_colour()
        .into_iter()
        .filter(|(name, _)| name.contains(&filter))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_and_back() {
        let orange = Hsva::from_rgba([255, 154, 38, 255]);
        assert!((orange.h - 32.1).abs() < 0.1, "{orange:?}");
        assert!((orange.s - 0.851).abs() < 0.001 && orange.v == 1.);
        for rgba in [
            [255, 154, 38, 255],
            [0, 0, 0, 255],
            [255, 255, 255, 0],
            [64, 160, 255, 120],
            [128, 0, 128, 255],
            [12, 200, 190, 7],
        ] {
            assert_eq!(Hsva::from_rgba(rgba).rgba(), rgba);
        }
        // Black keeps its hue while picking.
        let black = Hsva { v: 0., ..orange };
        assert_eq!(black.rgba(), [0, 0, 0, 255]);
        assert_eq!(Hsva { v: 1., ..black }.rgba(), [255, 154, 38, 255]);
    }

    #[test]
    fn the_wheel() {
        let close = |(x, y): (f64, f64), (ex, ey): (f64, f64)| {
            assert!((x - ex).abs() < 1e-9 && (y - ey).abs() < 1e-9, "{x}, {y}");
        };
        close(wheel_point(0., 1.), (0., -1.));
        close(wheel_point(90., 1.), (1., 0.));
        close(wheel_point(180., 0.5), (0., 0.5));
        close(wheel_point(123., 0.), (0., 0.));
        let (h, s) = wheel_colour(0., -1.);
        assert!(h.abs() < 1e-9 && (s - 1.).abs() < 1e-9);
        let (h, s) = wheel_colour(-3., 0.);
        assert!((h - 270.).abs() < 1e-9 && s == 1., "outside, at the edge");
        for (h, s) in [(32., 0.85), (210., 0.4), (359., 1.)] {
            let (x, y) = wheel_point(h, s);
            let (h2, s2) = wheel_colour(x, y);
            assert!((h - h2).abs() < 1e-9 && (s - s2).abs() < 1e-9);
        }
    }

    #[test]
    fn colours_as_text() {
        let tomato = [255, 99, 71, 255];
        assert_eq!(text(Form::Palette(4), [255, 0, 0, 255]), "4");
        assert_eq!(text(Form::Name("tomato"), tomato), "\"tomato");
        assert_eq!(text(Form::Rgb, tomato), "[255 99 71]");
        assert_eq!(text(Form::Rgba, [255, 99, 71, 128]), "[255 99 71 128]");
        assert_eq!(text(Form::Hex, tomato), "\"#ff6347");
        assert_eq!(text(Form::Hex, [255, 99, 71, 128]), "\"#ff634780");
    }

    #[test]
    fn exact_names_and_palette_numbers() {
        assert_eq!(name_of([255, 99, 71, 255]), Some("tomato"));
        assert_eq!(name_of([128, 128, 128, 255]), Some("gray"));
        assert_eq!(name_of([255, 99, 71, 254]), None, "not solid");
        assert_eq!(name_of([255, 99, 72, 255]), None);
        assert_eq!(palette_of([255, 0, 0, 255]), Some(4));
        assert_eq!(palette_of([155, 96, 59, 255]), Some(8));
        assert_eq!(
            forms([255, 0, 0, 255]),
            [
                Form::Palette(4),
                Form::Name("red"),
                Form::Rgb,
                Form::Rgba,
                Form::Hex
            ]
        );
        assert_eq!(forms([1, 2, 3, 4]), [Form::Rgb, Form::Rgba, Form::Hex]);
    }

    #[test]
    fn a_space_before_unless_it_isnt_needed() {
        assert_eq!(insertion(Some('C'), "4"), " 4");
        assert_eq!(insertion(Some(' '), "4"), "4");
        assert_eq!(insertion(Some('\n'), "4"), "4");
        assert_eq!(insertion(Some('['), "\"red"), "\"red");
        assert_eq!(insertion(None, "[1 2 3]"), "[1 2 3]");
    }

    #[test]
    fn what_the_fields_take() {
        assert_eq!(parse_channel(" 154 ", "G"), Ok(154));
        assert_eq!(parse_channel("12.6", "B"), Ok(13));
        assert_eq!(
            parse_channel("300", "R"),
            Err("R goes from 0 to 255.".into())
        );
        assert_eq!(parse_channel("", "A"), Err("A goes from 0 to 255.".into()));
        assert_eq!(parse_hex("#ff9a26"), Ok([255, 154, 38, 255]));
        assert_eq!(parse_hex("FF9A2680"), Ok([255, 154, 38, 128]));
        assert_eq!(parse_hex("\"#f64"), Ok([255, 102, 68, 255]));
        assert_eq!(parse_hex("#f648"), Ok([255, 102, 68, 136]));
        assert_eq!(
            parse_hex("#40a0f"),
            Err("Hex has 3, 4, 6 or 8 digits after the #.".into())
        );
        assert_eq!(
            parse_hex("#zz0000"),
            Err("Hex has only 0 to 9 and a to f after the #.".into())
        );
    }

    #[test]
    fn names_in_colour_order() {
        let names = names_by_colour();
        assert_eq!(names.len(), 148);
        assert_eq!(names[0].0, "black", "greys first, darkest first");
        let at = |n| names.iter().position(|(name, _)| *name == n).unwrap();
        assert!(at("white") < at("red"), "greys before colours");
        assert!(at("gray") + 1 == at("grey"), "spellings side by side");
        assert!(at("red") < at("yellow") && at("yellow") < at("blue"));
        let reds: Vec<_> = names_matching("RED").into_iter().map(|(n, _)| n).collect();
        assert_eq!(reds.len(), 6, "{reds:?}");
        assert!(reds.contains(&"palevioletred"));
    }
}
