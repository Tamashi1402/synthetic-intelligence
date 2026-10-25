# 08 — The GUI and Observability: five panels, and the observer-only laws made structural

Deliverable 9. The panel list and its three laws are the owner's, quoted in the
job: **CHAT** (the console, the only sense channel), **EMOTIONS**, **TESSERACT
PREVIEW** (3D, movable, inspectable), **ORGANS STATUS**, **VRAM/PERF**; and the
laws — the GUI is **observer-only** (no emotion slider, no arousal setter, no
direct row edit; "a settable emotion from a panel is a fakeable machine"), the
knowing loop must run **headless with the GUI killed** ("if the loop needs the
GUI, the design failed"), and Vulkan renders **the same buffers the walker
computed** ("never a re-simulation").

This file turns those three sentences into a build: what each panel reads, what it
may never send, the transport that makes a killed window harmless, and the places
where the panel list as written **predates the design's own later rounds** and has
to be re-specified rather than implemented (§4). It adds no organ, no door, no
signal, no label, no shelf and no state byte.

**Number rules:** the same as [06](06-resource-budget.md) — widths, arithmetic on
widths, and design targets marked **UNMEASURED**. Nothing in this file is a
measured frame rate, and no latency figure is promised, because the GUI is the one
place a spec gets away with promising milliseconds.

---

## 1. The three laws, as architecture

A law that lives in a review note is not a law. Each of the three is redrawn here
as something the build physically lacks.

| law | restated as a build fact | enforced by |
|-----|--------------------------|-------------|
| **OBSERVER-ONLY** | process B owns **zero** SI state and has exactly one writable file: `sense_inbox.siro`, and every frame it appends there carries `source: owner` because it is the owner's own keystroke ([00 §7](00-stack-workspace.md)). There is no second input path, no socket, no config write, no temp file the loop reads | the missing crate edges `si-vulkan ↛ si-learn / si-read / si-walk` ([02 §10](02-module-contracts.md)); `check_gui_observer_only`; the fact that the viewer has no handle to a lattice |
| **HEADLESS** | nothing in the loop's step order ([02 §11](02-module-contracts.md) steps 1–13, **including step 12b's four sleep passes**) waits on, opens, or checks a window. A session with process B never started is a configuration, not a degraded mode | `check_headless_run` ([05 §20](05-gate-harness.md)) — a full gate run with the viewer process absent, and a run with it `kill`-ed mid-session, both producing a byte-identical `state_hash.txt` against the untouched control (**G5(a)**'s discipline, reused). Sleep is inside that row's scope, so a *partial* night is reproducible with no window in sight |
| **REAL BUFFERS, NO RE-SIMULATION** | the viewer performs **no** arithmetic over the lattice. Every number it draws was produced by the loop and published on a lane; camera and glyph layout are presentation and are never sent back | §2's one-way contract + §5's layout rule (positions are a pure function of identity, so there is nothing to simulate) + `check_no_viewer_state_write` |

**The consequence engineers will dislike, stated plainly:** these laws make the
GUI a *reader of the same artifacts the gates read*. That is the design's choice,
and it buys the property that the owner can look at the product's own eyes to see
what a gate measured (§7). It costs the ability to do anything interesting on
demand — no live probe, no click-to-edit, no slider. §9 lists exactly what is
forbidden and why each one would break a named invariant.

---

## 2. The transport contract (what a panel actually reads)

Two append-only framed lanes ([01 §9](01-data-schemas.md), **OD-48**), plus the
files the loop already writes for the gates and for its own record. The panel reads
all of them; it writes exactly one.

| stream | writer | reader | role |
|--------|--------|--------|------|
| `runs/<s>/sense_inbox.siro` | B (or C through `si-supporter`) | A | the **only** inbound bytes; each frame `source`-tagged at birth by the module that received it (L-A, [04 §23](04-algorithms.md)) |
| `runs/<s>/stream.siro` | A | B | the owner-facing output lane: everything CHAT, EMOTIONS, PREVIEW and ORGANS render |
| `runs/<s>/monologue.siro` | A | B, in its **own window/tab** | the idle lane (§19.3) — never merged into the owner-facing lane; whether it is exposed at all is **OD-38** (pending), and §6 renders it *blank-but-present* until then |
| `runs/<s>/mindlog.json` | A | B, tail-only | the per-thought record: decisions, labels, arbitrations, probes, the `a` trajectory, the medians a thought conducted with |
| `runs/<s>/sleep.jsonl` | A | B, tail-only | the per-pass record of step 12b ([10 §8](10-temperament-sleep.md)): one line per pass per sleep, plus the interrupt line when a night was cut short. A run directory without it is a run that never slept, which is a legitimate state and not an error (§6 O11) |
| `runs/<s>/fingerprint.txt` | A, once at wake | B, tail-only | the derived-vector table ([10 §10](10-temperament-sleep.md)), for a person without a GPU. Written from RAM that is re-derived and never stored, so its existence costs no state byte — and it is **never re-read by the loop** |
| `runs/<s>/SIst` | A (atomic, on flush) | B, at open and on demand | the state snapshot a panel uses to paint the *whole* lattice once, then it lives on frames. Its core block carries `birth_seed`, `seed_format`, `temperament_digest` and the two sleep counters ([01 §13](01-data-schemas.md)) |

**One-way, and one-way means no request/reply.** The viewer may not ask the loop
for anything. If a panel needs a fact the lanes do not carry, that is either a
frame to add to §3's enum (a presentation change) or a **design question**, and the
list of the second kind is §9's table, not a feature request.

### 2.1 The frame-kind enum for the output lane

`SIro`'s `kind u8` was left unnamed by [01 §9](01-data-schemas.md); it is named
here because five panels consume it. **Every kind below is a record the loop
already writes** — `mindlog.json` for kinds 0–16, the seam's gate-log line for 17,
`sleep.jsonl` for 19, `fingerprint.txt` for 20. The panel adds no new fact to the
machine, it only forwards it, which is what keeps "the GUI reads the audit streams"
true rather than aspirational.

| `kind` | frame | payload (beyond `seq u64`, `t i32`, `source u8`) | produced at ([02 §11](02-module-contracts.md)) | panels |
|--------|-------|--------------------------------------------------|-----------------------------------------------|--------|
| 0 | `sense_echo` | the admitted line, verbatim + its normalized signature | 1–2 | CHAT |
| 1 | `decoded` | `Pattern \| Undecoded` + frame id + slot set | 2 | CHAT, ORGANS, PREVIEW |
| 2 | `lit_round0` | `{point, quantum}` pairs | 5 | PREVIEW |
| 3 | `round` | `{k, lit_set: [(point, h_q12)], new_edges: [(from, to, weight_eff_q12)], a_used_u8, modulation triple}` | 6–7 per round | PREVIEW, EMOTIONS |
| 4 | `settle` | `{stop_reason, settle_set: [(point, h_q12)], route_counts{owned, base}}` | 7 | PREVIEW, ORGANS |
| 5 | `signal_event` | `{signal 0..5, magnitude_q12, citation}` — **anonymous numbers**, no names | 6 | EMOTIONS |
| 6 | `a_update` | `{a_u8, applied_at_round}` (the lag is visible in the payload, as the law requires) | 6 | EMOTIONS |
| 7 | `median_used` | `{point, median_q12, n_stamps}` for the points a thought actually conducted with | 6/§9 of [04](04-algorithms.md) | EMOTIONS, PREVIEW |
| 8 | `gap` | named dark spot: `{slot, frame, kind G1..G4}` | 8 | ORGANS, CHAT |
| 9 | `door_call` | `{door, thought_id, arg_digest}` — six doors, and the walk makes exactly **two** calls per thought | 6, 8 | ORGANS (§6.3) |
| 10 | `ask` | the emitted question + its budget/queue accounting | 8 | CHAT, ORGANS |
| 11 | `label` | the label **string as received**, plus citation list | 9 | CHAT, EMOTIONS |
| 12 | `output` | the reverse-decoded surface (the owner-facing answer) | 10 | CHAT |
| 13 | `commit` | `{address (t,x,y), edge count, stamp count, gain_kind, arbitration verdict}` | 11 | CHAT, ORGANS, PREVIEW |
| 14 | `supersede` | `{loser address, winner address}` | 11 | ORGANS |
| 15 | `stage` | `{stage, threshold evidence}` | 13 | ORGANS |
| 16 | `tick_end` | `{t, hash_hex, eventlog_len}` | 13 | all, as the frame barrier |
| 17 | `seam_reject` | `{n_untaught_tokens, novel_claims, ref_row_id}` — the machine-readable *why*, forwarded from the gate log (§21.3) | seam | CHAT (a visible line), ORGANS |
| 18 | `probe_result` | **reserved, not emitted in v1** — §9's forbidden-button list, and **OD-67** if the owner wants it | harness only | — |
| 19 | `sleep_pass` | the `sleep.jsonl` line **verbatim, field set per pass** ([10 §8](10-temperament-sleep.md)): P1 `{pass, walks, events_available}`, P3 `{pass, edges_scanned, edges_changed, bytes_changed, max_relative_shift, within_shelf_rank_inversions, cross_shelf_rank_inversions, floor_hits}`, P4 adds the **calm view** counts, and a cut-short night emits the interrupt line `{t, passes_completed, aborted_reason}` instead of the missing passes | 12b | ORGANS (§6 O11) |
| 20 | `temperament_readout` | `{birth_seed hex, seed_format, deriv_order_len, rows: [(registry name, grid value, species box lo..hi)], temperament_digest hex}` — the same table `fingerprint.txt` holds, forwarded so a panel and a text file can never disagree | wake, before the first round of the first thought ([04 §25](04-algorithms.md) W.0) | ORGANS (§6 O12), STATUS |

**Kinds 19 and 20 are the two that a builder will be tempted to make
actionable.** They are not. `sleep_pass` is emitted *after* a pass's writes
complete, so the viewer is watching a record rather than steering one, and
`temperament_readout` is emitted from the vector that was just re-derived from
`birth_seed` — a value the loop computes anyway for **G14(c)**'s byte-compare.
Neither creates a row ([10 §8](10-temperament-sleep.md) law 2), neither is read
back, and §9 refuses the two buttons they would justify.

**And no flag on the old kinds.** Kinds 3, 4, 6 and 13 emitted *from inside a sleep
walk* are byte-for-byte ordinary: there is no "this was a dream" bit, because a
sleep walk **is** a thought (**OD-78**) and inventing a marker would be a second
taxonomy of the same event — the kind of thing that later gets read as a privilege.
What separates them is time, and the panel does not have to guess: kind 19's block
names the pass and its walk count, so the `t` range of a night is derivable by
correlating two artifacts the loop already wrote. A viewer that inferred "SI was
asleep" from a lull would be simulating the loop, which §1's third law is about.

`kind` is presentation-only and lives **nowhere in the state hash** ([01 §10](01-data-schemas.md)
is unchanged): killing the viewer cannot change a byte because a byte is not spent
on the viewer's existence, and the frame contents are derived from state that was
already committed. Kinds 19–20 obey this in the direction that matters: the counters
they display (`sleeps_completed`, `last_slept_at_t`, `temperament_digest`) are
hashed because [01 §13](01-data-schemas.md) puts them in item 9, not because a panel
wants them — while the **18 vector values kind 20 renders are stored nowhere**: item
9 carries their *digest*, and [01 §10](01-data-schemas.md)'s absence row plus
**G14(c)**'s re-derive-and-byte-compare are what keep the frame honest. A viewer
that wanted those numbers to be state would have to add a column, and that is §9's
territory, not §2's.

### 2.2 Back-pressure, lag, and a dead window

| situation | behavior | why |
|-----------|----------|-----|
| the viewer is slow | it tails less often; frames accumulate on disk; the viewer **drops display frames, never data**, and prints a lag counter ("`n` frames behind") | the loop never blocks on a reader (**OD-48**), and "the panel is a sense organ, not a controller" means a stuttering window must not slow a thought |
| the viewer is killed | no effect; the lanes keep appending; a restart reads `SIst` then the frames after it | the HEADLESS law, tested as §1's check |
| the viewer starts mid-session | it renders from the last `SIst` + subsequent frames, and marks everything before "reconstructed from snapshot" | so a reviewer never mistakes a late-opened panel for live observation |
| the disk fills | the **loop's own writes win**: the lanes are the loop's output, so this is a run-failure, not a viewer problem; the checkpoint's atomic-write rule ([00 §5](00-stack-workspace.md)) applies | a GUI cannot be allowed to corrupt a session by being greedy |
| replay (`--replay <run>`) | **the same five panels**, fed from the run directory instead of live files | this is the property worth having: a gate artifact is inspectable with the product's own eyes, so there is no "debug renderer" that lies in a different direction from the real one |

---

## 3. Panel 1 — CHAT, the only sense channel

| aspect | spec |
|--------|------|
| reads | kinds 0, 1, 8, 10, 11, 12, 13, 17 |
| writes | one `sense_inbox.siro` frame per submitted line, `source: owner` |
| shows | the owner's lines and SI's answers interleaved; the **label suffix on every answer** ([docs/09](../docs/09-claim-labels.md)) rendered from frame 11's payload, never from a literal (§3.2's allowlist is the loop's; the viewer holds **no** label words); the citation line under an answer (`INFERENCE/OWNED — from r4@s17 + b2@s20`), verbatim from the payload; ask-backs as first-class lines, with a nonce answer visibly marked as a nonce (§7.4's guess slot — the UI must not make a guess look like a fact); supporter lines visibly marked `grimm`, and a `seam_reject` line visible as *a rejected emission*, because a silently dropped voice would hide the whole of §21 |
| input | single line + multi-line paste: **one logical step per line, in order**, each its own sense record ([01 §12](01-data-schemas.md), **OD-54**); a line starting `/teach` or `/correct` inside a paste is still that line's own act |
| commands | `/correct …` and `/teach <pattern> => <text>` are **typed here and parsed by the born-in pattern table** ([docs/03 §3.2](../docs/03-birth-state.md)). They are the only two strings that change a belief, and they arrive through the same channel as a question. There is no command palette, no autocomplete of content words (§3.3), no macro button |
| scrollback | the lane itself; the panel keeps no private history file, so a replay shows exactly what was seen |
| never | edit, delete, retract or "forget" a row (I4: nothing is dropped, only aged); set `a` or any signal (§4); mark a line "as owner taught" for Grimm (only the seam mints `grimm`, [00 §7](00-stack-workspace.md)); send anything while the loop is not running (the frame is queued, which is legal, and the panel says so) |

