---
doc_class: FAQ
microservice: workflow-engine
persona: workflow-engineer + saga-architect + workflow-platform-engineer
date: 2026-05-20
doc_status: published
---

# Workflow Engineer FAQ — workflow-engine

## Why a custom workflow engine instead of Temporal / Camunda / Step Functions?

Per ADR-0145 + ADR-XXX-workflow-engine-rationale. Three drivers:

1. **Cedar + audit-chain integration**: every workflow step emits to audit-chain with Cedar evaluation context. Temporal / Camunda / Step Functions all have their own audit logs; integrating them with our cryptographic audit-chain would be a parallel system. Native integration is cheaper + more secure.
2. **Sovereign-pack residency**: Temporal Cloud is EU + US regions; Step Functions is AWS-bound. We need KR + CN + EU + US-GovCloud + on-prem (sovereign packs). Self-host Temporal would lose the managed-service value-add.
3. **Per-tenant isolation at scale**: Temporal namespaces work but the leader-election cost scales with namespace count; at 100k+ tenants the overhead is significant. Our per-tenant Kubernetes Deployment model amortises better.

The trade-off: we maintain a workflow engine instead of consuming one. The execution model is intentionally Temporal-like (durable functions, event-log replay, signals) so engineers familiar with Temporal can be productive quickly.

## What's the durable-function model? How does it differ from a DAG?

Per ADR-XXX-workflow-engine-execution-model.

- **DAG (Airflow, Argo)**: defines a static graph of nodes + edges. Each node executes in isolation; state is passed via inputs/outputs. No durability across worker restarts; if a worker dies mid-DAG, the entire DAG re-runs.
- **Durable function (Temporal, Step Functions, oyatie)**: defines workflow code (or a typed step graph) that executes against a per-workflow event log. Every effect (function call, signal, timer) is recorded; on worker restart, the engine replays the event log to reconstruct workflow state — only un-replayed effects re-execute.

Durable functions are strictly more expressive than DAGs (a DAG is a durable-function shape; a durable function can express loops, signals, dynamic branching).

oyatie's model is closer to Temporal than Airflow.

## What's the compensation policy and when does it fire?

Per ADR-XXX-workflow-engine-compensation. The saga pattern: forward steps execute; on a step failure, compensation steps fire in reverse order to undo the side effects.

Compensation fires when:

- A step exhausts its retry budget (e.g., 3 attempts all fail).
- A timeout is exceeded.
- An explicit `workflow.fail()` is called from user code.
- An external signal of type `cancel` is received.

Compensation does NOT fire when:

- A workflow completes successfully (forward path complete).
- A workflow is cancelled by `oya workflow-engine workflow cancel --no-compensate` (explicit opt-out).
- A step explicitly opts out via `compensation: null`.

If a compensation step itself fails, the compensation-failure-escalation policy fires:

- `escalate_to_on_call` (default): page the on-call + freeze the workflow in `compensation-stuck` state.
- `retry_compensation`: retry the compensation N times.
- `swallow_and_continue`: skip the failed compensation and proceed (use with extreme caution; can leave inconsistent state).

## How long can a workflow run?

Per tenant class:

- **demo_trial**: ≤ 1 h (effectively short-lived only; in-memory state).
- **paid**: ≤ 365 d (for HIPAA care-coordination, KR financial-monitoring, etc.).

