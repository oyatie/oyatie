---
ip_id: IP-021
microservice: compliance
bounded_context: cell-certification-attestation
layer: worker
status: planned
related_adrs: [ADR-0248, ADR-0250, ADR-0293, ADR-0263]
---

# IP-021 — cell-certification-attestation worker

## A. Problem

Enterprise and public-sector tenants need proof that a cell is eligible for their pack before workloads run there. `manifest.json` names `tenant-cell and regional-failover execution domain`, and compliance dashboards already track evidence health, but no worker rolls SOC 2, HIPAA, PCI, KR-CSAP, FedRAMP, and EU-sovereign signals into a per-cell attestation. The shell did not define source signals or how conflicting pack states are represented.

## B. Approach

Create `oya-compliance-cell-certification-attestation-worker` as a scheduled and on-demand worker. It reads observability SLOs, tenancy placement, cloud-secrets key custody, audit-chain seal status, pack registry state, and control mapping status. It emits a composite attestation per cell and per pack, never a single misleading boolean.

## C. Deliverables

| Artifact | Change |
|---|---|
| `microservices/compliance/catalog/oya-compliance-cell-certification-attestation-worker.yaml` | worker catalog row |
| `microservices/compliance/runbooks/certification-evidence-pipeline-stall.md` | operational response when worker cannot produce fresh attestations |
| `microservices/compliance/dashboards/evidence-coverage.json` | add cell/pack certification panels |
| `microservices/compliance/slos/evidence-coverage-rollup.openslo.yaml` | freshness SLO for rollup coverage |
| `microservices/compliance/AUDIT-FINDINGS-2026-05-20.json` | seed known gaps as initial worker fixtures where applicable |

## D. Implementation

1. Define `CellAttestation { cell_id, region, pack_id, status, source_signals, stale_signals, blockers, generated_at }`.
2. Pull pack requirements from IP-017 and control requirements from IP-022.
3. Read audit-chain seal status through existing evidence APIs; stale or failed seals become blocker signals.
4. Read SLO status from `slos/*` and dashboard summaries without copying raw metric time series into the artifact.
5. Represent conflicts as per-pack statuses: `soc2=pass`, `kr-csap=blocked`, `fedramp-high=unknown`; never collapse.
6. Emit `oya.compliance.cell-attestation-generated` and `cell-attestation-blocked` events.
7. Add daily schedule plus on-demand regulator query trigger.
8. Test missing observability source, stale secret-custody signal, pack conflict, and tenant-scope isolation.

## E. Acceptance

- Daily worker emits per-cell/per-pack attestations with source evidence ids.
- Conflicting pack states remain visible as separate rows.
- Stale source signals are explicit blockers, not silent pass/fail defaults.
- Regulator query path can request an attestation without waiting for the next cron.

## F. Evidence

- `microservices/compliance/manifest.json` names dependencies on audit-chain, tenancy, observability, cell, cloud-iac, and cloud-secrets.
- `microservices/compliance/slos/evidence-coverage-rollup.openslo.yaml` and dashboards provide local coverage signals.
- `microservices/compliance/competitor-parity-matrix.md` compares ServiceNow GRC, AuditBoard, Drata, and Vanta; none provide Oyatie's operator-owned cell certification.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| AWS Artifact | Provides certification-style evidence access while preserving Oyatie cell and pack semantics. |
| Drata / Vanta | Extends continuous evidence into cell eligibility rather than generic SaaS environment checks. |
| ServiceNow GRC | Narrows enterprise attestation workflow parity with machine-verifiable cell outputs. |

## H. Non-goals and handoff boundaries

- Do not certify a tenant globally when only one cell passes; attestation is per-cell and per-pack.
- Do not hide stale source signals behind `unknown == pass`.
- Do not copy raw observability series into certification artifacts; link source evidence.
- Do not collapse KR-CSAP, FedRAMP, SOC 2, HIPAA, and PCI into one boolean.
- Do not replace human certification bodies; provide evidence they can verify.

## I. Fixture set

- `cell_soc2_pass_kr_blocked.json` proves per-pack status granularity.
- `audit_chain_seal_stale_blocks.json` proves stale-source blocker.
- `observability_source_down_unknown.json` proves graceful degradation.
- `on_demand_regulator_query.json` proves non-cron trigger.
- `tenant_scope_cross_cell_denied.json` proves placement boundary.

## J. Launch blockers

- Worker emits a single global certified boolean.
- Stale source signals are treated as pass.
- Attestations omit pack id or cell id.
- Certification evidence copies raw metric series instead of source refs.
- Regulator-triggered run cannot be correlated to an audit event.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/compliance/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/compliance/IP-021-cell-certification-attestation-worker.md` matched `SLO`; anchors `microservices/compliance/runbooks/phi-access-anomaly.md, crates/oya-shared-compliance-evidence-kernel/src/lib.rs`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.
