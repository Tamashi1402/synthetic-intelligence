# 06 — Resource Budget: sizes derived from the schema, and the measurements this file may not make

**Rules of this file, stated before any number.**

1. Every quantity below is exactly one of three things: (a) a **schema constant**
   from [01](01-data-schemas.md) (byte widths, grid sizes, enum counts), (b)
   **closed-form arithmetic** on those constants, or (c) an **owner-supplied
   design target** quoted with its source and marked as such. Nothing else
   appears here.
2. **Derived ≠ measured.** "112 B × 10⁴ rows = 1.12 MB of envelopes" is
   arithmetic a reviewer can check without a build. "SI runs a 10⁴-row session in
   40 ms / 900 MB" is a measurement, and this file does not make one — not once,
   not as an estimate, not as a placeholder. Every measured quantity lives in
   `gates/G6_*/gate_report.json` (§10) and reads **UNMEASURED** here until that
   directory exists ([docs/13](../docs/13-gate-plan.md)'s rule,
   [gates/README.md](../gates/README.md)'s rule).
3. No wall-clock, no FPS, no device capacity, no "fits". The loop's own budget
   language is **logical ticks** (I5); wall-clock may be *recorded* by a run as
   telemetry and is still not a number this spec quotes.
4. Where growth is uncomfortable, the action is **R-12's alarm** (report,
   redesign the registry entry), never a sixth structure, never a side channel,
   never a cache of something the design calls derived.

---

## 1. Static geometry — what one thing costs

| object | bytes | where it comes from | assert |
|--------|-------|---------------------|--------|
| live belief/context record | **112** = 80 envelope + 32 body | [01 §4](01-data-schemas.md) | `len % 4 == 0`, reserved `u16` + body `u32` zero |
| `z` edge record | **16** | [01 §5.1](01-data-schemas.md) | `pad u16` zero |
| charge stamp | **16** | [01 §5.3](01-data-schemas.md) | `pad u16` + `pad2 u8` zero |
| rehearsal/decay pair (per edge) | **8** | [01 §8](01-data-schemas.md) | curve *value* is never stored (R-29) |
| one `a` trajectory point | **1** | [01 §8](01-data-schemas.md) | the round it applied at rides beside it |
| session core block (`SIck`/`SIst` header) | **36** fixed, never grows | [01 §13](01-data-schemas.md) | `% 4 == 0`; `pad`/`pad2` asserted zero; the individual is stored as a **digest**, so the block prices individuality and sleep bookkeeping together (§3.4) |
| `(t,x,y,z)` address identity | **12** | [01 §1](01-data-schemas.md) | not a stored row: the axes live inside the envelope |
| `hyperpoint_id` (content address) | **8** | [01 §1](01-data-schemas.md) | `md5` prefix, identity not security (**OD-49**) |
| activation on the grid | **4** (`i32` Q12, 1.0 = 4096) | [01 §3](01-data-schemas.md) | never a float in storage or hash |
| shelf-registry entry | **20** (18 B of named fields + padding) | [01 §2](01-data-schemas.md) | eleven entries, three reserved-empty |
| interned string | **4** (handle) + payload, padded to 4 | [01 §7](01-data-schemas.md) | `0` = `""`, `0xFFFF_FFFF` = NULL |
| container framing (`SIlat`/`SIgen`/`SIck`/`SIst`) | **12** + header JSON + payload | [01 §9](01-data-schemas.md) | magic, never extension |
| one `.siro` frame (transport, not state) | **16** fixed + payload | [01 §9](01-data-schemas.md) | append-only; the loop never blocks on a reader (**OD-48**) |

**The three arenas.** L3 lives in three append-only byte arenas — records (112 B),
edges (16 B), stamps (16 B) — plus the interner. Every `*_off` column in
[01 §4.1](01-data-schemas.md) is a `u32` **byte offset** into its arena, which
fixes the ceilings in §8. Nothing in this design indexes: no B-tree, no hashmap
of row ids on disk, no mmap'd secondary structure that could drift from the rows
it claims to index. The `(t, x, y)` address *is* the index ([docs/05](../docs/05-substrate-tesseract.md)),
and the one build-time index (`si-store`'s `(x, y)` / `z` adjacency map for the
genome) is a derived artifact, rebuildable, and off the hashed path (**OD-50**'s
`rayon` stays in the build tools).

---

## 2. The unit of growth: one teaching event

The loop's only size-increasing act is a commit. Everything else is either a
read, a log line, or an in-place bump on bytes already allocated.

| event | new state bytes | formula | notes |
|-------|-----------------|---------|-------|
| any sense input (owner **or** admitted grimm) | `112 + 4 + ‖surface‖₄` | context record + its interned raw surface | **the floor every input pays**; `charge_off = 0` and `edges_off = 0` asserted ([01 §6](01-data-schemas.md)) |
| committed belief row (`/correct`, answer-to-ask, `/teach`, statement) | `112 + 16·E + 16·S + 8 + T + I` | record + `E` edges + `S` stamps + rehearsal pair + `T` trajectory bytes + `I` new interned bytes | `E ≥ 1` (the primary `z[0]`), `S ≥ 1` (§16 of [04](04-algorithms.md) stamps at the commit instant) |
| **boring repeat** (`surprise = 0`) | **0 new bytes** | in-place, FLUID only: `repeats +4`, `last_confirmed_at +4`, `confidence +2`, `gain_exact +2`, `weight +2` = 14 B rewritten inside an existing record and its edge | §6.2's "write nothing" is *append*-nothing; the file does not grow. `source: grimm` ⇒ even these do not move (**R-35**) |
| supersession | `112 + 16·E′ + 16·S′` (the winner) + 8 B rewritten in the loser | new row + its edges/stamps; loser keeps `bake` and stamps, `weight → 0` | I4: aging, not deletion. The loser's bytes stay hashed forever |
| rejected emission at the seam | **0 state bytes** | one line in `G12/speak_gate_log.jsonl` | the filter is upstream of `sense.py` precisely so this row is arithmetic-zero (§21.2, [04 §23](04-algorithms.md)) |
| idle cascade (ignition + monologue) | **0 belief bytes** | monologue frames ride the *separate* lane; `IDLE_RESTAMP` **on** would add 16 B per re-light (**OPEN-24**) | I9 / R-25; G10(b) asserts the bake fields specifically, and **G11(h) asserts it for a composed two-hop answer too** |
| composed inference (`warm` reached through two taught edges) | **0** | the composite is a trace, a label and a citation — never a row ([04 §5.4](04-algorithms.md)) | the read-only law is also the cheapest law in this file |
| **a sleep** (P1→P4, [10 §7](10-temperament-sleep.md)) | **0 new bytes** | in-place FLUID only: `weight` (2 B) per edge P3 visits, the rehearsal pair (8 B) per edge P1 re-lights, and 17 mutable bytes inside the block (§3.4) | **the size-zero event that still costs the most compute**: no row, no stamp, no `bake`, no `confidence` write (**G13(b)/(i)/(j)**); the sleep's own record is one line in `runs/<session>/sleep.jsonl`, which is a log (§9), not state |
| a **different individual** (same build, same transcript, another `--seed`) | **0** | the block's 36 B is already counted: the seed allocates nothing, interns nothing, adds no shelf | a seed is arithmetic selection, not content — two seeds write the same *shape* of state and different values (**G14(d).2**'s honest weakening; **G14(a)**'s birth litmus is the 0-byte claim checked at `t = 0`) |
| RE-PARSE derived fact | `112 + 16·E + 16·S_inherited` | `bake = 0`, inherited stamps `src = 3` | G7's two asserted field values, at a known byte cost |

