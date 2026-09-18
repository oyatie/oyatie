//! Store handle: a refusing shim without the `fdb` feature, a connected
//! database handle with it. Port bindings land with the adapter proper.

use std::path::PathBuf;

use mail_kernel::Error;

/// Why the default build refuses. `mail_kernel::Error::Unavailable` carries no
/// payload, so callers that want the reason read this constant.
pub const UNAVAILABLE_REASON: &str = "built without fdb feature";

/// Connection parameters. `cell` prefixes every key this store writes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FdbConfig {
    pub cluster_file: PathBuf,
    pub cell: String,
}

/// Handle on one FoundationDB cell. Constructing it is the only operation the
/// shim exposes; every port method arrives with the adapter.
pub struct FdbStore {
    config: FdbConfig,
    #[cfg(feature = "fdb")]
    database: crate::ffi::Database,
}

impl FdbStore {
    /// Refusing constructor: the crate was compiled without `fdb`.
    #[cfg(not(feature = "fdb"))]
    pub fn open(config: FdbConfig) -> Result<Self, Error> {
        let _ = config;
        Err(Error::Unavailable)
    }

    /// Selects the client API version, starts the network thread once per
    /// process and opens the cluster named by `config.cluster_file`.
    #[cfg(feature = "fdb")]
    pub fn open(config: FdbConfig) -> Result<Self, Error> {
        let database =
            crate::ffi::Database::open(&config.cluster_file).map_err(|_| Error::Unavailable)?;
        Ok(Self { config, database })
    }

    /// `Some(reason)` when this build can never open a store.
    pub const fn refusal() -> Option<&'static str> {
        if cfg!(feature = "fdb") {
            None
        } else {
            Some(UNAVAILABLE_REASON)
        }
    }

    pub const fn config(&self) -> &FdbConfig {
        &self.config
    }

    #[cfg(feature = "fdb")]
    pub const fn database(&self) -> &crate::ffi::Database {
        &self.database
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(feature = "fdb"))]
    fn default_build_refuses_to_open() {
        let config = FdbConfig {
            cluster_file: PathBuf::from("/nonexistent/fdb.cluster"),
            cell: "cell-a".into(),
        };
        assert!(matches!(FdbStore::open(config), Err(Error::Unavailable)));
        assert_eq!(FdbStore::refusal(), Some(UNAVAILABLE_REASON));
    }

    #[test]
    #[cfg(feature = "fdb")]
    fn fdb_build_has_no_static_refusal() {
        assert_eq!(FdbStore::refusal(), None);
    }
}
