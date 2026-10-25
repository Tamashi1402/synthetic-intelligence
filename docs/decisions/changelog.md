# Design Changelog

## 2026-10-25 — **B13 acceptance, re-run: 19 families green, the sweep table still refuses, and G13(e) gets its ledger seat (OD-196)**
- **What this commit is.** The acceptance pass over the post-B12/B13.1 tree: the
  run battery, the sweep battery, the static checks, the crate map, the headless
  evidence, and the tree hash. **Docs only; no source line moves.** B13's earlier
  record (b871873) stands as the 18-family pass; this is its re-run on the tree
  that now carries G5's evaluator, B12's runners, and the c.1→c.2 relabel.
- **Battery.** **29 runs, all `rc=0`, every report `pass: true`**
  (`scripts/battery_stage3.tsv`, `scripts/stage3_summary.txt`): G1–G10 and
  G12–G20 at V0 (G20 at its pinned `0x5151`), plus G10 V2/V3, G12 V4/V5, G13 V9,
  G14 V8, G17 V12, G19 V13, G20 V14. **0 failing reports.** G5's report is the
  first with **zero unresolved legs** (eight of eight, both configs). **G14.f.1
  re-verified at the panel launch** — 18 individuals, 1 guard digest
  `b6dedd09a3c1821de43013735ad40812`, 1 law digest `c3b589bc18eac9a4ccdf56840f3042f4`,
  107 records from this build (`gates/G14_si_20261025-1312`); the 18-seed × 2-round
  panel re-run is **72 PASS / 0 FAIL** (`scripts/panel_stage3.log`).
- **Sweeps.** `sweep --gate` over every owner: **13 artifacts refused under
  OD-192** — G2, G8, G9, G10, G11 (median + composition), G13 (sleep + dimming),
  G15, G16, G17, G18, G20 — so no winner and no placeholder replaced
  (`scripts/sweep_stage3.log`). G14's panel publishes `panel_structure_ok`
  (18 members, 18 distinct digests, 121 records from this build) and says the
  whole-box half is still OD-192's; G12's trio is observation-only and verified
  byte-for-byte (49 log lines).
- **DoD 1/2/4–7.** `cargo test --workspace --all-features` → **EXIT=0, 80 suites,
  364 passed, 0 failed** (`scripts/tests_stage3.log`); `check --all` → **52 PASS**,
  42 placeholders, 0 disabled (`gates/checks/stage-B20.json`); replay at `0x51`
  over `gates/G1_si_20261025-1312` → section 0/1 identical, **PASS tolerance 0**;
  headless evidence intact — `runs/hl-long == runs/hl-alive == runs/hl-kill ==
  2a8155dd0936ef913e33e0f4c0bf9429`, and `b11-live`/`b11-view` before == after ==
  on-disk with their lanes named; `cargo metadata` → **17 packages**; `cargo tree`
  plus `check_deps` → no forbidden edge (`winit`/`ash`/`raw-window-handle` named
  under `si-vulkan` alone; `scripts/cargo_tree_stage3.txt`); `Cargo.lock` md5
  `8c77ab4157737668c6b57359dfcdffbb` == `gates/deps_lock.md5`; `tree_sha256.txt`
  → **6ac85e3610cc1f8baf163076e506c9777fcf793d9e3f4e77ab880e95664776ae** over
  **139 files**, recomputed identically with B13's pipeline on the CRLF working
  tree.
- **The two walls, and the one new ledger seat.** **G11** still has no evaluator
  (OD-194, unchanged). **G13(e)** names "one leg in ENGINE" while §2's V7 row
  gives G13 no engine run, §5 names only V0+V9, and `g13`'s probes are CONFIG SI
  by construction; both legs have been `unmeasured` since B8 with no ledger seat.
  They now have one: **OD-196** (build the engine instrument / amend (e) / keep
  the wall), build choice (c) recorded. G13 still passes **43 of 47** legs at V0;
  its four unmeasured legs are OD-196 ×2, OD-58 (i.2), OD-144 (k.3).
- **What is not claimed.** No placeholder is replaced; no sweep publishes a
  curve; G11 is not graded; G13's engine pair is not measured. The acceptance is
  **19 evaluable families green, 20 accounted** (G11 named and ledgered), not 20
  green.
- **Checks.** No source change; the tree hash is unchanged by the ledger line
  (docs are outside its path set).

