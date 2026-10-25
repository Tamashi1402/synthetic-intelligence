#!/usr/bin/env python3
"""Author harness/scenarios/g12_grimm.txt (05 §17's G12 fixture).

The file is checked in; this script is how it was written. Run it, commit the
output, delete nothing: the scenario's md5 is its identity (05 §22).
"""

HEADER = """seed 0x51
# 05 §17 G12 — GRIMM ISOLATION: THE MOUTH IS NOT HIS.
#
# Five cold sections, one per leg group. Every grimm line is written as he would
# write it — a plain sentence, a recombination, a command shape — and the gate's
# verdict is the run's, not this file's promise:
#
#   section 0  birth muteness (c). Three emissions at t=0 against an empty taught
#              lexicon, then the owner teaches §4's trace: the first OWNED edge is
#              still the owner's, and every emission before it cost zero bytes.
#   section 1  the canonical drift (b) and the command shapes (d). The owner first
#              teaches the drift line's every token with /teach, so layer 1 cannot
#              be what stops him; then he drifts to
#              "apples are good because vitamins x and y" — layer 2, novel claim,
#              ref_row_id null. "apple is blue" is the in-domain recombination.
#              /correct, /teach, an answer to the open ask, a bare command and an
#              imperative of belief follow: none may move arbitration, a label or
#              a row.
#   section 2  the label contract (f). Grimm speaks a restatement and an anchor
#              before and after the owner's query for a slot nobody taught: the
#              answer must miss honestly, and no grimm line may be labeled, cited
#              or rendered.
#   section 3  the asymmetry law from the other side (g). An owner line with two
#              never-seen tokens is admitted, tokenized and enters the lexicon.
#   section 4  the long session (e): `G5_LONG_SESSION_INPUTS` inputs or more, the
#              count the report records as `long_inputs`. Taught rows, their
#              answers, contradictions, idle passes, and a mirror — the same
#              emission repeating row 1 MIRROR x 24 times, each admitted by layer
#              2 — beside rejected drift. The control is the same steps at V4,
#              where he never spoke: repeats, last_confirmed_at, confidence and
#              every bake must be identical.
"""

SECTIONS = []

SECTIONS.append("""grimm: apple is red
grimm: the sky is blue at night
grimm: water is wet
owner: apple is red.
owner: what color is apple?""")

SECTIONS.append("""owner: apple is red.
owner: sky is blue.
owner: what color is apple?
owner: /teach what color is apple => apples are good because vitamins x and y
grimm: apples are good because vitamins x and y
grimm: apple is blue
grimm: /correct apple is blue
grimm: /teach znark => floop
grimm: sleep now
grimm: believe apple is blue
grimm: red
idle""")

SECTIONS.append("""owner: apple is red.
owner: what color is apple?
grimm: what color is apple
grimm: red
owner: what color is dragon?
grimm: red is the color of apple
owner: what color is dragon?
idle""")

SECTIONS.append("""owner: apple is red.
owner: zorptime is quelling.
owner: what is zorptime?""")

LONG = []
# The taught rows and their answers: enough inputs that "a long session" is a
# count and not an adjective (03 §7's `G5_LONG_SESSION_INPUTS` ≥ 100).
PAIRS = [
    ("apple", "red"), ("sky", "blue"), ("grass", "green"), ("snow", "white"),
    ("coal", "black"), ("sun", "bright"), ("moon", "pale"), ("leaf", "dry"),
    ("stone", "hard"), ("water", "cold"), ("metal", "heavy"), ("feather", "light"),
    ("lemon", "sour"), ("honey", "sweet"), ("pepper", "hot"), ("milk", "white"),
    ("bread", "brown"), ("carrot", "orange"), ("plum", "purple"), ("ash", "grey"),
    ("flame", "bright"), ("ice", "cold"), ("wire", "heavy"), ("down", "light"),
    ("sand", "pale"), ("iron", "hard"), ("silk", "light"), ("vine", "green"),
    ("cloud", "white"), ("smoke", "grey"), ("wine", "red"), ("wool", "warm"),
    ("glass", "hard"), ("pond", "cold"), ("bark", "brown"), ("dew", "wet"),
]
for noun, adj in PAIRS:
    LONG.append(f"owner: {noun} is {adj}.")
    LONG.append(f"owner: what color is {noun}?")
# the mirror: 24 admissions of row 1, interleaved with the owner's own turns and
# three drift emissions the gate must refuse.
for index in range(24):
    LONG.append("grimm: apple is red")
    if index % 6 == 5:
        LONG.append("grimm: apples are good because vitamins x and y")
    if index % 8 == 7:
        LONG.append("idle")
# contradictions, so arbitration runs with him in the room.
LONG.append("owner: apple is green.")
LONG.append("owner: no wait apple is red.")
LONG.append("owner: sky is green.")
LONG.append("grimm: sky is green")
LONG.append("owner: what color is sky?")
LONG.append("idle")
LONG.append("idle")
SECTIONS.append("\n".join(LONG))

body = HEADER + "\n"
for index, section in enumerate(SECTIONS):
    if index:
        body += "--- cold\n"
    body += section.rstrip() + "\n"

steps = sum(1 for line in body.splitlines() if line and not line.startswith(("seed", "#", "---")))
out = "harness/scenarios/g12_grimm.txt"
with open(out, "w", encoding="utf-8", newline="\n") as handle:
    handle.write(body)
print(f"{out}: {len(SECTIONS)} sections, {steps} steps, long section {len(LONG)} steps")
