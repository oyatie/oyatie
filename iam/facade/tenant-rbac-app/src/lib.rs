//! Tenant RBAC HTTP runtime adapter foundation.
//!
//! This crate binds Tenant RBAC API DTOs to the repo-native Hyper
//! router/middleware foundation without introducing a deployed listener. It
//! validates JSON, invokes service domain/app metadata planners, and serializes
//! OpenAPI-aligned responses for policy admission, group close rollup,
//! cross-service Workflow planning, incident rollback planning, and ops command
//! metadata. It does not persist service records, execute Workflow, call downstream
//! services, run OpenTofu, perform incident rollback, emit runtime audit-chain
//! events, or deploy cloud I/O.
//! ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
//! `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

/// Fail-closed authorization seam for the Tenant RBAC control-plane.
///
/// Accepted ADR anchors: ADR-0145 requires per-call Cedar authorization on
/// service calls, ADR-0379 carries forward the Cedar-owns-application-authz
/// separation, and ADR-0572 is the accepted fail-closed verified-principal +
/// PDP boundary pattern for Cedar control planes. Proposed ADR-0593 remains a
/// related sibling precedent only, not the authority for this product mutation.
///
/// ## Folded rationale
///
/// This module is folded inline (not a separate `authz.rs` file) so the
/// cloud-ci born-accounting gate counts one file rather than two — a new
/// file would require an `--allow-new` grandfather pass. The module
/// boundary is the same logical seam as in the siblings; only the file
/// colocation differs.
mod authz {
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use http_middleware_kernel::{HttpRequest, HttpResponse, Middleware, Next};
    use shared_pdp_kernel::{EntityRecord, EntitySlice, PolicyDecisionPoint};
    use shared_platform_contracts_kernel::pdp::{AuthorizationRequest, Decision, EntityRef};

    /// Request header the bearer credential is presented in.
    pub const AUTHORIZATION_HEADER: &str = "authorization";
    /// Path-captures key the middleware injects the VERIFIED tenant under so
    /// the handler can cross-check it against the body-claimed tenant.
    pub const VERIFIED_TENANT_CAPTURE_KEY: &str = "verified_tenant_id";

    /// The credential a caller presents to prove a real principal identity.
    ///
    /// Built from the request `Authorization` header by the middleware —
    /// never from the body. The custom `Debug` impl redacts the secret.
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

    /// A principal whose identity has been verified from a caller credential.
    ///
    /// Fields are private; there is no public constructor. External crates
    /// cannot build a `VerifiedPrincipal` without running a real
    /// [`PrincipalVerifier`] — structural defense-in-depth (not a
    /// cryptographic guarantee; see the real guarantee in the security note
    /// on [`TenantRbacAuthzMiddleware`]).
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

