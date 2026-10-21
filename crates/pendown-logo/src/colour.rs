//! Colours (ADR 0015): palette numbers, RGB and RGBA lists from 0 to 255,
//! CSS colour names and hex words, everywhere a colour is accepted.

use crate::value::Value;

/// RGBA, from 0 to 255.
pub type Rgba = [u8; 4];

/// UCBLogo's default palette, as its wxWidgets version has it.
pub const PALETTE: [Rgba; 16] = [
    [0, 0, 0, 255],
    [0, 0, 255, 255],
    [0, 255, 0, 255],
    [0, 255, 255, 255],
    [255, 0, 0, 255],
    [255, 0, 255, 255],
    [255, 255, 0, 255],
    [255, 255, 255, 255],
    [155, 96, 59, 255],
    [197, 136, 18, 255],
    [100, 162, 64, 255],
    [120, 187, 187, 255],
    [255, 149, 119, 255],
    [144, 113, 208, 255],
    [255, 163, 0, 255],
    [183, 183, 183, 255],
];

/// The 148 CSS named colours, in alphabetical order.
pub const NAMES: [(&str, u32); 148] = [
    ("aliceblue", 0xf0f8ff),
    ("antiquewhite", 0xfaebd7),
    ("aqua", 0x00ffff),
    ("aquamarine", 0x7fffd4),
    ("azure", 0xf0ffff),
    ("beige", 0xf5f5dc),
    ("bisque", 0xffe4c4),
    ("black", 0x000000),
    ("blanchedalmond", 0xffebcd),
    ("blue", 0x0000ff),
    ("blueviolet", 0x8a2be2),
    ("brown", 0xa52a2a),
    ("burlywood", 0xdeb887),
    ("cadetblue", 0x5f9ea0),
    ("chartreuse", 0x7fff00),
    ("chocolate", 0xd2691e),
    ("coral", 0xff7f50),
    ("cornflowerblue", 0x6495ed),
    ("cornsilk", 0xfff8dc),
    ("crimson", 0xdc143c),
    ("cyan", 0x00ffff),
    ("darkblue", 0x00008b),
    ("darkcyan", 0x008b8b),
    ("darkgoldenrod", 0xb8860b),
    ("darkgray", 0xa9a9a9),
    ("darkgreen", 0x006400),
    ("darkgrey", 0xa9a9a9),
    ("darkkhaki", 0xbdb76b),
    ("darkmagenta", 0x8b008b),
    ("darkolivegreen", 0x556b2f),
    ("darkorange", 0xff8c00),
    ("darkorchid", 0x9932cc),
    ("darkred", 0x8b0000),
    ("darksalmon", 0xe9967a),
    ("darkseagreen", 0x8fbc8f),
    ("darkslateblue", 0x483d8b),
    ("darkslategray", 0x2f4f4f),
    ("darkslategrey", 0x2f4f4f),
    ("darkturquoise", 0x00ced1),
    ("darkviolet", 0x9400d3),
    ("deeppink", 0xff1493),
    ("deepskyblue", 0x00bfff),
    ("dimgray", 0x696969),
    ("dimgrey", 0x696969),
    ("dodgerblue", 0x1e90ff),
    ("firebrick", 0xb22222),
    ("floralwhite", 0xfffaf0),
    ("forestgreen", 0x228b22),
    ("fuchsia", 0xff00ff),
    ("gainsboro", 0xdcdcdc),
    ("ghostwhite", 0xf8f8ff),
    ("gold", 0xffd700),
    ("goldenrod", 0xdaa520),
    ("gray", 0x808080),
    ("green", 0x008000),
    ("greenyellow", 0xadff2f),
    ("grey", 0x808080),
    ("honeydew", 0xf0fff0),
    ("hotpink", 0xff69b4),
    ("indianred", 0xcd5c5c),
    ("indigo", 0x4b0082),
    ("ivory", 0xfffff0),
    ("khaki", 0xf0e68c),
    ("lavender", 0xe6e6fa),
    ("lavenderblush", 0xfff0f5),
    ("lawngreen", 0x7cfc00),
    ("lemonchiffon", 0xfffacd),
    ("lightblue", 0xadd8e6),
    ("lightcoral", 0xf08080),
    ("lightcyan", 0xe0ffff),
    ("lightgoldenrodyellow", 0xfafad2),
    ("lightgray", 0xd3d3d3),
    ("lightgreen", 0x90ee90),
    ("lightgrey", 0xd3d3d3),
    ("lightpink", 0xffb6c1),
    ("lightsalmon", 0xffa07a),
    ("lightseagreen", 0x20b2aa),
    ("lightskyblue", 0x87cefa),
    ("lightslategray", 0x778899),
    ("lightslategrey", 0x778899),
    ("lightsteelblue", 0xb0c4de),
    ("lightyellow", 0xffffe0),
    ("lime", 0x00ff00),
    ("limegreen", 0x32cd32),
    ("linen", 0xfaf0e6),
    ("magenta", 0xff00ff),
    ("maroon", 0x800000),
    ("mediumaquamarine", 0x66cdaa),
    ("mediumblue", 0x0000cd),
    ("mediumorchid", 0xba55d3),
    ("mediumpurple", 0x9370db),
    ("mediumseagreen", 0x3cb371),
    ("mediumslateblue", 0x7b68ee),
    ("mediumspringgreen", 0x00fa9a),
    ("mediumturquoise", 0x48d1cc),
    ("mediumvioletred", 0xc71585),
    ("midnightblue", 0x191970),
    ("mintcream", 0xf5fffa),
    ("mistyrose", 0xffe4e1),
    ("moccasin", 0xffe4b5),
    ("navajowhite", 0xffdead),
    ("navy", 0x000080),
    ("oldlace", 0xfdf5e6),
    ("olive", 0x808000),
    ("olivedrab", 0x6b8e23),
    ("orange", 0xffa500),
    ("orangered", 0xff4500),
    ("orchid", 0xda70d6),
    ("palegoldenrod", 0xeee8aa),
    ("palegreen", 0x98fb98),
    ("paleturquoise", 0xafeeee),
    ("palevioletred", 0xdb7093),
    ("papayawhip", 0xffefd5),
    ("peachpuff", 0xffdab9),
    ("peru", 0xcd853f),
    ("pink", 0xffc0cb),
    ("plum", 0xdda0dd),
    ("powderblue", 0xb0e0e6),
    ("purple", 0x800080),
    ("rebeccapurple", 0x663399),
    ("red", 0xff0000),
    ("rosybrown", 0xbc8f8f),
    ("royalblue", 0x4169e1),
    ("saddlebrown", 0x8b4513),
    ("salmon", 0xfa8072),
    ("sandybrown", 0xf4a460),
    ("seagreen", 0x2e8b57),
    ("seashell", 0xfff5ee),
    ("sienna", 0xa0522d),
    ("silver", 0xc0c0c0),
    ("skyblue", 0x87ceeb),
    ("slateblue", 0x6a5acd),
    ("slategray", 0x708090),
    ("slategrey", 0x708090),
    ("snow", 0xfffafa),
    ("springgreen", 0x00ff7f),
    ("steelblue", 0x4682b4),
    ("tan", 0xd2b48c),
    ("teal", 0x008080),
    ("thistle", 0xd8bfd8),
    ("tomato", 0xff6347),
    ("turquoise", 0x40e0d0),
    ("violet", 0xee82ee),
    ("wheat", 0xf5deb3),
    ("white", 0xffffff),
    ("whitesmoke", 0xf5f5f5),
    ("yellow", 0xffff00),
    ("yellowgreen", 0x9acd32),
];

