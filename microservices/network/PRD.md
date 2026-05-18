---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-network
microservice: network
status: Accepted
sales_segment: connect-suite-product
tier: hero-product
milestone_first_ship: M02-foundation
bominal_source: [ADR-0208-connect-dual-context-unified-channel-hub.md]
related_adrs: [ADR-0008, ADR-0056, ADR-0105, ADR-0106, ADR-0135, ADR-0130, ADR-0131, ADR-0132, ADR-0133, ADR-0134]
related_specs: [/specs/microservices/network.json, /specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
owner_team: axis-network
doc_status: published
---

# PRD-network: First-Party Professional Network (Resume + Connections + InMail + Endorsements + Jobs handoff + Events + Pages + Recommendations + Skill Assessments + Recruiter-Stub)

## Purpose

The `network` microservice is oyatie's native LinkedIn-class first-party Professional social network. Per parallel-session ADR-0135 (Connect dissolution), it is one of the first-class µservices factored out of the legacy Connect umbrella; per ADR-0132 the `network` surface is single-concern and Professional-tier-only — distinct from the sibling `social` µservice which owns the Personal/General microblog surface. `network` owns **professional-profile (resume/CV, skills, endorsements, certifications, education, recommendations) + professional-graph (1st/2nd/3rd-degree directed-bidirectional-on-acceptance connection edges) + connection-request (with note) + follow (asymmetric, distinct from connect) + feed-timeline (Professional content; chronological + heuristic-algorithmic) + post-composition (article + status + document + poll + carousel) + reactions (extended set: like / celebrate / insightful / curious / funny) + comments + replies + share + repost + inmail (premium messenger surface routed through messenger µservice) + groups (private + open) + events (with calendar bridge) + pages (company / brand) + services-marketplace-stub (M04-onward) + recruiter-tooling-stub (M04-onward; off by default) + jobs-postings-stub (handoff to Tier-G ATS µservice) + learning-stub (LinkedIn-Learning equivalent; M05-onward) + endorsements + recommendations (long-form testimonial) + skill-assessments + profile-verification (badge + ID-attest + employer-confirm) + salary-insights-stub + hashtags + trending-topics (Professional context only) + notifications + accessibility-captions + Professional-context-isolation invariant + EU AI Act Annex III §4 (employment) high-risk obligations for ranking + recruiter + endorsement aggregation + Art. 22 right-to-human-review + EEOC + Title VII bias monitoring** across the 11 oyatie regulatory packs.

This µservice is **a hero product**, end-user-facing through Workflow Studio shell and standalone Professional-network clients (web + desktop + mobile). It is consumable as a shared substrate by other oyatie products via the `network.*` Workflow events and the `Person`, `Company`, `Skill`, `Certification`, `Education`, `JobPosting` Ontology object types. It is also the canonical B2B identity provider for the rest of the oyatie surfaces — every Professional-tier feature that needs a verified employment record reads through the `network` µservice's resume + verification surface.

Bominal predecessor: the `connect-network` slice of Bominal's unified Connect suite. Per parallel ADR-0135, that monolithic suite is dissolved into per-surface µservices; this PRD is the canonical Professional-network landing in oyatie. **network is NET-NEW** — no `oya-connect-network-*` crates exist; there is no migration-from-connect.md and no deprecation-notice.md.

## Tenant Value

- **Tenant Outcome 1 — Native Professional identity without identity fragmentation.** Tenants and their end-users get LinkedIn-class profile + connection + recommendations + endorsement + skill-assessments + job-handoff UX inside the same shell as mail, messenger, calendar, workflow studio, social — without leaving the surface to a third-party Professional network.
- **Tenant Outcome 2 — Professional-only, never federates to Personal.** Per parallel ADR-0135, `network` is Professional-tier-only; the `context_kind: Professional` discriminator is compile-time-only; no Personal posts, no Personal followers, no Personal-tier audit scope ever surfaces. The sibling `social` µservice owns the Personal/General microblog surface.
- **Tenant Outcome 3 — Real-time profile-view, connection-action, and feed-render.** Feed-render p95 ≤ 200ms; profile-view p95 ≤ 150ms; connection-action p95 ≤ 50ms; search-people p95 ≤ 250ms (the most-searched surface); search-content p95 ≤ 500ms; InMail-send p95 ≤ 100ms; notification-fanout p99 ≤ 2s for 30k-follower account.
- **Tenant Outcome 4 — Auditable Professional-context decisions.** Every endorsement, recommendation, recruiter-search invocation, jobs-ranking decision, and disclosure event emits an audit-chain record (Merkle / Ed25519); EU AI Act Annex III §4 (employment, workers management, access to self-employment) high-risk classification applies to recruiter ranking + jobs ranking + endorsement aggregation; GDPR Art. 22 right-to-human-review surfaces on every materially-impacting automated decision in the employment context.
- **Tenant Outcome 5 — Cross-product Professional integration.** Resume reads cross-link to mail (newsletter-of-record send), messenger (InMail premium-tier bridge), calendar (events bridge), workflow-engine (connection-request approval workflow), workflow-studio (B2B template marketplace), ATS µservice in Tier-G (jobs-posting handoff via contract-versioned event), foundry-runtime (article + caption + bio T1 assist; ranking + recommender T2 auto with EU AI Act guardrails).
- **Tenant Outcome 6 — Multi-pack residency by design with strong employment-law overlays.** 11 region-pinned packs; KR 직장 갑질 (workplace harassment) protections + 통신비밀보호법 for InMail; EU AI Act high-risk for recruiter + ranking + endorsement aggregation + GDPR Art. 22 (automated decision-making) + Equal Treatment Directives 2000/43/EC + 2000/78/EC; US Title VII + ADA + ADEA + EEOC UGESP + NYC AI Hiring Law (Local Law 144-2021) + CA AB-331 + CO SB-205; JP 個人情報保護法 + 労働基準法; SG PDPA employment guidance; AU Privacy Act + AHRC AI guidance; IN DPDPA 2023; BR LGPD + CLT; UAE PDPL + Federal Decree-Law 33/2021 (Labour); KSA PDPL + Labor Law.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | end-user | to author a Professional profile (resume: experience + education + skills + certifications + summary + headline + locale) | I have a Professional identity | professional-profile | Must |
| FR-02 | end-user | to send a connection request (with optional note ≤ 300 chars) | I grow my network | connection-request | Must |
| FR-03 | end-user | to accept / reject / ignore an inbound connection request | I curate my network | connection-request | Must |
| FR-04 | end-user | to follow another profile (asymmetric; distinct from connect) | I see their content without connecting | professional-graph | Must |
| FR-05 | end-user | to publish a Professional post (article + status + document + poll + carousel up to 4096 chars text + 10 media + 1 attached document) | I broadcast to my network | post-composition | Must |
| FR-06 | end-user | to repost / share / quote-post + add commentary | I amplify and commentary-add | post-composition | Must |
| FR-07 | end-user | to comment + reply (threaded depth ≤ 3) | conversation depth is preserved | post-composition | Must |
| FR-08 | end-user | to react inline with the extended Professional reaction set (like, celebrate, insightful, curious, funny, support, love) | low-overhead acknowledgement | reactions | Must |
| FR-09 | end-user | to see a chronological + a heuristic-algorithmic Professional feed | I choose how I consume | feed-timeline | Must |
| FR-10 | end-user | to @mention people, companies, skills, hashtags | recipients are notified + linked | mentions | Must |
| FR-11 | end-user | to use #hashtags for discoverability (Professional context only) | content is grouped by topic | hashtags | Must |
| FR-12 | end-user | to see trending Professional topics within my tenant + pack | discovery surfaces broader Professional context | trending-topics | Must |
| FR-13 | end-user | to search for people + content + skills + jobs + companies + events (all Cedar-filtered) | I recover and discover context | search | Must |
| FR-14 | end-user | to send an InMail (premium messenger-bridge to recipient with no existing connection) | I reach outside my graph | inmail-bridge | Must |
| FR-15 | end-user | to endorse a connection's skill (1-click; bounded to skills the endorsee has listed) | reciprocal validation | endorsement-engine | Must |
| FR-16 | end-user | to request + write a long-form recommendation for a connection | testimonial chain | endorsement-engine | Must |
| FR-17 | end-user | to take a skill-assessment quiz + display a passing-badge on profile | demonstrate competence | skill-assessments | Must |
| FR-18 | end-user | to request profile-verification (ID-attest or employer-confirm) | authenticity signals exist | profile-verification | Must |
| FR-19 | end-user | to export own profile as vCard 4.0 (RFC 6350) + JSON Resume + GDPR Art. 20 portable JSON | data portability + interoperability | professional-profile + portability | Must |
| FR-20 | end-user | to create / join / leave a group (private or open) | community within the network | groups | Must |
| FR-21 | end-user | to create an event (calendar bridge) + RSVP | Professional event coordination | events-bridge | Must |
| FR-22 | end-user | to follow a Page (company / brand) | I track companies + brands | pages | Must |
| FR-23 | end-user | to receive real-time + digest notifications | I stay informed | notifications | Must |
| FR-24 | end-user | to view a salary-insights stub (per role + region) | wage transparency | salary-insights-stub | Should |
| FR-25 | end-user | to set per-post visibility (public, network-only, group, private) | I control scope | post-composition | Must |
| FR-26 | end-user | to add accessibility-captions to media (WCAG 2.2 Level AA) | accessibility | accessibility-captions | Must |
| FR-27 | end-user | to opt-out of EU AI Act high-risk automated decisions on own profile (recruiter ranking + jobs ranking + endorsement aggregation) per GDPR Art. 22 | right-to-human-review | recommender + recruiter-stub | Must |
| FR-28 | tenant-admin | to enable / disable recruiter-tooling-stub on the tenant (OFF BY DEFAULT) | tenant control over high-risk module | recruiter-stub | Must |
| FR-29 | tenant-admin | to configure pack-aware retention + employment-law overlay (KR labor, EEOC, etc.) | regulatory bounds hold | professional-profile | Must |
| FR-30 | compliance-officer | to issue eDiscovery hold on Professional posts + InMails | regulatory request is satisfied | post-composition + inmail-bridge + audit-chain | Must |
| FR-31 | tenant-admin | to verify a profile or Page (blue-badge equivalent) | authenticity signals exist | profile-verification | Must |
| FR-32 | Workflow Studio | to consume `PostPublished` / `ConnectionEstablished` / `EndorsementAdded` / `JobApplied` events | downstream automation works | post-composition + connection-request + endorsement-engine + jobs-handoff | Must |
| FR-33 | messenger µservice | to deep-link an InMail thread into the messenger Professional surface | identity registry is shared | inmail-bridge | Must |
| FR-34 | ATS µservice (Tier G) | to receive contract-versioned `JobPostingPublished` + `JobApplicationFiled` events | jobs-handoff cleanly separates posting from pipeline mgmt | jobs-handoff | Must |
| FR-35 | mail µservice | to receive `NewsletterSendRequested` for company-page newsletter | newsletter-of-record path | pages → mail | Must |
| FR-36 | calendar µservice | to receive `NetworkEventCreated` for event bridge | calendar bridge | events-bridge → calendar | Must |
| FR-37 | tenant-operator | to query profile + connection-graph + endorsement-chain + recommender-fairness metrics | I plan capacity + verify SLAs + audit bias | observability | Must |
| FR-38 | recruiter (when tenant-enabled) | to invoke recruiter-search ranker against the tenant's tenant-scoped pool | finding-talent within tenant scope | recruiter-stub | Should |
| FR-39 | end-user | to file a Professional-context abuse report (harassment / discrimination / impersonation / 직장 갑질) | community safety | abuse-reporting | Must |
| FR-40 | end-user | to appeal a moderation verdict or a high-risk automated decision | due-process + GDPR Art. 22 | appeal-workflow | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p95 | p99 | p999 | Notes |
|---|---|---|---|---|---|
| Feed-render latency (top 50 posts; algorithmic mode) | ≤ 60ms | ≤ 200ms | ≤ 400ms | ≤ 1s | Redis hot-feed cache; warm hit on read |
| Profile-view latency | ≤ 40ms | ≤ 150ms | ≤ 350ms | ≤ 800ms | Postgres + Redis cache; first-degree extension over Postgres |
| Connection-action latency (request / accept / ignore) | ≤ 20ms | ≤ 50ms | ≤ 150ms | ≤ 400ms | Postgres adjacency-list write + audit-chain emit |
| Search-people latency (≤ 25 results) | ≤ 80ms | ≤ 250ms | ≤ 600ms | ≤ 1.2s | Meilisearch + Cedar filter; most-searched surface |
| Search-content latency (≤ 25 results) | ≤ 150ms | ≤ 500ms | ≤ 1s | ≤ 2s | Meilisearch + Cedar filter |
| Search-jobs latency | ≤ 100ms | ≤ 400ms | ≤ 900ms | ≤ 1.8s | faceted Meilisearch index; jobs-handoff materialised view |
| InMail-send latency | ≤ 40ms | ≤ 100ms | ≤ 300ms | ≤ 700ms | network → messenger bridge; per-tenant rate limit |
| Notification fanout (30k followers) | ≤ 200ms | ≤ 1s | ≤ 2s | ≤ 5s | per-recipient async via Redis Streams |
| Notification fanout (300k followers) | ≤ 600ms | ≤ 2.5s | ≤ 6s | ≤ 18s | sharded fanout workers |
| Endorsement add | ≤ 15ms | ≤ 50ms | ≤ 120ms | ≤ 300ms | Redis-buffered + Postgres flush; audit-chain emit |
| Recommendation publish | ≤ 30ms | ≤ 100ms | ≤ 250ms | ≤ 700ms | Postgres insert + audit-chain seal |
| Profile-export vCard 4.0 + JSON Resume | ≤ 100ms | ≤ 300ms | ≤ 700ms | ≤ 1.5s | server-side render |
| Media transcode (image, ≤ 10MB) | ≤ 800ms | ≤ 2s | ≤ 4s | ≤ 10s | ImageMagick |
| Media transcode (video, ≤ 200MB) | ≤ 30s | ≤ 90s | ≤ 180s | ≤ 300s | ffmpeg HLS segmentation |
| Skill-assessment quiz submission | ≤ 50ms | ≤ 200ms | ≤ 500ms | ≤ 1s | Postgres + scoring worker |
| Recruiter-search ranker (when enabled) | ≤ 200ms | ≤ 600ms | ≤ 1.5s | ≤ 3s | foundry-runtime T2 + bias-audit emit |

### Security

- Profile + post + connection-graph + endorsement + recommendation + InMail reads enforced server-side via Cedar policy (`policy/tenant-scope.cedar` + `policy/public-read.cedar` + `policy/professional-context-isolation.md`); client never trusted.
- Professional-tier posts + InMail bodies: tenant-DEK encrypted (envelope encryption per Bominal ADR-0111); admin disclosure requires four-eyes audit trail per Bominal ADR-0215.
- Personal-tier never federates into `network` — compile-time invariant per ADR-NET-0001 (Professional-context-isolation).
- Media uploads scanned via OPSWAT MetaDefender or ClamAV before publication; quarantine bucket pattern; ImageMagick + ffmpeg sandboxed (gVisor / Kata) per ADR-NET-0006-style media controls (see ADR-SOC-0006 inherited shape).
- All WebSocket connections mTLS-terminated; per-tenant API token bound at OpenBao with rotation 30d.
- Search index excludes redacted PII per `policy/data-residency.md` (pack-us-healthcare overlay maps health-context post-content to PHI when present; same pattern as messenger).
- Cross-context routing forbidden: a Personal `social` post cannot become a Professional `network` post; enforced by `policy/professional-context-isolation.md`.
- Federation egress: `network` does NOT federate to ActivityPub in P01. (No equivalent of social's federation-gateway; Professional Network is Professional-only and federation is scheduled-for-distinct-tracked-work to ADR-NET successor-IP if demand emerges.)
- InMail-bridge: contract-bound at messenger µservice; cannot escalate Professional InMail to Personal-tier DM (Personal-tier DM is owned by messenger / social-DM, distinct surface).
- Recruiter-tooling-stub: OFF BY DEFAULT; activation requires tenant-admin entitlement + EU AI Act Art. 27 fundamental-rights-impact-assessment + NYC Local Law 144 bias-audit attestation (when NYC-tenant).
- Salary-insights-stub: aggregate-only; per-individual disclosure forbidden.

### Audit + Compliance

- Every profile-update / post-create / post-delete / connection-action / endorsement-add / endorsement-revoke / recommendation-publish / inmail-send / inmail-disclosure / appeal-action / four-eyes-disclosure / hold / recruiter-search-invocation / jobs-handoff-event writes an audit-chain record (Merkle / Ed25519 per Bominal ADR-0028).
- Professional-tier disclosure (admin reads PII or InMail body) requires two distinct approving principals + reason code (per Bominal ADR-0215).
- Endorsement-chain integrity: each endorsement carries a per-endorser Ed25519 signature; aggregated endorsement counts are reconstructible from the audit-chain replay (ADR-NET-0005).
- Retention: per-pack bounds in `policy/data-residency.md`. KR PIPA + 근로기준법 (Labor Standards Act) work-record floor satisfied. GDPR storage-limitation honored. HIPAA pack: when health-context profiles surface (rare in Professional network), PHI retention 6y applies.
- EU AI Act 2024/1689 Annex III §4 (employment, workers management, access to self-employment): HIGH-RISK classification applies to (a) recruiter-search ranker, (b) jobs-ranking, (c) endorsement-aggregation that materially impacts recruiter-search rank, (d) people-you-may-know recommender when used for employment intent. Transparency + risk-management + post-deployment monitoring obligations per Arts. 9–15 + 27 + 50 + 73 satisfied via `capabilities/T2-auto.yaml` evidence pipeline + ADR-NET-0002.
- GDPR Art. 22 (automated decision-making): per-end-user opt-out + human-review path mandatory; surfaced at recruiter-search ranker + jobs-ranking + endorsement-aggregation.
- EEOC + Title VII + ADA + ADEA disparate-impact monitoring: bias audit per release; 4/5 rule statistical check; protected-group disparity ratio published per `dashboards/recommender-fairness-and-bias.json`.
- NYC AI Hiring Law (Local Law 144-2021): annual bias audit + candidate notice when recruiter-tooling activated for NYC tenants.
- CA AB-331 + CO SB-205: automated-decision transparency obligations active when CA / CO tenant activates recruiter-stub.
- KR 직장 갑질 protections: Professional-context abuse reports routed via dedicated `harassment-workplace` abuse category; `runbooks/harassment-report-escalation.md` shape (covered cross-cutting at content-moderation-rollback runbook).
- KR 통신비밀보호법: InMail intercept only via four-eyes audit; covered.

### Availability + SLO

- Availability target: 99.95 % monthly for feed-render + profile-view + connection-action.
- Notification fanout is best-effort; 99.9 % monthly.
- Search availability 99.9 % monthly.
- InMail-send availability 99.95 % monthly (chained to messenger µservice availability).
- RTO: ≤ 15 min for profile-store. RPO: ≤ 5 min (cross-region replication for Professional store).

### Data residency

- Per-tenant pack pinning per ADR-0117. `network` is Professional-tier-only, so residency follows the tenant (not per-user as in `social`).
- No federation egress in P01; if federation is added in a successor-IP ADR-NET, opt-in tenant-only + Professional-tier + SCC required.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename). Layers used: `kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-postgres`, `adapter-redis`, `adapter-s3`, `adapter-meilisearch`, `adapter-clamav`, `adapter-opswat`, `adapter-imagemagick`, `adapter-ffmpeg`, `adapter-messenger-bridge`, `adapter-calendar-bridge`, `adapter-mail-bridge`, `adapter-ats-bridge`, `rest`, `worker`, `sdk`, `app`.

| BC | Crate family | Purpose | Key entities |
|---|---|---|---|
| `professional-profile` | `oya-network-professional-profile-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,sdk,app}` | Profile CRUD; resume sections (experience, education, skills, certifications, summary, headline, locale); verification badge; vCard 4.0 + JSON Resume export | `Profile`, `ExperienceEntry`, `EducationEntry`, `SkillEntry`, `Certification`, `Headline`, `Summary`, `VerificationBadge` |
| `professional-graph` | `oya-network-professional-graph-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | 1st/2nd/3rd-degree connection edges (directed-bidirectional-on-acceptance); follow edges (asymmetric); block / restrict / disconnect lists; adjacency-list storage with degree-of-separation computation | `ConnectionEdge`, `FollowEdge`, `BlockEdge`, `RestrictEdge`, `DegreeOfSeparation` |
| `connection-request` | `oya-network-connection-request-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | Connection-request lifecycle (open, accepted, rejected, ignored, withdrawn); per-user-per-week rate limit; spam classifier signal | `ConnectionRequest`, `ConnectionRequestNote`, `ConnectionRequestStatus`, `RequestRateLimit` |
| `post-composition` | `oya-network-post-composition-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,adapter-imagemagick,adapter-ffmpeg,rest,worker,sdk,app}` | Article + status + document + poll + carousel + repost + share + comment CRUD; media upload + transcode; document-attach; link-preview; visibility scope; cross-link to messenger | `Post`, `Article`, `Status`, `DocumentAttachment`, `Poll`, `Carousel`, `Repost`, `Share`, `Comment`, `Media`, `LinkPreview`, `Visibility`, `ContentWarning` |
| `feed-timeline` | `oya-network-feed-timeline-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk,app}` | Chronological + heuristic-algorithmic feed materialisation; fanout-on-write for hot-tier accounts; fanout-on-read for cold-tier; Professional-context ranking | `FeedEntry`, `RankingSignal`, `FanoutPlan`, `RankSnapshot` |
| `reactions` | `oya-network-reactions-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk}` | Inline reactions extended Professional set (like, celebrate, insightful, curious, funny, support, love); conflict-free counter; per-user reaction record | `Reaction`, `ReactionTally`, `UserReactionRecord` |
| `mentions` | `oya-network-mentions-{kernel,domain,usecase,api,adapter,worker,sdk}` | @mention parse; Ontology lookup over Person + Company + Skill + Hashtag; fanout to notifications + cross-µservice bridges | `Mention`, `MentionTarget`, `MentionFanoutPlan` |
| `hashtags` | `oya-network-hashtags-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | #tag parse; per-tag corpus; Professional-context trending input emission | `Hashtag`, `HashtagCorpus`, `HashtagEmission` |
| `trending-topics` | `oya-network-trending-topics-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk}` | Windowed trend compute over hashtags + entities; per-tenant per-pack Professional-context ranking | `TrendingTopic`, `TrendWindow`, `TrendRank` |
| `notifications` | `oya-network-notifications-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk,app}` | Real-time + digest notification delivery; per-recipient idempotent; backpressure-coalesced; ratelimit-aware on 300k-follower bursts | `Notification`, `DigestBucket`, `RealtimeFrame` |
| `inmail-bridge` | `oya-network-inmail-bridge-{kernel,domain,usecase,api,adapter,adapter-messenger-bridge,worker,sdk}` | Premium InMail surface — Professional-tier-only routing to messenger µservice; rate-limit + spam-classifier; never federates to Personal-tier DM | `InMail`, `InMailThread`, `InMailRateBudget`, `InMailDeliveryReceipt` |
| `endorsement-engine` | `oya-network-endorsement-engine-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | Skill endorsements (per-skill 1-click) + long-form recommendations + per-endorser Ed25519 signature; Merkle-style endorsement chain via audit-chain; revocation flow | `Endorsement`, `Recommendation`, `EndorsementSignature`, `EndorsementChainEntry`, `RevocationRecord` |
| `skill-assessments` | `oya-network-skill-assessments-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | Skill quiz administration + scoring + passing-badge issuance; per-skill item-bank; anti-cheat | `SkillAssessment`, `QuizItem`, `Attempt`, `Score`, `PassingBadge` |
| `profile-verification` | `oya-network-profile-verification-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Verification badge (blue / organisation / government / employer-confirmed); ID-attest flow; employer-confirm flow; revocation | `VerificationRequest`, `VerificationBadge`, `EmployerAttestation`, `IdAttestation`, `RevocationEvent` |
| `pages` | `oya-network-pages-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,sdk,app}` | Company / brand Pages with multi-admin; newsletter-of-record bridge to mail; analytics; follower-count; verified Page badge | `Page`, `PageAdmin`, `PageNewsletter`, `PageAnalyticsSnapshot` |
| `groups` | `oya-network-groups-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,sdk,app}` | Private + open groups; per-group feed; moderation; admin / moderator roles; join-request flow | `Group`, `GroupMembership`, `GroupRole`, `JoinRequest`, `GroupVisibility` |
| `events-bridge` | `oya-network-events-bridge-{kernel,domain,usecase,api,adapter,adapter-calendar-bridge,worker,sdk}` | Professional events surface with calendar µservice bridge; RSVP; capacity; recurring; iCal export | `NetworkEvent`, `RSVP`, `EventCapacity`, `RecurringRule`, `ICalExport` |
| `jobs-handoff` | `oya-network-jobs-handoff-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-ats-bridge,worker,sdk}` | Jobs-posting surface bound to network; contract-versioned event handoff to Tier-G ATS µservice; jobs-search facets; applicant referral flow | `JobPosting`, `JobApplicationReferral`, `JobsFacet`, `ATSHandoffEvent` |
| `recruiter-stub` | `oya-network-recruiter-stub-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Recruiter-tooling-stub OFF BY DEFAULT; per-tenant opt-in; bias-audit emit; NYC Local Law 144 conformance hook; CA AB-331 transparency hook | `RecruiterStubConfig`, `RecruiterSearchRequest`, `RecruiterSearchAudit`, `BiasAuditRecord` |
| `services-marketplace-stub` | `oya-network-services-marketplace-stub-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Services-marketplace stub OFF BY DEFAULT in P01; M04-onward activation under future ADR-NET | `ServiceListing`, `ServiceOffer`, `ServiceMarketplaceConfig` |
| `learning-stub` | `oya-network-learning-stub-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | LinkedIn-Learning-equivalent stub OFF BY DEFAULT in P01; M05-onward activation under future ADR-NET | `LearningPath`, `LearningCourseStub`, `LearningEnrollmentStub` |
| `salary-insights-stub` | `oya-network-salary-insights-stub-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Aggregate salary-insights (per role + region); per-individual disclosure forbidden; tenant opt-in | `SalaryInsightSnapshot`, `RoleSalaryBand`, `RegionalSalaryAggregate` |
| `search` | `oya-network-search-{kernel,domain,usecase,api,adapter,adapter-meilisearch,worker,sdk}` | People + content + skills + companies + jobs + events search; faceted; Cedar-filtered; PHI-redacted in pack-us-healthcare | `SearchDoc`, `SearchQuery`, `SearchResultSet`, `SearchFacet` |
| `accessibility-captions` | `oya-network-accessibility-captions-{kernel,domain,usecase,api,adapter,worker,sdk}` | WCAG 2.2 Level AA caption + alt-text persistence; T1 auto-draft via foundry-runtime; per-media association | `Caption`, `AltText`, `WcagComplianceFlag` |
| `abuse-reporting` | `oya-network-abuse-reporting-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | Professional-context abuse + harassment reports (incl. KR 직장 갑질 category); routing to content-moderation; appeal-workflow input | `AbuseReport`, `HarassmentReportCategory`, `AppealLink` |

Naming justification — `professional-profile`:

```
NAME: oya-network-professional-profile-<layer>
JUSTIFICATION:
- microservice = network: per ADR-0131 per-microservice flat layout.
- bc-tokens = professional-profile: primary BC. ADR-0056 v4.1 BC-optionality rule honoured.
- layer = <layer>: ADR-0105 13-value canonical enum; ADR-0106 usecase rename.
- exemptions claimed: -adapter-postgres / -adapter-s3 / -adapter-meilisearch /
  -adapter-clamav / -adapter-opswat / -adapter-imagemagick / -adapter-ffmpeg /
  -adapter-messenger-bridge / -adapter-calendar-bridge / -adapter-mail-bridge /
  -adapter-ats-bridge are canonical *-adapter-<backend> per ADR-0105 Amendment 3.
