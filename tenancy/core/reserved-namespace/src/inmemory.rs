//! In-memory implementations of both ports.
//!
//! These are the deterministic doubles for what will become a
//! specs-file-backed reservation source and a Cedar policy engine. They are
//! also the reference semantics those adapters must reproduce:
//!
//! - an unreachable source is
//!   [`NamespaceUsecaseError::SourceUnavailable`] carrying a cause, never an
//!   empty list — an empty list would quietly permit the platform owner's
//!   own name, and a cause-less outage cannot be triaged;
//! - an engine that cannot decide is
//!   [`NamespaceUsecaseError::CedarEvaluationFailed`], never `Ok(false)`;
//! - a grant is scoped, and an authorizer answers about
//!   (principal, action, tenant) — not about (principal, action) — because a
//!   grant held in one tenant must not mint a name in another.

use std::collections::BTreeSet;

use crate::kernel::{
    NamespaceAction, NamespaceActionAuthorizer, NamespaceCandidate, NamespaceUsecaseError,
    ReservedNamespaceSource,
};

/// In-memory reservation list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InMemoryReservedNamespaceSource {
    entries: Vec<String>, // data_class: INTERNAL_ONLY
    /// `None` when the source is reachable; otherwise the outage cause the
    /// fallible read reports.
    outage: Option<String>, // data_class: INTERNAL_ONLY
}

impl InMemoryReservedNamespaceSource {
    /// An empty, reachable source.
    ///
    /// Note that the guard REFUSES an empty list
    /// ([`NamespaceUsecaseError::EmptyReservationList`]) rather than
    /// allowing everything, so this constructor is only useful as the base
    /// of a builder chain or as the fixture for that refusal.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            outage: None,
        }
    }

    /// A source standing in for an unreachable store: every fallible read
    /// is [`NamespaceUsecaseError::SourceUnavailable`].
    #[must_use]
    pub fn unavailable() -> Self {
        Self::unavailable_because("in-memory reservation source configured as unreachable")
    }

    /// An unreachable source whose outage names its own cause, the way a
    /// real adapter must: `"/specs/platform-owner-binding.json: no such
    /// file"` and `"resolver timed out"` are different pages at 03:00 and
    /// this port is the only place that distinction can be made.
    #[must_use]
    pub fn unavailable_because(cause: impl Into<String>) -> Self {
        Self {
            entries: Vec::new(),
            outage: Some(cause.into()),
        }
    }

    /// Add one reservation entry.
    #[must_use]
    pub fn with_entry(mut self, entry: impl Into<String>) -> Self {
        self.entries.push(entry.into());
        self
    }

    /// The reservation list IP-017 §D.2 derives from the platform-owner
    /// binding: the owner token, its foundry/internal/platform-owner
    /// siblings, and the tenancy principal roster from
    /// `tenancy/ARCHITECTURE.md`.
    ///
    /// `owner` is a parameter and never a literal, per ADR-0284: pointing
    /// this crate at a different platform owner is a change to the binding,
    /// not to any code. Per ADR-0242 there is no carve-out that lets the
    /// owner claim its own namespace back through this guard.
    #[must_use]
    pub fn for_owner(owner: &str) -> Self {
        Self::new()
            .with_entry(owner)
            .with_entry(format!("{owner}-foundry"))
            .with_entry(format!("{owner}-internal"))
            .with_entry(format!("{owner}-platform-owner"))
            .with_entry(format!("{owner}.tenancy.lifecycle-controller"))
            .with_entry(format!("{owner}.tenancy.isolation-policy-emitter"))
    }
}

impl Default for InMemoryReservedNamespaceSource {
    fn default() -> Self {
        Self::new()
    }
}

impl ReservedNamespaceSource for InMemoryReservedNamespaceSource {
    fn try_reserved(&self) -> Result<Vec<String>, NamespaceUsecaseError> {
        match &self.outage {
            Some(cause) => Err(NamespaceUsecaseError::source_unavailable(cause.clone())),
            None => Ok(self.entries.clone()),
        }
    }
}

