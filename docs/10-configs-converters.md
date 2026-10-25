# §10 — The Two Configs and the Converters (SPEC ONLY — do not build)

## 10.1 CONFIG SI vs CONFIG ENGINE — same code, different load

| | CONFIG SI | CONFIG ENGINE |
|--|-----------|---------------|
| L1/L2 | **not loaded** (load path asserts and refuses) | loaded frozen at t=0 |
| L3 | live, identical code | live, identical code |
| purpose | the REAL test; all v1 gates run here | the mature system; its t=0 is where SI climbs toward over years of conversation |
| gates that exist | G1–G5, G7–**G12** (all of v1's gates but one; **G12's legs run with Grimm both connected and disconnected**) | + G6 |

The post-N-session gap between the two configs is itself a designed
long-term leg; **label UNMEASURED until run** — no timestamp/size/accuracy
number in this workspace is claimed as measured.

## 10.2 Converter spec: WordNet → Layer 1 (dictionary rows)

- Mapping: synset → hyperpoint (`hyperpoint_id` = content hash of
  `(lemma, pos, sense_index)` ⇒ stable across versions); lemma/POS/forms →
  dictionary-row fields; sense keys → `y` within `dictionary` shelf.
  Hypernym chains emitted as relation rows in L2 (they are correlations),
  not as fields, keeping L1 rows pure nodes.
- Provenance/license stamp: `source: "wordnet@<version>"`, license field
  per file; **IMMUTABLE** — any change ⇒ full re-version with new t=0 tag.
- Volume estimate (~155k words / ~117k synsets, ~207k relations; row counts
  and bytes **UNMEASURED** — the converter's first act is producing the
  count report).
- t=0 version tag = content hash of the sorted row file; recorded in the
  shelf registry so CONFIG ENGINE runs are comparable across machines.

## 10.3 Converter spec: ConceptNet + grammar tables → Layer 2

- Mapping: edge `(a, rel, b)` → relation row `{a_id, b_id, edge_kind: rel,
  weight}`; ConceptNet `IsA`/`PartOf`/`Causes`/`UsedFor` map to `edge_kind`
  verbatim — **no relabeling** (relabeling = judgment = hidden ontology).
  Sources normalized to L1 hyperpoint ids by lemma+POS intersection;
  unmatched surface forms get their own L1 rows flagged
  `provenance: wordnet|conceptnet-only` (no silent drop — drops are content
  decisions).
- Grammar property tables → **rule rows** + grammatical relation rows
  (`Heat --is-a--> noun`, `Heat --is--> singular, not "Heats"`): inflection
  paradigm tables ⇒ rule rows with trigger/result hyperpoints, composable
  per §5.3.
- Volume: ~34M edges; bytes/split ratios **UNMEASURED** until build.
  Load budget: CONFIG ENGINE stays laptop-runnable ⇒ mmap index by `(x,y)`
  and `z` adjacency; CONFIG SI never opens the files (handle audit asserts
  zero open handles during a full gate run).
- Licenses: ConceptNet CC-BY — stamped per row-batch; redistribution audit
  is a build-time task ([docs/14](14-risk-register.md) R-10).

## 10.4 The supporter and the two configs ([docs/21](21-grimm-supporter.md))

Grimm is **orthogonal to the config axis** — he is neither loaded nor
not-loaded by the flag; he is a second process that either runs or does not.
Three consequences, all of them gate-checked:

| claim | why it holds | tested by |
|-------|-------------|-----------|
| **Every gate in §13 passes with the supporter process not started**, in both configs | the sense seam is an input path, not a dependency; with nothing arriving on it, SI behaves as designed | **G12(a)** (bit-identical state hash with and without Grimm connected over the same owner transcript) |
| CONFIG SI's clean-room law is **not** softened by having an LM in the room | I2 forbids pretrained *content* in SI's shelves; §21.3 forbids pretrained content in SI's *input*. The gate is the second half of the law | G5c + **G12(b)** (the vitamins rejection) |
| The supporter never makes a stage or a gate easier to pass | he cannot create rows, bake, confirm, or issue `/correct` (§21.4) — so no gate's assertions have a helper. **If SI needs Grimm's content to pass a gate, the design failed** (the chunk's own line, kept verbatim as the construction rule) | **G12(c)** + every existing gate's control run |

Version-ladder unlocks are the one place a **gate result** gates a *person*: v2
unlocks when G2's decode leg is green, v3 when G4 is (docs/11's stage counts are
unchanged — **SI's stages do not advance because Grimm is ready**, §11.1's law
extended to the new component).

## 10.5 The v2 arc and the two configs ([docs/22](22-crystallize.md)–[docs/26](26-world.md))

**G15–G19 are CONFIG SI gates, and the axis itself is untouched.** Five
mechanisms, one converter change: none. No new genome row, no new shelf, no new
record layout, no new file that the CONFIG SI process is allowed to open — so
§10.1's handle audit and G6's checksums re-run exactly as before, and the arc's
deletion legs (G15(b), G16(b), G17(b), G18(b)) are the config-law's own shape
applied to new code: *remove it, nothing in the loaded state differs.*

| mechanism | CONFIG SI | CONFIG ENGINE | what the config axis actually changes |
|-----------|-----------|---------------|----------------------------------------|
| crystallize | live, **weaker** | live | §22.3's detection rule lets frozen layers contribute **fillers but never members**. CONFIG SI has no BASE at all, so a nomination here can only be built from taught `links`/`s_r` fillers — fewer fillers, fewer candidates. That is I2 working as designed, not a bug to level away: **an engine build nominates categories the owner never typed more easily, and that is a reason to gate it, not to skip the gate.** |
| silt | live, identical physics | live | nothing — warmth is a session table over the owner's lines; no layer access |
| episodic binding | live, derived views | live | `prov-link` is a registered **weak edge kind**, not a converter row; the event view is a view over `context`, so no shelf of copies exists in either config (OD-110 ships derived) |
| skills | live, rows only | live | a skill row is a kind on shelf 9 in both configs; the *role-graph template* is serialized from already-taught surface words, so no engine-side grammar is consulted (§25's generality bound is stricter in CONFIG SI, same reason) |
| N voices | live | live | identity rides the **sense stream** (event body + transcript line), so the 80-byte envelope — and every stored record since B1 — is unchanged. Under tier (a) no non-owner line reaches a row at all, which is the strongest form of the config law: the class of a speaker is derived from the grant table, which is L3 content the owner wrote. |

**The one honest asymmetry to publish with the arc:** CONFIG ENGINE gives
crystallize and skills more raw material to nominate over, because WordNet and
ConceptNet fillers are cheap and everywhere. So the engine-side runs of G15 and
G18 are not the acceptance runs — the **CONFIG SI** legs are, and they are the
ones docs/15 item 2 requires green. An engine build that clears G15 while the SI
build stays `UNMEASURED` has demonstrated that a genome can find structure; it
has not demonstrated that *experience* nominated it (R-10's shape, one level up).

