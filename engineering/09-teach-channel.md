# 09 — The `/teach` Channel: parse, decode, key, commit, the gain ledger, and the bulk path

Deliverable 10. The logical owner of this file is
[docs/01 §1.4](../docs/01-interaction-model.md) (three teaching channels, the four
laws the fragment states, and the honest limit it publishes),
[docs/03 §3.2](../docs/03-birth-state.md) (`P_TEACH` and the three new string
constants),
[docs/05 §5.4](../docs/05-substrate-tesseract.md) (the teach row is **not** a new
row type), [docs/08 §8.2/§8.4](../docs/08-gain-loss.md) (demonstrated vs asserted
gain, owner-tier init) and [docs/06 §6.4](../docs/06-write-path.md) (T2 rank). The
algorithm-side steps already live in
[04 §12](04-algorithms.md) (the write path's ordered steps 0–9) and
[02 §11](02-module-contracts.md) (the tick's thirteen steps); this file specs the
**channel** end to end, adds the bulk-curriculum path the design anticipates but
does not detail, and publishes the limits an owner with two hundred lessons to
import needs to read *before* importing them.

It adds no row type, no shelf, no edge kind, no signal, no string constant, and no
law. Where the channel is uncomfortable with a law, the law wins and the discomfort
is written down (§9).

---

## 1. The channel in one page

| channel | act | provenance tier | confidence init | arbitration rank | error-before | `gain_kind` |
|---------|-----|-----------------|-----------------|------------------|--------------|-------------|
| **`/correct`** | override the whole last output | **T3** | 0.95 | T3 | **measured** (`o_wrong`, or the named empty slot) | demonstrated |
| **answer-to-ask** | a plain owner reply while an ask is pending | T3-context | 0.95 | T3 | measured (the slot SI itself asked about) | demonstrated |
| **`/teach`** | `/teach <query-pattern> => <response>` | T3 **tier** | 0.95 (**owner-tier**) | **T2** | **none** | asserted |
| plain owner statement, no pending ask | `apple is red` | T2 | 0.90 | T2 | none | asserted |

**`/teach` is not an exception to anything.** It is owner-authored content entering
through the *same* sense channel, decoded by the *same* born-in frames, gated by
the *same* surprise/consistency/arbitration machinery, writing **at most one row**.
The only structural difference from a plain statement is that the owner supplied
the answer half deliberately, so the slot never had to be asked.

**And four things it is not** (docs/01 §1.4 (i)–(iv), restated as build facts):

| it is not | mechanical consequence |
|-----------|------------------------|
| not a batch loader | one row per act, so a 2,000-line file is 2,000 logical steps, not one import (§8) |
| not a `/correct` | `bind_output_id = null`; it never supersedes *by tier*; a conflict goes through §6.4 arbitration and the disagreement stays recorded |
| not a synonym/template injector | the left side must **decode with the frames SI already has**; a teach that needs a new frame is a stage/S2 matter, not a channel feature (§3.3) |
| not demonstrated knowledge | `gain_exact = 0` at birth and no gate may count asserted units toward a calibration claim (§6) |

---

## 2. Grammar and parse

```
/teach <left> => <right>
```

| aspect | rule |
|--------|------|
| markers | exactly two string constants carry the act: `/teach` and `=>` — both already in [docs/03 §3.2](../docs/03-birth-state.md)'s allowlist, with `owner-teach` as the third (the provenance enum literal). **The channel has no other vocabulary**, and `check_teach_literals` ([05 §20](05-gate-harness.md)) fails a build that grew a synonym table, a template list or a POS guess to make teaching easier — R-03/R-18's "dying by curriculum" |
| detection | step 0/1 of the write path ([04 §12](04-algorithms.md)): `source = owner` **and** the `P_TEACH` shape ⇒ `Teach(pattern, response)`. For a `source: grimm` line the shape is inert and the line is ordinary sense input (§21.4, G12(d)) — a supporter typing `/teach what color is apple => red` teaches nothing and changes nothing |
| ordering | the teach short-circuit runs **before** the pending-ask match ([02 §11](02-module-contracts.md) step 3 before step 4, [04 §1.2](04-algorithms.md) rule "step 3 precedes step 4"): an explicit act is never read as a reply. Consequence G8(f) tests: a `/teach` that happens to cover the open ask resolves the ask **by coverage**, keeps its own `question_id = null`, and is not an `answer-to-ask` commit |
| the split point | `/teach` is split on the **first** `=>` only, so a right side may contain the marker's characters; the left side is handed to the decoder verbatim minus the leading marker; whitespace around both halves is trimmed, nothing else is rewritten |
| paste | a `/teach` line inside a multi-line paste is still that line's own act ([01 §12](01-data-schemas.md), OD-54) |

### 2.1 Malformations, and what each one costs

Every row below still **consumes a logical step and writes a `context` row** — a
rejected teach is not free, and pretending otherwise would make the transcript
liability-free, which is the same escape route R-07 warns about.

| input | outcome | rows | state cost |
|-------|---------|------|------------|
| `/teach` with no `=>` | not a `P_TEACH` match ⇒ the line is ordinary sense input, decoded as whatever it looks like | 0 | one `context` row; whatever the tokens light |
| `/teach => red` (empty left) | decode gets nothing to match ⇒ undecoded | 0 | one `context` row, one `Undecoded` log line |
| `/teach what color is apple =>` (empty right) | response surface empty ⇒ the act has no content to bind; rejected before commit | 0 | `context` + a named dark spot for the *response* slot, since the left side *did* decode |
| left side does not decode | **named dark spot + zero rows** (docs/01 §1.4 (iii)) | 0 | `context` + the gap surfaces on the `questions` queue like any other gap |
| left side decodes to a **non-question** frame (`apple is red`, `znark means floop`) | rejected with the reason `left_not_a_question`; **zero rows** | 0 | `context` + one log line |
| both halves fine | one row | 1 | `112 + 16·E + 16·S + 8 + T + I` ([06 §2](06-resource-budget.md)) |

**The non-question row is decided here, and it is a reading of docs/01, not a new
rule:** §1.4 says "a **decoded question shape** on the left, the content to bind on
the right", so the accept set for the left side is the four question frames —
`P_QUEST_A`, `P_QUEST_D`, `P_QUEST_W`, `P_OF_REL`
([docs/03 §3.2](../docs/03-birth-state.md)). `P_ATTRIB` and `P_DEF` are assertion
shapes: they have no *slot* to key, so binding a response to one would key the row
by something the retrieval path cannot ask. `check_teach_left_side`
(§10) is the leg. The reason string is printed to the owner because "my curriculum
wrote two thousand gaps and no knowledge" is the single most likely way to lose an
afternoon, and a silent zero would be indistinguishable from a channel that works.

---

## 3. The decode contract (the left side is decoded, not stored)

| field | derived from | notes |
|-------|--------------|-------|
| `stimulus.pattern` | the frame id the left side matched | the *same* decoder that answers questions; a teach does not have its own parser (R-14's barrier: two parsers would be two ontologies) |
| `stimulus.signature` | the left side, **normalized** exactly as an incoming question is: lowercase, punctuation stripped, plural-folded (R1–R2) | this is the row's *key*, and it is why `/teach What COLOR is Apples? => red` and a later `what color is apple?` meet at the same slot |
| `stimulus.terms` | the token ids the left side lights, after the alias rule of [04 §2](04-algorithms.md) (OD-55) | a token with no hyperpoint **fires no lighting and is recorded as a dark spot** — the same behavior as any input, and the reason a teach can *introduce* a term without teaching a fact about it |
| `stimulus.surface` | the left side verbatim (owner's casing) | quoted, never keyed |
| `response.terms` | the right side's tokens, interned and lit at commit | **not** frame-decoded: the right side is content, not a shape. A novel response token creates a term record and a point, exactly as a novel input token does |
| `response.surface` | the right side verbatim | v1 binds **whole outputs** ([docs/05 §5.4](../docs/05-substrate-tesseract.md): `bind_output_id` "whole output, v1"), so the surface is what recall echoes (§9.3) |

**Why the decode requirement is the anti-cheat law's seatbelt** (docs/01 §1.4 (iii)):
if `/teach` keyed rows by typed text, the channel would be a *second ontology* — an
owner could install phrasings SI cannot ask about, and the transcript would show
knowledge that the walk can never reach. Keying by the decoded pattern means a teach
can only ever fill a slot the question machinery can name. Everything the channel
cannot name stays unnameable at S0–S1, which is
[docs/14](../docs/14-risk-register.md) R-11's honest limit and is not patched here.

**What a teach may *not* do to the decoder** (the failure R-18 names): create a
frame, extend the pattern table, add a function word, or seed a `frame-synonym`
edge. `P_TEACH` binds a response to a pattern that already decodes; template edges
still come only from a `/correct` or a `P_DEF` event with step provenance. A build
in which the teach path grows a decoder edge is a build in which the decode barrier
G2 tests has been routed around by the friendliest-looking code in the repository.

---

## 4. Key and row shape (exactly one row, exactly these fields)

| field | value at a `/teach` commit | why |
|-------|----------------------------|-----|
| shelf (`x`) | `s_r_pairs` at S0; from S1 the re-parse path may *also* derive a `facts` row later — the teach itself still writes **one** row | [04 §12 step 3](04-algorithms.md); a second row per act fails G1.t |
| `y` | next handle in shelf order | deterministic allocation (I5) |
| `stimulus` | §3's table | keyed by the decoded pattern |
| `response` | §3's table | verbatim surface |
| `bind_output_id` | **`null` (0)** | no ask-back output precedes the act, so there is nothing to bind |
| `question_id` | **`null` (0)** | the act answered nothing it was asked; if it happens to cover an open ask, the *ask* records `resolved_by`, not the row (G8(f).3) |
| `prov_kind` | `owner-teach` | the enum literal already in the allowlist |
| `prov_speaker` | `0 owner` — **and only ever 0** | §21.4, G12(e); a `links` row with `prov_speaker = 1 grimm` is the fail ([05 G12.e](05-gate-harness.md)) |
| `text_ref` | the right side's interned bytes | provenance points at what the owner wrote, not at what the row claims |
| `confidence` | **95** hundredths (`CONF_INIT_OWNER_TEACH`) | owner tier |
| arbitration rank | **T2** | an asserted row never outranks a demonstrated correction — **OD-41** records the split so nobody "fixes" it by promoting the tier |
| `gain_exact` | **0** | no error-before existed (§6) |
| `gain_kind` | `asserted` | promoted at the first non-corrected serve (§6.2) |
| primary edge `z[0]` | stimulus→response, kind `0 owned-attribute` / `1 owned-definition` by frame, `weight` = init **plus the bake** | [docs/08 §8.4](../docs/08-gain-loss.md), [01 §5.3](01-data-schemas.md) |
| `bake` | `B_max · tanh(K_TANH · a_formation)`, computed **once** at this instant | §20.4a; OD-33's one write-side read of `a` — and §7 below is where this stops being academic |
| stamps | ≥ 1, `src = 0 write`, magnitudes = the firing signals' own values, `a_at` = the quantized `a` at this instant | [01 §5.1–§5.3](01-data-schemas.md); the novelty-spike at write time is a real signal (§18.1) |
| `repeats` | 1 | a later exact repeat bumps it (§5.2) |

---

## 5. Where it sits in the tick, and the arbitration it triggers

Reusing [02 §11](02-module-contracts.md) and [04 §12](04-algorithms.md), the teach
path is steps **0 → 1 → 5 → 6 → 7 → 11 → 13**; the read half (steps 8–10) still
runs, because a teach is also an input line and SI may react to being told something.

| step | what happens for a teach |
|------|--------------------------|
| 0–1 | `context` row written (`source: owner`), tokens interned and lit, `P_TEACH` shape matched **before** the pending-ask test |
| 3 (pending-ask match) | **skipped by design** — an explicit act is not a reply |
| 5–7 | the walk runs on the lit set: this is what makes a teach *also* a thought. A teach that lights a dark slot produces a gap, and a gap feeds the drive layer like any other |
| 11 (`learn.commit`) | surprise/coverage first (§5.1), then consistency, then arbitration |
| 13 | event log flush, hash compose, clock advance |

### 5.1 Coverage decides whether a row is born

| prior state at the slot | outcome | rows | bytes |
|--------------------------|---------|------|-------|
| empty (coverage 0) | commit fires, high surprise (novelty − coverage) | 1 | [06 §2](06-resource-budget.md)'s commit line |
| an active row with the **same** normalized key and the same response surface | `BORING_RULE`: `repeats += 1`, `confidence +5` (cap 99), `last_confirmed_at` bumped, **no new row** | 0 | 14 B in place (FLUID columns) |
| an active row with the same key and a **different** response | **not** a `/correct`: T2-vs-T? arbitration (§6.4's tier order) runs like any owner statement, winner commits fresh, loser aged to `weight = 0` keeping bake and stamps | 1 | the supersession line |
| an active **demonstrated** T3 row at the slot | the teach loses on rank (OD-41's split in action) and the disagreement is **recorded, never silently overwritten**; the loser's bytes are kept (I4) | 1 or 0 by the boring rule | — |

**The repetition caveat, published rather than hidden:** a curriculum that lists the
same lesson twice raises that row's confidence without a single recall having served
it. That is §8.4's mechanical law, not a bug, and it is exactly the
[docs/14](../docs/14-risk-register.md) **R-07** confidence-theater shape — which is
why the calibration ledger, not the confidence number, is what gates read (§6.3).

### 5.2 A teach can close an ask it was not aimed at

Resolution is **by row existence at the slot** ([04 §12 step 9](04-algorithms.md)):
the moment a committed row covers an open ask's slot, the ask goes `resolved`,
`resolved_by` names the teaching row, and one event is logged. The teach row's own
`question_id` stays `null`, because the citation must never imply SI asked.

---

## 6. The demonstrated-vs-asserted ledger

| column | demonstrated | asserted |
|--------|--------------|----------|
| born this way | `/correct`, `answer-to-ask` | `/teach`, plain T2 statement, T1 inference, re-parse derivation |
| error-before | measured (`o_wrong` or the named empty slot) | **none** — nothing wrong was guessed first |
| `gain_exact` at birth | the measured difference, ±1 units | **0** |
| what the number means | writing this row demonstrably replaced something that failed | a **claim by the owner** |
| promotion | n/a | the first recall that serves the row **and is not corrected** ⇒ `gain_kind` flips to `demonstrated`, one event-log line naming the serving step id ([docs/08 §8.3](../docs/08-gain-loss.md)); it is the only promotion the ledger has, and it is a **new version of the row's ledger**, not a rewrite of history |
| may a gate count it toward a calibration claim? | yes | **no** — "no gate may count asserted units toward a calibration claim" ([docs/08 §8.2](../docs/08-gain-loss.md)) |

**Why the ledger is the load-bearing part of this file:** it is what lets the design
say "SI learned that in conversation" and "the owner asserted that in bulk" with two
different words, and it is what keeps an imported lattice from reading as a earned
one. A transcript cannot launder asserted rows into demonstrated ones, because
promotion requires a *serving event* that is itself in the log
([docs/08 §8.2](../docs/08-gain-loss.md): "a transcript can never read an imported
lattice as a demonstrated one"). Nothing about the bulk path changes this: it is the
same ledger, filled the same way, and a two-thousand-row curriculum is a
two-thousand-row *asserted* lattice until recalls start landing.

---

## 7. Arousal, bake, and the one consequence a bulk author must be told

The write path reads `a` at exactly one instant through exactly one saturating
curve ([docs/08 §8.4](../docs/08-gain-loss.md), §20.4a, **OD-33**). A `/teach` is a
teaching event, so it bakes. Now compose the two facts:

| fact | arithmetic | consequence for an import |
|------|-----------|---------------------------|
| each novel line fires a **novelty-spike** signal at write time ([04 §6](04-algorithms.md)) | `a' = clamp(a·λ_a + Σ gains, 0, 1)` per signal **event** | a curriculum of novel rows is a run of repeated spikes with only one tick of decay between them |
| `a` decays per logical step, and one line = one step ([01 §12](01-data-schemas.md)) | `λ_a` per tick | `a` will sit **high for the whole import** — an import is a hot episode, not a calm one |
| the bake is permanent and bounded | `B_max · tanh(K_TANH · a_formation)` | **every row born during a bulk import shares roughly the same, high, permanent head start** |
| unbaked-at-rest is the *other* case | `a_formation = 0 ⇒ bake = 0` (**OD-56**) | a calm afternoon of one-off teaches produces near-zero bakes on its first rows and rising ones after |

So a wholesale import is not merely "the same thing, faster": it produces a
lattice whose edges were all born at a similar, high arousal level, and the
`σ(bake, a_at, a_now)` term will treat that whole cohort as one birth class
([04 §8](04-algorithms.md)). That is lawful — it is the design's own "one
genuinely scary moment makes that specific edge hot forever" applied to a moment
that lasted an afternoon — and it is **the single most surprising property of the
channel**, so it is published here instead of being discovered in a G11 result.

**Two mitigations, one refused:**

| idea | verdict |
|------|---------|
| suppress or lower the bake during a curriculum (a `--bulk` flag on the write path) | **refused.** A write path that consults *which kind of event* this is, to size an affective quantity, is R-21(c) — the event-kind table wearing a saturating function. The curve takes one number and no labels |
| cap the novelty gain during bulk | **refused** for the same reason: it is a hand on the gain table from outside the signal list |
| **interleave quiet ticks** between curriculum lines, so `a` decays between commits | **legal, and the owner's call: OD-70.** It is pure logical time, it uses machinery that already exists ([docs/19 §19.1](../docs/19-idle-ignition.md)'s precedence: queue revisit → re-parse → ignition), and it is visible in the transcript. It is *not* the default, because it changes the tick structure and therefore the state hash, and because quiet ticks with an empty agenda **ignite** — an import with settle gaps would also produce a monologue about the rows it just ate, which is a behavior change wearing a cosmetic one |

---

## 8. The bulk-curriculum path (an inbox writer, not a loader)

**The statement that keeps this honest:** the bulk path performs **no** parse,
**no** decode, **no** key derivation, **no** arbitration and **no** storage write.
It reads a file and appends one `sense_inbox.siro` frame per line
([00 §7](00-stack-workspace.md)'s B-writes-keystrokes rule, extended to a feeder),
each frame `source: owner`. Everything that happens afterwards is the ordinary
loop, doing what it does to a typed line.

| property | spec |
|----------|------|
| surface | `si-cli feed --session <id> --file <path>` — a subcommand of the **same binary the panel lives in**, not of the loop's step order |
| what it may open | the curriculum file and `sense_inbox.siro`. **Nothing else.** `check_curriculum_is_inbox_only` (§10) is a static assert on its reachable symbols: no `si-learn`, no `si-store::lattice`, no `si-walk` |
| why a "fast importer" is refused | a loader that writes rows directly would be (a) a second write path with no surprise/arbitration gate, (b) a batch row type, (c) a language pack — [docs/14](../docs/14-risk-register.md) R-11's sentence: the day content is imported wholesale *is* CONFIG ENGINE by definition, and the gates would have to be re-run in CONFIG SI to say so honestly. **The channel is slow on purpose** |
| ordering & determinism | lines are appended in file order; `t` advances one per line; arrival **rate** is wall-clock and is therefore *not* part of state (I5). The same file replayed into a fresh session produces a byte-identical state hash — which is the whole reason the feeder is allowed to exist |
| no throttle | the design budgets SI's *asks*, not the owner's input (**OD-22**: a queue rule + ceiling, not a level). A rate limit would make arrival timing a state input |
| provenance of the file | the file's name and sha256 go in `runs/<session>/feed_provenance.txt` — a **log line, outside state and outside the hash**, so replay needs only the lines themselves and no machine has to find the same file. R-33's rule cuts the other way here too: the log is output, never an input |
| per-line shape | a blank line still advances a step? **No** — a blank line produces no frame (nothing to intern, nothing to light, no `context` row worth citing); a line beginning `/teach` that fails §2.1's grammar produces a frame like any typed line would |
| comment/`#` lines | **not supported in v1**: a comment convention is a second grammar in the channel, and the feeder does not parse. A `#` line is a sense line whose tokens will light nothing and will cost a step — which is the honest cost of wanting comments |
| artifacts | the run directory is ordinary: `transcript.txt` (all lines, `source`-tagged), `mindlog.json`, `state_hash.txt`. A curriculum run is a **session**, and it can be gated, replayed and inspected in the five panels like any other ([08 §8](08-gui-observability.md)) |

---

## 9. Published limits (deliverable 9's "and its published limit")

Printed here because the honest version of a limit is worth less than nothing when
it is discovered at line 1,400.

1. **A teach earns no demonstrated gain.** A lattice imported in one afternoon is
   `asserted` everywhere, and its confidence is only as good as the recalls that
   follow (§6). Gates may not count asserted units toward calibration claims.
2. **A curriculum alone cannot leave S0.** [docs/11 §11.1](../docs/11-stages.md)'s
   exit is "≥ 5 S-R rows **with ≥ 1 supersession**": the supersession criterion needs
   an *edit*, not an addition. So the file that teaches 2,000 clean,
   non-contradictory facts leaves SI at S0 with a big lattice — and the way out is
   to contradict it once, which is a fact about stage-0 development worth stating
   rather than engineering around.
3. **Whole-output binding.** v1 recalls echo the entire `response.surface`
   ([docs/05 §5.4](../docs/05-substrate-tesseract.md)). A curriculum row written
   `/teach what color is apple => it is red, I think` will answer
   *"it is red, I think"*. Short responses are the author's job; the channel does not
   trim, summarize or interpret, because any of those would be a renderer with
   semantics.
4. **The frame ceiling (R-11, unabridged).** Anything the four question frames
   cannot name is unteachable at S0–S1 — a `/teach` with an undecodable left side
   writes zero rows and one gap. Lines-per-hour is not the limit; **shapes** are.
5. **Repetition inflation.** A duplicate lesson bumps confidence without a
   recall (§5.1's caveat, R-07). The ledger, not the number, is what a gate reads.
6. **The birth-cohort bake (§7).** Everything in one hot import shares a high,
   permanent head start. This is a property of §20.4a, not a defect of the bulk path.
7. **`/teach` never teaches a shape, a word's POS, a synonym, or a frame** (§3.3).
   Template edges still arrive only from `/correct` and `P_DEF` with step
   provenance, and R-18's audit treats an unprovenanced frame edge as a factory
   default.
8. **Grimm may not use the channel** (§21.4). His `/teach`-shaped lines are inert
   input; `source: owner` is carried from the receiving module, not inferred from
   text (L-A).
9. **One row per act, forever.** No plural form of the command, no heredoc, no
   "import this JSON". If a curriculum needs to be large, the file is long — the
   *act* stays atomic.
10. **Cost arithmetic, derived not measured:** `n` lines = `n` context rows
    `= n × (112 + 4 + ‖surface‖₄)` bytes minimum
    ([06 §2](06-resource-budget.md)/§3), plus one row per uncoversed slot. There is
    no batch compression, and that is deliberate: a compressed import would be a
    write path.

---

## 10. Gate legs and static checks for the channel

Existing, unchanged, and cited so the build does not re-litigate them:
`G1.t.1–t.4` ([05 §G1](05-gate-harness.md): one row / zero rows on a repeat /
zero rows + named dark spot on an undecodable left / gibberish parity with
substituted tokens), `G8.f.1–f.3` (the short-circuit-before-pending law and
closure-by-coverage), `G5.c.1` (the three literals, no more no fewer),
`G12.d.2` (a Grimm teach-shaped line is inert), `check_teach_literals`.

New, for the parts this file specs:

| id | assertion | fails when |
|----|-----------|-----------|
| `G1.u.1` | the **left-side accept set**: for each of the four question frames, one teach commits 1 row; for `P_ATTRIB` and `P_DEF` left sides, 0 rows and the reason string `left_not_a_question` appears in the log | the channel grew a second keying rule, or a statement-shaped left side silently produced a row |
| `G1.u.2` | **bulk equivalence**: the same `n`-line file (a) fed through the feeder into a fresh session and (b) typed line by line into a second fresh session ⇒ **byte-identical `state_hash.txt`** and identical `transcript.txt` | the feeder is a loader — any difference means it bypassed or added a step |
| `G1.u.3` | **no batch row type**: after a 200-line curriculum, `s_r_pairs` row count equals the number of *uncoversed* lines, `repeats` sums account for the rest, and no record has an address outside `(t, x, y)` | an "import" structure appeared in the substrate (R-12's alarm, patched silently) |
| `G1.u.4` | **asserted-only ledger**: every curriculum row reads `gain_kind = asserted`, and after `k` served recalls exactly `k` promotion events exist, one per row, each naming a serving step id | asserted units being counted as demonstrated — R-07 through the newest door |
| `G1.u.5` | **the birth cohort is visible, not tuned**: `bake` on curriculum rows is derived from the recorded `a_at` through the one curve, recomputable in review from the trajectory; `a` stayed high across the import | a bake that was suppressed, capped ad hoc, or keyed by event kind (**OD-33**'s scope, R-21(c)) |
| `check_curriculum_is_inbox_only` | static: the feeder's reachable symbol set contains no `si-learn`, no lattice write, no `si-walk`; it opens exactly two files | the importer became a writer |
| `check_teach_left_side` | static: the teach path calls the **same** decode entry point the sense path calls, and no second decoder exists | two ontologies, one of them hand-fed |
| `check_no_teach_vocabulary` | static: the pattern table still holds exactly ten `P_*` entries; the channel added no string constant beyond the three allowlisted ones | R-18's "dying by curriculum" |

---

## 11. What this file decided, and what it hands to the owner

**Decided here (HOW, inside the law):** the malformation table and what each
failure costs; the **left-side accept set** (the four question frames, per docs/01
§1.4's "decoded question shape") with an explicit reason string; the decode
contract including the *response* side's scope; the row-shape table for a teach
commit; the coverage/boring/repeat and supersession outcomes, including the
repetition-inflation caveat; the ledger's promotion mechanics as the only
calibration-relevant currency; the birth-cohort bake consequence, stated as the
surprise it is; the bulk path as a **pure inbox writer** with a static check that
keeps it one; and the ten published limits.

**Handed to the owner:**

| id | question | lean here |
|----|----------|-----------|
| **OD-69** | is the **file feeder** in v1 at all, or does the owner type teaches? | **yes, as an inbox writer only**, with `G1.u.2`'s byte-identity leg as its license: a feeder that cannot produce a different state than typing is a convenience, and one that can is a loader. Refusing it entirely is also coherent — it is the same channel with a longer reach, and reach is what R-11 worries about |
| **OD-70** | may the feeder interleave **quiet ticks** between lines so `a` decays and the birth cohort is not one hot class? | **off by default.** It is legal in every respect (pure logical time, existing idle precedence) but it changes the tick structure, hence the state hash, and quiet ticks with an empty agenda **ignite** — so an import with settle gaps also grows a monologue about its own rows. If the owner wants it, it is a flag whose value belongs in the scenario hash, not a build default |
| OD-41 | owner-tier init (0.95) with T2 rank | implemented exactly as split; this file is the place the split is most likely to be "fixed", so it is restated in §4 |
| OD-52 | LLM-assisted genome growth | unrelated to this channel, and the boundary is worth naming: **`/teach` is content the owner vouches for; expansion content is content the dictionary vouches for, and it may only enter as a frozen batch** ([07 §16](07-genome-build-plan.md)). A build that reaches the second through the first has crossed I2 |
