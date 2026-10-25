# §26 — THE WORLD: N voices, delegated authority (the I3 amendment round; v2 arc — folded law 2026-10-23; G19, OD-123…OD-132)

> Status: **FOLDED AS LAW 2026-10-23** by the owner's order to engineer the v2
> arc and build it. The HOW is [engineering/12](../engineering/12-v2-arc.md); the gate family
> is **G19** and the open items are **OD-123 … OD-132**. The draft's own rule
> still binds item by item: an unrulled OD item is not a licence for the build
> to choose, so wherever the build could not stop for a ruling it ran the
> **least-authority option**, said so in §26.13, and left the ruling with the
> owner — the OD-95 discipline, never a silent default.
>
> Numbering per §23.9: this doc takes **OD-123…OD-132 / G19**. It is the only
> proposal in the v2 arc that amends an invariant's *wording* — I3, following
> I3's own OD-35 precedent: "one channel, more than one voice" becomes "one
> world, more than one voice, authority owner-origin".

## 26.0 Thesis line

The deployment is now ruled by the owner: **one SI, born on a Discord
server, in conversation with 10+ users from t=0.** The honest engineering
answer to "we break the law" is narrower and stronger than breaking:
**the channel law generalizes, the teacher law delegates.**

- **The channel law generalizes:** I3 was already amended once (OD-35) from
  one voice to two; the server is the same amendment at N: one channel, N
  identity-tagged voices. Discord is not a second channel, it is the
  panel's transport (the gateway is an adapter; "one channel" survives as
  "one world").
- **The teacher law delegates:** authority remains owner-origin and is
  *granted*, never seized. A stranger with no grant is grimm-class: sense
  only, zero authority (the entire I10 template applies line for line:
  may light the walk, raise signals, deposit silt; may never create a
  belief row, auto-commit, correct, form or bake an edge, bump confidence,
  or confirm). A user the owner trusts holds exactly the rights the owner's
  grant row says, revocable, citable, append-only. **Every act of authority
  in the system traces to an owner act or to a grant that traces to one.**

Litmus: born on the server, 10+ users talking, owner silent: **no state
byte changes.** Strangers' lines light walks and deposit silt and write
nothing. The owner's first line does everything it does today, in public,
with strangers in the room. And at t=0 SI is structurally mute toward
everyone (the grimm birth argument, generalized: empty lexicon, nothing
citable, nothing to say).

## 26.1 What survives, what generalizes, what is genuinely new

| law | status under the amendment |
|-----|----------------------------|
| I1, I2, I4, I5, I6, I7, I8, I9 | **survive untouched** (the substrate does not know how many voices the panel has) |
| I3 | **amended wording (I3A)**: one channel becomes one world, N identity-tagged voices, priority law unchanged (an owner line preempts everything, always) |
| I10 | **generalized (I10A)**: grimm was the first proof that a voice can be in the room and out of the loop; strangers are that proof at scale, trusted users are it for everything outside their grant |

Genuinely new problems, none of which the docs have yet, all of which this
round must spec: **attention** (26.4), **ask routing** (26.5), **per-user
state** (26.6), **idle starvation on a busy server** (26.4), and
**retention of third-party lines** (26.8).

## 26.2 Identity and class (two fields, one derived)

- `speaker_identity` (the Discord user id) is stored per line at S0
  (schema extension, OD-123). The identity is metadata on sense, never a
  belief: SI knowing *who someone is* is content, and content arrives
  through the same gates as everything else.
- `speaker_class` (`owner | grimm | trusted | stranger`) is **computed at
  generation from the grant table and the owner line, stored nowhere** (the
  §20 median precedent, R-28/R-29 style): a derived class cannot be
  smuggled into a row, only re-derived. The write gate consults the class
  function exactly as it consults `source` today; the `prov_speaker` field
  gains its class-leg and the G12(e)-style grep grows one arm (G19(b)).

## 26.3 The tiers (the central ruling, OD-124)

