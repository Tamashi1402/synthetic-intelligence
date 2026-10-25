# ENGINEERING — the build spec for SI (SPECIFICATION ONLY, NO CODE)

This folder converts the **frozen logical design** in [`../docs/`](../docs) into a
build specification. It changes no law. It answers only the questions the logical
design leaves open when a compiler is involved: byte layouts, call order, crate
edges, error behavior, units, and the numbers that must come out of a sweep
before anyone quotes them.

**The test this folder is written against:** a competent engineer who has never
spoken to the owner should be able to build v1 without asking one design
question. If a question is unavoidable, it was a hole — it is either closed here
or it is in the ledger with a recommended lean ([`../docs/decisions/open-decisions.md`](../docs/decisions/open-decisions.md)).

---

## Authority order (which file wins)

| layer | owns | wins over |
|-------|------|-----------|
| [`../docs/`](../docs) + [`../gates/README.md`](../gates/README.md) | **WHAT and WHY** — the invariants I1–I10, the organs, the laws, the gates, the risks | everything |
| this folder | **HOW** — layouts, orders, types, units, files, processes | nothing above it |
| [`../docs/decisions/open-decisions.md`](../docs/decisions/open-decisions.md) | **UNDECIDED** — OD-01…OD-186, each with a lean | the build must implement the lean and mark it provisional |

A conflict between `docs/` and `engineering/` is **never** resolved by editing
`docs/` to match the build. It is resolved by an OD entry (merge law). This
folder therefore contains statements of the form *"the design does not decide X;
lean Y; OD-nn"* — 53 of them are raised by this folder (OD-41…OD-86, the two the v2
arc's engineering pass found — **OD-135, OD-136** — the three the tongue round's
engineering pass found — **OD-182, OD-183, OD-184** — and the two the tongue
*build* found, where the code disagreed with the HOW — **OD-185, OD-186**) and they
are listed at the bottom of this file.

## Rules of this folder

1. **No code, no pseudo-code** beyond what a schema, a byte layout, or a
   round-by-round trace requires. Tables and ordered prose are the spec language.
2. **Nothing is quoted before its sweep exists.** Every tunable appears in
   [`03-constants-registry.md`](03-constants-registry.md) with a legal range, a
   placeholder default (`—`), and the artifact that chooses it. A number in this
   folder that is not a byte count, an id, or a schema enum is a mistake.
3. **UNMEASURED stays UNMEASURED.** [`06-resource-budget.md`](06-resource-budget.md)
   gives byte formulas and derived arithmetic, never measured milliseconds.
4. **Reuse LoopNet.** The reference engine at
   `C:\Users\jakub\Documents\Qoder\2026-10-21\6a136381\tesseract` already solved
   the problems SI shares with it (deterministic host RNG carried in checkpoints,
   self-describing containers, atomic writes, persistent-mapped Vulkan with a
   flush counter, a CPU oracle behind `--no-default-features`, env-gated phase
   timers, `runs/` JSON artifacts). Where we reuse an idiom it is cited by path.
   Where SI needs something the engine does not have — the GUI is the big one, it
   has **zero** windowing code — that is stated, not glossed.

## Files