Beyond the tenant-class cap, workflows must explicitly self-restart (chain a new workflow as a continuation; oyatie's `workflow.continue_as_new()` API).

The duration cap exists because the event log grows linearly with workflow lifetime + event count. At 365 d × 1k events/d × 4 KiB/event = ~ 1.4 GiB per workflow. PostgreSQL can store this but query latency degrades; we cap to keep query latency bounded.

## What's the difference between a signal and a callback?

- **Signal**: an external event sent into a running workflow. The workflow blocks at `workflow.wait_for_signal("name")` until the signal arrives. Signals are typed + audited.
- **Callback**: a function the workflow calls into the user code. Synchronous within the step; not a separate event.

Signals are how external systems (humans, other services, scheduled events) interact with a running workflow. A loan-approval workflow waits for an `underwriter_signoff` signal; a customer-cancellation workflow waits for a `customer_confirmation` signal.

## How are workflow versions managed? Can I change a workflow definition while instances are running?

Per ADR-XXX-workflow-engine-versioning. Yes, workflow definitions are versioned. Each running workflow instance pins its definition version at start time. When you register a new version (`oya workflow-engine workflow register --version N+1`):

- New starts use version N+1.
- In-flight instances continue running version N until completion (the engine loads the pinned version from the event log).

This is critical: changing a workflow definition mid-run would break replay determinism.

For breaking changes (e.g., removing a step), the new version + the old version coexist until all old-version instances drain.

## What's the retry policy + backoff model?

Per step:

```yaml
retry:
  max_attempts: 3
  backoff: exponential  # or linear, exponential_jitter, fixed
  initial_delay_seconds: 2
  max_delay_seconds: 120
  retryable_errors: [transient, rate_limited, timeout]
  non_retryable_errors: [unauthorized, invalid_input, not_found]
```

`backoff` algorithms:

- `linear`: delay = initial_delay × attempt_number.
- `exponential`: delay = initial_delay × (2 ^ (attempt_number - 1)).
- `exponential_jitter`: same as exponential + ± 20 % random jitter (prevents retry storms).
- `fixed`: same delay every time.

The retry budget is per-step. Workflows can also have a workflow-level retry budget (`workflow.retry_max_attempts`).

`retryable_errors` is a whitelist; only errors in this list are retried. Errors in `non_retryable_errors` fail the step immediately without retry.

## How is cross-µservice tracing handled?

OpenTelemetry trace-id propagates from workflow start through every step. When a step calls another oyatie µservice (e.g., `payments.charge_create`), the trace-id is passed via the `traceparent` header (W3C Trace Context).

The full trace can be viewed in Tempo / Jaeger:

```sh
oya observability trace get --trace-id <id>
# Returns the full span tree across workflow-engine + payments + intelligence + audit-chain
```

The trace shows: workflow start → step starts → µservice calls (with their own internal spans) → step completions → workflow completion. Useful for debugging "where did this workflow get stuck."

## What's the per-tenant rate limit + how do I increase it?

Per-tenant rate limit caps workflow starts per second + workflow steps per second. Defaults per tenant class:

- demo_trial: 10 starts/sec, 100 steps/sec.
- paid: contract and pack specific.

Tenants exceeding the limit get rate-limited (HTTP 429); the SDK retries with exponential backoff.

To increase: `oya workflow-engine tenant-quota update --tenant T --starts-per-sec N`. Requires `governance::quota::increase` Cedar permission.

## What's the difference between this µservice and `workflow-studio`?

- `workflow-engine`: the EXECUTION substrate. Runs the workflows. Headless API.
- `workflow-studio`: the VISUAL AUTHORING surface. n8n-class visual editor. Outputs workflow definitions that the engine executes.

Tenants can use the engine without the studio (author workflows in YAML / Rust SDK directly). The studio is value-add for non-technical authors.

## How do I respond when a tenant says "Temporal is more battle-tested"?

Acknowledged — Temporal has > 5 years of production maturity at hyperscalers. Our response:

1. We modelled the durable-function execution after Temporal; the abstraction shapes are equivalent.
2. We have the Cedar + audit-chain + sovereign-pack integration that Temporal Cloud doesn't offer.
3. For tenants who prefer Temporal, we offer Temporal Cloud BYO integration (Temporal Cloud workflows can call oyatie µservices; events emit to oyatie audit-chain via the emission adapter).

Most tenants choose oyatie's engine for the integration value-add. A small minority (typically tenants with deep existing Temporal investment) keep Temporal + use oyatie µservices.

## What happens if a tenant's worker pool crashes mid-workflow?

The workflow event log is in PostgreSQL with synchronous replication for paid tenant-class workloads. On worker pool crash:

1. The worker's lease expires within 30 s (Valkey TTL).
2. Another worker pool pod in the same tenant picks up the lease (via Kubernetes HPA or a healthy peer).
3. The new pod replays the workflow event log to reconstruct state.
4. The workflow resumes from the last completed event.

No work is lost; at-least-once semantics with replay determinism guarantee correctness.

If the entire tenant worker pool is unhealthy (rare), the engine's leader-election picks up the workflows + executes them in a fallback pool. The tenant is alerted to investigate their worker pool health.