```

Total crates introduced: **~165** (24 BCs × 6-10 layers per BC depending on backend variety).

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implementation | Data classes touched |
|---|---|---|---|
| `ProfileRepository` | `oya-network-professional-profile-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING`, `EMPLOYMENT_RECORD` |
| `ProfessionalGraphRepository` | `oya-network-professional-graph-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `RELATIONSHIP_GRAPH` |
| `ConnectionRequestStore` | `oya-network-connection-request-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING` |
| `PostStore` | `oya-network-post-composition-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING` |
| `MediaBlobStore` | `oya-network-post-composition-kernel` | `-adapter-s3` | `BEHAVIORAL_TENANT_PRODUCT`, sometimes `PII_IDENTIFYING` |
| `ImageTranscoder` | `oya-network-post-composition-kernel` | `-adapter-imagemagick` | `INTERNAL_ONLY` |
| `VideoTranscoder` | `oya-network-post-composition-kernel` | `-adapter-ffmpeg` | `INTERNAL_ONLY` |
| `MalwareScanner` | `oya-network-post-composition-kernel` (cross-BC) | `-adapter-opswat` / `-adapter-clamav` | `INTERNAL_ONLY` |
| `FeedCache` | `oya-network-feed-timeline-kernel` | `-adapter-redis` | `BEHAVIORAL_TENANT_PRODUCT` |
| `ReactionCounter` | `oya-network-reactions-kernel` | `-adapter-redis` + `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` |
| `MentionResolver` | `oya-network-mentions-kernel` | `-adapter` (Ontology client) | `PII_IDENTIFYING`, `EMPLOYMENT_RECORD` |
| `TrendStore` | `oya-network-trending-topics-kernel` | `-adapter-postgres` + `-adapter-redis` | `BEHAVIORAL_TENANT_PRODUCT` |
| `NotificationStore` | `oya-network-notifications-kernel` | `-adapter-postgres` + `-adapter-redis` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING` |
| `SearchIndex` | `oya-network-search-kernel` | `-adapter-meilisearch` | `BEHAVIORAL_TENANT_PRODUCT`, `EMPLOYMENT_RECORD` |
| `InMailBridge` | `oya-network-inmail-bridge-kernel` | `-adapter-messenger-bridge` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING` |
| `EndorsementStore` | `oya-network-endorsement-engine-kernel` | `-adapter-postgres` | `EMPLOYMENT_RECORD`, `AUDIT` |
| `EndorsementSigner` | `oya-network-endorsement-engine-kernel` | `-adapter` (audit-chain client) | `AUDIT`, `SECRET` |
| `PageStore` | `oya-network-pages-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` |
| `GroupStore` | `oya-network-groups-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING` |
| `CalendarBridge` | `oya-network-events-bridge-kernel` | `-adapter-calendar-bridge` | `BEHAVIORAL_TENANT_PRODUCT` |
| `MailBridge` | `oya-network-pages-kernel` (cross-BC into mail) | `-adapter-mail-bridge` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING` |
| `AtsBridge` | `oya-network-jobs-handoff-kernel` | `-adapter-ats-bridge` | `BEHAVIORAL_TENANT_PRODUCT`, `EMPLOYMENT_RECORD` |
| `RecommenderRanker` | `oya-network-feed-timeline-kernel` + `oya-network-recruiter-stub-kernel` | `-adapter` (foundry-runtime client; T2) | `INTERNAL_ONLY`, `EMPLOYMENT_RECORD` |
| `CedarNetworkPolicy` | `oya-network-professional-profile-kernel` (cross-BC) | `-adapter` (Cedar evaluator) | `INTERNAL_ONLY` |
| `AuditChainClient` | (cross-BC) `-kernel` per BC | (cross-BC) `-adapter` to audit-chain µservice | `AUDIT` |

Data-class enforcement: `oya-check-data-class` LEAN lane refuses unannotated fields. `EMPLOYMENT_RECORD` is a `network`-specific data class because the EU AI Act Annex III §4 + EEOC + KR 근로기준법 + APPI labor + LGPD CLT employment-record protections demand a distinct class with stricter handling than generic `PII_IDENTIFYING`.

Cross-product rule: `network` MUST NOT import any other product µservice crate at any layer. Cross-product flows go through Workflow (events) or Ontology (entity reads/writes) or through the explicit bridge adapter pattern (`-adapter-messenger-bridge`, `-adapter-calendar-bridge`, `-adapter-mail-bridge`, `-adapter-ats-bridge`) where the adapter is a thin client that emits a typed Workflow event to the target µservice. LEAN-A2 lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice network` — dependency-direction
- `oya gate validate lean-a2 --microservice network` — cross-product-refusal
- `oya gate validate port-location --microservice network`
- `oya gate validate layer-correctness --microservice network`
- `oya gate validate per-microservice-layout --microservice network`
- `oya gate validate statelessness --microservice network`
- `oya gate validate shardability --microservice network`
- `oya gate validate authority-cohesion --microservice network` (HG-NETWORK)
- `oya gate validate professional-context-isolation --microservice network` (per ADR-NET-0001)
- `oya gate validate eu-ai-act-employment-conformance --microservice network` (per ADR-NET-0002)
- `oya gate validate endorsement-chain-integrity --microservice network` (per ADR-NET-0005)
- `oya gate validate jobs-handoff-contract --microservice network` (per ADR-NET-0004)

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine |
|---|---|---|---|
| `ProfileCreated` / `ProfileUpdated` / `ProfileDeleted` | end-user signup / edit / delete | ontology (`Person` write), audit-chain, search-index | append-only |
| `ConnectionRequestOpened` / `Accepted` / `Rejected` / `Ignored` / `Withdrawn` | end-user connection lifecycle | professional-graph, notifications, audit-chain | append-only state machine |
| `ConnectionEstablished` | bidirectional acceptance | feed-timeline, notifications, audit-chain | append-only |
| `FollowEdgeAdded` / `FollowEdgeRemoved` | end-user follows | feed-timeline, notifications, audit-chain | append-only |
| `PostPublished` / `PostEdited` / `PostDeleted` | end-user post lifecycle | search-index, mentions, feed-timeline, trending-topics, audit-chain, downstream Workflow engines | append-only / delta / tombstone |
| `ArticlePublished` | long-form article publish | search-index, feed-timeline, audit-chain | append-only |
| `DocumentAttached` | post carries a document | search-index (with PHI redactor), audit-chain | append-only |
| `PollPublished` / `PollVoteCast` | poll lifecycle | feed-timeline, audit-chain | append-only |
| `RepostCreated` / `ShareCreated` / `CommentPublished` | engagement | feed-timeline, mentions, audit-chain | append-only |
| `ReactionAdded` / `ReactionRemoved` | end-user reacts | feed-timeline, downstream engines | append-only |
| `EndorsementAdded` / `EndorsementRevoked` | skill endorsement lifecycle | endorsement-chain (audit-chain), search-index, ontology, recommender | append-only with signed seal |
| `RecommendationPublished` / `RecommendationRevoked` | long-form recommendation lifecycle | endorsement-chain, audit-chain, search-index, ontology | append-only with signed seal |
| `InMailSent` / `InMailDelivered` / `InMailRead` | InMail bridge to messenger | messenger µservice, notifications, audit-chain | append-only |
| `MentionEmitted` | mentions BC resolves a mention | notifications, messenger bridge, action-card consumer (Workflow Studio) | append-only |
| `HashtagEmission` | post carries hashtags | trending-topics, search-index | append-only |
| `ModerationVerdictEmitted` / `AppealOpened` / `AppealResolved` | classifier or reviewer issues verdict | feed-timeline (hide/show), notifications (sender), audit-chain | append-only |
| `AbuseReportFiled` | end-user files Professional-context abuse report | content-moderation, audit-chain | append-only |
| `HarassmentReportFiled` (KR 직장 갑질 category) | end-user files 직장 갑질 abuse | content-moderation, audit-chain, ops-security | append-only with elevated severity |
| `VerificationBadgeIssued` / `Revoked` | tenant-admin or ID-attest flow | feed-timeline, search-index, audit-chain, ontology | append-only |
| `EmployerAttestationGranted` / `Revoked` | employer-confirm verification flow | profile-verification, audit-chain | append-only |
| `SkillAssessmentPassed` / `Failed` | skill-assessment quiz | profile-verification, audit-chain, ontology | append-only |
| `PageCreated` / `PageUpdated` / `PageDeleted` | Page lifecycle | ontology (`Company` write), audit-chain | append-only |
| `PageNewsletterSendRequested` | Page newsletter | mail µservice, audit-chain | append-only |
| `GroupCreated` / `GroupMemberJoined` / `GroupMemberLeft` | group lifecycle | feed-timeline, notifications, audit-chain | append-only |
| `NetworkEventCreated` / `NetworkEventRsvped` | events lifecycle | calendar µservice, notifications, audit-chain | append-only |
| `JobPostingPublished` / `JobPostingUpdated` / `JobPostingClosed` | jobs lifecycle | ATS µservice (Tier G), search-index, audit-chain | append-only |
| `JobApplicationReferred` | applicant refers to ATS | ATS µservice, audit-chain | append-only |
| `RecruiterSearchInvoked` (when recruiter-stub enabled) | recruiter-search ranker invocation | audit-chain (with bias-audit), ops-compliance | append-only |
| `BiasAuditEmitted` | per-release / per-invocation bias-audit | observability, audit-chain, ops-compliance | append-only |
| `AutomatedDecisionOptOutRecorded` (GDPR Art. 22) | end-user opt-out | recommender + recruiter-stub, audit-chain | append-only |
| `EDiscoveryHoldOpened` / `Closed` | compliance-officer action | audit-chain, retention-purge worker | append-only |
| `FourEyesDisclosureExecuted` | tenant-admin pair approves Professional PII or InMail body read | audit-chain | append-only |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `OntologyEntityChanged` (Person/Company/Skill/Certification/Education) | ontology | mentions, search | refresh mention-resolution cache + search-index |
| `MessengerInMailDeliveryAck` | messenger | inmail-bridge | mark InMail delivered |
| `MessengerInMailReadReceipt` | messenger | inmail-bridge | mark InMail read |
| `CalendarEventScheduled` | calendar | events-bridge | confirm event scheduled |
| `MailNewsletterSent` | mail | pages | mark newsletter dispatched |
| `AtsJobApplicationProcessed` | ATS µservice (Tier G) | jobs-handoff | update referral state |
| `TenantRetentionPolicyUpdated` | tenancy | professional-profile + post-composition | reassign retention bounds |
| `AuditChainSealed` | audit-chain | (read-only) | confirm audit-write durability |
| `WorkflowStudioRunStarted/Completed` | workflow-engine | notifications | post status into bound profile |
| `FoundryRuntimeClassifierVersionPromoted` | foundry-runtime | content-moderation + recommender + recruiter-stub | reload model handle + bias-audit re-run |

