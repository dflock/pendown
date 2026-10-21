//! Numbers are `f64`, and print like C's `%.15g` (ADR 0002).

/// Formats a number the way Logo prints it: whole numbers without a decimal
/// point, others with up to 15 significant digits and no trailing zeros, and
/// exponent notation below 1e-4 and from 1e15.
pub fn format_number(x: f64) -> String {
    if x == 0.0 {
        // Includes -0, which would otherwise print as "-0".
        return "0".into();
    }
    let sci = format!("{x:.14e}");
    let (mantissa, exp) = sci.split_once('e').expect("{:e} always has an exponent");
    let exp: i32 = exp.parse().expect("exponent is an integer");
    if !(-4..15).contains(&exp) {
        let sign = if exp < 0 { '-' } else { '+' };
        format!("{}e{sign}{:02}", trim_zeros(mantissa), exp.abs())
    } else {
        trim_zeros(&format!("{x:.*}", (14 - exp) as usize)).to_string()
    }
}

fn trim_zeros(s: &str) -> &str {
    if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.') } else { s }
}

/// Reads a word as a number, if it looks like one: an optional minus sign,
/// digits with at most one decimal point, and an optional exponent. Unlike
/// `str::parse`, this rejects `inf`, `nan` and a leading `+`.
pub fn parse_number(s: &str) -> Option<f64> {
    let b = s.as_bytes();
    let mut i = usize::from(b.first() == Some(&b'-'));
    let mut digits = 0;
    let mut dot = false;
    while i < b.len() {
        match b[i] {
            b'0'..=b'9' => digits += 1,
            b'.' if !dot => dot = true,
            _ => break,
        }
        i += 1;
    }
    if digits == 0 {
        return None;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        i += 1;
        if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
            i += 1;
        }
        let exp_start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == exp_start {
            return None;
        }
    }
    if i != b.len() {
        return None;
    }
    s.parse().ok().filter(|x: &f64| x.is_finite())
}

/// The sine and cosine of an angle in degrees, exact at multiples of 90°,
/// so `RT 90 FD 100` lands on whole numbers rather than `6.1e-15` off them.
pub fn sin_cos_degrees(degrees: f64) -> (f64, f64) {
    let d = degrees.rem_euclid(360.0);
    match d {
        0.0 => (0.0, 1.0),
        90.0 => (1.0, 0.0),
        180.0 => (0.0, -1.0),
        270.0 => (-1.0, 0.0),
        _ => d.to_radians().sin_cos(),
    }
}

/// Compares numbers rounded to 15 significant digits, so two numbers are
/// equal exactly when they print the same: `0.1 + 0.2 = 0.3` is true.
/// `<` and `>` use the same rounding, so they agree with `=`.
pub fn compare(x: f64, y: f64) -> std::cmp::Ordering {
    // Whole numbers below 2^53 are exact, and the common case in loops.
    let exact = |n: f64| n.fract() == 0.0 && n.abs() < 9007199254740992.0;
    if x == y || (exact(x) && exact(y)) {
        return x.total_cmp(&y);
    }
    round15(x).total_cmp(&round15(y))
}

fn round15(x: f64) -> f64 {
    format!("{x:.14e}").parse().expect("{:e} output parses")
}
