---
doc_class: LocalizationEvidence
pack_code: kr
fd001_surface: foundry
source_microservice: foundry
status: planning-closed-foundational
activation_claim: not-active
---

# KR FD-001 Evidence — Foundry KR evidence automation and audit-pack hooks

## Evidence
- docs/localization-packs/kr.md
- docs/localization-packs/kr/pack.yaml
- docs/localization-packs/kr/corpus.lock
- microservices/intelligence/manifest.json
- microservices/intelligence/PRD.md
- Foundry manifest and PRD define the automation and evidence surface.
- KR corpus lock defines source-family ownership and claim controls.

## KR Pack Responsibilities
- Connects Foundry evidence generation, audit pack assembly, and import/export migration controls to KR pack-owned source locks.
- kr_pack_surface: pack_manifest
- kr_pack_surface: regulatory_bindings
- kr_pack_surface: audit_chain_evidence
- kr_pack_surface: import_export_migration_paths

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
