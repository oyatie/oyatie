# comms-email feature parity matrix — 2026-05-20

µservice: `comms-email`
Counterpart 1: SendGrid
Counterpart 2: Postmark
Counterpart 3: Mailgun
Coverage rule: union coverage against all three, not lowest-common-denominator parity
Tier-delta deliverable: retired and not authored
Quality rule: uniform industry-leader-grade quality across tenant classes
Method: compare current service artifacts against official counterpart surfaces and local product claims

## §1 Counterpart 1 capability surface — SendGrid

1. SendGrid Mail Send is the primary outbound API surface.
2. Evidence: `https://www.twilio.com/docs/sendgrid/api-reference/mail-send`, lines 298-300.
3. SendGrid Mail Send uses global and EU base URLs.
4. Evidence: `https://www.twilio.com/docs/sendgrid/api-reference/mail-send`, lines 303-310.
5. SendGrid supports API-key bearer authentication.
6. Evidence: `https://www.twilio.com/docs/sendgrid/api-reference/mail-send`, lines 312-322.
7. SendGrid supports personalizations for recipients, sender, subject, headers, substitutions, custom args, and send time.
8. Evidence: `https://www.twilio.com/docs/sendgrid/for-developers/sending-email/personalizations`, lines 116-127.
9. SendGrid limits personalizations to 1000 per API request.
10. Evidence: `https://www.twilio.com/docs/sendgrid/for-developers/sending-email/personalizations`, lines 224-226.
11. SendGrid limits total recipients to 1000 per request.
12. Evidence: `https://www.twilio.com/docs/sendgrid/api-reference/mail-send`, lines 390-392.
13. SendGrid limits total email size including attachments to less than 30 MB.
14. Evidence: `https://www.twilio.com/docs/sendgrid/api-reference/mail-send`, lines 390-393.
15. SendGrid supports dynamic templates with personalization and API integration.
16. Evidence: `https://www.twilio.com/en-us/products/email-api/dynamic-email-templates`, lines 871-907.
17. SendGrid supports open engagement through Event Webhook.
18. Evidence: `https://support.sendgrid.com/hc/en-us/articles/1260802360229-Tracking-with-the-Event-Webhook`, lines 21-30.
19. SendGrid Activity Feed exposes events including blocked, bounced, clicked, deferred, delivered, dropped, unsubscribe, opened, processed, and spam report.
20. Evidence: `https://support.sendgrid.com/hc/en-us/articles/6067924604955-Searching-with-Filters-in-the-Email-Activity-Feed`, lines 31-65.
21. SendGrid has unsubscribe methods for marketing modules, ASM tags, subscription tracking, and custom unsubscribe links.
22. Evidence: `https://support.sendgrid.com/hc/en-us/articles/1260806604209-Unsubscribe-Methods`, lines 22-70.
23. SendGrid inbound parse can parse attachments and contents from incoming email.
24. Evidence: `https://www.twilio.com/docs/sendgrid/for-developers/parsing-email/inbound-email`, lines 106-119.
25. SendGrid inbound parse requires MX to `mx.sendgrid.net` and parse settings.
26. Evidence: `https://www.twilio.com/docs/sendgrid/for-developers/parsing-email/inbound-email`, lines 128-145.
27. SendGrid inbound parse has a 30 MB message size limit.
28. Evidence: `https://www.twilio.com/docs/sendgrid/for-developers/parsing-email/inbound-email`, lines 149-151.
29. SendGrid inbound parse drops messages after 3 days without a valid 2xx response.
30. Evidence: `https://www.twilio.com/docs/sendgrid/for-developers/parsing-email/inbound-email`, lines 114-119.
31. SendGrid differentiator: mature transactional API plus broad marketing ecosystem.
32. Oyatie current match: transactional API, templates, webhooks, suppressions, DKIM/SPF/DMARC, tenant domains.
33. Oyatie current gap: no explicit SendGrid-style dynamic template management API in current OpenAPI.
34. Oyatie current gap: inbound parse plans exist, but OpenAPI/AsyncAPI do not expose them.
35. Oyatie current gap: no SendGrid-style activity-feed query contract in current OpenAPI.
36. Oyatie advantage: provider-neutral adapter and audit-chain evidence.
37. Evidence: `competitor-parity-matrix.md:40-49`.
38. Oyatie advantage: self-hosted Postal path for sovereign packs.
39. Evidence: `competitor-parity-matrix.md:28-34`.
40. SendGrid pressure summary: match transactional send, dynamic template governance, event activity, unsubscribe, inbound parse, EU endpoint separation, and migration inventory.

## §2 Counterpart 2 capability surface — Postmark