**The mutability split, stated once as a partition.** This is the question a byte
budget cannot dodge: which columns may a later event rewrite without allocating?
Three classes, and the class is a property of the *column*, not of the shelf.

| class | columns | rule | why the class exists |
|-------|---------|------|----------------------|
| **STRUCTURE** — append-only, never rewritten | row identity `(t, x, y)`; `edges_off`, `edges_len` (**a row's edge set is fixed at birth** — growth happens by a new row or a supersession, never by extending a list in place); every arena record's identity; `version_tag` | new bytes, or nothing | I4; and it is what makes the arena offsets in a replay reproducible |
| **FLUID** — mutable in place, same address | `weight` (u16), `confidence`, `repeats`, `last_confirmed_at`, `gain_exact`, `last_touched_at` (the rehearsal pair) | rewritten in place, 0 new bytes | docs/05 §5.3's own words: "each successful serve +Δ", "supersession sets the aged edge's live weight to 0 … history walks can **re-weight** it at its interval". A design that forbids this column from moving would forbid §8.4 |
| **PROVENANCE** — write-once (or single-set), permanent | `bake`, every charge stamp, `a_at`, `magnitude`, `signal`, `thought_id`, `signal_ref`, `prov_kind`, `prov_speaker`, `text_ref`, `derived_off`, `superseded_at` / `superseded_by` (set once, at the aging instant, never again), `gain_kind` (one-way promotion) | never recomputed; a second write is a bug a gate hunts | §5.3 law (c) and OD-27/OD-33: the bake is a birth property; R-23's realized shape is a **re-read** |

**What that partition buys, and why it is in a budget file:** it is the exact
condition a consolidation or re-weighting pass must satisfy to be lawful. Any
later pass that moves numbers may touch **FLUID and nothing else** — never
structure, never provenance — so "rescale" and "append a new belief" are different
operations at the byte level, not different wordings of the same one. **Sleep is
that clause's first real consumer, and the partition was written before it
existed**: P3 is precisely the "re-weight at its interval" docs/05 §5.3 already
licensed, `weight` plus the rehearsal pair are the whole of its write set, and the
closed list below did not need one new column when the organ arrived — which is
the byte-level form of **G13(j)**. The one thing the batch *could* have added and
did not is a per-edge floor: the floors are **registry constants**, not columns
([10 §6](10-temperament-sleep.md) D.2), so a dimmed edge costs 0 extra bytes and
`si-sleep` never needs a schema change. The frozen base sits outside all three
classes: its records have **no** FLUID column at run time
([07 §9](07-genome-build-plan.md)), which is why a genome edge is immutable
in the only sense the design cares about — and **G13(e)** is the leg that checks
sleep never found that claim inconvenient.

**Mutable-in-place set, closed.** `weight`, `confidence`, `repeats`,
`last_confirmed_at`, `gain_exact`, `last_touched_at` — and `gain_kind`'s
one-way promotion plus `superseded_at`/`superseded_by`'s single aging write.
That is the whole list. Everything else is write-once, and `questions` changes
`status` by a **new version row** ([01 §2](01-data-schemas.md)) rather than a
mutation — which is why the byte cost of a curiosity loop turn is a row, not a
field.

**Sleep's permission is a named subset of that closed list:** `weight`, written by
P3 alone, and `last_touched_at`, written by P1's re-light (plus OPEN-24's re-stamp
if it ever lands). It is *not* `repeats`, `confidence` or `gain_exact`, and that
exclusion is the batch's most attractive bug: a replay pass that "reinforced what
it rehearsed" would inflate a belief by dreaming about it, which is
**G13(i).1**'s zero-movement assertion and I9 in sleep's clothes.

---

## 3. Session arithmetic — three scenarios, all derived

Let `n` = inputs, `k` = teaching events, `E_i`, `S_i` = edges/stamps of commit
`i`, `d` = distinct interned strings, `‖s‖` = padded payload length.

```
live rows          = n + Σ_i 1                      (context + beliefs)
lattice bytes      = 112 · (n + k)
edge bytes         = 16 · Σ_i E_i
stamp bytes        = 16 · Σ_i S_i
rehearsal bytes    = 8 · (distinct edges touched)
trajectory bytes   = Σ_thoughts T_thought
interner bytes     = 4 · d + Σ ‖s‖₄
checkpoint (SIck)  = framing + all of the above
```

| scenario | what the design fixes | derived envelope | measured by |
|----------|----------------------|------------------|-------------|
| **cold birth** (docs/04's apple scene) | a handful of inputs, `k` ≤ the trace's commit count | tens of records: the whole live lattice is **kilobytes**, because CONFIG SI's connectome is what the owner typed. A sleep here is legal and costs the report line plus **0 state bytes** (§2's sleep row, **G13(f).2**) | G1's row export, G13(f).2 |
| **long session** — the harness's own definition: `n ≥ G5_LONG_SESSION_INPUTS = 100` with contradictions and idle passes ([03 §7](03-constants-registry.md)) | `n = 100` ⇒ **≥ 11,200 B** of context envelopes before a single belief row. Whether it sleeps at all depends on `SLEEP_EVERY_QUIET_TICKS`, which is UNMEASURED, so the row records **no sleep count** — a run of this scenario reports `sleeps_completed` rather than assuming one | the interner is the interesting term, not the lattice | G5/G9(d)/G10/G11/G12(e) runs; **G13(l).4** if the cadence fires |
| **10⁴ live rows** (a session an order of magnitude past the gate scenario, arithmetic only) | `112 · 10⁴ = 1.12 MB` of records; with ~2 edges and ~1 stamp per row, `+ 0.32 MB + 0.16 MB` | **≈ 1.6 MB hashed per tick** in the naive form (§7) | G6.d.1 |

**The honest shape of this budget.** In **CONFIG SI**, SI's own growth is bounded
by what the owner taught: the live lattice is small, the interner is small, and
the memory problem is not there. In **CONFIG ENGINE**, memory is dominated by the
*frozen genome*, which is `G` edges of arithmetic owned by
[07](07-genome-build-plan.md) and is resident; L3 is a rounding error beside it.
Every uncomfortable number in §8 is therefore an ENGINE number, and every
comfortable one is SI's birth state doing exactly what the thesis says it should.

