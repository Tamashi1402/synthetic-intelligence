//! 10 §3–§4 / 03 §1.2: the sealed derivation order and the individual's vector.
//! The vector is never stored, only its digest; the 18 values are re-derived at
//! wake from `birth_seed` and `DERIV_ORDER` (P.3).

use md5::{Digest, Md5};

pub const DERIV_ORDER_LEN: usize = 18;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DerivId {
    LambdaA,
    Gain0,
    Gain1,
    Gain2,
    Gain3,
    Gain4,
    Gain5,
    ModCurveVariant,
    Beta,
    Gamma,
    AskCapPerInput,
    Priority0,
    Priority1,
    Priority2,
    IgnRecency,
    IgnBake,
    IgnDecayDeficit,
    IgnContradiction,
}

/// 03 §1.2's TEMPERAMENT rows in the registry's reading order:
/// §3 (lambda, six gains, curve variant, beta, gamma), §4 (ask cap, three
/// priority weights), §5 (four ignition weights).
pub const DERIV_ORDER: [DerivId; DERIV_ORDER_LEN] = [
    DerivId::LambdaA,
    DerivId::Gain0,
    DerivId::Gain1,
    DerivId::Gain2,
    DerivId::Gain3,
    DerivId::Gain4,
    DerivId::Gain5,
    DerivId::ModCurveVariant,
    DerivId::Beta,
    DerivId::Gamma,
    DerivId::AskCapPerInput,
    DerivId::Priority0,
    DerivId::Priority1,
    DerivId::Priority2,
    DerivId::IgnRecency,
    DerivId::IgnBake,
    DerivId::IgnDecayDeficit,
    DerivId::IgnContradiction,
];

impl DerivId {
    /// The registry row this draw fills, by the registry's own name.
    pub const fn registry_name(self) -> &'static str {
        match self {
            DerivId::LambdaA => "LAMBDA_A",
            DerivId::Gain0 => "GAIN_SIGNAL[0]",
            DerivId::Gain1 => "GAIN_SIGNAL[1]",
            DerivId::Gain2 => "GAIN_SIGNAL[2]",
            DerivId::Gain3 => "GAIN_SIGNAL[3]",
            DerivId::Gain4 => "GAIN_SIGNAL[4]",
            DerivId::Gain5 => "GAIN_SIGNAL[5]",
            DerivId::ModCurveVariant => "MOD_CURVE_VARIANT",
            DerivId::Beta => "BETA",
            DerivId::Gamma => "GAMMA",
            DerivId::AskCapPerInput => "ASK_CAP_PER_INPUT",
            DerivId::Priority0 => "PRIORITY_WEIGHTS[0]",
            DerivId::Priority1 => "PRIORITY_WEIGHTS[1]",
            DerivId::Priority2 => "PRIORITY_WEIGHTS[2]",
            DerivId::IgnRecency => "IGN_WEIGHT_RECENCY",
            DerivId::IgnBake => "IGN_WEIGHT_BAKE",
            DerivId::IgnDecayDeficit => "IGN_WEIGHT_DECAY_DEFICIT",
            DerivId::IgnContradiction => "IGN_WEIGHT_CONTRADICTION",
        }
    }
}

/// The 18 drawn values, canonical order, fixed grids. A second copy of this in
/// any container, export or cache is forbidden (P.3, G14(c)).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Temperament {
    values: [u16; DERIV_ORDER_LEN],
}

impl Temperament {
    pub const fn new(values: [u16; DERIV_ORDER_LEN]) -> Self {
        Self { values }
    }

    pub const fn values(&self) -> &[u16; DERIV_ORDER_LEN] {
        &self.values
    }

    pub fn get(&self, id: DerivId) -> u16 {
        let index = DERIV_ORDER.iter().position(|d| *d == id).expect("sealed id");
        self.values[index]
    }

    /// Fixed order, u16le per value: the canonical byte string of 10 §4 P.3.
    pub fn canonical_bytes(&self) -> [u8; DERIV_ORDER_LEN * 2] {
        let mut out = [0u8; DERIV_ORDER_LEN * 2];
        for (i, v) in self.values.iter().enumerate() {
            out[i * 2..i * 2 + 2].copy_from_slice(&v.to_le_bytes());
        }
        out
    }

    /// The digest recorded in the core block; the vector itself is not stored.
    pub fn digest(&self) -> u64 {
        let digest = Md5::digest(self.canonical_bytes());
        let mut out = [0u8; 8];
        out.copy_from_slice(&digest[..8]);
        u64::from_le_bytes(out)
    }
}
