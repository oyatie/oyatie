use compute_k8s_api::CloudComputeK8sLifecycleRepositoryError;
use shared_postgres_command_kernel::RlsEnforceabilityError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PgK8sLifecycleConnectError {
    MissingDatabaseUrl,
    Sqlx(String),
    Schema(PgK8sLifecycleSchemaError),
    ServingPrincipalNotAllowed { role: String },
    ServingRoleGraphMismatch,
    PrivilegedAuthorityPresent { role: String },
    RlsUnenforceable { role: String },
    RlsRoleMismatch { role: String, expected: String },
    RlsNotForcedOnTable { table: String },
}

impl core::fmt::Display for PgK8sLifecycleConnectError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingDatabaseUrl => write!(f, "database url is empty"),
            Self::Sqlx(detail) => write!(f, "PostgreSQL connection validation failed: {detail}"),
            Self::Schema(error) => write!(f, "lifecycle schema attestation failed: {error}"),
            Self::ServingPrincipalNotAllowed { role } => write!(
                f,
                "connected role '{role}' is not an admitted lifecycle serving principal"
            ),
            Self::ServingRoleGraphMismatch => {
                f.write_str("lifecycle serving-role membership graph does not match")
            }
            Self::PrivilegedAuthorityPresent { role } => write!(
                f,
                "runtime role '{role}' carries authority beyond the lifecycle DML contract"
            ),
            Self::RlsUnenforceable { role } => {
                write!(f, "runtime role '{role}' can bypass row-level security")
            }
            Self::RlsRoleMismatch { role, expected } => write!(
                f,
                "runtime role '{role}' is not a member of policy role '{expected}'"
            ),
            Self::RlsNotForcedOnTable { table } => {
                write!(f, "governed table '{table}' is not protected by FORCE RLS")
            }
        }
    }
}

impl std::error::Error for PgK8sLifecycleConnectError {}

impl From<PgK8sLifecycleSchemaError> for PgK8sLifecycleConnectError {
    fn from(error: PgK8sLifecycleSchemaError) -> Self {
        Self::Schema(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PgK8sLifecycleRoleDatabaseClaimError {
    Unclaimed,
    ForeignOrUnresolvedDatabase,
    UnsupportedSharedDependency,
}

impl core::fmt::Display for PgK8sLifecycleRoleDatabaseClaimError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Unclaimed => "runtime role has no database claim",
            Self::ForeignOrUnresolvedDatabase => {
                "runtime role has a foreign or unresolved database claim"
            }
            Self::UnsupportedSharedDependency => {
                "runtime role has an unsupported shared dependency"
            }
        })
    }
}

impl std::error::Error for PgK8sLifecycleRoleDatabaseClaimError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PgK8sLifecycleSchemaError {
    LedgerMissing,
    MigrationCountMismatch { expected: usize, observed: usize },
    MigrationIdentityMismatch { version: i64 },
    RuntimeRoleContract,
    RuntimeRoleDatabaseClaim(PgK8sLifecycleRoleDatabaseClaimError),
    NamespaceContract,
    OwnershipContract,
    ColumnContract,
    ExpressionDependencyContract,
    ConstraintContract,
    IndexContract,
    PolicyContract,
    GrantContract,
    Sqlx(String),
}

impl core::fmt::Display for PgK8sLifecycleSchemaError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::LedgerMissing => f.write_str("schema migration ledger is missing"),
            Self::MigrationCountMismatch { expected, observed } => write!(
                f,
                "migration ledger count mismatch: expected {expected}, observed {observed}"
            ),
            Self::MigrationIdentityMismatch { version } => {
                write!(f, "migration identity mismatch at version {version}")
            }
            Self::RuntimeRoleContract => f.write_str("runtime role contract does not match"),
            Self::RuntimeRoleDatabaseClaim(error) => core::fmt::Display::fmt(error, f),
            Self::NamespaceContract => f.write_str("schema namespace contract does not match"),
            Self::OwnershipContract => {
                f.write_str("schema separation-of-duties ownership contract does not match")
            }
            Self::ColumnContract => f.write_str("column contract does not match"),
            Self::ExpressionDependencyContract => {
                f.write_str("schema expression has a noncatalog or unresolved dependency")
            }
            Self::ConstraintContract => f.write_str("constraint contract does not match"),
            Self::IndexContract => f.write_str("index contract does not match"),
            Self::PolicyContract => f.write_str("tenant policy contract does not match"),
            Self::GrantContract => f.write_str("runtime grant contract does not match"),
            Self::Sqlx(detail) => write!(f, "schema probe failed: {detail}"),
        }
    }
}

