---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M03-first-paying-tenant
phase: P07-workflow-studio-editor
status: Proposed
entry_gate: |
  M02-substrate-schema-foundation complete; oya-workflow-engine-kernel + oya-workflow-engine-domain ship (M02/P12);
  oya-workflow-approvals-kernel ships; oya-workflow-sla-domain ships;
  Leptos canvas component library decision resolved (M02/P01 open question #2).
exit_gate: |
  All IP acceptance gates green; deterministic replay test 100% pass rate;
  durable execution restart test green (engine killed mid-run; resumes from last step);
  four-eyes approval test green; SLA escalation timing test green;
  agentic node routing test green;
  all 10 M03 domain templates loadable in Studio; node-drop → edge-connect ≤16ms (Playwright);
  10k concurrent active runs p99 step execution ≤200ms (k6);
  `oya gate validate lean-a2 --ms workflow` exits 0;
  grit done on all P07 symbols; ICM phase-handoff row emitted.
depends_on:
  - milestone: M02
    phase: P12-workflow-engine
    reason: "Workflow Studio visual editor wraps the M02 engine; the engine domain + state-store + Kafka KRaft event bus must exist before Studio can render runs and wire triggers."
parallel_wave: 2  # Runs in parallel with P01-P05; only consumes M02 engine, not M03 product µservices
owner_team: council-architecture
---

# P07-workflow-studio-editor: Workflow Studio visual editor — Leptos canvas, 10 domain templates, agentic nodes, durable execution, approval chains

## Purpose

Delivers the Workflow Studio visual editor: the HERO product. An n8n-class
drag-drop canvas (Leptos WASM; node-drop @ 16ms p99 at 60fps) with durable
execution (Temporal parity — process-restart safe, deterministic replay), multi-step
approval chains with four-eyes constraint, SLA timers + escalation, agentic LLM
decision nodes (ADR-0107 agent gateway), and 10 domain templates shipping at M03
(Agentic / Developer / Business / Healthcare / Supply Chain / Delivery).

Workflow Studio is simultaneously the ecosystem's action/orchestration adapter
(LEAN-A2 enforcement plane) and the B2B hero product that tenants author, test,
deploy, and monitor their own automations — no engineering required.

Can execute in **parallel with P01–P06** because it depends only on the M02
Workflow engine substrate, not on any M03 product µservice.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Crate family (BNF v4.1) |
|---|---|---|
| `workflow` | `studio` | `oya-workflow-studio-{rest,graphql,sdk}` |
| `workflow` | `engine` | `oya-workflow-engine-{kernel,domain,application,adapter,worker}` (extends M02 foundation) |
| `workflow` | `transitions` | `oya-workflow-transitions-{kernel,domain}` |
| `workflow` | `approvals` | `oya-workflow-approvals-{kernel,domain,application}` |
| `workflow` | `sla` | `oya-workflow-sla-{domain,worker}` |
| `workflow` | `automations` | `oya-workflow-automations-{domain,worker}` |
| `workflow` | `triggers` | `oya-workflow-triggers-{adapter}` |
| `workflow` | `integrations` | `oya-workflow-integrations-{adapter}` |
| `workflow` | `app` | `oya-workflow-app` |

Naming justifications:

```
NAME: oya-workflow-studio-rest
JUSTIFICATION:
- microservice = workflow: Workflow µservice; registered; ADR-0056 v4.1; shared substrate (not Corporate per feedback_workflow_is_shared.md override)
- bc-tokens = studio: workflow has multiple BCs (studio/engine/transitions/approvals/sla/automations/triggers/integrations); studio BC owns WorkflowDefinition + WorkflowVersion + template library; ADR-0056 v4.1 BC-optionality
- layer = rest: Axum HTTP handlers for Studio authoring API; maps HTTP → application commands; no business logic; ADR-0056 §"Layer semantics"
- exemptions: none (this exact name appears as example in ADR-0056)

NAME: oya-workflow-studio-graphql
JUSTIFICATION:
- microservice = workflow; bc-tokens = studio; layer = graphql: async-graphql schema + resolvers for Studio live run history, version diff, template library queries; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-workflow-studio-sdk
JUSTIFICATION:
- microservice = workflow; bc-tokens = studio; layer = sdk: Leptos WASM client library for visual canvas component; depends on workflow-engine-kernel types only; ADR-0056 §"Layer semantics" + §"sdk-kernel-only"
- exemptions: none

NAME: oya-workflow-engine-kernel
JUSTIFICATION:
- microservice = workflow; bc-tokens = engine: engine BC owns WorkflowRun + StepExecution entities + WorkflowStateStore/TransitionEngine/EventBus port-traits; ADR-0056 v4.1 BC-optionality
- layer = kernel: pure RunId value types + port declarations; zero logic; ADR-0056 §"Layer semantics"
- exemptions: none (this exact name appears as example in ADR-0056)

NAME: oya-workflow-approvals-kernel
JUSTIFICATION:
- microservice = workflow; bc-tokens = approvals: approvals BC owns ApprovalRequest + ApprovalDecision entities + ApprovalStore port-trait; four-eyes constraint type; ADR-0056 v4.1 BC-optionality
- layer = kernel: pure ApprovalId value types + ApprovalStore port declaration; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-workflow-transitions-kernel
JUSTIFICATION:
- microservice = workflow; bc-tokens = transitions: transitions BC owns Transition + TransitionRule entities + SlaTimerStore port-trait; ADR-0056 v4.1 BC-optionality
- layer = kernel: pure TransitionId/TimerId value types + port declarations; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-workflow-automations-worker
JUSTIFICATION:
- microservice = workflow; bc-tokens = automations: automations BC owns AutomationRun entity + LLM decision node via ADR-0107 agent gateway; ADR-0056 v4.1 BC-optionality
- layer = worker: long-running background worker; consumes Kafka KRaft event topic; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-workflow-triggers-adapter
JUSTIFICATION:
- microservice = workflow; bc-tokens = triggers: triggers BC owns TriggerConfig + cron/webhook/event/ontology/manual/api trigger sources; adapter implements EventBus subscription port; ADR-0056 v4.1 BC-optionality
- layer = adapter: implements WorkflowStateStore/EventBus kernel port-traits; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-workflow-integrations-adapter
JUSTIFICATION:
- microservice = workflow; bc-tokens = integrations: integrations BC owns ConnectorConfig + ConnectorExecution; HTTP action executor; internal µservice node adapters; KR carrier connectors; plugin SDK (WASM, ADR-0037); ADR-0056 v4.1 BC-optionality
- layer = adapter: implements ConnectorPort kernel port-traits; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-workflow-app
JUSTIFICATION:
- microservice = workflow; bc-tokens: OMITTED — composition-root; ADR-0056 §"BC optionality"
- layer = app: main.rs + DI wiring for all workflow BCs; ADR-0056 §"Layer semantics"
- exemptions: none (this exact name appears as example in ADR-0056)
```

### M03 domain template library (10 templates — all must load at exit gate)

| Domain | Template name | Trigger |
|---|---|---|
| Agentic | Agent Task Orchestrator | Manual / API |
| Developer | PR Review Pipeline | GitHub webhook |
| Developer | Scaffold-Claim Release | API |
| Business | Leave Request Approval | Manual |
| Business | Expense Approval | Ontology-event |
| Business | Payroll Close | Cron (month-end) |
| Healthcare | Clinical Discharge | Ontology-event |
| Healthcare | Prescription Lifecycle | Ontology-event |
| Supply Chain | PO → Receipt → Reorder | Ontology-event |
| Delivery | Shipment Tracking | Webhook (carrier) |

### Out-of-scope

- Plugin SDK distribution to third-party developers (community marketplace) — post-M03.
- Workflow DSL YAML bidirectional sync (FR-03) — post-M03 per PRD open question #3.
- Windows/macOS/Linux/iOS/Android native Workflow Studio clients — post-M03 (Leptos web only at M03 per ADR-0210 client bar; native clients are M03 launch requirement but carried by P06-application).

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Workflow Studio: Leptos WASM canvas (16ms node-drop), durable execution engine (Postgres-backed deterministic replay), 10 domain templates, approval chains (four-eyes), SLA timers + escalation, agentic LLM node (ADR-0107), trigger adapters (cron/webhook/event/ontology), integration connectors (50+), load tests | pending | council-architecture |

---

## Acceptance Gates

### Cargo / CI gates

```bash
cargo check --workspace --all-features                                                          # exit 0
cargo build -p oya-workflow-app --all-features                                                  # exit 0
cargo clippy -p oya-workflow-engine-domain -p oya-workflow-approvals-domain -p oya-workflow-sla-domain -- -D warnings  # exit 0
cargo nextest run -p oya-workflow-engine-domain --test deterministic_replay                     # exit 0; 100% pass rate
cargo nextest run --test test_engine_durability_restart                                         # exit 0; resumes from last completed step
cargo nextest run -p oya-workflow-approvals-domain --test four_eyes_invariant                   # exit 0; single approver cannot approve twice
cargo nextest run -p oya-workflow-sla-domain --test sla_escalation_timing                       # exit 0; escalation fires on breach, not before
cargo nextest run --test test_agentic_node_routing                                              # exit 0; LLM decision routes correctly
cargo nextest run -p oya-workflow-engine-domain --test tenant_run_isolation                     # exit 0; tenant A cannot observe tenant B payloads
cargo nextest run --test test_template_library_load                                             # exit 0; all 10 M03 templates loadable
cargo deny check                                                                                # exit 0
```

### E2E / Playwright gates

```bash
# Studio canvas: node-drop → edge-connect ≤16ms
rtk playwright test tests/e2e/studio-canvas-perf.spec.ts
# Pass: node operation p99 <16ms; measured via Playwright performance.mark()
```

### Fitness lane gates

```bash
oya gate validate lean-a2 --ms workflow        # workflow crates have no product µservice imports
oya gate validate lean-a1 --ms workflow        # layer ordering
oya gate validate port-location --ms workflow  # port traits in kernel
oya gate validate shardability --ms workflow   # tenant_id shard key on all workflow tables
oya gate validate audit-chain --ms workflow    # every run sealed; tampering detected
```

### Load test gates

```bash
# 10k concurrent active runs; p99 step execution ≤200ms
k6 run tests/load/workflow-engine-10k.js
# Pass: http_req_duration{p(99)}<200; concurrent_runs=10000; error_rate<0.001

# Studio first-paint ≤500ms
k6 run tests/load/smoke-workflow-studio-firstpaint.js
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate | Layer | Port traits in kernel? | Impls in adapter? |
|---|---|---|---|
| `oya-workflow-engine-kernel` | `kernel` | Yes — `WorkflowStateStore`, `TransitionEngine`, `EventBus` | N/A |
| `oya-workflow-engine-domain` | `domain` | N/A | N/A |
| `oya-workflow-engine-application` | `application` | N/A | N/A |
| `oya-workflow-engine-adapter` | `adapter` | N/A | Yes — `PostgresWorkflowStateStore`, `KafkaKRaftEventBus` |
| `oya-workflow-engine-worker` | `worker` | N/A | No direct adapter import |
| `oya-workflow-approvals-kernel` | `kernel` | Yes — `ApprovalStore` | N/A |
| `oya-workflow-approvals-domain` | `domain` | N/A | N/A |
| `oya-workflow-transitions-kernel` | `kernel` | Yes — `SlaTimerStore` | N/A |
| `oya-workflow-automations-worker` | `worker` | N/A | No direct adapter import |
| `oya-workflow-triggers-adapter` | `adapter` | N/A | Yes — trigger source implementations |
| `oya-workflow-integrations-adapter` | `adapter` | N/A | Yes — connector implementations |
| `oya-workflow-studio-rest` | `rest` | N/A | No direct adapter import |
| `oya-workflow-studio-graphql` | `graphql` | N/A | No direct adapter import |
| `oya-workflow-studio-sdk` | `sdk` | N/A | kernel types only |
| `oya-workflow-app` | `app` | N/A | Unrestricted inward |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `studio` | `workflow` | pending |
| `engine` | `workflow` | pending |
| `transitions` | `workflow` | pending |
| `approvals` | `workflow` | pending |
| `sla` | `workflow` | pending |
| `automations` | `workflow` | pending |
| `triggers` | `workflow` | pending |
| `integrations` | `workflow` | pending |

---

## Grit Claim Symbols

```
crates/oya-workflow-engine-kernel/src/ports.rs::WorkflowStateStore
crates/oya-workflow-engine-kernel/src/ports.rs::TransitionEngine
crates/oya-workflow-engine-kernel/src/ports.rs::EventBus
crates/oya-workflow-approvals-kernel/src/ports.rs::ApprovalStore
crates/oya-workflow-transitions-kernel/src/ports.rs::SlaTimerStore
crates/oya-workflow-engine-domain/src/workflow_run.rs::WorkflowRun
crates/oya-workflow-approvals-domain/src/four_eyes.rs::FourEyesConstraint
contracts/workflow.openapi.yaml::createWorkflow
contracts/workflow.openapi.yaml::triggerRun
docs/standards/bounded-contexts.md::workflow.engine
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P07-workflow-studio-editor started; parallel with P01-P05 (M02 engine only); scope: Leptos WASM canvas, durable execution (Temporal parity), 10 domain templates, approvals/SLA/agentic nodes" \
  -i high \
  -k "M03,P07,phase-start,workflow,studio"

icm store \
  -t context-oyatie \
  -c "Phase P07-workflow-studio-editor complete; Workflow Studio HERO product live; Leptos 16ms canvas; durable execution; 10 domain templates; four-eyes approvals; SLA timers; agentic LLM nodes; next: P08-kr-acceptance-evidence" \
  -i high \
  -k "M03,P07,phase-complete,workflow,studio"
```

---

## References

- PRD: `docs/prds/workflow.md`
- Bominal ADRs inherited: ADR-0035 (hybrid SM+DAG engine), ADR-0103 (hexagonal), ADR-0121 (Studio Light M3 presets), ADR-0148 (extended engine), ADR-0028 (audit chain), ADR-0107 (agent gateway), ADR-0132 (pillars), ADR-0037 (plugin SDK), ADR-0009 (cell architecture)
- oyatie overrides: Workflow is shared (not Corporate); Workflow + Ontology = adapter layer (feedback_workflow_objectgraph_adapter_layer.md)
- oyatie ADRs: ADR-0056 (BNF v4.1)
