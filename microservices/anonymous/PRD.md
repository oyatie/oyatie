---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-anonymous
microservice: anonymous
status: Accepted
sales_segment: connect-suite-product
tier: hero-product
milestone_first_ship: M02-foundation
bominal_source: []
related_adrs: [ADR-0008, ADR-0056, ADR-0105, ADR-0106, ADR-0135, ADR-0130, ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/microservices/anonymous.json, /specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
owner_team: axis-anonymous
doc_status: published
---

# PRD-anonymous: Privacy-Pseudonymous Posting Tier (Sidechat / YikYak / Whisper / Blind-class)

## Purpose

The `anonymous` microservice is oyatie's native pseudonymous-and-affinity-bound posting platform. Per parallel ADR-0135 (Connect dissolution), it is one of the first-class µservices factored out of the legacy Connect umbrella, distinguished from `social` and `community` by a single axiom: **the platform cannot link a post to a real identity except under a Cedar-gated legal-process workflow**. It owns **pseudonymous-identity (rotating-handle-per-channel) + affinity-attestation (employer/edu/geo/workspace verification without identity disclosure) + post-thread (ephemeral text-first) + chronological feed + community-vote (upvote/downvote) + comment/reply tree + anonymous-DM (MLS E2E optional) + report-and-moderate + content-moderation (mandatory; EU AI Act low/medium-risk transparency obligations) + legal-process disclosure (court-ordered identity reveal with dual-control) + post-deletion and tombstone + geo-bound + employer-bound + university-bound communities + hashtags + within-affinity trending + notifications (no real-name push) + accessibility-captions + abuse-reporting + age-gate (per pack) + short retention (30/60/90-day tenant-selectable)** across the 11 oyatie regulatory packs.

This µservice is **a hero product**, end-user-facing through Workflow Studio shell and standalone anonymous-client surfaces (web + desktop + mobile). It is consumable as a shared substrate by other oyatie products via `anonymous.post.v1` Workflow events and the `Affinity` Ontology object type **only**. **The platform never writes a `Person` Ontology entity from `anonymous`** — that would defeat invariant I1.

**anonymous is NET-NEW** per parallel ADR-0135. No `oya-connect-anonymous-*` crates exist; there is no migration-from-connect.md or deprecation-notice.md.

## Design Invariants (LOAD-BEARING)

These seven invariants are the basis on which every downstream ADR, policy, schema, and runbook is authored. Any change to them requires a superseding ADR-ANON-* and explicit council-architecture + council-privacy + ops-security sign-off.

| ID | Invariant | Enforcement |
|---|---|---|
| **I1** | The platform CANNOT correlate `user_id ↔ post_id` outside the legal-process workflow. Posts are bound to a per-session blind-signed credential; the credential's issuer never sees the post body, and the post store never sees the issuer's identifier. | Cryptographic blinding (ADR-ANON-0001); database schema separation; LEAN lane `oya-check-blinding-column-isolation`; ADR-ANON-0003 legal-process Cedar gate |
| **I2** | Affinity attestation reveals AFFINITY (employer / edu / geo) NEVER IDENTITY. "You are a Bominal employee" without revealing WHICH Bominal employee. | ADR-ANON-0002 zero-knowledge attestation flow; BBS+ selective disclosure (W3C VC 2.0); `policy/affinity-attestation-verification.md` |
| **I3** | All retention defaults short (30 days). User-initiated deletion is hard-delete + audit-chain tombstone within p99 ≤ 5s propagation. | ADR-ANON-0004; `slos/hard-delete-propagation-correctness.openslo.yaml`; LEAN lane `oya-check-retention-default-short` |
| **I4** | No third-party analytics SDKs ever. No Google Analytics. No Segment. No Mixpanel. No Amplitude. No vendor pixel of any kind in the client SDK or web bundle. | LEAN lane `oya-check-third-party-tracker-refused` (mandatory P01 day-1); Helm chart forbids egress to any non-allowlisted domain |
| **I5** | Federation NEVER. No ActivityPub, no AT Protocol, no Matrix, no XMPP. Federation would defeat I1 because peer servers cannot be bound to the legal-process Cedar gate. | ADR-ANON-0006 compile-time refusal; no `federation-gateway` BC in this µservice (contrast with `social` which has one OFF-by-default) |
| **I6** | Anonymous-DM uses MLS (RFC 9420) end-to-end. Platform NEVER holds plaintext; key material lives in client keystore + per-channel sender keys. | ADR-MSGR-0002 inherited pattern; ADR-ANON-0001 binding to blind-signed handle; LEAN lane `oya-check-e2e-no-plaintext-server-state` |
| **I7** | Legal-process disclosure requires (a) Cedar `legal-process` policy authorisation, (b) dual-control (two-person rule, distinct principals, distinct entitlements), (c) 14-day end-user notice unless court-prohibited (gag-order doctrine per ECPA SCA §2705(a)/(b); UK IPA 2016 §57; KR 통신비밀보호법 Art. 9-2), (d) audit-chain seal with chain-of-custody hash, (e) inclusion in the quarterly transparency report (numerical only when court-prohibited). | ADR-ANON-0003; `policy/legal-process-disclosure.cedar`; `runbooks/legal-process-court-order-receipt.md` |

## Tenant Value

- **Tenant Outcome 1 — Native pseudonymous-and-affinity-bound posting.** Tenants and their end-users get Sidechat/YikYak/Whisper/Blind-class UX inside the same shell as mail, social, community, messenger, calendar, workflow studio — switching pseudonymous personas without leaking identity across channels.
- **Tenant Outcome 2 — Privacy-by-design surface.** Cryptographic blinding ensures the platform itself cannot answer "who wrote this post" without a court order. Tenants offering this surface to their end-users get a structural privacy promise no competitor can match (Sidechat / YikYak ship "anonymous" but platform operators retain access).
- **Tenant Outcome 3 — Affinity authenticity without identity disclosure.** Employer-bound communities ("Bominal-employees-only" channel) verifiable cryptographically without the platform learning which employee.
- **Tenant Outcome 4 — Short-retention-as-privacy.** 30-day default; user-deletion is hard-delete + audit-chain tombstone within p99 ≤ 5s.
- **Tenant Outcome 5 — Audit-grade legal-process disclosure.** Court-ordered identity disclosure is dual-controlled, time-bounded, audit-chain-sealed, and included in transparency reports — unlike competitors that handle this ad hoc.
- **Tenant Outcome 6 — Multi-pack residency by design.** 11 region-pinned packs; pack-kr (통신비밀보호법), pack-eu (GDPR Art. 11 Recital 26), pack-us (Section 230 + state anti-doxxing), pack-uk (OSA 2023 + IPA 2016), pack-jp (通信の秘密) all served from origin pack.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | end-user | to obtain a per-session pseudonymous handle bound to an affinity (employer / edu / geo / workspace) without revealing my identity | I can post under anonymity backed by attestation | pseudonymous-identity + affinity-attestation | Must |
| FR-02 | end-user | to rotate my handle per channel | linkability across channels is broken | pseudonymous-identity | Must |
| FR-03 | end-user | to publish a post (text ≤ 4096 chars; default text-only) | I share a thought anonymously | post-thread | Must |
| FR-04 | end-user | to comment on a post / reply in a thread | conversation depth is preserved under anonymity | post-thread | Must |
| FR-05 | end-user | to upvote / downvote a post or comment | community-curation works | upvote-downvote | Must |
| FR-06 | end-user | to see a chronological feed within my affinity community | I consume what's recent + relevant to my bound affinity | feed-timeline | Must |
| FR-07 | end-user | to see (optionally) an algorithmic ranked feed within my affinity | I get the best content first | feed-timeline | Should |
| FR-08 | end-user | to use #hashtags for discoverability inside the affinity scope | content is grouped by topic | hashtags | Must |
| FR-09 | end-user | to see within-affinity trending topics | discovery surfaces within-cohort context | trending | Must |
| FR-10 | end-user | to receive notifications (without real-name in push payload) | I stay engaged without privacy leak | notifications | Must |
| FR-11 | end-user | to send / receive anonymous direct messages with E2E (MLS) encryption | private conversation works without server plaintext | anonymous-dm | Should |
| FR-12 | end-user | to report a post for abuse / harassment / impersonation / hate / CSAM-suspect | community safety is maintained | report-and-moderate | Must |
| FR-13 | end-user | to delete my own post / comment (hard-delete + audit-chain tombstone) | I retract retroactively | post-thread + retention-policy | Must |
| FR-14 | end-user (per pack regulation) | to attest age at signup (COPPA <13 banned; pack-eu GDPR Art. 8 minor threshold) | age-gate is enforced | age-gate | Must |
| FR-15 | tenant-admin | to configure pack-aware retention policy (30/60/90-day tier) | regulatory bounds hold | retention-policy | Must |
| FR-16 | tenant-admin | to define an affinity community (employer / edu / geo bound) with cardinality + minimum-population for k-anonymity | community has structural privacy floor | affinity-attestation + community-definition | Must |
| FR-17 | tenant-admin | to receive a court-ordered legal-process disclosure request, dual-approve, gate via Cedar, audit-chain seal, deliver to law-enforcement | legal-process compliance + tenant audit | legal-process-disclosure | Must |
| FR-18 | end-user | to view the platform's quarterly transparency report (legal-process volume by category + jurisdiction) | trust signal | legal-process-disclosure | Must |
| FR-19 | content-moderator | to act on classifier verdicts + abuse reports | platform safety is operationalised | content-moderation | Must |
| FR-20 | end-user | to appeal a moderation verdict | due-process is honoured | content-moderation | Must |
| FR-21 | end-user (geo-affinity) | to see geo-bound posts within my pack | hyperlocal discovery (Sidechat / YikYak class) | community-definition | Must |
| FR-22 | end-user (employer-affinity) | to participate in employer-bound community (Blind-class) | workplace conversation under anonymity | community-definition | Must |
| FR-23 | end-user (university-affinity) | to participate in university-bound community (Sidechat-class) | campus conversation under anonymity | community-definition | Must |
| FR-24 | Workflow Studio | to consume `AnonymousPostPublished` / `ModerationVerdictEmitted` / `LegalProcessDisclosureExecuted` events (with author-blinded identifiers only) | downstream automation works without leaking identity | every BC | Must |
| FR-25 | tenant-operator | to query feed-render + post-create + vote + moderation metrics | I plan capacity + verify SLAs | observability (cross-µservice) | Must |
| FR-26 | end-user | to expect that my post never federates outside the affinity boundary | I-5 invariant preserved | (none — no federation BC) | Must (refusal) |
| FR-27 | abuse-classifier | to flag CSAM-suspect at p95 ≤ 200ms | NCMEC reporting SLA per 18 USC §2258A | content-moderation | Must |
| FR-28 | end-user | to expect that no third-party analytics SDK is ever loaded | I-4 invariant preserved | (none — refused by build) | Must (refusal) |
| FR-29 | end-user | to attach a (limited) image with abuse-scan applied (T2 capability; off by default per tenant) | sparingly-attachment use case | post-thread (optional adapter-s3) | Should |

## Non-Functional Requirements

### Performance

| Metric | p50 | p95 | p99 | p999 | Notes |
|---|---|---|---|---|---|
| Feed-render latency (top 50 posts within affinity) | ≤ 80ms | ≤ 250ms | ≤ 500ms | ≤ 1.2s | Redis hot-feed cache + Cedar policy filter |
| Post-create latency (post → durable + indexed) | ≤ 30ms | ≤ 100ms | ≤ 250ms | ≤ 700ms | Postgres insert + async fanout |
| Vote-action latency | ≤ 15ms | ≤ 30ms | ≤ 50ms | ≤ 150ms | Redis-buffered counter; Postgres flush |
| Affinity-attestation-verify latency (cryptographic) | ≤ 150ms | ≤ 500ms | ≤ 1s | ≤ 2s | BBS+ verify; cached per session |
| Abuse-classifier inference latency | ≤ 80ms | ≤ 200ms | ≤ 400ms | ≤ 1s | foundry-runtime; batched 100 |
| Hard-delete propagation (post deleted → no replica returns it) | ≤ 1s | ≤ 3s | ≤ 5s | ≤ 10s | RPO across all read paths |
| Legal-process disclosure E2E (court-order received → law-enforcement-delivery sealed) | n/a | n/a | n/a | n/a | manual workflow; SLA per `runbooks/legal-process-court-order-receipt.md` |
| Comment / reply create | ≤ 30ms | ≤ 100ms | ≤ 250ms | ≤ 700ms | Postgres insert |
| Search hashtag (within affinity) | ≤ 80ms | ≤ 300ms | ≤ 600ms | ≤ 1.2s | Meilisearch + Cedar filter |
| Notification fanout (≤ 5k affinity members) | ≤ 100ms | ≤ 500ms | ≤ 1s | ≤ 3s | per-recipient async via Redis Streams; push payloads ALWAYS use opaque handle |

### Security

- Post + vote + comment writes enforced server-side via Cedar policy (`policy/tenant-scope.cedar` + `policy/legal-process-disclosure.cedar`); client never trusted.
- Posts in Postgres are bound to a *blinded* author commitment (cryptographic commitment under the blind-signature protocol of ADR-ANON-0001); no row in `anonymous.post` ever stores `user_id` directly.
- Affinity attestation is BBS+-selective-disclosure (W3C VC 2.0); the platform learns the affinity (employer-domain, edu-domain, geo-bucket) but not the verifying credential's holder identifier.
- All client → server traffic mTLS-terminated; per-session Sphinx-like onion pad supported for the strictest pack-eu DPIA option (see DPIA R-04).
- T2-attachments (when tenant-enabled): OPSWAT MetaDefender / ClamAV scan in gVisor-sandboxed worker before publication; quarantine bucket pattern.
- Search index: hashtag corpus only — never per-author corpus, never per-post-author cross-reference field.
- Tenant operators + oyatie operators MUST NOT have plaintext disclosure access except through the legal-process Cedar policy + dual-control flow.
- No third-party analytics SDK; LEAN lane refuses if any new dependency carries known-tracker fingerprint (Google Analytics, Segment, Mixpanel, Amplitude, Heap, FullStory, etc.).
- Push-notification payloads NEVER include real-name; payload is opaque-handle + verb + permalink-hash only.

### Audit + Compliance

- Every post-create / post-delete / vote-action / moderation-verdict / appeal-action / legal-process-disclosure / affinity-attestation-binding event writes an audit-chain record (Merkle / Ed25519 per Bominal ADR-0028).
- Legal-process disclosure requires two distinct approving principals (each holding `legal_process_approver` Cedar entitlement) per ADR-ANON-0003.
- Retention: per-pack bounds in `policy/data-residency.md` + `policy/retention-policy.md` (tenant-selectable 30 / 60 / 90 days). KR PIPA Art. 21 (deletion) honoured; GDPR Art. 17 right-to-erasure (RTBF) honoured; CCPA right-to-delete honoured; COPPA §312.5 (parental deletion) honoured.
- EU AI Act 2024/1689: content-moderation classifier is classified as **limited risk** (not Annex III high-risk; users are anonymous → no individual significant-effect decision). Art. 50 transparency obligations apply (AI-assessed label on every verdict).
- EU DSA Arts. 14/16/17/27/28 transparency obligations apply (per-tenant ToS disclosure + statement-of-reasons + appeal + transparency report).
- US 18 USC §2258A NCMEC reporting: CSAM-suspect verdicts MUST trigger NCMEC CyberTipline within 48h (FR-27); chain-of-custody preserved per `runbooks/legal-process-court-order-receipt.md` Path E (NCMEC).

### Availability + SLO

- Availability target: 99.9% monthly for feed-render + post-create + vote-action.
- Anonymous-DM (MLS) best-effort 99.5% (client-driven retry).
- Hard-delete propagation correctness target: 100% (slipping below 100% on any RPO window opens Sev-1 incident).
- Legal-process disclosure correctness: 100% chain-of-custody preserved (slipping opens Sev-1 ops-security incident).
- RTO: ≤ 15 min for post-store + vote-store. RPO: ≤ 1 min (cross-AZ logical replication).

### Data residency

- Per-tenant pack pinning per ADR-0117. Anonymous-tier user data (blinded credential commitments) follow user-residency; geo-affinity communities pin to the user's geo-pack.
- Pack-eu Personal-tier never crosses pack boundary; structural invariant.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename). Layers used: `kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-postgres`, `adapter-redis`, `adapter-meilisearch`, `adapter-clamav`, `adapter-opswat`, `adapter-blind-signatures`, `adapter-bbs-plus`, `adapter-mls`, `rest`, `worker`, `sdk`, `app`.

