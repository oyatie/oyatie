use super::*;

pub(super) async fn replace_user<R, D, A, S, U, G>(
    State(state): State<SharedScimState<R, D, A, S, U, G>>,
    Path((tenant, id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(input): Json<NewUser>,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    guard!(state, headers, tenant);
    let native_admin = if requests_native_binding(&input) {
        guard_native_admin!(state, headers, tenant);
        true
    } else {
        optional_native_admin(&state, &headers, &tenant)
    };
    match state
        .server
        .replace_user_with_authority(
            &TenantId(tenant),
            &ScimId(id),
            input,
            (state.now_provider)(),
            native_admin,
        )
        .await
    {
        Ok(user) => Json(user).into_response(),
        Err(error) => scim_error_response(error),
    }
}

pub(super) async fn patch_user<R, D, A, S, U, G>(
    State(state): State<SharedScimState<R, D, A, S, U, G>>,
    Path((tenant, id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(op): Json<PatchOp>,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    guard!(state, headers, tenant);
    let native_admin = if patch_requires_native_admin(&op) {
        guard_native_admin!(state, headers, tenant);
        true
    } else {
        optional_native_admin(&state, &headers, &tenant)
    };
    match state
        .server
        .patch_user_with_authority(
            &TenantId(tenant),
            &ScimId(id),
            &op,
            (state.now_provider)(),
            native_admin,
        )
        .await
    {
        Ok(user) => Json(user).into_response(),
        Err(error) => scim_error_response(error),
    }
}

pub(super) async fn delete_user<R, D, A, S, U, G>(
    State(state): State<SharedScimState<R, D, A, S, U, G>>,
    Path((tenant, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    guard!(state, headers, tenant);
    let native_admin = optional_native_admin(&state, &headers, &tenant);
    match state
        .server
        .delete_user_with_authority(&TenantId(tenant), &ScimId(id), native_admin)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => scim_error_response(error),
    }
}
