---
doc_class: ImplementationPlan
milestone: M03-workspace-tier-foundation
phase: P01-forms-foundation
impl_plan_id: IP-008-meilisearch-adapter
status: pending
execution_unit: ChangeSet
owner: axis-forms + council-privacy
acceptance_lanes: [cargo-test, oya-forms-pii-column-not-indexed]
---

# IP-008: Meilisearch adapter (response search; PII-redacted)

## Intent

Per-pack Meilisearch index for response full-text + facet search. PII columns NEVER indexed (T-I-05 invariant). Index updated post-submit asynchronously.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/forms/src/adapter/meilisearch/client.rs` | create |
| `microservices/forms/src/adapter/meilisearch/index_policy.rs` | create — declares which fields indexable |
| `microservices/forms/src/adapter/meilisearch/sync_worker.rs` | create |
| `microservices/forms/tests/meilisearch_pii_not_indexed.rs` | create |

## Acceptance Gates

- `oya-forms-pii-column-not-indexed` exit 0; adversarial test attempts to index PII field, rejected.
- Index update latency ≤ 5s p95 after submit.

## References

- Meilisearch docs.
- ADR-FORMS-0003.

## Next IP

[`IP-009-captcha-adapter.md`](IP-009-captcha-adapter.md)
