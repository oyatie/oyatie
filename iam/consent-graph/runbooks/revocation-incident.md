# Runbook: revocation-incident

- Severity: P0 (sovereignty exposure) / P1 (latency burn)
- Trigger conditions:
  - SLO `oya-consent-graph-revocation-propagation-latency` fast burn (1h 14.4×).
  - PartiallyPropagated count > 100 in 5min.
  - Pulsar revocation topic publish failure rate > 1%.
  - Geo-replication lag > 5s on revocation topic.
- Audit-evidence emitted at every step (`oya consent-graph runbook incident <id>`).

## Steps

1. **Acknowledge page within 5min.**
2. Open incident: `oya inc start consent-graph revocation-propagation`.
3. Identify scope:
   - Region(s) affected (Pulsar broker dashboard).
   - Tenant pairs affected (revocation receipt query).
   - Backlog depth (`oya consent-graph revocation backlog`).
4. Triage class:
   - **Publish failure**: Pulsar broker outage. Skip to step 5.
   - **Subscriber lag**: enforcement-app or projection-gateway-app pod down/slow. Skip to step 6.
   - **Geo-replication lag**: WAN issue. Skip to step 7.
   - **Partial propagation**: subset of subscribers receipted, others didn't. Skip to step 8.
5. **Publish failure path**:
   a. Confirm broker health: `oya pulsar broker health <region>`.
   b. If multi-broker outage: failover to DR cluster; emit failover audit event.
   c. Replay dead-letter table: `oya consent-graph replay revocation-dlq --window 1h`.
   d. Verify revocation_id checksums match originals.
6. **Subscriber lag path**:
   a. Identify slow subscriber: `oya consent-graph receipts latency --window 30m`.
   b. Restart slow pod: `kubectl rollout restart deployment/<subscriber-app> -n <ns>`.
   c. If repeatedly slow, scale: `kubectl scale deployment/<subscriber-app> --replicas=N`.
7. **Geo-replication lag path**:
   a. Pulsar geo-rep status: `oya pulsar georep status <region>`.
   b. If paused: `oya pulsar georep resume <region>`.
   c. If WAN issue: notify network team; deny-by-default in affected dest region (already automatic).
8. **Partial propagation path**:
   a. For each PartiallyPropagated revocation, run deadline reconciler manually:
      `oya consent-graph revocation reconcile <rev-id>`.
   b. If subscriber never recovers: manually invalidate by emergency tool:
      `oya consent-graph enforcement invalidate-policy <agreement-id> --force`.
9. **Verify SLO recovery**: watch fast-burn rate drop below 14.4× threshold for 30min sustained.
10. **Close incident**: `oya inc close <id>` with summary.
11. **Post-mortem**: schedule within 48h.

## Verification

- Manual revoke + measure propagation latency to confirm <1s p99 restored.
- Run IP-013 reconciler ad-hoc to confirm zero divergences.

## Audit evidence

- Incident record sealed in audit-chain (incident-id + timeline + actions taken).
- Every replay action emits `replay_session_id` audit event.
- Post-mortem doc itself sealed.

## Escalation

- 30min sustained P0 without mitigation path → incident commander.
- Sovereignty implication suspected → privacy officer + DPO.
- Regulatory disclosure threshold (e.g., GDPR Art. 33 72h-clock potential) → DPO + legal.
