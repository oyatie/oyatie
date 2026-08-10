---
doc_class: DPIA
doc_id: DPIA-WORKPLACE_INTEGRATION
microservice: workplace-integration
status: wave-15-zf-doctrine-propagation
date: 2026-05-21
owner_team: axis-compliance + axis-governance
bounded_context: automation-event-driven-data-flow
implementation_phase: doctrine-propagation-only
rust_code_status: not-authored-in-this-wave
source_adrs:
  - ADR-0346
  - ADR-0347
  - ADR-0348
  - ADR-0349
---

# Data Protection Impact Assessment: workplace-integration
## automation-event-driven-data-flow

### Processing Trigger
- Scope: Wave 15-ZF doctrine propagation only; no new runtime processing is implemented in this DPIA entry.
- Event surface: automation events for this microservice include branch-protected cloud CI outcomes, central governance evidence, sharding automation decisions, and GitOps delivery transitions.
- Data subjects: tenant administrators, operator principals, service principals, and tenant users whose identifiers may appear in tenant-scoped operational evidence.

### Data Categories
- tenant_id, service_id, cell_id, shard_id, region_id, ResidencyClass, compliance_pack, and cell_placement_class.
- Cedar decision id, audit-chain event id, oya-ci-required run id, GitOps sync id, cosign verification result, and central governance evidence id.
- No tenant payload, PHI, payment card data, message body, document body, or secret material is introduced by this doctrine propagation entry.

### ADR-Bound Controls
- ADR-0346: historical verifier wording is subordinate to current SSOT. D-CICD-AUTHORITY binds this lane to the branch-protected `oya-ci-required` cloud-ci/oya-ci gate as live merge authority; local command output is transition evidence only.
- ADR-0347: historical lane-vocabulary wording is subordinate to current SSOT. D-GOVERNANCE-CENTRAL: central PaC/CaC/PDP/evidence pipelines own governance authority; do not scatter authority across local CLI lanes.
- ADR-0348: automation controls remain required; evidence flows through central governance and the branch-protected `oya-ci-required` gate.
- ADR-0349: historical self-hostable substrate wording is subordinate to current SSOT. D-CICD-AUTHORITY keeps one canonical CI authority now (`oya-ci-required`) and the owned oya-ci cutover later; self-hostable delivery references are subordinate to the current SSOT and are not parallel merge authorities.

### DPIA Assessment
- Necessity: automation evidence is necessary to prove verifier completeness, governance lane vocabulary, tenant placement, rebalance reversibility, dynamic shard threshold decisions, and CI/CD deployment accountability.
- Proportionality: records are limited to identifiers, decision ids, evidence pointers, and signed event metadata; payload data remains in the source microservice and is not copied into automation evidence.
- Residency and compliance: auto-rebalance honors residency and compliance pack constraints; cross-jurisdiction movement requires explicit Cedar permit and audit-chain emission.
- Access control: service principals, oya-ci-required principals, GitOps principals, and cell-orchestrator principals are least-privilege actors governed by Cedar and tenant namespace isolation.
- Retention and rights: tenant-scoped automation history can be exported for access and portability requests; immutable audit-chain rows are retained under the applicable legal basis and referenced rather than deleted.

### Residual Risk
- Residual risk is medium until Wave 15-ZA/ZB/ZD/ZE implementation PRs land the cloud CI authority, central governance evidence, sharding automation bodies, and GitOps delivery files.
- Residual risk becomes low when the D-CICD-AUTHORITY, D-GOVERNANCE-CENTRAL, ADR-0348 automation controls, and subordinate delivery checks pass for this microservice.
