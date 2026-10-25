# 04 — Algorithms in Words: every procedure, its order, and its failure mode

This file turns the prose laws of [docs/05](../docs/05-substrate-tesseract.md),
[06](../docs/06-write-path.md), [07](../docs/07-read-path.md),
[08](../docs/08-gain-loss.md), [09](../docs/09-claim-labels.md),
[16](../docs/16-think-walk-hardware.md), [17](../docs/17-drive-layer-curiosity.md),
[18](../docs/18-emotions-engine.md), [19](../docs/19-idle-ignition.md),
[20](../docs/20-emotional-plasticity.md), [21](../docs/21-grimm-supporter.md)
into procedures an engineer can implement without asking a design question.
No code, no pseudo-code beyond the two equations the design itself ships
(`h_k`, `W_eff`) and the record shapes in [01](01-data-schemas.md).

**Names used below are not invented here.** Every function signature comes from
[02](02-module-contracts.md); every constant comes from
[03](03-constants-registry.md); every field and grid comes from
[01](01-data-schemas.md). Every `—` in the registry stays `—` here: this file
describes *where* a swept value is used, never what it is.

---

## 1. Two kinds of order, and why this file is ordered at all

| kind | what it protects | what happens if it is re-shuffled |
|------|-----------------|-----------------------------------|
| **determinism order** | I5 / G5(a) byte-identical replay | the state hash changes: `settle_or_stop`, `update`, normalization, quantization point, iteration orders |
| **law order** | an invariant or a gate | the law dies quietly: the teach short-circuit before the pending-ask test (I3/§1.4), `boring` before `consistency` (§6.2/§6.3), `salience` exactly once at the commit instant (**OD-33**), normalization **before** the τ test (§18.5 cap), admission **before** any `a` movement for a supporter line (R-34), the labeler **after** the settle (§9) |

An engineer may parallelize anything that is not on one of these two lists, and
may parallelize nothing that is. §26 collects all of them in one table.

**The one equation, restated once.** Round *k* of a walk computes
`h_k = decay · norm(h_{k−1} + W_eff · act_{k−1})` where
`W_eff = W ⊙ (1 + β · σ(edge.bake, a_now) + γ · m(point, a_now))`, both
`W`-terms arriving from `drive.conduction()` and never computed in
`si-walk` ([02 §4](02-module-contracts.md)). Everything in §5 is the
enumeration of that sentence into ordered steps.

---

## 2. A — the tick: the outermost procedure

`si-session`, one call per logical step. The 13-step order — plus **12b**, the
sleep entry point this batch adds — is fixed in
[02 §11](02-module-contracts.md); these are the notes that only an algorithm
file can carry.

1. **A tick with no sense line is legal and is not skipped.** Step 1 yields
   `NONE`; steps 2–4 and 8–11 are then inert (no pattern, no candidate, no
   gap handoff from input); step 5–7 do not run (nothing to light from
   outside); step 12 may run, **12b may run (§28)**, and step 13 always runs.
   `clock.advance` fires exactly once per tick, including empty ones — otherwise
   `λ_a`'s decay and `Clock::recency`'s age would be functions of input density
   rather than of logical time (**I5**, [docs/16 §16.6](../docs/16-think-walk-hardware.md)).
2. **A tick with *k* pasted lines is *k* ticks** in order, each with its own
   `context` row (OD-54, [01 §12](01-data-schemas.md)). A `/teach` or
   `/correct` inside a paste is still that line's own act.
3. **Step 3 precedes step 4** — the teach short-circuit is tested before the
   pending-ask match, because a teach is an explicit owner act and must never
   be consumed as a reply (§6.1). Consequence: a `/teach` that *also* covers
   an open ask commits as `owner-teach` with `question_id = null`, and the ask
   is resolved later by row existence (step 11's `resolve_ask` pass), not by a
   binding (G8(f)).
4. **Steps 1–11 also run for a `source: grimm` line**, with exactly two
   differences: step 4 never matches (R-32) and step 11 never receives a
   candidate from it (§21.4). The differences are `source` tests on
   pre-existing branches, not new branches — that is what keeps I3 honest.
5. **Step 12's ignition cannot reach step 11**, structurally: the missing
   `si-walk ↛ si-learn` edge ([00 §3](00-stack-workspace.md)) is the mechanical
   form of "an idle walk can NOTICE, never LEARN" (I9).
6. **Order within step 13:** event-log flush → `StateHash::compose` →
   `clock.advance`. The hash is taken *after* every write of the tick has been
   logged, so a replayed tick hashes the same bytes it originally produced
   (§12 of this file).
7. **Step 12b buys real logical time, in the only currency the loop has.**
   A sleep is not a pause inside one tick and not a wall-clock event: when
   `due()` fires, `si-sleep` drives **one quiet tick per walk it runs** — each
   with the note-1 shape (step 1 yields `NONE`, its assigned pass is the only
   work, note 6's step-13 sequence closes it). So a sleep of `w` walks leaves the
   clock at `t₀ + w` and note 1's "exactly once per tick" is *not* bent: the
   sleep is `w` ticks, not one tick doing `w` things
   ([10 §7](10-temperament-sleep.md), **OD-78**). This is also why "sleep
   accelerates dimming honestly" is arithmetic rather than a slogan — the ages
   P3 reads in `Clock::recency` are larger because of the steps sleep itself
   spent.
8. **Step 12b cannot reach step 11** either, and for the same reason: the missing
   `si-sleep ↛ si-learn` edge ([00 §2](00-stack-workspace.md) win 4). With
   `INFERENCE_PROMOTION` off, the consolidation passes have no symbol through
   which a row could be written, so I9 extends to sleep by construction and not
   by a new promise (§28, ordering law 20).
9. **Step 12b is entered only from a quiet tick**, and its first act at each pass
   boundary is the same non-blocking inbox read as step 1. A pending frame is an
   *abort*, never a wait: the remaining passes are skipped, `sleeps_interrupted`
   advances, and the frame runs as the next tick (§28.3).
10. **The v2 arc adds no tick, and only five insertions.** Steps **1–13 keep their
    numbers and order**; the arc sits at **5a** (silt's decay), **5b** (the tug
    index), **6**'s round-0 re-light, **7a**/`7b` (the two observers) and **12**'s
    fifth term ([02 §11](02-module-contracts.md), laws 27–29). Every one of them is
    a *reader*: none can write a row, so note 1's "steps 2–4 and 8–11 are then inert"
    on an empty tick stays exactly true — an empty tick deposits no silt, nominates
    nothing and re-lights nothing, and still advances the clock once.
11. **An arc-off run is the control, byte for byte.** With the three features
    compiled in and their ruling rows at their shipped defaults — `CRYSTALLIZE`,
    `SILT_SPREAD_PRIMING`, `EPISODIC_TRAVERSAL`, `SKILLS`, `SKILL_COMPOSITION` off and
    `WORLD_TIERS = owner-only` — `tugs` is empty, `silt::view` sums to 0, both
    observers return no nomination, and steps 5–13 perform the same integer
    operations in the same order as B13's. That is the mechanism the arc's control
    legs assert *equality* of ([gates/README](../gates/README.md), B7's ruling), and
    `IGN_WEIGHT_SILT = 0` is why the fifth term is an addend rather than a fork.
12. **A line's identity is read before its class, and its class is never stored.**
    Step 1's frame body carries `speaker_identity` + `conversation`
    ([01 §9](01-data-schemas.md)); `world::class` runs at generation and its result
    crosses to step 11 as an argument (§33). The order is law 2's, applied to the
    new axis: a privilege test on an unread field is the R-32 failure shape, and a
    class that survives its grant is authority outliving the owner's act.
13. **A nomination is a gap, so the ask budget is one number.** Step 8 receives
    kinds 5/6 through the same queue, the same `ASK_CAP_PER_INPUT` and the same
    `DEDUPE_WINDOW` as kinds 1–4 (law 29). Nothing in the arc can add a step that
    emits, and `drive`'s three doors (§11) stay three doors.

---

## 3. B — DECODE: surface text to hyperpoint pattern

`si-front::sense/tokenize/decode` ([02 §7](02-module-contracts.md)).

**B.1 Marker split (before any tokenizing).** The line is tested against the
three TEACH/correct literals from the §3.2 allowlist — `/correct`, `/teach`,
`=>` — as a **prefix** test on the raw line. Nothing else in the design looks at
line prefixes, and the test is upstream of `sense.py`, which is why a
`source: grimm` line shaped like `/teach … => …` is inert: the shape may be
recognized, but the privilege test that follows it fails and the line continues
as ordinary sense input (§1.5, §21.4, G12(d)). The split for a teach is at the
**first** `=>` on the line; everything before it is the pattern text, everything
after it is the response surface, stored verbatim
([01 §4.2](01-data-schemas.md) `resp_surface`).

**B.2 Tokenize.** R1: whitespace and punctuation split, lowercase applied to the
**lookup key only** — the surface bytes stay verbatim in the interner so the
owner's casing survives export. R2: the plural fold produces an **alias
candidate**, never a merge (R-04): `clas` and `class` collide on one folded key,
and the collision is recorded in the lexicon's `aliases` side list
([01 §7](01-data-schemas.md)). A token with no term record is *not* an error; it
is the input to the unknown-word ask (§7.2.2 of docs/07) and to gap G3.

**B.3 Decode.** Match the token sequence against the born-in frame set (the
`P_*` table, R3/R14 — 10 patterns, [03 §8](03-constants-registry.md)). The test
is **shape-only**: frame + slot positions, no semantics, no similarity, no
embedding (§16.3, R-14 — there is no numeric comparison in this algorithm at
all). On a match, emit `Pattern{frame, slots→term handles, pattern_id}`; the
`pattern_id` is what keys a row, never the raw text (that is why an
undecodable `/teach` left side writes zero rows even though the owner typed a
complete sentence). On no match, emit `Undecoded{named_dark}` where `named_dark`
is the longest decoded sub-slot set, or none if no slot decoded.

**B.4 Gibberish parity is an assertion, not a test case.** `what znark is floop`
and `what color is apple` must produce the same frame with different slot
term-handles; the decode step reads no table that contains an English word, so
parity is a property of B.3 rather than a hope. G5d re-runs it on replay.

**B.5 What decode never does.** It does not normalize meaning, does not
synonym-replace, does not correct spelling, does not decide whether the input is
a question worth answering (I7 — that is the drive's), and does not look at
`source` except to carry it through untouched ("trust the wire, not the words",
§21.7).

---

## 4. C — LIGHTING: round 0

`si-walk::light(pattern, word_order)`.

1. Split the pattern's tokens back into **word order** (the pattern id alone
   would lose it; order must prime — "what is" lights the frame before "apple"
   arrives, §16.1).
2. For each token in order: light its term hyperpoint with the round-0
   activation quantum; light the frame hyperpoint when the token matched a
   frame shape; a token with no hyperpoint fires **no** lighting and is recorded
   as a lighting event of kind *novelty* for §6's gather step (a token landing
   where nothing exists).
3. **`PHRASE_BATCHING` is off** ([03 §2](03-constants-registry.md), LAW):
   one pulse per token. A multi-word taught chunk that exists as a learned
   surface lights as one point *in addition to* its members, not instead of
   them — batching is OPEN-13 and may only be switched on by a measured leg.
4. **Alias sets (OD-55).** A folded key with alias candidates lights **every**
   member with the token's activation divided among them by reciprocal
   multiply, and the alias set is written into the trace. Silently picking one
   member is R-04's merge by another name.
5. Round-0 activations are placed on the Q12 grid immediately
   ([01 §3](01-data-schemas.md)); no float survives this point.
6. Empty lit set ⇒ empty settle-set with `stop_reason = NothingLit` — data, not
   an error (§1.3 of docs/01: the honest birth miss).
7. **From B16, one more round-0 source, and it is not a token.** After steps
   1–5, `tugs` (§31 AC.1) light their points at
   `mul_q12(PROV_LINK_TUG, level_of_belief)` in `(event_id, point)` order, capped
   at `RELIGHT_MAX`. The order is fixed, the arithmetic is on the grid, and the
   re-light writes nothing (§31 AC.3), so §5's eleven steps run over a larger
   round-0 lit set and nothing else. **A tug may never be the only thing lit** in
   a way that makes `NothingLit` unreachable for a novel line — an
   empty-token line with a tug re-lighting is still `NothingLit` for the *query*,
   which is what keeps §11's dark-spot predicates about the input and not about
   the index. With `EPISODIC_TRAVERSAL = off` the vector is empty and this step
   is a no-op that changes no byte.

---

## 5. D — the WALK: eleven ordered steps per round

`si-walk::walk` + `settle_or_stop`. Rounds run `k = 1 … K_MAX_ROUNDS`; each
round is exactly this sequence, and the sequence is the contract
([02 §4](02-module-contracts.md) names it as non-re-shufflable).

| # | step | detail | constant(s) it reads |
|---|------|--------|----------------------|
| 1 | **gather signal events** | the events that occurred during round *k−1* (see §6 for what counts and in what sub-order): a token landing on a dark spot, a frame lighting, a hot-baked edge conducting, a round ending with nothing above threshold. Each event is `{signal kind 0..5, magnitude, citation}` — anonymous numbers, no names | — |
| 2 | **`drive.update(events, t)`** | integrate → new `a` (u8) appended to the trajectory with the round it was produced in (§7) | `LAMBDA_A`, `GAIN_SIGNAL[0..5]`, `LAG_UNIT` |
| 3 | **advance the lagged pointer** | the `a` produced at step 2 becomes the value round *k + MOD_LAG_ROUNDS* will conduct with. Round *k* uses the `a` that existed before step 2 ran | `MOD_LAG_ROUNDS` (**OPEN-29**) |
| 4 | **read the modulation** | `MODCurve[a_used]` → this round's `(reach, decay, settle-shift)` triple. Round *k* does not see round *k+1*'s `a`, and that is the whole content of "no emotion mush" (R-27) | `MODCurve` |
| 5 | **`drive.conduction()`** | the per-edge bake term `β·σ(edge.bake, a_at, a_now)` for every edge in the current frontier, plus the per-point median term `γ·m(point, a_now)` for every touched point (§8, §9). The median is **computed here, returned, and never written** | `BETA`, `GAMMA`, `MEDIAN_SHAPE`, `MEDIAN_MIN_N` |
| 6 | **spread** | iterate lit points in **ascending point-handle order**; for each, iterate its live edges in **ascending arena-offset order**, capped per source to its `reach(a)` highest-`weight_eff` edges (ties by edge offset). Each contribution = `activation(source) × weight_eff × hub_damp(source_degree)` — all three factors already on the grid, combined by **reciprocal multiply** (OD-45), each product **quantized to Q12 before it is added**. Targets accumulate by integer addition in that fixed iteration order — no atomics anywhere, because a float SpMV with atomics is not replayable (§16.6) | `FRONTIER_CAP` later; `B_max`-bounded `bake` inside `weight_eff` |
| 7 | **hub damp** | `hub_damp(deg) = deg^(−HUB_DAMP_EXPONENT)` from a precomputed reciprocal table over the degree range the lattice can actually reach; `deg` = live outgoing edge count of the source point, recomputed per tick from the live web (a superseded edge is weight 0, so it is not in `deg`), genome degree included in CONFIG ENGINE | `HUB_DAMP_EXPONENT` |
| 8 | **sum + decay** | `h_k = decay · (h_{k−1} + spread)` with `decay = DECAY_BASE` shifted upward in steepness by `MODCurve[a].decay` (higher `a` ⇒ faster cooling, §16.1) | `DECAY_BASE` |
| 9 | **normalize (per round, BEFORE the settle test)** | scale the affective share so that `β·σ + γ·m` contributes at most `BIAS_CAP_SHARE` of this round's total conduction, by reciprocal multiply on the Q12 grid; the attribution ("this edge was cheap because the state matched its stamp at t=…") is written into the trace here, because an uncitable bias is an unexplained behavior change (I8) | `BIAS_CAP_SHARE` |
| 10 | **frontier trim** | keep at most `FRONTIER_CAP` points lit, selected by activation descending, ties by (first-lit round ascending, point id ascending) — this is a **trim of the propagation set**, not of the settle-set | `FRONTIER_CAP` |
| 11 | **settle test** | `diff(h_k, h_{k−1}) < TAU_SETTLE` where `diff` is **L1 over the quantized activations of the union of lit sets** and `TAU_SETTLE` is shifted by `MODCurve[a].settle` (higher `a` ⇒ lower settle dominance ⇒ the walk keeps spreading instead of committing to its first attractor). `§16.1` delegates "exact operator fixed at build": L1-sum on the grid is this file's commitment (§14 below). Settled ⇒ stop; else next round | `TAU_SETTLE`, `MODCurve` |

**Step-order note.** Steps 9 and 11 are adjacent and in that order **by law**:
normalizing after the τ test would let the affective bias move the attractor
that the cap exists to bound, and the replay of the same transcript would
settle somewhere else. This is the single most expensive reordering available
in the build.

### 5.1 Stop reasons, and what each one becomes

| stop reason | emitted as | signal fed at the next round's step 1 | downstream |
|-------------|-----------|---------------------------------------|-----------|
| `Settled` | top-k settle-set | — (unless a baked edge conducted, then that stamp's kind re-fires) | gaps (§11), label (§10) |
| `TauMiss` (rounds left, no candidate above threshold at a round end) | continue, and record the pressure | settle-failure (kind 3) | contributes to §18.1's live `a` |
| `MaxK` | **UNCERTAINTY readout**, never a silent or confident answer | settle-failure (kind 3) | §9's SPECULATION-flavored uncertainty line; G4's oscillation scene asserts this branch, not a guess |
| `NothingLit` | honest birth miss | novelty (kind 0) | ask-back or nonce (§11) |

`OSCILLATION_RULE` selects between `max_k_only` and a period check
([03 §2](03-constants-registry.md), OPEN-11); whichever the sweep lands on, the
observable contract is identical — a 2-cycle ends in an uncertainty readout and
never in a hang or an answer.

### 5.2 Settle-set and top-k

Sort the settled activations by (activation descending, first-lit round
ascending, point id ascending), take the top *k* — the same *k* that
`MONOLOGUE_RENDER_DEPTH` reuses for the monologue (OPEN-27's lean: the settle's
top-k, so an idle cascade costs no new selector). The **stored/hashed**
settle-set is `{(point, quantized_activation)}` **sorted by point id** — a
different order from the display order, deliberately ([01 §10](01-data-schemas.md)
item 5); an implementation that hashes the display order makes replay depend on
a rendering choice.

### 5.3 The NOW filter is structural, not a query clause

Superseded edges carry `weight = 0` forever in storage (I4), so a live walk
cannot reach them and a stale row cannot silently answer a present-tense
question (§7.3). The history-anchored walk (§13) is the only way to reach an
aged edge, and it arrives by `store.anchors`, not by a filter in the spread.

### 5.4 COMPOSITION — two taught rows, one untaught answer

**What it is.** The walk has no composition step. `apple → red` and `red → warm`
are two OWNED edges; a query that lights `apple` spreads through the first in
round 1 and the second in round 2, and `warm` arrives in the settle-set although
nobody ever taught `apple is warm`. Composition is a **free consequence of the
spread** ([docs/07 §7.3](../docs/07-read-path.md)'s mechanisms 1 and 3 walking
edges that already exist). This section exists only to pin the three things that
follow from it, none of which the build may leave to taste:

1. **The route *is* the claim.** `warm`'s citation names the traversed edge pair
   with its rows and steps, in traversal order —
   `apple@r0 →(edge of r4@s11) red →(edge of r7@s18) warm`. The **count of
   distinct cited rows in the trace is the labeler's input** (§21 U.2 predicate 2
   is literally "composed across ≥ 2 rows/bridges, all cited"), so a composed
   answer citing one row is a labeling bug, not a style slip. The hop count is
   derived from the trace and never stored.
2. **Composition writes nothing.** The composite is a read-side event: no row, no
   edge, no stamp, no `repeats` bump, no `last_confirmed_at`, no `bake`, no
   agenda push. `si-walk ↛ si-learn` ([02 §4](02-module-contracts.md)) is the
   structural guarantee and G11(g)'s zero-row export is its measurement. SI
   re-derives the answer every time it is asked, forever — that is the honest
   price of I9 and the anti-cheat law, and §21 U.4 is the one place the owner is
   asked whether to pay it differently (**OD-58**).
3. **The claim is bounded by depth.** Cooling and `K_MAX_ROUNDS` bound the walk's
   arithmetic implicitly; the bound on the *claim* is explicit and sweepable:
   `INFERENCE_MAX_HOPS_OWNED` (home `si-walk`, 1 … `K_MAX_ROUNDS`, default `—`,
   swept by G11 into `gates/G11/composition_sweep.json`, version tag inside the
   state hash). An INFERENCE whose route uses more OWNED hops than the cap
   **downgrades to the uncertainty readout** — the same honest branch as `MaxK`
   (§5.1) — and never to a truncated confident answer that cites its first two
   hops and hides the third. The cap is applied in the **labeler's citation
   check, after the settle, read from the trace**; the walk itself is unchanged.
   That placement is load-bearing: a walk that truncated *at* the cap would make
   the bound invisible in the trace, and an invisible bound is not auditable.

**Counting a hop.** One hop = one traversed **OWNED** edge in the settle trace,
in contribution order. Not counted: round-0 lighting (§4), genome edges (see
below), an edge that accumulates into the same target twice inside one round (a
re-accumulation, not a step), and hub damping's reciprocal (arithmetic, not a
traversal). Ties and trim order are §5.2's existing convention, unchanged.

**Mixed routes.** *The design does not decide whether a genome edge consumes
owned-hop budget.* Lean (a sub-question of **OD-58**, so one ruling covers both):
mixed routes are bounded by the **tighter** of the two caps —
`INFERENCE_MAX_HOPS_OWNED` for the owned edges and the existing rule-composition
depth of 1 ([01 §6](01-data-schemas.md) `RULE`, [docs/11](../docs/11-stages.md)) —
and the labeler's citation line reports both counts (`owned=2 base=1`), because a
claim that hides which web it walked is the BASE/OWNED distinction docs/09 exists
to protect. CONFIG SI has no genome at all, so legs (b)–(f) below are the same run
either way; the question only bites in ENGINE.

**Why this is not a new mechanism.** Organs stay six, doors stay six, edge kinds
stay the closed enum of [01 §5.2](01-data-schemas.md). Nothing here adds a
structure, a shelf or a signal; what this pass adds is a bound on what the
labeler may say, and the gate legs that measure whether the bound holds and
whether the negatives stay honest.

**Reading against docs/11.** `S3 REASONING` lists "compose two rows (A—B, B—C ⇒
A—C) as INFERENCE with both citations; multi-hop over `z` edges; depth cap; loop
detection" as the **v2 interface, designed not built**, and this file does not
promote S3 into v1: what S3 reserves is a *reasoning mechanism* — something that
manufactures A—C. G11(e)–(i) tests the spread that already exists, plus a cap on
the claim about it. If the owner reads the composition batch as an S3 promotion,
the diff is this paragraph, and the stage law stays row-count-only (§11.2) either
way. Loop detection remains v2-listed; the v1 guard is decay + `τ` + max-K, which
cannot hang but **can** over-claim, and the hop cap is the over-claim guard.

