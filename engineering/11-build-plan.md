# 11 — The Build Plan: fourteen stages, machine-checkable done-conditions, and the stop rule

Provenance. Phase 2 of the build job. This is the only file in this folder that is
not a specification: it is the **plan the build obeys**, written by the engineer
who will execute it. It adds no law, no constant, no gate, no artifact and no
state byte. It orders what 00–10 already fix, names the gate ids that decide
whether a stage is done, and states in advance every reading the build would
otherwise have to make mid-flight.

The authority order still holds: `docs/` > this folder > this plan. A conflict
between this plan and 00–10 is a bug in the plan. A conflict between 00–10 and
`docs/` is an OD entry, never an edit. **When Phase 3 starts, every file in
`docs/` and `engineering/` is read-only**; the build's only writes are `src/`,
`Cargo.*`, `harness/`, `gates/` (outputs), `runs/` (outputs) and
`failure_report.md` if a wall is hit.

---

## 1. What "done" means here

| rule | statement |
|------|-----------|
| **D1** | A stage is done when **every leg of its exit gate set** has run and passed on the stated variant(s), and every measurement leg that cannot pass yet is listed in `gate_report.json`'s `unresolved_legs` with status `UNMEASURED`. Nothing is done because it "looks finished". |
| **D2** | "Passed" is an artifact: a `gate_report.json` with `pass: true` and a `criteria[]` row per leg, written by `si-harness` under `gates/`. A green unit test is not a gate. |
| **D3** | A gate that has not run is not green, not "implicitly green", and not "the same code as the gate that did run". |
| **D4** | A stage's DoD is checkable by a command line printed in this file. If a DoD sentence cannot be turned into a command, it is rewritten here before the stage starts, not interpreted while coding. |
| **D5** | No value that a sweep owns is ever written by hand. Unruled constants take **OD-86's placeholder**, are tagged `UNMEASURED_PLACEHOLDER`, appear in every report, and no gate leg passes *because of* one. `--strict-consts` refuses a run that still has one. **A gate whose values decide *emission, cadence or shape* (G2, G8, G9, G10, G13) runs its sweep runner in the same stage it first goes green; a placeholder may keep a scenario executable but may not decide a gate's verdict.** |
| **D6** | A red gate is attempted at most **three times**. Each attempt is a fix followed by a re-run of the same leg. The third red attempt writes `failure_report.md` (§6) and stops the run. |
| **D7** | The spec is read-only. A wall is reported as a contradiction and an OD question; it is never routed around in code. |
| **D8** | Every commit message names the engineering sections implemented (§8). A commit with a red exit gate is not made. |

**D1's second half is the honest one.** G6.d.1 (hardware budget), every G12
rate, every G13 cadence/dimming magnitude and every G14 spread are UNMEASURED by
construction until their sweep directories exist. A report that omits them reads
as a pass and is a D1 failure even when `pass: true`.

---

## 2. Readings pre-committed so the build cannot decide them while coding

Each row is a place where two faithfully-read sentences meet and the build needs
one resolution. They are readings of this folder, not new design; the first two
are the only ones with a ledger footprint.

| # | meeting point | reading this build takes | why this is the reading |
|---|---------------|--------------------------|--------------------------|
| R1 | 00 §2 says "**Sixteen packages**" from B14 (it said "Thirteen crates" through B13); its table's last row is `` `si-cli` / `si-harness` `` and the map draws `si-harness` under `si-cli` | **13 packages through B13, 16 from B14**, with `si-cli` and `si-harness` as two **binaries of the `si-cli` package** at every point in the run. The harness code lives in `si-cli/src/harness/`; only `src/bin/si-harness.rs` opens `gates/`. The arc's three (`si-crystallize`, `si-silt`, `si-skills`) are created **together at B14**, with silt and skills as stubs until their own stages, because a package's dependency edges are its contract and B14 is where the final graph is cut | counting the table's rows gives the same number as `cargo metadata` only if the two-binary row is read as one package — which is what R1 says; and from B14 a builder who counts 13 has not added the three leaves, so `CRATE_NAMES: [&str; 16]`, `check_deps`'s 16-name scan and B0/B13's `cargo metadata` assertion all move together, once, at B14 ([12 §9 row 1](12-v2-arc.md)) |
| R2 | 03 §1 says every swept row's default is `—`; a build cannot compile an empty value | **OD-86's placeholder policy**: midpoint of the legal range on the row's grid, tagged, printed, counted in `unresolved_legs`, replaceable only by a sweep artifact | OD-86 is the ledger row; this plan only binds the build to it |
| R3 | 05 §21 prescribes a per-commit behavioral short set; stages before the loop exists cannot run it | **the short set begins at B5**, the first commit with a session to run it against. B0–B4 run the `check_*` set enabled through their stage (§4). No gate is ever marked green early | the set presupposes a runnable session; running it against stubs would test the stubs and call it a gate |
| R4 | 08 §2 names kind 11's producer as "step 9", while 00 §2 forbids `si-session ↛ si-audit` and 04 §21 U.3 says the audit stream joins in the exporter/viewer over the written log | kind 11 carries the **claim label** computed by `si-front::label` at step 9 (04 §21 U.1–U.2). The six **emotion** labels are `si-audit`'s post-hoc `LabelStream` over the log; the viewer renders them from that artifact, never from a loop frame | the crate law is explicit and 04 §21 U.3 states where the audit stream joins; the frame's payload in 08 is a claim label with citations |
| R5 | G6 needs a `SIgen`; no WordNet/ConceptNet source is present offline, and 07's manifest is the lock | B10's genome artifact is built from a **pinned fixture manifest** under `harness/fixtures/genome/` (tiny owner-authored sources, hashed, licensed field non-empty). G6 runs on it and its report names it `fixture`. The shipped corpus is a fetch-time artifact of the owner, not a build output | 07 §1–§2 make the manifest the only source list; a fixture manifest is a legal source list. No content claim is made and no density number is quoted as a corpus property |
| R6 | V2/V3/V9 are "runtime launch states" while constants are compiled and there are no setters | the harness passes a **launch-state override** to `si-cli run` (`--harness-launch <variant>`), which selects the constant value **and its version tag for that run**. The loop has no general setter, the viewer/seam cannot reach it, and a normal session cannot pass it | 05 §2 is explicit that these are launch states, not code paths; a version tag per run keeps the hash attributable (03 §1) |
| R7 | "the GUI is tested" for a native window | the GUI smoke is a **process assertion**: `si-cli view --session <run> --exit-after-frames 120` exits 0, writes a frame log, and the run's `state_hash.txt` is unchanged. Visual inspection is reported as **not performed** unless the owner asks for it | 08's laws are about the loop not needing the viewer and the viewer writing nothing; both are machine-checkable. Pixel aesthetics are not a gate in any file |
| R8 | 06 §7's chained sub-digests and 01 §10's nine items | one `HashItem` accumulator per item, chained per shelf; P3 triggers a full edge-class re-digest; the digest function is `md-5` over canonical LE bytes | OD-60 already fixes the construction; the build only implements it |

---

## 3. Stage list at a glance

`B` stages, because `S0–S3` already mean the design's capability stages (birth,
derivation, generalization) and reusing them here would make every gate citation
ambiguous.

