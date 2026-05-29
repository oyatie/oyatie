# workflow-studio Feature Parity Matrix

Audit date: 2026-05-20.
µservice: `workflow-studio`.
Counterpart 1: n8n.
Counterpart 2: Zapier.
Counterpart 3: Make (Integromat).
Purpose: compare actual Workflow Studio artifacts against the union capability surface of the three requested industry counterparts.
Method: use local service artifacts for Oyatie capability claims and public counterpart sources for external capability surfaces.
No retired commercial capability class model is used in this matrix.
Tenant differentiation model: `demo_trial`, `paid`, and `revenue_share`, with uniform feature quality and different usage/billing/infrastructure envelopes.
Local purpose source: `PRD.md:23-35`.
Local counterpart source: `competitor-parity-matrix.md:25-44`.
Local migration source: `migration-playbooks/from-n8n.md`.
Canonical no commercial-tier source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_capability_stages_2026_05_20.md:10-24`.
Canonical tenant-class source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:133-142`.
Public n8n integrations source: https://n8n.io/integrations/n8n/ reports n8n connected with more than 1000 services.
Public n8n template source: https://n8n.io/workflows/ reports 9752 workflow automation templates at retrieval time.
Public n8n scale source: https://docs.n8n.io/hosting/scaling/queue-mode/ describes queue mode with main process, Redis, database, and worker nodes (counterpart-fact: n8n queue-mode docs).
Public Zapier app source: https://zapier.com/pricing reports more than 8000 app integrations.
Public Zapier limit source: https://help.zapier.com/hc/en-us/articles/8496181445261-Zap-limits documents 100-step Zap limit and rate limits.
Public Zapier trigger source: https://help.zapier.com/hc/en-us/articles/8496244568589-How-Zap-triggers-work documents polling and instant triggers.
Public Make app source: https://www.make.com/en/integrations reports 3479 app results at retrieval time.
Public Make schedule source: https://help.make.com/schedule-a-scenario documents schedule options and scenario rate limits.
Public Make operation source: https://help.make.com/operations documents operations, bundles, and per-module accounting.

## §1 Counterpart 1 Surface — n8n

1. n8n is a visual node-based workflow automation platform.
2. n8n's public integration page reports connectivity with more than 1000 apps and services.
3. n8n's public workflow page reports 9752 workflow automation templates at retrieval time.
4. n8n documentation says integrations are nodes and nodes are workflow building blocks.
5. n8n documentation says workflows can connect multiple nodes to build complex workflows.
6. n8n supports built-in nodes.
7. n8n supports community nodes.
8. n8n supports credential-only nodes for HTTP Request operations.
9. n8n supports generic HTTP Request integration for services without dedicated nodes.
10. n8n supports workflow templates for starting from existing automation examples.
11. n8n supports a custom template library for self-hosted environments.
12. n8n template API exposes template metadata and importable workflow data through separate endpoints.
13. n8n queue mode separates the main process, Redis broker, database, and worker processes (counterpart-fact: n8n queue-mode docs).
14. n8n queue mode allows scaling up by adding workers and scaling down by removing workers.
15. n8n queue mode requires Redis and a database (counterpart-fact: n8n queue-mode docs).
16. n8n recommends Postgres 13+ for queue mode.
17. n8n queue mode does not recommend SQLite.
18. n8n worker concurrency defaults to 10.
19. n8n recommends worker concurrency of 5 or higher.
20. n8n self-hosted concurrency control can use `N8N_CONCURRENCY_PRODUCTION_LIMIT`.
21. n8n concurrency control queues over-limit production executions FIFO.
22. n8n supports webhook processors as an optional scaling layer.
23. n8n supports multiple main processes in some self-hosted enterprise modes.
24. n8n's strongest product advantage against Workflow Studio today is mature integration and template volume.
25. n8n's strongest self-host advantage is a documented queue-mode architecture.
26. n8n's strongest developer advantage is a broad node ecosystem and HTTP escape hatch.
27. n8n's likely weakness against canonical Workflow Studio is lack of native Oyatie policy, audit-chain, tenant-class, and jurisdiction-pack semantics.
28. Local Oyatie artifacts already acknowledge n8n as a primary comparison target.
29. `migration-playbooks/from-n8n.md` proves migration from n8n is a first-class local concern.
30. `competitor-parity-matrix.md:48-58` compares visual authoring and round-trip properties against n8n.
31. `competitor-parity-matrix.md:80-88` compares LLM-assist surfaces against n8n.
32. `competitor-parity-matrix.md:90-100` compares performance and scale, but those local rows need source-backed refresh.
33. Oyatie parity requirement: import/export n8n workflows with explicit unsupported-node diagnostics.
34. Oyatie parity requirement: visual node editing for business users.
35. Oyatie parity requirement: advanced graph editing for technical users.
36. Oyatie parity requirement: template browsing and template installation.
37. Oyatie parity requirement: self-host and cloud deployment models.
38. Oyatie parity requirement: queue-aware execution or clean handoff to workflow-engine execution queues.
39. Oyatie parity requirement: custom node SDK or contract-driven node generation.
40. Oyatie advantage target: cryptographic audit-chain emission on authoring events.
41. Oyatie advantage target: Cedar-backed license and policy gates.
42. Oyatie advantage target: jurisdiction overlays directly visible on the canvas.
43. Oyatie advantage target: byte-identical DSL round-trip.
44. Oyatie advantage target: native templates for regulated tenant workflows.
45. Oyatie gap today: no canonical Rust/Leptos implementation surface under top-level `src/`.
46. Oyatie gap today: current Svelte/TypeScript assets conflict with the accepted Leptos direction.
47. Oyatie gap today: no OpenTofu deployment modules for all six contexts.
48. Oyatie gap today: no tenant_class schema binding.
49. Oyatie gap today: current benchmark file includes stale old class labels.
50. n8n parity verdict: product concept aligns, ecosystem parity is behind, governance differentiation is promising but not yet fully wired.

