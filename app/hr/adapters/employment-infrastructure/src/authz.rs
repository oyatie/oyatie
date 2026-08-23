//! Fail-closed authorization seam for HR employment control-plane mutations.
//!
//! The runtime adapter handles employee onboarding, labor-compliance workflow
//! planning, sensitive HR read policy decisions, and leave/payroll-impact
//! handoffs. All four routes carry tenant-scoped PII, sensitive, or financial
//! metadata, so identity must be verified before the request body becomes
//! authority. The concrete credential verifier and PDP client are ports: a
//! cloud-iam/SPIFFE verifier or Cedar PDP adapter can replace the reference
//! bearer verifier without changing the handler contract.
// ADR-0083 Tier 3: production code stays panic-free.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::sync::Arc;

use http_middleware_kernel::{HttpRequest, HttpResponse, Middleware, Next};

pub const AUTHORIZATION_HEADER: &str = "authorization";
pub const VERIFIED_TENANT_CAPTURE_KEY: &str = "verified_tenant_id";

#[derive(Clone, Eq, PartialEq)]
pub struct CallerCredential {
    pub authorization: Option<String>, // data_class: SECRET
}

impl std::fmt::Debug for CallerCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallerCredential")
            .field("authorization", &"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedPrincipal {
    principal_id: String, // data_class: INTERNAL_ONLY
    tenant_id: String,    // data_class: INTERNAL_ONLY
}

impl VerifiedPrincipal {
    pub(crate) fn new(principal_id: impl Into<String>, tenant_id: impl Into<String>) -> Self {
        Self {
            principal_id: principal_id.into(),
            tenant_id: tenant_id.into(),
        }
    }

    #[must_use]
    pub fn principal_id(&self) -> &str {
        &self.principal_id
    }

    #[must_use]
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrincipalVerificationError {
    MissingCredential,
    InvalidCredential,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HrControlPlaneAuthorizationError {
    Denied,
    Refused,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HrControlPlaneAction {
    OnboardEmployee,
    LaborCompliancePlan,
    SensitiveReadPolicyDecision,
    LeavePayrollImpactPlan,
}

impl HrControlPlaneAction {
    #[must_use]
    pub const fn surface(self) -> &'static str {
        match self {
            Self::OnboardEmployee => "hr.employment.onboard",
            Self::LaborCompliancePlan => "hr.labor_compliance.plan",
            Self::SensitiveReadPolicyDecision => "hr.sensitive_read.policy_decision",
            Self::LeavePayrollImpactPlan => "hr.leave_payroll_impact.plan",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HrControlPlaneResource {
    pub tenant_id: String,            // data_class: INTERNAL_ONLY
    pub action: HrControlPlaneAction, // data_class: INTERNAL_ONLY
    pub route_template: String,       // data_class: INTERNAL_ONLY
}

pub trait PrincipalVerifier: Send + Sync {
    fn verify_principal(
        &self,
        credential: &CallerCredential,
    ) -> Result<VerifiedPrincipal, PrincipalVerificationError>;
}

/// PORT: decide whether `principal` may run the HR control-plane action.
///
/// Adapter contract:
/// 1. Map every PDP/internal fault to `Err(HrControlPlaneAuthorizationError::Refused)`.
/// 2. Enforce an adapter-local deadline before returning `Refused`.
/// 3. Never panic; production panic-abort profiles cannot rely on unwind recovery.
pub trait HrControlPlaneAuthorizer: Send + Sync {
    fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        resource: &HrControlPlaneResource,
    ) -> Result<(), HrControlPlaneAuthorizationError>;
}

#[derive(Clone)]
pub struct HrAuthzProvider {
    verifier: Arc<dyn PrincipalVerifier>, // data_class: INTERNAL_ONLY
    authorizer: Arc<dyn HrControlPlaneAuthorizer>, // data_class: INTERNAL_ONLY
}

impl HrAuthzProvider {
    #[must_use]
    pub fn new(
        verifier: Arc<dyn PrincipalVerifier>,
        authorizer: Arc<dyn HrControlPlaneAuthorizer>,
    ) -> Self {
        Self {
            verifier,
            authorizer,
        }
    }

    pub fn verify_principal(
        &self,
        credential: &CallerCredential,
    ) -> Result<VerifiedPrincipal, PrincipalVerificationError> {
        self.verifier.verify_principal(credential)
    }

    pub fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        resource: &HrControlPlaneResource,
    ) -> Result<(), HrControlPlaneAuthorizationError> {
        let authorizer = Arc::clone(&self.authorizer);
        let principal = principal.clone();
        let resource = resource.clone();
        // Best-effort test/debug backstop only. Production adapters must return
        // `Err(Refused)` for faults instead of panicking.
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            authorizer.ensure_authorized(&principal, &resource)
        }))
        .unwrap_or(Err(HrControlPlaneAuthorizationError::Refused))
    }
}

