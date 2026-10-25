import io, sys

R = [
 (None,
"""│   │                              v3 CHALLENGER ladder, gate G12, OD-34…40)
│   └── decisions/""",
"""│   │                              v3 CHALLENGER ladder, gate G12, OD-34…40)
│   ├── 22-crystallize.md      ← CRYSTALLIZE: experience PROPOSES structure,
│   │                              only the owner mints it (ghost → candidate →
│   │                              nomination ask, `name=NONE`, gate G15,
│   │                              OD-97…101 + tail seats OD-133/134)
│   ├── 23-context.md          ← SILT: the middle timescale, a session table of
│   │                              topical warmth that never enters the lattice
│   │                              (per-conversation, decay on the one clock, the
│   │                              fifth ignition weight ships UNARMED, gate G16,
│   │                              OD-102…109 + §23.9's numbering note)
│   ├── 24-episodic.md         ← EPISODIC BINDING: provenance becomes traversable
│   │                              (derived event views + `prov-link`, the origin
│   │                              tug, `RELIGHT_MAX`, no re-stamp, no flashback,
│   │                              gate G17, OD-110…115)
│   ├── 25-skills.md           ← SKILLS: a move is a ROW, never a program
│   │                              (role-graph shape scratch, `move-nomination`,
│   │                              composition preference only, gate G18,
│   │                              OD-116…122)
│   ├── 26-world.md            ← THE WORLD: N voices, delegated authority
│   │                              (four class tags, `speaker_class` derived and
│   │                              stored nowhere, `/trust` + grant rows,
│   │                              tier-strict arbitration, owner lanes, logical
│   │                              suppress-only windows; gate G19, OD-123…132 —
│   │                              and tonight the LAW ships, the network does not)
│   └── decisions/"""),

 (None,
"""       ├── open-decisions.md   ← flagged conflicts awaiting owner ruling
│       │                        (OD-01…OD-40 from the design rounds;
│       │                        OD-41…OD-86 raised by the engineering pass)""",
"""       ├── open-decisions.md   ← flagged conflicts awaiting owner ruling
│       │                        (OD-01…OD-40 from the design rounds;
│       │                        OD-41…OD-86 raised by the engineering pass;
│       │                        OD-87…OD-96 from the B7/B8 build batches, where
│       │                        OD-87/88/91/93 are RULED and the rest are walls;
│       │                        OD-97…OD-134 from the v2-arc fold — 133/134 are
│       │                        tail seats the fold discovered after the drafts
│       │                        closed, deliberately not renumbered inward)"""),

 (None,
"""│   ├── 11-build-plan.md      ← the plan the build obeys: fourteen stages,
│   │                        gate-id DoD, commit boundaries, the three-attempt
│   │                        stop rule, global acceptance, OD-86's placeholders""",
"""│   ├── 11-build-plan.md      ← the plan the build obeys: fourteen stages,
│   │                        gate-id DoD, commit boundaries, the three-attempt
│   │                        stop rule, global acceptance, OD-86's placeholders
│   ├── 12-v2-arc.md          ← the v2 arc's HOW (docs/22–26): the five schemas,
│   │                        the three new crate edges, the ruling rows shipped
│   │                        `off`, G15–G19 procedures + artifacts, the sweeps that
│   │                        publish refusals while OD-83 is unrulled, and the V1…V5
│   │                        build stages appended to 11's plan"""),

 (None,
"""│                                G14/seed_panel, plus the two run artifacts
│                                `sleep.jsonl` and `fingerprint.txt`**, all
│                                readouts only, never inputs""",
"""│                                G14/seed_panel, plus the two run artifacts
│                                `sleep.jsonl` and `fingerprint.txt`**, all
│                                readouts only, never inputs — and the v2 arc's
│                                five families, specified in docs/13 §13.13 and
│                                engineering/12: **G15/recurrence_sweep +
│                                nomination_log, G16/silt_sweep + silt_log,
│                                G17/relight_log, G18/shape_sweep + skill_log,
│                                G19/server_transcript + grant_trace + lane_log**;
│                                every one of them records `build_variant` and the
│                                graph digest it ran on (OD-91/OD-93), and a sweep
│                                that cannot enumerate publishes a **refusal**"""),

 (None,
"""└── src/                      ← created only after design approval; v1 plan is
                                 in docs/12 (pure stdlib, zero deps, NO LM
                                 anywhere inside it). grimm/, if he is ever
                                 built, is a separate process outside src/ and
                                 outside the loop — docs/21""",
"""└── crates/                   ← the RUST WORKSPACE, 13 crates today (si-core,
                                 si-store, si-front, si-walk, si-affect, si-learn,
                                 si-read, si-session, si-sleep, si-audit,
                                 si-supporter, si-vulkan, si-cli); the v2 arc adds
                                 three (si-crystallize, si-silt, si-skills) because
                                 a crate edge is how a grep rule becomes a
                                 compile-time fact. **Pure stdlib, zero deps, NO LM
                                 inside any crate** (docs/12); `grimm/`, when he is
                                 built, stays a separate process outside the
                                 workspace and outside the loop — docs/21"""),

 (None,
"""- **Design only until approval.** No code, no schema migrations, no converter
  builds. `gates/` stays empty until a build exists.""",
"""- **The spec is read-only from here.** Phase 3 is running: `docs/` is the law,
  `engineering/` is the HOW, `crates/` is the rip, and `gates/` holds the run
  artifacts that are the only definition of done. A wall becomes
  `failure_report.md` plus its OD question — never a silent workaround."""),

 (None,
"""- **The one-channel law has a scope now:** the sense seam may carry more than one
  voice, and every line is tagged `source: owner | grimm` at the wire, never
  inferred from text. Belief rows are the owner's alone (docs/01 §1.5, docs/21
  §21.4, I10).""",
"""- **The one-channel law has a scope now:** the sense seam may carry more than one
  voice, and every line is tagged `source: owner | grimm | trusted | stranger` at
  the wire, never inferred from text — the last two are **derived** from the grant
  table and stored nowhere, so the 80-byte envelope and every record written since
  B1 stay untouched (docs/01 §1.6, docs/21 §21.4, docs/26, I3A/I10A). **Belief rows
  are the owner's alone** unless a live grant row says otherwise, and tonight zero
  grant rows exist: the protected class is `s_r_pairs` / `facts` / `links` /
  `skills`, edges, bakes, stamps and confidence, while `context` sense rows and
  ordinary `questions` ask rows keep appending for any voice (docs/03 §3.3)."""),
]

