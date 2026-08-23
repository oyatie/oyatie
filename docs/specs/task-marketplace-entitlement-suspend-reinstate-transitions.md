# Spec: marketplace-entitlement-suspend-reinstate-transitions

**Crate**: `cloud-marketplace-kernel`  
**Lane**: cloud  
**Priority**: high  
**Effort**: S

## Background

The `cloud-marketplace-kernel` crate exposes a pure, I/O-free model for
marketplace offers and entitlements. `EntitlementState::Suspended` is already
declared but has no associated transitions. This slice adds the two missing
lifecycle operations so the state machine is complete.

## State Machine

```
Pending ──activate──> Active ──suspend──> Suspended
   │                    │                    │
   └──cancel──> Cancelled ◄──cancel──────────┘
                   ▲                         │
                   └──cancel─────────────────┘
                                             │
                            reinstate ───────┘ (back to Active)
```

## Acceptance Criteria

### `pub fn suspend(ent: &mut Entitlement) -> Result<(), MarketplaceError>`

| Pre-condition | Post-condition |
|---|---|
| `ent.state == Active` | `ent.state = Suspended`, returns `Ok(())` |
| `ent.state != Active` | returns `Err(InvalidEntitlementTransition { from: ent.state, to: Suspended })` |

### `pub fn reinstate(ent: &mut Entitlement, offer: &Offer, now_unix_ms: u64, buyer_in_good_standing: bool) -> Result<(), MarketplaceError>`

| Pre-condition | Post-condition |
|---|---|
| `ent.state == Suspended` AND `offer.state == Published` AND `now_unix_ms < offer.valid_until_unix_ms` AND `buyer_in_good_standing` | `ent.state = Active`, returns `Ok(())` |
| `ent.state != Suspended` | returns `Err(InvalidEntitlementTransition { from: ent.state, to: Active })` |
| `offer.state != Published` | returns `Err(OfferNotPublished { .. })` |
| `now_unix_ms >= offer.valid_until_unix_ms` | returns `Err(OfferExpired { .. })` |
| `!buyer_in_good_standing` | returns `Err(BuyerSuspended)` |

Validation order for `reinstate`: state check first, then offer-published check,
then expiry check, then buyer standing check.

## Existing Transitions Preserved

- `activate_entitlement` (Pending → Active): unchanged
- `cancel` (* → Cancelled, except Cancelled → Cancelled rejected): unchanged

## Test Coverage Required (>= 8 tests)

1. `suspend_active_succeeds` — Active → Suspended ok
2. `suspend_pending_rejected` — Pending source rejected
3. `suspend_suspended_rejected` — already Suspended rejected
4. `suspend_cancelled_rejected` — Cancelled source rejected
5. `reinstate_suspended_succeeds` — Suspended → Active ok
6. `reinstate_from_active_rejected` — illegal source state
7. `reinstate_expired_offer_rejected` — offer expired
8. `reinstate_unpublished_offer_rejected` — offer not Published
9. `reinstate_suspended_buyer_rejected` — buyer not in good standing
10. `reinstate_from_pending_rejected` — Pending source rejected

## Zero Dependencies Policy

No new crate dependencies. No new workspace members. Root `Cargo.toml` untouched.
