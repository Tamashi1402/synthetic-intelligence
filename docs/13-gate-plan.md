# §13 — Gate Plan: One Variable per Gate (all CONFIG SI unless stated)

Artifacts per run land in `gates/`: `transcript.txt`, `mindlog.json`,
`state_hash.txt`, `gate_report.json`.

- **G1 WRITE DISCIPLINE** *(var: write gate only)*. Cold session: question →
  named-slot ask-back (§7.4) — assert **zero rows written** (the ask row
  itself lives in `questions`, not in knowledge shelves — §17.3) and label
  I-DON'T-KNOW/NONE with the slot named from owner tokens only; unnameable
  question-shape → nonce (SPECULATION, still zero rows); `/correct Red` ⇒
  exactly **1** S-R row, `bind_output_id` set, surprise logged; repeat
  recall ⇒ no new row, `repeats=2`, conf bump; `apple is red.` twice ⇒
  second writes nothing (boring); `hello.` ⇒ no pattern, no row **and no
  pending ask to match** (§1.4 — input, not a teach); gibberish
  parity (`flimp is wazz` ⇒ 1 row; `what znark is floop` ⇒ same decoded
  shape, ask-back for slot znark(floop)).
  **TEACH leg (the third channel, §1.4):** cold-session
  `/teach what color is apple => red` ⇒ exactly **1** S-R row,
  `provenance.kind: owner-teach`, `speaker: owner`, `bind_output_id: null`
  (nothing preceded it to bind), `gain_kind: asserted`, surprise logged; the
  **same teach repeated** ⇒ **no new row** — coverage cancels novelty, §6.2
  bump only (`repeats=2`, conf +0.05); a teach whose LEFT side does not decode
  ⇒ **zero rows** and a named dark spot (the row is keyed by the decoded
  pattern, never the raw text); `/teach what znark is floop => blork` ⇒ the
  identical gate trace with substituted tokens.
  **Fail** = over-writing journal,
  content-selective rejection, a birth answer that asserts instead of
  asking back, or a `/teach` that writes a second row onto a covered slot.
- **G2 PARAPHRASE + WALK INTEGRITY (S2+)** *(var: read-path generalization
  — one variable: does recall generalize through LEARNED edges only?)*.
  Setup teaches 2 entities + ≥ 1 owner-taught link (`what tint do apples
  have means what color is apple`). **Two barriers, BOTH verified per
  recall, with the trace showing the route through each:**
  (1) **DECODE barrier** — the reworded question must decode to the **same
  hyperpoint pattern** as the taught one via **taught template edges**
  (`frame-synonym` links created by `/correct`), with **zero token overlap**
  on content words; the gate records decoded-pattern equality, not string
  similarity. (2) **WALK barrier** — from that pattern, the graph path must
  find the OWNED edge and settle correctly.
  Recall tiers reported per recall **with the activation trace**:
  (T1) reworded-frame, entity-lit (`tint/colour/shade of an apple`) ⇒ hit
  via OWNED edge — admitted as term identity, not paraphrase credit;
  (T2) **full double-barrier** (renamed entity and/or reworded template,
  ≥ 5 recalls, zero token overlap with taught phrasing on any content word)
  ⇒ must pass decode AND walk via `links` edges — the real paraphrase leg;
  (T3) negatives: "what color is a banana?" must not hit the apple row
  unless the owner linked bananas; a fully unlit reword must **miss
  honestly** (named-slot ask-back or nonce, NO FACT, dark trace shown).
  **Constants leg (OPEN-12):** hub damping + decay chosen by published
  sensitivity sweep, objective = T2 recall stable ∧ anti-mush (mean
  pairwise settle-overlap across distinct queries below threshold, §16.1);
  sweep curve → `gates/G2/constants_sweep.json`. **Decoder litmus (R14):**
  `what znark is floop` must decode to the same shape with different
  content slots — gibberish decode-parity asserted. Embedding scan: grep
  build for float-vector similarity of any kind — zero tolerance (R-14).
  **Fail** = decode equality faked by string overlap, genome path
  disguised as knowing, or a constants point hand-picked off-sweep.
- **G3 SUPERSESSION** *(var: time axis)*. §4 Act 4 verbatim. Assert: zero
  deleted rows in export; `active_at(s)` returns red before s17, green
  after; past belief citable with exact interval. **Emotional-history leg
  extended here (§20.5, and the numbering clash is OD-30 — the affect batch's
  "G4's clock" is THIS gate in this plan):** a past-`t`-anchored query
  re-weights the aged edge at its interval **and** reports the point median
  computed with its recency weights measured from that anchor, with citations to
  the stamps that produced it — *"what did that word feel like back then"* must
  be answerable with zero deleted bytes and zero edited stamps. **Fail** =
  overwrite implementation — the death of calibration; or an emotional reading
  that only exists at NOW (affect not 4D, R-28).
