# tasks feature-parity matrix - 2026-05-20

Audit owner: solo Codex audit lane for `microservices/tasks`.
Target microservice: `tasks`.
Counterpart union bar: Linear, Jira Software, Asana.
Purpose: compare documented tasks product scope against the union of the three named counterpart surfaces.
Method: classify each capability as documented intent, contract evidence, operational evidence, implementation evidence, or missing evidence.
No implementation claim is made from documentation alone.
No retired capability ladder is used in this matrix.
Tenant-class note: quality expectations are uniform for `demo_trial` and `paid`; `revenue_share` is a paid billing component, not a tenant_class.

Local anchor 1: `microservices/tasks/PRD.md:22` defines the tasks product surface.
Local anchor 2: `microservices/tasks/PRD.md:40` through `microservices/tasks/PRD.md:63` define functional requirements.
Local anchor 3: `microservices/tasks/competitor-parity-matrix.md:34` includes Asana.
Local anchor 4: `microservices/tasks/competitor-parity-matrix.md:36` includes Linear.
Local anchor 5: `microservices/tasks/competitor-parity-matrix.md:37` includes Jira.
Local anchor 6: `microservices/tasks/contracts/tasks.openapi.yaml:1` provides OpenAPI contract evidence.
Local anchor 7: `microservices/tasks/contracts/tasks.asyncapi.yaml:1` provides AsyncAPI event evidence.
Local anchor 8: `microservices/tasks/contracts/tasks.proto:1` provides proto contract evidence.
Local anchor 9: `microservices/tasks/capacity-model.md:37` through `microservices/tasks/capacity-model.md:49` provides capacity target evidence.
Local anchor 10: `microservices/tasks/slos/task-create.openslo.yaml` provides task-create SLO evidence.

External anchor 1: Linear developer documentation and API references, especially rate limits and pagination, are public at `https://linear.app/developers/rate-limiting` and `https://linear.app/docs/api/pagination`.
External anchor 2: Jira Cloud platform rate-limit and guardrail documentation is public at `https://developer.atlassian.com/cloud/jira/platform/rate-limiting/` and `https://support.atlassian.com/jira-cloud-administration/docs/data-limits-and-guardrails/`.
External anchor 3: Asana developer rate-limit documentation is public at `https://developers.asana.com/docs/rate-limits`.

## Counterpart 1 - Linear capability surface

