---
doc_class: ComplianceMatrix
title: notes µservice — Regulatory + Standards Compliance Matrix
microservice: notes
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-legal + council-privacy + axis-notes
review_cadence: annually + on every regulatory change
doc_status: published
---

# Compliance Matrix — notes µservice

## Scope

This matrix maps notes-µservice controls to: GDPR; KR PIPA + 통신비밀보호법 + 정보통신망법; HIPAA; APPI; PDPA (SG, AU); DPDPA 2023; LGPD; UAE PDPL; KSA PDPL + SAMA; EU AI Act; ePrivacy; WCAG 2.2 AA; SLSA L3; NIST SSDF; SOC 2 Type 2; ISO 27001:2022; OWASP ASVS v4; CIS Kubernetes Benchmark v1.9; FIPS 140-3.

## Pack Activation Status

| Pack | Activated at M02 | Conditional gates |
|---|---|---|
| pack-kr | YES | KR PIPA Art. 28 controls + 통신비밀보호법 Art. 13 + 정보통신망법 Art. 28 |
| pack-eu | conditional | first GDPR-scope tenant + SCC on file |
| pack-us | conditional | first US-scope tenant |
| pack-us-healthcare | conditional | signed BAA + HIPAA-eligible source-target |
| pack-jp / sg / au / in / br / ae / ksa | conditional | first tenant + local-DPA review |

## GDPR (EU)

| Article | Control | Artifact |
|---|---|---|
| Art. 5(1)(a) lawfulness | Consent (Personal) + legitimate interest (Professional) | `dpia.md` §2.1 |
| Art. 5(1)(b) purpose limitation | µservice scope statement | `PRD.md` §Purpose |
| Art. 5(1)(c) data minimisation | Personal-tier events opaque; Ontology writes minimal | `PRD.md` §Workflow events |
| Art. 5(1)(d) accuracy | Inline edit; admin edit forbidden Personal | `policy/dual-context-isolation.md` |
| Art. 5(1)(e) storage limitation | per-pack retention bounds | `policy/data-residency.md` |
| Art. 5(1)(f) integrity + confidentiality | E2E (Personal), tenant-DEK (Professional), TLS + mTLS, MAC-tagged ciphertext | `policy/e2e-personal-tier-default.md`; `threat-model.md` |
| Art. 6 lawful basis | per-tier basis | `dpia.md` §2.1 |
| Art. 9 special category | E2E-protected; server-side processing impossible for Personal | ADR-NOTES-0001 |
| Art. 17 erasure | DSR cascade runner | `policy/data-residency.md` §DSR cascade |
| Art. 22 automated decision | T2 auto-organize disabled at MVP; opt-in per user | `capabilities/T2-auto.yaml` |
| Art. 25 PbD + PbDef | Per-microservice flat layout + LEAN lanes + Cedar default-deny | ADR-0131; ADR-0064 |
| Art. 28 processor | DPA + sub-processor list (foundry-runtime + drive + tasks) | `legal/sub-processors.md` (linked) |
| Art. 30 records | Workflow event ledger + audit-chain seals | `threat-model.md` §Audit |
| Art. 32 security | controls per `policy/e2e-personal-tier-default.md`; FIPS 140-3 where required | (this matrix) |
| Art. 33 + 34 breach notification | Sev-1 incident-response runbook | `incident-response.md` |
| Art. 35 DPIA | `dpia.md` (this directory) | `dpia.md` |
| Art. 44–50 transfers | pack pinning + SCC | `policy/data-residency.md` |
| Recital 26 anonymisation | content not anonymised; pseudonymisation via tenant_id + user_id | `dpia.md` §3 |

## EU AI Act

| Article | Control |
|---|---|
| Art. 50 transparency | AI-assist results labelled as AI-generated; pack-eu overlay enforces `evidence_topic: oya.notes.capability.t1_assist.evidence` |
| Limited-risk classification | T1 summarize / tag-suggest / link-suggest classified `limited_risk`; T2 auto-organize classified `limited_risk` with conformity-assessment commitments |
| Art. 27 conformity assessment (high-risk if applicable) | not currently in scope (notes is not safety-critical or HR-decision domain) |

## KR PIPA

| Article | Control |
|---|---|
| Art. 15 collection | consent at signup for Personal; tenant-of-tenant consent for Professional |
| Art. 17 third-party provision | tenant-controlled; sub-processor list under DPA |
| Art. 22-2 personal-information-protection-officer | per tenant + Council of Privacy |
| Art. 23 sensitive info | E2E protection on Personal-tier covers most cases; explicit consent required where not |
| Art. 28 security measures | Art. 29-aligned controls + audit-chain |
| Art. 29 cryptographic + identity controls | MLS RFC 9420 + Cedar + audit-chain; pack-kr overlay enforces |

## KR 통신비밀보호법 (Telecommunications Secrecy Act)

| Article | Control |
|---|---|
| Art. 13 | confidentiality of communications preserved by E2E (Personal) + tenant-DEK (Professional) |

## KR 정보통신망법 (Information & Communications Network Act)

