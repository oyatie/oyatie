---
doc_class: LocalizationEvidence
pack_code: kr
fd001_surface: cloud-iac
source_microservice: cloud-iac
status: planning-closed-foundational
activation_claim: not-active
---

# KR FD-001 Evidence — Cloud IaC KR residency and portable deployment hooks

## Evidence
- docs/localization-packs/kr.md
- docs/localization-packs/kr/pack.yaml
- docs/localization-packs/kr/corpus.lock
- microservices/cloud-iac/manifest.json
- microservices/cloud-iac/PRD.md
- Cloud IaC manifest and PRD define portable deployment ownership.
- KR pack manifest and corpus lock define source and activation status for pack-specific deployment evidence.

## KR Pack Responsibilities
- Anchors pack-specific deployment, residency, and runbook inputs to IaC without hardcoding jurisdiction into canonical modules.
- kr_pack_surface: pack_manifest
- kr_pack_surface: data_residency_and_privacy_controls
- kr_pack_surface: operational_runbooks_and_slos

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
