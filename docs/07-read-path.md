# §7 — The Read Path (organs: DECODE + WHAT)

## 7.0 The pipeline: decode → light → walk → settle → reverse-decode

```
surface text → DECODE (R14 frames → hyperpoint pattern: subject+relation+frame)
           → LIGHT initial lighting from the decoded pattern (word order kept)
           → WALK spreading activation to settle (§16), **live**: at every
             signal event inside the thought (a token landing on a dark spot, a
             frame lighting, a hot-baked edge conducting) the walk calls
             drive.update(signals, t) → one quantized scalar `a`, no named
             emotions in that call (§18.1, I8, §20.1). From the NEXT round the
             new `a` is in effect — the modulation lags one round (OPEN-29), so
             the frame finishes its job before the sentence recolors itself.
             Conduction each round = `W_eff = W ⊙ (1 + β·σ(edge.bake, a_now)
             + γ·m(point, a_now))` as returned by drive.conduction() (§18.5,
             §20.2–§20.4) — the per-edge bake plus the per-point recency-weighted
             median, bounded and normalized per round (§18.5 cap)
           → SETTLE top-k thought + activation trace + the `a` trajectory + label
             stream (§18.6 — the labels are the audit layer's readout, printed
             after the settle, never consulted by it)
           → REVERSE-DECODE (RENDER) the thought back to surface words
           → LABEL + cite (§9) — render may only use cited row surfaces (§7.5)
           → TONGUE ([docs/27](27-tongue.md), gated by the `TONGUE` ruling row,
             **off at V0**) re-walks the surface token by token over the term
             lattice, leaning on the five style draws, and runs three structural
             checks before the line goes out. Off, or refused, the label's own
             copy is the output byte for byte.
```

**The old "one arousal update per thought" step is gone.** Round 3 moved
integration *inside* the walk; detection-then-ask still happens at the settle,
because naming a dark spot needs the finished trace (§7.1, §20.1).

**The same pipeline runs with no surface text** (§19): on a quiet tick with an
empty agenda, DECODE is skipped, ignition lights **one** hyperpoint from the
seeded four-weight sampler, and the trace that comes out the render face is the
**monologue** — read-only, no write-path call (I9). There is no second thinking
engine; that is the whole point of §19.1's precedence rule (queue revisit →
RE-PARSE → ignition).

The walk **never sees raw spelling**; it consumes decoded patterns. Only
shapes light the right points. At birth (CONFIG SI) the decoder matches
only born-in frames (R3/R14) — exactly enough for the apple demo; every
other phrasing decodes to a NEW, unlinked frame node, which is where
teaching comes in (§7.3, mechanism 3).

**What the v2 arc adds to this pipeline — read-side, frame-scoped, deletable.**
Four things attach to the walk above, and none of them changes a step's output:

1. **Silt deposits** ([docs/23](23-context.md)): each light in a round warms
   that point's session entry, and the entry erodes on the logical tick, so a
   topic cools backwards in last-touch order with no clearing instruction. The
   warmth is read by **ignition only** (OD-104's declared default) and by
   nothing else; the lattice never hears of it.
2. **Shape reading at settle** ([docs/22](22-crystallize.md) §22.2,
   [docs/25](25-skills.md) §25.3): before `h` is destroyed, the reader-side
   detectors may look at the frame's ghost cluster and its role-graph (§16.8)
   and count a recurrence in *session* state. The frame's scratch is never
   evidence, never cited, and never a row.
3. **Origin traversal** ([docs/24](24-episodic.md) §24.3): a `prov-link` is a
   registered weak edge kind, **derived at generation rather than stored**, so
   a walk may land on the event view a belief came from and re-light its line,
   its stamps and its `a_at` into the frame. Recall then renders with *two*
   citations; what it may never render is an experience claim (G17(f)).
4. **Composition preference** ([docs/25](25-skills.md) §25.6): a frame MAY
   consult the owner-ratified `skill` rows when choosing traversal order. A
   habit changes which shape gets walked first; it writes nothing, moves no
   constant, and executes no code.

Identity tagging ([docs/26](26-world.md)) happens in the **front**, before
DECODE, and the only thing it changes downstream is the class function the write
gate consults — the read path itself is voice-blind, which is what keeps a
stranger's line and the owner's line lighting the *same* points.

## 7.1 The knowledge-gap detector — concrete v1 definition

The gap is now **physically visible**: a knowledge gap is where the walk's
activation dies below threshold — **dark spots in the spread ARE the ask
signal** (§16.4). The detector reads the activation trace of the last
settled (or max-K-stopped) thought; G1–G4 classify *why* a spot is dark:

- **G1** a term referenced by an active row has `defined_by == null`
  (r1 mentions `Red`; SI has no row about `Red`).
- **G2** a parsed question lights the entity node but the field/frame
  neighborhood stays dark while ≥ 2 candidate edges exist (ambiguous field
  ⇒ needs a taught link).
- **G3** a term has been pulsed in ≥ 2 owner utterances and exists in no
  row — pulsed, never spread from: the strongest curiosity signal.
- **G4** a conflict is pending resolution (the §6.4 ASK branch).

This is the "I need X" proxy: a gap is a property of SI's own state (a
dark region in *its* connectome), not a reflex echo of the input.