// ponytail: preview bearer verifier; replace with IAM/SPIFFE when the deployed listener exists.
pub struct ConfiguredBearerPrincipalVerifier {
    bearer_secret: String,      // data_class: SECRET
    bound_principal_id: String, // data_class: INTERNAL_ONLY
    bound_tenant_id: String,    // data_class: INTERNAL_ONLY
}

impl ConfiguredBearerPrincipalVerifier {
    pub fn new(
        bearer_secret: impl Into<String>,
        bound_principal_id: impl Into<String>,
        bound_tenant_id: impl Into<String>,
    ) -> Result<Self, AuthzProviderConfigError> {
        let bearer_secret = bearer_secret.into();
        let bound_principal_id = bound_principal_id.into();
        let bound_tenant_id = bound_tenant_id.into();
        if bearer_secret.trim().is_empty() {
            return Err(AuthzProviderConfigError::EmptyBearerSecret);
        }
        if bound_principal_id.trim().is_empty() || bound_tenant_id.trim().is_empty() {
            return Err(AuthzProviderConfigError::EmptyBoundIdentity);
        }
        Ok(Self {
            bearer_secret,
            bound_principal_id,
            bound_tenant_id,
        })
    }
}

impl PrincipalVerifier for ConfiguredBearerPrincipalVerifier {
    fn verify_principal(
        &self,
        credential: &CallerCredential,
    ) -> Result<VerifiedPrincipal, PrincipalVerificationError> {
        let Some(authorization) = credential.authorization.as_deref() else {
            return Err(PrincipalVerificationError::MissingCredential);
        };
        let Some(presented) = authorization.strip_prefix("Bearer ") else {
            return Err(PrincipalVerificationError::InvalidCredential);
        };
        if !constant_time_eq(presented.as_bytes(), self.bearer_secret.as_bytes()) {
            return Err(PrincipalVerificationError::InvalidCredential);
        }
        Ok(VerifiedPrincipal::new(
            self.bound_principal_id.clone(),
            self.bound_tenant_id.clone(),
        ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthzProviderConfigError {
    EmptyBearerSecret,
    EmptyBoundIdentity,
}

impl std::fmt::Display for AuthzProviderConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyBearerSecret => write!(f, "hr authz bearer secret must be non-empty"),
            Self::EmptyBoundIdentity => {
                write!(f, "hr authz bound principal/tenant must be non-empty")
            }
        }
    }
}

impl std::error::Error for AuthzProviderConfigError {}

#[must_use]
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let max_len = a.len().max(b.len());
    let mut diff = a.len() ^ b.len();
    for index in 0..max_len {
        let left = a.get(index).copied().unwrap_or(0);
        let right = b.get(index).copied().unwrap_or(0);
        diff |= (left ^ right) as usize;
    }
    diff == 0
}

#[must_use]
pub fn action_for_template(template: &str) -> Option<HrControlPlaneAction> {
    match template {
        crate::HR_EMPLOYEES_PATH => Some(HrControlPlaneAction::OnboardEmployee),
        crate::HR_LABOR_COMPLIANCE_WORKFLOW_PLANS_PATH => {
            Some(HrControlPlaneAction::LaborCompliancePlan)
        }
        crate::HR_SENSITIVE_READ_POLICY_DECISIONS_PATH => {
            Some(HrControlPlaneAction::SensitiveReadPolicyDecision)
        }
        crate::HR_LEAVE_PAYROLL_IMPACT_PLANS_PATH => {
            Some(HrControlPlaneAction::LeavePayrollImpactPlan)
        }
        _ => None,
    }
}

pub struct HrAuthzMiddleware {
    provider: HrAuthzProvider,
}

impl HrAuthzMiddleware {
    #[must_use]
    pub fn new(provider: HrAuthzProvider) -> Self {
        Self { provider }
    }

    fn unauthorized_401() -> HttpResponse {
        HttpResponse::new(401)
            .with_header("content-type", "application/json")
            .with_header("www-authenticate", "Bearer")
            .with_body(
                br#"{"error":{"code":"UNAUTHENTICATED","message":"hr control-plane mutation requires a verified principal"}}"#
                    .to_vec(),
            )
    }

    fn forbidden_403() -> HttpResponse {
        HttpResponse::new(403)
            .with_header("content-type", "application/json")
            .with_body(
                br#"{"error":{"code":"FORBIDDEN","message":"principal not authorized for this hr control-plane mutation"}}"#
                    .to_vec(),
            )
    }

    fn authorization_header(request: &HttpRequest) -> Option<String> {
        request
            .headers
            .get(AUTHORIZATION_HEADER)
            .cloned()
            .or_else(|| {
                request
                    .headers
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case(AUTHORIZATION_HEADER))
                    .map(|(_, value)| value.clone())
            })
    }
}

