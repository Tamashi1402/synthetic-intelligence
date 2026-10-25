# 13 — The tongue: HOW for G20 (surface realization by walk)

Provenance. The law is [docs/27](../docs/27-tongue.md), folded from the owner's
draft plus the owner's amendment of the same date (§27.11: *the generator is the
walk*). This file turns those into crates, bytes, call sites, registry rows, one
algorithm, six checks and one stage — and it is written against the same standing
requirement the owner stated with the draft: **engineer it thoroughly so nothing we
already made breaks.**

This file adds no law, no organ, no door, no signal, no label, no shelf, no frame
kind, no envelope byte and no vocabulary word. It is the **HOW** the way
[00](00-stack-workspace.md)–[12](12-v2-arc.md) are, and where it disagrees with
`docs/27` or `docs/07 §7.5`, those files win and this file is the bug.

**Reading order.** §0 is the summary a reviewer needs; §1–§7 are the deltas to
engineering 00–06; §8 is the stage; §9 is **the break-list**; §10 is the OD ledger
delta (three new seats, all of them clashes between the draft and older law); §11
is what this round refuses.

---

## 0. The nine facts that decide everything else

| # | fact | consequence |
|---|------|-------------|
| 1 | **One new crate, sixteen → seventeen.** `si-tongue` is a reader-only leaf whose imports are `si-core` and `si-walk` — see §10 row 3 for why `si-store` is *not* in that list. Nothing imports `si-tongue` but `si-session`. | `CRATE_NAMES: [&str; 16]` → `[&str; 17]` (`checks.rs:171`), 00 §2's count, 11 R1, B0/B13's `cargo metadata` count, docs/12's map, and `check_deps`' expected set all move (§9 rows 1–2). |
| 2 | **The 18 do not move.** The five style axes ride their own sealed order, `DERIV_ORDER_TONGUE`, keyed by the same `birth_seed`. `DERIV_ORDER_LEN` stays 18, `SEED_FORMAT` stays `0x0001`, `temperament_digest` is untouched, `CoreBlock` stays 36 bytes. | `check_temperament_closed`, `seed::order_is_closed()`, `G14(c)` and every V2/V3 seed-panel report stay green and stay attributable. Extending the 18 to 23 was the draft's other option and it re-hashes every individual in the repository (§10 OD-165's seat, already ruled **separate order**). |
| 3 | **The style vector is never stored.** `Style { values: [u16; 5] }` is derived at wake and re-derived per render; only its `digest()` (first 8 MD5 bytes of the 10 canonical bytes, LE, exactly the 18's rule) is ever read by anything outside `si-core::tongue`, and it is read into a seed mix and a report field, never into a container. | The P.3 discipline (10 §4) holds for a second vector without a second schema change. `check_tongue_closed` and `check_tongue_is_leaf` grade the absence. |
| 4 | **Sixth registry class, and it is inert by construction.** `Class::Tongue` exists so the five style rows are *neither* swept *nor* installable. They carry `sweep: None`, so OD-86's placeholder rule gives them no value, `axes_of()` cannot enumerate them (it filters `Class::Species`), and `launch_constants`' existing `Species \| Ruling` guard refuses them without one character of edit to that guard. | `placeholders_match_registry()` stays true **because** the new rows have no sweep tag (§3.3 derives the arithmetic). OD-87's ban extends to the sixth class by derivation, and `check_style_never_installed` grades it (§6.2 row 5). |
| 5 | **Hash items stay nine, and the tongue moves none of them.** `HashItem::ORDER` is unchanged; the realized string is not state, the style vector is not state, and the realization draws from a per-render PRNG that is not the session's runtime stream. | Item 7 (`rng`) is the sequence the ignition history records and `G14.b.3` promises the derivation never touches; spending draws from it at speak time would re-baseline every state hash for a reason that has nothing to do with temperament (**OD-176**). `G20(j)`'s whole point is that a voice change moves no hash byte. |
| 6 | **The licensed vocabulary is an argument, not a lookup.** The walker's only source of a printable string is `License { point, surface }`, built by the caller from the *cited* rows' key/value points, their surfaces, and the composed chunk each cited row already carries. `si-tongue` cannot reach the point table. | **OD-181** ("a line utters only surfaces of the points its own citation names") stops being a discipline and becomes a missing edge: no store import, no `points()`, no `surface(handle)` call. `G20(h)` greps the emission against that same set and `check_tongue_license_only` asserts the set is the only mouth. |
| 7 | **The gate is three structural checks and a fallback.** (a) the label and citation are computed before the call and are not inputs to it — `si-front::label` reads `Rendered.content`/`containment` and never `Rendered.surface` (a type fact, `crates/si-front/src/render.rs`); (b) every emitted chunk is in the license; (c) parse-back — the realized line's folded token set is a **superset** of the canonical surface's and a **subset** of the licensed one. Any failure ⇒ `None` ⇒ the canonical frame prints. | "Assert the fallback, not the crash" (§27.8(c)) is the function's return type: `Option<String>`, and the call site's `else` arm is the byte that v1 prints today. |
| 8 | **The only byte the tongue itself contributes is `", "`.** A chunk joiner is punctuation, `tokenize` yields no token from it, so it can never be an unlicensed word, and the vocabulary scan (`check_tongue_license_only`, R-21's extension) reads exactly that. | There is no place in `si-tongue` a word could come from except a `License`. No const array of `&str` words exists in the crate, and §5's rule is that the crate declares none. |
| 9 | **Eight new registry rows re-baseline hash item 8.** `VersionTags::encode` writes the row count and every `(name, ver)`, so 95 → **103** rows changes every `state_hash.txt` in the repository — once, at one stage, with a version bump. | `CONSTS_VERSION` `0x0002` → **`0x0003`**, `EMPTY_STATE_MD5_SEED_ZERO` re-pins, and its three asserting tests (`si-core/tests/kernel.rs`, `si-session/tests/seed.rs`, `si-session/tests/wake.rs`) move **in the same commit** — exactly B14's precedent (12 §3.3). Every control recorded under `0x0002` is superseded by a fresh run, never edited (**OD-91**). |

---

## 1. Delta to [00](00-stack-workspace.md) — the 17th crate, its edges, its files

### 1.1 The new row of the graph

| crate | logical id (docs/12) | depends on | may NEVER depend on |
|-------|---------------------|------------|---------------------|
| `si-tongue` | `tongue.py` — the phrase walker and the gate | core, walk | **store** (no lattice handle ⇒ no point table ⇒ no word outside the license), **learn** (a realizer that could commit is a self-teaching voice), **affect** (OD-168: no `a` term enters a weight), **front** (it must not be able to read a label, and it must not re-tokenize with a second fold law — see §1.2), **read** (the citation is handed to it as data), session, sleep, supporter, audit, the arc's three |

`si-session` is the only importer, and the edge is one call at one site (§4.1).

### 1.2 The one crate this file cannot give `si-front`

The realized line is *re-tokenized* by check (c), and the repo has exactly one fold
law: `si_store::interner::fold_key` (`si-read` and `si-front` both call it, and
`si-read`'s copy is documented as a three-line fold call rather than an import
because `si-read` may not import `si-front`, 00 §2). So `si-tongue` needs the fold
and must not need the front. The resolution is not a new edge: **the fold runs in
`si-session` at the license boundary** — the caller hands the walker already-folded
slot keys and already-folded licensed keys, and the walker compares sets. `License`
therefore carries `surface: String` (what prints) and `keys: Vec<String>` (what the
checks read), and `si-tongue` contains no fold implementation of its own.

**OD-189 keeps that boundary.** The variant bank is grouped *by the same fold*, in
the caller: `license_for` walks the interner once, keys every unique verbatim
surface with `si_front::tokenize`'s token keys, and attaches each surface to the
license entry whose `keys` match. A `Variant` carries `surface` + `keys` and
nothing else, so the walker still compares folded sets and still cannot spell a
word the caller did not fold for it. If the fold lived in `si-tongue`, a spelling
could key differently there than in check (c) — the second law §1.2 exists to
prevent.

**Derivation, not convenience:** a crate that folded its own way would be a second
vocabulary law, and the second law is what `G20(c)` exists to catch.

### 1.3 Files (00 §8 delta): the loop writes nothing new

No new file, no new log, no new lane. The realized line rides the **existing**
`EV_OUTPUT` frame (`{"label":…,"tag":…,"surface":…}`) and the existing
`transcript.txt`; `gates/G20/realized_log.jsonl` and `refusals.jsonl` are written by
the **harness** from a run's own records, not by the session (§6.4). `check_bake_no_event_kind`
and 08 §2.1's 21 sealed frame kinds are untouched — there is no `EV_TONGUE`.

### 1.4 Process topology, features

The `tongue` cargo feature ships **on** in the default build (like `crystallize`,
12 §1.4): off would delete the crate and make V14's `TONGUE = 1` unreachable. The
ruling row is the runtime switch; the feature is the build-graph switch, and V14
uses the row, never the feature.

---

## 2. Delta to [01](01-data-schemas.md) — what does not change (assert this first)

| item | value | proof |
|------|-------|-------|
| `RECORD_FIXED_BYTES` | **112** | unchanged: the tongue writes no row |
| shelf registry | **11 entries, 9 live, 2 reserved** | unchanged: `check_registry_size`'s first clause |
| `ProvKind` values | **5** | unchanged |
| OWNED edge kinds | **7** (`0…6`), genome slots `7…` | unchanged — the walk *reads* arcs, it adds none |
| `PatternId` table | **closed at 10** | unchanged |
| frame kinds | **21 sealed** | unchanged (§1.3) |
| `CORE_BLOCK_BYTES` | **36**, `BLOCK_VERSION 0x01` | unchanged: the style digest is **not** a block field. 01 §13's reserved pad is the honest place a future round would put it, and this round leaves it zero |
| hash items | **9**, `HASH_FORMAT 0x0001` | unchanged (§0 fact 5) |
| `Body.resp_surface` | same tag byte space | unchanged: no `phrasing` row kind (**OD-180**, §27.2's "there is no fourth thing") |
| `SEED_FORMAT` | `0x0001` | unchanged |
| `DERIV_ORDER_LEN` | 18 | unchanged |

`CONST_ROWS` is the only registry-side thing that moves, and `VersionTags::encode`'s
row count is the only hash-visible consequence (§3.4).

---

## 3. Delta to [03](03-constants-registry.md) — 8 rows, one new class, one version bump

### 3.1 The five style rows (class `TONGUE`, drawn, never swept)

All five: `home = si-core::tongue`, `gate = "G20"`, `class = Class::Tongue`,
`unit = Unit::Q12`, `lo = 0`, `hi = 2048`, `grid = 16`, `sweep = None`,
`law_value = None`, `declared = None`, `scalar = true`.

| name | axis | what a draw leans (04 §34) | drawn `0` means |
|------|------|---------------------------|-----------------|
| `STYLE_SPREAD` | verbosity | the share of `PHRASE_MAX_STEPS` the walk may spend **after** the slots are satisfiable | no extra chunk: the line is the canonical copy |
| `STYLE_DEPTH` | formality | `edge_kind_is_claim(kind)` arcs (0..=5) over the associative `6 cooccur-weak` one (**OD-186**) | no kind preference |
| `STYLE_LEAN` | playfulness | the far end's `Graph::out_degree` bucket over a leaf | no degree preference |
| `STYLE_RECENCY` | habit | the arc's `bake` and its arena position (see **OD-182**, §10) | no recency preference |
| `STYLE_PROXIMITY` | particle affinity | hop-1 from the anchor over hop-2 and hop-3+ | no distance preference |

**Why `hi = 2048` and `grid = 16`, derived rather than chosen.** A draw is
`map_fraction(stream.next_q12(), lo, hi, grid)` and `next_q12()` yields
`0 … 4095`, so `delta = round(span · f / 4096) ≤ span`. The `hi` corner is
reachable **iff** `span ≤ 2048` — that is **OD-161**'s corner-reachability rule,
found on the seed panel and still binding. `2048` is the largest box whose two
corners and every grid point a draw can actually land on. `grid = 16` gives 129
distinct values: `2048/16 + 1`. And `lo = 0` is the inert point, so a leg can run
the whole axis at `lo` under **OD-95**'s floor discipline and *mean* it: an axis
drawn `0` adds exactly `0` to a weight.

**Why the leans are additive.** A Q12 multiplier above 1.0 needs a widening the
ignition sampler does not have (`idle.rs` sums `mul_q12_u16` terms on a `u64`
accumulator and the cumulative scan normalizes). The tongue is the same shape:
`weight = arc.weight + Σ leans`, each lean `mul_q12_u16(style, property_share)`,
so a style value is *a share of a lean*, normalized by the scan. Multiplier > 1.0
would be a second arithmetic law in a second crate.

`exclamation` (§27.3's sixth draft axis) is **not implemented and not a row**:
punctuation carries no point and no edge, so a draw for it would be a value over an
empty property set. **OD-163** is the owner's seat to rule it back in with a
substrate that can express it.

### 3.2 The guard rows (3)

| name | class | unit / box | value | gate | why this class |
|------|-------|-----------|-------|------|----------------|
| `DERIV_ORDER_TONGUE` | **Law** | `Count`, `5 … 5` grid 1 | `5` | G20 | `DERIV_ORDER`'s own precedent: the draw count is a schema fact, and a Law row's `lo == hi` makes any other length a registry contradiction |
| `PHRASE_MAX_STEPS` | **Species** | `Count`, `1 … 65535` grid 1 | `—` (`UNMEASURED_PLACEHOLDER`) | G20 | 10 §2's test 1: it bounds a mechanism, so it may not be individual. Its `hi` is the type maximum because **OD-83** is unrulled, so its sweep publishes a **refusal** (§6.5) |
| `TONGUE` | **Ruling** | `Bool`, `0 … 1` | `Some(0)` = **off** | G20 | `INFERENCE_PROMOTION`'s and the arc's six precedent: the least-authority default, installable through the ordinary launch channel (**OD-178**) |

`STYLE_SPREAD` is a *share of* `PHRASE_MAX_STEPS`, never a second bound. That is
the same treatment `LAMBDA_SILT` gets from the one curve (12 §0 fact 5): the species
row owns the ceiling, the draw owns where inside it an individual sits, and neither
can exceed the other.

### 3.3 The registry arithmetic (prove this before coding)

`placeholders_match_registry()` is three conjuncts and the tongue passes all three
**only** with `sweep: None` on the five style rows:

| bucket | before | after | why |
|--------|--------|-------|-----|
| `placeholder_count()` == `swept_species_count()` | 46 | 47 | `PHRASE_MAX_STEPS` is the only row that qualifies; the five `Tongue` rows are excluded by `class != Species` **and** by `sweep.is_none()`, so the two sides move together |
| `temperament_count()` == `DERIV_ORDER_LEN` | 18 == 18 | 18 == 18 | the fifth class is not `Temperament`; `Class::Tongue` rows appear in neither count |
| `swept_row_count()` == placeholders + declared + non-scalar + temperament | balanced | +1 == +1 | a `Tongue` row with a sweep tag would add to the left side and to **no** bucket on the right ⇒ the check fails. This is the mechanical proof that OD-177 (the style rows are never swept) is not a preference but a consequence |

Total rows **95 → 103**; `check_registry_size`'s message reads 103.

### 3.4 The version consequence (read this before coding anything)

* `si_core::consts::CONSTS_VERSION`: `0x0002` → **`0x0003`**, at **B20**.
* `si_core::hash::EMPTY_STATE_MD5_SEED_ZERO` re-pins (its value is a function of
  item 8's `CONST_ROWS.len()` and the version tag), and the three tests that assert
  it move **in the same commit**: `si-core/tests/kernel.rs:131`,
  `si-session/tests/seed.rs:12`, `si-session/tests/wake.rs:23`.
* `HASH_FORMAT` stays `0x0001`, `SEED_FORMAT` stays `0x0001`, `DERIV_ORDER` stays
  18 values, `temperament_digest` is untouched, `CORE_BLOCK_BYTES` stays 36.
* Every artifact recorded under `0x0002` remains attributable to the arithmetic that
  made it (05 §21's rule) and is **superseded by a fresh run, not edited** — the
  full battery G1–G19 re-runs at B20 (§8's DoD 5), and `G20(j)` is the leg that
  proves the strings did not move while the hashes did.
* The `0x0003` build re-signs every recorded control **by re-running it**
  (**OD-91**). `gates/deps_lock.md5` **is re-published**: the tongue adds no
  dependency — `md-5`, `anyhow`, `serde` are already in the allowlist and
  `si-tongue` uses none of `serde` (its `License` is an in-RAM argument, never
  serialized by the crate itself) — but a workspace member is itself a `[[package]]`
  row in `Cargo.lock`, so the hashed file changes the moment the crate joins. Every
  prior new crate (B4, B14, B15, B17) re-published it the same way; the draft's
  "does not move" was wrong about the mechanism, not about the allowlist law
  (**OD-185**, **OD-42** untouched).

### 3.5 §1.2's class column gains 8 rows and **one new class**

`Class` becomes `{ Law, Species, Temperament, Ruling, Derived, Tongue }`. The sixth
variant is **OD-175's ruled answer** and it is a real widening, not a renaming: a
`Tongue` row is drawn like a temperament row and refused like a law row. Three
existing filters are named here so nobody "tidies" them into the new class:

| filter | site | required reading with `Tongue` present |
|--------|------|----------------------------------------|
| `launch_constants`' class guard | `runner.rs:1917` | **no edit.** `matches!(row.class, Species \| Ruling)` already refuses a `Tongue` name, and `check_sweep_never_touches_temperament` string-matches that exact expression (`checks.rs:1346`) — changing the text breaks the check, and loosening the guard breaks OD-87 |
| `axes_of()` | `sweep.rs:100` | **no edit.** filters `class == Species && scalar && sweep == tag` |
| `placeholder_value()` | `consts.rs:266` | **no edit.** requires `class == Species`, so a draw gets no invented value |

`check_temperament_closed`'s second loop ("a LAW/ruling row must not be drawn",
"`TEMPERAMENT` but never filled") gains a **parallel** clause for the fifth/sixth
class pair, and the parallel clause belongs to `check_tongue_closed` (§6.2 row 1),
not to it — the 18's law keeps its exact mechanical meaning (**OD-175**'s wording).

---

## 4. Delta to [02](02-module-contracts.md) — the tick, the contract, the grep rules

### 4.1 The tick, with one insertion

Steps 1–13 keep their numbers, their order and their contents; the tongue adds
**one** insertion and moves nothing. `Xa` means "inside X, after X's own body".

```
1–9   unchanged (front.tokenize / decode / walk.light / walk / settle /
      read.gaps → affect.prioritize → budget → emit / front.label + calibrate)
9a    B20 tongue: if the `TONGUE` ruling row is installed on —
        license = cited rows → (key_point, val_point) surfaces + each row's own
                  composed surface, folded by si_front::tokenize at the boundary
        seed    = render_seed(birth_seed, style.digest(), thought_id)        [OD-183]
        line    = si_tongue::realize(&graph_at_t, anchor, &license, &slots, &style,
                                     phrase_max_steps, &mut Rng::seeded(seed))
        surface = line.unwrap_or(rendered.surface.clone())
      Off ⇒ the arm is not entered; `surface` is `rendered.surface.clone()`, the
      expression this file did not touch (G20(j))
10    front.reverse_decode's copy is still what the fallback prints; stream.siro
      writes the same `Output::Answer { surface, label, tag, citation }`
11–13 unchanged, incl. 12/12b: idle mint nothing, sleep reads no voice
```

Insertion position, in code: [crates/si-session/src/lib.rs:2002](../crates/si-session/src/lib.rs#L2002)
— inside the `if !rendered.surface.is_empty() && label != "I-DON'T-KNOW"` arm, after
`label`/`tag`/`citation` are bound (line 1972–1992) and before the `EV_OUTPUT` log
(line 2008). `Output::Ask` (line 1951) and `Output::Inert` (2024) keep the canonical
render, and `si_front::render_monologue` keeps its own (§27.7: the internal voice
stays canonical until the owner rules it into the tongue).

**Why the label is bound first, twice stated.** Check (a) is not a promise about
ordering: `si_front::label` reads `Rendered.content` and `Rendered.containment` and
**never** `Rendered.surface` (`crates/si-front/src/render.rs`), so a surface rewrite
cannot reach a label by construction — and binding `label` before the call makes the
type system refuse a second, realized copy of it.

### 4.2 `si-tongue`'s contract (lands as **02 §18**, the seventeenth crate, in §14–§16's table style)

| ordered inputs | outputs | error behavior |
|----------------|---------|----------------|
| `&Graph` (the tick's own, already built by the caller — the crate cannot build one, because `Graph::build` wants `&Lattice`), the anchor point handle, `&[License]`, `&[String]` slot keys (already folded), `&Style`, `phrase_max_steps: usize`, `&mut Rng` | `Option<String>` — the realized surface, or `None` | no `Result`, no `panic!`, no `bail!`: every failure mode is a refusal, and a refusal is `None`. A missing license entry, an empty arc set, a spent budget and a lost slot are all the same answer |

The crate `may NEVER`: touch `si_store::store::Lattice` (no import); call
`si_learn::commit` or any row writer; read `si_affect` (no `a`, **OD-168**); read
`Graph`'s `a_at` (**OD-182**); write a file or a log line (the harness logs, §1.3);
call `si_front::label` or read a label; emit a string that is not a `License.surface`;
change `citation` (the caller appends the event ids after the call, unchanged).

`License { point: u32, surface: String, keys: Vec<String>, variants: Vec<Variant> }`
is declared in `si-tongue` and constructed in `si-session`. It is RAM-only.
**OD-189** adds the fourth field: `Variant { surface: String, keys: Vec<String> }`
is another taught spelling of the *same* point — same `keys` after the one fold,
different bytes. The walker draws among `surface` plus `variants` with one
`next_q12` **only when a real alternative exists**; a bank of one spends no draw,
which is why the bare fixture's bytes are the pre-bank bytes. Nothing else in the
contract moves: the option's string still comes from a license surface, the
citation is still appended by the caller, and no variant is a row.

### 4.3 Grep rules (xviii)–(xxi), restated as build checks

02 §12's table already ends at **(xvii)** — "a speaker class is a function, not a
field", minted by B18. The tongue's four rules continue that sequence; they are
written into 02 §12 in the same column shape, and the numbers below are the ones
that appear there.

| # | rule | the check |
|---|------|-----------|
| (xviii) | the voice is drawn, never installed and never swept: no `STYLE_*` name appears in a launch-constant list, a scenario's `constants` block, or any published sweep artifact's `axes` | `check_style_never_installed` |
| (xix) | `si-tongue`'s dependency set is exactly `si-core` + `si-walk`; the strings `si_store` and `si_front` appear nowhere in its sources | `check_tongue_is_leaf` |
| (xx) | the walker's only mouth is a `License`: `si-tongue` contains no `&'static str` table with more than one word, no `fs::`, no writer symbol, and its phrase buffer is built only by pushing a license surface | `check_tongue_license_only` |
| (xxi) | no arousal byte reaches a weight: `si-tongue` never reads `a_at`, `a()`, `arousal`, and the render seed carries no `a` | `check_tongue_reads_no_a` |

Plus one that is a *moved* check, not a new rule: `check_emotion_words`'s scope
gains `crates/si-tongue/` (R-21's extension — the tongue module carries no emotion
vocabulary; style readouts live in audit only).

---

## 5. Delta to [04](04-algorithms.md) — §34, the phrase walk, in full

### 34.1 The style derivation (the seed-only half, si-core::tongue)

```
style_stream(birth_seed)  = Rng::seeded(birth_seed ^ TONGUE_DOMAIN)
TONGUE_DOMAIN             = 0x544f_4e47_0001_0001   // "TONG" + a format tag, a fixed constant
derive_style(seed)        = for (i, id) in DERIV_ORDER_TONGUE:
                                map_fraction(stream.next_q12(), row.lo, row.hi, row.grid)
Style::canonical_bytes()  = 5 × u16le  (10 bytes)
Style::digest()           = first 8 bytes of MD5(canonical_bytes), LE, as u64
```

**Why a domain-tagged stream and not stream T's continuation.** `birth_streams(s)`
returns two clones of `Rng::seeded(s)`, so continuing T's sequence would mean
skipping exactly 18 draws — which makes `STYLE_SPREAD` a function of
`DERIV_ORDER`'s length. A future OD-71 flip that moved one temperament row would
then silently re-voice every individual, and the reverse: the style vector would be
a second consumer of the law's own stream. The salt makes the two orders
independent by construction, and it is a *constant*, so the derivation stays a pure
function of `(birth_seed, DERIV_ORDER_TONGUE)` — the same sentence 10 §4's header
writes about the 18, and the same scan `check_tongue_closed` runs.

Stream R is not read at all (§0 fact 5). `runtime_stream` is untouched.

### 34.2 The render seed (**OD-183**)

```
render_seed = first 8 bytes of MD5( birth_seed ‖ style.digest() ‖ thought_id ), LE
draws       = Rng::seeded(render_seed)          // per render, never the session's R
```

Same thought, same seed, same lattice ⇒ same sentence byte-for-byte, across
processes (**G20(a)**), and I5 holds because every operand is a snapshot value. The
draft's list included "the state hash before the line"; the state hash is
**excluded** here because item 3 is the stamp arena and a stamp carries `a_at`, so
seeding from the hash would let arousal choose the words through the back of the
seed — the exact thing **OD-168** bans, graded as **G20(e)**. The exclusion is
OD-183's seat.

### 34.3 The walk, step by step

```
0  anchor            = the answer row's val_point (an ask's named target is not in
                      scope this round: §4.1 realises Output::Answer only)
   used              = { anchor }
   parts             = [ license(chunk_of(anchor)) ]     // the canonical copy, first
1  budget            = (PHRASE_MAX_STEPS × STYLE_SPREAD) >> 12       // a share, §3.2
2  while parts.len() < 1 + budget:
3      candidates   = graph.arcs(current)
                       .filter(|a| !a.is_genome())                   // OD-179: live arcs only
                       .filter(|a| license covers a.to_point)         // OD-181: utter-able
                       .filter(|a| !used.contains(a.to_point))        // no repeats
4      if candidates is empty ⇒ break                                 // a refusal is not a crash
5      weight(a)      = a.weight                                     // Q12, the lattice's say
                      + mul_q12_u16(STYLE_DEPTH,     owned_share(a.kind))
                      + mul_q12_u16(STYLE_LEAN,      degree_share(graph.out_degree(a.to_point)))
                      + mul_q12_u16(STYLE_RECENCY,   recency_share(a.bake, a.edge_off))
                      + mul_q12_u16(STYLE_PROXIMITY, hop_share(distance(current, anchor)))
6                     // u64 accumulator, saturating — idle.rs's shape exactly
7      draw          = one next_q12();  target = (total × f) >> 12; scan the cumulative list
8      append        = that point's licensed chunk; current = that point; used += it
9  line             = parts.join(", ")                                 // §0 fact 8
```

The four shares are **structural, not tunable**, and each is a Q12 in `0 … 4096`:

| operand | share | law |
|---------|-------|-----|
| `owned_share(kind)` | `4096` if `edge_kind_is_claim(kind)`, else `1024` (`6 cooccur-weak` and every genome kind) | §27.3: formality = claim-side over associative. 1024 not 0 because a co-occurrence is still an edge SI earned. The predicate is `edge_kind_is_claim` = `owned && kind != EDGE_KIND_COOCCUR_WEAK` — the same rule `si-front::label::is_claim` already writes — because `edge_kind_is_owned`'s last slot *is* the co-occurrence (**OD-186**, seated at build) |
| `degree_share(deg)` | `min(deg, 16) × 256` | §27.3: playfulness walks into the hub. 16 buckets, the last of them saturating, so a hub of 40 and a hub of 16 are one preference |
| `recency_share(bake, edge_off)` | `clamp(bake >> 4, 0, 4096)`, plus `1024` when `edge_off` is above the median of the current arc set | §27.3's "recently-baked / recently-charged", realized with **no `a` operand** (**OD-182**): `bake` is the write-side permanent quantity and arena position is formation order. `a_at` is never read |
| `hop_share(d)` | `4096` at d = 1, `2048` at d = 2, `1024` beyond | §27.3's particle affinity: how tightly it sticks to the thing asked about |

`distance` is the walk's own hop counter, one `u8`, never a new field on any
structure — the graph stores no metric and the tongue invents none.

**Termination is three-sided and each side is a proof.** `budget` (a species bound ×
a Q12 share ≤ `PHRASE_MAX_STEPS`), `used` (a point is emitted at most once, and the
license is finite), and empty candidates. So no style value can make the walk run
away, which is `G20(i)`'s whole predicate; the line length is bounded by
`1 + budget` chunks and `budget ≤ PHRASE_MAX_STEPS`.

### 34.4 The three checks, as code shapes

| check | predicate | on failure |
|-------|-----------|-----------|
| (a) label immutability | the type fact (§4.1) plus: `realize` takes no `&Rendered`, no label, no tag, no citation | — (nothing to fail) |
| (b) licensed emission | every token key of `line` is in `⋃ license[i].keys` ∪ the joiner's (empty) | `None` |
| (c) parse-back | `folded(line) ⊇ folded(canonical surface)` — every semantic slot the line was generated from is still in it (**OD-173**: slots-present, no decode walk invented) | `None` |

`None` is the arm at the call site: `rendered.surface.clone()`, the canonical frame,
printed with the label, tag and citation the run already computed. A refusal moves
nothing else: no row, no log field, no gap, no ask (**G20(g)** forces the loss and
asserts exactly this; the assertion is on the *frame*, not on the crash).

---

## 6. Delta to [05](05-gate-harness.md) — V14, eleven legs, six checks, one refusal

### 6.1 The deletion variant V14 (V0–V13 keep their meanings)

| variant | what is excluded | how | what is asserted equal | legs |
|---------|-----------------|-----|------------------------|------|
| **V14** | `TONGUE` installed **`on`** at a named seed — the *only* variant in the table that adds behaviour instead of deleting it | `si-harness run --gate G20 --variant V14` with `constants: { "TONGUE": 1 }`; the row is `Ruling`, so it rides the ordinary launch channel (12 §3.2's widened filter, `Species \| Ruling`) | nothing is asserted equal **across variants**; `G20(b)` asserts label, tag and citation are byte-equal **inside** the pair, and `G20(j)` asserts the *strings* at `TONGUE = 0` are the copier's own bytes. The state hash is asserted **unchanged by the voice**, which is the whole point of §0 fact 5 | all eleven |

V14 is declared in 05 §2's table by this row and is part of `--gate all` and of
05 §21's per-release-candidate set. `TONGUE` ships **off** at V0 (**OD-178**, ruled:
"OFF at V0, ON at a named variant") — turning it on at birth would re-baseline every
golden string and log line in the repository, and B7's ruling that *a control's
identical hash is the pass* means the pass here is string-level, not hash-level.

### 6.2 The six new static legs (05 §20's table format; **46 → 52 ids**)

| id | scans | pass condition | gate |
|----|-------|----------------|------|
| `check_tongue_closed` | `si-core/src/tongue.rs`, `temperament.rs`, `seed.rs`, `rng.rs` | `style_order_is_closed()`: `DERIV_ORDER_TONGUE.len() == 5 == consts::tongue_count()`, every drawn row is `Class::Tongue`, `size_of::<Style>() == 10`, no non-`Tongue` row is drawn; the four files reach none of `getrandom, rand::, SystemTime, Instant::now, env::var, std::fs, File::open`; the identifiers `style_vector` / `style_values` appear in no other source file; `derive_style(seed)` is stable across two calls and differs from `derive_temperament(seed)`'s draw sequence | **G20(a)(k)** |
| `check_tongue_is_leaf` | `crates/si-tongue/Cargo.toml` + every file in the crate | the dependency set is exactly `{si-core, si-walk}`; `si_store`, `si_front`, `si_learn`, `si_affect`, `si_read`, `std::fs` appear nowhere in its text; the only crate in the workspace that names `si_tongue` is `si-session` | **G20(b)(c)(h)**, OD-181 |
| `check_tongue_writes_nothing` | `si-tongue`'s symbols + the call site | the crate has no `&mut` parameter except the `Rng` it is handed, no `commit`/`append`/`put`/`write`/`save` symbol, and its return type is `Option<String>`; the call site assigns the result to `surface` and to nothing else | **G20(d)(f)** |
| `check_tongue_reads_no_a` | `si-tongue` + `si-core::tongue` | `a_at`, `a()`, `arousal`, `Integrator` appear nowhere in either; `render_seed`'s operands are `birth_seed`, `style.digest()` and `thought_id` and nothing else | **G20(e)**, OD-168/182/183 |
| `check_style_never_installed` | every scenario fixture, every published `*_sweep.json`, `runner.rs` | no `STYLE_*` name appears in a `constants` declaration or in a sweep artifact's `axes`; `launch_constants`' guard text is byte-identical to the `Species \| Ruling` refusal; the `match` arms added at B20 name `PHRASE_MAX_STEPS` and `TONGUE` and **no** style row | **G20(i)**, OD-87/177 |
| `check_tongue_license_only` | `si-tongue`'s literals | no `&'static str` array or table of more than one entry; the only multi-char literal the crate emits is the joiner `", "`; every push into the phrase buffer takes its bytes from a `License.surface` | **G20(c)(h)(k)**, R-21 |

### 6.3 Checks that MOVE (the tongue must not quietly break them)

| check | what moves | why it stays green |
|-------|-----------|--------------------|
| `check_deps` | 16 → **17** expected names; `Cargo.lock` md5 **re-published** to `gates/deps_lock.md5` (the new member is a `[[package]]` row — **OD-185**) | no new external dependency; the allowlist law (OD-42) is untouched |
| `check_temperament_closed` | the seed-only scan list gains `crates/si-core/src/tongue.rs` | the 18 do not move (§0 fact 2); the scan simply stops covering less |
| `check_registry_size` | `{total}` reads **103**; the message's `18 TEMPERAMENT draws` clause is unchanged | §3.3's arithmetic; `Class::Tongue` enters none of the four buckets |
| `check_sweep_never_touches_temperament` | a `G20` prefix now exists in `row.sweep` (only `PHRASE_MAX_STEPS` carries it) | `axes("G20")` enumerates one **SPECIES** axis, so the `axis.class != Species` bail never fires; the guard text is byte-identical (§6.2 row 5) |
| `check_emotion_words` | scope gains `crates/si-tongue/` | the crate declares no word table at all (§6.2 row 6) |
| `check_record_layout` | nothing | no byte moves (§2) |
| `check_one_clock` | nothing | the realization reads no clock; `t` is an argument the caller already has |
| `check_no_seed_control_surface` | scope: `Style` and `derive_style` are read the way `Temperament` is — no setter, no install path | the fifth class is refused by `launch_constants`' existing guard |

### 6.4 G20's artifact dir and its four files

`gates/G20/` (the harness writes; `gates/*/` is gitignored, so a fresh run
supersedes, never edits — 05 §21):

| file | read by | what it proves |
|------|---------|----------------|
| `realized_log.jsonl` | **G20(a)(b)(h)** | one line per realized tick: `{t, thought, seed, anchor, licensed, chunk_cap, cap_note, citation, label, tag, canonical_surface, realized_surface, drive}`. **There is no `drew` list** (**OD-187**): the per-render PRNG never leaves `si-tongue`, so equality is over this row's own fields. Three legs read one file because they are the same emission read three ways — cross-process byte equality (a), frame immutability (b), and membership of the words (h). **OD-189**: `chunk_cap` is a ceiling, and `cap_note` says the license binds first (`G20_CAP_NOTE`), so the number is never read as the line's length |
| `refusals.jsonl` | **G20(c)(g)(i)** | every `None` with the **drive that forced it** (`drive: "budget_floor"`, `forced_by: {constants, fixture}`). The walker's internal names (`no_candidates`, `unlicensed`, `slot_lost`, `budget`) are graded in `crates/si-tongue/tests/realize.rs`, where each arm is constructed, not published here (**OD-187**). **An empty file is a pass for nothing** — the forcing legs make it non-empty on purpose |
| `phrase_sweep.json` | **G20(i)** | a **refusal** (`status: refused`, OD-83 + OD-172 named), per §6.5 |
| `v0_strings.json` | **G20(j)** | every `EV_OUTPUT` surface of the V0 run in step order, each proved to be one cited row's own `surface` byte for byte |

### 6.5 The sweep runner: `G20/phrase_sweep` publishes a refusal

`RUNNERS` gains `("G20", "gates/G20/phrase_sweep.json", "phrase")` and `scenario_path`
gains `"G20" => "harness/scenarios/g20_tongue.txt"`. The one axis is
`PHRASE_MAX_STEPS`, `hi` is `65535`, so `open_ceiling` is true and `sweep_one` takes
its written-refusal branch — the same branch `silt`, `prov` and `shape` take, with
the same OD-83 question and **OD-172**'s own half (what the step bound is *for*).
`objective("phrase")` quotes 05 §24's row rather than paraphrasing it, and
`objective_value("G20", report)` reads `G20.i.1`, `G20.i.2`, `G20.c.1`, `G20.g.1`.

Consequence, stated: **no leg quotes a swept phrase budget.** G20(i) runs at
`PHRASE_MAX_STEPS = 1` (a floor, named as one, **OD-95**) *and* at the registry
placeholder (midpoint, non-binding), and its assertion is that the cap holds at both
and that an over-cap walk is a **refusal**, never a truncation — which is the only
thing a leg can honestly assert while OD-83 is unrulled.

### 6.6 The eleven legs, as procedures

Fixture `harness/scenarios/g20_tongue.txt` (V14 runs it with `constants: {TONGUE: 1}`):
a `seed` line, an owner teach block that gives one anchor row a composed surface and
**at least two term-to-term live arcs with distinct surfaces**, then a question tick.
**OD-189** makes the fixture a pair: `g20_tongue.txt` also teaches **spelled repeats**
of its predicates (`Red`/`RED`, `Warm`/`WARM`, `Cozy`/`COZY`, `Blue`/`BLUE`,
`Cold`/`COLD`) in the
predicate position, so every repeat is a zero-row confirm that interns a new surface
and the bank is non-empty; `g20_tongue_bare.txt` is the same transcript **without**
them. The ordering is part of the fixture's law: every repeat of the `red` slot
precedes `red is bright.` (the one conflict), because a statement after the conflict
is an *insistence* that arbitrates and supersedes the live row — which would flip
the stage and land later teaches as new belief rows. The header says so.
The fixture's own md5 is pinned in the report (`expect_fixture`, 05 §23.5's discipline).

| leg | predicate | procedure |
|-----|-----------|-----------|
| **G20(a)** | replay | the V14 drive runs twice in **two processes**; assert `realized_log.jsonl` is byte-equal and that the state hash of the two runs is byte-equal (the voice moves no item). **As built** (`a.2`): the run's frozen hash = `Run::replay()` = `Run::replay_transcript()` at tolerance 0, **and** a frozen grading drive's per-step voice rows equal the launched run's rows field for field — the `drew` list does not exist (**OD-187**). Second pair with an empty lattice (size 0): every tick is a `no_candidates` refusal and every printed surface is the canonical one — **the fallback, not a crash** (`a.3`) |
| **G20(b)** | label immutability | for each settled frame in the fixture, realize at the five style corners (`0` and `2048` per axis at the panel points a two-seed pair gives) and assert `label`, `tag`, `citation` byte-equal across all of them while `surface` differs on at least one; assert the state hash equal to the same drive at `TONGUE = 0`. **As built — the corner half is not launchable** (**OD-177**/**OD-87**: a `STYLE_*` row is `Class::Tongue`, and `launch_constants` admits only swept SPECIES or RULING rows): `b.1` grades three named **budget** points (`PHRASE_MAX_STEPS` 8/32/64) and `b.2` the **seed panel**, asserting the printed string differs somewhere while every label/tag/citation and the state hash do not. **OD-189's two-fixture pass**: the eight-seed panel runs on `g20_tongue.txt` **and** on `g20_tongue_bare.txt`; with variants, the speaking seeds' transcripts must be **pairwise distinct** (style observable) and at least one taught spelling must actually draw; at the bare control, the speaking seeds must print **one** transcript, byte-identical, named as the expected outcome. Both panels must have every row's frames/state/census equal to the same seed at `TONGUE = 0`, and the protected-shelf census must be equal **across fixtures** — a taught spelling asserts no row. The panel still straddles the budget boundary: three seeds (spread ≤96) refuse every tick, five speak |
| **G20(c)** | no invented words | every realized surface's folded token set is a subset of the license's; plus a forced failure: a fixture row whose only arc leads to a point with no surface ⇒ the run prints the canonical frame and `refusals.jsonl` gains one line |
| **G20(d)** | the two keys still | a stranger line and a `voice`-prefixed grimm line that *name* a shape the tongue liked produce **zero** rows; the V14 run's row count equals the `TONGUE = 0` run's, and `G1`'s seam legs stay green on the V14 build |
| **G20(e)** | arousal isolation | `B_MAX`/`K_TANH`/`LAMBDA_A`-held-constant sweep at every arousal level, citation set pinned by the leg and reported; assert every realized line asserts the same claim for the ticks whose citation set is the same, and print the ticks where arousal legitimately changed *which row was cited* (that is v1 behaviour, not the tongue's). Type half: `check_tongue_reads_no_a`. **As built**: the sweep axis cannot be launched — arousal is not a registry row but the trajectory `a` walks, so `install_launch_state` has no shape for it — and `e.3` is a published `open` rather than a green. `e.2` is the honest substitute: the same transcript with a quiet tick after every sense line moves `a` at every tick, and every tick whose citation did not move must assert the same **claim** — the realized chunks' folded keys, not their bytes. **OD-189**: the pads may ignite and spend a thought id, a sanctioned operand of the render seed, so a taught spelling may differ while the claim cannot; e.1 types the operands, e.2 holds the claim fixed. `e.1` is the type check (**OD-83** remains the ruling that would make the swept form runnable) |
| **G20(f)** | no self-farming | a scripted session that re-enters SI's own realized lines as sense opens **zero** asks and mints **zero** rows; the realized line never reaches `seam.speak`'s input because the call site's output lane is the only consumer. **As built**: the owner channel is a **teaching channel by law** (04 §33), so re-entering a line as `owner:` legally mints the rows it teaches — measured identical at `TONGUE = 1` and `TONGUE = 0`, which is the invariance the voice can actually be held to (`f.1`). The **zero rows** claim is graded on the **stranger** class, where the mouth refuses the line before `tick` (measured: not even a context row) and a session-level stranger pays one context row and no belief; `f.2` then asserts no step of the transcript is a line the voice spoke, and `check_tongue_writes_nothing` grades the call site structurally |
| **G20(g)** | parse-back | the leg forces a lost slot (a license whose anchor surface's last content key is removed) and asserts the canonical frame printed and `refusals.jsonl` says `slot_lost`. **OD-173**: slots-present is the check; no decode walk is invented |
| **G20(h)** | **out-of-citation emission impossible** | grep every realized token against the surfaces of the points *that line's own citation* names, plus the composed chunks; the empty offender set is the pass. The registry-wide grep the draft proposed is **retired** — against a registry-wide set any legal line passes, so it is a leg that cannot fail (**OD-181**) |
| **G20(i)** | budget bound | at `PHRASE_MAX_STEPS ∈ {1, placeholder}` × every style corner: no line exceeds `1 + budget` chunks; an over-cap walk refuses. Artifacts cite `phrase_sweep.json` **as a refusal**, and `observed` says so (**OD-83**, **OD-95**). **As built**: the "style corner" half is not launchable (**OD-177**), so `i.1` grades the registry `lo` (1), the three bracket points (8/32/64) and the placeholder across the **seed panel**; it asserts no line exceeds `1 + budget` and prints, for every tick that lands exactly on its cap, **which bound bit** — `budget` or `license` (**OD-189**). `i.2` reads `gates/G20/phrase_sweep.json` and passes only when it is a refusal naming **no** `STYLE_*` axis |
| **G20(j)** | v1 compatibility | at V0, every `EV_OUTPUT` surface of every existing scenario equals the cited row's own `surface` byte for byte (`v0_strings.json`), and the full battery G1–G19 is green on the tongue-on build. This is the leg that makes the round safe to ship (**OD-178**), and the proof is **string equality**, not hash equality, because the tongue writes no state |
| **G20(k)** | genome containment | under CONFIG ENGINE, assert no realized chunk is reachable *only* over a `kind ≥ GENOME_KIND_FIRST` arc: `!a.is_genome()` is filter 3 of step 2, and a newborn whose live arcs are empty says nothing the species text taught it (**OD-179**) |

Leg ids in the report are `G20.a.1`, `G20.a.2`, … per 05 §1's convention, and every
one carries a `fail_meaning` naming the invariant it guards (I1, I5, I6, I8, I9, R-21,
R-43). **As built** a G20 run reports 21 G20 legs — `a.1–a.3`, `b.1–b.2`, `c.1–c.2`,
`d.1–d.3`, `e.1–e.3`, `f.1–f.2`, `g.1`, `h.1`, `i.1–i.2`, `j.1`, `k.1` — plus the nine
`G14.*` legs every gate report carries (30 criteria). Two ship `open` by design:
`d.3` is the battery clause and is graded by stage B20's verify pass (a G20 evaluator
re-running G1 from inside G20 would recurse into the evaluator that called it), and
`e.3` is the swept-arousal form, which the launch channel has no shape for (**OD-83**).

### 6.7 Sequencing additions (05 §21)

`si-harness run --gate G20 --variant V14` and `--variant V0` join the per-release set;
`check_*` `--all` grows to 52 ids; the B20 stage DoD's machine-checkable items are §8.

---

## 7. Delta to [06](06-resource-budget.md) — cost, honestly labeled

| item | formula | measured? |
|------|---------|-----------|
| realized line length | `1 + (PHRASE_MAX_STEPS × STYLE_SPREAD) >> 12` chunks, each chunk ≤ the longest licensed surface | bounded; `UNMEASURED_PLACEHOLDER` budget ⇒ a legal but non-binding ceiling |
| per-realize work | `O(chunks × out-degree(anchor-chain))` arc reads + one `next_q12()` per chunk | **UNMEASURED** — no millisecond is quoted (06's own law) |
| graph | the tick's `Graph::build(&lattice, t)`, **built by the caller**; `si-tongue` borrows it | the honest cost of a read-only leaf: at V14 a session that already builds a graph per idle tick builds one per realized answer. 06 §15 gets the row; if the profile says it matters, the fix is a caller-side cache, **not** an import |
| RAM | `Style` 10 bytes (RAM, re-derived, never stored) + one `Rng` (20-byte state) + the phrase buffer | derived from the schema, exact |
| hash | **zero** (§0 fact 5) | exact |
| files | **zero new files written by the loop** (§1.3) | exact |

---

## 8. Delta to [11](11-build-plan.md) — stage **B20**, and where it sits

12 §8's table ends with `| B19 | *(not a stage — the regression pass)* |`; **B20 is a
stage**, and it sits after B19 in the build order and after B13 in the *numbered*
order, exactly the way B14–B18 sit between B9 and B10 by build order while keeping
their numbers. `Stage::ALL` gains `B20` **last** so its `Ord` keeps the "cumulative
prefix" reading of `Stage::checks()`.

| stage | subject | files | introduced checks | DoD |
|-------|---------|-------|-------------------|-----|
| **B20** | the tongue | `crates/si-core/src/tongue.rs`, `crates/si-core/src/consts.rs` (8 rows, `Class::Tongue`, `CONSTS_VERSION 0x0003`), `crates/si-tongue/**` (new crate + `Cargo.toml` + workspace member + `tongue` feature), `crates/si-session/src/lib.rs` (one call site, two `ArcConfig` fields, the license builder + **OD-189**'s bank), `crates/si-cli/src/harness/{checks.rs,sweep.rs,gates.rs,stages.rs,runner.rs}`, `harness/scenarios/g20_tongue.txt`, `harness/scenarios/g20_tongue_bare.txt` (**OD-189**'s control), `crates/si-tongue/tests/**`, `crates/si-session/tests/tick.rs` (the zero-row spelling test) | `check_tongue_closed`, `check_tongue_is_leaf`, `check_tongue_writes_nothing`, `check_tongue_reads_no_a`, `check_style_never_installed`, `check_tongue_license_only` | 1. `cargo build --workspace` + `cargo test --workspace` green, incl. the three re-pinned `EMPTY_STATE_MD5_SEED_ZERO` tests **in the same commit** as the version bump. 2. `crates/si-tongue/tests/realize.rs`: the forced-failure table of §6.6 run as unit tests (step-6's per-crate DoD precedent) — no-candidates, unlicensed, slot-lost, budget-spent, replay-identity, empty-license ⇒ `None`; **OD-189** adds the no-bank-spends-no-draw and same-claim-different-bytes pair. 3. `cargo run -- check --all` — **52 ids, zero failures**, and `check_registry_size` reports 103 rows. 4. `si-harness run --gate G20` at V0 **and** V14: all eleven legs pass (a refusal-bearing leg passes on its refusal, printed with its OD). 5. `si-harness sweep --gate G20` writes the refusal, and `phrase_sweep.json`'s `axes` name no style row. 6. **The battery**: every pre-tongue gate G1–G19 re-run green on the 17-crate / `0x0003` build, and for each, the V0 transcript's output surfaces equal the cited rows' own surfaces (**G20(j)**). 7. Nothing is committed under `gates/*/` except the `gates/deps_lock.md5` re-publish that a new workspace member forces (**OD-185**) |

**The three-attempt stop rule and the wall.** A leg that cannot pass because a law
conflicts with a mechanism does **not** get worked around: it becomes
`gates/G20/failure_report.md` plus its OD number, per the README's standing rule.

---

## 9. The break-list — every pre-existing artifact the tongue moves

| # | artifact | file | stage | the move |
|---|----------|------|-------|----------|
| 1 | `CRATE_NAMES: [&str; 16]` | `harness/checks.rs:171` | B20 | → `[&str; 17]`, `si-tongue` before `si-cli` (registry order) |
| 2 | `CONSTS_VERSION: 0x0002` | `si-core/src/consts.rs:16` | B20 | → `0x0003`; item 8 changes for **every** run |
| 3 | `EMPTY_STATE_MD5_SEED_ZERO` + its three tests | `si-core/src/hash.rs:363`, `si-core/tests/kernel.rs:131`, `si-session/tests/{seed,wake}.rs` | B20 | re-pinned in the same commit as row 2 |
| 4 | `CHECK_IDS: [&str; 46]` | `checks.rs:16` | B20 | → 52, alphabetical, with six `run` arms and six `enable_stage` arms |
| 5 | `Stage` | `harness/stages.rs` | B20 | `ALL: [Stage; 20]`, `B20` last (Ord = the cumulative prefix) |
| 6 | `ArcConfig` + `launch_constants` + `arc_from_registry` | `si-session/src/lib.rs:146`, `harness/runner.rs:1903` | B20 | two fields (`tongue`, `phrase_max_steps`), two `match` arms, no change to the class guard's text |
| 7 | the `Output::Answer` arm | `si-session/src/lib.rs:2002` | B20 | the single call site; the `else` value is the expression shipped today |
| 8 | `RUNNERS`, `scenario_path`, `objective`, `objective_value` | `harness/sweep.rs` | B20 | the `("G20", …, "phrase")` row and the three match arms |
| 9 | the `check_temperament_closed` scan list | `checks.rs:1225` | B20 | gains `crates/si-core/src/tongue.rs` (one entry; nothing else in the check moves) |
| 10 | `check_emotion_words` scope | `checks.rs:372` | B20 | gains `crates/si-tongue/` |
| 11 | `gates/README.md`, docs/12, docs/13, docs/15, DESIGN, README | docs tree | B20 | **already moved by the fold** (G20's four artifacts, the 17-crate count, `tongue.py`, the DoD item); the ledger span the fold wrote is **OD-163 … OD-181**, §10's pass extends it to **OD-184**, and the build itself seats the rest — **OD-185 … OD-189** |
| 12 | every recorded control under `0x0002` | `runs/**` (gitignored) | B20 | superseded by a fresh run; nothing is edited |
| 13 | `Cargo.toml` (workspace members, `tongue` feature) | repo root, `si-session/Cargo.toml` | B20 | one member, one optional dependency, and the `gates/deps_lock.md5` re-publish the member forces — no external dep, so OD-42's allowlist is untouched (**OD-185**) |

**What this list does not contain, and that is the point:** no shelf, no envelope
byte, no record width, no `prov_kind`, no edge kind, no frame kind, no hash item, no
signal, no label, no `phrasing` row kind, no `CoreBlock` field, no golden string at
V0. §2's table is the proof and §6.3's "what moves" is the audit.

---

## 10. OD ledger delta — three new seats, and the mechanics the old ones licensed

The fold already holds **OD-163 … OD-181**. Engineering this file found three more
clashes between docs/27 and older law; each is a seat, each runs the least-authority
option, and each is written where the build reads it.

| id | the clash | the lean the build runs |
|----|-----------|-------------------------|
| **OD-182** | §27.3 defines `STYLE_RECENCY` on "recently-baked / **recently-charged**" arcs, while §27.6's I8 row and **OD-168** (ruled: banned) say the tongue reads **no `a`** — and `Arc.a_at` *is* the arousal level at formation, one `u8` away | **OD-168 wins.** Recency = `bake` (write-side, permanent, not `a`) plus arena position (formation order, already the graph's tie-break key). `a_at` is never read, and `check_tongue_reads_no_a` makes that a grep, not a promise. §27.3's "charged" is realized as "recently formed", which is what a charge *was* when it happened |
| **OD-183** | §27.4 seeds the per-render PRNG from "(birth_seed, the style digest, **the state hash before the line**, the thought id)", but hash item 3 is the stamp arena and stamps carry `a_at` — so the draft's own seed operand re-admits arousal through the back of the seed, exactly what **G20(e)** claims to grade | **seed = `(birth_seed, style digest, thought_id)` only.** All three are snapshot values, so I5 and byte-exact replay across processes hold; the state hash is excluded *because* it is arousal-sensitive. A pure function of the snapshot survives; a pure function of the hash does not survive OD-168 |
| **OD-184** | §27.7 gives `si-tongue` three imports — core, store, walk. Importing `si-store` means holding a `&Lattice`, which can enumerate **every** point, i.e. exactly the uncited word **OD-181** forbids an individual to utter | **two imports: `si-core`, `si-walk`.** The surfaces arrive as `License` data built by the caller, so OD-181 is a missing edge rather than a discipline, and `G20(h)`'s grep has a single mouth to grep against. Strictly *narrower* than the law asks, and the narrowing is the whole safety argument |
| **OD-185** | §3.4 and §6.3 and [11 §B20 DoD 7](11-build-plan.md) say `gates/deps_lock.md5` "does not move" while the same round adds a workspace member. `check_deps` hashes `Cargo.lock`, and a member **is** a `[[package]]` row in it — B4, B14, B15, B17 each re-published the file | the lock **is** re-published, in step 2's commit. No external package joins the graph, so **OD-42**'s allowlist is untouched; what moves is the hashed file, not the law |
| **OD-186** | §34.3's `owned_share` is spelled `edge_kind_is_owned(kind)`, and §2's axis table repeats it — but that predicate is `kind <= EDGE_KIND_OWNED_LAST`, whose value **is** 6, the co-occurrence. It answers "which columns does this edge row have", not "is this a claim", so the literal spelling gives `STYLE_DEPTH` no associative side to lean away from | **`si_core::addr::edge_kind_is_claim(kind)`** — `owned && kind != EDGE_KIND_COOCCUR_WEAK`, the rule `si-front::label::is_claim` already writes by hand. docs/27 §27.3's word is "claim-side", so the law decides and both tables here are corrected. Caught by this crate's own formality test at 28/64 draws |

And the mechanics **OD-163 … OD-181 already licensed**, stated here so the code has
one source: OD-165 (separate order) → §5.3; OD-168 (banned) → §6.2 row 4;
OD-172 (Species row, box declared, swept before quoted) → §3.2 + §6.5's refusal;
OD-173 (slots-present) → §34.4's check (c); OD-174 (read-only stats) → §34.3, the
walk mutates nothing but its own `used` set; OD-175 (a sixth class) → §3.5; OD-176
(per-render PRNG) → §34.2; OD-177 (OD-87 wins; never swept) → §3.3's arithmetic;
OD-178 (inert at V0) → §6.1; OD-179 (live arcs only) → §34.3's filter 1; OD-180 (no
new row kind) → §2; OD-181 (citation-scoped vocabulary) → §0 fact 6 and §6.6's
**G20(h)**; **OD-189** (the two-chunk license stands; the variant bank is licensed
grammar over the same endpoints) → §1.2's boundary, §4.2's `Variant`, §6.4's
`cap_note`, and §6.6's b.2 pair and e.2 claim predicate.

---

## 11. What this round refuses, stated so nobody "fixes" it

1. **No nomination detector, no new asker** (**OD-167**). The accent emerges from the
   lattice and the bake, not from a counter that asks the owner's permission; a round
   that adds a writer adds a gate family of its own.
2. **No `phrasing` row kind, no anchor bank, no new shelf** (**OD-180**). The record
   schema has no free tag byte, the shelf registry is closed at 11 and `PatternId` at
   10. Taught sentences are ordinary rows and they act as anchors the *walk* can
   start from — the owner's amendment's own words.
3. **No monologue voice.** `si_front::render_monologue` stays canonical until the
   owner rules it in (§27.7). Idle may not mint anything here (**I9**).
4. **No register-shift by speaker class** (**OD-166**). The tongue sees no `source`
   and no identity byte; the world round (docs/26) is untouched.
5. **No style adjective in the GUI's own words and no emotion vocabulary** (R-21).
   Style readouts are audit-side numbers.
6. **No swept number.** `phrase_sweep.json` is a refusal, so no leg quotes a swept
   phrase budget, and the style rows are never swept at all (**OD-87**/**OD-177**).
7. **No ask-side realization this round.** The call site is the single
   `Output::Answer` arm; `Output::Ask` and `Output::Inert` print canonical frames.
   When an ask gains a voice it needs its own legs, not borrowed ones.
8. **No second fold law and no second tokenizer** (§1.2). The fold runs at the
   license boundary in the crate that already owns the copier.
9. **No stored voice.** `Style` is never serialized, never hashed, never in the core
   block. If the owner ever wants the fingerprint panel to show it, that is 08 §12's
   `TEMPERAMENT FINGERPRINT` row and a ruling, not an engineering choice.
