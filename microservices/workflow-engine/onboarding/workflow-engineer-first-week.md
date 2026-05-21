---
doc_class: Onboarding
microservice: workflow-engine
persona: workflow-engineer + saga-architect + workflow-platform-engineer
related_adrs: [ADR-0263, ADR-0131, ADR-0329, ADR-0330, ADR-0331, ADR-0145]
date: 2026-05-20
doc_status: published
---

# Workflow Engineer onboarding — first 5 working days on `workflow-engine`

Audience: a new workflow engineer or saga architect joining the `workflow-engine` rotation. By Day-5 they will have: bootstrapped a demo_trial tenant-class cell, authored a 5-step workflow definition, executed a saga with compensation, sent external signals into running workflows, run a cross-AZ failover drill, and walked the workflow-stuck-at-retries runbook.

## Day 1 — Tour the substrate

1. Read `PRD.md` (∼ 45 min). Note the durable-function execution model + the per-tenant worker-pool isolation.
2. Read `ARCHITECTURE.md` § event-log + § compensation + § signals + § scheduler (∼ 60 min).
3. Open the Grafana folder `workflow-engine`. Canonical boards: `workflow-start-latency`, `workflow-step-success-rate`, `workflow-compensation-fire-rate`, `workflow-queue-depth`, `workflow-event-log-write-rate`, `workflow-retry-storm`, `workflow-cross-az-migration`.
4. Walk `runbooks/README.md`. The on-call runbooks: `workflow-stuck-at-retries.md`, `compensation-failed.md`, `worker-pool-oom.md`, `event-log-write-stall.md`, `scheduler-clock-skew.md`, `cross-az-migration-failed.md`, `signal-not-delivered.md`, `tenant-rate-limit-exceeded.md`, `pulsar-backpressure.md`.
5. Sit in on the Wednesday workflow-substrate handoff. Watch the outgoing rotation walk the past-week compensation-fire-rate + retry-storm board.

Acceptance: you can sketch the workflow-start path: tenant API → Cedar gate → workflow-definition lookup → event-log "WorkflowStarted" → per-tenant worker pool dispatch → step execution → audit-chain emit → completion event.

## Day 2 — demo_trial workflow-engine cell bootstrap

```sh
cargo run -p oya-dev-cli -- workflow-engine bootstrap \
    --tenant-class demo_trial \
    --cell drill-syd-1 \
    --postgres-endpoint postgres://drill-pg-syd-1:5432/workflow_engine \
    --valkey-endpoint valkey://drill-valkey-syd-1:6379 \
    --pulsar-endpoint pulsar://drill-pulsar-syd-1:6650 \
    --audit-chain-endpoint http://drill-audit-syd-1:8080 \
    --kubeconfig ./drill-syd-1.kubeconfig
```

Expected runtime: ≤ 10 min. Verify after bootstrap:

```sh
oya workflow-engine health --cell drill-syd-1
# Expected:
#   engine-api: up (3 nodes)
#   postgres.event-log: up (lag_ms=12)
#   valkey.lease: up (3 nodes)
#   pulsar.workflow-events: connected
#   audit-chain.emit: up
#   scheduler: up (cron-cycle=1s)
```

Acceptance: cell live; you can describe the role of each component + the per-tenant worker pool model.

## Day 3 — Author + execute a 5-step workflow

Define a customer-onboarding saga:

