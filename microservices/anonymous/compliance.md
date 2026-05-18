---
doc_class: Compliance
template_id: TPL-COMPLIANCE
microservice: anonymous
status: Accepted
date: 2026-05-17
owner_team: council-privacy + axis-anonymous + legal
related_adrs: [ADR-0126, ADR-0131, ADR-0133, ADR-ANON-0001, ADR-ANON-0002, ADR-ANON-0003, ADR-ANON-0004, ADR-ANON-0005, ADR-ANON-0006]
review_cadence: quarterly + on regulatory change
doc_status: published
---

# Compliance: anonymous µservice

## Frameworks in scope

| Framework | Status | Auditor cadence |
|---|---|---|
| SOC 2 Type 2 | applicable to all tenants | annual SOC 2 Type 2 with Trust Services Criteria CC1-CC9 |
| ISO 27001:2022 | applicable | annual surveillance + 3-year recert |
| ISO 27018:2019 (PII in cloud) | applicable (especially relevant) | annual |
| NIST SSDF SP 800-218 | applicable to build pipeline | per-release |
| SLSA L3 (provenance) | applicable to release artifacts | per-release |
| OWASP ASVS v4 (level 3 for sensitive backends) | applicable | annual |
| CIS Kubernetes Benchmark v1.8 | applicable to deploy substrate | continuous |
| FIPS 140-3 for cryptographic modules | applicable to blind-signature + BBS+ + MLS libraries | annual library validation review |
| EU AI Act 2024/1689 | applicable (Art. 50 transparency only — limited risk) | EU AI Office-aligned |
| EU DSA 2065/2022 | applicable (Arts. 14/16/17/20/24/27/28) | per-tenant transparency report quarterly |

## Per-framework controls

### SOC 2 Type 2

| Control | Coverage |
|---|---|
| CC6.1 Logical access | Cedar v4.2 default-deny; per-tenant RLS; per-principal entitlements |
| CC6.2 Pre-employment screening | n/a service-side |
| CC6.3 Onboarding/offboarding | tenant-admin onboarding flow; legal-process-approver entitlement issuance + revocation |
| CC6.6 Logical access for privileged users | dual-control for legal-process; audit-chain seal; quarterly access review |
| CC6.7 Restricting access via least privilege | DB GRANT separation between identity reader and post writer |
| CC7.1 Detection of security events | SIEM + Prometheus alerts on `oya_anonymous_personal_tier_*`, `oya_anonymous_legal_process_*` |
| CC7.2 Monitoring + response | `runbooks/anonymity-leak-incident-response.md` Sev-1 path |
| CC7.4 Incident response | full IR plan in `incident-response.md` |
| CC8.1 Change management | ADR-gated; GitOps via merge queue; SLO-gated promotion per ADR-0130 |

### ISO 27001:2022 + ISO 27018:2019

| Annex A control | Implementation |
|---|---|
| A.5.7 Threat intelligence | quarterly threat-model review |
| A.5.10 Acceptable use | per-tenant ToS + UI disclosure |
| A.5.14 Information transfer | TLS 1.3 + per-pack residency; cross-border transfer per SCC |
| A.5.23 Information security for cloud services | ISO 27018 controls — applicability matrix mapped to oyatie's PII-in-cloud posture |
| A.5.26 Response to incidents | `incident-response.md` |
| A.5.31 Legal, statutory, regulatory + contractual requirements | this document + `policy/data-residency.md` |
| A.5.32 Intellectual property | n/a service-side |
| A.5.33 Protection of records | retention policy + audit-chain |
| A.8.2 Privileged access rights | Cedar + DB GRANT separation |
| A.8.3 Information access restriction | per-tenant RLS; Cedar policy |
| A.8.5 Secure authentication | OIDC for affinity-IdP; per-session blinded credential for posting |
| A.8.11 Data masking | structured-log schema refuses user_id field |
| A.8.20 Network security | NetworkPolicy default-deny |
| A.8.23 Web filtering | Cloudflare WAF |
| A.8.24 Use of cryptography | FIPS 140-3 validated libraries where applicable |
| A.8.25 Secure development lifecycle | NIST SSDF + ADR-gated reviews |
| A.8.26 Application security requirements | OWASP ASVS v4 level 3 |
| ISO 27018 A.10.13 Disclosure to law enforcement | `policy/legal-process-disclosure.cedar` + dual-control |
| ISO 27018 A.11.12 Use of PII for marketing | refused; I4 no-third-party-tracker |

### GDPR + EU DSA + EU AI Act

