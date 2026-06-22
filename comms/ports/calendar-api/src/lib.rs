//! Calendar capability PORT (clean-arch boundary, owned-stack shape).
//!
//! Defines the trait seams the calendar usecase depends on and concrete
//! cloud/persistence/identity adapters implement LATER: a tenant-scoped event
//! store, a free/busy resolver, and the fail-closed authorization context that
//! EVERY calendar surface (HTTP/gRPC facade, worker) must carry before it
//! mutates or reads tenant data.
//!
//! The trait shapes model the W5 owned-stack destination, not any transient
//! infra (per `ports-designed-for-owned-stack`): adapters absorb Postgres /
//! Valkey / CalDAV. CalDAV stays OUT of this port and the kernel per ADR-0015 —
//! it is a protocol adapter, not a domain or port concern. No persistence,
//! cloud, or identity backend is pulled here; this crate is a pure leaf over
//! `comms-calendar-domain`.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use comms_calendar_domain::{Calendar, CalendarError, CalendarEvent, CalendarSlot};

/// Errors at the port boundary — distinct from `CalendarError` (the domain's
/// invariant failures) so the usecase can tell a denied/unauthenticated call
/// from an invalid aggregate from a backend failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CalendarApiError {
    /// The authorization context failed its fail-closed validation (missing
    /// principal / tenant scope / policy decision / audit correlation).
    Unauthorized(AuthzDenyReason),
    /// A request field was empty or malformed at the boundary.
    Invalid,
    /// The requested aggregate does not exist within the tenant scope.
    NotFound,
    /// A backend adapter (persistence / cache / cloud) failed; the concrete
    /// cause is logged out-of-band, never surfaced to the caller.
    Backend,
    /// A domain invariant was violated when constructing/validating the
    /// aggregate the store was asked to persist.
    Domain(CalendarError),
}

/// Why a calendar call was denied at the fail-closed authz boundary. Every
/// variant is a DENY — the context is default-deny and only `validate()`
/// returning `Ok` admits the call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthzDenyReason {
    MissingPrincipal,
    MissingTenantScope,
    MissingPolicyDecision,
    MissingAuditCorrelation,
    TenantMismatch,
}

/// The authorization context EVERY calendar surface must present before it
/// touches tenant data. Fail-closed by construction: the only admitting path is
/// [`AuthorizedCalendarContext::validate`] returning `Ok`. A facade that cannot
/// build a fully-populated context with a verified principal + a tenant scope +
/// a Cedar policy-decision ref + an audit correlation id CANNOT call the
/// usecase — there is no default/anonymous constructor (new-HTTP-surfaces
/// default-deny doctrine; mirrors `comms-mail-mailbox-api`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizedCalendarContext {
    /// The verified caller principal (e.g. `user:...`). Identity binding is the
    /// facade's job; this port only refuses to proceed without it.
    pub principal_ref: String,
    /// The tenant the call is scoped to (`tenant:...`). Cross-tenant reads/
    /// writes are refused by matching this against the aggregate's tenant.
    pub tenant_scope_ref: String,
    /// The Cedar PDP decision ref that ADMITTED this call. Absent => deny.
    pub policy_decision_ref: String,
    /// The audit correlation id threaded through to the event/audit plane.
    pub audit_correlation_id: String,
}

impl AuthorizedCalendarContext {
    /// Fail-closed admission check. Returns `Ok(())` ONLY when a verified
    /// principal, a `tenant:`-prefixed scope, a non-empty policy-decision ref,
    /// and an audit-correlation id are ALL present. Any gap is a DENY.
    pub fn validate(&self) -> Result<(), CalendarApiError> {
        if self.principal_ref.trim().is_empty() {
            return Err(CalendarApiError::Unauthorized(
                AuthzDenyReason::MissingPrincipal,
            ));
        }
        if !self.tenant_scope_ref.starts_with("tenant:") {
            return Err(CalendarApiError::Unauthorized(
                AuthzDenyReason::MissingTenantScope,
            ));
        }
        if self.policy_decision_ref.trim().is_empty() {
            return Err(CalendarApiError::Unauthorized(
                AuthzDenyReason::MissingPolicyDecision,
            ));
        }
        if self.audit_correlation_id.trim().is_empty() {
            return Err(CalendarApiError::Unauthorized(
                AuthzDenyReason::MissingAuditCorrelation,
            ));
        }
        Ok(())
    }

