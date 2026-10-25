# §8 — Gain/Loss: the Currency (organ: CALIBRATE)

Currency, defined before anything else:
`gain = (error without the retrieved row) − (error with it), same context.`
With the `/correct` protocol the error-before is SI's actual wrong guess,
so gain is **exact**.

## 8.1 The general probe — counterfactual walk (no-correction situations)

Gain is now cheap and honest: **re-run the same walk minus one edge, diff
the settle** ([docs/16](16-think-walk-hardware.md) §16.4).

```
context      = the exact token-pulse sequence of the input
walk_with    = settle-set S          (top-k {node, quantized activation})
walk_without = settle-set S′ minus edge e
gain(e)      = diff(S, S′)           — what this edge was worth in the thought
```

Cost: the frontier is ~10², so a counterfactual re-walk is one extra walk
per probed edge — milliseconds on GPU (GPU leg UNMEASURED; v1 runs this on
the CPU walker, still laptop-OK because probe count is bounded: only edges
in the settle trace are counterfactualized). No-LM discrete error metric
beyond `/correct` (rank of truth node in settle top-k) is **OPEN-14** —
developed at build, not silently fixed.

Pre-walk fallback retained: rank-of-truth probe over the token-transition
table for statements that never produce a walkable query
(`error = rank of truth`, ties by `last_seen`). Every probe writes its
numbers (trace hashes, not float bits — §16.6) to the mind log; a
confidence change with no logged probe is invalid.

## 8.2 The exact measurement (correction events — primary currency)

For a `/correct` triple (query, guess o_wrong, correction), **and
identically for an `answer-to-ask` commit** (§1.4/§17.2.3, where the
error-before is the named empty slot SI asked about instead of a wrong
claim):

```
error_without = 1  (the guess was wrong — measured, not estimated)
error_with    = 0  (the row, when it serves correctly)
gain_exact    = +1 correction-unit  on every successful later serve
                  that a subsequent /correct does NOT negate
loss_exact    = −1 correction-unit  when a later /correct supersedes
                  this row's answer for the same stimulus
```

Confidence writeback: `gain_exact` total normalized over serves, applied
per §8.4. In trace v2 the very first row's error-before is the **named empty
slot** SI asked about (Act 0 ask-back, §7.4) rather than a nonce — same
property, cleaner bookkeeping: the gain is trivially positive because
nothing served correctly before. Stated openly: early gain is about
*whether a row exists at all*; discriminating gain requires a populated
prediction table.

**Demonstrated gain vs asserted gain (TEACH fragment — the ledger names the
two, because `/teach` made the difference visible).** Every row records
`gain_kind: demonstrated | asserted` (§5.4's envelope):

| kind | rows born this way | error-before | what the number means |
|------|-------------------|--------------|------------------------|
| **demonstrated** | `/correct` (T3), `answer-to-ask` (§1.4) — **and the v2 arc's nomination confirmations are ordinary `answer-to-ask` commits, not a sixth `prov_kind`** (§22.4, §25.5: `question_id` names the `cluster-nomination` / `move-nomination` ask row, which *is* the named empty slot this table's error-before column asks for) | **measured** — a wrong output `o_wrong`, or the named empty slot SI itself asked about | `gain_exact` is a fact about this session: writing the row demonstrably replaced something that failed |
| **asserted** | `/teach` (§1.4), plain owner statements that pattern-match with no pending ask (T2), T1 inferences and RE-PARSE derivations | **none** — nothing wrong was guessed first | `gain_exact` starts at 0; the row's worth is a **claim by the owner**, and it converts to demonstrated at the first recall that serves from it (§8.2 above: "early gain is about whether a row exists at all") |

Nothing here is a new currency: it is the same `gain_exact`, labeled with how
it was earned, so a transcript can never read an imported lattice as a
demonstrated one, and no gate may count asserted units toward a calibration
claim. **This distinction existed in the design before `/teach`** — a bare T2
statement has no error-before either — the new channel is simply the first
thing that made it load-bearing enough to name. R-07 (confidence theater) is
what this field is the guard against.

## 8.3 Where each term is measured

| term | measurement point |
|------|-------------------|
| error-before | recorded from the actual served output o_wrong at correction time — **or from the open `questions` row** (named empty slot) at auto-commit time (§17.3). **Absent by construction for an asserted row** (`/teach`, plain statement, T1, re-parse): no output preceded it, so `bind_output_id` stays `null` (§5.4) and the row carries `gain_kind: asserted` — never a fabricated zero-error |
| gain_kind | decided once at commit from `provenance.kind` (§8.2 table): `correction`/`answer-to-ask` ⇒ **demonstrated**; `owner-teach`/`statement`/inferred/re-parse ⇒ **asserted**. The first recall that serves the row and is not corrected **promotes the row's ledger to demonstrated** — that promotion is an event-log entry with the serving step id, and it is the only promotion the ledger has |
| ground truth | the `/correct` payload, **or the reply line that closed a pending ask** (both stored in the row with provenance), **or the RIGHT side of `/teach … => …`** (§1.4) — **and in all three cases the line is `source: owner`; a supporter reply is not ground truth, it is input (§21.4, R-32)** |
| gain/loss | next serves: match ⇒ +1, superseded-by-correction ⇒ −1 |
| surprise | §6.2, at write time |
| contradiction | R6 at write time + labeler comparing retrieved sets |

