# Calendar Microservice Ownership-Coherence Audit

Audit date: 2026-05-20.
Target microservice: `calendar`.
Target path: `microservices/calendar/`.
Audit owner: single-agent calendar owner.
Write scope: `microservices/calendar/*` only.
Deliverable set: coherence audit, feature parity matrix, performance benchmark numbers.
Retired deliverable: no capability-tier delta document is authored for this batch.
Counterpart set: Google Calendar, Microsoft Outlook Calendar, Cal.com.

Canonical anchors used:
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1732-1749` requires explicit deployment-context coverage for `oyatie-public-cloud`.
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1785-1797` requires explicit deployment-context coverage for `guest-on-aws`.
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1834-1848` requires explicit deployment-context coverage for `guest-on-oci` and the OCI Always Free module.
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2243-2294` makes OpenTofu the canonical IaC substrate and names the required per-context module shape.
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3950-4000` requires an OS support manifest, package matrix, and CI evidence.
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4011-4083` defines the Rust-strict backend and frontend allowlist.
- `docs/standards/brief-template.md:666-728` defines the multi-context brief anchor and supported context matrix.
- `docs/standards/brief-template.md:809-966` defines the OpenTofu anchor and forbidden IaC patterns.
- `docs/standards/brief-template.md:967-1123` defines the supported-OS anchor and required OS evidence.
- `docs/standards/brief-template.md:1125-1163` defines the language-policy anchor.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_capability_tracks_2026_05_20.md:10-45` retires capability tiers and drops the fourth tier-delta deliverable.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:23-62` defines the tenant-class replacement model and billing-component vocabulary.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-31` requires scope, quality, and substance verification.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md:10-20` requires bespoke service-specific content.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16290-16311` confirms the Wave 3 calendar counterpart set as Google Calendar, Microsoft Outlook Calendar, and Cal.com.

## §1 Purpose

1. This audit determines whether `calendar` is internally coherent as a standalone microservice artifact set.
2. The microservice purpose is not generic date storage; it is an Oyatie-native calendar, scheduling, invitation, availability, recurrence, room-booking, CalDAV, `.ics`, and timezone service.
3. The product purpose is stated in `microservices/calendar/PRD.md:20-26`.
4. The tenant outcome set is stated in `microservices/calendar/PRD.md:30-35`.
5. The architecture says the service owns scheduling coordination, recurrent event expansion, freebusy projection, RSVP flow, and external protocol bridges in `microservices/calendar/ARCHITECTURE.md:40-46`.
6. The contract surface supports event CRUD, availability, room booking, RSVP, `.ics`, and CalDAV entrypoints in `microservices/calendar/contracts/openapi/calendar.yaml:47-363`.
7. The event stream surface supports event lifecycle, invitations, room bookings, recurrence, and legal holds in `microservices/calendar/contracts/asyncapi/calendar-events.yaml:27-69`.
8. The proto surface supports event storage and availability resolution in `microservices/calendar/contracts/proto/calendar.proto:75-83` and `microservices/calendar/contracts/proto/calendar.proto:169-173`.
9. The current artifact set is rich in domain writing and runbook coverage.
10. The current artifact set is weak where canonical Wave 3 requirements require machine-verifiable infrastructure, OS, source, and test evidence.
11. The audit therefore treats prose product depth and executable readiness as separate dimensions.
12. The audit also separates legacy tier vocabulary from legitimate autonomy-level capability files named `T0`, `T1`, and `T2`.
13. Existing tier references are findings, not a model this report extends.
14. No new feature tier model is introduced here.
15. OCI Always Free is evaluated as the OCI Always Free profile and demo-trial infrastructure constraint, not as a feature tier.
16. The audit uses the six deployment contexts required by canonical sequencing.
17. The six deployment contexts are `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`.
18. The audit uses the Rust-strict backend policy from canonical direction.
19. The audit uses Swift, Kotlin, WinUI 3, and Leptos/WASM SSR selective hydration as the frontend allowlist where frontend concerns appear.
20. The audit treats absence of files as a gap only when a canonical source or service artifact claims the missing evidence should exist.
21. The audit is not a code implementation review because `calendar` currently has no `src/` directory under the microservice path.
22. The audit is not a test execution review because `calendar` currently has no `tests/` directory under the microservice path.
23. The audit is a coherence audit across artifacts, contracts, operational docs, and canonical direction.
24. The stop condition for this report is three landed deliverables with evidence-backed findings and no fourth tier-delta document.

## §2 Inventory

Inventory method:
- `rg --files microservices/calendar | sort` was used for the complete file inventory.
- The inventory contained 139 files.
- `wc -l` over the inventory reported 26,495 source-document lines in scope.
- The target deliverables did not exist before the write phase.
- Pre-existing modified and untracked calendar files were left intact.

Complete inventory:
1. `microservices/calendar/ARCHITECTURE.md`
2. `microservices/calendar/AUDIT-FINDINGS-2026-05-18.json`
3. `microservices/calendar/IP-001-iac-bootstrap.md`
4. `microservices/calendar/IP-002-event-store-kernel.md`
5. `microservices/calendar/IP-003-event-store-domain-and-usecase.md`
6. `microservices/calendar/IP-004-event-store-adapter-postgres.md`
7. `microservices/calendar/IP-005-recurrence-engine.md`
8. `microservices/calendar/IP-006-availability-resolver.md`
9. `microservices/calendar/IP-007-room-booking.md`
10. `microservices/calendar/IP-008-invitation-flow.md`
11. `microservices/calendar/IP-009-ics-import-export-and-caldav.md`
12. `microservices/calendar/IP-010-tzdb-refresh-worker.md`
13. `microservices/calendar/IP-011-contracts-openapi-asyncapi-proto.md`
14. `microservices/calendar/IP-012-cedar-policies-and-data-residency.md`
15. `microservices/calendar/IP-013-workflow-handoff.md`
16. `microservices/calendar/IP-014-hg-calendar-authority-cohesion.md`
17. `microservices/calendar/IP-015-hg-calendar-registration-and-branch-protection.md`
18. `microservices/calendar/IP-journey-j100-pack-rollout-first-action.md`
19. `microservices/calendar/IP-journey-j113-shift-and-mentor-scheduling.md`
20. `microservices/calendar/IP-journey-j132-cross-tenant-interview-booking.md`
21. `microservices/calendar/IP-journey-j144-interview-slot-scheduling.md`
22. `microservices/calendar/IP-journey-j27-dual-context-freebusy.md`
23. `microservices/calendar/IP-journey-j35-work-freebusy.md`
24. `microservices/calendar/IP-journey-j56-interview-scheduling.md`
25. `microservices/calendar/IP-journey-j57-orientation-schedule.md`
26. `microservices/calendar/IP-journey-j58-one-on-one.md`
27. `microservices/calendar/IP-journey-j69-schedule-control.md`
28. `microservices/calendar/IP-journey-j91-us-msb-mtl-overlay.md`
29. `microservices/calendar/IP-journey-j92-br-lgpd-us-parent-dsar.md`
30. `microservices/calendar/IP-journey-j93-in-dpdpa-rbi-overlay.md`
31. `microservices/calendar/IP-journey-j94-sox404-public-company-controls.md`
32. `microservices/calendar/IP-journey-j95-iso27001-soc2-annual-audit.md`
33. `microservices/calendar/IP-journey-j96-ksa-uae-mena-onboarding.md`
34. `microservices/calendar/IP-journey-j97-sg-pdpa-mas-tenant.md`
35. `microservices/calendar/IP-journey-j98-au-privacy-apra-cps234.md`
36. `microservices/calendar/IP-journey-j99-multi-pack-conflict-resolution.md`
37. `microservices/calendar/PHASE-01-CALENDAR-FOUNDATION.md`
38. `microservices/calendar/PRD.md`
39. `microservices/calendar/backfill-replay.md`
40. `microservices/calendar/benchmarks/gcal-outlook-calendly-vs-oyatie.md`
41. `microservices/calendar/capabilities/T0-suggest.yaml`
42. `microservices/calendar/capabilities/T1-assist.yaml`
43. `microservices/calendar/capabilities/T2-auto.yaml`
44. `microservices/calendar/capability-tiers/tier-matrix.md`
45. `microservices/calendar/capacity-model.md`
46. `microservices/calendar/catalog/oya-calendar-availability-resolver-adapter-valkey.yaml`
47. `microservices/calendar/catalog/oya-calendar-availability-resolver-kernel.yaml`
48. `microservices/calendar/catalog/oya-calendar-event-store-adapter-postgres.yaml`
49. `microservices/calendar/catalog/oya-calendar-event-store-app.yaml`
50. `microservices/calendar/catalog/oya-calendar-event-store-domain.yaml`
51. `microservices/calendar/catalog/oya-calendar-event-store-kernel.yaml`
52. `microservices/calendar/catalog/oya-calendar-event-store-rest.yaml`
53. `microservices/calendar/catalog/oya-calendar-event-store-usecase.yaml`
54. `microservices/calendar/catalog/oya-calendar-event-store-worker.yaml`
55. `microservices/calendar/catalog/oya-calendar-ics-import-export-adapter-caldav-radicale.yaml`
56. `microservices/calendar/catalog/oya-calendar-ics-import-export-adapter-caldav-sabredav.yaml`
57. `microservices/calendar/catalog/oya-calendar-ics-import-export-adapter-icalendar.yaml`
58. `microservices/calendar/catalog/oya-calendar-invitation-flow-kernel.yaml`
59. `microservices/calendar/catalog/oya-calendar-recurrence-engine-adapter.yaml`
60. `microservices/calendar/catalog/oya-calendar-recurrence-engine-kernel.yaml`
61. `microservices/calendar/catalog/oya-calendar-room-booking-kernel.yaml`
62. `microservices/calendar/catalog/oya-calendar-tzdb-refresh-worker.yaml`
63. `microservices/calendar/competitor-parity-matrix.md`
64. `microservices/calendar/compliance.md`
65. `microservices/calendar/contracts/asyncapi/calendar-events.yaml`
66. `microservices/calendar/contracts/openapi/calendar.yaml`
67. `microservices/calendar/contracts/proto/calendar.proto`
68. `microservices/calendar/cost-budget.md`
69. `microservices/calendar/dashboards/availability-and-freebusy.json`
70. `microservices/calendar/dashboards/ics-import-export.json`
71. `microservices/calendar/dashboards/scheduling-pipeline.json`
72. `microservices/calendar/decisions/ADR-CAL-0001-caldav-server-backend-selection.md`
73. `microservices/calendar/decisions/ADR-CAL-0002-recurrence-engine-rfc-conformance.md`
74. `microservices/calendar/decisions/ADR-CAL-0003-jmap-vs-caldav-frontend-priority.md`
75. `microservices/calendar/decisions/ADR-CAL-0004-tzdb-refresh-and-pinning-policy.md`
76. `microservices/calendar/decisions/ADR-CAL-001-icalendar-rfc5545-rfc7986-freebusy-acl.md`
77. `microservices/calendar/decisions/README.md`
78. `microservices/calendar/deprecation-notice.md`
79. `microservices/calendar/dpia.md`
80. `microservices/calendar/failure-modes.md`
81. `microservices/calendar/faqs/calendar-engineer-faq.md`
82. `microservices/calendar/iac/helm/Chart.yaml`
83. `microservices/calendar/iac/helm/templates/cronjob.yaml`
84. `microservices/calendar/iac/helm/templates/deployment.yaml`
85. `microservices/calendar/iac/helm/templates/hpa.yaml`
86. `microservices/calendar/iac/helm/templates/networkpolicy.yaml`
87. `microservices/calendar/iac/helm/templates/pdb.yaml`
88. `microservices/calendar/iac/helm/templates/prometheusrule.yaml`
89. `microservices/calendar/iac/helm/templates/service.yaml`
90. `microservices/calendar/iac/helm/templates/servicemonitor.yaml`
91. `microservices/calendar/iac/helm/values.yaml`
92. `microservices/calendar/iac/kustomize/base/kustomization.yaml`
93. `microservices/calendar/iac/kustomize/base/namespace.yaml`
94. `microservices/calendar/iac/kustomize/base/serviceaccount.yaml`
95. `microservices/calendar/iac/kustomize/overlays/pack-kr/kustomization.yaml`
96. `microservices/calendar/iac/kustomize/overlays/pack-us-healthcare/kustomization.yaml`
97. `microservices/calendar/incident-response.md`
98. `microservices/calendar/manifest.json`
99. `microservices/calendar/migration-from-connect.md`
100. `microservices/calendar/migration-playbooks/from-google-calendar.md`
101. `microservices/calendar/multi-region.md`
102. `microservices/calendar/onboarding/calendar-engineer-first-week.md`
103. `microservices/calendar/packs/EU-AI-Act.md`
104. `microservices/calendar/packs/GDPR.md`
105. `microservices/calendar/packs/HIPAA.md`
106. `microservices/calendar/packs/KR-PIPA.md`
107. `microservices/calendar/packs/SOC2.md`
108. `microservices/calendar/policy/auditor-scope.cedar`
109. `microservices/calendar/policy/ci-scope.cedar`
110. `microservices/calendar/policy/data-residency.md`
111. `microservices/calendar/policy/event-isolation.md`
112. `microservices/calendar/policy/public-read.cedar`
113. `microservices/calendar/policy/tenant-scope.cedar`
114. `microservices/calendar/reference-implementations/create-event-with-recurrence-rust-sdk.md`
115. `microservices/calendar/runbooks/availability-cache-rebuild.md`
116. `microservices/calendar/runbooks/caldav-sync-loop.md`
117. `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`
118. `microservices/calendar/runbooks/calendar-restore.md`
119. `microservices/calendar/runbooks/ics-import-failure.md`
120. `microservices/calendar/runbooks/recurrence-storm.md`
121. `microservices/calendar/runbooks/room-booking-conflict.md`
122. `microservices/calendar/runbooks/rsvp-storm-throttle.md`
123. `microservices/calendar/runbooks/scheduling-poll-deadlock.md`
124. `microservices/calendar/runbooks/shared-cal-permission-drift.md`
125. `microservices/calendar/runbooks/timezone-db-refresh.md`
126. `microservices/calendar/runbooks/tzdb-rollback.md`
127. `microservices/calendar/scorecards/overrides.json`
128. `microservices/calendar/sdk-plan.md`
129. `microservices/calendar/slos/agenda-render-latency.openslo.yaml`
130. `microservices/calendar/slos/caldav-availability.openslo.yaml`
131. `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`
132. `microservices/calendar/slos/ics-import-throughput.openslo.yaml`
133. `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`
134. `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`
135. `microservices/calendar/slos/rsvp-fanout-latency.openslo.yaml`
136. `microservices/calendar/slos/scheduling-convergence-latency.openslo.yaml`
137. `microservices/calendar/slos/tzdb-staleness-bound.openslo.yaml`
138. `microservices/calendar/threat-model.md`
139. `microservices/calendar/tutorials/configure-freebusy-acl-cross-tenant-interview.md`

Inventory interpretation:
- Core product docs are present: PRD, architecture, capacity, failure modes, incident response, cost, DPIA, compliance, and threat model.
- Contract docs are present for OpenAPI, AsyncAPI, and proto.
- Operational docs are present for runbooks, dashboards, SLOs, migration, onboarding, FAQ, tutorial, and reference implementation.
- Implementation plans are broad and service-specific.
- IaC evidence is limited to Helm and Kustomize.
- No canonical OpenTofu context module is present.
- No `supported-oses.json` file is present.
- No `src/` directory is present under this microservice path.
- No `tests/` directory is present under this microservice path.
- The `capability-tiers/` directory is present and is a Wave 15J retirement target.
- The `capabilities/T0-suggest.yaml`, `capabilities/T1-assist.yaml`, and `capabilities/T2-auto.yaml` files appear to describe autonomy levels, but they still contain tenant-tier language that must be reviewed.

## §3 Nine-Dimension Audit

### §3.1 Dimension 1 - Product Purpose And Counterpart Fit

Evidence:
- `microservices/calendar/PRD.md:20-26` defines the service as the Oyatie calendar, scheduling, invitation, room-booking, recurring-event, timezone, CalDAV, and `.ics` boundary.
- `microservices/calendar/PRD.md:30-35` names tenant outcomes: create events, invite internal/external attendees, resolve availability, book shared resources, synchronize external calendars, and preserve policy.
- `microservices/calendar/PRD.md:41-55` lists functional requirements across event CRUD, recurrence, availability, invitations, rooms, CalDAV, `.ics`, timezone, reminders, and legal holds.
- `microservices/calendar/contracts/openapi/calendar.yaml:47-363` has API coverage for most of those product surfaces.
- `microservices/calendar/contracts/proto/calendar.proto:75-83` and `microservices/calendar/contracts/proto/calendar.proto:169-173` express service-level event-store and availability service boundaries.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16290-16311` confirms the current top-three counterpart set.

