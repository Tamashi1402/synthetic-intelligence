# §9 — Claim Labels (organ: CALIBRATE, output contract)

**Labels are measured from the walk** ([docs/16](16-think-walk-hardware.md)
§16.4), not from a separate classifier: settled via OWNED edges = FACT;
genome paths only = INFERENCE (BASE); died out = I-DON'T-KNOW. **The
activation trace (what lit, which round, via which edge) ships with every
output as the audit log — first-class, never debug noise.**

Source tag: **OWNED** (walked via L3 edges), **BASE** (walked via L1/L2
genome edges only, CONFIG ENGINE only), **NONE** (guess slot / nothing
lit). Tags are orthogonal to the label and **always** in the output line —
every answer says which side it came from. When OWNED and BASE both fire
and disagree, the output shows the OWNED answer as the claim and appends
the BASE disagreement with both traces (never hidden — critical law,
§5.3). **BASE-derived answers cap at INFERENCE** (OD-04 resolved by the
THINK fragment: genome paths are WordNet talking, never SI's FACT).

**Addressable gaps surface as QUESTIONS, not the words "I don't know"**
(CURIOSITY fragment): when the dark spot has a name, the I-DON'T-KNOW label
is rendered by the curiosity drive as an **ask-back** ("what is the color of
apple?") — the honest state *and* the address in one utterance (§17.2.1).
The literal "I don't know yet" appears only for gaps with no addressable
slot. Either way the label is I-DON'T-KNOW/NONE and **zero rows are
written** (G1/G8).

| label | runtime rule (evaluated on the settled thought, first match wins) | chat-panel surface |
|-------|--------------------------------|--------------------|
| **FACT** | thought settled via ≥ 1 OWNED edge; every content token ∈ one cited OWNED row; the row's time-status matches the question's tense (active now, or interval-stated for history) | `Red — you answered my question at step 5 (q1, asked at 3). [FACT/OWNED] trace: apple@r0 →(edge r1, +w) red@r2` |
| **INFERENCE** | (a) settled via OWNED edges but composed across ≥ 2 rows/bridges, all cited, no novel tokens; or (b) **settled via genome (BASE) paths only** — capped label, ENGINE config | (a) `INFERENCE/OWNED — from r4@s17 + b2@s20`; (b) `INFERENCE/BASE — the dictionary web says X; I was never told this myself. [trace]` |
| **SPECULATION** | *(fallback only — see §7.4/OD-10)* walk died in an **unnamed** dark spot and the input is a question-shaped input requiring an answer slot ⇒ nonce guess; also: **max-K hard stop without settle ⇒ UNCERTAINTY-flavored SPECULATION** ("I couldn't settle on an answer") | `I have no idea yet. Guessing: "blork mern". [SPECULATION/NONE]` |
| **I-DON'T-KNOW** | walk died below threshold and no answer row fired. **Named dark spot ⇒ targeted ask-back** (decode succeeded, slot named from the owner's own tokens — §7.1/§7.4): the birth answer by default. Also the default when the labeler is uncertain between tiers. The dark spot feeds WHAT (§7.1) | `What is the color of apple? You haven't told me that yet. [I-DON'T-KNOW/NONE] named-dark: color(apple)` |

Properties:
- The labeler runs *after* the walk settles and can only **demote**
  (trace-based, not confidence-based), never promote. **Round 2 makes this
  stronger than round 1 did.** Round 1 justified it as "the demote side is
  exactly where CAUTION's effector lives, so an emotion can talk SI down but
  never up" (§18.2) — that reading is **cancelled (OD-24)**: post-demotion
  **no affect quantity of any kind has a path into the labeler.** The label is
  a pure function of the activation trace and the cited rows; arousal reached
  this output only by having changed *which edges were cheap* on the way
  (§18.1), which is visible in the trace and therefore already audited. The
  demote-only law stays as the structural backstop: even a future affect
  channel cannot manufacture confidence, because the promote direction has no
  door.
- At birth, SPECULATION/I-DON'T-KNOW dominate the output stream — that
  majority *is* the correct calibration (empty connectome, §16.2). The
  honesty is the product. **`a` is at rest at t=0 for the same reason (§3.3):**
  with no signals there is nothing to integrate, and a non-zero arousal — or
  any emotion label, or any edge stamp — in a fresh session is smuggled
  content (litmus, §18; R-21). **Round 3 adds the second half of that sentence:
  with no stamps, every point median is *undefined* and no edge carries a bake
  (§20.3), so a fresh session has no emotional reading of any word to be biased
  by — not a neutral one, none.**
- **Every label ships with its arousal line and the audit's label stream
  (§18.6):** the same audit mechanism as the activation trace — *"FACT, cited
  r5@s23, edge r5 baked {novelty-spike @a_formation=0.7 → `B_max·tanh(k·0.7)`},
  point `red` median 0.31 over 4
  stamps; demoted to INFERENCE because the settle used one bridge; a: 0.22→0.41
  across this thought, labels: caution."* The **reason for the label** is the
  trace; the arousal trajectory, the median and the labels are the *description*
  shipped alongside it. An output whose arousal attribution is
  missing is an invalid output (I6 covers labels; I8 covers affect). **Round 3's
  change to this line is only its contents** — the attribution is now a
  trajectory plus two derived affect numbers (§20.1, §20.3); who may read them
  is unchanged: nobody in the control path. The citation prints the bake as
  **both** of its halves — the `a_formation` it was sized from and the curve that
  sized it — because `B_max·tanh(k··)` is what makes "nothing ever scores an 11
  that owns the walk" checkable from the log alone. *(the numbers above are shape only —
  every magnitude UNMEASURED until `gates/G9/arousal_sweep.json` and
  `gates/G11/median_sweep.json` exist).*