41. Postmark separates outbound and inbound email in its manual.
42. Evidence: `https://postmarkapp.com/manual`, lines 235-243.
43. Postmark outbound includes transactional and broadcast application email.
44. Evidence: `https://postmarkapp.com/manual`, lines 301-305.
45. Postmark Message Streams classify traffic as transactional or broadcast.
46. Evidence: `https://postmarkapp.com/message-streams`, lines 167-172.
47. Postmark can have up to 10 streams in a server.
48. Evidence: `https://postmarkapp.com/message-streams`, lines 171-174.
49. Postmark explicitly avoids mixing transactional and broadcast infrastructure.
50. Evidence: `https://postmarkapp.com/message-streams`, lines 171-172.
51. Postmark does not provide traditional marketing campaign tooling such as list uploads, WYSIWYG editor, or campaign reporting.
52. Evidence: `https://postmarkapp.com/message-streams`, lines 195-205.
53. Postmark batch endpoint permits up to 500 messages per API call.
54. Evidence: `https://postmarkapp.com/developer/user-guide/send-email-with-api`, lines 352-355.
55. Postmark batch endpoint supports up to 50 MB payload size including attachments.
56. Evidence: `https://postmarkapp.com/developer/user-guide/send-email-with-api`, lines 354-355.
57. Postmark webhooks retry bounce and inbound webhooks on a multi-step schedule from 1 minute through 6 hours.
58. Evidence: `https://postmarkapp.com/developer/webhooks/webhooks-overview`, lines 206-222.
59. Postmark webhooks retry click, open, delivered, and subscription-change events at 1, 5, and 15 minutes.
60. Evidence: `https://postmarkapp.com/developer/webhooks/webhooks-overview`, lines 222-229.
61. Postmark supports bounce, inbound, spam complaint, open, click, delivery, and subscription-change webhooks.
62. Evidence: `https://postmarkapp.com/developer/webhooks/webhooks-overview`, lines 230-238.
63. Postmark stores activity data and message content for 45 days by default.
64. Evidence: `https://postmarkapp.com/support/article/how-does-the-retention-add-on-work`, lines 132-136.
65. Postmark retention can be adjusted from 7 to 365 days through an add-on.
66. Evidence: `https://postmarkapp.com/support/article/how-does-the-retention-add-on-work`, lines 134-142.
67. Postmark templates support standard and layout template types.
68. Evidence: `https://postmarkapp.com/developer/api/templates-api`, lines 749-846.
69. Postmark suppressions can be pulled with reasons such as hard bounce, spam complaint, and manual suppression.
70. Evidence: `https://postmarkapp.com/support/article/881-can-i-export-a-list-of-all-bounces`, lines 142-148.
71. Postmark inbound gives each server an inbound message stream and inbound address.
72. Evidence: `https://postmarkapp.com/manual`, lines 1137-1142.
73. Postmark differentiator: high-confidence transactional stream isolation and simple developer workflow.
74. Oyatie current match: SLOs, webhook normalization, suppression policy, deliverability focus.
75. Evidence: `slos/send-latency-p99.openslo.yaml:12-44`, `decisions/SVC-ADR-002-suppression-list-policy.md:14-26`.
76. Oyatie current gap: no explicit transactional/broadcast message-stream separation.
77. Oyatie current gap: no retention API overlay matching Postmark's configurable retention window.
78. Oyatie current gap: no current contract for inbound server/message-stream configuration.
79. Oyatie advantage: audit-chain retention and pack residency.
80. Evidence: `dpia.md:72-83`, `compliance.md:47-55`.
81. Postmark pressure summary: separate transactional and broadcast streams, inbound message-stream setup, retention policy, template API, and webhook retry transparency.

## §3 Counterpart 3 capability surface — Mailgun

82. Mailgun supports sending through HTTP API and SMTP.
83. Evidence: `https://documentation.mailgun.com/docs/mailgun/user-manual/sending-messages/send-http`, lines 116-123.
84. Mailgun supports EU endpoint substitution for EU domains.
85. Evidence: `https://documentation.mailgun.com/docs/mailgun/user-manual/sending-messages/send-http`, lines 116-117.
86. Mailgun maximum message size is 25 MB.
87. Evidence: `https://documentation.mailgun.com/docs/mailgun/user-manual/sending-messages/send-http`, lines 116-122.
88. Mailgun accepts multiple recipients in HTTP examples and logs accepted/delivered events.
89. Evidence: `https://documentation.mailgun.com/docs/mailgun/user-manual/sending-messages/send-http`, lines 130-143.
90. Mailgun batch sending supports multiple recipients and recipient variables.
91. Evidence: `https://documentation.mailgun.com/docs/mailgun/user-manual/sending-messages/batch-sending`, lines 111-150.
92. Mailgun batch sending maximum is 1000 recipients.
93. Evidence: `https://documentation.mailgun.com/docs/mailgun/user-manual/sending-messages/batch-sending`, lines 147-150.
94. Mailgun webhooks support near-real-time event delivery.
95. Evidence: `https://documentation.mailgun.com/docs/mailgun/user-manual/webhooks/webhooks`, lines 99-107.
96. Mailgun webhook event types include accepted, delivered, temporary failure, permanent failure, opened, clicked, unsubscribed, and complained.
97. Evidence: `https://documentation.mailgun.com/docs/mailgun/user-manual/webhooks/webhooks`, lines 114-123.
98. Mailgun logs track inbound and outbound message events.
99. Evidence: `https://documentation.mailgun.com/docs/mailgun/api-reference/send/mailgun/events/get-v3-domain_name-events`, lines 502-515.
100. Mailgun event API retains inbound and outbound event data for at least 3 days.
101. Evidence: `https://documentation.mailgun.com/docs/mailgun/api-reference/send/mailgun/events/get-v3-domain_name-events`, lines 694-728.
102. Mailgun product page advertises attempted delivery of up to 15 million messages within five minutes.
103. Evidence: `https://www.mailgun.com/products/send/`, lines 164-169.
104. Mailgun product page advertises up to 72 million message requests per hour.
105. Evidence: `https://www.mailgun.com/products/send/`, lines 164-170.
106. Mailgun routes handle incoming email through filters and actions.
107. Evidence: `https://documentation.mailgun.com/docs/mailgun/user-manual/receive-forward-store/routes`, lines 104-115.
108. Mailgun route actions include forward, store, and stop.
109. Evidence: `https://documentation.mailgun.com/docs/mailgun/user-manual/receive-forward-store/route-actions`, lines 105-135.
110. Mailgun store action can temporarily store messages for up to 3 days.
111. Evidence: `https://documentation.mailgun.com/docs/mailgun/user-manual/receive-forward-store/route-actions`, lines 123-130.
112. Mailgun mailing lists can be created and maintained by API or Control Panel.
113. Evidence: `https://documentation.mailgun.com/docs/mailgun/user-manual/sending-messages/mailing-lists`, lines 99-128.
114. Mailgun mailing lists support recipient variables and generated unsubscribe URLs.
115. Evidence: `https://documentation.mailgun.com/docs/mailgun/user-manual/sending-messages/mailing-lists`, lines 186-206.
116. Mailgun suppressions cover bounces, unsubscribes, and complaints.
117. Evidence: `https://help.mailgun.com/hc/en-us/articles/360012287493-Suppressions-Bounces-Complaints-Unsubscribes-Allowlists`, lines 32-69.
118. Mailgun differentiator: bulk throughput, mailing-list surface, routes, logs, and webhooks.
119. Oyatie current match: Mailgun adapter, webhooks, suppressions, bounce handling, list plans.
120. Evidence: `IP-004-mailgun-adapter-impl.md`, `contracts/asyncapi.yaml:18-46`, `IP-018-list-management-usecase.md:11-24`.
121. Oyatie current gap: no Mailgun-style routes contract.
122. Oyatie current gap: no mailing-list member API in current OpenAPI.
123. Oyatie current gap: no public log-query contract equivalent.
124. Oyatie advantage: strict tenant audit-chain emission and provider-neutral routing.
125. Evidence: `PRD.md:25-34`, `decisions/SVC-ADR-003-webhook-retry-policy.md:15-24`.
126. Mailgun pressure summary: batch scale, routes, mailing lists, suppressions, logs, and event types must influence the broadened service contract.

