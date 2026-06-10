#!/bin/zsh
# slice-06 burndown driver — ADR-0540 / FRIC-1781063357 campaign (transient lane tooling, not shipped)
set -u
cd /Users/jasonlee/oyatie-worktrees/burndown-slice-06
SLICE=/Users/jasonlee/Developer/oyatie/.omc/ultragoal/burndown/slice-06.txt
LOG=slice06-progress.log
GENLOG=slice06-generator.log
TESTLOG=slice06-buck-test.log
: > "$LOG"
count=0
while IFS= read -r m; do
  [ -z "$m" ] && continue
  count=$((count+1))
  echo "=== [$count] $m" | tee -a "$LOG"
  echo "=== [$count] $m" >> "$GENLOG"
  out=$(buck2 run //tools/oya-buck-test-wiring-app:oya-buck-test-wiring -- --apply --limit 1 --root "$m" 2>&1)
  echo "$out" >> "$GENLOG"
  if echo "$out" | grep -qiE 'unsupported|skip'; then
    echo "UNSUPPORTED $m :: $(echo "$out" | grep -iE 'unsupported|skip' | head -1)" >> "$LOG"
    continue
  fi
  targets=$(git diff "$m/BUCK" | grep '^+    name = ' | sed 's/.*"\(.*\)".*/\1/')
  if [ -z "$targets" ]; then
    echo "NOCHANGE $m (no stanza added; inspect generator log)" >> "$LOG"
    continue
  fi
  member_ok=1
  for t in $targets; do
    echo "=== [$count] //$m:$t" >> "$TESTLOG"
    if buck2 test "//$m:$t" >> "$TESTLOG" 2>&1; then
      echo "PASS //$m:$t" >> "$LOG"
    else
      echo "FAIL //$m:$t" >> "$LOG"
      member_ok=0
    fi
  done
  if [ "$member_ok" = "1" ]; then
    git add "$m/BUCK"
    if [ $((count % 10)) -eq 0 ]; then
      git commit -q -m "test(buck): wire rust_test targets, slice-06 WIP through member $count (ADR-0540, FRIC-1781063357)" || true
      echo "COMMIT at member $count" >> "$LOG"
    fi
  fi
done < "$SLICE"
echo "DONE all members" >> "$LOG"