| Article | Coverage |
|---|---|
| GDPR Art. 5 principles | see DPIA Step 4 |
| GDPR Art. 6 lawful basis | see DPIA Step 3 |
| GDPR Art. 11 + Recital 26 pseudonymisation | EXPLICITLY invoked; controller obligations under Arts. 15-22 reduced |
| GDPR Art. 17 right-to-erasure | hard-delete + audit-chain tombstone (FR-13); p99 ≤ 5s propagation |
| GDPR Art. 22 automated individual decision-making | NOT triggered (users anonymous; no individual significant-effect) — documented |
| GDPR Art. 25 privacy-by-design | cryptographic blinding + DB GRANT separation + LEAN lanes |
| GDPR Art. 28 processor obligations | inherited from oyatie master DPA + per-pack overlay |
| GDPR Art. 30 records of processing | this document + DPIA |
| GDPR Art. 32 security measures | TLS 1.3 + at-rest encryption + audit-chain |
| GDPR Art. 33 breach notification | 72h to lead supervisory authority per breach severity |
| GDPR Art. 35 DPIA | `dpia.md` |
| GDPR Arts. 44-50 cross-border transfer | per-pack residency + SCC |
| EU DSA Art. 14 transparency | per-tenant ToS includes anonymous-tier disclosure |
| EU DSA Art. 16 notice-and-action | abuse-report flow (FR-12) |
| EU DSA Art. 17 statement-of-reasons | classifier verdict carries SoR per `capabilities/T2-auto.yaml` |
| EU DSA Art. 20 appeal | appeal flow (FR-20); 7-day SLA |
| EU DSA Art. 24 transparency report | per-tenant quarterly export |
| EU DSA Art. 27 recommender transparency | chronological-default; algorithmic-ranking emits signal explanation when opted-in |
| EU DSA Art. 28 minor protection | age-gate (FR-14); minor accounts chronological-only by default |
| EU AI Act Art. 50 transparency | every classifier verdict carries "AI-assessed" label |

### KR PIPA + 통신비밀보호법 + 정보통신망법

| Article | Coverage |
|---|---|
| KR PIPA Art. 15 collection | affinity-IdP collects; platform stores blinded commitment only |
| KR PIPA Art. 17 use | strictly purpose-bound |
| KR PIPA Art. 18 third-party transfer | NONE except legal-process under court order |
| KR PIPA Art. 21 deletion | hard-delete + tombstone; user-initiated + retention-tier-driven |
| KR PIPA Art. 22-2 consent (alternative pseudonymous processing) | NA — see Art. 24-2 |
| KR PIPA Art. 23 sensitive data | not knowingly collected; user-volunteered content reviewed via moderation |
| KR PIPA Art. 24 unique identifiers (RRN) | NEVER collected |
| KR PIPA Art. 24-2 alternative pseudonymous processing | CANONICAL USE CASE — this µservice is the regulatory archetype for Art. 24-2 |
| KR PIPA Art. 28 cross-border transfer | per-pack residency; KR users on KR pack |
| KR PIPA Art. 29 data-breach notification | 24h to PIPC per breach severity |
| KR PIPA Art. 29-2 automated decision-making | NA (users anonymous; no individual decision) |
| 통신비밀보호법 Art. 5 secrecy | anonymous-DM falls under secrecy of communications |
| 통신비밀보호법 Art. 9 legal-process disclosure | court order via prosecutor required; `runbooks/legal-process-court-order-receipt.md` Path A-KR |
| 통신비밀보호법 Art. 9-2 user notification | within 30 days unless gag order |
| 정보통신망법 Art. 22 user consent | obtained at signup |
| 정보통신망법 Art. 28 personal-data protection technology measures | TLS 1.3 + at-rest encryption + cryptographic blinding |
| 정보통신망법 Art. 29 destruction | hard-delete |
| KR Youth Protection Act | minor signup requires parental consent (per pack overlay) |

### US Section 230 + ECPA + COPPA + state laws

| Statute | Coverage |
|---|---|
| 47 USC §230 (Section 230 CDA) | platform-immunity for user content per Section 230(c)(1); good-faith moderation per Section 230(c)(2) — applies to limited-risk classifier |
| 18 USC §2510-2523 (ECPA Wiretap) | anonymous-DM interception requires court order under Title III |
| 18 USC §2701-2712 (Stored Communications Act) | legal-process disclosure framework — subpoena / court order / search warrant ladder per §2703 |
| 18 USC §2705 gag-order | gag order permitted ≤ 1 year; renewable; runbook Path A-US captures workflow |
| 18 USC §2258A NCMEC | CSAM-suspect reporting within 48h mandatory; runbook Path E |
| 15 USC §6501 + 16 CFR §312 (COPPA) | under-13 ban absolute; FR-14 age-gate enforces |
| First Amendment (anonymous-speech doctrine) | Doe v. Cahill / Krinsky v. Doe heightened standard for unmasking |
| Cal. Civ. Code §1708.7 (anti-doxxing) | doxxing-post moderation triggered |
| NY CRL §50/§51 (publicity rights) | name-without-consent moderation triggered |
| IL 720 ILCS 5/26.5 (cyberstalking) | stalking-pattern moderation triggered |
| 47 USC §223(d) | obscenity / harassing communication |