Linear pressure 1: issue-first work tracking with low interaction latency.
Linear pressure 2: cycles as a first-class planning primitive.
Linear pressure 3: roadmaps and initiatives for higher-level planning.
Linear pressure 4: teams with scoped workflows and conventions.
Linear pressure 5: projects tied to milestones and progress.
Linear pressure 6: labels and metadata optimized for fast triage.
Linear pressure 7: assignee ownership and subscriber/watch behavior.
Linear pressure 8: comments and activity timelines.
Linear pressure 9: attachments and linked resources.
Linear pressure 10: keyboard-heavy workflow and command palette behavior.
Linear pressure 11: fast issue search and filter views.
Linear pressure 12: backlog, active, and done status flow.
Linear pressure 13: automation and integrations for engineering work.
Linear pressure 14: GitHub and developer-tool integration.
Linear pressure 15: API coverage with GraphQL-like query patterns.
Linear pressure 16: webhook and event integrations.
Linear pressure 17: importers from Jira and other tools.
Linear pressure 18: cycle analytics and delivery health.
Linear pressure 19: fine-grained notification preferences.
Linear pressure 20: deterministic issue identifiers.
Linear pressure 21: relationship modelling such as blocking, duplicate, and related work.
Linear pressure 22: organization-level settings with team-level autonomy.
Linear pressure 23: audit expectations for enterprise customers.
Linear pressure 24: speed and density as product identity.
Linear pressure 25: developer-friendly API rate-limit behavior.
Oyatie documented match: core task CRUD appears in `microservices/tasks/PRD.md:40`.
Oyatie documented match: projects and views appear in `microservices/tasks/PRD.md:43`.
Oyatie documented match: dependencies appear in `microservices/tasks/PRD.md:45`.
Oyatie documented match: comments and attachments appear in `microservices/tasks/PRD.md:46`.
Oyatie documented match: sprint and iteration concepts appear in `microservices/tasks/PRD.md:51`.
Oyatie documented match: workflow and automation appear in `microservices/tasks/PRD.md:53`.
Oyatie documented match: notifications appear in `microservices/tasks/PRD.md:54`.
Oyatie documented match: search appears in `microservices/tasks/PRD.md:56`.
Oyatie documented match: import/export appears in `microservices/tasks/PRD.md:57`.
Oyatie documented match: APIs and webhooks appear in `microservices/tasks/PRD.md:58`.
Oyatie documented match: roadmap and portfolio appear in `microservices/tasks/PRD.md:61`.
Oyatie contract match: dependency events appear in `microservices/tasks/contracts/tasks.asyncapi.yaml:67`.
Oyatie operational match: task-list render has an OpenSLO in `microservices/tasks/slos/task-list-render.openslo.yaml`.
Oyatie operational match: search freshness has an OpenSLO in `microservices/tasks/slos/search-freshness.openslo.yaml`.
Oyatie gap: no source tree proves Linear-like issue creation, update, or cycle execution.
Oyatie gap: no command palette or dense UI implementation evidence exists under tasks.
Oyatie gap: Linear-style cycles are not clearly separated from Jira-style sprints.
Oyatie gap: developer-tool integration is mentioned through imports and APIs, but no GitHub integration implementation is present.
Oyatie gap: no GraphQL-like query surface is evidenced; local contract evidence is OpenAPI/proto/AsyncAPI.
Oyatie gap: roadmaps and initiatives need a boundary decision with product and portfolio services.
Linear parity conclusion: documented ambition is strong; implementation parity is not yet evidenced.

## Counterpart 2 - Jira Software capability surface

Jira pressure 1: issue type schemes.
Jira pressure 2: configurable workflows.
Jira pressure 3: workflow transitions and validators.
Jira pressure 4: permission schemes.
Jira pressure 5: notification schemes.
Jira pressure 6: custom fields.
Jira pressure 7: screens and field contexts.
Jira pressure 8: projects with company-managed and team-managed variants.
Jira pressure 9: boards and board filters.
Jira pressure 10: Scrum sprints.
Jira pressure 11: Kanban flow.
Jira pressure 12: backlog refinement.
Jira pressure 13: releases and versions.
Jira pressure 14: epics and higher-order planning.
Jira pressure 15: JQL-style advanced search.
Jira pressure 16: bulk edits.
Jira pressure 17: imports from CSV and other trackers.
Jira pressure 18: audit logs.
Jira pressure 19: marketplace integrations.
Jira pressure 20: automation rules.
Jira pressure 21: SLA and service-management adjacency.
Jira pressure 22: data residency controls.
Jira pressure 23: enterprise compliance exports.
Jira pressure 24: API pagination and rate-limit handling.
Jira pressure 25: scale guardrails for large tenants.
Oyatie documented match: custom fields appear in `microservices/tasks/PRD.md:48`.
Oyatie documented match: sprint and iteration support appears in `microservices/tasks/PRD.md:51`.
Oyatie documented match: workflow and automation support appears in `microservices/tasks/PRD.md:53`.
Oyatie documented match: bulk edit appears in `microservices/tasks/PRD.md:55`.
Oyatie documented match: imports and exports appear in `microservices/tasks/PRD.md:57`.
Oyatie documented match: audit and compliance appear in `microservices/tasks/PRD.md:91` through `microservices/tasks/PRD.md:101`.
Oyatie contract match: proto contains custom field and dependency structures around `microservices/tasks/contracts/tasks.proto:250`.
Oyatie contract match: proto contains import structures around `microservices/tasks/contracts/tasks.proto:529`.
Oyatie operational match: bulk-update SLO exists in `microservices/tasks/slos/bulk-update.openslo.yaml`.
Oyatie operational match: dependency-cycle SLO exists in `microservices/tasks/slos/dependency-cycle-check.openslo.yaml`.
Oyatie operational match: capacity model includes bulk edits at `microservices/tasks/capacity-model.md:44`.
Oyatie compliance match: DPIA covers processing purpose and risks in `microservices/tasks/dpia.md:84`.
Oyatie gap: no implemented workflow transition engine exists under `src/`.
Oyatie gap: no Jira-equivalent JQL parser or saved-query engine exists under `src/`.
Oyatie gap: no permission-scheme implementation exists under `src/`.
Oyatie gap: no marketplace app model is evidenced in tasks-specific code.
Oyatie gap: no deployment-context OpenTofu modules exist for enterprise install paths.
Oyatie gap: no supported OS artifact proves Jira-like enterprise install breadth.
Jira parity conclusion: enterprise feature intent is well represented; enterprise implementation and deployment evidence are missing.

