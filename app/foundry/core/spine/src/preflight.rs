//! Full-prefix publication preflight: every live operation is a read.
use crate::writethrough::{designations, folded_entry};
use crate::{ProjectionState, apply_sealed};
use data_ontology_kernel::OntologyEngine;
use foundry_projection_draft::{MemoryProjectionStore, ProjectionStore, ProjectionStoreError};
use foundry_records_draft::SealedEnvelope;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectionPreflightError {
    InvalidLog {
        ordinal: u64,
    },
    LogHead {
        expected: u64,
        found: u64,
    },
    ProjectionHead {
        expected: u64,
        found: u64,
    },
    Read(ProjectionStoreError),
    Entry {
        ordinal: u64,
        error: ProjectionStoreError,
    },
    Scratch {
        ordinal: u64,
        error: ProjectionStoreError,
    },
}

pub fn preflight_projection(
    tenant: &str,
    registry: &OntologyEngine,
    store: &dyn ProjectionStore,
    log: &[SealedEnvelope],
    log_head: u64,
) -> Result<ProjectionState, ProjectionPreflightError> {
    use ProjectionPreflightError as Error;
    for (index, entry) in log.iter().enumerate() {
        if entry.envelope.tenant_id != tenant || entry.receipt.ordinal != index as u64 + 1 {
            return Err(Error::InvalidLog {
                ordinal: entry.receipt.ordinal,
            });
        }
    }
    let found = log.last().map_or(0, |entry| entry.receipt.ordinal);
    if found != log_head {
        return Err(Error::LogHead {
            expected: log_head,
            found,
        });
    }
    let found = store.applied_head(tenant).map_err(Error::Read)?;
    if found != log_head {
        return Err(Error::ProjectionHead {
            expected: log_head,
            found,
        });
    }
    let mut state = ProjectionState::new(tenant, registry);
    let mut scratch = MemoryProjectionStore::default();
    for sealed in log {
        let fold = apply_sealed(&mut state, sealed);
        let entry = folded_entry(&state, sealed, &fold);
        let ordinal = sealed.receipt.ordinal;
        scratch
            .apply(
                entry.clone(),
                &designations(&state, &sealed.envelope.object_ref),
            )
            .map_err(|error| Error::Scratch { ordinal, error })?;
        store
            .verify_entry(&entry)
            .map_err(|error| Error::Entry { ordinal, error })?;
    }
    Ok(state)
}
