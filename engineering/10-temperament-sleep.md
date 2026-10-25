# 10 — Temperament Seed, Sleep Phase, Dimming

**Provenance.** The owner's **TEMPERAMENT SEED + SLEEP PHASE + DIMMING/SPARK**
chunk, with the standing instruction *"you can go straight to engineering those
(logic + engineering in once since its new object but no need to go first to
logic since you are engineering already)"*. This file therefore does the work
`docs/` would normally do first, and it does it under the folder's authority
order: **it adds no organ, no door, no shelf, no signal, no label, no emotion**,
and every place the chunk's wording cannot be built without breaking a law, the
law wins and the wording goes to the ledger with a lean.

**The four laws this batch has to survive** — everything below is either an
application of one of them or a refusal forced by one:

| law | source | what it forbids here |
|-----|--------|----------------------|
| **I5** determinism in logical `t`, never wall-clock | [docs/00](../docs/00-thesis-invariants.md), [docs/16 §16.6](../docs/16-think-walk-hardware.md) | a sleep triggered by a clock reading; a seed drawn from host entropy for a run that claims reproducibility; a rescale whose result depends on iteration order |
| **I8 / §18.1** installed affect is one scalar; the vocabulary lives only in the audit layer | [docs/18 §18.1/§18.4/§18.6](../docs/18-emotions-engine.md) | a constant *named* after an emotion in the build; a second integrator (hysteresis, a per-signal λ); a curiosity **level** that regulates emission |
| **I9 / R-25** an idle walk may notice, never learn | [docs/19 §19.4](../docs/19-idle-ignition.md), [docs/05 §5.3](../docs/05-substrate-tesseract.md) | sleep being the loophole that lets un-taught content become a row |
| **§5.3(e) + OD-27/OD-33** permanence is always bounded; the bake is one saturating curve read once | [docs/05 §5.3](../docs/05-substrate-tesseract.md), [docs/20 §20.4a](../docs/20-emotional-plasticity.md) | a dimming floor that bake makes unbeatable; any second, unbounded durable affective quantity |

**Renumbering note, stated once (no ruling needed).** The chunk names its gates
"GATE G12 — SLEEP" and "GATE G13 — TEMPERAMENT". **G12 is GRIMM ISOLATION**
([docs/13](../docs/13-gate-plan.md), [05 §17](05-gate-harness.md)), so this batch
lands them as **G13 SLEEP** and **G14 TEMPERAMENT** ([05 §18, §19](05-gate-harness.md)).
Likewise the chunk's "OD-57" was already assigned **OD-58** when the COMPOSITION
batch landed, and the collision was recorded there; the items below continue the
ledger from **OD-71**.

---

## 1. What temperament is, in one table — and the three things it is not

