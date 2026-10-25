# 03 — Constants Registry: every number, its owner, and the sweep that chooses it

**Rule of this file (inherited from the README):** a tunable appears here with a
unit, a quantization grid, a **legal range**, a placeholder default of `—`, the
sweep artifact that picks it, and the gate that reads the artifact. No default is
filled in before its artifact exists. Numbers that *are* quoted here are the ones
[docs/](../docs) already decided (confidence tiers, stage thresholds, novelty
tiers) — those are law, not tuning, and they are marked **LAW** in the last
column.

An engineer who invents a value for a `—` row has written a design change
through the back door; that is R-03's shape and each row names the gate that
catches it.

**Every row now also carries a class** (§1.2): **LAW** (the design fixed it),
**SPECIES** (one value for every individual, chosen by a sweep), or
**TEMPERAMENT** (one value per individual, drawn from the swept box by the birth
seed). The three-class column and the seed that fills it come from the
TEMPERAMENT/SLEEP batch and their laws are
[10 §1–§5](10-temperament-sleep.md); this file owns the assignment table, because
an assignment is an attribute of a constant and every constant lives here.

---

## 1. How a constant is declared (the pattern, in words)

Every tunable is **one version-tagged entry**, not a literal:

| attribute | value | why |
|-----------|-------|-----|
| `name` | SCREAMING_SNAKE, one per row below | greppable |
| `home` | exactly one crate + module | a second home is a second source of truth |
| `unit` | logical step / grid integer / count | **never wall-clock** for anything inside the loop (I5) |
| `grid` | from [01 §3](01-data-schemas.md) | a float constant in the hashed path is a determinism bug (§16.6) |
| `range` | the legal interval, open or closed | outside it ⇒ the run is invalid, not clamped |
| `default` | `—` (or a **LAW** value) | a `—` is a build blocker, not a suggestion |
| `sweep` | `gates/G<n>/<name>_sweep.json` | the artifact that produces the value |
| `ver` | `u16` version tag | **inside the state hash** ([01 §10](01-data-schemas.md) item 8): changing a swept constant changes the hash, so a re-sweep cannot silently alter a replayed session |

The sweep **output** files are never build **inputs** for anything except the
next run's declared constant list, and an artifact that a build reads back as a
design premise is R-33's failure shape ([gates/README.md](../gates/README.md)).

## 1.2 The class column — LAW / SPECIES / TEMPERAMENT (the whole registry)

