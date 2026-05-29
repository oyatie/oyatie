//! Reserved, committed-use, and spot capacity contracts (M03-P02-IP-003).
//!
//! Pure I/O-free types. Provider-specific discount or spot-interruption
//! APIs live in adapter crates; this module enforces structural invariants only.

// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use crate::{CapacityClass, RegionId};

// ── ReservedCapacity ─────────────────────────────────────────────────────────

/// A pre-purchased capacity block tied to a region and class.
///
/// Reserved capacity guarantees availability but must be validated against the
/// owning region's quota before activation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReservedCapacity {
    pub id: ReservedCapacityId, // data_class: INTERNAL_ONLY
    pub region: RegionId,       // data_class: INTERNAL_ONLY
    pub class: CapacityClass,   // data_class: INTERNAL_ONLY
    pub reserved_units: u64,    // data_class: INTERNAL_ONLY
    pub term: ReservationTerm,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReservedCapacityId(pub String); // data_class: INTERNAL_ONLY

/// Billing term for a reserved-capacity block.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ReservationTerm {
    OneYear,
    ThreeYear,
}

impl ReservationTerm {
    pub fn months(self) -> u32 {
        match self {
            Self::OneYear => 12,
            Self::ThreeYear => 36,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReservedCapacityError {
    EmptyId,
    EmptyRegionId,
    ZeroUnits,
}

impl ReservedCapacityError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::EmptyId => "reserved capacity id is empty",
            Self::EmptyRegionId => "region id is empty",
            Self::ZeroUnits => "reserved capacity requests zero units",
        }
    }
}

pub fn validate_reserved_capacity(rc: &ReservedCapacity) -> Result<(), ReservedCapacityError> {
    if rc.id.0.is_empty() {
        return Err(ReservedCapacityError::EmptyId);
    }
    if rc.region.0.is_empty() {
        return Err(ReservedCapacityError::EmptyRegionId);
    }
    if rc.reserved_units == 0 {
        return Err(ReservedCapacityError::ZeroUnits);
    }
    Ok(())
}

// ── CommittedUseContract ──────────────────────────────────────────────────────

/// A committed-use contract (CUC): discounted pricing in exchange for a
/// guaranteed minimum spend or usage over a fixed term.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedUseContract {
    pub id: CommittedUseContractId, // data_class: INTERNAL_ONLY
    pub region: RegionId,           // data_class: INTERNAL_ONLY
    pub class: CapacityClass,       // data_class: INTERNAL_ONLY
    pub committed_units: u64,       // data_class: INTERNAL_ONLY
    pub discount_bps: u32,          // data_class: INTERNAL_ONLY  (basis points, 0–10000)
    pub term: ReservationTerm,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedUseContractId(pub String); // data_class: INTERNAL_ONLY

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommittedUseError {
    EmptyId,
    EmptyRegionId,
    ZeroCommittedUnits,
    DiscountExceedsFull { discount_bps: u32 },
}

impl CommittedUseError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyId => "committed-use contract id is empty".to_owned(),
            Self::EmptyRegionId => "region id is empty".to_owned(),
            Self::ZeroCommittedUnits => {
                "committed-use contract has zero committed units".to_owned()
            }
            Self::DiscountExceedsFull { discount_bps } => {
                format!("discount_bps={discount_bps} exceeds 10000 (100%)")
            }
        }
    }
}

pub fn validate_committed_use_contract(
    cuc: &CommittedUseContract,
) -> Result<(), CommittedUseError> {
    if cuc.id.0.is_empty() {
        return Err(CommittedUseError::EmptyId);
    }
    if cuc.region.0.is_empty() {
        return Err(CommittedUseError::EmptyRegionId);
    }
    if cuc.committed_units == 0 {
        return Err(CommittedUseError::ZeroCommittedUnits);
    }
    if cuc.discount_bps > 10_000 {
        return Err(CommittedUseError::DiscountExceedsFull {
            discount_bps: cuc.discount_bps,
        });
    }
    Ok(())
}

// ── SpotPool ──────────────────────────────────────────────────────────────────

/// A pool of preemptible / spot capacity in a region.
///
/// Spot capacity is best-effort: it can be reclaimed at any time. The pool
/// tracks available and in-use units; admission prevents over-subscription.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpotPool {
    pub id: SpotPoolId,       // data_class: INTERNAL_ONLY
    pub region: RegionId,     // data_class: INTERNAL_ONLY
    pub class: CapacityClass, // data_class: INTERNAL_ONLY
    pub total_units: u64,     // data_class: INTERNAL_ONLY
    pub in_use_units: u64,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpotPoolId(pub String); // data_class: INTERNAL_ONLY