## §2 Counterpart 2 Surface — Zapier

1. Zapier is an automation platform built around workflows called Zaps.
2. Zapier public pricing material says it offers integrations for more than 8000 apps.
3. Zapier describes an app as a web service or application connected through its platform.
4. Zapier allows unlimited Zaps, Tables, and Forms subject to task limits in current pricing copy.
5. Zapier supports two-step Zaps on Free.
6. Zapier supports multi-step Zaps on paid plans.
7. Zapier supports filters, paths, data formatting, trigger scheduling, and other workflow features.
8. Zapier supports Webhooks by Zapier for custom integration.
9. Zapier supports unlimited premium apps on paid plans in current pricing copy.
10. Zapier supports shared Zaps and folders in team plans.
11. Zapier supports shared app connections in team plans.
12. Zapier supports unlimited users in Enterprise pricing copy.
13. Zapier supports advanced admin permissions and app controls in Enterprise pricing copy.
14. Zapier Copilot can create Zaps, generate code steps, map fields, and troubleshoot errors.
15. Zapier AI fields can enrich Table records with OpenAI account connectivity.
16. Zapier MCP is available to all accounts and consumes two tasks per tool call.
17. Zapier tasks count successful action steps, not triggers.
18. Zapier trigger/filter/certain built-in tool steps do not count as tasks.
19. Zapier pay-per-task billing applies after plan task limits in current pricing copy.
20. Zapier private app rate limits differ by plan.
21. Zapier general step limit is 100 steps, including steps within paths.
22. Zapier action-step field limit is 1000 fields.
23. Zapier instant triggers are rate-limited at 20000 requests per 5 minutes per user.
24. Zapier polling triggers depend on app and plan.
25. Zapier polling intervals are 15 minutes on Free, 2 minutes on Professional, 1 minute on Team, and 1 minute on Enterprise.
26. Zapier flood protection holds polling-trigger events when 100 or more items trigger at once.
27. Zapier replay processes confirmed held Zap runs at 1 per second.
28. Zapier does not support native two-way sync; it recommends paired one-way Zaps if needed.
29. Zapier's strongest product advantage is the breadth of app integrations and beginner-friendly automation.
30. Zapier's strongest enterprise advantage is admin controls, shared connections, and app governance.
31. Zapier's strongest adjacent product advantage is Tables, Forms, Interfaces-like workflow adjuncts, and MCP.
32. Zapier's likely weakness against canonical Workflow Studio is limited native source-code ownership, local deployability, and deep policy/audit integration.
33. Oyatie parity requirement: no-code workflow authoring with minimal setup.
34. Oyatie parity requirement: trigger/action mental model exposed clearly.
35. Oyatie parity requirement: filters and conditional branches.
36. Oyatie parity requirement: business-user forms or input collection.
37. Oyatie parity requirement: shared workspace semantics.
38. Oyatie parity requirement: shared connectors or app credentials with strict policy gates.
39. Oyatie parity requirement: admin approval for sensitive actions.
40. Oyatie parity requirement: private integrations or custom nodes.
41. Oyatie parity requirement: rate-limit and flood-protection behavior visible before publish.
42. Oyatie advantage target: private deployment in all six contexts, including on-prem and colo.
43. Oyatie advantage target: policy preview before workflow save.
44. Oyatie advantage target: jurisdiction-pack overlays as first-class editor affordances.
45. Oyatie advantage target: audit-chain proof for each editor action and workflow publish.
46. Oyatie advantage target: tenant_class-aware usage caps without feature-quality degradation.
47. Oyatie gap today: tenant_class is not present in core contracts.
48. Oyatie gap today: user-facing onboarding still teaches old class-gated feature availability.
49. Oyatie gap today: no explicit Zapier MCP equivalent surface is documented for Workflow Studio.
50. Zapier parity verdict: app ecosystem and ease-of-use parity are behind; governance, deployability, and compliance differentiation are plausible but need implementation wiring.

## §3 Counterpart 3 Surface — Make (Integromat)

