---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-community
microservice: community
status: Accepted
tier: product
tier_subtype: product-consumer-community
service_classification_rationale: |
  The community µservice is a tenant-scoped end-user surface. Its primary purpose
  is the human-facing community surface (spaces, channels, threads, Q&A, KB
  articles, polls, events) consumed by tenant members. It calls substrates
  (ontology, intelligence, audit-chain, policy-engine, identity, tenancy,
  workflow-engine, comms-email, consent-graph, observability) and never serves
  as a foundational substrate for other µservices. Per ADR-0245 §D-3.B
  classification.
tier_certified_at: 2026-05-20
tier_promotion_history:
  - from: shared-substrate (pre-ADR-0245 holding pattern)
    to: product (product-consumer-community)
    via: ADR-0245
    at: 2026-05-20
milestone_first_ship: M02-product-launch
bominal_source: [ADR-0208]
related_adrs:
  - ADR-0028
  - ADR-0056
  - ADR-0105
  - ADR-0106
  - ADR-0131
  - ADR-0132
  - ADR-0135
  - ADR-0139
  - ADR-0145
  - ADR-0150
  - ADR-0174
  - ADR-0183
  - ADR-0211
  - ADR-0218
  - ADR-0220
  - ADR-0240
  - ADR-0241
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0246
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
  - ADR-COMM-0001
  - ADR-COMM-0002
  - ADR-COMM-0003
  - ADR-COMM-0004
related_specs:
  - /specs/per-microservice-flat-layout.json
  - /specs/microservice-tier-classification.json
  - /specs/microservice-dependency-dag.json
  - /specs/tenant-model.json
date: 2026-05-20
owner_team: axis-community
doc_status: published
tenant_scoped: true
audience_modes:
  - B2C-personal
  - B2B-work
benchmarks:
  - discord-servers
  - reddit-subreddits
  - discourse-forums
  - stack-overflow
  - notion-teamspaces
  - github-discussions
  - zendesk-help-center
  - mastodon-fediverse
  - lemmy-threadiverse
  - slack-channels
  - viva-engage
  - salesforce-experience-cloud
  - atlassian-community
---

# PRD-community: Community Surface (Reddit + Teamblind + Handshake + Professional Profile, Day-One)

## 1. Purpose

The `community` µservice is oyatie's tenant-scoped community surface. It launches
**B2C (personal) and B2B (work) day-one** as a single product backed by one
shared substrate stack.

As of Wave 15K on 2026-05-21, `community` is also the successor for the retired
`network` µservice. The old `network` name was ambiguous with networking
infrastructure while its actual artifacts described a LinkedIn-class
professional product. Its substantive professional-profile, connections, InMail,
endorsement, recommendation, jobs, recruiter, skill-assessment, page, and event
surfaces are now community-owned.

The merged product has four pillars:

1. **Reddit-style community forums**: subcommunities, posts, comments,
   threading, voting, ranking-by-vote, tags, roles, and moderator chains.
2. **Teamblind-style anonymous workplace discussion**: verified workplace
   affiliation, anonymous or persona-anchored posting, workplace topic taxonomy,
   and per-workplace moderation.
3. **Handshake-style job search + recruitment**: job board, applications,
   resume submission, employer pages, skill-assessment tie-ins, campus and
   early-career recruiting, and recruiter direct-connect.
4. **LinkedIn-style professional profile + graph subset**: resume/profile,
   experience, skills, connections, mutual connections, InMail-equivalent
   outreach, endorsements, recommendations, recruiter search, and profile
   portability.

LinkedIn-style engagement-optimized text feed, status broadcasting, sponsored
post amplification, and follower-monetization mechanics are explicitly out of
scope. Community posts remain text-first, threaded, moderated, and sorted by
community-value signals such as votes, not by engagement-AI feed optimization.

It is the canonical surface for:

- **Interest-aggregation subreddits + threaded discussion forums** in the
  Reddit and Discourse shape.
- **Workplace-verifiable anonymous discussions** in the Teamblind shape.
- **Job search, applications, recruiter outreach, and employer pages** in the
  Handshake shape, with LinkedIn Jobs/Profile/Recruiter as secondary anchors.
- **Professional identity and graph**: resume/profile, connections, InMail,
  endorsements, recommendations, profile export, and skill assessments.
- **Real-time servers + channels** in the Discord shape (voice optional via
  meet integration).
- **Voted Q&A with accept-answer semantics** in the Stack Overflow / Stack
  Exchange shape.
- **Long-form Knowledge-Base articles with revision history + editorial
  review** in the Notion + Confluence + Zendesk Help Center shape, governed by
  ADR-COMM-0003 (Wikipedia-style revision model).
- **Repository-anchored Discussions** in the GitHub Discussions shape.
- **Fediverse-interoperable communities** via ActivityPub federation in the
  Mastodon / Lemmy shape (post-M02 federation pack).

Per ADR-0242 (`oyatie-is-a-tenant` doctrine) the audience question is answered
by *which tenant is calling*, never by which µservice. `community` serves
every tenant — the `oyatie` org-tenant, B2C personal-tier tenants (one tenant
per individual user), B2B enterprise-tier tenants (one tenant per
customer-org), and ephemeral preview / sandbox tenants.

Per ADR-0245 §D-3.B classification, `community` is a `product` µservice in the
`product-consumer-community` subtype. It consumes substrates and never appears
as a substrate dependency of another µservice.

Per ADR-COMM-0001 the moderation pipeline is the canonical chain-of-
responsibility composition with Cedar policy evaluation + audit-chain seal at
every hop. Per ADR-COMM-0003 KB articles use Wikipedia-style immutable
revision history with tenant-scoped editorial review. Per ADR-COMM-0004 the
search backend is Meilisearch 0.10.0 LTS primary with Tantivy 0.22.x embedded
fallback.

## 2. Audience + Tenant Modes

### 2.1 Tenant modes

The single `community` codebase serves multiple tenant *audience modes*, each
configured per-tenant in `microservices/tenancy/`'s `tenant.audience_type`
field (per ADR-0244 §D-4):

| audience_type | Mode | Example | Default surface |
|---|---|---|---|
| `PLATFORM_OWNER` | oyatie itself | `oyatie.foundry`, `oyatie.devrel`, `oyatie.platform-ops` | Internal-team server + ADR Q&A + KB article publication |
| `B2C-personal` | One individual | `tenant-acme-personal-jane` | Discord-style servers + Reddit-style subreddit aggregation + ActivityPub federation |
| `B2B-work` | One company | `tenant-acme-corp` | Internal team server + KB + Q&A + onboarding flows + Zendesk-style customer-facing help center |
| `B2B-developer` | Plugin / SDK builder org | `tenant-stripe-plugin-team` | GitHub-Discussions-style developer forum tied to plugin repo |
| `EDU` | School / university tenant | `tenant-stanford-cs101` | Class-server + assignment Q&A + course KB |
| `NONPROFIT` | Nonprofit org | `tenant-redcross` | Volunteer-coordination server + KB |

The same data model, BCs, contracts, and substrate dependencies serve all
modes. Per-mode UX defaults (default tab order, default channel templates,
default Cedar role bindings) are *Cedar policy fragments + UX template
overrides*, never separate µservices.

### 2.2 Sub-scope hierarchy under `oyatie` (per ADR-0242 §D-2)

When the platform-owner `oyatie` tenant uses `community`, it operates the
same as any other tenant — under sub-scope principals such as
`oyatie.devrel.community-manager`, `oyatie.foundry.adr-drafter` (author of
ADR-discussion threads), `oyatie.platform-ops.incident-commander` (author of
incident-postmortem KB articles), and `oyatie.security.threat-intel`
(maintainer of the threat-intel KB).

### 2.3 Audience-specific surfaces (UI mode, not µservice mode)

| Surface mode | Activated by | Hides | Surfaces |
|---|---|---|---|
| Personal / fediverse | `audience_type=B2C-personal` and the tenant opts in to federation | Moderator console, Cedar role manager | ActivityPub instance settings, follower graph, cross-instance follow |
| Work / internal | `audience_type=B2B-work` | Public-read toggle (default OFF), federation toggle (default OFF), upvote brigade signals | Onboarding KB linker, HR-integration trigger, support-ticket spawner |
| Customer-facing help center | `audience_type=B2B-work` AND space `surface=public_kb` | Internal threads, employee directory | Anonymous-question intake, ticket-deflection signals, Zendesk-style article voting |
| Developer forum | `audience_type=B2B-developer` | Marketing announcements | Code-block first-class rendering, repo cross-link, SDK release announcements |
| Class server | `audience_type=EDU` | Voting brigades, NSFW filter relaxation | Assignment thread template, grade-book linker, anonymous-question mode |

## 3. Tenant Outcomes

- **TO-1 — One product for personal + work + customer-facing communities.** No
  three product purchases; one URL, one data plane, one audit-chain stream,
  one Cedar policy lattice, three sets of UX templates.
- **TO-2 — Discord-quality DX at Stack-Overflow-quality answer hygiene.**
  Real-time presence + typing indicators + voice channels (via meet
  integration) sit next to voted Q&A with accept-answer semantics.
- **TO-3 — Notion-quality KB writing UX with Wikipedia-quality auditability.**
  Long-form curated content with revision history, editorial workflow,
  attachment store (S3), cross-product ontology links (per ADR-COMM-0003).
- **TO-4 — Auditable moderation by construction.** Every moderation action
  emits a Merkle/Ed25519-sealed audit-chain record at every hop of the
  chain-of-responsibility pipeline (ADR-COMM-0001).
- **TO-5 — Federation for B2C, isolation for B2B.** ActivityPub federation
  works for personal-tier tenants who opt in; B2B tenants stay locked behind
  Cedar tenancy boundaries with federation OFF by default and BLOCKED for
  enterprise pack tenants by default.
- **TO-6 — Cross-product mention + entity resolution.** Mentions resolve
  against messenger (chat), ontology (entities), tenancy (members), drive
  (files), mail (threads), calendar (events). Mentions never cross tenant
  boundaries unless explicit cross-tenant Cedar grant exists.
- **TO-7 — Email-to-post and post-to-email digest.** A space can be
  email-mirror configured (inbound MIME → post; outbound digest → MIME) for
  list-server parity (Mailman, Google Groups, Discourse mail-in).
- **TO-8 — Polls, events, AMAs, announcements as first-class.** Not
  bolt-ons.

## 4. Feature Matrix vs Benchmarks

Wave 15K resets the top-3 product counterparts to **Reddit / Teamblind /
Handshake**. LinkedIn Jobs, LinkedIn Profile, and LinkedIn Recruiter are
secondary anchors only for the jobs/profile/recruiter subset; LinkedIn's
engagement-feed model is not a target. Discourse, Circle, Vanilla Forums,
Discord, Stack Overflow Teams, Zendesk Help Center, and GitHub Discussions
remain tertiary references for the forum, KB, moderation, and developer-forum
subsets.

Legend: `Y` = full parity day-one, `y` = partial parity day-one (gaps
documented), `M02` = M02 launch, `M03+` = scheduled later, `N/A` = not in
scope for this µservice.

### 4.1 Spaces, channels, posts