| id | stage | fills | depends on | exit gate set (full list in §5) | commit subject |
|----|-------|-------|------------|----------------------------------|----------------|
| **B0** | workspace, kernel, gate skeleton | all 13 packages (stubs), `si-core`, `si-cli`/`si-harness` shell | — | checks C0 (11 static legs), unit suites | `B0: workspace + kernel — engineering/00 §1–§5, 01 §1–§3, §9–§10, 03, 05 §20` |
| **B1** | store, container, wake | `si-store`, `si-session` (shell), `si-core` hash items | B0 | C1 checks + unit: layout/atomic/resume/seed-derivation | `B1: store + wake — engineering/01 §4–§13, 04 §25, 06 §1–§3, 10 §3–§4` |
| **B2** | affect + drive | `si-affect` | B0, B1 | C2 checks + unit: `a`, bake, doors | `B2: affect + drive — engineering/04 §6–§8, §16, 01 §8, 03 §3–§4` |
| **B3** | walk + idle engine | `si-walk` | B1, B2 | C3 checks + unit: spread/tie-break/settle/idle gate | `B3: walk + idle engine — engineering/04 §5, §10, §19, 03 §2, §5` |
| **B4** | surface + retrieval | `si-front`, `si-read` | B2, B3 | C4 checks + unit: decode/teach left set/render containment/retrieve | `B4: front + read — engineering/04 §2–§4, §20, 09 §2–§3, 03 §3.2` |
| **B5** | write gate + the loop | `si-learn`, `si-session` (tick), `si-cli run/feed` | B3, B4 | **G1, G2, G3, G4, G7, G8**, G14(c)(f) — incl. the **G2 and G8 sweeps** | `B5: write gate + loop — engineering/04 §11–§18, §24, 02 §11, 09 §4–§7` |
| **B6** | audit labels | `si-audit` | B5 | **G9** incl. its sweep | `B6: audit labeler — engineering/04 §21–§22, 08 §2.1 kind 11` |
| **B7** | idle thought + monologue | `si-walk::idle` wiring, `si-session` lane | B5 | **G10** incl. its sweep | `B7: idle thought — engineering/04 §19, 08 §2, docs/19 law` |
| **B8** | sleep | `si-sleep`, step 12b, `sleep.jsonl` | B5 | **G13** incl. both sweeps | `B8: sleep — engineering/10 §7–§8, 04 §28, 06 §3.4, 05 §18` |
| **B9** | Grimm supporter | `si-supporter` | B5 | **G12**, C9 checks | `B9: supporter — engineering/02 §8, 04 §23, docs/21` |
| **B14** | arc substrate + crystallize | `si-crystallize` (+ `si-silt`/`si-skills` stubs with final edges), frame scratch, 22 registry rows, `CONSTS_VERSION 0x0002` | **B9** | **G15**, C14 checks, every §6.3 check re-green | `B14: crystallize + arc substrate — engineering/12 §1/§3/§5 §29, 02 §11, docs/22` |
| **B15** | silt | `si-silt`, the fifth ignition term, the `primed` tag, `silt.jsonl` | B14 | **G16**, C15 checks | `B15: silt — engineering/12 §5 §30, 02 §11, docs/23` |
| **B16** | episodic | `si-read::event_view`, `OriginTug`, the re-light path | B15 | **G17**, C16 checks | `B16: episodic — engineering/12 §5 §31, 04 §4/§31, docs/24` |
| **B17** | skills | shelf 9 live, `si-skills`, `NAMELESS`, composition + forwarding | B16 | **G18**, C17 checks | `B17: skills — engineering/12 §2/§5 §32, 01 §2/§6, docs/25` |
| **B18** | world | `si-core::world`, the class function, the tier arm, lane + windows, the scenario's `identity:` tag | B17 | **G19**, C18 checks | `B18: world — engineering/12 §5 §33, 02 §12, docs/26` |
| **B10** | genome pipeline + ENGINE loader | `si-store::build`, loader bodies, `SIgen` | B1, B5 | **G6** (fixture), G5(c) handle audit, C10 checks | `B10: genome build — engineering/07, 04 §25 W.1–W.5, 00 §6` |
| **B11** | viewer + five panels | `si-vulkan`, `si-cli view` | B6, B7, B8 | GUI smoke, `check_gui_observer_only`, `check_headless_run`, `check_no_viewer_state_write` | `B11: viewer — engineering/08 §1–§9` |
| **B12** | sweeps + variant matrix | ten sweep runners, V0–V9 | B5–B10 | G11, G14 full; G2/G9/G10/G13 panel-full; V-matrix | `B12: sweeps + variants — engineering/05 §2, §4` |
| **B13** | global acceptance | no new code | all | **G5**, full `check --all`, crate map, `tree_sha256.txt` | `B13: global acceptance — engineering/05 §21` |
| **B20** | tongue | `si-core::tongue`, the 17th crate `si-tongue`, the one call site, 8 registry rows, `CONSTS_VERSION 0x0003` | B13 | **G20** incl. its refused sweep, C20 checks (6), `check --all` = **52** ids | `B20: tongue — engineering/13, 02 §18, 04 §34, 05 §24, docs/27` |

**Why this order and not another.** `si-core` before everything (it is the leaf);
`si-store` before any organ (nothing has bytes to think about otherwise);
`si-affect` before `si-walk` (the walk's per-round order calls the drive after the
signal table and before `conduction`); `si-walk` before `si-read` (retrieval is a
walk with a question attached); the write gate after both (it consumes surprise
from the walk and coverage from the store); labels after the loop (R-24);
idle after the write path (its gate needs a quiet tick and its bake reads need
commits); sleep after the loop (it is step 12b and calls the walk); Grimm after
sleep because G12's control is a sleep-aware run; the genome after CONFIG SI is
green (I2's whole point); the viewer last (08 §1: a loop that needs the GUI has
failed); sweeps after every constant's consumer exists; acceptance last, because
G5 hashes the output of everything else.

**Where the arc and the tongue sit, and why the ids are out of order in the table
above.**
Execution order is B0…B9, **B14…B18**, B10…B13, **B20**, and `Stage`'s **enum
declaration order is execution order** (`ALL: [Stage; 19]` through B13,
`[Stage; 20]` from B20, `label()` carrying the printed id)
so that 05 §21's per-stage prefix semantics — `enable_stage(id) <= stage` — stay
true for every stage that follows, including B13's acceptance pass, which then runs
the full **46-check** union, and B20, which is **last** precisely so its
`check --all` is the **52**-id union with nothing after it. Two facts fix the arc's
position. **(i) B9 is a hard
prerequisite:** G15(d), G16(b), G17(e), G18(d) and G19(a)(d)(h) each need a
non-owner line arriving through the seam, and G19(h) *reuses* the two-layer speak
gate rather than meeting it — the arc has no test of delegated authority that does
not go through Grimm's door. **(ii) The genome and the viewer do not care about the
arc**, so B10–B13 keep their relative order and B13's acceptance pass simply runs
over nineteen gates instead of fourteen. **B20 is a third case, and it is the
tongue's own:** it depends on nothing but B13's green battery, because its question
is "does a drawn voice change the words and no byte of state?" — a question that can
only be asked once there *are* words and a state hash to compare against. Its
`CONSTS_VERSION` bump then sends B13's whole set back through the machine
(§4 B20 DoD 6), which is the same price B14 paid for its 22 rows.
The ids are numbered by *batch*, not by
*sequence*, and a reader who wants the sequence should read the table's rows in the
order they are printed.

---

## 4. Stage details

Each stage lists: **implements** (the exact engineering sections), **creates**,
**DoD** (commands, no adjectives), **exit gates**, **tests live in**, and the
**enable-set** of static checks. The static checks are turned on by stage; a
check whose subject does not exist yet is **disabled and listed**, never green.
The enable table is a `const` in `si-cli/src/harness/checks.rs`.

### B0 — Workspace, kernel, gate skeleton

- **Implements.** 00 §1–§5 (toolchain, workspace, conventions), §12 (crate laws);
  01 §1–§3 (addresses, shelf ids, grids), §9 (containers), §10 (hash item list),
  §11 (nulls); 03 §1–§8 (registry, class column, geometry); 04 §1–§2 (order
  vocabulary); 05 §1 (modes), §3 (report), §20 (the check runner).
- **Creates.** Root `Cargo.toml` (workspace, resolver 2, `default =
  ["vulkan","supporter"]`); all **13 packages** with final dependency edges and
  empty bodies; `si-core` with: `addr`, `shelves` (11 entries, 3 reserved),
  `quant` (Q12/Q8/u8/hundredths, reciprocal multiply, round-half-up), `rng`
  (PCG32, `RngState{state, inc, spare}`), `clock` (logical `t` + `recency`),
  `container` (magic/version/header/payload), `eventlog`, `consts` (every row of
  03 with lo/hi/grid/home/gate and OD-86 placeholders), `hash` (nine `HashItem`s,
  chained, empty-state value), `seed` (derivation arithmetic, unused until B1);
  `si-cli/src/bin/si-harness.rs` with `check`, `run`, `replay`, `verify` (the
  last three print `not built at this stage` and exit 2).
- **DoD (commands).**
  1. `cargo metadata --no-deps --format-version 1` ⇒ exactly 13 package names,
     the set in 00 §2.
  2. `cargo check --workspace --all-features`, `cargo check --workspace
     --no-default-features --features vulkan,supporter`, and `cargo check
     --workspace --no-default-features` ⇒ exit 0.
  3. `cargo test -p si-core` ⇒ exit 0; tests assert: every record `len % 4 == 0`;
     reserved bytes zero; `quant` rounds half-up without division; two `Rng`
     instances from one seed produce identical draws; empty-state hash equals the
     checked-in constant; a container round-trips byte-exactly; the placeholder
     for every swept row equals `lo + round((hi−lo)/2)` snapped to grid; the
     registry's swept-row count equals the placeholder count.
  4. `si-harness check --stage B0` ⇒ exit 0, report with all legs `pass`, and the
     disabled checks listed by id.
- **Exit checks (C0).** `check_deps`, `check_emotion_words`, `check_membrane`,
  `check_no_embeddings`, `check_no_json_readback`, `check_record_layout`,
  `check_registry_size`, `check_string_constants`, `check_one_clock`,
  `check_teach_literals` (the table exists in core's schema even before decode:
  exactly ten `P_*` names, `P_TEACH` present), `check_temperament_closed` (the
  vector type exists), `check_sweep_never_touches_temperament` (a sweep may never own
  a draw — **OD-87**; it needs only the registry and the axis enumeration, so it
  belongs to C0 rather than to B7, where the sweep it protects arrives).