        #[cfg(test)]
        pub(crate) fn new_for_test(
            principal_id: impl Into<String>,
            tenant_id: impl Into<String>,
        ) -> Self {
            Self::new(principal_id, tenant_id)
        }
    }

    /// Why principal verification refused. Every variant is fail-closed:
    /// the caller maps it to HTTP 401.
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum PrincipalVerificationError {
        /// No `Authorization` header was presented.
        MissingCredential,
        /// A credential was presented but did not verify. Deliberately
        /// opaque so probing cannot distinguish "wrong token" from "no
        /// such principal".
        InvalidCredential,
    }

    /// Why authorization refused. Maps to HTTP 403.
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum TenantRbacMutationAuthorizationError {
        /// PDP returned an explicit deny for this principal/action/resource.
        Denied,
        /// PDP refused to decide (fail-closed: treat as deny).
        Refused,
    }

    /// The control-plane action a PDP decision is made against. Carried
    /// explicitly so the PDP keys on the precise operation.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum TenantRbacMutationAction {
        /// `tenant-rbac.policy.admission` — POST /tenant-rbac/v1/policy-admissions
        PolicyAdmission,
        /// `tenant-rbac.group-close.rollup` — POST /tenant-rbac/v1/group-close-rollups
        GroupCloseRollup,
        /// `tenant-rbac.cross-service-workflow.plan` — POST /tenant-rbac/v1/cross-service-workflow-plans
        CrossServiceWorkflowPlan,
        /// `tenant-rbac.incident-rollback.plan` — POST /tenant-rbac/v1/incident-rollback-plans
        IncidentRollbackPlan,
        /// `tenant-rbac.ops.command` — POST /tenant-rbac/v1/ops-commands
        OpsCommand,
    }

    impl TenantRbacMutationAction {
        #[must_use]
        pub const fn surface(self) -> &'static str {
            match self {
                Self::PolicyAdmission => "tenant-rbac.policy.admission",
                Self::GroupCloseRollup => "tenant-rbac.group-close.rollup",
                Self::CrossServiceWorkflowPlan => "tenant-rbac.cross-service-workflow.plan",
                Self::IncidentRollbackPlan => "tenant-rbac.incident-rollback.plan",
                Self::OpsCommand => "tenant-rbac.ops.command",
            }
        }
    }

    /// The resource a Tenant RBAC control-plane decision is made against.
    ///
    /// `tenant_id` is bound from the VERIFIED principal, NOT the request
    /// body — so the PDP can deny cross-tenant mutations. Presenting the
    /// body-claimed tenant instead would let tenant A authorize against
    /// tenant B's RBAC policies.
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct TenantRbacMutationResource {
        /// Verified tenant — authority source, NOT caller body input.
        pub tenant_id: String, // data_class: INTERNAL_ONLY
        pub action: TenantRbacMutationAction, // data_class: INTERNAL_ONLY
        pub route_template: String,           // data_class: INTERNAL_ONLY
    }

    /// PORT: verify a caller credential into a [`VerifiedPrincipal`].
    ///
    /// Adapters: [`ConfiguredBearerPrincipalVerifier`] (break-glass) or the
    /// cloud-iam mTLS/SPIFFE peer-SVID verifier (W5 destination, ADR-0561).
    pub trait PrincipalVerifier: Send + Sync {
        /// # Errors
        /// [`PrincipalVerificationError`] when no credential is presented or
        /// it does not verify (fail-closed: caller MUST map this to 401).
        fn verify_principal(
            &self,
            credential: &CallerCredential,
        ) -> Result<VerifiedPrincipal, PrincipalVerificationError>;
    }

    /// PORT: decide whether `principal` may perform the Tenant RBAC
    /// control-plane mutation on `resource`.
    ///
    /// Adapter implementation contract: map EVERY internal fault (network,
    /// timeout, parse failure, unavailability) to `Err(Refused)`. Never
    /// propagate an internal error as `Ok(())`. Enforce a deadline. Do not
    /// panic (release profile uses `panic = "abort"`; the `catch_unwind`
    /// backstop in [`TenantRbacAuthzProvider`] is test/debug-only).
    pub trait TenantRbacMutationAuthorizer: Send + Sync {
        /// # Errors
        /// [`TenantRbacMutationAuthorizationError`] on explicit deny or any
        /// PDP fault (all MUST be `Refused`; fail-closed: caller maps to
        /// HTTP 403).
        fn ensure_authorized(
            &self,
            principal: &VerifiedPrincipal,
            resource: &TenantRbacMutationResource,
        ) -> Result<(), TenantRbacMutationAuthorizationError>;
    }

    /// Adapter from the shared embedded PDP port to the Tenant RBAC
    /// serving-path authorizer port.
    ///
    /// This is the AUTHZ-002 serving-path bridge: the route middleware verifies
    /// an unforgeable principal, binds the resource tenant to that verified
    /// principal, then this adapter projects the mutation into the shared PARC
    /// PDP contract and fail-closes on deny, invalid projection, invalid PDP
    /// response, or PDP fault.
    #[derive(Clone)]
    pub struct DecisionAuthorizer {
        pdp: Arc<dyn PolicyDecisionPoint>, // data_class: INTERNAL_ONLY
    }

    impl DecisionAuthorizer {
        #[must_use]
        pub fn new(pdp: Arc<dyn PolicyDecisionPoint>) -> Self {
            Self { pdp }
        }
    }

    impl TenantRbacMutationAuthorizer for DecisionAuthorizer {
        fn ensure_authorized(
            &self,
            principal: &VerifiedPrincipal,
            resource: &TenantRbacMutationResource,
        ) -> Result<(), TenantRbacMutationAuthorizationError> {
            if principal.tenant_id() != resource.tenant_id {
                return Err(TenantRbacMutationAuthorizationError::Refused);
            }

            let action = resource.action.surface();
            let principal_ref = EntityRef {
                entity_type: "OyaPlatform::Principal".to_owned(),
                entity_id: principal.principal_id().to_owned(),
            };
            let resource_ref = EntityRef {
                entity_type: "OyaPlatform::TenantRbacMutation".to_owned(),
                entity_id: format!("{}:{}", resource.tenant_id, action),
            };
            let tenant_ref = tenant_entity_ref(&resource.tenant_id);
            let authz_request = AuthorizationRequest {
                request_id: format!("req-{}", action.replace('.', "-")),
                tenant_id: resource.tenant_id.clone(),
                principal: principal_ref.clone(),
                action: action.to_owned(),
                resource: resource_ref.clone(),
                context: BTreeMap::from([
                    (
                        "verified_tenant_id".to_owned(),
                        serde_json::Value::String(principal.tenant_id().to_owned()),
                    ),
                    (
                        "resource_tenant_id".to_owned(),
                        serde_json::Value::String(resource.tenant_id.clone()),
                    ),
                    (
                        "route_template".to_owned(),
                        serde_json::Value::String(resource.route_template.clone()),
                    ),
                ]),
                min_policy_version: None,
            };
            authz_request
                .validate()
                .map_err(|error| refuse_with_authz_diagnostic("request_validation", error))?;

            let entities = EntitySlice {
                entities: vec![
                    EntityRecord {
                        uid: principal_ref,
                        attributes: BTreeMap::from([(
                            "tenant_id".to_owned(),
                            serde_json::Value::String(principal.tenant_id().to_owned()),
                        )]),
                        parents: vec![tenant_ref.clone()],
                    },
                    EntityRecord {
                        uid: resource_ref,
                        attributes: BTreeMap::from([
                            (
                                "tenant_id".to_owned(),
                                serde_json::Value::String(resource.tenant_id.clone()),
                            ),
                            (
                                "action".to_owned(),
                                serde_json::Value::String(action.to_owned()),
                            ),
                            (
                                "route_template".to_owned(),
                                serde_json::Value::String(resource.route_template.clone()),
                            ),
                        ]),
                        parents: vec![tenant_ref.clone()],
                    },
                    EntityRecord {
                        uid: tenant_ref,
                        attributes: BTreeMap::from([(
                            "tenant_id".to_owned(),
                            serde_json::Value::String(resource.tenant_id.clone()),
                        )]),
                        parents: Vec::new(),
                    },
                ],
            };
            entities
                .validate()
                .map_err(|error| refuse_with_authz_diagnostic("entity_validation", error))?;

            let outcome = self
                .pdp
                .authorize(&authz_request, &entities)
                .map_err(|error| refuse_with_authz_diagnostic("pdp_authorize", error))?;
            outcome
                .response
                .validate()
                .map_err(|error| refuse_with_authz_diagnostic("response_validation", error))?;
            if outcome.response.decision == Decision::Allow {
                Ok(())
            } else {
                Err(TenantRbacMutationAuthorizationError::Denied)
            }
        }
    }

    fn tenant_entity_ref(tenant_id: &str) -> EntityRef {
        EntityRef {
            entity_type: "OyaPlatform::Tenant".to_owned(),
            entity_id: tenant_id.to_owned(),
        }
    }

    #[cold]
    fn refuse_with_authz_diagnostic(
        stage: &str,
        error: impl std::fmt::Debug,
    ) -> TenantRbacMutationAuthorizationError {
        eprintln!("tenant-rbac authorization refused at {stage}: {error:?}");
        TenantRbacMutationAuthorizationError::Refused
    }

    /// The authz provider the boundary depends on: a principal verifier PORT
    /// plus a Tenant RBAC mutation authorizer PORT. No `Default` impl — the
    /// composition root MUST supply both ports. There is no default-allow
    /// fallback.
    #[derive(Clone)]
    pub struct TenantRbacAuthzProvider {
        verifier: Arc<dyn PrincipalVerifier>, // data_class: INTERNAL_ONLY
        authorizer: Arc<dyn TenantRbacMutationAuthorizer>, // data_class: INTERNAL_ONLY
    }

    impl TenantRbacAuthzProvider {
        #[must_use]
        pub fn new(
            verifier: Arc<dyn PrincipalVerifier>,
            authorizer: Arc<dyn TenantRbacMutationAuthorizer>,
        ) -> Self {
            Self {
                verifier,
                authorizer,
            }
        }

        /// # Errors
        /// [`PrincipalVerificationError`] — caller maps to HTTP 401.
        pub fn verify_principal(
            &self,
            credential: &CallerCredential,
        ) -> Result<VerifiedPrincipal, PrincipalVerificationError> {
            self.verifier.verify_principal(credential)
        }

        /// Authorize via the PDP port. `catch_unwind` is a **test/debug-only
        /// best-effort** backstop; in production `panic = "abort"` means it
        /// has no effect. Adapters MUST NOT panic — use `Err(Refused)`.
        ///
        /// # Errors
        /// [`TenantRbacMutationAuthorizationError`] — caller maps to HTTP 403.
        pub fn ensure_authorized(
            &self,
            principal: &VerifiedPrincipal,
            resource: &TenantRbacMutationResource,
        ) -> Result<(), TenantRbacMutationAuthorizationError> {
            let authorizer = Arc::clone(&self.authorizer);
            let principal = principal.clone();
            let resource = resource.clone();
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                authorizer.ensure_authorized(&principal, &resource)
            }))
            .unwrap_or(Err(TenantRbacMutationAuthorizationError::Refused))
        }
    }

    /// Constant-time byte comparison so bearer compares cannot be
    /// timing-probed. Mirrors `billing/adapters/accounting-http` and
    /// `intelligence/adapters/rest`. NEVER use naive `==` on secret material.
    ///
    /// Residual: input lengths are visible from the XOR seed; accepted, as
    /// the repo reference. Use a MAC if length-hiding is required.
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

    /// A reference [`PrincipalVerifier`] that verifies a bearer token by
    /// constant-time compare against a configured secret, then binds the
    /// principal identity from the config (NOT from caller headers/body).
    ///
    /// ## ⚠ BREAK-GLASS ONLY — NOT multi-tenant production
    ///
    /// Binds ONE static `(principal_id, tenant_id)` pair to a single shared
    /// secret. The production W5 adapter is the cloud-iam mTLS/SPIFFE
    /// peer-SVID verifier (ADR-0561).
    ///
    /// Construction REFUSES an empty secret or empty bound identity so a
    /// process that cannot prove a credential root can never authenticate
    /// a caller (mirrors the cloud-pdp boot-refusal doctrine).
    pub struct ConfiguredBearerPrincipalVerifier {
        bearer_secret: String,      // data_class: SECRET
        bound_principal_id: String, // data_class: INTERNAL_ONLY
        bound_tenant_id: String,    // data_class: INTERNAL_ONLY
    }

    impl ConfiguredBearerPrincipalVerifier {
        /// # Errors
        /// [`AuthzProviderConfigError`] when the secret or bound identity
        /// is empty.
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

    /// Why the authz provider refused construction. Boot-fatal: the
    /// composition root MUST refuse to serve.
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum AuthzProviderConfigError {
        /// The bearer secret was empty/whitespace (no provable credential
        /// root).
        EmptyBearerSecret,
        /// The bound principal/tenant identity was empty.
        EmptyBoundIdentity,
    }

    impl std::fmt::Display for AuthzProviderConfigError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::EmptyBearerSecret => {
                    write!(f, "authz provider bearer secret must be non-empty")
                }
                Self::EmptyBoundIdentity => {
                    write!(f, "authz provider bound principal/tenant must be non-empty")
                }
            }
        }
    }

    impl std::error::Error for AuthzProviderConfigError {}

    /// Map a matched route template to the control-plane action it gates.
    /// Returns `None` for non-mutation routes (health), which the middleware
    /// passes through unauthenticated.
    #[must_use]
    pub fn action_for_template(template: &str) -> Option<TenantRbacMutationAction> {
        match template {
            crate::TENANT_RBAC_POLICY_ADMISSIONS_PATH => {
                Some(TenantRbacMutationAction::PolicyAdmission)
            }
            crate::TENANT_RBAC_GROUP_CLOSE_ROLLUPS_PATH => {
                Some(TenantRbacMutationAction::GroupCloseRollup)
            }
            crate::TENANT_RBAC_CROSS_SERVICE_WORKFLOW_PLANS_PATH => {
                Some(TenantRbacMutationAction::CrossServiceWorkflowPlan)
            }
            crate::TENANT_RBAC_INCIDENT_ROLLBACK_PLANS_PATH => {
                Some(TenantRbacMutationAction::IncidentRollbackPlan)
            }
            crate::TENANT_RBAC_OPS_COMMANDS_PATH => Some(TenantRbacMutationAction::OpsCommand),
            _ => None,
        }
    }

    /// AUTHN-BEFORE-BODY middleware: verifies the caller principal and runs
    /// the PDP decision on control-plane mutation routes BEFORE the terminal
    /// handler deserializes the body. Short-circuits 401 (unauthenticated) /
    /// 403 (unauthorized or PDP fault) fail-closed. Non-mutation routes
    /// (health) pass through unauthenticated.
    ///
    /// On success injects the VERIFIED tenant into `path_captures` under
    /// [`VERIFIED_TENANT_CAPTURE_KEY`] so the handler can reject any body
    /// whose `tenant_id` does not match (no cross-tenant body substitution —
    /// true blast radius).
    pub struct TenantRbacAuthzMiddleware {
        provider: TenantRbacAuthzProvider,
    }

    impl TenantRbacAuthzMiddleware {
        #[must_use]
        pub fn new(provider: TenantRbacAuthzProvider) -> Self {
            Self { provider }
        }

        fn unauthorized_401() -> HttpResponse {
            HttpResponse::new(401)
                .with_header("content-type", "application/json")
                .with_header("www-authenticate", "Bearer")
                .with_body(
                    br#"{"error":{"code":"UNAUTHENTICATED","message":"tenant-rbac control-plane mutation requires a verified principal"}}"#
                        .to_vec(),
                )
        }

        fn forbidden_403() -> HttpResponse {
            HttpResponse::new(403)
                .with_header("content-type", "application/json")
                .with_body(
                    br#"{"error":{"code":"FORBIDDEN","message":"principal not authorized for this tenant-rbac control-plane mutation"}}"#
                        .to_vec(),
                )
        }
    }

    impl Middleware<HttpRequest, HttpResponse> for TenantRbacAuthzMiddleware {
        fn handle(
            &self,
            mut request: HttpRequest,
            next: Next<'_, HttpRequest, HttpResponse>,
        ) -> HttpResponse {
            // The matched template is the trusted route binding set by the
            // router at dispatch. Fall back to the raw path only when no
            // template was matched.
            let template = request
                .matched_template
                .clone()
                .unwrap_or_else(|| request.path.clone());

            // Non-mutation routes (health) need no authz.
            let Some(action) = action_for_template(&template) else {
                return next.run(request);
            };

            // 1. AUTHN: verify the caller principal from the Authorization
            //    header, BEFORE the handler deserializes the body.
            //    Missing/invalid -> 401.
            let credential = CallerCredential {
                authorization: request.headers.get(AUTHORIZATION_HEADER).cloned(),
            };
            let principal = match self.provider.verify_principal(&credential) {
                Ok(p) => p,
                Err(_) => return Self::unauthorized_401(),
            };

            // 2. AUTHZ: PDP decision bound to the VERIFIED tenant (not body
            //    input). Deny / fault -> 403 (fail-closed).
            let resource = TenantRbacMutationResource {
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

            // 3. Inject the verified tenant so the handler rejects any body
            //    whose tenant_id does not match (true blast radius / no IDOR).
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
            AuthzProviderConfigError, CallerCredential, ConfiguredBearerPrincipalVerifier,
            PrincipalVerificationError, PrincipalVerifier, TenantRbacAuthzProvider,
            TenantRbacMutationAction, TenantRbacMutationAuthorizationError,
            TenantRbacMutationAuthorizer, TenantRbacMutationResource, VerifiedPrincipal,
            action_for_template, constant_time_eq,
        };
        use std::sync::Arc;

        const SECRET: &str = "tenant-rbac-break-glass";

        fn verifier() -> ConfiguredBearerPrincipalVerifier {
            ConfiguredBearerPrincipalVerifier::new(SECRET, "sp_tenant_rbac", "ten_acme")
                .expect("verifier construction failed")
        }

        fn credential(authorization: Option<&str>) -> CallerCredential {
            CallerCredential {
                authorization: authorization.map(str::to_string),
            }
        }

        struct AllowAll;
        impl TenantRbacMutationAuthorizer for AllowAll {
            fn ensure_authorized(
                &self,
                _principal: &VerifiedPrincipal,
                _resource: &TenantRbacMutationResource,
            ) -> Result<(), TenantRbacMutationAuthorizationError> {
                Ok(())
            }
        }

        struct Panicker;
        impl TenantRbacMutationAuthorizer for Panicker {
            fn ensure_authorized(
                &self,
                _principal: &VerifiedPrincipal,
                _resource: &TenantRbacMutationResource,
            ) -> Result<(), TenantRbacMutationAuthorizationError> {
                panic!("boom");
            }
        }

        fn resource() -> TenantRbacMutationResource {
            TenantRbacMutationResource {
                tenant_id: "ten_acme".to_string(),
                action: TenantRbacMutationAction::PolicyAdmission,
                route_template: crate::TENANT_RBAC_POLICY_ADMISSIONS_PATH.to_string(),
            }
        }

        #[test]
        fn verifier_refuses_empty_secret_at_construction() {
            assert!(matches!(
                ConfiguredBearerPrincipalVerifier::new("  ", "sp_tenant_rbac", "ten_acme"),
                Err(AuthzProviderConfigError::EmptyBearerSecret)
            ));
            assert!(matches!(
                ConfiguredBearerPrincipalVerifier::new(SECRET, "", "ten_acme"),
                Err(AuthzProviderConfigError::EmptyBoundIdentity)
            ));
        }

        #[test]
        fn verifier_binds_identity_from_config_not_input() {
            let verified = verifier()
                .verify_principal(&credential(Some(&format!("Bearer {SECRET}"))))
                .unwrap();
            assert_eq!(verified.principal_id(), "sp_tenant_rbac");
            assert_eq!(verified.tenant_id(), "ten_acme");
        }

        #[test]
        fn verifier_rejects_missing_and_wrong_credential() {
            let v = verifier();
            assert_eq!(
                v.verify_principal(&credential(None)).unwrap_err(),
                PrincipalVerificationError::MissingCredential
            );
            assert_eq!(
                v.verify_principal(&credential(Some("Bearer wrong")))
                    .unwrap_err(),
                PrincipalVerificationError::InvalidCredential
            );
            assert_eq!(
                v.verify_principal(&credential(Some("Basic xyz")))
                    .unwrap_err(),
                PrincipalVerificationError::InvalidCredential
            );
        }

        #[test]
        fn provider_maps_panicking_authorizer_to_refused() {
            let provider = TenantRbacAuthzProvider::new(Arc::new(verifier()), Arc::new(Panicker));
            let principal = VerifiedPrincipal::new_for_test("sp_tenant_rbac", "ten_acme");
            assert_eq!(
                provider
                    .ensure_authorized(&principal, &resource())
                    .unwrap_err(),
                TenantRbacMutationAuthorizationError::Refused
            );
        }

        #[test]
        fn provider_allows_when_authorizer_allows() {
            let provider = TenantRbacAuthzProvider::new(Arc::new(verifier()), Arc::new(AllowAll));
            let principal = VerifiedPrincipal::new_for_test("sp_tenant_rbac", "ten_acme");
            assert!(provider.ensure_authorized(&principal, &resource()).is_ok());
        }

        #[test]
        fn constant_time_eq_matches_only_identical_inputs() {
            assert!(constant_time_eq(b"abc", b"abc"));
            assert!(!constant_time_eq(b"abc", b"abd"));
            assert!(!constant_time_eq(b"abc", b"ab"));
            assert!(!constant_time_eq(b"", b"a"));
            assert!(constant_time_eq(b"", b""));
        }

        #[test]
        fn action_surfaces_are_canonical() {
            assert_eq!(
                TenantRbacMutationAction::PolicyAdmission.surface(),
                "tenant-rbac.policy.admission"
            );
            assert_eq!(
                TenantRbacMutationAction::GroupCloseRollup.surface(),
                "tenant-rbac.group-close.rollup"
            );
            assert_eq!(
                TenantRbacMutationAction::CrossServiceWorkflowPlan.surface(),
                "tenant-rbac.cross-service-workflow.plan"
            );
            assert_eq!(
                TenantRbacMutationAction::IncidentRollbackPlan.surface(),
                "tenant-rbac.incident-rollback.plan"
            );
            assert_eq!(
                TenantRbacMutationAction::OpsCommand.surface(),
                "tenant-rbac.ops.command"
            );
        }

        #[test]
        fn action_for_template_only_maps_mutation_routes() {
            assert_eq!(
                action_for_template(crate::TENANT_RBAC_POLICY_ADMISSIONS_PATH),
                Some(TenantRbacMutationAction::PolicyAdmission)
            );
            assert_eq!(
                action_for_template(crate::TENANT_RBAC_GROUP_CLOSE_ROLLUPS_PATH),
                Some(TenantRbacMutationAction::GroupCloseRollup)
            );
            assert_eq!(
                action_for_template(crate::TENANT_RBAC_CROSS_SERVICE_WORKFLOW_PLANS_PATH),
                Some(TenantRbacMutationAction::CrossServiceWorkflowPlan)
            );
            assert_eq!(
                action_for_template(crate::TENANT_RBAC_INCIDENT_ROLLBACK_PLANS_PATH),
                Some(TenantRbacMutationAction::IncidentRollbackPlan)
            );
            assert_eq!(
                action_for_template(crate::TENANT_RBAC_OPS_COMMANDS_PATH),
                Some(TenantRbacMutationAction::OpsCommand)
            );
            assert_eq!(action_for_template(crate::TENANT_RBAC_HEALTH_PATH), None);
        }
    }
}

