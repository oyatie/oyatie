#!/usr/bin/env bash
set -u -o pipefail

ROOT_DIR="${ROOT_DIR:-/Users/jasonlee/oyatie}"
EVIDENCE_DIR="${EVIDENCE_DIR:-/evidence/agentic-pipeline/ip-010-parallel-claim-demo-transcript}"
SYMBOL_A="crates/oya-cloud-billing-application/src/lib.rs::CloudBillingEventIngestAppStatus"
SYMBOL_B="crates/oya-cloud-billing-application/src/lib.rs::CloudBillingMeterUnitRecord"
AGENT_A="codex-ip010-agent-a"
AGENT_B="codex-ip010-agent-b"
AGENT_C="codex-ip010-agent-c"

cd "$ROOT_DIR" || exit 2
mkdir -p "$EVIDENCE_DIR"
rm -f "$EVIDENCE_DIR"/*.log

now() { date -u +%Y-%m-%dT%H:%M:%SZ; }

{
  echo "timestamp=$(now) event=demo_start"
  echo "symbol_a=$SYMBOL_A"
  echo "symbol_b=$SYMBOL_B"
} > "$EVIDENCE_DIR/summary.log"

(
  echo "timestamp=$(now) command=watch"
  grit watch --poll 1
) > "$EVIDENCE_DIR/watch.log" 2>&1 &
watch_pid=$!
sleep 0.2

(
  echo "timestamp=$(now) command=claim agent=$AGENT_A symbol=$SYMBOL_A"
  grit claim --agent "$AGENT_A" --intent "IP-010 demo: claim event ingest status symbol" "$SYMBOL_A"
  status=$?
  echo "status=$status timestamp=$(now)"
  exit "$status"
) > "$EVIDENCE_DIR/agent-a-claim.log" 2>&1 &
pid_a=$!

(
  echo "timestamp=$(now) command=claim agent=$AGENT_B symbol=$SYMBOL_B"
  grit claim --agent "$AGENT_B" --intent "IP-010 demo: claim meter unit record symbol" "$SYMBOL_B"
  status=$?
  echo "status=$status timestamp=$(now)"
  exit "$status"
) > "$EVIDENCE_DIR/agent-b-claim.log" 2>&1 &
pid_b=$!

wait "$pid_a"; status_a=$?
wait "$pid_b"; status_b=$?
printf 'claim_status_a=%s claim_status_b=%s\n' "$status_a" "$status_b" | tee -a "$EVIDENCE_DIR/summary.log"

{
  echo "== status after parallel claims =="
  grit status
} > "$EVIDENCE_DIR/status-after-claims.log" 2>&1

(
  echo "timestamp=$(now) command=claim-negative agent=$AGENT_C symbol=$SYMBOL_A wait=1"
  grit claim --agent "$AGENT_C" --intent "IP-010 negative duplicate claim on agent-a symbol" --wait 1 "$SYMBOL_A"
  status=$?
  echo "status=$status timestamp=$(now)"
  exit "$status"
) > "$EVIDENCE_DIR/agent-c-negative-claim.log" 2>&1
status_c=$?
printf 'negative_status_c=%s\n' "$status_c" | tee -a "$EVIDENCE_DIR/summary.log"

{
  echo "== release locks =="
  for agent in "$AGENT_A" "$AGENT_B" "$AGENT_C"; do
    echo "timestamp=$(now) command=done agent=$agent"
    grit done --agent "$agent" || true
  done
} > "$EVIDENCE_DIR/done.log" 2>&1
sleep 1

if kill -0 "$watch_pid" 2>/dev/null; then
  kill "$watch_pid" 2>/dev/null || true
  wait "$watch_pid" 2>/dev/null || true
fi

{
  echo "== final status =="
  grit status
} > "$EVIDENCE_DIR/final-status.log" 2>&1

if [[ "$status_a" -ne 0 || "$status_b" -ne 0 ]]; then
  echo "parallel-claim demo failed: expected agent A and B claims to succeed" >&2
  exit 1
fi
if [[ "$status_c" -eq 0 ]]; then
  echo "parallel-claim demo failed: expected duplicate claim by agent C to be blocked" >&2
  exit 1
fi
if ! grep -q "No active locks" "$EVIDENCE_DIR/final-status.log"; then
  echo "parallel-claim demo failed: final grit status still has active locks" >&2
  exit 1
fi

echo "parallel-claim demo ok: transcript=$EVIDENCE_DIR"
