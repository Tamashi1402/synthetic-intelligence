# §5 — The Knowledge Substrate: the Tesseract Family

Three layers, **ONE uniform 4D addressing scheme `(t, x, y, z)`**.

## 5.1 The axes

| axis | meaning | encoding |
|------|---------|----------|
| `t` | time / knowledge state. **t=0 = frozen baseline** (version-tagged); live rows carry `t = step written`; NOW = max committed t. Point-in-time `active_at(s)`: `written_at ≤ s < superseded_at ∨ ∞` | int, logical, I5 |
| `x` | shelf id from the registry (§5.2) | shelf_id in registry |
| `y` | concept/row within shelf | row_id in shelf |
| `z` | link/edge to another hyperpoint — targets addressed as `(t,x,y)` | edge list on the row |

New knowledge types become **new shelves / row types inside the same schema,
never new structures** (structural rule). A proposed type that cannot be
expressed as a registry entry is a **design alarm to report, not to patch**
([docs/14](14-risk-register.md) R-12).

## 5.2 Shelf registry (designed, not hardcoded shelves)

Registry entry: `{shelf_name, row_schema_version, layer: frozen|live,
source, mutability, address_prefix}`. v1 contents:

| shelf | layer | row type | source | mutable? |
|-------|-------|----------|--------|----------|
| `dictionary` | L1 frozen | dictionary row | WordNet (CONFIG ENGINE only) | no — re-versioned |
| `relations` | L2 frozen | relation row | ConceptNet (CONFIG ENGINE only) | no |
| `rules` | L2 frozen | rule row | grammar property tables (CONFIG ENGINE only) | no |
| `s_r_pairs` | L3 live | S-R row | owner conversation | append-only |
| `facts` | L3 live | fact row (RE-PARSE derived) | conversation, via re-parse | append-only |
| `links` | L3 live | bridge rows (`X means Y`, same-question, **frame-synonym** templates) | taught + confirmed | append-only |
| `questions` | L3 live | **open-question rows** `{type: question, drive, kind: ask-back\|unknown-word\|cluster-nomination\|move-nomination, target: {slot\|token}, lane, asked_at, resolved_by, status}` (§17.3; the two nomination kinds and the `lane` field are [docs/22](22-crystallize.md)/[docs/25](25-skills.md)/[docs/26](26-world.md)) | curiosity drive emissions | append-only (status via new version) |
| `context` | L3 live | verbatim raw utterance rows, **each tagged `source: owner \| grimm \| trusted \| stranger`** (§21.4; the last two are derived, §5.2 note below) plus the speaking **identity** in the sense stream (§1.6) | every input line | append-only |
| `skills` | L3 live | **skill rows** — a serialized role-graph template plus its owner-given name, kind `skill` ([docs/25](25-skills.md) §25.5) | crystallize-shape nomination → owner-confirmed write | append-only (retirement = supersede, OD-122) |
| `strategies` | reserved | — | — | — |
| `events` | registered, **unwritten** | the **event view** is a *view over* `context` + write events, not a shelf of copies ([docs/24](24-episodic.md) §24.2, OD-110 ships derived) | — | — |

**The v2 arc's registry footprint, stated on the row where it belongs.** Five
proposals, **zero new structures**: `cluster-point` is a new **kind** on
`links`; `skill` is a kind on the already-reserved `skills` shelf; a
`prov-link` is a registered weak edge kind whose *rows* the fold declines to
store (OD-110: derived at generation, so `active_at` and the record layout are
untouched); a `grant` row is an owner-act kind (`links` shelf) that exists as a
schema and holds **zero rows at birth**; and the `trusted`/`stranger` classes
are **computed at generation from the grant table plus the owner's lines and
stored nowhere** — the §20-median precedent, R-28 style, which is also why the
80-byte envelope does not change. Any one of these arriving as a new structure,
a new hash item, or a stored class would be exactly the design alarm §5.1
names.