## §4 Union-coverage matrix

| # | Capability | SendGrid | Postmark | Mailgun | Current Oyatie evidence | Status |
|---:|---|---|---|---|---|---|
| 1 | Transactional send API | yes | yes | yes | `contracts/openapi.yaml:27-55` | covered |
| 2 | SMTP relay support | yes | yes | yes | `PRD.md:41-42` | covered |
| 3 | Provider adapter abstraction | no | no | no | `PRD.md:25-34` | ahead |
| 4 | EU endpoint or region handling | yes | account/server scoped | yes | `compliance.md:47-55` | partial |
| 5 | Per-tenant from-domain | yes | yes | yes | `contracts/openapi.yaml:140-162` | covered |
| 6 | Domain verification workflow | yes | yes | yes | `decisions/SVC-ADR-004-tenant-domain-onboard-flow.md:14-33` | covered |
| 7 | DKIM signing | yes | yes | yes | `slos/dkim-signing-rate.openslo.yaml:12-34` | covered |
| 8 | SPF posture | yes | yes | yes | `PRD.md:43-48` | covered |
| 9 | DMARC enforcement | yes | yes | yes | `slos/dmarc-alignment-rate.openslo.yaml:12-35` | covered |
| 10 | DKIM key rotation | partial | partial | partial | `decisions/SVC-ADR-001-dkim-cadence.md:14-24` | ahead |
| 11 | Emergency key revocation | partial | partial | partial | `decisions/ADR-CME-001-per-tenant-signing-key-custody-with-rotation-cadence.md:56-80` | ahead |
| 12 | Suppression list | yes | yes | yes | `contracts/openapi.yaml:87-115` | covered |
| 13 | Suppression removal governance | partial | partial | partial | `decisions/SVC-ADR-002-suppression-list-policy.md:14-26` | ahead |
| 14 | Bounce events | yes | yes | yes | `contracts/asyncapi.yaml:106-129` | covered |
| 15 | Complaint events | yes | yes | yes | `IP-009-bounce-complaint-handler.md` | partial |
| 16 | Delivery events | yes | yes | yes | `contracts/asyncapi.yaml:84-105` | covered |
| 17 | Open tracking | yes | yes | yes | `reference-implementations/send-transactional-rust-sdk.md:122-156` | partial |
| 18 | Click tracking | yes | yes | yes | `reference-implementations/send-transactional-rust-sdk.md:122-156` | partial |
| 19 | Unsubscribe event | yes | yes | yes | `IP-026-unsubscribe-async-emit.md:16-21` | planned |
| 20 | Webhook retry policy | yes | yes | yes | `decisions/SVC-ADR-003-webhook-retry-policy.md:15-24` | covered |
| 21 | Webhook DLQ replay | partial | partial | partial | `runbooks/webhook-replay.md:20-45` | covered |
| 22 | Activity feed query | yes | yes | yes | no current query API | gap |
| 23 | Event retention policy | yes | yes | yes | `dpia.md:72-83` | partial |
| 24 | Dynamic templates | yes | yes | yes | `decisions/SVC-ADR-005-mjml-liquid-canonical.md:14-24` | covered |
| 25 | Template API CRUD | yes | yes | yes | no current template CRUD API | gap |
| 26 | MJML rendering | no | no | no | `IP-006-mjml-template-renderer.md` | ahead |
| 27 | Liquid substitution | no | no | no | `IP-007-liquid-substitution-engine.md` | ahead |
| 28 | Template validation | yes | yes | yes | `decisions/SVC-ADR-005-mjml-liquid-canonical.md:37-42` | partial |
| 29 | Template versioning | yes | yes | yes | no explicit version API | gap |
| 30 | Broadcast/application email separation | marketing campaigns | message streams | mailing lists | no current contract | gap |
| 31 | Traditional list uploads | yes | no | yes | `IP-018-list-management-usecase.md:20-24` | planned |
| 32 | List segmentation | yes | external to Postmark | yes | `IP-018-list-management-usecase.md:20-24` | planned |
| 33 | Double opt-in | yes | external to Postmark | yes | `IP-018-list-management-usecase.md:22-24` | planned |
| 34 | One-click unsubscribe | yes | yes | yes | `IP-019-unsubscribe-handler-domain.md:11-26` | planned |
| 35 | Preference center | yes | suppression-oriented | partial | `IP-019-unsubscribe-handler-domain.md:11-14` | planned |
| 36 | Inbound parse | yes | yes | yes | `IP-016-inbound-receiver-kernel.md:11-23` | planned |
| 37 | Inbound REST retrieval | limited | yes | route/store | `IP-023-inbound-receiver-rest.md:15-21` | planned |
| 38 | Inbound quarantine | limited | partial | route-dependent | `IP-017-inbound-receiver-domain.md:20-24` | planned |
| 39 | Inbound attachment handling | yes | yes | yes | no contract evidence | gap |
| 40 | Inbound route filters | parse host | inbound stream | routes | no route model | gap |
| 41 | Mailing-list reply behavior | campaign-dependent | application-owned | yes | no contract evidence | gap |
| 42 | Dedicated IP warmup | yes | yes | yes | `onboarding/deliverability-engineer-first-week.md:55-94` | covered |
| 43 | Reputation dashboard | yes | analytics | yes | `IP-025-reputation-monitor-rest-and-dashboard.md:11-24` | planned |
| 44 | Google Postmaster integration | yes/analytics | deliverability guidance | yes | `IP-020-reputation-monitor-worker.md:11-25` | planned |
| 45 | Microsoft SNDS integration | yes/analytics | deliverability guidance | yes | `IP-020-reputation-monitor-worker.md:11-25` | planned |
| 46 | Spamhaus/Talos feeds | deliverability tooling | deliverability tooling | deliverability tooling | `capability-tiers/tier-matrix.md:89-91` | needs retirement-safe recut |
| 47 | Self-hosted relay | no | no | no | `iac/helm/postal/Chart.yaml:1-12` | ahead |
| 48 | Air-gapped operation | no | no | no | `PRD.md:32-34` | ahead |
| 49 | Provider-neutral failover | partial | no | partial | `IP-013-multi-region-routing.md` | covered |
| 50 | Postal failover runbook | no | no | no | `runbooks/postal-failover.md:19-63` | covered |
| 51 | SES failover runbook | no | no | no | `incident-response.md:74-85` | covered |
| 52 | Audit-chain emission | no | no | no | `PRD.md:27-28` | ahead |
| 53 | Schema registry integration | no | no | no | `PRD.md:27-28` | ahead |
| 54 | Idempotency key | partial | partial | partial | `reference-implementations/send-transactional-rust-sdk.md:60-120` | covered |
| 55 | Tenant-scoped Cedar authz | no | no | no | `policy/comms-email-send.cedar` | ahead |
| 56 | Abuse defense policy | yes | yes | yes | `policy/abuse-defence.cedar` | partial |
| 57 | Compliance packs | enterprise | enterprise | enterprise | `compliance.md:5-7` | partial |
| 58 | HIPAA path | enterprise | limited | yes | `compliance.md:65-75` | partial |
| 59 | GDPR residency | EU option | EU account | EU endpoint | `compliance.md:47-55` | partial |
| 60 | KR PIPA handling | no direct | no direct | no direct | `compliance.md:77-83` | ahead |
| 61 | KSA/UAE sovereign posture | no direct | no direct | no direct | `compliance.md:84-89` | ahead |
| 62 | Tenant-level cost tagging | billing | billing | billing | `cost-budget.md:47-60` | partial |
| 63 | Per-tenant rate ceiling | yes | yes | yes | `capacity-model.md:14-19` | covered |
| 64 | Usage hard cap | yes | yes | yes | `cost-budget.md:25-30` | covered |
| 65 | Tenant-class semantics | plans | plans | plans | none found | gap |
| 66 | OCI Always Free profile | no | no | no | none found | gap |
| 67 | Six deployment contexts | no | no | no | missing context dirs | gap |
| 68 | OpenTofu context modules | no | no | no | `iac/terraform-module.tf:1-10` | gap |
| 69 | OS support manifest | no | no | no | no `supported-oses.json` | gap |
| 70 | Rust-only backend | not applicable | not applicable | not applicable | no forbidden language files found | covered |
| 71 | SDK reference | yes | yes | yes | `reference-implementations/send-transactional-rust-sdk.md:13-248` | covered |
| 72 | Migration from SendGrid | not applicable | not applicable | not applicable | `migration-playbooks/from-sendgrid-and-mailgun.md:24-34` | covered |
| 73 | Migration from Postmark | not applicable | not applicable | not applicable | `migration-playbooks/from-sendgrid-and-mailgun.md:44-51` | covered |
| 74 | Migration from Mailgun | not applicable | not applicable | not applicable | `migration-playbooks/from-sendgrid-and-mailgun.md:35-42` | covered |
| 75 | Suppression import verification | yes | yes | yes | `migration-playbooks/from-sendgrid-and-mailgun.md:80-97` | covered |
| 76 | DKIM dual-run migration | yes | yes | yes | `migration-playbooks/from-sendgrid-and-mailgun.md:63-79` | covered |
| 77 | Webhook schema mapping | yes | yes | yes | `migration-playbooks/from-sendgrid-and-mailgun.md:138-160` | covered |
| 78 | DMARC tightening plan | yes | yes | yes | `migration-playbooks/from-sendgrid-and-mailgun.md:186-200` | covered |
| 79 | Customer support/onboarding guide | yes | yes | yes | `onboarding/deliverability-engineer-first-week.md:10-21` | covered |
| 80 | Deliverability incident response | yes | yes | yes | `incident-response.md:32-60` | covered |
| 81 | Blacklist response | yes | yes | yes | `incident-response.md:32-46` | covered |
| 82 | Bounce storm response | yes | yes | yes | `incident-response.md:47-60` | covered |
| 83 | DKIM compromise response | enterprise | enterprise | enterprise | `incident-response.md:17-31` | covered |
| 84 | Regulatory erasure suppression | no direct | no direct | no direct | `compliance.md:33-40` | ahead |
| 85 | PHI classification | enterprise | limited | yes | `dpia.md:55-57` | partial |
| 86 | Message body retention minimization | varies | 45 days default | plan-dependent | `dpia.md:72-83` | covered |
| 87 | Template-body storage policy | yes | yes | yes | not explicit | gap |
| 88 | Bot/non-human click handling | partial | partial | partial | no explicit contract | gap |
| 89 | IP pool isolation | yes | dedicated IPs | domains/subaccounts | legacy tier FAQ only | needs recut |
| 90 | Subaccount delegation | yes | servers | domains/subaccounts | legacy tier matrix only | needs recut |
| 91 | BYOK or external HSM | enterprise | no common public path | enterprise | `decisions/ADR-CME-001...:93-111` | partial |
| 92 | Per-pack HSM custody | no direct | no direct | no direct | legacy tier matrix only | needs recut |
| 93 | Rate-limit error taxonomy | yes | yes | yes | `contracts/openapi.yaml:256-273` | covered |
| 94 | Provider error taxonomy | yes | yes | yes | `contracts/openapi.yaml:256-273` | covered |
| 95 | Provider webhook normalization | yes | yes | yes | `contracts/openapi.yaml:117-139` | covered |
| 96 | Provider enum completeness | yes | yes | yes | `contracts/asyncapi.yaml:84-105` | covered |
| 97 | Postmark adapter | not applicable | yes | not applicable | none planned | possible gap |
| 98 | SendGrid adapter | yes | not applicable | not applicable | migration only | possible gap |
| 99 | Mailgun adapter | not applicable | not applicable | yes | `IP-004-mailgun-adapter-impl.md` | covered |
| 100 | SES adapter | no | no | no | `IP-001-ses-adapter-impl.md` | covered |
| 101 | SMTP fallback | yes | yes | yes | `IP-003-smtp-fallback-adapter-impl.md` | covered |
| 102 | In-house relay roadmap | no | no | no | `IP-015-in-house-relay-roadmap-phase-2.md` | planned |
| 103 | Message search by recipient | yes | yes | yes | no current API | gap |
| 104 | Message search by event type | yes | yes | yes | no current API | gap |
| 105 | Export bounces | yes | yes | yes | no current API | gap |
| 106 | Export suppressions | yes | yes | yes | suppression API is list-oriented | partial |
| 107 | Dedicated dashboard evidence | yes | yes | yes | dashboard JSON files | covered |
| 108 | Runtime deployment proof | SaaS-owned | SaaS-owned | SaaS-owned | context IaC missing | gap |
| 109 | Cold-start local dev path | SDK docs | SDK docs | SDK docs | `src/README.md` says future crates | partial |
| 110 | Production acceptance tests | vendor-owned | vendor-owned | vendor-owned | no local tests | gap |

