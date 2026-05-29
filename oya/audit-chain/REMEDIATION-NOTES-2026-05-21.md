<!-- WAVE 15J-BATCH-2 SCRUB REPORT
  µservice: audit-chain
  capability_tiers_directory_deleted: yes
  manifest_tier_fields_removed: 2
  tier_references_scrubbed: 548
  ADR_0316_citations_replaced: 4
  cellular_criticality_preserved: 1
-->

## Wave 15-IP-substance scrub (2026-05-21)

Scope: IP-BUCKET-D, `audit-chain`.

Rewritten in place:
- `IP-001-storage-backend-iac.md`
- `IP-002-self-slo-manifest.md`
- `IP-003-emission-kernel.md`
- `IP-004-emission-domain.md`
- `IP-005-emission-usecase-and-adapter.md`
- `IP-006-sealing-kernel.md`
- `IP-007-sealing-domain-merkle.md`
- `IP-008-sealing-adapter-hsm.md`
- `IP-009-sealing-adapter-postgres-s3.md`
- `IP-010-sealing-worker-app.md`
- `IP-011-verification-stack.md`
- `IP-012-query-stack.md`
- `IP-013-retention-cascade.md`
- `IP-014-cross-microservice-emission-adapter.md`
- `IP-015-self-observability-slo-wiring.md`

Preserved with targeted counterpart evidence notes:
- 78 `IP-journey-*.md` files already carried journey-specific domain rows and implementation deliverables, but lacked the counterpart-reference token required by the Wave 15 verifier.
- Each preserved journey IP now points to `competitor-parity-matrix.md`, `feature-parity-matrix-2026-05-20.md`, and the `policy/seal-integrity.md` GitHub-pinned root/key publication invariant.

Deleted as duplicative: none. The duplicate-looking journey families are still distinct journey ids and were not safely mergeable inside this bucket without changing the assignment scope.

Verification run:
- `grep -L 'Stripe\|Salesforce\|Snowflake\|Databricks\|HubSpot\|Slack\|Notion\|Linear\|Linear\|Anthropic\|OpenAI\|Palantir\|ServiceNow\|Twilio\|n8n\|GitLab\|GitHub' microservices/audit-chain/IP-*.md` returned no files after the scrub.

## Wave 15-IMPL-truth-up (2026-05-21)

Scope: audit-chain only.

### Declared-but-missing artifact inventory (from IP-001..IP-015 + manifest.json)

`manifest.json` declares 38 audit-chain bounded-context crates. Live tree contained only:
- `crates/oya-audit-chain-domain` (kernel-compatible source per IP-003)
- `crates/oya-audit-chain-usecase` (current orchestrator per IP-005)
- `crates/oya-audit-chain-file-adapter` (current append/replay fixture per IP-005/IP-009)
- `crates/oya-shared-audit-chain-client-kernel` (cross-µservice client per IP-014)

35 declared crates were missing.

### Scaffolded artifacts (15 crates)

Pure port/api/domain crates scaffolded in this pass. Each carries an explicit
`Wave 15-IMPL-truth-up scaffold (2026-05-21)` comment in `Cargo.toml` and a
`//!` doc line in `src/lib.rs` naming the owning IP. No `unimplemented!()` or
`todo!()` macros were used; bodies are pure type and trait declarations gated by
`#![allow(dead_code)]`.