| file | deliverable | owns |
|------|-------------|------|
| [`00-stack-workspace.md`](00-stack-workspace.md) | 1 | Rust workspace, crate graph as the *enforcement* of §12's one-way street, feature flags, dependency allowlist, file/container conventions, process topology, run directory layout |
| [`01-data-schemas.md`](01-data-schemas.md) | 1 | field-level schemas: addresses, shelf registry ids, every row type, every record layout with byte offsets, quantization grids, interning, null rules, serialization |
| [`02-module-contracts.md`](02-module-contracts.md) | 2 | per-crate contracts: ordered inputs/outputs, error behavior, explicit "may NEVER touch" import tables, the grep rules translated into compile-time edges |
| [`03-constants-registry.md`](03-constants-registry.md) | 3 | every UNMEASURED number: name, home crate, unit, quantization, legal range, default, sweep artifact, consuming gate |
| [`04-algorithms.md`](04-algorithms.md) | 4 | the walk, write gate, arbitration, drive update, salience/bake, median, labeler, idle tick, gain probe, hash composition — in words, with update order and normalization order fixed |
| [`05-gate-harness.md`](05-gate-harness.md) | 5 | G1–G20 as runnable procedures: inputs, asserts, artifacts, fail states; plus the engineering-only `check_*` legs (52 ids) and the sweep protocols (`G13/sleep_sweep`, `G13/dimming_sweep`, `G14/seed_panel`, `G20/phrase_sweep`, and the arc's four `status: refused` runners) |
| [`06-resource-budget.md`](06-resource-budget.md) | 6 | memory layout, expected sizes from the schemas, per-thought and per-idle-tick cost model, the per-sleep envelope, the chained sub-digest and its repair cost, sync budget, VRAM residency |
| [`07-genome-build-plan.md`](07-genome-build-plan.md) | 7 | the frozen base: source matrix, edge kinds, weight tiers, merge law, licenses, density gates, build pipeline + time budget, the LLM-expansion interface (ruling PENDING) |
| [`08-gui-observability.md`](08-gui-observability.md) | 8 | the five panels, the observer-only laws as *architecture*, loop↔viewer transport (21 frame kinds), Vulkan rendering the walker's own buffers, the SLEEP readout and the TEMPERAMENT FINGERPRINT |
| [`09-teach-channel.md`](09-teach-channel.md) | 9 | the `/teach` channel: parse, decode, key, commit, the demonstrated-vs-asserted gain ledger, the bulk-curriculum path and its published limit |
| [`10-temperament-sleep.md`](10-temperament-sleep.md) | 10 | the birth seed and what an individual may and may not be; the constant **class** split (LAW / SPECIES(sweep) / TEMPERAMENT(seed)); the four sleep passes, their triggers, their interrupt and their cost; dimming's arithmetic and its floors; G13/G14's subject matter; **OD-71…OD-85** |
| [`11-build-plan.md`](11-build-plan.md) | 11 | **the plan the build obeys**: twenty stages with commit boundaries, per-stage machine-checkable DoD naming gate and `check_*` ids, the gate-to-stage matrix, the three-attempt stop rule, global acceptance, and the eight readings pre-committed so no decision is made while coding; **OD-86** |
| [`12-v2-arc.md`](12-v2-arc.md) | 12 | **the HOW of docs/22–26** (crystallize · silt · episodic · skills · world): the three reader-only crates and the missing edges that make "may nominate, never mint" compile-time fact, the delta to 00–06 (bytes, tick steps 5a/5b/7a/7b, 22 registry rows + the `CONSTS_VERSION 0x0002` re-baseline, the five algorithms, the four refused sweeps), **the break-list** — every pre-existing artifact the arc moves and the stage that moves it — variants V10–V13, and what this round refuses; **OD-135, OD-136** |
| [`13-tongue.md`](13-tongue.md) | 13 | **the HOW of docs/27** (the tongue: surface realization by walk): the seventeenth crate `si-tongue` and the `License` argument that makes OD-181's citation-scoped vocabulary a missing edge instead of a discipline, the delta to 00–06 (the one tick step **9a**, 8 registry rows + the sixth class + the `CONSTS_VERSION 0x0003` re-baseline, §34's phrase walk and its three structural checks, variant **V14**, the six new `check_*` legs 46→52, the one refused `G20/phrase_sweep`), **the break-list**, stage **B20**, and what this round refuses; **OD-182, OD-183, OD-184** |

## LoopNet pattern-reuse table

