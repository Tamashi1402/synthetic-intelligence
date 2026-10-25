# §3 — The Birth State (t = 0, CONFIG SI)

## 3.1 Innate machinery (reflexes) — every component tagged

| # | Component | innate/learned | note |
|---|-----------|----------------|------|
| R1 | Tokenizer (whitespace split, punctuation strip, lowercase) | innate | orthography |
| R2 | Plural fold (`s`-suffix alias, meaning-blind) | innate | morphology shape; collision policy [docs/14](14-risk-register.md) R-04 |
| R3 | Pattern table (verbatim §3.2) | innate | syntax frames only |
| R4 | Gap detector (G1–G4, §7.1) | innate | predicate over empty state |
| R5 | Write gate / surprise (novelty − coverage over stored rows) | innate | arithmetic |
| R6 | Consistency checker (structural same-key conflict) | innate | no semantics |
| R7 | Arbitration policy (tiers §6.4) | innate | ordering over provenance metadata |
| R8 | Retrieval scorer (term identity, bridges, time-anchored) | innate | mechanics |
| R9 | Calibration labeler (§9 rules) | innate | labels SI's own state |
| R10 | Clock + event log + state hash | innate | the `t` axis hardware |
| R11 | **Guess slot** — nonce generator for required answers (§7.4) | innate | pseudo-tokens from character-shape stats of tokens seen THIS session; seeded by step; zero content |
| R12 | Chunk store + renderer templates | innate mechanism, learned content | v1 copy-renderer; LM render is a v2 slot (§12) |
| R13 | Shelf registry + 4D addressing | innate | schema of schemas (§5) |
| R14 | **DECODE/RENDER frames** — surface ↔ hyperpoint pattern routing (§2, §7.0) | innate frames, learned template edges | shape-only; litmus: gibberish emits the same shape; knows nothing about any word |
| R15 | **DRIVE interface** — {prioritize, emit, budget, salience} (§17.1; §18 adds `update`+`conduction` — OD-20) | innate interface | a socket, not a behavior; carries zero content; **post-round-2 it holds no emotion names either — the vocabulary lives only in the audit layer (§18.6)** |
| R16 | **CURIOSITY drive policy** — the default policy living behind R15 (§17.2) | innate policy, learned gaps | pure form: `what is X?` fires identically for gibberish X; at birth it has NO gaps to emit until tokens exist — the policy is innate, everything it acts on is learned. **Post-round-2 this is one of exactly two meanings of "curiosity"** (the other is the audit label, §18.2); the budget behind it is a queue rule + ceiling, not a level (**OD-22**) |
| R17 | **AROUSAL integrator** — one leaky scalar `a` fed by the six internal signals of §18.1's table, updated at every signal **event** during the walk (its effect lags one round, R20/§20.1), decaying in logical `t`, plus **the modulation curve** `a → (reach, decay, τ_settle)` (§18.1, §18.4) | innate **physics** (the integrator's shape + the fact that arousal bends the spread at all), learned **values**, curve swept (OPEN-16/16a) | zero content: **no named emotions are installed**, and the curve is form, not meaning (§18.1's litmus: at t=0 with zero memories **nothing fires**). **Round 1's six residents and coupling matrix are cancelled — OD-21** |
| R18 | **Edge charge field + bake** on L3 rows — `{signal, magnitude, a_at, thought_id, signal_ref}` stamped at write time and one scalar `bake` (§18.5, §20.2, §20.4a; `a_at` pending **OD-28**) | innate field, learned values | form only: the stamp names a **signal kind** and two quantized scalars, never an emotion and never a referent (round 1's `resident` field superseded → **OD-23**); frozen layers structurally have no charge field (I8/G9). At formation the stamp also buys the edge its **bake** — head start + confidence bump, **durable and saturating: `B_max · tanh(k · a_formation)`** (§20.4a, **OD-33**: the one place the write side reads `a`, at that instant only) |
| R19 | **IGNITION sampler** — on a quiet tick with an empty agenda, weight candidate hyperpoints by recency / hot stamps / decay deficit / live-contradiction warmth and light one; run the ordinary walk (§19) | innate **sampler shape**, learned **weights' inputs** | pure form: it selects among *this session's* points by quantities already in state, writes no rows, and draws from a PRNG whose **seed lives in the session snapshot** (I9, G5). "Rehearsal / worry / intrusive thought" are readouts of which weight dominated, never branches |
| R20 | **PLASTICITY LAWS** — (i) arousal integrates per signal **event** inside the walk with modulation **lagging one round**; (ii) a point's read-side emotional bias = the **recency-weighted median** of its stamp history, anchored NOW; (iii) stamps age on the **same recency curve facts use** (§20, I8); (iv) the write-side **bake saturates**: `B_max · tanh(k · a_formation)` (§20.4a) | innate **statistics + ordering + one curve**, learned **inputs** | four form laws, zero content: no word's reading exists before its first stamp (**undefined, not neutral**), the median is recomputed every thought and **stored nowhere**, the shared clock is structural — the bias module imports the fact recency curve and defines no decay of its own (R-28, R-29, G11(b)(c)) — and the bake curve consults **no event kind** (no "bad thing" category exists to bake with; R-03/R-21), which is what OD-33 had to rule open for it to read `a` at all |

| R21 | **SUPPORTER MEMBRANE** — `speakgate.py`: layer 1 = token membership over the lexicon shelf, layer 2 = **claim existence** over the live L3 rows (strict single-row restatement, row-id referenced — **OD-40(i)**); plus the sense seam's `source` tag and §21.4's privilege table ([docs/21](21-grimm-supporter.md)) | innate **form filter** | pure form, zero content: it compares surface strings and row ids that exist in *this* session's shelves, consults **no semantics, no `a`, no charge, no bake**, and adds **no string constant** to the §3.2 allowlist — anything Grimm invents that is not already taught is rejected, which is the whole point. **At `t=0` the lexicon is empty ⇒ the filter rejects everything ⇒ the supporter is structurally mute at birth** (the same empty-state litmus as §3.4, running on a component that is not part of `src/`'s loop). Asymmetry law L5: the filter applies to Grimm only, **never** to the owner |

## 3.2 The complete innate pattern table (verbatim — the whole grammar ontology)

```
P_ATTRIB   : <E> <is|are|was|were> <V> .          → attribute assertion
P_DEF      : <E> means <V>                        → taught link / same-question
P_QUEST_A  : what <C> <is|are|was|were> <E> ?     → field-question about entity
P_QUEST_D  : what <is|was> <E> ?                  → definition-question
P_QUEST_W  : <who|when|where> <is|are|was|were> <E> ?   → frame-question (§2)
P_OF_REL   : <R> of <E>                           → relation-extraction shape
              (R is a VARIABLE token slot, never a fixed word; emits R(E))
P_CORRECT  : (no wait | wait | actually | instead | rather | no,) <utterance>
P_CMD_C    : /correct <text>                      → correction act (§1.1)
P_TEACH    : /teach <pattern> => <text>           → owner teach, one row (§1.4)
              (the LEFT side is decoded by the frames above, exactly like a
               question; the RIGHT side is the response surface. Shape only —
               `/teach what znark is floop => blork` is the same act.)
```

Allowlist of string constants in the build = R1–R3 + R14 frames above +
function words {the, a, an, is, are, was, were, what, who, when, where, of,
means, yes, no} + correction markers + label names (§9) + shelf-registry
defaults (§5.2). Machine-audited by Gate G5 (§13). **The supporter batch adds
no vocabulary** — `speakgate.py` holds no word list, only a membership test
against the lexicon shelf (R21). Its two `source` enum literals (`owner`,
`grimm`) are **metadata names in the same class as the §9 label names**, and
were the only new string constants at that batch: G5c's byte-exact diff now
has to carry them, so a reviewer sees the addition rather than reading it as a
drift (§21.7). **The TEACH fragment adds exactly three more, all in the same
metadata class, and no content word: the command marker `/teach`, the binding
marker `=>` (both part of P_TEACH above), and the provenance enum literal
`owner-teach` (§5.4, §8.2).** G5c's diff carries all three; anything else on
that diff is drift, and a `/teach` implementation that grew a vocabulary to
work — a synonym table, a template list, a part-of-speech guess — is the
anti-cheat law dying by curriculum (R-03, R-18).

> **Decoder litmus (owner):** replace every word with gibberish — `what
> znark is floop`, `znark of floop`, `floop is wazz` — the decoder must
> emit the SAME SHAPE (relation slot + entity slot, frame id) with zero
> idea what it means. If any frame fires only for real English content
> words, it smuggled content. Inherited by G5d.
>
> **Flagged for ruling (OD-11):** `who/when/where` smuggle a *tiny* semantic
> distinction (person/time/place). v1 choice per owner fragment: innate, as
> pure question-frame shapes. Conservative alternative: drop P_QUEST_W and
> recognize ONLY `what`-frames at birth, letting who/when/where arrive as
> taught template edges like any other rewording.

## 3.3 Learned-at-birth? No — every learned store starts empty

| Store | t=0 | populated by |
|-------|-----|--------------|
| Lexicon (term records: id, surfaces, appears_in, bridges, defined_by) | empty | every token; **also the layer-1 reference the supporter's speak gate tests against, so an empty lexicon = a mute supporter** (R21, §21.3) |
| Shelf `s_r_pairs` (live) | empty | every `/correct`, every answer-to-ask (§1.4), every S0 assertion |
| Shelf `facts` (live) | empty | RE-PARSE (S1+) |
| Shelf `links` (bridges, live) | empty | taught `P_DEF`, confirmed resolutions |
| Co-occur / prediction table / chunk store | empty | in-session adjacency, corrections |
| Open-question queue (`questions`, §17.3) | empty | curiosity drive emissions on THINK ticks |
| **Arousal scalar `a`** (§18.1) | **at rest — 0, nothing fires** | this session's signals only — novelty spike, gain, contradiction count, settle failures, queue age, idle-cascade contradictions (§18.1, §19.5); gain + λ parked OPEN-16 |
| **Emotion labels** (audit readouts, §18.2) | none emitted — there is no pattern to name | the audit layer, over logged `(signals, a, behavior)`; **they are output, never state** |
| **Edge charge stamps** | no edges ⇒ nothing stamped | write-side stamping at commit (§18.5), stamped from the signal's own magnitude |
| **Idle state** (§19) | agenda empty, ignition candidates empty ⇒ **no idle thought at all** | only after the owner creates points worth lighting |
| **Silt table** ([docs/23](23-context.md)) | empty, and it stays empty: decay of nothing is nothing | lights deposited by frames; erodes on the logical tick |
| **Crystallize ghosts / shape-candidates** ([docs/22](22-crystallize.md), [docs/25](25-skills.md)) | **both switches ship `off`** (the `CRYSTALLIZE` / `SKILLS` ruling rows, the `INFERENCE_PROMOTION` pattern), and with no rows there is no recurrence to count and no filler-set to generalize | frames' scratch read at settle; a nomination ask; **an owner-confirmed write only** |
| **Skill rows** (`skills` shelf) | empty | owner-confirmed `move-nomination` answers |
| **Event views / `prov-link`** ([docs/24](24-episodic.md)) | nothing to bind: no commits, so no views, no links — and traversal ships `off` (`EPISODIC_TRAVERSAL`) | derived at generation from S0 rows + write events (OD-110) |
| **Grant table + speaker classes** ([docs/26](26-world.md)) | **zero grant rows**, so every non-owner line is grimm-class and `speaker_class` is a pure function of the owner's lines — tier (a) is the birth state, not a setting | `/trust` owner acts, one at a time, smallest right first |
| **Style vector + the tongue** ([docs/27](27-tongue.md)) | the five style draws **do** exist at wake — they are a lean over quantities, not a word list, and they are drawn from `birth_seed` exactly as the 18 are — but `TONGUE` ships **`off`** and with an empty lexicon the phrase walk has **no candidate**, so the newborn prints the canonical frame: §3.4's empty-state litmus running on the *read* side (I1, **G20(c)(j)(k)**) | the draws ride `DERIV_ORDER_TONGUE`; every word the walk can later utter is a surface of a point **this line's citation names** — earned through [docs/06](06-write-path.md)'s two keys and cited by one act (**OD-181**) — and the voice prints only when the owner turns the switch on (**V14**) |

**The v2 litmus, stated on the birth table where it belongs — and the one
exception, stated here once so the four mechanism docs can cite it instead of
each re-arguing it.** Born with the whole arc switched on and the owner silent:
**no belief row moves** — no `s_r_pairs`, `facts`, `links` or `skills` row, no
edge, no bake, no stamp, no confidence or `repeats` movement. Two appends still
happen, and both were already lawful before this arc: a `context` **sense row**
per admitted line (§21.4: the sense record is not a belief, carries no charge
and never answers a FACT; replay determinism depends on it existing) and the
ordinary `questions` **bookkeeping row** per emitted ask, nomination asks
included (§17.3, R-19 — an ask has always been a row about a gap, never
content). A gate that asserted literal byte-frozen state would red on its own
fixture; the protected class is beliefs, and the litmus says so precisely.
There is
no recurrence without content to recur, no warmth without a point to warm, no
origin without a commit, no trusted voice without a grant. Each of the five
proposals passes the same test §3.4 already runs on affect — *a mechanism that
can fire without content is smuggled content* — and each one is gated in the
negative (G15…G19 assert the absence, not the presence, as the first leg).

## 3.4 Proof sketch: nothing content-level is innate (CONFIG SI)

1. **Enumerability audit** — grep all string constants, diff against §3.2
   allowlist, byte-exact. Gate G5c.
2. **Empty-state behavioral test** — at t=0 every FACT-shaped answer is
   impossible (no row can exist); first answers are SPECULATION nonce guesses
   or I-DON'T-KNOW, never facts. **Affect version of the same test (the owner's
   litmus):** at t=0 `a` is **at rest and nothing fires**, because no signal has
   a source yet — and the audit layer emits **no emotion label**, because there
   is no pattern to name. A brand-new session cannot be "confused" or
   "satisfied" about anything. **Any non-zero `a`, any emitted label, or any
   charge stamp before the first signal row is written is smuggled content**
   (I1/I8, R-21). Same test on the idle side: with an empty lattice there is
   nothing to ignite, so a birth session whose monologue stream is non-empty is
   showing a preloaded corpus (I9, R-25).
3. **No-prior twin test** — disjoint transcripts ⇒ disjoint states, zero
   shared term_ids. Gate G5a/b.
4. **Gibberish parity** — `flimp is wazz` must trace identical gate
   decisions with substituted tokens; any component reacting differently to
   English vs nonce words is smuggling content. **Charge parity:** the same
   gibberish exchange must stamp the same `e` magnitudes as the English one —
   an emotion that only fires for real words knows what words mean (R-23).
   Gate G5d, G9.
5. **Clean-room isolation** — CONFIG SI binary never opens the frozen-layer
   files; enforced by a load-path assertion in the config flag itself, not
   by convention. Gate G5 + G6.
6. **Supporter-absence** — every one of the five tests above is run with the
   supporter process **not started**, and the result must be identical
   ([docs/21](21-grimm-supporter.md) §21.7, **G12(a)**). The reason this is
   listed as a proof step and not a nice-to-have: I1's claim used to be
   defeasible only by what sits in `src/`. With a pretrained model physically in
   the room, the claim must also survive the question *"what if the voice next to
   SI had said something first?"* — and the answer is that at `t=0` the gate
   rejects every emission he could make (R21's muteness), so there is nothing to
   answer.