    /// The bare tenant id (the `tenant:` prefix stripped) for matching against an
    /// aggregate's `tenant_id`. Only meaningful AFTER [`Self::validate`] passes.
    pub fn tenant_id(&self) -> &str {
        self.tenant_scope_ref
            .strip_prefix("tenant:")
            .unwrap_or(&self.tenant_scope_ref)
    }
}

/// A window over which free/busy is resolved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FreeBusyWindow {
    pub earliest_start_epoch_seconds: u64,
    pub latest_end_epoch_seconds: u64,
    pub duration_seconds: u64,
}

/// Tenant-scoped persistence seam for calendar aggregates. The concrete Postgres
/// (RLS-enforced, fail-closed on tenant isolation) and any cache adapter live
/// BEHIND this trait, DEFERRED out of this slice. Every method takes a validated
/// `AuthorizedCalendarContext`; an implementation MUST refuse a write/read whose
/// aggregate tenant differs from the context tenant.
pub trait CalendarStore {
    fn put_calendar(
        &self,
        ctx: &AuthorizedCalendarContext,
        calendar: &Calendar,
    ) -> Result<(), CalendarApiError>;

    fn get_calendar(
        &self,
        ctx: &AuthorizedCalendarContext,
        calendar_id: &str,
    ) -> Result<Calendar, CalendarApiError>;

    fn put_event(
        &self,
        ctx: &AuthorizedCalendarContext,
        event: &CalendarEvent,
    ) -> Result<(), CalendarApiError>;

    fn list_events(
        &self,
        ctx: &AuthorizedCalendarContext,
        calendar_id: &str,
    ) -> Result<Vec<CalendarEvent>, CalendarApiError>;
}

/// Free/busy + slot-finding seam. The owned-stack availability resolver (and any
/// transient Valkey cache) implements this LATER; the usecase composes it for
/// scheduling. Mirrors the domain's `SlotPicker` at the port boundary so a facade
/// never depends on a concrete resolver.
pub trait FreeBusyResolver {
    fn find_slots(
        &self,
        ctx: &AuthorizedCalendarContext,
        attendee_emails: &[String],
        window: FreeBusyWindow,
    ) -> Result<Vec<CalendarSlot>, CalendarApiError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_ctx() -> AuthorizedCalendarContext {
        AuthorizedCalendarContext {
            principal_ref: "user:u".into(),
            tenant_scope_ref: "tenant:t".into(),
            policy_decision_ref: "cedar:allow:calendar".into(),
            audit_correlation_id: "audit-1".into(),
        }
    }

    #[test]
    fn context_admits_only_fully_populated_tenant_scoped_call() {
        assert_eq!(ok_ctx().validate(), Ok(()));
        assert_eq!(ok_ctx().tenant_id(), "t");
    }

    #[test]
    fn missing_principal_is_denied() {
        let mut ctx = ok_ctx();
        ctx.principal_ref = "   ".into();
        assert_eq!(
            ctx.validate(),
            Err(CalendarApiError::Unauthorized(
                AuthzDenyReason::MissingPrincipal
            ))
        );
    }

    #[test]
    fn non_tenant_scope_is_denied() {
        let mut ctx = ok_ctx();
        ctx.tenant_scope_ref = "person:u".into();
        assert_eq!(
            ctx.validate(),
            Err(CalendarApiError::Unauthorized(
                AuthzDenyReason::MissingTenantScope
            ))
        );
    }

    #[test]
    fn missing_policy_decision_is_denied() {
        let mut ctx = ok_ctx();
        ctx.policy_decision_ref = "".into();
        assert_eq!(
            ctx.validate(),
            Err(CalendarApiError::Unauthorized(
                AuthzDenyReason::MissingPolicyDecision
            ))
        );
    }

    #[test]
    fn missing_audit_correlation_is_denied() {
        let mut ctx = ok_ctx();
        ctx.audit_correlation_id = "".into();
        assert_eq!(
            ctx.validate(),
            Err(CalendarApiError::Unauthorized(
                AuthzDenyReason::MissingAuditCorrelation
            ))
        );
    }
}
