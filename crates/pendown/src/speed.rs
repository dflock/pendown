//! Speed levels (ADR 0011).

use crate::animator::Speed;

/// The top level, which is Instant.
pub const INSTANT_LEVEL: u8 = 11;

/// The speed for a level from 1 to 11. Below 11, each level doubles the speed
/// of the one before, from 5 turtle steps a second at level 1 to 2,560 at
/// level 10.
pub fn level_speed(level: u8) -> Speed {
    let level = level.clamp(1, INSTANT_LEVEL);
    if level == INSTANT_LEVEL {
        Speed::Instant
    } else {
        Speed::Animated(5.0 * 2f64.powi(level as i32 - 1))
    }
}

/// The nearest level to a speed.
pub fn speed_level(speed: Speed) -> u8 {
    match speed {
        Speed::Instant => INSTANT_LEVEL,
        Speed::Animated(steps) => ((steps / 5.0).log2() + 1.0)
            .round()
            .clamp(1.0, (INSTANT_LEVEL - 1) as f64) as u8,
    }
}

/// How the slider describes a speed: "Speed 5 · 80 steps/s".
pub fn describe(speed: Speed) -> String {
    match speed {
        Speed::Instant => format!("Speed {INSTANT_LEVEL} · Instant"),
        Speed::Animated(steps) => format!("Speed {} · {steps:.0} steps/s", speed_level(speed)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels() {
        assert_eq!(level_speed(1), Speed::Animated(5.0));
        assert_eq!(level_speed(5), Speed::Animated(80.0));
        assert_eq!(level_speed(10), Speed::Animated(2560.0));
        assert_eq!(level_speed(11), Speed::Instant);
        assert_eq!(level_speed(0), Speed::Animated(5.0), "clamped");
        for level in 1..=11 {
            assert_eq!(speed_level(level_speed(level)), level);
        }
        // Nearest on the doubling scale: 100 is nearer 80 than 160.
        assert_eq!(speed_level(Speed::Animated(100.0)), 5);
        assert_eq!(speed_level(Speed::Animated(130.0)), 6);
        assert_eq!(describe(level_speed(5)), "Speed 5 · 80 steps/s");
        assert_eq!(describe(Speed::Instant), "Speed 11 · Instant");
    }
}
