use shared_postgres_command_kernel::split_migration_statements;
use sqlx::{PgConnection, PgPool, postgres::PgPoolOptions};

use crate::RUNTIME_ROLE;
use crate::error::PgK8sLifecycleMigrationError;
use crate::migrations::{
    CURRENT_MIGRATION_VERSION, K8S_LIFECYCLE_MIGRATIONS, MIGRATION_LEDGER_BOOTSTRAP,
    MIGRATION_LOCK_KEY, MIGRATIONS_TABLE, registry_is_valid,
};
use crate::schema::{
    attest_complete, governed_table_count, ledger_exists, load_applied, validate_applied_prefix,
};
use crate::{schema_catalog, schema_phase::SchemaPhase};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PgK8sLifecycleMigrationReport {
    pub applied_versions: Vec<i64>,
    pub adopted_unversioned_schema: bool,
    pub current_version: i64,
}

#[derive(Clone, Debug)]
pub struct PgK8sLifecycleMigrator {
    pool: PgPool,
}

impl PgK8sLifecycleMigrator {
    pub async fn connect(database_url: &str) -> Result<Self, PgK8sLifecycleMigrationError> {
        if database_url.trim().is_empty() {
            return Err(PgK8sLifecycleMigrationError::MissingDatabaseUrl);
        }
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(database_url)
            .await
            .map_err(|error| PgK8sLifecycleMigrationError::Sqlx(error.to_string()))?;
        Ok(Self { pool })
    }

    #[must_use]
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn migrate(
        &self,
    ) -> Result<PgK8sLifecycleMigrationReport, PgK8sLifecycleMigrationError> {
        if !registry_is_valid() {
            return Err(PgK8sLifecycleMigrationError::InvalidRegistry);
        }
        let mut transaction = self.pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
            .execute(&mut *transaction)
            .await?;
        crate::catalog_connection::use_catalog_path(&mut transaction).await?;
        sqlx::query("SET LOCAL lock_timeout = '30s'")
            .execute(&mut *transaction)
            .await?;
        sqlx::query("SELECT pg_advisory_xact_lock($1)")
            .bind(MIGRATION_LOCK_KEY)
            .execute(&mut *transaction)
            .await?;

        crate::role_database_claim::attest_role_database_claim(&mut transaction, true).await?;
        let had_ledger = ledger_exists(&mut transaction).await?;
        let governed_tables = governed_table_count(&mut transaction).await?;
        let applied = if had_ledger {
            load_applied(&mut transaction).await?
        } else {
            Vec::new()
        };
        let next = validate_applied_prefix(&applied)?;
        let unversioned_adoption = next == 0 && governed_tables == 2;
        let phase = match (next, governed_tables, had_ledger) {
            (0, 2, ledger) => {
                let ledger_read = if ledger {
                    sqlx::query_scalar("SELECT has_table_privilege($1, $2, 'SELECT')")
                        .bind(RUNTIME_ROLE)
                        .bind(MIGRATIONS_TABLE)
                        .fetch_one(&mut *transaction)
                        .await?
                } else {
                    false
                };
                SchemaPhase::LegacyAdoption {
                    ledger,
                    ledger_read,
                }
            }
            (0, 0, false) => SchemaPhase::Empty,
            (0 | 1, 0, true) => SchemaPhase::RoleBoundary,
            (2, 2, true) => SchemaPhase::LegacyRepository,
            (3, 2, true) => SchemaPhase::PendingIntentRepository,
            _ => return Err(PgK8sLifecycleMigrationError::SchemaStateAmbiguous),
        };
        schema_catalog::attest_schema(&mut transaction, phase).await?;
        if !had_ledger {
            execute_statements(&mut transaction, MIGRATION_LEDGER_BOOTSTRAP).await?;
        }

        let mut applied_versions = Vec::new();
        if unversioned_adoption {
            for migration in &K8S_LIFECYCLE_MIGRATIONS[..2] {
                insert_ledger_row(&mut transaction, *migration).await?;
            }
        }
        let next = if unversioned_adoption { 2 } else { next };
        for migration in &K8S_LIFECYCLE_MIGRATIONS[next..] {
            execute_statements(&mut transaction, migration.sql()).await?;
            insert_ledger_row(&mut transaction, *migration).await?;
            applied_versions.push(migration.version());
        }
        if !had_ledger || unversioned_adoption {
            sqlx::query(&format!(
                "GRANT SELECT ON {MIGRATIONS_TABLE} TO {RUNTIME_ROLE}"
            ))
            .execute(&mut *transaction)
            .await?;
        }
        attest_complete(&mut transaction).await?;
        crate::role_database_claim::attest_role_database_claim(&mut transaction, false).await?;
        transaction.commit().await?;

        Ok(PgK8sLifecycleMigrationReport {
            applied_versions,
            adopted_unversioned_schema: unversioned_adoption,
            current_version: CURRENT_MIGRATION_VERSION,
        })
    }
}

async fn execute_statements(
    connection: &mut PgConnection,
    migration: &str,
) -> Result<(), sqlx::Error> {
    for statement in split_migration_statements(migration) {
        sqlx::query(&statement).execute(&mut *connection).await?;
    }
    Ok(())
}

async fn insert_ledger_row(
    connection: &mut PgConnection,
    migration: crate::migrations::PgK8sLifecycleMigration,
) -> Result<(), sqlx::Error> {
    sqlx::query(&format!(
        "INSERT INTO {MIGRATIONS_TABLE} (version, name, sha256) VALUES ($1, $2, $3)"
    ))
    .bind(migration.version())
    .bind(migration.name())
    .bind(migration.sha256())
    .execute(connection)
    .await?;
    Ok(())
}