| Feature | Discord | Reddit | Discourse | Stack Overflow | Notion | GitHub Discussions | Zendesk Help Center | **community** |
|---|---|---|---|---|---|---|---|---|
| Servers / spaces (top-level container) | Y | Y (subreddits) | Y (sites) | Y (sites) | Y (teamspaces) | Y (repos) | Y (help centers) | **Y** |
| Per-space membership + roles | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Per-space public read | Y | Y | Y | Y | y | Y | Y | **Y** |
| Per-space invite-only | Y | y | Y | y | Y | Y | y | **Y** |
| Channels (sub-containers) | Y | N/A | Y (categories) | Y (tags) | Y (sub-pages) | Y (categories) | Y (sections) | **Y** |
| Channel types (text/voice/forum/announcement/Q&A) | Y | y | Y | y | y | y | y | **Y** |
| Threads inside channels | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Posts (top-level content) | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Comments / replies | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Nested comments (deep threading) | Y | Y | Y | Y | Y | Y | y | **Y** (materialised path) |
| Edit + edit history | Y | Y | Y | Y | Y | Y | y | **Y** (revisions sealed) |
| Soft-delete with audit trail | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Markdown body | Y | Y | Y | Y | Y | Y | Y | **Y** (CommonMark + GFM tables + Mermaid + KaTeX) |
| Code blocks (syntax-highlighted) | Y | Y | Y | Y | Y | Y | y | **Y** |
| Image upload | Y | Y | Y | Y | Y | Y | Y | **Y** (S3, ClamAV scan) |
| Video upload | Y | y | Y | y | Y | y | y | **Y** (S3, ≤ 256 MB M02, larger M03 via shorts) |
| File attachment (PDF, ZIP, DOCX) | Y | y | Y | y | Y | Y | Y | **Y** |
| Voice channels (live) | Y | N | N | N | N | N | N | **Y** (via meet substrate; M02) |
| Stage / town-hall channels | Y | N | Y | N | N | N | N | **Y** (M02, via meet) |

### 4.2 Reactions, voting, accept-answer

| Feature | Discord | Reddit | Discourse | Stack Overflow | Notion | GitHub Discussions | Zendesk | **community** |
|---|---|---|---|---|---|---|---|---|
| Reactions (custom emoji) | Y | Y | Y | y | Y | Y | y | **Y** |
| Upvote / downvote | y | Y | Y | Y | N | Y | y | **Y** (hot, top, controversial, wilson_score) |
| Vote tally visible | Y | Y | Y | Y | N | Y | y | **Y** |
| Accept-answer (Q&A mode) | N | N | y | Y | N | Y | y | **Y** (Stack Overflow shape) |
| Best-answer per asker | N | N | y | Y | N | Y | y | **Y** |
| Vote rate limit | N | Y | Y | Y | N | Y | Y | **Y** (≤ 600/min/member; ADR-COMM-0002) |
| Brigade detection | N | Y | Y | Y | N | y | y | **Y** (foundry-guardrails signal) |
| Wilson confidence-interval ranking | N | Y | y | Y | N | y | N | **Y** (per ADR-COMM-0002) |
| Vote weight (reputation-weighted) | N | y | Y | Y | N | N | N | **Y** (configurable per space) |

### 4.3 Q&A, AMA, polls, events, announcements

| Feature | Discord | Reddit | Discourse | Stack Overflow | Notion | GitHub Discussions | Zendesk | **community** |
|---|---|---|---|---|---|---|---|---|
| Q&A mode (one accepted answer) | N | y | Y | Y | N | Y | y | **Y** |
| AMA mode (host + scheduled) | y | Y | y | y | N | y | N | **Y** |
| Polls (single + multi-choice + ranked) | Y | Y | Y | y | Y | Y | N | **Y** |
| Poll closing date | Y | Y | Y | Y | Y | Y | N | **Y** |
| Poll anonymity (anonymous vote) | Y | y | Y | N | N | y | N | **Y** (audit-sealed but not surfaced) |
| Events (RSVP, calendar) | Y | y | Y | N | Y | N | N | **Y** (calendar integration) |
| Announcements (org-wide push) | Y | y | Y | y | Y | Y | Y | **Y** |
| Stickied / pinned posts | Y | Y | Y | y | Y | Y | Y | **Y** |
| Locked threads (no new replies) | Y | Y | Y | Y | Y | Y | Y | **Y** |

### 4.4 Tags, categories, search

| Feature | Discord | Reddit | Discourse | Stack Overflow | Notion | GitHub Discussions | Zendesk | **community** |
|---|---|---|---|---|---|---|---|---|
| Tags (free-form taxonomy) | y | Y | Y | Y | Y | Y | Y | **Y** |
| Tag wiki / per-tag KB linkage | N | y | Y | Y | N | y | N | **Y** (M02; KB cross-link) |
| Tag synonyms | N | Y | Y | Y | N | y | y | **Y** |
| Tag follow / unfollow | N | Y | Y | Y | Y | Y | Y | **Y** |
| Tag-restricted spaces | N | y | Y | Y | Y | Y | Y | **Y** |
| Categories | Y | y | Y | y | Y | Y | Y | **Y** |
| Full-text search | Y | Y | Y | Y | Y | Y | Y | **Y** (Meilisearch primary per ADR-COMM-0004) |
| Filtered search (tag, author, date) | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Typo-tolerant search | Y | Y | Y | Y | Y | y | Y | **Y** (Meilisearch built-in) |
| Cross-BC ranked search | y | y | Y | Y | Y | Y | Y | **Y** (per ADR-COMM-0004 §D5) |
| Search-as-you-type | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Saved searches | y | Y | Y | Y | Y | y | Y | **Y** (M02) |

### 4.5 Member directory, roles, permissions

| Feature | Discord | Reddit | Discourse | Stack Overflow | Notion | GitHub Discussions | Zendesk | **community** |
|---|---|---|---|---|---|---|---|---|
| Member directory (per-space) | Y | y | Y | Y | Y | Y | Y | **Y** |
| Member profile (avatar, bio, badges) | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Member reputation / karma | N | Y | Y | Y | N | y | N | **Y** (per ADR-COMM-0002) |
| Member badges (achievements) | y | N | Y | Y | N | y | y | **Y** |
| Custom roles (per-tenant) | Y | Y | Y | y | Y | y | Y | **Y** (per-tenant Cedar) |
| Role hierarchy | Y | Y | Y | Y | Y | y | Y | **Y** |
| Permission-per-action | Y | y | Y | Y | Y | y | Y | **Y** (Cedar fragment per action) |
| Per-space role override | Y | Y | Y | Y | Y | y | Y | **Y** |
| Invite link (revocable, bounded uses) | Y | y | Y | y | Y | Y | y | **Y** |
| Invite by email | y | y | Y | Y | Y | Y | Y | **Y** |
| SSO / OIDC (B2B) | Y | y | Y | Y | Y | Y | Y | **Y** (Zitadel; identity substrate) |

### 4.6 Moderation tools

| Feature | Discord | Reddit | Discourse | Stack Overflow | Notion | GitHub Discussions | Zendesk | **community** |
|---|---|---|---|---|---|---|---|---|
| Automod (rule-based filter) | Y | Y | Y | y | y | y | Y | **Y** (per ADR-COMM-0001 ClassifierHop) |
| Human moderation queue | Y | Y | Y | Y | y | Y | Y | **Y** (QueueAdmitHop) |
| Spam filter | Y | Y | Y | Y | Y | Y | Y | **Y** (foundry-guardrails) |
| NSFW filter / flag | Y | Y | Y | y | N | N | N | **Y** |
| Content warnings (CW) | y | y | Y | N | N | N | N | **Y** (M02) |
| Banned-words list | Y | Y | Y | Y | y | y | Y | **Y** |
| Slowmode (per-channel rate-limit) | Y | y | Y | y | N | N | N | **Y** |
| Per-member rate limit | Y | Y | Y | Y | y | y | y | **Y** |
| Ban (permanent) | Y | Y | Y | Y | Y | Y | Y | **Y** (two-eyes ≥ 100 posts; ADR-COMM-0001 D-4) |
| Mute (timed) | Y | Y | Y | Y | Y | y | Y | **Y** |
| Warn (visible to user) | Y | Y | Y | Y | y | y | Y | **Y** |
| Reporting (user flag) | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Appeals workflow (re-enter at higher tier) | y | Y | Y | Y | y | y | y | **Y** (AppealHop, DSA Art. 14) |
| Mod log (audit trail) | Y | Y | Y | Y | y | Y | Y | **Y** (Merkle-sealed per ADR-COMM-0001) |
| Mod-team escalation | Y | Y | Y | Y | y | y | Y | **Y** (higher-tier reviewer) |
| Cross-space mod actions | N | y | y | y | N | N | y | **Y** (Cedar scope-gated) |

### 4.7 Knowledge-base articles (Wikipedia-style per ADR-COMM-0003)

| Feature | Discord | Reddit | Discourse | Stack Overflow | Notion | GitHub Discussions | Zendesk | **community** |
|---|---|---|---|---|---|---|---|---|
| Long-form articles | N | y (wiki) | Y | Y (Articles) | Y | y | Y | **Y** |
| Revision history (immutable) | N | y | Y | Y | Y | y | Y | **Y** (per ADR-COMM-0003) |
| Editorial workflow (Draft → Review → Published) | N | N | Y | Y | Y | N | Y | **Y** (state machine) |
| Two-eyes publication | N | N | Y | Y | y | N | Y | **Y** (`submitter_id != reviewer_id`) |
| Rollback to prior revision | N | y | Y | Y | Y | y | Y | **Y** (revision append) |
| Attachments per revision | N | N | Y | Y | Y | y | Y | **Y** (S3-backed; ADR-COMM-0003 §8) |
| Cross-product ontology links | N | N | N | N | y | N | y | **Y** (per ADR-COMM-0003 §10) |
| Conflict detection (Wikipedia-style) | N | N | y | Y | y | N | y | **Y** (optimistic concurrency) |
| Fork / branch / merge (Git-style) | N | N | N | N | N | N | N | **N** (rejected per ADR-COMM-0003 Alt-B; explicit non-goal) |
| Category / collection grouping | N | N | Y | Y | Y | N | Y | **Y** |
| Article-level voting (helpful?) | N | N | Y | Y | N | N | Y | **Y** |
| Article-attached Q&A thread | N | N | Y | Y | Y | y | Y | **Y** |

### 4.8 Federation, embeds, mentions, notifications

| Feature | Discord | Reddit | Discourse | Stack Overflow | Notion | GitHub Discussions | Zendesk | **community** |
|---|---|---|---|---|---|---|---|---|
| ActivityPub federation (Mastodon/Lemmy) | N | N | y (plugin) | N | N | N | N | **M02-pack** (B2C-personal opt-in) |
| Matrix federation | N | N | y | N | N | N | N | **M03** (evaluate) |
| Embed posts in other sites (oEmbed) | y | Y | Y | Y | y | Y | Y | **Y** (M02) |
| Mentions (`@user`) resolved cross-product | y | y | Y | y | Y | Y | y | **Y** (per messenger substrate) |
| Mentions in markdown render | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Channel mention (`#channel`) | Y | y | Y | y | y | y | y | **Y** |
| Role mention (`@role`) | Y | N | Y | y | Y | y | N | **Y** |
| Push notifications (in-app, mobile) | Y | Y | Y | Y | Y | Y | Y | **Y** (M02 mobile; in-app M02) |
| Email notifications | Y | Y | Y | Y | Y | Y | Y | **Y** (comms-email substrate) |
| Email digest (daily / weekly) | y | Y | Y | Y | Y | Y | Y | **Y** |
| RSS feed (per-tag, per-channel) | y | Y | Y | Y | N | Y | Y | **Y** |
| Email-to-post (mailing-list mode) | N | N | Y | N | N | N | N | **Y** (per FR-21) |
| Post-to-email mirror | N | N | Y | N | N | N | N | **Y** |
| Webhook outbound | Y | Y | Y | Y | Y | Y | Y | **Y** |