## Counterpart 3 - Asana capability surface

Asana pressure 1: project and task management for technical and non-technical teams.
Asana pressure 2: task assignees, collaborators, due dates, and sections.
Asana pressure 3: project views such as list, board, timeline, and calendar.
Asana pressure 4: portfolio rollups.
Asana pressure 5: goals and goal progress.
Asana pressure 6: forms for intake.
Asana pressure 7: rules for automation.
Asana pressure 8: templates for repeatable work.
Asana pressure 9: approvals.
Asana pressure 10: workload and capacity planning.
Asana pressure 11: dependencies and critical-path style planning.
Asana pressure 12: comments and attachments.
Asana pressure 13: file and calendar integrations.
Asana pressure 14: custom fields and reporting.
Asana pressure 15: saved searches and dashboards.
Asana pressure 16: importers and migration flows.
Asana pressure 17: mobile and web user experience.
Asana pressure 18: admin controls and data export.
Asana pressure 19: API rate-limit and concurrent-request behavior.
Asana pressure 20: enterprise compliance and retention.
Asana pressure 21: multi-department taxonomy.
Asana pressure 22: task templates and recurring work.
Asana pressure 23: project status updates.
Asana pressure 24: inbox and notification ergonomics.
Asana pressure 25: onboarding workflows for broad adoption.
Oyatie documented match: project views appear in `microservices/tasks/PRD.md:43`.
Oyatie documented match: recurrence appears in `microservices/tasks/PRD.md:44`.
Oyatie documented match: dependencies appear in `microservices/tasks/PRD.md:45`.
Oyatie documented match: comments and attachments appear in `microservices/tasks/PRD.md:46`.
Oyatie documented match: templates appear in `microservices/tasks/PRD.md:50`.
Oyatie documented match: calendar bridge appears in `microservices/tasks/PRD.md:60`.
Oyatie documented match: roadmap and portfolio appear in `microservices/tasks/PRD.md:61`.
Oyatie documented match: AI-generated tasks and summarization appear in `microservices/tasks/PRD.md:62`.
Oyatie tutorial match: Asana migration tutorial exists at `microservices/tasks/tutorials/migrate-asana-project.md:1`.
Oyatie journey match: portfolio rollup implementation plan exists at `microservices/tasks/IP-journey-j98-portfolio-rollup.md`.
Oyatie journey match: project template import plan exists at `microservices/tasks/IP-journey-j91-project-template-import.md`.
Oyatie journey match: recurring task automation plan exists at `microservices/tasks/IP-journey-j92-recurring-task-automation.md`.
Oyatie operational match: recurring task generation SLO exists in `microservices/tasks/slos/recurring-task-generation.openslo.yaml`.
Oyatie gap: no forms/intake implementation evidence exists.
Oyatie gap: no goals implementation evidence exists.
Oyatie gap: no workload/capacity UI implementation evidence exists.
Oyatie gap: no mobile frontend evidence exists in tasks.
Oyatie gap: no Swift, Kotlin, WinUI3, or Leptos task UI implementation exists under tasks.
Oyatie gap: Asana migration docs required tenant_class cleanup at `microservices/tasks/tutorials/migrate-asana-project.md:15`.
Asana parity conclusion: broad work-management intent is present, but non-engineering adoption features need stronger implementation and tenant_class adoption.