Assessment:
- The actual product purpose is coherent and materially calendar-shaped.
- The product is closer to Google Calendar and Microsoft Outlook Calendar in enterprise-calendar scope than to Cal.com alone.
- Cal.com is still a valid counterpart because Oyatie Calendar contains booking, availability, external invitee, and workflow surfaces that overlap Cal.com.
- The older PRD competitor table includes Apple Calendar, Calendly, Fantastical, and Naver Works in `microservices/calendar/PRD.md:227-247`.
- That older competitor table is useful context but does not match the current top-three audit union.
- The service purpose should therefore be documented as an enterprise calendar core plus scheduling and booking edge, not as a narrow booking-link service.

Judgment:
- Product-purpose coherence: strong.
- Counterpart fit: strong after normalizing the counterpart set to Google Calendar, Microsoft Outlook Calendar, and Cal.com.
- Documentation drift: moderate because prior docs use Calendly and Apple Calendar as primary comparators while the current batch uses Cal.com.

### §3.2 Dimension 2 - Artifact Completeness And Internal Structure

Evidence:
- `microservices/calendar/PRD.md:149` declares 44 Rust crates.
- `microservices/calendar/PRD.md:171-183` lists CI gates and expected verification.
- `microservices/calendar/PRD.md:285-300` names acceptance tests and cargo benchmarks.
- The complete inventory in §2 has no `microservices/calendar/src/` directory.
- The complete inventory in §2 has no `microservices/calendar/tests/` directory.
- `microservices/calendar/ARCHITECTURE.md:3` says the architecture was created by an anchor sweep and that stub sections should be expanded.
- `microservices/calendar/ARCHITECTURE.md:571-578` describes Kubernetes deployment shape and names Helm/Kustomize evidence.
- `microservices/calendar/decisions/README.md` is present, and five ADR files are present.
- `microservices/calendar/slos/` contains nine OpenSLO YAML documents.
- `microservices/calendar/runbooks/` contains twelve runbooks.
- `microservices/calendar/dashboards/` contains three dashboard JSON files.