## 5. B2C Personal Mode

### 5.1 Hobby communities (Discord-server-like)

A B2C personal tenant (`tenant-personal-jane-doe`) creates a "server" called
"NYC Cyclists." The server has:

- **Real-time text channels** (`#general`, `#routes`, `#repairs`).
- **Voice channels** (`#sunday-ride-call`) via the meet substrate.
- **Forum channel** (`#race-strategy`) with threaded posts.
- **Q&A channel** (`#bike-mechanic-help`) with Stack-Overflow-shape voting +
  accept-answer.
- **Announcements channel** (`#group-rides-announced`).
- **Events** (`Sunday 8am Hudson River loop`) with RSVP.
- **Polls** (`Where should we ride next weekend?`).
- **Member directory + invite links** (revocable, bounded uses).
- **Roles** (`Owner`, `Moderator`, `Verified Rider`, `Newcomer`).

The server is **invite-only by default**. Owner can flip to public-read.
Public-read servers can opt into **fediverse federation** so Mastodon users
can follow the server's announcement channel.

### 5.2 Interest aggregation (Reddit-style)

A B2C personal tenant can create a "subreddit-style" space:
`personal-cycling/r/cycling`. The space:

- Is **public-read** by default (Reddit-shape default).
- Has **upvote + downvote** with `hot`, `top`, `controversial` sort modes per
  ADR-COMM-0002.
- Has **Wilson confidence-interval ranking** for Q&A sub-mode.
- Has **brigade detection** via foundry-guardrails ClassifierHop.
- Has **tag taxonomy** (`gear`, `nutrition`, `events`, `road`, `gravel`,
  `trail`).
- Federates over ActivityPub so Lemmy users can subscribe.

### 5.3 ActivityPub federation (fediverse interop)

For B2C personal tenants who opt in, the space implements:

- **ActivityPub Server-to-Server protocol** (W3C Recommendation 2018-01-23,
  refreshed 2024 living spec).
- **Actor** (`https://oyatie.com/community/@nyc-cyclists`) discoverable via
  WebFinger.
- **Inbox / Outbox** for `Create`, `Update`, `Delete`, `Follow`,
  `Announce`, `Like`, `Undo` activities.
- **NodeInfo 2.1** endpoint advertising oyatie's federation posture.
- **Mastodon-compatibility** for the `Create` → `Note` shape (text posts).
- **Lemmy-compatibility** for the `Page` shape (link posts) and the
  community-level `Group` actor.

Federation defaults:

- B2C-personal: opt-in per space.
- B2B-work: **blocked by default**; enterprise-pack tenants cannot enable
  without ops-compliance approval.
- `oyatie` org-tenant: federation enabled for the public DevRel community
  space; blocked for internal-ops sub-scopes.

### 5.4 Personal-tier safety rails

- Federated content is **moderated by the same chain** as native content
  (ADR-COMM-0001 hops apply).
- Inbound federated posts pass the ClassifierHop before being admitted to the
  queue.
- Per-tenant **federation blocklist** (deny incoming activities from specific
  fediverse instances; default oyatie-curated blocklist of known
  abuse-spreader instances).
- **Content warnings** are honoured both inbound and outbound (Mastodon
  `summary` field → community CW; community CW → ActivityPub `summary`).

## 6. B2B Work Mode

### 6.1 Internal company KB

A B2B-work tenant (`tenant-acme-corp`) deploys an internal KB:

- Spaces by team: `engineering`, `product`, `support`, `hr`, `finance`.
- Each space is **invite-only**; SSO via Zitadel (identity substrate).
- KB articles with editorial workflow (Draft → Pending Review → Published)
  per ADR-COMM-0003.
- **Onboarding KB article** triggers a workflow-engine flow that sends the
  new hire a welcome email (comms-email substrate), enrols them in the
  `acme-corp/onboarding` space, and books a calendar slot with their
  manager (calendar substrate).
- **HR-integration**: when an employee leaves, the HR substrate fires a
  workflow that tombstones their author_id across all their revisions per
  GDPR Art. 17 / KR PIPA Art. 36 (DSAR cascade per ADR-0242 Appendix B).

### 6.2 Ticket forums (Zendesk Help Center-shape)

A B2B-work tenant can expose **customer-facing help center** spaces:

- Anonymous-question intake (no login required to read; OAuth or email
  verification to post).
- KB articles are public-read; voting on "Was this article helpful?" is
  enabled.
- Tickets can be **spawned from forum questions** via workflow-engine →
  ticket-system integration.
- **Ticket deflection signals**: when a question gets a high-vote KB
  article reply, the ticket-system suppresses the spawn.

### 6.3 Engineering / Q&A (Stack Overflow Teams-shape)

A B2B-work tenant's `engineering` space defaults to Q&A mode with:

- Stack-Overflow-shape accept-answer.
- Wilson-confidence-interval ranking.
- Reputation-weighted voting (configurable per space).
- Code-block-first rendering.
- Cross-link to PR / issue / commit via ontology entity bindings.

### 6.4 B2B integrations (mail / messenger / HR)

The community µservice exposes integration hooks:

- **mail substrate**: post-to-email digest, email-to-post intake, `@`-
  mention surfacing in inbox.
- **messenger substrate**: cross-product mentions; messenger threads can
  be promoted to community Q&A; community accepted answers can be pinned
  back to a messenger channel.
- **calendar substrate**: events in community spaces appear on members'
  calendars; calendar event reminders post to space announcement channel.
