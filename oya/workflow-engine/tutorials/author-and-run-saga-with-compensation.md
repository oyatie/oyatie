---
doc_class: Tutorial
microservice: workflow-engine
persona: workflow-engineer + saga-architect
date: 2026-05-20
doc_status: published
---

# Tutorial — Author + run an order-fulfillment saga with compensation across payments + inventory + shipping

You will: define a 4-step order-fulfillment saga (validate inventory → charge customer → reserve shipping → notify customer), exercise the forward path, then exercise the compensation path by failing the shipping step, verify the resulting refund + inventory-release + customer-notification. Total time ≤ 60 minutes.

## Pre-requisites

- A tenant cell eligible for the paid tenant class.
- `oya-dev-cli` ≥ 1.42.0.
- A tenant principal in the `workflow_admin` Cedar role.
- The tenant must have `payments` µservice configured + a synthetic merchant set up.

## Step 1 — Define the saga (≤ 15 min)

```yaml
# order-fulfillment-v1.yaml
workflow_id: order-fulfillment
version: 1
description: "Fulfill an order: validate inventory, charge customer, reserve shipping slot, notify customer of dispatch"

inputs:
  tenant_id:             {type: string, required: true}
  customer_id:           {type: string, required: true}
  order_id:              {type: string, required: true}
  line_items:            {type: array, required: true, items: {sku: string, quantity: integer}}
  payment_method_id:     {type: string, required: true}
  amount_minor_units:    {type: integer, required: true}
  currency:              {type: string, default: USD}
  shipping_address:      {type: object, required: true}

steps:
  - id: validate_and_reserve_inventory
    handler: ontology.inventory.reserve
    inputs:
      tenant_id: "{{tenant_id}}"
      line_items: "{{line_items}}"
      reservation_id: "res-{{order_id}}"
    outputs:
      reservation_id: $.reservation_id
      total_quantity_reserved: $.total_quantity_reserved
    timeout_seconds: 30
    retry:
      max_attempts: 3
      backoff: exponential
      retryable_errors: [transient, rate_limited]
      non_retryable_errors: [insufficient_inventory]
    compensation:
      id: release_inventory_reservation
      handler: ontology.inventory.release
      inputs:
        reservation_id: "{{validate_and_reserve_inventory.reservation_id}}"

  - id: charge_customer
    handler: payments.charge_create
    inputs:
      customer_id: "{{customer_id}}"
      payment_method_id: "{{payment_method_id}}"
      amount_minor_units: "{{amount_minor_units}}"
      currency: "{{currency}}"
      description: "Order {{order_id}}"
      idempotency_key: "charge-{{order_id}}"
    outputs:
      charge_id: $.charge_id
    timeout_seconds: 30
    retry:
      max_attempts: 3
      backoff: exponential
      retryable_errors: [transient, psp_unavailable, rate_limited]
      non_retryable_errors: [card_declined, card_declined_velocity]
    compensation:
      id: refund_customer
      handler: payments.refund_create
      inputs:
        charge_id: "{{charge_customer.charge_id}}"
        reason: "workflow_compensation"

  - id: reserve_shipping_slot
    handler: workflow.tenant.shipping_reserve  # tenant-owned handler
    inputs:
      shipping_address: "{{shipping_address}}"
      line_items: "{{line_items}}"
      order_id: "{{order_id}}"
    outputs:
      shipping_slot_id: $.shipping_slot_id
      eta_iso: $.eta_iso
    timeout_seconds: 60
    retry:
      max_attempts: 2
      backoff: exponential
      retryable_errors: [transient]
    compensation:
      id: cancel_shipping_reservation
      handler: workflow.tenant.shipping_cancel
      inputs:
        shipping_slot_id: "{{reserve_shipping_slot.shipping_slot_id}}"

  - id: notify_customer
    handler: mail.send_template
    inputs:
      template_id: order_fulfillment_dispatched
      recipient: "{{customer_id}}"
      template_data:
        order_id: "{{order_id}}"
        eta: "{{reserve_shipping_slot.eta_iso}}"
        shipping_slot_id: "{{reserve_shipping_slot.shipping_slot_id}}"
    timeout_seconds: 60
    retry:
      max_attempts: 5
      backoff: exponential_jitter

transitions:
  - from: validate_and_reserve_inventory
    to: charge_customer
    on: success
  - from: charge_customer
    to: reserve_shipping_slot
    on: success
  - from: reserve_shipping_slot
    to: notify_customer
    on: success

compensation_policy:
  trigger: any_step_failure
  order: reverse_chronological
  on_compensation_failure: escalate_to_on_call
  audit_emit: every_compensation_step
```

