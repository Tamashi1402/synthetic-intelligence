# §12 — Component Map (v1 build plan — design only, do not build yet)

Standalone library; in-memory v1; CLI-testable; laptop-runnable; **no LM
anywhere in `src/`** — the supporter batch narrows the wording of this promise
without weakening what it protects (**OD-37**): there is now a locally hosted
LLM *next to* SI ([docs/21](21-grimm-supporter.md)), and it is **not a
dependency, not importable, and not required by any gate**. pure stdlib
(zero deps ⇒ nothing pretrained can sneak in
via a package) stays exactly true of the build. Planned `src/` layout mirrors
this table one module per
file. **`grimm/` is not in this table and not in `src/`** — it is an external
process, versioned and tuned separately, whose only contact with SI is the seam
below. *The gate is in the build; Grimm is not.*

| module | organ | innate/learned | responsibility |
|--------|-------|----------------|----------------|
| `panel.py` — chat REPL, CLI surface: `--config si|engine --session --replay --hash --export --gate N` | sense | — | the only sense channel — **and the only one that speaks for the owner: a line admitted here is `source: owner`** (§21.4). Post-[docs/26](26-world.md) it is the *world's* panel: it carries the speaking **identity** into the sense stream (OD-123) and the **owner lane** for nomination asks, and it still cannot grant anything — authority comes from a `/trust` row, not from a transport |
| `mcp.py` — **the supporter seam (SPEC, §21.7)**: read-only tools over the existing artifacts (`read_trace` / `read_state` / `read_vocab` / `read_rows`, `read_monologue` pending **OD-38**) + `sense_input` + the gate log | sense | innate mechanics, zero content | **the second finger on the same channel.** It is the only place a line may arrive with `source: grimm`, it can hand `sense.py` nothing else, and it holds **no handle on lattice, walk, drive or arousal** — L1 is enforced by what this module does not import. Reads are logged; `speak()` is not a state write, it is a keystroke from a different voice |
| `speakgate.py` — **the two-layer mouth filter** (§21.3, R21): layer 1 token membership over the lexicon shelf, layer 2 **strict single-row restatement** over the live L3 rows (**OD-40(i)**), machine-readable rejection reasons | sense (pre-admission) | innate **form filter** | sits **upstream of `sense.py`**, so a rejected emission never becomes a sense event: no signals, no `a` movement, no stamp, no row — one line in the gate log and nothing else. **Consults no semantics, no `a`, no charge, no bake, and no emotion word**; adds no vocabulary to §3.2. **Applies to Grimm only — never to the owner (L5)**, and it is stdlib, so his honesty is checkable without trusting him |
| `sense.py` — R1–R3, R12 tokenizer + patterns | shared | innate | parse — **and the `source` tag rides the decoded line through to `context` and `gate` (§21.4); it is carried, never inferred from the text**. Post-[docs/26](26-world.md) it is also where the **class function** lives: `owner \| grimm \| trusted \| stranger` is derived per line from the grant rows plus the owner's own lines and **stored nowhere** (§26.2, R-28 style) — G19(b) greps for a stored class and reds on one |
| `crystallize.py` — **the experience detector** ([docs/22](22-crystallize.md)) | KNOW + DRIVE (ask kinds) | innate detector shape; its thresholds are OD-97, **unrulled** | reads the frame's ghost scratch (§16.8) at settle, counts recurrences over **live rows only**, never reads `a`, and emits a `cluster-nomination` ask through the ordinary drive interface. **Reader: imported by nothing in the control path, and it holds no handle on `gate`** — the only path from its candidate table to a row is an owner line answering an ask. Its table is session state, hash-exempt by OD-98's declared default, logged to `crystallize.jsonl` |
| `silt.py` — **the middle timescale** ([docs/23](23-context.md)) | THINK (frame warmth) | innate mechanics, zero content | one session table of per-point salience: lights deposit, the logical tick erodes, entries fall below threshold in last-touch order, so **context clears backwards with no clearing instruction executing**. Read by `idle.py`'s sampler only (OD-104's declared default), never by the gate, the label ladder, or a `FLOOR_*` (OD-107). Its weight row is **SPECIES and unarmed at 0**, so the four temperament ignition weights stay four and B7's law holds |
| `skills.py` — **the shape detector** ([docs/25](25-skills.md)) | THINK (composition) + DRIVE (ask kinds) | innate detector shape; thresholds OD-116, **unrulled** | the exact twin of `crystallize.py` applied to *move* instead of *content*: reads role-graphs (roles, not members), requires `K_filler` distinct filler-sets before a candidate opens, reads outcome **rows** from the eventlog and never `a`, emits `move-nomination`, and lets an owner-ratified `skill` row bias traversal order only. **A skill is a row, never a program: no module executes one** — the missing dependency edge is what makes that a compile-time fact |
| `tesseract.py` — `(t,x,y,z)` addressing + shelf registry | KNOW | innate | schema of schemas |
| `layers_frozen.py` — L1/L2 loaders | KNOW | read-only, CONFIG ENGINE only | genome; file handles asserted closed in CONFIG SI runs |
| `lattice.py` — L3 shelves, `active_at(s)`, NOW, append-only | KNOW | learned contents | SI's memory |
| `walk.py` — spreading-activation engine (v1: CPU, fixed-order, quantized; GPU leg v2) | THINK | innate mechanics, learned web | §16 — the thought itself; emits settle-set + activation trace + the `a` **trajectory**. Two calls into the drive interface (`drive.update`, `drive.conduction`) — **`drive.update` fires per signal event inside the walk, not once per thought** (§20.1) — and **holds no emotion name and no emotion-shaped state** (§18.1, OD-14, OD-20). Post-v2 it owns three things it did not have: **frame-scoped scratch** (§16.8 — ghosts and role-graphs, destroyed at settle, never hashed, never cited), **`prov-link` traversal** ([docs/24](24-episodic.md) §24.3 — derived views, a re-light into the frame, `RELIGHT_MAX` cap, shipped behind `EPISODIC_TRAVERSAL = off`), and **skill-aware composition order** ([docs/25](25-skills.md) §25.6) |
| `arousal.py` — **one leaky integrator** `a(t+1) = clamp(a·λ_a + Σ gains(signals), 0, 1)` in logical `t`, the `a → (reach, decay, τ_settle)` modulation curve plus the **one-round lag** (version-tagged, OPEN-16/16a/29), the charge-stamp builder (§18.5), **the bake curve** `B_max · tanh(k · a_formation)` (§20.4a, OD-33, constants OPEN-33), and **the point-median statistic** — the recency-weighted median over a point's stamp history (§20.3) | DRIVE tissue | innate physics — **the signal list, the integrator, the modulation curve, the lag, the bake curve, the median law** | **wordless by law (§18.6, §20.8): it contains no emotion name.** Round 1's module `affect.py` — six integrators, the coupling matrix, per-resident λ/baselines/hysteresis — is **cancelled (OD-21)** and this is what replaces it. No organ calls it directly; only the drive interface does. **One-clock law is a dependency here, not a habit: `arousal.py` imports the recency curve from `clock.py` and defines no decay of its own** (R-29); **it never persists a median** — recomputed per thought, stored nowhere (R-28). **The bake is the one write-side magnitude derived from `a`** — the stamps' `a_at` *records* `a`, it never *becomes* a size; the saturating curve bounds that derivation, which is what OD-33 ruled open |
| `audit.py` — **the readout face of CALIBRATE**: classifies patterns over `(signals, a, behavior)` into the label stream, and writes the observed co-occurrence artifact `gates/G9/coupling_observed.json` (§18.3) | CALIBRATE (readout) | innate classifier shape; its thresholds/constants = **OPEN-22** | **the only module in the build that may contain an emotion word** (R-21 grep test). It is a **reader**: imported by nothing in the control path, and its output is log text + export. If `audit.py` ever gains an import edge into `walk/gate/drive`, that is the costume failing (R-24) |
| `idle.py` — **ignition sampler** (§19): the seeded 4-weight picker (recency / **edge bake** / decay deficit / live-contradiction warmth, **OPEN-25**; the bake reading is what **closed OPEN-32**), the quiet-tick precedence (queue revisit → RE-PARSE → ignition), the monologue renderer call, and the budget guard | THINK (idle tick) | innate sampler, **seed inside the state hash** (§19.4) | **structurally read-only**: no import of the commit/write path, so "an idle walk can notice, never learn" is a dependency fact, not a discipline (I9, G10). It **reads** a baked edge's `bake` and never writes one. Renders through `render.py`'s existing face — it does not own a generator (§19.3). Post-[docs/23](23-context.md)/[docs/26](26-world.md): it gains the **silt** term as a *fifth weight slot* (SPECIES, unarmed at 0 — the four temperament draws stay four), reads quiet **per conversation** instead of globally, and honors only windows that can **suppress** a tick (OD-126; I9) |
| `drive.py` — **DRIVE interface** `{prioritize, emit, budget, salience, update, conduction}` — the last two are the arousal doors (OD-20; §17.1 said "at minimum") | cross-cutting layer | innate interface | §17 — after round 2, `budget`/`emit` are **queue rule + integer ceiling** (OD-22: they lost their level input), `salience` is the **bake curve** `B_max · tanh(k · a_formation)` (§20.4a — OD-22's write-side half was reversed by **OD-33**, and it reversed only this door, at this one instant; `magnitude`/`budget` stay signal-valued), `update` integrates one scalar **per signal event**, `conduction` returns **both read-side affect numbers the walk may use: the per-edge bake term and the per-point median term** (§20.3/§20.4) as one quantized dict. Curiosity is its default **policy**. **Wordless** — round 1's shared naming privilege with `affect.py` is withdrawn (§18.6) |
| `askengine.py` — thin adapter: hands walk gaps to `drive`, renders emitted asks | WHAT→DRIVE | innate plumbing | §7.2 |
| `gate.py` — surprise + consistency + arbitration | LEARN | innate rules | §6 — also the **novelty/contradiction signal source** handed to `drive.update` (§18.1: gate never names an emotion; **arbitration takes no affective input**, OD-24). **Since the supporter batch it holds two source tests, both cheap and both load-bearing: a `source: grimm` line never matches a pending ask (§6.1) and never takes the boring-input confirmation bump (§6.2, R-35) — and every row it commits must be able to print `speaker: owner` or the commit is a bug (G12(e))** |
| `reparse.py` | LEARN + THINK | innate pass, learned rows | §6.6 — inherits charge stamps as history only, never as a fresh boost (§5.4) |
| `retrieve.py` | WHAT | innate scorer, learned keys | §7.3 — applies the per-round-normalized bias exactly as returned by `drive.conduction()` (§18.5 cap); never recomputes it from `a`, never reads a stamp's contents, and **passes the query's time anchor through** so a history-anchored walk gets the median at `t'` (§20.5) |
| `predict.py` — prediction game, probes, guess slot R11 | CALIBRATE | learned stats | §7.4, §8.1 |
| `calibrate.py` — labels + source tags + disagreement reporting | CALIBRATE | innate | §9 — demote-only, and **takes no affective argument at all** since round 2: the label is a pure trace function (§9 properties) |
| `render.py` | — | v1 template copier; **v2 LM slot lives here** (form only, demotion contract §7.5) | output — and §19.3's monologue face: the **same** decoder renders owner-directed output and idle cascades, so "internal speech" costs no new machinery and cannot be a hidden generator |
| `tongue.py` — **surface realization by walk** ([docs/27](27-tongue.md)): the five style draws lean a token-by-token walk over the term lattice, then the three structural checks gate the line | — (a read-side component, **not a seventh organ**: §2's six-organ law is untouched) | innate **walker + gate + derive order**, learned **surfaces** | sits between `calibrate`'s label and the outbound line, **after** it and never before it: it emits a surface string and nothing else — no row, no edge, no stamp, no byte. It reads no `a` (I8, OD-168), no `source` (OD-166), no runtime stream (OD-176), and no frozen-genome arc (OD-179), **and it utters only surfaces its own citation names** (§7.5's law, widened onto this module by OD-181 — it may walk further than it may speak). Delete it and `render.py`'s copy is the output again, byte for byte (**G20(j)**) |
| `clock.py` + `eventlog.py` — step counter, mind log, state hash | shared | innate | determinism; **`clock.py` owns the ONE recency curve** every age-driven quantity reads — §8.4's decay deficit, §19.2's ignition recency weight, §20.3's median weights (the **one-clock law**; R-29) — and nothing may define a second. **The per-thought `a` *trajectory* + label stream (§18.6, §20.1) ship next to the activation trace**; the trajectory, each stamp's `a_at`, and the ignition seed are inside the hash (quantized); the **median is never hashed and never stored** (it is derived, §20.3); the **monologue stream is a separate log lane** from owner-facing output (§19.3, OPEN-23) |

Dependency direction (no cycles): `panel → sense → gate/retrieve/askengine →
lattice → tesseract`; `predict/calibrate/render` hang off the read/write
paths; `layers_frozen` is reachable ONLY behind the config flag; **signals
flow one way into the drive interface and out of it again:**
`walk/gate/eventlog/idle →(signals, per event) drive.update →(integration) arousal →
(a, bake term, median term) drive →(reach/decay/τ modulation + W_eff) walk`,
while `drive.salience →(bake + stamp) write path`. **The bake is the only
arrow in this diagram that runs from the tissue into the write path** (OD-33),
and it is one-way and *formation-only*: the write path consults `a` exactly once
per edge, at the instant the edge is created, through one saturating curve; no
already-baked edge is ever re-consulted, so no stored row can be re-priced from
`a` later. That single-arrow scope is what keeps OD-33 from becoming round 2's
forbidden `if <signal> then <behavior>` in a new coat.
Nothing reaches `arousal` except through `drive.update`, and nothing leaves
it except through `drive.*` — the organs and the walk see **doors, never the
tissue**. **Round 2 shortened that one-way street to three links** — signals →
`a` → walk modulation — and hung the emotion names off it as a read-only
branch: `eventlog → audit → (log text, export)`, with **no edge from `audit`
back into anything** (§18.2: "the label never feeds back into control"). The
street is what makes I8 checkable rather than aspirational; the grep rules are
(i) emotion names appear **only** in `audit.py`, never in
`walk/gate/drive/arousal/calibrate/retrieve/reparse/render/lattice/idle`
**and never in the arc's three new modules `crystallize` / `silt` / `skills` or
in the event-view and traversal code, nor in `tongue.py` — whose five axis names
are movement words (spread, depth, lean, recency, proximity), never temperament
adjectives** (R-21, §22.8/§23.7/§24.7/§25.8/§27.8's
"R-21 extension" lines — the grep list grows with the build, the privilege does
not), (ii) `idle.py` has no import of `gate.py`'s commit path (R-25),
(iii) **no row schema, export, or cache field is named anything like a median /
bias / mood — the read-side emotional reading exists only as a value returned
inside a thought** (§20.3, R-28). The **`bake` field is the opposite case and is
deliberately stored**: it is write-side, frozen at formation, and the two rules
must never be merged into a grep that bans both, or the permanent bake becomes a
check-passing bug; (iv) **`arousal.py` contains no decay or
recency constant of its own** except `λ_a`, the arousal τ — it imports the clock
(§20.3 one-clock law, R-29). The bake curve's `B_max` and `k` are **scale**
constants, not decay constants, so they live in `arousal.py` and are swept by
OPEN-33 — and **`arousal.py` must not grow an event-kind table to pick them**
(OD-31: one curve for every event kind, or the anti-cheat law is gone).

**The supporter's dependency arrow, and it has exactly one head:**
`grimm (external process) → mcp.py → speakgate.py →(admitted line, source:
grimm) sense.py → …the ordinary pipeline`. **There is no arrow back.** SI imports
nothing from `grimm/`, never spawns him, never reads his scratch, and cannot be
made to wait on him; `mcp.py` is a listener that hands over text or hands over
nothing. That one-way shape is what "outside the loop" means in a component map
rather than in a paragraph, and it is why unplugging him is a configuration, not
a refactor (§10.4, G12(a)). Two grep rules join the list:
**(v)** no module under `src/` imports or references `grimm/` in any form —
**the membrane is in the build, the model is not** (OD-37); and
**(vi)** `speakgate.py` and `mcp.py` reference **no** affect symbol: neither may
read `a`, `charge`, `bake`, the median, or any drive door — a gate that could
see arousal is one refactor away from `if SI is upset: soothe()`, which is I8's
forbidden setter wearing a kindness (L3, R-34). Both modules are also emotion-
word-free under rule (i) like every other non-`audit` file.

**Three more arrows the v2 arc draws, and they all point the same way — away
from the lattice.** The arc's whole safety case in a component map is that
every new thing is a **reader**: `crystallize`, `silt` and `skills` are
imported by the session and by `idle`, and **none of them imports `gate.py`,
`lattice.py`'s append path, or `drive`'s emission doors** — their only route to
a row is an ask the owner answers, which is `gate.py`'s existing door, not a new
one. So: **(vii)** no edge `crystallize|silt|skills → gate|lattice.append`
exists, and G15(b)/G16(b)/G18(b) delete the module and re-run the lattice hash
to prove it; **(viii)** no `prov-link`, event view, ghost or role-graph is ever
written as a row, so the derived family stays derived (OD-110, R-28 — the grep
is on the shelf, and an `x == SHELF_EVENTS` row is a bug); **(ix)** `speaker_class`
is a **function, not a field** — a stored trusted/stranger tag anywhere in a
record, cache or export reds G19(b), because a class that survives its grant is
authority that outlives the owner's act.

**The map's count, stated instead of fudged.** This table now lists
**twenty-three** module rows over a thirteen-crate workspace — OD-43 keeps the
map *logical*, so the two numbers were never one number (one crate carries
several rows, and `clock.py` + `eventlog.py` share a row). The v2 arc adds three
rows for three new crates (`si-crystallize`, `si-silt`, `si-skills`) plus one
row's worth of behavior that adds **no crate at all**: episodic traversal is a
rule inside `walk.py` over views that live in no shelf, which makes it the
cheapest thing in the arc to delete — G17(b) is a grep, not a migration — and
the class function likewise lives inside `sense.py`/`panel.py`'s existing front.
`crystallize.py` and `skills.py` stay two files rather than one module with two
families on purpose ([docs/25](25-skills.md) OD-119 asked for exactly that, so
the deletion tests stay independent).

**One seat stays empty, on purpose.** `consolidate.py` ⇄ `si-sleep`: the crate
exists and ships (B8), the map has no row for it, and whether it grows one is
**OD-84** — a design-layer edit, so it is the owner's hand, not this fold's. The
arc above neither settles it nor silently pays it.
