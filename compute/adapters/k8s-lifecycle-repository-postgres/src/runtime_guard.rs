use sqlx::{PgConnection, PgPool, Row};

use crate::error::PgK8sLifecycleConnectError;
use crate::migrations::MIGRATIONS_TABLE;
use crate::schema::attest_complete;
use crate::serving_role_guard::attest_serving_role_graph;
use crate::{CLUSTERS_TABLE, OPERATIONS_TABLE, PgK8sLifecycleRuntimeContract, SCHEMA_NAME};

const RUNTIME_AUTHORITY_SQL: &str = r#"
SELECT
    r.rolname AS role_name,
    r.rolsuper
        OR r.rolcreatedb
        OR r.rolcreaterole
        OR r.rolreplication
        OR r.rolbypassrls
        OR EXISTS (
            SELECT 1
            FROM pg_roles candidate
            WHERE pg_has_role(r.oid, candidate.oid, 'MEMBER')
              AND (
                candidate.rolsuper
                OR candidate.rolcreatedb
                OR candidate.rolcreaterole
                OR candidate.rolreplication
                OR candidate.rolbypassrls
                OR has_database_privilege(candidate.oid, current_database(), 'CREATE')
                OR candidate.rolname IN (
                    'pg_read_server_files',
                    'pg_write_server_files',
                    'pg_execute_server_program',
                    'pg_signal_backend',
                    'pg_checkpoint',
                    'pg_maintain',
                    'pg_create_subscription'
                )
                OR COALESCE(
                    has_schema_privilege(candidate.oid, to_regnamespace($1), 'CREATE'),
                    false
                )
                OR COALESCE(has_table_privilege(candidate.oid, to_regclass($2), 'DELETE'), false)
                OR COALESCE(has_table_privilege(candidate.oid, to_regclass($2), 'TRUNCATE'), false)
                OR COALESCE(has_table_privilege(candidate.oid, to_regclass($2), 'REFERENCES'), false)
                OR COALESCE(has_table_privilege(candidate.oid, to_regclass($2), 'TRIGGER'), false)
                OR COALESCE(has_table_privilege(candidate.oid, to_regclass($3), 'DELETE'), false)
                OR COALESCE(has_table_privilege(candidate.oid, to_regclass($3), 'TRUNCATE'), false)
                OR COALESCE(has_table_privilege(candidate.oid, to_regclass($3), 'REFERENCES'), false)
                OR COALESCE(has_table_privilege(candidate.oid, to_regclass($3), 'TRIGGER'), false)
                OR COALESCE(has_table_privilege(candidate.oid, to_regclass($4), 'INSERT'), false)
                OR COALESCE(has_table_privilege(candidate.oid, to_regclass($4), 'UPDATE'), false)
                OR COALESCE(has_table_privilege(candidate.oid, to_regclass($4), 'DELETE'), false)
                OR COALESCE(has_table_privilege(candidate.oid, to_regclass($4), 'TRUNCATE'), false)
                OR COALESCE(has_table_privilege(candidate.oid, to_regclass($4), 'REFERENCES'), false)
                OR COALESCE(has_table_privilege(candidate.oid, to_regclass($4), 'TRIGGER'), false)
                OR COALESCE(
                    has_any_column_privilege(candidate.oid, to_regclass($4), 'INSERT'),
                    false
                )
                OR COALESCE(
                    has_any_column_privilege(candidate.oid, to_regclass($4), 'UPDATE'),
                    false
                )
                OR COALESCE(
                    has_any_column_privilege(candidate.oid, to_regclass($4), 'REFERENCES'),
                    false
                )
                OR EXISTS (
                    SELECT 1
                    FROM pg_namespace n
                    WHERE n.nspname <> $1
                      AND n.nspname <> 'information_schema'
                      AND n.nspname NOT LIKE 'pg\_%' ESCAPE '\'
                      AND has_schema_privilege(candidate.oid, n.oid, 'CREATE')
                )
                OR EXISTS (
                    SELECT 1
                    FROM pg_class c
                    JOIN pg_namespace n ON n.oid = c.relnamespace
                    CROSS JOIN (VALUES
                        ('SELECT'), ('INSERT'), ('UPDATE'), ('DELETE'),
                        ('TRUNCATE'), ('REFERENCES'), ('TRIGGER')
                    ) AS privilege(name)
                    WHERE n.nspname <> $1
                      AND n.nspname <> 'information_schema'
                      AND n.nspname NOT LIKE 'pg\_%' ESCAPE '\'
                      AND c.relkind IN ('r', 'p', 'v', 'm', 'f')
                      AND has_table_privilege(candidate.oid, c.oid, privilege.name)
                )
                OR EXISTS (
                    SELECT 1
                    FROM pg_class c
                    JOIN pg_namespace n ON n.oid = c.relnamespace
                    CROSS JOIN (VALUES
                        ('SELECT'), ('INSERT'), ('UPDATE'), ('REFERENCES')
                    ) AS privilege(name)
                    WHERE n.nspname <> $1
                      AND n.nspname <> 'information_schema'
                      AND n.nspname NOT LIKE 'pg\_%' ESCAPE '\'
                      AND c.relkind IN ('r', 'p', 'v', 'm', 'f')
                      AND has_any_column_privilege(candidate.oid, c.oid, privilege.name)
                )
                OR EXISTS (
                    SELECT 1
                    FROM pg_class c
                    JOIN pg_namespace n ON n.oid = c.relnamespace
                    CROSS JOIN (VALUES ('SELECT'), ('UPDATE'), ('USAGE')) AS privilege(name)
                    WHERE n.nspname <> $1
                      AND n.nspname <> 'information_schema'
                      AND n.nspname NOT LIKE 'pg\_%' ESCAPE '\'
                      AND c.relkind = 'S'
                      AND has_sequence_privilege(candidate.oid, c.oid, privilege.name)
                )
                OR EXISTS (
                    SELECT 1
                    FROM pg_proc p
                    JOIN pg_namespace n ON n.oid = p.pronamespace
                    WHERE p.prosecdef
                      AND n.nspname NOT IN ('pg_catalog', 'information_schema')
                      AND has_function_privilege(candidate.oid, p.oid, 'EXECUTE')
                )
                OR EXISTS (
                    SELECT 1
                    FROM pg_parameter_acl parameter_acl
                    LEFT JOIN pg_settings setting
                      ON lower(setting.name) = lower(parameter_acl.parname)
                    WHERE has_parameter_privilege(
                        candidate.oid,
                        parameter_acl.parname,
                        'ALTER SYSTEM'
                    )
                       OR (
                            has_parameter_privilege(
                                candidate.oid,
                                parameter_acl.parname,
                                'SET'
                            )
                            AND setting.context IS DISTINCT FROM 'user'
                       )
                )
              )
        )
        OR EXISTS (
            SELECT 1
            FROM pg_database d
            WHERE d.datname = current_database()
              AND pg_has_role(r.oid, d.datdba, 'MEMBER')
        )
        OR EXISTS (
            SELECT 1
            FROM pg_auth_members delegation
            WHERE delegation.admin_option
              AND pg_has_role(r.oid, delegation.member, 'MEMBER')
        ) AS has_authority
