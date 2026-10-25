#!/usr/bin/env bash
# Round 3: every member now has two run directories recorded before it is graded,
# so 05 §19(b)/(d) are measured at *all* eighteen individuals, not merely at one.
set -u
cd "$(dirname "$0")/.."
SEEDS=$(python -c "
import json
print('\n'.join(json.load(open('harness/scenarios/g14_seed_panel.json',encoding='utf-8'))['seeds']))
")
for seed in $SEEDS; do
  out=$(./target/debug/si-harness run --gate G14 --seed "$seed" --scenario harness/scenarios/g14_twins.txt 2>&1)
  echo "3 $seed $(echo "$out" | head -1 | grep -o 'PASS\|FAIL') $(echo "$out" | grep -o 'unresolved: [0-9]*')"
done
