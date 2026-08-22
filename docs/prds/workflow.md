---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-workflow
microservice: workflow
status: Accepted
sales_segment: shared-substrate
tier: B2B
milestone_first_ship: M02b-substrate-ready
bominal_source:
  - ADR-0035   # Workflow engine (hybrid state machine + DAG)
  - ADR-0103   # Workflow hexagonal migration
  - ADR-0121   # Workflow Studio Light scope (M03 launch presets)
  - ADR-0148   # Workflow engine (extended)
  - ADR-0028   # audit chain (every workflow run sealed)
  - ADR-0107   # agent gateway (agentic mode nodes)
  - ADR-0132   # pillars (org/person enforcement on workflow runs)
  - ADR-0037   # plugin substrate (custom nodes via WASM)
  - ADR-0018   # tenancy RLS posture
  - ADR-0009   # cell architecture
  - ADR-0019   # runtime target metadata model (active-active compatibility)
doc_status: published
amended_by:
  - ADR-0565 (amends the studio bounded-context and clean-architecture surface set; the studio-rest and studio-sdk BCs are unchanged)
---

# PRD-workflow: Workflow Studio (Shared Substrate + Hero Product)

---

## Purpose

Workflow Studio is oyatie's **first hero product** and simultaneously the
ecosystem's action/orchestration adapter layer. It is an **n8n-class** visual
workflow product that captures any business or operational logic: agentic
workflows, developer CI/CD pipelines, business approvals, healthcare clinical
handoffs, supply-chain orchestration, and delivery logistics.

**Dual nature** (per `feedback_workflow_studio_scope.md`):

1. **Shared substrate** (always-on adapter): all inter-µservice action flows
   route through Workflow. Products emit typed events; Workflow routes them via
   state machines + DAGs; consumers subscribe. Direct product-to-product calls
   are prohibited (LEAN-A2). This is the load-bearing architectural rule
   (per `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md`).

2. **End-user product** (B2B SaaS): tenant teams author, test, deploy, and
   monitor their own workflows via the visual Studio editor. No engineering
   required for business-operational automation.

Workflow is placed in the **shared substrate** (not Corporate-owned per
`feedback_workflow_is_shared.md` override of Bominal ADR-0232). It underlies
Healthcare, Enterprise, FinTech, Connect, and every other µservice equally.

