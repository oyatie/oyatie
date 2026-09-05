use std::collections::BTreeSet;

use sqlx::{PgConnection, Row};

use crate::error::PgK8sLifecycleConnectError;
use crate::{PgK8sLifecycleRuntimeContract, RUNTIME_ROLE};

pub(crate) async fn attest_serving_role_graph(
    connection: &mut PgConnection,
    contract: &PgK8sLifecycleRuntimeContract,
) -> Result<(), PgK8sLifecycleConnectError> {
    let current_role: String = sqlx::query_scalar("SELECT current_user::text")
        .fetch_one(&mut *connection)
        .await
        .map_err(sqlx_error)?;
    if !contract.contains(&current_role) {
        return Err(PgK8sLifecycleConnectError::ServingPrincipalNotAllowed { role: current_role });
    }

    let expected: BTreeSet<String> = contract.serving_roles().map(str::to_owned).collect();
    let direct = sqlx::query(
        "SELECT member.rolname, membership.admin_option, membership.inherit_option, membership.set_option FROM pg_auth_members membership JOIN pg_roles granted ON granted.oid = membership.roleid JOIN pg_roles member ON member.oid = membership.member WHERE granted.rolname = $1",
    )
    .bind(RUNTIME_ROLE)
    .fetch_all(&mut *connection)
    .await
    .map_err(sqlx_error)?;
    let mut observed = BTreeSet::new();
    for row in direct {
        let role: String = row.try_get("rolname").map_err(sqlx_error)?;
        let safe_edge = !row.try_get::<bool, _>("admin_option").map_err(sqlx_error)?
            && row
                .try_get::<bool, _>("inherit_option")
                .map_err(sqlx_error)?
            && !row.try_get::<bool, _>("set_option").map_err(sqlx_error)?;
        if !safe_edge || !observed.insert(role) {
            return Err(PgK8sLifecycleConnectError::ServingRoleGraphMismatch);
        }
    }
    if observed != expected {
        return Err(PgK8sLifecycleConnectError::ServingRoleGraphMismatch);
    }

    let roles = contract.owned_role_names();
    let unsafe_parent_edges: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM pg_auth_members membership JOIN pg_roles granted ON granted.oid = membership.roleid JOIN pg_roles member ON member.oid = membership.member WHERE member.rolname = ANY($1) AND (granted.rolname <> $2 OR membership.admin_option OR NOT membership.inherit_option OR membership.set_option)",
    )
    .bind(&roles)
    .bind(RUNTIME_ROLE)
    .fetch_one(&mut *connection)
    .await
    .map_err(sqlx_error)?;
    let transitive_members: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM pg_auth_members membership JOIN pg_roles granted ON granted.oid = membership.roleid WHERE granted.rolname = ANY($1)",
    )
    .bind(&roles)
    .fetch_one(connection)
    .await
    .map_err(sqlx_error)?;
    if unsafe_parent_edges != 0 || transitive_members != 0 {
        return Err(PgK8sLifecycleConnectError::ServingRoleGraphMismatch);
    }
    Ok(())
}

fn sqlx_error(error: sqlx::Error) -> PgK8sLifecycleConnectError {
    PgK8sLifecycleConnectError::Sqlx(error.to_string())
}
