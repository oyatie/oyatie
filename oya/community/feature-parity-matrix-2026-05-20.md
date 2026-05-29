# community feature-parity matrix — Discourse / Circle / Vanilla Forums — 2026-05-20

Audit owner: solo Codex audit lane.
Target µservice: `community`.
Target path: `/Users/jasonlee/oyatie/microservices/community/`.
Counterpart 1: Discourse.
Counterpart 2: Circle.
Counterpart 3: Vanilla Forums.
Scope: current Wave 3 Batch 3.2 community audit.
Retired scope: capability-tier delta deliverable is intentionally absent.
Method: compare public counterpart surfaces, service-local PRD/contracts/ADRs, and service-local product purpose.
Primary local purpose citation: `microservices/community/PRD.md:84-116`.
Primary local feature citation: `microservices/community/PRD.md:187-230`.
Primary local contract citation: `microservices/community/contracts/openapi/community.yaml:19-220`.
Primary local moderation citation: `microservices/community/decisions/ADR-COMM-0001-moderation-policy-pipeline-architecture.md:56-91`.
Primary local search citation: `microservices/community/decisions/ADR-COMM-0004-content-search-backend.md:66-89`.
Primary local tenant-mode citation: `microservices/community/PRD.md:122-138`.
Primary local NFR citation: `microservices/community/PRD.md:864-900`.
Discourse official feature source: https://discourse.org/features.
Discourse official hosting source: https://www.discourse.org/meta.
Discourse official enterprise source: https://www.discourse.org/enterprise.
Discourse official install source: https://raw.githubusercontent.com/discourse/discourse/main/docs/INSTALL.md.
Circle official platform source: https://circle.so/platform.
Circle official developer limits source: https://api.circle.so/apis/admin-api/usage-and-limits.
Vanilla official basics source: https://success.vanillaforums.com/kb/articles/195-vanilla-basics.
Vanilla official moderation source: https://success.vanillaforums.com/kb/articles/342-moderation-process-tools.
Vanilla official gamification source: https://success.vanillaforums.com/kb/articles/341-gamification.
Vanilla official analytics source: https://success.vanillaforums.com/kb/articles/1505-out-of-the-box-dashboards.
Vanilla official API limits source: https://success.vanillaforums.com/kb/articles/44-rate-limits.
Matrix notation `P`: parity present in current artifacts.
Matrix notation `A`: additive Oyatie capability exceeds counterpart scope.
Matrix notation `G`: gap or under-specified in current artifacts.
Matrix notation `H`: handoff to another Oyatie µservice is correct, but the handoff needs explicit contract evidence.
Matrix notation `N`: intentionally not owned by `community`.

## §1 Counterpart 1 — Discourse capability surface

