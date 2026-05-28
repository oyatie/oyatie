//! D7 — per-tenant Cedar isolation contract (forbid-wins).
//!
//! These tests use a fake `AuthzGate` that simulates Cedar's forbid-wins
//! semantics: cross-tenant requests are forbidden no matter how many `permit`
//! rules apply. The real Cedar policies + adapter live in a separate crate
//! (`oya-llm-gateway-policy-cedar-adapter`) and have their own adversarial
//! test corpus.
//!
//! Stage-4 RED: tests fail because `SubscriptionPool::select` returns
//! `NotYetImplemented` and never even consults the gate.
//! Stage-5 GREEN: kernel MUST consult the gate before returning a SeatId and
//! MUST return `ForbiddenByPolicy` on `AuthzDecision::Forbid`.
use std::cell::RefCell;
use std::time::Instant;

use oya_llm_gateway_oauth_pool_kernel::{
    AgentId, AuthzDecision, AuthzGate, AuthzRequest, OAuthSubscription, Provider, SeatId,
    SelectionStrategy, SubscriptionId, SubscriptionPool, SubscriptionPoolError, SubscriptionState,
    TenantId,
};

/// A gate that records every request it sees and forbids cross-tenant access.
/// `forbid_wins` mirrors Cedar's default-deny + forbid-wins semantics.
struct CedarLikeGate {
    requests: RefCell<Vec<(String, String)>>,
}

impl CedarLikeGate {
    fn new() -> Self {
        Self {
            requests: RefCell::new(Vec::new()),
        }
    }

    fn calls(&self) -> usize {
        self.requests.borrow().len()
    }
}

impl AuthzGate for CedarLikeGate {
    fn decide(&self, request: &AuthzRequest<'_>) -> AuthzDecision {
        self.requests.borrow_mut().push((
            request.principal_tenant.as_str().to_string(),
            request.resource_tenant.as_str().to_string(),
        ));
        if request.principal_tenant != request.resource_tenant {
            return AuthzDecision::Forbid;
        }
        AuthzDecision::Allow
    }
}

fn tenant(s: &str) -> TenantId {
    TenantId::new(s).unwrap()
}
fn agent(s: &str) -> AgentId {
    AgentId::new(s).unwrap()
}
fn seat(s: &str) -> SeatId {
    SeatId::new(s).unwrap()
}
fn sub(s: &str) -> SubscriptionId {
    SubscriptionId::new(s).unwrap()
}

fn make_sub(tenant_str: &str, seat_str: &str) -> OAuthSubscription {
    OAuthSubscription::new(
        tenant(tenant_str),
        seat(seat_str),
        sub(&format!("{seat_str}-sub-1")),
        Provider::Anthropic,
        SubscriptionState::Active,
        format!("openbao://{tenant_str}/{seat_str}/refresh"),
        0,
    )
}

#[test]
fn same_tenant_select_consults_gate_and_succeeds() {
    let mut pool = SubscriptionPool::new(
        tenant("t-acme"),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    );
    pool.add_seat(make_sub("t-acme", "seat-a")).unwrap();

    let gate = CedarLikeGate::new();
    let now = Instant::now();

    let picked = pool.select(&agent("agent-acme-1"), &gate, now).unwrap();
    assert_eq!(picked, seat("seat-a"));
    assert!(gate.calls() >= 1, "kernel MUST consult the AuthzGate");
}

#[test]
fn cross_tenant_pool_rejects_select_for_foreign_tenant_principal() {
    // The pool belongs to t-acme. A request whose principal_tenant is t-evil
    // must be forbidden. In practice the REST adapter is what builds the
    // AuthzRequest, but the kernel-level contract is: the gate's verdict is
    // authoritative.
    let mut pool = SubscriptionPool::new(
        tenant("t-acme"),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    );
    pool.add_seat(make_sub("t-acme", "seat-a")).unwrap();

    struct AlwaysForbid;
    impl AuthzGate for AlwaysForbid {
        fn decide(&self, _request: &AuthzRequest<'_>) -> AuthzDecision {
            AuthzDecision::Forbid
        }
    }

    let now = Instant::now();
    assert_eq!(
        pool.select(&agent("agent-evil-1"), &AlwaysForbid, now),
        Err(SubscriptionPoolError::ForbiddenByPolicy)
    );
}

#[test]
fn forbid_wins_even_when_pool_has_capacity() {
    let mut pool = SubscriptionPool::new(
        tenant("t-acme"),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    );
    for s in ["seat-a", "seat-b", "seat-c", "seat-d", "seat-e"] {
        pool.add_seat(make_sub("t-acme", s)).unwrap();
    }

    struct AlwaysForbid;
    impl AuthzGate for AlwaysForbid {
        fn decide(&self, _request: &AuthzRequest<'_>) -> AuthzDecision {
            AuthzDecision::Forbid
        }
    }

    let now = Instant::now();
    // Even though 5 seats are Active, a Forbid decision wins.
    assert_eq!(
        pool.select(&agent("agent-1"), &AlwaysForbid, now),
        Err(SubscriptionPoolError::ForbiddenByPolicy)
    );
}

#[test]
fn adding_seat_with_wrong_tenant_is_rejected() {
    // A pool belongs to exactly one tenant. Attempting to plant a seat from
    // a different tenant into the pool is a programmer error and a tenant-
    // isolation violation at the data plane — the kernel MUST refuse.
    let mut pool = SubscriptionPool::new(
        tenant("t-acme"),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    );
    let err = pool
        .add_seat(make_sub("t-evil", "seat-a"))
        .expect_err("foreign-tenant seat must be rejected");
    assert_eq!(err, SubscriptionPoolError::ForbiddenByPolicy);
    assert_eq!(pool.seat_count(), 0);
}

#[test]
fn invalid_identifiers_are_rejected_at_construction() {
    assert_eq!(
        TenantId::new("").unwrap_err(),
        SubscriptionPoolError::InvalidTenantId
    );
    assert_eq!(
        AgentId::new("   ").unwrap_err(),
        SubscriptionPoolError::InvalidAgentId
    );
    assert_eq!(
        SeatId::new("").unwrap_err(),
        SubscriptionPoolError::InvalidSeatId
    );
    assert_eq!(
        SubscriptionId::new("").unwrap_err(),
        SubscriptionPoolError::InvalidSubscriptionId
    );
}
