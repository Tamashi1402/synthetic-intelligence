//! 04 §5 step 4 / 03 §2: `MODCurve` is an artifact (`G9/arousal_sweep.json`,
//! leg 16a), a family of 256-level maps. This crate indexes the artifact and
//! supplies no value of its own; a fixture curve keeps a scenario executable
//! but no gate verdict depends on one (11 §1 D5).

use anyhow::{bail, Result};

use si_core::quant::Q12_ONE;

/// One `a` level of the swept curve.
///
/// `reach` is the per-source outgoing-edge cap (04 §27: per source, never a
/// per-round global cap). `decay_q12` is the fraction of activation this round
/// retains before the settle test; it is a multiplier on `DECAY_BASE`, so a
/// lower value is faster cooling — the registry's "decay ↑ with a" is the
/// steepness map, which the artifact loader turns into this retention.
/// `settle_shift_q12` is added to `TAU_SETTLE`; a negative shift lowers settle
/// dominance and keeps the walk spreading (04 §5 step 11).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CurvePoint {
    pub reach: u16,
    pub decay_q12: u16,
    pub settle_shift_q12: i32,
}

/// 256 rows, one per `a` level. No `Default`: a curve with no artifact is not
/// a neutral curve, it is an absent one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Curve {
    points: Vec<CurvePoint>,
}

impl Curve {
    pub fn from_points(points: Vec<CurvePoint>) -> Result<Self> {
        if points.len() != 256 {
            bail!("MODCurve has {} levels, expects 256 (03 §2)", points.len());
        }
        for (a, point) in points.iter().enumerate() {
            if point.decay_q12 > Q12_ONE {
                bail!("MODCurve[{a}].decay is {} on the Q12 grid; retention cannot exceed 1.0", point.decay_q12);
            }
        }
        Ok(Self { points })
    }

    /// A fixture curve: every level the same row. Explicitly caller-supplied;
    /// si-walk defines no curve value (11 §1 D5).
    pub fn fixture(point: CurvePoint) -> Self {
        Self { points: vec![point; 256] }
    }

    pub fn at(&self, a: u8) -> CurvePoint {
        self.points[a as usize]
    }
}
