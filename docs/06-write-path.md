# §6 — The Write Path (organ: LEARN)

## 6.1 Pipeline

```
input line (class: owner | grimm | trusted | stranger — §21.4, §1.6; the last
           two are DERIVED from grant rows at generation and stored nowhere)
           → is it a reply to a PENDING question? (drive match, §17.2.3)
              **and is its class `owner`, or `trusted` under an `answer` grant?**
              yes + owner (or granted) → AUTO-COMMIT: candidate row, provenance.kind=answer-to-ask,
                    linked question_id; treat as ground-truth-by-context
                    (surprise high, coverage none) → same gates as /correct
              yes + any other class → **NO MATCH.** The ask stays open for the owner; the line
                    is sense only — it may light the walk and raise signals
                    (§21.4, R-32). This is the one branch the supporter batch adds
                    to the pipeline, and it exists to close provenance laundering;
                    [docs/26](26-world.md) G19(d) is the same branch with ten more
                    voices on the refusal side of it.
              no  → pattern match (R3) → candidate row (kind depends on stage:
                    S0 → s_r_pairs; S1+ → facts via re-parse or direct teach)
                    ⤑ **P_TEACH short-circuits the pending-ask test above** (TEACH
                    fragment, §1.4): a line whose shape is
                    `/teach <pattern> => <text>` is an explicit owner act, so it
                    is never read as an auto-commit reply. Its LEFT side decodes
                    through R3/R14 exactly like the question it stands for and
                    becomes the row's stimulus; its RIGHT side is the response
                    surface; `provenance.kind = owner-teach` and
                    `bind_output_id = null`, because there is no output to bind —
                    nothing wrong was guessed first. An **undecodable left side ⇒
                    named dark spot, ask, zero rows**: the row is keyed by the
                    decoded pattern, never by the raw text the owner typed. When
                    the decoded pattern matches an open ask's target slot, that
                    ask resolves **by row existence** after the commit (§17.3's
                    existing rule, not a new one).
                    — **a `source: grimm` line is never a teaching candidate.**
                    Layer 2 admits only restatements of rows that already exist, so
                    the single write-path branch his shape could reach is the
                    boring-input **confirmation** bump — and §6.2 gates that on
                    `source: owner` too (R-35). Everything else about him stops
                    upstream, at `speakgate.py` (§21.3)
           → SURPRISE (R5) → boring? drop + CONFIRM bump : continue
              ⤑ the surprise number IS a signal: `drive.update(signals, t)` runs
                here — one arousal scalar integrates it (§18.1: no privileged
                sensor, no separate pass; the write path hands over numbers and
                never names an emotion — emotion words live only in `audit.py`,
                §18.6). **Round 3: this is one call among many, not the call** —
                arousal also integrates inside the walk, per signal event
                (§20.1); the surprise at the write is simply the event that
                happens *here*. **The same value is the stamp magnitude on the edge about
                to be written** (§18.5: arousal modulates the walk, not the
                write-side salience)
           → CONSISTENCY (R6) → clean? COMMIT : ARBITRATION (R7)
              ⤑ **Round 2 removed the affective reading of this step.** Round 1
                said a high CAUTION level "raises the cost of `assert` relative
                to `emit`" and that this gave Act 5's ASK branch its *reason*.
                Cancelled (**OD-24**): arbitration is §6.4's tier/branch table,
                full stop — arousal may have changed *what the walk surfaced*
                (spread, §16.1) but it has no seat at the write gate. The
                audit layer may print *caution* over this pattern; the branch
                name is the reason (§18.2's control chain has three links, not
                four).
           → append-only commit + event log + confidence init (§8.4),
             MODULATED by `drive.salience(event)` = **`B_max · tanh(k ·
             a_formation)`** (§17.1, §8.4, §18.5, §20.4a — **OD-33**: this is the
             one place the write side reads `a`, at this instant, through this
             curve)
           → WRITE-SIDE STAMPING + BAKE (§18.5, §20.4): the firing **signal** is
             stamped onto the new / re-weighted edges as charge `[{signal,
             magnitude, a_at, thought_id, signal_ref}]` (`a_at` pending **OD-28**)
             — assembled inside `arousal.py`, handed
             out through the drive interface, never formatted by the write path
             itself. L3 only, never a frozen edge (I8/G9c);
             low-signal edges form weakly by the same mechanism.
             **This instant is the only moment a bake can be bought:** the head
             start and confidence bump this edge gets here are its birth
             properties and are durable — nothing later may re-price them, and
             what the walk feels at some future thought changes only the
             *reading*, never this row's formation (§20.4). **A teaching event
             forms edges; only a teaching event bakes them** — which is why an
             idle cascade, which forms none, can create no new bake (I9, G10(b)),
             and — the supporter batch's one addition to this sentence — why a
             `source: grimm` line forms no edge and buys no bake either: it is not
             a teaching event, whatever it sounds like (I10, §21.4, G12).
             **The bake saturates (§20.4a): `B_max · tanh(k · a_formation)`**, so
             the first kitchen fire and the tenth both score near the cap and
             **nothing ever scores an unbeatable edge that owns the walk** —
             durability is bounded, that is what makes it lawful. No event kind
             is consulted: `a_formation` is the live sum the loop already
             integrated, so **nothing categorizes fires as bad — it felt a lot
             when it happened**, and the anti-cheat law is untouched.
           → on commit of an answer-to-ask: mark the question row resolved
             (status→resolved, resolved_by=row_id) — close the loop (§17.3)
             (the **realized-gain signal** fires here and feeds `a`; the audit
             labels the pattern *satisfaction*. Round 1's "and damps that slot's
             ask urge" is gone — the phase ends because the queue is empty, not
             because an urge was damped: **OD-22**)
           → gain measurement (§8.2 exact for corrections/answers-to-ask,
             probe otherwise)
```

