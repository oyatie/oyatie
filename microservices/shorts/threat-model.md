---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: shorts
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-shorts + ops-security
deciders: council-architecture, ops-security, axis-shorts, council-privacy, ops-legal
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + OWASP API Top 10 (2023) + NIST SP 800-154
related_adrs: [ADR-0008, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0126, ADR-0130, ADR-0131, ADR-0132, ADR-0140]
related_specs: [/specs/per-microservice-flat-layout.json]
review_cadence: quarterly + on every architecture or substrate change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC6.7, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.8.2, A.8.3, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.25, A.8.26, A.8.27, A.8.28"
  - "GDPR Arts. 5, 6, 7, 8, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35"
  - "EU DSA Arts. 14, 16, 17, 20, 23, 24, 27, 28 (transparency + content moderation + minor protection)"
  - "EU AI Act 2024/1689 Arts. 9, 13, 14, 15, 50 (high-risk + transparency)"
  - "EU AVMSD 2018/1808 (Audiovisual Media Services Directive — video-sharing-platform obligations)"
  - "OWASP API Top 10 (2023)"
  - "OWASP ASVS v4.0.3"
  - "DMCA Title II Safe Harbor (17 USC §512)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 15/17/18/22-2/23/23-2/24/25/28/29/29-2/33/34", "KR-ISMS-P §2.1-2.12", "KR 청소년 보호법 (Juvenile Protection Act)", "KR 정보통신망법 §49", "KR Telecommunications Business Act"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308-316 (only if patient-education videos)", "HITECH Act breach-notification", "US COPPA 15 USC §6501 (age-gate)"]
  pack-eu: ["GDPR Arts. 8 (child consent) + 25 + 32 + 35 + 44-50", "EU DSA 2065/2022 Arts. 14/16/17/20/24/27/28", "EU DMA where applicable", "EU AI Act 2024/1689", "EU AVMSD 2018/1808", "ePrivacy Directive 2002/58", "UK Online Safety Act 2023", "FR Audiovisual Code", "DE NetzDG"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24/26-2"]
  pack-sg: ["PDPA 2012 §11-26", "Online Safety (Miscellaneous Amendments) Act 2022"]
  pack-au: ["Privacy Act 1988 APP 1-13", "Online Safety Act 2021", "AU eSafety Commissioner BOSE 2022"]
  pack-in: ["DPDPA 2023 §6-10 + child-consent §9"]
  pack-br: ["LGPD Arts. 6/7/11/14/18/33/46/48", "Marco Civil da Internet"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021"]
  pack-ksa: ["PDPL Royal Decree M/19/2021", "SAMA Cybersecurity Framework 2017"]
  pack-us: ["DMCA Title II 17 USC §512", "US COPPA 15 USC §6501", "CA AB-2273 (CA Age-Appropriate Design Code Act)", "UT Social Media Regulation Act"]
doc_status: published
---

# Threat Model: shorts µservice

## Purpose

Identify, classify, and mitigate threats to shorts's confidentiality, integrity, availability, and privacy posture. The shorts µservice is the canonical first-party short-form-video platform across Personal (B2C) + Professional (B2B) contexts; a compromise leaks creator graphs, video corpora, watch-time behaviour, copyright fingerprint corpus, age-attestation + parental-link records (`SENSITIVE_CHILD_PROTECTION`), and DRM key material (`SECRET`). This document is reviewed by SOC 2 examiners, ISO 27001 auditors, GDPR DPAs, KR PIPC, HIPAA OCR (where applicable), EU DSA coordinators, EU AI Act notified bodies, EU AVMSD video-sharing-platform coordinators, UK Ofcom, AU eSafety Commissioner, and US Copyright Office (DMCA agent) at first-tenant onboarding per pack.

## Scope

### In-scope

All components introduced by parallel ADR-0126 (Connect dissolution → shorts µservice) and ADR-0132 (suite dissolution into shorts surface) for the shorts µservice. Deployed in the dedicated shorts Kubernetes cluster.

| Layer-A (adopted OSS / SaaS) | Layer-B (oyatie-owned) |
|---|---|
| Postgres 16 LTS (video metadata + upload-sessions + claims + ages + parental + analytics + audio-track-library) | `oya-shorts-video-upload-*` (10 crates) |
| Redis 7.2 (feed cache + watch-position + like-counters + trending + notification fanout) | `oya-shorts-video-transcode-*` (9 crates) |
| Meilisearch 0.10.0 (hashtag + sound + creator search) | `oya-shorts-video-storage-*` (8 crates) |
| S3-compatible (video blobs + transcode variants + thumbnails + quarantine) | `oya-shorts-thumbnail-generation-*` (8 crates) |
| CloudFront-class CDN (Cloudflare R2 + Workers; or OCI CDN) | `oya-shorts-audio-track-library-*` (7 crates) |
| ffmpeg 7.x LTS (transcode + thumbnail + composition) | `oya-shorts-audio-attribution-*` (6 crates) |
| Chromaprint (audio fingerprint) + perceptual-hash + DCT (video fingerprint) | `oya-shorts-video-composition-*` (8 crates) |
| OPSWAT MetaDefender 5.x / ClamAV 1.x (upload scan) | `oya-shorts-feed-timeline-*` (10 crates) |
| Widevine + FairPlay + PlayReady (DRM key systems) | `oya-shorts-watch-time-tracking-*` (8 crates) |
| Cedar v4.2 LTS policy evaluator | `oya-shorts-like-share-comment-*` (8 crates) |
| WebSocket gateway (Envoy + Cloudflare termination) | `oya-shorts-repost-stitch-duet-*` (8 crates) |
| ActivityPub gateway (optional federation; inherits social posture) | `oya-shorts-hashtag-*` (7 crates) |
| | `oya-shorts-trending-*` (8 crates) |
| | `oya-shorts-notifications-*` (10 crates) |
| | `oya-shorts-content-moderation-*` (8 crates) |
| | `oya-shorts-copyright-claim-*` (8 crates) |
| | `oya-shorts-age-gate-*` (7 crates) |
| | `oya-shorts-parental-controls-*` (7 crates) |
| | `oya-shorts-accessibility-captions-*` (8 crates) |
| | `oya-shorts-creator-analytics-*` (8 crates) |
| | `oya-shorts-monetization-stub-*` (5 crates; off by default) |
| | `oya-shorts-live-streaming-stub-*` (2 crates; off at M03) |
| | `oya-shorts-drm-*` (8 crates; tenant-tier gated) |

### Out-of-scope

- Threats to the underlying Kubernetes / hyperscaler — owned by `cloud-k8s`.
- Threats to OpenBao — owned by `cloud-secrets`.
- Threats to audit-chain µservice — owned by its own threat model; inherited.
- Threats to Ontology — owned by `ontology` µservice; inherited for sound + hashtag + mention resolution paths.
- Threats to social profile + follow-graph — owned by `social` µservice; shorts inherits via Workflow events.
- Threats to messenger share-to-DM bridge — owned by `messenger` µservice.
- Threats to GitHub Actions — owned by `governance`.
- Threats to foundry-runtime model deployment — inherited from `foundry-runtime` + `foundry-guardrails` threat models.

## Trust Boundaries

```text
┌─ Internet ────────────────────────────────────────────────────────────────┐
│                                                                           │
│   End-users (web/desktop/mobile)         Workflow Studio shell            │
│         │                                       │                         │
│         │ (TLS, WSS, OIDC, OAuth 2.1)           │ (mTLS internal)         │
│         ▼                                       ▼                         │
│  ┌─ Public ingress (Envoy/Cloudflare) ──────────────────────────────┐     │
│  │  TLS + WAF + DDoS + WebSocket upgrade + WebRTC media-relay      │     │
│  └──────────────────────────────────────────────────────────────────┘     │
│                              │                                            │
│  ┌─ CDN (Cloudflare R2 + Workers) ──────────────────────────────────┐     │
│  │  Signed-URL TTL ≤ 15min; per-fetch Cedar re-evaluation           │     │
│  └──────────────────────────────────────────────────────────────────┘     │
└──────────────────────────────│────────────────────────────────────────────┘
                               ▼
┌─ Dedicated shorts cluster ────────────────────────────────────────────────┐
│                                                                           │
│  TB1: External → Cluster ingress                                          │
│  TB2: WebSocket gateway → BC services (mTLS + SPIFFE)                     │
│  TB3: BC services → backing stores (per-tenant RLS + KMS)                 │
│  TB4: Personal/Professional context isolation (data-model invariant)      │
│  TB5: BC services → audit-chain (Ed25519)                                 │
│  TB6: BC services → ontology (Workflow event)                             │
│  TB7: Upload scan path (OPSWAT / ClamAV; quarantine bucket)               │
│  TB8: Transcode worker pool (gVisor / Kata; ffmpeg 7.x)                   │
│  TB9: Fingerprint matcher worker (Chromaprint + DCT perceptual-hash)      │
│  TB10: Federation egress (inherits social ADR-SOC-0004)                   │
│  TB11: Content-moderation classifier (foundry-runtime; EU AI Act HR)      │
│  TB12: Ranking model (foundry-runtime; EU AI Act HR)                      │
│  TB13: DRM key system (Widevine / FairPlay / PlayReady) — Premium-tier   │
│  TB14: Auto-caption ASR (foundry-runtime; T1 capability)                  │
│  TB15: Minor-protection isolation (age-attestation + parental-link)       │
│                                                                           │
└───────────────────────────────────────────────────────────────────────────┘
```

Fifteen trust boundaries:
1. **External → Cluster ingress** (TLS, WAF, DDoS, WebSocket upgrade).
2. **Gateway → BC services** (mTLS + SPIFFE identity).
3. **BC services → backing stores** (RLS + per-tenant prefix isolation + KMS-SSE).
4. **Personal/Professional context isolation** (data-model invariant per parallel ADR-0126).
5. **BC services → audit-chain** (Ed25519 seal).
6. **BC services → ontology** (Workflow event bus).
7. **Upload scan path** (untrusted blob → scanner → quarantine vs production).
8. **Transcode worker isolation** (ffmpeg 7.x sandboxed via gVisor / Kata Container; CVE-prone).
9. **Fingerprint matcher** (Chromaprint + DCT against `INTERNAL_ONLY` corpus).
10. **Federation egress / ingress** (inherits social ADR-SOC-0004; Personal-tier forbidden).
11. **Content-moderation classifier** (EU AI Act high-risk; transparency).
12. **Ranking model** (EU AI Act high-risk; transparency).
13. **DRM key system** (Widevine / FairPlay / PlayReady; tenant-tier gated; `SECRET` key material).
14. **Auto-caption ASR** (foundry-runtime T1; transient processing of audio).
15. **Minor-protection isolation** (age-attestation table + parental-link table; `SENSITIVE_CHILD_PROTECTION`).

## Assets & Data Classification

| Asset | Class | Retention | Authoritative store |
|---|---|---|---|
| Video metadata (creator_ref, caption, posted_at, visibility, hashtags, sound_ref) | `BEHAVIORAL_TENANT_PRODUCT` + sometimes `PII_IDENTIFYING` (caption may name people) | per-pack | Postgres (tenant-DEK encrypted when Professional configured) |
| Video blob (original) | `BEHAVIORAL_TENANT_PRODUCT` + sometimes `PII_IDENTIFYING` (faces) | per-pack; archive tier after 30d | S3 (KMS-encrypted; per-tenant prefix) |
| Video transcode variants (HLS + DASH segments per bitrate) | `BEHAVIORAL_TENANT_PRODUCT` + derived | rebuilt from original | S3 + CDN |
| Thumbnails (poster JPEG + animated GIF + WebP) | `BEHAVIORAL_TENANT_PRODUCT` + derived | rebuilt | S3 + CDN |
| Captions (auto + manual) | `BEHAVIORAL_TENANT_PRODUCT` | per-video | S3 + Postgres |
| Audio-track-library (licensed + UGC) | `BEHAVIORAL_TENANT_PRODUCT` + `AUDIT` (licensing metadata) | append-only history | Postgres + S3 |
| Audio attribution (per-video → sound rights chain) | `AUDIT` + `BEHAVIORAL_TENANT_PRODUCT` | append-only | Postgres |
| Watch-time sessions (per (viewer, video) seconds + completion-ratio) | `BEHAVIORAL_TENANT_PRODUCT` + `PII_QUASI_IDENTIFIER` (when joined to viewer_ref) | 90d hot, aggregated permanently | Postgres + Redis |
| Like / share / comment | `BEHAVIORAL_TENANT_PRODUCT` | 365d hot | Postgres + Redis |
| Repost-stitch / repost-duet records | `BEHAVIORAL_TENANT_PRODUCT` + rights chain | append-only | Postgres |
| Notification records | `BEHAVIORAL_TENANT_PRODUCT` + `PII_IDENTIFYING` | 90d hot | Postgres + Redis |
| Trending windows + sound-of-the-week | derived; `INTERNAL_ONLY` | rebuilt continuously | Redis |
| Search index | derived from videos + sounds + creators + hashtags | rebuilt from source | Meilisearch (per-tenant index) |
| Moderation verdicts + appeal trail | `AUDIT` + `INTERNAL_ONLY` + `PII_IDENTIFYING` (reporter_ref) | append-only; immutable | Postgres + audit-chain seal |
| **Copyright fingerprint corpus** | `INTERNAL_ONLY` + `AUDIT` | append-only with licensor-controlled lifecycle | Postgres + per-pack |
| **Copyright claims + counter-notices + repeat-infringer records** | `AUDIT` + `PII_IDENTIFYING` (claimant + creator refs) | append-only; per DMCA retention floor (3y default; pack-us 6y) | Postgres + audit-chain seal |
| **Age attestations + parental-link records** | `PII_QUASI_IDENTIFIER` + `SENSITIVE_CHILD_PROTECTION` (when minor) | per-pack | Postgres (separated table; restricted access) |
| Creator-analytics aggregates | derived; `INTERNAL_ONLY` for cluster, `BEHAVIORAL_TENANT_PRODUCT` exposed to creator | rebuilt | Postgres + Redis |
| Audit-chain seals (every state transition) | `AUDIT` | append-only; immutable | audit-chain µservice |
| Per-tenant DEK | `SECRET` | OpenBao 30d rotation; envelope KMS | OpenBao |
| **DRM per-content keys** | `SECRET` | per-content rotation 7d; replaceable | OpenBao + DRM key system |
| WebSocket gateway session tokens | `SECRET` | ≤ 24h | OpenBao-issued short-lived JWT |
| Federation peer keys + signatures | `SECRET` (private side) / `PUBLIC` (public side) | rotation 90d | OpenBao |
| Ranking-model snapshots + training data | `INTERNAL_ONLY` + `AUDIT` (model card + eval record) | per release | foundry-runtime evidence pipeline |
| Content-moderation classifier model card | `AUDIT` + `INTERNAL_ONLY` | per release | foundry-runtime + audit-chain |

## Actors

| Actor | Trust | Auth | Capability |
|---|---|---|---|
| Creator (human) | Untrusted external | OIDC + MFA + WSS bearer | Upload own video; edit caption; delete own video; receive analytics |
| Viewer (human) | Untrusted external | OIDC + MFA (or guest with restricted view) | Watch public videos; react; comment; share; report |
| Minor viewer (human < pack-threshold) | Untrusted external + heightened-protection | OIDC + parental-consent attestation | Watch via age-restricted feed; chronological-only default; no algorithmic-recommendation; no DM share |
| Tenant-admin | Semi-trusted internal-to-tenant | OIDC + MFA | Manage tenant config; verification policy; cannot read PII videos without four-eyes |
| Tenant compliance-officer | Semi-trusted internal-to-tenant | OIDC + MFA + Cedar entitlement | Issue eDiscovery hold (Professional only); execute DMCA takedown; trigger disclosure (four-eyes peer) |
| Tenant moderator | Semi-trusted internal-to-tenant | OIDC + MFA + Cedar entitlement | Manual review of flagged content; verdicts + appeals |
| Tenant security-admin | Semi-trusted internal-to-tenant | OIDC + MFA + Cedar entitlement | Configure pack policy; four-eyes pairing peer |
| Rights-holder / DMCA agent (external) | Untrusted external + verified-business | OIDC + MFA + business-verification | File copyright-claim; counter-notice ineligible from this actor |
| oyatie ops-security (human) | Trusted internal | OIDC + MFA + JIT via OpenBao | Admin access; no plaintext PII without breakglass + two-person rule |
| oyatie ops-legal (DMCA designated agent) | Trusted internal | OIDC + MFA + business-of-record | Repeat-infringer policy execution; designated agent of record |
| Workflow Studio (machine) | Semi-trusted internal | mTLS + SPIFFE | Consume Workflow events; emit action-cards |
| social µservice (machine) | Semi-trusted internal | mTLS + SPIFFE | Cross-link short-video deep-link; mention bridge |
| messenger µservice (machine) | Semi-trusted internal | mTLS + SPIFFE | Share-to-DM bridge |
| ontology µservice (machine) | Semi-trusted internal | mTLS + SPIFFE | Serve Sound / Hashtag / Sticker / Person lookups |
| audit-chain µservice (machine) | Trusted internal | mTLS + SPIFFE | Receive seals from every BC |
| foundry-runtime µservice (machine) | Semi-trusted internal | mTLS + SPIFFE | Provide classifier + ranking + ASR inference; high-risk per EU AI Act |
| ActivityPub federation peer (external) | Untrusted external | HTTP Signatures (RFC 9421) + peer allowlist | Receive Professional-tier video metadata (federation is metadata-only; blob never crosses pack) |
| External auditor | Read-only external | OIDC + MFA + JIT short-lived token | Read tenant-scoped audit-chain seals + policy artifacts |
| Attacker — opportunistic | Untrusted | none | Scans + low-skill exploitation |
| Attacker — targeted | Untrusted | none | Sophisticated supply-chain awareness; influence-op + copyright-grift attempts |
| Attacker — DRM-pirate (screen-scrape / EME-bypass) | Untrusted | varies | Attempts to extract content keys or rip transcode segments |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure retention or moderation |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case for confidentiality / integrity; mitigated by audit-chain + four-eyes |

## STRIDE Threat Catalog

### Spoofing (S)

**T-S-01 — Viewer-B impersonates Creator-A via WebSocket session hijack**
- Asset: WebSocket session
- Likelihood M / Impact H / Risk **H**
- Mitigations: per-connection short-lived JWT bound to device + IP; WSS only; rotation 24h; OIDC re-auth on token expiry; anomaly detection on geo-shift mid-session.
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.5; GDPR Art. 32(1)(a)(b)

**T-S-02 — Handle squatting (attacker registers handle of trademark / verified creator)**
- Asset: Creator handle namespace (inherited from social)
- Likelihood M / Impact M / Risk **M**
- Mitigations: handle is owned by social µservice; shorts inherits; verification-badge program; trademark-reservation onboarding option for Professional tenants.

**T-S-03 — Forged DMCA copyright-claim (attacker files claim against innocent creator)**
- Asset: DMCA claim filing
- Likelihood H / Impact H / Risk **H**
- Mitigations: claimant business-verification + perjury-attestation in claim text (DMCA §512(c)(3)(A)(vi)); counter-notice workflow + repeat-infringer policy applied to false-claimants; rate-limit per claimant; audit-chain seal per claim; abuse pattern detection.
- Frameworks: DMCA Title II 17 USC §512(f); EU DSA Art. 16

**T-S-04 — Sybil-coordinated like / repost-stitch farm**
- Asset: Engagement signals (likes, shares, reposts) → ranking integrity
- Likelihood H / Impact M / Risk **H**
- Mitigations: foundry-guardrails sybil detector; per-author influence cap; tenant-admin can require email/phone verification.

**T-S-05 — Tenant-admin impersonates compliance-officer to trigger Professional disclosure single-handed**
- Asset: Four-eyes disclosure path
- Likelihood L / Impact H / Risk **M**
- Mitigations: four-eyes requires two distinct SPIFFE identities with distinct entitlements + audit-chain seal of both consents; same principal cannot satisfy both halves; replay-resistant nonce.

**T-S-06 — Rights-holder counter-notice forgery (attacker counter-notices as creator)**
- Asset: DMCA counter-notice workflow
- Likelihood M / Impact M / Risk **M**
- Mitigations: counter-notice requires OIDC + MFA-bound identity matching the creator-ref of the affected video; audit-chain seal; oyatie cannot accept counter-notice from any other principal.

### Tampering (T)

**T-T-01 — Video blob tampering in S3**
- Asset: Video blobs
- Likelihood L / Impact H / Risk **M**
- Mitigations: SSE-KMS + S3 Object Lock (WORM) on Professional-tier; content-digest verified at fetch; tamper triggers quarantine; bucket access via service-account IAM only.

**T-T-02 — Transcode manifest tampering**
- Asset: HLS/DASH manifests pointing to wrong segments
- Likelihood L / Impact H / Risk **M**
- Mitigations: manifests signed via Ed25519 at write time; CDN verifies signature; mismatched signature → 403.

**T-T-03 — Fingerprint corpus poisoning**
- Asset: Copyright fingerprint corpus
- Likelihood M / Impact H / Risk **H**
- Mitigations: only ops-legal can write to corpus via signed manifest workflow; per-licensor namespace isolation; audit-chain seal per write; periodic licensor reconciliation.

**T-T-04 — Watch-time tampering (artificial watch inflation)**
- Asset: Watch-time signals → ranking integrity + creator-analytics
- Likelihood M / Impact M / Risk **M**
- Mitigations: per-(viewer, video) idempotent watch-session; scroll-velocity sanity-check; foundry-guardrails sybil detector; suspicious patterns capped + flagged.

**T-T-05 — Like-counter tampering (vote-stuffing)**
- Asset: Like tallies
- Likelihood M / Impact M / Risk **M**
- Mitigations: per-user-per-video idempotency; conflict-free counter; periodic reconciliation Postgres ↔ Redis.

**T-T-06 — Moderation classifier verdict tampering**
- Asset: Moderation verdict ledger
- Likelihood L / Impact H / Risk **M**
- Mitigations: every classifier verdict signed by foundry-runtime + sealed via audit-chain Ed25519; verdict-replay detector compares Postgres state vs audit-chain.

**T-T-07 — Trending sound poisoning**
- Asset: Trending sound ranking
- Likelihood M / Impact M / Risk **M**
- Mitigations: trend compute uses windowed dedup keyed by `(tenant_id, sound_id, author_ref)`; per-author influence cap; foundry-guardrails sybil detector; tenant-admin pin/unpin; periodic audit.

**T-T-08 — Caption injection (creator embeds malicious WebVTT)**
- Asset: Caption integrity
- Likelihood L / Impact M / Risk **L**
- Mitigations: WebVTT + TTML parsed via strict validator; HTML stripped; size-bounded; foundry-guardrails caption-injection detector for adversarial Unicode.

### Repudiation (R)

**T-R-01 — Creator denies uploading a video**
- Asset: Upload authorship
- Likelihood M / Impact M / Risk **M**
- Mitigations: every upload carries creator SPIFFE identity + session-token nonce + audit-chain seal; client-side device-key signing where available.

**T-R-02 — Tenant-admin denies authorising disclosure**
- Asset: Four-eyes disclosure record
- Likelihood L / Impact H / Risk **M**
- Mitigations: four-eyes requires both consents in audit-chain with distinct principal IDs + reason code; non-repudiable.

**T-R-03 — DMCA claimant denies filing claim (later disputes terms)**
- Asset: DMCA claim filing record
- Likelihood L / Impact H / Risk **M**
- Mitigations: claim filing requires perjury-attestation per DMCA §512(c)(3)(A)(vi); audit-chain Ed25519 seal of claim text + claimant identity + timestamp; non-repudiable.

**T-R-04 — Moderator denies issuing a verdict**
- Asset: Moderation verdict record
- Likelihood L / Impact M / Risk **L**
- Mitigations: verdict emits Ed25519-signed audit-chain record with reviewer principal + timestamp + scope.

### Information Disclosure (I)

**T-I-01 — Cross-tenant video leak via Postgres RLS misconfiguration**
- Asset: Video store
- Likelihood M / Impact H / Risk **H**
- Mitigations: Postgres Row-Level Security with `tenant_id = current_setting('app.tenant_id')`; gateway sets the GUC per connection; LEAN check `oya-check-postgres-rls-coverage` asserts RLS enabled on every shorts table; pen-test annually.

**T-I-02 — PHI leak in video (pack-us-healthcare patient-education edge case)**
- Asset: Video body + audio
- Likelihood L / Impact H / Risk **M**
- Mitigations: pack-us-healthcare default disables auto-caption + auto-moderation on patient-ed channels; PHI-redactor scans caption output; HIPAA Safe Harbor §164.514 honoured; minimum-necessary §164.502(b).

**T-I-03 — Personal-tier video leaked via tenant-admin pivot**
- Asset: Personal-tier video + watch-time
- Likelihood M / Impact H / Risk **H**
- Mitigations: Cedar `policy/tenant-scope.cedar` blocks tenant-admin reads of Personal-context resources; LEAN lane verifies; cross-context disclosure attempts emit `shorts_personal_admin_decrypt_attempt_total` (target=0).

**T-I-04 — Watch-time pivoting reveals viewer behavioural profile (sensitive under GDPR Art. 9 in some contexts)**
- Asset: Watch-time data
- Likelihood M / Impact M / Risk **M**
- Mitigations: watch-time reads bounded by Cedar (own-watch + aggregate-creator-analytics only); cross-tenant enumeration forbidden; per-tenant cardinality limits; aggregate-only for non-owners.

**T-I-05 — Search-result leak: returns videos viewer cannot read**
- Asset: Search results
- Likelihood M / Impact H / Risk **H**
- Mitigations: search post-filters by Cedar evaluation; result set redacted to caller-scope; integration test asserts no over-permitted result.

**T-I-06 — Video CDN URL leak via shared-link guess**
- Asset: Video CDN URL
- Likelihood M / Impact H / Risk **H**
- Mitigations: video URLs are signed short-TTL (≤ 15 min); per-fetch Cedar re-evaluation; DRM-protected content additionally requires EME license bound to viewer device.

**T-I-07 — Cross-context routing (Personal short → Professional feed)**
- Asset: Dual-context isolation
- Likelihood L / Impact H (regulatory + privacy breach) / Risk **H**
- Mitigations: data-model invariant — `PersonalShort` and `ProfessionalShort` are distinct entity types; cross-type write rejected by domain layer; LEAN-lane `oya-check-dual-context-isolation` validates type signatures forbid cross-context flows.

**T-I-08 — Federation egress leak (Personal-tier video escapes via ActivityPub)**
- Asset: Personal-tier confidentiality
- Likelihood L / Impact H / Risk **H**
- Mitigations: inherits social ADR-SOC-0004 + DCI-08 — `federation-gateway` rejects any Personal-tier post at the egress port (compile-time type signature); federation is metadata-only (no blob crosses pack boundary).

**T-I-09 — Minor-account discovery (attacker enumerates minors via age-attestation table pivot)**
- Asset: Age-attestation table + parental-link table
- Likelihood L / Impact H (child-protection) / Risk **H**
- Mitigations: separate `shorts_age_attestations` + `shorts_parental_links` tables; access bound to Cedar `age_verification_reader` entitlement (rare; only minor-protection compliance flows); no general staff read; encryption at rest; LEAN lane verifies isolation.

**T-I-10 — DRM key material leak**
- Asset: Per-content DRM keys
- Likelihood L / Impact H / Risk **H**
- Mitigations: keys stored in OpenBao with key-system-specific HSM (Widevine SecureStop; FairPlay key-server; PlayReady DRM-server); per-content key rotation 7d; license issuance bound to device-key attestation.

**T-I-11 — Auto-caption ASR leak (caption is server-emitted; viewer sees content not in source)**
- Asset: Caption integrity
- Likelihood L / Impact M / Risk **L**
- Mitigations: ASR runs server-side; output goes only into caption track for the same video; foundry-runtime evidence record per ASR call; EU AI Act Art. 50 transparency label on auto-captions.

**T-I-12 — Creator-analytics aggregate enables re-identification of individual viewers**
- Asset: Creator-analytics dashboards
- Likelihood M / Impact M / Risk **M**
- Mitigations: k-anonymity ≥ 10 on demographic slices; no per-viewer drill-down for creator; only aggregate counts; suppress slices with < 10 viewers.

### Denial of Service (D)

**T-D-01 — Feed-load storm: viral video causes mass concurrent feed-pulls**
- Asset: Feed cache + CDN
- Likelihood H / Impact H / Risk **H**
- Mitigations: Redis hot-feed cache; precomputed For-You for hot accounts; per-tenant feed-load rate limit; HPA on REST pods; CDN edge cache absorbs blob fetches; runbook `runbooks/feed-cache-rebuild.md`.

**T-D-02 — Transcode queue backup (mass-upload abuse or unexpected celebrity event)**
- Asset: ffmpeg worker pool
- Likelihood M / Impact H / Risk **H**
- Mitigations: per-tenant upload rate limit; queue depth bound; worker pool KEDA-style autoscale; priority lane for paid-tier tenants; runbook `runbooks/transcode-queue-backup.md`.

**T-D-03 — Copyright-claim storm (rights-holder bulk-files 10k claims)**
- Asset: Copyright-claim worker queue
- Likelihood M / Impact H / Risk **H**
- Mitigations: per-claimant rate limit (default 100/hr; verified-business 1000/hr); priority queue; rate-limit-throttle runbook; runbook `runbooks/copyright-claim-storm-throttle.md`.

**T-D-04 — CDN cache invalidation cascade (mass-takedown triggers global purge storm)**
- Asset: CDN POPs
- Likelihood M / Impact H / Risk **M**
- Mitigations: cache-key-based invalidation; sub-second TTL update primary mechanism; full-purge reserved for emergencies; runbook `runbooks/cdn-cache-invalidation-cascade.md`.

**T-D-05 — Notification fanout storm (creator with 1M followers posts)**
- Asset: notification worker queue
- Likelihood H / Impact M / Risk **M**
- Mitigations: sharded notification workers; per-recipient idempotent processing; coalesce digest for low-priority notifications; backpressure-throttle at hot-window.

**T-D-06 — Search-index lag during ingest spike**
- Asset: Meilisearch index
- Likelihood M / Impact M / Risk **M**
- Mitigations: backpressure on indexer; live-fallback to Postgres ILIKE-search; runbook in failure-modes pointer.

**T-D-07 — ffmpeg / Chromaprint CVE allows RCE in worker**
- Asset: Transcode + fingerprint worker pool
- Likelihood M / Impact H / Risk **H**
- Mitigations: workers run in gVisor / Kata Container sandbox; non-root; read-only root FS; no network egress except to S3 quarantine + production; LTS pin (ffmpeg 7.x); weekly CVE scan via Trivy + Grype.

**T-D-08 — DRM-license issuance storm (legitimate burst overwhelms key-server)**
- Asset: DRM key system
- Likelihood L / Impact M / Risk **M**
- Mitigations: per-tenant rate limit on license requests; key-server HA cluster; runbook `runbooks/drm-key-rotation.md`.

### Elevation of Privilege (E)

**T-E-01 — Cedar policy bug grants moderator entitlement to non-moderator**
- Asset: Cedar evaluator
- Likelihood L / Impact H / Risk **M**
- Mitigations: Cedar v4.2 LTS; fragment fuzz; integration test asserts no over-permitted action; periodic Cedar-fragment-coverage CI lane.

**T-E-02 — Compromised tenant-moderator pivots to read all tenant videos**
- Asset: Moderator scope
- Likelihood L / Impact H / Risk **M**
- Mitigations: moderator scope bounded to flagged-only reads (must come from `AbuseReportFiled` or classifier verdict); cannot read arbitrary videos; LEAN lane verifies moderator scope policy.

**T-E-03 — DMCA designated-agent abuse (insider executes mass takedown)**
- Asset: DMCA designated-agent path
- Likelihood L / Impact H / Risk **M**
- Mitigations: DMCA takedown action requires Cedar entitlement + audit-chain seal + per-action approval chain; insider abuse audited weekly by ops-legal.

**T-E-04 — Media scanner bypasses scan path**
- Asset: Quarantine boundary
- Likelihood L / Impact H / Risk **M**
- Mitigations: blob lifecycle: PUT → quarantine bucket → scanner → on-clean copy to production bucket; production bucket write-only by scanner SA.

**T-E-05 — ffmpeg CVE allows RCE in transcode worker**
- Asset: Transcode worker pool
- Likelihood M / Impact H / Risk **H**
- Mitigations: transcode workers run in gVisor / Kata Container sandbox; non-root; read-only root FS; no network egress except to S3 quarantine + production; LTS pin (ffmpeg 7.x); weekly CVE scan via Trivy + Grype.

**T-E-06 — DRM key-system substrate compromise (Widevine / FairPlay / PlayReady root key leak)**
- Asset: DRM key root
- Likelihood VL / Impact VH / Risk **H** (very high impact even at very low likelihood)
- Mitigations: key root never leaves OpenBao HSM; per-content keys derive from root with non-extractable wrapper; key-system rotation 90d; immediate revocation list on suspicion.

## LINDDUN Privacy-Threat Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | Watch-time across videos | Aggregate watch-time across videos forms a behavioural profile correlating viewer interests | Per-tenant scope; Cedar evaluation; never cross-tenant linkable; per-viewer opt-out of behavioural ranking; minor accounts no behavioural profile | L |
| T-L-02 | Identifiability | Creator handle + display-name | Combination identifies a creator across services | Per-tenant handle namespace (inherited from social); pseudonymous handle option for Personal-tier | L |
| T-L-03 | Non-repudiation | Video authorship | Creator cannot deny authoring a public video since session-token signs | Acceptable per GDPR Art. 5(2); explicit in onboarding notice | L |
| T-L-04 | Detectability | Posting time + watch-time | Timestamps reveal activity rhythm | Acceptable; public-by-default semantics; covered by tenant onboarding consent | M |
| T-L-05 | Disclosure | Compliance hold reveals Professional video bodies to admin | Hold + four-eyes disclosure inherently exposes professional video bodies to tenant admin | Mitigated to acceptable: four-eyes + audit-chain + reason code + tenant disclosure obligation | M |
| T-L-06 | Unawareness | End-user (tenant's user) | End-user may not know tenant-admin can disclose Professional-tier videos under four-eyes | Tenant DPA includes disclosure clause; tenant onboarding notice required | M |
| T-L-07 | Non-compliance | GDPR Art. 17 right-to-erasure | User requests erasure across all videos + comments + reactions + watch-history + follows | DSR cascade marks rows tombstoned + redacts identifiers; CDN purge; 30d SLA | M |
| T-L-08 | Linkability | Sound-of-the-week derivation | A creator's sound usage forms a profile of artistic identity | Acceptable; public-by-default semantics; covered by onboarding | L |
| T-L-09 | Identifiability | Verification badge | Verification badge ties a real-world identity to a handle | Acceptable; intentionally disclosive; only with user's verification request | L |
| T-L-10 | Disclosure | EU AI Act Art. 50 transparency for moderation + ranking + auto-caption | Algorithmic ranking + moderation classifier + ASR output must be disclosed to user | Every classifier + ASR output carries `eu_ai_act_label`; ranking explanation API per Art. 27 + 50 | L |
| T-L-11 | Non-compliance | GDPR Art. 8 child consent + KR PIPA Art. 8 + COPPA + CA AB-2273 + UT SMRA | Minor signup requires age-attestation + parental consent | `age-gate` BC routes minor accounts through parental-consent flow; `parental-controls` BC supervises; per-pack overlay | L |
| T-L-12 | Disclosure | Federation egress (Personal-tier leak attempt) | Personal-tier video accidentally federates | Inherits social DCI-08 (compile-time type-system invariant); LEAN lane verifies | L |
| T-L-13 | Non-compliance | DMCA repeat-infringer policy | Repeat-infringer policy not consistently enforced | `copyright-claim` BC tracks per-creator strike count; auto-suspend on 3+ confirmed claims within 6mo; per ops-legal policy | L |
| T-L-14 | Disclosure | Creator-analytics may reveal individual viewers if k-anonymity too low | Aggregate slices with < 10 viewers could re-identify | k-anonymity ≥ 10 floor; suppress slices below threshold; no per-viewer drill-down | L |
| T-L-15 | Disclosure | DRM device-binding leaks device fingerprint to creator-analytics | EME-bound license could expose device-id | Device-id hashed before any aggregate emission; per-tenant salt | L |

## Mitigations Catalog

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Postgres RLS on every shorts table | Preventive | axis-shorts | `oya-check-postgres-rls-coverage` lane |
| Per-connection short-lived JWT bound to device + IP | Preventive | axis-shorts | gateway audit log |
| Four-eyes disclosure with distinct principal IDs | Preventive | axis-shorts + ops-security | integration test |
| Cedar policy on every read/write | Preventive | axis-shorts | LEAN coverage lane |
| Media scan + quarantine workflow | Preventive | axis-shorts | end-to-end test |
| Personal-tier never-federates type invariant | Preventive | axis-shorts (inherits social) | `oya-check-federation-personal-tier-refused` lane |
| Audit-chain Ed25519 seal on every state transition | Detective + Non-repudiation | audit-chain | regression tests |
| Cross-context type-system invariant (PersonalShort ≠ ProfessionalShort) | Preventive | axis-shorts | `oya-check-dual-context-isolation` lane |
| Per-tenant rate + cardinality limits | Preventive (DoS) | axis-shorts | gateway + Postgres metrics |
| DSR cascade for right-to-erasure | Preventive (compliance) | council-privacy | DSR dashboard SLO |
| EU AI Act high-risk transparency record per classifier + ASR call | Preventive (compliance) | axis-shorts + axis-foundry-runtime | `capabilities/T2-auto.yaml` evidence pipeline |
| Age-gate routing at signup (pack-aware) | Preventive (compliance) | axis-shorts + council-privacy | `age-gate` BC unit + integration tests |
| Parental-link supervision (minor protection) | Preventive (compliance) | axis-shorts + council-privacy | `parental-controls` BC unit + integration tests |
| Copyright fingerprint pre-check at ingest | Preventive (compliance) | axis-shorts + ops-legal | `copyright-claim` BC integration test |
| DMCA takedown + counter-notice + repeat-infringer workflow | Preventive (compliance) | axis-shorts + ops-legal | DMCA cycle E2E test |
| Trending-sound sybil detector | Preventive | axis-shorts + axis-foundry-guardrails | trending-poisoning runbook drill |
| ffmpeg + Chromaprint sandboxed workers (gVisor / Kata) | Preventive | axis-shorts + cloud-k8s | IaC enforces; LEAN lane verifies |
| DRM per-content key rotation + HSM-bound root | Preventive | axis-shorts + cloud-secrets | `runbooks/drm-key-rotation.md` drill |
| Auto-caption `eu_ai_act_label` on emission | Preventive (compliance) | axis-shorts | `capabilities/T1-assist.yaml` evidence pipeline |
| Minor-account default = chronological + no algorithmic-recommendation + DM-restricted | Preventive | axis-shorts | `tests/e2e/minor-account-defaults.rs` |
| k-anonymity ≥ 10 in creator-analytics | Preventive | axis-shorts | integration test |

## Residual Risk Acceptance

| Risk ID | Residual | Why | Re-review |
|---|---|---|---|
| T-I-02 (PHI in patient-ed videos) | L–M | pack-us-healthcare auto-caption + auto-moderation disabled by default; opt-in with BAA | Quarterly |
| T-L-04 (timing detectability) | M | Public-by-default semantics; consent at onboarding | Annually |
| T-L-05 (hold disclosure inherent) | M | Four-eyes + audit are the load-bearing control; user-side opacity unavoidable | Annually |
| T-L-06 (end-user unawareness) | M | Joint-controllership clause | Annually |
| T-L-07 (erasure best-effort) | M | Retention bounds + audit immutability tradeoff | Annually |
| T-D-04 (CDN purge cascade) | L | Cache-key invalidation primary; full-purge rare | Quarterly |
| T-E-06 (DRM root compromise) | L | HSM-bound + rotation 90d; immediate revocation | Annually |
| T-S-03 (forged DMCA claim) | M | Inherent in DMCA filing process; perjury-attestation + counter-notice are the controls | Quarterly |

Sign-off:
- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`
- ops-legal: `pending`

## Per-Pack Overlay Sections

### pack-kr

- KR PIPA Art. 23 sensitive personal info — sensitive videos (medical, juvenile, biometric) require additional consent at signup.
- KR 청소년 보호법 (Juvenile Protection Act) — minor (< 14y) protection routing in `age-gate` + `parental-controls`.
- KR 정보통신망법 §49 (intercept) — admin reads only via four-eyes; covered.
- KR Telecommunications Business Act — video-sharing-platform obligations; covered.
- KR-ISMS-P §2.7 — access control via Cedar.

### pack-us-healthcare

- HIPAA §164.312(a)(1) — access control via Cedar + RLS (only if patient-education videos used).
- HIPAA §164.312(b) — audit-chain ≥ 6y retention overlay.
- HIPAA §164.502(b) — minimum-necessary: auto-caption + auto-moderation disabled by default.
- HIPAA §164.514 — de-identification; PHI redactor.
- US COPPA 15 USC §6501 — age-gate at signup; child-account flow (< 13y banned or COPPA-compliant).
- HIPAA §164.314 (Business Associate) — per-tenant BAA at `microservices/shorts/legal/baa-template.md` (Slice B).
- CA AB-2273 (CA Age-Appropriate Design Code Act) — minor protection.
- UT Social Media Regulation Act — parental consent for minors.

### pack-eu

- GDPR Art. 8 — child consent (16y default; member states may lower to 13y).
- GDPR Art. 25 — privacy-by-design via cross-context invariant + age-gate + parental-controls.
- GDPR Art. 32 — every mitigation above contributes.
- GDPR Arts. 44-50 — pack-eu data stays in EU pack; federation requires SCC.
- EU DSA Arts. 14, 16, 17, 20, 23, 24, 27, 28 — transparency reports + content moderation + appeal workflow + Statement of Reasons + recommender transparency + minor protection.
- EU AI Act 2024/1689 — ranking + moderation + ASR classifiers are high-risk; obligations satisfied per Arts. 9-15 + Art. 50 transparency.
- EU AVMSD 2018/1808 — video-sharing-platform obligations (Art. 28b); minor protection; commercial communication transparency.
- UK Online Safety Act 2023 — Ofcom illegal-content duty; safety-by-design (where UK tenant).
- FR Audiovisual Code — video-sharing-platform French overlay.
- DE NetzDG — 24h takedown for manifestly illegal content (DE-located tenant).
- ePrivacy Directive Art. 5(3) — confidentiality of communications.

### pack-us

- DMCA Title II 17 USC §512 — Safe Harbor; designated agent; counter-notice; repeat-infringer policy.
- US COPPA 15 USC §6501 — child-account flow.
- CA AB-2273 — CA Age-Appropriate Design Code Act.
- UT Social Media Regulation Act — parental consent for minors.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per pack overlay at `regional-packs/<pack>/shorts-overlay.md`; cross-mapped via compliance.md.

## Compliance Cross-Mapping

| Framework | Coverage | Mapping doc |
|---|---|---|
| SOC 2 Type 2 | CC1–CC9 covered in `compliance.md` |
| ISO 27001:2022 | A.5–A.8 covered |
| GDPR | Arts. 5, 6, 8, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35 covered in `dpia.md` + `compliance.md` |
| EU DSA | Arts. 14, 16, 17, 20, 23, 24, 27, 28 covered in `compliance.md` |
| EU AI Act | Arts. 9, 13, 14, 15, 50 covered via `capabilities/T2-auto.yaml` + ADR-SHORTS-0003 |
| EU AVMSD | Art. 28b covered in `compliance.md` |
| DMCA Title II | §512(c)(3), §512(g), §512(i)(1)(A) covered in `compliance.md` |

## Re-review Triggers

- Any new BC.
- Any change to dual-context invariant.
- Any new media scanner / transcoder / DRM substrate.
- Any Cedar fragment change.
- Annual scheduled review.
- Post-incident review (any Sev-1 or Sev-2).
- Pen-test or audit finding.
- New federation peer onboarded.
- New classifier version deployed (EU AI Act re-evaluation).
- New DMCA agent of record (ops-legal change).
- New pack activation.

## References

- Parallel ADR-0126 (Connect dissolution; shorts as a sibling µservice).
- Bominal ADR-0028, ADR-0111, ADR-0208, ADR-0215.
- ADR-0008 Data Use Boundary.
- `microservices/shorts/PRD.md`.
- `microservices/shorts/dpia.md`.
- `microservices/shorts/compliance.md`.
- `microservices/shorts/policy/dual-context-isolation.md`.
- OWASP API Top 10 (2023).
- NIST SP 800-154.
- EU DSA 2065/2022; EU AI Act 2024/1689; EU AVMSD 2018/1808; UK Online Safety Act 2023; DMCA Title II 17 USC §512.
