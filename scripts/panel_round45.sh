#!/usr/bin/env bash
# Rounds 4 and 5, after §19(f)'s guard half was scoped to one launch configuration.
# A record only counts toward the comparison once it carries `launch_digest`, so
# round 4 writes the field and round 5 is the round that can grade it.
set -u
cd "$(dirname "$0")/.."
BIN=./target/debug/si-harness
SEEDS=$(python -c "
import json
print('\n'.join(json.load(open('harness/scenarios/g14_seed_panel.json',encoding='utf-8'))['seeds']))
")
for round in 4 5; do
  echo "=== round $round ==="
  for seed in $SEEDS; do
    out=$($BIN run --gate G14 --seed "$seed" --scenario harness/scenarios/g14_twins.txt 2>&1)
    verdict=$(echo "$out" | head -1 | grep -o 'PASS\|FAIL')
    unresolved=$(echo "$out" | grep -o 'unresolved: [0-9]*' | head -1)
    echo "$round $seed ${verdict:-ERROR} $unresolved"
    if [ "$verdict" != "PASS" ]; then echo "$out" | tail -16; fi
  done
done