FROM pg_roles r
WHERE r.rolname = ANY($5)
ORDER BY r.rolname
"#;

pub(crate) async fn attest_runtime(
    pool: &PgPool,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) -> Result<(), PgK8sLifecycleConnectError> {
    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| PgK8sLifecycleConnectError::Sqlx(error.to_string()))?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *transaction)
        .await
        .map_err(|error| PgK8sLifecycleConnectError::Sqlx(error.to_string()))?;
    crate::catalog_connection::use_catalog_path(&mut transaction)
        .await
        .map_err(|error| PgK8sLifecycleConnectError::Sqlx(error.to_string()))?;
    for setting in [
        "SET LOCAL lock_timeout = '5s'",
        "SET LOCAL statement_timeout = '10s'",
    ] {
        sqlx::query(setting)
            .execute(&mut *transaction)
            .await
            .map_err(|error| PgK8sLifecycleConnectError::Sqlx(error.to_string()))?;
    }
    attest_serving_role_graph(&mut transaction, runtime_contract).await?;
    if let Some(role) = role_with_privileged_authority(&mut transaction, runtime_contract)
        .await
        .map_err(|error| PgK8sLifecycleConnectError::Sqlx(error.to_string()))?
    {
        return Err(PgK8sLifecycleConnectError::PrivilegedAuthorityPresent { role });
    }
    attest_complete(&mut transaction).await?;
    transaction
        .commit()
        .await
        .map_err(|error| PgK8sLifecycleConnectError::Sqlx(error.to_string()))?;
    Ok(())
}

async fn role_with_privileged_authority(
    connection: &mut PgConnection,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) -> Result<Option<String>, sqlx::Error> {
    let rows = sqlx::query(RUNTIME_AUTHORITY_SQL)
        .bind(SCHEMA_NAME)
        .bind(CLUSTERS_TABLE)
        .bind(OPERATIONS_TABLE)
        .bind(MIGRATIONS_TABLE)
        .bind(runtime_contract.owned_role_names())
        .fetch_all(connection)
        .await?;
    for row in rows {
        if row.try_get::<bool, _>("has_authority")? {
            return Ok(Some(row.try_get("role_name")?));
        }
    }
    Ok(None)
}