### 3.1 Rendering, given no GUI toolkit

**OD-47** chose `ash` + `winit` + a bespoke immediate-mode layer over `egui`/`wgpu`.
CHAT is therefore a glyph-atlas text surface: one baked atlas (a PNG/DX-like
coverage map, laid out as cells by codepoint), a quad batch per frame, and a
monospace advance. Consequences a build must accept:

| item | decision |
|------|----------|
| atlas contents | **glyphs, not words.** A bitmap font with a baked *word* list would be a lexicon in the assets — the §3.2 allowlist's spirit, applied to a `.png` |
| line breaking | by whitespace and character width only. **No** hyphenation dictionary, no word-boundary data table, no spell checker: each is a corpus, and a corpus inside `src/` is R-03/R-10's shape (and, for the *loop*, a language pack is by definition CONFIG ENGINE, R-11) |
| script scope | **printable ASCII in v1**, published as a limit rather than a bug. Unicode text shaping would drag in `harfbuzz`-class data tables — a dependency carrying content, which [00 §9](00-stack-workspace.md)'s allowlist law forbids, and the audit would have to hash it (**OD-42**). Owner text outside ASCII renders as substitute glyphs and is stored **verbatim** in the sense record, so nothing is lost, only unprettified |
| emoji/markdown | not interpreted. The renderer copies; it does not render markup, because "renders markdown" is a template engine asking to decide what the owner meant |

