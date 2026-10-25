# 00 — Stack, Workspace, Process Topology

Buildable answer to one question: *what are the crates, which way do their
edges point, and what has to be true at compile time for §12's one-way street
to stop being a paragraph.* This file changes no law; it turns the laws of
[docs/12](../docs/12-component-map.md), [docs/00 I1–I10](../docs/00-thesis-invariants.md)
and [docs/16 §16.6](../docs/16-think-walk-hardware.md) into a dependency graph
the compiler refuses to violate.

---

## 1. Language, toolchain, workspace shape

| item | value | why / reference |
|------|-------|-----------------|
| language | Rust, edition 2021 | owner's stack law; the reference engine is Rust |
| workspace resolver | `resolver = "2"` | matches LoopNet (`Cargo.toml`); per-target feature unification so the `vulkan` feature does not leak into the CPU-oracle crates |
| crate split | one crate per *enforcement boundary*, not per `*.py` file | the `*.py` names in [docs/12](../docs/12-component-map.md) stay the **logical module ids** (**OD-43**); they are shipped as Rust modules grouped into the **16 packages** of §2 below, mapped one-to-one in §2's table — `si-sleep` has **no** `*.py` id in docs/12, which is **OD-84**; the v2 arc's three (`si-crystallize`, `si-silt`, `si-skills`) do have ids there and arrive at **B14–B17** ([12 §1](12-v2-arc.md)) |
| `pure stdlib / zero deps` | restated as: **no dependency may carry pretrained content, a lexicon, or a corpus** | the phrase in [docs/12](../docs/12-component-map.md) predates Rust+Vulkan; the enforceable law is about *content*, and the proof is a byte-audited `Cargo.lock` hash inside G5c (**OD-42**) |
| config axis | runtime flag `--config si\|engine` (default `si`) **plus** a hard load assert | [docs/10 §10.1](../docs/10-configs-converters.md) says "same code, different load"; I2/R-10 says "no code path to open L1/L2 in CONFIG SI"; reconciled in §6 |
| determinism oracle | the hashed knowing loop walks on **CPU fixed-point**, always | [docs/16 §16.6](../docs/16-think-walk-hardware.md); Vulkan ships day one for **residency + the viewer**, not to run the walk (**OD-44**) |

## 2. The crate graph, and the edges that may not exist

Seventeen packages — the thirteen of **11 R1** plus the v2 arc's three reader-only
leaves (§1 of [12](12-v2-arc.md)) plus the tongue's one speaker-only leaf (§1 of
[13](13-tongue.md)). An arrow means "depends on". **Every law in §12's prose that is
about "module X must not reach module Y" is redrawn here as a missing edge**, so that
wiring it back up is a `Cargo.toml` edit a reviewer sees, not a `use` statement they
miss.

```
si-core ────────────────────────────────────────────────┐   (everything)
   │                                                     │
   ├── si-store ──────────────┐                          │
   │        │                 │                          │
   │        │            si-affect ──┐                   │
   │        │            (clock-only)│                   │
   │        ├──────────────┬─────────┴───────┐           │
   │        │              │                 │           │
   │     si-walk ──────  si-learn         si-read        │
   │     (+ idle)      (gate/reparse)  (retrieve/predict)│
   │        │              │                 │           │
   │     si-front ──────────┴─────────────────┘           │
   │   (sense/decode/render)                              │
   │                                                     │
   │   si-sleep ──► core, store, walk ONLY               │  ◄── NO si-learn edge
   │              (the four passes; §2's row)             │
   ├── si-supporter  (mcp + speakgate)  ──► core, store ONLY
   ├── si-vulkan     (residency + viewer) ─► core ONLY
   ├── si-audit      (the labeler)  ──► core ONLY  ◄── leaf: NOBODY in control depends on it
   │                                                     │
   │   the v2 arc's three readers (B14–B17), all leaves:  │
   ├── si-crystallize ──► core, store (read-only) ONLY   │  ◄── NO learn/affect/walk
   ├── si-silt         ──► core ONLY                     │  ◄── NO store/learn/affect/walk
   ├── si-skills       ──► core, store (read-only) ONLY  │  ◄── NO learn/affect/walk
   │                                                     │
   │   the tongue's speaker leaf (B20):                  │
   ├── si-tongue       ──► core, walk  ONLY              │  ◄── NO store/learn/affect/front
   │                                                     │
   └────────────► si-session (the loop orchestrator) ◄───┘
                     │
                  si-cli (binary: panel, --gate, --replay, --export, view)
                     │
                  si-harness (G1–G20 + check_* legs; dev-only)
```