## §5 Family summary

SendGrid family pattern: broad API platform, dynamic templates, marketing ecosystem, event activity, unsubscribe management, and inbound parse.
Postmark family pattern: product/application email, transactional and broadcast streams, strong webhook retry transparency, default 45-day activity retention, and developer-focused templates.
Mailgun family pattern: high-throughput email API, batch recipients, routes, mailing lists, logs, suppressions, and operational deliverability services.
Oyatie family target: provider-neutral, audit-chain-backed, tenant-scoped email substrate with self-host and sovereign pack support.
Oyatie is ahead where vendor neutrality, Postal self-hosting, Cedar, audit-chain, pack overlays, and per-tenant key custody matter.
Oyatie is at parity where transactional send, DKIM/SPF/DMARC, suppressions, webhooks, bounce handling, and migration playbooks exist.
Oyatie is behind where the broadened product needs explicit contracts for inbound, list management, unsubscribe, reputation, template CRUD, and activity search.
Oyatie is structurally behind canonical doctrine where six-context OpenTofu modules, OCI Always Free profile, OS support, and tenant-class semantics are missing.
The existing product docs should not claim complete SendGrid/Postmark/Mailgun union parity until F-01, F-02, F-05, F-06, F-07, and F-09 are closed.

## §6 Headline gap analysis