D-001 Discourse surface: flat forum topics and replies; official feature source describes simple flat forums and contextual replies.
D-002 Oyatie mapping: posts and replies are present in OpenAPI at `microservices/community/contracts/openapi/community.yaml:36-128`; status P.
D-003 Discourse surface: categories and discoverable discussion structure.
D-004 Oyatie mapping: spaces, channels, and tags are in PRD feature rows at `microservices/community/PRD.md:195-203`; status P.
D-005 Discourse surface: real-time chat channels that can be quoted into topics.
D-006 Oyatie mapping: PRD includes real-time servers/channels and meet integration at `microservices/community/PRD.md:88-89`; status H because chat/voice runtime is handoff-backed.
D-007 Discourse surface: trust system and community moderation.
D-008 Oyatie mapping: Cedar-per-hop moderation pipeline is specified in ADR-COMM-0001 at `microservices/community/decisions/ADR-COMM-0001-moderation-policy-pipeline-architecture.md:83-91`; status A.
D-009 Discourse surface: official plugins and integration ecosystem.
D-010 Oyatie mapping: PRD mentions webhooks and exports at `microservices/community/PRD.md:851-853`; status P for webhooks, G for plugin migration depth.
D-011 Discourse surface: API access and JSON endpoint behavior.
D-012 Oyatie mapping: OpenAPI 3.2.0 surface exists at `microservices/community/contracts/openapi/community.yaml:1-18`; status P.
D-013 Discourse surface: hosted service with 99.9% uptime SLA and high monthly page/post volume per official hosting page.
D-014 Oyatie mapping: read-path SLO 99.95%, write-path SLO 99.9%, federation 99.5% at `microservices/community/PRD.md:881-888`; status P.
D-015 Discourse surface: enterprise security, compliance, data residency, export, integrations.
D-016 Oyatie mapping: PRD covers tenant scoping, Cedar, audit, residency, and export at `microservices/community/PRD.md:902-920` and `microservices/community/PRD.md:851-852`; status P with OpenTofu context gaps.
D-017 Discourse surface: self-hosted install using Docker, Postgres, Valkey, Ruby, and minimum 1GB RAM.
D-018 Oyatie mapping: Oyatie backend language policy is Rust-strict and should not inherit Discourse's Ruby stack; status A by policy, but build implementation absent.
D-019 Discourse surface: mobile app and hosted push notification support.
D-020 Oyatie mapping: PRD requires mobile/web/desktop at `microservices/community/PRD.md:855`; status H because frontend app surfaces are not in this service path.
D-021 Discourse surface: AI translations.
D-022 Oyatie mapping: PRD includes intelligence substrate for summarization, related Q, search ranking, and AI moderation at `microservices/community/PRD.md:1027`; status H/G because translation is not explicit.
D-023 Discourse surface: data explorer and reporting through hosted/enterprise plans.
D-024 Oyatie mapping: dashboards exist for moderation queue, post throughput, and vote rate; status P for ops dashboards, G for tenant-facing analytics.
D-025 Discourse surface: email-in and digest patterns are common Discourse strengths.
D-026 Oyatie mapping: PRD names email-to-post and post-to-email digest at `microservices/community/PRD.md:181-183`; status P.
D-027 Discourse surface: Q&A and solved/voting plugins.
D-028 Oyatie mapping: accepted-answer and voting routes exist at `microservices/community/contracts/openapi/community.yaml:128-161`; status P.
D-029 Discourse surface: moderation flags and review queues.
D-030 Oyatie mapping: flag and moderation action routes exist at `microservices/community/contracts/openapi/community.yaml:162-199`; status P.
D-031 Discourse surface: knowledge-base-style discoverability through topics, categories, tags, and plugins.
D-032 Oyatie mapping: KB article APIs exist at `microservices/community/contracts/openapi/community.yaml:200-220`; status A.
D-033 Discourse surface: self-host/open-source extensibility.
D-034 Oyatie mapping: Oyatie has no user-facing plugin ecosystem documented inside community; status G unless plugin-app-store handoff is documented.
D-035 Discourse surface: migration from Discourse is a natural switching path.
D-036 Oyatie mapping: `microservices/community/migration-playbooks/from-discourse.md` exists; status P with retired tier-language cleanup required.
D-037 Discourse surface: spam defense and AI moderation plugins.
D-038 Oyatie mapping: Foundry-guardrails moderation bridge appears in PRD and ADR; status H/P because classifier is external but pipeline is specified.
D-039 Discourse surface: group management and roles.
D-040 Oyatie mapping: PRD per-space membership and roles at `microservices/community/PRD.md:197-200`; status P.
D-041 Discourse surface: webhooks and API automation.
D-042 Oyatie mapping: FR-40 outbound webhooks at `microservices/community/PRD.md:853`; status P.
D-043 Discourse surface: data export and no vendor lock-in in enterprise source.
D-044 Oyatie mapping: FR-39 export full tenant data at `microservices/community/PRD.md:851-852`; status P.
D-045 Discourse surface: discussion search.
D-046 Oyatie mapping: Meilisearch/Tantivy selection at `microservices/community/decisions/ADR-COMM-0004-content-search-backend.md:66-89`; status P.
D-047 Discourse surface: site-scale claim of more than 22,000 communities per official enterprise page.
D-048 Oyatie mapping: PRD targets 1M posts/tenant/month and 100K search qps/cell at `microservices/community/PRD.md:894-900`; status A target, not implementation evidence.
D-049 Discourse headline: Oyatie matches core forum, Q&A, moderation, email, search, and export.
D-050 Discourse headline gap: self-hosting and managed hosting are not deployable until six OpenTofu context modules exist.

## §2 Counterpart 2 — Circle capability surface

