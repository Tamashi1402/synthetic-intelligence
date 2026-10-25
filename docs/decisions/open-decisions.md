# Open Decisions (awaiting owner ruling)

Created while merging the SI fragment into the v1 design. Each is a clash
or a gap the fragment did not decide; **current doc choice** is what
DESIGN.md/docs currently encode. Rule with `OD-n: keep/option X` and the
ledger updates.

## OD-01 — Birth guess mechanism (guess slot)
- **Clash:** demo demands SI say "blork mern" at t=0, but the v1 design only
  allowed SPECULATION when in-session co-occur links exist — impossible at
  birth; nothing could generate a guess.
- **Current doc choice:** innate reflex R11 — nonce assembled from
  letter-class n-gram stats of tokens seen THIS session, PRNG seeded by
  step (deterministic), zero content (§7.4, R-13).
- **Alternatives:** (b) never guess, always I-DON'T-KNOW — breaks the
  `/correct` protocol's error-before; (c) guess = re-use any lexicon token at
  random — less "alien", but risks looking like word knowledge.

## OD-02 — SPECULATION vs I-DON'T-KNOW boundary
- **Clash:** fragment defines SPECULATION = "no rows fired" AND
  I-DON'T-KNOW = "honest newborn default" — both describe the no-evidence
  case; at birth they'd collapse.
- **Current doc choice:** a **parsed question** always gets an answer slot ⇒
  guess ⇒ SPECULATION/NONE; anything else without grounding ⇒ I-DON'T-KNOW.
  (The question-required rule exists because the correction protocol needs a
  wrong answer to correct.)
- **Alternatives:** guess only when entity known; I-DON'T-KNOW when entity
  unknown — contradicts demo Act 0 (apple was unknown there).
- **Post-CURIOSITY note (OD-10 resolved):** the stated reason for always
  guessing — "the correction protocol needs a wrong answer to correct" — has
  weakened: ask-back + `bind_output_id` supply a clean error-before, and
  auto-commit (§1.4) lets a plain reply teach without any guess at all. The
  rule is kept as-is for now; if you want SPECULATION reserved for
  partially-grounded guesses only (no nonce on cold questions), say
  "OD-02: ask-only" and §7.4/§9 flip.

## OD-03 — Bare owner statements at S0 (S-R purity vs the demo)
- **Clash:** fragment says S0 storage is S-R pairs only, yet Act 4's "no
  wait, apples are green" is a statement, not a `/correct`, and it must
  supersede.
- **Current doc choice:** T2 owner statements write S-R-shaped rows via the
  `marker-correction` arbitration branch, plus a verbatim `context` row;
  typed fact rows are deferred to RE-PARSE (§6.4, §6.6, trace s15–s17).
- **Alternatives:** (b) statements write nothing at S0 — demo breaks;
  (c) statements write fact rows directly at S0 — abandons S-R-purity and
  makes G7 re-parse pointless.

## OD-04 — Labeling BASE-backed answers — **RESOLVED by THINK fragment**
- Fragment rule adopted: **genome paths only = INFERENCE**. BASE-derived
  answers cap at INFERENCE in §9 (former option (b)); OWNED edges are the
  only path to FACT. BASE tag kept for citation. Critical law updated §5.3.

## OD-06 — What does the walk run on in CONFIG SI? *(resolved in merge, confirm)*
- **Clash:** fragment says THINK walks the Matrix Tesseract — but Matrix =
  L2, which CONFIG SI may not open (I2/G5c handle audit). Merging it
  literally would dissolve the clean-room.
- **Current doc choice:** the walker is config-agnostic machinery; **CONFIG
  SI walks L3-only (nearly empty connectome at birth), CONFIG ENGINE walks
  L1+L2+L3** (§16.2). Fragment's own "CONFIG SI at birth = nearly empty
  connectome, first /correct creates the first real edges" supports this.
- **If you meant otherwise** (e.g. SI should get a small innate seed-web of
  pure-form nodes), say so — it changes G5c's audit boundary.

## OD-07 — The walk erases the old "reworded questions MISS at birth" rule
- **Clash:** §1.3 (yours, previous fragment) required visible misses for
  any reword until S2; the new fragment says paraphrase recall = graph path
  "colour of an apple" → lights `apple` → finds OWNED `apple→red`. After
  the first `/correct`, entity-lit reworded questions would hit immediately
  — the miss limit evaporates for frame-rewording.
- **Current doc choice:** kept BOTH honestly — misses now apply to queries
  with **no lit content node at all** and to entity renames without taught
  links; frame-reword hits are explicitly labeled term-identity and get NO
  paraphrase credit (G2 tiers T1/T2/T3, §1.3 rewrite, R-17). Paraphrase
  intelligence = T2 only.
- **Alternatives:** (b) preserve the strict rule — require full-stimulus
  match for FACT until S2 regardless of what lights (walk becomes
  answer-only, read path stays signature-gated); (c) accept walk-hits
  everywhere and delete the stage-2 paraphrase gate as meaningless.

## OD-08 — GPU walker vs determinism law
- **Clash:** "milliseconds on GPU", SpMV per round, atomics ⇒ float
  nondeterminism vs I5 byte-identical replay (and v1 promise: in-memory,
  laptop, no dependencies).
- **Current doc choice:** **v1 = CPU fixed-order quantized walker**
  (frontier ~10² makes it cheap); GPU contract (§16.5) stands as the v2
  ENGINE-scale leg; state hash covers the quantized settle-set, trace
  determinism inside G5e (§16.6).
- **Alternatives:** (b) tolerance-hash from day one including v1 — weaker
  birth test, not recommended; (c) fixed-point GPU from v1 — commit to the
  hardware early, heavier build.

## OD-09 — Constants are the new attack surface (OPEN-12/11/13/14)
- Hub damping, cooling/decay, τ, max-K now decide whether G2 passes —
  "principled" is a gate requirement, not a build-time mood (§16.7).
- **Status:** development legs recorded in §16.7; nothing here is invented
  silently — sweep artifacts (`constants_sweep.json`) must exist before
  any constant is quoted. Phrase-batching default OFF until measured.

## OD-05 — Confidence init mismatch found during split (fixed)
- Original merged doc had the trace saying 0.90 for a `/correct` commit
  while §8.4 says T3 init = 0.95. Split now encodes **0.95** for `/correct`
  commits, 0.90 for owner statements, trace aligned. No action needed
  unless owner wants a single init for both.

## OD-10 — Birth answer: targeted ask-back replaces "blork mern" — **RESOLVED by CURIOSITY fragment**
- **Input:** DECODE fragment said "consider"; the CURIOSITY fragment then
  shipped a **DEMO TRACE v2** whose Act 0 is the ask-back and whose first
  lesson needs no `/correct` ("the newborn's FIRST OUTPUT is a question").
- **Doc choice, now locked:** ask-back is **primary** whenever the dark spot
  is nameable (decode succeeded), nonce **demoted to fallback** for unnamed
  slots (§4 Act 0–1 rewritten to v2, §7.4, §9, G1, G8, R-13, R-19).
- **Consequence you should see:** the error-before that teaching binds to is
  no longer a *wrong claim* — it is a *named empty slot SI pointed at itself*
  (`bind_output_id=o1` still holds; the ask line is the output). The
  `questions` row (`q1`, `resolved_by: r1`) is what makes the first lesson
  citable as an ask-event rather than a correction-event.
- **Reversible?** Say "OD-10: nonce" and Act 0 flips back; nothing else
  breaks — the drive layer, queue, and G8 stand either way.

## OD-11 — who/when/where as innate frames
- **Tension:** the DECODE fragment lists `who/when/where is X` as innate
  question frames — those three words carry a *tiny* semantic distinction
  (person/time/place), the closest thing to content the allowlist has ever
  admitted.
- **Current doc choice:** followed the fragment — P_QUEST_W is innate,
  words added to the §3.2 function-word allowlist {who, when, where, of},
  gibberish-parity litmus applies to the frame.
- **Conservative alternative:** drop P_QUEST_W; recognize `what`-frames at
  birth only; let who/when/where arrive as taught `frame-synonym` edges
  like any other rewording. Cleaner anti-cheat story; the demo doesn't need
  them.

## OD-12 — Two teaching channels: does `/correct` keep its crown?
- **Clash:** earlier fragments called `/correct` "THE teaching act" and made
  the triple its exclusive property. The CURIOSITY fragment adds a second
  channel — a plain reply to a pending ask commits as ground truth — and
  rewrites the demo so the first lesson never uses `/correct`.
- **Current doc choice:** `/correct` demoted to the **override channel**
  (§1.4): still top tier (T3, init 0.95), still the only way to restate a
  whole sentence over an existing row, but no longer the only way in. The
  auto-commit lane is **T3-context** — same init 0.95, but gated *only* on a
  matching open `questions` row (§17.2.3), with `question_id` in provenance
  and G8(b)'s control case (no pending ask ⇒ identical line writes nothing).
- **Why it is safe:** the gate is the *drive's own pending question*, not
  content plausibility — SI cannot guess that `red.` was meant as an answer
  to something it never asked. The risk this lane carries is mis-assignment
  (a stray line read as an answer), which is why the match is on the ask's
  named slot and nothing else.
- **Alternatives:** (b) auto-commit inits lower (0.90, as T2) — then recall
  of a taught answer and a corrected answer score differently for no good
  reason; (c) keep `/correct`-only teaching — the demo's "asks, then learns"
  beat dies and curiosity becomes decorative.

## OD-13 / OPEN-15 — Question-budget parameters are undefined numbers
- **Clash:** the budget is what stops the design from becoming a nagging
  machine, but every parameter of it (asks per owner turn, asks per idle
  pass, queue saturation policy, expiry for stale opens) is a constant with
  no principled value yet — the same attack surface OD-09 flagged for the
  walk.
- **Current doc choice:** **shape only, values parked** (§17.3). v1 default
  = 1 ask per owner turn, dedupe against the open queue, no expiry, idle
  pass may resolve but not emit more than its own budget. Marked **UNMEASURED
  / not-for-quoting** until a sweep artifact exists — and the fragment's own
  rule applies: *"develop, don't invent silently"*, so the annoying-phase
  dial is exposed as a config knob the owner can turn, not a hidden reflex.
- **How to apply:** no gate report, demo line, or doc may cite a budget
  number as chosen until `gates/G8/budget_sweep.json` exists.
- **Alternatives:** (b) hard-set 1/turn now and defend it as "principled" —
  that is exactly the hand-picked-constant failure G2's constants leg
  forbids; (c) no budget in v1 — G8(d) can't be written, curiosity is
  unbounded by construction.

# EMOTIONS chunk rulings (this batch)

## OD-14 — Internality vs the swap test: the walk changed
- **Clash:** §17.6's swap test said a second drive must be addable "without
  touching the walk"; §18.1's internality law says `e` is computed *inside*
  THINK every thought. Both are owner-set. They cannot both hold literally.
- **Current doc choice:** **internality wins; the test is re-scoped.** The walk
  owns an anonymous fixed-shape vector plus two calls into the **drive
  interface** (`drive.update`, `drive.conduction` — see OD-20); it names no
  emotion, holds no affect logic, and never imports the affect module. The
  literal reading is recorded as FAILED (§17.6 verdict table),
  the semantic reading — no emotion names outside `affect.py` + `drive.py`, no
  path in except signals, no path out except `drive.*` — is what G8/G9 enforce.
- **Alternative (b):** keep the walk pristine, run affect as a per-thought pass
  in the drive module. Cost: breaks I8 ("the feeling IS loop state"), turns `e`
  into telemetry observed from beside the loop — the exact shape the DESIGN LAW
  rejects. Pick (b) only if you value the swap test's letter over the organ
  system's premise.

## OD-15 — "Natural" decay vs determinism
- **Clash:** §18.4's decay should feel biological; I5 forbids wall-clock and
  §8.4 already bans clock decay ("event-driven only").
- **Current doc choice:** decay per **logical step `t`**, quantized, inside the
  state hash. "Characteristic time" = decay-per-tick constant. A session that
  receives no inputs feels nothing — silence is not forgetting, it is not
  thinking either.
- **Alternatives:** (b) wall-clock decay — a SI that gets bored because the
  owner stepped away, and replay stops being byte-exact (G5a fails); (c) decay
  only on thought count with a per-resident tick budget — more "alive", more
  constants to sweep. Say which you want; the doc currently ships (a).

## OD-16 — Composition rule at the drive interface
- **SUPERSEDED by OD-21 (round 2 of §18):** the question was "how do six
  residents combine", and there is now **one scalar** — nothing to compose.
  Kept for the audit trail; OPEN-17 is cancelled alongside it.
- **Clash:** §18.3 needs one rule for how six residents combine into budget /
  salience / bias: weighted sum, multiplicative gating, or priority.
- **Current doc choice:** **multiplicative gating with a floor** — residents
  scale each other's channels, and no single resident at zero can mute the loop
  (matches "composed not overriding" and matches §18.5's bias shape, so one
  rule serves both). OPEN-17 carries the constants.
- **Alternatives:** (b) weighted sum — linear, easy to sweep, but a loud
  resident can drown a correct one and zero-vs-zero behavior gets arbitrary;
  (c) priority (caution vetoes everything) — that is the rule-engine costume
  (§18.7) with a nicer name.

## OD-17 — Does a newborn feel CAUTION?
- **CANCELLED by OD-21:** there is no CAUTION and no matrix to mature. §11.1 now
  states that the physics never matures — what develops is the *content* the
  signals point at — and the surviving developmental question moved to the audit
  classifier (OPEN-22).
- **Clash:** §18.7 gives CONFIG SI the **full engine at birth** ("newborn
  emotions real but crude"), but §11 maturation and OPEN-20 ask whether the
  coupling matrix itself should mature — hysteresis, caution's very existence,
  surprise→curiosity gain — like depth perception arriving later.
- **Current doc choice:** full anatomy at birth, **all six residents live**,
  crude because the *levels* start at baseline and every signal this session
  produces is small; **maturation is parked (OPEN-20), not stage-gated.** No
  stage entry/exit criterion was changed — emotions can never promote a stage
  (§11.1), that would be a hidden prior.
- **Alternatives:** (b) unlock residents per stage (caution at S1, boredom at
  S2) — demo-friendly but it makes the tissue a curriculum artifact; (c) fixed
  gains from day one — simplest, and it forfeits the maturation story.

## OD-18 — Aversion is designed but not activated
- **MOOT after OD-21/OD-23:** with no residents there is nothing to avert
  *from*, and the `sign` field is **dropped from the v1 stamp schema** rather
  than defended (§18.5, OD-23). Re-open only if v2 wants real repulsion.
- **Clash:** §18.5's "scared-formed edges bond stronger" language implies
  fear-shaped memory, but §18.5's v1 lean says **attraction/reinforcement only**
  with CAUTION as the paired veto, and §18.7 excludes social emotions.
- **Current doc choice:** the charge field carries `sign` (**form** exists),
  v1 implements **no repulsion dynamics**; "avoid" is modeled as strong
  attraction + caution suppression. No invented aversion physics.
- **Alternatives:** (b) full valence in v1 — needs a source signal for
  *aversive* that isn't just "caution high" (today there isn't one), so it would
  be theater; (c) drop the sign field until v2 — cleaner schema, but then the
  stamp can't express what the owner's law asks it to express.

## OD-19 — Question budget: counter → starvation
- **REVERSED by OD-21 → OD-22:** the starvation curve needed a *curiosity
  level* to read, and round 2 cancelled both the level and any control path for
  it. The budget is a **queue rule + integer ceiling** again (§17.3), and the
  interrogation phase ends because fewer dark spots get named. OD-22 option (b)
  is the route back if the owner wants starvation kept, scoped to `a` alone.
- **Clash:** OD-13 parked budget *parameters*; §18.4 says the interrogation
  phase must end **because curiosity starved, not because a counter hit** — so
  the counter itself was the wrong shape, not just the wrong number.
- **Current doc choice:** emission allowance = quantized monotone function of the
  CURIOSITY level (§17.3), with the integer cap retained **only as a
  determinism/boundedness ceiling**; dedupe and salience floor unchanged. G8(d)
  still passes (3 gaps, low budget ⇒ ≤ budget asks), now because the level is
  low, and the ceiling keeps it decidable.
- **Consequence for OD-13:** its "1 ask per owner turn" default is demoted from
  *the mechanism* to *the ceiling*. The sweep (OPEN-15 + OPEN-16) must now
  produce the coupling curve, not a cap value.

## OD-20 — The DRIVE interface grew from four doors to six
- **STANDS after OD-21, and round 2 is the stronger evidence for it:** the
  residents were cancelled but **no door was removed** — `update` still
  integrates (one scalar now) and `conduction` still returns the stamp bias.
  What round 2 *did* change is the input list behind two older doors, which is
  **OD-22**, not a shrinkage of the contract. The §17.6 verdict table was
  re-run in the removal direction and passed.
- **Clash:** §17.1 defined the contract as `{prioritize, emit, budget,
  salience}` ("at minimum") and §17.6's swap test made that set the boundary a
  new drive may not cross. §18's internality law needs two more: affect must be
  *computed* inside the loop every thought and *read* by the walk's spread —
  which are not effector calls, they are integration and conduction.
- **Current doc choice:** **grow the contract, don't bypass it.** Add
  `update(signals, t)` (anonymous numbers in, next `e` out) and
  `conduction()` (the per-round-normalized bias term that turns `W` into
  `W_eff`). The tissue behind those two doors is unreachable any other way.
  *(as written in round 1: "`affect.py` … the naming privilege is therefore
  exactly two files — `affect.py` + `drive.py`" — **withdrawn by round 2**: the
  module is `arousal.py`, it is wordless, and the naming privilege is
  `audit.py` alone, §18.6/OD-21.)* §17.1's table now documents
  all six with "added by §18" marked on the new pair.
- **Why it is not a silent edit:** "at minimum" already licensed it, so this is
  an enlargement rather than a contradiction — but the swap test's whole value
  was that the door set is *closed and known*, so the owner should nod the new
  pair explicitly instead of inheriting it from a doc that grew.
- **Alternatives:** (b) keep four doors, give the walk a direct `affect` import —
  breaks I8 and the one-way street in §12; (c) keep four doors, run affect as an
  external per-thought pass — that is OD-14(b), the dial-panel reading the DESIGN
  LAW rejects; (d) fold integration into `prioritize` — one door doing two jobs,
  and the per-thought audit line loses its single call site.

## OD-21 — The coupling matrix is cancelled (round 1 of §18 is superseded)
- **Clash:** the IDLE-IGNITION batch's owner law says emotions are **not
  installed machinery** — "the coupling matrix with hand-wired gains is
  CANCELLED; what is innate is only the physics." Round 1 of §18 installed six
  resident integrators, a signed coupling matrix, per-resident
  λ/baselines/hysteresis, a composition rule, and "mood = the slow components".
  Both cannot be in the build.
- **Current doc choice:** the installed affective inventory shrinks to **three
  items** — the raw-signal list, one arousal scalar `a`, and the version-tagged
  modulation curve `a → (reach, decay, τ_settle)` (§18.1). Everything named is
  an **audit readout** with zero causal power (§18.2). The docs keep round 1's
  surviving laws (internality, no privileged sensors, charge on L3, cite-or-
  reject, embeddings ban) and mark the rest **CANCELLED** in §18.9 rather than
  deleting it, so the supersession stays auditable.
- **Consequences this fires (all folded into the docs):** OD-16 (composition
  rule) and OD-17 (newborn residents) and OPEN-17/OPEN-20 are **cancelled** —
  one scalar composes with nothing and has no maturation to schedule; OD-19's
  level-driven budget loses its input → **OD-22**; OD-18's aversion question is
  **moot** (no residents to avert from).
- **Alternatives:** (b) keep the matrix but call it "learned" — it would still
  be a config file with a mood API, and it fails the owner's own test ("a config
  file cannot produce an emotion nobody wrote"); (c) ship emotions as *nothing at
  all*, no labels — cheapest, but then §4 has no trajectory to print and G9 has
  nothing to check. Say `OD-21: keep` to lock the demotion.

## OD-22 — What may read `a`? (budget, salience and ask-damping lose their input) — **write-side half REVERSED by OD-33; read-side + budget half stands**
- **Clash:** §18.1's new law is that **the walk is the only thing arousal
  modulates**. But three round-1 consumers were promised a level: `budget()`
  (OD-19's starvation curve), `salience(event) = f(e)` (§8.4/§6.1), and
  SATISFACTION's per-slot ask damping (§17.3/§6.1). All three would be smuggling
  control back in through the door round 2 just closed.
- **Current doc choice (strict reading):** `budget` = queue rule + integer
  ceiling (§17.3, as it was before §18); `salience` = **f(signal magnitude)**,
  the firing signal's own value (§8.4/§18.5); ask damping deleted — the
  interrogation phase ends because **fewer dark spots get named**, not because
  an urge starved. `a` is retained as **provenance on the ask row**
  (`arousal_at_ask`, §5.4) so the transcript still explains itself.
  **PARTIALLY REVERSED by the §20.4a amendment — see OD-33.** The bake-scale
  chunk makes `salience` a function of `a` at the formation instant, so this
  item's **write-side half no longer stands**; its **read-side and budget halves
  do**: `budget`/`emit` stay level-free, and `magnitude` on a stamp stays the
  signal's own value. The reversal is scoped to one door, one instant, one
  saturating curve.
- **Why this is a ruling, not a cleanup:** it *weakens* the affective story the
  owner liked in round 1 ("asking because curiosity 0.7"). Post-OD-22 the honest
  sentence is "asking because a gap was named and the ceiling allowed it, and `a`
  was 0.7 while the walk found it."
- **Alternatives:** (b) let `a` gate the budget after all — one scalar, one door,
  arguably not a "coupling matrix", genuinely defensible, and **not** what §18.1
  says; if chosen, OD-19's starvation mechanism returns scoped to `a` alone;
  (c) let `a` gate only the *salience floor* for asks (below-floor gaps are
  logged, not emitted).
- **Left open inside this item:** whether `a` may bias **prioritize** (gap
  ranking). Current docs say no — ranking is salience × recency × usage, all row
  quantities.

## OD-23 — Edge stamp field `resident` → `signal` (and the `sign` field)
- **Clash:** §5.4/§6.1/§18.5 stamped `{resident, sign, magnitude, …}` — a
  **resident name stored on the edge**. With residents cancelled there is no such
  name to store, and storing one would put emotion vocabulary into the storage
  layer, which §18.6 forbids everywhere except `audit.py`.
- **Current doc choice:** stamp = `{signal, magnitude, thought_id, signal_ref}`,
  `signal` ∈ §18.1's six kinds. **`sign` is dropped from the v1 schema** (no
  repulsion dynamics exist; OD-18 is moot) and parked for v2 rather than
  defended. Write-side magnitude is the signal's own value, so "the stamp is
  permanent, the boost decays" and "frozen layers cannot hold charge" carry over
  unchanged.
  **Round 3 supersedes the second half of that sentence — see OD-27** (the bake
  is permanent; only the read-side median ages), and adds `a_at` to this very
  schema — see **OD-28**.
- **Alternatives:** (b) keep `sign` as an always-positive field for schema
  stability — wastes a byte and invites someone to invent aversion; (c) store the
  *audit label* alongside the signal for nicer logs — rejected on principle:
  labels are derived, and derived data in the substrate is a second source of
  truth (I4's audit story gets worse, not better).

## OD-24 — CAUTION's role in arbitration is cancelled (Act 5's ASK branch)
- **Clash:** §6.4/§6.1 round 1 explained the ASK branch as CAUTION's effector
  "making `emit` cheaper than `assert`", and §9 justified the demote-only labeler
  as the door that let an emotion "talk SI down". Round 2's control chain has
  three links and **arbitration and labeling are not among them.**
- **Current doc choice:** the ASK branch is §6.4's tier/branch rule (same-tier
  conflict + no marker + first occurrence ⇒ ask, never commit) plus **I7**; the
  labeler is a pure trace function with **no affective argument at all** (§9's
  demote-only law stays as a structural backstop, merely unoccupied). Arousal's
  only legitimate contribution at s20–s22 is that a hot walk spread differently
  and surfaced the row the ask then quotes.
- **What is lost, stated plainly:** round 1's Act 5 read as "it asked *because*
  it was cautious". Post-OD-24 the trace says "it asked because the rule said so,
  and it was aroused when it found the conflict" — less novelish, more honest. If
  the owner wants the causal version, that is option (b) and it re-opens OD-22.
- **Alternatives:** (b) let `a` shift the arbitration threshold (a hot SI asks
  instead of superseding more often) — one number, very demo-friendly, and a
  fourth control link; (c) let `a` only reorder **which** conflicting row the ask
  quotes — presentation, and since presentation is downstream of the walk it may
  already be free.

## OD-25 — The four ignition weights are constants, not a rule
- **Clash:** §19.2 says "this single weighting rule IS rehearsal, worry, and
  intrusive thought", but the rule has **four relative weights** (recency, the
  **edge's bake** — was "hot stamp", renamed by the §20.4a amendment which also
  bounded it at `B_max`, decay deficit, live-contradiction warmth) and no
  measured ratio. Any
  hand-chosen ratio decides what SI daydreams about — exactly the hidden prior
  G5c hunts elsewhere.
- **Current doc choice:** weights are **version-tagged constants**, published
  from a sweep before being quoted (OPEN-25, same discipline as OD-09/OPEN-12),
  with the **starting point = equal weights** so nothing is privileged at birth.
  G10(e) is the leg that measures them.
- **Alternatives:** (b) recency-dominant (freest thought wins) — simplest, reads
  as "distractible"; (c) decay-deficit-dominant — reads as "rehearses what it's
  losing", closest to the consolidation story, but makes boredom-shaped behavior
  the default; (d) weights learned from this session's own cascade outcomes —
  genuinely developmental, and it needs a "cascade paid off" signal that v1 does
  not have.

## OD-26 — Demo trace extended with the idle tail (s25–s27)
- **Clash:** the owner requires the new claims to be *visible in the transcript*
  (§4 must print the curve; the idle batch demands ignition on the record), but
  §4's trace was frozen at s1–s24 and renumbering owner acts would break every
  step-id citation in §5/§6/§9/§13/§17.
- **Current doc choice:** **append Act 6 (s25–s27) after s24**, never insert —
  s1–s24 keep their numbering, every existing citation survives, and the idle
  steps are labelled an extension pending this ruling rather than a rewrite of
  the protocol.
- **Alternatives:** (b) keep §4 pure and put a mini-trace inside §19 — splits the
  one north-star demo the design is organized around; (c) a second session
  (sess-0002) with a quiet tail from the start — cleanest for the gate, costs a
  second scene in G10's setup. **Say which scene G10(e) replays.**

## OD-27 — Bake permanent vs "the boost decays" (round 2's law) — **RESOLVED by the owner (2026-10-23, bake-scale amendment): permanent bake, no decay tail in v1**
- **Resolution:** the owner's amendment closes this item in one sentence —
  *"CLOSES the open item: the trauma bake gets NO decay tail (v1). Permanent
  bake, saturating scale. Only the median moves."* **The current doc choice
  (THE SPLIT) stands as law**, option (b)'s half-life is **refused**, and with it
  option (c)'s two-field version — so **R-23's decaying-write-side concern stays
  closed**. The amendment then adds the constraint nobody had asked for yet: the
  permanent bake is **saturating**, `B_max · tanh(k · a_formation)`, which is
  what makes "permanent" survivable — see **OD-33** for the clash that created.
  `edge.bake` is therefore **write-once, permanent, and bounded**, and G10(b)'s
  bit-identity leg asserts it unchanged.
- **Clash:** round-2 docs state, as a write-side law, that **"the stamp is
  permanent, the boost decays"** — the conduction head start an edge got from
  charge was a decaying quantity. The round-3 chunk says the opposite about the
  same field: *"one genuinely scary moment makes that SPECIFIC edge hot forever,
  even while the word's median drifts calm"* — the bake (head start + confidence
  bump) is durable **at formation** and does not fade. What fades is the word's
  read-side reading.
- **Why this is a ruling, not a cleanup:** the two readings imply different
  state. If the boost decays, `edge.bake` is a transient term; if the bake is
  permanent, a single event is **forever** readable in conduction, and the only
  thing time does to it is make it *less noticeable in the median*, not weaker.
  That is the difference between "SI gets over it" and "SI got over it but still
  won't reach over the flame".
- **Current doc choice (newer owner statement wins):** **THE SPLIT.**
  Median = read-side, per-point, recency-weighted, recomputed, never stored
  (§20.3). Bake = write-side, per-edge, bought at formation, permanent (§20.4,
  §5.4, §8). §18.5's "boost decays" is **superseded**; the decay that remains is
  §8.4's confidence/recency decay and the median's recency weighting, not the
  head start.
- **Alternatives:** (b) bake decays slowly (a half-life) — closer to round-2
  wording, but then "hot forever" is false and §20.4's stove sentence needs
  rewriting; (c) two fields (permanent stamp + decaying boost) — the most
  expressive and the most state; **it also re-opens R-23** (a decaying
  write-side salience field is exactly what G6 was made to guard).
- **Consequence to confirm:** if the permanent reading stands, `edge.bake` is a
  **write-once** field — which makes G10(b)'s bit-identity leg easier to assert
  and puts a value nothing can revise inside the hash. **Confirmed by the
  amendment, and the last clause is now the load-bearing one:** the value nothing
  can revise is bounded by `B_max`, which is the only reason putting it inside the
  hash is safe.

## OD-28 — `a_at` on the stamp (teach-state as part of the address)
- **Clash:** §20.2's claim — *"taught scared + asked scared ⇒ tighter settle"* —
  requires the arousal level **at formation** to be recoverable at query. The
  round-2 stamp schema (§5.4, OD-23) is `{signal, magnitude, thought_id,
  signal_ref}` and carries `arousal_at_ask` only on **ask rows**, not on ordinary
  edge stamps. Without a new field, G11(a) is not runnable at all.
- **Current doc choice:** add **`a_at`** to the edge stamp:
  `{signal, magnitude, a_at, thought_id, signal_ref}`. It is **evidence, never a
  multiplier** — §18.5's σ term reads `a_at` against `a_now` (state against stamp
  history, recency-weighted); nothing scales conduction by `a_at` alone. The
  parity litmus extends: identical `a_at` trajectory on gibberish exchanges
  (§3.4 item 4).
  **The §20.4a amendment gives this field a second job and narrows this
  sentence's scope:** `a_formation` — the argument of the saturating bake curve —
  **is this `a_at`**, so the field is now also what makes a baked edge's size
  reproducible. The narrowing is exact: "never a multiplier" means **the stored
  `a_at` is never re-multiplied later**. The bake is computed **once, at the
  formation instant, from live `a`** and then frozen; from the next round on,
  `a_at` is read only as evidence for the σ term. A row's bake can therefore
  never be re-derived, re-priced, or revised from a later `a` — and `B_max` still
  caps what the one derivation could do.
- **What is at risk, stated plainly:** `a_at` is the only *new field* the
  round-3 batch demands, and the tempting abuse is to treat it as a valence or a
  mood score. It is one number in `[0,1]`, it names no emotion, and it never
  appears on a frozen L1/L2 row (which have no charge field at all — R-23).
- **Alternatives:** (b) derive teach-state at query from the thought id's arousal
  **trajectory** instead of storing it — no schema change, replay-only, but every
  state-dependent read becomes a trajectory lookup and the whole trajectory must
  stay addressable per edge; (c) store the audit *label* instead — rejected on
  OD-23's grounds (derived vocabulary in the substrate).
- **Left open inside this item:** does `a_at` live on **every** stamp or only on
  stamps from content-bearing edges? Current docs put it on every stamp.

## OD-29 — Which logical unit ticks `λ_a`, and what the hash stores
- **Clash:** round 2 integrated arousal **once per thought step**, so `λ_a` had
  an obvious unit and the hash stored one scalar. §20.1 moves the update to
  **per signal event inside the walk**, so the same symbol now means "decay
  between events" — and events arrive at uneven rates, including zero events in a
  round. The chunk does not say which clock the decay ticks against, nor whether
  `a` at NOW is still the hashed quantity.
- **Current doc choice:** `λ_a` is **per logical step** (the deterministic tick
  §16 already uses for everything else), applied inside the walk between events;
  events do not get a private decay. The hash stores the **trajectory across the
  current thought plus `a` at each step**, not a single scalar — otherwise a
  replay could not reproduce *which* token colored what (§16.6). The one-round
  modulation lag is the lag between an event's `a` change and the `W_eff` term
  that uses it, counted in walk rounds, and is itself version-tagged.
- **Why this is a ruling:** it decides whether "the sentence colors itself as it
  is read" is reproducible. A trajectory that is not in the hash is a log line,
  not state, and G5's byte-exact replay would pass silently while the monologue
  replayed differently.
- **Alternatives:** (b) per-event decay (each event multiplies by `λ_a` once) —
  simplest, and makes `a` depend on event *count*, which ties a rate constant to
  content density; (c) continuous integration over rounds with events as
  impulses — most faithful to "the frame finishes its job first", costs a
  solver for a v1 that is trying to be a for-loop.

## OD-30 — The chunk's "G4 clock / G4 extension" is this plan's **G3**
- **Clash:** §5 of the round-3 chunk says stamps age on **"G4's clock"** and calls
  emotional archaeology a **"G4 extension"**. In this plan **G3** is the
  time-axis / interval re-weighting gate and **G4** is contradiction. §4, §8 and
  §13 all cite G3 for the recency curve.
- **Current doc choice:** renumber on the way in. The single recency curve
  (`clock.py`, §12) is the one the chunk means; its leg is **extended inside G3**
  with the emotional-archaeology check, and **G11(d)** cross-cites G3 rather than
  inventing a second time gate. Every "G4's clock" in the chunk merges as G3;
  nothing about the substance changes.
- **Why this is a ruling:** done silently, a future reader comparing chunk to
  docs would conclude an entire claim was dropped, and the risk-register rows
  would point at the wrong gate.
- **Alternatives:** (b) relabel G4 to mean time and renumber the rest — touches
  every citation in the plan for a naming preference; (c) treat the archaeology
  leg as G11(d) only and leave G3 untouched — cheaper, but G3 is where the
  *same-clock* law is actually verified, so the claim would be tested in the one
  gate that cannot see both clocks.