1. Product boundary gap: PRD transactional-only posture conflicts with README/IP product breadth.
2. Contract gap: OpenAPI/AsyncAPI/proto do not expose inbound, list, reputation, or unsubscribe surfaces.
3. Stream model gap: Postmark-style transactional/broadcast stream separation has no Oyatie contract.
4. Route model gap: Mailgun-style inbound routes and actions have no Oyatie contract.
5. Activity query gap: SendGrid/Mailgun/Postmark-style event and message search has no public API.
6. Template management gap: MJML/Liquid are chosen, but template CRUD/version/validation API is absent.
7. Retention controls gap: DPIA has retention statements, but no tenant/admin retention API overlay.
8. Tenant-class gap: no `tenant_class` semantics or usage overlays.
9. Deployment-context gap: no six-context OpenTofu module proof.
10. OCI profile gap: no OCI Always Free profile module or demo usage cap.
11. OS manifest gap: no service-level supported OS matrix.
12. Test evidence gap: no local tests or implementation crates under the µservice path.
13. Retirement gap: legacy tier documents still shape benchmarks, FAQ, capability files, and runbook language.
14. Counterpart adapter gap: SendGrid and Postmark are migration/counterpart sources but not adapters; if union parity includes adapters, this must be explicit.
15. Compliance gap: inbound/list/marketing expansion needs DPIA and compliance refresh.