| BC | Crate family | Purpose | Key entities |
|---|---|---|---|
| `pseudonymous-identity` | `oya-anonymous-pseudonymous-identity-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-blind-signatures,rest,sdk}` | Per-session blind-signed handle issuance; per-channel handle rotation; never stores `user_id` next to handle | `BlindedCredential`, `PerChannelHandle`, `HandleRotationEvent`, `BlindingNonce` |
| `affinity-attestation` | `oya-anonymous-affinity-attestation-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-bbs-plus,rest,sdk,worker}` | Cryptographic verification of employer/edu/geo/workspace affinity without identity disclosure; BBS+ + W3C VC 2.0 | `AffinityAttestation`, `AttestationIssuer`, `AffinityClaim`, `VerificationProof` |
| `post-thread` | `oya-anonymous-post-thread-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk}` | Post + comment + reply CRUD over blinded-commitment author column; T2 attachments optional | `Post`, `Comment`, `Reply`, `Tombstone`, `AuthorBlindingCommitment` |
| `feed-timeline` | `oya-anonymous-feed-timeline-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk}` | Chronological + algorithmic (optional) feed; affinity-scoped; ranking input free of author identifiers | `FeedEntry`, `RankSnapshot`, `FanoutPlan` |
| `upvote-downvote` | `oya-anonymous-upvote-downvote-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk}` | Conflict-free vote counter; per-blinded-credential one-vote-per-post bound | `Vote`, `VoteTally`, `VoteBindingProof` |
| `community-definition` | `oya-anonymous-community-definition-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Affinity-bound community definition (employer / edu / geo / workspace); k-anonymity floor enforcement | `Community`, `AffinityBound`, `KAnonymityFloor` |
| `hashtags` | `oya-anonymous-hashtags-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | #tag parse + per-tag corpus + trending input emission; never stores author-link | `Hashtag`, `HashtagCorpus`, `HashtagEmission` |
| `trending` | `oya-anonymous-trending-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk}` | Windowed trend compute within affinity scope; per-pack ranking | `TrendingTopic`, `TrendWindow`, `TrendRank` |
| `notifications` | `oya-anonymous-notifications-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk}` | Real-time + digest notification delivery; opaque-handle payloads only | `Notification`, `OpaqueHandlePayload`, `DigestBucket` |
| `content-moderation` | `oya-anonymous-content-moderation-{kernel,domain,usecase,api,adapter,adapter-clamav,adapter-opswat,worker,sdk}` | AI-classifier verdicts (EU AI Act limited-risk) + manual reviewer queue + appeal + abuse-report; NCMEC CyberTipline trigger | `ModerationVerdict`, `AbuseReport`, `Appeal`, `ClassifierVersion`, `NcmecReport` |
| `legal-process-disclosure` | `oya-anonymous-legal-process-disclosure-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Court-order receipt + dual-control approval + 14d user-notice + audit-chain seal + transparency-report inclusion | `LegalProcessRequest`, `DualApproval`, `DisclosurePackage`, `ChainOfCustodyHash` |
| `retention-policy` | `oya-anonymous-retention-policy-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | 30/60/90-day tenant-selectable tier; hard-delete worker; tombstone management | `RetentionTier`, `HardDeleteJob`, `Tombstone` |
| `anonymous-dm` | `oya-anonymous-anonymous-dm-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-mls,worker,sdk}` | MLS (RFC 9420) end-to-end DM over blinded handles; platform never holds plaintext | `MlsGroup`, `Welcome`, `Commit`, `AnonymousDmThread` |
| `age-gate` | `oya-anonymous-age-gate-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Pack-aware age-attestation at signup; COPPA <13 ban; pack-eu GDPR Art. 8 threshold; pack-kr Youth Protection | `AgeAttestation`, `AgeBracket`, `MinorRefusal` |

Naming justification — `pseudonymous-identity`:

```
NAME: oya-anonymous-pseudonymous-identity-<layer>
JUSTIFICATION:
- microservice = anonymous: per ADR-0131 per-microservice flat layout.
- bc-tokens = pseudonymous-identity: primary BC, hyphenated multi-token per ADR-0056 v4.1.
- layer = <layer>: ADR-0105 13-value canonical enum; ADR-0106 usecase rename.
- exemptions claimed: -adapter-postgres / -adapter-blind-signatures /
  -adapter-bbs-plus / -adapter-mls / -adapter-clamav / -adapter-opswat /
  -adapter-redis / -adapter-meilisearch are canonical *-adapter-<backend>
  per ADR-0105 Amendment 3.
