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
    ZeroListRate,
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
            Self::ZeroListRate => "list_rate_micros must be non-zero".to_owned(),
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

// ── Amortization + coverage math ─────────────────────────────────────────────

/// Returns the effective discounted rate in micros after applying `discount_bps`.
///
/// - `discount_bps == 0`: returns `list_rate_micros` unchanged.
/// - `discount_bps == 10000`: returns `0`.
/// - Saturating arithmetic; values > 10000 bottom at 0 (callers should pre-validate
///   via [`validate_committed_use_contract`]).
/// - Returns [`CommittedUseError::ZeroListRate`] when `list_rate_micros == 0`.
pub fn effective_discounted_rate(
    list_rate_micros: u128,
    discount_bps: u32,
) -> Result<u128, CommittedUseError> {
    if list_rate_micros == 0 {
        return Err(CommittedUseError::ZeroListRate);
    }
    let discount = list_rate_micros.saturating_mul(discount_bps as u128) / 10_000;
    Ok(list_rate_micros.saturating_sub(discount))
}

/// Returns the floor-divided monthly amortization of `total_commit_micros` over `term`.
///
/// Uses [`ReservationTerm::months`] as divisor. Remainder is dropped; callers who need
/// exact reconciliation compute `total_commit_micros % term.months() as u128`.
///
/// Defensive guard: returns `total_commit_micros` unchanged if `months()` is ever 0
/// (impossible with the current enum variants).
pub fn amortized_monthly_commit_micros(total_commit_micros: u128, term: ReservationTerm) -> u128 {
    let months = term.months() as u128;
    if months == 0 {
        return total_commit_micros;
    }
    total_commit_micros / months
}

/// Returns committed coverage in basis points (0–10000), capped at 10000.
///
/// - Returns `0` when `demand_units == 0`.
/// - Otherwise: `min(reserved_units * 10_000 / demand_units, 10_000)` cast to `u32`.
/// - Uses `u128` intermediate arithmetic to prevent overflow.
pub fn committed_coverage_bps(reserved_units: u64, demand_units: u64) -> u32 {
    if demand_units == 0 {
        return 0;
    }
    let coverage = (reserved_units as u128).saturating_mul(10_000) / demand_units as u128;
    coverage.min(10_000) as u32
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

    // ── effective_discounted_rate tests ───────────────────────────────────────

    #[test]
    fn discount_zero_bps_returns_list_rate_unchanged() {
        assert_eq!(effective_discounted_rate(1_000_000, 0).unwrap(), 1_000_000);
    }

    #[test]
    fn discount_full_10000_bps_returns_zero() {
        assert_eq!(effective_discounted_rate(1_000_000, 10_000).unwrap(), 0);
    }

    #[test]
    fn discount_partial_bps_applies_correctly() {
        // 20% discount on 10_000_000 micros -> 8_000_000
        assert_eq!(
            effective_discounted_rate(10_000_000, 2_000).unwrap(),
            8_000_000
        );
    }

    #[test]
    fn discount_zero_list_rate_returns_error() {
        assert!(matches!(
            effective_discounted_rate(0, 500),
            Err(CommittedUseError::ZeroListRate)
        ));
    }

    #[test]
    fn discount_over_10000_bps_saturates_to_zero() {
        // bps > 10000 bottoms at 0 via saturating sub
        assert_eq!(effective_discounted_rate(1_000, 10_001).unwrap(), 0);
    }

    #[test]
    fn validate_contract_rejects_discount_over_10000() {
        // acceptance criterion (c): validate gate still rejects bps > 10000
        let contract = CommittedUseContract {
            id: CommittedUseContractId("c1".into()),
            region: RegionId("kr1".into()),
            class: CapacityClass::Cpu,
            committed_units: 100,
            discount_bps: 10_001,
            term: ReservationTerm::OneYear,
        };
        assert!(matches!(
            validate_committed_use_contract(&contract),
            Err(CommittedUseError::DiscountExceedsFull { .. })
        ));
    }

    // ── amortized_monthly_commit_micros tests ─────────────────────────────────

    #[test]
    fn amortization_exact_division_one_year() {
        // 1200 over 12 months -> 100 per month, no remainder
        assert_eq!(
            amortized_monthly_commit_micros(1_200, ReservationTerm::OneYear),
            100
        );
    }

    #[test]
    fn amortization_with_remainder_drops_remainder() {
        // 13 over 12 months -> floor(13/12) = 1, remainder 1 dropped
        assert_eq!(
            amortized_monthly_commit_micros(13, ReservationTerm::OneYear),
            1
        );
    }

    #[test]
    fn amortization_three_year_term() {
        // 3600 over 36 months -> 100 per month
        assert_eq!(
            amortized_monthly_commit_micros(3_600, ReservationTerm::ThreeYear),
            100
        );
    }

    #[test]
    fn amortization_zero_total_returns_zero() {
        assert_eq!(
            amortized_monthly_commit_micros(0, ReservationTerm::OneYear),
            0
        );
    }

    // ── committed_coverage_bps tests ──────────────────────────────────────────

    #[test]
    fn coverage_saturates_at_10000_when_reserved_equals_demand() {
        assert_eq!(committed_coverage_bps(100, 100), 10_000);
    }

    #[test]
    fn coverage_saturates_at_10000_when_reserved_exceeds_demand() {
        assert_eq!(committed_coverage_bps(200, 100), 10_000);
    }

    #[test]
    fn coverage_partial_below_10000() {
        // 50 reserved / 100 demand -> 5000 bps (50%)
        assert_eq!(committed_coverage_bps(50, 100), 5_000);
    }

    #[test]
    fn coverage_zero_demand_returns_zero() {
        assert_eq!(committed_coverage_bps(100, 0), 0);
    }

    #[test]
    fn coverage_zero_reserved_returns_zero() {
        assert_eq!(committed_coverage_bps(0, 100), 0);
    }
}
