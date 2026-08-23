//! Storage subsystem: bring-up principal store behind the repository port.
//!
//! The service composes [`WorkloadPrincipalRepository`] /
//! [`RevocationDenylist`] PORT implementations. For single-node bring-up the
//! in-memory reference adapters are seeded from a deployment-mounted JSON
//! file; the durable store (sqlx over the owned data SQL interface)
//! arrives behind the SAME ports via the G03 persistence lane — this module's
//! seed format and the server wiring do not change at that cutover.

use std::fmt;

use serde::Deserialize;

use iam_identity_workload_app::{
    InMemoryRevocationDenylist, InMemoryWorkloadPrincipalRepository, RepositoryError,
    RevocationDenylist, WorkloadPrincipalRepository,
};
use iam_identity_workload_domain::{WorkloadIdentityError, WorkloadPrincipal, WorkloadState};

/// One seeded principal. `state` is one of `provisioned`, `active`,
/// `suspended`, `retired` (default `active`).
#[derive(Debug, Deserialize)]
struct SeedPrincipal {
    tenant_id: String,
    workload_id: String,
    owning_capability: String,
    #[serde(default)]
    scopes: Vec<String>,
    #[serde(default = "default_seed_state")]
    state: String,
}

fn default_seed_state() -> String {
    "active".into()
}

/// A principal seed document that could not be loaded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SeedError {
    /// The document is not valid JSON of the expected shape.
    Malformed(String),
    /// A seeded id or scope failed domain validation.
    Domain(WorkloadIdentityError),
    /// A workload id appears twice in the seed.
    Duplicate(String),
    /// A seed entry names an unknown lifecycle state.
    UnknownState { workload_id: String, state: String },
    /// The backing store rejected a write.
    Repository(RepositoryError),
}

impl fmt::Display for SeedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(detail) => write!(f, "malformed principal seed: {detail}"),
            Self::Domain(err) => write!(f, "invalid principal seed: {err}"),
            Self::Duplicate(id) => write!(f, "duplicate workload id in seed: {id}"),
            Self::UnknownState { workload_id, state } => {
                write!(f, "unknown seed state {state:?} for {workload_id}")
            }
            Self::Repository(err) => write!(f, "seed store write failed: {}", err.detail()),
        }
    }
}

impl std::error::Error for SeedError {}

impl From<WorkloadIdentityError> for SeedError {
    fn from(value: WorkloadIdentityError) -> Self {
        Self::Domain(value)
    }
}

impl From<RepositoryError> for SeedError {
    fn from(value: RepositoryError) -> Self {
        Self::Repository(value)
    }
}

/// Build the bring-up repository + denylist pair from a JSON seed document
/// (an array of principals). Suspended/retired seeds are also written to the
/// denylist, mirroring the lifecycle use-cases' revocation behaviour.
///
/// # Errors
/// Returns [`SeedError`] on malformed JSON, invalid ids/scopes, duplicate
/// workload ids, or an unknown lifecycle state.
pub fn seed_from_json(
    document: &str,
) -> Result<
    (
        InMemoryWorkloadPrincipalRepository,
        InMemoryRevocationDenylist,
    ),
    SeedError,
> {
    let seeds: Vec<SeedPrincipal> =
        serde_json::from_str(document).map_err(|e| SeedError::Malformed(e.to_string()))?;
    let mut repository = InMemoryWorkloadPrincipalRepository::new();
    let mut denylist = InMemoryRevocationDenylist::new();
    for seed in seeds {
        let mut principal =
            WorkloadPrincipal::provision(seed.tenant_id, seed.workload_id, seed.owning_capability)?;
        for scope in seed.scopes {
            principal.grant_scope(scope)?;
        }
        match seed.state.as_str() {
            "provisioned" => {}
            "active" => principal.transition_to(WorkloadState::Active)?,
            "suspended" => {
                principal.transition_to(WorkloadState::Active)?;
                principal.transition_to(WorkloadState::Suspended)?;
                denylist.revoke(principal.workload_id())?;
            }
            "retired" => {
                principal.transition_to(WorkloadState::Retired)?;
                denylist.revoke(principal.workload_id())?;
            }
            other => {
                return Err(SeedError::UnknownState {
                    workload_id: principal.workload_id().to_string(),
                    state: other.to_string(),
                });
            }
        }
        if repository.load(principal.workload_id())?.is_some() {
            return Err(SeedError::Duplicate(principal.workload_id().to_string()));
        }
        repository.save(&principal)?;
    }
    Ok((repository, denylist))
}

#[cfg(test)]
mod tests {
    use super::*;

    use iam_identity_workload_domain::WorkloadId;

    const SEED: &str = r#"[
        {"tenant_id":"ten_acme","workload_id":"wl_secrets_sync",
         "owning_capability":"cap.cloud.kms","scopes":["cloud.kms.decrypt"]},
        {"tenant_id":"ten_acme","workload_id":"wl_paused",
         "owning_capability":"cap.cloud.kms","state":"suspended"},
        {"tenant_id":"ten_acme","workload_id":"wl_gone",
         "owning_capability":"cap.cloud.kms","state":"retired"},
        {"tenant_id":"ten_acme","workload_id":"wl_new",
         "owning_capability":"cap.cloud.kms","state":"provisioned"}
    ]"#;

    fn load(repo: &InMemoryWorkloadPrincipalRepository, id: &str) -> WorkloadPrincipal {
        repo.load(&WorkloadId::new(id).expect("id"))
            .expect("store readable")
            .expect("principal present")
    }

    #[test]
    fn seeds_all_lifecycle_states() {
        let (repo, denylist) = seed_from_json(SEED).expect("seed");
        assert_eq!(
            load(&repo, "wl_secrets_sync").state(),
            WorkloadState::Active
        );
        assert!(load(&repo, "wl_secrets_sync").has_scope("cloud.kms.decrypt"));
        assert_eq!(load(&repo, "wl_paused").state(), WorkloadState::Suspended);
        assert_eq!(load(&repo, "wl_gone").state(), WorkloadState::Retired);
        assert_eq!(load(&repo, "wl_new").state(), WorkloadState::Provisioned);
        let revoked = |id: &str| {
            denylist
                .is_revoked(&WorkloadId::new(id).expect("id"))
                .expect("denylist readable")
        };
        assert!(revoked("wl_paused"));
        assert!(revoked("wl_gone"));
        assert!(!revoked("wl_secrets_sync"));
    }

    #[test]
    fn refuses_duplicate_workload_id() {
        let seed = r#"[
            {"tenant_id":"ten_a","workload_id":"wl_dup","owning_capability":"cap.x.y"},
            {"tenant_id":"ten_a","workload_id":"wl_dup","owning_capability":"cap.x.y"}
        ]"#;
        assert_eq!(
            seed_from_json(seed),
            Err(SeedError::Duplicate("wl_dup".into()))
        );
    }

    #[test]
    fn refuses_unknown_state() {
        let seed = r#"[{"tenant_id":"ten_a","workload_id":"wl_x",
            "owning_capability":"cap.x.y","state":"dormant"}]"#;
        assert!(matches!(
            seed_from_json(seed),
            Err(SeedError::UnknownState { .. })
        ));
    }

    #[test]
    fn refuses_invalid_domain_ids() {
        let seed = r#"[{"tenant_id":"acme","workload_id":"wl_x","owning_capability":"cap.x.y"}]"#;
        assert!(matches!(seed_from_json(seed), Err(SeedError::Domain(_))));
    }

    #[test]
    fn refuses_malformed_json() {
        assert!(matches!(
            seed_from_json("not json"),
            Err(SeedError::Malformed(_))
        ));
    }
}
