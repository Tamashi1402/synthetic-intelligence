# 02 — Module Contracts, Call Order, and the Import Tables

One section per crate from [00 §2](00-stack-workspace.md): what it takes, in
what order, what it returns, what it does on failure, and the exact list of
crates it may **not** import. The laws are [docs/12](../docs/12-component-map.md)
and [docs/17 §17.1](../docs/17-drive-layer-curiosity.md); the contract language
is LoopNet's (`anyhow::Result` + `Context`, typed outcome enums only where the
design branches).

**Conventions.** All times are the logical `t` handed in by the caller — no
crate calls a wall clock except `si-core::clock`, which owns the step counter.
Every function that can refuse returns a **typed outcome**, not an error, when
the refusal is a designed branch (`Rejected`, `Ask`, `ConfigRefusal`,
`Boring`); it returns `anyhow::Result` when the refusal would mean the build is
wrong. `debug_assert!` guards on every law-shaped precondition; the release
build keeps the ones named below.

---

## 1. `si-core` — schemas, addressing, quantization, rng, clock, event log

| surface | inputs (ordered) | returns | notes |
|---------|------------------|---------|-------|
| `Address::new(t, x, y, z_ref)` | logical step, shelf id, row handle | 12-byte address | no allocation, no lookup (§1) |
| `hyperpoint_id(key) -> u64` | canonical key string | content address | `md5` prefix, identity not security (OD-49) |
| `Quant::{to_grid, from_grid, mul_inv}` | value, grid tag | grid integer | reciprocal multiply only (**OD-45**); round-half-up |
| `Rng::{next_u32, next_fraction, snapshot, restore}` | — | sample, `RngState{state,inc,spare}` | PCG32 + Box-Muller, exactly LoopNet's `tesseract-core/src/rng.rs`; **snapshot rides the session checkpoint** so ignition replay is byte-identical (I5, §19.4) |
| `Clock::{step, tick, recency(age)}` | logical step | the **one** recency curve | R-29: §8.4's decay deficit, §19.2's ignition recency, §20.3's median weights and sleep P3's **dimming** arithmetic ([10 §6](10-temperament-sleep.md) D.1) all call **this** function; no other crate may define a decay, and `DIM_BASE` is a *share* bound, not a fourth τ |
| `EventLog::{push_row, push_decision, push_signal, push_label}` | record | offset | two lanes: owner-facing `transcript`/`stream` and `mindlog`; the `monologue` lane is **separate** (§19.3, OPEN-23) |
| `StateHash::compose(...)` | the **nine** item classes of [01 §10](01-data-schemas.md), 1–9 in that order | digest | order-fixed; a caller cannot pass a median because no median field exists, and cannot pass a stored temperament vector because item 9 carries its **seed** instead ([10 §4](10-temperament-sleep.md) P.3) |

**Error behavior:** `si-core` never panics on data; it panics on **contract**
(non-zero reserved byte, misaligned record, unknown grid tag). Those are build
bugs, not runtime states.
**May import:** nothing. **May never** know what a shelf's *contents* mean — it
carries schemas, not semantics.

## 2. `si-store` — `lattice.py` + `layers_frozen.py`

| surface | inputs | returns | contract |
|---------|--------|---------|----------|
| `append_row(env, body, edges, stamps)` | a fully-formed record | `row_id` | **append-only, the only write primitive**; rejects (assert) any attempt to mutate a row in place (I4) |
| `age(superseded_id, at_t, by_id)` | loser row, step, winner | — | sets `superseded_at`/`superseded_by`; the loser's stamps and `bake` **stay** (§5.3, §20.4) |
| `active_at(shelf, s)` | anchor step | iterator of live-at-`s` rows | `written_at ≤ s < superseded_at ∨ ∞`; NOW is `s = max committed t` |
| `anchors(shelf, t')` | a **past** anchor | re-weighted view | history queries re-weight aged edges at their interval (§7.3) — this is the only door to a non-NOW walk, and it is what makes G3/G11(d) archaeology possible |
| `terms::*` (lexicon, interner) | token / id | term record, `defined_by` | `defined_by = NULL` is gap **G1** |
| `queue::*` (`questions`) | — | open/resolved rows | doubles as the **idle agenda** (§19.1); the revisit pass reads it before ignition may run |
| `Frozen::load(config, manifest)` | `--config` flag | `Ok(rows)` \| `ConfigRefusal` | **the §6 guard of [00](00-stack-workspace.md)**: `si` returns `ConfigRefusal` before forming a path string; `engine` loads and checksums |
| `build_tools::*` (genome import, mmap index, export) | files | artifacts | the **only** `rayon` user in the hashed-path-adjacent crates, and it is off the walk (OD-50) |

**Error behavior:** a `ConfigRefusal` is a *success* path in CONFIG SI (the
assert did its job); a corrupted container (magic/version/`len % 4`) is
`anyhow` and aborts the run — a silently-partial lattice would be a false-pass.
**May never import:** `si-walk`, `si-learn`, `si-read`, `si-affect`, `si-audit`.
Storage does not think, does not feel, and is not told what to remember.

## 3. `si-affect` — `arousal.py` + `drive.py` (the tissue and its six doors)

This crate is the whole of I8's installed machinery. Its public surface **is**
the drive interface; nothing else is `pub`.