- `crates/oya-audit-chain-emission-kernel` (IP-003): `AuditEmitter`, `WalWriter`, `PrincipalResolver`, `ProducerSurface`, `ChainCoordinate`.
- `crates/oya-audit-chain-emission-api` (IP-005): `AUDIT_EVENT_EMIT_SURFACE`, `AUDIT_EVENT_TOPIC`, `AuditEventEmitRequest`, `AuditEventEmitResponse`.
- `crates/oya-audit-chain-emission-domain` (IP-004): `CanonicalEnvelope`, `EmissionDomainError`.
- `crates/oya-audit-chain-sealing-kernel` (IP-006): `SigningKeyRef`, `PackEpoch`, `SealStatus`, `SealRecord`, `MerkleEngine`, `SignerPort`, `RootPublisher`, `IndexWriter`, `ObjectStoreWriter`.
- `crates/oya-audit-chain-sealing-api` (IP-010): `SealCycleCommand`, `SealCycleResult`.
- `crates/oya-audit-chain-sealing-domain` (IP-007): re-exports `oya_audit_chain_domain::{MerkleTree, Sha256Hash}` until extraction lands; `SealingDomainError`.
- `crates/oya-audit-chain-verification-kernel` (IP-011): `RootRegistry`, `KeyResolver`, `MerkleVerifier`.
- `crates/oya-audit-chain-verification-api` (IP-011): `VerificationFailureReason`, `VerificationVerdict`.
- `crates/oya-audit-chain-verification-domain` (IP-011): pure verifier hooks, re-exports verdict DTOs.
- `crates/oya-audit-chain-query-kernel` (IP-012): `AuditQueryRepository`, `ExportBuilder`, `AuditorEngagementResolver`.
- `crates/oya-audit-chain-query-api` (IP-012): `AuditQuery`, `QueryResult`, `QueryRow`, `ResultSealState`, `ExportBundle`, `AuditorEngagement`.
- `crates/oya-audit-chain-query-domain` (IP-012): pure validation; `QueryDomainError`.
- `crates/oya-audit-chain-retention-cascade-kernel` (IP-013): `RetentionPolicySource`, `DsrCascadeSource`, `RedactionWriter`.
- `crates/oya-audit-chain-retention-cascade-api` (IP-013): `RetentionPolicy`, `DsrCascade`, `RedactionToken`, `RetentionRun`.
- `crates/oya-audit-chain-retention-cascade-domain` (IP-013): `RetentionDomainError`; re-exports retention DTOs.

Workspace `Cargo.toml` updated: all 15 added as workspace members.

### IP claims NOT scaffolded (deferred to owning IP)

