---
doc_class: PolicySpec
title: Data Residency Contract
microservice: social
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-social
deciders: council-privacy, ops-security, axis-social, gtm-customer-success
related_adrs: [ADR-0117, ADR-0135, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/social/threat-model.md (T-I-01, T-T-02; cross-region replication threats)
  - microservices/social/dpia.md (R-12)
  - microservices/social/policy/dual-context-isolation.md
  - microservices/social/multi-region.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (social µservice)

## Purpose

Define which jurisdictions' profile + post + follow-graph + media + reactions + search + notification data live in which social cluster, the cross-pack replication policy, the federation egress policy (Professional-tier only, opt-in), and the legal-transfer mechanisms that gate any exception. This document is reviewed by EU DPAs (GDPR Arts. 44–50), KR PIPC (PIPA Art. 28 + Art. 23-2), HIPAA Covered Entity counsel (BAA), EU DSA Coordinator, EU AI Act notified body, and equivalent supervisory authorities per active pack.

## Residency Model

### Default: pack-pinning

Every tenant is assigned a primary pack at onboarding. Cross-pack movement is **forbidden by default**.

| Pack | Primary region(s) | Cluster footprint | Activated? |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-social-{postgres,redis,s3,search,gateway} | YES (M02 launch) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | eu-social-* | Conditional (first EU SCC) |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 (DR pair) | us-social-* | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | us-hc-social-* (isolated from non-HC) | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | jp-social-* | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-social-* | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-social-* | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-social-* | Conditional |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-social-* | Conditional |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-social-* | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-social-* | Conditional |

### Personal vs Professional residency

Per parallel ADR-0135 dual-context model:

- **Professional profile + post data** follows the tenant's pack pinning (above).
- **Personal profile + post data** follows the user's per-user residency (set at user signup). For most users this matches the tenant's pack, but a user who travels or relocates may have personal data residency change while their tenant pack stays fixed. The system treats this as two separate residency keys.

### Pack-assignment routing

```text
Tenant onboarding → primary pack (Professional-tier)
User signup → personal-residency pack (default = HQ jurisdiction)
    ↓
OpenBao records both keys; social reads at every request.
```

## Cross-Pack Replication Policy

### Default: forbidden

- Postgres logical replication: within-pack only.
- Redis cluster replication: within-pack only.
- S3 cross-region replication: within-pack only.
- Meilisearch index replication: within-pack only.

### Allowed exceptions

- DR-pair packs replicate primary → warm-standby within the pack.
- Audit-chain seals are cross-pack-replicable because they contain no PII (just commit-hash + signature).
- ActivityPub federation egress (Professional-tier only) crosses pack borders to external peers under per-tenant opt-in + SCC.

### Disallowed everywhere

- Search index sharing across packs (rebuilt per-pack).
- Mention-resolution caches sharing across packs.
- Trending-topic windows across packs (computed per-pack).
- Follow-graph across packs (intra-pack only; cross-pack follow is treated as inactive edge for fanout).

## Retention Bounds Per Pack

| Pack | Professional post retention floor | Professional post retention ceiling | Personal post retention | Notes |
|---|---|---|---|---|
| pack-kr | 1 year (KR labor-record floor when work-context) | 5 years default; tenant-configurable up to 7y | per-user policy (default 1 year) | KR PIPA Art. 21 |
| pack-eu | none (GDPR storage-limitation; tenant-defined) | 7y max | per-user policy | GDPR Art. 5(1)(e) |
| pack-us | none | 7y max | per-user policy | varies state |
| pack-us-healthcare | 6 years (HIPAA §164.530(j)) | 10y max | n/a (no personal-tier HIPAA scenario typically) | HIPAA |
| pack-jp | 2 years (APPI labor) | 7y max | per-user policy | APPI |
| pack-sg | 1 year | 7y max | per-user policy | PDPA |
| pack-au | 7 years (Privacy Act default) | 10y max | per-user policy | Privacy Act 1988 |
| pack-in | 3 years (DPDPA processing-purpose) | 7y max | per-user policy | DPDPA 2023 |
| pack-br | 5 years (LGPD purpose-limitation) | 7y max | per-user policy | LGPD |
| pack-ae | 1 year | 7y max | per-user policy | UAE PDPL |
| pack-ksa | 5 years | 10y max | per-user policy | PDPL + SAMA |

## Media Residency

Media inherits the parent post's `context_kind` and pack. Cross-pack media URL sharing is forbidden by default; tenants who federate (Professional-tier only, opt-in) emit signed media-fetch URLs to peers but the underlying blob never leaves the source pack (federation peer fetches from oyatie-signed CDN endpoint within source pack's CDN POP).

## DSR Cascade (right-to-erasure)

When a data subject exercises right-to-erasure:

1. DSR cascade runner identifies all posts authored by the subject across all visibility scopes (within the pack).
2. Marks rows tombstoned + redacts identifiers (replaces handle with `«erased»`).
3. Search index re-emits affected docs in redacted form.
4. Audit-chain notes the redaction event (NOT the redacted content).
5. Follow-graph edges originating from / to the subject are tombstoned.
6. Reactions by the subject are anonymised but tally retained.
7. SLA: 30 days from request per GDPR; faster where local law requires.
8. Retention-floor conflict: if pack retention floor (e.g., HIPAA 6y) requires preservation, the redaction redacts identifiers only; body stays in audit-protected form with access bound to compliance-officer + four-eyes.

## Cross-Border Transfer

Forbidden by default. Allowed only with:

- Tenant SCC (Standard Contractual Clauses) on file for GDPR-scope tenants.
- Tenant-of-tenant consent for end-user data flowing under tenant's contract.
- pack-us-healthcare: BAA + HIPAA-eligible source + HIPAA-eligible target.
- ActivityPub federation (Professional-tier only): per-tenant opt-in + per-peer SCC where applicable.

Cross-border transfer register: `microservices/social/legal/transfer-register.md` (Slice B).

## Federation Egress Residency Rules

| Tier | Federation egress allowed? | Conditions |
|---|---|---|
| Personal-tier | **NEVER** | Compile-time invariant; Cedar belt-and-suspenders forbid |
| Professional-tier | Opt-in per tenant | Tenant must (a) sign SCC for cross-border peers, (b) attest peer-allowlist, (c) accept that federation egress = cross-pack data flow per pack residency rules |
| pack-us-healthcare Professional-tier | OFF by default | HIPAA Safe Harbor; tenant may activate with BAA + per-peer attestation |
| pack-kr (KR PIPA Art. 28) | Cross-border requires user-consent | Per Art. 28; explicit consent recorded at tenant + per-user level |

## Verification

- CI lane `oya-check-social-pack-residency` asserts every Helm release is pack-pinned via `commonLabels.oyatie/pack`.
- Periodic Postgres audit: no row carries `pack != cluster_pack`.
- Periodic S3 audit: no object lives in a bucket whose region != pack's region.
- Federation egress audit: every outbound activity carries source pack + target peer pack + tenant opt-in record; non-conforming activity blocked.

## References

- ADR-0117.
- Parallel ADR-0135.
- ADR-SOC-0004 (federation posture).
- `microservices/social/multi-region.md`.
- `microservices/social/policy/dual-context-isolation.md`.
- GDPR Arts. 44–50; KR PIPA Arts. 28, 23-2; HIPAA §164.502, §164.530; APPI Art. 27; LGPD Arts. 33, 46; ActivityPub W3C Rec 2018.