pub use authz::{
    AUTHORIZATION_HEADER, AuthzProviderConfigError, CallerCredential,
    ConfiguredBearerPrincipalVerifier, DecisionAuthorizer, PrincipalVerificationError,
    PrincipalVerifier, TenantRbacAuthzMiddleware, TenantRbacAuthzProvider,
    TenantRbacMutationAction, TenantRbacMutationAuthorizationError, TenantRbacMutationAuthorizer,
    TenantRbacMutationResource, VERIFIED_TENANT_CAPTURE_KEY, VerifiedPrincipal,
    action_for_template, constant_time_eq,
};

use std::time::Duration;

use iam_tenant_rbac_api::{
    ApiErrorEnvelope, CrossServiceWorkflowPlanRequest, GroupCloseRollupRequest,
    IncidentRollbackPlanRequest, ServiceWriteAdmissionRequest, TenantRbacOpsCommandRequest,
};
use iam_tenant_rbac_domain::{
    TenantRbacDomainError, admit_service_write, plan_cross_service_workflow,
    plan_incident_rollback, roll_up_group_close_status,
};
use iam_tenant_rbac_usecase::{
    TenantRbacApplicationError, prepare_cross_service_workflow_envelope,
    prepare_incident_rollback_envelope, prepare_tenant_rbac_ops_envelope,
};
use http_middleware_kernel::{HttpRequest, HttpResponse, MiddlewareChain};
use http_router_kernel::{HttpMethod, Router, RouterError};
use http_runtime_hyper_adapter::{
    ServerConfig, SyncHandler, dispatch as dispatch_http, handler_to_sync,
};
use serde::Serialize;

