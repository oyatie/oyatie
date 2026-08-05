# Tasks workplace source-backed Plan/Spec backlog

Kanban task: `t_2e92e2f3`
Status: Plan/Spec artifact only; no runtime code, generated JSON, shared manifest-index, cloud/runtime, Release Please, or product-readiness claim.

## Authority inputs

- Source lock: parent `t_516944f9`, created `specs/microservices/tasks.json` as the accepted tasks source-authority lock.
- Root authority: `specs/root-hub-pointers.json` keeps `docs/AGENTS.md` as the operating contract until explicit PHASE-5 promotion evidence.
- Service-index context: `specs/microservices/manifests-index.json` row `name=tasks` points to `oya/tasks/manifest.json`; that row is stale after the source-lock creation because it still says no `specs/microservices/tasks.json` existed before this chain. Treat the row as provenance/boundary context, not readiness evidence.
- Service inventory/provenance: `oya/tasks/manifest.json` names bounded contexts `dependency-graph`, `importers`, `project-list`, `recurrence`, `search-index`, `task-store`, and `view-engine`, plus contracts, capabilities, SLOs, IP references, ontology projections, policy, and catalog paths.
- Layout authority: ADR-0131 lines 24-28/60-63 and ADR-0512 lines 24-27/53-62 keep canonical service-owned paths under `{oya,cloud}/<service>/`, with `microservices/` legacy-only and `oya/tasks/crates/<crate>/` the bounded-context crate home.
- Prior source-map artifact: `/Users/jasonlee/.hermes/kanban/boards/oyatie/workspaces/t_746f01be/tasks-app-source-map-and-backlog.md` established the lifecycle chain and the source-lock prerequisite.

## Source and claim boundary

`specs/microservices/tasks.json` authorizes source-backed Plan/Spec and RED fixture decomposition only. It does not authorize handlers, storage adapters, UI implementation, deployment, live SLO evidence, production readiness, hyperscaler maturity, GA, or customer-facing claims.

Legacy `microservices/tasks/**` strings inside `oya/tasks/manifest.json` are stale provenance/path-discovery inputs. Future work must map them to verified `oya/tasks/**` paths or quarantine them as unresolved. No downstream card may recreate `microservices/tasks/` or treat those strings as live implementation authority.

Implementation remains held behind this Plan/Spec, then RED fixture/contract gates, then Build, Review/fix, Merge/Rollout/E2E, and Learning/observation harvest.

## Lifecycle plan

1. Research/source lock: complete via `t_516944f9` and `specs/microservices/tasks.json`.
2. Plan/Spec: this artifact defines atomic backlog slices, path/conflict constraints, evidence expectations, non-goals, and bounded-context coverage.
3. RED fixture/contract: child `t_91fc5a1f` must create failing-first guards before any Build card starts.
4. Build: child chain `t_4221a299` may implement only the selected RED-backed slice and must stay inside declared prefixes.
5. Review/fix: child chain `t_6777ef70` must independently check authority, contracts, security/privacy, accessibility/UX, observability/SLO, AI/fairness/no-overclaim, generated-face, and lifecycle evidence.
6. Merge/Rollout/E2E: child chain `t_f77d8125` records protected PR/CI state when applicable, API/contract replay, browser/user-story evidence where UI is touched, rollback, observability, release-governance/release-note impact, and rollout/E2E results.
7. Learning/observation harvest: child chain `t_6f142b12` turns useful observations into linked cards or duplicate notes.

## Global constraints for all slices

- Generated faces: never hand-edit `*.generated.json`.
- Shared authority: do not edit `specs/microservices/manifests-index.json` from these slices; if the stale tasks row needs correction, create a serialized authority-index update card.
- Runtime/cloud: no cloud/runtime/deployment surfaces until a RED-backed Build card explicitly scopes them.
- Contracts: service-local OpenAPI/AsyncAPI/proto are source candidates; contract replay must prove behavior before readiness claims.
- UI evidence: if a slice touches user-facing task flows, it requires browser/user-story and WCAG 2.2 AA accessibility evidence. If no UI is touched, the review must record an explicit N/A rationale.
- API evidence: service-facing changes require API/contract replay or a documented N/A for spec-only work.
- Data/security: task content, comments, attachments, assignee/requester/watcher identities, importer payloads, legal hold, search queries, saved views, webhook payloads, and AI-assist evidence are sensitive. RED and Build cards must include tenant/RBAC/Cedar deny-by-default and cross-tenant negative cases where applicable.
- AI/fairness: T2 auto-assignment or employment-impacting suggestions must require fairness evidence, protected-class non-use, human override, explicit disclosure, and EU AI Act/employment-context refusal.
- Observability/SLO: SLO files and dashboards remain inventory until concrete emitters/replay evidence exist.