### Ontology writes

| Object Type | Written by BC | Audit trail |
|---|---|---|
| `Person{user_id, tenant_id, context_kind: Professional, handle, verification_state, employment_record_summary}` | `professional-profile` | Ed25519 |
| `Company{company_id, tenant_id, owner_page_id, verified}` | `pages` | Ed25519 |
| `Skill{skill_id, canonical_label, taxonomy_ref}` | `professional-profile` + `skill-assessments` | Ed25519 |
| `Certification{certification_id, issuer, expiry}` | `professional-profile` | Ed25519 |
| `Education{education_id, institution, degree, period}` | `professional-profile` | Ed25519 |
| `JobPosting{job_id, posting_company_ref, requirements, location, posted_at}` | `jobs-handoff` | Ed25519 |
| `ProfessionalRelation{a_ref, b_ref, kind: connection|follow, established_at}` | `professional-graph` | Ed25519 |
| `Mention{post_id, target_ref, mention_kind}` | `mentions` | Ed25519 |
| `Endorsement{endorser_ref, endorsee_ref, skill_ref, signed_at, ed25519_signature}` | `endorsement-engine` | Ed25519 (chain) |
| `Recommendation{author_ref, recipient_ref, body_hash, signed_at, ed25519_signature}` | `endorsement-engine` | Ed25519 (chain) |

