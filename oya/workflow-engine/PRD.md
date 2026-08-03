---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-workflow-engine
microservice: workflow-engine
status: Proposed
sales_segment: shared-substrate-and-product
service_classification: hero-product-and-substrate
service_subtype: substrate-engine-with-product-companion-studio
tenant_class_eligibility: [demo_trial, paid]
paid_billing_components_emitted: []
service_classification_rationale: |
  Workflow is the first hero product per `feedback_workflow_studio_scope`.
  The product surface (visual editor, template gallery, app marketplace) lives
  in the sibling `workflow-studio` µservice; THIS µservice
  (`workflow-engine`) is the durable execution substrate that powers Studio
  AND every other oyatie µservice that needs durable orchestration. Engine
  has ZERO end-user UI. Per ADR-0245 product vs substrate layering; per
  ADR-0245 substrate µservices serve all products. Engine is the n8n-class
  durable runtime + Temporal-class deterministic replay + multi-domain
  (agentic / dev / business / healthcare / supply-chain / delivery).
keystone-bundle: 2026-05-20-foundational-doctrine
milestone_first_ship: M02-foundation
related_adrs:
  - ADR-0008
  - ADR-0009
  - ADR-0019
  - ADR-0028
  - ADR-0035
  - ADR-0037
  - ADR-0049
  - ADR-0050
  - ADR-0056
  - ADR-0103
  - ADR-0105
  - ADR-0107
  - ADR-0117
  - ADR-0131
  - ADR-0140
  - ADR-0141
  - ADR-0145
  - ADR-0148
  - ADR-0162
  - ADR-0172
  - ADR-0179
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0251
  - ADR-0255
  - ADR-0337
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
related_specs:
  - /specs/microservices/workflow-engine.json
  - /specs/tenant-model.json
  - /specs/per-microservice-flat-layout.json
related_memories:
  - workflow-studio-scope
  - workflow-is-shared
  - clean-architecture-requirements
  - tenant-as-universal-scoping-primitive
  - cedar-as-universal-gate
  - quality-performance-scalability-bar
date: 2026-05-20
owner_team: axis-workflow + council-architecture + ops-sre
doc_status: published
tenant_scoped: true
audience_modes:
  - B2C-personal
  - B2B-work
  - oyatie-internal-tenant
benchmarks:
  - temporal-cloud
  - cadence-uber
  - apache-airflow
  - camunda-platform-8
  - argo-workflows
  - n8n
  - aws-step-functions
  - dapr-workflows
  - prefect
  - restate
  - apache-flink-stateful-functions
  - inngest
  - trigger-dev
  - zapier
  - make-formerly-integromat
  - microsoft-power-automate
---

# PRD-workflow-engine: Durable Multi-Domain Workflow Execution Substrate

> Hero substrate paired with the sibling `workflow-studio` hero product. n8n-class durable runtime + Temporal-class deterministic replay + 200+ integration adapters day-one. Multi-domain: agentic / dev / business / healthcare / supply-chain / delivery / personal automation. Engine ships zero UI; Studio renders. Per ADR-0245 substrate vs product layering; per ADR-0244 per-tenant scoping; per ADR-0243 Cedar-gated step execution; per ADR-0251 HIPAA workflows live in HIPAA-eligible cells.

---

## A. Problem

### A.1 Why workflow needs its own µservice

Every oyatie product surface that triggers multi-step, time-spanning, retry-tolerant orchestration faces the same five hard problems:

1. **Durability** — a multi-week workflow MUST survive process crashes, pod evictions, region failovers, dependency outages.
2. **Determinism** — replay-from-event-log MUST produce identical state on re-execution (audit + debug + retroactive bug-fix).
3. **Per-tenant isolation** — tenant A's runs MUST NEVER observe tenant B's events even through shared infrastructure.
4. **Composability** — workflows trigger workflows; sub-workflows; fan-out / fan-in; long-lived signals.
5. **Multi-domain** — same engine powers agentic dev (Foundry CI), business (employee onboarding), healthcare (referral with PHI), supply chain (PO approval cascade), personal automation (IFTTT-class).

Without a substrate, each product builds its own coroutine machine over Postgres, with predictably divergent semantics and PII-handling regressions. The workflow-engine substrate consolidates: durable state, deterministic replay, retry policy, SLA timer, sub-workflow, event-bus, signal, pause/resume, audit-chain, Cedar-gated step execution, plugin sandbox, integration adapter contract.

### A.2 What competitors get wrong

Hyperscaler precedents:

- **Temporal Cloud / Temporal OSS** — reference durable execution; deterministic replay; SDK-driven (Go, Java, TS, Python, .NET, Ruby, PHP). Weakness: per-namespace pricing; no built-in marketplace; no Cedar gating; no built-in tenant-isolation invariants.
- **AWS Step Functions** — managed state-machine ASL DSL; deep AWS integration. Weakness: AWS-only; no built-in marketplace; ASL feels constrained.
- **Apache Airflow** — DAG scheduler; large operator ecosystem. Weakness: DAG-only (no long-running state machines); task-instance-centric not workflow-centric; weak multi-tenancy.
- **Camunda Platform 8** — BPMN 2.0 + Zeebe engine. Weakness: BPMN learning curve; cluster shardability; weak per-tenant model.
- **Argo Workflows** — container-native step execution on K8s. Weakness: K8s-tied; no marketplace.
- **n8n** — visual editor + execution engine; broad integration catalog. Weakness: best-effort durability (not deterministic-replay-grade); single-process for many deployments.
- **Make / Zapier / Power Automate** — visual-editor-first SaaS. Weakness: opaque execution; SaaS-tied; weak audit + customisation.
- **Cadence** — Temporal's predecessor; mostly subsumed.
- **Dapr Workflows** — actor-based building block. Weakness: nascent.
- **Restate** — distributed durable execution. Weakness: nascent.
- **Inngest / Trigger.dev** — developer-first event-driven; modern. Weakness: SaaS-tied; weak multi-tenant primitive.

Failure modes observed:

1. **n8n 2024 incident** — a tenant's failing step blocked the shared queue for 12 hours. Fix: per-tenant queue isolation.
2. **Airflow scheduler outage 2023** — DAG-parsing memory leak across all DAGs. Fix: per-tenant compilation isolation.
3. **Zapier outage 2024** — partial deliveries created duplicate downstream effects. Fix: exactly-once-effect step contract.
4. **Temporal Cloud 2024 incident** — namespace migrations corrupted some run histories. Fix: rigorous spec-version handshake.

### A.3 What "good" looks like

Engine consumers fall into 4 archetypes:

1. **Studio editor** — Studio submits a compiled spec → engine durably stores + versions; Studio observes run live; Studio replays past run in a debugger.
2. **Sibling µservices** — `payments` triggers `subscription-renewal-workflow`; `identity` triggers `onboarding-cascade-workflow`; `messenger` triggers `notification-fan-out-workflow`. Each calls engine via `oya-workflow-engine-sdk`.
3. **Tenant workloads via SDK** — tenant embeds engine SDK; in-process triggers fire workflows on the tenant's behalf.
4. **External webhooks** — Studio routes external webhooks (Stripe, Twilio, custom) into the engine as triggers.

Engine MUST: survive any crash without losing a step; replay deterministically; isolate tenants in queue + state + audit; scale linearly per cell via Citus + per-tenant partitions; emit audit-chain seal within 1s of completion; expose typed events on a per-tenant bus.

---

## B. Target Users (Personas)

### B.1 B2C personas

#### Persona B2C-1 — "Personal automation Patricia, US consumer building IFTTT-class personal automations"
- **Goals**: connect her smart home (Hue lights, Nest thermostat) to her oyatie messenger; sunset → dim lights; new message in family group chat → ping watch.
- **Frustrations**: opaque visual editors; flaky executions; vendor lock-in (Zapier 100-task limit, IFTTT $5/mo for >3 applets).
- **Tech comfort**: medium (knows Zapier; understands "trigger → action").
- **Locale + device**: en-US, ET, web + iOS.

#### Persona B2C-2 — "Creator-economy Carlos, Brazilian shorts creator automating his content pipeline"
- **Goals**: when his shorts publish → cross-post to community + send notification to subscribers + queue analytics report.
- **Frustrations**: paying $30/mo for Zapier when he barely uses 10% of features; latency between publish and downstream effects.
- **Tech comfort**: medium-high.
- **Locale + device**: pt-BR, BRT, Android + MacBook.

