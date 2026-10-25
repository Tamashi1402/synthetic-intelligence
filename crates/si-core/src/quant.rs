//! 01 §3: quantization grids. Rounding is a multiply by a reciprocal, never a
//! divide (OD-45); round-half-up on ties, one rule everywhere.

pub const Q12_SHIFT: u32 = 12;
pub const Q12_ONE: u16 = 4096;
pub const Q12_MAX: u16 = 65535;
pub const U8_ONE: u8 = 255;

/// `(a · b) / 4096`, round-half-up, saturating at the u16 grid.
pub fn mul_q12_u16(a: u16, b: u16) -> u16 {
    let product = a as u64 * b as u64;
    let rounded = (product + (1 << (Q12_SHIFT - 1))) >> Q12_SHIFT;
    rounded.min(Q12_MAX as u64) as u16
}

/// The same rule on the signed activation grid.
pub fn mul_q12_i32(a: i32, b: i32) -> i32 {
    let product = a as i64 * b as i64;
    let rounded = if product >= 0 {
        (product + (1 << (Q12_SHIFT - 1))) >> Q12_SHIFT
    } else {
        -(((-product) + (1 << (Q12_SHIFT - 1))) >> Q12_SHIFT)
    };
    rounded as i32
}

/// 01 §3: genome weight is u8 on the 255 grid; Q12 form is `w << 4`.
pub const fn u8_to_q12(v: u8) -> u16 {
    (v as u16) << 4
}

/// Q12 to the 1/255 grid, reciprocal multiply and round-half-up.
pub fn q12_to_u8(v: u16) -> u8 {
    let scaled = v as u32 * 255 + 2048;
    (scaled >> Q12_SHIFT).min(255) as u8
}

/// Q12 to hundredths (confidence grid), reciprocal multiply and round-half-up.
pub fn q12_to_hundredths(v: u16) -> u16 {
    let scaled = v as u32 * 100 + 2048;
    ((scaled >> Q12_SHIFT).min(99)) as u16
}

/// Snap a non-negative value to a grid step. Registry/creation-time only; the
/// walk's per-round conversions use the reciprocal multiply above.
pub fn snap_to_grid(value: i64, grid: i64) -> i64 {
    if grid <= 1 {
        value
    } else {
        let half = grid / 2;
        ((value + half) / grid) * grid
    }
}

/// 10 §4 P.2: `fraction · (hi − lo)` rounded half up onto the row's grid, plus lo.
/// `fraction` is Q12 in `0..4096`; the result cannot land outside the range.
pub fn map_fraction(fraction: u16, lo: i64, hi: i64, grid: i64) -> i64 {
    debug_assert!(fraction < Q12_ONE, "fraction is a Q12 value below 1.0");
    let span = hi - lo;
    let delta = ((span * fraction as i64) + (1 << (Q12_SHIFT - 1))) >> Q12_SHIFT;
    let snapped = snap_to_grid(delta, grid);
    (lo + snapped).clamp(lo, hi)
}

/// 01 §3: a confidence is stored in hundredths and 100 is unrepresentable.
pub fn clamp_confidence(v: u16) -> u16 {
    v.min(crate::CONF_MAX)
}

/// 01 §3: arousal lives on 1/255, `0` = rest.
pub fn clamp_arousal(v: i32) -> u8 {
    v.clamp(0, 255) as u8
}