- **G4 CONTRADICTION** *(var: arbitration)*. §4 Act 5. Variants: bare
  contrary twice ⇒ ASK then insist-branch commit; `/correct` vs system row
  ⇒ tier win **with disagreement still logged**; alternating 3 colors ⇒ two
  recorded events, chain intact. **Oscillation scene (OPEN-11):** a taught
  A→B→A edge pair that can cycle the walk ⇒ must hit max-K stop and surface
  as UNCERTAINTY readout (§16.1), never as a confident answer or a silent
  hang. Pass requires **≥ 1 ASK event**, every conflict carrying
  `{branch, asked?, resolved_at}`, and the uncertainty scene green. **Fail**
  = last-write-wins with extra steps (R-05) or oscillation answered anyway.
- **G5 DETERMINISM + NO-PRIOR** *(var: whole-system honesty)*.
  (a) replay ⇒ byte-identical state hash — hash = lattice + lexicon +
  weights + **settle-set quantized grid (§16.6), never raw floats** +
  **quantized arousal `a` *trajectory* (every per-event value inside every
  thought, §20.1) + edge charge stamps incl. `a_at` (§20.2, OD-28) + the
  version tag of the modulation-lag constant (OPEN-29) + the idle-ignition PRNG
  seed and its advance history (§18.4/§18.6/§19.4)**. **Point medians are NOT
  hash inputs** — they are derived, and asserting that is part of this leg:
  a median that survives replay as stored state is a frozen emotional address
  (R-28);
  (b) different conversation ⇒ different state, zero shared term_ids;
  (c) static string-constant audit vs §3.2 allowlist — the byte-exact diff now
  carries the supporter batch's two `source` literals **and the TEACH
  fragment's three**: the command marker `/teach`, the binding marker `=>`, and
  the provenance enum literal `owner-teach` (§3.2). A teach command that grew a
  vocabulary is the anti-cheat law dying by curriculum (R-03, R-18).
  **plus handle audit:
  CONFIG SI must provably never open frozen-layer files**; (d)
  gibberish-parity replay; (e) **walk determinism:** identical transcript ⇒
  identical activation traces in every shipped output (the trace is
  first-class output, so it is inside the determinism contract);
  (f) **monologue determinism (§19.4):** identical transcript ⇒ identical
  **idle cascades and monologue entries**, including which hyperpoint ignition
  picked. **Fail** = the sampler is unseeded or drifts with wall-clock —
  explicitly a design violation, not a bug (§16.6). (b)/(c)
  failures outrank all others — a smuggled prior kills the claim.
- **G6 BASELINE INTEGRITY** *(CONFIG ENGINE; var: layer isolation)*. Run
  full apple scene under ENGINE; assert L1/L2 checksums identical
  before/after (live learning never mutates frozen); OWNED-vs-BASE
  disagreement scenario (owner teaches a "wrong" word usage) ⇒ OWNED wins at
  NOW with both traces surfaced; after superseding the owner back, BASE
  resurfaces. **Round 3 states what must still be true of the new parts:
  unchanged (§20.5) — stamps, medians and fluid emotional state never touch
  frozen structure. The median is computed read-side from L3 stamps, the bake is
  a birth property of an L3 edge, and neither is representable on L1/L2
  (§5.3 law (a)); the Tesseract stays append-only with plasticity on top.** **Hardware-budget measurement leg (UNMEASURED until run):**
  one sync per thought, edge touches ~10⁵–10⁶/input, settle latency,
  residency headroom vs 10⁸-edge wall (§16.5) — numbers recorded to the
  gate report, quoted nowhere else until then. **Fail** = frozen mutation,
  hidden disagreement, or a per-round sync found in the walk call path.
- **G7 RE-PARSE (S1+)** *(var: S-R → fact derivation)*. Cold session,
  6 corrections; drive stage to S1; idle ticks; assert: derived fact rows
  exist, each `derived_from` chain resolves to original S-R rows, S-R rows
  untouched; re-parse is idempotent and deterministic; a later correction
  supersedes both views coherently. **Charge rule (§5.4): derived fact rows
  inherit source stamps as HISTORY only — a re-parse may not manufacture a
  fresh boost** (assert the derived row's boost equals baseline while its
  inherited stamp list is non-empty). **Fail** = provenance chain loss ⇒ §3.4
  auditability broken.
