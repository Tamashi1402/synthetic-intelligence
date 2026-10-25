import io, sys

EDITS = [
    # (path, old, new)
    ("docs/03-birth-state.md",
     """**The v2 litmus, stated on the birth table where it belongs.** Born with the
whole arc switched on and the owner silent: **no state byte changes.** There is""",
     """**The v2 litmus, stated on the birth table where it belongs — and the one
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
There is"""),

    ("docs/22-crystallize.md",
     """Litmus: with crystallize active but the owner silent, **no state byte in the
lattice changes.** Log lines and volatile frame state may change; the state
hash does not.""",
     """Litmus: with crystallize active but the owner silent, **no belief row
changes** — and the exception is [docs/03](03-birth-state.md) §3.3's, stated
there once: the emitted nomination writes its ordinary `questions` bookkeeping
row and nothing else. Log lines and volatile frame state may change; no
`s_r_pairs`/`facts`/`links`/`skills` row, no edge, no bake, no confidence moves.
**A candidate is never a row: the only state write crystallize owns is the ask.**"""),

    ("docs/22-crystallize.md",
     """- **G15(d) owner-only:** a `source: grimm` line answering a crystallize ask
  produces zero lattice writes.""",
     """- **G15(d) owner-only:** a `source: grimm` line answering a crystallize ask
  produces zero **belief-row** writes (§21.4's protected class; its `context`
  sense row still appends, which is the sense record doing its job and not a
  write of content)."""),

    ("docs/23-context.md",
     """Litmus: with silt active but the owner silent, **no state byte in the
lattice changes**, and after enough quiet ticks the silt table is empty
without any clearing instruction ever executing, only decay.""",
     """Litmus: with silt active but the owner silent, **no belief row moves** — silt
is the one mechanism of the arc that writes nothing at all, not even an ask, and
the [docs/03](03-birth-state.md) §3.3 exception is listed there for the others
and simply does not apply here. And after enough quiet ticks the silt table is
empty without any clearing instruction ever executing, only decay."""),

    ("docs/24-episodic.md",
     """Litmus: with episodic binding active and the owner silent, **no state byte
changes**: no row, no stamp, no bake, no confidence bump.""",
     """Litmus: with episodic binding active and the owner silent, **no belief row
moves**: no row, no stamp, no bake, no confidence bump (the `context` sense rows
the event views are *derived from* are already there, appended by the sense
stream as §21.4 requires — binding reads them, it does not create them)."""),

    ("docs/25-skills.md",
     """Litmus: with skills active and the owner silent, **no state byte changes**.""",
     """Litmus: with skills active and the owner silent, **no belief row changes**
(its only permitted state write is the ordinary `questions` row of the emitted
`move-nomination` — the exception is [docs/03](03-birth-state.md) §3.3's)."""),

    ("docs/01-interaction-model.md",
     """- **Litmus.** Born on the server with 10+ voices typing and the owner silent:
  **no state byte changes.** Strangers light walks, raise signals, deposit silt,
  and write nothing.""",
     """- **Litmus.** Born on the server with 10+ voices typing and the owner silent:
  **no belief row moves.** Strangers light walks, raise signals, deposit silt,
  and append `context` sense rows — which is the sense record, not content
  (§21.4; the exception is spelled out in [docs/03](03-birth-state.md) §3.3, and
  the protected class is `s_r_pairs` / `facts` / `links` / `skills`, edges,
  bakes, stamps and confidence)."""),

    ("docs/13-gate-plan.md",
     """first leg is the negative one — *with the mechanism off or the owner silent, no
state byte changes*.""",
     """first leg is the negative one — *with the mechanism off or the owner silent, no
belief row moves*, with the one exception spelled out in docs/03 §3.3 (`context`
sense rows and ordinary `questions` ask rows still append; they are not beliefs)."""),

    ("docs/13-gate-plan.md",
     "  silent, zero lattice writes**), class-derivation (a stored class is a red),",
     "  silent, zero **belief-row** writes**), class-derivation (a stored class is a red),"),

    ("docs/15-definition-of-done.md",
     """    and **no state byte changes**; every act of authority walks back to an owner""",
     """    and **no belief row moves** (`context` sense rows and ordinary ask rows still
    append, docs/03 §3.3); every act of authority walks back to an owner"""),
]

for path, old, new in EDITS:
    with io.open(path, encoding="utf-8") as f:
        text = f.read()
    n = text.count(old)
    if n != 1:
        print("FAIL anchor count=%d in %s :: %s" % (n, path, old[:60]))
        sys.exit(1)
    with io.open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write(text.replace(old, new))
    print("ok %s :: %s" % (path, old[:48].replace("\n", "\\n")))
print("all edits applied")