| door (the only six) | inputs (ordered) | returns | contract |
|---------------------|------------------|---------|----------|
| `prioritize(gaps, now)` | gap set from the read path, step | ranked `Vec<Gap>` | salience × recency × usage; recency **imports** `Clock::recency` |
| `emit(ranked)` | ranked gaps | `Vec<Ask>` ≤ budget | curiosity policy; `what is X?` fires identically for gibberish (R-16) |
| `budget()` | — | per-input cap + dedupe rule | **queue rule + integer ceiling only.** It takes no `a`, no level, no signal (OD-22 stands for this door) |
| `salience(formation_ctx)` | `a_formation` (u8), the step | `{bake u16, stamp}` | `B_max · tanh(k · a_formation)`; **read at the formation instant only** (OD-33); takes **no event-kind argument** — a `signal == contradiction ⇒ k *= 3` table is R-21(c) and fails G9 statically |
| `update(signals, t)` | the anonymous signal values of [01 §5.3](01-data-schemas.md)'s six kinds, in event order | next `a` (u8) | `a' = clamp(a·λ_a + Σ gains(signal_i), 0, 1)` — one λ, quantized, logical time (OD-15). Called **per signal event inside the walk**, not once per thought (§20.1) |
| `conduction()` | current `a`, the point set touched this round | `{β·σ(edge.bake, a_at, a_now), γ·m(point, a_now)}` — one quantized dict | the **per-edge bake term + the per-point median term**, already capped per round (§18.5). The median **arrives as a return value and never as a field** (R-28) |

**Internal order that must not be re-shuffled:** `update` integrates and stores
the new `a` in the walk's slot; `conduction` for round *k* reads the `a` produced
by events up to round *k−1* (**the one-round lag**, OPEN-29/OD-29). Callers
cannot see this — it is why the lag is testable (G9(a) asserts both halves: no
mid-thought variation = decorative; same-round variation = mush, R-27).

**Error behavior:** an empty signal list returns the *decayed* previous `a`, not
zero (silence is decay, not reset — [docs/16 §16.6](../docs/16-think-walk-hardware.md):
"no tick, no signal, no change"). A signal number outside §18.1's six kinds is
a **panic**: it would be a new sensor, and a new sensor is I8's forbidden
privileged input.
**May never import:** `si-store`, `si-walk`, `si-learn`, `si-read` — the tissue
sees anonymous numbers, never rows, and never the thing it modulates.
**May never contain:** an emotion word (§3 of [00](00-stack-workspace.md)), a
boolean-shaped integrator, a per-resident λ/baseline/hysteresis (round 1's
`affect.py` is cancelled, OD-21), or a decay/recency constant of its own except
`λ_a` — it imports the clock (R-29, §12 rule (iv)).

## 4. `si-walk` — `walk.py` + `idle.py`

| surface | inputs (ordered) | returns | contract |
|---------|------------------|---------|----------|
| `light(pattern, word_order)` | decoded pattern | initial lit set, round-0 activation | **per-token pulses in word order**, never a context blob (§16.1); OD-55: a plural-fold alias set lights every alias member with the token's activation, split by count, and the alias set is logged |
| `walk(lit, K, τ, config)` | max rounds, settle threshold | `Thought{settle_set, activation_trace, a_trajectory, stop_reason}` | round *k*: `h_k = decay · norm(h_{k−1} + W_eff·act_{k−1})` with `W_eff` **from `drive.conduction()`**, not computed here; hub damping normalizes each source by connectedness; frontier ~10² (design target, UNMEASURED) |
| `settle_or_stop(...)` | — | `Settled` \| `MaxK` \| `TauMiss` | max-K ⇒ **uncertainty readout**, never a silent answer (§9); each stop reason is itself a §18.1 signal fed to `update` |
| `idle_tick(now, queue_state)` | step, whether the agenda is empty | `Ignition \| Revisit \| Reparse \| Nothing` | **precedence is fixed**: queue revisit → due RE-PARSE → ignition, and ignition only when the agenda is empty (§19.1); ≤ 1 idle thought per quiet tick + per-hour ceiling in logical ticks (OPEN-26) |
| `ignite(entropy)` | the seeded `RngState` | one lit hyperpoint | weights: recency · **edge `bake`** (OPEN-32 closed: not raw stamps, not the median) · decay deficit · live-contradiction warmth (OPEN-25). **Reads `bake`, writes nothing**; the bake it reads is bounded by `B_max`, so a saturated edge cannot monopolize the sampler |

**Order within a round, fixed:** gather signals from events → `drive.update` →
advance `a` → `drive.conduction` for the *next* round → spread → hub-damp →
normalize (reciprocal multiply) → test `τ`. Reordering the normalization after
the settle test instead of before it changes attractors and is a determinism
failure, not a style choice.

**Error behavior:** an empty lit set returns an empty settle-set with
`stop_reason = NothingLit` — that is the honest birth miss, and it is *data*, not
an error (§1.3). A cycle that never satisfies `τ` hits the max-K stop (OPEN-11),
it does not hang.
**May never import:** **`si-learn`** (this missing edge *is* I9/R-25: idle
thought cannot commit), `si-audit` (no labels here), and `si-sleep` (a walk is a
thought; it cannot schedule a consolidation inside itself — sleep is called by
`si-session` at step 12b, between thoughts), and it may not name a
drive, a policy, or a feeling (OD-14's semantic reading, G8's swap-test leg).
The walk holds `h` and one `u8` `a` slot and makes exactly the two door calls;
**G8's leg asserts the walk's call sites grew by zero** even across round 3's
per-event arithmetic.

## 5. `si-learn` — `gate.py` + `reparse.py`

The write pipeline, in the order the law fixes it
([docs/06 §6.1](../docs/06-write-path.md)):

| step | inputs | returns | contract |
|------|--------|---------|----------|
| 1 `pending_ask_match(line)` | decoded line + `source` | `AskMatch \| NoMatch` | matches **only** `source: owner`; a `source: grimm` reply to an open ask returns `NoMatch` and the ask stays open (R-32, the one branch the supporter batch added) |
| 1a `teach_short_circuit(line)` | `source` + shape | `Teach(pattern, response)` \| `NotATeach` | `/teach … => …` is tested **before** step 1 — a teach is never read as a reply. `source: grimm` shaped like a teach is **inert input**, it does not reach this step at all (G12(d)) |
| 2 `candidate(...)` | pattern, stage | row skeleton | S0 ⇒ `s_r_pairs`; S1+ ⇒ `facts` via re-parse or direct |
| 3 `surprise(sig, response, slot)` | normalized signature | `{novelty, coverage, surprise}` | `novelty ∈ {0, 0.5, 1}` by §6.2's three cases; `surprise = novelty − coverage` |
| 4 `boring(...)` | surprise | `Confirm(Δrepeats, Δconf)` \| `Continue` | `surprise = 0` ⇒ **write nothing**; the bump is gated on `source: owner` (R-35) — a grimm echo touches only `clock`'s rehearsal curve |
| 5 `consistency(candidate)` | candidate vs live rows | `Clean` \| `Conflict(flag)` | the five rules of §6.3, no sixth; a conflict is **never** dropped as boring |
| 6 `arbitrate(conflict)` | tiers + marker + repeat count | `{branch, outcome}` | §6.4's table verbatim: `T3-vs-any → supersede` · `marker-correction` · **`ASK` (commit nothing)** · `insistence` · recency-within-tier → conf → lower step id. Returns `Ask` as a **first-class outcome**; **takes no affective argument** (OD-24) |
| 7 `commit(...)` | row, `a_formation`, firing signal | `row_id` | `drive.salience(·)` **once**, at this instant → the edge's `bake` + stamp; init `conf + bake` clamped to [0, 99]; then the event-log entry `{type, policy_branch, asked_at?, resolved_at?, old, new}` |
| 8 `resolve_ask(row_id)` | committed answer-to-ask | — | sets the question row `status: resolved, resolved_by` |
| 9 `reparse(sr_rows)` | step-ordered S-R rows | derived `facts` rows | runs on idle ticks at S1+; **idempotent**; inherits stamps as **history only** with `bake = 0` (G7) |

**Error behavior:** every rejection is a **named outcome** with a reason string
in the mind log — a silent drop is the failure G1 hunts. A `source: grimm` line
that somehow reaches step 7 with a candidate row is a panic: §5.4's field says
belief rows carry `speaker: owner` and the commit is a bug (G12(e)).
**May never import:** `si-walk` (the gate is not a thinker), `si-audit`,
`si-supporter`, `si-sleep` (reciprocally: the write gate cannot invite a
consolidation, so no commit path can "tidy up" by running a pass — [00 §2](00-stack-workspace.md)
win 4); may never name an emotion or read `a` **except** to hand
`a_formation` to the `salience` door at step 7.

## 6. `si-read` — `retrieve.py` + `predict.py` + `askengine.py`

| surface | inputs | returns | contract |
|---------|--------|---------|----------|
| `retrieve(pattern, anchor)` | decoded pattern, `NOW` \| `t'` | `Thought` + route report | retrieval **is** the walk settling; the activation trace `{(node, round, edge)}` is the route report and a first-class output (§7.3). `anchor = t'` moves the NOW filter to an anchor filter **and** moves the median's recency weights (OD-30's extension, G11(d)) |
| `gaps(thought)` | activation trace | gap set G1–G4 | §7.1's four predicates; a dark spot read **live** is an arousal signal, read **at settle** is a nameable slot — the same fact, two roles, and only the second may be emitted (§20.1) |
| `handoff(gaps)` | gap set | `drive.prioritize(·)` | the read path **detects, never decides** (I7); this is the only line of communication to the drive |
| `counterfactual(edge, context)` | one settle-trace edge | `gain(e) = diff(S, S′)` | §8.1; only edges **in the settle trace** are probed, so cost stays bounded |
| `guess_seed(step)` | step | nonce token ids | R11's letter-class n-gram shapes of tokens seen **this** session, PRNG from `RngState`; **zero rows written** (G1) |

**Error behavior:** a walk that dies with no rows returns
`Miss{named_slot: Option<u16>}`; `Some` ⇒ ask-back path, `None` ⇒ nonce fallback
(§7.4). Both are successes in the design sense.
**May never import:** `si-learn` (read-only), `si-audit`; and it may not
recompute the conduction bias — it **applies exactly what `conduction()`
returned**, capped (§18.5), and passes the anchor through (§20.5).

## 7. `si-front` — `sense.py` + decode + `render.py`

| surface | inputs | returns | contract |
|---------|--------|---------|----------|
| `tokenize(line)` | raw text | token ids | R1 whitespace/punct/lowercase; R2 plural fold as an **alias candidate**, never a merge (R-04) |
| `decode(tokens)` | token ids | `Pattern{frame, slots, terms}` \| `Undecoded{named_dark}` | R3/R14 shape-only frames from the §3.2 allowlist; **gibberish parity is the litmus**: `what znark is floop` yields the same frame with different slots (R-18, G5d) |
| `reverse_decode(thought, cited)` | settled top-k + cited rows | surface text | v1 = template copier: content tokens ⊆ cited row surfaces + learned chunks |
| `render_monologue(thought)` | an idle `Thought` | monologue frame | **the same face**, so an internal cascade costs no new generator and cannot be a hidden generator (§19.3); output goes to the separate lane only |
| `label(thought, caps)` | settle + trace; `caps` carries `INFERENCE_MAX_HOPS_OWNED`, **handed in by `si-session`** because `si-front ↛ si-walk` ([00 §2](00-stack-workspace.md)) — the same argument-passing shape as `walk(lit, K, τ, config)` | `{label, source_tag, citation, route_counts{owned, base}}` | §9's rules, **demote-only**, computed from post-render token containment against cited rows (R-08); takes no affective argument (OD-24). `route_counts` are **trace-derived at this call and never stored** — the hop count of a settle is a readout, and a persisted one would be a second source of truth about the trace. An over-cap owned route falls through to the uncertainty branch ([04 §21 U.2](04-algorithms.md)) |

**Error behavior:** an undecodable `/teach` left side ⇒ `Undecoded` + named dark
spot + **zero rows** (§1.4's anti-cheat condition). A render that emits a token
not in any cited row **demotes the label**, it does not error — the demotion
*is* the contract.
**May never import:** `si-affect`, `si-learn`. It carries the `source` tag
through, assigned by the receiving module, **never inferred from the text**
(§21.7: trust the wire, not the words).

## 8. `si-supporter` — `mcp.py` + `speakgate.py` (the seam and the mouth filter)

| surface | inputs (ordered) | returns | contract |
|---------|------------------|---------|----------|
| `read_trace / read_state / read_vocab / read_rows / read_monologue` | — | serialized artifacts | **read-only, and every read is logged** (OD-40(iii), R-33); `read_*` selects what he talks about, which §21.8(2) publishes as a real limit |
| `sense_input(line)` | the emission attempt | `Admitted(source: grimm)` \| `Rejected{reason}` | the **only** place a `source: grimm` line may be born; hands `sense.py` nothing else and holds no handle on lattice/walk/drive/arousal |
| `speakgate::layer1(tokens)` | emission tokens | `n_untaught` | membership test over the **taught** lexicon shelf; at t=0 the lexicon is empty ⇒ rejects everything ⇒ **structural muteness** (R21, G12(c)) |
| `speakgate::layer2(claim)` | normalized claim | `{novel_claims, ref_row_id}` | **strict single-row restatement, row-id-referenced** (OD-40(i) — Mike's call); this is the layer that rejects *"apples are good because vitamins x and y"* when every token is legal (§21.3) |
| `gate_log(entry)` | every attempt, admitted or rejected | — | append-only; feeds `gates/G12/speak_gate_log.jsonl` and `rejection_reasons.json` |

**Error behavior:** a rejected emission **never becomes a sense event** — no
signal, no `a` movement, no stamp, no row, exactly one gate-log line (§21.2).
That ordering is why the filter sits *upstream* of `sense.py`; a filter that ran
after admission would have already changed SI.
**May never import:** `si-affect` (**the missing edge that enforces rule (vi)**:
neither module may read `a`, `charge`, `bake`, the median, or any door — a gate
that could see arousal is one refactor from `soothe()`, R-34); `si-walk`;
`si-learn`. It may never reference `grimm/` **as a dependency** — the membrane
is in the build, the model is not (rule (v), OD-37). It adds **no** string
constant to the §3.2 allowlist beyond the two `source` enum literals.

## 9. `si-audit` — `audit.py`, the reader

`classify(signals, a_trajectory, behavior) -> LabelStream`. The six-label table
of [docs/18 §18.2](../docs/18-emotions-engine.md) is its whole installed
vocabulary; adding a seventh word (e.g. `soothed`) is an OPEN-22 ruling, not a
code change. It also writes `gates/G9/coupling_observed.json`, which is
**output-only**: an observed co-occurrence that becomes a wiring rule fails the
gate that produced it (§18.3, R-24).

**Structural guarantees, and they are the point:** it is imported by **nothing**;
it reads the persisted event log and nothing live; deleting the crate changes no
state byte except the label strings in the log (**G9(c)**, R-24). Every label
must be reproducible from the signal rows and `a` already in the log, byte-for-byte
on replay — an uncitable label fails G9(b) the way an uncited FACT fails I6.

## 10. `si-vulkan` — residency + the viewer (the engine has **zero** windowing code to reuse)

| surface | inputs | returns | contract |
|---------|--------|---------|----------|
| `genome_residency(SIgen)` | mapped index by `(x, y)` and `z` adjacency | resident buffers | persistently mapped `HOST_VISIBLE|HOST_COHERENT`, one batched command buffer per dependency chain, **flush only when the host reads or overwrites a live buffer**, `flush_count` exported — LoopNet's `tesseract-compute` idiom verbatim |
| `viewer_frame(stream_tail, mindlog)` | framed records | swapchain image(s) | renders the walker's **own quantized buffers**; the panel reads grid ints, never float vectors (R-14); observer-only ([08](08-gui-observability.md)) |
| `upload_overlay(edges)` | edges formed between thoughts | — | **between thoughts**, never mid-thought (§16.5 CPU/GPU split) |

**Error behavior:** device loss ⇒ the viewer degrades to headless; **the loop
never notices** (OD-48: the loop never blocks on the viewer).
**May never import:** `si-learn`, `si-read`, `si-walk` — Vulkan displays, it
does not decide (OD-44: the *walked* graph is the CPU oracle in v1).

## 11. `si-session` — the orchestrator

One THINK tick, in the order the design fixes (each arrow is a call, not a
thread):

```
1  next sense frame from sense_inbox (or NONE — a tick with no input is legal)
   B18: the frame body carries speaker_identity + conversation (§9's transport byte)
2  front.tokenize → front.decode          → Pattern | Undecoded
3   (if the line is `/correct` or `/teach`: learn's short-circuits, §1.4/09)
4  learn.pending_ask_match(line, source)  → commit path or continue
   OD-128: an origin=2 (nomination) ask answers only in the owner lane
5  walk.light(pattern)                    → round-0 lit set
5a B15 silt.tick(conversation, now)       → the ACTIVE table erodes BEFORE anything deposits
5b B16 tugs = read.event_view_index(rows, stamps, eventlog)  → Vec<OriginTug> (derived at frame load, writes nothing)
   `EPISODIC_TRAVERSAL = off ⇒` the vector is empty and step 6's arithmetic is bit-identical to B13's
6  walk.walk(..., &tugs)  ─┬─ per signal event → affect.update(signals, t)   [per-event, OD-29]
                            ├─ before each round → affect.conduction()         [lagged one round, OPEN-29]
                            ├─ B16: a tug re-lights its points at round 0, frame activation only (≤ RELIGHT_MAX)
                            ├─ B14: every light appends one CoLight to the frame log (ghosts/role-graphs build here)
                            └─ B15 OD-104(a): the per-round spread reads NO silt — the seed choice is its only read
7  walk.settle_or_stop                    → Thought{settle, trace, a_trajectory, frame_log}
   B14/B17: the frame log is the arc's whole input; ghosts and role-graphs DIE at settle
7a B14 crystallize.observe(frame_log, store, now) → Vec<Nomination>   (cold-frame rule, OD-105)
7b B17 skills.observe(frame_log, eventlog_rows, now) → Vec<Nomination> (forwarding rule, OD-121)
8  read.gaps(thought) → affect.prioritize → affect.budget → affect.emit
   B14/B17: nomination gaps join the SAME queue (kinds 5/6, origin 2), same cap, same dedupe window
9  front.label(thought) + calibrate demote-only → output frame + citation
   B20 9a: TONGUE ruling row ON? → license = retrieval.cited's own surfaces, folded here
            → si_tongue::realize(&graph, anchor, &license, &slot_keys, &style, budget, &mut rng)
            → Option<String>; None ⇒ the canonical surface, unchanged. `label`/`tag`/
            `citation` are already bound and are NOT re-derived (§18, [13 §4.1](13-tongue.md))
10 front.reverse_decode → stream.siro     (owner-facing lane; a NAMELESS surface is never printed)
11 learn.commit(class, ...) if a candidate survived  → drive.salience ONCE → bake+stamp
   B18: the tier arm runs first — a class other than Owner returns Refused{SpeakerNotRepresentable}, zero bytes
12 session: quiet tick && agenda empty? → walk.idle_tick → monologue lane (read-only)
   B15: ignite gains &silt_view (the fifth term, unarmed at 0) · B18 OD-126: a window may SUPPRESS, never cause
12b session: quiet-tick counter hit SLEEP_EVERY_QUIET_TICKS (or a freeze asks)? → sleep.run(passes 1..4)
    │        each pass boundary re-reads the inbox; a pending frame aborts the rest
    └─ one quiet tick per walk: each walk closes with 13's flush→compose→advance,
       so a sleep of w walks leaves the clock at t₀ + w ([04 §2](04-algorithms.md) note 7)
13 eventlog.flush; StateHash::compose; clock.advance     (nine items — unchanged)
```

`Xa` means "inside X, after X's own body"; the arc's six insertions — **5a**
(silt's tick), **5b** (the tug index), **6's round-0 re-light**, **7a**
(crystallize), **7b** (skills) and **12's fifth term** — and the tongue's one
insertion — **9a** — keep steps 1–13's numbers and order, and 05 §21's per-stage
enable sets are the reason they appear here rather than in a separate tick. Steps
**1**, **4**, **10** and **11** gain no new step, only a new *arm* inside a step they
already run. **9a is the only insertion that changes a printed string and no byte of
state:** it reads `rendered`, the `citation`, the graph and the style, and its whole
write set is one local `surface`.

Steps 6 and 7 are one pass — the numbering is for the contract, not for the
thread. Steps 12's ignition **cannot reach 11** (the missing `si-walk ↛ si-learn`
edge), and neither can 12b (`si-sleep ↛ si-learn`, [00 §2](00-stack-workspace.md)
win 4). Steps 1–11 run for a `source: grimm` line too, except that 4 never
matches and 11 never receives a candidate from it.
Step **12b** is sleep's whole entry point, and its position is load-bearing three
ways: strictly after 12 (ordering law 13 is unchanged — revisit → RE-PARSE →
ignition, and a sleeping mind daydreams first), strictly before 13 (so the hash
is of the **post-sleep** tick and a replay that skipped a rescale diverges, which
is what makes G13 testable), and it is a *quiet-tick-only* step, so an interrupt
can never land inside a pass ([10 §7](10-temperament-sleep.md)).
**Sleep reads none of the arc** — no silt table, no candidate, no skill row, no
event view: a pass sweeps rows and edges, and everything the arc holds is either
RAM the pass cannot name or a log it may not open (§15).
**May never import:** `si-audit` — labels are produced downstream, over the
written log, so no label can steer a later step (R-24).

**Four ordering laws join [04 §26](04-algorithms.md)'s list as 27, 28, 29, 30** (the
first 26 are unchanged, and the arc renumbers nothing; 30 is the tongue's):

* **27 — a decay is before the walk, a deposit is after it.** 5a precedes 6 and
  step 7's tail deposits, so silt never biases a selection using light from the
  very frame it is about to bias. The lag is OD-29's one-round shape, and it is
  what makes G16(d)'s "clears backwards" a pure function of the transcript.
* **28 — observation is after settle, before commit.** 7a and 7b sit between 7 and
  11: a detector that ran before settle would nominate on a half-formed thought,
  and one that ran after commit could read its own minted row as evidence. This is
  the ordering that makes G15(e)'s spam bar and G18(f)'s generality bar
  expressible at all.
* **29 — a nomination enters the queue as a `Gap`, never as an emission.** 7a/7b →
  8: neither detector can call `emit` (`↛ si-affect`), and `drive` decides at the
  same `ASK_CAP_PER_INPUT` and the same `DEDUPE_WINDOW` as every other gap. "A
  busier world may not buy more asks" ([docs/26 §26.5](../docs/26-world.md),
  OD-128) is a consequence of this line, not a rule added on top of it.
* **30 — the voice is after the label and never before it.** 9a runs after 9's
  `front.label`/`calibrate` have produced `label`, `tag` and `citation` and binds
  nothing but the surface it prints: [docs/27 §27.2](../docs/27-tongue.md) check (a).
  The ordering is also structural twice over — `si_front::label` reads
  `Rendered.content`/`Rendered.containment` and never `Rendered.surface`, and
  `si-tongue ↛ si-front`, so the crate has no symbol through which a label could be
  read, let alone rewritten. `G20(b)` grades the construction ([13 §4.1](13-tongue.md)).

## 12. The grep rules, restated as build checks

Each row is a `check_*` leg in [05](05-gate-harness.md) (static, no run needed,
**OD-53**), and each is *also* enforced structurally by a missing crate edge above
— the redundancy is deliberate: the grep catches what a future refactor re-adds.

| rule | assertion | structural edge that enforces it |
|------|-----------|----------------------------------|
| (i) emotion words | `curiosity\|surprise\|satisfaction\|caution\|boredom\|frustration` appear in **`si-audit` only** | `si-audit` is a separate crate with a clean directory to scope to; every other crate fails the build on a hit |
| (ii) idle is read-only | no `commit`/write-path symbol reachable from `si-walk::idle` | `si-walk ↛ si-learn` |
| (iii) no stored mood | no field named `median`/`bias`/`mood` in any schema, export, or cache | [01 §11](01-data-schemas.md): the field does not exist |
| (iii-b) **the bake is the opposite case** | `bake` **is** stored, deliberately; a grep that bans both is a bug that passes checks | [01 §5.3](01-data-schemas.md): write-once, bounded |
| (iv) one clock | `si-affect` contains no decay/recency constant but `λ_a`; `B_max`/`k` are **scale** constants, so they may live there | `si-affect → si-core::clock` only |
| (v) the membrane | nothing under `src/` references `grimm/` | `grimm` is not a crate at all |
| (vi) no affective setter | `si-supporter` reads no `a`/`charge`/`bake`/median/door | `si-supporter ↛ si-affect` |
| (vii) embeddings | no `Vec<f32>`-similarity API, no numeric model file, zero tolerance | R-14 scan; the allowlist of [00 §9](00-stack-workspace.md) carries no such crate |
| (viii) **sleep creates no row** | no `commit`/`append_row`/`age` symbol reachable from `si-sleep::`; the only store calls sleep makes are the four in-place FLUID writes of [10 §8](10-temperament-sleep.md) | `si-sleep ↛ si-learn` — the same trick as (ii), applied to a new organ (win 4 of [00 §2](00-stack-workspace.md)) |
| (ix) **sleep never brightens** | in `si-sleep::p3`, `w' ≤ w` for every live edge; the assert is a *contract* check, not a policy branch, because [10 §6](10-temperament-sleep.md) D.1's arithmetic (`0 < d ≤ 1`, `w ≥ floor`) makes brightening unrepresentable | §9's refusal of a mass-preserving rescale (R-07, I6) |
| (x) **the temperament is derived, never stored** | no `Temperament` field in any container, schema, export or cache; `birth_seed` + `seed_format` are the only individual bytes that persist | [01 §10](01-data-schemas.md) item 9's absence row; **G14(c)** re-derives and byte-compares |
| (xi) **a guard is never seeded** | no LAW row of [03 §6](03-constants-registry.md) — and no cap, bound, tier or floor that *bounds a safety property* — is read from `Temperament`; the type carries the 18 drawn values and nothing else | [10 §2](10-temperament-sleep.md) test 2; `check_temperament_closed` |
| (xii) **nomination proposes, never mints** (B14/B17) | no `commit`/`append_row`/`age` symbol reachable from `si-crystallize::` or `si-skills::`; their only store calls are reads and their only outputs are values | `si-crystallize ↛ si-learn`, `si-skills ↛ si-learn` — [00 §2](00-stack-workspace.md) win 5 |
| (xiii) **the detectors are wordless about arousal** | no `a`, `charge`, `bake`, median, door or gain symbol reachable from the arc's three crates, and no emotion word in their directories either (§1 above) | `↛ si-affect` on all three; `check_emotion_words`'s widened scope |
| (xiv) **silt names no row** | `si-silt` imports neither `si-store` nor `si-learn`; its key type is a `u32` point handle, so a shelf is not expressible in its source | `si-silt ↛ si-store` — win 6 |
| (xv) **frame scratch dies at settle** | no `Ghost`/`RoleGraph`/`CoLight` value escapes `si-walk`'s frame; nothing under `si-store`/`si-learn` can name those types | [docs/16 §16.8](../docs/16-think-walk-hardware.md); `check_ghost_is_frame_scoped` |
| (xvi) **the arc adds no vocabulary, no byte, no shelf** | `RECORD_FIXED_BYTES == 112`, five `prov_kind` values, seven OWNED edge kinds, eleven shelves, `RESERVED_SHELF_COUNT ≥ 2`, `HASH_ITEM_COUNT == 9`, `EDGE_KIND_COUNT` unmoved, and [00 §9](00-stack-workspace.md)'s allowlist byte-identical | `check_record_layout` + `check_string_constants` + `check_registry_size` — all three must **stay green**, which is the proof, not a new rule |
| (xvii) **a speaker class is a function, not a field** (B18) | no field named or typed as a speaker class in any record, cache, export or container; `speaker_identity` appears only in a framed sense body and `transcript.txt` | [docs/12](../docs/12-component-map.md) rule (ix); `check_speaker_class_derived` — the same shape as (iii) and (x) |
| (xviii) **the voice is drawn, never installed** (B20) | no `STYLE_*` name appears in a launch-constant list, a scenario's `constants` block, or any published `*_sweep.json`'s `axes`; `launch_constants`' guard text is byte-identical to the `Species \| Ruling` refusal | [13 §3.2](13-tongue.md)'s guard rows; `check_style_never_installed` — the same shape as (xi) and the B7 no-sweep law |
| (xix) **the tongue is a leaf** | `si-tongue`'s dependency set is exactly `{si-core, si-walk}`; the strings `si_store`, `si_front`, `si_learn`, `si_affect` and `fs::` appear nowhere in its text | `si-tongue ↛ si-store` — **win 8** of [00 §2](00-stack-workspace.md); `check_tongue_is_leaf` |
| (xx) **a license is the only mouth** | `si-tongue` holds no `&'static str` table of more than one entry, no writer symbol and no file handle; every push into its phrase buffer takes its bytes from a `License.surface` | [docs/07 §7.5](../docs/07-read-path.md) "utter only what the citation licenses" (**OD-181**); `check_tongue_license_only`, R-21 |
| (xxi) **no arousal byte reaches a weight** | `si-tongue` reads no `a_at`, `a()`, `arousal` or `Integrator`, and the render seed's operands are `birth_seed`, `Style::digest()` and `thought_id` and nothing else | **OD-168** widened to the voice, realized by **OD-182**/**OD-183**; `check_tongue_reads_no_a` |

## 13. `si-sleep` — `consolidate.py` *(the thirteenth crate; the logical id is
**OD-84**)*

Numbered last on purpose: [02 §11](02-module-contracts.md) and §12 are cited by
number from six other files, and a section's *position* in this file carries no
meaning — the dependency table of [00 §2](00-stack-workspace.md) does.

The full design of the passes, the refusals and the arithmetic is
[10 §6–§9](10-temperament-sleep.md); this section is only the call surface, the
order, and what each call may not do.

| surface | inputs (ordered) | returns | contract |
|---------|------------------|---------|----------|
| `sleep::due(quiet_run, freeze_requested)` | the same quiet-tick predicate idle uses (§19 S.1), plus the freeze flag | `Yes \| No` | a **counter and a checkpoint**, never a wall clock ([10 §7](10-temperament-sleep.md)). `No` while any sense frame is pending: a sleeping mind has to be a quiet one |
| `sleep::run(budget)` | `SLEEP_MAX_WALKS`, the pass mask, the inbox handle | `SleepReport{passes_completed, per-pass counts, rows_touched, bytes_changed, aborted_reason}` | runs passes P1→P4 **in that fixed order**; before each pass it does the same non-blocking inbox read as tick step 1, and a pending frame ⇒ `Aborted{at_pass}`. The budget is **one number spent in pass order** (P1 first, P4 the remainder — **OD-85**, [04 §28](04-algorithms.md) Z.2). A partial sleep is a legal state (**OD-79**): the report is logged, the counters advance, the replay re-derives it |
| `sleep::p1_replay()` | the `context` rows with `t > last_slept_at_t`, oldest-first, capped at the remaining budget (newest kept when truncating) | `{walks, events_available, edges_touched, rows_written: 0}` | writes `last_touched_at` **only** — idle's existing 8-byte allowance ([04 §19](04-algorithms.md) S.7). Coverage is 1.0, so §6.2's own arithmetic says `Boring`; the bump path must stay shut (R-07) |
| `sleep::p2_integration_scan()` | the composite ledger + `links` rows | `{candidates, stable_ge_N, rows_written: 0}` | **writes nothing in v1** (`INFERENCE_PROMOTION` off, **OD-58**). If promotion is ruled on, the one permitted write goes *through* `si-learn::gate.commit` — i.e. the missing edge is added by an owner ruling, in the open, and not by a refactor |
| `sleep::p3_rescale()` | every live OWNED edge, shelf by shelf, `(x, y)` order | `{edges_scanned, edges_changed, bytes_changed, max_relative_shift, within_shelf_rank_inversions, cross_shelf_rank_inversions, floor_hits}` | `w' = floor_shelf + (w − floor_shelf)·d(edge)`, `d` from `Clock::recency` and from nothing else — the **only** place in the build that rewrites `weight` while quiet, and the only per-edge factor a lawful rescale may use ([10 §9](10-temperament-sleep.md)'s (b′) refusal). Single-threaded, fixed order, integer, reciprocal multiply (OD-45/OD-50) |
| `sleep::p4_calm_reprocessing()` | the live web, point-handle order, `a = 0` per thought, **no RNG draw**, walks = the budget remainder | `{walks, points_swept, rows_read_calm_not_settleable, rows_relit, stamps_written}` + the calm view | full deterministic sweep in place of the seeded pick, and the sweep is **full even at 0 walks** (**OD-85**): selection is a read, walking is the spend. Writes the rehearsal pair (and OPEN-24's re-stamp if it lands) **and nothing more** — no agenda candidate, because a candidate is a `questions` row ([04 §28](04-algorithms.md) Z.2, **G13(i)**). **Its report is terminal** (`check_calm_view_is_terminal`): the calm view may not feed a weight, a floor, `d`, or `ignite` |
| `temperament::derive(birth_seed, seed_format)` | the `u64` and the format tag | `Temperament` — the 18 drawn values, in `DERIV_ORDER` | lives in **`si-core`**, not here: it is `Rng` + the registry's legal ranges + the grid, all core-owned, and every consumer crate already imports core. A pure function; called at wake before the first round of the first thought ([04 §25](04-algorithms.md) W.0) and re-called per process. Never serialized ([01 §10](01-data-schemas.md) item 9's absence row) |

**Order inside a sleep, fixed:** `due` → P1 → (inbox check) → P2 → (inbox check)
→ P3 → (inbox check) → P4 → (inbox check) → report → counters advance → return
to the tick. P3 precedes the checkpoint write *always*, never the reverse: a
rescale is the pass that must not be half-applied, and the checkpoint is the
durability boundary ([00 §5](00-stack-workspace.md)'s atomic write).

**Error behavior.** A pass that touches a STRUCTURE or PROVENANCE byte is a
**release-assert panic**, not a logged refusal — [06 §2](06-resource-budget.md)'s
partition says a consolidation pass *may touch FLUID and nothing else*, so any
other write is a build bug of the same class as a misaligned record.
`Aborted` is a designed outcome and returns `Ok`. A `due(Yes)` that produces zero
passes (empty lattice at birth) writes one report line and changes zero bytes —
the honest empty case, and G13(f) counts it rather than failing on it. Sleep
never panics on data: a cold-start sleep over a nearly-empty L3 is legal.

**May never import:** `si-learn` (win 4: sleep cannot make a row), `si-audit` (no
labeling here, and a sleep that could read labels would be a second causal path
from the labeler into control), `si-front` (it renders nothing), `si-read`
(retrieval answers a question; a sleep asks none — P4 walks directly),
`si-supporter`, `si-vulkan`. **Reciprocally**, `si-learn` and `si-walk` do not
import it: nothing in the write path or the thought path can schedule a pass.

## 14. `si-crystallize` — `crystallize.py` *(the fourteenth crate; B14;
[docs/22](../docs/22-crystallize.md))*

The arithmetic — the four rules in order, the merge, the exclusions — is
[04 §29](04-algorithms.md); this section is only the call surface and what a call
may not do. It is numbered like §13: a section's position carries no meaning, the
dependency table of [00 §2](00-stack-workspace.md) does.

| surface | inputs (ordered) | returns | contract |
|---------|------------------|---------|----------|
| `crystallize::observe(frame_log, store, now)` | the frame's `CoLight` slice, a **read-only** store view, `t` | `ObserveReport{nominations: Vec<Nomination>, candidates, frames_counted}` | a pure function of its arguments plus its own candidate table; reads **live L3 rows only** (frozen rows may be fillers, never members — I2); reads no `a` and cannot, having no edge to `si-affect`; a member-set hash **merges, never stacks**; sets that are open-contradicted are excluded; over `GHOST_MAX` the frame builds no ghost and the detector simply sees fewer members |
| `crystallize::cold_frame_count(log)` | the frame log | `u32` | the OD-105 rule made callable: a frame with any `primed` light contributes **no** recurrence, and recurrence counts **distinct frames** at `≥ DELTA_MIN_CRYSTALLIZE` apart. This is what makes G16(e) gradeable: silt on/off cannot move a recurrence count |
| `crystallize::table()` | — | a read-only view | for the audit and `crystallize.jsonl` only. Nothing in the loop reads the table to decide anything: the only route from a candidate to state is an ask, and the only route from an ask to a row is the owner's line ([docs/22 §22.0](../docs/22-crystallize.md)'s litmus) |

**Error behavior.** A candidate that cannot be named is not an error: it is a
`NAMELESS` mint once the owner answers ([01 §11](01-data-schemas.md)). An `observe`
that cannot fold its frame log (empty, or all-BASE members) returns an empty
report and writes one log line — I2's refusal is a **count**, not a panic. It
panics only on a structural violation, i.e. a frame-scratch type reaching a
container, which rule (xv) already refuses at compile time.

**May never import:** `si-learn` (win 5: it cannot mint), `si-affect` (no `a`, no
doors, no gain), `si-walk` (it is *handed* the log), `si-front` (it has no mouth),
`si-silt`, `si-skills` (the [docs/23 §23.4](../docs/23-context.md) collision is
resolved by neither detector holding an edge to the other), `si-session`,
`si-audit`, `si-vulkan`, `si-sleep`. **Reciprocally**, `si-walk`, `si-learn`,
`si-read`, `si-front` and `si-supporter` do not import it: only `si-session` calls
it, at step 7a.

## 15. `si-silt` — `silt.py` *(the fifteenth crate; B15;
[docs/23](../docs/23-context.md))*