#### Persona B2C-3 — "Agentic consumer Aoi, Japanese power user with personal AI assistant"
- **Goals**: her personal AI agent (per ADR-0255 consumer brand surface) orchestrates daily tasks: morning briefing fan-in (calendar + email + news + weather + commute) → assemble → deliver to messenger by 7:00 JST.
- **Frustrations**: agentic-workflow latency; flaky LLM calls; lack of per-step privacy boundary.
- **Tech comfort**: very high.
- **Locale + device**: ja-JP, JST, iPhone + AirPods + MacBook.

### B.2 B2B personas

#### Persona B2B-1 — "DevEx Diana, oyatie internal Foundry team building agentic CI workflow"
- **Goals**: code commit → trigger Foundry-CI workflow → Cedar lint → multi-spectrum review → merge-queue ChangeSet → deploy → smoke test. Workflow must replay deterministically for incident review; sub-workflows for each lane.
- **Frustrations**: opaque CI pipelines; non-deterministic flaky tests; manual incident retro.
- **Tech comfort**: very high (council-architecture).
- **Locale + device**: en-US, ET, MacBook + Linux dev box.

#### Persona B2B-2 — "Onboarding Ops Olivia, B2B SaaS people-ops orchestrating employee onboarding"
- **Goals**: HRIS `EmployeeHired` event → engine fans out: identity-provision, payroll-enroll, messenger-account, calendar-init, drive-quota, plugin-store-default-apps, welcome-message-day-1, equipment-shipping, manager-1on1-schedule. Each sub-flow has SLA timer; failures escalate.
- **Frustrations**: ServiceNow-class IT-orchestration tools that cost $$$$ and require professional services.
- **Tech comfort**: medium (no-code visual editor + scripted nodes).
- **Locale + device**: en-US, ET, desktop primary.

#### Persona B2B-3 — "Healthcare Coordinator Hannah, US clinic orchestrating patient referrals"
- **Goals**: patient referral form submitted → engine fan-out: insurance eligibility check → appointment scheduling → records request → referring-provider notification → patient instructions. PHI in payloads; HIPAA workflow.
- **Frustrations**: paper-based workflow with phone-call hand-offs; lost records; no audit trail; HIPAA-shy SaaS vendors.
- **Tech comfort**: medium.
- **Locale + device**: en-US, CT, desktop in clinic office + tablet at front-desk.

### B.3 Internal persona

#### Persona INT-1 — "SRE Sofía, oyatie ops-sre on-call for workflow-engine"
- **Goals**: monitor engine health (queue depth, step latency, replay throughput); respond to incidents; tune retry policies; review post-incident traces.
- **Frustrations**: noisy alerts; unclear ownership when a sub-workflow's external dependency is down.
- **Tech comfort**: very high.

---

## C. User Stories

Stories are workflow-engine-µservice specific. Do NOT duplicate `docs/user-stories/b2c-consumer-surfaces.md` or `b2b-work-surfaces.md`; add engine-specific NEW stories.

### US-workflow-01 — Personal automation: sunset → dim lights
- **As** Patricia (B2C-1)
- **I want** a workflow that triggers on local sunset (calculated from her location) and dims her Hue lights to 30%
- **so that** her evenings auto-tune without my intervention.
- **Acceptance criteria**:
  1. Studio submits compiled spec; engine `spec.persisted` event emitted; idempotent on resubmit.
  2. Trigger fires at sunset ± 1min (timer accuracy SLO).
  3. Step `hue.set_brightness(30)` executes; on transient failure, retry per default retry ladder.
  4. Run completes within 30s; audit-chain sealed; webhook to Patricia's notifications channel.
- **Accessibility AC**: Studio UI shows workflow in screen-reader friendly tree.
- **i18n AC**: sunset time in user TZ; brightness % localized.

### US-workflow-02 — Cross-post automation
- **As** Carlos (B2C-2)
- **I want** his shorts publish event to fan out to community + subscribers + analytics queue
- **so that** he doesn't manually copy.
- **Acceptance criteria**:
  1. `shorts.publish` event triggers workflow.
  2. Parallel fan-out: 3 child workflows or 3 steps run concurrently.
  3. Each step independently retries; failure of one does not block others.
  4. Aggregate completion event emitted when all complete (or all-but-quorum settles).
- **Accessibility AC**: Studio progress view accessible.
- **i18n AC**: pt-BR.

### US-workflow-03 — Morning briefing fan-in
- **As** Aoi (B2C-3)
- **I want** her agentic morning workflow to fan-in calendar + email + news + weather + commute and deliver by 7:00 JST
- **so that** her morning starts informed.
- **Acceptance criteria**:
  1. Cron at 06:55 JST.
  2. Fan-in: 5 parallel data fetches with 2-minute timeout each.
  3. LLM step (Cedar autonomy-tier T1) assembles narrative.
  4. Delivery to messenger by 07:00; SLA breach pages on-call (her).
- **Accessibility AC**: delivered message accessibility.
- **i18n AC**: ja-JP.

### US-workflow-04 — Foundry-CI deterministic replay
- **As** Diana (B2B-1)
- **I want** to replay a CI run from yesterday step-by-step to debug a flake
- **so that** I can find the root cause.
- **Acceptance criteria**:
  1. Studio "Open run in replay" launches deterministic replay session.
  2. Replay produces identical step sequence (no flake).
  3. UI shows each step's input + output + duration + retry attempts.
  4. Replay does NOT trigger external side-effects.
- **Accessibility AC**: replay UI accessibility AA.
- **i18n AC**: en-US.

### US-workflow-05 — Employee onboarding fan-out
- **As** Olivia (B2B-2)
- **I want** the `EmployeeHired` workflow to fan out 9 sub-flows with SLA timers
- **so that** new employees are onboarded within Day-1.
- **Acceptance criteria**:
  1. Workflow spec: parent + 9 children.
  2. Each child has SLA timer (e.g., identity-provision 5min, equipment-shipping 2 days).
  3. SLA breach escalates to manager.
  4. Workflow waits for all children OR fails-with-summary if any non-recoverable.
- **Accessibility AC**: Studio fan-out view accessibility AA.
- **i18n AC**: per-locale.

### US-workflow-06 — Healthcare referral with PHI
- **As** Hannah (B2B-3)
- **I want** PHI-touching steps to stay in HIPAA-eligible cells
- **so that** compliance is honored.
- **Acceptance criteria**:
  1. Spec annotation `data_class: PHI` triggers cell-affinity check.
  2. Engine refuses to dispatch the spec to a non-HIPAA cell.
  3. Step payloads encrypted at rest per ADR-0111.
  4. Audit-chain per disclosure.
- **Accessibility AC**: Studio shows cell-affinity badge.
- **i18n AC**: en-US.

### US-workflow-07 — SLA timer with escalation
- **As** Olivia (B2B-2)
- **I want** a step with a 24h SLA; on breach, escalate to the manager
- **so that** stuck workflows surface.
- **Acceptance criteria**:
  1. Spec: step has `sla: 24h, on_breach: notify_manager`.
  2. Timer armed at step entry; cancelled on completion.
  3. On breach, escalation step runs.
  4. Audit-chain emits `SlaBreachEscalated`.
- **Accessibility AC**: escalation notification accessible.
- **i18n AC**: localized.

### US-workflow-08 — Pause + resume on operator intervention
- **As** Diana (B2B-1) running a workflow that pauses mid-execution for human review
- **I want** to pause + later resume
- **so that** the engine waits without busy-loop.
- **Acceptance criteria**:
  1. Step `human_review(payload)` automatically pauses run.
  2. UI shows pending review; reviewer approves/rejects.
  3. Engine resumes within 1s of decision.
  4. Long-pause supported up to 90 days.
- **Accessibility AC**: review UI accessibility AA.
- **i18n AC**: en-US.

### US-workflow-09 — Sub-workflow with parent-child correlation
- **As** Olivia (B2B-2)
- **I want** child workflows to know they have a parent (for context + summary aggregation)
- **so that** the parent can summarize.
- **Acceptance criteria**:
  1. `spawn_child(spec_id, input, return_value=true)` returns child handle.
  2. Parent can await child OR fire-and-forget.
  3. Child sees `parent.run_id` in context.
- **Accessibility AC**: Studio tree-view shows parent-child.
- **i18n AC**: per-locale.

### US-workflow-10 — Signal sent to running workflow
- **As** Diana (B2B-1) running a long-lived workflow
- **I want** to send a signal (e.g., `update_threshold(new_value)`) without restart
- **so that** workflows adapt to runtime changes.
- **Acceptance criteria**:
  1. `POST /v1/runs/<id>/signals {name, payload}` delivers signal.
  2. Workflow receives signal in its step body.
  3. Signal ordered relative to step boundaries.
- **Accessibility AC**: N/A.
- **i18n AC**: per-locale.