### 3.2 The panel's own strings, and G5c

The loop's byte-exact string audit
([docs/03 §3.4](../docs/03-birth-state.md) test 1) would drown in "ORGANS
STATUS" the moment a GUI shares the binary. **The design does not say which
artifact the diff runs against. Lean (**OD-68**): the diff scopes to the **loop
binary** (`si-cli run`), and `si-vulkan`'s UI chrome strings are audited by a
different assertion — *they cannot reach the loop*, because the crate graph gives
no edge and the file contract gives one write. A third check makes that concrete:
**the viewer contains no label word, no emotion word, no temperament adjective and
no content word list** (§4's list, kinds 11's payload-only rule, and [10 §10](10-temperament-sleep.md)'s
refusal 1 — "neophilic"/"cautious"/"restless" are *classifications of the derived
vector*, so they belong to `si-audit` and arrive only as a forwarded string), so the
only English in it is chrome. `check_emotion_words` scopes the adjectives the same
way it scopes the six names.

---

## 4. Panel 2 — EMOTIONS, re-specified

The job's line for this panel says *"live e: resident levels + decay curves."*
That was written before the design cancelled the residents and the coupling matrix
(**OD-21**, [docs/18 §18](../docs/18-emotions-engine.md)'s "Nothing survives of:
per-resident baselines, per-resident λ, CAUTION's hysteresis"), superseded the
stamp's `resident` field (**OD-23**), and demoted the six names to an audit readout
([docs/18 §18.2](../docs/18-emotions-engine.md)). **A panel of six emotion levels
would be round 1's dial panel rendered** — R-21(a) with a nice skin — so the panel
is re-specified to show what actually exists. This is the one place the build must
not implement the job text literally, and it is called out instead of smoothed over.

| region | shows | source | law it serves |
|--------|-------|--------|---------------|
| E1 **the one scalar** | `a` now, plus the full **per-event trajectory** of the current thought as a sparkline, each point labelled with the round it was *applied* in (not the round it was produced in) | kinds 5, 6 | §20.1's live integration and **OPEN-29**'s one-round lag; the trajectory is the hashed artifact ([01 §8](01-data-schemas.md)), so the panel draws a real stored sequence, not a guess |
| E2 **the six anonymous signals** | the event stream: `signal 0..5`, magnitude, citation — **numbered, never named** | kind 5 | §18.1's table is six signals; naming them here would put emotion vocabulary in a crate that is not `si-audit` (R-21's grep) |
| E3 **what `a` is doing** | the modulation triple actually in force per round: `(reach, decay, τ_settle-shift)`, next to the round's `a` | kind 3 | this is the honest affect display — it shows the *only* way arousal touches behavior (I8's control path `signals → a → walk modulation`), so the panel cannot imply a feeling caused a word |
| E4 **the label stream** | the audit labels as they arrive, with the pattern that classified them | kind 11 (the string is a payload, not a literal) | §18.6: names live in `si-audit` only; the panel forwards |
| E5 **the median, on the record** | for the last thought: the per-point medians it conducted with, and `n_stamps` | kind 7 | [gates/README.md](../gates/README.md) logs "the per-point medians the thought conducted with" — **log only**. The panel must not compute one and must not display a value that is stored: a stored mood is a fixed emotional address (**R-28**), and there is no field to read |
| E6 **the bake** | per selected edge: `bake`, `a_at`, `thought_id`, and the note that it is write-once | `SIst` + kind 13 | §20.4a: durable, bounded, and *not* a read-time multiplier — the panel says which is which so a reviewer is not misled by a big number |

