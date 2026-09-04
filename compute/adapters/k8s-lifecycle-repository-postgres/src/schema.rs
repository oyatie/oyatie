use sqlx::{PgConnection, Row};

use crate::error::{PgK8sLifecycleMigrationError, PgK8sLifecycleSchemaError};
use crate::migrations::{CURRENT_MIGRATION_VERSION, K8S_LIFECYCLE_MIGRATIONS, MIGRATIONS_TABLE};
use crate::{CLUSTERS_TABLE, OPERATIONS_TABLE, schema_catalog};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AppliedMigration {
    pub version: i64,
    pub name: String,
    pub sha256: String,
}

pub(crate) async fn ledger_exists(connection: &mut PgConnection) -> Result<bool, sqlx::Error> {
    let table: Option<String> = sqlx::query_scalar("SELECT to_regclass($1)::text")
        .bind(MIGRATIONS_TABLE)
        .fetch_one(&mut *connection)
        .await?;
    Ok(table.is_some())
}

pub(crate) async fn load_applied(
    connection: &mut PgConnection,
) -> Result<Vec<AppliedMigration>, sqlx::Error> {
    let rows = sqlx::query(&format!(
        "SELECT version, name, sha256 FROM {MIGRATIONS_TABLE} ORDER BY version"
    ))
    .fetch_all(&mut *connection)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(AppliedMigration {
                version: row.try_get("version")?,
                name: row.try_get("name")?,
                sha256: row.try_get("sha256")?,
            })
        })
        .collect()
}

pub(crate) fn validate_applied_prefix(
    applied: &[AppliedMigration],
) -> Result<usize, PgK8sLifecycleMigrationError> {
    let observed = applied
        .iter()
        .map(|migration| migration.version)
        .max()
        .unwrap_or_default();
    if observed > CURRENT_MIGRATION_VERSION {
        return Err(PgK8sLifecycleMigrationError::DatabaseAhead {
            observed,
            supported: CURRENT_MIGRATION_VERSION,
        });
    }
    for (stored, expected) in applied.iter().zip(K8S_LIFECYCLE_MIGRATIONS) {
        if stored.version != expected.version()
            || stored.name != expected.name()
            || stored.sha256 != expected.sha256()
        {
            return Err(PgK8sLifecycleMigrationError::AppliedMigrationDrift {
                version: stored.version,
            });
        }
    }
    if let Some(extra) = applied.get(K8S_LIFECYCLE_MIGRATIONS.len()) {
        return Err(PgK8sLifecycleMigrationError::AppliedMigrationDrift {
            version: extra.version,
        });
    }
    Ok(applied.len())
}

pub(crate) async fn attest_complete(
    connection: &mut PgConnection,
) -> Result<(), PgK8sLifecycleSchemaError> {
    if !ledger_exists(connection).await? {
        return Err(PgK8sLifecycleSchemaError::LedgerMissing);
    }
    let applied = load_applied(connection).await?;
    if applied.len() != K8S_LIFECYCLE_MIGRATIONS.len() {
        return Err(PgK8sLifecycleSchemaError::MigrationCountMismatch {
            expected: K8S_LIFECYCLE_MIGRATIONS.len(),
            observed: applied.len(),
        });
    }
    for (stored, expected) in applied.iter().zip(K8S_LIFECYCLE_MIGRATIONS) {
        if stored.version != expected.version()
            || stored.name != expected.name()
            || stored.sha256 != expected.sha256()
        {
            return Err(PgK8sLifecycleSchemaError::MigrationIdentityMismatch {
                version: stored.version,
            });
        }
    }
    schema_catalog::attest_schema(connection).await
}

pub(crate) async fn governed_table_count(
    connection: &mut PgConnection,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT count(*)::bigint FROM pg_class WHERE oid IN (to_regclass($1), to_regclass($2)) AND relkind = 'r'",
    )
    .bind(CLUSTERS_TABLE)
    .bind(OPERATIONS_TABLE)
    .fetch_one(connection)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applied_prefix_validation_distinguishes_future_and_drifted_state() {
        let exact: Vec<AppliedMigration> = K8S_LIFECYCLE_MIGRATIONS
            .iter()
            .map(|migration| AppliedMigration {
                version: migration.version(),
                name: migration.name().to_owned(),
                sha256: migration.sha256(),
            })
            .collect();
        assert_eq!(validate_applied_prefix(&[]), Ok(0));
        assert_eq!(validate_applied_prefix(&exact[..1]), Ok(1));
        assert_eq!(validate_applied_prefix(&exact), Ok(2));

        let mut future = exact.clone();
        future.push(AppliedMigration {
            version: 3,
            name: "future".to_owned(),
            sha256: "a".repeat(64),
        });
        assert_eq!(
            validate_applied_prefix(&future),
            Err(PgK8sLifecycleMigrationError::DatabaseAhead {
                observed: 3,
                supported: CURRENT_MIGRATION_VERSION
            })
        );

        let mut duplicate = exact.clone();
        duplicate.push(exact[1].clone());
        assert_eq!(
            validate_applied_prefix(&duplicate),
            Err(PgK8sLifecycleMigrationError::AppliedMigrationDrift { version: 2 })
        );

        let mut tampered = exact;
        tampered[0].name = "rewritten".to_owned();
        assert_eq!(
            validate_applied_prefix(&tampered),
            Err(PgK8sLifecycleMigrationError::AppliedMigrationDrift { version: 1 })
        );
    }
}
