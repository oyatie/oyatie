---
microservice: compliance
ip: IP-013
title: Audit anomaly detection (seal chain anomaly detector → Sev-1 paging)
status: Drafting
authority_tier: 3
owner: axis-security
co_owners: [axis-compliance]
date: 2026-05-18
related_adrs: [ADR-0145, ADR-0209]
---

# IP-013 — Audit anomaly detection

## Purpose

Detect seal-chain anomalies + access anomalies + DSAR-rate anomalies. Sev-1 on chain break; Sev-2 on access spike; Sev-3 on DSAR-rate spike.

## Acceptance criteria

1. Seal-chain validator runs every 6 hours; verifies cosign keyless OIDC chain continuity.
2. Per-accessor + per-subject access anomaly detector (per IP-004 HIPAA threshold; broadened to all PHI / PII).
3. DSAR-rate anomaly: > 50 DSARs / tenant / day flagged Sev-3 (possible coordinated request attack).
4. Alerting routes to PagerDuty (Sev-1 / Sev-2) + Slack (Sev-3).
5. False-positive playbook at `runbooks/audit-anomaly-false-positive.md`.
6. ≥ 5 integration tests: seal-chain-break-Sev-1 + access-spike-Sev-2 + dsar-rate-Sev-3 + per-accessor-baseline-calibration + sev-1-pages-on-call.

## Detector matrix

| Detector | Window | Threshold | Severity |
|---|---|---|---|
| Seal chain break | continuous | any | Sev-1 |
| Per-accessor PHI access | 1 hour | > 100 / subject | Sev-2 |
| Per-tenant DSAR rate | 1 day | > 50 / tenant | Sev-3 |
| Engagement-end Cedar revoke fail | continuous | any | Sev-1 |
| EVT-AUDIT-SEAL-VERIFY-FAILED | continuous | any | Sev-1 |

## Risk + mitigation

- **Risk:** false positives drown on-call. **Mitigation:** per-accessor baseline calibration window (first 30 days); manual confirmation flow.
- **Risk:** detector misses a real attack. **Mitigation:** quarterly red-team validates detection.

## Acceptance evidence

`evidence/ip-013-audit-anomaly-detection-acceptance.json`.

## Cross-references

- ADR-0145 — substrate.
- ADR-0209 — substrate authority.
- IP-004 — HIPAA min-necessary log.
- IP-005 — audit chain seal coverage.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/compliance/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/compliance/IP-013-audit-anomaly-detection.md` matched `PHI`; anchors `microservices/compliance/runbooks/phi-access-anomaly.md, crates/oya-shared-compliance-evidence-kernel/src/lib.rs`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.