1. Make is a visual automation platform built around scenarios.
2. Make public integration material reports 3479 app results at retrieval time.
3. Make describes connecting favorite apps with clicks and custom business processes.
4. Make describes AI automation and agentic workflow integrations.
5. Make public pricing material exposes maximum active scenarios as a plan dimension.
6. Make public pricing material exposes minimum interval between scheduled scenarios as a plan dimension.
7. Make pricing page shows 2 active scenarios for the free entry plan and unlimited active scenarios for paid plans at retrieval time.
8. Make pricing page shows 15-minute minimum interval for entry and 1-minute minimum interval for paid plans at retrieval time.
9. Make scenarios can run at regular intervals, once daily, weekdays, weekly, monthly, specified dates, or on demand.
10. Make schedule documentation says on-demand scenarios wait for an API call or manual run.
11. Make schedule documentation supports start and end dates.
12. Make schedule documentation supports rate limits for instant triggers.
13. Make rate-limit example uses a maximum of 30 runs per minute.
14. Make returns HTTP 429 when a webhook response module exceeds configured rate limit.
15. Make uses credits as the billing unit.
16. Make says non-AI features preserve 1 operation to 1 credit.
17. Make says operations refer to individual module runs that process or check data.
18. Make says an operation is a single module run to process data or check for new data.
19. Make says modules process bundles separately, and each bundle can trigger a module run.
20. Make example reduces operations from 11 to 3 by aggregating messages.
21. Make first-scenario help says each team can create up to 100 scenarios per day.
22. Make first-scenario help says each scenario has a 2 MB size limit.
23. Make's strongest product advantage is visual scenario modeling with router-style branching and clear operation economics.
24. Make's strongest cost-transparency advantage is operations/credits visibility at module granularity.
25. Make's strongest rate-control advantage is user-configurable maximum runs per minute for instant-trigger scenarios.
26. Make's likely weakness against canonical Workflow Studio is lack of native Oyatie audit-chain, Cedar policy, and all-six-context deployability.
27. Oyatie parity requirement: visual scenario/workflow builder with module-level accounting.
28. Oyatie parity requirement: schedule intervals and on-demand execution.
29. Oyatie parity requirement: explicit bundle/fan-out accounting or equivalent preview.
30. Oyatie parity requirement: module-level operation count preview before publish.
31. Oyatie parity requirement: app and template browsing at meaningful scale.
32. Oyatie parity requirement: rate limiting per trigger/workflow.
33. Oyatie parity requirement: 429 behavior or equivalent backpressure when webhook response is overloaded.
34. Oyatie parity requirement: scenario size and complexity guards.
35. Oyatie parity requirement: scenario creation quota or abuse guard.
36. Oyatie advantage target: deployment-context-aware scaling overlays.
37. Oyatie advantage target: tenant_class usage caps that align to billing and substrate cost.
38. Oyatie advantage target: audit-chain event proof of rate-limit changes and publish actions.
39. Oyatie advantage target: compliance pack overlays on modules and connectors.
40. Oyatie advantage target: deterministic DSL round-trip and canonical JSON.
41. Oyatie gap today: no module-level credit/operation preview in contracts.
42. Oyatie gap today: capacity model has scale classes but not tenant_class overlays.
43. Oyatie gap today: old benchmark rows use stale commercial class terms.
44. Oyatie gap today: no Make import playbook exists; only n8n migration is visible.
45. Oyatie gap today: current template library count is 25 definitions, far behind Make's public app breadth.
46. Oyatie gap today: no public app-count goal is bound in local docs.
47. Oyatie gap today: no explicit rate-limit UX is cited in PRD acceptance criteria.
48. Oyatie gap today: no 2 MB or equivalent graph-size guard is in the contracts.
49. Oyatie gap today: no operation-accounting cost preview is visible in OpenAPI or protobuf.
50. Make parity verdict: visual modeling and operation accounting are the largest union gaps; governance and deployment controls are the largest Oyatie advantage opportunities.

## §4 UNION-Coverage Matrix

