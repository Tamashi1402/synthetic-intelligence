//! 01 §5.3 / 04 §6: the six raw signals, and no seventh. A number outside the
//! six is a new privileged sensor — I8 — so `from_u8` panics on it.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum SignalKind {
    NoveltySpike = 0,
    RealizedGain = 1,
    Contradiction = 2,
    SettleFailure = 3,
    OpenQuestionAge = 4,
    IdleCascadeContradiction = 5,
}

impl SignalKind {
    pub const ALL: [SignalKind; 6] = [
        SignalKind::NoveltySpike,
        SignalKind::RealizedGain,
        SignalKind::Contradiction,
        SignalKind::SettleFailure,
        SignalKind::OpenQuestionAge,
        SignalKind::IdleCascadeContradiction,
    ];

    /// §18.1's six labels, for the audit's readout table; never a branch here.
    pub const fn name(self) -> &'static str {
        match self {
            SignalKind::NoveltySpike => "novelty-spike",
            SignalKind::RealizedGain => "realized-gain",
            SignalKind::Contradiction => "contradiction",
            SignalKind::SettleFailure => "settle-failure",
            SignalKind::OpenQuestionAge => "open-question-age",
            SignalKind::IdleCascadeContradiction => "idle-cascade-contradiction",
        }
    }

    pub const fn index(self) -> usize {
        self as usize
    }

    /// A signal number is a closed enum. Anything else would be a seventh input
    /// to `a`, which is the one thing I8 forbids naming, let alone feeding.
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => SignalKind::NoveltySpike,
            1 => SignalKind::RealizedGain,
            2 => SignalKind::Contradiction,
            3 => SignalKind::SettleFailure,
            4 => SignalKind::OpenQuestionAge,
            5 => SignalKind::IdleCascadeContradiction,
            other => panic!("signal {other} is not one of the six kinds (01 §5.3, I8)"),
        }
    }
}

/// One firing signal: the kind, and **its own value** (01 §5.3's `magnitude`,
/// Q12). It never carries `f(a)`, an emotion name, a source or a referent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Signal {
    pub kind: SignalKind,
    pub magnitude: u16,
}

impl Signal {
    pub const fn new(kind: SignalKind, magnitude: u16) -> Self {
        Self { kind, magnitude }
    }
}