C-001 Circle surface: community, courses, events, content, revenue, email marketing, payments, website builder, AI agents, and CRM in one platform.
C-002 Oyatie mapping: community owns discussion and content, but courses, payments, email marketing, and website building are sibling-service handoffs; status H.
C-003 Circle surface: all-in-one business/community operation with memberships and subscriptions.
C-004 Oyatie mapping: PRD reserves payments and paid memberships to `payments` at `microservices/community/PRD.md:1223-1227`; status H, not gap if handoff is contracted.
C-005 Circle surface: discussions.
C-006 Oyatie mapping: posts, replies, threads, votes, and KB exist; status P.
C-007 Circle surface: events and live sessions.
C-008 Oyatie mapping: PRD includes polls, events, AMAs, announcements at `microservices/community/PRD.md:184-185`; live/voice via meet at `microservices/community/PRD.md:214-215`; status H/P.
C-009 Circle surface: courses.
C-010 Oyatie mapping: no course ownership in community PRD; status N/H and likely `learn` or equivalent service ownership.
C-011 Circle surface: paid memberships and branded checkout.
C-012 Oyatie mapping: PRD reserves payments; tenant_class model absent; status H/G.
C-013 Circle surface: free trials, installments, BNPL, subscriptions.
C-014 Oyatie mapping: absent in community and should remain billing/payments-owned; status H.
C-015 Circle surface: gamification and automated nudges.
C-016 Oyatie mapping: PRD supports reactions/voting and notifications but not Circle-style gamification programs; status G.
C-017 Circle surface: reporting and analytics.
C-018 Oyatie mapping: service has dashboards but tenant-facing analytics are under-specified; status G.
C-019 Circle surface: searchable member directory and rich member profiles.
C-020 Oyatie mapping: member identity and directory are identity/tenancy handoffs; community needs explicit profile-card read contract; status H/G.
C-021 Circle surface: custom domain and branding.
C-022 Oyatie mapping: FR-43 custom domain and branding at `microservices/community/PRD.md:856`; status P.
C-023 Circle surface: live rooms and live streams.
C-024 Oyatie mapping: live paths are meet/shorts handoffs; PRD includes stage/town-hall via meet at `microservices/community/PRD.md:214-215`; status H.
C-025 Circle surface: custom mobile apps under higher plans.
C-026 Oyatie mapping: FR-42 requires mobile/web/desktop at `microservices/community/PRD.md:855`; status H because frontend artifacts are absent.
C-027 Circle surface: headless/member API.
C-028 Oyatie mapping: OpenAPI exists; member API semantics not fully mapped; status P/G.
C-029 Circle surface: admin API rate limit of 2000 requests per 5 minutes per IP and monthly request allotments in official developer limits.
C-030 Oyatie mapping: PRD per-member post/vote/report rate limits exist at `microservices/community/PRD.md:913-916`; admin/API tenant limits need explicit contract; status G.
C-031 Circle surface: large platform statistics, including more than 15 million members and 20,000 communities on the official platform page.
C-032 Oyatie mapping: PRD scale targets are per-cell and per-tenant rather than global installed base; status target A, proof absent.
C-033 Circle surface: AI inbox and AI Copilot.
C-034 Oyatie mapping: `intelligence` substrate is listed at `microservices/community/PRD.md:1027`; status H.
C-035 Circle surface: website and landing pages.
C-036 Oyatie mapping: likely `sites` handoff; manifest depends on `sites` at `microservices/community/manifest.json:416`; status H/G because handoff contract unclear.
C-037 Circle surface: segmentation and automation.
C-038 Oyatie mapping: workflow-engine and comms-email substrates listed at `microservices/community/PRD.md:1028-1029`; status H.
C-039 Circle surface: member onboarding and migration services.
C-040 Oyatie mapping: onboarding doc exists, but it uses retired tier language; status P with P2 retirement cleanup.
C-041 Circle surface: flexible plans and business growth tooling.
C-042 Oyatie mapping: tenant_class replacement absent; status G.
C-043 Circle surface: gated content and access control.
C-044 Oyatie mapping: Cedar policy and per-space roles cover access; status A.
C-045 Circle surface: community CRM behavior.
C-046 Oyatie mapping: not in community PRD except member profiles and notifications; likely CRM/analytics handoff; status H/G.
C-047 Circle surface: data API/event stream for warehouse integrations.
C-048 Oyatie mapping: AsyncAPI events exist, but data warehouse/export integration is not explicit; status G.
C-049 Circle headline: Oyatie's technical forum/Q&A/moderation core is stronger than Circle's public developer limits.
C-050 Circle headline gap: revenue, course, CRM, website, automation, analytics, and tenant_class semantics require explicit sibling-handoff contracts.

## §3 Counterpart 3 — Vanilla Forums capability surface