/// Why a value isn't a colour, for the error message.
#[derive(Debug, PartialEq)]
pub enum ColourError {
    /// Not any colour form. The caller says "X doesn't like Y as input".
    NotAColour,
    /// A word that isn't a known name, with the closest names.
    UnknownName { name: String, suggestions: Vec<&'static str> },
    BadHex(String),
}

impl ColourError {
    /// A message for the errors that need more than "doesn't like".
    pub fn message(&self) -> Option<String> {
        match self {
            ColourError::NotAColour => None,
            ColourError::UnknownName { name, suggestions } => Some(match suggestions.as_slice() {
                [] => format!("I don't know the colour \"{name}\""),
                [one] => format!("I don't know the colour \"{name}\". Did you mean \"{one}\"?"),
                [rest @ .., last] => {
                    let rest: Vec<_> = rest.iter().map(|s| format!("\"{s}\"")).collect();
                    format!("I don't know the colour \"{name}\". Did you mean {} or \"{last}\"?", rest.join(", "))
                }
            }),
            ColourError::BadHex(word) => {
                Some(format!("\"{word}\" isn't a hex colour: use 3, 4, 6 or 8 hex digits after the #, as in \"#ff8800"))
            }
        }
    }
}

/// Reads a colour in any of its forms. The forms can't be confused: a number
/// is a palette index, a list is RGB or RGBA, a word starting with `#` is
/// hex, and any other word is a name.
#[cfg(test)]
pub fn parse(value: &Value) -> Result<Rgba, ColourError> {
    parse_with(value, &PALETTE)
}

/// Like `parse`, with the palette as `SETPALETTE` has left it.
pub fn parse_with(value: &Value, palette: &[Rgba; 16]) -> Result<Rgba, ColourError> {
    if let Some(n) = value.as_num() {
        return if n.fract() == 0.0 && (0.0..16.0).contains(&n) { Ok(palette[n as usize]) } else { Err(ColourError::NotAColour) };
    }
    match value {
        Value::List(list) => {
            let mut rgba = [0, 0, 0, 255];
            if !(3..=4).contains(&list.items.len()) {
                return Err(ColourError::NotAColour);
            }
            for (c, item) in rgba.iter_mut().zip(&list.items) {
                match item.as_num() {
                    Some(n) if (0.0..=255.0).contains(&n) => *c = n.round() as u8,
                    _ => return Err(ColourError::NotAColour),
                }
            }
            Ok(rgba)
        }
        Value::Word(word) if word.starts_with('#') => hex(&word[1..]).ok_or_else(|| ColourError::BadHex(word.to_string())),
        Value::Word(word) => {
            let lower = word.to_lowercase();
            match NAMES.binary_search_by(|(name, _)| name.cmp(&lower.as_str())) {
                Ok(i) => {
                    let rgb = NAMES[i].1;
                    Ok([(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 255])
                }
                Err(_) => Err(ColourError::UnknownName { name: word.to_string(), suggestions: suggest(&lower) }),
            }
        }
        Value::Num(_) | Value::Array(_) => Err(ColourError::NotAColour),
    }
}

fn hex(digits: &str) -> Option<Rgba> {
    if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let nibble = |i: usize| u8::from_str_radix(&digits[i..i + 1], 16).ok().map(|n| n * 17);
    let byte = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).ok();
    match digits.len() {
        3 => Some([nibble(0)?, nibble(1)?, nibble(2)?, 255]),
        4 => Some([nibble(0)?, nibble(1)?, nibble(2)?, nibble(3)?]),
        6 => Some([byte(0)?, byte(2)?, byte(4)?, 255]),
        8 => Some([byte(0)?, byte(2)?, byte(4)?, byte(6)?]),
        _ => None,
    }
}

/// The known names closest to an unknown one: up to three, within an edit
/// distance of a third of its length (but at least 2), or containing it.
fn suggest(name: &str) -> Vec<&'static str> {
    let limit = (name.chars().count() / 3).max(2);
    let mut close: Vec<_> = NAMES
        .iter()
        .map(|(known, _)| (distance(name, known), *known))
        .filter(|(d, known)| *d <= limit || (name.len() >= 4 && known.contains(name)))
        .collect();
    close.sort();
    close.into_iter().take(3).map(|(_, known)| known).collect()
}

