# §25 — SKILLS: procedural crystallization (v2 arc — folded law 2026-10-23; G18, OD-116…OD-122)

> Status: **FOLDED AS LAW 2026-10-23** by the owner's order to engineer the v2
> arc and build it. The HOW is [engineering/12](../engineering/12-v2-arc.md); the gate family
> is **G18** and the open items are **OD-116 … OD-122**.
> The draft's own rule still binds item by item: an unrulled OD item is not a
> licence for the build to choose, so wherever the build could not stop for a
> ruling it ran the **least-authority option**, said so in §25.11, and
> left the ruling with the owner — the OD-95 discipline, never a silent default.
> **No magnitude is quoted in this file**, and no value exists before its sweep
> artifact does.
>
> Numbering per §23.9: this doc takes **OD-116…OD-122 / G18**. It is the third reader-side proposal of the v2
> arc and the exact twin of [22-crystallize.md](22-crystallize.md) applied
> to *shape* instead of *content*: crystallize nominates things, silt warms
> them, episodic binds their origin, **skills nominate moves.**

## 25.0 Thesis line

**SI learns what, never how.** Zero gradient steps is the thesis (§0), and
its fine print is that SI's *thinking* never improves with practice: every
walk runs the same frozen shape, the sampler weights are species or
temperament (OD-87: no sweep may ever touch a temperament row), and the
only thing a thousand thoughts change is what SI knows, never what it does
with knowing. Skills close that gap the only way this constitution allows:
**a recurring, generalized, useful thought-shape may be nominated as a
named move, and the owner's word makes it a citable row.** Procedural
knowledge enters the lattice exactly one way: an owner-confirmed row. Not a
program, not a weight, not a constant: a row.