V-001 Vanilla surface: discussions, questions, comments, ideas, roles, permissions, addons, reactions, gamification, moderation, and analytics.
V-002 Oyatie mapping: discussions/posts/replies/Q&A exist in PRD and OpenAPI; status P.
V-003 Vanilla surface: moderation tab and Community Management Dashboard for reported content and queues.
V-004 Oyatie mapping: moderation queue routes and ADR-COMM-0001 chain exist; status A.
V-005 Vanilla surface: moderation messages.
V-006 Oyatie mapping: announcements and moderation actions exist, but global moderation-message UX is not explicit; status G.
V-007 Vanilla surface: spam queue.
V-008 Oyatie mapping: spam-flood runbook and moderation queue exist; status P.
V-009 Vanilla surface: change log of edited/deleted posts.
V-010 Oyatie mapping: PRD requires edit history and soft-delete with audit trail at `microservices/community/PRD.md:207-208`; status A.
V-011 Vanilla surface: inline moderation actions such as close, announce, split, merge, and delete.
V-012 Oyatie mapping: moderation action route exists, but exact action enum coverage needs contract inspection beyond current excerpt; status P/G.
V-013 Vanilla surface: analytics dashboards and custom analytics.
V-014 Oyatie mapping: operational dashboards exist, but tenant-facing analytics need explicit UI/API; status G.
V-015 Vanilla surface: gamification points, ranks, badges, reactions, and Q&A points.
V-016 Oyatie mapping: reactions/voting exist, but points/ranks/badges are not first-class; status G.
V-017 Vanilla surface: addons/plugins.
V-018 Oyatie mapping: plugin ecosystem is not owned locally; plugin-app-store handoff needed; status H/G.
V-019 Vanilla surface: categories, discussions, questions, comments, ideas.
V-020 Oyatie mapping: spaces/channels/posts/replies/Q&A exist; ideas mode not explicit; status P/G.
V-021 Vanilla surface: roles and permissions in dashboard settings.
V-022 Oyatie mapping: Cedar policy fragments and per-space membership/roles exist; status A.
V-023 Vanilla surface: branding and theme controls.
V-024 Oyatie mapping: FR-43 branding and custom domain at `microservices/community/PRD.md:856`; status P.
V-025 Vanilla surface: search and navigation.
V-026 Oyatie mapping: Meilisearch/Tantivy search ADR; status P.
V-027 Vanilla surface: API rate limits 300 GET/min/IP and 120 write requests/min/IP, with hard block >250 requests/10s.
V-028 Oyatie mapping: member-level limits exist, but IP/admin/API limits need contract policy; status G.
V-029 Vanilla surface: reporting of inappropriate content by staff/community members.
V-030 Oyatie mapping: flag route and moderation queue exist; status P.
V-031 Vanilla surface: post restore/permanent delete through change log.
V-032 Oyatie mapping: runbook for post mass-deletion and audit trail exist; specific restore API absent; status P/G.
V-033 Vanilla surface: knowledge articles and documentation through success community pattern.
V-034 Oyatie mapping: KB article APIs exist; status P.
V-035 Vanilla surface: SSO and user management.
V-036 Oyatie mapping: identity/tenancy substrates own SSO; status H.
V-037 Vanilla surface: third-party tracking and analytics integrations.
V-038 Oyatie mapping: observability and analytics handoffs exist, but tenant-facing embeds not defined; status G.
V-039 Vanilla surface: rich admin dashboard search.
V-040 Oyatie mapping: admin console not locally specified; status G.
V-041 Vanilla surface: permissions that govern moderation abilities and administrator access.
V-042 Oyatie mapping: Cedar fragments enforce policy and auditor/CI scopes; status A.
V-043 Vanilla surface: Q&A accepted answers and reactions.
V-044 Oyatie mapping: accepted-answer route and vote/reaction PRD rows exist; status P.
V-045 Vanilla surface: idea submissions and feedback boards.
V-046 Oyatie mapping: polls/events/AMAs/announcements exist, but idea-board mode is not explicit; status G.
V-047 Vanilla surface: migration from classic forum/community platforms.
V-048 Oyatie mapping: Discourse migration exists; Vanilla migration missing; status G.
V-049 Vanilla headline: Oyatie matches core discussion, Q&A, moderation, audit, and KB.
V-050 Vanilla headline gap: admin dashboard, analytics, gamification, rate limits, and Vanilla migration need explicit service-local artifacts.

## §4 UNION-coverage matrix

