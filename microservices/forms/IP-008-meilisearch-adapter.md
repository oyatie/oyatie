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

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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
- PRD FR-14 and AC-17.
- `microservices/forms/catalog/oya-forms-meilisearch-adapter.yaml`.
- `microservices/forms/policy/data-residency.md`.
- `microservices/forms/slos/analytics-render-latency.openslo.yaml`.
- `microservices/forms/dashboards/response-pipeline.json`.
- `microservices/forms/runbooks/pii-leak-incident-p0.md`.

## Foundation A-G Substance

- A. Product scope: search and analytics accelerate operators, but response-store remains authoritative.
- B. Domain model: `IndexableFieldPolicy`, `ResponseSearchDocument`, and `RedactionDecision` are explicit before adapter calls.
- C. Contracts: search results expose response IDs, facets, and redacted snippets; never raw PII fields.
- D. Policy: index eligibility is derived from field `data_class`, pack residency, and auditor/tenant Cedar scope.
- E. Operations: async indexing is replayable from response-store and can rebuild an index without changing response history.
- F. Observability: track post-submit index lag, redaction drops, indexing failures, and analytics freshness.
- G. Promotion: PII-not-indexed adversarial test, p95 index-lag check, residency check, and dashboard smoke test gate done.

## Counterpart Benchmark

- Counterpart: Notion Forms/Databases searchable submissions, ServiceNow list search, and HubSpot Forms submission analytics.
- Defensible parity claim: Oyatie must support useful operator search without placing sensitive answers in the search index.
- Differentiator: PII exclusion is a policy-derived invariant instead of a best-effort UI filter.
- Grep counterpart names: Notion Forms/Databases; ServiceNow list search; HubSpot Forms.

## Remediation Notes

- Expanded search scope with catalog, policy, dashboard, SLO, and incident-response bindings.
- Added A-G substance for redaction, replay, observability, and promotion.
- Added counterpart names for mechanical discovery.

## Verification Evidence Required

- Adversarial index corpus attempts PII, PHI, and hidden-field indexing and receives deterministic rejection.
- Post-submit replay test proves the index can be rebuilt from response-store without changing response history.
- Dashboard smoke confirms response-pipeline freshness and analytics render latency remain inside SLO.
- Pack residency probe proves per-pack index routing follows `policy/data-residency.md`.
- Search contract test proves snippets and facets never expose raw sensitive values.
- Runbook drill proves index quarantine and rebuild are available after a suspected PII leak.
- Catalog evidence proves Meilisearch remains an adapter capability, not the response-store authority.
- Benchmark evidence records search freshness separately from analytics render latency.
- Audit evidence links each privileged search to tenant purpose and data-class posture.

## Next IP

[`IP-009-captcha-adapter.md`](IP-009-captcha-adapter.md)