### US-workflow-11 — Query running workflow state
- **As** Diana (B2B-1)
- **I want** to query a running workflow's local state without mutating
- **so that** I can debug.
- **Acceptance criteria**:
  1. `GET /v1/runs/<id>/queries/<name>` invokes a query handler in the workflow.
  2. Query handlers are read-only (engine enforces).
- **Accessibility AC**: dashboard accessibility AA.
- **i18n AC**: en-US.

### US-workflow-12 — Retry ladder with backoff
- **As** a workflow step calling a flaky external API
- **I want** retry with exponential backoff
- **so that** transient failures recover.
- **Acceptance criteria**:
  1. Spec: `retry: { attempts: 5, backoff: exponential, initial: 1s, max: 5min, jitter: true }`.
  2. Engine retries per ladder; gives up after 5; emits `step.exhausted`.
- **Accessibility AC**: Studio shows retry attempts.
- **i18n AC**: per-locale.

### US-workflow-13 — Idempotency by step
- **As** an engine worker
- **I want** to avoid duplicating an external side-effect on retry
- **so that** retried HTTP POSTs don't create duplicate orders.
- **Acceptance criteria**:
  1. Step spec: `idempotency_key: order_id`.
  2. Adapter (HTTP) includes the key in outbound request.
  3. Server-side idempotency check (if the API supports) deduplicates.
- **Accessibility AC**: N/A.
- **i18n AC**: error messages localized.

### US-workflow-14 — Workflow versioning + migration
- **As** Diana (B2B-1) updating a workflow with breaking changes
- **I want** in-flight runs to continue on old version; new runs on new version
- **so that** I don't corrupt in-flight state.
- **Acceptance criteria**:
  1. Spec submit returns new `version_sha`; older specs retained.
  2. New runs use latest; in-flight pin to their version.
  3. Manual migration tool for upgrading in-flight runs (rare).
- **Accessibility AC**: Studio version chooser accessibility.
- **i18n AC**: per-locale.

### US-workflow-15 — Plugin custom node
- **As** Diana (B2B-1) needing a custom step type
- **I want** to write a WASM custom node per ADR-0037
- **so that** my logic runs inside the sandboxed step.
- **Acceptance criteria**:
  1. Custom node WASM uploaded; signed by tenant.
  2. Runtime sandbox: no filesystem, no network beyond declared, bounded CPU + memory.
  3. Custom node visible in Studio node library.
- **Accessibility AC**: N/A (developer-tool).
- **i18n AC**: per-developer locale.

### US-workflow-16 — Per-tenant rate limit
- **As** ops-sre
- **I want** per-tenant caps on step throughput
- **so that** one tenant's runaway can't starve others.
- **Acceptance criteria**:
  1. Per-tenant token-bucket; configurable.
  2. Over-cap steps queue; do not crash engine.
  3. Per-tenant queue-depth metric exposed.
- **Accessibility AC**: N/A.
- **i18n AC**: per-locale.

### US-workflow-17 — Cross-cell workflow span
- **As** Diana (B2B-1)
- **I want** workflows to call sub-workflows in a different cell (e.g., a global utility cell)
- **so that** I don't duplicate logic.
- **Acceptance criteria**:
  1. `cross_cell_spawn(cell_id, spec_id, input)` returns child handle.
  2. Cross-cell bridge (Kafka per ADR-0050) carries events.
  3. Audit chain spans cells.
- **Accessibility AC**: Studio shows cross-cell link.
- **i18n AC**: per-locale.

### US-workflow-18 — Webhook trigger
- **As** Carlos (B2C-2) integrating with an external event source
- **I want** to register a webhook URL that triggers a workflow on incoming POST
- **so that** external systems drive workflows.
- **Acceptance criteria**:
  1. `POST /v1/triggers/webhooks` returns a unique URL.
  2. Inbound POST verified via HMAC + nonce.
  3. Workflow triggered within 1s.
- **Accessibility AC**: webhook UI accessibility.
- **i18n AC**: per-locale.

### US-workflow-19 — Schedule trigger (cron)
- **As** Aoi (B2C-3)
- **I want** to schedule a workflow daily at 06:55 JST
- **so that** her morning briefing runs.
- **Acceptance criteria**:
  1. Spec: `schedule: { type: cron, expr: '55 6 * * *', tz: 'Asia/Tokyo' }`.
  2. Engine arms schedule; fires within ±10s.
  3. Schedule survives engine restarts.
- **Accessibility AC**: Studio schedule UI accessibility.
- **i18n AC**: ja-JP.