```

Total crates introduced: **~95–115** (14 BCs × 6–8 layers per BC; backend variety lower than `social` because no media-transcode + no federation).

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implementation | Data classes touched |
|---|---|---|---|
| `BlindSignatureIssuer` | `oya-anonymous-pseudonymous-identity-kernel` | `-adapter-blind-signatures` (`ring 0.17` or `rust-bls` per ADR-ANON-0001) | `SECRET` (private key) / `BEHAVIORAL_TENANT_PRODUCT` |
| `BlindedCredentialRepository` | `oya-anonymous-pseudonymous-identity-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` (blinded commitment only) |
| `AffinityAttestationVerifier` | `oya-anonymous-affinity-attestation-kernel` | `-adapter-bbs-plus` | `BEHAVIORAL_TENANT_PRODUCT` |
| `AffinityAttestationStore` | `oya-anonymous-affinity-attestation-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` |
| `PostStore` | `oya-anonymous-post-thread-kernel` | `-adapter-postgres` | `BEHAVIORAL_USER_CONTENT`, `BEHAVIORAL_TENANT_PRODUCT` |
| `FeedCache` | `oya-anonymous-feed-timeline-kernel` | `-adapter-redis` | `BEHAVIORAL_TENANT_PRODUCT` |
| `VoteCounter` | `oya-anonymous-upvote-downvote-kernel` | `-adapter-redis` + `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` |
| `TrendStore` | `oya-anonymous-trending-kernel` | `-adapter-postgres` + `-adapter-redis` | `BEHAVIORAL_TENANT_PRODUCT` |
| `HashtagSearchIndex` | `oya-anonymous-hashtags-kernel` | `-adapter-meilisearch` | `BEHAVIORAL_TENANT_PRODUCT` |
| `NotificationStore` | `oya-anonymous-notifications-kernel` | `-adapter-postgres` + `-adapter-redis` | `BEHAVIORAL_TENANT_PRODUCT` (opaque-handle only) |
| `ModerationClassifier` | `oya-anonymous-content-moderation-kernel` | `-adapter` (foundry-runtime client; T2) | `INTERNAL_ONLY` |
| `MalwareScanner` | `oya-anonymous-content-moderation-kernel` | `-adapter-opswat` / `-adapter-clamav` | `INTERNAL_ONLY` |
| `LegalProcessRepository` | `oya-anonymous-legal-process-disclosure-kernel` | `-adapter-postgres` | `AUDIT`, `PII_IDENTIFYING` (court-order subject identifier) |
| `RetentionWorker` | `oya-anonymous-retention-policy-kernel` | `-adapter-postgres` | `BEHAVIORAL_USER_CONTENT` |
| `MlsGroupStore` | `oya-anonymous-anonymous-dm-kernel` | `-adapter-postgres` + `-adapter-mls` | `BEHAVIORAL_TENANT_PRODUCT` (ciphertext only) |
| `AuditChainClient` | (cross-BC) `-kernel` per BC | (cross-BC) `-adapter` to audit-chain µservice | `AUDIT` |
| `CedarAnonymousPolicy` | `oya-anonymous-pseudonymous-identity-kernel` (cross-BC) | `-adapter` (Cedar evaluator) | `INTERNAL_ONLY` |

Data-class enforcement: `oya-check-data-class` LEAN lane refuses unannotated fields.

Cross-product rule: `anonymous` MUST NOT import any other product µservice crate at any layer. Cross-product flows go through Workflow (events) or Ontology (`Affinity` entity reads only — NEVER `Person`). LEAN-A2 lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice anonymous` — dependency-direction
- `oya gate validate lean-a2 --microservice anonymous` — cross-product-refusal
- `oya gate validate port-location --microservice anonymous`
- `oya gate validate layer-correctness --microservice anonymous`
- `oya gate validate per-microservice-layout --microservice anonymous`
- `oya gate validate statelessness --microservice anonymous`
- `oya gate validate shardability --microservice anonymous`
- `oya gate validate authority-cohesion --microservice anonymous` (HG-ANONYMOUS)
- `oya gate validate blinding-column-isolation --microservice anonymous` (per ADR-ANON-0001)
- `oya gate validate third-party-tracker-refused --microservice anonymous` (per I4)
- `oya gate validate retention-default-short --microservice anonymous` (per I3)
- `oya gate validate e2e-no-plaintext-server-state --microservice anonymous` (per I6)
- `oya gate validate ontology-person-write-refused --microservice anonymous` (per I1/I2 — never write `Person`)