pub const TENANT_RBAC_POLICY_ADMISSIONS_PATH: &str = "/tenant-rbac/v1/policy-admissions";
pub const TENANT_RBAC_GROUP_CLOSE_ROLLUPS_PATH: &str = "/tenant-rbac/v1/group-close-rollups";
pub const TENANT_RBAC_CROSS_SERVICE_WORKFLOW_PLANS_PATH: &str =
    "/tenant-rbac/v1/cross-service-workflow-plans";
pub const TENANT_RBAC_INCIDENT_ROLLBACK_PLANS_PATH: &str =
    "/tenant-rbac/v1/incident-rollback-plans";
pub const TENANT_RBAC_OPS_COMMANDS_PATH: &str = "/tenant-rbac/v1/ops-commands";
pub const TENANT_RBAC_HEALTH_PATH: &str = "/tenant-rbac/v1/healthz";

const POLICY_ADMISSION_TOPIC: &str = "policy.tenant-rbac.service-write.admission";
const GROUP_CLOSE_ROLLUP_TOPIC: &str = "projection.tenant-rbac.group-close.rollup";
const JSON_CONTENT_TYPE: &str = "application/json";
const SERVICE_NAME: &str = "tenant-rbac";
const MAX_TENANT_RBAC_BODY_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacRuntimeRoute {
    pub method: &'static str,              // data_class: INTERNAL_ONLY
    pub path: &'static str,                // data_class: INTERNAL_ONLY
    pub operation_id: &'static str,        // data_class: INTERNAL_ONLY
    pub request_data_class: &'static str,  // data_class: INTERNAL_ONLY
    pub response_data_class: &'static str, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantRbacRuntimeError {
    Router(RouterError),
}