## UNION-coverage matrix

01. Core task create/read/update/delete: documented in `PRD.md:40`; contract-backed by OpenAPI; implementation evidence absent.
02. Task identity and deterministic keys: documented by ADR-TASKS-0002; implementation evidence absent.
03. Task title and body fields: documented in OpenAPI; implementation evidence absent.
04. Status lifecycle: documented in PRD and catalog `tasks_status_transition.yaml`; implementation evidence absent.
05. Priority: documented in task contract fields; implementation evidence absent.
06. Due dates: documented in PRD task-management requirements; implementation evidence absent.
07. Assignee: documented in PRD and contracts; implementation evidence absent.
08. Multi-assignee handoff: documented by `IP-journey-j94-multi-assignee-handoff.md`; implementation evidence absent.
09. Watchers/subscribers: documented in PRD notification requirements; implementation evidence absent.
10. Comments: documented in `PRD.md:46` and `catalog/tasks_comment.yaml`; implementation evidence absent.
11. Attachments: documented in `PRD.md:46` and `catalog/tasks_attachment.yaml`; implementation evidence absent.
12. Checklists: documented in `PRD.md:46`; catalog coverage not clearly isolated.
13. Projects: documented in `PRD.md:43` and `catalog/tasks_project.yaml`; implementation evidence absent.
14. Project templates: documented in `PRD.md:50` and `IP-journey-j91-project-template-import.md`; implementation evidence absent.
15. Saved views: documented in `PRD.md:49` and `catalog/tasks_saved_view.yaml`; implementation evidence absent.
16. List view: documented intent; UI implementation evidence absent.
17. Board view: documented intent; UI implementation evidence absent.
18. Calendar view: documented in `PRD.md:60`; UI implementation evidence absent.
19. Timeline view: Asana union need; not clearly evidenced beyond roadmap/portfolio wording.
20. Roadmap view: documented in `PRD.md:61`; implementation evidence absent.
21. Portfolio rollup: documented in `PRD.md:61` and `IP-journey-j98-portfolio-rollup.md`; implementation evidence absent.
22. Goals: Asana union need; not clearly owned by tasks.
23. Initiatives: Linear/Jira union need; ownership with portfolio/product service unresolved.
24. Epics: Jira union need; tasks equivalent not explicitly decided.
25. Versions/releases: Jira union need; local evidence not strong.
26. Cycles: Linear union need; local sprint/iteration docs may cover partly.
27. Sprints: documented in `PRD.md:51` and `IP-journey-j93-sprint-planning.md`; implementation evidence absent.
28. Kanban flow: documented indirectly via boards/statuses; implementation evidence absent.
29. Backlog: documented indirectly via sprint planning; implementation evidence absent.
30. Dependencies: documented in `PRD.md:45`, AsyncAPI dependency events, and dependency SLO; implementation evidence absent.
31. Cycle detection: documented by `slos/dependency-cycle-check.openslo.yaml` and `IP-journey-j97-dependency-cycle-resolution.md`; implementation evidence absent.
32. Blocking relationships: documented via dependency edge catalog; implementation evidence absent.
33. Duplicate relationships: Linear/Jira union need; not clearly evidenced.
34. Related-work relationships: Linear/Jira union need; not clearly evidenced.
35. Recurrence: documented in `PRD.md:44`, ADR-TASKS-0003, and recurrence SLO; implementation evidence absent.
36. Automation rules: documented in `PRD.md:53` and `IP-010-automation-and-template-engine.md`; implementation evidence absent.
37. Forms/intake: Asana union need; not clearly evidenced.
38. Approvals: Asana union need; not clearly evidenced.
39. Custom fields: documented in `PRD.md:48` and proto structures; implementation evidence absent.
40. Field contexts: Jira union need; not clearly evidenced.
41. Screen schemes: Jira union need; not clearly evidenced.
42. Workflow validators: Jira union need; not clearly evidenced.
43. Permission schemes: Jira union need; architecture references Cedar gates, but no implementation evidence.
44. Notification schemes: Jira union need; PRD notifications present, scheme-level evidence incomplete.
45. Inbox behavior: Asana union need; local notification preference catalog partially covers.
46. Webhooks: documented in `PRD.md:58`, AsyncAPI, catalog subscription, and webhook SLO; implementation evidence absent.
47. API public surface: documented by OpenAPI and proto; implementation evidence absent.
48. GraphQL-style query: Linear union need; no local evidence.
49. JQL-style query: Jira union need; no local evidence.
50. Saved filters: documented in `PRD.md:49`; implementation evidence absent.
51. Full-text search: documented in `PRD.md:56`, search catalog, and search SLO; implementation evidence absent.
52. Activity index: documented by `catalog/tasks_activity_index.yaml`; implementation evidence absent.
53. Audit projection: documented by `catalog/tasks_audit_projection.yaml`; manifest audit-chain is narrow.
54. Bulk edit: documented in `PRD.md:55`, capacity model, and SLO; implementation evidence absent.
55. Bulk operation queue: documented by `catalog/tasks_bulk_operation.yaml`; implementation evidence absent.
56. Import from Jira: documented by migration playbook and PRD; implementation evidence absent.
57. Import from Asana: documented by tutorial and PRD; implementation evidence absent.
58. Import idempotency: Rust reference implementation exists; production crate evidence absent.
59. Export: documented in PRD and contracts; implementation evidence absent.
60. Compliance export: documented by journey J100 and compliance docs; implementation evidence absent.
61. Legal hold: documented by journey J100; implementation evidence absent.
62. Data residency: documented by policy and multi-region docs; deployment context evidence absent.
63. Retention/deletion: documented by policy files; implementation evidence absent.
64. Encryption boundaries: documented by policy; implementation evidence absent.
65. BYOK: paid tenant_class allows it by current directive; tasks docs do not express tenant_class semantics.
66. Compliance packs: paid tenant_class allows them by current directive; tasks docs do not express tenant_class semantics.
67. Best-effort demo caps: current directive requires demo_trial caps; tasks docs now carry tenant_class scrub markers.
68. Usage-based scaling: current directive requires paid usage scaling; tasks docs now carry billing-component scrub markers.
69. Revenue-share model: current directive requires `revenue_share` as a paid billing component; tasks docs need billing-component adoption.
70. Portfolio analytics: documented in PRD and journey plan; implementation evidence absent.
71. Workload capacity: Asana union need; local capacity model is service capacity, not user workload UI.
72. Project status updates: Asana union need; not clearly evidenced.
73. Goal progress: Asana union need; not clearly evidenced.
74. Initiative rollups: Linear/Jira union need; partially covered by portfolio wording.
75. Command palette: Linear union need; no local evidence.
76. Keyboard-first density: Linear union need; no local UI implementation evidence.
77. Mobile UI: Asana union need; no local mobile implementation evidence.
78. Web SSR/islands UI: canonical frontend path; no local Leptos implementation evidence.
79. Swift frontend: canonical allowlist; no local Swift implementation evidence.
80. Kotlin frontend: canonical allowlist; no local Kotlin implementation evidence.
81. WinUI3 frontend: canonical allowlist; no local WinUI3 implementation evidence.
82. Admin controls: Jira/Asana union need; compliance docs partial, implementation absent.
83. Organization settings: Linear/Jira union need; not strongly evidenced.
84. Team settings: Linear union need; partially implied by projects, not implemented.
85. Marketplace integrations: Jira union need; SDK plan exists, marketplace model not evidenced.
86. Developer integrations: Linear union need; webhooks/API docs exist, concrete integration absent.
87. Calendar integration: documented in `PRD.md:60`; implementation evidence absent.
88. File integration: Asana union need; attachments exist, external file bridge not clearly evidenced.
89. Notification preferences: catalog exists; implementation evidence absent.
90. Webhook fanout operations: runbook and SLO exist; implementation evidence absent.
91. Search lag operations: runbook and SLO exist; implementation evidence absent.
92. Import stalled operations: runbook exists; implementation evidence absent.
93. Recurrence lag operations: runbook and SLO exist; implementation evidence absent.
94. Dependency-cycle operations: runbook and SLO exist; implementation evidence absent.
95. Auto-assign review queue: runbook and SLO exist; implementation evidence absent.
96. Autonomy suggestion: capability JSON exists; implementation evidence absent.
97. Autonomy assist: capability JSON exists; implementation evidence absent.
98. Autonomy auto: capability JSON exists; implementation evidence absent.
99. AI summarization: documented in `PRD.md:62`; implementation evidence absent.
100. AI risk controls: documented in DPIA and safety policy; implementation evidence absent.
101. Tenant isolation: architecture and threat model discuss it; tenant_class absent.
102. Context deployment: canonical requirement; local OpenTofu context modules absent.
103. OCI Always Free profile: canonical requirement; local module absent.
104. On-prem deployment: canonical requirement; local module absent.
105. Colo deployment: canonical requirement; local module absent.
106. AWS guest deployment: canonical requirement; local module absent.
107. Public cloud deployment: canonical requirement; local module absent.
108. Oyatie provider deployment: canonical requirement; local module absent.
109. Supported OS matrix: canonical requirement; local artifact absent.
110. Rust backend: intended by canonical policy; local source tree absent.
111. Forbidden language avoidance: satisfied in source-file inventory.
112. Test set: required by implementation plans; local test tree absent.
113. Load-test evidence: required by performance claims; local load tests absent.
114. Contract parity tests: needed for OpenAPI/AsyncAPI/proto; local tests absent.
115. SLO burn alerts: runbooks and dashboards exist; runtime evidence absent.
116. Dashboards: JSON dashboards exist; deployment evidence absent.
117. Prometheus rules: Helm template exists; OpenTofu observability wiring absent.
118. Tenant billing overlay: cost-budget exists but uses old model.
119. Migration docs: present for Jira and Asana; contain retired wording.
120. Engineer onboarding: present; must be updated once implementation surface exists.

