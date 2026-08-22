---
doc_class: LocalizationEvidence
pack_code: kr
fd001_surface: ops-dashboard-control-center
source_microservice: ops-dashboard-control-center
status: planning-closed-foundational
activation_claim: not-active
---

# KR FD-001 Evidence — Ops Dashboard and Control Center KR escalation evidence

## Evidence
- docs/localization-packs/kr.md
- docs/localization-packs/kr/pack.yaml
- docs/localization-packs/kr/corpus.lock
- microservices/ops-dashboard-control-center/manifest.json
- microservices/ops-dashboard-control-center/PRD.md
- Ops Dashboard manifest and PRD define the control-center surface.
- KR pack sources define pack status, evidence directory, and activation blockers.

## KR Pack Responsibilities
- Names KR escalation, evidence-pack, incident, audit, policy, and recovery runbook responsibilities for the Ops Control Center without claiming operational maturity.
- kr_pack_surface: ops_control_center_localization_runbooks_and_escalation_flows
- kr_pack_surface: operational_runbooks_and_slos
- kr_pack_surface: audit_chain_evidence
- kr_pack_surface: data_residency_and_privacy_controls

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
