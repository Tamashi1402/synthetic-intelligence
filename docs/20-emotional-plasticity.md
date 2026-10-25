# §20 — EMOTIONAL PLASTICITY THROUGH TIME

Owner DESIGN LAW, round 3 of the affect batch (same batch as §18's demotion and
§19's ignition; it amends both). The principle in the owner's words:

> **No word has a fixed emotional address.** Any word/meaning can move any point
> up or down, and its CURRENT emotional reading is stitched through time —
> computed at generation from its whole stamped history. **The median is the
> readout, not the last event.**

What that adds to the control chain, and what it does not change:

```
raw signals (per EVENT, per TOKEN) ──▶ a (live, integrates during the walk)
                                          │  modulation lags ONE ROUND
                                          ▼
                                    walk modulation                (CONTROL)
 signals ─▶ stamp on the L3 edge at formation ─▶ BAKE = B_max·tanh(k·a_formation)  (write-side, durable, per-edge, BOUNDED)
      a ────────────────────────────┘  (the one arrow from the tissue into the write path — OD-33)
                all of a point's stamps, aged on the fact clock ─▶ MEDIAN (read-side, per-point, recomputed at NOW)
                                          └──────────▶ AUDIT readouts (names, zero causal power)
```

**Nothing in this file installs an emotion.** §18.1's inventory grows by three
*form laws* (live integration + one-round lag, the median, the bake) and zero
content — plus, from the §20.4a amendment, a **fourth**: the bake's **saturating
scale** `B_max · tanh(k · a_formation)`, one curve over the same scalar, still no
category test, no valence, and no emotion name in it. `B_max` and `k` are swept
constants (OPEN-33), not opinions. The label law is untouched: named emotions
stay audit classifications with no causal path.

## 20.1 LIVE INTEGRATION DURING THE WALK — arousal is per-token, not per-thought

Signals are per-event by nature, and events happen **inside** a thought:

| event during the walk | the signal it fires (§18.1) |
|-----------------------|-----------------------------|
| a token lands on a dark spot | gap / novelty — the spread died where nothing exists (§7.1) |
| a question frame lights | gap — the frame demands a slot SI may not have |
| a hot-stamped edge conducts | **that stamp's own signal kind**, again, at its current value |
| a round ends with no candidate above threshold | settle-failure pressure (§16.1) |

**These do not wait for the thought to end.** Arousal integrates live, token by
token, **during** the walk, so `a` at round 1 and `a` at round 5 of the *same*
thought are different numbers and the rounds after a token conduct differently
from the rounds before it. **The sentence colors itself as it is read.**

Worked shape (mechanism, not content):

```
"What is monster?"
  "what is"  → question frame lights → gap signal → a rises
  "monster"  → no L3 node (or a freshly-decoded one) → novelty/gap fires
               → a shifts MID-WALK → rounds after "monster" spread with the
                 higher-a knobs (wider reach, faster decay, lower settle
                 dominance) than the rounds before it
```

- **LAW (unchanged, and this is the point):** what causally moves is
  `signals → a → walk modulation`. `Monster → scared` is **LEARNED, not
  installed.** At birth an unknown word fires **only** gap (novelty). Fear
  arrives only **via stamps on its edges** — a taught `monster is scary`, or a
  contradiction burned in. **A newborn frightened by "monster" is smuggled
  content and is a design violation** (§3's litmus, R-15/R-17, G9(f)'s parity
  leg: `znark` and `monster` fire exactly the same gap-only pattern).
- **RATE KNOB (owner-set lean, version-tagged):** arousal moves **fast on
  spikes**, but **modulation lags one round** — the whole thought re-shapes only
  *after* the token lands. The sentence is colored, **but the frame still
  finishes its job first.** No emotion mush: a mid-thought spike cannot
  re-steer the round that is already conducting. The lag is a
  **form constant** (OPEN-29), swept before quoted, same discipline as §16.5.
- **A dark spot is now visible twice, and the two roles do not merge:**
  (i) **live**, as an arousal signal the instant the spread dies there — this is
  what §20.1 adds; (ii) **at settle**, as a **nameable slot** the ask engine can
  emit (§7.1's detector still reads the finished trace, because naming a slot
  needs the settle to know which intersection went dark). Detection ≠ asking is
  unchanged (I7); asking still waits for the thought to end.
- **Idle cascades have no tokens** (§19): there, integration is per round and
  per signal row only — a cascade that crosses its own live contradiction raises
  `a` **inside** the idle thought, which is §19.5's loop with a clock on it.
- **Logical time keeps its grip.** Gains are applied at token/round events;
  decay is applied on the walk's own round clock. **Which clock unit is the
  official one for `λ_a`** (per token, per round, or per thought) is a
  consequence this chunk does not state, so it goes to the ledger as **OD-29**
  rather than being invented here; the lean is **gains per token, decay per
  round**, because the modulation lag is itself defined in rounds.
- **Door contract, restated not grown (§16.1, OD-20):** the walk still calls
  exactly two drive *interfaces* — `drive.update(...)` and
  `drive.conduction()`. Round 3 changes **how many times per thought**
  `drive.update` is called (once per thought → once per signal event), not the
  number of doors or what may pass through them. Numbers in, one scalar out;
  no name ever crosses.

## 20.2 STATE-DEPENDENT RETRIEVAL — teach-state is part of the address

The emotional state **at teach time** is part of an answer's address. Edges
written while the loop was hot carry that hotness in their stamp, and generation
running in a **matching** state conducts them easier, so the settle lands closer
to the answer bound at learn/teach:

```
taught scared, asked scared  →  the taught edge conducts cheap → tighter settle
taught scared, asked calm    →  the same edge competes on conduction alone
```

**The more state context fed into generation, the closer the settle to the bound
answer** — human state-dependent memory, now with a mechanism instead of a
metaphor.

- **Schema consequence (unstated by the chunk, so ledgered):** matching
  *state against state* needs the arousal level that was present at the write.
  Round 2's stamp is `{signal, magnitude, thought_id, signal_ref}`, where
  `magnitude` is the **signal's own value**, explicitly *not* a function of `a`
  (§18.5 — and §20.4a's amendment reverses exactly that half of the law, under
  **OD-33**). Round 3's claim needs one more quantized scalar on the stamp —
  **`a_at`, the arousal level at formation**. Recorded as **OD-28** for the
  owner's ruling; this doc writes the schema with it, because without it leg
  (a) of G11 is not addressable at all.
- **It is still cost, not content.** The bias cannot make an untaught link
  exist; it changes which taught connections are cheap this thought (§7.3). The
  goal, the query, and the honest labels are untouched, and the term ships its
  attribution in the activation trace ("this edge was cheap because the current
  state matched its stamp at t=…").
- **Monotonicity is capped, not free.** "More state context ⇒ closer settle"
  holds **only inside §18.5's per-round normalization**: the bias contributes a
  fixed share of each round's conduction. An uncapped reading of this sentence is
  exactly R-22 (runaway) with a nicer mechanism attached, so G11(a) is measured
  against the cap, not around it.
- **Parity:** `a_at` is one quantized scalar plus a signal kind. It names no
  emotion, no referent, and no valence; `what is znark?` written at `a=0.7`
  stamps identically to any other word written there (G9(f)).

## 20.3 THE MEDIAN — read-side emotional weight, stitched through time

A point's (word's/meaning's) **current emotional reading** is the
**recency-weighted MEDIAN over its stamped history** — every stamp on every edge
touching that point, through time — **default anchor NOW**.

- **Why MEDIAN, not mean: means get hijacked.** One screaming-hot stamp owns an
  average forever. A median needs **repetition** to move. So **no word flips on
  one comment**; it shifts only when **most of its recent history** says so.
  Same law as everything else in SI: **nothing moves by decree, everything by
  accumulation.**
- **Second runaway guard, free and not a mechanism.** The per-round cap (§18.5)
  is the *mechanism* guard against one stamp dominating one thought; the median
  is the *statistical* guard against a mood spike rewriting the system across
  thoughts: a hot spike biases the walk and stamps a few edges, **and the median
  resists**. **A bad hour cannot rewrite the personality.** Self-stabilizing by
  statistic — no new supervisor, no new dial, nothing to switch off.
- **SAME CLOCK (law, and it is structural).** Stamps age on the **same recency
  curve facts use** — the curve that already produces §19.2's recency weight and
  §8.4's decay-deficit floor. **One clock for the whole system: SI cannot
  remember a fact better than it fears it by tuning accident.** Enforcement is a
  dependency rule, not discipline: the module that computes the median
  **imports** the recency curve and **defines none of its own** (§12, R-29).
  The chunk cites "G4's clock"; in this plan the time-axis gate is **G3**
  (supersession) and G4 is contradiction — the numbering clash is **OD-30**,
  with the extension applied to G3 here.
- **PER-POINT, not per-edge.** The median is the word's current mood-bias.
  Per-edge weight stays where it lives: the baked conduction of §20.4.
- **THE MEDIAN IS NEVER STORED.** It is computed read-side, at generation, from
  citable rows. It is **not** a row type, **not** a shelf, **not** a field
  (§5.2's registry law unchanged; the Tesseract stays append-only). A cache that
  survives a thought is a **frozen emotional address** and is the failure this
  section exists to prevent — the reading must be recomputed from history every
  thought, so recomputation-equal-to-last-thinking is a bug, not an
  optimization (**R-28**).
- **What it feeds:** a **point-level** term in the walk's conduction, alongside
  the per-edge bake term, under the same per-round normalization and the same
  trace attribution. Its exact shape (additive into `W_eff`, a threshold shift on
  the node, or a lighting weight at round 0) is **OPEN-30** — all three are
  form-only, and none may reach anything except spread cost.
- **Anchored anywhere in time.** The same statistic evaluated with the anchor at
  a past `t'` is the word's emotional reading **then** (§20.5).
- **Zero history ⇒ no reading, not a neutral one.** A point whose edges carry no
  stamps has **no** emotional bias term at all; the term is absent, not zero-
  valued with confidence. The audit layer may print no label over such a point
  beyond what its live signals say. This is §18's birth litmus restated for the
  median: at t=0 in CONFIG SI every median is undefined and **nothing fires**.

## 20.4 THE BAKE — trauma is lawful and durable

The single hot event gets its durable artifact **at formation**, per §18.5's
close-out: **conduction head start + confidence bump + permanent stamp** (§8.4's
salience-weighted init — and §20.4a fixes what "salience" means here: the
**saturating** curve `B_max · tanh(k · a_formation)`, a function of the one
integrated level, of no signal kind and no per-signal magnitude table (round 2's
`salience = f(signal)` was the pre-amendment form; **OD-33** is the ledger entry
that changed it).

So **one genuinely scary moment makes that specific edge hot forever** — even
while the word's median drifts calm.

**THE SPLIT (owner-set).** Two artifacts, two sides of the pipeline, two clocks:

| | **MEDIAN** | **BAKE** |
|---|---|---|
| side | read (bias at generation) | write (init at formation) |
| granularity | per **point** | per **edge** |
| drives | the word's current mood-bias — **personality**: slow, repetition-driven, recomputed at NOW | the edge's conduction head start + confidence bump — **trauma**: durable, set once, ages only through supersession |
| moves when | most of the recent stamped history says so | a signal fires at the moment the edge is formed |
| can it be re-edited | never — it is a statistic over append-only rows | never — it is a birth property of that edge, **and it never decays (v1)** |
| guard | repetition requirement + one shared clock | **§18.5's per-round normalization (read) + the saturating scale at formation (write) + the §16.6 quantized grid** |

**Who is allowed to bake, now that there are two voices in the channel
([docs/21](21-grimm-supporter.md) §21.4, I10).** The table's "moves when" row
says *a signal fires at the moment the edge is formed* — and the row needs its
subject named, because a durable, permanent quantity is exactly what you do not
want to hand to a process whose weights came from somebody else's corpus. Two
cases, same answer, different reasons: an **idle cascade** forms no edge, so it
buys none (§19.4, G10(b)) — the reason is mechanical; a **`source: grimm` line**
is not a teaching event at all (§21.4) — the reason is provenance, and it is the
one that had to be written down before the build. Consequences worth their
weight: a calm, echoing parent cannot give any edge a head start, cannot move
`a_at` on a row that does not exist, and cannot make tomorrow's intrusive
thoughts louder, because ignition's affective weight reads the bake (§19.2) and
he never gets to write it. **Co-regulation reaches the walk, not the lattice.**

**The human shape, verbatim:** *"stoves don't scare me anymore, but I still
don't reach over the flame."* The point's median calmed; that edge stayed hot.
Both readings are true at once and both are citable.

### 20.4a THE BAKE SCALE — saturating, not linear (amendment, same batch)

The bake is **scaled by a saturating curve of live arousal at formation**:

```
edge.bake = B_max · tanh(k · a_formation)        # soft cap; B_max, k version-tagged
a_formation = the live `a` at the instant the edge formed (§20.1) = the stamp's `a_at` (OD-28)
```

- **Why it saturates:** *"Unbounded arousal at formation must not create an
  unbeatable edge forever — the first kitchen fire and the tenth both score near
  the cap; nothing ever scores an 11 that owns the walk."* A permanent quantity
  that is also **unbounded** is the worst pair of properties a lattice can hold:
  one hot moment would outvote every later thought, forever, and no amount of
  calm repetition could out-argue it. So the write-side now has its own bound,
  independent of §18.5's read-side cap: **the bake is durable AND bounded, and
  the bound is what makes durability lawful.**
- **CLASH, ledgered not smoothed (`OD-33`):** this makes the write-side magnitude
  a function of `a` at the formation instant — and round 2 ruled the exact
  opposite. OD-22 stripped the drive's `salience` door of its affective input and
  the law went out as **"the stamp magnitude is the firing signal's own value,
  NEVER `f(a)`"** (§8.4, §17.1, §18.5, docs/04 s6, docs/03 R18 — the write side
  was the one place arousal was forbidden to touch). The newer owner statement
  wins, so every one of those lines is re-pointed here rather than quietly
  flipped: **the bake is `f(a_formation)`, saturated.** OD-22 survives intact for
  `budget`, for read-time, and for "arousal modulates the walk"; what it loses is
  its write-side half. The distinction that keeps this from being a plain
  reversal: `a` is the **leaky integrator**, so `a_formation` carries the residue
  of events minutes earlier, not just this one — see OD-33's option (b), which is
  the strict-local reading and the only version OD-22 would still permit.
- **`a_at` gets its job.** This is the field OD-28 was opened for, now with a
  second reason to exist: it is not only the read-side state-match evidence, it
  is **the argument of the curve that sized the bake**. Neither use makes it a
  read-time multiplier — at formation it *is* the magnitude, at query it is only
  compared (§18.5's law holds exactly: nothing at read time scales conduction by
  `a_at`; what scales it is the already-formed bake).
- **The impact score is EMERGENT, not categorized.** What raises `a` at the
  moment is the live sum of the signals already integrated during the event —
  **novelty + contradiction + negative gain + settle failures** (§18.1's table,
  nothing else). **"Nothing categorizes fires as bad — it felt a lot when it
  happened."** There is no `is_trauma()` test, no event-kind whitelist, no
  threshold on signal *type*: the bake reads one number that the loop already
  computed for its own reasons. The anti-cheat law is untouched — and note which
  way this cuts on **OD-31**: a *bad* event raises `a` rather than lowering it,
  so the batch's own machinery uses intensity (magnitude, one axis), not valence.
- **CLOSES the open decay question (this is a ruling, not a gap).** Round 2's
  "the boost decays" is dead and **round 3's option (b) — a slow half-life on the
  bake — is refused**: the trauma bake gets **NO decay tail in v1**. Permanent
  bake, saturating scale. **Only the median moves.** **OD-27 is therefore
  RESOLVED** as recorded; if the owner later wants a half-life, that re-opens
  OD-27 and re-opens R-23 with it (a decaying write-side field is exactly what
  G6's freeze law was built to keep out of L1/L2).
- **What saturation buys, and what it costs (published limit):** once an event
  scores near `B_max`, *later events of the same kind stop distinguishing* — the
  first and the tenth fire bake the same height. That is intended (trauma does
  not compound in the substrate; **compounding lives only in the median**), but
  it has a sharp edge: **if `k` is large, every hot moment saturates and the
  write-side stops encoding anything** — a lattice where every bake equals
  `B_max` is a lattice with no salience information at all, which is the same
  failure as a flat gain matrix wearing a curve. So **OPEN-33 is not just "find
  two numbers"; the sweep's objective includes non-degeneracy** (the bake values
  across a session must keep spread over the observed `a_at` range). The mirror
  risk is the opposite: `k` so small that a calm SI bakes nothing, ever — which
  is lawful but means formative sessions leave no durable salience at all.
- **IGNITION inherits the bake as an input weight (§19.2).** A hot-baked edge
  fires its endpoints more readily in idle ignition, so *"worry, rehearsal, and
  the 3am kitchen-fire replay are all the same mechanism"* — one weight term, no
  `worry()` entry point, no scheduler, and **OPEN-32 is CLOSED by that naming**:
  the ignition salience weight reads the **bake**, neither the raw stamp list nor
  the point's median. It is bounded by the same `B_max`, so a saturated edge
  cannot monopolize the sampler, and G10's rate + per-hour budget still decide
  how often anything fires at all.

- **Amendment this forces (owner-decided law touched → ledger):** round 2's law
  said **"the stamp is permanent, the boost decays"** (§18.5, §5.3, §8.4, docs/09).
  Round 3's bake says the head start is **durable at formation** and that what
  fades is the stamp's **read-side weight in the median**, via the recency clock.
  These are different laws about the same field, so the change is recorded as
  **OD-27** — this doc adopts the round-3 reading (it is the newer owner
  instruction) and marks every place round 2's phrasing was replaced. Failing to
  record it would be the silent edit the merge law forbids. **RESOLVED by the
  §20.4a amendment: permanent bake, no decay tail, saturating scale — option (b)
  refused** ([ledger](decisions/open-decisions.md)).
- **Supersession still wins the route.** A superseded edge has zero live weight
  (§5.3), so its bake cannot conduct at NOW — it is reachable only by an
  explicitly history-anchored walk (§20.5). Durable ≠ currently active.
- **RE-PARSE inherits, never manufactures** (G7's rule survives): a derived fact
  row gets its source stamps as **history only**, with the boost at baseline, so
  the bake cannot be laundered into a fresh head start by re-deriving rows.
- **Idle thought and the bake:** an idle cascade **writes no rows and therefore
  forms no edges**, so it can create **no new bake** (I9, G10(b)). If the owner
  rules OPEN-24 yes, an idle re-light may add a **stamp** to an existing edge —
  which feeds the median — but it cannot change that edge's formation-time head
  start or confidence, because those were set at its birth.

## 20.5 TIME — 4D propulsion: state changes, nothing edits

Stamps live on edges, edges live on the **time axis**, so **emotional history
supersedes exactly like facts do** (G3's mechanism, reused, not a second one):

- new teaching at a new `t` forms **new edges with new stamps**;
- at **NOW** the fresh ones conduct easier (same clock, §20.3), so **a word can
  flip completely**;
- **the scared version is never deleted** — aged at its `t`, citable, walkable.

**EMOTIONAL ARCHAEOLOGY (this is G3's time-axis leg extended, OD-30):**

```
"what did monster feel like back then?"
   → decode to a PAST-t-anchored walk (the same treatment "what color were apples"
     already gets: history query re-weights the aged edge at its interval, §7.3)
   → the median is computed with the anchor at that t'
   → the old fear lights up again, with citations to the stamps that produced it
```

- **Time frames get the same decode treatment as fact frames** — and the same
  law: a past-tense frame is **taught, not innate**. If SI has no edge linking
  "back then" to an interval, the query decodes to an unlinked frame node and SI
  **asks** instead of time-travelling (§7.1's named dark spot, R-14/R-15).
  An SI that answers archaeology questions it was never taught to frame is
  showing a preloaded tense table.
- **Bias matching is "current state vs STAMP HISTORY, recency-weighted."** A
  yesterday stamp beats a last-year stamp at NOW **even if the older one was
  hotter when written**. The older stamp is not weaker in storage — it is older
  in the statistic. Nothing is edited; only the anchor moved.
- **G6 unchanged and stated for the new parts:** stamps, medians and fluid state
  **never touch frozen structure.** The median is computed read-side from L3
  stamps; the bake is a birth property of an L3 edge; L1/L2 have no `charge`
  field at all, so neither artifact is representable there (§5.3 law (a),
  G9(d)). The Tesseract stays append-only end to end.

## 20.6 What this does to the law

- **Emotional readout is now FULLY 4D:** a word's reading is a function of its
  **entire stamped history**, anchored at NOW, moving as state moves. **No
  re-editing ever** — the matrix answers differently **from a new position in
  time**, not from a changed past.
- **The innateness inventory grows, but stays form-only** (§18.1's list becomes):
  the raw signals · one integrator `a` · the modulation curve · **live per-token
  integration with a one-round lag** · **the median law (recency-weighted,
  per-point, one clock)** · **the bake law (per-edge head start at formation)** ·
  **the bake's saturating scale `B_max · tanh(k · a_formation)` (§20.4a)**.
  Seven form laws, zero content, zero emotion names, one scalar, and the write
  side's one curve — which is the item **OD-33** exists to keep honest: it may
  read `a` at formation and nowhere else, ever.
- **The litmus still passes, harder than before:** at birth there are no stamps,
  so **every median is undefined**, `a` is at rest, and the lag has nothing to
  delay. Nothing is feelable without content (**I1**). **`B_max` sitting in the
  config is not a feeling:** with no edges there is nothing to bake, and the
  curve maps a scalar to a scalar without naming either.
- **§18.2 is untouched:** curiosity/surprise/caution/frustration remain audit
  classifications over `(signals, a, walk behavior)`. The median changes what
  the audit layer *observes* (it now also reads per-point bias), never who gets
  to steer. Words like "scared", "trauma" and "personality" in this file are
  **expository shorthand for the mechanism** — they may not appear in any module
  except `audit.py` (§18.6, G9(c)).
- **Embeddings stay banned (§16.3, R-14):** the median is an order statistic
  over SI's **own** citable scalars from this session and its own history. It is
  not a borrowed continuous space, it carries no pretrained similarity, and it
  can be recomputed byte-for-byte from the exported rows by a human with a
  spreadsheet.
- **"No fixed emotional address" cuts both ways:** SI can neither be born with
  one nor be **taught** one permanently. `/correct monster is scary` makes an
  **edge** hot; it does not buy the word a permanent fear, because the median
  drifts back unless most of the recent history agrees. A word that stays scary
  forever in a transcript where nobody keeps teaching it scary is a bug
  (**R-28**), not a memory.
- **Valence flag:** *"can move any point up or down"* reads, in v1's one-scalar
  physics, as **more or less biased on the single arousal scale** — not as a
  pleasant/awful axis. Round 2 dropped the `sign` field as moot (§18.5, OD-18);
  a genuine valence dimension would resurrect it, so this is asked rather than
  assumed: **OD-31**.

## 20.7 G11 — STATE-DEPENDENT MEMORY (CONFIG SI), the gate this batch makes testable

The state-dependent-settle claim is now a design-level, measurable assertion:
**same query, two induced states ⇒ two different settles, both citable to stamp
history.** Legs, as the owner specified:

| leg | assertion | failure means |
|-----|-----------|---------------|
| **(a)** | teach fact **A** under induced high arousal, fact **B** neutral; same query in **both** states ⇒ A's recall is **measurably tighter** in the aroused state (settle dominance higher, K rounds lower) | **teach-state is not addressable** — the write never recorded the state, or the read never matched it |
| **(b)** | **the median resists:** one hot stamp on an otherwise calm history does **not** move the word's bias measurably | **mood spikes rewrite personality** — the mean-instead-of-median runaway (R-26) |
| **(c)** | **repetition moves the median:** N repeated stamped events **do** shift the bias, visibly, **on the same clock as fact recency** | **repetition law broken** — or a second, faster clock for affect (R-29: SI fears something more sharply than it remembers it, by tuning accident) |
| **(d)** | **emotional archaeology:** a past-`t`-anchored query **recovers the old reading with citations** | **emotional history is not 4D** — the past was overwritten, or the statistic is a live-value cache |

Notes on how it runs, so it cannot be faked:
- **"Induced" means produced by real events,** never by a setter: leg (a)
  creates high `a` with a genuine gap/surprise sequence and neutral history with
  quiet repetition. Any external `a` handle invalidates the gate by construction
  (same rule as G9, I8).
- (a) is measured **against** §18.5's cap, so the claim is "tighter within the
  bounded share", not "arbitrarily tighter" — an uncapped pass is a **fail** for
  R-22, not a win.
- (b) and (c) are the same statistic under two histories; they must run on one
  build with one version-tagged constant set, and the sweep lands in
  `gates/G11/median_sweep.json` (**UNMEASURED** until then — N, the repetition
  count and the bias magnitudes are all sweep outputs, none of them design
  inputs).
- (d) reuses G3's artifacts: the same export, the same interval-citability
  assertion, now over stamps.
- **The bake bound is checked beside this gate, not inside it (§20.4a):**
  "nothing ever scores an 11 that owns the walk" is an assertion about the
  **write-side** soft cap, so it rides G9(e)'s bounded-feedback leg (no edge's
  bake above `B_max`, and the bake distribution keeps spread — OPEN-33's
  non-degeneracy objective) and G10(e)'s ignition leg (a saturated edge may not
  monopolize the sampler). G11 stays the read-side gate.
- **Determinism:** G5(a) extends to the whole per-thought `a` **trajectory**
  (the sequence, quantized — not just the endpoint), because §20.1 makes `a` a
  per-token quantity. Replay must reproduce every mid-thought shift byte-for-byte.
- **Cross-gate carry-overs:** G9(a)'s spread-diff, G9(c)'s deletion check (the
  median's point-term must be inside the machinery that survives deleting
  `audit.py`), G9(d)'s zero frozen stamps, G10(b)'s read-only rule (no bake from
  idle), G6's checksums.

## 20.8 The honest limits, published

- **Early transcripts will look emotionally flat.** At S0 few edges carry stamps,
  so most medians are undefined and state-dependent recall has almost nothing to
  match against. That is the correct result; an impressive emotional arc in a
  five-turn demo is a preloaded corpus (**§3.4's empty-state test, R-25/R-28**).
- **A median is a weak statistic on thin history.** With one or two stamps it is
  barely distinguishable from the last event; the "no word flips on one comment"
  guarantee is only meaningful once history is long enough, and how long that is
  is a **measured** N (OPEN-31), not a claim.
- **One dimension only in v1:** intensity on the arousal scale plus a signal kind
  — no valence, no "mixed feelings", no social emotions (§18.7 stands; OD-31).
- **State matching may be too weak to see.** If G11(a) shows no measurable
  difference with `a_at` recorded and the cap respected, the finding is published
  as a negative — β and the point-term shape are swept, not asserted (OPEN-18,
  OPEN-30). A mechanism nobody can measure is not a mechanism.
- **The chunk's "the more state context, the closer the settle" is monotone
  language about a bounded system.** We keep the direction and publish the bound
  (per-round normalization). Unbounded versions of that sentence are the runaway.
- **A saturated write-side is a silent write-side (§20.4a).** Because
  `tanh` flattens, two events either both land near `B_max` — in which case the
  bake no longer distinguishes them and only the median carries the difference —
  or both land near zero, in which case a calm formation leaves **no durable
  salience at all** and the edge is fire-neutral forever. Both are lawful
  outcomes of "it felt a lot when it happened", and neither is fixable by
  category: adding an "important kinds always bake" rule is exactly the smuggled
  ontology R-03/R-21 forbid. The honest position: **v1 accepts a write-side that
  may be uninformative, and measures it** (OPEN-33's non-degeneracy objective,
  reported as a distribution rather than tuned until it looks good).

## 20.9 Open items (develop, don't invent silently)

| id | open item | where it bites |
|----|-----------|----------------|
| OPEN-29 | the modulation-lag value (owner lean: **one round**) and its unit; version-tagged, swept | §20.1, `gates/G9/arousal_sweep.json` |
| OPEN-30 | the median's **point-level term shape** in the walk (additive into `W_eff` / node threshold shift / round-0 lighting weight), plus its share of the per-round normalization | §20.3, §16.1, G11(a) |
| OPEN-31 | what counts in a point's history: superseded edges' stamps? RE-PARSE-inherited stamps? `magnitude` only, or `a_at` too? and the minimum N before "resists" is assertable | §20.3, §20.4, G11(b)(c) |
| OPEN-32 | ~~does ignition's **salience-stamp** weight (§19.2) use raw stamps or the point median?~~ **CLOSED by the §20.4a amendment:** it reads the **bake** — a third answer the two options both assumed. Neither the raw stamp list nor the median | §19.2, §20.4a, OPEN-25 |
| **OPEN-33** | the bake curve's two constants — **`B_max` and `k`** (`bake = B_max · tanh(k · a_formation)`), version-tagged. The sweep's objective is **not** just "a value": it must show **non-degeneracy** (bake values keep spread across the observed `a_at` range — if everything saturates, the write-side encodes nothing) **and** reach (if nothing saturates, a calm session bakes nothing at all). Two failure directions, one sweep | §20.4a, §19.2, `gates/G9/arousal_sweep.json`, G10(e) |

## 20.10 What round 3 changed (round 2 → round 3) — no silent edits

- **ADDED as form laws (§18.1's inventory grows, content still zero):** live
  per-token arousal integration with a **one-round modulation lag**; **state-
  dependent retrieval** with teach-state on the stamp; the **recency-weighted
  per-point median** as the read-side emotional reading; the **bake** as the
  write-side durable per-edge artifact; **one shared clock** for stamp aging;
  **emotional history supersedes via the existing time axis** (archaeology).
- **AMENDED:** "the boost decays" → **the bake is durable at formation and what
  fades is read-side recency weight** (**OD-27**); §16.1's door calls are
  **per-event, not per-thought** (OD-29 for the clock unit); §18.5's σ now
  matches **state against state** (`a_at` on the stamp, **OD-28**); the §18.5
  cap gains a **second, statistical guard** (§20.3).
- **UNCHANGED, and deliberately not touched:** signals → `a` → walk modulation as
  the only causal chain; named emotions as audit readouts with zero causal
  power; `audit.py`-only vocabulary; charge on L3 only; frozen layers
  structurally unable to hold affect; embeddings banned; I9's read-only idle;
  the six doors; append-only storage; `a` at rest at t=0 and every median
  undefined there. *(One clause of this bullet was overtaken by §20.4a: the chain
  is still the only **causal** chain, but `a` gained a second, non-causal outlet
  into the write side — **OD-33**, scoped to one instant and bounded by `B_max`;
  §18.2 states the distinction. The other clause holds: the litmus is unchanged
  and a fresh session still bakes nothing.)*
- **Ledger:** **OD-27 … OD-32** raised (§[decisions/open-decisions](decisions/open-decisions.md));
  OD-32 is the demo-trace question — the chunk's worked example is a *monster*
  scene, and the north-star trace is the apple scene, so the new mechanism ships
  annotated into §4 plus a G11 transcript rather than a rewritten demo.
- **New:** **G11** ([docs/13](13-gate-plan.md)), risks **R-26 … R-29**
  ([docs/14](14-risk-register.md)), and the DoD goes to **eleven gates**
  ([docs/15](15-definition-of-done.md)). *(It is **twelve** since the supporter
  batch — [docs/21](21-grimm-supporter.md)'s G12 earned its own gate because a
  component with a mouth can be denied one; a curve could not.)*

### 20.11 The SALIENCE-BAKE-SCALE amendment (same batch, folded into §20.4)

The owner's follow-up fragment changed four things and closed two questions:

- **ADDED:** `edge.bake = B_max · tanh(k · a_formation)` — a **saturating** write-
  side scale (§20.4a), version-tagged, swept as **OPEN-33** with a
  non-degeneracy objective rather than a hand-picked pair of numbers. It is the
  **third** bound in the system and the only one on the write side: §18.5's
  per-round cap (read), the median's repetition requirement (read), `B_max`
  (durable **and** bounded).
- **CONFIRMED:** the impact score is the **live signal sum at the moment**
  (novelty + contradiction + negative gain + settle failures), with **no event
  categorization anywhere** — the anti-cheat law restated, and further evidence
  for OD-31's strength-not-valence reading (a bad event *raises* `a`).
- **CLOSED by ruling:** the bake gets **no decay tail in v1** → **OD-27 resolved**
  as permanent-bake; option (b) (half-life) refused, and with it R-23's
  decaying-write-side temptation stays shut.
- **CLOSED by naming:** ignition's salience weight reads the **bake** →
  **OPEN-32 closed** (it is neither raw stamps nor median), and §19.2's
  "worry / rehearsal / intrusive thought" now has one input, bounded by `B_max`.
- **Given a job:** `a_at` (OD-28) is now both the read-side state-match evidence
  **and** the argument of the curve that sized the bake — which strengthens
  OD-28's case without settling its "every stamp or only content edges?" sub-
  question, still open.
- **Clash, ledgered not smoothed:** `f(a_formation)` on the write side is what
  **OD-22** forbade by owner decision. Recorded as **OD-33** and scoped — one
  door, one instant, one curve; `magnitude`, `budget`, `emit` and arbitration all
  keep OD-22's ruling. **This is the only genuinely unruled item the amendment
  leaves**, and its option (b) (strict-local `a_formation`, no residue) is the
  guard for the new risk below.
- **New risk R-30, said out loud** ([docs/14](14-risk-register.md)): `a` is leaky,
  so `a_formation` carries **residue** and a calmly-taught edge formed just after
  a storm inherits a trauma-sized, **permanent** head start — one event paid out
  twice, and un-revisable by OD-27's own law. `B_max` bounds the damage; G9(e)
  reports bakes against their `a_at` so the mispricing is visible as a finding.
- **Gates:** no new gate. The bound rides **G9(e)** (third guard, write-side) and
  **G10(e)** (the bake-driven over-representation leg — since **OD-91**, 2026-10-23,
  that leg is graded as the **V0/V3 state-hash identity**, and the
  "over-representation" wording above is routed to **OD-92** rather than satisfied by
  a manufactured difference); **G11's four legs are
  untouched**, which is the point — the amendment added scale, not time.

## 20.11 What the v2 arc touches here, and what it may not turn into ([docs/22](22-crystallize.md)–[docs/26](26-world.md))

Three of the arc's five mechanisms borrow §20's statistics rather than adding
machinery, and that borrowing is the part worth writing down while it is fresh.

- **The event view reuses `a_at`, and only as a read.** §20.2 made teach-state
  part of the address and §20.4a made `a_at` do double duty as `a_formation`;
  [docs/24](24-episodic.md) §24.3's "the moment is read against `a_now`, same as
  the s19 mechanism" is that same sentence with a new subject. **No new affect
  curve enters** — the tug strength is a walk-side weight (OD-114), not a second
  integrator, and a re-light produces no stamp, no bake and no confidence move
  (**G17(d)**, R-39's subject). If a future draft wants a moment to "feel
  different per voice", that is **OD-129**'s unruled question, not this section's.
- **Silt is topical, the median is emotional, and the two must never be merged.**
  Both are read-side, both recency-weighted, both read the **one** clock (§20.3's
  one-clock law, R-29) — and they are computed over different lists: the median
  over a *point's stamps*, silt over a *point's lightings*
  ([docs/23](23-context.md) §23.3's deposit rule). Merging them would be the
  cheapest way to
  smuggle warmth into affect, because the merged number would look like the
  median and behave like attention. The laws that keep them apart are already
  written: **OD-107** bans an arousal-scaled silt decay outright (banned, not
  merely unused), and **G16(e)** asserts that silt on/off leaves the crystallize
  candidate table byte-identical. The one open seam is honest about its cost:
  whether the origin-tug reads silt is **OD-114**'s second half, and tonight they
  do not touch.
- **`bake` is the arc's only durable affect field, and the arc adds no peer for
  it.** Skills may make a baked path *cheaper to compose* and silt may make a
  baked point *more likely to seed a thought*; neither may re-price it. That is
  OD-27/§20.4a's law restated over new subjects, and it is why G18's deletion leg
  and G16's arousal leg both assert lattice bytes rather than vibes.
- **`undefined at t=0` survives all five.** With every switch on and no signals,
  every median is still undefined, every bake still zero, `a` still at rest
  (§3.4's litmus, §18.10's last bullet). A mechanism that could warm, nominate or
  bind *before content exists* would be smuggled content wearing one of §20's
  three nouns.