**A dark spot is read twice, and the two roles stay separate (§20.1).** *Live*,
the instant spread dies at a token, it is an **arousal signal** — that is what
makes arousal a per-token quantity. *At the settle*, it is a **nameable slot**
the ask engine can emit, because naming needs the finished trace. Detection is
still not decision (I7), and asking still waits for the thought to end; only the
feeling moved earlier.

**Named dark spots (DECODE consequence, §16 fragment):** because decode and
walk are separate stages, a gap can be *precise*: decode succeeds (valid
pattern), walk dies at the intersection (no edge exists) ⇒ the dark spot is
a **NAMED missing slot** — e.g. `color(apple) = ???`. The ask engine can
then emit a **targeted ask-back** ("What is the color of apple?") instead
of nonsense or a vague question. Naming a slot is not knowledge: the slot
name is built from the owner's own decoded tokens; SI asserts nothing about
it. (This is the mechanism behind birth-state ask-backs — §7.4, OD-10.)

## 7.2 Endogenous query formation — delegated to the DRIVE layer (§17)

The read path **detects** gaps; it does **not** decide to ask. Asking is the
curiosity drive's job, via the drive interface
([docs/17](17-drive-layer-curiosity.md)) — this separation is what keeps the
drive from being welded into the read path (§17.6 swap test).

On a THINK tick (or a settle that leaves dark spots), the read path hands
the gap set to `drive.prioritize(gaps)`; the curiosity drive ranks by
salience, checks `drive.budget` (per-input cap, dedupe against the
**open-question queue**, typo/filler floor), and `drive.emit` produces at
most the budgeted number of questions per owner turn:

- **ASK-BACK (missing slot):** decode succeeded, walk died at the
  intersection ⇒ emit the **named** slot as a question ("what is the color
  of apple?"), not "I don't know" — addressable gap ⇒ address it (§17.2.1).
- **UNKNOWN-WORD ASK (missing point):** sensed token with no hyperpoint ⇒
  "what is X?" (§17.2.2); `what is znark?` fires identically (anti-cheat).
- **G2/G4 conflicts:** the §6.4 ASK-branch and ambiguous-field questions are
  also routed through the drive, so budget and dedupe apply uniformly.

If the drive's budget yields nothing above threshold, SI stays **silent** —
silence is the drive choosing not to spend, not the absence of a gap. The
owner's reply to a pending emit auto-commits as `answer-to-ask` (§1.4,
§17.2.3) — **and only the owner's: the match is gated on `source: owner`, so a
supporter line addressed at an open ask leaves it open (§21.4, R-32)**;
unanswered asks **persist** in the open-question queue and are
revisited on idle ticks (§17.3).

**Whose question is it?** A `source: grimm` line like `apple red or green?`
arrives as input and decodes like any other, and any gap it surfaces becomes
**SI's** ask only after SI's own drive decides to spend one (§17). He can point
at a hole; he cannot fill it and he cannot make SI answer it. Provenance stays
one-headed, which is the entire point of tagging the voice (§21.4).

