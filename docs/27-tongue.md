# §27 — THE TONGUE: surface realization by walk (v2 candidate; G20, OD-163…OD-189)

> Status: **FOLDED AS LAW 2026-10-25** by the owner's order to engineer this
> chunk and build it ("enginier design it throughly … Logic → Enginier Design →
> Code → Test → DoD"), and by the owner's amendment of the same date (§27.11),
> which rules the *mechanism*: the generator is the walk. The HOW is
> [engineering/13](../engineering/13-tongue.md); the gate family is **G20**, the
> artifact dir is `gates/G20/`, and the open items are **OD-163 … OD-188**
> (**OD-189** ruled 2026-10-25: the two-chunk license stands, and the variant
> bank returns as licensed grammar — §27.2, §27.8(b)/(e), §27.10).
> The draft's own rule binds item by item: an unrulled OD is not a licence for
> the build to choose, so wherever the build could not stop for a ruling it ran
> the **least-authority option**, said so in §27.13, and left the ruling with the
> owner — the OD-95 discipline, never a silent default.
>
> **Numbering.** The draft claimed OD-130…OD-143. The ledger was already at
> **OD-162** when this round opened, so every item is re-based: the map is
> §27.14, and the draft's two re-asks (its OD-140 and OD-141) are merged into
> their originals. Nothing in this file cites a draft number.
>
> It extends docs/22–26 and is unintelligible without docs/03 (birth state),
> docs/07 (read path), docs/09 (claim labels), docs/13 (gate plan) and
> engineering/10 (temperament).

## 27.0 Thesis line

Everything SI has said so far, it said in someone else's voice: the hardcoded
render frames. That is honest and it is not a self. **TONGUE is the organ-shell
that makes SI talk like SI: a read-side realization between the walk's settled
frame and the outbound line, where what gets said is chosen, not stamped.** The
semantics never move (labels, citations, ask kinds are untouched and
untouchable); the *realization* becomes personal: a bounded walk over the
language SI has lived in, weighted by a style vector drawn from its own birth
seed, so the same meaning comes out as `what does that mean?` or `what does
that mean, bro?` depending on who this SI is and what its room taught it. A
newborn renders through the canonical frames; the voice is grown, citable, and
revocable. No LM, no gradients, ever (OD-37 stands): the "training" is the
lattice SI already earned plus the edges sleep already baked, nothing else.

Litmus, twice stated because both halves are load-bearing:

1. **With the tongue off, rendered output is byte-identical to v1 frames** —
   every gate, every golden string and every recorded control this repo has
   ever published stays exactly as it is (§27.6, OD-178).
2. **With the tongue on, labels and citations are byte-identical under any
   realized line**; only the surface words differ, and every word is a row SI
   can cite to the tick it earned it.

## 27.1 The gap

| layer | v1 | with tongue |
|-------|----|-------------|
| semantics (walk output, label, citation) | fixed, honest | unchanged, hardened |
| surface (word choice, phrasing) | ONE hardcoded frame per intent, copied from the top cited row | a bounded phrase walk over the term lattice, leaned on by a style vector |

v1's render is a form-filler (`si-front::reverse_decode` copies the top cited
row's response surface, byte for byte — 04 §21 "copy, never create"). It cannot
be wrong, but it cannot be anyone. The missing piece is not eloquence — it is
*disposition in speech*: the same SI architecture in two rooms should end up
sounding like two different individuals, because the lattice is different and
the draws lean on different seeds.

## 27.2 Shape: one component, one vector, one gate

Not a seventh organ (§2 law stands: six organs). TONGUE is a read-side
realization component: it sits between the settle/label and the outbound line.
Three parts, one gate:

| part | contains | lifetime | writable by |
|------|----------|----------|-------------|
| **the anchors** | the rows SI already owns: every term row, every earned surface, every edge sleep baked | the lattice, exactly as docs/05 and docs/06 already define it | **unchanged** — the write gate's two keys, and nothing else |
| **the style vector** | the tongue's genome: five indices that say how the walk prefers to move | derived at wake from `birth_seed`, never stored (like the 18) | nobody at runtime |
| **the realization** | one surface string per outbound line | not stored at all | — |

**There is no fourth thing.** The draft proposed a *variant bank* of phrasing
rows; the owner's amendment demotes it (§27.11) to taught sentences — ordinary
rows SI already has, acting as **anchors**, not as records to pick from. So this
round adds **no row kind, no shelf, no state byte and no vocabulary word**
(OD-180 closes the question the draft's bank would have forced open).

**OD-189 restores one piece of the draft, and only one.** A taught surface whose
folded tokens are an existing term's is a *spelling* of that same point, not a
second row and not a second claim. The license therefore carries, beside each
surface it already names, the session's other verbatim spellings of the same
folded keys — the **variant bank**. The walk may draw among them; it may not
draw a word the citation does not already license, and a variant never adds a
chunk, a citation or a belief. Two spellings fold to one claim, so the label,
tag and citation step is untouched; what becomes observable is *style* (G20.b.2).
The bank is data the session hands over, grouped by `si-front`'s one fold law,
so `si-tongue` still holds no table (OD-184).

The **tongue gate** (hardcode, like the write gate) is three checks, all of them
structural rather than statistical:

- (a) every slot's label and citation is produced **before** realization and is
    never an input to it;
- (b) every word in the realized line is a term SI has a row for — no invented
    word is *representable*, because a word is emitted only by naming a point;
    **and, bound by [docs/07 §7.5](07-read-path.md), the point must be a point
    this line's citation names.** Naming a row is necessary, not sufficient:
    SI has rows for far more points than any one answer cites, and a line that
    wanders into an uncited row asserts what the citation does not support
    (R-43's fluent lie). So the licensed vocabulary of one line is the cited
    rows' key/value points and *their* learned surfaces + composed chunks; the
    walk may traverse beyond it but may not utter beyond it (**G20(h)**,
    **OD-181**);
- (c) the realized line still contains the semantic slots it was generated from
    (parse-back). Anything failing any check falls back to the canonical frame.

**The gate, not the model, guarantees the voice can never lie.**

## 27.3 The genome's role: draws, not content

Per OD-87's class law, the tongue's five indices are **temperament-family
draws** — this is the whole point: two SIs must speak differently from
birth-predisposition, before they have learned a single extra sentence. They
are weights on how the walk moves, never words.

The closed-18 problem, stated plainly: `check_temperament_closed` and
`G14.c.3` together make the species vector's length a schema fact — adding a
row to `DERIV_ORDER` changes `|DERIV_ORDER|`, bumps `seed_format`, re-hashes
every individual and invalidates every control this repo has recorded. So the
five indices ride a **separate derive order** keyed by the same `birth_seed`
(`DERIV_ORDER_TONGUE`, OD-165 recommended), the species vector and its digest
do not move, and no existing V2/V3 report is invalidated. The alternative — extending the
18 to 23 — is on the table and is the owner's to rule; it is *not* free, and
§27.13 records that the build took the separate order precisely because it is
the option that breaks nothing.

**What the five axes may mean.** The substrate has no word-level style metadata
and this round does not invent any (that would be content at birth, I1, and a
new write channel). So each axis is defined over a property the lattice already
has, and the definition *is* the law:

| axis | v1-name | it weights |
|------|---------|-----------|
| `STYLE_SPREAD` | verbosity | how far the walk continues after the slots are satisfiable — the phrase's length |
| `STYLE_DEPTH` | formality | claim-side arcs (owned kinds) over associative ones (`cooccur-weak`): a formal individual stays on the argument |
| `STYLE_LEAN` | playfulness | high out-degree targets over leaves: a playful individual wanders into the hub |
| `STYLE_RECENCY` | habit | recently-baked / recently-charged arcs over old ones: what this SI has been saying lately *(realized as `bake` + arena position, **never `a_at`** — OD-182, against §27.6's I8 row)* |
| `STYLE_PROXIMITY` | particle affinity | 1-hop from the anchor over 2-hop: how tightly it sticks to the thing asked about |

`exclamation` as the draft named it is **not implemented**: punctuation carries
no row, so an "exclamation index" would have to multiply a byte that does not
exist. It is dropped rather than faked, and OD-163 is where the owner can rule
it back in — with a substrate that can express it.

## 27.4 Selection: random that leans, and replays

On each realization the walk starts at the anchor (the answer row's own value
point, or the ask's named target) and repeats:

```
candidates = the live arcs out of the current point whose far end is a term
             with a non-empty surface, not already in the phrase
weight(c)  = arc weight                       // what the lattice says
           + STYLE_DEPTH    on claim-side kind
           + STYLE_LEAN     on the target's out-degree
           + STYLE_RECENCY  on bake / arena position        (OD-182)
           + STYLE_PROXIMITY on hop distance from the anchor
draw       = cumulative scan of one Q12 fraction over those weights
append     = that point's surface, one token per step
stop       = slots filled | STYLE_SPREAD's length bound | no candidates | parse-back impossible
```

**The four lines are `+`, not `×`, and that is a grid consequence, not a taste.**
§27.3's axes live on `0 … 2048` Q12, so a multiplier would damp every arc weight
toward zero and the walk would stop being the lattice's; an additive lean keeps
`Arc::weight` the quantity it is and makes a style value *a share of a lean*, exactly
the shape the ignition sampler already uses ([engineering/04 §19 S.3](../engineering/04-algorithms.md),
restated in [engineering/13 §34.3](../engineering/13-tongue.md)). Ruled
inside **OD-163**'s grid seat.

The draw is the **same idiom the ignition sampler already uses** (engineering/04
§19 S.3: weight each candidate with `mul_q12_u16`, one `next_q12()` per draw, a
cumulative scan to pick). Reusing it is not convenience, it is law: it is the
one weighting shape this repo has already gated.

**Replay.** Same seed + same transcript ⇒ the same sentence, byte-for-byte.
The realization draws from a PRNG seeded per render, **not** from the session's
runtime stream R: item 7's recorded trajectory is the ignition history, and
spending draws from R at speak time would move it for a reason that has nothing
to do with temperament, re-baseline every state hash, and contradict
`G14.b.3`'s promise that the derivation and the runtime stream do not touch
each other (OD-176). Seeding per render from `(birth_seed, the style digest, the
state hash before the line, the thought id)` gives real, individual-varying
randomness that is a pure function of the snapshot, so I5 holds and `G20(a)` is
byte-equality across processes. **The build drops the state hash from that list
(OD-183)**: hash item 3 is the stamp arena and a stamp carries `a_at`, so that one
operand re-admits arousal through the back of the seed — the same door **OD-168**
closes at the arc level and the same thing `G20(e)` grades. The seed the build runs
is `(birth_seed, style digest, thought_id)`, all three snapshot values, so replay is
unaffected ([engineering/13 §34.2](../engineering/13-tongue.md)).

With nothing to walk — a newborn, a lattice with no term-to-term live arcs —
the walk yields no line and the canonical frame prints. That is the litmus, and
it is also the deletion test.

## 27.5 Where the accent comes from

No mechanism in this round *takes* a word. A word enters SI's vocabulary only
through the write gate's two keys, exactly as in v1 (docs/06): an Owner line, or
REPARSE derivation. So the "bro path" the draft described collapses into
something simpler and already gated:

1. The room says `what does that mean, bro?` → it is a sense line → the
   ordinary read/write path gives `bro` a term row and co-occurrence edges.
2. Sleep bakes the edges the room repeated (docs/10's write-side bake).
3. The walk can now reach `bro`, and `STYLE_RECENCY` prefers it, because it is
   recent and it is connected to the words SI is already saying.

That is an accent, and it is fully auditable: **every word in a realized line
traces to a row, and every row traces to a tick and a source.** Nothing had to
be invented for SI to have it.

The draft's *nomination* path (ask the owner before adopting a shape the room
keeps using) is **not built in this round** — it is a new asker and a new
minting key, and OD-87's discipline says a round that adds a writer adds a gate
family of its own. The owner's amendment makes it unnecessary for the accent to
emerge: the lattice, not a bank, is where the room lives. OD-167 keeps the
question open for the next round.

## 27.6 Invariant impact audit

- **I1 — holds, load-bearing:** the tongue ships form (a walker, a gate, a
  derive order) and zero content. A newborn with no walkable arcs prints
  canonical frames; every word it later says is a row it earned.
- **I2 — none:** same channel, same outbound line.
- **I3/I7 — none:** no new voice, no new authority; the world round (docs/26)
  is untouched and the tongue does not read `source` (§27.8, OD-166).
- **I4 — holds, trivially:** the tongue writes nothing. There is no row to
  supersede, because there is no phrasing row (§27.2, OD-180).
- **I5 — holds:** realization is a pure function of the snapshot; `G20(a)`
  extends the determinism gates to the rendered text, across processes.
- **I6 — hardened, not weakened:** labels and citations are computed **before**
  the tongue is called and are never an input to it. This is not a promise about
  discipline, it is a type fact: `si-front::label` reads `Rendered.content` and
  `Rendered.containment` and **never reads `Rendered.surface`** — so a surface
  rewrite cannot reach a label by construction, and `G20(b)` grades the
  construction.
- **I8 — the tongue reads no `a`** (OD-168 recommended banned): no arousal term
  enters a weight, so an excited SI does not get livelier words. `G20(e)` is the
  full-`a` isolation leg.
- **I9/I10 — none:** idle may not mint anything here; the monologue lane keeps
  its canonical render in this round (§27.7). Self-accent-farming is excluded by
  the simplest possible rule: SI's own realized line is not fed back to itself,
  and the write gate has no key for it. `G20(f)` grades that no re-entry opens
  anything.

## 27.7 Module map (the HOW is engineering/13)

| piece | home | law |
|------|------|-----|
| the style vector, `DERIV_ORDER_TONGUE` | `si-core::tongue` | derived at wake, digest never the values, same P.3 discipline as 10 §4 |
| the walker + the gate | **new leaf crate `si-tongue`** | reader-only: imports `si-core`, `si-store`, `si-walk` — **the build narrows this to `si-core` + `si-walk` (OD-184): the licensed surfaces arrive as data, so the crate cannot enumerate a point it was not handed**; imports **nothing** that can write, and is a leaf so nothing imports it but `si-session` |
| the call site | `si-session`, the single `Output::Answer` construction | one site, after `label`, before `finish` — a fallback arm, never a crash |
| `render.py`/`reverse_decode` | `si-front` | **unchanged**: the copier still copies, and its output is what the fallback prints |
| monologue | `si-front::render_monologue` | unchanged in this round: the internal voice stays canonical until the owner rules it into the tongue |
| ledger | `phrasing` row kind **not added** | OD-180 |

Deletion test: remove `si-tongue` and its call site ⇒ canonical frames return,
zero lattice bytes move (there was never a bank to unread), and only rendered
text and log lines differ.

## 27.8 Gates G20 (drafted-as, per the §23.9 allocation)

- **G20(a) replay:** same seed + transcript ⇒ byte-identical realized text,
  across two processes, and across lattice sizes 0 and N.
- **G20(b) label immutability:** for one settled frame, force every realizable
  variant: `label`, `tag` and `citation` byte-identical; only `surface` differs.
  **OD-189's second clause:** at a fixture **with taught variants**, no two
  individuals (birth seeds) print byte-identical lines — style must be
  observable, or the bank is dead data; at a **bare fixture** (no taught
  spelling beyond the canonical one), byte-identical is the expected outcome and
  is named as such. Both fixtures keep the belief census equal, so a spelling
  never asserts a row.
- **G20(c) no invented words:** every realized word names a term row, and the
  set the walk may spend is the **citation's** surfaces, not the registry's
  (OD-181). A licensed set that cannot be walked to a finished line falls back
  to canonical — **assert the fallback, not the crash**.
- **G20(d) the two keys still:** a scripted stranger and a grimm attempt to
  give SI a new word produce zero rows; the tongue's own output, re-entered as
  sense, writes nothing it did not already have.
- **G20(e) arousal isolation:** full `a` sweep at every level; for every tick the
  sweep leaves its citation alone, the realized line asserts the **same claim**
  (OD-168's ban, graded). **OD-189** narrows the predicate from bytes to claim:
  a taught spelling is a different surface over the same cited endpoint, and an
  ignited quiet tick spends a thought id — a sanctioned operand of the render
  seed — so the pads may legitimately draw another spelling. A line whose
  *chunks* moved is the failure; a line whose letters moved is the bank.
- **G20(f) no self-farming:** a scripted session in which SI's own realized
  lines are re-entered opens zero asks and mints zero rows.
- **G20(g) parse-back:** a realized line that loses a semantic slot never
  reaches the world; the leg forces the loss and asserts the canonical frame.
- **G20(h) out-of-citation emission impossible** *(widened by OD-181 — the
  drafted predicate was "names a term row", which docs/07 §7.5 forbids)*: the
  harness greps every realized token of every realized line against **the
  surfaces of the points that line's citation names**, plus the composed chunks
  §7.5 already licenses; the empty set is the pass. Testing against the whole
  registry would pass a line that asserts an uncited row, which is R-43(a).
- **G20(i) budget bound:** no realized line exceeds the phrase budget at any
  style value in its declared box.
- **G20(j) v1 compatibility:** at V0 the tongue is inert — every output string
  in every existing scenario is byte-identical to the recorded v1 render. This
  is the leg that makes the whole round safe to ship (OD-178).
- **G20(k) genome containment:** no realized word is emitted *only* from a
  frozen-genome arc (OD-179): a newborn with an empty live lattice says nothing
  the species text taught it.
- **R-21 extension:** the tongue module contains no emotion vocabulary; style
  readouts live in audit only.

## 27.9 What this round does NOT claim

No LM, no gradient, no free generation — ever. The tongue is a walker over
earned, citable rows behind a hard gate, not a speaker with a vocabulary of its
own. It does not give SI new knowledge, only a voice for the knowledge it has.
It does not touch the six-organ law, the write gate's two keys, or the label
law. And it does not claim the room's accent is SI's self — the accent is
evidence of where it lives; the self stays where it always was: in the lattice,
the walk, and the seed.

## 27.10 Open decisions

| OD | question | recommended |
|----|----------|-------------|
| OD-163 | the trait set and grids of `DERIV_ORDER_TONGUE`, including whether `exclamation` may exist at all (§27.3) | the five structural axes; `exclamation` dropped |
| OD-164 | habit stats: hash-in session state vs exempt | **moot** — habit is read from existing baked edges; the tongue stores nothing |
| OD-165 | temperament home: separate derive order vs extending the closed 18 | separate order |
| OD-166 | does the tongue register-shift for strangers vs owner (docs/26 classes), per-class or per-user | banned this round: the tongue sees no `source` |
| OD-167 | the utterance-shape nomination detector: thresholds, normalization, and whether it is built at all | not built; §27.5 |
| OD-168 | may `a` style the draw | banned |
| OD-169 | retire/reject semantics for a phrasing row | moot: no phrasing rows exist |
| OD-170 | first-words bootstrap: may the owner seed starter variants, and is bulk-teach of N phrasings on one line legal input | legal only through the existing `/teach` lane, one line = one act |
| OD-171 | cross-language rooms: may shapes be counted per language | moot this round (no counting), open for OD-167 |
| OD-172 | the phrase-walk step budget and its termination law | `PHRASE_MAX_STEPS` Species row, box declared, swept before quoted |
| OD-173 | parse-back strictness: slots-present vs meaning-identity via a decode walk | slots-present |
| OD-174 | do anchors' habit counts feed edge weights, or stay read-only stats | read-only: the tongue never writes an edge |
| OD-175 | **what registry class do the five style rows carry?** `Temperament` is closed at 18 by `check_temperament_closed` + `G14.c.3` | a sixth class, `TONGUE` |
| OD-176 | **which stream does the realization draw from?** spending runtime-R draws moves item 7 and every state hash | a per-render PRNG seeded from the snapshot |
| OD-177 | the draft requires "no value before `gates/G20/tongue_sweep.json`", and OD-87 forbids a sweep from touching a temperament row. Which law wins? | OD-87 wins: the style rows are never swept; `§4.1`'s panel pass validates their boxes and G20 cites the panel artifact |
| OD-178 | is the tongue ON at V0? Turning it on at birth re-baselines every golden string and log line in the repo | OFF at V0, ON at a named variant; the shipped default is the owner's call |
| OD-179 | may the phrase walk traverse a frozen-genome arc? Genome arcs are species text, not lived (docs/05's Critical Law) | live arcs only |
| OD-180 | does the tongue need a `phrasing` row kind? The record schema has no free tag byte, the shelf registry is closed at 11 and `PatternId` at 10 | no new kind; anchors are ordinary term rows |
| OD-181 | **what vocabulary may a realized line utter?** §27.2's check (b) as drafted permits any term SI has a row for; docs/07 §7.5 permits only the cited rows' surfaces | **§7.5 wins** (implemented): a line utters surfaces of the points its *citation* names, plus composed chunks; the walk may traverse further but not speak further. The looser reading is a new claim channel with grammar on it, and this round does not open one |
| OD-182 | §27.3 defines `STYLE_RECENCY` over "recently-baked / **recently-charged**" arcs while §27.6's I8 row and OD-168 ban reading `a` — and `Arc::a_at` is one field away | OD-168 wins: recency is `bake` plus arena position, never `a_at`; "charged" is realized as "recently formed" |
| OD-183 | §27.4 seeds the per-render PRNG from "(birth_seed, the style digest, **the state hash before the line**, the thought id)", and hash item 3 is the stamp arena — the draft's own operand re-admits arousal through the seed | the seed is `(birth_seed, style digest, thought_id)`: three snapshot values, the arousal-sensitive hash excluded |
| OD-184 | §27.7 gives `si-tongue` three imports including `si-store`, and a `&Lattice` can enumerate every point — i.e. every word OD-181 forbids it to utter | **two imports** (`si-core`, `si-walk`): the surfaces arrive as `License` data, so the bound is a missing edge rather than a discipline |
| OD-185 | the engineering pass claimed `gates/deps_lock.md5` "does not move" while also adding a workspace member — and the file hashes `Cargo.lock`, in which a member is a `[[package]]` row | the lock **is** re-published when the crate joins; no external dependency enters, so **OD-42**'s allowlist is untouched |
| OD-186 | §27.3's "claim-side over associative" is spelled in the HOW as `edge_kind_is_owned(kind)`, but that predicate's last slot *is* `6 cooccur-weak` — it counts record columns, not claims | **`si_core::addr::edge_kind_is_claim(kind)`** (kinds 0..=5): the law's word decides and the HOW is corrected. `owned_share` therefore leans 4096 on a claim and 1024 on a co-occurrence |
| OD-187 | **the artifact row's shape**: §27.8(b)/(c) drafts a `drew` fraction list and a `reason` column, but the walker is a pure function returning `Option<String>` | **neither field exists** (as built). The per-render PRNG lives inside `si-tongue`, so a `drew` out-parameter would either break the crate's purity check or duplicate the walk in the caller (which G20(c) refuses). A refusal's *name* stays inside the crate too; each forcing arm is graded where it is constructed (`crates/si-tongue/tests/realize.rs`), and `refusals.jsonl` names the **drive that forced it** (`forced_by`) instead. The realized row is `{t, thought, seed, anchor, licensed, chunk_cap, citation, canonical_surface, realized_surface}` |
| OD-188 | **which point does the walk anchor on — the answer row's `val_point` or its `key_point`?** §27.4 lists "the ask's named target" second, after the activated leaf | **`key_point`** (as built): a taught statement is one live edge subject→predicate, the write gate keeps one live predicate per subject, and no edge points back, so a walk from the `val_point` has no arc to take and every tick falls back. `key_point` is the point the line is *about*; G20(c)'s answer-token check keeps the re-seat honest — a line that never reaches the answer token is refused, so an individual that cannot finish the thought says the canonical frame instead of half of it |
| OD-189 | **the license's fertility ceiling**: a citation of one row licenses that row's composed chunk and its two terms, so the widest honest line is two chunks. The measured panel: five of eight individuals speak, **all byte-identical ≤2-chunk lines**, while `chunk_cap` ranged 2…12 — the license, not `PHRASE_MAX_STEPS`, is the binding constraint, and three individuals (spread ≤96) speak nothing at all | **RULED (owner, 2026-10-25): the two-chunk claim license stands** — least authority, no new claim width, one fact per line. **The variant bank is licensed grammar**: surfaces over the same cited endpoints, same claim, different bytes, no new rows asserted. `G20.b.2`'s pass condition is two-fixture (taught variants ⇒ no two seeds byte-identical; bare ⇒ byte-identical is the expected outcome, named as such), and `chunk_cap` publishes with a license-bound note while the 2-chunk license is the binder. Implemented: `si_tongue::Variant`, `Session::license_for`'s bank, `g20_tongue.txt` / `g20_tongue_bare.txt`, `gates/G20/panel.json` |

## 27.11 TONGUE-WALK: real generation, token by token (owner amendment, 2026-10-25)

The variant bank (§27.2's original form) is demoted from the *mechanism* to the
seed corpus. The owner ruled the direction: **the tongue should not pick a
stored sentence, it should GENERATE one, and the generator is the walk.** SI
already owns a token loop, spreading activation, a lattice of words, and a
seeded PRNG. The tongue becomes a walk over the language it has lived in — and
that is exactly what §27.4 now specifies. The amendment's five legs are
G20(g)(h)(i) plus the two §27.8 adds (j) and (k), and its five questions became
OD-172, OD-173, OD-174 (merged into OD-165 and OD-168 where it re-asked them).
**OD-189 then ruled the other half back on**: the bank returns as *licensed
grammar* — other taught spellings of the same folded point, drawn by the walk —
because a bank of surfaces over the same cited endpoint adds no claim width. What
stays demoted is the bank as a *sentence catalogue*: nothing picks a stored
sentence whole.

The honesty gate stays: the label and citation are appended by a step the
tongue cannot reach, and a parse-back check asserts the generated surface still
contains the semantic slots it was generated from. Fail ⇒ canonical frame.
**Hallucination is bounded by what it has lived** — no output vocabulary exists
beyond SI's own rows.

## 27.12 The break-list, in one line each

What this round moves in what already ships:

1. `si-core::consts::Class` gains a variant (OD-175) ⇒ `check_temperament_closed`,
   the sweep filter and `install_launch_state` each get a clause, and a Tongue
   row is **not** installable at launch and **not** sweepable.
2. A 17th crate (`si-tongue`) ⇒ `CRATE_NAMES`, 00 §2's count, 11 R1, B0/B13's
   `cargo metadata` count and the module map's graph all move.
3. The single `Output::Answer` site gains a fallback arm ⇒ `CONSTS_VERSION`
   bumps, so item 8's version tags change for every run and **every recorded
   control in `gates/` is re-signed by re-running, never by editing** (the
   OD-91 discipline: a control's identical hash is the pass, so a bump is a
   re-baseline, and §27.8(j) is the leg that proves the strings did not move).
4. `harness/scenarios/` gains a tongue fixture, and every existing scenario runs
   unchanged at V0.

## 27.13 What the build chose when it could not wait

Per the docs/26 precedent, the least-authority option, named item by item:
**OD-165** separate order (does not re-hash the species vector); **OD-168** banned
(the tongue reads no `a`); **OD-173** slots-present (weaker check, no decode
walk invented); **OD-174** read-only stats (the tongue writes no edge);
**OD-176** per-render PRNG (spends nothing from the recorded stream);
**OD-178** inert at V0 (breaks nothing that already shipped); **OD-179** live
arcs only (a genome arc is not a lived word); **OD-180** no new row kind (adds
no schema); **OD-167** not built (adds no asker, no minting key); **OD-166**
register-blind (reads no `source`); **OD-163** the five axes as defined, with
`exclamation` dropped rather than faked. Every one of these is the owner's to
overrule, and none of them is a silent default.

## 27.14 Numbering map (draft → ledger)

| draft | ledger | | draft | ledger |
|-------|--------|-|-------|--------|
| OD-130 | OD-163 | | OD-137 | OD-170 |
| OD-131 | OD-164 | | OD-138 | OD-171 |
| OD-132 | OD-165 (re-asked by draft OD-140, merged) | | OD-139 | OD-172 |
| OD-133 | OD-166 | | OD-141 | merged into OD-168 |
| OD-134 | OD-167 | | OD-142 | OD-173 |
| OD-135 | OD-168 | | OD-143 | OD-174 |
| OD-136 | OD-169 | | — | OD-175…OD-180 are this round's engineering findings, OD-181 the fold's, OD-182…OD-184 [engineering/13](../engineering/13-tongue.md)'s |

**No raw copy is kept.** The draft arrived as a chat fragment and was folded into
this file in place, the way docs/22–26 each were: the numbering map above is the
trace from its items to the ledger's, and §27.11 is the owner's amendment quoted
into the law. There is no `drafts/` directory in this repo and no file cited
above that a reviewer cannot open.