Register it:

```sh
oya workflow-engine workflow register \
    --tenant acme-corp \
    --definition ./order-fulfillment-v1.yaml
```

## Step 2 — Forward-path execution (≤ 10 min)

```sh
oya workflow-engine workflow start \
    --tenant acme-corp \
    --workflow-id order-fulfillment \
    --version 1 \
    --inputs '{
      "tenant_id":"acme-corp",
      "customer_id":"cust-001",
      "order_id":"ord-001",
      "line_items":[{"sku":"WIDGET-1","quantity":2},{"sku":"GADGET-2","quantity":1}],
      "payment_method_id":"pm_acme_001",
      "amount_minor_units":12500,
      "currency":"USD",
      "shipping_address":{"line1":"123 Main St","city":"Sydney","state":"NSW","postal_code":"2000","country":"AU"}
    }'
# Output: workflow_instance_id=wf_01HZX9...
```

Watch:

```sh
oya workflow-engine workflow watch --tenant acme-corp --workflow-instance-id wf_01HZX9...
```

Expected event stream:

```
[2026-05-20T14:32:01Z] step=validate_and_reserve_inventory status=running
[2026-05-20T14:32:02Z] step=validate_and_reserve_inventory status=succeeded reservation_id=res-ord-001 total_quantity_reserved=3
[2026-05-20T14:32:02Z] step=charge_customer status=running
[2026-05-20T14:32:04Z] step=charge_customer status=succeeded charge_id=ch_acme_001
[2026-05-20T14:32:04Z] step=reserve_shipping_slot status=running
[2026-05-20T14:32:18Z] step=reserve_shipping_slot status=succeeded shipping_slot_id=slot_001 eta_iso=2026-05-23T10:00:00+1000
[2026-05-20T14:32:18Z] step=notify_customer status=running
[2026-05-20T14:32:20Z] step=notify_customer status=succeeded
[2026-05-20T14:32:20Z] workflow status=completed
```

Total wall-clock ~ 20 s. Verify the side effects:

```sh
# Inventory reserved
oya ontology query --tenant acme-corp --predicate "reservation.id = 'res-ord-001'"
# Expected: reservation row with status=active, quantities reserved.

# Payment charged
oya payments charge get --tenant acme-corp --charge ch_acme_001
# Expected: status=succeeded, amount=12500 USD.

# Mail sent
oya mail message-history --tenant acme-corp --recipient cust-001 --since 5m
# Expected: 1 message with template=order_fulfillment_dispatched.
```

## Step 3 — Compensation-path execution (≤ 15 min)

Force a failure in the shipping step:

```sh
oya workflow-engine workflow start \
    --tenant acme-corp \
    --workflow-id order-fulfillment \
    --version 1 \
    --inputs '{
      "tenant_id":"acme-corp",
      "customer_id":"cust-001",
      "order_id":"ord-002",
      "line_items":[{"sku":"WIDGET-1","quantity":2}],
      "payment_method_id":"pm_acme_001",
      "amount_minor_units":8000,
      "currency":"USD",
      "shipping_address":{"line1":"FAULT-INJECT","city":"Sydney","state":"NSW","postal_code":"2000","country":"AU"}
    }' \
    --inject-fault 'reserve_shipping_slot:simulate-no-shipping-capacity'
# Output: workflow_instance_id=wf_01HZX9B...
```

Watch:

```sh
oya workflow-engine workflow watch --tenant acme-corp --workflow-instance-id wf_01HZX9B...
```

Expected event stream:

```
[14:34:01] step=validate_and_reserve_inventory status=succeeded reservation_id=res-ord-002
[14:34:03] step=charge_customer status=succeeded charge_id=ch_acme_002
[14:34:03] step=reserve_shipping_slot status=running
[14:34:65] step=reserve_shipping_slot status=failed reason=no_shipping_capacity (attempt 1/2)
[14:34:78] step=reserve_shipping_slot status=failed reason=no_shipping_capacity (attempt 2/2)
[14:34:79] step=reserve_shipping_slot status=failed_exhausted
[14:34:79] compensation_policy triggered: reverse_chronological
[14:34:80] compensation: refund_customer status=running
[14:34:82] compensation: refund_customer status=succeeded refund_id=re_acme_002
[14:34:82] compensation: release_inventory_reservation status=running
[14:34:83] compensation: release_inventory_reservation status=succeeded
[14:34:83] workflow status=compensated
```

Verify side effects:

```sh
# Inventory released
oya ontology query --tenant acme-corp --predicate "reservation.id = 'res-ord-002'"
# Expected: reservation status=released.

# Refund issued
oya payments refund get --tenant acme-corp --refund re_acme_002
# Expected: refund status=succeeded, amount=8000 USD, charge_id=ch_acme_002.

# No customer notification was sent (the shipping step failed before it)
oya mail message-history --tenant acme-corp --recipient cust-001 --since 2m
# Expected: 0 messages with template=order_fulfillment_dispatched for ord-002.
```

## Step 4 — Audit-chain verification (≤ 10 min)

```sh
oya audit query --tenant acme-corp --event-class "workflow.*" --since 10m
oya audit query --tenant acme-corp --event-class "payments.*" --since 10m
oya audit query --tenant acme-corp --event-class "ontology.*" --since 10m
```

Expected events for the compensation workflow:

- `workflow.requested` (1)
- `workflow.step.completed` for `validate_and_reserve_inventory`, `charge_customer` (2)
- `workflow.step.failed` for `reserve_shipping_slot` (2 — one per attempt)
- `workflow.step.failed_exhausted` (1)
- `workflow.compensation.started` (1)
- `workflow.compensation.step.completed` for `refund_customer`, `release_inventory_reservation` (2)
- `workflow.compensated` (1)
- Plus the underlying `payments.charge.created`, `payments.refund.executed`, `ontology.inventory.reserved`, `ontology.inventory.released` events.

All Ed25519-signed; chain verifies:

```sh
oya audit verify-chain --tenant acme-corp --since 10m
# Output: chain verified, all events signed, signature gaps: 0.
```

## Step 5 — Walk the OpenTelemetry trace (≤ 10 min)

The workflow emitted a trace-id at start. Extract:

```sh
WORKFLOW_TRACE_ID=$(oya workflow-engine workflow get \
    --tenant acme-corp \
    --workflow-instance-id wf_01HZX9B... \
    --json | jq -r .trace_id)

oya observability trace get --trace-id $WORKFLOW_TRACE_ID
```

The trace shows the full span tree:

```
workflow.order-fulfillment.run (20s)
├── workflow.step.validate_and_reserve_inventory (1s)
│   └── ontology.inventory.reserve (0.8s)
├── workflow.step.charge_customer (2s)
│   ├── payments.routing-policy.evaluate (0.05s)
│   ├── payments.psp.stripe.api-call (1.8s)
│   └── payments.ledger.post (0.1s)
├── workflow.step.reserve_shipping_slot (75s, attempt 1)
│   └── tenant.shipping_reserve (75s, FAIL)
├── workflow.step.reserve_shipping_slot (13s, attempt 2)
│   └── tenant.shipping_reserve (13s, FAIL)
├── workflow.compensation.refund_customer (2s)
│   └── payments.refund_create (1.9s)
└── workflow.compensation.release_inventory_reservation (1s)
    └── ontology.inventory.release (0.9s)
```

The trace is invaluable for debugging where time was spent + which sub-µservice caused a failure.

## What you've learned

- Multi-step saga authoring with compensation per step.
- Forward-path success + compensation-path execution.
- Fault injection to validate compensation invariants.
- Audit-chain emission for every step + compensation.
- OpenTelemetry trace walk across workflow + downstream µservices.

Next tutorial: `tutorials/long-running-workflow-with-signals.md` — author a year-long care-coordination workflow waiting for external signals.
