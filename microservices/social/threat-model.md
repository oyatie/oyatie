---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: social
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-social + ops-security
deciders: council-architecture, ops-security, axis-social, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + OWASP API Top 10 (2023) + NIST SP 800-154
related_adrs: [ADR-0008, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/microservices/social.json]
review_cadence: quarterly + on every architecture or substrate change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC6.7, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.8.2, A.8.3, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.25, A.8.26, A.8.27, A.8.28"
  - "GDPR Arts. 5, 6, 7, 8, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35"
  - "EU DSA Arts. 14, 16, 17, 20, 23, 24 (transparency and content moderation)"
  - "EU AI Act 2024/1689 Arts. 9, 13, 14, 15, 50 (high-risk + transparency)"
  - "OWASP API Top 10 (2023)"
  - "OWASP ASVS v4.0.3"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 15/17/18/22-2/23/23-2/24/25/28/29/29-2/33/34", "KR-ISMS-P §2.1-2.12", "KR 청소년 보호법 (Juvenile Protection Act)", "KR 정보통신망법 §49"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308-316", "HITECH Act breach-notification", "US COPPA 15 USC §6501 (age-gate)"]
  pack-eu: ["GDPR Arts. 8 (child consent) + 25 + 32 + 35 + 44-50", "EU DSA 2065/2022", "EU DMA where applicable", "EU AI Act 2024/1689", "ePrivacy Directive 2002/58", "UK Online Safety Act 2023 (where UK tenant)"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24/26-2"]
  pack-sg: ["PDPA 2012 §11-26", "Online Safety (Miscellaneous Amendments) Act 2022"]
  pack-au: ["Privacy Act 1988 APP 1-13", "Online Safety Act 2021", "AU eSafety Commissioner BOSE"]
  pack-in: ["DPDPA 2023 §6-10 + child-consent §9"]
  pack-br: ["LGPD Arts. 6/7/11/14/18/33/46/48", "Marco Civil da Internet"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021"]
  pack-ksa: ["PDPL Royal Decree M/19/2021", "SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Threat Model: social µservice

## Purpose

Identify, classify, and mitigate threats to social's confidentiality, integrity, availability, and privacy posture. The social µservice is the canonical first-party social platform across Personal (B2C) + Professional (B2B) contexts; a compromise leaks profile graphs, post history, follow-graph (relationship metadata = sensitive under GDPR Art. 9 in many contexts), and (for pack-us-healthcare) potentially PHI when patient-context posts surface. This document is reviewed by SOC 2 examiners, ISO 27001 auditors, GDPR DPAs, KR PIPC, HIPAA OCR, EU DSA coordinators, and EU AI Act notified bodies at first-tenant onboarding per pack.

## Scope

### In-scope

All components introduced by parallel ADR-0238 (Connect dissolution → social µservice) and ADR-0132 (suite dissolution into social surface) for the social µservice. Deployed in the dedicated social Kubernetes cluster.

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| Postgres 16 LTS (profile + post + follow-graph + reactions + bookmarks store) | `oya-social-user-profile-*` (9 crates) |
| Valkey 8.1 (Redis wire-compat) (feed cache + reaction counters + trending + notification fanout) | `oya-social-follow-graph-*` (8 crates) |
| Meilisearch 0.10.0 (people + content + hashtag search) | `oya-social-post-composition-*` (13 crates) |
| S3-compatible (media blobs + previews + quarantine) | `oya-social-feed-timeline-*` (10 crates) |
| ImageMagick 7.1 (image transcode) | `oya-social-reactions-*` (8 crates) |
| ffmpeg 7.x (video HLS transcode) | `oya-social-mentions-*` (7 crates) |
| OPSWAT MetaDefender / ClamAV 1.x (media scan) | `oya-social-hashtags-*` (8 crates) |
| ActivityPub gateway (optional federation) | `oya-social-trending-topics-*` (9 crates) |
| Cedar policy evaluator | `oya-social-notifications-*` (10 crates) |
| WebSocket gateway (Envoy + Cloudflare termination) | `oya-social-content-moderation-*` (9 crates) |
| | `oya-social-bookmarks-*` (7 crates) |
| | `oya-social-lists-*` (7 crates) |
| | `oya-social-search-*` (8 crates) |
| | `oya-social-profile-verification-*` (7 crates) |
| | `oya-social-age-verification-*` (7 crates) |
| | `oya-social-federation-gateway-*` (8 crates; M02-onward1) |

### Out-of-scope

- Threats to the underlying Kubernetes / hyperscaler — owned by `cloud-k8s`.
- Threats to OpenBao — owned by `cloud-secrets`.
- Threats to audit-chain µservice — owned by its own threat model; inherited.
- Threats to Ontology — owned by `ontology` µservice; inherited for mention-resolution path.
- Threats to GitHub Actions — owned by `governance`.
- Threats to foundry-runtime model deployment (content-moderation classifier itself) — inherited from `foundry-runtime` and `foundry-guardrails` threat models.

## Trust Boundaries

```text
┌─ Internet ────────────────────────────────────────────────────────────────┐
│                                                                           │
│   End-users (web/desktop/mobile)         Workflow Studio shell            │
│         │                                       │                         │
│         │ (TLS, WSS, OIDC, OAuth 2.1)           │ (mTLS internal)         │
│         ▼                                       ▼                         │
│  ┌─ Public ingress (Envoy/Cloudflare) ──────────────────────────────┐     │
│  │  TLS + WAF + DDoS + WebSocket upgrade                            │     │
│  └──────────────────────────────────────────────────────────────────┘     │
│                              │                                            │
└──────────────────────────────│────────────────────────────────────────────┘
                               ▼
┌─ Dedicated social cluster ────────────────────────────────────────────────┐
│                                                                           │
│  TB1: External → Cluster ingress                                          │
│                                                                           │
│  ┌─ WebSocket gateway (real-time notifications + feed updates) ──┐        │
│  │  per-tenant connection registry                                │        │
│  │  X-Scope-OrgID enforcement                                     │        │
│  └────────────────────────────────────────────────────────────────┘        │
│                                                                           │
│  TB2: WebSocket gateway → BC services (mTLS + SPIFFE)                     │
│                                                                           │
│  ┌─ user-profile-rest ──┐ ┌─ post-composition-rest ─┐ ┌─ feed-timeline ┐  │
│  │ Cedar evaluation     │ │ Cedar evaluation        │ │ Cedar eval     │  │
│  └──────────────────────┘ └─────────────────────────┘ └────────────────┘  │
│                                                                           │
│  TB3: BC services → backing stores                                        │
│                                                                           │
│  ┌─ Postgres (per-tenant RLS) ─┐  ┌─ Valkey cluster ─┐                     │
│  │ profiles, posts, follows    │  │ feed-cache, react│                     │
│  └─────────────────────────────┘  └──────────────────┘                     │
│  ┌─ S3 (media blobs; KMS) ──────┐  ┌─ Meilisearch ──┐                     │
│  │ per-tenant prefix isolation  │  │ per-tenant idx │                     │
│  └──────────────────────────────┘  └────────────────┘                     │
│                                                                           │
│  TB4: Personal/Professional context isolation (data-model invariant)      │
│                                                                           │
│  TB5: BC services → audit-chain µservice (Ed25519-signed)                 │
│                                                                           │
│  TB6: BC services → ontology µservice (Workflow event)                    │
│                                                                           │
│  TB7: Media scan path (OPSWAT/ClamAV; quarantine bucket)                  │
│                                                                           │
│  TB8: Media transcode (ImageMagick + ffmpeg; isolated worker pool)        │
│                                                                           │
│  TB9: Federation egress (ActivityPub) — Professional-tier only;           │
│         peer-allowlist; signed HTTP signatures                            │
│                                                                           │
│  TB10: Content-moderation classifier (foundry-runtime client) — high-risk │
│         per EU AI Act; classifier verdicts emit transparency record       │
│                                                                           │
└───────────────────────────────────────────────────────────────────────────┘
```

Ten trust boundaries:
1. **External → Cluster ingress** (TLS, WAF, DDoS, WebSocket upgrade).
2. **Gateway → BC services** (mTLS + SPIFFE identity).
3. **BC services → backing stores** (RLS + per-tenant prefix isolation).
4. **Personal/Professional context isolation** (data-model invariant per parallel ADR-0238).
5. **BC services → audit-chain** (Ed25519 seal).
6. **BC services → ontology** (Workflow event bus).
7. **Media scan path** (untrusted blob → scanner → quarantine vs production).
8. **Media transcode isolation** (ImageMagick + ffmpeg worker pool; CVE-prone; sandboxed).
9. **Federation egress / ingress** (ActivityPub peer-allowlist; signed HTTP signatures; Personal-tier forbidden).
10. **Content-moderation classifier** (EU AI Act high-risk; transparency).

## Assets & Data Classification

| Asset | Class | Retention | Authoritative store |
|---|---|---|---|
| User profiles (handle, display-name, bio, avatar, header) | `PII_IDENTIFYING` + sometimes `PII_QUASI_IDENTIFIER` | per-pack | Postgres |
| Posts (Professional) | `BEHAVIORAL_TENANT_PRODUCT` + sometimes `PII_IDENTIFYING` + occasionally `PHI` (pack-us-healthcare) | per-pack (90d hot, retention floor per regulator) | Postgres (tenant-DEK encrypted when configured) |
| Posts (Personal) | `PERSONAL` + `BEHAVIORAL_USER_CONTENT` | per-user policy + per-pack | Postgres (server-stored; not E2E-encrypted because posts are public-by-default) |
| Comments / replies | inherits parent | inherits | Postgres |
| Reactions | `BEHAVIORAL_TENANT_PRODUCT` | 365d hot | Valkey (persisted to Postgres asynchronously) |
| Follow-graph (directed edges + block/mute lists) | `RELATIONSHIP_GRAPH` + `PII_QUASI_IDENTIFIER` | append-only with tombstone-on-unfollow | Postgres (adjacency-list) |
| Media (image / video blobs) | `BEHAVIORAL_TENANT_PRODUCT` + sometimes `PII_IDENTIFYING` (faces) / `PHI` (pack-us-healthcare) | per-pack | S3 (KMS-encrypted; per-tenant prefix) |
| Media metadata (digest, preview, transcode variant, scan-verdict) | `INTERNAL_ONLY` + `AUDIT` | append-only | Postgres |
| Notification records | `BEHAVIORAL_TENANT_PRODUCT` + `PII_IDENTIFYING` | 90d hot | Postgres + Valkey |
| Trending-topic windows | derived; `INTERNAL_ONLY` | rebuilt continuously | Valkey |
| Search index | derived from profiles + posts | rebuilt from source | Meilisearch (per-tenant index) |
| Moderation verdicts + appeal trail | `AUDIT` + `INTERNAL_ONLY` + `PII_IDENTIFYING` (reporter ref) | append-only; immutable | Postgres + audit-chain seal |
| Hashtag corpus | derived | rebuilt | Postgres + Valkey |
| Bookmarks (per-user, private) | `BEHAVIORAL_USER_CONTENT` | user lifecycle | Postgres |
| Lists | `BEHAVIORAL_USER_CONTENT` (private) or `BEHAVIORAL_TENANT_PRODUCT` (public) | user lifecycle | Postgres |
| Verification badges | `AUDIT` + `BEHAVIORAL_TENANT_PRODUCT` | append-only history | Postgres |
| Age attestations | `PII_QUASI_IDENTIFIER` + `SENSITIVE_CHILD_PROTECTION` (when minor) | per-pack | Postgres (separated table; restricted access) |
| Audit-chain seals (every state transition) | `AUDIT` | append-only; immutable | audit-chain µservice |
| Per-tenant DEK | `SECRET` | OpenBao 30d rotation; envelope KMS | OpenBao |
| WebSocket gateway session tokens | `SECRET` | ≤ 24h | OpenBao-issued short-lived JWT |
| Federation peer keys + signatures | `SECRET` (private side) / `PUBLIC` (public side) | rotation 90d | OpenBao |
| Ranking-model snapshots + training data | `INTERNAL_ONLY` + `AUDIT` (model card + eval record) | per release | foundry-runtime evidence pipeline |
| Content-moderation classifier model card | `AUDIT` + `INTERNAL_ONLY` | per release | foundry-runtime + audit-chain |

## Actors

| Actor | Trust | Auth | Capability |
|---|---|---|---|
| End-user (human) | Untrusted external | OIDC + MFA + WSS bearer | Read/write own profile + posts; follow others; react; comment |
| Tenant-admin | Semi-trusted internal-to-tenant | OIDC + MFA | Manage tenant config; verification policy; cannot read PII posts without four-eyes |
| Tenant compliance-officer | Semi-trusted internal-to-tenant | OIDC + MFA + Cedar entitlement | Issue eDiscovery hold (Professional-tier only); trigger disclosure (requires four-eyes peer) |
| Tenant moderator | Semi-trusted internal-to-tenant | OIDC + MFA + Cedar entitlement | Manual review of flagged content; verdicts + appeals |
| Tenant security-admin | Semi-trusted internal-to-tenant | OIDC + MFA + Cedar entitlement | Configure pack policy; four-eyes pairing peer |
| oyatie ops-security (human) | Trusted internal | OIDC + MFA + JIT via OpenBao | Admin access; no plaintext PII without breakglass + two-person rule |
| Workflow Studio (machine) | Semi-trusted internal | mTLS + SPIFFE | Consume Workflow events; post action-cards via mention-bridge |
| messenger µservice (machine) | Semi-trusted internal | mTLS + SPIFFE | Deep-link post resolution; mention bridge |
| ontology µservice (machine) | Semi-trusted internal | mTLS + SPIFFE | Serve Person/Team/Topic lookups |
| audit-chain µservice (machine) | Trusted internal | mTLS + SPIFFE | Receive seals from every BC |
| foundry-runtime µservice (machine) | Semi-trusted internal | mTLS + SPIFFE | Provide classifier + ranking inference; high-risk per EU AI Act |
| ActivityPub federation peer (external) | Untrusted external | HTTP Signatures (RFC 9421) + peer allowlist | Receive Professional-tier outbox; submit to inbox |
| External auditor | Read-only external | OIDC + MFA + JIT short-lived token | Read tenant-scoped audit-chain seals; read policy artifacts |
| Attacker — opportunistic | Untrusted | none | Scans + low-skill exploitation |
| Attacker — targeted | Untrusted | none | Sophisticated supply-chain awareness; influence-op attempts |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure retention or moderation |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case for confidentiality / integrity; mitigated by audit-chain + four-eyes |

## STRIDE Threat Catalog

### Spoofing (S)

**T-S-01 — User-B impersonates User-A via WebSocket session hijack**
- Asset: WebSocket session
- Likelihood M / Impact H / Risk **H**
- Mitigations: per-connection short-lived JWT bound to device + IP; WSS only; rotation 24h; OIDC re-auth on token expiry; anomaly detection on geo-shift mid-session.
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.5; GDPR Art. 32(1)(a)(b)

**T-S-02 — Handle squatting (attacker registers handle of trademark / verified user)**
- Asset: Profile handle namespace
- Likelihood H / Impact M / Risk **H**
- Mitigations: per-tenant handle namespace; verification-badge program; handle-revocation policy for impersonation reports; trademark-reservation onboarding option for Professional tenants; Cedar policy `policy/tenant-scope.cedar` enforces handle-uniqueness scope; LEAN lane verifies.
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15

**T-S-03 — Forged @mention or comment from external sender (federation)**
- Asset: federation-gateway ingress
- Likelihood M / Impact M / Risk **M**
- Mitigations: HTTP Signatures (RFC 9421) verified on every inbox delivery; peer allowlist; mentions from un-allowlisted peers rejected; ActivityPub object signed by sender public key.
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15

**T-S-04 — Tenant-admin impersonates compliance-officer to trigger Professional-tier disclosure single-handed**
- Asset: Four-eyes disclosure path
- Likelihood L / Impact H / Risk **M**
- Mitigations: four-eyes requires two distinct SPIFFE identities with distinct entitlements + audit-chain seal of both consents; same principal cannot satisfy both halves; replay-resistant nonce.

**T-S-05 — Bot / scripted account masquerades as human (sybil)**
- Asset: Engagement signals (likes, follows, reposts)
- Likelihood H / Impact M / Risk **H**
- Mitigations: signup CAPTCHA; rate-limited account creation per IP + per device fingerprint; foundry-guardrails bot-detection classifier; tenant-admin can require email/phone verification; per-tenant policy controls.

### Tampering (T)

**T-T-01 — Post tampering at rest (Postgres row mutation)**
- Asset: Post store
- Likelihood L / Impact H / Risk **M**
- Mitigations: every post row carries `content_hash` (sha256(plaintext)) emitted to audit-chain on write; periodic batch verifier compares hashes; mismatch quarantines + alerts.

**T-T-02 — Media blob tampering in S3**
- Asset: Media blobs
- Likelihood L / Impact H / Risk **M**
- Mitigations: SSE-KMS + S3 Object Lock (WORM) on Professional-tier; content-digest verified at fetch; tamper triggers quarantine; bucket access via service-account IAM only.

**T-T-03 — Follow-graph tampering (mass-follow / unfollow attack)**
- Asset: Follow-graph integrity
- Likelihood M / Impact M / Risk **M**
- Mitigations: per-user follow rate limit (default 100/hr); audit-chain seal of every follow / unfollow; periodic graph-anomaly detector; account suspension on confirmed mass-follow attack.

**T-T-04 — Reaction-count tampering (vote-stuffing)**
- Asset: Reaction tally
- Likelihood M / Impact M / Risk **M**
- Mitigations: per-user-per-post idempotency (one reaction per user per post per emoji); conflict-free counter; periodic reconciliation Postgres ↔ Valkey.

**T-T-05 — Search index poisoning**
- Asset: Meilisearch index
- Likelihood L / Impact M / Risk **L**
- Mitigations: only social-search-worker writes to index; SPIFFE-validated; rebuild from source possible (deterministic).

**T-T-06 — Moderation classifier verdict tampering**
- Asset: Moderation verdict ledger
- Likelihood L / Impact H / Risk **M**
- Mitigations: every classifier verdict signed by foundry-runtime + sealed via audit-chain Ed25519; verdict-replay detector compares Postgres state vs audit-chain.

**T-T-07 — Trending-topic manipulation (artificial trend injection)**
- Asset: Trending-topic ranking
- Likelihood M / Impact M / Risk **M**
- Mitigations: trend compute uses windowed dedup keyed by `(tenant_id, hashtag, author_ref)`; per-author influence cap; foundry-guardrails sybil detector inputs; tenant-admin can pin / unpin trends; periodic audit.

### Repudiation (R)

**T-R-01 — User denies authoring a post**
- Asset: Post authorship
- Likelihood M / Impact M / Risk **M**
- Mitigations: every post carries author SPIFFE identity + session-token nonce + audit-chain seal; client-side device-key signing where available.

**T-R-02 — Tenant-admin denies authorising disclosure**
- Asset: Four-eyes disclosure record
- Likelihood L / Impact H / Risk **M**
- Mitigations: four-eyes requires both consents in audit-chain with distinct principal IDs + reason code; non-repudiable.

**T-R-03 — Moderator denies issuing a verdict**
- Asset: Moderation verdict record
- Likelihood L / Impact M / Risk **L**
- Mitigations: verdict emits Ed25519-signed audit-chain record with reviewer principal + timestamp + scope.

### Information Disclosure (I)

**T-I-01 — Cross-tenant post leak via Postgres RLS misconfiguration**
- Asset: Post store
- Likelihood M / Impact H / Risk **H**
- Mitigations: Postgres Row-Level Security with `tenant_id = current_setting('app.tenant_id')`; gateway sets the GUC per connection; LEAN check `oya-check-postgres-rls-coverage` asserts RLS enabled on every social table; pen-test annually.

**T-I-02 — PHI leak in post body or media (pack-us-healthcare)**
- Asset: Post body + media
- Likelihood M / Impact H / Risk **H**
- Mitigations: pack-us-healthcare disables federation by default; PHI-redactor scans OCR output on media; PHI in search-index stripped via `policy/redaction-phi.md`; HIPAA Safe Harbor §164.514 honoured.

**T-I-03 — Personal-tier profile data leaked via tenant-admin pivot**
- Asset: Personal-tier profile + posts
- Likelihood M / Impact H / Risk **H**
- Mitigations: Cedar `policy/tenant-scope.cedar` blocks tenant-admin reads of Personal-context resources; LEAN lane verifies; cross-context disclosure attempts emit `social_personal_admin_decrypt_attempt_total` (target=0).

**T-I-04 — Follow-graph pivoting reveals relationship graph (Art. 9 sensitive)**
- Asset: Follow-graph
- Likelihood M / Impact M / Risk **M**
- Mitigations: follow-graph reads bounded by Cedar (own-graph + public-followers-of-public-profile only); cross-tenant graph enumeration forbidden; per-tenant cardinality limits; aggregate stats only for non-owners.

**T-I-05 — Search-result leak: returns posts user cannot read**
- Asset: Search results
- Likelihood M / Impact H / Risk **H**
- Mitigations: search post-filters by Cedar evaluation; result set redacted to caller-scope; integration test asserts no over-permitted result.

**T-I-06 — Media URL leak via shared-link guess**
- Asset: Media URL
- Likelihood M / Impact H / Risk **H**
- Mitigations: media URLs are signed short-TTL (≤ 15 min); per-fetch Cedar re-evaluation; public posts use unsigned but Cedar-checked CDN URL; private posts require signed URL per fetch.

**T-I-07 — Cross-context routing (Personal post → Professional feed, or vice versa)**
- Asset: Dual-context isolation
- Likelihood L / Impact H (regulatory + privacy breach) / Risk **H**
- Mitigations: data-model invariant — `PersonalProfile` and `ProfessionalProfile` (with their owned posts) are distinct entity types; cross-type write rejected by domain layer; LEAN-lane `oya-check-dual-context-isolation` validates type signatures forbid cross-context flows.

**T-I-08 — Federation egress leak (Personal-tier post escapes via ActivityPub)**
- Asset: Personal-tier confidentiality
- Likelihood L / Impact H / Risk **H**
- Mitigations: `federation-gateway` rejects any Personal-tier post at the egress port (compile-time type signature: outbox accepts only `ProfessionalPost`); Cedar `policy/federation.cedar` forbid rule (ships in IP-016-federation-egress-cedar-rules of the social µservice's federation slice); runtime guard + LEAN lane.

**T-I-09 — Notification metadata leak (notification body reveals private post body to non-member)**
- Asset: Notification body
- Likelihood M / Impact M / Risk **M**
- Mitigations: notification bodies carry only redacted previews + signed deep-link; full post body fetched at click-time with Cedar re-eval; private-visibility posts omit body in notification.

**T-I-10 — Age-attestation table pivoting (minor list)**
- Asset: Age attestations (especially minor flags)
- Likelihood L / Impact H / Risk **H**
- Mitigations: separate `social_age_attestations` table; access bound to Cedar `age_verification_reader` entitlement (rare; only minor-protection compliance flows); no general staff read; encryption at rest; LEAN lane verifies isolation.

### Denial of Service (D)

**T-D-01 — Feed-render storm: viral post causes mass concurrent feed-pulls**
- Asset: Feed cache
- Likelihood H / Impact H / Risk **H**
- Mitigations: Valkey hot-feed cache; fanout-on-write for hot accounts (precomputed feed); per-tenant feed-render rate limit; HPA on REST pods; runbook `runbooks/feed-cache-rebuild.md`.

**T-D-02 — Mention storm: one post @-mentions thousands**
- Asset: mentions worker queue
- Likelihood M / Impact M / Risk **M**
- Mitigations: per-post mention cap (default 50; tenant-configurable to 500); over-cap mentions truncated + sender warned; runbook `runbooks/mention-storm-throttle.md`.

**T-D-03 — Notification fanout storm: a celebrity account with 1M followers posts**
- Asset: notification worker queue
- Likelihood H / Impact M / Risk **M**
- Mitigations: sharded notification workers; per-recipient idempotent processing; coalesce digest for low-priority notifications; backpressure-throttle to per-recipient at hot-window.

**T-D-04 — Media-transcode storm (mass-upload abuse)**
- Asset: ImageMagick + ffmpeg worker pool
- Likelihood M / Impact H / Risk **H**
- Mitigations: per-tenant upload rate limit; queue depth bound; worker pool sized per capacity model; runbook `runbooks/media-transcode-degraded.md` (referenced; same shape as messenger attachment-restore).

**T-D-05 — Trending-topic poisoning (sybil-amplified hashtag)**
- Asset: Trending-topic ranking integrity
- Likelihood M / Impact M / Risk **M**
- Mitigations: foundry-guardrails sybil detector; per-author influence cap in trending; tenant-admin pin/unpin; runbook `runbooks/trending-topic-poisoning.md`.

**T-D-06 — Search-index lag during ingest spike**
- Asset: Search index
- Likelihood M / Impact M / Risk **M**
- Mitigations: backpressure on indexer; live-fallback to Postgres ILIKE-search (slower but correct); runbook (search-index-rebuild shape; same as messenger).

**T-D-07 — Federation ingress flood (untrusted federation peer DDoSes inbox)**
- Asset: federation-gateway inbox
- Likelihood M / Impact M / Risk **M**
- Mitigations: peer allowlist; per-peer rate limit; HTTP Signature verification fails closed; runbook `runbooks/federation-bridge-degraded.md`.

**T-D-08 — Postgres ingest spike causes post-create latency breach**
- Asset: Post store
- Likelihood M / Impact H / Risk **H**
- Mitigations: per-tenant ingest rate limit; bulk-write buffering; HPA scale-out; sharding past per-cell capacity threshold.

### Elevation of Privilege (E)

**T-E-01 — Cedar policy bug grants moderator entitlement to non-moderator**
- Asset: Cedar evaluator
- Likelihood L / Impact H / Risk **M**
- Mitigations: Cedar v4 LTS; fragment fuzz; integration test asserts no over-permitted action; periodic Cedar-fragment-coverage CI lane.

**T-E-02 — Compromised tenant-moderator pivots to read all tenant posts**
- Asset: Moderator scope
- Likelihood L / Impact H / Risk **M**
- Mitigations: moderator scope bounded to flagged-only reads (must come from `AbuseReportFiled` event); cannot read arbitrary posts; LEAN lane verifies moderator scope policy.

**T-E-03 — Mentions BC pivots to read Ontology entities it shouldn't**
- Asset: Ontology read path
- Likelihood L / Impact M / Risk **L**
- Mitigations: mentions authenticates as scoped SPIFFE identity; ontology enforces per-caller Cedar; queries constrained to `Person`, `Team`, `Topic` resolution shapes.

**T-E-04 — Media scanner bypasses scan path**
- Asset: Quarantine boundary
- Likelihood L / Impact H / Risk **M**
- Mitigations: blob lifecycle: PUT → quarantine bucket → scanner → on-clean copy to production bucket; production bucket write-only by scanner SA.

**T-E-05 — ImageMagick / ffmpeg CVE allows RCE in transcode worker**
- Asset: Transcode worker pool
- Likelihood M / Impact H / Risk **H**
- Mitigations: transcode workers run in gVisor / Kata Container sandbox; non-root; read-only root FS; no network egress except to S3 quarantine + production; LTS pin (ImageMagick 7.1, ffmpeg 7.x); weekly CVE scan via Trivy + Grype.

## LINDDUN Privacy-Threat Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | Follow-graph | Mutual-follow + cross-tenant correlation could re-identify users across packs | Per-tenant scope; Cedar evaluation; never cross-tenant linkable; cross-pack federation Personal-tier forbidden | L |
| T-L-02 | Identifiability | User handle + display-name | Combination may identify a user across services | Per-tenant handle namespace; cross-tenant correlation forbidden; users can use pseudonymous handle for Personal-tier | L |
| T-L-03 | Non-repudiation | Post authorship | User cannot deny authoring a public post since session-token signs | Acceptable per GDPR Art. 5(2); explicit in onboarding notice | L |
| T-L-04 | Detectability | Posting time | Post timestamps reveal activity rhythm | Acceptable; public-by-default semantics; covered by tenant onboarding consent | M |
| T-L-05 | Disclosure | Compliance hold reveals Professional post bodies to admin | Hold + four-eyes disclosure inherently exposes professional post bodies to tenant admin | Mitigated to acceptable: four-eyes + audit-chain + reason code + tenant disclosure obligation | M |
| T-L-06 | Unawareness | End-user (tenant's user) | End-user may not know tenant-admin can disclose Professional-tier posts under four-eyes | Tenant DPA includes disclosure clause; tenant onboarding notice required | M |
| T-L-07 | Non-compliance | GDPR Art. 17 right-to-erasure | User requests erasure across all posts + comments + reactions + follows | DSR cascade marks rows tombstoned + redacts identifiers; 30d SLA | M |
| T-L-08 | Linkability | Reaction history pivot | A user's reactions across posts form a behavioral profile | Per-user opt-in to public-reaction-list; per-tenant policy; default private to non-followers | L |
| T-L-09 | Identifiability | Verification badge | Verification badge ties a real-world identity to a handle | Acceptable; verified-handle is intentionally disclosive; only with user's verification request | L |
| T-L-10 | Disclosure | EU AI Act Art. 50 transparency obligation | Algorithmic ranking + moderation classifier output must be disclosed to user | Every classifier verdict carries `eu_ai_act_label: ai_generated_assessment`; ranking explanation API per Art. 50 | L |
| T-L-11 | Non-compliance | KR PIPA Art. 8 child consent | Minor signup requires parental consent | `age-verification` BC routes minor accounts through parental-consent flow; per-pack overlay | L |
| T-L-12 | Disclosure | Federation egress (Personal-tier leak attempt) | Personal-tier post accidentally federates | Compile-time type-system invariant (federation outbox accepts only ProfessionalPost); LEAN lane verifies | L |

## Mitigations Catalog

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Postgres RLS on every social table | Preventive | axis-social | `oya-check-postgres-rls-coverage` lane |
| Per-connection short-lived JWT bound to device + IP | Preventive | axis-social | gateway audit log |
| Four-eyes disclosure with distinct principal IDs | Preventive | axis-social + ops-security | integration test |
| Cedar policy on every read/write | Preventive | axis-social | LEAN coverage lane |
| Media scan + quarantine workflow | Preventive | axis-social | end-to-end test |
| Personal-tier never-federates type invariant | Preventive | axis-social | `oya-check-federation-personal-tier-refused` lane |
| Audit-chain Ed25519 seal on every state transition | Detective + Non-repudiation | audit-chain | regression tests |
| Cross-context type-system invariant (PersonalProfile ≠ ProfessionalProfile) | Preventive | axis-social | `oya-check-dual-context-isolation` lane |
| Per-tenant rate + cardinality limits | Preventive (DoS) | axis-social | gateway + Postgres metrics |
| DSR cascade for right-to-erasure | Preventive (compliance) | council-privacy | DSR dashboard SLO |
| EU AI Act high-risk transparency record per classifier verdict | Preventive (compliance) | axis-social + axis-foundry-runtime | `capabilities/T2-auto.yaml` evidence pipeline |
| Age-gate routing at signup (pack-aware) | Preventive (compliance) | axis-social + council-privacy | `age-verification` BC unit + integration tests |
| Trending-topic sybil detector | Preventive | axis-social + axis-foundry-guardrails | trending-topic poisoning runbook drill |
| ImageMagick / ffmpeg sandboxed workers (gVisor / Kata) | Preventive | axis-social + cloud-k8s | IaC enforces; LEAN lane verifies |

## Residual Risk Acceptance

| Risk ID | Residual | Why | Re-review |
|---|---|---|---|
| T-I-02 (PHI in media) | L–M | pack-us-healthcare disables federation by default; PHI-redactor on OCR | Quarterly |
| T-L-04 (timing detectability) | M | Public-by-default semantics; consent at onboarding | Annually |
| T-L-05 (hold disclosure inherent) | M | Four-eyes + audit are the load-bearing control; user-side opacity unavoidable | Annually |
| T-L-06 (end-user unawareness) | M | Joint-controllership clause | Annually |
| T-L-07 (erasure best-effort) | M | Retention bounds + audit immutability tradeoff | Annually |
| T-D-05 (trending-topic poisoning) | M | Sybil detector + tenant-admin pinning; some residual risk unavoidable in any open social platform | Quarterly |

Sign-off:
- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`

## Per-Pack Overlay Sections

### pack-kr

- KR PIPA Art. 23 sensitive personal info — sensitive posts (medical, juvenile, biometric) require additional consent at signup.
- KR 청소년 보호법 (Juvenile Protection Act) — minor-protection routing in `age-verification`.
- KR 정보통신망법 §49 (intercept) — admin reads only via four-eyes; covered.
- KR-ISMS-P §2.7 — access control via Cedar.

### pack-us-healthcare

- HIPAA §164.312(a)(1) — access control via Cedar + RLS.
- HIPAA §164.312(b) — audit-chain ≥ 6y retention overlay; cost-budget.md reflects.
- HIPAA §164.502(b) — minimum-necessary: media OCR redactor + search-redaction.
- US COPPA 15 USC §6501 — age-gate at signup; child-account flow.
- HIPAA §164.314 (Business Associate) — per-tenant BAA at `microservices/social/legal/baa-template.md` (Slice B).

### pack-eu

- GDPR Art. 8 — child consent (16y default; member states may lower to 13y).
- GDPR Art. 25 — privacy-by-design via cross-context invariant.
- GDPR Art. 32 — every mitigation above contributes.
- GDPR Arts. 44-50 — pack-eu data stays in EU pack; federation requires SCC.
- EU DSA Arts. 14, 16, 17, 20, 23, 24 — transparency reports + content moderation + appeal workflow + Statement of Reasons.
- EU AI Act 2024/1689 — ranking + moderation classifiers are high-risk; obligations satisfied per Arts. 9-15 + Art. 50 transparency.
- UK Online Safety Act 2023 — Ofcom illegal-content duty; safety-by-design.
- ePrivacy Directive Art. 5(3) — confidentiality of communications.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per pack overlay at `regional-packs/<pack>/social-overlay.md`; cross-mapped via compliance.md.

## Compliance Cross-Mapping

| Framework | Coverage | Mapping doc |
|---|---|---|
| SOC 2 Type 2 | CC1–CC9 covered in `compliance.md` |
| ISO 27001:2022 | A.5–A.8 covered |
| GDPR | Arts. 5, 6, 8, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35 covered in `dpia.md` + `compliance.md` |
| EU DSA | Arts. 14, 16, 17, 20, 23, 24 covered in `compliance.md` |
| EU AI Act | Arts. 9, 13, 14, 15, 50 covered via `capabilities/T2-auto.yaml` + ADR-SOC-0003 |

## Re-review Triggers

- Any new BC.
- Any change to dual-context invariant.
- Any new media scanner / transcoder.
- Any Cedar fragment change.
- Annual scheduled review.
- Post-incident review (any Sev-1 or Sev-2).
- Pen-test or audit finding.
- New federation peer onboarded.
- New classifier version deployed (EU AI Act re-evaluation).

## References

- Parallel ADR-0135 (Connect dissolution; social as a sibling µservice).
- Bominal ADR-0028, ADR-0111, ADR-0208, ADR-0215.
- ADR-0008 Data Use Boundary.
- `microservices/social/PRD.md`.
- `microservices/social/dpia.md`.
- `microservices/social/compliance.md`.
- `microservices/social/policy/dual-context-isolation.md`.
- OWASP API Top 10 (2023).
- NIST SP 800-154.
- EU DSA 2065/2022; EU AI Act 2024/1689.
