---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-cloud-network-dns
microservice: cloud-network-dns
status: Drafting
sales_segment: cloud-provider-substrate
tier: internal
milestone_first_ship: M02-cloud-substrate
related_adrs: [ADR-0338, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
related_specs: [/specs/compliance-pack-floors.json, /specs/finops-dimensional-model.json]
date: 2026-05-21
owner_team: axis-cloud-network-dns
doc_status: drafted
---

# PRD-cloud-network-dns: DNS Control and Anycast Data Plane

## Purpose

`cloud-network-dns` owns authoritative and recursive DNS, zone scoping, DNSSEC, health checks, routing policy, encrypted DNS transports, and anycast advertising. Current service-local evidence is `README.md`, `feature-parity-matrix-2026-05-20.md`, reference implementations, migration playbooks, onboarding material, and the D-2 `manifest.json` doctrine fields.

## Functional Requirements

- Manage public and private hosted zones, records, delegation, DNSSEC signing, routing policies, and health-check backed failover.
- Provide tenant-scoped recursive resolution controls, resolver firewall policy, encrypted DNS transport, and query logging.
- Emit zone, resolver, DNSSEC, health-check, and privileged query-log actions to `audit-chain`.
- Keep authoritative data-plane anycast independent from regional control-plane recovery.

## Non-Functional Requirements

### DR posture (ADR-0343)

- Target: RTO ≤300s and RPO ≤60s for zone control-plane state, DNSSEC key metadata, routing policies, and health-check configuration, matching manifest `dr.rto_p99_seconds=300` and `dr.rpo_p99_seconds=60`.
- Compliance-pack floors considered: EU-AI-ACT-2024-HIGH-RISK (1800s/300s, multi-region), HIPAA-2024 (3600s/300s, multi-region), KR-CSAP-v3.1 (3600s/900s, multi-region), SOC2-T2 (14400s/900s), PCI-DSS-L1-v4 (86400s/3600s), ISO27001-2022/SOX-404 (14400s/3600s), and KR-PIPA-2023-amendment (14400s/900s). Effective target is RTO 300s, RPO 60s, multi-region for regulated zones.
- Failover runbook: `network/dns/runbooks/dns-zone-failover.md`, matching manifest `dr.failover_runbook`; migration cutover remains documented in `network/dns/migration-playbooks/from-route53-and-ns1.md`.
- Multi-region active-active: yes for zone intent, health-check policy, and authoritative publication; recursive resolver caches use bounded TTLs and refresh from the nearest healthy control cell.
- WHY: tenant endpoints remain discoverable during regional failure, and DNS failover decisions keep legal auditability instead of becoming manual console edits.

### Capacity model (ADR-0340)

- Per-tenant baseline: 0.06 vCPU, 128 MiB RAM, 1 GiB hosted-zone and query-log index storage, 3 Valkey connections, 1 Postgres connection, and 4 outbound health-check/DNSSEC/route slots, matching manifest `capacity_model`.
- Scaling dimension: `per_request`, because load tracks record mutations, query volume, resolver policy checks, DNSSEC signing, and health-check updates.
- Cell placement class: Tier-4 edge data plane, matching manifest `capacity_model.cell_placement_class`; runtime placement maps to pod runtime Tier 3 because manifest `pod_runtime_tier=3`.
- Autoscaling boundary: minimum 2 control-plane replicas, 3 authoritative edge publishers, and 2 resolver-policy workers per cell; maximum 24 edge publishers and 12 resolver workers per high-query tenant before zone/query sharding is required.
- WHY: the model handles bursty DNS query traffic and traffic-shift events without coupling tenant zone management to global resolver saturation.

### Sustainability + cost attribution (ADR-0344)

- Every zone mutation, DNSSEC signing action, resolver policy update, health-check state change, routing-policy change, and privileged query-log read emits `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, and `carbon_intensity_source`.
- Provider-routing affected by carbon: no for live DNS resolution, health-check failover, DNSSEC key operations, or regulated realtime paths; yes for batch query analytics and non-urgent zone export jobs.
- Per-tenant cost surface: the tenant FinOps dashboard exposes DNS cost/carbon by zone, query volume, capability, provider, cell, and compliance_pack.
- WHY: DNS is a high-volume edge service; tenants need transparent query-cost and carbon attribution without slowing resolution or failover.

### API versioning posture (ADR-0342)

- Public API version model: zone, record, resolver, DNSSEC, health-check, routing-policy, and query-log APIs use the YYYY-MM-DD carrier triplet: `Oyatie-Version` header, `/v/<YYYY-MM-DD>/...` URL prefix, and `oyatie_version` proto3 field.
- SDK semver model: cloud-network-dns SDKs ship as major.minor.patch with explicit support for date-versioned API contracts.
- Support window: last 3 public API versions for at least 180 days.
- Per-tenant pinning: yes for hosted-zone management clients and migration tooling.
- Internal-mesh exemption: yes; direct mesh publication and health propagation gRPC remains exempt under ADR-0145.

## Source Notes

- `manifest.json` is present and the PRD values above mirror its `pod_runtime_tier`, `dr`, and `capacity_model` fields.
- ADR-0339 is not cited because this service path currently has no `iac/<context>/` directory.
- ADR-0337 is not cited because current evidence does not show cloud-network-dns writing OLAP through the canonical data-warehouse path.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `cloud-network-dns` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `cloud-network-dns` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 4 module pin(s) across 3 context(s).
- Scaling input: `per_request` with cell placement `Tier-4` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