impl From<RouterError> for TenantRbacRuntimeError {
    fn from(error: RouterError) -> Self {
        Self::Router(error)
    }
}

impl std::fmt::Display for TenantRbacRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TenantRbacRuntimeError::Router(error) => {
                write!(f, "tenant-rbac router error: {error:?}")
            }
        }
    }
}

impl std::error::Error for TenantRbacRuntimeError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantRbacAcceptedResponse {
    pub accepted: bool,          // data_class: INTERNAL_ONLY
    pub topic: String,           // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
    pub schema_version: u32,     // data_class: PUBLIC
    pub service: String,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantRbacHealthResponse {
    pub status: String,                     // data_class: PUBLIC
    pub service: String,                    // data_class: PUBLIC
    pub runtime_adapter: String,            // data_class: PUBLIC
    pub deployed_listener: bool,            // data_class: PUBLIC
    pub auth_enforcement_runtime: bool,     // data_class: PUBLIC
    pub storage_attached: bool,             // data_class: PUBLIC
    pub workflow_execution: bool,           // data_class: PUBLIC
    pub open_tofu_execution: bool,          // data_class: PUBLIC
    pub incident_rollback_execution: bool,  // data_class: PUBLIC
    pub downstream_service_calls: bool,     // data_class: PUBLIC
    pub runtime_audit_chain_emission: bool, // data_class: PUBLIC
    pub cloud_service_integration: bool,    // data_class: PUBLIC
    pub schema_version: u32,                // data_class: PUBLIC
}

pub fn tenant_rbac_runtime_routes() -> Vec<TenantRbacRuntimeRoute> {
    vec![
        TenantRbacRuntimeRoute {
            method: "POST",
            path: TENANT_RBAC_POLICY_ADMISSIONS_PATH,
            operation_id: "admitTenantRbacServiceWrite",
            request_data_class: "INTERNAL_ONLY+FINANCIAL",
            response_data_class: "INTERNAL_ONLY",
        },
        TenantRbacRuntimeRoute {
            method: "POST",
            path: TENANT_RBAC_GROUP_CLOSE_ROLLUPS_PATH,
            operation_id: "rollUpTenantRbacGroupClose",
            request_data_class: "INTERNAL_ONLY+FINANCIAL",
            response_data_class: "INTERNAL_ONLY",
        },
        TenantRbacRuntimeRoute {
            method: "POST",
            path: TENANT_RBAC_CROSS_SERVICE_WORKFLOW_PLANS_PATH,
            operation_id: "planTenantRbacCrossServiceWorkflow",
            request_data_class: "INTERNAL_ONLY",
            response_data_class: "INTERNAL_ONLY",
        },
        TenantRbacRuntimeRoute {
            method: "POST",
            path: TENANT_RBAC_INCIDENT_ROLLBACK_PLANS_PATH,
            operation_id: "planTenantRbacIncidentRollback",
            request_data_class: "INTERNAL_ONLY",
            response_data_class: "INTERNAL_ONLY",
        },
        TenantRbacRuntimeRoute {
            method: "POST",
            path: TENANT_RBAC_OPS_COMMANDS_PATH,
            operation_id: "prepareTenantRbacOpsCommand",
            request_data_class: "INTERNAL_ONLY",
            response_data_class: "INTERNAL_ONLY",
        },
        TenantRbacRuntimeRoute {
            method: "GET",
            path: TENANT_RBAC_HEALTH_PATH,
            operation_id: "tenantRbacRuntimeHealth",
            request_data_class: "PUBLIC",
            response_data_class: "PUBLIC",
        },
    ]
}

pub fn tenant_rbac_server_config() -> ServerConfig {
    ServerConfig::default()
        .with_max_body_bytes(MAX_TENANT_RBAC_BODY_BYTES)
        .with_header_read_timeout(Duration::from_secs(10))
        .with_keepalive_timeout(Duration::from_secs(30))
}

/// Build the runtime middleware chain with the AUTH-005 fail-closed authz
/// middleware installed FIRST (ADR-0593). The [`TenantRbacAuthzMiddleware`]
/// verifies the caller principal and runs the PDP decision on every
/// control-plane mutation route BEFORE the terminal handler deserializes
/// the body; it short-circuits 401/403.
///
/// There is NO zero-argument / default chain: the composition root MUST
/// supply a [`TenantRbacAuthzProvider`] (verifier + PDP authorizer), so a
/// RBAC policy admission or group-close rollup can never be served without
/// both ports running.
#[must_use]
pub fn tenant_rbac_runtime_chain(
    provider: TenantRbacAuthzProvider,
) -> MiddlewareChain<HttpRequest, HttpResponse> {
    MiddlewareChain::new().push(Box::new(TenantRbacAuthzMiddleware::new(provider)))
}

pub fn tenant_rbac_runtime_router() -> Result<Router<SyncHandler>, TenantRbacRuntimeError> {
    let mut router = Router::new();
    router.route(
        HttpMethod::Post,
        TENANT_RBAC_POLICY_ADMISSIONS_PATH,
        handler_to_sync(PolicyAdmissionHandler),
    )?;
    router.route(
        HttpMethod::Post,
        TENANT_RBAC_GROUP_CLOSE_ROLLUPS_PATH,
        handler_to_sync(GroupCloseRollupHandler),
    )?;
    router.route(
        HttpMethod::Post,
        TENANT_RBAC_CROSS_SERVICE_WORKFLOW_PLANS_PATH,
        handler_to_sync(CrossServiceWorkflowHandler),
    )?;
    router.route(
        HttpMethod::Post,
        TENANT_RBAC_INCIDENT_ROLLBACK_PLANS_PATH,
        handler_to_sync(IncidentRollbackHandler),
    )?;
    router.route(
        HttpMethod::Post,
        TENANT_RBAC_OPS_COMMANDS_PATH,
        handler_to_sync(OpsCommandHandler),
    )?;
    router.route(
        HttpMethod::Get,
        TENANT_RBAC_HEALTH_PATH,
        handler_to_sync(HealthHandler),
    )?;
    Ok(router)
}

/// Dispatch a Tenant RBAC request through the fail-closed authz chain
/// (ADR-0593). The `provider` (principal verifier + PDP authorizer) is
/// required — control-plane mutation routes are gated 401/403 before any
/// handler runs. Health is passed through unauthenticated by the middleware.
pub fn dispatch_tenant_rbac_request(
    request: HttpRequest,
    provider: TenantRbacAuthzProvider,
) -> HttpResponse {
    match tenant_rbac_runtime_router() {
        Ok(router) => dispatch_http(request, &router, &tenant_rbac_runtime_chain(provider)),
        Err(error) => json_response(
            500,
            &ApiErrorEnvelope::validation(
                "Tenant RBAC runtime router failed",
                Some(error.to_string()),
            ),
        ),
    }
}

/// Cross-check that the body-claimed `tenant_id` equals the VERIFIED tenant
/// the authz middleware injected into `path_captures`. A mismatch is a
/// cross-tenant body substitution attempt (true blast radius): tenant A
/// presenting a verified credential but a body naming tenant B. Fail-closed
/// -> HTTP 403.
///
/// Returns `Err(403)` when the verified tenant is absent (the mutation
/// reached a handler without passing the authz middleware — defense in
/// depth) or differs from the body tenant.
fn enforce_body_tenant_matches_verified(
    req: &HttpRequest,
    body_tenant_id: &str,
) -> Result<(), HttpResponse> {
    match req.path_captures.get(VERIFIED_TENANT_CAPTURE_KEY) {
        Some(verified) if constant_time_eq(verified.as_bytes(), body_tenant_id.as_bytes()) => {
            Ok(())
        }
        _ => Err(json_response(
            403,
            &ApiErrorEnvelope::validation(
                "Tenant RBAC mutation tenant does not match the verified principal",
                None,
            ),
        )),
    }
}

struct PolicyAdmissionHandler;
struct GroupCloseRollupHandler;
struct CrossServiceWorkflowHandler;
struct IncidentRollbackHandler;
struct OpsCommandHandler;
struct HealthHandler;

impl http_middleware_kernel::Handler for PolicyAdmissionHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: ServiceWriteAdmissionRequest = parse_json(&req.body)?;
        enforce_body_tenant_matches_verified(&req, &request.tenant_id)?;
        let decision = admit_service_write(request.into_domain()).map_err(domain_error_response)?;
        Ok(json_response(
            202,
            &TenantRbacAcceptedResponse {
                accepted: true,
                topic: POLICY_ADMISSION_TOPIC.to_owned(),
                idempotency_key: decision.idempotency_key.value.clone(),
                schema_version: decision.schema_version.value,
                service: SERVICE_NAME.to_owned(),
            },
        ))
    }
}