| surface | inputs (ordered) | returns | contract |
|---------|------------------|---------|----------|
| `silt::tick(conversation, now)` | the thread key and `t` | entries decayed | erodes **the active table only**, at step **5a** — before anything this tick can deposit (ordering law 27). `d = LAMBDA_SILT + (255 − LAMBDA_SILT)·Clock::recency(now − last_touch)/255`, `level' = mul_q12(level, d)`, `< THETA_SILT ⇒ 0 and the entry drops. **One curve, no second τ** (R-29; the same treatment `DIM_BASE` gets, [03 §5.1](03-constants-registry.md)) |
| `silt::deposit(conversation, point)` | handles, at step 7's tail | `()` | `level' = min(SILT_LEVEL_MAX, level + SILT_BUMP)`. No source weighting (**OD-106 ships full-weight parity**: owner and non-owner lines deposit alike), no `a` scaling (**absent from the code**, not zero — [docs/23 §23.11](../docs/23-context.md)), and no row argument: a point handle is the widest name this crate can speak |
| `silt::level(conversation, point)` | handles | `u16` Q12 | the read that marks a `primed` co-light |
| `silt::view(conversation)` | the thread key | `&BTreeMap<u32, u16>` | **its only consumer is `walk::ignite`'s seed choice** — the fifth term. `spread`, `settle_or_stop` and `conduction` read it zero times, which is [docs/23 §23.4](../docs/23-context.md)'s collision ruling (OD-104 option (a)) made a static fact by `check_silt_ignition_only` |

