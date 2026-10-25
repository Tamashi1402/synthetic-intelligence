#!/usr/bin/env bash
# Stage 3 (post-B12/B13.1) global acceptance battery. 05 §21's per-release-candidate
# set: every runnable family at V0, plus every launch-state variant this single
# build can produce. G11 has no evaluator (OD-194) and is named, not run; V1/V10/V11
# are build-graph exclusions the gates grade as UNMEASURED in-report. Untracked tool.
set -u
cd "$(dirname "$0")/.." || exit 1
H=./target/debug/si-harness.exe
SEED=${SEED:-0x51}

run_one() { # gate scenario variant seed [extra args...]
  local g=$1 s=$2 v=$3 sd=$4; shift 4
  local start end dir out rc first
  start=$(date +%s%3N)
  out=$("$H" run --gate "$g" --variant "$v" --seed "$sd" --scenario "harness/scenarios/$s" "$@" 2>&1)
  rc=$?
  end=$(date +%s%3N)
  dir=$(printf '%s\n' "$out" | sed -n 's/^  run dir: //p' | tail -1)
  first=$(printf '%s\n' "$out" | head -1)
  printf '%s\t%s\t%s\t%s\t%d\t%s\t%s\n' "$g" "$v" "$sd" "$rc" "$((end - start))" "$dir" "$first"
}

SC_G1=g1_write.txt; SC_G2=g2_paraphrase.txt; SC_G3=g3_supersession.txt
SC_G4=g4_contradiction.txt; SC_G5=g5_determinism.txt; SC_G6=g6_layer.txt
SC_G7=g7_reparse.txt; SC_G8=g8_curiosity.txt; SC_G9=g9_arousal.txt
SC_G10=g10_idle.txt; SC_G12=g12_grimm.txt; SC_G13=g13_sleep.txt
SC_G14=g14_twins.txt; SC_G15=g15_fruit.txt; SC_G16=g16_warm.txt
SC_G17=g17_origin.txt; SC_G18=g18_compare.txt; SC_G19=g19_server.txt
SC_G20=g20_tongue.txt

echo -e "gate\tvariant\tseed\trc\tms_run\trun_dir\tverdict_line"
for g in G1 G2 G3 G4 G5 G6 G7 G8 G9 G10 G12 G13 G14 G15 G16 G17 G18 G19; do
  eval "sc=\$SC_$g"
  run_one "$g" "$sc" V0 "$SEED"
done
# g20_tongue.txt pins seed 0x5151 in its own header (G14.j.1: a scenario states its individual).
run_one G20 "$SC_G20" V0 0x5151

# 05 §2's launch-state rows this one build can produce.
run_one G10 "$SC_G10" V2 "$SEED"
run_one G10 "$SC_G10" V3 "$SEED"
run_one G12 "$SC_G12" V4 "$SEED"
run_one G12 "$SC_G12" V5 "$SEED"
run_one G13 "$SC_G13" V9 "$SEED"
run_one G14 "$SC_G14" V8 0x0
run_one G17 "$SC_G17" V12 "$SEED"
run_one G19 "$SC_G19" V13 "$SEED"
run_one G20 "$SC_G20" V14 0x5151

# G11 is the named wall: the dispatcher has no evaluator (OD-194). Record the refusal.
printf 'G11\t-\t%s\t-\t-\t-\t%s\n' "$SEED" \
  "$("$H" run --gate G11 --variant V0 --seed "$SEED" --scenario harness/scenarios/g11_composition.txt 2>&1 | head -1)"
