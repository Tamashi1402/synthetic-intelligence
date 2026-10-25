import io, sys

E = [
 ("docs/22-crystallize.md",
"""writes: new L3 point `fruit` (links shelf, kind `cluster-point`), edges `fruit --is-a-member-of-direction--> each member` (relation kind: OD-101), provenance `{source: crystallize, asked_at: t_ask, confirmed_by: owner line id}`. Candidate closes `status: resolved`.""",
"""writes: new L3 point `fruit` (links shelf, kind `cluster-point`), edges `fruit --is-a-member-of-direction--> each member` (relation kind: OD-101), and **ordinary provenance — no new enum value**: `prov_kind: answer-to-ask`, `question_id:` the nomination ask row, `prov_speaker: owner`, so `gain_kind: demonstrated` by §8.2's own mapping, exactly as an auto-commit earns it (the error-before is the open question). **The crystallize origin lives in the ask row's `kind`, not in a sixth provenance kind** — which is what keeps docs/15 item 3's constant list closed and the G5c diff empty. Candidate closes `status: resolved`."""),

 ("docs/25-skills.md",
"""name, provenance `{source: crystallize-shape, confirmed_by: owner line\nid}`.""",
"""name, and **ordinary provenance — no new enum value**: `prov_kind: answer-to-ask` with `question_id` set to the `move-nomination` ask row and `prov_speaker: owner`, so the row is `demonstrated` by §8.2's mapping exactly as a cluster point is; the shape origin rides the ask row's `kind` (docs/15 item 3's closed list gains no sixth provenance kind)."""),

 ("docs/08-gain-loss.md",
"""| gain_kind | decided once at commit from `provenance.kind` (§8.2 table):""",
"""| gain_kind | decided once at commit from `provenance.kind` (§8.2 table) — **and the v2 arc adds no new key to this mapping on purpose: a crystallize or skills confirmation is an ordinary `answer-to-ask` commit (the nomination ask row is the open question that preceded it), so it lands `demonstrated` like any answer, with its `question_id` naming the `cluster-nomination` / `move-nomination` row. A minted cluster point therefore carries real provenance and no new vocabulary (docs/15 item 3's closed literal list; R-03's audit is the reason that matters)**:"""),

 ("docs/15-definition-of-done.md",
"""   `cluster:` name expanded into vocabulary, a shape given an English name by""",
"""   `cluster:` name expanded into vocabulary, a shape given an English name by"),

 ("docs/05-substrate-tesseract.md",
"""| `questions` | L3 live | **open-question rows**""",
"""| `questions` | L3 live | **open-question rows**"""),
]

def apply(path, old, new):
    with io.open(path, encoding="utf-8") as f:
        text = f.read()
    n = text.count(old)
    if n != 1:
        print("FAIL %s count=%d :: %r" % (path, n, old[:60]))
        sys.exit(1)
    with io.open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write(text.replace(old, new))
    print("ok %s :: %r" % (path, old[:40]))

for path, old, new in E[:3]:
    apply(path, old, new)