Assessment:
- The documentation suite is broad and service-specific.
- The artifact suite is not yet executable in the sense implied by the PRD acceptance criteria.
- The architecture file contains useful domain sections, but its own header warns that stub expansion remains.
- The runbook and dashboard surfaces show operational seriousness.
- The absence of source and tests makes implementation claims unverified.

Judgment:
- Documentation breadth: strong.
- Executable artifact completeness: weak.
- Coherence risk: high where docs claim tests or crates that are not present in the microservice path.

### §3.3 Dimension 3 - Contract And Interface Coherence

Evidence:
- `microservices/calendar/contracts/openapi/calendar.yaml:47-69` defines event creation.
- `microservices/calendar/contracts/openapi/calendar.yaml:106-143` defines event lookup and mutation by event id.
- `microservices/calendar/contracts/openapi/calendar.yaml:149-158` defines legal hold operations.
- `microservices/calendar/contracts/openapi/calendar.yaml:174-177` defines recurrence expansion.
- `microservices/calendar/contracts/openapi/calendar.yaml:194-215` defines availability resolution.
- `microservices/calendar/contracts/openapi/calendar.yaml:212-230` defines cross-tenant grant behavior.
- `microservices/calendar/contracts/openapi/calendar.yaml:243-265` defines room booking behavior.
- `microservices/calendar/contracts/openapi/calendar.yaml:286-295` defines RSVP behavior.
- `microservices/calendar/contracts/openapi/calendar.yaml:313-333` defines `.ics` import and export.
- `microservices/calendar/contracts/openapi/calendar.yaml:360-363` points CalDAV details to `contracts/caldav/`.
- `microservices/calendar/contracts/asyncapi/calendar-events.yaml:27-69` defines lifecycle, invitation, room, recurrence, and legal-hold channels.
- `microservices/calendar/contracts/proto/calendar.proto:180-183` defines a freebusy projection that excludes private details.
- `microservices/calendar/contracts/proto/calendar.proto:195-199` caps a freebusy query at 100 attendees.