### Ontology reads

| Object Type | Read by BC | Query shape |
|---|---|---|
| `Person`, `Company`, `Skill`, `Hashtag` | `mentions` | `find_by(@-handle or label, tenant_id)` for mention resolution |
| `JobPosting` | `search` + `recruiter-stub` | faceted lookup |
| `RetentionPolicy` | `professional-profile` + `post-composition` + `inmail-bridge` | `lookup(tenant_id, context_kind)` |
| `TenantContext` | every BC | tenant scope + pack jurisdiction lookup + employment-law overlay |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| LinkedIn (Microsoft) | global Professional network + jobs + learning + recruiter | full feature parity; verified profile; InMail; endorsements; recommendations; jobs; recruiter | `learn.microsoft.com/linkedin` |
| Xing | DACH-market Professional network | German-market focus; events | `dev.xing.com` |
| Wantedly | Japan-market Professional network | startup-focused; culture-match | `wantedly.com/developers` |
| VietnamWorks / JobStreet (SEAGroup) | SEA-market jobs + Professional profiles | jobs-first; localised | (limited public docs) |
| Glassdoor | employer-review subset | reviews + salary insights | `glassdoor.com/developer` |
| AngelList / Wellfound | startup talent + jobs | startup-focused jobs | `wellfound.com` |
| Hashnode | developer-subset Professional blogging | technical-blog-first | `hashnode.com/api` |
| Polywork | portfolio-style Professional profile | non-LinkedIn aesthetic | `polywork.com` |
| Lunchclub | AI-matched 1:1 Professional intros | AI-matching | (limited public docs) |
| Bumble Bizz | bizz-mode of Bumble | swipe-based Professional networking | (limited public docs) |
| Shapr | Tinder-style Professional networking | swipe-based intros | (limited public docs) |
| Slack Communities | Professional groups within Slack | community-first | `api.slack.com` |
| Indeed | jobs-first | jobs search + apply | `developer.indeed.com` |
| Monster | jobs-first | jobs + resume | `partner.monster.com` |
| ZipRecruiter | jobs-first | recruiter outreach | `ziprecruiter.com/api` |
| Lever / Greenhouse | ATS-as-a-service | ATS pipeline | `lever.co/developer` / `developers.greenhouse.io` |