Per the dispatch prompt anti-pattern guidance ("Do NOT add code to crates that
are intentionally empty placeholders for future Wave-N work — flag in
REMEDIATION-NOTES instead"), 20 declared crates were NOT scaffolded here. They
require real I/O implementations (Postgres, S3 WORM, PKCS#11/HSM, Mimir
publication, GitHub-pinned root publication, Cedar evaluation, REST/HTTP
servers, SDK client logic, worker loops, composition roots). A premature stub
would either compile but panic at runtime, or hide cross-µservice import
violations under empty modules. They will land with their owning IPs:

- IP-005: `oya-audit-chain-emission-usecase`, `oya-audit-chain-emission-adapter`, `oya-audit-chain-emission-rest`, `oya-audit-chain-emission-sdk`, `oya-audit-chain-emission-app`.
- IP-008: `oya-audit-chain-sealing-adapter-hsm`.
- IP-009: `oya-audit-chain-sealing-adapter-postgres`, `oya-audit-chain-sealing-adapter-s3`, `oya-audit-chain-sealing-adapter` (umbrella).
- IP-010: `oya-audit-chain-sealing-usecase`, `oya-audit-chain-sealing-worker`, `oya-audit-chain-sealing-app`.
- IP-011: `oya-audit-chain-verification-rest`, `oya-audit-chain-verification-sdk`, `oya-audit-chain-verification-adapter`.
- IP-012: `oya-audit-chain-query-adapter-postgres`, `oya-audit-chain-query-adapter` (umbrella), `oya-audit-chain-query-rest`, `oya-audit-chain-query-sdk`, `oya-audit-chain-query-usecase`.
- IP-013: `oya-audit-chain-retention-cascade-usecase`, `oya-audit-chain-retention-cascade-worker`, `oya-audit-chain-retention-cascade-adapter`.

### IP claims trimmed

None. Inspection of IP-001 through IP-015 found every declared crate to be
within the audit-chain bounded context per `manifest.json` and ADR-0105's
13-layer enum. The IPs accurately describe what audit-chain owns; the gap is
implementation breadth, not scope creep. Trimming the IP claims would
contradict the manifest.

### IP-001 IaC artifacts: deferred, not trimmed

IP-001 declares OpenTofu modules under `microservices/audit-chain/iac/opentofu/`
(audit-postgres, audit-worm-store, audit-hsm-partition, pack-kr context) plus
Kustomize bases. These are configuration artifacts, not Rust crates, and
require a follow-on IaC dispatch (Wave 15-IMPL-IaC-truth-up) with reviewer
sign-off on OCI/AWS provider versions. Truth-up of the Rust crate claims does
not block on these.

### IP-002 / IP-015 observability artifacts: present

`manifest.json` SLO array (7 entries) all point at files under
`microservices/audit-chain/slos/`. Spot-checks confirmed the listed files exist
(`seal-write-latency.openslo.yaml`, `seal-cycle-latency.openslo.yaml`,
`merkle-chain-verification-latency.openslo.yaml`,
`chain-of-custody-integrity-correctness.openslo.yaml`, etc.). Dashboards
declared in IP-002/IP-015 live under `microservices/audit-chain/dashboards/`.
No scaffolding needed.

### Compile status

`cargo check` invoked for the 15 scaffolded crates in three parallel batches.
All three batches finished with `Finished dev profile`; no errors or warnings.

- Batch 1: emission-kernel, emission-api, emission-domain, sealing-kernel, sealing-api — PASS
- Batch 2: sealing-domain, verification-kernel, verification-api, verification-domain, query-kernel — PASS
- Batch 3: query-api, query-domain, retention-cascade-kernel, retention-cascade-api, retention-cascade-domain — PASS

### Follow-ups

1. IPs 005 / 008 / 009 / 010 / 011 / 012 / 013 still need their adapter, usecase, rest, sdk, worker, and app crates authored — these are the I/O-bearing surfaces.
2. `manifest.json` already lists every scaffolded crate name; no manifest update required.
3. The IP-007 RFC-6962 vs. length-prefixed domain-separation reconciliation (sealing-domain) remains an open IP-007 stop condition; the scaffolded `oya-audit-chain-sealing-domain` re-exports `oya_audit_chain_domain::MerkleTree` so the existing behavior is the source of truth until that reconciliation lands.
4. IP-014 cross-µservice emission SDK adoption is partially in place via `oya-shared-audit-chain-client-kernel`; no new scaffold needed here. Producer-side adoption (tenancy, observability, etc.) belongs to those µservices' own truth-up passes.
5. Journey IPs (IP-journey-j01 .. j148) were not re-inspected in this pass; they reference the same crate names and inherit the same truth-up outcome.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- None; inventory found no Redis references under microservices/audit-chain.

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- None.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture (ADR-0343): PRD now binds manifest RTO 300s/RPO 0s to the compliance-pack floors and names `runbooks/chain-replay-from-snapshot-protocol.md` plus seal/signature recovery runbooks. Alternative considered: relax to EU-AI/HIPAA floor RPO 300s; rejected because audit-chain manifest already requires zero-loss evidence continuity. Cost: failover tests must prove the stricter service target.
- Capacity model (ADR-0340): PRD now records manifest values: 0.22 vCPU, 384 MiB RAM, 8 GiB storage, 2 Valkey, 4 Postgres, 4 outbound/HSM slots, `per_message` scaling, Tier-0 evidence substrate/Tier-1 runtime placement, and pack/cell sealer boundaries. Alternative considered: `per_request`; rejected because D-2 manifest doctrine already names emitted messages as the scaling unit. Cost: hot tenant partitions need explicit shard-split admission.
- Sustainability + cost attribution (ADR-0344): PRD now requires emit/seal/verify/query/retention/export audit rows to carry cost/carbon/energy fields, with carbon routing excluded from synchronous evidence paths. Alternative considered: carbon-aware seal scheduling; rejected because delayed sealing weakens tenant and regulator proof. Cost: replay/export queues must split urgent evidence from carbon-deferrable jobs.
- API versioning posture (ADR-0342): PRD now requires public verifier/query/export/emitter date carriers, SDK semver, 3-version/180-day support, tenant pinning, and internal mesh exemption. Alternative considered: only SDK semver; rejected because auditors may use HTTP exports without SDK lockstep. Cost: verifier and export compatibility tests must cover the pinned date versions.

## Wave 15-doctrine-propagation-IPs (2026-05-21)

D4-BUCKET-1 trigger-based IP doctrine propagation.

- Root IPs scanned: 96
- Trigger A additions: 53
- Trigger B additions: 63
- Trigger C additions: 66
- Trigger D additions: 4
- Root IPs unmatched: 0
- Doctrine sources: ADR-0338, ADR-0342, ADR-0343, ADR-0344, ADR-0345; `specs/compliance-pack-floors.json`.
- Idempotence: skipped any IP section that already existed; no unmatched root IPs were edited.

IP-by-IP changes:
- `microservices/audit-chain/IP-001-storage-backend-iac.md`: added DR posture.
- `microservices/audit-chain/IP-002-self-slo-manifest.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-003-emission-kernel.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-004-emission-domain.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-005-emission-usecase-and-adapter.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-006-sealing-kernel.md`: added DR posture.
- `microservices/audit-chain/IP-007-sealing-domain-merkle.md`: added DR posture.
- `microservices/audit-chain/IP-008-sealing-adapter-hsm.md`: added DR posture.
- `microservices/audit-chain/IP-009-sealing-adapter-postgres-s3.md`: added DR posture.
- `microservices/audit-chain/IP-010-sealing-worker-app.md`: added DR posture.
- `microservices/audit-chain/IP-011-verification-stack.md`: added API Versioning, DR posture.
- `microservices/audit-chain/IP-012-query-stack.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-013-retention-cascade.md`: added DR posture.
- `microservices/audit-chain/IP-014-cross-microservice-emission-adapter.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-015-self-observability-slo-wiring.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j01-emergency-911-dispatch-emergency-classes.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j02-healthcare-code-blue-ehr-break-glass-classes.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j03-minor-safety-chain-of-custody.md`: added API Versioning, Sustainability emission.
- `microservices/audit-chain/IP-journey-j05-anonymous-chain-of-custody.md`: added API Versioning, Sustainability emission.
- `microservices/audit-chain/IP-journey-j06-publisher-only-custody-seal.md`: added API Versioning, Sustainability emission.
- `microservices/audit-chain/IP-journey-j07-inheritance-seal.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j100-pack-rollout-first-action.md`: added Sustainability emission.
- `microservices/audit-chain/IP-journey-j101-dual-seal-events.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j102-dual-seal-events.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j103-dual-seal-events.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j104-dual-seal-events.md`: added Sustainability emission.
- `microservices/audit-chain/IP-journey-j105-dual-seal-events.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j106-dual-seal-events.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j107-dual-seal-events.md`: added Sustainability emission.
- `microservices/audit-chain/IP-journey-j118-dual-tenant-read-seal.md`: added API Versioning.
- `microservices/audit-chain/IP-journey-j119-auction-award-seal.md`: added API Versioning, DR posture, Sustainability emission, Pod runtime tier.
- `microservices/audit-chain/IP-journey-j12-surge-bypass-accountability.md`: added API Versioning, Sustainability emission.
- `microservices/audit-chain/IP-journey-j124-bypass-and-reason-seal.md`: added API Versioning.
- `microservices/audit-chain/IP-journey-j125-dual-history-preservation.md`: added API Versioning, Sustainability emission.
- `microservices/audit-chain/IP-journey-j126-dual-tenant-emission-classes.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j129-warrant-query-emission.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j130-whistleblower-evidence-seal.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j131-region-local-seal.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j135-investigation-merkle-seal.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j137-corporate-internal-audit-sox-controls-test-evidence-bundler.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j138-corporate-audit-investigation-evidence-trail.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j139-internal-audit-cedar-permit-misuse-pattern-evidence.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j14-agent-action-seal.md`: added API Versioning, Sustainability emission.
- `microservices/audit-chain/IP-journey-j140-internal-audit-dlp-egress-evidence-trail.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j141-internal-audit-personal-tenant-boundary-deny-trail.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j143-epistemic-source-tagging.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j148-chain-of-custody-seal.md`: added API Versioning, DR posture, Pod runtime tier.
- `microservices/audit-chain/IP-journey-j15-disclosure-custody-seal.md`: added API Versioning, Sustainability emission.
- `microservices/audit-chain/IP-journey-j18-ncmec-chain-of-custody.md`: added API Versioning, Sustainability emission.
- `microservices/audit-chain/IP-journey-j19-shamir-reconstitution-seal.md`: added API Versioning, Sustainability emission.
- `microservices/audit-chain/IP-journey-j32-anonymous-proof-seal.md`: added Sustainability emission.
- `microservices/audit-chain/IP-journey-j33-admin-action-seals.md`: added Sustainability emission.
- `microservices/audit-chain/IP-journey-j38-regulator-seal.md`: added DR posture.
- `microservices/audit-chain/IP-journey-j43-hipaa-seal.md`: added DR posture.
- `microservices/audit-chain/IP-journey-j44-consult-seal.md`: added DR posture.
- `microservices/audit-chain/IP-journey-j45-record-correction-seal.md`: added DR posture.
- `microservices/audit-chain/IP-journey-j51-procure-to-pay-classes.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j55-dispute-seal.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j59-termination-seal.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j60-promotion-seal.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j61-hipaa-seal.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j62-prescription-seal.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j63-irb-hipaa-seal.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j64-baa-seal.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j65-dsar-proof.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j66-tax-seal.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j67-chain-of-custody.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j68-seal-service.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j70-ai-human-override-seal.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j71-fraud-seal.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j73-slsa-seal.md`: added API Versioning, DR posture, Sustainability emission, Pod runtime tier.
- `microservices/audit-chain/IP-journey-j75-revocation-seal.md`: added API Versioning, DR posture, Sustainability emission, Pod runtime tier.
- `microservices/audit-chain/IP-journey-j76-sealed-evidence-chain.md`: added API Versioning.
- `microservices/audit-chain/IP-journey-j77-sealed-evidence-chain.md`: added API Versioning, DR posture.
- `microservices/audit-chain/IP-journey-j78-sealed-evidence-chain.md`: added API Versioning.
- `microservices/audit-chain/IP-journey-j79-sealed-evidence-chain.md`: added API Versioning.
- `microservices/audit-chain/IP-journey-j80-sealed-evidence-chain.md`: added API Versioning.
- `microservices/audit-chain/IP-journey-j81-sealed-evidence-chain.md`: added API Versioning.
- `microservices/audit-chain/IP-journey-j82-sealed-evidence-chain.md`: added API Versioning, DR posture.
- `microservices/audit-chain/IP-journey-j83-sealed-evidence-chain.md`: added API Versioning.
- `microservices/audit-chain/IP-journey-j84-sealed-evidence-chain.md`: added API Versioning, DR posture.
- `microservices/audit-chain/IP-journey-j85-sealed-evidence-chain.md`: added API Versioning.
- `microservices/audit-chain/IP-journey-j86-sealed-evidence-chain.md`: added API Versioning, DR posture.
- `microservices/audit-chain/IP-journey-j87-sealed-evidence-chain.md`: added API Versioning.
- `microservices/audit-chain/IP-journey-j88-sealed-evidence-chain.md`: added API Versioning.
- `microservices/audit-chain/IP-journey-j89-sealed-evidence-chain.md`: added API Versioning.
- `microservices/audit-chain/IP-journey-j90-sealed-evidence-chain.md`: added API Versioning.
- `microservices/audit-chain/IP-journey-j91-us-msb-mtl-overlay.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j92-br-lgpd-us-parent-dsar.md`: added Sustainability emission.
- `microservices/audit-chain/IP-journey-j93-in-dpdpa-rbi-overlay.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j94-sox404-public-company-controls.md`: added DR posture, Sustainability emission.
- `microservices/audit-chain/IP-journey-j95-iso27001-soc2-annual-audit.md`: added Sustainability emission.
- `microservices/audit-chain/IP-journey-j96-ksa-uae-mena-onboarding.md`: added Sustainability emission.
- `microservices/audit-chain/IP-journey-j97-sg-pdpa-mas-tenant.md`: added Sustainability emission.
- `microservices/audit-chain/IP-journey-j98-au-privacy-apra-cps234.md`: added Sustainability emission.
- `microservices/audit-chain/IP-journey-j99-multi-pack-conflict-resolution.md`: added Sustainability emission.
## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: baseline_cpu_per_tenant 0.22 vCPU; baseline_ram_per_tenant 384 MiB; storage_per_tenant 8 GB; connections valkey=2, postgres=4, outbound_http=4; scaling_dimension per_message; cell_placement_class Tier-0.
- ADR: ADR-0340 capacity_model; ADR-0248 cellular criticality numbering.
- Why: Audit-chain ingests every security and compliance event, then seals and stores evidence, so per-message CPU, storage, and database write headroom dominate.
- Rejected: cell_placement_class=Tier-1 because ADR-0340 names audit-chain primary as Foundation Tier-0.
- Cost: Foundation placement and larger storage reserve are accepted to preserve non-repudiation.

### Block 2: dr
- Values: rto_p99_seconds 300; rpo_p99_seconds 0; multi_region_active_active true; backup_substrate postgres_wal_g, object_storage_versioned, audit_chain_merkle_seal, openbao_seal_unseal; failover_runbook runbooks/chain-replay-from-snapshot-protocol.md; replication_shape active-active-multi-az-cross-region-warm.
- ADR: ADR-0343 DR RTO/RPO matrix and compliance-pack floors.
- Why: Audit-chain is the evidence root for other services; losing acknowledged audit events invalidates downstream compliance claims.
- Rejected: RPO=300 seconds because a five-minute gap would break non-repudiation for dependent incident and regulator evidence.
- Cost: Zero-RPO sealing requires replicated write paths and replay drills, not just object snapshots.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier 1; evidence microservices/audit-chain/PRD.md, microservices/audit-chain/ARCHITECTURE.md, microservices/audit-chain/IP-006-sealing-kernel.md, microservices/audit-chain/IP-011-verification-stack.md, microservices/audit-chain/runbooks/chain-replay-from-snapshot-protocol.md.
- ADR: ADR-0338 pod runtime tiering; ADR-0340 D-6 cell/runtime co-variance.
- Why: Audit-chain is a tenant-data-touching foundation substrate that receives, seals, and replays audit events for regulated tenants. It does not execute tenant code, but its data plane is sensitive enough to require Tier 1 runtime isolation.
- Rejected: pod_runtime_tier=2 because audit evidence payloads and cryptographic seals are tenant data plane, not ordinary first-party app state.
- Cost: Tier 1 isolation adds sealing-path capacity overhead and tighter scheduling constraints.

### Block 4: tenant_version_pinning
- Values: declared_versions 2026-05-21, 2026-02-21, 2025-11-21; default_version 2026-05-21; supported_window_size 3; supported_window_minimum_days 180; supports_per_tenant_pinning true.
- ADR: ADR-0342 hybrid date-versioned public API policy.
- Why: Audit producers and auditors depend on stable event, query, and proof contracts.
- Rejected: one latest-only audit contract because evidence producers across tenants cannot all migrate at once.
- Cost: Maintains replay and query compatibility for three date windows.

### Block 5: consumes_upstream_oss and oss_stewardship_class_overrides
- Values: consumes_upstream_oss postgresql, openbao, opentelemetry, cedar, cilium, istio, kyverno; oss_stewardship_class_overrides empty because registry-default stewardship applies.
- ADR: ADR-0345 OSS stewardship class and CVE response policy.
- Why: Audit-chain depends on registry-governed persistence, key custody, policy admission, telemetry, and mesh substrates.
- Rejected: service-local stewardship overrides without a registry delta.
- Cost: CVE response is shared with registry owner teams; no local stewardship override is justified.

### Block 6: iac_module_invocations
- Values: oci-guest/postgresql-cluster@v1, on-prem/object-storage-versioned@v1, colo/openbao-secret-binding@v1, oyatie-as-cloud-provider/audit-chain-merkle-seal@v1.
- ADR: ADR-0339 shared IaC module library.
- Why: Sealing, storage, and key custody must be provisioned identically across regulated deployment contexts.
- Rejected: bespoke evidence store modules because seal and replay semantics must be shared.
- Cost: Audit-chain infra changes now need shared module version promotion.
