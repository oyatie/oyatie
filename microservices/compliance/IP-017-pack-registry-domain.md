---
ip_id: IP-017
microservice: compliance
bounded_context: pack-registry
layer: domain
status: planned
related_adrs: [ADR-0251, ADR-0244, ADR-0263, ADR-0324]
---

# IP-017 — pack-registry domain

## A. Problem

IP-016 defines pure pack lifecycle rules, but tenants need a side-effecting domain service that stores pack state, emits audit evidence, updates coverage expectations, and applies per-tenant residency and data-class constraints. The current shell only said "storage + observability"; it did not define how pack subscription affects auditor views, control mapping, or conflict handling.

## B. Approach

Build `oya-compliance-pack-registry-domain` around injected ports: `PackManifestStore`, `PackPublicationLedger`, `CoverageExpectationWriter`, `AuditEmitter`, and `Clock`. The domain composes the IP-016 kernel with tenant-scoped state and ADR-0263 events. It never writes directly to Postgres or SeaweedFS; IP-023 and future adapters expose it.

## C. Deliverables

| Artifact | Change |
|---|---|
| `microservices/compliance/catalog/oya-compliance-pack-registry-domain.yaml` | domain catalog row |
| `microservices/compliance/policy/pack-overlay-authorization.cedar` | authorization facts for pack subscribe/publish/deprecate |
| `microservices/compliance/dashboards/pack-overlay-coverage.json` | pack coverage and conflict dashboard |
| `microservices/compliance/runbooks/pack-overlay-conflict-resolution.md` | operations path for contradictory pack obligations |
| `microservices/compliance/decisions/ADR-COMP-001-pack-overlay-precedence-conflict-resolution.md` | precedence authority for higher-restriction-wins |

## D. Implementation

1. Define domain commands: `RegisterPack`, `StartPackSoak`, `PublishPack`, `SubscribeTenantToPack`, `DeprecatePack`, `SunsetPack`.
2. Validate every command through the IP-016 kernel before touching storage.
3. On publish, write coverage expectations that IP-022 and IP-026 can expose as control-mapping requirements.
4. On tenant subscription, evaluate `policy/pack-overlay-authorization.cedar` facts: tenant id, pack id, BAA/DPA status, home cell, and data classes.
5. Emit `oya.compliance.pack-registered`, `pack-soak-started`, `pack-published`, `tenant-pack-subscribed`, and `pack-deprecated` events.
6. Detect pack conflicts such as GDPR erasure versus statutory retention and route them to ADR-COMP-001 precedence.
7. Add tests for tenant-scope isolation, stale manifest rejection, publication soak, and conflict dashboard entries.

## E. Acceptance

- Domain commands are idempotent by pack id and version.
- A tenant cannot subscribe to a pack that conflicts with its home-cell residency.
- Published pack requirements appear in control mapping and evidence coverage rollups.
- Conflicting pack obligations create an explicit conflict record rather than silent override.

## F. Evidence

- `microservices/compliance/policy/pack-overlay-authorization.cedar` is the local policy surface.
- `microservices/compliance/dashboards/pack-overlay-coverage.json` and `runbooks/pack-overlay-conflict-resolution.md` are the operating surfaces.
- `microservices/compliance/competitor-parity-matrix.md` says per-pack regulatory overlays are an Oyatie differentiator over Drata/Vanta/AuditBoard/ServiceNow GRC.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| OneTrust / Tugboat Logic | Brings privacy-pack workflow closer to commercial GRC depth but keeps evidence in operator-controlled storage. |
| AuditBoard | Adds auditable state transitions behind framework changes rather than spreadsheet-style control updates. |
| Vanta | Exceeds simple framework enablement by binding pack subscription to Cedar, residency, and audit events. |

## H. Non-goals and handoff boundaries

- Do not duplicate IP-016 validation rules in the domain; call the kernel and preserve its typed errors.
- Do not let subscription mutate pack manifests; tenant subscription is separate from pack publication.
- Do not resolve pack conflicts by last-write-wins; route conflicts to ADR-COMP-001 higher-restriction logic.
- Do not store raw evidence payloads in pack subscription records; store evidence refs and audit refs.
- Do not expose domain mutation through auditor REST endpoints; IP-020 is read-only.

## I. Fixture set

- `tenant_subscribes_gdpr_success.json` proves normal subscription.
- `tenant_home_cell_outside_pack_residency.json` proves Cedar denial.
- `gdpr_erasure_vs_soc2_retention_conflict.json` proves explicit conflict record creation.
- `publish_without_soak.json` proves SLO-backed publish rejection.
- `duplicate_subscription_idempotent.json` proves retry safety.

## J. Launch blockers

- Pack publish command succeeds without an ADR-0263 event.
- Tenant subscription succeeds without Cedar decision evidence.
- Conflict records omit tenant id, pack id, or affected control ids.
- Coverage expectation writes are not idempotent by pack version.
- Dashboard rows cannot trace back to pack registry state.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/compliance/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/compliance/IP-017-pack-registry-domain.md` matched `SLO`; anchors `microservices/compliance/runbooks/phi-access-anomaly.md, crates/oya-shared-compliance-evidence-kernel/src/lib.rs`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.