## Integration via Workflow + Ontology

### Workflow events produced

All event types carry the post's *blinded-author-commitment* (never `user_id`). Downstream consumers MUST NOT attempt to correlate (LEAN lane `oya-check-downstream-no-author-correlation`).

| Event type | Trigger | Consumed by | State machine |
|---|---|---|---|
| `AnonymousPostPublished` | end-user publishes post | feed-timeline, hashtags, trending, audit-chain, downstream Workflow engines | append-only |
| `AnonymousPostDeleted` | end-user / admin / retention-worker deletes | feed-timeline, hashtags, trending, audit-chain, retention worker | tombstone |
| `AnonymousCommentPublished` | end-user comments | feed-timeline, audit-chain | append-only |
| `AnonymousVoteApplied` | end-user votes | feed-timeline, trending, audit-chain | append-only |
| `AffinityAttestationBound` | new attestation verified | audit-chain (binding seal) | append-only |
| `AffinityAttestationRevoked` | issuer / tenant revokes attestation | audit-chain, all bound credentials | tombstone |
| `HandleRotationOccurred` | per-channel rotation | audit-chain (rotation seal — NOT correlated) | append-only |
| `HashtagEmission` | post carries hashtags | trending, hashtags-corpus | append-only |
| `ModerationVerdictEmitted` | classifier or reviewer issues verdict | feed-timeline (hide/show), notifications (recipient = blinded), audit-chain | append-only |
| `AbuseReportFiled` | end-user files abuse report | content-moderation, audit-chain | append-only |
| `AppealOpened` / `AppealResolved` | end-user appeals; reviewer resolves | audit-chain, notifications | append-only |
| `LegalProcessDisclosureRequested` | court-order received + recorded | legal-process workflow, audit-chain | state-machine |
| `LegalProcessDisclosureApproved` | dual-control approval | legal-process workflow, audit-chain | state-machine |
| `LegalProcessDisclosureExecuted` | identity-linkage performed; package sealed | audit-chain (chain-of-custody), transparency-report worker | append-only |
| `LegalProcessUserNotificationSent` / `Suppressed` | 14d notice sent or court-prohibited gag-order | audit-chain | append-only |
| `HardDeleteJobCompleted` | retention worker hard-deletes | audit-chain (tombstone seal) | append-only |
| `NcmecReportFiled` | CSAM-suspect classifier verdict + reviewer confirmation → CyberTipline | audit-chain (chain-of-custody for evidence) | append-only |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `OntologyAffinityChanged` | ontology | affinity-attestation | refresh affinity-bound community membership |
| `TenantRetentionPolicyUpdated` | tenancy | retention-policy | reassign tier (30/60/90) |
| `AuditChainSealed` | audit-chain | (read-only) | confirm seal durability |
| `WorkflowStudioLegalProcessApprovalRequested` | workflow-engine | legal-process-disclosure | dual-control flow trigger |