**How to read a class.** A row is **TEMPERAMENT** only after passing both tests of
[10 §2](10-temperament-sleep.md): it must not be load-bearing for any sweep's
**objective** (else the sweep becomes a per-individual search), and it must not
exist to **bound** a mechanism (a guard may never be individual — *"temperament
may bend a mechanism's strength, never a guard's bound"*). A
**TEMPERAMENT** row has **no `default` cell at all**: its value is
`(birth_seed, position in DERIV_ORDER)`, and the `—` there is permanent and
correct rather than a build blocker. Its legal range and grid are inherited from
this file unchanged, because a temperament value is a swept-kind value that came
from a drawing instead of a search.

| constant | class | the reason, in one line |
|----------|-------|-------------------------|
| `K_MAX_ROUNDS` | SPECIES | G2's anti-mush objective reads it; an individual depth would make the objective per-individual |
| `TAU_SETTLE` | SPECIES | same; τ is what "settled" **means** in the build |
| `DECAY_BASE` | SPECIES | the walk's own decay is modulation's *target*, not its personality — individuality reaches it through `LAMBDA_A`/the curve |
| `HUB_DAMP_EXPONENT` | SPECIES | **G2's anti-mush leg (R-15) is the objective**; an individually-damped hub is an individually-defined mush |
| `FRONTIER_CAP` | SPECIES | objective + the cost model of [06 §4](06-resource-budget.md) is written in it |
| `OSCILLATION_RULE` | SPECIES | G4's pass condition; a rule is not a strength |
| `PAIR_OVERLAP_MAX` | SPECIES | it *is* a threshold in an objective — guard test |
| `PHRASE_BATCHING` | LAW | default-off is OPEN-13's ruling, not a dial |
| `INFERENCE_MAX_HOPS_OWNED` | SPECIES | bounds a **claim** ⇒ guard |
| `INFERENCE_STABLE_N` | SPECIES | a threshold over a count; OD-58 pending and [10 §8 P2](10-temperament-sleep.md) inherits it unchanged |
| `INFERENCE_PROMOTION` | ruling (OD-58) | an owner decision with two values, never a drawn one |
| `LAMBDA_A` | **TEMPERAMENT** | how fast arousal settles is the clearest individual difference in §18.4's physics, and it moves no objective — G9's legs are about *shape* (spread kept, no monotone compounding), which the box must satisfy for every draw (G14(h)) |
| `GAIN_SIGNAL[0..5]` | **TEMPERAMENT**, one draw **per index** | the chunk's "novelty sensitivity / satisfaction gain / frustration rise" — as **indices**. Six independent draws, so six individuals differ in which signals reach them; the emotion words name the *audit* pattern, never the constant (§18.6) |
| `MODCurve` (family, monotonicity) | SPECIES | §18.1: "swept before quoted, never hand-argued" — the *existence and shape* of the bend is the innate physics |
| `MOD_CURVE_VARIANT` (member index) | **TEMPERAMENT** | selection among a closed set the sweep published, never a new curve (**OD-74**) |
| `MOD_LAG_ROUNDS` | SPECIES | the one-round lag is an ordering law (law 8), not a strength; `0` and `≥ K_MAX` are fails for every individual |
| `LAG_UNIT` | SPECIES | a *unit* may not be individual: two twins whose logical units differ in kind is a second clock (R-29) |
| `BETA` (edge-bake read term) | **TEMPERAMENT** | how much a birth head-start bends conductance is a mechanism strength; its cap (`BIAS_CAP_SHARE`) is not |
| `GAMMA` (median point term) | **TEMPERAMENT** | how much a word's history colors a thought; bounded above by a species cap, and R-28 is about storage, not magnitude |
| `MEDIAN_SHAPE` | SPECIES | it decides **what counts** as a point's history; twins must be comparable or the statistic is not one statistic (OPEN-30/31) |
| `BIAS_CAP_SHARE` | SPECIES | guard (OPEN-19, R-22) |
| `B_MAX` | SPECIES | the bound on permanence is §5.3(e)'s law; sweeping it is fine, *individuating* it means some temperaments get unbeatable edges |
| `K_TANH` | SPECIES | saturation steepness changes what "bounded" costs; it belongs with `B_MAX`, which [10 §12](10-temperament-sleep.md) refuses to split |
| `MEDIAN_MIN_N` | SPECIES | threshold ⇒ guard |
| `ASK_CAP_PER_INPUT` | **TEMPERAMENT** | the chunk's "curiosity set point", shipped as a **static per-individual ceiling**, ceiling-only, never a regulated level (**OD-72**); `budget()` still reads no `a` (OD-22) |
| `GAP_SALIENCE_FLOOR` | SPECIES | a floor is a guard: below it a gap is logged, never emitted |
| `PRIORITY_WEIGHTS` (salience × recency × usage) | **TEMPERAMENT** | what an individual is curious *about* — three strengths, and G8's objective (budget respected, dedupe holds, no fabricated answers) does not read them |
| `DEDUPE_WINDOW` | SPECIES | "dedupe holds" is in G8's objective ⇒ the window is shared |
| `IDLE_RESTAMP` | ruling (OPEN-24) | a law about whether idle may stamp; not a value to draw |
| `AUDIT_THRESHOLDS` | SPECIES + ruling (OPEN-22) | the classifier's **ownership** is undecided; seeding thresholds whose rule-set has no owner compounds two unknowns, and it would make the label stream seed-dependent before OPEN-22 says who may read it |
| `LABEL_VOCAB` | LAW | six, until OPEN-22 rules |
| `MONOLOGUE_OWNER_FACING` | ruling (OPEN-23) | an I3 question about the sense channel |
| `IGN_WEIGHT_RECENCY` / `IGN_WEIGHT_BAKE` / `IGN_WEIGHT_DECAY_DEFICIT` / `IGN_WEIGHT_CONTRADICTION` | **TEMPERAMENT** ×4 | "which thought intrudes when it is quiet" is temperament's purest expression; each term is bounded (`bake ≤ B_MAX`, recency on the one curve), and G10(e) is a **box** requirement (G14(h)) rather than a point. **Ruled by the owner 2026-10-23 (OD-87)**, including the half this table could not settle: the decay-deficit **drive** is species and has **no row** — 03 holds only the multiplier — and **"no sweep may ever touch a temperament row"**, so these four are never axes of `G10/ignition_sweep.json`; the `G10` tag on their rows is the pointer to the artifact that publishes their **drawn** distribution, inert for enumeration and asserted so by `check_sweep_never_touches_temperament`. "Same constant set" in G10(e)'s clause reads **same birth seed** |
| `IDLE_PER_HOUR` | SPECIES | it bounds spinning ⇒ guard, and G10(a)'s leg is written in it |
| `IDLE_PER_TICK` | LAW | 1 |
| `MONOLOGUE_RENDER_DEPTH` | SPECIES | defined by the settle top-k, so it is not free |
| `TWO_REGIONS_TEST` | SPECIES | an undecided **rule** (OPEN-28), not a number |
| `GHOST_MAX` · `SHAPE_MAX` · `RELIGHT_MAX` · `SILT_MAX` | SPECIES ×4 | every one **bounds** a mechanism — frame scratch, the re-light fan-out, the diagnostic table — so the guard test of §1.2's header decides them: an individually-sized guard is R-22's runaway with a birth certificate (**G14(d)**) |
| `R_MIN_CRYSTALLIZE` · `R_ASK_CRYSTALLIZE` · `DELTA_MIN_CRYSTALLIZE` · `R_MIN_SHAPE` · `DELTA_MIN_SHAPE` · `K_FILLER` | SPECIES ×6 | they **are** G15(e)/G18(e)'s objectives ("the junk fixture opens zero asks", "below `K_FILLER` it is a concept, not a skill"); a per-individual recurrence bar makes the gate per-individual, and `K_FILLER`'s `lo = 2` is a law about what a skill *is* ([docs/25 §25.4](../docs/25-skills.md)) |
| `LAMBDA_SILT` | SPECIES | it is a **share of the one curve**, not a second τ: an individually-scaled recency would be R-29's second clock arriving as a personality, which is exactly how `DIM_BASE` is treated above |
| `SILT_LEVEL_MAX` · `SILT_BUMP` | SPECIES | deposit strength *and* its cap, and G16(d)'s objective is a **shape** (clears backwards, empty at `T_cold`) that must hold for every seed; a per-individual clamp makes the clearing order a per-individual claim |
| `THETA_SILT` | SPECIES | a floor ⇒ guard: below it a level **is** 0, so an individual θ would make "forgotten" mean something different per mind |
| `IGN_WEIGHT_SILT` | SPECIES, **not a 19th draw** | the only arc row that could have been temperament, and [10 §2](10-temperament-sleep.md) refuses it on arithmetic rather than taste: it would lengthen `DERIV_ORDER`, bump `SEED_FORMAT` and re-hash every run since B0 (**OD-108**, [12 §0](12-v2-arc.md) fact 6). Ships **unarmed at 0**, so it moves no seed's pick |
| `PROV_LINK_TUG` | SPECIES | it prices a route the walk **already found** — a claim weight, not a mechanism strength — and G17(c)(d) pin that it writes no stamp and no bake, which no individual may be permitted to un-pin |
| `CRYSTALLIZE` · `SILT_SPREAD_PRIMING` · `EPISODIC_TRAVERSAL` · `SKILLS` · `SKILL_COMPOSITION` · `WORLD_TIERS` (§5.2.2) | ruling (OD-97…101 / 104 / 110·111·115 / 116…119 / 124·125) | §5.2.2's own header: an owner decision with named values, `INFERENCE_PROMOTION`'s pattern; each ships at its **least-authority** value, which is what keeps every pre-arc gate byte-identical |
| `STYLE_SPREAD` · `STYLE_DEPTH` · `STYLE_LEAN` · `STYLE_RECENCY` · `STYLE_PROXIMITY` (§5.4) | **TONGUE** ×5 (the sixth class, **OD-175**) | drawn at wake from the **same** `birth_seed` over `DERIV_ORDER_TONGUE`, never stored, never hashed and **never swept** — [10 §2](10-temperament-sleep.md)'s two tests both pass (they weight a mechanism and bound none; they are in no sweep's objective), and **OD-87**'s ruled sentence ("no sweep may ever touch a temperament row") extends to them by the same argument (**OD-177**). A sixth class is what keeps `DERIV_ORDER` at its eighteen rows and `SEED_FORMAT` unmoved (**OD-165**) |
| `PHRASE_MAX_STEPS` (§5.4) | SPECIES, swept | it **bounds** the walk's own spend ⇒ guard, and `G20(i)`'s leg is written in it; its box stays open, so its sweep is a **refusal** (OD-83 unrulled, OD-172) |
| `TONGUE` (§5.4) | ruling | the switch itself, `INFERENCE_PROMOTION`'s pattern: `0` at V0, `1` at **V14** — an owner decision with two values, never a drawn one (**OD-178**) |
| `CONF_*`, `NOVELTY_TIERS`, `COVERAGE_RULE`, `BORING_RULE`, `ARBITRATION_TIERS`, `GAIN_EXACT_UNIT`, `STAGE_THRESHOLDS`, `GENOME_WEIGHT_CEILING` (§6) | LAW | §6's own header: quoted because the design decided them; **none may be seeded** — an individual confidence tier is the honesty law per-individual |
| §7's gate-internal thresholds | harness | not loop state; never hashed, never seeded |
| §8's geometry constants | schema (checked, not chosen) | a seed that changed `RECORD_FIXED_BYTES` would be a corrupt container, caught by `check_record_layout` |

**Count: 18 drawn values across 8 registry rows** — `LAMBDA_A` (1) · six
`GAIN_SIGNAL` indices (6) · `MOD_CURVE_VARIANT` (1) · `BETA` (1) · `GAMMA` (1) ·
four `IGN_WEIGHT_*` (4) · `ASK_CAP_PER_INPUT` (1) · three `PRIORITY_WEIGHTS`
components (3). A component is drawn separately because the chunk's whole point is
that the six signals and the three ranking factors do not move together. The
vector length `|DERIV_ORDER| = 18` is asserted against this table by
`check_temperament_closed` ([05 §20](05-gate-harness.md)); it is a schema fact, so
adding a TEMPERAMENT row bumps `seed_format` and therefore every hash
([10 §3](10-temperament-sleep.md)).

**The v2 arc adds 22 rows to this table and zero to that count.** Sixteen are
swept SPECIES, six are rulings, none is a draw: `|DERIV_ORDER|` stays 18,
`SEED_FORMAT` stays `0x0001`, `temperament_digest` is untouched, and the sentence
above about a TEMPERAMENT row re-hashing every run is the reason **`IGN_WEIGHT_SILT`
is not one** (§5.2.1, OD-108). What the arc *does* move is `CONST_ROWS.len()`, which
`VersionTags::encode` writes into hash item 8 — hence `CONSTS_VERSION` 0x0001 →
0x0002 and one re-pinned empty-state literal, both at **B14** and nowhere else
([12 §3.3](12-v2-arc.md)).