**Prohibited widgets, listed as the failure modes they are:**

| forbidden | it would be |
|-----------|-------------|
| any slider, setter, spinner or "level" box touching `a`, `λ_a`, a gain, the curve, or a threshold | the dial panel; I8's "nothing outside the signal table may move `a`", and R-21's costume test |
| six emotion gauges, a mood ring, a valence/arousal cross | the cancelled matrix (**OD-21**) resurrected as pixels; there is no such state to read |
| a "curiosity meter" or "caution bar" | OD-22: the budget is a **queue rule + ceiling**, not a level; a bar would imply a level |
| an editable stamp, bake, confidence or `repeats` field | I4 + §5.3 law (c): provenance is write-once; an editable provenance is a laundered belief |
| a decay-curve editor | R-29's one clock, owned by `si-core::clock` |
| a "force emotion" or "reassure" button (a click that moves `a`) | R-21(d)/R-34's `soothe()` in a new costume — and worse here, because it would be *in* the loop's own process |

---

## 5. Panel 3 — TESSERACT PREVIEW (3D, movable, inspectable)

### 5.1 The layout question, answered before an engineer answers it wrongly

A 3D graph needs positions. Positions may come from exactly two places:

| option | verdict |
|--------|---------|
| a learned or embedding-derived layout | **forbidden** — R-14 with a camera attached; an embedding that *positions* nodes is an embedding that *scores* edges by another name, and it would put borrowed geometry over someone else's corpus into the one component the owner watches |
| a force-directed relaxation over the live web | rejected for v1 on two grounds: it is a second computation the loop does not do (the **no re-simulation** law, and the viewer would then be *deriving* structure), and over a 34M-edge genome it is a per-frame cost the design never budgeted |
| **a deterministic layout that is a pure function of identity** | **the build.** `pos(point_handle)` from the same hash-mix idiom [01 §1](01-data-schemas.md) uses for `hyperpoint_id` (LoopNet's `coords_of`), plus a shelf-derived shell so `x` is legible, plus the walk's own adjacency drawn as lines. Stable across runs, zero stored bytes, nothing learned, and a replay puts every point exactly where it was |

The consequence is stated rather than hidden: **the picture is structural, not
semantic.** Nearby points are nearby because the registry and the hash say so, not
because the meaning is close — which is the honest rendering of what SI's substrate
actually is ([docs/05 §5.1](../docs/05-substrate-tesseract.md)). Any "nicer" layout
that clustered by corpus statistics would be a content judgment smuggled into a
camera.

### 5.2 What it renders

| element | encoding | budget |
|---------|----------|--------|
| lit points | radius and intensity from `h_q12` through a **display LUT**; the LUT is presentation and may never be inverted back into state | the lit set is ≤ `FRONTIER_CAP` points ([03 §2](03-constants-registry.md)), so this is small by construction |
| edges | line per traversed edge; **OWNED vs BASE by color and by record provenance**, read from the two-cursor merge ([07 §9](07-genome-build-plan.md)) and the `frozen-genome` flag bit, never from a name. And **the display LUT's floor is not the schema's floor**: a mapping that faded an edge toward invisible as its `weight` dimmed would make "dark = unroutable but still citable" ([10 §6](10-temperament-sleep.md)) invisible on precisely the screen the owner watches, so a live-but-dim edge draws at a minimum visible intensity and the *number* lives in the inspector row below. Presentation only — the LUT may not be inverted, and it changes no byte | per-thought edge touches are `FRONTIER_CAP · ρ · K_MAX_ROUNDS` by [06 §4](06-resource-budget.md)'s arithmetic; only traversed ones are drawn |
| spreading in real time | one draw per **kind 3 frame**, i.e. per round, at the loop's own cadence; nothing is interpolated into a value the loop did not compute | ≤ 8 B per lit point per round ⇒ ≤ `8 · FRONTIER_CAP` B/frame — derived, and **UNMEASURED** against any device |
| dark spots | the named empty slots (kind 8) drawn as visibly empty markers, because a gap is a *thing SI knows it does not know* ([docs/07 §7.1](../docs/07-read-path.md)) and the preview is where that is legible | bounded by the gap list of one thought |
| selection | click a point ⇒ its rows (any shelf whose key/val is that point), each row's provenance `{kind, speaker, step, text_ref}`, its `z` edges with `kind`/`weight`/`bake`/flags, its stamp list, and its `derived_from` chain — all from `SIst` + frames. **Plus the three facts that make a dimmed edge explain itself:** `last_touched_at`
(a stored 4 B of the rehearsal pair), the shelf's `floor_shelf` **read from the
registry rather than the row** — [06 §2](06-resource-budget.md) counts a floor as a
constant, which is exactly why a dimmed edge costs no extra byte — and whether
`weight == 0` *means* superseded (**G13(d)**'s biconditional is a UI claim as much
as an assert): "unroutable, citable, and when it went quiet" must be answerable from
the record, because nothing else on screen says so | a read of records that already exist; no query engine, no index beyond §1's arenas ([06 §1](06-resource-budget.md): no secondary index) |
| whole-lattice mode | the point cloud of `SIst`'s live points (+ genome points in ENGINE), **decimated for display only**, with the decimation count printed | v1 shows the live lattice comfortably; the genome's ~10⁵ points are a report, not a particle system. Paging by `x`-region is v2+ ([docs/16 §16.5](../docs/16-think-walk-hardware.md)) |

### 5.3 Vulkan specifics, and the one thing the engine cannot lend

The reference engine has **zero** windowing code ([engineering/README.md](README.md)
rule 4), so this is net-new; what it *does* lend is the residency idiom.

| item | spec |
|------|------|
| surface | `winit` window + `ash` swapchain (**OD-47**), one surface, several panels as immediate-mode regions, not several swapchains |
| genome buffers | the persistently mapped `HOST_VISIBLE\|HOST_COHERENT` regions from [07 §9](07-genome-build-plan.md)'s W.5 are **drawn in place** — this is the literal form of "renders the same buffers the walker computed", and it is why the CSR/record layout is chosen for mapping, not for copying |
| live lattice | read from the `SIst` mapping + frames; the viewer never holds a mutable copy of a row |
| sync discipline | the viewer's own submissions are its business; it may not add a host sync to the loop's path ([06 §6](06-resource-budget.md): v1 is structurally zero-sync, and a GUI-induced sync would break it silently — the PERF panel's sync counter (§7) is how that would become visible) |
| no compute shaders in the viewer | the walk is the CPU oracle (**OD-44**); a second walker in a shader would be a re-simulation and a determinism fork |
| quantized grid only | intensities are computed from `i32`/`u16` grid values, never from float vectors ([00 §9](00-stack-workspace.md)'s forbidden-API row reaches the renderer too) |

---

## 6. Panel 4 — ORGANS STATUS

Six organs plus DRIVE, as counters over frames — **counts, never levels** (§4's
rule). O1–O10 are that list; **O11 and O12 are the TEMPERAMENT/SLEEP batch's two
readouts**, and they obey the same rule for the same reason: a number on this panel
is a *label over an artifact*, and the moment one of them looks like a dial an owner
can turn, the observer-only law is a comment rather than a build fact.

| region | shows | source |
|--------|-------|--------|
| O1 **SENSE** | admitted lines by `source` (owner / grimm), undecoded lines, seam rejections with their reason fields | kinds 0, 1, 17 |
| O2 **KNOW** | row counts per shelf (the 8 real + 3 reserved, all listed — a reserved shelf that renders as "reserved, v2" keeps [docs/14](../docs/14-risk-register.md) R-12's alarm from being silently patched around); supersessions; the last commit's address | `SIst`, kinds 13, 14 |
| O3 **THINK** | thoughts this session, rounds used, stop reason per thought, frontier size, settle-set size | kinds 3, 4 |
| O4 **DRIVE** | the **six doors' call counts** per thought, and the invariant printed next to them: *the walk makes exactly two door calls* ([docs/02](../docs/02-organs.md), [04 §17](04-algorithms.md)). A third door call per thought shows up on screen as a number that contradicts its own label, which is the cheapest possible tripwire for an organ that grew a new socket | kind 9 |
| O5 **the queue and the agenda** | open-question rows `{kind, target, asked_at, status, origin: sense \| idle, arousal_at_ask}` — the last field rendered as *audit evidence*, with the note that nothing gates on it (§17/OD-22) | `SIst` + kinds 8, 10 |
| O6 **arbitration** | every conflict: tier of each side, winner, the loser's `superseded_at/by`, and **both citations** when BASE and OWNED disagreed (docs/05 §5.3's "reported, never hidden" is a UI requirement as much as a labeler one) | kinds 13, 14 |
| O7 **CALIBRATE** | the label stream and the demote-only note (labels come after the fact; `si-session ↛ si-audit`) | kind 11 |
| O8 **stage** | S0/S1/S2 and the exact threshold evidence that moved it (row counts only — no affect quantity and no Grimm flag may enter a threshold, [docs/11 §11.2](../docs/11-stages.md)) | kind 15 |
| O9 **idle** | quiet-tick count, ignition count, `IDLE_PER_TICK = 1` and the per-hour budget as *used / ceiling* counts | kinds 16 + `SIst` |
| O10 **monologue** | a pane that exists, reads `monologue.siro`, and is **blank until OD-38 rules** whether `read_monologue` / owner exposure is in v1 | the lane |
| O11 **SLEEP** | one block per sleep: the four pass records (walks run, and for P1 the pair `walks` / `events_available`, so a truncated night is *visible* rather than inferred — **OD-85**, **G13(l).4**); the P3 line (`edges_scanned`, `edges_changed`, `bytes_changed`, `max_relative_shift`, the two rank-inversion counts, `floor_hits`); P4's **calm view** counts, printed with the standing note that they are **terminal** ([04 §26](04-algorithms.md) law 26 and [10 §8](10-temperament-sleep.md) law 1: they feed no weight, floor, `d`, or sampler); the interrupt line with its `aborted_reason` when a night was cut short; and the session counters `sleeps_completed` / `sleeps_interrupted` / `last_slept_at_t` beside the cadence as a *count pair* ("quiet ticks since last sleep / `SLEEP_EVERY_QUIET_TICKS`") — **a count, not a countdown**, because a countdown implies a button. A session that has never slept renders `sleeps_completed: 0, last_slept_at_t: 0` and that is a legitimate state, not a blank pane (**G13(f).2**) | kinds 16, 19 + `SIst`'s core block |
| O12 **TEMPERAMENT FINGERPRINT** | `birth_seed` as hex with `seed_format` beside it; one line per TEMPERAMENT registry row — **name, grid value, and its SPECIES box `[lo, hi]`**, so the owner sees *where in the box* this individual landed; and `temperament_digest`. Two refusals are printed on the pane itself, because they are the pane's whole honesty: **no nickname** (a classification of the vector is a readout, so it may exist only in `si-audit` and arrive as a forwarded string — [10 §10](10-temperament-sleep.md) refusal 1, `check_emotion_words`, §3.2), and **no explanation** (the fingerprint never accounts for a behavior; the mindlog does — **G14(g)**). It is *not* a diagnosis of the session and must not be captioned as one | kind 20, `fingerprint.txt` |

Nothing in this panel may be clickable in a way that changes state, and the panel
cannot be the place a drive is "tuned up": the doors are a socket, not a behavior
(R15). **O11 extends that sentence to the new two: the panel may not start a sleep
and may not re-roll a seed** (§9's last two rows), and O12's numbers arrive on the
lane as text, never as inputs.

---

## 7. Panel 5 — VRAM/PERF, the instrument that fills G6's blanks

[06 §10](06-resource-budget.md) lists the fields a CONFIG ENGINE run must measure
and every value is UNMEASURED. **This panel is where those fields are produced**,
and that is the file's most useful property: the numbers in `G6/gate_report.json`
and the numbers on the owner's screen come from one set of accumulators, so there
is no benchmark harness that quietly measures a different thing than the product
does (the owner's honest-benchmark rule, applied to the GUI).

| field | what it prints | arithmetic partner |
|-------|----------------|--------------------|
| resident bytes, split | genome (CSR edges / point table / interners / kind table / batch table) vs live L3 (three arenas + interner) vs lanes vs the two sleep artifacts (`sleep.jsonl`, `fingerprint.txt` — **lane-class: bytes on disk that no arena owns**, which is the whole reason a sleep costs 0 new state bytes) | [06 §1](06-resource-budget.md), [07 §11](07-genome-build-plan.md) |
| device reality | `VkPhysicalDeviceMemoryProperties` budget and `VkMemoryAllocateInfo` actuals for this machine, plus the device name — **reported, never assumed**; the spec's 1.6 GB figure is arithmetic on a schema constant, and the panel is where the two meet without pretending to have predicted each other | [06 §8](06-resource-budget.md) |
| edge touches this thought | counted from the walk | [06 §4](06-resource-budget.md)'s `FRONTIER_CAP · ρ · K_MAX_ROUNDS` |
| frontier size, rounds, settle-set size | per thought | kinds 3, 4 |
| settle latency | wall-clock **as telemetry, stamped in the report, quoted nowhere else** | [06 §13](06-resource-budget.md)'s rule |
| **sync count vs the one-sync-per-thought budget** | a counter that must read `0` on the v1 CPU walker and `1` on the v2 GPU path, plus LoopNet's `flush_count` | [06 §6](06-resource-budget.md), [docs/16 §16.5](../docs/16-think-walk-hardware.md) |
| hash cost | sub-digests per shelf + total per tick, with the `HASH_FORMAT` tag (**OD-60**) | [06 §7](06-resource-budget.md) |
| **sleep cost** | the same five counters `G6` publishes, printed live: `sleeps_completed` / `sleeps_interrupted` / `walks_per_sleep`, `bytes_changed_per_sleep`, `edges_dimmed_per_sleep`, and `hash_bytes_per_sleep` — the last one being the P3 chain repair, the term a builder will want to shortcut | [06 §5.1, §7, §10](06-resource-budget.md) |
| sleep's row-count check | `rows_live` immediately before and immediately after a sleep, **side by side on one line**, because the pair is the assertion: equal, always. A panel that printed only the delta would hide the case that matters (**G13(i)**) | [06 §2](06-resource-budget.md)'s 0-new-bytes row |
| cadence observed | quiet ticks between sleeps as a count, against `SLEEP_EVERY_QUIET_TICKS` — **a measurement of a constant, never a suggestion about it** (§9's sweep-artifact row) | [06 §5.1](06-resource-budget.md) |
| lane lag + queue depth | frames behind (§2.2), inbox depth | — |
| degree | `max_degree` and the live degree distribution, against the reciprocal table's declared range | [07 §12](07-genome-build-plan.md)'s A7 |
| instrumentation state | when `SI_PROFILE` is unset, every timing cell prints **`—  (instrumentation off)`**, never a `0`. A blank that looks like a measurement is how an unmeasured number gets quoted | [00 §5](00-stack-workspace.md) |

---

## 8. The five panels side by side (the contract, in one table)

| panel | reads | writes | holds no |
|-------|-------|--------|----------|
| CHAT | 0, 1, 8, 10, 11, 12, 13, 17 | `sense_inbox.siro` | row-edit handle, label literals, content word list |
| EMOTIONS | 3, 5, 6, 7, 11, 13 (+ `SIst` for bakes) | — | a settable field, an emotion name, a stored median |
| TESSERACT PREVIEW | 2, 3, 4, 7, 8, 13 (+ `SIst`, + mapped genome) | — | a re-simulated walk, an embedding, a force layout |
| ORGANS STATUS | 1, 8, 9, 10, 11, 13, 14, 15, 16, 17, 19, 20 (+ `SIst`, + `sleep.jsonl`, + `fingerprint.txt`) | — | a level or threshold it can move; a sleep trigger; a seed |
| VRAM/PERF | 16, 19 + accumulators exported by A through the `SI_PROFILE` sidecar counters on the lane | — | an assumed device number |

Every panel is **the same code in live and replay mode** (§2.2). A panel that
works only live is a second data path, and a second data path is how a display
starts disagreeing with a gate report.

---

## 9. What the GUI must never be able to do (and the one open item)

| action | why it is refused | the invariant it would break |
|--------|-------------------|------------------------------|
| a slider/field that sets `a`, a gain, a threshold, or the modulation curve | a settable affective state is a fakeable machine — the job's own sentence | I8, R-21, R-34 |
| edit, delete, merge or "forget" a row or an edge | nothing is dropped, only aged; provenance is write-once | I4, §5.3 law (c) |
| a **"promote this inference to knowledge" button** over a `6 cooccur-weak` candidate | promotion is **an answer-to-ask or a `/teach`, typed through the channel** — a click that commits a belief is the clean-room law dying in a UI, and it would also let a candidate reach FACT without provenance a transcript can show | I2, [04 §21 U.4](04-algorithms.md), **OD-58** |
| a "send this line as grimm" toggle | only the seam mints `source: grimm`; the owner's process minting a supporter line would defeat G12(e) at the keyboard | I3, I10, R-35 |
| a **live counterfactual probe button** (re-run the walk minus one edge, on click) | it would need either a new sense-channel command — a new reflex **and** a new §3.2 string constant, so G5c's diff changes meaning — or a side door into the walk. v1 keeps probes in the harness | R-03, [04 §15](04-algorithms.md), [00 §9](00-stack-workspace.md). **OD-67** if the owner wants them |
| a widget that reads a sweep artifact and displays it as a *setting* | a sweep output is not a design input; re-reading one into the build is R-33's failure shape, and the GUI is the most tempting route for it | [gates/README.md](../gates/README.md), R-33 |
| a **"sleep now" button**, or a `si-cli sleep` subcommand | sleep's triggers are a quiet-tick counter and the freeze path, both **inside the loop** — a click that starts a night is an inbound event that is not the sense channel, and because each sleep walk is a tick (**OD-78**) the click would move `t` while writing no `context` row. The interrupt half of the story is the opposite direction and it is legal: an owner keystroke *arriving* aborts the remaining passes (**OD-79**), which is SI hearing the owner, not SI being poked | I3, I5, **OD-78/79/81**, [10 §7](10-temperament-sleep.md)'s "never" row; `check_no_seed_control_surface`, `check_no_viewer_state_write` |
| a **re-roll / "new temperament" control** | the birth seed is chosen at creation and **never re-rolled**; it sits inside item 9's hash, so overwriting it mid-session does not re-temper this individual — it silently renames a *different* one and invalidates every hash line above it. A new individual is `--seed` at launch, in a new run directory, with its own header | [10 §3, §5](10-temperament-sleep.md), **G14(b)(c)(j)**, I5; `check_no_seed_control_surface` |
| a **weight or "dim this edge" slider** on the SLEEP or PREVIEW pane | `weight` is FLUID, and sleep's P3 is the only writer that fires with no input at all ([04 §26](04-algorithms.md) law 25). Every weight change is a dated, logged event, which is what lets the owner ask *when did this go quiet* and get a `sleep.jsonl` line instead of "it drifted". An owner-set weight is undated, unlogged, and the first step back toward R-07's tunable-affect machine | law 25, **G13(b)(d)**, [10 §6](10-temperament-sleep.md) D.2 |
| writing any file the loop reads except `sense_inbox.siro` | the one-way contract | OD-48, observer-only |
| re-simulating the walk for a prettier animation | the panel would then be a second walker, and the owner would be watching a picture of a mind the build does not have | the third law, OD-44's determinism split |

---

## 10. Deferred, reserved, and the batch that comes next

| item | status |
|------|--------|
| the **SLEEP panel** (§6 O11) and the **TEMPERAMENT FINGERPRINT** (§6 O12) | **landed with the batch they were named by**, and they arrived the cheap way: frame kinds 19 and 20, two tail-only files ([00 §8](00-stack-workspace.md)), and **no new lane, no new state byte, no new door**. §9 grew the three refusals they would otherwise have invited |
| a per-edge dimming history — "show me how this word faded" as a chart | refused with a reason, and the refusal is the same one [06 §12](06-resource-budget.md) records: `(last_touched_at, weight)` plus `sleep.jsonl`'s aggregates already answer *when did this go quiet*, and a per-edge history would be a fifth arena (**R-12**) |
| a pass-graph / staging view over the four passes | parked with **OD-80**; if sleep staging ever lands, O11 grows and **kind 19 does not** — a pass id is already in the payload, which is what makes the deferral a display question rather than a schema question |
| `read_monologue` / monologue exposure to the owner | **OD-38**, pending; O10 renders blank-but-present |
| non-ASCII rendering | deferred with a reason (§3.1), and the sense record keeps the bytes verbatim meanwhile |
| multi-window / multi-monitor, themes, export-from-panel, a search box over the lattice | v2. Exports come from `--export` on the loop side ([01 §9](01-data-schemas.md)), never from a panel, so that "the GUI produced this file" can never be a provenance story |
| a debug overlay that shows internals the panels do not | refused: §8 already gives every panel the same source the gates read, and an overlay is where an un-citable number starts its life |

---

## 11. What this file decided, and what it hands to the owner

**Decided here (HOW):** the one-way frame contract and the 21-kind output enum,
each kind bound to a record the loop already writes so the GUI adds no facts; the
transport behaviors for lag, death, mid-session open and replay; CHAT's rendering
rules (glyph atlas, no dictionary, ASCII scope) and its command scope (typed
`/correct` and `/teach` only); the EMOTIONS re-specification that removes the
cancelled residents from the job text; the deterministic identity-hash layout for
the 3D preview; the counts-not-levels rule for ORGANS STATUS; VRAM/PERF as the
single instrument that fills [06 §10](06-resource-budget.md)'s UNMEASURED fields;
and the forbidden-widget table, which is the anti-cheat law expressed as UI.

**Decided here for the TEMPERAMENT/SLEEP batch — three small calls, all of the
"which artifact is the truth" kind:** O11 renders `sleep.jsonl`'s per-pass lines
**verbatim, field set per pass**, rather than a merged sleep summary, because a
merged summary would be a second presentation of the same facts and the two would
eventually disagree; the fingerprint reaches the panel as **kind 20 forwarded from
the loop's own re-derivation** rather than as the viewer calling
`temperament::derive` itself — and the honest note is that the viewer *could*: its
one crate edge is `si-core` ([00 §2](00-stack-workspace.md)) and `derive` lives
there, so this is a **rule, not a missing dependency**. It is the right rule for two
reasons: §1's third law says every number the viewer draws was produced by the loop
**and published on a lane**, and re-deriving in B would let a build-variant mismatch
print a fingerprint that disagrees with the session's own `fingerprint.txt`; and
**G14(c)**'s byte-compare is the loop's evidence, not the panel's opinion; and the
cadence is displayed as a **count pair** rather than a progress bar, because a bar
that fills is read as a thing the owner is entitled to interrupt.

**Handed to the owner:**

| id | question | lean here |
|----|----------|-----------|
| **OD-66** | does "renders the same buffers the walker computed" mean a **shared live buffer** between A and B, or is a **frame-driven** render (the walker's own per-round numbers, published on the lane it already writes) enough? | **frame-driven.** A shared ring would put a per-round coupling and a second synchronization point between the owner's screen and the knowing loop, and [06 §6](06-resource-budget.md)'s v1 zero-sync structure is the more valuable of the two. The law's real target — *never a re-simulation* — is met, because the viewer draws numbers the walk produced |
| **OD-67** | may the owner trigger a **counterfactual probe** from the GUI? | **no in v1.** If it is wanted, the clean route is a sense-channel command, which means a new born-in reflex and a new G5c allowlist line — a design change with a UI on top of it, and it should be decided as that rather than shipped as a button |
| **OD-68** | what does **G5c's byte-exact string diff** scope to, now that the build has a GUI? | the **loop binary**; `si-vulkan`'s chrome strings are audited by reachability instead (no crate edge, one writable file), plus a separate assert that the viewer contains **no** label word, emotion word or content list |
