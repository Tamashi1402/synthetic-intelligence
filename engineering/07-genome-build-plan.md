# 07 — The Genome Build Plan: how the frozen base is made, sealed, loaded, and never mutated

Deliverable 8. The logical source of truth is
[docs/10 §10.2–§10.3](../docs/10-configs-converters.md) (the converter spec),
[docs/05 §5.2–§5.3](../docs/05-substrate-tesseract.md) (the shelves, the frozen
row types, the critical law), [docs/16 §16.2/§16.5](../docs/16-think-walk-hardware.md)
(which web each config walks, residency), [docs/14](../docs/14-risk-register.md)
R-10 (hidden priors) and R-11 (the day a language pack is imported *is* CONFIG
ENGINE). This file adds only what a build needs: stage order, canonical bytes,
record widths for the frozen shelves, the weight mapping, the asserts that seal a
version, and the one place a future source may attach. It adds **no layer, no
shelf, no edge kind semantics, and no content decision.**

**Number rules (this folder's rules 2 and 3, applied to the file that is the
temptation to break them).** This plan contains three kinds of number and no
others: (a) a record width or an enum slot, (b) arithmetic on (a) and on the
volume figures the design itself supplies (~155k words / ~117k synsets / ~207k
relations / ~34M edges — owner-supplied design targets, **UNMEASURED**), and
(c) a count that only a build report can produce, printed here as a blank.
Every byte figure below is **derived arithmetic on a design target**, not a
measurement, and not a claim that any machine holds it. The count report
(stage M.7) is the first real measurement this project takes; until it exists,
quoting a size, a row count or a minute is a violation.

**One sentence of scope:** the genome is the only content SI may be born with,
it is born with it only in CONFIG ENGINE, and everything in this file runs
**offline** — the knowing loop never executes a single stage of it.

---

## 1. What the genome is, and the four things it may never be

| statement | source | consequence for the build |
|-----------|--------|---------------------------|
| The frozen base is **not SI's knowledge** | [docs/05 §5.3](../docs/05-substrate-tesseract.md) "THE CRITICAL LAW" | genome rows live on shelves 1–3, `t = 0`, and can never be cited as OWNED; the labeler caps any answer drawn only from them at **INFERENCE/BASE** ([04 §21](04-algorithms.md)) |
| It loads **only** in CONFIG ENGINE | I2, [docs/10 §10.1](../docs/10-configs-converters.md) | the `engine` feature compiles the loader bodies; off, the loader is a stub that returns `ConfigRefusal` before touching a handle ([00 §4/§6](00-stack-workspace.md)). G5c's handle audit proves it over a full gate run |
| It is **IMMUTABLE** — any change is a full re-version with a new `t=0` tag | docs/10 §10.2 | no build stage, no load path and no runtime path may write a genome byte (§13); the sealed file is opened read-only and mapped read-only |
| Nothing pretrained may sit **inside `src/`** | OD-37, R-34 | the pipeline is a build tool, not a loop component: `si-store::build` behind `engine` + `rayon` (**OD-50**), invoked by one CLI verb, linked by nothing that runs a thought |

**And the four things it may never be,** stated as the failure each would be:

1. **Never a second write path.** The converter writes shelves 1–3 and nothing
   else. A build tool that could open a lattice is R-23's genome-leak shape and
   is prevented by the crate graph (`si-store::build` has no path to `si-learn`).
2. **Never a hidden prior.** Any stage that *chooses* content — prunes a
   subgraph, merges two relation names, re-weights by "importance" — is a content
   decision, i.e. R-10. This plan permits none of them (§5, §6, §8, and **OD-65**
   for the one the owner may grant).
3. **Never a head start.** Genome edges have no `bake`, no charge, no
   `a_at` ([docs/05 §5.3](../docs/05-substrate-tesseract.md) law (a)) — and here
   that is a **record shape**, not a rule (§9).
4. **Never a comparison baseline that moves.** The version tag rides the state
   hash and the container header, so two CONFIG ENGINE runs on two machines are
   only comparable when the tag matches (§13).

---

## 2. The source matrix

Three sources, three shelves. Columns `rows` and `bytes` are **blank by design**:
they are stage M.7's output, and the manifest records whatever comes back.

| # | source | version pin | license (stamped, not ruled) | shelf | emitted row type | design's volume figure | rows / bytes |
|---|--------|-------------|------------------------------|-------|------------------|------------------------|--------------|
| S1 | **WordNet** — synsets, senses, lemma/POS/forms, hypernym + meronym chains | `wordnet@<v>` in the manifest, plus URL + sha256 of the exact file | per-file field, copied verbatim from the source's own notice; [docs/10 §10.2](../docs/10-configs-converters.md) requires it stamped | 1 `dictionary` | `DICT` | ~155k words, ~117k synsets | **UNMEASURED** |
| S1-rel | same, its **paradigmatic chains** | as S1 | as S1 | 2 `relations` | `REL` | ~207k relations | **UNMEASURED** |
| S2 | **ConceptNet** — asserted edges `(a, rel, b, weight)` | `conceptnet@<v>`, URL + sha256 | CC-BY per docs/10 §10.3; stamped **per row-batch**, attribution list exported (§10) | 2 `relations` | `REL` | ~34M edges | **UNMEASURED** |
| S3 | **grammar property tables** — inflection paradigm + property tables | owner-authored file, pinned by sha256 | the owner's own data; **the design does not say who authors it or in what format — lean: hand-maintained TSV in-repo, `data/grammar/`; OD-64** | 2 `relations` (property correlations) + 3 `rules` (paradigm tables) | `REL` + `RULE` | none given | **UNMEASURED** |

