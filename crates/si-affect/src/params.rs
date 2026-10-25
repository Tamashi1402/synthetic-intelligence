//! 03 §3–§4: the values `si-affect` runs on. TEMPERAMENT rows are drawn from
//! the individual; SPECIES rows are their OD-86 placeholders until a sweep
//! artifact replaces them. No value in this struct is chosen here.

use si_core::consts;
use si_core::temperament::{DerivId, Temperament};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LagUnit {
    PerToken,
    PerRound,
    PerThought,
}

impl LagUnit {
    /// 03 §3's enum order: `per_token | per_round | per_thought`.
    pub fn from_index(index: u8) -> Self {
        match index {
            0 => LagUnit::PerToken,
            1 => LagUnit::PerRound,
            2 => LagUnit::PerThought,
            other => panic!("LAG_UNIT {other} is not one of the three units"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub lambda_a: u8,
    pub gain: [u16; 6],
    pub beta: u16,
    pub gamma: u16,
    pub ask_cap_per_input: u8,
    pub priority: [u16; 3],
    pub gap_salience_floor: u16,
    pub dedupe_window: u16,
    pub bias_cap_share: u16,
    pub b_max: u16,
    pub k_tanh: u16,
    pub median_min_n: u16,
    pub median_shape: u8,
    pub lag_rounds: u16,
    pub lag_unit: LagUnit,
}

fn species(name: &str) -> u16 {
    consts::row(name)
        .and_then(consts::registry_value)
        .unwrap_or_else(|| panic!("{name} is not a scalar SPECIES row with a value"))
        .clamp(0, u16::MAX as i64) as u16
}

impl Params {
    pub fn from_temperament(t: &Temperament) -> Self {
        Self {
            lambda_a: t.get(DerivId::LambdaA).min(255) as u8,
            gain: [
                t.get(DerivId::Gain0),
                t.get(DerivId::Gain1),
                t.get(DerivId::Gain2),
                t.get(DerivId::Gain3),
                t.get(DerivId::Gain4),
                t.get(DerivId::Gain5),
            ],
            beta: t.get(DerivId::Beta),
            gamma: t.get(DerivId::Gamma),
            ask_cap_per_input: t.get(DerivId::AskCapPerInput).clamp(1, 4) as u8,
            priority: [
                t.get(DerivId::Priority0),
                t.get(DerivId::Priority1),
                t.get(DerivId::Priority2),
            ],
            gap_salience_floor: species("GAP_SALIENCE_FLOOR"),
            dedupe_window: species("DEDUPE_WINDOW"),
            bias_cap_share: species("BIAS_CAP_SHARE"),
            b_max: species("B_MAX"),
            k_tanh: species("K_TANH"),
            median_min_n: species("MEDIAN_MIN_N"),
            median_shape: species("MEDIAN_SHAPE").min(2) as u8,
            lag_rounds: species("MOD_LAG_ROUNDS"),
            lag_unit: LagUnit::from_index(species("LAG_UNIT").min(2) as u8),
        }
    }
}