### US-workflow-20 — Event-bus subscription
- **As** a sibling µservice (`payments`)
- **I want** to subscribe to `subscription.renewed` events
- **so that** I drive payment-renewal workflows.
- **Acceptance criteria**:
  1. SDK subscribe API returns a subscription handle.
  2. Backpressure-aware delivery (slow consumer doesn't drop).
  3. Replay from offset supported (last 30 days).
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-workflow-21 — Run-state durability across pod eviction
- **As** ops-sre (INT-1)
- **I want** a running workflow to survive pod eviction without loss
- **so that** Kubernetes can freely manage capacity.
- **Acceptance criteria**:
  1. Pod evicted mid-step; new pod claims the run via Valkey lease.
  2. Run resumes from last completed step.
  3. Step-result deduplication prevents double-effect.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-workflow-22 — Run-state durability across region failure
- **As** ops-sre (INT-1)
- **I want** workflows to survive a region-level event
- **so that** business continuity holds.
- **Acceptance criteria**:
  1. Postgres + Citus replicated cross-region within pack.
  2. Failover within 60s.
  3. RPO ≤ 5s.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-workflow-23 — Audit chain seal per run
- **As** Sasha (security)
- **I want** every run sealed with Ed25519 + Merkle per ADR-0028
- **so that** tampering is detectable.
- **Acceptance criteria**:
  1. Run completion triggers seal within 1s.
  2. Seal verifiable by anyone with the public key.
  3. Tampered run-history fails verification.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-workflow-24 — Tenant isolation at event-bus topic level
- **As** ops-sre
- **I want** tenant A's events to be unreachable from tenant B's subscriber
- **so that** no leak.
- **Acceptance criteria**:
  1. Topic naming: `tenant.<tenant_id>.<event_type>`.
  2. Cedar policy refuses cross-tenant topic subscribe.
  3. Postgres RLS enforces at storage layer.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-workflow-25 — Cedar autonomy ceiling for agentic steps
- **As** Aoi (B2C-3) running her agentic morning workflow
- **I want** the LLM step to be Cedar-bounded (autonomy tier T1)
- **so that** the agent can't escalate without approval.
- **Acceptance criteria**:
  1. Step spec: `autonomy_tier: T1`.
  2. Cedar policy refuses higher-tier ops within step.
  3. Tier escalation requires step exit + step-up auth.
- **Accessibility AC**: Studio shows tier badge.
- **i18n AC**: ja-JP.

### US-workflow-26 — Per-step compensation
- **As** Diana (B2B-1) designing a multi-step transaction
- **I want** to declare compensating actions per step
- **so that** a failed downstream step rolls back upstream effects (saga pattern).
- **Acceptance criteria**:
  1. Spec: `compensation: <fn>`.
  2. On failure, engine runs compensations in reverse order.
  3. Compensation failures audit-chained.
- **Accessibility AC**: Studio saga view accessibility AA.
- **i18n AC**: per-locale.

### US-workflow-27 — Cron schedule pause / resume
- **As** Olivia (B2B-2) on holiday
- **I want** to pause her scheduled workflows for 2 weeks
- **so that** they don't fire.
- **Acceptance criteria**:
  1. `PATCH /v1/schedules/<id> { state: paused }`.
  2. No triggers during pause.
  3. Resume re-arms.
- **Accessibility AC**: schedule UI accessibility AA.
- **i18n AC**: per-locale.

### US-workflow-28 — Bulk migration of legacy n8n workflows
- **As** Olivia (B2B-2) migrating from n8n
- **I want** to import a JSON export
- **so that** I don't rewrite.
- **Acceptance criteria**:
  1. Importer converts n8n JSON to oyatie spec.
  2. Reports unsupported nodes.
  3. Compiled spec runs deterministically.
- **Accessibility AC**: importer UI accessibility AA.
- **i18n AC**: per-locale.

### US-workflow-29 — Bulk migration from Zapier
- **As** Olivia (B2B-2) migrating from Zapier
- **I want** to import Zaps
- **so that** I switch.
- **Acceptance criteria**:
  1. Importer for Zap JSON.
  2. Reports unsupported integrations.
  3. Compiled spec runs deterministically.
- **Accessibility AC**: importer UI accessibility AA.
- **i18n AC**: per-locale.

### US-workflow-30 — Bulk migration from Temporal
- **As** Diana (B2B-1) migrating from Temporal Cloud
- **I want** the engine SDK to expose a Temporal-compatible API surface
- **so that** I migrate with minimum code change.
- **Acceptance criteria**:
  1. SDK surface mirrors Temporal Go/TS shapes where possible.
  2. Differences documented.
  3. Migration guide published.
- **Accessibility AC**: docs accessibility.
- **i18n AC**: en-US.

### US-workflow-31 — Long-lived workflow (90 days paused)
- **As** Olivia (B2B-2)
- **I want** workflows that can pause for 90 days
- **so that** approval-cycle workflows work.
- **Acceptance criteria**:
  1. Paused state stored in Postgres; engine workers don't poll.
  2. Resume on signal OR timer.
- **Accessibility AC**: N/A.
- **i18n AC**: per-locale.

### US-workflow-32 — Per-tenant workflow library access
- **As** Olivia (B2B-2)
- **I want** to see only my tenant's workflows
- **so that** I don't see other tenants'.
- **Acceptance criteria**:
  1. `GET /v1/workflows` filters by tenant.
  2. Cross-tenant access refused at Cedar gate AND RLS.
- **Accessibility AC**: dashboard accessibility AA.
- **i18n AC**: per-locale.

### US-workflow-33 — Step-execution metrics
- **As** Olivia (B2B-2)
- **I want** per-workflow P50/P99 step latency + failure rate
- **so that** I tune.
- **Acceptance criteria**:
  1. Dashboard shows charts.
  2. Filterable per (workflow, step).
  3. Export CSV.
- **Accessibility AC**: dashboard accessibility AA.
- **i18n AC**: per-locale.

### US-workflow-34 — DSAR-cascade through workflow data
- **As** EU consumer
- **I want** my data within step-payloads tombstoned
- **so that** GDPR Art. 17 is honored.
- **Acceptance criteria**:
  1. Engine catalogs which workflows touched the subject (via Ontology link).
  2. Tombstones step-payload PII; preserves operational metadata.
  3. Retention beyond DSAR per pack policy (audit retention).
- **Accessibility AC**: N/A.
- **i18n AC**: per-locale.

### US-workflow-35 — Compliance pack workflow restriction
- **As** Carl (security)
- **I want** to forbid certain step types in healthcare workflows (e.g., external HTTP to non-BAA endpoints)
- **so that** PHI doesn't leak.
- **Acceptance criteria**:
  1. Pack-level Cedar policy refuses step type.
  2. Engine refuses to compile a spec with disallowed step in this pack.
  3. CI lane catches at admission.
- **Accessibility AC**: Studio shows refusal with reason.
- **i18n AC**: per-locale.

### US-workflow-36 — Cross-Slice references
- See `docs/user-stories/b2b-work-surfaces.md#US-B2B-WF-*` for B2B workflow product-surface stories.
- See `docs/user-stories/b2c-consumer-surfaces.md#US-B2C-AUTO-*` for personal-automation product stories.
- This PRD's stories are engine-µservice-specific.

### US-workflow-37 — Cold-start of new pod ≤ 500ms
- **As** ops-sre
- **I want** new engine worker pods to be ready ≤ 500ms
- **so that** auto-scale responsive.
- **Acceptance criteria**:
  1. Pre-warmed pool of 10 standby pods.
  2. Cold-start budget enforced via container image optimization.
  3. Readiness probe within budget.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-workflow-38 — 10k concurrent runs per cell
- **As** ops-sre
- **I want** baseline capacity of 10k concurrent runs per cell
- **so that** day-one scale meets demand.
- **Acceptance criteria**:
  1. Load test sustains 10k runs without queue-depth growth.
  2. P99 step latency ≤ 200ms.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-workflow-39 — Replay throughput ≥ 1k steps/s/worker
- **As** Diana (B2B-1) replaying a 10k-step run
- **I want** replay to complete in ≤ 10s
- **so that** debugging is fast.
- **Acceptance criteria**:
  1. Replay throughput ≥ 1000 steps/s/worker.
  2. CPU-bound; deterministic.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-workflow-40 — Spec signature verification
- **As** Sasha (security)
- **I want** every submitted spec to be signed (Ed25519) by the submitter
- **so that** tampering is detected.
- **Acceptance criteria**:
  1. Submit requires signature.
  2. Read-time verification refuses tampered specs.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-workflow-41 — Workflow templating + tenant-fork
- **As** Olivia (B2B-2)
- **I want** to fork a marketplace template into our tenant
- **so that** I customize.
- **Acceptance criteria**:
  1. Marketplace template imported as base.
  2. Tenant-fork tracks divergence.
  3. Upstream template updates surfaced as suggestions.
- **Accessibility AC**: marketplace UI accessibility AA.
- **i18n AC**: per-locale.

### US-workflow-42 — Per-tenant compliance pack constraints
- **As** Carl (security)
- **I want** workflows in `pack-eu` to refuse storing personal data outside EU cells
- **so that** GDPR data-residency holds.
- **Acceptance criteria**:
  1. Cell-affinity check at admission.
  2. Cross-cell spawn into non-EU cell refused.
- **Accessibility AC**: refusal UI explanatory.
- **i18n AC**: per-locale.

---

## D. Functional Requirements

### D.1 Spec store surface

| ID | Requirement |
|---|---|
| FR-W-01 | `POST /v1/workflows` accepts compiled spec; returns `workflow_id` + `version_sha`. |
| FR-W-02 | Specs are content-addressed by SHA-256; immutable once published. |
| FR-W-03 | Specs signed (Ed25519); signature verified on read. |
| FR-W-04 | Spec hot-reload: in-flight runs pinned to their version. |
| FR-W-05 | Spec deprecation: 90-day grace before removal. |

### D.2 Execution engine surface

| ID | Requirement |
|---|---|
| FR-W-10 | `POST /v1/runs` starts a new run; returns `run_id`. |
| FR-W-11 | `GET /v1/runs/<id>` returns current state. |
| FR-W-12 | `POST /v1/runs/<id>/pause` pauses. |
| FR-W-13 | `POST /v1/runs/<id>/resume` resumes. |
| FR-W-14 | `POST /v1/runs/<id>/cancel` cancels. |
| FR-W-15 | `POST /v1/runs/<id>/signals` sends signal. |
| FR-W-16 | `GET /v1/runs/<id>/queries/<name>` invokes read-only query handler. |
| FR-W-17 | Step execution exactly-once-effect per step (idempotency + dedup). |
| FR-W-18 | Retry per spec policy. |
| FR-W-19 | SLA timer armed/cancelled per step; escalation on breach. |
| FR-W-20 | Sub-workflow invocation (sync + async + fire-and-forget). |
| FR-W-21 | Cross-cell spawn supported. |

### D.3 Event bus surface

| ID | Requirement |
|---|---|
| FR-W-30 | `POST /v1/events` publishes typed event. |
| FR-W-31 | `POST /v1/subscriptions` creates a subscription. |
| FR-W-32 | Backpressure-aware delivery (subscriber lag tolerable). |
| FR-W-33 | Replay from offset (up to 30 days). |
| FR-W-34 | Per-tenant topic namespace. |

### D.4 Replay debugger surface

| ID | Requirement |
|---|---|
| FR-W-40 | `POST /v1/replay/sessions` starts a replay session. |
| FR-W-41 | Replay does NOT trigger external side-effects. |
| FR-W-42 | Replay throughput ≥ 1k steps/s/worker. |
| FR-W-43 | Step-snapshot retrieval. |
| FR-W-44 | Analytics queries over run history (ClickHouse mirror). |

### D.5 Trigger surface

| ID | Requirement |
|---|---|
| FR-W-50 | Cron schedule trigger (timezone-aware). |
| FR-W-51 | Webhook trigger (HMAC-signed URL). |
| FR-W-52 | Event-bus trigger (sibling-µservice event). |
| FR-W-53 | Manual trigger (UI / API). |
| FR-W-54 | Workflow-spawn-workflow trigger. |

### D.6 Cedar gating + autonomy tier

| ID | Requirement |
|---|---|
| FR-W-60 | Every step's principal + action + resource Cedar-evaluated. |
| FR-W-61 | Autonomy tier ceiling per step (`T0..T3`). |
| FR-W-62 | Tier escalation refused without step-up. |
| FR-W-63 | EU AI Act conformity: Annex III flag per step type. |

### D.7 Plugin substrate (custom nodes per ADR-0037)

| ID | Requirement |
|---|---|
| FR-W-70 | WASM custom-node format; signed by tenant. |
| FR-W-71 | Sandbox per Wasmtime; no filesystem, declared network only. |
| FR-W-72 | CPU + memory bounds per node. |
| FR-W-73 | Custom node visible in Studio node library. |

### D.8 Audit + DSAR

| ID | Requirement |
|---|---|
| FR-W-80 | Per-run audit-chain seal within 1s (ADR-0028). |
| FR-W-81 | Per-step audit-chain emission. |
| FR-W-82 | DSAR-cascade tombstones step-payload PII. |
| FR-W-83 | Compliance retention per pack honored. |

---

## E. Non-functional Requirements

### E.1 Performance budgets

| Metric | P50 | P95 | P99 | Notes |
|---|---|---|---|---|
| Workflow spec save | 20 ms | 70 ms | 100 ms | compile + register |
| Workflow run start → first step | 50 ms | 150 ms | 200 ms | engine dispatch |
| Step execution (local action) | 5 ms | 30 ms | 50 ms | excludes external HTTP |
| Step execution (external HTTP) | — | 100 ms | 200 ms | network-bound |
| Event-to-action latency | 50 ms | 300 ms | 500 ms | outbox → bus → worker → dispatch |
| State persistence per step | 5 ms | 18 ms | 25 ms | Postgres single-row |
| Replay throughput | — | — | 1000 steps/s/worker | CPU-bound deterministic replay |
| Audit seal per run | — | 800 ms | 1 s | Ed25519 + Merkle |
| Cold-start worker pod | — | 400 ms | 500 ms | pre-warmed pool |
| Cross-cell spawn round-trip | 100 ms | 400 ms | 800 ms | Kafka-bridge bound |

(Evidence: modeling notes `docs/performance-budgets/workflow-engine-step-latency.md` + `docs/performance-budgets/workflow-engine-replay-throughput.md` to be authored M02.)

### E.2 Availability

| Surface | Target |
|---|---|
| Execution path | 99.95% monthly |
| Replay backend | 99.9% |
| Event bus | 99.95% |
| Spec store | 99.99% |

### E.3 Scalability

- Per-cell baseline: 10,000 concurrent active runs; max 500,000.
- Per-cell baseline: 5,000 steps/s dispatched; max 200,000.
- Per-cell baseline: 10,000 events/s on bus; max 1,000,000.
- Active-active per-cell per ADR-0009.
- Postgres + Citus sharded by `tenant_id`.

### E.4 Durability guarantees

- Step state persisted to Postgres BEFORE dispatch (write-ahead).
- Outbox pattern (ADR-0050) for cross-cell events.
- Replay deterministic given identical event log + initial state.
- 90-day paused-state retention.

### E.5 Security

- JWT tenant_id enforced at every REST/gRPC entry.
- Per-tenant workflow library; per-tenant run isolation; per-tenant event-bus topic namespace.
- Workflow specs signed (Ed25519); spec tampering detected on read.
- Plugin substrate per ADR-0037: Wasmtime sandbox; no host filesystem; CPU + memory bounded.
- Replay-attack window on inbound webhooks ≤ 5 minutes via HMAC-SHA256 + nonce.
- Cedar policy fragments gate spec submission, run start, pause/resume, cancel, replay.
- Audit-chain emission on every run start, every state transition, every operator intervention.

### E.6 Audit + compliance

- Per-run Merkle + Ed25519 seal per ADR-0028; seal latency P99 ≤ 1s.
- Deterministic replay required for audit + debug + retroactive bug-fix.
- Per-tenant jurisdiction code inherited per ADR-0117; runs pinned to pack region; cross-pack run-state replication refused by default.
- PII fields in step payloads encrypted at rest (ADR-0111); data_class annotations.

### E.7 Determinism contract

Step bodies MUST be deterministic:

- No use of system clock except via `engine_now()` (deterministic across replay).
- No use of system RNG except via `engine_random()` (seeded from event log).
- No I/O except via declared adapters (engine records request + response).
- No goroutine / async-task spawning outside engine SDK.

CI lane `oya gate validate deterministic-replay --microservice workflow-engine` enforces.

### E.8 DR posture (ADR-0343)

- RTO/RPO target: manifest-declared RTO p99 2100s and RPO p99 5s for workflow state, event log, and replayable execution history. Applicable floors are EU-AI-ACT-2024-HIGH-RISK 1800s/300s, HIPAA-2024 3600s/300s, SOX-404 14400s/3600s, SOC2-T2 14400s/900s, and KR-PIPA 14400s/900s; the engine RPO is stricter than all listed floors, while EU-AI high-risk placement requires either an admission refusal or a future manifest/runbook tightening to <= 1800s RTO.
- Failover reference: manifest `failover_runbook` is `runbooks/durable-execution-restart.md`; supporting recovery drills remain `runbooks/durable-execution-history-replay.md`, `runbooks/workflow-state-corruption-recovery.md`, and `runbooks/saga-compensation-failure-investigation.md`.
- Multi-region active-active posture: false per manifest; replication shape is `active-passive-cross-region-continuous` across `postgres_wal_g`, Valkey, `clickhouse_iceberg_layered`, and versioned object storage.
- Tenant-visible behavior: long-running workflows restart from the sealed event history after active-passive promotion instead of asking tenants to resubmit approvals, healthcare referrals, payroll steps, or incident automations.

### E.9 Capacity model (ADR-0340)

- Per-tenant baseline: manifest-declared 0.6 vCPU, 768 MiB RAM, 12 GB storage, six Postgres connections, four Valkey connections, and twelve outbound HTTP connections reserved before burst admission.
- Scaling dimension: `per_workflow_run` per manifest, with `per_step`, `per_event`, and `per_replay_job` as secondary operational dimensions for dispatch, bus ingress, and audit/debug rebuilds.
- Cell placement class: Tier-0 per manifest because workflow-engine owns tenant-authored workflow/plugin execution surfaces and must co-locate capacity, runtime isolation, and admission policy.
- Autoscaling boundaries: engine workers keep a 3-pod floor per cell and scale toward 500k active runs, 200k steps/s, and 1M events/s per cell through worker, bus, and Citus shard expansion before admitting new high-volume tenants to another cell.
- Tenant load profile: supports small automation tenants, high-frequency business workflows, and healthcare or incident-response flows without letting replay/debug jobs consume the live dispatch lane.

### E.10 Sustainability and cost attribution (ADR-0344)

- Per-call emission claim: run start, step dispatch, event ingestion, replay, timer fire, adapter call, and audit seal rows emit `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with tenant, workflow template, provider, cell, compliance_pack, and capability axes.
- Carbon-aware provider routing: opt-in only for async replay, non-urgent adapter batches, and background analytics; excluded for EU-AI-Act-Annex-III, HIPAA-EM, PCI-realtime-fraud, emergency dispatch, and Tier-0/Tier-1 SLO-critical run paths.
- Tenant transparency surface: workflow-studio and finops-portal expose per-run, per-step, replay, and adapter chargeback lines so operators can see which workflow templates drive spend and emissions.
- Regulatory driver: CSRD, SB-253, and SEC climate disclosure reporting require durable automation emissions to follow tenant, workflow, and provider dimensions because workflow-engine is shared substrate.

### E.11 API versioning posture (ADR-0342)

- Public API version model: `YYYY-MM-DD` carrier triplet across version header, URL prefix, and proto3 field for workflow spec, run, signal, timer, replay, adapter, and event contracts.
- SDK semver model: workflow SDKs use `major.minor.patch`; major increments are reserved for breaking public date-version changes and replay-incompatible SDK behavior.
- Support window: last 3 public versions are supported for at least 180 days.
- Per-tenant pinning: yes for workflow specs, run execution, adapter contracts, and SDK clients because long-lived workflows may outlive the current API version.
- Internal-mesh exemption: yes; ADR-0145 direct gRPC remains valid for internal worker, replay, bus, and lease coordination.

---

## F. UX Flows

### F.1 Workflow submission

```
[Studio: user clicks Publish]
       |
       v
[Studio compiles visual graph → JSON IR + Ed25519 sign]
       |
       v
[POST /v1/workflows { spec, signature }]
       |
       v
[Engine validates schema + signature]
       |
       v
[Engine stores spec; version_sha = sha256(spec_body)]
       |
       v
[Engine emits WorkflowSpecRegistered event]
       |
       v
[Studio receives version_sha; shows "Published"]
```

### F.2 Run trigger + dispatch

```
[Trigger: webhook OR cron OR event OR manual]
       |
       v
[Engine: load latest spec version]
       |
       v
[Persist Run row to Postgres (state=pending)]
       |
       v
[Engine emits WorkflowStarted event]
       |
       v
[Worker leases run via Valkey]
       |
       v
[Step dispatch: persist StepExecution row]
       |
       v
[Step execute (via adapter)]
       |
       v
[Persist result; emit StepCompleted]
       |
       v
[Repeat for each step]
       |
       v
[Emit WorkflowCompleted + audit-chain seal]
```

### F.3 Replay session

```
[Studio: user clicks Replay on past run]
       |
       v
[POST /v1/replay/sessions { run_id }]
       |
       v
[Engine loads event log]
       |
       v
[Replay engine re-executes steps deterministically; no side effects]
       |
       v
[For each step, emit StepSnapshot via SSE]
       |
       v
[Studio renders step-by-step; user inspects state]
       |
       v
[On replay end: ReplayCompleted]
```

### F.4 SLA breach escalation

```
[Step entry: arm SLA timer (24h)]
       |
       v
[Step completes (normally) → cancel timer]
   OR
[Timer expires → fire SlaTimerFired event]
       |
       v
[Escalation step runs: notify manager via messenger]
       |
       v
[Audit-chain emits SlaBreachEscalated]
```

### F.5 Sub-workflow fan-out

```
[Parent step: spawn 9 children]
       |
       v
[Engine creates 9 child Run rows]
       |
       v
[Parent state: waiting-for-children OR fire-and-forget]
       |
       v
[Each child runs independently]
       |
       v
[Each completion emits ChildCompleted to parent]
       |
       v
[Parent aggregates; resumes with summary]
```

### F.6 Pause + human review

```
[Step body: human_review(payload)]
       |
       v
[Engine pauses run; persists pending-review row]
       |
       v
[Notification dispatched to reviewer via messenger]
       |
       v
[Reviewer opens UI; approves OR rejects]
       |
       v
[Engine resumes run with decision input]
       |
       v
[Audit-chain emits HumanReviewDecided]
```

### F.7 Cross-cell spawn

```
[Step body: cross_cell_spawn(cell_id, spec_id, input)]
       |
       v
[Engine emits CrossCellSpawnRequested event]
       |
       v
[Kafka bridge routes to target cell]
       |
       v
[Target cell engine creates child Run]
       |
       v
[Child runs; on completion, emits ChildCompleted upstream]
       |
       v
[Parent engine receives event; resumes]
```

### F.8 DSAR cascade

```
[DSAR-erasure event received]
       |
       v
[Engine queries Ontology for runs touching subject]
       |
       v
[For each matching run: tombstone step-payload PII fields]
       |
       v
[Retain operational metadata (run_id, timing, state) for audit]
       |
       v
[Emit DsrCompletedForWorkflows event]
```

---

## G. Success Metrics

### G.1 Latency

- P50 run start → first step: ≤ 50 ms.
- P99 step execution (local): ≤ 50 ms.
- P99 event-to-action: ≤ 500 ms.

### G.2 Throughput

- Sustained 10k concurrent runs per cell.
- 5,000 steps/s sustained dispatch per cell.
- 10,000 events/s sustained on event bus per cell.

### G.3 Reliability

- Run durability (no loss across pod eviction): 100%.
- Deterministic replay correctness: 100%.
- Audit-seal latency P99 ≤ 1s.
- Exactly-once-effect step contract honored.

### G.4 Adoption + retention

- Internal-µservice adoption: 100% of µservices use the engine for cross-µservice orchestration (by M03).
- Studio MAU growth ≥ 30% QoQ post-M02.
- Workflow templates in marketplace ≥ 200 at M02; ≥ 1000 by M04.

### G.5 Support

- Tickets per 1k runs ≤ 0.3.
- Average time-to-resolution ≤ 1 business day for substrate incidents.

---

## H. Compliance Impact

Workflow engine touches every other µservice's compliance, so its packs include the union:

| Pack | Standards |
|---|---|
| pack-us | SOC 2 Type II; CCPA/CPRA |
| pack-us-healthcare | HIPAA + BAA chain; PHI in HIPAA-eligible cells |
| pack-eu | GDPR; DSA; AI Act Annex III for agentic steps |
| pack-uk | UK GDPR |
| pack-kr | PIPA; ISMS-P |
| pack-jp | APPI |
| pack-sg | PDPA |
| pack-au | Privacy Act 1988 |
| pack-br | LGPD |

Compliance evidence:

- Per-run audit-chain Ed25519 (ADR-0028).
- Deterministic replay = auditor-grade reconstruction.
- DSAR-cascade tombstoning + retention split.
- Cell-affinity enforcement (PHI in HIPAA cells, EU-PII in EU cells).
- Cedar autonomy-tier ceiling logged per agentic step (EU AI Act Art. 12 record-keeping).

---

## I. Open Questions

| # | Question | Owner | Target |
|---|---|---|---|
| 1 | Durable-execution: bespoke Postgres-state-machine vs embed Temporal SDK (Rust)? Bias: bespoke per autonomous-impl doctrine. | council-architecture | gates IP-005 |
| 2 | Event-bus substrate: Postgres outbox + NOTIFY, NATS JetStream, Kafka KRaft, or Valkey Streams? | council-architecture + ops-sre | gates IP-007 |
| 3 | Workflow DSL: YAML accepted at submit time, JSON-IR canonical? | council-architecture | resolved (yes) |
| 4 | Replay determinism: which std-lib APIs forbidden in step bodies? | council-architecture | docs/standards/workflow-step-determinism.md |
| 5 | Sub-workflow: sync default OR async default? | council-architecture | async default with sync opt-in |
| 6 | Cross-pack workflow spawn: refuse OR allow with overlay policy? | council-privacy + axis-workflow | M02 |
| 7 | Agentic LLM-step autonomy: per-step T0..T3 OR per-session? | council-privacy + axis-intelligence | M03 |
| 8 | n8n / Zapier importer: Day-1 OR M03? | axis-workflow | M02 (basic); M03 (advanced) |

---

## J. Out of Scope

1. **Visual editor UI** — out of scope (lives in `workflow-studio`).
2. **Template marketplace UI** — out of scope (Studio surface).
3. **Per-tenant billing** — out of scope (lives in `cloud-billing`).
4. **OAuth-token brokering for adapter steps** — out of scope (`identity` µservice handles).
5. **Container-image build for custom nodes** — out of scope (Wasmtime-only at M02).
6. **Email/SMS-delivery internals** — out of scope (`mail` + carrier-SMS µservices).
7. **Cron-expression parser** — use `cron` crate; not first-party.
8. **Inference-model hosting** — out of scope (`intelligence` µservice).

---

## K. Bounded Contexts (BC tree)

Per ADR-0105 13-value layer enum + ADR-0106 usecase rename:

| BC | Crate family | Purpose |
|---|---|---|
| `spec-store` | `oya-workflow-engine-spec-store-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,sdk,app}` | Compiled spec storage + versioning |
| `execution-engine` | `oya-workflow-engine-execution-engine-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-valkey,rest,worker,sdk,app}` | Run dispatch + step execution + retry + timer |
| `state-machine` | `oya-workflow-engine-state-machine-{kernel,domain,usecase,api,adapter,adapter-postgres}` | Transition validation + invariants |
| `event-bus` | `oya-workflow-engine-event-bus-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-valkey,rest,worker,sdk,app}` | Typed event pub/sub + outbox |
| `replay-debugger-backend` | `oya-workflow-engine-replay-debugger-backend-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-clickhouse,rest,sdk,app}` | Deterministic replay + analytics |
| `trigger-orchestrator` | `oya-workflow-engine-trigger-orchestrator-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` | Cron + webhook + event triggers |
| `plugin-substrate` | `oya-workflow-engine-plugin-substrate-{kernel,domain,usecase,api,adapter,adapter-wasmtime,sdk,app}` | Custom-node WASM sandbox |
| `audit-chain-bridge` | `oya-workflow-engine-audit-chain-bridge-{kernel,domain,usecase,api,adapter,worker,sdk}` | Ed25519 seal emission |
| `cross-cell-bridge` | `oya-workflow-engine-cross-cell-bridge-{kernel,domain,usecase,api,adapter,adapter-kafka,worker,sdk}` | Kafka-bridge across cells |
| `cedar-fragment-coverage` | `oya-workflow-engine-cedar-fragment-coverage-{kernel,domain,usecase,api,adapter}` | Cedar gate per step type |

Total crates ~50 across 10 BCs.

---

## L. Integration Surface

### L.1 Workflow events produced

| Event type | Trigger |
|---|---|
| `workflow.spec.registered` | spec save |
| `workflow.run.started` | run init |
| `workflow.step.started` | step dispatch |
| `workflow.step.completed` | step success |
| `workflow.step.failed` | step exhausted retry |
| `workflow.step.retried` | retry attempt |
| `workflow.paused` | operator intervention |
| `workflow.resumed` | operator intervention |
| `workflow.cancelled` | operator or self |
| `workflow.completed` | terminal success |
| `workflow.failed` | terminal failure |
| `workflow.sla.armed` | step entry |
| `workflow.sla.fired` | timer expiry |
| `workflow.child.completed` | sub-workflow done |
| `workflow.cross_cell.spawned` | cross-cell spawn |
| `workflow.signal.received` | signal handler |

### L.2 Workflow events consumed

Engine consumes ANY typed event published by any µservice on the bus. Examples:

| Event type | Producer |
|---|---|
| `identity.user.provisioned` | identity |
| `payments.subscription.renewed` | payments |
| `messenger.message.received` | messenger |
| `shorts.publish.completed` | shorts |
| `tenancy.tenant.onboarded` | tenancy |

### L.3 Ontology writes

| Object Type | Written by BC |
|---|---|
| `workflow::WorkflowSpec` | spec-store |
| `workflow::WorkflowRun` | execution-engine |
| `workflow::StepExecution` | execution-engine |

### L.4 Ontology reads

| Object Type | Read by BC |
|---|---|
| `tenancy::Tenant` | execution-engine (limits) |
| `compliance::CompliancePack` | execution-engine + cedar-fragment-coverage |

---

## M. Acceptance criteria

| ID | Criterion | Verification |
|---|---|---|
| AC-W-01 | Run start → first step P99 ≤ 200ms | k6 load |
| AC-W-02 | Deterministic replay correctness 100% | nextest |
| AC-W-03 | Engine kill mid-run → resume from last completed step | e2e |
| AC-W-04 | Pod eviction → resume on different node | e2e |
| AC-W-05 | 10k concurrent runs per cell | k6 |
| AC-W-06 | Tenant isolation at bus + state + audit | nextest |
| AC-W-07 | Per-run audit-chain seal | nextest |
| AC-W-08 | LEAN-A2: no product µservice imports | CI lane |
| AC-W-09 | Outbox crash recovery | e2e |
| AC-W-10 | SLA timer accuracy | nextest |
| AC-W-11 | Replay throughput ≥ 1k steps/s/worker | nextest |
| AC-W-12 | Spec signature verification | nextest |
| AC-W-13 | Per-µservice flat layout green | CI lane |
| AC-W-14 | Authority cohesion | CI lane |
| AC-W-15 | Deterministic-replay lane green | CI lane |
| AC-W-16 | Cross-cell spawn round-trip ≤ 800ms P99 | e2e |
| AC-W-17 | Plugin sandbox: no filesystem | nextest |
| AC-W-18 | Cedar autonomy-tier escalation refused | nextest |
| AC-W-19 | DSAR cascade tombstones PII | e2e |
| AC-W-20 | HIPAA cell-affinity enforced | nextest |

---

## N. Performance evidence

### N.1 Modeling notes

- `docs/performance-budgets/workflow-engine-step-latency.md` (TBD M02) — decomposes 200ms P99 step dispatch into: Postgres write (25ms), Cedar eval (10ms), worker lease (5ms), step body local (50ms), audit-emit (10ms), buffer (100ms).
- `docs/performance-budgets/workflow-engine-replay-throughput.md` (TBD M02) — decomposes 1k steps/s/worker into: event-log fetch (200µs/step), state-reconstruct (300µs), step replay-execute (400µs), buffer (100µs).

### N.2 Hyperscaler benchmark comparisons

- **Temporal Cloud**: undocumented P99 typically ~1s for step dispatch.
- **AWS Step Functions**: P99 transition ~25ms; whole-state-machine throughput limited per-account.
- **Airflow scheduler**: parsing-throughput-bound; not directly comparable.
- **n8n**: typical step latency ~50ms.
- **oyatie target**: P99 step dispatch ≤ 200ms; deterministic replay; per-tenant isolation.

### N.3 Sensitivity analysis

- Postgres write latency dominates P99 step dispatch (~12%).
- Worker leasing (Valkey) is the next-largest contributor.
- Cross-cell spawn is bounded by Kafka-bridge latency.

---

## O. Migration + rollout

### O.1 M02 ship plan

- Week-1 to Week-4: spec-store + execution-engine basic (single-cell, no replay).
- Week-5 to Week-8: state-machine + event-bus.
- Week-9 to Week-12: replay-debugger-backend + audit-chain bridge.
- Week-13 to Week-16: trigger-orchestrator + plugin substrate + cross-cell bridge.
- Week-17 to Week-20: cedar-fragment-coverage + DSAR cascade.
- Week-21 to Week-22: E2E + load + chaos.
- Week-23 to Week-26: M02 ship.

### O.2 M03 expansion

- 200+ integration adapters in marketplace.
- n8n / Zapier importers.
- Advanced agentic-step orchestration with Cedar autonomy tiers.
- Per-pack compliance overlays (HIPAA, AI Act).

### O.3 M04+ enhancements

- Multi-region active-active per pack.
- Container-image custom nodes (beyond Wasmtime).
- Visual debugger AI-assist.
- Workflow recommender (suggest next steps).

---

## P. Cross-Slice References (to be added)

- **Slice ADR-author** — link to any new workflow-engine-specific ADRs beyond the keystone bundle.
- **Slice runbook-author** — `microservices/workflow-engine/runbooks/run-queue-saturation.md`, `outbox-relay-failure.md`, `replay-incident.md`, `cross-cell-bridge-down.md`.
- **Slice spec-author** — `/specs/microservices/workflow-engine.json` for spec JSON-IR + run-state schema.
- **Slice user-story-bank** — extend `b2b-work-surfaces.md` and `b2c-consumer-surfaces.md` with workflow product-surface stories referencing this PRD.
- **Slice testing-strategy** — `microservices/workflow-engine/testing-strategy.md` for E2E, chaos (engine kill, pod evict, region fail), determinism property-based, plugin-sandbox fuzz.
- **Slice synthesis** — keystone-bundle synthesis doc.
- **Slice memory** — `feedback_workflow_engine_substrate_2026_05_20.md` capture.

---

## Q. Sample Workflow Spec (JSON IR)

```json
{
  "workflow_id": "wf_employee_onboarding_v3",
  "version_sha": "sha256:abc...",
  "tenant_id": "t_acme",
  "compliance_packs": ["pack-us"],
  "trigger": {
    "type": "event",
    "event_type": "identity.user.provisioned"
  },
  "steps": [
    {
      "id": "step_1_provision_messenger",
      "type": "ms_action",
      "ms": "messenger",
      "action": "create_workspace_account",
      "input": "{{trigger.user_id}}",
      "retry": { "attempts": 3, "backoff": "exponential" },
      "sla": "5m"
    },
    {
      "id": "step_2_fan_out",
      "type": "parallel",
      "branches": [
        { "id": "branch_payroll", "ms_action": "payroll.enroll" },
        { "id": "branch_calendar", "ms_action": "calendar.init" },
        { "id": "branch_drive", "ms_action": "drive.quota_init" }
      ]
    },
    {
      "id": "step_3_welcome_message",
      "type": "ms_action",
      "ms": "messenger",
      "action": "send_message",
      "input": { "to": "{{trigger.user_id}}", "template": "welcome_day_1" },
      "compensation": "messenger.unsend_message"
    }
  ],
  "signature": "ed25519:..."
}
```

---

## R. Sample step body (Rust SDK)

```rust
use oya_workflow_engine_sdk::*;