## Family summary

Core task model: strong documentation and contract intent, no implementation evidence.
Projects and views: strong PRD intent, partial catalog evidence, no UI implementation evidence.
Dependencies and recurrence: strong ADR/SLO/runbook intent, no implementation evidence.
Comments, attachments, and checklists: good PRD/catalog intent, no implementation evidence.
Custom fields: good PRD/proto intent, no implementation evidence.
Sprints, cycles, and planning: strong Jira/Linear ambition, ambiguous boundary between cycles, sprints, epics, initiatives, and portfolio.
Roadmaps and portfolios: strong PRD ambition, unresolved ownership with broader product and portfolio services.
Search and saved views: good catalog/SLO/runbook intent, no implementation evidence.
Bulk edit: good PRD/capacity/SLO/runbook intent, no implementation evidence.
Import/export: strong migration docs and Rust reference pattern, no production crate evidence.
Webhooks and APIs: strong contract intent, no runtime evidence.
Automation and templates: strong PRD/IP intent, no implementation evidence.
Notifications and watchers: documented intent, partial catalog evidence, no implementation evidence.
Compliance and audit: extensive docs, manifest audit-chain narrower than PRD event surface.
Security and privacy: strong policy, DPIA, threat-model intent; build and deployment evidence absent.
Tenant semantics: Wave 15J scrub introduces tenant_class and billing-component vocabulary; runtime semantics remain unimplemented.
Deployment contexts: canonical six-context requirements not met.
OpenTofu: canonical substrate absent.
OS support: canonical artifact absent.
Rust-strict source policy: no forbidden languages found; buildable Rust service absent.

