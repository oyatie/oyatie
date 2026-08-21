//! Entities, ports and the single error type of the reserved-namespace
//! guard. No decision logic lives here — see [`crate::domain`].

/// Decision returned by the guard for one candidate name.
///
/// The variants are ordered by the stage that produced them, and the
/// evaluation order is fixed: syntax, then reservation, then confusability,
/// then authorization. Every variant is reachable and each one means
/// something different to an operator reading an audit trail:
///
/// - [`NamespaceDecision::DenyMalformed`] — the caller sent a name that is
///   not a legal label at all. Nothing about the reservation list was
///   consulted, so this decision is stable across policy changes.
/// - [`NamespaceDecision::DenyReserved`] — the name IS a platform-owned
///   namespace once separators and case are normalized away.
/// - [`NamespaceDecision::DenyConfusable`] — the name is not the reserved
///   name, but folds onto it under the bounded ASCII skeleton in
///   [`crate::domain::skeleton`].
/// - [`NamespaceDecision::DenyUnauthorized`] — the name is fine; this
///   principal may not perform this action in this tenant.
/// - [`NamespaceDecision::Allow`] — passed every check.
///
/// A source outage is NOT a decision: it is
/// [`NamespaceUsecaseError::SourceUnavailable`], so an operator can always
/// tell "we refused you" from "we could not read the policy".
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NamespaceDecision {
    /// Every check passed and the authorizer permitted the action.
    Allow,
    /// The candidate collides with the platform-owner reservation list.
    DenyReserved,
    /// The candidate folds onto a reserved name under the ASCII skeleton.
    DenyConfusable,
    /// The candidate is not a syntactically legal namespace label.
    DenyMalformed,
    /// The name is acceptable but the principal may not take this action.
    ///
    /// Added to the scaffold's four variants rather than folded into one of
    /// them: "this name is reserved" and "you may not create tenants" are
    /// different facts, they are remediated differently, and collapsing
    /// them would tell a legitimate operator to rename when the real answer
    /// is to obtain the grant.
    DenyUnauthorized,
}

impl NamespaceDecision {
    /// Whether this decision permits the requested action.
    #[must_use]
    pub fn is_allowed(self) -> bool {
        matches!(self, Self::Allow)
    }

    /// Stable audit label, safe to emit into an event stream.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::DenyReserved => "deny_reserved",
            Self::DenyConfusable => "deny_confusable",
            Self::DenyMalformed => "deny_malformed",
            Self::DenyUnauthorized => "deny_unauthorized",
        }
    }
}

impl core::fmt::Display for NamespaceDecision {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.label())
    }
}

/// Inputs evaluated by the guard.
///
/// IP-017 §D.4 requires both entry points to carry a tenant context and an
/// audit correlation id alongside the actor. Both are fields here rather
/// than a separate parameter, so that everything the authorizer needs to
/// answer a RESOURCE-scoped question — not merely a principal/action one —
/// travels in the single value handed to
/// [`NamespaceActionAuthorizer::authorize`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamespaceCandidate {
    /// The tenant-supplied name being claimed, exactly as received. It is
    /// never normalized in place: normalization happens for comparison
    /// only, so the audit record can say what was actually asked for.
    pub candidate: String, // data_class: TENANT_PAYLOAD
    /// The principal requesting the action, e.g. `tenant.<id>.admin`.
    pub principal: String, // data_class: TENANT_SCOPED
    /// Which namespace-claiming action is being attempted.
    pub action: NamespaceAction, // data_class: INTERNAL_ONLY
    /// The tenant the claimed name would live under — IP-017 §D.4's
    /// `TenantContext`.
    ///
    /// `None` is legal ONLY for [`NamespaceAction::CreateTenant`], where the
    /// tenant does not exist yet; see
    /// [`NamespaceAction::requires_tenant_context`]. For a rename or a
    /// sub-scope alias the guard refuses to answer without it
    /// ([`NamespaceUsecaseError::TenantContextMissing`]), because a
    /// principal holding a grant in one tenant must not be able to mint a
    /// name in another simply by omitting the field.
    pub tenant: Option<String>, // data_class: TENANT_SCOPED
    /// IP-017 §D.4's `AuditCorrelationId`, echoed into
    /// [`crate::usecase::NamespaceEvaluation`] so a refusal event joins back
    /// to the request that caused it.
    ///
    /// Supplied by the caller and never generated here: minting one would
    /// need randomness, which the crate's determinism rule forbids below the
    /// port boundary.
    pub correlation_id: Option<String>, // data_class: INTERNAL_ONLY
}

