---
doc_class: ReferenceImplementation
microservice: workflow-engine
language: Rust + Bash
date: 2026-05-20
doc_status: published
---

# Reference implementation — Author + run a saga workflow via the workflow-engine Rust SDK

A runnable example that:

1. Authenticates as a tenant `workflow_admin` principal.
2. Registers a 3-step saga with compensation.
3. Starts a workflow instance, watches progress.
4. Sends a signal mid-run.
5. Inspects the resulting event log + audit-chain emissions.

## Cargo.toml

```toml
[package]
name = "workflow-engine-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-workflow-engine-client = { path = "../../../../crates/oya-workflow-engine-client" }
oya-audit-chain-client = { path = "../../../../crates/oya-audit-chain-client" }
oya-cedar-client = { path = "../../../../crates/oya-cedar-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
futures = "0.3"
```

## src/main.rs

```rust
use anyhow::Result;
use futures::StreamExt;
use oya_workflow_engine_client::{
    WorkflowEngineClient, WorkflowEngineConfig,
    WorkflowDefinition, Step, Transition, Compensation,
    CompensationPolicy, RetryPolicy, BackoffStrategy,
    WorkflowStartRequest, SignalSendRequest,
    WorkflowStatus,
};
use oya_cedar_client::CedarPrincipal;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Construct the client bound to a workflow_admin Cedar principal.
    let principal = CedarPrincipal::from_env("WORKFLOW_ADMIN_JWT")?;
    let client = WorkflowEngineClient::connect(WorkflowEngineConfig {
        cell_endpoint: std::env::var("WORKFLOW_ENGINE_ENDPOINT")?,
        tenant_id: "acme-corp".into(),
        principal: principal.clone(),
        request_timeout: std::time::Duration::from_secs(30),
    }).await?;

    // 2. Register a 3-step saga workflow.
    let definition = WorkflowDefinition::builder()
        .workflow_id("simple-saga")
        .version(1)
        .description("3-step saga: reserve resource, charge customer, dispatch notification; with compensation on failure")
        .input_schema(json!({
            "type": "object",
            "required": ["customer_id", "resource_id", "amount_minor_units"],
            "properties": {
                "customer_id": {"type": "string"},
                "resource_id": {"type": "string"},
                "amount_minor_units": {"type": "integer"}
            }
        }))
        .step(Step::builder()
            .id("reserve_resource")
            .handler("ontology.resource.reserve")
            .input(json!({"resource_id": "{{resource_id}}"}))
            .timeout_seconds(30)
            .retry(RetryPolicy {
                max_attempts: 3,
                backoff: BackoffStrategy::Exponential,
                initial_delay_seconds: 2,
                max_delay_seconds: 60,
            })
            .compensation(Compensation::new("ontology.resource.release", json!({"reservation_id": "{{reserve_resource.reservation_id}}"})))
            .build()?)
        .step(Step::builder()
            .id("charge_customer")
            .handler("payments.charge_create")
            .input(json!({
                "customer_id": "{{customer_id}}",
                "amount_minor_units": "{{amount_minor_units}}",
                "currency": "USD"
            }))
            .timeout_seconds(30)
            .retry(RetryPolicy { max_attempts: 3, backoff: BackoffStrategy::Exponential, initial_delay_seconds: 2, max_delay_seconds: 60 })
            .compensation(Compensation::new("payments.refund_create", json!({"charge_id": "{{charge_customer.charge_id}}", "reason": "workflow_compensation"})))
            .build()?)
        .step(Step::builder()
            .id("dispatch_notification")
            .handler("mail.send_template")
            .input(json!({
                "template_id": "saga_completed",
                "recipient": "{{customer_id}}",
                "template_data": {"charge_id": "{{charge_customer.charge_id}}"}
            }))
            .timeout_seconds(60)
            .retry(RetryPolicy { max_attempts: 5, backoff: BackoffStrategy::ExponentialJitter, initial_delay_seconds: 5, max_delay_seconds: 300 })
            .build()?)
        .transition(Transition::on_success("reserve_resource", "charge_customer"))
        .transition(Transition::on_success("charge_customer", "dispatch_notification"))
        .compensation_policy(CompensationPolicy::any_step_failure_reverse())
        .build()?;

    client.workflow_register(&definition).await?;
    println!("Registered workflow: {} v{}", definition.workflow_id, definition.version);

    // 3. Start an instance.
    let start = WorkflowStartRequest {
        workflow_id: "simple-saga".into(),
        version: 1,
        inputs: json!({
            "customer_id": "cust-001",
            "resource_id": "resource-abc",
            "amount_minor_units": 12500
        }),
        idempotency_key: format!("simple-saga-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap()),
    };
    let instance = client.workflow_start(&start).await?;
    println!("Started instance: {}", instance.workflow_instance_id);
    println!("Trace ID: {}", instance.trace_id);

    // 4. Watch the workflow progress in real-time.
    let mut watch_stream = client
        .workflow_watch(&instance.workflow_instance_id)
        .await?;

    while let Some(event) = watch_stream.next().await {
        let event = event?;
        println!(
            "[{}] step={} status={:?} attrs={}",
            event.timestamp.to_rfc3339(),
            event.step_id.as_deref().unwrap_or("(workflow-level)"),
            event.event_type,
            event.attributes.unwrap_or(json!({}))
        );
        if matches!(event.event_type, oya_workflow_engine_client::EventType::WorkflowCompleted | oya_workflow_engine_client::EventType::WorkflowCompensated | oya_workflow_engine_client::EventType::WorkflowFailed) {
            break;
        }
    }

    // 5. Get the final state.
    let final_state = client.workflow_get(&instance.workflow_instance_id).await?;
    println!("Final status: {:?}", final_state.status);
    println!("Started at: {}", final_state.started_at.to_rfc3339());
    println!("Ended at: {}", final_state.ended_at.map(|t| t.to_rfc3339()).unwrap_or_else(|| "(running)".into()));
    println!("Steps completed: {}", final_state.steps_completed);

    if matches!(final_state.status, WorkflowStatus::Completed) {
        println!("Workflow outputs: {}", final_state.outputs.unwrap_or(json!({})));
    } else if matches!(final_state.status, WorkflowStatus::Compensated) {
        println!("Compensation events: {}", final_state.compensation_event_count);
    }

    Ok(())
}
```

