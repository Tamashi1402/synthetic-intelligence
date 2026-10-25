//! docs/23 / engineering/12 §30 — silt: the warmth table that writes nothing.
//!
//! B14 ships this package with its **final edge** and no mechanism: `si-core`
//! only, so `si-silt` cannot name a row, a shelf, a store or an ask (§23.0's
//! litmus is a missing dependency, not a promise). B15 adds the table, the one
//! curve and the fifth ignition term; nothing before it may import this crate
//! for a value that does not exist yet.
//!
//! The whole mechanism is two mutators — [`Silt::tick`] (step 5a, before the
//! walk) and [`Silt::deposit`] (step 7's tail) — over a per-conversation table
//! keyed by point handles. The decay is `Clock::recency` **as a share**, the
//! same treatment `DIM_BASE` already gets, so R-29 keeps exactly one curve.
//! [`Silt::view`] is the one behavioural outlet and its only consumer is
//! `si-walk::ignite`'s seed choice; [`Silt::readout`] is the audit snapshot
//! `silt.jsonl` carries and is never an input (OD-103, `check_silt_ignition_only`).
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use si_core::clock;
use si_core::quant::{mul_q12_u16, u8_to_q12, Q12_ONE};

/// engineering/12 §5 §30: the five swept SPECIES rows, read once by the caller.
/// The default is the registry's own floors, which is the point every leg runs
/// at while `gates/G16/silt_sweep.json` is a refusal (OD-102, OD-83).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub lambda_silt: u8,
    pub level_max: u16,
    pub bump: u16,
    pub theta: u16,
    /// `SILT_MAX`: the audit bound. It is read out and never decides anything —
    /// no deposit is refused and no entry is evicted because of it (§23.3).
    pub max: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self { lambda_silt: 1, level_max: 1, bump: 1, theta: 0, max: 1 }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Table {
    levels: BTreeMap<u32, u16>,
    last_touch: BTreeMap<u32, i32>,
}

/// The per-conversation table as `silt.jsonl` and `mindlog.json`'s `silt_view{}`
/// show it: an owned snapshot, read by the audit trail and by nothing else.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Readout {
    pub conversation: u32,
    pub levels: Vec<(u32, u16)>,
    pub max: u16,
}

impl Readout {
    pub fn active(&self) -> usize {
        self.levels.len()
    }

    /// `SILT_MAX` **as a counter** (OD-102's declared build point): over bound is
    /// reported, never enforced.
    pub fn over_bound(&self) -> bool {
        self.levels.len() > self.max as usize
    }
}

/// The table. `BTreeMap<conversation, BTreeMap<point, level>>` (AB.4): warm-up is
/// per conversation, and a level is never summed across conversations.
#[derive(Clone, Debug)]
pub struct Silt {
    config: Config,
    conversations: BTreeMap<u32, Table>,
}

impl Silt {
    pub fn new(config: Config) -> Self {
        Self { config, conversations: BTreeMap::new() }
    }

    pub fn config(&self) -> Config {
        self.config
    }

    /// AB.1 step 5a, before the walk (law 27): erode the active conversation's
    /// table on the one curve and drop what has reached zero.
    ///
    /// `d = LAMBDA_SILT + (Q12_ONE − LAMBDA_SILT) · recency(now − last_touch)`,
    /// `level' = level · d`, and the entry goes when `level' < THETA_SILT` **or**
    /// when `level'` quantizes to `0`. The second clause is what keeps
    /// `THETA_SILT = 0` coherent: a level that rounds to zero Q12 contributes
    /// exactly zero (docs/23 §23.3's "below the floor … contributes exactly
    /// zero"), and leaving it in the table would make the audit claim warmth the
    /// sampler does not have.
    pub fn tick(&mut self, conversation: u32, now: i32) {
        let Some(table) = self.conversations.get_mut(&conversation) else {
            return;
        };
        let base = u8_to_q12(self.config.lambda_silt);
        let keep = Q12_ONE - base;
        let theta = self.config.theta;
        let Table { levels, last_touch } = table;
        let mut dropped = Vec::new();
        for (point, level) in levels.iter_mut() {
            let last = last_touch.get(point).copied().unwrap_or(now);
            let recency = clock::recency(now, last);
            let d = base.saturating_add(mul_q12_u16(keep, u8_to_q12(recency)));
            let next = mul_q12_u16(*level, d);
            if next < theta || next == 0 {
                dropped.push(*point);
            } else {
                *level = next;
            }
        }
        for point in dropped {
            levels.remove(&point);
            last_touch.remove(&point);
        }
        if levels.is_empty() {
            self.conversations.remove(&conversation);
        }
    }