## Atomic backlog slices

### Slice TASKS-PS-00: source-map guard and stale-reference quarantine

- Bounded context: cross-cutting source authority; prerequisite for every other slice.
- Conflict class: `tasks-source-map-authority-serialized`.
- Allowed prefixes: `specs/microservices/tasks.json`, `plan/tasks/`, `evidence/tasks/`, and future RED-only `specs/fixtures/tasks/source-map/` or `scripts/tests/tasks_source_map_*.py` after `t_91fc5a1f` selects the fixture path.
- Test/fixture plan: RED source-map guard fails when a future card omits the `specs/microservices/tasks.json` citation, treats `oya/tasks/manifest.json` as readiness evidence, or cites stale legacy `microservices/tasks/**` as a live destination.
- API/contract replay: N/A for source-map-only checks; contract replay expectations are asserted as prerequisites for downstream API slices.
- Browser/user-story/accessibility: N/A; no UI touched. Review must record that this is a planning/source-map guard.
- Non-goals: shared manifests-index edits, runtime code, source-root restoration under `microservices/`, generated JSON.
- Exit: `t_91fc5a1f` has a failing guard that proves stale/uncited fanout is rejected.

### Slice TASKS-PS-01: task-store lifecycle contract

- Bounded context: `task-store`.
- Conflict class: `tasks-task-store-contract-and-domain`.
- Allowed prefixes: `oya/tasks/contracts/openapi/tasks.yaml`, `oya/tasks/contracts/proto/tasks.proto`, `oya/tasks/crates/oya-tasks-domain/`, future `oya/tasks/crates/oya-tasks-task-store*/` only after crate existence/build shape is verified, `oya/tasks/policy/`, `oya/tasks/cedar/`, `oya/tasks/slos/task-create-latency.openslo.yaml`, `oya/tasks/slos/task-update-latency.openslo.yaml`, `plan/tasks/`, and `evidence/tasks/`.
- Test/fixture plan: RED contract fixture for create/read/update/archive/delete; idempotency keys; tenant mismatch denial; legal-hold/retention refusal; audit metadata; RLS/tenant-DEK expectation; malformed/custom field rejection; storage absent/mismatch expected first failure.
- API/contract replay: OpenAPI plus proto replay for task lifecycle and authorization negative cases before handlers can claim correctness.
- Browser/user-story/accessibility: employee creates/edits/archives a task, assigns it, sets due date/priority/labels/custom fields, and sees policy/audit state; keyboard and screen-reader labels/live-region updates required if UI is touched.
- Non-goals: project-board drag/drop, recurrence, search, importer, webhook, AI auto-assign, storage production readiness.
- Exit: one RED-backed build slice can implement minimal lifecycle only after this fixture fails first.

### Slice TASKS-PS-02: project-list and view-engine board/list semantics

- Bounded contexts: `project-list`, `view-engine`.
- Conflict class: `tasks-project-view-ui-contract`.
- Allowed prefixes: `oya/tasks/contracts/openapi/tasks.yaml`, `oya/tasks/contracts/asyncapi/tasks-events.yaml`, `oya/tasks/crates/oya-tasks-domain/`, future `oya/tasks/crates/oya-tasks-project-list*/` and `oya/tasks/crates/oya-tasks-view-engine*/` only after verified, `oya/tasks/policy/`, `oya/tasks/runbooks/`, `oya/tasks/slos/task-list-render-latency.openslo.yaml`, `plan/tasks/`, and `evidence/tasks/`.
- Test/fixture plan: RED fixture for project/list/board/sprint/milestone CRUD, board status transitions, drag/drop optimistic conflict states, collaboration/realtime event semantics, and unauthorized board/list reads.
- API/contract replay: OpenAPI/AsyncAPI replay for project/list/board reads and update events, including stale-view and conflict/error responses.
- Browser/user-story/accessibility: manager views a board, moves a task status, resolves visible blockers, and receives deterministic conflict/error feedback. Keyboard alternatives for drag/drop, visible focus order, screen-reader announcements for moves and list counts, reduced motion, high contrast, and no spinner-only waiting states required.
- Non-goals: dependency graph cycle algorithm internals beyond edge display, recurrence materialisation, search backend, bulk-update implementation.
- Exit: UI-facing implementation cannot pass review without browser/user-story/a11y evidence or explicit UI N/A for API-only sub-slice.