1. Capability: visual graph editor. n8n: yes. Zapier: yes, as Zap editor. Make: yes, as scenario builder. Oyatie: intended by `PRD.md:23-35`; implementation path conflicted by Svelte versus Leptos. Status: partial.
2. Capability: business-user no-code authoring. n8n: yes. Zapier: yes. Make: yes. Oyatie: intended by onboarding and PRD; onboarding uses retired class vocabulary. Status: partial.
3. Capability: developer-oriented custom code nodes. n8n: yes. Zapier: code steps and developer platform. Make: custom apps and HTTP. Oyatie: planned through custom-node SDK, but current TS reference conflicts with Rust-strict direction. Status: gap.
4. Capability: large app integration catalog. n8n: more than 1000 services. Zapier: more than 8000 apps. Make: 3479 apps observed. Oyatie: no comparable app-count artifact; templates show 25 internal workflows. Status: gap.
5. Capability: template library. n8n: 9752 templates observed. Zapier: templates exist through product pages, not counted in the opened source. Make: templates and apps exist, count not established in opened source. Oyatie: 25 template definitions plus explainers and fixtures. Status: partial.
6. Capability: import from counterpart. n8n: native import/export common in docs. Zapier: Transfer exists for historical data, not workflow import. Make: scenario import/export not established in opened source. Oyatie: `migration-playbooks/from-n8n.md` only. Status: partial.
7. Capability: export to canonical workflow DSL. n8n: workflow JSON. Zapier: Zap model proprietary. Make: scenario model proprietary. Oyatie: canonical JSON and byte-equality via `ADR-WS-0002`. Status: advantage if implemented.
8. Capability: byte-identical round-trip. n8n: not established. Zapier: not established. Make: not established. Oyatie: explicit in `PRD.md:37-47` and `ADR-WS-0002`. Status: advantage target.
9. Capability: collaborative editing. n8n: limited/plan-dependent per local matrix. Zapier: team sharing. Make: team roles and permissions. Oyatie: CRDT SLOs and collab docs, but ADR conflict between Loro and Yjs. Status: partial.
10. Capability: CRDT explicitness. n8n: not primary public surface. Zapier: not primary public surface. Make: not primary public surface. Oyatie: `ADR-WS-0001` chooses Loro while proposed `ADR-WFS-001` chooses Yjs. Status: gap due conflict.
11. Capability: queue-backed execution scale. n8n: documented queue mode. Zapier: SaaS managed. Make: SaaS managed with scenario limits. Oyatie: execution should hand off to workflow-engine, but handoff doc absent. Status: partial.
12. Capability: worker concurrency control. n8n: worker default 10 and recommended 5+. Zapier: task/rate limits. Make: maximum runs per minute. Oyatie: capacity model has replica formulas but no editor-visible limit contract. Status: partial.
13. Capability: trigger polling intervals. n8n: schedule/trigger model exists. Zapier: 15, 2, 1, 1 minutes by plan. Make: 15 and 1 minute intervals in public pricing source. Oyatie: no explicit polling interval contract. Status: gap.
14. Capability: instant triggers. n8n: webhooks and processors. Zapier: instant triggers via webhooks. Make: instant triggers with rate limits. Oyatie: event contracts exist, trigger UX not explicit. Status: partial.
15. Capability: webhooks. n8n: yes. Zapier: Webhooks by Zapier. Make: webhooks and HTTP 429 behavior. Oyatie: OpenAPI and AsyncAPI exist; public webhook authoring not clearly modeled. Status: partial.
16. Capability: scheduled workflows. n8n: yes. Zapier: schedule features. Make: rich schedule options. Oyatie: not explicit in Studio contracts; execution likely workflow-engine. Status: gap.
17. Capability: conditional paths. n8n: nodes can branch. Zapier: paths/filter logic. Make: routers/filters. Oyatie: template graphs include terminals, but condition UX not audited in contracts. Status: partial.
18. Capability: filters. n8n: nodes. Zapier: Filter by Zapier. Make: filters. Oyatie: not a clearly documented editor primitive. Status: gap.
19. Capability: data formatting. n8n: nodes. Zapier: Formatter by Zapier. Make: modules. Oyatie: DSL validation exists; formatter library not explicit. Status: gap.
20. Capability: tables/forms adjuncts. n8n: forms and nodes. Zapier: Tables and Forms. Make: app modules and forms via integrations. Oyatie: templates and workflow authoring, no adjunct table/form product in Studio docs. Status: gap.
21. Capability: AI workflow generation. n8n: AI workflows and templates. Zapier: Copilot creates Zaps and maps fields. Make: AI apps and agentic integrations. Oyatie: LLM-assist planned and bounded. Status: partial.
22. Capability: human approval for AI output. n8n: not established. Zapier: user-driven Copilot. Make: not established. Oyatie: `ADR-WS-0005` forbids auto-submit and requires explicit acceptance. Status: advantage target.
23. Capability: prompt-injection safeguards. n8n: external hardening variable. Zapier: not established in opened source. Make: not established in opened source. Oyatie: OpenAPI refuses suspicious prompts. Status: advantage target.
24. Capability: policy preview. n8n: not native. Zapier: enterprise app controls. Make: team roles and controls. Oyatie: Cedar preview planned. Status: partial.
25. Capability: audit trail. n8n: execution history. Zapier: history and enterprise logs. Make: audit logs in pricing. Oyatie: audit-chain emission required. Status: advantage target.
26. Capability: cryptographic audit proof. n8n: not established. Zapier: not established. Make: not established. Oyatie: audit-chain Merkle/Ed25519 references. Status: advantage target.
27. Capability: jurisdiction-aware canvas overlays. n8n: not established. Zapier: not native. Make: not native. Oyatie: `ADR-WS-0004` and PRD require overlays. Status: advantage target.
28. Capability: data residency pack routing. n8n: self-host controls location. Zapier: SaaS. Make: SaaS and enterprise controls. Oyatie: policy docs have pack routing but still reference foundry-providers. Status: partial.
29. Capability: BYO provider for AI. n8n: custom credentials. Zapier: OpenAI account for AI fields. Make: third-party AI apps. Oyatie: provider path through foundry-providers/intelligence providers. Status: partial.
30. Capability: self-host. n8n: yes. Zapier: no for main SaaS. Make: no standard self-host in opened source. Oyatie: intended across all six contexts, but modules absent. Status: gap.
31. Capability: on-prem. n8n: self-host possible. Zapier: not standard. Make: on-prem agents exist as adjacent link, not opened deeply. Oyatie: required by canonical six-context list. Status: gap.
32. Capability: colo. n8n: self-host possible. Zapier: not standard. Make: not standard. Oyatie: required by canonical six-context list. Status: gap.
33. Capability: guest cloud on AWS. n8n: self-host possible. Zapier: SaaS. Make: SaaS. Oyatie: required path absent. Status: gap.
34. Capability: guest cloud on OCI. n8n: self-host possible. Zapier: SaaS. Make: SaaS. Oyatie: required path and Always Free profile absent. Status: gap.
35. Capability: provider-agnostic deployment. n8n: self-host, but app itself not six-context doctrine. Zapier: no. Make: no. Oyatie: canonical requirement, not wired. Status: gap.
36. Capability: tenant_class-aware caps. n8n: plan quotas. Zapier: task quotas. Make: credit/operation quotas. Oyatie: canonical replacement model, not adopted. Status: gap.
37. Capability: usage-based billing. n8n: execution quota/overage. Zapier: task allowance/pay-per-task. Make: credits. Oyatie: paid and revenue_share should map to usage economics, but not in Studio contracts. Status: gap.
38. Capability: revenue-share tenant model. n8n: not native. Zapier: not native. Make: not native. Oyatie: canonical class, no service adoption. Status: gap.
39. Capability: free demo profile. n8n: hosted trials/self-host free. Zapier: Free plan. Make: free entry plan. Oyatie: demo_trial should map to OCI Always Free profile, but module absent. Status: gap.
40. Capability: contractual SLO. n8n: enterprise likely, not opened. Zapier: enterprise. Make: enterprise. Oyatie: OpenSLO files exist. Status: partial.
41. Capability: compliance packs. n8n: self-host/security docs. Zapier: enterprise controls. Make: enterprise compliance. Oyatie: compliance docs are extensive. Status: partial.
42. Capability: BYOK. n8n: not established. Zapier: not established. Make: not established. Oyatie: canonical paid class allows BYOK, but Studio docs do not bind it. Status: gap.
43. Capability: node-library signing. n8n: community nodes. Zapier: app platform review. Make: app submission. Oyatie: OpenBao and Ed25519 signing references, but Terraform path. Status: partial.
44. Capability: app submission/creator program. n8n: creator/template submission. Zapier: developer platform. Make: app partnership submission. Oyatie: marketplace and publisher model are partial. Status: partial.
45. Capability: private app/integration. n8n: custom/community nodes. Zapier: private apps. Make: custom apps. Oyatie: contract-driven custom nodes planned, implementation unresolved. Status: partial.
46. Capability: field mapping UX. n8n: visual node configuration. Zapier: Copilot maps fields. Make: module configuration. Oyatie: not deeply documented. Status: gap.
47. Capability: debug execution history. n8n: executions list. Zapier: Zap history. Make: run history. Oyatie: replay debugger planned. Status: partial.
48. Capability: time-travel debugging. n8n: not established. Zapier: not established. Make: replay/run details. Oyatie: planned in stale old class docs, not cleanly canonical. Status: partial.
49. Capability: versioning. n8n: workflow versions depending deployment. Zapier: Zap history. Make: scenario history. Oyatie: in-studio versioning mentioned, not contract-bound. Status: partial.
50. Capability: rollback. n8n: workflow exports/backup patterns. Zapier: replay/held runs. Make: scenario run history. Oyatie: runbooks and Foundry self-modification doctrine, not Studio-specific. Status: partial.
51. Capability: app governance/admin controls. n8n: enterprise features. Zapier: enterprise app controls. Make: roles/audit logs. Oyatie: Cedar and policy docs. Status: partial.
52. Capability: granular connector permissions. n8n: credentials/projects. Zapier: app controls and shared connections. Make: team roles/connections. Oyatie: policy docs imply but contract details incomplete. Status: partial.
53. Capability: rate-limit preview. n8n: concurrency variables. Zapier: limits documented. Make: maximum runs per minute. Oyatie: no editor preview evidence. Status: gap.
54. Capability: flood protection. n8n: concurrency/queue. Zapier: holds 100+ polling events. Make: queues at configured rate. Oyatie: not visible in UX contract. Status: gap.
55. Capability: cost preview. n8n: execution-based. Zapier: task-based. Make: credit/operation-based. Oyatie: FinOps docs but no authoring preview. Status: gap.
56. Capability: module fan-out accounting. n8n: execution logs. Zapier: task count per action. Make: operations and bundles. Oyatie: not explicit. Status: gap.
57. Capability: template provenance. n8n: creators. Zapier: templates. Make: app/templates. Oyatie: template fixtures and explainers, provenance is local. Status: partial.
58. Capability: template quarantine. n8n: not opened. Zapier: app controls. Make: app governance. Oyatie: runbook exists. Status: partial.
59. Capability: localization. n8n: product localization not opened. Zapier: global SaaS. Make: global SaaS. Oyatie: FTL sources for Arabic, Korean, source locale. Status: partial.
60. Capability: mobile/native editor. n8n: web. Zapier: web/mobile app not opened. Make: web. Oyatie: Swift/Kotlin/WinUI/GTK IPs exist, but OS manifest absent. Status: partial.
61. Capability: desktop editor. n8n: web. Zapier: web. Make: web. Oyatie: WinUI and GTK IPs, not manifest-bound. Status: partial.
62. Capability: offline draft. n8n: not opened. Zapier: SaaS. Make: SaaS. Oyatie: reliability gate in `SCOPE.md:2055`. Status: partial.
63. Capability: canvas frame budget. n8n: not opened. Zapier: not opened. Make: not opened. Oyatie: p99 frame OpenSLO. Status: advantage target.
64. Capability: save latency budget. n8n: not opened. Zapier: not opened. Make: not opened. Oyatie: editor REST p99 save SLO. Status: advantage target.
65. Capability: cold-load budget. n8n: not opened. Zapier: SaaS. Make: SaaS. Oyatie: TTI SLO. Status: advantage target.
66. Capability: 1000-node graph target. n8n: complex workflows possible. Zapier: 100-step limit. Make: 2 MB scenario size. Oyatie: 1000-node canvas SLO. Status: advantage target.
67. Capability: graph size hard ceiling. n8n: not source-backed here. Zapier: 100 steps. Make: 2 MB scenario. Oyatie: 1000-node target and 5000-node exploratory target, but old benchmark wording needs cleanup. Status: partial.
68. Capability: autoscaling authoring plane. n8n: worker scaling. Zapier: SaaS managed. Make: SaaS managed. Oyatie: capacity formulas, no OpenTofu context modules. Status: partial.
69. Capability: API readiness endpoint. n8n: health/config docs. Zapier: SaaS. Make: SaaS. Oyatie: OpenAPI readiness endpoint checks dependencies. Status: partial.
70. Capability: dependency readiness. n8n: Redis/database (counterpart-fact: n8n queue-mode docs). Zapier: SaaS. Make: SaaS. Oyatie: readiness checks foundry-providers SDK, which is stale under Foundry retirement. Status: gap.
71. Capability: app ecosystem migration. n8n: source system. Zapier: source system candidate. Make: source system candidate. Oyatie: only from-n8n playbook exists. Status: partial.
72. Capability: counterpart migration diagnostics. n8n: Oyatie has playbook. Zapier: absent. Make: absent. Oyatie: gap for two required counterparts. Status: gap.
73. Capability: public claim discipline. n8n: public docs. Zapier: public docs. Make: public docs. Oyatie: competitor matrix has claim boundary rules. Status: partial.
74. Capability: source-backed benchmark discipline. n8n: public data available. Zapier: public limits available. Make: public limits available. Oyatie: existing benchmark doc has stale rows. Status: gap.
75. Capability: Foundry workflow-ui absorption. n8n: not relevant. Zapier: not relevant. Make: not relevant. Oyatie: required special dimension, not incorporated by name. Status: gap.
76. Capability: `oyatie.foundry.*` principal routing. n8n: not relevant. Zapier: not relevant. Make: not relevant. Oyatie: canonical doctrine exists, Studio artifacts lack local binding. Status: gap.
77. Capability: Cedar policy inheritance. n8n: not native. Zapier: app/admin controls. Make: roles. Oyatie: policy files exist but not Foundry-inheritance-specific. Status: partial.
78. Capability: self-modification workflow visibility. n8n: not opened. Zapier: not opened. Make: not opened. Oyatie: ADR-0247 workflow-engine level, not Studio UI. Status: gap.
79. Capability: editor embeds for B2B2C. n8n: embeddable not opened. Zapier: Interfaces and embeds not opened. Make: embedded not opened. Oyatie: FAQ mentions embedding but under old class label. Status: partial.
80. Capability: revenue-share seller workflows. n8n: not native. Zapier: not native. Make: not native. Oyatie: canonical class exists, Studio absent. Status: gap.
81. Capability: regulated templates. n8n: templates broad. Zapier: templates broad. Make: templates broad. Oyatie: hospital, HR, payroll, operations, hiring templates. Status: partial.
82. Capability: template schema. n8n: template API schemas documented. Zapier: not opened. Make: not opened. Oyatie: local JSON schema exists. Status: partial.
83. Capability: template fixtures. n8n: not opened. Zapier: not opened. Make: not opened. Oyatie: local fixtures for every template. Status: advantage target.
84. Capability: OpenAPI contract. n8n: API docs. Zapier: developer docs. Make: developer hub. Oyatie: OpenAPI exists. Status: partial.
85. Capability: AsyncAPI contract. n8n: not primary. Zapier: not primary. Make: not primary. Oyatie: AsyncAPI exists. Status: advantage target.
86. Capability: protobuf contract. n8n: not primary. Zapier: not primary. Make: not primary. Oyatie: proto exists. Status: advantage target.
87. Capability: SDK plan. n8n: API clients/community. Zapier: developer platform. Make: developers hub. Oyatie: SDK plan exists but contains Python sample concern. Status: partial.
88. Capability: no-code first-week onboarding. n8n: tutorials. Zapier: help docs. Make: first scenario guide. Oyatie: onboarding exists but old class labels. Status: partial.
89. Capability: incident response. n8n: self-host docs, support. Zapier: SaaS. Make: status/support. Oyatie: incident playbook exists. Status: partial.
90. Capability: DPIA/compliance. n8n: security docs. Zapier: enterprise compliance. Make: enterprise compliance. Oyatie: detailed compliance/DPIA. Status: partial.
91. Capability: cost budget. n8n: self-host TCO. Zapier: task pricing. Make: credits. Oyatie: cost budget exists but old scale labels and OCI-specific leakage. Status: partial.
92. Capability: pack-specific compliance. n8n: self-host. Zapier: enterprise. Make: enterprise. Oyatie: pack overlays in compliance docs. Status: partial.
93. Capability: multi-region. n8n: self-host queue. Zapier: SaaS managed. Make: SaaS managed. Oyatie: multi-region doc exists. Status: partial.
94. Capability: all-six deployment contexts. n8n: self-host flexible. Zapier: no. Make: no. Oyatie: canonical requirement, missing modules. Status: gap.
95. Capability: OCI Always Free profile. n8n: can be self-hosted by user. Zapier: no. Make: no. Oyatie: canonical demo_trial infrastructure profile, missing. Status: gap.
96. Capability: OpenTofu-only IaC. n8n: not applicable to product surface. Zapier: SaaS. Make: SaaS. Oyatie: required, current Terraform present. Status: gap.
97. Capability: OS matrix. n8n: web/self-host. Zapier: web. Make: web. Oyatie: canonical OS manifest absent. Status: gap.
98. Capability: Leptos/Rust web. n8n: JS stack. Zapier: web stack. Make: web stack. Oyatie: accepted ADR, current Svelte conflict. Status: gap.
99. Capability: native client allowlist. n8n: web. Zapier: not opened. Make: web. Oyatie: Swift/Kotlin/WinUI IPs; GTK also appears and needs policy review. Status: partial.
100. Capability: codebase hygiene. n8n: mature. Zapier: mature SaaS. Make: mature SaaS. Oyatie: manifest stale, docs contradictory. Status: gap.
101. Capability: manifest/index accuracy. n8n: live site. Zapier: live site. Make: live site. Oyatie: manifest omits IP-016 through IP-027 and journeys. Status: gap.
102. Capability: anti-scaffold compliance. n8n: production docs. Zapier: production docs. Make: production docs. Oyatie: architecture file has generated sweep marker. Status: partial.
103. Capability: open-question closure. n8n: docs current. Zapier: docs current. Make: docs current. Oyatie: PRD open questions conflict with decision index. Status: gap.
104. Capability: exact deliverable verification. n8n: public pages. Zapier: public pages. Make: public pages. Oyatie: this audit performs line/count/content validation after write. Status: audit-only.
105. Capability: counterpart breadth. n8n: primary. Zapier: primary. Make: primary. Oyatie: current docs also include Workato and others. Status: acceptable, top-3 narrowed for this batch.
106. Capability: regulated authoring differentiator. n8n: general automation. Zapier: general automation. Make: general automation. Oyatie: strongest unique value if implemented. Status: partial.
107. Capability: enterprise admin. n8n: enterprise. Zapier: enterprise. Make: teams/enterprise. Oyatie: Cedar and policy docs. Status: partial.
108. Capability: marketplace. n8n: template creator program developing. Zapier: app ecosystem. Make: app ecosystem. Oyatie: template marketplace planned, stale old labels. Status: partial.
109. Capability: developer submission review. n8n: community nodes/templates. Zapier: developer platform. Make: submit app/partner route. Oyatie: publisher allowlist via Terraform/OpenBao. Status: partial.
110. Capability: signed custom nodes. n8n: not opened. Zapier: app review. Make: app review. Oyatie: Ed25519 key references. Status: advantage target.
111. Capability: SRI for WASM chunks. n8n: not opened. Zapier: SaaS internal. Make: SaaS internal. Oyatie: PRD acceptance and cost/security docs mention SRI. Status: partial.
112. Capability: CDN purge SLI. n8n: self-host. Zapier: SaaS. Make: SaaS. Oyatie: DPIA mitigation references CDN purge SLI. Status: partial.
113. Capability: XSS hardening. n8n: security. Zapier: SaaS. Make: SaaS. Oyatie: CSP and Trusted Types docs. Status: partial.
114. Capability: cross-tenant isolation. n8n: projects/credentials. Zapier: account/app controls. Make: org/team controls. Oyatie: Citus/RLS/Cedar docs. Status: partial.
115. Capability: per-seat licensing. n8n: plans. Zapier: users/tasks. Make: teams/credits. Oyatie: PRD license gate; tenant_class model not adopted. Status: partial.
116. Capability: usage metering. n8n: executions. Zapier: tasks. Make: credits/operations. Oyatie: FinOps and capacity docs; no editor-visible per-workflow preview. Status: gap.
117. Capability: generated app nodes from contracts. n8n: nodes ecosystem. Zapier: app platform. Make: app platform. Oyatie: auto-generated nodes from µservice contracts in FAQ, but stale old labels. Status: partial.
118. Capability: migration from Zapier. n8n: not relevant. Zapier: source. Make: not relevant. Oyatie: no playbook. Status: gap.
119. Capability: migration from Make. n8n: not relevant. Zapier: not relevant. Make: source. Oyatie: no playbook. Status: gap.
120. Capability: migration from n8n. n8n: source. Zapier: not relevant. Make: not relevant. Oyatie: playbook exists but old labels. Status: partial.
121. Capability: air-gapped authoring. n8n: self-host possible. Zapier: no standard. Make: no standard. Oyatie: on-prem/colo required but modules absent. Status: gap.
122. Capability: customer cloud deployment. n8n: self-host possible. Zapier: no standard. Make: no standard. Oyatie: required but modules absent. Status: gap.
123. Capability: public cloud SaaS. n8n: hosted cloud. Zapier: SaaS. Make: SaaS. Oyatie: intended but context module absent. Status: gap.
124. Capability: cloud-provider offering. n8n: no. Zapier: SaaS. Make: SaaS. Oyatie: `oyatie-as-cloud-provider` required but module absent. Status: gap.
125. Capability: compliance pack allowed. n8n: self-host/security. Zapier: enterprise. Make: enterprise. Oyatie: compliance docs strong; tenant_class binding missing. Status: partial.
126. Capability: best-effort demo SLO. n8n: free/self-host. Zapier: free plan. Make: free plan. Oyatie: canonical demo_trial class, not service-bound. Status: gap.
127. Capability: contractual paid SLO. n8n: enterprise. Zapier: enterprise. Make: enterprise. Oyatie: OpenSLO files, no tenant_class linkage. Status: partial.
128. Capability: at-cost revenue-share substrate. n8n: not native. Zapier: not native. Make: not native. Oyatie: canonical model, no service adoption. Status: gap.
129. Capability: authoring authorization. n8n: auth/credentials. Zapier: shared connections. Make: roles. Oyatie: Cedar policies. Status: partial.
130. Capability: auditor read scope. n8n: not opened. Zapier: enterprise. Make: audit logs. Oyatie: `policy/auditor-scope.cedar`. Status: partial.
131. Capability: public template read. n8n: public workflows. Zapier: templates. Make: app directory. Oyatie: `policy/public-read.cedar`. Status: partial.
132. Capability: CI scope. n8n: open-source CI. Zapier: internal. Make: internal. Oyatie: `policy/ci-scope.cedar`, but owner includes stale foundry label. Status: partial.
133. Capability: data residency. n8n: self-host location. Zapier: SaaS. Make: SaaS. Oyatie: policy doc exists, stale provider naming. Status: partial.
134. Capability: editor isolation. n8n: projects/users. Zapier: app controls. Make: teams. Oyatie: policy doc exists, old label drift. Status: partial.
135. Capability: route to execution engine. n8n: internal execution. Zapier: internal. Make: internal. Oyatie: cross-service workflow-engine handoff required but not consolidated. Status: partial.
136. Capability: route to ontology. n8n: no equivalent. Zapier: tables/fields. Make: data mapping. Oyatie: ontology integration in PRD. Status: advantage target.
137. Capability: route to application service. n8n: app nodes. Zapier: apps. Make: apps. Oyatie: application dependency in PRD. Status: partial.
138. Capability: route to tenancy. n8n: projects. Zapier: accounts. Make: orgs. Oyatie: tenancy dependency, no tenant_class contract. Status: partial.
139. Capability: route to intelligence providers. n8n: AI nodes. Zapier: AI fields/Copilot. Make: AI apps. Oyatie: old foundry-providers references need migration. Status: partial.
140. Capability: self-modification dashboard. n8n: not relevant. Zapier: not relevant. Make: not relevant. Oyatie: expected through workflow-engine/dev-tools, absent in Studio. Status: gap.