impl std::error::Error for PgK8sLifecycleSchemaError {}

impl From<sqlx::Error> for PgK8sLifecycleSchemaError {
    fn from(error: sqlx::Error) -> Self {
        Self::Sqlx(error.to_string())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PgK8sLifecycleMigrationError {
    MissingDatabaseUrl,
    InvalidRegistry,
    DatabaseAhead { observed: i64, supported: i64 },
    AppliedMigrationDrift { version: i64 },
    SchemaStateAmbiguous,
    Schema(PgK8sLifecycleSchemaError),
    Sqlx(String),
}

impl core::fmt::Display for PgK8sLifecycleMigrationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingDatabaseUrl => f.write_str("migration database url is empty"),
            Self::InvalidRegistry => f.write_str("migration registry is not contiguous and unique"),
            Self::DatabaseAhead {
                observed,
                supported,
            } => write!(
                f,
                "database migration version {observed} is ahead of supported version {supported}"
            ),
            Self::AppliedMigrationDrift { version } => {
                write!(
                    f,
                    "applied migration {version} differs from the immutable registry"
                )
            }
            Self::SchemaStateAmbiguous => {
                f.write_str("lifecycle schema state is partial or ambiguous")
            }
            Self::Schema(error) => write!(f, "post-migration schema attestation failed: {error}"),
            Self::Sqlx(detail) => write!(f, "migration execution failed: {detail}"),
        }
    }
}

impl std::error::Error for PgK8sLifecycleMigrationError {}

impl From<PgK8sLifecycleSchemaError> for PgK8sLifecycleMigrationError {
    fn from(error: PgK8sLifecycleSchemaError) -> Self {
        Self::Schema(error)
    }
}

impl From<sqlx::Error> for PgK8sLifecycleMigrationError {
    fn from(error: sqlx::Error) -> Self {
        Self::Sqlx(error.to_string())
    }
}

impl From<RlsEnforceabilityError> for PgK8sLifecycleConnectError {
    fn from(error: RlsEnforceabilityError) -> Self {
        match error {
            RlsEnforceabilityError::Unenforceable { role } => Self::RlsUnenforceable { role },
            RlsEnforceabilityError::RoleMismatch { role, expected } => {
                Self::RlsRoleMismatch { role, expected }
            }
            RlsEnforceabilityError::RlsNotForced { table, .. }
            | RlsEnforceabilityError::GovernedTableMissing { table } => {
                Self::RlsNotForcedOnTable { table }
            }
            RlsEnforceabilityError::RoleSwitchInEffect { .. }
            | RlsEnforceabilityError::ProbeFailed { .. } => Self::Sqlx(error.to_string()),
        }
    }
}

pub(crate) fn validate_database_url(database_url: &str) -> Result<(), PgK8sLifecycleConnectError> {
    if database_url.trim().is_empty() {
        return Err(PgK8sLifecycleConnectError::MissingDatabaseUrl);
    }
    Ok(())
}

pub(crate) fn unavailable<T>(_error: T) -> CloudComputeK8sLifecycleRepositoryError {
    CloudComputeK8sLifecycleRepositoryError::Unavailable
}

pub(crate) fn integrity<T>(_error: T) -> CloudComputeK8sLifecycleRepositoryError {
    CloudComputeK8sLifecycleRepositoryError::IntegrityViolation
}