U-001 Top-level spaces/communities: Discourse P, Circle P, Vanilla P, Oyatie P via PRD rows `microservices/community/PRD.md:195-201`.
U-002 Categories/channels/tags: Discourse P, Circle P, Vanilla P, Oyatie P via PRD rows `microservices/community/PRD.md:201-203`.
U-003 Threaded discussions: Discourse P, Circle P, Vanilla P, Oyatie P via OpenAPI replies `microservices/community/contracts/openapi/community.yaml:101-128`.
U-004 Deep nested comments: Discourse P, Circle partial, Vanilla P, Oyatie P via PRD row `microservices/community/PRD.md:206`.
U-005 Post create/read/edit/delete: all counterparts P, Oyatie P via OpenAPI `microservices/community/contracts/openapi/community.yaml:58-100`.
U-006 Revision history: Discourse P, Circle partial, Vanilla change-log P, Oyatie A via PRD `microservices/community/PRD.md:207-208`.
U-007 Soft-delete audit: Discourse P, Circle partial, Vanilla P, Oyatie A via audit-chain and PRD.
U-008 Markdown/rich text: all counterparts P, Oyatie P via PRD `microservices/community/PRD.md:209-210`.
U-009 File/image upload: all counterparts P, Oyatie P via PRD `microservices/community/PRD.md:211-213`.
U-010 Video upload/live: Circle P, Discourse partial, Vanilla partial, Oyatie H via meet/shorts `microservices/community/PRD.md:214-215`.
U-011 Q&A accepted answer: Discourse plugin P, Circle partial, Vanilla P, Oyatie P via OpenAPI `microservices/community/contracts/openapi/community.yaml:145-161`.
U-012 Voting: Discourse P, Circle partial, Vanilla reactions/votes P, Oyatie P via OpenAPI `microservices/community/contracts/openapi/community.yaml:128-144`.
U-013 Reactions: all counterparts P, Oyatie P via PRD `microservices/community/PRD.md:221-223`.
U-014 Ranking algorithms: Discourse P, Circle limited, Vanilla gamification P, Oyatie A via Wilson/hot ranking `microservices/community/PRD.md:222-228`.
U-015 Trust/reputation: Discourse P, Circle gamification P, Vanilla ranks P, Oyatie G because reputation mode exists in capability files but core PRD mapping is incomplete.
U-016 Badges/points/ranks: Circle P, Vanilla P, Discourse P, Oyatie G.
U-017 Gamification automations: Circle P, Vanilla P, Discourse P, Oyatie G.
U-018 Moderation queue: Discourse P, Circle P, Vanilla P, Oyatie P via OpenAPI moderation queue.
U-019 Automated moderation: Discourse AI/plugin P, Circle AI P, Vanilla tools P, Oyatie A via ADR-COMM-0001 + foundry guardrails.
U-020 Per-hop Cedar policy: counterparts not equivalent, Oyatie A via ADR-COMM-0001.
U-021 Per-hop audit seals: counterparts limited, Oyatie A via ADR-COMM-0001.
U-022 Appeals: counterparts partial, Oyatie P via ADR-COMM-0001 appeal hop.
U-023 Two-eyes destructive moderation: counterparts partial, Oyatie P via ADR-COMM-0001.
U-024 Spam queue: Discourse P, Circle P, Vanilla P, Oyatie P via runbooks and moderation pipeline.
U-025 Change log: Discourse P, Circle partial, Vanilla P, Oyatie A via audit-chain and revisions.
U-026 Cross-tenant leak response: counterparts enterprise-only, Oyatie G because runbook referenced but missing.
U-027 Search: all counterparts P, Oyatie P via ADR-COMM-0004.
U-028 Search backend residency: counterparts SaaS/enterprise partial, Oyatie P in ADR-COMM-0004 but missing OpenTofu context modules.
U-029 KB/articles: Discourse partial, Circle content P, Vanilla KB P, Oyatie P via OpenAPI KB routes.
U-030 Editorial workflow: Circle content P, Vanilla partial, Discourse plugins P, Oyatie P via PRD KB model.
U-031 Events: Circle P, Discourse calendar plugin P, Vanilla partial, Oyatie P/H via PRD + meet.
U-032 Live rooms/livestreams: Circle P, Discourse partial, Vanilla partial, Oyatie H via meet/shorts.
U-033 Courses: Circle P, Discourse plugin partial, Vanilla partial, Oyatie N/H.
U-034 Paid memberships: Circle P, Discourse subscriptions plugin P, Vanilla monetization via addons partial, Oyatie H via payments.
U-035 Checkout/installments/BNPL: Circle P, Discourse limited, Vanilla limited, Oyatie N/H via payments.
U-036 Tenant_class billing overlay: counterparts plan-based, Oyatie G because tenant_class absent.
U-037 Revenue-share model: counterparts marketplace-specific, Oyatie G because revenue_share absent.
U-038 Trial/demo class: Circle P via free trial, Discourse hosted trials P, Vanilla sales-led partial, Oyatie G because demo_trial absent.
U-039 Usage caps: Circle API limits P, Vanilla API limits P, Discourse hosted limits P, Oyatie partial via member rate limits only.
U-040 API rate limits: Circle P, Vanilla P, Discourse partial, Oyatie G for admin/API IP limits.
U-041 Per-member post rate limit: counterparts partial, Oyatie P via PRD `microservices/community/PRD.md:913-916`.
U-042 Per-member vote rate limit: counterparts partial, Oyatie P via PRD `microservices/community/PRD.md:913-916`.
U-043 Webhooks: Discourse P, Circle APIs P, Vanilla APIs P, Oyatie P via FR-40 `microservices/community/PRD.md:853`.
U-044 Data export: Discourse P, Circle P, Vanilla P, Oyatie P via FR-39 `microservices/community/PRD.md:851-852`.
U-045 Data import: Discourse migration P, Circle migration P, Vanilla migration P, Oyatie partial via Discourse-only playbook.
U-046 Discourse migration: Oyatie P.
U-047 Circle migration: Oyatie G.
U-048 Vanilla migration: Oyatie G.
U-049 OpenAPI: Discourse API P, Circle API P, Vanilla API P, Oyatie P.
U-050 Async events: counterparts partial, Oyatie P via AsyncAPI file in inventory.
U-051 Proto contract: counterparts generally API/SDK-based, Oyatie P via proto file.
U-052 SDK reference: counterparts APIs P, Oyatie P via Rust SDK reference.
U-053 Mobile app: Discourse P, Circle branded app P, Vanilla mobile web/app partial, Oyatie H because no frontend artifacts.
U-054 Web app: all counterparts P, Oyatie H because frontend web artifacts absent.
U-055 Desktop app: counterparts limited, Oyatie H via PRD but no artifact.
U-056 Custom domain: Discourse hosted P, Circle P, Vanilla P, Oyatie P via FR-43.
U-057 Branding: all counterparts P, Oyatie P via FR-43.
U-058 Website builder: Circle P, Discourse/Vanilla theme partial, Oyatie H/G via `sites`.
U-059 Member directory: Circle P, Vanilla users P, Discourse users P, Oyatie H/G via identity/tenancy.
U-060 Rich profiles: Circle P, Vanilla P, Discourse P, Oyatie G/P via capability files but incomplete PRD parity.
U-061 Workplace verification: Discourse no, Circle no, Vanilla no, Oyatie A via Teamblind-mode capability.
U-062 Anonymous posting: Discourse plugins partial, Circle no, Vanilla partial, Oyatie A via anonymity policy files.
U-063 SecureDrop/whistleblower intake: counterparts no, Oyatie A via capability and IP artifacts.
U-064 Bug bounty submissions: counterparts no, Oyatie A via capability artifact.
U-065 Handshake/jobs mode: counterparts no for target set, Oyatie A via capability and IP artifacts.
U-066 LinkedIn mode: counterparts no for target set, Oyatie A/G because capability exists but product scope changed in later chat.
U-067 Reddit mode: counterparts adjacent, Oyatie A via capability artifact.
U-068 Federation/ActivityPub: Discourse plugins partial, Circle no, Vanilla no, Oyatie P via PRD `microservices/community/PRD.md:98-99`.
U-069 Customer-facing help center: Circle content partial, Vanilla P, Discourse partial, Oyatie P via PRD surfaces.
U-070 Developer forum: Discourse P, Circle partial, Vanilla partial, Oyatie P via PRD audience modes.
U-071 Class server: counterparts partial, Oyatie P via PRD audience modes.
U-072 Accessibility WCAG: counterparts claim accessibility generally, Oyatie P via FR-44 `microservices/community/PRD.md:857`.
U-073 Compliance packs: Discourse enterprise P, Circle enterprise partial, Vanilla enterprise partial, Oyatie P/G because docs exist but tenant_class gap blocks demo_trial compliance denial semantics.
U-074 BYOK/encryption: Discourse enterprise partial, Circle enterprise partial, Vanilla enterprise partial, Oyatie P/G via cloud-secrets but no context modules.
U-075 Data residency: Discourse enterprise P, Circle enterprise partial, Vanilla enterprise partial, Oyatie P/G via PRD and ADR-COMM-0004 but missing IaC.
U-076 Multi-region: Discourse hosted P, Circle SaaS P, Vanilla SaaS P, Oyatie P/G via docs but missing context modules.
U-077 Self-hosting: Discourse P, Circle limited, Vanilla cloud/community edition history, Oyatie G until OpenTofu on-prem/colo modules exist.
U-078 OCI Always Free demo profile: counterparts not relevant, Oyatie G because `iac/oci-guest/always-free/` absent.
U-079 On-prem deployment: Discourse P, Circle no, Vanilla enterprise cloud mostly, Oyatie G until `iac/on-prem/` exists.
U-080 Colo deployment: counterparts limited, Oyatie G until `iac/colo/` exists.
U-081 Oyatie-as-cloud-provider: counterparts SaaS-owned, Oyatie G until `iac/oyatie-iaas/` exists.
U-082 OpenTofu modules: counterparts not equivalent, Oyatie G because Terraform artifact exists.
U-083 Supported OS manifest: counterparts product docs not equivalent, Oyatie G.
U-084 Rust backend policy: Discourse Ruby/Rails, Circle SaaS unknown, Vanilla PHP history, Oyatie A policy and clean scan.
U-085 Runtime source implementation: counterparts implemented, Oyatie G because no local `src/`.
U-086 Tests: counterparts production, Oyatie G because no local `tests/`.
U-087 Runbooks: counterparts SaaS internal, Oyatie P/G because runbooks exist but some referenced ones missing.
U-088 Dashboards: Vanilla analytics P, Circle reporting P, Discourse Data Explorer P, Oyatie P/G with ops dashboards but tenant analytics gap.
U-089 Cost model: counterparts SaaS pricing, Oyatie P/G because cost-budget exists but uses stale Elasticsearch and retired tier language.
U-090 Capacity model: counterparts opaque, Oyatie P/G because capacity model exists but uses stale Elasticsearch and sizing tier vocabulary.
U-091 Failure modes: counterparts opaque, Oyatie P/G because failure-modes exists but includes stale backend and missing runbooks.
U-092 Threat model: counterparts opaque, Oyatie P because threat model exists.
U-093 DPIA: counterparts enterprise docs partial, Oyatie P/G because dpia exists but references Elastic/OpenSearch drift.
U-094 Compliance doc: counterparts enterprise docs partial, Oyatie P.
U-095 Policy fragments: counterparts product-configurable, Oyatie A via Cedar files.
U-096 Public-read policy: Discourse/Vanilla public forums P, Circle private communities P, Oyatie P via policy file.
U-097 Auditor scope: enterprise counterparts partial, Oyatie A via Cedar file.
U-098 CI scope: counterparts internal, Oyatie P via Cedar file.
U-099 Anonymity modes: counterparts limited, Oyatie A via four anonymity policy files.
U-100 Pack-specific overlays: counterparts enterprise partial, Oyatie P/G with pack-kr kustomize only.
U-101 Tenant onboarding: counterparts P, Oyatie P/G because onboarding exists but retired tier language.
U-102 FAQ: counterparts P, Oyatie P/G because FAQ exists but retired tier language.
U-103 Tutorial: counterparts P, Oyatie P/G because tutorial exists but retired tier language.
U-104 Reference implementation: counterparts API samples P, Oyatie P/G because SDK doc has retired tier wording.
U-105 Benchmarks: counterparts public sparse, Oyatie P/G because local benchmark omits Circle/Vanilla and uses retired tiers.
U-106 Product purpose clarity: all counterparts clear, Oyatie P.
U-107 Boundary clarity: Discourse/Circle/Vanilla each own broad platform scopes, Oyatie G because product-to-product dependencies need handoff classification.
U-108 Schema version correctness: Oyatie OpenAPI P at 3.2.0.
U-109 AsyncAPI/proto coverage: Oyatie P inventory.
U-110 SLO coverage: Oyatie P with seven OpenSLO files.
U-111 Incident response: Oyatie P/G with missing runbook refs.
U-112 Cross-service handoffs: Oyatie G because handoff file absent and manifest dependency semantics unclear.
U-113 Tenant-class adoption: Oyatie G.
U-114 Tenant-class adoption compliance: Oyatie G because 56 exact references remain.
U-115 Overall forum parity: Oyatie P for Discourse core.
U-116 Overall creator-community parity: Oyatie partial for Circle; H/G dominates revenue/course/CRM surfaces.
U-117 Overall enterprise forum parity: Oyatie partial for Vanilla; G on admin analytics/gamification/migration.