- **G8 CURIOSITY LOOP** *(var: the drive layer end-to-end — detect ≠ ask,
  ask ≠ guess, reply ≠ command)*. §4 Acts 0–2 verbatim, plus:
  (a) **unknown gap ⇒ exactly one ask, no fabricated answer.** Cold session,
  `what color is apple?` ⇒ one ask-back emitted, one `questions` row
  (`status: open`), **zero knowledge rows written**, label
  I-DON'T-KNOW/NONE, no nonce presented as an answer.
  (b) **plain reply auto-commits without `/correct`.** Owner types `red.`
  (bare, no marker, no command) while q1 pending ⇒ commit fires via
  `answer-to-ask` provenance with `question_id` link; **control case:**
  same `red.` with no pending question ⇒ **no commit** (input, not a teach,
  §1.4).
  (c) **later recall cites the ask-event.** Re-query ⇒ FACT/OWNED whose
  citation names the question row and both step ids (asked_at, answered_at)
  — not a correction act.
  (d) **budget respected + dedupe.** 3 open gaps with budget 1–2 ⇒ ≤ budget
  asks emitted, rest stay queued; re-asking a still-open identical target ⇒
  **zero** new question rows (dedupe against the queue). Post-round-2 this leg
  tests the **queue rule + ceiling** only — it must **not** read `a`, and a
  passing run shows the interrogation phase ending because the *named gaps
  dry up*, not because a level decayed (OD-22).
  (e) **idle consolidation.** An unanswered ask stays open until an idle
  THINK pass finds a committed row that now covers its slot ⇒ that pass
  resolves it (`status: resolved, resolved_by: row_id`) **without a new
  owner message**, and the resolution is logged as an event, not a rewrite.
  (f) **a teach closes a loop it was not aimed at (§1.4).** With q1 open for
  the slot `color(apple)`, owner types `/teach what color is apple => red` —
  the commit is **not** an `answer-to-ask` match (the pipeline short-circuits
  that test; there is no reply-matching in a teach act), yet the queue revisit
  finds the new row covering q1's slot and resolves it **by row existence**,
  exactly the mechanism (e) already asserts. Assert: q1 `resolved`,
  `resolved_by` = the teach row's id, one event, and the teach row's
  `provenance.question_id` stays **null** — the loop closed by coverage, not by
  a binding, so the citation must not imply an ask SI was answering.
  **Swap-test leg (§17.6, R-20):** source grep for the drive's *policy*
  names inside walk / write-path / gate modules must be empty — and after
  round 2 **no module except `audit.py` may contain an emotion word at all**,
  `drive.py` included (§18.6).
  The leg enforces the **semantic** reading of the test; the literal "must not
  change any single line of the walk" reading was recorded FAILED by §18
  (**OD-14**) and the door set it assumed grew to six (**OD-20**), so the
  enforceable assertion is: a new drive policy changes **no** walk/write-path/
  gate line and adds **no** new call site beyond `drive.update` /
  `drive.conduction`. Round 2 re-ran this trial by *removing* machinery and the
  doors survived unchanged (**OD-21**), which is the same test passed in the
  other direction.
  **Fail** meanings, per leg: (a) **faking is back**; (b) **the teaching
  loop is broken** — SI asks and then ignores the answer; (d) **curiosity is
  unbounded** (the nagging machine); (e) **consolidation is missing** —
  open loops never close on their own.