Key parity gaps to close (ordered by priority):

1. **EU AI Act high-risk transparency for employment** — none of the competitors ship Annex III §4 transparency labels at the feature level; oyatie ships from day-1 per `capabilities/T2-auto.yaml` + ADR-NET-0002.
2. **Endorsement-chain integrity** — competitors store endorsements as plain counters; oyatie cryptographically signs each endorsement (Ed25519 chain via audit-chain µservice) per ADR-NET-0005.
3. **Native Workflow + Ontology integration** — competitors expose REST + webhooks; oyatie exposes typed Workflow events + Ontology object writes natively.
4. **OpenSLO + agentic gate** — none gate feature rollouts on SLO compliance; oyatie does (per ADR-0130).
5. **Multi-pack residency + per-pack employment-law overlays (KR 직장 갑질 + EEOC + Equal Treatment Directives)** — competitors are SaaS-region-coarse; oyatie is per-pack jurisdiction-pinned.
6. **GDPR Art. 22 right-to-human-review surfaced per-decision** — competitors offer at most a privacy-settings toggle; oyatie surfaces per-recommendation + per-recruiter-search + per-jobs-ranking.
7. **Profile portability (vCard 4.0 + JSON Resume)** — LinkedIn provides a basic PDF export only; oyatie provides RFC 6350 + open-standard JSON Resume + GDPR Art. 20 native portable JSON per ADR-NET-0006.
8. **Clean ATS-handoff seam** — competitors entangle posting + pipeline mgmt; oyatie cleanly separates posting (network µservice) from pipeline (Tier-G ATS µservice) per ADR-NET-0004.

