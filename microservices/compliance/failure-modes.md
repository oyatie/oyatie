---
microservice: compliance
doc: FailureModes
status: Drafting
authority_tier: 3
owner: ops-sre-reliability
co_owners: [axis-compliance, axis-security]
date: 2026-05-18
related_adrs: [ADR-0209]
---

# Compliance — Failure Modes Catalog

## Failure mode index

| ID | Failure mode | Severity | Mitigation |
|---|---|---|---|
| FM-01 | Cross-tenant DSAR leak | Sev-1 | Kernel invariant; 5-layer guard (per threat-model.md A2); integration test |
| FM-02 | Audit-chain seal verify failure | Sev-1 | Cosign keyless OIDC + cold-tier re-seal (IP-005) |
| FM-03 | Engagement-end Cedar revoke fails | Sev-1 | Webhook + integration test asserts revoke (IP-007) |
| FM-04 | SeaweedFS hot bucket loss | Sev-2 | DR drill restore from cold tier (IP-006 + IP-012) |
| FM-05 | DSAR backlog overflow | Sev-2 | Circuit-break new intake at backlog > 100 (IP-003) |
| FM-06 | PHI access anomaly (per IP-004 threshold) | Sev-2 | Streaming detector + on-call review |
| FM-07 | Per-µservice attestation miss before audit | Sev-2 | Quarterly reminder + escalation (IP-010) |
| FM-08 | Collector tier degraded (HPA at max) | Sev-2 | Backpressure + per-µservice rate limit |
| FM-09 | DSAR rate spike (potential attack) | Sev-3 | Per-tenant rate limit (10/day) + alert |
| FM-10 | Auditor portal p99 latency burn | Sev-3 | HPA + per-µservice trace sampling (per ADR-0210) |
| FM-11 | Cold-tier read fails (cold storage degraded) | Sev-3 | 3-way replication; off-site backup |
| FM-12 | Anomaly detector false positives | Sev-4 | Per-accessor baseline calibration |
| FM-13 | Pen-test report upload size > 100 MB | Sev-4 | Reject + ask for redaction |
| FM-14 | Cosign Fulcio outage | Sev-2 | Multi-issuer fallback (operator OIDC) |
| FM-15 | Per-pack manifest drift (canonical changed) | Sev-3 | Quarterly pack-canonical-diff review |
| FM-16 | Per-tenant retention policy misconfigured | Sev-3 | Retention gate fails-closed; manual override requires ADR exception |
| FM-17 | Audit anomaly detector miss | Sev-2 | Quarterly red-team validates |
| FM-18 | Pseudonym key compromise | Sev-1 | OpenBao key rotation; pseudonym key version embedded in ciphertext |
| FM-19 | DSAR API auth bypass | Sev-1 | Zitadel + Cedar; integration test asserts auth |
| FM-20 | Per-µservice attestation forged | Sev-1 | SPIFFE-ID + cosign seal on attestation form |

## Recovery time objectives

| FM tier | RTO | RPO |
|---|---|---|
| Sev-1 | 1 hour | 0 (no data loss tolerated) |
| Sev-2 | 4 hours | 1 hour |
| Sev-3 | 24 hours | 4 hours |
| Sev-4 | next business day | 1 day |

## References

- ADR-0209 — substrate authority.
- threat-model.md — security threat model.
- incident-response.md — response runbooks.