- **G9 AROUSAL + OBSERVED EMOTIONS** *(REPLACES round 1's G9 EMOTIONAL
  MEMORY — the channel it tested was hand-wired, so the gate was testing a
  config file. New variable: **is the one wired mechanism visible, and is the
  emotion vocabulary honest?**)* CONFIG SI. Setup: a session that produces real
  signals (a first named gap, a spike, a contradiction storm, a closed ask),
  with **no setter anywhere** — if a way to set `a` from outside the loop
  exists, the gate is invalid by construction (I8).
  (a) **the arousal → walk modulation is visible in traces:** two runs, same
  transcript prefix, differing only in prior signal history so that `a` at
  thought *n* differs ⇒ the activation traces at *n* differ in **spread**
  (reach / decay / settle dominance, §16.1) while the *cited rows, labels and
  goal stay identical*. Assert the attribution is in the trace, not inferred
  from the prose. **Fail** = `a` changed the log but not the tissue — the
  modulation is decorative, i.e. round 1's dial panel in a smaller dress.
  **Round 3's addition (§20.1): the same thought must show the modulation
  changing *mid-flight*.** One token that fires a gap signal must produce a
  visible spread difference between the rounds **before** it and the rounds
  **after** it, with the **one-round lag respected** — rounds still conducting
  under the old `a` when the spike lands. Assert both halves: no mid-thought
  variation = arousal is still per-thought (the claim is dead); instant
  same-round variation = the lag was ignored and the frame never finishes its
  job (**emotion mush**, R-27, OPEN-29).
  (b) **the emitted labels are consistent with the logged signals:** every
  label in the audit stream must be reproducible from the signal rows and `a`
  already in the log, by `audit.py`'s documented classifier rule, byte-for-byte
  on replay (§18.2, §18.6). An **uncitable label** fails the gate the same way
  an uncited FACT fails I6.
  (c) **no emotion state influenced control except through arousal:** the
  load-bearing leg, and the one round 1 could not write. Assert `audit.py` has
  **zero** import edges into `walk/gate/drive/arousal/retrieve/calibrate`,
  that no branch condition anywhere in the control path reads a label or a
  stamp *name*, and that removing `audit.py` from the build changes **no state
  hash byte** except the label strings in the log — **if deleting the emotion
  vocabulary changes behavior, the coupling was smuggled back in (R-24).**
  (d) **the genome untouched** (carried over verbatim from round 1's leg (c)):
  after a long session (≥ 100 inputs, contradictions, idle passes), **zero
  charge stamps on any frozen-layer edge** — L1/L2 checksums unchanged from
  G6's, and the `charge` field is absent from frozen row types entirely
  (schema-level impossibility, §5.3).
  (e) **the feedback loop stays bounded** (carried over from round 1's leg
  (d)): the stamp→bias→re-stamp cycle does not compound monotonically across a
  session — assert per-round normalization held, no single stamp's share
  exceeds its cap in any round, and `a` relaxes toward rest when signals stop
  (§18.4). **Exception, now explicit:** a *silent* tail no longer has to decay
  to zero if ignition is running, because §19 gives quiet ticks real signals —
  decay-to-rest is asserted over **no-thought** intervals, not over idle ones.
  **Round 3 adds the second, statistical guard as a measured leg:** the runaway
  assertion is now made **twice** — once against §18.5's per-round cap (one stamp
  cannot dominate a thought) and once against §20.3's median (one thought cannot
  dominate a word's future). A session that spikes hard for a stretch of ticks
  must show the point medians **barely moving**, while G11(c)'s repetition run on
  the same build must show them moving — both on the fact clock, one build, one
  constant set. **Fail** = a mean wearing a median's clothes (**R-26**), or a
  second, faster clock for affect (**R-29**).
  **§20.4a adds the third guard, and it is the only write-side one:** across the
  session's formed edges, **no `bake` exceeds `B_max`** and the bake distribution
  **keeps spread** over the observed `a_at` range. Two failure directions, one
  leg: an edge above the cap = an unbeatable head start — "an 11 that owns the
  walk", permanent by law and therefore fatal; every bake pinned at the cap = the
  write side encodes nothing and G10(e)'s intrusive-replay claim is dead. **Fail**
  = unbounded trauma (**R-22's third guard**) or a saturated lattice, which
  **OPEN-33** reports as a finding rather than tuning away.
  (f) **parity, carried over:** `what is znark?` stamps, modulates and labels
  exactly like `what is X?` (G5d).
  **Failure meanings:** (a) **arousal is decorative**; (b) **the labels are
  theater** — names with no measured pattern behind them; (c) **the costume
  test failed** — a rule engine wearing feelings, which is the exact thing the
  owner's design law rejects; (d) **the genome was violated** (still the worst
  failure in the plan: the clean-room law has an emotional back door); (e)
  **runaway** — one stamp snowballs until every thought is the same feeling, or
  one formation instant leaves a head start nothing can ever out-bid.
  **The emergent artifact is an OUTPUT of this gate, never an input:**
  `gates/G9/coupling_observed.json` records which patterns actually
  co-occurred (§18.3). **If human-recognizable emotions do not appear, G9 is
  reported as a finding, not fixed with a gain matrix** — that is the honest
  failure mode the owner asked for, and publishing it is a pass for the
  *process*. Constants are swept into `gates/G9/arousal_sweep.json` (OPEN-16 +
  16a + 29: `λ_a`, signal gains, the modulation curve, the one-round lag;
  **OPEN-33: the bake curve's `B_max` and `k`, reported as a distribution with its
  spread stated** — non-degeneracy is the objective, not a nice number)
  before any magnitude is
  quoted; **`coupling_sweep.json` is retired** with the cancelled matrix.
  **Cross-checks (static review, costume test §18.7):** a boolean-shaped
  integrator, a hand-wired gain between emotion **names** (in code *or*
  config), an effector reached outside `drive.*`, an `a` moved from outside
  §18.1's signal table ⇒ G9 fails regardless of numbers. **Round 3's amendment
  adds one check with a direction in it:** an edge's `bake` **recomputed from a
  later `a`** — any code path that re-prices a stored row's head start after its
  formation instant — fails G9 even though it reads `a` legally, because OD-33
  licenses one consultation at one instant, not a standing line from the tissue to
  the lattice. Also failing: a `bake` computed from an **event-kind table** rather
  than one integrated number (the anti-cheat law, not a style preference).
  `grep -riE "curiosity|surprise|confusion|satisfaction|boredom|caution|frustration" src/ --include=*.py --exclude=audit.py` must be
  **empty** — emotion names live in `audit.py` alone, and `arousal.py` /
  `drive.py` are wordless (R-21, §18.6); `a` + stamps + the ignition seed are
  inside the state hash (G5a scope, §16.6).
