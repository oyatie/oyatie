//! Env gate for the live workload harness. Mailbox/messenger slices are
//! not in this tree; community live coverage lives on those crates.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::env;

use shared_postgres_command_adapter_sqlx::{
    SqlxPostgresCommandError, SqlxPostgresConnectionConfig,
};
use shared_postgres_command_kernel::PostgresPoolConfig;

const WORKLOAD_LIVE_ENABLE_ENV: &str = "OYATIE_BACKBONE_LIVE_WORKLOAD_POSTGRES";
const WORKLOAD_POSTGRES_DATABASE_URL_ENV: &str = "OYATIE_BACKBONE_WORKLOAD_POSTGRES_URL";
const WORKLOAD_POSTGRES_APP_DATABASE_URL_ENV: &str = "OYATIE_BACKBONE_WORKLOAD_POSTGRES_APP_URL";
const WORKLOAD_POSTGRES_REQUIRE_TLS_ENV: &str = "OYATIE_BACKBONE_WORKLOAD_POSTGRES_REQUIRE_TLS";
const WORKLOAD_HARNESS_APPLICATION_NAME: &str = "oyatie-live-workload-harness";

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkloadLiveConfig {
    database_url: String,
    app_database_url: String,
    require_tls: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum WorkloadLiveError {
    Disabled {
        enable_env: &'static str,
    },
    MissingDatabaseUrl {
        database_url_env: &'static str,
    },
    MissingAppDatabaseUrl {
        app_database_url_env: &'static str,
    },
    InvalidBooleanEnv {
        env_name: &'static str,
        value: String,
    },
    Config(SqlxPostgresCommandError),
}

impl From<SqlxPostgresCommandError> for WorkloadLiveError {
    fn from(error: SqlxPostgresCommandError) -> Self {
        Self::Config(error)
    }
}

impl WorkloadLiveConfig {
    fn from_env_map(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, WorkloadLiveError> {
        let enabled = lookup(WORKLOAD_LIVE_ENABLE_ENV)
            .as_deref()
            .map(|value| parse_env_bool(WORKLOAD_LIVE_ENABLE_ENV, value))
            .transpose()?
            .unwrap_or(false);
        if !enabled {
            return Err(WorkloadLiveError::Disabled {
                enable_env: WORKLOAD_LIVE_ENABLE_ENV,
            });
        }
        let database_url = lookup(WORKLOAD_POSTGRES_DATABASE_URL_ENV).ok_or(
            WorkloadLiveError::MissingDatabaseUrl {
                database_url_env: WORKLOAD_POSTGRES_DATABASE_URL_ENV,
            },
        )?;
        let app_database_url = lookup(WORKLOAD_POSTGRES_APP_DATABASE_URL_ENV).ok_or(
            WorkloadLiveError::MissingAppDatabaseUrl {
                app_database_url_env: WORKLOAD_POSTGRES_APP_DATABASE_URL_ENV,
            },
        )?;
        let require_tls = lookup(WORKLOAD_POSTGRES_REQUIRE_TLS_ENV)
            .as_deref()
            .map(|value| parse_env_bool(WORKLOAD_POSTGRES_REQUIRE_TLS_ENV, value))
            .transpose()?
            .unwrap_or(true);
        let config = Self {
            database_url,
            app_database_url,
            require_tls,
        };
        config.connection_config()?;
        config.app_connection_config()?;
        Ok(config)
    }

    fn connection_config(&self) -> Result<SqlxPostgresConnectionConfig, SqlxPostgresCommandError> {
        SqlxPostgresConnectionConfig::new(
            self.database_url.clone(),
            workload_pool_config(self.require_tls)?,
        )
    }

    fn app_connection_config(
        &self,
    ) -> Result<SqlxPostgresConnectionConfig, SqlxPostgresCommandError> {
        SqlxPostgresConnectionConfig::new(
            self.app_database_url.clone(),
            workload_pool_config(self.require_tls)?,
        )
    }
}

fn workload_pool_config(require_tls: bool) -> Result<PostgresPoolConfig, SqlxPostgresCommandError> {
    PostgresPoolConfig::new(
        WORKLOAD_HARNESS_APPLICATION_NAME,
        4,
        1_000,
        5_000,
        require_tls,
    )
    .map_err(SqlxPostgresCommandError::Kernel)
}

fn parse_env_bool(env_name: &'static str, value: &str) -> Result<bool, WorkloadLiveError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(WorkloadLiveError::InvalidBooleanEnv {
            env_name,
            value: value.to_string(),
        }),
    }
}

#[test]
fn workload_live_config_is_env_gated_and_requires_app_url() {
    let _ = env::var(WORKLOAD_LIVE_ENABLE_ENV);
    assert_eq!(
        WorkloadLiveConfig::from_env_map(|_| None),
        Err(WorkloadLiveError::Disabled {
            enable_env: WORKLOAD_LIVE_ENABLE_ENV,
        })
    );

    let missing_app = WorkloadLiveConfig::from_env_map(|name| match name {
        WORKLOAD_LIVE_ENABLE_ENV => Some("true".to_string()),
        WORKLOAD_POSTGRES_DATABASE_URL_ENV => {
            Some("postgres://setup:secret@localhost/workloads?sslmode=require".to_string())
        }
        _ => None,
    });
    assert_eq!(
        missing_app,
        Err(WorkloadLiveError::MissingAppDatabaseUrl {
            app_database_url_env: WORKLOAD_POSTGRES_APP_DATABASE_URL_ENV,
        })
    );

    let config = WorkloadLiveConfig::from_env_map(|name| match name {
        WORKLOAD_LIVE_ENABLE_ENV => Some("true".to_string()),
        WORKLOAD_POSTGRES_DATABASE_URL_ENV => {
            Some("postgres://setup:secret@localhost/workloads?sslmode=require".to_string())
        }
        WORKLOAD_POSTGRES_APP_DATABASE_URL_ENV => {
            Some("postgres://app:secret@localhost/workloads?sslmode=require".to_string())
        }
        _ => None,
    })
    .unwrap();
    assert_eq!(
        config.connection_config().unwrap().database_url,
        "postgres://setup:secret@localhost/workloads?sslmode=require"
    );
    assert_eq!(
        config.app_connection_config().unwrap().database_url,
        "postgres://app:secret@localhost/workloads?sslmode=require"
    );
}