    /// AB.1 step 7's tail: `level' = min(SILT_LEVEL_MAX, level + SILT_BUMP)` and
    /// `last_touch = now`. Deposit-after-decay is law 27's one-round lag, and it
    /// is why G16(d)'s drop order is strictly last-touch order.
    pub fn deposit(&mut self, conversation: u32, point: u32, now: i32) {
        let table = self.conversations.entry(conversation).or_default();
        let level = table.levels.get(&point).copied().unwrap_or(0);
        let next = level.saturating_add(self.config.bump).min(self.config.level_max);
        if next == 0 {
            return;
        }
        table.levels.insert(point, next);
        table.last_touch.insert(point, now);
    }

    /// AB.2: the one behavioural outlet, handed to `si-walk::ignite` as an
    /// argument (`si-walk` has no edge to this crate).
    pub fn view(&self, conversation: u32) -> Option<&BTreeMap<u32, u16>> {
        self.conversations.get(&conversation).map(|table| &table.levels)
    }

    /// The audit snapshot (OD-103). It is logged, never read by a decision.
    pub fn readout(&self, conversation: u32) -> Readout {
        let levels = self
            .conversations
            .get(&conversation)
            .map(|table| table.levels.iter().map(|(point, level)| (*point, *level)).collect())
            .unwrap_or_default();
        Readout { conversation, levels, max: self.config.max }
    }

    pub fn conversations(&self) -> Vec<u32> {
        self.conversations.keys().copied().collect()
    }