`SILT_MAX` is a **counter reported by the audit**, never an eviction policy and
never a ranking: a table that exceeds it is a finding in `gate_report.json`, not a
truncation in the loop. The table is session RAM, hash-exempt by OD-103, and its
per-tick line goes to `silt.jsonl`; **nothing reads that file back** (R-33).

**May never import:** `si-store` (win 6: it cannot name a row), `si-learn`,
`si-affect`, `si-walk` (the table arrives as an argument at the sampler),
`si-front`, `si-read`, `si-crystallize`, `si-skills`. Only `si-core` — `clock` and
`quant` — and that single-edge import is what makes it the smallest crate in the
build.

## 16. `si-skills` — `skills.py` *(the sixteenth crate; B17;
[docs/25](../docs/25-skills.md))*

| surface | inputs (ordered) | returns | contract |
|---------|------------------|---------|----------|
| `skills::observe(frame_log, eventlog_rows, now)` | the frame's role-graphs and the citable outcome rows | `ObserveReport{nominations, candidates, shapes, forwarded}` | the **generality bar before the recurrence bar** (`K_FILLER` distinct filler-sets, then `R_MIN_SHAPE` cold frames at `DELTA_MIN_SHAPE` apart); outcome tags read only the event-log kinds `{question-resolved, realized-gain, owner-reply, none}` — **rows, never `a`** ([docs/25 §25.4](../docs/25-skills.md) rule 3); a shape whose recurrences share one filler-set is dropped from its own table **and forwarded** to crystallize's input (OD-121), the forwarding being a session-side decision, not a message between detectors — so one frame is counted by exactly one detector |
| `skills::composition_preference(settle)` | the ratified skill rows (read-only) | an ordering hint | `SKILL_COMPOSITION = off` ⇒ the empty hint and bit-identical B13 traces. On ⇒ it reorders the **candidate iteration** only; it may not add, drop, re-weight or re-light anything, and it cannot reach `commit` — law 20 is already the sentence "a composition reaches the gate never" |

