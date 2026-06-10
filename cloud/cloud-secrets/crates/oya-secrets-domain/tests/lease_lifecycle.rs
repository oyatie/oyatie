//! Dynamic secret-lease lifecycle (story G002: zero static secrets).
//!
//! Ladder rungs (AMENDMENT 7): unit + RED/GREEN fixture pairs for every
//! fail-closed gate (expiry bound, post-expiry renewal, renewal budget,
//! revocation dominance).

use oya_secrets_domain::{
    DynamicLease, LeaseError, LeaseId, LeasePolicy, LeaseState, MAX_LEASE_TTL_SECONDS,
    MIN_LEASE_TTL_SECONDS,
};

const T0: u64 = 1_750_000_000;
const TTL: u64 = 300;

fn lease() -> DynamicLease {
    DynamicLease::issue(
        LeaseId::new("lease/abc123").expect("lease id"),
        "secret://ten_alpha/db-creds",
        "spiffe://oyatie/ns/ten-alpha/sa/api",
        LeasePolicy::new(TTL, 2).expect("policy"),
        T0,
    )
    .expect("issue")
}

#[test]
fn green_issue_live_within_ttl() {
    let issued = lease();
    assert_eq!(issued.state(T0), LeaseState::Live);
    assert_eq!(issued.state(T0 + TTL - 1), LeaseState::Live);
    assert!(issued.assert_live(T0 + TTL - 1).is_ok());
    assert_eq!(issued.expires_at_epoch_seconds(), T0 + TTL);
    assert_eq!(issued.principal(), "spiffe://oyatie/ns/ten-alpha/sa/api");
}

#[test]
fn red_expiry_bound_is_inclusive_and_fails_closed() {
    let issued = lease();
    assert_eq!(issued.state(T0 + TTL), LeaseState::Expired);
    assert_eq!(
        issued.assert_live(T0 + TTL),
        Err(LeaseError::Expired { at_epoch_seconds: T0 + TTL })
    );
}

#[test]
fn renew_extends_from_now_never_stacking() {
    let mut renewed = lease();
    // Renew midway: new expiry = renewal time + TTL, not old expiry + TTL.
    let new_expiry = renewed.renew(T0 + 100).expect("renew");
    assert_eq!(new_expiry, T0 + 100 + TTL);
    assert_eq!(renewed.renewals_used(), 1);
    assert_eq!(renewed.state(T0 + TTL + 50), LeaseState::Live);
}

#[test]
fn red_expired_lease_cannot_renew() {
    let mut expired = lease();
    assert_eq!(
        expired.renew(T0 + TTL),
        Err(LeaseError::Expired { at_epoch_seconds: T0 + TTL })
    );
    // Still expired afterwards — no zombie revival.
    assert_eq!(expired.state(T0 + TTL), LeaseState::Expired);
}

#[test]
fn red_renewal_budget_exhausts() {
    let mut budgeted = lease();
    budgeted.renew(T0 + 10).expect("first");
    budgeted.renew(T0 + 20).expect("second");
    assert_eq!(
        budgeted.renew(T0 + 30),
        Err(LeaseError::RenewalsExhausted { max_renewals: 2 })
    );
    // Budget exhaustion does not kill the lease early; it just stops
    // extension.
    assert_eq!(budgeted.state(T0 + 30), LeaseState::Live);
}

#[test]
fn revocation_is_immediate_idempotent_and_dominates() {
    let mut revoked = lease();
    revoked.revoke(T0 + 5);
    assert_eq!(revoked.state(T0 + 6), LeaseState::Revoked);
    assert_eq!(
        revoked.assert_live(T0 + 6),
        Err(LeaseError::Revoked { at_epoch_seconds: T0 + 5 })
    );
    // Idempotent: first timestamp wins.
    revoked.revoke(T0 + 50);
    assert_eq!(
        revoked.assert_live(T0 + 60),
        Err(LeaseError::Revoked { at_epoch_seconds: T0 + 5 })
    );
    // Dominates expiry and blocks renewal.
    assert_eq!(revoked.state(T0 + TTL + 1), LeaseState::Revoked);
    assert_eq!(
        revoked.renew(T0 + 10),
        Err(LeaseError::Revoked { at_epoch_seconds: T0 + 5 })
    );
}

#[test]
fn red_zero_static_secrets_inputs_rejected() {
    let policy = LeasePolicy::new(TTL, 0).expect("policy");
    assert_eq!(
        DynamicLease::issue(LeaseId::new("lease/x").unwrap(), " ", "principal", policy, T0),
        Err(LeaseError::InvalidSecretReference)
    );
    assert_eq!(
        DynamicLease::issue(LeaseId::new("lease/x").unwrap(), "secret://s", "  ", policy, T0),
        Err(LeaseError::InvalidPrincipal)
    );
}

#[test]
fn red_ttl_bounds_enforced() {
    assert_eq!(
        LeasePolicy::new(MIN_LEASE_TTL_SECONDS - 1, 0),
        Err(LeaseError::TtlOutOfBounds { requested_seconds: MIN_LEASE_TTL_SECONDS - 1 })
    );
    assert_eq!(
        LeasePolicy::new(MAX_LEASE_TTL_SECONDS + 1, 0),
        Err(LeaseError::TtlOutOfBounds { requested_seconds: MAX_LEASE_TTL_SECONDS + 1 })
    );
    assert!(LeasePolicy::new(MIN_LEASE_TTL_SECONDS, 0).is_ok());
    assert!(LeasePolicy::new(MAX_LEASE_TTL_SECONDS, 0).is_ok());
}

#[test]
fn lease_id_validation() {
    assert!(LeaseId::new("lease/abc").is_ok());
    assert_eq!(LeaseId::new("abc"), Err(LeaseError::InvalidLeaseId));
    assert_eq!(LeaseId::new("lease/"), Err(LeaseError::InvalidLeaseId));
    assert_eq!(LeaseId::new("lease/a/b"), Err(LeaseError::InvalidLeaseId));
    assert_eq!(LeaseId::new("lease/abc").unwrap().value(), "lease/abc");
}
