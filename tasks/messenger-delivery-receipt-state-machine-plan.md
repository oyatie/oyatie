# Plan: messenger-delivery-receipt-state-machine

Vertical: messenger
Crate: `oya-messenger-message-stream-usecase`
Branch: `feat/task-messenger-delivery-receipt-state-machine-2026-05-28`

## Subtasks

### [messenger-delivery-receipt-state-machine-1] Define core types

**Scope**: Add `DeliveryStatus`, `RecipientDeliveryState`, and `MessengerUsecaseError::IllegalDeliveryTransition`
to `src/delivery_receipt.rs` (new mod) inside the usecase crate.

**Deliverables**:
- `DeliveryStatus` enum: `Sent`, `Delivered`, `Read` — total order `Sent < Delivered < Read`
- `RecipientDeliveryState` struct: `recipient_ref: String`, `status: DeliveryStatus`, `ordinal: u64`
- `MessengerUsecaseError::IllegalDeliveryTransition` variant with `from` and `to` fields
- All types `pub`; derive `Clone, Debug, Eq, PartialEq, Ord, PartialOrd` where meaningful

**Acceptance**:
- `cargo check -p oya-messenger-message-stream-usecase --all-targets` passes
- New types are `pub`; `DeliveryStatus` ordering is total; `Read` is terminal-forward (no regression)
- Root `Cargo.toml` unchanged; no new workspace member

---

### [messenger-delivery-receipt-state-machine-2] Idempotent `acknowledge_delivery` usecase fn

**Scope**: Add `acknowledge_delivery` function in `src/delivery_receipt.rs`.

**Signature**:
```rust
pub fn acknowledge_delivery(
    ctx: &AuthorizedMessengerContext,
    state: &RecipientDeliveryState,
    recipient: &str,
    target: DeliveryStatus,
    idempotency_key: &str,
) -> Result<RecipientDeliveryState, MessengerUsecaseError>
```

**Behaviour**:
1. `ctx.validate()` — map err to `MessengerUsecaseError::Api`
2. Principal/scope check: `ctx.principal_ref` must match `recipient` — map mismatch to `MessengerUsecaseError::PrincipalMismatch`
3. Duplicate idempotency key (same key already recorded on state): return `Ok(state.clone())` unchanged
4. Forward-only: if `target <= state.status` AND not same idempotency key → `MessengerUsecaseError::IllegalDeliveryTransition`
5. Advance: return new `RecipientDeliveryState` with `status = target`, `ordinal = state.ordinal + 1`

**Acceptance**:
- `cargo nextest run -p oya-messenger-message-stream-usecase` green
- Tests cover:
  - Happy path: `Sent → Delivered → Read`
  - Idempotent replay: same key → `Ok(unchanged)`
  - Regression rejection: `Read → Delivered` → `IllegalDeliveryTransition`
  - Context validation failures → `MessengerUsecaseError::Api`

---

### [messenger-delivery-receipt-state-machine-3] Channel-level aggregate

**Scope**: Add `ChannelDeliveryAggregate` struct and `aggregate_channel_delivery` fn in
`src/delivery_receipt.rs`.

**Deliverables**:
- `ChannelDeliveryAggregate` struct: `channel_id: String`, `aggregate_status: DeliveryStatus`
- `aggregate_channel_delivery(channel_id: &str, recipients: &[RecipientDeliveryState]) -> ChannelDeliveryAggregate`
  - Derives lowest-common (min) status across all recipients
  - Empty slice → `Sent` (safe default)

**Acceptance**:
- `cargo nextest run -p oya-messenger-message-stream-usecase` green
- `cargo check -p oya-messenger-message-stream-usecase --all-targets` passes
- Tests prove: mixed set (one `Sent` + rest `Read`) → `Sent`; all `Read` → `Read`

---

## Acceptance Summary (full slice)

| Gate | Command | Must pass |
|------|---------|-----------|
| Type-check | `cargo check -p oya-messenger-message-stream-usecase --all-targets` | yes |
| Tests | `cargo nextest run -p oya-messenger-message-stream-usecase` | yes, all green |
| Root Cargo.toml | unchanged | yes |
| New workspace member | none | yes |
