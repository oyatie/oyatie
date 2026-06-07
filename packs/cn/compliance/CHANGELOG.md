# CN-PIPL-2021 Pack Changelog

All notable changes to this compliance pack are documented in this file.
Versioning follows semantic versioning (semver 2.0): major bumps when the control surface
changes (new Cedar rules, new required fields); minor when overlays extend; patch for
editorial-only updates.

## [1.0.0] — 2026-05-20

### Added

- Initial publication of CN-PIPL-2021 compliance pack.
- `manifest.json` — full pack declaration conforming to `/specs/compliance-pack-schema.json`;
  pack_id `CN-PIPL-2021`, version `1.0.0`, binding ADR-0251.
- `cedar/data-localization-enforcement.cedar` — enforces `cn-pipl-eligible` cell requirement
  per PIPL Article 40; blocks cell::activate and cell::migrate to non-CN cells.
- `cedar/consent-gating.cedar` — per-purpose consent validation per PIPL Articles 13–15;
  12-month renewal enforcement; withdrawal honored immediately.
- `cedar/cross-border-transfer-gating.cedar` — blocks cross-border data transfer unless
  one of three PIPL Article 38 pathways is attested (CAC security assessment, PI protection
  certification, or standard contractual clauses filed).
- `cedar/minor-protection.cedar` — under-14 guardian consent requirement per PIPL Article 31
  and CAC Regulations on the Protection of Minors in Cyberspace (effective 2024-01-01);
  prohibits profiling, automated decisions, and cross-border transfer for under-14 data.
- `cedar/sensitive-pi-handling.cedar` — sensitive PI processing gates per PIPL Article 28:
  DPIA required, category-specific separate consent, encryption confirmation.
- `cedar/dsar-response.cedar` — DSAR enforcement per PIPL Articles 44–47; 15-business-day
  SLA; permits access/copy/portability/erasure for authenticated subjects.
- `breach-notification-workflow.yaml` — 72h breach notification workflow per PIPL Article 57;
  six-stage pipeline (detect → triage → emergency-response → CAC-notification → subject-
  notification → remediation → post-mortem); CAC notification endpoint wired.
- `dpia-template.md` — DPIA template per PIPL Article 51; sections A–G covering processing
  activity description, necessity/proportionality, risk identification, technical/organisational
  measures, cross-border transfer assessment, and sign-off.
- `regulator-references.yaml` — authoritative contact channels for CAC (Cyberspace
  Administration of China) and MIIT (Ministry of Industry and Information Technology);
  CCRC certification body details; hyperscaler reference implementations for AWS China,
  Alibaba Cloud, Tencent Cloud, and Microsoft Azure China (21Vianet).
- `README.md` — intern-buildable pack overview meeting documentation-rigor.md bar; sections
  A–G covering overview/scope, architecture, consent architecture, cross-border transfer,
  breach notification, data subject rights, and references.

### Context

Created to close F13 P1 finding from `evidence/debate/keystone-bundle-2026-05-20-F13-
compliance-r1.json`: China PIPL jurisdiction scope was ambiguous in the keystone bundle.
This pack makes the scope decision explicit (tenant-opt-in; oyatie own data out of scope)
and provides the full control surface for tenants operating PRC-resident data planes.

Also closes the F13 P1 finding in ADR-0251 for the `cn-pipl-eligible` cell certification
level: the certification level is now enumerated in the ADR-0251 cell-certification-levels
table with full requirements (mainland-CN data-plane, CAC security assessment, CAC-approved
KMS, mainland-CN-resident operations staff).

### Regulatory baseline

- PIPL effective 2021-11-01 (all 74 articles, 8 chapters)
- CAC cross-border transfer measures effective 2022-09-01
- CAC standard contract measures effective 2023-06-01
- CAC Minors Cyberspace Regulations effective 2024-01-01
- Cybersecurity Law effective 2017-06-01 (upstream)
- Data Security Law effective 2021-09-01 (upstream)

### Known limitations and future work

- Pathway 2 (PI protection certification via CCRC) flow is documented but the automated
  certification-renewal workflow (`wf-pipl-certification-renewal`) is deferred to a follow-up
  slice; manual renewal reminder is emitted at T-60 days before expiry.
- MIIT App Personal Information Collection Rules compliance (separate from PIPL) requires a
  dedicated `cn-miit-app-rules-2021` pack overlay; scoped to a future slice.
- Quebec Law 25 parallel (separate per-purpose consent, age-14 threshold) noted in the F13
  verdict as a P2 finding; `packs/qc-law-25/` deferred to Slice 5.
- PIPL Article 52 designated responsible person (DPO-equivalent) workflow not yet automated;
  the tenant onboarding checklist requires manual attestation of responsible-person designation.
- `wf-pipl-evidence-annual` regulator evidence workflow (annual records-of-processing report)
  referenced in manifest but implementation deferred until the `microservices/governance/`
  regulator-evidence-emission workflow is promoted to BLOCKER.