Inherits Bominal ADR-0035 (hybrid state machine + DAG engine), ADR-0103
(hexagonal migration), ADR-0121 (Studio Light M03 presets), ADR-0148
(extended engine). Bominal's Corporate-ownership stance overridden: Workflow
is shared/* per oyatie session decision.

---

## Tenant Value

- **Visual workflow authoring**: drag-drop node canvas; no-code for business
  users; full-code option for developers; version history; debug panel.
- **Any workflow domain**: one product for agentic AI pipelines, developer
  CI/CD, HR approval chains, clinical event routing, shipment tracking — no
  separate tools per domain.
- **Durable execution**: long-running workflows (days/weeks); deterministic
  replay; crash-safe (Temporal parity); never lose a workflow run.
- **Integration-library**: 50+ connectors at M03 launch (internal µservices
  + KR carriers + government APIs + major SaaS); Zapier breadth roadmap.
- **Agentic mode**: LLM-powered decision nodes via Ontology agent gateway
  (ADR-0107); agent reads Ontology, decides next action, Workflow executes.
- **Ecosystem glue**: all HR/Payroll/Connect/Accounting cross-µservice events
  route through Workflow automatically; tenant sees automation as a first-class
  product, not infrastructure.

---

## Functional Requirements

### Visual Studio Editor

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | Business user | drag nodes onto a canvas and connect them with edges | I build a workflow without writing code | `studio` | Must |
| FR-02 | Business user | configure each node (trigger, action, condition, loop) via a panel | workflow logic is precisely defined | `studio` | Must |
| FR-03 | Developer | author workflows in YAML/JSON DSL; sync bidirectionally with canvas | code-first authoring for complex workflows | `studio` | Must |
| FR-04 | Any user | view version history; diff two versions; roll back | mistakes are recoverable | `studio` | Must |
| FR-05 | Any user | run a workflow in debug mode; step through nodes; inspect payloads | errors found before production | `studio` | Must |
| FR-06 | Any user | view live run history; filter by status; inspect individual run trace | production monitoring done in Studio | `studio` | Must |

### Trigger Types

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-07 | Any user | trigger a workflow on a cron schedule | scheduled automations run without manual intervention | `triggers` | Must |
| FR-08 | Developer | trigger via webhook (inbound HTTP POST) | external systems start workflows | `triggers` | Must |
| FR-09 | Product µservice | trigger on an eventing topic event | Workflow subscribes to business events automatically | `triggers` | Must |
| FR-10 | Product µservice | trigger on an Ontology entity change (ontology-event) | entity mutations start downstream automation | `triggers` | Must |
| FR-11 | Any user | trigger manually (button in Studio or API call) | ad-hoc workflows run on demand | `triggers` | Must |

### Action Types + Logic

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-12 | Any user | call an external HTTP endpoint as a workflow action | third-party services integrated without custom code | `integrations` | Must |
| FR-13 | Any user | invoke an Ontology Action Type as a workflow step | entity mutations happen inside workflow runs; audit-sealed | `engine` | Must |
| FR-14 | Any user | call an LLM via the agent gateway node; use the result for next-step routing | agentic decision-making embedded in business workflows | `automations` | Must |
| FR-15 | Any user | branch on a condition; switch on a value; loop over a collection | complex logic expressible without code | `engine` | Must |
| FR-16 | Any user | configure retry count + backoff on any action node | transient failures handled automatically | `engine` | Must |
| FR-17 | Any user | configure a dead-letter handler node; receive alert on failure | no silent failures; all errors visible | `engine` | Must |
| FR-18 | Any user | invoke a sub-workflow as a node | reusable workflow components; DRY principle | `engine` | Must |

### Approvals + SLA

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-19 | HR admin | create a multi-step approval chain (manager → HR → finance) | expense/leave/payroll-close approvals routed correctly | `approvals` | Must |
| FR-20 | Any user | configure a four-eyes constraint on an approval step | sensitive actions require two independent approvers | `approvals` | Must |
| FR-21 | Any user | set an SLA timer on a workflow step; escalate on breach | SLA compliance tracked and enforced automatically | `sla` | Must |

### Templates + Governance

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-22 | Tenant admin | browse domain templates (Agentic / Developer / Business / Healthcare / Supply Chain / Delivery) | starting from a proven template is faster than from scratch | `studio` | Must |
| FR-23 | Tenant admin | require workflow approval before activation in production | no unapproved automations run in prod | `studio` | Must |
| FR-24 | Platform operator | publish curated templates to the tenant template library | tenants benefit from Oyatie-maintained workflow patterns | `studio` | Should |

### Integrations (connector library)

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-25 | Any user | connect to internal µservices (HR, Payroll, Accounting, Connect, Medical, Payments) | oyatie ecosystem integrations are first-class nodes | `integrations` | Must |
| FR-26 | Any user | connect to KR government APIs (NHIS EDI, NPS, 근로복지공단, HIRA) | regulatory automations built without custom integration code | `integrations` | Must |
| FR-27 | Any user | connect to KR carriers (CJ대한통운, 로젠, 한진, 우체국) | shipment tracking and handoff workflows supported | `integrations` | Should |
| FR-28 | Developer | author a custom node via the plugin SDK (WASM, per ADR-0037) | proprietary integrations built without forking oyatie core | `integrations` | Should |

---

## Non-Functional Requirements

### Performance
Per `feedback_quality_performance_scalability_bar.md` + `feedback_workflow_studio_scope.md`:

- Visual editor first-paint (web, Leptos SSR): ≤500 ms.
- Node-drop → edge-connect: ≤16 ms (60 fps; canvas operations are local-first).
- Workflow save → trigger-ready: ≤100 ms.
- Workflow execution start → first step: ≤200 ms.
- 10k+ concurrent active workflow runs per cell.
- Sub-second event-to-action latency (outbox → Kafka KRaft → worker → step).
- Long-running workflow durability: weeks (Temporal parity); deterministic replay.
- Step execution P99: ≤200 ms for local actions; network-bound external actions
  excluded from SLO (measured separately).

### Security
- JWT `tenant_id` enforced; per-tenant workflow library; per-tenant run isolation.
- Cedar policy gates: which roles can create/activate/delete workflows; which
  Action Types each role can invoke; agentic nodes gated per ADR-0107 +
  ADR-0132 pillars.
- Plugin SDK (ADR-0037): custom nodes run in Wasmtime sandbox; no host
  filesystem access; memory limit enforced per node execution.
- Webhook inbound: HMAC signature verification required; replay-attack window ≤5 min.

### Audit + Compliance
- Every workflow run Merkle/Ed25519-sealed per (tenant_id, run_id) per ADR-0028;
  seal latency ≤1 s.
- Deterministic replay: given the same initial state + event log, workflow
  re-execution produces identical step sequence (Temporal parity; required for
  audit and debugging).
- Jurisdiction overlay per ADR-0127; GDPR: workflow run payloads classified
  by data tier (ADR-0119); PII fields encrypted at rest (ciphertext property
  type per ADR-0111).

### Availability + SLO
- 99.95% monthly for workflow engine (execution path).
- 99.9% for Studio editor (authoring path; degraded-graceful on engine unavailability).
- RTO ≤15 s per-cell; RPO ≤5 s (outbox durability).

---

## Bounded Contexts

| BC name | Crate family (BNF v4.1) | Purpose | Key entities |
|---|---|---|---|
| `studio` | `workflow-studio-{rest,sdk}` | Visual editor authoring surface; version history; template library | `WorkflowDefinition`, `WorkflowVersion` |
| `engine` | `workflow-engine-{kernel,domain,application,adapter,worker}` | Runtime: state machines, DAGs, transitions, schedulers, step executors | `WorkflowRun`, `StepExecution` |
| `transitions` | `workflow-transitions-{kernel,domain}` | Transition rules + invariants; state machine graph validation | `Transition`, `TransitionRule` |
| `approvals` | `workflow-approvals-{kernel,domain,application}` | Multi-step + four-eyes approval primitive | `ApprovalRequest`, `ApprovalDecision` |
| `sla` | `workflow-sla-{domain,worker}` | SLA timers; escalation chains; breach detection | `SlaTimer`, `Escalation` |
| `automations` | `workflow-automations-{domain,worker}` | Agentic runner (LLM via ADR-0107) + scripted automation workers | `AutomationRun` |
| `triggers` | `workflow-triggers-{adapter}` | Cron / webhook / event-topic / ontology-event / manual / API trigger sources | `TriggerConfig`, `TriggerEvent` |
| `integrations` | `workflow-integrations-{adapter}` | Connector library; HTTP action; internal µservice nodes; KR carrier nodes; plugin SDK | `ConnectorConfig`, `ConnectorExecution` |
| `app` | `workflow-app` | Composition-root binary; wires all BCs; starts HTTP server + workers | — |

### Clean Architecture Layer Map

Dependency direction: strictly inward-only. Per `feedback_clean_architecture_requirements.md`.

```
{studio-rest, triggers-adapter, integrations-adapter, worker}
        ↑ depends on
   {engine-adapter, automations-worker, sla-worker}
        ↑ depends on
   {engine-application, approvals-application}
        ↑ depends on
   {engine-domain, transitions-domain, approvals-domain, sla-domain}
        ↑ depends on
   {engine-kernel, transitions-kernel, approvals-kernel}  ← studio-sdk
        ↑
   workflow-app  (composition root; wires all layers)
```

Port traits live in `kernel` — ZERO business logic, ZERO I/O:

```rust
// workflow-engine-kernel/src/ports.rs

#[doc(hidden)]
mod sealed { pub trait Sealed {} }

/// State persistence port — implemented in workflow-engine-adapter
#[async_trait::async_trait]
pub trait WorkflowStateStore: Send + Sync + sealed::Sealed {
    async fn load_run(&self, run_id: &RunId) -> Result<WorkflowRun, StoreError>;
    async fn save_run(&self, run: &WorkflowRun) -> Result<(), StoreError>;
    async fn save_step(&self, step: &StepExecution) -> Result<(), StoreError>;
}

/// Transition engine port — implemented in workflow-engine-adapter
#[async_trait::async_trait]
pub trait TransitionEngine: Send + Sync + sealed::Sealed {
    async fn evaluate(&self, run: &WorkflowRun, event: &WorkflowEvent)
        -> Result<Transition, TransitionError>;
}

/// Event bus port — implemented in workflow-engine-adapter (Kafka KRaft)
#[async_trait::async_trait]
pub trait EventBus: Send + Sync + sealed::Sealed {
    async fn publish(&self, topic: &str, event: &WorkflowEvent) -> Result<(), BusError>;
    async fn subscribe(&self, topic: &str) -> Result<EventStream, BusError>;
}

// workflow-approvals-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait ApprovalStore: Send + Sync + sealed::Sealed {
    async fn create_request(&self, req: &ApprovalRequest) -> Result<ApprovalId, StoreError>;
    async fn record_decision(&self, decision: &ApprovalDecision) -> Result<(), StoreError>;
    async fn load_request(&self, id: &ApprovalId) -> Result<ApprovalRequest, StoreError>;
}

// workflow-sla-kernel/src/ports.rs (in transitions-kernel crate)
#[async_trait::async_trait]
pub trait SlaTimerStore: Send + Sync + sealed::Sealed {
    async fn arm_timer(&self, timer: &SlaTimer) -> Result<(), StoreError>;
    async fn cancel_timer(&self, timer_id: &TimerId) -> Result<(), StoreError>;
}
```

Implementations: `workflow-engine-adapter` (Postgres + Citus state store;
Kafka KRaft event bus). Domain calls through ports; domain never imports adapter.

```
NAME: workflow-engine-kernel
JUSTIFICATION:
- microservice = workflow: Workflow µservice; flat catalog; shared substrate; ADR-0056 v4.1; no "shared|vertical" bisection; "workflow" IS the µservice name (not "shared-workflow" — BNF v4.1 drops shared slot)
- bc-tokens = engine: workflow has multiple BCs (studio/engine/transitions/approvals/sla/automations/triggers/integrations); engine BC owns WorkflowRun entity + StepExecution + transition engine port-traits; ADR-0056 v4.1 BC-optionality
- layer = kernel: shared WorkflowId + RunId value types + port-traits consumed cross-layer; no I/O; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: workflow-studio-rest
JUSTIFICATION:
- microservice = workflow
- bc-tokens = studio: visual editor authoring surface; owns WorkflowDefinition + version history; distinct from engine (runtime) BC
- layer = rest: HTTP handler wiring for Studio authoring API; maps HTTP → application commands; no business logic; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: workflow-app
JUSTIFICATION:
- microservice = workflow
- bc-tokens: OMITTED — composition-root binary assembles all BCs; no BC-level split at app layer
- layer = app: composition-root binary; main.rs; wires all infrastructure impls; ADR-0056 §"Layer semantics"
- exemptions: none
```

---

## Multi-Domain Template Library (M03 launch set)

Templates shipped at M03 GA. Each template is a `WorkflowDefinition` in the
tenant template library, authored and curated by oyatie. Tenants clone and customize.

| Domain | Template name | Trigger | Key nodes | Description |
|---|---|---|---|---|
| **Agentic** | Agent Task Orchestrator | Manual / API | LLM-decide → Ontology-read → action-branch → human-review | Agent decides next step via LLM; routes to Ontology Action Types |
| **Developer** | PR Review Pipeline | GitHub webhook | PR-opened → assign-reviewer → review-check → merge-gate | Automated PR assignment + merge enforcement |
| **Business** | Leave Request Approval | Manual | submit → manager-approve → HR-approve → calendar-block | 2-step approval; SLA timer; escalation on breach |
| **Business** | Expense Approval | Ontology-event | expense-created → amount-branch → approver-chain → accounting-post | Amount-based routing; posts to Accounting on approval |
| **Business** | Payroll Close | Cron (month-end) | payroll-run → review-gate → accounting-journal → payslip-distribute | End-to-end payroll close with approval gate |
| **Healthcare** | Clinical Discharge | Ontology-event | patient-discharge → care-summary → pharmacy-check → transport-arrange | Multi-department handoff; SLA on each step |
| **Healthcare** | Prescription Lifecycle | Ontology-event | prescription-created → pharmacy-dispense → patient-notify → refill-reminder | Prescription routing with cron refill reminder |
| **Supply Chain** | PO → Receipt → Reorder | Ontology-event | PO-approved → receipt-check → inventory-update → reorder-threshold | End-to-end procurement-to-inventory automation |
| **Delivery** | Shipment Tracking | Webhook (carrier) | shipment-created → carrier-handoff → status-poll → POD-confirm | CJ대한통운/로젠 carrier integration; POD confirmation |

---

## Integration via Workflow + Ontology

Workflow IS the action adapter. All inter-µservice events route through it.

### Events consumed (substrate role — all µservices)

Workflow subscribes to ALL product event topics via the triggers/adapter layer.
Key examples:

| Event type | Produced by | Trigger type | Workflow action |
|---|---|---|---|
| `EmployeeHired` | `hr` | ontology-event | Fan-out: provisioning-sm, payroll-enrollment-sm, connect-provisioning-sm |
| `PayrollRunCompleted` | `payroll` | event-topic | Trigger accounting journal-posting workflow |
| `ApprovalRequested` (any) | any µservice | event-topic | Route to correct approval chain; notify approvers via |
| `TenantActivated` | `tenancy` | event-topic | Trigger product-onboarding workflows per enabled products |

### Workflow events produced (to Ontology + audit)

| Object Type | Written by | Purpose |
|---|---|---|
| `WorkflowRun` | `engine` | Run record; queryable by Studio and consuming µservices |
| `StepExecution` | `engine` | Per-step record; audit trail for deterministic replay |
| `ApprovalDecision` | `approvals` | Sealed decision record |

---

## Competitive Benchmark

| Competitor | Product | Parity dimensions | Primary source |
|---|---|---|---|
| **n8n** | n8n (self-host) | Node-based canvas UX; node library breadth; webhook/cron triggers; self-host posture; community templates | https://docs.n8n.io |
| **Zapier** | Zapier Automation | Consumer/SMB breadth (5000+ integrations); Zap authoring UX; multi-step Zaps; error handling | https://zapier.com/apps |
| **Make.com** | Make (Integromat) | Visual canvas craft; scenario debugger; conditional routing; iterator/aggregator nodes | https://www.make.com/en/help/scenarios |
| **Workato** | Workato Enterprise | Recipe versioning; data transformation; enterprise governance; smart connector SDK | https://docs.workato.com |
| **Temporal** | Temporal Cloud | Durable workflow execution; deterministic replay; multi-week durability; workflow versioning; event sourcing | https://docs.temporal.io |
| **Camunda** | Camunda Platform 8 | BPMN 2.0 modeling; process governance; approval workflow; DMN decision tables | https://docs.camunda.io |
| **Apache Airflow** | Airflow 2 | DAG-based pipelines; scheduler; task retries; sensor operators; backfill | https://airflow.apache.org/docs |
| **GitHub Actions** | GitHub Actions | Developer YAML DSL; matrix builds; reusable workflows; marketplace; secret management | https://docs.github.com/en/actions |
| **Linear** | Linear Workflow Automation | UX excellence; opinionated craft; keyboard-driven; trigger-action rules; clean UI | https://linear.app/docs/automations |
| **Slack** | Workflow Builder | Simple no-code; chat-integrated actions; form triggers; approval steps | https://slack.com/help/articles/17542172840595 |
| **ServiceNow** | Flow Designer | Enterprise ITSM; governance; multi-step approvals; SLA enforcement; catalog integrations | https://docs.servicenow.com/flow-designer |
| **Power Automate** | Microsoft Power Automate | AI Builder integration; Office 365 connectors; approval flows; RPA bridge | https://learn.microsoft.com/en-us/power-automate |
| **Salesforce Flow** | Salesforce Flow | CRM-specific declarative; screen flows; approval chains; record-triggered automation | https://help.salesforce.com/s/articleView?id=sf.flow.htm |

Key parity gaps (ordered by M03 priority):
1. **Durable execution** (Temporal parity): workflow engine must survive process restarts; step state persisted to Postgres before execution; replay deterministic — this is the hardest engineering requirement and must be the first engine design decision.
2. **Node canvas UX** (n8n parity): drag-drop on Leptos canvas; edge routing; node configuration panel; zoom/pan; minimap — requires Leptos canvas component library decision.
3. **Integration breadth** (Zapier/Make parity): 50+ connectors at M03; 200+ at M04; community plugin SDK at M04.
4. **Approval workflows** (ServiceNow/Camunda parity): multi-step; four-eyes; SLA timers; escalation; delegation — required for HR/Payroll/Accounting use cases at M03.
5. **Agentic nodes** (no direct competitor; Oyatie-unique): LLM decision node via ADR-0107 agent gateway — differentiator, ship in M03 alpha.

---

## Performance Targets

Per `feedback_workflow_studio_scope.md` + `feedback_quality_performance_scalability_bar.md`:

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Studio first-paint (Leptos SSR web) | 150 ms | 500 ms | — | SSR pre-auth; SPA post-auth per ADR-0209 |
| Node-drop → edge-connect (canvas) | <1 ms | 16 ms | — | Local-first canvas; 60 fps |
| Workflow save → trigger-ready | 20 ms | 100 ms | — | Async compile + register |
| Workflow execution start → first step | 50 ms | 200 ms | 500 ms | Engine dispatch latency |
| Step execution (local action) | 5 ms | 50 ms | 100 ms | Ontology Action or internal call |
| Step execution (external HTTP) | — | 200 ms | 1 s | Network-bound; excluded from core SLO |
| Event-to-action latency (outbox) | 50 ms | 500 ms | 1 s | Outbox → Kafka KRaft → worker |
| Concurrent active runs per cell | — | 10,000 | — | Baseline; sharding to 100k+ aggregate |
| Long-running workflow durability | — | weeks | — | Temporal parity; no TTL on paused runs |
| Deterministic replay | 100% | 100% | — | Required for audit + debug |
| Audit chain seal per run | — | 1 s | — | Per (tenant_id, run_id); ADR-0028 |

Error budget: 0.05% monthly for engine path; 0.1% for Studio editor.
SLO burn-rate alarms: engine 3×; editor 5×.

---

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `postgres` for workflow
definitions + run state; `object-storage` for large step payloads (OCI Object
Storage); Kafka KRaft for event bus.

**Active-active compatibility**: `stateless-compatible` for Studio REST
layers and trigger adapter; `single-writer-compatible` for engine workers
processing a specific run (one worker owns one run; no concurrent writers per run).

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Concurrent active runs | 10,000 | 500,000 | Worker queue depth > 5k |
| Workflow definitions per tenant | 1,000 | 100,000 | DB shard fill > 80% |
| Events/sec inbound | 10,000 | 1,000,000 | Kafka consumer lag > 1 s |
| Studio concurrent users | 1,000 | 50,000 | CPU > 70% |

Scale-out policy:
- Engine workers: stateless HPA on queue depth >5k; min 3; max 200 pods.
- Studio REST: stateless HPA on CPU >70%; min 2; max 50 pods.
- Trigger adapter: stateless; Kafka consumer group auto-rebalance.
- Postgres + Citus: `tenant_id` shard key; linear shard addition.
- State store: Postgres append-only run log; ClickHouse replica for run history
  queries at scale.
- Pre-warmed worker pool: 10 standby pods; cold-start ≤500 ms per ADR-0020.

Cross-region:
- M03 launch: single KR region (OCI ap-seoul-1).
- Post-M03: active-active per-cell cross-region per ADR-0117 stages; workflow
  run state replicated via outbox + Kafka cross-region bridge.
- Long-running workflows: paused state written to Postgres; survives region
  failover if replica is current.

Sharding:
- Postgres + Citus on `tenant_id`; run log append-only; Citus distributed table.
- Kafka KRaft: per-tenant topic namespace; cell-local KRaft cluster; cross-cell
  bridge for multi-cell workflow fan-out.
- `check-shardability-cli` CI lane enforces `tenant_id` partition key
  presence on all workflow tables (M02 substrate phase).

---

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Workflow definition saved → trigger fires → first step executes in ≤200 ms | integration test `test_workflow_end_to_end_latency` |
| AC-02 | Deterministic replay: same event log produces same step sequence 100% | `cargo nextest run -p workflow-engine-domain --test deterministic_replay` |
| AC-03 | Durable execution: engine process killed mid-run; on restart, run resumes from last completed step | integration test `test_engine_durability_restart` |
| AC-04 | Multi-step approval: four-eyes constraint enforced; single approver cannot approve twice | `cargo nextest run -p workflow-approvals-domain --test four_eyes_invariant` |
| AC-05 | SLA timer fires escalation on breach; not before | `cargo nextest run -p workflow-sla-domain --test sla_escalation_timing` |
| AC-06 | Agentic node: LLM decision via agent gateway routes to correct branch | integration test `test_agentic_node_routing` |
| AC-07 | LEAN-A2: workflow crates have no product µservice imports (hr/payroll/connect etc.) | `oya gate validate lean-a2 --ms workflow` exits 0 |
| AC-09 | 10k concurrent active runs per cell; p99 step execution ≤200 ms | k6 load: `k6 run tests/load/workflow-engine-10k.js` |
| AC-10 | Tenant isolation: tenant A run cannot observe tenant B event payloads | `cargo nextest run -p workflow-engine-domain --test tenant_run_isolation` |
| AC-11 | Audit chain: every run sealed; tampering detected on verification | `oya gate validate audit-chain --ms workflow` |
| AC-12 | All 10 M03 domain templates loadable in Studio; no errors | integration test `test_template_library_load` |

---

## Open Questions

| # | Question | Owner | Target |
|---|---|---|---|
| 1 | Engine durable-execution model: bespoke Postgres-backed state machine vs embed Temporal SDK? | council-architecture | ADR-#### (highest priority) |
| 2 | Studio canvas tech: Leptos custom WebGL canvas vs SVG-based vs third-party (e.g. egui via WASM)? | council-architecture | M02/P01 |
| 3 | Workflow DSL format: YAML (n8n-style) vs JSON IR vs Rust proc-macro DSL? | council-architecture | M02/P01 |
| 4 | Plugin SDK distribution: WASM only, or also native dylib for internal nodes? | council-architecture | ADR-#### |
| 5 | Kafka KRaft cluster: one per cell (recommended) or shared across cells? | council-infrastructure | M02/P02 |

---

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| Bominal ADR-0035 | Workflow engine (hybrid state machine + DAG) | inherited — engine architecture |
| Bominal ADR-0103 | Workflow hexagonal migration | inherited — clean-arch placement |
| Bominal ADR-0121 | Workflow Studio Light scope | inherited — M03 launch presets (template library) |
| Bominal ADR-0148 | Workflow engine (extended) | inherited |
| Bominal ADR-0028 | Audit chain Merkle/Ed25519 | inherited |
| Bominal ADR-0107 | Ontology agent gateway | inherited — agentic nodes |
| Bominal ADR-0132 | Data ownership pillars | inherited — org/person enforcement on runs |
| Bominal ADR-0037 | Plugin substrate | inherited — custom node SDK |
| Bominal ADR-0009 | Cell architecture | inherited — per-cell run isolation |
| Bominal ADR-0019 | Runtime target metadata | inherited — active-active compatibility |
| oyatie override | Workflow is shared (not Corporate) | `feedback_workflow_is_shared.md` |
| oyatie override | Workflow + Ontology = ecosystem adapter | `feedback_workflow_objectgraph_adapter_layer.md` |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0106 | Ontology architecture | peer adapter (information plane) |
