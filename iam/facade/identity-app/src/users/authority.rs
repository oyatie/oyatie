use super::*;

pub(super) fn requests_native_binding(input: &NewUser) -> bool {
    input
        .oyatie
        .as_ref()
        .is_some_and(|extension| extension.native_user_id.is_some() || extension.roles.is_some())
}

pub(super) fn patch_requires_native_admin(op: &PatchOp) -> bool {
    op.operations.iter().any(|operation| {
        let Some(path) = operation.path.as_deref() else {
            // An absent path targets the whole resource. The kernel currently
            // refuses it; the guard must still precede any future support.
            return true;
        };
        matches!(path, "nativeUserId" | "roles")
            || path == "urn:oyatie:scim:extension:2.0:User"
            || path
                .strip_prefix("urn:oyatie:scim:extension:2.0:User:")
                .is_some_and(|attribute| matches!(attribute, "nativeUserId" | "roles"))
    })
}

/// An administrative credential may also authorize protected state changes
/// that ordinary request fields imply, such as reactivation or deletion.
/// The scope probe is only a hint; the full Cedar guard still decides. A
/// normal SCIM writer proceeds with unprivileged atomic store checks.
pub(super) fn optional_native_admin<R, D, A, S, U, G>(
    state: &ScimSurfaceState<R, D, A, S, U, G>,
    headers: &HeaderMap,
    tenant: &str,
) -> bool
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    U: UserStore + Send + Sync + 'static,
    G: GroupStore + Send + Sync + 'static,
{
    let scoped = bearer_token(headers)
        .and_then(|token| {
            validate_workload_token(
                token,
                &state.jwks,
                &state.validation,
                (state.now_provider)(),
            )
            .ok()
        })
        .is_some_and(|principal| principal.has_scope(NATIVE_USER_ADMIN_SCOPE));
    scoped && authorize_native_user_admin(state, headers, tenant).is_ok()
}