## §5 Family summary

Family 1 discussion substrate: Oyatie covers spaces, posts, comments, replies, tags, votes, and accepted answers.
Family 1 evidence: `microservices/community/contracts/openapi/community.yaml:36-161`.
Family 1 verdict: parity or better against all three counterparts.
Family 2 moderation and trust: Oyatie's Cedar-per-hop and audit-seal model exceeds public counterpart surfaces.
Family 2 evidence: `microservices/community/decisions/ADR-COMM-0001-moderation-policy-pipeline-architecture.md:83-91`.
Family 2 verdict: additive, but missing P0 runbook files reduce operational closure.
Family 3 search and knowledge: Oyatie has KB APIs and a search-backend ADR.
Family 3 evidence: `microservices/community/contracts/openapi/community.yaml:200-220` and `microservices/community/decisions/ADR-COMM-0004-content-search-backend.md:66-89`.
Family 3 verdict: parity on design, stale operations docs need cleanup.
Family 4 business/community platform: Circle is broader than community alone.
Family 4 evidence: `microservices/community/PRD.md:1223-1227` reserves payments and paid memberships to another service.
Family 4 verdict: acceptable only if sibling handoff contracts are explicit.
Family 5 enterprise admin: Vanilla's moderation/admin/analytics/gamification surfaces are more mature in public docs.
Family 5 evidence: local admin analytics are ops dashboards, not tenant-facing analytics.
Family 5 verdict: material product gap.
Family 6 deployment portability: Oyatie's canonical ambition exceeds counterparts but service-local IaC is not aligned.
Family 6 evidence: missing six context directories and active Terraform file.
Family 6 verdict: P1 blocker for deployability claims.
Family 7 monetization and tenant classes: Circle has public business monetization; Oyatie replacement model is absent locally.
Family 7 evidence: tenant_class scan absent and PRD reserves payments.
Family 7 verdict: product handoff plus tenant_class gap.
Family 8 migration: Discourse migration exists, Circle/Vanilla do not.
Family 8 verdict: partial.
Family 9 frontend/app: PRD requires mobile/web/desktop but no local frontend evidence exists.
Family 9 verdict: handoff or missing artifact needs clarification.
Family 10 future scope: chat history later points to broader Reddit/Teamblind/Instagram or adjacent scopes.
Family 10 verdict: this audit remains transitional under current explicit prompt.

