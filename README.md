# SI — Workspace Layout

```
├── README.md                 ← this map
├── DESIGN.md                 ← thin master index: § number → docs file
├── docs/                     ← the design, spread out, one section per file
│   ├── 00-thesis-invariants.md
│   ├── 01-interaction-model.md
│   ├── 02-organs.md
│   ├── 03-birth-state.md
│   ├── 04-demo-trace.md
│   ├── 05-substrate-tesseract.md
│   ├── 06-write-path.md
│   ├── 07-read-path.md
│   ├── 08-gain-loss.md
│   ├── 09-claim-labels.md
│   ├── 10-configs-converters.md
│   ├── 11-stages.md
│   ├── 12-component-map.md
│   ├── 13-gate-plan.md
│   ├── 14-risk-register.md
│   ├── 15-definition-of-done.md
│   ├── 16-think-walk-hardware.md
│   ├── 17-drive-layer-curiosity.md
│   ├── 18-emotions-engine.md    ← filename kept for link stability; the
│   │                              section is now §18 AROUSAL + OBSERVED READOUTS
│   ├── 19-idle-ignition.md      ← idle thought, ignition, the internal monologue
│   ├── 20-emotional-plasticity.md ← median (read-side) vs bake (write-side,
│   │                              permanent and **saturating**: §20.4a
│   │                              `bake = B_max·tanh(k·a_formation)`, OD-27
│   │                              resolved / OD-33 open), per-event arousal +
│   │                              one-round lag, `a_at`,
│   │                              4D emotional history, emotional archaeology, G11
│   ├── 21-grimm-supporter.md    ← GRIMM: the supporter — an organ-adjacent,
│   │                              outside-the-loop, locally hosted gated LM
│   │                              (L1–L5, the two-layer speak gate, the `source`
│   │                              privilege table, the v1 MIRROR / v2 QUESTIONER /
│   │                              v3 CHALLENGER ladder, gate G12, OD-34…40)
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
│   ├── 27-tongue.md           ← THE TONGUE: surface realization by walk
│   │                              (five style draws over lattice quantities,
│   │                              a token-by-token walk over LIVE arcs, the three
│   │                              structural checks — label before realization,
│   │                              every word a surface the CITATION names (§7.5,
│   │                              OD-181), parse-back — canonical frame as the only
│   │                              refusal; no seventh organ, no row kind, no state
│   │                              byte, no `a`/`source`/runtime-stream read;
│   │                              `TONGUE` ships `off`, V14 turns it on;
│   │                              gate G20, OD-163…184)
│   └── decisions/
│       ├── open-decisions.md   ← flagged conflicts awaiting owner ruling
│       │                        (OD-01…OD-40 from the design rounds;
│       │                        OD-41…OD-86 raised by the engineering pass;
│       │                        OD-87…OD-96 from the B7/B8 build batches, where
│       │                        OD-87/88/91/93 are RULED and the rest are walls;
│       │                        OD-97…OD-136 from the v2-arc fold and its
│       │                        engineering pass — 133/134 are tail seats the
│       │                        fold discovered after the drafts closed,
│       │                        135/136 are seats [12](engineering/12-v2-arc.md)
│       │                        found while pricing G19; none renumbered inward;
│       │                        OD-137…OD-162 are the seats the build itself opened
│       │                        (the night regression, B10, B11, G1.u, step 6,
│       │                        B12's panel pass), and
│       │                        OD-163…OD-184 from the tongue fold — the last one
│       │                        is the fold's own catch, not the draft's: the
│       │                        drafted vocabulary check grepped the whole
│       │                        registry, a leg that cannot fail, against §7.5's
│       │                        older law that the cited rows license the line)
│       └── changelog.md        ← design history / merge record
├── engineering/              ← the BUILD SPEC derived from docs/, no code:
│   ├── README.md             ← authority order + this folder's own OD list
│   ├── 00-stack-workspace.md … 10-temperament-sleep.md
│   │                          (HOW only: byte layouts, call order, crate
│   │                        edges, units, sweep artifacts. It changes no law;
│   │                        where docs/ under-decides it writes "lean Y; OD-nn".
│   │                        `10-temperament-sleep.md` is the one file the
│   │                        owner asked to be engineered straight from a
│   │                        fragment — temperament seed, sleep's four passes,
│   │                        dimming, gates G13/G14, OD-71…85.)
│   ├── 11-build-plan.md      ← the plan the build obeys: nineteen stages
│   │                        (execution B0–B9, B14–B18, B10–B13),
│   │                        gate-id DoD, commit boundaries, the three-attempt
│   │                        stop rule, global acceptance, OD-86's placeholders
│   ├── 12-v2-arc.md          ← the v2 arc's HOW (docs/22–26): the five schemas,
│   │                        the three new crate edges, the ruling rows shipped
│   │                        `off`, G15–G19 procedures + artifacts, the sweeps that
│   │                        publish refusals while OD-83 is unrulled, and the V1…V5
│   │                        build stages appended to 11's plan
│   ├── 13-tongue.md        ← the tongue round's HOW (docs/27): the five style rows
│   │                        + `PHRASE_MAX_STEPS` + the `TONGUE` switch under a sixth
│   │                        registry class, `DERIV_ORDER_TONGUE` keyed by the same
│   │                        birth seed (so no species vector, digest or recorded
│   │                        individual moves), the `si-tongue` crate edge and its
│   │                        one call site, the phrase-walk algorithm and its three
│   │                        checks, G20's eleven legs + artifacts, V14 and stage
│   │                        B20, the break-list (17 crates, `CONSTS_VERSION 0x0003`)
│   └─ (see engineering/README.md for the file-by-file map)
├── gates/                    ← reserved: gate run artifacts (transcript.txt —
│                                every input line with its `source` tag,
│                                mindlog.json, monologue.jsonl, state_hash.txt,
│                                gate_report.json + the sweep/observed JSONs:
│                                G2 + G8 + G9/arousal_sweep (incl. the bake
│                                curve's `B_max` + `k`, OPEN-33, reported as a
│                                distribution) + G9/coupling_observed
│                                (output-only) + G10/ignition_sweep +
│                                G11/median_sweep + G12/absence_diff,
│                                G12/rejection_reasons, G12/speak_gate_log —
│                                the G12 three are observation-only) — and,
│                                specified in engineering/05 §18–19 and pending
│                                the owner's fold-in ruling (OD-84's family):
│                                **G13/sleep_sweep, G13/dimming_sweep,
│                                G14/seed_panel, plus the two run artifacts
│                                `sleep.jsonl` and `fingerprint.txt`**, all
│                                readouts only, never inputs — and the v2 arc's
│                                five families, specified in docs/13 §13.13 and
│                                engineering/12: **G15/recurrence_sweep +
│                                nomination_log, G16/silt_sweep + silt_log,
│                                G17/relight_log, G18/shape_sweep + skill_log,
│                                G19/server_transcript + grant_trace + lane_log**;
│                                every one of them records `build_variant` and the
│                                graph digest it ran on (OD-91/OD-93), and a sweep
│                                that cannot enumerate publishes a **refusal** —
│                                and the tongue's **G20/realized_log + refusals +
│                                phrase_sweep (a refusal, OD-172) + v0_strings**,
│                                where `v0_strings` is the byte-identity artifact
│                                that makes V0 provably unchanged (G20(j), OD-178)
└── crates/                   ← the RUST WORKSPACE: 13 at the arc's opening
                                 (si-core, si-store, si-front, si-walk, si-affect,
                                 si-learn, si-read, si-session, si-sleep, si-audit,
                                 si-supporter, si-vulkan, si-cli), 16 after the v2
                                 arc (si-crystallize, si-silt, si-skills) and 17
                                 with the tongue (si-tongue) — because
                                 a crate edge is how a grep rule becomes a
                                 compile-time fact. **Pure stdlib, zero deps, NO LM
                                 inside any crate** (docs/12); `grimm/`, when he is
                                 built, stays a separate process outside the
                                 workspace and outside the loop — docs/21
```

