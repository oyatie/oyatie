//! Streaming partition strategy kernel (M06-P04-IP-002).
//!
//! Provider-neutral enum that controls how a streaming data-plane
//! partitions records across shards. Used by analytics.streaming.subscribe
//! to negotiate back-pressure and ordering guarantees.

// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// How a streaming adapter partitions event records across shards.
///
/// Ordering guarantees and throughput characteristics vary by variant;
/// callers choose the variant that matches their consistency requirement.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum StreamingPartitionStrategy {
    /// All records for a single tenant land on the same shard.
    /// Provides strict per-tenant ordering at the cost of hot-shard risk.
    TenantAffine,
    /// Records are distributed round-robin across all available shards.
    /// Maximises throughput; no ordering guarantee across records.
    RoundRobin,
    /// Records are hashed by a caller-supplied key (e.g. event correlation
    /// ID). Provides per-key ordering without full tenant affinity.
    KeyHash,
    /// Adapter chooses the strategy at runtime based on observed load.
    /// Callers must tolerate reordering between adjacent records.
    AdaptiveLoad,
}

impl StreamingPartitionStrategy {
    /// Returns the canonical kebab-case name used in topic configuration and
    /// observability labels.
    pub fn name(self) -> &'static str {
        match self {
            Self::TenantAffine => "tenant-affine",
            Self::RoundRobin => "round-robin",
            Self::KeyHash => "key-hash",
            Self::AdaptiveLoad => "adaptive-load",
        }
    }

    /// Returns `true` if this strategy guarantees strict ordering for records
    /// sharing the same partition key.
    pub fn is_ordered(self) -> bool {
        matches!(self, Self::TenantAffine | Self::KeyHash)
    }
}

/// Errors produced when validating a streaming partition configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamingPartitionError {
    /// The requested ordered strategy requires a non-empty partition key but
    /// none was supplied.
    MissingPartitionKey,
    /// The supplied shard count is zero; at least one shard is required.
    ZeroShardCount,
}

impl StreamingPartitionError {
    /// Returns a human-readable description of the error.
    pub fn message(&self) -> &'static str {
        match self {
            Self::MissingPartitionKey => {
                "ordered partition strategy requires a non-empty partition key"
            }
            Self::ZeroShardCount => "shard count must be at least 1",
        }
    }
}

/// Validates a streaming partition configuration.
///
/// `strategy` — the chosen partitioning strategy.
/// `partition_key` — caller-supplied key; required when `strategy` is ordered
///   ([`StreamingPartitionStrategy::TenantAffine`] or
///   [`StreamingPartitionStrategy::KeyHash`]), ignored otherwise.
/// `shard_count` — total number of shards; must be ≥ 1.
pub fn admit_streaming_partition(
    strategy: StreamingPartitionStrategy,
    partition_key: &str,
    shard_count: u32,
) -> Result<(), StreamingPartitionError> {
    if shard_count == 0 {
        return Err(StreamingPartitionError::ZeroShardCount);
    }
    if strategy.is_ordered() && partition_key.trim().is_empty() {
        return Err(StreamingPartitionError::MissingPartitionKey);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_strategy_names_are_distinct() {
        let names: HashSet<_> = [
            StreamingPartitionStrategy::TenantAffine,
            StreamingPartitionStrategy::RoundRobin,
            StreamingPartitionStrategy::KeyHash,
            StreamingPartitionStrategy::AdaptiveLoad,
        ]
        .iter()
        .map(|s| s.name())
        .collect();
        assert_eq!(names.len(), 4);
    }

    #[test]
    fn ordered_strategies_are_tenant_affine_and_key_hash() {
        assert!(StreamingPartitionStrategy::TenantAffine.is_ordered());
        assert!(StreamingPartitionStrategy::KeyHash.is_ordered());
        assert!(!StreamingPartitionStrategy::RoundRobin.is_ordered());
        assert!(!StreamingPartitionStrategy::AdaptiveLoad.is_ordered());
    }

    #[test]
    fn admit_tenant_affine_zero_shards_rejected() {
        assert_eq!(
            admit_streaming_partition(StreamingPartitionStrategy::TenantAffine, "", 0),
            Err(StreamingPartitionError::ZeroShardCount)
        );
    }

    #[test]
    fn admit_tenant_affine_missing_key_rejected() {
        assert_eq!(
            admit_streaming_partition(StreamingPartitionStrategy::TenantAffine, "   ", 4),
            Err(StreamingPartitionError::MissingPartitionKey)
        );
    }

    #[test]
    fn admit_tenant_affine_with_key_passes() {
        assert!(
            admit_streaming_partition(StreamingPartitionStrategy::TenantAffine, "tenant-1", 4)
                .is_ok()
        );
    }

    #[test]
    fn admit_key_hash_missing_key_rejected() {
        assert_eq!(
            admit_streaming_partition(StreamingPartitionStrategy::KeyHash, "   ", 4),
            Err(StreamingPartitionError::MissingPartitionKey)
        );
    }

    #[test]
    fn admit_key_hash_with_key_passes() {
        assert!(
            admit_streaming_partition(StreamingPartitionStrategy::KeyHash, "correlation-abc", 4)
                .is_ok()
        );
    }

    #[test]
    fn admit_round_robin_empty_key_allowed() {
        assert!(admit_streaming_partition(StreamingPartitionStrategy::RoundRobin, "", 8).is_ok());
    }

    #[test]
    fn admit_adaptive_load_passes() {
        assert!(admit_streaming_partition(StreamingPartitionStrategy::AdaptiveLoad, "", 1).is_ok());
    }

    #[test]
    fn error_messages_non_empty() {
        assert!(
            !StreamingPartitionError::MissingPartitionKey
                .message()
                .is_empty()
        );
        assert!(!StreamingPartitionError::ZeroShardCount.message().is_empty());
    }
}
