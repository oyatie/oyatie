---
doc_class: Runbook
title: Audit emission backlog
microservice: cloud-secrets
owner_team: axis-cloud-secrets + axis-governance
date: 2026-05-17
severity_default: Sev-2 (Sev-1 if backlog >300s)
---

# Runbook: audit emission backlog

## When to use

- `cloud_secrets_audit_emission_backlog_seconds > 60` (Sev-2 page).
- Backlog > 300s (Sev-1 escalation; regulatory compliance risk).
- Bridge worker crash-looping.
- audit-chain µservice reporting degraded.

## §A — Diagnosis

### Step 1 — Confirm scope

```bash
# Backlog metric
cargo run -p cloud-secrets-audit-emitter-app -- backlog --pack <pack>

# audit-chain health
cargo run -p audit-chain-app -- health --pack <pack>

# Local audit-device file growth
kubectl -n cloud-secrets-<pack> exec openbao-0 -- du -sh /var/log/openbao/audit
```

### Step 2 — Identify root cause

| Symptom | Likely cause |
|---|---|
| backlog growing; audit-chain healthy | audit-emitter bridge worker degraded |
| backlog growing; audit-chain unhealthy | audit-chain µservice problem (see audit-chain runbook) |
| local file growth normal; bridge throughput dropped | network path issue between cloud-secrets cluster and audit-chain |
| backlog growing; spike correlated with resolve qps spike | DDoS / cardinality spike; audit emission scaling lag |

## §B — Common causes + fixes

### B.1 — Bridge worker degraded

```bash
kubectl -n cloud-secrets-<pack> get pods -l app=audit-emitter
kubectl -n cloud-secrets-<pack> logs <pod> --tail 200
```

Restart:

```bash
kubectl -n cloud-secrets-<pack> rollout restart deployment/audit-emitter
```

Scale up:

```bash
kubectl -n cloud-secrets-<pack> scale deployment audit-emitter --replicas 4
```

### B.2 — audit-chain µservice degraded

Engage axis-governance + invoke audit-chain runbook. cloud-secrets impact:
- Local audit-device file remains durable; backlog continues until audit-chain recovers.
- If audit-chain recovery > 1h: file at `evidence/audit-chain-degradation/<incident_id>.md` documenting impacted retention obligations.

### B.3 — Network path failure

Test connectivity:

```bash
kubectl -n cloud-secrets-<pack> exec <audit-emitter-pod> -- \
    nc -zv audit-chain-<pack>.svc.cluster.local 443
```

If failing: engage ops-net. Possible causes:
- NetworkPolicy regression.
- Service mesh policy regression.
- Cross-cluster TLS issue.

### B.4 — Resolve qps storm

If audit emission lag correlates with resolve qps spike:

```bash
# Resolve qps over last 1h
cargo run -p cloud-secrets-secret-reference-resolver-app -- qps --pack <pack> --window 1h
```

Possible causes:
- Legitimate workload spike → scale audit-emitter; investigate consumer behaviour.
- DDoS / abusive client → engage ops-security; apply per-tenant rate-limit.
- Cache-hit-rate drop → investigate why; SDK config regression possible.

## §C — Sev-1 escalation (backlog > 300s)

Per `incident-response.md`. Regulatory implications:
- KR PIPA Art. 29 + Enforcement Decree Art. 30 (audit retention) — temporary lag does not violate retention, but availability lag may compromise audit completeness if events are dropped.
- SOC 2 CC7.1 + CC7.2 (system monitoring) — audit-emission lag is itself a control deficiency.

Actions:
1. Sev-1 incident open.
2. Confirm local audit-device file is durable + capped > backlog size.
3. If local file approaches cap: emergency log-rotate + offload to object storage.
4. ops-legal + council-privacy informed of potential compliance lag.

## §D — Local audit-device file management

OpenBao writes audit events to a local file (`/var/log/openbao/audit/`). audit-emitter bridges this file to audit-chain. If the file fills up:

```bash
# Inspect
kubectl -n cloud-secrets-<pack> exec openbao-0 -- ls -lh /var/log/openbao/audit/

# Emergency rotate + offload
kubectl -n cloud-secrets-<pack> exec openbao-0 -- \
    /bin/openbao-audit-rotate.sh --offload-to s3://<pack>-audit-cold-bucket/
```

Audit-chain bridge will pick up the new file when capacity returns; offloaded file is re-injected via batch tool.

## §E — Catch-up after recovery

After audit-chain recovers:

```bash
# Verify bridge throughput exceeds backlog drain rate
cargo run -p cloud-secrets-audit-emitter-app -- bridge-throughput --pack <pack>

# Monitor backlog drain
watch 'cargo run -p cloud-secrets-audit-emitter-app -- backlog --pack <pack>'

# Confirm audit-chain has caught up
cargo run -p audit-chain-app -- verify-seal --pack <pack> --window "last 24h"
```

## §F — Post-mortem inputs

For Sev-1 (backlog > 300s): post-mortem must include:
- Total audit events in lag window.
- Whether any events were lost (should be zero given durable local file).
- Tenant notification analysis (rarely needed; audit lag is internal control matter, not breach).
- LEAN gate proposed: `check-audit-emission-throughput-sla` adds backlog metric to SLO authoring.

## Verification

```bash
# Backlog clears
cargo run -p cloud-secrets-audit-emitter-app -- backlog --pack <pack>
# Expect: < 5s

# Bridge throughput steady
cargo run -p cloud-secrets-audit-emitter-app -- bridge-throughput --pack <pack>

# Audit-chain integrity
cargo run -p audit-chain-app -- verify-seal --pack <pack> --window "last 1h"
# Expect: exit 0
```

## References

- `microservices/cloud-secrets/failure-modes.md` FM-04
- `microservices/cloud-secrets/threat-model.md` T-D-03
- `microservices/cloud-secrets/IP-013-audit-emitter-bridge-to-audit-chain.md`
- `microservices/cloud-secrets/incident-response.md`
- `microservices/audit-chain/runbooks/*.md` (audit-chain µservice runbooks)
- Bominal ADR-0028 (audit-chain posture)