## 8.4 Confidence (event-driven only; no clock decay — I5)

| event | Δ |
|-------|---|
| commit from `/correct` (T3) | init 0.95 |
| commit from **`answer-to-ask`** auto-commit (T3-context, §17.2.3) | init 0.95 (hole was just measured ⇒ high surprise) |
| commit from **`/teach`** (owner-asserted, `gain_kind: asserted`, TEACH fragment §1.4) | init **0.95** — the owner's fragment says "owner-tier", and owner tier is the top confidence tier. **Its arbitration rank is nonetheless T2** (§6.4): an asserted row never outranks a demonstrated correction, and **OD-41** records the split so nobody "fixes" it later by promoting the tier |
| commit from owner statement (T2) | init 0.90 |
| commit system-inferred (T1: bridge, reparse first pass) | init 0.50–0.60 |
| served and not corrected next turn | +0.05 (cap 0.99), bump `last_confirmed_at` |
| exact repeat taught **by the owner** (`source: owner`) | +0.05, `repeats += 1`. **A `source: grimm` echo earns nothing here (§21.4, R-35)** — the row's table entry is "repeat **taught**", and only the owner teaches; his repetition is rehearsal, which touches §8.4's decay curve and buys no durable bump |
| loss_exact (superseded by correction) | aged row frozen; new row inits fresh |
| won arbitration against T1 | +0.10 |
| probe gain (§8.1) | +0.05 · min(gain, G_max)/G_max |
| **charge BAKE at formation (§20.4, §20.4a)** | the stamp buys the edge a **durable** head start on this row's plain init — the init is `conf + bake`, where **`bake = B_max · tanh(k · a_formation)` (§20.4a, OD-33)**; **the only write-time decay path in v1 is the normal §8.4 confidence/rehearsal curve applied to the ROW, not to the bake**. **Round 2's "boost decays toward plain confidence" row is superseded by OD-27 — RESOLVED: permanent, no decay tail, saturating scale** (owner round 3: *"one genuinely scary moment makes that specific edge hot forever, even while the word's median drifts calm"*) |
| **read-side recency (§20.3)** | not a confidence rule at all: a stamp's *weight in the point's median* ages on the same recency curve facts use (G3's clock, §19.2's recency weight, §8.4's decay-deficit floor — **one curve**, R-29). Storage never changes; only the statistic's anchor moves |

**Salience and the bake — one curve, one moment, and it is `a` (§20.4a,
OD-33).** Round 1 wrote `salience = f(e)`; round 2's OD-22 law was
`salience(event) = f(signal_magnitude)` and explicitly **not** `f(a)`, because
arousal modulates **the walk only**. Round 3's amendment narrows that law rather
than deleting it: **the stamp's `magnitude` field still reads the firing
signal's own value** (provenance, OD-22 intact), while **the edge's durable
write-side head start — the bake — is `B_max · tanh(k · a_formation)`**, the live
signal sum read through the one integrator **at the formation instant**. A row
that landed while the loop was hot is stamped harder and conducts easier forever;
a repeat aside in a calm walk does not.

What the amendment does **not** buy it: a read-time `a_at` multiplier, a category
test, or any influence on content. `a_formation` is one quantized scalar produced
by machinery that exists for its own reasons; nothing in the formula knows a
kitchen fire from a surprise, which is the owner's point — *"it felt a lot when it
happened."* If the curve ever took an event **kind** as an argument, this
section's whole anti-cheat argument would collapse, and that is what **OD-33**
exists to make visible: **one curve on the write side, in a design whose rule was
zero.** The saturation is the price of making it lawful: `tanh` is what stops a
permanent quantity from being an unbeatable one (§20.4a, R-22's third bound).
Constraints, all inherited rather than invented here: (a) **a floor** —
a zero-signal write still gets §8.4's plain init (round 1's "one resident at
zero can't mute the write" was composition-rule language, and composition is
**CANCELLED**: OPEN-17/OD-16 are superseded by OD-21) — and the floor is now
doubly load-bearing, because the bake is permanent and a mute-by-arousal init
would be permanent damage; (b) the result stays in
[0, 0.99] (§8.4's existing clamp); (c) `a`, its **per-thought trajectory**, the
signal rows, the chosen multiplier and the stamps **including `a_at`** are
**quantized into the state hash** (I5, §16.6), so
replay reproduces the arousal trajectory and the label stream byte-exactly;
(d) **every magnitude is UNMEASURED until `gates/G9/arousal_sweep.json`
exists** (OPEN-16, now two constants plus the modulation curve 16a, not a
matrix) — the v1 sweep starts from the flat-1.0 baseline this table had before
the emotions chunk, so the *shape* is testable and the *sizes* are earned, not
asserted.
