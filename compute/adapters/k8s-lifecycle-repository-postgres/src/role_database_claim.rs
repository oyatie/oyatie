use sqlx::{PgConnection, Row};

use crate::{
    RUNTIME_ROLE,
    error::{PgK8sLifecycleRoleDatabaseClaimError, PgK8sLifecycleSchemaError},
};

const ROLE_DATABASE_DEPENDENCIES: &str = r#"
SELECT
    dependency.dbid <> 0 OR dependency.classid = 'pg_catalog.pg_database'::regclass AS database_dependency,
    COALESCE(database.datname = current_database(), false) AS claims_current_database,
    COALESCE(database.datname = current_database(), false)
        OR (
            dependency.dbid = 0
            AND dependency.classid = 'pg_catalog.pg_parameter_acl'::regclass
            AND EXISTS (
                SELECT 1 FROM pg_parameter_acl parameter_acl
                JOIN pg_settings setting ON lower(setting.name) = lower(parameter_acl.parname)
                WHERE parameter_acl.oid = dependency.objid
                  AND setting.context = 'user'
                  AND has_parameter_privilege($1, parameter_acl.parname, 'SET')
                  AND NOT has_parameter_privilege($1, parameter_acl.parname, 'ALTER SYSTEM')
            )
        ) AS admitted
FROM pg_shdepend dependency
LEFT JOIN pg_database database ON database.oid = CASE
    WHEN dependency.dbid <> 0 THEN dependency.dbid
    WHEN dependency.classid = 'pg_catalog.pg_database'::regclass THEN dependency.objid
    ELSE NULL
END
WHERE dependency.refclassid = 'pg_catalog.pg_authid'::regclass
  AND dependency.refobjid = (SELECT oid FROM pg_roles WHERE rolname = $1)
"#;

pub(crate) async fn attest_role_database_claim(
    connection: &mut PgConnection,
    allow_absent: bool,
) -> Result<(), PgK8sLifecycleSchemaError> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = $1)")
            .bind(RUNTIME_ROLE)
            .fetch_one(&mut *connection)
            .await?;
    if !exists {
        return allow_absent
            .then_some(())
            .ok_or(PgK8sLifecycleSchemaError::RuntimeRoleContract);
    }
    let dependencies = sqlx::query(ROLE_DATABASE_DEPENDENCIES)
        .bind(RUNTIME_ROLE)
        .fetch_all(connection)
        .await?;
    let mut claims_current_database = false;
    for dependency in dependencies {
        if !dependency.try_get::<bool, _>("admitted")? {
            let reason = if dependency.try_get::<bool, _>("database_dependency")? {
                PgK8sLifecycleRoleDatabaseClaimError::ForeignOrUnresolvedDatabase
            } else {
                PgK8sLifecycleRoleDatabaseClaimError::UnsupportedSharedDependency
            };
            return Err(PgK8sLifecycleSchemaError::RuntimeRoleDatabaseClaim(reason));
        }
        claims_current_database |= dependency.try_get::<bool, _>("claims_current_database")?;
    }
    claims_current_database.then_some(()).ok_or(
        PgK8sLifecycleSchemaError::RuntimeRoleDatabaseClaim(
            PgK8sLifecycleRoleDatabaseClaimError::Unclaimed,
        ),
    )
}
