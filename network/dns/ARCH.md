---
doc_class: Architecture
template_id: TPL-ARCH
arch_id: ARCH-cloud-network-dns
microservice: cloud-network-dns
status: wave-15-zf-doctrine-propagation
date: 2026-05-21
owner_team: axis-cloud-network
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0243
  - ADR-0263
  - ADR-0346
  - ADR-0347
  - ADR-0348
  - ADR-0349
companion_docs:
  - network/dns/manifest.json
  - network/dns/PRD.md
  - network/dns/README.md
---

# Architecture: Cloud Network Dns

## Architecture Boundary

`cloud-network-dns` keeps its existing bounded context and flat `microservices/cloud-network-dns/src/` ownership under ADR-0131 and ADR-0132. This `ARCH.md` is the Wave 15-ZF architecture propagation surface for ADR-0346, ADR-0347, ADR-0348, and ADR-0349; service-specific deep architecture remains in `ARCHITECTURE.md` when that artifact exists.

## Wave 15-ZF Doctrine Context

This architecture artifact carries doctrine propagation for ADR-0346, ADR-0347, ADR-0348, and ADR-0349 only. It does not implement Wave 15-ZA, Wave 15-ZB, Wave 15-ZD, or Wave 15-ZE bodies.

### ADR-0346 Local CI Mirror
- `oya verify --ci-required` is the canonical local pre-push verifier for this microservice's future architecture changes.
- The verifier MUST locally mirror the full CI matrix and MUST block on exit-0 of EACH mandatory mirror step before success: cargo fmt, cargo check, cargo clippy, cargo nextest, and `oya gate run-all --ci-required`.
- Architecture changes that add generated docs, manifests, contracts, runbooks, or CI surfaces must assume the `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, and `oya-governance-oya-submit-calls-verify` lanes protect the local-to-CI contract.

### ADR-0347 Governance Lane Prefix
- Governance-owned fitness lanes for this microservice use the `oya-governance-*` prefix. The canonical vocabulary is enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- Any architecture reference to CI lane ownership must point at the governance prefix and preserve lane invariants, lane checks, and lane semantics across the rename surface.

### ADR-0348 Sharding Automation Context
- This microservice participates in the manifest-level `sharding_automation` doctrine: autosharding, auto_rebalance, and dynamic_sharding sub-blocks are declared per the D-1 schema unless an explicit cellular exemption applies.
- AUTOSHARDING is control-plane-driven tenant-to-cell/shard placement using capacity_model, compliance_pack constraints, ResidencyClass, cell_placement_class, and the oya-shuffle-sharding algorithm. No human operator picks placement.
- AUTO-REBALANCE migrates tenants from hot cells to cooler cells when cell load skews beyond promotion-gate criteria. Migration honors residency and compliance pack constraints; cross-jurisdiction migration requires an explicit Cedar permit and emits audit-chain evidence per ADR-0263.
- DYNAMIC SHARDING adjusts shard count within a cell by HOT-SPLIT when shard p99 latency exceeds SLO or utilization exceeds 80 percent, and by COLD-MERGE when adjacent shards both run below 20 percent utilization for more than 24 hours; per-microservice overrides must be explicit.
- Relevant admission lanes are `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, and `oya-governance-audit-chain-emit-on-automation-events`.

### ADR-0349 Jenkins And ArgoCD CI/CD Context
- Jenkins LTS and ArgoCD are the canonical self-hostable CI/CD substrates for this microservice across air-gap, on-prem, colo, and Oyatie-as-provider deployment contexts.
- GitHub Actions remains the hosted PR CI surface; Jenkins augments it in self-hosted contexts with JCasC plus Jenkinsfile parity enforced by `oya-governance-jenkins-github-actions-parity`.
- ArgoCD is the GitOps CD orchestrator. Application syncs verify cosign signatures per ADR-0181, emit audit-chain rows per ADR-0263, and preserve tenant namespace isolation through Cedar per ADR-0243.
- CI/CD architecture references must preserve `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit` as acceptance context.

## ADR-0339 integration
- Integration state: PROPOSED for `cloud-network-dns`; ACCEPTED waits for service wrapper implementation and signed module evidence.
- Ownership split: cloud-iac owns reusable OpenTofu primitive bodies; `cloud-network-dns` owns wrapper selection, variables, SLO-driven sizing, and blast-radius review.
- Current manifest pins: aws-guest/route53-zone@v1[both], oci-guest/vcn@v1[both], oyatie-as-cloud-provider/cell-zone@v1[both], oyatie-as-cloud-provider/tenant-namespace@v1[both].
- Wrapper shape: each `iac/<context>/main.tf` contains module, variable, output, terraform, and provider declarations only.
- Resource-body rule: no service-local shared resource bodies are introduced; common provider wiring belongs under cloud-iac modules.
- Cell placement: `Tier-4` is passed as placement intent under ADR-0248 and ADR-0341.
- Runtime isolation: pod runtime tier `3` informs module nodepool selection under ADR-0338.
- Capacity: `per_request` drives CPU `0.06`, RAM `128`, storage `1`, and connection pool variables.
- DR: RTO `300` seconds and RPO `60` seconds constrain backup and failover primitives.
- Sharding: autosharding `control_plane_driven` and explicit auto_rebalance/dynamic_sharding thresholds remain manifest-driven.
- Supply chain: every module pin carries ADR-0181 cosign evidence before blocker-mode consumption.
- Versioning: module semantic versions are independent from public API date versions and SDK semver releases.
- Observability: module releases emit cost, carbon, tenant, cell, primitive, and version labels for ADR-0344 FinOps review.
- Security: wrappers pass tenant_class and compliance-pack labels to prevent demo_trial, paid, regulated, and BYOK paths from sharing defaults.
- Blast radius: primitive updates are reviewed once in cloud-iac and then consumed by explicit per-service pin movement.
- Five-context posture: aws-guest, oci-guest, oci-guest/always-free, on-prem, colo, and oyatie-as-cloud-provider remain separate wrapper contexts.
- OCI Always Free: any always-free invocation is demo_trial-only and cannot silently inherit paid-tenant features.
- On-prem and colo: modules encode kubeadm, Cilium, Istio Ambient, Envoy Gateway, OpenBao, PostgreSQL, and Valkey substrate choices where selected.
- Oyatie-as-provider: modules encode cell-zone, shard-cell, tenant namespace, per-cell nodepool, observability, audit-chain, KMS, and Cedar bundle primitives where selected.
- Contract impact: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 files remain unchanged in this document-stage wave.
- Review boundary: architecture acceptance requires the IP line floor, manifest field, PRD section, ADR-citation gate, cohesion gate, and refreshed doc inventory.
- Implementation boundary: no Rust code, crate metadata, OpenTofu body, Helm chart, ArgoCD Application, or live infrastructure apply is part of this propagation.