## §5 Family Summary

1. n8n sets the strongest bar for self-hostable, technical-user workflow automation with a large template catalog.
2. Zapier sets the strongest bar for app ecosystem breadth, beginner-friendly task automation, and business-user adoption.
3. Make sets the strongest bar for visual scenario modeling, operation accounting, and rate-control clarity.
4. Workflow Studio's strongest intended differentiator is policy-aware regulated workflow authoring, not raw app count.
5. Workflow Studio's second differentiator is deterministic canonical DSL emission with byte-identical round-trip.
6. Workflow Studio's third differentiator is cryptographic audit-chain evidence across authoring and publishing.
7. Workflow Studio's fourth differentiator is all-six deployment contexts, but current IaC does not support that claim.
8. Workflow Studio's fifth differentiator is tenant_class-aware usage and billing, but current contracts do not express it.
9. Current ecosystem parity is behind all three counterparts.
10. Current visual-editor implementation coherence is blocked by the Svelte/TypeScript versus Leptos/Rust conflict.
11. Current enterprise governance parity is partly documented through Cedar, compliance, and audit docs.
12. Current performance parity is not source-backed enough; new benchmark targets must supersede old rows.
13. Current migration surface covers n8n but not Zapier or Make.
14. Current template surface is domain-specific and strong for regulated examples, but small in count.
15. Current capability docs must stop describing features as old commercial class promotions.
16. Current Foundry workflow-ui absorption is not fully represented in service artifacts.
17. Current `workflow-studio` corpus has enough substance to guide implementation but not enough coherence to claim launch readiness.
18. The top gap family is deployment and runtime compliance.
19. The second gap family is ecosystem and migration breadth.
20. The third gap family is no commercial-tier tenant_class adoption.
21. The fourth gap family is Foundry absorption and stale Foundry naming.
22. The fifth gap family is benchmark claim discipline.

