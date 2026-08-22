---
doc_class: DPIA
doc_id: DPIA-FINOPS_PORTAL
microservice: finops-portal
status: wave-15-zf-doctrine-propagation
date: 2026-05-21
owner_team: council-compliance + axis-governance
bounded_context: automation-event-driven-data-flow
implementation_phase: doctrine-propagation-only
rust_code_status: not-authored-in-this-wave
source_adrs:
  - ADR-0346
  - ADR-0347
  - ADR-0348
  - ADR-0349
---

# Data Protection Impact Assessment: finops-portal
## automation-event-driven-data-flow

### Processing Trigger
- Scope: Wave 15-ZF doctrine propagation only; no new runtime processing is implemented in this DPIA entry.
- Event surface: automation events for this microservice include local verifier outcomes, governance lane rename evidence, sharding automation decisions, and Jenkins/ArgoCD CI/CD transitions.
- Data subjects: tenant administrators, operator principals, service principals, and tenant users whose identifiers may appear in tenant-scoped operational evidence.

### Data Categories
- tenant_id, service_id, cell_id, shard_id, region_id, ResidencyClass, compliance_pack, and cell_placement_class.
- Cedar decision id, audit-chain event id, verifier run id, Jenkins build id, ArgoCD sync id, cosign verification result, and governance lane id.
- No tenant payload, PHI, payment card data, message body, document body, or secret material is introduced by this doctrine propagation entry.

### ADR-Bound Controls
- ADR-0346: `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix; it MUST block on exit-0 of EACH step before returning success. Enforced by `governance-verify-ci-mirror-coverage`, `governance-verify-ci-step-exit-semantics`, `governance-verify-skip-flag-allowlist`, `governance-submit-calls-verify`, and `governance-verify-exit-code-contract`.
- ADR-0347: every `governance-*` CI lane prefix in the Oyatie corpus RENAMES to `governance-*` in a single bulk-rename pull request (Wave 15-ZB). Enforced by `governance-no-foundry-fitness-residue`, `governance-lane-prefix-vocabulary`, and `governance-rename-inventory-presence`.
- ADR-0348: AUTOSHARDING means tenant→cell/shard placement is computed by the control plane automatically; AUTO-REBALANCE automatically migrates tenants from hot cells to cooler cells; DYNAMIC SHARDING adjusts shard count within a cell based on load. Enforced by `governance-sharding-automation-coverage`, `governance-autosharding-manual-mode-refusal`, `governance-auto-rebalance-residency-honored`, `governance-dynamic-sharding-threshold-coverage`, `governance-audit-chain-emit-on-automation-events`, and `governance-tenant-migration-reversibility`.
- ADR-0349: Jenkins (LTS) and ArgoCD are the two canonical self-hostable CI/CD substrates; Jenkins augments rather than replaces GitHub Actions, and ArgoCD REPLACES manual `kubectl apply` and Helm CLI deploys. Enforced by `governance-jenkins-github-actions-parity`, `governance-argocd-application-cosign-verified`, `governance-argocd-tenant-namespace-isolation`, `governance-jenkins-jcasc-only`, and `governance-deploy-audit-chain-emit`.

### DPIA Assessment
- Necessity: automation evidence is necessary to prove verifier completeness, governance lane vocabulary, tenant placement, rebalance reversibility, dynamic shard threshold decisions, and CI/CD deployment accountability.
- Proportionality: records are limited to identifiers, decision ids, evidence pointers, and signed event metadata; payload data remains in the source microservice and is not copied into automation evidence.
- Residency and compliance: auto-rebalance honors residency and compliance pack constraints; cross-jurisdiction movement requires explicit Cedar permit and audit-chain emission.
- Access control: service principals, Jenkins principals, ArgoCD principals, and cell-orchestrator principals are least-privilege actors governed by Cedar and tenant namespace isolation.
- Retention and rights: tenant-scoped automation history can be exported for access and portability requests; immutable audit-chain rows are retained under the applicable legal basis and referenced rather than deleted.

### Residual Risk
- Residual risk is medium until Wave 15-ZA/ZB/ZD/ZE implementation PRs land the verifier mirror, governance rename, sharding automation bodies, and Jenkins/ArgoCD substrate files.
- Residual risk becomes low when the ADR-0346, ADR-0347, ADR-0348, and ADR-0349 enforcement lanes promote from advisory/report-only to BLOCKER and pass for this microservice.
