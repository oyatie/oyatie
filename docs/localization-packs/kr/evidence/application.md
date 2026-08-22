---
doc_class: LocalizationEvidence
pack_code: kr
fd001_surface: application
source_microservice: application
status: planning-closed-foundational
activation_claim: not-active
---

# KR FD-001 Evidence — Application shell and Enterprise/SMB KR entry flows

## Evidence
- docs/localization-packs/kr.md
- docs/localization-packs/kr/pack.yaml
- docs/localization-packs/kr/corpus.lock
- microservices/application/manifest.json
- microservices/application/PRD.md
- Application manifest and PRD define the product entry shell and tenant workflow handoffs.
- KR pack overview, manifest, and corpus lock define status, source lock, and activation controls.

## KR Pack Responsibilities
- Binds tenant entry, pack selection, and Enterprise/SMB workflow launch to the governed KR pack contract without embedding KR rules in canonical application code.
- kr_pack_surface: pack_manifest
- kr_pack_surface: regulatory_bindings
- kr_pack_surface: tenant_rbac_operating_flows
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