## Expected output (forward path, all 3 steps succeed)

```
Registered workflow: simple-saga v1
Started instance: wf_01HZX9K3M2P4QR7S8T9V0W1X2Y
Trace ID: trace_01HZX9K3M2P4QR
[2026-05-20T14:32:01Z] step=(workflow-level) status=WorkflowStarted attrs={}
[2026-05-20T14:32:01Z] step=reserve_resource status=StepStarted attrs={}
[2026-05-20T14:32:02Z] step=reserve_resource status=StepCompleted attrs={"reservation_id":"res-abc-001"}
[2026-05-20T14:32:02Z] step=charge_customer status=StepStarted attrs={}
[2026-05-20T14:32:04Z] step=charge_customer status=StepCompleted attrs={"charge_id":"ch_acme_001"}
[2026-05-20T14:32:04Z] step=dispatch_notification status=StepStarted attrs={}
[2026-05-20T14:32:06Z] step=dispatch_notification status=StepCompleted attrs={"message_id":"msg_001"}
[2026-05-20T14:32:06Z] step=(workflow-level) status=WorkflowCompleted attrs={}
Final status: Completed
Started at: 2026-05-20T14:32:01Z
Ended at: 2026-05-20T14:32:06Z
Steps completed: 3
Workflow outputs: {"reservation_id":"res-abc-001","charge_id":"ch_acme_001","message_id":"msg_001"}
```

## Expected output (compensation path, charge_customer fails)

