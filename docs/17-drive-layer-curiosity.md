# §17 — The DRIVE Layer + Curiosity (the default drive policy)

Owner rule: **SI is curious by default — but curiosity is the first RESIDENT
of a dedicated DRIVE layer, not a hardcoded reflex welded into the read
path.** The drive layer is a defined interface; curiosity implements it and
is the default drive at birth, **CONFIG SI and CONFIG ENGINE alike**.
This chunk's test was the swap test at the bottom — **and the EMOTIONS chunk
(§18) is that test's first real trial.** Round 1 of §18 installed five more
residents; round 2 of §18 **demoted them to audit labels**, so the trial's
honest verdict is now: the effector doors survived the swap unchanged, the
two §18 doors survived but shrank (`update` integrates one scalar, `conduction`
reads stamps), and **one pre-existing door lost an input it was promised**
(`budget`, → **OD-22**). Round 3 (§20) is the trial's third run: **still six
doors, still two door-calls from the walk** — the median entered as a return
value, not as a door (§17.6). Verdict table in §17.6.

**Curiosity means exactly two things in this design and must never fuse them
(§18.7):** (a) **drive policy** — detect gap → spend an ask within budget, this
file's subject, unchanged; (b) **audit label** — the name for (arousal up ∧
novelty active ∧ asks emitting), which has zero causal power. There is no third
thing called curiosity that modulates control.

## 17.1 The DRIVE interface (a horizontal layer, not an organ)

Sits between the read path (which *detects* dark spots) and the output +
write paths (which *act* on them). Contract, at minimum:

| method | contract | who calls it |
|--------|----------|--------------|
| `prioritize(gaps, now)` → ranked list | which dark spot matters most right now (salience × recency × usage) | THINK tick, post-walk |
| `emit(rank) → ask?` | whether/when/how a drive produces a question, within budget | output stage (§7.0 reverse-decode) |
| `budget()` → per-input cap + dedupe | saturation curve is **per-drive, not global** — **and since round 2 of §18 it is a queue rule plus an integer ceiling, NOT a function of any affective level (OD-22):** §18.1 says the walk is the only thing arousal may modulate, so the level-driven budget of OD-19 lost its input. The interrogation phase ends because **fewer named gaps surface**, not because an urge starved. | ask engine before emission |
| `salience(event)` → tag | how strongly an event stamps memory — a scalar the write path reads at commit; **round 3's bake scale: `B_max · tanh(k · a_formation)` (§20.4a)**, i.e. the live signal sum read **through the one integrator at the formation instant**, saturating, version-tagged (OPEN-33). **This is the door whose input OD-22 removed and OD-33 restored under a bound** — round 2's `f(signal_magnitude)`-only law holds for everything else, and it still holds that **arousal modulates the walk and never categorizes an event**. It is what gets stamped on the edge as the bake. | write path (§6.1, §6.5) |
| `update(signals, t)` → new `a` | **the integration door (added by §18, OD-20):** the loop hands over anonymous numbers (novelty spike, realized gain, contradiction count, settle failures, queue age, idle-cascade contradiction) and gets back the next **arousal scalar**. Calls `arousal.py` behind it; the caller never learns anything about feelings, because there is nothing behind the door to learn | the walk, **once per signal event inside the spread** — round 3 re-scopes this door's caller (§20.1); the write path at the surprise step (§6.1) |
| `conduction()` → per-edge bias | **the read-side door (added by §18, OD-20):** returns the normalized `β · σ(edge.bake, a_now) + γ · m(point, a_now)` term that turns `W` into `W_eff` — the **per-edge bake** plus the **per-point median** (§20.3/§20.4) — already capped per round (§18.5). The walk applies the numbers; it does not compute them, and **the median arrives as a return value, never as a field** (R-28) | the walk before each spread round (§16.1, §7.3), **lagged one round behind the event that moved `a`** (§20.1) |

Curiosity is the **default drive policy**; it implements the first four. **The
affect layer round 1 installed was supposed to implement the same four** — that
is what the swap test was for — and needed two doors for itself, because §18
requires that arousal be *computed* inside the loop and *read* by the walk:
`update` and `conduction`. So the contract this file declared "at minimum" is
now **six methods** — an enlargement the owner should nod rather than inherit
silently (**OD-20**); round 2 shrinks what runs behind two of them but does not
remove a door (that would be a silent edit to the contract; **OD-21** records
the demotion instead).
The
**walk, the write path, and the gates must not name "curiosity"** — they
reference `drive.*`, so a second drive is added by implementing the
interface, not by editing organs (§17.6 swap test). After round 2 they must not
name *any* emotion either: the naming privilege moved out of `drive.py`
entirely, to `audit.py` (§18.6).

