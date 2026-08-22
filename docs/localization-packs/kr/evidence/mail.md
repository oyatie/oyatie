---
doc_class: LocalizationEvidence
pack_code: kr
fd001_surface: mail
source_microservice: mail
status: planning-closed-foundational
activation_claim: not-active
---

# KR FD-001 Evidence — Mail KR communication retention and document handoff hooks

## Evidence
- docs/localization-packs/kr.md
- docs/localization-packs/kr/pack.yaml
- docs/localization-packs/kr/corpus.lock
- microservices/mail/manifest.json
- microservices/mail/PRD.md
- Mail manifest and PRD define the mail boundary and delivery responsibilities.
- KR pack sources own retention and document-template overlays for localized mail workflows.

## KR Pack Responsibilities
- Anchors KR mail retention, document handoff, and template pack routing without claiming live archive certification.
- kr_pack_surface: messenger_mail_community_localization
- kr_pack_surface: typst_document_templates
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
