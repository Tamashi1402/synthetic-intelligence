# §19 — IDLE COGNITION: ignition, the monologue, and what idle may touch

New THINK behavior (this batch). **The no-input thinking state.** The drives
chunk listed three perturbations of ongoing thought (owner input, NOW, open
questions); **ignition is the 4th**, and it changes which floor the loop sits
on: **the backlog no longer floors the loop; IGNITION does.** An empty queue on
a quiet tick is not silence — it is a thought of SI's own.

## 19.1 WHEN idle cognition runs

`IDLE ⇔ senses quiet AND agenda empty` — **senses quiet** = no owner line
pending in this logical tick (I3's single channel is the only thing that can be
noisy); **agenda empty** = no open item in the `questions` shelf awaiting an
answer or a revisit pass (§17.3, docs/05 §5.2). When the agenda is not empty the
queue revisit runs first (§17.3); ignition is what happens **after** the backlog
is clear. Idle never preempts an owner line.

**[docs/26](26-world.md) §26.4 changes the reading of "quiet", not the rule.**
Once the panel is a world with N conversations in it, a *global* quiet test is
never true — a busy server would starve idle thought entirely, which is a
second-order way of deleting §19. So quiet is judged **per conversation**: the
thread whose working set is active is the one that must be quiet, and an owner
line in any thread preempts, exactly as it does today (I3A keeps the priority
law untouched). The arc also gives idle a **window**: an owner-set interval of
the **logical** axis in which quiet threads may ignite (**OD-126**). Read that
narrowly, because the direction of authority is the whole law: a closed window
**suppresses** ignition; an open window does not *cause* one — quiet-and-empty
still decides. A wall-clock trigger would be a timer wearing a schedule, which
I5's logical time and R-29's one-clock law both forbid, and which would make the
monologue irreplayable. **Seeds stay hyperpoints**: an event view
([docs/24](24-episodic.md)) is not ignitable in v2 (**OD-115**, recommended no),
and moments surface through origin tugs instead — which is already involuntary
recall.

## 19.2 IGNITION — fire one hyperpoint and run the ordinary walk

**One hyperpoint lights, and the ordinary walk (§16) runs from it.** No new
machinery: the same spreading activation, the same settle rule, the same trace.
The sampler is **not uniform random** — ignition probability is weighted by four
things the loop already has state for:

| weight | class | what it is | what it becomes, in plain words |
|--------|-------|-----------|--------------------------------|
| **recency** | TEMPERAMENT | recently touched points fire easier (last-lit `t` of the point) | what just happened stays on the mind |
| **edge bake** | TEMPERAMENT | **hot-*baked* edges pull their endpoints toward ignition** (§20.4a) | what mattered keeps interrupting |
| **decay deficit** | TEMPERAMENT | young weak edges close to their floor (§8.4 curve) weight up | the system drifts toward **what it is about to forget** |
| **live contradiction slots** | TEMPERAMENT | regions with an unsettled contradiction stay warm (§6.4) | unresolved things stay available |
| **silt warmth** *(the v2 arc's fifth term, [docs/23](23-context.md))* | **SPECIES, and unarmed at 0** | what this conversation is *about*, from the session's decaying salience table | the topic of the last few exchanges keeps coming up |

**The fifth row is classed differently on purpose, and it is the B7 batch's law
doing the work.** The four original weights are a **temperament** draw — what
intrudes when it is quiet is individual (OD-87), and *no sweep may ever touch a
temperament row*. Silt is **not** a personality: it is the same session
mechanism for every individual, so it enters as a SPECIES row, and it ships
**unarmed at `0`** so the pre-arc sampler stays bit-for-bit the one G10 measured.
A G16 leg installs the smallest non-zero step its grid can express, at a declared
point, and labels it a floor — the term is then *seen* biasing the sampler
without any magnitude being invented (OD-108 is the owner's open question about
this exact row). If the owner ever classes it as temperament instead, that is a
**19th draw**: `seed_format` bumps and every hash since B1 changes with it
(eng/03 §1.2, eng/10 §3) — which is why the row is called out here rather than
buried in a registry.

**Round 3 opened the question in this table and the bake-scale amendment closed
it (§20.4a):** the salience weight reads the edge's **bake** — neither the raw
stamp list nor the point's median. **OPEN-32 is CLOSED by that naming.** The
reason is mechanical, not stylistic: the bake is the only durable affective
quantity attached to an edge, and ignition is an edge-selection process; the
median belongs to *points* and belongs to the *read* side, so letting it steer
which edge lights would smuggle a read-side statistic into a launch weight and
make the intrusive-thought loop indistinguishable from ordinary retrieval. A
point with one old traumatic stamp and a calm recent history therefore still
daydreams about it — which is the honest behavior, since the trauma's durable
artifact is exactly that baked edge — while the *feeling* of the word while it
spreads remains the median's job alone. One consequence worth naming: because
`bake = B_max · tanh(k · a_formation)`, ignition's affective weight is **bounded
by `B_max`** — the 3am replay of the first kitchen fire and the tenth both score
near the same cap, and no baked edge can ever own the walk outright. Either way
the other three weights read the same `clock.py` recency curve (§8.4, R-29),
never a second decay.

**This single weighting rule IS rehearsal, worry, and intrusive thought. None of
them are designed, named, or scheduled.** There is no `rehearse()`, no
`worry()` entry point, no scheduler that decides "now we practice". They are
**readouts** of which weight dominated a given ignition — the same demotion §18
applied to emotions, applied here to idle behavior. Weight ratios are
**version-tagged constants tuned by G10(e), not hand-argued** (OPEN-25).

**The amendment's sentence — "worry, rehearsal, and the 3am kitchen-fire replay
are all the same mechanism" — is a claim about this table and nothing else.**
There is one sampler; "3am" is not a branch, it is the same weighting rule
running on a quiet tick with a high-bake edge in the candidate set (and, if `a`
has not decayed from the day's events, a widened reach on top of it — §20.1).

## 19.3 THE CASCADE IS THE INTERNAL MONOLOGUE

Worked shape (illustrative of mechanism, not of content):

```
fire "apple" → the apple→red edge lights → red spreads to whatever else touches
red → the next memory lights → chain continues until settle or budget end
```

- **The rendered decode of each idle walk's trace IS the internal monologue:**
  the lit points, in order, through the **same render face** the read path uses
  (lattice → words, §7.5). **NOT generated prose — a truthful rendering of what
  actually lit.** Nothing composes, expands, or narrates; the renderer may only
  surface row/point material that actually participated in the walk (the same
  copy-not-create contract as FACT, §9).
- **Logged as a monologue stream, kept separate from owner-directed output.** No
  owner-facing frame is active — SI is **not talking to anyone**. The stream is
  an artifact of the mind log (append-only, I4), exported with the session, and
  **never enters the response channel**. Whether SI may *volunteer* a monologue
  line to the owner is out of scope for v1 (OPEN-23).
  **The supporter batch adds a reader, not a channel ([docs/21](21-grimm-supporter.md)
  §21.2, OD-38):** `read_monologue()` lets a *third party* look at this lane and
  write about it in the lab notebook. That is not a new SI output — the lane was
  already exported with the session (`gates/*/monologue.jsonl`), so the change is
  in **legibility**, not in reachability, and OPEN-23's actual subject (SI
  volunteering a line into the response channel) is untouched. What it does
  change is worth stating plainly, because it is the least protected surface in
  the design: **the monologue now has an audience that can act on it**, so reads
  are logged, the notebook is citation-only, and nothing he reads may come back
  out as content (§21.3's filters run on the way in either way). R-33.
- **Label law:** a monologue entry asserts nothing, so it carries no FACT /
  INFERENCE / SPECULATION claim (I6 is satisfied by claiming nothing). It ships
  with its activation trace + `a` **trajectory**, like every other walk output
  (§16.4, §16.6) — a cascade is a walk, so it colors itself per event the same
  way a sensed thought does (§20.1); it just has no tokens arriving from outside.

## 19.4 BOUNDARIES (the 10,000-thoughts-per-second guard)

- **RATE:** **one idle thought per quiet tick, with a per-hour budget.**
  **Silence means slow thought, never spinning.** The per-hour budget is counted
  in logical ticks (I5 keeps its grip: no wall-clock), and the value is parked
  (OPEN-26) — G10(a) fails on a spinning idle loop regardless.
- **READ-ONLY:** **idle walks NEVER write rows.** They may
  (a) **reset rehearsal curves** — touching an edge **is** rehearsal (§8.4),
  and (b) **push items onto the agenda**: a cascade that crosses **two taught
  regions** raises a *candidate question* onto the queue (→ then the normal
  drive budget decides whether it is ever emitted, §17.3).
  **All content knowledge still traces to the owner: an idle walk can NOTICE,
  never LEARN.** The I4/I1 boundary is exact here — idle cognition changes
  curves and the queue, and writes no fact row.
- **Re-stamping (lean: yes, bounded):** a cascade that re-lights a hot edge
  **re-stamps it with the current state** — the natural rehearsal +
  state-congruence loop — under the conservation guard from §18.5's bias
  close-out, which still bounds it (**OPEN-24**).
- **DETERMINISM (G5 extension):** **the randomness lives in state, not in time.**
  A **seeded PRNG whose seed is part of the session snapshot** ⇒ **replay
  produces the identical monologue.** **Unseeded ignition breaks G5 and is a
  design violation** — the same discipline that made the guess slot (R11) and the
  wake/replay law (§16.5) reproducible.

## 19.5 Feeling-from-thought (the one allowed affective loop)

**A cascade that hits its own live contradiction raises the contradiction signal
→ arousal → the walk modulates** (§18.1). This is the owner's point, made
mechanical: **emotion comes from what it thinks about**, not from a timer and
not from an installed matrix. Note what it is *not*: it is not
`if contradiction: feel_caution()`. The cascade produces a **signal** the
arousal integrator already consumes; the label the audit layer may print over
the pattern ("frustration") has no causal role whatsoever (§18.2).

Because this is a genuine feedback path (cascade → contradiction → `a` → spread
→ cascade), it inherits the boundedness law: §18.5's per-round normalization plus
G9's bounded-feedback leg, and `a`'s τ is what makes it fade rather than ladder.
**Worry that compounds is a bug with a name** (R-22).

**Round 3 restates this loop's timing, not its permission (§20.1):** the
contradiction a cascade hits updates `a` **as an event inside that cascade**, and
the spread it widens lands on the **next round** — so an idle cascade colors
itself as it runs, with the same one-round lag as a sensed thought, and G10's
legs see a trajectory rather than a scalar. The second guard now sits here too: a
cascade that re-lights a contested region moves that point's **median** only if
it leaves **stamps**, and a stamp from an idle walk is the OPEN-24 question —
which is why "idle may re-stamp" and "idle may not bake" are the same ruling seen
from two sides (§20.4, G10(b)).

## 19.6 What idle may and may not change

| idle may | idle may not |
|----------|--------------|
| reset an edge's rehearsal / decay curve | create, modify, or supersede a row (I4's only writer is the write path) |
| push a **candidate** question onto the agenda | emit it itself (the drive's budget decides), or answer it |
| re-stamp charge on edges it re-lights (OPEN-24) | stamp a frozen L1/L2 edge (§18.5, G9) — and **never a bake**: an idle walk forms no edge, so it buys none (§20.4, G10(b)) |
| raise the contradiction signal via §19.5 | promote or demote a stage (§11.1) |
| be read by the audit layer as a pattern | feed any emotion label back into control (§18.7) |
| consume the per-tick / per-hour budget | run unseeded or non-reproducible (§19.4) |
| **host the v2 arc's scratch** — a ghost, a role-graph, a silt deposit, an origin re-light ([docs/22](22-crystallize.md)/[docs/23](23-context.md)/[docs/24](24-episodic.md)/[docs/25](25-skills.md)) — and accrue **candidates** in session state from its own frames | write, mint, ratify or name any of it: an idle recurrence is one more recurrence in the detector's count, **not** a privileged one, and the nomination still needs the owner's line; a re-light adds **no stamping site** (OPEN-24 ships unrulled, recorded `off`); no event view is ever an ignition **seed** (OD-115) |
| be suppressed by a closed window | be **caused** by an open one, or read a wall-clock at all (OD-126, I5, R-29) |

## 19.7 The honest limit, published

At S0 in CONFIG SI, L3 is nearly empty, so ignition has almost nothing to fire
and the monologue is **short or silent** — correctly so. A newborn with no
memories does not daydream much, and an SI whose idle transcript looks impressive
after three owner turns is showing a preloaded corpus, not a mind (§3.4's
empty-state test, R-25). Monologue density is a **reported curve**, not a demo
feature.

## 19.8 Open items

| id | open item | where it bites |
|----|-----------|----------------|
| OPEN-25 | ignition weight ratios (recency vs **bake** vs decay-deficit vs contradiction-warmth) — version-tagged constants, tuned by G10(e) | §19.2 |
| OPEN-26 | per-hour idle budget: value + its unit in logical ticks | §19.4, G10(a) |
| OPEN-27 | monologue rendering depth — how many lit points render per entry (lean: **the settle's top-k**, same as any readout) | §19.3 |
| OPEN-24 | does an idle cascade write fresh charge stamps (lean: yes, bounded by §18.5's guard) | §19.4, §18.5 |
| OPEN-23 | can a monologue line ever be owner-facing (v1 lean: **no**) | §19.3, I3 |
| OD-38 | the supporter's `read_monologue()` + the lab notebook: a **third-party audience** for this lane (SI's own output rules unchanged) | §19.3, §21.2, R-33 |
| OPEN-28 | "crosses two taught regions" — the exact adjacency test for agenda promotion | §19.4, G10(d) |
| ~~OPEN-32~~ | **CLOSED by the bake-scale amendment (§20.4a):** the ignition salience weight reads the edge's **bake**, bounded by `B_max` — not raw stamps, not the median. Reads `clock.py`'s curve for the other three weights, never a second decay | §19.2, G10(e) |

**The v2 arc's rows, on this table, unrulled and named.** §19 gains five open
items it cannot resolve alone, and each has a build default declared in the doc
that raised it:

| id | open item | the default tonight's build runs at |
|----|-----------|--------------------------------------|
| OD-104 | what silt may bias: **ignition only** vs ignition + spread | ignition only — the **recorded exception** to the calm-view terminality law (eng/10 §8 law 1), which is why it is an OD and not a refactor |
| OD-108 | the silt ignition weight's **class** under B7's split | SPECIES, unarmed at 0 (§19.2's fifth row); if the owner rules temperament, `seed_format` bumps and every hash moves |
| OD-126 | scheduled idle windows: logical-axis windows, a budget per window, and the interaction with sleep's P3/P4 | suppress-only, expressed in ticks; a window cannot make a night happen, because sleep is counter-triggered |
| OD-115 | may idle ignition select an **event view** as its seed | **no** — seeds stay hyperpoints; moments arrive through origin tugs, which is already involuntary recall |
| OD-127 | per-thread silt ([docs/23](23-context.md) OD-109 option (c)) | the table is **keyed by conversation from the first build**, so a per-thread world is a scope decision, not a new mechanism |