## OD-31 — "Any word can move any point up or down" vs the dropped `sign`
- **Clash:** the round-3 principle uses directional language (move a point **up
  or down**), and §20.3's median reads as a signed-seeming statistic. But OD-23
  **dropped `sign`** because v1 has no repulsion dynamics, and §18.1's inventory
  is **one** arousal scalar — which has no valence axis at all. The design
  currently cannot express "this word pushed that point down" as anything except
  "this word raised `a`, and `a` raised the point's bias magnitude".
- **Current doc choice:** v1 reads "up or down" as **strength, not valence**: `a`
  is one axis; the median is a bias *magnitude* on that axis; a point can be
  modulated more or less, never "negatively", because there is no negative pole
  to be on. §20.3's median therefore lives in `[0,1]` and OD-23's
  `sign`-dropped ruling **stands**. Anything that needs aversion is a second
  scalar, and a second scalar is a real valence axis.
- **Why this is a ruling:** it is the difference between "plasticity" and
  "affect with one dial". If the owner's "up or down" *is* valence, then I8's
  one-scalar inventory is wrong, `arousal.py` grows a second integrator, §18.7's
  costume test must be rewritten, and R-22's runaway guard doubles.
- **Alternatives:** (b) add a **signed** valence scalar alongside `a` — honest
  and expressive; it breaks the "installed items" inventory and re-opens OD-18
  (which OD-23 mooted, not decided on the merits); (c) keep one scalar but let
  the *median* be signed by borrowing signal kind (contradiction negative, gain
  positive) — cheap, and a direct violation of R-21, since it makes signal
  taxonomy do valence's job.
- **Left open inside this item:** whether G11(c)'s "visibly moves the median"
  needs a direction to be observable, or whether magnitude change alone
  satisfies the leg.

## OD-32 — The monster example vs the apple north-star trace
- **Clash:** the round-3 chunk's worked example is a **monster** scene (unknown
  word, fear stamp, later calm history, past-anchored reading). §4's north-star
  trace is the **apple** scene, frozen at s1–s24 plus the OD-26 idle tail, and
  every gate setup cites those step ids. Merging monster into §4 means
  renumbering the demo the plan is organized around; dropping it means ignoring
  the owner's example.
- **Current doc choice:** **annotate, don't rewrite.** §4 keeps the apple scene
  and gains round-3 readings *inside* it (per-event trajectory at s1/s3, the bake
  bought at s7, `a_at` and the near-flat median at s9, the G11(a)-in-miniature
  settle at s9–s11, the past-anchored double reading at s19 — each labelled as
  the shape the new mechanism takes in this scene, magnitudes UNMEASURED). The
  monster scene ships as **G11's own transcript** (§13), where its claims are
  actually tested, and §20.1 keeps the worked example in prose.
- **Why this is a ruling:** it decides what "visible in the transcript" means for
  a mechanism the demo was not designed to exercise. A new mechanism that appears
  only in a gate run is a claim nobody can see; a demo rewritten to show it is a
  demo that no longer tests what G1–G5 assert.
- **Alternatives:** (b) add Act 7 (s28–s31) to §4 with a second word — fits the
  append-only numbering rule, costs the demo a second teaching arc and lengthens
  every gate replay; (c) swap the north star to the monster scene — kills the
  apple/ownership storyline Acts 0–5 are built on; (d) leave §4 untouched —
  rejected: the trace would describe a system that no longer exists.
- **Say which:** whether the annotated §4 is **v3 of the trace** (owner re-reads
  it) or a shape-only annotation accepted without a re-run.

# SALIENCE BAKE SCALE amendment (this batch)

**State of the plasticity ledger after the fold: OD-27 RESOLVED (permanent bake,
no decay tail), OD-22's write-side half reversed, OD-32 and the OPEN-32 question
CLOSED by naming (ignition reads the bake), and OD-33 OPEN — the only genuinely
unruled item the amendment leaves.**