### Slice TASKS-PS-03: dependency-graph and bulk update refusal semantics

- Bounded contexts: `dependency-graph`, plus source-lock-selected `bulk-update-seam` for graph mutation semantics even though bulk update is not a separate manifest bounded context.
- Conflict class: `tasks-dependency-graph-cycle-and-bulk-update`.
- Allowed prefixes: `oya/tasks/contracts/openapi/tasks.yaml`, `oya/tasks/contracts/asyncapi/tasks-events.yaml`, `oya/tasks/contracts/proto/tasks.proto`, `oya/tasks/crates/oya-tasks-domain/`, future `oya/tasks/crates/oya-tasks-dependency-graph*/` only after verified, `oya/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `oya/tasks/slos/bulk-update-latency.openslo.yaml`, `oya/tasks/runbooks/dependency-cycle-corruption.md`, `oya/tasks/runbooks/bulk-edit-throttle.md`, `plan/tasks/`, and `evidence/tasks/`.
- Test/fixture plan: RED cycle-prevention fixture rejects self-edge, two-node cycle, multi-hop cycle, and batch patch that would introduce cycles; proves atomic/refusal semantics, partial-failure reporting where allowed, throttling/backpressure, and audit-chain refusal metadata.
- API/contract replay: OpenAPI/proto replay for dependency edge create/delete/traversal and bulk patch requests; AsyncAPI replay for dependency/bulk update events and dead-letter behavior.
- Browser/user-story/accessibility: user edits blockers/dependencies and bulk-updates tasks; receives clear cycle-refusal, atomicity/partial-failure, and audit-inspection feedback; keyboard and screen-reader support for dependency edge editing and bulk update progress required if UI touched.
- Non-goals: task lifecycle storage beyond fixtures, recurrence, search, importer, AI auto-assign.
- Exit: no graph mutation Build starts until the RED fixture proves cycles/bad batches fail first.

### Slice TASKS-PS-04: recurrence materialisation

- Bounded context: `recurrence`.
- Conflict class: `tasks-recurrence-idempotency-timezone`.
- Allowed prefixes: `oya/tasks/contracts/openapi/tasks.yaml`, `oya/tasks/contracts/asyncapi/tasks-events.yaml`, `oya/tasks/contracts/proto/tasks.proto`, `oya/tasks/crates/oya-tasks-domain/`, future `oya/tasks/crates/oya-tasks-recurrence*/` only after verified, `oya/tasks/slos/recurring-materialise-latency.openslo.yaml`, `oya/tasks/runbooks/recurring-task-materialisation-failure.md`, `plan/tasks/`, and `evidence/tasks/`.
- Test/fixture plan: RED fixture for bounded RRULE subset, timezone transitions, idempotency, duplicate prevention, backfill limits, paused/cancelled series, legal-hold interaction, and backpressure when expansion is unbounded.
- API/contract replay: OpenAPI/proto recurrence create/update/materialise/backfill replay; AsyncAPI replay for materialisation events with idempotency/replay metadata.
- Browser/user-story/accessibility: team member creates recurring tasks and verifies materialisation/backfill/disclosure states; keyboard-accessible recurrence builder, localized timezone display, screen-reader announcements for generated instances and errors if UI touched.
- Non-goals: general calendar event state, full RFC5545 implementation, storage/deployment readiness.
- Exit: recurrence Build can only implement the selected RRULE subset after failing-first fixtures capture idempotency and bounds.

### Slice TASKS-PS-05: search-index and authorization-aware saved views

- Bounded context: `search-index`.
- Conflict class: `tasks-search-auth-projection`.
- Allowed prefixes: `oya/tasks/contracts/openapi/tasks.yaml`, `oya/tasks/contracts/asyncapi/tasks-events.yaml`, `oya/tasks/crates/oya-tasks-domain/`, future `oya/tasks/crates/oya-tasks-search-index*/` only after verified, `oya/tasks/policy/`, `oya/tasks/slos/search-latency.openslo.yaml`, `oya/tasks/runbooks/search-index-rebuild.md`, `plan/tasks/`, and `evidence/tasks/`.
- Test/fixture plan: RED fixture for authorized search/filter/saved-view results, cross-tenant denial, legal-hold filtering, stale-index rebuild safety, delete/update propagation, and result redaction for confidential/regulated task content.
- API/contract replay: OpenAPI replay for search/filter/saved-view endpoints and negative authorization cases; AsyncAPI replay for projection update/rebuild events.
- Browser/user-story/accessibility: user searches/filters tasks and saved views without seeing unauthorized, held, stale, or cross-tenant results; keyboard search, result count live-region, empty/error states, high contrast, and no spinner-only rebuild state required if UI touched.
- Non-goals: Meilisearch production deployment, complete observability evidence, unrelated ontology projection implementation.
- Exit: search Build must prove authorization-aware projection behavior before any search-readiness claim.

### Slice TASKS-PS-06: importers, webhooks, and external handoff review

- Bounded contexts: `importers`, plus source-lock-selected `webhook-event-seam` for webhook/event fanout contract semantics.
- Conflict class: `tasks-importer-webhook-idempotent-fanout`.
- Allowed prefixes: `oya/tasks/contracts/openapi/tasks.yaml`, `oya/tasks/contracts/asyncapi/tasks-events.yaml`, `oya/tasks/contracts/proto/tasks.proto`, future `oya/tasks/crates/oya-tasks-importers*/` only after verified, `oya/tasks/runbooks/webhook-fanout-degraded.md`, `oya/tasks/slos/webhook-fire-latency.openslo.yaml`, `plan/tasks/`, and `evidence/tasks/`.
- Test/fixture plan: RED fixture for importer mapping review, duplicate detection, correlation identifiers, idempotent writes, webhook signing/authorization, retry/dead-letter/replay, audit-chain events, and external source not becoming source of truth.
- API/contract replay: OpenAPI importer endpoints, AsyncAPI webhook/event fanout, and proto import/job surfaces as applicable; include replay and dead-letter negative cases.
- Browser/user-story/accessibility: integrator imports tasks from an external system and reviews duplicates, mappings, and dead-letter/error rows; keyboard-accessible review table, screen-reader import progress/errors, and clear policy disclosure required if UI touched.
- Non-goals: adopting Jira-like systems as canonical source, broad connector platform buildout, live webhook delivery claims without replay evidence.
- Exit: importer/webhook Build cannot claim readiness until idempotency and replay/dead-letter fixtures pass.

### Slice TASKS-PS-07: AI-assist policy, fairness, and disclosure seam

- Bounded context: `ai-assist-policy-seam`, a cross-cutting AI-assist/policy seam. This is not one of the seven manifest bounded contexts; it is included because `specs/microservices/tasks.json` scopes bounded AI assistance and `oya/tasks/manifest.json` lists T0/T1/T2 capabilities.
- Conflict class: `tasks-ai-assist-fairness-employment-refusal`.
- Allowed prefixes: `oya/tasks/capabilities/`, `oya/tasks/policy/`, `oya/tasks/cedar/`, `oya/tasks/contracts/openapi/tasks.yaml`, `oya/tasks/contracts/asyncapi/tasks-events.yaml`, `oya/tasks/crates/oya-tasks-domain/`, `oya/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `oya/tasks/runbooks/ai-assign-classifier-rollback.md`, `plan/tasks/`, and `evidence/tasks/`.
- Test/fixture plan: RED fixture for T0 suggestions, T1 categorisation/priority assistance, and T2 auto-assign refusal unless fairness evidence, protected-class exclusion, human override, explicit disclosure, audit evidence, and employment-context policy allow it. Include false-positive/false-negative refusal cases.
- API/contract replay: OpenAPI/AsyncAPI replay for AI-assist request/response, override, refusal, audit, and event emission. No model/provider call is required for RED unless explicitly scoped; deterministic fixtures are preferred.
- Browser/user-story/accessibility: assignee receives recommendation with explanation, confidence/disclosure, human override, fairness/refusal status, and clear error state; accessible disclosure banners, keyboard override, and screen-reader announcements required if UI touched.
- Non-goals: provider integration, model routing, autonomous employment decisioning, hidden assignment automation, production fairness claims.
- Exit: any T2 automation remains refused until fairness/override/disclosure evidence exists.