impl Middleware<HttpRequest, HttpResponse> for HrAuthzMiddleware {
    fn handle(
        &self,
        mut request: HttpRequest,
        next: Next<'_, HttpRequest, HttpResponse>,
    ) -> HttpResponse {
        let template = request
            .matched_template
            .clone()
            .unwrap_or_else(|| request.path.clone());
        let Some(action) = action_for_template(&template) else {
            return next.run(request);
        };

        let credential = CallerCredential {
            authorization: Self::authorization_header(&request),
        };
        let principal = match self.provider.verify_principal(&credential) {
            Ok(principal) => principal,
            Err(_) => return Self::unauthorized_401(),
        };

        let resource = HrControlPlaneResource {
            tenant_id: principal.tenant_id().to_owned(),
            action,
            route_template: template,
        };
        if self
            .provider
            .ensure_authorized(&principal, &resource)
            .is_err()
        {
            return Self::forbidden_403();
        }

        request.path_captures.insert(
            VERIFIED_TENANT_CAPTURE_KEY.to_owned(),
            principal.tenant_id().to_owned(),
        );
        next.run(request)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AuthzProviderConfigError, ConfiguredBearerPrincipalVerifier, HrControlPlaneAction,
        PrincipalVerificationError, PrincipalVerifier, action_for_template, constant_time_eq,
    };

    const SECRET: &str = "hr-break-glass";

    fn credential(authorization: Option<&str>) -> super::CallerCredential {
        super::CallerCredential {
            authorization: authorization.map(str::to_string),
        }
    }

    #[test]
    fn verifier_refuses_empty_secret_or_identity() {
        assert!(matches!(
            ConfiguredBearerPrincipalVerifier::new(" ", "sp_hr", "ten_acme"),
            Err(AuthzProviderConfigError::EmptyBearerSecret)
        ));
        assert!(matches!(
            ConfiguredBearerPrincipalVerifier::new(SECRET, "sp_hr", ""),
            Err(AuthzProviderConfigError::EmptyBoundIdentity)
        ));
    }

    #[test]
    fn verifier_binds_identity_from_config_not_request_body() {
        let verifier =
            ConfiguredBearerPrincipalVerifier::new(SECRET, "sp_hr", "ten_acme").expect("verifier");
        let principal = verifier
            .verify_principal(&credential(Some(&format!("Bearer {SECRET}"))))
            .expect("principal");
        assert_eq!(principal.principal_id(), "sp_hr");
        assert_eq!(principal.tenant_id(), "ten_acme");
    }

    #[test]
    fn verifier_rejects_missing_and_wrong_credentials() {
        let verifier =
            ConfiguredBearerPrincipalVerifier::new(SECRET, "sp_hr", "ten_acme").expect("verifier");
        assert_eq!(
            verifier.verify_principal(&credential(None)).unwrap_err(),
            PrincipalVerificationError::MissingCredential
        );
        assert_eq!(
            verifier
                .verify_principal(&credential(Some("Bearer wrong")))
                .unwrap_err(),
            PrincipalVerificationError::InvalidCredential
        );
        assert_eq!(
            verifier
                .verify_principal(&credential(Some("Basic xyz")))
                .unwrap_err(),
            PrincipalVerificationError::InvalidCredential
        );
    }

    #[test]
    fn route_templates_map_only_mutating_hr_routes() {
        assert_eq!(
            action_for_template(crate::HR_EMPLOYEES_PATH),
            Some(HrControlPlaneAction::OnboardEmployee)
        );
        assert_eq!(
            action_for_template(crate::HR_LABOR_COMPLIANCE_WORKFLOW_PLANS_PATH),
            Some(HrControlPlaneAction::LaborCompliancePlan)
        );
        assert_eq!(
            action_for_template(crate::HR_SENSITIVE_READ_POLICY_DECISIONS_PATH),
            Some(HrControlPlaneAction::SensitiveReadPolicyDecision)
        );
        assert_eq!(
            action_for_template(crate::HR_LEAVE_PAYROLL_IMPACT_PLANS_PATH),
            Some(HrControlPlaneAction::LeavePayrollImpactPlan)
        );
        assert_eq!(action_for_template(crate::HR_HEALTH_PATH), None);
        assert_eq!(action_for_template("/hr/v1/unknown"), None);
    }

    #[test]
    fn action_surfaces_are_stable() {
        assert_eq!(
            HrControlPlaneAction::OnboardEmployee.surface(),
            "hr.employment.onboard"
        );
        assert_eq!(
            HrControlPlaneAction::SensitiveReadPolicyDecision.surface(),
            "hr.sensitive_read.policy_decision"
        );
    }

    #[test]
    fn constant_time_eq_matches_only_identical_inputs() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
        assert!(!constant_time_eq(b"", b"a"));
        assert!(constant_time_eq(b"", b""));
    }
}
