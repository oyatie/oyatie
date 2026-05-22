---
doc_class: MigrationPlaybook
microservice: workflow-engine
vendor: Temporal Cloud (managed Temporal SaaS)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Temporal Cloud → oyatie workflow-engine

Audience: a team using Temporal Cloud (managed Temporal) for durable-function execution who wants to move to oyatie's `workflow-engine`. Drivers: integration with oyatie's audit-chain + Cedar + sovereign-pack; consolidation of workflow execution with the rest of oyatie's substrate; per-tenant SaaS-shape multi-tenancy.

## Why this migration matters

Temporal Cloud is excellent at:

- Durable function semantics, replay determinism, worker autoscaling.
- Multi-language SDKs (Go, Java, TS, Python, .NET, Ruby, PHP).
- High availability + regional residency.

oyatie `workflow-engine` adds:

- Native audit-chain integration (every step → cryptographically signed event).
- Sovereign-pack residency (KR + EU + CN + US-GovCloud + on-prem) without managed-service compromise.
- Tighter integration with `payments`, `intelligence`, `ontology`, `cloud-iam` (every cross-µservice call traced + audited).
- Cost amortisation: self-hosted at the oyatie envelope is significantly cheaper than Temporal Cloud per-action pricing.

The trade-off: less mature multi-language SDK ecosystem (Rust native + TS + Python + Go vs Temporal's 7 languages). Tenants with Java or .NET workflows need to either rewrite or use the oyatie REST API.

## Step 1 — Inventory Temporal Cloud usage (≤ 1 week)

```bash
# From Temporal Cloud admin (tcld):
tcld namespace list
tcld namespace get --namespace <ns>
tcld workflow list --namespace <ns> --query "ExecutionStatus='Running'"
```

Document:

- Namespaces (per-team or per-tenant).
- Active workflows + workflow types (e.g., "OrderFulfillment", "DataProcessing").
- Worker pools per namespace.
- Workflow query handlers + signal handlers.
- Schedule (cron) workflows.
- Activity types (Temporal "Activities" = oyatie steps).
- Daily action counts (Temporal's billing unit).

Typical mid-size Temporal Cloud install: 3-10 namespaces, 20-100 workflow types, 50-200 activity types, 1-10M actions/day.

## Step 2 — Map Temporal concepts to oyatie (≤ 1 week)

| Temporal concept | oyatie equivalent |
|---|---|
| Namespace | Tenant (oyatie tenants are equivalent to Temporal namespaces) |
| Workflow type | Workflow definition (registered via `oya workflow-engine workflow register`) |
| Workflow execution | Workflow instance (`workflow_instance_id`) |
| Activity | Step (defined in workflow YAML) |
| Worker | Per-tenant Kubernetes Deployment |
| Signal | Signal (1:1 concept) |
| Query | Query (workflow state introspection) |
| Schedule | Schedule (cron / interval) |
| Saga / compensation | Compensation policy (oyatie-native) |
| Retry policy | Retry policy (per-step) |
| Continue-as-new | `workflow.continue_as_new()` (oyatie-native) |

## Step 3 — Translate workflow definitions (≤ 2-4 weeks per 10 workflow types)

Temporal workflows are code (Go / TS / Python). oyatie workflows are typed declarative YAML + handlers registered in the worker pool.

Temporal Go example:

```go
func OrderFulfillmentWorkflow(ctx workflow.Context, input OrderInput) error {
    err := workflow.ExecuteActivity(ctx, ValidateInventory, input.LineItems).Get(ctx, &reservation)
    if err != nil { return err }
    err = workflow.ExecuteActivity(ctx, ChargeCustomer, input.PaymentMethod, input.Amount).Get(ctx, &charge)
    if err != nil {
        workflow.ExecuteActivity(ctx, ReleaseInventory, reservation)
        return err
    }
    // ... saga continues
}
```

oyatie YAML equivalent:

```yaml
workflow_id: order-fulfillment
steps:
  - id: validate_inventory
    handler: ontology.inventory.reserve
    inputs: { ... }
    compensation: { handler: ontology.inventory.release, ... }
  - id: charge_customer
    handler: payments.charge_create
    compensation: { handler: payments.refund_create, ... }
  - id: ...
transitions: [...]
compensation_policy: { trigger: any_step_failure, order: reverse_chronological }
```

The translation tooling:

```sh
oya workflow-engine migrate temporal-to-oyatie \
    --source-dir ./temporal-workflows/ \
    --output-dir ./oyatie-workflows/ \
    --language-source go
```

The tool auto-translates ~ 70-80 % of Temporal workflow constructs. Remaining 20-30 % requires manual review (complex Go state, custom Activity options, etc.).

## Step 4 — Migrate workers (≤ 4-6 weeks per worker pool)

Temporal worker pools dispatch activities; oyatie worker pools dispatch handlers. The shape is similar but the registration model differs:

```diff
- // Temporal Go worker
- worker := worker.New(temporalClient, "task-queue", worker.Options{})
- worker.RegisterWorkflow(OrderFulfillmentWorkflow)
- worker.RegisterActivity(ValidateInventory)
- worker.RegisterActivity(ChargeCustomer)
- worker.Run(worker.InterruptCh())

+ // oyatie Rust worker
+ let pool = WorkerPool::new(WorkerPoolConfig {
+     tenant_id: "acme-corp".into(),
+     handlers_namespace: "tenant.order-fulfillment".into(),
+     ..Default::default()
+ });
+ pool.register_handler("ontology.inventory.reserve", validate_inventory_handler);
+ pool.register_handler("payments.charge_create", charge_customer_handler);
+ pool.register_handler("workflow.tenant.shipping_reserve", reserve_shipping_handler);
+ pool.run().await?;
```

For tenants who insist on Go/Java/Python workers: the oyatie REST + gRPC worker SDK supports Go + Java + Python + TS (Rust is canonical; bindings auto-generated).

## Step 5 — Dual-execution shadow (≤ 4-8 weeks)

Run BOTH Temporal Cloud and oyatie in parallel for each new workflow start:

```javascript
const [temporalResult, oyaResult] = await Promise.all([
  temporalClient.workflow.execute(orderFulfillmentWorkflow, { args: [input] }),
  oyaWorkflowEngine.startAndWait("order-fulfillment", "v1", input)
]);

// Return temporalResult to the user; log oyaResult for comparison.
await shadowReconciliation.compare({
  workflow_type: "order-fulfillment",
  inputs_hash: hash(input),
  temporal_outputs: temporalResult.outputs,
  oya_outputs: oyaResult.outputs,
  temporal_duration_s: ...,
  oya_duration_s: ...,
});
```

Acceptance criteria for cutover:

- Output-divergence < 0.5 % across all workflow types (some divergence is expected; e.g., timestamps, random IDs).
- Latency parity within 2× across all percentiles.
- Compensation correctness verified on 100 fault-injected runs.

## Step 6 — Cutover + monitor (≤ 1 d)

```sh
oya governance set-config \
    --tenant acme-corp \
    --key default_workflow_engine \
    --value oyatie

oya audit emit \
    --tenant acme-corp \
    --event-class governance.workflow_substrate.cut_over \
    --payload '{"from":"temporal-cloud","to":"oyatie","cutover_at":"2026-05-20T14:00:00Z"}'
```

Existing Temporal workflows continue running until they complete (Temporal Cloud namespace remains active in read-mostly mode).

## Step 7 — Temporal Cloud decommission (≤ 90-180 d post-cutover)

Keep Temporal Cloud active until:

- All in-flight Temporal workflows have completed.
- The longest-running Temporal schedule has fired ≥ once in oyatie.
- All Temporal Cloud query consumers (dashboards, alerting) have migrated to oyatie equivalents.

Then cancel the Temporal Cloud subscription per their contract notice period.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Workflow definition translation gap | High | Plan 2-4 weeks per 10 workflow types; manual review for complex Temporal idioms |
| Worker pool migration breaks downstream consumers | High | Dual-write to both Temporal + oyatie for ≥ 4 weeks |
| Long-running Temporal workflows can't be migrated mid-run | High | Wait for completion; don't migrate in-flight |
| Per-action pricing makes Temporal Cloud cheaper at low volume | Medium | Validate the TCO before cutover; if tenant is at < 5M actions/year, Temporal Cloud may stay cheaper |
| Multi-language SDK gap (Java / .NET) | Medium | Use REST/gRPC worker SDK; or migrate to TS/Python/Go |
| Continue-as-new chains differ in semantics | Medium | Carefully test `workflow.continue_as_new()` on critical workflows |
| Temporal-specific advanced patterns (heartbeats with metadata, child workflows) | Medium | Use oyatie equivalents; document edge cases |
| Worker autoscaling differs (Temporal's slot-based vs oyatie's HPA) | Low | Tune oyatie HPA for tenant's traffic shape |
| Schedule workflow timezone semantics differ | Low | Standard cron + IANA timezones; pre-validate |
| Cross-µservice tracing differs (Temporal Workflow History vs oyatie audit-chain) | Low | Both visible in their respective observability stacks; tenant chooses primary view |
