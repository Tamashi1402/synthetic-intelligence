# Failure report — B7 (answered), B8 (open), B12 (the sweeps refuse; G11's composition unreachable), B13 (two walls named), B20 (two legs re-graded, ceiling ruled), B13 re-run (19 families green, OD-196)

## B7 — idle thought + the monologue lane

This file recorded B7's **walls** — legs that could not be greened without a design
decision that is not the build's to make — per the rule "a wall becomes
`failure_report.md` + the OD question, never a silent workaround." The owner answered the
batch on 2026-10-23: **OD-87 … OD-93**
([docs/decisions/open-decisions.md](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/docs/decisions/open-decisions.md#L1364)).
Three of the four walls are closed by ruling and green in this build; the fourth — sweep
enumeration — stays open and blocks **B12, not B7**. The report is kept rather than deleted
so each wall's answer, the code that implements it and the measurement that grades it sit in
one place. Every number below is from the runs this build produced, not from the runs that
raised the walls.

## Wall 1 → it was two walls, and they got different answers

**1a — the enumeration wall: STILL OPEN (OD-83, standing since B2).**
`si-harness sweep --gate G10` writes its artifact and **refuses**
([gates/G10/ignition_sweep.json](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/gates/G10/ignition_sweep.json):
`status: refused`, `curve: []`, `winner: null`, `placeholders_replaced: []`). After OD-87
the axis set is **1**, not 5:

| gate | axes its runner owns | open-bound axes | full cross product |
|---|---|---|---|
| G2 | 6 | `TAU_SETTLE`, `HUB_DAMP_EXPONENT`, `FRONTIER_CAP` | 18,516,314,440,531,700,690,625 points |
| G8 | 2 | `GAP_SALIENCE_FLOOR`, `DEDUPE_WINDOW` | 4,294,901,760 points |
| G9 | 6 | `B_MAX`, `K_TANH`, `MOD_CURVE_VARIANTS` | 221,296,755,218,306,040,000 points |
| G10 | **1** (`IDLE_PER_HOUR`, `class: "Species"`, open bound) | 1 | 65,535 points |

The row's `hi` is the type maximum (`u16::MAX`), not a design ceiling, so the axis has no
last point; 05 §4 step 1 names no per-axis search rule (cross product / one-factor-at-a-time
/ declared sub-grid); and the row's current value is OD-86's `UNMEASURED_PLACEHOLDER`
(32768), so there is no baseline to sweep against either. Choosing an enumeration is a
design decision, so the runner refuses. **B12 cannot begin until this is ruled.**

**1b — the class wall (OPEN-25): CLOSED by OD-87.** The ruling splits the row DoD 1 named:
the decay-deficit **drive** ("the mind drifts toward what it is about to forget") is SPECIES
mechanism and gets **no registry row of its own**, while the four `IGN_WEIGHT_*`
**multipliers** are TEMPERAMENT draws that **no sweep may ever touch**; "same constant set"
in G10(e) means **same birth seed** — instance identity is seed identity, the law G13's
divergence/identity pair already states. What the build did with it:

- `sweep::axes()` is Species-only by predicate and cannot select a drawn row, so the
  artifact enumerates `IDLE_PER_HOUR` alone (measured above); each axis now also carries its
  registry **true** `class` instead of a hardcoded "Species", so a future mis-classing is
  visible in the artifact rather than silent.
- One new check, `check_sweep_never_touches_temperament`
  ([crates/si-cli/src/harness/checks.rs](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/crates/si-cli/src/harness/checks.rs)),
  enabled from **B0** — where a structural law belongs, not where one stage's DoD happened
  to trip over it. Four legs: the OD-86 placeholder channel gives no TEMPERAMENT row a
  value; every axis enumerated under any gate prefix read off the registry's own sweep tags
  is `Class::Species` and is not a drawn name; the runner's launch-override path keeps its
  literal `row.class != Class::Species` guard; and each published `*_sweep.json` names no
  draw. Verdict on this build: **25 swept axes across 7 enumeration prefixes, every one
  SPECIES; the 18 TEMPERAMENT draws — the four `IGN_WEIGHT_*` among them — keep their gate
  tag as inert metadata only; 4 published artifacts name none of them.**
- The draws keep their `G10/ignition_sweep.json` tag **as inert metadata**, because
  `placeholders_match_registry`'s count identity (`swept_row_count == placeholder_count +
  non_scalar_swept_rows + temperament_count`) depends on the tag and because the value
  channel already refuses them by class. This is stated in
  [engineering/03-constants-registry.md](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/engineering/03-constants-registry.md)
  §1.2 and §5, where **OPEN-25 is recorded as closed by OD-87**. Drawn values at seed 0x51,
  from [gates/G10_si_20261023-1802/fingerprint.txt](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/gates/G10_si_20261023-1802/fingerprint.txt):
  `RECENCY 57423`, `BAKE 38991`, `DECAY_DEFICIT 52975`, `CONTRADICTION 50831`, each over
  `[0, 65535]`.
- Measured and reported rather than hidden: the three launched variants still carry
  byte-identical `constant_version_tags` (3114 bytes each). Under OD-87 with OD-93 that is
  **correct** — a launch variant is a build-graph state, never a registry row — so nothing
  in the tag vector is meant to change. DoD 1's "the winners replace their placeholders and
  the version tags change" half was rewritten in
  [engineering/11-build-plan.md](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/engineering/11-build-plan.md)
  §4 to say that a **written refusal satisfies the artifact item and is not a green sweep**.

## Wall 2 — G10.e.5: CLOSED by OD-91; one design sentence still open (OD-92)

The ruling: audit is observer-only **by law**, so V0 and V3 on the same transcript must
produce an **identical state hash**, and that identical hash **is** the pass condition. The
missing transcripts became real named artifacts instead of a manufactured difference being
graded:

| transcript | distinct bake values | V0 section hash | V3 section hash | rows / edges / stamps |
|---|---|---|---|---|
| `harness/scenarios/g10_high_bake.txt` (`34cd6a9583d72869d9036e3ef33cd59a`) | 2 (`2053`, `32768`) | `89f4a17ed140240a879d48fe6ccb4e91` | `89f4a17ed140240a879d48fe6ccb4e91` | identical |
| `harness/scenarios/g10_all_saturated.txt` (`61411f1722e7c9d400a8e2d648467659`) | 1 (`32768`) | `d116acfbc4eb1ccb2af6999b59124cdd` | `d116acfbc4eb1ccb2af6999b59124cdd` | identical |

`named_artifacts_missing: []`, `pair_differently_shaped: true` — they are a pair precisely
because the two bake profiles differ in shape, and the readout still cannot steer which edge
lights. `launched_transcript_v3_changed_behavior: false` means exactly what the harness
compares: ignition and monologue-entry **counts** (3 against 3, 6 entries against 6) did not
move; the **drawn point sequence is not compared** by that field.

The other half of 05 §15(e)'s sentence — the bake term leaving a point "over-represented
relative to V3" — demands a measurable difference at one seed, which the readout law
forbids. Per the ruling's last line, that text is **routed as its own open item, OD-92**,
and this build does not grade it: the leg reports the routing in
`over_representation_predicate` instead of inventing a number.

## Wall 3 — G10.a.3 / G10.e.2: CLOSED by OD-89 as a report; OD-90 stays open

The ruling said: verify the mechanism **before** proposing a constant, and report either
outcome. Both halves, measured on
[gates/G10_si_20261023-1802/gate_report.json](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/gates/G10_si_20261023-1802/gate_report.json):

- **(a) the rehearsal feedback was genuinely missing.** The writer 04 §S.7 names —
  `Clock::touch_edge_curve` — was not called from the idle cascade, so the decay map the
  sampler reads was empty by construction. Wiring it, plus the RAM-only `Lattice::touched`
  the read side needs, closed `G10.e.2` with **zero new constants**:
  `reset_narrowed_the_curve: true`, `curve_carried_a_deficit: true`, `ignitions_measured: 6`,
  and the traversed arc's deficit narrowing on the first ignition — `[64,16] → [32,16]` q12
  at t = 6 → 7 (arc 27, seed point 5) and holding `[32,16]` at t = 8; the second ignition's
  arc 2 reads `[32,16]` at t = 4/5/6 (seed point 2).
- **(b) with the feedback verified, the repetition persists.** `seed_points: [5,5,5,2,2,2]`,
  entries on ticks `[[6,7,8], [], [4,5,6]]`, `shared_ticks_within_a_section: []` over 7 quiet
  ticks. `G10.a.3` passes because the failure §15(a) names is a **budget** violation and none
  of the railed bounds (one entry per quiet tick, entries ≤ quiet ticks, no shared `t`) is
  crossed. What nothing rails is **novelty**: under the ruled max-aggregate a point's
  deficit is the furthest-drifted of its live arcs, so re-heating the arc the cascade
  traversed — `point_pull_risen_next_tick: 0`, the byte still moved — cannot lower the pull
  another arc holds up, and the same point can win the race on consecutive quiet ticks.
  **The proposal the ruling authorises is therefore raised, not applied**: per-node
  refractory `IDLE_REFRACTORY_TICKS` (SPECIES, named, swept, design ceiling per the
  enumeration ruling) versus the literal-*youngest* (min) reading of the same phrase —
  **OD-90**, owner's call. Both readings are now written into
  [engineering/04-algorithms.md](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/engineering/04-algorithms.md)
  §S.3, and neither adds a constant.

## Wall 4 — V3's version tag: CLOSED by OD-93

Ruling: a launch variant gets **no registry row, ever**; it is pinned by the deps-graph
digest; every run, replay and export record the variant id **and** the graph digest; the
determinism clause reads **per-variant**; cross-variant hash comparison is a category error,
not a failure. What landed: `harness::build_stamp()`
([crates/si-cli/src/harness/mod.rs:196](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/crates/si-cli/src/harness/mod.rs#L196))
is copied by `stamp_into`
([crates/si-cli/src/harness/runner.rs:834](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/crates/si-cli/src/harness/runner.rs#L834))
into every report header, the gate report's `build_identity` names `graph_digest` beside
`build_variant`, the CLI prints the pair above its stamped output and into `export.json`,
and `replay` re-executes at the variant the run itself recorded. Measured on this build: V0
replay 3/3 sections identical, V3 replay 3/3 sections identical, graph digest
`04d8ba8cde7088aaba0595f158cfe660` = `gates/deps_lock.md5`.

The OD-91/OD-93 tension is named, not smoothed: V0-against-V3 identity inside one leg is a
**designed control comparison**, not a determinism claim, and the ledger entry says so.

## What IS green for B7, on this build

- **G10 `pass: true` at all three variants**, seed 0x51, 25 legs each:
  [V0 `gates/G10_si_20261023-1802`](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/gates/G10_si_20261023-1802/gate_report.json)
  23 pass / 0 fail / 2 unmeasured;
  [V2 `gates/G10_si_20261023-1800`](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/gates/G10_si_20261023-1800/gate_report.json)
  22 / 0 / 3 — V2's `G10.e.2` is unmeasured *by construction*, an ignition-disabled launch
  has no curve to narrow;
  [V3 `gates/G10_si_20261023-1800-2`](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/gates/G10_si_20261023-1800-2/gate_report.json)
  23 / 0 / 2.
- **The rate law** — `G10.a.1` `idle_per_tick_law: 1`, per-section `{idle_ticks 3, thoughts
  3}`, `{1, 0}`, `{3, 3}`, `quiet_ticks: 7`; `G10.a.2` 6 thoughts against the ceiling 32768,
  reported with `ceiling_is: "UNMEASURED_PLACEHOLDER"` and `counted_in: "logical ticks,
  never wall clock"`.
- **The store claims, byte-level** — `G10.b.1` rows `row_count: 18` and digest
  `9bcb4b475b8fdd46a47fbc50b0865bd4` **identical** V0 vs V2 with ignitions 6 vs 0
  (`non_vacuous: true`); `G10.b.2` **as amended by OD-88** — stamps structural: `[8 records,
  128 bytes, be4605e21887e55893367dbc965e65c1]` in both variants, `edges_identical: true`,
  values free to move the way §S.7 licenses; `G10.b.3` edges
  `40bc17f876e2e2adc8a7b188706b191a` both, plus `check_idle_no_commit` and
  `check_bake_is_the_exception`. OD-88's required evidence — the G7 control pair — is in the
  ledger: rows 4032 B / edges 192 B / S-R envelope `811daffea525b8cc44c9927a500100c8`
  identical while the 208-byte stamp arena keeps its record set and changes digest
  (`58371b59…` V0 vs `8e6f9603…` V2). That is the case the old byte-strict assert forbade.
- **The lane** — `G10.c.1` `ignited_while_the_queue_had_work: false`,
  `revisited_on_pending_tick: [0,0,0,0,0,0,0]`; `G10.c.2` the owner line after a quiet tick
  still carries the ask (`open_asks_after_the_owner_line: 1`, `owner_line_output: "ask"`);
  `G10.d.1` `citations: [3,3,3,2,2,2]`, `defects: []` (every entry maps onto a point that
  lit); `G10.d.2` `idle_steps_carrying_an_owner_surface: 0`, renderer
  `si_front::render_monologue`; `G10.d.3` `foreign_tokens: []`; `G10.e.1` liveliness +
  artifact/launched consistency (V0 6/6, V2 0/0, V3 6/6); `G10.e.4` the probe — **bake weight
  alone drew point 5**, all four weights zeroed drew **nothing**, on a section-0 web with 3
  bake-carrying points and `max_bake 32768`, with `sampler_names_a_median: false` and
  `idle_cascade_passes_no_medians: true` beside it.
- **DoD 3** — `cargo test --workspace` exit 0: **178 passed, 0 failed** across 56 test
  binaries, including the five idle session tests in
  [crates/si-session/tests/idle.rs](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/crates/si-session/tests/idle.rs).
- **Exit checks** — `si-harness check --stage B7` exit 0: **28 checks, all pass**, 8 disabled
  (three turn on at B8, two at B9, three at B11), 26 placeholders
  ([gates/checks/stage-B7.json](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/gates/checks/stage-B7.json)).
- **No regression** — at V0/0x51: G1 21 legs/2 unresolved, G2 17/9, G3 14/2, G4 14/2,
  G7 17/2, G8 29/4, G9 24/12, G10 25/2, every one `pass: true`; stage checks B0 12, B1 15,
  B2 18, B3 19, B4 23, B5 27, B6 28, B7 28 — all PASS.

## Deferred, not walls (correctly `unmeasured`; they belong to later stages)

- `G10.e.3` — OPEN-28 is undecided and `TWO_REGIONS_TEST` is a Ruling with no value, so the
  cascade-side agenda push has no ruled producer; 04 §S.7 permits it only once that ruling
  lands, and the idle path writes no rows.
- `G14.f.1` — the 16-seed panel is a **B12** artifact; this run names one individual.
- `G2.e.1`, `G8.s.3`, `G9.c.3` sit in B12 for the reason they always have: OD-83 is unruled.

## Code-integrity notes carried forward

1. **V2 is the documented mechanism, not a flag the build invented.** 05 §2's V2 row is
   "rate bound set to zero in the harness launch state, loop code unchanged", so the session
   carries `idle.per_window_override: Option<u32>` and V2 sets `Some(0)`; the same
   `agenda_clear && within_rate` predicate still runs. V3 zeroes a **TEMPERAMENT draw at
   launch** and holds no registry row — OD-87 and OD-93 make that the ruling, and the
   launch path keeps the `row.class != Class::Species` guard that `check_sweep_never_touches_temperament`
   asserts is present.
2. **`state_hash` is not the store, and they are not conflated.** V0 and V2 differ in
   section hashes because ignition consumes the seeded stream and the cascade writes
   settle/trajectory entries, which I5 requires *inside* the hash; 05 §15(b)'s claim is about
   the belief store, so b.1/b.2/b.3 grade the arena digests. Claiming "state_hash identical"
   would have been a fabricated pass.
3. **`G10.e.1`'s first predicate was wrong and fixing it changed a verdict** — the old form
   could never hold on a control launch, whose artifact is empty by design. A bug fix, not a
   loosening: V2 still shows 0/0.
4. **`G10.e.4` is a probe, not a grep** — two `si_walk::ignite` calls on the reference's own
   section-0 graph at the run's own seed.
5. **The dependency graph did not change.** `Cargo.lock` md5
   `04d8ba8cde7088aaba0595f158cfe660` equals `gates/deps_lock.md5`. The code edits are the
   harness (checks/gates/runner/sweep/mod), `si-session`'s idle launch state, `si-store`'s
   rehearsal-curve wiring, three new scenarios and two new session tests.
6. **The design files moved because the owner ruled, not because the build wanted them to.**
   OD-87 … OD-93 are recorded in the ledger and folded into
   [engineering/03](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/engineering/03-constants-registry.md)
   §1.2/§5, [engineering/04](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/engineering/04-algorithms.md)
   §S.3, [engineering/05](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/engineering/05-gate-harness.md)
   §2/§15(b)/§15(e)/§20,
   [engineering/11](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/engineering/11-build-plan.md)
   §4 (B7 DoD + C0's exit-check set), plus the two docs sentences that carried the old
   over-representation wording (docs/13 G10(e), docs/20 §20.4a). No constant was invented and
   no value was filled by the build.

## Questions still with the owner

1. **OD-83 — sweep enumeration.** Which enumeration does a sweep runner own for {G2, G8,
   G9, G10}: full cross product, one axis at a time against a baseline, or a declared
   sub-grid; what is the baseline when every current value is an `UNMEASURED_PLACEHOLDER`;
   and may `hi` be a real design ceiling for open-bound rows. Blocks **B12**.
2. **OD-90 — what stops the same point re-igniting** (refractory SPECIES constant vs the
   literal-youngest aggregate vs leave it). G10 is green either way; the question is whether
   green is the behavior the design wants.
3. **OD-92 — 05 §15(e)'s "over-represented relative to V3".** Either it is design intuition
   the identity leg already discharges, or the readout law needs an exception. This build
   grades neither.

---

## B8 — sleep: three walls, all named, none worked around

B8's exit gate is **G13 + C8**, and both are green in this build
(`gates/G13_si_20261023-2147` V0 `pass: true`, `gates/G13_si_20261023-2147-2` V9
`pass: true`, `gates/checks/stage-B8.json` with `check_sleep_writes_nothing` and
`check_sleep_never_brightens` both `pass`, and `cargo test -p si-sleep` at 14 tests /
0 failures). Three things it could not do without a decision that is not the build's.

**Wall 1 — the dimming box has no walkable interior (OD-94, raised here).**
`gates/G13/dimming_sweep.json` is written and **refused**: 3 of 4 axes carry an open
bound, the cross product of the rows as written is 71,772,833,461,370,625 points, and
this artifact has a second blocker the other sweep runners do not — 05 §4.2 defines it
as "a distribution report, not a winner search", so `winner` is never its content.
The box itself is degenerate at both ends: `FLOOR_*` `lo = 1` Q12 sits under every live
edge (a live OWNED edge forms at ~4096 Q12 plus bake) so the clamp never binds, and any
floor above the largest live weight turns P3 into a no-op. Measured at the run's own
point: 8 live edges, 0 below floor, minimum floor 1 (`G13.d.2`). What the build did
instead of choosing: P3 keeps the literal D.2 arithmetic, leaves a below-floor edge
where it is, and counts it in `edges_below_floor`; the shape is asserted, not assumed
(`crates/si-sleep/tests/sleep.rs`, `an_edge_below_the_floor_stays_and_is_counted`, a
`links` edge at 40 under a floor of 64).

**Wall 2 — with the sweep refused, where is a G13 leg graded at all (OD-95, the G13
instance of OD-83, standing since B2).**
05 §18's prerequisite needs the cadence to fire "at least twice", and the registry
placeholder for `SLEEP_EVERY_QUIET_TICKS` is its box midpoint **32769**, unreachable in
any scenario of `G5_LONG_SESSION_INPUTS` shape — so at the shipped value the gate has no
input. `si-harness sweep --gate G13` refuses to name a curve, so it cannot supply one
either. The build installed the six sleep rows at their registry **`lo`**
(`runner::sleep_lo_point()`) because a `lo` is a point the spec already declares rather
than a hand-picked one, printed the reason on the run's own stdout, and wrote it into the
sweep artifact's `od_question`. That is why every cadence and walk count in this stage's
reports is labelled *at `lo`* and why 05 §18(n)'s magnitudes stay UNMEASURED: the run
cites both artifacts (`sweeps cited: 2`) and no row of a curve exists to cite.
Consequence for B12: OD-83 must be ruled before either G13 sweep can produce a row.

**Wall 3 — the V9 control has no input for four of §18's legs (OD-96, raised here).**
05 §2 defines V9 as the variant where `due()` never returns `Yes`, and 05 §18 writes
`a.1`, `c.1`, `c.2` and `f.2` as quantifiers over the run's own nights — at V9 each is a
sum over the empty set, while §18's closing paragraph demands that a run in which no
sleep fired be "reported as a finding … not as a green gate". Both readings cannot be
honoured by a verdict of `pass` or `fail`, so the build reports `unmeasured` with the
launched variant named in `observed`, and leaves the pair's claim where §18(a) puts it:
`G13.a.2` compares V0 against V9 in one process and measures `hashes_differ: true`,
`v0_nights: 5`, `v9_nights: 0`, `envelope_fields_moving: []`, `weight_moved_upward:
false`. At any variant other than V9 the same vacuity is still a **fail**. The cleanest
fix is a spec edit — 05 §18 naming, per leg, which variant grades it — and the authority
order puts that edit in the owner's hand.

**What B8 did not meet, and why it is not a wall.** `G13.e.1`/`e.2` (CONFIG ENGINE, an
`SIgen` checksum) are B10's; `G13.i.2`'s second half needs OD-58 ruled on; `G13.k.3`
needs the median wired and G11's sweep; `G14.f.1` needs B12's seed panel. Each is
`unmeasured` with its own reason and none is a pass.

## Questions still with the owner, as of B8

4. **OD-94 — the dimming box**: narrow it (05 §4.1 step 4), accept `hi` as a design
   ceiling, or treat the no-op floor band as something the distribution shows.
5. **OD-95 — the G13 operating point**: keep the registry `lo` with its label, mark the
   night legs unmeasured until the sweep lands, sweep first and grade after, or let a
   scenario declare constants (which amends OD-86).
6. **OD-96 — a control variant's vacuous legs**: keep `unmeasured`-naming-the-variant,
   fail the control, pass it silently (05 §3 forbids), or amend 05 §18 to scope each leg
   to the variant that grades it — and say whether the same treatment covers G12's V4/V5
   and G9's V1.

## B20 — the tongue: two red legs re-graded, one ceiling named

B20 shipped G20's 21 legs, 52 static checks, the fixture and the four artifacts. Three
things were **findings**, not build choices, and each is recorded with its measurement.

**Wall 1 — G20.a.2: the two processes disagreed because one of them froze
(`fixed, measured`).** The leg compared the launched run's `state_hash` with a grading
drive's and read `28ff7fc1…` against `13fea404…`. The cause was not the voice: the run
calls `session.freeze()` before hashing (`runner.rs:998–999`), and `Run::replay()` and
`Run::replay_transcript` do too, but `drive()` with `freeze_at_end: false` did not —
so the two sides were hashing different states. The fix is the law's own shape: the
leg asserts the run's frozen hash = `Run::replay()` = `Run::replay_transcript()` at
tolerance 0 **and** a frozen grading drive's per-step voice rows equal the launched
run's field for field. All four hashes agree; the five voice rows are byte-identical.

**Wall 2 — G20.f.1: the anti-farming clause graded the wrong channel (`re-scoped,
measured`).** The leg re-entered every realized line as `sense` and expected zero rows;
it measured `shelf_delta {4:1, 7:1, 8:5}` and `protected_delta 1`. The finding: the
**owner channel is a teaching channel by law** (04 §33) — "apple, red" as an owner line
mints an S-R pair and an ask the first time, exactly as it would with the voice off. That
is the channel working, not the voice feeding itself. The zero-rows claim now grades the
**stranger class**, where the mouth refuses the line before `tick` (measured: not even a
`context` row), and the owner re-entry grades **invariance** against `TONGUE = 0`
(measured identical at both settings). The call site is still graded structurally by
`check_tongue_writes_nothing`.

**Ceiling — OD-189: the license, not the budget, is the bound (`RULED 2026-10-25`).**
G20's eight-seed panel measured: three individuals (spread ≤96) speak nothing and print
the canonical frame; five speak, and all five print **byte-identical** lines —
`apple, red` / `red, warm` / `warm, cozy` / `sky, blue` — while `chunk_cap` ranged
**2…12**. One cited row licenses one composed chunk plus its two terms, so two chunks is
the widest honest line; widening `PHRASE_MAX_STEPS` changes no word. The randomness the
walk offers is real but **unobservable at this fixture**, which is the half of docs/27
§27.11's intent the build could not decide for the owner.

**The owner's ruling (2026-10-25), quoted:** *"two-chunk claim license stands (least
authority, no new claim width, one fact per line). Variant-bank surfaces over the same
cited endpoints are licensed grammar: same claim, different surface, no new rows
asserted. G20.b.2's pass condition becomes: at a fixture WITH taught variants, no two
seeds print byte-identical lines (style must be observable); at a bare fixture,
byte-identical is the expected outcome, named as such. chunk_cap publishes with a
license-bound note while the 2-chunk license is the binder."*

**As implemented, measured.** The bank rides `License.variants` (built by
`Session::license_for` with the one fold law; `si-tongue` still holds no table) and
`utter` spends a draw only when a genuine alternative exists. The variants fixture
teaches case spellings in the **predicate** position (`Red`/`RED`, `Warm`/`WARM`,
`Cozy`/`COZY`, `Blue`/`BLUE`, `Cold`/`COLD`) — each a zero-row `Inert{boring}`
confirm that interns the surface — and `g20_tongue_bare.txt` is the control without
them. G20.b.2 on the final build: variants panel, **5 speaking seeds, 5 distinct
transcripts, 27 taught spellings drawn**; bare panel, **5 speaking seeds, 1 transcript,
byte-identical** (named as the expected outcome); both panels' frames/state/census
equal to `TONGUE = 0` per seed, and the protected-shelf census equal across fixtures.
The fixture's order is load-bearing and documented in its header: repeats of the `red`
slot precede `red is bright.`, because a statement after that conflict is an insistence
that supersedes the live row, flips the stage and would land later teaches as new
belief rows. `G20.e.2` was re-graded from byte identity to **claim** identity for the
same reason: an idle pad can ignite and spend a thought id — a sanctioned render-seed
operand — so the padded drive may draw a different spelling while the chunks stay put.
`chunk_cap` publishes everywhere beside `cap_note` (`G20_CAP_NOTE`): the number is a
ceiling and the two-chunk license is the binder. Recorded in
`gates/G20/panel.json` (`od_189`, `chunk_cap_note`) and ledgered as **OD-189 — RULED**.

**Spec corrections owed, and made in this commit** (HOW only; the law did not move):
engineering/13 §6.4 loses the `drew` list and the `reason` column (**OD-187**); §6.6's
(b) five-style-corner procedure is replaced by three budget points plus the seed panel
(a `STYLE_*` row is not installable, **OD-177**); (e) by the quiet-padded trajectory
plus a published `open` for the swept form (**OD-83**); (f) by the stranger-class zero
rows plus owner invariance (04 §33); (i) by `lo`/8/32/64/placeholder across the panel
plus `phrase_sweep.json`-as-refusal. §6.6's (c)/(g) forcing arms keep their names in
`crates/si-tongue/tests/realize.rs` rather than the artifact (**OD-187**).

**What was green on this build, measured.** `check --all` → `stage B20 — 52 checks,
PASS` (42 placeholders, 0 disabled). `run --gate G20` → **PASS** at V14 and V0 (30
criteria; 3 unresolved: `G20.d.3` is the battery clause, `G20.e.3` the swept-arousal
form, and `G14.f.1` by construction — seed `0x5151` is not a member of §4.1's panel, so
no second record can exist at this launch and transcript; the panel pass grades that leg,
36/36). `sweep --gate G20` → **refused**, naming only `PHRASE_MAX_STEPS`
and raising OD-83 + **OD-172**. **Battery**: G1–G19 at V0 plus G10 V2/V3, G12 V4/V5,
G13 V9, G17 V12, G19 V13 — **23/23 `pass: true`**
(`gates/B20/battery_od189.log`). **G14
panel**: 18 seeds × 2 rounds, **36/36 PASS**; `G14.f.1` graded with 18 individuals at
one launch, 1 guard digest, 1 law digest (`gates/B20/panel_od189.log`).
`cargo test --workspace --all-features` → **80 suites, 360 passed, 0 failed**
(`gates/B20/od189_tests.log`); `Cargo.lock` unchanged, so `gates/deps_lock.md5`
stays as B20 step 3 re-published it. **Tree:** `tree_sha256.txt` regenerated after
OD-189 by the same shell pipeline B13 used (`find crates … ; find harness …` → sorted →
`xargs sha256sum` → `sha256sum`) → `4a83d02ce39ff9d53c8219a04023df754ff5144c582979e1d6f78d8dc3ad645c`
over **136 files**, recomputed twice identically. The B13 value (`5f9914…`) is superseded
by this tree; it does **not** reproduce from commit `b871873` under the same pipeline
(that yields `9516779a…` over the same 135 paths), so whatever bytes it was taken over are
not the committed ones — the current value is the reproducible one and the old one is
recorded here rather than silently replaced.

## Questions still with the owner, as of B20

7. **OD-189 — the fertility ceiling** *(answered 2026-10-25)*: the two-chunk claim
   license **stands**; the variant bank returns as licensed grammar over the same cited
   endpoints; `G20.b.2` grades a variants fixture (no two seeds byte-identical) and a
   bare control (byte-identical is the expected outcome). Implemented and green — see
   the B20 section above and `docs/decisions/open-decisions.md`.
8. **OD-172 — what the step bound is for**: a walk-length economics bound or a
   phrase-shape bound; until ruled, every leg that needs it runs at the box's `lo` under
   OD-95 and the sweep stays a written refusal.
9. **OD-83 / OD-94 / OD-95 / OD-96** remain as B8 left them; they gate B12's sweeps and
   the control-variant legs, not this build's greens.

## B13 — global acceptance: 18 of 20 families, the replay pair, and two named walls

The acceptance pass ran every executable item of
[engineering/11](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/engineering/11-build-plan.md)'s B13 DoD on the post-B20 build.
**No new code** was written; this section is what the instruments measured, and what
they cannot.

**Green, measured.**

1. `cargo test --workspace --all-features` → **EXIT=0**, **80 suites, 357 passed,
   0 failed** (`gates/B13/tests.log`).
2. `check --all` → **52 checks PASS**, 0 disabled, 42 placeholders
   (`gates/checks/stage-B20.json`; at `--stage B13` the six B20 checks are disabled and
   the report is `gates/checks/stage-B13.json`, 46 enabled).
3. The build's latest report is `pass: true` for **18 of the 20 families**: G1 (28
   criteria), G2 (17), G3 (14), G4 (14), G6 (10), G7 (17), G8 (29), G9 (24), G10 (25),
   G12 (24), G13 (47), G14 (26), G15 (19), G16 (20), G17 (20), G18 (23), G19 (17),
   G20 (30). The two families with no report are **G5** and **G11** (Walls 1–2).
4. **V0 vs V6:** `replay --session gates/G1_si_20261025-0803 --seed 0x51` →
   `section 0: identical`, `section 1: identical`, **PASS (tolerance 0)**; the recorded
   hash is `5d00fb870e785511783aed8d1ebae8e1` /
   `9734f8f37659eee974b0c6e9d013d148`.
5. **Headless:** `check_headless_run` passes with two viewer-present runs
   (`b11-live`, `b11-view`), each `state_hash before == after == the headless
   state_hash.txt` digest-for-digest, each naming its lane. The **killed-mid-run** arm
   is measured by hand: `runs/hl-long == runs/hl-alive == runs/hl-kill ==
   2a8155dd0936ef913e33e0f4c0bf9429` (B11.7). A killed viewer writes no `viewer.log`,
   so no check can grade that arm — the limit is stated in
   [OD-150](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/docs/decisions/open-decisions.md).
6. **Crates:** `cargo metadata` names **17 packages**; `check_deps` passes with **124
   dependency declarations, all allowlisted**, `Cargo.lock` md5
   `8c77ab4157737668c6b57359dfcdffbb` equal to `gates/deps_lock.md5`, and no
   lexicon/corpus/model package.
7. **Tree:** `tree_sha256.txt` = `5f991430f8b78e6b7c96a781aeb03f582406715b18b5ce6a5585a5b38bd83a85`,
   recomputed identical over 135 files.
8. The B20 verify already re-ran the **battery** (G1–G19 at V0, plus G10 V2/V3, G12
   V4/V5, G13 V9, G17 V12, G19 V13 — 23/23 `pass: true`,
   `gates/B20/battery.log`) and the **G14 panel** (18 seeds × 2 rounds, 36/36 PASS;
   `G14.f.1` graded with 18 individuals at one launch, `gates/B20/panel.log`).

**Wall 1 — G5 has no evaluator at the stage the plan gives it (`OD-190`).**
[11 §5](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/engineering/11-build-plan.md) puts G5's `full` at **B13**, B13's Creates
column says "no new code", and B13's exit gates name **G5** — but no `run --gate G5`
evaluator was ever written (`si-harness: gate G5 has no evaluator at this stage`). Of
its eight legs, four have an instrument in this build: `a.1` (the V6 replay), `a.2`
(`check_no_mood_field`), `c.1` (`check_string_constants`), `c.2` (`check_config_guard`);
`b.1` has a unit test (`interner_is_per_session_and_never_merges_surfaces`), and `d.1`
is covered in substance because the replayed G1 transcript contains the gibberish
teach. **`e.1` (identical per-step activation traces) and `f.1` (identical idle
cascades and monologue entries) have no runner at all**: `Run::replay_traces` exists
([runner.rs:1233](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/crates/si-cli/src/harness/runner.rs#L1233))
and nothing calls it. The acceptance pass does not report G5 green.

**Wall 2 — G11 is B12's, and B12 is waiting on OD-83 (`OD-191`, parent `OD-83`).**
[11 §5](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/engineering/11-build-plan.md) grades G11 through **two sweeps** at B12;
B12's Creates list is five runners (G6, G11 ×2, G12, G14), and none exists. Every
runner that does exist publishes a **written refusal** while the sweep enumeration is
unrulled — ten artifacts, all `status: refused` — so a G11 sweep cannot produce the
curve its legs read. The median arithmetic is unwired and `G3.em.1`, `G7.charge.2`,
`G13.k.3` stay `unmeasured` for the same reason. The acceptance pass reports **18 of
20 families**, not 20.

**What the acceptance pass did not do, and why.** It wrote no code:
[11 §B13](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/engineering/11-build-plan.md) says "no new code", and both walls are the
plan's own dependence on a stage that could not run (B12) or on a stage row that
assumes an evaluator nobody wrote (G5). Building either now would decide the owner's
question silently. Both are ledgered (**OD-190**, **OD-191**) with the reading this
build would take if ruled.

## Questions still with the owner, as of B13

10. **OD-190 — G5 at its own stage**: accept it by instrument and name `e.1`/`f.1`
    UNMEASURED (amends 11 §5's G5 row), write the evaluator (new code at a code-free
    stage — `replay_traces` is already there, only a caller is missing), or move G5's
    full to B12 with G11.
11. **OD-191 — the 20-family claim**: rule **OD-83** so B12's five runners can exist
    and G11 can be graded (the only literal path to 20/20), accept the 18/20 record
    and keep G11 visibly open, or amend 05 §16 to let a V0-only G11 evaluator grade
    the composition half and name both sweeps UNMEASURED.

## B12 — the sweep table is complete and refuses; G11's composition is unreachable, not untested

B12 built the four missing runners and ran every artifact §4 names. The report's
honest headline is that **no curve was published**, and the reason is not a missing
runner: it is the enumeration question (**OD-192**) plus, for G11, two engine walls
this pass measured rather than assumed (**OD-193**, **OD-194** / **OD-144**).

**Measured, green.**

1. `si-harness sweep` now owns **14 artifacts** — the four B12 runners (**G11 median**,
   **G11 composition**, **G14 panel**, **G15 recurrence**) plus the ten that already had
   a file. **G12** is dispatched to an observation-only verifier; its trio is verified,
   not rewritten.
2. **G14** publishes its structure: 18 members, 18 distinct `temperament_digest`s,
   `size_ok`, `members_are_distinct`, `rule_matches_file` all true; `sweep_validation`
   is written into `gates/G14/seed_panel.json` with `whole_box: unmeasured` — the
   honest state while no winner exists.
3. **G12**: `absence_diff.json`, `rejection_reasons.json` and `speak_gate_log.jsonl`
   verify (`output_only: true`, 49 log lines), and no fourth file is invented.
4. **The machinery for a published value exists and is closed:** `consts/swept.rs` is
   the table a sweep would fill, `registry_value` is the one reader, and
   `check_registry_size` binds every entry to its artifact's **sha256**, its box and
   its version tag. The table is empty because no sweep published.
5. `check --all` → **52 checks PASS**, 42 placeholders, 0 disabled;
   `--strict-consts` still refuses every swept row
   (`42 swept SPECIES row(s) still carry UNMEASURED_PLACEHOLDER`).
6. A latent vacuity is closed: `gates/G15/recurrence_sweep.json` (B14's DoD 4) had
   never been written because G15 had no runner; it now exists as a named refusal and
   `check_sweep_never_touches_temperament` scans G15's four rows.

**Wall 1 — OD-192: §4 step 1's enumeration is undefined and every box is open.**
Twelve of fourteen artifacts are named refusals. At least one axis of every box
declares an open bound in 03 (`> 0`, `≥ 0`, `(0,1)`), so the registry's `hi` is
`u16::MAX` and the grid has no last point; the closed cross product runs from 65,535
points (G10, G20) to 3.08×10²⁶ (G16). §4 names no per-axis search rule. OD-83 ruled
**validation**, and its text explicitly leaves step 1 to §4 as written. The artifact
carries the axis list, product, identity, whole-box protocol and
`placeholders_replaced: []`. Full table and options in
[OD-192](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/docs/decisions/open-decisions.md).

**Wall 2 — OD-193: the composite cannot appear, for three independent reasons.**
The §16(e)–(i) scene (`harness/scenarios/g11_composition.txt`) was run as a probe
(run `gates/G11_si_20261025-1250`, seed `0x51`, V0):

- `is an apple warm?` → frame **UNDECODED**, pattern 9, `Output::Inert{reason:
  "input_not_a_teach"}`. The nine-frame table has **no yes/no frame**.
- `what is apple?` after teaching `apple is red` and `red is warm` → answer `red`,
  label `FACT`, citation `[1]`, `route {owned: 1, hops: 1}`. `retrieve` keeps only
  rows keyed on the slot point ([retrieve.rs:159](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/crates/si-read/src/retrieve.rs#L159)),
  so the far row can never be cited and the labeler's two-row INFERENCE branch is
  unreachable ([label.rs:149](file:///C:/Users/jakub/Documents/Qoder/2026-10-22/8c015168/crates/si-front/src/label.rs#L149)).
- The walk stops at one round: `TAU_SETTLE`'s placeholder is **32768**, a round's
  activation is O(2048–4096 Q12), and settle fires when the round-to-round difference
  is below `tau` — the probe's thought 2 stops `Settled` with settle
  `[apple, red, frame]`; `warm` is never lit. Composition therefore also depends on
  **G2's** sweep publishing `TAU_SETTLE` (OD-192), a cross-gate dependency §16 does
  not name.

This is the "if (f) shows no composition at all, publish that" branch. (f)–(i) are
UNMEASURED, not failed-by-design: the legs were not re-scened and no yes/no frame was
invented.

**Wall 3 — OD-194 / OD-144: G11 has no evaluator, and its median half has no
producer.** The dispatcher has no G11 arm; the sweeps that would grade its artifacts
refuse (Wall 1); (b)/(c) need the read-side median H, which is **OD-144**; (e)–(i)
are Wall 2. Writing a partial evaluator now would make a red family look partly green
while its blockers wait. The build therefore records the wall instead of grading
around it.

**What B12 did not claim.** No placeholder was replaced, no gate quotes a sweep
magnitude, and no G11 report exists. The plan's B12 row names "G6" among the runners,
but §4 assigns G6 no artifact and G12's trio is observation-only; the faithful runner
set is G11 ×2 + G14 + G15, recorded in OD-192 rather than patched by inventing a G6
curve.

## Questions still with the owner, as of B12

12. **OD-192 — the enumeration**: full cross product, one-axis-at-a-time against a
    named baseline (which baseline, when every value is a placeholder?), or a declared
    sub-grid? And is `> 0` in 03 an open bound or a closed design box?
13. **OD-193 — composition**: add a yes/no frame to 03 §3.2 and let `retrieve` cite
    reached rows (then wait for G2's `TAU_SETTLE`), or declare G11's (e)–(i)
    UNMEASURED for this build?
14. **OD-194 — G11 grading**: write a partial evaluator now (a red family with named
    blockers), or defer it to the stage that builds H and the frame, or amend B12's
    DoD to "runners and artifacts"?
15. **OD-144 (still open from B11) — the median H**: it is now the explicit blocker of
    G11(b)(c), not only of frame kind 7.

## B13 acceptance, re-run (post-B12/B13.1) — 19 families green, four walls named

**What was run.** The stage-3 acceptance pass over the tree that carries G5's
evaluator, B12's five runners, and the B13.1 relabel. Docs only; no source line
moved. Artifacts: `scripts/battery_stage3.tsv` (29 runs), `scripts/sweep_stage3.log`
(13 sweep owners), `scripts/tests_stage3.log`, `scripts/panel_stage3.log`,
`scripts/cargo_tree_stage3.txt`.

- **Battery — 29 runs, all `rc=0`, 0 failing reports.** G1–G10 and G12–G20 at V0
  (G20 at its pinned `0x5151`), plus G10 V2/V3, G12 V4/V5, G13 V9, G14 V8, G17 V12,
  G19 V13, G20 V14. **G5 has zero unresolved legs** (eight of eight, both configs).
  **G14.f.1 is green at the panel launch**: 18 individuals, 1 guard digest, 1 law
  digest, 107 records from this build; the 18-seed × 2-round panel is 72 PASS / 0
  FAIL.
- **Sweeps — 13 artifacts refused under OD-192** (G2, G8, G9, G10, G11 median +
  composition, G13 sleep + dimming, G15, G16, G17, G18, G20); G14's panel is
  `panel_structure_ok` over 18 members / 18 distinct digests / 121 records and its
  whole-box half is still OD-192's; G12's trio is observation-only and verified.
  **No winner, no placeholder replaced.**
- **DoD.** Tests `EXIT=0` (80 suites, 364 passed, 0 failed); `check --all` 52 PASS,
  42 placeholders, 0 disabled; replay at `0x51` → both sections identical, PASS at
  tolerance 0; headless evidence intact (`hl-long` = `hl-alive` = `hl-kill` =
  `2a8155dd…`, `b11-live`/`b11-view` before == after == on-disk); `cargo metadata`
  17 packages; no forbidden edge in `cargo tree` + `check_deps`; `Cargo.lock` md5
  == `gates/deps_lock.md5`; `tree_sha256.txt` =
  `6ac85e3610cc1f8baf163076e506c9777fcf793d9e3f4e77ab880e95664776ae` over 139
  files, recomputed identically.

**The walls, none worked around.**

1. **G11 has no evaluator (OD-194).** The family is not graded; the sweep table
   refuses; the median half waits on OD-144 and the composition half on OD-193.
   The acceptance reports **19 evaluable families green, 20 accounted**.
2. **G13(e) has no engine run and no instrument (OD-196, new).** 05 §18 heads G13
   "one leg in ENGINE", but §2's V7 row gives G13 no engine variant, §5 names only
   V0+V9, and `g13`'s probes are CONFIG SI by construction. The legs are
   `unmeasured` since B8; they now have a ledger seat. G13 still passes 43/47 at
   V0.
3. **The sweep enumeration (OD-192).** Every winner-searching runner refuses in
   writing; the whole box is unmeasured; no swept value replaces a placeholder.
4. **The read-side median (OD-144) and composition (OD-193).** They keep G3.em.1,
   G7.charge.2, G13.k.3 and G11(b)(c)(e)–(i) unmeasured.

**Honest limits.** The killed-mid-run arm is the B11.7 measurement over the same
300-line transcript (`runs/hl-kill`; a killed viewer writes no log, so no check
grades it — OD-150). The binary has been rebuilt since, but the session loop is
unchanged, and the headless arms are state hashes, not binary hashes. The other
unmeasured legs (S2-unreachable, build-exclusion V1/V10/V11, OD-58's promotion
row, G20's 0x5151 launch group) are unchanged from their stages and name their
blockers in each report.

## Questions still with the owner, as of the B13 re-run

16. **OD-195 — the c.2 relabel**: confirm that `g6_si_pair`'s runtime handle probe
    is `G5.c.2` (the reading B13.1 executed), or restore the historical id.
17. **OD-196 — G13(e)**: build the engine instrument (add V7 to G13's row), amend
    (e) to cite G6(a.1), or keep the pair as a named wall?
18. **OD-192, OD-193, OD-194, OD-144** stand as listed above; the acceptance does
    not depend on their answers, but no sweep or G11 grade can land before them.