**The tongue round adds 8 rows and zero to that count too.** Five are
**TONGUE**-class draws, one is a swept SPECIES bound, two are Law/Ruling guard
rows; `|DERIV_ORDER|` stays 18, `SEED_FORMAT` stays `0x0001`, `temperament_digest`
and `CoreBlock` are untouched, and the five style draws ride
`DERIV_ORDER_TONGUE` — a **separate** five-entry order over the same birth seed and
a domain-tagged stream, which is exactly what keeps a future **OD-71** flip from
re-voicing every individual ([13 §0](13-tongue.md) fact 2, [13 §34.1](13-tongue.md)).
They are never swept, so no `sweep` tag points at them and `placeholder_value()`
therefore names none: the registry arithmetic that
`placeholders_match_registry()` asserts only survives with `sweep: None` on the
style rows ([13 §3.3](13-tongue.md)). `CONST_ROWS.len()` moves **95 → 103**, so
`CONSTS_VERSION` moves 0x0002 → **0x0003** at **B20**, and the three tests that pin
`EMPTY_STATE_MD5_SEED_ZERO` are re-pinned **in the same commit** as the rows
([13 §3.4](13-tongue.md)).

**The proposed-table caveat, stated on the row where it belongs.** This column is
the answer to the chunk's *"Mike proposes the table, owner rules"*: every class
above is a **proposal the build implements** and the owner may re-assign, and the
re-assignment is one cell here plus, if a row enters or leaves the drawn set, one
`seed_format` bump. **OD-71** holds the ruling.


## 2. The walk's own arithmetic (OPEN-11, OPEN-12, OPEN-13)

| name | home | unit / grid | legal range | default | sweep | gate | kind |
|------|------|-------------|-------------|---------|-------|------|------|
| `K_MAX_ROUNDS` | `si-walk` | rounds | 2 … 64 | `—` | `G2/constants_sweep.json` | G2, G4 (oscillation), G6 | swept |
| `TAU_SETTLE` | `si-walk` | Q12 activation-diff | > 0 | `—` | `G2/constants_sweep.json` | G2, G9(a) | swept |
| `DECAY_BASE` | `si-walk` | u8 1/255 | `a`'s modulation bounds it (§3) | `—` | `G2/constants_sweep.json` | G2, G9 | swept |
| `HUB_DAMP_EXPONENT` | `si-walk` | Q12 exponent | > 0 (PageRank-style) | `—` | `G2/constants_sweep.json` | **G2 anti-mush leg (R-15)** | swept |
| `FRONTIER_CAP` | `si-walk` | points | ~10² design target | `—` (target 100, **UNMEASURED**) | `G2/constants_sweep.json` | G2 asserts it, G6 measures it | swept |
| `OSCILLATION_RULE` | `si-walk` | enum | `max_k_only` \| `period_check` | `—` | forced-oscillation scene | **G4** (OPEN-11) | swept |
| `PAIR_OVERLAP_MAX` | `si-harness` | fraction of settle-set pairs | (0,1) | `—` | `G2/constants_sweep.json` | G2 anti-mush objective | swept |
| `PHRASE_BATCHING` | `si-walk` | bool | per-token first | **off** (OPEN-13: batching only if a measured leg beats it) | — | G2 | LAW (default-off) |
| `INFERENCE_MAX_HOPS_OWNED` | `si-walk` | **OWNED edges in one settle route** | 1 … `K_MAX_ROUNDS` | `—` | `G11/composition_sweep.json` | **G11(e)(i)** (COMPOSITION batch) | swept |

The **objective** of the G2 sweep is fixed by the design, not tunable:
*(zero-predicate-overlap recall stable) ∧ (mean pairwise settle-overlap across
distinct queries below `PAIR_OVERLAP_MAX`)*. A constants point chosen off the
sweep curve is a **G2 fail** by name.

### 2.1 Composition: the bound on the *claim*, not on the walk (COMPOSITION batch)

`INFERENCE_MAX_HOPS_OWNED` is the only constant in this file that gates nothing.
It does not change which edges conduct, which round stops, or what is stored; it
changes **what the labeler is allowed to say** about a settle the walk already
produced ([04 §5.4](04-algorithms.md)). It is still version-tagged and still
inside the state hash (item 8, [01 §10](01-data-schemas.md)), because a session
replayed under a different cap would print different labels — and a label that
changes on replay is a G5(a) failure even though no state byte moved.

| name | home | unit | legal range | default | sweep | gate | kind |
|------|------|------|-------------|---------|-------|------|------|
| `INFERENCE_STABLE_N` | `si-learn` | **separate settled thoughts** in which the same composite point re-appears | ≥ 2 | `—` | `G11/composition_sweep.json` | G11(h) — only meaningful if promotion lands | **OD-58**, swept |
| `INFERENCE_PROMOTION` | `si-learn` | bool | `off` \| `on` | **off** | — | G11(g) asserts `off` behavior; if ruled `on`, G11(h) + a new zero-answerability leg | **OD-58** (owner ruling PENDING) |

**The composition sweep's objective, stated so it cannot be tuned into
comfort:** report the deepest composed route the scenario actually settles at,
and pick the **smallest** cap that still passes legs (b) and (c) — a cap above
the measured composition depth of any real session is not generous, it is
untested, and leg (i)'s depth-boundary scene then has nothing to bound. The
sweep output is `gates/G11/composition_sweep.json` with `{cap, owned_hops_observed
per scenario, uncertainty_count, rows_written_during_composition (must be 0 in
every row)}`. `INFERENCE_STABLE_N` and `INFERENCE_PROMOTION` are **not** swept to
a value while the ruling is pending: the first is a number with no artifact to
produce it and the second is an owner decision, and both are recorded here as
parked, not as defaults to be filled in by whoever builds first.

## 3. Arousal and the two curves out of `a` (OPEN-16, 16a, 18, 19, 29, 30, 31, 33)

| name | home | unit / grid | legal range | default | sweep | gate | kind |
|------|------|-------------|-------------|---------|-------|------|------|
| `LAMBDA_A` | `si-affect` | u8 1/255 | (0, 1) | `—` | `G9/arousal_sweep.json` | G9(e), G5(a) | swept |
| `GAIN_SIGNAL[0..5]` | `si-affect` | u16 Q12, one per §18.1 signal kind | ≥ 0, six values | `—` | `G9/arousal_sweep.json` | G9(a) | swept |
| `MODCurve` = `a → (reach, decay, τ_settle)` | `si-affect` | three grid maps over 256 `a` levels | monotone: reach ↑, decay ↑, settle dominance ↓ | `—` | `G9/arousal_sweep.json` (leg **16a**) | G9(a) | swept |
| `MOD_LAG_ROUNDS` | `si-affect` | rounds | **1** (owner lean) | `—` until swept; `0` and `≥ K_MAX` are both **fails** | `G9/arousal_sweep.json` (OPEN-29) | **G9(a) both directions (R-27 mush / decorative layer)** | swept |
| `LAG_UNIT` | `si-affect` | enum: `per_token` \| `per_round` \| `per_thought` | one of three | lean **gains per token, decay per round** | `G9/arousal_sweep.json` | G5(a) trajectory replay | **OD-29** |
| `BETA` (edge-bake read term) | `si-affect` | u16 Q12 | ≥ 0 | `—` | `G9/arousal_sweep.json` (OPEN-18) | G11(a) | swept |
| `GAMMA` (median point term) | `si-affect` | u16 Q12 | ≥ 0 | `—` | `G11/median_sweep.json` (OPEN-30) | G11(b)(c) | swept |
| `MEDIAN_SHAPE` | `si-affect` | enum: `additive_W_eff` \| `node_threshold_shift` \| `round0_lighting` | one of three, all form-only | **undefined until the first qualifying stamp** | `G11/median_sweep.json` | G11(b)(c) | **OPEN-30/31** |
| `BIAS_CAP_SHARE` | `si-affect` | fraction of a round's conduction | (0,1) | `—` | `G9/arousal_sweep.json` (OPEN-19) | **G9(e): no stamp's share exceeds it in any round** | swept |
| `B_MAX` | `si-affect` | u16 Q12 | > 0, and it **is the ceiling every bake obeys** | `—` | `G9/arousal_sweep.json` (OPEN-33) | **G9(e) write-side bound, G10(e)** | swept |
| `K_TANH` | `si-affect` | u16 Q12 | > 0 | `—` | `G9/arousal_sweep.json` (OPEN-33) | G9(e) **non-degeneracy** | swept |
| `MEDIAN_MIN_N` | `si-affect` | stamps per point | ≥ 1 | `—` | `G11/median_sweep.json` (OPEN-31) | G11(b)(c) | swept |