impl http_middleware_kernel::Handler for GroupCloseRollupHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: GroupCloseRollupRequest = parse_json(&req.body)?;
        enforce_body_tenant_matches_verified(&req, &request.tenant_id)?;
        let rollup =
            roll_up_group_close_status(request.into_domain()).map_err(domain_error_response)?;
        Ok(json_response(
            200,
            &TenantRbacAcceptedResponse {
                accepted: true,
                topic: GROUP_CLOSE_ROLLUP_TOPIC.to_owned(),
                idempotency_key: format!(
                    "{}:{}:group-close-rollup",
                    rollup.tenant_id.value.value, rollup.group_id.value.value
                ),
                schema_version: rollup.schema_version.value,
                service: SERVICE_NAME.to_owned(),
            },
        ))
    }
}

impl http_middleware_kernel::Handler for CrossServiceWorkflowHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: CrossServiceWorkflowPlanRequest = parse_json(&req.body)?;
        enforce_body_tenant_matches_verified(&req, &request.tenant_id)?;
        let plan =
            plan_cross_service_workflow(request.into_domain()).map_err(domain_error_response)?;
        let envelope = prepare_cross_service_workflow_envelope(&plan);
        Ok(json_response(
            200,
            &TenantRbacAcceptedResponse {
                accepted: true,
                topic: envelope.topic.value.clone(),
                idempotency_key: envelope.idempotency_key.value.clone(),
                schema_version: envelope.schema_version.value,
                service: SERVICE_NAME.to_owned(),
            },
        ))
    }
}