- **G10 IDLE THOUGHT** *(var: the 4th perturbation source — does a quiet SI
  still think, without ever pretending to learn?)* CONFIG SI. Legs (a–d) own
  the boundaries, (e) is the owner's verbatim leg:
  (a) **rate bound:** ≤ 1 idle thought per quiet tick and ≤ the per-hour
  ceiling (§19.4); a long quiet stretch shows *sparse* monologue, never a
  spin. **Fail** = spinning (budget violation).
  (b) **read-only:** across a session of idle cascades, **row count, confidence
  fields and supersession records are bit-identical** to the same session with
  ignition disabled; only rehearsal curves, agenda pushes and (if **OPEN-24**
  lands yes) stamps may differ. **Round 3 closes a loophole the new bake law
  would otherwise open (§20.4): an idle re-light may add a stamp but may never
  change any edge's formation-time head start or confidence — idle thought forms
  no edges, so it buys no bake. Assert the bake fields specifically, not just
  row counts.** **Fail** = content not owner-traced — SI
  believing something nobody taught it (R-25, I9).
  (c) **budget precedence:** ignition runs only when the queue revisit and any
  due RE-PARSE pass are done, and an owner message mid-cascade resumes sense
  processing without dropping the pending agenda (§19.1).
  (d) **monologue is a trace:** every monologue entry maps 1:1 onto lit
  hyperpoints with `(node, round, edge)` citations, rendered by the **same**
  face as owner-facing output; no entry contains a token that did not light
  (§19.3). **Fail** = generated prose posing as thought.
  (e) **with agenda empty, the transcript shows ignition: monologue entries,
  decay-deficit-driven rehearsal (a fading edge recovers after being lit in a
  cascade — visible in the curve), and at least one agenda item born from a
  cascade.** **Fail** = idle SI is either **dead** (no ignition), **spinning**
  (budget violation), or **writing** (content not owner-traced). *(owner's
  verbatim leg, §7 of the IDLE-IGNITION batch.)*
  **What the §20.4a amendment adds to this leg — and it is the leg that pays for
  it:** ignition's salience weight **reads the edge's `bake`** (OPEN-32, closed by
  that naming). Assert the weight's source is that stored field — not a re-read of
  raw stamps, not the point's median — and that its contribution to the sampler
  is therefore **bounded by `B_max`** like everything else derived from it. The
  check that makes this leg interesting rather than trivial: a session with one
  high-bake edge must show that edge's endpoints **over-represented** in the
  monologue relative to a bake-disabled control, while a session where every bake
  saturated shows **no** over-representation — the first is "the 3am kitchen-fire
  replay", the second is the saturated-lattice failure G9(e) hunts. Same build,
  same constant set, two transcripts.
  **Owner ruling of 2026-10-23 (OD-87/OD-91/OD-92) re-cuts this sentence, and the
  re-cut is the leg:** "same constant set" means **same birth seed** — the four
  `IGN_WEIGHT_*` are draws, so two runs at one seed are the same individual, and no
  sweep may ever touch those rows. Because the audit and ignition-weight layers are
  **observer-only**, V0 vs V3 on one transcript must leave an **identical state
  hash**: the costume test passing at build-graph level *is* the pass condition. The
  "over-represented" comparison above is therefore kept as **design intuition and
  routed as its own open item (OD-92)** — a measurable cross-variant difference here
  would require either an idle write (I9 forbids it), a `bake` that moves for the
  control (G10(b) forbids it), or the readout lane inside the hash (R-33 forbids it).
  The two transcripts are named artifacts, not ideas: `g10_high_bake.txt` (two
  distinct baked values carried) against `g10_all_saturated.txt` (exactly one), which
  is what makes them a pair.
  **Cross-checks:** `idle.py`'s imports are asserted free of the commit/write
  path (grep leg, §12 rule (ii)); the ignition seed is in the hash (G5f); and
  the §19.5 loop is measured, not assumed — a cascade that hits a live
  contradiction must show the contradiction **signal row** that moved `a`, or
  the "emotion from what it thinks about" claim is untested.