**OPEN-33's objective is not "a value" (do not tune it until it looks good):**
the sweep must report the bake **distribution over the session's formed edges**
and show both (a) no edge above `B_max` and (b) the distribution **keeps spread**
across the observed `a_at` range. All-saturated ⇒ the write side encodes nothing;
all-near-zero ⇒ a calm SI bakes nothing. Both directions are findings, published,
never patched with an event-kind table (OD-31, R-21(c)).
`a_formation`'s unit **is** `LAG_UNIT`'s clock — the three legs (OPEN-29, 30, 33)
cannot be tuned apart, and a build that treats them as independent sweeps is
violating R-29 with extra steps.

**Cancelled, and listed so nobody reintroduces them:** per-resident baselines,
per-resident λ, CAUTION's hysteresis, the composition rule (OPEN-17/OD-16),
`coupling_sweep.json` (OD-21), and **any** seventh signal or eighth `a` input.
**The TEMPERAMENT batch was offered hysteresis and a per-signal rise/decay by
name and refused both** (**OD-73**,
[10 §12](10-temperament-sleep.md)): the rise half already exists — it is
`GAIN_SIGNAL[k]`, per index — and the decay half is `LAMBDA_A`, one value for all
six signals because §18.4 says so twice. A second decay per signal would be a
second integrator, which is I8 by another name.

## 4. Drive, budget, and curiosity (OPEN-15, OPEN-22, OPEN-23, OPEN-24)

| name | home | unit | legal range | default | sweep | gate | kind |
|------|------|------|-------------|---------|-------|------|------|
| `ASK_CAP_PER_INPUT` | `si-affect::drive` | integer | 1 … 4 | `—` | `G8/budget_sweep.json` (OPEN-15) | **G8(d)** | swept |
| `GAP_SALIENCE_FLOOR` | `si-affect::drive` | u16 Q12 | ≥ 0 | `—` | `G8/budget_sweep.json` | G8(d) (typo/filler floor) | swept |
| `PRIORITY_WEIGHTS` (salience × recency × usage) | `si-affect::drive` | u16 Q12 ×3 | ≥ 0 | `—` | `G8/budget_sweep.json` | G8(a)(d) | swept |
| `DEDUPE_WINDOW` | `si-affect::drive` | logical steps | > 0 | `—` | `G8/budget_sweep.json` | **G8(d): re-asking an open target ⇒ zero new question rows** | swept |
| `IDLE_RESTAMP` | `si-affect` + `si-walk` | bool | `on` (lean) \| `off` | **pending owner** | — | G10(b) measures whichever way it lands (OPEN-24) | **OPEN-24** |
| `AUDIT_THRESHOLDS` | `si-audit` | per-label predicate set | rule-set over logged signals | `—` | none (static review) | G9(b) citability | **OPEN-22** |
| `LABEL_VOCAB` | `si-audit` | closed set of **six** | six, until ruled | `—` (adding `soothed` needs OPEN-22) | — | **G9(b), R-21** | **OPEN-22** |
| `MONOLOGUE_OWNER_FACING` | `si-front` | bool | `no` (v1 lean) | **off** | — | G10(d), G12(f) | **OPEN-23** |

`budget` and `emit` read **no** affective level, ever (OD-22). A `budget()` that
takes `a` as an argument is the forbidden dial panel and G8(d)'s "must not read
`a`" assertion is written for exactly that refactor.
**`a_formation = 0 ⇒ bake = 0` is law, not tuning** (**OD-56**): the floor under
§8.4's plain init is what keeps a permanent quantity from being able to mute a
write, so a build that substitutes a neutral `0.5`-style default for a zero-arousal
formation is the doubly-load-bearing case §8.4(a) warns about.

## 5. Idle ignition (OPEN-25, 26, 27, 28)

| name | home | unit | legal range | default | sweep | gate | kind |
|------|------|------|-------------|---------|-------|------|------|
| `IGN_WEIGHT_RECENCY` | `si-walk::idle` | u16 Q12 | ≥ 0 | drawn, **no default cell** | `G10/ignition_sweep.json` — **inert metadata: never an axis (OD-87)** | G10(e) | **TEMPERAMENT draw** (OPEN-25, closed by **OD-87**) |
| `IGN_WEIGHT_BAKE` | `si-walk::idle` | u16 Q12 | ≥ 0, contribution **bounded by `B_MAX`** | drawn, **no default cell** | `G10/ignition_sweep.json` — **inert: never an axis (OD-87)**; V3 zeroes it as a **launch state**, not as a swept floor (**OD-93**) | **G10(e): the V0/V3 state-hash identity leg (OD-91)** — the old "over-representation vs a bake-disabled control" wording is routed to **OD-92** | **TEMPERAMENT draw** (OPEN-32 **closed**: reads the edge `bake`) |
| `IGN_WEIGHT_DECAY_DEFICIT` | `si-walk::idle` | u16 Q12 | ≥ 0 | drawn, **no default cell** | `G10/ignition_sweep.json` — **inert: never an axis (OD-87)** | G10(e) (the rehearsal-recovery leg, **OD-89**) | **TEMPERAMENT draw** — the *drive* it scales is SPECIES and has **no row** (OD-87) |
| `IGN_WEIGHT_CONTRADICTION` | `si-walk::idle` | u16 Q12 | ≥ 0 | drawn, **no default cell** | `G10/ignition_sweep.json` — **inert: never an axis (OD-87)** | G10(e) | **TEMPERAMENT draw** |
| `IDLE_PER_HOUR` | `si-walk` | **logical ticks**, not wall-clock | ≥ 1 | `—` | `G10/ignition_sweep.json` (OPEN-26) — **the only axis this gate owns** | **G10(a)** (spinning) | swept |
| `IDLE_PER_TICK` | `si-walk` | idle thoughts / quiet tick | **1** | 1 | — | G10(a) | **LAW** |
| `MONOLOGUE_RENDER_DEPTH` | `si-front` | lit points per entry | = settle top-k (lean) | `—` | — | G10(d) 1:1 citation | **OPEN-27** |
| `TWO_REGIONS_TEST` | `si-walk::idle` | adjacency predicate for agenda promotion | exact rule undecided | `—` | — | **G10(d)/(e)** | **OPEN-28** |

The four weights read `Clock::recency` for the three time-driven ones and
`edge.bake` for the fourth — **never a second decay curve** (R-29).

**OPEN-25 is closed by OD-87** (owner ruling, 2026-10-23): the four multipliers are
draws, the decay-deficit **drive** is species and rowless, and no sweep may own a
draw — `IDLE_PER_HOUR` is this gate's only axis, and the law is held structurally by
`check_sweep_never_touches_temperament` ([05 §20](05-gate-harness.md)).

## 5.1 Sleep, dimming, and the derivation (the TEMPERAMENT/SLEEP batch)

