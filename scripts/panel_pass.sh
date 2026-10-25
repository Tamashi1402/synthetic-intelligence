#!/usr/bin/env bash
# 05 §19's panel pass: every member of harness/scenarios/g14_seed_panel.json runs
# harness/scenarios/g14_twins.txt, twice, in separate processes — round 1 fills the
# index, round 2 grades it (§19(b)(d)(i) are cross-process pair legs).
set -u
cd "$(dirname "$0")/.."
BIN=./target/debug/si-harness
SEEDS=$(python -c "
import json
print('\n'.join(json.load(open('harness/scenarios/g14_seed_panel.json',encoding='utf-8'))['seeds']))
")
for round in 1 2; do
  echo "=== round $round ==="
  for seed in $SEEDS; do
    out=$($BIN run --gate G14 --seed "$seed" --scenario harness/scenarios/g14_twins.txt 2>&1)
    verdict=$(echo "$out" | head -1 | grep -o 'PASS\|FAIL')
    unresolved=$(echo "$out" | grep -o 'unresolved: [0-9]*' | head -1)
    echo "$round $seed ${verdict:-ERROR} $unresolved"
    echo "$out" | tail -14
  done
done