### 5.5 The frame log — the arc's only addition to the walk, and it is RAM

From **B14** the walk appends, at every lighting it performs — round-0 seeds
(§4), spread contributions, the frontier trim's survivors — one
`CoLight { point, role, edge_kind, round, primed }` to a **frame-local** vector.
That is the whole of the walk's change: no step in §5's table reorders, no step
reads the log, and the log's cap is `FRONTIER_CAP × rounds`, which is already
bounded (§6 row 6 of [06](06-resource-budget.md)'s shape). Ghosts (§29) and
role-graphs (§32) are then built **from** the log inside the same frame, both
capped (`GHOST_MAX`, `SHAPE_MAX`), and **all three die at settle** — nothing in
`si-store` or `si-learn` can even name the types
(`check_ghost_is_frame_scoped`, [12 §2.6](12-v2-arc.md)).

| thing | when | reads | writes | off-state |
|-------|------|-------|--------|-----------|
| co-light append | every lighting, steps 1–11 of each round | the activation the walk already computed | frame RAM only | always on; 12 B/light, logged to `mindlog.json`'s `frame_log[]` |
| tug re-light | **round 0**, before the first spread | `tugs` (§31 AC.1) — `Vec<OriginTug>` the session filled at step 5b | frame activation only: **no stamp, no bake, no confidence** | `EPISODIC_TRAVERSAL = off ⇒` empty vector ⇒ bit-identical to B13 |
| ghost build | after the settle, inside the frame, for §29 | the log | frame RAM | over `GHOST_MAX`: builds **no** ghost, so the detector simply sees fewer members |
| role-graph build | same place, for §32 | the log + first-lit rounds | frame RAM | over `SHAPE_MAX`: same degradation by omission |

**Why the log is the arc's whole input, and why that is a safety claim.** Both
observers at 7a/7b are folds over `(frame_log, store, t)`, so a detector can never
see a state the walk did not already produce, and a state the walk produces is
already hashed. Deleting the log deletes the arc without touching a byte of the
lattice — which is exactly the shape **G15(b)**/**G16(b)**/**G18(b)** exploit: remove
the reader, re-hash, assert **equality** (B7's ruling, the arc-off variant is the
control). The `primed` bit is set by a tug re-light or by silt's future option (b)
and is read by neither: with `SILT_SPREAD_PRIMING = off` it is always false, present
in the struct and inert, so **OD-104**'s option (b) is a switch rather than a
refactor and G16(e)'s cold-frame rule is testable the day it is flipped.

---

## 6. E — the signal-event table: where each of the six kinds is born inside a tick

`a` has exactly six possible inputs ([01 §5.3](01-data-schemas.md) numbers
them); a seventh is a privileged sensor and `si-affect` panics on it. **The
sub-order inside step 1's gather is fixed here** so the trajectory is
reproducible: events are appended in (round ascending, class ascending), where
class is the column order below.

| kind | born at | predicate | magnitude is | class order within a round |
|------|---------|-----------|--------------|----------------------------|
| 0 novelty-spike | `light` (a token with no hyperpoint), the walk (a frame lighting whose slot set is dark), `si-learn` step 3 (`novelty − coverage` at write) | existence tests over rows/points, no similarity | the signal's **own** value (§18.1), never `f(a)` | 1 (lighting), 3 (write-time, in its own tick) |
| 1 realized-gain | `si-learn` steps 8/9 — an ask resolved, a serve that was not corrected | row existence + the citation | its own value | 4 (post-commit, in the write tick) |
| 2 contradiction | `si-learn` step 5 (consistency) and the labeler comparing retrieved sets | flag from §6.3's five rules | count of conflicting rows, quantized | 4 |
| 3 settle-failure | `settle_or_stop` (§5.1) | stop reason | its own value | 2 (round-end) |
| 4 open-question-age | tick start, over the `questions` shelf | `now − asked_at` on the one clock | aged value via `Clock::recency` | 0 (before any lighting) |
| 5 idle-cascade-contradiction | `si-walk::idle` when a cascade crosses its own live contradiction (§19.5) | the same §6.3 test, run read-only on the cascade's settle | its own value | 2 (round-end, inside the idle walk) |

**What may not appear in this table.** No entry is keyed by an emotion name, by
an event kind (`is_trauma()`), by `source`, or by a referent; nothing here is a
setter. `arousal.py` receives numbers in the order above and returns one u8 per
call. The write-time row is the *only* entry produced outside the walk, and the
write path hands it over as `drive.update` input like any other caller (§6.1:
the surprise number **is** a signal; round 3 made it one call among many rather
than *the* call).

**And the arc adds no seventh kind, in any of its five families** (B14–B18). A
nomination (§29), a silt deposit (§30), a tug re-light (§31), a shape-forwarding
(§32) and a class derivation (§33) are **never** signal events: none of them can
move `a`, and `SIGNAL_KIND_COUNT` stays 6 — the panic on a seventh is the same
line that refuses a privileged sensor. The converse also holds and is the reason
the detectors are wordless: because `a` is only ever fed by the six rows above,
and none of those six is produced by a detector, no arc mechanism can reach the
integrator even indirectly. `check_emotion_words`'s scope growing to three more
crate directories ([12 §9 row 8](12-v2-arc.md)) is the static half of this
sentence; **G15(c)**/**G16(c)**/**G17(c)**/**G18(c)**'s frame-trace-fixed legs are
the dynamic half.

---

## 7. F — `drive.update`: the one integrator, per signal event

`si-affect::update`. Per call, in this order:

1. decay the current `a` by `LAMBDA_A` on the 1/255 grid — **decay once per
   event call**, and the unit that owns decay is `LAG_UNIT`'s ruling (**OD-29**;
   the lean is *gains per token, decay per round*, which this file implements as
   "the decay step runs once per round-boundary call even when several gain
   events land inside it"). The design does not settle the unit; **OD-29** keeps
   it open and the build keeps the branch in one function so a ruling is a
   one-line change, not a refactor.
2. add `GAIN_SIGNAL[kind] · magnitude` for each event, in gather order (§6).
3. clamp to `[0, 255]` (the design's `[0,1]`, §18.4 — the grid makes the clamp
   integer).
4. append `(a_new, round)` to the trajectory stream
   ([01 §8](01-data-schemas.md)) — **the whole sequence is the hashed artifact**;
   reproducing only the endpoint fails G5(a).
5. return `a_new`.

**Boundary conditions that are laws, not defaults.** No events and no round
boundary ⇒ no call ⇒ no change ("no tick, no signal, no change", §16.6). An
empty event list at a boundary ⇒ the *decayed* previous value, never zero and
never a reset ([02 §3](02-module-contracts.md)). `a` at t=0 in a fresh session
is 0 because nothing has fired — a non-zero birth value is smuggled content
(§3's litmus). A silent tail relaxes toward rest only over **no-thought**
intervals: an idle tick is a thought and may legitimately hold or raise `a`
(§18.4's exception, G9(e)).

---

## 8. G — `drive.conduction`: two terms, one cap, no adjectives

Called before each round (step 5 of §5). Returns, per frontier edge and per
touched point, already quantized:

**σ(edge.bake, a_at, a_now)** — the state-match term. Shape: a monotone
decreasing function of `|a_now − a_at|` on the 1/255 grids, scaled by the
edge's stored `bake`, times `BETA`. The design licenses only the direction
(*taught scared, asked scared* conducts tighter than *taught scared, asked
calm*, §20.2) and explicitly refuses the reading where `a_at` multiplies the
conduction at read time (OD-33's scope; **`a_at` is compared, never
multiplied**). The falloff's exact form is OPEN-18 → swept into
`gates/G9/arousal_sweep.json`; until that artifact exists this function is
implemented behind its own module boundary so a shape change touches no caller.

**m(point, a_now)** — the median term (§9).

Then: `W_eff = W ⊙ (1 + β·σ + γ·m)`, with `β·σ + γ·m` clipped to
`BIAS_CAP_SHARE` of the round's total conduction by reciprocal multiply. The
clip is applied to the **sum** of the two terms, not to each term in turn:
normalizing them separately would allow a total of twice the cap, which is
R-22's runaway with extra arithmetic.

**Attribution, mandatory.** Every edge whose cost moved by more than zero gets
one trace field: `{edge_off, bias_share, a_now, a_at, median_used}`. G11(a) is
measured **against** the cap using these fields; a pass that cannot cite them is
not a pass.

---

## 9. H — the median: an order statistic, recomputed, never stored

`si-affect` computes it; only `conduction()` returns it; nothing writes it
([01 §11](01-data-schemas.md) has no field for it — it is *unrepresentable*, not
disallowed).

1. **Collect** the stamp set: every charge stamp on every edge touching the
   point, read from live rows via `store.active_at(anchor)`. **OPEN-31** asks
   whether superseded edges' stamps and re-parse-inherited stamps count; the
   design ships the conservative shape and so does v1: only stamps on edges of
   rows live at the anchor, and RE-PARSE-inherited stamps are history-only
   (`src = 3`, [01 §5.3](01-data-schemas.md)) and **excluded** — including them
   would launder a fresh reading out of a re-derived row, which is exactly what
   G7's "inherit, never manufacture" rule forbids on the write side.
2. **Weight** each stamp by `Clock::recency(anchor − stamp.thought_step)` (§10)
   multiplied by its `magnitude` on the Q12 grid. Not by `a_at`, not by signal
   kind — both would make the statistic a category machine.
3. **Sort by value** (the stamp's magnitude as an integer), accumulating weight
   until it reaches half the total; the statistic is that element. It is an
   **order statistic** (§20.6 calls it that in words), so **interpolated and
   mean-style variants are out of scope** — a mean is hijackable by one hot
   stamp, which is R-26 and the entire reason the owner chose a median. Whether
   the crossing element is the lower or the upper of the two neighbours is a
   convention the design does not state: **OD-57**, lean = **lower** (the more
   conservative reading for "no word flips on one comment").
4. **`MEDIAN_MIN_N`**: fewer qualifying stamps than N ⇒ the median is
   **undefined**, and the point contributes **no term at all** — absent, not
   zero-valued (§20.3's "zero history ⇒ no reading"). This is the birth litmus
   for the statistic: in a fresh session every median is undefined and nothing
   fires.
5. **Anchor**: the same procedure evaluated with `anchor = t'` is the word's
   reading *then* (§13). Only the weights move; storage never does.
6. **Discard** at the end of the thought. A cache that survives a thought is a
   frozen emotional address (**R-28**) and G5(a)+G9(c) catch it — the check is
   that deleting `si-audit` and re-running changes no hash byte, and that no
   schema field exists to hold the value.

**Cost note.** The collection set is bounded by the frontier, not by the graph:
§5 step 5 asks for the medians of *touched* points only. An implementation that
precomputes medians for all points is both slower and a de-facto store.

---

## 10. I — `Clock::recency`: one curve, six consumers, zero alternatives

`si-core::clock` owns the only decay/recency arithmetic in the build
(R-29; [02 §1](02-module-contracts.md) makes it the single import `si-affect`
may take). The curve maps an age in **logical steps** to a weight on the
1/255 grid, monotone decreasing, rate a version-tagged swept constant.

| consumer | what it computes with the curve |
|----------|--------------------------------|
| §8.4 rehearsal / decay-deficit | how far an edge's `last_touched_at` has drifted toward its floor; feeds `IGN_WEIGHT_DECAY_DEFICIT` and the §8.4 init floor |
| §19.2 ignition recency | how recently a point was lit |
| §20.3 the median | how much each stamp counts in the order statistic |
| `drive.prioritize` | gap recency in the ranking |
| §7 kind 4 | open-question age |
| §28 Z / P3 **dimming** | the same age, read the other way: a **share** scalar `d = DIM_BASE + (1 − DIM_BASE)·recency(age)`, `0 < d ≤ 1`, that contracts a live edge toward its shelf floor ([10 §6](10-temperament-sleep.md) D.1). **This batch adds no curve and no rate**: `DIM_BASE` bounds a *share*, it is not a second τ, so `check_one_clock` counts `si-sleep` as a caller of `Clock::recency` and nothing else |

**The law this file exists to enforce:** one curve, one set of parameters, all
six consumers. Two clocks would mean SI can remember a fact worse than it
fears a word, which the owner's own sentence makes impossible (§20.3). The
grid value `last_touched_at` is stored as an 8-byte pair and the curve value is
**derived on read**, never stored
([01 §8](01-data-schemas.md)) — so the curve can be re-swept without rewriting
history, and history cannot drift out of sync with it. **Sleep's dimming is the
one consumer that writes, and that exception is priced, not hidden**: the curve
stays derived, but P3 folds it into a stored FLUID byte, and the reason it may
do that — while a read-side `W_eff` term may not — is [10 §6](10-temperament-sleep.md)
D.2's four objections (**OD-76**).

---

## 11. K — gap detection, then the drive's three doors

Two readings of the same dark spot, and they must not merge (§20.1, I7).

**K.1 Live (§5 step 1).** At the instant spread dies at a token, it is an
arousal signal — kind 0 — and nothing else. It does not name a slot: the settle
has not happened, so the intersection that went dark is not yet identifiable.

**K.2 At settle (`si-read::gaps`).** Four predicates over the finished trace,
evaluated in this order, and the first match per slot wins so a slot cannot be
double-emitted:

1. **G1** — a term referenced by an active row has `defined_by == NULL`
   ([01 §7](01-data-schemas.md)'s load-bearing null).
2. **G2** — a parsed question lights the entity but its field/frame
   neighborhood stays dark while ≥ 2 candidate edges exist (ambiguous field ⇒
   needs a taught link).
3. **G3** — a term pulsed in ≥ 2 owner utterances and present in no row:
   pulsed, never spread from. **Owner utterances only** — counting a `source:
   grimm` mention here would let rehearsal mint curiosity (§21.4; the supporter
   batch's asymmetry, L5, cuts the other way for *tokens*, but a *gap* is
   evidence and evidence is provenanced).
4. **G4** — a conflict pending resolution (the §12 ASK branch).

Named-vs-unnamed is a property of the pair (decode result, dark intersection):
decode succeeded **and** the walk died at the intersection ⇒ `named_dark =
Some(slot)` built only from the owner's own decoded tokens (§7.1); otherwise
`None`. Naming a slot is not knowledge and asserts nothing (I6).

**K.2a The arc's two kinds are not predicates of this list.** `gap_kind = 5
cluster-nomination` and `6 move-nomination` are minted at steps **7a/7b** from a
recurrence over *frames*, not from a dark intersection in one trace (§29 AA.4,
§32 AD.3), and they enter at step 8's **queue**, after K.3 and K.4 — they are not
evaluated inside K.2, and the "first match per slot wins" rule is a rule about a
slot a walk went dark at, which a nomination never names (`target_slot = 0`). The
ordering law that makes this safe is law 29: a nomination is a `Gap`, so it passes
the same floor, the same cap and the same dedupe window as G1–G4, and no amount of
detector activity buys an extra ask. K.4's dedupe test still applies verbatim: a
nomination whose member-set is already an open ask mints **zero** rows.

**K.3 `drive.prioritize(gaps, now)`.** Rank by `PRIORITY_WEIGHTS`: gap
salience × recency × usage, each on the Q12 grid, multiplied, ties by (slot
term-handle ascending). Salience here is **informational**, not affective: it is
the count of rows and pulses bearing on the slot, and the door takes no `a`
argument (OD-22 holds for this door; the third column of §17.1 is the evidence).

**K.4 `drive.budget()`.** Returns a per-input ceiling and the dedupe rule. Pure
queue arithmetic: `ASK_CAP_PER_INPUT`, `DEDUPE_WINDOW`, `GAP_SALIENCE_FLOOR`
(below it ⇒ logged as a gap, never emitted — that is the typo/filler case), and
a dedupe test against the **open** question rows ([01 §6](01-data-schemas.md)'s
`questions` body). Re-asking an open target must mint **zero** rows (G8(d)); the
test is on `(target_slot, status = open)`, so an expired or resolved ask may be
re-raised, and that is the only way a slot re-enters the queue.

**K.5 `drive.emit(ranked)`.** For each gap that survives the budget, compose the
question **surface** by reverse-decoding the named slot through tokens the owner
already supplied (never a template with a vocabulary of its own — §17.4's
anti-cheat), append one `questions` row `{kind, target_slot, asked_at,
asked_by_thought, arousal_at_ask = a_now (u8; 255 only when there was no
thought), status = open}`, and hand the surface to the output lane. **Silence is
a citable state**: if nothing is emitted, the mind log records which rule
suppressed it — cap reached, below salience floor, already open. The old "the
level was low" excuse is gone (§17.3), and a build that writes an affective
reason string here is failing G8(d)'s "must not read `a`" assertion.

**K.6 The ask engine never answers.** Emitting is the last thing it does. A
question resolves only through §12's commit path or §15's idle revisit, and in
both cases because **a row now exists**, never because a walk guessed
(§17.3: no fabrication).

---

## 12. L — the write gate: the pipeline, with its seven ordering laws

`si-learn` ([02 §5](02-module-contracts.md)). Ordered, with the law that fixes
each position.

| # | step | algorithm | ordering law |
|---|------|-----------|--------------|
| 0 | privilege read | carry `source` from the receiving module (`panel` → owner, `mcp` → grimm); never re-derive it from text | **L-A: the tag is an input, not a computation** (§21.7). A classifier that "sounds like the owner" is semantics wearing provenance |
| 0b | **class arm** (B18) | `world::class(identity, owner_identity, grants)` (§33 AE.1) arrives as an **argument** from step 11's call site; a belief-row write whose class ≠ `Owner` returns `Refused{SpeakerNotRepresentable}` with a log line and **zero bytes**, before any of steps 1–10 runs | **L-G: the class is derived, never read** — no container carries it, so there is no stale grant to consult (**OD-123**, docs/12 rule (ix)); and the refusal is the honest answer while **OD-135** is open, because the three `prov_speaker` values have ruled scopes and a granted teacher is none of them. `WORLD_TIERS = owner-only` makes the arm unreachable except by a granted line that cannot exist — and G19(c) grades the refusal as its **pass condition** |
| 1 | `teach_short_circuit` | §3.B.1's shape + `source = owner` ⇒ `Teach(pattern, response)`; else `NotATeach` | **L-B: before the pending-ask test** — an explicit act is never read as a reply (§6.1) |
| 2 | `pending_ask_match` | if `source = owner`: scan open asks for `target_slot` == the line's decoded response slot ⇒ `AskMatch(question_id)`; else `NoMatch`. From B18 the scan also checks the **lane**: an `origin = 2` nomination ask matches only in the owner lane (**OD-128**) | **L-C: owner-gated** (R-32). A grimm line addressed at an open ask leaves it open; the ask's `asked_at` keeps aging and keeps firing kind 4. Kinds 5/6 name no slot (`target_slot = 0`), so they match by `question_id` in the answer-to-ask commit, never by slot |
| 3 | `candidate` | from `pattern_id` + stage: S0 ⇒ `s_r_pairs`; S1+ ⇒ `facts` (via re-parse or direct). The left side of a teach is the stimulus; its response surface is verbatim | keyed by the **decoded pattern**, never the raw text — an undecodable teach dies here with a named dark spot and **zero rows** (§1.4) |
| 4 | `surprise` | §13 below | **L-D: before consistency** — a repeat of a *conflicting* row must not be eaten as boring; the boring branch never sees a conflict because conflict is decided by slot identity, and the two tests share that computation (§6.2's last line) |
| 5 | `boring` | `surprise = 0` ⇒ `Confirm`: `repeats += 1`, `last_confirmed_at = now`, conf `+5` (hundredths), cap 99 — **and only if `source = owner`**; a grimm echo reaches §10's rehearsal curve and buys nothing durable (R-35) | the write path stops here for boring input: **no new row, ever** (G1's central assertion) |
| 6 | `consistency` | §14 | — |
| 7 | `arbitrate` | §15, only if a conflict flag | **L-E: after consistency, before commit** — there is no other place an `Ask` outcome can mean "commit nothing" |
| 8 | `commit` | build envelope+body; `conf_init` by tier ([03 §6](03-constants-registry.md)); **call `drive.salience(formation_ctx)` exactly once** (§16); `bake` onto the new primary edge, `conf = clamp(conf_init + bake, 0, 99)`; stamp `{signal, magnitude, a_at, thought_id, signal_ref}`; `store.append_row` | **L-F: the salience read happens at this instant and nowhere else** (OD-33). Reading `a` again later to re-price a stored bake fails G9's static leg; reading an event **kind** at all fails R-21(c) |
| 9 | `resolve_ask` | if the commit was an `answer-to-ask`, or if any just-committed row now covers an open ask's slot ⇒ `status = resolved`, `resolved_by = row_id`, one **event** logged | resolution is by **row existence**; a teach that closes an ask leaves its own `question_id` null (G8(f)) |
| 10 | gain ledger | §17 | last, because it needs the row id to cite |

Every rejection is a named outcome plus a reason string in the mind log. A
silent drop is the failure G1 hunts, and "silent" includes the case where the
reason is written but the row count moved anyway.

---

## 13. M — surprise = novelty − coverage

`si-learn::surprise`, over **stored rows only** (§6.2 — the pending event log is
not knowledge and must not cancel novelty).

1. `coverage` = 1.0 iff an active row exists at the same slot (matching **or**
   conflicting); else 0. Slot identity is `(key_field, key_point)` on the
   [01 §4.2](01-data-schemas.md) body, resolved through `active_at(NOW)`.
2. `novelty` ∈ {0, 0.5, 1} — three discrete cases, no continuum
   (`NOVELTY_TIERS`, LAW):
   * `0` — an active row with identical `(stim_signature ⇒ response)` (or
     `key ⇒ value`) exists;
   * `0.5` — the response is reachable via a learned bridge, same endpoint;
   * `1` — otherwise.
3. `surprise = novelty − coverage`; boring ⇔ `surprise = 0`. The interesting
   case is the owner teaching the same thing in different words: coverage 1 and
   novelty 1 differ, so `surprise > 0` and the row commits; matching coverage is
   what cancels, which is precisely §6.2's "matching-only coverage" clause
   implemented as step 1's slot test rather than a response test.
4. The computed number is handed over as a **kind-0 event** at step 4's
   boundary (§6) and is *also* the stamp's `magnitude` — one number, two uses,
   and neither is a function of `a` (OD-22 intact for this field).

---

## 14. N — consistency: five rules, in order, no sixth

`si-learn::consistency`. The set is closed (§6.3); a sixth rule is a design
change and the build should treat its own author's cleverness as a risk here.

1. Same slot, different active value ⇒ `CONFLICT`.
2. A new `P_DEF`-style taught link targets a term already bridged ⇒ flag; the
   newer owner teaching wins and the old bridge is aged.
3. System-inferred row vs owner-origin row at the same slot ⇒ flag; owner wins
   by tier, **and the flag is still recorded** — a disagreement that resolves
   quietly is the failure mode R-05 names "last-write-wins with extra steps".
4. OWNED vs BASE (CONFIG ENGINE only) ⇒ OWNED wins at NOW; **both** traces must
   reach the output ([docs/09](../docs/09-claim-labels.md)); L1/L2 files are
   never written (G6).
5. That is all. There are no "plausible values": a contradiction exists only
   relative to what was previously taught, or to BASE in ENGINE.

Output is `{Clean}` or `{Conflict(flag_id, old_row, new_candidate)}` — and the
flag list, not a single verdict, because one candidate can conflict on several
slots and every conflict needs its own `{branch, asked?, resolved_at}` record
(G4's pass condition).

---

## 15. O — arbitration: tiers first, then insistence

`si-learn::arbitrate`. **Inputs are the candidate's provenance tier, whether a
marker was present, the repeat count since the last surfaced ask, and the flag
list. There is no affective argument** (OD-24 — round 1's "CAUTION raises the
cost of assert" is cancelled; the audit layer may print *caution* over this
pattern afterwards, and the branch name is the reason).

1. **Tier from `prov_kind`** ([01 §4.1](01-data-schemas.md)): `correction` ⇒ T3;
   `owner-teach` ⇒ **owner-tier confidence init with T2 arbitration rank** — the
   split is **OD-41** and the build must implement two numbers, not one;
   `statement` ⇒ T2; `answer-to-ask` ⇒ T3-context; `reparse` ⇒ T1.
2. **Table, in this precedence** (§6.4 verbatim):
   * T3 vs anything ⇒ supersede (age the loser, record the event, cite both) —
     the correction protocol **is** the teaching act and does not re-ask;
   * T2-with-marker vs T2/T1 ⇒ supersede, branch `marker-correction`;
   * **T2 bare, first occurrence ⇒ `Ask`** — hold the candidate as a pending
     event, **commit nothing** (§4 Act 5; this is the branch G4 counts, and a
     run with zero ASK events is a fail, not a pass);
   * T2 bare, repeat ≥ 2 after that ask surfaced ⇒ supersede, branch
     `insistence`;
   * T1 vs T1 ⇒ recency within tier, tie → higher conf, tie → **lower step id**
     (all three comparisons integer, so the outcome never depends on iteration
     order);
   * system vs owner anywhere ⇒ owner wins, always, with the T1 loser aged and
     its reason string stored.
3. **The `Ask` outcome is a first-class return value**, not an error and not a
   suppression: it appends a `questions` row (kind: the §11 G4 case) so the
   held candidate is citable, and the candidate itself lives in the pending
   event log — not on a shelf. The repeat counter that branch 2's fourth row
   reads is a count over pending events for that slot, so it survives a restart
   through the log rather than through a mutable field.
4. **Aging, never deleting.** Supersede = `store.age(loser, now, winner)` ⇒
   `superseded_at = now`, `superseded_by = winner`, live `weight → 0`. The
   loser's stamps **and bake** stay (§20.4: durable ≠ currently active); the
   export prints them, the walk cannot reach them except anchored (§13).
5. **The class axis is strictly above the tier axis, and it is only an ordering
   of authority, never of strength** (B18, §33 AE.3): `Owner > Trusted >
   Stranger-sense`, and a Trusted correction supersedes **Trusted rows only** —
   it may never age an Owner row, because that would be delegated authority
   overruling the person who delegated it. This table has no fourth row for
   "Trusted beats owner" and no code path can produce one; tonight the `Trusted`
   arm is additionally unreachable by construction (`grants` empty, **OD-125**),
   and under V13 its belief write is **refused at step 11** rather than
   arbitrated (**OD-135**), so the practical content of rows 1–4 does not move.
   `repeats` and every `conf` delta stay owner-earned (R-35): a stranger or a
   granted teacher saying a thing twice buys nothing durable, which is the same
   sentence §12 step 5 already enforces, now graded over identities instead of
   over the one `source` byte.

---

## 16. P — `drive.salience`: the bake, at one instant, through one curve

1. Read `a_formation` = the walk's current u8 `a` at the commit instant.
   Whether that value is the raw leaky-integrator level (with minutes of
   residue) or a strict-local reading is **OD-33's option (b)**; v1 ships the
   raw integrator value and makes the residue **visible** by storing `a_at`
   beside it, so G9(e) can report bakes against their `a_at` and R-30's
   mispricing reads as a finding rather than a mystery.
2. `bake = B_max · tanh(K_TANH · a_formation)` on the Q12 grid; the `tanh` is a
   table over the 256 possible `a` levels — one row per level, so the curve's
   shape is *the swept artifact* and the function here is a lookup.
3. **No argument is consulted except that scalar.** No event kind, no signal
   type, no `source`, no referent, no valence (§20.4a: *"nothing categorizes
   fires as bad — it felt a lot when it happened"*; OD-31's strength-not-valence
   reading). A table keyed by any of those is R-21(c) and fails G9 statically.
4. `a_formation = 0 ⇒ bake = 0`, **and that is law, not tuning** (OD-56). The
   floor is the row's plain §8.4 init: a calm write is not muted, because a
   permanent quantity that could mute a write would be permanent damage (§8.4's
   constraint (a)).
5. Write the stamp: `{signal = the firing kind, magnitude = that signal's own
   value, a_at = a_formation, thought_id, signal_ref}`, `src = 0` (write) —
   assembled inside `si-affect` and handed over; `si-learn` never formats a
   stamp (§6.1, §12's one-way street).
6. `bake` is **write-once** ([01 §5.1/§5.3](01-data-schemas.md)). There is no
   decay path for it in v1 (OD-27 resolved: permanent, saturating, no half-life;
   option (b) refused), and the only way it stops steering is supersession.

**Who may bake.** Exactly a teaching event: `/correct`, `answer-to-ask`,
`/teach`, an owner statement that commits. An idle cascade forms no edge and so
buys none (I9, G10(b)); a `source: grimm` line forms no edge and buys none
(I10, G12(d)). Both cases are the same absence — no commit — reached from
different laws (mechanical vs provenance), which is why no separate test exists
for either.

---

## 17. Q — the gain ledger and the counterfactual probe

**Q.1 Exact units** (§8.2). A `/correct` triple, and identically an
`answer-to-ask` commit, has a *measured* error-before: `o_wrong` bound through
`bind_output_id`, or the named empty slot SI itself asked about through
`question_id`. So `gain_exact = +1` per successful later serve that no
subsequent `/correct` negates, and `−1` when a later `/correct` supersedes this
row's answer for the same stimulus. Signed i16 ([01 §4.1](01-data-schemas.md)).

**Q.2 Demonstrated vs asserted** (§8.2's table). At commit, `gain_kind` is set
once from `prov_kind`: `correction`/`answer-to-ask` ⇒ **demonstrated**;
`owner-teach`/`statement`/inferred/`reparse` ⇒ **asserted**, `gain_exact` starts
at **0** and `bind_output_id` stays **null** — never a fabricated zero-error.
The **only** promotion in the ledger is the first recall that serves the row and
is not corrected, which writes an event-log entry carrying the serving step id
and flips `gain_kind` to demonstrated. No gate counts asserted units toward a
calibration claim (this is the R-07 guard, and it is what stops an imported
lattice from reading as a demonstrated one).

**Q.3 The counterfactual probe** (`si-read::counterfactual`), for everything
that has no error-before:

1. context = the exact token-pulse sequence of the input (§4's lighting, replayed
   bit-identically);
2. `walk_with` = the settle-set `S` already recorded — **not recomputed**, so the
   probe cannot disagree with the thought it measures;
3. for each edge `e` in the settle trace, `walk_without` = re-run §5 with `e`
   removed from the frontier, everything else identical (same `a` trajectory, same
   caps, same seed);
4. `gain(e) = diff(S, S′)` — the diff is on quantized activations, so the probe
   emits integers, and the mind log stores **trace hashes, not float bits**
   (§16.6);
5. confidence writeback `+0.05 · min(gain, G_MAX)/G_MAX` — a normalizer, so
   `G_MAX` is swept, not asserted ([03 §6](03-constants-registry.md));
6. **probe count is bounded by construction:** only edges that appear in the
   settle trace are counterfactualized, so the cost is one extra walk per trace
   edge, not per graph edge.

**Q.4 Pre-walk fallback**, retained for statements that never produce a
walkable query: rank-of-truth over the token-transition table, `error = rank`,
ties by `last_seen`. OPEN-14's discrete metric stays open — the fallback is not
a substitute for it.

**Q.5 The law.** **A confidence change with no logged probe is invalid** (§8.1).
The build enforces it by shape: `si-learn` step 10 returns a confidence delta
only given a probe record, so an unprovened bump is a missing argument rather
than a discipline problem.

---

## 18. R — RE-PARSE: the decoder maturing retroactively

`si-learn::reparse`, on idle ticks at stage ≥ S1 (§6.6). Ordered, and the order
is the determinism claim:

1. **Eligibility** = an `s_r_pairs` row with **no derived child** (`derived_off
   = 0`) whose stimulus and response both parse under the **current** reflex set
   (R3 patterns + R14 frames, matured by learned chunk forms and taught
   template edges). "Once per row" is expressed by the absence of a child, so
   there is no cursor field to get out of sync and the pass is idempotent by
   construction (G7 asserts it).
2. **Iterate in step order** (`prov_step` ascending, ties by `row_id`): never by
   arena order, so the derivation sequence matches the teaching sequence.
3. **Derive** a `facts` row: key = the decoded entity/field from
   stimulus+response, value from the response surface, `derived_from = [sr_id]`,
   `prov_kind = reparse`, `gain_kind = asserted`, `gain_exact = 0`.
4. **Charge rule** (§5.4/G7): the derived row inherits the source stamps as
   **history only** — `src = 3`, list copied, **`bake = 0`** on the derived
   edge. The boost equals baseline while the inherited stamp list is non-empty,
   which is exactly the assertion G7 makes: a re-parse may not manufacture a
   fresh head start. (And §9 excludes `src = 3` stamps from the median, so the
   inheritance cannot launder a *reading* either — the same rule seen from both
   sides of the pipeline.)
5. **The original S-R row is never touched** — it stays ground truth forever;
   the fact row is a typed view.
6. Route the derived row through the **ordinary** write path (§12 from step 4),
   so it can conflict and arbitrate like any candidate; a later correction
   supersedes both views coherently because each keeps its own `superseded_by`
   chain (G7's last leg).
7. `si-learn` is reached here from the **idle** tick through a call owned by
   `si-session`, not by `si-walk` — the walk's read-only law (§2.5) survives
   RE-PARSE only if the scheduler, not the walker, makes the call.

---

## 19. S — the idle tick: precedence, one sampler, and a read-only cascade

`si-walk::idle_tick` + `ignite` ([02 §4](02-module-contracts.md)), §19's
behavior.

**S.1 Gate.** `IDLE ⇔ senses quiet AND agenda empty`. Senses quiet = no line
pending in this logical tick (I3's single channel is the only thing that can be
noisy). Agenda empty = no open item in `questions` awaiting an answer or a
revisit. Idle never preempts an owner line, and a real question always
interrupts daydreaming without a scheduler (§17.3).
**From B18 this predicate may gain a third *deny* conjunct and nothing else**
(**OD-126**, §33 AE.4): a declared retention/quiet window can say "not now", never
"now" — there is no path from a window value to `ignite`, `sleep::due`,
`clock.advance` or a frame read, so a busy server cannot buy thinking time the
quiet-and-empty-agenda law did not grant, and a window opening is never a cause.
`check_windows_suppress_only` and **G19(g)** are the leg and the static half.
Ignition's fifth term enters at S.4's weight sum (§30 AB.2), which is inside an
already-gated idle tick, so nothing in the arc changes *when* SI daydreams.

**S.2 Precedence, fixed:** queue revisit → due RE-PARSE → ignition.

1. **Revisit.** For each open question, in `asked_at` order: re-walk it against
   current knowledge (§5 with the ask's own target slot as the lighting). If a
   row now covers the slot ⇒ `status = resolved`, `resolved_by = row_id`, **one
   event log entry, no row rewrite, no new owner message** (G8(e)). The walk
   runs; the **commit** does not — resolving a queue row is `si-store` bookkeeping
   through `si-session`, which is the only shelf write an idle tick may perform,
   and it is a `questions` row, not a belief row (I9's boundary is exact here).
2. **RE-PARSE** if due (§18 step 1's eligibility test returns non-empty).
3. **Ignition** — only now, and only if the rate bound allows (§S.6).

**S.3 The sampler.** Build the candidate set = points reachable in the live web
(not the whole lattice — the candidate set is what the residency already holds).
For each candidate point, weight =

```
IGN_WEIGHT_RECENCY   · Clock::recency(now − last_lit(point))
+ IGN_WEIGHT_BAKE    · max(edge.bake over edges touching point)   / B_max
+ IGN_WEIGHT_DECAY_DEFICIT · deficit(point's youngest weak edges)
+ IGN_WEIGHT_CONTRADICTION · warmth(point ∈ an unsettled contradiction region)
+ mul_q12(IGN_WEIGHT_SILT, silt_level(point's conversation))   [B15, §30 AB.2]
```

all summed on the Q12 grid, then drawn from by the seeded
`Rng::{next_u32, next_fraction}` against a cumulative-weight scan in
**candidate order ascending by point handle** (the scan order is what makes the
draw reproducible, not the seed alone). **OPEN-32 is closed: the salience weight
reads the `bake`** — neither the raw stamp list nor the median — because the
bake is the only durable affective quantity on an edge, ignition is an
edge-selection process, and letting a read-side statistic steer which edge
lights would make the intrusive loop indistinguishable from retrieval (§20.4a).
Because `bake ≤ B_max`, the normalized term is bounded by construction and a
saturated edge cannot monopolize the sampler (G10(e)).

**The two readings this line carries, recorded because the build had to pick one.**
(i) The four `IGN_WEIGHT_*` are **TEMPERAMENT draws**, not swept constants: the
*drive* — the mind drifts toward what it is about to forget — is species and has no
row of its own, while *where* the idle mind wanders (puzzles vs people vs places) is
the personality layer, drawn once at birth from `birth_seed`. No sweep may ever touch
one of these rows (**OD-87**; `check_sweep_never_touches_temperament` is the leg that
holds it, and instance identity is therefore **seed identity** — where the design says
"same constant set" it means **same birth seed**). The **fifth** term is *not* one of
the four: `IGN_WEIGHT_SILT` is SPECIES and ships unarmed at 0 (§30 AB.2, **OD-108**),
because a 19th draw would lengthen `DERIV_ORDER`, bump `SEED_FORMAT` and re-hash every
run recorded since B0 — and at 0 it adds exactly 0, so every pre-arc pick here is
unchanged. (ii) `deficit(point's youngest weak
edges)` is computed as the **max over the point's live incident arcs** — the
*furthest*-drifted arc, which is what "about to forget" names — while the rehearsal
input is per-arc: §S.7's write lowers the deficit of **the arc the cascade traversed**,
and only that arc's movement is what `G10.e.2` measures. A literal *youngest* reading
(the least-drifted arc) is the opposite aggregate, and the difference is exactly what
**OD-89(b)** measured: under the max, re-heating one arc cannot lower the pull another
arc is holding up, so the same point can win the sampler's race on consecutive quiet
ticks. That choice belongs to the owner and is open at **OD-90**; nothing here adds a
constant for it.

**S.4 The cascade.** The sampled hyperpoint is the round-0 lighting; §5 runs
**unchanged** — same doors, same caps, same τ, same trace, no tokens arriving
from outside, and integration here is per round and per signal row only (§20.1).
A cascade that crosses its own live contradiction fires kind 5 **inside** the
cascade, and the widened spread lands on the **next** round (§19.5's restated
timing): the loop that makes "emotion comes from what it thinks about"
mechanical, and the reason G10's legs must see a trajectory rather than a
scalar. `drive.update`/`conduction` call counts inside an idle walk match a
sensed one; only the lighting source differs — there is no second thinking
engine, which is the whole point of S.2.

**S.5 The monologue.** Render the cascade's trace through the **same** reverse
face as any readout (§19.3): the lit points, in order, top-*k* at depth
`MONOLOGUE_RENDER_DEPTH`. It is a truthful rendering of what lit, not generated
prose; it asserts nothing, so it carries **no** FACT/INFERENCE/SPECULATION
claim (I6 satisfied by claiming nothing); it goes to the **separate** lane
(`monologue.jsonl`), and `MONOLOGUE_OWNER_FACING` is off in v1 (OPEN-23's lean).
Nothing may compose, expand, or narrate it, and no write path is reachable
from it — the missing crate edge, again.

**S.6 Rate bound.** ≤ `IDLE_PER_TICK = 1` (LAW) idle thought per quiet tick, and
≤ `IDLE_PER_HOUR` counted in **logical ticks**, never wall-clock (I5). A long
quiet stretch must show *sparse* monologue, never a spin; the counter lives in
session state so it replays.

**S.7 What an idle tick may change, and nothing else.**

| may | may not |
|-----|---------|
| reset an edge's rehearsal/decay curve (`{edge_ref, last_touched_at}`, 8 B) | create, modify or supersede a belief row (I4's only writer is §12) |
| push a **candidate** question onto the agenda when a cascade crosses two taught regions (`TWO_REGIONS_TEST`, OPEN-28 — the exact adjacency rule is undecided; lean: the cascade's settle-set contains endpoints of two edges whose kinds are both `link-*`/`owned-*` and whose `pattern_id` sets are disjoint) | emit it (the drive's budget decides) or answer it |
| re-stamp an edge it re-lights, if **OPEN-24** lands yes (`src = 2`, bounded by the same cap) — which feeds the median | change any edge's **`bake`** or confidence: an idle walk forms no edge, so it buys no head start (G10(b)); a re-light that re-prices a birth property fails G9's static leg even though it reads `a` legally (OD-33 licenses one consultation, at one instant, ever) |
| fire kind 5 and move `a` | promote or demote a stage (§11.1), or feed a label into any of the above (§18.7) |
| consume the budget | run unseeded (§19.4: unseeded ignition is a design violation, not a bug) |

**S.7 is the write ceiling for every walk that is not answering a sense line, and
§28 Z inherits part of it**: sleep's P1 and P4 may touch the rehearsal pair and, if
**OPEN-24** lands yes, one re-stamp — and they may **not** take the agenda
candidate, because pushing one creates a `questions` row and sleep's promise is
zero rows ([10 §8](10-temperament-sleep.md) law 2, **G13(i)**). A ceiling is not an
entitlement: the candidate belongs to a walk that is *wondering*, and a sleep is
tidying. P3 adds one more column *with a different permission*, the stored FLUID
`weight`, which no idle walk may touch. That asymmetry is the whole batch in one
line: **a daydream may refresh a memory's recency and may ask; only a sleep may dim
its reach, and it may ask nothing at all**
([10 §6](10-temperament-sleep.md) D.1, law 25).

---

## 20. T — the history-anchored walk (archaeology, both halves)

One mechanism, two readouts; G3's already-shipped machinery is reused, not
duplicated (§20.5).

1. The time frame must be **taught**. "back then" decodes through the same §3
   shape test; with no edge linking that frame to an interval, the query decodes
   to an unlinked frame node ⇒ SI **asks** instead of time-travelling (§7.1). An
   SI that answers archaeology questions it was never framed for is showing a
   preloaded tense table (R-14/R-15).
2. Once an anchor `t'` is resolved, `si-store::anchors(shelf, t')` returns the
   live-at-`t'` view: `written_at ≤ t' < superseded_at ∨ ∞`. Aged edges are
   re-weighted at their interval exactly as G3 already does for facts, and the
   NOW filter becomes an **anchor** filter — the walk itself is unchanged.
3. The median (§9) is evaluated with **its** recency weights measured from
   `t'`, so *"what did that word feel like back then"* is the same statistic at a
   different position in time. Storage never moves; only the anchor does.
4. Citations are mandatory and are printed from the stamps that produced the
   reading, with their `thought_step` and `a_at`. Zero deleted bytes, zero
   edited stamps (G3), and a reading that exists only at NOW is a fail
   (affect not 4D, R-28).
5. Bias matching at NOW keeps its own rule: a yesterday stamp beats a last-year
   stamp even if the older one was hotter when written — the older stamp is not
   weaker in storage, it is older in the statistic (§20.5).

---

## 21. U — LABEL and RENDER: demote-only, and copy-not-create

**U.1 Render (`si-front::reverse_decode`).** Content tokens of the output must
be a byte-subset of the cited row's `resp_surface` / `value.surface`, plus
learned chunks for composed sentences (§7.5). v1 is a template copier. The v2 LM
slot lives behind the same interface and may reorder/inflect, but any content
token absent from cited rows **auto-demotes the label** — labels are computed
from constraint satisfaction, never from LM confidence.
**`NAMELESS` (`pattern_id == 0xFFFF`) is not a surface** (B14/B17, §29 AA.4,
§32 AD.2): a row carrying it is citable, enumerable by its member handles and
reason-over-able, and the renderer **refuses to print any token derived from it**.
That is the existing containment law doing G15(f)/G18's no-word-leak leg — an
un-named mint is unspeakable by the law that already holds, not by a new branch,
and the harness's grep for a `cluster:`/`skill:` string over RENDER output is the
leg that proves no placeholder name ever leaked to the surface.

**U.2 Label (`si-front::label`).** A pure function of the activation trace and
the cited rows, evaluated on the settled thought, **first match wins**:

| order | label | predicate |
|-------|-------|-----------|
| 1 | **FACT** | settled via ≥ 1 OWNED edge **and** every content token ∈ one cited OWNED row **and** the row's time-status matches the question's tense (live at the anchor, or interval-stated for history) |
| 2 | **INFERENCE** | (a) settled via OWNED edges but composed across ≥ 2 rows/bridges, all cited, no novel tokens, **and the route's owned-hop count ≤ `INFERENCE_MAX_HOPS_OWNED`** — over the cap this predicate does **not** fire and the first-match rule falls through to the uncertainty branch (row 3's `MaxK` family), so the output is UNCERTAINTY-flavored with the full route cited, never a confident truncation (§5.4); or (b) settled via genome (BASE) paths only — **capped label**, ENGINE config only |
| 3 | **SPECULATION** | fallback: walk died in an **unnamed** dark spot and the input is question-shaped and requires an answer slot ⇒ the nonce guess; **or** `MaxK` without settle ⇒ uncertainty-flavored SPECULATION; **or** the thought settled but its route exceeds `INFERENCE_MAX_HOPS_OWNED` ⇒ the same uncertainty surface with a distinct reason string (`depth`), because the ladder must be total (I6) and an over-deep composite that could not be labeled would otherwise render as nothing |
| 4 | **I-DON'T-KNOW** | walk died below threshold and no answer row fired; **named dark spot ⇒ the render is the ask-back**, and the literal "I don't know yet" appears only for a gap with no addressable slot |

Source tag is orthogonal and always present: `OWNED` (walked via L3), `BASE`
(genome only, ENGINE), `NONE` (guess slot or nothing lit). `6 cooccur-weak` is
never FACT-eligible ([01 §5.2](01-data-schemas.md)).

**U.3 Three laws about this step.**
* **Demote-only, and affect has no door here at all.** Round 1 justified
  demote-only by CAUTION's effector; that reading was cancelled (OD-24). Now:
  **no affect quantity of any kind reaches the labeler** — `arousal` influenced
  the output only by having changed which edges were cheap, which is visible in
  the trace and therefore already audited. Demote-only stays as the structural
  backstop because the promote direction has no door.
* **`§7.4`'s two birth branches both write zero rows** (G1). The nonce
  (`si-read::guess_seed`) draws letter-class n-gram shapes of tokens seen
  **this** session from `RngState` seeded by step, so it is deterministic on
  replay and it appears in no table shipped with the build ("blork" is not in
  any file — that is the anti-cheat proof, and G5c's string audit checks it).
* **The line's attribution is assembled from two producers.** The loop's output
  frame carries `{label, source_tag, citation, trace_ref, a_trajectory_ref}`;
  the audit label stream joins **in the exporter/viewer over the written log**,
  never in `si-session` — because `si-session ↛ si-audit` is what makes G9(c)'s
  deletion test pass. Which side composes the printed line is not a design
  question; the design's constraint (no causal path from label to control) is
  satisfied by the missing edge, and this file records the join point so nobody
  "optimizes" it into the tick.

**U.4 The proposed composite — specified now, shipped OFF (OD-58).**

Composed answers are never written, so SI re-derives them forever (§5.4). That is
honest, and its cost is that composed understanding never accumulates. The lawful
fix exists already in the design — [docs/07 §7.3](../docs/07-read-path.md)
mechanism 4: weak adjacency feeds **bridge candidates**, and "a candidate becomes
a taught link upon owner confirmation". So the shape is specified here and the
promotion ships disabled (`INFERENCE_PROMOTION = off`,
[03 §5.1](03-constants-registry.md)); **inference proposes, owner disposes**, and
this file does not assume the ruling:

| attribute | value if the owner rules it in | why this and not something else |
|-----------|--------------------------------|---------------------------------|
| shelf | `links` (x=6) | a bridge is a link row; no new shelf, no new row type ([01 §2](01-data-schemas.md)) |
| edge kind | `6 cooccur-weak` | the only kind that is **already** "candidate only, never FACT-eligible" ([01 §5.2](01-data-schemas.md)) — the candidate law is a schema fact, not a policy |
| `prov_kind` | `2 reparse` is **wrong**; a candidate is not a derivation | the enum has no "inferred" value, which is the gap below |
| `prov_speaker` | `2 system-inferred` (proposed enum value — **OD-59**) | a candidate is nobody's testimony; recording it as `owner` would be the laundering docs/21 §5.4 exists to prevent |
| `confidence` | `0` | representable on the hundredths grid and *below* every tier; a 0-confidence row cannot win an arbitration tier (§6.4) and cannot be a FACT's citation, so "never answerable" is arithmetic, not a filter |
| answerable? | **no** — excluded from the labeler's cited-row set and from `emit`'s targets | "surfaced as a proposal" is an output act; being an answer is a belief act, and only the second is dangerous |
| provenance | the confirming `s_r_pairs`/`facts` row carries `derived_off →` the candidate row | so the confirmed inference reads as *inference proposed, owner disposed* on replay, and the chain is walked, not printed (G7's `a.2` machinery reuses this) |
| surfacing | one owner-facing line per candidate, **once**, at the moment it crosses `INFERENCE_STABLE_N` | the alternative (repeating) is nagging, and R-20's failure is the nagging machine; `DEDUPE_WINDOW` is the existing dedupe mechanism |
| threshold | `INFERENCE_STABLE_N` = the number of **separate thoughts** in which the same composite settled stably; default `—`, swept into `gates/G11/composition_sweep.json` | "stable" is the existing settle-set comparison (§5.2's hashed order), so the threshold needs no new identity test and no similarity measure (R-14) |

**Stability is defined over the settle-set, not over `a`.** Two thoughts agree iff
their `{(point, quantized_activation)}` sets contain the same composite
`(entity, slot, value)` point; arousal, labels and stamps play no part, which is
what keeps this path out of R-22's runaway and out of the emotion-word grep (§3 of
[00](00-stack-workspace.md)). If the owner parks promotion to v2, the row shape
above stays in this file as the interface, `INFERENCE_PROMOTION` stays `off`, and
nothing else in the build changes — which is the reason it is written out now
rather than after the ruling.

**Where the tally lives, because sleep needs it and may not import the read
path** (§28 Z.2 P2, [02 §13](02-module-contracts.md)): the composite ledger is a
**derived tally over logged settle-sets** — `si-core::eventlog` already carries
per-thought settle entries, and the count of separate thoughts containing a point
is accumulated as each entry is logged, in `(t, point)` order, and **rebuilt from
the eventlog by one scan when a session is opened**. It is RAM, not a shelf: not
persisted, not hashed, discardable at any checkpoint and reconstructible —
exactly §5's live out-degree counter's status ([06 §4](06-resource-budget.md)'s
first cost trap), and for the same reason: recomputing the whole tally at every
sleep would make P2 O(session) per sleep and O(session²) per session, which is the
one idle-path cost law [06 §9](06-resource-budget.md) refuses to break. It is
**not** a `si-read` return value, so `si-sleep` can rank candidates with the three
crates it is allowed to import. A ledger that were *serialized* would be a second
source of truth about composition — and would be `INFERENCE_STABLE_N`'s count
creeping into state that items 1–3 of the hash do not cover.

**The gap this exposed, stated because the build hits it immediately:**
[docs/07 §7.3](../docs/07-read-path.md) mechanism 2 already creates bridges that
are `speaker=system-inferred` (unique-slot resolution), but
[01 §4.1](01-data-schemas.md)'s `prov_speaker` encoded only `{owner, grimm}` — the
shipped schema could not represent a row the design already writes, and G12(e)'s
assert as literally worded ("zero belief rows with `speaker ≠ owner`") would fail
on every legal birth of a weak bridge. Both files now read the same way: **belief
rows** are `s_r_pairs`, `facts` and `questions` and their speaker is `owner` or
the run is a G12(e) fail; **bridge/candidate rows** on `links` may carry
`2 system-inferred` and are never FACT-eligible and never answerable. This is
OD-59 — the enum widening is mechanical, the *scope sentence* is the ruling that
matters, and an implementer who unifies the two scopes either breaks clean-room
provenance or breaks the bridge mechanism.

---

## 22. V — the audit classifier: six predicates, output-only

`si-audit::classify(signals, a_trajectory, behavior) → LabelStream`, run over the
**persisted event log**, never live state, imported by nothing ([02 §9]).

Each of the six labels is a rule over already-logged rows, with
`AUDIT_THRESHOLDS` as the predicate parameters (OPEN-22 owns the rule-set; the
six names are the closed installed set and a seventh word needs a ruling, not a
pull request):

| label | predicate over logged artifacts |
|-------|--------------------------------|
| curiosity | `a` above its floor **∧** a kind-0 event in this thought **∧** an ask emitted |
| surprise | a sharp kind-0 spike at a write, decaying within a tick or two |
| satisfaction | a kind-1 realized-gain row **∧** a pending question closed |
| caution | kind-2 count rising **∧** a recent ASK-branch arbitration |
| boredom | sustained low kind-0 **∧** low spikes **∧** low open-question pressure |
| frustration | high `a` **∧** repeated `TauMiss`/`MaxK` or repeated contradiction **on one slot** |

**Reproducibility is the acceptance condition:** every emitted label must be
recomputable byte-for-byte from the signal rows and the `a` values already in
the log. An **uncitable label** fails G9(b) the way an uncited FACT fails I6.
The classifier reads the median's *printed* value where a label needs it (a
readout), and never writes one; `coupling_observed.json` is written from the
co-occurrence counts and is **output-only** — an observed adjacency that becomes
a wiring rule fails the gate that produced it (§18.3, R-24). If
human-recognizable emotions do not appear, this file reports a finding; there is
no gain matrix to reach for and the build must not invent one.

---

## 23. W — the speak gate: two filters, in order, upstream of sense

`si-supporter::speakgate` ([02 §8](02-module-contracts.md)), §21.3.

1. **Layer 1 — token filter.** Every token of the emission must be in the
   **taught** lexicon (the term records made taught by `source: owner` teaching
   events — [01 §7](01-data-schemas.md)'s distinction between "interned" and
   "taught" is what makes this layer meaningful: every arriving token is
   interned, only owner-taught ones count here). At `t=0` the taught set is empty
   ⇒ **structural muteness**: he cannot bootstrap SI, cannot seed a vocabulary,
   cannot be the first OWNED edge.
2. **Layer 2 — claim filter.** The normalized claim must be a **strict
   restatement of one existing row**, referenced by row id (OD-40(i), Mike's
   call). "Not words that exist but claims that exist." This is the layer that
   rejects *apples are good because vitamins x and y* when every token passed
   layer 1 — the canonical failure, shipped as a test rather than as advice.
   v1 accepts the published recombination limit (§21.8(1)) *because* the strict
   single-row rule is in force; the day it is relaxed for convenience, that
   limit goes live.
3. **Admit or reject, before anything else happens.** Rejected ⇒ one gate-log
   line with the reason counts (`n_untaught`, `novel_claims`), and **no sense
   event at all**: no signal, no `a` movement, no stamp, no row. Admitted ⇒
   `sense_input` appends one `context` row with `source = grimm` and hands the
   line to the ordinary tick path. The order is the whole design: a filter that
   ran *after* admission would already have changed SI.
4. **His loop is the rejection.** The gate returns *why*; he rephrases within
   the rules or stays quiet. Rejection feedback replaces the next fine-tune,
   which is why L4 puts discipline in code between him and SI rather than in his
   prompt.
5. **What admitted lines may still not do** is enforced downstream, not here:
   §12's step 0/1/2 and 5 (`pending_ask_match` owner gate, teach short-circuit
   owner gate, `Confirm` owner gate). The filter narrows content; the `source`
   tests on pre-existing branches narrow *privilege*. Neither is redundant and
   neither may be merged — the asymmetry law (L5) forbids filtering the
   **owner's** unknown tokens, which is the only way new vocabulary ever
   arrives.
6. **From B18 a granted `Trusted` line passes this same gate, unchanged**
   (§33 AE.5): the same taught-lexicon membership test, the same strict
   single-row restatement, the same reject-before-admit order, and the same
   `Refused{SpeakerNotRepresentable}` downstream for any belief row. There is no
   second filter for the new class — one function, one filter, one class — which
   is what makes **G19(h)** a reuse of G12(b)(c)'s fixture rather than a new
   mechanism, and why delegated authority adds no door (ORGAN/DOOR/SIGNAL/LABEL
   counts stay six, six, six, six).

---

## 24. X — hash composition and replay

**X.1 Compose** (`si-core::StateHash::compose`, [01 §10](01-data-schemas.md)):
md5 over the concatenation of the **nine** item classes in the recorded order —
live envelopes+bodies sorted by `(x, y)`; edge records (with `bake`, `weight`,
`kind`) sorted; stamps (with `a_at`, `magnitude`, `signal`); interner + lexicon
sorted by id; last settle-set as `{(point, quantized)}` sorted by **point id**;
the whole `a` trajectory event-ordered; `RngState{state, inc, spare}` plus its
advance history; the version tags (modulation curve **and its selected variant
index**, lag constant, genome `t=0`, the inference depth cap); **and the session
core block** — `birth_seed`, `seed_format`, `|DERIV_ORDER|`, the derived-vector
digest, and `{sleeps_completed, sleeps_interrupted, last_slept_at_t}`
([01 §13](01-data-schemas.md)). The API takes these nine, in that order, and a
caller **cannot pass a median** because no such field exists
([01 §11](01-data-schemas.md)) — and cannot pass a *stored* temperament vector
either, because the vector has no address to read from: item 9 carries its
**cause** (the seed), and the values are re-derived before the hash is ever
computed (§25 W.0).

**X.2 Replay.** Re-run the tick sequence over the recorded sense stream
(`transcript.txt` / the `context` rows, which already contain the admitted
`source: grimm` lines — this is why determinism survives a non-deterministic
LM next door, §21.7). Compare `state_hash.txt` at `REPLAY_TOLERANCE = 0` bytes.
**Step 12b replays for free**: sleep is scheduled by the quiet-tick counter and
the pass mask, both of which are functions of the recorded stream, so a replay
that sleeps on the same ticks it originally slept on is the only replay that
hashes the same. A harness that suppressed sleep in the re-run would diverge at
P3's first byte — which is exactly the hook **G13(a)** uses, and why the
`SLEEP`-disabled variant V9 is a *comparison* rather than a mode
([05 §2](05-gate-harness.md)).
Every excluded item is an assertion, not an oversight: no wall-clock, no
`--config` flag (config is not state), no viewer state, no Grimm process state,
no raw floats, no audit labels, no median, **no calm view** (§28.4's terminal
law), no stored temperament vector.

**X.3 The two claims this file does not make.** The GPU leg is v2 and its
parity contract is "same seed + same fused plan ⇒ same settle-set hash **within
the grid**", never bit-exact floats (§16.6); and per-event `drive.update` fused
on-device is an open engineering leg with three candidate shapes and **no**
measured one ([docs/16 §16.5](../docs/16-think-walk-hardware.md)). Both stay
UNMEASURED in [06](06-resource-budget.md).

**X.4 One seed, two streams, and a third that does not exist.** The birth seed
expands into a **derivation stream `T`** — read to exhaustion by
`temperament::derive` at wake, then *discarded* — and the **runtime stream `R`**
(`RngState`, item 7), which ignition, RE-PARSE and every seeded draw use. `R` is
seeded from the seed *after* `T` is drained, in the fixed order `DERIV_ORDER`
specifies ([10 §4](10-temperament-sleep.md) P.1–P.2), so the 18 drawn values
cannot shift `R`'s trajectory between a run that reads the registry table and one
that does not. **Sleep draws from neither stream**: it has no RNG handle, and
P4's seeded pick is replaced by a full deterministic sweep in point-handle order
(§28.4). A third stream for sleep was refused ([10 §12](10-temperament-sleep.md))
because two streams are already the whole reason a non-deterministic neighbor
cannot break a replay.

**X.5 The arc's five tables are hash-exempt, and that is a stronger claim than
"not hashed"** (B14–B18, [12 §2.6](12-v2-arc.md)). The crystallize and skills
candidate tables, the silt table, the frame scratch (co-light log, ghosts,
role-graphs) and the derived event views are **recomputed** from the recorded
sense stream on every replay — each is a pure fold over `(frame_log, store, t)`
or over rows already inside items 1–3, so a replay restores nothing and diverges
nowhere. **Step 5a/5b/7a/7b replay for free**, exactly as §12b does: silt's levels
are a function of the touch ticks, the recurrence bars a function of the frame
sequence, the tug index a function of the committed rows. What a replay therefore
reproduces byte-exactly is the **hash** (nine items, order unchanged) and what it
reproduces value-exactly is every arc table — the two `.jsonl`/`mindlog` blocks
are the same computation printed twice. `transcript.txt` carries the
`speaker_identity` per line, so **G19(e)**'s identity-tagged replay needs no new
state item; the identity lives in the sense frame body, which is already hashed
transitively through the `context` row it mints. And the one hash byte that does
move is item 8's row count — 22 new registry names, `CONSTS_VERSION` 0x0002, at
B14 once ([01 §10](01-data-schemas.md), [12 §3.3](12-v2-arc.md)).

---

## 25. Y — freeze/wake and the config guard, as procedures

1. **Freeze** = the genome version tag (content hash of the sorted row file).
   Nothing else changes at freeze; L1/L2 bodies are immutable from then on
   (G6's checksums are the check). **A freeze first sleeps** (sleep trigger 2,
   [10 §7](10-temperament-sleep.md)): the pass sequence runs *before* the
   checkpoint is written, so the consolidation is inside the durable state and a
   resume never repeats it. The order is not cosmetic — a crash mid-sleep leaves
   the pre-sleep snapshot, so a half-applied rescale is unreadable, and
   `last_slept_at_t` makes the re-run idempotent (**OD-81**).
2. **Wake** = **W.0** re-derive the temperament, then upload genome + replay the
   overlay. W.0 is `temperament::derive(birth_seed, seed_format)` over the
   container's core block ([01 §13](01-data-schemas.md)), before the first round
   of the first thought, and it is *not* a load: the vector is never serialized,
   so a container whose digest does not match its own seed is refused at open
   (**G14(c)** byte-compares the re-derived values against item 9's digest
   rather than trusting a stored copy). **The overlay replay is the CONFIG SI
   walker run in full** — it is deterministic because §5 is integer arithmetic on
   the grid, which is the one place the quantization law pays for itself.
3. **Config guard** ([00 §6](00-stack-workspace.md)): in CONFIG SI,
   `Frozen::load` refuses **before** forming a path string, `engine`-feature
   bodies are compiled out of the shipped binary by default, and `SIgen` is
   physically absent from a SI release artifact. G5(c)'s handle audit asserts
   that no CONFIG SI code path can open a frozen-layer file. CONFIG SI walks
   L3 only, so the birth connectome really is nearly empty (§16.2) — and a
   transcript that looks impressive after three owner turns is a preloaded
   corpus, not a mind (R-25).
4. **Stage is derived, not stored.** Each tick recomputes the stage from the
   row counts (`STAGE_THRESHOLDS`, [03 §6](03-constants-registry.md)); the
   envelope's `stage` byte is write-time provenance only. No stage transition
   may be defined on Grimm's readiness (§21.6) and no affect quantity enters the
   count (§11.2). Sleep cannot move a stage in either direction, and it cannot
   move it *accidentally*: P3 changes no row count
   ([10 §7](10-temperament-sleep.md)'s stage row, **G13(m)**), so a mind never
   wakes up a different organism than the one that fell asleep.
5. **Resume refuses on a seed mismatch.** The core block's `birth_seed` and
   `seed_format` ([01 §13](01-data-schemas.md)) are compared against the resume's
   `--seed` argument before a single row is read, in the same spirit as the
   geometry-mismatch refusal of [00 §5](00-stack-workspace.md): resuming an
   individual under a different seed is not continuing a mind, it is starting a
   different one and letting its rows claim the first one's history
   ([10 §3](10-temperament-sleep.md)). `--seed` is therefore mandatory on a
   resume, and the seed a run used is printed in the run header rather than left
   to be recovered from a container.

---

## 26. The ordering laws, collected

| # | order | law it protects | gate that catches a violation |
|---|-------|-----------------|-------------------------------|
| 1 | teach shape test **before** pending-ask match | §1.4 / I3 (an explicit act is not a reply) | G8(f) |
| 2 | `source` read from the wire **before** any privilege test | §21.7 / R-32 | G12(a)(e) |
| 3 | layer 1 **before** layer 2 **before** admission | §21.3 / structural muteness | G12(b)(c) |
| 4 | `boring` **before** `consistency` | §6.2 (conflict is never dropped as boring) | G1, G4 |
| 5 | `consistency` **before** `arbitrate`, `arbitrate` **before** `commit` | §6.4 (`Ask` must be able to commit nothing) | G4 |
| 6 | `drive.salience` **exactly once**, at the commit instant | OD-33 / §20.4a | **G9 static** (a re-priced bake) |
| 7 | gather → `update` → advance → `conduction` → spread → damp → **normalize** → **τ test** | §20.1's lag, §18.5's cap, determinism | G9(a) both halves, G5(a) |
| 8 | round *k* conducts with round *k−1*'s `a` | OPEN-29 / R-27 (no mush, no decorative affect) | G9(a) |
| 9 | bias clip on the **sum** of `β·σ + γ·m` | R-22 (double cap is a runaway) | G9(e) |
| 10 | quantize **before** accumulate; fixed iteration order; no atomics | §16.6 / I5 | G5(a) byte-exact replay |
| 11 | normalize (step 9) **before** frontier trim and τ | determinism + the cap's meaning | G5(a), G2 |
| 12 | settle-set hashed by **point id**, displayed by activation desc | §16.6 (rendering must not reach the hash) | G5(a) |
| 13 | revisit → RE-PARSE → ignition | §19.1 (a real question interrupts daydreaming) | G10(a)(d) |
| 14 | ignition draw over candidates sorted by point handle, seeded by state | §19.4 / I5 | G5(f), G10 |
| 15 | labeler **after** the settle, **after** render containment | §9 (trace function, demote-only) | G9(c), G12 |
| 16 | `resolve_ask` **after** commit | §17.3 (a loop closes by row existence) | G8(e)(f) |
| 17 | gain ledger **last**, and only with a logged probe | §8.1 / R-07 | G7, G11 |
| 18 | event-log flush → compose hash → advance clock | I5 (the hash is of the tick, not of the render) | G5(a) |
| 19 | the settle runs **in full**, then the labeler counts owned hops, then the claim is bounded | composition is the walk's free consequence; the cap bounds the *claim*, not the spread — truncating the walk instead would erase the deep route from the trace and make the bound unauditable (§5.4) | **G11(i)**, `check_hop_cap_is_claim_side` |
| 20 | a composite is read by the labeler and the renderer, and reaches `learn.commit` **never** — not in this statement, not in a later one | I9 / R-25: SI may not believe its own arithmetic; and the anti-cheat law: a row in L3 always traces to a conversation | **G11(h)**, `check_compose_writes_nothing`, and **law 22 below**, which is the same sentence applied to a new organ |
| 21 | sleep is entered **only** from a quiet tick, strictly **after** step 12's idle thought and strictly **before** step 13 | the hash is of the post-sleep tick, so a replay that skipped a rescale diverges — sleep is testable because it is *inside* the durable sequence, not beside it ([10 §7](10-temperament-sleep.md)) | **G13(a)**, G13(f); laws 13 and 18 unchanged |
| 22 | **P1 → P2 → P3 → P4, fixed, with an inbox check at every boundary**, and a pass's writes complete before that check | a pass is the atomic unit because its writes are one FLUID sweep; a rescale is never half-applied and never half-aborted (**OD-79**). A partial sleep is a legal, logged, replayable state | **G13(g)** (interrupt legs, one per boundary), `check_sleep_writes_nothing` |
| 23 | at freeze: **sleep first, then the checkpoint write** — never the reverse | sleep's durability unit is the checkpoint; a crash before it leaves the pre-sleep snapshot, so no half-applied rescale is ever readable, and a resume never repeats a sleep (**OD-81**) | **G13(h)** (crash mid-sleep → resume → byte-identical hash) |
| 24 | **`temperament::derive` before the first round of the first thought**, in every process, from item 9's seed only | the vector is a pure function, so it is *derived* state; storing it would let a container carry values its seed does not justify ([10 §4](10-temperament-sleep.md) P.3, §25 W.0) | **G14(b)(c)**, `check_temperament_closed` |
| 25 | `weight` is written **only** at a commit (init), a supersession (→ 0, i.e. law §6.4), a history-anchor re-weight (§7.3, read-only view), or **P3 while quiet** — and P3 is the only one of those that fires with no input at all | "dark = unroutable but still citable" is honest only because every weight change is a dated, logged event: the owner can always ask *when* a word went quiet and the answer is a `sleep.jsonl` line, never "it drifted" ([10 §6](10-temperament-sleep.md) D.2) | **G13(b)** (before/after arena diff), **G13(d)** (the `weight == 0 ⇔ superseded` biconditional holds after every sleep) |
| 26 | the calm view is produced **last** inside P4 and read by **nobody** | P4's statistic is a readout over rows already hashed as items 1–3; letting it feed a weight, a floor, `d` or `ignite` is R-28's frozen emotional address arriving through a side door — the median becoming state without being called a median ([10 §8](10-temperament-sleep.md) law 1) | **G13(k)**, `check_calm_view_is_terminal` |
| 27 | **a decay is before the walk, a deposit is after it** — tick step 5a precedes step 6, and every deposit follows step 7 | silt must never bias a selection using light from the very frame it is about to bias; the lag is OD-29's one-round shape, and it is the only reason G16(d)'s "clears backwards" is a pure function of the transcript instead of of the table's own history ([12 §4.1](12-v2-arc.md)) | **G16(a)(d)**; `check_one_clock` (widened to `si-silt`) proves the decay is the same curve, not a second one |
| 28 | **observation is after settle, before commit** — 7a and 7b sit strictly between steps 7 and 11 | a detector that ran before settle would nominate on a half-formed thought; one that ran after commit could read its own minted row as evidence, which is I9 through a microscope. This ordering is what makes G15(e)'s spam bar and G18(f)'s generality bar expressible at all — both count *frames*, and a frame only exists once the settle has fixed its lit set | **G15(c)/(e)**, **G18(c)/(f)**, `check_crystallize_no_commit`, `check_skills_no_commit` |
| 29 | **a nomination enters the queue as a `drive::Gap`, never as an emission** — 7a/7b feed step 8's existing queue and nothing else | neither crate can reach `emit` (`↛ si-affect`), so `drive` decides at the same `ASK_CAP_PER_INPUT` and the same `DEDUPE_WINDOW` as every other gap. "A busier world may not buy more asks" (**OD-128**) is a consequence of this line, not a rule added on top of it | **G15(e)**, **G18(e)**, **G19(f)**; ordering law 1 is untouched — an explicit act is still not a reply |
| 30 | **the voice is after the label and never before it** — 9a runs after step 9's `front.label` + `calibrate`, and no realized string is an input to either | a trace function may not be re-run on a surface the loop did not choose; if the voice could reach the label, "how sure was I when I said it?" would become "how sure was my *phrasing*?", which is R-24's frozen address again. Made structural twice over: `si_front::label` reads `Rendered.content`/`containment` and never `surface`, and `si-tongue ↛ si-front` ([02 §11](02-module-contracts.md), [13 §4.1](13-tongue.md)) | **G20(b)**, `check_tongue_is_leaf` |

---

## 27. What this file decided, and what it left open

**Decided here, inside the law (the design explicitly delegated these):**

* `diff` for the settle test = L1 over the quantized activations of the union of
  lit sets — §16.1 names this as "exact operator fixed at build".
* `reach(a)` = a **per-source** outgoing-edge cap, not a per-round global one
  (a global one would make `FRONTIER_CAP` redundant and would couple
  modulation to graph size); frontier trim stays a separate step.
* Tie-break order for top-k and frontier: (activation desc, first-lit round
  asc, point id asc) — one convention, used in both places.
* Where the audit label stream joins the owner-facing line (§21.U.3).
* RE-PARSE eligibility expressed as "no derived child" rather than a cursor
  field, which is what makes G7's idempotence assertion structural.
* An idle tick's queue resolution is `questions`-shelf bookkeeping performed
  through `si-session`, not a belief-row write — the boundary I9 needs, stated
  as a call site.
* **What a "hop" is** (§5.4): one traversed OWNED edge in the settle trace, in
  contribution order — not a round, not a lighting, not a re-accumulation. The
  design's `INFERENCE_MAX_HOPS_OWNED` range ("1 … K") is in rounds on the walk's
  side and in edges on the labeler's side, and without this definition the cap
  would not be checkable.
* **Where the depth cap is enforced** (§5.4, ordering law 19): the labeler's
  citation check, after a full settle — never a walk-side truncation, which
  would remove the evidence the leg needs.
* **"Stable" for a re-derived composite** (§21 U.4): the same composite
  `(entity, slot, value)` point present in two settle-sets compared on the
  quantized grid — the existing §5.2 machinery, no similarity measure, no
  affective quantity, nothing new to store.
* **`prov_speaker`'s scope** (§21 U.4, OD-59): belief rows are owner-only;
  `links` bridges and candidates may be `system-inferred`, and the two sets are
  disjoint by shelf, which is what keeps G12(e) both true and satisfiable.
* **A "cold frame" is defined once, for both detectors** (§29 AA.3, §32 AD.3): a
  frame with any `primed` light contributes no *recurrence* but still contributes
  *members*. One definition, cited twice — two definitions of "counts as evidence"
  would make G15(e)/G18(e)'s spam bars incomparable.
* **Skills' generality bar runs before its recurrence bar, and forwarding is
  evaluated there** (§32 AD.3/AD.4): so one frame is counted by exactly one
  detector, and the exclusion is one test rather than two claims.
* **`Role` is read off the frame log, not off a second trace** (§32 AD.1): first-lit
  round plus edge direction, both already in `CoLight`. A new trace type would be a
  new state the hash does not carry.
* **A re-light's degradation is by omission** (§31 AC.2, §29 AA.5): over-bound
  `RELIGHT_MAX`/`GHOST_MAX`/`SHAPE_MAX` means less scratch or fewer re-lights,
  **never** a truncated walk — law 19's principle applied to the arc's caps.
* **The lane is a byte, not a column** (§33 AE.4, OD-128): `origin = 2` *is* the
  owner-lane tag, which is what lets [docs/26 §26.13](../docs/26-world.md)'s
  "80-byte envelope unchanged" stay a fact rather than a hope.

**Left open, unchanged and not tuned away** (all already in
[03](03-constants-registry.md) /
[docs/decisions/open-decisions.md](../docs/decisions/open-decisions.md)):
`LAG_UNIT` (**OD-29**), the median's point-term shape (**OPEN-30**), what counts
in a stamp history and `MEDIAN_MIN_N` (**OPEN-31**), `B_max`/`K_TANH` with a
non-degeneracy objective (**OPEN-33**), idle re-stamping (**OPEN-24**),
`TWO_REGIONS_TEST` (**OPEN-28**), the seventh audit word (**OPEN-22**), valence
(**OD-31**), the tier/rank split for `/teach` (**OD-41**), the whole v2 GPU leg,
and every magnitude that still reads `—`.

**Ledger items raised by this file.**

* **OD-57** — the weighted-median rank convention (lower vs upper element at the
  half-weight crossing). The design calls the median an order statistic (§20.6),
  which rules out interpolation and rules out a mean; it does not say which side
  of the crossing the statistic takes. Lean: **lower**, as the conservative
  reading of "no word flips on one comment". It is a build blocker only in the
  sense that a build must pick one and record it; a pick that changes
  mid-session changes replay, so it rides the version-tag mechanism
  ([03 §1](03-constants-registry.md)) rather than being a literal.
* **OD-58** — **does a stably re-derived inference become a bridge candidate row
  in v1, or is promotion parked to v2?** This is the owner's ruling, asked and
  not assumed (§21 U.4). The row shape, the `INFERENCE_STABLE_N` constant and the
  gate leg are all specified here so that either answer costs one constant; the
  build ships `INFERENCE_PROMOTION = off`, so the default answer is "parked"
  without anyone having decided it in code. Its sub-question, answered the same
  time: **does a genome edge consume owned-hop budget on a mixed route** (§5.4) —
  lean, the tighter of the two caps applies and both counts are printed.
* **OD-59** — `prov_speaker` gains the value `2 system-inferred`, because
  [docs/07 §7.3](../docs/07-read-path.md) mechanism 2 already writes such bridges
  and the schema as encoded could not represent them. The mechanical part is not
  in question; the part that needs a word from the owner is the **scope sentence**
  (belief rows vs `links` rows), because an implementer who unifies them either
  breaks clean-room provenance or breaks the bridge mechanism.

---

## 28. Z — SLEEP: the four passes as a procedure

The *what and why* is [10 §6–§9](10-temperament-sleep.md); this section is the
ordered procedure a builder transcribes, in the same shape as §19 S is for idle.
Numbered last, like `02 §13`, because §24–§27 are cited by number from four other
files and a section's position here carries no meaning.

**Z.1 Entry.** At step 12b of a quiet tick, `sleep::due(quiet_run, freeze)`
returns `Yes` when either trigger fires: the quiet-tick counter reaches
`SLEEP_EVERY_QUIET_TICKS` (the same quiet predicate as §19 S.1 — one definition,
not two), or the freeze path (§25 Y.1) is about to write a checkpoint. Nothing
else in the build can start a sleep: no wall clock, no panel widget, no CLI verb,
no MCP tool ([08 §9](08-gui-observability.md)'s refused widgets).

**Z.2 The sequence.** `sleep` then runs **one quiet tick per walk**, each closing
with §2 note 6's flush→compose→advance, and each opening with the inbox read
described in note 9. Per pass, in fixed order:

| pass | procedure, in order | writes | aborts to |
|------|---------------------|--------|-----------|
| **P1 REPLAY** | for each `context` row **new since the last sleep** (`t > last_slept_at_t`), in row order, up to the sleep's remaining walk budget: light it as a pattern, run §5's eleven steps to its own settle or stop, record the touched edges | `last_touched_at` (the 8-byte rehearsal pair) | inbox check before the pass |
| **P2 INTEGRATION** | read the derived composite tally (§21 U.4): for each composite `(entity, slot, value)` point, the count of **separate thoughts** whose settle-sets contained it (§5.2's comparison, quantized grid, no similarity measure); rank those with count ≥ `INFERENCE_STABLE_N` | **nothing in v1** (`INFERENCE_PROMOTION` off, OD-58) | inbox check before the pass |
| **P3 GLOBAL RESCALE** | per live belief shelf, in `(x, y)` order, for every OWNED edge: `d = DIM_BASE + (1 − DIM_BASE)·Clock::recency(now − last_touched_at)`; `w' = floor_shelf + (w − floor_shelf)·d`; both on the Q12 grid by reciprocal multiply | `weight` (u16) — and, per law 25, nothing else anywhere in the build while quiet | inbox check before the pass |
| **P4 CALM REPROCESSING** | sweep the live web in **point-handle order** — no seeded pick, no RNG handle — and walk the first `remaining` start points, each thought starting at `a = 0` (a *start*, not a pin: §7's per-event integration runs normally, so a signal that fires during a calm walk is a real event) | rehearsal pair; a re-stamp `src = 2` **iff** OPEN-24 lands yes; **no agenda candidate** (pushing one creates a `questions` row — the paragraph below Z.2's table, and **G13(i)**) | inbox check before the pass, and its report line is the sleep's last write |

**The walk budget is one number, spent in pass order (OD-85).** A sleep runs at
most `SLEEP_MAX_WALKS` walks, and because each is a quiet tick (**OD-78**) that is
also the exact ceiling on how far `t` moves. The split is not a second constant:
P1 takes `min(rows_new_since_last_sleep, SLEEP_MAX_WALKS)`, keeping the **newest**
rows when it must truncate and walking that set oldest-first so row order is
never violated; P4 takes the remainder, and a remainder of 0 leaves P4's **sweep**
intact — it still sweeps the live set in handle order and still reports its
counts, and writes nothing, because
its report is a readout (Z.4) and not a permission. Two facts make this the shape
rather than a truncation anyone should apologize for: (i) a sleep only starts on a
quiet run, so "new since the last sleep" is bounded by the input burst that
preceded the quiet, and (ii) P3's rescale and P4's selection sweep are both
O(live edges) and *not* walk-bounded, so no bounded night can leave an edge
undimmed — the budget caps the re-lights, never the sweep. An engineer who reads
"replay this session's own sense events" as "replay all of them, every time" gets
a sleep whose cost grows with the session, which is the one idle-path cost law
§19 and [06 §9](06-resource-budget.md) both refuse to break.

**Z.3 The interrupt.** `Aborted{at_pass}` is an `Ok` outcome. When the inbox read
at a boundary finds a pending frame, the remaining passes are skipped, the
partial report is written, `sleeps_interrupted` advances, and the frame runs as
the next ordinary tick — SI hears the owner *before* it finishes dreaming, and the
night it did not finish is a logged fact, not a silent truncation.

**Z.4 The terminal law.** P4 emits the **calm view**: counts of rows that read
differently at rest than they settled, edges re-lit, stamps written. It is
published to `sleep.jsonl` and to the SLEEP panel
([08 §6 O11](08-gui-observability.md)) and **consumed by nothing in the loop** —
not a weight, not a floor, not `d`, not `ignite`, not a label, not a gate
criterion. That is law 26, and it is the difference between "consolidation" and a
second, unnamed affect system.

**Z.5 What a sleep may not do, in one sentence each, because each is a door
someone will try to open later.** It may not create a row of any shelf — not a
belief row, not a `questions` candidate (Z.2/P2, Z.2/P4, law 22's missing edge).
It may not brighten a weight (P3 is monotone non-increasing by
construction — the assert is a contract check, so a brightening is a panic and not
a policy branch; §9 of [10](10-temperament-sleep.md) prices why). It may not lower
`confidence`, move a `bake`, touch a stamp's `a_at`/`magnitude`, extend an edge
list, or set `superseded_*` — all PROVENANCE or STRUCTURE
([06 §2](06-resource-budget.md)'s partition). It may not change a stage
(§25 Y.4, **G13(m)**). It may not run at all while a sense frame is pending. And
it may not be *scheduled* by anything except a quiet counter and a freeze: a
supporter message, a panel click or a wall clock that could start a sleep would
be an external setter of SI's inner state (I8), which is the one thing in this
batch that is not a lean but a law.

**Ledger items raised by this section:** **OD-85** (the replay set and the walk
budget split — Z.2's paragraph, found while [06 §5.1](06-resource-budget.md)
priced the sleep), and otherwise none: the rest are
**OD-71…OD-84** in [10 §13](10-temperament-sleep.md), and the two that a builder
must not "resolve" locally are **OD-58** (P2's write stays off until the owner
rules) and **OD-80** (no staging, no extra passes, no pass reading a previous
pass's output).

---

## 29. AA — CRYSTALLIZE: the four rules, as a fold over one frame log

The *what and why* is [docs/22 §22.3–§22.6](../docs/22-crystallize.md); this is the
procedure a builder transcribes. Numbered after §28 for the same reason `02 §13`
and §28 are: a section's position here carries no meaning, and four files cite
§24–§27 by number.

**AA.1 Entry.** Tick step **7a** — strictly after `settle_or_stop` returned its
`Thought`, strictly before step 11's commit (ordering law 28). Its whole input is
the frame's own `CoLight` slice ([12 §2.6](12-v2-arc.md)'s scratch), a **read-only**
store view and `t`. Its whole output is `{nominations, candidates, frames_counted}`.
It is a pure fold over those three arguments plus its own session table, and the
table is hash-exempt (**OD-98**) — so a replay recomputes it from the transcript
rather than restoring it, and G15(a)'s two hashes are byte-identical.

**AA.2 The four rules, in this order, because the order is the spam bound.**

| # | rule, exactly | why in this position |
|---|---------------|----------------------|
| 1 | **co-light set**: from the frame log, `M` = the distinct **L3-OWNED** point handles that lit in this frame and share a relation; `|M| ≥ 3` or the frame contributes nothing | the cheapest test runs first, and the OWNED restriction is I2: an all-BASE set is inherited structure, not a thought the mind had |
| 2 | **shared-filler test**: for every member, the set of `(relation_kind, filler_point)` pairs it carried this frame must intersect a **common relation kind** | a set that merely co-lit is a coincidence; the shared relation is what makes it one *cluster*. Frozen rows may be **fillers**, never **members** — a superseded belief may help name a shape but may not be cited as its evidence |
| 3 | **recurrence**: the same member-set hash must appear in `R_MIN_CRYSTALLIZE` **distinct cold frames** whose ticks are pairwise `≥ DELTA_MIN_CRYSTALLIZE` apart; `R_ASK_CRYSTALLIZE` is the recurrence at which the ask opens, and `R_ASK ≥ R_MIN` is the detector's own assert, never a sweep's | "one hot afternoon cannot mint a concept": `DELTA_MIN` is measured in **logical** ticks, so it is clock-independent (I5) and a single dense burst is one frame, not evidence |
| 4 | **novelty against the lexicon**: if an existing live **or frozen** row already binds exactly `hash(M)` under one label, the candidate **does not open**; instead the ordinary `confidence` leg runs on that row and reports "already held" | this is the leg that makes a detector's hit rate auditable — a re-mint would be a duplicate row, and G1's dedupe is not the only guard because a cluster has no `stim_signature` yet |

Between 3 and 4, a candidate **merges by member-set hash** (never by proximity,
never by stack): `hash(M)` ascending order is the table's iteration order, so the
fold is single-threaded, integer-only, and the same transcript yields the same
bytes. Open contradictions (§14's set) are excluded from membership — a cluster
that straddles a live contradiction is a disagreement, not a concept.

**AA.3 The cold-frame rule (OD-105).** A frame in which **any** point lit because
of a `primed` tag — silt's `primed` set, or B16's tug re-light — contributes **no**
recurrence, though it still contributes members. The rule is what makes G16(e)
(crystallize isolation) a *proof* rather than a hope: with the rule in force, the
candidate table is byte-identical with silt on and off, and a primed frame that
*did* increment a recurrence is the red. The `primed` flag ships in `CoLight`
regardless of OD-104's outcome, which is why option (b) would be a switch and not a
refactor.

**AA.4 The ask.** A nomination at `R_ASK` becomes one `drive::Gap` at step 8 with
`gap_kind = 5 cluster-nomination`, `origin = 2`, `arousal_at_ask = 255`,
`target_slot = 0` and a citation **enumeration of the member surfaces** — the
existing ask-channel law, not an English template
([docs/22 §22.4](../docs/22-crystallize.md)'s sentences are illustrations, R-27).
Zero new literals, so `check_string_constants` does not move. The answer, when the
owner gives one, is an ordinary answer-to-ask commit minting a `links` row with
`key_field`'s kind byte `8 cluster-point`; an unnamed mint carries
`pattern_id = NAMELESS` and is therefore **addressable and reason-over-able but
unspeakable** — step 10's renderer refuses any surface whose row carries it
(**G15(f)**).

**AA.5 What it never does.** Reads no `a` (the crate has no edge to `si-affect`);
writes nothing (no edge to `si-learn`); never ranks candidates by anything except
set hash for iteration and recurrence for the bar; never merges two tables' rows.
`GHOST_MAX`/`SHAPE_MAX` over-bound means the frame **builds less scratch**, so the
detector sees fewer members — the guard degrades by omission, never by truncation
of the walk (law 19's shape, unchanged).

**Ledger items raised by this section:** OD-97…OD-101 (the family's own range) plus
**OD-133** (stage criterion is kind-blind) and **OD-134** (reading (i): G15(c) is
frame-trace-fixed), and nothing else.

---

## 30. AB — SILT: one curve in, one outlet out

The *what and why* is [docs/23](../docs/23-context.md); [12 §5](12-v2-arc.md) fixes
the shape. Silt is **the one mechanism of the arc that writes nothing at all, not
even an ask**, and the sentence is mechanical: `si-silt ↛ si-store`, so the table's
keys are `u32` point handles and the crate cannot name a row if it wanted to.

**AB.1 Tick (step 5a, before the walk — law 27).** For the active conversation's
table, in point-handle order:

```
d_u8    = LAMBDA_SILT + (255 − LAMBDA_SILT) · Clock::recency_u8(now − last_touch) / 255
level'  = mul_q12(level, d)          // reciprocal multiply, no division (OD-45)
level' < THETA_SILT ⇒ level' = 0 and the entry drops
```

`Clock::recency` is **the** build's only decay curve (R-29); `LAMBDA_SILT` is a
*share* of it, exactly the treatment `DIM_BASE` already gets
([03 §5.1](03-constants-registry.md)), which is why `check_one_clock` widens to
`si-silt` and still finds one implementation. Deposits run at step **7**'s tail:
`level' = min(SILT_LEVEL_MAX, level + SILT_BUMP)` with `last_touch = now`. Decay
before, deposit after is the same one-round lag as OD-29's, and it is the reason
G16(d)'s drop order is **strictly last-touch order** — a pure function of the
transcript, with no level comparison anywhere.

**AB.2 The one outlet.** `silt::view(conv) → &BTreeMap<u32,u16>` has exactly one
consumer, `walk::ignite`'s seed choice, and the fifth term is the whole of it:

```
terms = [recency, bake, deficit, warmth]
      → [recency, bake, deficit, warmth, mul_q12(IGN_WEIGHT_SILT, silt_level)]
```

with `silt_level = 0` for a point the table does not carry. At the shipped species
default (`IGN_WEIGHT_SILT = 0`, **OD-108**) the addend is numerically 0, the Q12 sum
is identical, and the cumulative scan picks the same seed for the same draw — so
every pre-arc gate replays byte-identical with the row present. `last_lit` ordering,
the RNG stream `R`, one draw per quiet tick, and the four TEMPERAMENT draws are
untouched; a 19th draw is refused in [03 §9](03-constants-registry.md).

**AB.3 What the table never is.** Not a ranking (`SILT_MAX` is a **counter in the
audit readout**, never an eviction policy — evicting by level would make the
sediment a second sampler); not a modulation input (the per-round spread reads **no**
silt — OD-104's option (a), `check_silt_ignition_only`); not arousal-scaled (OD-107:
absent from the code, not zero); not source-weighted (OD-106 ships parity — a
stranger's line deposits what an owner's line deposits, because silt is *context*,
and context that grades its speaker is provenance in disguise).

**AB.4 Per-conversation keying.** The table is `BTreeMap<conversation, BTreeMap<point,
level>>`: warm-up is **per conversation**, which is the whole of OD-109's shipped
half, and the conversation key is the same `u32` the framed sense record carries
([01 §9](01-data-schemas.md)). No cross-conversation term exists: silt does not
"generalize", and a level is never summed over conversations.

**Ledger items raised by this section:** OD-102…OD-109. **OD-103**'s answer is
*hash-exempt declared session state*, so `silt.jsonl` differs across on/off runs
while the hash does not — which is asserted as a leg (G16(a)), not assumed.

---

## 31. AC — EPISODIC: the derived index, and a re-light that writes nothing

The *what and why* is [docs/24](../docs/24-episodic.md). There is **no crate** for
this family: the view is `si-read`'s (§17 of [02](02-module-contracts.md)) and the
traversal is a rule inside `walk.py`, which makes it the cheapest thing in the arc
to delete — **G17(b)** is a grep, not a migration.

**AC.1 Index build (step 5b).** `read::event_view_index(rows, stamps, eventlog)`
returns `Vec<OriginTug>`, where `OriginTug { event_id, points: [u32], weight_q12 }`.
`event_id` **is** the sense row's `y` — no new id space, no new shelf, no
`prov-link` row ever appended (**OD-110** ships *derived*, so shelf 11 stays
reserved and its `row_schema_version` stays 0). The build is one pass over committed
rows' provenance at frame load; `weight = PROV_LINK_TUG`, registry-low, **never
modulated by `a`**. `EPISODIC_TRAVERSAL = off` ⇒ the vector is empty and step 6's
arithmetic is bit-identical to B13's.

**AC.2 Re-light (inside step 6, round 0).** In `(event_id, point)` order, capped at
`RELIGHT_MAX` per frame: the tug lights its points at
`mul_q12(weight, level_of_belief)`, **read-only** over the belief's live level. The
over-bound degrades by omission — fewer re-lights, **never** a truncated walk (law
19 again). Downstream the re-lit point is an ordinary light: it spreads, it damps,
it is quantized, and §20.2's existing conduction term is what lets a re-lit row's
`a_at` (then) differ from `a_now` (now). **No new curve** — that is the whole of
OD-111's answer, and `check_one_clock` is the judge.

**AC.3 The write ban, in one line each.** No `Stamp` is constructed (so no
`src = 2 idle-re-light` is ever emitted); no `bake` is assigned; no `confidence` is
written; no edge list extends; no `questions` row appears. **G17(d)** grades this
*structurally* — same stamp-arena record count and byte length after N re-lights,
values free, the way OD-88 taught this repository to grade a stamp leg — and
`check_relight_writes_nothing` makes the same claim statically. OD-112 rides
**OPEN-24**, which is off, and the gate report prints that as the reason.

**AC.4 What a re-light may be used for, and what it may never be called.** The
monologue and the labeler may **cite** an event view (both citations printed,
`FACT/OWNED`, per §21's copy-not-create law), and G17(f)'s needle set (`felt`,
`flashback`, `remember how`) must be **absent** from RENDER output — recall is a
fact about rows, not an experience. **G17(e)** is grimm isolation: grimm event views
re-lit N times, zero writes, plus the absence grep that no tug names a `context` row
whose `prov_speaker` is `1` as a **belief origin** (R-35's shape, one notch out).

**Ledger items raised by this section:** OD-110…OD-115. **OD-114** and **OD-104**
both claim the tug↔silt edge, so it is **unbuilt** and `PROV_LINK_TUG` reads no silt;
nobody may "unify" them while both are open.

---

## 32. AD — SKILLS: roles, a canonical template, two bars, one forwarding rule

The *what and why* is [docs/25](../docs/25-skills.md). A skill is **a row, not a
program**: `si-skills ↛ si-learn` and `si-skills ↛ si-walk`, and the only thing the
detector can hand the loop is a value.

**AD.1 Role extraction (step 7b, from the same frame log as §29).** For each lit
point in the frame, `Role ∈ {seed, lit_a, lit_b, shared_filler}` is assigned from
**the round the point first lit** and the **direction** of the edge that reached it —
both already in `CoLight`, so no second trace exists. The shape's **root role** is
the seed role's most-recently-lit member, and that handle becomes the skill row's
`key_point`, which is what makes a skill *addressable in the walk's handle space*.

**AD.2 The template.** The role/kind adjacency set is sorted and canonicalized to a
**folded string** (`seed|lit_a|shared_filler:2`-shaped), interned, and the intern
id is what the minted row carries in `stim_signature`; `stim_surface` carries the
same id, because a skill has no raw stimulus. Its speakable surface is its
**name** (`resp_surface` = the owner's word, or `skill:{hash(role_graph)}`), so the
existing render-containment law — tokens must be a subset of the row's surfaces — is
what makes an unnamed skill unspeakable. **Zero new vocabulary** (G18's no-word-leak
leg is the same needle set as G15(f)'s).

**AD.3 The two bars, in this order.**

| # | bar | note |
|---|-----|------|
| 1 | **generality**: the shape must have been seen over `≥ K_FILLER` **distinct filler sets** | `lo = 2` is not a courtesy: "a move that only ever connects the same three points is a concept, not a skill" — and a concept is §29's object, not this one. This bar runs **first** because it is the one that decides *which detector owns the frame* |
| 2 | **recurrence**: `≥ R_MIN_SHAPE` distinct **cold** frames at `≥ DELTA_MIN_SHAPE` apart | §22.3 rule 3 verbatim, and §29's cold-frame rule (AA.3) applies here with the same meaning: a primed frame counts nothing |

**AD.4 The forwarding rule (OD-121).** A shape whose recurrences all share one
filler set is **dropped from the skills table and forwarded** — that is, the frame
stays visible to §29's fold and becomes a cluster candidate instead. The exclusion
is evaluated **before** the recurrence bar, so **one frame is counted by exactly one
detector**: the same transcript may not feed two asks about the same evidence, which
would be G15(e)/G18(e) spam through a door neither of them opened. Forwarding is a
session-side decision at 7b about the session's own tables; it is **not** a message
between detectors — the two crates have no edge to each other and must not gain one.

**AD.5 Outcome tags, and the composition preference.** Outcome is read from
`eventlog` kinds only, closed set `{question-resolved, realized-gain, owner-reply,
none}` — never from `a`, never from a label. Usage counters are **derived** over
rows plus the log (**OD-120**, R-28's shape): there is no `times_consulted` byte to
stale. `SKILL_COMPOSITION = on` reorders **candidate iteration** in the walk by
(skill-matched, then the existing tie-break) and nothing else — no weight, no
threshold, no sampler, no RNG draw changes (§19's precedence is untouched), which is
OD-118's recommended half and its only buildable half. Detection and use are two
rows precisely so that one may be on while the other is off and both states are
gated.

**AD.6 Mint.** A skill row is written by **step 11's ordinary commit**, through the
same ten-step pipeline, with `prov_kind = 3 answer-to-ask`, the naming line as its
`text_ref`, `confidence = 95`, `gain_kind = 1 demonstrated` (§8.2's own mapping) and
`arousal_at_ask = 255` — the I8 assertion living in the row rather than in a
sentence. Retirement is by supersede, never erase (I4).

**Ledger items raised by this section:** OD-116…OD-122, and **OD-133**'s kind-blind
stage criterion is what lets a skill row sit in the same L3 stage as any other.

---

## 33. AE — WORLD: a pure function, one refusal arm, and a lane

The *what and why* is [docs/26](../docs/26-world.md). This family adds **no crate**
either: `world::class` is a function in `si-core`'s leaf — reachable by `si-front`
and `si-learn` without either importing the other ([02 §17](02-module-contracts.md)) —
and the multi-voice world tonight is a harness scenario, not a transport.

**AE.1 The class function.** `class = f(identity, owner_identity, grants)`, four arms,
first match wins:

```
identity == owner_identity                      ⇒ Owner
identity == grimm's registered id               ⇒ Grimm
grants.iter().find(g.subject == identity
                   && g.live_at(now))           ⇒ Trusted
otherwise                                       ⇒ Stranger
```

Computed **at generation** of each line and stored **nowhere** — not in a row, not
in a cache, not in an export, not in a checkpoint (**OD-123**'s shipped half,
docs/12's rule (ix)); `check_speaker_class_derived` fails the build on any container
that carries a class. `grants` is **empty at birth** (OD-125: coded, not granted),
so `class` never returns `Trusted` before a scenario mints a grant row, which is
what keeps every pre-arc gate bit-identical. The `grant` row type ships **declared
and unwritable** outside the owner lane.

**AE.2 The refusal arm (step 11, ahead of the existing ten steps).** A belief-row
write whose derived class is not `Owner` returns
`Refused{SpeakerNotRepresentable}` with a log line and **zero bytes**. Under
`WORLD_TIERS = owner-only` the arm is unreachable except by a granted line, and a
granted line cannot exist while OD-135 is open: the three `prov_speaker` values have
ruled scopes and a granted teacher is none of them, so the honest build **refuses
and reports** rather than minting a fourth value. **The refusal is G19(c)'s pass
condition** and OD-135 is printed in `observed` as the reason — a named absence,
never a silent skip.

**AE.3 Tier-strict arbitration (§15's tiers, one new axis).**
`Owner > Trusted > Stranger-sense`, and it is **strict**: a Trusted correction
supersedes Trusted rows only, never an Owner row, and a stranger's line can never
raise `repeats` or a confidence — that is R-35's own sentence re-graded over
identities, which is why **G19(a)**'s stranger-silence leg reuses G12's fixture
shape. Sense still happens: a stranger's `context` row **does** append (§21.4's
birth muteness, unamended), and a run that suppressed those rows is reported as a
*different design*, not a stricter one.

**AE.4 The lane, the cap, and the window.** Every `origin = 2` nomination ask lands
in the **owner lane** and is answerable nowhere else (**OD-128**; the lane *is*
`key_field`'s origin half, so no envelope byte moves — [01 §4.3](01-data-schemas.md)).
The ask budget is **one** `ASK_CAP_PER_INPUT` plus the **one** `DEDUPE_WINDOW` over
every gap kind, so N voices do not buy N asks. A declared retention/quiet window may
appear **only** as a deny conjunct on step 12's existing
quiet-and-empty-agenda predicate (**OD-126**): there is no path from a window to
`ignite`, `sleep::due`, `clock.advance` or a frame read, and **G19(g)**'s starvation
leg asserts that no tick ever occurs *because* a window opened. Owner preemption
(§19's precedence, law 13) is untouched.

**AE.5 The speak gate is reused, not re-derived.** Under the grant scenario a Trusted
line passes the **same** two-layer filter (§23 W) and the **same** token-membership
test that grimm's lines pass, over the same taught lexicon. That is the whole of
**G19(h)**, and it is why delegated authority is cheap here: one function, one
filter, one class — and the class is a return value.

**Ledger items raised by this section:** OD-123…OD-132, plus the two engineering-
found seats, **OD-135** (no honest `prov_speaker` for a granted teacher — the write
is refused) and **OD-136** (who mints `source: grimm` and `speaker_identity` once an
adapter exists — not built tonight, recorded rather than answered in code).

## 34. AF — TONGUE: a drawn voice, one bounded phrase walk *(B20;
[docs/27](../docs/27-tongue.md), [13 §5](13-tongue.md))*

The law file's section numbers map in order: **13 §34.1 → AF.1**, §34.2 → AF.2,
§34.3 → AF.3, §34.4 → AF.4. Nothing here is a new curve, a new field or a new
constant shape — the derivation is `derive_temperament`'s, the sampler is §19 S.3's,
and the accumulator is `idle.rs`'s.

**AF.1 The style derivation (seed-only, `si-core::tongue`).**
```
style_stream(birth_seed)  = Rng::seeded(birth_seed ^ TONGUE_DOMAIN)
TONGUE_DOMAIN             = 0x544f_4e47_0001_0001   // "TONG" + a format tag, a fixed constant
derive_style(seed)        = for (i, id) in DERIV_ORDER_TONGUE:
                                map_fraction(stream.next_q12(), row.lo, row.hi, row.grid)
Style::canonical_bytes()  = 5 × u16le  (10 bytes)
Style::digest()           = first 8 bytes of MD5(canonical_bytes), LE, as u64
```
**Why a domain-tagged stream and not stream T's continuation:** `birth_streams(s)`
returns two clones of `Rng::seeded(s)`, so continuing T's sequence means skipping
exactly 18 draws, which makes `STYLE_SPREAD` a function of `DERIV_ORDER`'s length —
a future OD-71 flip that moved one temperament row would silently re-voice every
individual. The salt makes the two orders independent by construction, and because
it is a *constant* the derivation stays a pure function of
`(birth_seed, DERIV_ORDER_TONGUE)`: the same sentence 10 §4's header writes about the
18, and the same scan `check_tongue_closed` runs. **Stream R is not read at all, and
`runtime_stream` is untouched** — a voice must not move the ignition history for a
reason that has nothing to do with ignition.

**AF.2 The render seed (OD-183).**
```
render_seed = first 8 bytes of MD5( birth_seed ‖ style.digest() ‖ thought_id ), LE
draws       = Rng::seeded(render_seed)          // per render, never the session's R
```
Same seed + same transcript + same lattice ⇒ the same sentence byte-for-byte across
processes (**G20(a)**); I5 holds because every operand is a snapshot value. The state
hash is **not** an operand: item 3 of the nine hashed items is the stamp arena and a
stamp carries `a_at`, so seeding from the hash would let arousal choose the words
through the back of the seed — what **OD-168** bans, graded by **G20(e)**.

**AF.3 The walk, step by step.**
```
0  anchor            = the answer row's val_point (an ask's named target is not in
                      scope this round: 13 §4.1 realises Output::Answer only)
   used              = { anchor }
   parts             = [ license(chunk_of(anchor)) ]     // the canonical copy, first
1  budget            = (PHRASE_MAX_STEPS × STYLE_SPREAD) >> 12       // a share, 03 §5.4
2  while parts.len() < 1 + budget:
3      candidates   = graph.arcs(current)
                       .filter(|a| !a.is_genome())                   // OD-179: live arcs only
                       .filter(|a| license covers a.to_point)         // OD-181: utter-able
                       .filter(|a| !used.contains(a.to_point))        // no repeats
4      if candidates is empty ⇒ break                                 // a refusal is not a crash
5      weight(a)      = a.weight                                     // Q12, the lattice's say
                      + mul_q12_u16(STYLE_DEPTH,     owned_share(a.kind))
                      + mul_q12_u16(STYLE_LEAN,      degree_share(graph.out_degree(a.to_point)))
                      + mul_q12_u16(STYLE_RECENCY,   recency_share(a.bake, a.edge_off))
                      + mul_q12_u16(STYLE_PROXIMITY, hop_share(distance(current, anchor)))
6                     // u64 accumulator, saturating — idle.rs's shape exactly
7      draw          = one next_q12();  target = (total × f) >> 12; scan the cumulative list
8      append        = that point's licensed chunk; current = that point; used += it
9  line             = parts.join(", ")                                 // the only literal
```
The four shares are **structural, not tunable**, and each is a Q12 in `0 … 4096`:

| operand | share | law |
|---------|-------|-----|
| `owned_share(kind)` | `4096` if `edge_kind_is_claim(kind)`, else `1024` (`6 cooccur-weak` and every genome kind) | [docs/27 §27.3](../docs/27-tongue.md): formality = claim-side over associative. 1024 not 0 because a co-occurrence is still an edge SI earned. `edge_kind_is_claim` = `owned && kind != EDGE_KIND_COOCCUR_WEAK`, the same rule [13 §34.3](13-tongue.md) names and `si-front::label::is_claim` already writes: `edge_kind_is_owned` is the **record-shape** predicate and its last slot *is* the co-occurrence, so it cannot be the formality predicate (**OD-186**) |
| `degree_share(deg)` | `min(deg, 16) × 256` | [docs/27 §27.3](../docs/27-tongue.md): playfulness walks into the hub. 16 buckets, the last saturating, so a hub of 40 and a hub of 16 are one preference |
| `recency_share(bake, edge_off)` | `clamp(bake >> 4, 0, 4096)`, plus `1024` when `edge_off` is above the median of the current arc set | [docs/27 §27.3](../docs/27-tongue.md)'s "recently-baked / recently-charged", realized with **no `a` operand** (**OD-182**): `bake` is the write-side permanent quantity and arena position is formation order. `a_at` is never read |
| `hop_share(d)` | `4096` at d = 1, `2048` at d = 2, `1024` beyond | [docs/27 §27.3](../docs/27-tongue.md)'s particle affinity: how tightly it sticks to the thing asked about |

`distance` is the walk's own hop counter, one `u8` on the stack, never a new field on
any structure — the graph stores no metric and the tongue invents none.

**Termination is three-sided, and each side is a proof:** `budget` (a species bound ×
a Q12 share ≤ `PHRASE_MAX_STEPS`), `used` (a point is emitted at most once and the
license is finite), and empty candidates. So no style value can make the walk run
away — that is `G20(i)`'s whole predicate — and the line is bounded by `1 + budget`
chunks.

**AF.4 The three checks, as code shapes.**

| check | predicate | on failure |
|-------|-----------|-----------|
| (a) label immutability | the type fact of [02 §11](02-module-contracts.md): `realize` takes no `&Rendered`, no label, no tag, no citation | — (nothing to fail) |
| (b) licensed emission | every token key of `line` is in `⋃ license[i].keys` ∪ the joiner's (empty) | `None` |
| (c) parse-back | `folded(line) ⊇ folded(canonical surface)` — every semantic slot the line was generated from is still in it (**OD-173**: slots-present; no decode walk is invented for this) | `None` |

`None` is the arm at the call site: `rendered.surface.clone()` — the canonical frame,
printed with the label, tag and citation the run already computed. A refusal moves
nothing else: no row, no log field, no gap, no ask (**G20(g)** forces the loss and
asserts exactly this, on the *frame*, not on a crash).

**Ledger items seated by this section:** **OD-163** (the five style rows and the
additive-lean shape), **OD-179** (live arcs only), **OD-181** (the license is the
only mouth), **OD-182** (recency without `a_at`), **OD-183** (the seed's three
operands), **OD-184** (two imports, not three) — each implemented at its
least-authority reading, all of them in [13 §10](13-tongue.md).


