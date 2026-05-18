---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-012-search-and-cedar-filter
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social
acceptance_lanes: [cargo-check, cargo-nextest, cedar-policy-eval, e2e-search-scope]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: search BC + Cedar-scoped filter (kernel → domain → usecase → adapter-meilisearch + worker + sdk)

## Intent

Author the `search` BC: people + content + hashtag search via Meilisearch
with Cedar post-filter. Server-side Cedar evaluation over every search hit
before return. Index partitioned per `(tenant_id, context_kind)`; query path
enforces both at adapter layer + post-filter. PHI redaction at index-time per
`policy/redaction-phi.md` (pack-us-healthcare overlay, Slice B).

## ChangeSet boundary

`search` BC end-to-end.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-social-search-kernel/src/{ports,entities,errors}.rs` | create |
| `src/crates/oya-social-search-domain/src/{search_doc,search_query,result_set,redaction}.rs` | create |
| `src/crates/oya-social-search-usecase/src/{search_people,search_content,reindex}.rs` | create |
| `src/crates/oya-social-search-adapter-meilisearch/src/{client,query,reindex}.rs` | create |
| `src/crates/oya-social-search-worker/src/indexer_loop.rs` | create |
| `tests/search_cedar_scope_e2e.rs` | create |

## Code Shape

```rust
// usecase/src/search_content.rs
pub async fn search_content<DEP: SearchDeps>(deps: &DEP, q: SearchQuery) -> Result<SearchResults> {
    // 1. Pre-narrow by tenant + context_kind + caller visibility
    let visibility_scope = deps.cedar.evaluate_visibility_scope(&q.principal).await?;
    let hits = deps.search_index.query(&q.text, &visibility_scope).await?;
    // 2. Post-filter — defence-in-depth Cedar evaluation per hit
    let filtered = hits.into_iter()
        .filter(|h| deps.cedar.allows_read(&q.principal, &h.post_id))
        .collect();
    Ok(SearchResults { results: filtered, ... })
}
```

## Acceptance Gates

```bash
cargo nextest run --test search_cedar_scope_e2e
oya gate validate cedar-policy-spec --microservice social
```

## Test Plan

- AC-07 E2E: tenant A searches; tenant B's posts never appear.
- Personal-context query; Professional-context posts never returned (and vice versa).
- PHI redaction: PHI fields stripped from index doc; tenant-admin search returns clean.
- Search lag fallback: Postgres ILIKE-fallback when Meilisearch indexer behind > 60s.
- People + content + hashtag search latency within SLO.

## Halt Conditions

- Over-permitted search result detected — Sev-1 (regression of Cedar post-filter).

## Next IP

[`IP-013-age-verification-and-profile-verification.md`](IP-013-age-verification-and-profile-verification.md)