## §7 Additive surface

Add `tenant_class` to manifest with allowed values `demo_trial`, `paid`, and `revenue_share`.
Add per-tenant usage overlays for daily send cap, burst cap, retention cap, and provider eligibility.
Add `iac/oyatie-public-cloud/` OpenTofu module or explicit N/A with evidence.
Add `iac/guest-on-aws/` OpenTofu module or explicit N/A with evidence.
Add `iac/oci-guest/` OpenTofu module and `iac/oci-guest/always-free/` profile.
Add `iac/on-prem/` OpenTofu module or explicit N/A with facility assumptions.
Add `iac/colo/` OpenTofu module or explicit N/A with egress and IP reputation assumptions.
Add `iac/oyatie-iaas/` OpenTofu module or explicit N/A with cloud-iac ownership.
Add `supported-oses.json` with Tier 1, test-only, and out-of-scope OS support.
Add a current OpenAPI surface for inbound messages if inbound remains in scope.
Add a current OpenAPI surface for lists and segments if list management remains in scope.
Add a current OpenAPI surface for reputation scores if reputation monitoring remains in scope.
Add an AsyncAPI channel for unsubscribe propagation if unsubscribe remains in scope.
Add an activity-query API for message and event audit search.
Add template CRUD and validation endpoints around MJML/Liquid.
Add retention controls that map to compliance and audit-chain constraints.
Add a route/action model if Mailgun-style inbound routing is a target.
Add a stream model if Postmark-style transactional/broadcast separation is a target.
Add a retirement-safe replacement for the old benchmark report.
Add a retirement-safe replacement for old capability files using capabilities/classes without old tier semantics.
Add a migration note that SendGrid/Postmark/Mailgun are counterpart surfaces, not necessarily all runtime adapters.
Add local test references or crate links so reference implementation claims are executable.
Add cross-microservice handoff data for tenancy, identity, audit-chain, cloud-iac, cell, detection, observability, and intelligence.

## §8 Counterpart union critical-flow decomposition

Outbound send flow: SendGrid exposes a single Mail Send endpoint with personalization arrays and documented request ceilings.
Outbound send evidence: `https://www.twilio.com/docs/sendgrid/api-reference/mail-send`; `https://www.twilio.com/docs/sendgrid/for-developers/sending-email/personalizations`.
Outbound send flow: Postmark separates message streams and has batch send limits that should influence Oyatie batch contracts.
Outbound send evidence: `https://postmarkapp.com/message-streams`; `https://postmarkapp.com/developer/user-guide/send-email-with-api`.
Outbound send flow: Mailgun exposes HTTP send plus batch sending up to counterpart-defined recipient limits.
Outbound send evidence: `https://documentation.mailgun.com/docs/mailgun/user-manual/sending-messages/send-http`; `https://documentation.mailgun.com/docs/mailgun/user-manual/sending-messages/batch-sending`.
Oyatie coverage evidence: `contracts/openapi.yaml:27-64`; `contracts/comms_email.proto:18-37`; `PRD.md:99-107`.
Oyatie gap judgment: outbound transactional send is covered, but batch/campaign/list semantics are not fully contract-aligned.
Parity action: keep transactional send stable while adding explicit batch/list boundaries if marketing-class surface remains in scope.

Domain authentication flow: SendGrid and Mailgun treat authenticated domains as core onboarding surfaces.
Domain authentication evidence: SendGrid docs cited in header; Mailgun sending docs cited in §2.
Domain authentication flow: Postmark also requires server/message stream sender controls and domain verification.
Domain authentication evidence: `https://postmarkapp.com/manual`; `https://postmarkapp.com/message-streams`.
Oyatie coverage evidence: `PRD.md:21-34`; `contracts/openapi.yaml:163-232`; `IP-008-domain-onboarding-state-machine.md`; `SVC-ADR-004-domain-onboarding-state-machine-and-dns-retry-policy.md`.
Oyatie gap judgment: API coverage exists, but deployment-context evidence for DNS, egress, and abuse posture is incomplete.
Parity action: bind domain onboarding to per-context OpenTofu outputs and tenant-class usage caps.

Webhook/event flow: SendGrid exposes event webhook and email activity search surfaces.
Webhook/event evidence: `https://support.sendgrid.com/hc/en-us/articles/1260802360229-Tracking-with-the-Event-Webhook`; `https://support.sendgrid.com/hc/en-us/articles/6067924604955-Searching-with-Filters-in-the-Email-Activity-Feed`.
Webhook/event flow: Postmark documents webhook event categories and retry timing.
Webhook/event evidence: `https://postmarkapp.com/developer/webhooks/webhooks-overview`.
Webhook/event flow: Mailgun exposes event webhooks and a domain events API.
Webhook/event evidence: `https://documentation.mailgun.com/docs/mailgun/user-manual/webhooks/webhooks`; `https://documentation.mailgun.com/docs/mailgun/api-reference/send/mailgun/events/get-v3-domain_name-events`.
Oyatie coverage evidence: `contracts/openapi.yaml:117-162`; `contracts/asyncapi.yaml:18-46`; `SVC-ADR-003-webhook-retry-and-dlq-policy.md`.
Oyatie gap judgment: event emission is strong, while user-facing activity search/export is weak.
Parity action: add query/export endpoints for event type, recipient, domain, provider, and time range.