- **Tests live in.** `si-core/tests/*`; harness stage sets in
  `si-cli/src/harness/stages.rs`.
- **Commit.** `B0: workspace + kernel — engineering/00 §1–§5, 01 §1–§3/§9–§10, 03 §1–§8, 05 §20`

### B1 — Store, container, wake

- **Implements.** 01 §4–§13 (records, edges, stamps, body notes, interning, two
  stored streams, serialization, hash composition, nulls, paste, core block);
  00 §5–§6 (containers, config guard); 04 §24 X.1–X.2 (hash and chained
  sub-digests), §25 W.0–W.5 (wake/freeze); 06 §1–§3, §3.4 (geometry and core
  block); 07 §9 (loader stub); 10 §3–§4 (seed storage, P.1–P.3), §10 (fingerprint).
- **Creates.** `si-store`: three append-only arenas (112/16/16) with `u32` byte
  offsets, interner, `active_at`/NOW, `append_row`/`age` primitives, out-degree
  and composite-tally derived RAM; session store open/save (`SIck`/`SIst`) with
  atomic write and geometry/seed refusal on resume; frozen loader stub returning
  `ConfigRefusal` before a path exists when `engine` is off. `si-session` shell:
  `new(seed)`, W.0 derive-and-compare, `fingerprint.txt`, wake/hash. `si-core`:
  the nine hash items over the arenas.
- **DoD.**
  1. `cargo test -p si-store -p si-session` ⇒ exit 0; asserts: every arena write
     asserts alignment and zero pads; resume with a different seed/geometry
     returns the refusal before any open; atomic save leaves either old or new
     file after a simulated failure; the core block is 36 B with the 13 mutable
     bytes at their documented offsets; the derived vector is not present in
     `SIst` (byte scan + schema), while its digest is.
  2. `si-harness check --stage B1` ⇒ exit 0 with C0 + `check_config_guard`.
  3. A wake-only run (`si-cli run --seed 0x… --transcript harness/scenarios/empty.txt
     --run-dir runs/b1-smoke`) exits 0, writes `SIst`, `mindlog.json`,
     `state_hash.txt`, `fingerprint.txt`; the hash is identical across two runs;
     the same command with `--seed 0x…2` differs **only** in the core block's seed
     fields and digest, and `fingerprint.txt` differs only in derived values.
- **Exit checks (C1).** C0 + `check_config_guard`, `check_bake_is_the_exception`
  (the `bake` column exists and is stored), `check_no_mood_field`.
- **Tests.** `si-store/tests/{arena,container,resume}.rs`,
  `si-session/tests/{wake,seed}.rs`, unit tests in `si-core/src/hash.rs`.
- **Commit.** `B1: store + wake — engineering/01 §4–§13, 04 §24–§25, 06 §1–§3, 10 §3–§4`

### B2 — Affect and the six doors

- **Implements.** 04 §6–§8 (signal table, `drive.update` order, `conduction`
  two-term cap), §16 P (salience/bake, one instant, one curve); 01 §8 (`a`
  trajectory, rehearsal/decay pair); 03 §3–§4 (arousal and drive constants);
  04 §10 (the one recency curve's consumers); 10 §6 (dimming consumes recency —
  the curve, not a new rate).
- **Creates.** `si-affect`: six `SignalKind`s and their birth table; the single
  `a` integrator (`a' = clamp(a·λ_a + Σ gains, 0, 1)`, per-event order, one-round
  apply lag); `bake(a_formation)` as a pure function with the `a_formation = 0 ⇒ 0`
  law; the six drive doors (`prioritize`, `emit`, `budget`, `salience`, `update`,
  `conduction`); `budget()` with no `a` parameter; modulation triple returned by
  `conduction`.
- **DoD.**
  1. `cargo test -p si-affect` ⇒ exit 0; asserts: events apply in emission order;
     the lag field records the round applied; `a` never leaves `[0,1]`; the bake
     is monotone, saturating, exactly 0 at `a_formation = 0`, and cannot be
     recomputed after formation (the only assignment site is `salience`); all six
     doors exist and no seventh; `budget` takes no affect argument; no float in
     the crate.
  2. `si-harness check --stage B2` ⇒ exit 0 with C1 + `check_signal_kinds`,
     `check_bake_no_event_kind`, `check_one_clock`, `check_drive_doors`
     (the door-surface half; the walk-call-site half turns on at B3).
  3. A unit vector from 04 §7 reproduces the worked `a` sequence to the integer.
- **Exit checks (C2).** as above + `check_no_affective_setter` is **still
  disabled** (no supporter yet) and listed.
- **Tests.** `si-affect/tests/{a_integration,bake,doors}.rs`.
- **Commit.** `B2: affect + drive — engineering/04 §6–§8/§16, 01 §8, 03 §3–§4`

### B3 — Walk and the idle engine

- **Implements.** 04 §5 D (eleven ordered steps per round), §10 I, §19 S
  (precedence, one sampler, read-only cascade), §5.4 (composition trace,
  route counts); 01 §8; 03 §2, §5; 02 §4.
- **Creates.** `si-walk`: `h_k = decay · norm(h_{k−1} + W_eff·act_{k−1})` with
  the per-round order gather → `drive.update` → advance `a` → `drive.conduction`
  → spread (ascending handle/offset) → hub damp → sum/decay → normalize → τ test;
  frontier cap/trim with the (activation desc, first-lit round asc, point id asc)
  tie-break; settle set hashed by point id, displayed by activation; the ignition
  sampler (reads stored `bake`, one draw per quiet tick from stream R), idle
  gate, and the monologue cascade as a **read-only** path.
- **DoD.**
  1. `cargo test -p si-walk` ⇒ exit 0; asserts: a hand-computed 3-point graph
     reproduces expected Q12 activations for 3 rounds; edge iteration order is
     fixed across insertion orders (sort by handle before spread); reactor order
     is single-threaded; settle stops for each documented reason; trim order is
     exact; ignition draws only from `R` and reads only stored `bake`; the idle
     gate admits only an empty agenda + quiet senses; no symbol from `si-learn`
     is reachable.
  2. `si-harness check --stage B3` ⇒ exit 0 with C2 + `check_idle_no_commit`,
     `check_drive_doors` (both halves: exactly two call sites), `check_no_mood_field`.
  3. A two-hop fixture walk produces the same trace twice; changing insertion
     order does not change it.
- **Exit checks (C3).**
- **Tests.** `si-walk/tests/{spread,settle,idle}.rs`.
- **Commit.** `B3: walk + idle engine — engineering/04 §5/§10/§19, 03 §2/§5`

### B4 — Surface and retrieval

- **Implements.** 04 §2–§4 (tick head, decode, lighting), §20 T (history walk as
  the read half), §21 U.1–U.2 (label + render containment); 09 §2–§3 (grammar,
  malformations, decode contract); 03 §3.2 (string allowlist); 01 §7; 02 §6–§7.
- **Creates.** `si-front`: R1–R3 tokenizer, ten `P_*` frames including `P_TEACH`,
  `=>` split on first marker, malformation outcomes, renderer that copies and
  does not create, claim label (`FACT`/`INFERENCE`/`SPECULATION`/`UNCERTAINTY` +
  `OWNED`/`BASE` tag) with citation and route counts. `si-read`: retrieve by
  decoded pattern key, ask engine (four question frames), guess slot with nonces,
  `emit` from owner tokens only, gap naming G1–G4, hop counting on **claims**
  (cap belongs to the labeler, not the walk).
- **DoD.**
  1. `cargo test -p si-front -p si-read` ⇒ exit 0; asserts: exactly ten patterns;
     `/teach` split at the first `=>`; empty-left/empty-right/no-marker outcomes
     match 09 §2.1 row by row; the left side accepts only the four question
     frames (`P_ATTRIB`/`P_DEF` ⇒ `left_not_a_question`, zero rows); render
     containment (every content token of an answer ∈ cited surfaces); no emotion
     literal; no template list; retrieve returns the decoded slot's row only;
     `INFERENCE_MAX_HOPS_OWNED` is read in the labeler and nowhere in
     `walk`/`spread`/`settle`.
  2. `si-harness check --stage B4` ⇒ exit 0 with C3 + `check_teach_literals`,
     `check_teach_left_side`, `check_no_teach_vocabulary`,
     `check_string_constants` (full), `check_hop_cap_is_claim_side`,
     `check_candidate_unanswerable`, `check_no_embeddings`.
  3. A decode fixture of 20 lines (checked in, hashed) produces the expected
     `Pattern|Undecoded` sequence byte-for-byte.
- **Exit checks (C4).**
- **Tests.** `si-front/tests/{tokenize,decode,teach,render}.rs`,
  `si-read/tests/{retrieve,ask,guess}.rs`.
