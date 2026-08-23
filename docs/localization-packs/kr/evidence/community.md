---
doc_class: LocalizationEvidence
pack_code: kr
fd001_surface: community
source_microservice: community
status: planning-closed-foundational
activation_claim: not-active
---

# KR FD-001 Evidence — Community KR localization and public trust handoffs

## Evidence
- docs/localization-packs/kr.md
- docs/localization-packs/kr/pack.yaml
- docs/localization-packs/kr/corpus.lock
- microservices/community/manifest.json
- microservices/community/PRD.md
- Community manifest and PRD define the social/community product boundary.
- KR pack overview lists community in the connect cluster and keeps substrate concerns pack-neutral.

## KR Pack Responsibilities
- Keeps community localization, privacy notices, and trust workflow handoffs pack-owned while community stays pack-neutral.
- kr_pack_surface: messenger_mail_community_localization
- kr_pack_surface: data_residency_and_privacy_controls
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
