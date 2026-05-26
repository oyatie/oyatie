---
doc_class: Runbook
title: Supervision-bus replay (lag remediation; audit-chain catch-up)
microservice: foundry-supervisor
severity: "Sev-2 (audit-chain seals delayed; not a breach)"
status: Accepted
owner_team: axis-foundry-control-plane + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-08)
  - microservices/intelligence/contracts/asyncapi/foundry-supervisor-events.yaml
doc_status: published
---

# Runbook: Supervision-bus replay

## Trigger

ONE of:

1. **Bus lag** — `oya_supervisor_supervision_event_bus_lag_p99 > 500 ms` for ≥ 5 min (FM-08).
2. **foundry-evidence ingest slow** — backpressure from downstream.
3. **Replay needed for audit recovery** — post-incident; reseal a window of events.

## Severity

- Bus lag (data flowing but lagging): Sev-2.
- Lag with audit-chain Merkle break suspected: Sev-1 (data integrity).

## Steps — Bus lag (FM-08)

| Step | Action | Time |
|---|---|---|
| 1 | Open `#inc-<id>`; assign IC | ≤ 5 min |
| 2 | Inspect Valkey Streams (Redis wire-compat) consumer lag: `redis-cli XLEN supervision-events` vs `XPENDING supervision-events evidence-group` | ≤ 5 min |
| 3 | Apply backpressure: pause non-critical event publishing (e.g., FleetState rebroadcasts; keep critical KillSwitch + AutonomyViolated + DeploymentRolledBack flowing): `cargo run -p oya-dev-cli -- supervisor bus-backpressure --classes "non-critical"` | ≤ 1 min |
| 4 | Scale foundry-evidence ingest: `kubectl scale deployment foundry-evidence-ingest --replicas=+2 -n foundry-evidence` | ≤ 5 min |
| 5 | Verify bus lag clearing | ≤ 30 min |
| 6 | Resume normal event publishing: `cargo run -p oya-dev-cli -- supervisor bus-backpressure --release` | ≤ 1 min |
| 7 | Post-incident: capacity-model review of supervision-bus | – |

## Steps — Replay needed

| Step | Action |
|---|---|
| 1 | Identify replay window: `(start_ts, end_ts)` |
| 2 | Pause new publishing for affected scope: `cargo run -p oya-dev-cli -- supervisor pause-bus --scope <scope>` |
| 3 | Replay events: `cargo run -p oya-dev-cli -- supervisor replay-bus --start <ts> --end <ts> --reason "<rfc>"`. Events are re-emitted with `replayed=true` label; audit-chain verifies signatures (originals unchanged) + appends replay record. |
| 4 | Verify foundry-evidence sealed the replay events; check audit-chain Merkle integrity |
| 5 | Resume new publishing: `cargo run -p oya-dev-cli -- supervisor resume-bus --scope <scope>` |
| 6 | Audit-chain seal records the replay-window |

## Audit-chain break (Sev-1)

If Merkle integrity check fails (events out-of-sequence, missing seals, signature invalid):
1. Sev-1 declared; engage ops-security + council-privacy + audit-chain µservice owner.
2. Engage fleet-wide kill-switch with 2-person rule (refuse all new invocations until cause known).
3. Forensic + breach-notification chain (data-integrity per GDPR Art. 32(1)(b)).
4. Fix + rebuild Merkle subtree from underlying events.
5. Post-mortem within 24 h.

## Verification

- `oya_supervisor_supervision_event_bus_lag_p99 <= 200 ms` for ≥ 1 h.
- foundry-evidence audit-chain Merkle integrity green.
- Replay events properly tagged `replayed=true` (do not displace originals).
- Per-changeset evidence updated.

## References

- `failure-modes.md` FM-08.
- `contracts/asyncapi/foundry-supervisor-events.yaml`.
- ADR-0028 (audit-chain).
- Valkey Streams (Redis wire-compat) — `redis.io/docs/data-types/streams/`.
- `incident-response.md` §"Sev-2 response".