```
Registered workflow: simple-saga v1
Started instance: wf_01HZX9B...
[14:34:01] step=(workflow-level) status=WorkflowStarted
[14:34:02] step=reserve_resource status=StepCompleted reservation_id=res-abc-002
[14:34:02] step=charge_customer status=StepStarted
[14:34:04] step=charge_customer status=StepFailed reason=card_declined (attempt 1/3, retryable=false)
[14:34:04] step=charge_customer status=StepFailedExhausted reason=card_declined
[14:34:04] step=(workflow-level) status=CompensationStarted
[14:34:05] step=reserve_resource status=CompensationCompleted reason=released
[14:34:05] step=(workflow-level) status=WorkflowCompensated
Final status: Compensated
Steps completed: 1 (reserve_resource only)
Compensation events: 1
```

## Sending a signal mid-workflow

For workflows that wait at a `workflow.signal_wait()` step, send a signal via:

```rust
let signal = SignalSendRequest {
    workflow_instance_id: "wf_01HZX9...".into(),
    signal_name: "underwriter_signoff".into(),
    signal_payload: json!({"underwriter":"u-42","decision":"approved","conditions":"none"}),
    idempotency_key: "signal-001".into(),
};
client.workflow_signal_send(&signal).await?;
```

## HTTP alternative (curl)

```sh
# Register workflow
curl -X POST https://workflow-engine.prod-syd-1.oyatie.local/v1/workflows/register \
    -H "Authorization: Bearer $WORKFLOW_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d @./workflow-definition.json

# Start an instance
curl -X POST https://workflow-engine.prod-syd-1.oyatie.local/v1/workflows/start \
    -H "Authorization: Bearer $WORKFLOW_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "workflow_id":"simple-saga","version":1,
        "inputs":{"customer_id":"cust-001","resource_id":"resource-abc","amount_minor_units":12500},
        "idempotency_key":"simple-saga-001"
    }'

# Watch (SSE stream)
curl -N https://workflow-engine.prod-syd-1.oyatie.local/v1/workflows/wf_01HZX9.../events \
    -H "Authorization: Bearer $WORKFLOW_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Accept: text/event-stream"

# Signal send
curl -X POST https://workflow-engine.prod-syd-1.oyatie.local/v1/workflows/wf_01HZX9.../signals \
    -H "Authorization: Bearer $WORKFLOW_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "signal_name":"underwriter_signoff",
        "signal_payload":{"underwriter":"u-42","decision":"approved"}
    }'
```

## Error handling

| Error class | HTTP | Retry? | Action |
|---|---|---|---|
| `cedar_denied` | 403 | No | Principal lacks `workflow_engine::workflow::start` |
| `workflow_not_registered` | 404 | No | Register the workflow first |
| `idempotency_key_mismatch` | 422 | No | Client bug |
| `tenant_rate_limit` | 429 | Yes (auto, backoff) | Tenant hit per-second rate limit |
| `worker_pool_unavailable` | 503 | Yes (auto) | Tenant's worker pool is starting; retry |
| `signal_workflow_not_found` | 404 | No | Workflow instance has already completed or never existed |
| `workflow_quota_exceeded` | 429 | Yes (auto) | Tenant hit concurrent-workflow quota |

## Audit-chain events emitted

| Event | Audit class |
|---|---|
| Workflow start | `workflow.requested` |
| Step start | `workflow.step.started` |
| Step success | `workflow.step.completed` |
| Step failure | `workflow.step.failed` |
| Step exhaustion | `workflow.step.failed_exhausted` |
| Compensation start | `workflow.compensation.started` |
| Compensation step | `workflow.compensation.step.completed` |
| Workflow completion | `workflow.completed` |
| Workflow compensation | `workflow.compensated` |
| Signal received | `workflow.signal.received` |

## Where this file lives

`microservices/workflow-engine/reference-implementations/saga-workflow-rust-sdk.md` (this file). The runnable Cargo project lands at `microservices/workflow-engine/reference-implementations/saga-example/` once the SDK ships.