- **(a) World as subject (birth state, recommended):** strangers never
  teach; every non-owner line is grimm-class. SI still learns *from* the
  server, through crystallize: recurrences across many users' lines are
  nominated like any other ("three different people keep telling me the
  tomato is a fruit — is that so?"), and only the owner's word mints
  anything. The world becomes experience; the owner stays the only teacher.
- **(b) Granted trust (a later owner act, one user at a time):** the owner
  may grant rights, and the grant itself is a lattice row (owner-act kind,
  citable, revocable, append-only): `/trust @user teach|correct|answer`.
  Trusted-tier rows carry provenance `user:X via grant G`. Arbitration is
  **tier-strict**: owner outranks trusted, trusted outranks stranger-sense;
  an owner correction supersedes everything; a trusted user's correction
  supersedes only trusted-tier rows, never owner rows. Answer-to-ask
  auto-commit (§1.4, §17.2.3) may be granted (`answer`); nomination
  confirmation (§22/§25) stays owner-only in v2.
- **(c) Full peer teaching: REJECTED.** A class that teaches itself is
  provenance laundering with a bigger room (the §21.4 / **live** OD-36 line:
may a Grimm line form an edge — the fold took sense-only); every
  row's authority must trace to an owner act, or the audit trail that the
  whole design is built on stops being a trail.

Recommendation: born at (a); (b) only after the speakgate filters are
proven in the wild, per-user, one grant at a time, smallest right first.

## 26.4 Attention and the busy world

- **Senses-quiet redefined:** per-thread quiet, not global. A busy server
  never goes globally quiet, so idle ignition as spec'd would starve.
- **Scheduled idle windows:** the sleep doc's phases (P3/P4 calm
  reprocessing) gain a window: owner-set intervals of the **logical** axis in
  which quiet threads may ignite (OD-126). Idle never preempts an owner line
  (unchanged). *Engineering note, folded in:* the draft said "calendar", and a
  wall-clock calendar is banned for anything that reaches state — I5 and R-29
  put every trigger on the logical tick — so a window is expressed in ticks and
  may only **suppress** ignition, never cause it. A window that could *cause*
  thought would be a timer wearing a schedule, which is exactly what §19.4's
  rate law exists to forbid.
- **Response policy:** SI speaks when addressed (@SI, DM) or when a drive
  ask fires; per-channel output budget; the owner preempts as always.
- **Silt goes per-conversation:** simultaneous threads get simultaneous
  working sets; silt's scope ruling (live **OD-109**, this arc's [docs/23](23-context.md)
  §23.8 row) gains option (c): per-thread tables,
  global decay, an @SI in another thread switches the active table
  (OD-127).

## 26.5 Ask routing and nomination visibility

Asks visible in-channel will be answered by strangers; per 26.3 that is
harmless (their answers are sense, never commits) but noisy. Routing
(OD-128): nomination asks (crystallize §22, skills §25) land in the
owner-side lane (a DM or a #si-teaching channel whose confirmation rule is
owner-only no matter who types there). The queue rule and integer cap are
unchanged; a busier world may not buy more asks.

## 26.6 People as rows (not a parallel system)

Per-user state is ordinary rows through ordinary gates: who taught what
(provenance carries it once 26.3(b) lands), per-thread silt warmth,
per-user stamp medians (§20 keyed per identity: "this user's corrections
arrive hot, that one teaches calm" — plasticity's existing machinery,
keyed, OD-129). A belief *about* a person goes through the same gates as a
belief about apples: a stranger cannot teach "X is a liar"; the owner can;
crystallize may *nominate* "this user's claims keep contradicting" as a
pattern-ask (a §22-round-2 nomination kind over provenance identity,
listed as unruled, not built).

## 26.7 Grimm on the server (OD-130)

He was never in the loop, so the world does not move him. Options: retire;
keep as one more tagged voice; or demote his filters to their true role:
the template the stranger/trusted filters reuse if (b) lands
(speakgate-style token membership over the taught lexicon, G19(h)).

## 26.8 Retention of third-party lines (OD-131, the one honest law-bend on the table)

S0 keeps every line verbatim, forever (I4). On the owner's own server that
is his policy call, and this doc surfaces the only place a bend is even
discussable: an ops-side retention window that ages *stranger* S0 lines
into digests, never belief rows, never owner lines, never provenance.
Recommendation: **keep everything** (the transcript is the audit trail;
storage is the cheap part), and note the window exists if the owner ever
needs it. Ruled, not defaulted.

