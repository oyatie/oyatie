---
doc_class: Reference
status: Accepted
date: 2026-05-20
---

# Social µservice — Changelog

## Unreleased
- Added `ARCHITECTURE.md`, `README.md`, `CHANGELOG.md`.
- Added `policy/abuse-defence.cedar` (ADR-0297 — extensive), `policy/minor-protection.cedar` (ADR-0292), `policy/content-policy.cedar`, `policy/profile-verification.cedar`, `policy/federation-egress.cedar`, `policy/dm-scope.cedar`.
- Added runbooks: `csam-detect-and-ncmec-report.md`, `social-bot-score-recalibration.md`, `sock-puppet-cluster-takedown.md`, `coordinated-inauthentic-behavior-response.md`, `dsa-transparency-report-generation.md`.
- Added `IP-016-minor-protection-strict-defaults.md`, `IP-017-abuse-defence-edge-and-cedar.md`, `IP-018-dsa-compliance-overlay.md`.
- Added IaC: `iac/edge-waf.yaml`, `iac/ech-config.yaml`, `iac/pqc-cert.yaml`, `iac/openbao-policy.yaml`, `iac/secret-bindings.yaml`.
- Added catalog: `oya-community-social-csam-classifier-adapter-photodna.yaml`, `oya-community-social-sock-puppet-detector-kernel.yaml`, `oya-community-social-profile-verification-adapter-idv.yaml`, `oya-community-social-dsa-transparency-worker.yaml`.
- Added dashboards: `abuse-defence-outcomes.json`, `minor-protection-health.json`, `csam-and-trust-safety.json`.
- Added SLOs: `csam-classifier-latency.openslo.yaml`, `minor-protection-engagement-correctness.openslo.yaml`.

## 0.1.0 — 2026-05-18
- Initial scaffolding per PR-143 with 99 artifacts.
- ADR-SOC-0001..0006 published.
