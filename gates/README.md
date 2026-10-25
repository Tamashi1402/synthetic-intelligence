# gates/ — reserved for run artifacts

This directory holds run artifacts, and it is the only definition of done that
counts. Each gate run G1–G12 (see
[../docs/13-gate-plan.md](../docs/13-gate-plan.md)) produces, per run — plus, for
the two gates the TEMPERAMENT/SLEEP batch adds, **G13 SLEEP and G14
TEMPERAMENT**: those are specified in
[../engineering/05-gate-harness.md](../engineering/05-gate-harness.md) §18–19, and
docs/13 carries them only as a debt note (§13.13's count paragraph) because the
build folder may not silently renumber the design's gate plan — the fold-in is the
same class of ruling **OD-84** records for the component map. Their artifacts are
reserved here so a run directory has one home for them; see the two extra files
below and the three sweeps. The v2 arc adds **G15–G19**, which do have a law
section (§13.13) and are listed at the end of this file. The tongue round adds
**G20**, whose legs are law at [../docs/27-tongue.md](../docs/27-tongue.md) §27.8
and whose HOW is [../engineering/13-tongue.md](../engineering/13-tongue.md); its
artifacts are below, and one of them (`G20/phrase_sweep.json`) is a **refusal**,
not a measurement.
plus the sweep artifacts the open legs demand (`G2/constants_sweep.json`,
`G8/budget_sweep.json`, `G9/arousal_sweep.json` = `λ_a`, signal gains, the
modulation curve 16a, the one-round lag 29 **and the bake curve's `B_max` + `k`
(OPEN-33 — reported as a distribution over the session's formed edges, since
"no edge above the cap, no lattice pinned at it" is the finding, not a value)**,
`G10/ignition_sweep.json` = the
four ignition weights,
OPEN-25, `G11/median_sweep.json` = N, the repetition count and every bias
magnitude, `G11/composition_sweep.json` = the owned-hop depth cap
`INFERENCE_MAX_HOPS_OWNED` (its every row carries
`rows_written_during_composition`, which must be 0 in all of them — the number
being swept is the *claim* bound, never a write permission), plus the composition
legs' two evidence artifacts `G11/route_traces.jsonl` (per leg: the settle trace,
`route_counts{owned, base}`, the label emitted and the label refused) and
`G11/zero_row_export.txt` (the belief-shelf export immediately before and after
one composite answer, byte-compared), and the supporter batch's three
**observation-only** artifacts — `G12/absence_diff.json` (the byte-level diff of
the two G12(a) runs: supporter disconnected vs connected-but-silent; a non-empty
diff is a finding, not a value to tune), `G12/rejection_reasons.json` (per
rejected emission: `{n_untaught_tokens, novel_claims, ref_row_id}` — the
machine-readable WHY the gate returns, §21.3) and `G12/speak_gate_log.jsonl`
(every `speak()` attempt, admitted or rejected, with the layer that decided) —
all sweep **outputs**, none design inputs, §20.7) and the **output-only** emergent
artifact
`G9/coupling_observed.json` (observed label co-occurrence; it may never be read
back into a build — §18.3) and the three sweeps plus one diff the two new gates
demand — `G13/sleep_sweep.json` = `SLEEP_EVERY_QUIET_TICKS` and `SLEEP_MAX_WALKS`
(the cadence and the one per-night budget, **OD-85**; its rows carry the
`walks` / `events_available` pair so a truncated replay pass is visible in the
sweep and not only in a run report), `G13/dimming_sweep.json` = `DIM_BASE` and the
per-shelf floors, each row reporting `within_shelf_rank_inversions`,
`cross_shelf_rank_inversions`, `max_relative_shift` and `floor_hits` — a row with
a suppressed count **fails G13(n)** even when every behavioral leg is green,
because the inversions are real behavior and not noise to trim — and
`G14/seed_panel.json`, which is **not a tuning artifact at all**: G14 tunes
nothing, it validates the declared boxes across a fixed panel of ≥ 16 seeds, and a
failing point narrows the box in
[../engineering/03-constants-registry.md](../engineering/03-constants-registry.md)
and re-runs instead of re-sampling the panel (**OD-83**). `G12/absence_diff.json`
has a sleep-side twin, `sleep_diff.json` (the V0-vs-V9 diff of the same transcript
— the evidence that a sleep happened at all, which must be attributable to
`weight` and `last_touched_at` and to nothing else). Same rule for all four:
**outputs, never inputs — R-33 gets no exemption because the file is a sleep log**:

```
gates/G<n>_<config>_<yyyymmdd-hhmm>/
  transcript.txt     # the sense stream (input lines), each carrying its
                     # `source: owner | grimm` tag (§21.4) — the tag is part of
                     # the recorded input, so G12's replay is reproducible and
                     # the owner's half stays separable
                     # (G12(a) runs the same transcript twice: supporter process
                     #  not started, then started-and-silent. Two run
                     #  directories, one absence_diff.)
  mindlog.json       # every gate decision, label, arbitration, probe,
                     # per-thought arousal **trajectory** `a` (every per-event
                     # value, with the round it applied — §20.1) + the
                     # per-point medians the thought conducted with (§20.3, log
                     # only) + audit label stream
  monologue.jsonl    # idle cascades (§19.3) — a separate lane, never sent to
                     # the panel; G10(d) checks each entry against its trace
  sleep.jsonl        # step 12b's lane (engineering/10 §8): one fixed-shape line
                     # per pass per sleep, plus one interrupt line
                     # {t, passes_completed, aborted_reason} for a night cut
                     # short. Its ABSENCE means the run never slept — a
                     # legitimate state, not a missing artifact (G13(f)). A log:
                     # it is never state, never hashed, and never read back (R-33)
  fingerprint.txt    # the temperament readout (engineering/10 §10): birth seed
                     # in hex, `seed_format`, one line per TEMPERAMENT constant
                     # {registry name, grid value, its declared box} and the
                     # digest — written from RAM that is re-derived and stored
                     # nowhere, so publishing it costs no state byte. Output
                     # only: G14(c)'s digest check re-DERIVES from the seed, it
                     # does not read this file
  state_hash.txt     # deterministic state fingerprint at NOW (includes the `a`
                     # trajectory, charge stamps incl. `a_at`, the per-edge
                     # `bake` (write-once, permanent, bounded by `B_max` —
                     # §20.4a), the ignition seed
                     # — G5a/G5f; **point medians are deliberately absent**:
                     # they are derived, and a stored one is R-28)
  gate_report.json   # pass/fail + per-criterion evidence (routes, ASK
                     # counts, confidence transitions, checksums)
```

`G9/coupling_sweep.json` is **retired** with the cancelled coupling matrix
(OD-21); a run directory that cites it is citing a design that no longer exists.

**The v2 arc's five families (G15–G19)** land here with the same shapes, and one
new rule binding all of them: each family's **control run is the arc-off variant**
— an OD-91-style designed comparison that asserts *equality* of the state hash,
never a difference — and every artifact records `build_variant` **and** the graph
digest it ran on, because the arc grows the workspace graph (13 crates → 16) and
the digest is the only thing that pins a variant (OD-93).

**G20 TONGUE** is the sixth such family and keeps the rule with a harder edge: its
control is **V0**, where the switch is `off`, and the assert is not a hash equality
but a **string** equality over every rendered line in the repo (`G20(j)`) — because
the tongue writes no state, a state hash would stay identical even if the voice had
changed every sentence. The graph grows again (16 → 17), so `build_variant` and the
digest are mandatory here too, and `CONSTS_VERSION` `0x0003` re-signs every recorded
control by **re-running** it (OD-91), never by editing a file.

