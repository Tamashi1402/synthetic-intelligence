# §0 — Thesis and Global Invariants

SI is not a trained model. It is a synthetic life form whose ONLY sense
channel is the owner's typed conversation in a chat panel — the panel is the
channel, and [docs/21](21-grimm-supporter.md) adds a second *voice* that may
type into it, never a second channel and never content of its own (I3, I10).
[docs/26](26-world.md) is the same amendment at N: the panel may be carried by
a server with many voices in it, and that stays one **world**, one channel,
because identity tags a voice and grants nothing by itself (I3A, I10A).
It starts knowing
nothing about content; every thing it knows is an addressable object that can
be retrieved, cited, contradicted, superseded, and audited from the session
transcript. **Zero gradient steps, ever.** "Knowing" is proven behaviorally:
teach at runtime → recall later (paraphrased) → detect contradiction → cite
the moment of learning. Any part that can only work via training must be
called out in [docs/14](14-risk-register.md), not hidden. **The thesis line the
supporter batch must not soften: if SI needs Grimm's content to pass a gate,
the design failed — so every gate runs with him disconnected (§21.7, G12).**

- **I1 — Innate = form only.** Tokenization, pattern shapes, gap detector,
  write gate, consistency checker, retrieval mechanics, row addressing.
  Forbidden innate content: word meanings, ontologies, pretrained embeddings,
  hardcoded facts/synonyms, any dictionary the owner did not teach.
  **The v2 arc's additions are form too** ([docs/22](22-crystallize.md)–
  [docs/26](26-world.md)): a ghost names points, a silt level names warmth, a
  role-graph names a shape, an event view names rows that already exist, and
  `speaker_identity` is metadata on sense — *who someone is* is content, and it
  arrives through the same gates as everything else. Nothing here names a word.
- **I2 — Clean-room law (two configs).** Content knowledge exists ONLY as
  the two frozen baseline layers (§5, [docs/05](05-substrate-tesseract.md)),
  which load ONLY in CONFIG ENGINE. All v1 gates run in CONFIG SI (empty
  baseline). A gate that only passes with the library loaded has tested
  WordNet, not SI.
- **I3 — Single sense channel** *(one channel, more than one possible voice —
  amended by [docs/21](21-grimm-supporter.md), ledgered as **OD-35**; amended
  again to **I3A** by [docs/26](26-world.md), ledgered at **OD-124**: one
  world, more than one voice, authority owner-origin)*. Typed
  chat-panel text is the whole channel: nothing else reaches SI. Every line in
  it is SENSE — a perturbation to ongoing thought, never a command that owns
  the loop. Each line carries `source: owner | grimm | trusted | stranger`
  (the two new classes are *derived*, [docs/26](26-world.md) §26.2, never
  stored, and the shipped default holds no grant rows at all), and **the source
  decides
  what the line may do, not whether it may arrive**: only a line the owner
  speaks, or one a live grant row of his covers, can teach, auto-commit,
  correct, form or bake an edge, or confirm
  ([docs/21](21-grimm-supporter.md) §21.4, [docs/26](26-world.md) §26.3). The channel stays single; the voice
  is tagged. **Priority is unchanged: an owner line preempts everything, always.**
- **I4 — Append-only truth.** Nothing is deleted. Belief change = aging
  (`superseded_at`), never erasure. Frozen layers are never mutated by live
  learning.
- **I5 — Determinism.** Time is the logical axis `t` (knowledge step);
  wall-clock is metadata excluded from the state hash. All tie-breaks by
  `(t, row_id)`, all randomness seeded by step ⇒ replay ⇒ byte-identical
  state.
- **I6 — Honest label always.** Every output carries FACT / INFERENCE /
  SPECULATION / I-DON'T-KNOW plus a source tag BASE | OWNED | NONE
  ([docs/09](09-claim-labels.md)). No label ⇒ invalid output.
- **I7 — Ask, don't assume.** Any failed write-check or unresolved read-gap
  produces a question, not a committed guess. **Detection ≠ decision:** the
  read path names the gap; the **DRIVE layer** decides whether to spend an
  ask ([docs/17](17-drive-layer-curiosity.md) §17.1) — and when the owner
  answers, that reply is the teaching event (§1.4).
