//! 05 §4.1 + §19: the seed panel, and the artifact that makes "we checked the
//! whole box" auditable instead of a promise.
//!
//! Three things live here and none of them is a sweep of a constant:
//! * the **panel** — `harness/scenarios/g14_seed_panel.json`, ≥ 16 fixed `u64`s
//!   produced by one published rule. The rule is re-derived from the code on every
//!   read, so a hand-edited member is a named mismatch rather than a quiet
//!   cherry-pick (§4.1 step 4 forbids re-sampling the panel until a leg passes);
//! * the **index** — `gates/G14/seed_panel.json`, written per run: which seeds
//!   exist, and at each one which gates ran, in which run directory, with what
//!   derived-vector digest and what guard digest. Every run whose seed is a panel
//!   member adds its record, so the cross-process legs (§19(b)(d)) accumulate from
//!   ordinary runs rather than from a special mode;
//! * the **box arithmetic** — §19(e)'s corner reachability, computed exactly
//!   rather than by sampling, because the draw is a Q12 fraction: each row has at
//!   most 4096 reachable values and the set is enumerable.
//!
//! The digests recorded here are over canonical byte strings, never magnitudes:
//! 05 §3's honesty rule holds for a temperament vector as it does for an emotion.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use md5::{Digest, Md5};
use serde_json::{Value, json};
use si_store::snapshot::write_atomic;

use si_core::consts::{self, Class};
use si_core::quant::map_fraction;
use si_core::seed::derive_temperament;
use si_core::temperament::DERIV_ORDER;
use si_session::Session;

use super::runner::Run;
use super::Workspace;

/// 05 §19's prerequisite, checked in as an asset and hashed because everything
/// under `harness/` enters `binary_tree_md5` (`runner::tree_digest`).
pub const PANEL_ASSET: &str = "harness/scenarios/g14_seed_panel.json";
/// The artifact §4.1 names. `engineering/10 §11` and `05 §19` both forbid quoting
/// it before it exists; this module is the place that makes the quoting legal.
pub const PANEL_ARTIFACT: &str = "gates/G14/seed_panel.json";
/// 05 §19's shared transcript: the same words at every member.
pub const TWINS_SCENARIO: &str = "harness/scenarios/g14_twins.txt";
/// 03 §7's declared size floor, repeated here as the number the leg compares to.
pub const MIN_PANEL: usize = 16;
/// `si_core::quant`'s Q12: the draw's whole alphabet, and so the exact size of the
/// set the corner scan enumerates.
pub const FRACTION_ALPHABET: u32 = 1 << 12;

/// The published rule, in code. Member 1 and member 2 are the two seeds 05 §2
/// itself names (V0's reference `0x51`, V8's control twin `0x0`); members 3..18 are
/// `u64::from_le_bytes(md5("SI-panel-<k>")[0..8])` for `k = 0..15`.
pub fn rule_seeds() -> Vec<u64> {
    let mut out = vec![0x0000_0000_0000_0051u64, 0x0000_0000_0000_0000u64];
    for k in 0..16u32 {
        let digest = Md5::digest(format!("SI-panel-{k}").as_bytes());
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&digest[..8]);
        out.push(u64::from_le_bytes(bytes));
    }
    out
}

/// The panel as the asset declares it, checked against the rule rather than
/// trusted.
pub struct Panel {
    pub seeds: Vec<u64>,
    /// False when a member of the file is not the member the rule produces.
    pub rule_matches: bool,
    /// 03 §7's floor: `>= 16`.
    pub size_ok: bool,
    pub distinct_ok: bool,
}

impl Panel {
    pub fn member_of(&self, seed: u64) -> Option<usize> {
        self.seeds.iter().position(|s| *s == seed).map(|index| index + 1)
    }

    fn json(&self) -> Value {
        json!({
            "asset": PANEL_ASSET,
            "size": self.seeds.len(),
            "declared_floor": MIN_PANEL,
            "size_ok": self.size_ok,
            "members_are_distinct": self.distinct_ok,
            "rule_matches_file": self.rule_matches,
        })
    }
}