## 2026-10-25 — **B13.1: G5 measures all eight legs in both configs; the engine G6 pair exists for the first time**
- **What was built.** `run --gate G5` now has its evaluator (05 §10's a–f), and
  `Run::replay_traces` gains its first caller: it returns **per-step event
  frames**, not the settle set, because `e.1` says comparing only the settle-set
  is not enough. A new `Run::replay_idle` returns each section's `IdleReport`
  sequence and `MonologueLine` sequence for `f.1`. New fixture
  `harness/scenarios/g5_determinism.txt`: a served recall empties the agenda so
  three quiet ticks ignite, a committed English statement and the gibberish pair
  give `d.1` its parity scene, and a disjoint cold section gives `b.1` two real
  conversations.
- **Measured at seed `0x51`.** **G5 PASS at V0**
  (`gates/G5_si_20261025-1306`) and **at V7** (`gates/G5_engine_20261025-1306`),
  eight of eight legs each. `e.1` compared **126 frames over 12 steps** with zero
  differences; `f.1` compared **4 idle ticks, 3 ignitions and 3 monologue lines**
  (the sampler picked point 2 three times in CONFIG SI and point 9 three times
  under the frozen layer — a difference between configs, never compared as one);
  `b.1` found **zero shared terms**, with ids 1 and 2 mapping to different terms
  in the two sessions and every interner token spoken by its own transcript;
  `a.1` reproduced both section hashes at **0 bytes** tolerance in both configs;
  `d.1` saw one row written, pattern 2 on both asks, and commit rows 2 vs 3 with
  equal shape.
- **The G6 gap closed.** No CONFIG ENGINE G6 report existed: every prior G6 run
  was the SI pair, because the default build carries no loader body. Built the
  B10 twin (`cargo build -p si-cli --features engine`), ran **G6 V7**
  (`gates/G6_engine_20261025-1305`, 6 pass / 2 named-unmeasured: `b.4`'s B10
  ruling and `d.1`'s corpus rule) and its **SI pair**
  (`gates/G6_si_20261025-1305`), whose open-register probe measured **0 frozen
  opens with a positive control**.
- **An id clash, fixed and ledgered.** The SI pair's handle probe had carried
  `G5.c.1` since B10, the id 05 §10 gives the string-constant audit; B13.1's
  evaluator now grades that id as the string audit, so the probe was relabeled
  `G5.c.2` (the handle audit 05 §10 names, whose runtime half it is) and the
  static `c.2` leg now points at it. **OD-195** records the finding.
- **What this entry does not claim.** `G14.f.1` stays **unmeasured** (the seed
  panel holds 3 of 18 members at this launch — Stage 3's panel pass owns it);
  G11 still has no evaluator (**OD-194**); no constant, box or threshold moved.
- **Checks.** `cargo test --workspace` green; `check --all` **52 PASS**,
  **42 placeholders**, 0 disabled.

## 2026-10-25 — **B12: the sweep machinery lands, the enumeration is the wall, and the panel is the one artifact that publishes**
- **What was built.** `si-harness::sweep` now owns the full §4 runner set — the four
  B12 was missing (**G11 median**, **G11 composition**, **G14 panel**, **G15
  recurrence**) plus the ten that already had a file — and dispatches **G12** to an
  observation-only verifier instead of inventing a fourth file for a trio §17 defines
  as output-only. Every refusal is written into the artifact with its axis list,
  registry home, declared class, open bounds, cross product, the build's identity
  (one variant, one build, `cross_variant_hash_comparison: false` — OD-93), the
  whole-box protocol (§4.1) and `temperament_rows_untouched` (OD-87: G11's `GAMMA`
  and G20's style rows are named, never installed).
- **The mechanism for a published value.** `si-core::consts` split into
  `consts/mod.rs` + `consts/swept.rs`; `registry_value` is now the single reader for
  a swept row (declared value → published sweep → placeholder → law), and
  `check_registry_size` validates every `SWEPT` entry against its artifact's
  **sha256**, its box, its grid and its version tag — so a hand-edited value with no
  artifact fails the check. The table is **empty** because no sweep published: no
  placeholder was replaced, and `--strict-consts` still refuses every swept row.
- **Measured, not asserted.** Fourteen artifacts at `0x51`: **twelve named
  refusals**, **G14** structure-only (18 members, 18 distinct temperament digests,
  `whole_box: unmeasured`), **G12** verified observation-only (`absence_diff.json`,
  `rejection_reasons.json`, `speak_gate_log.jsonl`, 49 log lines). Every refusal
  cites **OD-192** — the enumeration question — with the product it cannot walk
  (65,535 for G10/G20 up to 3.08×10²⁶ for G16). OD-83's ruling covers validation
  only; the search object is the open question, and it is ledgered, not guessed.
- **A latent vacuity closed.** `gates/G15/recurrence_sweep.json` (B14's DoD 4) had
  never been written because G15 had no runner, which also left
  `check_sweep_never_touches_temperament` scanning an empty axis list for the gate.
  Both are fixed by the runner, and the artifact now exists as a named refusal.
- **The composition probe, recorded as a wall.** The §16(e)–(i) scene was run at V0
  rather than assumed: `is an apple warm?` decodes to **UNDECODED** (03 §3.2 has no
  yes/no frame), `what is apple?` answers `red` because `retrieve` cites only
  slot-point rows, and `TAU_SETTLE`'s placeholder settles the walk at `k = 1`. Those
  are **OD-193**; the missing G11 evaluator and the unwired median (**OD-144**) are
  **OD-194**. No yes/no frame, citation widening or partial evaluator was invented.
- **What this entry does not claim.** No constant has a value, no gate quotes a
  sweep magnitude, and G11 still has no evaluator (its family stays open per
  OD-191, now with OD-193/194 naming the engine blockers). The plan's B12 row names a
  G6 runner, but §4 assigns G6 no artifact — recorded in OD-192 rather than invented.
- **Checks.** `si-harness check --all` → **52 checks PASS**, **42 placeholders**, 0
  disabled; `cargo test -p si-cli --lib` green (the odometer, the corners and the
  runner table are unit-tested).

## 2026-10-25 — **The B12 batch's rulings: the box is the law, artifacts are the only values, and B13's two walls open**
- **The rulings.** Eight owner rulings, all recorded in
  [open-decisions.md](open-decisions.md) with the batch date:
  **OD-83** — whole-box validation **stands**: every sweep re-runs the winner *and
  both box corners* (range floor, range ceiling) at every `seed_panel` member, with
  the gate's behavioral legs graded at each point (§4.1 steps 1–4); point-only is
  rejected because it makes the panel a lottery, and a failing panel point narrows
  the **box** in [03](../../engineering/03-constants-registry.md), never the panel.
  **OD-97** — the crystallize constants take their values from the sweep artifacts
  only; no hand-argued constants. **OD-98** — candidate tables are **hash-in**
  session state, no hash-exempt exceptions (OD-103 silt and OD-117 skills take the
  same ruling). **OD-99** — the ghost budget is a species guard; no temperament
  draws among v2 candidate/ghost constants; `check_temperament_closed` stays **18**.
  **OD-190** — B13.1 is authorized to write G5's `e.1`/`f.1` evaluator and wire
  `Run::replay_traces`; no other source moves. **OD-191** — the 18/20 acceptance
  record is **sequencing**, not failure: G11 grades when B12's runners exist, and
  B12 is the only remaining engine work.
  **OD-172** — `PHRASE_MAX_STEPS` is option **(a)**: a swept SPECIES box with the
  floor leg at `lo` until the phrase sweep publishes. **OD-189** — already ruled
  and implemented (`aa57ff4`), with the added clause that a variant-rich fixture
  still showing zero style divergence reopens the license question as a **new OD**,
  never a silent widening.
- **The numbering clash, recorded rather than propagated.** The work order heads
  its three v2 items "OD-94 / OD-95 / OD-96", which are **§22's draft numbers**:
  [docs/22 §22.11](../22-crystallize.md) re-seated the draft's OD-94…98 as ledger
  **OD-97…101** precisely because B8's sleep walls took OD-94/95/96 the same day.
  The rulings therefore land on **OD-97/98/99**, and B8's three walls — the dimming
  box (OD-94), the G13 operating point (OD-95) and the control-variant vacuity
  (OD-96) — **stay OPEN**, each now carrying a pointer to the ruling's true seat.
- **What is not claimed.** No sweep has run and no placeholder is replaced by this
  entry; the whole-box ruling is validated when B12's runners execute. The B12
  stage report carries the measured outcome.
- **Measured, not asserted.** Docs-only at this point: `si-harness check --all` →
  **52 checks PASS**, 42 placeholders, 0 disabled.

## 2026-10-25 — **OD-189: the variant bank returns as licensed grammar — style observable, the claim fixed, the ceiling named**
- **The ruling.** The owner's words: *"two-chunk claim license stands (least
  authority, no new claim width, one fact per line). Variant-bank surfaces over the
  same cited endpoints are licensed grammar: same claim, different surface, no new
  rows asserted. G20.b.2's pass condition becomes: at a fixture WITH taught variants,
  no two seeds print byte-identical lines (style must be observable); at a bare
  fixture, byte-identical is the expected outcome, named as such. chunk_cap publishes
  with a license-bound note while the 2-chunk license is the binder."* Recorded in
  [docs/27 §27.10](../27-tongue.md) and
  [open-decisions.md](open-decisions.md) as **OD-189 — RULED**.
- **What was built.** A taught surface whose `si_front::tokenize` keys fold to an
  existing term is a **spelling of that same point**, so the license may carry it:
  `si_tongue::Variant { surface, keys }` and `License { point, surface, keys, variants }`
  ([crates/si-tongue/src/lib.rs](../../crates/si-tongue/src/lib.rs)). `utter` is the one
  chooser: it spends one `next_q12` per entry **only when a genuine alternative
  exists**, and returns `Some(entry.surface.as_str())` when it does not.
  `Session::license_for` walks the interner once, groups unique verbatim surfaces by
  the one fold law and attaches matching spellings
  ([crates/si-session/src/lib.rs](../../crates/si-session/src/lib.rs)); `si-tongue`
  still holds no table (**OD-184**). No new row, no wider citation, no extra chunk,
  no state: the bank is grammar, not content.
- **The wall the bank met, and how it was taught.** `check_tongue_license_only` had
  pinned the old shape (`parts.push(anchor_license.surface…)`). It now requires every
  `parts.push(` argument to be `utter(…)` (rule 3) and audits `utter`'s body (rule 4):
  the only `.surface` reads are `entry` and `variant`, the canonical arm is exactly
  `Some(entry.surface.as_str())`, the drawn arm exactly `then_some(chosen)`, and both
  its lookups come from the caller's `license` slice
  ([crates/si-cli/src/harness/checks.rs](../../crates/si-cli/src/harness/checks.rs)).
  The guarantee ("phrase bytes only from a licensed surface") is unchanged; only its
  shape moved. `check --all` → **52 PASS**, 42 placeholders, 0 disabled.
- **The fixtures, and the order that matters.** `harness/scenarios/g20_tongue.txt`
  gains ten spelled repeats (`Red`/`RED`, `Warm`/`WARM`, `Cozy`/`COZY`, `Blue`/`BLUE`,
  `Cold`/`COLD`) taught in the **predicate** position — zero-row `Inert{boring}`
  confirms that still intern the surface. All spelled repeats of the `red` slot precede
  `red is bright.`, because a statement after that conflict is an *insistence* that
  arbitrates, commits a row, supersedes `red → warm` and turns later teaches into new
  belief rows; the header documents the law. New control
  `harness/scenarios/g20_tongue_bare.txt` is the same transcript without the repeats.
- **G20.b.2, re-graded to the ruling.** Variants panel: **5 speaking** of 8 seeds,
  **5 distinct transcripts**, **27 taught spellings drawn**. Bare panel: **5 speaking**,
  **1 transcript — byte-identical**, named as the expected outcome. Both panels'
  frames, state and protected-shelf census equal `TONGUE = 0` per seed, and the census
  is equal **across** fixtures — a spelling asserted no row
  ([crates/si-cli/src/harness/gates.rs](../../crates/si-cli/src/harness/gates.rs)).
- **G20.e.2, re-graded from bytes to the claim.** The padded drive can ignite and spend
  a thought id, a sanctioned render-seed operand, so a spelling may move while the
  asserted chunks do not. The leg now compares the chunk-wise folded key bag:
  **6 ticks compared, 4 respelled-only, 6 padded voice ticks, citation drift `[]`**.
  A chunk that moves still fails; letters that move are the bank working.
- **`chunk_cap` publishes with its note.** `G20_CAP_NOTE` travels beside the number in
  the report and in `gates/G20/panel.json` (`od_189`, `chunk_cap_note`): chunk_cap is a
  ceiling, and the two-chunk license binds first.
- **Law and HOW.** [docs/27 §27.2, §27.8(b)(e), §27.10, §27.11](../27-tongue.md);
  [engineering/13 §1.2, §4.2, §6.4, §6.6, §9, §10](../../engineering/13-tongue.md) and
  the B20 DoD row (adds the bare fixture and the `si-session` test). OD-190/OD-191 are
  untouched.
- **Measured, not asserted.** `run --gate G20` → **PASS** at **V14** and **V0** on
  `g20_tongue.txt` (30 criteria; 3 unresolved: `G20.d.3` the battery clause, `G20.e.3`
  the swept-arousal form, `G14.f.1` by construction — seed `0x5151` is not a panel
  member; the panel pass grades it). **Battery** 23/23 `pass: true`
  (`gates/B20/battery_od189.log`). **Panel** 18 seeds × 2 rounds → 36/36 PASS
  (`gates/B20/panel_od189.log`). **Sweeps** 10 of 10 `status: refused`, no winner.
  **Replay** V0→V6: `section 0/1 identical`, PASS at tolerance 0.
  `cargo test --workspace --all-features` → **80 suites, 360 passed, 0 failed**
  (`gates/B20/od189_tests.log`). `Cargo.lock` md5 unchanged
  (`8c77ab4157737668c6b57359dfcdffbb`). `tree_sha256.txt` →
  `4a83d02ce39ff9d53c8219a04023df754ff5144c582979e1d6f78d8dc3ad645c` over 136 files,
  recomputed identically; the B13 value does not reproduce from `b871873` under the same
  pipeline (`9516779a…` over the same 135 paths), which is recorded in
  [failure_report.md](../../failure_report.md) rather than overwritten.

## 2026-10-25 — **B13: global acceptance — 18 of 20 families, the replay pair, and two named walls**
- **What this commit is.** Not code. The acceptance pass of
  [engineering/11 §4](../../engineering/11-build-plan.md) over the post-B20 build:
  every executable B13 item run, and the two families that cannot be run named in
  `failure_report.md` instead of silently counted green. The working tree holds no
  source change; the commit is the record the plan's DoD 8 asks for, plus
  **OD-190** and **OD-191** in the ledger.
- **Green, measured.** `cargo test --workspace --all-features` → **EXIT=0, 80 suites,
  357 passed, 0 failed** (`gates/B13/tests.log`). `check --all` → **52 PASS**, 0
  disabled, 42 placeholders (`gates/checks/stage-B20.json`; `--stage B13` is 46
  enabled + 6 disabled by stage). **18 of 20 families** carry a latest report with
  `pass: true` — G1, G2, G3, G4, G6, G7, G8, G9, G10, G12, G13, G14, G15, G16, G17,
  G18, G19, G20 — with 28/17/14/14/10/17/29/24/25/24/47/26/19/20/20/23/17/30 criteria
  respectively.
- **The replay pair.** `replay --session gates/G1_si_20261025-0803 --seed 0x51` →
  `section 0: identical`, `section 1: identical`, **PASS (tolerance 0)**; the hash is
  `5d00fb870e785511783aed8d1ebae8e1` / `9734f8f37659eee974b0c6e9d013d148`. `V6` is
  the `replay` mode of [05 §2](../../engineering/05-gate-harness.md)'s variant table,
  so this is the V0→V6 leg the DoD names. The **headless** half passes in
  `check_headless_run` with two viewer-present runs (`b11-live`, `b11-view`) whose
  before/after/on-disk hashes agree and whose lanes are named; the **killed-mid-run**
  arm is the B11.7 measurement — `runs/hl-long == runs/hl-alive == runs/hl-kill ==
  2a8155dd0936ef913e33e0f4c0bf9429` — and stays hand-measured because a killed viewer
  writes no log ([OD-150](open-decisions.md)).
- **Crates and tree.** `cargo metadata` names **17 packages**; `check_deps` passes
  with **124 dependency declarations, all allowlisted**, `Cargo.lock` md5
  `8c77ab4157737668c6b57359dfcdffbb` equal to `gates/deps_lock.md5`.
  `tree_sha256.txt` is `5f991430f8b78e6b7c96a781aeb03f582406715b18b5ce6a5585a5b38bd83a85`
  over 135 files and recomputes identically.
- **The B20 evidence still holds.** Battery: G1–G19 at V0 plus G10 V2/V3, G12 V4/V5,
  G13 V9, G17 V12, G19 V13 — **23/23 `pass: true`** (`gates/B20/battery.log`).
  Panel: 18 seeds × 2 rounds — **36/36 PASS**, `G14.f.1` graded
  (`gates/B20/panel.log`). Sweeps: **ten artifacts, all `status: refused`** — no
  swept winner replaces any of the 42 placeholders.
- **Wall 1 — G5 has no evaluator at the stage that owns it (`OD-190`).** 11 §5 puts
  G5's full at B13, B13 says "no new code", and the exit gate names G5 — but
  `run --gate G5` refuses. Four legs have instruments (`a.1` replay, `a.2`
  `check_no_mood_field`, `c.1` `check_string_constants`, `c.2` `check_config_guard`),
  `b.1` has a unit test, `d.1` is covered by the replayed G1 transcript — **`e.1`
  (traces) and `f.1` (idle cascades + monologue) have no caller at all**:
  `Run::replay_traces` exists and nothing invokes it.
- **Wall 2 — G11 waits on B12, B12 waits on OD-83 (`OD-191`).** G11's full is B12's;
  B12's five runners (G6, G11 ×2, G12, G14) do not exist; every runner that does
  exist refuses while the sweep enumeration is unrulled; the median arithmetic is
  unwired, so `G3.em.1`, `G7.charge.2` and `G13.k.3` stay unmeasured. The pass
  reports **18/20**, not 20.
- **What the plan said, and what the record says.** The plan's commit subject is
  kept; the outcome is the honest one: all executable items green, two families
  ungraded, both ledgered with the reading this build would take if ruled.

## 2026-10-25 — **B20 step 4: the gate learns to hear — G20 a–k, 52 checks, the refusing phrase sweep**
- **What this commit is.** The **G20** evaluator
  ([crates/si-cli/src/harness/gates.rs](../../crates/si-cli/src/harness/gates.rs)) with
  21 G20 legs; the fixture `harness/scenarios/g20_tongue.txt` (seed `0x5151`, six
  teaches, four asks, a `voice` line and a grimm line); the four artifacts under
  `gates/G20/`; `RUNNERS` gains `("G20", "gates/G20/phrase_sweep.json", "phrase")`
  with its scenario path; and the six static checks land (46 → **52 ids**). The
  tongue round's Phase 3 ends here; spec and HOW are read-only from the next stage on.
- **Two legs were red and were re-graded by instrumenting the law, not relaxing it.**
  **G20.a.2** compared the launched run's state hash with a grading drive's and they
  disagreed because the run hashes **after `session.freeze()`** (runner.rs:998) and a
  `drive` with `freeze_at_end: false` does not. The leg now asserts the run's frozen
  hash = `Run::replay()` = `Run::replay_transcript()` at tolerance 0, **and** a frozen
  grading drive's per-step voice rows equal the launched run's field for field; all
  four hashes agree and the 5 voice rows are byte-identical.
  **G20.f.1** re-entered the realized lines as `sense` and expected zero rows; it
  measured `{4:1, 7:1, 8:5}` and one protected row. The finding: the **owner channel
  is a teaching channel by law** (04 §33), so those rows are the channel working, not
  the voice farming. The zero-rows claim now grades the **stranger** class (the mouth
  refuses before `tick`; measured: not even a context row), and the owner re-entry
  grades **invariance** against `TONGUE = 0` — measured identical.
- **Ledger.** **OD-187** (no `drew`, no `reason`: the walker's diagnostics stay in the
  crate), **OD-188** (the anchor is the answer row's `key_point` — `val_point` is a
  leaf with no out-arc), **OD-189** (**OPEN for the owner**: the license, not the step
  budget, is the fertility bound — all speaking individuals print byte-identical
  ≤2-chunk lines across a panel whose `chunk_cap` ranged 2…12).
  Ledger seats are in [docs/27 §27.10](../27-tongue.md) and
  [open-decisions.md](open-decisions.md); the HOW's §6.4/§6.6 procedures are corrected
  to the as-built gate.
- **Measured, not asserted.** `check --all` → `stage B20 — 52 checks, PASS`
  (42 placeholders, 0 disabled). `run --gate G20 --variant V14|V0 --seed 0x5151
  --scenario harness/scenarios/g20_tongue.txt` → **PASS** both (30 criteria; 2 open:
  `G20.d.3` the-battery clause and `G20.e.3` the swept-arousal form). `sweep --gate
  G20 --seed 0x5151` → **refused**, `gates/G20/phrase_sweep.json` naming only
  `PHRASE_MAX_STEPS` and raising **OD-83** + **OD-172**. **Battery**: G1–G19 at V0 plus
  G10 V2/V3, G12 V4/V5, G13 V9, G17 V12, G19 V13 — 23/23 `pass: true` on this 17-crate
  `0x0003` build (log: `gates/B20/battery.log`, ignored). **G14 panel**: 18 seeds × 2
  rounds, 36/36 PASS, and **`G14.f.1` is graded for the first time** — 18 individuals
  at one launch, 1 guard digest, 1 law digest (`gates/G14/seed_panel.json`).
  `cargo build --workspace` and `cargo test --workspace` green; `Cargo.lock`'s md5 is
  unchanged (`8c77ab4157737668c6b57359dfcdffbb`), so `gates/deps_lock.md5` stays as
  B20 step 3 re-published it (**OD-185**).

## 2026-10-25 — **B20 step 3: the loop has a mouth — the one call site, its license, its fallback**
- **What this commit is.** 13 §4.1's step **9a**, realized as **one** insertion at
  [crates/si-session/src/lib.rs:2002](../../crates/si-session/src/lib.rs#L2002) —
  the `if !rendered.surface.is_empty() && label != "I-DON'T-KNOW"` arm, after
  `label`/`tag`/`citation` are bound and before the `EV_OUTPUT` log, exactly where the
  HOW says. `ArcConfig` gains `tongue` and `phrase_max_steps` read off the registry by
  the same `arc_bool`/`arc_value` channel as the arc's 22 rows; `launch_constants`
  gains its two arms and the `Species | Ruling` guard text **does not move** — which
  is the whole of OD-177's protection for the five `STYLE_*` rows. `si-session` gains
  the `tongue` cargo feature (on by default, 13 §1.4) and three private methods:
  `realized_surface` (the `cfg` pair: off, or `TONGUE` uninstalled, or the walker
  refuses ⇒ `rendered.surface.clone()`, the expression this file shipped before B20),
  `tongue_line` (anchor, license, seed, walk, and checks (b)/(c) at the boundary over
  the same `si_front::tokenize` fold), and `license_for` (the mouth as **data**, per
  **OD-181**: the answer row's composed chunk pushed first, then every non-nameless
  citation's composed surface and both its endpoints' own terms).
- **The anchor is chosen by the printer's own rule, not a new one.** `tongue_line`
  selects the answer row with the *same* pool and the *same* comparator
  `reverse_decode` uses — non-nameless, then max on `settle_set` activation, then
  lower `row_id` — so the row that speaks and the row that was decoded can never
  disagree. The seed is **OD-183**'s triple `(birth_seed, style.digest(), thought_id)`;
  the state hash is not an operand anywhere in this file.
- **What was measured, not asserted (G20(j)'s shape).** `g2_paraphrase.txt` was run
  twice at seed `0x51`, once clean and once with `TONGUE=1 PHRASE_MAX_STEPS=4`.
  `state_hash.txt` is the same string both times — `bfba85395611c8050825a92b25a1df02` —
  `transcript.txt` is byte-identical, and every `.siro` differs only in its own
  header's `run_id` field. So the installed row changes no byte of this fixture. That
  is the *string-equality* proof 13 §6.6 asks for, not a hash-and-hope.
- **And the honest half of that sentence.** The two runs agree because on this
  fixture the walker returns the canonical copy or refuses: the answer row's
  `val_point` has no qualifying licensed live arc, so the realized path is never
  entered. **The fallback is proven; the realization is not exercised.** This is
  recorded as step 4's binding obligation rather than papered over:
  `harness/scenarios/g20_tongue.txt` must teach a chain whose anchors *do* carry
  licensed, live, non-genome outgoing edges, or every V14 leg would silently equal
  V0 and the gate would be grading its own absence.
- **OD-185's consequence repeated, and caught by this stage's own gate.** Adding
  `si-tongue` as an optional dependency of `si-session` changes `Cargo.lock`, so
  `check_deps` failed (`8c77ab…` vs the pinned `d03127…`) on the first run of
  `check --all`. The lock md5 is re-published to `8c77ab4157737668c6b57359dfcdffbb`;
  no external package enters the graph, so **OD-42**'s allowlist is still untouched.
  `check --all` then reports **46 ids, zero failures**, placeholders at 42.
- **Pointers, not law.** Three source-line references the insertion moved are
  corrected to the post-build lines (13 §4.1, 13 §5's break rows 6–7, 11 §B20's file
  list); row 6's `runner.rs` pointer was already 23 lines stale at HEAD and is
  repinned to 1903 while it is being read.
- **What is *not* in this commit, on purpose.** The G20 legs, the scenario, the
  refusing `phrase` runner, the six new `check_*` ids and `Stage::B20` (step 4), and
  therefore any *measurement* of the tongue in the harness. `TONGUE` still ships `0`.

## 2026-10-25 — **B20 step 2: the leaf crate walks — `si-tongue`, OD-185, OD-186**
- **What this commit is.** The seventeenth crate. [engineering/13](../../engineering/13-tongue.md)
  §5's file list realized: `crates/si-tongue/{Cargo.toml,src/lib.rs,tests/realize.rs}`
  with exactly the two imports **OD-184** allows, the `License` argument as the only
  mouth (**OD-181** as a hole, not a rule), `realize`'s phrase walk in 04 §34 AF.3's
  step order, and `realizes_the_slots` as the boundary half of checks (b) and (c).
  `si-walk` gains `Graph::from_arcs` — a `Graph` the tests can build without naming a
  `Lattice`, which is what `check_tongue_is_leaf` forces; and the workspace registers
  the member (`Cargo.toml`, `check_deps`' 16 → **17** names).
- **Two clashes the *build* found in the HOW** (the ledger's span becomes
  **OD-163 … OD-186**), both settled toward the older law, neither a silent workaround:
  **OD-185** — 13 §3.4, 13 §6.3 and 11 §B20 DoD 7 all said `gates/deps_lock.md5`
  "does not move", but that file is `md5(Cargo.lock)` and a workspace member *is* a
  `[[package]]` row in it; `git log` shows B4, B14, B15, B17 and B11 each re-publishing
  it when their crate arrived. So it is re-published here — and no external package
  enters the graph, which is what **OD-42**'s allowlist actually governs.
  **OD-186** — §34.3 spelled `owned_share`'s formality predicate as
  `edge_kind_is_owned(kind)`, whose last slot **is** `6 cooccur-weak`: that function
  counts record columns, not claims. Taken literally, `STYLE_DEPTH` had no associative
  side to lean away from, and this crate's own formality test measured the flat split
  it produces. The law's word is "claim-side", so `si_core::addr::edge_kind_is_claim`
  (kinds 0..=5 — the rule `si-front::label::is_claim` already writes by hand) is the
  predicate, and 03 §7, 04 §34 AF.3 and 13 §2/§34.3's tables are corrected to it.
- **B20 DoD 2 is met here.** `crates/si-tongue/tests/realize.rs` runs 13 §6.6's
  forced-failure table as unit tests (step-6's per-crate precedent): no candidates ⇒
  the canonical copy, unanchored ⇒ `None`, empty license ⇒ `None`, a licensed point
  with no surface ⇒ `None` (G20(c)'s failure, as a real refusal the harness can log),
  a lost slot ⇒ `None` before the walk starts, spread drawn `0` ⇒ zero extra chunks,
  budget spent ⇒ `1 + budget` chunks on a 12-point chain, replay identity on
  `render_seed(7, digest, 3)` (**G20(a)**/**I5**), a genome arc never walked
  (**OD-179**), an unlicensed neighbour not utterable (**OD-181**), both gate halves,
  the ceiling formality lean landing on the claim side, and a zero axis adding exactly
  zero. 14 tests, all green; `cargo test --workspace` green; `check --all` **46 ids,
  zero failures** on the 17-crate / `0x0003` build with placeholders at 42.
- **What is *not* in this commit, on purpose.** The `si-session` call site and the
  `tongue`/`phrase_max_steps` fields (step 3), the G20 legs, the six new `check_*`
  ids and `Stage::B20` (step 4). `TONGUE` is still installed `0`, so nothing in the
  loop can reach the walker yet and every pre-tongue gate stays bit-identical — the
  crate is dead code until step 3 wires it, which is exactly how V14 is meant to be
  additive.

## 2026-10-25 — **Step 8b: the tongue is engineered — engineering/13, OD-182 … OD-184**
- **What this commit is.** The HOW for [27-tongue.md](../27-tongue.md), written
  before any code and folded into the numbered files it changes, so the build has
  one source of truth: [engineering/13-tongue.md](../../engineering/13-tongue.md)
  plus in-place deltas in **00** (the seventeenth crate, its edges, win 8, the
  `tongue` feature), **02** (tick step **9a**, ordering law **30**, grep rules
  **(xviii)–(xxi)**, the new **§18** contract), **03** (§1.2's sixth class, the
  "adds 8 rows" arithmetic, new **§5.4**), **04** (new **§34 AF.1–AF.4**, ordering
  law 30), **05** (**V14** in §2, **§20.1**'s six checks 46 → 52, **§24**'s eleven
  legs + four artifacts + refusing `phrase` runner, §21's sequencing), **06** (new
  **§15** cost), **11** (B20's row, §4 stage detail, the matrix row, §7's DoD) and
  this file.
- **Three clashes engineering found in the law, each minted as a seat and each
  settled toward the older law** (the ledger's span becomes **OD-163 … OD-184**):
  **OD-182** — §27.3 defines the recency draw on "recently-baked / **recently-charged**"
  arcs, but **OD-168** rules the tongue reads no `a`, and `Arc.a_at` is exactly that:
  recency is realized as `bake` + arena position, `a_at` never read, and
  `check_tongue_reads_no_a` turns the promise into a grep.
  **OD-183** — §27.4 seeds the per-render PRNG from the **state hash** too, but hash
  item 3 is the stamp arena and stamps carry `a_at`, so the draft's own operand
  re-admits arousal through the back of the seed: the seed is
  `(birth_seed, style digest, thought_id)` and nothing else.
  **OD-184** — §27.7 gives the crate `si-store`; a `&Lattice` enumerates every point,
  which is the uncited word **OD-181** forbids. Two imports instead, surfaces handed in
  as `License` data — **OD-181 becomes a hole in an import list rather than a rule it
  obeys** (00 §2 win 8), and that is what makes G20(h) gradeable.
- **The break-avoidance finding worth more than the mechanism.** The sixth registry
  class is **inert by construction**: five `STYLE_*` rows carry `sweep: None`, so
  `placeholders_match_registry()` stays true; `axes_of` filters `Class::Species`, so
  no sweep can see them; and `check_sweep_never_touches_temperament` **string-matches**
  the exact `Species | Ruling` refusal literal in `runner::launch_constants` — which
  therefore needs **no edit**, and the guard that already protects the 18 draws
  protects the 5 style values without knowing they exist.
- **Two refusals kept as refusals.** `PHRASE_MAX_STEPS` gets an open box, so its
  sweep publishes `status: refused` like the arc's four instead of enumerating 65 535
  scenario runs — and G20(i) runs at `1` and names it a floor (**OD-95**'s precedent),
  because a leg cannot honestly quote a magnitude no artifact has (**OD-83**).
  `G20(j)`'s proof is **string equality**, not hash equality: the tongue writes no
  hashed item, so a hash comparison would stay green even if the voice rewrote every
  sentence in the repository.
- **Not built yet.** This is the last spec commit of the round. Stage **B20**
  follows: `si-core::tongue` and the eight rows with the `0x0002 → 0x0003` bump and
  its three golden re-pins **in the same commit**, then the `si-tongue` crate, the
  one call site, then G20(a)–(k) and the 52-id check union.

## 2026-10-25 — **Step 8a: the tongue is folded as law — G20, OD-163 … OD-181**
- **What arrived.** The owner queued a new idea mid-run ("this is the piece that
  will let our SI have some randomness in talking within bounds of knowledge so
  it's not as 'form'") and delivered it as [27-tongue.md](../27-tongue.md):
  surface realization by walk, five style draws, a token-by-token generator over
  the term lattice, three structural checks as the gate. The same message carried
  the routing — Logic → Engineer Design → Code → Test → DoD, "against everything we
  have so its not brokn" — and the amendment in §27.11 rules the mechanism: the
  generator **is** the walk, and the draft's variant bank is demoted to anchors.
- **What this commit is.** The law fold only, ahead of any code: the draft's items
  re-based into the ledger at **+33** (the ledger was at OD-162; the draft claimed
  OD-130…OD-143, and its two re-asks merged into OD-165 and OD-168), then the
  ripple across [03](../03-birth-state.md), [07](../07-read-path.md),
  [11](../11-stages.md), [12](../12-component-map.md), [13](../13-gate-plan.md),
  [14](../14-risk-register.md), [15](../15-definition-of-done.md) and
  [gates/README](../../gates/README.md). Twenty gate families now, G20 as the
  eleventh in docs/15's order, and R-43 as the risk the tongue brings.
- **The catch worth the whole fold (OD-181).** §27.2's gate as drafted checks that
  every realized word *names a term row*, and `G20(h)` greps the whole registry —
  a leg whose empty set is the pass, against a set containing every word SI knows,
  **cannot fail**. [07 §7.5](../07-read-path.md) is older law and says the answer's
  content tokens must come from *cited* rows. So a line can satisfy the drafted
  gate and still assert an uncited row, and because the tongue runs *after* the
  label it has no demotion available: R-43's fluent lie walking through the gate
  built to stop it. Ruled in §7.5's favour — a line utters only the surfaces of the
  points **its own citation names**, plus the composed chunks §7.5 always allowed;
  the walk may traverse further than it may speak. The cost is recorded rather than
  absorbed: SI can only sound fluent in connectives it was taught, because a word
  is a row.
- **Three clashes, each settled toward the law that already existed.** The
  closed-18 problem (OD-165/OD-175): `check_temperament_closed` + `G14.c.3` make
  the species vector's length a schema fact, so adding five rows would bump
  `seed_format` and invalidate every individual this repo has recorded — the round
  takes a separate `DERIV_ORDER_TONGUE` keyed by the same `birth_seed`, plus a
  sixth registry class, and nothing already measured moves. Sweep vs temperament
  (OD-177): the draft's "no value before its sweep" loses to **OD-87**, so the
  style rows are never swept and 05 §4.1's panel pass validates their boxes.
  Entropy (OD-176): spending the session's runtime stream at speak time would move
  hash item 7 for no temperamental reason, so each render gets its own PRNG seeded
  from the snapshot, and `G20(a)` is a real cross-process byte-identity leg.
- **What it costs, said here first.** The call site changes the binary ⇒
  `CONSTS_VERSION` `0x0002` → `0x0003` ⇒ **every recorded control in `gates/` is
  re-signed by re-running it** (OD-91), never by editing a number. `G20(j)` — V0
  string byte-identity including `s1–s27` — is the leg that proves the strings
  themselves did not move, and it is the reason the switch ships `off`.
- **Next.** [engineering/13-tongue.md](../../engineering/13-tongue.md) (the HOW),
  then `si-core::tongue` + registry rows, the `si-tongue` leaf crate, the one call
  site, G20's eleven legs, and the build's own DoD pass.

## 2026-10-25 — **Step 7: the seed panel exists, and 05 §19 is measured rather than promised (OD-161, OD-162)**
- **What this stage is.** [05 §19](../../engineering/05-gate-harness.md) makes G14 a
  gate about an *individual*, and an individual is only comparable to other
  individuals: legs (a)(b)(d)(f)(i) and the prerequisite `seed_panel` are
  cross-seed or cross-process claims that a single run cannot grade. Until B12's
  panel pass, every one of them was UNMEASURED and §19's own closing paragraph
  forbade quoting any spread or "how different are two seeds" magnitude. This stage
  builds the panel, runs it, and converts the legs from promises into measurements —
  `harness/scenarios/g14_seed_panel.json` (18 members), `harness/scenarios/g14_twins.txt`
  (the deliberately seed-less shared transcript), `crates/si-cli/src/harness/panel.rs`
  (rule, guard bytes, LAW bytes, §19(e)'s exact box arithmetic, the artifact writer)
  and the `G14`/`g14_block` legs in `gates.rs`.
- **The rule, not a hand.** 03 §7 asks for ≥ 16 fixed `u64`s "produced by one
  published rule"; §4.1 step 4 forbids re-sampling the panel until a leg passes,
  which is what makes hand-picking a corner-witness seed illegal. So members 1–2 are
  the two seeds 05 §2 itself names (`0x…51`, `0x…00`) and members 3–18 are
  `u64::from_le_bytes(md5("SI-panel-<k>")[0..8])`. `rule_seeds()` re-derives the list
  on every read and `G14.panel` fails on a mismatch: an edited member is a named
  contradiction, not a quiet cherry-pick. The asset sits under `harness/`, so it
  enters `runner::tree_digest` and `binary_tree_md5` — editing the panel changes the
  build stamp. Raised for ruling as **OD-162**.
- **The artifact outlives its binary, and that broke a leg.** `gates/` is ignored and
  `seed_panel.json` accumulates across rebuilds, so a panel comparison that mixes two
  binaries compares a guard surface, a LAW table and a derivation from different
  builds — OD-91's category error. `panel::record` now stamps every row with
  `runner::tree_digest`, `PanelRecords::of` keeps only same-build rows, and the count
  it excluded is reported rather than hidden. The same read fixed §19(d): "every gate
  holds for both" reads the **newest** record per (seed, gate, variant, scenario),
  because graded against every row ever written, G14's own incomplete early rounds
  would poison the leg forever.
- **`G14.f.1` was wrong twice before it was right.** As written it was unmeasured in
  every run. Graded globally it then failed 13 battery runs — not a temperament leak:
  `guard_bytes` covers the Species and Ruling rows a launch is *allowed* to move, so a
  run launched at a different constant point holds a different guard surface by
  design. 05 §19(f) never asked for a cross-launch comparison: its procedure is
  "**V0 vs V8** diff over the whole registry" and its assert is "every **LAW** row is
  byte-identical across all panel seeds". The leg is scoped to match: LAW digests
  compared across every same-build record (1 value), guard digests only within the
  run's own `launch_digest` + transcript group (1 value across the 18 members), and a
  record with no launch stamp goes to a named `unrecorded` group instead of joining
  the current one silently.
- **Two birth litmuses were lying, and neither was about temperament.** `G14.a.1`
  failed at every seed with `interner_bytes: 8`: 01 §7 reserves handle 0 as `""`, and
  `encode_empty_interner` writes exactly those 8 bytes, so "the interner is empty" is
  `interner().len() == 1`, not `is_empty()`. `G14.g.2`'s banned-adjective list
  contained the word **`nickname`** — the same word the pane's own "no nickname"
  refusal line must print — so the leg graded the ban as the crime; it is out of the
  list, and the refusal line stays.
- **§19(e) is arithmetic, and the arithmetic says no.** `e.3` claims every box corner
  is reachable by some panel seed. The draw is a Q12 fraction
  (`rng::next_q12`, 4096 values, not 65536), so `map_fraction(4095, 0, 65535, 1) =
  65520`: for the four `0..65535 grid 1` rows the `hi` corner is unreachable by **any**
  `u64` seed — 16 of 36 corners, computed by enumerating all 4096 fractions per row
  rather than sampling. §4.1 step 4 forbids fixing that by picking better seeds, so it
  is raised as **OD-161** with three rulings and a recommendation, and `e.3` is
  published as UNMEASURED. Only `LAMBDA_A` (1..255) and `ASK_CAP_PER_INPUT` (1..4)
  reach both corners.
- **Measured.** Five rounds of the panel (18 members each; rounds 4–5 after the
  `f.1` scoping): **all PASS**, 26 legs, **23 graded / 3 unmeasured** (`e.3` → OD-161,
  `h.1` and `k.1` → the standing OD-83 sweep-publication and OD-74
  `MOD_CURVE_VARIANTS` refusals). 18 individuals × 17 gate families recorded in
  `gates/G14/seed_panel.json`, 95 same-build records, 0 failing. Consolidated verify
  pass: **336 tests / 0 failed**, `cargo clippy` exit 0 (the two warnings are in
  untouched `bin/` files and predate this stage), `si-harness check --all` = stage
  B13 **46 checks PASS**, 41 placeholders, 0 disabled.
- **Not claimed.** No spread, histogram or "individuals differ by X" magnitude is
  quoted: 95 records is an index of *which gates ran at which individual*, and §19's
  closing paragraph still makes the interesting-ness a research readout for the
  owner, not a gate criterion. `h.1` remains UNMEASURED precisely because the box
  validation it wants is a sweep winner, and OD-83 says there is no published sweep
  winner to re-run.


## 2026-10-25 — **Step 6: the arc's per-crate DoD tests exist, and one of them had to be placed against two checks' blind spot (OD-160)**
- **What this stage is.** [11 §B14 DoD 1](../../engineering/11-build-plan.md) asks for
  "`cargo test -p si-core -p si-walk -p si-crystallize` ⇒ exit 0" asserting "the three new
  packages exist with **exactly** the edges of 12 §1.1 (a graph snapshot test), the frame scratch
  is constructed and dropped inside one frame, the four rules fire in order, and a primed frame
  increments no recurrence", and §B15 DoD 1 asks `si-silt` for "drop-at-`THETA_SILT` sets 0 and
  removes the entry". Measured in this repo, `si-crystallize` shipped **0 tests**, `si-silt` had no
  theta leg and `si-walk` no scratch leg (`si-skills`' 7 tests already covered §B17 DoD 1) — the
  first tally had been read off the wrong tree. DoD 1 was not green. This stage closes exactly
  that, and does not re-write an assert a static check or a gate leg already grades.
- **The four rules, as a suite that proves its own ordering**
  ([`crates/si-crystallize/src/lib.rs`](../../crates/si-crystallize/src/lib.rs)). Four tests: the
  rule-legs are each built so the rule *before* the one under test is the only one that could
  deny it (the `NoSharedFiller` frame names a member set rule 4 would also deny, so 2 demonstrably
  ran first; the `AlreadyBound` frame carries a filler kind rule 2 would clear), plus
  OD-105's warm-frame rule read from the detector's side — a `primed` light leaves
  `table_bytes()` byte-identical and never moves `last_seen_t`, so the next cold frame still
  clears the delta — that recurrence counts **frames** not calls, and that a candidate nominates
  exactly once.
- **The crate edges, asserted where nothing asserted them**
  ([`crates/si-crystallize/tests/edges.rs`](../../crates/si-crystallize/tests/edges.rs)).
  `check_crystallize_no_commit` only ever *denies* names, never scans `si-silt`, and nothing in the
  workspace tested the reciprocal rows. The file pins the exact dependency set of the three reader
  manifests (and their empty `dev-dependencies`), and asserts `si-walk`/`si-learn`/`si-front`/
  `si-read` declare none of the three while `si-session` declares all three. Placement note, said
  plainly: it is one file in `si-crystallize` that opens the other two manifests, not three
  same-named files, so `cargo test -p si-silt` does not itself run `si-silt`'s edge row — the law
  is 12 §1.1's and it is graded, just not from inside each crate.
- **§B15's theta drop, and §B14's "scratch is constructed and dropped inside one frame".**
  `si-silt` gained the armed-vs-`THETA_SILT = 0` paired table: the same seed, the same ticks, and
  the row leaves the armed table while its level is still in `(0, theta)` and is gone from
  `view()`/`readout()` — strictly before the control's own floor-to-zero tick (the decay
  **compounds**, so the naive single-curve model I first wrote was wrong and its test failed at
  tick 18). `si-walk/tests/scratch.rs` walks with the caps armed and asserts the ghost and role
  graph *are* this frame's `FrameLog` read back once — then walks with `ghost_max = shape_max = 0`
  and asserts every activation, light, round, settle set and stop reason is **identical** to the
  armed walk, which is 04 §31 AC.2 / OD-99's degradation-by-omission made checkable.
- **What gates and checks grade instead of a unit test, so nothing is double-counted.** B17's "a
  forwarded shape … its frames stay visible to §29's fold" is leg **G18.f.2**
  ([`crates/si-cli/src/harness/gates.rs:10296`](../../crates/si-cli/src/harness/gates.rs#L10296));
  B15's "consumes `Clock::recency` and defines no curve of its own" is `check_one_clock` (exactly
  one `fn recency(` in the workspace); B15's "the crate imports `si-core` only" and B17's
  no-commit row are `check_crystallize_no_commit`/`check_skills_no_commit` plus `check_deps`. None
  of those three got a duplicate assert here — only the genuinely-absent legs were written.
- **The wall, taken as OD-160 rather than worked around.** `check_ghost_is_frame_scoped` lets a
  `CoLight` be *constructed* only inside `si-walk`, so the four-rule frames could not live in
  `tests/`; moved inline, `check_crystallize_no_commit` then denied the fixture's own
  `&mut Lattice`/`append_row` helpers — which is how both DoD 1 legs became ungrunnable at the same
  time. The build choice is the placement the crate idiom already supports (a `#[cfg(test)]`
  module) plus the **same** exemption the ghost check already applies to its own scan, extended to
  the commit check: a `#[cfg(test)]` module compiles into no shipped binary, so it cannot carry the
  write capability that check exists to deny. **Proved, not asserted**: a temporary `probe()`
  under the marker with `&mut Lattice` and `append_row` made the check FAIL at the right line
  before removal, and every line above the marker stays scanned. Recorded with the alternatives
  (widen the ghost check to `crates/*/tests/`; a `FrameLog::from_json` dev-dep that still names
  `FrameLog` and breaks `core, store ONLY`; deferring to G15, whose floors never fire
  `AlreadyBound`/`InsideDelta`/`primed`) in [OD-160](open-decisions.md), status RESOLVED by the
  build.
- **Disclosed, because it happened earlier in this night's run and no entry records it.**
  Rewriting one of `gates.rs`'s comment lines through Python's `open(…, 'w')` with a Windows
  default (cp1252) encoding destroyed that 11,332-line file mid-run; it was rebuilt, and the
  rebuild was **proved** the way B7 asks a control to be proved — by re-running everything it feeds
  and getting the recorded measurements back, not by eye. The rule this leaves for the rest of the
  build: never rewrite a repo file through a Python text-mode `open` without an explicit
  `encoding='utf-8'`, and read gate reports with `io.open(..., encoding='utf-8')` for the same
  reason. The `check --all` and 16 gate families below are the evidence the rebuild is sound.
- **Verified:** `cargo test --workspace` **336 passed / 0 failed** (was 326); `cargo clippy
  --workspace --all-targets` exit **0**, 154 warnings — unchanged, none in a new file; `si-harness
  check --all` **46 checks PASS**, placeholders 41, disabled 0; 16/16 gate families `PASS` at seed
  `0x51` (`G1 28/4 · G2 17/9 · G3 14/2 · G4 14/2 · G6 10/1 · G7 17/2 · G8 29/4 · G9 24/12 ·
  G10 25/2 · G12 24/1 · G13 47/5 · G15 19/3 · G16 20/3 · G17 20/1 · G18 23/3 · G19 17/1`, all in
  `gates/*_si_20261024-2342`; 348 criteria — 293 `pass`, 55 `unmeasured`, **0 fails** — same as
  before this stage, so nothing moved).
- **Still walled:** **OD-83** (B12's sweeps and B13's global acceptance — `--strict-consts` still
  refuses, placeholders 41), the owner's three open asks **OD-157/OD-158/OD-159**, and DoD 5's full
  sweep of every gate run against V0.

## 2026-10-25 — **OD-152 dissolved by derivation: `G19(g)`'s law is three claims, and clippy exits 0**
- **What this stage is.** The owner's rule for the night: dissolve uncertainties by
  measurement, then take the weak spots. The weak spot named in the ledger was the only
  **denied** lint in the build — [OD-152](open-decisions.md)'s duplicated conjunct in the
  `G19(g)` starvation law, which made `cargo clippy --workspace --all-targets` exit 101 while
  `cargo test --workspace` and `check --all` were green.
- **The question answered itself when the producer was read.** OD-152 asked whether the dead
  operand was meant as (b) nothing or as (c) the converse implication `!denied || suppressed`.
  [`crates/si-session/src/lib.rs:3076-3077`](../../crates/si-session/src/lib.rs) is
  `suppressed_by_window = denied && agenda_clear && within_rate` and
  `if agenda_clear && within_rate && !denied { ignite }`. From that definition
  `suppressed ⇒ denied` holds, the **guarded** converse `denied ∧ agenda_clear ∧ within_rate ⇒
  suppressed` holds (and is leg 4, with `quiet_grew` the harness's proxy for the two
  preconditions), and the **unguarded** converse (c) is *false*: a window denial while the agenda
  is busy or the idle rate is spent suppresses nothing, because there was nothing there to
  suppress. Restoring (c) would have made a shipped gate grade a lie, so the surviving law is the
  three claims the spec states — [05 §23.5's (g)](../../engineering/05-gate-harness.md) "idle
  ticks occur only where the window is open *and* the quiet-and-empty-agenda law already held;
  no tick occurs *because* a window opened" — and each conjunct now carries the derivation in a
  comment. This is a correction of a dead operand, not a widening: nothing was removed that ever
  constrained a verdict.
- **Behaviour-preserving, measured rather than asserted.** `G19` at seed `0x51` before and after
  (`gates/G19_si_20261024-2318`, `gates/G19_si_20261024-2321`) publish an identical
  `state_hash.txt` (md5 `0f5af6b1b8674f9214e6566143683b48`), an identical `transcript.txt`
  (`17f514c413387fd28ad8182c4c703be9`) and an identical `G19.g` `observed` block; 17 legs / 1
  unresolved and `PASS` both times. B7's rule applies: the control's identical hash **is** the
  pass.
- **Verified:** `cargo clippy --workspace --all-targets` exits **0** (≈100 style warnings remain,
  no denied lint); 16/16 gate families `PASS` at seed `0x51` (`G1 28/4 · G2 17/9 · G3 14/2 ·
  G4 14/2 · G6 10/1 · G7 17/2 · G8 29/4 · G9 24/12 · G10 25/2 · G12 24/1 · G13 47/5 · G15 19/3 ·
  G16 20/3 · G17 20/1 · G18 23/3 · G19 17/1`, all in `gates/*_si_20261024-2321`);
  `cargo test --workspace` **326 passed / 0 failed**; `si-harness check --all` `pass: true`
  (placeholders 41, disabled 0).
- **Still walled:** **OD-83** (B12's sweeps and B13's global acceptance — `--strict-consts` still
  refuses, placeholders 41), the owner's three open asks from steps 3–4 (**OD-157** the seven
  verdicts for §6(u)'s five rows, **OD-158** a refusal's own frame, **OD-159** whether `G1.u.2`
  must run the `si-cli feed` binary cross-process), and DoD 5's full sweep of every gate run
  against V0.

## 2026-10-25 — **G1.u step 4: the feeder is tested as a program, and one nightly flake was caught in the act**
- **What this stage is.** Step 3 left a fidelity question open: [09 §10's `u.2`
  row](../../engineering/09-teach-channel.md) says "fed through **the feeder** … ⇒ byte-identical
  `state_hash.txt`", and the shipped leg reached its equivalence in-process, so the body of
  `si-cli feed` — the filter, the append, the strict walk — had no witness. No spec row changed;
  what the harness can and cannot grade is now said out loud as
  [OD-159](open-decisions.md).
- **`G1.u.2` now travels the transport's storage.** The fed lane is written into the run
  directory as `g1u2_fed_lane.siro` and **read back off the disk** before the loop hears it, so
  the graded bytes are the file the owner's feeder writes. Measured at seed `0x51`, V0,
  `gates/G1_si_20261024-2317`: 439 bytes, `lane_file_survived_the_disk = true`, 6 frames written /
  6 heard, `torn_tail_bytes = 0`, fed hash `d0dccedede4a625150194bae119077f6` = typed hash, and
  the one-line-short control still differs at `26736a8fdb41e1c1df28a2b86535ffac`. The writer stays
  the loop's single one, so no second feeder appeared.
- **The feeder's own file handling, tested by the code that owns it.** Three tests in
  `crates/si-cli/src/bin/si-cli.rs`, run against real files and no subprocess: the filter (a
  header comment and blank lines dropped, trailing spaces trimmed, order kept, and the directory
  holds **only** the curriculum file and the lane), the append (a second feed lands behind the
  lane already written, never in front of it), and the strict walk (a last frame cut 3 bytes
  short refuses with the named error `the lane on disk is torn`, and the lane's bytes are
  byte-for-byte what they were before the refused feed).
- **A flake found by running the suite, not by writing it.**
  `si-store/tests/genome.rs::pipeline::the_loader_body_records_its_one_frozen_open` failed 1 run
  in 15 (`23 passed; 1 failed`) with its own trace holding **two** artifacts. The open register is
  one process-wide log and cargo runs tests on parallel threads, so when the traced test turned
  `SI_TRACE_OPENS` on, a neighbour's unlocked `load(Config::Engine, …)` recorded *its* artifact in
  the same log — `["open-ro …si-b10-…-opens\…\SIgen", "open-ro …si-b10-…-loader\…\SIgen"]`. Every
  test that opens a frozen layer now takes `TRACE_LOCK`, which is what the register's
  count-based claims ("its **one** frozen open", "the refusal names **no** path") silently
  assumed all along. Measured after: **0 failed in 30 runs**. This is the shape of failure a
  nightly regression hides — a neighbour's schedule can read as a regression, and a real one can
  read as that noise.
- **Verified:** 16/16 gate families `PASS` at seed `0x51` (`G1 28/4 · G2 17/9 · G3 14/2 · G4 14/2 ·
  G6 10/1 · G7 17/2 · G8 29/4 · G9 24/12 · G10 25/2 · G12 24/1 · G13 47/5 · G15 19/3 · G16 20/3 ·
  G17 20/1 · G18 23/3 · G19 17/1`, legs/unresolved), `si-harness check --all` `pass: true`
  (placeholders 41, disabled 0), `cargo test --workspace` **326 passed / 0 failed** (323 plus the
  three feeder tests), and the genome suite hammered 30× at `--features engine` with 0 failures.
- **Still walled:** B12's sweeps and B13's global acceptance wait on the owner's
  sweep-enumeration ruling (**OD-83**; `--strict-consts` still refuses, placeholders 41),
  **OD-152** (G19(g)'s duplicated conjunct — a question about the law, not a typo to guess at),
  **OD-159** (whether `G1.u.2` must run the `si-cli feed` *binary* cross-process, which needs a
  mechanism this build does not have: 0 hits of `Command::new` workspace-wide), and DoD 5's
  full sweep of every gate run against V0.

## 2026-10-25 — **G1.u step 3: the `/teach` bulk family is whole, and one over-strict leg was corrected**
- **What this stage is.** The weak spot left by B11.7: [05 §6(u)](../../engineering/05-gate-harness.md)
  and [09 §10](../../engineering/09-teach-channel.md) name five `/teach` bulk legs and steps 1–2
  had shipped the shared reader and `G1.u.2` only. `G1.u.1`, `G1.u.3`, `G1.u.4` and `G1.u.5`
  are graded here, on the checked-in fixture §6(u) itself names. No spec row changed.
- **`G1.u.1` — the left-side accept set through the tick.** The six lines
  `check_teach_left_side` grades statically are driven as real teaches; the loop's own
  `EV_DECODED` bodies are the witness (four commits say nothing and carry 1 row, two
  statement-shaped refusals say `left_not_a_question` and carry 0) and the shelf count is
  its twin: `s_r_pairs = 4`.
- **`G1.u.3`/`u.4`/`u.5` — one 200-line import, three readings.** `harness/scenarios/
  g01_u_curriculum_200.txt` (fifty entities × four passes, digest `7609e1cb88098b8af15c6e14d92042c8`)
  is written by the loop's own lane writer and heard before the steps: 200 frames → **50
  uncoversed rows** and `repeats` summing to **200**, shelves `{4: 50, 8: 200}`, `y` exactly
  `1..=50` / `1..=200` with no `row_id 0` — so nothing was addressed outside `(t, x, y)` and
  no "import" row type appeared (R-12's alarm). Every taught row reads `gain_kind = asserted`
  (R-07 through the newest door), and 50 of 50 stamped edges recompute as
  `bake_q12(a_at, B_MAX, K_TANH)` with 0 mismatches — the birth cohort's bake is reviewable,
  not trusted (04 §16 P).
- **Two clauses have no producer, and they are said out loud.** `u.4`'s served-recall
  promotion count belongs to `INFERENCE_PROMOTION`, a Ruling row that ships off (OD-58's
  conditional, the same wall as `G13.i.2`); `u.5`'s "`a` stayed high" names no number in the
  registry, and inventing one while coding would be the ad-hoc cap **OD-33** owns. Each row
  is split into graded + `open()` halves, so G1 reports **28 legs / 4 unresolved** instead of
  quietly 22 / 2 — recorded as [OD-157](open-decisions.md).
- **A red gate found by running the family, not by writing it:** `G12.b.2` demanded the
  refused emission own **no event**, which was true until B11.2 gave the seam its 08 §2.1
  `seam_reject` frame — the frame §7 requires so a dropped voice stays visible. 05 §17(b)
  prices the refusal in **state**; the harness clause was the over-strict one. It now asserts
  the step's only event is its own rejection (`event_kinds: [17]`), every state clause
  unchanged — [OD-158](open-decisions.md).
- **Verified:** 16/16 gate families `PASS` at seed `0x51` (`G1 G2 G3 G4 G6 G7 G8 G9 G10 G12
  G13 G15 G16 G17 G18 G19`), `check --all` 46/46 `PASS` (placeholders 41, disabled 0),
  `cargo test --workspace` 323 passed / 0 failed. `G1.u.2` still reproduces
  `d0dccedede4a625150194bae119077f6` for both arms and `26736a8fdb41e1c1df28a2b86535ffac`
  for its one-line-short control.
- **Still walled:** B12's sweeps and B13's global acceptance wait on the owner's
  sweep-enumeration ruling (OD-83; `--strict-consts` still refuses, placeholders 41), plus
  OD-152's clippy `eq_op` decision and DoD 5's full sweep of every gate run against V0.


## 2026-10-24 — **B11.7: the fed inbox has a reader, the live lane is measured, and a dead window cannot stop a wake**
- **What this stage is.** Not new surface — the two uncertainties B11 shipped with, taken
  to a run instead of argued, plus the weak spots they exposed. [11 §4](../../engineering/11-build-plan.md)'s
  B11 row is unchanged; B12 stays walled on the unrulled sweep enumeration (#48 /
  [OD-83](open-decisions.md)).
- **Uncertainty 1 — was the live lane ever exercised?** B11 only ever ran the
  `mindlog.json` re-frame, so `RECONSTRUCTED` might have been the only path that works.
  It is not: 322 gate directories carry a real `stream.siro`, and `si-cli view` pointed at
  `runs/b11-live` tailed **25478 lane bytes → 60 frames, 8300 quads on the last frame,
  0 over budget, 310 rows not shown**, against `runs/b11-view`'s **1538-byte re-frame →
  120 frames, 2483 quads, 22 rows not shown**. Both logs say `state_hash unchanged true`.
  The two are now distinguishable in writing — `viewer.log` carries a
  `lane stream.siro live tail N bytes` / `lane mindlog.json re-framed N bytes` line
  ([`si-cli.rs:653`](../../crates/si-cli/src/bin/si-cli.rs)) and
  `check_headless_run` **refuses a log that names no lane**
  ([OD-150 addendum](open-decisions.md)).
- **Uncertainty 2 — the HEADLESS law's second half** (B13 DoD 5: *killed* mid-run, not
  merely absent). Three arms over the same 300-line transcript: headless
  `runs/hl-long`, viewer alive throughout `runs/hl-alive`, viewer
  `Stop-Process -Force`'d **1.2 s into a 2.8 s run** `runs/hl-kill` — all three settle at
  **`2a8155dd0936ef913e33e0f4c0bf9429`**, and the killed arm's directory holds the same
  four artifacts as the headless one. The honest limit, stated rather than hidden: a
  killed viewer never reaches its exit block, so it writes no `viewer.log`, so **no check
  can grade this** — the artifact is on this machine (`runs/` is gitignored) and the gate
  leg stays UNMEASURED until B13 sweeps it over every run.
- **Weak spot 1 — the `/teach` feeder wrote a lane nothing read.**
  [09 §10](../../engineering/09-teach-channel.md) makes `si-cli feed` a pure writer and
  09 §8 says "everything that happens afterwards is the ordinary loop" — but no part of
  the loop opened `sense_inbox.siro`. Measured: fed two lines then ran
  (`runs/feed-gap`) = `c9654e12086696a0aae43a459833b5ec`, and nothing fed
  (`runs/feed-gap-control`) = **the same digest**. A feeder that provably cannot change the
  state is the exact failure `G1.u.2` exists to catch. The drain now runs ahead of the
  transcript block, one `tick_frame` per frame in file order
  ([`si-cli.rs:194`](../../crates/si-cli/src/bin/si-cli.rs)), and **`runs/g1u-fed` ==
  `runs/g1u-typed` == `b71cf552d1219f4cfb8caeff27eb40c8`** with `mindlog.json` differing on
  the `"run_id"` line only. `check_curriculum_is_inbox_only` gained the reader half so this
  cannot regress into a writer-only proof ([OD-154](open-decisions.md)); the
  `transcript.txt` leg of `G1.u.2` stays **UNMEASURED from the CLI**, because `si-cli run`
  writes four files and `transcript.txt` belongs to `Session::write_run_dir`, which no CLI
  arm calls.
- **Weak spot 2 — a real defect, found by pointing the viewer at a gate run.** A run with
  N settled sections writes an **N-line** `state_hash.txt` (measured: `runs/b11-live` = 3
  lines) while `viewer.log` keeps one value per key; the raw write put two unlabelled
  digests into the log and the check compared one line against the whole file. Folded on
  both sides ([`si-cli.rs:610`](../../crates/si-cli/src/bin/si-cli.rs),
  [`checks.rs:4403`](../../crates/si-cli/src/harness/checks.rs)) — digests unchanged, only
  the separator — and the leg now says **"digest-for-digest in file order"**, not
  "byte-identical", because after the fold the two texts are deliberately not the same
  bytes ([OD-155](open-decisions.md)).
- **Weak spot 3 — §2.2's dead-window row against the loop's own reader.**
  `read_sense_inbox` **bailed** when a frame ran past its payload: a viewer killed
  mid-append could have made the next wake refuse to run, which is an observer holding a
  veto over the loop (§1's HEADLESS law). The reader now stops at the last whole frame and
  returns `Accepted { frames, torn_tail_bytes }`
  ([`si-session/src/lib.rs:609`](../../crates/si-session/src/lib.rs)), the run prints the
  tail as its own line and applies nothing invented, and the display crate keeps its
  refusal ([`si-vulkan/src/frames.rs:110`](../../crates/si-vulkan/src/frames.rs)) because
  the loop publishes its lanes atomically — a torn display lane is a corrupt artifact, not
  a concurrent append. Proved by a new `world.rs` test at both cuts: mid-body
  (`4 + len(b) − half` tail bytes) and a 2-byte stub, which the while-guard rather than the
  break handles ([OD-156](open-decisions.md)).
- **The drain's three branches, each proved on a real lane file.** The unit test covers the
  decode; these runs cover the CLI, and the control digest does the arguing:
  1. **whole lane** — the 216-byte fed lane hears 2 frames → `b71cf552d1219f4cfb8caeff27eb40c8`;
  2. **torn tail** — the same lane with its last 5 bytes absent (a window killed
     mid-append) hears 1 frame and prints "70 torn tail byte(s) were never written and are
     not heard" → `fa31c2027d1a2716a7af4ee03d88a9f4`, i.e. the state of *one* teach, neither
     the two-frame state nor a refusal; a 32-byte absence names 43 torn and settles the same
     digest, because the cut is counted from the last whole frame, prefix included;
  3. **refusal** — a lane whose container version is rolled back to 1 prints "sense inbox
     container version 1 is not 2", and a v2 lane whose frame body is 6 bytes (no word
     beside the 12-byte transport prefix) prints "sense frame 0 carries no word beside its
     12-byte transport prefix" — and **both settle at the nothing-fed control digest
     `c9654e12086696a0aae43a459833b5ec`**. "No frame was applied" is therefore not a promise
     in a string: a refused lane is indistinguishable from an absent one.
- **One rule broken and corrected the same night.** Two comments cited 08 §2.2 with the
  quotation *"reads as much as has been written so far"* — §2.2 contains no such sentence
  and the grep found it nowhere in `docs/` or `engineering/`. Both sites now cite the rows
  that exist (§2.2's late-start row, read at its limit). An invented cite is worse than no
  cite: it makes the next reader trust a law that was never written.
- **The gate.** `cargo test --workspace` → **75 groups, 323 tests, 0 failed** (the +1 is
  the torn-lane test); `si-harness check --all` → **`stage B13 — 46 checks, PASS`**, exit 0
  (`gates/checks/stage-B13.json`: 46 enabled, `placeholders: 41`, `disabled: 0`);
  `check --stage B11` → PASS, exit 0; `cargo build` warning-free. No dependency, schema,
  gate-leg or constant changed; `Cargo.lock` and `gates/deps_lock.md5` untouched.
- **Three seats ledged:** [OD-154](open-decisions.md) (the feeder had no reader; plus the
  `transcript.txt` question it cannot answer yet), [OD-155](open-decisions.md) (N-line hash
  vs key-per-line log), [OD-156](open-decisions.md) (reader tolerates a torn tail, display
  refuses) — and OD-150's addendum records that the live tail is no longer an unexercised
  branch.

## 2026-10-24 — **B11 SHIPPED: the viewer draws five panels, reads zero artifacts, and writes exactly one file**
- **What this stage is.** [engineering/08 §1–§9](../../engineering/08-gui-observability.md)'s
  five panels as the `si-vulkan` crate — lane decode (`frames`), the §5 layout rule
  (`layout`), panel content (`panels`), the baked glyph sheet (`atlas`), the two
  hand-assembled shaders (`spirv`), and the window + swapchain + `genome_residency`
  (`surface`) — plus [05 §20](../../engineering/05-gate-harness.md)'s three legs as
  harness checks and `si-cli view --session runs/<id> --exit-after-frames 120` as
  process B. §1's three laws are not prose here: they are the crate's own test group,
  [crates/si-vulkan/tests/laws.rs](../../crates/si-vulkan/tests/laws.rs), 14 legs.
- **The Handover (OD-148).** 00 §8's table says the viewer opens the lane files;
  08 §2/§8 says process B holds no path to a lattice. The build takes the **narrower**
  reading: `si-cli view` — the launcher, which already owns the schema crates — decodes
  `SIst` / `mindlog.json` / `sleep.jsonl` / `fingerprint.txt` and hands the panel
  *values*, and [`genome_residency`](../../crates/si-vulkan/src/surface.rs) takes
  **bytes, never a path** ([`surface.rs:1733`](../../crates/si-vulkan/src/surface.rs)).
  Measured by `check_gui_observer_only`: si-vulkan opens **exactly 1 file in the whole
  crate** — the writable one,
  [`surface.rs:347`](../../crates/si-vulkan/src/surface.rs) — and **reads 0 artifacts**.
- **The lane got its missing voices.** Kinds 3/7/9/17/20 now have producers, the parser
  covers §2.1's 0–20 and refuses kind 18 by name as reserved
  ([`frames.rs:136`](../../crates/si-vulkan/src/frames.rs)), and the idle-ignition
  record sits **above** the sealed table as kind 21
  ([`eventlog.rs:50`](../../crates/si-core/src/eventlog.rs)) because §2.1 is sealed at 20
  and its own note forbids a "this was a dream" flag on the kinds that already exist
  ([OD-143](open-decisions.md)).
- **The gate.** `si-harness check --stage B11` → **46 enabled, 0 failing, exit 0**
  (`gates/checks/stage-B11.json`), including DoD 3's C10 + `check_gui_observer_only` +
  `check_no_viewer_state_write`; `check --all` prints **`stage B13 — 46 checks, PASS`**.
  `cargo test --workspace` → **75 groups, 322 tests, 0 failed**; `cargo test -p si-vulkan`
  → 3 unit + 14 laws, 0 failed; `cargo build` warning-free.
- **The window run.** `runs/b11-view/viewer.log`: exit 0, *"120 frames presented on
  NVIDIA GeForce RTX 3060; last frame 2483 quads drawn, 0 quads over the batch budget,
  22 rows not shown"*, and **`state_hash` before == after ==
  `e98db13d74aead4639ea0a90b9e51a2d`** — the OBSERVER-ONLY law graded as a byte-compare,
  not asserted. **No human looked at the window**: 11 §R7 leaves DoD 4 unperformed unless
  the owner asks, so the log is a program's self-report and nothing more. Two honest
  limits on the same leg: no `stream.siro` exists at CONFIG SI yet, so the launcher
  re-frames `mindlog.json` and marks it
  [`RECONSTRUCTED`](../../crates/si-vulkan/src/panels.rs) instead of passing it off as a
  live tail (OD-150); and `check_headless_run`'s artifact half compares the **one**
  viewer-present run it can find — the full sweep over every run against V0 is B13 DoD 5,
  and the detail says UNMEASURED on a checkout with no artifact.
- **Four pre-existing scans read the viewer as a violation; every repair added an
  assertion** (OD-151). `check_speaker_class_derived` (the viewer names
  `speaker_identity`) → `IDENTITY_HOMES` 6→8, because a kind-0 sense frame's *first
  field* is the identity and the crate that mints it must be a home.
  `check_windows_suppress_only` (winit's `Window` is not the interval's `Window`) → the
  viewer is now scanned against the interval's own five spellings, so OD-126's law stays
  enforced **by name** instead of by a blind spot. `check_config_guard` (`SIgen` in
  viewer prose, tests and one refusal message) → the scan reads **code text** rather than
  raw text and the message was reworded. `check_no_seed_control_surface` (§9's refusal
  table *lists* widget names) → the table is exempted **by span**, and a new leg requires
  a refusal table to exist at all: no table, fail
  ([`checks.rs:2590`](../../crates/si-cli/src/harness/checks.rs)). Plus
  `check_no_embeddings` (a `layout.rs` doc line) and `gates/deps_lock.md5` republished to
  `2449f2c6c8828b801d361b6457a427d5` — the stage enables the **already-allowlisted**
  `ash`/`winit`/`raw-window-handle` edges, so 00 §9's table did not widen (OD-153 records
  the one place it is stale: si-vulkan's `serde_json` row).
- **Where viewer SPIR-V comes from (OD-147).** There is no `glslang`/`shaderc` row in
  §9's closed allowlist and 08 §3.1 says the atlas has no build step, so the two shaders
  are **hand-assembled words** in [`spirv.rs`](../../crates/si-vulkan/src/spirv.rs),
  walked by unit tests so a miscounted opcode fails on the CPU rather than as a driver
  error at runtime. `spirv-val` from the local SDK is **review-time only, never a build
  dependency**; it caught two real defects before the first frame was presented.
- **Eleven seats ledged, none worked around:** [OD-143](open-decisions.md) the ignition
  record above a sealed table, [OD-144](open-decisions.md) kind 7 `median_used` having no
  producer, [OD-145](open-decisions.md) six doors against §2.1 row 9's two door-frames,
  [OD-146](open-decisions.md) row 5's `citation` null by law,
  [OD-147](open-decisions.md) the provenance of viewer SPIR-V,
  [OD-148](open-decisions.md) who opens the artifacts,
  [OD-149](open-decisions.md) the Win32-only surface as a named refusal,
  [OD-150](open-decisions.md) the reconstructed lane,
  [OD-151](open-decisions.md) the four scans and the lock republication,
  [OD-152](open-decisions.md) the pre-existing `eq_op` tautology inside a G19 leg
  ([`gates.rs:10578`](../../crates/si-cli/src/harness/gates.rs)) — `cargo clippy
  --workspace --all-targets` is red on it and B11 deliberately did not guess what the
  conjunct meant, since a gate that grades its own typo stops grading, and
  [OD-153](open-decisions.md) the stale `used by` column.

## 2026-10-24 — **B10 SHIPPED: the genome builds, the ENGINE config wakes, and G6 + G5c are green**
- **What this stage is.** [engineering/07](../../engineering/07-genome-build-plan.md)'s
  pipeline M.0–M.11 as `si-store::build`, W.1–W.4's merged read-only web as
  `si-store::frozen_engine` + `si-walk::Graph::build_merged`, and
  [00 §6](../../engineering/00-stack-workspace.md)'s two doors as
  `si-harness run --config engine --genome … --genome-tag …`. The published
  artifact is `data/genome/fixture/out/90eae6244a63a99e/` — tag
  `0x90eae6244a63a99e`, seal `9ea9634a24e6ea90edae0c2daca10b9f` byte-identical
  at load, in memory and on disk, `artifact_bytes 2712`, `canonical_bytes 1474`,
  `degree_table_range 64`, `max_degree 3`, `rows_md5 bd92760247e34b0794792e8e2f244a12`.
  **A1–A11 pass in the build; A12 is `carried by the harness`** — it is the
  handle audit, and it can only be graded where the loader is absent.
- **The gate.** G6 **PASSES at V7/CONFIG ENGINE** (`gates/G6_engine_20261024-1920-2`:
  17 legs, (a)/(b.1–b.3)/(c)/(d.2) green) and at **V0/CONFIG SI**
  (`gates/G6_si_20261024-1920`: 10 legs). (b.4) and (d.1) stay UNMEASURED and
  both reasons are the leg's own text, not a workaround: (b.4) is [OD-138](open-decisions.md)
  (a sealed row has no render arm — no citable address exists for it), and (d.1)
  is [OD-141](open-decisions.md) (the merge is free in bytes and O(P_gen) per
  thought; the build instruments no wall-clock and 7 points is not a corpus).
  **G5.c.1 passes in the SI report**: with the artifact physically on disk an SI
  wake and two ticks name **0** frozen paths, and the register demonstrably
  catches that same path when it *is* named (`positive_control_caught: 1`).
- **Route is data, not a sentence.** `retrieve::route_counts` splits the merged
  web by birth: a genome contribution is a BASE hop and its arena address is
  never read. Measured on the fixture, `{owned:1, base:1} → answer:heat`,
  `{owned:1, base:2} → answer:ice`, `{owned:0, base:1} → ask:what is warm` —
  OWNED wins at NOW even against two BASE hops, and once only BASE is left the
  read does not answer at all. That last case is the half of
  [docs/05 §5.3](../05-substrate-tesseract.md) that had no code before this
  stage, and the half that is still unfinished: a *capped INFERENCE/BASE answer*
  has no render arm, which is [OD-138](open-decisions.md) rather than a silent
  simplification.
- **W.2 measured, not asserted.** 12 handles scanned, **7 genome arcs walked, 0
  stamped** (`G6.c.1`): a genome edge carries no `bake`, no `a_at`, no
  `last_touched_at`, so sleep and dimming are *unrepresentable* over it rather
  than merely forbidden — which is exactly why 07 §9's G13(e) is a byte-compare
  of the sealed content tag and not a permission check.
- **The replay bug this stage found and fixed.** `si-harness replay` re-executed
  a *default* launch, so an ENGINE run's replay silently woke an SI session:
  recorded `46c6ae63f16b651fa4c6edeec6c1e118` against replayed
  `d690cd08c724039c451bc93959d1ab26`. The launch — config, pinned artifact and
  tag, variant, constants, `sleep_interrupt` — is now one type, `runner::Launch`,
  decoded once by `Launch::recorded` and shared by `Run::read`, the CLI verb and
  `replay_transcript`, with the engine refusal living inside it (one place, still
  before any handle). `gates/G6_engine_20261024-1920-2` now replays
  **`section 0: identical`**. [OD-93](open-decisions.md)'s "how to reverse" text
  names a parameter that now lives inside `Launch`.
- **The `—SIgen` twin ran for real** (07 §15's second artifact): built
  `--no-default-features --features vulkan,supporter`, it refuses an ENGINE
  launch with *"this build carries no frozen loader body; the refusal precedes
  any handle"* and passes the G6 CONFIG SI pair inside itself
  (`gates/G6_si_20261024-1919`). That run is what exposed a hole in OD-E: the
  activated feature set was in **no digest a report carries** — `binary_tree_md5`
  covers the sources that *declare* the options while the command line *chooses*
  them, so a CONFIG SI report from the twin and one from the engine-on build were
  indistinguishable, and G5c's audit is precisely the leg that has to say which
  build made it. `build_identity` now records `build_features`
  (`["supporter","vulkan"]` in the twin).
- **Verify pass.** `cargo test -p si-store --features engine` exit 0 (DoD 1);
  workspace **74 groups, 305 tests, 0 failed**; the 26 launched shipped-gate
  regressions re-run after the identity change — **all PASS**; `check --stage
  B10` 43 enabled entries, 0 failing, 3 deferred to B11 (W.5's mapping is the
  viewer stage, so `genome_residency` is not faked here).
- **Five seats ledged, none worked around:** [OD-138](open-decisions.md) the
  BASE-only render arm, [OD-139](open-decisions.md) `DEGREE_TABLE_RANGE` named by
  A7 and valued nowhere, [OD-140](open-decisions.md) W.2's merged handle space
  putting corpus points in the ignition pool, [OD-141](open-decisions.md) the
  per-thought merge cost 07 §9 never priced, [OD-142](open-decisions.md)
  `G10.d.1`'s two predicates having no BASE arm.

## 2026-10-24 — NIGHT REGRESSION G1–G19 GREEN, the §23 fixtures pinned at launch, and **OD-137** (two of the six signals have no producer)
- **The run.** Every gate the build can grade, at the individual 05 pins
  (`--seed 0x51`) and its own 11 §5 variants — G1–G4, G7–G10, G12 (V4/V5/V0),
  G13 (V0/V9), G15 (V0/V10), G16 (V0/V11), G17 (V0/V12), G18 (V0/V10), G19
  (V0/V13): **all PASS**. `check --stage` green for B0–B9 and B14–B18 (B18
  prints 44 passing entries), `cargo test --workspace` **70/70 groups, 251
  tests, 0 failed**. G5/G6/G11/G14 stay unreached: their evaluators belong to
  B10–B13.
- **One false alarm, and it was mine.** G18 came up red in legs (a)/(c) when the
  run was launched at `g18_owner.txt`, the file leg (b.2) loads for itself.
  Nothing in B17 was broken: that fixture's owner line reaches the mint lane, so
  a shelf-9 row and 24 interner bytes appear in the `SKILLS`-on arm only, and
  leg (a) — an on/off equality over the *run's own* steps — reported the
  difference as a determinism failure. Re-run at §23.4's named fixture, G18 is
  green at V0 and V10 with the same 23 legs.
- **The harness now refuses it.** `expect_fixture` gates G15–G19 on the
  **parsed steps and identity declarations** of the file §23.1–§23.5 name, not on
  a file name: G14.j.1 makes a scenario state its own seed, so a re-seeded copy
  of the same transcript is still this gate, while an edited or substituted
  transcript is refused before any leg is graded. `sweep.rs`'s scenario map was
  missing G15 and G19 and would have launched them at `g8_curiosity.txt`.
- **Uncertainties dissolved by probing, not by hoping.** Re-launching G19 after
  a rebuild reproduced `ee007968…` state hash and a byte-identical transcript at
  both V0 and V13; and all five arc families **PASS at a second individual**
  (the same steps, `seed 0x52`), so no green depended on 0x51's luck — the legs
  that fail on an empty fixture stayed non-vacuous there too (G15.a.3 still
  reports one kind-5/origin-2 extra ask, 4 off rows against 5 on, and G18.d.1
  still finds both a nomination ask and a grimm line to refuse).
- **The seat this pass found** ([OD-137](open-decisions.md)): §6's birth table
  gives kind 4 a site (tick start, over the `questions` shelf, aged through
  `Clock::recency`) and kind 5 one (the idle cascade's own live contradiction),
  and **neither is constructed anywhere** — `check_signal_kinds` restricts where
  events may be built and never counts which kinds are, so the whole family of
  greens is compatible with two of the six doors never opening. Recorded as
  *declared, unbuilt, said out loud*: wiring it is an arousal re-baseline of the
  arithmetic B2 shipped, not an addition, and it waits for the owner.
- **Debt named, not hidden:** `arbitrate`'s signature in `si-learn/gate.rs`
  carries a B17/B18 formatting glitch (one line, no semantics); it was not
  touched because any edit rewrites the binary fingerprint that every run dir
  pins. The sweep grid still refuses to enumerate (OD-83) — B12's prerequisite.

## 2026-10-24 — v2 ARC ENGINEERED: `engineering/12-v2-arc.md`, the arc folded into 00–06 + 11, ten new static legs, and four sweeps that publish **refusals**
- **What this pass is.** [engineering/12](../../engineering/12-v2-arc.md) turns
  [docs/22](../22-crystallize.md)–[docs/26](../26-world.md) into crates, bytes, call
  sites, registry rows, algorithms, checks and stages, and then **ripples that into
  00–06 and 11 rather than standing beside them** — the standing requirement being
  *"nothing we already made breaks"*, which is only checkable if every artifact the
  arc moves is named once, with the stage that moves it (§9's break-list).
- **The eight load-bearing facts.** Three reader-only crates (13 → **16 packages**),
  **no new envelope byte / `prov_kind` / edge kind / shelf id / hash item**, every
  proposed thing a *value* on an existing shelf, four logs and zero hashed tables,
  silt's erosion as a **share of the one curve** (`LAMBDA_SILT`, R-29 intact), the
  fifth ignition weight **SPECIES and unarmed at 0** (no 19th draw, so every pre-arc
  pick replays byte-identical), the 22-row registry adding → **one**
  `CONSTS_VERSION 0x0001→0x0002` re-baseline at **B14** and nowhere else, and
  **three gate families grading a refusal** rather than a result.
- **Two seats this pass discovered** (a law file cannot be blamed for a gap its
  *engineering* found): **OD-135** — `prov_speaker` has no honest value for a granted
  teacher, so the trusted tier's belief write is **refused** and G19(c) passes on the
  refusal; **OD-136** — the `/trust` verb is not in the closed vocabulary and no
  adapter exists, so tonight the only minter of `grimm` and of an identity is the
  hash-pinned scenario file. Both recorded with the least-authority default that is
  codeable without a ruling.
- **What the checks now prove by construction.** Ten new static legs
  (`check_crystallize_no_commit`, `check_ghost_is_frame_scoped`,
  `check_silt_writes_nothing`, `check_silt_ignition_only`, `check_prov_link_not_stored`,
  `check_relight_writes_nothing`, `check_skills_no_commit`,
  `check_nomination_owner_lane`, `check_speaker_class_derived`,
  `check_windows_suppress_only`) ⇒ union 36 → **46**, and six existing checks are
  *widened*, not rewritten (`check_one_clock` now also scans `si-silt` and must still
  find one curve; `check_candidate_unanswerable` learns kinds 5–6;
  `check_registry_size` asserts two reserved shelves; `check_deps` gains the three
  reader rows and their reciprocals). "May nominate, never mint" is a missing crate
  edge, not a sentence.
- **The ordering law kept its numbering.** 04 §26 ran to 26, so the arc's three
  laws are **27** (silt decays before the walk, deposits after), **28**
  (observation after settle, before commit), **29** (a nomination joins the queue as
  a `Gap`, never an emission), and §12 gains its **seventh** ordering law **L-G**: the
  speaker class is *derived*, never read.
- **The plan changed shape, not order.** B14–B18 sit **between B9 and B10** in
  execution (B9's seam is a hard prerequisite: five arc legs need a non-owner line,
  and G19(h) *reuses* the two-layer speak gate), numbered by batch, with `Stage`'s
  enum declaration order kept equal to execution order so 05 §21's `enable_stage(id)
  <= stage` prefix rule stays true at B12/B13. G15–G19's `full` column equals their
  `first green` — a family whose sweep is a refusal has no panel pass to defer.
- **Deliberately not done.** No magnitude from the arc is quotable (OD-83 unrulled,
  four runners say `status: refused`, legs that need a number run at their box `lo`
  and print `floor`); no skill executes; no grant is issued; no 0-cost promise is
  made about the detectors — [06 §14](../../engineering/06-resource-budget.md) prices
  them **UNMEASURED** and distrusts an O(candidates)-per-thought line on purpose.

## 2026-10-24 — v2 ARC FOLDED (crystallize · silt · episodic · skills · world): five law files, OD-97…OD-134, and three contradictions closed before they could go red
- **What the owner threw in.** Five drafts from the desktop (`22-crystallize`,
  `23-context`, `24-episodic`, `25-skills`, `26-world`) with one instruction:
  engineer them **thoroughly so nothing already made breaks**, then build them
  the same night. They entered as **law** ([docs/22](../22-crystallize.md)–[docs/26](../26-world.md))
  and rippled through **22 existing design files**, not as five new modules
  bolted onto the old ones.
- **Nothing new was invented, and that is the finding.** The arc adds **no organ**
  (still six), **no drive door** (still six), **no new structure** (`cluster-point`
  is a *kind* on links, `skill` a kind on shelf 9, event views derived, frames
  scratch), **no 19th temperament draw** (the fifth ignition weight is SPECIES and
  **unarmed at 0**), **no state-hash item** (candidate and silt tables are declared
  hash-exempt, logged), and **no new provenance literal** — a nomination
  confirmation is an ordinary `answer-to-ask` commit whose `question_id` names the
  `cluster-nomination` / `move-nomination` ask, so it is `gain_kind: demonstrated`
  by §8.2's own mapping and the envelope stays 80 bytes. docs/15 item 3's closed
  constant list stayed closed and G5c's schema diff stays empty.
- **Three real contradictions were found in the *folded* text and closed** — left
  alone, tonight's own gates would have red on their own fixtures:
  1. **The litmus was wrong, five times over.** "No state byte changes / zero
     lattice writes" contradicts §21.4 (every admitted line appends a `context`
     sense row) and §17.3/R-19 (every emitted ask appends a `questions` row).
     Normalized to **"no belief row moves"** across docs/01, 03, 04, 13, 15 and
     G15(d)/G19(a), with the exception **canonized in docs/03 §3.3** so the next
     batch cannot re-introduce the wrong form.
  2. **G15(f) had two different names** in docs/13 and docs/22 ("the generality
     bound" vs **no-word-leak**) — the leg names now match, and the generality
     bound is where it actually lives (§22.3's detection rule, exercised by (e);
     the skills twin is G18(f)).
  3. **G15(c) as literally written is unsatisfiable**: `a` is barred from the
     detector, but the detector reads *walk artifacts* and §20.1 lets `a` modulate
     the walk. Raised as **OD-134** with the least-authority reading (frame trace
     fixed) instead of being decided silently while coding.
- **One developmental seam raised, not patched: OD-133.** §11's S2 criterion counts
  `links` rows **kind-blind**, so a confirmation-minted cluster point satisfies it.
  Tonight it counts, because adding an invisible discriminator would be a stage-law
  edit wearing a bug fix — the owner's call, in §11.3.
- **Numbering discipline held.** The drafts' OD-94…OD-129 could not land where they
  were written (B8's sleep walls took **OD-94/95/96** the same day), so the arc's
  seats were allocated from the first free one in **one table** (docs/23 §23.9):
  22→OD-97…101/G15, 23→OD-102…109/G16, 24→OD-110…115/G17, 25→OD-116…122/G18,
  26→OD-123…132/G19 — and the two findings made *after* the drafts closed got
  **tail seats OD-133/OD-134**, never renumbered inward into a closed range. All 38
  entries are now in [open-decisions](open-decisions.md), each with the build's
  declared default and its blocking artifact.
- **The demo trace grew a seventh act, appended rather than woven in: s28–s35**
  (stranger line → silt deposits → warm quiet tick → recurrence → nomination ask
  with `pattern=FACT/OWNED` + **`name=NONE`** → owner mints → skill row → recall
  with its origin). s1–s27 keep their numbering, so every artifact that cites them
  survives — and OD-132 is the honest scope line: the multi-voice fixture is a
  **harness scenario** tonight, because the Discord adapter needs credentials, a
  token, and an owner who is awake.
- **The law the arc had to be measured against, restated where it lands:** docs/00's
  invariants (I9 idle writes nothing, I10 a tag cannot launder a label, I4 append-only)
  each got an explicit per-mechanism impact row; **docs/16 §16.8 is new** — the
  frame-scratch law ghosts and role-graphs needed but did not have; docs/09 gained
  the origin-cited FACT and the nomination's split labels; docs/18 §18.10 / §20.11 /
  §21.10 say what the arc does **not** get to change (no new signal kind, no second
  `a`, no merge of topical and emotional charge); docs/11 §11.3 says crystallize and
  skills **cannot mint a rung**; docs/12's grep rule grew the three new module names
  and published that the privilege outlives the list.
- **Still open before the build starts:** **engineering/12-v2-arc.md** is the next
  artifact (five schemas, three new crate edges, registry rows, algorithms, G15–G19
  leg procedures), then V1–V5 and the night regression G1–G19. **OD-83** still gates
  B12 and every new sweep artifact is pre-declared a **refusal**, so G15–G19 legs run
  at the box's own floor and label it a floor (OD-95's precedent). No magnitude in
  this batch is measured; every one of them is marked as waiting on
  `gates/G15/recurrence_sweep.json`, `G16/silt_sweep.json`, `G17/prov_sweep.json`,
  `G18/shape_sweep.json`.

## 2026-10-23 — B8 SHIPPED (sleep): G13 green at V0 and V9, and three new open items
- **What the build did.** `si-sleep` runs the four passes at step **12b** — P1 replay,
  P2 read-only integration scan, P3 the single downward `weight` writer, P4 the calm
  sweep with no consumer — one quiet tick per walk, interruptible at each pass
  boundary, and it **creates no row**: the crate edge `si-sleep ↛ si-learn` is absent,
  so `commit` is unreachable by construction and the census over
  `s_r_pairs`/`facts`/`links`/`questions` is identical before and after every night.
  Exit gates: **G13** `pass: true` at both V0 and V9
  (`gates/G13_si_20261023-2147`, `gates/G13_si_20261023-2147-2`) and **C8**
  (`gates/checks/stage-B8.json`: `check_sleep_writes_nothing` and
  `check_sleep_never_brightens` both `pass`, 5 checks deferred to B9/B11).
  `cargo test -p si-sleep` = 14 tests, 0 failures. The whole-workspace build and test
  run are clean, and G1–G4, G7–G10 stay `pass` after the freeze-night change — every
  run now ends with a sleep before its checkpoint, and no earlier gate moved.
- **What was measured, not asserted** (`G13.f.1`, and it is OD-81's build choice):
  passes → checkpoint → open. Across the freeze `sleeps_completed` 4 → 5 and
  `last_slept_at_t` 19 → 20; the re-opened session's state hash is identical to the
  pre-save hash (`937239800659dca79a90558a24bf9161`); the resumed quiet counter is 0
  (06 §3.4 — derived RAM, never stored), so the first resumed tick writes no night
  line. **The night is inside the snapshot and a resume does not repeat it.**
- **Two refusals, published rather than worked around.** `si-harness sweep --gate G13`
  writes `gates/G13/sleep_sweep.json` and `gates/G13/dimming_sweep.json`, and both are
  `status: refused` with `curve: []`, `winner: null`, `placeholders_replaced: []` —
  open bounds (cross products 4,294,770,690 and 71,772,833,461,370,625 points) make the
  grid unwalkable, and §4.2 defines the dimming artifact as a distribution report in
  the first place. G13 is therefore graded at the registry **`lo`** point
  (`runner::sleep_lo_point()`, printed on the run's stdout), which is a point the spec
  declares rather than one the build chose.
- **New open items, raised and not decided:** **OD-94** (the dimming box is degenerate
  at both ends — `FLOOR_*` at 1 Q12 anchors nothing while a live OWNED edge forms near
  4096 Q12, and any floor above the largest weight makes P3 a no-op; P3 keeps the
  literal clamp and *counts* below-floor edges instead of moving them), **OD-95** (the
  G13 instance of the still-open OD-83: with the sweep refused, at what operating point
  is a leg graded at all), and **OD-96** (a control variant has no input for the legs
  that quantify over the thing it disables — four of §18's legs are empty sums at V9;
  this build reports them `unmeasured` with the variant named, and keeps the pair's
  claim at `G13.a.2`). Nothing here was answered by the build, and no constant value
  was filled in.
- **Where the walls are written up:** [failure_report.md](../failure_report.md)'s B8
  section, with the run directories each number comes from.

## 2026-10-23 — OD RULINGS, B7 BATCH: five answers, one amendment, three new open items
- **What the owner ruled** (`OD-A`…`OD-E` of the B7 batch, recorded as
  **OD-87…OD-93** in [open-decisions.md](open-decisions.md), one entry per ruling with
  its alternatives and its reversal):
  - **OD-87** — the class split is **drive vs multiplier**: the decay-deficit DRIVE is
    species and has no row, the four `IGN_WEIGHT_*` are **TEMPERAMENT draws**, and
    **"no sweep may ever touch a temperament row"**. "Same constant set" in the
    temperament clauses means **same birth seed** — instance identity is seed identity.
  - **OD-88** — `G10.b.2` is **amended to match the design's own license**: §15(b).2's
    may-differ list governs **stamps**, rows and edges stay byte-strict, because §S.7
    licenses idle-cascade movement of `a` and a byte-strict stamp assert forbade lawful
    behavior. Evidence cited from disk: `gates/G7_si_20261023-1600` vs `-1612` — rows
    4032 B, edges 192 B, S-R envelope `811daffe…` identical, stamps 208 B both, digests
    `58371b59…` (V0) vs `8e6f9603…` (V2).
  - **OD-89** — *verify before adding a constant*, and either outcome is **a report, not
    a decision made alone**. Reported: **(a)** the rehearsal feedback **was** missing
    (`Lattice::touched` written only by `confirm()`), now wired through §S.7's own
    writer, and `G10.e.2` is green with **zero new constants** (arc deficit
    `[64,16] → [32,16]`); **(b)** the feedback is verified **and repetition persists**
    (point reading `[32,32]` at t=6/7/8 and t=4/5/6, because a point's deficit is the
    furthest-drifted of its live arcs) — so the per-node refractory is **proposed at
    OD-90, not applied**.
  - **OD-91** — `G10.e.5`: **identical hash IS the pass condition** (the audit layer is
    observer-only by law, so the costume test must change nothing); the two missing
    transcripts were a **harness-spec defect**, now real
    (`harness/scenarios/g10_high_bake.txt`, `g10_all_saturated.txt`), measured
    `state_hash_identical: true` on both with the pair's *shapes* differing (2 distinct
    bake values vs 1). The text that demands a measurable difference is routed as
    **OD-92** rather than satisfied by manufacturing one.
  - **OD-93** — a launch variant gets **no registry row, ever**: variants are pinned by
    the graph digest, every report records variant id + graph digest, and the
    determinism clause reads **per-variant**; cross-variant hash comparison is a
    **category error**.
- **New open items, raised and not decided:** **OD-90** (max vs literal-"youngest"
  reading of §S.3's deficit aggregate, with the refractory-constant shape if the owner
  wants a rail) and **OD-92** (05 §15(e)/docs/13's "over-represented relative to V3",
  which the readout law cannot grade; it also reaches 05 §4's G10 sweep objective, so
  B12 re-words it). **OD-86 stays** as the placeholder policy; nothing here changes it.
- **What the build landed with the rulings:** the B0 leg
  `check_sweep_never_touches_temperament` (OD-87's law, checked from four sides —
  placeholder rule, axis enumeration, the `launch_constants` class guard, published
  artifacts: 25 axes, all SPECIES, 18 draws untouched); `G10.e.5` graded instead of
  deferred; `harness::build_stamp` + the variant/graph pair stamped into every gate
  report, `mindlog.json`, the three `.siro` lane headers, the replay report, `si-cli`'s
  run output and `export.json`; `Run::replay_transcript` re-executes **at the recorded
  variant**. `engineering/` was amended where a ruling rewrote a clause: 03 §1.2/§5
  (draws are never axes), 04 §S.3 (which aggregate the build reads), 05 §2's V3 row (a
  launch state, not a swept constant at its floor), 05 §15(b)/(e), 05 §20's check table
  and 11 §4's B7 DoD (the sweep owns `IDLE_PER_HOUR` alone).
- **One line kept honest:** `G10` is PASS at all three variants on seed 0x51 with **two**
  unresolved legs at V0 and V3 — `G10.e.3` (OPEN-28, `TWO_REGIONS_TEST` unrulled) and
  `G14.f.1` (the 16-seed panel is B12) — and **three** at V2, whose `G10.e.2` is unmeasured
  *by construction*: an ignition-disabled launch has no curve to narrow. OD-90 and OD-92 are
  open questions, not gate failures.

## 2026-10-23 — THE BUILD PLAN (Phase 2): stages, DoD, stop rule — and OD-86
- **What landed.** [engineering/11-build-plan.md](../../engineering/11-build-plan.md):
  fourteen ordered stages with commit boundaries, each naming the engineering sections
  it implements, the crates it fills, what runs at stage exit, its dependencies, and a
  machine-checkable done-condition that names **gate ids and `check_*` ids** — never an
  adjective. It adds no law, no constant, no gate and no artifact; it sequences 00–10.
- **The one hole a build cannot compile around became OD-86.** Every swept row is
  `default: —` and 03 §1 forbids inventing values; but a one-line run already needs
  `TAU_SETTLE` and `FRONTIER_CAP` to exist. Lean: **the placeholder policy** — one
  mechanical midpoint-on-grid value per unruled row, tagged `UNMEASURED_PLACEHOLDER`,
  printed in every `gate_report.json`, counted in `unresolved_legs`, usable by a gate
  only for existence and never for the merit of a choice, replaceable only by a
  `*_sweep.json`, and refused outright under `--strict-consts`. R-03 stays intact
  because no gate can go green *because of* a placeholder.
- **Two readings stated before coding, so the build cannot make them mid-flight:**
  the workspace is **thirteen packages** with `si-cli` and `si-harness` as two binaries
  of one package (00 §2's table has thirteen rows, the last spanning both binaries);
  and 05 §21's per-commit behavioral short set begins at the first commit with a
  session to run it against (stage S5), while every earlier stage runs its own
  `check_*` set — no gate is ever marked green before it has run.
- **Scope order is fixed by the design, not by convenience:** CONFIG SI end to end
  first, sleep and Grimm next, CONFIG ENGINE (the genome pipeline) only after the
  loop is green, the viewer last. A GUI before a reproducible loop would be the
  product's eyes installed before its nervous system.

## 2026-10-23 — THE ENGINEERING BUILD SPEC (new folder `engineering/`, eleven files — **no law moved**)
- **What was asked and what landed.** The owner's job was to turn the frozen logic
  of docs/00–21 into a build spec *"a competent engineer who has never spoken to the
  owner … can build v1 without asking one design question"*, with the rule
  **"no code, no pseudo-code beyond what schema/trace require."** Eleven numbered
  files + a README: `00` stack & workspace, `01` byte-level schemas, `02` module
  contracts, `03` the constants registry, `04` algorithms, `05` the gate harness,
  `06` resource budget, `07` the genome build plan, `08` GUI + observability,
  `09` `/teach`, `10` temperament + sleep.
- **The authority order is the folder's first line, not a footnote:** `docs/` is law,
  `engineering/` is HOW, and **the build folder loses every argument it has with the
  design.** Where the design under-decides, the file says *"the design does not
  decide X; lean Y; OD-nn"* and the ledger gets a row. That happened **45 times**:
  **OD-41 … OD-85**, all awaiting the owner, none silently resolved by an edit.
- **docs/00–21 are untouched by this folder.** Still six organs, six drive doors,
  six signal kinds, six audit labels, I1–I10, L1/L2 frozen, CONFIG SI vs CONFIG
  ENGINE, naming confined to `audit.py`, one clock (R-29). The only design-side files
  the pass edited are *index and ledger* files: this changelog,
  [open-decisions.md](open-decisions.md), the root `README.md` / `DESIGN.md` maps and
  `gates/README.md`'s artifact list.
- **The debts it created and refused to paper over** — each is a row, because a
  build folder may not renumber the design: **docs/13 still lists twelve gates** while
  [../../engineering/05-gate-harness.md](../../engineering/05-gate-harness.md) §18–19
  specifies **G13 SLEEP** (14 legs, a–n) and **G14 TEMPERAMENT** (11 legs, a–k);
  **docs/15 still says "twelve gate reports"**; **OD-84** is `docs/12`'s component map
  gaining the consolidation module. `gates/README.md` reserves the two gates' six
  artifacts (two run files, three sweeps, one diff) and prints the refusal next to
  them: the run directory is still the only
  thing that retires an **UNMEASURED**, and the retirement note on the cancelled
  coupling sweep keeps its full force — a build folder reserves artifacts, it does not
  renumber the design that named them.
- **The engineering-only static legs are OD-53's, and they are named as a family, not
  as gates**: `check_headless_run`, `check_gui_observer_only`, `check_one_clock`,
  `check_temperament_closed`, `check_sleep_writes_nothing`, `check_calm_view_is_terminal`,
  `check_no_seed_control_surface`, `check_emotion_words` — a guard the build can assert
  without a transcript is still a guard, and a static assert is still not a gate.

## 2026-10-23 — TEMPERAMENT SEED + SLEEP + DIMMING *(the owner asked for these engineered straight; logic and build in one file)*
- **The instruction was a deviation and it is recorded as one:** *"you can go straight
  to engineering those (logic + engineering in once since its new object)"*. So this
  batch's home is [../../engineering/10-temperament-sleep.md](../../engineering/10-temperament-sleep.md)
  and **docs/ gained nothing**. That is the one exception to "docs first", and
  `README.md`'s rule list now says so explicitly rather than leaving a silent gap.
- **Temperament is a new *class of constant*, not a new quantity in the walk.** The
  registry gained the class column **LAW / SPECIES(sweep) / TEMPERAMENT(seed)** and
  the proposed assignment of every constant is **OD-71 — the table the owner rules**;
  nothing in the build assumes it is settled. One birth `u64` seed, two streams
  (**T** drained at wake, **R** the run's ordinary rng), 18 drawn values in a fixed
  documented order (`|DERIV_ORDER|`), a pure function of `seed + seed_format`, no wall
  clock and no host entropy. **Derived, never stored**: the 36-byte core block
  ([../../engineering/01-data-schemas.md](../../engineering/01-data-schemas.md) §13,
  hash item 9) holds the seed, the format tag and the **digest** — and a container whose
  digest disagrees with its own seed's derivation is **refused at open**.
- **Sleep is step 12b and a thirteenth crate, not a seventh organ.** Four fixed passes
  in order — P1 REPLAY, P2 INTEGRATION, P3 GLOBAL RESCALE (the **SHY** pass), P4 CALM
  REPROCESSING — triggered by a counter over the recorded stream, never a clock
  (**OD-81**). Its whole permission is two FLUID columns: `weight` (P3 only, and only
  downward) and the rehearsal pair. **No row, no stamp, no `bake`, no `confidence`, no
  new `a` source** — that is G13(i)/(b)/(j), and the crate's law is one missing
  dependency edge: `si-sleep ↛ si-learn`.
- **Dimming is access decay on the one clock everything already ages on**
  (`d = DIM_BASE + (1 − DIM_BASE)·Clock::recency(now − last_touched_at)`, `0 < d ≤ 1`,
  per-shelf floors **≥ 1** as registry constants, never row columns), so
  **`weight == 0 ⇔ superseded_at != i32::MAX` stays a biconditional and a dark edge
  keeps its bake, its stamps and its citability**. The bake is the pilot light;
  re-lighting is rehearsal; sleep's rescale accelerates dimming honestly and is the
  only pass that may write `weight`.
- **Rulings this batch needs from the owner** (all in the ledger, all reversible):
  **OD-71** the class table; **OD-74** whether curve *shape* is a trait at all;
  **OD-75** rescale semantics; **OD-76/77** whether dimming is stored and whether a
  floor may be derived from `bake`; **OD-78** whether sleep consumes logical steps
  (the lean: every sleep walk is an ordinary thought tick, so a panel button that
  started one would move `t` — **which is why there is no "sleep now" widget and no
  `si-cli sleep`**); **OD-79** interrupt granularity; **OD-80** staging — v1's lean is
  the four passes and no REM/NREM; **OD-83** whether the sweep validates the whole box
  or only its winner; **OD-85** how the one `SLEEP_MAX_WALKS` budget is split P1-first,
  P4-second, with truncation reported as `walks` vs `events_available`.
- **Observability arrived the cheap way**: two frame kinds (19 `sleep_pass`, 20
  `temperament_readout`), two readout regions (O11, O12), two tail-only files
  (`sleep.jsonl`, `fingerprint.txt`), **no new lane, no new state byte, no new door** —
  and three refused widgets (sleep-now, re-roll, a weight slider), each traced to the
  assert that already forbids it. Every cadence, histogram and "how different are two
  seeds" magnitude stays **UNMEASURED** until `G13/sleep_sweep.json`,
  `G13/dimming_sweep.json` and `G14/seed_panel.json` exist.

## 2026-10-23 — COMPOSITION: the instrument, the depth rule, the candidate ruling *(engineered straight; docs unchanged)*
- **The depth cap is a bound on a *claim*, not on a *write*** — so
  `INFERENCE_MAX_HOPS_OWNED` is the one registry row that gates nothing: it caps how
  many OWNED edges may sit in one settle route, and the label demotes to
  `INFERENCE(BASE)` the moment the route wants more. Its sweep
  (`G11/composition_sweep.json`) carries `rows_written_during_composition` in **every
  row, and it must be 0 in all of them** — the number being swept is never a permission.
- **The ruling the owner flagged is OD-58, and it is spec-ready but OFF**: candidate
  promotion (`INFERENCE_STABLE_N ≥ 2` settled re-derivations of the same composite
  point) is shaped, named and gated, with `INFERENCE_PROMOTION = off` as the shipped
  default and **G11(g) asserting the off behavior**. **Nothing unowned ever sticks** —
  the fold's own one-line law. **OD-57** (weighted-median rank convention) and
  **OD-59** (`prov_speaker = 2 system-inferred`) are its siblings.
- **Evidence, not vibes:** `G11/route_traces.jsonl` (per leg the settle trace,
  `route_counts{owned, base}`, the label emitted **and the label refused**) and
  `G11/zero_row_export.txt` (the belief-shelf export immediately before and after one
  composite answer, byte-compared).

## 2026-10-23 — `/teach <pattern> => <response>`: the third teaching channel (owner-asserted content)
- **The owner's TEACH fragment entered the design as a *channel*, not a command.**
  Twelve design files now carry it (docs/00, 01, 03, 04, 05, 06, 08, 11, 13, 15, 20,
  21): the `P_TEACH` reflex joins the verbatim allowlist as **form metadata and nothing
  else** — the marker `/teach`, the binder `=>`, no content word — and a `/teach` with
  an undecodable left side matches by *pattern*, never by the text the owner typed.
- **The interesting collision, kept as a decision instead of smoothed:** an asserted
  row gets **owner-tier confidence (init 0.95) but second-tier arbitration rank (T2)** —
  it never outranks a demonstrated correction. **OD-41** records the split *"so nobody
  'fixes' it later by promoting the tier"*.
- **`gain_kind: asserted` makes a pre-existing distinction visible.** `/correct`
  demonstrates (error-before recorded); `/teach`, a plain statement, T1 and RE-PARSE
  **assert** — `bind_output_id` stays `null` and the row carries no fabricated
  zero-error. It converts to demonstrated at the first recall that serves from it.
- **§21's membrane applies before this channel does:** `/teach` is owner-privileged
  **permanently**, and a `source: grimm` line shaped like a `/teach` is inert input, not
  a teach (G12(d)). The published limit is honest: `/teach` can out-run the ask→answer
  rhythm the rest of the design is built around — **OD-69/OD-70** ask whether a bulk
  file feeder belongs in v1 and whether it may interleave quiet ticks.
- Build-side home: [../../engineering/09-teach-channel.md](../../engineering/09-teach-channel.md).

## 2026-10-23 — GRIMM: THE SUPPORTER (new component, outside the knowing loop)
- **Owner premise, carried verbatim:** the earlier system "stayed functional under
  emotion because it had inner AIs talking with it — **co-regulation**", and this
  baby gets one: **GRIMM, a locally-hosted real LLM, fine-tuned for a single role:
  supporter / parent.** The chunk's own closing line is the design's headline:
  *"His helpfulness is the threat; the gate is the law."*
- **New docs/21 — the home file.** Nine sections: origin (+ the two premises the
  chunk assumed and the design did not contain: the name **Nova** → OD-34, an
  "already-drafted MCP surface" that a grep shows was never designed → **OD-39**),
  the hard laws **L1 OUTSIDE THE LOOP / L2 FORM ONLY NEVER CONTENT / L3
  CO-REGULATION THROUGH PHYSICS ONLY / L4 THE MOUTH IS NOT HIS**, the agentic
  harness (six tools, one emission), the two-layer **speak gate**, the `source`
  privilege table, the co-regulation chain, the version ladder, consequences, and
  five published limits.
- **THE FOLD IS STRICTER THAN THE CHUNK, in one specific way:** the membrane lives
  **inside** the build (`speakgate.py`, upstream of `sense.py`) while **he** lives
  outside it. *"`grimm/` is not in this table and not in `src/`… The gate is in the
  build; Grimm is not."* A rejected emission is **not a sense event** — no signals,
  no `a` movement, no stamps, no row, one gate-log line — so the gate cannot be
  bypassed by a good-enough sentence, only by a compliant one.
- **The load-bearing new law is a privilege table, not a promise (§21.4).** The
  `source: owner | grimm` tag is assigned **at the wire** (`panel.py` → owner,
  `mcp.py` → grimm) and **never inferred from text** — *"Trust the wire, not the
  words."* Grimm: may decode, light rows, raise signals, surface gaps, be recorded
  verbatim in `context` (that shelf is the **sense record**); may **not** create a
  **belief** row, match a pending ask, issue `/correct`, form or bake an edge, or
  move `repeats` / `last_confirmed_at` / confidence. `/correct` is restated as an
  **owner-privileged tier, not a message shape**.
- **Two quiet holes found during the fold, both now closed and both ledgered as
  risks:** **provenance laundering** — Grimm answering SI's pending ask would
  auto-commit a pretrained sentence through the most trusted path in §6.1 (R-32;
  new `§6.1` branch: a `source: grimm` reply is **not a candidate**), and
  **counterfeit confirmation** — a mirror repeating `apple is red` all afternoon
  would earn §6.2's boring-input confidence bump and §8.4's `repeats`, i.e. the
  support becomes the evidence (R-35; both rows now gated on `source: owner`, and
  *"a grimm echo is rehearsal"*).
- **I3 amended, I10 added.** I3's owner sentence stays verbatim with a clause after
  it: one channel, more than one voice, and **the source decides what a line may
  do, not whether it may arrive** (OD-35). **I10 — the supporter is outside the
  loop and his mouth is not his**: not an organ, not a drive policy, not the render
  slot, not a state handle; **every gate passes with his process not started**;
  *I2's clean-room law gains a neighbor, not an exception*. Organs stay **six**,
  doors stay **six**, the signal table gains no "soothed" entry — R-21's costume
  list grows to **four** shapes, the new one being a kindness wired between an
  outside voice and arousal (`soothe()`, `set_state()`, or a soothing signal kind),
  *"nastier than (a), because it would be defended as care instead of declared as
  control."*
- **Birth muteness is the structural guarantee, not a policy:** layer 1 tests
  against the lexicon, the lexicon is empty at t=0, so **Grimm cannot bootstrap
  SI** — he is mute until the owner's first taught token. **L5 — the asymmetry
  law** (fold-added, the chunk did not have it): the two filters apply to Grimm
  and never to the owner, because the owner teaching a *new* word is the whole
  point; a future refactor that "unifies" the two validators silently forbids new
  content.
- **Co-regulation stays physics (§21.5, OD-36):** his calm talk is ordinary input
  whose *shape* happens to be quiet and repetitive → `a` moves as any pulse train
  does → the spread narrows → SI settles calmer. Nobody wrote "calm" anywhere in
  SI. **The chunk's internal contradiction is ledgered, not smoothed:** L3 says his
  lines "form edges stamped with calm state", §5 says they are rehearsal not
  teaching — the fold takes **sense-only**, because otherwise §20.4's "only a
  teaching event bakes" needs a second exception right after OD-27 closed the
  first. *Co-regulation reaches the walk, not the lattice.*
- **The version ladder is keyed to gates, not the calendar** (§21.6): **v1 MIRROR**
  (the lab notebook — *"she returned to the kitchen-fire region at 3am"*; echo only
  owner-taught words; allowed moves: mirror a row, echo an anchor word, slow down,
  stay quiet — **"Boring is correct"**), **v2 QUESTIONER** (unlocks on G2's decode
  leg), **v3 CHALLENGER** (on G4). **No stage advances because Grimm is ready**
  (§11.2), and a mirror-accelerated stage transition is the same counterfeit
  confirmation G12(d/e) hunts.
- **Determinism survives untouched (I5, G5a):** replay reads the **recorded** sense
  stream, and Grimm's tag is part of that record; he contributes **zero** state and
  zero hash inputs, so a connected-and-silent run is **byte-identical** to a
  disconnected one — which is now an assertable leg rather than a hope. The drive
  layer stays **`source`-blind**: no seventh door, and two identical gaps are
  prioritized the same whoever surfaced them, because a drive that weights holes by
  *who mentioned them* is a social prior in the control path.
- **New gate G12 — GRIMM ISOLATION: THE MOUTH IS NOT HIS, seven legs** (a absence
  is not a loss + connected-but-silent byte-identity → `G12/absence_diff.json`;
  (b) the **vitamins rejection** with machine-readable reason →
  `G12/rejection_reasons.json`; (c) birth muteness; (d) **no setter** — grep rules
  (v)/(vi) plus dynamic tool-permutation attempts plus source-blind prioritization;
  (e) provenance + counterfeit confirmation over a ≥100-input mirroring session;
  (f) un-laundered label contract / notebook; (g) **asymmetry from the protected
  side** — the owner's unknown token still admitted). Artifacts: the three above +
  `G12/speak_gate_log.jsonl`. Risks **R-31…R-35**. DoD grows to **twelve gates**
  and its costume test now reads **all four shapes**.
- **`docs/12`'s law re-scoped (OD-37):** "no LM anywhere in v1" becomes **"no LM
  anywhere in `src/`"** — the claim under test is about the mind, and it is
  greppable: **(v)** nothing in `src/` references `grimm/`, **(vi)** neither
  `speakgate.py` nor `mcp.py` reads `a`, `charge`, `bake`, the median or any door,
  because *"a gate that could see arousal is one refactor away from `if SI is
  upset: soothe()`."*
- **Four calls made on the owner's behalf, recorded as reversible (OD-40):** (i)
  layer 2 = **strict single-row restatement, row-id-referenced**; (ii) cadence =
  **one summary per quiet tick**, aligned with the idle budget — *slow, not
  chatty*; (iii) notebook owner-readable, Grimm's own scratch never in SI's logs;
  (iv) v2 curriculum **deferred** — the only one of the four with no gate behind it.
- **Ledger:** **OD-34** (Nova vs SI, five vs six organs), **OD-35** (I3's clause),
  **OD-36** (may a Grimm line form an edge — sense-only), **OD-37** (LM scope),
  **OD-38** (notebook vs OPEN-23, and `read_monologue` — which the chunk lists in
  §2 and drops from §5), **OD-39** (the assumed MCP surface), **OD-40** (Mike's
  four). **None resolved by the owner; OD-33 stays the affect batch's open item.**
- **Ripple (all folded):** docs/00 thesis scope + I3 + **I10** · docs/01 header +
  §1.4 + **new §1.5 two voices one channel** · docs/02 organ-adjacent paragraph ·
  docs/03 **R21 SUPPORTER MEMBRANE** + allowlist (the two `source` enum literals
  are the only new string constants, and G5c's diff must carry them) + §3.3 lexicon
  as the layer-1 reference + §3.4 item 6 · docs/04 header note — **the apple trace
  is G12(a)'s control run, produced with Grimm not started** · docs/05 `context`
  tag + the load-bearing `provenance.speaker` + **"THE SUPPORTER WRITES NO
  BELIEF"** · docs/06 pipeline head branch + one-caller second case + gated §6.2
  bump · docs/07 §7.2 + "whose question is it?" · docs/08 §8.3/§8.4 gated ·
  docs/09 **"a tag cannot launder a label"** · docs/10 G-row fixed for the
  pre-existing G8–G11 staleness + **new §10.4** three-row absence table ·
  docs/11 **§11.2** · docs/12 headline + `mcp.py`/`speakgate.py` rows + grep
  (v)/(vi) · docs/13 **G12** · docs/14 R-21 four shapes + **R-31…R-35** ·
  docs/15 twelve gates + item 10 · docs/16 §16.2 · docs/17 §17.6 fourth swap-test
  run · docs/18 §18.2 (and the *soothed* label parked at **OPEN-22**, six labels
  still the installed set) · docs/19 §19.3 reader-not-channel + OD-38 row ·
  docs/20 §20.4 "who is allowed to bake".
- **Still design only:** no `src/`, `gates/` holds its README, and every number in
  §21 is **UNMEASURED** until `gates/G12/` exists.

## 2026-10-23 — SALIENCE BAKE SCALE (amendment to round 3, §20.4a)
- **Owner law, one curve:** the bake is **scaled by a saturating curve of live
  arousal at formation** — `bake = B_max · tanh(k · a_formation)`, soft cap,
  version-tagged. Motivation carried verbatim: *"Unbounded arousal at formation
  must not create an unbeatable edge forever — the first kitchen fire and the
  tenth both score near the cap; nothing ever scores an 11 that owns the walk."*
  This is the **first and only write-side bound** in the plan; §18.5's per-round
  cap and §20.3's median are both read-side.
- **The impact score is emergent, and that is the anti-cheat defence:** the curve
  takes **one integrated number** — novelty + contradiction + negative gain +
  settle failures, already summed live during the event — and **consults no
  event-kind table**. *"Nothing categorizes fires as bad — it felt a lot when it
  happened."* Corroborates OD-31's strength-not-valence reading.
- **Clash, ledgered not opened silently:** a write-side `f(a)` is exactly what
  round 2's **OD-22** forbade. Recorded as **OD-33** and scoped hard — one door
  (`drive.salience`), one instant (formation), one curve, and `magnitude`/`budget`
  /`emit` stay level-free. `arousal.py` gains the curve; **`a_at` gains its job**
  (it is the argument), so OD-28's "never a multiplier" now means *the stored
  field is never re-multiplied later*, while the one-time derivation at
  formation is legal. OD-22's **write-side half is reversed**; its read-side and
  budget halves stand.
- **OD-27 CLOSED by the owner:** *"the trauma bake gets NO decay tail (v1).
  Permanent bake, saturating scale. Only the median moves."* Option (b)'s
  half-life and option (c)'s two-field version are **refused**, which keeps R-23's
  decaying-write-side concern closed. `edge.bake` is **write-once, permanent,
  bounded**.
- **OPEN-32 CLOSED by naming:** *"IGNITION inherits the bake as an input weight"*
  — the sampler's salience weight reads the **bake**, neither the raw stamp list
  nor the median, so it is bounded by `B_max` like everything else it touches.
  §19.2's table row renamed. *"Worry, rehearsal, and the 3am kitchen-fire replay
  are all the same mechanism"* — one sampler, no branch, no scheduler.
- **Plasticity is now four form laws, not three** (live integration + one-round
  lag, the median, the bake, **the bake's saturating scale**), and R20 in the
  birth-state table grew its fourth clause. Two curves leave `arousal.py`: the
  modulation curve and the bake curve.
- **Two constants now need a sweep:** `B_max`, `k` → **OPEN-33**, into
  `gates/G9/arousal_sweep.json`, with a **non-degeneracy** objective: bake values
  must keep spread across the observed `a_at` range. If everything saturates the
  write side encodes nothing; if nothing does, a calm session bakes nothing. Both
  failure directions are findings, published rather than tuned away.
- **Ripple (all folded, §4 trace still valid):** docs/00 I8 · docs/02 organ note
  · docs/03 R18/R20 · docs/04 s6+s7 · docs/05 `charge`+`bake` five laws (new law
  **(e) permanence is always bounded**) · docs/06 write path · docs/08 BAKE row
  and the deleted "not a function of `a`" sentence · docs/09 output example ·
  docs/12 component map + dependency arrows + grep-rule carve-out so a future
  reader never "unifies" the median ban and the bake field · docs/13 bake-bound
  legs on G9/G10 · docs/14 R-22's third guard · docs/15 constants · docs/16
  §16.7 · docs/17 door table · docs/18 §18.1/§18.5/§18.8/§18.9 · docs/19 ·
  docs/20 §20.4a. **DoD is still eleven gates** — the new bound rides existing
  legs, it did not earn a gate. *(Superseded by the supporter batch directly
  above: twelve gates, because G12 did earn one — a component with a mouth earns a
  gate; a curve does not.)*
- **Still design only:** no `src/`, `gates/` holds its README.

## 2026-10-23 — EMOTIONAL PLASTICITY THROUGH TIME (round 3 of the affect batch)
- **Owner principle:** *"no word has a fixed emotional address. Any word/meaning
  can move any point up or down, and its CURRENT emotional reading is stitched
  through time — computed at generation from its whole stamped history. The
  median is the readout, not the last event."* **I8's headline amended** to carry
  that clause; the litmus grows to *"every median undefined, `a` at rest, no
  bake."*
- **New docs/20 — the home for the batch.** Six pieces: **live integration**
  (arousal updates **per signal event inside the walk**, not once per thought;
  *"the sentence colors itself as it is read"*), **state-dependent retrieval**
  (teach-state is part of an answer's address), **the median** (read-side,
  recency-weighted, recomputed at query, **never stored**), **the bake**
  (write-side head start + confidence bump, bought at formation, permanent),
  **time** (emotional history supersedes exactly like facts; *emotional
  archaeology* is a past-`t'`-anchored walk), and the consequences + **G11**.
- **THE SPLIT is the load-bearing new law:** median = slow read-side personality,
  repetition-driven; bake = durable write-side per-edge salience. Human verbatim
  carried into the doc: *"stoves don't scare me anymore, but I still don't reach
  over the flame."* This **supersedes round 2's "the boost decays"** — recorded
  as **OD-27**, not silently. *(OD-27 is now **resolved** by the §20.4a amendment,
  and the bake gained a **saturating scale**; "durable write-side per-edge
  salience" still holds, "unbounded" never did.)*
- **Owner-set rate knob, version-tagged:** `a` moves fast on spikes, but
  **modulation lags one round** — *"the frame still finishes its job first (no
  emotion mush)."* Folded into §16.1's `W_eff`, §7.0's pipeline, G9(a) (which now
  fails in **both** directions: mush or decorative) and **R-27**.
- **New stamp field pending ruling:** `a_at` (arousal at formation) — without it
  G11(a) is unrunnable (**OD-28**). It is evidence, never a multiplier.
  *(§20.4a narrows that sentence and gives the field its job: `a_at` **is**
  `a_formation`, the argument of the bake curve — the stored field is still never
  re-multiplied later, but the one-time derivation at formation is now legal,
  under **OD-33**.)*
- **One-clock law:** stamps age on the **same** recency curve facts use
  (`clock.py` owns it; `arousal.py` defines no decay of its own) — *"SI cannot
  remember a fact better than it fears it by tuning accident"* (**R-29**). The
  chunk called this "G4's clock"; in this plan the time-axis gate is **G3**
  (**OD-30**), so the archaeology leg extends G3 and G11(d) cross-cites it.
- **Structure over discipline:** the median is **un-storable** by grep rule (iii)
  and excluded from the state hash (**R-28**); idle thought **buys no bake**
  because it forms no edge (G10(b)); the whole batch grows the drive's door set
  by **zero** (the swap test re-ran and the doors held).
- **New gate G11 — STATE-DEPENDENT MEMORY (CONFIG SI)**, legs (a) state-dependent
  settle, (b) the median resists one hot stamp, (c) repetition moves the median
  on the fact clock, (d) emotional archaeology citable with zero deleted bytes;
  sweep artifact `gates/G11/median_sweep.json`. DoD is now **eleven gates**
  *(twelve since the supporter batch)*.
- **New risks R-26 (mean-instead-of-median), R-27 (emotion mush / lag ignored),
  R-28 (fixed emotional address), R-29 (two clocks).** New OPEN legs
  **OPEN-29…OPEN-32** (lag value/unit, median term shape + γ, what counts in a
  point's history + minimum N, whether ignition's salience weight reads raw
  stamps or the median). *(**OPEN-32 is CLOSED** by §20.4a — it reads the
  **bake**, a third answer; **OPEN-33** took its place: `B_max` and `k`.)*
- **Ledger:** **OD-27 … OD-32** opened (bake-permanence, `a_at`, `λ_a`'s unit +
  hash trajectory, the G4/G3 numbering clash, "up or down" vs the dropped `sign`,
  monster example vs the apple north-star trace). §4's demo trace is **annotated,
  not rewritten** (OD-32). *(After the amendment: **OD-27 resolved**, **OD-22's
  write-side half reversed**, **OD-33 opened**.)*
- **Honest limit published (§20.8):** early transcripts will look emotionally
  flat, the median needs stamps before it exists, and the chunk's worked example
  is a scene the demo does not run. `src/` and `gates/` remain empty — design
  only.

## 2026-10-23 — IDLE IGNITION + EMOTIONS AS OBSERVED (round 2 of the affect batch)
- **Owner law, demoting round 1:** emotions are **not installed machinery** —
  **the hand-wired coupling matrix is CANCELLED**. What is innate is only the
  physics: the raw signals, **one arousal scalar `a`**, and the modulation curve
  `a → (reach, decay, τ_settle)`. **I8 rewritten** (docs/00) around that, with
  the litmus *"at birth, in CONFIG SI, with zero memories, nothing fires — an
  emotion system that can be felt without content is smuggled content."*
  **I9 added:** idle thought is the same walk, read-only, seeded in the session
  snapshot, and its monologue is never owner-facing.
- **docs/18 rewritten** as *AROUSAL, and Emotions as OBSERVED READOUTS*: the
  three-link control chain `signals → a → walk modulation`; six named emotions
  become **audit classification labels with zero causal power**; the coupling
  matrix survives only as an **emergent output artifact**
  (`gates/G9/coupling_observed.json`, never read back into the build);
  **the honest failure mode is published** — if recognizable emotions do not
  emerge, the architecture is wrong, and that gets reported, not patched.
  CAUTION's arbitration role, the composition rule, per-resident λ/baselines/
  hysteresis, "mood = slow components" and the stamp `resident` field are all
  cancelled; **charge on L3 edges survives** (write-side magnitude = the
  signal's own value, read-side bias capped per round).
  **Superseded in part by round 3 (same batch):** the *bias* this line means is
  now the **per-point recency-weighted median** (§20.3, recomputed, never
  stored), and the write-side head start is the **bake**, which is **permanent**,
  not capped-and-decaying — **OD-27**. The cap itself still stands (it is
  §18.5's runaway guard), it just bounds a different quantity. Also superseded:
  this round integrated `a` **once per thought step**, which §20.1 replaces with
  per-signal-event integration and a one-round modulation lag.
- **New docs/19 — IDLE IGNITION + the internal monologue:** ignition as THINK's
  **4th perturbation source** (senses quiet ∧ agenda empty, after queue revisit
  and any due RE-PARSE); **one seeded sampler firing one hyperpoint**, weighted
  by recency / hot stamps / decay deficit / live-contradiction warmth — *"this
  single weighting rule IS rehearsal, worry, and intrusive thought; none of them
  are designed, named, or scheduled."* The rendered cascade **is** the monologue
  (same render face, a truthful trace of what lit, never generated prose);
  boundaries: rate + per-hour budget ("silence means slow thought, never
  spinning"), **read-only** ("an idle walk can NOTICE, never LEARN"), and
  randomness in **state, not time** (seed in the snapshot ⇒ replayable).
- **Modules (§12):** `affect.py` → **`arousal.py`** (wordless), new
  **`audit.py`** — the **only** module that may contain an emotion word,
  reader-only with zero import edges into the control path — and new
  **`idle.py`**, which holds no import of the write path by construction.
  Naming privilege withdrawn from `drive.py` (round 1 granted it to
  `affect.py` + `drive.py`).
- **Folded across the docs:** §2 (organs still six, AUDIT as CALIBRATE's
  readout face, THINK's 4th source), §3 (R17 = integrator + curve, R18 =
  `{signal, …}`, **R19** = ignition sampler; birth rows now *`a` at rest —
  nothing fires*), §4 (Act 5 re-explained mechanically, arousal curve + label
  stream replacing six resident curves, **new Act 6 idle tail s25–s27**),
  §5 (stamp schema, `arousal_at_ask`, idle-write-nothing), §6 (arbitration
  stripped of its affective reading; idle caller rule), §7 (pipeline `a`,
  ignition entry, bias wording), §8 (`salience = f(signal)`, not `f(e)`),
  §9 (labeler takes no affective argument), §11 (no affective maturation),
  §16 (three knobs in the walk, seed inside the hash), §17 (doors' new
  semantics, budget back to a queue rule, swap test re-run in the removal
  direction, idle wiring).
- **Gates:** **G9 REPLACED** — from "EMOTIONAL MEMORY" (which tested a designed
  coupling) to **AROUSAL + OBSERVED EMOTIONS**: modulation visible in traces,
  labels consistent with logged signals, **no emotion state in control except
  via `a`**, plus the carried-over frozen-genome, bounded-feedback and parity
  legs; **G10 IDLE THOUGHT added** with the owner's verbatim leg (e); **G5**
  gained (f) monologue determinism. `coupling_sweep.json` retired →
  `arousal_sweep.json` (+ `G10/ignition_sweep.json`).
- **Register:** **R-21 re-pointed** (costume now has two shapes — `if-signal-
  then-behavior` *and* any hand-wired coupling between names, in code or
  config), R-14/R-20/R-22/R-23 amended, **R-24** label-feeding-control and
  **R-25** idle-masquerading-as-learning added. §15 DoD → **ten gates** *(round 3
  → eleven; R-22's guard gains the median; OD-23's "boost decays" half is
  superseded by OD-27).*
- **Ledger:** **OD-21** (matrix cancelled), **OD-22** (what may read `a` —
  budget/salience lose their input), **OD-23** (`resident`→`signal`, `sign`
  dropped), **OD-24** (CAUTION out of arbitration), **OD-25** (four ignition
  weights), **OD-26** (trace extended with the idle tail); OD-16 **superseded**,
  OD-17 **cancelled**, OD-18 **moot**, OD-19 **reversed**, OD-20 **stands**.
- Numbers: every arousal, modulation, bias and ignition magnitude remains
  **UNMEASURED**; nothing is quotable before its sweep artifact exists.

## 2026-10-22 — EMOTIONS fragment (synthetic organ system woven into the loop)
*(**Round 1 — substantially superseded by the entry above.** Read it as history:
the six residents, the coupling matrix, `f(e)` salience, CAUTION-in-arbitration
and G9 EMOTIONAL MEMORY are cancelled; internality, charge, the embeddings ban
and the swap-test verdict survive. OD-21 is the ruling.)*
- **Owner DESIGN LAW recorded as invariant I8 (docs/00):** affect is **loop
  state** — computed inside THINK from measurements the organs already produce,
  decaying in logical `t`, reaching behavior **only through the drive
  interface**; no privileged sensors, no external setter, and **any
  `if <signal> then <behavior>` is a rule engine wearing a costume and is
  rejected**. Organs stay six; the DRIVE layer gains its residents.
- **New docs/18 — the emotions engine.** Anatomy (SIGNAL SOURCE / STATE
  INTEGRATOR / EFFECTORS — "an emotion without a source signal is theater; cut
  it"), core six residents (CURIOSITY, SURPRISE, CONFUSION, SATISFACTION,
  BOREDOM, CAUTION) each bound to an existing internal signal, **coupling
  matrix as a first-class artifact with a justification per edge**, emergent
  boredom/frustration (readouts, not rows), homeostatic decay + mood = slow
  components, **emotional edges** (write-side stamping, read-side
  mood-congruent bias, per-round normalization cap), law consequences
  (frozen genome never stamped; charge is form not content; supersession keeps
  emotional history; boost decays, stamp is permanent; every affect change
  citable), boundaries (social emotions excluded and *named*, curiosity stays
  the default resident, CONFIG SI born with the full engine).
- **Emotional charge added to the substrate:** `charge` field on L3
  belief-bearing rows/edges (§5.4), absent by schema design from frozen layers
  and from `context` rows; §8.4's salience hook becomes `f(e)` (was flat 1.0)
  with a charge-decay row; §16.1's walk equation gains `W_eff` bias + the `e`
  vector; §16.6 puts `e` and stamps inside the state hash.
- **Pipeline changes:** §6.1 now runs `drive.update(signals, t)` at the surprise
  step and stamps charge through the drive at commit,
  composes CAUTION at arbitration (raises the cost of `assert` vs `emit` — it
  is the *reason* Act 5's ASK branch fires), and lets
  SATISFACTION damp the slot it just closed; §7.0/§7.3 gain the bias step
  (`drive.conduction()`) with trace attribution; §16.1 states the walk's two
  door-calls. **§17.3's question budget converted from counter to
  starvation** (level-driven allowance, integer cap kept only for determinism).
- **Swap test executed for real, and it partly failed — reported, not
  papered over:** the literal "no walk line changes" reading broke (the walk
  gained one anonymous vector + two drive-interface calls) while the semantic
  reading (no emotion names outside `affect.py` + `drive.py`, no path in except
  `drive.update`, no path out except `drive.*`) holds.
  Recorded as **OD-14**, with the contract growth it forced recorded separately
  as **OD-20**.
- **§4 trace v2 → v2.1:** `e` annotations added per act (no step renumbering),
  plus the **trajectory curve** the log must print (curious → surprise-stamped
  first synapse → satisfied → contradiction-driven caution → slow release),
  all magnitudes UNMEASURED pending `gates/G9/coupling_sweep.json`.
- **Gates/risks:** **G9 EMOTIONAL MEMORY** (a stamped conduction, b state-
  dependent recall beyond the weight delta, c zero frozen stamps, d bounded
  feedback + costume test); G5a hash scope, G7 boost-inheritance rule;
  R-20 status updated, **R-21** costume/theater, **R-22** emotional runaway,
  **R-23** charge-as-content + genome leak; **R-14 amended** to state why
  affect is inside the anti-cheat law and embeddings are not. DoD → **nine
  gates** + trajectory requirement.
- **Ledger:** OD-14 (internality vs swap test), OD-15 (logical-`t` decay vs
  wall-clock "natural"), OD-16 (composition = gating with a floor), OD-17 (does
  a newborn feel caution), OD-18 (aversion designed but not activated),
  OD-19 (budget: starvation replaces the counter — supersedes OD-13's framing),
  **OD-20** (the DRIVE contract grows from four doors to six — `update` +
  `conduction`; nod or amend).
  New open legs OPEN-16…OPEN-21 (coupling constants, composition, bias shape,
  runaway guard, maturation, memory-stamp boundary). §12 gains `affect.py`
  (the tissue) and the rule is that emotion names exist only in `affect.py` +
  `drive.py` — everything else sees numbers and doors.

## 2026-10-22 — CURIOSITY fragment (drive layer)
- **New docs/17 — DRIVE layer.** Curiosity is the *default resident* of a
  cross-cutting interface `{prioritize, emit, budget, salience}`, **not** a
  seventh organ (organs stay six) and **not** a reflex welded into the read
  path. Design test published as the **swap test** (§17.6): a second drive
  (affect, v2 — explicitly *not* designed here) must be addable without
  touching walk, write path, or gates. §3 adds R15 (interface) + R16
  (curiosity policy, content-free: `what is znark?` fires identically for
  gibberish).
- **Three behaviors** (§17.2): ASK-BACK (named missing slot → question, not
  "I don't know"), UNKNOWN-WORD ASK (`what is X?` for a token with no
  hyperpoint), AUTO-COMMIT (owner reply to a pending ask = ground truth by
  context; provenance `kind: answer-to-ask`, `question_id` link; `/correct`
  stays the override channel).
- **Two teaching channels** now in §1.4 — a bare statement with no pending
  ask and no pattern is *input, not a teach* (that gate is what keeps
  auto-commit from becoming a content-guessing machine). OD-12 opened.
- **Machinery:** `questions` shelf added to the registry (§5.2, append-only,
  status via new version); write path re-ordered to check pending asks
  **before** pattern match (§6.1); read path §7.2 rewritten as *detection ≠
  asking*; §8.4 gains T3-context init + `drive.salience` modulation
  (default 1.0, in the state hash, clamped); §9 makes FACT cite the
  ask-event; §16.4 hands gaps to the drive and adds the **idle queue
  revisit** (consolidation = closing your own loops).
- **DEMO TRACE v2** (§4): the newborn's first output is a question; the
  first lesson is a plain `red.` reply that auto-commits r1 and resolves
  q1; recall says *"you answered my question at s5 (q1, asked at s3)"*.
  Steps renumbered s1–s24; Acts 3–5 (sentence form, supersession,
  contradiction ASK) unchanged in substance. **OD-10 RESOLVED**
  (ask-back primary); OD-02 rationale annotated.
- **New gate G8 CURIOSITY LOOP** (§13): (a) gap → exactly one ask, no
  fabricated answer; (b) plain reply auto-commits (+ no-pending-ask control
  case); (c) recall cites the ask-event; (d) budget + dedupe; (e) idle
  consolidation. Failure meanings per leg: (a) faking is back, (b) teaching
  loop broken, (d) curiosity unbounded, (e) consolidation missing. G1/G3
  annotated for the new trace. Definition of Done now **eight gates** + the
  swap-test leg.
- **Risks/decisions:** R-20 added (curiosity welded into the read path —
  grep test as closure), R-13/R-19 reworded for ask-rows vs knowledge rows;
  §12 maps `drive.py` as the only module allowed to name curiosity;
  OD-10 resolved, **OD-12** (two channels) and **OD-13/OPEN-15** (budget
  parameters = UNMEASURED until `gates/G8/budget_sweep.json`) opened.

## 2026-10-22 — DECODE fragment (decode/render layer)
- **Sixth organ added** by owner: DECODE/RENDER — one component, two
  faces; surface ↔ hyperpoint pattern (subject + relation + frame); the
  walk consumes patterns, never raw spelling. Full pipeline:
  decode → light → walk → settle → reverse-decode (§7.0).
- Anti-cheat line for decoder stated: frames are pure shape (P_QUEST_A/D/W,
  P_OF_REL added to §3.2 with {who, when, where, of} in the allowlist);
  gibberish litmus (`what znark is floop` → same shape) inherited by
  G1/G2/G5d.
- Decoding is LEARNED growth: taught synonyms/paraphrase templates become
  `frame-synonym` overlay edges created by `/correct`; RE-PARSE reframed as
  decoder maturing retroactively (§6.6).
- I-DON'T-KNOW made precise: decode succeeds, walk dies at intersection ⇒
  NAMED missing slot ⇒ targeted ask-back. **Demo Act 0 rewritten** to
  ask-back primary, nonce demoted to fallback (owner said "consider" —
  OD-10 veto open).
- G2 amended: two barriers — DECODE barrier (same decoded pattern via
  taught template edges, zero token overlap) AND WALK barrier (path finds
  the edge), trace-verified per recall.
- §9 labels updated (named-dark surface); risks R-18 (decoder content
  drift), R-19 (ask-back echoes read as understanding); OD-10/OD-11 opened;
  §2 now six organs (owner amendment noted).

## 2026-10-22 — THINK fragment (walk + hardware contract)
- New **docs/16**: spreading-activation walk (per-token pulses in word
  order, K rounds or τ-settle, max-K ⇒ uncertainty), hub damping
  (PageRank-style) + cooling decay, frontier ~10² / ~10⁵–10⁶ edge touches
  (owner design targets, UNMEASURED); GPU-walker/CPU-judgment contract,
  one sync per thought; freeze/wake = genome version + overlay replay;
  residency wall at 10⁸+ edges → region paging v2+.
- Embeddings **banned even as helpers** (R-14; G2 static scan).
- Folds: §1.3 restated (miss = no lit content node; frame-reword = term
  identity, OD-07); §7.1 gaps = dark spots in the spread; §7.3 read path IS
  the walk, activation trace = first-class route report; §8.1 gain =
  counterfactual minus-edge re-walk; §9 labels measured from the walk,
  **BASE capped at INFERENCE (OD-04 resolved)**, uncertainty readout;
  §5.3 walkable-web paragraph; §4 trace s2/s6/s8–9 annotated (edges+trace);
  §13 G2 rewritten (tiers T1/T2/T3 + constants sweep), G4 oscillation
  scene, G5 quantized-hash + trace determinism, G6 hardware-budget leg;
  §14 adds R-14…R-17.
- Open legs recorded, not invented: OPEN-11 oscillation, OPEN-12 constants,
  OPEN-13 phrase-batching (default off), OPEN-14 discrete error metric.
- OD ledger: OD-04 RESOLVED; OD-06 (SI walks L3-only), OD-07 (paraphrase
  semantics), OD-08 (v1 CPU walker vs GPU), OD-09 (constants surface)
  opened for owner confirmation.

## 2026-10-22 — Layout split (this commit)
- DESIGN.md (single big doc) → `docs/00…15` numbered section files +
  `docs/decisions/` ledger + `gates/` artifact reservation. DESIGN.md
  demoted to master index; § numbers unchanged, cross-refs map via index.
- Fixed OD-05 (confidence init 0.95 for T3 across trace + §8.4).
- Added R-13 (guess-slot-looks-like-cheating risk) while splitting — the
  fragment's Act 0 guess needed its own false-pass route on the register.

## 2026-10-22 — SI merge (fragment 2)
- Renamed "Baby SI" → **SI**; newborn framing confined to stages.
- Adopted owner-decided: five organs (THINK/WHAT/KNOW/LEARN/CALIBRATE);
  `/correct` protocol as THE teaching act (exact-gain triples, whole-output
  binding, length = curriculum); S-R-pair storage at S0 + RE-PARSE at S1+;
  Tesseract-family substrate with `(t,x,y,z)` addressing, shelf registry,
  frozen L1 WordNet / L2 Matrix correlations (Critical Law: not SI's
  knowledge), live L3 lattice; CONFIG SI vs CONFIG ENGINE clean-room split;
  BASE/OWNED source tags; gates extended G1–G5 → G1–G7 (G6 baseline
  integrity, G7 re-parse).
- Rewrote demo trace to the new north-star (question-first, guess,
  correction). Kept from v1: invariants, pattern allowlist + G5c audit,
  surprise formula, arbitration ASK branch, FACT-copy rule, determinism.
- Opened OD-01…OD-04 for owner ruling.

## 2026-10-22 — v1 design (original)
- DESIGN.md #1: birth state, apple trace, row schema, write/read paths,
  currency, labels, stages, components, gates 1–5, risk register R-01…R-09.