## 17.2 Curiosity's behaviors (from the dark-spot rule)

A gap **never** produces a fake answer or silence — it produces a QUESTION.

1. **ASK-BACK (missing slot):** decode succeeds, walk dies at the
   intersection ⇒ SI emits the named missing slot as a question:
   *"what is the color of apple?"* Not "I don't know" — the gap is
   **addressable, so address it**. (This is §7.4's primary birth answer,
   made a drive behavior; see OD-10 now RESOLVED ask-back-primary.)
2. **UNKNOWN-WORD ASK (missing point):** a sensed token has no hyperpoint
   ⇒ *"what is X?"* Vocabulary acquired through conversation, like a
   toddler.
3. **AUTO-COMMIT (answer-to-ask):** the owner's reply to a **pending SI
   question** is ground truth **by context** — it commits like a `/correct`
   (surprise high, coverage none, the hole was just measured), provenance
   `kind: answer-to-ask`, **linked to the question it resolved**
   (`resolved_by` on the question row, `question_id` on the fact row).
   `/correct` remains the **override channel**; conversation is now **also**
   the teaching channel (§1.4). Auto-commit is gated on a pending open
   question matching the reply — an unsolicited "red." with no pending ask
   is just input, not a teach.

## 17.3 Supporting mechanics

- **OPEN QUESTION QUEUE (a shelf):** asks are rows too — shelf `questions`
  (§5.2), row `{type: question, drive: curiosity|…, kind: ask-back|
  unknown-word|cluster-nomination|move-nomination, target: {slot|token}, lane:
  owner, asked_at, asked_by_thought,
  resolved_by: row_id|step|null, status: open|resolved|expired}`.
  **Unresolved questions persist** across turns.
- **IDLE THOUGHT revisits the queue first (§19.1):** on a quiet THINK tick
  (§16), each
  open question is re-walked against current knowledge — *does new
  knowledge answer an old question now?* If covered, `status → resolved`
  with `resolved_by`; **consolidation = closing your own loops**. No
  fabrication: a question resolves only because a row now exists, never
  because the walk guessed. **Only when the agenda is also empty does §19's
  ignition run** — so the queue is both a drive shelf and the idle loop's
  gatekeeper, and a real question always interrupts daydreaming without a
  scheduler.