## 7.3 Retrieval = the walk settling — no embeddings, honestly

Retrieval is not a lookup pipeline anymore: **the read path IS the walk**
([docs/16](16-think-walk-hardware.md)). Input pulses per token in word
order ("what is" lights the frame before "apple" arrives); the thought is
what settles; the **activation trace (nodes lit, round, edge used) is the
route report** — first-class output, audit log, never debug noise.

**State-dependent conduction bias (§18.5 + §20, read-side).** Before each round
the walk asks the drive layer for a conduction bias, and it gets **two numbers
and no adjectives**:
`W_eff = W ⊙ (1 + β · σ(edge.bake, a_now) + γ · m(point, a_now))`, where `σ`
matches **current arousal against the state recorded on the edge at teach time**
(`a_at`, §20.2, OD-28) — *taught scared, asked scared conducts cheaper than
taught scared, asked calm* — and `m` is the point's **recency-weighted median
over its whole stamped history** (§20.3), recomputed every thought, aged on the
**same recency curve facts use**, never stored anywhere. Edges stamped while the
loop was this hot conduct cheaper, so
**the state of the moment decides which connections are cheap — not what the
goal is.** The goal, the query, and the honest labels are untouched; only the
route cost changes, and this is one of the walk influences §18.1 licenses (the
others are reach, decay and settle dominance). It is deliberately an influence on
retrieval that is **not** a generalization mechanism: it cannot make an untaught
link exist, it can only make a taught one arrive sooner, it is normalized per
round so it cannot compound (§18.5 cap), **and the median cannot be hijacked by
one hot stamp** (§20.3, G11(b)) — the statistical guard is what stops the
monotone reading of *"more state context ⇒ closer settle."* Its trace attribution
ships with the activation trace ("this edge was cheap because the current state
matched its stamp at t=…"), because an uncitable bias is an unexplained behavior
change (I8). **No emotion name appears anywhere in this step** — `a`, `a_at` and
the stamps are numbers and signal kinds; the words are what `audit.py` prints
afterwards (§18.6).

**Anchor: NOW by default, `t'` on request (§20.5).** The same read path answers a
history-anchored query — the NOW filter becomes an *anchor* filter, aged edges are
re-weighted at their interval exactly as G3 already does for facts, and the median
is computed with its recency weights measured from `t'`. That is
**emotional archaeology**: *"what did monster feel like back then"* recovers the
old reading with citations, no deleted history, no edited stamp. It needs a
**taught** time frame; with none, the query decodes to an unlinked frame node and
SI asks instead of time-travelling (§7.1, R-14).

Generalization comes from exactly three learned mechanisms plus one innate
fallback, all expressed as edges in the web (§16.2: CONFIG SI walks L3
only; ENGINE adds the genome web under the same rules):

1. **Shared-term pulsed lighting (S1+).** A token present in the query
   lights its L3 node; spread then reaches that node's OWNED edges. This
   is term *identity*, honestly labeled — it generalizes over frame
   rewording, not over renaming (§1.3, R-02). (The old S0 pattern-
   signature equality is now a special case: identical stimulus =
   identical initial lighting = same settle.)
2. **Taught links — word edges AND template edges (S2+).** `links` shelf:
   `P_DEF` teachings (`X means Y`, taught "same question" pairs) at strong
   weight; bridges created by unique-slot resolution at weak weight,
   `speaker=system-inferred`, superseded by any owner teaching of the same
   term. **Decoding is learned, not installed:** taught synonyms and
   paraphrase templates (`colour`, `what shade is X`) become overlay edges
   **into the same frame** (kind `frame-synonym`), created by `/correct`
   events — a reworded question that decoded to a fresh frame node, when
   corrected, wires that node to the canonical frame. **Paraphrase recall
   over renamed entities or reworded templates runs on these edges, never
   on innate synonymy and never on embeddings (banned — §16.3, R-14).**
   The two barriers of a true paraphrase: **decode barrier** (rewording
   maps to the SAME pattern via taught template edges, zero token overlap)
   and **walk barrier** (the graph path finds the edge) — G2 verifies both
   with the trace showing the route.
3. **Edge strengthening (in-session).** Each successful serve or
   repetition raises edge weight (§8); the walk's path preference is
   literally SI's habit — nothing about it is innate.
4. **Co-occur adjacency.** Weak edges feeding SPECULATION suggestions and
   bridge *candidates* only; a candidate becomes a taught link upon owner
   confirmation.

Scoring/time: activation carries the NOW filter structurally — superseded
edges have zero weight in the live web (aged in the graph, alive only in
the history view, reachable via explicit history queries that re-weight
the aged edge at its interval). A stale row cannot silently answer a
present-tense question because the spread never reaches it. Hub damping
keeps the frontier ~10² (§16.1); max-K stop without settle ⇒ UNCERTAINTY
readout (§9). In CONFIG ENGINE: OWNED edges outrank genome edges at NOW by
tier weight (§5.3 law), and both lit paths appear in the trace citation.
Ties by `(t, row_id)`.

## 7.4 Birth answers: targeted ask-back (primary), nonce guess (fallback)

v1 output policy: **a parsed question demands an answer** — the correction
protocol needs its error-before (§1.1). When the walk dies with zero rows
reached, two branches:

- **Named dark spot (PRIMARY, via curiosity drive §17.2.1 — OD-10 RESOLVED
  ask-back-primary):** decode succeeded — the question has a NAMED missing
  slot (`color(apple) = ???`). SI's answer is the **targeted ask-back**,
  emitted by the curiosity drive (not a read-path reflex): *"What is the
  color of apple? — you haven't told me that yet."* Label **I-DON'T-KNOW /
  NONE**, slot named from the owner's own tokens. The error-before is
  explicit and zero-content, and the owner's reply then **auto-commits as
  answer-to-ask** (§1.4) — so the very first lesson needs no `/correct`.
  "At the edge of its knowledge it asks instead of bluffing" (§17.7).
- **Unnamed dark / decode partial (FALLBACK):** a question-shaped input
  whose slot cannot be named (broken frame, wholly unknown terms in every
  slot) ⇒ the nonce guess: syllable shapes from orthographic stats of
  tokens seen this session (letter-class n-grams only; "blork" is not in
  any table), PRNG seeded by step ⇒ deterministic on replay. Label
  **SPECULATION / NONE**, surfaced with the admission ("I have no idea
  yet. Guessing…").
- Non-question input with no grounding ⇒ plain **I-DON'T-KNOW**.

Both branches write **zero rows** (G1 asserts). This is the single place SI
speaks without evidence; the label and the absence of any written row keep
it honest. The correction triple binds cleanly to either branch via
`bind_output_id` — the named-slot version is why the row schema recorded
the binding target from day one (§1.1).

## 7.5 FACT answers copy their row

Render v1 = template copier: the answer's content tokens must be a
byte-subset of the cited row's `response.surface` / `value.surface` (+
learned chunks for composed sentences). v2 LM slot lives behind the same
interface: it may reorder/inflect, but any content token not present in
cited rows **auto-demotes the label** to INFERENCE/SPECULATION — labels are
computed from constraint satisfaction, never from LM confidence. The LM is
form only, never source of truth.

**The tongue is under this law, not beside it** ([docs/27](27-tongue.md),
OD-181). §27.2's gate checks that every word names a point SI has a row for;
that alone is *looser* than this section, because SI has rows for far more
points than any one answer cites — and a line that wanders into an uncited row
is a claim the citation does not support, which is R-43's fluent lie wearing
good grammar. So the licensed vocabulary of one realized line is the
**surfaces of the points the citation names** (each cited row's key and value
point, its canonical surface and its learned aliases — the same surfaces §7.0
already lets render use), plus the composed chunks §7.5 has always allowed. The
phrase walk may *traverse* anything cheap; it may *utter* only what the
citation licenses. A candidate outside that set is a step, not a word; if the
line cannot be finished inside the set, the gate refuses and the canonical
frame prints (**G20(h)**, **OD-181**).

Consequence, stated rather than hidden: SI can only sound fluent in the
connectives and inflections it was actually taught, because those are rows too.
The tongue varies *which owned surface* gets used and *in what order*, never
what words exist. That is §7.5's sentence — "form only, never source of truth"
— applied to a component that has no model.