- **workflow-engine substrate**: any post action can be a workflow trigger
  (e.g., "when KB article published with `policy-change` tag, fan-out to
  all-hands announcement channel + Slack-compatible mirror channel").
- **HR substrate** (reserved): onboarding / offboarding / role-change
  triggers consume community events.

### 6.5 B2B-work safety rails

- Federation OFF by default; admin override requires ops-compliance grant.
- Per-tenant data residency per ADR-0240 sovereign-cloud-per-pack.
- BAA (HIPAA) automatic via pack-us-healthcare if active.
- ePHI handling: posts containing PHI flagged at submit-time; reviewer
  must be HIPAA-trained (per ADR-COMM-0003 §Regulatory).

## 7. Moderation Pipeline (per ADR-COMM-0001)

The canonical pipeline is a Chain-of-Responsibility with five fixed hops:

```
PostCreated → ClassifierHop → QueueAdmitHop → ModeratorVerdictHop → [AppealHop?] → AuditSealHop
```

### 7.1 Hops

1. **ClassifierHop** — Calls `foundry-guardrails` substrate for spam / abuse
   / impersonation / NSFW / banned-word / brigade signals. Emits
   `ClassifierVerdict` audit event. Optionally fires `PostShouldHide`.
2. **QueueAdmitHop** — Admits the envelope into the human moderation queue.
   Emits `QueueAdmitted`.
3. **ModeratorVerdictHop** — Human moderator applies a verb (`hide`, `lock`,
   `pin`, `move`, `merge`, `delete`, `ban`, `mute`, `warn`). Two-eyes
   required for `ban` if affecting > 100 posts (per ADR-COMM-0001 D-4).
   Emits `ModerationActioned`.
4. **AppealHop** (optional) — Subject of moderation files an appeal. The
   envelope re-enters at a **higher tier** (different moderator group than
   the original verdict-issuer). Emits `AppealLodged`. Implements EU DSA
   Art. 14 internal complaint mechanism.
5. **AuditSealHop** — Mandatory; cannot be omitted. Seals every hop's
   output with Ed25519 per ADR-0028. Seal latency p99 ≤ 1 s.

### 7.2 Automod rule engine

Per-space tenant-authored rules:

```yaml
# Example automod rule (per-space, authored via UI)
when: post_created
filters:
  body_contains_any: ["buy crypto", "free nft", "telegram link"]
  author_age_lt: 24h
  author_post_count_lt: 5
action: hide
notify: mods
audit_reason: "Possible scam — new user + crypto keywords"
```

Rule firing emits a `ClassifierVerdict` audit event (per ADR-COMM-0001 §3).

### 7.3 Reporting + appeals workflow

```
Member-flags-post → Report queued → ClassifierHop re-scores → ModeratorVerdictHop reviews
  ↓ (if hidden)
Author notified with verdict + reason
  ↓ (if author appeals)
AppealHop → routed to higher-tier moderator (≠ original verdict-issuer)
  ↓
Higher-tier verdict → seals audit event → subject re-notified
```

### 7.4 Cedar policy per hop

Each hop has its own Cedar fragment:

- `policy/moderation-hop-classifier.cedar` — who can run automod, which
  rules.
- `policy/moderation-hop-moderator.cedar` — which roles can act, which
  verbs allowed in which spaces.
- `policy/moderation-hop-appeal.cedar` — who can appeal, who can review.
- `policy/two-eyes.cedar` — destructive verb threshold + second-eye
  requirements.

## 8. User Stories (15+, narrative form)

### US-01 — Alice creates a hobby community for cycling enthusiasts (B2C)

Alice signs up to oyatie with a personal tenant. From the community surface
she clicks `New Server`. She chooses the **Personal / Hobby Server**
template, names it `NYC Cyclists`, sets visibility to `Public-read,
Invite-only-write`, and opts in to ActivityPub federation. She creates four
channels: `#general` (text), `#routes` (forum), `#sunday-ride-call`
(voice), and `#bike-mechanic-help` (Q&A). She invites 30 friends via
shareable invite link (revocable, 100-use cap). Within 24h she has 47
members. From Mastodon, three users discover the server via federation and
subscribe to the announcement channel. *Acceptance:* server is live in <
10s; channels render in < 300ms; invite link works; first federated follow
arrives within 5min.

### US-02 — Bob asks a Q&A question and accepts the best answer (B2C)

Bob, member of `NYC Cyclists`, posts a question in `#bike-mechanic-help`:
"My disc brakes squeal in wet weather, what fixed it for you?" He tags
`brakes`, `wet-weather`, `disc`. Within 2h, 7 answers post. Three accumulate
upvotes (Wilson-confidence-interval rank). Bob clicks `Accept Answer` on the
top-ranked answer. The accepted answer is pinned to the top, the responder
earns a `Verified Helper` badge, and the Q&A pair is indexed for future
search hits on "disc brake squeal." *Acceptance:* accept-answer click flips
ranking in < 250ms; badge appears in responder's profile; question shows
`[Solved]` tag in feed.

### US-03 — Carol moderates her server using automod (B2C)

Carol is a moderator of `NYC Cyclists`. She authors three automod rules:
(a) hide posts with telegram-link keywords if author is < 24h old;
(b) slowmode 30s on `#bike-mechanic-help`; (c) auto-flag posts containing
banned-words. Within a week, 4 spam posts are auto-hidden, 1 false-positive
is appealed by the author; Carol's appeal-review (AppealHop) escalates to
the server owner Alice (higher tier), who restores the post. Every action
seals to audit-chain. *Acceptance:* auto-hide fires in < 1s; appeal flow
routes to non-original-verdict-issuer; full audit trail visible in mod log.

### US-04 — Engineer creates an internal KB article about deployment (B2B)

Dani, an engineer at `tenant-acme-corp`, drafts a KB article titled
"Production Deployment Runbook" in the `engineering` space. She marks it
`Draft`. She submits for review. Her teammate Eve (cannot be Dani per
two-eyes) reviews, requests one revision (add rollback section), Dani
submits revision, Eve approves → `Published`. The article emits a
`KBArticlePublished` audit event; the workflow-engine substrate fires a
flow that pings `#deployments` channel with a link. *Acceptance:* state
machine enforced (Dani can't approve her own draft); revision history
shows three immutable revisions; published article surfaces in cross-BC
search within 5s.

### US-05 — User reports a spam post → appeal → resolution (B2C)

Frank, a member of `NYC Cyclists`, sees a post advertising crypto. He
clicks `Report`. ClassifierHop re-scores the post (high spam confidence);
QueueAdmitHop queues it. Moderator Carol applies `delete + ban` (author
posted 200+ similar posts org-wide, triggers two-eyes; Alice
co-approves). Banned author appeals; AppealHop routes to a third
moderator (not Carol, not Alice). Third moderator reviews evidence,
upholds ban. *Acceptance:* every hop emits one sealed audit event; appeal
routes to non-original-issuer; resolution within 7-day SLA.

### US-06 — Stack-Overflow-style verified answers in B2B engineering Q&A

Gail asks in `tenant-acme-corp/engineering`: "How do we configure
Cilium Network Policies for the Frobnitz cluster?" Five engineers reply.
Voting + reputation-weighting promotes Henry's answer (reputation
multiplier × Wilson score). Cluster lead Iris clicks `Verify Answer`
(diamond-mod equivalent in oyatie). The answer is now marked `Verified`,
indexed for higher search rank, and surfaces first when searching
"Cilium." *Acceptance:* verified answer indexed within 5s; vote weight
visible in dev console.

### US-07 — AMA event with a domain expert (B2C)

A `Personal / Hobby` server hosts an AMA with cycling pro `@chris-froome`.
Owner creates an `AMA` channel typed `q-and-a`, schedules for Saturday
3pm. Pre-AMA, members submit questions; during the AMA, Chris answers
top-voted ones; post-AMA, the channel is locked for new questions but
remains readable. *Acceptance:* AMA channel template enforces ask-mode
during window; locks at scheduled end; recording persists.

### US-08 — Poll with ranked-choice voting (B2C + B2B)

A community owner creates a poll: "Where should we ride next weekend?"
with 5 options + `ranked-choice` mode. Members rank options; closing date
auto-tallies via Borda-count adapter. Result posts back to channel. Audit
events sealed. *Acceptance:* ranked-choice supported; result posts at
closing time without manual intervention.

### US-09 — Event with calendar RSVP (B2C + B2B)

A space creates `Sunday 8am Hudson River loop` event. Members RSVP `Going
/ Maybe / No`. Calendar substrate adds event to each RSVP'd member's
calendar with a 24h reminder. On event day, a reminder posts to the
channel. *Acceptance:* RSVP syncs to calendar in < 5s; reminder fires.

### US-10 — Member directory + role-based permissions (B2B)

`tenant-acme-corp` admin defines roles: `Engineer`, `Engineering-Manager`,
`Product`, `Sales`. Cedar fragments grant per-role action sets
(Engineering can publish to `#engineering`; Sales can read but not post).
Roles assigned via SCIM provisioning from acme's IDP. *Acceptance:*
Cedar policy evaluated < 5ms per request; SCIM sync end-to-end < 60s.

### US-11 — Email-to-post mailing list mode (B2B)

`tenant-acme-corp/announcements` space is configured as `mailing-list`:
inbound emails to `announcements@acme.community.oyatie.com` become posts
(MIME → post; attachments to S3); outbound digest emails to subscribers
contain new posts since last digest. *Acceptance:* email arrival to post
visibility < 30s; digest delivered per subscription schedule.

### US-12 — KB article with attachment + ontology link (B2B)

PM Jules drafts a KB article "Q3 Roadmap" with a PDF attachment and
links to four ontology objects (epics, PRDs). Article published →
ontology relationships surface in the linked entity panels. Rolling back
a revision rolls back the link set (per ADR-COMM-0003 §8). *Acceptance:*
attachment uploads resumable; ontology links bind to revision_id; rollback
deterministic.

### US-13 — Cross-product mention across messenger / drive / mail (B2B)

In a community post, Kim writes "Hey `@liam`, see `[Q3-roadmap.pdf]` and
the discussion in `#planning`." `@liam` resolves to a tenancy member; the
attachment link resolves to a drive object; `#planning` resolves to a
messenger channel. Liam gets a notification across all three substrates.
*Acceptance:* mentions resolve within 100ms; notification surfaces in
inbox + messenger + push within 5s.

### US-14 — Federated cross-instance follow (B2C personal)

A Mastodon user `@nora@mastodon.social` searches for the `NYC Cyclists`
server. WebFinger returns the actor URL; Mastodon issues `Follow`;
community-actor responds `Accept`. Each new public post in the server's
announcement channel federates to Nora's home timeline. *Acceptance:*
WebFinger discovery < 1s; Follow accepted within 5s; subsequent post
visible in Nora's Mastodon timeline within 30s.

### US-15 — Customer-facing help center deflects ticket (B2B)

A potential customer arrives at `acme.com/help` (a public-read community
space). They search "How do I rotate my API key?" Top result is a KB
article with > 80% helpful votes. They read it, mark "Helpful," and don't
file a ticket. *Acceptance:* search returns top KB article < 500ms; vote
recorded; ticket-deflection signal fires (sent to analytics).

### US-16 — Anonymous question in EDU class server

In `tenant-stanford-cs101`, students can submit anonymous questions to
`#anonymous-help`. Instructor sees aggregate counts of question themes
(via analytics). Per ADR-0244, "anonymous" means audit-sealed but
not user-surfaced; instructor with cause + Cedar grant can de-anonymize
under FERPA-equivalent override.

### US-17 — Polls reveal sentiment for product decision (B2B)

PM runs a poll "Which integration should we ship next?" with 5 options.
Engineering reads the result + reads tagged community threads to inform
decision. *Acceptance:* poll embedded inside RFC KB article.

### US-18 — Onboarding KB triggers welcome workflow (B2B)

New hire joins `tenant-acme-corp`. HR substrate fires `EmployeeJoined`.
Workflow-engine fires `OnboardWelcome`: subscribes hire to
`acme-corp/onboarding`; posts a personalized welcome message; posts the
"First-Week Checklist" KB article; books a 1:1 with their manager via
calendar substrate. *Acceptance:* full chain executes < 5min; new hire
sees community + checklist on first login.

### US-19 — Disaster: spam brigade attacks a public community

An attacker launches 5000 sock-puppet accounts that mass-upvote a spam
post in a public-read subreddit-style space. Brigade-detection signal
fires (foundry-guardrails); vote-counter caps brigades per ADR-COMM-0002;
post gets auto-hidden; cluster of accounts gets ban-queued. *Acceptance:*
brigade detected within 60s; vote inflation contained.

### US-20 — KR-tenant operates under PIPA pack

A KR enterprise tenant operates community under pack-kr: data residency
in `ap-northeast-2` cell, audit-chain in KR-resident storage, retention
per PIPA Art. 21 (indefinite for KB), KR-resident Meilisearch instance.
*Acceptance:* no cross-region data leak; KR auditor query returns
KR-resident evidence.

## 9. UX Strive / Avoid

### 9.1 Strive

- **Discord-quality DX**: real-time presence, typing indicators, message
  reactions in < 100ms, voice channels that just work, server-customisable
  emoji.
- **Stack-Overflow-grade voting hygiene**: Wilson-confidence-interval
  ranking, brigade detection, reputation-weighted voting, accept-answer
  semantics, edit-suggestions reviewable.
- **Notion-quality writing UX for KB articles**: rich markdown editor,
  drag-drop attachments, slash-commands for embeds, autosave-with-revision,
  side-by-side revision diff.
- **Discourse-grade community hygiene**: trust levels, slowmode, civility
  filters, threaded forum view with depth-aware lazy load.
- **GitHub Discussions-grade dev forum**: code-block-first rendering,
  repo cross-link, syntax-highlight, theme-aware.
- **Zendesk-grade public help center**: anonymous-question intake, helpful
  votes, ticket deflection.
- **ActivityPub-correct fediverse interop**: NodeInfo 2.1, WebFinger,
  Mastodon/Lemmy actor compatibility.

### 9.2 Avoid

- **Reddit-style downvote brigades**: brigade detection, vote weighting,
  per-tenant rate limits, reputation-aware vote contribution. No raw
  unweighted downvote tally exposed at scale.
- **AMA spam**: AMA channels enforce ask-mode windows; outside windows
  default to locked.
- **KB rot**: stale articles flagged by a freshness signal (age + last
  reviewer + view-to-vote ratio); review-required-after-N-months Cedar
  fragment per space.
- **Low-signal content**: minimum-effort threshold via per-space karma
  gates; "low-effort" hot-key flag option for moderators.
- **Confluence-grade clutter**: no nested taxonomy depth > 4; no
  page-trees that hide structure; flat-categories-with-tags wins.
- **Discord-noisy mentions**: `@everyone` defaults to disabled in
  B2B-work and EDU; requires explicit Cedar permission.
- **Slack-style ephemerality**: posts persist indefinitely by default;
  ephemeral mode is opt-in per channel.
- **Forum-style decay (Reddit hide-after-N-months)**: KB articles never
  archive without explicit action.

## 10. Substrate Dependencies

Per ADR-0245 §D-4 cross-tier dependency rules: `community` (product)
consumes substrates only; never depends on another product.

| Substrate | Purpose | SLO floor consumed |
|---|---|---|
| `tenancy` | Tenant + sub-scope resolution; reserved-namespace check | 99.99% |
| `identity` | OIDC, passkey, service-principal issuance | 99.99% |
| `policy-engine` | Cedar evaluation at every hop, every action | 99.99% |
| `audit-chain` | Merkle/Ed25519 seal on every state change | 99.99% |
| `ontology` | Cross-product entity links; mention resolution | 99.99% |
| `intelligence` | Summarisation, related-Q, search ranking augmentation, AI-moderation classifier | 99.95% |
| `workflow-engine` | Durable workflows (onboarding, KB review, ticket spawn) | 99.95% |
| `comms-email` | Outbound digest, inbound mail-to-post, transactional notifications | 99.95% |
| `consent-graph` | DSAR cascade; consent state | 99.99% |
| `observability` | Metrics, traces, dashboards, SLO authoring | 99.99% |
| `cell` | Per-tenant + per-region cell provisioning | 99.99% |
| `cloud-secrets` | KMS / OpenBao for tenant DEKs | 99.99% |
| `governance` | Fitness gates; oya-check-* lanes | 99.99% |
| `compliance` | Per-pack overlay enforcement (KR, US-HC, EU) | 99.99% |
| `api-gateway` | Request routing, per-tenant rate limit | 99.99% |
| `network` | Service mesh, NetworkPolicy authoring | 99.99% |
| `cloud-k8s` | Compute scheduling | 99.99% |
| `cloud-iac` | Helm chart + IaC module registry | 99.99% |

Optional sibling **products** consumed (product-to-product per ADR-0145):

| Product | Purpose |
|---|---|
| `messenger` | Mention-resolution + thread promotion + cross-product reactions |
| `mail` | Email-to-post intake; post-to-email digest forwarding via comms-email |
| `drive` | Attachment cross-link; file embeds |
| `calendar` | Event RSVP sync; reminders |
| `meet` | Voice / stage channels; AMA broadcast |
| `tasks` | Spawn task from post action |
| `forms` | Embed forms inside posts (surveys) |
| `analytics` | Per-space + per-tenant engagement dashboards (read-only) |

## 11. Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | tenant member | to create a space (server / subreddit / help-center) | I have my own community | spaces | Must |
| FR-02 | tenant member | to create channels within a space (text / voice / forum / Q&A / announcement / stage) | content is organised | channels | Must |
| FR-03 | tenant member | to publish a post / thread / comment in a channel | conversations happen | post-store, thread-tree | Must |
| FR-04 | tenant member | to reply nested ≥ 8 levels deep with lazy load | conversation depth preserved | thread-tree | Must |
| FR-05 | tenant member | to upvote / downvote, react, accept-answer | best content rises algorithmically; vote p99 ≤ 100 ms | voting-engine | Must |
| FR-06 | tenant moderator | to hide / lock / pin / move / merge / delete / ban / mute / warn | community health is maintainable | moderation-queue | Must |
| FR-07 | tenant member | to search across announcements + Q&A + KB + threads | discovery is one entry point; search p99 ≤ 500 ms | search-index | Must |
| FR-08 | tenant member | to subscribe to a space / tag / thread / user with email + push notifications | I receive notifications | notifications | Must |
| FR-09 | foundry-guardrails | to consume `PostCreated` and `PostEdited` events for spam / abuse | I can take moderation action without polling | post-store | Must |
| FR-10 | ontology | to expose cross-product links inside posts + KB | community threads can deep-link entities | post-store, kb-article-store | Must |
| FR-11 | tenant member | to publish a long-form KB article with attachments, revisions, two-eyes review | curated content lives next to threads | kb-article-store | Must |
| FR-12 | tenant member | to roll back a KB article to a prior revision | mistakes are reversible | kb-article-store | Must |
| FR-13 | tenant member | to author + run automod rules per space | spam is filtered by tenant-tuned policy | automod | Must |
| FR-14 | tenant member | to report a post + appeal a moderator decision | DSA Art. 14 internal complaint mechanism honoured | moderation-queue | Must |
| FR-15 | tenant operator | to view per-space usage + moderation metrics | capacity planning + contractual SLA verification | observability | Must |
| FR-16 | tenant member | to create polls (single / multi / ranked-choice) | sentiment + decisions visible | polls | Must |
| FR-17 | tenant member | to create events with RSVP + calendar sync | offline + online meetups coordinated | events | Must |
| FR-18 | tenant member | to host an AMA (ask-mode window with scheduled host) | event-style Q&A works | events, post-store | Must |
| FR-19 | tenant member | to tag posts + KB articles with shared taxonomy | content is discoverable by tag | tags | Must |
| FR-20 | tenant member | to invite members via link or email | onboarding works | invites | Must |
| FR-21 | tenant member | to read RSS feed for any space / tag / channel | consumption via aggregator works | rss | Must |
| FR-22 | tenant operator | to mirror a space to a mailing-list (email-to-post + post-to-email) | list-server parity (Mailman / Discourse mail-in) | email-mirror | Must |
| FR-23 | tenant member | to embed a post via oEmbed in external sites | sharing works | embeds | Must |
| FR-24 | tenant member | to follow another member; see their feed | network effects within tenant | follow-graph | Must |
| FR-25 | B2C-personal tenant | to enable ActivityPub federation per-space | fediverse interop works | federation | Should (M02-pack) |
| FR-26 | tenant member | to receive AI-summarised digest of long threads | I can consume catch-up at scale | intelligence | Should |
| FR-27 | tenant member | to use markdown + code-blocks + math + Mermaid + GFM tables | rich content authoring works | post-store | Must |
| FR-28 | tenant operator | to enforce content warnings + NSFW flags + banned-words | safety rails are tenant-tuneable | moderation-queue | Must |
| FR-29 | tenant member | to slowmode + per-member rate-limit | brigades and spam don't overwhelm channels | moderation-queue, voting-engine | Must |
| FR-30 | tenant member | to use cross-product mentions (`@user`, `#channel`, `&role`) | mention surface unified | mentions | Must |
| FR-31 | tenant operator | to define custom roles + per-role Cedar permissions | tenant-tuned authorization works | roles, policy-engine | Must |
| FR-32 | tenant member | to view member directory + member profiles + badges + reputation | community identity surface | directory | Must |
| FR-33 | tenant operator | to set per-tenant data retention | regulatory compliance | retention | Must |
| FR-34 | DSAR requester | to exercise GDPR Art. 17 / PIPA Art. 36 erasure | their data is removed / tombstoned per ADR-0242 cascade | consent-graph | Must |
| FR-35 | regulator / auditor | to e-discover community evidence | legal hold + audit trail satisfied | e-discovery | Must |
| FR-36 | foundry-guardrails | to consume brigade-detection signals | brigade abuse contained | voting-engine | Must |
| FR-37 | tenant member | to schedule a post (publish at future time) | timezone-friendly publishing | post-store | Should |
| FR-38 | tenant operator | to migrate import from Discord / Slack / Reddit / Discourse / Notion / GitHub / Zendesk exports | switchers can move in | import | Should (M02) |
| FR-39 | tenant operator | to export full tenant data (posts + KB + members) | data portability per GDPR Art. 20 | export | Must |
| FR-40 | webhook subscriber | to receive outbound webhooks for post/vote/mod events | external integrations work | webhooks | Must |
| FR-41 | tenant member | to mute / block another member per-tenant | personal safety | mute-block | Must |
| FR-42 | tenant member | to use mobile + web + desktop app | multi-device | client-apps | Must |
| FR-43 | tenant operator | to brand a community with logo + colour scheme + custom domain | white-label community | branding | Should |
| FR-44 | tenant member | to use accessibility features (screen reader, keyboard nav, high contrast, WCAG 2.2 AA) | universal access | a11y | Must |
| FR-45 | tenant operator | to integrate with HR substrate (onboarding / offboarding triggers) | B2B workflows work | hr-integration | Should |
| FR-46 | tenant member | to maintain a professional profile with resume, experience, education, certifications, skills, headline, and summary | jobs + recruiting identity is portable inside community | professional-profile | Must |
| FR-47 | tenant member | to export that profile as vCard 4.0, JSON Resume, and GDPR Art. 20 portable JSON | profile portability survives the network retirement | professional-profile | Must |
| FR-48 | tenant member | to request, accept, ignore, withdraw, disconnect, block, and restrict professional connections | relationship graph semantics move from network into community | professional-graph, connection-request | Must |
| FR-49 | tenant member | to send InMail-equivalent outreach through the messenger bridge when no connection exists | recruiter and professional outreach works without reopening the retired professional-network service | inmail-bridge | Must |
| FR-50 | tenant member | to give, revoke, and audit skill endorsements and long-form recommendations | reputation and referral evidence is tamper-evident | endorsement-engine | Must |
| FR-51 | tenant member | to take skill assessments and receive passing badges bound to the profile | job-fit and recruiter search can rely on assessed skills | skill-assessments | Must |
| FR-52 | candidate / recruiter | to search jobs, submit applications or referrals, and receive contract-versioned ATS handoff events | Handshake-style recruiting is first-class in community | jobs-recruiter | Must |
| FR-53 | recruiter / tenant admin | to enable recruiter-stub search only after tenant-admin opt-in and required employment-law bias-audit gates | high-risk employment ranking stays gated and auditable | jobs-recruiter | Must |
| FR-54 | employer / brand admin | to manage employer pages, public company profiles, newsletters, and recruiting events | page/event content lands in the community substrate | pages-events | Should |

## 12. Non-Functional Requirements

### 12.1 Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Feed render (per-space) | ≤ 80 ms | ≤ 300 ms | ≤ 800 ms | Valkey hot-feed cache; warm hit |
| Search query (cross-BC, ranked, Cedar-filtered) | ≤ 120 ms | ≤ 500 ms | ≤ 1.2 s | Meilisearch 0.10.0 LTS per ADR-COMM-0004 |
| Vote cast (idempotent) | ≤ 25 ms | ≤ 100 ms | ≤ 300 ms | Valkey-buffered + Postgres flush; per ADR-COMM-0002 |
| Post create | ≤ 80 ms | ≤ 250 ms | ≤ 700 ms | Postgres insert + async fan-out |
| Post edit | ≤ 80 ms | ≤ 250 ms | ≤ 700 ms | Revision append; async reindex |
| KB article publish | ≤ 200 ms | ≤ 500 ms | ≤ 1.5 s | Postgres insert + S3 attachment uploads |
| KB article render (cached) | ≤ 50 ms | ≤ 200 ms | ≤ 500 ms | CDN edge cache |
| Moderation action seal | ≤ 80 ms | ≤ 200 ms | ≤ 500 ms | Postgres update + audit-chain seal |
| Audit-chain seal latency | ≤ 200 ms | ≤ 800 ms | ≤ 1500 ms | per hop; per ADR-COMM-0001 §3 ≤ 1 s |
| Threaded reply render (1000 nodes) | ≤ 100 ms | ≤ 350 ms | ≤ 900 ms | Materialised path + lazy load |
| Notification fan-out (push + email) | ≤ 1 s | ≤ 5 s | ≤ 15 s | comms-email + push substrate |
| Mention resolution | ≤ 30 ms | ≤ 100 ms | ≤ 300 ms | Cross-product cache |
| Federation outbound delivery | ≤ 5 s | ≤ 30 s | ≤ 120 s | ActivityPub queue |
| Real-time presence update | ≤ 100 ms | ≤ 500 ms | ≤ 2 s | WebSocket |

### 12.2 Availability + SLO (product-tier floor)

- Read paths: 99.95% monthly (product-consumer floor per ADR-0245 §D-8 +
  consumer-UX bias).
- Write paths: 99.9% monthly.
- Real-time paths (voice / typing / presence): 99.9% (UX tolerance).
- Federation paths: 99.5% (best-effort).
- RTO ≤ 15 min; RPO ≤ 30 s (Postgres WAL + Valkey AOF).
- Per ADR-0241 DR tier: T2 (< 1h RTO; some throughput degradation tolerable
  on read; T1 for moderation-queue).

### 12.3 Scalability

- **5 TB / cell ceiling** for search index per pack per ADR-COMM-0004.
- **100 GB / tenant** ceiling for Tantivy fallback; Meilisearch primary
  past.
- **1M posts / tenant / month** sustained ingest.
- **10K concurrent WebSocket connections / cell** for real-time.
- **100K queries / sec / cell** for search reads.
- **Per-tenant Citus shard** with horizontal-scale-up to 256 shards / cell.

### 12.4 Security (per ADR-0243 Cedar-as-universal-gate + ADR-COMM-0001)

- All writes authenticated via Zitadel-issued JWT.
- Cedar policy evaluated at every action; per-hop in moderation pipeline.
- Tenant boundary enforced at every layer:
  - Postgres RLS per `tenant_id`.
  - Meilisearch per-tenant indexes.
  - Valkey per-tenant key prefix.
  - S3 per-tenant prefix + tenant-DEK envelope.
- All attachments scanned (ClamAV inline) before publication.
- Cross-tenant mention-resolution forbidden by Cedar fragment.
- Per-tenant rate limits:
  - post create ≤ 60 / min / member
  - vote ≤ 600 / min / member
  - report ≤ 30 / min / member
- All state-changing events emit Merkle/Ed25519-sealed audit records.
- All federation activities verified against signed actor key.
- Encryption: at-rest via tenant-DEK envelope; in-transit mTLS per
  ADR-0148.
- encryption-BYOK per-tenant via cloud-secrets / OpenBao.

### 12.5 Privacy + DSAR

- Per ADR-0242 §D-4 uniform DSAR cascade.
- Per ADR-COMM-0003 §Regulatory: GDPR Art. 17 tombstones author_id;
  revision body retained as community-contributed content under tenant
  licence.
- Per-tenant data residency per ADR-0240 sovereign-cloud-per-pack.

### 12.6 Audit + Compliance

- Append-only audit log: every `PostCreated`, `PostEdited`, `PostDeleted`,
  `VoteCast`, `ReactionAdded`, `ReactionRemoved`, `ModerationActioned`,
  `ClassifierVerdict`, `QueueAdmitted`, `AppealLodged`, `KBArticlePublished`,
  `KBArticleRejected`, `KBArticleArchived`, `RoleGranted`, `RoleRevoked`,
  `InviteCreated`, `InviteConsumed`, `MemberJoined`, `MemberLeft`,
  `FederationInbound`, `FederationOutbound` event sealed within 1 s.
- Section 230 + similar safe-harbor stance in `compliance.md`: oyatie is a
  provider; tenants are publishers; moderation is good-faith.
- Per-tenant retention:
  - Posts (announcements): 7y default, configurable per space.
  - Q&A: indefinite (knowledge value).
  - KB articles: indefinite; revisions sealed; bodies older than 7y may
    compress per ADR-COMM-0003 §Consequences-Negative.
  - Moderation actions: 7y per SOX-equivalent.
  - Audit-chain emissions: per-jurisdiction minimum (HIPAA 6+y; KR-PIPA
    3+y; SOX 7y).
- Per-jurisdiction overlay packs (pack-kr, pack-us-healthcare, pack-eu)
  apply transparently.
- E-discovery: legal-hold tag freezes a post / KB article / space /
  member's content from retention sunset; per FRCP 37(e).

### 12.7 Data residency

- Per-tenant `jurisdiction_code` from tenancy substrate determines region
  pinning.
- Postgres + Meilisearch + Valkey + S3 per-region; cross-region
  replication opt-in.
- ActivityPub federation respects per-pack outbound rules (pack-kr does
  not federate beyond KR-resident instances by default).

### 12.8 Sustainability (per ADR-0174)

- Per-action sustainability tag emitted with audit event (compute-grams +
  bytes-transferred).
- Per-tenant per-month sustainability rollup in finops-portal.
- Power-aware scheduling: brown-out signal per ADR-0176 downgrades
  non-critical features (e.g., AI-summarisation, federation outbound)
  during cell brown-outs.

### 12.9 DR posture (per ADR-0343)

- Manifest target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_active_active=true`, `replication_shape=active-active-multi-az-cross-region-warm`. The older §12.2 15-minute/30-second objective remains a stretch SLO, not the ADR-0343 manifest contract.
- Applicable pack floors from `specs/compliance-pack-floors.json`: HIPAA-2024 `3600s/300s` with multi-region required; KR-PIPA-2023 default `14400s/900s`; SOC2-T2 `14400s/900s`; ISO27001-2022 `14400s/3600s`; KR-CSAP-v3.1 `3600s/900s` with multi-region required. The effective maximum pack floor is ISO27001 `14400s/3600s`; community keeps the stricter product target to protect moderation and knowledge-base continuity.
- `failover_runbook=runbooks/dr-failover.md`, resolved at `microservices/community/runbooks/dr-failover.md`; backup substrates are `postgres_wal_g`, `object_storage_versioned`, `valkey`, and `audit_chain_merkle_seal`.
- `multi_region_active_active=true` for post-store writes, moderation action sealing, and KB publication in activated regulated packs; federation outbound remains throttleable during regional recovery.
- Why: tenants use community for support forums, Q&A, whistleblower-style escalation, and policy knowledge; failover must preserve moderation state and authored content rather than merely keeping reads alive.

### 12.10 Capacity model (per ADR-0340)

- Per-tenant baseline: `0.12 vCPU`, `256 MiB RAM`, `12 GiB storage`, `connections_per_tenant={valkey:3, postgres:3, outbound_http:4}`.
- Scaling dimension: `per_request` for posts, votes, moderation, KB publication, and search traffic.
- Cell placement class: `Tier-3` with manifest `pod_runtime_tier=2`; community is tenant-facing and compliance-sensitive, but traffic scales mostly with requests and storage grows with posts plus KB attachments.
- Autoscaling boundaries: min `2` api/worker replicas per tenant-cell, max `48` before Citus/search shard split; moderation queue workers have independent burst ceilings to prevent abuse floods from starving KB reads.
- Why: community traffic mixes read-heavy forums with sudden moderation spikes, so capacity must reserve enough write headroom for safety actions while allowing search and federation to back off.

### 12.11 Sustainability + cost attribution (per ADR-0344)

- Every post, vote, moderation action, KB publish, search query, attachment operation, and federation event audit row emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with tenant, product, capability, provider, cell, and compliance-pack dimensions.
- Provider routing is carbon-aware for search rebuilds, KB attachment processing, digest generation, and non-urgent federation backfill; it is not carbon-routed for moderation decisions, abuse reports, healthcare/community safety escalations, or regulated appeal actions.
- Tenant cost transparency surface: community admin shows post/write volume, search index size, moderation/classifier spend, attachment storage, and federation egress; finops-portal supplies monthly tenant and compliance-pack rollups.
- Why: community has public-support and regulated-discussion footprints, so CSRD, SB-253, and SEC climate-disclosure exports need per-capability cost/emission attribution for moderation, knowledge storage, and federation separately.

### 12.12 API versioning posture (per ADR-0342)

- Public API version model: `YYYY-MM-DD` carrier triplet using `Oyatie-Version` header, `/v/YYYY-MM-DD/` REST/ActivityPub gateway prefix, and proto3 field `string oyatie_version = 8001` for public events/contracts.
- SDK semver model: community SDKs publish `major.minor.patch`; client compatibility is pinned by date carrier for REST and event consumers.
- Support window: last `N=3` public versions for at least `180` days after deprecation.
- Per-tenant pinning: yes for tenant communities, help centers, moderation integrations, and regulated publisher workflows.
- Internal-mesh exemption: yes; ADR-0145 direct gRPC over HTTP/3 remains tag-compatible and exempt from public carrier routing.

### 12.13 Accessibility

- WCAG 2.2 AA minimum.
- Keyboard navigation for all actions.
- Screen-reader-compatible markup (semantic HTML + ARIA).
- High-contrast theme.
- Reduced-motion preference honoured.
- Per-locale i18n (EN, KO, JA, ZH, DE, FR, ES, PT, AR base; expandable).

## 13. Bounded Contexts (BCs)

Per ADR-0105 (13-layer canonical enum) and ADR-0106. Each BC follows the
per-microservice flat layout per ADR-0131.

| BC | Crate family | Purpose | Key entities |
|---|---|---|---|
| `spaces` | `oya-community-spaces-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,sdk,app}` | Server / subreddit / help-center top-level container | `Space`, `SpaceMembership`, `SpaceConfig`, `SpaceTemplate` |
| `channels` | `oya-community-channels-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,sdk}` | Channel within space (text/voice/forum/Q&A/announcement/stage) | `Channel`, `ChannelKind`, `ChannelMembership` |
| `post-store` | `oya-community-post-store-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk,app}` | Author + edit + delete posts; mention resolution; revision history | `Post`, `Author`, `Mention`, `Revision`, `SpaceRef` |
| `thread-tree` | `oya-community-thread-tree-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Materialised-path thread structure; lazy-load children | `Thread`, `Node`, `Path`, `Depth` |
| `comments` | `oya-community-comments-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Replies on posts; nested ≥ 8 levels | `Comment`, `Parent`, `Mention` |
| `reactions` | `oya-community-reactions-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Emoji reactions (custom + standard) | `Reaction`, `EmojiId`, `CustomEmoji` |
| `voting-engine` | `oya-community-voting-engine-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | Up/downvote; accept-answer; Wilson-ranking; brigade detection | `Vote`, `Tally`, `Acceptance`, `WilsonScore`, `BrigadeSignal` |
| `kb-articles` | `oya-community-kb-articles-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,rest,sdk,app}` | Wikipedia-style immutable revision history; editorial workflow | `Article`, `Revision`, `PublicationState`, `Attachment` |
| `moderation-queue` | `oya-community-moderation-queue-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-moderation-bridge,worker,sdk}` | Chain-of-responsibility moderation per ADR-COMM-0001 | `Envelope`, `Hop`, `Verdict`, `Appeal`, `ModerationAction` |
| `automod` | `oya-community-automod-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | Per-space rule engine; tenant-authored rules | `AutomodRule`, `RuleMatch`, `RuleAction` |
| `search-index` | `oya-community-search-index-{kernel,domain,usecase,api,adapter,adapter-search-meilisearch,adapter-search-tantivy,worker,sdk}` | Meilisearch primary + Tantivy fallback per ADR-COMM-0004 | `Document`, `Index`, `Tag`, `RankSignal`, `MultiSearchQuery` |
| `federation` | `oya-community-federation-{kernel,domain,usecase,api,adapter,adapter-activitypub,adapter-webfinger,worker,sdk}` | ActivityPub Server-to-Server + WebFinger + NodeInfo | `Actor`, `Activity`, `Inbox`, `Outbox`, `FederationPeer` |
| `notifications` | `oya-community-notifications-{kernel,domain,usecase,api,adapter,adapter-comms-email,adapter-push,worker,sdk}` | In-app + email + push + RSS notifications | `Notification`, `Subscription`, `Channel`, `Digest` |
| `retention` | `oya-community-retention-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | Per-tenant retention policy; legal hold; tombstoning | `RetentionPolicy`, `LegalHold`, `Tombstone` |
| `e-discovery` | `oya-community-e-discovery-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Auditor / regulator query interface | `EvidenceQuery`, `EvidenceBundle`, `LegalHoldFreeze` |
| `professional-profile` | `oya-community-professional-profile-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,sdk,app}` | Resume/profile CRUD; experience, education, skills, certifications, headline, summary, verification badge, vCard 4.0 + JSON Resume export | `Profile`, `ExperienceEntry`, `EducationEntry`, `SkillEntry`, `Certification`, `VerificationBadge` |
| `professional-graph` | `oya-community-professional-graph-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | Connection edges, mutual connections, degree-of-separation compute, block/restrict/disconnect lists | `ConnectionEdge`, `BlockEdge`, `RestrictEdge`, `DegreeOfSeparation` |
| `connection-request` | `oya-community-connection-request-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | Connection-request lifecycle with notes, anti-spam rate limits, and audit-chain emission | `ConnectionRequest`, `ConnectionRequestNote`, `ConnectionRequestStatus`, `RequestRateLimit` |
| `inmail-bridge` | `oya-community-inmail-bridge-{kernel,domain,usecase,api,adapter,adapter-messenger-bridge,worker,sdk}` | InMail-equivalent professional outreach through messenger; never opens a separate network DM surface | `InMail`, `InMailThread`, `InMailRateBudget`, `InMailDeliveryReceipt` |
| `endorsement-engine` | `oya-community-endorsement-engine-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | Skill endorsements, recommendations, Ed25519 endorsement-chain signing, revocation | `Endorsement`, `Recommendation`, `EndorsementSignature`, `RevocationRecord` |
| `skill-assessments` | `oya-community-skill-assessments-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | Skill quiz item bank, scoring, passing badge issuance, anti-cheat | `SkillAssessment`, `QuizItem`, `Attempt`, `Score`, `PassingBadge` |
| `jobs-recruiter` | `oya-community-jobs-recruiter-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-ats-bridge,worker,sdk}` | Job board, application/referral surface, employer pages, recruiter-stub default-off search, ATS handoff events | `JobPosting`, `JobApplicationReferral`, `EmployerPage`, `RecruiterSearchRequest`, `ATSHandoffEvent` |
| `pages-events` | `oya-community-pages-events-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-calendar-bridge,rest,sdk,app}` | Company / brand / employer pages, community events, recruiting events, calendar bridge | `Page`, `PageAdmin`, `PageAnalyticsSnapshot`, `Event`, `RSVP` |

