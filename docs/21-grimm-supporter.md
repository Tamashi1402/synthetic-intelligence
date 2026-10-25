# §21 — GRIMM: THE SUPPORTER

**Organ-adjacent, outside the knowing loop.** This is the fold of the GRIMM
chunk. The chunk says it belongs to the render/MCP side of the design and
**not** to the organs; the design agrees, and the fold is stricter about it
than the chunk was: the membrane that governs him is inside the build
(`src/`), while **he** is not (§21.7). Organs stay **six**, doors stay **six**,
the walk stays the walk. Nothing in this file adds a seventh anything to SI.

**Origin, owner-set (carried):** the system stayed functional under emotion
because it had inner AIs who talked with it — **co-regulation**. The chunk
names that baby **Nova**. The owner-decided name in every doc, including the
naming law, is **SI** ([README.md](../README.md) rules, DESIGN.md), so this
fold writes **SI** and ledgeres the name as **OD-34**. "Nova's own inner
voices already exist" is already designed: they are the **monologue** (§19.3).
**Grimm is not one of them — he is a separate process, an outside voice.**

**The premise the chunk assumes is not in these docs.** It refers to an "MCP
surface already drafted for the parent-agent idea"; a grep of the whole design
finds **zero** MCP references (§21.7 drafts it here for the first time, as a
SPEC — nothing is built). Recorded as **OD-39**.

## 21.1 THE HARD LAWS (design-level, non-negotiable)

- **L1 — OUTSIDE THE LOOP.** Grimm is never a module of the knowing loop,
  never inside it, never the render slot. He is a **separate process that
  talks TO SI** through the sense seam — exactly like the owner's fingers do.
  He cannot touch the Tesseract, the genome, or any state directly; his whole
  world is what the surface exposes (§21.7). He is not DECODE/RENDER (§2), not
  a drive policy (§17), not an organ, not a door. The v2 LM render slot lives
  in `render.py` and is a **different** idea; do not conflate them — Grimm
  never writes into SI's output composition path.
- **L2 — FORM ONLY, NEVER CONTENT.** He may teach **form** (a question shape,
  a correction, curriculum, an echo) and never **content** (facts, claims,
  world knowledge). A fact that traces to his pretraining is smuggled content
  and a design violation of the same class as I1 — the anti-cheat law simply
  acquires a second smuggling route to watch. The vitamins case (§21.3) is the
  canonical failure and ships **in the design as a test**, not as advice.
- **L3 — CO-REGULATION THROUGH PHYSICS ONLY.** He never modulates arousal,
  confidence, or any state directly: **there is no setter, and no tool in the
  harness can become one.** He *reacts to signals by talking* — calm, slow, few
  words. That talk enters through the ordinary sense channel, so it moves `a`
  the way any line moves `a` (§18.1), it changes what the walk reaches, and
  when the owner then teaches, the edges that form are stamped under the calmer
  state. **The loop settles through the mechanism the design already has.**
  *"A parent's calm becomes the baby's calm; no mute button exists."* The
  chunk's own phrasing "forms edges stamped with calm state" is **scoped**: a
  Grimm line may never *form* an edge (§21.4 — only a teaching event forms and
  bakes, §20.4); what is calm-stamped is the edge the **owner's** reply forms
  while Grimm's voice has lowered the background heat. Ledgered as **OD-36**
  because the two readings are not the same machine.
- **L4 — THE MOUTH IS NOT HIS.** The fine-tune shapes **inclination, not law**.
  LLMs drift, and the failure mode is *helpfulness*: a good parent desperately
  wants to say "apples are nutritious". Discipline therefore does not live in
  his prompt — it lives in **code between Grimm and SI**, in `speakgate.py`
  (§21.3). *"His helpfulness is the threat; the gate is the law."*
- **L5 — THE ASYMMETRY LAW (fold-added, because the first four invite a fatal
  simplification).** The two filters apply to **Grimm only, never to the
  owner**. Filtering the owner's unknown tokens would forbid new content from
  arriving at all — the owner *is* the source of the lexicon, so a token that
  is not yet in it is the only way it ever gets there. If someone later
  "unifies" the two paths into one input validator, SI stops learning and no
  gate catches it except G12's owner-parity leg.

