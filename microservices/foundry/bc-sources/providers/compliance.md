---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: foundry-providers
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-foundry, council-architecture, ops-compliance
related_adrs: [ADR-0025, ADR-0028, ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/foundry-providers/threat-model.md
  - microservices/foundry-providers/dpia.md
  - microservices/foundry-providers/policy/credential-isolation.md
  - microservices/foundry-providers/policy/data-residency.md
  - microservices/foundry-providers/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (foundry-providers µservice)

## Purpose

Canonical control-to-framework mapping for foundry-providers. Tells an external auditor which control implementation satisfies which framework clause, with pointers to evidence artifacts.

## Enforced Frameworks

### SOC 2 Type 2 (2017 Trust Services Criteria)

| TSC | Control objective | Implementation | Evidence |
|---|---|---|---|
| CC6.1 | Logical + physical access | OIDC + Cedar + OpenBao JIT credentials | `policy/provider-router-tenant-scope.cedar` + `policy/openbao-credential.cedar` |
| CC6.2 | Authentication + authorization | OIDC + SPIFFE adapter identity + 2-person rule for adapter publish | `policy/credential-isolation.md` CI-INV-09 |
| CC6.3 | Access provisioning + de-provisioning | OpenBao lifecycle + adapter pod SPIFFE rotation | OpenBao audit log |
| CC6.6 | Logical access control | Cedar default-deny on provider-router + OpenBao read-only tokens | `policy/*.cedar` |
| CC6.7 | Transmission + disposal | mTLS to vendor edges + ZDR negotiation + audit-chain Ed25519 | adapter impl + `policy/data-residency.md` |
| CC6.8 | Vulnerability management | `cargo deny` + Sigstore attestation + adapter digest verify | `ci-cd-and-automation` lanes |
| CC7.1 | System operations | provider-health-monitor + auto-failover + per-pack HA | `failure-modes.md` + `capacity-model.md` |
| CC7.2 | Monitoring system inputs | per-vendor SLI + cost SLI + audit-chain emission | dashboards + `observability` integration |
| CC7.3 | Anomaly evaluation | response-shape anomaly detector + burn-rate alerts | `dashboards/provider-error-rate.json` |
| CC7.4 | Incident response | severity-classified response + on-call rotation | `incident-response.md` |
| CC8.1 | Change management | PR review + LEAN gates + 2-person rule on adapter publish | branch-protection.yaml |

### ISO 27001:2022

| Annex A | Control | Implementation | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | Vendor breach-notification subscription + quarterly threat-model refresh | `threat-model.md` |
| A.5.10 | Acceptable use of information assets | Tenant DPA + per-pack residency contract | `policy/data-residency.md` |
| A.5.14 | Information transfer | mTLS + ZDR + per-pack SCC | `policy/data-residency.md` |
| A.5.15 | Access control | OpenBao + Cedar | `policy/*.cedar` |
| A.5.17 | Authentication information | OpenBao credential isolation + zeroize-on-drop | `policy/credential-isolation.md` |
| A.5.23 | Information security for cloud services | Vendor DPA + sub-processor list | this doc §"Sub-processors" |
| A.5.26 | Response to information security incidents | `runbooks/provider-credentials-revoke.md` + `incident-response.md` | runbooks |
| A.5.31 | Legal, statutory, regulatory | per-pack regulatory mapping table below | `policy/data-residency.md` |
| A.5.32 | Intellectual property rights | Vendor terms compliance per-tenant DPA | tenant onboarding doc |
| A.8.2 | Privileged access rights | 2-person rule + JIT elevation | `policy/openbao-credential.cedar` PERMIT 2 |
| A.8.3 | Information access restriction | Cedar + OpenBao scope | `policy/*.cedar` |
| A.8.5 | Secure authentication | OIDC + mTLS + SPIFFE | adapter impl |
| A.8.11 | Data masking | PHI redaction (where vendors support) + audit-chain hash-only | adapter impl |
| A.8.12 | Data leakage prevention | `oya-check-no-raw-credentials` + `credential-isolation.md` | LEAN lane |
| A.8.16 | Monitoring activities | SLI emission + dashboards + audit-chain | `observability` µservice |
| A.8.20 | Networks security | Istio mTLS + egress proxy + pinned vendor CA | `cell` µservice |
| A.8.21 | Security of network services | Per-vendor pinned cert + envelope signing | adapter impl |
| A.8.23 | Web filtering | Egress proxy allowlist for vendor edges only | `cell` µservice |
| A.8.24 | Cryptography | Ed25519 envelope + BLAKE3 content hash + KMS keyring | adapter impl |
| A.8.25 | Secure development lifecycle | Per-µservice SDL + LEAN gates | `docs/standards/` |
| A.8.26 | Application security requirements | OWASP LLM Top 10 mitigations | `threat-model.md` |
| A.8.27 | Secure system architecture | Per-ADR architecture review | ADRs |

### GDPR (EU; applicable when pack-eu activated)

| Article | Requirement | Implementation | Evidence |
|---|---|---|---|
| Art. 5 | Lawfulness + minimisation | Per-vendor DPA + hash-only audit | adapter impl + DPA |
| Art. 6 | Lawful basis | Tenant contract (Art. 6(1)(b)) | tenant onboarding |
| Art. 25 | Data protection by design | OpenBao isolation + Cedar default-deny | `policy/credential-isolation.md` |
| Art. 28 | Processor obligations | Vendor sub-processor DPA + SCC | sub-processor list (this doc) |
| Art. 30 | Records of processing | `ProviderInvoked` events as Art. 30 records | audit-chain |
| Art. 32 | Security of processing | mTLS + Ed25519 + Cedar + OpenBao | full posture |
| Art. 33 | Breach notification | 72h notification per `runbooks/provider-credentials-revoke.md` | runbook |
| Art. 35 | DPIA | This µservice's DPIA | `dpia.md` |
| Arts. 44–50 | Transfers | SCC 2021/914 + supplementary measures per pack | `policy/data-residency.md` |

### EU AI Act (Reg. (EU) 2024/1689)

| Article | Requirement | Implementation | Evidence |
|---|---|---|---|
| Art. 13 | Transparency + information to deployers | Per-call disclosure record | `contracts/asyncapi/provider-events.yaml` (`EuAiActDisclosure`) |
| Art. 14 | Human oversight | Tool calls not auto-executed; human-in-loop via `cell` | `cell` Cedar policy |
| Art. 27 | FRIA for deployers of high-risk AI | Per-tenant FRIA appended to DPIA when applicable | `dpia.md` |
| Art. 50 | Transparency obligations | `EuAiActDisclosure` event emitted alongside `ProviderInvoked` | event stream |

### HIPAA (pack-us-healthcare)

| §  | Requirement | Implementation | Evidence |
|---|---|---|---|
| §164.308(a)(1) | Risk analysis | This threat-model + DPIA | `threat-model.md` + `dpia.md` |
| §164.308(a)(4) | Information access management | Cedar + OpenBao | policy fragments |
| §164.308(a)(5) | Security awareness + training | onboarding | docs/standards/ |
| §164.310 | Physical safeguards | inherited from `cloud-k8s` | cloud-k8s ToC |
| §164.312(a) | Access control + emergency | OpenBao JIT + revoke runbook | `runbooks/provider-credentials-revoke.md` |
| §164.312(b) | Audit controls | `ProviderInvoked` audit-chain | audit-chain |
| §164.312(c) | Integrity controls | BLAKE3 + Ed25519 envelope | adapter impl |
| §164.312(d) | Person/entity authentication | OIDC + SPIFFE | mesh |
| §164.312(e) | Transmission security | mTLS + ZDR | adapter impl |
| §164.502(e) | BAA with vendors | per-vendor BAA executed pre-PHI | sub-processor list |

### Per-Pack Regulatory Mapping

| Pack | Frameworks | Notes |
|---|---|---|
| pack-kr | KR PIPA + ISMS-P + 전자문서법 + KR-FSS commercial code | per `policy/data-residency.md` |
| pack-eu | GDPR + EU AI Act + NIS2 + eIDAS | per `policy/data-residency.md` |
| pack-us-healthcare | HIPAA + HITECH + state PHI laws | per `policy/data-residency.md` |
| pack-jp | APPI | per `policy/data-residency.md` |
| pack-sg | PDPA + MAS-TRM | per `policy/data-residency.md` |
| pack-au | Privacy Act + APRA-CPS 234 | per `policy/data-residency.md` |
| pack-in | DPDPA 2023 | per `policy/data-residency.md` |
| pack-br | LGPD + BACEN Res. 4.893/2021 | per `policy/data-residency.md` |
| pack-ae | UAE PDPL | per `policy/data-residency.md` |
| pack-ksa | KSA PDPL + SAMA | per `policy/data-residency.md` |

## Sub-Processors (per pack)

| Sub-processor | Role | Packs | DPA / BAA |
|---|---|---|---|
| Anthropic | Provider edge | pack-kr (SCC + ZDR), pack-eu (SCC), pack-us, pack-us-healthcare (BAA + ZDR), pack-jp, pack-sg, pack-au | per-tenant DPA + (PHI) BAA |
| OpenAI | Provider edge | pack-eu (post-SCC), pack-us, pack-jp, pack-sg, pack-au | per-tenant DPA + (PHI) BAA |
| Google (Gemini) | Provider edge | pack-kr, pack-eu, pack-us, pack-jp, pack-sg, pack-au | per-tenant DPA + (PHI) BAA |
| (oyatie in-house) | Self-served | all packs | n/a (internal) |
| Oracle (OCI) | Infrastructure substrate | all packs | OCI DPA + per-region terms |

## Sub-Processor List Refresh

Per ADR-0028 audit-chain posture: any sub-processor change (add, remove, region shift) triggers:
1. Update this matrix.
2. Notify all affected tenants ≥ 30 days in advance per DPA (industry standard for sub-processor change).
3. Re-execute DPA addendum if needed.
4. Emit `SubProcessorChanged` event (cross-µservice) to audit-chain.

## Breach Notification

| Trigger | Notification clock | Audience | Owner |
|---|---|---|---|
| GDPR Art. 33 personal-data breach | 72h from awareness | EU DPAs + affected data subjects | council-privacy + DPO |
| KR PIPA breach | 24h to PIPC + 72h to subjects | PIPC + affected | council-privacy |
| HIPAA breach (PHI) | 60 days notification to affected; HHS notification | HHS + affected | council-privacy + ops-security |
| Vendor-credential compromise (T-01 realisation) | per `runbooks/provider-credentials-revoke.md` | tenant operator + DPO + (if data-class crossed) regulators | ops-security + PrivacyLead |

## Verification

- `cargo run -p oya-dev-cli -- gate validate compliance --microservice foundry-providers` exits 0.
- Annual SOC 2 audit; quarterly continuous-compliance evidence emission.
- Per-pack DPIA on file before first-tenant activation.

## References

- ADR-0028 — audit-chain.
- ADR-0117 — pack residency model.
- SOC 2 Type 2 TSC (2017 + 2022 PoF) — AICPA.
- ISO/IEC 27001:2022.
- GDPR (Reg. (EU) 2016/679).
- EU AI Act (Reg. (EU) 2024/1689).
- HIPAA (45 CFR Part 160 + 164).
- KR PIPA + 전자문서법 + ISMS-P.
- Per-pack additional frameworks listed in `policy/data-residency.md`.