/// Levenshtein distance.
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut previous = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let substitute = previous + usize::from(ca != *cb);
            previous = row[j + 1];
            row[j + 1] = substitute.min(row[j] + 1).min(row[j + 1] + 1);
        }
    }
    row[b.len()]
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::value::List;

    fn list(items: &[f64]) -> Value {
        Value::List(Rc::new(List::new(items.iter().map(|n| Value::Num(*n)).collect())))
    }

    #[test]
    fn every_form() {
        assert_eq!(parse(&Value::Num(4.0)), Ok([255, 0, 0, 255]));
        assert_eq!(parse(&Value::word("4")), Ok([255, 0, 0, 255]), "numbers are words");
        assert_eq!(parse(&list(&[10.0, 20.0, 30.0])), Ok([10, 20, 30, 255]));
        assert_eq!(parse(&list(&[10.0, 20.0, 30.0, 128.0])), Ok([10, 20, 30, 128]));
        assert_eq!(parse(&Value::word("CornflowerBlue")), Ok([100, 149, 237, 255]));
        assert_eq!(parse(&Value::word("#ff8800")), Ok([255, 136, 0, 255]));
        assert_eq!(parse(&Value::word("#f80")), Ok([255, 136, 0, 255]));
        assert_eq!(parse(&Value::word("#ff880080")), Ok([255, 136, 0, 128]));
        assert_eq!(parse(&Value::word("#f808")), Ok([255, 136, 0, 136]));
    }

    #[test]
    fn not_colours() {
        assert_eq!(parse(&Value::Num(16.0)), Err(ColourError::NotAColour));
        assert_eq!(parse(&Value::Num(2.5)), Err(ColourError::NotAColour));
        assert_eq!(parse(&list(&[1.0, 2.0])), Err(ColourError::NotAColour));
        assert_eq!(parse(&list(&[1.0, 2.0, 300.0])), Err(ColourError::NotAColour));
        assert_eq!(parse(&Value::word("#12")), Err(ColourError::BadHex("#12".into())));
        assert_eq!(parse(&Value::word("#ggg")), Err(ColourError::BadHex("#ggg".into())));
    }

    #[test]
    fn names_are_sorted_for_searching() {
        assert!(NAMES.windows(2).all(|w| w[0].0 < w[1].0));
    }

    #[test]
    fn suggestions() {
        let message = |w: &str| parse(&Value::word(w)).unwrap_err().message().unwrap();
        assert_eq!(message("bleu"), "I don't know the colour \"bleu\". Did you mean \"blue\"?");
        assert_eq!(message("gren"), "I don't know the colour \"gren\". Did you mean \"green\", \"grey\" or \"gray\"?");
        assert_eq!(message("cornflower"), "I don't know the colour \"cornflower\". Did you mean \"cornflowerblue\"?");
        assert_eq!(message("zzzzzz"), "I don't know the colour \"zzzzzz\"");
    }
}