/// `None` when the asset is not in the tree at all — the state §19 called
/// UNMEASURED before B12, and the reason a leg stays open rather than failing.
pub fn load(ws: &Workspace) -> Result<Option<Panel>> {
    if !ws.exists(PANEL_ASSET) {
        return Ok(None);
    }
    let text = ws.read(PANEL_ASSET)?;
    let document: Value =
        serde_json::from_str(&text).with_context(|| format!("parse {PANEL_ASSET}"))?;
    let listed = document["seeds"]
        .as_array()
        .with_context(|| format!("{PANEL_ASSET} has no `seeds` array"))?;
    let mut seeds = Vec::with_capacity(listed.len());
    for entry in listed {
        let hex = entry
            .as_str()
            .with_context(|| format!("{PANEL_ASSET} lists a seed that is not a hex string"))?;
        seeds.push(
            u64::from_str_radix(hex.trim_start_matches("0x").trim_start_matches("0X"), 16)
                .with_context(|| format!("{PANEL_ASSET} lists `{hex}`, which is not a u64 seed"))?,
        );
    }
    let rule = rule_seeds();
    Ok(Some(Panel {
        size_ok: seeds.len() >= MIN_PANEL,
        distinct_ok: seeds.iter().collect::<std::collections::BTreeSet<_>>().len() == seeds.len(),
        rule_matches: seeds == rule,
        seeds,
    }))
}

// ---------------------------------------------------------------------------
// The two byte strings a member is identified by.
// ---------------------------------------------------------------------------

fn md5_hex(bytes: &[u8]) -> String {
    super::scenario::hex(Md5::digest(bytes).as_slice())
}

/// §19(c)(f)'s comparison target: every constant the **live session holds** that is
/// not one of the eighteen draws. The drawn half is deliberately absent —
/// `Params::from_temperament` fills `lambda_a`/`gain`/`beta`/`gamma`/`ask_cap_per_input`
/// /`priority` from the vector, so including them would make the guard digest move
/// with the seed by construction and the leg would prove nothing. If a future
/// refactor routes a draw into `bias_cap_share`, `b_max`, a floor or a tier, its
/// value appears in these bytes and every panel member's digest stops matching.
pub fn guard_bytes(session: &Session) -> String {
    let p = session.params();
    let w = session.walk();
    let s = session.sleep_config();
    let a = session.arc();
    let mut out = String::new();
    // The guard-bearing half of `si_affect::Params`, in field order.
    out.push_str(&format!(
        "gap_salience_floor={};dedupe_window={};bias_cap_share={};b_max={};k_tanh={};median_min_n={};median_shape={};lag_rounds={};lag_unit={:?};",
        p.gap_salience_floor,
        p.dedupe_window,
        p.bias_cap_share,
        p.b_max,
        p.k_tanh,
        p.median_min_n,
        p.median_shape,
        p.lag_rounds,
        p.lag_unit,
    ));
    out.push_str(&format!(
        "k_max={};tau_settle={};decay_base={};hub_exponent_q12={};frontier_cap={};oscillation={:?};",
        w.k_max, w.tau_settle, w.decay_base, w.hub_exponent_q12, w.frontier_cap, w.oscillation,
    ));
    out.push_str(&format!(
        "cadence_ticks={};max_walks={};dim_base={};floor_s_r_q12={};floor_facts_q12={};floor_links_q12={};stable_n={};",
        s.cadence_ticks, s.max_walks, s.dim_base, s.floor_s_r_q12, s.floor_facts_q12, s.floor_links_q12, s.stable_n,
    ));
    out.push_str(&format!(
        "ghost_max={};shape_max={};relight_max={};r_min={};r_ask={};delta_min={};crystallize={};silt_spread_priming={};episodic_traversal={};skills={};skill_composition={};lambda_silt={};silt_level_max={};silt_bump={};theta_silt={};silt_max={};ign_weight_silt={};prov_link_tug={};r_min_shape={};delta_min_shape={};k_filler={};world_tiers={};",
        a.ghost_max, a.shape_max, a.relight_max, a.r_min, a.r_ask, a.delta_min, a.crystallize as u8,
        a.silt_spread_priming as u8, a.episodic_traversal as u8, a.skills as u8, a.skill_composition as u8,
        a.lambda_silt, a.silt_level_max, a.silt_bump, a.theta_silt, a.silt_max, a.ign_weight_silt,
        a.prov_link_tug, a.r_min_shape, a.delta_min_shape, a.k_filler, a.world_tiers,
    ));
    // The two LAW rows the core block itself carries. `deriv_order_len` is the
    // vector's *length*, so it is law, not a draw; `birth_seed` is the individual
    // and is never part of this string.
    out.push_str(&format!(
        "seed_format={};deriv_order_len={};",
        session.core().seed_format, session.core().deriv_order_len
    ));
    out
}

pub fn guard_digest(session: &Session) -> String {
    md5_hex(guard_bytes(session).as_bytes())
}

