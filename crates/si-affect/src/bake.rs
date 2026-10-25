//! 04 §16 P: `bake = B_max · tanh(K_TANH · a_formation)`, once, at the
//! formation instant. The crate has no float, so the curve is evaluated in
//! Q32 fixed point: halve until the Taylor series converges, then double back
//! through `tanh(2x) = 2t/(1+t²)`. The sweep artifact replaces the evaluation
//! with its 256-row table at B12; the law is this shape either way.

const FRAC: u32 = 32;
const ONE: i128 = 1 << FRAC;

fn mul(a: i128, b: i128) -> i128 {
    (a * b) >> FRAC
}

fn tanh_q32(x: i128) -> i128 {
    debug_assert!(x >= 0);
    if x == 0 {
        return 0;
    }
    if x >= 16 * ONE {
        return ONE;
    }
    let mut y = x;
    let mut doublings = 0u32;
    while y > (ONE >> 4) {
        y >>= 1;
        doublings += 1;
    }
    let y2 = mul(y, y);
    let mut power = y;
    let mut t = y;
    power = mul(power, y2);
    t -= power / 3;
    power = mul(power, y2);
    t += (power * 2) / 15;
    power = mul(power, y2);
    t -= (power * 17) / 315;
    power = mul(power, y2);
    t += (power * 62) / 2835;
    power = mul(power, y2);
    t -= (power * 1382) / 155925;
    for _ in 0..doublings {
        let t2 = mul(t, t);
        t = (2 * t * ONE) / (ONE + t2);
    }
    t
}

/// The one curve out of `a` into the write side (OD-33, OD-56). Exactly zero
/// at `a_formation = 0` is law, not tuning; `B_max` is the hard ceiling.
pub fn bake_q12(a_formation: u8, b_max: u16, k_tanh: u16) -> u16 {
    if a_formation == 0 {
        return 0;
    }
    let x = ((a_formation as i128 * k_tanh as i128) << FRAC) / (255 * 4096);
    let t = tanh_q32(x);
    let scaled = b_max as i128 * t + (ONE / 2);
    (scaled >> FRAC).clamp(0, b_max as i128) as u16
}