## Headline gap analysis

Gap 1: Product breadth is ahead of implementation evidence.
Gap 1 evidence: `PRD.md:22` defines a large product surface, while no `src/` or `tests/` directory exists.
Gap 1 impact: parity claims must be framed as target intent, not shipped capability.
Gap 1 remedy: land a minimal Rust workspace and prove task CRUD, events, and SLO instrumentation first.
Gap 2: Linear-like speed and density are targets, not proven behavior.
Gap 2 evidence: local benchmark estimates exist in `benchmarks/tasks-vs-asana-jira-linear-monday.md:18`, but the benchmark uses retired assumptions.
Gap 2 impact: the service cannot claim Linear parity until UI and API latency are measured.
Gap 2 remedy: replace historical benchmark docs with a single target set and load-test harness.
Gap 3: Jira-like enterprise configurability is documented but not bounded.
Gap 3 evidence: PRD covers workflows, custom fields, sprints, and audit, but no source tree implements schemes or validators.
Gap 3 impact: uncontrolled configurability could overload the tasks service boundary.
Gap 3 remedy: decide which workflow primitives belong in tasks versus workflow service.
Gap 4: Asana-like broad-work adoption features are incomplete.
Gap 4 evidence: forms, goals, workload, and mobile UI are not strongly evidenced in local files.
Gap 4 impact: non-engineering teams may get less coverage than the top-3 union requires.
Gap 4 remedy: add explicit ownership decisions for forms, goals, workload, and portfolio.
Gap 5: Current docs needed alignment with the tier-retirement directive.
Gap 5 evidence: 167 retired-word references are listed in `coherence-audit-2026-05-20.md`.
Gap 5 impact: future implementation could recreate retired feature stratification.
Gap 5 remedy: Wave 15J cleanup replaces retired vocabulary with tenant_class, usage cap, billing component, or deployment-context language.
Gap 6: tenant_class runtime semantics are absent.
Gap 6 evidence: tenant_class documentation exists, but runtime event and billing implementations are absent.
Gap 6 impact: billing, onboarding, demo caps, and revenue-share behavior cannot be specified cleanly.
Gap 6 remedy: update cost, runbook, benchmark, migration, and onboarding docs.
Gap 7: canonical deployment is missing.
Gap 7 evidence: only Helm/Kustomize files exist under `iac/`.
Gap 7 impact: six deployment contexts cannot be audited or provisioned.
Gap 7 remedy: add per-context OpenTofu modules under the canonical path names.
Gap 8: OS support is not expressed.
Gap 8 evidence: no `supported-oses.json` exists.
Gap 8 impact: install/deploy claims can drift into generic Linux assumptions.
Gap 8 remedy: add the service-level OS support artifact.

