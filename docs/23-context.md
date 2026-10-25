# §23 — CONTEXT: the middle timescale (v2 arc — folded law 2026-10-23; G16, OD-102…OD-109)

> Status: **FOLDED AS LAW 2026-10-23** by the owner's order to engineer the v2
> arc and build it. The HOW is [engineering/12](../engineering/12-v2-arc.md); the gate family
> is **G16** and the open items are **OD-102 … OD-109**.
> The draft's own rule still binds item by item: an unrulled OD item is not a
> licence for the build to choose, so wherever the build could not stop for a
> ruling it ran the **least-authority option**, said so in §23.11, and
> left the ruling with the owner — the OD-95 discipline, never a silent default.
> **No magnitude is quoted in this file**, and no value exists before its sweep
> artifact does.
>
> It extends [22-crystallize.md](22-crystallize.md) and is not fully
> intelligible without it. The module is **silt** (`silt.py` in the logical
> module map, crate `si-silt`).

## 23.0 Thesis line

**The lattice is forever, the frame is a blink, and nothing between them
exists yet.** SI has exactly two timescales: rows that never die (I4) and
scratch that dies at settle (§16, ghosts included, §22.2). What is missing
is the working set: the warm middle timescale that makes a stretch of
conversation cohere as *about something* without any of it being knowledge.
CONTEXT (module name `silt.py`, §23.3) is one session table of per-point
salience that decays on the logical clock: lights deposit, idle ticks
erode, and points fall below threshold in last-touch order, so **context
clears backwards on its own.** Warm during a conversation, cold again after
long idle, and neither state was written by anyone: the lattice never heard
of it.

Litmus: with silt active but the owner silent, **no belief row moves** — silt
is the one mechanism of the arc that writes nothing at all, not even an ask, and
the [docs/03](03-birth-state.md) §3.3 exception is listed there for the others
and simply does not apply here. And after enough quiet ticks the silt table is
empty without any clearing instruction ever executing, only decay.

## 23.1 The gap (what exists, what is missing)

| timescale | exists? | carrier today |
|-----------|---------|---------------|
| one thought (frame) | yes | THINK frame scratch; ghosts live and die here (§22.2) |
| **the current conversation** | **missing** | nothing: each walk, driven or idle, starts from one weighted seed and spreads lattice-wide; the next entry re-derives "what were we doing" from scratch, or does not re-derive it at all |
| forever | yes | the lattice (I4) |

What exists *near* the gap, and why it is not the gap:

- **Ignition recency weight** (R19, §19.2): priming of a *seed*, not of a
  topic. It raises the chance that `apple` fires; it does not make the walk
  preferentially traverse "the apple conversation" once fired.
- **Per-row recency** (`last_touched_at`, [10 §8](../engineering/10-temperament-sleep.md)): a
  row-level timestamp the next traversal reads. A statistic, not a coherent
  working set; nothing aggregates it into "what this session is about".
- **The co-occur family** (§7 item 4): statistics *about the past*, never a
  state *about now*.
- **Hot stamps / bake** (§20.4): per-edge durability. A belief that mattered
  stays findable; it does not stay *in the way*.

Silt is the aggregate over all four: an inspectable table saying which
points the session is currently about, decaying on its own.

## 23.2 Shape of the mechanism (one table, three laws)

Silt is **not a seventh organ** (§2), not a drive, and not a new channel. It
is one component: a session-state table plus a decay law, living on the same
side of every boundary the crystallize detector lives on.

| property | value |
|----------|-------|
| lives in | session state (the R19 snapshot family; which hash items it occupies is OD-103) |
| lifetime | per-tick decay; scope (session-scoped vs cross-session) is OD-109 |
| writable to lattice? | **never**; it is invisible to the write path |
| in state hash? | OD-103 (I5 wording is load-bearing, §23.5) |
| proposes anything? | no; unlike crystallize it has no ask kind, no candidate table, no queue presence at all |

## 23.3 The mechanism (SILT)

**Table.** `silt: {hyperpoint_id → salience}`. Bounded by decay, not by
eviction games: an entry below the floor is dropped; `SILT_MAX` is a
diagnostic bound read out by audit, not a competitive cap (same discipline
as `GHOST_MAX`, §22.2: over bound means the deposit is refused, never
ranked).