impl NamespaceCandidate {
    /// Assemble a candidate from string-like parts, with no tenant context
    /// and no correlation id.
    ///
    /// Use [`NamespaceCandidate::in_tenant`] for anything other than
    /// [`NamespaceAction::CreateTenant`]; the other two actions are refused
    /// with [`NamespaceUsecaseError::TenantContextMissing`] until a tenant
    /// is attached.
    #[must_use]
    pub fn new(
        candidate: impl Into<String>,
        principal: impl Into<String>,
        action: NamespaceAction,
    ) -> Self {
        Self {
            candidate: candidate.into(),
            principal: principal.into(),
            action,
            tenant: None,
            correlation_id: None,
        }
    }

    /// Attach the tenant this claim is scoped to.
    #[must_use]
    pub fn in_tenant(mut self, tenant: impl Into<String>) -> Self {
        self.tenant = Some(tenant.into());
        self
    }

    /// Attach the audit correlation id of the originating request.
    #[must_use]
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// The tenant context, treating blank as absent.
    #[must_use]
    pub fn tenant_context(&self) -> Option<&str> {
        self.tenant
            .as_deref()
            .map(str::trim)
            .filter(|tenant| !tenant.is_empty())
    }
}

/// The namespace-claiming actions this guard gates.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NamespaceAction {
    /// A new tenant slug is being minted.
    CreateTenant,
    /// An existing tenant is taking a different slug. Gated identically to
    /// creation: a rename is a fresh claim on a name, and exempting it
    /// would leave a trivial two-step bypass of tenant creation.
    RenameTenant,
    /// An alias is being minted below an existing tenant.
    CreateSubScope,
}

impl NamespaceAction {
    /// Stable audit label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::CreateTenant => "create_tenant",
            Self::RenameTenant => "rename_tenant",
            Self::CreateSubScope => "create_sub_scope",
        }
    }

    /// Shortest legal label for this action, counted on the NORMALIZED
    /// form.
    ///
    /// Tenant slugs are human-visible and globally unique, so two-character
    /// slugs are refused: they exhaust fast, they read as abbreviations of
    /// something else, and short strings are the easiest to confuse. A
    /// sub-scope alias is already qualified by its owning tenant, so it
    /// carries less impersonation weight and is allowed one character less.
    ///
    /// The count is taken after [`crate::domain::normalize`] precisely so
    /// that `a-b` cannot buy the two-character identity `ab` that this rule
    /// exists to refuse. [`MAX_LABEL_LEN`] is the mirror image and is
    /// counted on the RAW form, because that is the string that has to fit
    /// in a DNS label.
    #[must_use]
    pub fn min_label_len(self) -> usize {
        match self {
            Self::CreateTenant | Self::RenameTenant => 3,
            Self::CreateSubScope => 2,
        }
    }

    /// Whether this action can only be evaluated inside a tenant context.
    ///
    /// A rename targets an existing tenant and a sub-scope alias is minted
    /// below one, so both name a resource that already has an owner and the
    /// authorizer cannot answer without it. Tenant creation is the one case
    /// where the resource does not exist yet, so its tenant context is
    /// optional; see the crate Gaps note.
    #[must_use]
    pub fn requires_tenant_context(self) -> bool {
        match self {
            Self::CreateTenant => false,
            Self::RenameTenant | Self::CreateSubScope => true,
        }
    }
}

impl core::fmt::Display for NamespaceAction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.label())
    }
}

/// Longest legal label, in bytes.
///
/// 63 is the DNS label ceiling (RFC 1035 §2.3.4). Tenant slugs surface in
/// hostnames, so a slug that cannot be a DNS label is refused at the mint
/// point rather than discovered at first ingress.
pub const MAX_LABEL_LEN: usize = 63;

/// Sealed port reading the platform-owner reservation list.
///
/// The entries come from `/specs/platform-owner-binding.json` plus the
/// principal roster in `tenancy/ARCHITECTURE.md`. Per ADR-0284 the owner
/// name is NEVER hard-coded here: substituting `oyatie` for another owner
/// is a change to the source, not to this crate. Per ADR-0242 the owner is
/// a tenant like any other, so there is no carve-out anywhere in this crate
/// that lets the owner name through.
pub trait ReservedNamespaceSource {
    /// The reservation list, or the reason it could not be read. This is
    /// the ONLY method any decision path calls.
    ///
    /// Entries may be bare tokens (`oyatie`) or dotted principal paths
    /// (`oyatie.tenancy.lifecycle-controller`).
    ///
    /// # Errors
    ///
    /// Reporting an outage as `Ok(vec![])` is the one thing an
    /// implementation must not do, because an empty list read literally
    /// permits every name including the platform owner's own. Return
    /// [`NamespaceUsecaseError::SourceUnavailable`] with a cause instead.
    /// This method is required, rather than defaulted onto the infallible
    /// view below, exactly so that an adapter cannot satisfy the port
    /// without deciding what its outage looks like.
    fn try_reserved(&self) -> Result<Vec<String>, NamespaceUsecaseError>;

