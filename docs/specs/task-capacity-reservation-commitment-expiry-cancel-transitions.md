# Spec: capacity-reservation-commitment-expiry-cancel-transitions

## Crate

`cloud-capacity-domain`

## Problem

`CapacityReservationState` and `CommitmentState` both declare `Expired` and
`Cancelled` terminal variants, but no transition logic exists to move a record
from `Active` to either terminal state. Callers cannot enforce lifecycle
correctness purely in the domain layer.

## Solution

Add two methods to each of `CapacityReservation` and `CommittedUseContract`:

### `expire_at(now_epoch_seconds: u64) -> Result<Self, CloudCapacityError>`

| Current state | `now >= end_epoch_seconds` | Result              |
|---------------|---------------------------|---------------------|
| `Active`      | true                      | `Ok(Self{Expired})` |
| `Active`      | false                     | `Err(InvalidTimeOrder)` |
| `Expired`     | any                       | `Err(InvalidTransition)` |
| `Cancelled`   | any                       | `Err(InvalidTransition)` |

### `cancel() -> Result<Self, CloudCapacityError>`

| Current state | Result                    |
|---------------|---------------------------|
| `Active`      | `Ok(Self{Cancelled})`     |
| `Expired`     | `Err(InvalidTransition)`  |
| `Cancelled`   | `Err(InvalidTransition)`  |

## Error variant

`CloudCapacityError::InvalidTransition` — added to existing enum; no new deps.

## Acceptance criteria

- `expire_at` on Active record with `now >= end_epoch_seconds` returns `Ok` with
  state `Expired`.
- `expire_at` on Active record with `now < end_epoch_seconds` returns
  `Err(InvalidTimeOrder)`.
- `expire_at` on Expired record returns `Err(InvalidTransition)`.
- `cancel` on Active record returns `Ok` with state `Cancelled`.
- Double `cancel` (cancel then cancel) returns `Err(InvalidTransition)`.
- `cancel` on Expired record returns `Err(InvalidTransition)`.
- All six scenarios tested for both `CapacityReservation` and
  `CommittedUseContract` (12+ tests total, satisfies >=10 requirement).
- Pure deterministic: no I/O, no new deps, no new workspace member.
- Root `Cargo.toml` untouched.