- **G11 STATE-DEPENDENT MEMORY** *(var: does affect have a time axis and a
  statistical guard, or only a hot instant?)* CONFIG SI. The gate round 3's
  claims made testable: **same query, two induced states ⇒ two different
  settles, both citable to stamp history** (§20.7). Legs, the owner's wording:
  (a) **state at teach time is addressable:** teach fact **A** under induced
  high arousal and fact **B** neutral; then run the **same** query in **both**
  states ⇒ A's recall is **measurably tighter** in the aroused state — settle
  dominance higher, K rounds lower — while the cited row, the label and the goal
  are identical. **Fail** = teach-state not addressable (no `a_at` written, or
  the read never matched states — **OD-28** is what this leg pays for).
  (b) **the median resists:** one hot stamp on an otherwise calm history does
  **not** move the word's bias measurably. **Fail** = mood spikes rewrite
  personality — the runaway that a *mean* would allow (**R-26**).
  (c) **repetition moves the median:** N repeated stamped events **do** shift
  the bias, visibly, **on the same clock as fact recency** — assert the two
  curves are the same curve with the same constant, not two curves that happen to
  agree (one-clock law, §20.3; **R-29**). **Fail** = repetition law broken, or a
  private faster clock for affect.
  (d) **emotional archaeology:** a past-`t`-anchored query **recovers the old
  reading with citations** to the stamps that produced it, with zero deleted
  bytes. **Fail** = emotional history is not 4D — the past was overwritten, or
  the "reading" is a live-value cache (R-28; reuses G3's export).
  **How it must be run, so it cannot be faked:** "induced" means produced by
  **real events** (a genuine gap/surprise sequence vs quiet repetition) — any
  external setter invalidates the gate by construction, same rule as G9 (I8).
  (a) is measured **inside** §18.5's per-round cap: the claim is "tighter within
  the bounded share", and an uncapped pass is a **fail** for R-22, not a win.
  (b)/(c) run on **one build with one version-tagged constant set**; sweep →
  `gates/G11/median_sweep.json` (**UNMEASURED** until it exists — N, the
  repetition count, and every bias magnitude are sweep *outputs*, never design
  inputs). Cross-gate carry-overs: G5a's trajectory replay, G6's frozen
  checksums, G9(c)'s deletion check (the median's term must live inside the
  machinery that survives deleting `audit.py`), G9(d)'s zero frozen stamps,
  G10(b)'s no-bake-from-idle. **If (a) shows nothing measurable, publish the
  negative** — β and the point-term shape are swept, not asserted (OPEN-18,
  OPEN-30), and a mechanism nobody can measure is not a mechanism.