Laws in [10 §6–§9](10-temperament-sleep.md); the rows here, because every number
lives in this file. **Sleep draws no random number** (§10 §4's stream law), so
none of these rows touch `RngState`, and **none of them is TEMPERAMENT** — an
individual does not choose its own cadence or its own forgetting bound, because
both are guards on a write that happens while nobody is watching.

| name | home | unit / grid | legal range | default | sweep | gate | kind / class |
|------|------|-------------|-------------|---------|-------|------|--------------|
| `SLEEP_EVERY_QUIET_TICKS` | `si-sleep` | **quiet ticks**, same predicate as `IDLE` (§19 S.1) | > `IDLE_PER_HOUR`'s window ⇒ ≥ 2 | `—` | `G13/sleep_sweep.json` | **G13(f)** (a sleep happens, and happens once) | swept / SPECIES |
| `SLEEP_MAX_WALKS` | `si-sleep` | walks per sleep, each an ordinary thought step; **one budget, spent P1 first and P4 second** (**OD-85**) | ≥ 1 | `—` | `G13/sleep_sweep.json` | **G13(l)** + [06 §5.1](06-resource-budget.md)'s cost bound | swept / SPECIES |
| `DIM_BASE` | `si-store` | u8 1/255 → Q12 share | (0, 1]: how far **one** P3 may shrink headroom | `—` | `G13/dimming_sweep.json` | **G13(c)(k)(l)**: order kept inside a recency group, one curve, no blackout | swept / SPECIES |
| `FLOOR_S_R` · `FLOOR_FACTS` · `FLOOR_LINKS` | `si-store` | u16 Q12, one per live belief shelf | **≥ 1** each (`weight == 0` means superseded — [10 §6.3](10-temperament-sleep.md)) | `—` | `G13/dimming_sweep.json` | **G13(d)** (no live edge at 0) · **G13(n)** (cross-shelf inversion count reported) | swept / SPECIES |
| `MOD_CURVE_VARIANTS` | `si-affect` | count of sealed curve members | ≥ 1 (1 ⇒ temperament has no curve lever) | `—` | `G9/arousal_sweep.json` | G9(a), **G14(e)** | swept / SPECIES (the **member index** is TEMPERAMENT, **OD-74**) |
| `DERIV_ORDER` | `si-core` | sealed list, `u8`-addressable | the registry's §1.2 TEMPERAMENT rows, in reading order | **18 values / 8 rows** | none — it is a list, not a number | **G14(c)** purity, `check_temperament_closed` | **LAW** (its length is schema) |
| `SEED_FORMAT` | `si-core` | `u16` | `0x0001` | `0x0001` | none | **G14(c)**, resume refusal | **LAW** |
| `birth_seed` | `si-core` header | `u64` | any bit pattern, `0` legal | **required per session** | none — *there is no default and there must never be one* | **G14(a)(b)(c)(f)(g)** | **TEMPERAMENT's input** |
| `seed_panel` size | `si-harness` | seeds per swept gate | ≥ 16, fixed per run, recorded in `gate_report.json` | `—` (harness) | every re-sweep | **G14(h)**, **OD-83** | harness |

**Three things this section is being explicit about:**

1. **`birth_seed` has no `default` row and no sweep**, which looks like a
   violation of §1's rule and is not: §1 forbids inventing a value before its
   artifact exists, and a seed is not a value to be chosen but a value to be
   **declared at creation**. `—` would invite an engineer to write `0` or `42`
   into a test; the honest entry is "required, and printed". A gate scenario that
   omits `--seed` fails at argument parsing, before the loop starts
   ([05 §1](05-gate-harness.md)'s build-identity block).
2. **No sleep or dimming row may be TEMPERAMENT**, by the guard test — and the
   sharper reason is that sleep is the only moment a number moves without a
   thought behind it, so its cadence and its floor are the design's *promise to
   the owner* about how fast SI goes quiet. An individual that decides that for
   itself is a memory-loss rate nobody can cite.
3. **The dimming sweep is a distribution report, not a winner search.**
   `G13/dimming_sweep.json` records, per box point: the weight histogram over
   live edges, the count of edges at floor, the cross-shelf inversion count, and
   the number of sleeps until a taught-but-unrehearsed row stops settling — the
   same non-degeneracy discipline OPEN-33 imposed on the bake
   (§3's note above), pointed at forgetting instead of at head start.


## 5.2 The v2 arc's rows — 16 swept, 6 rulings (G15–G19, [12 §3](12-v2-arc.md))

Provenance: [docs/22](../docs/22-crystallize.md)/[23](../docs/23-context.md)/[24](../docs/24-episodic.md)/[25](../docs/25-skills.md)/[26](../docs/26-world.md),
engineered at B14–B18. **These rows grow `CONST_ROWS` by 22, and that is itself a
consequence the build must not lose:** `VersionTags::encode` writes the row *count*
and every `(name, ver)` pair, so hash item 8
([01 §10](01-data-schemas.md)) re-baselines for every run in the repository —
once, at **B14**, with `CONSTS_VERSION` 0x0001 → **0x0002** and the pinned
`EMPTY_STATE_MD5_SEED_ZERO` re-recorded in the same commit. Nothing else about the
hash moves: nine items, same order.

### 5.2.1 The swept SPECIES rows (16)

Every one is `Class::Species`, `default = —` (OD-86's midpoint placeholder, tagged
`UNMEASURED_PLACEHOLDER`) **except `IGN_WEIGHT_SILT`**, whose shipped default is a
declared `0`. The `hi` bound is the type maximum wherever the box is open, because
**OD-83's enumeration is unrulled** — so each of the four artifacts below publishes
a *refusal*, and a leg that needs a number runs at its box's `lo` and says it is a
floor (OD-95's precedent, `runner.rs::sleep_lo_point()`).

| name | home | unit / grid | legal range | default | sweep | gate | kind |
|------|------|-------------|-------------|---------|-------|------|------|
| `GHOST_MAX` | `si-walk` (frame scratch) | count | 0 … `u16::MAX` | `—` | `G15/recurrence_sweep.json` | G15(e) | swept (guard: OD-99's arousal-scaled budget is **absent**) |
| `R_MIN_CRYSTALLIZE` | `si-crystallize` | distinct **cold** frames | 1 … `u16::MAX` | `—` | `G15/recurrence_sweep.json` | G15(e) | swept — [docs/22 §22.3](../docs/22-crystallize.md) rule 3 |
| `R_ASK_CRYSTALLIZE` | `si-crystallize` | count | 1 … `u16::MAX` | `—` | `G15/recurrence_sweep.json` | G15(a)(e) | swept; `≥ R_MIN` by the detector's own assert, never by a sweep |
| `DELTA_MIN_CRYSTALLIZE` | `si-crystallize` | logical ticks | 1 … `u16::MAX` | `—` | `G15/recurrence_sweep.json` | G15(e) | swept — "one hot afternoon cannot mint a concept" |
| `LAMBDA_SILT` | `si-silt` | u8 1/255 | 1 … 255 | `—` | `G16/silt_sweep.json` | G16(d) | swept, and a **share of the one curve**, not a second τ (§5.1's `DIM_BASE` treatment; `check_one_clock` widens to `si-silt` and still finds one recency implementation) |
| `SILT_LEVEL_MAX` | `si-silt` | u16 Q12 | 1 … 65535 | `—` | `G16/silt_sweep.json` | G16(d) | swept (`S_max`, the deposit clamp) |
| `SILT_BUMP` | `si-silt` | u16 Q12 | 1 … 65535 | `—` | `G16/silt_sweep.json` | G16(d) | swept |
| `THETA_SILT` | `si-silt` | u16 Q12 | 0 … 65535 | `—` | `G16/silt_sweep.json` | G16(d) | swept — below it a level **is** 0 and the entry drops |
| `SILT_MAX` | `si-silt` | count | 1 … 65535 | `—` | `G16/silt_sweep.json` | G16(d) | swept, and **read out by the audit**: never an eviction policy, never a ranking |
| `IGN_WEIGHT_SILT` | `si-walk::idle` | u16 Q12 | 0 … 65535 | **`0` = unarmed** (OD-108) | `G16/silt_sweep.json` — **inert: a refusal cannot arm it** | G16(a)(d) | swept SPECIES. A 19th temperament draw is **refused**: it would lengthen `DERIV_ORDER`, bump `SEED_FORMAT` and re-hash every run since B0. G16 installs `lo + grid` — the smallest non-zero on-grid step — so the term is present and numerically `0`-additive at default |
| `PROV_LINK_TUG` | `si-read` (derived index) | u16 Q12 | 0 … 65535 | `—` | `G17/prov_sweep.json` | G17(b)(c) | swept (OD-114); **reads no silt** — the tug↔silt edge is unbuilt because OD-104 *and* OD-114 both claim it |
| `RELIGHT_MAX` | `si-walk` | count per frame | 1 … 65535 | `—` | `G17/prov_sweep.json` | G17(d) | swept; over-bound ⇒ fewer re-lights, **never** a truncated walk |
| `R_MIN_SHAPE` | `si-skills` | distinct cold frames | 1 … 65535 | `—` | `G18/shape_sweep.json` | G18(e)(f) | swept — [docs/25 §25.4](../docs/25-skills.md) rule 1 |
| `DELTA_MIN_SHAPE` | `si-skills` | logical ticks | 1 … 65535 | `—` | `G18/shape_sweep.json` | G18(e) | swept — §22.3 rule 3 verbatim |
| `K_FILLER` | `si-skills` | distinct **filler sets** | 2 … 65535 | `—` | `G18/shape_sweep.json` | **G18(f)** | swept; `lo = 2` because "a move that only ever connects the same three points is a concept, not a skill" |
| `SHAPE_MAX` | `si-walk` (frame scratch) | count | 0 … 65535 | `—` | `G18/shape_sweep.json` | G18(e) | swept — the role-graph's `GHOST_MAX` twin ([docs/16 §16.8](../docs/16-think-walk-hardware.md)'s second tenant) |

### 5.2.2 The six ruling rows — the least-authority default, installable

`INFERENCE_PROMOTION`'s exact pattern: an owner decision with named values, never
a drawn one, never swept to a value. All six ship **off** (or at its least-authority
value), which is what makes every pre-arc gate replay byte-identical.

| name | home | unit | legal range | default | sweep | gate | kind |
|------|------|------|-------------|---------|-------|------|------|
| `CRYSTALLIZE` | `si-session` | bool | `off` \| `on` | **off** | — | G15 | ruling (OD-97…101) |
| `SILT_SPREAD_PRIMING` | `si-walk` | bool | `off` \| `on` | **off** (OD-104 option (a)) | — | G16(e) | ruling (OD-104). The `primed` **tag** ships regardless, so option (b) becomes a switch, not a refactor |
| `EPISODIC_TRAVERSAL` | `si-walk` | bool | `off` \| `on` | **off** | — | G17 | ruling (OD-110/111/115) |
| `SKILLS` | `si-session` | bool | `off` \| `on` | **off** (detection) | — | G18 | ruling (OD-116…119) |
| `SKILL_COMPOSITION` | `si-walk` | bool | `off` \| `on` | **off** (use) | — | G18(b) | ruling (OD-118) — detection and use are separable, so they are two rows |
| `WORLD_TIERS` | `si-core::world` | enum | `owner-only` \| `granted` | **owner-only** | — | G19 | ruling (OD-124 tier (a), OD-125 coded-not-granted) |

**The launch channel widens by exactly one axis, at B14.**
`si-cli/src/harness/runner.rs::launch_constants` currently refuses anything but
`Class::Species`, and all four arc drafts state that their gate scenario installs
`on` "through the ordinary launch-constant channel". So the class filter becomes
`Species | Ruling`, every row above gains a `match` arm, and
**`Class::Temperament`, `Class::Law` and `Class::Derived` stay refused** — OD-87's
ban ("no sweep may ever touch a temperament row") and §1's frozen-law rows keep
their exact mechanical meaning. Nothing else changes: a declared value must still
be inside its own box and on its own grid, and the run still records its version
tags in the build identity ([05 §1](05-gate-harness.md)).

**Objective, stated so no row here can be tuned into comfort.** The four recurrence
bars are swept against *their own gate's spam leg* (G15(e)/G18(e): the junk fixture
must open zero asks) and the smallest value that passes is the pick — a bar above
the measured recurrence depth of any real session is not generous, it is untested.
`SILT_*` is swept against G16(d)'s clears-backwards shape (strict last-touch drop
order, empty at `T_cold`), not against a warmth number. `PROV_LINK_TUG` and
`RELIGHT_MAX` are swept against G17(c)(d)'s structural legs (zero stamps, zero
bakes), so no tug value can buy a re-light that writes.

## 5.4 The tongue's rows — 5 draws + 3 guards, one new class, `CONSTS_VERSION` 0x0003 (G20, [13 §3](13-tongue.md))

*(numbered **5.4**, not 5.3, on purpose: the "§5.3(e)" cited elsewhere in this
folder is [docs/05 §5.3](../docs/05-substrate-tesseract.md), the bounded-permanence
law, and a second §5.3 in this file would read as that one.)*

### 5.4.1 The five style rows — class **TONGUE**, drawn, never installed, never swept

`home` is `si-core::tongue` for all five: the values are derived there and read by
`si-tongue`, which is a leaf and may not own a registry row's home. `unit` is Q12 on
every axis, `grid` 16, and the **box is `0 … 2048`** — `hi = 2048` is the largest
`map_fraction` corner that is reachable at all (**OD-161**: `debug_assert!(f < 4096)`
makes a `hi` corner reachable only while `span ≤ 2048`), and grid 16 gives 129
on-grid values inside it. `sweep` is `None` on all five, which is not an oversight
but the arithmetic: a swept row without a `law_value`/`declared` gains an
`UNMEASURED_PLACEHOLDER`, and **OD-87**/**OD-177** forbid these rows from ever being
axes. `check_style_never_installed` ([05 §20](05-gate-harness.md)) greps for it.

| name | v1-name | what a draw leans (04 §34) | drawn `0` means | gate |
|------|---------|---------------------------|-----------------|------|
| `STYLE_SPREAD` | verbosity | how far the walk continues **after** the slots are satisfiable — it prices the budget: `budget = (PHRASE_MAX_STEPS × STYLE_SPREAD) >> 12` | no extra chunk: the line is the canonical copy | G20(i) |
| `STYLE_DEPTH` | formality | `edge_kind_is_claim(kind)` arcs (kinds 0..=5) over the associative `6 cooccur-weak` one — **not** `edge_kind_is_owned`, whose last slot *is* the co-occurrence (**OD-186**): a formal individual stays on the argument | no kind preference | G20(k) |
| `STYLE_LEAN` | playfulness | the far end's `Graph::out_degree` bucket over a leaf: a playful individual wanders into the hub | no degree preference | G20(i) |
| `STYLE_RECENCY` | habit | high-`bake` and late-`edge_off` arcs over old ones — **`bake` and arena position only, never `a_at`** (**OD-182**) | no habit: a fresh and a settled edge weigh the same | G20(e) |
| `STYLE_PROXIMITY` | particle affinity | hop-1 from the anchor over hop-2 and hop-3+ (`hop_share` 4096 / 2048 / 1024) | every hop distance weighs the same | G20(i) |

The five are **additive leans in Q12**, never multipliers above or below 1.0: a
multiplier on the `0 … 2048` grid would damp `Arc::weight` — the lattice's own
quantity — and the walk would then be a function of the voice rather than the voice
a function of the walk ([13 §3.1](13-tongue.md), [13 §34.3](13-tongue.md)); the
ignition sampler's shape is the precedent, and it sums `mul_q12_u16` terms on one
`u64` accumulator. `exclamation` is **not implemented** — no substrate carries
punctuation (**OD-163**).

### 5.4.2 The three guard rows

| name | home | class | unit | legal range | default | sweep | gate | kind |
|------|------|-------|------|-------------|---------|-------|------|------|
| `DERIV_ORDER_TONGUE` | `si-core::tongue` | **LAW** | count | 5 … 5 | **5** | — | G20(a) | Law, `DERIV_ORDER`'s pattern: a length the code asserts, quoted so `check_tongue_closed` can grep it |
| `PHRASE_MAX_STEPS` | `si-tongue` | SPECIES | count | 1 … `u16::MAX` | `—` | `G20/phrase_sweep.json` — **publishes a refusal while OD-83 stands** | G20(i) | swept, and the open box is deliberate: `hi` is the type maximum because the design names no ceiling, so the runner refuses rather than inventing one (**OD-172**) |
| `TONGUE` | `si-session` | ruling | bool | `off` \| `on` | **off** (V0) | — | G20(j) | ruling (**OD-178**): inert at V0, ON at **V14** — the only additive variant in the matrix |

**The registry arithmetic, stated before the code ([13 §3.3](13-tongue.md)).** 95 →
**103** rows. `placeholders_match_registry()` keeps holding because the five style
rows carry `sweep: None`: `placeholder_count()` stays `= swept_species_count()` (only
`PHRASE_MAX_STEPS` joins the swept set, and it is one scalar SPECIES row with no
`law_value`), `temperament_count()` stays 18, and the swept-row identity gains
exactly one on both sides. `check_registry_size` prints **103**.

**The version consequence, stated before the code ([13 §3.4](13-tongue.md)).** A row
count is a hashed quantity: `CONSTS_VERSION` 0x0002 → **0x0003**, the
`EMPTY_STATE_MD5_SEED_ZERO` literal re-pinned in `si-core/tests/kernel.rs`,
`si-session/tests/seed.rs` and `si-session/tests/wake.rs` **in the same commit**, and
every recorded control under `0x0002` superseded by a **fresh run** — never by an edit
(**OD-91**).

## 6. Confidence, gain, and the tiers (**LAW values — not swept**)


Quoted because [docs/08 §8.4](../docs/08-gain-loss.md) and
[docs/06 §6.2/§6.4](../docs/06-write-path.md) decided them; a sweep may not
"optimize" them.

| name | value | grid | source |
|------|-------|------|--------|
| `CONF_INIT_T3_CORRECT` | 0.95 | hundredths 95 | §8.4 |
| `CONF_INIT_ANSWER_TO_ASK` | 0.95 | 95 | §8.4 |
| `CONF_INIT_OWNER_TEACH` | **0.95** (owner-tier init, **T2 arbitration rank**) | 95 | §8.4 + **OD-41** — the split is the ruling |
| `CONF_INIT_T2_STATEMENT` | 0.90 | 90 | §8.4 |
| `CONF_INIT_T1_INFERRED` | 0.50 … 0.60 | 50–60 | §8.4 (a range; the exact point is a build choice inside it) |
| `CONF_CAP` | **0.99** | 99 | §8.4 — 100 is unrepresentable ([01 §3](01-data-schemas.md)) |
| `CONF_BUMP_SERVED` | +0.05 | +5 | §8.4, gated `source: owner` |
| `CONF_BUMP_EXACT_REPEAT` | +0.05 | +5 | §8.4 (R-35: **owner only**) |
| `CONF_BUMP_ARBITRATION_WIN_VS_T1` | +0.10 | +10 | §8.4 |
| `PROBE_GAIN_FORMULA` | `+0.05 · min(gain, G_MAX)/G_MAX` | — | §8.4; **`G_MAX` is swept** into `G9/arousal_sweep.json`-adjacent gain normalization, it is a normalizer, not a law |
| `NOVELTY_TIERS` | `{0, 0.5, 1}` | u16 Q12 | §6.2 — three discrete cases, not a continuum |
| `COVERAGE_RULE` | 1.0 iff an active row exists at the slot | — | §6.2 |
| `BORING_RULE` | `surprise = 0 ⇒ write nothing, bump only` | — | §6.2, asserted by G1 |
| `ARBITRATION_TIERS` | T3 > T2 > T1; ASK on a bare first T2 | enum | §6.4, asserted by G4 |
| `GAIN_EXACT_UNIT` | ±1 per serve / per superseding correction | i16 | §8.2 |
| `STAGE_THRESHOLDS` | S0→S1: ≥5 S-R rows ∧ ≥1 supersession · S1→S2: ≥3 fact rows across ≥2 entities · S2 exit: ≥3 taught links ∧ ≥1 recall served via one | counts | [docs/11](../docs/11-stages.md) — **row-counts only; no affect quantity and no Grimm flag may enter them** (§11.2) |
| `GENOME_WEIGHT_CEILING` | **49 hundredths** → `u8` tier max **125**, `2000` on the Q12 grid | the three grids | **derived, not measured and not swept**: `CONF_INIT_T1_INFERRED`'s floor minus one hundredth, so no frozen edge can ever conduct above the weakest owned edge at its birth ([docs/16 §16.2](../docs/16-think-walk-hardware.md), [07 §8](07-genome-build-plan.md)). Sweeping it would be choosing how much SI trusts its inheritance over what it was told — R-10 with a knob on it |

## 7. Gate-internal thresholds (harness constants, not loop constants)

| name | value | notes |
|------|-------|-------|
| `G2_T2_MIN_RECALLS` | ≥ 5 | zero token overlap on content words |
| `G4_MIN_ASK_EVENTS` | ≥ 1 | a run where every conflict auto-supersedes is a **green-looking fail** (R-05) |
| `G5_LONG_SESSION_INPUTS` | ≥ 100 inputs (G9(d), G10, G11, G12(e)) | "a long session" is a count, so it is reproducible |
| `G12_MIRROR_REPETITIONS` | "dozens" → pinned as a run parameter, recorded in `gate_report.json` | the leg asserts equality vs a stay-quiet control, so the number only needs to be fixed per run |
| `REPLAY_TOLERANCE` | **0 bytes** | `state_hash.txt` equality is exact; there is no epsilon (I5) |
| `GRID_TOLERANCE` (v2 GPU leg) | settle-set hash equal **within the grid** | bit-exact float equality is **never** claimed (§16.6) |

## 8. Substrate / geometry constants (fixed by schema, checked not swept)

| name | value | enforcement |
|------|-------|-------------|
| `RECORD_FIXED_BYTES` | **112** (80 envelope + 32 body) | `% 4 == 0` assert, mirrored in [06](06-resource-budget.md) |
| `EDGE_RECORD_BYTES` / `STAMP_BYTES` | 16 / 16 | same assert |
| `SHELF_COUNT` | 11 registered (8 real + 3 reserved) | a 12th `x` is a design change (R-12) |
| `EDGE_KIND_COUNT` | 7 OWNED + genome range | [01 §5.2](01-data-schemas.md); `4 frame-synonym` requires step provenance (R-18). The genome slots are indices into a sealed **kind table**, capped at 256 entries with a hard fail past it (**OD-62**, [07 §6](07-genome-build-plan.md)) |
| `PROV_BATCH_COUNT` | ≤ 256 (v1 ships 4: S1, S1-rel, S2, S3) | `u8` on every frozen record; a 257th batch fails M.5 rather than widening silently ([07 §10](07-genome-build-plan.md)) |
| `GENOME_REC_BYTES` | `DICT 32 · REL 16 · RULE 16` | frozen shelves carry packed records, not the 112 B envelope; `% 4 == 0` asserted ([01 §6](01-data-schemas.md), [07 §11](07-genome-build-plan.md)) |
| `SIGNAL_KIND_COUNT` | **6** | a 7th is a privileged sensor; `si-affect` panics on it (§3 of [02](02-module-contracts.md)) |
| `LABEL_COUNT` | **6** | adding a 7th word needs OPEN-22 |
| `DOOR_COUNT` | **6** (`prioritize emit budget salience update conduction`) | a 7th door is OD-20 reopened, not a refactor |
| `ORGAN_COUNT` | **6** | §2's own note: a 7th organ is an OD item, not an edit |
| `PATTERN_TABLE_SIZE` | 10 patterns (`P_ATTRIB … P_TEACH`) | §3.2 allowlist, G5c byte-diff |
| `STRING_CONSTANT_BUDGET` | exactly the §3.2 allowlist + the 2 `source` literals + the 3 TEACH literals (`/teach`, `=>`, `owner-teach`) | **G5c**, byte-exact diff; anything else on the diff is drift |
| `GENOME_VERSION_TAG` | content hash of the sorted row file | recorded in the registry ([docs/10 §10.2](../docs/10-configs-converters.md)) |

## 9. What may **never** be a constant

Listed because the tempting versions of these are exactly what the gates hunt:

| forbidden | why | gate |
|-----------|-----|------|
| any gain table keyed by **emotion name** | the cancelled matrix (OD-21) | G9 static |
| an **event-kind table** feeding `k` or `B_max` (`if contradiction: k *= 3`) | R-21(c) — round 1's dial panel wearing a saturating function | G9 costume test |
| a `soothe()` coefficient, or a "soothing" **signal kind** | R-21(d)/R-34: co-regulation upgrading itself into control | G12(d) |
| a stored per-point **median** | R-28 frozen emotional address | G5(a)/G9(c) |
| a second decay curve for affect | R-29 | G11(c) |
| a confidence decay on wall-clock | §8.4's event-driven rule + I5 | G5(a) |
| an `a_at` **multiplier** at read time | OD-33's scope: compared, never multiplied | G9 static |
| a `repeats`/conf bump reachable by a `source: grimm` line | R-35 | G12(e) |
| an embedding or float-vector similarity anywhere, "even as a helper" | I1/R-14 | **G2 static scan, zero tolerance** |
| a **swept** `GENOME_WEIGHT_CEILING`, or a tier boundary computed from the corpus's own statistics | R-10: the base's conduction strength becoming a knob is choosing inherited trust over taught knowledge; a data-derived boundary re-prices every edge when unrelated rows are added ([07 §8](07-genome-build-plan.md)) | A4 + M.10, then G6 |
| a **grouping** of genome relation names (two source names into one slot) | relabeling = judgment = hidden ontology, forbidden outright by [docs/10 §10.3](../docs/10-configs-converters.md) | A3 ([07 §6](07-genome-build-plan.md)) |
| a numeric **prune threshold** in the converter (drop edges below some score) | a drop is a content decision; OD-65 reserves the cut for the owner | A1 + the no-drop law ([07 §5](07-genome-build-plan.md)) |
| a **guard** that varies by individual — any §1.2 row of class SPECIES entering `DERIV_ORDER` | [10 §2](10-temperament-sleep.md)'s second test: "temperament may bend a mechanism's strength, never a guard's bound". One temperament out of many is always in the ungoverned corner, and R-22's runaway then has a birth certificate | **G14(d)** (every guard identical across the whole panel) |
| a **`default` birth seed** compiled into the binary, or a seed drawn from host entropy for a run that claims reproducibility | [10 §3](10-temperament-sleep.md): `--seed` is required and printed. A silent default makes every "individual" in the test corpus the same one and hides it | G14(c), [05 §1](05-gate-harness.md)'s build-identity block |
| a **confidence** written by the sleep pass | §8.4's bumps are event-driven and citable; a sleep that lowers or raises a belief's standing is a belief revised with no evidence behind it (I6, R-07 in reverse) | **G13(b)** (the before/after arena diff) |
| a **brightened** edge — any rescale that makes a row conduct better than the last teaching event earned, mass preservation included | [10 §9](10-temperament-sleep.md): (b) is refused precisely because paying for a dim with a bright is an unprovenanced gain | **G13(c)**, `G13(o)` (no weight increases during any pass) |
| a **dimming rate** of its own (a second decay constant) | R-29: dimming *is* `Clock::recency` on `last_touched_at`; `DIM_BASE` bounds a share, it is not a τ | `check_one_clock`, **G13(k)** |
| a **floor of 0** on a live shelf | `weight == 0` means **superseded** — the two-cursor merge, the OWNED-vs-BASE display and the history-anchored walk all read that byte as a discriminator, so a zero floor mints fake aging | **G13(d)**, [10 §6.3](10-temperament-sleep.md) |
| a `floor` **derived from `bake`** | bake's third read outlet; [10 §6.5](10-temperament-sleep.md) refuses it and §5.3(e)'s bounded-permanence argument is the reason | **OD-77** |
| an **identifier** named after an emotion (`CURIOSITY_SET_POINT`, `FRUSTRATION_THRESHOLD`, `BOREDOM_WINDOW`) | §18.6's naming law and R-21; the six words are `si-audit` literals only, so the build would not compile. The chunk's words name **audit patterns**; the constants keep their indices | `check_emotion_words`, [10 §4](10-temperament-sleep.md)/§12 |
| a language pack, synonym table or POS tagger inside `src/` | R-03/R-10/R-11 | G5c + G5d |
| a **19th temperament draw** for the silt ignition weight | [12 §0 fact 6](12-v2-arc.md): a draw lengthens `DERIV_ORDER`, bumps `SEED_FORMAT`, and re-hashes every run recorded since B0 — the arc would arrive as a mutation of every individual, not as a mechanism. **OD-108** keeps the question open; the row is SPECIES at `0` | `check_temperament_closed`, **G14(b)(c)**, G16(a) |
| an **arousal-scaled** ghost budget, shape budget, recurrence bar or silt deposit (`GHOST_MAX · f(a)`) | OD-99/107: the scaling is **absent from the code**, not zero. A guard whose bound breathes with the level is R-22's shape, and it would also make G15(e)'s spam bar unreadable — "zero asks" means nothing if the bar itself moved with `a` | `check_emotion_words`'s reach into the three new crates, **G15(c)**/**G18(c)**'s frame-trace-fixed legs |
| a **silt eviction policy** or a level-ranked candidate set | `SILT_MAX` is a diagnostic counter read out by the audit, never a decision: evicting by level would make the sediment a selector, which is a second ignition sampler, and the sampler is one draw per quiet tick | **G16(d)** (drop order is *last-touch*, not level), `check_silt_writes_nothing` |
| a **silt term inside the spread** (a primed point conducting better) | OD-104's option (a): the seed choice is silt's only read outlet. Priming the spread would make a per-tick RAM table a modulation input, i.e. a second curve out of `a` next to R-29's one | `check_silt_ignition_only`, **G16(e)** |
| a **stored** ghost, role-graph, co-light log, candidate table, event view or `prov-link` | R-28's shape (derived, recomputed, stored nowhere) and OD-98/103/110/117's declared hash exemption: any row on a shelf makes the arc *state*, and then a replay that skipped a frame diverges for a reason no gate can name | `check_ghost_is_frame_scoped`, `check_prov_link_not_stored`, **G15(b)**/**G16(b)**/**G17(b)**'s deletion legs |
| a **skill interpreter**, an executable skill, or a skill that reaches the sampler | [docs/25 §25.11](../docs/25-skills.md) note 2: a skill is a row and, at most, an ordering preference over candidate iteration. Execution needs an edge the graph does not contain, and OD-118's use-half is deliberately separable from detection | `si-skills ↛ {si-learn, si-walk}`, `check_skills_no_commit`, **G18(b)** |
| a **stored speaker class**, or a grant that outlives its revocation row | [docs/26 §26.13](../docs/26-world.md): `class` is a pure function of `(identity, owner_identity, grants)` computed at generation. A class that survives the owner's act is authority outliving the act that granted it — R-28 in the social channel instead of the affective one | `check_speaker_class_derived`, **G19(b)**, docs/12's rule (ix) |
| a **grant's `prov_speaker` value** | OD-135: the three existing values have ruled scopes and a granted teacher is none of them; `0` would claim the owner spoke, which is §26.3(c)'s laundering. The write is **refused**, and the refusal is G19(c)'s pass | **G19(c)**, `check_teach_literals` (the closed set stays closed) |
| a **retention window** that *causes* an idle tick, or a window wired to `sleep::due` or `clock.advance` | OD-126's suppress-only ruling: a window is a denial conjunct on the quiet-and-empty-agenda law, never a trigger — otherwise a busy server buys thinking time the quiet-tick law did not grant | `check_windows_suppress_only`, **G19(g)** |
| an **ask-count scaling** for nominations (`ASK_CAP × voices`) | OD-128: "a busier world may not buy more asks" — the cap is one species row and the nomination joins the same queue at the same cap and the same `DEDUPE_WINDOW` | ordering law 29 ([02 §11](02-module-contracts.md)), **G15(e)**/**G18(e)** |
