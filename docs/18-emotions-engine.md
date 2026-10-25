# §18 — AROUSAL, and Emotions as OBSERVED READOUTS

Owner DESIGN LAW, amended by the IDLE-IGNITION batch. Round 1 (EMOTIONS
fragment) said: the emotions engine is **not an external system that observes
SI and adjusts dials** — it is internal to the loop, made of the same tissue as
the other organs, and it modulates the connective tissue itself: **edges carry
emotional charge.** Anything structured as `if <signal> then <behavior>` is a
rule engine wearing a costume and is rejected.

Round 2 (this batch, amends round 1): **emotions are NOT installed machinery.**
**The coupling matrix with hand-wired gains is CANCELLED.** What is innate is
only the **physics**; everything emotional is an **observed pattern in the walk**
— the same law as the anti-cheat: *no preloaded emotion semantics, just as no
preloaded word meanings.* One causal chain, no loops to name:

```
raw signals ──▶ ONE arousal level a ──▶ walk modulation          (CONTROL)
     └────────────────────────────────────▶ AUDIT readouts       (names, zero causal power)
```

**The litmus that decides what may stay innate:** at birth, in CONFIG SI, with
zero memories, **nothing fires — because no signal has a source yet.**
**An emotion system that can be felt without content is smuggled content.**

## 18.1 What remains innate — the whole innate emotional machinery

1. **RAW SIGNALS** — already emitted by the organs for their own reasons:

| internal signal | already produced by | consumed by |
|-----------------|---------------------|-------------|
| novelty spike (= surprise = novelty − coverage at write time, §6.2) | LEARN write gate | arousal (per event, §20.1) — and, through `a` at the formation instant, the edge's **bake** (§18.5, §20.4a, **OD-33**) |
| realized gain (`gain_exact` / probe gain, §8) | CALIBRATE | arousal |
| contradiction count / arbitration branch (§6.4) | LEARN + CALIBRATE | arousal |
| settle failures (τ-miss, max-K stop, §16.1) | THINK walk | arousal |
| open-question age (`asked_at` vs `t`, §17.3) | `questions` shelf | arousal |
| idle cascade crossing its own live contradiction (§19.5) | THINK idle walk | arousal |

**When each one fires changed in round 3, not what may fire:** all of these are
**events inside a thought** — a token landing on a dark spot, a frame lighting, a
baked edge conducting — and `a` takes them at that moment, lagging its
modulation one round (§20.1). The table's "already produced by" column is the
anti-cheat guarantee: every source is machinery that exists for its own
non-affective reasons.

2. **ONE arousal level `a`** — a single quantized scalar. Rises when signals
spike, decays over its own τ. **No named emotions exist at this level.**

3. **AROUSAL MODULATES THE WALK** — *"this is the only wired mechanism: 'stung'
systems think differently."* Higher `a` changes the **spread**, never the
content: **wider reach, faster decay, lower settle dominance** — the three knobs
of §16.1. **HOW** it modulates (the parameter curve `a → reach, decay, τ_settle`)
is a **version-tagged constant** — **form, not content** — and gets the same
treatment as the walk's own damping/decay constants (OD-09/OPEN-12): swept before
quoted, never hand-argued.

That is the complete inventory of installed affective machinery **as round 2
left it**. **Round 3 (§20) grew it by more form laws and no content:**
arousal integrates **live, per signal event, during the walk**, with modulation
**lagging one round**; a point's **read-side** emotional reading is the
**recency-weighted median** of its whole stamped history; the single hot event's
durable artifact is the **per-edge bake** at formation — **scaled by one
saturating curve of `a` at that instant (§20.4a, OD-33)** — with **stamps aging on
the same clock facts use.** Seven form laws, one scalar, zero word meanings, and
**two curves: one out of `a` into the walk, one out of `a` into the write side.**
Everything else in the emotional vocabulary is still a readout (§18.2).

**No privileged sensors** (round 1 law, survives): `a` has no input path from
outside the loop — nothing in the panel, config, or a designer's hand can set
arousal directly; only a signal that also changed real machinery can.

## 18.2 Named emotions are OBSERVED readouts, not internal states

Curiosity, surprise, satisfaction, caution, boredom, frustration are
**classification labels applied by the AUDIT layer** to recognized patterns over
`(signals, a, walk behavior)`. They are **not** internal states and they drive
**nothing**.