```yaml
# customer-onboarding-v1.yaml
workflow_id: customer-onboarding
version: 1
description: "Onboard a new customer: create record, send welcome email, charge first invoice, provision tenant resources, audit completion"

inputs:
  customer_email:    {type: string, required: true, format: email}
  customer_company:  {type: string, required: true}
  tenant_class:      {type: enum, values: [demo_trial, paid], default: paid}
  initial_charge_amount_minor_units: {type: integer, required: true}

steps:
  - id: create_customer_record
    handler: cloud_iam.principal_create
    inputs:
      email: "{{customer_email}}"
      organization: "{{customer_company}}"
    outputs:
      principal_id: $.principal_id
    timeout_seconds: 30
    retry:
      max_attempts: 3
      backoff: exponential

  - id: send_welcome_email
    handler: mail.send_template
    inputs:
      template_id: welcome
      recipient: "{{customer_email}}"
      principal_id: "{{create_customer_record.principal_id}}"
    timeout_seconds: 60
    retry:
      max_attempts: 5
      backoff: exponential_jitter

  - id: charge_first_invoice
    handler: payments.charge_create
    inputs:
      customer_id: "{{create_customer_record.principal_id}}"
      amount_minor_units: "{{initial_charge_amount_minor_units}}"
      currency: USD
      description: "First invoice for {{tenant_class}} tenant class"
    outputs:
      charge_id: $.charge_id
    timeout_seconds: 30
    retry:
      max_attempts: 3
      backoff: exponential
    compensation:
      id: refund_first_invoice
      handler: payments.refund_create
      inputs:
        charge_id: "{{charge_first_invoice.charge_id}}"
        reason: workflow_compensation

  - id: provision_tenant_resources
    handler: tenancy.tenant_resources_create
    inputs:
      tenant_id: "{{create_customer_record.principal_id}}"
      tenant_class: "{{tenant_class}}"
    timeout_seconds: 180
    retry:
      max_attempts: 2
      backoff: linear
    compensation:
      id: deprovision_tenant_resources
      handler: tenancy.tenant_resources_delete
      inputs:
        tenant_id: "{{create_customer_record.principal_id}}"

  - id: emit_completion_audit
    handler: audit_chain.emit
    inputs:
      event_class: customer.onboarded
      tenant_id: "{{create_customer_record.principal_id}}"
      payload:
        tenant_class: "{{tenant_class}}"
        first_charge_id: "{{charge_first_invoice.charge_id}}"

transitions:
  - from: create_customer_record
    to: send_welcome_email
    on: success
  - from: send_welcome_email
    to: charge_first_invoice
    on: success
  - from: charge_first_invoice
    to: provision_tenant_resources
    on: success
  - from: provision_tenant_resources
    to: emit_completion_audit
    on: success

compensation_policy:
  trigger: any_step_failure
  order: reverse_chronological
  on_compensation_failure: escalate_to_on_call
```

Register the workflow definition:

```sh
oya workflow-engine workflow register \
    --tenant drill-acme \
    --definition ./customer-onboarding-v1.yaml
# Output: workflow_id=customer-onboarding, version=1, registered_at=...
```

Execute one:

```sh
oya workflow-engine workflow start \
    --tenant drill-acme \
    --workflow-id customer-onboarding \
    --version 1 \
    --inputs '{"customer_email":"alice@drill.test","customer_company":"Drill Co","tenant_class":"paid","initial_charge_amount_minor_units":12500}'
# Output: workflow_instance_id=wf_01HZX9...
```

Watch progress:

```sh
oya workflow-engine workflow watch \
    --tenant drill-acme \
    --workflow-instance-id wf_01HZX9...
# Output: live event stream
#   step=create_customer_record status=running
#   step=create_customer_record status=succeeded principal_id=u_drill_001
#   step=send_welcome_email status=running
#   ...
```

Wait ~ 30 s for completion:

```sh
oya workflow-engine workflow get \
    --tenant drill-acme \
    --workflow-instance-id wf_01HZX9...
# Expected: status=completed, completed_at=..., 5 steps all succeeded.
```

Verify audit-chain emissions:

```sh
oya audit query --tenant drill-acme --event-class "workflow.*" --since 5m
```

Expected: 1 `workflow.requested` + 1 `workflow.completed` + 5 × `workflow.step.completed` = 7 events.

Acceptance: 5-step workflow executed end-to-end; you can articulate the saga shape + compensation policy.

## Day 4 — Saga compensation drill + external signaling

Force a step failure to exercise compensation:

```sh
oya workflow-engine workflow start \
    --tenant drill-acme \
    --workflow-id customer-onboarding \
    --version 1 \
    --inputs '{"customer_email":"compensation-test@drill.test","customer_company":"Compensation Co","tenant_class":"paid","initial_charge_amount_minor_units":12500}' \
    --inject-fault 'provision_tenant_resources:simulate-tenant-quota-exceeded'
```

