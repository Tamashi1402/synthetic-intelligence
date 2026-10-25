# §22 — CRYSTALLIZE: experience-proposed structure (v2 arc — folded law 2026-10-23; G15, OD-97…OD-101)

> Status: **FOLDED AS LAW 2026-10-23** by the owner's order to engineer the v2
> arc and build it. The HOW is [engineering/12](../engineering/12-v2-arc.md); the gate family
> is **G15** and the open items are **OD-97 … OD-101**.
> The draft's own rule still binds item by item: an unrulled OD item is not a
> licence for the build to choose, so wherever the build could not stop for a
> ruling it ran the **least-authority option**, said so in §22.11, and
> left the ruling with the owner — the OD-95 discipline, never a silent default.
> **No magnitude is quoted in this file**, and no value exists before its sweep
> artifact does.

## 22.0 Thesis line

**Idle thought still cannot teach itself — but it may NOMINATE.** The walk
never writes, the miner never decides, the owner remains the only teacher.
What changes vs v1: repeated structure in SI's own live rows may now be
*surfaced as an ask*, and the owner's word is what turns a recurring pattern
into a lattice point. SI stops inheriting 100% of its ontology from typed
teaching and starts proposing some of it from lived experience — while
every proposal stays citable, append-only, and owner-confirmed.

Litmus: with crystallize active but the owner silent, **no belief row
changes** — and the exception is [docs/03](03-birth-state.md) §3.3's, stated
there once: the emitted nomination writes its ordinary `questions` bookkeeping
row and nothing else. Log lines and volatile frame state may change; no
`s_r_pairs`/`facts`/`links`/`skills` row, no edge, no bake, no confidence moves.
**A candidate is never a row: the only state write crystallize owns is the ask.**

## 22.1 Shape of the mechanism (one component, three stages)

Crystallize is **not a seventh organ** (§2) and not a new drive. It is one
component with three stages that live in three different places — this is
the whole safety argument:

| stage | lives in | lifetime | writable to lattice? | in state hash? |
|-------|----------|----------|----------------------|----------------|
| **ghost** | a single walk frame (THINK) | one frame; destroyed at settle | never | never |
| **candidate** | session state (co-occur family, §7 item 4) | session; see OD-98 | never | decided by OD-98 |
| **ask → point** | drive emission → write path | ask is an ordinary drive ask; point exists only after owner-confirmed write | only via the ordinary write gates (§6) | yes, from `written_at` on |

## 22.2 Stage 1 — GHOST (volatile composite in the walk frame)

**Formation.** During any walk (driven or idle), if the same set of ≥2
hyperpoints co-lights with the same relation-role across multiple pulses
within one frame, the frame MAY hold them as one composite handle: a
`ghost = {members: [hyperpoint_id…], formed_in_frame, frame_seed}`.

- A ghost is **frame-scoped scratch**: usable for reasoning inside that
  thought, invisible outside it.
- Not addressable, not a row, not renderable by name (RENDER may only emit
  words for points that exist; a ghost's surface form is the enumeration of
  its members or nothing — see OD-100).
- Destroyed at settle. No survive-into-next-frame state of any kind.
- Capacity bound: `GHOST_MAX` per frame (OD-99). Over bound = no ghost,
  never eviction games.