**A skill is a row, not a program.** The crate holds no interpreter, no closure and
no plan type, and `si-walk` does not list it, so "executing a skill" has no name in
this build ([docs/25 §25.11](../docs/25-skills.md) note 2, [00 §2](00-stack-workspace.md)
win 7). The only path from a shape to a skill row is `move-nomination` ask → the
owner's answer → `si-learn::commit` ([01 §6](01-data-schemas.md)'s `SKILL` note).

**May never import:** `si-learn`, `si-affect`, `si-walk`, `si-front`, `si-silt`,
`si-crystallize`. Reciprocally nothing in the loop imports it except `si-session`,
at step 7b.

## 17. Episodic views and the class function *(no crate; B16 and B18;
[docs/24](../docs/24-episodic.md), [docs/26](../docs/26-world.md))*

Two of the arc's five families add **no crate**: one derived surface in a crate
that exists, one pure function in the kernel.

| surface | home | inputs | returns | contract |
|---------|------|--------|---------|----------|
| `read::event_view_index(store, stamps, eventlog)` | `si-read` | the committed rows | `Vec<OriginTug>` | **derived at frame load, zero writes** (OD-110 ships *derived*, so shelf 11 stays reserved-empty); `event_id` **is** the sense row's `y` — no new id space; capped at `RELIGHT_MAX` per frame; `EPISODIC_TRAVERSAL = off` ⇒ the vector is empty and the walk's arithmetic is bit-identical to B13's. Deleting the index changes traces and citations only, which is why G17(b) is a grep, not a migration |
| `walk::relight(&tugs)` | `si-walk` | the vector, as an ordinary `walk(...)` argument | frame activation | lights at round 0 at `mul_q12(PROV_LINK_TUG, level_of_belief)`, **read-only**: no `Stamp` is constructed, no `bake` assigned, no `confidence` written, and `src = 2 idle-re-light` is never emitted (OD-112 rides OPEN-24, which is off). `a_at` vs `a_now` is §20.2's existing conduction term applied to a re-lit row — **no new curve** |
| `world::class(identity, owner_identity, grants)` | **`si-core`** | the frame's identity, the owner's, the (empty tonight) grant view | `Owner \| Grimm \| Trusted \| Stranger` | a **pure function computed at generation, stored nowhere** ([docs/12](../docs/12-component-map.md) rule (ix)); `si-front` calls it when the frame arrives, `si-learn` consults the result at step 11 — and it lives in the leaf because **neither may import the other**, so a `si-front::world` module would force an edge [00 §2](00-stack-workspace.md) forbids. `grants` is empty at birth (OD-125: *coded, not granted*), so `Trusted` is unreachable in a pre-B18 run and every pre-arc gate stays bit-identical |
| `learn::commit(class, …)` | `si-learn` | unchanged + the derived class | `Ok \| Refused` | one new arm **ahead** of the ten steps: a belief-row write whose class is not `Owner` ⇒ `Refused{SpeakerNotRepresentable}`, one log line, **zero bytes**. Tier (b) has no honest `prov_speaker` value, so the refusal *is* the honest state until **OD-135** is ruled ([12 §10](12-v2-arc.md)) |