- **Commit.** `B4: front + read — engineering/04 §2–§4/§20/§21, 09 §2–§3, 03 §3.2`

### B5 — Write gate and the loop (first end-to-end)

- **Implements.** 04 §11–§18 (gap doors, write pipeline, surprise, five
  consistency rules, arbitration, commit, gain ledger/re-parse), §12 steps 0–9,
  §24 (hash of a live session), §26 ordering laws; 02 §5, §11 (steps 1–13, no
  12b yet), §12; 09 §4–§7 (row shape, arbitration, ledger, bake cohort); 01 §6,
  §12; 05 §6–§9, §12–§14 (G1, G2, G3, G4, G7, G8; the G2/G8 sweeps run
  before their gates).
- **Creates.** `si-learn`: the ten-step gate with its six ordering laws (teach
  before pending-ask, boring before consistency, arbitrate, salience exactly once
  at commit, ledger last), the boring rule's 14 B in-place bump, supersession
  (weight → 0, bake and stamps kept), the gain ledger (`asserted` → promotion on
  first uncorrected serve), RE-PARSE. `si-session`: the tick, run directory
  (`transcript.txt`, `mindlog.json`, `monologue.jsonl` empty,
  `state_hash.txt`, lane frames), source-tagged inbox, per-line logical steps.
  `si-cli`: `run`, `feed` (the inbox-only bulk path), `export`.
- **DoD.**
  1. `cargo test -p si-learn -p si-session` ⇒ exit 0; asserts: all 09 §2.1
     malformations produce their exact row counts and byte costs; the boring
     repeat writes 0 new bytes and bumps exactly the six FLUID columns; a
     supersession keeps the loser's bake and stamps; `salience` is called exactly
     once (instrumented counter); the gain ledger promotes only on a serving
     event; `/teach` writes one row with `question_id = 0`, `bind_output_id = 0`,
     `prov_speaker = 0`, confidence 95, gain 0/asserted.
  2. `si-harness run --gate G1 --variant V0 --seed 0x51 --scenario
     harness/scenarios/g1_write.txt` ⇒ `pass: true`; likewise G3 (supersession),
     G4 (contradiction), G7 (re-parse).
  3. **The G2 and G8 sweeps run here, before their gates.** `si-harness sweep
     --gate G2` writes `gates/G2/constants_sweep.json` (the whole curve, the
     winner, the pair-overlap objective); `--gate G8` writes
     `gates/G8/budget_sweep.json`; the winners replace their placeholders and the
     run's constant version tags change. Then `run --gate G2` and `--gate G8` ⇒
     `pass: true` with `sweeps_cited` non-empty.
  4. G14(c)(f): the run's `SIst` re-derives to its digest; `--seed` is required;
     a resume with another seed is refused; no re-roll verb exists.
  5. The per-commit short set (05 §21) runs from this commit on.
- **Exit gates.** G1, G2, G3, G4, G7, G8, G14(c)(f), C5 checks.
- **Exit checks (C5).** C4 + `check_bake_write_once`, `check_bake_is_the_exception`,
  `check_compose_writes_nothing`, `check_curriculum_is_inbox_only`,
  `check_no_json_readback`, `check_no_seed_control_surface`.
- **Tests.** `si-learn/tests/{gate,arbitrate,boring,ledger,reparse}.rs`,
  `si-session/tests/tick.rs`; scenarios under `harness/scenarios/`.
- **Commit.** `B5: write gate + loop — engineering/04 §11–§18/§24/§26, 02 §11, 09 §4–§7`

### B6 — Audit labels

- **Implements.** 04 §21 U.3 (frame/label separation), §22 V (six predicates
  over the log, reproducibility as acceptance); 02 §9; 08 §2.1 kind 11 (claim
  label) and the EMOTIONS E4 source; 05 §14 (G9 legs that need labels).
- **Creates.** `si-audit`: `classify(event_log) -> LabelStream`, one predicate per
  label, every label citable to logged rows; `coupling_observed.json` output-only;
  the crate is imported by nothing in control.
- **DoD.**
  1. `cargo test -p si-audit` ⇒ exit 0; asserts: every emitted label cites rows
     that exist in the log; deleting the audit crate's output changes no run's
     `state_hash.txt` (V1 build); all six words appear in this crate only.
  2. `si-harness sweep --gate G9` writes `gates/G9/arousal_sweep.json` with the
     bake distribution objective; then `run --gate G9 --variant V0` and V1 ⇒
     all legs green; only G6.d.1-style measurement legs stay UNMEASURED.
  3. `si-harness check --stage B6` ⇒ exit 0 with C5 + `check_audit_edges`,
     `check_emotion_words` (full scan).
- **Exit gates.** G9, C6.
- **Commit.** `B6: audit labeler — engineering/04 §21–§22, 02 §9`

### B7 — Idle thought and the monologue lane

- **Implements.** 04 §19 (gate, precedence, ignition, read-only cascade),
  §19.3 (separate lane); 05 §15 (G10); docs/19's law as fixed by 03 §5.
- **Creates.** session wiring of the idle tick at step 12's quiet branch; the
  `monologue.siro` lane; the V2/V3 launch states.
