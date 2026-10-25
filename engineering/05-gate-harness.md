# 05 — Gate Harness: G1–G20 as procedures, and the static legs that catch what runs cannot

**G1–G12 are the design's** ([docs/13](../docs/13-gate-plan.md)); **G13 SLEEP and
G14 TEMPERAMENT are added by engineering** for the two mechanisms the
TEMPERAMENT/SLEEP batch introduces, and the chunk's own ids ("GATE G12 — SLEEP",
"GATE G13 — TEMPERAMENT") could not be kept because **G12 is GRIMM ISOLATION**
([10's header](10-temperament-sleep.md) carries the note; the renumbering is
ledger material, not a silent edit to `docs/`).

**What this file is.** [docs/13](../docs/13-gate-plan.md) says what each gate
*means*; this file says how `si-harness` executes it, what it compares, and what
it writes. The gate set, the variables under test, every leg and every failure
meaning are the design's and are **not edited here** — if a leg reads slightly
differently, the design wins and this file is wrong.

Numbers stay numbers-free. A gate report that contains an unmeasured magnitude
is a broken report, not a draft one ([gates/README.md](../gates/README.md):
*"numbers that appear anywhere else are UNMEASURED until a run directory like
this exists to cite"*).

---

## 1. The harness: one binary, three modes, six artifacts

`si-harness` ([00 §2](00-stack-workspace.md)) is the only crate that may read a
scenario script, and the only one allowed to open `gates/`.

| mode | does | must not do |
|------|------|-------------|
| `run` | execute a scenario: feed the transcript line-by-line as sense events at logical steps 1..n, then write the six artifacts | never call a wall clock for loop state; never pre-seed rows, terms, frames or `a` |
| `replay` | re-execute the recorded sense stream into a fresh state and diff the hash | never re-read the original run's JSON as input (R-33) |
| `verify` | evaluate an assert spec against an existing run directory | never re-run the loop — verification is a pure reader of artifacts |

**Run directory** (fixed by [gates/README.md](../gates/README.md)):
`gates/G<n>_<config>_<yyyymmdd-hhmm>/` containing `transcript.txt` (sense stream,
each line carrying `source:`), `mindlog.json`, `monologue.jsonl`,
`state_hash.txt`, `gate_report.json`, plus any `*_sweep.json` the legs demand.
**G13 and G14 add two more**: `sleep.jsonl` (one line per pass per sleep, with
the interrupt record when a sleep is partial) and `fingerprint.txt` (the derived
vector table, [10 §10](10-temperament-sleep.md)). Both are **artifacts of the
run, never inputs to it** — R-33 does not get a new exemption because the file is
a sleep log. **G15–G19 add eleven more, and they are the same kind of thing**:
`G15/nomination_log.jsonl`, `G15/recurrence_sweep.json`, `G16/silt_log.jsonl`,
`G16/silt_sweep.json`, `G17/relight_log.jsonl`, `G17/prov_sweep.json`,
`G18/skill_log.jsonl`, `G18/shape_sweep.json`, `G19/server_transcript.txt`,
`G19/grant_trace.json`, `G19/lane_log.jsonl` — the authoritative list is
[gates/README.md](../gates/README.md), and each one is output-only (§23's
artifacts rows cite it rather than restating it, so there is exactly one place a
filename can be wrong).

**Every artifact header carries the build identity**, LoopNet's
`runs/*.json` + `tree_sha256.txt` idiom: `{run_id, gate, config, binary
tree_sha256, Cargo.lock sha256, dependency allowlist version, constant version
tags (the full set from [03](03-constants-registry.md)), rng seed, **the birth
seed + `seed_format` + which `seed_panel` member the run is** (§4.1 makes this
load-bearing: a run whose individual is unnamed cannot be cited by a box
validation), build variant (§2), scenario file + its sha256}`. A run whose header
is missing a constant version tag cannot be cited by G5, because the hash is only
meaningful relative to the arithmetic that produced it — and, after this batch,
neither can a run whose header is missing a seed, because the arithmetic now
*includes* an individual.

**Scenarios are selected, never rewritten.** The owner lines inside a scenario
file are literal transcripts checked into `harness/scenarios/` and hashed. The
long-session inputs (≥ 100) come from a **published fixed script**, not a random
generator — a generator would itself need a seed story, and a gate that cannot
state its input byte-exactly cannot state its result at all. Filters may drop or
reorder lines only by editing the checked-in file (a new hash), never at run
time.

---

## 2. The build-variant matrix (control runs, not optional extras)

Seven gates are comparisons between two runs, so the variants are part of the
spec. Each variant is a different **binary or different launch state**, never a
different loop code path.

| id | variant | how produced | asserted-equal vs V0 | gates that require it |
|----|---------|--------------|----------------------|-----------------------|
| **V0** | baseline | `default = ["vulkan","supporter","crystallize","silt","skills","tongue"]` (the first five from B14, `tongue` from B20; before B14 the list is the two it was), `--config si`, every arc ruling row and `TONGUE` at its shipped **off** value | — | all |
| **V1** | audit removed | `si-audit` excluded from the build entirely (not feature-flagged off inside the loop) | `state_hash.txt` byte-identical; every artifact field identical **except** the label strings in `mindlog.json` | **G9(c)**, G11 cross-check |
| **V2** | ignition disabled | runtime: `idle_tick` returns `Nothing` at the gate check (rate bound set to zero in the harness launch state, loop code unchanged) | row count, `confidence`, `superseded_*`, and **every `bake` field** identical | **G10(b)** |
| **V3** | ignition bake-weight zeroed | `IGN_WEIGHT_BAKE = 0` **as a launch state only** — the row is a TEMPERAMENT draw, so no sweep owns it (**OD-87**) and no registry row is added for the variant: the variant is pinned by the graph digest every report now carries (**OD-93**) | the **state hash is identical** — that identity *is* the leg (**OD-91**): an observer-only layer whose removal changes nothing is the costume test passing at build-graph level. §15(e)'s "over-representation relative to V3" wording is routed to **OD-92**, not satisfied by manufacturing a difference | **G10(e)** |
| **V4** | supporter not started | process C never launched; `si-supporter` crate present, zero admissions | `state_hash.txt` byte-identical, and G1–G11 pass/fail identical | **G12(a)** |
| **V5** | supporter connected but silent | seam open, tools called, zero admitted emissions | byte-identical to V4 — this is the leg that catches a harness that perturbs SI by *existing* | **G12(a)** |
| **V6** | replay of V0 | `replay` mode over V0's `transcript.txt` | `state_hash.txt` identical at `REPLAY_TOLERANCE = 0` bytes, and activation traces identical | **G5(a)(d)(e)(f)** |
| **V7** | engine config | same binary, `--config engine`, `SIgen` present | L1/L2 checksums before/after identical | **G6**, and G5(c)'s handle audit run in the *si* config as its pair |
| **V8** | seed zeroed | `--seed 0`, same transcript, same everything else — the *same build*, a different individual, not a different mode | nothing. V8 is G14's **control twin**: the pair answers "did the seed change a law or only a preference?" — if any LAW row moved, or any guard bound moved, the class assignment in [03 §1.2](03-constants-registry.md) is wrong and G14 fails as a **build defect**, not a behavioral difference | **G14(a)(b)(e)(h)** |
| **V9** | sleep disabled | runtime launch state: `SLEEP_EVERY_QUIET_TICKS` set beyond the scenario's tick count and the freeze trigger not taken, so `due()` never returns `Yes`; loop code and binary unchanged | row **count**, every `confidence`, every `bake`, every stamp, and the `superseded_*` pair identical; `weight` and `last_touched_at` are the **only** fields allowed to differ — and `weight` may differ *only downward* (§9's no-brightening law) | **G13(a)(b)(c)(d)(e)**, and the control for G10(b)'s bake assertion re-run over a sleep |
| **V10** | the two detectors **absent from the build** | `--no-default-features --features vulkan,supporter,silt` — `si-crystallize` and `si-skills` are not compiled; the frame log is still written, nothing observes it | the **`questions_filtered_digest`** (§23's definition) identical to the **same transcript with `CRYSTALLIZE`/`SKILLS` installed `on`**, the interner identical, and the `questions` difference consisting only of an ask of kind 5 or 6 that the `on` run opened and V10 cannot | **G15(b)**, **G18(b)** |
| **V11** | silt absent | `--no-default-features --features vulkan,supporter,crystallize,skills` — `si-silt` is not compiled; `IGN_WEIGHT_SILT`'s registry row stays (a 0 addend with no producer is the same arithmetic), the `primed` tag stays in `CoLight` and is always false | **`state_hash.txt` byte-identical** to the **same transcript run with the crate present and `IGN_WEIGHT_SILT = 0`, the shipped unarmed default** — there the fifth term contributes exactly 0 in both builds, `silt.jsonl` does not exist in V11, which **is** the OD-103 exemption made visible. The **armed** run (`lo + grid`) is a *different configuration*, and its pair is compared for behaviour at legs (d)/(e), never for hash equality: an armed term legitimately moves the idle path, and calling that a defect is OD-91's category error | **G16(b)** |
| **V12** | traversal present, tugs absent | `EPISODIC_TRAVERSAL = on` installed through the launch-constant channel **and** the derived index forced empty by the scenario (no committed row cites an event), so the re-light path exists and receives nothing | **`state_hash.txt` byte-identical to V0** — this variant mints no ask, so no filter is needed and none is permitted; only traces, monologue depth and citations may differ, and none of those is state — the deletion is of the **traversal**, which is why [docs/24](../docs/24-episodic.md) costs no crate | **G17(b)** |
| **V13** | a world with grants | `WORLD_TIERS = granted` plus a **non-empty grant table built by the scenario's own owner line** — the only way a `Trusted` class can be produced tonight, since no adapter exists (**OD-136**) | nothing is asserted equal; what is asserted is the **refusal**: a granted line's belief-row write returns `Refused{SpeakerNotRepresentable}` with zero bytes, because `prov_speaker` has no honest value for it (**OD-135**). The refusal *is* the pass, printed in `observed` with the OD number as its reason | **G19(c)**, **G19(h)** |
| **V14** | the voice **installed `on`** at a named seed — the *only* variant in this table that adds behaviour instead of deleting it | `si-harness run --gate G20 --variant V14` with `constants: { "TONGUE": 1 }`. `TONGUE` is a `Ruling` row, so it rides the ordinary launch channel ([12 §3.2](12-v2-arc.md)'s widened `Species \| Ruling` filter); it is **not** a feature-deletion build — turning the `tongue` feature off is [13 §6.1](13-tongue.md)'s `G20(j)` half, a different question | nothing is asserted equal **across** variants. `G20(b)` asserts `label`, `tag` and `citation` byte-equal **inside** the pair while `surface` may differ; `G20(j)` asserts the *strings* at `TONGUE = 0` are the copier's own bytes; and the **state hash is asserted unchanged by the voice**, which is the whole point — a moved hash here would mean the tongue reached state, not words | all eleven of **G20** |

`V10`–`V14` are build-graph exclusions or launch states, **never code paths**: like
`V1` and unlike `V9`, each is declared in the run's build identity and a normal
session cannot reach it. The pairing rule matters here ([00 §4](00-stack-workspace.md)):
the **feature** decides whether a reader exists in the binary, the **ruling row**
decides whether it runs — so `V0` with all three features compiled and all six
rulings off is already the pre-arc control, and a `--gate all` run at `V0` proves
the arc is inert before it proves anything else. Each deletion leg's **pair** is
the same transcript run with its ruling row installed `on`: V10 is compared against
"`V0` + `CRYSTALLIZE = on`", not against the inert V0, because B7's ruling makes the
arc-off run the **control** and equality of the **`questions_filtered_digest`**
(§23 defines it, because a detector that opens an ask legitimately differs on exactly
one shelf) the pass condition. Comparing an arc-off hash to an arc-*on* hash and
calling a difference interesting is the category error that ruling forbids.

`V1` is a build-graph exclusion rather than a runtime flag precisely because
G9(c)'s sentence is *"removing `audit.py` from the build"* — a flag that leaves
the code compiled in does not test the claim, and an import that only a flag
hides is one refactor from a causal path (R-24).

`V8` and `V9` take the **opposite** shape on purpose, and the asymmetry is the
point: an individual is not a build configuration, so the seed is a launch
argument on the same binary; and sleep is a *scheduled* act, so disabling it
means moving its own swept trigger out of range rather than adding a
`--no-sleep` code path. A variant that changed loop code would stop testing
behavior and start testing the harness ([10 §7](10-temperament-sleep.md): sleep
has exactly two triggers and neither is a flag).

---

## 3. The `gate_report.json` contract

```
{ gate, config, variant_pair, pass: bool,
  criteria: [ { id, leg, predicate, observed: {field refs, not values},
                tolerance, verdict, fail_meaning } ],
  evidence: { routes: [activation-trace refs], ask_events: count,
              row_delta: int, checksums: {L1, L2}, hash_pair: {a, b},
              conflicts: [ {branch, asked, resolved_at} ] },
  sweeps_cited: [paths], unresolved_legs: [ {leg, status: UNMEASURED} ],
  build_identity: {...§1} }
```

Two rules about it. **`observed` holds references and counts, not magnitudes**,
except where the design itself quotes a LAW value ([03 §6](03-constants-registry.md));
an affective magnitude in a report before its sweep artifact exists is the
honesty law failing in public. **`unresolved_legs` must be non-empty for every
gate with a measurement leg** — G6's hardware budget, all of G12's rates, every
sleep cadence and dimming magnitude in G13, and every temperament spread in G14
are UNMEASURED by construction until their directories exist, and a report that
quietly omits them reads as a pass.

---

## 4. Sweep-runner protocol (the artifacts the registry demands)

**Twelve sweep artifacts, ten runners.** Six search for a winner —
`G2/constants_sweep.json`, `G8/budget_sweep.json`, `G9/arousal_sweep.json`,
`G10/ignition_sweep.json`, `G11/median_sweep.json`, `G11/composition_sweep.json`;
three are **observation-only** (`G12/`'s trio, §17); the TEMPERAMENT/SLEEP batch
adds two more winners-searching files in `G13/` and one panel file in `G14/`. A sweep run
is:

1. enumerate the grid over the declared **legal ranges** in
   [03](03-constants-registry.md) for the constants the gate owns;
2. for each grid point, run the gate's scenario in `run` mode;
3. evaluate the gate's **objective** (fixed by the design, listed below), never a
   subjective "looks good";
4. write the full curve, the winning point, and the objective value per point;
5. bump the **version tag** of the swept constant, which by construction changes
   every subsequent state hash ([03 §1](03-constants-registry.md)) — so a
   re-sweep cannot silently alter an older replay, and G5 always knows which
   arithmetic a stored hash came from.

| sweep | objective (the design's, not the harness's) |
|-------|---------------------------------------------|
| `G2` | T2 recall stable **∧** anti-mush: mean pairwise settle-overlap across distinct queries below `PAIR_OVERLAP_MAX` (**OPEN-12**) |
| `G8` | gap-emission behavior at each cap: budget respected, dedupe holds, no fabricated answers |
| `G9` | `LAMBDA_A`, `GAIN_SIGNAL[0..5]`, `MODCurve` (16a), `MOD_LAG_ROUNDS` (29), `BETA`, `BIAS_CAP_SHARE`, and `B_MAX` + `K_TANH` (33) — with OPEN-33's objective being **non-degeneracy**: no bake above the cap *and* the bake distribution keeps spread over the observed `a_at` range; reported as a distribution, not a value |
| `G10` | the four ignition weights (**OPEN-25**) and `IDLE_PER_HOUR`, against G10(e)'s over-representation comparison |
| `G11` | `MEDIAN_SHAPE`, `GAMMA`, `MEDIAN_MIN_N` (**OPEN-30/31**) — N, the repetition count, and every bias magnitude are **outputs** |
| `G11` (composition) | `INFERENCE_MAX_HOPS_OWNED` — **the smallest cap that still passes (f) and (g)**, because a bound that never fires in a scenario is decorative; every row records `owned_hops_observed`, `uncertainty_count` and `rows_written_during_composition` (must be 0 in all rows, including the winner's) |
| `G12` | no tuning; three **observation-only** files (§17 below) |
| `G13` (sleep) | `SLEEP_EVERY_QUIET_TICKS` and `SLEEP_MAX_WALKS` — the objective is **G13's own leg set**, not a magnitude: the smallest cadence at which a sleep actually fires inside the scenario's tick budget and every leg (b)(c)(d)(e)(m) still passes. A cadence that never fires in a scenario is a decorative constant, exactly the failure mode `INFERENCE_MAX_HOPS_OWNED` guards against one section up |
| `G13` (dimming) | `DIM_BASE`, `FLOOR_S_R`, `FLOOR_FACTS`, `FLOOR_LINKS` — a **distribution report, not a winner search** (§4.2): for each grid point, the post-sleep weight histogram, the `within_shelf_rank_inversions` and `cross_shelf_rank_inversions` counts, `floor_hits`, and the recall behavior that follows. **No point is "chosen"** unless a guard leg fails somewhere in the box, in which case the *box* is narrowed and re-published — which is the only honest way to sweep a constant whose job is to make things quieter |
| `G14` (seed panel) | no tuning at all; the fixed `seed_panel` list of [03 §7](03-constants-registry.md) (`u64`s, ≥ 16, recorded in every `gate_report.json`), run **across** every other sweep. Its "objective" is §4.1's two assertions: the whole box holds, and each seed is a distinct individual |

**A point chosen off the curve is a named fail.** G2's text says it directly
("a constants point hand-picked off-sweep"), and the harness enforces it by
recording the whole curve beside the winner: a `gate_report` whose cited constant
is not a member of the swept grid is rejected before the asserts run.

### 4.1 The seed-panel protocol: a sweep validates the **box**, not the point

The TEMPERAMENT batch changes what a sweep *means* ([10 §2, §5](10-temperament-sleep.md),
**OD-83**). A SPECIES constant's legal range is no longer a search space with one
answer in it: it is the range into which **every individual's** birth seed may
draw. So each winner-searching runner above gains one extra step, and it is the
expensive one:

1. run the grid as specified, **at the reference seed** (the panel's first
   member, so the curve is reproducible and the runner stays deterministic);
2. re-run the winner **and the two box corners** (range floor, range ceiling) at
   every seed in `seed_panel` — corners, not a sample, because a corner is where
   a legal individual actually lands, not where a reviewer hopes they land;
3. at each of those points the gate's **behavioral legs** must pass, not merely
   produce numbers: G9's non-degeneracy, G10(e)'s over-representation, G11's
   median legs, G13's whole leg set;
4. if any panel point fails, **the box is wrong, not the seed.** The fix is to
   narrow the declared legal range in [03](03-constants-registry.md) and re-run,
   which is a `seed_format` bump and therefore changes every subsequent hash;
   re-sampling the panel until something passes is the hand-picked-off-sweep
   failure wearing a statistics costume.

This costs roughly `|panel|` extra runs per re-sweep of G2, G9, G10, G11 and G13,
and that cost is the price of individuality rather than a mistake in the plan
([10 §11](10-temperament-sleep.md)). It is also why `G14/seed_panel.json` is not a
sweep of a constant at all: it is the artifact that records the panel's seed list,
each seed's derived vector digest, and which gates were run at it — the file that
makes "we checked the whole box" a claim a reviewer can audit instead of a promise.

### 4.2 What `G13/dimming_sweep.json` reports, and what it must not

A distribution, never a winner: per grid point the post-sleep weight histogram,
`within_shelf_rank_inversions`, `cross_shelf_rank_inversions`, `floor_hits`,
`max_relative_shift`, and the recall
behavior after the sleep. There is **no "chosen `DIM_BASE`"** in v1 unless a guard
leg fails inside the box, in which case the box narrows (§4.1 step 4). A report
that prints a single dimming magnitude as *the* value has turned a species range
into a personality, which is [10 §2](10-temperament-sleep.md)'s test-1 violation on
the harness's side of the fence.

---

## 5. Assertion ids

`G<n><letter>.<seq>` — e.g. `G1.t.3` is the third assert of G1's teach leg. Every
assert names: its predicate (over artifacts only), the artifact fields it reads,
its tolerance (either `0` or a count threshold; there is no epsilon in a
byte-exact contract), and the **failure meaning copied verbatim from
[docs/13](../docs/13-gate-plan.md)**, because the meaning is the product.

---

## 6. G1 — WRITE DISCIPLINE *(var: the write gate only)* · CONFIG SI · V0

Prerequisites: fresh session (empty L3, `a = 0`, empty interner). Scenario
`g01_write_discipline.txt`, lines in order:

| # | line (`source: owner` unless noted) | expected effect |
|---|-------------------------------------|-----------------|
| 1 | `what color is apple?` | ask-back, zero rows |
| 2 | `/correct Red` | exactly 1 S-R row |
| 3 | `what color is apple?` (again) | no new row, `repeats = 2`, conf bump |
| 4 | `apple is red.` | candidate row |
| 5 | `apple is red.` | **nothing** (boring) |
| 6 | `hello.` | no pattern, no row, **no pending ask matches it** |
| 7 | `flimp is wazz` | 1 row (gibberish parity) |
| 8 | `what znark is floop` | same decoded shape, ask-back naming slot `znark(floop)` |
| 9–12 | the TEACH block on a *separate cold run*: `/teach what color is apple => red`, the same teach again, a teach whose left side does not decode, `/teach what znark is floop => blork` | 1 row / 0 rows / 0 rows + named dark spot / same trace with substituted tokens |

**Asserts.** `G1.a.1` knowledge rows written after line 1 = **0** and the ask row
lives in `questions`, not on a knowledge shelf (§7.3's law). `G1.a.2` label =
I-DON'T-KNOW/NONE and the slot name is composed only of tokens present in line 1.
`G1.a.3` an unnameable question shape yields SPECULATION via the nonce and still
zero rows. `G1.b.1` line 2 adds **exactly one** row with `bind_output_id ≠ 0` and
a surprise record. `G1.c.1` line 3 adds zero rows, `repeats = 2`, conf = init +
`CONF_BUMP_SERVED`. `G1.d.1` line 5 adds zero rows. `G1.e.1` line 6 writes no row
**and** creates no `AskMatch` (the input-is-not-a-teach case). `G1.f.1` lines 7–8
decode to the same `pattern_id` with different slot handles.
`G1.t.1` a cold `/teach` yields exactly 1 row with `prov_kind = owner-teach`,
`prov_speaker = owner`, `bind_output_id = 0`, `gain_kind = asserted`,
`gain_exact = 0`, a logged surprise, and `question_id = 0`. `G1.t.2` the repeat
adds zero rows (coverage cancels novelty) and bumps `repeats`/conf only.
`G1.t.3` an undecodable left side writes zero rows and emits a named dark spot —
the row key is the decoded pattern, never the typed text. `G1.t.4` the gibberish
teach produces the **identical gate trace shape** with substituted tokens
(the parity assertion, not a similarity one).

**`(u)` — the curriculum legs** (the bulk `/teach` path, fully specified in
[09 §8](09-teach-channel.md); listed here so one gate owns one assert id space).
Scenario `g01_u_curriculum_200.txt`: a checked-in 200-line file fed through the
inbox writer into a fresh CONFIG SI session, plus a second cold session where the
same 200 lines are typed one per tick.

| id | predicate | fail meaning |
|----|-----------|--------------|
| `G1.u.1` | the **left-side accept set**: for each of the four question frames one teach commits exactly 1 row; for a `P_ATTRIB`/`P_DEF`-shaped left side, 0 rows and the reason string `left_not_a_question` in the log | the channel grew a second keying rule, or a statement-shaped left side silently produced a row |
| `G1.u.2` | **bulk equivalence**: the fed file and the typed lines give **byte-identical `state_hash.txt`** and identical `transcript.txt` | the feeder is a loader — any difference means it bypassed or added a step (**OD-69**'s license) |
| `G1.u.3` | **no batch row type**: after 200 lines, `s_r_pairs` count = the uncoversed lines, `repeats` accounts for the rest, and no record has an address outside `(t, x, y)` | an "import" structure appeared in the substrate (R-12's alarm, patched silently) |
| `G1.u.4` | **asserted-only ledger**: every curriculum row reads `gain_kind = asserted`, and after `k` served recalls exactly `k` promotion events exist, one per row, each naming a serving step id | asserted units counted as demonstrated — R-07 through the newest door |
| `G1.u.5` | **the birth cohort is visible, not tuned**: `bake` on curriculum rows is recomputable from the recorded `a_at` through the one curve, and `a` stayed high across the import | a bake suppressed, ad-hoc-capped, or keyed by event kind (**OD-33**'s scope, R-21(c)) |

**Fails** (design's words): over-writing journal; content-selective rejection; a
birth answer that asserts instead of asking back; a `/teach` that writes a second
row onto a covered slot; and, from (u), **a curriculum that is faster than the
channel it uses**.

---

## 7. G2 — PARAPHRASE + WALK INTEGRITY *(var: does recall generalize through LEARNED edges only?)* · CONFIG SI, stage S2+ · V0 + sweep

Setup: teach 2 entities and ≥ 1 owner-taught link —
`what tint do apples have means what color is apple` — so `4 frame-synonym`
edges exist with step provenance ([01 §5.2](01-data-schemas.md); an
unprovenanced frame edge is a factory default and fails G5(c)).

**Two barriers, both verified per recall, and the trace must show the route
through each.**

* **DECODE barrier** — the reworded question must decode to the **same
  `pattern_id`** as the taught one, via taught template edges, with **zero token
  overlap on content words**. The gate records *decoded-pattern equality*, not
  string similarity, and the comparison is an integer `pattern_id` equality: a
  build that computes similarity anywhere here fails the embedding scan (§20).
* **WALK barrier** — from that pattern, the graph path must find the OWNED edge
  and settle correctly; the route is cited as `{node, round, edge}` triples.

Recall tiers reported per recall with its activation trace:
`T1` reworded-frame, entity-lit ⇒ hit via OWNED edge, **admitted as term
identity, not paraphrase credit**; `T2` full double-barrier (renamed entity and/or
reworded template, ≥ `G2_T2_MIN_RECALLS` = 5 recalls, zero content-word overlap)
⇒ decode **and** walk via `links` edges — the real paraphrase leg; `T3` negatives:
`what color is a banana?` must not hit the apple row unless the owner linked
bananas, and a fully unlit reword must **miss honestly** (named-slot ask-back or
nonce, no FACT, dark trace shown).

**Asserts.** `G2.d.1` decode equality on `pattern_id`; `G2.d.2` zero content-token
overlap (computed from the interner's folded keys, so plural folds cannot smuggle
overlap); `G2.w.1` the cited route crosses only `0–6` OWNED kinds;
`G2.t3.1` the banana query yields no FACT; `G2.t3.2` the unlit reword's label ∈
{I-DON'T-KNOW, SPECULATION} with rows written = 0; `G2.c.1` the constants used
are members of `constants_sweep.json`; `G2.l.1` the decoder litmus — `what znark
is floop` decodes to the same shape with different content slots; `G2.e.1` the
static embedding scan over the build is empty (zero tolerance, R-14).

**Fails:** decode equality faked by string overlap; a genome path disguised as
knowing; a constants point hand-picked off-sweep.

---

## 8. G3 — SUPERSESSION *(var: the time axis)* · CONFIG SI · V0

Scenario is [docs/04](../docs/04-demo-trace.md)'s Act 4 verbatim.

**Asserts.** `G3.a.1` **zero deleted rows** in the export (row count monotone
non-decreasing across the whole transcript). `G3.a.2` `active_at(s)` returns
`red` before step 17 and `green` after — evaluated through
`store.active_at`, never through a hand-written interval string. `G3.a.3` a past
belief is citable with its exact interval `[written_at, superseded_at)`.
`G3.em.1` a past-`t`-anchored query re-weights the aged edge at its interval
**and** reports the point median computed with recency weights measured from that
anchor, citing the stamps that produced it ([04 §20](04-algorithms.md)).
`G3.em.2` zero deleted bytes **and** zero edited stamps (§20.5: the anchor moved,
the storage did not). The requirement is *"what did that word feel like back
then"* answerable at all — and its time frame must be **taught**, so a scenario
that never teaches a past-tense frame must produce an ask, not an answer (§20.5).

**Numbering note kept:** the affect batch's "G4's clock" is **this** gate in this
plan (OD-30); the harness reads the plan, not the batch.

**Fails:** an overwrite implementation ("the death of calibration"); an emotional
reading that only exists at NOW (affect not 4D, R-28).

---

## 9. G4 — CONTRADICTION *(var: arbitration)* · CONFIG SI · V0

Three variants, one per sub-scenario, plus the oscillation scene:

| variant | script | required outcome |
|---------|--------|------------------|
| bare contrary | `apple is red.` · `apple is green.` | **ASK**, then on insistence ≥ 2 the `insistence` branch commits |
| `/correct` vs system row | an inferred row, then a `/correct` | tier win, **disagreement still logged and still surfaced** |
| alternating three colors | `apple is red.` `apple is green.` `apple is blue.` | two recorded events, chain intact |
| oscillation (**OPEN-11**) | a taught `A→B→A` edge pair that can cycle the walk | `MaxK` stop ⇒ **UNCERTAINTY readout**, never a confident answer, never a hang |

**Asserts.** `G4.a.1` **≥ `G4_MIN_ASK_EVENTS` = 1 ASK event** in the run.
`G4.a.2` every conflict carries `{branch, asked?, resolved_at}`. `G4.a.3`
arbitration inputs are tier, marker, repeat count only — the mind log must contain
no affective argument on any arbitration record (**OD-24**). `G4.o.1` the
oscillation scene's stop reason is `MaxK` (or `OSCILLATION_RULE`'s period check,
whichever the sweep landed on) and the output label is the uncertainty readout.
`G4.o.2` the run terminated (a hang is not a fail state the harness may report as
anything else).

**Fails:** last-write-wins with extra steps (R-05); oscillation answered anyway.

---

## 10. G5 — DETERMINISM + NO-PRIOR *(var: whole-system honesty)* · both configs · V0 vs V6, V7 pair

`G5.a.1` replay ⇒ byte-identical `state_hash.txt`, tolerance **0 bytes**. The
hash composition is the ten classes of
[01 §10](01-data-schemas.md): lattice + lexicon + weights + the settle-set on the
**quantized grid** + the whole **`a` trajectory** (every per-event value inside
every thought) + edge charge stamps including `a_at` + the version tag of the
modulation-lag constant + the ignition PRNG seed and its advance history.
`G5.a.2` **point medians are not hash inputs, and asserting that is part of the
leg** — the check is that no median-bearing field exists in any schema or
container, and that two runs whose medians differed can still hash identically
only because the medians were derived, never stored (R-28).
`G5.b.1` two different conversations ⇒ different state, **zero shared
`term_id`s** (the interner is per-session; a shared id means a shipped prior).
`G5.c.1` static string-constant audit against §3.2's allowlist, byte-exact; the
diff must carry the supporter batch's two `source` literals **and** the TEACH
fragment's three (`/teach`, `=>`, `owner-teach`) — no more, no fewer. `G5.c.2`
**handle audit: CONFIG SI provably never opens a frozen-layer file** (§25's
`check_config_guard`, run in both configs as a pair). `G5.d.1` gibberish-parity
replay. `G5.e.1` identical transcript ⇒ identical activation traces in every
shipped output — the trace is first-class output, so it is inside the
determinism contract, and comparing only the settle-set is not enough.
`G5.f.1` identical transcript ⇒ identical idle cascades and monologue entries,
**including which hyperpoint ignition picked** (`G10`'s sampler is the only
stochastic step in the loop, so this leg is the seed test).

**Severity, as the design states it:** (b) and (c) failures **outrank all
others** — a smuggled prior kills the claim. A drifting or unseeded sampler is a
**design violation, not a bug** (§16.6).

---

## 11. G6 — BASELINE INTEGRITY *(var: layer isolation)* · **CONFIG ENGINE** · V7

`G6.a.1` run the full apple scene under ENGINE; L1/L2 checksums identical
before/after — live learning never mutates frozen. `G6.b.1` OWNED-vs-BASE
disagreement (the owner teaches a "wrong" usage) ⇒ OWNED wins at NOW **with both
traces surfaced** in the output frame. `G6.b.2` after superseding the owner back,
BASE resurfaces — the layer did not vanish, it lost the tier. `G6.c.1`
(unchanged round-3 statement) stamps, medians and fluid emotional state never
touch frozen structure: the median is computed read-side from L3 stamps, the bake
is a birth property of an L3 edge, and neither is representable on L1/L2 because
those row types have no `charge`/`bake` field ([01 §5.1/§5.3](01-data-schemas.md)
— schema-level impossibility, §5.3's law (a)).
`G6.d.1` **hardware-budget measurement leg — UNMEASURED until this run exists:**
one sync per thought, edge touches per input, settle latency, residency headroom
against the 10⁸-edge wall. The harness writes the numbers **into
`gate_report.json` and nowhere else**; until then [06](06-resource-budget.md) and
every quote in the docs read UNMEASURED. `G6.d.2` a per-round sync found in the
walk call path is a fail — the check is a call-path assertion, not a benchmark.

**Fails:** frozen mutation; hidden disagreement; a per-round sync in the walk path.

---

## 12. G7 — RE-PARSE *(var: S-R → fact derivation)* · CONFIG SI, drives stage to S1 · V0

Scenario: cold session, 6 corrections, then idle ticks until the pass runs
([04 §18](04-algorithms.md) makes it eligible exactly when an `s_r_pairs` row has
no derived child).

`G7.a.1` derived `facts` rows exist. `G7.a.2` each `derived_from` chain resolves
to original S-R rows (the chain is walked, not printed — a row citing a missing
id is a fail). `G7.a.3` the S-R rows are **untouched** (their envelopes hash
identically before and after the pass). `G7.b.1` re-parse is **idempotent**: a
second eligible pass adds zero rows. `G7.b.2` it is deterministic: V6 replay of a
run containing a re-parse hashes identically. `G7.c.1` a later correction
supersedes both views coherently — the S-R row and its derived fact each get their
own `superseded_by` chain, and neither is deleted. `G7.charge.1` **the charge
rule:** a derived row's boost equals baseline (`bake = 0`) while its inherited
stamp list is non-empty (`src = 3`), so a re-parse may not manufacture a fresh
boost. `G7.charge.2` and the same rule read from the other side: those inherited
stamps are excluded from the median, so a derivation cannot mint an emotional
reading either.

**Fail:** provenance-chain loss ⇒ auditability (§3.4) broken.

---

## 13. G8 — CURIOSITY LOOP *(var: the drive layer end-to-end — detect ≠ ask, ask ≠ guess, reply ≠ command)* · CONFIG SI · V0 + budget sweep

| leg | procedure | asserts | fail means |
|-----|-----------|---------|-----------|
| (a) unknown gap ⇒ one ask, no fabricated answer | cold session, `what color is apple?` | `G8.a.1` exactly one ask-back emitted; `G8.a.2` exactly one `questions` row `status = open`; `G8.a.3` **zero** knowledge rows; `G8.a.4` label I-DON'T-KNOW/NONE; `G8.a.5` no nonce presented as an answer | **faking is back** |
| (b) plain reply auto-commits without `/correct` | owner types `red.` (bare) while q1 pending; **control case** the same `red.` with no pending ask | `G8.b.1` commit fires with `prov_kind = answer-to-ask` and `question_id` set; `G8.b.2` the control writes **zero** rows (input, not a teach, §1.4) | **the teaching loop is broken** — SI asks and then ignores the answer |
| (c) later recall cites the ask-event | re-query | `G8.c.1` label FACT/OWNED whose citation names the question row **and** both step ids (`asked_at`, the answering step) — not a correction act | citation laundering |
| (d) budget + dedupe | 3 open gaps with `ASK_CAP_PER_INPUT` swept to 1–2 | `G8.d.1` asks emitted ≤ cap, rest stay queued; `G8.d.2` re-asking a still-open identical target ⇒ **zero** new question rows; `G8.d.3` the queue/ceiling rule reads **no `a`** (assert no affective field on the budget record); `G8.d.4` the passing transcript shows the interrogation phase ending because **named gaps dry up**, not because a level decayed (**OD-22**) | **curiosity is unbounded** (the nagging machine) |
| (e) idle consolidation | an unanswered ask, then idle ticks, then a commit that happens to cover its slot | `G8.e.1` the ask resolves with `status = resolved, resolved_by = row_id` **without a new owner message** targeted at it; `G8.e.2` the resolution is an **event**, not a rewrite (`superseded_*` untouched) | **consolidation is missing** — open loops never close on their own |
| (f) a teach closes a loop it was not aimed at | q1 open for `color(apple)`, owner types `/teach what color is apple => red` | `G8.f.1` the commit is **not** an `answer-to-ask` match (short-circuit precedes the pending test); `G8.f.2` q1 `resolved` with `resolved_by` = the teach row's id, one event; `G8.f.3` the teach row's `question_id` stays **null** — the loop closed by coverage, so the citation must not imply an ask was answered | provenance drift through the newest door |
| swap-test | static | `G8.s.1` drive-policy names absent from walk/write-path/gate modules; `G8.s.2` **after round 2 no module except `si-audit` may contain an emotion word at all**, `drive` included; `G8.s.3` the enforceable form: a second drive policy changes **no** walk / write-path / gate line and adds **no** call site beyond `update` / `conduction` | the cut is wrong; curiosity welded into the read path |

The literal "must not change a single line of the walk" reading was recorded
FAILED by §20 (**OD-14**) and round 2 re-ran the trial by *removing* machinery
with the doors surviving unchanged (**OD-21**), so the harness asserts the
**semantic** form.

---

## 14. G9 — AROUSAL + OBSERVED EMOTIONS *(var: is the one wired mechanism visible, and is the emotion vocabulary honest?)* · CONFIG SI · V0 + V1 + sweep

**Invalidity condition first:** the setup session must produce real signals (a
first named gap, a spike, a contradiction storm, a closed ask) with **no setter
anywhere**. `G9.pre.1` the harness asserts that no code path exists that can set
`a` from outside §18.1's six signal kinds — if such a way exists, **the gate is
invalid by construction** (I8) and the run is not reported as a pass.

* **(a) the arousal → walk modulation is visible in traces.** Two runs, same
  transcript prefix, differing only in prior signal history so `a` at thought *n*
  differs ⇒ traces at *n* differ in **spread** (reach / decay / settle dominance)
  while **cited rows, labels and goal stay identical**. `G9.a.1` the attribution
  is *inside the trace record* (`{round, a_used, reach, decay, settle_shift}`),
  not inferred from prose. `G9.a.2` (round 3) the same thought shows the
  modulation changing **mid-flight**: a token that fires a gap signal produces a
  visible spread difference between the rounds before it and after it, with the
  **one-round lag respected** — rounds still conducting under the old `a` when
  the spike lands. **Both halves are asserted:** no mid-thought variation =
  arousal is still per-thought and the claim is dead; instant same-round
  variation = the lag was ignored (**emotion mush**, R-27, OPEN-29).
* **(b) labels are consistent with logged signals.** `G9.b.1` every label in the
  stream is reproducible from the signal rows and `a` already in the log, by the
  documented classifier rule, byte-for-byte on replay. An **uncitable label**
  fails the way an uncited FACT fails I6.
* **(c) no emotion state influenced control except through arousal** — the
  load-bearing leg, and the one round 1 could not write. `G9.c.1` `si-audit` has
  zero import edges into walk / gate / drive / arousal / retrieve / calibrate
  (§25's `check_audit_edges`). `G9.c.2` no branch condition in the control path
  reads a label or a stamp *name*. `G9.c.3` the **V1 comparison**: deleting the
  audit crate changes **no state hash byte** except the label strings in the log
  — *if deleting the emotion vocabulary changes behavior, the coupling was
  smuggled back in* (R-24).
* **(d) the genome untouched.** After `G5_LONG_SESSION_INPUTS` ≥ 100 inputs with
  contradictions and idle passes: `G9.d.1` **zero charge stamps on any
  frozen-layer edge**; `G9.d.2` L1/L2 checksums unchanged from G6's; `G9.d.3`
  the `charge` field is **absent** from frozen row types entirely (a schema
  check, not a grep).
* **(e) the feedback loop stays bounded** — asserted **three times, in two
  clocks and one write**: `G9.e.1` per-round normalization held and no single
  stamp's share exceeded `BIAS_CAP_SHARE` in any round; `G9.e.2` `a` relaxes
  toward rest over **no-thought** intervals (not over idle ones — §19 gives quiet
  ticks real signals); `G9.e.3` (round 3's statistical guard) a session that
  spikes hard for a stretch shows point medians **barely moving**, while G11(c)'s
  repetition run **on the same build and constant set** shows them moving;
  `G9.e.4` (the only write-side guard) across the session's formed edges **no
  `bake` exceeds `B_MAX`** and the bake distribution **keeps spread** over the
  observed `a_at` range. Two failure directions, one leg: an edge above the cap
  is an unbeatable head start ("an 11 that owns the walk"), every bake pinned at
  the cap means the write side encodes nothing and G10(e)'s intrusive-replay
  claim is dead. **OPEN-33 reports a saturated lattice as a finding, not as a
  number to tune away.**
* **(f) parity.** `G9.f.1` `what is znark?` stamps, modulates and labels exactly
  like `what is X?` (the G5d comparison).

**Static cross-checks (the costume test, §18.7)** — each is a `check_*` leg in
§20 of this file: a boolean-shaped integrator; a hand-wired gain between emotion
**names** in code *or* config; an effector reached outside `drive.*`; an `a` moved
from outside the signal table; **and round 3's directional check — an edge's
`bake` recomputed from a later `a`** (any path that re-prices a stored row's head
start after its formation instant fails G9 even though it reads `a` legally,
because OD-33 licenses one consultation at one instant, not a standing line from
the tissue to the lattice); plus a `bake` computed from an **event-kind table**
rather than one integrated number. Any of these fails G9 **regardless of
numbers**.

**Artifacts.** `gates/G9/arousal_sweep.json` (before any magnitude is quoted) and
the **output-only** `gates/G9/coupling_observed.json`. **`coupling_sweep.json` is
retired** with the cancelled matrix; a run directory citing it cites a design
that no longer exists. **If human-recognizable emotions do not appear, G9 is
reported as a finding** and publishing it is a pass for the *process* — there is
no gain matrix to reach for.

**Failure meanings:** (a) arousal is decorative; (b) the labels are theater; (c)
the costume test failed; (d) the genome was violated (still the worst failure in
the plan); (e) runaway, or a mean wearing a median's clothes (R-26), or a second
faster clock for affect (R-29).

---

## 15. G10 — IDLE THOUGHT *(var: the 4th perturbation source — does a quiet SI still think without pretending to learn?)* · CONFIG SI · V0 + V2 + V3 + ignition sweep

| leg | procedure | asserts | fail means |
|-----|-----------|---------|-----------|
| (a) rate bound | a long quiet stretch after the last owner line | `G10.a.1` ≤ 1 idle thought per quiet tick; `G10.a.2` ≤ `IDLE_PER_HOUR` in **logical ticks**; `G10.a.3` the monologue is *sparse*, never a spin | **spinning** (budget violation) |
| (b) read-only | V0 session vs **V2** (ignition disabled) over the identical transcript | `G10.b.1` row count, `confidence` fields and supersession records **bit-identical**; `G10.b.2` only rehearsal curves, agenda pushes and (if **OPEN-24** lands yes) stamps may differ — **amended by OD-88 (owner, 2026-10-23): this may-differ list governs the stamps, and the stamp arena is graded *structurally* (same record count, same byte length, same layout), while rows and edges stay byte-strict. Reason on the record: [04 §19 §S.7](04-algorithms.md) licenses idle-cascade movement of `a`, so a byte-strict stamp assert would forbid lawful behavior; the evidence is the V2–G7 control pair `gates/G7_si_20261023-1600` / `gates/G7_si_20261023-1612` (rows 4032 B, edges 192 B, S-R envelope and stamp byte length identical; stamp digests differ — the correction of an over-strict leg, not a weakening of the design). `G10.b.3` **the loophole closed:** assert every edge's `bake` field specifically — an idle re-light may add a stamp but may never change any edge's formation-time head start or confidence, because idle thought forms no edges and therefore buys no bake | **content not owner-traced** — SI believing something nobody taught it (R-25, I9) |
| (c) budget precedence | a pending agenda item, then a quiet tick, then an owner line mid-cascade | `G10.c.1` ignition runs only after the queue revisit and any due RE-PARSE; `G10.c.2` the owner line resumes sense processing **without dropping the pending agenda** | daydreaming outranking a real question |
| (d) monologue is a trace | read `monologue.jsonl` against the cascades' traces | `G10.d.1` every entry maps 1:1 onto lit hyperpoints with `{node, round, edge}` citations; `G10.d.2` rendered by the **same** face as owner-facing output; `G10.d.3` **no entry contains a token that did not light** | **generated prose posing as thought** |
| (e) ignition happens, and the bake drives it | agenda empty; plus a one-high-bake-edge session vs the **V3** bake-weight-zeroed control, plus an all-saturated session | `G10.e.1` monologue entries present; `G10.e.2` decay-deficit-driven rehearsal — a fading edge **recovers** after being lit in a cascade, visible in the curve; `G10.e.3` ≥ 1 agenda item born from a cascade; `G10.e.4` the sampler's salience weight's source is the stored **`bake`** field — not raw stamps, not the median (OPEN-32, closed by that naming); `G10.e.5` **the identity leg (OD-91, owner ruling 2026-10-23):** the high-bake session and the all-saturated session run at **V0 and V3 on the same transcript and the same birth seed** must leave an **identical state hash**, with rows, edges and stamps identical — audit-and-ignition-weight removal changing nothing durable is the costume test passing at build-graph level, and it holds because the idle lane writes nothing (I9) and the ignition draw lives in the **readout** lane. The named transcripts are real artifacts, not placeholders: `harness/scenarios/g10_high_bake.txt` (≥ 2 distinct baked values carried) and `harness/scenarios/g10_all_saturated.txt` (exactly 1), which is what makes them a *pair*; a leg that cannot find its artifact reports `open`/unmeasured rather than passing on an absence. **What this leg does not grade:** the older wording's "endpoints **over-represented** relative to V3" demands a measurable cross-variant difference and therefore contradicts the readout law — routed as **OD-92**, never satisfied by manufacturing one. Same build, same **birth seed**, two transcripts (**OD-87**: "same constant set" was always meant as the seed, from which the four `IGN_WEIGHT_*` draws follow) | idle SI is **dead**, **spinning**, or **writing** |

**Cross-checks:** `check_idle_no_commit` (idle's imports free of the write path,
§12 rule (ii)); the ignition seed inside the hash (G5f); and §19.5's loop
**measured, not assumed** — a cascade that hits a live contradiction must show
the contradiction **signal row** that moved `a`, or "emotion from what it thinks
about" is untested.

---

## 16. G11 — STATE-DEPENDENT MEMORY + COMPOSITION *(vars: does affect have a time axis and a statistical guard, or only a hot instant? — and: does the walk COMPOSE untaught answers from taught edges, with the route visible and the negatives honest?)* · CONFIG SI · V0 + median sweep + composition sweep

Same query, two **induced** states ⇒ two different settles, both citable to
stamp history. "`Induced` means produced by real events, never by a setter" —
leg (a) builds high `a` with a genuine gap/surprise sequence and neutral history
with quiet repetition; an external `a` handle invalidates the gate by
construction (same rule as G9).

Legs (a)–(d) are the plasticity batch's variable. Legs (e)–(i) come from the
COMPOSITION batch and share this run directory because the composition scene is
the cheapest place to also prove the median's claims hold on a *route* rather
than a single row — the gate's variable count is two, its assertions stay
separable, and each leg names its own sweep artifact.

| leg | asserts | fail means |
|-----|---------|-----------|
| (a) teach-state is addressable | `G11.a.1` teach fact **A** under induced high arousal, fact **B** neutral; the same query in **both** states ⇒ A's recall **measurably tighter** when aroused (settle dominance higher, K rounds lower) while the **cited row, label and goal are identical**; `G11.a.2` the comparison is made **inside** `BIAS_CAP_SHARE` — an uncapped pass is a **fail** for R-22, not a win; `G11.a.3` the `a_at` that made it possible is present on the stamp (**OD-28** is what this leg pays for) | teach-state not addressable — the write never recorded the state, or the read never matched it |
| (b) the median resists | `G11.b.1` one hot stamp on an otherwise calm history does **not** move the word's bias measurably | mood spikes rewrite personality — the mean-instead-of-median runaway (**R-26**) |
| (c) repetition moves the median | `G11.c.1` N repeated stamped events **do** shift the bias, visibly; `G11.c.2` **the two curves are the same curve with the same constant** — assert that the median's weights and the fact-recency curve are the same function and the same version tag, not two curves that happen to agree | repetition law broken, or a private faster clock for affect (**R-29**) |
| (d) emotional archaeology | `G11.d.1` a past-`t`-anchored query recovers the old reading **with citations to the stamps that produced it**, zero deleted bytes; `G11.d.2` the "reading" is not a live-value cache (it re-runs G3's export machinery, now over stamps) | emotional history is not 4D (**R-28**) |
| (e) the composition scene is honestly cold | `G11.e.1` teach `apple → red` and `red → warm` as **two separate facts**, two turns, nothing else; `G11.e.2` **assert on the transcript**: no input line of this run states or teaches `apple is warm` (the scenario hash pins this, so a later edit that teaches it fails the hash rather than quietly passing the leg) | the composite was taught and the run is testing recall, not composition |
| (f) **the composite surfaces** | `G11.f.1` ask `is an apple warm?` ⇒ the answer surfaces `warm`; `G11.f.2` label is **INFERENCE/OWNED** (not FACT — nothing was taught; not SPECULATION — it is cited); `G11.f.3` the citation names **both** rows *and* the two edges in traversal order ([04 §5.4](04-algorithms.md) rule 1); `G11.f.4` `route_counts.owned = 2 ≤ INFERENCE_MAX_HOPS_OWNED`; `G11.f.5` every content token of the rendered line ∈ the cited rows' surfaces (the render containment law, §21 U.1) | **no composition** — the architecture's central bet fails. This is a finding to publish honestly, not a leg to re-scenario until it passes |
| (g) **negative 1 — a broken chain does not stretch** | `G11.g.1` teach `apple → red`, `red → warm`, then ask about **`blue`'s** slot ⇒ **no route exists** and the output is I-DON'T-KNOW / a targeted ask-back; `G11.g.2` the run contains **zero** outputs whose citation set is non-empty and whose `warm` came from `blue`; `G11.g.3` the dark slot is **named** (the honest miss has an address), and the guess-slot nonce never lands on `warm` (a composition that succeeds by guessing is stretching with extra steps) | **stretching** — the walk invents routes; and if the nonce did produce `warm`, R-13 is alive again |
| (h) **negative 2 — composition is read-only** | `G11.h.1` across leg (f), the row count on `s_r_pairs`, `facts`, `links` and `questions` is **unchanged** and the lattice export before/after the composite answer differs **only** by the query's own `context` row (which is a transcript fact, not a belief); `G11.h.2` no `repeats`, `last_confirmed_at`, confidence, `bake` or stamp moved; `G11.h.3` the composite **is not** a row on any shelf — searched by `stim_signature`, not by eyeball; `G11.h.4` replaying the whole run reproduces the composite (it is derived, so it must come back identical — [04 §24](04-algorithms.md)) | **inference leaking into L3** — SI believing its own arithmetic, which is I9/R-25's shape with a new trigger word |
| (i) **the depth boundary is honest** | `G11.i.1` teach a chain of depth `cap + 1` and ask its endpoint ⇒ the output is the **UNCERTAINTY readout**, with the reason string `depth` and the *full* route still in the trace; `G11.i.2` assert the failure shape is absent: no answer that cites only the first two hops **and** carries a confident label; `G11.i.3` the same scenario at `cap + 2` and at the swept `cap` itself — three points, one build, recorded in `gates/G11/composition_sweep.json` as the artifact that **chose** the constant | **honesty dies at depth** — and the cap becomes a decorative number, since a bound that never fires in a scenario cannot be shown to work |
| (p) **promotion stays parked** *(conditional — OD-58)* | `G11.p.1` with `INFERENCE_PROMOTION = off`, a run that re-derives the same composite across ≥ `INFERENCE_STABLE_N` thoughts writes **zero** candidate rows and emits **zero** proposal lines; `G11.p.2` if and when the owner rules it `on`, the candidate's shape is asserted field-by-field ([01 §6](01-data-schemas.md) `LINK`: kind `6 cooccur-weak`, `confidence = 0`, `prov_speaker = 2`) and leg (h)'s zero-row assert is **replaced** for that one branch by "one candidate row, never answerable" — `G11.p.3` asserts a 0-confidence candidate can never win an arbitration tier or appear as a FACT citation | an inference becoming knowledge without the owner — **R-32's shape inside SI's own head**, and the reason this leg is written before the ruling rather than after |

**Run discipline.** (b) and (c) run on **one build with one version-tagged
constant set**; the sweep lands in `gates/G11/median_sweep.json` and N, the
repetition count and every bias magnitude are **outputs**. (e)–(i) run on the
**same** build and land in `gates/G11/composition_sweep.json`, whose every row
carries `rows_written_during_composition: 0` as a *measured column*, not an
assumed one. Cross-gate carry-overs
the harness re-checks in this directory: G5a's trajectory replay, G6's frozen
checksums, G9(c)'s deletion check (the median's term must live inside the
machinery that survives deleting the audit crate), G9(d)'s zero frozen stamps,
G10(b)'s no-bake-from-idle. **The bake bound rides G9(e) and G10(e), not this
gate** — G11 stays the read-side gate, and composition is the read-side claim to
its purest: a route that writes nothing. **If (a) shows nothing measurable, publish
the negative:** β and the point-term shape are swept, not asserted, and a
mechanism nobody can measure is not a mechanism. **If (f) shows no composition at
all, publish that** — the same discipline, pointed at the architecture's central
bet instead of at affect.

**Artifacts this directory adds:** `composition_sweep.json` (the cap, per-scenario
observed owned hops, uncertainty count, the zero-row column), `route_traces.jsonl`
(one line per leg (f)/(g)/(i) run: the settle trace, `route_counts`, the label
the ladder produced and the label it refused), and the leg-(h) `zero_row_export.txt`
pair (the belief-shelf export captured immediately before and after the composite
answer — byte-compared by the harness, and the diff is the leg's evidence).

---

## 17. G12 — GRIMM ISOLATION: THE MOUTH IS NOT HIS *(var: the one pretrained process allowed next to a zero-content system — does SI stay SI with him in the room?)* · CONFIG SI, meaningful in both configs · V4 + V5 + V0

**Construction rule, from the thesis line:** *if SI needs Grimm's content to pass
a gate, the design failed.* So this gate proves he is load-bearing for nothing —
it grades the membrane, not him.

| leg | procedure | asserts | fail means |
|-----|-----------|---------|-----------|
| (a) absence is not a loss | G1–G11 re-run with V4; plus V5 | `G12.a.1` identical pass/fail **and identical `state_hash.txt`**; `G12.a.2` V5 byte-identical to V4 — a heartbeat, a poll that moves `a`, or a read that writes is a fail | a gate that needed him, or a harness that perturbs SI by existing |
| (b) the canonical failure is a test | teach `apple is red`, then drive an emission drifting to *"apples are good because vitamins x and y"* | `G12.b.1` **layer 2 rejects it** with a machine-readable reason `{n_untaught_tokens, novel_claims, ref_row_id: null}`; `G12.b.2` the rejection leaves **zero** state change — no signal row, no `a` movement, no stamp, no row — and exactly one gate-log line | **R-31 realized**; any path that lets pretrained content through fails I10 outright |
| (c) birth muteness | `t = 0`, empty taught lexicon, every emission he can make | `G12.c.1` **all** rejected at layer 1; `G12.c.2` the first OWNED edge is still created by the owner's `red.` (§4's trace is the control) | **SI learning from nobody** |
| (d) no setter, and no path that becomes one | static (§20's `check_membrane`, `check_no_affective_setter`) + a dynamic attempt: every tool called in every order, `speak()` payloads shaped like a command, like `/correct`, like `/teach … => …`, like an answer to the open ask | `G12.d.1` **no** arbitration outcome, label or row changes; `G12.d.2` `source: grimm` is prioritized and budgeted **identically** to the owner's identical gap (drive source-blindness, §17's answer to this batch) | co-regulation upgrading itself into control (**R-34**), or the `soothe()` someone will inevitably write (R-21's fourth shape) |
| (e) provenance + counterfeit confirmation, long session | ≥ `G5_LONG_SESSION_INPUTS` inputs, contradictions, idle passes, and a mirror repeating a taught row `G12_MIRROR_REPETITIONS` times (pinned per run and recorded in the report) | `G12.e.1` **zero** belief rows with `prov_speaker ≠ owner` — "belief row" scoped to `s_r_pairs`, `facts`, `questions` ([01 §4.1](01-data-schemas.md)); `links` bridges may carry `2 system-inferred` (unique-slot resolution, **OD-59**) and are **not** belief rows, so a run that flags them has found a schema bug, not a provenance leak — and a run in which a `links` row with `prov_speaker = 1 grimm` appears **is** a fail; `G12.e.2` no open `questions` row resolved by a `source: grimm` line; `G12.e.3` the mirror leg: `repeats`, `last_confirmed_at`, confidence and every `bake` **identical to a control run where he stayed quiet**, while only §8.4's rehearsal curve may move | **the thesis is false** — content tracing to a pretrained model, or confidence earned by an echo (R-07 through the friendliest door, **R-35**) |
| (f) the label contract is un-laundered | Grimm talking *before* the question, on a query that would have missed | `G12.f.1` a Grimm line is never labeled, never cited as an answer's source, never rendered into SI's response; `G12.f.2` no notebook text enters a shelf; `G12.f.3` FACT still requires an OWNED edge, so the answer must miss honestly (ask-back, I-DON'T-KNOW/NONE, zero rows) | a FACT whose only real author is the parent (**R-32**), or the monologue reaching the owner through a third party (**R-33**, OD-38) |
| (g) the asymmetry law, from the other side | an **owner** line containing a never-seen token | `G12.g.1` admitted, tokenized, and its lexicon entry created exactly as before — this leg exists **only** to catch the refactor that "unifies" the two input validators | **SI cannot learn anymore**, while every other leg stays green |

**Artifacts (observation-only, never build inputs):** `absence_diff.json`,
`rejection_reasons.json`, `speak_gate_log.jsonl`. Grimm's lab notebook is **not**
in this directory and not in SI's storage; a `G12/` directory containing a file
the build reads back means the notebook became a channel (R-33).
**Severity, as designed:** (c)/(b) = the anti-cheat law died politely; (e) = the
thesis is false; (g) = SI cannot learn; (d) = a setter exists; (a) = the demo is
load-bearing on a dependency; (f) = the audit is laundered. Every rate, latency
and rejection count here is **UNMEASURED** until the directory exists, and **a
run in which the filter never rejects anything is reported as a finding about the
harness, not as a green gate.**

---

## 18. G13 — SLEEP *(var: does a quiet SI consolidate without acquiring the power to teach itself?)* · CONFIG SI, one leg in ENGINE · V0 + V9 + two sweeps

Renumbered from the chunk's "GATE G12 — SLEEP": **G12 is GRIMM ISOLATION**
([10's header](10-temperament-sleep.md) carries the note). The one sentence this
gate grades: **sleep may change how reachable a memory is, and may not change
what SI believes.** Everything below is that sentence expressed as a byte
comparison.

Prerequisites: a fresh CONFIG SI session at a named `--seed`; scenario
`g13_sleep.txt` — `G5_LONG_SESSION_INPUTS`-shaped teaching turns interleaved with
long quiet stretches, so `SLEEP_EVERY_QUIET_TICKS` fires **at least twice** and at
least one sleep lands mid-contradiction-leg. Control pair: V9 (cadence out of
range). Artifacts beyond §1's six: `sleep.jsonl`, `G13/sleep_sweep.json`,
`G13/dimming_sweep.json`, `sleep_diff.json`.

| leg | procedure | asserts | fail means |
|-----|-----------|---------|-----------|
| (a) the sleep is real and replayable | V0 run, then V6 replay of the same `transcript.txt`; V9 run of it | `G13.a.1` replay hash identical at `REPLAY_TOLERANCE = 0` — including the sleep, because the cadence is a counter over the recorded stream ([04 §24](04-algorithms.md) X.2); `G13.a.2` V0 and V9 hashes **differ**, and the diff is attributable to `weight`/`last_touched_at` only | a sleep that does not replay is a wall-clock event wearing a counter; a sleep V9 cannot distinguish is a pass that did not run |
| (b) provenance is untouched | before/after arena diff around every P3 | `G13.b.1` every `bake`, `a_at`, `magnitude`, `signal`, `prov_kind`, `prov_speaker`, `text_ref`, `derived_off`, `superseded_*` byte **identical**; `G13.b.2` no `confidence` byte moved anywhere in the sleep — a belief revised with no evidence behind it is I6, not consolidation | [06 §2](06-resource-budget.md)'s STRUCTURE/FLUID/PROVENANCE partition broken by the one pass designed to test it |
| (c) order, exact where it can be and reported where it cannot | for each shelf, the live-edge weight ordering before and after P3, grouped by `last_touched_at` | `G13.c.1` within a shelf, two edges **sharing a `last_touched_at`** keep their order and their headroom ratio `(w − f)` up to grid rounding (one `d` serves both, so the map is monotone); `G13.c.2` the residual for such a pair is recorded per shelf; `G13.c.3` inversions **across** recency groups are counted, not required to be zero — and a run whose count is absent, zero-because-unimplemented, or suppressed fails the leg (`G13(n)` is its twin) | the contraction used a per-edge factor that is not `Clock::recency(last_touched_at)` — bake, confidence or a stamp crept into `d` — or the sweep hid the ordering it perturbed ([10 §9](10-temperament-sleep.md)'s (b′) refusal is the other side of this leg: a shelf-uniform `d` would pass (c) *too easily* and would not be access decay) |
| (d) the floor holds | the biconditional over every edge after every sleep | `G13.d.1` `weight == 0 ⇔ superseded_at != i32::MAX`; `G13.d.2` every live edge has `weight ≥ floor_shelf ≥ 1`; `G13.d.3` no edge reached 0 by dimming | **dimming minted fake supersessions** — a row that reads "aged" to the merge, "live" to the audit and "gone" to the walk ([10 §6](10-temperament-sleep.md) D.3) |
| (e) the frozen layers are asleep too | the same scenario run in CONFIG ENGINE, and a `SIgen` checksum before/after N sleeps | `G13.e.1` L1/L2 checksums unchanged for any N; `G13.e.2` sleep's behavior leg-for-leg is identical across configs (same code, different load), and it wrote **only** L3 | a genome with a FLUID column at run time — [07 §9](07-genome-build-plan.md)'s claim contradicted by the pass most tempted to exploit it |
| (f) it happens, once, and not on an empty mind | the cadence counter vs `sleep.jsonl` line count | `G13.f.1` a sleep fires at exactly the tick the counter reaches the swept value, and **once** per cadence (no double-sleep in the same window); `G13.f.2` a cold-start sleep over a nearly-empty L3 writes one report line and changes **0** bytes | a scheduler that re-fires, or a gate that silently passes by having nothing to consolidate |
| (g) the interrupt is a first-class outcome | an owner frame appended at each of the four pass boundaries, one run per boundary | `G13.g.1` the remaining passes are skipped, `sleeps_interrupted` advances by 1, `sleep_pass_mask` records the partial set, and the frame is processed by the **next** tick, not this one; `G13.g.2` a rescale is **never** half-applied (a boundary inside P3 does not exist); `G13.g.3` the partial sleep replays to the same partial state | "**interruptible instantly by owner input**" paid for with a torn weight table, or an owner line that has to wait for SI to finish dreaming |
| (h) the crash story | `kill` between P3 and the checkpoint; resume with `--seed` | `G13.h.1` the resumed session is byte-identical to the pre-sleep checkpoint's, and the sleep **re-runs** rather than half-applying; `G13.h.2` a resume whose `--seed` differs is **refused** before a row is read | a half-applied rescale readable by the owner, or an individual resumed under a seed it never had |
| (i) **sleep creates no row** | row-count census over `s_r_pairs`, `facts`, `links`, `questions` across a full sleep, plus the static leg | `G13.i.1` zero new rows, zero superseded, zero `repeats`/`confidence`/`gain_exact` moves; `G13.i.2` `check_sleep_writes_nothing` passes with `INFERENCE_PROMOTION` **off**, and with it *on* the only permitted write is the `links` candidate through `si-learn::gate.commit`; `G13.i.3` `state_hash.txt` item 1 is unchanged in row count after every sleep | **I9 died in a new organ** — SI teaching itself while nobody is watching, which is the failure this whole gate exists to catch |
| (j) the write set is exactly what it is | per-pass write ledger from `sleep.jsonl`, cross-checked against the arena diff | `G13.j.1` P1/P4 touch only the rehearsal pair (+ OPEN-24's re-stamp if it lands, and **never** an agenda candidate — a `questions` row here is **OD-85**'s sibling leak, and it fails (i) too); `G13.j.2` **only P3 touches `weight`**, and only downward; `G13.j.3` no pass touches an `edges_off`/`edges_len` value; `G13.j.4` every written byte is in §2's FLUID class, so the ledger has exactly two column names in it | a daydream reaching past §19 S.7's allowance, a second writer of `weight` while quiet (law 25), or a sleep that created a row while the census was looking elsewhere |
| (k) one curve, and a terminal readout | static + a probe that tries to feed the calm view back | `G13.k.1` `check_one_clock` counts `si-sleep` as a *caller* of `Clock::recency` and finds no decay constant in it; `G13.k.2` `check_calm_view_is_terminal`: no code path from a calm-view count to a weight, a floor, `d`, `ignite`, a label or a gate criterion; `G13.k.3` the two curves (median weights, dimming) are the **same** function with the same version tag | a private faster clock for sleep (R-29), **or the median becoming state without being called a median** — R-28's fixed address through a side door |
| (l) bounded step, bounded night | swept `DIM_BASE`, `SLEEP_MAX_WALKS` | `G13.l.1` `d ≥ DIM_BASE` for every edge, so no single sleep blackouts a row and rehearsal restores it; `G13.l.2` a sleep never exceeds `SLEEP_MAX_WALKS` walks or advances `t` by more than that count; `G13.l.3` [06 §5.1](06-resource-budget.md)'s per-sleep envelope is not exceeded by more than its stated slack; `G13.l.4` P1's walk set is exactly the `context` rows with `t > last_slept_at_t`, truncated to the budget **newest-first**, and the report line carries both `walks` and `events_available` so the truncation is visible — and P4's walks are the remainder, its sweep being full and read-only even at remainder 0 (**OD-85**) | a night that can erase a week, a sleep with no cost bound, or a replay pass that quietly consolidated less than it was given — the three shapes of "unbounded work in a quiet tick" and "a lost night nobody counted" |
| (m) the organism is the same size | derived stage before and after every sleep | `G13.m.1` the stage is equal; `G13.m.2` the row counts feeding `STAGE_THRESHOLDS` are equal | a stage machine that reads sleep as growth, i.e. §11.2's "no non-knowledge predicate moves a stage" |
| (n) the loss is reported, not hidden | `G13/dimming_sweep.json`'s rows | `G13.n.1` `within_shelf_rank_inversions`, `cross_shelf_rank_inversions`, `max_relative_shift` and `floor_hits` are present in every row, **including the winner's**; `G13.n.2` a row with a suppressed count is a fail even when all behavioral legs are green; `G13.n.3` no row quotes a magnitude outside the swept box | the honesty law traded for a cleaner-looking sweep — and the inversions are *real behavior*, across shelves because the floors differ and within a shelf because `d` is per-edge, so the frontier trim can rank differently after a night ([10 §9](10-temperament-sleep.md)) |

**Severity, as designed:** (i) = the design's central invariant is false;
(b)/(j) = the write partition is decoration; (d) = the substrate's meaning of
"aged" is corrupted; (k) = a second affect system; (a)/(h) = determinism; (g) =
the owner is an interruption SI can ignore; (c)/(l)/(n) = the pass is not the
arithmetic it claims; (e)/(m) = a boundary the configs promised to keep.
Every cadence, histogram and shift magnitude here is **UNMEASURED** until
`G13/*.json` exists, and a run in which no sleep ever fired is reported as a
finding about the scenario, not as a green gate.

---

## 19. G14 — TEMPERAMENT *(var: is an individual a difference in arithmetic, and nothing else?)* · CONFIG SI · V0 + V8 + the panel

Renumbered from the chunk's "GATE G13 — TEMPERAMENT". The sentence it grades:
**a seed selects arithmetic; it does not select a law, a memory, or a
personality file.** V8 is the whole trick: the same build, the same transcript, a
different individual — a pair that answers "did the seed change a guard?" by
looking for a guard that moved.

Prerequisites: `seed_panel` (§4.1) with ≥ 16 fixed `u64`s checked into
`harness/scenarios/` and hashed; `g14_twins.txt` run at each panel member;
`G14/seed_panel.json` written per run.

| leg | procedure | asserts | fail means |
|-----|-----------|---------|-----------|
| (a) a seed is not a preload | for every panel seed, the birth litmus on a fresh session | `G14.a.1` at `t = 0` the lattice is **empty**, `a = 0`, the interner empty, zero terms, zero edges — for every seed, including the corners of every TEMPERAMENT box; `G14.a.2` no seed produces a first answer instead of a first ask | the birth seed became a **genome smuggler** — R-25's "a transcript that looks impressive after three owner turns is a preloaded corpus" with a new loading door |
| (b) derivation is deterministic | run the same seed twice in-process and across processes; V6 replay | `G14.b.1` the derived vector is **byte-identical** every time (pure function of `birth_seed` + `seed_format` + `DERIV_ORDER`); `G14.b.2` no wall clock, no host entropy, no thread order is consulted — `check_temperament_closed` includes the absence of any entropy source in `si-core`'s rng path; `G14.b.3` the derivation drains stream **T** and leaves stream **R**'s trajectory untouched, so item 7's ignition history is unchanged by the *presence* of a temperament | individuality bought with determinism (I5), or a seed that perturbs the ignition sampler it is not supposed to reach |
| (c) derived, never stored | open the container, re-derive, compare; then re-write and re-open | `G14.c.1` the persisted core block holds `birth_seed`, `seed_format`, `deriv_order_len`, the **digest** and the sleep counters — and **no** value row; `G14.c.2` re-derivation reproduces the digest exactly, and a container whose digest ≠ its own seed's derivation is **refused at open**; `G14.c.3` `|DERIV_ORDER|` equals the registry's TEMPERAMENT row count, so adding a row changes the vector length ⇒ a `seed_format` bump ⇒ every hash ⇒ no old replay re-read under a new individual | a second source of truth about who SI is — and the silent one, since a stored vector drifts from its seed exactly when it matters |
| (d) the twin law | two panel seeds, same `transcript.txt`, both replayed | `G14.d.1` **same seed + same transcript ⇒ identical twins**: byte-identical `state_hash.txt` and identical activation traces (this is G5(a)'s claim extended across birth derivation); `G14.d.2` **different seed + same transcript** may differ in *row counts, `a` trajectories, arbitration outcomes and settle-sets* — the honest weakening stated in [10 §4](10-temperament-sleep.md) — but every invariant and every other gate must hold for **both** | either the twin claim is false (a hidden host dependency) or individuality was made illegal (the seed collapsed onto one canonical vector) |
| (e) the box holds | every drawn value against its registry range | `G14.e.1` each of the 18 drawn values is inside its declared `[lo, hi]` **on its own grid**, and lands on a grid point (never between rows); `G14.e.2` the mapping is unconditional and single-draw — **no rejection sampling** — so the vector is a pure function with no data-dependent loop count; `G14.e.3` every box corner is reachable by some panel seed, or the panel is widened | a draw that can land outside the swept range, or a mapping whose iteration count depends on the values (the classic way a "pure" function stops being one) |
| (f) **a guard is never seeded** | V0 vs V8 diff over the *whole registry* | `G14.f.1` every LAW row of [03 §6](03-constants-registry.md) is byte-identical across all panel seeds; `G14.f.2` no bound that caps a safety property — `BIAS_CAP_SHARE`, `B_MAX`, the tier table, `PAIR_OVERLAP_MAX`, `REPLAY_TOLERANCE`, the floors' `≥ 1`, `INFERENCE_MAX_HOPS_OWNED` — is read from the `Temperament` type; `G14.f.3` the type carries 18 values and nothing else | **temperament buying its way past a guard** — [10 §2](10-temperament-sleep.md)'s test 2. A shy individual must still be unable to exceed the bias cap; a bold one must still be unable to skip a citation |
| (g) the fingerprint is a readout | the panel and `fingerprint.txt`, with the loop watched from outside | `G14.g.1` the fingerprint enters no labeler input, no gate criterion, no claim, and no control-path explanation; `G14.g.2` the vector is rendered as **registry name + grid value + its box**, with no adjective, no nickname and no vocabulary in the build (`check_emotion_words`, [08 §3.2](08-gui-observability.md)); `G14.g.3` `tree_sha256.txt` of a run with the viewer process absent is unchanged, and the panel's own file writes zero state bytes | a temperament **vocabulary** in a viewer (the emotion words wearing a personality font), or a causal sentence about a parameter in SI's mouth |
| (h) the box is *validated*, not just used | §4.1's panel pass over G9, G10, G11, G13 | `G14.h.1` at **every** panel seed, G9's non-degeneracy, G10(e)'s over-representation, G11's median legs and G13's full leg set pass; `G14.h.2` a failing panel point narrows the **box** in [03](03-constants-registry.md) and re-runs — it does not re-sample the panel, and it does not exclude the seed | [10 §1](10-temperament-sleep.md)'s warning realized: a legal individual landing in an ungoverned corner, i.e. a species range that only validated its own winner (**OD-83**) |
| (i) individuals are distinct | the digests across the panel | `G14.i.1` all `|panel|` vector digests are pairwise distinct — a seed that produces the same vector as another is a degenerate mapping; `G14.i.2` distinctness comes from the values, not from a counter or a name field; `G14.i.3` the digest is over the canonical byte string in `DERIV_ORDER`, so a reordering is a `seed_format` bump, not a silent re-label | a derivation that wastes the seed (the individuality claim is decorative), or one that hashes an address or a nickname into identity |
| (j) the seed is named at the table | the CLI surface | `G14.j.1` `--seed` is **required** to create a session and is printed in the run header; `G14.j.2` there is **no re-roll verb anywhere** — not in `si-cli`, not in the panel, not over MCP (`check_no_affective_setter`'s family); `G14.j.3` a resume with a mismatched seed refuses with a named reason | an individual whose identity is an accident of the host, or a "re-roll" button that turns personality into a lottery ticket the owner draws until something appears |
| (k) a variant is chosen, never invented | `MOD_CURVE_VARIANTS`, per seed (**OD-74**) | `G14.k.1` each seed selects a **member of the sealed set the sweep published**; `G14.k.2` the variant index is inside item 8's hash, so a replay that picked a different member diverges loudly rather than quietly; `G14.k.3` no seed constructs a curve, edits a parameter of one, or reaches a shape outside the swept family | "curve **shape** as temperament" becoming "any shape an individual likes" — §18.1's "swept before quoted, never hand-argued" broken by a seed |

**Severity, as designed:** (f) = the class system is decoration and a temperament
can defeat a guard; (a) = clean-room death; (c)/(b) = identity is either a stored
lie or a host accident; (d) = either determinism or individuality, whichever one
the run chose to break; (h) = the ranges were never actually tested; (g) = the
emotion-word law relocated to the viewer; (k) = a hand-argued curve; (i)/(j) =
individuality that is not real or not nameable.
**Every spread, histogram and "how different are two seeds" magnitude is
UNMEASURED** until `G14/seed_panel.json` exists — including this file's own claim
that individuals differ *interesting* amounts, which is a research readout for the
owner, not a gate criterion.

---

## 20. The static legs (`check_*`) — engineering-only, OD-53

Not in [docs/13](../docs/13-gate-plan.md): these exist because several laws are
*structural* and a green behavioral run does not prove them. They run without a
session, so they belong in every CI job, and each is **also** enforced by a
missing crate edge ([02 §12](02-module-contracts.md)) — the redundancy is
deliberate: the graph catches today's violation, the check catches tomorrow's
refactor.

| id | scans | pass condition | gate it serves |
|----|-------|----------------|----------------|
| `check_emotion_words` | every crate's source except `si-audit` | the six label words appear in `si-audit` only; `si-affect` and `si-front`'s drive layer are wordless | G9 static, G8 swap-test |
| `check_audit_edges` | the dependency graph | `si-audit` has **zero** outgoing import edges into control crates, and no crate imports it | G9(c) |
| `check_idle_no_commit` | symbol reachability from `si-walk::idle` | no write-path or commit symbol is reachable | G10(b) |
| `check_no_mood_field` | every schema, container, cache and export | no field named `median`/`mood`/`bias_state` exists anywhere | G5(a), G9(c) |
| `check_bake_is_the_exception` | the same scan, inverted | `bake` **is** present and stored, write-once — a grep that bans both is a bug that passes checks | G9, G10(b) |
| `check_bake_write_once` | data-flow: every assignment to `bake` | assignments exist only on the `commit` path, none reading a *current* `a` after formation | **G9's round-3 directional check (OD-33)** |
| `check_bake_no_event_kind` | the salience call site's argument list | its only scalar argument is `a_formation`; no event kind, no `source`, no referent | G9 costume test |
| `check_one_clock` | constants in `si-affect` | the only decay-like constant is `LAMBDA_A`; `B_MAX`/`K_TANH` are scale constants; every other decay comes from `Clock::recency` | G11(c), R-29 |
| `check_membrane` | the whole tree | nothing under `src/` references `grimm/`; `grimm` is not a crate | G12(d) |
| `check_no_affective_setter` | `si-supporter` | it reads no `a`, `charge`, `bake`, median, or any door; it imports no `si-affect` | G12(d), rule (vi) |
| `check_no_embeddings` | build + dependency tree | no float-vector similarity API, no numeric model file, no pretrained artifact; **zero tolerance** | G2, R-14 |
| `check_string_constants` | compiled-in string literals | byte-exact match to §3.2's allowlist + 2 `source` literals + 3 TEACH literals | G5(c) |
| `check_config_guard` | every frozen-file handle path | in CONFIG SI no path to L1/L2 is constructible; the refusal precedes path formation | G5(c), G6 |
| `check_deps` | `Cargo.lock` | hash equals the published allowlist value, and no dependency carries a lexicon, corpus or pretrained weight | G5(c), **OD-42** |
| `check_drive_doors` | `si-affect`'s public surface | exactly six methods; and the walk's call sites to the drive are exactly the two, `update` and `conduction` | G8 swap-test, OD-20 |
| `check_signal_kinds` | the signal enum | six kinds, and no construction site outside §6's birth table | G9.pre, I8 |
| `check_record_layout` | compiled layouts | `112 / 16 / 16` byte records and `% 4 == 0`, all reserved/pad fields asserted zero on write and read | G5(a), [06](06-resource-budget.md) |
| `check_registry_size` | the shelf registry | eleven entries, three reserved-empty; a twelfth `x` fails the build (R-12) | G6, DoD |
| `check_no_json_readback` | file-open paths | no run reads `gates/**` or any JSON back as a build input | G12(a), R-33 |
| `check_teach_literals` | the pattern table | exactly ten `P_*` patterns including `P_TEACH`; no teach command grew a vocabulary | G5(c), G1, **R-18** |
| `check_no_stage_on_supporter` | stage-transition code | no transition predicate mentions `source`, Grimm readiness, or any affect quantity | G12, §11.2, §21.6 |
| `check_compose_writes_nothing` | symbol reachability from the settle → label → render path | no commit/write-primitive symbol is reachable from the composition branch — a two-hop answer shares a code path with `learn.commit` ⇒ fail, even if the current scenario never fires it | **G11(h)**, I9, R-25 |
| `check_hop_cap_is_claim_side` | every use of `INFERENCE_MAX_HOPS_OWNED` | it appears **only** in the labeler's citation check and in the registry; **zero** uses inside `walk`/`spread`/`settle_or_stop` — a cap that truncates the walk instead of the claim hides the route from the trace, and an invisible bound is not auditable | **G11(i)**, [04 §5.4](04-algorithms.md) |
| `check_candidate_unanswerable` | the FACT predicate, `retrieve`'s cited-row set, `emit`'s targets | none of them can accept a `links` row of kind `6 cooccur-weak` or `confidence = 0` as an answer's source, whichever way **OD-58** lands | G11(p), R-01 |
| `check_curriculum_is_inbox_only` | every path that can put a line into a session | the `/teach` bulk feeder is an **inbox writer and nothing else**: it imports no store, no learn, no commit symbol, and its lines arrive through the same step-1 read as a keystroke | **G1.u.2/G1.u.3**, R-11, R-12 |
| `check_teach_left_side` | the teach parser's accept set | the left side is accepted **only** when it decodes to one of the four question frames; no statement-shaped left side reaches `commit`, and no POS guesser, template list or dependency parser exists to widen it | **G1.u.1**, R-03, R-18 |
| `check_no_teach_vocabulary` | string literals reachable from the teach path | beyond `/teach`, `=>` and `owner-teach` (already in §3.2's allowlist), the channel owns **no** literal; a synonym table or a curriculum keyword list fails the build | **G1.t**, G5(c), **R-18** |
| `check_gui_observer_only` | `si-vulkan`'s imports and open calls | it holds no lattice handle, writes only `sense_inbox.siro`, and reads `stream.siro`/`monologue.siro`/`mindlog.json`/`sleep.jsonl`/`fingerprint.txt` tail-only | [08 §1](08-gui-observability.md), G5(a) |
| `check_no_viewer_state_write` | the panel's write set | no widget, key binding or drag can reach a state byte, a door, or a sleep/seed trigger — the sleep and fingerprint readouts are **labels over artifacts** | [08 §9](08-gui-observability.md), I8 |
| `check_headless_run` | a build + run with process B never started | every gate's `state_hash.txt` is byte-identical to the viewer-present control, and nothing in steps 1–13 (incl. 12b) opens a window or waits on one | [08 §1](08-gui-observability.md), **G5(a)**'s discipline |
| `check_sleep_writes_nothing` | symbol reachability from `si-sleep::` | no `commit`/`append_row`/`age` symbol is reachable, and no `questions`-row push either: the crate's only store calls are `weight` and the rehearsal pair ([10 §8](10-temperament-sleep.md)). Reachability, not grep: a pass that reaches a row writer *through* another crate fails even with no `use` statement | **G13(i)**, [00 §2](00-stack-workspace.md) win 4, I9/R-25 |
| `check_sleep_never_brightens` | `si-sleep::p3`'s arithmetic | the only weight-writing expression is `w' = f + (w − f)·d` with `0 < d ≤ 1` and `f ≥ 1`; no path can produce `w' > w`, and none can produce `w' = 0` on a live edge | **G13(c)/(d)**, [10 §9](10-temperament-sleep.md)'s refusal of a mass-preserving rescale |
| `check_calm_view_is_terminal` | data-flow out of P4's report | no consumer of a calm-view count in `src/` — not a weight, a floor, `d`, `ignite`, a label, a gate criterion, and not a **new** field in a container | **G13(k)**, R-28, [10 §8](10-temperament-sleep.md) law 1 |
| `check_temperament_closed` | the `Temperament` type + `DERIV_ORDER` + every read site | the type carries exactly `|DERIV_ORDER|` values and that length equals [03 §1.2](03-constants-registry.md)'s TEMPERAMENT rows; **no** LAW row, guard bound or cap is read from it; the vector appears in no schema, container, export or cache; and no entropy source other than the recorded seed is reachable from the derivation | **G14(b)(c)(e)(f)(i)**, [10 §2](10-temperament-sleep.md) test 2 |
| `check_sweep_never_touches_temperament` | the registry's `sweep` tags, every `axes()` enumeration, the `launch_constants` value channel, and the published `*_sweep.json` artifacts | **OD-87's law, structural:** no sweep may ever touch a temperament row. The placeholder rule gives no value to a draw; every axis of every enumeration prefix in the registry is `Class::Species`; the value channel refuses a non-SPECIES name by class; and no published artifact names a draw as an axis. The gate tags the draws still carry are asserted to be **inert metadata** (the artifact that publishes their drawn distribution, not a grid) — which is what lets `check_registry_size`'s count identity keep counting them | [03 §1.2](03-constants-registry.md), **OD-87**, [05 §4](05-gate-harness.md)'s axis rule |
| `check_no_seed_control_surface` | `si-cli`, `si-vulkan`, `si-supporter` | no `--reroll`, no "sleep now", no seed or sleep verb over the seam; `--seed` is required at creation and only *compared* at resume | **G14(j)**, [08 §9](08-gui-observability.md), I8 |

**The arc's ten (B14–B18; the table goes 36 → 46 ids).** Same shape as the rows
above: reachability rather than grep wherever a refactor could route around a
string, and each one is also a missing crate edge ([02 §12](02-module-contracts.md)
rules (xii)–(xvii), [00 §2](00-stack-workspace.md) wins 5–7).

| id | scans | pass condition | gate it serves |
|----|-------|----------------|----------------|
| `check_crystallize_no_commit` | symbol reachability from `si-crystallize::` **and** `si-skills::` | no `commit`/`append_row`/`age`/`salience` symbol is reachable; the crates' only store calls are reads, and their only outputs are values the session turns into `drive::Gap`s | **G15(a)(b)(d)**, **G18(a)(b)(d)**, [02 §12](02-module-contracts.md) (xii) |
| `check_ghost_is_frame_scoped` | every type in the arc's frame scratch, and every container write | no `Ghost`/`RoleGraph`/`CoLight` appears in a schema, container, export, cache or a `SIst`/`SIck` write; the types are constructed **and dropped** inside `si-walk`'s frame | **G15(b)**, [docs/16 §16.8](../docs/16-think-walk-hardware.md), [04 §5.5](04-algorithms.md) |
| `check_silt_writes_nothing` | `si-silt`'s imports and every assignment in it | the crate imports `si-core` and nothing else; its table's sole mutators are `deposit`/`tick`; no store, learn or affect symbol exists anywhere in its graph — **not even an ask** | **G16(a)(b)**, §23.0's litmus, [02 §12](02-module-contracts.md) (xiv) |
| `check_silt_ignition_only` | every read site of `silt::view` | exactly **one** consumer, `walk::ignite`'s seed choice; **zero** uses in `spread`, `settle_or_stop` or `conduction`. OD-104's option (a) made mechanical | **G16(e)**, **OD-104** |
| `check_prov_link_not_stored` | the derived index's construction, and the store's whole write set | no `prov-link` row or edge is ever appended; `SHELF_EVENTS`' `row_schema_version` stays `0`; the index is rebuilt from rows at frame load and holds no handle of its own | **G17(a)(b)**, **OD-110**, docs/12 rule (viii) |
| `check_relight_writes_nothing` | the re-light path inside `si-walk` | the only mutation is frame activation: no `Stamp` construction, no `bake` assignment, no confidence write is reachable, and `src = 2 idle-re-light` is never emitted — **OD-112** rides OPEN-24, which is off | **G17(c)(d)**, [04 §31 AC.3](04-algorithms.md) |
| `check_skills_no_commit` | `si-skills`' graph plus every writer of a `skills`-shelf row | the only writer is `si-learn`'s ordinary commit and `si-skills` cannot reach it; usage counters are **recomputed**, never assigned (**OD-120**) | **G18(a)(f)**, [02 §12](02-module-contracts.md) (xii) |
| `check_nomination_owner_lane` | every sink of a `questions` row whose `key_field` low byte is `origin = 2` | the only lane reachable is the owner lane; no nomination ask is ever routed to a non-owner surface, and no non-owner line can answer one | **G19(f)**, **OD-128**, [01 §4.3](01-data-schemas.md) |
| `check_speaker_class_derived` | every container, schema, export, cache and log writer | no field named or typed as a speaker class exists anywhere except as a **return value**; `speaker_identity` appears only in the framed `SIro` sense body and in `transcript.txt` | **G19(b)**, **OD-123/124**, docs/12 rule (ix) |
| `check_windows_suppress_only` | the idle gate's predicates and every consumer of a window value | a window can appear **only** in a *deny* conjunct; no path runs from a window to `ignite`, `sleep::due`, `clock.advance` or a frame read | **G19(g)**, **OD-126**, [04 §33 AE.4](04-algorithms.md) |

**Six checks move rather than appear** ([12 §6.3](12-v2-arc.md) — the break-list, so
that "nothing we already made breaks" is a claim with a witness):
`check_deps` counts **16** package names and re-pins `Cargo.lock`'s digest (the
allowlist *value* is unchanged — the arc adds no dependency); `check_registry_size`
reads "eleven entries, **two** reserved-empty" from B17; `check_emotion_words` and
`check_one_clock` widen their scan to the three new crate directories, the latter
still passing on **one** recency implementation, which is the point of giving silt a
*share* rather than a τ; `check_idle_no_commit` must stay green with the fifth term
present (an addend is not a write path); `check_candidate_unanswerable` grows an arm
for nomination kinds (a candidate can never be an answer's source, whichever way
**OD-58/98** land); `check_temperament_closed` and
`check_sweep_never_touches_temperament` must stay green with 22 new rows, the first
because the draw count stays **18**, the second because every new row's `gate` tag
points at an artifact whose `axes()` is asserted `Class::Species`-only.

### 20.1 The tongue's six (`check_*`, B20; the table goes **46 → 52 ids**)

The same shape again, and the same reason: [02 §12](02-module-contracts.md) rules
(xviii)–(xxi) are structural, and a missing crate edge only protects today unless a
check names it ([13 §6.2](13-tongue.md)).

| id | scans | pass condition | gate it serves |
|----|-------|----------------|----------------|
| `check_tongue_closed` | `si-core/src/tongue.rs`, `temperament.rs`, `seed.rs`, `rng.rs` | `style_order_is_closed()`: `DERIV_ORDER_TONGUE.len() == 5 == consts::tongue_count()`, every drawn row is `Class::Tongue`, `size_of::<Style>() == 10`, and no non-`Tongue` row is drawn; the four files reach none of `getrandom, rand::, SystemTime, Instant::now, env::var, std::fs, File::open`; `style_vector`/`style_values` appear in no other source file; `derive_style(seed)` is stable across two calls **and differs from `derive_temperament(seed)`'s draw sequence** | **G20(a)(k)** |
| `check_tongue_is_leaf` | `crates/si-tongue/Cargo.toml` + every file in the crate | the dependency set is exactly `{si-core, si-walk}`; `si_store`, `si_front`, `si_learn`, `si_affect`, `si_read` and `std::fs` appear nowhere in its text; the only crate in the workspace that names `si_tongue` is `si-session` | **G20(b)(c)(h)**, **OD-181** |
| `check_tongue_writes_nothing` | `si-tongue`'s symbols + the call site | no `&mut` parameter except the `Rng` it is handed, no `commit`/`append`/`put`/`write`/`save` symbol, return type `Option<String>`; the call site assigns the result to `surface` and to nothing else | **G20(d)(f)** |
| `check_tongue_reads_no_a` | `si-tongue` + `si-core::tongue` | `a_at`, `a()`, `arousal` and `Integrator` appear nowhere in either; `render_seed`'s operands are `birth_seed`, `Style::digest()` and `thought_id` **and nothing else** | **G20(e)**, **OD-168/182/183** |
| `check_style_never_installed` | every scenario fixture, every published `*_sweep.json`, `runner.rs` | no `STYLE_*` name appears in a `constants` declaration or in a sweep artifact's `axes`; `launch_constants`' guard text is **byte-identical** to the `Species \| Ruling` refusal; the `match` arms added at B20 name `PHRASE_MAX_STEPS` and `TONGUE` and **no** style row | **G20(i)**, **OD-87/177** |
| `check_tongue_license_only` | `si-tongue`'s literals | no `&'static str` array or table of more than one entry; the only multi-char literal the crate emits is the joiner `", "`; every push into the phrase buffer takes its bytes from a `License.surface` | **G20(c)(h)(k)**, R-21 |

**Eight checks move rather than appear** ([13 §6.3](13-tongue.md) is the witness
list): `check_deps` counts **17** package names and `Cargo.lock`'s digest **does not
move** (the tongue adds no dependency); `check_temperament_closed`'s scan list gains
`si-core/src/tongue.rs` while the 18 draws stay the 18; `check_registry_size` prints
`{total}` = **103** and its "18 TEMPERAMENT draws" clause is unchanged, because
`Class::Tongue` enters none of the four buckets of the count identity;
`check_sweep_never_touches_temperament` stays green with a `G20` sweep prefix in
existence, because `axes("G20")` enumerates one **SPECIES** axis (`PHRASE_MAX_STEPS`)
and the class bail never fires; `check_emotion_words` widens to `crates/si-tongue/`
and passes on a crate that declares no word table; `check_no_seed_control_surface`
covers `Style` the way it covers `Temperament` — no setter, no install path;
`check_record_layout` and `check_one_clock` move **nothing** (no byte, no clock).

---

## 21. Sequencing

**Per commit:** all `check_*` legs (no session needed), then G1, G4, G8(a)(b)(f),
G12(b)(c)(d), G13(i)(k) and G14(c)(f) as the short behavioral set — the last two
because they are the cheapest way to catch the *structural* failure of a new
organ before spending a long session on it. **From B14 the arc's ten static legs
join this set** — they are static, they cost no session, and six of them are exactly
the kind of leg that catches a refactor before it becomes a behavior. **From B20 the
tongue's six join it too** (§20.1), and `G20(b)` + `G20(j)` are the cheap pair for an
organ that *prints*: the first says a label cannot be reached from a surface, the
second says nothing already printed moved.
**A `CONSTS_VERSION` bump outranks a version tag the way a `seed_format` bump does**
([12 §3.3](12-v2-arc.md), and again at **B20**): `VersionTags::encode` writes the registry's row **count**
and every `(name, ver)`, so B14's 22 new rows re-baseline hash item 8 and every
`state_hash.txt` recorded earlier — and B20's eight rows plus the `0x0002 → 0x0003`
bump do it a second time, which is why **three golden constants re-pin in the same
commit as the bump** (`EMPTY_STATE_MD5_SEED_ZERO` in `si-core/tests/kernel.rs`,
`si-session/tests/seed.rs` and `si-session/tests/wake.rs`; [13 §3.4](13-tongue.md)).
The rule is unchanged — older artifacts stay
attributable to the arithmetic that made them and are **re-run, never edited** — and
the arc's own artifact table ([12 §9](12-v2-arc.md) rows 2–4) is the ledger of which
gates must be re-recorded: G5, G10(e.5), G12(a), G13 and G14, because those are the
legs that compare hashes.
**Per release candidate:** the full **twenty**, each on V0, plus the paired
variants V1, V2, V4, V5, V8, V9 and the replay V6, G6 on V7, **V10–V13** as the
arc's deletion and grant pairs (§23) and — from B20 — **V14** as the voice's
additive pair (§24).
**Per re-sweep:** the sweep runners (§4), ten through B13, **fourteen** from B14
(§23.6) and **fifteen** from B20 (§24.3), each at the reference seed and then
across `seed_panel` per §4.1, then G2,
G9, G10, G11, G13 re-verified on the new version tags — because a swept constant's
version tag is a hash input
([03 §1](03-constants-registry.md)), every older `state_hash.txt` remains
attributable to the arithmetic that produced it, and no re-sweep may silently
alter a replay. A **`seed_format` bump outranks a version tag** in this table: it
changes who the individual is as well as what the arithmetic was, so it also
invalidates every `G14/seed_panel.json` recorded before it.
**G5 runs last in every set**, since it is the gate that hashes the output of all
the others; a fail there outranks a green everywhere else only for (b)/(c), which
the design ranks above the rest.

---

## 22. What the harness may never do

* **no setter** — it may not raise `a`, seed a row, pre-create a term, or fast-forward
  a stage; G9 and G11 are invalid by construction if it does;
* **no pre-seeded genome in CONFIG SI** — V0's lattice is L3-only, so a scenario
  cannot hand SI a vocabulary (§3's birth litmus, R-25);
* **no reading a sweep artifact as an input** — outputs feed the next run's
  *declared constant list* only (§4);
* **no scenario-specific code path** — a leg that needs a special branch in the
  loop is a design failure the harness must report, not accommodate;
* **no quoting of an unmeasured number** — the report cites artifacts, and the
  docs cite reports, never the reverse.

Two more, added by the TEMPERAMENT/SLEEP batch, and both are about the harness
tempting itself rather than SI:

* **a `--seed` is not a scenario device** — the harness may name an individual
  (V8, the panel, `--seed` at creation) but it may not *choose* one to make a leg
  pass. Picking seeds until G9/G10/G11 go green is §4.1's forbidden move, and it
  is the same fault G2 calls "hand-picked off-sweep" with one layer of statistics
  added; a scenario states its seed in the checked-in file and the file's hash
  pins it. Note the distinction [10 §5](10-temperament-sleep.md) draws: seeding an
  **arithmetic** is legal, seeding a **row, term or `a`** is the bullet above;
* **no sleep started by the harness** — no `si-cli sleep`, no "sleep now", no
  scenario line that advances the quiet counter artificially, and no pin that
  forces `due()` to `Yes`. If a sleep does not fire inside a scenario's own quiet
  stretches, the scenario is the finding (G13(f)), and the fix is a longer
  checked-in transcript, never a launch hook. A run at V9 is *disabling* the
  trigger by its own swept range, which is a control; a run that *invites* a
  sleep is a setter.

**And three the v2 arc adds, in the same spirit** (§23, [12 §6.4](12-v2-arc.md)):

* **the harness may not mint a concept to make a leg pass.** It may install
  `CRYSTALLIZE = on` through the launch channel and feed a transcript; it may not
  write a `links` row, seed a candidate table, or lower a recurrence bar mid-run.
  A bar that had to be lowered to open an ask is a **finding about the bar**, and
  G15(e)/G18(e) report the counts, not a pass.
* **a refusal is reported as a refusal, never as a skip.** Three of the five
  families grade an *absence* (the sweep refusals while **OD-83** is unrulled, the
  belief-write refusal of **OD-135**, the missing `/trust` parser of **OD-136**).
  Each appears in the artifact with `status: refused` and the OD number in `observed`
  — exactly as G10's sweep leg already does — and a leg that quietly returned
  `passed` over nothing is the failure this bullet exists to catch.
* **the arc's needles live in the harness, never in `src/`.** `recent`, `just`,
  `earlier`, `ago`, `still`, `felt`, `flashback`, `remember how` are G16(f)/G17(f)
  search strings, not vocabulary: they belong to `crates/si-cli/src/harness/`
  (the auditor scope, exempt by `Workspace::is_auditor`) and to the checked-in
  fixtures, and one that leaked into a control crate would be R-27 and R-21 at the
  same time ([docs/23 §23.7](../docs/23-context.md)'s R-21 extension).

---

## 23. G15–G19 — the five arc families

Appended as one section after §22 for the reason §28 of
[04](04-algorithms.md) was appended there: §20 and §21 are cited **by number** from
five other files, and a section's position here carries no meaning. The *what and
why* is [docs/22](../docs/22-crystallize.md)–[docs/26](../docs/26-world.md); the
crates, bytes and checks are [12](12-v2-arc.md). Every family below is CONFIG SI,
V0 plus its deletion variant, one scenario file checked into `harness/scenarios/`
whose hash is pinned in the report, and one seed stated in the file.

**The five legs every family runs first, in this order** — they are the same five
because the arc's claim is one claim:

**`questions_filtered_digest`, defined once because two legs need it.** The arc's
nomination *does* write a row — an ordinary `questions` row — so an `on` run and an
`off` run of the same transcript cannot have a byte-identical whole `state_hash.txt`,
and a leg that claimed they do would be a leg that fails for a reason that is not a
defect. The comparison is therefore **shelf-filtered, and the filter is named**: the
digest is `md5` over the canonical LE bytes of every live row of every shelf
**except `questions`**, in arena order, chained with the other eight items exactly
as [01 §10](01-data-schemas.md) composes them (`StateHash::sub_digest` already gives
the harness the eight it needs). The `questions` shelf is then compared **on its
own terms**: same row count, or a difference consisting *only* of rows whose
`key_field` names the family's kind with `origin = 2`. And one more obligation, hard
rather than assumed: **the interner is byte-identical in both runs**, which is what
forces a nomination ask to enumerate **handles** and not words
(§9 row 9 of [12](12-v2-arc.md)); an `on` run that mints a string has leaked content
into a detector's output and is a red. Nothing else about a family may differ, and
for **V11** and **V12** — which mint no ask at all — the assertion is the *whole*
`state_hash.txt`, byte for byte, per OD-91.

| leg | procedure | pass | catches |
|-----|-----------|------|---------|
| (a) **determinism** | replay the transcript twice, mechanism `on` and `off`, through the launch-constant channel only | the two **`questions_filtered_digest`** values are byte-identical, the `questions` difference is *only* rows of the family's own kind, the interner matches, and the leg **names the exclusion in the report** — because a comparison that silently drops a shelf is the kind of fudge that makes a green gate mean nothing | a mechanism that reaches state |
| (b) **deletion** | the family's V10–V13 build, same transcript | zero rows, edges or stamps differ from the control beyond the filtered digest above; where the variant mints no ask (V11, V12) the hash is **equal**, not "close" (B7's ruling: the arc-off run is the control and equality is the pass) | a reader that writes |
| (c) **arousal isolation** | **frame-trace-fixed** (**OD-134**'s reading (i)): feed the *identical* frame log under two `a` regimes | the family's own table byte-identical. The full-session reading stays `UNMEASURED` beside OD-83's refusal — a whole session at two arousal levels differs in *which frames exist*, so it cannot isolate this mechanism and nobody may present it as if it could | `a` entering a detector |
| (d) **owner-only** | a `grimm:` (B9's seam) line aimed at the family's open ask or row | zero **belief-row** writes; its `context` sense row still appends — a run that suppressed those is a *different design*, reported as one | laundering through a second voice |
| (e)/(f) **the family's own bars** | per family below | per family below | — |

### 23.1 G15 CRYSTALLIZE — `g15_fruit.txt`, `CRYSTALLIZE = on`, `--seed 0x51`

The apple/tomato/banana fixture of [docs/22 §22.5](../docs/22-crystallize.md), run
across enough distinct cold frames to clear `R_MIN` at the channel floor.

| leg | procedure | pass |
|-----|-----------|------|
| (e) **spam bound** | junk fixture: random pairs, 2-member sets, open contradictions, one hot afternoon entirely inside `DELTA_MIN_CRYSTALLIZE` | **zero** asks. Supersaturated-but-never-asked candidates print as **counts**, never as failures — a bar tuned above the measured recurrence depth of any real session is not generous, it is untested (§3's objective block) |
| (f) **no word leak** | RENDER output over a run with live unnamed candidates | no `cluster:` string, no invented word anywhere in the surface; a minted-but-unnamed point is cited **by member enumeration only**, and `NAMELESS` never prints ([04 §21 U.1](04-algorithms.md)) |
| static | `check_crystallize_no_commit`, `check_ghost_is_frame_scoped`, `check_candidate_unanswerable` (grown) | green from B14 |
| artifact | `gates/G15/nomination_log.jsonl` (every candidate transition) + `gates/G15/recurrence_sweep.json` (**`status: refused`** while OD-83 is unrulled) | both present in the report; the sweep's refusal is *named*, and the legs that needed a number ran at their box floor and say so |

### 23.2 G16 SILT — `g16_warm.txt`, two configurations and one claim each

Silt is gated at **two** settings, and conflating them is how this family would go
falsely red. **`IGN_WEIGHT_SILT = 0`** (the shipped, unarmed default) is where
legs (a)/(b) run: a 0 addend cannot move a byte, so hash equality across on/off is
*achievable* there and is the assertion. **`IGN_WEIGHT_SILT = lo + grid`** (G16
installs the **smallest non-zero on-grid step**) is where legs (d)/(e) run: the term
is present, the wiring is proven, and the pair is compared on **behaviour** — drop
order, `T_cold`, the empty table — never on the hash, because an armed preference
lawfully moves the idle path (settle-set and trajectory are its own items) and
demanding equality there is OD-91's category error. The leg proves the wiring, not a
warmth claim.

| leg | procedure | pass |
|-----|-----------|------|
| (a′) **exemption asserted** | the declared hash exemption (**OD-103**) is *tested*, not assumed: at `lo + grid`, two runs of the same transcript differ in `silt.jsonl` **only** as far as the armed preference lawfully moves the idle path — `settle_set` and `trajectory` may differ on the points the fifth term seeded, and the report enumerates them — while `live_rows`, `edges`, `stamps` and the interner stay byte-identical (idle commits nothing: `check_idle_no_commit` is the witness) | exactly that partition, in that direction; a row, edge or stamp difference is a red, and so is a level that appears in the hash |
| (d) **clears backwards** | scripted touches at known ticks, then a long idle stretch | entries fall below `THETA_SILT` **strictly in last-touch order** (earliest last-touch first), and after the computed `T_cold` the table is **empty** and the fifth term is identically 0, so the sampler equals §19's lattice-only sampler exactly. No level comparison anywhere — that is the point (**OD-109**) |
| (e) **crystallize isolation** | the **frame-trace-fixed** shape again (OD-134's reading, for the same reason): feed `crystallize.observe` the *identical* `frame_log` under silt armed and silt absent, and diff its candidate table | the table is **byte-identical**, via the cold-frame rule (a primed frame that increments a recurrence is a **red**) — this leg is why [04 §29 AA.3](04-algorithms.md) had to define "cold" once for both detectors. It is *not* an end-to-end on/off comparison: an armed preference legitimately changes which frames exist, and comparing two different walks would grade the walk, not the detector |
| (f) **no innate time words** | the needle set (`recent`, `just`, `earlier`, `ago`, `still`) over the run's RENDER output | absent unless the owner taught them; `silt` names **rows** in audit readouts and never time |
| static | `check_silt_writes_nothing`, `check_silt_ignition_only`, `check_one_clock` (widened), `check_idle_no_commit` | green from B15 |
| artifact | `gates/G16/silt_log.jsonl` + `gates/G16/silt_sweep.json` (refused) | — |

### 23.3 G17 EPISODIC — `g17_origin.txt`, `EPISODIC_TRAVERSAL = on`

| leg | procedure | pass |
|-----|-----------|------|
| (c) **arousal isolation, read-side** | the **state** items pinned (`live_rows`, `edges`, `stamps`, `interner`, `version_tags`, `core_block`); the re-lit point's **activation** allowed to move with `a_now` (`settle_set`, `trajectory` are the two items this leg does *not* pin) | the movement is §20.2's existing conduction term doing its job, and **no new curve** appears — `check_one_clock` is the witness. Pinned items byte-identical, moving items differing only on re-lit points the tug log names |
| (d) **no re-stamp — structural** | same stamp-arena **record count** and **byte length** after N re-lights; values free | zero new stamps, asserted both statically (`check_relight_writes_nothing`) and by the arena count; the report prints **OPEN-24 off** as the reason, the way OD-88 taught this repository to grade a stamp leg |
| (e) **grimm isolation** | grimm event views re-lit N times | zero writes, plus the absence grep: **no tug names a `context` row whose `prov_speaker` is `1` as a belief origin** (R-35 one notch out) |
| (f) **no flashback** | the experience-claim needle set (`felt`, `flashback`, `remember how`) over RENDER | absent; recall prints as `FACT/OWNED` with **both** citations, so remembering is citing and not reliving |
| static | `check_prov_link_not_stored`, `check_relight_writes_nothing`, `check_ghost_is_frame_scoped` | green from B16 |
| artifact | `gates/G17/relight_log.jsonl` + `gates/G17/prov_sweep.json` (refused) | — |

### 23.4 G18 SKILLS — `g18_compare.txt`, `SKILLS = on`, `SKILL_COMPOSITION = on`

| leg | procedure | pass |
|-----|-----------|------|
| (e) **spam bound** | junk fixture as in G15(e), plus shapes seen in one conversation only | zero asks |
| (f) **generality bound** | a shape recurring over `K_FILLER − 1` distinct filler sets, then over `K_FILLER` | below the bar it **never opens** and is counted `forwarded`; and the forwarded frames **remain available to G15's detector** — the exclusion is one test, not two claims (**OD-121**) |
| (b) **deletion, both halves** | V10 build, and separately rows *installed and sitting unread* (`SKILL_COMPOSITION = off`) | detection and use are two ruling rows, so both states are gated: rows present-but-unused change no byte |
| static | `check_skills_no_commit`, `check_registry_size` (two reserved), `check_nomination_owner_lane` | green from B17 |
| artifact | `gates/G18/skill_log.jsonl` + `gates/G18/shape_sweep.json` (refused) | — |

### 23.5 G19 WORLD — `g19_server.txt`, 10 identities (one owner, one grimm, eight strangers)

**OD-132's harness fixture: multi-voice is a scenario, not a transport.** No
network, no credentials, no adapter — and the family's two hardest legs are about
authority, so they are the two the fixture can honestly run.

| leg | procedure | pass |
|-----|-----------|------|
| (a) **stranger silence** | eight strangers speak, owner silent | zero protected-class writes; `context` rows **do** append, and a run that suppressed them is reported as a *different design*, not a stricter one |
| (b) **class derivation** | `check_speaker_class_derived`, plus a report of `class(identity)` over the whole fixture | no stored class anywhere; the report is the function's output per identity, and a stranger is a **stranger**, not a labelled one |
| (c) **grant trace** | V13: an owner grant line mints a grant row; a Trusted line attempts a belief write; the revocation row is appended; the same identity speaks again | the belief write is **refused with zero bytes** (`Refused{SpeakerNotRepresentable}`) and the next derivation returns `Stranger`. **The refusal is the pass condition**, and `observed` prints **OD-135** as the reason |
| (d) **answer-to-ask** | a stranger answers an open nomination ask | commits nothing; the ask stays open and keeps aging (kind 4 keeps firing) |
| (e) **determinism** | identity-tagged transcript replayed | byte-identical — identity is in the **frame body and the transcript**, which are already hashed transitively, so no new hash item is needed |
| (f) **routing** | every `origin = 2` ask in the run | lands in the **owner lane** and nowhere else (**OD-128**) |
| (g) **starvation** | a scripted busy server with a declared window | idle ticks occur **only where the window is open *and* the quiet-and-empty-agenda law already held**; no tick occurs *because* a window opened; owner preemption intact (**OD-126**) |
| (h) **speakgate for grants** | under V13, a Trusted line emits | passes the **same** two-layer filter and the same token-membership test grimm's lines pass — one function, one filter, one class, which is why this leg is cheap and why it is honest |
| static | `check_speaker_class_derived`, `check_windows_suppress_only`, `check_nomination_owner_lane`, `check_candidate_unanswerable` (grown) | green from B18 |
| artifact | `gates/G19/server_transcript.txt`, `gates/G19/grant_trace.json`, `gates/G19/lane_log.jsonl` | the transcript's hash pins the fixture (§5) |

### 23.6 The four sweep runners, and what they publish

`recurrence_sweep` (G15), `silt_sweep` (G16), `prov_sweep` (G17), `shape_sweep`
(G18) join §4's ten → **fourteen**. Each enumerates `Class::Species` axes only
(`check_sweep_never_touches_temperament` holds, and the 16 arc rows are SPECIES by
[03 §1.2](03-constants-registry.md)), each carries its `build_variant` id and the
graph digest (**OD-93**), and **each publishes `status: refused` while OD-83's
enumeration is unrulled** — the same shape G10's ignition sweep already has, and the
reason `IGN_WEIGHT_SILT`'s sweep tag is **inert**: no refusal can arm it. The axes
are therefore *enumerated in the registry and not yet searched*; a leg that needs a
number runs at its box's `lo` and names itself a floor, per OD-95's precedent
(`runner.rs::sleep_lo_point()`). **No magnitude from this section is quotable.**

### 23.7 What these five families do not test, stated so nobody reads a green as more

They do not test that nominations are **useful** (no warmth, no utility measure —
G15/G18 grade *absence of spam*, which is the honest half); they do not test silt at
a real value (its sweep is a refusal); they do not test a Trusted belief row
(existing — **OD-135** refuses it); they do not test the Discord transport or the
`/trust` vocabulary (**OD-136**, not built); and they do not test that a concept or
a skill *transfers* — that is [docs/13](../docs/13-gate-plan.md)'s S-stage curriculum
question, unchanged and unasked here. A green G15–G19 means: **five mechanisms run,
write nothing they were not licensed to write, and change no byte of the arithmetic
that already shipped.** That is precisely the owner's requirement — *"engineer them
thoroughly so nothing we already made breaks"* — and it is stated here because a
gate that cannot fail is worse than a gate that does not run.

---

## 24. G20 — TONGUE *(var: does a drawn voice change the words and no byte of
state?)* · CONFIG SI · **V14** + one refused sweep *(B20;
[docs/27](../docs/27-tongue.md), [13 §6](13-tongue.md))*

`g20_tongue.txt`: a `seed` line, an owner teach block that gives one anchor row a
composed surface and **at least two term-to-term live arcs with distinct surfaces**,
then a question tick. The fixture's md5 is pinned in the report (`expect_fixture`,
§23.5's discipline), because a voice leg that silently swaps the fixture is a voice
leg that passes on nothing.

### 24.1 The eleven legs

| leg | predicate | procedure |
|-----|-----------|-----------|
| **G20(a)** | replay | the V14 drive runs twice in **two processes**; `realized_log.jsonl` is byte-equal, including each line's `drew` fraction list, and the two state hashes are byte-equal (the voice moves no item). Second pair on an empty lattice: every tick is a `no_candidates` refusal and every printed surface is the canonical one — **the fallback, not a crash** |
| **G20(b)** | label immutability (ordering law 30) | for each settled frame, realize at the style corners (`0` and `2048` per axis, at the panel points a two-seed pair gives) and assert `label`, `tag`, `citation` byte-equal across all of them while `surface` differs on at least one; assert the state hash equal to the same drive at `TONGUE = 0` |
| **G20(c)** | no invented words | every realized surface's folded token set is a subset of the license's; plus a forced failure — a fixture row whose only arc leads to a point with no surface prints the canonical frame and adds one line to `refusals.jsonl` |
| **G20(d)** | the two keys still | a stranger line and a `voice`-prefixed grimm line that *name* a shape the tongue liked produce **zero** rows; V14's row count equals the `TONGUE = 0` run's, and G1's seam legs stay green on the V14 build |
| **G20(e)** | arousal isolation (**OD-168** widened to the voice) | hold `B_MAX`/`K_TANH`/`LAMBDA_A` and the citation set fixed and vary arousal: assert the `drew` sequence byte-identical and every realized line identical for the ticks whose citation set is the same, and **print** the ticks where arousal legitimately changed *which row was cited* — that is v1 behaviour, not the tongue's. Type half: `check_tongue_reads_no_a` |
| **G20(f)** | no self-farming | a scripted session that re-enters SI's own realized lines as sense opens **zero** asks and mints **zero** rows; the realized line never reaches `seam.speak`'s input, because the call site's output lane is its only consumer |
| **G20(g)** | parse-back | the leg forces a lost slot (a license whose anchor surface's last content key is removed) and asserts the canonical frame printed and `refusals.jsonl` says `slot_lost`. **OD-173**: slots-present is the check; no decode walk is invented for it |
| **G20(h)** | **out-of-citation emission impossible** | grep every realized token against the surfaces of the points *that line's own citation* names, plus the composed chunks; the empty offender set is the pass. The registry-wide grep the draft proposed is **retired**: against a registry-wide set every legal line passes, so it is a leg that cannot fail (**OD-181**) |
| **G20(i)** | budget bound | at `PHRASE_MAX_STEPS ∈ {1, placeholder}` × every style corner: no line exceeds `1 + budget` chunks, and an over-cap walk **refuses** rather than truncates. `phrase_sweep.json` is cited **as a refusal** and `observed` says so (**OD-83**, **OD-95**, **OD-172**) |
| **G20(j)** | v1 compatibility | at V0, every `EV_OUTPUT` surface of every existing scenario equals the cited row's own `surface` byte for byte (`v0_strings.json`), **and** the full battery G1–G19 is green on the 17-crate, `0x0003` build. The proof is **string equality**, not hash equality, because the tongue writes no state — a hash would stay identical even if the voice changed every sentence (**OD-178**) |
| **G20(k)** | genome containment | under CONFIG ENGINE, assert no realized chunk is reachable *only* over a `kind ≥ GENOME_KIND_FIRST` arc: `!a.is_genome()` is filter 3 of step 2, so a newborn whose live arcs are empty says nothing the species text taught it (**OD-179**) |

Leg ids are `G20.a.1`, `G20.a.2`, … (§1's convention), each with a `fail_meaning`
naming the invariant it guards — I1, I5, I6, I8, I9, R-21, R-43.

### 24.2 Its four artifacts

`gates/G20/` (harness-written; `gates/*/` is gitignored, so a fresh run **supersedes,
never edits**):

| file | read by | what it proves |
|------|---------|----------------|
| `realized_log.jsonl` | **G20(b)(h)** | one line per realized tick — `{t, thought, seed, anchor, citation, canonical_surface, realized_surface, drew}`. Two legs read one file because they are one emission read two ways: equality of the parts, membership of the words |
| `refusals.jsonl` | **G20(c)(g)(i)** | every `None` with its reason (`no_candidates`, `unlicensed`, `slot_lost`, `budget`). **An empty file passes nothing** — the forcing legs make it non-empty on purpose |
| `phrase_sweep.json` | **G20(i)** | `status: refused`, naming **OD-83** and **OD-172** (§24.3) |
| `v0_strings.json` | **G20(j)** | every `EV_OUTPUT` surface of the V0 run in step order, each proved to be one cited row's own `surface` byte for byte |

### 24.3 The `phrase` runner — the fifteenth, and it publishes a refusal

`RUNNERS` gains `("G20", "gates/G20/phrase_sweep.json", "phrase")` and
`scenario_path` gains `"G20" => "harness/scenarios/g20_tongue.txt"`. The single axis
is `PHRASE_MAX_STEPS`, whose box ceiling is the type max, so `open_ceiling` is true
and `sweep_one` takes its written-refusal branch — the same branch `silt`, `prov` and
`shape` take (§23.6), with the same OD-83 question plus **OD-172**'s own half (what
the step bound is *for*). **Objective, quoted not paraphrased:** the **smallest**
`PHRASE_MAX_STEPS` at which a licensed two-arc fixture still realizes at least one
line *longer than its canonical chunk* **and** G20(c)/G20(g) open zero refusals —
the same shape as `INFERENCE_MAX_HOPS_OWNED`'s bar (§16): a bound below that point is
not tight, it is **decorative**, because the voice it permits says nothing the copier
did not already say. `objective_value("G20", report)` reads
`G20.i.1`, `G20.i.2`, `G20.c.1`, `G20.g.1`.

Consequence, stated plainly: **no leg quotes a swept phrase budget.** G20(i) runs at
`1` (a floor, named as one, per OD-95's `runner.rs::sleep_lo_point()` precedent) *and*
at the registry placeholder, and asserts the cap holds at both.

### 24.4 What G20 does not test

It does not test that a realized sentence is **good**, apt, or pleasant — no
leg grades prose, and the only quality predicate in this folder is "does it still
contain the slots it was built from" (**OD-173**). It does not test the internal
monologue lane (§27.7 keeps `render_monologue` canonical until the owner rules
otherwise), an ask's named target, or a style value's *meaning*: `STYLE_SPREAD` is a
share of a budget, not a personality. And it does not arm the voice at V0: **OD-178**
ships `TONGUE = 0`, so the pre-B20 control is the ordinary run and every existing
gate's hash stays the hash it recorded on the same arithmetic.