### UK + AU + JP + SG + IN + BR + AE + KSA

| Pack | Statute | Coverage |
|---|---|---|
| pack-uk | UK OSA 2023 | Ofcom safety-by-design report; illegal-content priority |
| pack-uk | UK IPA 2016 §57 | legal-process disclosure framework |
| pack-uk | UK DPA 2018 + UK GDPR | mirrors GDPR coverage above |
| pack-au | Privacy Act 1988 APP 1-13 | privacy controls |
| pack-au | AU OSA 2021 + BOSE | basic online safety expectations |
| pack-au | TIA Act | intercept legal-process |
| pack-jp | 通信の秘密 Constitutional Art. 21 | secrecy of communications |
| pack-jp | APPI Arts. 17/18/20/21/23/24/26-2/27 | personal-data protection |
| pack-sg | PDPA 2012 §11-26 | data-protection controls |
| pack-in | DPDPA 2023 §6-10 | recently-enacted India regime |
| pack-br | LGPD Arts. 6/7/11/14/18/33/46/48 | Brazilian GDPR-equivalent |
| pack-ae | UAE PDPL Federal Decree-Law 45/2021 | data-protection |
| pack-ksa | PDPL Royal Decree M/19/2021 | data-protection |
| pack-ksa | SAMA Cybersecurity Framework 2017 | financial-sector overlay (where tenant-relevant) |

### FIPS 140-3 cryptographic modules

| Module | Status |
|---|---|
| Blind-signature (`ring 0.17`) | `ring` is BoringSSL-derived; pursuing FIPS 140-3 validation track via Amazon-curated builds |
| BBS+ signatures | `rust-bls` — no FIPS validation; alternative: HashiCorp BBS+ implementation (where required for FIPS tenants) |
| MLS (`oya-mls-rs` wrapper) | MLS algorithms (HKDF, AES-GCM, X25519) per RFC 9420 — base primitives FIPS-validatable; library validation pending |
| TLS | TLS 1.3 via Cloudflare; AWS / OCI native FIPS endpoints when tenant requires |

## Mappings to LEAN lanes

| LEAN lane | Enforces | Framework hooks |
|---|---|---|
| `oya-check-blinding-column-isolation` | I1 | SOC 2 CC6.1 / ISO 27018 A.10.13 |
| `oya-check-third-party-tracker-refused` | I4 | GDPR Art. 25 + ePrivacy Art. 5(3) |
| `oya-check-retention-default-short` | I3 | GDPR Art. 5(1)(e) + KR PIPA Art. 21 |
| `oya-check-e2e-no-plaintext-server-state` | I6 | ECPA / 통신비밀보호법 / 통신의 비밀 |
| `oya-check-ontology-person-write-refused` | I1 | GDPR Art. 25 |
| `oya-check-log-schema-no-user-id` | T-I-06 | ISO 27018 A.10.13 / GDPR Art. 32 |
| `oya-check-notification-payload-opaque-handle-only` | T-I-04 | GDPR Art. 25 |
| `oya-check-search-index-no-author-column` | T-I-05 | GDPR Art. 25 |

## Audit evidence collection

- Audit-chain records exportable per tenant per pack for SOC 2 + ISO 27001 evidence runs.
- LEAN lane pass/fail history retained in artifact-capabilities-registry.json per ADR-0130.
- Per-release SLSA L3 provenance attestation.
- Per-release model card for classifier (EU AI Act Art. 11).
- Per-release golden-set eval report (capabilities/eval/).

## Open questions

| # | Question | Owner |
|---|---|---|
| 1 | FIPS 140-3 validation of BBS+ library — alternative selection if `rust-bls` not validatable in time | ops-security + axis-anonymous |
| 2 | EU AI Office liaison protocol for limited-risk classifier transparency report cadence | council-privacy + legal |
| 3 | Per-pack regulator notification thresholds (data breach 72h GDPR vs 24h KR PIPA) reconciliation in `incident-response.md` | council-privacy + ops-security |
