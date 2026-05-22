---
doc_class: PolicySpec
title: Data Residency Contract
microservice: meet
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-meet
deciders: council-privacy, ops-security, axis-meet, gtm-customer-success
related_adrs: [ADR-0117, ADR-0135, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/meet/threat-model.md (T-I-09 cross-pack misroute)
  - microservices/meet/dpia.md (R-09)
  - microservices/meet/policy/recording-consent.md
  - microservices/meet/multi-region.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (meet µservice)

## Purpose

Define which jurisdictions' meeting + recording + transcript + participant data live in which meet cluster, the cross-pack replication policy, and the legal-transfer mechanisms that gate any exception. Reviewed by EU DPAs (GDPR Arts. 44–50), KR PIPC (PIPA Art. 28 + Art. 23-2), HIPAA Covered Entity counsel (BAA), SEC/FINRA supervisors (pack-us-financial), and equivalent supervisory authorities per active pack.

## Residency Model

### Default: pack-pinning

Every tenant is assigned a primary pack at onboarding. Cross-pack movement is **forbidden by default**.

| Pack | Primary region(s) | Cluster footprint | Activated? |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-meet-{postgres,valkey,s3,meilisearch,livekit,coturn,whisper-gpu} | YES (M02 launch) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | eu-meet-* | Conditional (first EU SCC) |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 (DR pair) | us-meet-* | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | us-hc-meet-* (isolated from non-HC) | Conditional (post-BAA) |
| pack-us-financial | OCI us-ashburn-1 (FedRAMP-eligible) | us-fin-meet-* (isolated; WORM-Object-Lock active) | Conditional (SEC/FINRA gating) |
| pack-jp | OCI ap-tokyo-1 | jp-meet-* | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-meet-* | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-meet-* | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-meet-* | Conditional |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-meet-* | Conditional |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-meet-* | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-meet-* | Conditional |

### Cross-pack meeting attendance

A pack-eu user joining a pack-us tenant's meeting is allowed:

- User authenticates against their home-pack (pack-eu).
- Meet-rest issues LiveKit token scoped to the host-pack room (pack-us).
- Media flows pack-eu-edge → pack-us-SFU via inter-region SFU mesh.
- Recording (if enabled) lives in the **host-tenant pack** (pack-us) — recording residency follows host pack, not attendee.
- SCC required when host-tenant is in GDPR-scope and attendee is EU data subject.
- Per-attendee notice at join: "This meeting is hosted in pack-us; your participation records (audio/video/transcript) will be retained under host-tenant residency."

## Cross-Pack Replication Policy

### Default: forbidden

- Postgres logical replication: within-pack only.
- Valkey cluster replication: within-pack only.
- S3 recording + transcript cross-region replication: within-pack only.
- Meilisearch transcript index replication: within-pack only.

### Allowed exceptions

- DR-pair packs replicate primary → warm-standby within the pack.
- Audit-chain seals are cross-pack-replicable because they contain no PII (just commit-hash + signature).
- Public OpenAPI / AsyncAPI schemas are public.

### Disallowed everywhere

- Transcript index sharing across packs (rebuilt per-pack).
- Recording blob cross-pack replication (forbidden by default).
- LiveKit SFU media-plane cross-pack (allowed only via the inter-region SFU mesh for cross-pack attendance, NOT for replication).

## Retention Bounds Per Pack

| Pack | Recording retention floor | Recording retention ceiling | Transcript retention | Notes |
|---|---|---|---|---|
| pack-kr | 1 year (KR PIPA Art. 21 labor) | 5 years default; tenant-configurable up to 7y | inherits recording | KR PIPA + 전자문서법 |
| pack-eu | none (GDPR storage-limitation; tenant-defined) | 7y max (GDPR Art. 5(1)(e)) | inherits recording | GDPR; MiFID II for investment firms: 5-7y |
| pack-us | none | 7y max | inherits recording | varies state |
| pack-us-healthcare | 6 years (HIPAA §164.530(j)) | 10y max | inherits recording | HIPAA; BAA |
| pack-us-financial | 3 years (SEC 17a-4(b) immediate); 7 years total retention | 10y max | inherits recording | SEC 17a-4(f) WORM + FINRA 4511 |
| pack-jp | 2 years (APPI labor) | 7y max | inherits | APPI |
| pack-sg | 1 year | 7y max | inherits | PDPA |
| pack-au | 7 years (Privacy Act default) | 10y max | inherits | Privacy Act 1988 |
| pack-in | 3 years (DPDPA processing-purpose) | 7y max | inherits | DPDPA 2023 |
| pack-br | 5 years (LGPD purpose-limitation) | 7y max | inherits | LGPD |
| pack-ae | 1 year | 7y max | inherits | UAE PDPL |
| pack-ksa | 5 years | 10y max | inherits | PDPL + SAMA |

## Recording Egress Residency

Outbound RTMP egress (live-stream to YouTube/Twitch/Vimeo/custom CDN per ADR-MEET-0004): the destination is outside oyatie's pack residency by definition. Tenant attests at egress-start that:
- The destination is legally permitted to receive the content.
- All participants have been informed (KR PIPA Art. 15; GDPR Art. 13).
- Cross-border data flow is acceptable to all data subjects (SCC if required).

The outbound stream is not recorded by oyatie beyond what the tenant's own recording configuration captures (in-pack S3).

## DSR Cascade (right-to-erasure)

When a data subject exercises right-to-erasure for meet recordings/transcripts:

1. DSR cascade runner identifies all recordings + transcripts in which the subject participated (within the pack).
2. For each recording:
   - If retention floor allows: tombstone the recording manifest + delete the S3 blob.
   - If retention floor requires preservation (HIPAA 6y; SEC 17a-4 3-7y; MiFID II 5-7y; etc.): apply face-blur (video) + voice-mask (audio) for the requesting subject; keep body in access-restricted form bound to compliance-officer access only.
3. Transcript redaction: replace subject's name/handle with `«erased»`; retain content for retention floor compliance.
4. Search index re-emits affected transcripts in redacted form.
5. Audit-chain notes the redaction event (NOT the redacted content).
6. SLA: 30 days from request per GDPR; faster where local law requires.

Face-blur + voice-mask use deterministic open-weights models with on-prem GPU; no cloud-API.

## Cross-Border Transfer

Forbidden by default. Allowed only with:

- Tenant SCC (Standard Contractual Clauses) on file for GDPR-scope tenants.
- Tenant-of-tenant consent for end-user data flowing under tenant's contract.
- pack-us-healthcare: BAA + HIPAA-eligible source + HIPAA-eligible target.

Cross-border transfer register: `microservices/meet/legal/transfer-register.md`.

## Verification

- CI lane `oya-check-meet-pack-residency` asserts every Helm release is pack-pinned via `commonLabels.oyatie/pack`.
- Periodic Postgres audit: no row carries `pack != cluster_pack`.
- Periodic S3 audit: no recording object lives in a bucket whose region != pack's region.
- Periodic LiveKit room audit: room residency matches host-tenant pack.

## References

- ADR-0117.
- ADR-0135.
- ADR-MEET-0004 (egress policy).
- `microservices/meet/multi-region.md`.
- `microservices/meet/policy/recording-consent.md`.
- `microservices/messenger/policy/data-residency.md` (shape reference).
- GDPR Arts. 44–50; KR PIPA Arts. 28, 23-2; HIPAA §164.502, §164.530; SEC Rule 17a-4(f); FINRA Rule 4511; MiFID II Art. 16(7); APPI Art. 27; LGPD Arts. 33, 46.
