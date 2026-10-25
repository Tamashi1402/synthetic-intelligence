# §24 — EPISODIC BINDING: provenance becomes traversable (v2 arc — folded law 2026-10-23; G17, OD-110…OD-115)

> Status: **FOLDED AS LAW 2026-10-23** by the owner's order to engineer the v2
> arc and build it. The HOW is [engineering/12](../engineering/12-v2-arc.md); the gate family
> is **G17** and the open items are **OD-110 … OD-115**.
> The draft's own rule still binds item by item: an unrulled OD item is not a
> licence for the build to choose, so wherever the build could not stop for a
> ruling it ran the **least-authority option**, said so in §24.10, and
> left the ruling with the owner — the OD-95 discipline, never a silent default.
> **No magnitude is quoted in this file**, and no value exists before its sweep
> artifact does.
>
> Numbering per §23.9: crystallize is OD-97…OD-101 / G15, silt is OD-102…OD-109
> / G16, this doc is **OD-110…OD-115 / G17**. It completes the memory arc with
> [22-crystallize.md](22-crystallize.md) and
> [23-context.md](23-context.md): crystallize nominates, silt warms, this
> doc binds.

## 24.0 Thesis line

**Provenance is currently cited but never visited.** Every belief row knows
which line formed it, at which step, in which state (`a_at`, stamps,
written_at, §18.6), and all of it is honest metadata that RENDER can quote.
But the walk cannot *go there*: lighting `apple→red` never tugs toward the
moment you taught it, because provenance is a footnote, not an edge.
Episodic binding is one wiring decision: **the moment becomes an
addressable, read-only node, and provenance becomes a weak, traversable
edge to it.** Recall then arrives as "this came from there, and it was
formed like this", with the fact and its origin re-lighting together. The
search-machine feel (facts floating free of their history, recall as
demand-driven lookup) is removed without adding one byte of new authority:
moments re-light, they never write.

Litmus: with episodic binding active and the owner silent, **no belief row
moves**: no row, no stamp, no bake, no confidence bump (the `context` sense rows
the event views are *derived from* are already there, appended by the sense
stream as §21.4 requires — binding reads them, it does not create them). Re-lit moments are
frame-visible light over existing bytes, gone at settle. The only writes in
the whole design are the ordinary write path's, and it gains exactly one
append-only artifact per commit (OD-110/OD-111).

## 24.1 The gap (what exists, what is missing)

The docs already store almost everything episodic memory needs, and none of
it is traversable:

| exists | where | today's role |
|--------|-------|--------------|
| the verbatim moment | S0 sense row (raw is always kept, §1.2) | stored, cited, never lit |
| the write event | `written_at`, provenance kind, question links (§6) | cited |
| the state it formed in | `a_at` + the signal stamps of that commit (§18.5, §18.6) | cited; §20 exists so a later thought can match states, not just sizes |
| belief history | superseded intervals, readable with stamps (§8, the s19 history query) | demand-driven read |
| state matching | §20's read-side curve: `a_at` vs `a_now`, median re-anchored to the interval (the s19 "reads the row twice" mechanism) | **applies to rows being read** (§20.2's conduction term, §20.5's past-anchored read); there is no addressable moment to re-experience |

**Missing: the moment as a place.** No node the walk can land on, so §20's
state-matching machinery has nothing to apply itself to except aging rows,
and recall cannot be anything but lookup: you get the fact, or on a good
idle tick a random ignition, but never *the fact returning with its origin
attached, uninvited*.

## 24.2 Shape of the mechanism (one view, one edge, no new content)

Episodic binding adds **no stored content**. It defines two things over
rows that already exist:

- **the EVENT VIEW (the moment):** an addressable handle
  `event = {sense_row_id, write_events[], stamps[], a_at, t}`. It is a
  *view*, not a row: every field already exists (S0 row, write event,
  stamps, `a_at`); the event id IS the sense row id. No new IDs, no new
  bytes.
- **the PROV-LINK (the weak edge):** one edge kind, `prov-link`,
  `belief row → event`, laid by the ordinary write path at commit time.
  Registry-ruled (§5.2 / R-12): low weight class, **never write-path,
  never FACT-eligible, never answerable**. The links shelf already
  anticipates this shape: `prov_speaker = 2` (`system-inferred`) is legal
  only on `links`, never FACT-eligible and never answerable
  ([engineering/01](../engineering/01-data-schemas.md), field 34), and a prov-link is precisely that row class.