**Idle's window rule** ([docs/26 §26.4](../docs/26-world.md), OD-126): a declared
window may appear only in a **deny** conjunct of the idle gate — closed means no
idle tick; open does not mean one happens, because the quiet-and-empty-agenda law
still decides. There is no path from a window to `ignite`, `sleep::due`,
`clock.advance` or a frame read (`check_windows_suppress_only`): it is the same
sentence §19.4's rate law already says about timers.

## 18. `si-tongue` — `tongue.py` *(the seventeenth crate; B20;
[docs/27](../docs/27-tongue.md), [13](13-tongue.md))*

One surface, one caller, and a return type that is the whole removal surface.

| surface | inputs (ordered) | returns | contract |
|---------|------------------|---------|----------|
| `tongue::realize(graph, anchor, license, slot_keys, style, phrase_max_steps, rng)` | `&Graph` (the tick's own — the crate **cannot build one**, because `Graph::build` wants a `&Lattice` it may not name), the anchor point handle, `&[License]`, `&[String]` slot keys already folded, `&Style`, `phrase_max_steps: usize`, `&mut Rng` | `Option<String>` | the **phrase walk** of [13 §34.3](13-tongue.md): at each step the candidates are `graph.arcs(current)` minus genome edges minus already-used points, filtered to those whose end point appears in a `License`; weight is `arc.weight` **plus** four additive style leans; the draw is the ignition sampler's own shape (one `next_q12()`, a cumulative scan). `None` is not an error — a spent budget, an empty arc set, a missing license entry and a lost slot are all the same answer, and the caller's fallback is the canonical surface, unchanged |
| `tongue::render_seed(birth_seed, style_digest, thought_id)` | the three values, nothing else | `u64` | **OD-183**: the state hash is *not* an operand. Item 3 of the nine hashed items is the stamp arena and a stamp carries `a_at`, so a seed that read the hash would read arousal through the back door — the exact thing §12 (xxi) greps for |
| `License { point, surface, keys }` | built by `si-session` at the license boundary from `retrieval.cited`'s own rows | a RAM-only value | **OD-181 as a hole, not a rule**: with no `si-store` import there is no symbol in this crate that enumerates a point, so "utter only what the citation licenses" ([docs/07 §7.5](../docs/07-read-path.md)) cannot be violated by refactor, only by an import that a build check refuses ([00 §2](00-stack-workspace.md) win 8) |

**Error behaviour is a refusal, not a type.** No `Result`, no `panic!`, no `bail!`.
A voice that cannot be realized leaves the sentence exactly as B13 wrote it, which
is what makes **V14** an additive variant and `G20(j)` a *string*-equality check
rather than a hash one ([13 §6.1](13-tongue.md)).

**May never:** touch `si_store::store::Lattice`; call `si_learn::commit` or any row
writer; read `si_affect` (no `a`, **OD-168**); read `Graph`'s `a_at` (**OD-182**);
write a file or a log line (the harness logs, [13 §1.3](13-tongue.md)); call
`si_front::label` or read a label (law 30); emit a string that is not a
`License.surface`; change `citation` (the caller appends the event ids after the
call, unchanged).

**May never import:** `si-store`, `si-learn`, `si-affect`, `si-front`, `si-read`,
`si-crystallize`, `si-skills`, `si-silt`, `si-sleep`, `si-supporter`. Its dependency
set is exactly `{si-core, si-walk}`, and reciprocally nothing in the workspace names
it except `si-session`, at step 9a and at exactly one site
(`check_tongue_is_leaf`, `check_tongue_reads_no_a`, `check_tongue_license_only`).