### 13.1 BC naming justification

```
NAME: oya-community-<bc>-<layer>
JUSTIFICATION:
  L1 prefix: oya (workspace per BNF v4.1)
  L2 domain: community (this microservice per ADR-0131)
  L3 BC: <bc> (one of the BCs above)
  L4 layer: kernel|domain|usecase|api|adapter|adapter-postgres|adapter-search-*|
            adapter-s3|adapter-moderation-bridge|adapter-activitypub|
            adapter-webfinger|adapter-comms-email|adapter-push|rest|worker|sdk|app
  Conforms: BNF v4.1, 13-layer enum ADR-0105
```

## 14. Cross-µservice Consumption

| Consumed substrate | Purpose |
|---|---|
| `tenancy` | JWT issuance; per-tenant scope claims; sub-scope hierarchy |
| `identity` | OIDC; passkey / WebAuthn; service-principal |
| `policy-engine` | Cedar evaluation at every action; per-hop in moderation pipeline |
| `audit-chain` | Append-only sealing |
| `ontology` | Cross-product entity links; mention resolution |
| `intelligence` | Summarisation; related-Q; search ranking augmentation; AI moderation classifier |
| `workflow-engine` | Durable workflows (KB review, onboarding triggers, ticket spawning) |
| `comms-email` | Email-to-post; post-to-email digest; transactional notifications |
| `consent-graph` | DSAR cascade |
| `observability` | SLO authoring; burn-rate gating; promotion eligibility |
| `cell` | Per-tenant + per-region cell provisioning |
| `cloud-secrets` | KMS / OpenBao for tenant DEKs |
| `governance` | Fitness gates; oya-check-* lanes |
| `compliance` | Per-pack overlay enforcement |
| `api-gateway` | Request routing; per-tenant rate limit |
| `cloud-network` | Service mesh, NetworkPolicy, DNS, and edge-connectivity authoring; not the retired professional `network` product |
| `cloud-k8s` | Compute scheduling |
| `cloud-iac` | Helm + Terraform registry |

