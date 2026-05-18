---
doc_class: Compliance
title: "Compliance posture"
microservice: developer-sdk
status: Accepted
owner_team: axis-ecosystem
date: 2026-05-18
related_adrs: [ADR-0213, ADR-0131]
doc_status: published
---

# Compliance posture


## Frameworks engaged

| Framework | Status | Evidence |
|---|---|---|
| GDPR Article 28 | Required for EU developer onboarding | DPA acceptance captured at signup |
| US BSA | Required for US developer payouts | KYC + 1099-MISC emission |
| EU AML5 | Required for EU developer payouts | KYC + sanctions screening |
| KR FSS / KFTC | Required for KR developer payouts | KYC + KFTC firm-bank protocol |
| HIPAA BAA | Required for plugins touching PHI in pack-us-healthcare | BAA acceptance in dev onboarding overlay |
| EU AI Act | Required for plugins using AI capabilities | Risk class declared in plugin manifest |
| SLSA L3 | Required for all published plugins | Cosign-signed artifact + provenance attestation |
| WCAG 2.2 AA | Required for tenant-facing UI surfaces | axe + pa11y CI lane |
| FATF | Sanctions list daily refresh | OFAC + EU + UN consolidated |
| OFAC SDN | Daily refresh | Sanctions screening in KYC pipeline |

## Pack-specific overlays

See `microservices/<ms>/packs/<pack>/manifest.json` per pack.

## Audit chain integration

Every plugin/developer state transition emits a seal event to audit-chain µservice per ADR-0003. Daily chain-integrity verification required.