Assessment:
- OpenAPI, AsyncAPI, and proto are aligned on the major calendar concepts.
- The cross-tenant freebusy privacy rule is explicit in proto.
- The AsyncAPI surface covers the event families expected by the PRD.
- The OpenAPI CalDAV pointer is not backed by a `contracts/caldav/` directory in the inventory.
- That missing CalDAV contract detail matters because CalDAV is one of the service's explicit differentiators.

Judgment:
- Contract family breadth: strong.
- Contract-to-file closure: moderate.
- Highest contract gap: missing CalDAV detail directory referenced by the OpenAPI document.

### §3.4 Dimension 4 - Canonical-Direction Alignment

Evidence:
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1732-1749` requires the `oyatie-public-cloud` context.
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1785-1797` requires the `guest-on-aws` context.
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1834-1848` requires the `guest-on-oci` context and OCI Always Free module.
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1882-1895` requires the `on-prem` context.
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1932-1945` requires the `colo` context.
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1981-1994` requires the `oyatie-as-cloud-provider` context.
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2079-2084` requires microservice manifests to name context support as an array of ids or justify N/A.
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2198-2200` forbids claiming context support without an IaC module or N/A manifest.
- `microservices/calendar/manifest.json:434-445` lists dependencies but does not expose a deployment-context matrix.
- `microservices/calendar/IP-001-iac-bootstrap.md:16-26` makes Helm and Kustomize the planned IaC surface.
- `microservices/calendar/IP-001-iac-bootstrap.md:37-56` lists Helm and Kustomize file targets rather than OpenTofu context modules.
- `microservices/calendar/compliance.md:1110` mentions Helm, Kustomize, and OpenTofu in one inventory sentence, but the inventory has no OpenTofu files in the calendar path.

Assessment:
- Calendar is not aligned with the canonical six-context deployment model.
- Calendar is not aligned with OpenTofu-only IaC requirements.
- Calendar is not aligned with the supported-OS manifest requirement.
- Calendar is aligned with Rust-strict file-type absence because no forbidden source-language file extension was found in the path.
- Calendar is not aligned with the tenant-class model because no tenant-class vocabulary appears in the path.
- Calendar carries retired tier language in many docs.

#### §3.4.T - Tier Retirement Candidates

Tier scan method:
- `rg -n -i "demo_trial|paid|paid|compliance_pack-bound paid" microservices/calendar` returned 64 raw matches.
- Seven raw matches are false positives because they refer to baseline signals, baseline dashboards, baseline test data, or serialization names.
- Fifty-seven direct retired-name citation lines remain as Wave 15J retirement candidates.
- Additional generic `tier` vocabulary appears in PRD, architecture, cost, capability, and manifest files and should be reviewed, but the fifty-seven direct-name lines are the counted retirement candidate set.

