# §1 — The Interaction Model: Chat Panel and `/correct`

The panel is the only sense organ. Two input kinds, and — post-
[docs/21](21-grimm-supporter.md) — two possible **voices** typing into them,
generalized to **N** by [docs/26](26-world.md) (§1.6):

```
normal input : conversational text (question, statement, noise)
             : also — a plain reply to a question SI is currently asking
               counts as teaching (§1.4, answer-to-ask)
correction   : /correct <new answer>   ← the override channel (highest tier)
teach        : /teach <query-pattern> => <response>
             ← owner-authored content in one act (§1.4, added by the TEACH
               fragment; there is NO error-before in front of it, and that is
               its only difference from the other two channels)
trust        : /trust <identity> teach|correct|answer   ← an owner act that
             writes a GRANT ROW, and the only way a voice other than the
             owner's ever gets authority (§1.6, [docs/26](26-world.md) §26.3(b))

source       : owner | grimm | trusted | stranger   ← what the line may DO.
               `owner` and `grimm` are typed tags; `trusted` and `stranger`
               are DERIVED at generation from the grant table plus the owner's
               lines and stored nowhere (§1.6). The channel is single; the
               voice is tagged, and the tag decides what the line may DO,
               never whether it may arrive.
identity     : the speaking account, metadata on sense (§1.6). It is not a
               belief: SI knowing who someone is arrives through the same
               gates as everything else (I1).
```

## 1.1 Correction protocol (core teaching signal)

- A correction event is the triple **(query, SI's wrong guess, owner
  correction)** — a fully labeled learning event. The guess is *measured*
  error-before; the correction is ground truth; therefore the **gain of
  writing the row is known exactly** ([docs/08](08-gain-loss.md) §8.2), not
  estimated.
- **Binding:** a correction binds to the WHOLE last output in v1. Every row
  records `bind_output_id` from day one so v2 partial corrections never
  rewrite history.
- **Later `/correct` with conflicting content** ⇒ supersession on the time
  axis (old belief aged, not deleted, change citable) AND the contradiction
  arbitration ([docs/06](06-write-path.md) §6.4) runs and is recorded — a
  win by tier still logs the disagreement, never silently overwrites.
- **Repeated successful recalls** raise confidence; exact repeats of a known
  statement write nothing.
- **Correction length is the curriculum.** Single-word corrections teach
  single-word answers; sentence corrections teach sentence forms. Rendering
  composes from learned chunks; grammar grows from chunk reuse. The owner
  controls developmental pace by typing alone.

## 1.2 Stage-0 storage: S-R pairs first

At stage S0, storage is the raw **S-R pair** (stimulus pattern → response),
kept forever as ground truth. A later **RE-PARSE pass**
([docs/06](06-write-path.md) §6.6) derives typed fact rows from old S-R
pairs once parsing reflexes mature. Both coexist, linked by provenance
(`derived_from: [sr_row_ids]`).

## 1.3 Honest stage-0 limit — stated out loud

At birth the connectome is empty: the walk lights nodes but finds no paths,
so **every question misses until a `/correct` creates the first OWNED
edges** ([docs/16](16-think-walk-hardware.md) §16.2). Once an OWNED edge
exists, recall generalizes over *frame rewording* (any sentence that lights
`apple` can reach `apple→red`) — but that is entity-term identity, not
paraphrase magic (§14 R-02; full generalization rules restated in §7.3).
**A reworded question that lights no known content node still MISSES at
S0/S1** — that is correct newborn behavior, not a bug. True paraphrase
(entity renamed, no shared content word) unlocks only through teaching:
shared-term generalization, taught "same-question" links (`X means Y`
teachings), and — in CONFIG ENGINE only — genome paths. Never faked, never
claimed as FACT before the OWNED edge or taught link exists.
(Recall-semantics shift from the pre-walk signature-match rule is logged as
OD-07 for the owner to confirm.)

## 1.4 Teaching channels (two since CURIOSITY, three since TEACH)