impl http_middleware_kernel::Handler for IncidentRollbackHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: IncidentRollbackPlanRequest = parse_json(&req.body)?;
        enforce_body_tenant_matches_verified(&req, &request.tenant_id)?;
        let plan = plan_incident_rollback(request.into_domain()).map_err(domain_error_response)?;
        let envelope = prepare_incident_rollback_envelope(&plan);
        Ok(json_response(
            202,
            &TenantRbacAcceptedResponse {
                accepted: true,
                topic: envelope.topic.value.clone(),
                idempotency_key: envelope.idempotency_key.value.clone(),
                schema_version: envelope.schema_version.value,
                service: SERVICE_NAME.to_owned(),
            },
        ))
    }
}

impl http_middleware_kernel::Handler for OpsCommandHandler {
    type Error = HttpResponse;

    fn call(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        let request: TenantRbacOpsCommandRequest = parse_json(&req.body)?;
        enforce_body_tenant_matches_verified(&req, &request.tenant_id)?;
        let envelope =
            prepare_tenant_rbac_ops_envelope(request.into_app()).map_err(app_error_response)?;
        Ok(json_response(
            202,
            &TenantRbacAcceptedResponse {
                accepted: true,
                topic: envelope.topic.value.clone(),
                idempotency_key: envelope.idempotency_key.value.clone(),
                schema_version: envelope.schema_version.value,
                service: SERVICE_NAME.to_owned(),
            },
        ))
    }
}

impl http_middleware_kernel::Handler for HealthHandler {
    type Error = HttpResponse;

    fn call(&self, _req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        Ok(json_response(
            200,
            &TenantRbacHealthResponse {
                status: "ok".to_owned(),
                service: SERVICE_NAME.to_owned(),
                runtime_adapter: "router-ready".to_owned(),
                deployed_listener: false,
                auth_enforcement_runtime: true,
                storage_attached: false,
                workflow_execution: false,
                open_tofu_execution: false,
                incident_rollback_execution: false,
                downstream_service_calls: false,
                runtime_audit_chain_emission: false,
                cloud_service_integration: false,
                schema_version: 1,
            },
        ))
    }
}

fn parse_json<T>(body: &[u8]) -> Result<T, HttpResponse>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_slice(body).map_err(|error| {
        json_response(
            400,
            &ApiErrorEnvelope::validation(
                "Invalid Tenant RBAC JSON request",
                Some(error.to_string()),
            ),
        )
    })
}

fn domain_error_response(error: TenantRbacDomainError) -> HttpResponse {
    json_response(
        400,
        &ApiErrorEnvelope::validation("Invalid Tenant RBAC command", Some(format!("{error:?}"))),
    )
}

fn app_error_response(error: TenantRbacApplicationError) -> HttpResponse {
    json_response(
        400,
        &ApiErrorEnvelope::validation("Invalid Tenant RBAC command", Some(format!("{error:?}"))),
    )
}

