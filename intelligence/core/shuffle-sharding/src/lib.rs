//! Deterministic shuffle-sharding for tenant-to-cell placement.
//!
//! This crate intentionally contains no service runtime, storage adapter, or
//! network client. It is the pure algorithmic surface that `tenancy` can call
//! during tenant provisioning while `iac-app` and `observability` own the
//! mutable cell topology and live health inputs.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

/// A cell candidate supplied by the caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellCandidate {
    /// Stable cell identifier from the iac-app cell registry.
    pub cell_id: String,
    /// Regional pack label used to keep assignments residency-safe.
    pub pack: String,
    /// Runtime region label for optional regional narrowing.
    pub region: String,
    /// Whether this cell is eligible for new tenant placement.
    pub accepts_new_tenants: bool,
}

/// Request for deterministic tenant shuffle-shard selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShuffleShardRequest {
    /// Stable tenant identifier. Raw user identifiers do not belong here.
    pub tenant_id: String,
    /// Number of distinct cells to select.
    pub shard_width: usize,
    /// Salt/version string controlled by the caller for deliberate rebalancing.
    pub placement_salt: String,
    /// Optional pack constraint; when set, only matching candidates are eligible.
    pub required_pack: Option<String>,
    /// Optional region constraint; when set, only matching candidates are eligible.
    pub required_region: Option<String>,
    /// Candidate cells read from the iac-app-owned registry.
    pub candidates: Vec<CellCandidate>,
}

/// Deterministic selection result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShuffleShard {
    /// Tenant used for selection.
    pub tenant_id: String,
    /// Salt/version used for selection.
    pub placement_salt: String,
    /// Selected cell identifiers in deterministic rank order.
    pub cell_ids: Vec<String>,
}

/// Validation and selection failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShuffleShardError {
    EmptyTenantId,
    EmptyPlacementSalt,
    ZeroShardWidth,
    EmptyCellId,
    EmptyPack,
    EmptyRegion,
    DuplicateCellId(String),
    NotEnoughEligibleCells { required: usize, available: usize },
}

/// Selects a deterministic shuffle shard for one tenant.
///
/// The caller owns topology freshness and health filtering. This function only
/// validates candidate shape, filters by the optional pack/region constraints,
/// ranks each eligible cell with a stable hash of tenant, salt, and cell id,
/// then returns the first `shard_width` unique cells.
///
/// # Example
///
/// ```
/// use intelligence_shuffle_sharding::{
///     CellCandidate, ShuffleShardRequest, select_shuffle_shard,
/// };
///
/// let request = ShuffleShardRequest {
///     tenant_id: "ten_acme".to_string(),
///     shard_width: 2,
///     placement_salt: "cell-assignment-v1".to_string(),
///     required_pack: Some("pack-kr".to_string()),
///     required_region: None,
///     candidates: vec![
///         CellCandidate {
///             cell_id: "kr-cell-001".to_string(),
///             pack: "pack-kr".to_string(),
///             region: "ap-northeast-2".to_string(),
///             accepts_new_tenants: true,
///         },
///         CellCandidate {
///             cell_id: "kr-cell-002".to_string(),
///             pack: "pack-kr".to_string(),
///             region: "ap-northeast-2".to_string(),
///             accepts_new_tenants: true,
///         },
///     ],
/// };
///
/// let shard = select_shuffle_shard(request)?;
/// assert_eq!(shard.cell_ids.len(), 2);
/// # Ok::<(), intelligence_shuffle_sharding::ShuffleShardError>(())
/// ```
pub fn select_shuffle_shard(
    request: ShuffleShardRequest,
) -> Result<ShuffleShard, ShuffleShardError> {
    validate_request(&request)?;

    let eligible = eligible_cells(&request);
    if eligible.len() < request.shard_width {
        return Err(ShuffleShardError::NotEnoughEligibleCells {
            required: request.shard_width,
            available: eligible.len(),
        });
    }

    let mut ranked = eligible
        .into_iter()
        .map(|candidate| {
            (
                rank_cell(
                    &request.tenant_id,
                    &request.placement_salt,
                    &candidate.cell_id,
                ),
                candidate.cell_id.clone(),
            )
        })
        .collect::<Vec<_>>();

    ranked.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));

    Ok(ShuffleShard {
        tenant_id: request.tenant_id,
        placement_salt: request.placement_salt,
        cell_ids: ranked
            .into_iter()
            .take(request.shard_width)
            .map(|(_, cell_id)| cell_id)
            .collect(),
    })
}

fn validate_request(request: &ShuffleShardRequest) -> Result<(), ShuffleShardError> {
    if request.tenant_id.trim().is_empty() {
        return Err(ShuffleShardError::EmptyTenantId);
    }
    if request.placement_salt.trim().is_empty() {
        return Err(ShuffleShardError::EmptyPlacementSalt);
    }
    if request.shard_width == 0 {
        return Err(ShuffleShardError::ZeroShardWidth);
    }

    let mut seen_cell_ids = BTreeSet::new();
    for candidate in &request.candidates {
        validate_candidate(candidate)?;
        if !seen_cell_ids.insert(candidate.cell_id.clone()) {
            return Err(ShuffleShardError::DuplicateCellId(
                candidate.cell_id.clone(),
            ));
        }
    }

    Ok(())
}

fn validate_candidate(candidate: &CellCandidate) -> Result<(), ShuffleShardError> {
    if candidate.cell_id.trim().is_empty() {
        return Err(ShuffleShardError::EmptyCellId);
    }
    if candidate.pack.trim().is_empty() {
        return Err(ShuffleShardError::EmptyPack);
    }
    if candidate.region.trim().is_empty() {
        return Err(ShuffleShardError::EmptyRegion);
    }
    Ok(())
}

fn eligible_cells(request: &ShuffleShardRequest) -> Vec<&CellCandidate> {
    request
        .candidates
        .iter()
        .filter(|candidate| candidate.accepts_new_tenants)
        .filter(|candidate| {
            request
                .required_pack
                .as_ref()
                .is_none_or(|pack| candidate.pack == *pack)
        })
        .filter(|candidate| {
            request
                .required_region
                .as_ref()
                .is_none_or(|region| candidate.region == *region)
        })
        .collect()
}

fn rank_cell(tenant_id: &str, placement_salt: &str, cell_id: &str) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv1a_write(hash, tenant_id.as_bytes());
    hash = fnv1a_write(hash, &[0]);
    hash = fnv1a_write(hash, placement_salt.as_bytes());
    hash = fnv1a_write(hash, &[0]);
    hash = fnv1a_write(hash, cell_id.as_bytes());
    splitmix64_final(hash)
}

fn fnv1a_write(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn splitmix64_final(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
