---
doc_class: Runbook
title: Capability registry resync (cache stale + signature invalid)
microservice: foundry-runtime
severity: "Sev-1 (signature invalid; tampering suspect) / Sev-2 (cache stale)"
status: Accepted
owner_team: axis-foundry-runtime + axis-foundry
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-04, FM-05)
  - microservices/intelligence/threat-model.md (T-T-04)
  - microservices/intelligence/incident-response.md
doc_status: published
---

# Runbook: Capability registry resync

## Trigger

ONE of:
- Capability registry cache stale (FM-04): `oya_foundry_runtime_registry_cache_age_seconds > 60`.
- Capability descriptor signature invalid (FM-05): `oya_capability_mirror_signature_invalid_total > 0`.

## Severity

- Cache stale: Sev-2 (operational; gate fails-graceful with last-known descriptor).
- Signature invalid: Sev-1 (potential tampering; engage ops-security).

## Cache stale (FM-04)

| Step | Action | Time |
|---|---|---|
| 1 | Verify foundry-supervisor reachability: `kubectl exec <runtime-pod> -- curl -s https://foundry-supervisor.internal/health` | ≤2min |
| 2 | If supervisor unreachable: engage foundry-supervisor on-call | ≤5min |
| 3 | Verify replication path: Postgres logical replication slot active; `pg_replication_slots` | ≤5min |
| 4 | Force cache resync: `cargo run -p oya-intelligence-runtime-capability-registry-cache-app -- resync --tenant all` | ≤10min |
| 5 | Verify cache age returns < 60s | ≤10min |
| 6 | If outage > 30min: tenant comms (some newly-registered descriptors delayed) | ≤30min |

## Signature invalid (FM-05)

| Step | Action | Time |
|---|---|---|
| 1 | Engage Sev-1; open `#inc-sec-<id>` Slack; declare ops-security | immediate |
| 2 | Identify affected descriptors: `SELECT capability_id, tenant_id, descriptor_signature_invalid_since FROM capability_mirror WHERE signature_valid = false` | ≤5min |
| 3 | Blacklist affected descriptors: runtime refuses dispatch (fail-closed) | ≤2min |
| 4 | Compare with supervisor source-of-truth: signed descriptor at supervisor vs runtime mirror | ≤10min |
| 5 | Determine cause: (a) replication corruption, (b) supervisor signing-key rotation incomplete, (c) tampering | ≤30min |
| 6 | (a) Replication: drop replica slot; resync from supervisor; verify signatures match | ≤30min |
| 7 | (b) Rotation incomplete: complete rotation at supervisor; runtime pulls fresh signed copies | ≤30min |
| 8 | (c) Tampering: ops-security incident; breach-notification chain per `incident-response.md` | per pack regulatory |
| 9 | Verify recovery: `signature_invalid_total` returns to 0 for ≥30min | – |

## Manual hot-reload (operational use; no incident)

```bash
# Trigger a single-capability hot reload bypassing event-driven path
cargo run -p oya-intelligence-runtime-capability-registry-cache-app -- hot-reload \
  --tenant <tenant_id> \
  --capability-id <capability_id> \
  --reason "<rfc>"
```

This is audit-chain-emitted but does not require Sev-1 declaration if the trigger is operational (e.g., tenant requested a forced refresh after a descriptor edit).

## Verification

After recovery:
- `oya_foundry_runtime_registry_cache_age_seconds < 60` for ≥15min.
- `oya_capability_mirror_signature_invalid_total == 0`.
- Dispatch resumes against affected capabilities.
- Self-observability dashboard green.

## Post-incident updates

- Postmortem within 5 business days.
- For repeated FM-04: investigate replication channel resilience; consider redundant supervisor connectivity.
- For FM-05 with tampering confirmed: forensic trace + Postgres audit log review + harden replication path.

## References

- `microservices/intelligence/failure-modes.md` FM-04, FM-05.
- `microservices/intelligence/threat-model.md` T-T-04.
- `microservices/intelligence/incident-response.md` §"Regulatory Notifications".
- foundry-supervisor µservice (sibling) runbooks.
