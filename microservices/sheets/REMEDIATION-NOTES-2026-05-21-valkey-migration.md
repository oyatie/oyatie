## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/sheets/AUDIT-FINDINGS-2026-05-18.json
- microservices/sheets/IP-001-iac-bootstrap.md
- microservices/sheets/IP-005-collab-crdt-loro-aligned-ws-0001.md
- microservices/sheets/IP-013-cell-grid-rest-leptos-wasm-app-license-gate.md
- microservices/sheets/PHASE-01-SHEETS-FOUNDATION.md
- microservices/sheets/PRD.md
- microservices/sheets/capacity-model.md
- microservices/sheets/catalog/oya-sheets-collab-crdt-adapter-valkey.yaml
- microservices/sheets/catalog/oya-sheets-collab-crdt-worker.yaml
- microservices/sheets/catalog/oya-sheets-recalc-engine-worker.yaml
- microservices/sheets/coherence-audit-2026-05-20.md
- microservices/sheets/dashboards/collab-and-fanout.json
- microservices/sheets/decisions/ADR-SHEETS-0001-crdt-library-selection.md
- microservices/sheets/iac/helm/visual-grid-rest/templates/deployment.yaml
- microservices/sheets/iac/helm/visual-grid-rest/values.yaml
- microservices/sheets/iac/kustomize/base/kustomization.yaml
- microservices/sheets/iac/kustomize/base/openbao-secret-references.yaml
- microservices/sheets/manifest.json
- microservices/sheets/multi-region.md
- microservices/sheets/policy/data-residency.md
- microservices/sheets/runbooks/collab-conflict-resolution-crdt.md
- microservices/sheets/runbooks/recalc-storm-throttle.md

Counterpart-fact preservations:
- None; every Redis hit in this bucket described Oyatie-owned substrate vocabulary.

Files renamed (git mv):
- microservices/sheets/catalog/oya-sheets-collab-crdt-adapter-redis.yaml -> microservices/sheets/catalog/oya-sheets-collab-crdt-adapter-valkey.yaml