- **I8 — Affect is one installed scalar; emotion names are readouts; no word has
  a fixed emotional address.** Innate
  affective machinery is exactly: the raw signals the organs already produce for
  their own reasons, **one arousal level `a`** that rises when they spike and
  decays over its τ in **logical time**, and **`a` modulating the walk** (reach,
  decay, settle dominance — form, via a version-tagged curve). Curiosity,
  surprise, satisfaction, caution, boredom, frustration are **classification
  labels the audit layer prints over observed patterns**; they have **zero causal
  power** and never feed back into control ([docs/18](18-emotions-engine.md)).
  No privileged sensors, no external setter, no installed coupling matrix, no
  `if <signal> then <behavior>`, no word→emotion edge — a rule engine wearing a
  costume is rejected, and so is a matrix of hand-wired gains between named
  emotions.
  **Plasticity ([docs/20](20-emotional-plasticity.md)):** a point's current
  emotional reading is **computed at generation** as the recency-weighted
  **median** over its whole stamped history (the **read-side** personality,
  repetition-driven — no word flips on one comment), while one hot event's
  durable artifact is the **per-edge bake** at formation, head start +
  confidence bump (the **write-side** trauma, permanent, **no decay tail**). The
  bake's **scale saturates** — `B_max · tanh(k · a_formation)` — and that curve
  is the **one** place the write side reads `a` (the exception OD-22 had to
  reopen, ruled as **OD-33**); no event is unbeatable forever. One scalar, two
  artifacts, and **stamps age on the same clock facts do.** Arousal integrates
  **live, per signal event, during the walk**, and its modulation **lags one
  round** so the frame still finishes its job first.
  **Litmus: at birth, in CONFIG SI, with zero memories, nothing fires — every
  median undefined, `a` at rest, no bake; an emotion system that can be felt
  without content is smuggled content (I1).**
  **Every affect-driven change must be citable** in the per-thought log, under
  the same honesty law as facts (I6).
- **I9 — Idle thought is the same walk, and it cannot teach itself.** When
  senses are quiet and the agenda is empty, SI may **ignite** one hyperpoint and
  run an ordinary walk (§19). "Quiet" is read **per conversation**, not
  globally, once [docs/26](26-world.md) §26.4 lands — a busy server never goes
  globally quiet, so a global test would starve idle forever — and the only
  schedule it may honor is a window on the **logical** axis that can
  **suppress** ignition, never cause one (OD-126; wall-clock triggers are
  excluded by I5 and R-29). Ignition is **read-only**: it may reset rehearsal
  curves and queue a *candidate* question, but it writes **no rows** — all
  content still traces to the owner. Its randomness lives **in state**: the
  ignition seed is part of the session snapshot, so replay reproduces the
  identical monologue (I5). The rendered trace of an idle walk is the internal
  monologue — a truthful rendering of what lit, kept out of the owner-facing
  channel.
- **I10 — The supporter is outside the loop, and his mouth is not his.**
  *(generalized to **I10A** by [docs/26](26-world.md): the supporter was the
  first proof that a voice can be in the room and out of the loop; a stranger
  is that proof at scale, and a trusted user is it for everything outside his
  grant — OD-124)*.
  [docs/21](21-grimm-supporter.md) puts a locally hosted LLM in the *room* as
  SI's co-regulator and keeps it out of SI: **he is not an organ, not a drive
  policy, not the render slot, and holds no state handle.** His lines arrive as
  tagged sense (`source: grimm`) and may light the walk and raise or lower
  signals exactly like any input; they may **never** create a belief row,
  auto-commit against a pending ask, issue `/correct` or `/teach`, form or bake
  an edge, or
  bump confidence (§21.4 — the TEACH fragment's third channel is inside that
  "never" list, because a curriculum the supporter can author is provenance
  laundering wearing a lesson plan). Content never comes from him because two mechanical
  filters in `speakgate.py` check token membership over the taught lexicon and
  claim existence over the live rows — **not** because his prompt says so: *his
  honesty is a filter, not a promise* (L2/L4). Two consequences make the law
  cheap to trust: at `t=0` the lexicon is empty, so **he is structurally mute
  at birth** and cannot bootstrap SI; and **every gate must pass with him
  disconnected**, so the anti-cheat claim never rests on him. I2's clean-room
  law gains a neighbor, not an exception — a pretrained model may talk *at* SI
  through the gate while nothing pretrained may be *inside* `src/`.
  **Under I10A this same template is the stranger's law:** a line from an
  identity with no grant may light walks, raise or lower signals and deposit
  context, and may never create a row, auto-commit, correct, form or bake an
  edge, bump confidence, or confirm a nomination. **Every act of authority in
  the system traces to an owner act, or to a grant row that traces to one**
  ([docs/26](26-world.md) §26.3, §26.4).