Litmus: with skills active and the owner silent, **no belief row changes**
(its only permitted state write is the ordinary `questions` row of the emitted
`move-nomination` — the exception is [docs/03](03-birth-state.md) §3.3's).
And with skill rows installed (owner-confirmed), removing them changes no
lattice byte and no law: walks still run, gates still pass; the only diff
is composition order.

## 25.1 The gap

| kind of knowledge | exists? | carrier |
|-------------------|---------|---------|
| things (facts) | yes | taught rows, §6 |
| corrections | yes | supersede chain, §8 |
| concepts (from experience) | §22, proposed | crystallize points |
| working context | §23, proposed | silt |
| origin | §24, proposed | prov-links, event views |
| **how to think (moves)** | **missing** | nothing: the walk's shape is species-fixed, and no mechanism can even *notice* that a shape recurs |

The gap is invisible in a demo and fatal in a life: a mind that never
forms habits of thought must re-derive "compare two things via their shared
filler" from lattice mechanics every single time, and cannot prefer it when
it keeps working.

## 25.2 Shape of the mechanism (three stages, the §22 table again)

| stage | lives in | lifetime | writable to lattice? |
|-------|----------|----------|----------------------|
| **shape-scratch** (role-graph) | a single walk frame | one frame; destroyed at settle | never |
| **shape-candidate** | session state (crystallize's OD-98 family) | session; storage ruled with it | never |
| **ask → skill row** | drive emission → write path | ask is one more ask kind; the row exists only after owner-confirmed write | only via the ordinary write gates (§6) |

## 25.3 Stage 1 — SHAPE-SCRATCH (frame-scoped)

During any walk, the frame MAY hold its own traversal pattern as a
**role-graph**: nodes are roles (`seed`, `lit_a`, `lit_b`, `shared_filler`),
edges are the relation-kinds actually traversed. Frame-scoped scratch in the
ghost family (§22.2): destroyed at settle, never evidence, bounded by the
`GHOST_MAX` discipline, over bound = no scratch, never eviction games.

**The defining rule, and the seam with §22:** a shape is *roles, not
members*. The same role-graph with different fillers is the SAME shape; the
same fillers are NOT a shape at all. Mutual exclusion: a shape-candidate
whose recurrences all share one filler-set is a content cluster and is
forwarded to the §22 detector's territory, never opened as a skill (OD-121;
the generality bound, G18(f)).

## 25.4 Stage 2 — SHAPE-CANDIDATE (the detector and its evidence law)

Reader-side, on crystallize.py's discipline: imports nothing from
write/gate/drive-action. A candidate opens for role-graph `S` iff:

1. `S` recurs across `≥ R_min_shape` **distinct frames**, each `Δ_min_shape`
   apart (the §22.3 rule 3 anti-hot-afternoon law, verbatim);
2. the recurrences instantiate `≥ K_filler` **distinct filler-sets**:
   generality is earned, a move that only ever connects the same three
   points is a concept, not a skill;
3. **utility is citable, not felt.** Each recurrence frame carries an
   outcome tag read from the eventlog ONLY (citable event rows: question
   resolved, realized-gain event §8.2, owner reply received, or none). The
   detector **never reads `a`** (I8, the same leg as §22.3's first
   exclusion): the gain rows are content evidence; the arousal they once
   fed is not;
4. a live or frozen **skill row for the same role-graph does not already
   exist**; if one does, recurrence increments *that row's* confidence leg
   (§8), and no candidate opens (the §22.3 rule 4 pattern).

Candidate record (session state): `{role_graph, filler_sets_seen,
recurrence_count, outcome_tags[], first_seen_t, last_seen_t}`.

Idle-sourced recurrences count, same as in §22; the spam bound (G18(e)) is
what keeps an idle loop from farming a junk move.

## 25.5 Stage 3 — ASK and the only path to a skill row

Same drive interface, same queue discipline, same integer cap. New ask kind
`move-nomination`; template under the honest-label law (I6):

```
"I keep solving things the same way: {shape in taught words}. Is this a
move worth keeping? Does it have a name?"
labels: pattern=FACT/OWNED (citations: each recurrence frame + each outcome
row), name=NONE
kind: ask-back, target: move-nomination, status: open
```

Branches: owner names it ("that's comparison") → ordinary gates → skill row
in L3 (`skills` shelf, id 9 — the reserved shelf §5.2 opens with this kind; see
§25.11's build note), the serialized role-graph template, the
name, provenance `prov_kind: answer-to-ask`, `question_id:` the
`move-nomination` ask row, `prov_speaker: owner` — the same ordinary
answer-to-ask commit §22.4 uses, so `gain_kind: demonstrated` per §8.2 and **no
new `prov_kind` literal** (engineering/01 §4's five stay five). "Real, no word"
→ placeholder `skill:{hash(role_graph)}`, renamable
later by an ordinary owner-confirmed row (the OD-100 pattern). No answer →
normal ask lifecycle. `source: grimm` can never confirm (I10); once §26
lands, **neither can a stranger** (the G19(d) leg).

## 25.6 What a skill may and may not do once written

A skill row is a **read-side composition hint**: a frame MAY consult skill
rows when choosing traversal order. A ratified habit biases how the walk
composes; it never:

- writes, forms, or bakes anything;
- touches arbitration, the label ladder, or any ask decision;
- moves any constant (OD-87 stands: a skill is not a constant and not a
  sweep target);
- becomes code. **A skill is a row, not a program.** No new machinery ever
  executes a skill; the ordinary walk merely prefers a shape it has on
  file.

Usage tracking is read-side stats (times consulted, outcome tags),
audit-visible, following the §8 confidence discipline on the skill row
(OD-120). Nothing feeds back automatically; the confidence leg is the
confidence leg.

## 25.7 Invariant impact audit

| invariant | impact | why it holds |
|-----------|--------|--------------|
| I1 | none | installed form; the shape was lived, the *name* is taught. No innate move vocabulary ships: an unratified shape is addressable as `skill:{hash}`, and RENDER describes shapes in taught words only |
| I2 | none | filler-sets instantiate from live rows; frozen rows filler-only |
| I3 | none | single channel, owner confirms |
| I4 | none | skill rows are append-only; retirement = supersede, never erase |
| I5 | **load-bearing** | detection deterministic; skill rows deterministic writes; candidate storage rides crystallize's OD-98 storage ruling |
| I6 | none | the ask carries the honest template |
| I7 | none | detection ≠ decision; a shape can only become a move through ask + owner answer |
| I8 | **load-bearing** | the detector reads outcome *rows*, never `a`; audit may LATER print readouts over skills ("keeps using comparison when asked about food") with zero causal power |
| I9 | none | idle walks may host shape-scratch; candidates accrue through the same gates; idle still writes nothing |
| I10 | none | grimm lines may light walks whose shapes scratch; confirm is owner-only |

## 25.8 Gates — G18

- **G18(a) determinism:** fixed transcript, skills on vs off: lattice hash
  byte-identical (rows only appear on owner-confirmed writes, which are
  deterministic).
- **G18(b) deletion:** remove the skill machinery, rerun: no lattice byte
  and no ask output except `move-nomination` asks; installed skill rows sit
  unread (removing the *rows* is a fold-time migration, not runtime).
- **G18(c) arousal isolation:** sweep `a` full range: candidate table
  byte-identical.
- **G18(d) owner-only:** grimm (and, post-§26, stranger or ungranted
  trusted) answers to a move-nomination produce zero writes.
- **G18(e) spam bound:** junk shape repetitions (random walks, single
  filler-set, no outcome rows) produce zero asks.
- **G18(f) generality bound:** a "shape" below `K_filler` distinct
  filler-sets never opens a candidate and is forwarded to the §22 detector.
- **R-21 extension:** no emotion vocabulary in the shape machinery;
  readouts in audit.py only.

## 25.9 Open decisions introduced

| OD | question |
|----|----------|
| OD-116 | constants: `R_min_shape`, `Δ_min_shape`, `K_filler`, shape-scratch budget — no value before `gates/G18/shape_sweep.json` |
| OD-117 | candidate storage: rides crystallize's OD-98 ruling or its own hash ruling (I5 wording) |
| OD-118 | what installed skills may bias: composition order only (recommended) vs also ignition seeding (rejected for v2: it drifts toward sampler-tuning, OD-87's territory) |
| OD-119 | module: `skills.py` reader vs a second family inside crystallize.py (recommended: separate file, same discipline, for the deletion tests to be independent) |
| OD-120 | usage/confidence tracking on skill rows: full §8 confidence legs vs read-side counters only (recommended counters: a skill that "loses confidence" should be asked about, not decayed) |
| OD-121 | the mutual-exclusion seam with §22: forwarding rule, and who owns a shape whose members are crystallize-born points |
| OD-122 | skill retirement: supersede path (an owner may retire a move he ratifies as harmful), and whether a retired skill's row keeps its usage stats citable (I4: yes) |

## 25.10 What this document does NOT claim

- **Not learning-to-learn.** The owner has ruled that round last, after
  prototyping survives; nothing here reopens it. The walk's laws stay
  species-fixed, its constants untouched (OD-87), and a skill cannot modify
  anything, only be preferred.
- No new code path, ever: a skill is knowledge about how SI likes to think,
  and in this constitution knowledge has exactly one entry point: an
  owner-confirmed row.
- No claim that skills make SI "smarter": they remove one ceiling (a
  re-derived move every time) and add zero authority, like every other
  proposal in this arc.

## 25.11 What the fold changed, and what the build runs at

Three edits, none of them to a law:

1. **Numbering** per §23.9: OD-113…OD-119 → **OD-116…OD-122**; crystallize's
   candidate-storage ruling, which this doc's §25.2 and §25.7 ride, is
   **OD-98** (not the draft's "OD-94 family" — B8's sleep walls occupy those
   seats), and the placeholder-rename pattern is **OD-100**.
2. **The shelf already exists.** `SHELF_SKILLS = 9` is a registered id in
   [engineering/01](../engineering/01-data-schemas.md) §2, reserved since the
   substrate round; a skill row is a **kind** on it, so §5.1's "never new
   structures" law holds without bending anything.
3. **"a skill is a row, not a program" is now mechanically true.** The crate
   that detects shapes has no dependency edge into the write gate, and the walk
   reads skill rows only as an ordering preference — so *executing* a skill
   would require an edge the build graph does not contain. That is the same
   proof style as `si-sleep ↛ si-learn`, and it is what makes G18(b)'s deletion
   test independent of G15(b)'s.

**Declared build points.**

- **`SKILLS = off | on` and `SKILL_COMPOSITION = off | on`** — ruling rows,
  shipped **off**, installed **on** by G18's scenario. Two rows because
  detection and use are separable decisions: the owner may want nominations
  firing while preferring the composition order stays unblessed. OD-118 is the
  use-side question and OD-116 blocks the detection-side values, so neither
  default is a ruling.
- **OD-116**: `R_MIN_SHAPE`, `DELTA_MIN_SHAPE`, `K_FILLER`, `SHAPE_MAX` are
  swept SPECIES rows at `UNMEASURED_PLACEHOLDER` with `hi` open per OD-83, so
  `gates/G18/shape_sweep.json` publishes a **refusal**; legs that need a value
  run at the box's floor and label it.
- **OD-117**: candidate storage **rides crystallize's OD-98 ruling** as drafted
  — session-only, declared hash-exempt, logged to `skills.jsonl`. A separate
  table, not a second family inside the crystallize detector, which is OD-119's
  recommended answer and makes the two deletion tests independent.
- **OD-118: composition order only.** Ignition seeding is rejected in the code,
  not only in the prose — there is no path from a skill row to the sampler, so
  OD-87's "no sweep may ever touch a temperament row" cannot be reached around
  by way of a habit.
- **OD-120: read-side counters, not confidence legs.** A skill's usage stats
  live in the same derived, never-stored family as the median (R-28), so a
  habit cannot lose confidence and cannot launder an authority bump through a
  counter. Retirement is §8 supersession on the row (OD-122), and a retired
  skill's stats stay citable.
- **OD-121 (the mutual-exclusion seam)** is built as a forwarding rule inside the
  session, not inside either detector, so neither reader owns the other's
  territory: a shape whose recurrences share one filler-set is dropped from the
  shape table and its frames stay available to the content detector.
- **No new provenance kind.** A named skill is minted by an ordinary
  `answer-to-ask` commit — `question_id` names the `move-nomination` ask,
  `prov_speaker: owner`, therefore `gain_kind: demonstrated` (§8.2's mapping,
  unchanged). `SHELF_SKILLS = 9` is a *shelf*, and shelf 9's rows are the
  `skills` protected class; neither is a `prov_kind`. The arc's two minting
  channels together add zero literals to [engineering/01 §4](../engineering/01-data-schemas.md)'s
  envelope, which is what keeps G5c's schema diff empty and docs/15 item 3's
  closed list closed.