```
  G15/nomination_log.jsonl  # one line per candidate transition: frame id,
                        # member set, shared fillers, recurrence_count, and the
                        # ask it emitted with `name=NONE`. The candidate table
                        # itself is hash-EXEMPT (OD-98), so this file is the only
                        # audit trail it has — G15(a)'s byte-identity is proved
                        # against state, G15's content against this log
  G15/recurrence_sweep.json # R_min / R_ask / Δ_min / GHOST_MAX — publishes a
                        # REFUSAL while OD-83's enumeration is unrulled, and the
                        # legs then run at the box's declared floor (OD-95)
  G16/silt_log.jsonl        # deposits, per-tick decay, entries dropped below
                        # θ_silt, `SILT_MAX` refusals (refused, never evicted) and
                        # the moment the table went empty with no clearing
                        # instruction executing — G16(d)'s evidence
  G16/silt_sweep.json       # S_bump / S_max / λ_s / θ_silt / T_cold / SILT_MAX —
                        # same refusal shape; OD-107's arousal-scaled decay is
                        # absent from the code, so it is not even a grid point
  G17/relight_log.jsonl     # every frame that traversed a `prov-link`: seed,
                        # moment, `a_now` vs the stamp's `a_at`, and the re-light
                        # count against `RELIGHT_MAX`; G17(d) reads this to show
                        # zero stamps and zero bakes were written by any of them
  G17/prov_sweep.json       # PROV_LINK_TUG / RELIGHT_MAX (OD-114) — same refusal
                        # shape; whether the tug reads silt is OD-104's and the
                        # code holds no edge between them, so it is not a grid
                        # point either
  G18/skill_log.jsonl       # shape candidates, the `K_filler` generality test's
                        # verdict, the `move-nomination` ask, and each ratified
                        # skill row's composition preference — as a row, never a
                        # program
  G18/shape_sweep.json      # K_filler / R_min_SHAPE / GHOST_MAX sharing — refusal
  G19/server_transcript.txt # the scripted multi-voice fixture (OD-132): every
                        # line with its `source` tag AND its identity, because
                        # identity rides the sense stream — this file is where it
                        # lives, not in any envelope
  G19/grant_trace.json      # each trusted-class write → the grant row it cites →
                        # the owner line that created that grant; revocation
                        # included (G19(c)). Zero rows at birth is the pass, not
                        # an empty artifact
  G19/lane_log.jsonl        # nomination asks and their lane field: the only
                        # legal value is `owner` (OD-128), and
                        # the busy-server starvation leg reads it
  G20/realized_log.jsonl    # one line per outbound line with `TONGUE` on: the
                        # canonical frame, the realized frame, the label, the tag,
                        # the citation and the licensed-surface set the line was
                        # checked against. G20(b) reads it for label immutability
                        # and G20(h) reads it for the out-of-citation grep — the
                        # two legs the draft could not have failed separately
  G20/refusals.jsonl        # every line the gate sent back to canonical, with the
                        # check that fired. An empty file is a pass for nothing:
                        # the fixture forces each of the three checks (G20(c)(g))
  G20/phrase_sweep.json     # PHRASE_MAX_STEPS — status `refused` while OD-83
                        # stands (OD-172), so the leg that needs the number runs at
                        # the box's `lo` and says so. The five style rows appear
                        # nowhere in it (OD-87, OD-177)
  G20/v0_strings.json       # every rendered string in every existing scenario at
                        # V0, re-hashed after the CONSTS_VERSION 0x0003 bump,
                        # byte-compared against the recorded v1 render (G20(j)).
                        # This is the artifact that makes the round shippable
```

**OD-134 is stated on `G15/`'s threshold:** the arousal-isolation leg sweeps
`a` with the **frame trace fixed**, feeding the detector an identical trace under
two arousal regimes, because `a` lawfully moves the walk whose traces the
detector reads. The full-session reading of the same leg is recorded
`UNMEASURED` beside the refusal — the ban on the detector reading `a` is the
claim being tested, and it is tested where it can actually fail.

Grimm's lab notebook (§21.6's v1 MIRROR, §21.7) is **not** in this directory and not in
SI's storage — it is a supporter-side document that *cites* these artifacts. If
a `G12/` run directory contains a file that the build reads back, the notebook
has become a channel, which is R-33.

Rule: numbers that appear anywhere else are **UNMEASURED** until a run
directory like this exists to cite.