## OD-33 — The bake scale is `f(a_formation)`: the write side reads `a` again
- **Clash:** the amendment's first line is a function signature — *"bake = B_max
  \* tanh(k \* a_formation)"* — and `a_formation` is the arousal **level**. Round 2
  ruled the exact opposite, by owner decision, in **OD-22**: `salience(event) =
  f(signal magnitude)`, **not** `f(a)`, because §18.1 said *the walk is the only
  thing arousal modulates*. That ruling is not a stray sentence — it is load-
  bearing in six files (§8.4, §6.1, §17's `salience` door, §18.1's table, §12's
  drive row, §3's R15/R16 rows), and it was bought deliberately at the cost of
  round 1's "asking because curiosity 0.7" story. The amendment spends that cost
  back.
- **Why this is a ruling, not a cleanup:** three different things look alike here.
  A **provenance record** of `a` (OD-28's `a_at` — evidence, keeps the transcript
  honest); a **read-side** use of `a` (§18.5's σ term against `a_now`, already
  legal, already capped); and a **write-side magnitude derived from `a`**, which
  is new, permanent, and stored in the hash. Only the third is OD-22's subject.
  Left implicit, a reader would conclude OD-22 was quietly deleted; the truth is
  that one door changed state.
- **Current doc choice (newer owner statement wins, scoped as tightly as the
  sentence allows):** the write side may consult `a` **once per edge, at the
  instant that edge forms, through one saturating curve**. Everything else in
  OD-22 stands — `magnitude` on a stamp stays the firing signal's own value,
  `budget` and `emit` stay level-free, arbitration takes no affective input
  (OD-24), and no stored row is ever re-priced from a later `a`. The saturation
  is not decoration: it is the **first and only write-side bound** in the plan
  (§18.5's per-round cap and §20.3's median are both read-side), and it is what
  makes a *permanent* bake tolerable — "the first kitchen fire and the tenth both
  score near the cap; nothing ever scores an 11 that owns the walk." Anti-cheat
  holds because the curve is monotone in one integrated number and **consults no
  event-kind table**: nothing categorizes fires as bad, the impact is emergent
  from intensity at the moment (OD-31's evidence, since "it felt like a lot" is a
  magnitude, not a valence).
- **The wrinkle the chunk did not address:** `a` is a **leaky integrator**, so
  `a_formation` carries **residue** from signals earlier in the walk and the
  round before it. A calm edge formed seconds (logically) after a storm therefore
  bakes hot — which is either the right answer ("trauma bakes in context, that is
  the stove story") or a way for one event's arousal to be paid out twice: once
  as its own stamp, again as the head start on whatever happened to form next.
  Both readings are defensible and they are **not the same mechanism**; the
  current docs take the integrated reading because the amendment says "live
  arousal at formation" and §20.1 already made `a` per-event-live, but the
  ambiguity is owner's to settle, not the fold's.
- **Alternatives:** (b) **strict-local `a_formation`** = only the gains of the
  signals belonging to *this* event, summed before decay — keeps OD-22's letter
  ("the event's own intensity") almost intact, removes the double-payment, and
  costs the residue story; (c) keep `f(signal)` and give the bake a **capped
  function of summed signal magnitudes** instead of `a` — closest to round 2's
  law, but it contradicts the amendment's explicit wording and loses state-
  congruence at formation; (d) `B_max` fixed small, or the curve linear — refused:
  the owner asked for a soft cap, and a linear bake with a permanent tail is
  precisely the unbeatable-edge bug the amendment names.
- **Consequence to confirm:** if OD-33 stands as written, **§8.4's "salience is
  not a function of `a`" sentence is deleted**, `drive.salience` becomes the only
  door with an affective *and* durable output, `arousal.py` gains two constants
  (**OPEN-33**), and G9/G10 each gain a bake-bound leg. If option (b) is chosen
  instead, `a_formation` needs its own definition in §18.1's table and §20.4a's
  `a_at = a_formation` identity **breaks** (the stamp would carry the integrated
  level while the bake used the local sum) — so pick (b) only if you also want a
  sixth stamp field.
- **Say which:** keep OD-33 / take (b) / take (c). This item, **not** OD-27, is
  the one the amendment left genuinely open.

# GRIMM: THE SUPPORTER (this batch)

**State of the ledger after the fold: seven new items, OD-34…OD-40, of which six
are clashes against owner-decided cores and one (OD-40) is four calls made on the
owner's behalf because the chunk delegated them. Nothing in this batch is RESOLVED
by the owner. The previous batch's genuinely open item — OD-33 — is untouched by
Grimm, and so is OD-31's leftover direction-of-motion question.**

## OD-34 — "Nova" vs the name SI, and the chunk's "five organs"
- **Clash:** the chunk names the baby **Nova** and says Grimm folds "NOT into the
  five organs". Both hit owner-decided cores: the naming law (README rules) makes
  the system **SI** everywhere with newborn framing only in docs/11, and docs/02
  fixes the organ count at **six** (THINK / WHAT / KNOW / LEARN / CALIBRATE /
  DECODE-RENDER).
- **Current doc choice:** every normative sentence writes **SI**; "Nova" survives
  only inside the quoted owner passages of §21.0 and §21.6, marked as the owner's
  word for his baby. "five organs" is read as the chunk's shorthand for the
  knowing organs and renumbers nothing — docs/21's opening line states that
  nothing in that file adds a seventh anything, and Grimm in fact adds no organ
  at all. Separately: **a name SI has is content.** If SI answers "Nova", it came
  through the channel; it cannot be installed in the genome (I1, I2).
- **Why this is a ruling:** silently renaming would put a proper noun with
  pretrained flavour into the birth state, and G5c's constant-and-handle diff
  would then have to carry it; silently "correcting" five→six would hide that this
  batch was scoped against a different organ count than the design has, which is
  exactly the comparison a future reader needs.
- **Alternatives:** (b) rename throughout — touches every doc, the naming law and
  the memory files; (c) keep Nova as the name the owner *types* from turn 1 —
  legal, and this is what happens anyway if he does; it is content SI learns, not
  a constant.
- **Say which:** keep SI in the design and let "Nova" arrive through the channel
  / rename to Nova / both (docs say SI, you type Nova).

## OD-35 — I3 amended: one channel, more than one voice
- **Clash:** I3's owner-decided sentence ("every line is SENSE — a perturbation to
  ongoing thought, never a command that owns the loop") was written when every
  line had one author: the owner's typing. Grimm's lines are sense too, from a
  second voice on the same wire.
- **Current doc choice:** the invariant's **sentence stays verbatim**; the
  amendment adds a clause after it — the channel admits more than one voice, each
  line carries `source: owner | grimm` assigned at the seam and never inferred
  from text, and **the source decides what a line may do, not whether it may
  arrive**. The enforcement is §21.4's privilege table, not a new law about
  input.
- **Why this is a ruling:** I3 is the load-bearing law that keeps an LM out of the
  loop. Widening "line" without widening the law is precisely how a supporter
  becomes a controller; and the batch turns on the distinction between "one
  channel" (still true) and "one author" (no longer true), which every G12 leg
  cites.
- **Alternatives:** (b) put the source rule in I10 only — I10 exists and is about
  Grimm, but leaving I3 unqualified keeps it readable as "all lines are the
  owner's", which is now false; (c) treat Grimm's input as owner input and amend
  nothing — that is R-32's laundering path with the law's own blessing; (d) move
  the clause *into* I3's sentence, rewriting an owner-decided core rather than
  appending to it.
- **Say which:** accept docs/00's appended clause / fold it into I3's own sentence
  / refuse and un-design the supporter.

## OD-36 — May a Grimm line form an edge? L3's literal wording vs §21.4
- **Clash:** the chunk's L3 says his calm talk "**forms edges stamped with calm
  state**"; its §5 says his emissions are "**REHEARSAL, not teaching**". In this
  design those are different mechanisms, not different phrasings: edge formation
  is the write path's teaching event (§6), and §20.4 rules that **only a teaching
  event buys a bake**.
- **Current doc choice:** **sense-only.** A Grimm line is parsed and decoded, may
  light existing rows and raise signals, and does **not** form an edge, buy a
  bake, bump `repeats`, or match a pending ask (§21.4, docs/06, docs/08 §8.3 and
  §8.4). Co-regulation reaches the walk — his quiet, repetitive shape genuinely
  moves `a` (§21.5) — not the lattice. The chunk's wording is kept as a quote with
  this substitution noted.
- **Why this is a ruling:** if a Grimm-formed edge is legal, provenance traces to a
  pretrained model's sentence selection, I2's clean-room law has an exception
  nobody voted for, and §20.4 needs a second exception right after OD-27 closed
  the first. "A parent's calm becomes the baby's calm" stops being physics and
  becomes content injection wearing a nice story.
- **Alternatives:** (b) allow edge formation tagged `provenance.speaker: grimm` —
  honest about what happened, and it instantly makes the lattice's least
  trustworthy shelf its most populated one; (c) allow edges only between words the
  owner already taught — §21.8(1)'s recombination door, same laundering risk, more
  code; (d) keep L3's wording literally and let the write path decide later —
  refused: the docs would then contradict each other on the most audited seam.
- **Say which:** keep sense-only / take (b), which reopens I2 / take (c).

## OD-37 — "No LM anywhere in v1" vs a locally hosted LM in the house
- **Clash:** docs/12's headline law and the README's v1 promise say **no LM
  anywhere**; §21's supporter is a real, locally hosted, fine-tuned LM running an
  agentic loop.
- **Current doc choice:** restate the law as **no LM inside `src/`**. Grimm lives
  in `grimm/`, is imported by nothing in the build (grep rule (v)), and contributes
  zero state and zero hash inputs. The LM-freedom claim is about the **mind**, not
  about the room; every gate passes with his process not started (§10.4, G12(a)).
- **Why this is a ruling:** the blunt sentence was the anti-cheat law's easy form.
  Standing next to §21 it is simply false, and a false law in a design doc teaches
  the reader to distrust the true ones. Scoping it is also what makes G12
  falsifiable: the proposition under test is "the build contains no LM", which is
  greppable, not "nothing language-model-shaped was ever switched on", which is
  not.
- **Alternatives:** (b) refuse Grimm in v1, ship §21 as a v2 addendum — cleanest
  for the law, ignores the owner's co-regulation premise; (c) keep the sentence and
  add an "except" inline — a law carrying its own exception clause is how R-21's
  costumes walk in; (d) say the fine-tune is "just data" — refused: it is weights.
- **Say which:** accept the re-scope / Grimm waits for v2 / keep the old sentence
  and mean it (which deletes §21).

## OD-38 — The lab notebook vs OPEN-23, and whether `read_monologue` exists
- **Clash:** the chunk's §2 hands Grimm `read_monologue()`; its §5 surface list
  drops it. Behind both sits **OPEN-23**, the standing question of whether SI's
  idle monologue (§19.3) may ever be *shown*, while I9 keeps idle thought read-only
  and non-teaching. A supporter who reads the monologue and writes a notebook the
  owner reads is a route from private thought into the conversation that does not
  pass through the response channel.
- **Current doc choice:** expose `read_monologue` **read-only and logged**, with
  every read recorded; the notebook is a **Grimm-side, owner-facing document of
  citations** — `gates/*/monologue.jsonl` already exists as an artifact, so the
  notebook is a *legibility* change, not a *reachability* change. It never enters a
  shelf and nothing in the build reads it back. The chunk's §5 omission is
  recorded rather than harmonized silently.
- **Why this is a ruling:** this is R-33's exact shape — a derived, human-sounding
  summary of a mind's private thought becoming an input to that mind. The legal
  path stays legal only while the *person* is the one who decides to paste it back;
  the moment a file like that is readable by SI, I9 is dead and the monologue has
  an audience inside the system it was designed to have none in.
- **Alternatives:** (b) no `read_monologue` in v1 — Grimm sees traces and state
  only, the notebook loses its best entries ("she returned to the kitchen-fire
  region at 3am"), OPEN-23 stays untouched; (c) notebook internal-only, owner never
  sees it — then it is a log, not a parent's observation, and the chunk's
  flagship v1 output never reaches the person doing the parenting; (d) rule
  OPEN-23 now and let the notebook be its answer.
- **Say which:** keep / (b) / (c) — and separately: does OPEN-23 get ruled with
  this batch or stay open?

## OD-39 — The MCP surface the chunk assumes was never designed
- **Clash:** the chunk refers to "the MCP surface (already drafted for the
  parent-agent idea)" as though it were in the plan. A grep of every doc found
  **zero** references to MCP, `read_trace`, or any tool surface before this batch.
- **Current doc choice:** §21.7 drafts it as **SPEC only** — `mcp.py` appears in
  the component map as "the second finger on the same channel", holding no handle
  on lattice, walk, drive or arousal; `sense_input` stays the single entry point
  for content; the surface becomes real when the build does. Nothing was claimed
  retroactively.
- **Why this is a ruling:** the supporter's entire safety case is that his world is
  exactly what the surface exposes. If that surface is an assumption imported from
  a document nobody wrote, G12(d)'s "no setter" leg has no scope to prove over —
  you cannot grep for a hole in an interface that exists only in a chunk's prose.
- **Alternatives:** (b) declare the sense seam plus the existing logs to *be* the
  surface and refuse `mcp.py` — cheaper, but then `read_state`, `read_vocab` and
  `read_rows` need another home; (c) ask the owner for the earlier parent-agent doc
  and merge with it rather than re-deriving it — if one exists, this batch may be
  duplicating decisions already made.
- **Say which:** keep the SPEC / take (b) / hand mike the parent-agent doc so this
  merges instead of re-derives.

## OD-40 — Four calls made on the owner's behalf, recorded as reversible
- **Clash (of a different kind):** the chunk explicitly delegates item (i) —
  "string match vs row-id reference — **Mike's call**" — and leaves (ii), (iii),
  (iv) as open questions. A design that cannot be built because four interface
  details are unruled is not finished, so all four are ruled here, each scoped so
  that reversing it touches prose but no law.
- **Current doc choice:**
  **(i)** layer 2 = **strict single-row restatement, row-id-referenced** — the
  emission must name a row that exists, and rejection returns a machine-readable
  reason `{n_untaught_tokens, novel_claims, ref_row_id}` (§21.3, G12(b)).
  Strict first, because a relaxed matcher is §21.8(1)'s recombination risk opened
  by the same hand that wrote the filter; loosening later is measurable against
  `G12/rejection_reasons.json`, tightening after a leak is not.
  **(ii)** cadence = **one summary per quiet tick**, aligned with the idle budget
  (§19.4) — *he is slow, not chatty*. His cadence is the second knob that could
  turn co-regulation into pressure, and reusing a budget SI already has means one
  constant rather than two, already covered by G10(a)'s rate bound.
  **(iii)** the notebook is **owner-readable while Grimm's own scratch never
  enters SI's logs** — the owner needs the observations; SI must never hold a file
  that reads like an authority about itself.
  **(iv)** the v2 curriculum question (Grimm-side vs owner-scripted) is
  **deferred**, and it can wait because stages advance on row counts over the
  owner's teaching (§11) and §11.2 forbids Grimm from moving them — so a Grimm-side
  curriculum is, for now, only a way to ask better questions.
- **Why this is a ruling rather than a to-do:** each of the four silently decides
  something the gates test. (i) *is* G12(b)'s pass criterion; (ii) is what makes
  G12's cadence note and §19.4's budget one mechanism; (iii) is the boundary R-33
  patrols; (iv) is the only one with no gate behind it, which is why it is the only
  one that can stay open.
- **Alternatives:** (i-b) fuzzy/normalized string matching — more usable, strictly
  less safe, and it moves the judgment from a table lookup into code that needs its
  own gate; (ii-b) every walk — a chatty parent, and the failure mode the chunk's
  own "Boring is correct" line warns about; (iii-b) notebook internal-only — see
  OD-38(c); (iv-b) rule the curriculum now — premature while §11's law holds.
- **Consequence to confirm:** none of the four adds a field, a door, a constant
  class or a gate leg. If the owner reverses (i) to fuzzy matching, only §21.3
  changes and G12(b) gains a "why did this pass?" leg — not a thirteenth gate.
- **Say which:** accept all four / (i) fuzzy / (ii) its own budget / rule (iv) now.

---

# Engineering-pass items — OD-41 … OD-85

Everything from here on was raised while converting the frozen design into
[`engineering/`](../../engineering/README.md), and each item names the engineering
file that holds the HOW. **No `docs/` law was changed to make room for any of
them**, which is the point of writing them down: where the design under-decides,
the build records the gap, implements a lean, and waits.

Format note: these entries are one line shorter than OD-01…OD-40 because the
detail lives in the engineering folder — the **Clash** is what the design left
open, the **Build choice** is what ships until you rule, and the **Say which**
line is the reversible knob.

## OD-41 — `/teach` gets owner-tier confidence but second-tier arbitration rank
- **Clash:** the TEACH fragment says an owner-asserted commit inits at owner tier
  (0.95); docs/06 §6.4 says an asserted row never outranks a demonstrated
  correction. Both readings are in the design and they disagree about one row.
- **Build choice:** ship the split exactly as written — init **0.95**, rank **T2**
  (docs/08 §8.4, docs/06 §6.4, engineering/09). The split *is* the ruling, not a bug.
- **Alternatives:** (b) owner tier in both columns, which lets a claim outrank a
  correction and quietly inverts §6.4; (c) second tier in both, which makes `/teach`
  a slower way to say what `read` already learned.
- **Say which:** keep the split / (b) / (c).

## OD-42 — "pure stdlib, zero dependencies" (docs/12) against the Rust+Vulkan stack law
- **Clash:** docs/12's component map says pure-stdlib; the stack law says Rust +
  Vulkan + a GUI day one. `ash` is not stdlib.
- **Build choice:** restate the law as **no dependency may contain pretrained
  content, a lexicon, or a corpus**; publish a byte-audited allowlist whose
  `Cargo.lock` hash is inside G5c (engineering/00 §9).
- **Alternatives:** (b) literal zero-deps — then there is no Vulkan and no panel;
  (c) any crate allowed — then R-14's scan has nothing to scan.
- **Say which:** keep the restatement / (b) / (c). If (a) stands, the allowlist is
  a document you own, not a build detail.

## OD-43 — docs/12 names `*.py` modules in a Rust build
- **Clash:** the design's component map is a Python file list; the stack is Rust.
- **Build choice:** keep the names as **logical module ids** and ship Rust crates
  with a one-to-one mapping table (engineering/00 §2). `consolidate.py` ⇄
  `si-sleep`, etc.
- **Alternatives:** rename to crate-idiomatic names — cheap, but it breaks every
  `docs/` cross-reference and makes the trace §4 unreadable beside the build.
- **Say which:** keep the mapping table / rename (and accept the doc edits).

## OD-44 — the v1 CPU walker (§16.6) against "Rust + Vulkan, GUI day one"
- **Clash:** docs/16 fixes the deterministic walker on CPU fixed-point; the stack
  law loads Vulkan from day one. Two answers to "what runs the walk".
- **Build choice:** **both.** The hashed knowing loop walks on CPU fixed-point (the
  determinism oracle); Vulkan is loaded for genome residency and for the viewer
  rendering the walker's own buffers (engineering/00 §4, 07 §9, 08 §5).
- **Alternatives:** (b) GPU walk in v1 — buys speed, costs byte-exact replay and
  every gate's authority; (c) no Vulkan until v2 — costs the panel's zero-copy
  property, which is what makes ENGINE laptop-runnable.
- **Say which:** keep both / (b) / (c).

## OD-45 — the `norm` operator in `h_k = decay · norm(h + W_eff·act)`
- **Clash:** the design writes `norm(·)` and never says divide by what.
- **Build choice:** per-round **max-normalize via a reciprocal multiply** on the
  fixed-point grid — one shared shift, never a division (engineering/01 §3, 04 §5).
  LoopNet's `INV_GROUP_MAX = 1.0/7.0` idiom.
- **Alternatives:** (b) true division with rounding — a second rounding table to
  audit and a byte-exactness risk; (c) sum-normalize — changes the cap semantics of
  §18.5, so it is a law question rather than a HOW question.
- **Say which:** keep max-normalize / (b) / (c → needs a design ruling first).

## OD-46 — quantization grids were named per field but never collected
- **Clash:** each `docs/` page picks its own units; nothing said the whole set.
- **Build choice:** one table — activation `i32` Q12 · arousal `u8` (1/255) ·
  confidence `u16` hundredths · bake `u16` Q12 · stamp magnitude `u16` Q12 ·
  genome weight `u8` (engineering/00 §6, 01 §3, 03 §1). Conversion between the two
  weight grids is a shift, per OD-45's discipline.
- **Alternatives:** float storage for one field — reopens §16.6's determinism annex.
- **Say which:** keep the grid table (this is a lock, not a preference: a change
  here re-versions every artifact).

## OD-47 — GUI toolkit
- **Clash:** the job says "Vulkan GUI day one"; the reference engine has **zero**
  windowing code, so nothing can be reused.
- **Build choice:** `ash` swapchain + `winit` input + bespoke immediate-mode panels.
  **No** `wgpu`, **no** retained-mode framework, **no** egui (engineering/08 §5.3).
- **Alternatives:** (b) egui/`wgpu` — faster to write, a second rendering stack, and
  a dependency family the allowlist audit has to grow around; (c) terminal panels —
  cheaper, but "renders the buffers the walker computed" dies with it.
- **Say which:** keep (a) / (b) / (c).

## OD-48 — loop ↔ viewer transport, and how a keystroke reaches a running session
- **Clash:** the design never says how a panel talks to a loop that may not wait.
- **Build choice:** two append-only framed lanes — `runs/<session>/stream.siro`
  (loop writes, viewer tails) and `sense_inbox.siro` (viewer/CLI appends, loop
  consumes); **the loop never blocks on a reader**, and there is no request/reply
  (engineering/00 §7, 08 §2).
- **Alternatives:** (b) a socket — a second input path, so I3's single channel becomes
  a policy instead of a topology; (c) shared memory ring — OD-66's question.
- **Say which:** keep the two lanes / (b) / (c).

## OD-49 — hash function for both integrity and state
- **Clash:** docs name "a deterministic fingerprint" without an algorithm.
- **Build choice:** `md-5` for both, and the `t=0` genome tag is the same id —
  documented as **identity, not security** (engineering/00 §5, 01 §10).
- **Alternatives:** (b) blake3 — faster, an extra dependency for no legal gain;
  (c) a cryptographic signature anywhere — out of scope for v1 and would need a
  key, which is a new kind of state.
- **Say which:** keep md5-labeled-as-identity / (b).

## OD-50 — concurrency, and where it may live
- **Clash:** the design is silent on threads; a single-threaded loop is a claim
  nobody wrote down.
- **Build choice:** `rayon` **only** in the genome-build and export/index passes. No
  crate on the hashed path may depend on it, enforced by the crate graph
  (engineering/00 §2, §9).
- **Alternatives:** (b) parallel walk — a fixed iteration order is law (§16.6), and
  a parallel frontier is a data-dependent order; (c) no `rayon` anywhere — the
  converter then takes days instead of hours, which is a budget question.
- **Say which:** keep the split / (b) (needs a design ruling) / (c).

## OD-51 — id representation and L3 storage
- **Clash:** docs speak of `r1`, `q1`, `t_apple` display ids and of JSON artifacts.
- **Build choice:** `u32` handles everywhere; display ids derived **at render**;
  L3 rows in packed binary plus a string interner; JSONC as export only
  (engineering/01 §1, §9).
- **Alternatives:** (b) string keys in records — a row's size stops being a constant
  and every §6 size formula in the budget dies; (c) JSON as the store — readable,
  and no byte-exact replay.
- **Say which:** keep (a) / (b) / (c).

## OD-52 — genome source versions, and the LLM-assisted expansion
- **Clash:** docs/10 lists corpus sources without pinning them, and the job's §7
  expansion question is still **PENDING**.
- **Build choice:** sources pinned by URL + sha256 in a manifest lock; expansion
  stays PENDING and the plan is complete without it, naming the one sealed
  insertion point where it would attach (engineering/07 §2, §16).
- **Say which:** keep the pinning / rule the expansion item (it is OD-52's second
  half, and it is the only one with a v1 consequence: an expansion changes the
  sealed `t=0` tag and therefore every resume).

## OD-53 — where the engineering-only checks live
- **Clash:** three properties are buildable but not design-law (headless with the
  GUI killed, observer-only, no viewer state write), and docs/13's gate list is the
  design's.
- **Build choice:** they ride the harness as `check_*` legs of existing gates
  (engineering/05 §20), **not** as a new numbered gate.
- **Note on the wording:** this item was written when the list ended at G12 and said
  "not a thirteenth gate". **G13 SLEEP and G14 TEMPERAMENT now exist** — they came
  from the owner's own batch, not from this one — so OD-53's claim is now the
  narrower one: the `check_*` family does not get promoted just because the
  numbering has room. Nothing about the three properties changed.
- **Say which:** keep them as `check_*` / promote one to a numbered gate.

## OD-54 — a multi-line paste at the CHAT panel
- **Clash:** docs treat "an input" as one line; a paste is many.
- **Build choice:** *k* lines are *k* logical steps **in order**, each its own
  `context` record and its own sense event, `t` +1 per line (engineering/01 §12).
- **Alternatives:** (b) one step for the paste — collapses a real ordering the walk
  depends on and changes the arbitration outcome; (c) drop the paste path.
- **Say which:** keep (a) / (b).

## OD-55 — plural-fold alias lighting at round 0
- **Clash:** §2's fold lookup can match several members of one alias set; the design
  says nothing about which lights.
- **Build choice:** light **every** alias member with the token's activation, split
  by count, and log the alias set in the mind log (engineering/04 §2).
- **Alternatives:** (b) light the first — arbitrary, and it makes a fold's row order
  observable in behavior; (c) light none until disambiguated — a gap detector's job.
- **Say which:** keep (a) / (b) / (c).

## OD-56 — `bake` when the forming `a` is at rest (0)
- **Clash:** §20.4a's curve `B_max·tanh(k·a_formation)` gives 0 at rest, and a 0
  could be read as "no bake", as "missing", or as "unbaked".
- **Build choice:** `bake = 0` **is** unbaked, and the edge keeps §8.4's plain init —
  the floor is law, not tuning (engineering/01 §5.3, 03 §4).
- **Alternatives:** (b) a non-zero floor for rest-formed edges — that is a second
  unbounded durable affective quantity, which §5.3(e) forbids.
- **Say which:** keep (a) / (b) (needs a design ruling first).

## OD-57 — the weighted-median rank convention
- **Clash:** "the middle of the stamps" is undefined at the exact half-weight
  crossing.
- **Build choice:** lean **lower** element — the conservative reading of "no word
  flips on one comment"; version-tagged, never a literal (engineering/04 §9).
- **Say which:** keep lower / upper. Either is lawful; the tag must move if you flip
  it, because a re-read of old runs would otherwise shift every median.

## OD-58 — does a stably re-derived inference become a bridge **candidate row** in v1?
- **Clash:** the COMPOSITION chunk asks for candidate accumulation; I9 says an idle
  walk may notice, never learn. The chunk's own line is "nothing unowned ever
  sticks".
- **Build choice:** **ruling PENDING, not assumed.** The row shape,
  `INFERENCE_STABLE_N` and the gate legs are specified now, and
  `INFERENCE_PROMOTION` ships **off** (engineering/03 §2.1, 04 §5.4/§21 U.4,
  05 G11(e)–(i)/(p)). Its sub-question — does a genome edge consume owned-hop budget
  on a mixed route? — is answered **no** in the same spec, and that answer is part of
  what you are ruling.
- **Say which:** promote in v1 / park to v2 (the build runs either way; parked is
  what ships today).

## OD-59 — `prov_speaker` gains `2 system-inferred`
- **Clash:** docs/07 §7.3's mechanism 2 already *writes* system-inferred bridges, and
  the encoded schema could not hold them. The widening itself is mechanical.
- **Build choice:** the field is widened; **the scope sentence is the ruling** — does
  `2` apply to belief rows, to `links` rows, or to both? Unifying them breaks either
  clean-room provenance or the bridge mechanism (engineering/01 §4.1, 04 §21 U.4).
- **Say which:** links only / beliefs only / both (and say which law you are
  trading).

## OD-60 — the state-hash **construction**
- **Clash:** the design fixes *what* is hashed and *in what order*; it never says
  whole-per-tick or chained-per-shelf.
- **Build choice:** lean **chained sub-digests**, fixed once before the first G5 run,
  version-tagged as `HASH_FORMAT` so an old `state_hash.txt` cannot be replayed under
  a new construction without saying so (engineering/06 §7, §13).
- **Consequence worth knowing before you rule:** chained construction is what makes
  sleep affordable, and it is also what makes sleep *loud* — an in-place FLUID write
  invalidates its shelf's sub-digest from the write offset to the shelf end, so P3's
  rescale pays a full edge-class re-digest per sleep. A dirty-bit "fix" is forbidden
  because it would change the hash's covering set.
- **Say which:** keep chained / whole-per-tick (slower per tick, simpler proof).

## OD-61 — the bound on the **ignition candidate pool**
- **Clash:** §19.2's four weights are per-edge and per-point; the pool they rank is
  unnamed, and an unbounded pool makes idle cost grow with the session — against the
  design's own position that idle thought is rate-bound, not size-bound.
- **Build choice:** lean **the rehearsal stream's newest window** (the edges
  `Clock::recency` still scores above zero), so the pool shrinks by the same curve
  everything else uses and needs no second decay law (R-29). Size swept into
  `G10/ignition_sweep.json` (engineering/06 §13).
- **Say which:** keep the recency window / a fixed-size newest-N (a second constant).

## OD-62 — how "relation names **verbatim**" survives a closed `u8` kind enum
- **Clash:** docs/10 requires source relation names verbatim; the substrate's edge
  kind is one byte.
- **Build choice:** a sealed, version-tagged **genome kind table** — 256 slots
  holding the source's own strings, **listed not mapped**, and over-capacity is a
  hard fail (engineering/07 §6).
- **Alternatives:** (b) nine hand-picked relations — that is the relabeling docs/10
  forbids; (c) a string column per edge — breaks the fixed record width and every
  size in the budget.
- **Say which:** keep the kind table / (b) / (c). If a corpus ever needs more than
  256 distinct names, that is a re-version of the enum, not an extension.

## OD-63 — who consumes **`RULE` rows**, and may they conduct?
- **Clash:** the schema reserves a rule shelf, but a rule is n-ary and the walk is
  binary.
- **Build choice:** **parked in v1** — sealed, loaded, citable, not walkable.
  Collapsing a rule to pairwise edges changes its meaning, and doing it faithfully
  needs a structure the substrate does not have, which is R-12's alarm
  (engineering/07 §7).
- **Say which:** keep parked / allow pairwise decomposition (and accept the meaning
  loss in writing).

## OD-64 — the **grammar property tables** have no author, format or licence in the design
- **Clash:** docs/10 names grammar sources; nothing says who made them or under what
  rights, and R-10 is a license law.
- **Build choice:** source **S3** — hand-maintained TSV in-repo under `data/grammar/`,
  owner-licensed, pinned by sha256, joining the pipeline through the same five
  adapter contracts as every other source (engineering/07 §2).
- **Say which:** keep S3 as authored-by-you / name an external source and its licence.

## OD-65 — may a genome be **pruned or subset** for the load budget?
- **Clash:** §10.3 requires CONFIG ENGINE to stay laptop-runnable; §8's arithmetic
  says the full seal may not fit.
- **Build choice:** **no** — the default is a full seal. A cut is a content decision
  (R-10) and an attribution loss; if laptop-runnability ever forces one, the **owner**
  names the batch, never a scoring function in the converter (engineering/07 §12).
- **Say which:** keep the full seal / pre-author one named subset now (that is the
  honest way to do it, and it is a sentence from you, not from a ranker).

## OD-66 — does "renders the same buffers the walker computed" mean a **shared live buffer**?
- **Clash:** the job's third GUI law names a property ("never a re-simulation") that
  can be read as a memory-topology claim or as a data-provenance claim.
- **Build choice:** **frame-driven.** The viewer draws the walker's own per-round
  numbers off the lane the loop already writes (engineering/08 §1, §11). A shared ring
  would add a per-round coupling and a second sync point between your screen and the
  knowing loop, and the v1 zero-sync structure is the more valuable of the two.
- **Say which:** keep frame-driven / require a shared live buffer (buys latency,
  costs §6's sync budget).

## OD-67 — may you trigger a **counterfactual probe** from the GUI?
- **Clash:** "re-run this walk minus one edge, on click" is the most natural thing a
  panel could offer, and it is a door.
- **Build choice:** **no in v1** — probes stay in the harness (engineering/08 §9,
  kind 18 reserved and unemitted). If wanted, the clean route is a sense-channel
  command, which means a new born-in reflex **and** a new G5c allowlist line: a
  design change with a UI on top, to be decided as that.
- **Say which:** keep refused / want it (then rule the reflex first).

## OD-68 — what does G5c's byte-exact string diff scope to, now that a GUI exists?
- **Clash:** docs/03 §3.4's test 1 predates a second binary; a GUI's chrome strings
  would drown the diff.
- **Build choice:** the diff scopes to the **loop binary**; `si-vulkan`'s chrome
  strings are audited by reachability (no crate edge, one writable file), plus a
  separate assert that the viewer contains no label word, no emotion word, no
  temperament adjective and no content list (engineering/08 §3.2).
- **Say which:** keep the split scope / diff everything including the viewer.

## OD-69 — is the bulk **`/teach` file feeder** in v1 at all?
- **Clash:** a curriculum file is the cheapest way to teach 400 rows and the easiest
  way to build a loader, which I2 forbids.
- **Build choice:** **yes, as an inbox writer only**, licensed by `G1.u.2`: the fed
  file and the typed lines must give **byte-identical `state_hash.txt`** and an
  identical transcript (engineering/09 §8, 05 G1). A feeder that cannot produce a
  different state than typing is a convenience; one that can is a loader.
- **Alternatives:** (b) no feeder, you type — coherent, and slower by a factor you
  will feel.
- **Say which:** keep (a) / (b).

## OD-70 — may the feeder interleave **quiet ticks** between curriculum lines?
- **Clash:** a bulk import is one hot cohort for the bake curve, and §20.4a says a
  formation `a` that never decays is a permanent property of the rows.
- **Build choice:** **off by default.** It is legal in every respect — pure logical
  time, existing idle precedence — but it changes the tick structure and therefore
  the hash, and quiet ticks with an empty agenda **ignite**: an import with settle
  gaps also grows a monologue about the rows it just ate, which is a behavior change
  wearing a cosmetic one (engineering/09 §7).
- **Say which:** keep off / on as a scenario-hash flag (never a build default).

## OD-71 — the **final class assignment** of every constant: LAW vs SPECIES vs TEMPERAMENT
- **Clash:** your batch says *"final class assignment of each constant — Mike
  proposes the table, owner rules"*. This is that proposal, and it is the only item
  in this ledger whose flip is not a one-file edit.
- **Build choice:** the table is engineering/03 §1.2, built by the two tests of
  engineering/10 §2 — a constant is TEMPERAMENT only if two individuals may
  legitimately differ in it **and** no guard's bound moves with it; SPECIES if the
  sweep must validate a range across a panel; LAW otherwise (never swept, never
  seeded).
- **Consequence of flipping a cell:** if the row crosses into or out of
  `DERIV_ORDER`, the vector length changes ⇒ `seed_format` bumps ⇒ **every hash in
  every existing run and checkpoint becomes non-replayable**.
- **Say which:** accept the table / move named rows (list them by registry name).

## OD-72 — is there ever a **regulated curiosity level**?
- **Clash:** the chunk's "curiosity set point" reads as a homeostatic controller, and
  OD-22's strict reading is that arousal may not gate emission.
- **Build choice:** **refused as a controller; shipped as a static per-individual ask
  ceiling** (engineering/10 §13). §18.4's own consequence paragraph already chose the
  strict reading; a set point re-opens it.
- **Say which:** keep the ceiling / fund the controller (a design amendment first).

## OD-73 — hysteresis, or per-signal rise/decay constants?
- **Clash:** "rise fast, fall slow" is the classic affect shape and §18.4 cancelled it
  by name.
- **Build choice:** **refused** — one scalar, one `λ_a`. The rise half already exists
  as `GAIN_SIGNAL[k]`, so nothing is lost but the name (engineering/10 §13).
- **Say which:** keep refused / reinstate (that reinstates the second integrator I8
  was written against).

## OD-74 — is the arousal curve **shape** a temperament trait?
- **Clash:** §18.1 says the curve is "swept before quoted, never hand-argued", and
  your batch says an individual gets drawn traits.
- **Build choice:** **family = SPECIES (swept), member selection = TEMPERAMENT
  (seeded).** `MOD_CURVE_VARIANTS` is a species-chosen count; a seed may pick a sealed
  member — the variant index lives inside hash item 8 — and may never construct a
  curve or reach outside the published set (engineering/10 §13, 05 G14(k)).
- **Say which:** keep the two-level split / seed the shape continuously.

## OD-75 — rescale semantics: ratio-preserving contraction or mass-preserving redistribution?
- **Clash:** "renormalize the live weights, preserving relative ratios" has two
  readings, and one of them makes dimming brighter somewhere.
- **Build choice:** **headroom-ratio-preserving contraction; mass is not preserved;
  brightening is refused** — `w' = floor_shelf + (w − floor_shelf)·d` with `0 < d ≤ 1`
  (engineering/10 §9, 05 G13(b)(d)). A mass-preserving rescale is R-07 and I6 in a
  consolidation costume, and it would let a night of sleep *strengthen* an edge no
  one rehearsed.
- **Say which:** keep contraction / ask for redistribution (needs a law change).

## OD-76 — is dimming **stored**, or a third read-side term in `W_eff`?
- **Clash:** "access decay" could be a value or a view.
- **Build choice:** **stored, and written only by the sleep pass** — the citation,
  the panel and the walk must read one byte, and "dark = unroutable but still
  citable" is honest only because every weight change is a dated, logged event
  (engineering/10 §13, 04 §26 law 25).
- **Alternatives:** (b) read-side — no write, but then three consumers compute
  `recency` at three instants and disagree, and the trace cannot show why a route
  failed.
- **Say which:** keep stored / (b).

## OD-77 — may a shelf **floor** be derived from `bake` (the pilot light as immunity)?
- **Clash:** the chunk's "THE BAKE = THE PILOT LIGHT" is poetic and could be read as
  an arithmetic floor: a baked edge never dims below X.
- **Build choice:** **refused.** bake keeps its two licensed outlets (the saturation
  curve and the ignition preference); deriving a floor from it would be a second,
  unbounded durable affective quantity and would make some edges permanently immune
  to quiet (engineering/10 §13, §6.3).
- **Say which:** keep refused / allow immunity (then state which bound it, because
  immunity without a bound is R-28).

## OD-78 — does sleep **consume logical steps**?
- **Clash:** "offline consolidation" suggests a phase outside `t`; I5 says all
  durability is in logical time.
- **Build choice:** **yes — every walk sleep runs is a thought and advances `t` by
  1.** A sleep runs one quiet tick per walk, each closing with the ordinary
  `flush → compose → advance` and each opening with the inbox read; a sleep of *w*
  walks leaves the clock at `t₀ + w`, because it *is* *w* ticks and not one tick doing
  *w* things (engineering/10 §7, 02 §11 step 12b).
- **Alternatives:** (b) a parallel sleep-time axis — a second clock (R-29) and an I5
  hole; (c) zero-step sleep — then a replay that skipped a rescale still matches, so
  the phase becomes untestable.
- **Say which:** keep (a) / (b) / (c) (and know that (c) deletes G13's basis).

## OD-79 — interrupt granularity when you type mid-sleep
- **Clash:** "interruptible instantly by owner input" against "a rescale preserves
  relative ratios", which is not a partial operation.
- **Build choice:** **pass boundary.** `si-sleep` performs the same non-blocking inbox
  read at each boundary; a pending frame aborts the remaining passes. A partial sleep
  is a legal, logged, replayable state (`sleep.jsonl`'s interrupt line), and a
  rescale is never half-applied (engineering/10 §7, 05 G13(g)).
- **Say which:** keep pass-boundary / mid-edge abort (needs a rollback design, which
  is a journal, which is a new durable thing).

## OD-80 — REM/NREM-style **staging** of the four passes
- **Clash:** your chunk proposed the four-pass shape and separately asked whether
  staging belongs in v1.
- **Build choice:** **parked; v1 = four fixed passes in fixed order**, with no pass
  reading a previous pass's output (engineering/10 §13, 04 §28 Z). A pass graph is a
  scheduler in disguise, and a scheduler is a new organ nobody scoped.
- **Say which:** keep parked / schedule in v2 (the display already tolerates it —
  kind 19 carries a `pass` id).

## OD-81 — "session close" as a trigger, in a loop that may not read a wall clock
- **Clash:** the chunk names session close as a sleep trigger; the loop's step order
  has no wall clock.
- **Build choice:** **sleep-at-snapshot** — the freeze path runs the passes *before*
  writing the checkpoint, so the trigger is a step-anchored event, the sleep is
  inside the durable state, and a resume never repeats it (`last_slept_at_t`)
  (engineering/10 §7, 05 G13(h)).
- **Measured at B8** (`gates/G13_si_20261023-2147`, inside `G13.f.1`, because "once
  per cadence" is the same claim as "a resume never repeats it"): the day is driven,
  the passes run, then the file is written and re-opened. Across the freeze
  `sleeps_completed` goes 4 → 5 and `last_slept_at_t` 19 → 20; the re-opened session's
  state hash is **byte-identical** to the pre-save hash
  (`937239800659dca79a90558a24bf9161`), and its quiet counter is 0 (06 §3.4 — derived
  RAM, never stored), so the first tick after the resume writes **no** night line and
  `sleeps_completed` stays 5. The night is inside the snapshot and is not repeated.
  The crash ordering (a cut *before* the passes reach the checkpoint) is the other
  half and is graded separately at `G13.h.1`.
- **Say which:** keep sleep-before-checkpoint / sleep-after (after means a crash can
  leave a half-applied rescale readable, which is the state §23's ordering law exists
  to prevent).

## OD-82 — `prov_kind = 5 consolidated`, if OD-58 lands yes
- **Clash:** P2's integration write needs a provenance kind, and P2 has nothing
  lawful to write until inference promotion exists.
- **Build choice:** **specified inert** — the row shape and the export literal are
  documented so that ruling OD-58 becomes one edit, and nothing ships on it today
  (engineering/10 §13, 06 §12).
- **Say which:** nothing here is independent of OD-58; answer it with OD-58.

## OD-83 — does the species sweep validate the **whole box** or only the point it picks?
- **Clash:** a swept range that is tested only at its winner lets a legal seed land
  in an ungoverned corner.
- **Build choice:** **the whole box** — corners plus panel points, at every re-sweep
  (engineering/10 §13, 05 §4.1, G14(e)(h)). It costs `|panel|` runs per re-sweep, and
  that is the price of individuality rather than a rounding error.
- **Say which:** keep whole-box validation / point-only (cheaper, and it makes the
  seed panel a lottery rather than a design).
- **RULING (owner, 2026-10-25, B12 batch).** **The whole box stands.** Every sweep
  validates the entire box at every re-sweep: the winning point **and both box
  corners** (range floor, range ceiling) re-run at every `seed_panel` member, with
  the gate's behavioral legs graded at each such point (§4.1 steps 1–4). The
  point-only alternative is **rejected** — it makes the seed panel a lottery rather
  than a design. A panel point that fails narrows the **box** in
  [03](../../engineering/03-constants-registry.md) (a `seed_format` bump and
  therefore every subsequent hash), never the panel and never the point.
- **Status:** **RULED (2026-10-25, B12 batch)** — validation: the whole box.
  §4 step 1's search enumeration is the sweep law's own text; this ruling does not
  restate it, and B12 executes it as written.

## OD-84 — `docs/12`'s component map gains a logical module (`consolidate.py` ⇄ `si-sleep`)
- **Clash:** the build has a thirteenth module and the design's map has twelve;
  engineering may not edit `docs/` silently (authority order).
- **Build choice:** **lean yes** (engineering/10 §13). The alternative — folding
  `consolidate` into `si-learn` — is worse: it deletes the missing crate edge that
  makes "sleep creates no row" a compile-time fact instead of a promise.
- **Say which:** accept (then someone edits docs/12 — that is a design-layer edit,
  so it is your hand, not mine) / fold into `si-learn` / leave the map stale.

## OD-85 — how many sense events does **P1 replay**, and how do P1 and P4 share `SLEEP_MAX_WALKS`?
- **Clash:** your chunk said P1 "replays" (open-ended), the registry said the walk
  count is **bounded**, and nothing reconciled them until the budget arithmetic was
  actually run: replaying every event since birth makes sleep O(session) inside a
  loop whose whole cost story is "nothing grows with session length except through
  `E_live`, which is capped" (engineering/06 §5.1, 10 §13).
- **Build choice:** **replay the `context` rows new since the last sleep**
  (`t > last_slept_at_t`), P1 spending the one budget first (keeping the newest,
  walking oldest-first) and P4 taking the remainder. A P4 with remainder 0 still
  sweeps in handle order, still reports, writes nothing. P3's rescale and P4's
  *selection* are never budget-bound, so no bounded night leaves an edge undimmed.
  Truncation is **reported** as `walks` vs `events_available`, and asserted by
  **G13(l).4**.
- **Say which:** keep one shared budget / give P1 its own cap (a second constant to
  sweep, and it makes a night's work depend on a knob nobody asked for).

## OD-86 — the build must run before any sweep exists: what value does a `—` constant take?
- **Clash:** [03 §1](../03-birth-state.md)'s rule (restated in
  [engineering/03](../../engineering/03-constants-registry.md) §1) is that every swept
  row has `default: —` and *"an engineer who invents a value for a `—` row has written
  a design change through the back door"* (R-03). But Phase 3 is a build that must
  compile, run, and produce the very artifacts the sweeps read — and a single-line
  transcript already needs `TAU_SETTLE`, `FRONTIER_CAP`, the decay pair, the six signal
  gains and the bake pair to have *some* value. Refusing to run until every row is ruled
  makes the build untestable and every gate unmeetable; picking "sensible" values by
  hand is exactly the failure the rule names.
- **Build choice:** **the placeholder policy** — one mechanical rule, no judgment, no
  optimization: `value = lo + round_half_up((hi − lo)/2)` snapped to the row's own grid
  (the same mapping a temperament draw uses, 10 §4 P.2). It is not called a default. It
  lives in `si-core`'s registry wrapper, every entry is tagged
  `UNMEASURED_PLACEHOLDER`, the whole set carries `consts_version = 0x0001` and is
  printed in every `gate_report.json` under `placeholders[]` and counted in
  `unresolved_legs`. A gate leg whose *assert* depends on the choice of a placeholder
  (not merely on its existence) may not report green because of it: it reports
  `UNMEASURED` with the placeholder named, and turns green only when a `*_sweep.json`
  supplies the swept value and its version tag (the sweep reads the legal box, which is
  legal by construction). `--strict-consts` refuses to start while any placeholder is
  live, so "shipped vs measured" is a flag on the run, not a memory in someone's head.
  A placeholder is replaced only by a sweep artifact, never by a hand edit; the
  registry's owning row names the artifact and its sha256, and `check_registry_size`
  asserts the placeholder table's length equals the registry's swept-row count, so a
  row that silently acquires a value fails the build.
- **Say which:** keep the placeholder policy (the build runs, nothing measured is
  claimed, every artifact names its arithmetic) / refuse to build until every row is
  ruled (honest, and it means no gate of this job can go green) / let the engineer pick
  values (forbidden by 03 §1 — this is the option the policy exists to make unnecessary).

## OD-87 — the four `IGN_WEIGHT_*` rows: swept species constants or temperament draws? — **RULED by the owner (2026-10-23, B7 batch: the class split)**
- **Ruling:** the split is **drive vs multiplier**. The decay-deficit DRIVE is
  SPECIES — *the mind drifts toward what it is about to forget* is mechanism, not
  personality — and it has **no row of its own**: 03 holds no drive constant, only
  the multiplier that scales its term in §S.3's sum. The four `IGN_WEIGHT_*`
  multipliers across the ignition sources are **TEMPERAMENT draws**: where the idle
  mind wanders — puzzles vs people vs places — is exactly the personality layer,
  chosen once at birth, never swept, and no best value exists. **"No sweep may ever
  touch a temperament row."** And "same constant set" in the temperament clauses
  means **same birth seed**: the seed deterministically draws the constants, so
  **instance identity is seed identity** — which is how G13's divergence/identity
  pair already states it.
- **Build consequence (measured):** the registry carries all four as
  `Class::Temperament` and `sweep::axes()` filters on `Class::Species`, so the G10
  sweep enumerates `IDLE_PER_HOUR` alone. The gate tags the 18 draws still carry are
  **inert metadata** — they name the artifact that publishes the row's *drawn*
  distribution, not a grid — and `check_registry_size`'s count identity depends on
  their presence, so they stay. The new B0 leg
  `check_sweep_never_touches_temperament` ([engineering/05 §20](../../engineering/05-gate-harness.md))
  locks the law from the four sides a sweep can reach: **25 swept axes across 7
  enumeration prefixes, every one SPECIES**; the placeholder rule gives no value to
  any of the 18 draws; `launch_constants` refuses a non-SPECIES name by class; the 4
  published sweep artifacts name no draw (`gates/checks/stage-B0.json`).
- **Alternatives:** (b) the four weights as SPECIES rows — G10's sweep would then
  *search* personality, and every individual would be measured against one winner,
  which 03 §1.2's own test 1 refuses ("else the sweep becomes a per-individual
  search"); (c) `IGN_WEIGHT_DECAY_DEFICIT` species and the other three draws — two
  classes inside one summed sampler, so individuals differ in the sampler's *shape*,
  not its strengths; (d) strip the tags off the TEMPERAMENT rows — that rewrites
  `check_registry_size`'s identity for no behavioral gain and loses the pointer to
  the artifact that publishes the distribution.
- **How to reverse:** re-assign the class in [03 §1.2](../../engineering/03-constants-registry.md)
  and §5's `kind` column **and** in `si_core::consts::CONST_ROWS`, add or remove the
  rows in `DERIV_ORDER` (a length change is a `seed_format` bump ⇒ every hash changes
  ⇒ no old replay reads under a new individual), then re-run
  `si-harness check --stage B0`; `check_temperament_closed` and
  `check_sweep_never_touches_temperament` fail together if the two tables disagree.
  To reverse the "same birth seed" reading, re-word [05 §15(e)](../../engineering/05-gate-harness.md)
  and docs/13's G10(e) back to a *constants* identity — but that asks two
  individuals to be twins, which is what this ruling refuses.

## OD-88 — G10.b.2's byte-strict stamp assert forbade a move the design licenses — **AMENDED by the owner (2026-10-23, B7 batch: the stamp tolerance)**
- **Ruling:** the gate is amended to match the design's own license:
  [05 §15(b).2](../../engineering/05-gate-harness.md)'s may-differ list governs
  **stamps**; **rows and edges stay byte-strict**. Recorded reason:
  [04 §19 §S.7](../../engineering/04-algorithms.md) licenses idle-cascade movement of
  `a`, so a byte-strict stamp assert forbids lawful behavior. This is the correction
  of an over-strict gate leg, not a weakening of the design — the design never
  demanded stamp-byte equality; the gate draft did.
- **Required artifact, cited as the evidence (measured):** the V2–G7 control run
  pair — `gates/G7_si_20261023-1600` (V0) and `gates/G7_si_20261023-1612` (V2), same
  birth seed `0x0000000000000051`, same transcript (`harness/scenarios/g7_reparse.txt`,
  16 steps, md5 `167018fb2333d5011f873e6911877f37`): rows **4032 B** identical, edges
  **192 B** identical, the S-R envelope digest
  `811daffea525b8cc44c9927a500100c8` identical, and the stamp arena **208 B in both**
  with digests `58371b598896fae9f681212814a9beb0` (V0) vs
  `8e6f96035d6084b7fc346a33731346b9` (V2). Same bytes, same number of records,
  different values — exactly the shape the amendment licenses and the old leg failed.
- **Build consequence:** `G10.b.2` now asserts *structurally* on stamps (record count,
  arena byte length, layout) and *byte-strictly* on rows and edges. In the G10
  fixture's own V0/V2 pair the stamp triple is `[8, 128, be4605e21887e55893367dbc965e65c1]`
  on both sides (`gates/G10_si_20261023-1732/gate_report.json`) — identical there
  *because that scenario's quiet ticks moved no stamp value*, which is a measurement,
  not a tolerance.
- **Alternatives:** (b) keep byte-strict stamps — G10(b) then fails on every lawful
  run where an idle cascade moves `a`, and the honest reading of that failure is the
  gate, not the build; (c) tolerate stamps by count only and drop the byte length too
  — that lets the arena *grow* during quiet ticks, which I9/R-25 forbids; (d) leave it
  deferred and raise the question (what the build did before this ruling: the leg
  reported `open`).
- **How to reverse:** restore the stamp-digest byte compare in `G10.b.2`
  (`crates/si-cli/src/harness/gates.rs`) **and** delete §S.7's rehearsal allowance
  from the design — the two must move together, or the leg fails on a run that is
  lawful by §S.7. Re-runnable evidence: both G7 runs are on disk, and
  `si-harness replay --run gates/G7_si_20261023-1612` re-executes the V2 side at its
  recorded variant.

## OD-89 — G10.e.2's decay-deficit rehearsal: is the feedback wired, and does repetition stop? — **RULED by the owner (2026-10-23, B7 batch: verify before adding a constant)**
- **Ruling:** do not add a sparsity rail yet. First verify that a just-ignited
  point's decay deficit actually drops — rehearsal feeding back into the ignition
  selection weight — because if it is wired correctly, same-point re-ignition within
  three quiet ticks should not win the race. Then report **either** (a) the feedback
  was missing or ordered wrong, now fixed, gate green with zero new constants, **or**
  (b) the feedback is verified correct and repetition persists, and *then* propose a
  per-node refractory constant. "Either outcome is a report, not a decision made
  alone."
- **Report — both halves are true, and they are different halves.**
  **(a) The feedback was missing, end-to-end.** `Lattice::touched` — the deficit
  map's only source — was written by `confirm()` and nothing else, so during a quiet
  tick the map was empty and `IGN_WEIGHT_DECAY_DEFICIT` scaled a term that never
  moved. Wiring §S.7's own writer (`touch_edge_curve(edge_off, at_t)`) into the
  cascade's traversed arcs made rehearsal reach the sampler, and **G10.e.2 is green
  with zero new constants**: the measured arc deficit on the traversed edge falls
  `[64, 16] → [32, 16]` from t=6 to t=7 (`gates/G10_si_20261023-1732/gate_report.json`,
  `reset_narrowed_the_curve: true`).
  **(b) Verified correct — and repetition persists.** The point-level reading stays
  `[32, 32]` at every one of seed 5's ticks 6/7/8 and seed 2's ticks 4/5/6, because
  the implemented aggregate over a point's live incident arcs is the **furthest-drifted**
  one (04 §S.3's `deficit(point's youngest weak edges)`, read as a max — the reading
  is recorded at OD-90), `recency_u8` is 254/255 for a just-lit point, and every term
  of §S.3's sum is non-negative. Re-heating one arc therefore cannot lower the pull
  another arc is holding up. The gate still passes: §15(a)'s failure is a *budget*
  violation, and 6 entries across 7 quiet ticks with no two sharing a `t` violates
  none of the three bounds (`G10.a.3`, `shared_ticks_within_a_section: []`).
- **So the proposal the ruling authorises is raised, not applied:** see **OD-90**.
- **Alternatives refused by the ruling itself:** adding a sparsity rail before
  measuring (the build would have hidden whether the feedback existed at all);
  deciding the refractory constant while coding it (D5 — a value a sweep owns is
  never written by hand, and this one is not even named).
- **How to reverse:** nothing was added, so nothing to remove — the reversal is of the
  *fix*: stop calling §S.7's writer in the cascade (`si-walk`'s idle path) and the
  deficit map goes back to empty during quiet ticks, which `G10.e.2`'s
  `reset_narrowed_the_curve` leg then fails on.

## OD-90 — proposed, not applied: what stops the same point re-igniting three ticks running? (raised by OD-89(b))
- **Question:** the design's line is
  [`IGN_WEIGHT_DECAY_DEFICIT · deficit(point's youngest weak edges)`](../../engineering/04-algorithms.md)
  §S.3. The build reads that as the **max** over the point's live incident arcs — the
  furthest-drifted arc, "what I am about to forget" — which is what makes OD-89(b)'s
  persistence arithmetic rather than a bug. A literal **youngest** reading (the
  least-drifted arc) says the opposite. Three ways out, and all three are the
  owner's:
  - **(a) add a per-node refractory constant.** Proposed shape only, no value:
    name `IDLE_REFRACTORY_TICKS`, home `si-walk::idle`, class **SPECIES** (it guards
    a mechanism's cadence rather than bending its strength, so 03 §1.2's test 2 keeps
    it out of the drawn set), swept at **G10**, and given a **design ceiling** instead
    of the type maximum — which is the shape the still-open sweep-enumeration ruling
    needs anyway (a grid with no last point cannot be walked; see the refusal recorded
    in `gates/G10/ignition_sweep.json`). No value is proposed here: **D5** and OD-86
    say a sweep owns it.
  - **(b) read §S.3's "youngest" literally** — aggregate = the least-drifted live
    arc, so reheating the arc an ignition just traversed *does* drop that point's
    deficit and repetition stops with **no new constant at all**. Cheapest, and it is
    the docs' own wording; it changes a reading of a law line, so every
    deficit-weighted draw made so far becomes non-comparable and the hash format's
    version story has to say so.
  - **(c) keep it as it is** — three consecutive intrusions of the same point across
    7 quiet ticks may be the liveliness the design asked for ("the mind returns to
    what it is about to forget"), and §15(a)'s bounds already catch the pathological
    case. Then §S.3's wording gets amended to "furthest-drifted", which is what the
    build does today.
- **Why this is not decided in the build:** (a) spends a constant the design does not
  have; (b) re-reads an arithmetic line of a law; (c) accepts the behavior OD-89's
  ruling expected *not* to happen. Nothing in the build implements (a); it runs (c)'s
  behavior through (b)'s ambiguity, and says so in `G10.e.2`'s
  `point_level_reading_means`.
- **Status:** OPEN. Ask: `OD-90: a / b / c`.

## OD-91 — G10.e.5: must V0 and V3 on one transcript *differ*? — **RULED by the owner (2026-10-23, B7 batch: identical hash is the pass condition)**
- **Ruling:** the audit layer is observer-only **by law** (G9's own fail condition is
  an emotion label feeding back into control), so V0 vs V3 on the same transcript must
  produce an **identical state hash** — "audit removal changing nothing" is the costume
  test passing at build-graph level. That the two named transcripts did not exist is a
  **harness-spec defect**: fix the names to real artifacts (or regenerate them), keep
  identical-hash as the assert, and record the amendment. If any spec text demands a
  measurable difference *here*, that text contradicts the readout law and gets routed
  as its own OD — **never satisfied by manufacturing one**. Such text does exist
  (05 §15(e) and docs/13's G10(e), "over-represented relative to V3"): it is **OD-92**.
- **Named exception, stated so it does not read as a contradiction with OD-93:** the
  determinism clause is **per-variant**, and cross-variant comparison is a category
  error — except that `G10.e.5` *is* the designed control comparison, the one place a
  cross-variant identity is asserted on purpose. It holds for a structural reason the
  design chose: the idle lane writes nothing durable (I9), so what a bake-zeroed
  control can change lives in the **readout** lane (`EV_IGNITION`'s payload carries the
  drawn seed and point) and never among the nine hash classes. The leg proves that
  separation rather than assuming it.
- **Measured (the evidence — `gates/G10_si_20261023-1732/gate_report.json`, seed
  `0x0000000000000051`, `state_hash_identical: true` on both):**
  - `harness/scenarios/g10_high_bake.txt` (digest `34cd6a9583d72869d9036e3ef33cd59a`) —
    bake values carried `[2053, 2053, 32768, 32768, 32768, 32768, 32768, 32768]`,
    **2 distinct**; V0 sections `[89f4a17ed140240a879d48fe6ccb4e91]` = V3 sections;
    rows, edges and stamps identical; 3 ignitions and 3 monologue lines on each side.
  - `harness/scenarios/g10_all_saturated.txt` (digest `61411f1722e7c9d400a8e2d648467659`)
    — bake `[32768 ×8]`, **1 distinct**; V0 = V3 = `[d116acfbc4eb1ccb2af6999b59124cdd]`.
  - The two are a *pair* because their **shapes** differ (spread ≥ 2 vs exactly 1); the
    comparison inside each is identity. Beside it the leg reports
    `launched_transcript_v3_changed_behavior` — the V0/V3 **ignition and monologue-entry
    counts** compared, and on these two transcripts the counts did not differ (`false`).
    Which points V3 drew is a readout question the leg does not compare, and the identity
    does not depend on it: the drawn point and its seed live in the `EV_IGNITION`
    payload, which is not among the nine hash classes.
- **Alternatives:** (b) grade the old wording — needs a durable difference, i.e. an
  idle-adjacent write I9 forbids, or a hash that includes the readout lane, i.e. R-33
  and G5(f) inverted; (c) leave the leg deferred, which is what the build did before
  this ruling and which kept §15(e) partly unmeasured; (d) assert on monologue counts
  instead of the hash — weaker, because a count is a readout and the law says a
  readout may not be *required* to move.
- **How to reverse:** make the ignition lane a hash input (add the `EV_IGNITION`
  payload to one of the nine classes in `si_core::hash`) — that reverses the readout
  law, not just this leg, and it fails G9(c)'s V1 identity pair and I9 in the same
  commit. Or re-grade the leg as a difference assert, which needs OD-92 ruled the other
  way *and* the same hash change, because today no cross-variant difference exists to
  measure.

## OD-92 — §15(e)'s "over-represented relative to V3" demands a difference the readout law forbids (raised by OD-91)
- **Clash:** docs/13's G10(e) says a session with one high-bake edge "must show that
  edge's endpoints **over-represented** in the monologue relative to a bake-disabled
  control", while the same row says "same build, same constant set, two transcripts".
  Over-representation **in the monologue** is a comparison of readouts; the design's
  own laws then forbid the only durable difference that could stand behind it — V2's
  leg requires *every* `bake` field identical, I9 forbids an idle write, and putting
  the monologue into the hash reverses R-33. So the sentence either grades nothing, or
  it grades a readout statistic, or it contradicts the readout law.
- **Build position:** the leg does **not** grade that text. `G10.e.5` asserts the
  identity (OD-91) and names the routed text in its own payload
  (`over_representation_predicate`). No difference has been manufactured to satisfy it,
  and no statistic has been invented to stand in for it. It is not in
  `unresolved_legs`, because the leg that would have graded it now grades something
  else — which is exactly what needs the owner's eye.
- **Options:** (a) score over-representation **as a readout statistic** — monologue
  mentions per endpoint, V0 vs V3, published beside the identity with the standing
  note that it compares readouts and never state; this needs a denominator the design
  never gives (entries? ticks? point-mentions?), so it is a new definition, not a new
  measurement; (b) drop the sentence from 05 §15(e) and docs/13's G10(e) and let
  `G10.e.4` ("the sampler's salience weight's source is the stored `bake`") carry the
  claim — it is the load-bearing leg and it is green; (c) re-point the comparison at a
  different control that *may* differ durably, which re-opens OD-87's class split;
  (d) keep the sentence as design intuition and gate nothing on it.
- **Downstream:** [05 §4](../../engineering/05-gate-harness.md)'s G10 sweep objective,
  as written in `sweep::objective("G10")`, cites this text ("the bake weight's effect
  against the V3 control shows as over-representation"), so B12's ignition sweep must
  be re-worded once this is ruled. Nothing at B7 is blocked by it.
- **Status:** OPEN. Ask: `OD-92: a / b / c / d`.

## OD-93 — does a launch variant need a registry row, and is "same seed, same transcript ⇒ same hash" global? — **RULED by the owner (2026-10-23, B7 batch: the variant tag)**
- **Ruling:** **no registry row, ever.** Build-graph variants are pinned by the
  deps_lock/graph digest, not by constants-registry rows — that is already the
  mechanism and it is enough. Every run, replay and export report records the variant
  id **and** the graph digest it executed on. The global determinism clause reads
  **per-variant**: same seed + same transcript + same variant ⇒ identical state hash.
  A cross-variant hash comparison is a **category error**, not a failure.
- **Build consequence (measured):** `harness::build_stamp(ws, variant)` is the single
  pair, and it now appears in every gate report's `build_identity`
  (`build_variant` + `cargo_lock_md5`), in `mindlog.json`, in all three `.siro` lane
  headers (`sense_inbox.siro`, `stream.siro`, `monologue.siro`) via `stamp_into`, in
  the replay report — which prints the variant it found recorded, re-executes **at that
  variant** and prints the live graph digest beside it — in `si-cli`'s run output and
  in `export.json`. The current graph digest is
  `04d8ba8cde7088aaba0595f158cfe660` (the same md5 `check_deps` compares against
  `gates/deps_lock.md5`).
- **Tension named rather than hidden:** OD-91's `G10.e.5` *is* a cross-variant identity
  assert. It is lawful because it is the designed control comparison and not a
  determinism claim — this ruling forbids *concluding a failure* from a cross-variant
  comparison; OD-91 requires *equality* at one named place, for the structural reason
  its entry states. If the owner reads this ruling as forbidding even that, the fix is
  to move `G10.e.5` onto a readout statistic, which is OD-92's option (a).
- **Alternatives:** (b) give V2/V3 registry rows (`IDLE_PER_WINDOW = 0`,
  `IGN_WEIGHT_BAKE = 0`) — that puts a TEMPERAMENT draw in a swept table, which OD-87
  refuses, and makes a launch state look like a constant the loop owns; (c) hash the
  variant id into the state hash — then a control run's hash can *never* equal its
  baseline's, which breaks G9(c)'s V1 pair and G12(a)'s V4/V5 pair by construction;
  (d) record only the graph digest — the digest pins the build graph, and V2/V3 are
  launch *states*, so a replay would silently re-run at the wrong one.
- **How to reverse:** delete `build_stamp`/`stamp_into` and drop the `variant`
  parameter from `Run::replay_transcript`, then add the rows to `CONST_ROWS`. The first
  V2 or V3 replay after that refuses to match its own recorded hashes, because a
  replay re-executes at the registry's values instead of at the variant the run was
  made at — that failure *is* the reason the parameter exists.

## OD-94 — is the dimming box walkable at all, and what is a `FLOOR_*` for? (raised by B8's `dimming_sweep` refusal)
> **Numbering note (2026-10-25).** The B12-batch work order's item 2 is headed
> "OD-94", but its text — "crystallize constants take their values from the B12
> sweep artifacts only" — is the **§22 draft's** OD-94, which §22.11 re-seated as
> ledger **OD-97**. This entry is B8's dimming box, a different question, and it
> stays **OPEN**; the ruling is recorded under OD-97.
- **Clash:** [03 §5](../../engineering/03-constants-registry.md) gives `FLOOR_S_R`,
  `FLOOR_FACTS` and `FLOOR_LINKS` a legal range of **1…65535 Q12** with the `hi` open,
  while a live OWNED edge forms at a weight of the order of **4096 Q12 plus its bake**.
  So both ends of the box are degenerate, for opposite reasons: at `lo` the floor sits
  under every edge and anchors nothing (10 §6 D.2's clamp never binds), and above the
  largest live weight every edge is already "below floor", where D.2's clamp turns P3
  into a no-op. The interesting band is between them, and the registry says nothing
  about it.
- **Measured, from this stage's own run** (`gates/G13_si_20261023-2147`, the registry
  `lo` point per OD-95): `G13.d.2` reads **8 live edges, 0 below floor, minimum floor
  1**; `gates/G13/dimming_sweep.json` refuses its grid because 3 of 4 axes have an open
  bound and the cross product of the rows as written is **71772833461370625** points.
  The `lo` end is not hypothetical either: `crates/si-sleep/tests/sleep.rs`
  `an_edge_below_the_floor_stays_and_is_counted` builds a `links` edge at 40 under a
  floor of 64 and asserts it stays put and lands in P3's `edges_below_floor` count —
  the conflict is *reported* by the pass, not swallowed by a clamp the spec does not
  license.
- **Options:** (a) narrow the box (05 §4.1 step 4 is the honest move) so that
  `FLOOR_*`'s legal range is expressed relative to the weight scale, e.g.
  `lo = 1, hi = INIT_WEIGHT_Q12`, which makes every point of the sweep a point where
  the floor can actually bind; (b) declare `hi` a real design ceiling and leave `lo`
  where it is, accepting that the low band is a no-anchor region the distribution is
  meant to *show*; (c) leave the rows as written and let the artifact stay refused,
  which keeps OD-83's whole-box validation unsatisfiable for these three rows.
- **Downstream:** B12 (the sweep stage) cannot publish `dimming_sweep.json`'s rows
  without this ruling, and 05 §18(c)/(l)/(n)'s magnitudes are UNMEASURED until it
  does. Nothing at B8 is blocked: the three legs that touch a floor (`d.1`, `d.2`,
  `l.1`) grade at the point the run actually holds and name OD-94 in their `observed`.
- **Status:** OPEN. Ask: `OD-94: a / b / c`.

## OD-95 — with `sleep_sweep` refused, at what operating point is a G13 leg graded at all? (raised by B8; the G13 instance of OD-83 and OD-86)
> **Numbering note (2026-10-25).** The B12-batch work order's item 3 is headed
> "OD-95", but its text — "candidate tables are hash-in session state" — is the
> **§22 draft's** OD-95, re-seated as ledger **OD-98** by §22.11. This entry is the
> G13 operating-point wall and stays **OPEN**; the ruling is recorded under OD-98.
- **Clash:** 05 §18's prerequisite is a scenario in which "`SLEEP_EVERY_QUIET_TICKS`
  fires **at least twice**", but the registry's placeholder for that row is the
  midpoint of its own box, **32769**, which no scenario of `G5_LONG_SESSION_INPUTS`
  shape can reach — so at the shipped value the prerequisite is unsatisfiable and the
  gate has nothing to grade. `si-harness sweep --gate G13` refuses to pick a winner
  (the enumeration question is OD-83's and it is still open), so the sweep cannot
  supply a value either.
- **Build choice:** **the registry `lo` point, named in the artifact and in every
  report line**. `si-harness run --gate G13` installs the six sleep rows at their
  declared `lo` (`runner::sleep_lo_point()`: cadence 2, `SLEEP_MAX_WALKS` 1,
  `DIM_BASE` 1, the three floors 1), because `lo` is a point the spec already declares
  rather than a number a reviewer chose. It prints the reason on the run's stdout
  ("G13 launch point: the six sleep rows at their registry `lo` (05 §18's
  prerequisite, OD-95)"), and `gates/G13/sleep_sweep.json` carries the same paragraph
  in its `od_question`. Measured at that point: 9 nights across the scenario's three
  sections, 4 of them cadence nights in the `V0` probe, 22 ledger rows, and the
  cross product the refusal is about — **4294770690** points.
- **What this is not:** a swept winner. Every cadence, walk count and dimming
  magnitude G13 reports is a value *at `lo`*, not a curve endpoint, and the leg that
  would have read the curve (`n.1`–`n.3`) says so in its own `observed.note`.
- **Options:** (a) keep the `lo` point and the label, which is what shipped; (b) mark
  every night-quantifying leg `unmeasured` until B12 lands a ruled sweep — the cost is
  that B8's exit gate G13 would then have no graded legs at all, and the
  `si-sleep` crate's own assertions (DoD item 1) would be the only evidence; (c) rule
  OD-83's enumeration first and sweep before grading, which re-orders B8 and B12;
  (d) let the scenario declare its own constant values in the checked-in file — lawful
  only if OD-86's placeholder policy is amended, because a scenario line is not a
  registry row and 05 §2 forbids the harness choosing a constant to make a leg pass.
- **Downstream:** B12 owns the ruling that makes this moot; OD-83 and OD-86 are the
  parent questions and this stage adds no new claim to either.
- **Status:** OPEN. Ask: `OD-95: a / b / c / d`, or fold it into OD-83 when that is
  ruled.

## OD-96 — a control variant has no input for the legs that quantify over the thing it disables (raised by B8's V9 run)
> **Numbering note (2026-10-25).** The B12-batch work order's item 4 is headed
> "OD-96", but its text — "ghost budget remains a species guard" — is the **§22
> draft's** OD-96, re-seated as ledger **OD-99** by §22.11. This entry is the
> control-variant vacuity wall and stays **OPEN**; the ruling is recorded under
> OD-99.
- **Clash:** 05 §2 defines **V9** as "sleep disabled … `due()` never returns `Yes`",
  and 05 §17 makes that a *control* rather than a code path ("a run at V9 is disabling
  the trigger by its own swept range"). But 05 §18 writes four legs as quantifiers
  over the run's own nights — `a.1` ("replay … with the nights in the stream"),
  `c.1`, `c.2` ("within a shelf and within one night"), `f.2` ("a cold-start sleep over
  a nearly-empty L3") — and at V9 every one of them is a sum over the empty set. The
  gate's own honesty law says a run in which no sleep fired "is reported as a finding
  about the scenario, not as a green gate"; the variant's definition says the same run
  is exactly what the control is *for*.
- **Build choice:** **verdict `unmeasured`, naming the launched variant, with the
  graded half left where 05 §18 puts it.** The V9 run reports those four legs as
  `unmeasured` with the reason and the witness in `observed` (`nights_recorded: 0`,
  `cold_nights: 0`, an empty sweep ledger, and both replays exact); `a.2` still grades
  the pair inside one process, which is where §18(a) puts the control's claim ("V0 and
  V9 hashes **differ**, and the diff is attributable to `weight`/`last_touched_at`
  only"). At any *other* variant the same vacuity stays a **fail** — the honesty law
  keeps its teeth, because only V9 is defined as the absence.
- **Measured:** `gates/G13_si_20261023-2147` (V0, `pass: true`, 47 legs, 5 unresolved,
  all `unmeasured`) and `gates/G13_si_20261023-2147-2` (V9, `pass: true`, 9 unresolved,
  each named with the variant reason). `a.2` reports `hashes_differ: true`,
  `v0_nights: 5`, `v9_nights: 0`, `envelope_fields_moving: []`,
  `weight_moved_upward: false` — so the control *does* distinguish the pair, which is
  the failure §18(a) warns about ("a sleep V9 cannot distinguish is a pass that did not
  run").
- **Options:** (b) fail a V9 run on those legs — which makes the pair 05 §18(a)
  requires unrunnable through the gate path, since `run --gate G13 --variant V9` is the
  only way to produce the control's own artifacts; (c) pass them silently — forbidden by
  05 §3 ("a leg with no reachable input at this stage is `unmeasured` … never quietly a
  pass"); (d) amend 05 §18 to name, per leg, which variant grades it (the cleanest
  fix, and a spec edit the authority order puts in the owner's hand rather than ours).
- **Generalises:** the same shape exists at `G12`'s V4/V5 pair and `G9`'s V1, both of
  which are build-graph exclusions rather than launch states, so they land in B12's
  acceptance pass instead of here. If the owner rules (a), the ruling should say
  whether it covers those too.
- **Status:** OPEN. Ask: `OD-96: keep the treatment / b / c / d`.


# The v2 arc (docs/22–26) — OD-97 … OD-136

Folded 2026-10-23 from the owner's five drafts. Numbering follows the one
allocation table in [docs/23 §23.9](../23-context.md): B8's sleep walls took
**OD-94/95/96** the same day, so the arc starts at the next free seat. **OD-133
and OD-134 are tail seats** — found after the drafts closed, therefore appended
to the ledger's end rather than inserted into a doc's own range. **OD-135 and
OD-136 are engineering seats** — found while [engineering/12](../../engineering/12-v2-arc.md)
priced G19(c) and G19's fixture, appended for the same reason, and each ships its
**refusal** as the gate's pass condition so that no builder has to invent a value.

Every entry below states the **least-authority default the build ships**, because
an unrulled item still has to be codeable tonight; a default is a declaration,
not a ruling, and the owner can flip any of them without touching a law.

## OD-97 — the crystallize constants: `R_MIN_CRYSTALLIZE`, `R_ASK_CRYSTALLIZE`, `DELTA_MIN_CRYSTALLIZE`, `GHOST_MAX`
- **Clash:** §22.7's anti-spam bar needs four numbers, and the standing law
  ([docs/13](../13-gate-plan.md) shared shape) forbids quoting a magnitude before
  its sweep artifact exists. `si-harness sweep --gate G15` cannot pick a winner
  either, because OD-83's enumeration question is still unrulled.
- **Current doc choice:** four **SPECIES** registry rows at
  `UNMEASURED_PLACEHOLDER` (OD-86), `hi` left at the type maximum (OD-83 open), so
  `gates/G15/recurrence_sweep.json` publishes a **refusal** exactly like G13's two
  artifacts. G15 legs that cannot run without a value run at **the box's own
  floor** and name it a floor in `observed` (OD-95's precedent). No G15 leg cites
  a swept winner.
- **Alternatives:** (b) write the four numbers into §22.7 — forbidden by the same
  law that made OPEN-16 and OPEN-30 `UNMEASURED`; (c) mark every
  candidate-dependent leg `unmeasured`, which leaves G15 with only its negative
  legs graded; (d) rule OD-83 first and re-order V1 behind B12.
- **RULING (owner, 2026-10-25, B12 batch; the work order heads this item "OD-94",
  the §22 draft's number — §22.11 re-seated the draft's OD-94…98 as ledger
  OD-97…101).** *"crystallize constants take their values from the B12 sweep
  artifacts only. No hand-argued constants."* So `R_MIN_CRYSTALLIZE`,
  `R_ASK_CRYSTALLIZE`, `DELTA_MIN_CRYSTALLIZE` and `GHOST_MAX` are replaced only by
  `gates/G15/recurrence_sweep.json`, each value citing the artifact's sha256; until
  that artifact publishes a winner they stay `UNMEASURED_PLACEHOLDER`, and every leg
  that needs one runs at the box's `lo` and names it a floor (OD-95's precedent).
  **The numbering clash is recorded, not propagated:** no ruling attaches to B8's
  OD-94/95/96 from this item.
- **Status:** **RULED (2026-10-25, B12 batch).**

## OD-98 — candidate-table storage: hash-in vs declared hash-exempt
- **Clash:** I5 wants a replayable session, and a candidate table is session
  state that changes what a replay would have to reproduce. Hashing it in
  **re-hashes every run recorded since B0**, which is precisely the "nothing we
  already made breaks" constraint spent on an unrulled item.
- **Current doc choice:** **session-only, declared hash-exempt** (§22.11).
  Exempt ≠ invisible: every candidate transition is appended to
  `crystallize.jsonl` and to `gates/G15/nomination_log.jsonl`, so G15(a)'s
  byte-identity is proved against state and G15's content is proved against the
  log.
- **Alternatives:** (b) hash-in with a deterministic seeded table (re-baselines
  G1–G14 artifacts), (c) persist across sessions (then it must be hash-in and
  replayed, and the I5 wording changes).
- **Downstream:** OD-103 (silt) and OD-117 (skills) are deliberately the same
  ruling, so the arc cannot be accused of three different answers to one
  question.
- **RULING (owner, 2026-10-25, B12 batch; the work order heads this item "OD-95",
  the draft's number — §22.11's numbering note).** *"candidate tables are hash-in
  session state (the item-6 precedent). No hash-exempt exceptions."* The doc's
  current choice (session-only, declared hash-exempt) is **reversed**: the candidate
  table joins the state hash, is replayed, and is built deterministically from the
  recorded input stream so a replay re-derives it byte-for-byte. Because it is
  hash-in, every earlier run re-baselines at B12 — the bump is the announcement, not
  a side effect. **No hash-exempt exceptions** covers the two downstream entries
  this one already named: **OD-103** (the silt table) and **OD-117** (the skills
  table) take the same ruling.
- **Status:** **RULED (2026-10-25, B12 batch)** — hash-in; the B12 re-baseline is
  the implementation point.

## OD-99 — ghost budget: fixed cap vs arousal-scaled — **arousal scaling is BANNED by construction**
- **Clash:** "the ghost budget shrinks when SI is hot" reads as a nice story, but
  §22.3 bars `a` from the detector entirely, and OD-21 cancelled the coupling
  matrix that would have made such a rule coherent.
- **Current doc choice:** `GHOST_MAX` is a fixed cap and the arousal-scaled
  option is **absent from the code, not merely zero** — so G15(c) is a
  compile-time fact plus OD-134's frame-trace diff. Over budget, the frame builds
  no composite (§16.8).
- **Alternatives:** (b) arousal-scaled budget — requires the owner to reopen this
  entry explicitly *and* a coupling law that currently does not exist; recorded
  here so nobody reinstates it as a "fix".
- **RULING (owner, 2026-10-25, B12 batch; the work order heads this item "OD-96",
  the draft's number — §22.11's numbering note).** *"ghost budget remains a species
  guard. No temperament draws among v2 candidate/ghost constants;
  `check_temperament_closed` stays 18."* So `GHOST_MAX` stays a swept SPECIES row,
  (b) stays absent from the code rather than merely zero, no v2 candidate or ghost
  constant is added to `DERIV_ORDER`, and `check_temperament_closed`'s count is
  pinned at **18**.
- **Status:** **RULED (2026-10-25, B12 batch)** — (b) remains banned; the guard is
  species.

## OD-100 — placeholder naming and the rename path
- **Clash:** "real, but no word" must mint something addressable (that is the
  whole point of §22), yet a placeholder is not a taught word, and renaming it
  later touches I4's append-only law.
- **Current doc choice:** `cluster:{t_confirm}:{hash(members)}`, written as an
  ordinary owner-confirmed row. A rename is a **new owner-confirmed row** with
  the placeholder stamped `superseded_at`, so the chain stays citable and I4
  holds. The placeholder is never treated as a word — RENDER may not emit it
  (G15(f) is the leg, not a convention).
- **Alternatives:** (b) re-version the same row in place (breaks I4), (c) no
  rename path (the address stays ugly forever and later `is-a`-style reasoning
  cites a hash).
- **Status:** OPEN. Ask: `OD-100: a / b / c`.

## OD-101 — the membership relation kind for cluster edges
- **Clash:** `fruit → each member` wants a relation kind, and R-12 / §5.1 make a
  new kind a registry act with downstream greps (FACT eligibility, arbitration,
  the labeler's edge-class table).
- **Current doc choice:** ship on the **existing generic owned-link kind**; no new
  registry entry is created. What makes it a cluster is the ask row's `kind`
  (`cluster-nomination`) plus the citations, and the direction is printed by
  audit rather than encoded in a new literal.
- **Alternatives:** (b) reversed `is-a` (reads naturally, but silently claims an
  ontology SI was never taught), (c) a new `member-of` kind (opens §5.1's door
  and every table that enumerates kinds).
- **Status:** OPEN. Ask: `OD-101: a / b / c`.

## OD-102 — the silt constants: `LAMBDA_SILT`, `SILT_LEVEL_MAX`, `SILT_BUMP`, `THETA_SILT`, `SILT_MAX` (+ the idle-deposit weight)
- **Clash:** §23.3's deposit/decay/clearing law is five numbers, and the
  no-value-before-sweep law applies unchanged.
- **Current doc choice:** swept **SPECIES** rows at `UNMEASURED_PLACEHOLDER`, `hi`
  open per OD-83; `gates/G16/silt_sweep.json` publishes a **refusal**; legs that
  need a value run at the box's own floor and label it. The clearing leg (G16(d),
  last-touch order) is a *structural* assert and needs no swept value, so the
  refusal does not void it.
- **Alternatives:** as OD-97 (b)/(c)/(d) — same shape, same costs.
- **Status:** OPEN. Ask: `OD-102: a / b / c / d`.

## OD-103 — silt-table storage
- **Clash:** the table is genuinely session state, and B8's OD-96 lesson says a
  run's interesting statistics can be exactly the thing a control variant
  disables; hashing it in has the same re-baseline cost as OD-98.
- **Current doc choice:** **hash-exempt session state, declared**, logged per
  tick to `silt.jsonl` (and `gates/G16/silt_log.jsonl`) — deposits, decay,
  entries dropped below θ, `SILT_MAX` refusals.
- **Alternatives:** (b) hash-in, (c) cross-session persisted, which *requires*
  hash-in for replay to reproduce it (OD-109's option c).
- **Status:** OPEN. Ask: `OD-103: a / b / c`.

## OD-104 — what silt may bias: ignition only vs ignition + spread
- **Clash:** [engineering/10 §8](../../engineering/10-temperament-sleep.md) law 1
  makes the calm view **terminal** — nothing downstream may read it — enforced by
  `check_calm_view_is_terminal`, documented as **G13(k)** and recorded in
  `gate_report.json` as **G13.k.2**. Letting a topical statistic bias the walk is
  the recorded exception to that precedent, so it has to be nameable.
- **Current doc choice:** **ignition only** — the draft's own option (a) — while
  option (b)'s `primed` tag is implemented anyway, because G16(e) needs the tag
  to be gradeable whichever way the owner rules on spread priming. The bias
  amount is the unarmed fifth weight (OD-108), so at the species default even
  option (a) changes nothing.
- **Alternatives:** (b) ignition + spread (the `primed` path goes live), (c) silt
  biases nothing at all and is pure readout — lawful, and it would make §23's
  thesis weaker than its own draft.
- **Status:** OPEN. Ask: `OD-104: a / b / c`.

## OD-105 — how the crystallize detector treats primed frames
- **Clash:** if a frame primed by its own topic's warmth counts toward the
  recurrence that will nominate that topic, warmth launders itself into
  evidence — the accumulation half of §23.4's collision.
- **Current doc choice:** per-light `primed` tag on the frame log, and recurrence
  counting over **cold frames only** — the draft's recommendation, and it ships
  regardless of OD-104's ruling, because it is the conservative side of whichever
  way spread priming goes. Graded by **G16(e)** (crystallize isolation).
- **Alternatives:** (b) count primed frames (needs a separate laundering guard),
  (c) no tag at all (then G16(e) has nothing to assert).
- **Status:** OPEN. Ask: `OD-105: a / b / c`.

## OD-106 — silt deposit-source policy: full-weight parity vs a differential
- **Clash:** parity means a stranger's chatter warms the same table the owner's
  teaching does, which looks like the laundering §21.4 refuses; a differential
  means silt secretly tracks *authority*, which it was built not to do.
- **Current doc choice:** **full-weight parity** ([docs/04](../04-demo-trace.md)
  s29) — owner and non-owner lines deposit and light walks at the same weight,
  because silt tracks *thinking*, not teaching, and confers **zero** authority.
  The parity is bought back where it matters: G16(b) and G19(a) still show **no
  belief row moves** from a non-owner line.
- **Alternatives:** (b) a differential, which must name the laundering path it
  prevents — and none exists, which is why (a) shipped.
- **Status:** OPEN. Ask: `OD-106: keep a / b (name the path)`.

## OD-107 — arousal-scaled silt decay — **BANNED, not merely unused**
- **Clash:** §18.1's whole contract is that `a` changes which edges are *cheap*,
  and a decay rate is exactly such a knob; making silt age faster when hot would
  smuggle a coupling term back in after OD-21 cancelled the matrix.
- **Current doc choice:** arousal-scaled deposit and decay are **absent from the
  code** (§23.11), so they are not even a grid point in
  `gates/G16/silt_sweep.json`. Silt decays on the one logical clock
  (`clock.py`, R-29).
- **Alternatives:** (b) reopened only by an explicit owner ruling that also says
  which §18.1 modulation curve value it uses.
- **Status:** OPEN, with (b) banned until reopened.

## OD-108 — the fifth ignition weight's class under OD-87's split
- **Clash:** OD-87 ruled the four `IGN_WEIGHT_*` rows **TEMPERAMENT** draws. A
  fifth *draw* lengthens `DERIV_ORDER` from 18 to 19, bumps `seed_format`, and
  changes **every hash in the repository** — and B7's ruling says no sweep may
  ever touch a temperament row.
- **Current doc choice:** **SPECIES, and unarmed**: `IGN_WEIGHT_SILT` exists with
  live weight **0** at the species default, so every pre-existing gate stays
  byte-identical. G16's scenario installs the smallest non-zero step the row's
  grid can express, so the term is *seen* biasing the sampler at a declared point
  and never at an invented magnitude.
- **Alternatives:** (b) a fifth draw — lawful only as an owner ruling plus a
  `seed_format` bump and a re-baseline of every run since B0, which is the whole
  gate corpus; (c) delete the term and make silt read-only (weakens §23 to a
  log).
- **Status:** OPEN. Ask: `OD-108: a / b (accept the re-baseline) / c`.

## OD-109 — silt scope: session, per-conversation, or cross-session
- **Clash:** the draft's "not always cold" reading wants warmth to survive a
  session, which drags OD-103 (hash-in) and OD-127 (per-thread) in behind it.
- **Current doc choice:** **session-scoped, keyed by conversation** — the key
  exists from the first build so [docs/26](../26-world.md)'s per-thread option
  (OD-127) is a keying decision rather than a new mechanism. The default key is
  the single conversation every pre-existing scenario already is, which is why
  nothing already built notices.
- **Alternatives:** (b) one global key (then a busy server's threads average each
  other out), (c) cross-session persisted (requires hash-in per OD-103).
- **Status:** OPEN. Ask: `OD-109: a / b / c`.

## OD-110 — prov-link storage: stored rows vs derived at generation
- **Clash:** the draft **recommends stored** prov-link rows (byte-strict gates,
  citable as rows, weight in the registry), but a stored edge appends to every
  commit — and that is the row/edge byte-strictness OD-B just ruled for
  **G10.b.2**, which nine green gates record today.
- **Current doc choice:** **derived.** The event view is assembled from bytes that
  already exist (`row_id`, `written_at`, `prov_step`, `prov_session`, the commit's
  stamps with their `a_at`, the event log): no new row, no new edge, no new
  stored field — the §20-median family (R-28: computed at query, never stored).
  This is the one place the arc ships the *non*-recommended option, and the
  reason is the owner's standing requirement, not taste. Flipping OD-110 later is
  a migration that re-baselines those artifacts, said out loud here so nobody
  discovers it at B13.
- **Alternatives:** (b) stored links (the draft's rec; costs above), (c) no
  traversal at all (§24's thesis stays unbuilt).
- **Status:** OPEN. Ask: `OD-110: keep derived / b (accept the re-baseline) / c`.

## OD-111 — the registry entry for `prov-link` (§5.1 / R-12)
- **Clash:** a named edge kind needs a registry row, a never-write-path rule,
  never-FACT-eligibility, and confirmation that `prov_speaker = 2`'s scope
  covers it — four tables grow if the kind exists.
- **Current doc choice:** **no entry is made**, because OD-110 ships derived
  traversal and there is no edge to name. Its strength is the swept **SPECIES**
  row `PROV_LINK_TUG` (OD-114), and the scope check the draft asked for already
  passes: [engineering/01 §4](../../engineering/01-data-schemas.md) field 34 makes
  `2 = system-inferred` legal **only on `links`**, never FACT-eligible, never
  answerable — asserted live by **G17(e)**.
- **Alternatives:** (b) a `prov-link` kind now (only lawful if OD-110 flips to
  stored), (c) reuse the generic owned-link kind without a name (then the audit's
  trace cannot say which edge is a moment).
- **Status:** OPEN, blocked on OD-110. Ask: `OD-111: a / b (with OD-110: b)`.

## OD-112 — re-light stamping: does a traversed moment re-stamp its edge?
- **Clash:** re-lighting an aged edge *looks* like rehearsal, and rehearsal
  matters — but the license for idle re-stamping is **OPEN-24**
  (`IDLE_RESTAMP`, still unrulled), and the B7 batch's OD-88/OD-89 ruled stamp
  *tolerance* and the decay-deficit feedback, not this.
- **Current doc choice:** **re-lights never stamp, never bake, never bump
  confidence**, and this round adds **no stamping site at all**: the only stamps
  an idle walk writes are the ones `drive.update` already licenses mid-walk
  (engineering/04 §19 S.7), which `gates/G10/gate_report.json` records as
  `open_24_restamp: "off (03 §6's row is unrulled)"`. **G17(d)** is therefore a
  structural assert over the stamp arena, not a claim about a value this file has
  no ruling for.
- **Alternatives:** (b) re-light stamps (needs OPEN-24 ruled first), (c) re-light
  bakes — forbidden by §20.4 outright: a bake is bought only at formation by a
  teaching event.
- **Status:** OPEN, riding OPEN-24. Ask: `OD-112: a / b (rule OPEN-24 first)`.

## OD-113 — mixed-voice event views: per line or per frame, and does Grimm get one?
- **Clash:** per-frame views are cheaper and blur two speakers into one moment;
  denying Grimm views would make his lines uncitable, which §21.4's honest
  separation already refuses.
- **Current doc choice:** **one view per line**, and grimm lines get views —
  addressable, **zero authority**, no incoming belief link (§24.4 as drafted). A
  derived view's `prov_speaker` is `2` and never `1`, asserted by **G17(e)**.
- **Alternatives:** (b) per frame (cheaper, and it fuses two voices into one
  citable moment), (c) owner-only views (uncitable grimm history — a laundering
  path in the *read* direction).
- **Status:** OPEN. Ask: `OD-113: a / b / c`.

## OD-114 — `PROV_LINK_TUG`, `RELIGHT_MAX`, and whether the tug reads silt
- **Clash:** the origin tug is a walk weight (no value before its sweep), and
  whether it may read silt is OD-104's decision wearing a different hat.
- **Current doc choice:** `PROV_LINK_TUG` and `RELIGHT_MAX` are swept **SPECIES**
  rows at `UNMEASURED_PLACEHOLDER` with `hi` open per OD-83, so
  `gates/G17/prov_sweep.json` publishes a **refusal**; legs needing them run at the
  box's floor and label it. **The tug does not read silt** — unbuilt, with no
  dependency edge between the traversal and `si-silt`, so OD-104 cannot be
  reached around by way of a moment.
- **Alternatives:** (b) tug reads silt (needs OD-104 ruled and a new crate edge),
  (c) fixed non-swept values (forbidden by the no-value-before-sweep law).
- **Status:** OPEN. Ask: `OD-114: a / b (with OD-104) / c`.

## OD-115 — may idle ignition seed an event view?
- **Clash:** seeding from a moment is the most literal "involuntary recall" the
  design can print, and it changes §19's sampler, its seeded randomness law, and
  therefore G5's determinism contract.
- **Current doc choice:** **no.** Seeds stay hyperpoints and §19 is untouched;
  involuntary recall arrives through a **tug** on an already-seeded walk, which
  is what §24.3 claims and nothing more.
- **Alternatives:** (b) allow event-view seeds (a sampler change plus the §19.4
  "randomness lives in state, not time" re-derivation), (c) no tugs either (§24
  ships views as citations only — lawful, and weaker).
- **Status:** OPEN. Ask: `OD-115: a / b / c`.

## OD-116 — the shape constants: `R_MIN_SHAPE`, `DELTA_MIN_SHAPE`, `K_FILLER`, shape-scratch budget
- **Clash:** §25.4's generality bound is a number, and `K_filler` is the whole
  difference between "a move" and "one habit replayed"; the no-value law applies.
- **Current doc choice:** swept **SPECIES** rows at `UNMEASURED_PLACEHOLDER`, `hi`
  open per OD-83, so `gates/G18/shape_sweep.json` publishes a **refusal**;
  **G18(f)** (the generality bound) and **G18(e)** (the spam bound) run at the
  box's own floor and name it. `SKILLS`'s scratch budget reuses the §16.8 frame
  law's shape, not a second cap.
- **Alternatives:** as OD-97 (b)/(c)/(d).
- **Status:** OPEN. Ask: `OD-116: a / b / c / d`.

## OD-117 — skill candidate storage
- **Clash:** one question, two detectors: a second, differently-worded ruling
  here would let the arc be accused of two answers to the same I5 problem.
- **Current doc choice:** **rides OD-98 verbatim** — session-only, declared
  hash-exempt, logged to `skills.jsonl` — in its **own separate table**, not a
  second family inside the crystallize detector (OD-119's recommendation), which
  is what makes G18(b)'s deletion test independent of G15(b)'s.
- **Alternatives:** (b) its own hash ruling (no argument for it), (c) one merged
  candidate table (cheap, and it deletes the independent-diff evidence).
- **Status:** OPEN. Ask: `OD-117: a / b / c`.

## OD-118 — what installed skills may bias: composition order only, or ignition seeding too
- **Clash:** ignition seeding is sampler-tuning, i.e. OD-87's protected
  temperament territory, reached *around* through a habit instead of through a
  sweep — the exact class of move B7 ruled out.
- **Current doc choice:** **composition order only**, and the rejection is in the
  code, not the prose: there is no path from a skill row to the sampler, so the
  round-trip to `DERIV_ORDER` is structurally impossible. A skill is a read-side
  preference (§25.6) and **nothing executes it**.
- **Alternatives:** (b) allow skills to seed ignition (needs OD-87 reopened and a
  new law about habits), (c) skills bias nothing at all — then a ratified habit is
  a citation and nothing else, which OD-122's retirement path still supports.
- **Status:** OPEN. Ask: `OD-118: a / b / c`.

## OD-119 — module: a `skills` reader vs a second family inside the crystallize detector
- **Clash:** the two detectors are the same algorithm over different evidence, so
  one file is tempting; but one file makes "delete the machinery" a single diff
  that cannot tell the two mechanisms apart, and R-40 ("a skill that became code")
  wants the skill machinery isolated.
- **Current doc choice:** **separate crate `si-skills`**, same reader-only
  discipline as `si-crystallize`: it holds no handle on the write gate, so a
  missing dependency edge — not a review note — is what makes "a skill is a row,
  never a program" true (§25.11's point 3, the `si-sleep ↛ si-learn` proof style).
- **Alternatives:** (b) one detector file (cheaper, and the two deletion tests
  collapse into one), (c) inside `si-walk` (forbidden: that is the control path).
- **Status:** OPEN. Ask: `OD-119: a / b / c`.

## OD-120 — skill usage tracking: full §8 confidence legs vs read-side counters
- **Clash:** a habit that "loses confidence" implies a decay curve over skills,
  and §8's confidence legs are for beliefs; a decayed skill that still gets used
  is a hidden prior, while a skill with no tracking cannot be retired on
  evidence.
- **Current doc choice:** **read-side counters only**, in the same derived,
  never-stored family as the median (R-28), so a habit cannot lose confidence and
  cannot launder an authority bump through a counter. Retirement is §8
  **supersession on the row** (OD-122), and a retired skill's stats stay citable.
- **Alternatives:** (b) full §8 legs (skills become beliefs with nobody teaching
  them), (c) no tracking at all (then "this move has never helped" is
  unprintable).
- **Status:** OPEN. Ask: `OD-120: a / b / c`.

## OD-121 — the crystallize/skills mutual-exclusion seam and who owns an overlapping shape
- **Clash:** a recurrence set of one member-set is also a role-graph with one
  filler-set; if both detectors claim it, one act mints two nominations and the
  spam bounds (§22.7, §25.7) double-count against two queues.
- **Current doc choice:** a **forwarding rule inside the session**, not inside
  either detector: a shape whose recurrences share one filler-set is dropped from
  the shape table and its frames stay available to the content detector. Neither
  reader owns the other's territory, so G18(f) and G15(e) each keep their own
  bound.
- **Alternatives:** (b) content detector wins always (a genuine move could never
  be nominated), (c) shape detector wins always (the reverse), (d) both may
  claim, with a dedupe at the queue (then the queue's dedupe is silently doing
  the ownership decision).
- **Status:** OPEN. Ask: `OD-121: a / b / c / d`.

## OD-122 — skill retirement and the fate of a retired skill's stats
- **Clash:** an owner who ratified a move may decide it is harmful, and retiring
  it must not mean deleting history (I4) or silently leaving a bad preference in
  the walk's ordering.
- **Current doc choice:** retirement is an **ordinary owner supersession on the
  row** — the same gate, the same provenance, `superseded_at` stamped, successor
  pointer set — so a retired skill's row and its usage stats stay citable (I4:
  yes, per the draft). No delete path exists.
- **Alternatives:** (b) a dedicated retire verb (an eighth envelope semantic for
  something §8 already does), (c) no retirement (an owner cannot take back a
  ratified habit — refused).
- **Status:** OPEN. Ask: `OD-122: a / b / c`.

## OD-123 — the identity schema: `speaker_identity`, the `prov_speaker` extension, and where grant rows live
- **Clash:** the 80-byte envelope is frozen and **an added field would
  re-baseline every stored record since B1**; meanwhile tier (b) needs a trusted
  row's provenance to say *who*, and `prov_speaker` has only three literals
  (`owner | grimm | system-inferred`, OD-59's scope).
- **Current doc choice:** identity rides the **sense stream** — the event body and
  the transcript line, where a perturbation's metadata already lives — and
  `speaker_class` is computed at generation and **stored nowhere** (R-28). Grant
  rows are a **kind on the L3 `links` shelf** (an owner-act kind, §5.1's
  sanctioned shape: new kind, never new structure), written through the ordinary
  gates. Under the shipped tier (a) **no non-owner line ever writes a row**, so
  the `prov_speaker` extension is never reached: the enum stays three and the
  envelope stays 80 bytes. The `source` tag itself already ships
  (`si-core::SOURCE_LITERALS`, `Source::{Owner, Grimm}`), so V5 extends the stream
  and not the lattice; B9's supporter filters remain a separate pending stage.
- **Alternatives:** (b) a fourth `prov_speaker` literal or an identity field in
  the envelope (owner ruling + G5c schema diff + re-baseline), (c) a gated
  `grants` shelf (a new structure, against §5.1), (d) identity in `text_ref`
  interning (misuses a content slot for metadata).
- **Blocking:** nothing tonight — the question only bites when OD-124 grants
  tier (b).
- **Status:** OPEN. Ask: `OD-123: a / b (with OD-124: b) / c / d`.

## OD-124 — the tiers ruling: (a) world-as-subject, (b) granted trust, (c) full peer teaching
- **Clash:** the whole design's authority chain is "every row traces to an owner
  act"; opening the room to teachers breaks the audit trail that gates G1, G8,
  G12 and G17 are built to protect.
- **Current doc choice:** **born at (a)**, and only (a): `WORLD_TIERS` is a ruling
  row shipped **off**, so every non-owner line is grimm-class and the world is
  *experience* (crystallize nominates across many users' lines; only the owner's
  word mints). **(c) is rejected as written**, recorded here so nobody
  re-proposes it as an implementation detail: a class that teaches itself is
  provenance laundering with a bigger room (OD-36's live line took sense-only for
  Grimm). **(b) exists as machinery and is exercised by G19(c)/(d)/(h) inside
  its own scenario, never at the species default.**
- **Alternatives:** (b) at birth (hands out authority with no owner in the room),
  (c) peer teaching (rejected), (d) keep (a) and refuse to build (b)'s machinery
  (then G19 has no grant-trace leg, and the *law* would be untested).
- **Downstream:** OD-125, OD-128, OD-129, OD-130, OD-131 and OD-123's extension
  all sit behind this ruling.
- **Status:** OPEN. Ask: `OD-124: keep a / b (and the smallest-first grant order)`.

## OD-125 — grant semantics: shape, revocation, and a revoked user's standing
- **Clash:** a grant is an act of authority, so it must be citable and revocable
  without editing history; and after revocation, does a row written *while
  granted* still outrank a later stranger's line?
- **Current doc choice:** **coded, not granted.** The grant-row kind, the
  revocation law and tier-strict arbitration (owner > trusted > stranger-sense; a
  trusted correction supersedes only trusted rows, never owner rows) all exist
  and **G19(c)** gates them, because the machinery has to be honest before the
  first grant is possible. The birth state holds **zero** grant rows. Revocation
  is append-only supersession (class re-derives at the next tick; nothing is
  deleted), and **G19(c) asserts the forward effect** — no trusted-tier write
  succeeds after the revoke line. The **backward** half (standing of rows written
  while granted) ships as the draft's recommendation *not* implemented as a
  re-grade: rows keep their content and their provenance, and no rule retroactively
  demotes them, because re-deriving stored rows would be a migration over frozen
  bytes.
- **Alternatives:** (b) retroactive demotion of a revoked user's rows (needs the
  OD-110-style migration the owner's constraint forbids), (c) grants expire on a
  timer (a wall-clock trigger reaching state: I5/R-29 refuse it).
- **Status:** OPEN. Ask: `OD-125: a / b / c`.

## OD-126 — scheduled idle windows: owner-set windows on the logical axis
- **Clash:** the draft said "calendar", and a wall-clock calendar that reaches
  state breaks I5 and R-29 on its own; and a window that *causes* thought is a
  timer wearing a schedule, which §19.4's rate law exists to forbid.
- **Current doc choice:** windows are expressed in **logical ticks** and their
  only authority is **suppress** — closed means no idle tick, open does **not**
  mean an idle tick happens, because the quiet-and-empty-agenda law still decides
  (§19.1). Sleep is counter-triggered, so a window cannot make a night happen.
- **Alternatives:** (b) windows that can cause ignition (a fifth perturbation
  source, and it breaks G10's ignition law), (c) wall-clock windows (banned),
  (d) no windows (a busy server never yields an idle moment — the starvation
  problem G19(g) is about stays unsolved).
- **Status:** OPEN. Ask: `OD-126: a / b / c / d`.

## OD-127 — per-thread silt (OD-109 option c)
- **Clash:** simultaneous threads in one table average each other out, so the
  busy world needs per-conversation warmth — which is exactly the scope question
  OD-109 defers.
- **Current doc choice:** the **build already keys the silt table by
  conversation** (OD-109's default), so per-thread silt is a *keying* decision,
  not a new mechanism; global decay stays one clock over all tables. The draft's
  own note is preserved: this entry exists to keep anyone from re-deriving a
  second silt engine for the server case.
- **Alternatives:** (b) per-thread *and* per-thread decay (two clocks: R-29),
  (c) global key with a thread tag in the log (then "primed" is a lie).
- **Status:** OPEN. Ask: `OD-127: a / b / c`.

## OD-128 — ask routing and nomination visibility: owner lane vs public-with-owner-confirm
- **Clash:** a nomination is a question about experience, and the answer is the
  only act that can mint a belief — posting it to a public lane invites exactly
  the non-owner reply G15(d)/G19(d) refuse to write.
- **Current doc choice:** **owner lane.** Every nomination ask carries a lane
  field, the only legal value for a nomination is `owner`, and it is logged to
  `gates/G19/lane_log.jsonl`. The queue rule and the integer per-turn ceiling are
  untouched: **"a busier world may not buy more asks" is a leg** (G15(e) +
  G19(f)/(g)), not a suggestion.
- **Alternatives:** (b) public-with-owner-confirm (more experience, and a visible
  wrong answer from a stranger that the write path must discard while the panel
  shows the ask — two more greps for no new capability), (c) per-channel routing
  chosen by the drive (that is drive policy owning a security decision).
- **Status:** OPEN. Ask: `OD-128: a / b / c`.

## OD-129 — per-user plasticity keying and what a readout may say about a person
- **Clash:** "this user's corrections arrive hot, that one teaches calm" is
  §20's machinery re-keyed per identity — but a user-facing readout about another
  user is a belief about a person, and beliefs about people go through the same
  gates as beliefs about apples (§26.6).
- **Current doc choice:** **unbuilt.** Stamps and medians keep one global key
  tonight, so no per-user median exists to print and no seam leaks. The label
  ladder stays **voice-blind** (§9): per-identity keying is a *statistic* change,
  never a promotion path. §26.6's nomination kind ("this user's claims keep
  contradicting") is recorded as **listed, not built**, and a sentence about a
  person could only ever be owner-taught content.
- **Alternatives:** (b) key now (an identity dimension on every stamp for a server
  nobody is connected to, and it grows the derived-key family), (c) key
  per-thread rather than per-person (a different question, and it smells like the
  OD-127 confusion).
- **Downstream:** only reachable after OD-124 grants (b) or OD-123 extends
  `prov_speaker`.
- **Status:** OPEN. Ask: `OD-129: defer / b / c`.

## OD-130 — Grimm on the server
- **Clash:** the supporter was built to be outside the loop with zero authority;
  a multi-voice world makes him one voice among many, which reads as a demotion
  and is the opposite — §21.10's point.
- **Current doc choice:** **stays exactly as G12 and B9 leave him**: one tagged
  voice (`Source::Grimm`), zero authority, every gate passing with him
  disconnected. Nothing is demoted, nothing is promoted, and Mike's lean — **no
  grant for the supporter, ever** — is recorded as a **lean**, not a rule; this
  entry owns the ruling. §26.7's third option (his filters demoted to the
  template the stranger/trusted filters reuse) is unbuilt and would be OD-124's
  (b) arriving by the back door.
- **Alternatives:** (b) retire him when the server lands, (c) keep as one tagged
  voice (shipped), (d) demote his filters to the shared template (needs G19(h)).
- **Status:** OPEN. Ask: `OD-130: b / c / d`.

## OD-131 — S0 retention of third-party lines (the one honest law-bend on the table)
- **Clash:** I4 keeps every line verbatim forever, and on the owner's own server
  that is his policy call; a retention window for *stranger* lines is the only
  place a bend is even discussable (§26.8).
- **Current doc choice:** **keep everything** — no retention window exists in the
  code at all, so the bend is not merely declined but **absent**. A future digest
  or redact op would be an owner act with its own law, not a cleanup task.
- **Alternatives:** (b) an ops-side window over stranger S0 lines (the bend, ruled
  or not by the owner alone), (c) digest to hashes for everyone (breaks I4 for the
  owner too, which is the part nobody argued for).
- **Status:** OPEN. Ask: `OD-131: keep everything / b (rule the bend) / c`.

## OD-132 — the demo-trace fixture becomes multi-voice
- **Clash:** §4's walkthrough is the single-conversation proof, and the class law
  is only honest when strangers are in the room; but a live server fixture needs
  credentials, a token, and an owner who is awake.
- **Current doc choice:** the arc's fixture is **Act 7, s28–s35**
  ([docs/04](../04-demo-trace.md)) — **a scenario, not a renumbering** of s1–s27,
  so the existing trace keeps its identity and every gate that cites it survives.
  Tonight it runs as a **harness scenario** over
  `gates/G19/server_transcript.txt` (every line with its `source` tag *and* its
  identity), asserting the same first-commit moment: only the owner's line ever
  writes. The Discord adapter is a transport at the panel's edge, outside the
  loop, and is **not built here** — naming that gap is the honest move
  (§26.13's scope line).
- **Alternatives:** (b) a live-server walkthrough (needs the owner), (c) no
  multi-voice fixture (then G19(a) has no transcript and the class law is prose),
  (d) renumber §4 to include strangers (breaks every artifact citing s1–s27 —
  refused).
- **Status:** OPEN. Ask: `OD-132: a / b (when you are awake) / c / d`.

## OD-133 — does a confirmation-minted `cluster-point` count as a "taught link" for the stage ladder? *(tail seat — fold-discovered)*
- **Clash:** [docs/11](../11-stages.md)'s S2 criterion counts `links` rows
  **kind-blind**, and it was written for `P_DEF`/same-question teaching; a
  crystallize confirmation writes an ordinary owner-confirmed `links` row, so it
  satisfies that criterion — which may or may not be what the stage law means.
- **Current doc choice:** **tonight it counts, unchanged.** The build adds no
  discriminator, because adding an invisible kind-check to a stage criterion is a
  stage-law edit wearing a bug fix, and the owner — not the builder — decides
  whether an owner-nominated cluster may carry a rung. Stated in §11.3, §22.9 and
  §23.9; the seat is a **tail seat** because the drafts closed before it was
  found.
- **Alternatives:** (b) exclude crystallize-born links from stage counts (needs
  the kind in the criterion), (c) a separate S2 criterion for clusters (a new
  stage law), (d) accept and say so in §11 permanently.
- **Status:** OPEN. Ask: `OD-133: a (accept, publish it) / b / c / d`.

## OD-134 — how is G15(c) actually swept? *(tail seat — fold-discovered)*
- **Clash:** §22.3 bars `a` from the detector, but the detector reads **walk
  artifacts**, and the walk's settle-set is `a`-modulated by §20.1 — so a
  full-session `a` sweep can move the candidate table *through the walk*, and the
  leg would red for a reason that is not a cheat.
- **Current doc choice:** **reading (i), frame-trace-fixed** — feed the detector
  an identical frame trace under two `a` regimes and diff its table, plus the
  static dependency check (the detector imports no affect module). That tests the
  *ban*, which is what the leg is for. The **full-session sweep** (reading (ii),
  as literally drafted) would require an `a`-free walk, contradicting §18/§20, so
  it is recorded `UNMEASURED` next to OD-83's refusal. Stated on
  `gates/G15/`'s threshold line in [gates/README](../../gates/README.md).
- **Alternatives:** (b) full-session sweep as written (unsatisfiable — it is a
  spec defect, and OD-93's variant discipline routes exactly this shape), (c) two
  sweeps: (i) as the leg and (ii) as a published *observation* about walk
  sensitivity, which is the honest way to keep the number people will ask for.
- **Status:** OPEN. Ask: `OD-134: keep (i) / b / c (add the observation)`.

## OD-135 — what is `prov_speaker` for a **granted** (Trusted, non-owner, non-grimm) speaker? *(engineering-discovered while pricing G19(c))*
- **Clash:** [docs/26 §26.5](../26-world.md)'s tier table gives a Trusted
  line the right to teach, and the write path encodes a teacher in exactly three
  bytes: `prov_speaker ∈ {0 owner, 1 grimm, 2 system-inferred}`. A granted stranger
  is **none of those three**. Forcing it to `0` launders delegated authority into
  owner provenance (I6, R-35's neighborhood); forcing it to `2` makes a person's
  sentence indistinguishable from an inference; a fourth literal is a schema change
  the arc promised not to make ([12 §2.1](../../engineering/12-v2-arc.md)).
- **Current doc choice:** **the belief-row write is refused.** `Trusted` is a
  *derived* class over the frame body and the transcript (OD-123's shipped half),
  it may be read, cited and re-lit, and it may **not** mint a belief row while the
  byte has no honest value. G19(c)'s pass condition is therefore a **refusal** —
  `Refused{SpeakerNotRepresentable}`, zero bytes written, and `observed` prints
  OD-135 as its reason. The grant row itself (a `links` row recording the owner's
  act) is untouched: the owner's word about trust *is* owner-provenanced.
- **Alternatives:** (b) `prov_speaker = 3 granted` — one more value, a schema
  widening, and every pre-arc provenance consumer must learn to read it; (c) map
  granted → `2 system-inferred` and document the abuse (refused: it destroys the
  distinction the byte exists to keep); (d) keep the refusal and make granting
  owner-only *forever*, i.e. Trusted is a read-side class only (a law change in
  §26.5, not a build choice).
- **Status:** OPEN. Ask: `OD-135: keep the refusal / b (add the value) / c / d`.

## OD-136 — who mints `source: grimm` and a `speaker_identity`, once a real adapter exists? *(engineering-discovered while writing G19's fixture)*
- **Clash:** G19's legs need N voices, and the honest way to give them tonight is
  a **harness scenario** whose lines carry an `identity:` tag (OD-132). But the
  design's world has a Discord-like transport, and on that transport *anyone* can
  type `source: grimm` or claim an identity — the privilege table (docs/21 §21.3)
  is enforced at the **adapter's** edge, and no adapter is built here. Worse, the
  `/trust` command that would mint a grant is **not in the closed vocabulary**, so
  the arc cannot ship the parser without opening the vocabulary it promised to
  keep shut.
- **Current doc choice:** **not built, and said out loud.** Tonight the only
  minter of `grimm` and of a grant is the scenario file, checked into
  `harness/scenarios/` and pinned by hash in the report; a real adapter is a
  **fourth process** writing `sense_inbox.siro` through `si-supporter`'s seam
  ([12 §1.4](../../engineering/12-v2-arc.md)), outside the loop, and it is not in
  any stage of [11](../../engineering/11-build-plan.md). No `/trust` parser ships;
  G19's grant table is built by an owner *line*.
- **Alternatives:** (b) build the adapter and the `/trust` verb in this arc
  (vocabulary + G5c allowlist + a new credential surface — a different batch),
  (c) let the supporter seam authenticate (needs a key, i.e. a secret in the run
  directory), (d) rule that grants are only ever issued at the panel by the owner,
  which is a §26 law edit and removes the question.
- **Status:** OPEN. Ask: `OD-136: keep (scenario-only) / b / c / d`.

## OD-137 — 04 §6 gives kinds 4 and 5 a birth site; no shipped stage constructs them *(night regression, uncertainty pass)*
- **Clash:** [04 §6](../../engineering/04-algorithms.md)'s birth table says where
  each of the six signals is born. Kind 4 is **tick start, over the `questions`
  shelf**, magnitude the aged value via `Clock::recency`, class order 0 ("before
  any lighting"); kind 5 is **`si-walk::idle` when a cascade crosses its own live
  contradiction (§19.5)**. The build constructs 0 (`si-learn/gate.rs` step 3), 1
  (`si-learn` steps 8/9 and the serve in `si-session`), 2 (`si-learn` step 5) and 3
  (`si-walk`'s `settle_or_stop`) — and nothing anywhere constructs a 4 or a 5 except
  the enum's own `ALL`/`from_u8`. Yet the same files assert the event fires: §12
  step 2's L-C note says the unanswered ask "keeps aging and **keeps firing kind
  4**", and §23.5(d)'s pass condition repeats it. `check_signal_kinds` even names
  `si-session` as the "tick-start age" birth crate, and passes, because it restricts
  *where* an event may be built and never counts *which kinds* are.
- **Why no gate caught it:** every family grades relations between what runs. G19(d)
  already prints the absence — "**UNMEASURED as a signal**: `SignalKind::OpenQuestionAge`
  has no producer in this build" — and grades the ask staying open plus the agenda
  staying non-empty instead, which is what actually suppresses ignition. So a
  fully-green G1–G19 is compatible with two of the six doors never opening, which is
  precisely the reading [05 §23.7](../../engineering/05-gate-harness.md) warns
  against. This seat was found by *looking for* the gap, not by a red leg.
- **Why it is not a build-it-now:** a tick-start gain is an input to the one
  integrator on **every** tick that holds an open ask. It moves `a`, and with it
  conduction, bake, the ignition weights, the sleep trigger and the whole trajectory
  artifact — the arithmetic B2 shipped and G2–G13 + G15–G19 grade green. Turning it
  on is a re-baseline of shipped numbers, not an addition, and it needs a ruling on
  where "tick start" lands against §12's step 1, which already reads `a`.
- **Current doc choice:** **declared, unbuilt, and said out loud** — the enum and
  `GAIN_SIGNAL[4]`/`[5]` registry rows exist, the producers do not, and the leg that
  would have shown it names itself UNMEASURED rather than claiming a firing.
- **Alternatives:** (b) build both producers and re-grade the whole set (a
  deliberate arousal re-baseline, with its own stage and its own gate legs); (c)
  build kind 4 only and leave 5's contradiction-warmth unbuilt, which is already
  OPEN-28's undecided adjacency rule; (d) amend §6/§12/§23.5(d) so the age is a
  **read-side** quantity that never enters `a` — a law edit, and it also retires one
  of [04 §10](../../engineering/04-algorithms.md)'s six curve consumers, so
  "one curve, six consumers" becomes five; (e) keep it as-is through B13, where
  G5(a)'s trajectory hash forces the question either way.
- **Status:** OPEN. Ask: `OD-137: b (build both) / c (build 4) / d (read-side) / e (keep until B13)`.

## OD-138 — a BASE-only answer is capped at INFERENCE by law and has no render arm in the build *(raised by B10's `G6.b.4`)*
- **Clash:** [docs/05 §5.3](../05-substrate-tesseract.md)'s critical law says an answer
  drawn from L1/L2 **is** an answer — tagged `BASE`, capped at INFERENCE, and cited
  beside the OWNED one when the two disagree — and [07 §15](../../engineering/07-genome-build-plan.md)
  repeats it: a genome path "may settle and must come out labelled INFERENCE/BASE,
  capped, with both citations". [04 §21](../../engineering/04-algorithms.md) U.1 then
  says how an output is made: **content tokens must be a byte-subset of the cited
  row's `resp_surface`**. But U.2's row 2(b) is the only arm that would label a
  BASE-only settle, and it needs a citation; and the sealed relation record is
  `{a, b, edge_kind, weight, provenance}` — two handles and a slot, **no surface and
  no S-row** ([01 §5.3](../../engineering/01-data-schemas.md), 07 §11). The read path
  builds `cited` from live rows and takes the answer slot from it
  ([`crates/si-read/src/retrieve.rs:192`](../../crates/si-read/src/retrieve.rs)), so a
  route that lit only corpus points yields `cited = []` and **asks back**.
- **Measured at B10** (`gates/G6_engine_20261024-1904`, seed 0x51, the pinned 7-point /
  7-edge fixture): `what is warm?` → `route {owned: 0, base: 1}` → output
  `ask:what is warm`. `G6.b.4` is therefore UNMEASURED — the arm exists in the labeler
  and cannot be reached. The disagreement legs *do* work, because an OWNED row is
  present to render: `G6.b.1` `{owned: 1, base: 1}` → `answer:heat`, `G6.b.2` after the
  supersede `{owned: 1, base: 2}` → `answer:ice`.
- **Sub-question the ruling has to answer, not just the direction:** `cites` is
  `Vec<(u32, u32, u32)>` = `{node, round, edge}`
  ([`crates/si-session/src/lib.rs:347`](../../crates/si-session/src/lib.rs)), and its
  third field is *the arena offset of the edge that carried the point* — a genome arc
  has no arena offset, only a slot in the sealed CSR row. That is the shape the ruling
  has to name, and OD-142 below is where it already bites.
- **Build choice (B10):** leave the arm unreachable, name the leg UNMEASURED in the
  report with the reason, and change nothing else — the render law, the write gate and
  the labeler's other arms are all shipped and graded, and inventing a citation format
  while coding is exactly what this ledger is for.
- **Alternatives:** (b) mint a synthetic arena offset per genome edge at load — cheap,
  and it puts a number in a field that resolves against the edges arena, so every
  consumer of `cites` (G10.d.1, the audit export, B11's citation panel) inherits a
  address that names no edge; (c) **the two
  sealed tables already in the artifact** — cite `(genome point handle, §6 kind slot)`,
  render the surface from §5's sealed retrieval space (the fixture seals 8 keys /
  0 aliases) and name the relation from §6's verbatim kind table; this is what 07 §6
  says the kind byte *exists for* ("`[trace] via genome `IsA``"), and it makes a BASE
  answer speak a word the mind never lived — which is the config axis's whole point,
  but it also means an S-row-free surface, so U.1 needs a second source clause and
  G1's "both birth branches write zero rows" has to be re-read against it;
  (d) rule that a BASE-only route never answers — then [docs/05 §5.3](../05-substrate-tesseract.md)
  and 07 §15's "may settle and must come out labelled" both need amending, and `G6.b.4`
  becomes a graded *ask-back* rather than a gap; (e) materialise an S-row per corpus
  edge at wake — moves `rows_written` and the arena bytes every shipped count rests on.
- **Status:** OPEN. Ask: `OD-138: c (cite the sealed tables, amend U.1) / d (a BASE-only route never answers) / e (materialise rows)`. Blocks only `G6.b.4`; B11's viewer citation panel and B13's G5 label legs both want the answer.

## OD-139 — `DEGREE_TABLE_RANGE` is named by A7 and given no value anywhere *(raised by B10's M.7)*
- **Clash:** [07 §12](../../engineering/07-genome-build-plan.md)'s A7 asserts
  `max_degree ≤ DEGREE_TABLE_RANGE` "from the sealed header", and the failure it
  prevents is concrete: an out-of-range index in W.4's hub-damp reciprocal table at the
  first thought. But the constant appears in no table of
  [03](../../engineering/03-constants-registry.md) — it is named once, in that assert
  row — so the value has no documented owner.
- **Build choice (B10):** treat it exactly as §8 treats the tier boundaries — a
  **manifest authoring**, per corpus, not a registry row: `degree_table_range` is a
  required `Manifest` field with `deny_unknown_fields`
  ([`crates/si-store/src/build.rs:64`](../../crates/si-store/src/build.rs)), A7 asserts
  against it at M.7, and the loader re-reads it from the sealed header before a row
  decodes (`Frozen::max_degree`, [`crates/si-store/src/layers_frozen.rs:185`](../../crates/si-store/src/layers_frozen.rs)).
  The fixture authors **64** against a measured `max_degree` of **3**
  (`data/genome/fixture/out/90eae6244a63a99e/build_report.json`).
- **Why it still needs a ruling:** a per-manifest number means a real corpus sets its
  own bound, so W.4's table can be sized differently by two builds of the same binary —
  lawful for the tag (the header is sealed, §13 re-versions it) and unprecedented for
  anything the registry calls a LAW-shaped constant. If the hub-damp range is a law, it
  belongs in 03 with a class and a swept box.
- **Alternatives:** (b) registry row `DEGREE_TABLE_RANGE` with a `lo…hi`, and the
  manifest may only *narrow* it; (c) derive it as `max_degree + 1` at M.5, which makes
  A7 assert nothing (it can never fail) and hides the size of the table the walk pays
  for; (d) keep it manifest-authored, and add the value to 07 §12's text so the named
  constant has a documented home.
- **Status:** OPEN. Ask: `OD-139: keep (manifest-authored) / b (registry LAW, manifest narrows) / c (derived) / d (keep, and document the number in 07 §12)`.

## OD-140 — W.2's merged handle space puts corpus points in the ignition pool *(B10, found while wiring `G6.c.1`)*
- **Clash:** 04 §19 S.4 samples an ignition candidate from `graph.points()`, and W.2's
  merge makes that set every live endpoint **plus every genome point**
  ([`crates/si-walk/src/graph.rs:195`](../../crates/si-walk/src/graph.rs)) — because
  "the genome owns handles `0 … P_gen−1` of the same `u32` space". So the idle engine's
  candidate list is a corpus-sized list at CONFIG ENGINE.
- **What the build does:** nothing special, and that is the point — the five weight
  terms (recency, bake, decay-deficit, contradiction warmth, silt) all read *live*
  state, so a corpus point nothing has ever lit scores 0 and
  [`crates/si-walk/src/idle.rs:168`](../../crates/si-walk/src/idle.rs) skips it
  (`if weight == 0 { continue }`). Nothing ignites on an untouched genome. But once the
  THINK walk lights a corpus point, its `last_lit` is real, its recency term is
  non-zero, and a later idle round can **start** a cascade from a sealed node — a
  thought whose whole route is BASE, which is OD-138's unrenderable shape. No shipped
  leg covers this: G10 and G13 grade ignition at CONFIG SI, where there is no layer.
- **Why it is not a build-it-now:** the alternative is a branch on the source class of
  a candidate, and 07 §16.2 forbids exactly that — "the spread never branches on source
  class"; W.3's two-cursor run is the design's answer to keeping the merge
  class-blind. Adding a constant to cap corpus-point ignition fails the B7 batch's
  mechanism-check-first rule.
- **Current doc choice:** left structurally uniform, priced by the `weight == 0` skip,
  and **ungraded** — `G6.c.1` proves only that no genome arc is stamped (measured: 7
  genome arcs walked, 0 stamped, 12 handles scanned = 7 corpus + 5 live).
- **Alternatives:** (b) seed `last_lit` at load with nothing, as now, and rule that the
  recency term is the only corpus door, i.e. accept it and *name* it in 04 §19;
  (c) exclude `point < point_base()` from `candidates` behind W.2's own base — a
  read-side filter, not a branch on a flag byte, since the base is the class; (d) wait
  for OD-138's ruling, because if a BASE route can cite and render then an igniting
  corpus point is a *feature* (the dictionary thinking on its own) and if it cannot, the
  cascade is a wasted round.
- **Status:** OPEN. Ask: `OD-140: b (accept, and write it into 04 §19) / c (exclude below the base) / d (defer to OD-138)`.

## OD-141 — W.2's merge is free in bytes and O(P_gen) per thought; 07 §9 prices only the bytes *(B10's `G6.d.1`)*
- **Clash:** 07 §9 forbids one thing by name at W.5 — materialising a **merged edge
  arena** per wake, the 5.5·10⁸-byte figure the load budget is about — and the build
  complies: the artifact is read once, the web is two cursors over two address spaces,
  and nothing is copied ([`crates/si-store/src/frozen_engine.rs:8`](../../crates/si-store/src/frozen_engine.rs)).
  But `Graph::build_merged` runs **once per thought**, and its genome half is a loop
  that inserts all `P_gen` handles into the endpoint set every time
  ([`crates/si-walk/src/graph.rs:195`](../../crates/si-walk/src/graph.rs)) — a per-thought
  cost that no section of [06](../../engineering/06-resource-budget.md) states, because
  06 counts bytes and hops.
- **Why it is UNMEASURED and not a number:** the pinned fixture is **7 points /
  7 edges / 16 rows / 3 batches** (`build_report.json`), a build of 2712 artifact bytes
  over 1474 canonical bytes. Quoting headroom against the 10⁸-edge wall from those
  counts would be a fixture measurement labelled a corpus one — the honest-benchmarks
  rule this ledger's own OD-93 entry cites. So `G6.d.1` prints the leg's *inputs*
  (5 thoughts, 7 genome arcs walked, 8 live rows, 5 live points above the base) and
  `residency_headroom_vs_1e8: null`, and 06 stays UNMEASURED until a real-corpus ENGINE
  run exists.
- **Current doc choice:** correct and unproven — no cache, no hoist, no claim. W.5's
  Vulkan `genome_residency` (`HOST_VISIBLE | HOST_COHERENT` over exactly these bytes)
  arrives with the viewer stage at B11, which is where the shape may change anyway.
- **Alternatives:** (b) cache `0 … P_gen−1` in `Frozen` once per wake and union it
  per thought without re-inserting (a read-side memo, zero byte movement, and it keeps
  07 §13's tag comparison as the only staleness test); (c) leave it and measure at
  B12's first real-corpus run, where the sweep harness can print the per-thought merge
  cost as a statistic; (d) rule the point set is a *load-time* property of the graph, so
  `Graph` may hold the merged endpoints across thoughts — a law change, because the
  graph is rebuilt at a new `anchor` each tick and its endpoint set is only one of the
  three things that changes.
- **Status:** OPEN. Ask: `OD-141: b (memoise the range) / c (measure at B12 first) / d (cross-thought graph, needs 07 §9's own edit)`.

## OD-142 — `G10.d.1`'s two predicates have no BASE arm *(B10's nearest wall)*
- **Clash:** 05 §15's `G10.d.1` is the leg that proves the monologue is *a trace and not
  a re-walk*: every `{node, round, edge}` citation must name a point the store holds
  (`store.points().canonical(node)`) and, for `round > 0`, `edge` must land on an
  edges-arena slot boundary inside the arena
  ([`crates/si-cli/src/harness/gates.rs:3352`](../../crates/si-cli/src/harness/gates.rs)).
  Both predicates are true statements about a CONFIG SI mind. Neither is a statement
  about a CONFIG ENGINE one: the cascade the citation is read off runs over W.3's merged
  cursors ([`crates/si-walk/src/graph.rs:205`](../../crates/si-walk/src/graph.rs)), so an
  idle round can be carried by a genome arc, and the builder that turns that trace into
  `cites` copies `contribution.edge_off` **without consulting `contribution.genome`**
  ([`crates/si-session/src/lib.rs:2961`](../../crates/si-session/src/lib.rs)). A corpus
  point's handle is in L1, not in the live canonical table; a genome arc's address is a
  CSR slot, not an arena offset — and [`crates/si-walk/src/graph.rs:27`](../../crates/si-walk/src/graph.rs)
  states the contract the copier skips: "any consumer that hands an address to the store
  first asks `Arc::is_genome`".
- **What it would do:** the first ENGINE mind that thinks to itself while a corpus arc is
  in the trace reports "citation names a point the store does not hold" and "edge
  citation is not an arena slot" — a **red leg on a correct build**, which is the
  failure mode this ledger keeps calling the expensive one.
- **Why B10 shipped without touching it:** `G10` is a CONFIG SI gate and the `G6`
  scenario has no idle line, so no run in the graded set reaches the combination. The
  seat is recorded before the wall, not after it.
- **Build choice (B10):** nothing. The merge stays class-blind as 07 §16.2 requires, the
  leg stays as B7 shipped it, and the gap is named here.
- **Alternatives:** (b) widen the triple to `{node, round, edge, space}` — a real marker,
  and it moves the field every consumer reads, including the export lane and G10's own
  1:1 count check; (c) give `cites` the same `genome` flag `Contribution` already
  carries — the trace *knows*, so this is a faithful copy rather than an invention; (d)
  fix the leg instead of the record: resolve a node through `Frozen`'s §5 surface table
  when the session has a layer, and skip the arena check for a flagged contribution —
  the leg then proves the same thing in both configs and gains the BASE arm OD-138's
  ruling would need anyway; (e) forbid an idle cascade from crossing a genome arc, which
  is a branch on source class and 07 §16.2 forbids it.
- **Status:** OPEN. Ask: `OD-142: c (flag on the record) + d (arm on the leg) / b (widen the triple) / e (needs a 07 §16.2 exception first)`. Cheapest honest pair is (c)+(d), and it is the same decision as OD-138's citation shape — rule them together.

## OD-143 — the idle-ignition record has no slot in 08 §2.1's sealed table *(B11.1)*
- **Clash:** [08 §2.1](../../engineering/08-gui-observability.md)'s kind table is sealed
  at 0…20 and names no ignition record, while B7 shipped the idle ignition (I9) and
  12 §2.5's five-term weights are the only witness the ORGANS panel could draw for it.
  §2.1's own note refuses the cheap alternative — a "this was a dream" flag on kinds
  3/4/6/13 — so the record cannot hide inside a row the table does name.
- **Build choice:** `IGNITION: u8 = 21`, the first slot **above** the sealed table, with
  the displacement stated at every cite site
  ([`crates/si-core/src/eventlog.rs:44`](../../crates/si-core/src/eventlog.rs),
  [`crates/si-session/src/lib.rs:80`](../../crates/si-session/src/lib.rs)) and refused
  above it ([`crates/si-vulkan/src/frames.rs:143`](../../crates/si-vulkan/src/frames.rs)).
  `LAST_SEALED = 20` keeps DoD 1's "handles kinds 0–20" one list rather than two, the
  crate's own test counts 21 table rows plus this slot
  ([`crates/si-vulkan/tests/laws.rs:272`](../../crates/si-vulkan/tests/laws.rs)), and the
  replay lane leaves it out on purpose
  ([`crates/si-cli/src/harness/runner.rs:42`](../../crates/si-cli/src/harness/runner.rs)).
- **Alternatives:** (b) a flag on the round/settle kinds — §2.1's note forbids it;
  (c) take 18's reserved row — that seat is OD-67's counterfactual probe and B11 DoD 1
  asserts the parser *refuses* it; (d) rule that the ORGANS panel has no idle row, which
  leaves the most interesting record in the build undrawn.
- **Status:** OPEN. Ask: `OD-143: a (kind 21 as shipped) / d (no idle row)`.

## OD-144 — kind 7 has a table row, a panel column and no producer *(B11.2)*
- **Clash:** §2.1 row 7 is `median_used`, 20 §20.4's read-side median is the value that
  would fill it, and 04 §9's H — the median computation — has no producer anywhere in
  `src/`. So the row is reachable in the schema and empty in every run, and `GAMMA` is
  inert. This is OD-137's shape (kinds 4 and 5) arriving one row later.
- **Build choice:** the lane says so. The session writes the absence as a string
  ([`crates/si-session/src/lib.rs:2589`](../../crates/si-session/src/lib.rs):
  `"absent (OD-144: 04 §9's H is unbuilt, so GAMMA is inert)"`) and the panel prints a
  null as absent, never as zero
  ([`crates/si-vulkan/src/panels.rs:328`](../../crates/si-vulkan/src/panels.rs)).
- **Why not a number:** drawing 0 would be the expensive failure this ledger keeps
  naming — a panel that reports a measured zero for a quantity no code computes.
- **Alternatives:** (b) build H and emit 7 — a read-path change, and 11 §4 puts it in no
  shipped stage; (c) delete row 7 from §2.1 — a spec edit; (d) leave the column blank
  with no reason attached, which is what a silent workaround would look like.
- **Status:** OPEN. Ask: `OD-144: a (the string, as shipped) / b (build H, name its stage) / c (retire row 7)`.

## OD-145 — six doors, four frames: §2.1 row 9's "exactly two calls per thought" *(B11.2)*
- **Clash:** §2.1's `door_call` row says **exactly two** calls per thought and
  §5's panel rule prints that invariant next to the number
  ([`crates/si-vulkan/src/panels.rs:124`](../../crates/si-vulkan/src/panels.rs)), but the
  door table in 01/04 lists six doors and only four of them mint a frame — `prioritize`
  has none ([`crates/si-session/src/lib.rs:3403`](../../crates/si-session/src/lib.rs)).
  The panel's count and the invariant's number are therefore drawn from two different
  sets, which a reviewer reading the screen cannot see.
- **Build choice:** the invariant is kept as the *walk-side* statement 08 §5 makes it —
  the THINK walk makes two door calls, and those are the frames row 9 names
  ([`crates/si-session/src/lib.rs:2595`](../../crates/si-session/src/lib.rs)). The other
  doors are not walk calls, so the row's "exactly two" survives; the four-of-six gap is
  recorded here instead of being papered over by drawing six.
- **Alternatives:** (b) restate row 9 as "six doors, two walk calls each thought" — the
  truest sentence, and it edits the table; (c) emit a frame for all six doors, which
  changes the lane's volume and every replay hash; (d) print the door table on the panel
  as well, which needs a kind the table does not have.
- **Status:** OPEN. Ask: `OD-145: b (restate row 9) / a (walk-side reading, as shipped) / c (six frames)`.

## OD-146 — row 5's `citation` column is unfillable by law *(B11.2)*
- **Clash:** §2.1 row 5 (`signal_event`) has a `citation` column, and 01 §5.3 defines a
  `Signal` as kind plus its own value — "never a source and never a referent". So the
  column has no legal content: the viewer may not import `si-audit` (02 §10) and the
  loop may not widen `Signal` (R-28's social-channel rule).
- **Build choice:** the null is written **as a law, not an omission** —
  `citation: null` with the reason in the comment at
  [`crates/si-session/src/lib.rs:1957`](../../crates/si-session/src/lib.rs) — and the
  panel prints the frame's own `(t, thought)` pair in that column, so the row is still
  traceable to the act that produced it. No name appears anywhere: §18.1's labels live
  in `si-audit`.
- **Alternatives:** (b) delete the column from row 5 (a spec edit, and it hides that the
  schema was asked for something the law forbids); (c) let the labeler fill it — a banned
  edge for both processes; (d) print the referent handle, which is a citation without a
  resolver and reads as a claim the panel cannot support.
- **Status:** OPEN. Ask: `OD-146: a (explicit null + the pair, as shipped) / b (retire the column)`.

## OD-147 — where viewer SPIR-V comes from *(B11.4)*
- **Clash:** [00 §9](../../engineering/00-stack-workspace.md)'s allowlist is compute-only
  and closed — no `glslang`, no `shaderc`, no `spirv-reflect` row — while
  [08 §3.1](../../engineering/08-gui-observability.md) says the glyph atlas has "no build
  step", and a Vulkan viewer needs a vertex and a fragment module. **Nothing in
  `engineering/` states where those bytes come from.** That silence is the gap this entry
  exists to close, because the three obvious answers each break something already built.
- **Build choice:** [`crates/si-vulkan/src/spirv.rs`](../../crates/si-vulkan/src/spirv.rs)
  emits both stages as words. No `build.rs`, no system tool, no artifact — the opcode and
  enum numbers are transcribed from the Khronos grammar and the module's unit tests walk
  the word stream, so a miscounted instruction fails a CPU test instead of arriving as
  `NVVM compilation failed` from a driver at runtime. Both stages validate offline under
  `spirv-val --target-env vulkan1.0` (VulkanSDK 1.4.363.0 on this machine), and that is
  recorded as a **review-time** check, deliberately not a build dependency: a gate cannot
  assume a SDK exists on the machine that runs it.
- **Two defects the validator caught and a word-stream test now holds:** a sample
  instruction needs a loaded `OpTypeSampledImage` *value*, not the `UniformConstant`
  pointer the descriptor binds; and in SPIR-V 1.0–1.3 an `OpEntryPoint` interface may list
  only `Input`/`Output` variables, so the atlas is bound purely by its
  `DescriptorSet`/`Binding` decorations.
- **Alternatives:** (b) widen 00 §9 with a shader compiler — the allowlist is the law and
  a build-dependency on an unpinned system tool breaks gate reproducibility (G5's whole
  point); (c) ship precompiled `.spv` as an artifact — then it needs a registry slot, a
  hash and a loader, none of which 03 §1 has; (d) raise the module to SPIR-V 1.4+ to name
  the sampler in the interface — 1.0 is the oldest a Vulkan 1.0 driver must accept, and
  the runtime evidence says the driver's acceptance is the narrower gate; (e) write the
  shaders in GLSL and compile at first run — a second compiler to ship and a first-run
  cost the resource budget never priced.
- **Status:** OPEN (lean: a, as shipped). Ask: `OD-147: a (hand-built words) / b (widen 00 §9) / c (pinned artifact)`.

## OD-148 — who opens the artifacts: 00 §8 against 08 §2 and §8's table *(B11.3–B11.6)*
- **Clash:** [00 §8](../../engineering/00-stack-workspace.md) says the only files
  `si-vulkan` may open are process B's own lane tails, while
  [08 §2](../../engineering/08-gui-observability.md)'s table and §8's rows have the panels
  reading `SIst`, `mindlog.json`, `sleep.jsonl`, `fingerprint.txt` — and §7's VRAM panel
  needs the `SIgen` mapping, which is `si-store`'s file to map. Both sentences cannot be
  literal: reading `SIst` needs `si-store`, and
  [02 §10](../../engineering/02-module-contracts.md)'s graph has no such edge (00 §2: core
  ONLY).
- **Build choice:** the **Handover**. The launcher (`si-cli view`, which owns the schema
  crates) decodes each artifact and passes values; a panel *forwards* them
  ([`crates/si-vulkan/src/panels.rs:131`](../../crates/si-vulkan/src/panels.rs),
  [`crates/si-vulkan/src/surface.rs:18`](../../crates/si-vulkan/src/surface.rs)).
  `genome_residency` takes **bytes, never a path**
  ([`crates/si-vulkan/src/surface.rs:1733`](../../crates/si-vulkan/src/surface.rs)) for the
  same reason. The consequence is machine-checked and *narrower* than 05 §20's row:
  `check_gui_observer_only` proves the crate opens exactly one file — `sense_inbox.siro` —
  and reads zero artifacts, so process B holds no path to a lattice at all.
- **Alternatives:** (b) give the viewer the schema edges — breaks 00 §2 and every check
  that reads the crate graph; (c) keep §8 literal and let the viewer re-parse `SIst` — a
  second copy of the record schema outside the crate that owns it, which is how a schema
  drifts; (d) redefine process B's boundary to include its launcher, then 00 §8's sentence
  has to be rewritten to say what "process B" is.
- **Status:** OPEN. Ask: `OD-148: a (Handover as shipped; 08 §8's "reads" becomes "receives") / d (redefine the boundary)`.

## OD-149 — the surface is Win32, and every other platform is a refusal *(B11.4)*
- **Clash:** 00 §1 pins Windows and 08 never mentions portability, but a Vulkan surface is
  created from a raw window handle, so the code has to say what happens elsewhere.
- **Build choice:** `win32_handles` is the only implementation and the other branch names
  its own absence rather than falling back
  ([`crates/si-vulkan/src/surface.rs:750`](../../crates/si-vulkan/src/surface.rs):
  "no Win32 surface on this platform (OD-149)"). A silent fallback would be an unmeasured
  code path presented as a working one.
- **Alternatives:** (b) add xcb/wayland branches — unrunnable here, and a path no gate
  exercises is not a feature; (c) drop the `cfg` so the crate fails to build elsewhere —
  honest but noisier than a named refusal; (d) leave the `cfg` returning an empty
  extension list, which is the shape that turns a platform question into a driver error.
- **Status:** OPEN (lean: a). Ask: `OD-149: a (named refusal) / b (more backends, with a gate that runs one)`.

## OD-150 — a CONFIG SI session has published no `stream.siro`, so the launcher re-frames `mindlog.json` *(B11.5)*
- **Clash:** §2 names `stream.siro` as the output lane the viewer tails, and the lane
  writer is not in B11's creates-list
  ([11 §4](../../engineering/11-build-plan.md)), so the sessions this build can actually
  produce — `si-cli run` at CONFIG SI — have no lane file at all. The viewer would open a
  window over nothing.
- **Build choice:** `lane_bytes` reads `stream.siro` when it exists and otherwise re-frames
  the `mindlog.json` event lines **exactly as the loop wrote them**, since §2's own table
  already names `mindlog.json` as the source of kinds 0–16
  ([`crates/si-cli/src/bin/si-cli.rs:577`](../../crates/si-cli/src/bin/si-cli.rs)). The
  lane is then marked [`RECONSTRUCTED`](../../crates/si-vulkan/src/panels.rs) — §2.2's rule
  that a viewer starting mid-session says so — and the screen header carries the word, not
  just the log.
- **Alternatives:** (b) ship the lane writer in B11 — it belongs to B5/B7's organ surface
  and moving it here would change hashes the night regression already holds; (c) let the
  panels parse JSON — the schema would live in the display crate, which is OD-148's
  option (c) by another door; (d) refuse to start with no lane, which fails the DoD's own
  120-frame run.
- **Addendum (B11.7) — the live tail is no longer an unexercised branch.** A
  `si-harness run` directory does carry `stream.siro` (322 gate dirs hold one), so
  `si-cli view` was pointed at `runs/b11-live` and took the tail path for real:
  **25478 lane bytes, 60 frames, 8300 quads on the last frame, 310 rows not shown**,
  against `runs/b11-view`'s re-frame of **1538 bytes, 120 frames, 2483 quads, 22 rows not
  shown**. Both wrote `state_hash unchanged true`. §2's row is therefore the *fallback* it
  was written as, not the only path that ever ran. To keep a report from confusing the two,
  `viewer.log` now names the witness on its own line —
  `lane stream.siro live tail N bytes` or `lane mindlog.json re-framed N bytes`
  ([`crates/si-cli/src/bin/si-cli.rs:653`](../../crates/si-cli/src/bin/si-cli.rs)) — and
  `check_headless_run` refuses a `viewer.log` that names no lane.
- **Status:** OPEN. Ask: `OD-150: a (re-frame and label it, as shipped) / b (move the lane writer earlier)`.

## OD-151 — four pre-B11 scans read the viewer as a violation, and the dependency stamp moved *(B11.6)*
- **Clash:** B11's sources landed on checks written between B0 and B10, none of which
  assumed a process B existed. Four collisions, and every one of them is a *naming*
  question, not a reachability one:
  1. `check_speaker_class_derived` licensed `speaker_identity` in six files. The viewer
     decodes that field and mints the owner's byte for its single write (00 §7's privilege
     table), which is the framed sense body — so the list is 6 → 8, the two new names being
     `si-vulkan/src/frames.rs` and `surface.rs`.
  2. `check_windows_suppress_only` flagged any code line naming `Window`. winit's
     `Window`/`WindowEvent`/`WindowId`/`WindowAttributes` and the viewer's own
     `Outcome::Windowed` are a *different type with the same spelling* as OD-126's
     session-RAM interval.
  3. `check_config_guard` scanned raw text, so a doc comment stating that `SIgen` is
     **not** the viewer's file to map counted as a frozen handle, and 05 §20's
     seed/sleep scan flagged 08 §9's refusal table because its entries name the widgets
     they refuse ("sleep now", "re-roll the temperament").
- **Build choice:** each repair **adds** an assertion rather than dropping one. The viewer
  is now scanned for the interval's own spellings (`si_core::world`, `world::Window`,
  `window_denies`, `set_window`, `suppressed_by_window`) and any hit is still a stray; the
  two mention scans read code text with `/tests/` exempt, which is how the rest of this
  file already scans; the §9 table is exempt **by span** and the leg now fails outright if
  no refusal table is declared anywhere. `check_no_viewer_state_write` was added so the
  widened identity homes are policed from the other side as well.
- **Also republished:** `gates/deps_lock.md5`, 877d8a81035c187c4bc5f5d1c635a6c4 →
  2449f2c6c8828b801d361b6457a427d5, because B11 turns on the already-allowlisted
  `ash`/`winit`/`raw-window-handle` edges for `si-vulkan`. The precedent is B4, B14, B15
  and B17, each of which republished at its own stage commit; the allowlist itself did not
  widen.
- **Alternatives:** (b) reword every viewer label and doc comment to dodge the needles —
  the checks would then prove nothing; (c) license `si-vulkan` as a home for the interval
  type — OD-126's law is that only the idle gate sees it, and the split above keeps that
  law enforced by name; (d) delete the refusal table from `panels.rs` — §9 requires it on
  screen.
- **Status:** OPEN, leaning on the build. Ask: `OD-151: a (accept the four span/code-text repairs + the stamp) / b (restate 05 §20's three B11 rows so they name the viewer's real file set)`.

## OD-152 — the G19(g) `law` conjunction carries two identical operands *(surfaced, not caused, by B11's clippy pass)*
- **Fact:** [`crates/si-cli/src/harness/gates.rs:10578`](../../crates/si-cli/src/harness/gates.rs)
  reads `(!tick.suppressed || tick.denied) && (tick.denied || !tick.suppressed)` — the same
  disjunction with its operands commuted, so one conjunct is dead. `clippy::eq_op` denies
  it, which is why `cargo clippy --workspace --all-targets` exits 101 while
  `cargo test --workspace` and `si-harness check --all` (46/46) are green. Verified
  unchanged at HEAD, so the defect predates B11 and no B11 leg reads it.
- **Why B11 ships without touching it:** it is a leg of **G19**, and changing a conjunct
  changes what a shipped green gate proves. The fourth conjunct
  (`!(denied && quiet_grew && !suppressed)`) implies a denial *without* a suppression is
  legal, which decides whether the dead operand was meant as the converse implication
  (`!denied || suppressed`) or as a third distinct predicate. Guessing is how a gate starts
  grading its own typo, so the wall is recorded here rather than fixed in the dark at
  04:00 — B12/B13 own G19's legs.
- **Alternatives:** (b) drop the dead operand — behaviour-preserving by construction (a
  duplicated conjunct carries no information the first one lacks) and it silences clippy
  without changing a verdict; (c) restore the converse implication and re-run G19 to see
  whether the leg still holds, which is the only option that *strengthens* the gate;
  (d) `#[allow(clippy::eq_op)]`, which hides a question about the law behind an attribute.
- **Status:** RESOLVED **by derivation** (the build chose (b); reopen if you read the row as (c)).
  The question stopped being a design choice the moment the producer's own definition was read:
  [`crates/si-session/src/lib.rs:3076`](../../crates/si-session/src/lib.rs) is
  `suppressed_by_window = denied && agenda_clear && within_rate`, and ignition at `:3077` is
  `agenda_clear && within_rate && !denied`. So `suppressed ⇒ denied` (the surviving operand),
  `¬(ignited ∧ denied)` and the **guarded** converse `denied ∧ agenda_clear ∧ within_rate ⇒
  suppressed` (leg 4's `!(denied && quiet_grew && !suppressed)`, with `quiet_grew` the harness's
  proxy for the two preconditions) are all that the law says. An **unguarded** converse — option
  (c)'s `!denied || suppressed` — is false by that same definition: a window denial while the
  agenda is busy or the idle rate is spent suppresses nothing, because there was nothing there to
  suppress. Restoring it would have made the gate grade a lie.
- **Measured:** with the duplicate gone the leg's truth value is unchanged — `G19` at seed `0x51`
  is `PASS`, 17 legs / 1 unresolved, and the pre- and post-change runs
  (`gates/G19_si_20261024-2318`, `gates/G19_si_20261024-2321`) publish **identical**
  `state_hash.txt` (md5 `0f5af6b1b8674f9214e6566143683b48`), identical `transcript.txt`
  (`17f514c413387fd28ad8182c4c703be9`) and an identical `G19.g` `observed` block (B7: the
  identical hash *is* the pass). `cargo clippy --workspace --all-targets`, the check that OD-152
  blocked, now exits **0** (style warnings remain; no denied lint).

## OD-153 — `si-vulkan`'s `serde_json` edge is not in 00 §9's `used by` column *(B11.3)*
- **Clash:** 00 §9 lists `bytemuck` as used by vulkan and the `serde` pair as "the JSON
  format of every lane body", but its `used by` column does not name `si-vulkan` for
  `serde_json`. B11's frames decode a lane body and mint the inbox header, so the crate
  needs it — both live in
  [`crates/si-vulkan/src/frames.rs`](../../crates/si-vulkan/src/frames.rs), with one read
  of a decoded value in `panels.rs`. The cite site records the widening
  ([`crates/si-vulkan/Cargo.toml:13`](../../crates/si-vulkan/Cargo.toml)).
- **Build choice:** declare it and say so. No gate widened: `EXTERNAL_DEPS` in the harness
  already carries `serde_json` workspace-wide, so `check_deps` passed before and after;
  only §9's column is behind the build.
- **Alternatives:** (b) decode JSON in the launcher alone and hand the viewer
  pre-decoded strings — the purest reading of OD-148's Handover, and it would delete this
  edge; (c) amend §9's row to name `si-vulkan`.
- **Status:** OPEN. Ask: `OD-153: b (drop the edge, keep the Handover pure) / c (amend the row)`.


## OD-154 — the `/teach` feeder wrote a lane nothing read *(B11.7)*
- **Clash:** [09 §10](../../engineering/09-teach-channel.md) builds `si-cli feed` as a pure
  inbox writer and [09 §8](../../engineering/09-teach-channel.md) says "everything that
  happens afterwards is the ordinary loop". But no part of the ordinary loop read
  `sense_inbox.siro`: `si-cli view` opens it to *write* the owner's keystrokes
  (08 §1's OBSERVER-ONLY write), and the harness runner drives lines through `tick` from a
  transcript. So a fed line was a frame in a file and nothing more.
- **Measured before the fix:** two `/teach` lines fed into `runs/feed-gap`, then
  `si-cli run --seed 0x51` in that directory — `c9654e12086696a0aae43a459833b5ec`; and a
  fresh directory with nothing fed (`runs/feed-gap-control`) — the *same*
  `c9654e12086696a0aae43a459833b5ec`. A feeder that cannot change the state is exactly the
  failure shape `G1.u.2` exists to catch, and it was caught by running the two arms instead
  of by reading the spec.
- **Build choice:** the drain in `run_session`, ahead of the transcript block — a lane that
  already arrived is earlier in logical time than lines typed at this invocation
  ([`crates/si-cli/src/bin/si-cli.rs:194`](../../crates/si-cli/src/bin/si-cli.rs)). Each
  frame goes through `tick_frame`, the same steps 1–13 a keystroke takes, one tick each, in
  file order; a refusal prints what the lane said and applies nothing (01 §9). **Measured
  after:** `runs/g1u-fed` and `runs/g1u-typed` both settle at
  `b71cf552d1219f4cfb8caeff27eb40c8`, 2 rows committed, and their `mindlog.json` differ on
  exactly one line — `"run_id"` — which is also the whole of their 2-byte size difference.
  The CLI half of `G1.u.2` is now a real comparison, and `check_curriculum_is_inbox_only`
  got a reader leg so it cannot silently become a writer-only proof again.
- **The refusal branch is measured against the same control.** A lane whose container
  version is rolled back to 1, and a v2 lane whose frame body is 6 bytes (no word beside
  the 12-byte transport prefix), both print their own name and both settle at the
  **nothing-fed control digest** `c9654e12086696a0aae43a459833b5ec`. "No frame was applied"
  is therefore an observed equality, not a promise inside an error string: a refused lane
  is indistinguishable from an absent one.
- **Still open in it:** `G1.u.2` names `transcript.txt` byte-identity too, and `si-cli run`
  writes four files — `SIst`, `state_hash.txt`, `fingerprint.txt`, `mindlog.json`
  ([`crates/si-cli/src/bin/si-cli.rs:261`](../../crates/si-cli/src/bin/si-cli.rs), through
  [`crates/si-cli/src/bin/si-cli.rs:281`](../../crates/si-cli/src/bin/si-cli.rs)).
  `transcript.txt` is `Session::write_run_dir`'s
  ([`crates/si-session/src/lib.rs:3587`](../../crates/si-session/src/lib.rs)), which no CLI
  arm calls. So the transcript leg of `G1.u.2` is **UNMEASURED from the CLI**; the two arms
  above were compared on `state_hash.txt` plus a line-by-line `mindlog.json` diff.
- **Alternatives:** (b) transcript first, inbox after — a fed line would then sort *after*
  lines typed at the same invocation, contradicting the lane's own `t`; (c) refuse when
  both a lane and a transcript are present — honest but it makes `G1.u.2`'s two arms
  mutually exclusive by construction, which is the opposite of the test; (d) a `listen`
  verb that tails the inbox — a second loop, and 09 §8's point is that no second loop is
  needed; (e) let `feed` tick the lines itself — that is R-11/R-12 and the check that names
  them.
- **Status:** OPEN. Ask: `OD-154: a (drain ahead of the transcript, as shipped) + rule the
  transcript.txt leg: (i) si-cli run adopts write_run_dir / (ii) G1.u.2 measures inside the
  harness runner, which already writes both lanes`.

## OD-155 — a settled run's `state_hash.txt` is N lines; a key-per-line log holds one *(B11.7)*
- **Fact:** `si-cli run` writes `format!("{}\n", state_hash.hex())` — one line
  ([`crates/si-cli/src/bin/si-cli.rs:279`](../../crates/si-cli/src/bin/si-cli.rs)) — while
  the harness runner joins per-section digests with `\n`, so a G13 run's file is three
  lines (measured: `runs/b11-live/state_hash.txt`, 3 lines:
  `7418cbb90fbfd97682c05ce582aa4a56 / 9e2e46636e0f383e1aa8fe8804abf43f / 61d0d0c60c29bd28f8492d1bbb23d6fb`).
- **The defect this caused, and how it was found:** `viewer.log` is key-per-line and
  `check_headless_run` parses one value per key, so writing the file text raw put two
  unlabelled digest lines into a five-line log and the check compared a single line against
  the whole file. It only showed up by pointing the viewer at a real gate run rather than at
  a directory `si-cli run` had just made — the intermediate failure printed the mismatch
  (`before 7418cbb9…` against three digests on disk).
- **Build choice:** fold. `read_state_hash` joins trimmed, non-empty lines with spaces and
  `check_headless_run` normalises the file and the log value with the same rule, so the
  digest set is compared in file order
  ([`crates/si-cli/src/bin/si-cli.rs:610`](../../crates/si-cli/src/bin/si-cli.rs)). Only the
  separator changes; every digest stays byte-identical. Consequently the leg says
  **"digest-for-digest in file order"**, not "byte-identical", because after an N-line run
  the log value and the file are deliberately not the same bytes.
- **Alternatives:** (b) one key per section (`state_hash before 1 … 2 …`) — the log's shape
  then depends on the section count, and the parser has to know it; (c) make `si-cli run`
  always write a flattened single line — that changes what an N-section artifact's hash file
  means to every gate that reads it; (d) keep raw newlines and teach the parser section
  boundaries — 03's schema leaking into a display log, which is OD-148's option (c) again.
- **Status:** OPEN. Ask: `OD-155: a (fold in the writer and the checker, as shipped) / b (per-section keys)`.

## OD-156 — a lane cut mid-append: the loop's reader tolerates it, the display's refuses *(B11.7)*
- **Clash:** `Session::read_sense_inbox` bailed when a frame ran past its payload. §2.2's
  first row makes a killed viewer a no-op for the loop, and §1's HEADLESS law says the loop
  must not notice its observer — yet that bail was reachable: the viewer's single write
  appends to `sense_inbox.siro` while a window is alive, and a window killed mid-append
  leaves the tail of a frame on disk. The next wake would then have refused to run **because
  a display died**: an observer with a veto over the loop. The same shape in the display
  crate has the opposite law, since the loop publishes its lanes with `write_atomic` — a
  torn `stream.siro` is a corrupt artifact, not a concurrent append.
- **Build choice:** the asymmetry, named on both sides. The reader stops at the last whole
  frame and returns `InboxOutcome::Accepted { frames, torn_tail_bytes }`
  ([`crates/si-session/src/lib.rs:609`](../../crates/si-session/src/lib.rs), walk at
  [`crates/si-session/src/lib.rs:1051`](../../crates/si-session/src/lib.rs)); the run prints
  the count as its own line — "were never written and are not heard" — and applies no
  invented frame. `si-vulkan/src/frames.rs:110` keeps its `bail!` unchanged. Proved by
  [`crates/si-session/tests/world.rs`](../../crates/si-session/tests/world.rs): two frames
  cut mid-body read as 1 frame plus `4 + len(b) - half` tail bytes, and a cut that leaves a
  2-byte stub reads as 1 frame plus 2 — the while-guard, not the break, is what a stub hits,
  which the test now records rather than assumes.
- **Measured end-to-end, not only in a unit test:** the 216-byte fed lane with its last 5
  bytes absent hears 1 frame, names **70 torn tail bytes** and settles at
  `fa31c2027d1a2716a7af4ee03d88a9f4` — the state of one teach, which is neither the
  two-frame state nor a refusal; a 32-byte absence names 43 and settles the same digest,
  because the count starts at the last whole frame and includes its length prefix.
- **Alternatives:** (b) truncate the lane on read — an observer-owned write into loop data,
  the same door as OD-150's option (c); (c) keep the bail and accept that a rare kill
  breaks a wake — a HEADLESS violation that only ever shows in the field; (d) tolerate in
  both readers — the panel would then draw a half frame as if it were whole, and §2.1's kind
  table is a contract about whole frames.
- **Status:** OPEN. Ask: `OD-156: a (reader tolerates and names the tail; display refuses) / c (bail everywhere, accept the coupling)`.

## OD-157 — `G1.u` ships seven verdicts for the five rows §6(u) names *(G1.u step 3)*
- **Clash:** [05 §6(u)](../../engineering/05-gate-harness.md) lists `G1.u.1`…`G1.u.5`, and
  [09 §10](../../engineering/09-teach-channel.md) mirrors it. Two of those rows carry a
  clause this build has no producer or threshold for: `u.4`'s second half ("after `k` served
  recalls exactly `k` promotion events, each naming a serving step id") belongs to
  `INFERENCE_PROMOTION`, a **Ruling** row that ships `off` — the same wall `G13.i.2` names —
  and `u.5`'s second half ("the birth cohort is visible because `a` *stayed high*") names no
  number anywhere in the registry. Setting one while coding is exactly the ad-hoc cap
  **OD-33**'s ruling owns, so the build cannot supply it.
- **Build choice:** split each row into the half that has a witness and the half that does
  not, following the standing precedent `G13.i.1` (graded) + `G13.i.2` (`open()`, OD-58's
  conditional). `G1.u.4` grades "every curriculum row reads `gain_kind = asserted`" and
  `G1.u.4b` is unmeasured naming OD-58; `G1.u.5` grades "every stamped edge's `bake`
  recomputes as `bake_q12(a_at, B_MAX, K_TANH)`" and `G1.u.5b` is unmeasured naming OD-33,
  with the measured range reported rather than judged. A gate's `pass` stays "no criterion
  has verdict `fail`", so the family is honest at 28 legs / 4 unresolved instead of quietly
  24 / 2.
- **Measured (seed `0x51`, V0, `gates/G1_si_20261024-2300`):** the checked-in 200-line
  curriculum (`harness/scenarios/g01_u_curriculum_200.txt`, digest
  `7609e1cb88098b8af15c6e14d92042c8`) reaches `u.3` as 200 lane frames → 50 uncoversed rows
  with `repeats` summing to 200, shelves `{4: 50, 8: 200}` and `y` exactly `1..=50` / `1..=200`;
  `u.4` reads 50 rows all `gain_kind = 0` (asserted) at `prov_kind = 4`; `u.5` recomputes
  50 of 50 stamped edges with 0 mismatches, `a_at` in `[0, 212]` and `bake` in `[0, 32768]`
  against `B_MAX = K_TANH = 32768`.
- **Alternatives:** (b) mark the two clauses `pass` on the strength of the first half — the
  silent-fold the ledger exists to forbid; (c) invent an `a` threshold (say "min ≥ 128") —
  a new constant with no registry row and no ruling, and it would grade OD-33 instead of
  asking it; (d) drop the two clauses from the harness — 05 §6(u) then has no witness at all.
- **Status:** OPEN. Ask: `OD-157: a (split rows into graded + open halves, as shipped) / c (rule a threshold for "a stayed high" and I fold u.5b into u.5)`.

## OD-158 — a refusal is free in state and still owes 08 §2.1 one visible frame *(G1.u step 3)*
- **Clash:** [05 §17's (b)](../../engineering/05-gate-harness.md) prices a seam rejection as
  "**zero** state change — no signal row, no `a` movement, no stamp, no row — and exactly one
  gate-log line". `G12.b.2` operationalised that as "the step owns **no** event", which was
  true until B11.2 shipped [08 §2.1](../../engineering/08-gui-observability.md)'s kind 17:
  the seam now emits `seam_reject` so a rejected emission is *visible in CHAT* — §7's own
  requirement, "a silently dropped voice would hide the whole of §21". The two spec rows are
  consistent with each other; the harness clause was the over-strict one, and it went red
  with 08 §2.1 obeyed rather than R-31 broken.
- **Build choice:** re-read the clause against the row it grades. The state clauses are
  untouched (boundary pair equal, `a` unmoved, `rows_written = 0`, `reached_loop = false`,
  one gate-log frame), and the event clause becomes **"the step's only event is its own
  `seam_reject` frame"** — every other event a line pays for (sense echo, decoded, stage,
  round, `a` update, commit) is still forbidden. Measured at
  `event_kinds: [17]` for the drift step in `gates/G12_si_20261024-2301`, which is now green;
  this is the correction of an over-strict leg, not a weakening of the design, in the same
  shape as OD-88's stamp-arena amendment.
- **What the leg still does not price:** the event log is not in the boundary pair, so a
  refusal's own record is invisible to `G12.b.2`'s state comparison. If the owner wants "free"
  to reach the durable hash too, that is a new clause, not this one.
- **Alternatives:** (b) delete kind 17's emission for layer-2 rejections — 08 §7's CHAT panel
  then shows nothing where the law says a rejected line must be visible; (c) keep the leg
  red and call it B11.2's regression — the build would then be graded against the harness's
  wording instead of the spec's rows; (d) fold the clause into `frame_count == 1` only —
  the gate log and the event log are different artifacts, and the leg would stop seeing what
  the step actually costs.
- **Status:** OPEN. Ask: `OD-158: a (free in state + exactly its own seam_reject frame, as shipped) / b (a refusal emits no frame, and 08 §7's visible rejected line goes unmeasured)`.

## OD-159 — `G1.u.2` names *the feeder* and two artifacts; the harness grades one writer, one real lane file and the feeder's own tests *(G1.u step 4)*
- **Clash:** [09 §10's `u.2` row](../../engineering/09-teach-channel.md) reads "(a) fed
  through **the feeder** into a fresh session and (b) typed line by line into a second fresh
  session ⇒ **byte-identical `state_hash.txt`** and identical `transcript.txt`". Taken as a
  process recipe, that row asks the harness to run the `si-cli feed` *binary* and compare two
  run directories. Taken as the law it is written under — 09 §8's "one writer, one reader,
  never a harness branch" — the row's subject is that a fed lane and a typed lane reach the
  same state through the same code. The two readings are not the same test, and the build had
  to pick one.
- **Build choice:** grade the second reading as far as it can be graded, and give the first
  reading's missing halves their own witnesses rather than a silent fold.
  1. **One writer, one reader, real transport.** `G1.u.2`'s fed lane is now written to the run
     directory as `g1u2_fed_lane.siro` and **read back off the disk** before the loop hears it,
     so the bytes graded travel the same file the owner's `si-cli feed` writes. Measured at seed
     `0x51`, V0, `gates/G1_si_20261024-2311`: 439 bytes, `lane_file_survived_the_disk = true`,
     6 frames written / 6 heard, `torn_tail_bytes = 0`, fed hash
     `d0dccedede4a625150194bae119077f6` = typed hash, and the one-line-short control at
     `26736a8fdb41e1c1df28a2b86535ffac` still differs.
  2. **The feeder's own file handling, tested by the code that owns it.** The body of
     `feed_inbox` stays in `crates/si-cli/src/bin/si-cli.rs` — where
     `check_curriculum_is_inbox_only` reads it, because that check grades 09 §10's isolation
     half ("opens exactly two files, reaches no store/learn/walk symbol") **from that file's
     text** — and gains three unit tests in the same file: the filter (header comment and blank
     lines dropped, trailing spaces trimmed, order kept, and the directory holds only the
     curriculum file and the lane), the append (a second feed lands behind the lane already
     written, never in front of it), and the strict walk (a last frame cut 3 bytes short is
     refused with the named error `the lane on disk is torn`, and the lane's bytes are
     **unchanged** after the refusal — measured, `cargo test --bin si-cli`: 3 passed).
  3. **What is still not graded:** no test spawns `si-cli feed`. `grep -rn "Command::new"
     crates/ --include=*.rs` returns **0** hits workspace-wide, so a cross-process leg is a new
     mechanism — a built-binary path, a run directory the harness owns and writes `SIst` into,
     and a check that a gate's red does not just mean "the binary was not built".
- **Why not the other readings:** (b) move `feed_inbox` into the `si_cli` lib so the gate calls
  the producer's body — the check would then grade a file that also holds the harness's learn,
  walk and store symbols, and 09 §10's isolation half stops being enforceable by text; (c) give
  the harness a subprocess — 0 hits of precedent, and a gate that fails because `target/debug`
  is stale is a gate that measures the build machine; (d) leave u.2 as the pure in-process pair
  and the feeder's file handling untested — the filter, the append and the torn-lane refusal
  would then have no witness at all, which is the fold this ledger exists to forbid.
- **Status:** OPEN. Ask: `OD-159: a (as shipped: in-process one-writer equivalence + the lane read back off the run dir + the feeder's own file tests) / c (authorise a cross-process feeder leg, and I build the binary-path mechanism and its check)`.

## OD-160 — the arc's reader-crate checks left `si-crystallize`'s DoD fixture nowhere to sit; the build put it inline and shared the `#[cfg(test)]` exemption *(step 6)*
- **Clash:** [11 §B14 DoD 1](../engineering/11-build-plan.md) asks
  `cargo test -p si-core -p si-walk -p si-crystallize` to assert "the four rules fire in
  order" and "a primed frame increments no recurrence". Both need two things the two C14
  checks police in **opposite** scopes: a store fixture (`append_row`, a `&mut Lattice` —
  the only way 02 §2 writes a member row and its edges) and hand-built frame scratch
  (`FrameLog`, `CoLight {`). `check_ghost_is_frame_scoped` denies a `CoLight` construction
  outside `crates/si-walk/src/` and denies naming `FrameLog` outside four src paths — and
  it already cuts its scan at `#[cfg(test)]` with the stated reason "A unit test … builds
  values of it on the stack; that is the opposite of keeping one". `check_crystallize_no_commit`
  scanned **all** of `crates/si-crystallize/src/` and `crates/si-skills/src/`, test module
  included. So the fixture in `tests/` failed the ghost check and the fixture inline failed
  the no-commit check: DoD 1 and DoD 3 (`check_crystallize_no_commit` green from `--stage
  B14`) could not both be satisfied without deciding which scan is the shipped one.
- **Build choice:** the fixture is inline (`#[cfg(test)] mod tests` in
  `crates/si-crystallize/src/lib.rs` — the idiom `si-silt` and `si-skills` already use), and
  `check_crystallize_no_commit` got the *same* `#[cfg(test)]` cut its sibling applies, with
  the reasoning written at the scan site. What the check polices is the shipped detector's
  write capability; a `#[cfg(test)]` module compiles into no binary, and 12 §1.1's actual law
  — the missing `si-learn`/`si-affect`/`si-walk` **edge** — is untouched by either choice and
  is now asserted by a test, not only by a check.
- **Measured:** the exemption was proved, not assumed. A temporary
  `fn probe(lattice: &mut Lattice) { let _ = lattice.append_row(...) }` placed **above** the
  marker made the check fail and name both needles —
  `crates/si-crystallize/src/lib.rs:34: &mut Lattice`, `:35: append_row` — and it was removed
  once seen. After: `si-harness check --all` **46 checks, PASS**, placeholders 41, disabled 0
  (`gates/checks/stage-B13.json`); `cargo test --workspace` **336 passed / 0 failed** (was
  326 — 4 crystallize rules, 2 graph snapshots, 1 silt `THETA_SILT` branch, 3 frame-scratch);
  `cargo clippy --workspace --all-targets` exit **0**, 154 warnings, unchanged, and none in a
  new file; **16/16 gates PASS at seed `0x51`**, 348 criteria, **0 fails**, 55 unresolved,
  the same counts as the run before this step.
- **Why not the other readings:** (b) widen `check_ghost_is_frame_scoped` to exempt
  `crates/*/tests/` instead — then that crate's integration tests could keep a scratch value
  in a struct field and the `KEEPERS`/field-list policing would not see it, which is the half
  of the law that has no second witness; (c) build the frames through `FrameLog::from_json`
  with a `serde_json` dev-dependency — it names `FrameLog` either way, so it still trips the
  scope ban, and it puts a dev-dependency on a crate 12 §1.1 lists as `core, store ONLY`;
  (d) leave the four rules to G15 alone — G15's legs run at the registry floors, where rule
  4's `AlreadyBound`, `InsideDelta` and the `primed` branch never fire, so "in order" would
  be claimed and not seen.
- **Status:** RESOLVED by the build (fixture inline, exemption shared). Reopen with `b` if you
  want the arc's reader tests **outside** `src/`, which needs the ghost check widened.

## OD-161 — 05 §19(e)'s "every box corner is reachable" is false arithmetic for 16 of 36 corners *(B12's panel pass, `G14.e.3`)*
- **Where the law is.** [05 §19 leg (e)](../../engineering/05-gate-harness.md) —
  `G14.e.3` *every box corner is reachable by some panel seed, or the panel is
  widened* — against [03 §7](../../engineering/03-constants-registry.md)'s
  `seed_panel` size row and §4.1 step 4 (*"if any panel point fails, the box is
  wrong, not the seed … re-sampling the panel until something passes is the
  hand-picked-off-sweep failure wearing a statistics costume"*).
- **The finding.** The derivation is one `next_q12()` per row
  ([`si-core/src/rng.rs:69`](../../crates/si-core/src/rng.rs)) — a **Q12**
  fraction, so each row has an alphabet of exactly **4096** values — mapped by
  `map_fraction(fraction, lo, hi, grid)`
  ([`si-core/src/quant.rs:57`](../../crates/si-core/src/quant.rs)). For the
  sixteen rows declared `0..65535, grid 1` the largest product the arithmetic can
  form is `(65535 · 4095 + 2048) >> 12 = 65520`, so **`hi = 65535` is not
  draw-able by any `u64`, not merely un-draw-able by the panel**. Counted
  exhaustively (not sampled) over the whole alphabet: **16 of the 18 rows' `hi`
  corners are unreachable**, so 16 of 36 corners fail; only `LAMBDA_A`
  (`1..255`) and `ASK_CAP_PER_INPUT` (`1..4`) reach both of theirs. The rows
  never leave their box and never fall between grid points — §19(e)'s first two
  clauses pass at every seed — so the unreachable half is a *declaration*
  problem, not a draw problem, and widening the panel cannot fix it: the ceiling
  is a property of a 4096-value alphabet against a 65535-value range.
- **What the build did.** `G14.e.3` is an `open()` leg that publishes
  `panel::box_reachability()` — per row the declared box, the reachable min and
  max, and both corner bits — as `unmeasured` with the method named. It did not
  add corner-witness seeds: §4.1 step 4 forbids re-sampling until something
  passes, and a hand-picked witness for a value the arithmetic excludes would be
  a false claim, not an unmeasured one. The panel's other five legs
  (a, b, d, f, i, plus the `c`/`g`/`j` block) are graded at all 18 members and
  were, at three rounds per member.
- **The decision, three ways.** (a) **Keep the boxes and re-word the leg**: accept
  that a Q12 draw reaches `lo` and `hi − 15`, and amend §19(e) to "each corner or
  its Q12 neighbor" — cheapest, and honest about what `map_fraction` is.
  (b) **Make the boxes draw-exact**: declare them `0..65520` (the largest
  reachable value on the grid), which the sweep literature calls the row's real
  ceiling — every recorded hash before the change is then historical, and the
  41 placeholders' midpoints move. (c) **Widen the draw**: two `next_q12()` per
  row for a Q24 fraction — a `seed_format` bump, and per 05 §19(c) it re-hashes
  every individual in every recorded control pair.
- **Why not other readings:** "the panel is too small" is testable and false —
  the scan is exhaustive over the alphabet, so no panel size reaches 65535.
- **Status:** OPEN. Needs (a), (b) or (c) ruled. The ledger recommends **(a)**:
  the leg's point is that no draw escapes its swept range, and (b)/(c) invalidate
  recorded evidence to buy a corner no gate has ever claimed a use for.

## OD-162 — who authors `seed_panel`, and what an artifact that outlives its binary proves *(B12's panel pass)*
- **Where the law is.** [03 §7](../../engineering/03-constants-registry.md)'s
  `seed_panel` size row (*"≥ 16, fixed per run, recorded in `gate_report.json`"*,
  serves **G14(h)** and **OD-83**) and [05 §19](../../engineering/05-gate-harness.md)'s
  prerequisite (*"`seed_panel` (§4.1) with ≥ 16 fixed `u64`s checked into
  `harness/scenarios/` … `G14/seed_panel.json` written per run"*). Neither names
  **how the 18 numbers are produced.**
- **The gap the build had to fill.** A list typed into a file is authorship by
  hand, and §4.1 step 4's ban on re-sampling the panel has no teeth if the panel
  itself is a hand-picked list. So `harness/scenarios/g14_seed_panel.json` is
  generated by one published rule and **re-derived from the code on every read**
  (`panel::load` sets `rule_matches_file`, and a member that is not the member the
  rule produces fails `G14.panel`): members 1 and 2 are the two seeds 05 §2 itself
  names (`0x…51` the V0 reference, `0x…00` V8's control twin — the panel begins
  with the pair the spec already cites), and members 3..18 are
  `u64::from_le_bytes(md5("SI-panel-<k>")[0..8])` for `k = 0..15`. Size 18 ≥ 16,
  all distinct. The asset lives under `harness/`, so it enters
  `runner::tree_digest` and is inside `binary_tree_md5`; editing a member changes
  the build stamp, which is the second half of the anti-hand-picking law.
- **The second finding, while wiring it.** `gates/` is git-ignored and
  `seed_panel.json` accumulates across **rebuilds**. A panel comparison that mixes
  records from two binaries compares a guard surface, a LAW table and a
  derivation from different builds, which is a category error of OD-91's family.
  The build therefore stamps every record with `tree_md5` and
  `PanelRecords::of(index, tree_md5)` keeps only same-build rows; the count of
  excluded rows is reported as `records_from_other_builds`. Two related choices
  came out of the same artifact: `G14.f.1` grades the **live** session's non-drawn
  constants (`panel::guard_bytes`) against the recorded ones, because the drawn
  half is excluded by construction (`Params::from_temperament`) and a LAW-row
  digest alone would be a tautology; and §19(d)'s "every gate holds for both"
  reads the **newest** record per (seed, gate, variant, scenario) rather than
  every row ever, so an earlier round of the panel's own incomplete legs cannot
  poison the leg forever.
- **The third finding, from the battery.** With `f.1` graded, the full variant
  battery produced **13 failing records** — every run that is not the reference
  launch (G13/V0+V9, G15, G16, G17/V0+V12, G18, G19/V0+V13, G10/V2+V3, G12/V4+V5).
  The guard digest differed from the panel's, and it was not a temperament leak:
  `guard_bytes` covers the Species and Ruling rows a launch **is allowed** to move
  (`install_launch_state` refuses only `Temperament` and `Law` rows by class), so a
  run launched at a different constant point or variant holds a different guard
  surface *by design*. Comparing it across launches is OD-91's category error in a
  new costume. 05 §19(f) never asked for that comparison: its procedure is "**V0 vs
  V8** diff over the whole registry" — one launch, two individuals — and its assert
  is "every **LAW** row is byte-identical across all panel seeds". The leg is
  therefore scoped: the LAW digest is compared against **every** same-build record
  (no launch and no seed can move a Law row), the guard digest only against the
  records sharing this run's `launch_digest` (config + variant + swept constant
  list + interrupt boundary + genome tag) and transcript. Measured after the
  scoping: 1 LAW digest across 95 same-build records, and within the reference
  launch's group the 18 individuals hold 1 guard digest. A record written by a
  harness that does not stamp a launch sits in a named `unrecorded` group rather
  than silently joining the current one.
- **The decision, three ways.** (a) **Rule the rule** — accept the md5-phrase
  generator, as the harness now enforces, and record the phrase in 03 §7 so the
  panel is reproducible by a reviewer. (b) Name a fixed list in the spec itself
  (18 literal `u64`s) and let `rule_seeds()` be the checker rather than the source
  — reviewer-visible, but the spec becomes the thing that can be quietly edited.
  (c) Derive the panel from the genome corpus so a panel member is an individual
  the build plan already ships — the strongest claim, and a new artifact dependency
  at B12.
- **Why not other readings:** "the seeds were drawn at run time from entropy" is
  forbidden by G14(b) and by I5, so a random-at-load panel is not on the table.
- **Status:** OPEN, recommended **(a)** with the phrase written into 03 §7.

# The TONGUE round ([docs/27](../27-tongue.md), G20, engineering/13) — OD-163 … OD-189

**Numbering note.** The owner's draft claimed OD-130…OD-143. The ledger was at
**OD-162** when the round opened, so every item is re-based at +33 and the draft's
two re-asks (its OD-140, OD-141) are merged into OD-165 and OD-168. The map is
[docs/27 §27.14](../27-tongue.md); no entry below cites a draft number.

## OD-163 — the trait set and grids of `DERIV_ORDER_TONGUE`, and whether `exclamation` may exist at all *(docs/27 §27.3)*
- **Where the law is.** [docs/27 §27.3](../27-tongue.md)'s five axes, each defined
  over a property the lattice already has, against [10 §4](../../engineering/10-temperament-sleep.md)'s
  P.2 (one draw per row, mapped onto the row's own range and grid) and I1 (form at
  birth, content earned).
- **The finding.** The draft named six traits and one of them — *exclamation* — has
  no substrate: punctuation carries no row, no edge and no interner field, so an
  exclamation index could only multiply a byte that does not exist. Shipping it
  would mean inventing a punctuation channel, which is a write path (docs/06's two
  keys) wearing a temperament name. The five structural axes (spread, depth, lean,
  recency, proximity) each weight a quantity `si-walk::Graph` already answers
  (`out_degree`, `Arc::kind`, `Arc::bake`, hop distance), so they are form.
- **The decision.** (a) the five axes, `exclamation` dropped; (b) five axes plus a
  punctuation channel with its own row kind and gate family; (c) a different axis
  set. **Status:** OPEN, implemented **(a)** — dropped, not faked.

## OD-164 — habit statistics: hash-in session state, or exempt *(draft OD-131)*
- Moot by construction: the tongue stores nothing. "Habit" is `STYLE_RECENCY`
  reading the `bake` and the arena position already on the lived edges
  ([docs/27 §27.3](../27-tongue.md), realized without `a_at` by **OD-182**),
  so there is no counter to place in state and no hash item to argue about.
- **Status:** CLOSED by derivation (OD-174 is the live half of the same question).

## OD-165 — temperament home: a separate derive order, or extending the closed 18 *(docs/27 §27.3, draft OD-132 + OD-140)*
- **Where the law is.** `check_temperament_closed`
  ([`si-cli/src/harness/checks.rs`](../../crates/si-cli/src/harness/checks.rs)) and
  `seed::order_is_closed()` make `|DERIV_ORDER| == temperament_count()` a schema
  fact, and `DERIV_ORDER` is a registry LAW row (`18..18`)
  ([`si-core/src/consts.rs:203`](../../crates/si-core/src/consts.rs)).
- **The finding.** Adding five rows to the species vector is not a five-row change:
  it moves `DERIV_ORDER_LEN`, so `size_of::<Temperament>()` (the 18×2 assert),
  `canonical_bytes`, the digest recorded in the core block, and therefore **every
  individual's fingerprint and every recorded control pair in `gates/`** change.
  The draw stream is also positional — a fifth trait inserted at position 3 would
  re-price the four rows after it for every seed that ever shipped.
- **The decision.** (a) **`DERIV_ORDER_TONGUE`, five draws keyed by the same
  `birth_seed`** — the species vector and its digest do not move; (b) extend the 18
  to 23 and re-baseline every individual; (c) make the five a launch-swept Species
  family (banned by OD-87: a seed-predisposition that a sweep can set is a control
  surface). **Status:** OPEN, implemented **(a)**, because it is the option that
  breaks nothing already measured.

## OD-166 — does the tongue register-shift per speaker class *(docs/27 §27.3, docs/26)*
- **Where the law is.** [docs/26](../26-world.md)'s class table and the R-21/R-39
  ancestry: a read side that varies by *who* is talking is a policy input, and the
  tongue's own thesis is that the voice varies by *who SI is*.
- **The finding.** A per-class or per-user style would give a stranger's presence a
  path into SI's surface — the same authority question G19 answers for writes,
  asked again for speech, and answered by a component whose contract is "reads the
  lattice and the style vector, nothing else".
- **The decision.** (a) **register-blind this round** — the tongue reads no
  `source`; (b) per-class shift with `source` as an input; (c) per-user, learned.
  **Status:** OPEN, implemented **(a)**; (b)/(c) need their own gate leg, not a
  clause in G20.

## OD-167 — the utterance-shape nomination detector *(draft OD-134)*
- **Where the law is.** [docs/27 §27.5](../27-tongue.md) and the doc-22 precedent:
  experience **nominates**, only the owner **mints** (G15). The draft's "ask the
  owner before adopting a shape the room keeps using" is a third nomination source.
- **The finding.** The accent already emerges without it: a room's repeated word
  gets a term row through the ordinary write gate, sleep bakes the co-occurrence
  edges, and `STYLE_RECENCY` prefers what is recent and connected. A detector would
  add an asker and a minting key — and per OD-87's discipline, a round that adds a
  writer adds a gate family of its own.
- **The decision.** (a) **not built**; (b) built as a nomination-only detector with
  a G20 leg and no write path; (c) built with a mint. **Status:** OPEN, implemented
  **(a)**; (b) is the honest next round if the owner wants the question asked.

## OD-168 — may `a` style the draw *(docs/27 §27.6 I8, draft OD-135 + OD-141)*
- **The finding.** Arousal-modified weights would make an excited SI talk differently
  in a way no row records, and I8's whole content is that `a` reaches the walk's
  *spread*, not a surface choice; the only licensed exception is OD-33's write-time
  bake size, which is not a precedent for a read-time one.
- **The decision.** (a) **banned** — the walker reads no `a` field, graded by
  `G20(e)`'s full-`a` isolation leg; (b) allowed through the existing modulation
  curve (which would reach speech only by moving activation, i.e. the walk's own
  already-gated path, so it adds nothing); (c) allowed directly.
  **Status:** OPEN, implemented **(a)**. Merged here is the draft's OD-141 (the same
  ban asked for the recency axis).

## OD-169 — retire/reject semantics for a phrasing row *(draft OD-136)*
- Moot: no phrasing row kind exists (OD-180). The rows the walk reads are term rows
  and `links` rows with the supersession, dimming and confidence laws they already
  have (docs/03's R-rules, docs/06, G13).
- **Status:** CLOSED by derivation, reopenable only with OD-180.

## OD-170 — first-words bootstrap: may the owner seed starter phrasings *(draft OD-137)*
- **Where the law is.** docs/06's two keys and [09](../../engineering/09-teach-channel.md)'s
  `/teach` lane; I1's empty-state litmus (§3.4, §18.1's "at t=0 nothing fires").
- **The finding.** A starter corpus *installed by the build* is content at birth —
  exactly the costume R-21 bans in its first shape. A starter corpus *taught by the
  owner* is an ordinary act in the ordinary lane, and needs nothing new: one line,
  one commit, one provable provenance. The question the draft really asked is
  whether bulk-teaching N paraphrasings of one meaning on one line is legal input;
  that is OD-69/OD-70's feeder territory, already gated by `G1.u`.
- **The decision.** (a) **legal only through the existing `/teach` lane, one line =
  one act**; (b) a build-shipped starter bank (banned); (c) a pinned artifact loaded
  at wake (a frozen layer, so ENGINE-only, and it would make V0's voice a corpus).
  **Status:** OPEN, implemented **(a)**.

## OD-171 — cross-language rooms: are shapes counted per language *(draft OD-138)*
- Moot this round — nothing counts utterance shapes (OD-167). The tokenizer already
  folds per script and a term row is a term row, so the walk is language-blind by
  accident of the substrate, not by a rule. Reopen with OD-167.
- **Status:** OPEN, inert.

## OD-172 — the phrase-walk step budget and its termination law *(docs/27 §27.4, draft OD-139)*
- **Where the law is.** [03 §1.2/§7](../../engineering/03-constants-registry.md)
  (a swept row ships a box, not a value), OD-86's placeholder policy, OD-83's
  unrulled enumeration (which is why the arc's four sweep runners publish
  `status: refused`), and `G20(i)`'s budget bound.
- **The finding.** Termination needs four exits, and only one of them is a number:
  slots satisfied, no candidate, parse-back impossible, and `PHRASE_MAX_STEPS`. The
  first three are laws; the fourth is swept, so the round ships it as a SPECIES row
  with a declared box and a `G20/phrase_sweep.json` that behaves like the arc's
  four — a published refusal while OD-83 stands. Every leg that needs the number
  then runs at the box's `lo` and names itself a floor, per OD-95's precedent.
- **The decision.** (a) **Species row, swept box, floor leg until the sweep
  publishes**; (b) a LAW constant (an un-swept magnitude, which 03 §1.2 forbids);
  (c) `STYLE_SPREAD` alone as the budget (rejected: a temperament draw would then
  own the run's worst-case cost, and the budget is a resource fact, not a persona).
- **RULING (owner, 2026-10-25, B12 batch).** *"option (a) confirmed."*
  `PHRASE_MAX_STEPS` is a **SPECIES** row with a swept box; the floor leg runs at
  the box's `lo` per OD-95's precedent until `gates/G20/phrase_sweep.json` publishes,
  and it converts at re-run once that sweep's enumeration is executable. (b) — an
  unswept magnitude — is forbidden by 03 §1.2; (c) — a resource bound handed to a
  temperament draw — is rejected.
  **Status:** **RULED (2026-10-25, B12 batch)** — option (a), as implemented.

## OD-173 — parse-back strictness: slots-present, or meaning-identity *(docs/27 §27.2(c), draft OD-142)*
- **The finding.** The strong form — decode the realized line and compare its frame
  and slots to the settled thought's — would need a second decode inside the answer
  path, i.e. the read path calling itself. The weak form asserts what the invariant
  actually claims: the slots the line was generated from are all still in it, so no
  realized line can drop the thing that was asked about or the answer it stands on.
  Weaker is honest here; stronger would be a new algorithm with no gate of its own.
- **The decision.** (a) **slots-present** (every content key of the canonical render
  appears in the realized line's folded tokens), fall back on a miss; (b) meaning
  identity via a decode walk; (c) no parse-back (banned — §27.11 requires it).
  **Status:** OPEN, implemented **(a)**.

## OD-174 — do anchors' habit counts feed edge weights *(docs/27 §27.5, draft OD-143)*
- **The finding.** "Feed" is a write. The walk reads the `bake` and the recency the
  sleep pass already left on the edge; a count of how often SI *said* a word is a
  sixth statistic, and it would live either in state (a new hash item, a schema
  change) or in the edge (a new field, breaking 01 §5's layout).
- **The decision.** (a) **read-only**: the tongue writes nothing and stores nothing;
  (b) a session-side habit field hashed in state; (c) an edge field.
  **Status:** OPEN, implemented **(a)** — and `G20(d)`/`G20(f)` grade the zero-write.

## OD-175 — what registry class do the five style rows carry *(docs/27 §27.12 item 1)*
- **Where the law is.** `Class` is `{Law, Species, Temperament, Ruling, Derived}`
  ([`si-core/src/consts.rs:20`](../../crates/si-core/src/consts.rs)) and OD-87's
  ruling is that **no sweep touches a TEMPERAMENT row**. `placeholders_match_registry()`
  ties `temperament_count()` to `DERIV_ORDER_LEN`, so a sixth class is the only way
  to have five drawn rows that are not the 18.
- **The finding.** `Temperament` cannot take them (OD-165, and it would make them
  sweepable-by-class in the registry's own counting); `Species` cannot take them
  (a sweep could then set a persona — OD-87 in a new costume); `Derived` means
  "computed, not drawn"; `Ruling` means "a switch, not a magnitude". So the round
  adds a **sixth class**, and each of the three places that branches on class gets
  a clause: `check_temperament_closed` (a Tongue row is not in `DERIV_ORDER` and an
  18-row vector stays 18), `install_launch_state` (a Tongue row is **not**
  installable at launch — same refusal as Law and Temperament), and the sweep filter
  (a Tongue row is **not** sweepable — the same refusal as Temperament).
- **The decision.** (a) **a sixth class `Tongue`**; (b) reuse `Temperament` and
  re-baseline every individual (OD-165's option b); (c) reuse `Species` (banned by
  OD-87's law). **Status:** OPEN, implemented **(a)**.

## OD-176 — which stream does the realization draw from *(docs/27 §27.4)*
- **Where the law is.** hash item 7 is the runtime stream's `RngState`
  ([`si-core/src/rng.rs:11`](../../crates/si-core/src/rng.rs)), `G14.b.3` promises
  the derivation and the runtime stream never touch each other, and item 7's
  recorded trajectory is the **ignition** history.
- **The finding.** Drawing from the session's runtime stream at speak time spends
  the same sequence the ignition sampler spends, so two transcripts that differ only
  in how many answers were realized would diverge in *which idle thought lit next*
  — a state-hash difference with no temperament or lattice cause, re-baselining
  every recorded control. The realization is therefore a pure function of a
  snapshot-seeded PRNG: `(birth_seed, the style digest, the pre-line state hash,
  the thought id)`, built with the already-gated `Rng::seeded` non-stream precedent
  (`si-read/guess.rs`). Nothing is spent from R, nothing is stored, and the same
  snapshot replays the same sentence byte for byte in another process — which is
  what makes `G20(a)` a cross-process leg at all.
- **The decision.** (a) **per-render PRNG seeded from the snapshot**; (b) the
  runtime stream R (rejected above); (c) a fifth stored stream in the snapshot (a
  new hash item, and it buys nothing over (a) because the seed already includes the
  hash it would have to match). **Status:** OPEN, implemented **(a)**.

## OD-177 — "no value before its sweep" against "no sweep touches a temperament row" *(docs/27 §27.3)*
- **The clash, stated.** [README rule 2](../../engineering/README.md) and 03 §1.2
  say a tunable ships a box and waits for `gates/G20/tongue_sweep.json`; OD-87 says
  a sweep may never touch a temperament-family row. The five style draws are
  temperament-family by class law.
- **Which law wins.** **OD-87.** The style rows are never swept, so there is no
  winner to quote and no value that could be invented: a draw's "value" *is*
  `(birth_seed, position in DERIV_ORDER_TONGUE)`, exactly as 03's header note says a
  TEMPERAMENT row works. The boxes the draws map onto are still declared, and their
  validation is [05 §4.1](../../engineering/05-gate-harness.md)'s panel pass — the
  same instrument G14(h) uses — so "the whole box governs" is bought by the panel,
  not by a sweep. `G20` cites the panel artifact instead of a `tongue_sweep.json`,
  and the one genuinely swept magnitude of the round is `PHRASE_MAX_STEPS` (OD-172),
  which is a Species row and therefore sweepable.
- **Status:** CLOSED by ruling-precedent (OD-87 + OD-83 + 03 §1.2's temperament
  note); recorded here because the draft asked which law wins.

## OD-178 — is the tongue ON at V0 *(docs/27 §27.1 litmus 1)*
- **The finding.** Turning the voice on at birth changes every rendered string in
  the repo: every golden answer, every `transcript.txt`, every log-line leg that
  greps a surface, and hence every recorded control. That is not a reason to refuse
  the tongue — it is a reason to make the switch a **Ruling row** (`TONGUE`, default
  `0`) like `CRYSTALLIZE`, `SKILLS` and `WORLD_TIERS` before it, and to ship the ON
  state as a named variant (**V14**) so `G20(j)` can grade the OFF state as
  byte-identity rather than as a promise.
- **The decision.** (a) **OFF at V0, ON at V14; shipped default is the owner's
  call**; (b) ON at birth (re-baselines every control, and fails litmus 1);
  (c) owner-runtime toggle outside the registry (a control surface with no class —
  banned by OD-87's rule 1). **Status:** OPEN, implemented **(a)**.
- **The consequence the build cannot hide:** the call site changes the binary, and
  a `CONSTS_VERSION` bump moves hash item 8 for every run. So the bump to `0x0003`
  is a **re-baseline, signed by re-running every recorded control**, never by
  editing a record (OD-91's discipline).

## OD-179 — may the phrase walk traverse a frozen-genome arc *(docs/27 §27.8(k))*
- **Where the law is.** [docs/05](../05-substrate-tesseract.md)'s Critical Law and
  `Arc::is_genome()` (kind ≥ 7): a genome arc is species text, not something this
  individual lived.
- **The finding.** Under CONFIG ENGINE the merged graph hands live and genome arcs
  over one cursor, so a walker that does not branch on `is_genome()` would print a
  species word as though it were SI's own — content without provenance, and the one
  way this round could smuggle a corpus into the voice without a bank. The rule is
  therefore structural: a candidate arc must be live, and `G20(k)` grades it by
  asserting a genome-only lattice realizes **no** line.
- **The decision.** (a) **live arcs only**; (b) genome arcs allowed (rejected: it
  makes V14's voice partly inherited); (c) allowed and labelled in the citation (a
  genome row id in a citation reads as lived evidence, which is a lie of omission).
  **Status:** OPEN, implemented **(a)**.

## OD-180 — does the tongue need a `phrasing` row kind *(docs/27 §27.2)*
- **The finding.** The record schema has no free tag byte (`Body.resp_surface`'s
  reserved slot is checked by `decode` and bails when nonzero), the shelf registry is
  closed at eleven with two reserved, and `PatternId` is closed at ten. Each of those
  is a schema fact some gate already grades, so a `phrasing` kind is not one new
  enum: it is an envelope change, a shelf, a supersession rule, a dimming floor and a
  prov kind. The owner's amendment removes the need — taught sentences are ordinary
  rows and act as **anchors** — so the round adds no row kind, no shelf, no state
  byte and no vocabulary word, and the deletion test shrinks to "remove the crate".
- **The decision.** (a) **no new kind**; (b) a `phrasing` kind with the full schema
  ripple and its own gate legs. **Status:** OPEN, implemented **(a)**.

## OD-181 — what vocabulary may a realized line utter *(docs/27 §27.2 check (b), docs/07 §7.5)*
- **The finding.** This is the fold's own catch, not the draft's: §27.2's gate as
  drafted checks that every realized word **names a term row**, and the build has
  that predicate written into `G20(h)`. It is too loose. SI has rows for far more
  points than any one answer cites — after a hundred teaching lines the registry is
  full of terms this particular answer has no business saying — so a walker that
  satisfies check (b) perfectly can still emit "the apple is red" from a citation
  about weight. §7.5 is older law and says the opposite: the answer's content tokens
  must come from **cited** rows, and anything else demotes the label. The tongue runs
  *after* the label, so it has no demotion available; a line that wanders out of the
  citation ships with the label and the provenance already minted. That is R-43(a),
  the fluent lie, reached by a gate that was written to prevent it.
- **Why it was not caught by the other legs.** `G20(g)` parse-back asserts the
  semantic slots are *present*, not that nothing extra arrived; `G20(b)` asserts the
  label cannot move, which becomes *more* dangerous once an over-reaching line is
  possible; `G20(h)` as drafted greps against the whole registry, which every legal
  line passes vacuously. A leg whose empty set is the pass, against a set that
  contains every word SI knows, is a leg that cannot fail.
- **The decision.** (a) **§7.5 wins** — a realized line may utter only the surfaces
  of the points *its citation names* (each cited row's key and value point: canonical
  surface plus learned aliases — the very surfaces `reverse_decode` already reads) and
  the composed chunks §7.5 has always allowed; the walk may *traverse* a cheap
  out-of-set arc but may not *utter* from it, and a line that cannot be finished
  inside the set falls back to canonical. `G20(h)`'s predicate is widened to the
  citation-scoped set and the whole-registry grep is retired. **Implemented.**
  (b) keep the registry-wide reading and accept that the tongue can assert uncited
  content (rejected: a new claim channel with grammar on it and no demotion path);
  (c) give the tongue its own demotion (rejected: §27.2 check (a) — label before
  realization, never after — is what makes the voice harmless, and (c) would delete
  it).
- **The cost, said out loud.** Under (a) SI can only sound fluent in the connectives
  and inflections it was actually taught, because those are rows like any other. A
  newborn with three term rows says three-term-row sentences however warm its style
  draws run, and the variety lives in ordering, length and which owned alias is
  picked. That is the amendment's own logic carried to its conclusion — the variant
  bank was demoted because SI's sentences are the rows it earned, and a word is a row.
  It also narrows `STYLE_SPREAD`'s reach: verbosity can only spend the licensed set,
  so two individuals differ in *how much of what they know* a line uses, not in how
  many words they have. Recorded here because the draft did not price it.
- **Status:** OPEN, recommended **(a)** — implemented as (a), and the owner can rule
  (b) only together with a demotion path that does not violate §27.2 check (a).

## OD-182 — what "recently-charged" means when `a` is banned *(docs/27 §27.3, §27.6's I8 row, OD-168)*
- **Where the law is.** [docs/27 §27.3](../27-tongue.md) defines `STYLE_RECENCY` over
  "recently-baked / **recently-charged**" arcs, while the same document's §27.6 I8 row
  and **OD-168** (ruled: the tongue reads no `a`) forbid the operand. `si-walk::Arc`
  carries `a_at: u8` — the arousal level at formation — one field away from `bake`.
- **The finding.** The two sentences are both in the draft and cannot both be
  implemented. Reading `a_at` makes the voice arousal-sensitive, which re-opens the
  OD-168 argument inside a fresh crate and gives `G20(e)` (the label cannot move) a
  seed path around it: a line whose wording depends on a charged `u8` is a mood
  readout wearing a grammar name.
- **The decision.** (a) **`bake` plus arena position** — `bake` is the write-side,
  permanent quantity the arc already stores, and `edge_off` is formation order, already
  the graph's tie-break key; "charged" is realized as "recently formed", which is what
  the charge *was* at the moment it happened. `a_at` is never read, and
  `check_tongue_reads_no_a` makes that a grep rather than a promise. (b) read `a_at`
  (rejected: it violates a ruled OD to honor a draft line). **Status:** OPEN,
  implemented **(a)** — see [engineering/13 §10](../../engineering/13-tongue.md).

## OD-183 — whether the state hash seeds the per-render PRNG *(docs/27 §27.4)*
- **Where the law is.** [docs/27 §27.4](../27-tongue.md) seeds the per-render stream
  from "(birth_seed, the style digest, **the state hash before the line**, the thought
  id)". Hash item 3 of [01 §9](../../engineering/01-data-schemas.md)'s nine is the stamp
  arena, and stamps carry `a_at`.
- **The finding.** The draft's own seed operand re-admits arousal through the back of
  the seed: a hash that moves with `a` makes the line move with `a`, which is exactly
  the route OD-182 closes at the arc level and exactly what `G20(e)` claims to grade.
  It also costs I5's cleanest case — a snapshot-only seed replays byte-exactly across
  processes without reproducing the arena.
- **The decision.** (a) **`seed = (birth_seed, style digest, thought_id)`** — three
  snapshot values, so I5 and replay hold and the arousal-sensitive operand is excluded;
  (b) keep the hash in the mix (rejected: OD-168 by another door). **Status:** OPEN,
  implemented **(a)**.

## OD-184 — `si-tongue`'s import set: three crates, or two *(docs/27 §27.7)*
- **Where the law is.** [docs/27 §27.7](../27-tongue.md)'s module map gives the crate
  `si-core`, `si-store` and `si-walk`. [engineering/13 §0 fact 6](../../engineering/13-tongue.md)
  makes the licensed vocabulary an argument (`License`), not a lookup.
- **The finding.** Holding a `&si_store::Lattice` means the crate can enumerate every
  point in the world, which is precisely the uncited word **OD-181** forbids an
  individual to utter. With the surfaces handed in as data, the restriction is a
  missing edge rather than a discipline, and `G20(h)` has one mouth to grep against.
- **The decision.** (a) **two imports — `si-core`, `si-walk`** (strictly *narrower*
  than the law asks; `check_tongue_is_leaf` grades it); (b) the draft's three (rejected:
  it hands the leaf crate a registry-wide vocabulary it is gated not to use).
  **Status:** OPEN, implemented **(a)**.

## OD-185 — does the deps lock move when a crate joins *(engineering/13 §3.4, §6.3, 11 §B20 DoD 7)*
- **Where the law is.** [docs/00 §2](../00-stack-workspace.md)'s allowlist law
  (**OD-42**) is the thing `gates/deps_lock.md5` exists to sign;
  `check_deps` compares `md5(Cargo.lock)` against the published file.
- **The finding.** The draft wrote three times that the lock "does not move" because
  the tongue adds no dependency. Both halves are true and the conclusion is not: a
  workspace member is itself a `[[package]]` row in `Cargo.lock`, so joining the
  workspace changes the hashed file. `git log -- gates/deps_lock.md5` shows B4, B14,
  B15, B17 and B11 each re-publishing it when their crate arrived.
- **The decision.** (a) **re-publish `gates/deps_lock.md5` with the crate**, in B20
  step 2's commit, and correct the three spec lines (done); (b) leave the claim and
  let `check_deps` go red (rejected: a red gate is a wall, not a finding). OD-42's
  allowlist itself is untouched — no external package enters the graph.
  **Status:** OPEN, implemented **(a)**.

## OD-186 — which predicate is "claim-side" in `owned_share` *(docs/27 §27.3, engineering/04 §34 AF.3, 13 §34.3)*
- **Where the law is.** [docs/27 §27.3](../27-tongue.md): `STYLE_DEPTH` is formality —
  "claim-side arcs (owned kinds) over associative ones (`cooccur-weak`)".
  [docs/03 §3.2](../03-birth-state.md) fixes the kind space: 0..=5 are the claim edges,
  `6 cooccur-weak` is the candidate, 7+ are genome.
- **The finding.** The engineering table spelled the predicate as
  `edge_kind_is_owned(kind)`, but that function is `kind <= EDGE_KIND_OWNED_LAST` and
  `EDGE_KIND_OWNED_LAST` **is** 6 — it is the record-*shape* test (which columns an
  edge row has), not the semantic claim test. Implemented literally, `6 cooccur-weak`
  would weigh 4096 like a claim and `STYLE_DEPTH` would have no associative side to
  lean away from; the first draft's own unit test measured the split at 28/64 and
  caught it. `si-front::label::is_claim` already writes the right rule by hand
  (`owned && kind != EDGE_KIND_COOCCUR_WEAK`).
- **The decision.** (a) **`si_core::addr::edge_kind_is_claim(kind)`**, one named
  predicate beside the kind table, used by `owned_share`; the law's word ("claim-side")
  decides, the engineering spelling was a transcription error — so this corrects the
  HOW, not the law (done, in 03 §7, 04 §34 AF.3, 13 §2 and §34.3's tables);
  (b) the literal `edge_kind_is_owned` (rejected: it contradicts docs/27 §27.3's own
  example); (c) rewrite `si-front::is_claim` to call it too (deferred: same rule, but
  si-front is not this round's crate and its label arithmetic is gated by G9/G12).
  **Status:** OPEN, implemented **(a)**.

## OD-187 — the realized artifact's row shape: no `drew`, no `reason` *(docs/27 §27.8(b)(c), engineering/13 §6.4)*
- **Where the law is.** [docs/27 §27.8](../27-tongue.md) drafts `realized_log.jsonl`
  with a `drew: [fractions]` list and `refusals.jsonl` with a `reason` column
  (`no_candidates`, `unlicensed`, `slot_lost`, `budget`).
- **The finding.** The walker is `si_tongue::realize(...) -> Option<String>`: the
  per-render PRNG is a `&mut Rng` inside the crate, and a refused walk returns
  `None`. A `drew` list or a reason name would have to leave the crate through an
  out-parameter — which `check_tongue_writes_nothing` refuses — or be recomputed by
  the caller, which is a second copy of the walk and is exactly what G20(c)'s
  boundary check forbids. G20.a.2 additionally compares **voice rows byte for byte**
  across processes, so an unstable diagnostic would have been a false red.
- **The decision.** (a) **No `drew`, no `reason`** (as built): the realized row is
  `{t, thought, seed, anchor, licensed, chunk_cap, citation, canonical_surface,
  realized_surface}`; a refusal line names the **drive that forced it**
  (`forced_by: {constants, fixture}`), and the four arms' names are graded in
  `crates/si-tongue/tests/realize.rs` where each arm is constructed;
  (b) add the fields anyway by threading an out-param through `realize`
  (rejected: breaks the purity check and the crate's whole claim).
  **Status:** OPEN, implemented **(a)**.

## OD-188 — the walk's anchor: `val_point` or `key_point` *(docs/27 §27.4, crates/si-session `tongue_line`)*
- **Where the law is.** [docs/27 §27.4](../27-tongue.md) lists two anchor
  candidates: the activated leaf, then "the ask's named target".
- **The finding.** A taught statement is one live edge subject→predicate; the write
  gate keeps one live predicate per subject and no edge points back. A walk starting
  at the answer row's `val_point` (the leaf) therefore has no out-arc to take and
  every tick falls back — measured before the re-seat, and recorded in
  `failure_report.md` (B20).
- **The decision.** (a) **`key_point`** (as built): the point the line is *about*,
  chosen by the printer's own pool and comparator so the speaking row and the decoded
  row cannot disagree; G20(c)'s answer-token check keeps it honest — a line that never
  reaches the answer token is refused; (b) `val_point` with a synthetic back-edge
  (rejected: invents a record the write gate never made).
  **Status:** OPEN, implemented **(a)**.

## OD-189 — the license's fertility ceiling *(docs/27 §27.2 OD-181, §27.11; G20.b.2)*
- **Where the law is.** [docs/27 §27.11](../27-tongue.md) asks for "some randomness
  in talking within bounds of knowledge"; §27.2/OD-181 restrict a line to the
  surfaces of the points its citation names, plus composed chunks.
- **The finding.** One cited row licenses its composed chunk and its two endpoint
  terms — two chunks — so every speaking individual prints byte-identical lines.
  Measured across G20's eight-seed panel: five speak (`apple, red` / `red, warm` /
  `warm, cozy` / `sky, blue`), **all the same**, while `chunk_cap` ranged 2…12; three
  seeds (spread ≤96) speak nothing and print the canonical frame. The randomness the
  walk offers is real but **unobservable at this fixture**: the budget is not the
  bound, the license is.
- **The decision (owner, 2026-10-25). RULED.** Verbatim: *"two-chunk claim license
  stands (least authority, no new claim width, one fact per line). Variant-bank
  surfaces over the same cited endpoints are licensed grammar: same claim, different
  surface, no new rows asserted. G20.b.2's pass condition becomes: at a fixture WITH
  taught variants, no two seeds print byte-identical lines (style must be observable);
  at a bare fixture, byte-identical is the expected outcome, named as such. chunk_cap
  publishes with a license-bound note while the 2-chunk license is the binder."*
- **As built.** `si_tongue::Variant` + `License.variants` (the crate still holds no
  table); `Session::license_for` groups every unique interned verbatim surface by
  `si_front::tokenize`'s folded keys and attaches same-key spellings to each entry;
  `utter` spends one `next_q12` only when a genuine alternative exists. The G20
  fixture gained taught case spellings in the **predicate** position — a zero-row
  confirm that interns the surface — and a bare control without them.
  `G20.b.2` grades both panels: variants ⇒ the five speaking seeds' transcripts are
  pairwise distinct and a taught spelling is drawn (27 draws in the recorded run);
  bare ⇒ the five speaking seeds print one byte-identical transcript, named as the
  expected outcome; both panels keep frames/state/census equal to `TONGUE = 0` and
  the protected-shelf census equal across fixtures. `G20.e.2` was narrowed from byte
  identity to **claim** identity: an idle pad can ignite and spend a thought id, a
  sanctioned render-seed operand, so a different taught spelling is licensed there
  while a moved *chunk* is not. `chunk_cap` is published everywhere beside
  `cap_note` (`G20_CAP_NOTE`): a ceiling whose binder is the two-chunk license.
  **No new row, shelf, state byte or citation width.** If a variant-rich fixture
  still shows zero style divergence, the license question reopens as a **NEW OD**
  with that evidence — it is not widened silently here.
  **Status:** **RULED — implemented; G20 green** (`gates/G20_si_*/gate_report.json`,
  `panel.json`).

# B13's acceptance pass (engineering/11 §4) — OD-190 … OD-191

## OD-190 — G5's family has no evaluator at the stage the plan gives it *(B13)*
- **Where the law is.** [engineering/11 §5](../../engineering/11-build-plan.md) puts
  G5 DETERMINISM + NO-PRIOR's `full` column at **B13**, the B13 row's Creates column
  says **no new code**, and B13's exit gates name **G5**. [05 §10](../../engineering/05-gate-harness.md)
  writes eight legs (a–h).
- **The finding.** No `run --gate G5` evaluator exists: the dispatcher refuses it
  (`crates/si-cli/src/harness/gates.rs:72`). Four legs have an instrument in this
  build — `a.1` (the V6 replay), `a.2` (`check_no_mood_field`), `c.1`
  (`check_string_constants`), `c.2` (`check_config_guard`) — `b.1` has a unit test
  (`interner_is_per_session_and_never_merges_surfaces`), and `d.1` is covered in
  substance because the replayed G1 transcript contains the gibberish teach.
  **`e.1`** (per-step activation traces) and **`f.1`** (idle cascades and monologue
  entries) have no runner: `Run::replay_traces` is implemented
  (`crates/si-cli/src/harness/runner.rs:1233`) and nothing calls it.
- **Options.** (a) Accept G5 by the acceptance instruments and name `e.1`/`f.1`
  UNMEASURED — the pass then reports 19 of 20 families with a named instrument
  instead of a report; this amends 11 §5's G5 row and 05 §10's expectation that all
  eight legs are gate-graded. (b) Write the evaluator — new code at the stage the
  plan calls code-free: `b.1`/`d.1` are cheap, `e.1` is wiring the existing
  `replay_traces` to its first caller, `f.1` needs a monologue comparison across the
  same two runs. (c) Move G5's full to B12's V-matrix pass, where G11 also waits, and
  let this acceptance pass grade 19 families.
- **Build choice:** none — the pass records the state and does not invent an
  evaluator the plan excludes.
- **RULING (owner, 2026-10-25, B12 batch).** **B13.1, a micro-stage, is
  authorized**: write the G5 evaluator and wire `Run::replay_traces` (which has no
  caller today). Scope is exactly **`e.1`** (per-step activation traces) and
  **`f.1`** (idle cascades + monologue entries); `a.1`/`a.2`/`b.1`/`c.1`/`c.2`/`d.1`
  are verified by their existing instruments, not rebuilt. No other source moves.
  This is the one deliberate exception to B13's docs-only scope and it is ledgered
  as such.
- **Status:** **RULED (2026-10-25, B12 batch)** — B13.1 authorized, scope
  `e.1` + `f.1` + the `replay_traces` call site.

## OD-191 — can global acceptance close at 18/20 while B12's five runners are unwritten? *(B13; parent OD-83)*
- **Where the law is.** [engineering/11 §4](../../engineering/11-build-plan.md)'s
  "full gate suite green" row demands every family **G1–G20**; [§5](../../engineering/11-build-plan.md)
  puts G11's full at **B12** and gives B12 five new runners (G6, G11 ×2, G12, G14).
  [05 §16](../../engineering/05-gate-harness.md) grades G11 through two sweeps.
- **The finding.** B12 has not run: the enumeration question (**OD-83**) is unrulled,
  so every existing sweep runner publishes `status: refused` (ten artifacts, all
  refusals) and the five B12 runners do not exist. G11 therefore has no evaluator and
  no curve; the median arithmetic is unwired, and `G3.em.1`, `G7.charge.2` and
  `G13.k.3` remain `unmeasured` for the same reason.
- **Options.** (a) Rule **OD-83** (the enumeration and its baseline), build B12's
  five runners, grade G11, and re-run acceptance — the only path to a literal 20/20.
  (b) Accept the 18/20 record for this build and keep G11 visible as an open family
  (what this pass does). (c) Amend 05 §16 so a V0-only G11 evaluator grades the
  composition legs and names both sweeps UNMEASURED — a spec edit, and the median
  half (the half that makes G11 G11) would stay ungraded.
- **Build choice:** **(b) recorded, not decided** — the pass reports only the families
  it can run and names the two it cannot.
- **RULING (owner, 2026-10-25, B12 batch).** **Accept the G11 wall as sequencing,
  not failure** — the 18/20 record is a sequencing record, not a final state: G11
  grades when B12's runners exist, and **B12 is the build's only remaining engine
  work**. Acceptance does not close on the 18/20 record; it re-runs after B12 (or
  names B12's own walls in the ledger).
- **Status:** **RULED (2026-10-25, B12 batch)** — sequencing accepted; B12 is the
  remaining engine work.

# B12's sweep pass — OD-192

## OD-192 — which enumeration does 05 §4 step 1 own when every swept box declares an open bound? *(B12; residual of OD-83)*
- **Where the law is.** [engineering/05 §4](../../engineering/05-gate-harness.md)'s
  step 1: *"Enumerate the grid over the declared legal ranges in 03 for the
  constants the gate owns"*, with step 2 running the gate's scenario per point, step
  3 evaluating the design's objective and step 4 publishing the full curve, winner
  and per-point objective. [§4.1](../../engineering/05-gate-harness.md) adds the
  validation protocol. [engineering/03 §2](../../engineering/03-constants-registry.md)
  writes the legal ranges as **open** bounds — `> 0`, `≥ 0`, `(0,1)`, `≥ 1` — so the
  registry's `hi` is the row's type maximum (`u16::MAX`), not a declared design
  ceiling.
- **The finding.** B12 built the missing runners — G11's median and composition, G14's
  panel, G15's recurrence — and ran all fourteen artifacts at the reference seed
  (`0x51`). Every winner-searching runner **refuses**, and not for a missing
  evaluator: at least one axis of each declares an open bound, so the grid has no
  last point, and the cross product over the registry's closed representation is
  beyond any stage:

  | gate / runner | axes | open axes | cross product |
  |---|---|---|---|
  | G2 constants | 6 | `TAU_SETTLE`, `HUB_DAMP_EXPONENT`, `FRONTIER_CAP` | 18,516,314,440,531,700,690,625 |
  | G8 budget | 2 | `GAP_SALIENCE_FLOOR`, `DEDUPE_WINDOW` | 4,294,901,760 |
  | G9 arousal | 6 | `B_MAX`, `K_TANH`, `MOD_CURVE_VARIANTS` | 221,296,755,218,306,040,000 |
  | G10 ignition | 1 | `IDLE_PER_HOUR` | 65,535 |
  | G11 median | 2 | `MEDIAN_MIN_N` (plus `GAMMA`, a TEMPERAMENT draw, off the sweep by class) | 196,605 |
  | G11 composition | 2 | `INFERENCE_STABLE_N` | 4,194,176 |
  | G13 sleep | 2 | `SLEEP_EVERY_QUIET_TICKS`, `SLEEP_MAX_WALKS` | 4,294,770,690 |
  | G13 dimming | 4 | `FLOOR_S_R`, `FLOOR_FACTS`, `FLOOR_LINKS` | 71,772,833,461,370,625 |
  | G15 recurrence | 4 | `GHOST_MAX`, `R_MIN_CRYSTALLIZE`, `R_ASK_CRYSTALLIZE`, `DELTA_MIN_CRYSTALLIZE` | 18,445,899,661,664,256,000 |
  | G16 silt | 6 | `SILT_LEVEL_MAX`, `SILT_BUMP`, `THETA_SILT`, `SILT_MAX`, `IGN_WEIGHT_SILT` | 308,261,972,457,841,313,710,080,000 |
  | G17 prov | 2 | `PROV_LINK_TUG`, `RELIGHT_MAX` | 4,294,901,760 |
  | G18 shape | 4 | `R_MIN_SHAPE`, `DELTA_MIN_SHAPE`, `K_FILLER`, `SHAPE_MAX` | 18,445,618,195,277,414,400 |
  | G20 phrase | 1 | `PHRASE_MAX_STEPS` | 65,535 |

  **G14** publishes its structure — 18 members, 18 distinct temperament digests,
  size/distinctness/rule all green — but its `whole_box` is `unmeasured`, because
  §4.1's first assertion validates a *published* winner and none exists. **G12**'s
  trio is observation-only (§17): verified by hash, never a curve. The artifacts are
  the run's own evidence: `gates/<G>/*_sweep.json`, each carrying the axis list,
  cross product, identity (one build, V0, no cross-variant comparison — OD-93),
  whole-box protocol and `placeholders_replaced: []`.
- **Why this is not OD-83 re-opened.** OD-83's ruling text says the whole-box
  **validation** stands and *"§4 step 1's search enumeration is the sweep law's own
  text; this ruling does not restate it, and B12 executes it as written."* Executing
  it as written is what produced the refusals: §4 names no per-axis search rule, and
  every declared range is open in 03. OD-83 would be re-opened by *weakening*
  validation; this entry asks the prior question — what object step 1 enumerates —
  and leaves validation untouched.
- **Options.** (a) The full cross product, literally: at 65,535 points (the smallest
  box, G10/G20) each point is a full gate run, and at 3.08×10²⁶ (G16) the curve is
  not a stage artifact. Literal and infeasible. (b) One axis at a time against a
  named baseline: the baseline must be a law-named point, and the only ones in the
  box today are `lo` and `hi`; OFAT at `lo` proves a neighbourhood, not the box, and
  still needs a rule for what "passes" at an interior point. (c) A **declared
  sub-grid**: each axis at a small published set of on-grid points (at minimum `lo`,
  `hi` and the placeholder), full cross product over *that*, then §4.1 unchanged.
  This is the smallest object that can carry a curve while every current value is a
  placeholder, and it adds no un-published point: the two corners are already §4.1's
  protocol. (d) Close the boxes: edit 03's legal ranges to declared design ceilings
  and bump `seed_format`, making step 1 literally enumerable — cleanest, but it is a
  spec edit that narrows the species space the owner wrote open.
- **Build's proposed reading (evidence, not a ruling).** (c), with the ladder written
  into 03's §2 tables (a new column or a per-row declared step set), because (a) is
  infeasible, (b) has no law-named baseline, and (d) forecloses the very ranges the
  owner left open. A ladder's *width* is a design decision: the build will not pick
  three points for the owner and call it the box.
- **Build choice:** none. Every runner writes its named refusal with the axis list,
  product, identity and whole-box protocol; no winner is invented, no placeholder is
  replaced (**42 remain**), and `--strict-consts` still refuses a run of any swept
  row. `runner::walkable_ceiling` (1,000,000 points) is a refusal threshold, not a
  search rule: it names no point and drops no axis.
- **Debts this pass closes, and one it finds.** `gates/G15/recurrence_sweep.json`
  (B14's DoD 4) never existed because G15 had no runner; it now exists as a named
  refusal, and `sweep::axes("G15")` is no longer empty, so
  `check_sweep_never_touches_temperament` now scans G15's four swept rows. The plan's
  B12 row names "(G6, G11 ×2, G12, G14)", but §4 assigns **no G6 artifact** (G6 is a
  hardware-budget gate, not a constant search) and G12's trio is observation-only;
  the faithful runner set is **G11 ×2 + G14 + G15**. The mismatch is recorded here,
  not papered over by inventing a G6 curve.
- **Status:** **OPEN (2026-10-25, B12)** — the owner rules the enumeration; until
  then every curve-bearing sweep is a ledgered refusal and no leg quotes a magnitude
  from one.

## OD-193 — composition cannot surface at this build: three independent blockers *(B12; parent G11(f), G11(i))*
- **Where the law is.** [05 §16](../../engineering/05-gate-harness.md) leg (f): teach
  `apple → red` and `red → warm` as two facts, ask `is an apple warm?`, and the answer
  must surface `warm` with an INFERENCE/OWNED label citing **both rows and the two
  edges in traversal order**; `route_counts.owned = 2 ≤ INFERENCE_MAX_HOPS_OWNED`.
  [04 §5.4](../../engineering/04-algorithms.md) rule 1 is the citation shape.
  [03 §3.2](../03-birth-state.md) is the closed nine-frame table.
- **The finding, measured.** The §16(e)–(i) scene was run as a probe on this build
  (`harness/scenarios/g11_composition.txt`, seed `0x51`, V0; run
  `gates/G11_si_20261025-1250`). It shows **three independently sufficient** reasons
  the composite cannot appear:
  1. **No yes/no frame exists.** The table decodes `P_ATTRIB`, `P_DEF`, `P_QUEST_A`,
     `P_QUEST_D`, `P_QUEST_W`, `P_OF_REL`, `P_CORRECT`, `P_CMD_C`, `P_TEACH` — and
     nothing shaped `is <E> <V>?`. `is an apple warm?` decodes to
     `{"frame":"UNDECODED","pattern":9,"slots":[]}` and the session returns
     `Output::Inert{reason:"input_not_a_teach"}`
     ([lib.rs:1856](../../crates/si-session/src/lib.rs#L1856)); it never reaches the
     read path at all.
  2. **The read path cannot cite a far row.** `retrieve` keeps only records whose
     `key_point == slot.point`
     ([retrieve.rs:159](../../crates/si-read/src/retrieve.rs#L159)), and the answer is
     `cited[0]`. So *any* decodable question about `apple` can cite only rows keyed on
     apple. The labeler's INFERENCE (a) branch requires **≥ 2 claim rows**
     ([label.rs:149](../../crates/si-front/src/label.rs#L149)) — unreachable by
     construction when the second row is keyed on `red`. Measured with
     `what is apple?` on the same two teaches: answer `red`, label `FACT`, citation
     `[1]`, `route = {owned: 1, base: 0, hops: 1}`; the `red → warm` edge is never
     traversed into a citation.
  3. **`TAU_SETTLE` at its placeholder settles the walk after one round.**
     `TAU_SETTLE` is a G2 row declared `1 … 65535`; its placeholder is **32768**
     (OD-86's midpoint), while a round's activation is O(2048–4096 Q12). §04's stop
     rule settles when the round-to-round difference is below `tau`, so
     `diff ≈ 3072 < 32768` ends every thought at `k = 1` — measured: thought 2's
     trace stops `Settled` with `settle = [apple, red, frame]`; `warm` is never lit.
     A two-hop route is therefore impossible until **G2's sweep** publishes
     `TAU_SETTLE` (OD-192), which is a cross-gate dependency §16 does not name.
- **What this does not claim.** The composite is *not* refuted as a design; it is
  unreachable in this build. (f.1–f.5, g.1–g.3, h.1–h.4 and i.1–i.3 are all graded
  against a route that cannot exist, so they would be a red gate, not a finding about
  the architecture.)
- **Options.** (a) Add a yes/no frame to 03 §3.2 (`P_YESNO: is <E> <V> ?`), which is a
  spec edit and a decoder change; then change `retrieve` to cite the route's reached
  rows rather than the slot-point rows (a read-path change; the labeler's ≥ 2-row
  INFERENCE branch already exists), and wait for G2's `TAU_SETTLE` sweep (OD-192).
  (b) Keep composition graded but re-scened as a decodable question — impossible
  without either a new frame or a taught relation word, and the latter teaches the
  composite, which (e).2 forbids. (c) Declare composition out of this build's scope
  and name G11's (e)–(i) UNMEASURED until the frame and read-path questions are ruled;
  G11 then grades only the plasticity half. (d) Build a second read path for
  composition only, which duplicates §04's walk and is the workaround this ledger
  exists to refuse.
- **Build's proposed reading (evidence, not a ruling).** (a), sequenced: the frame is
  a one-line spec addition and the two-hop citation is the architecture's own claim
  (04 §5.4); the TAU_SETTLE dependency is real and stays blocked on OD-192. Until the
  owner rules, **the build does not invent a yes/no frame**, does not widen
  `retrieve`'s citation filter, and does not write a G11 evaluator that would grade
  legs against an unreachable route.
- **Status:** **OPEN (2026-10-25, B12)** — the owner rules the frame and the citation
  rule; G11(f)/(g)/(h)/(i) stay UNMEASURED with this entry as their blocker.

## OD-194 — B12's DoD says "G11 full", but G11 has no evaluator and its median half is OD-144 *(B12; parent OD-191)*
- **Where the law is.** [engineering/11 §4](../../engineering/11-build-plan.md) B12's
  DoD 3: *"G11's composition and median sweeps and G14's panel report are produced
  here and **G11/G14 are full**"*, with exit gates **G11, G14 (full)**. [05 §16](../../engineering/05-gate-harness.md)
  writes 28 legs (a–p).
- **The finding.** The dispatcher has no G11 arm (it bails at
  [gates.rs:72](../../crates/si-cli/src/harness/gates.rs#L72)), so no G11 report has
  ever existed. The sweeps now exist but refuse (OD-192), so the two artifacts the
  legs name cannot grade a cap or a median. More fundamentally, the gate's two halves
  are blocked by other open questions: (b)/(c) need the read-side median H, which is
  **OD-144** (kind 7's row has no producer; `si-affect::drive::PointMedian` exists and
  nothing computes it), and (e)–(i) are **OD-193**. The legs that could be graded
  from one V0 session — (a) teach-state addressability, (e).2's fixture hash, (g)'s
  broken chain, (p)'s parked promotion — are a minority, and writing a report for them
  would make a *red* family look partially green while its two named blockers wait.
- **Options.** (a) Write the evaluator now for the gradeable legs and name the rest
  UNMEASURED — B12's exit gate then reads `pass: false` and the family is red for
  reasons OD-144/OD-193 already own. (b) Defer the evaluator to the stage that builds
  H and the composition frame, and record G11 as a named wall here (what this pass
  does). (c) Amend B12's DoD to "G11's runners and artifacts", which is what the work
  order's Stage 1 actually asks for.
- **Build choice:** **(b) recorded, not decided** — no G11 evaluator is invented; the
  wall is this entry plus OD-193 and OD-144, and the acceptance pass reports **19 of
  20 families** (G5 now has an evaluator after B13.1; G11 does not).
- **Status:** **OPEN (2026-10-25, B12)** — the owner rules whether G11 grades
  partially now or waits for H and the composition frame.

## OD-195 — the G6 SI pair's handle probe carried `G5.c.1`, the id 05 §10 gives the string audit *(B13.1; parent OD-190)*
- **Where the law is.** [05 §10](../../engineering/05-gate-harness.md) names
  `G5.c.1` the static string-constant audit and `G5.c.2` the **handle audit**
  — "CONFIG SI provably never opens a frozen-layer file" (`check_config_guard`,
  run in both configs as a pair). [09 §3.2](../../engineering/09-teach-channel.md)
  likewise calls `G5.c.1` the three-literals check.
- **The finding.** `g6_si_pair` (`crates/si-cli/src/harness/gates.rs`) emits the
  **runtime open-register probe** — an SI wake with the artifact on disk, zero
  frozen opens, a positive control — under the criterion id `G5.c.1`. That has
  been the id since B10, and `docs/decisions/changelog.md`'s B10 entry records
  the report faithfully. B13.1's evaluator now grades `G5.c.1` as the
  string-constant audit, so reports of the same build can carry two different
  checks under one id, and a G6 SI report claims a leg 05 §10 assigns elsewhere.
- **Options.** (a) Leave the probe's id and accept the collision across gates —
  byte-strict names stop being byte-strict between two reports of one run.
  (b) Relabel the probe to `G5.c.2`, the id 05 §10 gives the handle audit and
  whose runtime half this probe is; G5's own `c.2` (the static half) then names
  the runtime half and its report in `observed`. (c) Delete the probe — rejected:
  it is the only *runtime* measurement of c.2's claim, and G6's SI pair is where
  V7's row puts it.
- **Build choice:** **(b), executed in B13.1** — three id sites in `g6_si_pair`
  relabeled `G5.c.2`; `g5`'s c.2 observed gained the `runtime_half` pointer. No
  claim text, threshold, or behaviour changed. The changelog's B10 line is left
  as the historical record of the report it described.
- **Status:** **OPEN (2026-10-25, B13.1)** — the owner confirms the reading; the
  relabel is reversible and touches no measurement.

## OD-196 — G13(e) names "one leg in ENGINE", but no variant or evaluator gives G13 an engine run *(B13 acceptance; G13(e), 05 §18)*
- **Where the law is.** [05 §18](../../engineering/05-gate-harness.md) heads G13
  *"CONFIG SI, one leg in ENGINE · V0 + V9 + two sweeps"*, and (e) reads: *"the
  same scenario run in CONFIG ENGINE, and a `SIgen` checksum before/after N
  sleeps"* — `G13.e.1` L1/L2 unchanged for any N, `G13.e.2` sleep's behavior
  leg-for-leg identical across configs and L3 the only layer it wrote.
  [07 §9](../../engineering/07-genome-build-plan.md)'s last paragraph makes `e.1`
  a byte-compare of the sealed content tag after N sleeps.
- **The finding.** The engine half has no home in the harness. §2's **V7** row
  assigns the engine config to **G6** and G5(c)'s handle audit only; §5's variant
  matrix names **V0 + V9** for G13; and `g13` ([gates.rs:6037](../../crates/si-cli/src/harness/gates.rs#L6037))
  builds every probe as a plain `DriveRequest` — no `config`, no genome — so its
  instrument is CONFIG SI by construction. Both legs are hard `open` with
  `engine_config_runs: 0` and a B8 reason. What changed since B8 is the input,
  not the instrument: B10 landed the loader and the published fixture
  `data/genome/fixture/out/90eae6244a63a99e` (seal
  `9ea9634a24e6ea90edae0c2daca10b9f`), so this scenario *can* be launched with a
  genome in the handle space — but no requirement ever launches it, and the leg's
  comparison cannot be a digest (OD-91: two launches with different handle spaces
  are not comparable by hash), so grading it needs the sleep ledger and the
  L1/L2 boundary read side by side.
- **Options.** (a) Add V7 to G13's row and build the instrument: launch
  `g13_sleep.txt` at CONFIG ENGINE with the fixture genome, checksum L1/L2
  before/after N sleeps, and compare its sleep ledger leg-for-leg with the V0
  run's. (b) Amend G13(e) to cite G6(a.1)'s checksum as the engine half and drop
  the cross-config pair. (c) Leave both legs UNMEASURED with this entry as their
  blocker.
- **Build choice:** **(c) recorded, not decided** — the acceptance stage invents
  no engine instrument; B13's plan row is "no new code" and B13.1's authorization
  (OD-190) is G5's e.1/f.1 only. Every G13 report keeps `e.1`/`e.2`
  `unmeasured`; **this entry is their blocker from now on** (the report's reason
  strings still carry the B8 wording until the owner rules, because changing
  them is code and B13 is "no new code"). G13 still **passes**: 43 of 47 legs
  graded at V0.
- **Status:** **OPEN (2026-10-25, B13 acceptance)** — the owner rules whether the
  engine instrument is built (a), the law's (e) is amended (b), or the pair stays
  a named wall (c).