## §6 Headline Gap Analysis

1. Gap H1: `workflow-studio` lacks the counterpart app catalog scale; n8n has more than 1000 services, Zapier more than 8000 apps, and Make 3479 apps observed.
2. Gap H2: `workflow-studio` lacks source-backed migration playbooks for Zapier and Make.
3. Gap H3: `workflow-studio` has only 25 template definitions, which is useful but not category-leading.
4. Gap H4: `workflow-studio` has no tenant_class binding even though the replacement model is mandatory.
5. Gap H5: `workflow-studio` has no six-context OpenTofu module set.
6. Gap H6: `workflow-studio` has no OCI Always Free profile for demo_trial infrastructure.
7. Gap H7: `workflow-studio` has no supported OS manifest.
8. Gap H8: `workflow-studio` has hand-authored Svelte/TypeScript files that contradict the Leptos/Rust direction.
9. Gap H9: `workflow-studio` has old commercial class labels in user-facing and benchmark docs.
10. Gap H10: `workflow-studio` has Foundry absorption only by implication, not by direct workflow-ui ownership artifact.
11. Gap H11: `workflow-studio` lacks module-level cost preview comparable to Make's operation accounting.
12. Gap H12: `workflow-studio` lacks flood-protection/rate-limit UX comparable to Zapier and Make.
13. Gap H13: `workflow-studio` lacks explicit trigger polling semantics comparable to Zapier and Make.
14. Gap H14: `workflow-studio` lacks private app/custom node governance clarity after the Rust-strict policy change.
15. Gap H15: `workflow-studio` lacks a current source-backed benchmark harness.
16. Gap H16: `workflow-studio` has conflicting CRDT direction: accepted Loro ADR and proposed Yjs ADR.
17. Gap H17: `workflow-studio` has stale manifest indexing.
18. Gap H18: `workflow-studio` has PRD open questions conflicting with the ADR index.
19. Gap H19: `workflow-studio` has no consolidated cross-microservice handoff doc.
20. Gap H20: `workflow-studio` has README absence despite product complexity.