### Ontology writes

| Object Type | Written by BC | Audit trail | Notes |
|---|---|---|---|
| `Affinity{affinity_id, kind, scope, pack, k_floor, established_at}` | `community-definition` | Ed25519 | Affinity entity — NEVER `Person` |
| `AffinityAttestationBinding{binding_id, affinity_ref, blinded_commitment, established_at}` | `affinity-attestation` | Ed25519 | Binding to blinded commitment — NEVER user_id |

### Ontology reads

| Object Type | Read by BC | Query shape |
|---|---|---|
| `Affinity` | `community-definition`, `feed-timeline`, `hashtags` | `find_by(affinity_kind, tenant_id)` |
| `RetentionPolicy` | `retention-policy` | `lookup(tenant_id)` |
| `TenantContext` | every BC | tenant scope + pack jurisdiction lookup |

**Explicitly NOT read:** `Person`. Anonymous never resolves a person — that would defeat I1.

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | "Do NOT replicate" anti-pattern | Primary source |
|---|---|---|---|---|
| Sidechat | university-bound anonymous board | geo-affinity; ephemeral; iOS-first | none material | sidechat.lol (limited public docs) |
| YikYak | geo-bound anonymous board | 5-mile-radius geo; mobile-first | history of abuse on small campuses → harassment cascade; mitigate via I7 + classifier T2 + minimum-population k-anonymity | yikyak.com (limited public docs) |
| Whisper | open-topic anonymous board | photo-first anonymous confession | platform-side correlation (Whisper had server-side identity records discoverable) — DO NOT replicate; I1 prevents | whisper.sh (acquired 2017) |
| Blind | employer-bound anonymous board (Blind-class) | employer-verification via corporate email; salary-discussion focus | platform retained verified-email plaintext (verified by 2024 audits); DO NOT replicate; I2 affinity-only no-identity prevents | teamblind.com |
| Fishbowl | industry-bound anonymous board | LinkedIn acquired; verification via LinkedIn employer | similar Blind-class verification weakness; same I2 mitigation | fishbowlapp.com |
| Jodel | geo-bound anonymous board (DE market) | hyperlocal; chat threads | none material | jodel.com |
| Secret (defunct) | open-topic anonymous app | shut down 2015 due to harassment | DO NOT replicate: no abuse-reporting, no moderation, no legal-process workflow — caused harassment cascades and class-action settlement | (defunct) |
| Reddit anonymous subs (r/anonymous) | pseudonymous-by-username subreddit | persistent pseudonym | persistent-pseudonym enables long-term correlation; we DO NOT replicate; I1 rotates handle |
| 4chan | board-based no-account posting | text + image board; ephemeral threads | DO NOT replicate: no moderation accountability; CSAM hosting historical; we satisfy 18 USC §2258A NCMEC reporting via FR-27 + ADR-ANON-0005 |
| Burnbook (defunct) | school-bound anonymous gossip app | shut down 2017 due to bullying | DO NOT replicate: no minor protection, no school-admin oversight, no anti-harassment classifier; we satisfy via FR-14 (age-gate per pack) + FR-12 (report) + ADR-ANON-0005 (classifier bounds) |
| Tor + onion-routed forums | network-anonymity | strong network anonymity | server-side correlation is not solved by Tor; we add I1 server-side cryptographic blinding ON TOP of optional Tor-style transport |

