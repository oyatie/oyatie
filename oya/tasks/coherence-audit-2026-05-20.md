# tasks ownership-coherence audit - 2026-05-20

Audit owner: solo Codex audit lane for `microservices/tasks`.
Target microservice: `tasks`.
Top-3 counterpart union bar: Linear, Jira Software, Asana.
Requested deployment contexts: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, `oyatie-as-cloud-provider`.
Deliverable set: three reports only; the capability-tier delta report is retired.
Audit stance: evidence-bound, read-first, write-only under `microservices/tasks`.

Canonical anchor 1: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1732` requires the six-context deployment matrix.
Canonical anchor 2: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2243` makes OpenTofu the canonical infrastructure surface.
Canonical anchor 3: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2659` defines the operating-system support matrix.
Canonical anchor 4: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4011` defines the Rust-only implementation audit dimension.
Canonical anchor 5: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3831` defines the nine-dimension audit structure.
Canonical anchor 6: `specs/master-plan-sequencing.json:704` lists all six deployment contexts.
Canonical anchor 7: `specs/master-plan-sequencing.json:747` marks OpenTofu as the IaC substrate and bans Terraform-family drift.
Canonical anchor 8: `specs/master-plan-sequencing.json:777` defines the supported OS matrix.
Canonical anchor 9: `specs/master-plan-sequencing.json:817` defines the language policy: Rust backend, allowlisted client frontends, Leptos web.
Canonical anchor 10: `feedback_no_tenant_class_adoption_2026_05_20.md:10` retires demo_trial tenant_class, paid tenant_class baseline, paid tenant_class scale, and compliance_pack-gated paid tenant_class capability tiers.

Chat-history anchor 1: `.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:17187` records the tier-retirement and tenant-class adoption guidance.
Chat-history anchor 2: `.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:17203` records that `tasks` was explicitly in the Batch 3.2 audit set.
Memory anchor 1: `feedback_microservice_ownership_coherence_2026_05_20.md:10` requires one owner per microservice audit and no inter-agent blending.
Memory anchor 2: `feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10` warns that line count is insufficient without substance verification.
Memory anchor 3: `feedback_docs_substance_not_scaffold_2026_05_20.md:10` requires audit substance instead of scaffold repetition.

## §1 Purpose

This report audits whether `microservices/tasks` is coherent as the owner of the tasks product surface.
The product purpose is a native work-item and task-management substrate for Oyatie.
The local PRD says it owns task CRUD, projects, views, recurrence, dependencies, checklists, attachments, comments, time tracking, custom fields, filters, templates, sprint and iteration features, milestones, workflows, automation, notifications, bulk edit, search, imports, exports, API, webhooks, calendar bridge, roadmap, portfolio, and AI-assisted task workflows at `microservices/tasks/PRD.md:22`.
The PRD further positions migration from Asana, Trello, Linear, Jira, and Todoist as a core customer path at `microservices/tasks/PRD.md:37`.
The audit therefore treats `tasks` as a product-grade work-management system, not as a thin ticket table.
The audit compares current artifacts against Linear, Jira Software, and Asana because the dispatch names those as the top-3 counterpart union bar.
The audit also checks the canonical direction sources that apply across every microservice.
The most important cross-cutting checks are multi-context deployment, OpenTofu IaC, supported OS coverage, Rust-strict backend policy, OCI Always Free profile, tier retirement, and tenant-class adoption.
The audit is documentation-only and does not try to implement the missing runtime.
The audit does not create a fourth capability-tier delta deliverable because the current directive retires that deliverable.
The audit does not normalize other microservices or shared docs because the write scope is `microservices/tasks/*`.
The audit uses file evidence, chat-history evidence, and explicit memory constraints for every substantive finding.
The audit classifies issues by the ADR-0328 severity posture: P0 for false completion, P1 for missing canonical implementation substrate, P2 for documentation or alignment gaps that can be corrected without product invention, and P3 for polish or prioritization concerns.
The audit uses "tenant_class" for the replacement model required by the current dispatch: `demo_trial`, `paid`, and `revenue_share`.
The audit treats all three tenant classes as having the same product-quality bar.
The audit treats demo infrastructure limits as usage or infrastructure caps, not as reduced feature quality.
The audit treats OCI Always Free as an infrastructure profile and never as a capability tier.
The audit treats T0, T1, and T2 strings in the tasks artifacts as autonomy-level labels unless local wording clearly uses them as product tiers.
The audit does not scrub existing retired wording because the requested action is an audit, not a cleanup pass.
The audit calls out retired tier references as Wave 15J retirement candidates.
The audit separately calls out generic `tier` language where it describes tenant billing, customer classes, or infrastructure class.
The audit distinguishes that generic `tier` wording from the explicit demo_trial tenant_class, paid tenant_class baseline, paid tenant_class scale, and compliance_pack-gated paid tenant_class retirement catalog.
The audit accepts local Helm and Kustomize artifacts as useful deployment packaging evidence.
The audit does not treat Helm and Kustomize as a replacement for required OpenTofu context modules.
The audit accepts OpenAPI, AsyncAPI, proto, OpenSLO, runbooks, DPIA, and capacity docs as product maturity evidence.
The audit does not treat those docs as buildability evidence when `src/` and `tests/` are absent.
The audit uses absence checks only where the required files were searched directly.
The inventory count is 124 files under `microservices/tasks`.
The inventory line count read is 20,568 lines under `microservices/tasks`.
The chat-history search produced 2,502 total `tasks` matches and 152 targeted matches for Batch 3.2, tier retirement, tenant classes, and counterpart context.
The strongest positive signal is that tasks has broad product documentation, contracts, OpenSLOs, runbooks, compliance materials, and detailed competitor ambition.
The strongest negative signal is that the canonical deployment, OpenTofu, OS, and Rust buildability surfaces are not yet coherent with the documentation claims.
The second strongest negative signal is that retired tier vocabulary is still deeply embedded in the tasks materials.
The third strongest negative signal is that tenant-class semantics are not adopted in tasks artifacts despite the current directive.
The fourth strongest negative signal is that several artifacts look generated or scaffold-like while the doctrine requires intern-buildable substance.
The stop condition for this audit is not "all gaps fixed".
The stop condition is that the three requested reports identify the current state, cite evidence, classify gaps, and give a concrete path for Wave 15J and follow-on service hardening.

## §2 Inventory

Inventory method: `rg --files microservices/tasks | sort` returned 124 files.
Inventory scope: every file under `microservices/tasks` was included in the inventory count.
Inventory read depth: root docs, decisions, implementation plans, contracts, SLOs, capability-tier docs, runbooks, migration docs, benchmark docs, compliance docs, dashboards, IaC packaging, and a representative Rust reference implementation were read or sampled directly.
Inventory caveat: no root `README.md` exists under `microservices/tasks`.
Inventory caveat: no `microservices/tasks/src/` directory exists.
Inventory caveat: no `microservices/tasks/tests/` directory exists.
Inventory caveat: the audit did not touch files outside the requested microservice path.

### §2.1 Complete file inventory

1. `microservices/tasks/ARCHITECTURE.md`
2. `microservices/tasks/AUDIT-FINDINGS-2026-05-18.json`
3. `microservices/tasks/IP-001-iac-bootstrap.md`
4. `microservices/tasks/IP-002-cargo-workspace-bootstrap.md`
5. `microservices/tasks/IP-003-task-core-domain.md`
6. `microservices/tasks/IP-004-task-projects-and-views.md`
7. `microservices/tasks/IP-005-task-dependencies-and-recurrence.md`
8. `microservices/tasks/IP-006-comments-attachments-and-checklists.md`
9. `microservices/tasks/IP-007-import-export-connectors.md`
10. `microservices/tasks/IP-008-realtime-notifications-and-watchers.md`
11. `microservices/tasks/IP-009-search-analytics-and-reporting.md`
12. `microservices/tasks/IP-010-automation-and-template-engine.md`
13. `microservices/tasks/IP-011-security-compliance-and-audit.md`
14. `microservices/tasks/IP-012-task-api-sdk-and-webhooks.md`
15. `microservices/tasks/IP-013-slo-runbooks-and-ops.md`
16. `microservices/tasks/IP-014-frontend-integration.md`
17. `microservices/tasks/IP-015-acceptance-and-load-testing.md`
18. `microservices/tasks/IP-journey-j100-compliance-export-and-legal-hold.md`
19. `microservices/tasks/IP-journey-j91-project-template-import.md`
20. `microservices/tasks/IP-journey-j92-recurring-task-automation.md`
21. `microservices/tasks/IP-journey-j93-sprint-planning.md`
22. `microservices/tasks/IP-journey-j94-multi-assignee-handoff.md`
23. `microservices/tasks/IP-journey-j95-bulk-status-update.md`
24. `microservices/tasks/IP-journey-j96-task-search-saved-view.md`
25. `microservices/tasks/IP-journey-j97-dependency-cycle-resolution.md`
26. `microservices/tasks/IP-journey-j98-portfolio-rollup.md`
27. `microservices/tasks/IP-journey-j99-autonomy-suggestion-review.md`
28. `microservices/tasks/PHASE-01-TASKS-FOUNDATION.md`
29. `microservices/tasks/PRD.md`
30. `microservices/tasks/backfill-replay.md`
31. `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md`
32. `microservices/tasks/capabilities/T0-suggest.autonomy-capability.json`
33. `microservices/tasks/capabilities/T1-assist.autonomy-capability.json`
34. `microservices/tasks/capabilities/T2-auto.autonomy-capability.json`
35. `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md`
36. `microservices/tasks/capability-tiers/tier-matrix.md`
37. `microservices/tasks/capacity-model.md`
38. `microservices/tasks/catalog/tasks_activity_index.yaml`
39. `microservices/tasks/catalog/tasks_attachment.yaml`
40. `microservices/tasks/catalog/tasks_audit_projection.yaml`
41. `microservices/tasks/catalog/tasks_bulk_operation.yaml`
42. `microservices/tasks/catalog/tasks_comment.yaml`
43. `microservices/tasks/catalog/tasks_dependency_edge.yaml`
44. `microservices/tasks/catalog/tasks_import_batch.yaml`
45. `microservices/tasks/catalog/tasks_notification_pref.yaml`
46. `microservices/tasks/catalog/tasks_project.yaml`
47. `microservices/tasks/catalog/tasks_recurrence_rule.yaml`
48. `microservices/tasks/catalog/tasks_saved_view.yaml`
49. `microservices/tasks/catalog/tasks_search_index.yaml`
50. `microservices/tasks/catalog/tasks_sequence_counter.yaml`
51. `microservices/tasks/catalog/tasks_sprint.yaml`
52. `microservices/tasks/catalog/tasks_status_transition.yaml`
53. `microservices/tasks/catalog/tasks_task.yaml`
54. `microservices/tasks/catalog/tasks_template.yaml`
55. `microservices/tasks/catalog/tasks_webhook_subscription.yaml`
56. `microservices/tasks/competitor-parity-matrix.md`
57. `microservices/tasks/compliance.md`
58. `microservices/tasks/contracts/tasks.asyncapi.yaml`
59. `microservices/tasks/contracts/tasks.openapi.yaml`
60. `microservices/tasks/contracts/tasks.proto`
61. `microservices/tasks/cost-budget.md`
62. `microservices/tasks/dashboards/tasks-autonomy-review-dashboard.json`
63. `microservices/tasks/dashboards/tasks-latency-slo.json`
64. `microservices/tasks/dashboards/tasks-operational-dashboard.json`
65. `microservices/tasks/decisions/ADR-TASKS-0001-domain-boundary.md`
66. `microservices/tasks/decisions/ADR-TASKS-0002-task-identity.md`
67. `microservices/tasks/decisions/ADR-TASKS-0003-recurrence-model.md`
68. `microservices/tasks/decisions/ADR-TASKS-0004-import-export-idempotency.md`
69. `microservices/tasks/decisions/ADR-TASKS-0005-realtime-watchers.md`
70. `microservices/tasks/decisions/ADR-TASKS-0006-search-indexing.md`
71. `microservices/tasks/decisions/ADR-TSK-001-task-projections-and-outbox.md`
72. `microservices/tasks/decisions/README.md`
73. `microservices/tasks/deprecation-notice.md`
74. `microservices/tasks/dpia.md`
75. `microservices/tasks/failure-modes.md`
76. `microservices/tasks/faqs/engineer-faq.md`
77. `microservices/tasks/iac/helm/tasks/Chart.yaml`
78. `microservices/tasks/iac/helm/tasks/templates/deployment.yaml`
79. `microservices/tasks/iac/helm/tasks/templates/prometheusrule.yaml`
80. `microservices/tasks/iac/helm/tasks/templates/service.yaml`
81. `microservices/tasks/iac/helm/tasks/templates/servicemonitor.yaml`
82. `microservices/tasks/iac/helm/tasks/values.yaml`
83. `microservices/tasks/iac/kustomize/base/kustomization.yaml`
84. `microservices/tasks/iac/kustomize/overlays/pack-eu/kustomization.yaml`
85. `microservices/tasks/iac/kustomize/overlays/pack-kr/kustomization.yaml`
86. `microservices/tasks/incident-response.md`
87. `microservices/tasks/manifest.json`
88. `microservices/tasks/migration-from-connect.md`
89. `microservices/tasks/migration-playbooks/from-jira-cloud.md`
90. `microservices/tasks/multi-region.md`
91. `microservices/tasks/onboarding/engineer-first-week.md`
92. `microservices/tasks/policy/availability-slo.md`
93. `microservices/tasks/policy/data-retention-and-deletion.md`
94. `microservices/tasks/policy/data-residency.md`
95. `microservices/tasks/policy/encryption-key-boundary.md`
96. `microservices/tasks/policy/privacy-and-dpia.md`
97. `microservices/tasks/policy/safety-and-autonomy.md`
98. `microservices/tasks/policy/security-controls.md`
99. `microservices/tasks/reference-implementations/rust/importer-idempotency.rs`
100. `microservices/tasks/runbooks/auto-assign-review-queue.md`
101. `microservices/tasks/runbooks/bulk-edit-throttle.md`
102. `microservices/tasks/runbooks/dependency-cycle-alert.md`
103. `microservices/tasks/runbooks/import-stalled.md`
104. `microservices/tasks/runbooks/recurrence-runner-lag.md`
105. `microservices/tasks/runbooks/search-index-lag.md`
106. `microservices/tasks/runbooks/task-create-error-budget-burn.md`
107. `microservices/tasks/runbooks/webhook-fanout-degraded.md`
108. `microservices/tasks/scorecards/overrides.json`
109. `microservices/tasks/sdk-plan.md`
110. `microservices/tasks/slos/auto-assign-review.openslo.yaml`
111. `microservices/tasks/slos/bulk-update.openslo.yaml`
112. `microservices/tasks/slos/dependency-cycle-check.openslo.yaml`
113. `microservices/tasks/slos/recurring-task-generation.openslo.yaml`
114. `microservices/tasks/slos/search-freshness.openslo.yaml`
115. `microservices/tasks/slos/task-create.openslo.yaml`
116. `microservices/tasks/slos/task-list-render.openslo.yaml`
117. `microservices/tasks/slos/task-update.openslo.yaml`
118. `microservices/tasks/slos/webhook-fire.openslo.yaml`
119. `microservices/tasks/threat-model.md`
120. `microservices/tasks/tutorials/migrate-asana-project.md`
121. `microservices/tasks/upstream-glossary-audit.md`
122. `microservices/tasks/xliff/en.xlf`
123. `microservices/tasks/xliff/ko.xlf`
124. `microservices/tasks/xliff/README.md`

### §2.2 Artifact-family inventory assessment

Root product docs are present: `PRD.md`, `ARCHITECTURE.md`, `PHASE-01-TASKS-FOUNDATION.md`, capacity, failure modes, incident response, cost, DPIA, compliance, multi-region, threat model, SDK plan, migration, and deprecation docs.
Root README is absent, so entrypoint orientation relies on PRD and architecture rather than a simple root map.
Decisions are present under `decisions/` and include six ADR-TASKS files plus one ADR-TSK file.
Implementation plans are numerous and cover foundation through journeys J91-J100.
Contracts are present in OpenAPI, AsyncAPI, and proto forms.
OpenSLO definitions are present for nine headline operational paths.
Capability docs exist, but the `capability-tiers/` directory is a Wave 15J retirement candidate.
Runbooks exist for import, search, recurrence, task-create burn, webhook fanout, dependency cycles, bulk edit throttling, and auto-assign review queues.
Migration and tutorial docs exist for Jira and Asana.
Reference implementation coverage is narrow: one Rust importer-idempotency reference exists, but no full Rust crate tree exists.
IaC packaging exists for Helm and Kustomize only.
OpenTofu context modules are absent.
The contract family is more mature than the implementation family.
The SLO family is more mature than the infrastructure-context family.
The compliance family is extensive but contains repeated tier-era vocabulary and some claims not backed by local files.
The cost-budget family still uses old tenant-pricing classes rather than tenant_class semantics.
The benchmark family uses an old tiered hardware assumption and must be replaced or rewritten.
The catalog family contains 18 YAML entities while the PRD and IP-002 discuss 57 crates or modules.
The manifest lists the 18 catalog files and 15 core IPs, but it omits the journey IPs.
The manifest also carries `tenant_class_adoption` and `tier_classification` keys that should be reviewed carefully during Wave 15J.
The xliff files indicate localization scaffolding, but localization is not the dominant risk in this audit.

## §3 Nine-dimension audit

### §3.1 Dimension 1 - product purpose and ownership boundary

Evidence: `microservices/tasks/PRD.md:22` defines a broad task-management substrate.
Evidence: `microservices/tasks/PRD.md:40` through `microservices/tasks/PRD.md:63` define twenty functional requirement families.
Evidence: `microservices/tasks/PRD.md:187` says `tasks` must not import product microservice crates directly.
Evidence: `microservices/tasks/ARCHITECTURE.md:40` through `microservices/tasks/ARCHITECTURE.md:47` names cross-service relationships.
Evidence: `microservices/tasks/manifest.json:455` through `microservices/tasks/manifest.json:471` lists dependencies including `connector`.
Finding: the intended ownership boundary is large but coherent: task records, projects, views, search, dependency graph, imports, exports, automation, recurrence, comments, attachments, and webhooks belong here.
Finding: the product purpose is closer to Linear plus Jira plus Asana union coverage than to a small issue CRUD service.
Finding: the boundary statement in PRD is stronger than the manifest dependency statement.
Finding: `connector` dependency language in the manifest conflicts with the PRD's statement that tasks is no longer part of the old platform.
Finding: the cross-product rule is appropriate and should stay: tasks should publish contracts and events, not import product microservice crates.
Finding: the local docs correctly identify many consumer services, including mail, calendar, drive, messenger, workflow, ontology, audit, tenancy, and billing.
Risk: product breadth is large enough that the service needs a tighter contract/implementation split before development begins.
Risk: if every competitor feature is treated as in-scope for the first implementation slice, the service will not become buildable.
Risk: if the manifest is not updated, future agents may keep rebuilding the retired dependency path.
Positive signal: the PRD's requirements map well to a full work-management product.
Positive signal: the architecture uses ports and adapters language that can support Rust crate boundaries.
Positive signal: the compliance and DPIA docs recognize task data as potentially sensitive work data.
Coherence verdict: directionally coherent, but the manifest and implementation evidence lag behind the PRD.

### §3.2 Dimension 2 - artifact completeness and buildability

Evidence: `microservices/tasks/PRD.md:153` through `microservices/tasks/PRD.md:165` describe a 57-crate or 57-module layer mapping.
Evidence: `microservices/tasks/IP-002-cargo-workspace-bootstrap.md:16` through `microservices/tasks/IP-002-cargo-workspace-bootstrap.md:26` calls for 57 crate stubs.
Evidence: `microservices/tasks/IP-002-cargo-workspace-bootstrap.md:50` through `microservices/tasks/IP-002-cargo-workspace-bootstrap.md:61` names Cargo workspace and source paths.
Evidence: `microservices/tasks/IP-002-cargo-workspace-bootstrap.md:85` references `microservices/tasks/specs/naming-justification.md`.
Evidence: `microservices/tasks/PRD.md:151` also references `microservices/tasks/specs/naming-justification.md`.
Evidence: filesystem inventory shows no `microservices/tasks/src/` directory.
Evidence: filesystem inventory shows no `microservices/tasks/tests/` directory.
Evidence: filesystem inventory shows no `microservices/tasks/specs/naming-justification.md`.
Evidence: `microservices/tasks/manifest.json:166` through `microservices/tasks/manifest.json:256` lists IP-001 through IP-015, but not journey plans J91-J100.
Finding: the artifact tree is documentation-heavy and contract-heavy.
Finding: buildability evidence is absent because there is no crate tree, Cargo manifest, source tree, or test tree.
Finding: the 57-crate claim is not backed by local source files.
Finding: the implementation plans acknowledge pending work but still reference missing paths as if they are planned deliverables.
Finding: the journey implementation plans are present but not represented in the manifest IP list.
Finding: the root README absence makes the large document set harder for a new engineer to navigate.
Finding: the single Rust reference implementation is useful but cannot prove the service builds.
Risk: auditors could mistakenly count artifact volume as implementation progress.
Risk: future agents may implement against missing paths without first deciding crate boundaries.
Risk: the missing naming-justification doc undermines a PRD acceptance point.
Positive signal: contracts, SLOs, runbooks, and policy docs give a strong target surface for implementation.
Positive signal: a Rust reference implementation exists under `reference-implementations/rust/importer-idempotency.rs`.
Coherence verdict: strong planning surface, weak implementation substrate.

### §3.3 Dimension 3 - contracts, events, data, and operational promises

Evidence: `microservices/tasks/contracts/tasks.openapi.yaml:1` declares OpenAPI 3.2.0.
Evidence: `microservices/tasks/contracts/tasks.asyncapi.yaml:1` declares AsyncAPI 3.1.0.
Evidence: `microservices/tasks/contracts/tasks.proto:1` declares proto3.
Evidence: `microservices/tasks/contracts/tasks.asyncapi.yaml:35` through `microservices/tasks/contracts/tasks.asyncapi.yaml:38` covers task-created and task-updated events.
Evidence: `microservices/tasks/contracts/tasks.asyncapi.yaml:67` through `microservices/tasks/contracts/tasks.asyncapi.yaml:72` covers dependency events.
Evidence: `microservices/tasks/contracts/tasks.asyncapi.yaml:97` through `microservices/tasks/contracts/tasks.asyncapi.yaml:102` covers import/export events.
Evidence: `microservices/tasks/PRD.md:93` lists more audit and event types than the manifest audit-chain excerpt.
Evidence: `microservices/tasks/manifest.json:343` through `microservices/tasks/manifest.json:350` lists only three capability audit events.
Evidence: `microservices/tasks/slos/task-create.openslo.yaml` defines a task-create SLO.
Evidence: `microservices/tasks/slos/task-update.openslo.yaml` defines a task-update SLO.
Evidence: `microservices/tasks/slos/search-freshness.openslo.yaml` defines a search freshness SLO.
Evidence: `microservices/tasks/capacity-model.md:37` through `microservices/tasks/capacity-model.md:49` lists per-cell RPS and throughput targets.
Finding: contract coverage is broad enough to support public APIs, events, and inter-service integration.
Finding: OpenAPI, AsyncAPI, and proto coverage is a positive maturity signal.
Finding: the audit chain in the manifest is narrower than the PRD's event surface.
Finding: events around assignment, comments, bulk edit, legal hold, and retention should be checked against contract and audit-chain coverage.
Finding: OpenSLO files create measurable targets, but they need load-test evidence before being claimed as achieved.
Finding: the capacity model is specific and stronger than generic scale claims.
Risk: a partial audit-chain list can create false compliance confidence.
Risk: event-contract drift can break billing, audit, workflow, and notifications consumers.
Risk: proto/OpenAPI/AsyncAPI parity has not been mechanically verified in this audit.
Positive signal: the contracts are concrete files rather than prose-only API descriptions.
Positive signal: the capacity model gives concrete values for fetch, write, status update, and bulk edit capacity.
Coherence verdict: good contract base; audit-chain and parity validation still needed.

### §3.4 Dimension 4 - canonical-direction alignment

Evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1732` requires all six deployment contexts.
Evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2079` requires manifests to name supported contexts and N/A rationale fields.
Evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2243` requires OpenTofu, not Terraform.
Evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2275` through `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2294` requires per-service `iac/<context>/` modules.
Evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2659` through `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2854` defines the OS support matrix.
Evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4011` through `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4083` defines the Rust-only language audit.
Evidence: `specs/master-plan-sequencing.json:704` through `specs/master-plan-sequencing.json:746` names all six deployment contexts.
Evidence: `specs/master-plan-sequencing.json:747` through `specs/master-plan-sequencing.json:775` bans Terraform, Pulumi, CloudFormation, ARM, null_resource, local-exec, and SSH bootstrap patterns.
Evidence: `specs/master-plan-sequencing.json:857` through `specs/master-plan-sequencing.json:868` describes OCI Always Free constraints.
Finding: tasks does not contain the required six context OpenTofu module directories.
Finding: tasks contains Helm and Kustomize packaging, which is useful but not sufficient for the canonical IaC substrate.
Finding: tasks has no `supported-oses.json`.
Finding: tasks has no forbidden backend source file extensions found by source-file grep.
Finding: tasks has no Python, JavaScript, TypeScript, Ruby, Go, Java, Scala, Groovy, PHP, or F# source files under the microservice path.
Finding: tasks does contain one allowed Rust reference file.
Finding: tasks contains a high count of retired tier words and must be included in Wave 15J retirement work.
Finding: tasks does not yet use current tenant_class semantics.
Risk: the service could look mature from docs while failing the canonical deployment and OS gates.
Risk: stale tier vocabulary may cause future implementation to recreate forbidden pricing or capability stratification.
Risk: absence of tenant_class semantics blocks correct billing, onboarding, usage caps, and revenue-share modelling.
Positive signal: no forbidden IaC technologies or forbidden source-language files were found in tasks.
Positive signal: no Terraform, Pulumi, CloudFormation, provisioner, null_resource, local-exec, remote-exec, or hand-edited tfstate patterns were found in tasks.
Coherence verdict: high documentation ambition, incomplete canonical substrate adoption.

#### §3.4.A Multi-context deployment check

Required context: `oyatie-public-cloud`.
Evidence status: missing required `microservices/tasks/iac/oyatie-public-cloud/` module.
Required context: `guest-on-aws`.
Evidence status: missing required `microservices/tasks/iac/guest-on-aws/` module.
Required context: `guest-on-oci`.
Evidence status: missing required `microservices/tasks/iac/oci-guest/` module.
Required context: `guest-on-oci` Always Free profile.
Evidence status: missing required `microservices/tasks/iac/oci-guest/always-free/` module.
Required context: `on-prem`.
Evidence status: missing required `microservices/tasks/iac/on-prem/` module.
Required context: `colo`.
Evidence status: missing required `microservices/tasks/iac/colo/` module.
Required context: `oyatie-as-cloud-provider`.
Evidence status: missing required `microservices/tasks/iac/oyatie-iaas/` module.
Local deployment evidence: `microservices/tasks/iac/helm/tasks/Chart.yaml`.
Local deployment evidence: `microservices/tasks/iac/kustomize/base/kustomization.yaml`.
Local deployment evidence: `microservices/tasks/iac/kustomize/overlays/pack-eu/kustomization.yaml`.
Local deployment evidence: `microservices/tasks/iac/kustomize/overlays/pack-kr/kustomization.yaml`.
Conclusion: packaging exists, but canonical context modules do not.
Severity: P1 because deployment-context substrate is a cross-cutting canonical requirement.

#### §3.4.B OpenTofu check

Expected directory pattern: `microservices/tasks/iac/<context>/main.tf`.
Expected directory pattern: `microservices/tasks/iac/<context>/variables.tf`.
Expected directory pattern: `microservices/tasks/iac/<context>/outputs.tf`.
Expected directory pattern: `microservices/tasks/iac/<context>/versions.tf`.
Expected directory pattern: `microservices/tasks/iac/<context>/README.md`.
Current evidence: no OpenTofu context files were found under `microservices/tasks/iac`.
Current evidence: only Helm and Kustomize files were found under `microservices/tasks/iac`.
Forbidden-pattern evidence: no `Terraform Cloud`, `Pulumi`, `CloudFormation`, `null_resource`, `local-exec`, `remote-exec`, `provisioner`, `tfstate`, `terraform`, or `tofu` command content was found in tasks, except a compliance claim that inventory spans OpenTofu at `microservices/tasks/compliance.md:1030`.
Conclusion: absence is the gap; forbidden replacement technology was not found.
Severity: P1 for missing canonical IaC, P2 for the compliance claim that implies OpenTofu coverage without local files.

#### §3.4.C Tenant-class adoption gaps

Required current model: `tenant_class = demo_trial`.
Required current model: `tenant_class = paid`.
Required current model: `tenant_class = revenue_share`.
Search result: no exact `tenant_class` string was found under `microservices/tasks`.
Search result: no exact `demo_trial` string was found under `microservices/tasks`.
Search result: no exact `revenue_share` string was found under `microservices/tasks`.
Evidence: `microservices/tasks/cost-budget.md:52` through `microservices/tasks/cost-budget.md:61` uses Free, Starter, Pro, and Enterprise tenant classes instead of the current tenant_class model.
Evidence: `microservices/tasks/cost-budget.md:67` through `microservices/tasks/cost-budget.md:69` uses a tier bill model.
Evidence: `microservices/tasks/cost-budget.md:81` uses a metric label named `tier`.
Evidence: `microservices/tasks/runbooks/bulk-edit-throttle.md:32` uses tenant-tier language.
Evidence: `microservices/tasks/runbooks/bulk-edit-throttle.md:97` asks about tenant tier in escalation context.
Evidence: `microservices/tasks/runbooks/webhook-fanout-degraded.md:108` uses enterprise-tier wording.
Evidence: `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:139` through `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:142` drops tier-deltas and requires single-target performance with no tier segmentation.
Current-prompt override: the active dispatch defines three tenant classes and includes `revenue_share` as its own tenant class.
Conclusion: tasks has a tenant-class adoption gap.
Severity: P2 because it is a documentation and model-alignment gap, not a source-code build failure.
Required correction path: replace customer pricing classes and tenant-tier labels with `tenant_class`, billing component, usage-cap, and deployment-context terms.

#### §3.4.T Tier retirement candidates

Catalog method: `rg -n "demo_trial tenant_class|paid tenant_class baseline|paid tenant_class scale|compliance_pack-gated paid tenant_class" microservices/tasks` found 167 references.
Catalog interpretation: every entry below is a Wave 15J retirement candidate unless it is retained only inside an audit finding.
Default severity: P2.
Retirement candidate: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:12`.
Retirement candidate: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:18`.
Retirement candidate: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:33`.
Retirement candidate: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:59`.
Retirement candidate: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:70`.
Retirement candidate: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:74`.
Retirement candidate: `microservices/tasks/migration-playbooks/from-jira-cloud.md:21`.
Retirement candidate: `microservices/tasks/migration-playbooks/from-jira-cloud.md:34`.
Retirement candidate: `microservices/tasks/tutorials/migrate-asana-project.md:15`.
Retirement candidate: `microservices/tasks/tutorials/migrate-asana-project.md:34`.
Retirement candidate: `microservices/tasks/tutorials/migrate-asana-project.md:161`.
Retirement candidate: `microservices/tasks/faqs/engineer-faq.md:42`.
Retirement candidate: `microservices/tasks/faqs/engineer-faq.md:64`.
Retirement candidate: `microservices/tasks/faqs/engineer-faq.md:72`.
Retirement candidate: `microservices/tasks/faqs/engineer-faq.md:74`.
Retirement candidate: `microservices/tasks/faqs/engineer-faq.md:77`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-matrix.md:13`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-matrix.md:22`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-matrix.md:32`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-matrix.md:34`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-matrix.md:44`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-matrix.md:53`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-matrix.md:55`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-matrix.md:71`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-matrix.md:80`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-matrix.md:82`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-matrix.md:98`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-matrix.md:104`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-matrix.md:108`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-matrix.md:125`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-matrix.md:127`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-matrix.md:129`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-matrix.md:137`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-matrix.md:139`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-matrix.md:141`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:18`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:19`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:20`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:21`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:22`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:23`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:24`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:25`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:31`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:36`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:37`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:38`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:39`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:43`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:47`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:49`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:50`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:85`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:86`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:87`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:89`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:93`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:119`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:120`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:121`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:122`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:123`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:124`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:125`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:126`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:127`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:130`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:134`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:171`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:172`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:173`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:174`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:175`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:176`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:178`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:182`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:211`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:216`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:219`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:223`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:224`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:225`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:226`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:227`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:228`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:229`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:230`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:231`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:232`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:233`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:234`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:235`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:236`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:237`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:238`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:239`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:250`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:251`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:252`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:253`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:254`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:255`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:256`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:257`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:258`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:259`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:260`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:261`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:262`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:263`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:264`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:265`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:266`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:267`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:268`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:269`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:270`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:271`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:272`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:273`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:274`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:279`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:280`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:281`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:282`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:283`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:284`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:285`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:286`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:287`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:288`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:289`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:290`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:291`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:292`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:293`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:294`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:295`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:296`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:297`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:298`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:299`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:300`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:301`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:302`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:303`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:304`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:305`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:306`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:310`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:311`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:312`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:313`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:314`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:315`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:316`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:317`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:318`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:319`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:320`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:321`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:322`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:323`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:324`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:325`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:326`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:327`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:328`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:329`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:330`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:337`.
Retirement candidate: `microservices/tasks/capability-tiers/tier-deltas-and-pricing.md:359`.
Conclusion: tier-retirement work is not cosmetic; it affects benchmarks, tutorials, migration playbooks, FAQs, pricing docs, and capability matrices.

### §3.5 Dimension 5 - counterpart product-surface alignment

Evidence: `microservices/tasks/competitor-parity-matrix.md:34` includes Asana.
Evidence: `microservices/tasks/competitor-parity-matrix.md:36` includes Linear.
Evidence: `microservices/tasks/competitor-parity-matrix.md:37` includes Jira.
Evidence: `microservices/tasks/competitor-parity-matrix.md:148` through `microservices/tasks/competitor-parity-matrix.md:156` lists differentiators.
Evidence: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:8` includes Asana, Jira, Linear, and Monday.
Evidence: `microservices/tasks/PRD.md:255` through `microservices/tasks/PRD.md:278` includes a broad competitor benchmark table.
Finding: tasks correctly recognizes Linear, Jira, and Asana as direct counterparts.
Finding: the existing competitor materials are broader than this batch's top-3 bar and include Monday, Trello, ClickUp, Wrike, Todoist, Notion, SmartSuite, and others.
Finding: broad competitor awareness is useful, but the Batch 3.2 report must focus on the union of Linear, Jira Software, and Asana.
Finding: Linear pressure is strongest around speed, issue lifecycle, cycles, roadmaps, command palette workflows, and developer workflows.
Finding: Jira pressure is strongest around configurable workflows, permission schemes, boards, sprints, JQL-style search, auditability, marketplace depth, and enterprise administration.
Finding: Asana pressure is strongest around portfolios, goals, forms, rules, timeline, workload, templates, approvals, and non-engineering adoption.
Finding: tasks PRD claims a large enough surface to cover most of the union, but source and tests do not yet prove coverage.
Risk: if the service over-indexes on engineering issue tracking, it will miss Asana's operational work-management surface.
Risk: if the service over-indexes on flexible Jira workflows, it can miss Linear's low-latency simplicity.
Risk: if the service over-indexes on broad templates, it can miss hard enterprise permission and audit requirements.
Positive signal: existing docs already name many competitor feature areas and migration routes.
Coherence verdict: product ambition meets the union bar; implementation evidence does not yet.

### §3.6 Dimension 6 - deployment, topology, tenancy, and runtime placement

Evidence: `microservices/tasks/ARCHITECTURE.md:571` through `microservices/tasks/ARCHITECTURE.md:582` describes Kubernetes pods with Cloud Hypervisor and Kata-style isolation.
Evidence: `microservices/tasks/multi-region.md:20` through `microservices/tasks/multi-region.md:34` describes OCI region-pack topology.
Evidence: `microservices/tasks/multi-region.md:67` through `microservices/tasks/multi-region.md:72` gives RTO and RPO values.
Evidence: `microservices/tasks/policy/data-residency.md:22` through `microservices/tasks/policy/data-residency.md:38` describes pack-pinned primary storage.
Evidence: `microservices/tasks/threat-model.md:274` through `microservices/tasks/threat-model.md:280` discusses cross-context boundaries.
Finding: topology docs are rich for region packs and residency packs.
Finding: topology docs are not the same as the six required deployment contexts.
Finding: no local context module exists for AWS guest, OCI guest, on-prem, colo, public cloud, or Oyatie as provider.
Finding: data residency and multi-region docs are valuable but are skewed toward pack and OCI region concepts.
Finding: tenant isolation appears in architecture and threat-model docs, but current tenant_class semantics are absent.
Finding: runtime placement mentions Kubernetes and isolation but lacks per-context OpenTofu wiring.
Finding: capacity numbers assume a cell model, but no context-specific resource plan binds them to all six environments.
Risk: a deployment may be pack-aware but not provider-agnostic.
Risk: OCI-specific assumptions can leak into on-prem, colo, or AWS guest expectations.
Risk: tenant onboarding cannot be verified without tenant_class and context admission metadata.
Positive signal: the service has meaningful topology language rather than a single-region assumption.
Coherence verdict: strong regional-residency thinking, incomplete deployment-context productization.

### §3.7 Dimension 7 - OpenTofu and infrastructure-as-code coherence

Evidence: `microservices/tasks/IP-001-iac-bootstrap.md:16` describes Helm and Kustomize bootstrap work.
Evidence: `microservices/tasks/IP-001-iac-bootstrap.md:34` describes Helm and Kustomize targets.
Evidence: `microservices/tasks/IP-001-iac-bootstrap.md:46` through `microservices/tasks/IP-001-iac-bootstrap.md:52` lists Helm/Kustomize acceptance evidence.
Evidence: `microservices/tasks/PHASE-01-TASKS-FOUNDATION.md:82` names Helm and Kustomize as a foundation milestone.
Evidence: `microservices/tasks/compliance.md:1030` says inventory spans Helm, Kustomize, and OpenTofu.
Evidence: filesystem inventory contains Helm and Kustomize files but no OpenTofu context files.
Finding: local implementation plans were written before the OpenTofu-only canonical ratchet or have not been updated to it.
Finding: the service currently has deployment packaging but lacks infrastructure provisioning.
Finding: compliance text overclaims OpenTofu presence.
Finding: no forbidden hand-rolled cloud bootstrap technology was found in the tasks tree.
Finding: missing OpenTofu is a more important gap than forbidden IaC replacement.
Risk: packaging-only IaC can pass Kubernetes smoke checks while failing cloud account, IAM, network, storage, quota, and observability provisioning.
Risk: future audit readers could interpret the compliance file as evidence that OpenTofu already exists.
Required correction path: add per-context OpenTofu modules with explicit N/A rationale only where the master plan permits N/A.
Required correction path: keep Helm/Kustomize as deployment packaging under the OpenTofu-managed substrate.
Coherence verdict: infrastructure packaging exists, canonical substrate does not.

### §3.8 Dimension 8 - supported operating-system coverage

Evidence: ADR-0328 requires explicit OS support artifacts at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2907`.
Evidence: `feedback_os_support_matrix_2026_05_20.md:56` through `feedback_os_support_matrix_2026_05_20.md:77` requires per-microservice supported OS metadata.
Evidence: no `microservices/tasks/supported-oses.json` file exists.
Evidence: no equivalent local OS matrix file was found under `microservices/tasks`.
Evidence: task docs reference Apple Reminders and macOS import surfaces, but those are integration use cases, not supported OS matrices.
Finding: supported OS coverage is missing as a service-level artifact.
Finding: the absence is not balanced by a clear N/A rationale.
Finding: tasks has deployment-context and customer-import claims that make OS support evidence important.
Finding: the OS surface should separate backend runtime OS support from client importer and frontend support.
Finding: the current docs do not state Talos, RHEL, Oracle Linux, SLES, Ubuntu, Debian, Rocky, Alma, CentOS Stream, Amazon Linux, Flatcar, Photon, and macOS M5+ support status for tasks.
Risk: deployment claims can become generic "Linux" claims, which the canonical docs explicitly reject.
Risk: customer migration docs can imply OS compatibility that has not been mapped.
Required correction path: add `supported-oses.json` with Tier 1, test-only architecture, and out-of-scope entries matching the canonical matrix.
Coherence verdict: missing canonical OS artifact.

### §3.9 Dimension 9 - language, source, tests, and Rust-strict policy

Evidence: `specs/master-plan-sequencing.json:817` through `specs/master-plan-sequencing.json:855` defines Rust backend and allowlisted client languages.
Evidence: `feedback_rust_strict_only_no_python_2026_05_20.md:10` through `feedback_rust_strict_only_no_python_2026_05_20.md:18` requires Rust-only backend implementation.
Evidence: `feedback_rust_strict_only_no_python_2026_05_20.md:51` through `feedback_rust_strict_only_no_python_2026_05_20.md:60` forbids Python, JavaScript, TypeScript, Ruby, Go, Java, Scala, Groovy, PHP, and F# as implementation paths.
Evidence: source-file grep found no `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, or `.fs` files under `microservices/tasks`.
Evidence: `microservices/tasks/reference-implementations/rust/importer-idempotency.rs` is an allowed Rust reference implementation.
Evidence: no `Cargo.toml` or Rust crate tree exists under `microservices/tasks`.
Evidence: no Rust test tree exists under `microservices/tasks`.
Finding: tasks currently satisfies the negative language check because forbidden source extensions are absent.
Finding: tasks does not yet satisfy positive Rust buildability because the workspace and crates are absent.
Finding: implementation plans point to Rust crates but do not instantiate them.
Finding: the single Rust reference implementation is useful as a pattern for importer idempotency only.
Finding: no cargo build, cargo test, clippy, or cargo-deny evidence can be collected from the current tree.
Risk: a docs-only microservice can appear Rust-compliant because there is no implementation at all.
Risk: future agents could use scripting to generate substantive docs or code, which the memory constraints explicitly forbid.
Required correction path: land a minimal Rust workspace before claiming language-policy maturity.
Coherence verdict: no forbidden implementation language, but no buildable Rust service.

## §4 Findings table

| ID | Severity | Finding | Evidence | Required action |
| --- | --- | --- | --- | --- |
| F-TSK-001 | P1 | The six canonical OpenTofu deployment context modules are missing. | `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1732`; `specs/master-plan-sequencing.json:704`; local inventory under `microservices/tasks/iac` contains only Helm/Kustomize. | Add per-context OpenTofu modules or explicit canonical N/A rationale where permitted. |
| F-TSK-002 | P1 | The OCI Always Free profile module is missing. | `specs/master-plan-sequencing.json:857`; `feedback_oci_always_free_maximization_2026_05_20.md:65`; no `microservices/tasks/iac/oci-guest/always-free/`. | Add `iac/oci-guest/always-free/` with demo_trial infrastructure caps and no old tier wording. |
| F-TSK-003 | P1 | The service lacks a supported OS artifact. | `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2907`; `feedback_os_support_matrix_2026_05_20.md:56`; no `supported-oses.json`. | Add service-level OS support metadata matching the canonical matrix. |
| F-TSK-004 | P1 | The Rust implementation substrate is absent. | `microservices/tasks/IP-002-cargo-workspace-bootstrap.md:16`; `microservices/tasks/IP-002-cargo-workspace-bootstrap.md:50`; no `src/`, `tests/`, or Cargo workspace under tasks. | Create the minimal Rust workspace before claiming implementation maturity. |
| F-TSK-005 | P2 | Retired demo_trial tenant_class/paid tenant_class baseline/paid tenant_class scale/compliance_pack-gated paid tenant_class vocabulary appears 167 times. | §3.4.T catalog; `feedback_no_tenant_class_adoption_2026_05_20.md:10`. | Schedule Wave 15J cleanup for benchmarks, tutorials, FAQs, migration playbooks, and capability-tier docs. |
| F-TSK-006 | P2 | Tenant-class semantics are not adopted. | §3.4.C; `microservices/tasks/cost-budget.md:52`; no `tenant_class`, `demo_trial`, or `revenue_share` exact strings. | Replace old pricing classes and tenant-tier labels with current `tenant_class` and billing-component language. |
| F-TSK-007 | P2 | The benchmark doc uses old tiered hardware assumptions. | `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:12`; `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:139`. | Rewrite benchmark numbers as a single industry-leader target set with deployment-context and tenant_class overlays. |
| F-TSK-008 | P2 | Manifest dependency on `connector` conflicts with PRD boundary language. | `microservices/tasks/PRD.md:187`; `microservices/tasks/manifest.json:455` through `microservices/tasks/manifest.json:471`. | Decide whether `connector` is compatibility-only, then update manifest dependency semantics. |
| F-TSK-009 | P2 | PRD and IP-002 reference a missing naming justification spec. | `microservices/tasks/PRD.md:151`; `microservices/tasks/IP-002-cargo-workspace-bootstrap.md:85`; no referenced file in inventory. | Add the spec or remove the acceptance criterion if superseded. |
| F-TSK-010 | P2 | Manifest audit-chain coverage is narrower than PRD event coverage. | `microservices/tasks/PRD.md:93`; `microservices/tasks/manifest.json:343` through `microservices/tasks/manifest.json:350`. | Align audit-chain, AsyncAPI, and PRD event lists. |
| F-TSK-011 | P2 | Architecture still contains scaffold markers. | `microservices/tasks/ARCHITECTURE.md:3`; `feedback_docs_substance_not_scaffold_2026_05_20.md:10`. | Replace scaffold markers and scaffold-like summaries with buildable architecture detail. |
| F-TSK-012 | P2 | Multi-region and residency docs are pack-rich but not six-context complete. | `microservices/tasks/multi-region.md:20`; `microservices/tasks/policy/data-residency.md:22`; `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3854`. | Add context-specific overlays for all six contexts. |
| F-TSK-013 | P2 | Compliance text implies OpenTofu coverage that local files do not support. | `microservices/tasks/compliance.md:1030`; local `iac/` inventory. | Correct the claim or land the OpenTofu modules. |
| F-TSK-014 | P2 | Implementation-plan and manifest inventories disagree about journey plans. | `microservices/tasks/manifest.json:166` through `microservices/tasks/manifest.json:256`; local inventory includes `IP-journey-j91` through `IP-journey-j100`. | Add journey IPs to the manifest or move them to a separate tracked program. |
| F-TSK-015 | P3 | Counterpart docs are broader than this batch's top-3 union bar. | `microservices/tasks/competitor-parity-matrix.md:34`; `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:8`. | Keep broad market notes, but make Batch 3.2 reports focus on Linear, Jira Software, and Asana. |
| F-TSK-016 | P3 | Generic `tier` vocabulary remains outside the explicit retired-word catalog. | `microservices/tasks/cost-budget.md:81`; `microservices/tasks/runbooks/bulk-edit-throttle.md:32`; `microservices/tasks/multi-region.md:69`. | Decide whether each generic usage means tenant_class, billing component, cell class, autonomy level, or capacity class. |

## §5 Open questions

1. Should `connector` remain in `manifest.json` as a compatibility migration dependency, or should it be removed to match the PRD's standalone tasks boundary?
2. Should the first Rust workspace slice implement the 18 current catalog entities or the 57-crate shape described by PRD and IP-002?
3. Should the missing `specs/naming-justification.md` be created as a service-local spec or promoted to a shared naming registry?
4. Should T0/T1/T2 autonomy labels be renamed in manifest keys to avoid confusion with retired capability-tier language?
5. Should Wave 15J delete the `capability-tiers/` directory outright or replace it with a tenant-class and billing-overlay directory?
6. Should the task migration playbooks be rewritten before implementation so import contracts stop encoding retired customer classes?
7. Should the benchmark doc keep local estimates as historical evidence or be superseded entirely by the new performance benchmark report?
8. Should all OpenSLO files get explicit deployment-context labels once OpenTofu context modules exist?
9. Should tasks have a dedicated `supported-oses.json` or should service OS support be generated from a shared canonical registry with local overrides?
10. Should OCI Always Free profile caps be expressed in `cost-budget.md`, OpenTofu variables, or both?
11. Should the Asana/Jira/Linear importers be implemented as first-class Rust crates or as connector adapters under a smaller import crate?
12. Should the manifest include the journey IP files as active delivery slices or archive them as exploratory journeys?
13. Should the audit-chain list become generated from AsyncAPI/proto event definitions to prevent future drift?
14. Should compliance signoff remain blocked until OpenTofu, supported OS, and tenant_class docs are aligned?
15. Should Jira-style workflow customization be in the tasks service or delegated to the workflow service with tasks holding only lifecycle state?
16. Should Asana-style goals and portfolios be native tasks features or references to product/portfolio services?
17. Should Linear-style cycles be modeled as tasks-native constructs or as a specialized project view?
18. Should task search rely only on the local search index catalog or integrate with a shared search service?
19. Should legal hold and retention events be modeled in the manifest audit_chain before implementation starts?
20. Should the first implementation gate require cargo build, cargo test, OpenAPI validation, AsyncAPI validation, and OpenSLO lint together?

<!-- ORCHESTRATOR REPORT
  µservice: tasks
  deliverables_landed: microservices/tasks/coherence-audit-2026-05-20.md (720 lines); microservices/tasks/feature-parity-matrix-2026-05-20.md (416 lines); microservices/tasks/performance-benchmark-numbers-2026-05-20.md (341 lines)
  inventory_files_seen: 124
  inventory_lines_read: 20568
  chat_history_matches_processed: 152 targeted / 2502 total tasks matches
  findings_p0: 0
  findings_p1: 4
  findings_p2: 10
  findings_p3: 2
  tier_retirement_candidates_found: 167 refs - benchmarks/tasks-vs-asana-jira-linear-monday.md:12,18,33,59,70,74; migration-playbooks/from-jira-cloud.md:21,34; tutorials/migrate-asana-project.md:15,34,161; faqs/engineer-faq.md:42,64,72,74,77; capability-tiers/tier-matrix.md:13,22,32,34,44,53,55,71,80,82,98,104,108,125,127,129,137,139,141; capability-tiers/tier-deltas-and-pricing.md:18-25,31,36-39,43,47,49,50,85-87,89,93,119-127,130,134,171-176,178,182,211,216,219,223-239,250-274,279-306,310-330,337,359
  tenant_class_adoption_gaps: yes - no tenant_class/demo_trial/revenue_share adoption; old pricing and tier labels remain
  top_3_counterparts_confirmed: Linear / Jira Software / Asana
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1477
-->
