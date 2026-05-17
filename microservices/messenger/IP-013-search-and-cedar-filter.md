---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-team-channels-dm-threads
impl_plan_id: IP-013-search-and-cedar-filter
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-messenger
acceptance_lanes: [cargo-nextest, cedar-policy-eval, e2e-search-scope]
---

# IP-013: Search + Cedar-scoped filter

## Intent

Server-side Cedar evaluation over every search hit before return. Index
partitioned per `(tenant_id, context_kind)`; query path enforces both at
adapter layer + post-filter. PHI redaction at index-time per
`policy/redaction-phi.md` (pack-us-healthcare overlay).

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-messenger-message-stream-adapter-meilisearch/src/query.rs` | create |
| `src/crates/oya-messenger-message-stream-usecase/src/search.rs` | create |
| `tests/search_cedar_scope_e2e.rs` | create |

## Code Shape

```rust
// usecase/src/search.rs
pub async fn search<DEP: SearchDeps>(deps: &DEP, q: SearchQuery) -> Result<SearchResults> {
    // 1. Pre-narrow by tenant + context_kind + caller membership
    let channel_acl = deps.cedar.evaluate_search_scope(&q.principal).await?;
    let hits = deps.search_index.query(&q.text, &channel_acl).await?;
    // 2. Post-filter — defence-in-depth Cedar evaluation per hit
    let filtered = hits.into_iter()
        .filter(|h| deps.cedar.allows_read(&q.principal, &h.message_id))
        .collect();
    Ok(SearchResults { results: filtered, ... })
}
```

## Acceptance Gates

```bash
cargo nextest run --test search_cedar_scope_e2e
oya gate validate cedar-policy-spec --microservice messenger
```

## Test Plan

- Tenant A searches; tenant B's channel never appears.
- Personal-context query; Professional-channel results never returned.
- PHI redaction: PHI fields stripped from index doc; tenant-admin search returns clean.

## Next IP

[`IP-014-huddles-livekit-signaling.md`](IP-014-huddles-livekit-signaling.md)