Key parity gaps to close (ordered by priority):

1. **Cryptographic identity-correlation refusal at platform layer** — every competitor retains the technical capability to link user_id ↔ post_id (Sidechat, YikYak, Whisper, Blind all do; Whisper case proved it). Target: ADR-ANON-0001 cryptographic blinding makes platform structurally unable.
2. **Affinity attestation without identity disclosure** — Blind ships employer-verification via plaintext-email; we ship BBS+ selective-disclosure with verified-attestation-only.
3. **Legal-process disclosure with dual-control + transparency report** — competitors handle ad hoc. We codify ADR-ANON-0003.
4. **No third-party analytics SDK as structural invariant** — every competitor ships at minimum Google Analytics + Crashlytics. We refuse at build-time.
5. **Native Workflow + Ontology integration without leaking author** — competitors expose webhooks/REST with author identifier. Ours carry blinded-commitment only.
6. **k-anonymity floor on affinity communities** — competitors permit single-member communities (Blind allows single-employee "verified at FooCorp" → de-anonymises trivially). We require k=50 geo / k=20 employer / k=10 small-employer-with-anonymization-fallback.

## Performance Targets

(See Performance NFR table above.)

Error budget:
- Monthly error budget for post-create + feed-render: 0.1% (≈ 43 min/month).
- Burn-rate alarm on `anonymous.post-create.availability` is 14.4× burn rate over 1h.
- Error budget policy: `microservices/anonymous/runbooks/anonymity-leak-incident-response.md` is the P0 escalation path (which overrides standard budget policy — anonymity leak is P0 regardless of budget).

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed`. Postgres for posts + votes + attestation-bindings; Redis for feed cache + vote-counter; Meilisearch for hashtag search.

**Active-active compatibility**: stateless REST + worker pods + Postgres logical-replicated within pack; Redis primary-replica HA.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Active users / cell | 200k | 2M | feed-render p99 > 250ms |
| Posts/sec sustained | 500 | 10k | Postgres write IOPS > 70% |
| Affinity communities per tenant | 100 | 50k | per-tenant cardinality limit hit |
| Vote events/sec | 2k | 50k | Redis counter saturation |
| Hashtag index size | 50GB | 2TB | shard count exceeded |

Scale-out policy:
- HPA on REST pods: CPU > 70%, min 4, max 100 replicas.
- Postgres shard-by-tenant once cell hits 10k posts/sec aggregate.
- Redis cluster sharding by `(tenant_id, affinity_id) mod N`.

Sharding:
- Post store partitions by `(tenant_id, affinity_id, year-month)`.
- Vote store partitions by `(tenant_id, post_id mod N)`.
- Feed cache partitions by `(tenant_id, affinity_id mod N)`.
- `oya-check-shardability-cli` lane verifies partition keys are present in every kernel struct.

Note: Sharding key MUST NOT use `user_id` (would defeat I1). Sharding is by `affinity_id` + blinded post commitment.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | Post-create + comment + vote roundtrip completes within p99 < 250ms post-create | `microservices/anonymous/tests/e2e/post-comment-vote.rs` |
| AC-02 | Hard-delete a post → no read path returns the post within p99 ≤ 5s | `tests/e2e/hard-delete-propagation.rs` |
| AC-03 | A post in Postgres carries blinded author commitment, NOT user_id | `tests/e2e/blinding-column-presence.rs` + LEAN lane |
| AC-04 | Affinity-attestation verify within p95 ≤ 500ms; rejects forged proof | `tests/e2e/affinity-attestation-verify.rs` |
| AC-05 | k-anonymity floor enforced: affinity-community with cardinality < 50 (geo) cannot be created | `tests/e2e/k-anonymity-floor.rs` |
| AC-06 | Federation attempt (cross-pack ActivityPub egress) is REFUSED at build (no BC + no chart toggle) | `tests/e2e/federation-refused.rs` |
| AC-07 | Notification push payload contains opaque handle only — never real-name field | `tests/e2e/notification-opaque-payload.rs` |
| AC-08 | Third-party tracker dependency rejected at build (LEAN lane) | `oya-check-third-party-tracker-refused` |
| AC-09 | Moderation classifier verdict → audit-chain seal within 2s + appeal workflow opens | `tests/e2e/moderation-appeal.rs` |
| AC-10 | Age-gate: under-13 signup REFUSED for all packs (COPPA universal) | `tests/e2e/age-gate-coppa.rs` |
| AC-11 | Legal-process disclosure requires two distinct approvers + Cedar gate + audit-chain seal | `tests/e2e/legal-process-dual-control.rs` |
| AC-12 | Legal-process disclosure executed → transparency-report queue populated (with court-prohibited flag where applicable) | `tests/e2e/transparency-report-inclusion.rs` |
| AC-13 | `oya gate validate per-microservice-layout --microservice anonymous` exit 0 | ADR-0131 lane |
| AC-14 | `oya gate validate authority-cohesion --microservice anonymous` exit 0 | ADR-0123 lane; HG-ANONYMOUS registered |
| AC-15 | `oya gate validate ontology-person-write-refused --microservice anonymous` exit 0 | I1/I2 |
| AC-16 | Anonymous-DM (MLS) — server holds ciphertext only; plaintext never appears in any log / metric / audit | `tests/e2e/mls-server-plaintext-refused.rs` |
| AC-17 | NCMEC CyberTipline reporting triggered within 48h on confirmed CSAM verdict | `tests/e2e/ncmec-reporting.rs` |
| AC-18 | EU AI Act Art. 50 transparency label "AI-assessed" appears on every classifier verdict | `tests/e2e/eu-ai-act-art50-label.rs` |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Should the abuse-classifier be foundry-runtime-only (cross-µservice) or have a dedicated in-µservice inference path for stricter privacy (no cross-µservice plaintext)? | axis-anonymous + axis-foundry-runtime + council-privacy | ADR-ANON successor-IP after P02 |
| 2 | Affinity attestation: support OIDC-with-blinding (issuer = corporate IdP via OIDC) in addition to BBS+/W3C VC 2.0? | axis-anonymous + council-architecture | ADR-ANON-0002 successor-IP |
| 3 | Anonymous-DM (MLS): default-on or default-off per tenant? Default-off conserves complexity; default-on maximizes privacy. | axis-anonymous + gtm | ADR-ANON successor-IP after MLS minimum-shippable-tier |
| 4 | Geo-affinity k-anonymity floor sliding-scale (k=50 default; what about rural sparsely-populated regions where k=50 forces big merge)? Optional anonymization fallback? | axis-anonymous + council-privacy | ADR-ANON-0007 |
| 5 | Transparency-report cadence: quarterly default; tenants want monthly? | axis-anonymous + gtm | resolution pending |
| 6 | Retention default: 30 days globally vs per-pack overrides (pack-eu shorter? pack-us longer for legal-hold defaults)? | council-privacy + ops-security | ADR-ANON-0004 |
| 7 | T2 attachments: keep default-off forever (text-only platform), or per-tenant opt-in image attachments? | axis-anonymous + gtm | ADR successor-IP after M03 |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0008 | Data Use Boundary | personal/professional + anonymity data-use invariants |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum + Amendment 3 | layer + backend-qualified authority |
| ADR-0106 | application → usecase | layer naming |
| ADR-0135 | Connect dissolution (parallel) | anonymous as a sibling µservice |
| ADR-0130 | Agentic SLO-gated promotion | gates anonymous releases |
| ADR-0131 | Per-microservice flat layout | this PRD authored under it |
| ADR-0132 | Suite-and-bundle dissolution | factored Connect into surfaces |
| ADR-0133 | Industry best-practice conformance | HG-ANONYMOUS under this |
| ADR-ANON-0001 | Cryptographic-blinding-protocol | this µservice (I1) |
| ADR-ANON-0002 | Affinity-attestation-verification | this µservice (I2) |
| ADR-ANON-0003 | Legal-process-disclosure-workflow | this µservice (I7) |
| ADR-ANON-0004 | Retention-and-deletion-policy | this µservice (I3) |
| ADR-ANON-0005 | Abuse-classifier-bounds | this µservice |
| ADR-ANON-0006 | Federation-refusal-and-anti-pattern-anchoring | this µservice (I5) |
| ADR-ANON-0007 | Affinity-cluster-design | this µservice (I2 + k-anonymity) |
| Bominal ADR-0028 | Audit-chain Merkle + Ed25519 | inherited |
| Bominal ADR-0019 | State strategy enum | inherited |
| ADR-MSGR-0002 | E2E DM key escrow | informs anonymous-DM MLS design |