Direct retired-name candidates:
1. `microservices/calendar/benchmarks/gcal-outlook-calendly-vs-oyatie.md:13` uses the retired ladder in benchmark framing.
2. `microservices/calendar/benchmarks/gcal-outlook-calendly-vs-oyatie.md:21` uses a retired class label.
3. `microservices/calendar/benchmarks/gcal-outlook-calendly-vs-oyatie.md:22` uses a retired class label.
4. `microservices/calendar/benchmarks/gcal-outlook-calendly-vs-oyatie.md:23` uses a retired class label.
5. `microservices/calendar/benchmarks/gcal-outlook-calendly-vs-oyatie.md:35` uses a retired class label.
6. `microservices/calendar/benchmarks/gcal-outlook-calendly-vs-oyatie.md:36` uses a retired class label.
7. `microservices/calendar/benchmarks/gcal-outlook-calendly-vs-oyatie.md:48` uses a retired class label.
8. `microservices/calendar/benchmarks/gcal-outlook-calendly-vs-oyatie.md:49` uses a retired class label.
9. `microservices/calendar/benchmarks/gcal-outlook-calendly-vs-oyatie.md:61` uses a retired class label.
10. `microservices/calendar/benchmarks/gcal-outlook-calendly-vs-oyatie.md:62` uses a retired class label.
11. `microservices/calendar/benchmarks/gcal-outlook-calendly-vs-oyatie.md:74` uses a retired class label.
12. `microservices/calendar/benchmarks/gcal-outlook-calendly-vs-oyatie.md:75` uses a retired class label.
13. `microservices/calendar/benchmarks/gcal-outlook-calendly-vs-oyatie.md:87` uses a retired class label.
14. `microservices/calendar/benchmarks/gcal-outlook-calendly-vs-oyatie.md:88` uses a retired class label.
15. `microservices/calendar/benchmarks/gcal-outlook-calendly-vs-oyatie.md:110` uses a retired class label.
16. `microservices/calendar/capability-tiers/tier-matrix.md:15` uses the retired ladder.
17. `microservices/calendar/capability-tiers/tier-matrix.md:58` uses a retired class label.
18. `microservices/calendar/capability-tiers/tier-matrix.md:60` uses a retired class label.
19. `microservices/calendar/capability-tiers/tier-matrix.md:91` uses a retired class label.
20. `microservices/calendar/capability-tiers/tier-matrix.md:93` uses a retired class label.
21. `microservices/calendar/capability-tiers/tier-matrix.md:122` uses a retired class label.
22. `microservices/calendar/capability-tiers/tier-matrix.md:126` uses a retired class label.
23. `microservices/calendar/capability-tiers/tier-matrix.md:128` uses a retired class label.
24. `microservices/calendar/capability-tiers/tier-matrix.md:144` uses a retired class label.
25. `microservices/calendar/capability-tiers/tier-matrix.md:147` uses a retired class label.
26. `microservices/calendar/capability-tiers/tier-matrix.md:162` uses a retired class label.
27. `microservices/calendar/capability-tiers/tier-matrix.md:171` uses a retired class label.
28. `microservices/calendar/capability-tiers/tier-matrix.md:172` uses a retired class label.
29. `microservices/calendar/capability-tiers/tier-matrix.md:173` uses a retired class label.
30. `microservices/calendar/capability-tiers/tier-matrix.md:175` uses a retired class label.
31. `microservices/calendar/capability-tiers/tier-matrix.md:176` uses a retired class label.
32. `microservices/calendar/capability-tiers/tier-matrix.md:177` uses a retired class label.
33. `microservices/calendar/capability-tiers/tier-matrix.md:178` uses a retired class label.
34. `microservices/calendar/capability-tiers/tier-matrix.md:179` uses a retired class label.
35. `microservices/calendar/capability-tiers/tier-matrix.md:180` uses a retired class label.
36. `microservices/calendar/faqs/calendar-engineer-faq.md:124` uses a retired class label.
37. `microservices/calendar/faqs/calendar-engineer-faq.md:128` uses a retired class label.
38. `microservices/calendar/faqs/calendar-engineer-faq.md:130` uses a retired class label.
39. `microservices/calendar/faqs/calendar-engineer-faq.md:184` uses a retired class label.
40. `microservices/calendar/migration-playbooks/from-google-calendar.md:30` uses a retired class label.
41. `microservices/calendar/migration-playbooks/from-google-calendar.md:32` uses a retired class label.
42. `microservices/calendar/migration-playbooks/from-google-calendar.md:77` uses a retired class label.
43. `microservices/calendar/migration-playbooks/from-google-calendar.md:79` uses a retired class label.
44. `microservices/calendar/migration-playbooks/from-google-calendar.md:109` uses a retired class label.
45. `microservices/calendar/migration-playbooks/from-google-calendar.md:179` uses a retired class label.
46. `microservices/calendar/onboarding/calendar-engineer-first-week.md:12` uses a retired class label.
47. `microservices/calendar/onboarding/calendar-engineer-first-week.md:27` uses a retired class label.
48. `microservices/calendar/onboarding/calendar-engineer-first-week.md:31` uses a retired class label.
49. `microservices/calendar/onboarding/calendar-engineer-first-week.md:173` uses a retired class label.
50. `microservices/calendar/onboarding/calendar-engineer-first-week.md:175` uses a retired class label.
51. `microservices/calendar/onboarding/calendar-engineer-first-week.md:311` uses a retired class label.
52. `microservices/calendar/onboarding/calendar-engineer-first-week.md:318` uses a retired class label.
53. `microservices/calendar/reference-implementations/create-event-with-recurrence-rust-sdk.md:167` uses a retired class label.
54. `microservices/calendar/reference-implementations/create-event-with-recurrence-rust-sdk.md:264` uses a retired class label.
55. `microservices/calendar/tutorials/configure-freebusy-acl-cross-tenant-interview.md:16` uses a retired class label.
56. `microservices/calendar/tutorials/configure-freebusy-acl-cross-tenant-interview.md:119` uses a retired class label.
57. `microservices/calendar/tutorials/configure-freebusy-acl-cross-tenant-interview.md:266` uses a retired class label.

Classification:
- Severity: P2 for all direct retired-name candidate lines.
- Remediation: Wave 15J should retire `microservices/calendar/capability-tiers/` and convert benchmark, onboarding, FAQ, migration, tutorial, and reference examples to tenant-class and billing-component vocabulary.
- False-positive raw matches: `microservices/calendar/ARCHITECTURE.md:673`, `microservices/calendar/manifest.json:325`, `microservices/calendar/manifest.json:407`, `microservices/calendar/migration-from-connect.md:397`, `microservices/calendar/onboarding/calendar-engineer-first-week.md:21`, `microservices/calendar/runbooks/caldav-sync-loop.md:124`, and `microservices/calendar/runbooks/caldav-sync-loop.md:164` are not customer capability class labels.
- Generic `tier` terms remain in `microservices/calendar/PRD.md:8-9`, `microservices/calendar/cost-budget.md:48-74`, `microservices/calendar/manifest.json:379-409`, `microservices/calendar/ARCHITECTURE.md:321-332`, `microservices/calendar/ARCHITECTURE.md:571-578`, and `microservices/calendar/ARCHITECTURE.md:700`; these need a separate vocabulary pass because some may mean service classification or autonomy level rather than customer capability tier.

#### §3.4.C - Tenant-Class Adoption Gaps

Evidence:
- `rg -n "tenant_class|demo_trial|revenue_share|per_seat|per_usage" microservices/calendar` returned no matches during investigation.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:23-35` defines demo-trial and paid tenant behavior.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:38-62` defines billing components including revenue share, per-seat, and per-usage.
- The current user directive for this batch names `demo_trial`, `paid`, and `revenue_share` as the three tenant-class terms.

Assessment:
- Calendar currently expresses no tenant-class semantics.
- Calendar cost docs still express Free, Starter, Pro, and Enterprise vocabulary in `microservices/calendar/cost-budget.md:48-53`.
- Calendar SLO and capability docs use `tier` labels without tenant-class semantics.
- The service cannot yet describe demo-trial caps, paid scaling, or revenue-share substrate economics in machine-readable form.
- There is a canonical tension between the current batch instruction that treats `revenue_share` as a tenant class and the memory file that treats revenue share as a billing component under paid tenants.

Judgment:
- Tenant-class adoption gap: yes.
- Severity: P2 until Wave 15J canonical registry finalizes the exact enum.
- Remediation: introduce a calendar-owned tenant entitlement and quota model after the central tenant-class vocabulary is stabilized.

### §3.5 Dimension 5 - Deployment Context And IaC Coherence