## Additive surface recommended for Oyatie

Additive 1: keep Linear-like speed as a product invariant, not a paid-only feature.
Additive 2: keep Jira-like workflow configurability behind explicit workflow-service boundaries.
Additive 3: keep Asana-like broad-work surfaces visible in the first product architecture, even if implementation is sliced.
Additive 4: add tenant_class metadata to task import, export, SLO, capacity, and billing documents.
Additive 5: add an import capability model for Jira and Asana with measured rate-limit backoff.
Additive 6: add a Linear importer only after issue/cycle/roadmap boundary decisions are made.
Additive 7: add a command/search interaction spec before UI work begins.
Additive 8: add a saved-view query model that can later support Jira-style advanced search without copying Jira semantics wholesale.
Additive 9: add cycle/sprint vocabulary mapping so Linear cycles and Jira sprints do not become contradictory concepts.
Additive 10: add portfolio ownership rules so Asana portfolios and Linear roadmaps do not duplicate product-service state.
Additive 11: add forms/intake ownership decision for Asana parity.
Additive 12: add approvals ownership decision for Asana parity.
Additive 13: add workload/capacity ownership decision for Asana parity.
Additive 14: add legal-hold and compliance export event coverage to AsyncAPI and manifest audit_chain.
Additive 15: add source-generated event parity tests across OpenAPI, AsyncAPI, proto, and manifest.
Additive 16: add OpenTofu modules before deployment readiness is claimed.
Additive 17: add supported OS matrix before on-prem or colo readiness is claimed.
Additive 18: add Rust workspace before any feature parity is represented as shipped.
Additive 19: add benchmark harnesses that measure p50, p95, p99, throughput, fanout, import backoff, and cycle-check behavior.
Additive 20: keep all tenant classes on the same feature-quality bar and express only caps or billing overlays where constraints differ.

## Boundary decisions needed before implementation