/// 03 §6's LAW rows as one canonical byte string: `name=value;` in registry order.
/// `§19(f)`'s "every LAW row is byte-identical across all panel seeds" has two
/// halves, and this is the half no seed can touch — `install_launch_state` refuses
/// a LAW row by class, so it is recorded to prove the artifact's own records agree
/// with that, and to catch an artifact that quietly mixes two builds.
pub fn law_bytes() -> String {
    registry_law_snapshot()
        .iter()
        .map(|row| {
            format!(
                "{}={};",
                row["row"].as_str().unwrap_or_default(),
                row["value"]
            )
        })
        .collect()
}

pub fn law_digest() -> String {
    md5_hex(law_bytes().as_bytes())
}

/// The launch configuration this run was executed under, as one byte string: the
/// config name, the variant, every swept constant the run was launched with, the
/// interrupt boundary, and the pinned genome tag.
///
/// This is the key §19(f)'s guard half has to be compared under. `guard_bytes`
/// deliberately includes the Species and Ruling rows a launch *is allowed* to move
/// (`install_launch_state` refuses only a row of `Temperament` or `Law` class), so
/// two runs of the box at different constant points hold different guard surfaces
/// by design — OD-87's ruling is that "the same constant set" means the same birth
/// seed **and** the same launch. Comparing a plain-V0 run's guard against a swept
/// or variant run's is not a temperament leak; it is a category error, and it would
/// fail the leg for every battery run that is not the reference launch.
///
/// What stays comparable across launches, and is compared globally, is the LAW
/// digest: no launch and no seed can move a Law row.
pub fn launch_digest(run: &Run) -> String {
    let request = &run.request;
    let mut bytes = format!("config={};variant={};", request.config, request.variant);
    for (name, value) in &request.constants {
        bytes.push_str(&format!("{name}={value};"));
    }
    bytes.push_str(&format!(
        "sleep_interrupt={:?};genome_tag={:?};",
        request.sleep_interrupt, request.genome_tag
    ));
    md5_hex(bytes.as_bytes())
}

/// §19(b)(i)'s identification of an individual: the canonical vector bytes, as the
/// digest §19 names and as the hex a reviewer can re-derive from the seed alone.
pub fn vector_digest(session: &Session) -> String {
    format!("0x{:016x}", session.temperament().digest())
}

pub fn vector_hex(session: &Session) -> String {
    super::scenario::hex(&session.temperament().canonical_bytes())
}

// ---------------------------------------------------------------------------
// §19(e)'s box arithmetic, computed exactly.
// ---------------------------------------------------------------------------

/// Per drawn row: the declared box, the value set the Q12 draw can actually produce,
/// and whether each of the box's two corners is in it. `sampled` is never the word
/// here — the alphabet is 4096 fractions, so this is an enumeration.
pub fn box_reachability() -> Vec<Value> {
    let mut out = Vec::new();
    for id in DERIV_ORDER.iter() {
        let Some(row) = consts::row(id.registry_name()) else { continue };
        let mut reachable: Vec<i64> = Vec::with_capacity(FRACTION_ALPHABET as usize);
        for fraction in 0..FRACTION_ALPHABET {
            reachable.push(map_fraction(fraction as u16, row.lo, row.hi, row.grid));
        }
        let distinct = reachable.iter().copied().collect::<std::collections::BTreeSet<_>>().len();
        let min = reachable.iter().copied().min().unwrap_or(i64::MIN);
        let max = reachable.iter().copied().max().unwrap_or(i64::MAX);
        // The draw is unconditional, so the value the row takes is a function of the
        // fraction alone; a value between grid rows is impossible by `snap_to_grid`.
        let on_grid = reachable.iter().all(|v| (v - row.lo) % row.grid.max(1) == 0);
        let in_box = reachable.iter().all(|v| *v >= row.lo && *v <= row.hi);
        out.push(json!({
            "row": row.name,
            "declared_box": [row.lo, row.hi],
            "grid": row.grid,
            "reachable_distinct_values": distinct,
            "reachable_min": min,
            "reachable_max": max,
            "lo_corner_reachable": reachable.contains(&row.lo),
            "hi_corner_reachable": reachable.contains(&row.hi),
            "every_value_on_grid": on_grid,
            "every_value_in_box": in_box,
        }));
    }
    out
}

// ---------------------------------------------------------------------------
// The artifact.
// ---------------------------------------------------------------------------

pub fn read(ws: &Workspace) -> Result<Option<Value>> {
    let path = ws.path(PANEL_ARTIFACT);
    if !path.is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&text)
        .with_context(|| format!("parse {}", path.display()))
        .map(Some)
}