## §7 Additive Surface Recommended By Union Coverage

1. Add a tenant_class schema surface with `demo_trial`, `paid`, and `revenue_share`.
2. Add a deployment-context manifest showing all six contexts and their OpenTofu module paths.
3. Add an OCI Always Free profile module for demo_trial infrastructure.
4. Add `supported-oses.json` with the canonical OS matrix.
5. Add a Leptos/Rust replacement plan for the current Svelte/TypeScript template browser.
6. Add a Zapier migration playbook.
7. Add a Make migration playbook.
8. Add a no commercial-tier template onboarding guide.
9. Add an operation-cost preview model inspired by Make's bundle and operation accounting.
10. Add trigger polling and instant-trigger behavior docs.
11. Add rate-limit and flood-protection editor affordances.
12. Add a private integration governance model.
13. Add contract-driven custom-node generation that avoids forbidden app languages.
14. Add source-backed benchmark harness docs.
15. Add workflow-ui inherited ownership notes from Foundry dissolution.
16. Add `oyatie.foundry.*` principal-routing notes where Studio presents inherited Foundry workflow UI.
17. Add a cross-microservice handoff matrix.
18. Add a README focused on local development, docs map, and verification commands.
19. Add manifest refresh to index IP-016 through IP-027 and journey IPs.
20. Add CRDT decision reconciliation before implementation proceeds.
21. Add admin approval and sensitive-action publish gates comparable to Zapier enterprise controls.
22. Add shared connector credential policy comparable to Zapier shared connections and Make teams.
23. Add template provenance and signing view.
24. Add template quarantine UI linked to the existing runbook.
25. Add app catalog growth goal with source-backed counterpart baselines.
26. Add dependency readiness rename away from stale foundry-providers labels.
27. Add no commercial-tier benchmark table using a single industry-leader target set with context overlays.
28. Add claim-boundary linting for benchmark and marketing docs.
29. Add performance gates for canvas, save, CRDT, cold load, and publish.
30. Add user-visible workload size warnings mapped to Zapier's 100-step and Make's 2 MB public limits.