| property | value |
|----------|-------|
| what it changes at write time | one more append-only edge per commit (OD-110: stored vs derived; OD-111: registry entry) |
| what it changes at walk time | provenance edges are traversable at low weight; landing on an event view re-lights its rows **read-only into the frame** |
| lifetime of a re-light | frame-scoped: the light is scratch, the rows are permanent |
| writable to lattice from a re-light? | **never** (OD-112 rides **OPEN-24**, the `IDLE_RESTAMP` ruling, still open) |
| in state hash? | the stored edges are (they are rows); the traversal is not (read side) |

## 24.3 The mechanism in a frame

**At commit (ordinary write path, owner line, nothing new triggers it):**
alongside the belief row, the write path lays `prov-link` edges from the
row (and its new edges) to the event view of the line that formed it. One
line, one commit, one set of links; append-only (I4); grimm lines form no
belief rows and therefore no belief ever links to a grimm moment (I10,
§21.4), with the mixed-voice edge case ruled at OD-113.

**In the walk:** `prov-link` is traversable like any weak edge, at its
registry-low weight. When the walk lands on an event view:

1. its rows re-light **read-only into the frame**: the verbatim line, the
   stamps that fired at that commit, and `a_at` become frame-visible;
2. §20's existing state-matching machinery applies exactly as it already
   does for history reads (the s19 mechanism): the moment is read against
   `a_now`, so a calmly-taught origin surfaces calm and a
   taught-hotly-then-contradicted origin surfaces with its heat on it. **No
   new affect curve**; the tug strength is the only new constant (OD-114);