ARC_ROWS = """| 22 | [22-crystallize](docs/22-crystallize.md) | **CRYSTALLIZE — experience NOMINATES, only the owner MINTS**: ghost (frame-scoped composite, destroyed at settle, §16.8's law) → candidate (session recurrence table, hash-exempt, `crystallize.jsonl`) → **ask** (an ordinary drive emission, `pattern=FACT/OWNED` + **`name=NONE`**, owner lane) → owner-confirmed `cluster-point` row on `links`. Depth is earned (\\|M\\| ≥ 3, overlapping fillers, `R_min` distinct frames); frozen layers lend fillers never members; **the detector never reads `a`**; `CRYSTALLIZE` ships **`off`**. **G15 a–f**, OD-97…101 (+ tail **OD-133** the stage-ladder seam, **OD-134** how the arousal-isolation leg is swept) |
| 23 | [23-context](docs/23-context.md) | **SILT — the middle timescale stays out of the lattice**: `silt: {point → salience}` deposited when a point lights, decayed per **logical** tick, cleared backwards by decay alone, table-per-conversation (OD-109/127). Feeds **ignition only** (OD-104), ships as the sampler's **fifth weight, SPECIES and unarmed at 0** so the four temperament draws stay four; **no arousal-scaled decay (OD-107 banned, not merely unused)**; deposits at full weight for every voice (OD-106). **G16 a–f**, OD-102…109 + §23.9's numbering note that re-issued the arc's seats |
| 24 | [24-episodic](docs/24-episodic.md) | **EPISODIC BINDING — origin is traversable, experience is not claimed**: the event **view** (derived, OD-110 — no new shelf, no envelope change) + `prov-link` weak edges, so a recalled fact can come back with the moment that taught it; the origin tug drags a warm topic's moment into an idle walk; re-lights are **read-only, capped (`RELIGHT_MAX`), and bake nothing** (G17(d)); a grimm moment is scenery, never an origin. **G17 a–f**, OD-110…115 |
| 25 | [25-skills](docs/25-skills.md) | **SKILLS — a move is a ROW, never a program**: role-graph shape scratch in the frame → `move-nomination` → owner-named skill row on the `skills` shelf (id 9), serialized template + citations; a ratified habit may **order composition** and may not touch a constant, the sampler's temperament, or be *executed* (the crate that detects shapes has no edge into the write gate). Generality is earned (`K_filler` distinct filler sets — one pair is an association, not a move). **G18 a–f**, OD-116…122 |
| 26 | [26-world](docs/26-world.md) | **THE WORLD — N voices, zero authority without a grant**: I3 amended to **I3A** (four tags, classes derived and stored nowhere, owner-or-granted teaching, priority law unchanged); `speaker_identity` rides the **sense stream**, not the envelope; `WORLD_TIERS` ships at **tier (a)** — owner-only — so identity never enters a row; **`/trust` is coded, not granted** (zero grant rows at birth; revocation moves the arbitration ladder, never the lattice); nomination asks land in the **owner lane**; idle windows are **logical and suppress-only**; ten strangers cannot buy thoughts (**G19 a–h**), and **the law is gated while the Discord transport is not built**, OD-123…132 |
"""

