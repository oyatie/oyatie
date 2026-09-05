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

const ADOPTABLE_UNVERSIONED_SCHEMA_VERSION: usize = 2;

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
        if !had_ledger && governed_tables == 1 {
            return Err(PgK8sLifecycleMigrationError::SchemaStateAmbiguous);
        }
        if !had_ledger {
            execute_statements(&mut transaction, MIGRATION_LEDGER_BOOTSTRAP).await?;
        }

        let applied = load_applied(&mut transaction).await?;
        let next = validate_applied_prefix(&applied)?;
        let unversioned_adoption = next == 0
            && governed_tables == 2
            && K8S_LIFECYCLE_MIGRATIONS.len() == ADOPTABLE_UNVERSIONED_SCHEMA_VERSION;
        let expected_tables = next.checked_sub(1).map_or(0, |index| {
            K8S_LIFECYCLE_MIGRATIONS[index].governed_table_count_after()
        });
        if !unversioned_adoption && governed_tables != expected_tables {
            return Err(PgK8sLifecycleMigrationError::SchemaStateAmbiguous);
        }

        let mut applied_versions = Vec::new();
        if unversioned_adoption {
            for migration in K8S_LIFECYCLE_MIGRATIONS {
                insert_ledger_row(&mut transaction, *migration).await?;
            }
        } else {
            for migration in &K8S_LIFECYCLE_MIGRATIONS[next..] {
                execute_statements(&mut transaction, migration.sql()).await?;
                insert_ledger_row(&mut transaction, *migration).await?;
                applied_versions.push(migration.version());
            }
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