| SI need | LoopNet idiom (verified in the reference engine) | where it lands |
|---------|--------------------------------------------------|----------------|
| replay byte-identical incl. randomness | `RngState { state, inc, spare }` is *serializable*; `Rng::snapshot()`; PCG32 + Box-Muller in `tesseract-core/src/rng.rs` | ignition seed and guess-slot seed ride the session snapshot — §19.4, I5 |
| content-addressed coordinate hashing | `hash_u32(a,b,c)` (PCG round mix) and `coords_of(key)` = three rotations of the same mix, `tesseract-core/src/{hash,lattice}.rs` | SI's `(t,x,y,z)` addressing derives point ids the same way — deterministic across machines, zero stored index |
| self-describing binary artifacts | `MAGIC \| version u32le \| header_len u32le \| header JSON \| payload`, format read from magic not filename (`ckpt.rs` `TNCK`, `quant.rs` `TNT4`) | every SI container (`SIlat`, `SIgen`, `SIck`, `SIst`, `SIro`) — see 00 §5 |
| crash-safe writes | `name.tmp` → `sync_all` → `rename`; resume **refuses** on `RunParams` geometry mismatch | session snapshots and genome files; a resume that would change meaning aborts |
| exact byte accounting per record | packed row layout documented with offsets, `16 + d + 8g` bytes, and a `% 4 == 0` assert | every SI record layout in 01; every size in 06 is derived from those asserts |
| bit-identical CPU/GPU scalar math | multiply by a reciprocal constant (`INV_GROUP_MAX = 1.0/7.0`) instead of dividing | SI's per-round normalization and bake/median scaling use reciprocal constants on the same grid — OD-45 |
| CPU oracle beside the GPU | `default = ["vulkan"]`, `vulkan = ["dep:ash"]`, `--no-default-features` = pure CPU; parity tested | SI: the hashed walker is the CPU oracle; Vulkan ships day one for residency + the viewer — OD-44 |
| residency without staging copies | persistently mapped `HOST_VISIBLE\|HOST_COHERENT`, one batched command buffer per dependency chain, **flush only when the host reads or overwrites a live buffer**, `flush_count` metric | the one-sync-per-thought budget (§16.5) and the VRAM/PERF panel's sync counter |
| runs as the reporting surface | `runs/*.json` per cell + `runs/matrix.json` sweep aggregate + `tree_sha256.txt`, `{tag}_step{07d}.{ext}` sortable names | `gates/<run>/` and `runs/<session>/` — see 00 §7, 05 |
| profiling without a dependency | env-gated phase timers, no-op when unset (`TESSERACT_PROFILE`) | `SI_PROFILE`, same shape; the VRAM/PERF panel reads its accumulators |
| error handling | `anyhow::Result` + `Context` everywhere, no custom error taxonomy in the hot path | all crates; SI adds typed *outcome* enums only where the design requires a branch (arbitration, gate decisions) |

## What this folder deliberately does not do