fn json_response<T>(status: u16, body: &T) -> HttpResponse
where
    T: Serialize,
{
    match serde_json::to_vec(body) {
        Ok(bytes) => HttpResponse::new(status)
            .with_header("content-type", JSON_CONTENT_TYPE)
            .with_body(bytes),
        Err(error) => HttpResponse::new(500)
            .with_header("content-type", "text/plain; charset=utf-8")
            .with_body(format!("json serialization failed: {error}").into_bytes()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use http_router_kernel::HttpMethod;

    const BEARER_SECRET: &str = "tenant-rbac-break-glass";
    const BOUND_TENANT: &str = "ten_acme";
    const BOUND_PRINCIPAL: &str = "sp_tenant_rbac";

    struct AllowAll;
    impl TenantRbacMutationAuthorizer for AllowAll {
        fn ensure_authorized(
            &self,
            _principal: &VerifiedPrincipal,
            _resource: &TenantRbacMutationResource,
        ) -> Result<(), TenantRbacMutationAuthorizationError> {
            Ok(())
        }
    }

    struct RefuseAll;
    impl TenantRbacMutationAuthorizer for RefuseAll {
        fn ensure_authorized(
            &self,
            _principal: &VerifiedPrincipal,
            _resource: &TenantRbacMutationResource,
        ) -> Result<(), TenantRbacMutationAuthorizationError> {
            Err(TenantRbacMutationAuthorizationError::Refused)
        }
    }

    fn make_provider<A: TenantRbacMutationAuthorizer + 'static>(
        authorizer: A,
    ) -> TenantRbacAuthzProvider {
        let verifier =
            ConfiguredBearerPrincipalVerifier::new(BEARER_SECRET, BOUND_PRINCIPAL, BOUND_TENANT)
                .expect("verifier construction failed");
        TenantRbacAuthzProvider::new(Arc::new(verifier), Arc::new(authorizer))
    }

    fn post_request(path: &str, bearer: Option<&str>, body: Vec<u8>) -> HttpRequest {
        let mut headers = BTreeMap::new();
        headers.insert("content-type".to_owned(), "application/json".to_owned());
        if let Some(token) = bearer {
            headers.insert("authorization".to_owned(), format!("Bearer {token}"));
        }
        HttpRequest {
            method: HttpMethod::Post,
            path: path.to_owned(),
            headers,
            body,
            path_captures: BTreeMap::new(),
            matched_template: None,
        }
    }

    /// Valid JSON body for the policy-admissions route. Uses the same field
    /// values as the integration test fixtures in tests/runtime.rs so the domain
    /// function succeeds for the happy-path case. `tenant_id` is parameterized
    /// so the cross-tenant test can pass a mismatched value.
    fn policy_admission_body(tenant_id: &str) -> Vec<u8> {
        serde_json::json!({
            "service": "PAYROLL",
            "writeKind": "PAYROLL_CLOSE",
            "tenantId": tenant_id,
            "legalEntityId": "le_kr_001",
            "payloadDataClass": "FINANCIAL",
            "auditEvidenceRef": "audit/tenant-rbac/write/payroll-close",
            "policyGatewayRef": "policy/tenant-rbac/shared-gateway",
            "idempotencyKey": format!("{tenant_id}:le_kr_001:payroll-close"),
            "sequence": 1
        })
        .to_string()
        .into_bytes()
    }

    #[test]
    fn missing_bearer_on_mutation_route_returns_401() {
        let resp = dispatch_tenant_rbac_request(
            post_request(TENANT_RBAC_POLICY_ADMISSIONS_PATH, None, vec![]),
            make_provider(AllowAll),
        );
        assert_eq!(resp.status, 401);
    }

    #[test]
    fn wrong_bearer_on_mutation_route_returns_401() {
        let resp = dispatch_tenant_rbac_request(
            post_request(
                TENANT_RBAC_POLICY_ADMISSIONS_PATH,
                Some("forged-token"),
                vec![],
            ),
            make_provider(AllowAll),
        );
        assert_eq!(resp.status, 401);
    }

    #[test]
    fn pdp_fault_returns_403_before_handler_runs() {
        // Body is empty — the PDP short-circuit fires before body parse.
        let resp = dispatch_tenant_rbac_request(
            post_request(
                TENANT_RBAC_POLICY_ADMISSIONS_PATH,
                Some(BEARER_SECRET),
                vec![],
            ),
            make_provider(RefuseAll),
        );
        assert_eq!(resp.status, 403);
    }

    #[test]
    fn cross_tenant_body_returns_403() {
        // Valid bearer (authn+authz pass), but body tenant != verified tenant.
        let resp = dispatch_tenant_rbac_request(
            post_request(
                TENANT_RBAC_POLICY_ADMISSIONS_PATH,
                Some(BEARER_SECRET),
                policy_admission_body("ten_other"),
            ),
            make_provider(AllowAll),
        );
        assert_eq!(resp.status, 403);
    }

    #[test]
    fn valid_bearer_and_matching_tenant_returns_202() {
        // Happy path: bearer verifies, PDP permits, body tenant matches.
        let resp = dispatch_tenant_rbac_request(
            post_request(
                TENANT_RBAC_POLICY_ADMISSIONS_PATH,
                Some(BEARER_SECRET),
                policy_admission_body(BOUND_TENANT),
            ),
            make_provider(AllowAll),
        );
        assert_eq!(resp.status, 202);
    }

    #[test]
    fn health_route_passes_without_bearer_and_reports_auth_enforced() {
        // Health is a non-mutation route: middleware passes it through.
        let resp = dispatch_tenant_rbac_request(
            HttpRequest {
                method: HttpMethod::Get,
                path: TENANT_RBAC_HEALTH_PATH.to_owned(),
                headers: BTreeMap::new(),
                body: vec![],
                path_captures: BTreeMap::new(),
                matched_template: None,
            },
            make_provider(AllowAll),
        );
        assert_eq!(resp.status, 200);
        let body: serde_json::Value = serde_json::from_slice(&resp.body).unwrap();
        assert_eq!(body["authEnforcementRuntime"], true);
    }
}
