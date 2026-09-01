//! Reading the reference store's edge set.
//!
//! Edges are keyed `(tenant, from, link_type, to)` in a `BTreeSet`, so
//! deduplication and deterministic read order both fall out of the key
//! rather than being maintained by hand.

use std::collections::BTreeMap;

use crate::store::ProjectedLink;

type EdgeKey = (String, String, String, String);
type Edges = BTreeMap<EdgeKey, u64>;

/// Edges leaving `object_ref`.
pub(crate) fn outbound(links: &Edges, tenant_id: &str, object_ref: &str) -> Vec<ProjectedLink> {
    collect(links, |(tenant, from, _, _)| {
        tenant == tenant_id && from == object_ref
    })
}

/// Edges arriving at `object_ref`.
pub(crate) fn inbound(links: &Edges, tenant_id: &str, object_ref: &str) -> Vec<ProjectedLink> {
    collect(links, |(tenant, _, _, to)| {
        tenant == tenant_id && to == object_ref
    })
}

fn collect(links: &Edges, keep: impl Fn(&EdgeKey) -> bool) -> Vec<ProjectedLink> {
    links
        .iter()
        .filter(|(key, _)| keep(key))
        .map(|((_, from, link_type, to), observed_at)| ProjectedLink {
            link_type: link_type.clone(),
            from_object_ref: from.clone(),
            to_object_ref: to.clone(),
            observed_at_epoch_ms: *observed_at,
        })
        .collect()
}