| label (audit vocabulary) | the pattern it names |
|--------------------------|----------------------|
| **curiosity** | arousal up ∧ novelty signal active ∧ asks being emitted |
| **surprise** | a sharp novelty spike at a write, decaying within a tick or two |
| **satisfaction** | realized gain ∧ a pending question just closed |
| **caution** | contradiction count rising ∧ recent ASK-branch arbitration |
| **boredom** | sustained low novelty ∧ low spikes ∧ low open-question pressure |
| **frustration** | high arousal ∧ repeated settle failure or repeated contradiction **on one slot** |

**The label never feeds back into control.** Control is exactly
`signals → a → walk modulation`. The audit layer is a **reader** of the same log
the owner reads, which is why the labels cost the system nothing — and why they
are allowed to exist.

**The supporter batch tests this sentence from outside the build, and it holds
(§21.1 L3).** Grimm is not a fourth link in the chain: he is a *source of
signals*, which is what every input line already is, and his calm arrives the
long way round — fewer tokens light, so smaller gains feed `drive.update`, so `a`
decays toward rest, so the spread narrows (§16.1). **Nothing in §18.1's signal
table gains a "soothed" entry, and no tool in his harness can move `a`, a
confidence, or a bias — I8's "no privileged sensors, no external setter" now has
a named candidate setter to refuse, which is the first time that clause has had
to be checked against a process that would *like* to help.** The one honest
extension is on the readout side: `audit.py`'s classifier *may* learn to print
*soothed* over the co-occurrence "arousal falling while a `source: grimm` line is
recent", because the log already carries the tag and the label would have no
causal role. That is a **vocabulary question, not a mechanism question**, and it
belongs to **OPEN-22** — the six-label table above is the installed set, and a
build that adds a seventh word without that ruling is adding a prior.

**Scope of that sentence, stated exactly (§20.4a):** `a` now has **two** outlets,
not one — the walk (control) and, at a single instant, the size of a new edge's
`bake` (**OD-33**). The second is not a control link and must not be cited as one:
it writes a stored birth property, decides nothing about what SI does next, and can
never be recomputed later, so no behavior is steered by `a` outside the walk. If a
future doc or implementation reads the bake as a *policy* input, this line and R-21
both fail — that is the test to run against it.

Round 1's own words were the test — *"an engine that produces emotions you
didn't write is the strongest proof it's a real organ system, not a config
file."* Installing a coupling matrix would have **failed that test on
principle**: a config file with a mood API. The demotion turns the sentence from
rhetoric into a checkable claim.

## 18.3 The coupling matrix is now an EMERGENT artifact

Cancelled as design input, retained as **output**: after N sessions the audit
layer reports **which patterns actually co-occurred** over the real signal
stream — an observed adjacency, written to `gates/G9/coupling_observed.json` and
nowhere else. It may never be read back into the build: a co-occurrence that
becomes a wiring rule is the costume failing in slow motion (§18.7, R-21).

**The honest failure mode, and it is the point:** if human-recognizable emotions
do **not** appear from the dynamics alone, **the architecture is wrong.** That
result is published as a finding, not patched with a matrix — same discipline as
§14's published limits and I6. *A config file cannot produce an emotion nobody
wrote.*

## 18.4 Arousal physics — one integrator, in logical time

- `a(t+1) = clamp(a(t) · λ_a + Σ gains(signal_i(t)), 0, 1)` — one λ for all
  signals, quantized, **in logical step `t`, never wall-clock** (I5; OD-15's
  ruling carries over: determinism outranks the biology metaphor). Round 2's
  §19 amends one consequence: an idle tick is now a *thought*, so a quiet
  session is not necessarily a feeling-less one — but `a` still moves only on a
  signal row, and a tick with no signal and no cascade changes nothing.
  **Round 3 amends the clock's granularity, not its currency (§20.1):** the
  gains arrive at signal **events** inside a thought — several per thought — and
  the modulation they imply starts **one round later**. Which logical unit is
  official for `λ_a` (per token / per round / per thought) is **OD-29**, the lag
  constant is **OPEN-29**, and the whole `a` **trajectory** — not its endpoint —
  is inside the state hash (G5a, §16.6).
