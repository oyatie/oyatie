---
doc_class: LocalizationEvidence
pack_code: kr
fd001_surface: ontology
source_microservice: ontology
status: planning-closed-foundational
activation_claim: not-active
---

# KR FD-001 Evidence — Ontology KR semantic adapter boundaries

## Evidence
- docs/localization-packs/kr.md
- docs/localization-packs/kr/pack.yaml
- docs/localization-packs/kr/corpus.lock
- microservices/ontology/manifest.json
- microservices/ontology/PRD.md
- Ontology manifest and PRD define semantic boundary ownership.
- KR pack manifest and corpus lock define regulatory source families and claim controls.

## KR Pack Responsibilities
- Anchors KR regulatory terms, policy fragments, audit semantics, and migration mappings as pack adapters while the canonical ontology stays neutral.
- kr_pack_surface: regulatory_bindings
- kr_pack_surface: cedar_policy_fragments
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