## 26.9 Invariant impact audit

| invariant | impact | why it holds |
|-----------|--------|--------------|
| I1 | none | nothing innate ships about users or servers; identity is metadata on sense |
| I2 | none | the baseline is untouched; the server is lived experience, not content installation |
| I3 | **amended (I3A), load-bearing** | one channel becomes one world; the priority law and the source-decides-authority law are unchanged, and the amendment is ledgered like OD-35 was |
| I4 | none (OD-131 names the one possible bend, recommend against) | all rows append-only; grants and revocations are rows |
| I5 | none | identity-tagged transcripts replay byte-identically; class is re-derived, never stored |
| I6 | none | labels are honest toward strangers too; a stranger gets the same FACT/INFERENCE/SPECULATION discipline the owner does |
| I7 | none | gaps still produce asks; strangers' answers to asks are sense |
| I8 | none | signals fire from strangers' lines exactly as from grimm's (perturbation); nothing new reads `a` |
| I9 | none | idle still writes nothing; scheduled windows change *when* idle runs, not what it may do |
| I10 | **generalized (I10A)** | the grimm template is the stranger's law; granted trust is the same law with a scope list; every grant traces to an owner act |

## 26.10 Gates — G19

- **G19(a) stranger-silence:** scripted 10-user server, owner silent, N
  stranger lines: **zero belief-row writes** — no `s_r_pairs`, no `facts`, no
  `links`, no `skills`, no edge, no bake, no confidence movement. `context`
  sense rows still append, exactly as Grimm's do and for the same documented
  reason (§21.4: `context` is the sense record, not a belief, and it carries no
  charge and never answers a FACT); session state may move; **rows of the
  protected class do not.** A run that suppresses `context` too is not stricter,
  it is a different design: replay determinism (§21.7) depends on those rows.
- **G19(b) class-derivation:** every write's speaker-class re-derives from
  grants + owner lines; a stored class anywhere is a red.
- **G19(c) grant-trace:** every trusted-tier write cites a live grant row;
  revoke → the user's next line is stranger-class; pre-revoke rows keep
  provenance (I4) but lose arbitration standing at NOW.
- **G19(d) answer-to-ask owner-only (or granted):** stranger/ungranted
  answers to a pending ask produce zero commits.
- **G19(e) determinism:** identity-tagged transcript replay byte-identical.
- **G19(f) routing:** nominations land owner-side only.
- **G19(g) starvation:** scheduled idle windows fire on a scripted busy
  server; owner preemption intact.
- **G19(h) speakgate-for-grants:** if (b) lands, trusted-user lines pass
  the same token-membership filters grimm's lines pass.

## 26.11 Open decisions introduced

