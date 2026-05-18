---
doc_class: ImplementationPlan
milestone: M03-workspace-tier-foundation
phase: P01-forms-foundation
impl_plan_id: IP-007-redis-adapter
status: pending
execution_unit: ChangeSet
owner: axis-forms
acceptance_lanes: [cargo-test]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: Redis adapter (rate-limit + session)

## Intent

Per-IP / per-form / per-tenant rate-limit token-bucket; submitter session state (≤ 30min TTL); captcha-token cache.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/forms/src/adapter/redis/client.rs` | create |
| `microservices/forms/src/adapter/redis/rate_limit.rs` | create |
| `microservices/forms/src/adapter/redis/session.rs` | create |
| `microservices/forms/src/adapter/redis/captcha_cache.rs` | create |
| `microservices/forms/tests/redis_rate_limit.rs` | create |

## Acceptance Gates

- Per-IP rate-limit verified under burst.
- Session TTL respected.
- Sentinel HA failover ≤ 30s.

## References

- Redis Sentinel docs.

## Next IP

[`IP-008-meilisearch-adapter.md`](IP-008-meilisearch-adapter.md)
