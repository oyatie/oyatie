use shared_postgres_command_kernel::{
    RLS_ROLE_PROBE_SQL, RLS_TABLE_FORCED_PROBE_SQL, RlsEnforceabilityError, evaluate_rls_forced,
    evaluate_rls_role_flags,
};
use sqlx::{PgPool, Row};

use crate::{GOVERNED_TABLES, RUNTIME_ROLE, catalog_connection::use_catalog_path};

pub(crate) async fn attest_rls(pool: &PgPool) -> Result<(), RlsEnforceabilityError> {
    let mut transaction = pool.begin().await.map_err(probe_failed)?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *transaction)
        .await
        .map_err(probe_failed)?;
    use_catalog_path(&mut transaction)
        .await
        .map_err(probe_failed)?;
    let row = sqlx::query(RLS_ROLE_PROBE_SQL)
        .bind(RUNTIME_ROLE)
        .fetch_one(&mut *transaction)
        .await
        .map_err(probe_failed)?;
    let role_name: String = row.try_get("role_name").map_err(probe_failed)?;
    let session_role: String = row.try_get("session_role").map_err(probe_failed)?;
    if session_role != role_name {
        return Err(RlsEnforceabilityError::RoleSwitchInEffect {
            session_role,
            current_role: role_name,
        });
    }
    evaluate_rls_role_flags(
        &role_name,
        RUNTIME_ROLE,
        row.try_get("rolsuper").map_err(probe_failed)?,
        row.try_get("rolbypassrls").map_err(probe_failed)?,
        row.try_get("is_runtime_member").map_err(probe_failed)?,
    )?;
    for qualified_table in GOVERNED_TABLES {
        let (schema, table) = qualified_table
            .split_once('.')
            .unwrap_or(("", qualified_table));
        let row = sqlx::query(RLS_TABLE_FORCED_PROBE_SQL)
            .bind(schema)
            .bind(table)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(probe_failed)?
            .ok_or_else(|| RlsEnforceabilityError::GovernedTableMissing {
                table: (*qualified_table).to_owned(),
            })?;
        evaluate_rls_forced(
            qualified_table,
            row.try_get("row_security").map_err(probe_failed)?,
            row.try_get("force_row_security").map_err(probe_failed)?,
        )?;
    }
    transaction.commit().await.map_err(probe_failed)
}

fn probe_failed(error: sqlx::Error) -> RlsEnforceabilityError {
    RlsEnforceabilityError::ProbeFailed {
        detail: error.to_string(),
    }
}
