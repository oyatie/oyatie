---
doc_class: LocalizationEvidence
pack_code: kr
fd001_surface: workflow-studio
source_microservice: workflow-studio
status: planning-closed-foundational
activation_claim: not-active
---

# KR FD-001 Evidence — Workflow Studio KR template authoring boundaries

## Evidence
- docs/localization-packs/kr.md
- docs/localization-packs/kr/pack.yaml
- docs/localization-packs/kr/corpus.lock
- microservices/workflow-studio/manifest.json
- microservices/workflow-studio/PRD.md
- Workflow Studio manifest and PRD define authoring and template UX boundaries.
- KR pack overview lists Workflow Studio templates and document template families.

## KR Pack Responsibilities
- Connects KR workflow and document-template authoring to pack-owned templates without embedding localized content into canonical Studio logic.
- kr_pack_surface: workflow_templates
- kr_pack_surface: typst_document_templates
- kr_pack_surface: tenant_rbac_operating_flows

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