#[workflow]
pub async fn employee_onboarding(ctx: Context, user_id: UserId) -> Result<()> {
    // Step 1
    ctx.exec(messenger::CreateWorkspaceAccount { user_id }).await?;

    // Step 2: parallel fan-out
    let (payroll_r, calendar_r, drive_r) = tokio::join!(
        ctx.exec(payroll::Enroll { user_id }),
        ctx.exec(calendar::Init { user_id }),
        ctx.exec(drive::QuotaInit { user_id }),
    );
    payroll_r?; calendar_r?; drive_r?;

    // Step 3: welcome message
    ctx.exec(messenger::SendMessage {
        to: user_id,
        template: "welcome_day_1",
    }).await?;

    Ok(())
}
```

---

## S. Sample event payload

```json
{
  "id": "evt_01HZX...",
  "type": "workflow.step.completed",
  "tenant_id": "t_acme",
  "occurred_at": "2026-05-20T14:32:11.420Z",
  "data": {
    "run_id": "run_01HZX...",
    "step_id": "step_1_provision_messenger",
    "status": "succeeded",
    "duration_ms": 142,
    "retry_count": 0
  },
  "_meta": {
    "audit_chain_seal": "sha256:abc...",
    "spec_version_sha": "sha256:def..."
  }
}
```

---

## T. Adapter catalog (200+ integrations day-one — sample)

### T.1 Productivity + collaboration

- Gmail, Google Calendar, Google Drive, Google Sheets, Google Docs, Google Meet
- Microsoft Outlook, Exchange, Teams, OneDrive, SharePoint
- Slack, Notion, Asana, Trello, Linear, Monday.com, ClickUp
- Calendly, Cal.com
- oyatie's own: messenger, mail, calendar, drive, notes, community

### T.2 Developer tools

- GitHub (issues, PRs, actions, releases)
- GitLab
- Bitbucket
- Jira, Confluence
- PagerDuty, Opsgenie
- DataDog, New Relic
- Sentry, Bugsnag, Rollbar
- Docker, K8s
- AWS (S3, EC2, Lambda, SQS, SNS, RDS, DynamoDB, CloudWatch)
- GCP (Cloud Storage, Cloud Run, Pub/Sub, BigQuery)
- Azure (Blob, Functions, Service Bus)
- Cloudflare (Workers, Pages, R2)
- oyatie internal: foundry, observability, ops-dashboard

### T.3 CRM + sales

- Salesforce
- HubSpot
- Zoho CRM
- Pipedrive
- Microsoft Dynamics 365

### T.4 Customer support

- Zendesk
- Intercom
- Freshdesk
- Front

### T.5 Marketing

- Mailchimp, SendGrid, Postmark
- Hootsuite, Buffer
- Twitter / X API
- Meta Graph API (FB + IG)
- TikTok for Business

### T.6 Finance + accounting

- Stripe, Adyen (via payments µservice)
- QuickBooks, Xero, NetSuite, SAP
- Plaid

### T.7 HR + people-ops

- BambooHR, Rippling, Workday, Personio
- Greenhouse, Lever
- DocuSign, HelloSign
- Saramin (KR), Recruit (KR)

### T.8 Communications + messaging

- Twilio (SMS, voice, WhatsApp Business)
- SendGrid (email)
- Discord (webhooks)
- Telegram Bot API
- KakaoTalk Channel API (KR)
- LINE Notify (JP)
- WeCom (CN enterprise)

### T.9 E-commerce

- Shopify
- WooCommerce
- Stripe Checkout
- BigCommerce

### T.10 AI + ML

- OpenAI API
- Anthropic API (Claude)
- Google Gemini API
- AWS Bedrock
- Hugging Face Inference
- oyatie's own `intelligence` µservice

### T.11 IoT + smart home

- Hue Bridge
- Nest
- SmartThings
- Home Assistant

### T.12 Healthcare

- FHIR API (R4 + R5)
- Athena Health (USA)
- Epic MyChart (limited)
- Cerner / Oracle Health

### T.13 Supply chain + logistics

- Shopify Shipping
- Easypost
- ShipEngine
- FedEx, UPS, DHL APIs

### T.14 Custom HTTP

- Generic HTTP REST adapter (POST/GET/PUT/DELETE/PATCH).
- Generic gRPC adapter.
- Generic GraphQL adapter.
- Webhook in / webhook out.

Total adapter count target: 200+ at M02; 500+ by M04.

---

## U. Internal SDK API (Rust)

### U.1 Spec submission

```rust
use oya_workflow_engine_sdk::*;