/// One member's record of one gate run: what individual it was, what guard bytes it
/// held, what hash its transcript left, and where the run lives. Called for every
/// run whose seed is a panel member, so the index says which gates have really been
/// seen at each individual (§4.1's audit claim).
pub fn record(ws: &Workspace, run: &Run, pass: bool) -> Result<()> {
    let Some(panel) = load(ws)? else { return Ok(()); };
    let Some(member) = panel.member_of(run.request.seed) else { return Ok(()); };
    let session = &run.sections[0].session;
    let entry = json!({
        "member": member,
        "gate": run.request.gate,
        "variant": run.request.variant,
        "scenario": run.request.scenario.path,
        "scenario_digest": run.request.scenario.digest,
        "state_hash": run.sections.iter().map(|section| section.hash.clone()).collect::<Vec<_>>(),
        "vector_digest": vector_digest(session),
        "vector_bytes_md5": md5_hex(&session.temperament().canonical_bytes()),
        "guard_digest": guard_digest(session),
        "law_digest": law_digest(),
        "launch_digest": launch_digest(run),
        "report_pass": pass,
        "run_dir": ws.relative(&run.dir).replace('\\', "/"),
        // The build that made this record. The artifact lives under the ignored
        // `gates/` tree and accumulates across rebuilds, so a panel comparison is
        // only honest within one binary: `runner::tree_digest` is the key.
        "tree_md5": super::runner::tree_digest(ws)?,
    });
    let mut document = read(ws)?.unwrap_or_else(|| json!({
        "artifact": PANEL_ARTIFACT,
        "law": "engineering/05 §4.1, §19's prerequisites; engineering/03 §7 (`seed_panel` size)",
        "rule": "si_cli::harness::panel::rule_seeds",
        "panel": panel.json(),
        "twins_scenario": TWINS_SCENARIO,
        "members": [],
    }));
    // The header is rewritten from the asset each time, so a panel file edited after
    // the fact is visible in the artifact rather than buried in an old copy.
    document["panel"] = panel.json();
    document["seeds"] = Value::Array(
        panel.seeds.iter().map(|seed| json!(format!("0x{seed:016x}"))).collect(),
    );
    let members = document["members"]
        .as_array_mut()
        .context("seed_panel.json has no `members` array")?;
    let seed_hex = format!("0x{:016x}", run.request.seed);
    if let Some(member_record) = members
        .iter_mut()
        .find(|record| record["seed_hex"].as_str() == Some(seed_hex.as_str()))
    {
        let runs = member_record["runs"]
            .as_array_mut()
            .context("a panel member has no `runs` array")?;
        // Idempotent: a re-verify of the same run directory never duplicates a row.
        if !runs.iter().any(|row| row["run_dir"] == entry["run_dir"]) {
            runs.push(entry);
        }
    } else {
        members.push(json!({
            "seed_hex": seed_hex,
            "member": member,
            "runs": [entry],
        }));
    }
    members.sort_by_key(|record| record["member"].as_u64().unwrap_or(0));
    write_file(ws, &document)
}

/// `sweep --gate G14` is the only caller that writes this document without a
/// session in hand: it re-writes the header from the asset and stamps
/// `sweep_validation` on the records ordinary runs accumulated.
pub fn write_document(ws: &Workspace, document: &Value) -> Result<()> {
    write_file(ws, document)
}

fn write_file(ws: &Workspace, document: &Value) -> Result<()> {
    let path = ws.path(PANEL_ARTIFACT);
    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent)?;
    }
    write_atomic(&path, serde_json::to_string_pretty(document)?.as_bytes())
}

/// §19(f)'s registry half: the scalar LAW rows as the registry declares them. These
/// are read at run time and are not session state — `install_launch_state` refuses a
/// LAW row by class, so no launch and no seed can move one. Recorded so the leg's
/// evidence says which half of the guard surface was measured across individuals and
/// which half is law by construction.
pub fn registry_law_snapshot() -> Vec<Value> {
    consts::all()
        .iter()
        .filter(|row| row.class == Class::Law)
        .map(|row| {
            json!({
                "row": row.name,
                "scalar": row.scalar,
                "law_value": row.law_value,
                "value": consts::registry_value(row),
            })
        })
        .collect()
}

/// §19(b)'s in-process half: the same seed derived twice is byte-identical, and the
/// vector is a pure function of `(birth_seed, seed_format, DERIV_ORDER)` — so a
/// derivation that consults a clock or entropy would show up here as a difference
/// between two calls in one process.
pub fn derivation_is_pure(seed: u64) -> bool {
    let a = derive_temperament(seed);
    let b = derive_temperament(seed);
    a.canonical_bytes() == b.canonical_bytes() && a.digest() == b.digest()
}