Evidence:
- `microservices/calendar/IP-001-iac-bootstrap.md:16-26` scopes IaC to Helm and Kustomize.
- `microservices/calendar/IP-001-iac-bootstrap.md:37-56` names Helm chart and Kustomize overlays as target files.
- `microservices/calendar/iac/helm/Chart.yaml` exists.
- `microservices/calendar/iac/kustomize/base/kustomization.yaml` exists.
- `microservices/calendar/iac/kustomize/overlays/pack-kr/kustomization.yaml` exists.
- `microservices/calendar/iac/kustomize/overlays/pack-us-healthcare/kustomization.yaml` exists.
- No `microservices/calendar/iac/oyatie-public-cloud/` path exists in the inventory.
- No `microservices/calendar/iac/guest-on-aws/` path exists in the inventory.
- No `microservices/calendar/iac/oci-guest/` path exists in the inventory.
- No `microservices/calendar/iac/oci-guest/always-free/` path exists in the inventory.
- No `microservices/calendar/iac/on-prem/` path exists in the inventory.
- No `microservices/calendar/iac/colo/` path exists in the inventory.
- No `microservices/calendar/iac/oyatie-iaas/` path exists in the inventory.
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2275-2294` names the required OpenTofu per-context module directories.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_zero_handroll_opentofu_only_2026_05_20.md:20-35` says each microservice must have OpenTofu modules for all applicable contexts or explicit N/A decisions.

Assessment:
- Calendar has useful Kubernetes deployment artifacts.
- Calendar does not have the canonical OpenTofu deployment substrate.
- Helm and Kustomize can remain lower-level deploy packaging, but they cannot satisfy the Wave 3 OpenTofu requirement by themselves.
- The absence of OCI Always Free profile IaC is a distinct gap because the canonical doctrine calls it out separately.

Judgment:
- Multi-context coverage: failing.
- OpenTofu coverage: failing.
- OCI Always Free profile: failing.
- Severity: P1 for deployment/IaC because the service cannot substantiate all claimed deployable contexts.

### §3.6 Dimension 6 - OS And Runtime-Portability Coherence

Evidence:
- `docs/standards/brief-template.md:967-1123` requires the OS manifest, OS list, architecture matrix, package formats, and CI evidence.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:10-31` names the OS-support doctrine.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:35-78` requires build, packaging, and manifest evidence.
- `microservices/calendar/PRD.md:171-183` lists CI gates but not the supported OS matrix.
- No `microservices/calendar/supported-oses.json` file exists in the inventory.
- No calendar-local packaging matrix exists in the inventory.

Assessment:
- Calendar docs are deployment-oriented but not OS-portability-complete.
- Rust backend portability is plausible, but plausibility is not canonical evidence.
- Helm/Kustomize deployment docs do not answer native package support for macOS, Windows, Linux, BSD, illumos, Haiku, or mobile platforms.
- The service needs a supported-OS manifest tied to its actual runtime surfaces: backend service, worker processes, CLI/admin utilities, and any web/native frontend surfaces.

Judgment:
- OS support manifest: absent.
- OS support CI evidence: absent.
- Severity: P1 because canonical Wave 3 treats absence as a required audit finding.

### §3.7 Dimension 7 - Language Policy And Source-Shape Coherence

