#![allow(dead_code)]

use std::path::PathBuf;

use foundry_ontology_app::{AppState, Config, OperatorCredential, compose, router};
use foundry_records_draft::RecordsLog;
use foundry_records_sqlite_draft::SqliteRecordsLog;

mod session;
pub use session::*;

pub const TENANT: &str = "ten_acme";
const OPERATOR_TOKEN: &str = "operator-token-for-tests";
const FOREIGN_TOKEN: &str = "foreign-token-for-tests";
const ROLELESS_TOKEN: &str = "roleless-token-for-tests";

pub struct Fixture {
    action: PathBuf,
    denial: PathBuf,
    store: PathBuf,
}

impl Fixture {
    pub fn new(case: &str) -> Self {
        let stamp = format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock is after the epoch")
                .as_nanos()
        );
        let temp = std::env::temp_dir();
        let fixture = Self {
            action: temp.join(format!("foundry-facade-{case}-action-{stamp}.sqlite")),
            denial: temp.join(format!("foundry-facade-{case}-denial-{stamp}.sqlite")),
            store: temp.join(format!("foundry-facade-{case}-store-{stamp}.sqlite")),
        };
        fixture.remove_files();
        fixture
    }

    /// A process serving a DIFFERENT tenant from the one its operator
    /// credential names: the roster does not hold the credential's tenant,
    /// so the request is refused before the policy decision point is asked.
    /// Reachable in production through a config whose operator list and
    /// tenant roster disagree.
    pub fn config_with_unserved_operator_tenant(&self) -> Config {
        let mut config = self.config();
        config.tenants = vec!["ten_elsewhere".into()];
        config
    }

    pub fn unserved_session(&self) -> Session {
        Session {
            router: router(compose(&self.config_with_unserved_operator_tenant()).expect("boots")),
        }
    }

    pub fn config(&self) -> Config {
        Config {
            listen_addr: "127.0.0.1:0".into(),
            action_log: self.action.clone(),
            denial_log: self.denial.clone(),
            projection_store: self.store.clone(),
            tenants: vec![TENANT.into()],
            operators: vec![
                OperatorCredential {
                    token: OPERATOR_TOKEN.into(),
                    tenant_id: TENANT.into(),
                    principal_id: "prn_alice".into(),
                    roles: vec!["foundry-operator".into()],
                },
                OperatorCredential {
                    token: FOREIGN_TOKEN.into(),
                    tenant_id: "ten_other".into(),
                    principal_id: "prn_mallory".into(),
                    roles: vec!["foundry-operator".into()],
                },
                OperatorCredential {
                    token: ROLELESS_TOKEN.into(),
                    tenant_id: TENANT.into(),
                    principal_id: "prn_nobody".into(),
                    roles: Vec::new(),
                },
            ],
        }
    }

    /// A process serving BOTH fixture tenants, so the foreign credential is
    /// a served operator of its own tenant rather than a stranger.
    pub fn both_tenants_session(&self) -> Session {
        let mut config = self.config();
        config.tenants.push("ten_other".into());
        Session {
            router: router(compose(&config).expect("boots")),
        }
    }

    pub fn state(&self) -> AppState {
        compose(&self.config()).expect("the fixture must boot")
    }

    pub fn operator_token(&self) -> &'static str {
        OPERATOR_TOKEN
    }

    pub fn foreign_token(&self) -> &'static str {
        FOREIGN_TOKEN
    }

    pub fn roleless_token(&self) -> &'static str {
        ROLELESS_TOKEN
    }

    pub fn action_log_path(&self) -> std::path::PathBuf {
        self.action.clone()
    }

    pub fn projection_store_path(&self) -> std::path::PathBuf {
        self.store.clone()
    }

    pub fn denial_log_path(&self) -> std::path::PathBuf {
        self.denial.clone()
    }

    /// Removes the three stores and their SQLite WAL sidecars.
    fn remove_files(&self) {
        for path in [&self.action, &self.denial, &self.store] {
            for suffix in ["", "-wal", "-shm"] {
                let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
            }
        }
    }

    /// The action log's head, read from the durable store itself rather
    /// than from anything the process reports about itself.
    pub fn log_head(&self) -> u64 {
        SqliteRecordsLog::open(&self.action)
            .expect("the action log opens")
            .head(TENANT)
            .expect("head is readable")
    }

    pub fn denial_head(&self) -> u64 {
        SqliteRecordsLog::open(&self.denial)
            .expect("the denial trail opens")
            .head(TENANT)
            .expect("head is readable")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.remove_files();
    }
}