| question | answer | why the design can afford it |
|----------|--------|------------------------------|
| What is temperament, mechanically? | **a per-individual point inside the legal ranges the sweeps already publish** | every value it selects already has a home, a grid, a range, a sweep artifact and a consuming gate in [03](03-constants-registry.md). Temperament chooses *where in the box*; it never opens the box |
| When is it fixed? | once, at birth, before the first round of the first thought | §3: "never re-rolled" — and a value that cannot change during a session cannot be steered by anything inside the session |
| What does it change? | how strongly signals integrate, how fast arousal settles, how much a birth head-start bends conductance, which points intrude when quiet, how often SI asks | all of them are **mechanism strengths** already licensed as form (§18.1's inventory: the innate part is the signal list, one integrator, the existence of a curve) |
| What does it not change? | the walk's integrity constants, every guard, every bound, every LAW value, the pattern table, the shelf registry, the signal kinds, the label vocabulary | [docs/03 §3.4](../docs/03-birth-state.md)'s enumerability audit is byte-exact and seed-independent; G14(b) asserts it |
| Is it content? | **no.** It buys no word, no fact, no relation, no prior | the anti-magic leg, G14(b): the union of term ids over a whole seed panel equals the term ids of any one run of the same transcript |
| Is it a mood? | **no.** At `t = 0`, for every seed in the panel, `a` is at rest, no label is emitted, no stamp exists, no edge exists | §18.1's litmus and §3.4's affective empty-state test, re-run per-individual (G14(a)) |

**Not a privileged sensor.** §18.1: "nothing in the panel, config, or a
designer's hand can set arousal directly." The seed sets no level. See §5.
**Not a per-individual sweep.** §2's class test keeps every constant that a
sweep *objective* depends on at SPECIES rank, so the objectives stay population
properties and the sweep stays one search.
**Not a personality file.** Nothing is derived from the owner, from a corpus, or
from any artifact; the derivation's only input is a `u64` (§4).

## 2. Three classes of constant

The registry has carried a `kind` column with two values (`swept`, `LAW`). It
gains a third partition, which is the chunk's actual contribution:

| class | owns | default | sweep artifact | may a seed move it? | count in v1 |
|-------|------|---------|----------------|---------------------|-------------|
| **LAW** | a value the design fixed in prose (confidence tiers, stage thresholds, `IDLE_PER_TICK = 1`, `CONF_CAP = 99`, `GENOME_WEIGHT_CEILING`, `B_MAX` being a bound at all) | quoted | none — and inventing one is a design change | **no** | [03 §6](03-constants-registry.md)'s table |
| **SPECIES** | what makes this *kind* of mind: a value shared by every individual, chosen once by a sweep whose **objective** is stated over the whole population | `—` | `gates/G<n>/*_sweep.json`, as today | **no** | the swept rows of [03 §§2–5](03-constants-registry.md) that pass test 2 below |
| **TEMPERAMENT** | what makes this *individual*: a mechanism's strength, drawn at birth inside the SPECIES box | **no default exists, by construction** — the seed is the default | none, and that is the point: the box is swept, the point is drawn | **yes** | [03 §1.2](03-constants-registry.md)'s class column |

**The two tests, in order.** A row is TEMPERAMENT only if it passes both:

1. **The objective test (→ SPECIES).** Does any sweep's *objective* depend on
   this constant? If yes, it is SPECIES — because a per-individual value would
   make the objective a function of an individual, and the sweep would have to
   re-search for every seed. Concretely: `K_MAX_ROUNDS`, `TAU_SETTLE`,
   `FRONTIER_CAP`, `HUB_DAMP_EXPONENT`, `OSCILLATION_RULE`, `PAIR_OVERLAP_MAX`
   are all load-bearing for G2's anti-mush objective, so they are SPECIES even
   though "how wide SI spreads" reads like a personality trait.
2. **The guard test (→ SPECIES).** Does the constant exist to **bound** a
   mechanism rather than to **scale** it? Guards are never individual, because
   "every temperament can be ungoverned, and one of them will be" is R-22's
   failure with a birth certificate. `BIAS_CAP_SHARE`, `MEDIAN_MIN_N`,
   `MOD_LAG_ROUNDS`, `B_MAX`, `K_TANH`, `IDLE_PER_HOUR`, `DEDUPE_WINDOW`,
   `INFERENCE_MAX_HOPS_OWNED` are guards or bound-adjacent ⇒ SPECIES.
   **The one-line rule: temperament may bend a mechanism's strength, never a
   guard's bound.**

Anything left is TEMPERAMENT-eligible, subject to §14's ledger.

**Two consequences of class 3 that cost the build something, written down rather
than smoothed over:**

* **The sweep changes shape.** A SPECIES range is no longer "the range one value
  will be chosen from"; it is **the box every individual will be drawn from**, so
  a range is only legal if the gate legs stay green across it. §4's seed panel is
  therefore part of the sweep protocol, not a later test: the artifact records the
  objective **at the box's corners and at panel points inside it**
  ([05 §4](05-gate-harness.md)). A corner that fails is a finding about the
  *range*, never a disqualification of the seed.
* **A TEMPERAMENT row has no `default` cell.** [03 §1](03-constants-registry.md)
  says a `—` is a build blocker; for these rows the blocker is permanent and
  correct — the value is `(seed, position in DERIV_ORDER)`, which no engineer may
  write down. **An engineer who fills one in has made an individual without
  declaring one**, and G14(e) (box containment) plus G14(c) (derivation purity)
  are the checks that catch it.

## 3. The birth seed: stored where, and never re-rolled

| attribute | value | law |
|-----------|-------|-----|
| representation | `birth_seed u64le` | the same width as LoopNet's serializable `RngState.state`; no float, no string, no host entropy inside the loop |
| format version | `seed_format u16` | `0x0001`. Changing `DERIV_ORDER`, the draw count, the mapping or the grid set **bumps this and changes every hash** — the same discipline as `HASH_FORMAT` (**OD-60**), because an old session must not be re-tempered by new arithmetic |
| stored in | the `SIck`/`SIst` **header** ([01 §9](01-data-schemas.md) already lists `seed` there) and the session core header block; **not** a row and **not** a shelf ([01 §2](01-data-schemas.md) is a closed enum) | the chunk's "stored in core forever" made structural: no new `x`, no new record type |
| version-tagged | yes: `{birth_seed, seed_format, DERIV_ORDER_hash}` travel together | a replay needs all three to reproduce the individual, and needs nothing else |
| inside the state hash | yes — **item 8** ([01 §10](01-data-schemas.md), [04 §24 X.1](04-algorithms.md)) | determinism outranks the metaphor (OD-15's ruling, quoted for the same reason in §18.4): a session whose seed is outside the hash is a session whose arithmetic is unverifiable |
| never re-rolled | the codebase contains **no** re-roll path, and a resume whose header seed differs from the live run's `--seed` **refuses** | LoopNet's geometry-mismatch refusal (`ckpt.rs`) and [07 §13](07-genome-build-plan.md)'s genome-tag rule, applied to the individual instead of the base. Re-rolling mid-session would be a personality edit with no event behind it — I4's shape |
| where it comes from | `--seed 0x…` (hex, required by every gate run), or `--seed random` once at creation, **printed into the artifact header and the run directory name** | honest handling: a launcher that draws entropy silently produces a run nobody can re-create from its command line, which is exactly what §1's I5 row refuses. A gate scenario always names its seed |
| observer-only | the panel **reads** it and writes nothing; there is no setter widget and no `set_seed` surface | [08 §1/§9](08-gui-observability.md)'s OBSERVER-ONLY law; §5's no-privileged-sensor law |

## 4. One seed, two streams: the derivation, in words

**P.1 Split the seed.** `birth_seed` keys a PCG32 exactly as
`tesseract-core/src/rng.rs` does, giving a fresh `Rng` **disposed of after P.3**.
Two streams come out of the birth key and they never mix afterwards:

| stream | contents | lifetime | in the snapshot |
|--------|----------|----------|-----------------|
| **T** (temperament) | the derivation draws only | born at P.1, **discarded at P.3**; nothing can draw from it again because nothing holds it | no |
| **R** (runtime stochastic) | the ignition sampler's draws and the guess slot's nonces ([04 §19 S.3](04-algorithms.md), R11) | the whole session, advanced by every draw | yes — `RngState{state, inc, spare}` + advance history, hash item 7 (unchanged) |

**Why exactly two.** T's existence is what makes temperament *a drawing* rather
than *a dial* (a dial could be turned between thoughts; a stream held by nothing
cannot be). R's separation is what makes a re-tempered twin's ignition sequence
reproducible: the two individuals' R streams start from the same seed and
advance only on thoughts, so a difference in *which* point intrudes is a
difference in weights, never a difference in random numbers. **A third stream is
refused** (§13), and the sleep phase draws from neither (§8) — so the chunk's
"sleep seeded by the birth seed" is satisfied *indirectly and completely*: what
sleep consolidates is a lattice that the seed shaped, and sleep itself adds no
chance.

**P.2 Draw, in a fixed documented order.** The order is a sealed list named
`DERIV_ORDER`: the TEMPERAMENT rows of [03 §1.2](03-constants-registry.md), in
the registry's own reading order (section, then table row). Each entry consumes
**exactly one** draw and no other state:

```
value_i = map_next(Rng_T, legal_range_i) on grid_i        (one draw, no rejection, no clock, no rows)
```

| rule | statement | why it is load-bearing |
|------|-----------|------------------------|
| one draw per row, unconditional | the mapping is `fraction · (hi − lo)` rounded onto `grid` with round-half-up, then `+ lo` — so it **cannot** land outside the range and there is never a second draw | if a draw could be rejected, the vector would depend on a *history of rejections*, which is state, and two identical seeds would stop being twins. Determinism is bought by refusing rejection sampling here |
| pure function of the seed | no lattice, no transcript, no host, no wall clock, no `--config`, no OS entropy enters P.2 | the derivation is *birth*, not *learning*. A seed-to-value map that read the session would be a second, unprovenanced write path |
| range and grid inherited | the interval is the row's SPECIES legal range and the grid is the row's own ([01 §3](01-data-schemas.md)) | a temperament value is a **swept-kind value that came from a drawing instead of a search**: same grid, same range, same version tags, same hash treatment. This is why no new constant type is needed anywhere |
| bounded set | `|DERIV_ORDER|` = the count of TEMPERAMENT rows, asserted equal to the registry, `u8`-addressable | [05](05-gate-harness.md)'s `check_temperament_closed`: adding a TEMPERAMENT row changes the vector length ⇒ a `seed_format` bump ⇒ every hash changes ⇒ no old replay can be silently re-read under a new individual |
| no name-collision | the derived rows are addressed by **registry name and index**. `GAIN_SIGNAL[4]` is a build identifier; the chunk's word for it (*"the curiosity set point"*) is not | §18.6's naming law and `check_emotion_words` ([02 §12](02-module-contracts.md)): the six words appear only in `si-audit`, so a constant named `FRUSTRATION_THRESHOLD` would fail the build in CI, and it should |

**P.3 Discard T** and record only `{birth_seed, seed_format, the derived vector
as a hash of its own canonical byte string}`. **The vector is never stored as
state.** It is re-derived at wake, from the header, before the first round of the
first thought ([04 §25](04-algorithms.md)'s wake steps gain this as W.0), because
a stored copy is a second source of truth about the individual and would let a
container carry values its seed does not justify — G14(c) reproduces the vector
and byte-compares it against whatever the run used.

**The twin law, stated exactly (this is G14, extending G5):**

| run pair | expected |
|----------|----------|
| same seed, same transcript, same build identity | **byte-identical** `state_hash.txt`, `REPLAY_TOLERANCE = 0` — including the sleep counters, because sleep is in logical time (§7) |
| different seed, same transcript | **identical term ids, identical shelf usage, identical row types, identical pattern table, identical guard values, identical `a` at t=0 (rest)**; different values, and *possibly different row counts* |
| the seed panel (≥ 16 seeds, `SEED_PANEL_SIZE` a harness constant) over one transcript | the **union** of term ids equals the term ids of any single member |

The middle row's "possibly different row counts" is an honest weakening of G5's
no-prior twin claim, and it must not be smoothed: arousal changes the spread, the
spread changes which gaps surface, a surfaced gap changes what gets asked, and an
ask changes which later line counts as an answer-to-ask. **Temperament changes
what an individual does; it never changes what vocabulary is available to it.**
The panel publishes the count spread as an artifact
(`gates/G14/seed_panel.json`); nobody quotes it before it exists
([README rules](README.md), rule 3).

## 5. Why the seed is not the dial the design refuses

The strongest objection to this batch is that §18.1's "no privileged sensors, no
external setter" and round 1's cancelled dial panel are one concept away from a
birth seed, so the boundary is stated rather than asserted:

| the dial panel (refused, §18.1/§18.2, R-21) | the birth seed (this batch) |
|---------------------------------------------|------------------------------|
| sets `a`, a charge, a confidence or a bias, at any time, from outside | sets none of them; `a` is at rest for every seed until a signal row exists |
| writes a **state** the loop will act on | writes a **parameter** the loop was already allowed to have |
| reachable while thinking | reachable only at creation, before the first round of the first thought |
| no version tag, no hash, no artifact | version-tagged, inside the state hash, printed in every artifact header |
| a designer's judgment about what SI should feel | a uniform draw over a box the sweeps validated |
| reversible, therefore deniable | permanent, therefore citable: every `sleep_report` and every `state_hash.txt` names its seed |

**The three guards that keep it on the right side of that table, each of them a
gate leg, not a promise:** guards are never seeded (§2 test 2); every derived
value is inside a swept box (G14(e)); and every seed in the panel keeps G9's
non-degeneracy, G10(e)'s over-representation and G11's median legs green
(G14(h)) — if a corner of the box makes those fail, the *box* is wrong and the
sweep re-runs, which is precisely the discipline that keeps a temperament from
becoming an excuse for a broken constant.

One clarification so a reviewer does not mis-cite [05 §22](05-gate-harness.md):
"the harness may not seed a row" is a prohibition on **pre-creating state** (a
term, a row, an `a`). The birth seed creates nothing; it selects arithmetic. The
two are not in tension, but the words are close enough that G14(a) runs the
empty-state litmus on every seed.

## 6. Dimming: access decay, the floor, and the pilot light

**D.1 What dims, and where the number lives.** An OWNED edge's live `weight`
(u16 Q12) approaches its shelf's floor as a function of the age of
`last_touched_at`. This is **not a new quantity**: [04 §10](04-algorithms.md)
already defines `Clock::recency`'s first consumer as *"how far an edge's
`last_touched_at` has drifted toward its floor"*. This batch makes that sentence
load-bearing on the read side of the walk instead of only inside the ignition
sampler, and it is the **only** curve involved — R-29 forbids a dimming rate.

| term | form | class |
|------|------|-------|
| `d(edge)` = `DIM_BASE + (1 − DIM_BASE) · Clock::recency(now − last_touched_at)` | on the Q12 grid, reciprocal multiply, `0 < DIM_BASE ≤ d ≤ 1` | `DIM_BASE` SPECIES, swept into `gates/G13/dimming_sweep.json` |
| `w' = floor_shelf + (w − floor_shelf) · d` | one multiply, one subtract, one add, all on the grid | the pass is §8's P3; nothing else writes a weight while quiet |
| `floor_shelf` | per live belief shelf: `FLOOR_S_R`, `FLOOR_FACTS`, `FLOOR_LINKS`; `questions`/`context` have **no** floor because neither has a `z` weight | SPECIES, each swept into the same artifact, each **≥ 1** |

**D.2 Why the stored byte and not a fourth term in `W_eff` (a real choice, made
visible).** The chunk says "the FLUID weight decays toward its floor", and the
alternative — leaving `weight` alone and multiplying by recency inside
`conduction()` — is cheaper to write and wrong:

| objection to read-side dimming | the law it cites |
|-------------------------------|------------------|
| it would be a **third additive term** in `W = W ⊙ (1 + β·σ + γ·m)`, whose per-round normalization the design fixes at two terms and a cap | [docs/18 §18.5](../docs/18-emotions-engine.md), [04 §8](04-algorithms.md); OPEN-19's guard would have to be re-derived for a third share |
| the citation and the byte would diverge: the panel would show a weight the walk did not use, and the answer line would cite a row SI cannot actually reach | [08 §1](08-gui-observability.md)'s REAL BUFFERS law; I6's honesty law for the label |
| a `source`-class value that only *appears* to be stored is how a cache of a derived reading sneaks in | R-28's argument generalised: derived values must not acquire addresses |
| the design already expresses staleness in this byte — supersession sets live `weight` to 0 and history walks re-weight at their interval | [docs/05 §5.3](../docs/05-substrate-tesseract.md) |

So dimming is **stored, and written only by the sleep pass**. That single
restriction is what makes "dark = unroutable but still citable" honest: a weight
moves only at a logged, replayable, dated event, so the owner can always ask
*when* and *why* a word went quiet, and the answer is a `sleep.jsonl` line, never
"it drifted".

**D.3 The invariant the floor exists to protect.** `weight == 0` **means
superseded** — that is the whole basis of the two-cursor merge, the OWNED-vs-BASE
display ([07 §9](07-genome-build-plan.md), [08 §5](08-gui-observability.md)) and
the history-anchored walk. Therefore a live edge's floor is **≥ 1 grid unit** and
the build asserts the biconditional:

```
weight == 0  ⇔  superseded_at != i32::MAX          (live ⇒ weight ≥ floor ≥ 1)
```

`G13(d)` runs it after every sleep. Without it, dimming would silently mint fake
supersessions — a row that reads "aged" to the merge, "live" to the audit and
"gone" to the walk.

**D.4 dark ≠ deleted, and dark ≠ disbelieved.**

| may | may never |
|-----|-----------|
| go so low that a spread dies before reaching the row ⇒ the settle misses ⇒ SI asks instead of answering, or answers SPECULATION because the route is not there | lose the row, its `provenance`, its `confidence`, its stamps or its bake (all PROVENANCE/STRUCTURE, [06 §2](06-resource-budget.md)) |
| be cited by `text_ref`/`thought_id` in an audit line about what SI was told | become FACT-*ineligible*: eligibility is a provenance property (`prov_kind`, `prov_speaker`, `gain_kind`) and dimming touches none of them |
| come back: any traversal sets `last_touched_at`, so the next walk reads the row near its old strength again | have its **confidence** lowered by a sleep: §8.4's bumps are event-driven with a citation; an un-bumped confidence is a belief revised with no evidence behind it (I6, and R-07's repetition-inflation shape in reverse) |

That asymmetry — *reach decays, standing does not* — is the batch's central
design claim, and it is why the labeler needs no new label and the walk needs no
new branch.

**D.5 The pilot light, needing nothing new.** "Re-lighting = rehearsal on a baked
edge" is already implemented, twice over, by fields that exist:

1. `IGN_WEIGHT_BAKE` makes the sampler prefer points whose edges carry bake
   ([04 §19 S.3](04-algorithms.md)), so **dark baked edges are the first ones an
   idle mind reaches** — that is the pilot light seen from outside.
2. `β · σ(edge.bake, a_at, a_now)` gives the re-lit edge a conduction above its
   dimmed byte ([04 §8](04-algorithms.md)), so when it is reached it conducts as
   though it mattered.
3. Rehearsal resets the rehearsal pair, so the dim reverses (§8.4's curve, [01 §8](01-data-schemas.md)).
4. `bake` never decays (§20.4, OD-27), so (1)–(3) keep working for the life of
   the row — **and the bake's bound is what keeps the pilot light from being a
   unbeatable head start** (§5.3(e): `B_max`, saturating).

**Zero new fields, zero new outlets.** The refused alternative — deriving
`floor_shelf(edge)` from `edge.bake` — would give bake a *third* read outlet
(today: `σ`, and the ignition weight) and would make the birth-cohort hot
episode of [09 §7](09-teach-channel.md) permanent in a second dimension: not just
a head start, but immunity to going quiet. That is §5.3(e)'s "a permanent quantity
that outvotes every later thought" wearing a decay curve. **Ledger: OD-77, lean
refuse.**

## 7. The sleep phase: triggers, steps, interrupt

| item | rule | law / precedent |
|------|------|-----------------|
| what it is | an ordered pass sequence over **existing** rows, run inside the loop, in logical time | the chunk's "deterministic offline consolidation pass", with "offline" meaning *no sense input pending*, not *another process* |
| home | the new `si-sleep` crate, which **does not depend on `si-learn`** ([00 §2](00-stack-workspace.md), [02 §13](02-module-contracts.md)) | the same structural trick that makes idle read-only: with `INFERENCE_PROMOTION` off, `si-sleep ↛ si-learn` means sleep **physically cannot create a row**, exactly as `si-walk ↛ si-learn` means an idle cascade cannot call `commit()` |
| trigger 1 | every `SLEEP_EVERY_QUIET_TICKS` **quiet ticks**, counting the same predicate idle uses (§19 S.1: senses quiet ∧ agenda empty) | I5: a counter, not a clock. `quiet tick` already has one definition in the build and this batch does not add a second |
| trigger 2 | at **freeze** — i.e. before a checkpoint/snapshot is written ([04 §25](04-algorithms.md)) | the chunk's "session close" is a wall-clock event; the deterministic reading is *"sleep before you become durable"*, so the sleep is inside the snapshot and a resume never repeats it (**OD-81**) |
| never | on a wall clock, on a tick count while senses are noisy, or on a message from the panel | I5; I3's single channel; [08 §9](08-gui-observability.md)'s refused-widget table (no "sleep now" button, no `si-cli sleep`) |
| precedence | strictly **after** the idle tick and before `eventlog.flush → StateHash::compose → clock.advance` — i.e. step **12b** of [02 §11](02-module-contracts.md) | ordering law 13 and 18 are unchanged; a tick that both daydreams and sleeps does them in that order, and the hash is of the post-sleep tick (so a replay that skipped sleep diverges — which is what makes it testable) |
| steps | **every walk sleep runs is a thought and advances `t` by 1** — and the bookkeeping that makes that literal is: a sleep runs **one quiet tick per walk**, each closing with the ordinary `flush → compose → advance` and each opening with the inbox read. `clock.advance` still fires exactly once per tick ([04 §2](04-algorithms.md) note 1, unchanged); a sleep of `w` walks leaves the clock at `t₀ + w`, because it *is* `w` ticks, not one tick doing `w` things | I5, and the reason "sleep accelerates dimming honestly" is not a slogan: sleeping ages SI's own memories by real logical steps. `SLEEP_MAX_WALKS` (SPECIES) bounds how many, and §11 prices them (**OD-78**) |
| interrupt | at each **pass boundary** `si-sleep` performs the same non-blocking inbox read as tick step 1; a pending frame ⇒ abort the remaining passes, run the frame | "interruptible instantly by owner input", made deterministic: the interrupt point is a logged logical event. **Not** mid-arithmetic — a pass is the atomic unit because its writes are one FLUID sweep |
| partial sleep | legal, logged, replayable: `{t, passes_completed: [1..4], rows_touched, bytes_changed, aborted_reason}` in `runs/<session>/sleep.jsonl` | the counters `sleeps_completed` / `sleeps_interrupted` live in the session header and are hashed; `last_slept_at_t` makes resume idempotent |
| crash | sleep's durability unit is the **checkpoint**: a crash before it leaves the pre-sleep snapshot, so no half-applied rescale is ever readable | `name.tmp → sync_all → rename` ([00 §5](00-stack-workspace.md)); this is why P3's sweep must precede the checkpoint and never the reverse |
| config | runs identically in CONFIG SI and CONFIG ENGINE; it writes **only** L3 | [docs/10 §10.1](../docs/10-configs-converters.md)'s "same code, different load", and G13(e): the genome file's content tag is byte-identical after any number of sleeps — frozen layers have no FLUID column at run time ([07 §9](07-genome-build-plan.md)) |
| stage | a sleep can never change a stage, in either direction | `STAGE_THRESHOLDS` count **rows**, dimming changes no count ([04 §25.4](04-algorithms.md)); G13(m) asserts the derived stage before and after every sleep is equal — a mind that lost a stage overnight would be a stage machine reading a clock |

## 8. The four passes, in order

Fixed order, fixed membership, no staging (§13). Each pass names its reads, its
writes, its forbidden set and its artifact, so "consolidation" is a spec rather
than a mood.

| # | pass | reads | writes | may never | artifact line |
|---|------|-------|--------|-----------|---------------|
| **P1** | **REPLAY** — re-walk the sense events **new since the last sleep**, in `context` row order, each as a thought, within the sleep's walk budget (**OD-85**) | live lattice, genome (ENGINE), `Clock::recency`, R stream **not touched** | `last_touched_at` only (the 8-byte rehearsal pair — exactly [04 §19 S.7](04-algorithms.md) row 1, the write set idle already has) | reach the write gate at all: no `repeats` bump, no `confidence` bump, no `gain_exact`, no row. **Enforced by the missing edge**, and separately needed because coverage is 1.0 on a replay so §6.2's own arithmetic says BORING — and BORING's *bump* path is R-07's repetition inflation with a sleep excuse | `{pass: 1, walks, events_available, edges_touched, rows_written: 0}` |
| **P2** | **INTEGRATION** — scan the composite ledger [04 §21 U.4](04-algorithms.md) maintains (separate settled thoughts in which the same composite point re-appeared), rank by `INFERENCE_STABLE_N` | the composition tally — **derived RAM over the logged settle-sets, accumulated per thought and rebuilt at open, not a shelf and never serialized** ([04 §21](04-algorithms.md) U.4's closing paragraph: this is what lets `si-sleep` rank candidates with only core, store and walk imported, and what keeps P2 flat rather than O(session) — [06 §5.1](06-resource-budget.md)) — and `links` rows | **nothing in v1**: `INFERENCE_PROMOTION` is off (**OD-58**), and ordering law 20 says a composite reaches `learn.commit` "never — not in this statement, not in a later one" | be that later statement. If the owner rules promotion **on**, the write goes **through `si-learn::gate.commit`** (the edge would exist for this one call site), and only in the shape §9 of [09](09-teach-channel.md) forbids no part of: shelf `links`, kind `6 cooccur-weak`, `confidence = 0`, `prov_speaker = 2 system-inferred`, never FACT-eligible, never answerable (`check_candidate_unanswerable` unchanged) | `{pass: 2, candidates, stable_ge_N, rows_written: 0}` — if ruled on, `rows_written` is the only non-zero write count in this table |
| **P3** | **GLOBAL RESCALE — the SHY pass** — apply §6's dim arithmetic to every live OWNED edge, shelf by shelf, in `(x, y)` order | live edges' `weight`, `last_touched_at`, `floor_shelf`, `Clock::recency` | `weight` (u16) **only** | touch STRUCTURE or PROVENANCE — no `bake`, no stamp, no `a_at`, no `magnitude`, no `confidence`, no `superseded_*`, no offset; never brighten (§9); never take a live edge to 0 (§6.3) | `{pass: 3, edges_scanned, edges_changed, bytes_changed, max_relative_shift, within_shelf_rank_inversions, cross_shelf_rank_inversions, floor_hits}` |
| **P4** | **CALM REPROCESSING** — walk the live web **starting at rest** (`a = 0` per thought, not pinned there), in point-handle order, with **no RNG draw** (the sampler's seeded pick is replaced by a full deterministic sweep — **the sweep is full and read-only, the walks it runs are the budget's remainder, OD-85**) | the same everything P1 reads | exactly P1's set plus the rehearsal touches and, **iff OPEN-24 lands yes**, one re-stamp `src = 2` — and **not** S.7's agenda candidate: pushing one creates a `questions` row, and law 2 below promises **zero** rows ([04 §19](04-algorithms.md) S.7's closing paragraph: a ceiling is not an entitlement) | mint a stamp from a signal that did not fire: at rest, §18.1's litmus says nothing fires, so a stamp whose `magnitude` is 0 with no citing signal row would be §18.6's "label with no cited signal row" in provenance clothing. Signals that *do* fire during a calm walk (kind 3, kind 4) are stampable because they are real events | `{pass: 4, walks, points_swept, rows_read_calm_not_settleable, rows_relit, stamps_written}` + the **calm view** counts |

**Two laws about the pass set, and both are the interesting parts.**

1. **P4's calm view feeds nothing.** It is published as counts in
   `sleep.jsonl`/`G13/sleep_diff.json` and rendered as counts by the SLEEP panel
   ([08 §6 O11](08-gui-observability.md)). It may not feed a weight, a floor, the
   ignition sampler or a rescale factor — because a stored number derived from
   "how this word reads when SI is calm" is exactly R-28's fixed emotional address
   arriving through a side door, and it would be the *median* becoming state
   without being called a median. `check_calm_view_is_terminal` ([05 §20](05-gate-harness.md)).
2. **Three of four passes write only FLUID, and the fourth writes nothing.** That
   is the batch's safety property, stated as bytes: **sleep adds no row-creation
   power to the build in v1** — and "row" here means *any* shelf, so **P4 does not
   take S.7's agenda candidate**, which would be a `questions` row
   ([04 §19](04-algorithms.md) S.7's closing paragraph: a ceiling is not an
   entitlement). The one conditional exception is OPEN-24's re-stamp, which is a
   *stamp* under an existing cap and stays off until the owner rules it on; as
   specified today, **no pass writes STRUCTURE or PROVENANCE at all**. The write path keeps exactly the call sites it
   already had (§12 of [04](04-algorithms.md), the `/teach` short-circuit of
   [09](09-teach-channel.md), RE-PARSE), and I9's argument extends to sleep by the
   same reasoning it extends to idle: the passes transform rows a teaching event
   made; they do not make rows. That is also the condition [06 §2](06-resource-budget.md)'s
   STRUCTURE/FLUID/PROVENANCE partition was written to license — it says any later
   pass "may touch FLUID and nothing else", and P3 is that pass.

## 9. GLOBAL RESCALE's arithmetic, and the finding in the chunk's own sentence

The chunk asks for two properties at once: *"renormalize the live FLUID weights
… preserving relative ratios"*. On a bounded interval with a floor, **those two
requirements are mutually exclusive except trivially**, and the build has to say
so rather than ship a function that quietly drops one. There is a second
exclusion the chunk did not notice: exact ratio preservation across a whole shelf
is achievable only with a shelf-uniform factor, and a shelf-uniform factor is not
access decay — it cannot see which edge was rehearsed (row (b′) below). The design
wants decay, so the build keeps decay and reports the ordering it perturbs.

| reading | arithmetic | ratios preserved? | total mass preserved? | verdict |
|---------|-----------|-------------------|-----------------------|---------|
| (a) **one contraction map, per-edge factor** | `w' = f + (w − f)·d(edge)`, `d = DIM_BASE + (1 − DIM_BASE)·Clock::recency(now − last_touched_at)` | **yes inside a recency group** — for two edges sharing `last_touched_at`, `d` is the same number, so both the order and the headroom ratio are exact up to grid rounding. Across groups the order is *bounded*, not preserved, and the residual is reported (§9's honesty row) | **no** (mass contracts toward the floors) | **shipped** |
| (b) mass-preserving redistribution | per-row `s_i` chosen so `Σw' = Σw` | **no** — per-row scalars chosen by *mass* are the definition of a ratio change, and it brightens rows to pay for it | yes | **refused** |
| (b′) one shelf-uniform scalar | `w' = f + (w − f)·s_shelf` | yes, exactly, for every pair in the shelf | no | **refused too**, and this is the finding: a shelf-uniform factor makes dimming **blind to rehearsal**, so the pilot light (§6 D.4) would not protect the edge the owner just re-taught, and "access decay" stops meaning anything about *this* edge. The chunk's "preserving relative ratios" and its "dimming = access decay" are mutually exclusive **across** recency groups; the build keeps the second and reports the first's residual |
| (c) both, exactly | — | yes | yes | only `d ≡ 1` for every edge, i.e. the pass is a no-op |

**Why (b) is refused, and why it is worth a paragraph:** a mass-preserving
rescale *must brighten some rows* to pay for dimming others. A brightened edge
conducts better than the last teaching event earned, with no signal behind the
change, no stamp to cite, and no owner line to point at — which is
[docs/08 §8.2–§8.4](../docs/08-gain-loss.md)'s confidence arithmetic doing its work
in reverse, and R-07's inflation by a different door. Under (b) an individual's
memory of one word could be strengthened by the *volume* of its memory of
another; the design has no room for that claim and this batch does not invent
one. Under (a) **no row ever gets better because of sleep** — the pass can only
make the quiet parts quieter, and how much quieter is a function of how long
*that* edge has gone unused, which is what a night's forgetting is *for*.

The properties (a) does keep, each one a G13 leg:

| kept property | statement | leg |
|---------------|-----------|-----|
| order, exactly | within a shelf, any two live edges **with the same `last_touched_at`** keep their weight order and their headroom ratio (one `d` for both, so the map is monotone) | `G13(c)` |
| order, bounded | across recency groups an inversion is *possible* (`d` differs), so the count is a reported output of `dimming_sweep.json` next to `max_relative_shift` — suppressed counts fail the leg even when every behavioral leg is green | `G13(c)`, `G13(n)` |
| honesty of the loss | absolute ratios are **not** preserved, and cross-shelf comparisons **can** invert because floors differ per shelf. That too is reported, never suppressed: the walk's frontier trim compares activations across shelves, so this is real behavior and the owner should see its size | `G13(n)` |
| bounded step | `DIM_BASE ≤ d ≤ 1`, so no single sleep can blackout a row, and restoration by rehearsal is always possible | `G13(l)` |
| one curve | `d` is derived through `Clock::recency`, so "dimming rate" is not a new swept decay; `DIM_BASE` bounds a *share*, it is not a second τ. It is also the only per-edge factor allowed: a `d` that read `bake`, `confidence` or a stamp would be a new affective law wearing the rescale's clothes | `check_one_clock`, `check_sleep_never_brightens`, `G13(k)` |
| untouched affect | every `bake`, `a_at`, `magnitude`, stamp and provenance byte is identical after P3 | `G13(b)` — a before/after arena diff, which is [06 §2](06-resource-budget.md)'s partition made mechanically checkable |
| grid arithmetic | reciprocal multiply, round-half-up, no division, integer only (OD-45) | `G5(a)` replay |

## 10. The fingerprint, and the honest reason it is not a diagnosis

The chunk asks the GUI to show the seed as a **TEMPERAMENT FINGERPRINT**,
observer-only. The region is [08 §6 O12](08-gui-observability.md), its frame kind is
20 and its refused controls are [08 §9](08-gui-observability.md); what belongs
here is the arithmetic it displays and the two refusals attached to it:

| shown | form |
|-------|------|
| `birth_seed` | the `u64` as hex, verbatim, with `seed_format` beside it |
| the derived vector | one line per TEMPERAMENT row: **registry name**, its grid value, and its SPECIES box `[lo, hi]` — so the owner sees where in the box this individual landed. Constant names, grid values, no adjectives |
| the digest | the canonical byte string of the vector, hashed (item 9's `temperament_digest`), so a claim of "same temperament" is checkable without re-listing it and without storing the vector |
| the panel's twin | `runs/<session>/fingerprint.txt`, the same table, exported for a person without a GPU |

**Refusal 1 — no nickname in the build.** A label ("neophilic", "cautious",
"restless") is a *classification of the vector*, which means it is a readout, so
it may be produced only by `si-audit` ([02 §12](02-module-contracts.md) rule (i))
and reach the panel as a forwarded payload string, never as a viewer literal
([08 §3.2](08-gui-observability.md)). A viewer that held a temperament vocabulary
would be the emotion words wearing a personality font, and `check_emotion_words`
plus G5c's byte diff would both have to be weakened to let it in.
**Refusal 2 — the fingerprint never explains a behavior.** It is not an input to
the labeler, a gate criterion, or a claim. "SI's answer was tentative because its
`GAIN_SIGNAL[2]` is high" is a causal sentence about a parameter and would put a
constant in the control path's explanation; the honest sentence is the one the
mindlog already carries — *which signals fired, what `a` did, which edges
conducted*. The fingerprint is there so the owner can **notice patterns across
individuals**, which is a research readout, not an mechanism claim (§18.3's
`coupling_observed.json` is the same species of artifact, with the same
never-read-back law).

## 11. What the batch costs, where it is priced

Priced in [06 §3.4, §5.1, §7](06-resource-budget.md) as derived arithmetic only
(§12 of this file), and here as the list of honest costs:

| cost | shape |
|------|-------|
| sweep protocol | species ranges must be validated **across** a seed panel, not at one point: roughly `|panel|` extra runs for G9/G10/G11 at every re-sweep ([05 §4](05-gate-harness.md)) |
| two more gates, both paired runs | G13 compares a sleep against a `SLEEP-disabled` control; G14 compares a seed against a `seed-zeroed` control ⇒ four extra run directories per release candidate ([05 §2](05-gate-harness.md) V8/V9) |
| sleep's loop cost | at most `SLEEP_MAX_WALKS` full walks per sleep — P1 spends the budget first, P4 takes the remainder (**OD-85**) — each priced by [06 §4](06-resource-budget.md)'s per-thought model, plus a hash recomputation per walk ([06 §7](06-resource-budget.md)); the **sweeps** (P3's rescale, P4's selection) are a separate O(live edges) line, bounded in [06 §5.1](06-resource-budget.md) |
| the derived vector | **0 stored bytes** — re-derived at wake (P.3), so the individual costs a dozen multiplies at startup and nothing in the checkpoint |
| sleep bookkeeping | **13 mutable bytes** inside the 36-byte core block (`sleeps_completed`, `sleeps_interrupted`, `last_slept_at_t`, `sleep_pass_mask`) — [01 §13](01-data-schemas.md), priced per sleep in [06 §3.4](06-resource-budget.md) — plus one `sleep.jsonl` line per pass per sleep (a log lane, not state) |
| dimming's arithmetic | 1 read + 1 multiply + 1 write per live edge, once per sleep — the 16 B edge record is **read**, 2 B of `weight` are rewritten in place, 0 allocated ([06 §2](06-resource-budget.md)'s FLUID class) |
| a new crate | `si-sleep` + its contract row and its two missing edges ([00 §2](00-stack-workspace.md)) |

## 12. Refused by this batch, listed so nobody reintroduces them quietly

| refused | the law that refused it | where the same law already says so |
|---------|------------------------|-------------------------------------|
| **hysteresis** on any signal's rise/decay; a per-signal λ; a second integrator state | I8, [docs/18 §18.4](../docs/18-emotions-engine.md) "**Nothing survives of:** per-resident baselines, per-resident λ, CAUTION's hysteresis" — one λ, one scalar, one clamp | [03 §3](03-constants-registry.md)'s cancelled list; **OD-73** |
| a **curiosity set point** as a regulated level (homeostatic controller reading `a`, or any level feeding `budget`) | §18.7's "a third thing that modulates control is forbidden", OD-22, and `budget()`'s documented no-level contract ([02 §3](02-module-contracts.md)) | shipped instead: a static per-individual **ceiling**, `ASK_CAP_PER_INPUT`. **OD-72** |
| constants **named** after emotions (`FRUSTRATION_THRESHOLD`, `BOREDOM_WINDOW`, …) | §18.6's naming law, R-21, `check_emotion_words` | shipped instead: the audit predicate `AUDIT_THRESHOLDS` keeps its owner-undecided status (OPEN-22) and the build keeps indices. **OD-71**'s table says which is which |
| a **read-side** dimming term in `W_eff` | §18.5's two-term cap, OPEN-19, R-28, 08's REAL BUFFERS | shipped instead: stored `weight`, written only by P3. **OD-76** |
| **mass-preserving** rescale (and therefore any brightening of a row by sleep) | §8.4's event-driven confidence, R-07, I6 | shipped instead: headroom-ratio-preserving contraction. **OD-75** |
| a **bake-derived dimming floor** (bake's third read outlet) | §5.3(e) bounded permanence, OD-33's one-instant scope, [09 §7](09-teach-channel.md)'s birth cohort | shipped instead: the pilot light is `IGN_WEIGHT_BAKE` + `β·σ` + rehearsal reset, all existing. **OD-77** |
| a `src` value for sleep stamps | §18.5's `magnitude` = the firing signal's own value; a 0-magnitude stamp cites no signal (§18.6) | shipped instead: P4 reuses `src = 2` under OPEN-24, or writes nothing |
| a **third** RNG stream for sleep; any RNG draw inside a sleep pass | determinism annex §16.6, and the simplest possible replay story | shipped instead: sleep is order-only (by `t`, by `(x,y)`, by point handle). **OD-78**'s step rule is what replaces randomness with counting |
| **REM/NREM staging**, extra passes, or a pass that reads a previous pass's output | §5.1's registry law (no new structure), and a pass graph is a scheduler in disguise | v1 = four fixed passes in fixed order; staging parked. **OD-80** |
| a **"sleep now"** or **"re-roll seed"** control anywhere (panel, CLI, MCP) | I8's no-external-setter, [08 §9](08-gui-observability.md)'s refused widgets, §3's never-re-rolled rule | the trigger is a counter and a freeze; nothing else fires it |
| storing the derived vector in state | P.3; a second source of truth about the individual | re-derivation at wake, W.0 |
| a stage transition, gate criterion, or label that depends on a seed, a sleep count, or a temperament value | §11.2 and §21.6 (no non-knowledge predicate moves a stage); gates assert over behavior, not parameters | G13(m), `check_no_stage_on_supporter`'s family |
| `prov_kind` reuse for a sleep-written row (`2 reparse` for a consolidated bridge) | provenance must not lie: `2` is *defined* as RE-PARSE-derived with a `derived_off` lineage ([docs/05 §5.4](../docs/05-substrate-tesseract.md)) | if OD-58 lands on, `prov_kind` gains `5 consolidated` and the §3.2 allowlist diff carries the one new export literal. **OD-82** |

## 13. Decisions raised by this file (OD-71 … OD-85)

| id | subject | lean here |
|----|---------|-----------|
| **OD-71** | the **final class assignment** of every constant — SPECIES vs TEMPERAMENT vs LAW. The chunk says *"Mike proposes the table, owner rules"*, so the table is [03 §1.2](03-constants-registry.md), built by the two tests of §2, and the build implements it as written | ship [03 §1.2](03-constants-registry.md)'s column; a flip changes one cell and one `seed_format` tag if it moves a row into or out of `DERIV_ORDER` |
| **OD-72** | **is there ever a regulated curiosity level?** The chunk's "curiosity set point" as a homeostatic controller | **refused as a controller; shipped as a static per-individual ask ceiling.** OD-22's strict reading is what §18.4's own consequence paragraph already chose; a set point re-opens it |
| **OD-73** | hysteresis / per-signal rise-decay | **refused** (one scalar, one λ; §18.4 cancelled it by name). The rise half already exists as `GAIN_SIGNAL[k]`, so nothing is lost but the name |
| **OD-74** | "arousal curve **shape**" as temperament, against §18.1's "swept before quoted, never hand-argued" | **family = SPECIES (swept), variant selection among the sealed set the sweep publishes = TEMPERAMENT.** `MOD_CURVE_VARIANTS` is a species-chosen count; a seed may pick a member, never a shape outside the published set |
| **OD-75** | rescale semantics: ratio-preserving contraction vs mass-preserving redistribution | **(a) headroom-ratio-preserving contraction; mass not preserved, brightening refused** (§9's table) |
| **OD-76** | is dimming stored (P3) or read-side (`W_eff` third term)? | **stored, written only by the sleep pass** — the citation, the panel and the walk must read one byte |
| **OD-77** | may `floor_shelf(edge)` be derived from `edge.bake` (the pilot light as immunity)? | **refused.** bake keeps its two licensed outlets; the pilot light is the existing ignition preference |
| **OD-78** | does sleep consume logical steps? | **yes: every sleep walk is a thought and advances `t` by 1**, bounded by `SLEEP_MAX_WALKS`. The alternative (a parallel sleep-time axis) is a second clock and an I5 hole |
| **OD-79** | interrupt granularity | **pass boundary; a partial sleep is a legal, logged, replayable state.** A rescale is never half-applied |
| **OD-80** | REM/NREM-style staging | **parked; v1 = the four fixed passes in fixed order** (the chunk's own lean) |
| **OD-81** | "session close" as a trigger, in a loop that may not read a wall clock | **sleep-at-snapshot**: the freeze path runs the passes before writing the checkpoint, so the trigger is a step-anchored event and a resume never repeats it |
| **OD-82** | if OD-58 lands on, `prov_kind` gains `5 consolidated` (one more export literal, in `owner-teach`'s metadata class) | **specified inert.** Nothing ships until OD-58 is ruled; the row shape and the literal are documented so the ruling is one edit |
| **OD-83** | does the species sweep validate the **whole box**, or only the point it picks? | **the whole box** (corners + panel points), because a range that only validates its winner would let a legal seed land in an ungoverned corner. It costs `|panel|` runs per re-sweep and it is the price of individuality |
| **OD-84** | `docs/12`'s component map gains one logical module (`consolidate.py` ⇄ `si-sleep`). Engineering may not edit `docs/` silently (authority order, [README](README.md)) | **lean yes**; the alternative is folding `consolidate` into `si-learn`, which is worse: it deletes the missing edge that makes "sleep creates no row" a compile-time fact |
| **OD-85** | how many sense events does P1 replay, and how do P1 and P4 share `SLEEP_MAX_WALKS`? The chunk said "replay", the registry said "bounded", and the two disagreed until [06 §5.1](06-resource-budget.md) priced it | **replay the `context` rows new since the last sleep, P1 spending the budget first and P4 taking the remainder.** One constant, no second cap; a P4 with remainder 0 still sweeps in handle order, still reports, and writes nothing. P3's rescale and P4's *selection* sweep are never budget-bound, so no bounded night leaves an edge undimmed. Asserted by **G13(l).4** |

*Every row above is provisional: the build implements the lean, the ledger holds
the ruling, and a flip changes exactly one file in this folder — except OD-71,
which changes the registry column and `DERIV_ORDER`, and therefore every
`seed_format` tag and every hash.*
