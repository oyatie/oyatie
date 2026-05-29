---
ip_id: IP-022
microservice: compliance
bounded_context: compliance-control-mapping
layer: domain
status: planned
related_adrs: [ADR-0251, ADR-0250, ADR-0209, ADR-0324]
---

# IP-022 — compliance-control-mapping domain

## A. Problem

The PRD requires SOC 2, GDPR, HIPAA, and PCI-DSS evidence coverage, and later IPs add pack-specific frameworks. But controls cannot be managed as prose tables only. Auditors need to know which control is satisfied by which collector, microservice, artifact kind, evidence id, and attestation timestamp. The existing 30-line IP gave a struct sketch but not the domain behavior needed for Drata/Vanta/AuditBoard parity.

## B. Approach

Implement `oya-compliance-control-mapping-domain` as the source of truth for framework controls, requirements, collector bindings, satisfaction status, and attestation history. It consumes pack requirements from IP-017, evidence artifacts from IP-001/IP-011, and manual uploads from IP-014. REST and SDK surfaces in IP-026 read from this domain.

## C. Deliverables

| Artifact | Change |
|---|---|
| `microservices/compliance/catalog/oya-compliance-control-mapping-domain.yaml` | domain catalog row |
| `microservices/compliance/contracts/openapi.yaml` | schemas later exposed by IP-026 |
| `microservices/compliance/dashboards/evidence-coverage.json` | control coverage by framework and tenant |
| `microservices/compliance/slos/evidence-coverage-rollup.openslo.yaml` | freshness and completeness SLO |
| `microservices/compliance/compliance.md` | framework/control vocabulary source |

## D. Implementation

1. Define `FrameworkControl`, `EvidenceCollectorBinding`, `ControlSatisfaction`, and `AttestationHistoryEntry`.
2. Support SOC 2 CC/A/C/PI/P families, GDPR articles, HIPAA safeguards, PCI-DSS requirements, KR-PIPA, FedRAMP, and EU AI Act pack controls.
3. Bind controls to collector ids from `manifest.json` and to microservice owners.
4. Mark controls `Unsatisfied` when a required collector is missing, stale, or deprecated by pack lifecycle.
5. Store attestation history references by evidence id and audit seal, not copied payload.
6. Emit `oya.compliance.control-satisfied`, `control-unsatisfied`, and `control-mapping-stale` events.
7. Add tests for missing collector blocker, stale evidence, pack overlay control, manual evidence satisfaction, and owner lookup.
8. Reject unrecognized framework ids instead of accepting arbitrary strings.

## E. Acceptance

- Every control returned by the domain has a responsible microservice, collector binding, satisfaction status, and last-attested reference.
- Removing a collector changes affected controls to `Unsatisfied`.
- Pack overlays add controls without mutating canonical SOC 2/GDPR mappings.
- REST/SDK in IP-026 can be implemented without duplicating mapping logic.

## F. Evidence

- `microservices/compliance/PRD.md` defines SOC 2/GDPR/HIPAA/PCI goals and evidence kinds.
- `microservices/compliance/competitor-parity-matrix.md` shows pre-mapped controls as table stakes for Drata, Vanta, AuditBoard, and ServiceNow GRC.
- `microservices/compliance/slos/evidence-coverage-rollup.openslo.yaml` is the local coverage freshness anchor.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| Drata / Vanta | Matches continuous control mapping while keeping source evidence and seals in Oyatie. |
| AuditBoard SOXHUB | Narrows control-catalog parity with typed collector bindings. |
| ServiceNow GRC | Provides enterprise control status without adopting a separate GRC system. |

## H. Non-goals and handoff boundaries

- Do not store full evidence payloads in control mapping; store refs and seal ids.
- Do not allow controls without owner, collector, framework, and satisfaction status.
- Do not let pack overlays mutate canonical framework definitions in place.
- Do not treat manual uploads as satisfied until IP-014 seal and metadata checks pass.
- Do not expose write methods through the REST/SDK surface in IP-026.

## I. Fixture set

- `soc2_cc6_access_review_satisfied.json` proves collector binding.
- `collector_removed_unsatisfied.json` proves stale mapping behavior.
- `gdpr_article_15_dsar_history.json` proves privacy framework mapping.
- `hipaa_safeguard_phi_log.json` proves PHI control mapping.
- `kr_pipa_overlay_control_added.json` proves pack-added controls.

## J. Launch blockers

- A control exists without collector, owner, framework, and status.
- Manual evidence satisfies a control without seal verification.
- Pack overlay mutates canonical control ids in place.
- Removed collectors leave controls marked satisfied.
- Attestation history stores raw artifact payloads.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/compliance/IP-022-compliance-control-mapping-domain.md` matched `openapi`; contract files `microservices/compliance/contracts/openapi.yaml, microservices/compliance/contracts/asyncapi.yaml, microservices/compliance/contracts/compliance.proto`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/compliance/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/compliance/IP-022-compliance-control-mapping-domain.md` matched `SLO, PHI`; anchors `microservices/compliance/runbooks/phi-access-anomaly.md, crates/oya-shared-compliance-evidence-kernel/src/lib.rs`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.
