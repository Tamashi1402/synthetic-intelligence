# §16 — THINK: the Walk (spreading activation) + Hardware Contract

Owner-decided mechanism: the thinking loop is **a walk over the Tesseract
edge web; a thought is what the walk settles into**. Same machine as a
recurrent loop: state `h` = which hyperpoints are lit, `W` = the edge web,
`K` rounds or until settle.

## 16.1 The walk

- **Per-token pulses in word order** — NOT one whole-context blob. Order
  must prime: "what is" lights the frame before "apple" arrives; the state
  carries the full context anyway.
- Each round: lit points spread along edges (`weight × source activation`);
  points sum / keep / decay.
  `h_k = decay · norm(h_{k-1} + W_eff·act_{k-1})` (form; exact operator fixed at
  build, constants are tuning legs §16.5; `act` is the **activation** vector —
  it is *not* the arousal scalar, which round 3 made a live per-token quantity
  and the collision was too dangerous to leave in the one equation the build
  reads). Where
  **`W_eff = W ⊙ (1 + β · σ(edge.bake, a_now) + γ · m(point, a_now))`** — the
  **per-edge bake term** (§20.4) plus the **per-point median term** (§20.3),
  normalized per round (§18.5 cap — state changes which connections are cheap,
  nothing else; `γ` and the exact shape of `m` are **OPEN-30**, and §20.3 forbids
  `m` from being anything but recomputed-at-generation).
- **The three knobs arousal turns (§18.1, I8).** `a` — one scalar, not a vector —
  enters the walk as **version-tagged modulation of its own parameters**: higher
  `a` ⇒ **wider reach** (more candidates per round / larger spread radius),
  **faster decay** (steeper `decay` above), **lower settle dominance** (τ_settle
  shifted, so a hot walk keeps spreading instead of committing to its first
  attractor). This is the *only* wired affective mechanism: *"stung systems
  think differently."* It changes how the thought travels, never what it is
  about, and the curve `a → (reach, decay, τ_settle)` is swept before quoted
  (OPEN-16a, same discipline as §16.5).
- **`a` is live, and its effect lags one round (§20.1, I8).** Arousal integrates
  at **signal events** — a token landing on a dark spot, a frame lighting, a
  hot-baked edge conducting — **during** the walk, not once at its end. The
  modulation the new `a` implies applies from the **next** round (version-tagged
  lag, OPEN-29), so *the sentence colors itself as it is read* but the frame that
  lit first still finishes its job: **no emotion mush.** Rounds before a token
  and rounds after it legitimately run at different `a`, and the trace records
  which round used which value.
- **Two states, one loop (§18.1, I8):** the walk carries `h` (activation) and
  `a` (one quantized arousal scalar) and makes exactly two calls into the
  **drive interface** — `drive.update(signals, t)` (integrate — anonymous
  numbers in, next `a` out) and `drive.conduction()` (the bias that produces
  `W_eff`). **Round 3 changes the call *count* per thought, not the door set:**
  `drive.update` now runs once per signal event inside the walk (previously once
  per thought) and `drive.conduction()` returns the bake term **and** the
  per-point median term (§20.3, §20.4, **OD-29** for the clock unit). The slot is
  the walk's; the integrator is `arousal.py`'s, behind those doors — the walk
  names no emotion, holds no emotion-shaped state, and never touches `arousal`
  directly (OD-14, OD-20, §18.1). **Round 2's difference from round 1 is the size
  of the slot, not the plumbing:** one scalar where six resident levels used to
  be, and the emotion words moved all the way out of the control path into
  `audit.py` (§18.6).
