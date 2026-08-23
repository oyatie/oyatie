---
doc_class: LocalizationEvidence
pack_code: kr
fd001_surface: messenger
source_microservice: messenger
status: planning-closed-foundational
activation_claim: not-active
---

# KR FD-001 Evidence — Messenger KR communication retention and localization hooks

## Evidence
- docs/localization-packs/kr.md
- docs/localization-packs/kr/pack.yaml
- docs/localization-packs/kr/corpus.lock
- microservices/messenger/manifest.json
- microservices/messenger/PRD.md
- Messenger manifest and PRD define the communication boundary.
- KR pack bindings include connect-retention and privacy controls consumed by the communication surfaces.

## KR Pack Responsibilities
- Routes KR communication behavior through pack-owned retention, legal hold, language, and audit hooks while messenger remains canonical-base neutral.
- kr_pack_surface: messenger_mail_community_localization
- kr_pack_surface: regulatory_bindings
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
