#!/usr/bin/env bash
# B13 DoD 3 / 11 §5's variant column: every gate family at V0 and at every launch
# state this build can produce. Wall-clock per run is recorded so OD-83's ask can
# be priced. Untracked tool.
cd "/c/Users/jakub/Documents/Qoder/2026-10-22/8c015168" || exit 1
H=./target/debug/si-harness.exe
SEED=${SEED:-0x51}

run_one() { # gate scenario variant [extra args...]
  local g=$1 s=$2 v=$3; shift 3
  local start end dir out rc
  start=$(date +%s%3N)
  out=$("$H" run --gate "$g" --variant "$v" --seed "$SEED" --scenario "harness/scenarios/$s" "$@" 2>&1)
  rc=$?
  end=$(date +%s%3N)
  dir=$(ls -dt gates/"${g}"_si_* 2>/dev/null | head -1)
  printf '%s\t%s\t%s\t%s\t%d\t%s\n' "$g" "$v" "$SEED" "$rc" "$((end - start))" "$dir"
}

SC_G1=g1_write.txt; SC_G2=g2_paraphrase.txt; SC_G3=g3_supersession.txt
SC_G4=g4_contradiction.txt; SC_G6=g6_layer.txt; SC_G7=g7_reparse.txt
SC_G8=g8_curiosity.txt; SC_G9=g9_arousal.txt; SC_G10=g10_idle.txt
SC_G12=g12_grimm.txt; SC_G13=g13_sleep.txt; SC_G15=g15_fruit.txt
SC_G16=g16_warm.txt; SC_G17=g17_origin.txt; SC_G18=g18_compare.txt
SC_G19=g19_server.txt

echo -e "gate\tvariant\tseed\tms_run\trc\trun_dir"
for g in G1 G2 G3 G4 G6 G7 G8 G9 G10 G12 G13 G15 G16 G17 G18 G19; do
  eval "sc=\$SC_$g"
  run_one "$g" "$sc" V0
done

# 05 §2's launch-state rows: the variants one shipped build can produce.
run_one G10 "$SC_G10" V2
run_one G10 "$SC_G10" V3
run_one G12 "$SC_G12" V4
run_one G12 "$SC_G12" V5
run_one G13 "$SC_G13" V9
run_one G17 "$SC_G17" V12
run_one G19 "$SC_G19" V13
