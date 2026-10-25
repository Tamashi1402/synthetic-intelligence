//! 04 §7 / 01 §8: the one integrator. `a` is a u8 on the 1/255 grid; decay
//! happens on `LAG_UNIT`'s unit, gains arrive per signal event, and every value
//! the integrator produces is appended to the trajectory, because the whole
//! sequence is the hashed artifact (G5(a)), not its endpoint.

use si_core::quant::{mul_q12_u16, q12_to_u8, u8_to_q12};

use crate::params::{LagUnit, Params};
use crate::signal::Signal;

/// What a call produced and when the walk may conduct with it (04 §5 step 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lagged {
    pub a: u8,
    pub applies_at: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Integrator {
    a: u8,
    thought: Option<u32>,
}

impl Integrator {
    /// §7: at t=0 `a` is 0 because nothing has fired. A non-zero birth value
    /// would be smuggled content.
    pub const fn at_birth() -> Self {
        Self { a: 0, thought: None }
    }

    pub const fn a(&self) -> u8 {
        self.a
    }

    /// One call is one round boundary (04 §7). `signals` are in gather order
    /// (§6): round ascending, then the class column. Empty input still decays
    /// when the unit says the boundary owns a decay; it never resets.
    pub fn update(
        &mut self,
        signals: &[Signal],
        round: u16,
        thought: u32,
        params: &Params,
        trajectory: &mut Vec<(u16, u8)>,
    ) -> Lagged {
        let decay_each = matches!(params.lag_unit, LagUnit::PerToken);
        let decay_once = match params.lag_unit {
            LagUnit::PerToken => signals.is_empty(),
            LagUnit::PerRound => true,
            LagUnit::PerThought => self.thought != Some(thought),
        };
        self.thought = Some(thought);

        if decay_once {
            self.a = decay(self.a, params.lambda_a);
        }

        let mut any = false;
        for signal in signals {
            if decay_each {
                self.a = decay(self.a, params.lambda_a);
            }
            let gain = mul_q12_u16(params.gain[signal.kind.index()], signal.magnitude);
            self.a = self.a.saturating_add(q12_to_u8(gain));
            trajectory.push((round, self.a));
            any = true;
        }
        if !any && decay_once {
            trajectory.push((round, self.a));
        }
        Lagged { a: self.a, applies_at: round.saturating_add(params.lag_rounds) }
    }

    /// The value a resumed session carries. The trajectory rides the snapshot.
    pub fn restore(a: u8) -> Self {
        Self { a, thought: None }
    }
}

/// `a · λ_a` on the 1/255 grid, reciprocal multiply and round-half-up. One λ for
/// all six signals; a second one would be a second integrator (I8).
fn decay(a: u8, lambda: u8) -> u8 {
    let lambda_q12 = ((lambda as u32 * 4096) / 255) as u16;
    q12_to_u8(mul_q12_u16(u8_to_q12(a), lambda_q12))
}