| Article | Control |
|---|---|
| Art. 28 | technical + administrative controls per ADR-0028 audit-chain + ADR-NOTES-0001 E2E |

## HIPAA (pack-us-healthcare only)

| Section | Control |
|---|---|
| 45 CFR §164.308 administrative | risk analysis (DPIA) + workforce training |
| 45 CFR §164.310 physical | infra controls inherited from OCI HIPAA-eligible regions |
| 45 CFR §164.312(a)(2)(iv) encryption | tenant-DEK envelope + MLS (where Personal-tier in HIPAA scope, which is rare) |
| 45 CFR §164.312(b) audit | audit-chain Ed25519 seal per state transition |
| 45 CFR §164.502(b) minimum necessary | Cedar member-check + per-channel scope |
| 45 CFR §164.530(j) retention | 6-year floor for PHI-class notes |

## APPI (JP)

| Article | Control |
|---|---|
| Art. 17 + 18 collection/use limit | per `dpia.md` §2 |
| Art. 21 retention | 2-year floor (labor) per `policy/data-residency.md` |
| Art. 27 cross-border | pack-jp pinning |

## PDPA (SG) / PDPA (AU) / DPDPA 2023 (IN) / LGPD (BR) / UAE PDPL / KSA PDPL + SAMA

Pack overlays in `policy/data-residency.md` enforce per-pack retention + transfer rules. Each overlay carries:

- jurisdiction-specific consent text;
- retention floor;
- DPO contact;
- cross-border transfer permission gate.

## ePrivacy Directive 2002/58/EC

| Article | Control |
|---|---|
| Art. 5(3) cookies/storage | Web-clipper extension manifest declares minimum-permission storage; Workflow Studio shell uses essential storage only |
| Art. 5(1) confidentiality | E2E (Personal) + tenant-DEK (Professional) |

## WCAG 2.2 AA

| SC | Control |
|---|---|
| 1.3.1 info-and-relationships | semantic HTML in editor + clipper |
| 1.4.3 contrast | per WCAG AA design tokens |
| 2.1.1 keyboard | full keyboard navigation; no mouse-required affordance |
| 2.4.6 headings | Markdown heading hierarchy preserved |
| 3.1.1 language | per-pack language tag on rendered HTML |
| 4.1.3 status messages | ARIA live region for sync state + share-link emission |

## SLSA L3

- Reproducible Cargo builds (`cargo --frozen --locked`).
- HSM-signed artifacts at release.
- Provenance recorded in `evidence/release-pointer-*.json`.
- Dependency pinning verified by `oya gate validate version-pinning-conformance`.

## NIST SSDF (SP 800-218)

| Practice | Control |
|---|---|
| PO.1 prepare | Security requirements in PRD §NFR Security |
| PS.1 protect | Cedar v4 default-deny + LEAN lanes |
| PW.4 produce | Signed builds + reproducible Cargo |
| RV.1 respond | incident-response runbook |

## SOC 2 Type 2

| Trust Service Criterion | Control |
|---|---|
| CC6.1 logical access | OIDC + Cedar + per-tenant scope |
| CC6.7 transmission | mTLS + TLS 1.3 |
| CC7.2 monitoring | observability µservice + Prometheus rules |
| A1.2 availability | OpenSLO + burn-rate gates |

## ISO 27001:2022

| Annex A clause | Control |
|---|---|
| A.5.15 access control | Cedar + tenant-scope |
| A.8.3 cryptographic | MLS + tenant-DEK + FIPS 140-3 modules |
| A.5.23 cloud service customer | per-pack residency contract |
| A.8.24 use of cryptography | ADR-NOTES-0001 |

## OWASP ASVS v4

| Level | Section | Control |
|---|---|---|
| L2 | §2.4 password storage | PBKDF2-SHA256 ≥ 600k iter for share-link passphrase |
| L2 | §4 authorization | Cedar v4.2 default-deny |
| L2 | §6 cryptography | MLS + tenant-DEK + FIPS 140-3 |
| L2 | §8 data protection | E2E-default Personal-tier |
| L2 | §11 business logic | dual-context-isolation invariants |

## CIS Kubernetes Benchmark v1.9

Pod security per Helm templates:
- `runAsNonRoot: true`
- `readOnlyRootFilesystem: true`
- `allowPrivilegeEscalation: false`
- `capabilities.drop: [ALL]`
- NetworkPolicy default-deny + explicit egress

## FIPS 140-3

| Module | Use |
|---|---|
| openmls 0.6 (when built with `fips` feature) | MLS encryption for Personal-tier |
| Cargo-RustCrypto (FIPS-mode) | random for share-link tokens |
| OCI FIPS-validated KMS | tenant-DEK wrapping |

## Verification Cadence

- `oya gate validate per-microservice-layout` per PR.
- `oya gate validate version-pinning-conformance` per PR.
- `oya gate validate e2e-ai-refusal` per PR.
- Quarterly compliance review per active pack.
- Annual external pen-test + SOC 2 Type 2 audit cycle + ISO 27001 surveillance audit.

## References

- See header `references:` in `dpia.md`.
- ADR-NOTES-0001..0006.
- All policy + runbook artifacts in this directory.
