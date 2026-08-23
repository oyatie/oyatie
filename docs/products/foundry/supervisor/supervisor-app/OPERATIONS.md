---
doc_class: Runbook
purpose: "Operational runbook for starting daemon, tuning watchdog, triage procedures"
owner_team: axis-foundry
status: draft
doc_status: published
---

# Supervisor App — Operations

## Starting the Daemon

### Quick Start

```bash
# 1. Set configuration
export OYATIE_SUPERVISOR_MAX_IN_FLIGHT=12
export OYATIE_SUPERVISOR_WATCHDOG_TIMEOUT_SECS=300

# 2. Create config directories
mkdir -p ~/.oya/supervisor
mkdir -p /var/log/oya/supervisor

# 3. Start daemon (foreground, for debugging)
RUST_LOG=info intelligence-supervisor

# 4. In another terminal, check health
curl http://localhost:8080/health
```

### Systemd Service

```ini
# /etc/systemd/system/intelligence-supervisor.service
[Unit]
Description=Oyatie Foundry Supervisor Daemon
After=network-online.target

[Service]
Type=simple
User=oya
Group=oya
WorkingDirectory=/opt/oya
EnvironmentFile=/etc/oya/supervisor.env
ExecStart=/usr/local/bin/intelligence-supervisor
Restart=always
RestartSec=5s
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

```bash
# Enable and start
sudo systemctl enable intelligence-supervisor
sudo systemctl start intelligence-supervisor
sudo systemctl status intelligence-supervisor

# View logs
journalctl -u intelligence-supervisor -f
```

## Configuration

### Environment Variables

| Variable | Default | Meaning |
|----------|---------|---------|
| `OYATIE_SUPERVISOR_MAX_IN_FLIGHT` | 12 | Concurrent sessions per tick |
| `OYATIE_SUPERVISOR_WATCHDOG_TIMEOUT_SECS` | 300 | SIGKILL after timeout |
| `OYATIE_SUPERVISOR_SETTINGS_RENDERER_MODE` | Disabled | VerifyOnly / Reconcile |
| `OYATIE_SUPERVISOR_SETTINGS_VERIFY_DEBOUNCE_SECS` | 60 | Drift cache TTL |
| `OYATIE_SUPERVISOR_MINIMUM_ELIGIBLE_ACCOUNTS` | 1 | Min accounts before DriftExcluded |
| `OYATIE_CONFIG_PATH` | `~/.oya` | Config directory |
| `RUST_LOG` | info | Tracing filter (debug, info, warn, error) |

### TOML Config File

```toml
# ~/.oya/supervisor.toml
[supervisor]
max_in_flight = 12
watchdog_timeout_secs = 300
settings_renderer_mode = "Disabled"  # or "VerifyOnly" or "Reconcile"
settings_verify_debounce_secs = 60
minimum_eligible_accounts = 1

[tracing]
level = "info"
format = "json"  # or "compact"

[inbox]
path = "~/.oya/inbox"
dead_letter_path = "~/.oya/inbox/dead-letter"

[outbox]
path = "~/.oya/outbox"
```

## Watchdog Tuning

The watchdog enforces a timeout on every spawned session. To tune:

### Scenario 1: Sessions Timing Out Prematurely

**Symptom:** `TickOutcome::Quarantined` after 5 minutes, even though CLI is responsive.

**Diagnosis:**
```bash
# Check watchdog timeout
echo $OYATIE_SUPERVISOR_WATCHDOG_TIMEOUT_SECS  # shows 300

# Check actual session duration
jq '.[] | select(.event_class == "foundry_supervisor_session_duration_micros") | .duration_micros' \
  evidence/audit-chain.jsonl | \
  awk '{sum+=$1; sumsq+=$1*$1; n++} END {print "avg:", sum/n, "p99:", ...}'
```

**Fix:** Increase timeout. If p99 duration is 180s, set `WATCHDOG_TIMEOUT_SECS=240`.

```bash
export OYATIE_SUPERVISOR_WATCHDOG_TIMEOUT_SECS=240
systemctl restart intelligence-supervisor
```

### Scenario 2: Hung Sessions Not Being Killed

**Symptom:** Inbox fills up; memory usage grows; no `foundry_supervisor_session_killed` audit events.

**Diagnosis:**
```bash
# Check if watchdog task is spawning
jq '.[] | select(.event_class == "foundry_supervisor_watchdog_spawn")' \
  evidence/audit-chain.jsonl | wc -l

# Check system for hung processes
ps aux | grep intelligence-supervisor | grep -v grep
ps aux | grep "Claude\|claude" | wc -l  # spawned session count
```

**Fix:** Watchdog may not be running due to a bug. Check logs:

```bash
journalctl -u intelligence-supervisor --since="5 min ago" | grep watchdog
```

If no watchdog messages, restart the daemon.

## Dead-Letter Triage

Messages that fail to process are moved to `dead-letter/`. To triage:

```bash
# List dead-lettered messages
ls -la ~/.oya/inbox/dead-letter/

# Check reason file
cat ~/.oya/inbox/dead-letter/msg-abc123.reason

# Re-queue a message
mv ~/.oya/inbox/dead-letter/msg-abc123 ~/.oya/inbox/
```

## Settings Drift Triage

If `TickOutcome::DriftExcluded` is happening:

```bash
# Check drift report
cat .omc/state/settings-drift-report.json | jq '.[] | select(.state != "Match")'

# If RendererMode is VerifyOnly, manually reconcile
export OYATIE_SUPERVISOR_SETTINGS_RENDERER_MODE=Reconcile
systemctl restart intelligence-supervisor

# Wait for next tick
sleep 2

# Check if drift is resolved
cat .omc/state/settings-drift-report.json | jq 'all(.state == "Match")'
```

## Health Checks

### HTTP Health Endpoint

```bash
# Liveness
curl -v http://localhost:8080/health

# Expect 200 OK
# {"status": "ok", "uptime_seconds": 3600}
```

### Metrics Endpoint

```bash
# Prometheus metrics
curl http://localhost:8080/metrics

# Expect Prometheus format:
# foundry_supervisor_inbox_depth{account_id="..."} 42
# foundry_supervisor_idle_ticks_total 1234
```

### Manual Smoke Test

```bash
# Inject a test message
curl -X POST http://localhost:8080/inbox \
  -H 'Content-Type: application/json' \
  -d '{
    "account_id": "test-account-uuid",
    "provider_family": "Claude",
    "request_json": "{ ... }"
  }'

# Watch logs for processing
journalctl -u intelligence-supervisor -f | grep "foundry_supervisor_spawn"

# Verify spend record was written
tail -5 ~/.oya/outbox/spend-records.jsonl
```

## Troubleshooting

| Symptom | Root Cause | Fix |
|---------|-----------|-----|
| "Max in-flight exceeded" warnings | Too many concurrent sessions | ↓ `max_in_flight` |
| "Inbox locked" errors | Previous peek_lock TTL didn't expire | Wait 60s or manually clear lock file |
| "No eligible accounts" (frequent) | All accounts in drift-excluded state | Check settings; reconcile |
| OOM (memory leak) | Hung sessions accumulating | Restart; investigate session driver |
| High latency (>250ms per tick) | Routing policy or snapshot slow | Profile with flamegraph |

## References

- **Config:** v4 Plan § B.5 (SupervisorConfig struct)
- **Watchdog:** v4 Plan § B.4 step 12
- **Signal handling:** Linux man pages: signal(7), sigterm(7)