## Performance Targets

(See Performance NFR table above.)

Error budget:
- Monthly error budget for feed-render + profile-view + connection-action: 0.05 % (≈ 22 min/month).
- Burn-rate alarm on `network.feed-render.availability` is 14.4× burn rate over 1h.
- Error budget policy: `microservices/network/runbooks/error-budget-policy.md` (Slice B).

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed`. Postgres for profiles + posts + connection-graph + endorsements + recommendations + jobs + pages + groups; Redis for feed cache + reaction counters + presence + InMail rate-budget; S3 for media + document attachments; Meilisearch for people + content + skills + jobs + companies + events search.

**Active-active compatibility**: stateless REST + worker pods + Postgres logical-replicated within pack; Redis primary-replica HA; S3 cross-AZ replication.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Active Professional users / cell | 1M | 10M | profile-view p99 > 200ms |
| Posts/sec sustained | 500 | 10k | Postgres write IOPS > 70% |
| Profiles per tenant | 1k | 5M | per-tenant cardinality limit hit |
| Connection edges per cell | 200M | 10B | adjacency-list shard saturation |
| Media/day | 50k | 2M | S3 PUT rate > 70% provisioned |
| Search index size | 200GB | 8TB | shard count exceeded |
| InMails/sec sustained | 50 | 1k | messenger-bridge throughput cap |
| Endorsements/sec sustained | 100 | 2k | audit-chain emit throughput cap |
| Jobs postings (active) | 100k | 5M | facet-cardinality limit |

Scale-out policy:
- HPA on REST pods: CPU > 70 %, min 6, max 200 replicas.
- Postgres shard-by-tenant once cell hits 10k posts/sec aggregate.
- Redis cluster sharding by `(tenant_id, user_ref) mod N`.
- Connection-graph: adjacency-list sharded by `(tenant_id, a_ref mod N)`.

Sharding:
- Profile store partitions by `(tenant_id, user_ref mod N)`.
- Post store partitions by `(tenant_id, author_ref, year-month)`.
- Connection-graph partitions by `(tenant_id, a_ref mod N)`.
- Endorsement store partitions by `(tenant_id, endorsee_ref mod N)`.
- Feed cache partitions by `(tenant_id, user_ref mod N)`.
- `oya-check-shardability-cli` lane verifies partition keys are present in every kernel struct.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | Profile-create + connect (request + accept) + post + endorse + react + comment roundtrip completes within p99 < 300ms profile-touch | `microservices/network/tests/e2e/profile-connect-endorse.rs` |
| AC-02 | Personal `social` profile cannot post as a Professional network entity (compile-time + runtime refusal) | `tests/e2e/professional-context-isolation.rs` |
| AC-03 | Professional InMail body admin disclosure requires two distinct approving principals + audit-chain seal | `tests/e2e/four-eyes-disclosure-inmail.rs` |
| AC-04 | Media upload + document attach + scan + transcode + finalize + revoke after retention TTL | `tests/e2e/media-document-lifecycle.rs` |
| AC-05 | @mention of Person + Company + Skill resolves via Ontology and emits `MentionEmitted` within 250ms | `tests/e2e/mention-emit.rs` |
| AC-06 | Notification fanout to 30k followers within 2s p99 | `tests/e2e/notification-fanout.rs` |
| AC-07 | People + content + skills + jobs + companies + events search returns only Cedar-permitted results | `tests/e2e/search-cedar-scope.rs` |
| AC-08 | Endorsement chain: 10 sequential endorsements verified via audit-chain replay; tampered count rejected | `tests/e2e/endorsement-chain-integrity.rs` |
| AC-09 | Moderation classifier verdict → audit-chain seal within 2s + appeal-workflow opens | `tests/e2e/moderation-appeal.rs` |
| AC-10 | InMail-send: network → messenger bridge end-to-end < 100ms p95 | `tests/e2e/inmail-bridge.rs` |
| AC-11 | Jobs-handoff: `JobPostingPublished` event delivered to ATS µservice within 1s; ATS ack reaches network within 1s | `tests/e2e/jobs-handoff-ats.rs` |
| AC-12 | Profile export: vCard 4.0 + JSON Resume + GDPR Art. 20 JSON returned within 300ms p95 | `tests/e2e/profile-export.rs` |
| AC-13 | Recruiter-stub OFF by default; tenant-admin opt-in required + NYC Local Law 144 bias-audit pre-condition | `tests/e2e/recruiter-stub-default-off.rs` |
| AC-14 | GDPR Art. 22 opt-out: opted-out user receives no automated ranking on own profile; ranker emits human-review-only verdict | `tests/e2e/gdpr-art-22-opt-out.rs` |
| AC-15 | EU AI Act Annex III §4 transparency label appears on every recruiter-search + jobs-ranking + endorsement-aggregation decision on pack-eu | `tests/e2e/eu-ai-act-employment-transparency.rs` |
| AC-16 | Bias audit: recruiter-stub ranker reports 4/5-rule disparity ratio across protected groups; deployment refused above threshold | `tests/e2e/recruiter-bias-audit.rs` |
| AC-17 | KR 직장 갑질 harassment-report routes to dedicated category + elevated severity audit-chain seal | `tests/e2e/kr-workplace-harassment.rs` |
| AC-18 | `oya gate validate per-microservice-layout --microservice network` exit 0 | ADR-0131 lane |
| AC-19 | `oya gate validate authority-cohesion --microservice network` exit 0 | ADR-0123 lane; HG-NETWORK registered |
| AC-20 | `oya gate validate professional-context-isolation --microservice network` exit 0 | per ADR-NET-0001 |
| AC-21 | `oya gate validate eu-ai-act-employment-conformance --microservice network` exit 0 | per ADR-NET-0002 |
| AC-22 | `oya gate validate endorsement-chain-integrity --microservice network` exit 0 | per ADR-NET-0005 |
| AC-23 | `oya gate validate jobs-handoff-contract --microservice network` exit 0 | per ADR-NET-0004 |
| AC-24 | Salary-insights stub returns aggregate only; per-individual disclosure refused | `tests/e2e/salary-insights-aggregate-only.rs` |
| AC-25 | Recruiter-tooling-stub + services-marketplace-stub + learning-stub default OFF | `tests/e2e/stubs-default-off.rs` |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Recruiter-tooling-stub: keep interface-only-pending-impl indefinitely vs activate-with-tenant-opt-in vs delete; tied to NYC Local Law 144 + CA AB-331 readiness | council-architecture + gtm + ops-legal | ADR-NET successor-IP after M03 |
| 2 | Services-marketplace-stub fate: activate in M04 vs delete | council-architecture + gtm | ADR-NET successor-IP after M04 |
| 3 | Learning-stub fate: build native vs partner integration vs delete | council-architecture + gtm | ADR-NET successor-IP after M05 |
| 4 | Endorsement aggregation: should aggregated endorsement count feed feed-ranking + recruiter-search ranker, or stay display-only? Implications for EU AI Act high-risk classification | council-privacy + axis-network + axis-foundry-runtime | ADR-NET-0002 references; successor-IP after M03 |
| 5 | Jobs-handoff: should `network` host job-posting authoring or only the surface? Boundary with Tier-G ATS µservice | council-architecture + axis-network + axis-ats | ADR-NET-0004 references; successor-IP |
| 6 | Federation (ActivityPub or AT Protocol) for Professional-context-only: should Professional network ever federate? Currently NO; revisit if Bluesky-style Professional federation emerges | council-architecture + axis-network | ADR-NET successor-IP if demand emerges |
| 7 | Salary-insights: data source — opt-in user-self-report vs market-data-vendor (Glassdoor-style) vs delete | council-privacy + axis-network + gtm | ADR-NET successor-IP |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0008 | Data Use Boundary | personal/professional data-use invariants |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum + Amendment 3 | layer + backend-qualified authority |
| ADR-0106 | application → usecase | layer naming |
| ADR-0135 | Connect dissolution (parallel) | Professional-context isolation source; network as a sibling µservice; distinct from social |
| ADR-0130 | Agentic SLO-gated promotion | gates network releases |
| ADR-0131 | Per-microservice flat layout | this PRD authored under it |
| ADR-0132 | Suite-and-bundle dissolution | factored Connect into surfaces |
| ADR-0133 | Industry best-practice conformance | HG-NETWORK under this |
| ADR-0134 | Hyperscaler maturity claims | network claims under this |
| ADR-NET-0001 | Professional-graph-storage + context-isolation | this µservice |
| ADR-NET-0002 | Recommender + recruiter EU AI Act + EEOC bounds | this µservice |
| ADR-NET-0003 | InMail bridge to messenger | this µservice |
| ADR-NET-0004 | Jobs handoff to ATS | this µservice |
| ADR-NET-0005 | Endorsement chain integrity | this µservice |
| ADR-NET-0006 | Profile portability and export (vCard 4.0 + JSON Resume) | this µservice |
| ADR-SOC-0005 | Dual-context-feed-isolation (sibling) | paired with this µservice; `network` is the Professional pillar |
| Bominal ADR-0208 | Connect dual-context unified channel hub | inherited |
| Bominal ADR-0215 | Connect retention legal-hold dual-context | inherited |
| Bominal ADR-0028 | Audit-chain Merkle + Ed25519 | inherited |
| Bominal ADR-0111 | Ciphertext property type + envelope encryption | inherited |
