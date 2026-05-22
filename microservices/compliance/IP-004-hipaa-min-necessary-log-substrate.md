---
microservice: compliance
ip: IP-004
title: HIPAA minimum-necessary access log substrate (per 45 CFR § 164.514(d))
status: Drafting
authority_tier: 3
owner: axis-compliance
co_owners: [axis-security]
date: 2026-05-18
related_adrs: [ADR-0145, ADR-0183, ADR-0209]
---

# IP-004 — HIPAA minimum-necessary access log substrate

## Purpose

Provide the HIPAA "minimum necessary" access-log substrate per 45 CFR § 164.514(d). Every Protected Health Information (PHI) access logs:

- Subject pseudonym (never raw subject name).
- Accessor identity (SPIFFE-ID per ADR-0148).
- Access purpose (closed enum: treatment, payment, healthcare-ops, research, audit, dsar).
- Cedar policy decision (allow / deny / explain).
- Timestamp + cell-id + tenant-id.
- Audit-chain seal hex (per ADR-0145).

Drives HIPAA compliance reporting + minimum-necessary audit + breach detection.

## Acceptance criteria

1. Append-only JSONL log at `microservices/compliance/evidence/hipaa-min-necessary/<yyyy>/<mm>/<dd>.jsonl`.
2. Per-entry cosign keyless OIDC seal hex.
3. Continuous compaction to cold tier per retention policy (HIPAA statutory 6 years).
4. Query API: `GET /api/v1/hipaa/min-necessary?subject={pseudonym}&from={ts}&to={ts}` (auditor-only, Cedar-gated).
5. Cross-tenant isolation invariant: queries scoped by tenant.
6. Sev-1 anomaly detector: > 100 accesses to one subject in 1 hour by a single accessor flagged for review.
7. ≥ 6 integration tests: emit + read-back + seal-verify + cross-tenant guard + anomaly trigger + retention enforcement.

## Schema

```jsonl
{"ts": "2026-05-18T12:00:00Z", "subject_pseudonym": "subj_abc", "accessor_spiffe_id": "spiffe://oya/identity/healthcare-portal-pod-1", "purpose": "treatment", "cedar_decision": "allow", "cedar_policy_id": "phi-read-treatment", "tenant_id": "tenant_h", "cell_id": "cell-us-east", "seal_hex": "..."}
```

## Cedar policy fragments

```cedar
// capabilities/phi-read-treatment.cedar
permit (
  principal in Accessor::"clinician",
  action == Action::"read-phi",
  resource is Subject
) when {
  context.purpose == "treatment" &&
  principal.tenant_id == resource.tenant_id &&
  resource.has_consent_for("treatment")
};
```

## Anomaly detector

A streaming job consumes EVT-PHI-ACCESS events:

- Window: 1 hour, per-accessor + per-subject.
- Threshold: 100 accesses.
- Sev-1 trigger → AlertManager → PagerDuty.
- False-positive mitigation: review by privacy officer within 24 hours.

## Cross-tenant isolation invariant

Same belt-and-suspenders as DSAR (IP-003):

1. API handler asserts `query.tenant_id == principal.tenant_id`.
2. Storage adapter filters by `tenant_id` in the JSONL scan.
3. Cedar capability requires `principal.tenant_id == resource.tenant_id`.
4. Integration test asserts cross-tenant query returns 403 + zero records.

## Risk + mitigation

- **Risk:** log volume blows up storage at scale (millions of PHI accesses / day). **Mitigation:** cold tier after 30 days (SeaweedFS cold per ADR-0184); compaction to columnar Parquet.
- **Risk:** raw subject name leaks via log payload. **Mitigation:** schema rejects fields not in the closed list; CI grep at `tests/no_raw_name_in_log_payload.rs`.
- **Risk:** anomaly detector false positives drown the privacy officer. **Mitigation:** per-accessor baseline calibration window (first 30 days).

## Acceptance evidence

`evidence/ip-004-hipaa-min-necessary-acceptance.json`.

## Cross-references

- ADR-0145 — audit-chain seal substrate.
- ADR-0148 — service mesh (SPIFFE-ID).
- ADR-0183 — Cedar policy engine.
- ADR-0209 — substrate authority.
- IP-001 — collector bootstrap.
- IP-013 — audit anomaly detection.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/compliance/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/compliance/IP-004-hipaa-min-necessary-log-substrate.md` matched `PHI, payment`; anchors `microservices/compliance/runbooks/phi-access-anomaly.md, crates/oya-shared-compliance-evidence-kernel/src/lib.rs`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.
