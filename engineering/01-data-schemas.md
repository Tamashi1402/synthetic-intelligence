# 01 — Field-Level Schemas and Byte Layouts

Every stored thing in SI, down to its byte offset. Source of truth for the
field *names and meanings* is [docs/05 §5.2–§5.4](../docs/05-substrate-tesseract.md),
[docs/18 §18.5](../docs/18-emotions-engine.md),
[docs/20 §20.2–§20.4a](../docs/20-emotional-plasticity.md); this file adds only
what a compiler needs: widths, endianness, null encodings, alignment, and which
fields enter the state hash. It adds **no field the design does not already
name.**

Rules of this file: little-endian throughout (`u32le`, matching LoopNet's
`ckpt.rs`/`quant.rs`); every record length asserts `len % 4 == 0`; `0` in a
`u32` handle column means **absent**; string bytes never appear inline in a
record — every string is an interned `u32`.

---

## 1. The address `(t, x, y, z)` — 12 bytes of identity, 0 of index

| axis | width | type | encoding | notes |
|------|-------|------|----------|-------|
| `t` | 4 | `i32le` | logical knowledge step | `t = 0` is the frozen baseline (genome version tag); a live row's `t` is the step it was written at; **NOW = max committed `t`**, never wall-clock (I5) |
| `x` | 2 | `u16le` | shelf id from the registry | closed enum, §2 |
| `y` | 4 | `u32le` | row handle within the shelf | monotone per shelf, allocation order = arrival order ⇒ deterministic (I5); display ids (`r1`, `q1`, `f3`) are **derived at render** from `(x, y)`, never stored (**OD-51**) |
| `z` | — | edge list | arena offset + count in the record | targets addressed `(t, x, y)`; the list itself is §5's edge record |

**Hyperpoint identity.** A lit node in the walk is a `point_handle: u32`,
allocated on first arrival (deterministic). Alongside it the interner stores a
**content address** `hyperpoint_id: u64le` = the first 8 bytes of `md5` over the
canonical key string (folded term, or `(frame, slot, entity)` for a named
dark spot). This is LoopNet's `coords_of(key)` idiom (`tesseract-core/src/lattice.rs`
— three rotations of one hash mix) reduced to what SI needs: **dedup and
cross-shelf identity with zero stored index** (**OD-49**, *identity, not
security*). Two records that hash equal are the same point; nothing else about
them is inferred, and no similarity is computed from the hash (I1: it is not a
feature vector, it is a name).

**Never stored:** the point's emotional reading. There is no `median` column, no
bias cache, no mood field anywhere in this file — a persisted one is a fixed
emotional address (**R-28**, [docs/20 §20.3](../docs/20-emotional-plasticity.md)).

## 2. Shelf registry ids (the `x` axis is an enum, not a string)

| `x` | shelf_name | layer | row body (§6) | mutability |
|-----|-----------|-------|---------------|------------|
| 1 | `dictionary` | L1 frozen | `DICT` | re-versioned, never edited |
| 2 | `relations` | L2 frozen | `REL` | re-versioned |
| 3 | `rules` | L2 frozen | `RULE` | re-versioned |
| 4 | `s_r_pairs` | L3 live | `SR` | append-only |
| 5 | `facts` | L3 live | `FACT` | append-only |
| 6 | `links` | L3 live | `LINK` | append-only |
| 7 | `questions` | L3 live | `Q` | append-only; status changes by **new version row** |
| 8 | `context` | L3 live | `CTX` | append-only; **no charge, no bake** |
| 9 | `skills` | L3 live **from B17** | `SKILL` (§6) | append-only; retirement by **supersede**, never erase. Reserved-empty until the skills build — the id was always registered, which is [docs/25 §25.11](../docs/25-skills.md) note 2's whole point |
| 10 | `strategies` | reserved | — | v2 |
| 11 | `events` | reserved | — | **stays reserved after the arc**: [docs/24](../docs/24-episodic.md)'s event views are *derived* (OD-110), so nothing writes here and `row_schema_version` stays `0` — a non-zero write is a release-assert panic, not a policy violation |

`RESERVED_SHELF_COUNT` is **3** through B16 and **2** from B17; `SHELF_COUNT` is
**11** and never moves, and `SHELF_NAMES` is unchanged because `"skills"` was
always in the list. `check_registry_size` reads "eleven entries, *n*
reserved-empty" with *n* asserted against that stage line, and a twelfth `x` still
fails the build (R-12). Nothing in the v2 arc adds a shelf, a structure or an
envelope byte — that claim is the arc's compatibility contract and this table is
where a reviewer can check it ([12 §2.1](12-v2-arc.md)).

Registry entry (in-file, versioned): `{shelf_id u16, shelf_name interned u32,
row_schema_version u16, layer u8, source interned u32, mutability u8,
address_prefix interned u32}` — [docs/05 §5.2](../docs/05-substrate-tesseract.md)
field-for-field. A new knowledge type is a new `x` **inside this enum**, never a
new structure; a type that cannot be expressed this way is R-12's alarm, not a
sixth axis.