**The write path has one caller: a sense event (I3).** An idle cascade (§19)
reaches the same code only to *reset a rehearsal curve* or *push an agenda
item*; it never arrives here with a candidate row, and `idle.py` holds no
import of the commit function — that absence is G10's read-only leg, and it is
what "an idle walk can notice, never learn" means mechanically (I9, §19.4).
**The supporter batch adds a second case with the same shape and a different
reason to exist:** a `source: grimm` line is a sense event, so it *could* reach
this code — the only thing that stops it is the `source` test at the top of the
pipeline and the fact that layer 2 leaves it nothing novel to carry (§21.4). Idle
thought is kept out of the shelves because it forms no teaching event; the
supporter is kept out because **he is not the owner**. Two different laws, one
observable consequence: no row and no `speaker` value but `owner` (§5.4).

Three teaching channels feed this pipeline: explicit `/correct` (override,
T3), `answer-to-ask` auto-commit (context, also ground-truth tier) —
§1.4 — and `/teach <pattern> => <response>` (owner-authored content, T2 rank
with owner-tier confidence init; the tier/rank split is **OD-41**).
A plain statement with no pending question does NOT auto-commit.
**All three channels are owner-privileged.** `/correct` in particular is a tier, not
a message shape: letting the supporter issue it would put a pretrained model at
the top of §6.4's table, which is the highest-consequence single change this
batch could have made and did not (§21.4, I10). `/teach` is barred to him for
the same reason and by the same mechanism — the shape test lives upstream of
`sense.py`, so his `/teach` never becomes a teaching candidate at all (§1.5,
§21.4, G12(d)).

## 6.2 Surprise = novelty − coverage (over stored rows only)

- `novelty` = 0.0 if an active row with identical (stimulus-signature ⇒
  response) / (key ⇒ value) exists; 0.5 if reachable via learned bridge with
  same response; 1.0 otherwise.
- `coverage` = 1.0 if an active row exists at the same slot (matching or
  conflicting); matching-only coverage is what cancels novelty.
- **Boring input** (repeat of known statement) ⇒ surprise 0 ⇒ **write
  nothing**, bump `repeats` + `last_confirmed_at` + conf (+0.05). Repetition
  is a confirmation signal, not noise. **Gated on `source: owner` (§21.4, R-35):**
  confirmation means *the owner agrees*, and a mirror that repeats a row all
  afternoon would otherwise inflate confidence in a belief nobody re-endorsed —
  R-07's confidence theater arriving through the friendliest door in the design.
  A grimm echo is **rehearsal**: it may light the walk and touch §8.4's decay
  curve the way any mention does, and it buys nothing durable.
- **Conflict** ⇒ never dropped as boring; always routed to consistency.

## 6.3 Consistency checker rules (R6)

1. Same slot, different active value ⇒ CONFLICT flag.
2. A new taught link (`P_DEF`) targets a term already bridged ⇒ flag; newer
   owner teaching wins; old bridge aged.