- **Settle criterion:** `‖h_k − h_{k−1}‖ < τ` ⇒ thought done; **max-K hard
  stop** ⇒ reported as UNCERTAINTY (never a silent answer — surfaces via
  §9's uncertainty rule).
- **Hub policy is load-bearing:** `Heat` connects to everything, so undamped
  hubs make every thought the same mush. Normalize each source's
  contribution by connectedness (PageRank-style degree damping) + per-round
  decay. **The decay constant is the "cooling"** — it controls thought
  reach.
- **Frontier stays ~10² points** (only lit points propagate) ⇒ cost
  ~10⁵–10⁶ edge touches per input **regardless of graph size**. (Owner-
  supplied design targets — UNMEASURED until runs.)
- **Oscillation rule (OPEN-11):** a 2-cycle that never satisfies τ hits the
  max-K stop ⇒ uncertainty readout; damping makes sustained oscillation
  unlikely but the rule must be explicit at build, not emergent-hoped.

## 16.2 Substrate per config (clean-room preserved)

The walker is innate machinery; **which web it walks depends on config**:

| config | web | consequence |
|--------|-----|-------------|
| CONFIG SI | **L3 only** (frozen L1/L2 provably unopened — I2/G5c unchanged) | birth = nearly empty connectome, can barely form thoughts; the **first committed row** (in trace v2: the answer to SI's own first ask, §17.2.3) creates the first real OWNED edges. Developmental stages are literal neuro-development: the connectome grows only from conversation — **and since [docs/21](21-grimm-supporter.md), only from the owner's half of it: a supporter line is a pulse train like any input, never a new edge (§21.4, G12(d)).** |
| CONFIG ENGINE | L1 + L2 + L3 merged web, OWNED edges weighted above genome edges at NOW | mature thinking; OWNED-vs-genome disagreement still reported (§5.3 law) |

## 16.3 Embeddings — banned, even as helpers

Graph-path generalization replaces every proposed embedding crutch:
paraphrase recall = the walk ("colour of an apple" lights `apple`, spread
finds the OWNED `apple→red` edge your `/correct` strengthened). Embeddings
are **BANNED even as "helpers"** — an embedding that scores edges is a
hidden prior wearing a hat; that is how the anti-cheat law dies quietly.
(→ risk R-14.) The only numeric edge weights in v1 are: taught weight
(init from §8.4 confidence, **plus the edge's `bake` = `B_max · tanh(k ·
a_formation)`**, §8.4/§18.5/§20.4a — **round 3's OD-33 reversal: the write side
now does read `a`, but only at the formation instant and only through this one
saturating curve, and never as a read-time multiplier**; the stamp's own
`magnitude` field stays the firing signal's value under OD-22),
repetition-strengthened (each
successful serve +Δ), and genome
`weight` fields frozen at t=0 in ENGINE. **Charge is legal here and an
embedding is not, and the difference is checkable:** charge is a scalar +
**signal kind** stamped by *this session's own signals* on a row the owner can
audit (§18.6), it lives in no continuous similarity space, it was not
pretrained on anything, and its *bake is permanent and bounded* while its
*provenance* stays (I4, **OD-27**, §20.4a). An "edge-scoring embedding" carries a whole vector of
borrowed content and never dies. R-14's grep and G9's parity leg both stay
armed.

## 16.4 Consequences folded into other sections

- **Read path (§7):** the walk is wrapped by the decode layer —
  **decode → light → walk → settle → reverse-decode** (§7.0); key
  generalization solved without embeddings — retrieval IS the walk
  settling; route reporting = activation trace; decode succeeded + walk
  died at the intersection ⇒ NAMED missing slot ⇒ targeted ask-back
  (§7.1/§7.4).
- **WHAT (§7.1):** a knowledge gap is physically visible — activation dies
  below threshold where no rows exist. **Dark spots in the spread ARE the
  gap signal** — but detection ≠ asking: gaps are handed to the **drive
  interface** ([docs/17](17-drive-layer-curiosity.md)), whose curiosity
  resident prioritizes, budgets, and emits them as questions (§7.2).
- **Arousal (§18):** the walk is where every signal is *born* — τ-misses and
  max-K stops, the settle result (§8.2), dark spots (§17.3), novelty spikes at
  write time (§6.2) — and all of them feed the **one** scalar via
  `drive.update`. The only thing coming back is the three-knob modulation plus
  the capped `W_eff` (§18.1). The loop is the sensor; `arousal.py` is the
  integrator; **`audit.py` is the only place a feeling gets a name** (§18.6).
- **THINK idle (§17.3, §19):** an idle tick runs the **same walk** with no sense
  event. Precedence is fixed (§19.1): first the **open-question queue revisit**
  — re-walk each pending ask against current knowledge; if a row now covers the
  slot, resolve it (`status → resolved`); then RE-PARSE if due; only then
  **ignition** — the seeded four-weight sampler lights one hyperpoint and the
  cascade renders as the **monologue** (§19.3). Bounded: one idle thought per
  quiet tick plus a per-hour budget (§19.4), **read-only** — it may reset
  rehearsal curves and push agenda items, never write a row (I9). The queue
  pass is drive-layer work over walk output; the ignition pass is the walk with
  a different lighting source. **Neither one asks.**
- **Gain/loss (§8):** cheap and counterfactual — re-run the same walk minus
  one edge, diff the settle. Milliseconds on GPU (v1 CPU walker: frontier
  stays tiny, still laptop-OK).
- **Labels (§9):** measured from the walk — settled via OWNED edges = FACT
  (cite row + correction step); genome paths only = INFERENCE (BASE); died
  out = I-DON'T-KNOW. **The activation trace (what lit, which round, via
  which edge) is a first-class output — audit log, not debug noise.**
- **Stages (§11):** CONFIG SI's connectome growth is now the physical
  substrate of S0→S2; the CONFIG SI vs CONFIG ENGINE gap after N sessions
  remains a designed leg — **UNMEASURED**.

## 16.5 GPU/CPU contract (design constraint)

| side | role | detail |
|------|------|--------|
| **GPU** | numeric walker | frozen genome edge-list **resident, append-only**; K rounds fused into one go (**frontier-masked SpMV per round**); returns **ONCE at settle** — top-k + trace, few KB; **zero per-round sync** |
| **CPU** | judgment | input parsing → initial lighting; overlay edges (small — merged **between thoughts**, uploaded on `/correct`); gates, arbitration, labels, readout, rendering |
| sync budget | — | **one sync per thought** |

- **Freeze/wake maps over:** freeze = genome version tag; wake = upload
  genome + replay overlay (deterministic — overlay replay is the CONFIG SI
  walker run in full, I5-compatible because §16.6 quantization).
- **Residency, not compute, is the eventual wall** at 10⁸+ edges ⇒ region
  paging parked as **v2+** (a shelf-registry concern: page regions by
  `x`-shelf — design alarm R-12 if it ever doesn't fit).
- **Round 3 costs the one-sync contract nothing in v1, and something in v2.**
  The v1 walker is CPU (§16.6), so `drive.update` running per signal event is
  just a function call. The v2 GPU leg cannot ask the host for a new `a`
  mid-thought without breaking **zero per-round sync** — so the v2 contract is
  recorded as an open engineering leg, not quietly promised: either the whole
  thought's signal events are known before launch (they are not — a dark spot is
  discovered *by* the spread), or `a`'s live integration is fused into the
  on-device round loop (wordless arithmetic, deterministic within the
  quantization grid), or §20.1's lag becomes "one host sync per thought boundary"
  for ENGINE only. **UNMEASURED, v2, do not claim it works.**

## 16.6 Determinism annex (conflict resolution — see OD-08)

I5 requires byte-identical replay; GPU float SpMV with atomics does not
guarantee it. v1 rule: **the walker is CPU, fixed-order, integer/fixed-
point quantized activations**; the state hash covers the settle-set as
`{(node_id, quantized_activation)}` sorted by id — not raw floats, **and now
also the quantized arousal **trajectory** `a` (the whole per-event sequence
inside a thought, not its endpoint — §20.1), each edge's charge stamps
**including `a_at`** (§20.2, OD-28), the ignition PRNG state (§19.4), and the
version tag of the modulation-lag constant (OPEN-29)** — arousal, stamps and the
sampler's seed are loop state, so they are inside the determinism contract, not
beside it. **The per-point median is NOT in the hash: it is derived at generation
from rows that are** (storing it would be the frozen-emotional-address failure,
§20.3 / R-28). The GPU
leg (CONFIG ENGINE scale) is **v2**, and its determinism contract is:
same seed + same fused plan ⇒ same settle-set hash within the quantization
grid; bit-exactness of floats is NOT claimed and never gate-tested.
UNMEASURED until an ENGINE run exists.

**The seed is state, not time (§19.4).** Idle ignition is the only stochastic
step in the loop, so the rule is stated twice for emphasis: the PRNG seed is
part of the session snapshot and advances per logical tick, which makes the
monologue exactly replayable. **Unseeded ignition breaks G5 and is a design
violation**, not a bug — a sampler that drifts with wall-clock would make the
inner life un-auditable.

**Corollary for arousal (I8 + I5 together):** decay is a function of the
logical step `t`, never wall-clock. So a replayed session reproduces its
trajectory, its cascades and its label stream exactly (§18.4, OD-15, §19.4) —
and note what round 2 changed about the old slogan *"an idle session feels
nothing"*: idle ticks are now **thoughts** (ignition, §19.1), so a quiet session
can still feel something, but only because it *thought* something it can cite.
Silence still cannot move `a` by itself: no tick, no signal, no change.

## 16.7 Open tuning legs (develop, don't invent silently)

| id | leg | what "principled" means here |
|----|-----|------------------------------|
| OPEN-12 | hub damping + decay constants | G2 depends on them not being magic numbers: chosen by **sensitivity sweep** (grid over decay τ_K, damping exponent) with the objective = (zero-predicate-overlap recall stable) ∧ (anti-mush: mean pairwise overlap of settle-sets across distinct queries below threshold); the winning point and the full sweep curve go into `gates/G2/constants_sweep.json` — published, not hand-picked. |
| OPEN-11 | oscillation rule | max-K ⇒ uncertainty (§16.1); exact detection (period check vs pure K-stop) decided at build, evidenced by a forced-oscillation unit scene in G4. |
| OPEN-13 | phrase-batching | one-pulse-per-phrase instead of per-token as a *later tuning leg* — token-pulse first; batching only if a measured leg beats it (UNMEASURED, default off). |
| OPEN-14 | discrete error metric beyond `/correct` | candidate: error = rank of truth node in settle top-k (counterfactual minus-edge walk diff, §8.1); exact discretization is an open item, not silently fixed. |
| OPEN-16, 18, 19, 24 (→ §18.8, §19.4) | arousal legs that bite the walk: `λ_a` + signal gains (**two constants**, not a matrix) and **16a** the modulation curve `a → (reach, decay, τ_settle)`, plus bias shape (multiplicative vs threshold shift), the runaway-feedback guard, and whether an idle cascade may re-stamp the edges it re-lights | same discipline as OPEN-12: **no constant quoted before its sweep artifact exists** (`gates/G9/arousal_sweep.json`; the observed co-occurrence artifact is `gates/G9/coupling_observed.json` and is output-only, §18.3). The walk's `W_eff` term is undefined until OPEN-18/19 land, which is why G9's bounded-feedback leg is a bound test, not a value test. Round 1's composition-rule leg (OPEN-17) is **cancelled** — one scalar composes with nothing (OD-21). |
| OPEN-25 (→ §19.2) | the four ignition weights (recency / salience stamp / decay deficit / live-contradiction warmth) | version-tagged constants, same sweep discipline; G10(e) is the leg that measures them (§19.2, §19.8). |
| OPEN-29 (→ §20.1) | the **one-round modulation lag** — its value and its unit | "one round" is the owner's lean, stated as a version-tagged constant, not a proof. Its unit is the walk round of §16.1, and the lag is what makes `W_eff` at round k read `a` from round k−1. Nothing here may be quoted before `gates/G9/arousal_sweep.json` carries it; **a lag of 0 is emotion mush (R-27) and a lag ≥ K is a decorative affect layer** — G9(a) fails in both directions. |
| OPEN-30 (→ §20.3) | the median point-term `m(point, a_now)` — its shape and the weight `γ` | `γ` is a **third gain on the walk's own arithmetic**, not an emotion gain, and it still gets OPEN-12/16's discipline: swept into `gates/G11/median_sweep.json`, published with the winning point. `m` is capped by §18.5's per-round normalization, so `γ` and the cap are tuned together, never one after the other. |
| OPEN-31 (→ §20.3) | what counts in a point's stamp history, and the minimum N before the median is defined | every stamp on every edge touching the point, or only L3 content edges; and whether a 1-stamp "median" is a reading or a placeholder. Until decided, §20.3 ships the conservative shape (**undefined until the first qualifying stamp**) so a fresh session can have no emotional address for any word (birth litmus, §3.4). |
| OPEN-32 (→ §19.2) | ~~does ignition's salience weight read raw hot stamps or the point's median~~ **CLOSED: it reads the edge's BAKE** (§20.4a) | decided by the §20.4a amendment; measured as part of G10(e), bounded by `B_max` so a saturated edge cannot monopolize the sampler |
| OPEN-33 (→ §20.4a) | the bake curve's `B_max` and `k` | same sweep discipline, and the objective is **non-degeneracy**, not a nice number: across a session the bake values must keep spread over the observed `a_at` range (all-saturated = the write side encodes nothing; all-near-zero = a calm SI bakes nothing). Swept into `gates/G9/arousal_sweep.json`, reported as a distribution; **`a_formation`'s unit is OD-29's unit** (the same logical-step clock), so the three legs cannot be tuned apart |

## 16.8 Frame-scoped scratch (the v2 arc's addition — [docs/22](22-crystallize.md), [docs/25](25-skills.md))

§16.1 says the walk carries **two** states, `h` and `a`, and that is still the
whole control-path truth. The v2 arc adds a third kind of thing a frame may
hold, and it is deliberately not a third *state*:

- **What it is.** A frame MAY build a **scratch structure** describing the
  *shape* of its own traversal — a ghost cluster over co-lit points
  ([docs/22](22-crystallize.md) §22.2) or a role-graph over the relation-kinds
  it actually walked ([docs/25](25-skills.md) §25.3). Both are **form over
  bytes that already exist**: a ghost names points, a role-graph names edges.
  Neither holds content, and no word is attached to either until an owner line
  supplies one.
- **Its lifetime is the frame.** Destroyed at settle, in the same breath as
  `h`. It never survives into the next thought, never enters the state hash,
  and is therefore invisible to replay except through whatever a reader-side
  detector *noticed before the destroy* — which is exactly how §22.3 and §25.4
  accrue candidates in session state instead of rows.
- **It is never evidence.** A scratch structure may not be cited as a FACT, may
  not answer an ask, may not form or bake an edge, may not raise or lower `a`.
  Only the lattice is citable (§16.7's honesty law, unchanged). What a frame's
  scratch may do is *bias which shape a later frame composes* — read-side,
  owner-ratified, [docs/25](25-skills.md) §25.6.
- **It is bounded, and the bound is a refusal, not an eviction.** Over budget
  (the `GHOST_MAX` family, OD-97/OD-116) the frame simply builds **no
  scratch**. An eviction policy would make the presence of a structure depend
  on the history of unrelated frames, which turns a read-side hint into a
  second state machine; "no scratch" keeps every frame's behavior a function of
  that frame alone.
- **Why the law is in §16 and not only in §22/§25.** Those docs' detectors are
  **frame-scoped by definition**, and §16 is the only place that says what a
  frame is. Without this row, "the walk carries `h` and `a`" would read as a
  prohibition on the mechanism, and a mechanism that contradicts the walk's own
  contract cannot be gated — which is why the fold added the law rather than
  letting §22.2 cite a section that does not exist.
- **What it does not make the frame.** No new organ (§2's six doors are
  unchanged), no new sense, no new constant *inside* the control path: the
  scratch budget is a SPECIES row and its value is published by a sweep or not
  at all.
