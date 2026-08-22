# Spec: Messenger Delivery-Receipt State Machine

**Vertical**: messenger
**Task slug**: messenger-delivery-receipt-state-machine
**Crate**: `messenger-message-stream-usecase`
**ADR authority**: ADR-0509 (flat single-crate per service, mod-based clean arch)

---

## Objective

Extend the messenger message-stream usecase crate with a delivery-receipt progression state machine.
Each message recipient has an independent delivery status that advances monotonically
`Sent → Delivered → Read`. The slice exposes:

1. Core domain types: `DeliveryStatus`, `RecipientDeliveryState`
2. An idempotent `acknowledge_delivery` usecase function
3. A channel-level aggregate that derives the lowest-common delivery state across all recipients

All orchestration is runtime-neutral — no DB I/O, no broker publish, consistent with the existing
`send_message` no-I/O posture.

---

## Vertical & Boundaries

```
messenger (microservice)
└── crates/
    ├── messenger-domain              # domain invariants (read-only for this task)
    ├── messenger-message-stream-api  # API types + AuthorizedMessengerContext (read-only)
    └── messenger-message-stream-usecase  ← ONLY crate modified by this task
```

This task MUST NOT touch root `Cargo.toml`, any other crate, or create new workspace members.

---

## Mod Layout (flat clean-arch inside `src/`)

```
src/
├── lib.rs               # existing: re-exports send_message, prepare_disclosure_audit
└── delivery_receipt.rs  # NEW: DeliveryStatus, RecipientDeliveryState,
                         #      acknowledge_delivery, ChannelDeliveryAggregate,
                         #      aggregate_channel_delivery
```

`lib.rs` gains `pub mod delivery_receipt;` and re-exports the public surface.

---

## Type Contracts

### `DeliveryStatus`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DeliveryStatus {
    Sent,
    Delivered,
    Read,
}
```

Total order: `Sent < Delivered < Read`. `Read` is the terminal-forward state — no transition
from `Read` to any earlier variant is permitted.

### `RecipientDeliveryState`

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipientDeliveryState {
    pub recipient_ref: String,
    pub status: DeliveryStatus,
    pub ordinal: u64,
    pub last_idempotency_key: String,
}
```

`ordinal` is a monotonic counter incremented on each accepted forward transition.
`last_idempotency_key` enables idempotent replay detection.

### `MessengerUsecaseError` extension

```rust
// added variant (existing enum in lib.rs):
IllegalDeliveryTransition { from: DeliveryStatus, to: DeliveryStatus },
```

### `ChannelDeliveryAggregate`

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChannelDeliveryAggregate {
    pub channel_id: String,
    pub aggregate_status: DeliveryStatus,
}
```

Derived as `min(recipient statuses)` over the `Ord` on `DeliveryStatus`.
Empty recipient slice defaults to `DeliveryStatus::Sent`.

---

## Function Contracts

### `acknowledge_delivery`

```rust
pub fn acknowledge_delivery(
    ctx: &AuthorizedMessengerContext,
    state: &RecipientDeliveryState,
    recipient: &str,
    target: DeliveryStatus,
    idempotency_key: &str,
) -> Result<RecipientDeliveryState, MessengerUsecaseError>
```

Steps (in order):

1. `ctx.validate()` — map `MessengerApiError` → `MessengerUsecaseError::Api`
2. `ctx.principal_ref != recipient` → `MessengerUsecaseError::PrincipalMismatch`
3. `idempotency_key == state.last_idempotency_key` → `Ok(state.clone())` (idempotent replay)
4. `target <= state.status` → `MessengerUsecaseError::IllegalDeliveryTransition { from: state.status, to: target }`
5. Return `RecipientDeliveryState { status: target, ordinal: state.ordinal + 1, last_idempotency_key: idempotency_key.to_owned(), ..state.clone() }`

### `aggregate_channel_delivery`

```rust
pub fn aggregate_channel_delivery(
    channel_id: &str,
    recipients: &[RecipientDeliveryState],
) -> ChannelDeliveryAggregate
```

Returns `ChannelDeliveryAggregate { channel_id: channel_id.to_owned(), aggregate_status }` where
`aggregate_status = recipients.iter().map(|r| r.status).min().unwrap_or(DeliveryStatus::Sent)`.

---

## OpenAPI 3.2.0 Sketch (informational — adapter concern, not implemented here)

```yaml
# POST /channels/{channelId}/receipts/{recipientRef}/ack
requestBody:
  content:
    application/json:
      schema:
        type: object
        required: [targetStatus, idempotencyKey]
        properties:
          targetStatus: { type: string, enum: [Delivered, Read] }
          idempotencyKey: { type: string }
responses:
  "200":
    description: Current delivery state (advanced or unchanged on replay)
  "409":
    description: Illegal regression attempt
```

## proto3 Sketch (informational — gRPC adapter concern, not implemented here)

```proto
syntax = "proto3";
package oya.messenger.v1;

enum DeliveryStatus {
  DELIVERY_STATUS_SENT = 0;
  DELIVERY_STATUS_DELIVERED = 1;
  DELIVERY_STATUS_READ = 2;
}

message AcknowledgeDeliveryRequest {
  string channel_id = 1;
  string recipient_ref = 2;
  DeliveryStatus target_status = 3;
  string idempotency_key = 4;
}

message RecipientDeliveryStateProto {
  string recipient_ref = 1;
  DeliveryStatus status = 2;
  uint64 ordinal = 3;
  string last_idempotency_key = 4;
}

service MessageStream {
  rpc AcknowledgeDelivery(AcknowledgeDeliveryRequest)
      returns (RecipientDeliveryStateProto);
}
```

---

## Testing Strategy

All tests live in `src/delivery_receipt.rs` under `#[cfg(test)]` (consistent with existing crate pattern).

| Test | What it proves |
|------|----------------|
| `happy_path_sent_delivered_read` | Full progression advances ordinal and status each step |
| `idempotent_replay_returns_unchanged` | Same idempotency key → `Ok(state.clone())`, ordinal unchanged |
| `regression_read_to_delivered_rejected` | `Read → Delivered` → `IllegalDeliveryTransition` |
| `context_validation_failure_maps_to_api_error` | Invalid ctx → `MessengerUsecaseError::Api` |
| `aggregate_min_status_mixed_recipients` | One `Sent` + two `Read` → aggregate `Sent` |
| `aggregate_all_read` | All `Read` → aggregate `Read` |
| `aggregate_empty_slice_defaults_to_sent` | Empty → aggregate `Sent` |

---

## Constraints

- Runtime-neutral: no `async`, no I/O, no external crate dependencies added
- No new workspace member; root `Cargo.toml` unchanged
- All public types derive `Clone, Debug, Eq, PartialEq`; `DeliveryStatus` additionally derives `Ord, PartialOrd, Copy`
- `#![cfg_attr(test, allow(clippy::unwrap_used, ...))]` already present in `lib.rs`