**3.4 — the core block: the whole byte cost of individuality plus sleep
bookkeeping.** [01 §13](01-data-schemas.md)'s 36 bytes are the only thing this
batch stores, and unlike every other line in §3 they are *constant*:

```
core block bytes             = 36            independent of n, k, rows, edges, sleeps
mutable bytes inside it      = 13           4 + 4 + 4 + 1  (the two counters,
                                             last_slept_at_t, the pass mask); the other
                                             23 are the seed, its format, the order
                                             length, the digest, block_version and the
                                             two pads — fixed for the life of the
                                             individual
bytes that change per tick   = 0             except at a sleep boundary
bytes that change per sleep  = 9 or 5        a completed sleep moves sleeps_completed,
                                             last_slept_at_t and clears a non-zero mask
                                             = 9; an aborted one moves
                                             sleeps_interrupted and writes the mask = 5.
                                             Never more than 13 of the 36, once per sleep
item 9's hash work per tick  = reuse         §7's chained sub-digest lets an unchanged
                                             36-byte class keep its previous digest
derived, NOT stored          = the quiet-run counter due() reads — a function of the
                               transcript's t values and the agenda's open count, so it
                               costs O(1) RAM and no hash byte, exactly like §4's
                               out-degree counter ([04 §24](04-algorithms.md) X.2)
```

**Why the split is the interesting part:** the cadence needs a counter, and the
resume needs to know *where the last sleep stopped*. The first is derivable from
the transcript, so it is not stored; the second is not derivable from anything
(only `si-sleep` knows whether P3 finished), so it is in the block and therefore
in item 9 — which is why `sleep_pass_mask` and `last_slept_at_t` are the block's
only two fields a **resume** reads, and why **G13(h)** can promise a re-run rather
than a half-applied rescale without adding a journal, a WAL, or a tenth hashed
class.

---

## 4. Per-thought cost — the walk

Per round `r`, with `L_r` lit sources (bounded by `FRONTIER_CAP`), per-source
`reach(a)` = ρ and live out-degree `deg`:

```
contributions(round r) = Σ_{p ∈ lit} min(deg(p), ρ(a))
edge touches(thought)  = Σ_{r=1..R} contributions(r)   ≤ FRONTIER_CAP · ρ · K_MAX_ROUNDS
bytes read(thought)    = 16 · edge touches             (the edge record is the unit of reading)
```

The design's own targets, quoted with their source and their status:
**frontier ~10² points** and **~10⁵–10⁶ edge touches per input, *regardless of
graph size*** ([docs/16 §16.1](../docs/16-think-walk-hardware.md) — owner-supplied
design targets, **UNMEASURED until runs**). The clause that matters for this file
is the one nobody quotes: *regardless of graph size*. Sparse frontier
propagation is why a two-word question costs the same on day one and day ten, and
it is the property G6.d.1 has to demonstrate rather than assume.

**Working set per thought** (all derived from [01 §3](01-data-schemas.md) grids):

