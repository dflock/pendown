//! `RANDOM`'s numbers: a small PCG generator (O'Neill's PCG32), so the
//! interpreter needs no dependencies and a fixed seed always gives the same
//! drawing.

#[derive(Clone, Debug)]
pub struct Random {
    state: u64,
}

const MULTIPLIER: u64 = 6364136223846793005;
const INCREMENT: u64 = 1442695040888963407;

impl Random {
    pub fn new(seed: u64) -> Random {
        let mut r = Random { state: 0 };
        r.next();
        r.state = r.state.wrapping_add(seed);
        r.next();
        r
    }

    /// A seed that's different every time, for each Run (ADR 0006).
    pub fn fresh_seed() -> u64 {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64);
        let local = 0u8;
        nanos ^ (&local as *const u8 as u64).rotate_left(32)
    }

    fn next(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// A whole number from 0 to `n - 1`, without bias.
    pub fn below(&mut self, n: u64) -> u64 {
        let wide = ((self.next() as u64) << 32) | self.next() as u64;
        // Rejecting the top of the range, so every result is equally likely.
        let zone = u64::MAX - (u64::MAX % n);
        if wide < zone { wide % n } else { self.below(n) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_gives_the_same_numbers() {
        let a: Vec<_> = {
            let mut r = Random::new(42);
            (0..5).map(|_| r.below(100)).collect()
        };
        let b: Vec<_> = {
            let mut r = Random::new(42);
            (0..5).map(|_| r.below(100)).collect()
        };
        let c: Vec<_> = {
            let mut r = Random::new(43);
            (0..5).map(|_| r.below(100)).collect()
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn every_number_turns_up() {
        let mut r = Random::new(7);
        let mut seen = [0; 6];
        for _ in 0..6000 {
            seen[r.below(6) as usize] += 1;
        }
        assert!(seen.iter().all(|&n| (800..1200).contains(&n)), "{seen:?}");
    }
}
