# 12 — The v2 arc: HOW for G15–G19 (crystallize · silt · episodic · skills · world)

Provenance. Phase 1's last artifact. The law is
[docs/22](../docs/22-crystallize.md)–[docs/26](../docs/26-world.md); this file turns
those five files into crates, bytes, call sites, registry rows, algorithms, checks
and stages — and it is written against one standing requirement the owner stated
in the same breath as the drafts: **engineer them thoroughly so nothing we already
made breaks.**

This file adds no law, no organ, no door, no signal, no label, no shelf, no state
byte and no vocabulary word. It is the **HOW** the way [00](00-stack-workspace.md)–[11](11-build-plan.md)
are, and where it disagrees with `docs/22–26` the docs win and this file is the bug.

**Reading order.** §0 is the summary a reviewer needs; §1–§7 are the deltas to
engineering 00–06; §8 is the stage table; §9 is **the break-list** — every
pre-existing artifact the arc moves, and the stage that moves it; §10 is the OD
ledger delta; §11 is what this round refuses.

---

## 0. The eight facts that decide everything else

| # | fact | consequence |
|---|------|-------------|
| 1 | **Three new crates, thirteen → sixteen packages.** `si-crystallize`, `si-silt`, `si-skills` — all three **reader-only leaves** (core, store; nothing else). Episodic (G17) and world (G19) add **no crate**: G17 is a derived view in `si-read` plus an argument in `si-walk`, G19 is a pure function in `si-core` plus a tier arm in `si-learn`. | 00 §2's "Thirteen crates", `CRATE_NAMES: [&str; 13]`, 11 R1 and B0/B13's `cargo metadata` count all move (§9 row 1). |
| 2 | **No new envelope byte, no new record width, no new `prov_kind` literal, no new edge kind, no new shelf id.** `RECORD_FIXED_BYTES` stays 112; the five `prov_kind` values stay five; the seven OWNED edge kinds stay seven; shelf 9 (`skills`) and 11 (`events`) were already registered ids. | `check_record_layout`, `check_string_constants`, `check_teach_literals` and G5c's schema diff stay **empty** — the strongest available proof that the arc is not a schema change. |
| 3 | **Everything the arc *proposes* is a value on the shelf the design already has.** A nomination is a `questions` row with two new **values** in `key_field`'s high byte (kinds 5 · 6) and one new **value** in its low byte (`origin = 2`); a cluster point is a `links` row; a skill is a `skills` row; an event view is derived from `context` rows + stamps; silt is session RAM; a ghost is frame RAM. | 05 §5.1's "a new knowledge type is a new kind inside the same schema, never a new structure" holds without a bend. |
| 4 | **Two tables in RAM, four logs on disk, zero hash items.** Candidate tables (crystallize, skills), the silt table, ghosts and role-graphs are **declared hash-exempt session/frame state** (OD-98/103/117 as folded) and are appended to `crystallize.jsonl`, `silt.jsonl`, `skills.jsonl` and `mindlog.json`'s new view blocks. | The hash stays nine items. `check_no_hash_item_added` is not needed because the item count is a `const` the layout test asserts. |
| 5 | **The arc's decay is the one curve.** Silt's per-tick erosion is `Clock::recency` scaled by `LAMBDA_SILT` **as a share**, exactly the treatment `DIM_BASE` already gets (03 §5.1, 03 §9's "a dimming rate of its own is forbidden"). | `check_one_clock` widens from `si-affect` to `si-affect` **and `si-silt`** and still finds one recency implementation in the build (§9 row 6). |
| 6 | **The fifth ignition weight is SPECIES and ships unarmed at 0.** A 19th temperament draw would lengthen `DERIV_ORDER`, bump `seed_format` and re-hash every run since B0; OD-108 is open, so it cannot be a draw. | Every pre-arc gate replays byte-identical with the row present, because a 0 term adds exactly 0 to the Q12 sum. |
| 7 | **Adding registry rows re-baselines hash item 8.** `VersionTags::encode` writes `CONST_ROWS.len()` and every `(name, ver)`, so 22 new rows change every `state_hash.txt` in the repository — once, at one stage, with a version bump. | `CONSTS_VERSION` 0x0001 → **0x0002** and the three pinned `EMPTY_STATE_MD5_SEED_ZERO` literals re-pin, all in **B14** and nowhere else (§9 rows 2–3). |
| 8 | **Three of the arc's five gate families grade a *refusal*.** The trusted tier cannot write a belief row because `prov_speaker` has no value for a granted teacher (**OD-135**); `gates/G15/G16/G17/G18` sweeps publish **refusals** while OD-83 is unrulled; no `/trust` parser ships because the vocabulary is closed (**OD-136**). | A leg that would be vacuous becomes a leg over an absence. The refusal is the pass condition, and it is *named* in the report — never a silent skip. |

---

## 1. Delta to [00](00-stack-workspace.md) — the crate graph, the edges, the layout

### 1.1 The three new rows of the graph

```
si-crystallize ──► core, store ONLY   ◄── NO si-learn, NO si-affect, NO si-walk
si-silt        ──► core ONLY          ◄── NO si-store, NO si-learn, NO si-affect
si-skills      ──► core, store ONLY   ◄── NO si-learn, NO si-affect, NO si-walk
```

| crate | logical `*.py` id (OD-43) | may depend on | may NEVER depend on (the law it makes mechanical) |
|-------|--------------------------|---------------|---------------------------------------------------|
| `si-crystallize` | `crystallize.py` | core, store (read-only) | **`si-learn`** — a detector that could commit is a self-teaching ontology; **`si-affect`** — §22.3's first exclusion ("`a` is NEVER read by the detector") stops being a sentence and becomes a missing edge; `si-walk` — it consumes the frame log *as an argument*, never by reaching into the walk |
| `si-silt` | `silt.py` | core (`clock`, `quant`) | **`si-store`** — silt tracks point **handles**, not rows, so it cannot read or write a belief; **`si-learn`**, **`si-affect`** (OD-107: arousal-scaled deposit/decay is *absent from the code*, not zero); `si-walk` — the table is handed to `ignite` as an argument |
| `si-skills` | `skills.py` | core, store (read-only) | **`si-learn`** — §25.11's note 2 verbatim: "*executing* a skill would require an edge the build graph does not contain"; **`si-affect`** (§25.4 rule 3: the detector reads outcome **rows**, never `a`); `si-walk` — the ordering preference is an argument |

**Reciprocal edges, and they matter as much:** `si-walk ↛ {si-crystallize, si-silt,
si-skills}`, `si-learn ↛` the three, `si-front ↛` the three, `si-read ↛` the three.
Nothing in the thought path, the write path or the surface can *ask* the arc for a
decision; only `si-session` — which already imports everything in the loop — calls
them, and only at the four tick steps §4 names.

### 1.2 Structural wins 5–7 (the same trick, three more times)

00 §2 lists four wins. The arc adds three, all of the same shape as win 4
(`si-sleep ↛ si-learn`): a law stated in prose becomes a fact `Cargo.toml` refuses.

5. **"Crystallize may nominate, never mint" is a missing edge.** With
   `si-crystallize ↛ si-learn` the crate has no symbol through which a row, an
   edge, a bake or a confidence byte could move; its only output is a `Nomination`
   value that `si-session` turns into an ordinary `drive::Gap`. G15(a)/(b)/(d) and
   §22.0's litmus are `check_crystallize_no_commit` plus this edge.
6. **"Silt writes nothing, not even an ask" is a missing edge** — and the sharper
   half is `si-silt ↛ si-store`: the table is keyed by `u32` handles, so silt is
   *physically unable* to name a row, a shelf, or a `context`/`questions` byte.
   §23.0's "the one mechanism of the arc that writes nothing at all, not even an
   ask" is a type-system fact.
7. **"A skill is a row, not a program" is a missing edge** (`si-skills ↛ si-learn`,
   `si-skills ↛ si-walk`). The only way a skill could be *executed* is an argument
   the walk accepts, and the argument is an ordering preference — which is OD-118's
   recommended half and nothing more.

**The three wins are why the arc is safe at all**, and they are why G15(b), G16(b)
and G18(b)'s deletion tests are *independent*: removing a reader cannot perturb a
writer, because no writer can name a reader.

### 1.3 Files the loop writes (00 §8 delta)

```
runs/<session-id>/
  crystallize.jsonl        # one line per candidate transition (open/recurrence/close/resolve) — OD-98's audit trail
  silt.jsonl               # one line per tick per active table: {t, conversation, [(point, level)]} — OD-103's audit trail
  skills.jsonl             # one line per shape-candidate transition — OD-117 rides OD-98
  mindlog.json             # + four new blocks: frame_log[], nomination_views{}, event_views{}, silt_view{}
```

The **eleven** gate artifacts, named exactly as
[gates/README](../gates/README.md) names them (that file is the authority; a
filename invented here is a bug): `G15/nomination_log.jsonl`,
`G15/recurrence_sweep.json`, `G16/silt_log.jsonl`, `G16/silt_sweep.json`,
`G17/relight_log.jsonl`, `G17/prov_sweep.json`, `G18/skill_log.jsonl`,
`G18/shape_sweep.json`, `G19/server_transcript.txt`, `G19/grant_trace.json`,
`G19/lane_log.jsonl` — the four `*_sweep.json` publish `status: refused` while OD-83
is unrulled ([05 §23.6](05-gate-harness.md)). No new `.siro` stream: `stream.siro`
stays one output per user line, and the per-thought views are log, not state
(00 §8's contract).

### 1.4 Process topology

Unchanged, with one sentence added: the Discord adapter, when it is built, is a
**fourth process writing `sense_inbox.siro` through `si-supporter`'s seam** — it is
a transport at the panel's edge, outside the loop (docs/26 §26.13's scope line), and
it is **not built here**. Tonight's multi-voice world arrives as a harness scenario
(OD-132), which is why G19 needs no network and no credentials.

---

## 2. Delta to [01](01-data-schemas.md) — bytes, kinds, and the absent things

### 2.1 What does not change (assert this first)

`RECORD_FIXED_BYTES = 112`, the 80-byte envelope's 78 offsets, the 32-byte body,
`EDGE_RECORD_BYTES = 16`, `STAMP_BYTES = 16`, `SHELF_COUNT = 11`, the five
`prov_kind` values, the `prov_speaker` value set `{0,1,2}`, the seven OWNED edge
kinds, and §11's null table **except the two additions in §2.5 below**. This list is
the arc's compatibility contract; `check_record_layout` and G5c are its judges.

**The one byte the arc does change is not a row byte.** From **B18** the framed
`SIro` sense record's body carries `speaker_identity u64le` and `conversation
u32le` ([01 §9](01-data-schemas.md)) — a *transport* field on sense, which bumps the
container **version** (and a resume into an older-versioned inbox therefore refuses,
per [00 §5](00-stack-workspace.md)'s mismatch law) while touching no envelope, no
shelf and no hash item. It is listed here rather than in §2.6 because a reader who
misses it will look for identity in a row and conclude the arc broke the contract.

### 2.2 Shelf 9 goes live (B17, with the skills build)

| `x` | shelf | layer | row body | mutability | from |
|-----|-------|-------|----------|------------|------|
| 9 | `skills` | L3 live | `SKILL` | append-only; retirement by supersede, never erase | B17 |

`RESERVED_SHELF_COUNT` 3 → **2** (`strategies`, `events` stay reserved);
`SHELF_NAMES` is unchanged — `"skills"` was always in the list, which is the whole
point of §25.11's note 2. `registry_is_valid()` still asserts ids `1..n` and the
reserved count, so `check_registry_size` reads "eleven entries, **two**
reserved-empty; a twelfth `x` fails the build (R-12)". Shelf 11 (`events`) stays
reserved: OD-110 ships *derived*, so nothing writes there (its row schema version
stays `0` and a non-zero write is a release-assert panic).

### 2.3 The `SKILL` body (32 bytes, one per shelf — reuses columns, adds none)

| off | field | value for a skill row | why it is representable today |
|-----|-------|----------------------|-------------------------------|
| 0 | `pattern_id` | **`0xFFFF` = `NAMELESS`** for a placeholder-named skill; else `0` | the body's `u16` has 65535 unused (the pattern table is 0–9); one sentinel makes G18's no-word-leak mechanical |
| 2 | `key_field` | `kind = 7 SKILL_SHAPE` in the low byte; `0` high byte | §4.2's `key_field` is per-shelf by contract ("`LINK` kind · `Q` kind+origin packed") |
| 4 | `key_point` | the shape's **root role** point handle (the seed role's most-recently-lit member) | a skill *is* addressable by a point; it must participate in the walk's handle space |
| 8 | `val_point` | `0` (no response term — a move has no slot) | §11: `0` = absent |
| 12 | `stim_signature` | interned **canonical role-graph template** (folded, e.g. `seed|lit_a|shared_filler:2`) | "interned normalized signature — the boring-input key" is exactly a shape key |
| 16 | `stim_surface` | same interning id (a skill has no raw stimulus) | keeps the render containment test meaningful: a skill's speakable surface is its *name*, not its shape |
| 20 | `resp_surface` | the minted name: the owner's word, or `skill:{hash(role_graph)}` | the renderer may only use tokens ∈ this — so an unnamed skill is unspeakable by the existing law, not by a new one |
| 24 | `target_slot` | `0` | — |
| 26 | `arousal_at_ask` | `255` = none | **OD-112/I8 mechanical**: a skill row never records an arousal level; 255 is the existing null |
| 27 | `status` | 0 live · 1 retired (supersede keeps the old row live-by-history per I4) | mirrors `Q`'s status byte; usage counters are **derived** (OD-120) |
| 28 | `body_reserved` | **asserted zero** | §11 |

Envelope: `prov_kind = 3 answer-to-ask`, `prov_speaker = 0`, `question_id` = the
`move-nomination` ask row, `confidence = 95`, `gain_kind = 1 demonstrated` (by
§8.2's own mapping — the same reasoning docs/25 §25.11 gives), `text_ref` = the
owner's verbatim naming line, `charge_off`/`derived_off` as usual for a belief row.
**A cluster point is the identical shape on shelf 6**, with `key_field = 8
CLUSTER_POINT` (the `LINK` shelf's kind byte gains **one** value, `8`; `7
SKILL_SHAPE` lives on the `skills` shelf's kind byte, not here — one value per
shelf, §2.5), `prov_kind = 3`, `question_id` = the `cluster-nomination` ask.

### 2.4 The two new ask values (no new column)

`key_field` on `questions` is `(gap_kind << 8) | origin`. Arc values:

| byte | pre-arc | arc adds |
|------|---------|----------|
| `gap_kind` (high) | 1 `G1 defined_by` · 2 `G2 slot-empty` · 3 `G3 no-link` · 4 `G4 contradiction` | **5 `cluster-nomination`** · **6 `move-nomination`** |
| `origin` (low) | 0 `sense` · 1 `idle` | **2 `nomination`** |

Consequences, each of which is a check rather than a note:

* **`origin = 2` decides the lane.** OD-128's "every nomination ask carries a lane
  field" is satisfied by the byte the row already has: the lane is `key_field`'s
  origin half, *derived*, never stored separately — which is what lets §26.13's
  80-byte-envelope sentence stay true. `check_nomination_owner_lane` asserts the
  only sink for an `origin = 2` gap is the owner lane.
* **`arousal_at_ask = 255` for both kinds.** A nomination is detector output and the
  detector cannot read `a` (it has no edge to the crate that holds it); 255 is the
  mechanical I8 assertion in the row itself.
* **Kinds 5/6 are unanswerable as *targets*** (they name no slot: `target_slot = 0`)
  but answerable as *asks* — an answer-to-ask commit whose `question_id` names kind
  5 mints a `links` row, which is the entire point of §22.4.
* `Q_OPEN/RESOLVED/EXPIRED` are unchanged; a candidate that closes without an owner
  answer leaves its ask `open` per the normal lifecycle.

### 2.5 Three rows added to [01 §11](01-data-schemas.md)'s null table (and only three)

| field | new value | why |
|-------|--------------|-----|
| body `pattern_id` | `0xFFFF` = **`NAMELESS`** | an un-named minted point/skill is **addressable and reason-over-able but unspeakable** (OD-100, G15(f), G18's twin). `0` cannot serve — `0` is a real pattern index — and the body's `u16` has 65535 unused because the pattern table is 0…9. The renderer refuses any surface whose row carries it, so no *new* render rule is needed ([04 §21 U.1](04-algorithms.md)) |
| `SKILL.val_point`, nomination `Q.target_slot` | `0` | the **ordinary** absent handle, used for a new reason: a move has no response slot and a nomination names no slot — which is exactly what makes kinds 5/6 unanswerable *as targets* while staying answerable *as asks* |
| `SKILL.arousal_at_ask`, nomination `Q.arousal_at_ask` | `255` | **mandatory, not merely legal**: the detector has no edge to the crate holding `a`, so the row itself is the I8 assertion |

The **row-body kind bytes** the arc introduces are `links.key_field` kind `8 =
CLUSTER_POINT` and `skills.key_field` kind `7 = SKILL_SHAPE` (with `skills`' high
byte `0`). They are *per-shelf kind namespaces*, a different enum from
[03 §5.2](03-constants-registry.md)'s seven **edge** kinds, which is why
`EDGE_KIND_COUNT` does not move and `check_teach_literals` /
`check_record_layout` stay empty — a reader who confuses the two tables will try to
add an eighth edge kind and break G5c for nothing.

### 2.6 Frame RAM, session RAM (declared, hashed nowhere)

| carrier | type | lifetime | in hash? | logged to |
|---------|------|----------|----------|-----------|
| frame co-light log | `Vec<CoLight>` where `CoLight { point: u32, role: Role, edge_kind: u16, round: u8, primed: bool }` | one walk frame | never | `mindlog.json` `frame_log[]` (per thought) |
| ghost | `{members: [u32], formed_in_frame: u64, frame_seed: u64}` ≤ `GHOST_MAX` | one frame, destroyed at settle (docs/16 §16.8) | never | the same block, `role = GHOST` |
| role-graph | `{roles: [Role; 4], traversed_kinds: [u16], filler_sets: u16}` ≤ `SHAPE_MAX` | one frame | never | the same block |
| crystallize candidate | `{members: [u32], shared_fillers: [u16], recurrence: u16, frames_seen: [i32], first_seen_t, last_seen_t}` | session, per conversation | **no** (OD-98) | `crystallize.jsonl` |
| shape candidate | `{role_graph_template: u32, filler_sets_seen: u16, recurrence: u16, outcome_tags: [u8; ≤4], first_seen_t, last_seen_t}` | session | **no** (OD-117) | `skills.jsonl` |
| silt table | `BTreeMap<u32 /*conversation*/, BTreeMap<u32 /*point*/, u16 /*Q12*/>>` | session, tick-decayed | **no** (OD-103) | `silt.jsonl` per tick |
| event view (derived) | `{event_id = context row's y, belief_rows: [u32], stamps: [u32], a_at: u8, t: i32, speaker_identity: u64}` | assembled at frame load, never stored | **no** — every field is already in items 1–3 | `mindlog.json` `event_views{}` |
| skill usage counters | `{times_consulted: u32, outcome_tags: [u8]}` — **derived** over rows + the log | session | **no** (OD-120, R-28 family) | `mindlog.json` |

`event_id` is the sense row's `y`, exactly as §24.2 says ("the event id IS the sense
row id"), so **no new id space** exists. `speaker_identity` lives in the framed
sense record's body and in `transcript.txt`, never in a row — that is OD-123's
shipped half, and it is the reason tier (a) leaves the envelope untouched.

---

## 3. Delta to [03](03-constants-registry.md) — 22 rows, one version bump

### 3.1 New swept SPECIES rows (16)

All sixteen are `Class::Species`, `sweep = Some(their own artifact)`, default
`—` = OD-86's midpoint placeholder tagged `UNMEASURED_PLACEHOLDER` **except**
`IGN_WEIGHT_SILT`, whose shipped default is a declared `0`. `hi` is the type
maximum wherever the row's box is open, because OD-83 is unrulled; every artifact
below therefore publishes a **refusal**, and a leg that cannot run without a value
runs at `lo` and names it a floor (OD-95's precedent, `sleep_lo_point()`'s code).

| name | home | unit / grid | box | sweep | gate | note |
|------|------|-------------|-----|-------|------|------|
| `GHOST_MAX` | `si-walk` (frame scratch) | Count | 0 … 65535 | `G15/recurrence_sweep.json` | G15(e) | guard ⇒ never a draw (OD-99: fixed cap, arousal-scaled **absent**) |
| `R_MIN_CRYSTALLIZE` | `si-crystallize` | Count (distinct frames) | 1 … 65535 | `G15/recurrence_sweep.json` | G15(e) | §22.3 rule 3 |
| `R_ASK_CRYSTALLIZE` | `si-crystallize` | Count | 1 … 65535 | `G15/recurrence_sweep.json` | G15(a)(e) | ≥ `R_MIN` by the detector's own assert, not by a sweep |
| `DELTA_MIN_CRYSTALLIZE` | `si-crystallize` | Ticks (logical) | 1 … 65535 | `G15/recurrence_sweep.json` | G15(e) | "one hot afternoon cannot mint a concept" |
| `LAMBDA_SILT` | `si-silt` | U8 1/255 | 1 … 255 | `G16/silt_sweep.json` | G16(d) | a **share of the one curve**, not a second τ (§0 fact 5) |
| `SILT_LEVEL_MAX` | `si-silt` | Q12 | 1 … 65535 | `G16/silt_sweep.json` | G16(d) | `S_max` — the deposit clamp |
| `SILT_BUMP` | `si-silt` | Q12 | 1 … 65535 | `G16/silt_sweep.json` | G16(d) | `S_bump` |
| `THETA_SILT` | `si-silt` | Q12 | 0 … 65535 | `G16/silt_sweep.json` | G16(d) | below it a level **is** 0 and the entry drops |
| `SILT_MAX` | `si-silt` | Count | 1 … 65535 | `G16/silt_sweep.json` | G16(d) | **diagnostic bound read out by audit**, never an eviction policy |
| `IGN_WEIGHT_SILT` | `si-walk::idle` | Q12 | 0 … 65535 | `G16/silt_sweep.json` — **inert: a refusal cannot arm it** | G16(a)(d) | **default `0` = unarmed** (OD-108, §0 fact 6); G16 installs the smallest non-zero on-grid step (`lo + grid`) |
| `PROV_LINK_TUG` | `si-read` (derived index) | Q12 | 0 … 65535 | `G17/prov_sweep.json` | G17(b)(c) | OD-114; **reads no silt** — the tug↔silt edge is unbuilt because OD-104/OD-114 both own it |
| `RELIGHT_MAX` | `si-walk` | Count | 1 … 65535 | `G17/prov_sweep.json` | G17(d) | per-frame re-light cap; over bound = fewer re-lights, never a truncation of the walk |
| `R_MIN_SHAPE` | `si-skills` | Count | 1 … 65535 | `G18/shape_sweep.json` | G18(e)(f) | §25.4 rule 1 |
| `DELTA_MIN_SHAPE` | `si-skills` | Ticks | 1 … 65535 | `G18/shape_sweep.json` | G18(e) | §22.3 rule 3 verbatim |
| `K_FILLER` | `si-skills` | Count (distinct filler sets) | 2 … 65535 | `G18/shape_sweep.json` | **G18(f)** | the generality bound; `lo = 2` because "a move that only ever connects the same three points is a concept" |
| `SHAPE_MAX` | `si-walk` (frame scratch) | Count | 0 … 65535 | `G18/shape_sweep.json` | G18(e) | the role-graph's `GHOST_MAX` twin (§16.8's second tenant) |

### 3.2 New ruling rows (6) — the least-authority defaults, each installable

| name | home | values | default | gate | the OD that owns the question |
|------|------|--------|---------|------|-------------------------------|
| `CRYSTALLIZE` | `si-session` | `off \| on` | **off** | G15 | OD-97…101 (the arc's own range) |
| `SILT_SPREAD_PRIMING` | `si-walk` | `off \| on` | **off** (OD-104's option (a)) | G16(e) | OD-104; the `primed` tag ships **regardless** |
| `EPISODIC_TRAVERSAL` | `si-walk` | `off \| on` | **off** | G17 | OD-110/111/115 |
| `SKILLS` | `si-session` | `off \| on` | **off** (detection) | G18 | OD-116…119 |
| `SKILL_COMPOSITION` | `si-walk` | `off \| on` | **off** (use) | G18(b) | OD-118 — detection and use are separable, so they are two rows |
| `WORLD_TIERS` | `si-core::world` | `owner-only \| granted` | **owner-only** | G19 | OD-124/125 |

**The launch channel widens — and only along one axis.** `runner.rs::launch_constants`
currently refuses anything but `Class::Species`; four of the five drafts state that
their gate scenario installs `on` "through the ordinary launch-constant channel",
so at **B14** the class filter becomes `Species | Ruling`, every row above gains a
`match` arm, and **`Class::Temperament`, `Class::Law` and `Class::Derived` stay
refused** — OD-87's ban ("no sweep may ever touch a temperament row") and 03 §1's
frozen-law rows keep their exact mechanical meaning. Nothing else about the channel
changes: a value still must be inside the row's own box and on its grid, and a run
with a declared list still records its version tags in the build identity.

### 3.3 The version consequence (read this before coding anything)

`CONST_ROWS` grows by 22. `VersionTags::encode` writes the row **count** and every
`(name, ver)` pair, so hash item 8 changes for every run in the repository. One
stage absorbs it:

* `si_core::consts::CONSTS_VERSION`: `0x0001` → **`0x0002`**, at B14.
* `EMPTY_STATE_MD5_SEED_ZERO` re-pins (its value is a function of item 8), and the
  three places that assert it — `si-core/tests/kernel.rs`,
  `si-session/tests/seed.rs`, `si-session/tests/wake.rs` — are updated **in the same
  commit**, together with the recorded pre-arc hashes in the G10/G13 report fixtures.
* `HASH_FORMAT` stays, `SEED_FORMAT` stays `0x0001` (no draw changes: §0 fact 6),
  `DERIV_ORDER` stays 18 values / 8 rows, and `temperament_digest` is untouched.
* Every artifact recorded under `0x0001` remains attributable to the arithmetic that
  made it (05 §21's rule) and is superseded by a fresh run, not edited.

### 3.4 §1.2's class column gains 22 rows and no new class

All 16 swept rows pass [10 §2](10-temperament-sleep.md)'s two tests ⇒ SPECIES: each
is either a guard (`GHOST_MAX`, `SHAPE_MAX`, `RELIGHT_MAX`, `SILT_MAX`, the four
recurrence bars — every one bounds a mechanism, so it may not be individual) or a
strength whose gate's objective is a *shape* not a magnitude. The tempting
re-assignments the drafts already refused stay refused: `IGN_WEIGHT_SILT` is the
only one that could have been a 19th draw, and it is not (OD-108 open, §0 fact 6).
The 6 ruling rows are exactly `INFERENCE_PROMOTION`'s pattern.
`check_temperament_closed` and `check_sweep_never_touches_temperament` must stay
green with the new rows present — the first because the draw count does not move,
the second because **every** new row's `gate` tag points at an artifact whose
`axes()` enumeration is asserted to be `Class::Species`-only.

---

## 4. Delta to [02](02-module-contracts.md) — the tick, the contracts, the grep rules

### 4.1 The tick, with the arc's six insertions

Steps 1–13 (incl. 12, 12b) keep their numbers and their order; the arc inserts at
four points, adds one arm inside step 6, and closes one hole. `Xa` means "inside X,
after X's own body" — which is exactly why the tug index is **5b** and not 6a: it
must exist *before* the walk body consumes it.

```
1   next sense frame (or NONE)                        B18: body carries speaker_identity + conversation (§9's transport byte)
2   front.tokenize → front.decode                     unchanged
3–4 /correct, /teach, pending-ask match               OD-128: origin=2 asks answer only in the owner lane
5   walk.light(pattern)                               unchanged
5a  B15 silt.tick(conversation, now)                  decay the ACTIVE table before anything deposits
5b  B16 tugs = read.event_view_index(rows, stamps, eventlog)   derived prov-link index; writes nothing
6   walk.walk(..., &tugs)                             B16: a tug re-lights its points at round 0, frame activation only (≤ RELIGHT_MAX)
                                                    B14: every light appends one CoLight to the frame log (ghosts/role-graphs build here)
                                                    B15 OD-104(a): the per-round spread reads NO silt — seed choice is its only read
7   walk.settle_or_stop → Thought                     + Thought.frame_log (CoLight[], ghosts, role-graphs; die at settle)
7a  B14 crystallize.observe(frame_log, store, now)    → Vec<Nomination>;  cold-frame rule (OD-105)
7b  B17 skills.observe(frame_log, eventlog_rows, now) → Vec<Nomination>;  forwarding rule (OD-121)
8   read.gaps → affect.prioritize → budget → emit     nomination gaps join the SAME queue (kinds 5/6, origin 2)
9   front.label + calibrate demote-only               unchanged; cluster/skill citations name rows
10  front.reverse_decode → stream.siro                unchanged; NAMELESS surfaces never printed
11  learn.commit(if a candidate survived)             B18: the tier arm runs first — class ≠ Owner ⇒ belief write REFUSED
12  quiet && agenda empty? → walk.idle_tick           B15: + arg &silt_view (the fifth term)
                                                    B18 OD-126: a declared window may SUPPRESS this step and cause nothing else
12b sleep.run(passes 1..4)                            unchanged; sleep reads no silt, no candidate, no skill
13  eventlog.flush; StateHash::compose; clock.advance unchanged — nine items, same order
```

**The conversation key at B15 is not the transport byte.** Silt is keyed by
conversation **from its first build** — that is OD-109's point, so that OD-127's
per-thread option is a keying decision and not a new mechanism — but `conversation`
only enters the frame body at **B18**. Between B15 and B18 the key is the declared
default (`0`, "the one conversation this session is"), which is exactly
[docs/23 §OD-109](../docs/23-context.md)'s "the single conversation every
pre-existing scenario already is", and is why nothing built before the arc notices.

**Three ordering laws the arc adds.** [04 §26](04-algorithms.md) already runs to 26
and 02 §11 cites those numbers, so the arc's are **27, 28, 29** — appended, never
renumbered, and stated identically in both files.

27. **A deposit is after the walk, a decay is before it** (5a precedes 6, deposits
    follow 7). Silt therefore never biases a selection using light from the very
    frame it is about to bias — the lag is the same one-round shape as OD-29's, and
    it is what makes G16(d)'s "last-touch order" a pure function of the transcript.
28. **Observation is after settle, before commit** (7a/7b sit between 7 and 11).
    A detector that ran before settle would nominate on a half-formed thought; one
    that ran after commit could read its own minted row as evidence. This is the
    ordering that makes G15(e)'s spam bar and G18(f)'s generality bar expressible.
29. **A nomination enters the queue as a `Gap`, never as an emission** (7a/7b → 8).
    `crystallize`/`skills` cannot call `emit`; `drive` decides, at the same
    `ASK_CAP_PER_INPUT` and the same `DEDUPE_WINDOW` as every other gap — "a busier
    world may not buy more asks" (OD-128) is a consequence of this line, not a rule
    added on top of it.

### 4.2 The three crate contracts (02 §13's table style)

| surface | inputs (ordered) | returns | contract |
|---------|------------------|---------|----------|
| `crystallize::observe(frame_log, store, now)` | the frame's `CoLight` slice, a read-only store view, `t` | `{nominations: Vec<Nomination>, candidates: u32, frames_counted: u32}` | pure function of the arguments plus its own table; reads **live L3 rows only** (frozen rows may be fillers, never members); never reads `a`; a member-set hash merges, never stacks; over `GHOST_MAX` the frame builds no ghost and the detector sees fewer members |
| `crystallize::cold_frame_count(log)` | the frame log | `u32` | the OD-105 rule: a frame with any `primed` light contributes **no** recurrence; recurrence counts *distinct frames* at `≥ DELTA_MIN_CRYSTALLIZE` apart |
| `silt::tick(active_conversation, now)` / `silt::deposit(conv, point)` / `silt::level(conv)` | handles and `t` | levels on the Q12 grid | `level' = clamp(level + SILT_BUMP, SILT_LEVEL_MAX)` at deposit; `level' = level · d` at tick with `d = LAMBDA_SILT + (255−LAMBDA_SILT)·recency(now−last_touch)` from **`Clock::recency` only**; `< THETA_SILT ⇒ 0 and drop`; **no** eviction, **no** ranking, **no** `a`, **no** source weighting (OD-106 ships parity) |
| `silt::view(conv) → &BTreeMap<u32,u16>` | the conversation key | the read-only table | its only consumer is `walk::ignite`; `check_silt_ignition_only` is the whole of OD-104's (a) made mechanical |
| `skills::observe(frame_log, outcome_rows, now)` | frame log + the citable `eventlog` rows | `{nominations, candidates, shapes: u32, forwarded: u32}` | generality bar (`K_FILLER`) *before* the recurrence bar; a shape whose recurrences share one filler-set is **dropped from its own table and forwarded** (OD-121) — the forwarding is a session-side decision, not a message between detectors |
| `read::event_view_index(store, stamps, eventlog)` | the committed rows | `Vec<OriginTug>` | **derived at frame load**; zero writes; a tug names an `event_id` and the points to re-light; capped at `RELIGHT_MAX` per frame; `EPISODIC_TRAVERSAL = off ⇒` the vector is empty and the walk's arithmetic is bit-identical to B13's |
| `world::class(identity, owner_identity, grants) → SpeakerClass` | the frame's identity and the (empty tonight) grant view | `Owner \| Grimm \| Trusted \| Stranger` | a **pure function**, computed at generation, stored nowhere (R-28's shape); `check_speaker_class_derived` fails the build on any container, schema, export or cache that carries a class |
| `learn::commit(...)` | (unchanged) + the derived class | `Ok \| Refused` | one new arm ahead of the existing ten steps: a belief-row write whose class is not `Owner` returns `Refused{SpeakerNotRepresentable}` with a log line and **zero bytes**; under `WORLD_TIERS = owner-only` the arm is unreachable except by a granted line, which cannot exist (OD-135/136) |

### 4.3 Grep rules (xii)–(xvi), restated as build checks

02 §12's table (i)–(xi) keeps its eleven rows; the arc adds five, each with the
same double enforcement — a check for tomorrow's refactor and a missing edge for
today's violation.

| rule | assertion | the edge that enforces it |
|------|-----------|---------------------------|
| (xii) **nomination proposes, never mints** | no `commit`/`append_row`/`age` symbol reachable from `si-crystallize::` or `si-skills::`; their only outputs are values | `si-crystallize ↛ si-learn`, `si-skills ↛ si-learn` |
| (xiii) **the detector and the table are wordless about arousal** | no `a`, `charge`, `bake`, median, door or gain symbol reachable from the three new crates | `↛ si-affect` on all three |
| (xiv) **silt names no row** | `si-silt` imports neither `si-store` nor `si-learn`; its key type is `u32` | `si-silt ↛ si-store` |
| (xv) **frame scratch dies at settle** | no `Ghost`/`RoleGraph`/`CoLight` value escapes `si-walk`'s frame; nothing under `si-store`/`si-learn` can name those types | docs/16 §16.8; `check_ghost_is_frame_scoped` |
| (xvi) **the arc adds no vocabulary, no byte, no shelf** | `RECORD_FIXED_BYTES == 112`, five `prov_kind` values, seven OWNED edge kinds, eleven shelves, `RESERVED_SHELF_COUNT ≥ 2`, `HASH_ITEM_COUNT == 9`, and §3.2's allowlist byte-identical | `check_record_layout`, `check_string_constants`, `check_registry_size` (all three must stay green, which is the proof) |

---

## 5. Delta to [04](04-algorithms.md) — §29–§33 in one table

The algorithms get their own sections in 04 at B14–B18 (this file pre-commits their
shape so the build cannot invent one mid-flight).

| § | algorithm | the arithmetic, exactly | determinism note |
|---|-----------|-------------------------|------------------|
| **§29** | crystallize | `§22.3`'s four rules, in order: (1) `|M| ≥ 3`; (2) **shared-filler test** — for every member, the set of `(relation_kind, filler_point)` pairs it carried in this frame must intersect a common relation kind; (3) recurrence over **distinct cold frames** at `≥ DELTA_MIN_CRYSTALLIZE` apart; (4) no existing live/frozen row binds exactly `hash(M)` under one label, else `confidence` leg on that row and **no candidate**. Merge by member-set hash. Contradiction-open rows excluded. All-BASE members ⇒ discard (I2). | a pure fold over `(frame_log, store, t)`; the table is iterated by set hash ascending, single-threaded, integer-only. Same transcript ⇒ same bytes |
| **§30** | silt | `d = LAMBDA_SILT + (255 − LAMBDA_SILT) · recency_u8 / 255` on the **one curve**; `level' = mul_q12(level, d)`; drop at `THETA_SILT`; `level' = min(SILT_LEVEL_MAX, level + SILT_BUMP)` on deposit; `SILT_MAX` is a counter in the audit readout, never a decision | reciprocal multiply, no division (OD-45); last-touch ordering is why G16(d) "clears backwards" needs no extra rule |
| **§31** | episodic | `OriginTug { event_id, points: [u32], weight_q12 }`, `weight = PROV_LINK_TUG` (registry-low, never modulated by `a`); at round 0 the tug lights `points` at `mul_q12(weight, level_of_belief)` **read-only**; `RELIGHT_MAX` frames per walk, in `(event_id, point)` order; a re-light writes **no stamp, no bake, no confidence** (OD-112 rides OPEN-24, which is off) | derived index, so deleting it changes traces only; `a_at` vs `a_now` is §20.2's existing conduction term applied to a re-lit row — **no new curve** |
| **§32** | skills | role extraction: `Role ∈ {seed, lit_a, lit_b, shared_filler}` from the round the point first lit and its edge direction; template = the sorted role/kind adjacency canonicalized to a folded string (the interning id becomes `stim_signature`); bars: `K_FILLER` distinct filler-sets **then** `R_MIN_SHAPE` cold frames at `DELTA_MIN_SHAPE` apart; outcome tag read from `eventlog` kinds only `{question-resolved, realized-gain, owner-reply, none}` | the forwarding rule (OD-121) is evaluated before the recurrence bar, so one frame is counted by exactly one detector |
| **§33** | world | `class = f(identity, owner_identity, grants)`: `identity == owner ⇒ Owner`; `identity == grimm's registered id ⇒ Grimm`; `grants.iter().find(g.subject == identity ∧ g.live_at(now)) ⇒ Trusted`; else `Stranger`. Tier-strict arbitration: `Owner > Trusted > Stranger-sense`; a Trusted correction supersedes Trusted rows only; **no write path can produce a Trusted belief row tonight** (§7.4) | `grants` is empty at birth (OD-125 "coded, not granted"), so `class` never returns `Trusted` in a pre-B18 run and every pre-arc gate is bit-identical |

**Ignition's fifth term, verbatim** (the one arithmetic change to shipped code):
`terms = [recency, bake, deficit, warmth]` becomes
`terms = [recency, bake, deficit, warmth, mul_q12(IGN_WEIGHT_SILT, silt_level)]`
with `silt_level = 0` for a point the table does not carry, so at the species
default the Q12 sum is **numerically identical** and the cumulative scan picks the
same seed for the same draw. `last_lit` ordering, the RNG stream `R`, one draw per
quiet tick, and the four TEMPERAMENT draws are untouched.

---

## 6. Delta to [05](05-gate-harness.md) — variants, legs, checks, sweeps

### 6.1 Deletion variants V10–V13 (V0–V9 keep their meanings)

| variant | what it is | gates it serves |
|---------|-----------|---------------|
| **V10** | `crystallize` + `skills` **absent from the build** (`--no-default-features --features vulkan,supporter` minus the two crates; the frames still log, nothing observes) | G15(b), G18(b) — "removing the reader changes no shelf except its own `origin = 2` asks", asserted as **`questions_filtered_digest`** equality plus an interner byte-identity (05 §23); a whole-`state_hash.txt` claim here would be false |
| **V11** | `silt` absent (the fifth term's row stays at 0, the `primed` tag stays in the log and is always false) | G16(b) |
| **V12** | `EPISODIC_TRAVERSAL` installed `on` **and** the derived index forced empty (the traversal exists, the tugs do not) | G17(b) — the deletion is of the *traversal*, per §24.6 |
| **V13** | `WORLD_TIERS = granted` with a **non-empty grant table built by the scenario's own owner line** (the only way a Trusted class can ever be produced tonight) | G19(c)/(d) — and the pass condition is the *refusal*, OD-135 |

All four are build-graph exclusions or launch states, never code paths: like V9,
they are declared in the run's build identity and a normal session cannot reach
them. `--gate all` and 05 §21's per-release-candidate set gain them.

### 6.2 The ten new static legs (05 §20's table format; 36 → 46 ids)

| id | scans | pass condition | gate |
|----|-------|----------------|------|
| `check_crystallize_no_commit` | symbol reachability from `si-crystallize::` and `si-skills::` | no `commit`/`append_row`/`age`/`salience` symbol reachable; the crates' only store calls are reads | **G15(a)(b)(d)**, **G18(a)(b)(d)** |
| `check_ghost_is_frame_scoped` | every type in the arc's frame scratch | no `Ghost`/`RoleGraph`/`CoLight` appears in a container, schema, export, cache or `SIst`/`SIck` write; the types are constructed and dropped inside `si-walk` | **G15(b)**, docs/16 §16.8 |
| `check_silt_writes_nothing` | `si-silt`'s imports and every assignment | the crate imports `si-core` only; its table's sole mutators are `deposit`/`tick`; no store/learn/affect symbol exists in its graph | **G16(a)(b)**, §23.0's litmus |
| `check_silt_ignition_only` | read sites of `silt::view` | exactly one consumer, `walk::ignite`'s seed choice; **zero** uses in `spread`/`settle_or_stop`/`conduction` | **G16(e)**, OD-104 (a) |
| `check_prov_link_not_stored` | the derived index's construction and the store's write set | no `prov-link` row or edge is ever appended; `SHELF_EVENTS`' `row_schema_version` stays 0; the index is rebuilt from rows at frame load | **G17(a)(b)**, OD-110 |
| `check_relight_writes_nothing` | the re-light path inside `si-walk` | the only mutation is frame activation; no `Stamp` construction, no `bake` assignment, no confidence write is reachable — `src = 2 idle-re-light` is never emitted | **G17(c)(d)** |
| `check_skills_no_commit` | `si-skills`' graph + the skill row's writer | the writer is `si-learn`'s ordinary commit; `si-skills` cannot reach it; skill usage counters are recomputed, never assigned | **G18(a)(f)**, OD-120 |
| `check_nomination_owner_lane` | every sink of a `questions` row with `origin = 2` | the only lane reachable is the owner lane; no `origin = 2` row is ever routed to a non-owner surface | **G19(f)**, OD-128 |
| `check_speaker_class_derived` | every container, schema, export, cache | no field named or typed as a speaker class exists anywhere but as a return value; `speaker_identity` appears only in the framed sense body and `transcript.txt` | **G19(b)**, OD-123/124 |
| `check_windows_suppress_only` | the idle gate's predicates | a window value can only appear in a **deny** conjunct; no path from a window to `ignite`, `sleep::due`, `clock.advance` or a frame read | **G19(g)**, OD-126 |

### 6.3 Checks that MOVE (the arc must not quietly break them)

`check_deps` (13 → **16** package names; `Cargo.lock` hash re-pinned — the lock's
allowlist value is unchanged because the arc adds **no dependency**),
`check_registry_size` (reserved count 3 → 2 at B17), `check_emotion_words` (scope
gains three crate directories), `check_one_clock` (scope gains `si-silt`; the
pass condition is that the build still contains exactly **one** recency
implementation), `check_idle_no_commit` (the fifth term must not become a write
path), `check_string_constants` (must stay green with `skills` live — the word list
already carries it), `check_candidate_unanswerable` (grows an arm: a nomination
candidate can never be an answer's source, whichever way OD-58/98 land), and
`check_temperament_closed` (the new rows must not enter `DERIV_ORDER`; the count
stays 18).

### 6.4 The five gate families, leg by leg

Each leg below is the procedure the harness runs; where the docs' wording needed a
mechanical reading, the reading is stated and cited to §10's OD list. The families
are **appended** exactly as G13/G14 were: `gates/G15..G19/`, `gate_report.json`
per leg, `unresolved_legs` for the magnitude legs whose sweep is a refusal.

**G15 CRYSTALLIZE** (scenario: `g15_fruit.txt`, the apple/tomato/banana fixture,
`CRYSTALLIZE = on` at the channel floor, `--seed 0x51`)

| leg | procedure | pass |
|-----|-----------|------|
| (a) determinism | replay the transcript twice, `CRYSTALLIZE on` and `off` | the two **`questions_filtered_digest`** values are byte-identical ([05 §23](05-gate-harness.md) defines the digest; a whole-hash equality would be false, because the `on` run really does open a `questions` row), the interner is byte-identical, and the only extra rows are the family's own `origin = 2` asks — named in the report, never silently dropped |
| (b) deletion | V10 build, same transcript | the same filtered digest against "`V0` + `CRYSTALLIZE = on`"; the ask of kind 5 is the only possible difference, and the interner matches |
| (c) arousal isolation | **frame-trace-fixed** (OD-134's reading (i)): feed the identical `frame_log` under two `a` regimes | candidate table byte-identical; the full-session reading stays `UNMEASURED` beside OD-83's refusal |
| (d) owner-only | a `grimm:` line answering a pending nomination ask | **zero belief-row writes** and its `context` sense row still appending (§21.4's rule, unchanged by the arc). This is B9's *existing* non-owner arm doing its existing job — `Refused{SpeakerNotRepresentable}` is **B18's** granted-class refusal (**OD-135**) and must not be cited here, because at B14 the tier arm does not exist yet |
| (e) spam bound | junk fixture: random pairs, 2-member sets, open contradictions, one hot afternoon inside `DELTA_MIN` | **zero** asks; supersaturated-but-never-asked candidates are reported as counts, not failures |
| (f) no-word-leak | RENDER output over a run with live unnamed candidates | no `cluster:` string, no invented word; a minted-but-unnamed point is cited by member enumeration only (`NAMELESS` never printed) |

**G16 SILT** (scenario: `g16_warm.txt`, `IGN_WEIGHT_SILT` at `lo + grid`,
`SILT_*`/`THETA_SILT` at their boxes' floors)

(a) determinism — replay the transcript twice at the **shipped unarmed `IGN_WEIGHT_SILT
= 0`**, where the fifth term is a literal 0 addend and `state_hash.txt` is byte-identical
on and off; (a′) the **OD-103 exemption asserted at the armed value** — `silt.jsonl`
differs, `live_rows`/`edges`/`stamps`/interner do not, and `settle_set`/`trajectory` may
move only on the points the fifth term seeded (05 §23.2 fixes that partition, and it is
the honest one: an armed preference lawfully moves the idle path, so this family's hash
equality is claimed at `0` and nowhere else); (b) deletion — V11, whose control is the
**unarmed** arc-on run, per §2's variant table; (c) arousal isolation — the same
frame-trace shape as G15(c); (d) **clears backwards** — scripted touches at known
ticks then a long idle: drop order is **strictly** last-touch order, and after the
computed `T_cold` the table is empty and the fifth term is identically 0, so the
sampler equals §19's lattice-only sampler exactly; (e) crystallize isolation —
**frame-trace-fixed** on the identical `frame_log`, silt armed vs absent, candidate
table identical via the cold-frame rule (a primed frame that increments a recurrence is
a red); (f) **no innate time words** —
the harness's needle set (`recent`, `just`, `earlier`, `ago`, `still`) over the
run's RENDER output: absent, because they were never taught; the needles live in
`crates/si-cli/src/harness/` (the auditor scope), never in `src/` proper — R-27's
own law.

**G17 EPISODIC** (scenario: `g17_origin.txt`, `EPISODIC_TRAVERSAL = on`)

(a) determinism; (b) deletion — V12, **whole `state_hash.txt`** byte-identical to V0
(this variant mints no ask, so §23's filtered digest is not needed and reaching for it
here would be a fudge), only traces/monologue/
citations differ — and none of those is state; (c) arousal isolation — the **state items
pinned** (`live_rows`, `edges`, `stamps`, `interner`, `version_tags`, `core_block`), the
two the re-light lawfully moves **not** pinned (`settle_set`, `trajectory`) with every
moved point named from the tug log; that move is §20.2's existing conduction term doing
its job and `check_one_clock` is the witness that no new curve appeared; (d) no re-stamp — the **structural**
grade OD-88 established: same stamp-arena record count and byte length after N
re-lights, values free; zero new stamps is asserted by `check_relight_writes_nothing`
and by the arena count, and OPEN-24's off state is printed as the reason; (e) grimm
isolation — grimm event views re-lit N times, zero writes, and the absence grep:
no tug names a `context` row whose `prov_speaker` is `1` as a **belief origin**;
(f) no-flashback — the experience-claim needle set (`felt`, `flashback`, `remember
how`) absent from RENDER, recall printed as `FACT/OWNED` with both citations.

**G18 SKILLS** (scenario: `g18_compare.txt`, `SKILLS = on`, `SKILL_COMPOSITION = on`)

(a) determinism (skills on/off ⇒ identical **`questions_filtered_digest`**, identical
interner, the only extra rows being the family's own kind-6 `origin = 2` asks — 05 §23
defines the digest and says why a whole-hash claim would be false here); (b) deletion — V0 vs V10
plus installed rows sitting unread; (c) arousal isolation; (d) owner-only; (e) spam
bound; (f) **generality bound** — a "shape" below `K_FILLER` distinct filler-sets
never opens and is counted as `forwarded`, and the forwarded frames remain
available to G15's detector (the exclusion is one test, not two claims).

**G19 WORLD** (scenario: `g19_server.txt` — 10 identities, one owner, one grimm,
eight strangers; OD-132's harness fixture)

(a) stranger-silence — zero protected-class writes with the owner silent; `context`
rows **do** append, and a run that suppressed them is reported as a *different
design*, not a stricter one; (b) class-derivation — `check_speaker_class_derived`
plus a report of `class(identity)` over the whole fixture; (c) grant-trace — V13:
an owner grant line mints a grant row, a Trusted line's belief write is
**refused with zero bytes** (`Refused{SpeakerNotRepresentable}`), the revocation row
is appended, and the same identity's next line derives `Stranger` — **the refusal is
the pass condition**, and OD-135 is printed in `observed` as the reason; (d)
answer-to-ask owner-only-or-granted — a stranger answer commits nothing; (e)
determinism — identity-tagged transcript replay byte-identical; (f) routing — every
`origin = 2` ask lands in the owner lane; (g) starvation — a scripted busy server
with a declared window: idle ticks occur only where the window is open **and** the
quiet-and-empty-agenda law already held, and no tick occurs because a window opened;
owner preemption intact; (h) speakgate-for-grants — under V13 a Trusted line passes
the **same** token-membership filter grimm's lines pass (it is the same code path
over the same taught lexicon, which is what makes this leg cheap and honest).

### 6.5 Sequencing additions (05 §21)

Per commit: the arc's ten checks join the static set from B14 on. Per release
candidate: G15–G19 run at V0 plus V10–V13. **Per re-sweep:** the four new sweep
runners are added to the ten (→ fourteen), each publishing a refusal while OD-83 is
unrulled, and G15–G19 re-verify on the new version tags. `CONSTS_VERSION`'s bump at
B14 outranks a version tag in the same way a `seed_format` bump does: it re-baselines
every `state_hash.txt`, so §9's artifact table is the ledger of what has to be
re-run.

---

## 7. Delta to [06](06-resource-budget.md) — cost, honestly labeled

| mechanism | per | added work | bound | measured? |
|-----------|-----|-----------|-------|-----------|
| frame co-light log | light | one `CoLight` push, 12 B RAM | `FRONTIER_CAP` × rounds (already bounded) | UNMEASURED until G15's first run |
| ghost / role-graph build | frame | a fold over the frame log | `GHOST_MAX`, `SHAPE_MAX` (both `—`) | UNMEASURED |
| crystallize observe | thought | `O(candidates × members)` integer set-intersections over live L3 reads | candidate count ≤ live rows/3 | UNMEASURED |
| silt tick | tick | `O(active entries)` reciprocal multiplies | `SILT_MAX` (diagnostic) | UNMEASURED |
| event-view index | thought | one pass over committed rows' provenance | commits per session | UNMEASURED |
| re-light | frame | ≤ `RELIGHT_MAX` point lightings, no arena writes | `RELIGHT_MAX` (`—`) | UNMEASURED |
| skill observe | thought | role extraction + template interning | shapes ≤ distinct role-graphs | UNMEASURED |
| class derivation | line | one comparison chain over `grants` (empty ⇒ O(1)) | grant rows, zero tonight | UNMEASURED |

The three `.jsonl` logs are **append-only per tick/thought** and are the arc's real
byte cost; they join 06 §6's "log ≠ state" line, and none of them is read back as a
build input (R-33). No row, edge or stamp byte grows, so 06's per-row arithmetic
(112/16/16) and every derived capacity stay valid unchanged.

---

## 8. Delta to [11](11-build-plan.md) — stages B14–B18, and where they sit in the order

**Execution order** is B0…B9, **B14…B18**, B10…B13. `Stage`'s enum declaration
order *is* execution order and `label()` carries the printed id, so B14–B18 are
declared between `B9` and `B10` (`ALL: [Stage; 19]`) — that keeps 05 §21's prefix
semantics (`enable_stage(id) <= stage`) true for every stage that follows, including
B13's acceptance pass, which then runs the full 46-check union.

| id | stage | implements | creates | exit gates | commit subject |
|----|-------|-----------|---------|-----------|----------------|
| **B14** | arc substrate + crystallize | §1.1–§1.3, §3, §4.1 (5a/7a), §5 §29, §6.2/§6.3 | the three packages with final edges (stubs for silt/skills), `CoLight`/ghost/role-graph scratch, `si-crystallize`, 22 registry rows, `CONSTS_VERSION 0x0002`, the re-pinned empty-state hashes, `launch_constants` class widening + arms, `CRATE_NAMES: [&str; 16]` | **G15** + `check_crystallize_no_commit`, `check_ghost_is_frame_scoped`, `check_silt_*` disabled-and-listed, all §6.3 checks re-green | `B14: crystallize + arc substrate — engineering/12 §1/§3/§5 §29, 02 §11, docs/22` |
| **B15** | silt | §2.6, §4.1 (5a/6/12), §5 §30, §6.4 G16 | `si-silt`, the fifth ignition term, the `primed` tag, `silt.jsonl` | **G16**, `check_silt_writes_nothing`, `check_silt_ignition_only`, `check_one_clock` (widened) | `B15: silt — engineering/12 §5 §30, 02 §11, docs/23` |
| **B16** | episodic | §2.6, §4.1 (5b/6/7), §5 §31 | `si-read::event_view`, `OriginTug`, the walk's re-light path, `mindlog.event_views` | **G17**, `check_prov_link_not_stored`, `check_relight_writes_nothing` | `B16: episodic — engineering/12 §5 §31, 04 §20, docs/24` |
| **B17** | skills | §2.2/§2.3, §4.1 (7b/8), §5 §32 | shelf 9 live (`RESERVED_SHELF_COUNT 2`), `si-skills`, `NAMELESS`, the composition preference, the forwarding rule | **G18**, `check_skills_no_commit`, `check_registry_size`, `check_nomination_owner_lane` | `B17: skills — engineering/12 §2/§5 §32, 01 §2/§6, docs/25` |
| **B18** | world | §2.6, §4.2, §5 §33, §6.4 G19 | `si-core::world`, the grant-row type (declared, unwritable), tier-strict arbitration, the lane rule, suppress-only windows, the scenario's `identity:` tag | **G19**, `check_speaker_class_derived`, `check_windows_suppress_only`, `check_candidate_unanswerable` (grown) | `B18: world — engineering/12 §5 §33, 02 §12, docs/26` |
| B19 | *(not a stage — the regression pass)* | — | — | **G1…G19** at V0 + V10–V13 + `check --all`, run inside B13's acceptance and again after B10–B13 land | — |

**Prerequisite, and it is load-bearing:** **B9 (supporter) must be green first.**
G15(d), G16(b), G17(e), G18(d) and G19(a)(d)(h) all need a non-owner line arriving
through the seam, and the two-layer speak gate is what G19(h) reuses for grants.
That is why the arc's stages sit after B9 and not after B8.

---

## 9. The break-list — every pre-existing artifact the arc moves

| # | thing that breaks if the arc is dropped in carelessly | where | the stage that fixes it | how |
|---|-------------------------------------------------------|-------|-------------------------|-----|
| 1 | `CRATE_NAMES: [&str; 13]`, the `"13 crates"` detail string, 00 §2's "Thirteen crates", 11 R1, B0/B13's `cargo metadata` count | `si-cli/src/harness/checks.rs`, 00 §2, 11 §2/§4 | **B14** | `[&str; 16]` + R1 re-read ("sixteen packages, `si-cli`/`si-harness` two binaries of one package") |
| 2 | hash item 8 (`VersionTags::encode` writes the row count and every name+tag) | `si-core/src/hash.rs` | **B14** | `CONSTS_VERSION` 0x0001→0x0002; recorded hashes re-run, not edited |
| 3 | `EMPTY_STATE_MD5_SEED_ZERO` + its three asserting tests | `si-core/tests/kernel.rs`, `si-session/tests/{seed,wake}.rs` | **B14** | re-pin in the same commit that adds the rows |
| 4 | every `state_hash.txt` in `gates/G1..G14/` | gate artifacts | **B14** then re-verified at each of B15–B18 | pre-arc dirs stay as history; the arc commits re-run the gates whose legs compare hashes (G5, G10.e.5, G12(a), G13, G14) and record the new values |
| 5 | `check_registry_size` + the shelf list in `check_string_constants` | `si-core/src/shelves.rs`, checks | **B17** | reserved 3 → 2; `"skills"` already in the name list, so the allowlist itself does not move |
| 6 | R-29 / `check_one_clock` under a naive `LAMBDA_SILT` implementation | `si-silt` | **B15** | the decay consumes `Clock::recency`; the check's scan widens and still finds one curve (§0 fact 5) |
| 7 | `launch_constants` bails on any new name | `si-cli/src/harness/runner.rs` | **B14** (crystallize + all rows) and per stage for consumers | arms for all 22 rows + the `Species | Ruling` class filter |
| 8 | `check_emotion_words`'s directory scope | checks | **B14** | three new crate dirs join the wordless set; none of the three crate names is one of the six audit words, so `si-silt` passes [03 §9](03-constants-registry.md)'s naming law by construction, and its readouts name **rows**, never time ([docs/23 §23.5's I6 row](../docs/23-context.md)) |
| 9 | docs/22 §22.4 and docs/25 §5.5's English ask **templates** | `si-read::ask`, `si-front` | **B14/B17** | they are *illustrations* (R-27: "teaches X" is not a literal). The build emits a nomination as an **enumeration of the cited surfaces** — the same law §23's ask channel already obeys — with `pattern=FACT/OWNED`, `name=NONE`. Zero new literals. |
| 10 | the scenario grammar: no `identity:` tag, no non-owner multi-voice fixture | `si-cli/src/harness/scenario.rs` | **B18** | one new optional `identity <u64>` declaration + `voice <id>` step prefix built from the **existing** two literals; no new word enters `src/` proper |
| 11 | G16(f)/G17(f) needles (`recent`, `still`, `remember how`) vs R-27 | harness | **B15/B16** | needles live in `crates/si-cli/src/harness/` (the auditor scope, exempt by `Workspace::is_auditor`) and in fixture files under `harness/` — never in a control crate |
| 12 | the 17-scenario-file requirement vs 12 present | `harness/scenarios/` | **B14–B18** | five new checked-in, hash-pinned transcripts (`g15_fruit`, `g16_warm`, `g17_origin`, `g18_compare`, `g19_server`) + `sleep:` step (B8's known gap) → the set closes at 17+ |
| 13 | `docs/26` §26.13's "lane **field**" wording vs §0 fact 2 | 01 §4.1, docs/26 | **B18** | satisfied by `key_field`'s origin half — the lane is derived, no envelope byte moves. Precision note lands in docs/26 §26.13 (Phase 1 still allows the edit; it narrows a wording, changes no law). |
| 14 | `SHELF_EVENTS = 11` looks like the natural home for event views | 01 §2 | **B16** | stays reserved: OD-110 ships *derived*, so writing an events row would be a schema change the ruling does not license |
| 15 | the trusted tier has no honest `prov_speaker` value | 01 §4.1 field 34, `si-learn` | **B18** | **OD-135**: the write is refused and the refusal is G19(c)'s pass. No fourth value is minted, so §11's closed-constant list and G5c's diff stay intact. |
| 16 | a 19th temperament draw would re-hash every run | `si-core/src/seed.rs`, `SEED_FORMAT` | **B15** | refused: `IGN_WEIGHT_SILT` is SPECIES at 0 (OD-108 stays open, `DERIV_ORDER` stays 18) |

---

## 10. OD ledger delta — OD-135 and OD-136 are the only new seats

The arc's 38 seats (OD-97…134) are in the ledger already. The engineering pass adds
two, both tail seats, both found by trying to *code* the law rather than read it:

* **OD-135 — which `prov_speaker` value encodes a granted trusted teacher?** docs/26
  §26.2 says the field "gains its class-leg" under tier (b), and §4.1 lists exactly
  `0 owner · 1 grimm · 2 system-inferred`, all three with a ruled scope. A granted
  teacher is none of them: writing `0` would claim the owner spoke (laundering,
  §26.3(c)); `1` and `2` are already reserved for a non-voice and an inference. The
  build therefore **codes the class function, the grant row and the tier-strict
  arbitration, and refuses the belief write** — `Refused{SpeakerNotRepresentable}` —
  and G19(c)/(d) grade that refusal. Ask: `OD-135: a new value / b (tier (b) is
  answer-to-ask-only and inherits the ask's owner-voice provenance — say so in
  docs/26) / c (keep the refusal)`.
* **OD-136 — who mints `source: grimm` and `speaker_identity` once an adapter
  exists?** Today the seam is the *only* minter of a non-owner line and the panel
  cannot mint one (00 §7). A Discord adapter would stamp identities for strangers;
  whoever stamps them holds the class function's input, which is an authority the
  design has never had a name for. Not built tonight, so the question is recorded
  rather than answered in code. Ask: `OD-136: the adapter is a process of the
  owner's, and its minting is an owner act / b / c`.

Everything else the build needed but the drafts left open was already seated
(OD-83's refusal, OD-98/103/117's shared storage ruling, OD-104's option (a),
OD-108's class question, OD-110's derived shape, OD-112 riding OPEN-24, OD-118's
composition-only, OD-124's tier (a), OD-126's suppress-only, OD-128's owner lane,
OD-132's harness fixture, OD-133's kind-blind stage criterion, OD-134's reading
(i)) and this file implements each one's least-authority half, naming it in the
report.

---

## 11. What this round refuses, stated so nobody "fixes" it

1. **No seventh organ, no seventh door, no seventh signal, no eighth label.**
   `ORGAN_COUNT = 6`, `DOOR_COUNT = 6`, `SIGNAL_KIND_COUNT = 6`, `LABEL_COUNT = 6`
   are untouched; a nomination is a *kind* in the queue, not a new interface.
2. **No new shelf, no new structure, no new envelope byte, no new `prov_kind`, no
   new edge kind.** The arc is kinds-on-existing-shelves, and §2.1 is the proof.
3. **No 19th temperament draw, no new hash item, no new `.siro` stream.**
4. **No skill execution.** A skill is a row and an ordering preference; OD-87's ban
   is not reachable around through a habit because `si-skills` cannot name the
   sampler and the sampler cannot name a skill.
5. **No arousal scaling** of the ghost budget, the silt table, or either detector —
   absent from the code (OD-99/107), not zero.
6. **No spread priming, no stored prov-link, no event-view row, no per-identity
   median, no retention window, no `/trust` parser, no Discord transport, no grant
   issued.** Each is a named OD; each is *coded-and-refused* or *not-coded-and-
   reported*, never half-built behind a default.
7. **No value without its artifact.** Sixteen swept rows stay placeholders, four
   sweeps publish refusals, and every leg that needs a number runs at its box's
   floor and says so. This is the same discipline that made G13's magnitudes
   `UNMEASURED`, and it is the reason "nothing we already made breaks" is a checkable
   claim rather than a hope.