impl SpotPool {
    pub fn available(&self) -> u64 {
        self.total_units.saturating_sub(self.in_use_units)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpotPoolError {
    EmptyPoolId,
    EmptyRegionId,
    ZeroTotalUnits,
    ZeroRequestedUnits,
    InUseExceedsTotal { total: u64, in_use: u64 },
    SpotExhausted { available: u64, requested: u64 },
}

impl SpotPoolError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyPoolId => "spot pool id is empty".to_owned(),
            Self::EmptyRegionId => "region id is empty".to_owned(),
            Self::ZeroTotalUnits => "spot pool has zero total units".to_owned(),
            Self::ZeroRequestedUnits => "spot request has zero requested units".to_owned(),
            Self::InUseExceedsTotal { total, in_use } => {
                format!("in_use_units={in_use} exceeds total_units={total}")
            }
            Self::SpotExhausted {
                available,
                requested,
            } => {
                format!("spot exhausted: requested={requested} available={available}")
            }
        }
    }
}

pub fn validate_spot_pool(pool: &SpotPool) -> Result<(), SpotPoolError> {
    if pool.id.0.is_empty() {
        return Err(SpotPoolError::EmptyPoolId);
    }
    if pool.region.0.is_empty() {
        return Err(SpotPoolError::EmptyRegionId);
    }
    if pool.total_units == 0 {
        return Err(SpotPoolError::ZeroTotalUnits);
    }
    if pool.in_use_units > pool.total_units {
        return Err(SpotPoolError::InUseExceedsTotal {
            total: pool.total_units,
            in_use: pool.in_use_units,
        });
    }
    Ok(())
}