| Consumed sibling product | Purpose |
|---|---|
| `messenger` | Cross-product mentions; thread promotion to community Q&A; InMail-equivalent professional outreach delivery bridge |
| `mail` | Email-to-post intake |
| `drive` | Attachment cross-link |
| `calendar` | Event RSVP + reminder |
| `meet` | Voice / stage / AMA broadcast |
| `tasks` | Spawn task from post |
| `ats` | Optional downstream applicant-pipeline owner; community owns posting, referral, and contract-versioned handoff only |
| `forms` | Embed surveys |
| `analytics` | Engagement dashboard (read-only) |

## 15. Acceptance Criteria

### AC-01 — Spaces, channels, posts

The µservice ships with passing tests demonstrating:
- Tenant can create a space in < 10s.
- Space has channel types: text, voice (via meet), forum, Q&A, announcement, stage.
- Channels render in < 300ms (p99).
- Posts persist with revision history; edits emit `PostEdited` audit
  events.
- Deletes are soft + sealed.

### AC-02 — Threading

The µservice supports threaded replies ≥ 8 levels deep with lazy load. A
1000-node thread renders in < 350ms (p99).

### AC-03 — Voting + ranking

Wilson-confidence-interval ranking implemented per ADR-COMM-0002. Brigade
detection signal fires within 60s of a brigade onset. Vote cast p99 ≤
100ms. Accept-answer flips ranking + emits audit event.

