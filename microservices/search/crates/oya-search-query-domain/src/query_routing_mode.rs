//! Query routing mode — selects the physical execution path for a query plan.
//!
//! Per M03-P05-IP-001/IP-002: inverted-index and vector-index shards are
//! independent subsystems. `QueryRoutingMode` lets the planner advertise
//! *where* a query plan should be dispatched so coordinators can fan-out to
//! the correct shard pool without inspecting `QueryMode` at every call-site.

#![forbid(unsafe_code)]

/// Physical execution path chosen by the query router.
///
/// Variants are ordered by declaration order for deterministic comparisons
/// (e.g. storing in `BTreeSet`, using as a `BTreeMap` key).  Both
/// `InvertedShardPool` and `VectorShardPool` fan out to exactly one shard pool
/// and have equal cost; `BothShardPools` fans out to two pools.  Use
/// [`QueryRoutingMode::fan_out_count`] for cost-based comparisons rather than
/// relying on the derived `Ord`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum QueryRoutingMode {
    /// Route only to the inverted-index shard pool (pgroonga).
    InvertedShardPool,
    /// Route only to the vector-index shard pool (pgvector).
    VectorShardPool,
    /// Fan-out to both shard pools and merge results at the coordinator.
    BothShardPools,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryRoutingModeError {
    /// The wire byte value does not correspond to any known variant.
    UnknownWireValue,
}

impl QueryRoutingMode {
    /// Stable wire encoding: 0 = inverted, 1 = vector, 2 = both.
    pub fn to_wire(self) -> u8 {
        match self {
            Self::InvertedShardPool => 0,
            Self::VectorShardPool => 1,
            Self::BothShardPools => 2,
        }
    }

    /// Decode from the stable wire encoding.
    pub fn from_wire(value: u8) -> Result<Self, QueryRoutingModeError> {
        match value {
            0 => Ok(Self::InvertedShardPool),
            1 => Ok(Self::VectorShardPool),
            2 => Ok(Self::BothShardPools),
            _ => Err(QueryRoutingModeError::UnknownWireValue),
        }
    }

    /// Returns `true` when this mode requires the inverted-index shard pool.
    pub fn needs_inverted(self) -> bool {
        matches!(self, Self::InvertedShardPool | Self::BothShardPools)
    }

    /// Returns `true` when this mode requires the vector-index shard pool.
    pub fn needs_vector(self) -> bool {
        matches!(self, Self::VectorShardPool | Self::BothShardPools)
    }

    /// Number of distinct shard pools this mode fans out to.
    ///
    /// Both `InvertedShardPool` and `VectorShardPool` return `1`; `BothShardPools`
    /// returns `2`.  Use this for cost-based threshold comparisons instead of
    /// relying on the derived `Ord`.
    pub fn fan_out_count(self) -> u8 {
        match self {
            Self::InvertedShardPool | Self::VectorShardPool => 1,
            Self::BothShardPools => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_round_trip_all_variants() {
        for mode in [
            QueryRoutingMode::InvertedShardPool,
            QueryRoutingMode::VectorShardPool,
            QueryRoutingMode::BothShardPools,
        ] {
            let decoded = QueryRoutingMode::from_wire(mode.to_wire()).expect("round-trip");
            assert_eq!(decoded, mode);
        }
    }

    #[test]
    fn unknown_wire_value_returns_error() {
        let err = QueryRoutingMode::from_wire(99).expect_err("unknown value");
        assert_eq!(err, QueryRoutingModeError::UnknownWireValue);
    }

    #[test]
    fn needs_inverted_correct() {
        assert!(QueryRoutingMode::InvertedShardPool.needs_inverted());
        assert!(QueryRoutingMode::BothShardPools.needs_inverted());
        assert!(!QueryRoutingMode::VectorShardPool.needs_inverted());
    }

    #[test]
    fn needs_vector_correct() {
        assert!(QueryRoutingMode::VectorShardPool.needs_vector());
        assert!(QueryRoutingMode::BothShardPools.needs_vector());
        assert!(!QueryRoutingMode::InvertedShardPool.needs_vector());
    }

    #[test]
    fn fan_out_count_single_pools_equal_cost() {
        assert_eq!(QueryRoutingMode::InvertedShardPool.fan_out_count(), 1);
        assert_eq!(QueryRoutingMode::VectorShardPool.fan_out_count(), 1);
        assert_eq!(QueryRoutingMode::BothShardPools.fan_out_count(), 2);
    }

    #[test]
    fn ordering_is_deterministic_declaration_order() {
        // Ord is by declaration order, not fan-out cost.
        // Both single-pool variants are cheaper than BothShardPools.
        assert!(QueryRoutingMode::InvertedShardPool < QueryRoutingMode::BothShardPools);
        assert!(QueryRoutingMode::VectorShardPool < QueryRoutingMode::BothShardPools);
    }
}