## 3. Quantization grids (**OD-46**) — every affective/numeric scalar

| quantity | width | grid | range | clamp / null |
|----------|-------|------|-------|--------------|
| walk activation | `i32` | **Q12** (1.0 = 4096) | 0 … 4096·reach_max | settle-set stored as `{(point, quantized_activation)}` sorted by id — never a float (§16.6) |
| edge `weight` (OWNED live) | `u16` | Q12 | 0 … 65535 | superseded ⇒ **0**, permanently in storage, reachable only history-anchored (I4) |
| genome `weight` (L1/L2) | `u8` | 255 = 1.0 | 0 … 255 | frozen at t=0; **no charge/bake field exists on these row types** |
| arousal `a`, and every point of its per-thought **trajectory** | `u8` | **1/255** | 0 … 255 (0 = rest) | `[0,1]` by §18.4's clamp; `0` at t=0 is law, not a default |
| `bake` (per L3 edge, permanent) | `u16` | Q12 | 0 … `B_max·4096` | `0` = unbaked, and `a_formation = 0 ⇒ bake = 0` (**OD-56** — the floor is law) |
| stamp `magnitude` (the firing signal's own value) | `u16` | Q12 | 0 … 65535 | never derived from `a` (OD-22 intact) |
| stamp `a_at` / `arousal_at_ask` | `u8` | 1/255 | 0 … 255 | `255` in `arousal_at_ask` = **none**; `a_at` has no null — a stamp always records the live level |
| confidence | `u16` | hundredths | 0 … **99** | §8.4's cap is 0.99, so 100 is unrepresentable, not truncated |
| `gain_exact` | `i16` | correction units | signed | `0` for an `asserted` row at commit (**OD-41/§8.2**) |
| decay `λ_a`, recency curve factors | `u8` | 1/255 | 0 … 255 | **every** decay-driven family reads **one** curve from `clock` (**R-29**); dimming's floor arithmetic ([10 §6](10-temperament-sleep.md)) reads that same curve and defines no other |
| modulation-lag | `u8` | rounds | small int | version-tagged constant, **OPEN-29**; its unit is OD-29's |
| `birth_seed` | `u64` | **no grid — opaque bits** | any, including 0 | `0` is a legal individual, not a null; there is no sentinel and no default ([10 §3](10-temperament-sleep.md)) |

**Rounding is a multiply, never a divide (OD-45).** Every grid conversion is
`x · INV` where `INV` is a reciprocal constant on the same grid — LoopNet's
`INV_GROUP_MAX = 1.0/7.0` idiom (`tesseract-core/src/quant.rs`), which is what
makes a CPU result and a GPU result bit-comparable. Round-half-up on ties, one
rule everywhere. Per-round normalization (§18.5 cap) uses the same reciprocal
multiply on the activation grid.

## 4. The record = 80-byte envelope + 32-byte body + arena refs

All **live L3 belief rows share one envelope** so every row is citable in one
mechanism ([docs/05 §5.4](../docs/05-substrate-tesseract.md)'s "common
envelope"). `context` reuses it and leaves the charge/bake offsets at `0`.

### 4.1 Envelope — 80 bytes, offsets fixed

| off | field | type | width | null encoding | meaning |
|-----|-------|------|-------|---------------|---------|
| 0 | `row_id` | `u32` | 4 | — | the `y` axis |
| 4 | `shelf_x` | `u16` | 2 | — | the `x` axis (§2) |
| 6 | `row_version` | `u16` | 2 | — | `0x0001` = `row-v1` |
| 8 | `written_at_t` | `i32` | 4 | — | the `t` axis |
| 12 | `last_confirmed_at` | `i32` | 4 | `0` = never | bumped only by a `source: owner` repeat (R-35) |
| 16 | `superseded_at` | `i32` | 4 | **`i32::MAX` = live** | `active_at(s)`: `written_at ≤ s < superseded_at ∨ ∞` |
| 20 | `superseded_by` | `u32` | 4 | `0` = none | successor row id, keeps the chain citable (I4) |
| 24 | `repeats` | `u32` | 4 | — | owner-taught repetitions only |
| 28 | `confidence` | `u16` | 2 | — | hundredths (§3) |
| 30 | `gain_exact` | `i16` | 2 | — | §8.2 correction units |
| 32 | `gain_kind` | `u8` | 1 | — | 0 = `asserted`, 1 = `demonstrated`; set once at commit from `prov_kind`, promoted by the first uncorrected serve (§8.3) |
| 33 | `prov_kind` | `u8` | 1 | — | 0 `correction` · 1 `statement` · 2 `reparse` · 3 `answer-to-ask` · 4 `owner-teach` |
| 34 | `prov_speaker` | `u8` | 1 | — | 0 `owner` · 1 `grimm` · 2 `system-inferred` (**OD-59**). Scope is the ruling, not the enum: **belief rows** (`s_r_pairs`, `facts`, `questions`) are `owner` or the run fails G12(e) — a `1` there is a bug, and a `2` there is the same bug wearing a third name. `2` is legal **only** on `links` (bridges and §5.2's `6 cooccur-weak` candidates, [docs/07 §7.3](../docs/07-read-path.md) mechanism 2/4), where it is never FACT-eligible and never answerable. **The v2 arc adds no fourth value.** [docs/26 §26.2](../docs/26-world.md) says the field "gains its class-leg" under tier (b), but §26.2's `speaker_class` is *derived, stored nowhere*, so the class is an input to the write gate, not a byte in the row: a granted trusted teacher has **no representable value** here, `si-learn` refuses the belief write (`Refused{SpeakerNotRepresentable}`) and G19(c) grades that refusal (**OD-135**, [12 §10](12-v2-arc.md)). Tier (a) — the shipped default — never puts identity in a row at all, so this field's value set stays `{0,1,2}` and `check_record_layout`/G5c's schema diff stay empty |
| 35 | `stage` | `u8` | 1 | — | the stage S0–S3 **derived at write time**, stored as provenance only; stage is re-derived each tick (docs/11), never read back as state |
| 36 | `bind_output_id` | `u32` | 4 | **`0` = null** | the output the correction bound to; `0` for `/teach` (nothing preceded it, §1.4) |
| 40 | `question_id` | `u32` | 4 | `0` = null | set iff `prov_kind = answer-to-ask`; **stays null for a teach even when the teach resolved an ask** (G8(f)) |
| 44 | `prov_session` | `u32` | 4 | — | interned session id |
| 48 | `prov_step` | `i32` | 4 | — | provenance step (the citable teaching act) |
| 52 | `text_ref` | `u32` | 4 | `0` = none | interned verbatim source text |
| 56 | `edges_off` | `u32` | 4 | `0` = empty | arena offset of the `z` list (§5) |
| 60 | `charge_off` | `u32` | 4 | `0` = no stamps | **alias of `z[0]`'s stamp list** — see §5.3 |
| 64 | `derived_off` | `u32` | 4 | `0` = none | `derived_from` list (fact rows only) |
| 68 | `derived_len` | `u16` | 2 | — | |
| 70 | `edges_len` | `u16` | 2 | — | |
| 72 | `terms_off` | `u32` | 4 | `0` = empty | arena list of `u32` point handles (stimulus+response terms packed) |
| 76 | `terms_len` | `u16` | 2 | — | |
| 78 | `reserved` | `u16` | 2 | — | **asserted zero** — the reserved slot is checked, not trusted |

### 4.2 Body — 32 bytes, one per shelf (`unused = 0`)

| off | field | type | used by |
|-----|-------|------|---------|
| 0 | `pattern_id` | `u16` | `SR`, `FACT`, `Q` — the `P_*` enum index of the decoded pattern (R3 table). `SKILL` (B17): `0` for a placeholder-named skill, **`0xFFFF` = `NAMELESS`** for a minted one (§11) |
| 2 | `key_field` | `u16` | `FACT` slot id (`pattern-slot` \| `linked-term`) · `LINK` kind · `Q` kind+origin packed · `SKILL` kind (`7` = `SKILL_SHAPE`, high byte `0`) |
| 4 | `key_point` | `u32` | entity / `term_a` / the question's target token handle · `SKILL`: the shape's **root role** point handle |
| 8 | `val_point` | `u32` | response / `term_b` · `SKILL`: `0` (a move has no slot) |
| 12 | `stim_signature` | `u32` | interned **normalized** signature (folded, lowercased) — the boring-input key (§6.2) · `SKILL`: the interned **canonical role-graph template** (a shape *is* a normalized signature) |
| 16 | `stim_surface` | `u32` | interned raw stimulus surface · `SKILL`: the same id (a skill has no raw stimulus) |
| 20 | `resp_surface` | `u32` | interned response surface — the renderer may only use tokens ∈ this + learned chunks (§7.5) · `SKILL`: the minted **name** (the owner's word, else `skill:{hash(role_graph)}`), so the existing containment law alone makes an unnamed skill unspeakable |
| 24 | `target_slot` | `u16` | `Q`: the named dark slot id (from the owner's own decoded tokens). `0` for a nomination ask (kinds 5/6 name no slot) and for `SKILL` |
| 26 | `arousal_at_ask` | `u8` | `Q` only: `255` = none. **Audit evidence, never a cause** — nothing gates on it (OD-22). **`255` is mandatory for a nomination ask and for `SKILL`**: the detector has no edge to the crate that holds `a`, so the row itself carries the I8 assertion (§2.4 below) |
| 27 | `status` | `u8` | `Q`: 0 open · 1 resolved · 2 expired · `SKILL`: 0 live · 1 retired (superseded, never erased — I4); usage counters are **derived**, never stored (OD-120) |
| 28 | `body_reserved` | `u32` | 4 | **asserted zero** |

`envelope + body = 112 bytes` fixed, `% 4 == 0` asserted, and 112 is the number
every estimate in [06](06-resource-budget.md) is derived from. **The v2 arc adds
no column and no byte**: a skill row and a cluster row are readings of these ten
columns ([12 §2.3](12-v2-arc.md) gives the skill table cell-by-cell with the
reason each is representable today).

### 4.3 The `Q` kind and origin bytes, including the arc's values (no new column)

`key_field` on `questions` is `(gap_kind << 8) | origin`.

| byte | pre-arc values | arc values (B14/B17) |
|------|----------------|----------------------|
| `gap_kind` (high) | 1 `G1 defined_by` · 2 `G2 slot-empty` · 3 `G3 no-link` · 4 `G4 contradiction` | **5 `cluster-nomination`** · **6 `move-nomination`** |
| `origin` (low) | 0 `sense` · 1 `idle` | **2 `nomination`** |

Three consequences, each of which is a check rather than a note:
* **`origin = 2` *is* the lane.** [docs/26 §26.13](../docs/26-world.md) says every
  nomination ask "carries a lane field"; the field is this byte, **derived**, never
  stored separately — which is what keeps the 80-byte envelope untouched.
  `check_nomination_owner_lane` asserts the only sink of an `origin = 2` gap is the
  owner lane (OD-128).
* Kinds 5/6 are unanswerable as **targets** (`target_slot = 0` names no slot) but
  answerable as **asks**: an answer-to-ask commit whose `question_id` names kind 5
  mints the `links` cluster row — [docs/22 §22.4](../docs/22-crystallize.md)'s whole
  point, and the reason a nomination is not a mute observation.
* `Q_OPEN/RESOLVED/EXPIRED` are unchanged; a candidate that closes without an owner
  answer leaves its ask `open` under the normal lifecycle.

## 5. The `z` edge and the charge stamp

### 5.1 Edge record — 16 bytes

| off | field | type | null / meaning |
|-----|-------|------|----------------|
| 0 | `to_point` | `u32` | target hyperpoint handle |
| 4 | `kind` | `u8` | enum §5.2 |
| 5 | `flags` | `u8` | bit0 `born-this-step` · bit1 `frozen-genome` (L1/L2 edges: **no `bake`/`charge` field exists**, §5.3 law (a)) · bit2 `history-only` |
| 6 | `weight` | `u16` | Q12 live weight; `0` ⇒ superseded ⇒ cannot conduct at NOW |
| 8 | `bake` | `u16` | Q12, **write-once and permanent**, bounded by `B_max`; `0` ⇒ unbaked (OD-56) |
| 10 | `pad` | `u16` | asserted zero |
| 12 | `charge_off` | `u32` | arena offset of this edge's stamp list; `0` = none |

### 5.2 Edge kinds (closed enum — adding one is a design change, not a build choice)

`0 owned-attribute` · `1 owned-definition` · `2 link-means` ·
`3 link-same-question` · `4 link-frame-synonym` · `5 link-slot-unique` ·
`6 cooccur-weak` (candidate only, never FACT-eligible) ·
`7 genome-hypernym` … `15 genome-*` (L2, verbatim ConceptNet/WordNet rel names —
**no relabeling**, [docs/10 §10.3](../docs/10-configs-converters.md)).
Kinds 0–6 are the only ones an OWNED edge may hold; `4 frame-synonym` is the
edge G2's decode barrier is about, and it is created **only** by a `/correct` or
`P_DEF` event with step provenance (R-18: an unprovenanced frame edge is a
factory default and fails G5c).
**On a frozen edge the byte is a slot, not a name:** `7…255` index the
version-tagged **genome kind table** sealed inside `SIgen`, whose entries are the
source's own relation strings, listed and never mapped
([07 §6](07-genome-build-plan.md), **OD-62**) — the nine advertised slots cannot
hold a corpus's vocabulary verbatim, and choosing nine of them would be the
relabeling docs/10 forbids.

**The v2 arc adds no edge kind, and `EDGE_KIND_COUNT` does not move.** A cluster
point and a skill are **rows**, and a row's kind lives in the body's `key_field`
byte — a per-shelf namespace, distinct from this one (§4.2 documents the column
shelf-by-shelf). So the arc's values are `links.key_field = 8` (`CLUSTER_POINT`) and
`skills.key_field = 7` (`SKILL_SHAPE`), each inside its own shelf's byte, while kinds
`0…15` here stay exactly as written. The two namespaces are why
`check_record_layout` and `check_teach_literals` stay green with the arc installed
and G5c's edge-kind diff stays empty ([12 §2.5](12-v2-arc.md)).

### 5.3 The stamp — 16 bytes, and the row/edge alias

| off | field | type | meaning |
|-----|-------|------|---------|
| 0 | `thought_id` | `u32` | the thought that stamped it (citation) |
| 4 | `signal_ref` | `u32` | interned citation string (`"surprise@6.2=1.0"`, `"queue_age(q1)"`) |
| 8 | `magnitude` | `u16` | Q12 — **the firing signal's own value**, never `f(a)` (OD-22 for this field stands) |
| 10 | `a_at` | `u8` | 1/255 — arousal at formation (OD-28). Two jobs: the read-side state-match argument for `σ`, and the **argument of the curve that sized this edge's bake** (§20.4a). **Neither makes it a multiplier** |
| 11 | `signal` | `u8` | one of §18.1's **six** raw signals, by number: 0 novelty-spike · 1 realized-gain · 2 contradiction · 3 settle-failure · 4 open-question-age · 5 idle-cascade-contradiction. Round 1's `resident` column is **superseded** (OD-23); `sign` is **dropped** (OD-18/OD-31) |
| 12 | `src` | `u8` | 0 write · 1 re-weight · 2 idle-re-light (OPEN-24) · 3 reparse-inherited-history |
| 13 | `pad` | `u16` | asserted zero |
| 15 | `pad2` | `u8` | asserted zero |

**The alias rule, stated because docs/05 shows `bake`/`charge` on the row and
docs/05 §5.3 shows them on the edge:** every belief row has exactly **one
primary OWNED edge** (`z[0]`, stimulus→response), so a row's `bake`/`charge` in
the JSONC export are printed from `z[0]`. Storage keeps them on the edge record
only — there is no duplicate copy, and the exporter's `charge_off` is the same
arena offset `z[0]` holds. **Consequence the build must not lose:** a row whose
primary edge is superseded keeps its bake and its stamps (I4) while its live
`weight` goes to 0; "durable ≠ currently active" (§20.4) is a read filter, not a
write.

**Bake is write-once.** `bake` is set at the formation instant through
`B_max · tanh(k · a_formation)` and **no code path may recompute it later**; a
stored-value re-read from a later `a` fails G9's static check (§14 R-23), and a
`bake` derived from an **event-kind table** rather than the one integrated
number fails the anti-cheat law (OD-31, R-21(c)).

## 6. Per-shelf body notes (beyond the envelope)

**Frozen shelves do not carry the envelope.** The 112 B form (§4) exists to hold
mutable session state — `confidence`, `repeats`, `last_confirmed_at`,
`superseded_*`, `charge_off`, `text_ref` — and at `t = 0` every one of those is
either constant or meaningless. Shelves 1–3 therefore ship **packed, immutable
records of their own** (`DICT 32 / REL 16 / RULE 16`, all `% 4 == 0`), addressed
`(0, x, y)` under the same one scheme; the registry entry's `row_schema_version`
is what licenses a per-shelf layout, and the addressing law
([docs/05 §5.1](../docs/05-substrate-tesseract.md)) is about `(t,x,y,z)`, not
about record width. Field lists, byte tables, the index space and the reason
(8–12× the genome's resident bytes) are in
[07 §11](07-genome-build-plan.md). A frozen record has **no** `bake`, `a_at` or
`charge_off` column, so a stamp on a genome edge is a missing field, not a policy
violation (§5.3 law (a) made mechanical, [07 §9](07-genome-build-plan.md)).

- **`DICT` (L1, body 32 B, read-only):** `{hyperpoint_id u64, lemma interned
  u32, pos u8, forms_off u32, forms_len u16, synset_id u32, version_tag u32,
  provenance {source interned, license interned}}`. **No charge field exists on
  this type at all** — a stamp on a genome edge is unrepresentable, not merely
  forbidden (§5.3 law (a), G9(d)).
- **`REL` (L2):** `{a_point u32, b_point u32, edge_kind u8, weight u8,
  version_tag u32, provenance_off u32}`. Hypernym chains are rows here, not
  fields — they are correlations ([docs/10 §10.2](../docs/10-configs-converters.md)).
- **`RULE` (L2):** `{triggers_off u32, triggers_len u16, operator u8,
  results_off u32, results_len u16}`; composable to depth 1 in v1, cycle-checked
  at load (ENGINE).
- **`CTX` (context):** body carries only `stim_surface` + `terms_*` and
  `prov_speaker`; **`charge_off = 0` asserted and `edges_off = 0`** — raw
  utterances are the substrate's control group, so affect cannot launder itself
  into evidence (§5.4).
- **`LINK`:** `key_point`/`val_point` = `term_a`/`term_b`, `key_field` =
  `means | same-question | slot-unique | frame-synonym`; carries the full
  envelope including charge/bake, because a bridge **is** an OWNED edge. A
  `6 cooccur-weak` candidate row lives here too with `confidence = 0` and
  `prov_speaker = 2` (**OD-59**, [04 §21 U.4](04-algorithms.md)); the type is
  representable today and the promotion path that would write it ships off.
- **`FACT`:** identical envelope, `derived_off ≠ 0` mandatory, and its primary
  edge's `bake` **equals baseline (0)** with a non-empty inherited stamp list —
  G7's "re-parse may not manufacture a fresh boost" is two asserted field
  values, not a review note (§20.4).
- **`Q` (questions):** `key_field = (gap_kind << 8) | origin` — **kind in the high
  byte, origin in the low one** (`si-learn::gate` line for line; §4.3 lists the
  value sets, the arc's 5/6 and 2 included). An `origin: idle` row is an **agenda**
  item pushed by a cascade that never emitted it (§19.4) — the shelf's dual role
  needs no extra column; an `origin: nomination` row is the same trick with the
  detector as the asker, and its lane is derived from that same low byte.
- **`SKILL` (skills, shelf 9 — live at B17):** the body is read column-by-column in
  [12 §2.3](12-v2-arc.md) and adds no column. Envelope: `prov_kind = 3
  answer-to-ask`, `prov_speaker = 0`, `question_id` = the `move-nomination` ask row,
  `confidence = 95`, `gain_kind = 1 demonstrated` (by §8.2's own mapping, which is
  [docs/25 §25.11](../docs/25-skills.md)'s reasoning), `text_ref` = the owner's
  verbatim naming line. A skill row is **an ordinary belief row with an ordinary
  commit**: `si-skills` cannot write it, only nominate it (structural win 7,
  00 §2).
- **`CLUSTER` is not a shelf or a type:** it is a `links` row with
  `key_field = 8 cluster-point`, `prov_kind = 3`, `question_id` = the
  `cluster-nomination` ask and the member terms in `terms_*` — §22's minted concept
  lands as a bridge row, the type the shelf already is.
- **Teach rows are not a new type** ([docs/05 §5.4](../docs/05-substrate-tesseract.md)):
  `SR` with `prov_kind = owner-teach`, `bind_output_id = 0`,
  `gain_kind = asserted`, `stim_signature` from the **decoded** left side (never
  the raw text), `question_id = 0`. See [09](09-teach-channel.md).

## 7. Interning

| rule | value |
|------|-------|
| id space | one `u32` table per session for all strings; `0` = `""`, `0xFFFF_FFFF` = NULL |
| identity | the string is stored **verbatim** (context surfaces keep the owner's casing); the *lookup* key is the folded, lowercased, plural-folded form (R1–R2) |
| aliasing | `clas`(s) collisions create **alias candidates**, visible in the lexicon export as an `aliases: [u32…]` side list; an owner `P_DEF` splits them (R-04) — the merge is never silent |
| who writes it | every arriving token, from **either** voice — `context` needs it for replay; but only a `source: owner` teaching event makes a token **taught**, and "taught" is the layer-1 reference the supporter's filter tests against (§21.3) |
| term record | `{term_id, surfaces[], appears_in[], bridges[], defined_by}` — `defined_by = NULL` is gap **G1** (§7.1), so the null encoding is load-bearing, not cosmetic |

## 8. Two more stored streams, because they are not rows

| stream | record | fields |
|--------|--------|--------|
| **rehearsal/decay curve** (§8.4, per edge) | 8 B | `{edge_ref u32, last_touched_at i32}` — the curve value itself is **derived** from `clock`'s recency function, never stored, so it cannot drift out of sync with the one clock (R-29) |
| **per-thought `a` trajectory** (§20.1) | `n × 1 B` | the ordered sequence of `u8` arousal values, one per signal event, **plus** the round each applied at. This is the hashed artifact (§16.6); the endpoint alone is *not* enough — a replay that reproduces the final `a` but not the trajectory has failed G5(a) |

## 9. Serialization: three formats, strictly separated

| format | used for | contract |
|--------|----------|----------|
| **binary containers** (`SIlat` lattice dump, `SIgen` genome, `SIck` checkpoint, `SIst` state snapshot, `SIro` one framed stream record) | the machine's own words | `MAGIC \| version u32le \| header_len u32le \| header JSON \| payload` (LoopNet `ckpt.rs`); format read from magic, never the extension; header carries `{shelf → row_schema_version}`, grid tags, genome version tag, **the §13 core block** (`birth_seed`, `seed_format`, the vector digest, the sleep counters) |
| **JSON / JSONC** (`mindlog.json`, `monologue.jsonl`, `gate_report.json`, `transcript.txt`, exports) | humans and gates | export-only. **No run may read a JSON back as build input** — a JSON that re-enters the loop is a channel, which is R-33's failure shape |
| **framed `.siro` append-only streams** (`stream.siro`, `sense_inbox.siro`) | process transport ([00](00-stack-workspace.md) §7) | each frame = one `SIro` record `{seq u64, t i32, source u8, kind u8, body}`; writer appends, reader tails an offset; **the loop never blocks on a reader** (OD-48) |

**The arc's only byte is a transport byte.** A sense frame arriving from the
panel, from Grimm or from the scenario carries, from **B18**, two more body
fields: `speaker_identity u64le` and `conversation u32le` (interned thread id).
That bumps the `SIro` container **version** (a resume into an older-versioned
inbox refuses, per [00 §5](00-stack-workspace.md)'s mismatch law) and touches
nothing else: no envelope byte, no row, no hash item — identity is metadata on
*sense*, exactly [docs/26 §26.13](../docs/26-world.md) rules, and the class is
re-derived from it per line rather than stored
([docs/12](../docs/12-component-map.md) rule (ix)). `conversation` is what makes
[docs/23](../docs/23-context.md)'s per-thread quiet and silt's per-thread table
addressable at all ([12 §2.6](12-v2-arc.md)). Because the fields live in the
recorded sense stream, **G19(e)**'s identity-tagged replay is a replay of the same
bytes, not a re-reading of a transcript file.

## 10. State-hash composition — exactly these bytes, in this order

`md5` over the concatenation of, in order:

1. the lattice: every live row's envelope+body, sorted by `(x, y)`;
2. every `z` edge record (including `bake`, `weight`, `kind`) — **sorted**;
3. every charge stamp (including `a_at`, `magnitude`, `signal`);
4. the lexicon + interner tables (id → folded bytes), sorted by id;
5. the settle-set of the last thought as `{(point, quantized_activation)}`,
   sorted by point id — **quantized grid, never raw floats** (§16.6);
6. the full **`a` trajectory** of every thought this session, event-ordered;
7. the ignition PRNG state `{state, inc, spare}` (LoopNet's serializable
   `RngState`) and its advance history (§19.4);
8. the version tags: modulation curve (OPEN-16a) **and its selected variant
   index** (**OD-74**), lag constant (OPEN-29), genome `t=0` tag, **and the
   inference depth cap `INFERENCE_MAX_HOPS_OWNED`** —
   it gates no byte of state, but it decides which label a settle gets, and a
   session that replays with different labels is a G5(a) failure even with zero
   state drift ([04 §5.4](04-algorithms.md)).
9. **the core block** (§13): `birth_seed u64`, `seed_format u16`,
   `DERIV_ORDER` length, the digest of the derived temperament vector, and the
   sleep counters `{sleeps_completed, sleeps_interrupted, last_slept_at_t}`.
   A session's *arithmetic* is only attributable if its individual is named: an
   un-hashed seed would let two different minds share a hash, and an un-hashed
   sleep counter would let a replay skip a rescale and still match.

**Deliberately absent** — each omission is an assertion, not an oversight:

| excluded | why | tested by |
|----------|-----|-----------|
| per-point **median** | derived from rows already in 1–3; storing it would be a frozen emotional address (**R-28**) | G5(a) + G9(c) |
| the **derived temperament vector**, as stored values | item 9's seed *is* the vector: it is a pure function of the seed and `DERIV_ORDER`, re-derived at wake before the first round of the first thought. A second copy in state would let a container carry values its seed does not justify ([10 §4](10-temperament-sleep.md) P.3) | **G14(c)** (re-derive and byte-compare) |
| the **calm view** produced by sleep's P4 | a read-side statistic over rows already in 1–3, terminal by construction: it may not feed a weight, a floor or the sampler ([10 §8](10-temperament-sleep.md) law 1) | **G13(k)**, `check_calm_view_is_terminal` |
| audit **labels** | output, never state; deleting `audit.py` must change no hash byte except the label strings in the log | G9(c) (R-24) |
| wall-clock, `--config` flag, viewer state, Grimm's own process state | I5; a replay is over the **recorded sense stream**, and every admitted `source: grimm` line is already inside item 1's `context` rows (§21.7) | G12(a) |
| raw floats anywhere | determinism annex (§16.6); GPU parity is claimed **within the grid**, never bit-exact | G5(a), G6 |
| the v2 arc's **candidate tables** (crystallize, skills) | declared hash-**exempt** session state (OD-98/OD-117), so `CRYSTALLIZE` on/off over one transcript yields byte-identical items 1–9 and the audit trail is `crystallize.jsonl`/`skills.jsonl`, not a hash byte | **G15(a)**, **G18(a)** |
| the **silt table** | exempt by the same ruling (OD-103): per-tick RAM over point handles, written to `silt.jsonl`; item 7's ignition history still captures every *choice* silt influenced | **G16(a)**, `check_silt_writes_nothing` |
| **ghosts, role-graphs, the co-light log** | frame-scoped scratch, constructed and destroyed inside `si-walk` ([docs/16 §16.8](../docs/16-think-walk-hardware.md)); a stored one would be a frozen address of a thought that already ended (R-28's shape) | `check_ghost_is_frame_scoped`, **G15(b)** |
| **event views** and the `prov-link` index | derived at frame load from items 1–3 (OD-110 ships *derived*, so shelf 11 stays reserved-empty) | **G17(b)**, `check_prov_link_not_stored` |
| **skill usage counters** | derived over rows + the log (OD-120), the per-user-median family's rule ([docs/26 §26.6](../docs/26-world.md)) | `check_skills_no_commit` |
| **`speaker_class`** | a **function, not a field** ([docs/12](../docs/12-component-map.md) rule (ix)): a class that survived its grant would be authority outliving the owner's act | **G19(b)**, `check_speaker_class_derived` |

**The one way the arc does move the hash:** item 8 is the version-tag list, and
`VersionTags::encode` writes the registry's **row count** with every
`(name, ver)` pair. 22 new rows therefore re-baseline item 8 for every run in the
repository — once, at B14, with `CONSTS_VERSION` 0x0001 → **0x0002** and the
pinned empty-state digest re-recorded in the same commit
([12 §3.3](12-v2-arc.md)). No item is added: the count stays **nine**.

## 11. Null / sentinel table (the whole set, so a build has no local convention)

| field | "absent" | why that value |
|-------|----------|----------------|
| `superseded_at` | `i32::MAX` | makes `active_at(s)` one comparison |
| `superseded_by`, `bind_output_id`, `question_id`, `derived_off`, `edges_off`, `charge_off`, `text_ref` | `0` | `row_id 0` is never allocated, so 0 cannot alias |
| `bake` | `0` | a real value: unbaked (OD-56), and `B_max·tanh(0) = 0` |
| `weight` | `0` | superseded ≠ deleted (I4) |
| `arousal_at_ask` | `255` | 0 is a meaningful level (rest) |
| `a_at`, trajectory points | *no null* | a stamp always records the live level |
| `defined_by` (term record) | NULL | gap detector G1 reads exactly this |
| point median, emotion label, mood | **no field exists** | §20.3 / R-28 — unrepresentable, not disallowed |
| reserved/pad columns | must be `0` | asserted on every read: a non-zero reserved byte is a schema-version lie |
| body `pattern_id` = **`0xFFFF` `NAMELESS`** (B17) | the *name* is absent, the point is not | an un-named minted skill or cluster is **addressable and reason-over-able but unspeakable** (OD-100, G15(f), G18's twin). `0` cannot serve — `0` is a real pattern index — and the body's `u16` has 65535 unused because the pattern table is 0…9. The renderer refuses any surface whose row carries it, which is why no new render rule is needed |
| `SKILL.val_point`, nomination `Q.target_slot` | `0` | the ordinary absent-handle: a move has no response slot and a nomination names no slot (§4.3) — **this is what makes kinds 5/6 unanswerable as targets while staying answerable as asks** |
| `SKILL.arousal_at_ask`, nomination `Q.arousal_at_ask` | `255` | mandatory, not merely legal: the detector has no edge to the crate holding `a`, so the row itself is the I8 assertion (§4.3) |

## 12. Multi-line paste at the panel (OD-54)

A paste of *k* lines is *k* logical steps in **order**, each its own `context`
record and its own sense event, `t` increasing by 1 per line. There is no
"batch" record: a chunk of pasted text is one step per newline so that
arbitration, surprise, the `a` trajectory and the replay all keep the same
granularity they have for typed input. A line starting `/teach` or `/correct`
inside a paste is still that line's own act.

## 13. The session core block — the individual and its bookkeeping

Not a row, not a shelf, not a cache: a fixed block in the `SIck`/`SIst` header,
**inside hash item 9**, and the only place the TEMPERAMENT/SLEEP batch stores
anything. Its existence is what makes "the seed is in core forever, version-
tagged, inside the state hash beside genome version + t=0" a byte layout instead
of an intention.

| off | field | type | width | null / meaning |
|-----|-------|------|-------|----------------|
| 0 | `birth_seed` | `u64` | 8 | the individual. **`0` is a legal seed**, so there is no sentinel; a container with this field unreadable aborts the run rather than defaulting ([10 §3](10-temperament-sleep.md)) |
| 8 | `seed_format` | `u16` | 2 | `0x0001`. Bumps whenever `DERIV_ORDER`, the draw count, the mapping or a grid changes ⇒ every hash changes ⇒ no old session can be re-tempered by new arithmetic |
| 10 | `deriv_order_len` | `u8` | 1 | the count of drawn **values** (`18` per [03 §1.2](03-constants-registry.md)); asserted equal to the registry by `check_temperament_closed` |
| 11 | `pad` | `u8` | 1 | asserted zero |
| 12 | `temperament_digest` | `u64` | 8 | md5 prefix over the canonical 18-value byte string (fixed order, fixed grids). A **digest, not the vector**: the vector is re-derived, and this is the byte-comparison target of G14(c) |
| 20 | `sleeps_completed` | `u32` | 4 | full four-pass sleeps. A counter, never a clock reading (I5) |
| 24 | `sleeps_interrupted` | `u32` | 4 | aborted at a pass boundary by a pending sense frame ([10 §7](10-temperament-sleep.md)) |
| 28 | `last_slept_at_t` | `i32` | 4 | `0` = never slept; drives the cadence so a **resume cannot repeat a sleep** over content it already consolidated |
| 32 | `sleep_pass_mask` | `u8` | 1 | the passes completed by the last, unfinished sleep (bit0 P1 … bit3 P4); `0` when the last sleep completed. This is the record that makes a **partial sleep legal** (**OD-79**) |
| 33 | `pad2` | `u16` | 2 | asserted zero |
| 35 | `block_version` | `u8` | 1 | `0x01` — the block's own schema tag, because §11's rule is that a reserved byte is *checked, not trusted* |

**36 bytes**, `% 4 == 0` asserted like every record here, which is also the whole
storage cost of individuality plus sleep bookkeeping ([06 §3.4](06-resource-budget.md)).

**Two laws about the block, because it is the one place this batch could have
smuggled state.** (i) It holds **no affective value**: no `a`, no bias, no
median, no floor per edge, no derived temperament number. The derived vector is
recomputed from `birth_seed`; the floors are registry constants; `a` is
session state hashed in item 6. (ii) The block is **read-only to the loop** except
the sleep counters, which `si-sleep` alone advances, and the seed fields, which
nothing advances after creation — so the resume-refusal rule
([00 §5](00-stack-workspace.md)'s atomic-write idiom, [07 §13](07-genome-build-plan.md))
has exactly one writer to check.