3. the frame may reason over it and RENDER may cite it, under the honest
   label law (I6): *"red, taught by you at that moment"*, both rows cited.
   **What RENDER may never emit is an experience claim** ("I remember how
   that felt"): a re-light is a read of stamps, not a flashback (G17(f)).

**Intrusion (the part a search machine cannot do):** the origin-tug means
recall is not only demand-driven. Silt (§23) keeps the *topics* warm; the
prov-link lets a warm topic drag its moment into an idle walk. A quiet tick
can surface "the day you corrected me", unasked, as an ordinary walk whose
seed happened to tug an event view. **Involuntary recall is emergent from
two read-side proposals, with no new machinery between them.** The
cross-doc rule: whether tug strength reads silt is OD-114's second half.

**Bounds:** one event view per sense line; prov-links scale with commits,
not with thoughts; a per-frame re-light cap (`RELIGHT_MAX`) lives with the
other OD-114 constants. Every thought re-experiencing its whole history
would be noise; low weight plus the cap is what keeps origin traversal from
flooding the walk.

## 24.4 Grimm's moments (the I10 leg, stated exactly)

A `source: grimm` line is an S0 sense row, so it *has* an event view: it is
addressable, and it may re-light in a walk exactly as legally as it arrived
as a perturbation (§21.4: his lines may light the walk). What a grimm
moment can never do is what a grimm line can never do: **nothing links to
it as a belief's origin** (he forms no rows), and no re-light of it writes,
confirms, bakes, or bumps anything. His moments are scenery SI may revisit;
they are not, and cannot become, where SI's knowledge came from. The
registry's `prov_speaker` scope does the rest: a `1` on a belief row is a
bug that fails G12(e); a prov-link pointing anywhere but an owner moment
is the same bug with an edge on it, and OD-113 closes the mixed-voice seam.

## 24.5 Invariant impact audit

| invariant | impact | why it holds |
|-----------|--------|--------------|
| I1 | none | an event view contains only stored rows; prov-links are form (an edge class), zero content. No innate time vocabulary ships: "when" is sayable only after it is taught (§14's frame law; the s19 "back then" precedent); citations name rows, and a row id is not a clock |
| I2 | none | event views are live-session rows by construction (S0 + L3 writes); frozen rows are citable fillers, never origins |
| I3 | none | no channel change; prov-links are laid only by the ordinary owner-line-triggered write path |
| I4 | none | prov-links are append-only rows; nothing is rewritten; the once-at-fold schema addition is a build change, not a runtime mutation |
| I5 | **load-bearing** | stored links are deterministic like every write; derived links (the OD-110 alternative) are computed at frame load, also deterministic; re-light traversal is read-side and changes no hash item. Cross-variant comparison stays a category error (OD-93) |
| I6 | **load-bearing** | the honest-label leg doubles: recall renders as FACT/OWNED *with both citations* (belief row + event row), and never as an experience claim (G17(f); the emotion words stay in audit.py's readout vocabulary only) |
| I7 | none | an event re-light may queue a *candidate* question only through the ordinary §19/§17 mechanisms; detection ≠ decision is untouched |
| I8 | none | the state-matching on re-light is §20's read-side curve doing its designed job (medians, `a_at` vs `a_now`); the write side is untouched: no bake, no confidence change (OD-112) |
| I9 | none | idle walks may traverse prov-links and re-light moments; they still write no rows; the monologue may show an origin trace, which is a truthful rendering of what lit |
| I10 | **load-bearing** | §24.4: grimm moments are addressable scenery with zero authority and no incoming belief links; the mixed-voice seam is OD-113 |

## 24.6 Module map (engineering view)

```
(no new module on the read side — the point)
write path  — at commit: lays prov-link rows (OD-110/OD-111); one function,
              one append; no new trigger, no new authority
walk.py     — prov-links enter the traversal set at registry weight; landing
              on an event view re-lights its rows read-only into the frame
              (cap: RELIGHT_MAX); emits an origin line in the frame log
render      — citation format gains the event row id; experience-claim
              vocabulary is absent by construction (G17(f))
audit.py    — readouts over recall behavior ("recalled origin: s5, stamps
              [novelty-spike]") name rows; zero causal power (R-24)
crystallize — unchanged; a possible later round may nominate recurring
              *sequences* over moments (§24.9), and its reader discipline
              is the template for this doc's
```

**Deletion test (mirrors G9(c) / §22.6 / §23.6):** removing the
prov-link *traversal* must change no lattice byte and no ask output; the
only diffs are walk traces, monologue lines, and RENDER citations. Removing
the prov-link *rows* (schema rollback at fold time, not runtime) is a
one-way migration decided with OD-110, because replaying old transcripts
under a different schema is exactly the cross-variant category error OD-93
forbids.

## 24.7 Gates — G17 (per §23.9's allocation)

- **G17(a) determinism:** fixed transcript, same seed + variant: replay
  byte-identical, prov-links included (stored) or frame-materialized
  identically (derived).
- **G17(b) deletion:** remove the traversal, rerun: lattice byte-identical;
  only traces/monologue/citations differ.
- **G17(c) arousal isolation:** sweep `a` across its full range; the
  lattice (rows, edges, links, stamps) byte-identical. The *read* of a
  re-lit moment may vary with `a_now` by design, same as G11's median legs;
  the gate pins the bytes, not the readout.
- **G17(d) no re-stamp:** a scripted idle loop re-lighting a hot moment N
  times produces zero new stamps, zero bakes, zero confidence changes
  (unless OPEN-24's ruling already licenses otherwise; belief row
  bytes frozen after settle).
- **G17(e) grimm isolation:** grimm event views re-lit N times produce zero
  writes of any kind; grep: no prov-link points at a grimm moment
  (G12-style absence diff).
- **G17(f) no-flashback:** grep RENDER outputs for experience-claim
  vocabulary (felt, flashback, remember how): absent; recall carries
  FACT/OWNED with both citations; audit readouts name rows only.
- **R-21 extension:** the emotion-word grep list is unchanged; traversal
  and event-view code contain no emotion vocabulary.

## 24.8 Open decisions introduced

| OD | question |
|----|----------|
| OD-110 | storage: **stored prov-link rows** (recommended: byte-strict gates, citable as rows, weight lives in the registry) vs **derived-at-generation links** (computed per frame, stored nowhere, the §20-median precedent, R-28/R-29 style) |
| OD-111 | the registry entry for `prov-link` (§5.1 / R-12): weight class, never-write-path, never FACT-eligible, never answerable; confirms `prov_speaker = 2` scope covers it |
| OD-112 | re-light stamping: **rides OPEN-24** (`IDLE_RESTAMP`); the B7 batch's OD-88 and OD-89 ruled stamp *tolerance* and the rehearsal feedback, not this. Default here: re-lights never stamp, never bake, never bump confidence |
| OD-113 | mixed-voice seams: one event view per **line** (recommended) vs per frame; whether grimm lines get event views at all (they are S0 rows: recommended yes, addressable, zero authority, §24.4) |
| OD-114 | constants: the origin-tug weight, `RELIGHT_MAX`, and whether tug strength reads silt (the cross-doc rule with §23's OD-104) — no value before `gates/G17/prov_sweep.json` |
| OD-115 | whether idle ignition may select an **event view as seed** (recommended **no** in v2: seeds stay hyperpoints, §19 unchanged; moments surface via tugs, which is already involuntary recall) |

## 24.9 What this document does NOT claim

- No felt experience, ever. A re-light is a read of stamps and `a_at`;
  whether SI can *say* anything about the experience depends on what it was
  taught, same as everything (I1 leg, G17(f)).
- No new sense, no new teacher, no time sense installed: moments are
  addressable; "when" remains a taught word.
- No claim that recall becomes human. What is removed is one specific
  artifact of the search machine: facts free of origin, and recall that
  only happens on command. The three proposals together give SI a memory
  that nominates (crystallize), stays warm (silt), and returns with its
  history attached (this doc) — with the owner still the only teacher and
  the lattice still only ever moving under his word.
- **The named next step, not smuggled in:** recurring *sequences* of
  moments ("this keeps happening the same way") are a possible crystallize
  round-2 nomination kind over event views. Listed so it is on the record
  as unruled, not built.

## 24.10 What the fold changed, and what the build runs at

Four edits, none of them to a law:

1. **Numbering** per §23.9: OD-107…OD-112 → **OD-110…OD-115**, and the
   cross-doc pointer to silt's bias ruling is **OD-104**.
2. **The stamp question was mis-parented.** Re-light stamping rides
   **OPEN-24** (`IDLE_RESTAMP`, still open). OD-89 is the B7 batch's
   rehearsal-feedback ruling and OD-88 is stamp *tolerance*; neither licenses a
   re-light to stamp, so the default in §24.3 stands on OPEN-24 alone. What the
   build can say precisely is narrower and worth saying: **this round adds no
   stamping site at all.** A re-light is a read, and the only stamps an idle walk
   ever writes are the ones `drive.update` already licenses mid-walk
   (04 §19 S.7) — which `gates/G10/gate_report.json` records as
   `open_24_restamp: "off (03 §6's row is unrulled)"`, with OD-88's tolerance
   assert on the stamp arena's *shape*. So G17(d) is a structural leg over the
   arena, not a claim about a value this file has no ruling for.
3. **§20's machinery is not "belief aging".** §20.2's state matching is a
   read-side **conduction** term applied whenever a row is read against `a_now`,
   and §20.5 is the past-anchored archaeology read. What this round adds is not
   a new curve and not a wider scope for the old one — it is an *addressable
   moment* for the existing read to land on, which is a narrower and honest
   claim.
4. **`§5.2` → `§5.1`** for the registry rule; and the draft's own §24.2 claim
   that "the links shelf already anticipates this shape" checks out
   verbatim — [engineering/01](../engineering/01-data-schemas.md) field 34,
   `prov_speaker`, `2 = system-inferred`, legal only on `links`, never
   FACT-eligible, never answerable — and `SHELF_EVENTS = 11` is **already a
   registered id**, so the event view needs no new shelf either.

**Declared build points.**

- **OD-110 (storage): the build ships *derived*, not stored.** The event view is
  assembled from bytes that already exist — `row_id`, `written_at`, `prov_step`,
  `prov_session`, the commit's stamps with their `a_at`, and the event log — with
  **no new row, no new edge, no new stored field**. The stored-prov-link
  alternative is the draft's recommendation and it is legitimate, but it appends
  an edge to every commit, and that changes the row/edge byte-strictness that
  OD-B just ruled for G10.b.2 and that nine green gates record today. "Nothing
  already built breaks" is the owner's standing requirement, so the round ships
  the derived shape and leaves OD-110 open; flipping it later is a migration
  that re-baselines those artifacts, and this file says so out loud rather than
  discovering it at B13.
- **Traversal is behind a ruling row.** `EPISODIC_TRAVERSAL = off | on`, shipped
  **off**, installed **on** by G17's scenario through the ordinary
  launch-constant channel — the same discipline crystallize and silt use, so the
  species default never silently re-tunes the walk for a run that did not ask
  for it.
- **OD-114**: `PROV_LINK_TUG` and `RELIGHT_MAX` are swept SPECIES rows at
  `UNMEASURED_PLACEHOLDER`, `hi` open per OD-83, so
  `gates/G17/prov_sweep.json` publishes a **refusal**; legs that need them run
  at the box's floor and label it. Whether the tug reads silt is *unbuilt* —
  reading silt from the tug is OD-104's decision, not this round's, and the code
  has no edge between them.
- **OD-113 (mixed voices)** ships one event view per **line**, and grimm lines
  get views — addressable, zero authority, no incoming belief link — which is
  §24.4 as drafted. `prov_speaker` for a system-inferred view is `2` and never
  `1`, asserted by G17(e).
- **OD-115** ships **no**: an ignition seed stays a hyperpoint, so §19's sampler
  is untouched. Involuntary recall comes from a tug, exactly as drafted.