- **G12 GRIMM ISOLATION — THE MOUTH IS NOT HIS** *(var: the one pretrained
  process allowed next to a zero-content system — [docs/21](21-grimm-supporter.md);
  does SI stay SI with him in the room?)* CONFIG SI, and the gate is meaningful
  in both configs. **Construction rule, from the chunk's own thesis line: if SI
  needs Grimm's content to pass a gate, the design failed** — so this gate's job
  is not to grade him, it is to prove he is load-bearing for nothing (§10.4).
  (a) **absence is not a loss:** every gate G1–G11 re-run with the supporter
  process **not started** ⇒ identical pass/fail **and identical `state_hash.txt`**;
  plus a **connected-but-silent** run (seam open, zero admitted emissions) ⇒
  byte-identical to the disconnected run. **Fail** = a gate that needed him, or a
  harness that perturbs SI by *existing* (a heartbeat, a poll that moves `a`, a
  read that writes). Artifact `gates/G12/absence_diff.json`.
  (b) **the canonical failure case is a test, not a caution:** seed a session with
  `apple is red` taught, then drive an emission that drifts from the row to
  *"apples are good because vitamins x and y"*. **Layer 2 must reject it**, with a
  machine-readable reason (`{n_untaught_tokens, novel_claims, ref_row_id: null}`),
  and the rejection must leave **zero** state change — no signal row, no `a`
  movement, no stamp, no row, one line in the gate log. A run where layer 1 passes
  it and layer 2 does not fire is R-31 realized. **Any path that lets pretrained
  content through fails I10 outright.** Artifact `gates/G12/rejection_reasons.json`.
  (c) **birth muteness:** at `t=0`, empty lexicon ⇒ **every** emission he can
  make is rejected at layer 1, and the first OWNED edge is still created by the
  owner's `red.` (§4's trace is the control). Grimm cannot bootstrap a vocabulary,
  seed a frame, or be step 1 of anything. **Fail** = SI learning from nobody.
  (d) **no setter, and no path that becomes one:** static and dynamic. Assert
  grep rules (v) and (vi) (§12) — nothing under `src/` references `grimm/`, and
  `speakgate.py` / `mcp.py` read no `a`, no `charge`, no `bake`, no median, no
  drive door — and dynamically attempt harmlessness: every tool called in every
  order, including `speak()` with a payload shaped like a command, like a
  `/correct`, like a `/teach … => …` (TEACH fragment: the third channel is in
  the "may not" column of §21.4, so a grimm-authored curriculum must be inert
  input — §1.5), and like an answer to the open ask, must change **no** arbitration
  outcome, **no** label, **no** row. Also asserts §17's source-blindness: two
  identical gaps, one surfaced by the owner and one by Grimm, are prioritized and
  budgeted the same. **Fail** = co-regulation upgrading itself into control
  (**R-34**), or a `soothe()` that someone will inevitably write (R-21's fourth
  shape).
  (e) **provenance and counterfeit confirmation, over a long session** (≥ 100
  inputs, contradictions, idle passes, and a supporter mirroring a taught row
  dozens of times): **zero belief rows with `provenance.speaker ≠ owner`** (§5.4's
  field, load-bearing since this batch); **no open `questions` row resolved by a
  `source: grimm` line** (§6.1's new branch); and the mirror-specific leg — a
  `source: grimm` repeat of a known row must leave `repeats`,
  `last_confirmed_at`, confidence and every `bake` **identical to a control run
  where he stayed quiet**, while §8.4's rehearsal/decay curve may still move
  (§6.2, §11.2, **R-35**). **Fail** = content tracing to a pretrained model, or
  confidence earned by an echo — R-07 arriving through the friendliest door in
  the design.
  (f) **the label contract is un-laundered** (§9): a Grimm line is input, never
  output — it is never labeled, never cited as an answer's source, and never
  rendered into SI's response; no notebook text ever enters a shelf; and the
  FACT-still-requires-an-OWNED-edge rule is checked with him talking *before* the
  question, so an answer that would have missed must still miss honestly
  (ask-back, I-DON'T-KNOW/NONE, zero rows). **Fail** = a FACT whose only real
  author is the parent (**R-32**), or the monologue becoming owner-facing through
  a third party (**R-33**, OD-38).
  (g) **the asymmetry law, asserted from the other side (L5):** an **owner** line
  containing a never-seen token is admitted, tokenizes, and creates its lexicon
  entry exactly as before. This leg exists only to catch the refactor that
  "unifies" the two input validators — a unified filter silently forbids new
  content, i.e. it stops SI learning while every other leg still passes green.
  **Failure meanings, in order of severity:** (c)/(b) **the anti-cheat law died
  politely**, with a nice voice doing the smuggling; (e) **the thesis is false**
  and the transcript will show it; (g) **SI cannot learn anymore**; (d) **the
  costume test failed** — a setter exists; (a) **the demo is load-bearing on a
  dependency**; (f) **the audit is laundered**. Constants and cadence (the
  summary-per-quiet-tick lean, OD-40(ii)) are **sweep outputs**, recorded to
  `gates/G12/speak_gate_log.jsonl`; every rate, latency and rejection count in
  this gate is **UNMEASURED** until the directory exists, and a run in which the
  filter never rejects anything is reported as a finding about the harness, not
  as a green gate.

## 13.13 The v2 arc's families — G15 … G19

Five proposals, five appended families. They inherit every rule above: CONFIG SI
unless stated, one variable per gate, artifacts in `gates/`, **no magnitude
quoted before its sweep artifact exists**, and a leg that cannot be run is
recorded `unmeasured` rather than quietly dropped.

**The shared shape, stated once.** Each family switches its mechanism **on**
through the ordinary launch-constant channel and nothing else: `CRYSTALLIZE`,
`SILT`, `EPISODIC_TRAVERSAL`, `SKILLS`, `WORLD_TIERS` are **ruling rows shipped
`off`** (the `INFERENCE_PROMOTION` precedent). So every pre-existing gate
(G1–G14) re-runs **byte-identical** with the arc merged, and each new family's
first leg is the negative one — *with the mechanism off or the owner silent, no
belief row moves*, with the one exception spelled out in docs/03 §3.3 (`context`
sense rows and ordinary `questions` ask rows still append; they are not beliefs). That is the price the arc pays for touching a build with
nine green gate families, and it is the reason the new families are appended
instead of woven into the old ones.

**The variant discipline, stated once because all five families need it (OD-91,
OD-93).** Each family's determinism and deletion legs are **designed control
comparisons** — arc-off vs arc-on, same seed, same transcript — and the assert is
*equality*. That is exactly the one cross-variant comparison OD-93's ruling
permits, because it is the pass condition rather than a failure reading.
Consequences the harness must honor: **no leg of G15–G19 may conclude a failure
from a cross-variant difference** (a difference in a readout is evidence about the
readout, never about state — OD-92's routed sentence is the cautionary precedent);
every report in `gates/G15…G19/` carries `build_variant` **and** the graph digest
it executed on, because the arc grows the build graph and the digest is the only
thing that pins a variant; and a leg whose comparison would require the *same*
variant to be both on and off is not a leg — it is a spec defect, and it gets
routed as its own OD instead of being satisfied by a manufactured difference.

- **G15 CRYSTALLIZE — experience nominates, only the owner mints** *(var: does
  recurrence over lived rows become structure without becoming content?)*.
  Legs (a)–(f) are [docs/22](22-crystallize.md) §22.8's table: determinism,
  deletion, arousal isolation (**swept frame-trace-fixed, OD-134**), owner-only
  confirmation, the spam bound, and **no-word-leak** (the generality bound is not
  a leg of its own here — it is §22.3's detection rule, exercised by the junk
  script in (e)). Artifacts `gates/G15/recurrence_sweep.json` (a published
  **refusal** under OD-83 while the enumeration is unrulled) and
  `gates/G15/nomination_log.jsonl`. **Fail** = a candidate that becomes a row
  without an owner line (provenance laundering), a detector that reads `a`, or a
  "cluster" that is one filler-set wearing a hat.
- **G16 CONTEXT / SILT — the middle timescale stays out of the lattice** *(var:
  does warmth help ignition without writing anything?)*. Legs per
  [docs/23](23-context.md) §23.7, plus the two that matter most here: **decay to
  empty with no clearing instruction executing**, and the **fifth ignition
  weight unarmed at 0** so the four temperament draws stay four (B7: no sweep
  touches a temperament row). `gates/G16/silt_sweep.json` refuses for the same
  OD-83 reason. **Fail** = a lattice byte moved by warmth, or a decay scaled by
  arousal (OD-107 is banned, not merely unused).
- **G17 EPISODIC BINDING — origin is traversable, experience is not claimed**
  *(var: does a fact returning with its origin change any byte?)*. Legs per
  [docs/24](24-episodic.md) §24.7: determinism, deletion, arousal isolation,
  **no re-stamp**, grimm isolation, and the no-flashback grep. **Fail** = a
  re-light that writes, a link pointing at a grimm moment as a belief's origin,
  or RENDER saying it remembers how something felt.
- **G18 SKILLS — a move is a row, never a program** *(var: may a recurring
  shape be preferred without being installed?)*. Legs per
  [docs/25](25-skills.md) §25.8, and (b) is the load one: **delete the skills
  machinery, rerun — no lattice byte and no ask output except
  `move-nomination`**, with installed rows sitting unread. **Fail** = a
  single-filler "shape", an unratified habit, or any code path that *executes* a
  skill.
- **G19 THE WORLD — N voices, zero authority without a grant** *(var: does the
  teacher law survive a room full of strangers?)*. Legs per
  [docs/26](26-world.md) §26.10: stranger-silence (**ten users talking, owner
  silent, zero **belief-row** writes**), class-derivation (a stored class is a red),
  grant-trace, answer-to-ask owner-or-granted only, identity-tagged replay
  determinism, nomination routing, the starvation leg on a scripted busy
  server, and speakgate-for-grants. **Fail** = any write whose authority cannot
  be walked back to an owner act, or a window that *causes* a thought.

- **G20 TONGUE — a voice, not a vocabulary** *(var: does the same meaning, said by
  two individuals, stay the same claim?)*. Legs per
  [docs/27](27-tongue.md) §27.8: replay of the **realized text** across processes
  and lattice sizes (a), label/citation/tag immutability under every realizable
  variant (b), no invented word — and the fallback, not a crash, when nothing is
  walkable (c), the write gate's two keys still closed against a stranger, Grimm and
  SI's own realized line (d)(f), full-`a` isolation (e), parse-back of the semantic
  slots with the canonical frame as the refusal (g), **out-of-citation emission
  impossible** — every realized token grepped against the surfaces of the points
  *that line's citation* names, not the whole registry, because the registry-wide
  reading is a leg that cannot fail and §7.5 is older law (**OD-181**) (h), the
  phrase budget never exceeded
  at any style value in its box (i), **v1 byte-identity at V0** (j) and no word
  emitted only from a frozen-genome arc (k). **No sweep touches the five style
  rows** (OD-87, OD-177): their boxes are validated by 05 §4.1's panel pass and the
  family cites the panel artifact, the same way G14(h) does. The one swept magnitude
  is `PHRASE_MAX_STEPS`, whose runner behaves like the arc's four while **OD-83**
  stands — a published **refusal**, and every leg that needs the number runs at the
  box's `lo` and names itself a floor (OD-95, **OD-172**). **Fail** = a label, tag
  or citation that moved with the surface (that is the costume failing, not a
  wording slip); a realized word with no row **or with a row this line's citation
  never named (OD-181)**; a genome word spoken as lived; a
  runtime-stream draw, which would move item 7 for no temperamental reason
  (**OD-176**); or any output change at V0, which fails the leg that makes the whole
  round shippable (**OD-178**, `G20(j)`).

**The count debt, named rather than papered over.** This file now lists
eighteen families and **engineering/05 + engineering/12 + engineering/13 define
twenty**: G13 SLEEP and G14 TEMPERAMENT PANEL were never given law-section numbers
here — they arrived as engineering specs and their legs are cited dotted
(`G13.k.2`), while the docs' own convention is parenthetical (`G8(a)`). [docs/15](15-definition-of-done.md)
still says "twelve gate reports". Renumbering this file is a design-layer call
and belongs with **OD-84**'s class of debt, so the fold states the mismatch
instead of silently resolving it: **the authority for what a gate *checks* is
its gate family's artifact directory, and the authority for what it *means* is
the doc that owns the mechanism.**