D = [
 (None,
"""**Design only — no code, no builds, no approval yet.** The design lives""",
"""**The law, the HOW and the rip.** Phase 3 is running: this index, `docs/` and
`engineering/` are read-only from here, `crates/` is the build, and `gates/`
artifacts are the only definition of done. The design lives"""),

 (None,
"""| — | [decisions/open-decisions](docs/decisions/open-decisions.md) |""",
ARC_ROWS + """| — | [decisions/open-decisions](docs/decisions/open-decisions.md) |"""),

 (None,
"""**OD-86** (what a `—` constant takes while the build must run: the tagged placeholder policy). The table with the leans is""",
"""**OD-86** (what a `—` constant takes while the build must run: the tagged placeholder policy); **OD-87…OD-96** from the B7/B8 build batches — **OD-87 (the four ignition weights are TEMPERAMENT), OD-88 (G10.b.2 is structural), OD-91 (identical hash IS the pass) and OD-93 (the variant tag; cross-variant comparison is a category error) RULED by the owner**, the rest are walls; **and the v2 arc's OD-97…OD-132, none ruled, each naming its declared least-authority default and the artifact that blocks it — plus two tail seats the fold discovered afterwards (OD-133 the kind-blind `links` stage criterion, OD-134 how an arousal-isolation sweep can mean anything)**. The table with the leans is"""),

 (None,
"""Twelve numbered files (00–11) plus its own README, no code:""",
"""Thirteen numbered files (00–12) plus its own README:"""),

 (None,
"""the gate harness **G1–G14** + the `check_*` family,""",
"""the gate harness **G1–G19** (`12-v2-arc.md` is the arc's HOW: five schemas, three crate edges, the ruling rows shipped `off`, G15–G19 procedures, the V1–V5 stages) + the `check_*` family,"""),

 (None,
"""the temperament/sleep batch engineered straight from the owner's fragments)** |""",
"""the temperament/sleep batch engineered straight from the owner's fragments)** → **THE v2 ARC (docs/22–26 folded as law: crystallize, silt, episodic binding, skills, N voices — five gate families G15–G19 appended, five ruling rows shipped `off`, zero new organs, zero new doors, zero new structures, zero new hash items, and the Discord transport honestly left unbuilt)** |"""),

 (None,
"""| 13 | [13-gate-plan](docs/13-gate-plan.md) | **G1–G12**:""",
"""| 13 | [13-gate-plan](docs/13-gate-plan.md) | **G1–G12 in §13's body, plus §13.13's arc families G15–G19** (and its own count debt: seventeen law-text families here against nineteen in engineering/05, because G13/G14 never received a law section — **OD-84** — and one variant discipline paragraph that binds all five: every determinism and deletion leg is an **OD-91-shaped designed control comparison asserting equality**, no leg may read a cross-variant *difference* as a failure, and every report carries `build_variant` + graph digest):"""),

 (None,
"""| 15 | [15-definition-of-done](docs/15-definition-of-done.md) | acceptance: §4 replay + **twelve** gate reports""",
"""| 15 | [15-definition-of-done](docs/15-definition-of-done.md) | acceptance: §4 replay (**items 12–16 are G15–G19; Act 7's s28–s35 are OD-132's fixture, not more steps of the north-star run — with the arc's five rows `off`, s1–s27 replays byte-identical**) + gate reports (**nineteen families**; the header's "twelve" carries its own correction note) + **a `refused` sweep is not a green magnitude**"""),

 (None,
"""grep rules (i)–(vi)""",
"""grep rules (i)–(ix) — the arc adds **(vii)** no `crystallize`/`silt`/`skills` edge into the write path, **(viii)** no scratch, prov-link or ghost written as a row, **(ix)** `speaker_class` is a function and not a field, and rule (i)'s wordless file list grows with the three new modules"""),

 (None,
"""**§11.2 the supporter does not move the ladder — stages never advance because Grimm is ready, and a mirror-accelerated transition is counterfeit confirmation (R-35)** |""",
"""**§11.2 the supporter does not move the ladder — stages never advance because Grimm is ready, and a mirror-accelerated transition is counterfeit confirmation (R-35)**; **§11.3 the v2 arc moves no rung either** — a nomination mints a row only through the owner's own typing, no other voice advances a ladder it cannot write, and the one kind-blind `links` seam the fold found is **OD-133**, not a silent discriminator |"""),

 (None,
"""**§10.4 the supporter and the two configs — every gate passes with Grimm not started, the clean-room law is not softened by an LM in the room, and he never makes a gate easier** |""",
"""**§10.4 the supporter and the two configs — every gate passes with Grimm not started, the clean-room law is not softened by an LM in the room, and he never makes a gate easier**; **§10.5 the v2 arc and the two configs — G15–G19 are CONFIG SI gates, the arc converts nothing and opens no new handle, CONFIG SI stays the acceptance config, and an engine build that nominates more easily has proven a genome rather than an experience** |"""),

 (None,
"""annotated into the apple scene, not rewritten (**OD-32**);""",
"""annotated into the apple scene, not rewritten (**OD-32**); **Act 7 (s28–s35) is OD-132's multi-voice fixture — a `gates/` scenario with strangers, silt, a nomination, a mint and an origin-cited recall, and deliberately NOT more steps of the north-star run**;"""),

 (None,
"""**and, since §21, with a parent in the room:""",
"""**and, since the v2 arc, with a room full of voices it may not learn from unless
the owner says otherwise — it may only NOMINATE the shapes it keeps seeing, keep
topics warm for the next thought, carry the moment a fact was taught back with the
fact, prefer a move it has earned, and never let any of that write a belief by
itself;** and, since §21, with a parent in the room:"""),
]

def apply(path, pairs):
    with io.open(path, encoding="utf-8") as f:
        text = f.read()
    for _path, old, new in pairs:
        n = text.count(old)
        if n != 1:
            print("FAIL %s count=%d :: %r" % (path, n, old[:70]))
            sys.exit(1)
        text = text.replace(old, new)
    with io.open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    print("ok %s (%d edits)" % (path, len(pairs)))

apply("README.md", R)
apply("DESIGN.md", D)