- **NOMINATION ASKS (the v2 arc's two kinds — no new door, no new budget).**
  [docs/22](22-crystallize.md) §22.4 emits `cluster-nomination` ("three people
  keep telling me the tomato is a fruit — is that so?") and
  [docs/25](25-skills.md) §25.5 emits `move-nomination` ("I keep solving things
  the same way — is this a move worth keeping? does it have a name?"). Both go
  out through the **same `emit` and `budget` doors** as any gap-ask, obey the
  same queue rule, the same dedupe, the same integer ceiling and the same
  salience floor — **a detector that could buy its own asks would be R-20's
  nagging machine with a statistics degree**. They differ in exactly two
  documented ways: the template's *name* field is `NONE` under I6 because the
  pattern is lived and the word is not yet taught, and the **lane**: a nomination
  lands in the owner-side lane only (OD-128), because answering one is a
  confirmation and confirmation is the one right
  [docs/26](26-world.md) §26.3(b) does not grant in v2. When the owner answers,
  the reply is the ordinary teaching event (§1.4) through the ordinary gates
  (§6) — the drive layer's job ends at the ask.
- **QUESTION BUDGET (drive interface, per-drive):** max asks per input, priority
  by gap salience, **dedupe against the open-question queue**
  (never re-ask an open slot), **don't ask about typos/filler** (below
  salience floor ⇒ logged as a gap, not emitted). **Round 2 of §18 reverts
  the mechanism: the budget is a queue rule plus an integer ceiling again
  (OD-22).** Round 1 had made the allowance a quantized monotone function of a
  CURIOSITY *level* (OD-19) — that level no longer exists, and §18.1 licenses
  arousal to modulate **the walk only**, so wiring it to the budget would be
  the smuggled coupling §18.7 bans. The interrogation phase ends because the
  **named gaps run out**: as rows fill, coverage rises (§6.2) and fewer dark
  spots are named at all. The curve stays swept, never invented (OPEN-15). The
  **"annoying phase" dial** still applies — to the cap and the salience floor.
- **COMPOSITION, not command — CANCELLED (§18.3, OD-21/OD-22).** Round 1 said
  CAUTION *shrinks* the allowance, CONFUSION forces a double-check before
  emission, SATISFACTION damps the slot it just closed. **None of that survives
  as machinery**, because there are no residents to compose: emission is
  silence-explained-by-a-queue-rule, and every former "the level was low"
  excuse is replaced by "no gap was named / the ceiling was reached / the slot
  was already open". What *does* survive is the spirit of the line: **silence
  must always be explainable by a citable state of the loop, never by a rule
  that fired for no recorded reason.**

## 17.4 Anti-cheat (curiosity is a reflex, not knowledge)

Asking requires **zero content knowledge**: the question template is pure
form — frame + named slot + unknown token, all assembled from the owner's
own decoded tokens. `"what is X?"` must fire **identically for gibberish
X** (`what is znark?`), asserted in G1/G5d/G8. The drive interface, the
curiosity policy, and the salience function are innate **machinery**; the
**gaps it prioritizes and the questions it queues are learned state**.
Nothing about a specific word is innate. (Table rows R15–R16, §3.1.)

## 17.5 Where curiosity plugs in (the wiring, to prove it's not welded)

```
walk (§16) ── produces dark spots (gaps) ──▶ drive.prioritize(gaps)
                                              │
                          drive.budget(dedupe) ▼
                              open question queue (shelf §5.2) ──▶ drive.emit
                                                                    │ reverse-decode
                                                                    ▼ ask-back / unknown-word question
owner replies ──▶ drive matches pending question ──▶ AUTO-COMMIT (§6.5, kind answer-to-ask)
                                                     │
                                       drive.salience(event) tags confidence init (§8.4)
idle THINK tick ──▶ queue revisit (§17.3) ──▶ [agenda empty?] ──▶ ignition (§19: seeded
                                                    one-hyperpoint lighting → same walk
                                                    → monologue trace, READ-ONLY, I9)
```

Every arrow is an interface call. The walk never calls "ask"; the write path
never calls "curiosity"; the gates test the loop generically (G8). Note what
**is not** on this diagram: no emotion label points at any box. The audit layer
reads the finished trace from the side (§18.2), which is the one structural
difference between round 1's picture and round 2's.

**§19 is not a second drive** — that reading would be wrong, and the reason is
worth keeping in writing: ignition adds a **lighting source** to the walk (the
4th perturbation entry, §16.1), not a resident behind the doors. It touches no
effector door, asks nothing, writes nothing, and therefore neither passes nor
fails the swap test; it sits *under* it. Idle cascades feed one thing back into
the loop: the contradiction signal of §18.1's table, which is a signal row like
any other (§19.5).

## 17.6 The swap test (the ONLY design test for this chunk) — first run, executed by §18

> Could a second drive (affect, later) be added by implementing
> {prioritize, emit, budget, salience} **without touching the walk, the
> write path, or the gates?**

- **Yes ⇒ the layer is correctly cut.**
- **No ⇒ curiosity is welded into the read path ⇒ wrong — refactor before
  build.** This is an acceptance condition on §12's module list and a
  review item in G8; a `grep -ri "curiosity" src/` must return hits ONLY
  in the drive/curiosity module (R-20).

**Result of the first real trial (EMOTIONS chunk, §18) — reported, not
hidden, and re-read after round 2:** round 1 added five more residents and
reached every *behavior* through the doors that already existed (write path,
budget, salience, labels all consumed them interface-only, no new organ).
**Two things did change then, and both are ledger items rather than quiet
edits:**

- the walk gained one anonymous state slot and two interface calls
  (§18.1), because the owner's internality law says affect is computed
  *inside* the loop rather than observed from beside it → **OD-14**;
- the contract itself grew from four methods to six — `update` and
  `conduction` (§17.1) → **OD-20**. §17.1 said "at minimum", so this is an
  enlargement, not a contradiction, but it is the owner's to nod.

**Round 2 (arousal demotion) re-runs the trial in the other direction, and the
honest result is that the doors did not shrink with the machinery.** The six
residents became one scalar plus a readout, yet `update` and `conduction` are
still needed — `update` now integrates `a`, `conduction` still returns the
stamp bias — so **the contract stays six-wide and the reduction is recorded as
OD-21, not as a door removal.** Two doors did lose an input they had been
promised: `budget` lost its level, `salience` lost `f(a)` → **OD-22**, §17.1.
That is the swap test doing its actual job: the affect layer was removable
**without touching the walk, the write path, or the gates** — which is stronger
evidence for the cut than round 1 ever was.

**Round 3 (plasticity) re-runs the trial a third time and the door count is
still six (§20).** This is the batch that could easily have demanded a seventh
door — the per-point median is a new read, and `a_at` is a new write-side field —
and instead the median arrives as a **return value of `conduction`** and `a_at`
arrives as a **stamp field the substrate already owns**. What changed is
*inside* two existing doors: `update` gained a caller frequency (per signal
event, §20.1) and `conduction` gained a second term. **No door added, no door
removed, no organ, and no new call site in the walk** — the walk still makes
exactly the two calls it made before, which is the semantic reading passing
again, this time in the additive direction. G8's swap-test leg is what would
catch a violation: round 3 must grow the walk's **call sites by zero** even
though it grows its arithmetic.
**One door did change state, and it is a reversal rather than a growth:**
`salience` gets its affective input back as the saturating bake scale
(§20.4a) — the input OD-22 took away returns through one bounded curve, so the
door set is unchanged and the ledger item is **OD-33**, not a new socket.

**The supporter batch ([docs/21](21-grimm-supporter.md)) is the trial's fourth
run, and it is the first one that is not about a drive at all.** Grimm is **not**
a second drive policy behind R15, so the correct result here is a **refusal**:
he gains no door, no `drive.*` call site, and no seat in `prioritize`. The
temptation is real and worth naming — a "soothing drive" or a `drive.calm()`
would make co-regulation a first-class mechanism and would look *more* alive in
a demo — and it is exactly the thing L3 forbids and I8 already forbade in a
different century (**no external setter**, §18.1). So the drive layer stays
**`source`-blind**: the curiosity policy prioritizes, budgets and emits gaps the
same way whether a hole was surfaced by the owner's question or by Grimm's,
because a drive that weights gaps by *who mentioned them* is a social prior
sitting in the control path, which is R-21's costume in its fourth shape.
**Doors still six. The swap test's answer for this batch is "no drive was added,
and that is the pass."**

| reading of the test | verdict |
|---------------------|---------|
| literal — "not a single line of the walk changes" | **FAILED**, recorded as **OD-14**. The two laws (internality §18.1 vs strict swap test) cannot both hold literally; the owner's newer law (internality) wins, and the honest statement is that the walk gained a *slot* and two *door-calls*, not a *dependency*. Round 2 made the slot **one scalar**, so the violation got smaller — it did not disappear. |
| semantic — "the walk names no drive, imports no emotion, and touches arousal only through `drive.update` / `drive.conduction`" | **PASSES**, and this is the version gates enforce (G8's swap-test leg, G9, §18.7's costume test). `grep -ri "curiosity\|surprise\|caution\|confusion\|satisfaction\|boredom" src/walk.py src/write.py src/gates/` must be empty. **Round 2 moved the naming privilege:** round 1 allowed the words in `affect.py` **and** `drive.py` — the tissue and its doors; that grant is **withdrawn**, because a module that names a feeling *and* sits in the control path is the smuggled coupling. Now `audit.py` alone may speak them (§18.6, R-21). `arousal.py` and `drive.py` are wordless. |

If the owner prefers the literal version, the alternative is **OD-14(b)**:
`a` moves out of the walk into a per-thought pass in the drive module, which
keeps the walk untouched but breaks I8 ("arousal IS loop state") and makes `a`
a telemetry channel. Current doc choice: keep internality, relax the
literal test, and make the grep the enforcement point.

## 17.7 The thesis in one behavior

The newborn's FIRST OUTPUT is a question. At the edge of its knowledge SI
**asks instead of bluffing** — and a weight blob cannot do this: it has no
measurable edge to notice itself standing at. (Owner's words; the honest
birth state is a question, not nonsense and not silence.)