- FACT citations include the step of learning **and the edge id** used by
  the spread: auditability is part of the label contract, not decoration.
  For auto-committed rows the citation is the **ask-event** ("you answered
  my question at step N") — the question row id is part of provenance (§17.2.3).
  **Signal history is citable too (§18.6):** a superseded belief can be
  reported with the signals its edges were stamped under — "formed on a calm
  signal, contradicted on a spike" is a fact about the row's charge stamps, and
  the reason the stamps outlive every boost: **the stamp is permanent and the
  bake it gave the edge at formation is durable AND bounded (§20.4, §20.4a)**;
  what ages is only the
  stamp's **read-side weight in the point's median** (§20.3). Round 2's phrasing
  "keep stamps after their boost decays" is superseded by **OD-27**.
- I-DON'T-KNOW is a complete, first-class answer; no shame-prefixed hedging.
  (Naming note, so the metaphor doesn't leak into the machinery: emotion words
  appear **only in `audit.py`** (§18.6) — never in the labeler and never in the
  surface text; the label ladder is a trace function, and arousal adds no
  hedging language because arousal has no line into output composition at all.)
- **The supporter batch changes nothing here — and that is the check (§21, G12).**
  Grimm's lines are **inputs**, not outputs: they are never labeled, never
  cited, never rendered into SI's response, and the lab notebook is a Grimm-side
  artifact outside SI's storage that asserts nothing of SI's (I6 does not reach
  him; §19.3's monologue label law is the shape he follows). **A tag cannot
  launder a label:** a FACT still requires an OWNED edge, and OWNED edges are
  formed only by owner teaching events (§20.4, §21.4) — so *"SI said it after
  Grimm said it"* is not a provenance chain. Worth stating the failure this
  rules out in the labeler's own language: if a grimm line could settle a walk
  **via an edge it created**, the output would print FACT/OWNED over content
  nobody taught, and every gate downstream would read as a pass.

**The v2 arc adds three label facts and removes none.** The ladder, the
demote-only law and the trace-basis are untouched — these three are about what
may be *cited*, not about what may be *claimed*:

- **An origin-cited recall keeps FACT/OWNED and gains a second citation.**
  When the walk lands on an event view through a derived `prov-link`
  ([docs/24 §24.3](24-episodic.md)), the settled answer prints both rows — the
  belief row that owns the content and the event row that says *when it was
  taught* — under the existing "you answered my question at step N" contract:
  *"red — taught by you at that moment. [FACT/OWNED] cited r5@s23 + ev@s23."*
  What render may **never** emit is an experience claim ("I remember how that
  felt"): a re-light is a read of stamps, not a flashback, and the labeler has
  no state that would license the sentence (**G17(f)**). A label that would be
  true of a memory is not therefore true of a traversal.
- **A nomination ask is two half-sentences with different labels.** The
  recurring *pattern* is citable, so it carries **FACT/OWNED** with every
  member row and every recurrence frame listed; the *name* is not, because no
  such point exists yet, so its slot stays **NONE**
  ([docs/22 §22.4](22-crystallize.md), [docs/25 §25.5](25-skills.md)). This is
  I7 one level up, and it is the check that keeps crystallize honest: a
  nomination that arrived with a name already in it has either smuggled a
  vocabulary entry or answered its own question. The candidate table is not a
  belief table — nothing in it is labelable, because nothing in it is a row.
- **The ladder is voice-blind.** The four class tags
  ([docs/00 I3A](00-thesis-invariants.md), [docs/26](26-world.md)) decide who
  may *confirm a write*, never what a claim may *claim about* them; a stranger
  gets the same FACT/INFERENCE/SPECULATION discipline as the owner (I6). What a
  user-facing readout may say about *another* user's line is **OD-129** and
  stays unruled — the build writes no such readout until it is ruled.
