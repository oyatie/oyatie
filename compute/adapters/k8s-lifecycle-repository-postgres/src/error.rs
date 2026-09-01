use compute_k8s_api::CloudComputeK8sLifecycleRepositoryError;
use shared_postgres_command_kernel::RlsEnforceabilityError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PgK8sLifecycleConnectError {
    MissingDatabaseUrl,
    Sqlx(String),
    RlsUnenforceable { role: String },
    RlsRoleMismatch { role: String, expected: String },
    RlsNotForcedOnTable { table: String },
}

impl core::fmt::Display for PgK8sLifecycleConnectError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingDatabaseUrl => write!(f, "database url is empty"),
            Self::Sqlx(detail) => write!(f, "PostgreSQL connection validation failed: {detail}"),
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
