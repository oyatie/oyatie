---
doc_class: PolicySpec
title: Data Residency + Retention + DSR Cascade
microservice: network
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + axis-network
deciders: council-privacy, ops-security, axis-network, council-architecture, ops-compliance
related_adrs: [ADR-0117, ADR-0126, ADR-0131]
related_artifacts:
  - microservices/network/policy/professional-context-isolation.md
  - microservices/network/threat-model.md
  - microservices/network/dpia.md
  - microservices/network/compliance.md
  - microservices/network/multi-region.md
review_cadence: annually + on every pack activation
doc_status: published
---

# Data Residency + Retention + DSR Cascade (network µservice)

## Purpose

Define data-residency boundaries, retention floors + ceilings, cross-pack policy, and Data-Subject-Rights (DSR) cascade for the `network` µservice. Authority is per-pack; `network` is Professional-tier only so residency follows the tenant (not the user, as in sibling `social`).

## Per-Pack Residency Matrix

| Pack | Primary residency | Cross-border transfer policy | Retention floor (Professional) | Retention ceiling | Notes |
|---|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | Forbidden by default; SCC required + tenant explicit consent per KR PIPA Art. 17 | 365d work-record floor per KR 근로기준법 Art. 42 | 1825d (5y; tenant-configurable to 7y) | `KR PIPA Arts. 21, 28, 29; 근로기준법 Arts. 42, 49; 직장 갑질 protections; 통신비밀보호법` |
| pack-eu | OCI eu-frankfurt-1 (DR: eu-amsterdam-1) | EU + EEA only; cross-border requires SCC (Art. 46) + transfer impact assessment | 180d minimum for audit + bias-audit records per EU AI Act Art. 12 | 2555d (7y; GDPR storage-limitation principle Art. 5(1)(e)) | `GDPR Arts. 5, 22, 25, 32, 33, 35, 44–50; EU AI Act 2024/1689 Annex III §4; EU DSA 2065/2022; EU Pay Transparency Directive 2023/970` |
| pack-us | OCI us-ashburn-1 (DR: us-phoenix-1) | US only by default; tenant-opt-in cross-region within US | 730d per EEOC UGESP 29 CFR §1607.4 record-keeping minimum | 2555d (7y) | `Title VII; ADA; ADEA; EEOC UGESP; CCPA + CPRA; NYC LL144 + DCWP; CA AB-331; CO SB 24-205; IL HB 3773` |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible; DR: us-phoenix-1 HIPAA-eligible) | HIPAA-eligible regions only | 6y per HIPAA §164.530(j) | 6y minimum; 10y default | `HIPAA + BAA + COPPA` |
| pack-jp | OCI ap-tokyo-1 | JP only by default | Per JP 労働基準法 Art. 109 (3y work record); 5y for audit | 1825d (5y) | `APPI; 個人情報保護法; 労働基準法; 労働契約法` |
| pack-sg | OCI ap-singapore-1 | SG only by default | Per PDPA + Fair Consideration Framework | 1825d (5y) | `PDPA; PDPC employment guidance; FCF; SkillsFuture` |
| pack-au | OCI ap-sydney-1 (DR: ap-melbourne-1) | AU only by default | Per Fair Work Act 2009 §535 (7y employee records) | 2555d (7y) | `Privacy Act 1988; AHRC AI guidance; Fair Work Act 2009` |
| pack-in | OCI ap-hyderabad-1 (DR: ap-mumbai-1) | IN only by default; cross-border requires DPDP Board approval | Per DPDPA 2023 + Industrial Disputes Act 1947 | 1825d (5y) | `DPDPA 2023; IDA 1947; Equal Remuneration Act 1976` |
| pack-br | OCI sa-saopaulo-1 (DR: sa-vinhedo-1) | BR only by default | Per CLT (Consolidação das Leis do Trabalho) art. 477 | 1825d (5y) | `LGPD; CLT` |
| pack-ae | OCI me-abudhabi-1 (DR: me-dubai-1) | UAE only by default | Per UAE Federal Decree-Law 33/2021 (Labour Law) | 1825d (5y) | `UAE PDPL; Federal Decree-Law 33/2021` |
| pack-ksa | OCI me-jeddah-1 (DR: me-riyadh-1) | KSA only by default | Per KSA Labor Law (Royal Decree M/51) | 1825d (5y) | `KSA PDPL; KSA Labor Law; SDAIA AI guidance` |

## Data Classes (Professional-only)

