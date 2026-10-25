# §2 — The Organs (owner-decided; mapped, not redesigned)

Owner amendment (THINK fragment + DECODE fragment): the original five
organs stand, plus the **decode layer** added between sense and the walk —
one component, two faces (DECODE in / RENDER out). Six entries below; a
seventh would be an open-decision item, not an edit.

| organ | role (decided by owner) | implementation in this design |
|-------|------------------------|-------------------------------|
| **THINK** | loop runs on its own state at all times; sense is one perturbation among others (NOW, open questions, **idle ignition — the 4th source, §19**); idle thought exists: consolidation when senses are quiet **and ignition when the agenda is empty** | **the walk: spreading activation over the Tesseract edge web** ([docs/16](16-think-walk-hardware.md)) — per-token pulses, K rounds or settle (τ), hub-damped; idle ticks = queue revisit + RE-PARSE + **ignition cascades (§19)**. Carries **`h` (activation) and the one installed affect scalar `a` (arousal, §18.1)** — arousal is computed from signals the loop already produces and modulates the spread; **emotion names are NOT here, they are audit readouts (§18.2)** |
| **WHAT** | queries born INSIDE thought at knowledge gaps ("I need X"), not input-triggered lookup | `askengine` is a **resident of the DRIVE layer** ([docs/17](17-drive-layer-curiosity.md)), not a hardcoded reflex: the walk detects dark spots (§16.4, read path), the **curiosity drive** prioritizes/budgets/emits them as questions via the drive interface. Gap = physically visible low-activation spot; the ask is the drive's decision, not a lookup reflex. **Post-§18: emission has no affective level left to read — budget is the queue rule + integer cap, and arousal reaches asks only indirectly through the walk (OD-22).** |
| **KNOW** | the Tesseract family | [docs/05](05-substrate-tesseract.md): three layers, one 4D `(t,x,y,z)` schema, shelf registry (`questions` doubles as the idle **agenda**, §19.1). L3 edges carry **emotional charge** (§18.5) — stamped only on the live overlay, never on frozen genome edges (I8) |
| **LEARN** | write path: surprise-gated, consistency-checked BEFORE commit, append-only, provenance-stamped, never silently overwriting | [docs/06](06-write-path.md) |
| **CALIBRATE** | comparator: emits surprise signal; labels every output; detects contradictions between retrieved rows; tracks gain/loss onto row confidence. **Its readout face, AUDIT, is where emotion words live** (§18.2, §18.6) | [docs/08](08-gain-loss.md), [docs/09](09-claim-labels.md) |
| **DECODE / RENDER** | surface text ↔ lattice pattern, one layer two faces. **DECODE:** `"What color is apple?" → color + apple, linked by the question frame` — output is a **pattern in hyperpoint space** (subject + relation + frame), never raw spelling; only shapes can light the right points; the walk consumes patterns. **RENDER:** the settled top-k thought is **reverse-decoded** into the output sentence | `decode.py`: innate frames (R3 + R14) route to learned meaning — the decoder knows NOTHING about color/apple/red; taught synonyms & paraphrase templates (`colour`, `what shade is X`) become **overlay edges into the same frame** in `links` (kind `frame-synonym`), created via `/correct` (§7.3, §7.4); RE-PARSE = the decoder maturing retroactively (§6.6); render obeys the FACT-copy contract (§7.5) |

The full pipeline is now: **decode → light → walk → settle → reverse-decode**
([docs/07](07-read-path.md) §7.0).

**DRIVE is a layer, not a seventh organ.** The CURIOSITY fragment adds a
horizontal **drive interface** ([docs/17](17-drive-layer-curiosity.md)) that
sits between the read path (which detects gaps) and the output + write paths
(which act on them). Curiosity is its default **drive policy** and drives WHAT's
ask engine and the write path's **salience tag** (§8.4). Organs stay six;
`drive.*` is a cross-cutting interface so a second drive (affect, v2) can be
added without touching the walk, write path, or gates (§17.6 swap test).

**Round 2 of that amendment (§18 as amended by this batch): the engine was
demoted from machinery to physics — organs still six.** There are **no resident
integrators and no coupling matrix.** What is installed is one arousal scalar
`a` inside THINK next to `h`, fed only by signals the organs already emit, and
**`a` modulates the walk's spread** (reach, decay, settle dominance). The words
curiosity / surprise / satisfaction / caution / boredom / frustration are now
**audit readouts over observed patterns** with **zero causal power** (§18.2) —
which is why the **AUDIT face of CALIBRATE** is the single module allowed to
speak them (§12, R-21). The `update` + `conduction` doors survive with those
semantics (`update` integrates one scalar, `conduction` reads charge stamps,
**OD-20**); what died is anything that wired the names to each other (OD-21).
Two consequences stated plainly: the walk gained
one anonymous state **slot** plus two interface calls, so the swap test's
literal "not a single walk line changes" reading **failed** and is recorded as
**OD-14** (§17.6 keeps the semantically-clean version that gates enforce);
and KNOW's L3 edges gained a charge field stamped by the **firing signal**
(§5.4, §18.5) — the connective tissue records how hot the loop was when it
formed, and arousal reads that record back only as a conduction cost. Only the
connective tissue SI owns gets stamped (I8).