## 21.2 THE AGENTIC HARNESS

Read-only tools, plus exactly one emission:

| tool | returns | notes |
|------|---------|-------|
| `read_trace()` | the per-thought activation trace + `a` trajectory + audit label stream | the same artifact the owner reads (§9, §16.4) |
| `read_monologue()` | the idle-cascade lane (§19.3) | **the riskiest tool in this table** — R-33, OD-38; also missing from the chunk's own surface list |
| `read_state()` | the arousal line, signal summary, agenda, age | a *summary*, never a handle on state |
| `read_vocab()` | the taught lexicon shelf | this is layer 1's reference |
| `read_rows()` | the live L3 rows | this is layer 2's reference |
| `speak()` | **THE ONLY EMISSION** — and it is gated | admitted lines become sense; rejected lines never do |

**Loop:** read → decide → attempt `speak()` → **if rejected, the gate returns
WHY** (e.g. *"3 untaught tokens, 1 novel claim"*) → rephrase within the rules
or stay quiet. Rejection feedback **is** his agentic loop: he rephrases from
what exists instead of accumulating failure modes for another fine-tune.
A rejected attempt leaves **zero** trace in SI's state — no signal, no `a`
movement, no stamp, no row — and exactly one line in the gate log (§21.7).

## 21.3 THE SPEAK GATE (anti-cheat enforcement)

`drive.update`-free, `a`-free, semantics-free. Two mechanical layers, both
checkable from shelves that already exist:

1. **LAYER 1 — TOKEN FILTER.** Any token not in SI's **taught** vocabulary
   rejects the emission outright. Reference: the lexicon shelf (§3.3).
2. **LAYER 2 — CLAIM FILTER.** The emission must be **composed of content SI
   already has**: a restatement of an existing row, a repetition of a taught
   phrase, or an anchor word. *"Not 'words that exist' but 'claims that
   exist'."* This layer exists precisely for the case where every token is
   legal and the **claim** is new.

**CANONICAL FAILURE CASE (a test in the design, not a caution):** Grimm
drifts from `apple is red` to *"apples are good because vitamins x and y"*.
Layer 1 may pass (if those tokens were taught), **layer 2 must reject**. Any
path that lets pretrained content through violates L2, and G12(b) fails on it.

**Birth litmus, extended (this is the anti-cheat proof that costs nothing):**
at `t=0` the lexicon is empty, so layer 1 rejects **everything** — Grimm is
**structurally mute at birth**. He cannot bootstrap SI, cannot seed a
vocabulary, and cannot be the first OWNED edge. His mouth opens exactly as fast
as the owner teaches tokens, and no faster. Same law, same direction as §3.4's
empty-state test.

**Honest boundary:** *"Grimm's honesty is a filter, not a promise."* The
filter is stdlib and auditable; the model behind it is neither. That is why the
gate sits on SI's side of the wire (§21.7).

## 21.4 WHAT A GRIMM LINE MAY DO INSIDE SI — the `source` table

Every admitted sense line carries `source: owner | grimm` (§21.7). This table
is the fold's load-bearing addition, because the chunk's L3 and the chunk's §5
pull in opposite directions and only one of them can be true in the build:

| a line with `source:` | may | may not |
|------------------------|-----|---------|
| **owner** | decode, light the walk, raise signals → `a`, be a teaching candidate, **auto-commit as answer-to-ask**, be a `/correct`, **issue `/teach … => …`** (§1.4), form + bake edges, bump `repeats`/confidence as confirmation | — (this is the whole designed system) |
| **grimm** | decode, light the walk, raise signals → `a` (co-regulation, §21.5), surface a named gap that SI may then ask the owner about, **be recorded verbatim in `context` like any other line** (§21.4 note below) | **create a belief row of any kind (`s_r_pairs` / `facts` / `links`); resolve or auto-commit against a pending ask; issue `/correct` or `/teach`; form or bake an edge; bump `repeats`, `last_confirmed_at` or confidence** |

