---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-cloud-storage
microservice: cloud-storage
status: Drafting
sales_segment: cloud-provider-substrate
tier: internal
milestone_first_ship: M02-cloud-substrate
related_adrs: [ADR-0338, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
related_specs: [/specs/compliance-pack-floors.json, /specs/finops-dimensional-model.json]
date: 2026-05-21
owner_team: axis-cloud-storage
doc_status: drafted
---

# PRD-cloud-storage: Object, Block, and File Storage Substrate

## Purpose

`cloud-storage` owns Oyatie storage control and data-plane capabilities for bucket/object, block volume, file share, archive, backup, restore, KMS binding, lifecycle, policy, quota, SLO, and billing surfaces. Current service-local evidence is the coherence audit, FAQ, tutorials, benchmarks, reference implementations, migration playbooks, and the D-2 `manifest.json` doctrine fields.

## Functional Requirements

- Provide tenant-scoped bucket, object, versioning, lifecycle, replication, object-lock, retention, quota, and billing APIs.
- Extend the same ownership boundary to block volume, file share, archive, backup, restore, and KMS-envelope policy as the service matures.
- Emit object/control-plane state changes, retention decisions, restores, lifecycle actions, and privileged reads to `audit-chain`.
- Support S3-compatible migration where useful without letting provider-specific semantics replace Oyatie's storage resource model.

## Non-Functional Requirements

### DR posture (ADR-0343)

- Target: RTO ≤3600s and RPO ≤300s for bucket/object metadata, lifecycle state, replication intent, KMS binding metadata, and restore queues, matching manifest `dr.rto_p99_seconds=3600` and `dr.rpo_p99_seconds=300`.
- Compliance-pack floors considered: EU-AI-ACT-2024-HIGH-RISK (1800s/300s, multi-region), HIPAA-2024 (3600s/300s, multi-region), KR-CSAP-v3.1 (3600s/900s, multi-region), SOC2-T2 (14400s/900s), PCI-DSS-L1-v4 (86400s/3600s), ISO27001-2022/SOX-404 (14400s/3600s), and KR-PIPA-2023-amendment (14400s/900s). Effective target is RTO 3600s, RPO 300s, multi-region for regulated buckets.
- Failover runbook: `storage/runbooks/storage-replication-failover.md`, matching manifest `dr.failover_runbook`; migration cutover remains documented in `storage/migration-playbooks/from-s3-and-azure-blob.md`.
- Multi-region active-active: yes for metadata/control-plane intent and regulated-bucket replication policy; object data locality follows tenant pack and bucket residency constraints.
- WHY: tenants can preserve object durability, retention evidence, and restore ability through region loss without violating pack residency or object-lock guarantees.

### Capacity model (ADR-0340)

- Per-tenant baseline: 0.3 vCPU, 768 MiB RAM, 100 GiB storage allowance, 3 Valkey connections, 3 metadata-store/Postgres connections, and 8 outbound KMS/replication/audit slots, matching manifest `capacity_model`.
- Scaling dimension: `per_request`, because hot load follows object reads/writes, lifecycle transitions, inventory, restore, and replication operations; storage bytes remain quota-governed.
- Cell placement class: Tier-1 storage substrate, matching manifest `capacity_model.cell_placement_class`; runtime placement maps to pod runtime Tier 1 because manifest `pod_runtime_tier=1`.
- Autoscaling boundary: minimum 2 API replicas, 2 metadata workers, 1 lifecycle worker, and 1 replication worker per cell; maximum 24 API replicas and 12 metadata/lifecycle workers per high-throughput tenant before bucket/prefix sharding is required.
- WHY: the model keeps object I/O and lifecycle work isolated by tenant while allowing storage bytes to scale separately from request workers.

### Sustainability + cost attribution (ADR-0344)

- Every object PUT/GET privileged read, DELETE, lifecycle transition, replication, restore, object-lock, quota, KMS binding, and backup action emits `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, and `carbon_intensity_source`.
- Provider-routing affected by carbon: no for online object reads/writes, legal-hold, emergency restore, HIPAA emergency, or PCI realtime contexts; yes for lifecycle tiering, inventory, archive transitions, and batch replication when pack policy permits.
- Per-tenant cost surface: storage admins and FinOps users see cost/carbon by bucket, prefix, storage class, capability, provider, cell, and compliance_pack.
- WHY: storage is a high-cost and high-energy substrate, so tenant billing and climate disclosures must be explainable at bucket and lifecycle granularity.

### API versioning posture (ADR-0342)

- Public API version model: native storage APIs and control-plane extensions use the YYYY-MM-DD carrier triplet: `Oyatie-Version` header, `/v/<YYYY-MM-DD>/...` URL prefix, and `oyatie_version` proto3 field. S3/GCS/Azure-compatible adapters translate their provider-native version behavior into the native control plane.
- SDK semver model: cloud-storage SDKs ship as major.minor.patch and publish compatibility with both native date versions and adapter compatibility profiles.
- Support window: last 3 public API versions for at least 180 days.
- Per-tenant pinning: yes for native storage APIs and migration adapters.
- Internal-mesh exemption: yes; replication, KMS, and lifecycle gRPC between Oyatie services remains exempt under ADR-0145.

## Source Notes

- `manifest.json` is present and the PRD values above mirror its `pod_runtime_tier`, `dr`, and `capacity_model` fields.
- ADR-0339 is not cited because this service path currently has no `iac/<context>/` directory.
- ADR-0337 is not cited because current service-local evidence frames cloud-storage as storage substrate, not as an OLAP writer through the canonical data-warehouse path.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `cloud-storage` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `cloud-storage` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 4 module pin(s) across 4 context(s).
- Scaling input: `per_request` with cell placement `Tier-1` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