/// In-memory action authorizer standing in for Cedar.
///
/// It is a grant set, not a policy engine, but it does reproduce the one
/// structural property a real engine must have: a grant can be bound to a
/// tenant, and a tenant-bound grant answers `false` for every other tenant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InMemoryNamespaceActionAuthorizer {
    /// `(principal, action, tenant)`; `None` in the third position is a
    /// grant that is not bound to any tenant.
    grants: BTreeSet<(String, NamespaceAction, Option<String>)>, // data_class: INTERNAL_ONLY
    permit_all: bool,        // data_class: INTERNAL_ONLY
    failure: Option<String>, // data_class: INTERNAL_ONLY
}

impl InMemoryNamespaceActionAuthorizer {
    /// Permits every principal, action and tenant.
    #[must_use]
    pub fn permit_all() -> Self {
        Self {
            grants: BTreeSet::new(),
            permit_all: true,
            failure: None,
        }
    }

    /// Permits nothing until a grant is added.
    #[must_use]
    pub fn deny_all() -> Self {
        Self {
            grants: BTreeSet::new(),
            permit_all: false,
            failure: None,
        }
    }

    /// An engine that cannot reach a verdict.
    #[must_use]
    pub fn failing() -> Self {
        Self::failing_because("in-memory authorizer configured to reach no verdict")
    }

    /// An engine that cannot reach a verdict, naming why.
    #[must_use]
    pub fn failing_because(cause: impl Into<String>) -> Self {
        Self {
            grants: BTreeSet::new(),
            permit_all: false,
            failure: Some(cause.into()),
        }
    }

    /// Grant one principal one action in EVERY tenant.
    ///
    /// The unscoped form, kept for callers that gate an action which has no
    /// tenant context ([`NamespaceAction::CreateTenant`]). Reach for
    /// [`InMemoryNamespaceActionAuthorizer::with_scoped_grant`] for anything
    /// that names an existing tenant: an unscoped grant is exactly the
    /// cross-tenant authority a resource-scoped policy exists to withhold.
    #[must_use]
    pub fn with_grant(mut self, principal: impl Into<String>, action: NamespaceAction) -> Self {
        self.grants.insert((principal.into(), action, None));
        self
    }

    /// Grant one principal one action inside ONE tenant.
    #[must_use]
    pub fn with_scoped_grant(
        mut self,
        principal: impl Into<String>,
        action: NamespaceAction,
        tenant: impl Into<String>,
    ) -> Self {
        self.grants
            .insert((principal.into(), action, Some(tenant.into())));
        self
    }
}

impl Default for InMemoryNamespaceActionAuthorizer {
    /// Deny-by-default, because a policy engine that has been configured
    /// with nothing has authorized nothing.
    fn default() -> Self {
        Self::deny_all()
    }
}