**Hypernym chains are L2 rows, not L1 fields** (docs/10 §10.2: "they are
correlations"), which keeps L1 rows pure nodes. `Heat --is-a--> noun` and
`Heat --is--> singular` are `REL` rows too, so grammar lives in the same
structure as everything else ([docs/05 §5.3](../docs/05-substrate-tesseract.md):
"**not** in a third structure").

**No fourth source exists in v1.** An expansion source is §16's subject, and the
manifest's `sources[]` is the only list it can join.

---

## 3. The converter pipeline — twelve stages, their order fixed

`si-cli build-genome --manifest <path> [--out <dir>]`. One verb. It opens no
lattice, no session, no gate directory, and no `runs/` path.

| stage | name | in | out | determinism rule | fail-stop condition |
|-------|------|----|-----|------------------|---------------------|
| M.0 | **MANIFEST** | the manifest file | resolved source list `{id, url, sha256, license, batch_id, adapter}` | the manifest is the lock: nothing is fetched that is not pinned in it | an unpinned source, an empty license field, an unknown `target_shelf` |
| M.1 | **FETCH** | pinned URLs | byte-exact local copies | a hash mismatch is a **hard fail**, never a retry-with-what-arrived | sha256 ≠ manifest |
| M.2 | **PARSE** | source files | per-source intermediate rows (the adapter's own type) | single-threaded per source; sources parse independently | a syntax the adapter cannot parse ⇒ **report + stop** (§5: a drop is a content decision) |
| M.3 | **KEY** | intermediate rows | canonical key strings | verbatim lemma/POS from the source; **no folding here** (§5) | an empty key |
| M.4 | **ALIGN** | keys from all sources | the point table `{hyperpoint_id u64, …}` + the unmatched-surface list | pure intersection on `(lemma, pos)`; order-independent because the output is sorted before it is used | none — this stage cannot fail, only report (§5) |
| M.5 | **EMIT** | aligned rows | fixed-width records (`DICT` / `REL` / `RULE`, §11) | every record `% 4 == 0` asserted at emit; reserved bytes written zero | a value that does not fit its column (**never truncated silently**) |
| M.6 | **SORT** | records | the canonical row file | **total order defined in §4** — this is the stage the version tag depends on | n/a |
| M.7 | **COUNT REPORT** | the sorted file | `genome/<tag>/count_report.json` | output-only; the first measurement in the project (§12) | n/a — an empty report is still a report |
| M.8 | **INDEX** | sorted `REL` | CSR adjacency (§9) + the surface lookup index (§5) | built from sorted input ⇒ buildable by partition and merge; **`rayon` legal only from here in** (**OD-50**) | `max_degree` exceeding the reciprocal table's declared range (§12) |
| M.9 | **SEAL** | sorted file + index + manifest | `GENOME_VERSION_TAG` = `md-5` over the sorted row file (**OD-49**), written into the `SIgen` header | the tag covers kinds (§6), tiers (§8), batch list (§10) — all of them are part of the sealed bytes | n/a |
| M.10 | **VERIFY** | the sealed artifact | pass/fail + the assert log | the density gates (§12), run against the **file that will ship**, not the in-memory tables | any assert failure ⇒ the artifact is not published |
| M.11 | **PUBLISH** | verified artifact | `genome/<tag>/SIgen` + `manifest.lock` + `build_report.json` | atomic write (`name.tmp` → `sync_all` → `rename`, [00 §5](00-stack-workspace.md)); a published tag is never overwritten | a collision with an existing tag whose bytes differ ⇒ **that is a new version**, and the old one stays |

**Order law, and why it is a law.** M.6 before M.7 before M.9: a count report and
a version tag computed over anything but the canonical sorted bytes would not be
reproducible on a second machine, which is the exact property
[docs/10 §10.2](../docs/10-configs-converters.md) asks for ("recorded in the
shelf registry so CONFIG ENGINE runs are comparable across machines"). A build
that indexes before it sorts produces a different adjacency layout per machine,
and the walk's edge-offset tie-breaks
([04 §5 step 6](04-algorithms.md)) would then differ per machine — a determinism
failure that looks like a corpus problem.

**The pipeline is idempotent by construction:** M.1–M.9 read only the manifest
and the pinned files, so re-running it on the same manifest either reproduces the
identical tag byte-for-byte or the source moved and the tag legitimately changed.
There is no third outcome, and a third outcome is a bug in M.3–M.6, not a
version.

---

## 4. Canonical form — the bytes the `t = 0` tag is taken over

`GENOME_VERSION_TAG` is the content hash of the **sorted row file**, so the file
must be one byte string, not a data structure with several encodings. The
canonical form:

| rule | value |
|------|-------|
| endianness | little-endian throughout, same as every live record ([01](01-data-schemas.md)) |
| record order | by shelf (`x` ascending), then by `(t=0, y)` ascending within a shelf, where `y` is the record's index in the sorted file |
| sort key inside a shelf | the **verbatim bytes of the record**, compared as an unsigned lexicographic byte string — never by lemma, never by locale, never by a human collation |
| padding / reserved | written `0`, and M.10 asserts every reserved byte is `0` on read-back |
| strings | interned ids only (`u32`), and the interner itself is part of the sealed file, sorted by id — so a name change is a version change |
| absent | `0` in handle columns, exactly as in [01 §11](01-data-schemas.md) |
| what is **not** in it | the build host, the toolchain version, the wall clock, the file's own name, the manifest's URLs |

The last line is the point: a tag that moved because a build machine changed is a
tag nobody can compare, and the design's cross-machine claim
(docs/10 §10.2) would be decorative. Toolchain and host go in
`build_report.json`, which is **output** (§12), never in the hashed bytes.

---

## 5. Point identity, and the no-silent-drop law

**Identity is verbatim; retrieval is folded. Two index spaces, one point set.**

| concern | key | where | why it is not the other |
|---------|-----|-------|-------------------------|
| source-to-source identity (M.4) | `(lemma, pos)` verbatim, plus `sense_index` where the source supplies it | the point table | folding here would silently merge senses, and a merge is a content decision (R-10) — the plural fold R2 is a *born-in reflex of SI's own tokenizer*, not a corpus operation |
| live lookup at run time | the folded, lowercased, plural-folded surface form ([01 §7](01-data-schemas.md)) | the **surface index**, built by M.8 from every record's `forms[]` + lemma | the walk can only hit the genome with the keys its own tokenizer produces, so the surface index must exist — and it may map *many* surfaces to *one* point without deciding anything |
| collisions | `clas`/`class` in the surface index ⇒ an **alias list**, both points kept | surface index | R-04's policy survives: the fold may alias, it may not merge, and only an owner `P_DEF` splits them at runtime |

**Hyperpoint identity** stays what [01 §1](01-data-schemas.md) says: `hyperpoint_id
u64` = the `md-5` prefix of the canonical key string, "identity not security"
(**OD-49**). Two source rows that hash equal are the same point; nothing else is
inferred from that, and no similarity comes out of a hash (I1).

**Drops.** [docs/10 §10.3](../docs/10-configs-converters.md): unmatched surface
forms get their own L1 rows flagged `provenance: wordnet | conceptnet-only`,
"no silent drop — drops are content decisions". The mechanical reading:

1. a ConceptNet `a`/`b` that intersects no WordNet `(lemma, pos)` **creates a
   `DICT` row** in its own batch (`prov_batch` → the S2 batch), then the `REL`
   row referencing it. It is not discarded, not "fuzzily matched", and not left
   dangling.
2. a source relation name is never mapped onto a near neighbour (§6).
3. a row the adapter cannot parse stops the build (M.2) rather than vanishing
   into a warning count — a warning that nobody re-reads is how 0.4% of a corpus
   disappears quietly, and 0.4% of 34M is a content decision made by a `continue`.
4. the only *legitimate* exclusion is a **documented subset**, and that is
   **OD-65** — this plan ships with none.

**The unmatched-count is a report line, not a gate number** (§12). The design
gives no threshold for "too many unmatched", and inventing one would be the
judgment call the whole section exists to refuse.

---

## 6. The edge-kind space: "verbatim" against a closed `u8` enum

The clash, written down because a build hits it on day one:

| requirement | source |
|-------------|--------|
| `edge_kind` = the relation name **verbatim**, "no relabeling (relabeling = judgment = hidden ontology)" | [docs/10 §10.3](../docs/10-configs-converters.md) |
| `kind` is a **closed u8 enum**; "adding one is a design change, not a build choice"; `7 genome-hypernym … 15 genome-*` | [01 §5.2](01-data-schemas.md) |

A byte with nine advertised slots cannot hold a corpus's relation-name vocabulary
verbatim, and the source's names are not nine.

**The design does not decide how verbatim survives the enum. Lean (OD-62): a
version-tagged GENOME KIND TABLE.**

| property | value |
|----------|-------|
| what it is | a sorted list of the source's **own relation strings**, emitted by M.5 alongside the rows, interned, and hashed as part of the sealed bytes |
| how a `REL` row refers to it | `edge_kind u8` = the table **slot**, valid only when `flags bit1 frozen-genome` is set |
| capacity | 256 slots, `7 … 255` available to the genome (kinds `0–6` stay OWNED-only, [01 §5.2](01-data-schemas.md) unchanged) |
| why this is not relabeling | the table **lists** names; it maps none. Two source relations that a human would call the same thing get two slots. The forbidden act is grouping, and grouping is impossible when the row's slot resolves to the source's string |
| audit | M.7 prints the table and, per batch, `source → name → row count`; a slot whose name is not a substring-free exact match of a source name fails M.10 ("the converter invented a name") |
| version coupling | a new source version that adds one relation name adds one slot ⇒ the table's bytes change ⇒ the tag changes ⇒ re-version (§13). Nothing is silently absorbed |
| over-capacity | **hard fail at M.5**, not a clamp. If the real vocabulary exceeds 249 names, that is a schema-version change to `edge_kind`'s width (01's reserved bytes exist for this and only this) and the owner should see it as a diff, which is exactly why it is a fail rather than a fallback |

**Why the walk stays untouched.** The kind byte is *not* consulted by
`conduction` or the spread — no kind-keyed weight, no kind-keyed gain (that would
be an ontology smuggled as arithmetic, R-14's cousin). It exists for citation,
export, and the audit trail: `[trace] via genome `IsA``, printed from the table.
Kinds are also what G6's OWNED-vs-BASE disagreement leg reads to name which web a
route walked ([04 §5.4](04-algorithms.md)'s `owned=2 base=1`).

---

## 7. RULE rows: the arity finding (OD-63)

`RULE` = `{triggers[…], operator, results[…]}`, composable to depth 1,
cycle-checked at load
([docs/05 §5.3](../docs/05-substrate-tesseract.md),
[01 §6](01-data-schemas.md)). Two facts meet here and do not obviously fit:

- the walk propagates along **binary** edges — `z` targets are `(t,x,y)` points
  and a spread contribution is `activation(source) × weight × damp`
  ([04 §5 step 6](04-algorithms.md));
- a rule row is **n-ary**: `Heat + Ice → Water` has two triggers, and its meaning
  is the *conjunction*.

Compiling it into pairwise genome edges (`Heat → Water`, `Ice → Water`) is
representable and **lossy in a way that changes behavior**: `Heat` alone would
light `Water`, which is not the rule. Emitting it faithfully needs conjunction,
i.e. an edge with a guard, i.e. a structure the substrate does not have — which is
[docs/14](../docs/14-risk-register.md) **R-12**'s alarm: *report it, do not patch
it with a sixth structure.*

**The design does not name a v1 consumer of rule content. Lean (OD-63): rules are
sealed, loaded, indexed and quoted — never conducted.**

| decision | consequence |
|----------|-------------|
| `RULE` rows are emitted, sorted, hashed, version-tagged and mapped like any other frozen row | the registry stays honest (shelf 3 exists and holds the design's row type); re-versioning covers grammar edits |
| they are **not** in the walkable web | the walk's single-loop shape survives; no source-class branch appears in the spread; R-12 is not triggered by a build shortcut |
| they **are** citable | an export or a trace line may quote a rule address `(0, 3, y)` as provenance for a *BASE* observation, e.g. the paradigm table that a future renderer consults |
| the load-time cycle check still runs | it is a property of the data, and it must keep being checked so the day rules do get a consumer the corpus is already clean |
| the inflection case (`Heat --is--> singular, not "Heats"`) | the *property* half of S3 is `REL` rows and **is** walkable — `Heat → singular` is a legal binary correlation. Only the n-ary paradigm tables are parked |

**What this costs, stated honestly:** v1's ENGINE cannot inflect by rule; it
inflects by the same mechanism CONFIG SI uses (born-in shape + taught template
edges), with the genome available as correlations. That is a **scope** loss, not a
law violation, and it is the cheapest thing in this file to reverse later: adding
a consumer changes one load path, not the sealed format. The ruling the owner owes
this item is *who reads rules, when, and through which organ* — a v2 render
question, and it must not be answered by an engineer mid-build.

---

## 8. Weight tiers, and the two different laws they serve

**Grids.** Genome `weight` is `u8`, 255 = 1.0
([01 §3](01-data-schemas.md)); the live edge weight the walk actually conducts
with is `u16` Q12, 4096 = 1.0
([04 §5 step 6](04-algorithms.md)). Conversion is a **shift**: `w_q12 = w_u8 << 4`
— reciprocal-multiply discipline (**OD-45**), no division, no rounding table.

**Two laws that are often confused, and must not be:**

| law | source | mechanism | where it is enforced |
|-----|--------|-----------|----------------------|
| **"OWNED edges weighted above genome edges at NOW"** | [docs/16 §16.2](../docs/16-think-walk-hardware.md) | a **band constraint on stored weight** + a **cursor order at merge** (§9) | the tier table's ceiling, and the merged adjacency's ordering |
| **"OWNED wins, and the disagreement is reported, never hidden"** | [docs/05 §5.3](../docs/05-substrate-tesseract.md) critical law, G6 | **arbitration and citation**, not arithmetic | the labeler and the write-gate arbitration ([04 §12, §21](04-algorithms.md)) |

Keeping them separate matters because they diverge over time: a live OWNED edge
*ages* (§8.4's rehearsal/decay curve, the per-shelf floor), so a numeric band
cannot guarantee OWNED dominance forever. If the band were the only mechanism, an
old OWNED edge would be silently out-conducted by the dictionary and the
disagreement would never surface — the exact failure §5.3 forbids. So: the band
orders **fresh learning above the corpus**; arbitration and the two-citation
readout handle **everything else**.

**Sleep is that paragraph's first real consumer, and it changes nothing.** P3 dims
live OWNED edges toward their shelf floors ([10 §6](10-temperament-sleep.md)), so an
edge can end up conducting *below* the genome band it once sat above. That is not a
new hole, because it was always legal: a superseded OWNED edge has `weight == 0`
([01 §11](01-data-schemas.md)) and is unroutable outright, and dimming is the graded
form of a state the write path already produces. Three consequences, each of which a
builder should be able to point at:

| claim | status after a sleep | why |
|-------|----------------------|-----|
| the two-cursor merge order | **unchanged, and unchanged by design** | §9's W.3 order is a cursor position, not a magnitude comparison — dimming a live edge cannot promote a genome edge into its slot |
| the band (`GENOME_WEIGHT_CEILING`, `weight ≤ 125`) | **still enforced, still only over genome bytes** | it is a constraint on *stored corpus* weight at build time (A4), and sleep writes only L3. **G13(e)** asserts the genome file's content tag byte-identical after any number of sleeps; a sleep that could touch the corpus would be a converter with a new door |
| OWNED-vs-corpus disagreement | **handled by arbitration, exactly as §5.3 says** | a dimmed OWNED edge that loses to a corpus edge produces the same two-citation readout a fresh one would; the panel prints both (O6), which is the whole reason the law was split in two above |

**`GENOME_WEIGHT_CEILING` — kind LAW, derived, not swept.**

| step | arithmetic |
|------|-----------|
| the weakest owned edge at birth | `CONF_INIT_T1_INFERRED` = 50–60 hundredths ([03 §6](03-constants-registry.md)); the build chooses inside the range, so the floor of the range is the guarantee |
| minus one hundredth, strict | 49 hundredths |
| on the Q12 grid | `floor(0.49 · 4096) = 2007` |
| on the u8 grid | `floor(0.49 · 255) = 124`, and because the conversion is a shift the largest legal tier value is **125** (2000 ≤ 2007); `126 << 4 = 2016` is over the ceiling and M.10 fails it |

No genome edge, from any source, at any source weight, enters the walk above the
weakest owned birth value. **This is why it is a LAW and not a sweep entry:**
making the corpus's conduction strength a tunable is R-10 wearing a knob — the
sweep would be choosing how much SI trusts its inheritance over what it was told.

**The tier table itself.** `M.5` maps a source weight into a `u8` tier through a
**closed step table** declared in the manifest:

| property | rule |
|----------|------|
| shape | `(source id, source weight bucket) → u8 tier`, monotone in the source's own units |
| monotone | a higher source bucket never yields a lower tier — non-monotone mapping is relabeling by another name |
| boundaries | authored in the **manifest**, from the source's own declared scale, and **never computed from the data**. A boundary derived from the corpus would move when unrelated rows are added, silently re-pricing every edge — the version-tag trap §13 exists to catch |
| rows the source gives no weight for (S1's paradigmatic chains) | the **top tier**. Not because they are "important" but because the only alternative is weight `0`, which is unroutable ([01 §11](01-data-schemas.md)), and an unroutable row is a drop wearing the no-drop flag (§5) |
| review | M.7 prints the resulting **weight histogram** per batch, so the table's real effect is readable before the artifact is published — a histogram is a review artifact, not a threshold |
| version coupling | the tier table ships inside the sealed bytes; editing a boundary is a re-version |

**Bias on a genome edge, exactly.** [docs/05 §5.3](../docs/05-substrate-tesseract.md)
law (a): frozen layers are *structurally* incapable of holding charge, yet genome
edges "may still be biased at runtime by the read-side term, which touches no
stored byte". The build reading:

| term of `W_eff` | owned edge | genome edge |
|-----------------|-----------|-------------|
| per-edge `β·σ(bake, a_at, a_now)` ([04 §5 step 5](04-algorithms.md)) | present | **absent, because the columns do not exist in the record** (§11) — `σ` has nothing to read. The term contributes `0`, so `W_eff = W` for a frozen edge |
| per-point `γ·m(point, a_now)` | present | **present** — the median is a property of the *point* and is computed read-side from L3 stamps; a genome edge's endpoint can have stamps, and the edge inherits the bias through it without any frozen byte being touched |

That is the whole of "biased at runtime, no stored byte touched", and it is worth
a line because the two ways to get it wrong are both gates' business: giving a
genome edge a bake is R-23 (G9(d)), and giving the point median no path into a
genome edge loses §18.5's reading.

---

## 9. The merged web at wake — ENGINE's load procedure

Five steps, in order, and each one is a *load-time* cost, never a per-thought
cost. CONFIG SI runs **none** of them (it has no genome and cannot open one).

| # | step | detail |
|---|------|--------|
| W.1 | **open** | `si-store::layers_frozen` maps the pinned `SIgen` read-only (the `engine` feature compiled its body; [00 §6](00-stack-workspace.md)). The header's `GENOME_VERSION_TAG` is compared to the manifest lock and to the session snapshot if one is being resumed (§13) |
| W.2 | **point merge** | genome points are allocated **first**, in sealed sorted order, handles `0 … P_gen−1`; live points then allocate from `P_gen` upward on arrival ([01 §1](01-data-schemas.md)). Allocation order is therefore a function of the tag, which is what makes a wake replayable and what keeps `hyperpoint_id`-based dedup one-way |
| W.3 | **adjacency merge** | per point, a **two-cursor run**: the live `z` list (byte-offset arena, `EDGE_RECORD_BYTES = 16`) is walked **first**, then the mapped CSR genome run. Ascending-offset tie-breaks in [04 §5 step 6](04-algorithms.md) therefore resolve in OWNED's favor **for free** — the ordering *is* the §16.2 weighting law, and the spread never branches on a source class (the walk stays source-blind, §17's law) |
| W.4 | **degree counters** | `deg` for the hub-damp table is seeded from the CSR row lengths **plus** the live counts, then maintained live by the per-point `u32` counter [06 §4](06-resource-budget.md) commits to (increment at `append_row`, decrement at `age`, derived so never hashed). Genome degree is included in ENGINE by W.4's seed ([04 §5 step 7](04-algorithms.md)) |
| W.5 | **residency** | `si-vulkan::genome_residency` maps the same read-only buffers as persistently mapped `HOST_VISIBLE\|HOST_COHERENT` ([02 §7](02-module-contracts.md), LoopNet's `tesseract-compute` idiom); **no staging copy**. The v1 walker is CPU anyway (**OD-44**); the GPU path is the day-one consumer of exactly these bytes |

**Zero-copy is the load budget.** [docs/10 §10.3](../docs/10-configs-converters.md)
says CONFIG ENGINE must stay laptop-runnable; materializing a merged edge arena per
wake would copy the whole genome every start (§11's derived 5.5·10⁸-byte figure), so
the mapped CSR form is not a micro-optimization, it is the thing that keeps the
config runnable at all. Freeze/wake semantics as
[04 §25](04-algorithms.md) states them: freeze = the tag, wake = map + replay the
overlay, and the overlay replay **is** the ordinary deterministic loop.

**Two record shapes, and that is the enforcement.** A genome edge is a 16 B
`REL`-derived element with `to_point u32`, `kind u8`, `weight u8`, batch/provenance
bytes and **no** `bake`, **no** `a_at`, **no** `charge_off` columns (§11). An owned
edge is the [01 §5.1](01-data-schemas.md) record with all of them. The clean-room
affect law is therefore a *type* fact: stamping a genome edge is a missing field,
not a policy violation, which is the strongest available form of G9(d) and the
reason [docs/14](../docs/14-risk-register.md) R-23's "unrepresentable, not merely
disallowed" survives contact with a build.

**Sleep reads that paragraph, and it is the same fact one row further along.** P3's
arithmetic touches exactly two columns — `weight` and `last_touched_at`
([10 §6](10-temperament-sleep.md), [01 §5.1](01-data-schemas.md)) — and the genome
element has the first one on a *different grid* (`u8` tiers, §8) inside a
**read-only** mapping (W.1, W.5) and does not have the second one at all. So "a
sleep cannot dim a corpus edge" is not a policy **G13(e)** hopes holds; it is
unrepresentable for the same reason a genome edge cannot be stamped. **G13(e)**'s
job is therefore exactly what a byte-comparison of the sealed `SIgen` content tag
after *N* sleeps is: the test of a claim the record shapes already make, kept as a
test because a schema can be edited and a law cannot.

---

## 10. Licenses, provenance, and the redistribution audit (R-10)

| item | rule |
|------|------|
| stamping granularity | [docs/10 §10.2](../docs/10-configs-converters.md): license **per file**, and §10.3: CC-BY stamped **per row-batch** — so `prov_batch u8` names one entry of a sealed batch table `{batch_id, source, version, license, url, sha256}`, and no per-row license string exists. That is what makes §11's 16 B `REL` record fit, and it is a *design-faithful* saving, not an invention |
| capacity | 256 batches, `u8`, **hard fail at M.5** past that (same discipline as §6's kind capacity) |
| the audit | R-10's redistribution audit is a **build-time task**, not a runtime check: it reads the batch table + `count_report.json` and asks the ordinary question — does the *published* artifact redistribute what each notice permits? Its finding is a document beside the artifact, never a byte inside it |
| attribution | exported as `genome/<tag>/attribution.txt` from the batch table, one line per batch, with the row count from M.7. CC-BY attribution is the reason a drop is *also* an attribution problem: a row that vanished takes its credit with it (§5) |
| what this file does not do | it does not interpret any license and does not claim one is compatible with anything. It records the string the source ships, refuses the build when the field is empty, and hands the judgment to the audit |
| the leak guard that matters | none of this reaches CONFIG SI: the batch table is inside `SIgen`, which the `engine`-off build has no compiled path to open, and whose zero-open-handles G5c asserts across a whole gate run |

---

## 11. Index space and record form — the numbers [06 §8](06-resource-budget.md) asked for

[06](06-resource-budget.md) deferred a decision with a warning: *"if it reuses the
L3 arena convention, `G` is capped by 2.7·10⁸ by construction."* It does not, and
here is the statement.

**Frozen records are not 112 B rows.** The live envelope exists to carry mutable,
provenance-bearing session state (`confidence`, `repeats`, `last_confirmed_at`,
`superseded_*`, `charge_off`, `text_ref`…). At `t = 0` every one of those columns
is either constant or meaningless, and paying 80 bytes of them per dictionary row
would make the genome **8–12× larger than the live lattice it is supposed to
outweigh gracefully**. The registry law ([docs/05 §5.1](../docs/05-substrate-tesseract.md))
is "one `(t,x,y,z)` addressing, no new structures" — it is not "one record width",
and the registry entry already carries `row_schema_version`, which is what a
per-shelf layout is for.

| shelf | record | fields | bytes | `% 4` |
|-------|--------|--------|-------|-------|
| 1 `dictionary` | `DICT` | `hyperpoint_id u64 · lemma u32 · forms_off u32 · forms_len u16 · pos u8 · prov_batch u8 · synset_id u32 · version_tag u32` | **32** | ✓ |
| 2 `relations` | `REL` | `a_point u32 · b_point u32 · edge_kind u8 · weight u8 · prov_batch u8 · pad u8 · version_tag u32` | **16** | ✓ |
| 3 `rules` | `RULE` | `triggers_off u32 · triggers_len u16 · results_off u32 · results_len u16 · operator u8 · prov_batch u8 · pad u16` | **16** | ✓ |
| aux | point table | `hyperpoint_id u64 · dict_row u32 · surface_off u32 · surface_len u16 · batch u8 · pad u8` | 24 | ✓ |
| aux | CSR row start | `u32` per point, `P + 1` entries | 4·(P+1) | ✓ |
| aux | surface index | folded-surface bytes + `(point u32, alias_off u32)` per key | **UNMEASURED** | — |
| aux | interner + batch table + kind table | sealed, sorted by id (§4) | **UNMEASURED** | — |

`DICT`'s `synset_id u32` is a **local** id, not WordNet's numeric key: the
canonical key string (which *does* carry lemma/POS/sense) lives in the interner,
and a row stores one `u32` into it. That is what closes the field list at exactly
32 bytes without dropping anything [docs/05 §5.3](../docs/05-substrate-tesseract.md)
names. (`forms[]` payloads — irregular plural forms, alternates — live in one
sorted payload blob addressed by `(offset, len)`; the blob is part of the sealed
bytes.)

**The two index spaces, declared:**

| space | width | ceiling | binding limit |
|-------|-------|---------|---------------|
| genome edges (CSR element indices, **counted**, not byte-offsets) | `u32` | **4.29·10⁹ edges** | ~43× the design's own 10⁸ residency wall ([docs/16 §16.5](../docs/16-think-walk-hardware.md)), so **handles are not the genome's constraint — bytes are** |
| genome points | shared `u32` handle space with L3, allocated genome-first (W.2) | 4.29·10⁹ points total, genome-first | same — bytes first |
| L3 live edges / stamps | `u32` **byte offsets** into the live arena | `2³²/16 = 2.68·10⁸` | unchanged from [06 §8](06-resource-budget.md); widening there is a schema-version change, and the genome needs none |

**Derived arithmetic on the design's own volume figures — not measurements:**

| quantity | derivation | value |
|----------|------------|-------|
| `DICT` bytes | `117,000 × 32` | 3.74 MB |
| unmatched-surface rows | unknown until M.7 (docs/10 promises they exist, not how many) | blank |
| genome edges (S1-rel + S2) | `(207,000 + 34,000,000) × 16` | 547.3 MB |
| CSR starts | `155,000 × 4` | 0.62 MB |
| **genome resident, edge part dominant** | sum of the above, index and interners on top | ≈ **5.5·10⁸ B before the aux tables** |
| CONFIG SI, same session | L3 only: [06 §3](06-resource-budget.md)'s `1.12 MB` at 10⁴ rows | the config gap is **genome-dominated by three orders of magnitude**, which is the honest reading of [docs/10 §10.1](../docs/10-configs-converters.md)'s "post-N-session gap … UNMEASURED" |

At the design's own 10⁸-edge wall the same arithmetic gives `10⁸ × 16 = 1.6 GB`
(1.49 GiB) — [06 §8](06-resource-budget.md)'s figure, unchanged, because the
record width is shared. **Region paging stays v2+** and the CSR form is what makes
it a *shelf-region* problem rather than a rewrite: a paged genome region is a
contiguous run of sorted `REL` records plus their slice of the CSR index, so the
v2 answer exists inside this layout and needs no new structure (R-12 respected).

---

## 12. Density gates — asserts that must pass, and lines that only report

Run at M.10 against the sealed bytes. The split is deliberate and it is the whole
of R-10 applied to a build: **a structural property can be asserted; a statistical
property can only be read.** A numeric accept/reject cutoff over content — "reject
the corpus if more than 2% is unmatched" — is a content decision disguised as a
quality gate, and it is forbidden here.

**Group A — asserts (any failure = the artifact is not published).**

| id | assert | the failure it prevents |
|----|--------|------------------------|
| A1 | every `REL.a_point`, `REL.b_point` resolves to an allocated point | a dangling edge, i.e. a silent drop with the file left intact |
| A2 | `a_point ≠ b_point`; no self-loop emitted | an edge that conducts into its own source and makes `reach(a)`'s top-k a lottery |
| A3 | `edge_kind` slot ≤ the table's declared count, and every slot resolves to a name that appears verbatim in a source file | the converter inventing an ontology (§6) |
| A4 | `weight ≤ 125` (the ceiling row in §8), and the tier map is monotone over the manifest's buckets | the corpus out-weighting fresh learning |
| A5 | every row's `prov_batch` resolves; every batch has a non-empty license field | unattributable content in a CC-BY corpus (§10) |
| A6 | every reserved byte is `0`; every record length `% 4 == 0` | a schema-version lie ([01 §11](01-data-schemas.md)) |
| A7 | `max_degree ≤ DEGREE_TABLE_RANGE` from the sealed header | an index-out-of-range in the hub-damp reciprocal table at the first thought (W.4), i.e. a crash at the least testable moment |
| A8 | the surface index round-trips: fold(surface) → key → point, for every form the source lists | a genome row nothing can ever reach |
| A9 | the `RULE` cycle check (depth-1 composition, [docs/05 §5.3](../docs/05-substrate-tesseract.md)) | an infinite paradigm — load-time, since the walk never reads rules (§7) |
| A10 | `md-5(sorted row file)` recomputed from the published artifact equals the header tag | a file edited after sealing (§13) |
| A11 | the build opened no lattice, no `runs/` path and no gate directory (a handle assertion on the tool itself) | the converter becoming a write path (§1) |
| A12 | the CONFIG SI twin binary, built `engine` off, still passes G5c's zero-open-handle audit **with this artifact on disk** | "it is only unopened because it is absent" — the file must exist and stay unreachable |

**Group B — report lines (printed, read, no threshold).** row counts per
source/batch/shelf · the kind table with per-name row counts · the weight
histogram · the degree histogram with `max_degree` and the top-20 hubs · the
matched / unmatched surface split and the list of batches with zero matched
points · the average fan-out per POS · the artifact bytes and the tag ·
per-stage timings (§14) · the interner's entry count and total payload bytes.

**Why Group B is not a gate.** The hubs in A7's degree histogram are the same
quantity `HUB_DAMP_EXPONENT` exists to tame
([docs/16 §16.1](../docs/16-think-walk-hardware.md): "hubs make every thought the
same mush", R-15). G2's anti-mush leg measures that **in CONFIG SI, where no
genome exists**; the report is therefore a *preview* of ENGINE's behavior, not a
pre-measurement of it, and it may never be used to tune `HUB_DAMP_EXPONENT` before
G2's sweep runs — a constant chosen against a corpus histogram would be a swept
value smuggled into the build ([03](03-constants-registry.md)'s rule: the sweep
artifact chooses, or the default stays `—`).

---

## 13. Re-versioning, resume refusal, and the version ladder

| event | consequence |
|-------|-------------|
| any content change to any source | a **full re-version**: new sorted file, new tag, new `t = 0` ([docs/10 §10.2](../docs/10-configs-converters.md) "IMMUTABLE — any change ⇒ full re-version"). There is no in-place edit path, and the mapped file is opened read-only so there is no handle with which to try |
| a tag change | every CONFIG ENGINE session started under the old tag is a different individual. `GENOME_VERSION_TAG` is item 8 of the state-hash inputs and a container header field ([01 §10](01-data-schemas.md)), so the hash, the replay and the gate report all name the base they were computed on |
| resuming a snapshot whose recorded tag ≠ the artifact's tag | **refuse** — LoopNet's geometry-mismatch refusal, same shape ([00 §5](00-stack-workspace.md)): a resume that would change what `t = 0` means is a meaning change, and it aborts rather than "migrating" the session |
| gate comparability | G6 (ENGINE) pins the tag and re-asserts L1/L2 checksums before/after a full session; a G6 result is only comparable to another G6 result at the same tag ([05 §…G6](05-gate-harness.md)) |
| the version ladder | stages and unlocks are **unchanged** by anything in this file: a genome version grants no stage, unlocks no version, and does not advance a single step. [docs/10 §10.4](../docs/10-configs-converters.md)'s rule for the supporter extends here — *SI's stages do not advance because the base got better* |
| multiple tags on one machine | allowed as files; **one** is pinned by `manifest.lock`. An ENGINE session names its tag at open and cannot switch mid-session; swapping a base mid-session is a new SI, not a new setting |

**Old tags are never deleted** (I4's reasoning generalized: history is kept, aged,
never dropped). Storage growth of the genome directory is a report line and an
operator concern, not a law.

---

## 14. Time and size budget — how it will be measured, not what it will be

The design's only stated budget is qualitative: *CONFIG ENGINE stays
laptop-runnable* ([docs/10 §10.3](../docs/10-configs-converters.md)) and *residency,
not compute, is the eventual wall*
([docs/16 §16.5](../docs/16-think-walk-hardware.md)). Both are tested by §11's
derived arithmetic and §12's asserts; neither is a minute.

| measured at | instrument | lands in |
|-------------|-----------|----------|
| every stage M.0–M.11 | `SI_PROFILE=1` env-gated phase timers, compiled no-op when unset ([00 §5](00-stack-workspace.md), LoopNet's `TESSERACT_PROFILE`) | `build_report.json` → `{stage, rows_in, rows_out, bytes_out, wall_ns}`, one row per stage |
| whole build | same | the report's total line |
| `SIgen` open → ready-to-walk (W.1–W.5) | the same timers, run by the load path | the gate report's `engine_load` block, which is a **G6 field** ([06 §10](06-resource-budget.md)'s checklist) |
| per-thought cost on the merged web | [06 §4](06-resource-budget.md)'s edge-touch arithmetic, then G6's measured leg | `G6/<config>_<ts>/gate_report.json` |
| residency headroom vs the 10⁸ wall | arithmetic now, device reality later | the same G6 block |

**Every one of those cells is blank today.** The first build produces them; until
it does, no timestamp, byte count or "takes about a minute" may be quoted anywhere
— which is the folder's rule 3 and the owner's standing honest-benchmark rule, and
the genome is exactly where they get broken, because a build has a wall clock in
front of it and an obvious temptation to write the number down in the spec.
Two honest statements instead of a figure: the pipeline is **parallel after
M.6** (partition-sorted emit and CSR build), and `rayon` is legal there and
nowhere else (**OD-50**) — so the build's wall-clock problem, if it has one, has
a solution that does not touch the loop.

---

## 15. CONFIG SI's mirror: the twin that must stay empty

The clean-room claim is not "SI without a corpus performs worse"; it is that the
*same code* performs with a base it provably cannot reach. Two artifacts of this
plan carry that:

| artifact | what it is | gate |
|----------|-----------|------|
| **the `—SIgen` build** | `--no-default-features --features vulkan,supporter`: `layers_frozen` compiles to a stub returning `ConfigRefusal` **before a handle is opened** ([00 §4/§6](00-stack-workspace.md)) | G5c's handle audit across a full gate run, with the sealed file present on disk (assert A12) |
| **the twin transcript** | the same `transcript.txt`, run once in SI and once in ENGINE, byte-for-byte inputs, both reports keeping their tags | [05 §G6](05-gate-harness.md) + G5's OWNED/BASE label legs — the gap between the two state hashes is the design's own long-term leg, **UNMEASURED until run** ([docs/10 §10.1](../docs/10-configs-converters.md)) |

One asymmetry to keep visible: the two runs are **not** expected to produce equal
readouts, only honest ones. In ENGINE a genome path may settle and must come out
labelled **INFERENCE/BASE, capped**, with both citations when it disagrees with an
OWNED route; in SI the same question must die at a dark spot and produce an
ask-back. If an ENGINE run ever prints an OWNED citation for a route made only of
frozen edges, the build has lost the distinction the entire config axis exists for.

---

## 16. The expansion interface: one insertion point, sealed (OD-52, ruling PENDING)

The owner's ruling on LLM-assisted genome growth is **pending** and this plan is
complete without it. What "complete" means here: the *where* is fixed and closed,
so the ruling can be about *whether* and never about *how* — because a design
question smuggled into a build is how §21's boundary got redrawn quietly once
already (R-34's shape).

**The one insertion point: the manifest's `sources[]` list and a per-source
adapter implementing the same five stage contracts (M.2 PARSE, M.3 KEY, M.4
ALIGN's key format, M.5 EMIT, and the bucket row it contributes to the tier
table).** Nothing else in this file changes for a fourth source, and nothing in
`src/` changes at all.

| seal | rule | enforced by |
|------|------|-------------|
| E1 **build-time only** | the generator runs offline, as a separate tool/step, exactly like Grimm being outside `src/` (OD-37, grep rule (v)); the loop binary never links it, never invokes it, never knows it exists | the crate graph + A11's handle assert |
| E2 **frozen shelves only** | an expansion may emit `DICT` / `REL` / `RULE` rows and **no L3 row, ever** — a batch that writes a live shelf is not a genome build | `si-store::build` has no dependency edge to `si-learn`; G12(e)'s belief-row scope keeps `prov_speaker` on the live side honest (**OD-59**'s boundary) |
| E3 **BASE-capped forever** | expanded rows are as little SI's knowledge as WordNet's are; any answer through them is **INFERENCE/BASE**, never FACT, no matter who or what generated the row | [docs/05 §5.3](../docs/05-substrate-tesseract.md), [04 §21](04-algorithms.md) — the cap is config-scoped, not source-scoped, and that is what makes E3 free |
| E4 **a batch of its own** | a new `prov_batch` entry naming the generator and its version (`prov: <tool>@<v>`), never merged into S1/S2's batches, so the histogram, the attribution list and the count report can separate corpus from generation | A5 + Group B's per-batch lines |
| E5 **a re-version, not a patch** | an expansion changes the sealed bytes ⇒ new tag ⇒ new `t = 0` ⇒ all ENGINE sessions fork. There is no "append to the existing genome" | §13 |

**What an expansion must never be allowed to do,** listed because each is a
plausible feature request: relabel or merge relation names (§6 — the verbatim rule
binds a generator as tightly as a corpus); prune by "usefulness" (**OD-65**);
assign weights by confidence-in-the-answer rather than the source's own scale (§8
buckets are the only legal input); mint a new edge kind outside the source names
(§6's A3); write bake or charge (structurally impossible, §9); or produce content
that reaches L3 through the owner's *keyboard* rather than the genome — that route
already has a name and a gate (`/teach`, T2/T3, I2's "only as the two frozen
layers").

**The honest framing of the pending ruling.** An LM-generated row is *pretrained
content in SI's shelves* — and I2's text is what makes it a ruling rather than an
implementation detail: content knowledge "exists ONLY as the two frozen baseline
layers, which load ONLY in CONFIG ENGINE", and §5.3 declares those layers **not
SI's knowledge**. So the law permits it, the label cap contains it, and the
question the owner owns is a different one: whether a generated base still counts
as "the dictionary talking", or whether a synthetic corpus turns G6 into a test of
the generator. **OD-52 stays open, E1–E5 stay shipped, and the default is that no
expansion batch exists in `manifest.lock`.**

---

## 17. What this file decided, and what it hands to the owner

**Decided here (HOW, no law changed):**

- the twelve-stage pipeline and its order, with **sort → count → seal** as the
  reproducibility spine, and idempotence by manifest pinning;
- the canonical form the `t = 0` tag is taken over — byte-lexicographic record
  order, host/clock/URLs excluded;
- **identity verbatim / retrieval folded**: two index spaces, alias never merge
  (R-04 preserved at corpus scale);
- no-silent-drop made mechanical: an unmatched form creates its own L1 row, and an
  unparseable row stops the build instead of becoming a warning;
- the frozen record widths (`DICT 32 / REL 16 / RULE 16`) and the reason the live
  112 B envelope is not reused on shelves 1–3; per-batch provenance instead of
  per-row license strings, which is what fits the design's "license per file";
- the **genome index space**: CSR element counts in `u32` ⇒ 4.29·10⁹ edges, so
  bytes and not handles are ENGINE's wall — [06 §8](06-resource-budget.md)'s open
  question, closed;
- the weight law split in two: `GENOME_WEIGHT_CEILING` (LAW, derived to 49
  hundredths / `u8` 125) for fresh learning, arbitration + dual citation for the
  OWNED-wins-at-NOW disagreement rule;
- the **two-cursor merge**, live-run-first, so §16.2's weighting is an ordering
  fact and the spread never branches on source class;
- genome edges hold **no bake/`a_at`/charge columns** — affect on the frozen base
  is unrepresentable at the record level, and the same two missing or foreign-grid
  fields (`weight u8` under a read-only mapping, no `last_touched_at`) make
  **sleep's** dim arithmetic unrepresentable on a corpus edge too, which is why
  **G13(e)** is a byte-compare rather than a permission check (§9's last paragraph);
  the per-point median term remains the only legal runtime bias, exactly as §5.3 law
  (a) intends;
- asserts vs report lines as the boundary R-10 imposes on a build: statistics are
  read, never thresholded;
- re-versioning and **resume refusal** on a tag mismatch;
- the expansion interface as one manifest list + five stage contracts, sealed E1–E5,
  shipped empty.

**Handed to the owner:**

| id | question | lean here |
|----|----------|-----------|
| **OD-62** | how does "relation names verbatim" survive a closed `u8` enum? | a version-tagged **genome kind table** (256 slots, source names listed not mapped, over-capacity = hard fail). The alternative reading of `01 §5.2` — narrowing the genome to nine hand-picked relations — *is* relabeling, and docs/10 forbids it outright |
| **OD-63** | who consumes `RULE` rows, and may they conduct? | **parked for v1**: sealed, loaded, citable, not walkable. Making them walkable needs n-ary conjunction ⇒ R-12's alarm, not a pairwise collapse that changes the rule's meaning |
| **OD-64** | the grammar property tables have no author, format, or licence named in the design | hand-maintained in-repo TSV (`data/grammar/`), owner-licensed, pinned by sha256, treated as source **S3** with the same five adapter contracts. Everything else in this plan assumes they exist |
| **OD-65** | may a genome be **pruned/subset** for the load budget? | **no**, and the default is a full seal. A subset is a content decision (R-10) and an attribution loss (§10); if laptop-runnability ever requires one, the owner chooses the cut and the cut is a named batch — never a scoring function in the converter |
| OD-52 | LLM-assisted expansion | unchanged: **PENDING**, §16, and the plan works with it never granted |