let spec = WorkflowSpec::builder("employee_onboarding_v3")
    .trigger(Trigger::Event {
        event_type: "identity.user.provisioned".into(),
    })
    .step("step_1", Step::ms_action("messenger.create_workspace_account"))
    .step("step_2_fan_out", Step::parallel([
        Step::ms_action("payroll.enroll"),
        Step::ms_action("calendar.init"),
        Step::ms_action("drive.quota_init"),
    ]))
    .step("step_3", Step::ms_action("messenger.send_message"))
    .build();

let response = client.workflows().publish(&spec).await?;
println!("Published as {}", response.version_sha);
```

### U.2 Event publish

```rust
client.events().publish(WorkflowEvent {
    event_type: "shorts.publish.completed".into(),
    tenant_id: "t_acme".into(),
    data: json!({ "shorts_id": "sh_01HZX..." }),
}).await?;
```

### U.3 Event subscribe

```rust
let subscription = client.subscriptions().create(SubscriptionRequest {
    event_types: vec!["payments.subscription.renewed".into()],
    backpressure_buffer: 1000,
}).await?;

while let Some(event) = subscription.next().await? {
    process(event).await?;
    subscription.ack().await?;
}
```

### U.4 Signal + query

```rust
client.runs().signal(run_id, "update_threshold", json!({ "value": 100 })).await?;
let state = client.runs().query(run_id, "current_state").await?;
```

---

## V. Determinism contract details

### V.1 Forbidden std-lib APIs inside step bodies

- `std::time::SystemTime::now()` → use `ctx.now()`.
- `std::time::Instant::now()` (for relative-time only — but use `ctx.elapsed_since(...)`).
- `rand::random()` → use `ctx.random_u64()`.
- `tokio::spawn()` → use `ctx.spawn_child()`.
- Direct env-var reads → use `ctx.env(key)` for deterministic substitution.
- Filesystem I/O → declared via adapter contract.
- Network I/O → declared via adapter contract.

### V.2 Engine-provided deterministic APIs

```rust
impl Context {
    pub fn now(&self) -> DateTime<Utc>;            // deterministic per replay
    pub fn random_u64(&self) -> u64;                // seeded from event log
    pub fn elapsed_since(&self, t: Instant) -> Duration;
    pub fn env(&self, key: &str) -> Option<String>;
    pub fn exec<A: Action>(&self, action: A) -> impl Future<Output = Result<A::Output>>;
    pub fn spawn_child<S: Spec>(&self, spec: S) -> impl Future<Output = Result<ChildHandle>>;
}
```

### V.3 CI lane enforcement

`oya gate validate deterministic-replay --microservice workflow-engine` performs:

- AST-scan of step bodies for forbidden std-lib references.
- Property-based replay test: run N times; assert identical step-sequence.
- Coverage of all step types in spec catalog.

---

## W. Observability + SLO

### W.1 Per-engine metrics

- `engine_runs_active`: gauge per cell + tenant.
- `engine_step_dispatch_latency_seconds`: histogram per spec.
- `engine_replay_throughput_steps_per_second`: gauge per worker.
- `engine_audit_seal_latency_seconds`: histogram.
- `engine_queue_depth`: gauge per tenant.
- `engine_outbox_lag_seconds`: gauge per cell.

### W.2 SLO authoring

Per ADR-0139 every engine SLO authored under `microservices/workflow-engine/slos/`:

- `slos/run-start-latency.openslo.yaml` — 99.9% P99 ≤ 200ms.
- `slos/step-execution-latency.openslo.yaml` — 99.9% P99 ≤ 50ms (local).
- `slos/audit-seal-latency.openslo.yaml` — 99.99% P99 ≤ 1s.
- `slos/replay-throughput.openslo.yaml` — 99% ≥ 1k steps/s/worker.
- `slos/outbox-lag.openslo.yaml` — 99.9% lag < 1s.

### W.3 Burn-rate alerts

- 14.4× burn over 1h → page on-call (high-burn-rate, low-window).
- 6× burn over 6h → ticket (medium-burn-rate, mid-window).
- 3× burn over 24h → email (low-burn-rate, wide-window).

---

## X. Change log

- **2026-05-20** — Comprehensive rewrite (from 459-line stub to ≥1500-line PRD) as part of keystone-bundle 2026-05-20 foundational-doctrine documentation pass. Closes `feedback_autonomous_implementation_artifacts` gap: workflow-engine is hero substrate paired with workflow-studio hero product and MUST be intern-buildable from doc alone. Adds B2C + B2B personas + ≥40 stories + ≥6 UX flows + sample spec + sample SDK + sample event + compliance per pack + cross-µservice integration surface + 200+ adapter catalog + determinism contract + observability/SLO.
- **2026-05-17** — Initial stub publication (459 lines).

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is legacy/local-feedback provenance only after ADR-0515; protected merge authority is `oya-ci-required`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins CI wording is historical/provenance after ADR-0515; GitHub Actions produces `oya-ci-required` until explicit owned-runner cutover, while ArgoCD remains separately authorized CD evidence with cosign, tenant namespace, and audit-chain controls.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `workflow-engine` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `workflow-engine` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 3 module pin(s) across 1 context(s).
- Scaling input: `per_workflow_run` with cell placement `Tier-0` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
