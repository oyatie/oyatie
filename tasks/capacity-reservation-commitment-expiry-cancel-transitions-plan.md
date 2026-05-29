# Plan: capacity-reservation-commitment-expiry-cancel-transitions

## Goal

Add deterministic state-transition methods to `CapacityReservation` and
`CommittedUseContract` for the terminal states `Expired` and `Cancelled` that
are declared in the enums but have no transition logic.

## Methods

### `CapacityReservation::expire_at(now_epoch_seconds: u64) -> Result<Self, CloudCapacityError>`

- Allowed only when `state == Active`
- Expires only when `now >= end_epoch_seconds` (premature expiry returns
  `InvalidTimeOrder`)
- Returns `InvalidTransition` for non-Active states

### `CapacityReservation::cancel() -> Result<Self, CloudCapacityError>`

- Allowed only when `state == Active`
- Returns `InvalidTransition` for `Expired` and `Cancelled` states

### `CommittedUseContract::expire_at(now_epoch_seconds: u64) -> Result<Self, CloudCapacityError>`

Same contract as `CapacityReservation::expire_at`.

### `CommittedUseContract::cancel() -> Result<Self, CloudCapacityError>`

Same contract as `CapacityReservation::cancel`.

## Error variant

Add `InvalidTransition` to `CloudCapacityError` (no new deps, no new crates).

## Tasks

1. Add `InvalidTransition` to `CloudCapacityError` enum.
2. Implement `expire_at` and `cancel` on `CapacityReservation`.
3. Implement `expire_at` and `cancel` on `CommittedUseContract`.
4. Write >=10 hermetic unit tests (red-first approach confirmed, then green).
5. `cargo check -p oya-cloud-capacity-domain --all-targets` passes.
6. `cargo nextest run -p oya-cloud-capacity-domain` passes.