## 5.3 Frozen row types (Layers 1–2 — CONFIG ENGINE only; SPEC here)

- **dictionary row (L1):** `{hyperpoint_id, lemma, pos, forms[],
  synset_id, version_tag, provenance: {source:"wordnet", license}}`.
  IMMUTABLE: re-versioned, never edited.
- **relation row (L2):** `{a: hyperpoint_id, b: hyperpoint_id, edge_kind,
  weight, version_tag, provenance}` — e.g. `Heat --associates--> feeling`,
  `Heat --is-a--> noun`. Grammar lives here as correlations (noun-ness,
  number, tense) — **not** in a third structure.
- **rule row (L2):** `{triggers: [hyperpoint_id…], operator, results:
  [hyperpoint_id…]}` — e.g. `Heat + Ice → Water, Melt, State-Change`.
  Rules are **composable**: one rule's output hyperpoint can satisfy
  another's trigger (v1: depth cap = 1 composition; cycle checks are
  load-time, CONFIG ENGINE).

**THE CRITICAL LAW:** Layer-2 edges are precomputed structural/statistical
correlations — the genome, like the dictionary. **This is NOT SI's
knowledge.** SI's own experiential connections never live in L2; they are L3
rows in `links`/`facts`. An answer drawn from L1/L2 is tagged **BASE** and
**capped at INFERENCE** (genome paths are the dictionary talking — §9,
OD-04 resolved); an answer drawn from L3 is tagged **OWNED** and may claim
FACT. They disagree ⇒ **OWNED wins at NOW** and the disagreement is
**reported, never hidden** (both citations in the output line,
[docs/09](09-claim-labels.md)).

**The web is walkable:** every `z` edge — frozen genome edge or live OWNED
edge — is a propagation edge for the THINK walk
([docs/16](16-think-walk-hardware.md)). `/correct` events, answer-to-ask
auto-commits (§1.4), `/teach` commits (TEACH fragment — owner-authored content,
same channel class, no new edge kind) and S0 assertions create OWNED edges whose
weight starts from §8.4 confidence init and strengthens with
successful serves; supersession sets the aged edge's live weight to 0 (the
edge stays in storage, I4 — history walks can re-weight it at its
interval). CONFIG SI walks L3 only; the clean-room law (I2) is unchanged.

**EMOTIONAL CHARGE — a field on L3 edges only (§18.5, §20, I8).** Each live OWNED
edge carries `charge: [{signal, magnitude, a_at, thought_id, signal_ref}]` **and
one scalar `bake`** — **the signal-kind table of §18.1, never an emotion name**
(round 1 wrote `resident`; the field is superseded, **OD-23**). `magnitude` is
stamped at write/tuning time from the signal that fired at that write, **whose own
value is the magnitude**; `a_at` is **the quantized arousal level at formation**,
and `bake` is the durable head start that level bought:
**`bake = B_max · tanh(k · a_formation)` (§20.4a)** — **saturating**, so the
write side reads `a` at exactly one instant, through exactly one curve, and can
never produce an unbeatable edge (**OD-33**, which narrows round 2's "not a
function of `a`" law; `magnitude` itself stays signal-valued). `a_at` is also the
addressable teach-state §20.2's state-matching needs (**OD-28**; without it G11(a)
is unrunnable, and it is never a read-time multiplier). Round 1
also carried a `sign` field; with no residents there is nothing to avert
*from* (§18.5), so `sign` is **dropped from the v1 schema** and parked for v2
rather than defended (a real valence axis would resurrect it — **OD-31**, §20.6).
Five laws attached:
(a) **frozen layers are structurally incapable of holding charge** — the field
does not exist on L1/L2 row types, so a stamp on a genome edge is not merely
forbidden but unrepresentable; genome edges may still be *biased at runtime*
by the read-side term (§18.5), which touches no stored byte; (b) **charge is
form, not content** — the stamp names a signal kind and two scalars, never a
referent; (c) **the stamp and the bake it bought are permanent** — the edge's
formation-time head start and confidence bump are birth properties (§20.4:
*"that edge stays hot"*), and **the only thing that ages is a stamp's
read-side weight inside the point's median** (§20.3). Round 2's phrasing "the
stamp is permanent, the boost decays" is **superseded by OD-27**; provenance
stays citable either way (I4); (d) **THE EMOTIONAL READING OF A POINT IS NOT
STORED ANYWHERE (§20.3).** The recency-weighted median is a **derived**
read-side value, computed inside a thought from rows that are themselves
append-only. It is not a field, not a shelf, not a cache that outlives its
thought — a persisted per-word mood value would be a **fixed emotional
address**, which is the exact thing round 3's principle forbids (**R-28**;
G11(b)(c) measure the statistic instead); (e) **permanence is always bounded.**
The one durable affective field in the substrate is the bake, and its scale
saturates (§20.4a): `B_max` is a law, not a tuning preference, because a
permanent quantity that could grow without limit would outvote every later
thought forever and the median would be decorative. The registry law is untouched:
plasticity added **no new structure**, only rules over existing rows.
G9(d) asserts zero charge writes anywhere but L3 after a long
session.

