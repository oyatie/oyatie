---
doc_class: PolicySpec
title: Data Residency Contract
microservice: messenger
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-messenger
deciders: council-privacy, ops-security, axis-messenger, gtm-customer-success
related_adrs: [ADR-0117, ADR-0126, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/messenger/threat-model.md (T-I-01, T-T-02; cross-region replication threats)
  - microservices/messenger/dpia.md (R-11)
  - microservices/messenger/policy/dual-context-isolation.md
  - microservices/messenger/multi-region.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (messenger µservice)

## Purpose

Define which jurisdictions' message + channel + attachment + presence + search data live in which messenger cluster, the cross-pack replication policy, and the legal-transfer mechanisms that gate any exception. This document is reviewed by EU DPAs (GDPR Arts. 44–50), KR PIPC (PIPA Art. 28 + Art. 23-2), HIPAA Covered Entity counsel (BAA), and equivalent supervisory authorities per active pack.

## Residency Model

### Default: pack-pinning

Every tenant is assigned a primary pack at onboarding. Cross-pack movement is **forbidden by default**.

| Pack | Primary region(s) | Cluster footprint | Activated? |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-messenger-{postgres,redis,s3,search,gateway} | YES (M02 launch) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | eu-messenger-* | Conditional (first EU SCC) |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 (DR pair) | us-messenger-* | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | us-hc-messenger-* (isolated from non-HC) | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | jp-messenger-* | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-messenger-* | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-messenger-* | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-messenger-* | Conditional |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-messenger-* | Conditional |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-messenger-* | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-messenger-* | Conditional |

### Personal vs Professional residency

Per parallel ADR-0126 dual-context model:

- **Professional channel data** follows the tenant's pack pinning (above).
- **Personal-DM data** follows the user's per-user residency (set at user signup). For most users this matches the tenant's pack, but a user who travels or relocates may have personal data residency change while their tenant pack stays fixed. The system treats this as two separate residency keys.

### Pack-assignment routing

```text
Tenant onboarding → primary pack
User signup → personal-residency pack (default = HQ jurisdiction)
    ↓
OpenBao records both keys; messenger reads at every request.
```

## Cross-Pack Replication Policy

### Default: forbidden

- Postgres logical replication: within-pack only.
- Redis cluster replication: within-pack only.
- S3 cross-region replication: within-pack only.
- Tantivy/ES index replication: within-pack only.

### Allowed exceptions

- DR-pair packs replicate primary → warm-standby within the pack.
- Personal-DM E2E ciphertext is portable (still ciphertext); user device determines decryption locus.
- Audit-chain seals are cross-pack-replicable because they contain no PII (just commit-hash + signature).

### Disallowed everywhere

- Search index sharing across packs (rebuilt per-pack).
- Mention-resolution caches sharing across packs.
- Presence across packs (presence is in-region only).

## Retention Bounds Per Pack

| Pack | Professional message retention floor | Professional message retention ceiling | Personal DM retention | Notes |
|---|---|---|---|---|
| pack-kr | 1 year (KR labor-record floor) | 5 years default; tenant-configurable up to 7y | per-user policy (default 1 year) | KR PIPA Art. 21 |
| pack-eu | none (GDPR storage-limitation; tenant-defined) | 7y max (GDPR storage-limitation principle) | per-user policy | GDPR Art. 5(1)(e) |
| pack-us | none | 7y max | per-user policy | varies state |
| pack-us-healthcare | 6 years (HIPAA §164.530(j)) | 10y max | n/a (no personal-DM HIPAA scenario typically) | HIPAA |
| pack-jp | 2 years (APPI labor) | 7y max | per-user policy | APPI |
| pack-sg | 1 year | 7y max | per-user policy | PDPA |
| pack-au | 7 years (Privacy Act default) | 10y max | per-user policy | Privacy Act 1988 |
| pack-in | 3 years (DPDPA processing-purpose) | 7y max | per-user policy | DPDPA 2023 |
| pack-br | 5 years (LGPD purpose-limitation) | 7y max | per-user policy | LGPD |
| pack-ae | 1 year | 7y max | per-user policy | UAE PDPL |
| pack-ksa | 5 years | 10y max | per-user policy | PDPL + SAMA |

## Attachment Residency

Attachments inherit the parent message's `context_kind` and pack. Cross-pack attachment URL sharing is forbidden; tenants who federate (out-of-scope, future ADR) must opt in per-tenant + per-channel.

## DSR Cascade (right-to-erasure)

When a data subject exercises right-to-erasure:

1. DSR cascade runner identifies all messages authored by the subject across all channels they participated in (within the pack).
2. Marks rows tombstoned + redacts identifiers (replaces handle with `«erased»`).
3. Search index re-emits affected docs in redacted form.
4. Audit-chain notes the redaction event (NOT the redacted content).
5. SLA: 30 days from request per GDPR; faster where local law requires.
6. Retention-floor conflict: if pack retention floor (e.g., HIPAA 6y) requires preservation, the redaction redacts identifiers only; body stays in audit-protected form with access bound to compliance-officer + four-eyes.

## Cross-Border Transfer

Forbidden by default. Allowed only with:

- Tenant SCC (Standard Contractual Clauses) on file for GDPR-scope tenants.
- Tenant-of-tenant consent for end-user data flowing under tenant's contract.
- pack-us-healthcare: BAA + HIPAA-eligible source + HIPAA-eligible target.

Cross-border transfer register: `microservices/messenger/legal/transfer-register.md`.

## Verification

- CI lane `oya-check-messenger-pack-residency` asserts every Helm release is pack-pinned via `commonLabels.oyatie/pack`.
- Periodic Postgres audit: no row carries `pack != cluster_pack`.
- Periodic S3 audit: no object lives in a bucket whose region != pack's region.

## References

- ADR-0117.
- Parallel ADR-0126.
- `microservices/messenger/multi-region.md`.
- `microservices/messenger/policy/dual-context-isolation.md`.
- GDPR Arts. 44–50; KR PIPA Arts. 28, 23-2; HIPAA §164.502, §164.530; APPI Art. 27; LGPD Arts. 33, 46.