Rules of the workspace:
- **The spec is read-only from here.** Phase 3 is running: `docs/` is the law,
  `engineering/` is the HOW, `crates/` is the rip, and `gates/` holds the run
  artifacts that are the only definition of done. A wall becomes
  `failure_report.md` plus its OD question — never a silent workaround.
- Edits land in the owning `docs/NN` file; `DESIGN.md` changes only if a
  section is added/renumbered.
- A fragment that clashes with an owner-decided core (see
  docs/02 + docs/05 law lines) goes to `docs/decisions/open-decisions.md`
  as a numbered OD-item, never a silent edit.
- **`engineering/` is HOW, and it loses every argument it has with `docs/`.** It
  may add crates, byte layouts, call orders, units, sweeps and gate procedures; it
  may not change a law. Where the design under-decides, it writes *"the design does
  not decide X; lean Y; OD-nn"* into the ledger instead of picking silently. The one
  exception to "docs first" is a fragment the owner explicitly asked to be
  engineered straight — `engineering/10-temperament-sleep.md` (temperament, sleep,
  dimming) — and there every place the chunk's wording had to bend to a law is an
  OD row, not an edit.
- Naming law: the system is **SI** everywhere; "Baby SI" is dead; newborn
  framing lives only in `docs/11`. A chunk's own pet name for the baby (docs/21's
  "Nova") is quoted in the fold, never adopted — and a name SI ends up with can
  only arrive through the channel (OD-34).
- **The one-channel law has a scope now:** the sense seam may carry more than one
  voice, and every line is tagged `source: owner | grimm | trusted | stranger` at
  the wire, never inferred from text — the last two are **derived** from the grant
  table and stored nowhere, so the 80-byte envelope and every record written since
  B1 stay untouched (docs/01 §1.6, docs/21 §21.4, docs/26, I3A/I10A). **Belief rows
  are the owner's alone** unless a live grant row says otherwise, and tonight zero
  grant rows exist: the protected class is `s_r_pairs` / `facts` / `links` /
  `skills`, edges, bakes, stamps and confidence, while `context` sense rows and
  ordinary `questions` ask rows keep appending for any voice (docs/03 §3.3).
- Anything numeric that was not produced by an actual run is labeled
  **UNMEASURED**.
