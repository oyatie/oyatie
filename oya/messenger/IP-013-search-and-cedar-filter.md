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

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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

## Wave 15 substance conversion — search and Cedar filter

### §A Problem

Search is a high-risk disclosure path because indexes can return messages a caller cannot otherwise read.
This IP closes the gap between Meilisearch/Tantivy indexing, tenant/context partitioning, PHI redaction, and Cedar
post-filtering.

### §B Approach

Pre-narrow queries by tenant, context, channel membership, and pack, then post-filter each hit through Cedar before
returning it.
Index documents are minimized and redacted before storage.

### §C Deliverables

- `src/crates/oya-messenger-message-stream-adapter-meilisearch/src/query.rs`
- `src/crates/oya-messenger-message-stream-usecase/src/search.rs`
- `tests/search_cedar_scope_e2e.rs`
- rebuild/runbook references for index drift

### §D Implementation

1. Partition indexes by `(tenant_id, context_kind)`.
2. Strip or tokenize PHI according to `policy/redaction-phi.md`.
3. Build query filters from channel membership and Cedar scope.
4. Run post-filter Cedar checks on every hit.
5. Return only message ids/snippets allowed by policy.
6. Emit search latency and denied-hit metrics.

### §E Acceptance

E2E must prove tenant B never sees tenant A, Personal searches never return Professional messages, and PHI snippets
are redacted for healthcare pack searches.

### §F Evidence

Local anchors: `policy/channel-scope.cedar`, `policy/personal-dm-scope.cedar`, `policy/redaction-phi.md`,
`slos/search-latency.openslo.yaml`, `runbooks/search-index-rebuild.md`.

### §G Counterparts

Slack and Teams set enterprise search expectations, while Matrix/Mattermost show self-hosted tradeoffs; oyatie
closes parity with Cedar-scoped search rather than index-only ACLs.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/messenger/IP-013-search-and-cedar-filter.md` matched `PHI`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), KR-PIPA-2023-amendment(14400s/900s), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/messenger/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/messenger/slos/attachment-scan-freshness.openslo.yaml`, `microservices/messenger/slos/mention-fanout.openslo.yaml`, `microservices/messenger/slos/message-send-availability.openslo.yaml`, `microservices/messenger/slos/message-send-latency.openslo.yaml`, `microservices/messenger/policy/auditor-scope.cedar`.
