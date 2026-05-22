---
ip_id: IP-016
microservice: compliance
bounded_context: pack-registry
layer: kernel
status: planned
related_adrs: [ADR-0251, ADR-0083, ADR-0064, ADR-0324]
---

# IP-016 — pack-registry kernel

## A. Problem

Compliance packs are the mechanism that turns generic SOC 2/GDPR/HIPAA evidence into jurisdiction-specific obligations such as KR-PIPA, FedRAMP High, EU AI Act, and PCI-DSS overlays. Today `manifest.json` lists packs and `policy/pack-overlay-authorization.cedar` can gate subscription, but there is no pure kernel definition of a pack manifest lifecycle. Without that kernel, pack publication can drift into ad hoc JSON edits that look like Drata/Vanta checklist rows but do not give Oyatie a verifiable, replayable pack state machine.

## B. Approach

Create `oya-compliance-pack-registry-kernel` as a no-I/O crate for pack manifest validation and lifecycle transitions: `register`, `publish`, `deprecate`, and `sunset`. The kernel owns closed enums for pack id, pack status, framework ids, artifact-kind ids, data-class scope, residency scope, and publication soak state. Domain and adapters call this crate; Cedar and storage do not live in the kernel.

## C. Deliverables

| Artifact | Change |
|---|---|
| `microservices/compliance/catalog/oya-compliance-pack-registry-kernel.yaml` | catalog the crate and owner |
| `microservices/compliance/packs/GDPR.md`, `HIPAA.md`, `KR-PIPA.md`, `SOC2.md`, `EU-AI-Act.md` | serve as source examples for manifest fields |
| `microservices/compliance/policy/pack-overlay-authorization.cedar` | consume kernel statuses through the domain layer |
| `microservices/compliance/slos/pack-publish-soak-respected.openslo.yaml` | acceptance SLO for publication soak |

## D. Implementation

1. Define `CompliancePackManifest { pack_id, version, frameworks, artifact_kinds, data_classes, residency_rules, required_collectors, effective_from }`.
2. Define `PackLifecycleState` as `Registered`, `Soaking`, `Published`, `Deprecated`, `Sunset` with transition functions that return typed errors.
3. Validate that every framework id maps to a known evidence/control namespace from `manifest.json` and `compliance.md`.
4. Reject manifests that add PHI, SECRET, or PII_IDENTIFYING handling without an explicit data-class and residency rule.
5. Require publish soak metadata so ADR-0294-style rollout discipline can be enforced by downstream domain code.
6. Add property tests for illegal transitions: publish-before-register, sunset-before-deprecate, version regression, and framework-without-artifact-kind.
7. Add fixtures for `gdpr`, `hipaa`, `kr-pipa`, and `fedramp-high` drawn from current pack docs.

## E. Acceptance

- Kernel compiles without adapter, database, network, or Cedar dependencies.
- Every pack in `manifest.json` can be represented or is flagged with a precise validation error.
- Property tests cover all legal and illegal lifecycle transitions.
- `pack-publish-soak-respected` can observe the transition timestamps emitted by the domain layer.

## F. Evidence

- `microservices/compliance/manifest.json` lists active packs and data classes.
- `microservices/compliance/packs/*.md` provide pack-specific obligations.
- `microservices/compliance/competitor-parity-matrix.md` names Drata, Vanta, OneTrust/Tugboat, AuditBoard, and ServiceNow GRC as evidence-management counterparts.
- ADR-0251 supplies compliance-pack primitive doctrine; ADR-0324 forbids generating pack content by templates.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| Drata / Vanta | Replaces opaque SaaS framework toggles with a typed pack lifecycle that Oyatie can test and replay. |
| ServiceNow GRC | Narrows framework-management parity while keeping pack state in code-owned kernel types. |
| AWS Audit Manager | Matches framework registration discipline without externalizing evidence custody. |

## H. Non-goals and handoff boundaries

- Do not read Postgres, SeaweedFS, Cedar, OpenBao, or the filesystem from the kernel crate.
- Do not accept arbitrary string framework ids; pack ids and framework ids are closed enums or validated newtypes.
- Do not publish a pack from a manifest that lacks data-class and residency declarations.
- Do not use line-count or shape-only checks as proof that a pack manifest is valid.
- Do not generate pack manifests from templates; each pack remains a bespoke regulatory artifact under ADR-0324.

## I. Fixture set

- `gdpr_valid_pack_manifest.json` validates GDPR Article and DSAR artifact bindings.
- `hipaa_missing_baa_rule.json` must fail because PHI handling lacks a provisioning precondition.
- `kr_pipa_valid_pack_manifest.json` validates KR-PIPA data-class and residency rules.
- `fedramp_high_missing_control.json` must fail because the framework lacks a required control binding.
- `publish_before_register.json` must fail with `IllegalTransition`.

## J. Launch blockers

- Missing enum coverage for any pack in `manifest.json`.
- Any fixture passing without data-class declarations.
- Any transition function that mutates state in place instead of returning a new state.
- Any dependency that introduces filesystem, database, or network I/O into the kernel.
- Any generated manifest body used as evidence of regulatory substance.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/compliance/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/compliance/IP-016-pack-registry-kernel.md` matched `SLO, PHI`; anchors `microservices/compliance/runbooks/phi-access-anomaly.md, crates/oya-shared-compliance-evidence-kernel/src/lib.rs`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.
