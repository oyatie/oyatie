use super::*;

pub(super) async fn list_groups<R, D, A, S, U, G>(
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
        .list_groups(&TenantId(tenant), &page_query(&uri))
        .await
    {
        Ok(listing) => Json::<_>(listing).into_response(),
        Err(error) => scim_error_response(error),
    }
}

pub(super) async fn create_group<R, D, A, S, U, G>(
    State(state): State<SharedScimState<R, D, A, S, U, G>>,
    Path(tenant): Path<String>,
    headers: HeaderMap,
    Json(input): Json<NewGroup>,
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
        .create_group(&TenantId(tenant), input, (state.now_provider)())
        .await
    {
        Ok(group) => (StatusCode::CREATED, Json(group)).into_response(),
        Err(error) => scim_error_response(error),
    }
}

pub(super) async fn get_group<R, D, A, S, U, G>(
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
    match state.server.get_group(&TenantId(tenant), &ScimId(id)).await {
        Ok(group) => Json(group).into_response(),
        Err(error) => scim_error_response(error),
    }
}

pub(super) async fn patch_group<R, D, A, S, U, G>(
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
    match state
        .server
        .patch_group(&TenantId(tenant), &ScimId(id), &op, (state.now_provider)())
        .await
    {
        Ok(group) => Json(group).into_response(),
        Err(error) => scim_error_response(error),
    }
}

pub(super) async fn delete_group<R, D, A, S, U, G>(
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
    match state
        .server
        .delete_group(&TenantId(tenant), &ScimId(id))
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => scim_error_response(error),
    }
}