**IDLE THOUGHT WRITES NOTHING (I9, §19.4).** An ignition cascade may reset a
rehearsal curve, push an agenda item onto the `questions` shelf, and — if the
owner rules **OPEN-24** yes — re-stamp the edges it re-lights. It may **never**
create a row, change a confidence, or supersede anything: no sense event means
no teaching event, and "an idle walk can notice, never learn" is enforced by
the write gate having no call site inside `idle.py` (G10 fails if it grows one).
**Round 3 sharpens it (§20.4):** because idle thought **forms no edge**, it can
create **no new bake** — an idle re-light may add a stamp (which moves the
point's *median*) but can never move the head start or the confidence a real
teaching event gave that edge at its birth. Rehearsal changes what a word
currently reads as; it cannot re-traumatize an edge that already exists.

**THE SUPPORTER WRITES NO BELIEF (§21.4, I10).** The same argument that keeps
idle thought out of the shelves applies to the one new thing in the room: **no
teaching event, no row.** The `source` tag sits on the **sense record**
(`context`, which every admitted line gets — including his, because replay reads
it, §21.7) and the belief rows carry `provenance.speaker`, which **may only ever
equal `owner`**. That is G12(e)'s whole assertion, run over a full session: one
row with `speaker: grimm` means content traces to a pretrained model and the
thesis is false — so the field is load-bearing now, not decorative as it was when
only the owner could type. `context` remains the substrate's control group for
the older law too: it has **no charge field**, so a Grimm line can neither become
a belief nor have affect stamped onto it as evidence.

## 5.4 Live row schemas v1 (L3)

```jsonc
// S-R row (shelf s_r_pairs) — the stage-0 atom
{
  "id": "r1",
  "address": [5, "s_r_pairs", "r1", []],        // (t, x, y, z)
  "stimulus": {
    "pattern": "P_QUEST_A",                     // matched R3 pattern id
    "signature": "what color be apple ?",       // normalized: folded, lowercased
    "terms": ["t_color", "t_apple"],
    "surface": "what color is apple?"
  },
  "response": { "terms": ["t_red"], "surface": "Red" },
  "bind_output_id": "o1",                       // the ask-back output it fills — whole output, v1
  "provenance": { "session": "sess-0001", "step": 5, "speaker": "owner",
                  "text_ref": "red.",
                  "kind": "correction|statement|reparse|answer-to-ask|owner-teach",
                  "question_id": "q1" },        // set when kind=answer-to-ask (§17.2.3)
                                                  // `speaker` is LOAD-BEARING since the
                                                  // supporter batch: belief rows may only
                                                  // ever carry "owner" — G12(e) asserts it
                                                  // across a whole session (§21.4)
  "time": { "written_at": 5, "last_confirmed_at": 9,
            "superseded_at": null, "superseded_by": null },
  "confidence": 0.95,
  "bake": "UNMEASURED",                          // §20.4a: B_max·tanh(k·a_formation),
                                                 // L3 only, permanent, bounded (OD-33)
  "charge": [                                  // §18.5 + §20.2 write-side stamp, L3 only
    { "signal": "novelty-spike", "magnitude": "UNMEASURED", "a_at": "UNMEASURED",
      "thought_id": "th-5", "signal_ref": "surprise@6.2=1.0" },
    { "signal": "open-question-age", "magnitude": "UNMEASURED", "a_at": "UNMEASURED",
      "thought_id": "th-5", "signal_ref": "queue_age(q1)" }
  ],
  "repeats": 1,
  "derived_from": [],                            // reparse lineage (fact rows only)
  "gain_exact": 1,                               // correction units, §8.2
  "gain_kind": "demonstrated",                   // §8.2, decided once at commit from
                                                 //   provenance.kind. "asserted" rows (a
                                                 //   /teach, a plain T2 statement, T1, a
                                                 //   reparse view) start with gain_exact 0:
                                                 //   no error-before existed to subtract.
                                                 //   The first non-corrected serve promotes
                                                 //   asserted → demonstrated and logs the
                                                 //   serving step id (§8.3)
  "version": "row-v1"
}

// Fact row (shelf facts) — derived by RE-PARSE from S-R rows at S1+
//   same envelope; key {entity_term, field: pattern-slot|linked-term},
//   value {terms, surface}, derived_from: [sr_row_ids], rest identical.
//   RE-PARSE copies charge as HISTORY (the derived edge inherits the source
//   rows' stamps as provenance, never as a fresh boost — §18.6).
// Bridge row (shelf links) — {term_a, term_b, kind: means|same-question|
//   slot-unique, provenance, confidence, charge}.
// Question row (shelf questions) — {kind, origin: sense|idle, target,
//   asked_at, asked_by_thought, resolved_by, status, arousal_at_ask}
//   — `arousal_at_ask` is a **quantized snapshot of `a` kept as audit
//   evidence, not as a cause** (§18.2/OD-22: post-round-2 nothing gates the
//   budget on a level, so the ask is reconstructible from §17's policy + the
//   queue rule, and `a` explains only how the walk was spread when the gap
//   surfaced). `origin: idle` marks the agenda items an idle cascade pushed
//   without ever emitting them (§19.4) — the shelf doubles as the **idle
//   agenda** §19.1 checks before igniting, which is why idle thought can be
//   interruptible by a real question without a scheduler.
// Context row (shelf context) — verbatim utterance + address + `source:
//   owner|grimm` (§21.4), never used
//   for FACT answers, only quoted. NO charge field (it stores raw input,
//   not a belief) — stamping raw utterances would leak affect into evidence.
// Teach row (shelf s_r_pairs, provenance.kind = owner-teach) — NOT a new row
//   type (§5.1 registry law: a new knowledge type is a new shelf, and this one
//   needs none). It is an S-R row whose `stimulus` is the DECODED left side of
//   `/teach <pattern> => <response>` — pattern id + normalized signature +
//   terms, never the raw text (§1.4) — whose `response` is the right side, with
//   `bind_output_id: null` (no ask-back output precedes the act, so there is
//   nothing to bind) and `gain_kind: "asserted"` until a later recall actually
//   serves from it (§8.2).
```

The common envelope (provenance / time / confidence / version /
bind_output_id / gain_exact / **gain_kind** / **charge**) is shared across
belief-bearing live row types so every row is citable-audit in one mechanism.
`gain_kind` joined the envelope with the TEACH fragment, but it is not a
`/teach`-only field: every row already had one of the two values, and the
ledger only refused to name the difference before an owner-authored channel
made it the common case (§8.2).
Rows that
carry no charge (`context`) are the audit's control group: if a charge value
appears where the schema says none exists, that is a frozen-layer violation
attempt one shelf earlier than G9(d) looks for.
