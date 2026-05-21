---
doc_class: Reference
status: Accepted
date: 2026-05-20
---

# Mail µservice — Changelog

## Unreleased
- Added `ARCHITECTURE.md`, `README.md`, `CHANGELOG.md` per documentation-rigor §2.
- Added `policy/abuse-defence.cedar` (ADR-0297), `policy/anti-phishing.cedar`, `policy/phi-dlp.cedar`, `policy/minor-protection.cedar`.
- Added runbooks: `dmarc-rollout-monitoring.md`, `phi-leak-recovery.md`, `account-compromise-recovery.md`, `mail-bot-score-recalibration.md`.
- Added `IP-016-jmap-rfc-8620-frontend.md`, `IP-017-anti-phishing-edge-wiring.md`, `IP-018-hipaa-overlay-rollout.md`.
- Added IaC: `iac/edge-waf.yaml`, `iac/ech-config.yaml`, `iac/pqc-cert.yaml`, `iac/openbao-policy.yaml`, `iac/secret-bindings.yaml`.
- Added catalog: `oya-mail-jmap-frontend-rest.yaml`, `oya-mail-anti-phishing-kernel.yaml`, `oya-mail-phi-dlp-adapter-kernel.yaml`.
- Added dashboards: `dmarc-deliverability.json`, `abuse-defence-outcomes.json`.
- Added SLO `mail-jmap-mailbox-fetch-latency.openslo.yaml`.

## 0.1.0 — 2026-05-18
- Initial scaffolding per PR-143 with 94 artifacts.
- ADR-MAIL-0001..0004 published.