### AC-04 — Q&A, AMA, polls, events

Q&A mode (one accepted answer per question), AMA mode (ask-mode window),
polls (single/multi/ranked-choice with closing time), events (RSVP +
calendar sync) all ship and emit audit events.

### AC-05 — KB articles (Wikipedia-style)

Per ADR-COMM-0003: immutable revisions; head-pointer model; Draft → Pending
Review → Published state machine; two-eyes for publish; conflict detection
on submit; rollback via revision append; attachments per revision via
S3.

### AC-06 — Moderation pipeline

Per ADR-COMM-0001: five-hop chain implemented with per-hop typed handler;
Cedar evaluation at every hop; per-hop audit-chain seal; appeals route to
higher-tier moderator; two-eyes for ban > 100 posts.

### AC-07 — Automod

Tenant can author per-space automod rules; rules fire on `PostCreated` /
`PostEdited`; rule firing emits `ClassifierVerdict` audit event.

### AC-08 — Search

Per ADR-COMM-0004: Meilisearch primary, Tantivy embedded fallback;
cross-BC ranked search; Cedar-policy filtered server-side; typo-tolerant;
search p99 ≤ 500ms.

### AC-09 — Mentions

Cross-product mentions (`@user`, `#channel`, `&role`, drive object refs,
ontology object refs) resolve in < 100ms; never cross tenant boundary.

### AC-10 — Notifications

Email + push + RSS + in-app notifications work; per-user subscriptions;
per-tenant digest cadence; email-to-post + post-to-email mirror modes.

### AC-11 — Federation (B2C personal opt-in)

ActivityPub Server-to-Server protocol implemented; WebFinger discoverable;
NodeInfo 2.1 endpoint live; Mastodon + Lemmy interop verified by
integration tests.

### AC-12 — Per-tenant Cedar roles

Tenant-defined custom roles with per-role Cedar fragments; per-space
overrides; SSO/OIDC integration via Zitadel.

### AC-13 — Audit + compliance

Every state-changing event sealed within 1s per ADR-0028. Per-tenant
retention policy + per-jurisdiction overlay packs (KR, US-HC, EU)
apply transparently.

### AC-14 — DSAR

GDPR Art. 17 / KR PIPA Art. 36 erasure cascade works per ADR-0242 §D-4.
Author tombstoning preserves revision body as community-contributed
content; audit-chain entries become hashed-subject.

### AC-15 — Performance budgets

All performance numbers in §12.1 met under 10x baseline load per quarterly
capacity drill.

### AC-16 — Brigade detection

Per ADR-COMM-0002 + foundry-guardrails signal; mass-coordinated voting
detected within 60s; vote inflation contained via per-member rate limits +
weighting.

### AC-17 — Multi-mode UX

B2C-personal + B2B-work + B2B-developer + EDU + NONPROFIT modes all ship
with their template defaults; mode switch is a tenancy `audience_type`
field; never a µservice fork.

### AC-18 — Import / export

Day-one importers: Discord export, Slack export, Reddit export, Discourse
backup, Notion export, GitHub Discussions export, Zendesk Help Center
export. Day-one exporter: full tenant data + media.

### AC-19 — Accessibility

WCAG 2.2 AA verified via automated lighthouse + manual screen-reader
audit. Keyboard-only navigation for all actions. Per-locale i18n.

### AC-20 — Observability

Per-space + per-tenant dashboards: feed render, search latency, vote QPS,
moderation-queue depth, audit-seal latency, federation-queue depth, KB
publish-time-to-published, notification fan-out time.

### AC-21 — DR drill (T2)

Quarterly DR drill: per-region cell loss; community state restored from
Postgres WAL + Valkey AOF; RTO < 1h verified; RPO < 30s verified.

### AC-22 — Cell isolation

Per-tenant + per-region cell isolation: a tenant-A cell failure does not
degrade tenant-B. Verified by chaos test.

### AC-23 — Reserved-namespace + cross-tenant

A community space cannot be registered with `oyatie` / `oya` / `oyat`
prefixes per ADR-0242 §D-6. Cross-tenant mention / read forbidden per
Cedar tenant-scope fragment.

### AC-24 — Federation isolation

For B2B-work tenants, federation defaults OFF; cannot be enabled without
ops-compliance grant. Inbound federation activities pass ClassifierHop
before queue admission.

### AC-25 — Sustainability + brown-out

Per ADR-0174 sustainability tag emitted; per ADR-0176 brown-out signal
downgrades AI-summarisation / federation-outbound during cell
brown-outs.

## 16. Deferrals

- **Live-stream / video-post hosting** at high bitrate: post-M02; sibling
  `shorts` handles short-form; M03 multimedia upgrade.
- **AI-generated answer synthesis** at the auto-post level: relies on
  intelligence substrate hardening; opt-in M02, auto-suggest M03.
- **Matrix federation** in addition to ActivityPub: M03 evaluate.
- **Cross-tenant federated communities** within oyatie's own customer
  base: M04 (requires Cedar cross-tenant grant model maturity).
- **Marketplace of community plugins** (custom emoji packs, integration
  bots, theme packs): M04 once marketplace-catalog substrate hardens.
- **End-to-end-encrypted threads** in community spaces: M04 (requires
  encryption-substrate reserved µservice promotion per ADR-0245
  §D-3.D).
- **Voice transcription for AMA + stage channels** beyond meet-substrate
  baseline: M03 intelligence pipeline upgrade.

## 17. Out-of-Scope

- **Real-time DMs and ephemeral chat** → `messenger` µservice.
- **LinkedIn-style engagement feed / status broadcasting** → forbidden by the
  2026-05-21 directive; community owns professional profiles, jobs, recruiter,
  InMail, endorsements, and recommendations, not thought-leadership feed
  farming.
- **For-You-style algorithmic attention feed, sponsored post promotion, and
  influencer monetization through followers** → forbidden anti-pattern; ranking
  is community-native vote / moderation / relevance logic.
- **Visual-first social posting and short-form video** → `social` + `shorts`
  µservices; community may embed/link supporting media but does not own the
  Instagram/TikTok product shape.
- **Identity directory + SSO** → `identity` / `tenancy` substrates.
- **Entity-type modelling** → `ontology` substrate.
- **Long-form video** (> 256 MB) → `shorts` and meet substrate.
- **Payments + paid memberships** → reserved `payments` µservice
  per ADR-0245 §D-3.D.
- **KYC / identity verification** → reserved `identity-verification`
  µservice.
- **Tax** on paid-community memberships → reserved `tax-engine`.

## 18. Risks + Mitigations

