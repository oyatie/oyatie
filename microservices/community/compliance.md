---
doc_class: Compliance
template_id: TPL-COMPLIANCE
microservice: community
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-security + axis-community
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0131]
related_artifacts:
  - microservices/community/threat-model.md
  - microservices/community/dpia.md
  - microservices/community/policy/community-isolation.md
  - microservices/community/policy/data-residency.md
doc_status: published
---

# Compliance: community µservice

## Section 230 + Similar Safe-Harbor Posture

oyatie operates the community µservice as an **interactive computer service provider** under 47 USC §230(c)(1) (US) and equivalent intermediary safe-harbor regimes per pack:

| Pack | Safe-harbor regime | Posture |
|---|---|---|
| pack-us | 47 USC §230(c)(1)+(c)(2) (CDA) | Provider; tenant is publisher; good-faith moderation under (c)(2)(A) |
| pack-eu | DSA 2022/2065 Arts. 4-8 (mere conduit / caching / hosting) | Hosting provider; notice-and-action under Art. 16; transparency reports |
| pack-kr | Telecommunications Business Act Art. 22-5 + Information Communications Network Act Art. 44 | Information-communications-service provider; notice-and-takedown 24 h |
| pack-jp | Provider Liability Limitation Act (2001) Arts. 3 + 4 | Provider; sender disclosure on court order only |
| pack-au | Online Safety Act 2021 + Broadcasting Services Act Schedule 5 | Industry code obligations; eSafety Commissioner notice response 24 h |
| pack-in | IT Rules 2021 (Intermediary Guidelines) Rules 3 + 4 | Significant social media intermediary thresholds; due diligence + grievance officer |
| pack-br | Marco Civil da Internet 2014 Arts. 18 + 19 | Court order required for content removal; user notice |
| pack-sg | Online Safety (Miscellaneous Amendments) Act 2022 + POFMA 2019 | Content provider; takedown directions response 24 h |

## HIPAA (pack-us-healthcare)

When PHI may be processed (tenant in healthcare vertical, BAA signed):

| Safeguard | Implementation |
|---|---|
| 45 CFR §164.308 Administrative | Role-based access (Cedar); workforce training; per-tenant BAA |
| 45 CFR §164.310 Physical | Cloud provider physical security (SOC 2); per-region segregation |
| 45 CFR §164.312 Technical | Encryption at rest (KMS); encryption in transit (mTLS); audit-chain |
| 45 CFR §164.314 Organizational | BAA with each sub-processor (S3, hosting) |
| 45 CFR §164.316 Policies + Procedures | This document; threat-model.md; dpia.md |

PHI in community: tenant opt-in only. Default warning surfaced: "do not post PHI". When opted in, Cedar entitlement `phi_eligible == true` required for post create; classifier alerts on suspected PHI in non-eligible spaces.

## GDPR

| Article | Implementation |
|---|---|
| Art. 5 (principles) | Lawful, fair, transparent; data minimisation; retention matrix |
| Art. 6 (lawfulness) | (1)(b) contract + (1)(f) legitimate interest (abuse prevention) |
| Art. 9 (special category) | Explicit consent for PHI under pack-us-healthcare |
| Art. 13/14 (notice) | Tenant-onboarding privacy notice; member onboarding consent |
| Art. 15 (access) | Tenant export; member self-service |
| Art. 17 (erasure) | DSR cascade runbook |
| Art. 22 (ADM) | Moderation is reversible; appeal; human-in-loop two-eyes for bans |
| Art. 25 (by design + default) | Per-tenant isolation; deny-by-default Cedar |
| Art. 28 (processor) | DPA with each tenant |
| Art. 30 (records of processing) | Audit-chain seals every event |
| Art. 32 (security) | mTLS + RLS + Cedar + audit-chain + DSR cascade |
| Art. 33/34 (breach) | 72 h authority notification; data subject notification when high risk |
| Art. 35 (DPIA) | dpia.md |

## KR PIPA (pack-kr)

| Article | Implementation |
|---|---|
| Art. 3 (principles) | Lawful processing; purpose limitation; data minimisation |
| Art. 15 (collection + use) | Tenant-onboarding consent; purpose declared |
| Art. 17 (provision to third parties) | Sub-processor list in Annex B of dpia.md |
| Art. 18 (purpose-limited use) | Cedar action scope |
| Art. 22-2 (children under 14) | Tenant opt-in flow; parental consent gate |
| Art. 23 (sensitive data) | PHI / political opinion / sexual orientation flagged; explicit consent |
| Art. 28 (cross-border transfer) | Default no transfer; opt-in flow per data-residency.md |
| Art. 29 (security) | mTLS + RLS + Cedar + audit-chain |
| Art. 33 (DPIA) | dpia.md; PIPC notification when threshold engaged |
| Art. 34 (breach) | 24 h authority notification |
| Art. 36 (DSR) | DSR cascade runbook; 10 d response |

## APPI (pack-jp)

Arts. 17/18/20/21/23/24 covered: purpose limitation; consent for special-category; cross-border disclosure under Art. 24.

## PDPA (pack-sg)

§§11-26: protection obligation; retention limitation; transfer limitation per data-residency.md.

## Privacy Act 1988 (pack-au)

APPs 1-13: especially APP 6 (use + disclosure), APP 8 (cross-border), APP 11 (security).

## DPDPA 2023 (pack-in)

§§6-10 (consent + notice + processing limits); §16 cross-border restrictions per data-residency.md.

## LGPD (pack-br)

Arts. 6/7/11/14/18/33/46/48: lawful processing; consent; cross-border per Art. 33.

## UAE PDPL / KSA PDPL

Local-only default; cross-border via authority approval.

## SOC 2 Type 2 Mapping

| TSC | Control | Evidence |
|---|---|---|
| CC6.1 (logical access) | tenancy JWT + Cedar | policy/*.cedar; audit-chain logs |
| CC6.2 (provisioning) | tenant onboarding workflow | tenancy µservice handoff |
| CC6.3 (auth) | OIDC + 2FA for admin/mod | tenancy posture |
| CC6.6 (system protection) | mTLS + WAF | iac/ overlays |
| CC7.1 (monitoring) | observability SLOs | slos/*.openslo.yaml |
| CC7.2 (anomaly detection) | foundry-guardrails | bridge integration |
| CC7.4 (incident) | incident-response.md | runbooks/ |
| CC8.1 (change mgmt) | branch-protection + CI gates | governance µservice |

## ISO 27001:2022 Mapping

Annex A controls per threat-model.md `enforced_frameworks`.

## Transparency Report (pack-eu DSA + similar)

Quarterly: per-tenant moderation actions count; appeal outcomes; authority orders received; response times.

## Retention + Erasure Cadence

- Daily TTL job per retention matrix.
- DSR cascade as per `policy/data-residency.md`.
- Legal hold overrides retention; documented per-tenant.

## Audit Evidence Catalog

- `audit-chain` seals = primary evidence stream.
- Per-tenant audit log via `auditor-scope.cedar`.
- Cedar fragment coverage report (CI artifact).
- Penetration test report (annual).
- DPIA review minutes (annual).
- Sub-processor list + DPAs.
