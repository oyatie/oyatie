---
doc_class: Reference
status: Accepted
date: 2026-05-20
---

# Notes µservice — Changelog

## Unreleased
- Added `ARCHITECTURE.md`, `README.md`, `CHANGELOG.md`.
- Added `policy/abuse-defence.cedar` (ADR-0297), `policy/minor-protection.cedar` (ADR-0292), `policy/phi-hipaa-notes.cedar`, `policy/pci-payments-notes.cedar`, `policy/share-link-scope.cedar`.
- Added runbooks: `notes-share-link-revocation.md`, `notes-bot-score-recalibration.md`, `clinical-note-leak-recovery.md`, `crdt-divergence-recovery.md`.
- Added `IP-016-collab-edit-mls-loro-hardening.md`, `IP-017-hipaa-clinical-notes-overlay.md`, `IP-018-abuse-defence-edge-wiring.md`.
- Added IaC: `iac/edge-waf.yaml`, `iac/ech-config.yaml`, `iac/pqc-cert.yaml`, `iac/openbao-policy.yaml`, `iac/secret-bindings.yaml`.
- Added catalog: `oya-notes-share-link-adapter-postgres.yaml`, `oya-notes-phi-classifier-kernel.yaml`, `oya-notes-mls-key-escrow-adapter-openbao.yaml`.
- Added dashboards: `e2e-encryption-health.json`, `abuse-defence-outcomes.json`.
- Added SLO `notes-collab-edit-merge-latency.openslo.yaml`.

## 0.1.0 — 2026-05-18
- Initial scaffolding per PR-143 with 99 artifacts.
- ADR-NOTES-0001..0006 published.