**Deposit.** When a hyperpoint lights in a frame (owner-driven or idle),
`silt(p) ← min(S_max, silt(p) + S_bump)`. Frozen rows never deposit (they
may be lit mid-walk; they are the owner's installed knowledge, not
experience; parallel to crystallize's frozen-filler rule, §22.3). Deposit
weight for idle-sourced lights is OD-102: the loop `silt → ignition → light →
silt` is a real feedback path, bounded by decay per tick (sustained warmth
requires sustained re-touch), the idle per-hour budget (§19.4), and OD-90's
proposed refractory if it lands; whether idle deposits come at reduced
weight is the owner's call, made once, as a constant with a sweep artifact.

**Decay.** Per logical tick: `silt(p) ← silt(p) · λ_s`. **Logical ticks
only** (I5): wall-clock decay is banned for the same reason wall-clock is
banned everywhere; a replay must recompute the table from the transcript
byte-identically. Below `θ_silt` an entry contributes exactly zero and is
dropped.

> **`λ_s` is a share of the one curve, not a second τ** (R-29). The design has
> exactly one recency implementation — `Clock::recency` — and a dimming rate of
> its own is forbidden outright, which is the same treatment `DIM_BASE` already
> gets: the arithmetic is `d = λ_s + (255 − λ_s) · Clock::recency_u8(Δt) / 255`,
> so `λ_s` sets *how much of the single curve silt follows*, never *when time
> passes*. `check_one_clock` therefore scans silt's crate as well and must still
> find one implementation
> ([engineering/12 §0 fact 5](../engineering/12-v2-arc.md)). A reader who builds
> `silt · exp(−λ_s·Δt)` has built a second clock and broken I5 in the same motion.

**Clears backwards.** No extra rule: exponential decay makes last-touch
order the fall-out order. The start of a long conversation goes cold before
its tail; enough quiet ticks and the table is empty, at which point silt's
ignition term is identically zero everywhere and the sampler is exactly the
§19 lattice-only sampler again. **Not always cold, not never cold: cold is
what long idle converges to, not the resting state between two lines.**

**What silt may bias (read side only):**

- the ignition candidate weights (a fifth term beside the four of R19; its
  class under OD-87's species/temperament split is OD-108);
- the walk's spread (OD-104: this is the one ruling that touches
  crystallize, §23.4).

**What silt may never do:** write rows, form or bake edges, touch drive
arbitration or the label ladder, scale with `a` (OD-107, the OD-99 pattern:
banned by default so nobody "fixes" it silently), appear in an owner-facing
label, or carry an innate word (§23.5, the I1 leg).

**Naming.** Candidates: **Silt** (sediment that settles while the river
runs and clears when it stops; dry physics register, no emotion words,
passes the R-21 grep); *Solvent* (pairs with crystallize, but over-claims:
silt dissolves nothing); *Residue* (true but reads as leftover junk);
*Warm bank* (reads as a UI term). **Echo is disqualified: the word already
means a specific thing in the docs** (the calm echo line, §4 demo trace).
Pick: **silt**, module `silt.py`, constants `S_max`, `S_bump`, `λ_s`,
`θ_silt`.

## 23.4 The collision with crystallize (the one real interaction)

Crystallize's detector counts co-lighting **across distinct frames** as
content statistics, with `Δ_min` spacing "so one hot afternoon cannot mint a
concept" (§22.3 rule 3). Silt breaks the independence assumption behind
that rule: if silt keeps re-igniting (or re-spreading through) the same
constellation, recurrence is no longer independent sampling of lattice
structure, and `recurrence_count` inflates without new content evidence.
§22.3's exclusion list already bans arousal recency weighting; it says
nothing about session-salience priming, because silt does not exist yet.
The contamination is purely at the **candidate** stage: ghosts are never
evidence, and the shared-filler rule (§22.3 rule 2) means mere adjacency
does not mint anything. But a constellation that *does* share fillers can be
kept warm by silt until it crosses `R_ask` on its own heat.

**Options (ruled as OD-104/OD-105):**

- **(a) ignition-only priming** (recommended for v2): silt biases which
  point fires, never how the walk spreads. Weak contamination remains via
  seed selection, but a silt-chosen seed still walks real lattice edges, so
  what co-lights is what the lattice supports.
- **(b) spread priming + a `primed` tag**: the frame co-light log gains a
  per-light flag (the light's selection read silt above `θ_silt`), and the
  crystallize detector counts **cold frames only** toward recurrence. The
  tag is cheap and G16(e) needs it regardless of the OD-104 outcome.
- **(c) spread priming, detector counts everything**: **not recommended**;
  this re-opens exactly the hot-afternoon hole §22.3 rule 3 exists to
  close.

Recommendation: **(a), with (b)'s tag implemented anyway.** If the owner
ever wants spread priming, the cold-count rule is then already in place.

## 23.5 Invariant impact audit

| invariant | impact | why it holds |
|-----------|--------|--------------|
| I1 | none, one leg to state | silt is installed FORM (a table shape + one decay curve); at birth the table is empty and selects nothing. The leg: **recency is felt, not sayable**. Silt may make SI warm toward `apple`; the word "recently" is only sayable after it is taught, like every word (§3.2's shape-only frame allowlist, whose risk row is R-18;
the demo trace's "back then" precedent is [docs/04](04-demo-trace.md) s19). No innate temporal vocabulary ships with silt. |
| I2 | none | frozen rows never deposit; the table tracks live-layer lights only |
| I3 | none | silt does not touch the channel, and both voices' lines deposit identically (a sense perturbation lights the walk regardless of source) |
| I4 | none | silt writes no rows; decay is not deletion: the rows it tracks are untouched, and a table entry is scratch with a longer lifetime than a frame |
| I5 | **load-bearing** | decay per logical tick only; wall-clock decay banned. Hash-in vs hash-exempt is OD-103; engineering/01 item 6 currently hashes session state, so hash-exempt is the exception that must be declared, not the default |
| I6 | none | silt never appears in a label; audit readouts name rows ("warm toward: apple, tomato"), never time words |
| I7 | none | silt neither detects nor decides; it has no ask kind and no queue presence. Detection ≠ decision is untouched, one level stricter than crystallize |
| I8 | **load-bearing** | deposit and decay never read `a`; arousal-scaled decay is OD-107, banned by default. Audit may LATER print readouts over silt ("keeps returning to the apple rows") with zero causal power, same rules as §18.2 |
| I9 | none | idle walks may deposit (they light points) but write no rows; silt is session state, not a row; the idle feedback path and its guards are §23.3 |
| I10 | one leg to state | deposit-source policy is OD-106. Recommended: full-weight parity for both voices, because silt tracks *thinking*, not teaching, and confers zero authority (no write, no confirm, no bake), so there is no laundering path; any differential must name the specific path it prevents |

**One precedent to name rather than hide:** [10 §8](../engineering/10-temperament-sleep.md) law 1 rules that
P4's calm view feeds nothing, and that a stored number derived from rows may
never feed the ignition sampler (`check_calm_view_is_terminal`, G13(k)).
Silt, if OD-104 passes, is the **first session statistic allowed to feed
the sampler**. That is not a contradiction (the calm view is a *sleep
output*, silt is a live walk artifact, and the law's scope is P4's counts),
but the exception is exactly the kind that must be owner-ruled, not
shipped: OD-104 records it.

## 23.6 Module map (engineering view)

```
silt.py       — the session table + decay law; deposit API called by the
                loop per frame; read API for the ignition sampler and (per
                OD-104) the walk spread. Imports: read-side only. NEVER
                imports write/gate/drive-action; NEVER imported by them.
                (Same reader-only discipline as audit.py and
                crystallize.py, R-24 style.)
walk.py       — per-light log line gains the `primed` flag (OD-105 tag) when
                a light's selection read silt above θ_silt
crystallize.py— unchanged except the detector's cold-frame rule (OD-105)
drive.py      — unchanged; silt is not a drive, has no ask kind, emits nothing
write path    — unchanged; silt is invisible to it
audit.py      — readouts over silt name rows only; zero causal power
```

**Deletion test (mirrors G9(c) / §22.6):** removing `silt.py` must change
no lattice row or edge byte and **no ask output at all** (stricter than
crystallize, which owns one ask kind); the only diffs are ignition
candidate ordering, the walk's spread (if OD-104 allowed it), and log
lines.

## 23.7 Gates — G16 (proposed; see §23.9 for the number)

- **G16(a) determinism:** fixed transcript, silt on, same seed + variant:
  replay is byte-identical (the standard G5 leg, now including the silt
  block per OD-103). Cross-variant comparison stays a category error
  (OD-93).
- **G16(b) deletion:** remove `silt.py`, rerun, diff per §23.6.
- **G16(c) arousal isolation:** sweep `a` across its full range; silt table
  byte-identical (the I8 leg, same shape as G9's and crystallize's).
- **G16(d) clears-backwards:** scripted conversation touching distinct
  points at known ticks, then a long idle stretch: entries fall below
  `θ_silt` strictly in last-touch order (earliest last-touch first); after
  `T_cold` ticks the table is empty and silt's ignition term is identically
  zero.
- **G16(e) crystallize isolation:** same transcript, silt on vs off: the
  crystallize candidate table is byte-identical (via OD-105's cold-count
  regardless of the OD-104 outcome; a primed-frame recurrence that
  increments a candidate is a red).
- **G16(f) no innate time words:** grep the RENDER outputs of the scripted
  session for temporal vocabulary (recent, just, earlier, ago, still):
  absent unless the owner taught those words; silt appears in audit
  readouts only, naming rows, never time.
- **R-21 extension:** the emotion-word grep list is unchanged; `silt.py`
  must contain no emotion vocabulary. Readouts about it live in
  `audit.py` only.

## 23.8 Open decisions introduced

| OD | question |
|----|----------|
| OD-102 | values of `λ_s`, `S_max`, `S_bump`, `θ_silt`, and the idle-deposit weight — blocked on `gates/G16/silt_sweep.json`, per the standing no-value-before-sweep discipline |
| OD-103 | silt table: hash-in session state (replayed, item 6 family) vs hash-exempt declared session state — decides the exact I5 wording |
| OD-104 | what silt may bias: ignition only (recommended) vs ignition + spread; **this ruling is the recorded exception to the calm-view terminality precedent** ([10 §8](../engineering/10-temperament-sleep.md) law 1) |
| OD-105 | crystallize detector treatment of primed frames: per-light `primed` tag + cold-frames-only counting (recommended regardless of OD-104), vs none |
| OD-106 | deposit source policy: owner/grimm full-weight parity (recommended) vs a differential; a differential must name the laundering path it prevents |
| OD-107 | arousal-scaled decay: **BANNED unless the owner explicitly reopens it**; listed so nobody "fixes" it silently (the OD-99 pattern) |
| OD-108 | the silt ignition weight's class under OD-87's split: species constant vs temperament draw; if temperament, the sweep characterizes, never tunes ("no sweep may ever touch a temperament row") |
| OD-109 | scope: session-scoped (clears at entry boundary) vs cross-session persisted with tick decay (the "not always cold" reading; requires hash-in per OD-103 for replay to reproduce it) |

## 23.9 Fold note: the numbering the fold actually used

This draft proposed an allocation against a ledger that has since moved: B8's
sleep walls took **OD-94, OD-95 and OD-96** on 2026-10-23, so crystallize's five
items could not land at OD-94…OD-98. The fold allocated from the first free seat
and rewrote every cross-reference in one mechanical pass:

| doc | draft ids | **folded ids** | gate family | artifact dir |
|-----|-----------|----------------|-------------|--------------|
| [22](22-crystallize.md) crystallize | OD-94…OD-98 | **OD-97…OD-101** | **G15** | `gates/G15/` |
| 23 this doc, silt | OD-99…OD-106 | **OD-102…OD-109** | **G16** | `gates/G16/` |
| [24](24-episodic.md) episodic | OD-107…OD-112 | **OD-110…OD-115** | **G17** | `gates/G17/` |
| [25](25-skills.md) skills | OD-113…OD-119 | **OD-116…OD-122** | **G18** | `gates/G18/` |
| [26](26-world.md) world | OD-120…OD-129 | **OD-123…OD-132** | **G19** | `gates/G19/` |

Two things the fold did **not** touch: the live ledger's own seats — this file
cites OD-87, OD-90 and OD-93 exactly as the B7 batch ruled them, and OD-83,
OD-94, OD-95, OD-96 as B5's and B8's still-open walls — and G1…G14. G15…G19 are
**appended**, the way G13 and G14 were.

The count debt grows with them, named rather than papered over: [docs/13](13-gate-plan.md)
now carries the arc in **§13.13**, which makes it **seventeen** families of law
text while [engineering/05](../engineering/05-gate-harness.md) specifies
**nineteen** — G13 SLEEP and G14 TEMPERAMENT PANEL never received law sections,
and [docs/15](15-definition-of-done.md) still opens with the word "twelve" above
its own correction note. That is OD-84's class of debt, deliberately unpaid;
renumbering docs/13 is the owner's call and not a build's.

**Seats outside the allocation, and why they sit at the tail.** The fold's audit
found two seams none of the five drafts owned. (1) The stage ladder: a
crystallize confirmation writes an ordinary owner-confirmed `links` row, and §11's
S2 criteria count `links` rows without asking their kind — **OD-133**. (2) The
shape of G15's arousal-isolation sweep: `a` is barred from the detector, yet `a`
lawfully moves the walk whose traces the detector reads — **OD-134**. Both are
numbered from the free tail of the ledger rather than inserted into this table,
because re-opening a per-doc range to hide a late discovery is exactly the
renumbering churn §23.9 exists to prevent. They live in
[docs/22](22-crystallize.md) §22.9 and §22.11, and OD-133 is also stated in
[docs/11](11-stages.md) §11.3.

## 23.10 What this document does NOT claim


- No seventh organ, no drive seat, no ask kind, no channel change.
- No claim that silt makes SI "context-aware": it makes the walk warm, not
  the words. An SI that is silt-heavy toward `apple` and is asked "what are
  you thinking about?" can honestly enumerate the lit rows, and "recently"
  is only sayable once taught (I1 leg, G16(f)).
- **No mutation step.** Silt is not self-evolution and does not propose
  anything (it is one level below crystallize: crystallize nominates, silt
  merely warms). The forward note, stated once and honestly: a future
  owner-gated search over lattice edits would need exactly this kind of
  prior store to make candidate edits plausible instead of random, and that
  round would need its own invariants and its own owner gates; nothing here
  builds toward it beyond providing the substrate it would read. The owner
  remains the only teacher (I3, I7), and silt never touches the write path.

## 23.11 What the fold changed, and what the build runs at

Four edits, none of them to a law:

1. **Numbering**, per §23.9: OD-99…OD-106 → **OD-102…OD-109**. The cross-doc
   pointer to crystallize's arousal-scaling ban is now **OD-99** (that draft's
   OD-38) and the silt-bias question is **OD-104**.
2. **"§14's frame law" was a wrong address.** docs/14 is the risk register and
   holds no frame law; the frame law is §3.2's shape-only allowlist, whose risk
   row is R-18. The "back then" precedent is real and lives at
   [docs/04](04-demo-trace.md) s19.
3. **The four ignition weights verify as drafted** — recency, edge bake, decay
   deficit, live contradiction — and in the build all four are **TEMPERAMENT**
   draws. That is why OD-108 is not a cosmetic question: a fifth *draw*
   lengthens `DERIV_ORDER` from 18 to 19, bumps `seed_format`, and changes every
   hash in the repository. See the declared point below.
4. **The calm-view precedent is stated with both ids.** engineering/10 §8 law 1
   is the law, `check_calm_view_is_terminal` is the static leg, and the leg is
   documented as **G13(k)** but recorded in `gate_report.json` as **G13.k.2**.
   The exception OD-104 bends has to be nameable.

**Declared build points.**

- **OD-104 (bias scope): ignition only — the draft's own option (a) — with
  option (b)'s `primed` tag implemented anyway**, because the tag is what makes
  G16(e) gradeable whichever way the owner rules on spread priming.
- **OD-108: the fifth term is a SPECIES row and it is unarmed.**
  `IGN_WEIGHT_SILT` exists in the registry with live weight **0 at the species
  default**, which keeps every pre-existing gate byte-identical: an individual's
  fifth sampler weight is the 19th draw, and 19 draws is a `seed_format` bump
  that silently invalidates every run recorded since B0. OD-108 stays open; the
  ruling is one cell in [engineering/03](../engineering/03-constants-registry.md)
  plus one bump if the owner wants the draw. G16's scenario installs the smallest
  non-zero step the row's grid can express, so the term is *seen* biasing the
  sampler at a declared point and never at an invented magnitude.
- **OD-103: hash-exempt session state, declared.** The table is logged to
  `silt.jsonl` per tick, so the I5 wording being open still leaves an audit
  trail — B8's OD-96 lesson, stated on the row where it belongs.
- **OD-102 (values)**: `LAMBDA_SILT`, `SILT_LEVEL_MAX`, `SILT_BUMP`,
  `THETA_SILT`, `SILT_MAX` are swept SPECIES rows at `UNMEASURED_PLACEHOLDER`;
  where a leg needs one, it runs at the box's own floor and names it a floor.
  `gates/G16/silt_sweep.json` publishes a refusal under OD-83.
- **OD-107 is not bent at all**: arousal-scaled deposit and decay are **absent
  from the code**, not merely zero, so the isolation leg is a compile-time fact
  plus a sweep — the same shape as the missing `si-sleep ↛ si-learn` edge.
- **OD-109: session-scoped, keyed by conversation.** The key is per-conversation
  from the first build so that [docs/26](26-world.md)'s per-thread option
  (OD-127) is a keying decision and not a new mechanism; the default key is the
  single conversation every pre-existing scenario already is, which is why
  nothing already built notices.