| OD | question |
|----|----------|
| OD-123 | the identity schema: `speaker_identity` at S0, the `prov_speaker` extension, and whether grant rows live in L3 or a gated shelf |
| OD-124 | the tiers ruling: (a) at birth (recommended), the (b) grant menu and its smallest-first order, (c) rejected as written |
| OD-125 | grant semantics: shape, revocation, and whether a revoked user's rows keep arbitration standing against *later* strangers (recommended: no) |
| OD-126 | scheduled idle windows: owner-set windows on the **logical** axis (§26.4's engineering note), a budget per window, and the interaction with sleep's P3/P4 — which is already bounded, because sleep is counter-triggered and a window cannot make a night happen |
| OD-127 | per-thread silt (OD-109 option (c)): table per conversation, global decay, active-table switching — **the build already keys the table by conversation**, so this is a scope decision, not a new mechanism |
| OD-128 | ask routing and nomination visibility: owner-lane (recommended) vs public-with-owner-confirm |
| OD-129 | per-user plasticity keying: stamps and medians keyed per identity, and what a *user-facing* readout may say about another user (honest-label law, applied to people) |
| OD-130 | Grimm's fate on the server |
| OD-131 | the S0 retention window for stranger lines (recommended: never; the bend stays on the table, ruled) |
| OD-132 | the demo-trace fixture becomes multi-voice: the §4 walkthrough rerun with strangers in the room, asserting the same first-commit moment (only the owner's line ever writes) |

## 26.12 What this document does NOT claim

- No peer teaching by default, no second channel, no new sense.
- No reopening of learning-to-learn (the owner's ruling: last, after
  prototyping survives; skills §25 and this doc both leave it shut).
- No claim that the world becomes SI's teacher. The world becomes SI's
  *subject*; who may teach it is a list the owner writes, one grant at a
  time, and the list is rows.

## 26.13 What the fold changed, and what the build runs at

Four edits, one of them a scope line the owner should read before B13:

1. **Numbering** per §23.9: OD-120…OD-129 → **OD-123…OD-132**, and the silt
   scope ruling this doc's per-thread option bends is **OD-109**.
2. **OD-35 and OD-36 in §26.3 are *live* ledger seats**, not this round's —
   annotated inline so nobody "fixes" them into the new range.
3. **"Calendar" → logical windows**, per §26.4's engineering note. A wall-clock
   trigger would break I5 and R-29 on its own; the fold ships the schedule on
   the logical axis with suppress-only authority.
4. **Scope, stated once and honestly: tonight's build ships the *law*, not the
   network.** Identity tagging, derived class, grant rows, revocation,
   tier-strict arbitration, nomination lanes and logical windows are in-loop
   machinery and G19 exercises them over a scripted multi-voice transcript. The
   Discord adapter is a **transport at the panel's edge** — outside the loop,
   where `speakgate.py` and the supporter seam already live — and it is not
   built here: it needs credentials, a token, and an owner who is awake. Naming
   that gap is the honest move; calling the class law a gateway is the
   dishonest one. **OD-132** holds the multi-voice demo fixture, which tonight
   produces as a harness scenario, not as a walkthrough of a live server.

**Declared build points.**

- **OD-124 ships tier (a) and nothing else**: `WORLD_TIERS = owner-only` is the
  species default, every non-owner line is grimm-class, and `speaker_class` is
  **computed at generation from the grant table plus the owner's lines and
  stored nowhere** — the §20-median precedent, R-28 style. Under tier (a) no
  non-owner line ever writes a row, so identity never enters a row and **the
  80-byte envelope does not change**. That is the most important fact in this
  section: an Envelope field would re-baseline every stored record since B1. So
  **OD-123** ships as: identity lives in the sense row's *stream* — the event
  body and the transcript line — which is where a perturbation's metadata
  already lives.
- **OD-125 (`/trust`) is coded, not granted.** The grant-row kind, the
  revocation law and tier-strict arbitration exist and G19(c) gates them,
  because the machinery has to be honest before a first grant is even possible;
  the birth state holds **zero** grant rows, and a build that handed out a grant
  to make a gate interesting would be the laundering §26.3(c) rejects.
- **OD-128 ships owner-lane**: every nomination ask carries a lane and the
  only legal value for a nomination is the owner lane. **The lane is the ask's
  `origin` value, not a column** — a nomination is born from a *nomination*, and
  that fact is already the byte the row has, so `origin = nomination` **is** the
  lane field and a new field would be a schema change this section just refused
  (the encoding is [engineering/12 §2.4](../engineering/12-v2-arc.md);
  `check_nomination_owner_lane` is the leg). The queue rule and the
  integer ceiling are untouched — "a busier world may not buy more asks" is a
  leg, not a suggestion.
- **OD-126 ships windows that can only suppress.** Closed means no idle tick;
  open does not mean an idle tick happens, because the quiet-and-empty-agenda
  law still decides.
- **OD-106 (silt's deposit-source policy) ships full-weight parity** — owner and
  non-owner lines deposit silt and light walks at the same weight, because silt
  tracks thinking, not teaching, and confers zero authority. A differential
  would have to name the laundering path it prevents, and none exists.
- **OD-131 ships "keep everything"** — no retention window exists in the code at
  all, so the bend is not merely declined but absent; a future digest op is an
  owner act, not a cleanup.
- **OD-130 stays open and unreified**: Grimm's fate on the server. Tonight's
  build keeps him exactly as G12 and B9 leave him — one tagged voice, zero
  authority, every gate passing with him disconnected — and demotes nothing.
