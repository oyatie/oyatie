---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-cloud-network
microservice: cloud-network
status: Drafting
sales_segment: cloud-provider-substrate
tier: internal
milestone_first_ship: M02-cloud-substrate
related_adrs: [ADR-0338, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
related_specs: [/specs/compliance-pack-floors.json, /specs/finops-dimensional-model.json]
date: 2026-05-21
owner_team: axis-cloud-network
doc_status: drafted
---

# PRD-cloud-network: Tenant Network Control Plane

## Purpose

`cloud-network` owns tenant-scoped VPC-equivalent networking, ingress/egress policy, mTLS enforcement, flow telemetry, and network isolation across Oyatie cells and deployment contexts. Current service-local evidence is `README.md`, `feature-parity-matrix-2026-05-20.md`, FAQ/runbook material, Rust domain sources, and the D-2 `manifest.json` doctrine fields.

## Functional Requirements

- Provision tenant network spaces, subnets, route tables, private endpoints, NAT/egress policy, load-balancer attachments, and network-security policy.
- Enforce mTLS and tenant/cell isolation on east-west and north-south paths.
- Emit flow telemetry and privileged network-control actions to `audit-chain`.
- Provide reachability, route-health, DDoS, and cross-cell routing diagnostics.
- Support `demo_trial` and `paid` tenant_class envelopes through capacity, context, and compliance-pack policy rather than customer ladder labels.

## Non-Functional Requirements

### DR posture (ADR-0343)

- Target: RTO ≤600s and RPO ≤300s for network control-plane state, route policies, mTLS policy, and flow-telemetry checkpoints, matching manifest `dr.rto_p99_seconds=600` and `dr.rpo_p99_seconds=300`.
- Compliance-pack floors considered: EU-AI-ACT-2024-HIGH-RISK (1800s/300s, multi-region), HIPAA-2024 (3600s/300s, multi-region), KR-CSAP-v3.1 (3600s/900s, multi-region), SOC2-T2 (14400s/900s), PCI-DSS-L1-v4 (86400s/3600s), ISO27001-2022/SOX-404 (14400s/3600s), and KR-PIPA-2023-amendment (14400s/900s). Effective target is RTO 600s, RPO 300s, multi-region for regulated cells.
- Failover runbook: `network/runbooks/network-control-plane-failover.md`, matching manifest `dr.failover_runbook`; mTLS and edge-attack recovery use `network/runbooks/mtls-handshake-failure-cascade.md` and `network/runbooks/ddos-mitigation-engagement.md`.
- Multi-region active-active: yes for control-plane policy and route intent; data-plane forwarding keeps last-known-good policy while control cells recover.
- WHY: tenants keep private connectivity and enforceable isolation during control-plane loss without accepting silent route or firewall drift.

### Capacity model (ADR-0340)

- Per-tenant baseline: 0.14 vCPU, 256 MiB RAM, 2 GiB route/security/flow metadata, 2 Valkey connections, 2 Postgres connections, and 10 outbound controller/mesh/router slots, matching manifest `capacity_model`.
- Scaling dimension: `per_capability`, because manifest doctrine treats VPC, route, LB, security-rule, and telemetry work as network-capability-object shaped.
- Cell placement class: Tier-1 network substrate, matching manifest `capacity_model.cell_placement_class`; runtime placement maps to pod runtime Tier 1 because manifest `pod_runtime_tier=1`.
- Autoscaling boundary: minimum 2 controllers, 2 route evaluators, and 1 telemetry writer per cell; maximum 20 control-plane workers per high-churn tenant before network namespace sharding is required.
- WHY: the model serves VPC-style churn and sustained flow telemetry without coupling one tenant's route storm to another tenant's connectivity.

### Sustainability + cost attribution (ADR-0344)

- Every VPC, subnet, route, firewall, endpoint, mTLS policy, DDoS mitigation, and privileged flow-log access audit row emits `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, and `carbon_intensity_source`.
- Provider-routing affected by carbon: no for live routing, mTLS, DDoS mitigation, failover, or HIPAA/PCI realtime paths; yes for scheduled topology analysis and non-urgent reporting jobs.
- Per-tenant cost surface: the tenant FinOps dashboard exposes network cost/carbon by tenant, product, capability, provider, cell, and compliance_pack, with egress and flow-telemetry shown as separate capability filters.
- WHY: network substrate spend and emissions are a major cloud-provider bill driver, and regulated tenants need attribution without letting carbon routing delay protective controls.

### API versioning posture (ADR-0342)

- Public API version model: network, route, security-group, endpoint, flow-log, and diagnostics APIs use the YYYY-MM-DD carrier triplet: `Oyatie-Version` header, `/v/<YYYY-MM-DD>/...` URL prefix, and `oyatie_version` proto3 field.
- SDK semver model: cloud-network SDKs ship as major.minor.patch, with each minor mapping to supported public date versions.
- Support window: last 3 public API versions for at least 180 days.
- Per-tenant pinning: yes for tenant network-management clients and migration tools.
- Internal-mesh exemption: yes; direct gRPC for mesh policy propagation remains exempt under ADR-0145.

## Source Notes

- `manifest.json` is present and the PRD values above mirror its `pod_runtime_tier`, `dr`, and `capacity_model` fields.
- ADR-0339 is not cited because this service path currently has no `iac/<context>/` directory.
- ADR-0337 is not cited because current evidence does not show cloud-network writing OLAP through the canonical data-warehouse path.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `cloud-network` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `cloud-network` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 4 module pin(s) across 3 context(s).
- Scaling input: `per_capability` with cell placement `Tier-1` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
