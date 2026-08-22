---
doc_class: LocalizationEvidence
pack_code: kr
fd001_surface: workflow-engine
source_microservice: workflow-engine
status: planning-closed-foundational
activation_claim: not-active
---

# KR FD-001 Evidence — Workflow Engine KR template execution boundaries

## Evidence
- docs/localization-packs/kr.md
- docs/localization-packs/kr/pack.yaml
- docs/localization-packs/kr/corpus.lock
- microservices/workflow-engine/manifest.json
- microservices/workflow-engine/PRD.md
- Workflow Engine manifest and PRD define orchestration ownership.
- KR pack overview names workflow templates as pack-owned overlays.

## KR Pack Responsibilities
- Anchors KR workflow-template execution boundaries and audit events while cross-service orchestration remains in Workflow adapters.
- kr_pack_surface: workflow_templates
- kr_pack_surface: tenant_rbac_operating_flows
- kr_pack_surface: audit_chain_evidence

## Non-Claims
- This KR pack evidence is not active and does not claim production-current legal interpretation.
- This file proves no live tenant readiness and no signed regulatory attestation.
- The canonical base remains jurisdiction-neutral; KR behavior flows through pack adapters and evidence controls.

## Exit Blockers
- Active promotion still requires signed source snapshots, acceptance evidence, tenant dry-run proof, and regression tests.
- Operational maturity remains separate from this planning-closed evidence surface.

## Acceptance Commands
- cargo run -q -p dev-cli -- gate validate korea-localization-evidence
- cargo run -q -p dev-cli -- gate validate planning-closure
- cargo run -q -p dev-cli -- gate validate canonical-base-neutrality
