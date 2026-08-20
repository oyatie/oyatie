//! Registry-view helper — partitions capability entries into discoverable and
//! invocable subsets using the [`CapabilityStatus`] predicates.
//!
//! No I/O, no async, std-only.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

use crate::{CapabilityId, CapabilityStatus};

/// The result of partitioning a capability collection into its observable
/// subsets.
///
/// Both maps use [`BTreeMap`] so iteration order is deterministic (lexicographic
/// on [`CapabilityId`], which derives [`Ord`]).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryViews {
    /// Capabilities visible to MCP discovery endpoints
    /// (i.e. [`CapabilityStatus::is_discoverable`] is `true`).
    pub discoverable: BTreeMap<CapabilityId, CapabilityStatus>,
    /// Capabilities that may be invoked by an authorised principal
    /// (i.e. [`CapabilityStatus::is_invocable`] is `true`).
    pub invocable: BTreeMap<CapabilityId, CapabilityStatus>,
}

/// Partition an iterator of `(CapabilityId, CapabilityStatus)` pairs into
/// [`RegistryViews`].
///
/// Each entry is tested against [`CapabilityStatus::is_discoverable`] and
/// [`CapabilityStatus::is_invocable`]; an entry may appear in both maps, one,
/// or neither.
///
/// # Duplicate IDs
///
/// When the same [`CapabilityId`] appears more than once, the **last** entry
/// wins (consistent with [`BTreeMap::insert`] semantics applied after
/// deduplication).
///
/// # Ordering
///
/// Both output maps are [`BTreeMap`]-backed, giving lexicographic ordering on
/// [`CapabilityId`] regardless of input order.
pub fn partition_views(
    entries: impl IntoIterator<Item = (CapabilityId, CapabilityStatus)>,
) -> RegistryViews {
    // Deduplicate first: last-writer-wins on duplicate IDs.
    let deduped: BTreeMap<CapabilityId, CapabilityStatus> = entries.into_iter().collect();

    let mut discoverable = BTreeMap::new();
    let mut invocable = BTreeMap::new();

    for (id, status) in deduped {
        if status.is_discoverable() {
            discoverable.insert(id.clone(), status);
        }
        if status.is_invocable() {
            invocable.insert(id, status);
        }
    }

    RegistryViews {
        discoverable,
        invocable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(s: &str) -> CapabilityId {
        CapabilityId::new(s)
    }

    #[test]
    fn active_appears_in_both_views() {
        let views = partition_views([(id("svc.op.active"), CapabilityStatus::Active)]);
        assert!(views.discoverable.contains_key(&id("svc.op.active")));
        assert!(views.invocable.contains_key(&id("svc.op.active")));
    }

    #[test]
    fn deprecated_invocable_only() {
        let views = partition_views([(id("svc.op.deprecated"), CapabilityStatus::Deprecated)]);
        assert!(!views.discoverable.contains_key(&id("svc.op.deprecated")));
        assert!(views.invocable.contains_key(&id("svc.op.deprecated")));
    }

    #[test]
    fn disabled_in_neither_view() {
        let views = partition_views([(id("svc.op.disabled"), CapabilityStatus::Disabled)]);
        assert!(!views.discoverable.contains_key(&id("svc.op.disabled")));
        assert!(!views.invocable.contains_key(&id("svc.op.disabled")));
    }

    #[test]
    fn ordering_is_stable_and_lexicographic() {
        let entries = vec![
            (id("z.op"), CapabilityStatus::Active),
            (id("a.op"), CapabilityStatus::Active),
            (id("m.op"), CapabilityStatus::Active),
        ];
        let views = partition_views(entries);
        let keys: Vec<_> = views.discoverable.keys().map(|k| k.0.as_str()).collect();
        assert_eq!(keys, ["a.op", "m.op", "z.op"]);
        let inv_keys: Vec<_> = views.invocable.keys().map(|k| k.0.as_str()).collect();
        assert_eq!(inv_keys, ["a.op", "m.op", "z.op"]);
    }

    #[test]
    fn mixed_collection_partitioned_correctly() {
        let entries = vec![
            (id("cap.active"), CapabilityStatus::Active),
            (id("cap.deprecated"), CapabilityStatus::Deprecated),
            (id("cap.disabled"), CapabilityStatus::Disabled),
        ];
        let views = partition_views(entries);

        // discoverable: Active only
        assert_eq!(views.discoverable.len(), 1);
        assert!(views.discoverable.contains_key(&id("cap.active")));

        // invocable: Active + Deprecated
        assert_eq!(views.invocable.len(), 2);
        assert!(views.invocable.contains_key(&id("cap.active")));
        assert!(views.invocable.contains_key(&id("cap.deprecated")));
        assert!(!views.invocable.contains_key(&id("cap.disabled")));
    }

    #[test]
    fn empty_input_yields_empty_views() {
        let views = partition_views(std::iter::empty());
        assert!(views.discoverable.is_empty());
        assert!(views.invocable.is_empty());
    }
}
