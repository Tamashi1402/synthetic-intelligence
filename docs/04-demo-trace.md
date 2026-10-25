# §4 — The North-Star Demo: Numbered State Trace (CONFIG SI)

Notation: `s` = step (every owner input, THINK tick, or system question).
Rows `r#`, **ask rows `q#`** (rows in the `questions` shelf, §17.3),
corrections `c#`, source tag BASE|OWNED, **`a` = the one installed arousal
scalar (§18.1), now a per-event **trajectory** inside each thought with the
walk's modulation **lagged one round** (§20.1)**, **`median(point)` = the
read-side recency-weighted emotional reading, logged and stored nowhere
(§20.3, R-28)**, and **`labels:` = the audit layer's readouts over
(signals, `a`, behavior)** (§18.2 — names that describe, never states that
drive; trajectory at the bottom of this file). Owner-tier correction
confidence init per §8.4. **All events are OWNED — CONFIG SI has no BASE.**
**DEMO TRACE v2** — the birth answer is a question, and the first lesson
needs no `/correct`: SI asked, so the owner's plain reply teaches (§1.4).
**v2.1** added per-act `e` annotations, **v2.2** the arousal curve + label
stream and the idle tail, and **v2.3 (round 3)** annotates plasticity into
this scene — per-event `a`, the bake bought at s7, `a_at`, the median line and
the past-anchored reading at s19 — **without renumbering a single step**
(OD-26's append-only rule; OD-32 is the ruling that this is annotation, not a
rewrite, and that the chunk's monster example ships as G11's transcript).
**Supporter note, added without touching a single step ([docs/21](21-grimm-supporter.md)):
this trace is G12(a)'s control run — every step below is produced with the
supporter process **not started**.** Nothing in Acts 0–6 needs him: the ask-back,
the auto-commit, the FACT citation and the ASK branch are all owner-channel
events. A second transcript with Grimm echoing and co-regulating is **not** a
second north-star trace; it is G12's own artifact (`gates/G12/`), for the same
reason OD-32 sent the monster scene to G11 — a mechanism that only appears in a
gate run is a claim nobody can see in the demo, and a demo rewritten to show it
stops testing what G1–G5 assert. Where he *would* be visible, if he spoke: s3's
ask would be followed by a calm echo line in `context` with `source: grimm`, s5's
`red.` would still be the only line that could teach, and s7's bake would still
have exactly one author.

## Act 0 — birth answer: the first OUTPUT is a question