Decision 1: decide whether cycles and sprints are one internal planning primitive or separate Linear-style and Jira-style projections.
Decision 1 evidence: `microservices/tasks/PRD.md:51` names sprint and iteration support, while Linear parity pressure requires cycles.
Decision 2: decide whether epics, initiatives, goals, and portfolios are tasks-native entities or references to higher-level planning services.
Decision 2 evidence: `microservices/tasks/PRD.md:61` names roadmap and portfolio, while Asana and Linear counterparts each use different hierarchy language.
Decision 3: decide whether forms and intake belong in tasks or in a workflow/forms service that creates tasks through contracts.
Decision 3 evidence: Asana parity requires intake forms, but local tasks contracts currently emphasize task/project/import resources.
Decision 4: decide whether Jira-style workflow validators are local state-machine rules or calls to the broader workflow microservice.
Decision 4 evidence: `microservices/tasks/PRD.md:53` names workflow and automation, while `microservices/tasks/PRD.md:187` warns against direct product-service imports.
Decision 5: decide whether advanced search should mimic JQL syntax or expose an Oyatie-native saved-view query language.
Decision 5 evidence: `microservices/tasks/PRD.md:49` and `microservices/tasks/PRD.md:56` cover filters and search, but no parser implementation exists.
Decision 6: decide whether marketplace integrations are a tasks API concern, a developer-platform concern, or both.
Decision 6 evidence: Jira parity pressure includes marketplace depth, while local tasks evidence is limited to API, webhook, and SDK planning.
Decision 7: decide how tenant_class metadata appears in task events.
Decision 7 evidence: no exact `tenant_class` strings exist under tasks, but billing and admission behavior require the metadata.
Decision 8: decide how deployment_context metadata appears in performance tests and dashboards.
Decision 8 evidence: canonical direction requires six deployment contexts, while current tasks IaC has only Helm and Kustomize packaging.
Decision 9: decide whether importers are standalone crates or adapters inside a smaller import subsystem.
Decision 9 evidence: `microservices/tasks/reference-implementations/rust/importer-idempotency.rs` proves one narrow importer pattern only.
Decision 10: decide whether audit-chain events are declared from AsyncAPI, proto, manifest, or a generated registry.
Decision 10 evidence: `microservices/tasks/manifest.json:343` through `microservices/tasks/manifest.json:350` is narrower than the PRD event surface.
Decision 11: decide whether task templates are purely tasks-native or shared with project and workflow services.
Decision 11 evidence: `microservices/tasks/PRD.md:50` names templates and `IP-journey-j91-project-template-import.md` covers project template import.
Decision 12: decide whether workload/capacity planning is a user-facing Asana-parity feature or an operational capacity model only.
Decision 12 evidence: `microservices/tasks/capacity-model.md:37` through `microservices/tasks/capacity-model.md:49` is service capacity, not user workload UI.
Decision 13: decide whether admin permission schemes are represented as Cedar policy resources or tasks-specific role tables.
Decision 13 evidence: `microservices/tasks/ARCHITECTURE.md:71` through `microservices/tasks/ARCHITECTURE.md:82` references Cedar gates, but no implementation exists.
Decision 14: decide whether recurrence execution is event-sourced, scheduled-job based, or hybrid.
Decision 14 evidence: ADR-TASKS-0003 and the recurrence SLO exist, but no runtime implementation exists.
Decision 15: decide whether customer-visible import progress is part of tasks or a shared migration service UI.
Decision 15 evidence: Jira and Asana migration docs exist under tasks, but source and frontend evidence are absent.

## Matrix conclusion

The tasks microservice has a product vision broad enough to challenge Linear, Jira Software, and Asana as a union bar.
The existing documents already identify most of the major counterpart feature families.
The current tree does not prove shipped feature parity because it lacks source, tests, OpenTofu context modules, and OS support artifacts.
The highest-value next step is not another feature list.
The highest-value next step is a small Rust workspace plus contract/event parity tests and the canonical deployment/OS substrate.
The tier-retirement cleanup must stay complete before new benchmark and migration docs are treated as authoritative.
The tenant_class model is introduced in documentation before billing, demo caps, revenue-share usage, and customer onboarding are implemented.