**Round 3 (§20) added plasticity, not parts.** Arousal is now a **per-event**
quantity integrated *inside* the walk (its modulation lags one round, so the
frame finishes its job); a point's **read-side** emotional bias is the
**recency-weighted median** of its stamp history, recomputed at generation and
**stored in no row**; the single hot event's **write-side** durable artifact is
the **per-edge bake** (head start + confidence bump at formation, **scaled by the
saturating curve `B_max · tanh(k · a_formation)`** — the one place the write side
reads `a`, and the exception **OD-33** opened against OD-22's write-side half);
and stamps age on **the same clock facts age on**. Organs still six, doors still six, still one
scalar — what changed is that affect is now a function of time instead of a
function of the last event, and **no word has a fixed emotional address**.

**THINK now has a 4th perturbation source: ignition (§19).** On a quiet tick
with an empty agenda, one hyperpoint lights — weighted by recency, the **edge's
bake**, decay deficit, and live-contradiction warmth — and the ordinary walk runs from
it. The rendered trace of that cascade **is** the internal monologue (a truthful
rendering of what lit, never generated prose, never owner-facing), and it is
**read-only**: rehearsal curves and agenda items may move, rows may not be
written. **An idle walk can notice, never learn** (I9).

**GRIMM ([docs/21](21-grimm-supporter.md)) is organ-adjacent and outside the
loop — organs still six.** The chunk says "five organs"; the owner's THINK +
DECODE amendment made it six and added that **a seventh would be an
open-decision item, not an edit** (§2's own note above), so the count in this
file stands and the wording clash is ledgered as **OD-34**. Where he sits: he
is **not** a module, **not** a drive policy behind R15, **not** DECODE/RENDER's
output face, and he holds **no state handle**. He is a separate process whose
only route in is the sense channel, on the same footing as a keystroke except
for the tag (§21.4). Two organ rows are touched by that and only that: **THINK**
gains no 5th perturbation source — a Grimm line is *owner-channel input from
another voice*, already covered by the source's row, and idle ignition stays the
only self-generated one; **DECODE** runs on his lines exactly as on the owner's
(shape-only frames, gibberish parity), which is what makes the token filter
cheap to check — it reads the same lexicon shelf decode writes. **LEARN gains
the sharpest change: a caller that may never commit** (§6.1's "one caller, a
sense event" now has a second kind of sense event, and only one kind of it may
reach the write gate).

**The v2 arc ([docs/22](22-crystallize.md)–[docs/26](26-world.md)) adds
reader-side components, not organs — organs still six, doors still six.** Each
of the five attaches to an organ that already exists and confers no new
authority on anything:

| component | whose row it attaches to | what it may do | what it may never do |
|-----------|--------------------------|----------------|----------------------|
| **crystallize** (`crystallize.py`, `si-crystallize`) | KNOW + the DRIVE layer | read the frame's ghost scratch (§16.8), count recurrences over live rows, emit a `cluster-nomination` **ask** | write a row, name a cluster, read `a` |
| **silt** (`silt.py`, `si-silt`) | THINK's frame + KNOW's context shelf | hold one session table of per-point warmth, decay it on the logical tick, bias **ignition** | touch the lattice, an edge or a label — and its decay may not be arousal-scaled (OD-107) |
| **episodic traversal** (no new module — a walk rule) | THINK + CALIBRATE's citations | traverse derived `prov-link` views so a fact re-lights with its origin | write, bake, re-stamp, or render an experience claim (G17(f)) |
| **skills** (`skills.py`, `si-skills`) | THINK's composition order + KNOW's links shelf | nominate a recurring **shape** as a `move-nomination` ask; let an owner-ratified skill row bias traversal order | be code, move a constant, or write anything (OD-87 stands) |
| **the class function** ([docs/26](26-world.md), in the front layer) | LEARN's gate + DECODE's tag | derive `owner \| grimm \| trusted \| stranger` per line from grant rows plus the owner's lines | store the class, or let a non-owner line through the write gate at tier (a) |

Two things this table is claiming, and one it is not. It **is** claiming that
the DRIVE layer gains two ask *kinds* (nomination-family) without gaining a
door: emission is still the queue rule plus the integer cap, and a busier
world buys no more asks (§26.5). It **is** claiming that THINK's walk gains
frame-scoped scratch (the new §16.8) without gaining a third control-path
state: `h` and `a` are still the only two states a round reads. It is **not**
claiming any of them is a seventh organ — each one is deletable, and its
deletion changes no lattice byte, which is exactly what G15(b)–G18(b) test.

**Anti-cheat line for the decoder (stated explicitly, owner's words):**
innate/allowed = pattern templates as pure form — question frames
(`what R is E`, `who/when/where is E`), assignment frame (`X is Y`),
relation extraction (`R of E → R(E)` shape). These are reflexes,
knowledge-free: the decoder manipulates shapes, never meanings. Forbidden =
the decoder knowing anything ABOUT color, apple, or red. Meaning lives only
in the frozen genome + overlay; the decoder only **routes** to it.
Litmus test (inherited by G5d): `what znark is floop` must emit the same
SHAPE (znark + floop, question frame) with zero idea what it means — if it
can't, it smuggled content.