## §6 Headline gap analysis

Gap 1: Circle monetization and business tooling is not represented as first-class community-owned behavior.
Gap 1 severity: P2 handoff gap, not a reason to pull payments into community.
Gap 1 citation: `microservices/community/PRD.md:1223-1227`.
Gap 2: Vanilla tenant-facing analytics and gamification are under-specified.
Gap 2 severity: P2 parity gap.
Gap 2 citation: local dashboards are ops JSON files, while no tenant analytics contract appears in OpenAPI excerpt `microservices/community/contracts/openapi/community.yaml:1-230`.
Gap 3: Circle and Vanilla migration paths are missing.
Gap 3 severity: P2 switching-cost gap.
Gap 3 citation: inventory has `migration-playbooks/from-discourse.md` only.
Gap 4: API/IP-level rate limiting is weaker than public Circle and Vanilla docs.
Gap 4 severity: P2 contract gap.
Gap 4 citation: only per-member limits appear at `microservices/community/PRD.md:913-916`.
Gap 5: deployability claims are blocked by missing OpenTofu context modules.
Gap 5 severity: P1 canonical-direction gap.
Gap 5 citation: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2275-2294`.
Gap 6: retired tier language contaminates onboarding, FAQ, tutorial, migration, reference, and benchmark artifacts.
Gap 6 severity: P2 Wave 15J retirement gap.
Gap 6 citation: `coherence-audit-2026-05-20.md §3.4.T`.
Gap 7: tenant_class replacement semantics are absent.
Gap 7 severity: P2 canonical-direction gap.
Gap 7 citation: tenant-class memory lines 101-142 and local scan result.
Gap 8: search backend drift creates operator confusion.
Gap 8 severity: P2 documentation contradiction.
Gap 8 citation: ADR-COMM-0004 lines 66-89 versus capacity/cost/failure drift.
Gap 9: product-to-product dependency semantics are ambiguous.
Gap 9 severity: P1 if any dependency is synchronous at runtime.
Gap 9 citation: `microservices/community/ARCHITECTURE.md:197-205` and `microservices/community/manifest.json:400-420`.
Gap 10: service root README is missing.
Gap 10 severity: P2 discoverability gap.
Gap 10 citation: inventory omission.

## §7 Additive Oyatie surface

Additive 1: Cedar policy evaluation at every moderation hop.
Additive 1 evidence: `microservices/community/decisions/ADR-COMM-0001-moderation-policy-pipeline-architecture.md:83-85`.
Additive 2: per-hop audit-chain seal and P0 classification for missing seal.
Additive 2 evidence: `microservices/community/decisions/ADR-COMM-0001-moderation-policy-pipeline-architecture.md:87-91`.
Additive 3: tenant-scoped multi-audience mode in one codebase.
Additive 3 evidence: `microservices/community/PRD.md:122-138`.
Additive 4: ActivityPub federation for personal/fediverse mode.
Additive 4 evidence: `microservices/community/PRD.md:98-99`.
Additive 5: Wikipedia-style KB revision model.
Additive 5 evidence: `microservices/community/PRD.md:94-96`.
Additive 6: Stack Overflow accepted-answer and Wilson ranking within the same service.
Additive 6 evidence: `microservices/community/PRD.md:224-228`.
Additive 7: Teamblind-style anonymity and workplace posting modes.
Additive 7 evidence: anonymity policy files and `teamblind-mode.yaml` in inventory.
Additive 8: SecureDrop and whistleblower submission capability files.
Additive 8 evidence: `securedrop-press-source.yaml` and `whistleblower-submission.yaml` in inventory.
Additive 9: bug-bounty submission capability.
Additive 9 evidence: `bug-bounty-submission.yaml` in inventory.
Additive 10: per-pack policy/residency direction in search ADR.
Additive 10 evidence: `microservices/community/decisions/ADR-COMM-0004-content-search-backend.md:85-87`.
Additive 11: rust-strict backend policy alignment and no forbidden backend source files.
Additive 11 evidence: forbidden-language scan result.
Additive 12: OpenAPI 3.2.0 compliance.
Additive 12 evidence: `microservices/community/contracts/openapi/community.yaml:1`.
Additive 13: seven OpenSLO files in service inventory.
Additive 13 evidence: SLO inventory.
Additive 14: nine runbooks already present, even though more are referenced.
Additive 14 evidence: runbook inventory.
Additive 15: broad compliance and DPIA surfaces exist.
Additive 15 evidence: `microservices/community/compliance.md` and `microservices/community/dpia.md` in inventory.

## §8 Final parity verdict

Discourse verdict: strong design parity, blocked by deployment and tier-retirement cleanup.
Circle verdict: partial parity because business, course, revenue, CRM, branded-app, and analytics surfaces are sibling handoffs or gaps.
Vanilla verdict: partial parity because admin analytics, gamification, exact moderation action taxonomy, and migration are incomplete.
Union verdict: community has a strong technical core but cannot claim union coverage until gaps 1 through 10 in §6 are remediated.
Batch verdict: deliverable satisfies Discourse/Circle/Vanilla audit scope without authoring retired capability-tier deltas.
