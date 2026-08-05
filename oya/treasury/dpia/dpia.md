---
doc_class: DPIA
doc_id: DPIA-TREASURY
microservice: treasury
status: preview-planning-non-authoritative
date: 2026-05-21
owner_team: council-compliance + axis-governance
bounded_context: automation-event-driven-data-flow
implementation_phase: doctrine-propagation-only
rust_code_status: not-authored-in-this-wave
authority_ref: ../../../specs/microservices/treasury.json
runtime_claim: false
source_adrs:
  - ADR-0346
  - ADR-0347
  - ADR-0348
  - ADR-0349
---

# Data Protection Impact Assessment: treasury
## automation-event-driven-data-flow

### Authority Boundary
This DPIA entry is preview planning/control inventory only under `specs/microservices/treasury.json`. It does not claim live data processing, live bank connectivity, payment execution, durable persistence/Postgres/RLS, Workflow execution, runtime audit-chain emission, cloud deployment, measured SLO/DR readiness, GA, or production readiness.

### Processing Trigger
- Scope: Wave 15-ZF doctrine propagation only; no new runtime processing is implemented in this DPIA entry.
- Event surface: future-planning automation event names for this microservice include local verifier outcomes, governance lane rename evidence, sharding automation decisions, and CI/CD transition evidence; they are not current runtime audit-chain emissions.
- Data subjects: tenant administrators, operator principals, service principals, and tenant users whose identifiers may appear in tenant-scoped operational evidence.

### Data Categories
- tenant_id, service_id, cell_id, shard_id, region_id, ResidencyClass, compliance_pack, and cell_placement_class.
- Cedar decision id, audit-chain event id, verifier run id, GitHub Actions `oya-ci-required` status URL, optional owned-runner cutover receipt, ArgoCD sync id, cosign verification result, and governance lane id.
- No tenant payload, PHI, payment card data, message body, document body, or secret material is introduced by this doctrine propagation entry.

### ADR-Bound Controls
- ADR-0346 (amended by ADR-0515): legacy `oya verify` / `./bin/oya verify --ci-required` output is optional local-feedback/provenance only; protected-branch merge authority is the GitHub Actions + branch-protection `oya-ci-required` context produced by cloud-ci Rust gate packets.
- ADR-0347: every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB). Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348: AUTOSHARDING means tenant→cell/shard placement is computed by the control plane automatically; AUTO-REBALANCE automatically migrates tenants from hot cells to cooler cells; DYNAMIC SHARDING adjusts shard count within a cell based on load. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349: Jenkins CI wording is historical/provenance after ADR-0515; GitHub Actions produces `oya-ci-required` until explicit owned-runner cutover, and ArgoCD remains the separately authorized GitOps CD evidence surface where applicable. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

### DPIA Assessment
- Necessity: future automation evidence would be necessary to prove verifier completeness, governance lane vocabulary, tenant placement, rebalance reversibility, dynamic shard threshold decisions, and CI/CD deployment accountability before any runtime promotion.
- Proportionality: records are limited to identifiers, decision ids, evidence pointers, and signed event metadata; payload data remains in the source microservice and is not copied into automation evidence.
- Residency and compliance: future auto-rebalance must honor residency and compliance pack constraints; cross-jurisdiction movement would require explicit Cedar permit and audit-chain evidence before promotion.
- Access control: service principals, GitHub Actions/owned-runner principals, ArgoCD principals, and cell-orchestrator principals are least-privilege actors governed by Cedar and tenant namespace isolation.
- Retention and rights: future tenant-scoped automation history must support access/portability exports; any future immutable audit-chain rows would be retained under the applicable legal basis and referenced rather than deleted.

### Residual Risk
- Residual risk is medium until Wave 15-ZA/ZB/ZD/ZE implementation PRs land ADR-0515 `oya-ci-required` evidence, governance rename, sharding automation bodies, and current CD/owned-runner substrate files.
- Residual risk becomes low when ADR-0346 references are provenance-only, ADR-0347/ADR-0348 lanes pass, and ADR-0515 `oya-ci-required` evidence is present for this microservice.