- **DoD.**
  1. `si-harness sweep --gate G10` writes `gates/G10/ignition_sweep.json`. Its axes
     are **`IDLE_PER_HOUR` alone** — the four `IGN_WEIGHT_*` are TEMPERAMENT draws and
     no sweep may touch them (**OD-87**; `check_sweep_never_touches_temperament` fails
     the build if an enumeration ever lists one). The artifact is either a published
     curve whose winner replaces its placeholder and changes the run's constant version
     tags, or a **refusal in writing** with its `od_question` — and it is the refusal
     today, because the declared range is an open bound and the enumeration rule is
     still unruled (**OD-83**, and 05 §4's step 1). A refusal is a satisfied DoD *item*,
     not a green sweep: the legs that cite the artifact stay UNMEASURED.
  2. `si-harness run --gate G10 --variant V0` and V2/V3 ⇒ pass; asserts: an idle
     tick fires only with quiet senses + empty agenda; ≤ 1 idle thought per tick;
     zero rows, zero stamps, zero bake changes (V2 comparison, with the stamp arena
     graded **structurally** per **OD-88**: same record count and byte length, values
     free to move, rows and edges byte-strict); the monologue never enters the owner
     lane; an empty-agenda run with ignition disabled differs in exactly the documented
     fields; the decay-deficit rehearsal feedback is measured end-to-end with **no new
     constant** (**OD-89**); and V0 vs V3 on each named transcript leaves an
     **identical state hash** — that identity is the pass condition (**OD-91**), while
     the older "over-representation" wording is routed to **OD-92** and never satisfied
     by manufacturing a difference. Every report carries the variant id + graph digest
     pair (**OD-93**), and a replay re-executes **at the recorded variant**.
  3. `cargo test -p si-walk -p si-session` ⇒ exit 0 with idle tests.
- **Exit gates.** G10, C7 (`check_idle_no_commit` re-run).
- **Commit.** `B7: idle thought — engineering/04 §19, 08 §2`

### B8 — Sleep

- **Implements.** 10 §7–§8 (triggers, four passes, reports, laws), 04 §28
  (Z.1–Z.5), §26 law 21–23, §24 (sleep in the hash for free); 06 §3.4, §5.1
  (budget split); 05 §18 (G13); 01 §13 (counters).
- **Creates.** `si-sleep`: `due()` (counter + freeze only), P1 REPLAY
  (new-since-last-sleep contexts, newest kept, oldest walked), P2 INTEGRATION
  (read-only while promotion is off), P3 GLOBAL RESCALE (the only weight writer:
  `w' = f + (w−f)·d`, `d = DIM_BASE + (1−DIM_BASE)·recency`), P4 CALM
  (terminal view, no writes); step 12b with the interrupt-at-pass-boundary rule;
  `sleep.jsonl` and the core-block counters.
- **DoD.**
  1. `cargo test -p si-sleep` ⇒ exit 0; asserts: no `commit`/`append_row`/`age`
     symbol is reachable; P3 is the only weight writer and cannot brighten or
     zero a live edge; floors are constants, not columns; P4's counts have no
     consumer; each sleep walk advances `t` and is an ordinary thought; abort at
     a pass boundary is `Ok` and leaves the mask.
  2. `si-harness sweep --gate G13` writes `gates/G13/sleep_sweep.json` and
     `gates/G13/dimming_sweep.json`; then `run --gate G13 --variant V0` and V9 ⇒
     all legs green; `sleep.jsonl` has one line per pass per sleep.
  3. A freeze-first sleep: checkpoint written after the passes; resume does not
     repeat it (`last_slept_at_t`).
- **Exit gates.** G13, C8.
- **Commit.** `B8: sleep — engineering/10 §7–§8, 04 §28, 06 §3.4`

### B9 — Grimm, the gated supporter

- **Implements.** 02 §8; 04 §23 (two-layer speak gate upstream of sense);
  docs/21; 05 §17 (G12); 00 §7 (process topology), §3 (emotion words).
- **Creates.** `si-supporter`: the seam (stdin/stdout frames, `source: grimm`),
  speak gate layer 1 (taught-lexicon membership ⇒ structural muteness at t=0) and
  layer 2 (single-row restatement with `ref_row_id`), rejection before any sense
  event; rejected emission costs 0 state bytes and one log line.
- **DoD.**
  1. `cargo test -p si-supporter` ⇒ exit 0; asserts: a line with a token outside
     the taught lexicon is rejected before decode; a claim not restating one row
     is rejected; the crate imports no affect and reads no `a`, charge, bake or
     median.
  2. `si-harness run --gate G12 --variant V4` and V5 ⇒ (a)–(g) green; rates
     UNMEASURED until B12; V4/V5 hashes byte-identical to V0.
  3. `si-harness check --stage B9` ⇒ exit 0 with C8 + `check_membrane`,
     `check_no_affective_setter`, `check_no_stage_on_supporter`,
     `check_emotion_words` (supporter scope).
- **Exit gates.** G12, C9.
- **Commit.** `B9: supporter — engineering/02 §8, 04 §23`

### B14 — Arc substrate + crystallize (G15)

- **Implements.** [12 §1](12-v2-arc.md)/§2.6/§3/§4.1 (5a, 7a)/§5 **§29**,
  [02 §11](02-module-contracts.md) (steps 5a/7a, laws 27–29),
  [04 §4 note 7](04-algorithms.md), §5.5, §29, §24 X.5;
  [01 §2](01-data-schemas.md) (shelf 9 registered-unused), §4.3, §10, §11;
  [05 §2](05-gate-harness.md) V10/V13, §20's first two arc legs, §23.1; docs/22.
- **Creates.** The three packages with their **final** dependency edges
  (`si-silt`/`si-skills` as stubs that return empty), `si-core::quant`'s use of the
  existing `mul_q12`, the frame scratch (`CoLight`, `Ghost`, `RoleGraph`) inside
  `si-walk`, `si-crystallize`'s four rules + cold-frame rule + candidate table,
  `crystallize.jsonl`, the 22 registry rows, `CONSTS_VERSION = 0x0002`, the
  re-pinned `EMPTY_STATE_MD5_SEED_ZERO`, `CRATE_NAMES: [&str; 16]`,
  `launch_constants`' `Species | Ruling` filter + an arm per new row,
  `gap_kind` 5/6 + `origin` 2, `NAMELESS`.
- **DoD.**
  1. `cargo test -p si-core -p si-walk -p si-crystallize` ⇒ exit 0; asserts: the
     three new packages exist with **exactly** the edges of 12 §1.1 (a graph
     snapshot test), the frame scratch is constructed and dropped inside one frame,
     the four rules fire in order, and a primed frame increments no recurrence.
  2. `cargo test -p si-session` ⇒ the re-pinned empty-state hash matches at
     `--seed 0` (this is the commit that moves it, so an unre-pinned literal is a
     red here and nowhere else).
  3. `si-harness check --stage B14` ⇒ exit 0 with C0–C9 + `check_crystallize_no_commit`,
     `check_ghost_is_frame_scoped`; `check_silt_*` **disabled and listed** (their
     subject is a stub); every §6.3-moved check green at 16 names.
  4. `si-harness run --gate G15 --variant V0` then `--variant V10` ⇒ legs
     (a)(b)(d)(e)(f) green, (c) green as the frame-trace pair, and **the arc-off
     `questions_filtered_digest` equal to the arc-on one** (05 §23 defines the digest
     and forbids pretending a whole `state_hash.txt` can match when an ask was
     opened), with the `questions`-class exclusion named in `gate_report.json`, the
     interner asserted identical, and the row-count difference consisting only of
     kind-5/`origin = 2` rows.
     `gates/G15/recurrence_sweep.json` written with `status: refused`;
     `gates/G15/nomination_log.jsonl` present. Any leg needing a swept magnitude
     runs at its box `lo` and prints `floor`.
  5. `cargo metadata` names 16 packages; `git diff` shows **no** change under
     `docs/` (Phase 1 is closed at this commit).
- **Exit gates.** G15, C14. **Red ⇒ 3 fix attempts, then `failure_report.md` +
  STOP** (§6's stop rule; the likely walls are named in [12 §9](12-v2-arc.md)).
- **Commit.** `B14: crystallize + arc substrate — engineering/12 §1/§3/§5 §29, 02 §11, docs/22`

### B15 — Silt (G16)

- **Implements.** [12 §5 §30](12-v2-arc.md), [04 §30](04-algorithms.md) + §19
  S.1/S.3's fifth term + §2 note 11, [02 §15](02-module-contracts.md),
  [03 §5.2.1](03-constants-registry.md)'s five `SILT_*`/`LAMBDA_SILT` rows,
  [05 §23.2](05-gate-harness.md), V11, docs/23.
- **Creates.** `si-silt` (`tick`/`deposit`/`level`/`view`, `BTreeMap` tables keyed
  by `(conversation, point)`), step 5a's call, step 12's `&silt_view` argument, the
  `primed` tag's producer, `silt.jsonl`.
- **DoD.**
  1. `cargo test -p si-silt` ⇒ exit 0; asserts: the decay consumes
     `Clock::recency` and defines no curve of its own; drop-at-`THETA_SILT` sets 0
     and removes the entry; deposit clamps at `SILT_LEVEL_MAX`; the crate imports
     `si-core` only.
  2. `si-harness run --gate G16 --variant V0/V11` ⇒ (a)(b)(d)(e)(f) green; (d)
     asserts **strict last-touch drop order** and an empty table at `T_cold`, after
     which the sampler equals §19's lattice-only sampler exactly; (e) asserts the
     crystallize candidate table byte-identical with silt on and off.
  3. `check_silt_writes_nothing`, `check_silt_ignition_only`, `check_one_clock`
     (widened), `check_idle_no_commit` all green from `--stage B15`.
  4. `G16/silt_sweep.json` = `status: refused`; the armed term is `lo + grid` and
     the report prints that it is a **wiring** leg, not a warmth measurement.
- **Exit gates.** G16, C15.
- **Commit.** `B15: silt — engineering/12 §5 §30, 02 §11, docs/23`

### B16 — Episodic traversal (G17)

- **Implements.** [12 §5 §31](12-v2-arc.md), [04 §31](04-algorithms.md) + §4 note 7
  + §5.5, [02 §17](02-module-contracts.md), V12, [05 §23.3](05-gate-harness.md),
  docs/24. **No new crate.**
- **Creates.** `si-read::event_view_index` → `Vec<OriginTug>` (a `si-core` schema
  type), step 5b, the round-0 re-light path in `si-walk`, `mindlog.json`'s
  `event_views{}` block.
- **DoD.**
  1. `cargo test -p si-read -p si-walk` ⇒ exit 0; asserts: `event_id` **is** the
     sense row's `y`; the index is rebuilt at frame load and holds no handle;
     re-lights fire in `(event_id, point)` order, capped at `RELIGHT_MAX`; with the
     ruling row `off` the vector is empty and the walk's activations are
     bit-identical to B13's recorded values.
  2. `si-harness run --gate G17 --variant V0/V12` ⇒ (a)(c)(d)(e)(f) green; (d)
     graded **structurally** (stamp-arena record count + byte length unchanged after
     N re-lights, values free), with `OPEN-24 off` printed as the reason.
  3. `check_prov_link_not_stored` (shelf 11's `row_schema_version` still `0`),
     `check_relight_writes_nothing` green; `G17/prov_sweep.json` = refused;
     `G17/relight_log.jsonl` present.
- **Exit gates.** G17, C16.
- **Commit.** `B16: episodic — engineering/12 §5 §31, 04 §4/§31, docs/24`

### B17 — Skills (G18)

- **Implements.** [12 §2.2/§2.3/§5 §32](12-v2-arc.md), [04 §32](04-algorithms.md),
  [02 §16](02-module-contracts.md), [01 §2/§4.2/§6](01-data-schemas.md), V10's
  second half, [05 §23.4](05-gate-harness.md), docs/25.
- **Creates.** Shelf 9 **live** (`RESERVED_SHELF_COUNT` 3 → 2), the `SKILL` body
  reading, `si-skills` (roles, template interning, the two bars, forwarding),
  `NAMELESS` in the renderer's refusal path, the composition preference, the
  forwarding rule, `skills.jsonl`, step 7b.
- **DoD.**
  1. `cargo test -p si-skills -p si-store` ⇒ exit 0; asserts: generality **before**
     recurrence; a sub-`K_FILLER` shape is `forwarded` and its frames stay visible
     to §29's fold; the `SKILL` body's reserved bytes are zero; `arousal_at_ask`
     is `255` on every skill row; retirement is by supersede (I4 — no erase).
  2. `cargo run -p si-cli -- registry` (or the shelf unit test) ⇒ eleven entries,
     **two** reserved-empty, `"skills"` unchanged in the name list.
  3. `si-harness run --gate G18 --variant V0/V10` ⇒ (a)(b)(c)(d)(e)(f) green, and
     installed-rows-sitting-unread (`SKILL_COMPOSITION = off`) change no byte.
  4. `check_skills_no_commit`, `check_registry_size`, `check_string_constants`
     green; `G18/shape_sweep.json` = refused; `G18/skill_log.jsonl` present.
- **Exit gates.** G18, C17.
- **Commit.** `B17: skills — engineering/12 §2/§5 §32, 01 §2/§6, docs/25`

### B18 — World, N voices, delegated authority (G19)

- **Implements.** [12 §2.6/§4.2/§5 §33](12-v2-arc.md), [04 §33](04-algorithms.md)
  + §12 step 0b + §15 row 5 + §23 W.6, [02 §17](02-module-contracts.md),
  [01 §9](01-data-schemas.md)'s transport byte, V13,
  [05 §23.5](05-gate-harness.md), docs/26. **No new crate.**
- **Creates.** `si-core::world` (`SpeakerClass`, `class()`), the `grant` row type
  (declared, owner-lane-only writer), the tier arm at step 11, the lane rule at
  step 4, suppress-only windows at step 12, the `SIro` frame body's
  `speaker_identity u64le` + `conversation u32le` with the container **version**
  bump and the resume-mismatch refusal, the scenario grammar's optional
  `identity <u64>` declaration + `voice <id>` step prefix.
- **DoD.**
  1. `cargo test -p si-core -p si-learn` ⇒ exit 0; asserts: `class` is a total pure
     function of the three arguments and returns `Stranger` for every identity not
     owner/grimm/granted; `grants` empty ⇒ never `Trusted`; the tier arm fires
     **before** steps 1–10 and writes zero bytes; a stranger's `context` row still
     appends; `origin = 2` asks match only in the owner lane.
  2. `cargo test -p si-session` ⇒ a container written by B18's reader resumes; a
     pre-B18 container is **refused with a named version mismatch**, not silently
     upgraded — and the state hash is byte-identical to the same run before this
     commit (transport bytes are not state).
  3. `si-harness run --gate G19 --variant V0/V13` ⇒ (a)–(h) green, with **the
     refusal printed as the pass** on (c) and `OD-135` named in `observed`;
     (g) shows no idle tick caused by a window opening; (h) reuses §23's filter.
  4. `check_speaker_class_derived`, `check_windows_suppress_only`,
     `check_nomination_owner_lane`, `check_candidate_unanswerable` (grown) green;
     `G19/server_transcript.txt` (hashed), `G19/grant_trace.json`,
     `G19/lane_log.jsonl` present. **No `/trust` parser, no adapter, no grant
     issued** — OD-136 stays a seat, and the report says so.
- **Exit gates.** G19, C18.
- **Commit.** `B18: world — engineering/12 §5 §33, 02 §12, docs/26`

### B10 — Genome pipeline and the ENGINE loader

- **Implements.** 07 §1–§13 (stages M.0–M.11, canonical bytes, point identity,
  kind table, weights, density gates); 00 §6 (config guard); 04 §25 W.1–W.5
  (live-first two-cursor merge); 05 §11 (G6), §10 (G5c); 01 §6 (frozen records).
- **Creates.** `si-store::build` behind `engine` + `rayon`: manifest → fetch
  (pinned, hash-checked) → parse → key → align → emit → sort → count → index →
  seal (`md-5`) → verify (A1–A12) → publish (atomic); `SIgen` reader and the
  loader bodies with the runtime guard; the two-cursor walk over frozen + live.
- **DoD.**
  1. `cargo test -p si-store --features engine` ⇒ exit 0; asserts: canonical sort
     is byte-lexicographic; A1–A12 on the fixture; a mutated byte after sealing
     fails A10; re-running the same manifest reproduces the same tag; an
     `engine`-off binary returns `ConfigRefusal` before forming a path (handle
     audit via a `--trace-opens` log); the build tool opens no lattice/run/gate
     path (A11).
  2. `si-harness run --gate G6 --variant V7 --config engine …` on the pinned
     fixture ⇒ (a)(b)(c)(d.2) green; (d.1) UNMEASURED in the report.
  3. G5c: a CONFIG SI run with the `SIgen` on disk opens zero frozen handles.
- **Exit gates.** G6 (fixture), G5(c), C10.
- **Commit.** `B10: genome build — engineering/07, 04 §25, 00 §6`

### B11 — Viewer and the five panels

- **Implements.** 08 §1–§9 (three laws, transport, 21 frame kinds, five panels,
  forbidden widgets); 05 §20 (`check_gui_observer_only`,
  `check_no_viewer_state_write`, `check_headless_run`).
- **Creates.** `si-vulkan`: winit window + `ash` swapchain (Vulkan feature), a
  glyph-atlas CHAT, EMOTIONS (E1–E6), PREVIEW (layout from `SIst`), ORGANS
  (O1–O12 incl. sleep and temperament readouts), VRAM/PERF; `si-cli view`
  (process B) reading the run directory only; replay mode reuses the same panels.
- **DoD.**
  1. `cargo test -p si-vulkan` ⇒ exit 0; asserts: the crate's only writable path
     is `sense_inbox.siro`; no control-crate import; the frame parser handles
     kinds 0–20 and treats 18 as reserved.
  2. `si-cli view --session runs/<id> --exit-after-frames 120` ⇒ exit 0, writes
     `viewer.log`; the session's `state_hash.txt` is byte-identical before/after;
     `check_headless_run` (a build+run with no viewer) is byte-identical to V0.
  3. `si-harness check --stage B11` ⇒ exit 0 with C10 +
     `check_gui_observer_only`, `check_no_viewer_state_write`.
  4. Visual inspection: **not performed** unless requested (R7).
- **Exit gates.** GUI smoke + three checks.
- **Commit.** `B11: viewer — engineering/08 §1–§9`

### B12 — Sweeps and the variant matrix

- **Implements.** 05 §2 (V0–V9), §4 (ten sweep runners, objectives), §4.1 (seed
  panel, whole-box validation), §4.2 (dimming distribution); 03 §1 (a sweep's
  artifact chooses a value and its version tag); 10 §5 (panel ≥ 16 seeds).
- **Creates.** The remaining sweep runners (G6, G11 ×2, G12, G14); one `sweep`
  subcommand over all ten runners, re-running the five already exercised at
  B5–B8 through the same code path; V1–V9 production; `--strict-consts`; sweep
  artifacts under `gates/<gate>/`.
- **DoD.**
  1. Every sweep runner writes its artifact with the grid, the objective, the
     winner and the version tag; a re-run reproduces the winner byte-for-byte.
  2. Every swept value replaces its placeholder in a generated
     `consts/swept.rs` with the artifact's sha256; `--strict-consts` passes; a
     hand edit without an artifact fails `check_registry_size`.
  3. **The panel re-validates the box, not a point.** G2, G9, G10 and G13 are
     re-run at every `seed_panel` member and at both box corners (05 §4.1) and
     stay green; G11's composition and median sweeps and G14's panel report are
     produced here and G11/G14 are full; `unresolved_legs` holds only
     measurement legs (G6.d.1, G12 rates).
  4. V8's twin: LAW rows and guard bounds identical across seeds; only
     TEMPERAMENT rows differ; the seed panel's term-id union equals each member's.
- **Exit gates.** G11, G14 (full), G2/G9/G10/G13 panel re-validation, V-matrix.
- **Commit.** `B12: sweeps + variants — engineering/05 §2/§4, 03 §1`

### B13 — Global acceptance

- **DoD — the run itself.**
  1. `cargo test --workspace --all-features` ⇒ exit 0.
  2. `si-harness check --all` ⇒ exit 0 with all **46** checks (the arc's union;
     **52** once B20 lands — B13 re-runs at B20's DoD 6, it does not pre-empt it).
  3. For every gate in G1…G14: its reference run(s) and required variants exist
     under `gates/`, each report `pass: true`, and every measurement leg is
     either pass or listed `UNMEASURED` with its directory named.
  4. Same seed + same transcript: V0 vs V6 replay ⇒ `cmp` of `state_hash.txt`
     byte-identical at tolerance 0, plus identical activation traces.
  5. `si-harness run` with the viewer absent and with it killed mid-run ⇒ hash
     identical to V0 (`check_headless_run`).
  6. `cargo metadata` ⇒ the 16 packages (**17** from B20); `cargo tree` ⇒ no
     forbidden edge.
  7. `tree_sha256.txt` written over `src/`, `Cargo.*`, `harness/`.
  8. Final report (chat) lists: every gate with its artifact path, the OD leans
     implemented (grep `OD-` in `src/`), every placeholder/swept value with its
     artifact, and anything left red with its `failure_report.md`.
- **Commit.** `B13: global acceptance — engineering/05 §21`

### B20 — The tongue: a drawn voice inside the citation *(G20; last, by build
order)*

- **Implements.** [13 in full](13-tongue.md) — §1 (the crate), §3 (the 8 registry
  rows, `Class::Tongue`, `CONSTS_VERSION 0x0003`), §4.1 (tick step **9a**), §4.2
  ([02 §18](02-module-contracts.md)'s contract), §5 ([04 §34](04-algorithms.md)
  AF.1–AF.4), §6 ([05 §24](05-gate-harness.md)'s eleven legs, V14, the six checks,
  the refusing `phrase` runner), §7 ([06 §15](06-resource-budget.md)'s cost), docs/27.
  **One new crate** (`si-tongue`, the seventeenth) and one new `si-core` module.
- **Creates.** `crates/si-core/src/tongue.rs` (`Style`, `derive_style`,
  `render_seed`, `style_order_is_closed`, `TONGUE_DOMAIN`), `crates/si-tongue/**`
  (`Cargo.toml` with deps `si-core` + `si-walk` only, `lib.rs`, `License`), the
  workspace member + the `tongue` feature in [00 §4](00-stack-workspace.md)'s
  default list, the **one** call site at
  [`crates/si-session/src/lib.rs:2002`](../crates/si-session/src/lib.rs#L2002) with
  the license builder beside it, `ArcConfig { tongue, phrase_max_steps }` and two
  `launch_constants` arms (its guard text **untouched**), `harness/scenarios/g20_tongue.txt`,
  `crates/si-tongue/tests/realize.rs`, `Stage::B20` **last** in `ALL` (19 → 20).
- **DoD.**
  1. `cargo build --workspace` and `cargo test --workspace` ⇒ exit 0, **including
     the three re-pinned `EMPTY_STATE_MD5_SEED_ZERO` tests in the same commit as the
     `0x0003` bump** (`si-core/tests/kernel.rs`, `si-session/tests/seed.rs`,
     `si-session/tests/wake.rs`) — a bump and its goldens in two commits is a build
     that is red in the history.
  2. `cargo test -p si-tongue -p si-core` ⇒ `derive_style` is stable across two
     calls and differs from `derive_temperament`'s draw sequence; `Style` is 10
     bytes; the forced-failure table of [13 §6.6](13-tongue.md) runs as unit tests
     (no-candidates, unlicensed, slot-lost, budget-spent, empty-license ⇒ `None`;
     replay-identity ⇒ the same bytes).
  3. `si-harness check --all` ⇒ exit 0 with all **52** checks, and
     `check_registry_size` prints **103** rows / "18 TEMPERAMENT draws".
  4. `si-harness run --gate G20 --variant V0` **and** `--variant V14` ⇒ (a)–(k)
     green, the refusal-bearing legs passing **on their refusal** with the OD named
     in `observed`; `si-harness sweep --gate G20` writes `status: refused` and its
     `axes` name no style row.
  5. **G20(j)** as a string proof: `v0_strings.json` shows every `EV_OUTPUT`
     surface equal, byte for byte, to one cited row's own `surface`.
  6. **The battery:** every pre-tongue gate G1–G19 re-run green on the
     17-crate / `0x0003` build, and every recorded control **re-signed by
     re-running it** (**OD-91**) — nothing edited.
  7. Nothing committed under `gates/*/` except `gates/deps_lock.md5`, which a new
     workspace member **forces** re-publishing — a `[[package]]` row lands in
     `Cargo.lock` even though no external dependency does (**OD-185**; OD-42's
     allowlist is untouched, and B4/B14/B15/B17 each did the same);
     `scripts/` and `tree_sha256.txt` stay untracked.
- **Exit gates.** G20, C20 — and, because of the bump, **every** gate again.
- **Commit.** `B20: tongue — engineering/13, 02 §18, 04 §34, 05 §24, docs/27`

---

## 5. Gate-to-stage matrix

A gate's **first green** is the stage where every leg can run, which for a gate
whose values decide emission, cadence or shape is the stage that runs its sweep
(D5). Before that stage the gate is in its earlier stage's `disabled` list; at
that stage it must be green (or, for measurement legs, `UNMEASURED` in the
report). The **full** column is the stage where the gate is re-run under the
final variant matrix and, where it owns swept values, across the seed panel and
both box corners (05 §4.1).

| gate | legs | first green | full | variants |
|------|------|-------------|------|----------|
| G1 WRITE DISCIPLINE | a–u | B5 | B12 (G1.u) | V0 |
| G2 PARAPHRASE + WALK | a–l | B5 (sweep-backed) | B12 (panel) | V0 + sweep |
| G3 SUPERSESSION | a–f | B5 | B12 | V0 |
| G4 CONTRADICTION | a–g | B5 | B12 | V0 |
| G5 DETERMINISM + NO-PRIOR | a–h | B13 | B13 | V0/V6/V7, V1 |
| G6 BASELINE INTEGRITY | a–d | B10 | B10 (d.1 UNMEASURED) | V7 |
| G7 RE-PARSE | a–e | B5 | B5 | V0 |
| G8 CURIOSITY LOOP | a–r | B5 (sweep-backed) | B5 | V0 + budget sweep |
| G9 AROUSAL + EMOTIONS | a–k | B6 (sweep-backed) | B12 (panel) | V0/V1 + sweep |
| G10 IDLE THOUGHT | a–j | B7 (sweep-backed) | B12 (panel) | V0/V2/V3 |
| G11 STATE MEMORY + COMPOSITION | a–p | B5/B6 | **B12** (two sweeps) | V0 + two sweeps |
| G12 GRIMM ISOLATION | a–g | B9 | **B12** (rates) | V4/V5/V0 |
| G13 SLEEP | a–n | B8 (sweep-backed) | B12 (panel) | V0/V9 + two sweeps |
| G14 TEMPERAMENT | a–k | B5 (c,f) → B12 | **B12** | V0/V8 + panel |
| G15 CRYSTALLIZE | a–f | B14 | **B14** | V0/V10 |
| G16 SILT | a, a′–f | B15 | **B15** | V0/V11 |
| G17 EPISODIC | a–f | B16 | **B16** | V0/V12 |
| G18 SKILLS | a–f | B17 | **B17** | V0/V10 |
| G19 WORLD | a–h | B18 | **B18** | V0/V13 |
| G20 TONGUE | a–k | B20 | **B20** | V0/V14 + one refused sweep |

**Why the arc's five families have one stage in both columns.** The `full` column
exists for gates whose legs read a **swept magnitude**: those must wait for B12's
panel pass and both box corners (05 §4.1). G15–G19 own no such leg — their four
sweep runners publish `status: refused` while **OD-83** is unrulled (05 §23.6), and
every leg that needs a number runs at its box's `lo` and names itself a floor. So
there is nothing to defer: first green *is* full, and what B12/B13 add for these
five is the wider static-check union and the variant matrix over the same
transcripts — never a new magnitude. **A `gates/G15/recurrence_sweep.json` that
reports a winner is a red gate**, not an improvement.

**Static checks by stage.** B0: the C0 list in §4. B1 adds `check_config_guard`,
`check_bake_is_the_exception`, `check_no_mood_field`. B2 adds
`check_signal_kinds`, `check_bake_no_event_kind`, `check_drive_doors` (surface),
`check_one_clock`. B3 adds `check_idle_no_commit`, `check_drive_doors` (sites).
B4 adds `check_teach_literals`, `check_teach_left_side`,
`check_no_teach_vocabulary`, `check_string_constants`,
`check_hop_cap_is_claim_side`, `check_candidate_unanswerable`.
B5 adds `check_bake_write_once`, `check_compose_writes_nothing`,
`check_curriculum_is_inbox_only`, `check_no_seed_control_surface`.
B6 adds `check_audit_edges`, `check_emotion_words`.
B8 adds `check_sleep_writes_nothing`, `check_sleep_never_brightens`,
`check_calm_view_is_terminal`.
B9 adds `check_membrane`, `check_no_affective_setter`, `check_no_stage_on_supporter`.
B10 adds `check_config_guard` (engine body), `check_no_json_readback`.
B11 adds `check_gui_observer_only`, `check_no_viewer_state_write`,
`check_headless_run`.
**B14** adds `check_crystallize_no_commit`, `check_ghost_is_frame_scoped` and grows
`check_candidate_unanswerable` to ask kinds 5–6. **B15** adds
`check_silt_writes_nothing`, `check_silt_ignition_only` and widens `check_one_clock`
to `si-silt` (which must still find one curve). **B16** adds
`check_prov_link_not_stored`, `check_relight_writes_nothing`. **B17** adds
`check_skills_no_commit`, `check_nomination_owner_lane` and makes
`check_registry_size` assert two reserved shelves. **B18** adds
`check_speaker_class_derived`, `check_windows_suppress_only`. Those are **ten new
ids**, so the union is **46**, and 46 is what B12/B13 run with `--all` — the arc's
legs sit at the end of `CHECK_IDS` in registry order, and `enable_stage`'s
`<= stage` prefix rule (05 §21) is why B14–B18 are declared between B9 and B10.
**B20** adds `check_tongue_closed`, `check_tongue_is_leaf`,
`check_tongue_writes_nothing`, `check_tongue_reads_no_a`,
`check_style_never_installed`, `check_tongue_license_only`
([05 §20.1](05-gate-harness.md)) — **six** more, so the union is **52** and 52 is
what B20's `--all` runs. They are appended after the arc's ten for the same reason:
`Stage::B20` is last in `ALL`, so every earlier stage's prefix is unchanged and B13's
46-check acceptance keeps its meaning.

---

## 6. The stop rule, and what a wall is

**A wall is not a compile error.** A borrow-checker fight, a wrong offset, a
failed unit test are ordinary work and do not count against the three attempts of
a gate. A wall is one of:

1. **Spec contradiction** — two sections of `docs/` + `engineering/` cannot both
   be satisfied by one build, and one of them must yield.
2. **Undefined value that a sweep owns** — a leg needs a number no artifact has,
   and using a placeholder would make the leg pass *because of* the placeholder.
3. **Missing external artifact** — a source, license, or file the design requires
   and this environment does not contain (R5's fixture is the pre-committed
   resolution only for the genome; anything else is a wall).
4. **A gate red three times** after three distinct fixes, each re-run on the same
   leg, with the third observed failure unchanged in kind.

On a wall: write `failure_report.md` at the workspace root with exactly these
fields and **stop**:

```
# FAILURE REPORT
- gate / leg:        <id>
- stage:             B<n>
- attempts:          1..3, one line each: command, observed, fix tried
- spec sections:     <file §n> that the build cannot satisfy together
- contradiction:     one paragraph, quoting both sentences
- what would unblock: the OD question, and the reading the build would take if ruled
- tree state:        last green commit <sha>; working tree <dirty/clean>
```

No workaround is committed. No gate is reported green. No spec file is edited.
The run stops; the owner rules.

---

## 7. Global acceptance (the definition of done for the job)

The job's own three sentences, made checkable, plus the owner's fourth sentence
for this batch:

| job sentence | command / artifact |
|--------------|-------------------|
| "full gate suite green" | §4 B13 DoD 3: every `gates/**/gate_report.json` has `pass: true`, `criteria` covers every asserted leg, and `unresolved_legs` contains every measurement leg with a directory named — **over all twenty families, G1–G20**, not the fourteen that shipped before the arc |
| "same birth seed + same transcript = identical state hash" | B13 DoD 4: `cmp -s gates/G5_.../a/state_hash.txt gates/G5_.../b/state_hash.txt` against a V0/V6 pair; `REPLAY_TOLERANCE = 0`; plus V8's *different* pair differing only per §5's twin law — and **V14's pair differing by nothing at all**, because the voice writes no hashed item (**G20(a)**, [13 §0](13-tongue.md) fact 5) |
| "delivered crates match the workspace map" | B13 DoD 6: `cargo metadata` names = **00 §2's seventeen** (thirteen through B13 of the pre-arc plan, `si-crystallize`/`si-silt`/`si-skills` from B14, `si-tongue` from B20); `cargo tree` has no forbidden edge — including every row of [12 §1.1](12-v2-arc.md)'s exclusion table: the three readers reach only core/store, and `si-walk`/`si-learn`/`si-front`/`si-read` reach none of them — **plus `si-tongue ↛ si-store`**, which is [00 §2](00-stack-workspace.md) win 8 and the only structural proof of **OD-181**; R1's two-binary row |
| "nothing we already made breaks" (the owner's own words for this batch) | B13 DoD 7: every **pre-arc** gate G1–G14 is re-run green on the arc-on build, and for each of V10–V13 the arc-off run's `state_hash.txt` is **byte-equal** to its arc-on control for V11/V12 — and for V10 equal on 05 §23's `questions_filtered_digest` with an identical interner, since its control legitimately differs on one shelf of one row kind (05 §23's leg (b), B7's ruling: equality is the pass, a difference is a red). Plus the two substrate re-baselines in their single stage: `CONSTS_VERSION = 0x0002` with the three `EMPTY_STATE_MD5_SEED_ZERO` literals re-pinned, and `CRATE_NAMES: [&str; 16]`. **And the tongue's own half:** `CONSTS_VERSION = 0x0003` with those same three literals re-pinned **in the bump's commit**, `CRATE_NAMES: [&str; 17]`, G1–G19 re-run green on that build, and **G20(j)**'s string proof — every `EV_OUTPUT` surface at V0 still byte-equal to the cited row's own `surface`, which is the leg that makes the round safe to ship (**OD-178**) |

A green suite with an empty `unresolved_legs` on a measurement gate is a
**failure**, not a pass (D1). A green suite whose build identity omits a constant
version tag or the birth seed is a **failure** (05 §1). A hash that matches only
at a non-zero tolerance is a **failure** (05 §10). And a `gates/G1x/*_sweep.json`
that names a **winner** while OD-83 is unrulled — or a G19(c) that reports a
believed row instead of `Refused{SpeakerNotRepresentable}` — is a **failure**:
those three legs pass by refusing, and a refusal that turned into a result means
the build decided something §9 says it may not.


---

## 8. Execution rules

1. **Git.** `git init` once, before B0. One commit per stage, only after its exit
   gates are green. Commit messages follow the subject column in §3 verbatim.
   `.gitignore` covers `target/`, `runs/`, `gates/*/` (the root `gates/README.md`
   is tracked), and temporary files. `Cargo.lock` is committed (binary workspace).
2. **Read-only spec.** From the B0 commit on, `docs/**` and `engineering/**` are
   not edited. A needed change is a wall (§6).
3. **No invented values.** Every constant is a registry row; swept rows take
   OD-86 placeholders until their sweep stage (B5–B8 for G2/G8/G9/G10/G13, B12
   for the rest); **the arc's sixteen swept rows are the exception that is not a
   placeholder problem — their four runners publish `status: refused` until OD-83
   is ruled, their shipped defaults stay `UNMEASURED_PLACEHOLDER`, and a leg that
   needs one of them runs at its box's `lo` and prints `floor`** (05 §23.6);
   `IGN_WEIGHT_SILT` alone ships a *declared* 0, because unarmed is a value and not
   a guess (**OD-108**). No literal number appears in `src/` outside `consts/`
   (asserted by a check).
4. **No code without a section.** Every module's doc comment names the
   engineering section it implements. That is the one comment class this build
   requires, because it is how the final report can cite coverage.
5. **OD leans.** Every place the build implements an OD lean is marked with the
   OD id in the code's module comment and recorded in the final report; the build
   never presents a lean as a ruling.
6. **One GPU job at a time.** The viewer's smoke run is the only GPU process;
   no benchmark runs concurrently with it. Nothing is rebuilt while a run holds a
   binary.
7. **Honest measurement.** No wall-clock, size or rate is quoted from anywhere
   except a `runs/` or `gates/` JSON produced by this build; everything else is
   UNMEASURED.
8. **Stop at a wall, not at fatigue.** The only legitimate stop before B13 is
   §6's failure report.

---

## 9. What this plan refuses to decide

- Any swept value, its winner, or its objective (03 §1, OD-86, B12's runners).
- Any gate leg's predicate, threshold or scenario (05 is the owner of those; the
  build reads them).
- Any content word, taught fact, or corpus row (R-03, R-10; scenarios are
  selected, never rewritten — 05 §1).
- Any change to the design's organs, doors, signals, labels, shelves or crate
  laws (docs/ is law; 00 §2 is the build's copy of it).
- Whether the owner accepts OD-41…OD-86. The build implements the leans and says
  so; a ruling replaces the lean and re-runs the affected gates.

**And, for the arc specifically** — the fourteen items [12 §11](12-v2-arc.md)
enumerates are refused here too, in the same words, so that a builder reading only
this file still cannot decide one of them locally. The three that most often get
"fixed" by a well-meaning hand: **the OD-83 sweep enumeration** (four arc runners
publish `status: refused` and no build stage may turn a refusal into a winner),
**OD-135** (a granted teacher's belief write is refused with zero bytes because
`prov_speaker` has no honest value — inventing one is a schema decision this plan
does not own), and **OD-136** (no `/trust` parser, no Discord transport, no grant
minted by code; the scenario's owner line is the only grant that exists tonight).
The arc's own OD range is **OD-97…OD-136**, and a leg that needs one of them to be
ruled differently than the lean is a §6 wall, not a §8 discretion.
