---
doc_class: PolicySpec
title: Data Residency Contract
microservice: recordings
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-recordings
deciders: council-privacy, ops-security, axis-recordings, ops-compliance
related_adrs: [ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-RECORDINGS-0002, ADR-RECORDINGS-0005]
related_artifacts:
  - microservices/recordings/threat-model.md
  - microservices/recordings/dpia.md
  - microservices/recordings/multi-region.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (recordings µservice)

## Purpose

Define which jurisdictions' recording media + transcript + redaction overlay
+ retention policy + legal-hold + audit-chain seal data live in which
recordings cluster, the cross-pack replication policy, and the legal-transfer
mechanisms that gate any exception. Reviewed by EU DPAs (GDPR Arts. 44–50),
KR PIPC (PIPA Art. 28 + Art. 23-2 + 통신비밀보호법), HIPAA Covered Entity
counsel (BAA), SEC examiners (17a-4(f)), and FINRA (4511).

## Residency Model

### Default: pack-pinning

Every tenant is assigned a primary pack at onboarding. Cross-pack movement
is **forbidden by default**.

| Pack | Primary region(s) | Cluster footprint | Activated? |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-recordings-{postgres,valkey,s3-hot,s3-cold,meilisearch,foundry-runtime,cdn-self-host} | YES (M02 launch) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | eu-recordings-* | Conditional (first EU SCC) |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 (DR pair) | us-recordings-* | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | us-hc-recordings-* (isolated from non-HC) | Conditional (post-BAA) |
| pack-us-financial | OCI us-ashburn-1 (SEC 17a-4-eligible) | us-fin-recordings-* (S3 object-lock WORM) | Conditional |
| pack-jp | OCI ap-tokyo-1 | jp-recordings-* | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-recordings-* | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-recordings-* | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-recordings-* | Conditional |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-recordings-* | Conditional |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-recordings-* | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-recordings-* | Conditional |

## Cross-Pack Replication Policy

### Default: forbidden

- Postgres logical replication: within-pack only.
- S3 hot + S3 cold cross-region replication: within-pack only.
- Valkey cluster replication: within-pack only.
- Meilisearch index replication: within-pack only.
- foundry-runtime (Whisper + pyannote) inference: within-pack only.

### Allowed exceptions

- DR-pair packs replicate primary → warm-standby within the pack.
- Audit-chain seals are cross-pack-replicable (no PII; Merkle root + signature
  only).

### Disallowed everywhere

- Source media sharing across packs.
- Transcript sharing across packs (translate µservice operates within-pack).
- Search index sharing across packs (rebuilt per-pack).
- Per-viewer watermark key sharing across packs.

## Retention Bounds Per Pack

| Pack | Recording retention floor | Recording retention ceiling | Transcript retention | Notes |
|---|---|---|---|---|
| pack-kr | 1 year (default labor floor) | 5 years default; tenant-configurable up to 7y | matches recording | KR PIPA Art. 21; 전자문서법 |
| pack-eu | none (GDPR storage-limitation; tenant-defined) | 7y max | matches recording | GDPR Art. 5(1)(e) |
| pack-us | none | 7y max | matches recording | varies state |
| pack-us-healthcare | 6 years (HIPAA §164.530(j)) | 10y max | matches recording (PHI scope) | HIPAA |
| pack-us-financial | 3 years (SEC 17a-4(b)(4); first 2y in non-erasable) | 7y max; on-request to 10y | matches recording | SEC 17a-4 + FINRA 4511 + MiFID II 16(7) |
| pack-jp | 2 years (APPI labor) | 7y max | matches recording | APPI |
| pack-sg | 1 year | 7y max | matches recording | PDPA |
| pack-au | 7 years (Privacy Act default) | 10y max | matches recording | Privacy Act 1988 |
| pack-in | 3 years (DPDPA processing-purpose) | 7y max | matches recording | DPDPA 2023 |
| pack-br | 5 years (LGPD purpose-limitation) | 7y max | matches recording | LGPD |
| pack-ae | 1 year | 7y max | matches recording | UAE PDPL |
| pack-ksa | 5 years | 10y max | matches recording | PDPL + SAMA |

## Storage Tiering Per Pack (per ADR-RECORDINGS-0005)

| Pack | Hot tier retention | Cold tier (Glacier-class) retention |
|---|---|---|
| pack-kr | 90 days | until pack ceiling |
| pack-eu | 90 days | until tenant policy ceiling |
| pack-us | 90 days | until tenant policy ceiling |
| pack-us-healthcare | 1 year (frequent access for clinical review) | until 10y ceiling |
| pack-us-financial | full first 2y in hot (SEC 17a-4 non-erasable) — actually WORM-class hot | until 7y/10y |
| (other packs) | 90 days | until ceiling |

## Attachment / Manual Upload Residency

Manual uploads inherit the uploader's tenant pack. Cross-pack uploads are
refused; CI lane `oya-check-recordings-pack-residency` enforces.

## DSR Cascade (right-to-erasure)

When a data subject exercises right-to-erasure:

1. DSR cascade runner identifies all recordings + transcripts where the
   subject appears (by speaker_id binding + named participant ref).
2. Marks rows tombstoned + emits redaction overlay over the identified
   speaker's spans.
3. Search index re-emits affected docs in redacted form.
4. Audit-chain notes the redaction event (NOT the redacted content).
5. SLA: 30 days from request per GDPR; faster where local law requires.
6. Retention-floor conflict: if pack retention floor (e.g., HIPAA 6y, SEC
   17a-4 3y) requires preservation, the redaction redacts identifiers only;
   body stays in audit-protected form with access bound to compliance-
   officer + four-eyes.

## Cross-Border Transfer

Forbidden by default. Allowed only with:

- Tenant SCC (Standard Contractual Clauses) on file for GDPR-scope tenants.
- Tenant-of-tenant consent for end-user data flowing under tenant's contract.
- pack-us-healthcare: BAA + HIPAA-eligible source + HIPAA-eligible target.
- pack-us-financial: SEC 17a-4-eligible storage at target.

Cross-border transfer register: `microservices/recordings/legal/transfer-register.md`.

## Recording-Consent Gate at Ingest

Per KR 통신비밀보호법 + TIA Act (AU) + state Surveillance Devices Acts +
ePrivacy Art. 5(3):

- Ingest contract refuses recording without `consent_banner_confirmed: true`
  flag from the producer.
- Producer µservice (meet / messenger / live-broadcast) emits the banner at
  session start; the recording metadata reflects the consent-confirmation
  state.

## Verification

- CI lane `oya-check-recordings-pack-residency` asserts every Helm release
  is pack-pinned via `commonLabels.oyatie/pack`.
- Periodic Postgres audit: no row carries `pack != cluster_pack`.
- Periodic S3 audit: no object lives in a bucket whose region != pack's region.
- Per ADR-0139: residency breach is Sev-1.

## References

- ADR-0117, ADR-RECORDINGS-0002, ADR-RECORDINGS-0005.
- Parallel ADR-0135.
- `multi-region.md`, `compliance.md`.
- GDPR Arts. 44–50; KR PIPA Arts. 28, 23-2; HIPAA §164.502, §164.530; SEC
  17a-4(f); FINRA 4511; MiFID II 16(7); APPI Art. 27; LGPD Arts. 33, 46;
  KR 통신비밀보호법; TIA Act + Surveillance Devices Act; ePrivacy 2002/58
  Art. 5(3).