## Deferred or explicitly out-of-scope contexts

- No manifest bounded context is omitted: `task-store`, `project-list`, `dependency-graph`, `recurrence`, `search-index`, `view-engine`, and `importers` are covered above.
- Webhook/event fanout, bulk update, and AI-assist are not separate manifest bounded contexts; they are included as contract/policy seams because the source lock scopes them.
- Calendar, messenger, mail, workflow-engine, ontology, intelligence, drive, identity, tenancy, observability, audit-chain, application, and cloud services are dependency/service boundaries only. Their implementations are deferred to separate service cards; tasks slices may only use explicit contracts or stubs selected by RED.
- Shared manifests-index correction is deferred because this Plan/Spec card is forbidden from editing shared `specs/microservices/manifests-index.json`.

## RED gate expectations for `t_91fc5a1f`

The RED child should select the smallest safe first set of fixtures, but it must preserve coverage for the above classes:

1. Source-map guard: missing source lock, stale legacy destination, and inventory-as-readiness refusal.
2. API/contract replay: OpenAPI/AsyncAPI/proto surface validation for the selected slice before handlers/storage can pass.
3. Dependency/bulk refusal: cycle and batch mutation fail-first evidence.
4. Recurrence: idempotency, timezone, backfill, and bounds fail-first evidence.
5. Search/auth: tenant/RBAC/legal-hold/stale-index negative evidence.
6. Importer/webhook: duplicate, idempotency, signing/auth, retry/dead-letter/replay negative evidence.
7. AI-assist: fairness, protected-class non-use, human override, disclosure, and employment-context refusal.
8. Accessibility/user story: for any UI slice, a failing story/a11y check or explicit UI N/A.