3. System-inferred row vs owner-origin row at same slot ⇒ flag; owner side
   wins by tier — but the flag is still **recorded** (critical-law mirror:
   disagreements are reported, never hidden).
4. OWNED vs BASE conflict (CONFIG ENGINE only) ⇒ OWNED wins at NOW, output
   must cite both sides ([docs/09](09-claim-labels.md)). L1/L2 files are
   never written (Gate G6).
5. That is the complete rule set. No "plausible values" — a contradiction
   exists only relative to what was previously taught (or BASE, in ENGINE).

## 6.4 Arbitration policy (R7) — provenance tiers first, then insistence

Tiers: **T3 owner `/correct`** (outranks everything) > **T2 owner
statement** (with or without correction marker) > **T1 system-inferred**.

| conflict situation | outcome |
|--------------------|---------|
| T3 vs anything | supersede (age loser, record event, cite both) — the correction protocol is the teaching act; it does not re-ask |
| T2-marker vs T2/T1 | supersede (branch `marker-correction`) |
| T2 bare, first occurrence | **ASK** — hold candidate as pending event, commit nothing (§4 Act 5) |
| T2 bare, repeat ≥ 2 after the ask surfaced | supersede (branch `insistence`) |
| T1 vs T1 | recency within tier, tie → higher conf, tie → lower step id (deterministic) |
| system-vs-owner anywhere | owner wins, always; T1 loser aged with reason string |

Not last-write-wins, because: (a) a bare same-tier contrary input **cannot
commit** — only a question; (b) losers are aged, never deleted, and the
winning **reason** is a typed event; (c) identical sentences produce
different outcomes depending on marker/repeat/confirmation state. Gate G4
counts ASK events and fails a run with zero.

**Tier-strict arbitration, once [docs/26](26-world.md) tier (b) exists.** The
ladder above is *provenance* tiers, and it gains one rule and no new rung at
birth: the speaking **class** bounds what a line may arbitrate against before
its marker does. An owner line supersedes everything, as it does today. A
**granted** correction (`/trust X correct`) supersedes only rows whose
provenance is itself grant-traced — never an owner row, whatever the insistence
counter says. A revoked user's rows keep their bytes and their citations (I4)
but **lose arbitration standing at NOW** (OD-125, G19(c)): a grant that cannot
be withdrawn would be a second owner, and a revocation that erased history
would break I4, so the withdrawal moves the ladder, not the lattice. Under the
birth state — tier (a), zero grant rows — every non-owner line is sense-only and
this paragraph changes no outcome, which is why G4's existing legs re-run
byte-identical and G19(c) is the leg that tests the ladder.

**Nomination confirms are owner-only, and that is a write-path fact, not a
courtesy.** A `cluster-nomination` ([docs/22](22-crystallize.md) §22.5) or
`move-nomination` ([docs/25](25-skills.md) §25.5) ask is answered the same way
any ask is: the reply is the teaching event (§1.4). The difference is the
*lack* of a grantable right — confirmation of a nomination stays owner-only in
v2 (§26.3(b)), so the write gate's answer-to-ask branch must check the class
before it accepts a nomination reply, and a granted `answer` right does not
reach it.

## 6.5 Commit semantics

Append-only. Commit = insert row + set `superseded_at`/`superseded_by` on
any aged rows + one event-log entry `{type, policy_branch, asked_at?,
resolved_at?, old, new}`. Two logs exist: the **owner transcript** (sense)
and the **mind log** (every gate decision, label, arbitration) — both
exportable as gate artifacts (`gates/`).

## 6.6 RE-PARSE (S1+ consolidation, runs on idle THINK ticks)

Trigger: stage ≥ S1 (entry criteria [docs/11](11-stages.md)) AND the S-R
row's stimulus/response parse under the current reflex set (R3 patterns +
R14 frames, matured via learned chunk forms and taught template edges).
Conceptually: **RE-PARSE is the DECODER maturing retroactively** — old S-R
pairs stored when the decoder was weak get re-decoded with today's frames
and edges. For each eligible S-R row in step order (deterministic, I5):
derive a `facts` row (key = decoded entity/field from stimulus+response;
value from response; `derived_from: [sr_id]`), then run it through the
ordinary write path (it can conflict with an existing fact ⇒ normal
arbitration). The S-R row remains forever as ground truth; the fact row is
a typed view. Cost: one parse pass per S-R row, once. Gate G7.
