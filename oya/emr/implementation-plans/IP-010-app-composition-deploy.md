---
ip_id: IP-EMR-010
title: Top-level app composition + cell deployment manifests
microservice: emr
status: planned
date: 2026-05-21
sequence: 10
depends_on: [IP-EMR-004, IP-EMR-005, IP-EMR-006, IP-EMR-007, IP-EMR-008, IP-EMR-009]
unblocks: []
estimated_effort_hours: 50
owner: axis-emr + ops-sre-reliability
---

# IP-EMR-010: App composition + deployment

## Goal

Compose the EMR µservice binary (`oya-emr-app`) that mounts the REST server, gRPC servers, AsyncAPI publishers + consumers, workers, healthchecks, metrics, and config loader. Author the 6 deployment-context OpenTofu modules for cell rollout.

## Deliverables

- Crate `oya-emr-app` — single binary composition root.
- Distroless multi-arch container image (linux/amd64 + linux/arm64).
- Per-OS package outputs (rpm + deb) per `supported-oses.json`.
- 6 deployment-context OpenTofu modules under `iac/<context>/`:
  - `oyatie-public-cloud`
  - `guest-on-aws`
  - `guest-on-oci`
  - `on-prem`
  - `colo`
  - `oyatie-as-cloud-provider`
- Helm chart `iac/helm/emr/` with values per cell.
- Kustomize overlay `iac/kustomize/<env>/`.

## Acceptance criteria

- `cargo build -p oya-emr-app --release` exits 0.
- `docker build` exits 0 on each architecture.
- `tofu apply` against a sandbox tenant in each deployment context succeeds.
- Helm chart `helm install` succeeds in a kind cluster.
- Cell-rollout smoke test (chart-open + order-entry + audit-emit) passes.
- SLOs from `microservices/emr/slos/` registered in observability µservice.

## Out of scope

- Production rollout (operations runbook).
- Per-tenant onboarding (separate per-tenant IP per pilot).

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/emr/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/emr/implementation-plans/IP-010-app-composition-deploy.md:42` - - SLOs from `microservices/emr/slos/` registered in observability µservice..

## Pod runtime tier (per ADR-0338)

- Binding ADR: ADR-0338.
- `pod_runtime_tier: 0`.
- Runtime class: Kata Containers + Cloud Hypervisor (`kata-cloud-hypervisor`) is required for this execution path.
- Justification: Trigger D matched a sandbox/plugin/workflow/capability surface; treat the execution path as tenant-customer or third-party code until a narrower manifest declaration proves otherwise.
- Surface evidence: `microservices/emr/implementation-plans/IP-010-app-composition-deploy.md:39` - - `tofu apply` against a sandbox tenant in each deployment context succeeds..