    /// Best-effort infallible view, for diagnostics and for the scaffold's
    /// original single-method shape.
    ///
    /// An unreachable source yields an empty vec here, which is precisely
    /// why this must never feed [`crate::domain::ReservedSet::build`] on a
    /// decision path: the build would report
    /// [`NamespaceUsecaseError::EmptyReservationList`] ("the owner binding
    /// did not resolve") for what is really
    /// [`NamespaceUsecaseError::SourceUnavailable`] ("the source is
    /// unreachable"), and those two have opposite remediations. Call
    /// [`ReservedNamespaceSource::try_reserved`] instead.
    fn reserved(&self) -> Vec<String> {
        self.try_reserved().unwrap_or_default()
    }
}

/// Sealed port evaluating Cedar action-authorization.
pub trait NamespaceActionAuthorizer {
    /// Whether `input.principal` may perform `input.action` in
    /// `input.tenant`.
    ///
    /// The whole [`NamespaceCandidate`] is passed, not a
    /// `(principal, action)` pair, so a real Cedar adapter can express a
    /// resource-scoped rule: [`NamespaceCandidate::tenant_context`] is the
    /// resource's owning tenant and is guaranteed present by the time this
    /// is called for any action where
    /// [`NamespaceAction::requires_tenant_context`] holds.
    ///
    /// # Errors
    ///
    /// `Ok(false)` is a policy decision and becomes
    /// [`NamespaceDecision::DenyUnauthorized`]. An engine that could not
    /// reach a verdict returns
    /// [`NamespaceUsecaseError::CedarEvaluationFailed`] instead — never
    /// `Ok(false)`, which would make an outage indistinguishable from a
    /// deliberate deny.
    fn authorize(&self, input: &NamespaceCandidate) -> Result<bool, NamespaceUsecaseError>;
}

/// A failure that prevented the guard from reaching a decision.
///
/// Every variant means "no verdict was produced", never "denied". Callers
/// must fail closed on all of them; what they must NOT do is report them as
/// a reservation hit, because the remediation is completely different.
///
/// The two port-failure variants carry a `cause`, so an operator paging on
/// a total tenant-creation outage can tell a missing binding file from a
/// parse error from a resolver timeout without reading adapter logs this
/// crate's contract does not require to exist.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NamespaceUsecaseError {
    /// The reservation source could not be read. `cause` is the adapter's
    /// own description of the failure and is INTERNAL_ONLY: it may name a
    /// path or an io error and must not be echoed to a tenant.
    SourceUnavailable { cause: String },
    /// The authorization engine could not reach a verdict. `cause` carries
    /// the engine's description, on the same terms.
    CedarEvaluationFailed { cause: String },
    /// The source returned successfully with no entries at all.
    ///
    /// A guard with an empty reservation list permits every name including
    /// the platform owner's own, which is precisely the state ADR-0242
    /// forbids. Since a correctly resolved owner binding always yields at
    /// least the owner token, an empty list means the binding did not
    /// resolve — treated as no-verdict rather than as "nothing is
    /// reserved".
    EmptyReservationList,
    /// A reservation entry normalized to nothing (blank, or separators
    /// only). Refused rather than skipped: silently dropping a broken
    /// policy row shrinks the guard by exactly the row an attacker would
    /// most like to see dropped.
    MalformedReservationEntry { entry: String },
    /// The candidate carried no principal, so no authorization question
    /// could be asked. A caller bug, not a tenant-visible policy outcome.
    PrincipalMissing,
    /// The candidate carried no tenant context for an action that is
    /// meaningless without one. A caller bug — and the fail-closed half of
    /// tenant scoping, since answering anyway would let a grant held in one
    /// tenant mint a name in every other.
    TenantContextMissing { action: NamespaceAction },
}

impl NamespaceUsecaseError {
    /// A [`NamespaceUsecaseError::SourceUnavailable`] carrying `cause`.
    #[must_use]
    pub fn source_unavailable(cause: impl Into<String>) -> Self {
        Self::SourceUnavailable {
            cause: cause.into(),
        }
    }