- It does not add an organ, a door, a shelf, a signal, a label, or an emotion.
  Organs stay six, doors stay six, signals stay the six of §18.1, labels stay the
  six of §18.2, shelves stay the registry of §5.2 plus the three reserved ones.
  **The TEMPERAMENT/SLEEP batch is held to that sentence literally:** `si-sleep` is
  a thirteenth *crate* and step **12b** of the existing loop, not a seventh organ;
  the temperament is a new **class of constant**, not a new quantity in the walk;
  and sleep adds **no shelf, no row type, and no state byte** — its whole write set
  is in-place FLUID ([10 §8](10-temperament-sleep.md), [06 §2](06-resource-budget.md)).
  **The v2 arc is held to it equally literally:** `si-crystallize`, `si-silt` and
  `si-skills` are three more *crates* — not three more organs — and the arc adds
  **no shelf, no envelope byte, no `prov_kind`, no edge kind, no state-hash item, no
  signal, no label and no vocabulary word**; everything it proposes is a *value* on
  a structure that already exists ([12 §0](12-v2-arc.md)'s eight facts,
  [12 §2.1](12-v2-arc.md)'s compatibility contract).
  **The tongue round is held to it the same way:** `si-tongue` is the seventeenth
  *crate*, not a seventh organ, and it adds **no shelf, no envelope byte, no row
  kind, no `prov_kind`, no edge kind, no state-hash item, no signal, no label and no
  vocabulary word** — everything it proposes is a *choice of surface* over rows the
  two keys already minted ([13 §0](13-tongue.md)'s nine facts,
  [13 §2](13-tongue.md)'s "what does not change" table). Its one schema-wide
  consequence is a **sixth registry class** (`TONGUE`, drawn and never swept) and the
  `CONSTS_VERSION 0x0003` re-baseline that follows from eight new rows.
  Where the owner's chunk wording needed a new thing to exist, the law won and the
  wording went to the ledger.
- It does not soften a law to make a build easier. Where a law costs the build
  something, the cost is written down (the GUI being a separate process, the
  median being recomputed rather than cached, `audit`-only emotion words, the
  walker being CPU-quantized while Vulkan is already loaded, and the full
  edge-class re-digest **P3** pays because an in-place FLUID write breaks a chained
  sub-digest — [06 §7](06-resource-budget.md)).
- It does not assume the LLM-expansion ruling (owner ruling **PENDING**, §7 of
  the job) — [`07-genome-build-plan.md`](07-genome-build-plan.md) is complete
  without it and names the one insertion point where it would attach.

## Open decisions raised by this folder (OD-41 … OD-86, OD-135–136, OD-182–184)

| id | subject | lean in these files |
|----|---------|---------------------|
| OD-41 | `/teach` gets owner-tier **confidence init** but T2 **arbitration rank** | implement as written in docs/08 §8.4 / docs/06 §6.4; the split is the ruling, not a bug |
| OD-42 | "pure stdlib, zero deps" (docs/12) vs the Rust+Vulkan stack law | restate the law as **no dependency may contain pretrained content, a lexicon, or a corpus**; publish a byte-audited allowlist (`Cargo.lock` hash inside G5c) |
| OD-43 | `docs/12` names `*.py` modules | keep the names as **logical module ids**; ship them as Rust crates with a one-to-one mapping table (00 §2) |
| OD-44 | v1 CPU walker (§16.6) vs "Rust + Vulkan, GUI day one" | **both**: the hashed knowing loop walks on CPU fixed-point (the determinism oracle); Vulkan is loaded day one for genome residency and for the viewer rendering the walker's own buffers |
| OD-45 | the `norm` operator in `h_k = decay · norm(h + W_eff·act)` | per-round max-normalize via a reciprocal multiply on the fixed-point grid; one shared shift, never a division |
| OD-46 | quantization grids | activation i32 Q12 · arousal u8 (1/255) · confidence u16 hundredths · bake u16 Q12 · stamp magnitude u16 · genome weight u8 (00 §6, 01 §9) |
| OD-47 | GUI toolkit | `ash` swapchain + `winit` input + bespoke immediate-mode panels; **no** `wgpu`, **no** retained-mode framework, **no** egui |
| OD-48 | loop ↔ viewer transport and the CHAT input path | append-only framed `runs/<session>/stream.siro` (loop writes, viewer tails) + a `sense_inbox.siro` queue (viewer/CLI appends, loop consumes); the loop never blocks on the viewer |
| OD-49 | hash function | `md-5` for both payload integrity and state hash; the t=0 genome tag is the same id, documented as *identity, not security* |
| OD-50 | concurrency | `rayon` only in the genome-build and export/index passes; **no** hashed-path crate may depend on it, enforced by the crate graph |
| OD-51 | id representation and L3 storage | `u32` handles everywhere, display ids (`r1`, `q1`, `t_apple`) derived at render; L3 rows in packed binary + a string interner, JSONC as export only |
| OD-52 | genome source versions + LLM-assisted expansion | source files pinned by URL + sha in the manifest; expansion remains **PENDING** and the plan works without it |
| OD-53 | engineering-only checks (headless-with-GUI-killed, observer-only GUI) | ride the harness as `check_*` legs in 05, **not** as a thirteenth gate |
| OD-54 | multi-line paste at the panel | one logical step per line, order preserved, each line its own sense record (01 §12) |
| OD-55 | plural-fold alias lighting at round 0 | light every alias member with the token's activation, split by count, and log the alias set in the mind log (04 §2) |
| OD-56 | `bake` when `a_formation` is at rest (0) | `bake = 0` and the edge keeps §8.4's plain init — the floor is law, not tuning (03 §4) |
| OD-57 | the weighted-median rank convention (lower vs upper element at the half-weight crossing) | lean **lower**, as the conservative reading of "no word flips on one comment"; version-tagged, never a literal (04 §9) |
| OD-58 | **does a stably re-derived inference become a bridge candidate row in v1, or is promotion parked?** + its sub-question: does a genome edge consume owned-hop budget on a mixed route? | **ruling PENDING, not assumed.** Row shape, `INFERENCE_STABLE_N` and the gate leg are specified now; `INFERENCE_PROMOTION` ships **off** (03 §2.1, 04 §5.4/§21 U.4, 05 G11(e)–(i)/(p)) |
| OD-59 | `prov_speaker` gains `2 system-inferred` | docs/07 §7.3 mechanism 2 already writes system-inferred bridges and the encoded schema could not hold them. The mechanical widening is not in question; the **scope sentence** (belief rows vs `links` rows) is the ruling — unifying them breaks either clean-room provenance or the bridge mechanism (01 §4.1, 04 §21 U.4) |
| OD-60 | the state-hash **construction** (whole-per-tick vs chained per-shelf sub-digests) | lean **chained**, fixed once before the first G5 run and version-tagged as `HASH_FORMAT` so an old `state_hash.txt` can never be replayed under a new construction silently (06 §7) |
| OD-61 | the bound on the **ignition candidate pool** | lean **the rehearsal stream's newest window** (`Clock::recency` > 0), so idle cost follows the one curve instead of growing with the session (R-29); size swept into `G10/ignition_sweep.json` (06 §13) |
| OD-62 | how "relation names **verbatim**" survives a closed `u8` kind enum | a sealed, version-tagged **genome kind table**: 256 slots holding the source's own strings, listed not mapped, over-capacity a hard fail. The alternative reading — nine hand-picked relations — is the relabeling docs/10 forbids (07 §6) |
| OD-63 | who consumes **`RULE` rows**, and may they conduct? | **parked in v1**: sealed, loaded, citable, not walkable. A rule is n-ary and the walk is binary; collapsing it to pairwise edges changes its meaning, and doing it faithfully needs a structure the substrate does not have ⇒ R-12's alarm (07 §7) |
| OD-64 | the **grammar property tables** have no author, format or licence named in the design | source **S3**: hand-maintained TSV in-repo (`data/grammar/`), owner-licensed, pinned by sha256, joining the pipeline through the same five adapter contracts (07 §2) |
| OD-65 | may a genome be **pruned or subset** for the load budget? | **no** — the default is a full seal. A cut is a content decision (R-10) and an attribution loss; if laptop-runnability ever forces one, the **owner** names the batch, never a scoring function in the converter (07 §12) |

| OD-66 | does "renders the same buffers the walker computed" mean a **shared live buffer** between the two processes, or is a **frame-driven** render enough? | **frame-driven.** A shared ring would put a per-round coupling and a second sync point between the owner's screen and the knowing loop; the law's real target is *never a re-simulation* (08 §1, §11) |
| OD-67 | may the owner trigger a **counterfactual probe** from the GUI? | **no in v1.** The clean route is a sense-channel command — a new born-in reflex and a new G5c allowlist line — so it should be decided as that, not shipped as a button (08 §9) |
| OD-68 | what does **G5c's byte-exact string diff** scope to now that the build has a GUI? | the **loop binary**; `si-vulkan`'s chrome strings audited by reachability, plus an assert that the viewer holds no label word, emotion word, temperament adjective or content list (08 §3.2) |
| OD-69 | is the bulk **`/teach` file feeder** in v1 at all? | **yes, as an inbox writer only**, licensed by `G1.u.2`'s byte-identity leg: a feeder that cannot produce a different state than typing is a convenience, and one that can is a loader (09 §8, §11) |
| OD-70 | may the feeder interleave **quiet ticks** between curriculum lines so `a` decays between commits? | **off by default.** Legal (pure logical time), but it changes the tick structure, hence the hash, and quiet ticks with an empty agenda **ignite** — an import with gaps would also grow a monologue about its own rows (09 §7, §11) |
| OD-71 | the **final class assignment** of every constant: LAW vs SPECIES(sweep) vs TEMPERAMENT(seed). The chunk says *"Mike proposes the table, owner rules"* | the proposed table is [03 §1.2](03-constants-registry.md), built from [10 §2](10-temperament-sleep.md)'s two tests; a flip moves one cell and, if it crosses into or out of `DERIV_ORDER`, bumps `seed_format` ⇒ every hash (10 §13) |
| OD-72 | is there ever a **regulated curiosity level** (a set point the loop steers toward)? | **refused as a controller; shipped as a static per-individual ask ceiling.** OD-22's strict reading is what §18.4's own consequence paragraph already chose (10 §13) |
| OD-73 | hysteresis / per-signal rise-decay constants? | **refused** — one scalar, one `λ_a`. The rise half already exists as `GAIN_SIGNAL[k]`, so nothing is lost but the name (10 §13) |
| OD-74 | does "arousal curve **shape"** belong to temperament, against §18.1's "swept before quoted"? | **family = SPECIES (swept), member = TEMPERAMENT (seeded).** A seed may pick a sealed variant, never invent a curve; the variant index sits inside hash item 8 (10 §13, **G14(k)**) |
| OD-75 | rescale semantics: ratio-preserving contraction vs mass-preserving redistribution | **headroom-ratio-preserving contraction; mass is not preserved; brightening is refused** (§9's table, **G13(b)**) |
| OD-76 | is dimming **stored** (P3) or a read-side third term in `W_eff`? | **stored, written only by the sleep pass** — the citation, the panel and the walk must read one byte (**OD-76**, law 25) |
| OD-77 | may `floor_shelf(edge)` be **derived from `edge.bake`** (the pilot light as immunity)? | **refused.** bake keeps its two licensed outlets; the pilot light is the existing ignition preference, not a floor |
| OD-78 | does sleep **consume logical steps**? | **yes: each walk sleep runs is a thought and advances `t` by 1** — one quiet tick per walk, so a sleep of `w` walks leaves the clock at `t₀ + w`. The alternative (a parallel sleep-time axis) is a second clock and an I5 hole |
| OD-79 | interrupt granularity when the owner types mid-sleep | **pass boundary.** A partial sleep is a legal, logged, replayable state; a rescale is never half-applied |
| OD-80 | REM/NREM-style **staging** of the four passes | **parked; v1 = four fixed passes in fixed order**, none reading a previous pass's output. A pass graph is a scheduler in disguise |
| OD-81 | "session close" as a sleep trigger, in a loop that may not read a wall clock | **sleep-at-snapshot:** the freeze path runs the passes *before* writing the checkpoint, so the trigger is a step-anchored event and a resume never repeats a sleep (**G13(h)**) |
| OD-82 | if OD-58 lands yes, `prov_kind` gains `5 consolidated` — one more export literal | **specified inert.** Nothing ships until OD-58 is ruled; the row shape and the literal are documented so the ruling is one edit |
| OD-83 | does the species sweep validate the **whole legal box**, or only the point it picks? | **the whole box** (corners + panel points): a range that validates only its winner lets a legal seed land in an ungoverned corner. It costs `\|panel\|` runs per re-sweep, and that is the price of individuality (**G14(h)**) |
| OD-84 | `docs/12`'s component map gains one logical module (`consolidate.py` ⇄ `si-sleep`), and engineering may not edit `docs/` silently | **lean yes.** The alternative — folding `consolidate` into `si-learn` — deletes the missing edge that makes "sleep creates no row" a compile-time fact |
| OD-85 | how many sense events does **P1 replay**, and how do P1 and P4 share `SLEEP_MAX_WALKS`? The chunk said "replay", the registry said "bounded", and [06 §5.1](06-resource-budget.md)'s pricing is what found they disagreed | **replay the `context` rows new since the last sleep, P1 spending the one budget first and P4 taking the remainder.** A P4 with remainder 0 still sweeps in handle order, still reports, writes nothing; P3's rescale and P4's *selection* are never budget-bound, so no bounded night leaves an edge undimmed (**G13(l).4**) |
| OD-86 | a build must run before any sweep exists, and every swept row's default is `—`; the registry forbids inventing a value | **the placeholder policy:** midpoint of the legal range on the row's own grid, tagged `UNMEASURED_PLACEHOLDER`, printed in every `gate_report.json`, counted in `unresolved_legs`, never the reason a leg passes, replaceable only by a `*_sweep.json`, and refused outright under `--strict-consts` ([11 §1 D5](11-build-plan.md)) |
| **OD-135** | what is `prov_speaker` for a **granted** (Trusted, non-owner, non-grimm) speaker? The three encoded values are owner / grimm / system-inferred, and a granted stranger is none of them | **the belief-row write is refused**: `Refused{SpeakerNotRepresentable}`, zero bytes, and **the refusal is G19(c)'s pass condition** with OD-135 printed as its reason. Trusted stays a *derived*, readable, citable, re-lit-able class ([12 §5](12-v2-arc.md), [12 §6.4](12-v2-arc.md)) |
| **OD-136** | who mints `source: grimm` and a `speaker_identity` once a real adapter exists — and the `/trust` verb is **not in the closed vocabulary** | **not built, said out loud:** tonight the only minter is the `harness/scenarios/g19_server.txt` fixture (OD-132), hash-pinned in the report; an adapter is a fourth process at `si-supporter`'s seam and is in no stage of [11](11-build-plan.md) ([12 §1.4](12-v2-arc.md)) |
| **OD-182** | §27.3 defines `STYLE_RECENCY` over "recently-baked / **recently-charged**" arcs while OD-168 (ruled) bans reading `a`, and `Arc::a_at` is one field away | **OD-168 wins:** recency is `bake` plus arena position; `a_at` is never read, and `check_tongue_reads_no_a` makes it a grep ([13 §10](13-tongue.md), [13 §34.3](13-tongue.md)) |
| **OD-183** | §27.4 seeds the per-render PRNG from the state hash too, and hash item 3 is the stamp arena — the draft's own operand re-admits arousal through the seed | **the seed is `(birth_seed, style digest, thought_id)`**: three snapshot values, the arousal-sensitive hash excluded ([13 §34.2](13-tongue.md)) |
| **OD-184** | §27.7 gives `si-tongue` `si-store` as an import, and a `&Lattice` enumerates every point — every word OD-181 forbids it to utter | **two imports** (`si-core`, `si-walk`); surfaces arrive as `License` data, so the bound is structural ([13 §0 fact 6](13-tongue.md), [13 §4.2](13-tongue.md)) |
| **OD-185** | 13 §3.4/§6.3 and 11 §B20 DoD 7 say `gates/deps_lock.md5` does not move, while the same stage adds a workspace member — and the file hashes `Cargo.lock`, where a member is a `[[package]]` row | **the lock is re-published with the crate.** No external package joins, so OD-42's allowlist is untouched; the spec line was wrong about the mechanism ([13 §10](13-tongue.md)) |
| **OD-186** | §27.3's "claim-side" is spelled `edge_kind_is_owned(kind)` in the HOW, but that predicate's last slot *is* `6 cooccur-weak` — it counts record columns, not claims | **`si_core::addr::edge_kind_is_claim(kind)`** (kinds 0..=5), the rule `si-front::label::is_claim` already writes; the law's word decides and 03/04/13's tables are corrected ([13 §34.3](13-tongue.md), [04 §34 AF.3](04-algorithms.md)) |

*Every row above is provisional: the build implements the lean, the ledger holds
the ruling, and a flip changes exactly one file in this folder. The exception is
**OD-71**, which changes the registry column and `DERIV_ORDER`, and therefore every
`seed_format` tag and every hash.*
