---
doc_class: PolicySpec
title: Data Residency Contract
microservice: shorts
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-shorts
deciders: council-privacy, ops-security, axis-shorts, gtm-customer-success, ops-legal
related_adrs: [ADR-0117, ADR-0135, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/shorts/threat-model.md (T-I-01, T-I-08; cross-region replication threats)
  - microservices/shorts/dpia.md (R-12)
  - microservices/shorts/policy/dual-context-isolation.md
  - microservices/shorts/multi-region.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (shorts µservice)

## Purpose

Define which jurisdictions' video metadata + video blobs + transcode variants + thumbnails + captions + watch-time + likes + comments + claims + ages + parental + analytics + DRM-key + fingerprint-corpus + search + notification data live in which shorts cluster, the cross-pack replication policy, the federation egress policy (Professional-tier only, opt-in, metadata-only), the DRM tenant-tier gating policy, and the legal-transfer mechanisms. This document is reviewed by EU DPAs (GDPR Arts. 44–50), KR PIPC (PIPA Art. 28 + Art. 23-2), HIPAA Covered Entity counsel (BAA), EU DSA Coordinator, EU AI Act notified body, EU AVMSD coordinator, UK Ofcom, AU eSafety Commissioner, US Copyright Office DMCA agent, and equivalent supervisory authorities per active pack.

## Residency Model

### Default: pack-pinning

Every tenant is assigned a primary pack at onboarding. Cross-pack movement is **forbidden by default**.

| Pack | Primary region(s) | Cluster footprint | Activated? |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-shorts-{postgres,redis,s3,cdn,search,gateway,ffmpeg-pool,drm-keyserver} | YES (M03 launch) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | eu-shorts-* | Conditional (first EU SCC) |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 (DR pair) | us-shorts-* | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | us-hc-shorts-* (isolated from non-HC) | Conditional (post-BAA; patient-ed use case) |
| pack-jp | OCI ap-tokyo-1 | jp-shorts-* | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-shorts-* | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-shorts-* | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-shorts-* | Conditional |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-shorts-* | Conditional |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-shorts-* | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-shorts-* | Conditional |

### Personal vs Professional residency

Per parallel ADR-0135 dual-context model:

- **Professional creator + video data** follows the tenant's pack pinning (above).
- **Personal creator + video data** follows the user's per-user residency (set at user signup). For most users this matches the tenant's pack, but a user who travels or relocates may have personal data residency change while their tenant pack stays fixed. The system treats this as two separate residency keys.

### Pack-assignment routing

```text
Tenant onboarding → primary pack (Professional-tier)
User signup → personal-residency pack (default = HQ jurisdiction)
    ↓
OpenBao records both keys; shorts reads at every request.
```

## Cross-Pack Replication Policy

### Default: forbidden

- Postgres logical replication: within-pack only.
- Redis cluster replication: within-pack only.
- S3 cross-region replication: within-pack only.
- Meilisearch index replication: within-pack only.
- ffmpeg transcode jobs: within-pack only.
- DRM key system: within-pack only.
- Fingerprint corpus: within-pack only (unless licensed cross-pack).

### Allowed exceptions

- DR-pair packs replicate primary → warm-standby within the pack.
- Audit-chain seals are cross-pack-replicable because they contain no PII (just commit-hash + signature).
- ActivityPub federation egress (Professional-tier only) crosses pack borders to external peers under per-tenant opt-in + SCC + metadata-only (video blob NEVER crosses pack boundary; federation peer fetches from source-pack CDN POP under signed-URL).
- CDN edge POPs serve from in-pack S3 origin; cross-POP cache propagation is permitted for in-pack content.

### Disallowed everywhere

- Video blob cross-pack replication (federation never crosses blobs; only manifest reference + signed CDN URL).
- Search index sharing across packs (rebuilt per-pack).
- Trending-sound windows across packs (computed per-pack).
- DRM per-content keys sharing across packs.
- Age-attestation table sharing across packs.
- Parental-link table sharing across packs.
- Fingerprint corpus sharing across packs unless explicitly licensed cross-pack (rare; ops-legal sign-off).

## Retention Bounds Per Pack

| Pack | Professional video retention floor | Professional video retention ceiling | Personal video retention | Notes |
|---|---|---|---|---|
| pack-kr | 1 year (KR PIPA Art. 21 work-context) | 5 years default; tenant-configurable up to 7y | per-user policy (default 1 year) | KR Telecommunications Business Act + KR 청소년 보호법 |
| pack-eu | none (GDPR storage-limitation; tenant-defined) | 7y max | per-user policy | GDPR Art. 5(1)(e); EU AVMSD record-keeping where applicable |
| pack-us | none | 7y max | per-user policy | DMCA repeat-infringer records: 3y minimum |
| pack-us-healthcare | 6 years (HIPAA §164.530(j); patient-ed only) | 10y max | n/a (HIPAA Personal-tier rare) | HIPAA + BAA |
| pack-jp | 2 years (APPI labor) | 7y max | per-user policy | APPI |
| pack-sg | 1 year | 7y max | per-user policy | PDPA + Online Safety Act 2022 |
| pack-au | 7 years (Privacy Act 1988 default) | 10y max | per-user policy | Privacy Act + Online Safety Act 2021 + BOSE 2022 |
| pack-in | 3 years (DPDPA processing-purpose) | 7y max | per-user policy | DPDPA 2023 |
| pack-br | 5 years (LGPD purpose-limitation) | 7y max | per-user policy | LGPD + Marco Civil |
| pack-ae | 1 year | 7y max | per-user policy | UAE PDPL |
| pack-ksa | 5 years | 10y max | per-user policy | PDPL + SAMA |

### Retention for derived data

- Watch-time sessions: 90d hot, aggregated permanently (k-anonymity ≥ 10 floor in creator-analytics).
- Like / share / comment: 365d hot.
- Notifications: 90d hot.
- Captions: per-video (tied to video retention).
- Transcode variants: rebuilt from originals; retention follows original.
- Thumbnails: rebuilt; retention follows original.
- Trending windows + sound-of-the-week: rebuilt continuously; archived per-pack 90d.
- Search index: rebuilt from Postgres; retention follows source.
- DMCA records: pack-aware (default 3y; pack-us §512(i)(1)(A) repeat-infringer floor; pack-eu 7y).
- Age attestation: per-pack (KR 청소년 보호법 retention; EU AI Act per-classifier-version evidence; COPPA verifiable-consent retention).
- Parental-link records: per-pack; retention follows tenant policy.
- Fingerprint corpus: append-only with licensor-controlled lifecycle.
- DRM per-content keys: per-content rotation 7d; replaceable.
- Audit-chain seals: append-only; immutable; per pack.

## Media Residency

- Video blobs (original + transcode variants + thumbnails + captions) follow source-post pack.
- CDN POPs within pack region; cross-pack edge replication is metadata-only (manifest references).
- Cross-pack video URL sharing forbidden by default; tenants who federate (Professional-tier only, opt-in) emit signed media-fetch URLs to peers but the underlying blob never leaves the source pack (federation peer fetches from oyatie-signed CDN endpoint within source pack's CDN POP).

## DRM Tenant-Tier Gating

| Tier | Widevine | FairPlay | PlayReady | Default |
|---|---|---|---|---|
| Free | OFF | OFF | OFF | DRM unavailable |
| Basic | OFF | OFF | OFF | DRM unavailable |
| Premium | ✓ | ✓ | ✓ | DRM available; tenant-opt-in per video |
| Enterprise | ✓ | ✓ | ✓ | DRM default-on (tenant-policy override) |

Per-tier gating enforced at:
- `oya-shorts-drm-kernel` port: `DrmLicenseIssuer::issue(post, tier)` refuses Premium-feature on non-Premium tier.
- Cedar `policy/tenant-scope.cedar` PERMIT 1 → DRM action requires `tenant.tier in ["Premium", "Enterprise"]`.

## DSR Cascade (right-to-erasure)

When a data subject exercises right-to-erasure:

1. DSR cascade runner identifies all videos authored + watched + reacted to + commented + reposted + claimed by the subject across all visibility scopes (within the pack).
2. Marks rows tombstoned + redacts identifiers (replaces handle with `«erased»`).
3. Video blobs are deleted from S3 with versioning hold cleared.
4. CDN cache purge for affected URLs (signed-URL TTL ensures expiry within 15 min absent purge; full-purge for explicit erasure).
5. Search index re-emits affected docs in redacted form.
6. Audit-chain notes the redaction event (NOT the redacted content).
7. Watch-time sessions by the subject are anonymised but aggregate tally retained.
8. Likes/comments/shares by the subject are anonymised but tally retained.
9. Caption tracks deleted.
10. SLA: 30 days from request per GDPR; faster where local law requires.
11. Retention-floor conflict: if pack retention floor (e.g., HIPAA 6y or DMCA 3y repeat-infringer) requires preservation, the redaction redacts identifiers only; body stays in audit-protected form with access bound to compliance-officer + four-eyes.

## Cross-Border Transfer

Forbidden by default. Allowed only with:

- Tenant SCC (Standard Contractual Clauses) on file for GDPR-scope tenants.
- Tenant-of-tenant consent for end-user data flowing under tenant's contract.
- pack-us-healthcare: BAA + HIPAA-eligible source + HIPAA-eligible target.
- ActivityPub federation (Professional-tier only): per-tenant opt-in + per-peer SCC where applicable.

Cross-border transfer register: `microservices/shorts/legal/transfer-register.md` (Slice B).

## Federation Egress Residency Rules

| Tier | Federation egress allowed? | Conditions |
|---|---|---|
| Personal-tier | **NEVER** | Compile-time invariant DCI-08; Cedar belt-and-suspenders forbid |
| Professional-tier | Opt-in per tenant; metadata-only | Tenant must (a) sign SCC for cross-border peers, (b) attest peer-allowlist, (c) accept that federation egress = cross-pack data flow per pack residency rules |
| pack-us-healthcare Professional-tier | OFF by default | HIPAA Safe Harbor; tenant may activate with BAA + per-peer attestation |
| pack-kr (KR PIPA Art. 28) | Cross-border requires user-consent | Per Art. 28; explicit consent recorded at tenant + per-user level |
| Free/Basic tier | Federation unavailable | Premium-only feature |

## Verification

- CI lane `oya-check-shorts-pack-residency` asserts every Helm release is pack-pinned via `commonLabels.oyatie/pack`.
- Periodic Postgres audit: no row carries `pack != cluster_pack`.
- Periodic S3 audit: no video blob lives in a bucket whose region != pack's region.
- Federation egress audit: every outbound activity carries source pack + target peer pack + tenant opt-in record; non-conforming activity blocked.
- DRM tier audit: per-tenant DRM-license issuance log validated against tier; non-tier-matching emit Sev-1.

## References

- ADR-0117.
- Parallel ADR-0135.
- ADR-SHORTS-0004 (DRM substrate + tenant-tier gating).
- `microservices/shorts/multi-region.md`.
- `microservices/shorts/policy/dual-context-isolation.md`.
- GDPR Arts. 44–50; KR PIPA Arts. 28, 23-2; HIPAA §164.502, §164.530; APPI Art. 27; LGPD Arts. 33, 46; ActivityPub W3C Rec 2018; DMCA Title II 17 USC §512.