    pub fn level(&self, conversation: u32, point: u32) -> u16 {
        self.conversations
            .get(&conversation)
            .and_then(|table| table.levels.get(&point).copied())
            .unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.conversations.values().all(|table| table.levels.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AT_BOX_LO: Config =
        Config { lambda_silt: 1, level_max: 1, bump: 1, theta: 0, max: 1 };

    /// OD-102's declared point: at the box floors a level is exactly 1, and
    /// `mul_q12(1, 1)` is 0, so even an armed `IGN_WEIGHT_SILT = 1` adds a zero
    /// term. This is why G16(a.2) claims equality at the armed value.
    #[test]
    fn floor_levels_are_numerically_inert() {
        assert_eq!(mul_q12_u16(1, 1), 0);
        let mut silt = Silt::new(AT_BOX_LO);
        silt.deposit(0, 7, 0);
        assert_eq!(silt.level(0, 7), 1);
        assert_eq!(mul_q12_u16(1, silt.level(0, 7)), 0);
    }

    /// G16(d): three touches at known ticks then a long idle. Each entry drops at
    /// exactly `last_touch + 256` logical steps, so the drop order is strictly
    /// last-touch order — no level comparison anywhere.
    #[test]
    fn entries_clear_backwards_on_last_touch_order() {
        let mut silt = Silt::new(AT_BOX_LO);
        silt.deposit(0, 10, 0);
        silt.deposit(0, 11, 1);
        silt.deposit(0, 12, 2);
        let mut drops = BTreeMap::new();
        for now in 1..400 {
            silt.tick(0, now);
            for point in [10u32, 11, 12] {
                if silt.level(0, point) == 0 && !drops.contains_key(&point) {
                    drops.insert(point, now);
                }
            }
        }
        assert_eq!(drops.get(&10), Some(&256));
        assert_eq!(drops.get(&11), Some(&257));
        assert_eq!(drops.get(&12), Some(&258));
        assert!(silt.is_empty());
        assert_eq!(silt.view(0), None);
    }

    #[test]
    fn a_cold_table_contributes_nothing() {
        let mut silt = Silt::new(AT_BOX_LO);
        silt.deposit(0, 3, 0);
        for now in 1..=256 {
            silt.tick(0, now);
        }
        assert!(silt.is_empty());
        assert_eq!(silt.level(0, 3), 0);
        assert_eq!(silt.view(0), None);
    }

    #[test]
    fn ticking_an_absent_conversation_is_a_no_op() {
        let mut silt = Silt::new(Config { level_max: 2, ..AT_BOX_LO });
        silt.deposit(0, 1, 0);
        silt.tick(9, 1000);
        assert_eq!(silt.level(0, 1), 1);
        assert_eq!(silt.level(9, 1), 0);
        assert_eq!(silt.conversations(), vec![0]);
    }

    #[test]
    fn deposit_clamps_at_level_max() {
        let mut silt = Silt::new(Config { level_max: 3, bump: 2, ..AT_BOX_LO });
        silt.deposit(0, 5, 0);
        assert_eq!(silt.level(0, 5), 2);
        silt.deposit(0, 5, 1);
        assert_eq!(silt.level(0, 5), 3);
        silt.deposit(0, 5, 2);
        assert_eq!(silt.level(0, 5), 3);
    }

    /// AB.4: the same point in two conversations is two entries, and one
    /// conversation's decay never touches the other's level.
    #[test]
    fn conversations_are_keyed_separately() {
        let mut silt = Silt::new(AT_BOX_LO);
        silt.deposit(0, 4, 0);
        silt.deposit(1, 4, 0);
        for now in 1..=256 {
            silt.tick(0, now);
        }
        assert_eq!(silt.level(0, 4), 0);
        assert_eq!(silt.level(1, 4), 1);
    }

    #[test]
    fn readout_is_the_audit_snapshot_only() {
        let mut silt = Silt::new(AT_BOX_LO);
        silt.deposit(0, 8, 0);
        silt.deposit(0, 9, 0);
        let readout = silt.readout(0);
        assert_eq!(readout.active(), 2);
        assert!(readout.over_bound());
        assert_eq!(readout.levels, vec![(8, 1), (9, 1)]);
        // The readout is a snapshot: mutating the table does not rewrite it.
        silt.deposit(0, 10, 1);
        assert_eq!(readout.active(), 2);
    }

    /// 11 §B15 DoD 1's second clause, and the one branch no leg reaches at the
    /// shipped floors: `THETA_SILT`'s registry floor is `0` and the sweep that
    /// would move it is refused (OD-102, OD-83), so `next < theta` fires nowhere
    /// but here. `100` and `50` are scenario constants, not registry values, and
    /// no gate verdict may read them (11 §1 D5).
    ///
    /// The pair is what makes it a theta test rather than a decay test: the same
    /// table with `theta = 0` survives that tick holding a level above zero, so
    /// the entry left because it crossed the floor, not because it quantized away.
    #[test]
    fn a_level_below_theta_leaves_the_table_while_it_is_still_warm() {
        let level = 100;
        let theta = 50;
        let mut floored = Silt::new(Config { level_max: level, bump: level, theta, ..AT_BOX_LO });
        let mut unfloored = Silt::new(Config { level_max: level, bump: level, ..AT_BOX_LO });
        floored.deposit(0, 21, 0);
        unfloored.deposit(0, 21, 0);
        assert_eq!(floored.level(0, 21), level);

        let mut drop_at = None;
        for now in 1..1000 {
            floored.tick(0, now);
            unfloored.tick(0, now);
            if floored.level(0, 21) == 0 {
                drop_at = Some(now);
                break;
            }
        }
        let at_theta = drop_at.expect("a decaying level never reached THETA_SILT");
        let still_warm = unfloored.level(0, 21);
        assert!(still_warm > 0, "the floor clause did nothing at {at_theta}");
        assert!(still_warm < theta, "theta fired on a share above itself");

        // The same entry with no floor leaves on its own, and later: the gap is
        // the whole of what `THETA_SILT` decides.
        let mut zero_at = None;
        for now in at_theta + 1..1000 {
            unfloored.tick(0, now);
            if unfloored.level(0, 21) == 0 {
                zero_at = Some(now);
                break;
            }
        }
        assert!(zero_at.is_some_and(|now| now > at_theta), "theta did not decide: {zero_at:?}");
        assert_eq!(floored.view(0), None, "a dropped entry leaves no ghost in the view");
        assert_eq!(floored.readout(0).active(), 0, "and nothing for the audit trail");
    }
}