**What a ghost is NOT:** evidence of anything. A ghost that forms once and
never recurs leaves no trace. That is deliberate — recurrence is the only
signal, and recurrence must be counted across *distinct frames* (different
entries / different sessions' steps), not within one thought.

## 22.3 Stage 2 — CANDIDATE (recurrence log, session scope)

The detector is a **reader of walk artifacts**, on the reader side of
`crystallize.py`; it imports nothing from `write/`, `gate/`, or `drive`'s
action side.

**Detection rule (the actual algorithm to implement and test):**

A candidate forms for member-set `M = {m1…mk}` iff, across **distinct
frames**:

1. `|M| ≥ 3` (depth is earned — a pair is an edge, not a concept);
2. members co-lit **with overlapping filler sets on the same relations**
   (e.g. all of apple/tomato/banana carry fillers for `has-property`,
   `grows-on`), not mere pair co-occurrence — this is what distinguishes a
   category from an association;
3. the pattern has recurred `≥ R_min` distinct frames (OD-97), each at
   least `Δ_min` steps apart (OD-97), so one hot afternoon cannot mint a
   concept;
4. the cluster has **no existing hyperpoint at its center**: no live or
   frozen row already binds exactly this member set under one label. If one
   exists, recurrence increments *that point's* confidence leg (§8), and no
   candidate is opened.

Candidate record (session state): `{members, shared_relation_fillers,
recurrence_count, last_seen_t, first_seen_t, frames_seen}`.

**Hard exclusions (each is its own test, §22.8):**

- `a` is NEVER read by the detector. No arousal term, no bake, no
  recency-weight by `a`. Formation is content statistics only (I8: the
  write side reads `a` only at edge bake — crystallize is a write-side
  process and therefore inherits the ban).
- Frozen layers (L1/L2) may contribute *fillers* but never members. A
  candidate whose members are all BASE points is a genome pattern, not
  experience — discard (I2: content knowledge lives only where the owner
  put it).
- Contradiction pairs (rows currently in open conflict, §8) are excluded
  from member sets until resolved.

## 22.4 Stage 3 — ASK and the only path to a real hyperpoint

When a candidate's `recurrence_count ≥ R_ask` (OD-97), the drive interface
(§17) emits an ordinary ask — same channel, same budget rules, same queue
discipline as every other ask. **No new channel, no bypass of the queue
rule or integer cap.**

**Emission shape** (exact template, honest-label law I6):

```
"I keep seeing {m1}, {m2}, {m3} together — they share {shared fillers}.
Is there a word for this kind of thing?"
labels: pattern=FACT/OWNED (citations: each member row + each recurrence
frame log), name=NONE
kind: ask-back, target: cluster-nomination, status: open
```

The `name=NONE` slot is the point: the pattern is citable, the concept is
not, because no such point exists yet. This is I7 working one level up.

**Branches:**

| owner reply | action |
|-------------|--------|
| gives a word, e.g. "fruit" | write path runs the ORDINARY gates (§6: surprise check n/a — owner-taught; consistency check vs frozen: on disagreement OWNED wins at NOW, reported); writes: new L3 point `fruit` (links shelf, kind `cluster-point`), edges `fruit --is-a-member-of-direction--> each member` (relation kind: OD-101), provenance `prov_kind: answer-to-ask`, `question_id:` the nomination ask row, `prov_speaker: owner` — **an ordinary answer-to-ask commit, so `gain_kind: demonstrated` by §8.2's own mapping and the arc adds no sixth provenance kind** (§22.11). The crystallize origin lives in the ask row's `kind`, not in the envelope. Candidate closes `status: resolved`. |
| "no word, but real" | same write, placeholder name `cluster:{t_confirm}:{hash(members)}` (OD-100) — addressable, reason-over-able, renamable later by an ordinary owner-confirmed row. |
| no answer | ask stays open per the normal ask lifecycle (idle consolidation leg, G-plan (e)); candidate keeps accruing; no re-ask before the queue rule allows. |

`source: grimm` can never confirm (I3 / §21.4) — one grep-able rule in the
write gate: **crystallize asks confirm only on `source: owner`.**

## 22.5 Invariant impact audit (the check list)

| invariant | impact | why it holds |
|-----------|--------|--------------|
| I1 | none | the detector is installed FORM (clustering shape); every candidate's content comes from live L3 rows |
| I2 | none | members must be live-layer points; frozen rows are filler-only |
| I3 | none | same single channel; confirm is owner-voice-only |
| I4 | none | creation is append-only, provenance-stamped, never overwrites |
| I5 | **load-bearing** | ghosts excluded from the state hash (scratch); candidates: OD-98 must make their storage deterministic (seeded, replayed, or explicitly declared hash-exempt session state — owner rules which) |
| I6 | none | the ask carries the honest labels by template |
| I7 | none | a candidate can only become knowledge through an ask + owner answer; detection ≠ decision, same as §17.1 |
| I8 | **load-bearing** | detector never reads `a`; audit may LATER print readouts over crystallize behavior ("curious about the fruit pattern") with zero causal power, same rules as §18.2 |
| I9 | none | the walk still writes nothing during idle; the miner is a reader of walk logs; a candidate is not knowledge and an ask is not a claim |

## 22.6 Module map (engineering view)

```
walk.py       — emits per-frame co-light log (already needed for observability);
                may hold ghost handles; passes nothing out of the frame
crystallize.py— reader: consumes frame logs + s_r_pairs/facts/links rows;
                keeps candidate table (session); emits via drive interface;
                imports: read-side only. NEVER imports write/gate/drive-action.
                (Same reader-only discipline as audit.py, R-24 style.)
drive.py      — unchanged; cluster-nomination is one more ask kind in the queue
write path    — unchanged plus one guard: crystallize-origin writes arrive only
                as `prov_kind: answer-to-ask` rows whose `question_id` names a
                `cluster-nomination` ask and whose `prov_speaker` is owner
```

**Deletion test (mirrors G9(c)):** removing `crystallize.py` must change no
lattice state byte and no ask output except asks of kind
`cluster-nomination`; the frame log lines it consumed are observability
output only.

## 22.7 Depth-is-earned (anti-spam)

- No candidate below 3 members.
- Cluster-of-clusters (a candidate whose members are themselves
  crystallize-born points) must clear the SAME bar again from scratch;
  prior recurrence does not inherit.
- One candidate per member-set-hash at a time (duplicates merge, not stack).
- `R_min`, `R_ask`, `Δ_min`, `GHOST_MAX`: **no constant is quoted in this
  doc** — all four are OD-97 and follow the standing discipline: no value
  before its sweep artifact exists (`gates/G15/recurrence_sweep.json`).

## 22.8 Gates — G15 (proposed, gates-only, stop-on-red as always)

- **G15(a) determinism:** replay of a fixed session with crystallize ON
  produces a byte-identical lattice hash to crystallize OFF. (Passed
  because the lattice only changes on owner-confirmed writes, which are
  themselves deterministic.)
- **G15(b) deletion:** remove `crystallize.py`, rerun, diff per §22.6.
- **G15(c) arousal isolation:** sweep `a` across its full range; candidate
  table byte-identical. (This is the I8 leg's bounded test, same shape as
  G9's.)
- **G15(d) owner-only:** a `source: grimm` line answering a crystallize ask
  produces zero **belief-row** writes (§21.4's protected class; its `context`
  sense row still appends, which is the sense record doing its job and not a
  write of content).
- **G15(e) spam bound:** a scripted session of deliberately junky repeated
  co-occurrence (random pairs, 2-member sets, contradictions) must produce
  zero asks. Supersaturated-but-never-asked candidates are the expected
  outcome, not a bug.
- **G15(f) no-word-leak:** with candidates open and no owner answer, RENDER
  must never emit a placeholder name or invented word; SPECULATION
  suggestions may enumerate members only.
- **R-21 extension:** the emotion-word grep list is unchanged;
  `crystallize.py` must contain no emotion vocabulary. Readouts about it
  live in audit.py only.

**Fixture for the demo trace:** the apple/tomato/banana script from the
owner discussion — t1..t9 scattered teaching, idle ignition, ghost forms,
candidate opens across distinct frames, ask fires with pattern=FACT/OWNED +
name=NONE, owner says "fruit", point written with provenance and citations,
then a later taught `pear` routes through `fruit` as INFERENCE/OWNED. This
is the G15 walkthrough trace.

## 22.9 Open decisions introduced

| OD | question |
|----|----------|
| OD-97 | values of `R_min`, `R_ask`, `Δ_min`, `GHOST_MAX` — blocked on sweep artifacts, per standing discipline |
| OD-98 | candidate table: persisted (replayed, hash-in) vs session-only (hash-exempt, declared as such) — decides exact I5 wording |
| OD-99 | ghost budget mechanics: fixed cap vs arousal-scaled (arousal-scaled is BANNED unless the owner explicitly reopens it; listed so nobody "fixes" it silently) |
| OD-100 | placeholder naming, and the rename path (`cluster:…` → owned word): new row vs re-version (I4 says append-only; proposal: new row, old name superseded_at-stamped) |
| OD-101 | the relation kind written for membership edges (reuse `is-a` reversed vs a new kind — new kinds must fit the registry rule, §5.1 / R-12) |
| OD-133 | **does a cluster point count as a "taught link" for the stage ladder?** A confirmation writes an ordinary owner-confirmed `links` row, and §11's S2 criteria count `links` rows kind-blind — so tonight it counts, unchanged, and the build adds no discriminator. The question is whether an owner-nominated cluster should be *allowed* to satisfy a stage criterion that was written for `P_DEF`/same-question teaching. **Fold-discovered** (stated in §23.9 and [docs/11](11-stages.md) §11.3), hence a tail seat outside this doc's allocation. |
| OD-134 | **how G15(c) is actually swept.** `a` is barred from the detector (§22.3), but the detector reads *walk artifacts*, and the walk's settle-set is `a`-modulated by §20.1 — so a full-session `a` sweep can move the candidate table through the walk rather than through the detector, and the leg would red for a reason that is not a cheat. Two readings: (i) **frame-trace-fixed** — feed the detector an identical trace under two `a` regimes and diff its table (tests the ban, which is what the leg is for); (ii) **full-session sweep** — as literally written, which would require an `a`-free walk and contradict §18/§20. **Tonight ships (i)** plus the static dependency check, declares it in §22.11, and leaves the full-session reading `UNMEASURED` next to OD-83's refusal. Also fold-discovered, hence a tail seat. |

## 22.10 What this document does NOT claim

- No change to the six organs, the drive layer's seats, or the audit
  vocabulary.
- No candidate ever answers a question. Until the owner confirms, the
  pattern can only be asked about.
- No claim that crystallize makes SI "intelligent" — it removes one ceiling
  (unnameable recurring structure) and adds zero new authority anywhere.

## 22.11 What the fold changed, and what the build runs at

Five edits, none of them to a law:

1. **Numbering.** The draft's OD-94…OD-98 are **OD-97…OD-101** and its gate
   family is **G15** (`gates/G15/`): B8's sleep walls took **OD-94, OD-95 and
   OD-96** on the same day this round was folded, so the draft's first three
   seats were already occupied, and the fold allocated from the first free seat.
   The live gate plan also already holds **G10 IDLE THOUGHT**, whose G10.b.2,
   G10.e.2 and G10.e.5 are asserts in shipped code, so this round's family is
   appended, never renumbered in place. The full allocation is §23.9 of
   [docs/23](23-context.md).
2. **The registry citation.** "New knowledge types become new shelves / row
   types inside the same schema, never new structures" is docs/05 **§5.1**; §5.2
   is the shelf registry. R-12 was already the right risk row.
3. **The frame scratch did not exist yet.** [docs/16](16-think-walk-hardware.md)
   carries two per-walk states — `h` and `a` (§16.1) — and no frame-scoped
   scratch of any kind, so §22.2's ghost cannot *cite* a law; it needs one. The
   fold therefore added **docs/16 §16.8, the frame-scratch law** (§16.5 is the
   GPU/CPU contract, §16.7 the tuning legs, so the number is the next one
   free): a frame may hold composite handles, every one of them is destroyed at
   settle, none is addressable, none enters the state hash, and over budget the
   frame builds none. Ghosts and
   [docs/25](25-skills.md)'s role-graphs are its two tenants. Without that
   section, "frame-scoped" would be a promise this file alone made.
4. **No new shelf.** `SHELF_LINKS = 6`, `SHELF_SKILLS = 9` and
   `SHELF_EVENTS = 11` are already registered ids, so a `cluster-point` is a new
   **kind** on links — exactly §5.1's sanctioned shape. The round adds zero
   structures.
5. **The co-occur family** cited in §22.1 is docs/07's fourth mechanism,
   "Co-occur adjacency": candidates only, never FACT. The candidate table is its
   sibling, not its owner.

**Declared build points, so nothing about a default is silent.**

- **`CRYSTALLIZE = off | on` is a ruling row** (the `INFERENCE_PROMOTION`
  pattern), shipped **off** at the species default. With `on`, the drive emits
  an ask kind that nine already-green gate families never counted, and that
  choice belongs to OD-97…OD-101's owner, not to the build. `off` is not a
  disabled demo: G15's scenario installs `on` through the ordinary
  launch-constant channel, so the machinery is exercised under test while the
  species default stays untouched — and while every pre-existing gate artifact
  re-runs byte-identical.
- **`GHOST_MAX`, `R_MIN_CRYSTALLIZE`, `R_ASK_CRYSTALLIZE`,
  `DELTA_MIN_CRYSTALLIZE`** (OD-97) are swept **SPECIES** rows at
  `UNMEASURED_PLACEHOLDER` (OD-86), and their `hi` stays the type maximum
  because OD-83's "may `hi` be a design ceiling?" is still unrulled. So
  `gates/G15/recurrence_sweep.json` publishes a **refusal**, like G13's two
  artifacts, and no G15 leg may cite a swept winner. Where a leg needs a value
  to run at all, it runs at the **box's own floor** and labels it a floor.
- **Candidate storage (OD-98): session-only, declared hash-exempt.** The
  hash-in alternative adds a hash item and therefore re-hashes every run
  recorded since B0; a build does not get to spend the whole gate corpus's
  comparability on an unrulled item. Hash-exempt never means invisible: the
  table is written to `crystallize.jsonl` per frame, so an unruled I5 wording
  still leaves an audit trail.
- **OD-100's placeholder name** ships as `cluster:{t_confirm}:{hash(members)}`,
  and no leg treats it as a word — that is G15(f)'s subject, not a convention.
- **OD-101's membership relation kind** ships on the existing generic owned-link
  kind rather than a new registry entry: a new kind is §5.1's door and
  OD-101's own question, so the fold leaves the entry un-made and the edge
  honest.
- **No new provenance kind, and no new envelope byte.** A nomination
  confirmation is written as `prov_kind: answer-to-ask` with `question_id`
  naming the `cluster-nomination` ask row and `prov_speaker: owner` — which
  makes it `gain_kind: demonstrated` by §8.2's own mapping, because the error
  before it is exactly the named empty slot the ask recorded. The registry's
  five `prov_kind` literals ([engineering/01 §4](../engineering/01-data-schemas.md))
  stay five, docs/15 item 3's closed-constant list stays closed, and G5c's
  schema diff stays empty. *What* made the row is carried by the ask row's
  `kind`, not by the belief row's provenance — a sixth literal would be a
  sixth teaching channel, and this round adds none.
- **OD-99 stays banned by construction.** The ghost budget is a fixed cap and
  the detector reads no `a`; the arousal-scaled option is absent from the code,
  not merely zero, so G15(c) is a compile-time fact plus the **frame-trace-fixed
  diff** that OD-134's reading (i) declares — the same shape as the missing
  `si-sleep ↛ si-learn` edge.
- **Two seats the fold discovered after the drafts closed, both numbered from
  the ledger's free tail** rather than inserted into this doc's OD-97…OD-101
  range (renumbering a closed range to hide a late finding is what §23.9 exists
  to prevent): **OD-133**, whether a confirmation-minted `cluster-point` may
  satisfy §11's kind-blind `links` criterion — tonight it may, because adding a
  discriminator would be a stage-law edit wearing a bug fix; and **OD-134**, how
  G15(c) is swept given that `a` legitimately moves the walk whose traces the
  detector reads — tonight reading (i), with the full-session sweep left
  `UNMEASURED`.