| s | event | detail |
|---|-------|--------|
| 1 | owner: `what color is apple?` | DECODE (R14): question frame fires, emits pattern `color + apple` (relation slot + entity slot + frame). The decoder knows nothing about either word — pure shape (§2 litmus). Lexicon += tokens. Walk lights nothing (CONFIG SI connectome empty, §16.2). **`a` moves HERE, not at s3 (§20.1):** the question frame lighting is a **gap signal event**, and arousal integrates per event *inside* the walk — so `a` is already off rest before the thought ends, and the rounds after the spike conduct under the new value (**one round later**, the lag). At this step `red` has **no emotional reading at all**: zero stamps ⇒ the point median is **undefined**, not neutral (§20.3, §9's birth line). |
| 2 | settle | Walk dies at the intersection `color(apple)`: decode **succeeded**, no edge exists ⇒ the dark spot is a **NAMED missing slot** (`color(apple) = ???`), §7.1. No answer row can fire (zero rows). Detection stops here — the read path does not decide to ask. |
| 3 | ask | The named gap is handed to the **DRIVE interface** (R15); its curiosity *policy* prioritizes (only candidate), budgets (spend allowed), **emits** — *"What is the color of apple? You haven't told me that yet."* Ask row **q1** written to `questions`: `{kind: ask-back, target: slot color(apple), asked_at: 3, status: open}` (§17.2.1, §17.3). **`a`: the gap signal has been firing through this thought** — the frame at s1, then the spread dying at the slot — so arousal is **already off rest** when the ask is emitted, and this event is another lift on top of it (§18.1, §20.1: integration is per event, inside the walk, not per thought). **`labels:`** audit prints *curiosity* over the pattern (arousal up ∧ novelty ∧ an ask emitting) — a readout on the log, not a state that caused the ask (§18.2). |
| 4 | label | **I-DON'T-KNOW / NONE** on the ask line. **OD-10 RESOLVED — ask-back is primary** (CURIOSITY fragment + this trace); the §7.4 nonce survives only as the fallback for unnamed slots. Output logged as o1 (binding target). **Zero content: no guess, no nonce invented as an answer** — G1 and G8(a) both hinge on this. |

## Act 1 — the first teaching (exactly one row, no `/correct`)

| s | event | detail |
|---|-------|--------|
| 5 | owner: `red.` | **A bare statement — and it still teaches.** The write path checks for a pending question BEFORE pattern matching (§6.1): q1 is open, its target slot is `color(apple)`, and the line's shape is exactly that slot's filler ⇒ **drive match ⇒ AUTO-COMMIT** as ground-truth-by-context, provenance `kind: answer-to-ask` (§1.4, §17.2.3). Error-before is explicit: the named dark spot SI itself pointed at, not a wrong claim. |
| 6 | write gate | novelty(`red` @ q1's slot)=1.0; coverage=0 ⇒ surprise 1.0. Consistency: no prior response at that slot ⇒ clean. **`a`: the spike feeds the one integrator** (§18.1) — and at the round-3 reading this instant supplies **`a_formation`, the argument of the saturating bake curve** (§20.4a): the stamp's `magnitude` stays the signal's own value, while the edge's durable head start becomes `B_max · tanh(k · a_formation)` — **the write side reads `a` exactly here, once, through one curve (OD-33, narrowing OD-22)**. Magnitudes UNMEASURED until `arousal_sweep.json` carries `B_max` and `k` (OPEN-33). **`labels:`** *surprise*. |
| 7 | commit | **r1** shelf `s_r_pairs`: stimulus = normalized pattern of the s1 question (`what color is apple` post-fold), response = `Red` + surface, **bind_output_id=o1**, prov `(s5, kind: answer-to-ask, question_id: q1)`, written_at=5, conf **0.95** (T3-context), `gain_exact=+1 correction-unit` (§8.2). **Exactly one row.** Loop closes: **q1** → `status: resolved, resolved_by: r1` (§17.3). Physical effect: r1 creates the first real OWNED edges in the connectome — `apple —(r1)→ red` (+ frame edges from the stimulus terms); SI can now form this thought at all (§16.2 — first synapse). **Charge stamped on those edges** (§18.5, §20.2): `charge: [{signal: novelty-spike, mag UNMEASURED, a_at UNMEASURED, thought@s5, signal_ref}, …]` — **this instant is where the edge's BAKE is bought (§20.4):** the conduction head start and the confidence bump it got at formation are its **birth properties and are durable** (OD-27 replaces "the boost decays", **RESOLVED: no decay tail in v1**), sized by the **saturating** curve `B_max · tanh(k · a_formation)` (§20.4a, **OD-33**) — so this edge is hot forever *up to a bound*, and s7's spike does not buy an unbeatable head start; and `a_at` is what lets a later thought *match states* rather than just sizes. From s7 on, `red` also has a **readable** emotional value for the first time: its point median is defined (one stamp), though §20.3 warns it is a weak statistic on thin history. |
| 8 | THINK tick | gap scan: `apple`, `color`, `Red` referenced but undefined ⇒ candidate queue. **No ask emitted**, and post-round-2 the reason is *mechanical, not affective*: the per-turn ceiling and dedupe against the queue (§17.3), plus a walk that settled cleanly so no dark spot surfaced (§7.1). **`a`: decaying on its own τ** — nothing fired at s7's write except the one spike. **`labels:`** audit may print *satisfaction* (gain realized, question closed); **no label suppressed the ask** (OD-22 — this is the place where round 1's "starvation" story was removed). |

## Act 2 — recall as FACT

| s | event | detail |
|---|-------|--------|
| 9 | owner: `what color is apple?` | **The walk runs:** token pulses in word order light frame nodes, then `apple`; spread reaches `red` via the OWNED edge r1 (§7.3); settles round 2–3 (identical pulses to s1 that died — now the edge exists: same lighting, different settle). Identical stimulus = identical initial lighting = same settle (old signature rule as special case). **State-dependent conduction active (§18.5, §20.2):** `a` is still elevated from s5–s7 and r1's edge was **formed** in that window, so its stamp's `a_at` matches the current state ⇒ `σ(edge.bake, a_now)` > 0 ⇒ that edge conducts cheaper — **this row is G11(a) in miniature: taught hot, asked hot ⇒ tighter settle** (settle dominance higher, K rounds lower — all UNMEASURED). The **point median** of `red` (§20.3) contributes the other read-side term, `m(red, a_now)`: one stamp on a calm history, so the bias is barely nonzero and the word is *not* "marked as scary" by a single event. Nothing here is a token-level dark spot, so **no gap signal fires inside this walk** and `a`'s trajectory across s9's rounds is nearly flat (§20.1) — the trace shows which round ran at which `a`. The walk settled, so **no settle-failure signal fired**. |
| 10 | output | response copied byte-exact from r1: **"Red"** + activation trace shipped: `apple@r0 →(edge r1, w=0.95) red@r2` (§16.4 — trace is first-class output). **The arousal line ships with it** (§18.6, §20.1): `a@s9 = near-flat trajectory, decaying-from-s6-spike` + `median(red) defined, |bias| UNMEASURED, 1 stamp` + `labels: satisfaction` + `bias attribution: r1 edge cheapened by state match (a_at vs a_now)`. |
| 11 | label | **FACT / OWNED** — *"'Red' — you answered my question at s5 (q1, asked at s3)."* **The citation is the ask-event**, not a correction act: the question row id lives in r1's provenance (§9). Coverage: known answer re-served ⇒ no new row; `repeats += 1`, `last_confirmed_at=9`, conf +0.05. **Gain signal fires** (§8.2) ⇒ feeds `a`; the re-served edge's rehearsal curve resets (§8.4). **`labels:`** *satisfaction* — a readout over (gain ∧ question closed), with **no causal role**: the slot stopped being asked about because q1 is `resolved`, not because a label said so (§18.2). |

## Act 3 — sentence form from one correction

| s | event | detail |
|---|-------|--------|
| 12 | owner: `/correct The apple is red.` | triple again: (the s1 question, the s10 single-word serve, the new sentence). P_CMD_C ⇒ **r2**: same stimulus pattern, response surface `The apple is red.`, bind_output_id=o2. `/correct` is now the **override channel** (§1.4): it still works, it is just no longer the only way in. |
| 13 | supersession (refinement) | r1 aged: `superseded_at=13, superseded_by=r2`. Not deletion — r1 stays citable ("first answer was one word"). Event logged, branch `owner-correction`. |
| 14 | chunk learning | chunk store learns form `THE X IS Y .` from response surface; tokens reused from lexicon. Grammar = chunk reuse; next recall renders the full sentence. Still S0 S-R storage — no fact row yet. |

## Act 4 — supersession on the time axis

| s | event | detail |
|---|-------|--------|
| 15 | owner: `no wait, apples are green.` | P_CORRECT marker wrapping a P_ATTRIB statement. R2 folds `apples`→`apple`; resolves to the same field slot as r1/r2's response (entity apple, attrib). Contradiction vs active r2 (`red`). **`a`: two signals fire in the same tick** — novelty spike (the retrieved row just failed) **and** contradiction count +1 (§18.1) ⇒ arousal lifts further than a first lesson did. **`labels:`** *surprise*, then *caution*. |
| 16 | arbitration | tier: owner-statement WITH correction marker ⇒ `marker-correction` branch (§6.4) ⇒ supersede, flag, record. |
| 17 | commit | **r4** (response `green`) supersedes r2 (`superseded_at=17`). **r3**: verbatim context row for the utterance itself (S0 always keeps raw, §1.2). Event e3: `{old:r2, new:r4, branch:"marker-correction"}`. **Emotional history kept citable (§18.6):** r2's edge ages **with its calm stamps intact**; r4 forms stamped `{novelty-spike high, contradiction high}` — "taught calmly, contradicted hotly" is now a retrievable fact about the belief, not a lost detail. The stamps name **signals**, never emotions (§18.5, OD-23). |
| 18 | owner: `what color are apples?` | stimulus normalizes via R2 to r4 ⇒ **"green"**, FACT / OWNED, cites s17. **State-dependent retrieval visible in the trace (§18.5):** `a` still carries the contradiction-window elevation, and r4's stamps match that window ⇒ r4 conducts cheaper than a neutrally-stamped alternative would. |
| 19 | owner: `what did you believe before?` | history query over superseded rows — time-scoped **FACT / OWNED**: *"red from s7 to s13, then 'The apple is red.' until s17, then green."* Every token row-cited with intervals; **each interval carries its stamps** (the signal history of a belief's life, §18.6). **Round 3 reads this row twice (§20.5):** the same past anchor that re-weights the aged belief re-weights its **affect** — a median computed with recency measured from that interval, plus the stamps' `a_at` — so *"what did you believe, and how much did it matter then?"* is **one** history-anchored walk with citations and zero deleted bytes (**G3's extended leg, G11(d)**). Time framing is still taught, not innate (§14's frame law): with no edge linking "back then" to an interval, this query decodes to an unlinked frame node and SI **asks**. |

## Act 5 — bare contradiction (the ask branch)

| s | event | detail |
|---|-------|--------|
| 20 | owner: `the apple is blue.` | P_ATTRIB, **no marker, not a `/correct`**, first occurrence. Conflict vs active r4 (`green`), same tier. **`a`: two signals in the same tick** — contradiction count +1 **and** a novelty spike (the retrieved row failed again) ⇒ arousal reaches its session peak (§18.1). **`labels:`** audit prints *caution* over the pattern (contradictions rising ∧ about to arbitrate to ASK) — a description of these two rows, with **no vote** (§18.2). |
| 21 | arbitration | branch **ASK** (I7, §6.4): nothing committed; candidate held as pending event only. **Post-round-2 this is a rule, not a feeling:** same-tier conflict + no marker + first occurrence ⇒ the write check fails ⇒ ask rather than commit, by §6.4's own table. Round 1 told this as *"CAUTION's effector making `emit` cheaper than `assert`"* — **that machinery is cancelled (OD-24)**: arousal modulates the **spread**, never the arbitration table or the label ladder. What `a` legitimately did here is §18.1's only permitted job: at peak arousal the spread reached wider and decayed faster, which is why the walk surfaced the very row the ask now quotes. |
| 22 | output | drive-emitted question (endogenous to the conflict, §7.1 G4, §17.2.2): *"At s17 you said green. Now you say blue. Should I use blue? (or /correct me)"* **Arousal line cited with the ask (§18.6):** `a@s22 = window peak, fed by s20's two signals`; `labels: caution, frustration`. The labels **describe** the ask; the ask is fully explained — without an emotion word — by I7, §6.4's branch rule, and the queue/dedupe ceiling (§17.3). |
| 23 | owner: `/correct Blue.` | correction act ⇒ top tier ⇒ supersede immediately. **r5** written (response `Blue`, stamped `{signal: realized-gain, mag UNMEASURED, thought@s23, signal_ref}`), r4 aged at s23. Event e4: `{branch:"owner-corrected-conflict", asked_at:22, resolved_at:23}` — disagreement visible even though owner won outright ("owner corrections outrank everything" — but never quietly). **`a`:** the contradiction signal stops firing and a realized-gain signal lands (the open question closed, §8.2) ⇒ `a` starts **its one decay on `λ_a`** — no hysteresis, no per-resident release constant (§18.4). **`labels:`** *satisfaction*; *caution* simply stops appearing, because its pattern stopped. |
| 24 | query `what color is the apple?` | FACT / OWNED `Blue`, cites s23. Full chain citable: red(s7) → sentence-form(s13) → green(s17) → blue(s23). Each interval keeps its stamps, so the chain is citable **as emotionally conditioned history** while `a` is mid-decay and still cheapens the hot end of it (§18.5). |

## Act 6 — the idle tail (extension, pending owner nod: **OD-26**)

Round 2's second half says the quiet tail is not silence — the backlog no
longer floors the loop, **ignition does** (§19). These three steps are the
proof the demo owes that claim; they are appended, not inserted, so the
s1–s24 owner-facing trace above keeps its numbering.

| s | event | detail |
|---|-------|--------|
| 25 | quiet tick (no owner input) | senses quiet ∧ agenda empty ⇒ **ignition fires** (§19.1) — a THINK tick with no sense event, the 4th perturbation source. The **seeded** sampler (§19.4) lights one hyperpoint: `green` (r4's response term) wins on **decay deficit + recency**, the two weights that are nonzero here (§19.2). The walk spreads **read-only** (§19.4, I9). |
| 26 | monologue | the cascade rendered through the **same decode face** as owner-facing output (§19.3): *"green — superseded at s23 — blue — at s20 this slot was contested."* A truthful rendering of what actually lit, not generated prose; logged in the **monologue stream**, kept separate from the panel (OPEN-23). **`a`:** the cascade crossed its own live contradiction (r4 vs r5, one slot) ⇒ the contradiction signal fires ⇒ **arousal lifted because of what it thought about, not because a timer said so** (§19.5). `labels:` *caution* on the same pattern as s20 — the audit cannot tell the owner-driven conflict from the self-driven one, and §18.2 says it doesn't need to. |
| 27 | consequences | **Zero rows written** (I9) — the aged r4 edge had its **rehearsal curve reset** by being re-lit, and one **agenda item** was born from the cascade (`slot color(apple) still contested?`). Whether that item is ever spent is the queue rule's business (§17.3), not an urge's. Idle re-stamping of the re-lit edge = **OPEN-24**, leaning yes under §18.5's cap. |

Zero retraining anywhere. Determinism: replay of s1–s27 reproduces
identical state hash (Gate G5, §13) — **`a` is inside that hash**, quantized
on the same grid as the settle-set (§16.6, §18.4: decay in logical `t`, never
wall-clock), and so is **the ignition seed** (§19.4: randomness lives in state,
not time). A replayed session therefore has byte-identical arousal,
byte-identical cascades, and a byte-identical monologue — **unseeded ignition
breaks G5 and is a design violation.**

**The third teaching channel is deliberately not a step in this trace (TEACH
fragment, §1.4).** `/teach` joins `/correct` and `answer-to-ask` as a way content
enters the shelves, and this scene already demonstrates the other two with their
error-before visible; a `/teach` has **no** error-before, so inserting one here
would make the reference trace weaker evidence, not different evidence — every row
in s1–s27 would carry `gain_kind: demonstrated` except an isolated one that could
not show what it replaced. It is gated instead by **G1's TEACH leg and G8(f)**
(§13), which exercise it against an empty lattice and against an open ask without
renumbering anything. What the trace does owe the channel is the honest asymmetry
in §8.2: a session built by curriculum can print two hundred rows and **zero**
demonstrated gains until something actually serves from them, and the ledger says
so rather than letting a confident row look like a corrected one.

## The curve this session must print (owner requirement, §18.4)

Round 2 re-scopes this section: it is **one scalar's trajectory plus the audit
layer's label stream over the same rows** — not six residents' curves, which
are cancelled machinery (OD-21). It is still the readable proof that this is
tissue rather than a rule engine, because every step of `a` names the signal
row that produced it. Qualitative shape; **every magnitude UNMEASURED** until
`gates/G9/arousal_sweep.json` (OPEN-16, reduced to `λ_a` + gains + the
modulation curve 16a + the lag 29) **and `gates/G11/median_sweep.json`**
(OPEN-30/31) exist, and the label stream is checked against the signals by
the REPLACED G9 (§13):

Round 3 restakes what a *row* of this curve means. `a` is no longer one value
per thought step: it moves **per signal event inside the walk** (§20.1), so the
cell under s3 is a short ascending trajectory, not a dot — the gap signal fires
on each token that lands on a dark spot, and the sentence colors itself while it
is being read. What the curve column *owns* is still the thought's round
attribution, and that is where the owner's **one-round modulation lag** shows:
the events inside s17 raise `a`, and the spread they widen appears in **s18**,
not in s17. Reading the curve as "round N is emotional because of round N's
signals" is the emotion-mush misreading R-27 guards.

The second line is not a second scalar. `median(point)` beside each row is the
audit's readout of §20.3 — a recency-weighted statistic recomputed at query,
**logged and never stored**, undefined for every point until a point's history
has its first qualifying stamp (s7 is where `red` acquires one). Its only
visible job in this trace is to *resist*: a single hot stamp at s7 leaves the
median essentially where it was, and only repetition (s9–s24, several stamped
events on the same point) moves it measurably.

```
s        │ s1–s2   s3        s5–s7        s9–s11       s15–s17            s20–s22            s23–s24        s25–s27
─────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────────
a        │ rest ─▶ ▲         ─▶ ⬤ spike    ─▶ ⬐ decay ─▶ ▲▲                ─▶ ⬤ session peak   ─▶ ⬐ one τ    ─▶ ▲ small
         │ (per-event trajectory inside each cell; the *effect* on the walk
         │  is the next round's — §20.1's lag)
median   │  all undefined ─▶ `red` 1 stamp, ≈flat ─▶ drifting only as stamps pile up ─▶ settled, UNMEASURED
labels   │ none ─▶ curiosity ─▶ surprise   ─▶ satisf.  ─▶ surprise, caution ─▶ caution, frustr. ─▶ satisf.   ─▶ caution
signals  │ none ─▶ novelty    ─▶ novelty=1 ─▶ gain     ─▶ novelty+contrad.  ─▶ contradiction(2)  ─▶ gain      ─▶ contradiction
         │  (named gap) (stamps+bake r1's edges) (q1 closed)  (contradiction #1)   (contradiction #2 → ASK) (owner wins) (own thought)
```

- **One curve, one scalar.** The test is not "does it look emotional" but
  "**does every move cite a signal row**" — a jump in `a` with no row behind it
  is the costume test failing (§18.4), same law as an uncited FACT. The median
  line is held to the same law: it is a statistic over cited stamp rows, so a
  median that moved without stamps moving is the same failure wearing a
  different dress (R-26).
- **The labels are a stream, not a cause.** *curiosity* at s3 is audit's name
  for (arousal up ∧ novelty active ∧ an ask emitted). Nothing in the loop
  consults the word; the ask is emitted by §17's drive policy under the queue
  rule (§18.2).
- **Round 1's "self-extinguishing curiosity" story is gone** (OD-22). The
  interrogation phase ends because the *signals dry up* — fewer named dark spots
  surface as rows get filled — and because of the mechanical per-turn ceiling
  and dedupe. No urge starved, no level gated the budget.
- **The readouts nobody wrote still appear:** *frustration* at s20–s22 is the
  audit's label for (high `a` ∧ repeated failure on **one slot**); §18.3 keeps
  this as the load-bearing claim in both directions — if human-recognizable
  patterns do **not** fall out of these dynamics, **the architecture is wrong**,
  and that gets published as a finding rather than patched with a gain matrix.
- **The idle tail is thought, not quiet:** after s24 `a` decays toward rest,
  but rest is no longer the floor of the loop — ignition takes over (§19), so
  the tail prints monologue entries and a small arousal bump born from content.
  **Silence means slow thought, never spinning** (§19.4).
- **Round 3 leaves one mark in the tail, and it is a law, not a decoration:** the
  s25–s27 bump moves `a` and the point medians, but **bakes nothing**. Idle
  thought forms no edge, and a bake is bought only at formation by a teaching
  event (§20.4) — so the monologue can change what *red* currently feels like
  while it is being read and can never change any edge's permanent head start.
  G10(b) is what enforces that here; the curve alone would not show the
  difference.
- **The two clocks in this trace are one clock:** the median's recency weights
  and the s23–s24 fact-decay deficit read the same curve (`clock.py`, §12), so
  *frustration* aging and *red*'s median aging cannot be tuned apart by
  accident (R-29, G11(c)).

## Act 7 — the v2 arc's fixture: s28–s35, a scenario and not a renumbering (**OD-132**)

**The arc gets its own steps; it does not borrow this trace's.** The five
mechanisms — crystallize, silt, episodic binding, skills, N-voice world — ship
their ruling rows **`off`** ([docs/03 §3.3](03-birth-state.md)), so with the arc
merged and nobody switching anything on, **s1–s27 replay byte-identical.** That
byte-identity is the arc's first obligation, not a nice side effect: it is
G15(b)/G16(b)/G17(b)/G18(b) — delete the machinery, rerun, no lattice byte moves
([docs/13 §13.13](13-gate-plan.md)). Inserting a v2 event into s1–s27 would
therefore be the cheapest way to make the arc's own safety leg unrunnable, and
**OD-132 stays unruled on purpose.**

What Act 7 is instead: the **same scene, with more than one voice in the room
and every switch installed by the gate scenario itself**, run as a harness
scenario over a scripted transcript (`gates/G15…G19/`), with steps numbered in
this file's continuing namespace so ordering means what it usually means. No
line below is a live-server walkthrough: the Discord transport is out of scope
tonight and said so ([docs/26 §26.13](26-world.md)).

| s | event | detail — and the leg that owns it |
|---|-------|--------|
| 28 | a second voice arrives | a stranger line, identity tagged, class **derived** at generation and stored nowhere (**tier (a)** ⇒ non-owner). It lights the walk exactly as an owner line does and **moves no belief row**: ten users talking with the owner silent is a **zero-belief-write** run (**G19(a)**). Its `context` sense row still appends — the sense record is not a belief (§21.4, and the exception is docs/03 §3.3's). Labels apply unchanged — the ladder is voice-blind (§9). |
| 29 | silt accrues | the conversation's silt table takes deposits from **both** voices at the same weight (OD-106 full-weight parity — warmth tracks thinking, not teaching) and decays on the one clock, **unscaled by `a`** (OD-107 is banned, not merely unused). The table stays outside the lattice and outside the state hash; it is logged to `silt.jsonl` (OD-103). **G16(a)/(c)**. |
| 30 | quiet tick, silt warm | ignition now has a **fifth** candidate weight — silt warmth — which is **SPECIES and ships unarmed at 0**, so the four temperament draws stay four (B7: no sweep touches a temperament row). A warm topic may drag its **moment** in through a `prov-link`: the origin tug is an ordinary read-side weight at registry-low value, capped per frame by `RELIGHT_MAX`. **G16(d)**, **G17(c)**. |
| 31 | a recurrence surfaces | three lived rows share fillers often enough to be a **candidate** — a volatile, hash-exempt fact about session state, logged to `crystallize.jsonl`, **never a row**. Ghost composites exist only inside their frames (§16.8). **G15(a)/(b)**. |
| 32 | the nomination ask | the drive emits an ordinary ask on the **owner lane** (OD-128): *"I keep seeing … is there a word for this?"* with `pattern=FACT/OWNED` citations and **`name=NONE`** (§22.4, §9). Same queue rule, same integer ceiling — a busier world buys no extra asks (**G15(e)**, **G19(f)**). |
| 33 | the owner mints | the owner's reply is the only event that can turn the nomination into a point: an ordinary gated write, new L3 cluster point, edges, provenance `prov_kind: answer-to-ask` with `question_id` = the nomination ask row and `prov_speaker: owner` — so an ordinary answer-to-ask commit, `gain_kind: demonstrated`, **no new provenance kind** (§22.4). **`source: grimm` — and any ungranted stranger — can never confirm** (**G15(d)**, **G19(d)**). The candidate closes `resolved`. |
| 34 | the move is kept | the same shape over *solved* frames yields a `move-nomination`; an owner name mints a **skill row** on the `skills` shelf, id 9 — the reserved shelf §5.2 opens with this kind (§25.11) — through the same ordinary `answer-to-ask` commit, so no new provenance kind appears anywhere in Act 7 (§25.5). The row is a read-side composition preference and **nothing executes it** — a skill that became code is R-40 (**G18(a)/(b)/(e)**). |
| 35 | recall with its origin | a query settles through the belief row and its `prov-link`, and prints **both** citations — *"red, taught by you at that moment"* — while the re-light remains a read of stamps. RENDER says no sentence about how that felt: **a re-light is not a flashback** (**G17(f)**). The idle tail's rule extends: this traversal **bakes nothing** (§20.4), because no edge was formed. |

**What Act 7 does not add.** No organ (§2 stays six), no drive door (six), no
thirteen crates' worth of new storage shapes (§5's "zero new structures"), no
19th temperament draw, no new hash item, and no envelope field — identity rides
the sense stream, which is why the 80-byte record is untouched. Two grant rows
appear only inside a scenario that **installs** one to exercise revocation
(**G19(c)**); birth holds zero, and a build that minted a grant to make a gate
interesting would be the laundering §26.3(c) rejects. **Every magnitude in this
section is UNMEASURED** until `gates/G15/recurrence_sweep.json`,
`gates/G16/silt_sweep.json` and their siblings exist — and those files publish
**refusals** while OD-83's enumeration stays unruled, which is the honest shape,
not a missing artifact.