Evidence:
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4011-4083` defines Rust-strict backend policy and authorized non-Rust boundaries.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:10-18` states the Rust-only backend doctrine.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:38-49` lists authorized non-Rust file families.
- The forbidden-language scan for `*.py`, `*.js`, `*.ts`, `*.rb`, `*.go`, `*.java`, `*.scala`, `*.groovy`, `*.php`, and `*.fs` returned no files under `microservices/calendar`.
- `microservices/calendar/PRD.md:149` declares 44 Rust crates.
- `microservices/calendar/catalog/` contains YAML component catalog entries for Rust-shaped modules and adapters.
- No `microservices/calendar/src/` directory exists in the inventory.
- No Rust source files exist under the calendar path inventory.

Assessment:
- Calendar passes the forbidden-source-file extension scan.
- Calendar does not yet prove the Rust implementation because the microservice path contains plans and catalogs but no source.
- The strict-language finding is therefore positive for forbidden files and negative for implementation evidence.
- Proto and YAML contract files are authorized artifact families.
- Cedar policy files are authorized artifact families.

Judgment:
- Forbidden language scan: pass.
- Rust implementation evidence: missing.
- Severity: P2 for missing Rust source evidence, not P1, because this audit is document-focused and the stronger P1 is already captured under executable artifact completeness.

### §3.8 Dimension 8 - Operational Readiness, SLOs, Failure Modes, And Incident Response

Evidence:
- `microservices/calendar/slos/agenda-render-latency.openslo.yaml` exists.
- `microservices/calendar/slos/caldav-availability.openslo.yaml` exists.
- `microservices/calendar/slos/freebusy-query-latency.openslo.yaml` exists.
- `microservices/calendar/slos/ics-import-throughput.openslo.yaml` exists.
- `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml` exists.
- `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml` exists.
- `microservices/calendar/slos/rsvp-fanout-latency.openslo.yaml` exists.
- `microservices/calendar/slos/scheduling-convergence-latency.openslo.yaml` exists.
- `microservices/calendar/slos/tzdb-staleness-bound.openslo.yaml` exists.
- `microservices/calendar/failure-modes.md:17` says each failure mode has a runbook and an SLO or dashboard hook.
- `microservices/calendar/failure-modes.md:80-86` references `runbooks/invitation-dispatch-recovery.md`.
- `microservices/calendar/failure-modes.md:96-102` references `runbooks/audit-chain-emission-recovery.md`.
- `microservices/calendar/failure-modes.md:105-111` references `runbooks/caldav-pagination.md`.
- `microservices/calendar/failure-modes.md:113-119` references `runbooks/postgres-connection-storm.md`.
- `microservices/calendar/failure-modes.md:121-127` references `runbooks/cross-pack-mesh-degradation.md`.
- None of those five referenced runbooks appear in the inventory.
- `microservices/calendar/incident-response.md` is present.
- `microservices/calendar/dashboards/availability-and-freebusy.json` is present.
- `microservices/calendar/dashboards/ics-import-export.json` is present.
- `microservices/calendar/dashboards/scheduling-pipeline.json` is present.

Assessment:
- Operational coverage is much deeper than many early microservice doc sets.
- The runbook suite includes recurrence storms, cache rebuilds, CalDAV sync loops, restore, RSVP storms, shared calendar permission drift, timezone refresh, and rollback.
- The failure-mode document overclaims runbook closure for at least five named scenarios.
- This is a practical incident-readiness gap because the first responder would follow a broken path.

Judgment:
- SLO breadth: strong.
- Runbook breadth: strong.
- Failure-mode closure: incomplete.
- Severity: P1 for broken runbook references because they affect incident response execution.

### §3.9 Dimension 9 - Cross-Service Ownership, Compliance, Cost, And Migration Coherence

Evidence:
- `microservices/calendar/PRD.md:167-169` classifies calendar event metadata and cross-product participation.
- `microservices/calendar/PRD.md:188-197` lists events produced for workflow, mail, people, audit, search, room, and billing consumers.
- `microservices/calendar/PRD.md:203-207` lists events consumed from identity, tenancy, rooms, workflow, and mail services.
- `microservices/calendar/manifest.json:434-445` lists dependencies on mail, tenancy, identity, observability, audit-chain, network, intelligence, ontology, detection, cell, and cloud-iac.
- The inventory has no `microservices/calendar/cross-microservice-handoffs.md`.
- `microservices/calendar/compliance.md` is present.
- `microservices/calendar/dpia.md` is present.
- `microservices/calendar/threat-model.md` is present.
- `microservices/calendar/packs/` contains GDPR, HIPAA, KR-PIPA, SOC2, and EU-AI-Act docs.
- `microservices/calendar/migration-playbooks/from-google-calendar.md` is present.
- `microservices/calendar/migration-from-connect.md` is present.
- `microservices/calendar/cost-budget.md:18-44` defines unit economics and per-cell cost envelope.
- `microservices/calendar/cost-budget.md:48-53` uses Free, Starter, Pro, and Enterprise terms.
- `microservices/calendar/cost-budget.md:61-74` uses tier labels in billing metrics and budget keys.

Assessment:
- Calendar is clearly cross-service by design.
- The absence of a cross-microservice handoff file makes ownership seams hard to verify.
- Compliance and DPIA coverage is broad and service-specific.
- Cost modeling is substantive but tied to retired commercial vocabulary.
- Migration coverage is good but also includes retired class language in the Google migration playbook.

Judgment:
- Compliance depth: strong.
- Cost depth: moderate but vocabulary-stale.
- Cross-service handoff closure: weak.
- Migration depth: moderate.

## §4 Findings Table

| ID | Severity | Finding | Evidence | Required correction |
| --- | --- | --- | --- | --- |
| CAL-AUD-001 | P1 | Calendar lacks canonical six-context deployment evidence. | ADR requires context ids in manifests at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2079-2084`; calendar manifest dependencies appear at `microservices/calendar/manifest.json:434-445` but no context matrix appears; IP-001 targets Helm/Kustomize at `microservices/calendar/IP-001-iac-bootstrap.md:37-56`. | Add calendar-owned deployment-context matrix plus per-context OpenTofu modules or explicit N/A decisions. |
| CAL-AUD-002 | P1 | Calendar lacks canonical OpenTofu IaC modules. | OpenTofu is mandatory at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2243-2294`; IP-001 scopes to Helm/Kustomize at `microservices/calendar/IP-001-iac-bootstrap.md:16-26`; inventory contains only `iac/helm` and `iac/kustomize`. | Add OpenTofu modules under canonical context paths and keep Helm/Kustomize as deploy payload only. |
| CAL-AUD-003 | P1 | Calendar has no OCI Always Free profile module. | OCI context requires `iac/oci-guest/always-free` at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1834-1848`; OCI memory defines Always Free capacity at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_oci_always_free_maximization_2026_05_20.md:10-29`; inventory has no `iac/oci-guest/always-free/`. | Add demo-trial compatible OCI Always Free profile IaC and explicit capacity caps. |
| CAL-AUD-004 | P1 | Calendar lacks supported-OS manifest and OS CI evidence. | OS doctrine is in `docs/standards/brief-template.md:967-1123`; calendar PRD has CI gates at `microservices/calendar/PRD.md:171-183` but no OS matrix; inventory has no `supported-oses.json`. | Add `supported-oses.json`, package/runtime matrix, and CI mapping. |
| CAL-AUD-005 | P1 | PRD claims executable Rust crates and tests that are not present under the microservice path. | PRD declares 44 crates at `microservices/calendar/PRD.md:149`; acceptance criteria cite tests and benches at `microservices/calendar/PRD.md:285-300`; inventory has no `src/` or `tests/`. | Either land the source/test artifacts or reclassify these as future implementation commitments with owned issue links. |
| CAL-AUD-006 | P1 | OpenAPI references a CalDAV contract directory that is missing. | CalDAV pointer appears at `microservices/calendar/contracts/openapi/calendar.yaml:360-363`; inventory has no `contracts/caldav/`. | Add CalDAV contract details or change the pointer to an existing authoritative file. |
| CAL-AUD-007 | P1 | Failure-mode document references runbooks that are absent. | Runbook guarantee appears at `microservices/calendar/failure-modes.md:17`; missing references appear at `microservices/calendar/failure-modes.md:80-86`, `microservices/calendar/failure-modes.md:96-102`, `microservices/calendar/failure-modes.md:105-111`, `microservices/calendar/failure-modes.md:113-119`, and `microservices/calendar/failure-modes.md:121-127`. | Add the five runbooks or revise the failure-mode document to point at existing runbooks. |
| CAL-AUD-008 | P1 | Cross-service ownership handoffs are not closed despite many declared dependencies. | Produced and consumed event lists appear at `microservices/calendar/PRD.md:188-207`; dependency list appears at `microservices/calendar/manifest.json:434-445`; ownership directive expects handoffs in `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:18-46`; inventory has no `cross-microservice-handoffs.md`. | Add calendar handoff contract for mail, tenancy, identity, workflow, audit-chain, billing, rooms, and cloud-iac. |
| CAL-AUD-009 | P2 | Retired capability-tier language remains throughout calendar docs. | Direct retired-name lines are listed in §3.4.T; tier retirement directive is `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_capability_tracks_2026_05_20.md:10-45`. | Wave 15J should remove the customer capability ladder and replace it with tenant-class and billing-component language. |
| CAL-AUD-010 | P2 | Tenant-class semantics are absent. | Tenant-class scan found no matches; replacement model appears in `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:23-62`. | Add calendar entitlement/quota semantics for demo-trial, paid, and revenue-share handling after enum clarification. |
| CAL-AUD-011 | P2 | Cost budget uses retired commercial labels. | Cost labels appear at `microservices/calendar/cost-budget.md:48-53` and tier metrics at `microservices/calendar/cost-budget.md:61-74`; tier retirement directive appears at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_capability_tracks_2026_05_20.md:28-45`. | Rewrite cost model around tenant class, usage caps, paid scaling, and revenue-share economics. |
| CAL-AUD-012 | P2 | Architecture still advertises anchor-sweep stub debt. | Stub warning appears at `microservices/calendar/ARCHITECTURE.md:3`; deployment detail appears at `microservices/calendar/ARCHITECTURE.md:571-578`. | Replace sweep-generated caveats with final service-owned architecture sections. |
| CAL-AUD-013 | P2 | Architecture says cell eligibility is not declared in manifest. | Cell eligibility text appears at `microservices/calendar/ARCHITECTURE.md:321-332`; manifest dependency shape appears at `microservices/calendar/manifest.json:434-445`. | Add machine-readable cell/deployment eligibility in the calendar manifest. |
| CAL-AUD-014 | P2 | Current counterpart lineage is split across older and newer sources. | PRD competitor table appears at `microservices/calendar/PRD.md:227-247`; current batch counterpart set appears in chat at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16290-16311`. | Normalize current docs around Google Calendar, Microsoft Outlook Calendar, and Cal.com while preserving older references as secondary comparators. |
| CAL-AUD-015 | P2 | Capability YAML uses tenant-tier language even when T0/T1/T2 may be autonomy levels. | Generic tier vocabulary appears in `microservices/calendar/capabilities/T1-assist.yaml:35`, `microservices/calendar/capabilities/T1-assist.yaml:125`, `microservices/calendar/capabilities/T2-auto.yaml:63`, `microservices/calendar/capabilities/T2-auto.yaml:97`, `microservices/calendar/capabilities/T2-auto.yaml:114`, `microservices/calendar/capabilities/T2-auto.yaml:136`, and `microservices/calendar/capabilities/T2-auto.yaml:141`. | Preserve autonomy-level files if needed, but remove customer tier vocabulary and map controls to tenant class or policy entitlements. |
| CAL-AUD-016 | P2 | Calendar compliance prose mentions OpenTofu without local OpenTofu evidence. | `microservices/calendar/compliance.md:1110` includes OpenTofu in dependency inventory; actual inventory has no OpenTofu module files; OpenTofu doctrine appears at `docs/standards/brief-template.md:809-966`. | Replace the prose-only mention with actual OpenTofu module evidence. |
| CAL-AUD-017 | P2 | Benchmark doc is stale after tier retirement. | Legacy benchmark tier lines are listed in §3.4.T; current performance methodology is delivered in `microservices/calendar/performance-benchmark-numbers-2026-05-20.md`. | Retire or rewrite `benchmarks/gcal-outlook-calendly-vs-oyatie.md` to use single industry-leader targets plus deployment and tenant overlays. |
| CAL-AUD-018 | P3 | Calendar lacks a root README. | Inventory contains `decisions/README.md` but no `microservices/calendar/README.md`; core product purpose is scattered across `microservices/calendar/PRD.md:20-35` and `microservices/calendar/ARCHITECTURE.md:40-46`. | Add a concise root README after the canonical machine-readable gaps are resolved. |
| CAL-AUD-019 | P3 | SLO and dashboard linkage should be mechanically checked after runbook closure. | SLO files exist in inventory; dashboards exist in inventory; failure-mode promise appears at `microservices/calendar/failure-modes.md:17`. | Add a small manifest mapping SLOs, dashboards, alerts, and runbooks once missing runbooks are added. |
| CAL-AUD-020 | P3 | Previous chat history shows tier-era calendar doc work that should not be treated as current doctrine. | Calendar tier-era doc completion appears at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:11578-12275`; no-tier correction appears at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:15947-16020`. | Treat those older docs as migration input only. |
| CAL-AUD-021 | P3 | The service has strong domain-specific operational docs worth preserving during cleanup. | Runbook inventory lines in §2 and SLO inventory lines in §3.8 show substantial coverage; bespoke-substance requirement appears at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md:10-20`. | Refactor vocabulary and evidence gaps without flattening the calendar-specific operational content. |
| CAL-AUD-022 | P3 | Tenant-class naming has a canonical ambiguity to resolve before machine-readable migration. | Current batch names three tenant-class terms; tenant memory defines two tenant classes and revenue share as a billing component at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:10-62`. | Resolve the enum centrally, then apply to calendar once. |

Finding counts:
- P0: 0.
- P1: 8.
- P2: 9.
- P3: 5.

## §5 Open Questions

1. Should `revenue_share` be encoded as a tenant class for calendar, as this batch directive says, or as a billing component under paid tenants, as the later memory file says at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:10-62`?
2. Should the `capabilities/T0-suggest.yaml`, `capabilities/T1-assist.yaml`, and `capabilities/T2-auto.yaml` files remain as autonomy-level capability specs after customer capability tiers retire, or should those files be renamed to avoid the word `tier` entirely?
3. Should CalDAV conformance live in `contracts/caldav/` as the OpenAPI file implies at `microservices/calendar/contracts/openapi/calendar.yaml:360-363`, or should it live under an ADR plus protocol test manifest?
4. Should the calendar source tree live directly under `microservices/calendar/src/`, or should the catalog entries point to a shared Rust workspace path outside this microservice directory?
5. Should `calendar` own external calendar connector migration from Google Calendar and Outlook Calendar, or should connector import/export be split with a separate integration microservice?
6. Should `calendar` own resource-room truth, or should it own only booking transactions while `rooms` owns inventory and constraints?
7. Should on-prem and colo be default-supported for calendar now, or should they receive explicit N/A decisions until CalDAV, mail-loop, and timezone-update controls are facility-tested?
8. Should the OCI Always Free profile run the full calendar dependency stack in a single compact cell, or should demo-trial tenants use shared managed calendar substrate with hard usage caps?
9. Should the old benchmark document be retired after this batch's performance benchmark document lands, or should it be rewritten in place to preserve historical comparator notes?
10. Should the cross-microservice handoff file be authored before OpenTofu work, because cloud-iac, mail, tenancy, and identity dependencies shape the context modules?

<!-- ORCHESTRATOR REPORT
  µservice: calendar
  deliverables_landed:
    - microservices/calendar/coherence-audit-2026-05-20.md: 625 lines
    - microservices/calendar/feature-parity-matrix-2026-05-20.md: 435 lines
    - microservices/calendar/performance-benchmark-numbers-2026-05-20.md: 397 lines
  inventory_files_seen: 139
  inventory_lines_read: 26495
  chat_history_matches_processed: 190 case-sensitive matches scanned; 216 case-insensitive matches observed
  findings_p0: 0
  findings_p1: 8
  findings_p2: 9
  findings_p3: 5
  tier_retirement_candidates_found: 57 direct retired-name citation lines listed in section 3.4.T; 64 raw rg lines included 7 false-positive baseline/baseline-test/baseline-dashboard matches
  tenant_class_adoption_gaps: yes; no tenant_class, demo_trial, revenue_share, per_seat, or per_usage matches were found in calendar artifacts
  top_3_counterparts_confirmed: Google Calendar / Microsoft Outlook Calendar / Cal.com
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1457
-->
