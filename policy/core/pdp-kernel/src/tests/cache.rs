use shared_platform_contracts_kernel::pdp::Decision;

use crate::{CachedDecision, DecisionCache, DecisionCacheKey};

#[test]
fn cache_is_bounded_and_evicts_in_insertion_order() {
    let mut cache = DecisionCache::new(2);
    let value = CachedDecision {
        decision: Decision::Deny,
        determining_policy_ids: vec![],
        obligations: vec![],
    };
    for i in 0..3 {
        cache.insert(
            DecisionCacheKey {
                request_fingerprint: format!("fp-{i}"),
                policy_version: "psv-1".to_owned(),
            },
            value.clone(),
        );
    }
    assert_eq!(cache.len(), 2);
    assert!(
        cache
            .get(&DecisionCacheKey {
                request_fingerprint: "fp-0".to_owned(),
                policy_version: "psv-1".to_owned(),
            })
            .is_none(),
        "oldest entry must be evicted first"
    );
}

#[test]
fn cache_key_separates_policy_versions() {
    let mut cache = DecisionCache::new(8);
    cache.insert(
        DecisionCacheKey {
            request_fingerprint: "fp".to_owned(),
            policy_version: "psv-1".to_owned(),
        },
        CachedDecision {
            decision: Decision::Allow,
            determining_policy_ids: vec!["rbac-tenant-admin-group".to_owned()],
            obligations: vec![],
        },
    );
    assert!(
        cache
            .get(&DecisionCacheKey {
                request_fingerprint: "fp".to_owned(),
                policy_version: "psv-2".to_owned(),
            })
            .is_none(),
        "a bundle swap must make prior entries unreachable"
    );
}

#[test]
fn zero_capacity_disables_caching() {
    let mut cache = DecisionCache::new(0);
    cache.insert(
        DecisionCacheKey {
            request_fingerprint: "fp".to_owned(),
            policy_version: "psv-1".to_owned(),
        },
        CachedDecision {
            decision: Decision::Deny,
            determining_policy_ids: vec![],
            obligations: vec![],
        },
    );
    assert!(cache.is_empty());
}
