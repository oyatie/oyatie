# Plan: marketplace-entitlement-suspend-reinstate-transitions

## Objective

Extend `oya-cloud-marketplace-kernel` with two new entitlement lifecycle
transitions for the already-declared `EntitlementState::Suspended` variant:

- `suspend(ent)` — Active → Suspended
- `reinstate(ent, offer, now_unix_ms, buyer_in_good_standing)` — Suspended → Active
  (re-validates offer Published + non-expired + buyer in good standing)

Illegal source states are rejected via `MarketplaceError::InvalidEntitlementTransition`.

## Constraints

- Zero new dependencies, no new workspace member, root `Cargo.toml` untouched
- All changes in `crates/oya-cloud-marketplace-kernel/src/lib.rs`
- Hermetic unit tests only (no I/O)
- ADR-0509 flat clean-arch: no new modules, no separate files

## Tasks

1. Write spec doc at `docs/specs/task-marketplace-entitlement-suspend-reinstate-transitions.md`
2. Add `suspend` function (Active → Suspended)
3. Add `reinstate` function (Suspended → Active, re-validates offer + buyer)
4. Add >= 8 unit tests covering:
   - `suspend` legal (Active → Suspended) ✓
   - `suspend` illegal source states: Pending, Suspended, Cancelled ✓
   - `reinstate` legal (Suspended → Active) ✓
   - `reinstate` illegal source states: Pending, Active, Cancelled ✓
   - `reinstate` expired offer rejection ✓
   - `reinstate` unpublished offer rejection ✓
   - `reinstate` suspended buyer rejection ✓
5. `cargo check -p oya-cloud-marketplace-kernel --all-targets` green
6. `cargo nextest run -p oya-cloud-marketplace-kernel` green