Watch:

```sh
oya workflow-engine workflow watch --tenant drill-acme --workflow-instance-id wf_01HZX9...
# Expected event sequence:
#   step=create_customer_record status=succeeded
#   step=send_welcome_email status=succeeded
#   step=charge_first_invoice status=succeeded
#   step=provision_tenant_resources status=failed reason=tenant_quota_exceeded
#   compensation: deprovision_tenant_resources status=running
#   compensation: deprovision_tenant_resources status=succeeded
#   compensation: refund_first_invoice status=running
#   compensation: refund_first_invoice status=succeeded
#   workflow status=compensated
```

This is the saga pattern: forward steps succeed, then one fails, compensation runs in reverse for the steps that already succeeded. Audit-chain shows the full sequence including compensation events.

External signaling: some workflows wait for an external event (e.g., "approve the loan when the underwriter signs off"). Send a signal:

```sh
# Imagine a loan-approval workflow waiting for underwriter signoff:
oya workflow-engine workflow start \
    --tenant drill-acme \
    --workflow-id loan-approval-with-signoff \
    --inputs '{"applicant_id":"app-001","amount_minor_units":2500000}'
# Output: workflow_instance_id=wf_01HZX9B...

# The workflow now sits at the "await_underwriter_signoff" step.
oya workflow-engine workflow signal-send \
    --tenant drill-acme \
    --workflow-instance-id wf_01HZX9B... \
    --signal-name underwriter_signoff \
    --signal-payload '{"underwriter":"u-underwriter-42","decision":"approved","conditions":"none"}'

# The workflow resumes.
```

Acceptance: saga compensation flow tested; external signal sent + received; you understand the durable-function model where workflows can wait indefinitely for external events.

## Day 5 — Cross-AZ migration drill + workflow-stuck runbook

Read `runbooks/workflow-stuck-at-retries.md` first.

Run the cross-AZ migration drill (rehearsal mode for demo_trial; production for paid):

```sh
oya workflow-engine drill az-failure \
    --cell drill-syd-1 \
    --target-az syd-1a \
    --duration 10m \
    --background-load 100-workflows-per-sec
```

The drill simulates AZ-A failure:

1. Cordons all worker pods in AZ-A.
2. In-flight workflows (~ 600 active at 100/sec × 6 s avg) lose their workers.
3. Per-workflow lease in Valkey expires after 30 s.
4. Surviving AZs' worker pods pick up via lease re-acquisition.
5. Workflows resume by event-log replay.
6. After 10 min, restore AZ-A.

Expected outcome: zero workflows lost; ≤ 30 s pause per workflow during the cutover; alerts page on-call after the cutover (post-mortem trail).

Walk a stuck-at-retries scenario. The runbook covers:

1. Identify the stuck workflow from `workflow-queue-depth` panel (queue depth growing without throughput).
2. Query the workflow's event log: `oya workflow-engine workflow events --workflow-instance-id wf_...`. Look for repeated retry events on the same step.
3. Identify the underlying error: query the step's adapter logs (e.g., if the step calls `payments.charge_create`, check the `payments` µservice logs for the failed attempt).
4. Decide: temporary external outage (let retries continue) or pathological bug (cancel the workflow + page on-call).
5. If cancel: `oya workflow-engine workflow cancel --workflow-instance-id wf_... --reason stuck_at_retries`.

Target end-to-end recovery: ≤ 30 min for the drill (production target ≤ 1 h per `slos/workflow-stuck-recovery.openslo.yaml`).

Acceptance: drill executed; stuck-at-retries runbook walked; you can recover from a sustained downstream-µservice outage.

## What you've learned

- demo_trial bootstrap + 5-step workflow definition with saga compensation.
- External signaling + the durable-function execution model.
- Cross-AZ migration drill (the most-likely page for paid production workloads).
- The workflow-stuck-at-retries recovery path.

Next week: paid tenant-class promotion (per-tenant worker-pool isolation + retry policy override), paid multi-AZ tour (sharded worker pool + cross-AZ failover + 30-d workflow duration), paid sovereign-pack tour (sovereign-pack workflow residency), and your first production shadow.
