---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: network
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-network + ops-security
deciders: council-architecture, ops-security, axis-network, council-privacy, ops-compliance, ops-legal
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + OWASP API Top 10 (2023) + NIST SP 800-154 + NIST AI RMF 1.0
related_adrs: [ADR-0008, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0130, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-NET-0001, ADR-NET-0002, ADR-NET-0003, ADR-NET-0004, ADR-NET-0005, ADR-NET-0006]
related_specs: [/specs/microservices/network.json]
review_cadence: quarterly + on every architecture or substrate or classifier-version change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC6.7, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.8.2, A.8.3, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.25, A.8.26, A.8.27, A.8.28"
  - "ISO 30414:2018 (HR analytics + workforce reporting)"
  - "GDPR Arts. 5, 6, 7, 8, 9, 13, 14, 17, 21, 22, 25, 28, 30, 32, 33, 35"
  - "EU Equal Treatment Directives 2000/43/EC + 2000/78/EC (employment context)"
  - "EU DSA Arts. 14, 16, 17, 20, 23, 24 (transparency and content moderation)"
  - "EU AI Act 2024/1689 Arts. 9, 13, 14, 15, 27, 50, 73 (high-risk + transparency; Annex III §4 employment)"
  - "OWASP API Top 10 (2023)"
  - "OWASP ASVS v4.0.3"
  - "NIST AI RMF 1.0 (Govern + Map + Measure + Manage)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 15/17/18/22-2/23/23-2/24/25/28/29/29-2/33/34", "KR-ISMS-P §2.1-2.12", "KR 근로기준법 (Labor Standards Act)", "KR 직장 갑질 protections", "KR 통신비밀보호법", "KR 정보통신망법 §49", "KR 채용절차의 공정화에 관한 법률 (Fair Hiring Procedures Act)"]
  pack-us: ["EEOC UGESP 1978", "Title VII Civil Rights Act 1964", "ADA 1990", "ADEA 1967", "OFCCP regulations (federal contractors)", "CCPA + CPRA", "NYC AI Hiring Law (Local Law 144-2021)", "CA AB-331 (automated decision-making transparency)", "CO SB-205 (AI Act)", "IL AI Video Interview Act"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308-316", "HITECH Act breach-notification"]
  pack-eu: ["GDPR Arts. 21 + 22 + 25 + 32 + 35 + 44-50", "EU DSA 2065/2022", "EU AI Act 2024/1689 Annex III §4", "EU Equal Treatment Directives 2000/43/EC + 2000/78/EC", "ePrivacy Directive 2002/58", "UK Equality Act 2010", "UK ICO ADM guidance", "Council of Europe Convention 108+"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24/26-2", "JP 労働基準法 (Labor Standards Act)", "JP 労働契約法 (Labor Contract Act)", "JP 個人情報保護法"]
  pack-sg: ["PDPA 2012 §11-26", "PDPC Employment guidance"]
  pack-au: ["Privacy Act 1988 APP 1-13", "Australian Human Rights Commission AI guidance", "Fair Work Act 2009"]
  pack-in: ["DPDPA 2023 §6-10"]
  pack-br: ["LGPD Arts. 6/7/11/14/18/33/46/48", "Brazilian CLT (Labor Code)"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021", "UAE Federal Decree-Law 33/2021 (Labour)"]
  pack-ksa: ["PDPL Royal Decree M/19/2021", "KSA Labor Law", "SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Threat Model: network µservice

## Purpose

Identify, classify, and mitigate threats to network's confidentiality, integrity, availability, privacy, and employment-context-fairness posture. The network µservice is the canonical first-party Professional network (LinkedIn-class); a compromise leaks employment records, connection graphs (Article 9 sensitive in many interpretations), endorsement signals, InMail bodies (Professional-tier confidentiality), and recruiter-search ranking decisions (EU AI Act Annex III §4 high-risk + EEOC + Title VII disparate-impact concerns). This document is reviewed by SOC 2 examiners, ISO 27001 auditors, GDPR DPAs, KR PIPC, EU DSA coordinators, EU AI Act notified bodies, EEOC + OFCCP (when US federal contractor tenant), and NYC Department of Consumer and Worker Protection (when NYC tenant activates recruiter-stub) at first-tenant onboarding per pack.

## Scope

### In-scope

All components introduced by parallel ADR-0135 (Connect dissolution → network µservice) and ADR-0132 (suite dissolution into network surface) for the network µservice. Deployed in the dedicated network Kubernetes cluster.

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| Postgres 16 LTS (profile + post + connection-graph + endorsement + recommendation + page + group + event + job store) | `oya-network-professional-profile-*` (9 crates) |
| Redis 7.2 (feed cache + reaction counters + trending + notification fanout + InMail rate budget) | `oya-network-professional-graph-*` (8 crates) |
| Meilisearch 0.10.0 (people + content + skills + companies + jobs + events search; faceted) | `oya-network-connection-request-*` (8 crates) |
| S3-compatible (media blobs + document attachments + previews + quarantine) | `oya-network-post-composition-*` (13 crates) |
| ImageMagick 7.1 (image transcode) | `oya-network-feed-timeline-*` (10 crates) |
| ffmpeg 7.x (video HLS transcode) | `oya-network-reactions-*` (8 crates) |
| OPSWAT MetaDefender / ClamAV 1.x (media + document scan) | `oya-network-mentions-*` (7 crates) |
| Cedar v4.2 LTS policy evaluator | `oya-network-hashtags-*` (8 crates) |
| WebSocket gateway (Envoy + Cloudflare termination) | `oya-network-trending-topics-*` (9 crates) |
| Audit-chain client (Ed25519 + Merkle; signed endorsement chain) | `oya-network-notifications-*` (10 crates) |
| foundry-runtime client (T1 + T2 capabilities) | `oya-network-inmail-bridge-*` (8 crates) |
| messenger-bridge adapter | `oya-network-endorsement-engine-*` (8 crates) |
| calendar-bridge adapter | `oya-network-skill-assessments-*` (8 crates) |
| mail-bridge adapter (Page newsletter) | `oya-network-profile-verification-*` (7 crates) |
| ats-bridge adapter (Tier-G jobs handoff) | `oya-network-pages-*` (9 crates) |
| | `oya-network-groups-*` (9 crates) |
| | `oya-network-events-bridge-*` (8 crates) |
| | `oya-network-jobs-handoff-*` (9 crates) |
| | `oya-network-recruiter-stub-*` (7 crates; OFF) |
| | `oya-network-services-marketplace-stub-*` (7 crates; OFF) |
| | `oya-network-learning-stub-*` (7 crates; OFF) |
| | `oya-network-salary-insights-stub-*` (7 crates) |
| | `oya-network-search-*` (8 crates) |
| | `oya-network-accessibility-captions-*` (7 crates) |
| | `oya-network-abuse-reporting-*` (8 crates) |

### Out-of-scope

- Threats to the underlying Kubernetes / hyperscaler — owned by `cloud-k8s`.
- Threats to OpenBao — owned by `cloud-secrets`.
- Threats to audit-chain µservice — owned by its own threat model; inherited.
- Threats to Ontology — owned by `ontology` µservice; inherited for mention-resolution path.
- Threats to GitHub Actions — owned by `governance`.
- Threats to foundry-runtime model deployment (classifier + recommender + recruiter ranker themselves) — inherited from `foundry-runtime` and `foundry-guardrails` threat models; this document covers integration-layer threats only.
- Threats to the Tier-G ATS µservice — owned by its own threat model; this document covers handoff-contract integrity only.
- Threats to messenger µservice — owned by its own threat model; this document covers InMail-bridge integrity only.

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
┌─ Dedicated network cluster ───────────────────────────────────────────────┐
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
│  ┌─ profile-rest ───┐ ┌─ post-rest ─┐ ┌─ feed-timeline ┐ ┌─ pages-rest ┐  │
│  │ Cedar evaluation │ │ Cedar eval  │ │ Cedar eval     │ │ Cedar eval  │  │
│  └──────────────────┘ └─────────────┘ └────────────────┘ └─────────────┘  │
│                                                                           │
│  TB3: BC services → backing stores                                        │
│                                                                           │
│  ┌─ Postgres (per-tenant RLS) ─┐  ┌─ Redis cluster ─┐                     │
│  │ profiles, posts, graph,     │  │ feed-cache,     │                     │
│  │ endorsements, jobs, etc.    │  │ react, InMail RB │                     │
│  └─────────────────────────────┘  └──────────────────┘                    │
│  ┌─ S3 (media+doc; KMS) ─────────┐  ┌─ Meilisearch ──┐                    │
│  │ per-tenant prefix isolation   │  │ per-tenant idx │                    │
│  └───────────────────────────────┘  └────────────────┘                    │
│                                                                           │
│  TB4: Professional-context isolation (Personal-tier NEVER federates here) │
│                                                                           │
│  TB5: BC services → audit-chain µservice (Ed25519-signed)                 │
│         Includes per-endorser signature chain (ADR-NET-0005)              │
│                                                                           │
│  TB6: BC services → ontology µservice (Workflow event)                    │
│                                                                           │
│  TB7: Media scan path (OPSWAT/ClamAV; quarantine bucket)                  │
│                                                                           │
│  TB8: Media transcode (ImageMagick + ffmpeg; isolated worker pool)        │
│                                                                           │
│  TB9: InMail-bridge → messenger µservice (Professional-tier-only;         │
│         contract-versioned event; never federates to Personal-tier DM)    │
│                                                                           │
│  TB10: events-bridge → calendar µservice (contract-versioned event)       │
│                                                                           │
│  TB11: pages → mail µservice (newsletter-of-record send)                  │
│                                                                           │
│  TB12: jobs-handoff → ATS µservice (Tier G; contract-versioned event;     │
│         clean boundary per ADR-NET-0004)                                  │
│                                                                           │
│  TB13: Recommender + recruiter-stub ranker (foundry-runtime client) —     │
│         EU AI Act Annex III §4 HIGH-RISK; bias-audit emit per invocation; │
│         GDPR Art. 22 human-review-only path on opt-out                    │
│                                                                           │
│  TB14: Endorsement-chain integrity (Ed25519 per-endorser signature;       │
│         Merkle-style chain; tamper detection via audit-chain replay)      │
│                                                                           │
└───────────────────────────────────────────────────────────────────────────┘
```

Fourteen trust boundaries — extension of the social pattern with Professional-context-specific TB9..TB14 covering InMail-bridge, calendar bridge, mail bridge, ATS handoff, employment-context AI surfaces, and endorsement-chain integrity.

## Assets & Data Classification

| Asset | Class | Retention | Authoritative store |
|---|---|---|---|
| Professional profiles (handle, headline, summary, contact, locale) | `PII_IDENTIFYING` + `EMPLOYMENT_RECORD` | per-pack | Postgres |
| Resume sections (experience, education, skills, certifications) | `EMPLOYMENT_RECORD` (network-specific class) | per-pack labor floor | Postgres |
| Posts (article, status, document, poll, carousel) | `BEHAVIORAL_TENANT_PRODUCT` + sometimes `PII_IDENTIFYING` | per-pack | Postgres (tenant-DEK encrypted when configured) |
| Comments / replies | inherits parent | inherits | Postgres |
| Reactions | `BEHAVIORAL_TENANT_PRODUCT` | 365d hot | Redis (persisted to Postgres asynchronously) |
| Connection graph (directed-bidirectional-on-acceptance edges + block / restrict / disconnect lists) | `RELATIONSHIP_GRAPH` + `EMPLOYMENT_RECORD` + `PII_QUASI_IDENTIFIER` | append-only with tombstone-on-disconnect | Postgres (adjacency-list) |
| Connection requests | `BEHAVIORAL_TENANT_PRODUCT` + `PII_IDENTIFYING` | 90d retained then tombstone | Postgres |
| InMail bodies (Professional-tier-only) | `BEHAVIORAL_TENANT_PRODUCT` + `PII_IDENTIFYING` | per-pack | (lives at messenger µservice; network holds only routing metadata) |
| Endorsements (per-skill, signed Ed25519) | `EMPLOYMENT_RECORD` + `AUDIT` | append-only with revocation | Postgres + audit-chain seal |
| Recommendations (long-form, signed Ed25519) | `EMPLOYMENT_RECORD` + `AUDIT` | append-only with revocation | Postgres + audit-chain seal |
| Skill-assessment scores + passing-badges | `EMPLOYMENT_RECORD` | append-only | Postgres |
| Profile-verification artifacts (ID-attest, employer-confirm) | `PII_IDENTIFYING` + `SENSITIVE_VERIFICATION` (new sub-class) | retention floor per pack | Postgres (separated table; restricted access) |
| Pages (company / brand) | `BEHAVIORAL_TENANT_PRODUCT` | append-only | Postgres |
| Groups + membership | `BEHAVIORAL_TENANT_PRODUCT` + `PII_QUASI_IDENTIFIER` | per-pack | Postgres |
| Events + RSVPs | `BEHAVIORAL_TENANT_PRODUCT` | per-pack | Postgres |
| Job postings | `BEHAVIORAL_TENANT_PRODUCT` + `EMPLOYMENT_RECORD` | per-pack | Postgres (mirrored to ATS via handoff event) |
| Recruiter-search invocations (when enabled) | `AUDIT` + `INTERNAL_ONLY` + `EMPLOYMENT_RECORD` | append-only; immutable; ≥ 6y retention for OFCCP / EEOC | Postgres + audit-chain seal |
| Bias-audit records (per release + per invocation) | `AUDIT` + `EMPLOYMENT_RECORD` | append-only; immutable | foundry-runtime evidence pipeline + audit-chain |
| Media (image / video blobs) + document attachments | `BEHAVIORAL_TENANT_PRODUCT` + sometimes `PII_IDENTIFYING` | per-pack | S3 (KMS-encrypted; per-tenant prefix) |
| Media metadata (digest, preview, transcode variant, scan-verdict) | `INTERNAL_ONLY` + `AUDIT` | append-only | Postgres |
| Notification records | `BEHAVIORAL_TENANT_PRODUCT` + `PII_IDENTIFYING` | 90d hot | Postgres + Redis |
| Trending-topic windows (Professional context only) | derived; `INTERNAL_ONLY` | rebuilt continuously | Redis |
| Search index | derived from profiles + posts + skills + jobs + companies + events | rebuilt from source | Meilisearch (per-tenant index) |
| Moderation verdicts + appeal trail | `AUDIT` + `INTERNAL_ONLY` + `PII_IDENTIFYING` | append-only; immutable | Postgres + audit-chain seal |
| Salary-insights aggregate snapshots | `INTERNAL_ONLY` + `EMPLOYMENT_RECORD` | rebuilt weekly | Postgres |
| Audit-chain seals | `AUDIT` | append-only; immutable | audit-chain µservice |
| Per-tenant DEK | `SECRET` | OpenBao 30d rotation; envelope KMS | OpenBao |
| Per-endorser signing key | `SECRET` | OpenBao-bound; user-controlled | OpenBao (per-user) |
| WebSocket gateway session tokens | `SECRET` | ≤ 24h | OpenBao-issued short-lived JWT |
| Recommender + recruiter model snapshots + training data | `INTERNAL_ONLY` + `AUDIT` + `EMPLOYMENT_RECORD` | per release | foundry-runtime evidence pipeline |
| GDPR Art. 22 opt-out records | `AUDIT` + `PII_IDENTIFYING` | per-user lifetime | Postgres + audit-chain |

## Actors

| Actor | Trust | Auth | Capability |
|---|---|---|---|
| End-user (Professional) | Untrusted external | OIDC + MFA + WSS bearer | Read/write own Professional profile; connect; endorse; recommend; react; comment; apply to jobs |
| Tenant-admin | Semi-trusted internal-to-tenant | OIDC + MFA | Manage tenant config; verification policy; recruiter-stub enable / disable; cannot read InMail body without four-eyes |
| Tenant compliance-officer | Semi-trusted internal-to-tenant | OIDC + MFA + Cedar entitlement | Issue eDiscovery hold (Professional-tier); trigger disclosure (requires four-eyes peer) |
| Tenant moderator | Semi-trusted internal-to-tenant | OIDC + MFA + Cedar entitlement | Manual review of flagged content; verdicts + appeals; KR 직장 갑질 escalation |
| Tenant recruiter | Semi-trusted internal-to-tenant; activated only when recruiter-stub on | OIDC + MFA + Cedar entitlement + EU AI Act FRIA + NYC LL 144 attestation | Invoke recruiter-search ranker against tenant-scoped pool |
| Tenant Page admin | Semi-trusted internal-to-tenant | OIDC + MFA + per-Page entitlement | Manage Page; send newsletter via mail bridge |
| Tenant Group admin | Semi-trusted internal-to-tenant | OIDC + MFA + per-Group entitlement | Manage Group + moderation |
| Tenant security-admin | Semi-trusted internal-to-tenant | OIDC + MFA + Cedar entitlement | Configure pack policy; four-eyes pairing peer |
| oyatie ops-security (human) | Trusted internal | OIDC + MFA + JIT via OpenBao | Admin access; no plaintext PII or InMail body without breakglass + two-person rule |
| Workflow Studio (machine) | Semi-trusted internal | mTLS + SPIFFE | Consume Workflow events; post action-cards via mention-bridge |
| messenger µservice (machine) | Semi-trusted internal | mTLS + SPIFFE | InMail-bridge delivery; never federates to Personal-tier DM |
| calendar µservice (machine) | Semi-trusted internal | mTLS + SPIFFE | Events-bridge delivery |
| mail µservice (machine) | Semi-trusted internal | mTLS + SPIFFE | Page newsletter send |
| ATS µservice (Tier G; machine) | Semi-trusted internal | mTLS + SPIFFE | Receive jobs-handoff; ack referral processing |
| ontology µservice (machine) | Semi-trusted internal | mTLS + SPIFFE | Serve Person/Company/Skill lookups |
| audit-chain µservice (machine) | Trusted internal | mTLS + SPIFFE | Receive seals from every BC; including signed endorsement chain |
| foundry-runtime µservice (machine) | Semi-trusted internal | mTLS + SPIFFE | Provide classifier + recommender + recruiter ranker inference; HIGH-RISK per EU AI Act Annex III §4 |
| External auditor (EEOC / OFCCP / DPA / KR PIPC) | Read-only external | OIDC + MFA + JIT short-lived token | Read tenant-scoped audit-chain seals + bias-audit records + policy artifacts |
| Attacker — opportunistic | Untrusted | none | Scans + low-skill exploitation |
| Attacker — targeted | Untrusted | none | Sophisticated supply-chain awareness; recruiter-ranker influence-op attempts; endorsement-graph manipulation |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure retention or recruiter-stub or bias-audit threshold |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case for confidentiality / integrity / employment-fairness; mitigated by audit-chain + four-eyes + bias-audit |

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
- Mitigations: per-tenant handle namespace; verification-badge + employer-confirm program; handle-revocation policy for impersonation reports; trademark-reservation onboarding option for Professional tenants; Cedar enforces handle-uniqueness scope; LEAN lane verifies.

**T-S-03 — Fake employment-record claim (claiming employment at a company without entitlement)**
- Asset: Employment record + endorsement chain integrity
- Likelihood H / Impact H / Risk **H**
- Mitigations: `profile-verification` BC employer-confirm flow; Page-admin attestation; per-endorser Ed25519 signature; bot-detection at signup; LEAN lane `oya-check-endorsement-chain-integrity` validates signature chain on every replay.

**T-S-04 — Tenant-admin impersonates compliance-officer to trigger Professional-tier disclosure single-handed**
- Asset: Four-eyes disclosure path
- Likelihood L / Impact H / Risk **M**
- Mitigations: four-eyes requires two distinct SPIFFE identities with distinct entitlements + audit-chain seal of both consents; same principal cannot satisfy both halves; replay-resistant nonce.

**T-S-05 — Bot / scripted account masquerades as Professional (sybil)**
- Asset: Engagement signals (endorsements, follows, connection requests)
- Likelihood H / Impact H (employment-record integrity) / Risk **H**
- Mitigations: signup CAPTCHA; rate-limited account creation per IP + per device fingerprint; foundry-guardrails bot-detection classifier; tenant-admin can require email/phone verification + employer-confirm; per-tenant policy controls; sybil-mass-endorsement detector.

**T-S-06 — Forged endorsement (attacker signs an endorsement claiming to be User-A)**
- Asset: Endorsement chain integrity
- Likelihood L / Impact H (CV fraud + EU AI Act fairness pollution) / Risk **M**
- Mitigations: per-endorser Ed25519 signing key bound to OpenBao + user device; endorsement chain replay validates signature against endorser's published public key; tampered or unsigned endorsements rejected; ADR-NET-0005 invariant.

### Tampering (T)

**T-T-01 — Post tampering at rest (Postgres row mutation)**
- Asset: Post store
- Likelihood L / Impact H / Risk **M**
- Mitigations: every post row carries `content_hash` (sha256(plaintext)) emitted to audit-chain on write; periodic batch verifier compares hashes; mismatch quarantines + alerts.

**T-T-02 — Media or document tampering in S3**
- Asset: Media + document blobs
- Likelihood L / Impact H / Risk **M**
- Mitigations: SSE-KMS + S3 Object Lock (WORM) on Professional-tier; content-digest verified at fetch; tamper triggers quarantine; bucket access via service-account IAM only.

**T-T-03 — Connection-graph tampering (mass-connect / mass-disconnect / mass-endorse attack)**
- Asset: Professional-graph integrity + employment-signal integrity
- Likelihood M / Impact H (employment fraud) / Risk **H**
- Mitigations: per-user connection-request rate limit (default 100/week); per-user endorsement rate limit (default 50/day per endorsee skill); audit-chain seal of every connection / endorsement / unfollow; periodic graph-anomaly detector; account suspension on confirmed mass-attack; foundry-guardrails sybil detector wired to connection-edge stream.

**T-T-04 — Reaction-count tampering (vote-stuffing)**
- Asset: Reaction tally
- Likelihood M / Impact L / Risk **L**
- Mitigations: per-user-per-post idempotency (one reaction per user per post per emoji); conflict-free counter; periodic reconciliation Postgres ↔ Redis.

**T-T-05 — Search index poisoning**
- Asset: Meilisearch index
- Likelihood L / Impact M / Risk **L**
- Mitigations: only network-search-worker writes to index; SPIFFE-validated; rebuild from source possible (deterministic).

**T-T-06 — Recruiter-search ranker verdict tampering**
- Asset: Recruiter-search invocation audit + bias-audit record
- Likelihood L / Impact H (EEOC + Title VII regulatory exposure) / Risk **H**
- Mitigations: every recruiter invocation + per-release bias-audit signed by foundry-runtime + sealed via audit-chain Ed25519; tamper triggers per-tenant Sev-1; ADR-NET-0002 invariant; lane `oya-check-eu-ai-act-employment-conformance`.

**T-T-07 — Trending-topic manipulation (artificial trend injection)**
- Asset: Trending-topic ranking (Professional context only)
- Likelihood M / Impact M / Risk **M**
- Mitigations: trend compute uses windowed dedup keyed by `(tenant_id, hashtag, author_ref)`; per-author influence cap; foundry-guardrails sybil detector inputs; tenant-admin can pin / unpin trends.

**T-T-08 — Endorsement chain replay tamper (insert / re-order / drop endorsement)**
- Asset: Endorsement chain integrity
- Likelihood L / Impact H / Risk **M**
- Mitigations: Merkle-style chain via audit-chain µservice; each endorsement carries `prev_endorsement_seal` linking to the prior endorsement's audit-chain seal; replay rejects out-of-order or missing-link entries; ADR-NET-0005.

**T-T-09 — Jobs-handoff event tampering (in-flight to ATS)**
- Asset: Jobs-handoff contract integrity
- Likelihood L / Impact H / Risk **M**
- Mitigations: handoff events signed by network µservice SPIFFE identity; ATS µservice verifies signature on receipt; replay-resistant nonce; per-event audit-chain seal both ends.

### Repudiation (R)

**T-R-01 — User denies authoring a post or recommendation**
- Asset: Post + recommendation authorship
- Likelihood M / Impact M / Risk **M**
- Mitigations: every post carries author SPIFFE identity + session-token nonce + audit-chain seal; recommendations + endorsements carry per-endorser Ed25519 signature; non-repudiable.

**T-R-02 — Tenant-admin denies authorising disclosure**
- Asset: Four-eyes disclosure record
- Likelihood L / Impact H / Risk **M**
- Mitigations: four-eyes requires both consents in audit-chain with distinct principal IDs + reason code; non-repudiable.

**T-R-03 — Recruiter denies invoking recruiter-search ranker on a particular candidate**
- Asset: Recruiter-search invocation audit
- Likelihood L / Impact H (EEOC discovery exposure) / Risk **M**
- Mitigations: every invocation emits Ed25519-signed audit-chain record with recruiter principal + timestamp + scope + per-candidate result; ≥ 6y retention for OFCCP / EEOC.

**T-R-04 — User claims they never opted out of GDPR Art. 22**
- Asset: GDPR Art. 22 opt-out record
- Likelihood L / Impact M / Risk **L**
- Mitigations: opt-out records signed + audit-chain sealed; per-user view of own opt-out history; surfaced in privacy dashboard.

### Information Disclosure (I)

**T-I-01 — Cross-tenant profile / post leak via Postgres RLS misconfiguration**
- Asset: Profile + post store
- Likelihood M / Impact H / Risk **H**
- Mitigations: Postgres Row-Level Security with `tenant_id = current_setting('app.tenant_id')`; gateway sets the GUC per connection; LEAN check `oya-check-postgres-rls-coverage` asserts RLS enabled on every network table; pen-test annually.

**T-I-02 — InMail body leak (admin disclosure or tenant pivot)**
- Asset: InMail body
- Likelihood L / Impact H / Risk **M**
- Mitigations: InMail body lives at messenger µservice (network holds only routing metadata); disclosure requires four-eyes + audit-chain seal at both µservices; ADR-NET-0003 contract bounds.

**T-I-03 — Employment-record leak via tenant-admin pivot to read end-user's full Professional history**
- Asset: Resume sections (experience, education, skills, certifications)
- Likelihood M / Impact H / Risk **H**
- Mitigations: Cedar `policy/tenant-scope.cedar` bounds tenant-admin reads to consented Professional-context scope; per-user privacy settings honor "make profile invisible to tenant-admin" toggle; LEAN lane verifies; cross-tenant pivot forbidden.

**T-I-04 — Connection-graph pivoting reveals relationship graph (Article 9 sensitive interpretation)**
- Asset: Professional-graph
- Likelihood M / Impact M / Risk **M**
- Mitigations: graph reads bounded by Cedar (own-graph + public-facing-1st-degree-of-public-profile only); cross-tenant graph enumeration forbidden; per-tenant cardinality limits; aggregate stats only for non-owners; degree-of-separation computed at usecase, not exposed raw.

**T-I-05 — Search-result leak: returns posts / profiles / jobs user cannot read**
- Asset: Search results
- Likelihood M / Impact H / Risk **H**
- Mitigations: search post-filters by Cedar evaluation; result set redacted to caller-scope; integration test asserts no over-permitted result.

**T-I-06 — Media URL or document URL leak via shared-link guess**
- Asset: Media + document URL
- Likelihood M / Impact H / Risk **H**
- Mitigations: media + document URLs are signed short-TTL (≤ 15 min); per-fetch Cedar re-evaluation; public-Page media uses Cedar-checked CDN URL; private posts require signed URL per fetch.

**T-I-07 — Personal-tier `social` post accidentally surfaces as Professional `network` content**
- Asset: Professional-context isolation
- Likelihood L / Impact H (regulatory + privacy breach) / Risk **H**
- Mitigations: data-model invariant — `network` types are Professional-only; no shared type with `social`; LEAN-lane `oya-check-professional-context-isolation` validates type signatures forbid cross-context flows (ADR-NET-0001).

**T-I-08 — Recruiter-search reveals candidates outside tenant scope**
- Asset: Recruiter scope
- Likelihood L / Impact H / Risk **M**
- Mitigations: recruiter-search bounded by Cedar entitlement scoped to tenant; LEAN lane verifies recruiter scope policy; pen-test before recruiter-stub activation.

**T-I-09 — Notification metadata leak (notification reveals private InMail or private post body to non-recipient)**
- Asset: Notification body
- Likelihood M / Impact M / Risk **M**
- Mitigations: notification bodies carry only redacted previews + signed deep-link; full body fetched at click-time with Cedar re-eval; private-visibility content omits body in notification.

**T-I-10 — Profile-verification artifacts (ID-attest) leaked via table pivoting**
- Asset: ID attestations (national ID number, passport scan, etc.)
- Likelihood L / Impact H / Risk **H**
- Mitigations: separate `network_verification_attestations` table; access bound to Cedar `verification_reader` entitlement (rare; only verification-flow + compliance-officer); no general staff read; encryption at rest with separate DEK; ADR-NET-0001-DR threat-model row.

**T-I-11 — Salary-insights aggregate disclosure de-anonymises an individual**
- Asset: Salary-insights aggregate
- Likelihood M / Impact M / Risk **M**
- Mitigations: aggregate-only; k-anonymity ≥ 5 enforced; per-individual disclosure refused at usecase layer; LEAN lane verifies; ADR-NET-0001 boundary.

**T-I-12 — Endorsement aggregation leaks endorser identity to non-followers**
- Asset: Endorsement chain
- Likelihood L / Impact L / Risk **L**
- Mitigations: endorser identity disclosed only when endorsee makes the endorsement public; per-endorsement visibility honored at search + display.

**T-I-13 — Jobs-handoff event leaks candidate PII to ATS µservice beyond consented scope**
- Asset: Jobs-handoff payload
- Likelihood M / Impact H / Risk **H**
- Mitigations: handoff payload scoped to minimum-necessary (candidate ref + consent token + application metadata); contract-versioned per ADR-NET-0004; ATS µservice attests scope at receipt.

### Denial of Service (D)

**T-D-01 — Feed-render storm: viral Professional article causes mass concurrent feed-pulls**
- Asset: Feed cache
- Likelihood M / Impact H / Risk **H**
- Mitigations: Redis hot-feed cache; fanout-on-write for hot accounts (precomputed feed); per-tenant feed-render rate limit; HPA on REST pods; runbook `runbooks/feed-cache-rebuild.md`.

**T-D-02 — Endorsement storm: one user receives thousands of sybil endorsements**
- Asset: Endorsement worker queue + endorsement-chain throughput
- Likelihood M / Impact M / Risk **M**
- Mitigations: per-endorsee per-skill rate cap; sybil detector at foundry-guardrails; per-tenant policy controls; runbook `runbooks/endorsement-storm-throttle.md`.

**T-D-03 — InMail flood: mass-InMail abuse (cold-outreach spam)**
- Asset: InMail-bridge throughput + recipient inbox
- Likelihood H / Impact M / Risk **H**
- Mitigations: per-sender InMail rate budget (default 100 / month for free-tier; tenant-configurable for premium); spam-classifier on every InMail; per-recipient block-sender list honored; runbook `runbooks/inmail-fanout-degraded.md`.

**T-D-04 — Notification fanout storm: a Professional account with 300k followers posts**
- Asset: notification worker queue
- Likelihood H / Impact M / Risk **M**
- Mitigations: sharded notification workers; per-recipient idempotent processing; coalesce digest for low-priority notifications; backpressure-throttle to per-recipient at hot-window.

**T-D-05 — Media-transcode storm (mass-upload abuse)**
- Asset: ImageMagick + ffmpeg worker pool
- Likelihood M / Impact H / Risk **H**
- Mitigations: per-tenant upload rate limit; queue depth bound; worker pool sized per capacity model; sandbox per T-E-05.

**T-D-06 — Trending-topic poisoning (sybil-amplified hashtag)**
- Asset: Trending-topic ranking integrity
- Likelihood M / Impact M / Risk **M**
- Mitigations: foundry-guardrails sybil detector; per-author influence cap in trending; tenant-admin pin/unpin.

**T-D-07 — Jobs-handoff ATS-side flood: tenant publishes 100k jobs in burst**
- Asset: ATS µservice handoff endpoint
- Likelihood L / Impact M / Risk **L**
- Mitigations: per-tenant jobs-publish rate limit; ATS µservice backpressure honored; runbook `runbooks/jobs-handoff-ats-failure.md`.

**T-D-08 — Postgres ingest spike causes connection-action latency breach**
- Asset: Connection-graph store
- Likelihood M / Impact H / Risk **H**
- Mitigations: per-tenant ingest rate limit; bulk-write buffering; HPA scale-out; sharding past per-cell capacity threshold.

### Elevation of Privilege (E)

**T-E-01 — Cedar policy bug grants recruiter entitlement to non-recruiter**
- Asset: Cedar evaluator
- Likelihood L / Impact H / Risk **M**
- Mitigations: Cedar v4.2 LTS; fragment fuzz; integration test asserts no over-permitted action; periodic Cedar-fragment-coverage CI lane.

**T-E-02 — Compromised tenant-moderator pivots to read all tenant posts + InMails**
- Asset: Moderator scope
- Likelihood L / Impact H / Risk **M**
- Mitigations: moderator scope bounded to flagged-only reads (must come from `AbuseReportFiled` or `HarassmentReportFiled` event); cannot read arbitrary posts or InMails; LEAN lane verifies moderator scope policy.

**T-E-03 — Recruiter-stub activation without NYC LL 144 / CA AB-331 bias-audit pre-condition**
- Asset: Recruiter-search authorisation
- Likelihood M / Impact H (regulatory exposure) / Risk **H**
- Mitigations: activation requires pre-condition green check (bias-audit attestation on file); LEAN lane `oya-check-recruiter-stub-activation-prerequisite` validates; CI refuses recruiter activation PR without attestation.

**T-E-04 — Media scanner bypasses scan path**
- Asset: Quarantine boundary
- Likelihood L / Impact H / Risk **M**
- Mitigations: blob lifecycle: PUT → quarantine bucket → scanner → on-clean copy to production bucket; production bucket write-only by scanner SA.

**T-E-05 — ImageMagick / ffmpeg CVE allows RCE in transcode worker**
- Asset: Transcode worker pool
- Likelihood M / Impact H / Risk **H**
- Mitigations: transcode workers run in gVisor / Kata Container sandbox; non-root; read-only root FS; no network egress except to S3 quarantine + production; LTS pin (ImageMagick 7.1, ffmpeg 7.x); weekly CVE scan via Trivy + Grype.

**T-E-06 — Endorser signing-key compromise leads to bulk forged endorsements**
- Asset: Endorser Ed25519 signing keys
- Likelihood L / Impact H / Risk **M**
- Mitigations: keys OpenBao-bound; per-device attestation; revocation flow when device lost; replay-resistant nonce on every signature; periodic key rotation; ADR-NET-0005 covers.

## LINDDUN Privacy-Threat Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | Connection-graph | Cross-tenant correlation could re-identify users across packs | Per-tenant scope; Cedar; cross-tenant federation forbidden | L |
| T-L-02 | Identifiability | Profile + employment record | Combination uniquely identifies a Professional | Per-tenant handle namespace; explicit consent at signup | L |
| T-L-03 | Non-repudiation | Endorsement signature | User cannot deny signed endorsement | Acceptable per GDPR Art. 5(2); explicit at endorse-time | L |
| T-L-04 | Detectability | Job-search activity | Job-search activity reveals job-seeking intent — high-sensitivity under EU labor law | Job-search activity NEVER disclosed to current employer (Page admin scope refuses); per-tenant Cedar; warning on profile-view by current-employer Page admin | M |
| T-L-05 | Disclosure | Compliance hold reveals Professional post + InMail bodies to admin | Hold + four-eyes disclosure inherently exposes Professional bodies to tenant admin | Mitigated to acceptable: four-eyes + audit-chain + reason code + tenant disclosure obligation | M |
| T-L-06 | Unawareness | End-user (tenant's user) | End-user may not know tenant-admin can disclose Professional posts + InMail under four-eyes | Tenant DPA includes disclosure clause; tenant onboarding notice required | M |
| T-L-07 | Non-compliance | GDPR Art. 17 right-to-erasure | User requests erasure across all posts + endorsements + connections + recommendations + jobs-applications | DSR cascade marks rows tombstoned + redacts identifiers; 30d SLA; endorsement chain preserved with redacted identifier per ADR-NET-0005 | M |
| T-L-08 | Linkability | Endorsement history pivot | A user's endorsements across people form a behavioral profile | Endorsement visibility honors endorsee + endorser settings; default scope per pack | L |
| T-L-09 | Identifiability | Profile-verification badge + employer-confirm | Badge ties a real-world identity + employer to a handle | Acceptable; verified-handle is intentionally disclosive; only with user's verification request | L |
| T-L-10 | Disclosure | EU AI Act Art. 50 + Art. 27 transparency obligation | Recruiter ranking + jobs ranking + endorsement aggregation output must be disclosed to user | Every high-risk decision carries `eu_ai_act_label`; ranking explanation API per Art. 27; per-user opt-out path per Art. 22 | L |
| T-L-11 | Non-compliance | KR 직장 갑질 abuse not routed correctly | Workplace-harassment report buried in general moderation queue | Dedicated `harassment-workplace` abuse category; elevated severity; tenant ops-security notified; KR-specific runbook | L |
| T-L-12 | Disclosure | Salary-insights de-anonymises individual | Salary aggregate at small-cell reveals individual salary | k-anonymity ≥ 5 enforced; aggregate-only; LEAN lane | L |
| T-L-13 | Linkability | Cross-µservice mention identity correlation | A `network` mention + a `social` mention of the same physical user across µservices form a linkage | Per-µservice ontology scope; cross-µservice correlation forbidden via Ontology Cedar; ADR-NET-0001 (Professional-context-isolation) covers cross-µservice surface as well | L |
| T-L-14 | Disclosure | NYC LL 144 / CA AB-331 mandatory candidate-notice | Candidate must be informed when recruiter-search ranker is used; failure to notify is a per-candidate fine | Activation pre-condition includes candidate-notice template; per-invocation notice emitted; bias-audit on file | L |

## Mitigations Catalog

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Postgres RLS on every network table | Preventive | axis-network | `oya-check-postgres-rls-coverage` lane |
| Per-connection short-lived JWT bound to device + IP | Preventive | axis-network | gateway audit log |
| Four-eyes disclosure with distinct principal IDs | Preventive | axis-network + ops-security | integration test |
| Cedar policy on every read/write | Preventive | axis-network | LEAN coverage lane |
| Media + document scan + quarantine workflow | Preventive | axis-network | end-to-end test |
| Professional-context type invariant (Personal `social` types ≠ Professional `network` types) | Preventive | axis-network | `oya-check-professional-context-isolation` lane |
| Audit-chain Ed25519 seal on every state transition | Detective + Non-repudiation | audit-chain | regression tests |
| Per-endorser Ed25519 signed endorsement chain | Preventive + Non-repudiation | axis-network + axis-audit-chain | `oya-check-endorsement-chain-integrity` lane |
| Per-tenant rate + cardinality limits | Preventive (DoS) | axis-network | gateway + Postgres metrics |
| DSR cascade for right-to-erasure (including endorsement chain redaction) | Preventive (compliance) | council-privacy | DSR dashboard SLO |
| EU AI Act Annex III §4 transparency record per recruiter + jobs-ranking + endorsement-aggregation invocation | Preventive (compliance) | axis-network + axis-foundry-runtime + council-privacy | `capabilities/T2-auto.yaml` evidence pipeline; `oya-check-eu-ai-act-employment-conformance` lane |
| GDPR Art. 22 right-to-human-review per-decision opt-out | Preventive (compliance) | axis-network + council-privacy | per-user privacy dashboard |
| EEOC + Title VII + ADA + ADEA bias-audit per release (4/5-rule) | Preventive (compliance) | axis-foundry-runtime + ops-compliance | per-release bias-audit + dashboards |
| NYC LL 144 + CA AB-331 + CO SB-205 activation pre-condition gate | Preventive (compliance) | ops-legal + axis-network | `oya-check-recruiter-stub-activation-prerequisite` lane |
| KR 직장 갑질 dedicated category routing | Preventive (compliance) | axis-network + council-privacy | abuse-routing test |
| Jobs-handoff contract-versioned event + per-event audit-chain seal both ends | Preventive | axis-network + axis-ats | `oya-check-jobs-handoff-contract` lane |
| InMail-bridge Professional-tier-only routing (never to Personal-tier DM) | Preventive | axis-network + axis-messenger | type-system invariant + LEAN lane |
| ImageMagick / ffmpeg sandboxed workers (gVisor / Kata) | Preventive | axis-network + cloud-k8s | IaC enforces; LEAN lane verifies |
| Salary-insights k-anonymity ≥ 5 | Preventive (privacy) | axis-network + council-privacy | usecase test |
| Profile-verification (ID-attest) artifact isolation table + entitlement | Preventive (privacy) | axis-network + ops-security | LEAN lane |

## Residual Risk Acceptance

| Risk ID | Residual | Why | Re-review |
|---|---|---|---|
| T-L-04 (job-search detectability via current-employer) | M | Best-effort policy; perfect concealment requires omitting job-search-related signals from profile altogether | Annually |
| T-L-05 (hold disclosure inherent) | M | Four-eyes + audit are load-bearing controls; user-side opacity unavoidable | Annually |
| T-L-06 (end-user unawareness) | M | Joint-controllership clause | Annually |
| T-L-07 (erasure best-effort; chain redaction) | M | Audit immutability tradeoff; chain redaction replaces identifier with `«erased»` | Annually |
| T-D-06 (trending-topic poisoning) | M | Sybil detector + tenant-admin pinning; some residual risk unavoidable in any open Professional platform | Quarterly |
| T-T-03 (mass-connect / mass-endorse) | M-H | Rate limits + sybil detector; persistent threat in employment-record-integrity space | Quarterly |
| T-E-03 (recruiter-stub activation without pre-condition) | L | Activation gate is BLOCKER; residual is mis-installation | Per activation |

Sign-off:
- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`
- ops-compliance: `pending`
- ops-legal: `pending`

## Per-Pack Overlay Sections

### pack-kr

- KR PIPA Art. 23 sensitive personal info — sensitive profile fields (race, religion, health, political views) require additional consent.
- KR 근로기준법 — employment record retention floor (3 years general; 5 years where labor incident involved).
- KR 직장 갑질 protections — dedicated harassment-workplace abuse category with elevated routing.
- KR 통신비밀보호법 — InMail intercept only via four-eyes audit.
- KR 채용절차의 공정화에 관한 법률 (Fair Hiring Procedures Act) — when recruiter-stub activated, candidate-notice + bias-audit on file.

### pack-us

- EEOC UGESP 1978 + Title VII Civil Rights Act 1964 — 4/5-rule disparate-impact monitoring on recruiter ranker + jobs ranker.
- ADA 1990 — accessibility-captions enforcement; WCAG 2.2 Level AA conformance.
- ADEA 1967 — age-related disparate-impact monitoring; opt-out per Art. 22.
- OFCCP regulations (federal contractor tenants) — recruiter audit ≥ 6y retention.
- CCPA + CPRA — California opt-out + right-to-delete cascaded.
- NYC AI Hiring Law (Local Law 144-2021) — annual bias audit + candidate notice when recruiter-tooling active for NYC tenants.
- CA AB-331 — automated-decision transparency obligations.
- CO SB-205 — Colorado AI Act developer + deployer obligations.
- IL AI Video Interview Act — video-interview features (if added later) require consent + storage limits.

### pack-us-healthcare

- HIPAA §164.502(b) minimum-necessary — Professional posts that surface health-context default to PHI-redactor.
- HIPAA §164.312(b) audit controls — audit-chain ≥ 6y retention overlay.
- Per-tenant BAA at `microservices/network/legal/baa-template.md` (Slice B).

### pack-eu

- GDPR Art. 8 — child consent (not applicable to Professional network; minor accounts blocked at signup).
- GDPR Art. 21 — right to object to profiling.
- GDPR Art. 22 — right to human review of automated decisions (recruiter + jobs ranking + endorsement aggregation).
- GDPR Art. 25 — privacy-by-design via Professional-context invariant.
- GDPR Art. 32 — every mitigation above contributes.
- GDPR Arts. 44-50 — pack-eu data stays in EU pack.
- EU AI Act 2024/1689 Annex III §4 — recruiter + jobs ranking + endorsement aggregation classified HIGH-RISK; Arts. 9-15 + 27 + 50 + 73 operative.
- EU Equal Treatment Directives 2000/43/EC (racial / ethnic origin) + 2000/78/EC (employment) — disparate-impact monitoring on all employment-context AI.
- ePrivacy Directive Art. 5(3) — confidentiality of communications (covers InMail-bridge metadata).
- UK Equality Act 2010 + ICO ADM guidance — UK-tenant overlay.
- Council of Europe Convention 108+ — broader processing protections.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per pack overlay at `regional-packs/<pack>/network-overlay.md`; cross-mapped via compliance.md.

## Compliance Cross-Mapping

| Framework | Coverage | Mapping doc |
|---|---|---|
| SOC 2 Type 2 | CC1–CC9 covered in `compliance.md` |
| ISO 27001:2022 | A.5–A.8 covered |
| ISO 30414:2018 (HR analytics) | Aggregated workforce-reporting controls covered |
| GDPR | Arts. 5, 6, 8, 9, 13, 14, 17, 21, 22, 25, 28, 30, 32, 33, 35 covered in `dpia.md` + `compliance.md` |
| EU DSA | Arts. 14, 16, 17, 20, 23, 24 covered in `compliance.md` |
| EU AI Act | Arts. 9, 13, 14, 15, 27, 50, 73 covered via `capabilities/T2-auto.yaml` + ADR-NET-0002 |
| EU Equal Treatment Directives | 2000/43/EC + 2000/78/EC covered via bias-audit lane |
| EEOC + Title VII + ADA + ADEA | Covered via bias-audit lane + ADR-NET-0002 |
| NYC LL 144 + CA AB-331 + CO SB-205 | Covered via recruiter-stub activation pre-condition lane |

## Re-review Triggers

- Any new BC.
- Any change to Professional-context-isolation invariant.
- Any new media scanner / transcoder.
- Any Cedar fragment change.
- Annual scheduled review.
- Post-incident review (any Sev-1 or Sev-2).
- Pen-test or audit finding.
- New classifier / recommender / recruiter ranker version deployed (EU AI Act re-evaluation).
- Recruiter-stub activation on a new tenant (NYC LL 144 + CA AB-331 + CO SB-205 cross-check).
- Jobs-handoff contract version bump (per ADR-NET-0004).

## References

- Parallel ADR-0135 (Connect dissolution; network as a sibling µservice; distinct from social).
- Bominal ADR-0028, ADR-0111, ADR-0208, ADR-0215.
- ADR-0008 Data Use Boundary.
- ADR-NET-0001 through ADR-NET-0006.
- `microservices/network/PRD.md`.
- `microservices/network/dpia.md`.
- `microservices/network/compliance.md`.
- `microservices/network/policy/professional-context-isolation.md`.
- `microservices/social/threat-model.md` (paired pattern reference).
- `microservices/messenger/threat-model.md` (paired InMail-bridge reference).
- OWASP API Top 10 (2023); NIST SP 800-154; NIST AI RMF 1.0.
- EU DSA 2065/2022; EU AI Act 2024/1689 Annex III §4.
- EU Equal Treatment Directives 2000/43/EC + 2000/78/EC.
- EEOC UGESP 1978; Title VII Civil Rights Act 1964; ADA 1990; ADEA 1967.
- NYC Local Law 144-2021; CA AB-331; CO SB-205; IL AI Video Interview Act.
- KR PIPA + 근로기준법 + 직장 갑질 protections + 통신비밀보호법 + 채용절차의 공정화에 관한 법률.