Suppression flow: SendGrid supports unsubscribe methods and suppression-style sender controls.
Suppression evidence: `https://support.sendgrid.com/hc/en-us/articles/1260806604209-Unsubscribe-Methods`.
Suppression flow: Postmark exposes bounces and activity retention windows that shape suppression investigation.
Suppression evidence: `https://postmarkapp.com/support/article/881-can-i-export-a-list-of-all-bounces`; `https://postmarkapp.com/support/article/how-does-the-retention-add-on-work`.
Suppression flow: Mailgun exposes bounces, complaints, unsubscribes, and allowlists.
Suppression evidence: `https://help.mailgun.com/hc/en-us/articles/360012287493-Suppressions-Bounces-Complaints-Unsubscribes-Allowlists`.
Oyatie coverage evidence: `contracts/openapi.yaml:79-115`; `SVC-ADR-002-suppression-list-removal-and-legal-hold-policy.md`; `PRD.md:112-118`.
Oyatie gap judgment: suppression list core exists, but unsubscribe-center and export controls need current contracts.
Parity action: expose suppression import/export, legal-hold status, unsubscribe token, and audit-chain correlation.

Inbound flow: SendGrid supports inbound parse with MX routing and retry behavior.
Inbound evidence: `https://www.twilio.com/docs/sendgrid/for-developers/parsing-email/inbound-email`; `https://support.sendgrid.com/hc/en-us/articles/46513815674395-Understanding-Inbound-Parse-Webhook-Retry-Logic`.
Inbound flow: Postmark supports inbound streams and inbound message processing.
Inbound evidence: `https://postmarkapp.com/manual`.
Inbound flow: Mailgun supports routes with forward, store, and stop actions.
Inbound evidence: `https://documentation.mailgun.com/docs/mailgun/user-manual/receive-forward-store/routes`; `https://documentation.mailgun.com/docs/mailgun/user-manual/receive-forward-store/route-actions`.
Oyatie coverage evidence: `README.md:17-21`; `IP-016-inbound-receiver-kernel.md:11-23`; `IP-023-inbound-receiver-rest.md:15-21`; `runbooks/inbound-receiver-quarantine-release.md`.
Oyatie gap judgment: inbound is planned and operationally described, but the PRD and public contracts still defer or omit it.
Parity action: choose either active inbound ownership or future-scope retirement, then make every artifact agree.

Template flow: SendGrid dynamic templates and Postmark templates are counterpart expectations for application email.
Template evidence: `https://www.twilio.com/docs/sendgrid/api-reference/mail-send`; `https://postmarkapp.com/developer/api/templates-api`.
Template flow: Mailgun supports templated variables through recipient variables and mailing-list personalization.
Template evidence: `https://documentation.mailgun.com/docs/mailgun/user-manual/sending-messages/batch-sending`; `https://documentation.mailgun.com/docs/mailgun/user-manual/sending-messages/mailing-lists`.
Oyatie coverage evidence: `SVC-ADR-005-template-rendering-engine-selection.md`; `IP-022-template-rendering-mjml-engine.md`; `reference-implementations/send-transactional-rust-sdk.md:13-248`.
Oyatie gap judgment: rendering technology is chosen, but lifecycle APIs for templates, versions, previews, and validation are not visible in the current OpenAPI.
Parity action: add template CRUD/version/preview/validation contracts or explicitly delegate template lifecycle to another µservice.

List and audience flow: SendGrid and Mailgun expose list/unsubscribe-adjacent capabilities, while Postmark intentionally separates broadcast streams and avoids full marketing-platform scope.
List and audience evidence: `https://support.sendgrid.com/hc/en-us/articles/1260806604209-Unsubscribe-Methods`; `https://documentation.mailgun.com/docs/mailgun/user-manual/sending-messages/mailing-lists`; `https://postmarkapp.com/message-streams`.
Oyatie coverage evidence: `README.md:17-21`; `IP-018-list-management-usecase.md:11-24`; `IP-024-list-management-rest.md:15-22`; `PRD.md:92-98`.
Oyatie gap judgment: list management is the most scope-sensitive parity area because it can turn comms-email into a marketing automation product.
Parity action: split list primitives from campaign orchestration, and cite the owner for each boundary.

Deliverability and reputation flow: all three counterparts present reputation, deliverability, bounce, complaint, and suppression behavior as production-critical.
Deliverability evidence: SendGrid Event Webhook docs, Postmark webhook/bounce docs, and Mailgun suppressions/events docs cited above.
Oyatie coverage evidence: `failure-modes.md:27-61`; `incident-response.md:32-60`; `dashboards/reputation-monitoring.json`; `IP-020-reputation-monitor-worker.md`; `IP-025-reputation-monitor-rest-and-dashboard.md`.
Oyatie gap judgment: operational docs are strong, while dashboard/API/customer-visible reputation semantics remain less explicit.
Parity action: define reputation score, sending pause, provider circuit-breaker, and customer notification contracts.

Provider-neutrality flow: counterparts are mostly single-vendor SaaS surfaces, while Oyatie intentionally supports SES, Postal, Mailgun, SMTP, and future relay.
Provider-neutrality evidence: `PRD.md:29-34`; `contracts/asyncapi.yaml:84-105`; `IP-001-ses-adapter-impl.md`; `IP-002-postal-adapter-impl.md`; `IP-003-smtp-fallback-adapter-impl.md`; `IP-004-mailgun-adapter-impl.md`.
Oyatie gap judgment: provider-neutrality is a differentiator, but context IaC and provider eligibility need machine-readable constraints.
Parity action: express provider eligibility by deployment context, tenant class, residency pack, and failover mode.

