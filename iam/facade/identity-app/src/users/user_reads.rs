use super::*;

pub(super) async fn list_users<R, D, A, S, U, G>(
    State(state): State<SharedScimState<R, D, A, S, U, G>>,
    Path(tenant): Path<String>,
    uri: Uri,
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
    match state
        .server
        .list_users(&TenantId(tenant), &page_query(&uri))
        .await
    {
        Ok(listing) => Json(listing).into_response(),
        Err(error) => scim_error_response(error),
    }
}

pub(super) async fn create_user<R, D, A, S, U, G>(
    State(state): State<SharedScimState<R, D, A, S, U, G>>,
    Path(tenant): Path<String>,
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
    let native_admin = requests_native_binding(&input);
    if native_admin {
        guard_native_admin!(state, headers, tenant);
    }
    match state
        .server
        .create_user_with_authority(
            &TenantId(tenant),
            input,
            (state.now_provider)(),
            native_admin,
        )
        .await
    {
        Ok(user) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(error) => scim_error_response(error),
    }
}

pub(super) async fn register_account<R, D, A, S, U, G>(
    State(state): State<SharedScimState<R, D, A, S, U, G>>,
    Path(tenant): Path<String>,
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
    let native_admin = requests_native_binding(&input);
    if native_admin {
        guard_native_admin!(state, headers, tenant);
    }
    match state
        .server
        .create_user_with_authority(
            &TenantId(tenant),
            input,
            (state.now_provider)(),
            native_admin,
        )
        .await
    {
        Ok(user) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(error) => scim_error_response(error),
    }
}

pub(super) async fn get_user<R, D, A, S, U, G>(
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
    match state.server.get_user(&TenantId(tenant), &ScimId(id)).await {
        Ok(user) => Json(user).into_response(),
        Err(error) => scim_error_response(error),
    }
}