| structure | size | bound |
|-----------|------|-------|
| activation vector over lit points | 4 B × `FRONTIER_CAP` | `FRONTIER_CAP` is swept; trim order is (activation desc, first-lit round asc, point id asc) |
| previous-round vector (for the L1 `diff`) | 4 B × same | the `diff` operator [04 §5](04-algorithms.md) committed to needs both rounds |
| per-round attribution lines (I8's citable bias) | O(edges whose `β·σ` or `γ·m` term was non-zero) | written into the trace at step 9, never summarised away |
| `a` trajectory append | 1 B × signal events this thought | events per thought are *not* bounded by a constant — §9's alarm row |
| settle-set + top-k | O(lit points at settle) | hashed sorted by point id, displayed sorted by activation ([01 §10](01-data-schemas.md) item 5) |

**Three cost traps this file closes, because the prose law does not mention
bytes.** Each is a place where a lawful reading and an unlawful one differ only in
how many times the loop touches something it already touched.

1. **`deg` recompute.** [04 §5 step 7](04-algorithms.md) says the hub-damp
   denominator is "recomputed per tick from the live web". Read literally that is
   O(all edges) per tick — the one sentence in the algorithm file capable of
   making the walk quadratic in the session. Commitment: maintain a **per-point
   live out-degree counter** (`u32`, 4 B), incremented at `append_row` and
   decremented at `age` (a superseded edge is weight 0, so it leaves `deg`). The
   counter is *derived* state — rebuildable by a scan — so it is **not** in the
   hash ([01 §10](01-data-schemas.md)), and a checkpoint that persists it must be
   able to discard it. The reciprocal table over the reachable degree range is
   built once per tick from the counter's observed max, never per point.
2. **The median's stamp scan.** [04 §9](04-algorithms.md) already restricts it to
   points touched this thought; the byte consequence is that a point's stamp list
   is walked through the edge's `charge_off` arena, so median cost is
   `O(touched edges × stamps per edge)` and **not** `O(all stamps)`. `src = 3`
   rows are skipped at read, which is a branch, not a filter pass.
3. **Recomputing the composite tally per sleep.** [04 §21](04-algorithms.md) U.4
   puts the tally over logged settle-sets, and "over the log" reads like "scan the
   log". Done that way, P2 costs `O(thoughts this session)` **per sleep** and the
   session becomes quadratic in the thing meant to be quiet. Commitment: keep the
   tally as **derived RAM**, folded in as each settle-set is logged and rebuilt by
   one scan at open — the same status as trap 1's counter, so it is not hashed, not
   serialized, and discardable. Its cost per sleep is then `O(composite points)`
   and its memory is one entry per distinct composite the owner has ever produced,
   which is bounded by the interner it borrows its ids from (§9's row).

---

## 5. Per-idle-tick cost — the floor, not the spike

`IDLE_PER_TICK = 1` is **LAW** ([03 §5](03-constants-registry.md));
`IDLE_PER_HOUR` is in **logical ticks**, swept. Precedence is fixed
([04 §19](04-algorithms.md)): queue revisit → due RE-PARSE → ignition, and
ignition only with an empty agenda.

| part of an idle tick | cost | who bounds it |
|----------------------|------|---------------|
| the quiet-tick test itself | O(1) — a step counter and the agenda's open-row count | the agenda is a shelf, so "empty" is a count, not a scan |
| queue revisit pass | O(open question rows) | curiosity's own budget: `ASK_CAP_PER_INPUT`, `DEDUPE_WINDOW` |
| RE-PARSE pass (only when eligible) | O(S-R rows **with no derived child**) | [04 §18](04-algorithms.md)'s eligibility rule makes the pass a delta, not a full re-derivation; idempotence (G7.b.1) is what proves the delta emptied |
| ignition candidate pool | O(pool) — see **OD-61** | the pool must be bounded by a window, not by the lattice |
| the ignited cascade | one walk — §4's arithmetic, unchanged | the same `K_MAX_ROUNDS`, `FRONTIER_CAP`, `τ` |
| monologue render | O(settle top-k) frames into the **separate lane** | `MONOLOGUE_RENDER_DEPTH`; never the owner-facing stream (OPEN-23) |
| proposed-composite bookkeeping | **0** while `INFERENCE_PROMOTION = off` | OD-58; if it lands on, the cost is one `links` row per newly stable composite, and `INFERENCE_STABLE_N` is the anti-spam bound |

Idle cost is a **floor on the session's tick count**, not a spike in memory: a
quiet hour is `IDLE_PER_HOUR` walks, and each is the same arithmetic as an
answered question. The budget that keeps a quiet SI from spinning is the
per-hour ceiling expressed in logical ticks (OPEN-26) — expressed there, measured
there, never here.

### 5.1 Per-sleep cost — the envelope **G13(l).3** checks

A sleep is at most `SLEEP_MAX_WALKS` walks (**OD-78**, **OD-85**) plus two
read-mostly sweeps, so its cost has three terms — walks, sweep traffic, and the
hash repair the sweep forces (§7) — and each is bounded by a registry constant or
by `E_live`, not by the session's length:

```
walks            ≤ SLEEP_MAX_WALKS                (P1 first, P4 the remainder)
edge touches     ≤ SLEEP_MAX_WALKS · FRONTIER_CAP · ρ · K_MAX_ROUNDS   (§4)
bytes read       = 16 × the above
P3's sweep       = 16 B read + 2 B rewritten per live OWNED edge
                 = 18 B × E_live, in place, 0 allocations
P4's selection   = the live web, read-only, point-handle order
                 (full even when the walk remainder is 0)
t consumed       = the walks actually run — a sleep of w walks is w quiet ticks
hash work        = w chain steps of §7, + item 9's ≤ 13 mutable bytes once,
                   + the class-2 chain repair §7 prices (the expensive term, and
                   the one a sleep alone causes)
report           = one sleep.jsonl line per pass + one per sleep (log lane, §9)
```

| part of a sleep | cost | who bounds it |
|-----------------|------|---------------|
| P1's replay walks | one ordinary thought each, §4's arithmetic unchanged | `min(rows_new_since_last_sleep, SLEEP_MAX_WALKS)` — **OD-85**; the truncation is *reported* (`events_available` vs `walks`), which is **G13(l).4** |
| P2's tally | a read of the composite ledger, which is **derived RAM folded in as each settle-set is logged and rebuilt by one scan at open** ([04 §21](04-algorithms.md) U.4, §4's third cost trap) | `O(composite points)` per sleep, **not** `O(session)` — the trap exists precisely because the lawful prose reads like a log scan. It writes nothing either way while `INFERENCE_PROMOTION` is off (OD-58) |
| P3's rescale | `18 B × E_live`: one read, one multiply, one in-place 2-byte write per edge | the live edge count (§8's arena ceiling), **once per cadence** — this is the only linear term sleep adds, and the reason `SLEEP_EVERY_QUIET_TICKS` is swept rather than hand-argued (**OD-83**'s discipline applies to it too) |
| P4's sweep and walks | read-only selection over the live web, then the budget remainder as walks | the same single `SLEEP_MAX_WALKS`, shared with P1 — there is no second pool and no per-pass cap, because a second cap is a second constant to sweep |
| the sleep's hash work | §7's chain per walk; item 9's 36 B re-digested when a counter moves; **and the class-2 chain repair**, which is a full edge-class re-digest because P3 writes near every shelf's head | 36 B is the cheapest class in the composition and moves ≤ 13 bytes per sleep (§3.4); the repair is `16 B × E_live` once per sleep and is the term §7 prices — **the most expensive thing a quiet SI does, and the reason the cadence is swept** |
| an interrupted sleep | a **prefix** of the rows above | four inbox reads, O(1); OD-79's partial state needs **no rollback bytes**, because a pass's writes are one FLUID sweep and the unit of atomicity is the pass, not the transaction. An abort before P3 therefore skips the repair entirely, which is the cheapest interruption and the one an owner causes most often |

**Amortized, and what this file will not claim.** Per tick, sleep adds
`(18 B + 16 B) × E_live / SLEEP_EVERY_QUIET_TICKS` of traffic — the read/multiply/
write of P3 plus its hash repair — together with at most
`SLEEP_MAX_WALKS / SLEEP_EVERY_QUIET_TICKS` walks of §4 cost. Which of the terms
dominates at a given live-edge count is a **G13/G6 measurement**, not a statement
available here. What *is* derivable, and is the whole point of the section: no
term grows with session length except through `E_live`, which §8 already caps, so
sleep joins §9's list of things a reviewer can check without a profiler rather
than becoming a sixth growth stream. A cold-start sleep over a nearly-empty L3
costs the report line and 0 state bytes (**G13(f).2**), and that case is
legitimate rather than a pass that skipped its work.

---

## 6. Sync budget

| build | contract | status |
|-------|----------|--------|
| **v1 (`si` config, the hashed walker)** | the walk is CPU fixed-point ([docs/16 §16.6](../docs/16-think-walk-hardware.md)), so `drive.update` per signal event is a function call: **zero host syncs exist to count**. The one flush per tick is the event log at [02 §11](02-module-contracts.md) step 13 — and **sleep does not change that count**, because **OD-78** makes each of its walks a tick with its own step 13 instead of one tick doing `w` things | structurally satisfied; `check_record_layout` + G6.d.2's call-path assertion keep it satisfied |
| **v2 (`engine` config, GPU residency)** | **one sync per thought** (§16.5): K rounds fused into one go, frontier-masked SpMV per round, return once at settle with top-k + trace (few KB) | the design's constraint, not a measurement. LoopNet's idiom carries over: persistently mapped `HOST_VISIBLE\|HOST_COHERENT`, one batched command buffer per dependency chain, **flush only when the host reads or overwrites a live buffer**, `flush_count` exported |
| v2 + per-event arousal | §16.5 names three candidate shapes (pre-launch signal knowledge — impossible, a dark spot is discovered *by* the spread; on-device fused wordless integration; or one host sync at the thought boundary for ENGINE only) and **no measured one** | **UNMEASURED, v2, do not claim it works.** Recorded here so a reviewer cannot accidentally promise it |

**G6.d.2 is a call-path assertion, not a benchmark:** a per-round sync reachable
from the walk is a fail whatever the timings look like. The GPU leg's determinism
contract is grid-parity, never bit-exact floats (§16.6), and `GRID_TOLERANCE`
([03 §7](03-constants-registry.md)) is the only epsilon anywhere in this budget —
it is a *representation* tolerance, not a performance one. `REPLAY_TOLERANCE`
stays **0 bytes**.

---

## 7. The hash, which is the only O(session) line in a tick

[01 §10](01-data-schemas.md)'s composition covers the **whole** lattice every
tick: classes 1–4 are `112·rows + 16·edges + 16·stamps + interner`. **Items 5–9**
are the session's own arithmetic — the settle grid, the `a` trajectory, the PRNG
state, the version tags, and the 36-byte core block — and of those only item 9 can
move without a new row, which is exactly **13 bytes once per sleep** (§3.4) rather
than a per-tick cost. The naive reading — re-hash everything per input — is
deterministic and wrong to ship, because it makes the determinism law the most
expensive thing in the loop:

```
naive hash input per tick  ≈ 112·rows + 16·edges + 16·stamps + interner
                           = 1.6 MB at 10⁴ live rows (arithmetic from §3)
walk bytes per input       = 16 · (10⁵ … 10⁶) = 1.6 MB … 16 MB (design targets)
```

So the two costs **cross at roughly the low end of the walk's own range**, and
which side of the crossing a real session is on is a G6.d.1 measurement, not a
claim. Commitment, and it changes nothing about the law:

* **per-class sub-digests**, chained, composed as `md5(class₁ ‖ … ‖ class₉)` in
  exactly [01 §10](01-data-schemas.md)'s order. The **covering set and the order
  are unchanged**; only the construction is incremental, so per-tick work becomes
  O(bytes written this tick) plus the chain repair described in the next
  paragraph — for an ordinary input, a context record, maybe one belief row, its
  edges and stamps, a handful of interner entries.
* Because the construction changes **every digest byte** relative to a naive
  whole-lattice scan, it must be fixed **once, before the first G5 run**, and
  ride the version-tag mechanism as `HASH_FORMAT`
  ([03 §8](03-constants-registry.md)). A format flip after run directories exist
  invalidates every stored `state_hash.txt`, and `REPLAY_TOLERANCE = 0` makes
  that a loud failure, not a silent one — which is the right direction for a
  change of this kind. **OD-60**.
* Sorted classes stay sorted: within a shelf, `y` is monotone in arrival order, so
  a shelf's digest is genuinely append-only; `(x, y)` order across shelves is
  fixed by the registry, so the class digest is a chain over shelf digests. That
  is the whole reason the sub-digest boundary is the shelf, not the row.

**The one thing that breaks the chain, and its price — because sleep breaks it
deliberately.** A chain digest is append-only, and §2's FLUID class rewrites bytes
**in the middle**: a boring repeat's 14 B, a supersession's 8 B in the loser, and
P3's whole sweep. So a write at offset `o` in class `c` invalidates `c`'s
sub-digest from `o` to that shelf's end, and the repair is
`O(bytes after o in that shelf)`. Ordinary inputs append and repair nothing; a
boring repeat on the newest row repairs a few hundred bytes; a supersession
repairs the loser's shelf suffix. **A sleep is the design's worst case**: P3
rewrites 2 bytes across every live edge, so its first write sits near the head of
every shelf and the repair is a full re-digest of the edge class =
`16 B × E_live`, once per sleep — at §3's 10⁴-row scenario with ~2 edges per row,
**≈ 320 KB of hash input per sleep**, divided by the cadence. Two consequences a
builder must not miss: `SLEEP_EVERY_QUIET_TICKS` is swept *because* it is what
divides this term (**G13/sleep_sweep.json** carries a hash-cost column per point,
[05 §4](05-gate-harness.md)), and a cadence that fired every tick would be the
naive scanner wearing a counter — which **G13(l).3** reads as an envelope breach,
not a style choice. The alternative optimizations are all forbidden by §11: a
dirty-bit per edge would be a second, unhashed truth about reachability, and a
"hash only the changed edges" shortcut changes item 2's covering set, which is
law, not arithmetic.

**What stays out of this arithmetic:** the median (derived, and a stored one is
R-28), audit labels (output, R-24), wall-clock, `--config`, viewer state, Grimm's
process state. Deleting `si-audit` must change no hash byte except the label
strings in the log — G9(c) — which is also why no audit structure has a byte
anywhere in this file.

---

## 8. Residency, and the address space that caps it

| quantity | derived | consequence |
|----------|---------|-------------|
| genome edge bytes | `16 B × G`, `G` owned by [07](07-genome-build-plan.md) | the resident set is `G`-driven, not session-driven |
| the design's own wall | "residency, not compute, is the eventual wall at **10⁸+ edges**" ([docs/16 §16.5](../docs/16-think-walk-hardware.md)) | `10⁸ × 16 B = 1.6 GB` (1.49 GiB) **of edge records alone**, before point tables, degree counters or the interner. That is arithmetic on the design's target and the schema's constant — **it is not a measurement, and it is not a claim that any device holds it** |
| edge/stamp arena ceiling (`u32` byte offsets, 16 B records) | `2³² / 16 = 268,435,456` ≈ **2.7·10⁸** | the arena scheme reaches only ~2.7× the wall the design itself names |
| record arena ceiling (112 B) | `2³² / 112 = 38,347,922` ≈ **3.8·10⁷** rows | ~4·10⁷ teaching events per shelf before the handle space, which is not the binding constraint |
| point handles | `u32`, `0` = absent ([01 §11](01-data-schemas.md)) | the walk's address space, and `0xFFFF_FFFF` is spent on NULL |
| `t` | `i32` logical step | `2.1·10⁹` steps; `i32::MAX` is spent as the "never superseded" sentinel, so the working ceiling is one below it |

**The finding worth writing down before anyone builds:** the `u32` arena offset is
*adequate and not generous*. Widening is not free — the edge record has one 2-byte
pad ([01 §5.1](01-data-schemas.md)) and a `u64` `charge_off` would break the
16-byte law that every estimate in this file is derived from, so a widening is a
**schema version change** (`row_version`/`EDGE_RECORD_BYTES` in
[03 §8](03-constants-registry.md)), and the reserved-byte assert exists precisely
so nobody widens silently. The genome question this file deferred is **closed in
[07 §11](07-genome-build-plan.md)**: the frozen base uses an **element-counted**
`u32` CSR index, not a byte-offset arena, so `G` reaches 4.29·10⁹ by construction
and the design's own 10⁸ wall is a **bytes** problem, not a handle problem — which
is why the `2.7·10⁸` ceiling above binds L3 only, and why frozen records are packed
(`DICT 32 / REL 16 / RULE 16`) instead of reusing the 112 B live envelope.

**Region paging by `x`-shelf stays parked as v2+** (§16.5, a shelf-registry
concern). If a knowledge type ever appears that neither `(t,x,y,z)` nor the
registry can express, that is **R-12's alarm**: report it, redesign the registry
entry, do not add a sixth structure or a side channel — and if it appears as a
*memory* problem, the answer is still the alarm, not an eviction policy that
deletes rows (I4 forbids the cheapest version of paging outright: nothing is
ever dropped, only aged).

---

## 9. Growth pressure — what grows, how fast, who caps it

| stream | grows as | resident or disk | capped by | alarm |
|--------|----------|------------------|-----------|-------|
| `context` rows | `112 B` per input, both voices | disk; only live rows are hashed | nothing — this is the transcript, and I5 needs it | none; it is the point |
| belief rows + edges + stamps | per §2 | resident-ish (L3 is small in CONFIG SI) | `surprise = 0 ⇒ write nothing` is the big one (§6.2), then arbitration's supersede-not-delete | a session that writes on every input is G1 failing, not a budget problem |
| interner | distinct folded strings + surfaces | resident (a walk needs it) | vocabulary = what the owner taught; `0xFFFF_FFFF` reserves the top id | unbounded *by design*; it is learning |
| `a` trajectory | 1 B per **signal event** | resident for the session (it is hashed) | the six kinds and the walk's own arithmetic; per-thought event count is not a constant | if trajectories dominate, `a` is being fed by something outside §18.1's table — that is an I8 violation, not a sizing problem |
| rehearsal stream | 8 B per distinct edge touched | disk, tail-only reads | the live edge set | none |
| monologue lane | per idle cascade | separate file | `IDLE_PER_TICK`, `IDLE_PER_HOUR` | a spin is G10(a)'s fail |
| `sleep.jsonl` | one fixed-shape line per pass plus one per sleep, so it grows with **quiet time**, not with content | disk, append | `SLEEP_EVERY_QUIET_TICKS` sets the rate; each line is a handful of counts, so its size is O(1) per sleep | far more lines than `ticks / cadence` is **G13(f).1**'s double-sleep, not a logging problem |
| the composite tally | one entry per distinct composite point the loop has settled | **RAM only**: derived, never serialized, never hashed, rebuilt from the eventlog at open (§4's trap 3) | the same vocabulary the interner already owns, because a point it cannot name it cannot count | a tally that got persisted is [01 §10](01-data-schemas.md)'s excluded-class law broken — `INFERENCE_STABLE_N`'s count becoming state nobody hashes |
| `mindlog` / `stream.siro` | per decision / per output line | disk, append | never capped (audit needs it), and **never read back as build input** (R-33) | a run that reads its own log is the failure, not the log's size |
| gate run directories | per run | disk | `gates/README.md`'s contract | a sweep artifact read back as a design premise |

**Derived, and the most useful line in this file:** the only quantities that grow
with *session length* rather than with *content* are the append-only logs (§2) and
the hash's naive form (§7). Everything that must stay flat — walk cost per input
(§4), median cost (touched points), conduction cost (the frontier), idle cost
(rate-bound), sleep's three terms (§5.1) — is bounded by a constant the registry
owns. A build that shows superlinear per-input cost has broken one of those five
bounds, and the five names are checkable in a review without a profiler.

**Dimming is not a deletion, and the budget says so in one line.** A dark edge
keeps its 16-byte record, its `bake`, its stamps and its citability; `weight`
falls toward a floor that is **≥ 1 by registry law** ([10 §6](10-temperament-sleep.md)
D.3), so the live edge set never shrinks because SI slept, and no eviction,
compaction or "prune the unroutable" optimization is available to a builder who
wants the memory back. `weight == 0` means superseded and nothing else — which is
why `G13(d)` is a budget assertion as much as a semantic one: if dimming could mint
zeros, the merge would read a live row as aged and the arena's only compaction
signal would lie.

---

## 10. The G6 field checklist — every number this file leaves blank

`gates/G6_<config>_<yyyymmdd-hhmm>/gate_report.json` writes these fields and
nowhere else; the harness fills them on a real run, and until it exists every one
of them is **UNMEASURED** in all of `docs/` and `engineering/`.

| field | unit | source of the requirement |
|-------|------|---------------------------|
| `rows_live`, `edges_live`, `stamps_live`, `interner_entries` at session end | counts | §2, §3 |
| `lattice_bytes`, `edge_bytes`, `stamp_bytes`, `interner_bytes` | bytes | §3's formulas, checked against the file |
| `edge_touches_per_input` (min/median/max over the run) | touches | docs/16 §16.1's design target of 10⁵–10⁶ |
| `rounds_per_thought`, `settle_reason_hist` | rounds / counts | `K_MAX_ROUNDS`, `OSCILLATION_RULE` |
| `hash_input_bytes_per_tick` (naive vs sub-digest form) | bytes | §7 |
| `genome_resident_bytes`, `flush_count_per_thought` | bytes, counts | §6, §8; LoopNet's `flush_count` idiom |
| `device_memory_observed` | bytes | §8 — **reported as telemetry, never converted into a "fits" claim** |
| `idle_cascades_per_quiet_tick`, `per_hour_achieved` | counts, logical ticks | §5, G10(a) |
| `sleeps_completed`, `sleeps_interrupted`, `walks_per_sleep` | counts | §5.1, [01 §13](01-data-schemas.md) — G6 reads the counters back out of the container it is already measuring |
| `bytes_changed_per_sleep`, `edges_dimmed_per_sleep` | bytes, counts | §2's sleep row and §5.1: both are sums over `sleep.jsonl` lines, so G6 records **a count the run already wrote**, never a swept magnitude |
| `hash_bytes_per_sleep` (the class-2 repair) | bytes | §7's repair paragraph — the term that is the only reason the cadence is a swept constant instead of a mood |
| `rows_live_before_sleep` vs `rows_live_after_sleep` | counts | §2's 0-new-bytes row and **G13(i)**: these are equal, always, and a run where they differ is not a budget finding but an I9 finding |
| `monologue_entries`, `monologue_bytes` | counts, bytes | §19.3's lane |
| `replay_wallclock_ms` vs `run_wallclock_ms` | ms | **telemetry only**; the gate's actual assertion is byte-identical `state_hash.txt` |

`G6.d.1` is the leg that fills this table and `G6.d.2` is the leg that asserts the
sync shape from the call path ([05 §11](05-gate-harness.md)). CONFIG SI gets its
own G6 run for L3-only arithmetic; the ENGINE run is the one that touches
residency, and it is also where **G13(e)** checks that sleep wrote only L3.

**What G6 does not price, and no field above may imply:** `DIM_BASE`, the three
floors, `SLEEP_EVERY_QUIET_TICKS`, `SLEEP_MAX_WALKS`, and every "how different are
two individuals" spread. Those live in `G13/sleep_sweep.json`,
`G13/dimming_sweep.json` and `G14/seed_panel.json`
([05 §4, §4.1, §4.2](05-gate-harness.md)) and read **UNMEASURED** here until those
directories exist. G6 counts bytes a run already wrote; a sweep describes a
distribution the owner has not seen; §1's rule 2 does not bend because a sleep
number would look natural beside a hash number.

---

## 11. What may never be "optimized" into this budget

| forbidden shortcut | law it breaks | caught by |
|--------------------|---------------|-----------|
| caching a point's median (a dict that outlives the thought) | R-28 — a frozen emotional address | G5(a), G9(c), `check_no_mood_field` |
| storing the rehearsal/decay curve value | R-29 — the value is derived from the one clock | [01 §8](01-data-schemas.md), G11(c) |
| materializing the whole adjacency graph as float vectors to make the walk faster | I1, R-14 — zero tolerance | `check_no_embeddings` |
| a page cache that drops superseded rows to save bytes | I4 — superseded ≠ deleted; aging is not eviction | G3, G8(c) |
| a bloom filter / embedding-ish "maybe this word relates" index | I1, R-14 | `check_no_embeddings` |
| sampling the walk's spread ("skip half the edges") | determinism + the settle test's meaning | G2, G5(a) |
| a second, faster clock for affect to cheapen the median — **or a private decay inside `si-sleep`** | R-29 | `check_one_clock`, G11(c), **G13(k)** |
| reading `mindlog.json` back to avoid a re-scan | R-33 — a JSON that re-enters the loop is a channel | `check_no_json_readback` |
| a `u64` arena offset "just for now" | breaks the 16/112-byte law every number here is derived from | `check_record_layout`, reserved-byte assert |
| writing a composed inference to a row so it need not be re-derived | I9/R-25; and OD-58 is the owner's, not the build's | **G11(h)**, `check_compose_writes_nothing` |
| **sleep committing a row so consolidation "sticks"** — including the friendly shape, "just a `questions` candidate" | I9/R-25 extended to the new organ; a candidate *is* a row ([10 §8](10-temperament-sleep.md) law 2) | the `si-sleep ↛ si-learn` missing edge, `check_sleep_writes_nothing`, **G13(i)/(j)** |
| **a rescale that brightens a row to conserve mass** | I6, §8.4's event-driven confidence, R-07's inflation by a new door | `check_sleep_never_brightens`, **G13(c)/(d)**, **OD-75** |
| **a per-edge floor** stored beside `weight` so bake could protect an edge | §5.3(e)'s bounded permanence; bake's two licensed outlets; and it would make the floors a fourth mutable column §2's closed list does not contain | **OD-77**, **G13(d)** |
| **a third `W_eff` term** so dimming is computed at read time and nothing is stored | §18.5's two-term cap (OPEN-19); and the citation, the panel and the walk would be reading three different numbers about one edge | **OD-76**, [02 §12](02-module-contracts.md)'s grep rules (ix)–(xi) |
| **storing the derived temperament vector** as a cache of 18 values in the container | an address the seed does not justify, which drifts from its own seed silently — R-28's shape in a personality font | `check_temperament_closed`, **G14(c)** |
| **evicting dark edges** to reclaim the bytes §5.1's sweep spends | I4: superseded ≠ deleted; and `weight == 0` means *superseded* only, which is the merge's one compaction signal (§9's dimming paragraph) | **G13(d)**, G3, G8(c) |
| **scanning the whole eventlog for the composite tally at every sleep** | flat-cost law §9; it makes quiet time quadratic in the session | §4's third cost trap, **G13(l).3** |

---

## 12. Deferred by design (and where the deferral is recorded)

* **Region paging by `x`-shelf** — v2+ (§16.5; R-12 if the registry ever resists).
* **GPU walker parity** — v2, grid-parity contract only (§16.6, `GRID_TOLERANCE`).
* **Per-event `a` fused on device** — v2, three candidate shapes, UNMEASURED (§6).
* **Phrase batching** (`PHRASE_BATCHING`) — default off; only a measured leg may
  turn it on (OPEN-13).
* **The v2 LM renderer** — form only, never a source of truth; its memory is not
  SI's state and never enters §3's arithmetic (R-08).
* **Sleep staging** — v1 is four fixed passes in fixed order, with no pass reading
  a previous pass's output (**OD-80**); a pass graph is a scheduler in disguise,
  and a scheduler is a new organ nobody scoped.
* **`prov_kind = 5 consolidated`, and with it any lawful write from P2** — inert
  until **OD-58** lands (**OD-82**). The row shape is documented so the ruling is
  one edit; the `si-sleep ↛ si-learn` edge stays missing until then, so the
  *cost* of that ruling is also `—` here, not a number.
* **A per-edge dimming history** (a weight-snapshot lane so the panel can chart one
  word's fade across nights) — `(last_touched_at, weight)` plus the aggregates in
  `sleep.jsonl` already answer "when did this go quiet"; a per-edge history would
  be a fifth arena, which is **R-12**'s alarm and not a feature.
* **Reserved shelves** (`skills`, `strategies`, `events`) — registry entries with
  zero bytes today; a twelfth `x` fails the build (`check_registry_size`, R-12).

---

## 13. What this file decided, and the two items it hands to the owner

**Decided here, inside the law:** the three-arena layout and the no-secondary-index
rule; the closed mutable-in-place column set (§2); the per-point live out-degree
counter and the touched-points-only median scan (§4); the shelf-scoped chained
sub-digest as the *construction* of [01 §10](01-data-schemas.md)'s unchanged
covering set (§7); the degree-counter's derived-not-hashed status; and the rule
that wall-clock appears in `gate_report.json` as telemetry and never as a promise.

**Decided here for the thirteenth crate, the same way — a construction, not a new
law.** Four of them, and each is cheap to state and expensive to reverse, which is
why they are written down rather than left to the builder:

* **Sleep's per-sleep envelope (§5.1).** One `SLEEP_MAX_WALKS` pool, spent in pass
  order, plus exactly two O(live-edges) sweeps — P3's rescale and P4's selection.
  The arithmetic is what forced it: [10 §8](10-temperament-sleep.md)'s P1 said
  "replay" and this file said "bounded", and un-reconciled those two read as an
  O(session) cost inside a tick loop that has no third growth stream. The ruling
  is **OD-85** and it lives in [10 §13](10-temperament-sleep.md), because that file
  owns the passes; this file owns the price, and the price is what settled it.
* **The composite tally is derived RAM (§4's third trap).** It is folded in as each
  settle-set is logged and rebuilt by one scan at open, so it is never a stored
  column, never serialized and never hashed — the same status the degree counter
  won in §4. A second source of truth for "how many times did this compose" would
  be a *PROVENANCE* claim resting on a mutable byte.
* **The chain-repair rule (§7).** An in-place FLUID write invalidates its shelf's
  sub-digest from the write offset to the shelf end, so P3's first write — which
  sits near every edge shelf's head — costs a full `16 B × E_live` re-digest per
  sleep, and no dirty-bit shortcut is allowed because a shortcut changes item 2's
  covering set. That number is a construction cost, not a tuning target: the law
  is "the digest covers the bytes", and the honest price of obeying it is the only
  thing §7 was willing to publish before a sweep.
* **"Dimming is not a deletion" (§9).** A dark edge keeps its 16-byte record, its
  bake, its stamps and its citability; `weight == 0` means superseded and nothing
  else. Stated as a budget rule because the tempting optimization — evict edges
  below a threshold — is exactly the kind of cheapening §11 exists to refuse.

**Two new ledger items.**

* **OD-60 — the state-hash construction.** The design fixes *what* is hashed and
  *in what order*; it does not say whether the digest is recomputed whole per tick
  or chained per shelf. Lean: **chained sub-digests**, fixed once before the first
  G5 run, version-tagged as `HASH_FORMAT` so an old `state_hash.txt` can never be
  replayed under the new construction without saying so.
* **OD-61 — the bound on the ignition candidate pool.** §19.2's four weights are
  per-edge and per-point, but the pool they rank is unnamed, and an unbounded pool
  makes idle cost grow with the session — which contradicts the design's own
  position that idle thought is rate-bound, not size-bound. Lean: **the rehearsal
  stream's newest window** (the edges `Clock::recency` still scores above zero), so
  the pool shrinks by the same curve everything else uses and needs no second
  decay law (R-29). Its size is swept with `G10/ignition_sweep.json`.

---

## 14. The v2 arc's cost — every row UNMEASURED, deliberately

Appended last for the same reason §28 of [04](04-algorithms.md) and §23 of
[05](05-gate-harness.md) are: §1–§13 are cited by number from five files. Nothing
in §1–§13 **moves**: no row, edge or stamp byte grows (`112 / 16 / 16`, §1), so
every per-row arithmetic and every derived capacity in this file stays valid for
the arc's five families unchanged. What follows is the *added* work, in the same
shape as §4 and §5, and it is written before any of it is measured because the
alternative is a builder inventing a number under time pressure.

| mechanism | per | added work | bounded by | measured? |
|-----------|-----|-----------|------------|-----------|
| co-light append | **light** | one 12 B struct push into frame RAM | `FRONTIER_CAP × K_MAX_ROUNDS` — already bounded, so this is the cheapest line in the table and the only one that runs on *every* tick | UNMEASURED until G15's first run |
| ghost + role-graph build | frame | one fold over the frame log | `GHOST_MAX`, `SHAPE_MAX` (both `—`, so both boxes are open: **OD-83**) | UNMEASURED |
| `crystallize::observe` | thought | `O(candidates × members)` integer set-intersections, over **live L3 reads** | candidate count ≤ live rows ⁄ 3 (a member set is ≥ 3), and every member test is an integer hash compare — **no similarity, no float, ever** (R-14) | UNMEASURED |
| silt tick | tick | `O(active entries)` reciprocal multiplies, one clamp | the table's own live entries; `SILT_MAX` is a **diagnostic counter**, not a bound on work — a big table is a big fold, which is why the drop-at-`THETA_SILT` rule is the real cap | UNMEASURED |
| event-view index | thought | one pass over committed rows' provenance | commits per session (the same scan §4's composite tally already does at open) | UNMEASURED |
| tug re-light | frame | ≤ `RELIGHT_MAX` point lightings, **zero** arena writes | `RELIGHT_MAX` (`—`) | UNMEASURED |
| `skills::observe` | thought | role extraction + one template interning per shape | shapes ≤ distinct role-graphs seen; the interner is the one §9 already calls unbounded *by design*, so this adds strings to an existing growth stream and no new stream | UNMEASURED |
| `world::class` | line | a four-arm comparison chain over `grants` | `grants` is **empty at birth**, so tonight it is O(1); the cost only becomes real when a grant table exists, and then it is O(grants) per line with grants written only by the owner | UNMEASURED |
| the three `.jsonl` logs | tick / thought / transition | append-only bytes on disk | nothing — and they are §9's "log ≠ state" line, never read back as build input (R-33) | UNMEASURED |

**Four things this table does not add, which is the point.** No new arena, no new
resident column set, no new growth stream that is not already in §9 (the logs are
disk-append like `sleep.jsonl`; the RAM tables are derived like the composite
tally), no new O(session) line inside a tick — §7 stays the only one — and no new
per-thought term in the walk's own arithmetic: §4's cost model in `FRONTIER_CAP`,
`K_MAX_ROUNDS` and the frontier's edges is **unchanged**, because the arc's five
observers read the frame log *after* the settle and cannot make a round cost more.

**The one line a builder should distrust.** "The arc adds no per-thought cost to
the walk" is true of the *spread*, but `crystallize::observe` and `skills::observe`
are both **O(candidates)** per thought, and a candidate table that never clears is
a per-thought cost that grows with the session. The bound is the detectors' own
rule 4 (§29 AA.2: an already-held cluster opens nothing) plus the fact that
recurrence is counted per **member-set hash**, so the table is bounded by the
number of distinct ≥3-point co-light sets seen — which is bounded by content, not
by length. That is the honest shape, it is the same argument §9's composite tally
line makes, and **G15/G18 do not measure it**: no leg of the arc's grades table
growth, because no artifact exists to quote. If a real session ever shows
superlinear per-thought cost in a detector, the finding is **this** row, and the
fix is a bar in the registry — not an eviction, which §11 exists to refuse.

**Ledger items raised by this section:** none new. OD-102 (silt's values, blocked
on its sweep), OD-83 (the four open boxes, unrulled) and OD-116 (the shape
constants and the shape-scratch budget) are already the seats; this section's job
was only to make the price of each explicit before a build could discover it.

## 15. The tongue's cost — bounded, and every millisecond UNMEASURED *(B20;
[13 §7](13-tongue.md))*

Same reason as §14 for sitting last: §1–§13 are cited by number. Nothing in
§1–§14 **moves** — the voice writes no row, no edge, no stamp and no byte of the
hash inputs, so §1's geometry, §2's per-event arithmetic, §7's hash line and §9's
growth table are unchanged for a V14 run as for a V0 one.

| item | formula | measured? |
|------|---------|-----------|
| realized line length | `1 + ((PHRASE_MAX_STEPS × STYLE_SPREAD) >> 12)` chunks, each ≤ the longest licensed surface | **bounded**; the `UNMEASURED_PLACEHOLDER` budget makes the ceiling legal but non-binding — which is the honest state while OD-83/OD-172 are unrulled |
| per-realize work | `O(chunks × out-degree(anchor-chain))` arc reads + one `next_q12()` per chunk + one cumulative scan | **UNMEASURED**: no millisecond is quoted, per this file's own law. `G20(i)` grades the *bound*, not the cost |
| the graph | `Graph::build(&lattice, t)` is built by the **caller** and borrowed | the real price of a read-only leaf ([00 §2](00-stack-workspace.md) win 8): at V14, a session that already builds a graph per idle tick now builds one per realized answer. If a profile ever says it matters, the fix is a **caller-side cache**, not a `si-store` import — an import would delete the leg that makes OD-181 structural |
| RAM | `Style` = 10 bytes (re-derived, never stored) + one `Rng` (20-byte state) + the phrase buffer, alive for one tick step | derived from the schema, exact |
| hash | **zero** — nine items, none of them a voice | exact |
| files | **zero** new files written by the loop; the four `gates/G20/` artifacts are harness output | exact |

**What is *not* in the table, on purpose.** The realization adds no shelf, no row
kind, no `prov_kind`, no edge kind, no signal, no label and no container field — so
there is no §2-style "per teaching event" delta to write, and §9's growth-pressure
table gains no row. The tongue's growth bound is `PHRASE_MAX_STEPS`, a launch
constant, not a session quantity: **a longer-lived SI does not speak longer**, which
is the difference between this mechanism and the arc's two candidate tables.

**Ledger items seated by this section:** **OD-172** (the step budget, blocked on the
refused sweep) and **OD-179** (genome arcs are not walkable, which is what keeps the
candidate set bounded by *learned* content rather than by species size).