    /// A [`NamespaceUsecaseError::CedarEvaluationFailed`] carrying `cause`.
    #[must_use]
    pub fn cedar_evaluation_failed(cause: impl Into<String>) -> Self {
        Self::CedarEvaluationFailed {
            cause: cause.into(),
        }
    }

    /// Whether this failure came from a port rather than from the shape of
    /// the request or of the policy data.
    #[must_use]
    pub fn is_port_failure(&self) -> bool {
        matches!(
            self,
            Self::SourceUnavailable { .. } | Self::CedarEvaluationFailed { .. }
        )
    }
}

impl core::fmt::Display for NamespaceUsecaseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SourceUnavailable { cause } => write!(
                f,
                "reserved-namespace source is unavailable ({cause}); no verdict was reached"
            ),
            Self::CedarEvaluationFailed { cause } => write!(
                f,
                "action-authorization evaluation failed ({cause}); no verdict was reached"
            ),
            Self::EmptyReservationList => f.write_str(
                "reserved-namespace source returned an empty reservation list; \
                 the platform-owner binding did not resolve",
            ),
            Self::MalformedReservationEntry { entry } => write!(
                f,
                "reservation entry {entry:?} normalizes to an empty token"
            ),
            Self::PrincipalMissing => {
                f.write_str("candidate carries no principal; authorization cannot be evaluated")
            }
            Self::TenantContextMissing { action } => write!(
                f,
                "candidate carries no tenant context; {action} cannot be evaluated without one"
            ),
        }
    }
}

impl std::error::Error for NamespaceUsecaseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sub_scope_aliases_may_be_one_byte_shorter_than_tenant_slugs() {
        assert_eq!(NamespaceAction::CreateTenant.min_label_len(), 3);
        assert_eq!(NamespaceAction::RenameTenant.min_label_len(), 3);
        assert_eq!(NamespaceAction::CreateSubScope.min_label_len(), 2);
    }

    #[test]
    fn only_allow_is_an_allowed_decision() {
        assert!(NamespaceDecision::Allow.is_allowed());
        for denial in [
            NamespaceDecision::DenyReserved,
            NamespaceDecision::DenyConfusable,
            NamespaceDecision::DenyMalformed,
            NamespaceDecision::DenyUnauthorized,
        ] {
            assert!(!denial.is_allowed(), "{denial} must not be allowed");
        }
    }

    #[test]
    fn error_display_distinguishes_outage_from_policy() {
        let outage = NamespaceUsecaseError::source_unavailable("binding file missing").to_string();
        let empty = NamespaceUsecaseError::EmptyReservationList.to_string();
        assert_ne!(outage, empty);
        assert!(outage.contains("unavailable"), "{outage}");
        assert!(empty.contains("empty reservation list"), "{empty}");
        assert!(
            NamespaceUsecaseError::MalformedReservationEntry {
                entry: "--".to_owned(),
            }
            .to_string()
            .contains("\"--\"")
        );
    }

    #[test]
    fn a_port_failure_names_its_cause_and_two_causes_are_not_equal() {
        let missing = NamespaceUsecaseError::source_unavailable("/specs/binding.json: not found");
        let timeout = NamespaceUsecaseError::source_unavailable("resolver timed out after 2s");
        assert_ne!(missing, timeout);
        assert!(
            missing
                .to_string()
                .contains("/specs/binding.json: not found")
        );
        assert!(timeout.to_string().contains("resolver timed out"));
        assert!(missing.is_port_failure());
        assert!(!NamespaceUsecaseError::PrincipalMissing.is_port_failure());
    }

    #[test]
    fn only_tenant_creation_may_omit_a_tenant_context() {
        assert!(!NamespaceAction::CreateTenant.requires_tenant_context());
        assert!(NamespaceAction::RenameTenant.requires_tenant_context());
        assert!(NamespaceAction::CreateSubScope.requires_tenant_context());
    }

    #[test]
    fn a_blank_tenant_is_the_same_as_no_tenant() {
        let bare =
            NamespaceCandidate::new("acme", "tenant.acme.admin", NamespaceAction::CreateTenant);
        assert_eq!(bare.tenant_context(), None);
        assert_eq!(bare.clone().in_tenant("   ").tenant_context(), None);
        assert_eq!(
            bare.clone().in_tenant(" acme ").tenant_context(),
            Some("acme")
        );
        assert_eq!(
            bare.with_correlation_id("req-7").correlation_id,
            Some("req-7".to_owned())
        );
    }
}