## §9 Remediation-ready parity slices

| Slice | Product family | Counterpart pressure | Current Oyatie evidence | Needed artifact |
|---|---|---|---|---|
| P-01 | Transactional send | SendGrid/Postmark/Mailgun all support HTTP send | `contracts/openapi.yaml:27-64`; `contracts/comms_email.proto:18-37` | Keep stable; add executable test evidence. |
| P-02 | Batch send | SendGrid personalization, Postmark batch, Mailgun batch | `capacity-model.md:14-19`; benchmark report | Add batch contract with recipient/message ceilings. |
| P-03 | Dynamic template lifecycle | SendGrid and Postmark expose templates | `SVC-ADR-005...`; `IP-022...` | Add template CRUD/version/preview API. |
| P-04 | Broadcast stream separation | Postmark differentiates transactional and broadcast streams | `PRD.md:92-98`; `README.md:13-21` | Decide whether broadcast stream belongs here. |
| P-05 | Mailing lists | Mailgun mailing lists and SendGrid unsubscribe methods | `IP-018...`; `IP-024...` | Add list/segment/subscriber REST contract. |
| P-06 | Unsubscribe center | SendGrid/Mailgun suppression expectations | `IP-019...`; `IP-026...`; `contracts/openapi.yaml:79-115` | Add unsubscribe token, center, and event channel. |
| P-07 | Inbound parse/routes | SendGrid inbound parse, Mailgun routes, Postmark inbound streams | `IP-016...`; `IP-023...`; inbound runbook | Align PRD/contracts/compliance with active inbound scope. |
| P-08 | Activity search | SendGrid activity feed, Mailgun events API, Postmark retention | `contracts/asyncapi.yaml:18-46` | Add event query/export API. |
| P-09 | Bounce export | Postmark and Mailgun expose bounce/suppression data | `contracts/openapi.yaml:117-139`; `SVC-ADR-003...` | Add bounce export and retention constraints. |
| P-10 | Domain onboarding | All counterparts require sender/domain authentication | `contracts/openapi.yaml:163-232`; `SVC-ADR-004...` | Bind DNS state to context and pack overlays. |
| P-11 | Dedicated provider adapters | Counterparts are vendors; Oyatie is adapter substrate | `IP-001...`; `IP-002...`; `IP-003...`; `IP-004...` | State adapter set and non-adapter counterpart roles. |
| P-12 | Sovereign self-host | Vendors do not provide customer-owned Postal deployment | `iac/helm/postal/Chart.yaml`; `PRD.md:32-34` | Add OpenTofu context modules and N/A evidence. |
| P-13 | OCI demo footprint | Vendors do not expose OCI Always Free profile | no service path evidence | Add `iac/oci-guest/always-free/` and demo caps. |
| P-14 | OS support | Vendors abstract host OS | no `supported-oses.json` | Add service OS matrix. |
| P-15 | Tenant classes | Vendors sell plans; Oyatie uses tenant classes | no `tenant_class` evidence | Add `demo_trial`, `paid`, `revenue_share` semantics. |
| P-16 | Audit-chain proof | Counterparts provide vendor logs; Oyatie targets audit-chain | `PRD.md:27-28`; `slos/audit-chain-emit.openslo.yaml` | Add correlation IDs across send/webhook/domain events. |
| P-17 | Cedar authorization | Counterparts provide account permissioning | `policy/*.cedar`; `manifest.json:134-142` | Add handoff to identity/tenancy and policy tests. |
| P-18 | Compliance packs | Vendors have enterprise compliance posture | `compliance.md`; `dpia.md`; pack IaC dirs | Refresh compliance after inbound/list decision. |
| P-19 | Provider failover | Vendors own their internal failover | `failure-modes.md:27-61`; `runbooks/postal-failover.md` | Add automated failover tests and context constraints. |
| P-20 | Reputation dashboard | Vendors expose deliverability analytics | `dashboards/reputation-monitoring.json`; `IP-025...` | Add customer-visible API/dashboard contract. |
| P-21 | Migration support | Vendors provide import/export or API surfaces | `migration-playbooks/from-sendgrid-and-mailgun.md` | Add Postmark-specific migration checks if Postmark remains top-3. |
| P-22 | Retirement-safe capability model | Vendors use plans; Oyatie retired named capability tiers | `manifest.json:96-101`; `capability-tiers/tier-matrix.md` | Replace old capability tier model with tenant-class and context overlays. |

## §10 Substantive closeout

The union-coverage bar is not only “can Oyatie send an email.”
The union-coverage bar includes sender authentication, event delivery, suppression state, inbound handling, audience/list state, template lifecycle, deliverability controls, searchable activity, migration paths, and operational ownership.
The current service is strong in transactional send, provider adapters, DKIM/domain design, suppression core, webhook policy, audit-chain intent, and sovereign/self-host ambition.
The current service is weak in product-boundary coherence because PRD, README, manifest, IPs, contracts, compliance, and benchmarks are not aligned to the same surface.
The current service is weak in canonical-direction compliance because six-context OpenTofu, OCI Always Free profile, OS matrix, and tenant-class semantics are absent.
The current service should not be assessed as a marketing automation suite unless campaign orchestration, segmentation, WYSIWYG editing, experimentation, and reporting ownership are explicitly assigned.
The current service can be assessed as a provider-neutral application-email substrate if it cleanly owns transactional, inbound, list primitive, suppression, webhook, domain, template rendering, and reputation primitives.
The most important next artifact is not another parity matrix.
The most important next artifact is a boundary decision that makes PRD, contracts, compliance, and implementation plans describe one product.