## Review lenses

- Source-authority/path lens: `specs/microservices/tasks.json` cited; `microservices/` not revived; ADR-0131/ADR-0512 preserved.
- Contract/API lens: replay evidence covers REST/async/proto surfaces for the selected slice.
- Security/privacy lens: tenant/RBAC/Cedar, RLS/tenant-DEK, legal hold, importer/webhook secrecy, and search projection leakage are covered.
- Data/audit-chain lens: task/event/audit metadata is deterministic, replayable, and non-overclaiming.
- Observability/SLO lens: SLO files are not readiness evidence until emitters and replay evidence exist.
- UX/a11y lens: browser/user-story and WCAG 2.2 AA evidence for UI, or explicit N/A for non-UI work.
- AI/fairness/no-overclaim lens: T2 automation refused unless fairness/disclosure/override evidence exists.
- Generated-face lens: no hand edits to generated outputs.
- Lifecycle lens: Build remains downstream of RED; Merge/Rollout/E2E and Learning cards capture closure and follow-ups.

## Recommended first implementation ordering after RED

1. `TASKS-PS-00` source-map guard, because it protects all later cards from stale authority drift.
2. `TASKS-PS-01` task-store lifecycle with minimal task aggregate, because other slices depend on a task identity/state contract.
3. `TASKS-PS-03` dependency-graph/cycle refusal, because it is high-risk data-integrity logic and can remain backend-only.
4. `TASKS-PS-04` recurrence, because idempotency/backfill can corrupt tenant state if delayed.
5. `TASKS-PS-05` search-index authorization, because cross-tenant leaks are high risk.
6. `TASKS-PS-02` project-list/view-engine UI once lifecycle and dependency semantics are fixed enough for user-story replay.
7. `TASKS-PS-06` importers/webhooks once core lifecycle and authorization are stable.
8. `TASKS-PS-07` AI-assist last unless a separate policy/fairness-only RED slice is selected, because T2 auto-assign has employment-impacting risk and must stay fail-closed.

This ordering is not FIFO; it prioritizes authority safety, data integrity, tenant/privacy risk, and prerequisite contracts before UI breadth or AI automation.
