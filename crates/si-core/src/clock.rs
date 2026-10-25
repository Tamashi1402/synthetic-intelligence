//! 04 §10: `Clock::recency` — the only decay/recency arithmetic in the build.
//! Every age-driven family (rehearsal deficit, ignition, median weights, gap
//! priority, open-question age, sleep dimming) calls this one function; no
//! crate may define a second curve (R-29, `check_one_clock`).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Clock {
    now: i32,
}

impl Clock {
    pub const fn new(now: i32) -> Self {
        Self { now }
    }

    pub const fn now(&self) -> i32 {
        self.now
    }

    pub fn advance(&mut self) -> i32 {
        self.now = self.now.saturating_add(1);
        self.now
    }

    pub fn set_now(&mut self, now: i32) {
        self.now = now;
    }
}

/// A logical age in steps to a weight on the 1/255 grid. Monotone decreasing;
/// age 0 is 255. The curve reads logical steps only, never wall-clock (I5).
pub fn recency_age(age: u32) -> u8 {
    let denom = 255u64 + age as u64;
    let scaled = 255u64 * 255u64;
    ((scaled + denom / 2) / denom).min(255) as u8
}

/// The same curve read as an age between two logical steps. A `last` later than
/// `now` is clamped to age 0; it cannot happen in a commit-ordered lattice.
pub fn recency(now: i32, last: i32) -> u8 {
    let age = now.saturating_sub(last).max(0) as u32;
    recency_age(age)
}