pub fn admit_spot_request(pool: &SpotPool, requested_units: u64) -> Result<(), SpotPoolError> {
    validate_spot_pool(pool)?;
    if requested_units == 0 {
        return Err(SpotPoolError::ZeroRequestedUnits);
    }
    if requested_units > pool.available() {
        return Err(SpotPoolError::SpotExhausted {
            available: pool.available(),
            requested: requested_units,
        });
    }
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CapacityClass, RegionId};

    // ── ReservedCapacity tests ─────────────────────────────────────────────

    fn rc(id: &str, region: &str, class: CapacityClass, units: u64) -> ReservedCapacity {
        ReservedCapacity {
            id: ReservedCapacityId(id.into()),
            region: RegionId(region.into()),
            class,
            reserved_units: units,
            term: ReservationTerm::OneYear,
        }
    }

    #[test]
    fn reservation_term_months() {
        assert_eq!(ReservationTerm::OneYear.months(), 12);
        assert_eq!(ReservationTerm::ThreeYear.months(), 36);
    }

    #[test]
    fn valid_reserved_capacity_passes() {
        assert!(validate_reserved_capacity(&rc("rc-1", "kr1", CapacityClass::Cpu, 100)).is_ok());
    }

    #[test]
    fn reserved_capacity_empty_id_rejected() {
        assert!(matches!(
            validate_reserved_capacity(&rc("", "kr1", CapacityClass::Cpu, 100)),
            Err(ReservedCapacityError::EmptyId)
        ));
    }

    #[test]
    fn reserved_capacity_empty_region_rejected() {
        assert!(matches!(
            validate_reserved_capacity(&rc("rc-1", "", CapacityClass::Cpu, 100)),
            Err(ReservedCapacityError::EmptyRegionId)
        ));
    }

    #[test]
    fn reserved_capacity_zero_units_rejected() {
        assert!(matches!(
            validate_reserved_capacity(&rc("rc-1", "kr1", CapacityClass::Cpu, 0)),
            Err(ReservedCapacityError::ZeroUnits)
        ));
    }

    // ── CommittedUseContract tests ─────────────────────────────────────────

    fn cuc(
        id: &str,
        region: &str,
        class: CapacityClass,
        units: u64,
        bps: u32,
    ) -> CommittedUseContract {
        CommittedUseContract {
            id: CommittedUseContractId(id.into()),
            region: RegionId(region.into()),
            class,
            committed_units: units,
            discount_bps: bps,
            term: ReservationTerm::ThreeYear,
        }
    }

    #[test]
    fn valid_committed_use_contract_passes() {
        assert!(
            validate_committed_use_contract(&cuc("cuc-1", "kr1", CapacityClass::Memory, 50, 2000))
                .is_ok()
        );
    }

    #[test]
    fn committed_use_empty_id_rejected() {
        assert!(matches!(
            validate_committed_use_contract(&cuc("", "kr1", CapacityClass::Memory, 50, 500)),
            Err(CommittedUseError::EmptyId)
        ));
    }

    #[test]
    fn committed_use_empty_region_rejected() {
        assert!(matches!(
            validate_committed_use_contract(&cuc("cuc-1", "", CapacityClass::Memory, 50, 500)),
            Err(CommittedUseError::EmptyRegionId)
        ));
    }

    #[test]
    fn committed_use_zero_units_rejected() {
        assert!(matches!(
            validate_committed_use_contract(&cuc("cuc-1", "kr1", CapacityClass::Memory, 0, 500)),
            Err(CommittedUseError::ZeroCommittedUnits)
        ));
    }

    #[test]
    fn committed_use_discount_at_boundary_passes() {
        assert!(
            validate_committed_use_contract(&cuc("cuc-1", "kr1", CapacityClass::Cpu, 10, 10_000))
                .is_ok()
        );
    }

    #[test]
    fn committed_use_discount_exceeds_full_rejected() {
        assert!(matches!(
            validate_committed_use_contract(&cuc("cuc-1", "kr1", CapacityClass::Cpu, 10, 10_001)),
            Err(CommittedUseError::DiscountExceedsFull { .. })
        ));
    }

    // ── SpotPool tests ─────────────────────────────────────────────────────

    fn pool(id: &str, region: &str, class: CapacityClass, total: u64, in_use: u64) -> SpotPool {
        SpotPool {
            id: SpotPoolId(id.into()),
            region: RegionId(region.into()),
            class,
            total_units: total,
            in_use_units: in_use,
        }
    }

    #[test]
    fn spot_pool_available_saturates() {
        assert_eq!(
            pool("sp-1", "kr1", CapacityClass::Gpu, 10, 3).available(),
            7
        );
        assert_eq!(
            pool("sp-1", "kr1", CapacityClass::Gpu, 5, 10).available(),
            0
        );
    }

    #[test]
    fn valid_spot_pool_passes() {
        assert!(validate_spot_pool(&pool("sp-1", "kr1", CapacityClass::Gpu, 8, 4)).is_ok());
    }

    #[test]
    fn spot_pool_empty_id_rejected() {
        assert!(matches!(
            validate_spot_pool(&pool("", "kr1", CapacityClass::Gpu, 8, 4)),
            Err(SpotPoolError::EmptyPoolId)
        ));
    }

    #[test]
    fn spot_pool_empty_region_rejected() {
        assert!(matches!(
            validate_spot_pool(&pool("sp-1", "", CapacityClass::Gpu, 8, 4)),
            Err(SpotPoolError::EmptyRegionId)
        ));
    }

    #[test]
    fn spot_pool_zero_total_rejected() {
        assert!(matches!(
            validate_spot_pool(&pool("sp-1", "kr1", CapacityClass::Gpu, 0, 0)),
            Err(SpotPoolError::ZeroTotalUnits)
        ));
    }

    #[test]
    fn spot_pool_in_use_exceeds_total_rejected() {
        assert!(matches!(
            validate_spot_pool(&pool("sp-1", "kr1", CapacityClass::Gpu, 5, 6)),
            Err(SpotPoolError::InUseExceedsTotal { .. })
        ));
    }

    #[test]
    fn admit_spot_zero_units_rejected() {
        let p = pool("sp-1", "kr1", CapacityClass::Gpu, 10, 3);
        assert!(matches!(
            admit_spot_request(&p, 0),
            Err(SpotPoolError::ZeroRequestedUnits)
        ));
    }

    #[test]
    fn admit_spot_within_available_passes() {
        let p = pool("sp-1", "kr1", CapacityClass::Gpu, 10, 3);
        assert!(admit_spot_request(&p, 7).is_ok());
    }

    #[test]
    fn admit_spot_at_boundary_passes() {
        let p = pool("sp-1", "kr1", CapacityClass::Gpu, 10, 3);
        assert!(admit_spot_request(&p, 7).is_ok());
    }

    #[test]
    fn admit_spot_over_available_rejected() {
        let p = pool("sp-1", "kr1", CapacityClass::Gpu, 10, 3);
        assert!(matches!(
            admit_spot_request(&p, 8),
            Err(SpotPoolError::SpotExhausted { .. })
        ));
    }

    #[test]
    fn class_isolation_spot_cpu_vs_gpu() {
        // GPU pool validation is independent of CPU pool state
        let gpu_pool = pool("sp-gpu", "kr1", CapacityClass::Gpu, 8, 8);
        let cpu_pool = pool("sp-cpu", "kr1", CapacityClass::Cpu, 100, 10);
        assert!(matches!(
            admit_spot_request(&gpu_pool, 1),
            Err(SpotPoolError::SpotExhausted { .. })
        ));
        assert!(admit_spot_request(&cpu_pool, 1).is_ok());
    }
}