| Class | Examples in `network` | Default storage |
|---|---|---|
| `PUBLIC` | Public-visibility Professional profiles, public posts, public Pages | encrypted at rest; readable by anonymous |
| `INTERNAL_ONLY` | Tenant operator views, dashboards | tenant-scope Cedar; never anonymous |
| `BEHAVIORAL_TENANT_PRODUCT` | Feed render, ranker score signal, search index entries | tenant-DEK; tenant-scope Cedar |
| `BEHAVIORAL_USER_CONTENT` | Posts, comments, reactions, hashtags, mentions | tenant-DEK; tenant-scope Cedar |
| `PII_IDENTIFYING` | Display name, handle, headline, location, contact field | tenant-DEK; minimisation; Cedar |
| `EMPLOYMENT_RECORD` | Resume, experience entries, education entries, certifications | tenant-DEK; pack retention floor applies; ADR-NET-0001 |
| `ENDORSEMENT_RECORD` | Per-skill endorsement, recommendation body, per-endorser Ed25519 signature | tenant-DEK + per-endorser KMS Ed25519; immutable record |
| `RELATIONSHIP_GRAPH` | Connection edges, follow edges, block/restrict edges, degree cache | tenant-DEK; tenant-scope Cedar |
| `INMAIL_BODY` | InMail bodies, threads | tenant-DEK; four-eyes for disclosure |
| `JOB_POSTING` | Job postings, applicant referrals | tenant-DEK; ATS-handoff event contract |
| `RECRUITER_DECISION_AUDIT` | Recruiter-stub ranker inputs, decision, contributing signals, bias-audit verdicts | tenant-DEK + audit-chain seal; 2y minimum (EEOC UGESP) |
| `MINOR_PROTECT` | Minor-account flag, parental-consent proof | sealed; never enters recruiter / salary-insights / search results; Cedar `minor_protect_reader` only |
| `AUDIT` | Audit-chain seals over all of the above | hash-only seals; cross-pack OK |

## Retention Mechanism

1. Each row carries `tenant_id` + `pack_id` + `data_class` + `created_at`.
2. Retention worker daily-runs per pack overlay.
3. Per-tenant per-data-class TTL is enforced; expiration triggers tombstone + audit-chain seal.
4. Endorsement records are *never auto-deleted* during retention floor; tenant-admin DSR is required to revoke (per Art. 17 + ADR-NET-0005).
5. Recruiter-decision audit records have an EEOC UGESP 2y minimum floor and EU AI Act Art. 12 ≥ 180d floor; the more-protective applies per pack.

## DSR Cascade (GDPR Art. 17 + Art. 20 + Art. 21 + Art. 22)

When a tenant or end-user invokes DSR:

| Right | Cascade |
|---|---|
| Art. 15 (access) | Tenant operator under Cedar `tenant_operator` reads own data; end-user under Cedar `end_user` reads own data via SDK helper. |
| Art. 16 (rectification) | Edit-profile / edit-experience / edit-skills surfaces emit audit; pack retention applies. |
| Art. 17 (erasure) | Cascade across `professional_profiles` + `connection_edges` + `posts` + `endorsements` + `recommendations` + `inmail_bridge` + `pages` + `groups` + `events` + `jobs_handoff` + `recruiter_audit` + `search_indexes` + `audit-chain`. Endorsement records are tombstoned (cryptographic chain intact; body-tombstoned); search indexes purge; audit-chain seals retained per record-keeping floor. End-user-initiated erasure subject to retention-floor (work-record retention; legal-hold). |
| Art. 20 (portability) | Profile-export emits vCard 4.0 (RFC 6350) + JSON Resume + GDPR Art. 20 portable JSON; includes connection-graph references, endorsement references, post references (signed-URLs for media). |
| Art. 21 (object) | Opt-out of recommender + recruiter ranker + people-you-may-know recommender via `setAutomatedDecisionPreference`. |
| Art. 22 (automated decision-making) | Right-to-human-review on recruiter-stub + jobs-ranker + endorsement-aggregation; surfaced via SDK `getHumanReviewOption()`; tenant operator + compliance lead in loop. |

DSR cascade test: `cargo run -p oya-dev-cli -- network dsr-cascade-test --tenant <t> --user <u>` (synthetic data only).

## Cross-Pack Replication: Forbidden

No `network` data crosses pack boundaries except:

- Audit-chain seals (hash-only; no PII).
- Public OpenAPI / AsyncAPI / proto schemas.
- Endorsement-chain Merkle root hashes for cross-pack replay verification (hash-only).
- Per-tenant aggregate salary-insights bands (aggregate; ≥ k-anonymity floor; no per-individual disclosure).

Any cross-pack data flow outside these exceptions triggers `network_pack_residency_violation_total` (target = 0); Sev-1 alert (FM-13).

## Tenant Operator + End-User Surfaces

- Tenant operators see own-tenant data only; Cedar `tenant_operator` entitlement scoped via OpenBao.
- End-users see own data + connections' data subject to per-resource Cedar; never cross-tenant.
- Anonymous (public-read) sees only public-visibility Professional profiles + public posts + public Pages.

## Verification

- Per-pack DSR cascade integration test: synthetic user; invoke erasure; verify all classes erased per cascade matrix.
- Annual residency audit: per-pack row count by `tenant.pack_id`; no row should belong to a tenant whose `pack_id` differs from the cluster's pack overlay.
- Annual third-party audit (council-privacy-engaged): confirm DPA + BAA + DPDPA / LGPD / PIPA / APPI per-tenant signed status.

## References

- ADR-0117 (pack-pinning).
- Parallel ADR-0126.
- ADR-0131 (per-microservice flat layout).
- ADR-NET-0001 (EMPLOYMENT_RECORD data class).
- ADR-NET-0005 (endorsement-chain tombstone semantics).
- ADR-NET-0006 (profile portability + export).
- `microservices/network/threat-model.md`.
- `microservices/network/compliance.md`.
- `microservices/social/policy/data-residency.md` (sibling reference).
- GDPR Arts. 5, 17, 20, 21, 22; KR PIPA Arts. 17, 21, 28, 29; HIPAA §164.530(j); EU AI Act Art. 12; EEOC UGESP 29 CFR §1607.4.