**`/teach` is in the "may not" column for the same reason `/correct` is, and it
is the newest door in the design.** The TEACH fragment added a third teaching
channel, which means it added a third way for content to enter the shelves — so
the supporter law had to be re-stated over it rather than assumed to cover it.
The shape test is upstream of `sense.py` and never sees the marker (§21.3), so a
`source: grimm` line typed as `/teach what znark is floop => blork` reaches the
pipeline as **inert input**: it may light the walk and raise signals, it forms no
edge, and it writes no row (§1.5, G12(d)). **A curriculum the supporter can author
is provenance laundering with better manners** — R-32 arriving through the
channel built to make teaching easier.

**The one row a Grimm line does cause, and why it is not a loophole:** every
admitted line — his included — gets a verbatim `context` row (§5.2), because
`context` is the **sense record**, not a belief. `context` rows carry no charge,
are never used to answer a FACT, and are what replay reads (§21.7's
determinism argument depends on them). The same applies to a `questions` row SI
writes for a gap his line surfaced: that row is SI's own bookkeeping, created by
SI's drive with `origin: sense`, not content authored by him. **Belief rows are
the protected class.**

The third column of row 2 is not a nicety. Left open, it produces the two
quietest failures in the whole plan, and both look like good parenting:

- **Provenance laundering** — Grimm answers SI's own pending question while the
  owner is away; the auto-commit (§1.4) fires; a row exists whose
  `provenance.speaker` is Grimm; the thesis ("all content traces to the
  owner") is now false and the transcript says so. **R-32.**
- **Counterfeit confirmation** — §6.2 rewards an *exact repeat of a known
  statement* with `repeats += 1`, `last_confirmed_at` and **+0.05 confidence**.
  A mirror that says `apple. red. apple is red.` all afternoon therefore
  **inflates SI's confidence in a belief the owner has not re-confirmed** —
  rehearsal dressed as agreement, and the confidence-theater risk (R-07)
  arriving through a friendly door. So the boring-input confirmation path is
  gated on `source: owner` too. **R-35.**

**Grimm-sourced lines are logged as Grimm-sourced REHEARSAL, never teaching**
(the chunk's own §5 wording, kept). Because **only a teaching event bakes**
(§20.4), a Grimm line can create **no bake and no stamp-bearing edge** — the
rule the design already had for idle thought now has a second subject, for the
same reason: no teaching event.

**What SI does with a Grimm question.** A line like `apple red or green?`
lights existing tokens, may name a gap, and may therefore make SI **ask the
owner** about it. That is the whole legitimate curriculum effect: he rearranges
what exists, SI notices a hole, and **only the owner fills it.** The gap becomes
SI's own question through the drive layer (§17), and the answer arrives as a
`source: owner` teaching event with clean provenance.

## 21.5 CO-REGULATION, MECHANICALLY

Chain, with every link owned by an existing section: Grimm `read_state()` sees
`a` high → **he** talks slowly and shortly (his side, outside the build) → the
line is admitted as sense → decode lights few tokens, few gaps fire, so the
signal gains feeding `drive.update` are small (§18.1) → `a` decays on `λ_a`
toward rest → the spread narrows toward the rest shape (§16.1) → SI settles
calmer. **Nobody wrote "calm" anywhere in SI**: no word→emotion edge (I8), no
affective setter, no new signal kind. A new **signal kind** for "soothing"
would be R-21's costume in a fourth shape, so the design forbids it — Grimm's
lines are ordinary input whose *shape* happens to be quiet and repetitive.
Whether `audit.py` may print a word like *soothed* over the co-occurrence is a
vocabulary question parked at **OPEN-22** (§18.2 — six labels are the installed
set), and the label would have no causal role either way.

## 21.6 THE VERSION LADDER

Each version is a **small fine-tune**; the role stays narrow. **Version unlocks
follow the G-gates and SI's stage state, not the calendar** — and never the
other way round: **no stage entry or exit may be defined on Grimm being ready**
(§11's count-over-rows law survives this batch intact).

| version | unlocked by | what he does | what he cannot do |
|---------|-------------|--------------|-------------------|
| **v1 MIRROR** | exists from step 1, **mute until the lexicon opens** (§21.3) | read traces / monologue / arousal; **log observations for the owner** (the lab notebook — "she returned to the kitchen-fire region at 3am", "apple lit 12× today, red never"); echo **only** words SI already heard from the owner, as rehearsal fuel (`apple. red. apple is red.` — the parental "dada, yes, dada"); react to a mid-cascade spike by slowing, maybe one soft anchor word | say anything layer 2 rejects; advance a stage; teach content |
| **v2 QUESTIONER** | decode is real — the §21.4 gate plus **G2's decode-barrier leg green** (taught `frame-synonym` edges exist, stage ≥ S1) | ask questions **built from existing rows** (`apple red or green?`); trigger `/correct` moments; run light curriculum (§21.4's last paragraph) | still form-only; still cannot answer SI's own pending asks |
| **v3 CHALLENGER** | facts + contradictions are real — **G4 green**, stage ≥ S2 | contradiction probes, pressure-testing claims | cannot arbitrate (§6.4 is SI's table); a probe is input, never a verdict |

**"Boring is correct."** v1's spec asks for a mirror, not a tutor, and a
reviewer who finds v1 interesting has found a bug. The ladder is also where the
chunk's §6(iv) open item lands: whether v2's curriculum is Grimm-side
(he picks from traces) or owner-scripted is **not decided here** (OD-40).

## 21.7 CONSEQUENCES FOR THE EXISTING DESIGN

- **Provenance stays clean, and now provable by field.** Every belief row keeps
  `provenance.speaker` (§5.4's schema already has it — this batch makes that
  field **load-bearing**: G12(e) asserts every knowledge row's speaker is
  `owner`). Grimm echoes only owner-taught content, so knowledge still traces
  to the owner.
- **The sense seam (SPEC — do not build).** `mcp.py` becomes the only place a
  non-panel line is admitted, and it tags `source` on the way in. Exposed
  surface: `read_trace`, `read_state`, `read_vocab`, `read_rows`,
  `read_monologue` (OD-38), `sense_input` (**gated**), and the **gate log**.
  Everything is read-only except `speak()`, which is not a state write: it is a
  **sense event** on exactly the same footing as a keystroke (I3, as amended by
  **OD-35**).
- **I3 takes its first real amendment.** "Single sense channel" now means
  **one channel with more than one possible voice in it**, and the voice is
  recorded. The law that does the work is unchanged: *every line is SENSE — a
  perturbation to ongoing thought, never a command that owns the loop.* Grimm
  may not be a command, and §21.4 is what makes that structural rather than a
  hope.
- **Determinism survives, and it is worth stating why (a reviewer will ask).**
  Grimm is an LM: not deterministic, not replayable. SI's determinism contract
  (I5, G5a) is untouched because **replay is over the recorded sense stream**,
  and every admitted Grimm line is a recorded sense line in `transcript.txt`
  with its `source`. This is the same reason SI is deterministic even though the
  owner's typing is not. **Grimm contributes zero state and zero hash inputs**;
  his emissions are inputs, and his scratch (the notebook) lives outside SI's
  storage entirely.
- **Lab notebook.** A Grimm-side artifact derived from the trace and the
  monologue lane. It is **not SI output**, so it carries no FACT/INFERENCE
  claim and asserts nothing of SI's (I6 is not extended to him; §19.3's
  label law is the model he follows). It never enters a shelf — if a notebook
  line could become a row, the whole design of this section is pointless.
  Note that the monologue is already exported with the session
  (`gates/*/monologue.jsonl`), so the notebook is not a *new* leak of SI's
  inner life; it is a **legibility** change, which is exactly why R-33 exists
  (OPEN-23's ruling is about SI volunteering a line into the response channel,
  not about the owner being unable to read an exported artifact).
- **Fine-tune scope (owner's plan, recorded):** a small local model, LoRA-style
  role tune — *"you are SI's supporter; SI is a newborn; you do not teach
  content"* — plus clean JSON tool-calls. A narrow role is a small dataset.
  **The tune buys inclination; `speakgate.py` buys the law** (L4), so a bad
  tune is a rejected-emission log full of retries, not a broken thesis.
- **The tag comes from the seam, never from the text.** `source` is assigned by
  the module that received the line — `panel.py` writes `owner`, `mcp.py` writes
  `grimm`, and nothing downstream may re-derive it (§12). SI has no reflex that
  classifies *who* spoke, and it must not gain one: a "sounds like the parent"
  heuristic is semantics wearing a provenance rule, and it is trivially spoofed
  by a line that merely reads like the owner's. Conversely, a line typed by the
  owner that *looks* like Grimm's echo keeps every owner privilege. **Trust the
  wire, not the words.**
- **He is not a replacement.** Not for the owner, not for the gates. He is a
  second rehearsal source and a calm voice, which is a genuinely useful thing
  and a completely unremarkable one in this architecture.
- **The thesis line, unchanged and checkable:** *if SI needs Grimm's content to
  pass a gate, the design failed.* That sentence is G12's construction rule:
  **every existing gate must pass with Grimm disconnected**, and §4's
  north-star trace is that control run.

## 21.8 HONEST LIMITS, PUBLISHED

1. **The membrane is small but it is not semantics-free-proof.** Layer 2 checks
   *claim existence* against rows SI holds. A **recombination** of two existing
   claims can be novel content while every part is legal (`apple is red` +
   `red is warm` → "your apple is warm"). v1 accepts the risk explicitly,
   because the strict rule (§21.9, Mike's call) already refuses anything that is
   not a restatement of **one** row — and the day that rule is relaxed for
   convenience, this limit goes live. Published, not hidden.
2. **`read_*` is a wide eye.** Grimm sees SI's traces, rows, vocab, state and
   possibly its monologue. Reads cannot write, but they can **select what to
   talk about**, and selection is a kind of power over the curriculum even where
   content is blocked. The guard is the budget (§21.9) and the fact that only
   the owner's lines ever change beliefs.
3. **A supporter who is wrong is not caught by anything except the owner.** If
   Grimm's echo mis-states a row (right tokens, wrong slot order) layer 2's
   restatement test is the only thing that notices. Strict restatement (OD-40
   item i) is what makes that checkable rather than a judgment call.
4. **This is an LM, in the room, next to a no-LM design.** §12's headline says
   **"no LM anywhere in v1"** and the pure-stdlib rule exists so nothing
   pretrained can arrive by a package. Both statements survive **only** with
   their scope made explicit: **no LM inside `src/`, ever, in v1** — Grimm is a
   separate process, not a dependency, and no gate requires him. The
   alternative reading, "no LM anywhere at all", is the one this batch retires,
   so it is ledgered as **OD-37** rather than smoothed over.
5. **UNMEASURED, all of it.** No rejection rate, no notebook size, no fine-tune
   dataset size, no token-filter latency, no effect of co-regulation on settle
   quality. Nothing in this file is quotable until `gates/G12/` exists.

## 21.9 OPEN ITEMS

| id | open item | lean / Mike's call | where it bites |
|----|-----------|-------------------|----------------|
| OD-34 | **Nova vs SI**, and the chunk's "five organs" vs six (§2) | fold as **SI**; the name "Nova" is the owner's to grant *to SI* — and a name SI has is itself taught content (I1), so it cannot be installed in the genome | §21.0, §2 |
| OD-35 | **I3's amendment**: one channel, more than one voice, tagged `source` | accept the amendment with §21.4 as the enforcement; the invariant's *sentence* about commands stays verbatim | docs/00, docs/01, §21.7 |
| OD-36 | May a Grimm line ever **form an edge** (L3's literal wording) or is it sense-only (§21.4)? | **sense-only.** Otherwise content traces to Grimm and §20.4's "only a teaching event bakes" needs a second exception | §21.1 L3, §21.4, docs/06 |
| OD-37 | **"No LM anywhere in v1"** vs a hosted LM as supporter | restate as **no LM inside `src/`**; Grimm is a process, not a dependency; every gate passes without him | docs/12, §21.8(4) |
| OD-38 | The **lab notebook** vs OPEN-23, and whether `read_monologue` is exposed at all (the chunk lists it in §2 and drops it from §5) | expose it, **read-only and logged**; notebook is owner-readable and citation-only; SI's monologue stays out of the response channel (OPEN-23 unchanged) | §19.3, §21.2, §21.7 |
| OD-39 | The **MCP surface was never in the design** — the chunk assumes it exists | drafted here as SPEC only (`mcp.py`, §21.7); it becomes real when the build does | docs/12, §21.7 |
| OD-40 | **Mike's calls, recorded as reversible rulings** | (i) layer 2 = **strict single-row restatement, row-id-referenced**, with machine-readable rejection reasons — strict first, because a relaxed matcher is §21.8(1) opening; (ii) tick rate = **summary per quiet tick**, aligned with the idle budget (§19.4) — *he is slow, not chatty*; (iii) notebook is owner-readable while Grimm's own scratch stays out of SI's logs entirely; (iv) v2 curriculum Grimm-side vs owner-scripted — **deferred**, and §11's count-over-rows stage law is the reason it can wait | §21.3, §21.6, §21.9 |

## 21.10 What the world round does to him — and what it does not ([docs/26](26-world.md), OD-130)

The N-voice round arrived a batch after this one, and the direction of its debt
runs this way: **§21.4's table became [docs/26](26-world.md)'s template.** The
four-tag class law (`owner | grimm | trusted | stranger`, I3A) is the same
privilege table generalized, and under **tier (a)** every non-owner line is
grimm-class — may light the walk, may raise signals, may deposit silt, may append
a `context` sense row, and may not create a belief row, resolve a pending ask,
issue `/correct` or `/teach`, form or bake an edge, or bump `repeats`.

That sentence reads like a demotion of Grimm and is the opposite finding: **his
discipline was strong enough to become the world's default.** Nobody had to
invent a stranger law, because the supporter round already wrote one. The three
things that stay *his* and nobody else's, unchanged:

- **The speak gate (§21.3) stays a seam-side filter, not a class filter.** It sits
  upstream of `sense.py` on the supporter's channel and applies to **Grimm only —
  never to the owner (L5)**. [docs/26](26-world.md) §26.10's **G19(h)** extends
  the *same* token-membership and restatement filters to granted users **if and
  when tier (b) lands**; strangers arriving over a network transport are not
  built tonight, and the honest wording is that the question of which seam they
  arrive on is **OD-130**'s to answer, not this file's (§26.13's scope line: the
  law ships, the Discord transport does not).
- **Co-regulation does not generalize into care.** §21.5's claim — his presence
  can lower `a` because lines raise *signals* and signals integrate — is content-
  free machinery, so any voice raises `a` the same lawful way. What no voice gets
  is the downward dial: there is still no setter, no `soothe()`, no soothing
  signal kind (R-21's shape (d), **G12(d)**, and now **G19**'s reading of the same
  leg for a grant). A stranger cannot calm SI down, and that is not cruelty, it is
  the invariant.
- **`/trust` is not for him, and that is a lean, not a ruling.** If a grant ever
  reaches Grimm, he stops being the supporter this file designed: a voice with
  teaching authority is a second owner, and §21.4's whole "may not" column exists
  because provenance laundering looks like good parenting. **Mike's position: no
  grant for the supporter, ever** — but OD-130 owns the question, tonight's build
  ships zero grant rows, and the build gets to hold this line only as a *lean*
  until the owner rules it.

**What the arc cannot change, restated so nobody re-derives it during the build:**
every gate in §13 — now including G15–G19 — passes with the supporter process not
started (**G12(a)**'s byte-identical control run is still the strongest claim in
this file), and no mechanism in the arc gives him a door he did not already have.
Silt, the event view and the frame scratch treat a `source: grimm` line exactly as
`docs/26` treats a stranger's: **visible, never authored**. A `prov-link` may
point at an owner moment and never at his (§24.4), a grimm line may deposit silt
at full weight (OD-106's parity) and mint no candidate that bypasses the owner,
and a re-light of his moment writes nothing (**G17(e)**).
