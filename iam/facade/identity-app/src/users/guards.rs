use super::*;

/// Emit the single deny-class audit record for a guard refusal that occurred
/// BEFORE the PEP hot path ran (invalid credential / missing scope / tenant
/// mismatch). Same record shape + vocabulary as the authorize surfaces; the
/// raw token never reaches the record. The `detail` carries the refusal
/// stage (`scim-guard` for a rejected credential, `scope-missing`,
/// `tenant-mismatch`) — deliberately richer than the PEP's `token-rejected`
/// record (`detail: None`) so forensics can tell WHICH surface refused.
pub(super) fn audit_guard_refusal<R, D, A, S, U, G>(
    state: &ScimSurfaceState<R, D, A, S, U, G>,
    workload_id: Option<String>,
    outcome: &'static str,
    detail: &str,
    tenant: &str,
    action: &str,
) where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    state.authz.audit().record(
        AuditRecord::new(
            AuditEvent::Authorize,
            workload_id,
            outcome,
            Some(detail.to_owned()),
        )
        .with_authorization_target(action, SCIM_TENANT_RESOURCE_TYPE, tenant),
    );
}

/// Validate the Bearer credential and require [`SCIM_MANAGE_SCOPE`].
/// `Err` is the ready-to-return refusal response (401/403) — never a pass.
/// Exactly one audit record is emitted per credential-bearing guard run:
/// pre-authorize refusals emit here, the PEP decision emits inside
/// `authorize_token_for` (see the module-level audit section).
pub(crate) fn authorize_scim<R, D, A, S, U, G>(
    state: &ScimSurfaceState<R, D, A, S, U, G>,
    headers: &HeaderMap,
    tenant: &str,
) -> Result<(), Box<Response>>
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    authorize_scim_action(
        state,
        headers,
        tenant,
        SCIM_MANAGE_SCOPE,
        SCIM_MANAGE_ACTION,
    )
}

pub(crate) fn authorize_native_user_admin<R, D, A, S, U, G>(
    state: &ScimSurfaceState<R, D, A, S, U, G>,
    headers: &HeaderMap,
    tenant: &str,
) -> Result<(), Box<Response>>
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    authorize_scim_action(
        state,
        headers,
        tenant,
        NATIVE_USER_ADMIN_SCOPE,
        NATIVE_USER_ADMIN_ACTION,
    )
}

pub(super) fn authorize_scim_action<R, D, A, S, U, G>(
    state: &ScimSurfaceState<R, D, A, S, U, G>,
    headers: &HeaderMap,
    tenant: &str,
    required_scope: &str,
    required_action: &str,
) -> Result<(), Box<Response>>
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    let token = bearer_token(headers)
        // No credential -> no decision rendered -> no audit record (module doc).
        .ok_or_else(|| scim_refusal(StatusCode::UNAUTHORIZED, "missing bearer credential"))?;
    let principal = validate_workload_token(
        token,
        &state.jwks,
        &state.validation,
        (state.now_provider)(),
    )
    .map_err(|_| {
        audit_guard_refusal(
            state,
            None,
            "token-rejected",
            "scim-guard",
            tenant,
            required_action,
        );
        scim_refusal(StatusCode::UNAUTHORIZED, "invalid bearer credential")
    })?;
    if !principal.has_scope(required_scope) {
        audit_guard_refusal(
            state,
            Some(principal.workload_id().as_str().to_owned()),
            "deny",
            "scope-missing",
            tenant,
            required_action,
        );
        return Err(scim_refusal(
            StatusCode::FORBIDDEN,
            &format!("credential lacks the {required_scope} scope"),
        ));
    }
    if principal.tenant_id().as_str() != tenant {
        audit_guard_refusal(
            state,
            Some(principal.workload_id().as_str().to_owned()),
            "deny",
            "tenant-mismatch",
            tenant,
            required_action,
        );
        return Err(scim_refusal(
            StatusCode::FORBIDDEN,
            "credential tenant does not match the SCIM tenant",
        ));
    }
    let mut context = BTreeMap::new();
    context.insert(
        "scim_tenant".to_owned(),
        ClaimValue::Text(tenant.to_owned()),
    );
    match state.authz.authorize_token_for(
        token,
        Action::new(required_action),
        Resource::new(SCIM_TENANT_RESOURCE_TYPE, tenant.to_owned()),
        context,
    ) {
        AuthorizeOutcome::Decided(decision) if decision.is_allow() => Ok(()),
        AuthorizeOutcome::TokenRejected => Err(scim_refusal(
            StatusCode::UNAUTHORIZED,
            "invalid bearer credential",
        )),
        AuthorizeOutcome::StoreUnavailable => Err(scim_refusal(
            StatusCode::SERVICE_UNAVAILABLE,
            "identity authorization store unavailable",
        )),
        AuthorizeOutcome::PrincipalUnknown => Err(scim_refusal(
            StatusCode::FORBIDDEN,
            "credential principal is not registered",
        )),
        AuthorizeOutcome::Revoked => Err(scim_refusal(
            StatusCode::FORBIDDEN,
            "credential principal is revoked",
        )),
        AuthorizeOutcome::Decided(_) => Err(scim_refusal(
            StatusCode::FORBIDDEN,
            "credential is not permitted to perform the identity action",
        )),
    }
}

pub(super) fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
}

pub(super) fn scim_refusal(status: StatusCode, detail: &str) -> Box<Response> {
    Box::new((status, Json(ScimError::new(status.as_u16(), None, detail))).into_response())
}

pub(super) fn scim_error_response(error: ScimError) -> Response {
    let status = StatusCode::from_u16(error.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(error)).into_response()
}

// =====================================================================
// Handlers
// =====================================================================

/// Parse `startIndex`/`count`/`filter` per RFC 7644 §3.4.2.4 (1-indexed)
/// from the request URI. Hand-rolled key=value split rather than the serde
/// `Query` extractor: the workspace axum build excludes the `query` feature.
/// Unparseable values fall back to the kernel defaults (a malformed page
/// param must not fail provisioning reads).
pub(super) fn page_query(uri: &Uri) -> ListQuery {
    let mut query = ListQuery::default();
    for pair in uri.query().unwrap_or_default().split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        match key {
            "startIndex" => {
                if let Ok(value) = value.parse() {
                    query.start_index = value;
                }
            }
            "count" => {
                if let Ok(value) = value.parse() {
                    query.items_per_page = value;
                }
            }
            "filter" if !value.is_empty() => {
                query.filter = Some(value.to_owned());
            }
            _ => {}
        }
    }
    query
}