impl NamespaceActionAuthorizer for InMemoryNamespaceActionAuthorizer {
    fn authorize(&self, input: &NamespaceCandidate) -> Result<bool, NamespaceUsecaseError> {
        if let Some(cause) = &self.failure {
            return Err(NamespaceUsecaseError::cedar_evaluation_failed(
                cause.clone(),
            ));
        }
        if self.permit_all {
            return Ok(true);
        }
        let unscoped = (input.principal.clone(), input.action, None);
        if self.grants.contains(&unscoped) {
            return Ok(true);
        }
        let Some(tenant) = input.tenant_context() else {
            return Ok(false);
        };
        Ok(self.grants.contains(&(
            input.principal.clone(),
            input.action,
            Some(tenant.to_owned()),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_binding_reserves_the_owner_and_its_siblings() {
        let source = InMemoryReservedNamespaceSource::for_owner("oyatie");
        let entries = source.try_reserved().expect("fixture source is reachable");
        assert_eq!(entries.len(), 6);
        assert!(entries.contains(&"oyatie".to_owned()));
        assert!(entries.contains(&"oyatie-internal".to_owned()));
        assert!(entries.contains(&"oyatie.tenancy.lifecycle-controller".to_owned()));
    }

    #[test]
    fn owner_name_is_a_parameter_not_a_literal() {
        let entries = InMemoryReservedNamespaceSource::for_owner("northwind")
            .try_reserved()
            .expect("fixture source is reachable");
        assert!(entries.contains(&"northwind".to_owned()));
        assert!(!entries.iter().any(|entry| entry.contains("oyatie")));
    }

    #[test]
    fn unavailable_source_errors_instead_of_reporting_an_empty_list() {
        let source =
            InMemoryReservedNamespaceSource::unavailable_because("/specs/binding.json: not found");
        assert_eq!(
            source.try_reserved(),
            Err(NamespaceUsecaseError::source_unavailable(
                "/specs/binding.json: not found"
            ))
        );
        // Two outages of the same port are distinguishable by their cause.
        assert_ne!(
            source.try_reserved(),
            InMemoryReservedNamespaceSource::unavailable_because("resolver timed out")
                .try_reserved()
        );
    }

    #[test]
    fn the_infallible_view_is_defaulted_and_never_the_decision_path() {
        // `reserved()` is a diagnostics convenience with a default body; a
        // decision path calls `try_reserved`, which is the required method
        // and therefore cannot be satisfied by returning an empty vec.
        let source = InMemoryReservedNamespaceSource::unavailable();
        assert!(source.reserved().is_empty());
        assert!(
            source
                .try_reserved()
                .expect_err("an unreachable source has no list")
                .is_port_failure()
        );
        let healthy = InMemoryReservedNamespaceSource::for_owner("oyatie");
        assert_eq!(
            healthy.reserved(),
            healthy.try_reserved().expect("reachable")
        );
    }

    #[test]
    fn authorizer_grants_are_per_principal_and_per_action() {
        let authorizer = InMemoryNamespaceActionAuthorizer::deny_all()
            .with_grant("tenant.acme.admin", NamespaceAction::CreateTenant);
        let create =
            NamespaceCandidate::new("acme", "tenant.acme.admin", NamespaceAction::CreateTenant);
        let rename =
            NamespaceCandidate::new("acme", "tenant.acme.admin", NamespaceAction::RenameTenant);
        let other =
            NamespaceCandidate::new("acme", "tenant.other.admin", NamespaceAction::CreateTenant);
        assert_eq!(authorizer.authorize(&create), Ok(true));
        assert_eq!(authorizer.authorize(&rename), Ok(false));
        assert_eq!(authorizer.authorize(&other), Ok(false));
        assert_eq!(
            InMemoryNamespaceActionAuthorizer::default().authorize(&create),
            Ok(false)
        );
    }

    #[test]
    fn a_scoped_grant_does_not_reach_into_another_tenant() {
        let authorizer = InMemoryNamespaceActionAuthorizer::deny_all().with_scoped_grant(
            "tenant.acme.admin",
            NamespaceAction::CreateSubScope,
            "acme",
        );
        let alias = |tenant: &str| {
            NamespaceCandidate::new(
                "billing",
                "tenant.acme.admin",
                NamespaceAction::CreateSubScope,
            )
            .in_tenant(tenant)
        };
        assert_eq!(authorizer.authorize(&alias("acme")), Ok(true));
        assert_eq!(authorizer.authorize(&alias("zeta")), Ok(false));
        // And with no tenant context at all, a scoped grant matches nothing.
        assert_eq!(
            authorizer.authorize(&NamespaceCandidate::new(
                "billing",
                "tenant.acme.admin",
                NamespaceAction::CreateSubScope,
            )),
            Ok(false)
        );
    }

    #[test]
    fn failing_authorizer_errors_rather_than_denying_and_names_the_cause() {
        let candidate =
            NamespaceCandidate::new("acme", "tenant.acme.admin", NamespaceAction::CreateTenant);
        assert_eq!(
            InMemoryNamespaceActionAuthorizer::failing_because("policy store unreachable")
                .authorize(&candidate),
            Err(NamespaceUsecaseError::cedar_evaluation_failed(
                "policy store unreachable"
            ))
        );
        assert!(
            InMemoryNamespaceActionAuthorizer::failing()
                .authorize(&candidate)
                .expect_err("a failing engine reaches no verdict")
                .is_port_failure()
        );
    }
}