- **Two guards against a feeling taking over, and they are different kinds of
  guard.** §18.5's per-round normalization is the **mechanism** guard (one stamp
  cannot dominate *this* thought); §20.3's median is the **statistical** guard
  (one thought cannot dominate the word's *future*). The second one is free —
  it is a property of which statistic is read, not an extra part — which is why
  round 3 could add "a bad hour cannot rewrite the personality" without adding
  a supervisor, a dial, or a seventh organ.
- **Nothing survives of:** per-resident baselines, per-resident λ, CAUTION's
  hysteresis, "mood = the slow components of the vector", the composition rule
  (OPEN-17 → **CANCELLED**, there is one scalar to compose nothing with).
- **What replaces "mood":** one slow tail — `a`'s own decay after a spike. §4's
  trajectory requirement is re-scoped accordingly (docs/04's closing curve
  section, plus the new idle-tail act there, **OD-26**): a curve of one scalar
  plus the audit's label stream, both citable. A visible **jump** in `a` with no
  signal row behind it is still the costume test failing.
- **Homeostasis is the decay, not a supervisor.** The one remaining feedback path
  (cascade → contradiction → arousal → walk, §19.5) is bounded by τ plus G9's
  bounded-feedback leg.
- **Consequence for the ask budget:** round 1 made emission a function of a
  *curiosity level* (§17.3, OD-19). There is no curiosity level anymore, and
  §18.1 says the walk is the *only* thing arousal may modulate — so the
  starvation budget loses its input. Recorded as **OD-22**, not silently rewired:
  current doc choice is the strict reading (budget returns to the queue rule +
  integer cap, and the interrogation phase ends because **fewer gaps surface**
  at higher arousal's faster decay, not because an urge starved).

## 18.5 Emotional edges — what survived, and what it reads now

Charge on L3 edges stays: it is the write-side record of *"this mattered while
the loop was hot"*, and it is one of the four weights that make idle ignition
non-random (§19.2). Charge is a field on the L3 edge (§5.4), not a shadow table.

- **WRITE-SIDE STAMP.** At a commit or re-weighting the edge gets
  `{signal, magnitude, a_at, thought_id, signal_ref}` — **`signal` names one of
  §18.1's six raw signals, never an emotion** (round 1 wrote `resident`; that
  field is superseded → **OD-23**), and **`a_at` is the arousal level at the
  formation instant** (**OD-28**). **`magnitude` is still the signal's own value**
  — round 2's OD-22 law holds for the stamp's provenance field, unchanged. **What
  is new is the other write-side quantity: the edge's durable `bake` is
  `B_max · tanh(k · a_formation)`** (§20.4a), the live signal sum read **through
  the one integrator at that instant**, saturating, version-tagged, swept
  (OPEN-33). **That is the reversal OD-33 exists for:** round 2 wrote "the walk is
  the only thing arousal modulates", and round 3 lets arousal reach the write side
  exactly **once, at formation, through one curve** — no category test, no valence,
  no read-time `a_at` multiplier, and `budget`/`salience`-for-asks stay under
  OD-22. Assembled inside the arousal module and handed to the write path through
  the drive interface (`salience`, §17.1) — the write path never formats a stamp
  (§6.1, §12).
  High-bake edges get a **head start** (higher initial weight, §8.4);
  cold-formative edges form weakly. **Round 3's amendment: that head start is the
  edge's BAKE — a durable birth property with no decay tail in v1, not a boost
  that decays (§20.4, **OD-27 RESOLVED**).** The stamp itself is permanent
  provenance either way (I4).
- **READ-SIDE BIASING (state-dependent retrieval), capped as before.** This one
  *is* walk modulation, so it reads `a`. Round 2 wrote it as one term; round 3
  splits it into the two artifacts §20 names, both returned by
  `drive.conduction()` so the walk consumes numbers rather than feelings
  (§16.1, OD-20):
  `W_eff = W ⊙ (1 + β · σ(edge.bake, a_now) + γ · m(point, a_now))` — where
  `σ` now matches **current arousal against the state recorded on the edge at
  teach time** (`a_at`, §20.2, OD-28: *taught scared, asked scared* conducts
  tighter than *taught scared, asked calm*), and `m` is the point's
  **recency-weighted median over its entire stamp history**, recomputed every
  thought and stored nowhere (§20.3, R-28). β and γ are parked (OPEN-18,
  OPEN-30); normalized **per round**, always. State colors a
  thought **not by changing the goal but by changing which connections are
  cheap.**
- **CAP IT (mandatory, unchanged).** Per-round normalization (`β·σ` contributes a
  fixed share of each round's total conduction, so no stamp can dominate a
  thought) plus a **session bound** on the quantized values; G9's bounded-feedback
  leg fails if the bias compounds monotonically. Exact guard shape = **OPEN-19**.
- **Re-stamping on re-lighting:** an idle cascade that re-lights a hot edge
  re-stamps it with the *current* state — the natural rehearsal +
  state-congruence loop, bounded by the same guard (lean position, **OPEN-24**).
- **Frozen genome is never stamped** (unchanged, and unchanged in force): charge
  exists **only on L3**; L1/L2 row types have no charge field, so a stamp on a
  genome edge is **unrepresentable**, not merely forbidden; G9 audits zero frozen
  stamps after a long session on top of G6's checksums.
- **Charge is FORM.** A stamp names a signal kind and a magnitude; it never knows
  what anything is *about*. Parity litmus unchanged: `what is znark?` stamps
  exactly like `what is X?`.
- **Supersession keeps historical stamps** ("taught calmly, contradicted hotly")
  and **keeps their bake on an edge that is no longer live** (I4) — the aged
  edge's head start does not evaporate; it simply cannot conduct at NOW because
  its live weight is zero, and it becomes reachable again only through an
  explicitly **history-anchored** walk (§20.5, G3 + G11(d)). What *does* fade is
  a stamp's read-side weight inside the median, on the shared clock.
- **Emotional history is 4D, the same way facts already are (§20.5).** New
  teaching at a new `t` forms new edges with new stamps; at NOW the fresh ones
  conduct easier, so **a word can flip completely** without anything being
  edited; and *"what did that word feel like back then"* is a past-anchored
  decode of the same mechanism that answers *"what color were apples"*. Bias
  matching is **current state vs stamp history, recency-weighted** — a yesterday
  stamp beats a last-year stamp at NOW even if the older one was hotter when it
  was written.
- **Aversion (OD-18) is now moot:** with no residents there is nothing to avert
  *from*; the `sign` field's purpose shrinks to attraction/reinforcement only and
  is parked for v2 rather than defended.

## 18.6 Audit — the only place an emotion word may be spoken

`a` is logged per thought next to the activation trace — since round 3 that
means its **whole per-event trajectory** plus the **per-point median** the
thought actually conducted with (§20.1, §20.3) — and the audit layer emits
its label stream beside it: *"asked with a=0.6 while novelty spiked; silent with
a decayed below the emission floor."* **A label with no cited signal row is an
unexplained behavior change and is rejected** — same honesty law as facts (I6),
same hash treatment (quantized `a` trajectory + stamps inside the state hash,
§16.6). **The median is the exception to the hash rule and it is deliberate: it
is derived from hashed rows, never hashed or stored itself (§20.3, R-28).**

**Naming rule (round 2, replaces round 1's):** the emotion words appear in
**exactly one module — the audit layer (`audit.py`)**. Not in `arousal.py`, not
in `drive.py`, not in the walk, the gates, the write path or the labeler
(R-21, §12). Round 1 granted the naming privilege to `affect.py` + `drive.py`;
that grant is **withdrawn**, because a module that both names an emotion and
sits in the control path is exactly the smuggled coupling this batch cancels.

## 18.7 Boundaries

- **Anti-cheat:** arousal is control policy over internal signals, never content;
  no signal, stamp, or label knows anything about apples. The innate part is the
  signal list, the one integrator, and the existence of a modulation curve; every
  *value* is a sweep artifact before it is a number.
- **Embeddings stay banned and the distinction still holds** (§16.3, R-14): a
  stamp is one citable scalar born from this session's own measured signals; an
  embedding is a borrowed continuous space whose similarity *is* content.
- **Curiosity keeps two meanings and must not fuse them:** as **drive policy**
  (§17: detect gap → spend an ask within budget) it is unchanged and still the
  default resident; as an **audit label** it names the pattern (arousal up ∧
  novelty ∧ asks firing). A third thing that modulates control is forbidden
  (OD-22).
- **Social emotions remain out of scope and named** (shame, guilt, anger, love
  need theory of mind). Post-demotion the reason is sharper: they would be
  **labels with no source pattern in v1's signal set** — theater by §18.2's own
  rule.
- **Costume test, re-pointed (review-time, every PR).** Two shapes, both fail:
  (a) round 1's `if <signal> then <behavior>`; (b) **any hand-wired coupling
  between emotion names — in code or in config** (a gain matrix between named
  residents is the same smuggled control in a nicer shape). Also: a label that
  feeds back into control, an `a` moved by anything outside §18.1's signal rows,
  or an emotion word outside `audit.py`.
- **CONFIG SI is born with the full physics** — one integrator, at rest, with no
  signals to integrate. There is no affective maturation to schedule (§11.1):
  what develops is the *content* the signals point at.

## 18.8 Open items (develop, don't invent silently)

| id | open item | where it bites |
|----|-----------|----------------|
| OPEN-16 (reduced) | arousal gain + λ_a (two constants, not a matrix); **16a** = the modulation curve `a → (reach, decay, τ_settle)`, version-tagged | §18.1/§18.4/§16.1 — `gates/G9/arousal_sweep.json` before any quote |
| OPEN-17 | **CANCELLED** — composition rule for six residents; there is one scalar | — |
| OPEN-18 | bias shape (multiplicative on weight vs threshold shift), now over stamp magnitudes × `a` | §18.5; determinism prefers multiplicative on the quantized grid |
| OPEN-19 | runaway guard for cascade → contradiction → arousal → walk | §19.5, G9 |
| OPEN-20 | **CANCELLED** — maturation of a matrix that no longer exists | (replaced by: does the audit classifier itself improve? → OPEN-22) |
| OPEN-21 | **DECIDED by this batch** — memory-stamp boundary: scalar + signal kind + citation; no referent, no emotion name on the edge | §5.4, §18.5 |
| OPEN-22 | audit classifier: rule-set over logged signals vs learned from co-occurrence — and who owns its threshold constants | §18.2, §12, G9(b) |
| OPEN-23 | is the label stream owner-facing or export-only? (I3: one sense channel) | §1, §19.3 |
| OPEN-24 | does an idle cascade write fresh charge stamps (lean: **yes**, bounded by §18.5's cap) | §19.4, §18.5 |

**Round 3 (§20) inherits this table rather than replacing it.** The legs that
moved: **16** now carries the modulation **lag** too (OPEN-29) because `a`
updates per signal event inside the walk instead of once per thought; **18**'s
"bias shape" question is now **two** terms — the per-edge bake and the per-point
median (OPEN-30 for `γ` and `m`'s shape, OPEN-31 for what counts in a point's
history); **19**'s runaway guard gained a second, free mechanism (the median,
§20.3) and a new clock question (**R-29**). **§18.5's "boost decays" is
superseded by OD-27** — the bake is durable at formation, and what ages is the
point's read-side median on the same curve facts use. **The §20.4a amendment adds
OPEN-33 (`B_max`, `k`) and the OD-33 reversal** described above. Nothing in §20 changes
§18.1's inventory law (signals, one integrator, one modulation curve) except by
adding the median as a **statistic over rows that already exist** — which is why
it is not a fourth mechanism and why R-28 keeps it out of storage.

## 18.9 What round 2 changed (round 1 → round 2) — no silent edits

- **CANCELLED as machinery:** six residents as internal states; the coupling
  matrix with hand-wired gains; per-resident λ / baselines / hysteresis;
  "mood = the slow components"; the composition-rule question; the `resident`
  field on edge stamps (→ `signal`, **OD-23**); the naming privilege granted to
  `affect.py` + `drive.py` (now `audit.py` only); CAUTION's role in the
  arbitration branch (arousal modulates the walk, not the label ladder).
- **RETAINED:** the `e`-slot inside the walk (now one scalar, `a`); signals →
  arousal → walk modulation; write-side charge stamps and the capped read-side
  bias; frozen genome never stamped; stamp permanent (**round 3: and so is the
  bake — "boost decays" is superseded, OD-27**);
  cite-or-reject audit; embeddings ban; social-emotion exclusion; §17's
  curiosity-as-drive-policy; §17.6's swap-test verdict (OD-14) and the six doors
  (OD-20 — `update` now integrates one scalar, `conduction` now reads stamps).
- **Amended elsewhere:** **I8** rewritten ([docs/00](00-thesis-invariants.md)) —
  installed affect is **one scalar**; emotion names are readouts with **zero
  causal power**. §2 gains the **AUDIT** readout face and THINK's idle behavior.
  §3 R17 becomes the single integrator + modulation curve; R18's stamp schema
  drops `resident`; the birth litmus is **nothing fires at t=0**. §4's trace
  annotations switch from six residents to `a` + label stream. §9's emotion words
  move to the audit layer. §12: `affect.py` → `arousal.py`, new `audit.py`, new
  `idle.py`. §13: **G9 REPLACED** + **G10 IDLE** added. §14: R-21 re-pointed,
  **R-24** (label feeding control) and **R-25** (idle masquerading as learning)
  added. §16: arousal knobs + seeded ignition in the hash. §17: budget loses its
  level input (**OD-22**). **New [docs/19](19-idle-ignition.md)** owns idle
  ignition, the monologue, and its boundaries.
- **Ledger supersession:** OD-16, OD-17, OD-18 and OD-19's level half are
  **cancelled** by this batch (OD-21 records the cancellation); OD-14, OD-15,
  OD-20 stand as written.

## 18.10 What the v2 arc inherits, and what it may not touch ([docs/22](22-crystallize.md)–[docs/26](26-world.md))

Five mechanisms arrive, and §18's installed machinery is still the one integrator
plus the curve it drives. Four statements, because each of the five could have
been built as an affective door and none of them may be.

- **The signal table gains no kind.** §18.1's list stays closed. A nomination is
  an **emission**, not a signal — the drive asks under §17's queue rule and
  integer ceiling, and asking cannot raise `a` any more than answering does. A
  re-lit moment is a **read** (G17(d): zero new stamps, zero bakes). Silt is a
  session table with no handle on the drive (G16(c)), and a grant is an owner act
  on the write path. A stranger's line is an **ordinary sense event**: novelty and
  contradiction fire on it exactly as on the owner's, because detection is
  content-free and OD-106 rules the same parity on the silt side. If any of these
  ever needed its own signal kind, that is a new dial panel and it fails G15(c),
  G16(c), G17(c) or G12(d) as applicable.
- **One `a`, not one per voice.** N speakers raise the same session-wide scalar.
  Per-identity arousal would be round 1's six-integrator shape returning under a
  social key, and it would hand the loudest stranger the mood of the room.
  **OD-129** (per-user plasticity keying) stays unruled, and the build's honest
  default follows it: nothing is keyed by identity — stamps stay per-edge and
  medians stay per-point (§20.3), so `a` and the read-side statistics do not know
  who talked.
- **The audit face grows readouts, never doors.** `audit.py` may classify over
  the new behaviors the same way it classifies over asks: *nomination*
  (recurrence ≥ floor ∧ an ask emitted), *re-lit* (a frame traversed a
  `prov-link` under `RELIGHT_MAX`), *primed* (silt's ignition term above its
  floor, **OD-105**), *granted* / *revoked* (an authority event and its
  revocation). All of it printed **after** the settle and consulted by nobody;
  the deletion check still says removing `audit.py` changes no state byte except
  label strings, in the arc's runs as in round 2's (**G9(c)**). The emotion-word
  grep grows its file list — `crystallize`, `silt`, `skills`, the event view and
  the traversal are all wordless like everything else (§12 rule (i),
  §22.8/§23.7/§24.7/§25.8).
- **R-21's four shapes, re-run against the arc.** (a) a rule engine in
  affective clothing — `if recurrence > 3: ask()` would be it, and the reason the
  threshold is a swept **SPECIES** row with a published refusal rather than a
  number the build liked; (b) no coupling matrix, unchanged, and no new door
  between `a` and any effector: the arc's five modules hold **no** drive handle
  (§12's dependency arrows, and the 19th temperament draw stays undrawn so
  warmth cannot smuggle a temperament); (c) no event-kind table behind the bake —
  and a skill row is the closest the arc comes to shape (c), which is why §25's
  law is "a row that orders composition, never a constant and never the sampler's
  temperament"; (d) no outside path into `a` — extended: **no path from a grant
  row, a silt value, or a candidate count into arousal**, which is G19's reading
  of the same leg G12(d) enforces for Grimm.
- **The empty-state litmus is unchanged and still passes:** a fresh session
  prints `a = 0`, zero labels, zero stamps, zero defined medians, zero bakes —
  with the arc's five ruling rows `off`, and also with them `on` and no signals
  (§3.4, §18's t=0 law, OD-132's fixture inherits it as its first assert).
