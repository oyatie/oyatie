#!/usr/bin/env bash
# oci-a1-capacity-retry.sh — keep retrying `tofu apply -var=create_stage0_a1=true`
# until A1.Flex capacity opens in ap-chuncheon-1/AD-1 and the instance creates.
#
# OCI A1 Always-Free shapes are highly contested in this region; capacity
# becomes available intermittently for ~seconds at a time. This loop drives
# the OpenTofu config (no hand-rolled `oci compute instance launch`) at a
# bounded poll cadence + exponential off-time on persistent failure.
#
# Usage:
#   bash /home/oyatie/projects/oyatie/scripts/oci-a1-capacity-retry.sh
#
# Or as a systemd user-timer (preferred long-running shape — see below).
#
# Exit codes:
#   0  apply succeeded (instance running)
#   2  permanent error (config invalid, auth, missing dep)
#   3  signaled exit
set -euo pipefail

TOFU=${TOFU_BIN:-/home/oyatie/.local/bin/tofu}
INFRA=${INFRA_DIR:-/home/oyatie/projects/oyatie/infra/oci}
LOG=${A1_RETRY_LOG:-/home/oyatie/projects/oyatie/evidence/oci-readiness/a1-capacity-retry.log}
MAX_ATTEMPTS=${A1_RETRY_MAX:-288}     # 288 × 5 min = 24h max
SLEEP_SECS=${A1_RETRY_SLEEP:-300}     # 5 minutes between attempts
SUCCESS_MARKER=${A1_SUCCESS_MARKER:-/home/oyatie/projects/oyatie/evidence/oci-readiness/a1-acquired.marker}

mkdir -p "$(dirname "$LOG")"

trap 'echo "[$(date -u +%FT%TZ)] signal received; exit 3" >> "$LOG"; exit 3' INT TERM

for i in $(seq 1 "$MAX_ATTEMPTS"); do
  ts=$(date -u +%FT%TZ)
  echo "[$ts] attempt $i/$MAX_ATTEMPTS: tofu apply -var=create_stage0_a1=true" >> "$LOG"
  if (cd "$INFRA" && "$TOFU" apply -auto-approve -no-color -var=create_stage0_a1=true >> "$LOG" 2>&1); then
    # Verify the instance is in state
    if "$TOFU" -chdir="$INFRA" state list 2>/dev/null | grep -q '^oci_core_instance\.stage0\[0\]$'; then
      ip=$("$TOFU" -chdir="$INFRA" output -raw stage0_public_ip 2>/dev/null || echo unknown)
      echo "[$ts] SUCCESS: A1 stage-0 acquired (ip=$ip) on attempt $i" >> "$LOG"
      printf "acquired_at=%s\nattempt=%d\npublic_ip=%s\n" "$ts" "$i" "$ip" > "$SUCCESS_MARKER"
      exit 0
    fi
  fi
  # Inspect the last error class. "Out of host capacity" is expected and
  # retried; any other 4xx/5xx is treated as permanent and the loop exits 2.
  if grep -q 'Out of host capacity' "$LOG"; then
    echo "[$ts] capacity miss; sleep ${SLEEP_SECS}s" >> "$LOG"
    sleep "$SLEEP_SECS"
    continue
  else
    last=$(tail -50 "$LOG")
    echo "[$ts] non-capacity error; exit 2" >> "$LOG"
    echo "$last" >&2
    exit 2
  fi
done

echo "[$(date -u +%FT%TZ)] exhausted $MAX_ATTEMPTS attempts; exit 1" >> "$LOG"
exit 1