| crate | logical `*.py` modules it carries (OD-43) | may depend on | may NEVER depend on (the law it enforces) |
|-------|-------------------------------------------|---------------|--------------------------------------------|
| `si-core` | `tesseract.py` (addressing, shelf registry ids), `clock.py`, `eventlog.py`, the row/record **schemas** (types only), `quant`, container format, `rng`, from B18 the **speaker-class function** (`world::class`, [docs/12](../docs/12-component-map.md)'s "function, not a field") — it lives in the leaf because `si-front` *and* `si-learn` must reach it and neither may import the other — and from B20 the **style derivation** (`tongue::derive_style`, `DERIV_ORDER_TONGUE`, `Style::digest`: [13 §34.1](13-tongue.md)) | *(nothing)* | — the kernel; leaf of the graph |
| `si-store` | `lattice.py` (L3 append-only shelves, `active_at`, NOW), `layers_frozen.py` (L1/L2 loaders behind the §6 guard) | core | walk, learn, read, affect, audit — storage is not a thinker |
| `si-affect` | `arousal.py`, `drive.py` (the six doors) | core (**imports the one recency curve from `clock`, defines no decay of its own** — R-29) | store, walk, learn, read — the tissue sees anonymous numbers on doors, never rows or a lattice (**OD-20**) |
| `si-walk` | `walk.py` (+ B14's **frame-scoped scratch**: `CoLight`, ghost, role-graph — constructed and dropped inside a frame, [12 §2.6](12-v2-arc.md)), `idle.py` (ignition sampler; B15 gives it the fifth weight), and from B16 the `prov-link` **traversal**: the `OriginTug` vector — a `si-core` schema type, so neither side needs an edge — arrives as the ordinary `walk(...)` **argument** the session fills at step 5b | core, store, affect | learn — **`idle.py` has no import of the commit/write path** (R-25, I9): because `si-walk` does not list `si-learn`, an idle pass physically cannot call `commit()`. Also never: crystallize, silt, skills, read — the arc hands it *arguments*, it is not the arc's host |
| `si-learn` | `gate.py` (surprise/consistency/arbitration; from B18 the **tier arm** — one test ahead of the ten steps, over the class **value** the session computed), `reparse.py` | core, store, affect | walk, audit, crystallize, silt, skills — the write gate is not a thinker, names no emotion (OD-24), and cannot be *asked* by a detector to mint anything |
| `si-read` | `retrieve.py`, `predict.py`, `askengine.py`, and from B16 `event_view` — the **derived** episodic index (provenance rows → `OriginTug`s, rebuilt at frame load, never stored, [12 §5 §31](12-v2-arc.md)) | core, store, affect, walk | learn — retrieval reads, never commits; and crystallize/silt/skills — the arc is *handed* to retrieval by the session |
| `si-front` | `sense.py` (R1–R3, R12 tokenizer; from B18 it carries the frame's `speaker_identity` and calls the class function), `decode` (frames), `render.py` (template copier + monologue face; from B17 it refuses any surface whose row carries `NAMELESS`) | core, store, read | affect, learn — surface text in/out touches no arousal and writes no belief; and the arc's three — the detectors have no mouth |
| `si-supporter` | `mcp.py` (the seam), `speakgate.py` (the two-layer mouth filter) | core, store (store **read-only**) | **affect, drive, walk** — this missing edge is rule (vi) of [docs/12](../docs/12-component-map.md): neither the seam nor the filter can read `a`, charge, bake, or the median, because the crate they live in cannot import them (L3, R-34) |
| `si-vulkan` | *(new — the engine has zero windowing)* genome residency + viewer swapchain | core | learn, read, walk — Vulkan **displays** buffers, it never decides what SI knows (**OD-44, OD-47**) |
| `si-sleep` | *(new — **OD-84**: docs/12's `consolidate.py` id does not exist yet)* the four passes: replay, integration scan, global rescale, calm reprocessing | core, store, walk (it **runs walks**; it does not own the walk) | **learn** — the fourth missing edge, and the one that makes [10 §8](10-temperament-sleep.md)'s "three of four passes write only FLUID, and the fourth writes nothing" a *compile-time* fact rather than a code-review hope: with `INFERENCE_PROMOTION` off, `si-sleep` has no symbol through which a row could be created. Also never: audit, front, supporter, session (a pass may not render, speak, or label) — and reciprocally **`si-learn ↛ si-sleep`**, so the write gate cannot invite a consolidation |
| `si-audit` | `audit.py` — the label stream + `coupling_observed.json` | core (it reads the persisted **event log**, nothing live) | — and reciprocally **no control crate depends on it**: that reciprocity is R-24 and is what makes G9(c)'s deletion check pass |
| `si-crystallize` | `crystallize.py` — the experience detector ([docs/22](../docs/22-crystallize.md)) | core, store (store **read-only**) | **`si-learn`** — a detector that could commit is a self-teaching ontology; **`si-affect`** — [docs/22 §22.3](../docs/22-crystallize.md)'s first exclusion ("`a` is NEVER read by the detector") stops being a sentence and becomes a missing edge; `si-walk` — it consumes the frame's co-light log **as an argument** ([12 §4.2](12-v2-arc.md)), never by reaching into the walk |
| `si-silt` | `silt.py` — the middle timescale ([docs/23](../docs/23-context.md)) | core (`clock`, `quant`) — **and nothing else** | **`si-store`** — silt tracks point **handles**, not rows, so it cannot read or write a belief; **`si-learn`**; **`si-affect`** ([docs/23 §23.11](../docs/23-context.md): arousal-scaled deposit/decay is *absent from the code*, not zero); `si-walk` — the table is handed to `ignite` as an argument |
| `si-skills` | `skills.py` — the shape detector ([docs/25](../docs/25-skills.md)) | core, store (store **read-only**) | **`si-learn`** — [docs/25 §25.11](../docs/25-skills.md) note 2: "*executing* a skill would require an edge the build graph does not contain"; **`si-affect`** (§25.4 rule 3: the detector reads outcome **rows**, never `a`); `si-walk` — the ratified skill's only reach is an ordering **argument** |
| `si-session` | the loop: `panel` orchestration, state-hash composition, snapshot/replay, config guard | core, store, affect, walk, learn, read, front, supporter, sleep, crystallize, silt, skills, **tongue** | **audit** — the loop never calls the labeler; labels are produced *after* the fact over the log, so the label can never feed back into control (I8, R-24). It *does* call `si-sleep` at tick step 12b ([02 §11](02-module-contracts.md)), which is the only place sleep enters the loop, and it is the **only** crate that calls the arc's three readers — at steps 5a, 5b, 7a and 7b ([12 §4.1](12-v2-arc.md)) — and the only one that calls the tongue, at step **9a** and at exactly one site ([13 §4.1](13-tongue.md)) |
| `si-tongue` | `tongue.py` — the phrase walker **and** the three structural checks ([docs/27](../docs/27-tongue.md)) | core, walk (it **reads** the arc set the session already built; `Graph::build` wants `&Lattice`, so the crate that cannot hold a lattice cannot build a graph — **OD-184**) | **`si-store`** — the seventeenth crate's decisive missing edge: with no `&Lattice` it has **no symbol that enumerates a point**, so [docs/07 §7.5](../docs/07-read-path.md)'s "utter only what the citation licenses" (**OD-181**) is a hole in its import list rather than a rule it obeys, and the licensed surfaces arrive as the `License` argument; **`si-learn`** — a realizer that could commit is a self-teaching voice; **`si-affect`** — **OD-168**'s ban made structural (`a` is not reachable, and `check_tongue_reads_no_a` greps for `a_at` anyway, **OD-182**); **`si-front`** — it must not read a label (check (a) is *before* it by construction), and it must not re-tokenize: the one fold law runs in `si-session` at the license boundary ([13 §1.2](13-tongue.md)); `si-read` — the citation is handed to it as data |
| `si-cli` / `si-harness` | binaries: the REPL, `--gate N`, `view`, export; and the G1–G20 runner + `check_*` | everything (harness may inspect all state) | — dev/observation surface; not linked into a run's determinism |

**Eight structural wins the graph buys, restated so a reviewer can check them
without re-deriving:** (1–4 pre-arc, 5–7 the v2 arc, [12 §1.2](12-v2-arc.md), 8 the
tongue, [13 §1](13-tongue.md))
1. **I9 / R-25 ("an idle walk can notice, never learn") is a missing edge**, not a
   discipline: `si-walk ↛ si-learn`.
2. **R-24 / I8 ("the label never feeds back into control") is a missing edge**,
   not a code-review hope: `si-session ↛ si-audit` and `si-audit ↛ control`.
3. **R-34 / L3 ("a gate that could see arousal is one refactor from `soothe()`")
   is a missing edge**: `si-supporter ↛ si-affect`.
4. **"Sleep creates no row" is a missing edge**, and it is I9's argument extended
   to a new organ by the same trick rather than by a new promise:
   `si-sleep ↛ si-learn` ([10 §8](10-temperament-sleep.md)). With
   `INFERENCE_PROMOTION` off — the shipped default (**OD-58**) — there is no
   symbol in `si-sleep` through which a belief row could be made, so
   [06 §2](06-resource-budget.md)'s "any later pass may touch FLUID and nothing
   else" is enforced by `Cargo.toml`, and `check_sleep_writes_nothing`
   ([05 §20](05-gate-harness.md)) only re-checks what the build already refuses.
5. **"Crystallize may nominate, never mint" is a missing edge.** With
   `si-crystallize ↛ si-learn` the crate has no symbol through which a row, an
   edge, a bake or a confidence byte could move; its only output is a `Nomination`
   value that `si-session` turns into an ordinary `drive::Gap` ([12 §4.1](12-v2-arc.md)
   step 8, ordering law 16). G15(a)/(b)/(d) are that edge plus
   `check_crystallize_no_commit`.
6. **"Silt writes nothing, not even an ask" is a missing edge** — and the sharper
   half is `si-silt ↛ si-store`: the table is keyed by `u32` point handles, so silt
   is *physically unable* to name a row, a shelf, or a `context`/`questions` byte.
   [docs/23 §23.0](../docs/23-context.md)'s "the one mechanism of the arc that
   writes nothing at all, not even an ask" is a type-system fact.
7. **"A skill is a row, not a program" is a missing edge** (`si-skills ↛ si-learn`,
   `si-skills ↛ si-walk`). The only way a skill could be *executed* is an argument
   the walk accepts, and the argument is an ordering preference — which is OD-118's
   recommended half and nothing more.
8. **"A line utters only what its citation licenses" is a missing edge**
    (`si-tongue ↛ si-store`, **OD-184**). [docs/07 §7.5](../docs/07-read-path.md)'s
    copy law (**OD-181**) is the tongue's whole safety argument, and a crate holding a
    `&Lattice` could enumerate every point in the world. Without that edge the walker
    has **no symbol that names a point it was not handed**: the surfaces, and only the
    surfaces, arrive in the `License` argument the session builds at the call site.
    `G20(h)`'s grep therefore has one mouth to grep, and `check_tongue_is_leaf`
    ([05 §20](05-gate-harness.md)) grades the import list rather than the intent.

**Wins 5–7 are why the arc is safe at all**, and why G15(b), G16(b) and G18(b)'s
deletion tests are *independent*: removing a reader cannot perturb a writer,
because no writer can name a reader. **Win 8 is the same trick pointed the other
way** — at what a *speaker* may say — and it is why `G20(j)`'s deletion test ("remove
`si-tongue` and its one call site ⇒ the canonical frame returns, byte for byte") is
checkable by the missing edge alone: the crate writes nothing, so its whole removal
surface is one `Option<String>`.

## 3. Emotion-word law, made buildable

R-21's grep ("emotion names appear **only** in `audit.py`") cannot be a `use`
constraint, so it is a CI law over the crate split, checked in the harness
([05](05-gate-harness.md) `check_emotion_words`):

- the six label words (`curiosity surprise satisfaction caution boredom
  frustration`) are **string literals legal only under `si-audit/`**; any
  occurrence elsewhere fails the build. This is why `si-affect` and `si-walk` are
  separate crates — the grep has a clean directory boundary to scope to.
- `arousal.py` / `drive.py` / `gate.py` carry **no** such literal and hold no
  bool-shaped "confused"-style state (§3.4's empty-state litmus; §20.3: nothing
  fires and nothing is readable at `t=0`).
- the arc's three crates are in the **wordless** set by the same law
  ([docs/12](../docs/12-component-map.md), and [docs/23 §23.7](../docs/23-context.md)'s
  R-21 extension: "the emotion-word grep list is unchanged; `silt.py` must contain
  no emotion vocabulary"). `crystallize.py`, `silt.py` and `skills.py` gain no
  exception at B14 — `check_emotion_words`'s scope simply grows three directories.
- the grep is a **static** test, so it needs no run: it is a `check_*`, not a
  numbered gate leg (**OD-53**).

## 4. Feature flags

| feature | default | effect | enforced by |
|---------|---------|--------|-------------|
| `vulkan` | **on** | compiles `si-vulkan`: genome residency + the viewer swapchain (`ash`). Off ⇒ pure-CPU build | the walk lives in `si-walk` and does **not** list `si-vulkan`; turning Vulkan off changes nothing the walk reads — G5a determinism runs on both builds and must agree |
| `engine` | off | compiles the frozen L1/L2 loader **bodies**; with it off, `layers_frozen` compiles to a stub that returns `ConfigRefusal` before touching a file handle | G5c + the handle audit; this is the *buildable* half of I2 — see §6 |
| `supporter` | on | compiles `si-supporter` (the seam + filter). Grimm the **model** is never a crate; unplugging him is running with nothing arriving on the seam, not a rebuild | G12(a): bit-identical state hash with the seam connected-but-silent vs disconnected |
| `rayon` | off on the walk path | enables parallelism **only** in the genome-build and export/index passes (`si-store` build tools, `si-harness`) | **OD-50**: `si-walk`/`si-learn`/`si-affect`/`si-sleep` do not list `si-core/rayon`; the CPU oracle stays single-threaded and fixed-order (I5). **Sleep is on that list because a sleep is a state write**: P3's sweep over live edges is one of the few places a `rayon`-sized loop looks tempting, and `(x, y)` order is the contract ([10 §8](10-temperament-sleep.md), pass P3) |
| `crystallize` | **on** | compiles `si-crystallize` **and** `si-session`'s step-7a call. Off ⇒ the reader is absent from the object file, not idled | **V10** ([12 §6.1](12-v2-arc.md)): G15(b) / G18(b) are *build-graph* deletions like G12(a)'s, never `if` statements. The frame log still writes (`si-walk` owns it), nothing observes it |
| `silt` | **on** | compiles `si-silt` and the fifth ignition term's row lookup. Off ⇒ `walk::ignite` has no table argument to read | **V11**: G16(b). With the crate gone `IGN_WEIGHT_SILT` stays at its shipped `0` and the sampler is bit-identical to B7's |
| `skills` | **on** | compiles `si-skills` and `si-session`'s step-7b call. Off ⇒ shelf 9 stays reserved-empty | **V10** (same variant, both detectors gone) — kept a separate feature so the two deletion tests stay **independent**, which is why `crystallize.py` and `skills.py` stayed two modules ([docs/12](../docs/12-component-map.md), OD-119) |
| `tongue` | **on** | compiles `si-tongue` **and** `si-session`'s step-9a call at the one `Output::Answer` site. Off ⇒ the phrase walker is absent from the object file and the site is the expression that shipped before B20 | the **build-graph** half of `G20(j)`: a release without a mouth is this feature off, not an `if`. **V14 is not that build** — V14 is the ordinary build with the `TONGUE` ruling row installed `1` ([13 §6.1](13-tongue.md)). The feature is the build-graph switch and the row is the runtime switch, and V14 uses the row, never the feature, because off-by-feature would make `TONGUE = 1` unreachable ([13 §1.4](13-tongue.md)) |

**The pairing rule, so the two mechanisms are not confused:** the *feature* decides
whether a reader or the voice **exists in the binary**; the **ruling row** (`CRYSTALLIZE`,
`SKILLS`, `EPISODIC_TRAVERSAL`, `SKILL_COMPOSITION`, `WORLD_TIERS` — [12 §3.2](12-v2-arc.md);
`TONGUE` — [13 §3.2](13-tongue.md))
decides whether it **runs in the session**. A gate scenario installs a ruling row
through the launch channel; a deletion variant removes the crate from the graph.
Off-by-default plus present-by-default is not a contradiction: the arc ships inert,
the tongue ships inert, and their own gates are the runs that arm them.

`default = ["vulkan", "supporter", "crystallize", "silt", "skills", "tongue"]` (the
last four arrive with B14–B17 and B20). A CONFIG SI release build is
`--no-default-features --features vulkan,supporter,crystallize,silt,skills,tongue`
(engine off ⇒ the genome loader cannot open a file even by accident), and
**V10/V11 are that same command minus the arc features** — which is why the arc's
deletion variants are expressible at all. Dropping `tongue` too is how a build with
**no mouth at all** is made; that is `G20(j)`'s build-graph half, and it is *not* V14,
which is the ordinary command with the `TONGUE` row installed `1`.

## 5. File and container conventions (reused from LoopNet)

| convention | exact form | reuse source |
|------------|-----------|--------------|
| self-describing binary | `MAGIC(4) \| version u32le \| header_len u32le \| header(JSON) \| payload`; the **format is read from magic, never the extension** | `tesseract-core` `ckpt.rs` `TNCK`, `quant.rs` `TNT4` |
| SI magics | `SIlat` (lattice export), `SIgen` (genome file), `SIck` (session checkpoint), `SIst` (state-snapshot for hash/replay), `SIro` (one framed record in a stream) | — new names, same container contract |
| atomic write | `name.tmp` → `sync_all` → `rename`; a resume **refuses** when `RunParams`/`GenomeParams` geometry mismatches the file it is resuming into | `ckpt.rs` save/load; a mismatched resume is a *meaning* change and aborts |
| record alignment | every packed row length asserts `len % 4 == 0`; per-row byte sizes are documented with offsets in [01](01-data-schemas.md) and every number in [06](06-resource-budget.md) is derived from those asserts | `quant.rs` `16 + d + 8g` byte-accounting idiom |
| runs as the reporting surface | `runs/<session>/` for live sessions, `gates/<run>/` for gate output, plus `tree_sha256.txt` and sortable `{tag}_step{07d}.{ext}` names | `tesseract-bench` `runs/*.json` + `runs/matrix.json` aggregate |
| profiling with no dependency | env-gated phase timers, compiled no-op when unset | `TESSERACT_PROFILE` idiom → `SI_PROFILE`; the VRAM/PERF panel reads its accumulators ([08](08-gui-observability.md)) |
| error handling | `anyhow::Result` + `Context` everywhere; **no** custom error taxonomy in the hot path. Typed *outcome* enums exist only where the design requires a branch: arbitration verdict, gate decision, config refusal, speak-gate rejection | `tesseract-*` all use `anyhow` |

## 6. The config guard, stated exactly (reconciles docs/10 and I2)

[docs/10 §10.1](../docs/10-configs-converters.md) says CONFIG SI and CONFIG
ENGINE are "same code, different load". I2/R-10 say CONFIG SI has **no code path
to open L1/L2**. Both are honored, in two layers:

1. **Runtime**: `--config si` (default). The frozen loader entry runs a
   `debug_assert`-plus-release-assert guard that returns `ConfigRefusal` before
   it forms a path string, so no `open`/`mmap` handle is ever created. G5c's
   handle audit proves zero open genome handles across a full CONFIG SI gate run.
2. **Build**: the `engine` feature (§4) gates the loader *bodies*. A CONFIG SI
   binary built with `--no-default-features` has the file-open code **absent from
   the object file** — the strongest available reading of "no code path", at the
   cost of a second feature build, which is why it is offered as a flag and the
   runtime assert is the always-on floor.
3. **Physical**: CONFIG SI ships with no `SIgen` file on disk; the loader has
   nothing to open. Recorded in the manifest as an expected-absent path so its
   presence is itself a G5c failure.

The **config is one field of `RunParams`** and is **not** in the state hash
(wall-clock and environment are excluded per I5); two configs over the same
transcript produce two hashes, and their gap is the designed long-term leg,
**UNMEASURED** until an ENGINE run exists ([docs/10 §10.1](../docs/10-configs-converters.md)).

## 7. Process topology

Three OS processes, one of which is not ours. SI never blocks on either of the
other two.

| process | built from | owns | reads | writes |
|---------|-----------|------|-------|--------|
| **A — the loop** | `si-cli run` (`si-session`) | the lattice, the walk, all state, the state hash | `sense_inbox.siro` (the sense queue) | `runs/<session>/stream.siro` (the output lane), `monologue.siro` (separate lane), `sleep.jsonl` (one line per pass per sleep, [10 §7](10-temperament-sleep.md)), `SIck`/`SIst` snapshots, `mindlog.json` |
| **B — the viewer / panel** | `si-cli view` (`si-vulkan`) | zero SI state (observer-only, §8 of the job → [08](08-gui-observability.md)) | `stream.siro`, `monologue.siro`, `mindlog.json`, `sleep.jsonl`, `fingerprint.txt` (all tail-only) | `sense_inbox.siro` **only** — it appends a keystroke and never touches a state byte |
| **C — Grimm (external)** | *not built from `src/`* | his own process, versioned/tuned separately | the read-only MCP tools over A's artifacts | `sense_inbox.siro` **through `si-supporter`'s seam**, every line arriving `source: grimm` |

- **A never waits on B or C.** The transport is append-only framed files
  (**OD-48**): a writer appends, a reader tails and advances an offset; there is
  no synchronous request/reply that can stall the loop, which is what "the panel
  is a sense organ, not a controller" costs and buys.
- **B is read-mostly by construction**: the only byte it may write is a new
  record on `sense_inbox.siro`, and that record carries `source: owner` because
  it is the owner's own channel; the viewer cannot mint a `source: grimm` line —
  only the seam can, and the seam lives in A (§2, I3/I10).
- **C reaches A only through `si-supporter`.** There is no reverse arrow (I10):
  SI imports nothing from Grimm, never spawns him, never reads his scratch, and a
  session with process C absent is a configuration, not a code path.
- **The v2 arc adds no process.** The N-voice world ([docs/26](../docs/26-world.md))
  is *one* sense queue with identity-tagged lines: G19 is gated tonight by a
  scripted harness fixture (OD-132), not by a network. When a Discord adapter is
  built it joins as **a fourth process writing `sense_inbox.siro` through
  `si-supporter`'s seam** — a transport at the panel's edge, outside the loop, and
  **not built here** ([docs/26 §26.13](../docs/26-world.md)'s scope line). Who mints
  an identity or a `source: grimm` stamp once such a process exists is **OD-136**
  ([12 §10](12-v2-arc.md)).

## 8. Run / gate directory layout

```
runs/<session-id>/                 # a live or replayed SI session
  SIck                             # latest checkpoint (atomic write, §5)
  SIst                             # state snapshot used for the hash
  stream.siro                      # A→B owner-facing output lane (framed records)
  sense_inbox.siro                 # B/C→A sense queue (framed records, source-tagged)
  monologue.siro                   # idle cascade lane (§19.3) — never owner-facing
  sleep.jsonl                      # the SLEEP lane: one line per pass per sleep (10 §8) — log, never state
  crystallize.jsonl                # B14: one line per candidate transition (open/recurrence/close/resolve) — OD-98's audit trail
  skills.jsonl                     # B17: one line per shape-candidate transition — OD-117 rides OD-98
  silt.jsonl                       # B15: one line per tick per active table {t, conversation, [(point, level)]} — OD-103's
  fingerprint.txt                  # the temperament readout (10 §10) — observer-only, never re-read
  exports/                         # JSONC lattice dump (export only; never re-read into a run)
  tree_sha256.txt                  # integrity of the directory

gates/G<n>_<config>_<yyyymmdd-hhmm>/   # one run of one gate (gates/README.md is authoritative)
  transcript.txt                 # the sense stream, each line source-tagged (§21.4)
  mindlog.json                   # every decision, label, arbitration, per-event `a` trajectory
  monologue.jsonl                # idle cascades (G10(d) checks each against its trace)
  state_hash.txt                 # deterministic fingerprint (trajectory, stamps + a_at,
                                 #   per-edge bake, ignition seed, birth seed + the derived
                                 #   vector digest (01 §10 item 9); medians deliberately ABSENT — R-28)
  gate_report.json               # pass/fail + per-criterion evidence
  *_sweep.json                   # G2/G8/G9/G10/G11/G13/G14 sweep artifacts (outputs, never design inputs),
                                 #   plus the arc's G15/recurrence, G16/silt, G17/prov, G18/shape (12 §3.1)
```

The `gates/README.md` contract is **not** duplicated here; this section only fixes
that the harness writes into exactly those filenames, and that any tunable
*output* of a sweep lands in the run directory it belongs to ([05](05-gate-harness.md)).
A run directory whose artifact is later read back as a build *input* is R-33; the
only file `si-store` may open at run time is the pinned `SIgen`, and the only
files `si-vulkan` may open are B's own output-lane tails.

## 9. Dependency allowlist (the whole transitive closure must stay inside it)

| crate | version (pin in `Cargo.lock`, hash in G5c) | used by | carries content? |
|-------|---------------------------------------------|---------|------------------|
| `anyhow` | 1 | all | no — error plumbing |
| `serde` + `serde_json` | 1 (derive) | core, store, front, supporter, session, sleep, crystallize, skills | no — formats the schemas in [01](01-data-schemas.md), the `sleep.jsonl`/`fingerprint.txt` lines in [10 §8/§10](10-temperament-sleep.md) and the arc's three `.jsonl` logs in [12 §1.3](12-v2-arc.md). **`si-silt` needs none of it**: its table is RAM keyed by handles and its line is formatted by `si-session` |
| `bytemuck` | 1.20 (derive) | core, store, vulkan, sleep | no — packed-record casts |
| `clap` | 4.5 (derive) | cli, harness | no — the `--config/--session/--replay/--hash/--export/--gate` surface |
| `md-5` | 0.10 | core, store | no — integrity + state-hash identity (**OD-49**, *identity, not security*) |
| `rayon` | 1 | store build-tools, harness **only** | no — and **OD-50** keeps it off every hashed-path crate |
| `ash` | 0.38 | vulkan | no — the loader |
| `winit` | 0.30 | vulkan (viewer) | no — window + input events |
| `raw-window-handle` | 0.6 | vulkan | no — bridges a winit window to the ash swapchain |

**The law, restated for the build (OD-42):** every row above is compute-only and
carries zero lexical or corpus content. The audit that proves it is the
`Cargo.lock` sha256 recorded in G5c — if a transitive dependency smuggles a
word list, embedding table, or pretrained weight, the lock hash changes and G5c
fails. **Forbidden even as a "helper" (R-14):** any crate exposing a
`Vec<f32>`/embedding similarity API — `si-vulkan`'s panel reads quantized
ints, never a float vector.

**The v2 arc adds no row here.** Its three crates are built from the same
allowlist, so `check_deps`'s `Cargo.lock` hash moves only because the *workspace
graph* grew from 13 to 16 packages (the digest recorded per report,
[12 §6.1](12-v2-arc.md)), not because a new library entered the build.

## 10. What is deferred, and where it is decided

| open | lean in this file | ruling lives in |
|------|-------------------|-----------------|
| OD-42 "zero deps" wording | content-based allowlist + `Cargo.lock` hash (§9) | [ledger](../docs/decisions/open-decisions.md) |
| OD-43 `*.py` names | kept as logical ids, 13-crate mapping table (§2) | [ledger](../docs/decisions/open-decisions.md) |
| OD-44 CPU walk + Vulkan day one | walk in `si-walk` (CPU oracle), Vulkan for residency + viewer only | [ledger](../docs/decisions/open-decisions.md) |
| OD-47 GUI toolkit | `ash` swapchain + `winit` + bespoke immediate panels; **no** `wgpu`/egui/retained mode | [ledger](../docs/decisions/open-decisions.md) |
| OD-48 transport | framed append-only `stream.siro` / `sense_inbox.siro`; loop never blocks | [ledger](../docs/decisions/open-decisions.md) |
| OD-50 concurrency | `rayon` in build/export only; no hashed-path crate may depend on it | [ledger](../docs/decisions/open-decisions.md) |
| OD-84 a 13th crate with no `*.py` id | `si-sleep` gets `consolidate.py` as its logical id in docs/12; folding it into `si-learn` is refused because it deletes structural win 4 | [ledger](../docs/decisions/open-decisions.md) |
| OD-135 the `prov_speaker` value for a granted trusted teacher | the arc's three crates have ids in docs/12 already, so 00 grows three rows and no seat; the *provenance* half is the open one: tonight the class function, the grant row and tier-strict arbitration are coded and the belief write is **refused**, so `prov_speaker` stays `{0,1,2}` and G19(c) grades the refusal ([12 §10](12-v2-arc.md)) | [ledger](../docs/decisions/open-decisions.md) |
| OD-136 who mints `source: grimm` and an identity once an adapter exists | the seam stays the only minter inside A; an adapter is a process of the owner's, unbuilt tonight (§7) | [ledger](../docs/decisions/open-decisions.md) |