| channel | how | role |
|---------|-----|------|
| **Explicit correction** | `/correct <answer>` — binds to the whole last output (bind_output_id) | THE override channel; always available; top provenance tier (T3) |
| **Answer-to-ask (auto-commit)** | **the owner** (`source: owner`) types a plain reply while an SI question is **pending** in the open question queue | conversation is *also* the teaching channel: the reply is ground truth **by context** — commits like a `/correct` (surprise high, coverage none, the hole was just measured), provenance `kind: answer-to-ask`, linked to the question it resolves (§17.2.3). **A `source: grimm` reply never matches a pending ask** — that is how content would launder its provenance (§21.4, R-32) |
| **Owner teach** | **the owner** types `/teach <query-pattern> => <response>` — a decoded question shape on the left, the content to bind on the right | the third channel: **owner-authored content through the sense channel**, not an exception to any law. Same decode (form only, R3/R14 frames), same surprise/coverage gate, same consistency check and arbitration, **exactly one row**, provenance `kind: owner-teach`, owner-tier confidence init, **bound to the decoded pattern** rather than to an output. What it lacks is an **error-before**: nothing wrong was guessed first, so the gain is **owner-asserted, not owner-demonstrated** (§8.2's gain ledger names the two kinds). **`source: grimm` may not use it** (§21.4) |

A plain statement with no pending question and no pattern is input, not a
teach — auto-commit is gated on the queue, never on vibes. This is why the
demo's first lesson needs no `/correct`: SI asked, so the reply teaches.

**What the third channel does not change (TEACH fragment).** Every law that
governed the other two still governs `/teach`, and the fold states the four
that a bulk-curriculum command is most likely to be argued around:
(i) **it writes at most one row per act** — a covered slot means a confidence
bump and **no new row** (G1 is unchanged, and G1 does not know or care which
channel the line arrived on);
(ii) **it is not a `/correct`** — it does not supersede by tier and does not
bind to an output id, so a conflicting existing row goes through §6.4's
arbitration like any other owner statement and the disagreement is still
recorded, never silently overwritten;
(iii) **its left side must decode**, because the row is keyed by a *decoded
pattern*, not by the text the owner typed — `/teach` with an undecodable left
side produces a named dark spot and **zero rows** (§7.1), which is what keeps
the channel inside the anti-cheat law instead of beside it;
(iv) **it earns no demonstrated gain** until a later recall actually serves
from it (§8.2), so a lattice imported wholesale in one afternoon starts with
`asserted` everywhere and its confidence is only as good as the recalls that
follow. **The honest published limit:** `/teach` can out-run the ask→answer
loop that the demo trace is made of, so a transcript of two hundred teaches
contains no error-before at all; that is a property of the curriculum the
owner chose, not a broken channel, and §1.3's stage-0 limit still applies to
anything the pattern cannot name.

## 1.5 Two voices, one channel (the supporter batch — the template [docs/26](26-world.md) §26.3 generalizes to N)

| privilege | `source: owner` | `source: grimm` |
|-----------|-----------------|-----------------|
| arrives as sense, lights the walk | yes | yes |
| raises signals → `a` (§18.1) | yes | yes — this **is** co-regulation, and it is the only causal effect he has |
| names a gap SI may then ask about | yes | yes (§21.4) |
| creates a **belief** row (`s_r_pairs` / `facts` / `links`) | yes | **no** |
| is recorded verbatim in `context` (the sense record, not a belief — §5.2) | yes | yes |
| auto-commits against a pending ask | yes | **no** |
| `/correct` (the override channel) | yes — **owner-privileged, permanently** | **no** |
| `/teach` (the curriculum channel, §1.4) | yes — **owner-privileged, permanently** | **no** — and a `source: grimm` line shaped like a `/teach` is **inert input**, not a teach (§21.4, G12(d)) |
| forms an edge, buys a bake (§20.4) | yes | **no** |
| confirmation bump: `repeats`, `last_confirmed_at`, +0.05 conf (§6.2) | yes | **no** (R-35) |
| gets filtered for unknown tokens | **never** — the owner is the source of the lexicon | always (L5, §21.3) |

The right-hand column is not a demotion of Grimm's status; it is the reason a
pretrained model may sit next to a zero-content system at all. Note what it
costs: **the supporter cannot help SI learn faster in the only way that would
actually be cheating.** Everything he does, he does by being *in the room*.

**And the right-hand column is the template.** [docs/26](26-world.md) does not
invent a stranger's law; it applies this one to every identity the owner has
not granted, and to a trusted user for everything outside his grant row. Read
that column as "anyone who is not the owner", and §1.6 is already written.

## 1.6 N voices, one world (the I3A amendment round — [docs/26](26-world.md))

The deployment the owner ruled: **one SI, born on a server, in conversation
with 10+ users from `t=0`.** The channel law generalizes and the teacher law
delegates; nothing else changes. What that means for the panel:

- **Identity is metadata on sense, never a belief.** Every line arrives tagged
  with the speaking account, kept in the sense record's **stream** — the event
  body and the transcript line — where a perturbation's metadata already
  lives. It is not a field of the belief envelope: under tier (a) below, no
  non-owner line writes a row, so identity never reaches a row and the stored
  record layout is untouched. **OD-123** holds the question if the owner wants
  it in a row.
- **The class is derived, stored nowhere.** `owner | grimm | trusted | stranger`
  is computed at generation from the grant table plus the owner's own lines —
  the §20-median precedent, R-28 style: a derived class cannot be smuggled into
  a row, only re-derived. The write gate consults the class function exactly as
  it consults `source` today.
- **Tier (a) is the birth state, and tonight's build ships it**: strangers never
  teach; every non-owner line is grimm-class; SI still learns *about* the
  server through crystallize's recurrences ([docs/22](22-crystallize.md)) and
  only the owner's word mints anything. **Tier (b)** — `/trust <identity>
  teach|correct|answer` — is coded, gated (G19(c)), and **granted to nobody**: a
  build that handed out a grant to make a gate interesting would be the
  laundering §26.3(c) rejects. **Tier (c)**, peer teaching, is rejected.
- **Nomination asks go to the owner lane.** A `cluster-nomination`
  ([docs/22](22-crystallize.md) §22.4) or `move-nomination`
  ([docs/25](25-skills.md) §25.5) ask carries a lane field whose only legal
  value is the owner lane (OD-128). The queue rule and the integer ceiling are
  untouched: **a busier world may not buy more asks.**
- **An owner line still preempts everything**, and idle ignition is judged per
  conversation, not globally (§26.4, I9) — the only schedule it may honor is a
  window on the logical axis that can suppress a tick, never cause one.
- **Litmus.** Born on the server with 10+ voices typing and the owner silent:
  **no belief row moves.** Strangers light walks, raise signals, deposit silt,
  and append `context` sense rows — which is the sense record, not content
  (§21.4; the exception is spelled out in [docs/03](03-birth-state.md) §3.3, and
  the protected class is `s_r_pairs` / `facts` / `links` / `skills`, edges,
  bakes, stamps and confidence). The owner's first line does everything it does today, in
  public, with strangers in the room.

**Scope, stated honestly:** tonight ships this *law*, not the network. The
Discord adapter is a transport at the panel's edge — outside the loop, where
`speakgate.py` and the supporter seam already live — and G19 exercises the class
law over a scripted multi-voice transcript instead of a live server.