| Risk | Likelihood | Severity | Mitigation |
|---|---|---|---|
| Brigade overload on public-read space | M | H | foundry-guardrails ClassifierHop + per-member rate limits + Wilson ranking |
| KB rot (stale articles) | H | M | Freshness signal + review-required-after-N-months Cedar fragment |
| Federation abuse (spam from peer instances) | M | H | Curated default blocklist + per-tenant blocklist + same ClassifierHop applies inbound |
| Cross-tenant mention leak | L | C | Cedar tenant-scope fragment; per-tenant Postgres RLS; CI lane forbids cross-tenant queries |
| Moderation appeal SLA breach | M | M | Per ADR-COMM-0001 AppealHop SLA; queue depth alerted at 80% capacity; staffing playbook |
| Two-eyes for ban unavailable (second moderator offline) | M | M | Per ADR-COMM-0001 30-min window + fallback "queued pending second eye" |
| Search backend (Meilisearch) outage | L | M | Tantivy embedded fallback per ADR-COMM-0004 |
| Audit-chain seal latency breach | L | C | Per ADR-COMM-0001 §3 ≤ 1s; P0 alert; chain depth dashboards |
| ePHI accidentally in community post | M | C | Pack-us-healthcare overlay flags + HIPAA-trained reviewer + redaction at queue admission |
| ActivityPub key compromise | L | H | Per-actor signing key rotation; revocation propagated via tombstone activity |
| GDPR Art. 17 request volume spike | M | M | Per ADR-0242 §D-4 DSAR cascade engine + workflow-engine durable execution |

## 19. References

### 19.1 Internal ADRs

- ADR-COMM-0001 — Moderation policy pipeline (chain-of-responsibility +
  Cedar at every hop + audit-seal per hop).
- ADR-COMM-0002 — Voting engine ranking (Wilson confidence interval +
  brigade detection).
- ADR-COMM-0003 — KB article versioning (Wikipedia-style immutable
  revisions + tenant-scoped editorial review; no fork/branch/merge).
- ADR-COMM-0004 — Content search backend (Meilisearch 0.10.0 LTS primary +
  Tantivy 0.22.x embedded fallback; aligned with ADR-MSGR-0003).
- ADR-0028 — Audit-chain Merkle / Ed25519 sealing.
- ADR-0056 — BNF v4.1 naming.
- ADR-0105 — Thirteen-layer canonical enum.
- ADR-0106 — Layer rename (`application` → `usecase`).
- ADR-0131 — Per-microservice flat layout.
- ADR-0132 — Product-suite-and-bundle dissolution.
- ADR-0135 — Connect-unbundle (community + messenger sibling split).
- ADR-0139 — Agentic-SLO-gated promotion.
- ADR-0145 — Inter-microservice communication reform.
- ADR-0148 — Service-mesh Cilium.
- ADR-0150 — Cedar policy engine.
- ADR-0174 — Sustainability tag.
- ADR-0138 — Six-path deprecation pattern adapted for Wave 15K
  network-to-community retirement.
- `microservices/network/RETIRED.md` — successor marker for the retired
  professional-network corpus; this PRD now owns the migrated profile, graph,
  InMail, endorsement, recommendation, skill-assessment, jobs, recruiter,
  pages, and events scope.
- ADR-0176 — Brown-out / degradation signal.
- ADR-0183 — Policy engine separation (Cedar app authz + Kyverno
  admission).
- ADR-0211 — In-house tech-stack preference.
- ADR-0218 — Tenant granular control surface.
- ADR-0220 — Consumer Intelligence Substrate (amended by ADR-0242).
- ADR-0240 — Sovereign cloud per regional pack.
- ADR-0241 — DR + business-continuity portfolio policy.
- ADR-0242 — `oyatie`-is-a-tenant doctrine (audience moves to tenant).
- ADR-0243 — Cedar as universal gate.
- ADR-0244 — Tenant as universal scoping primitive.
- ADR-0245 — Substrate vs Product layering (`community` is product-tier
  `product-consumer-community`).
- ADR-0246 — Policy-engine substrate promotion.

### 19.2 Industry sources (2024-2026)

#### Discord

- Discord Engineering Blog, "How Discord Stores Trillions of Messages"
  (2023, updated 2024 / 2025 posts):
  `https://discord.com/blog/how-discord-stores-trillions-of-messages`
- Discord Trust & Safety + Safety Center 2024-2025:
  `https://discord.com/safety`
- Discord Developer Documentation (gateway + REST + interactions; 2024-2026):
  `https://discord.com/developers/docs/intro`
- Discord AutoMod 2.0 (2024 feature update):
  `https://discord.com/blog/automod-2-update`

#### Reddit

- Reddit Engineering Blog (community ranking + brigade defence; 2023-2024):
  `https://www.reddit.com/r/RedditEng/`
- Reddit Automoderator documentation:
  `https://www.reddit.com/wiki/automoderator`
- Reddit transparency report 2024:
  `https://www.redditinc.com/policies/transparency-report-january-to-june-2024`

#### Discourse

- Discourse Documentation (open-source forum platform; updated 2024-2026):
  `https://docs.discourse.org/`
- Discourse Reviewable Queue source:
  `https://github.com/discourse/discourse/blob/main/app/models/reviewable.rb`
- Discourse trust levels:
  `https://blog.discourse.org/2018/06/understanding-discourse-trust-levels/`

#### Stack Overflow

- Stack Overflow Meta (flag queue + diamond-mod + accept-answer):
  `https://meta.stackexchange.com/q/161541`
- Stack Overflow Teams product documentation (2024):
  `https://stackoverflow.com/teams`
- Stack Overflow developer survey 2024 (relevance ranking discussion):
  `https://survey.stackoverflow.co/2024/`

#### Notion

- Notion Engineering Blog, "Data Model Behind Notion":
  `https://www.notion.com/blog/data-model-behind-notion`
- Notion API reference (2024-2025):
  `https://developers.notion.com/`
- Notion Teamspaces feature documentation:
  `https://www.notion.so/help/teamspaces`

#### GitHub Discussions

- GitHub Discussions documentation (2024):
  `https://docs.github.com/en/discussions`
- GitHub Discussions API:
  `https://docs.github.com/en/graphql/reference/objects#discussion`

#### Zendesk Help Center

- Zendesk Guide / Help Center documentation (2024-2025):
  `https://support.zendesk.com/hc/en-us/categories/4405298722074-Guide`
- Zendesk content development guide:
  `https://support.zendesk.com/hc/en-us/articles/4408823923354`

#### Federation (ActivityPub + Matrix)

- W3C ActivityPub Recommendation (2018-01-23; living spec updates 2024):
  `https://www.w3.org/TR/activitypub/`
- W3C WebFinger RFC 7033:
  `https://www.rfc-editor.org/rfc/rfc7033`
- NodeInfo 2.1 spec:
  `https://nodeinfo.diaspora.software/protocol.html`
- Mastodon documentation (federation guide; 2024-2026):
  `https://docs.joinmastodon.org/spec/activitypub/`
- Lemmy federation guide:
  `https://join-lemmy.org/docs/contributors/05-federation.html`
- Matrix.org specification (2024 stable):
  `https://spec.matrix.org/`

#### Search

- Meilisearch documentation (0.10.0 LTS; pinned per ADR-COMM-0004):
  `https://www.meilisearch.com/docs`
- Tantivy project (0.22.x):
  `https://github.com/quickwit-oss/tantivy`
- Quickwit project (Meilisearch-adjacent at large scale):
  `https://quickwit.io/`
- BM25 ranking, Robertson & Zaragoza:
  `https://doi.org/10.1561/1500000019`
- Wilson confidence-interval ranking — Evan Miller "How Not To Sort By
  Average Rating": `https://www.evanmiller.org/how-not-to-sort-by-average-rating.html`

#### Moderation + safety

- Reddit Automoderator documentation: see above.
- Discord AutoMod 2.0: see above.
- Stack Overflow flag queue: see above.
- Discourse Reviewable queue: see above.
- Lemmy report dispatcher source:
  `https://github.com/LemmyNet/lemmy`
- Mastodon report state machine:
  `https://docs.joinmastodon.org/admin/moderation/`
- Wikipedia revision-deletion + history:
  `https://en.wikipedia.org/wiki/Wikipedia:Revision_deletion`
- Cloudflare Trust & Safety 2024 retrospective:
  `https://blog.cloudflare.com/cloudflare-trust-and-safety/`

#### Regulatory

- EU Digital Services Act, Regulation (EU) 2022/2065 (esp. Art. 14):
  `https://eur-lex.europa.eu/eli/reg/2022/2065`
- GDPR Articles 12 + 17 + 20 + 33-34:
  `https://eur-lex.europa.eu/eli/reg/2016/679`
- KR PIPA (개인정보 보호법) Art. 21 + 22 + 28 + 36 + 39:
  `https://www.law.go.kr/법령/개인정보보호법`
- HIPAA 45 CFR §164.312:
  `https://www.ecfr.gov/current/title-45/subtitle-A/subchapter-C/part-164`
- FRCP 37(e) (legal hold + ESI):
  `https://www.federalrulesofcivilprocedure.org/frcp/title-v-disclosures-and-discovery/rule-37-failure-to-make-disclosures-or-to-cooperate-in-discovery-sanctions/`
- Sedona Conference Working Group 1 — "The Sedona Principles, Third
  Edition": `https://thesedonaconference.org/publication/The_Sedona_Principles`
- Section 230 (US Communications Decency Act):
  `https://www.law.cornell.edu/uscode/text/47/230`
- ISO/IEC 27001:2022:
  `https://www.iso.org/standard/27001`
- SOC 2 Type II Trust Service Criteria (AICPA TSC 2017):
  `https://www.aicpa-cima.com/topic/audit-assurance/audit-and-assurance-greater-than-soc-2`
- WCAG 2.2 (W3C 2023-10):
  `https://www.w3.org/TR/WCAG22/`

#### Architectural

- Gang of Four — Chain of Responsibility pattern.
- Pinheiro et al. 2007 — "Failure trends in a large disk drive
  population": `https://research.google/pubs/pub32774/`
- Verma et al. 2016 — "Borg, Omega, and Kubernetes" (CACM):
  `https://research.google/pubs/borg-omega-and-kubernetes/`
- Pat Helland 2007 — "Life Beyond Distributed Transactions":
  `https://queue.acm.org/detail.cfm?id=3025012`
- Werner Vogels 2016 — "10 Lessons from 10 Years of AWS":
  `https://www.allthingsdistributed.com/2016/03/10-lessons-from-10-years-of-aws.html`
- Brewer (PODC 2000) — CAP theorem context.
- AWS Well-Architected Framework (2024):
  `https://docs.aws.amazon.com/wellarchitected/latest/framework/welcome.html`
- Google SRE Workbook (chapters 2 + 5 on SLO composition):
  `https://sre.google/workbook/table-of-contents/`

### 19.3 Specs + memory

- `/specs/per-microservice-flat-layout.json`
- `/specs/microservice-tier-classification.json`
- `/specs/microservice-dependency-dag.json`
- `/specs/tenant-model.json`
- `feedback_oyatie_is_a_tenant_doctrine`
- `feedback_quality_performance_scalability_bar`
- `feedback_clean_architecture_requirements`
- `feedback_no_silent_regression`
- `feedback_canonical_base_localization`
- `feedback_doc_coverage_enforced`
- `feedback_autonomous_implementation_artifacts`
- `feedback_workflow_objectgraph_adapter_layer`
- `feedback_bominal_inheritance_precedence`

---

*End of PRD-community.*

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `community` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `community` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 2 module pin(s) across 1 context(s).
- Scaling input: `per_request` with cell placement `Tier-3` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
