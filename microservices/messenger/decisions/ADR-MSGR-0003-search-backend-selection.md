---
id: ADR-MSGR-0003
status: Accepted
date: 2026-05-17
microservice: messenger
deciders: axis-messenger, ops-sre-reliability, council-architecture
owner: axis-messenger
supersedes: []
superseded_by: []
related:
  - ADR-0131
  - ADR-0132
  - ADR-0133
related_artifacts:
  - microservices/messenger/PRD.md (Open Question 1 — search backend; FR-08)
  - microservices/messenger/PHASE-01-TEAM-CHANNELS-DM-THREADS.md (IP-005 search-index-iac, IP-013 search-and-cedar-filter)
  - microservices/messenger/catalog/
purpose: Close PRD-messenger Open Question 1 (derived gap) — select the canonical message-search backend with explicit fallback for small-cell deployments.
---

# ADR-MSGR-0003: Message search backend — Meilisearch 0.10.0 LTS primary; Tantivy embedded fallback for single-cell deployments

## Status

Accepted — 2026-05-17.

## Context

PRD-messenger FR-08 + AC-07 mandate message search returning only Cedar-permitted results at p99 ≤ 350ms per single-tenant 50-result query, scaling to 5TB per cell. The catalog and PRD note Meilisearch as a reference but the choice between Meilisearch / Tantivy / Elasticsearch / OpenSearch / Sonic / Typesense was not formally fixed. Open Question 1 asks specifically whether Tantivy is sufficient or whether Elasticsearch is needed for large tenants. The question generalises to the broader backend-selection problem.

Search-backend choices at M03:

- **Meilisearch (0.10.0 LTS)** — single-binary Rust server, HTTP API, fast indexing, native typo-tolerance, faceted search, MIT/Apache-2 dual-licensed. Strong DX. Multi-tenant via index-per-tenant. Mature feature set; smaller operator pool than Elasticsearch.
- **Tantivy (0.22.x)** — embedded Rust full-text library; no separate process; embeddable in `oya-messenger-message-stream-adapter-search`; fast; no facets in core (but available via extensions); ideal for single-cell, low-volume, simple-search use cases.
- **Elasticsearch (8.x or 7.17 final OSS fork)** — mature, JVM-based, heavyweight ops surface, license-encumbered (8.x is Elastic License + SSPL; not OSI-approved); operator pool plentiful but expensive.
- **OpenSearch (2.x)** — Elastic fork from AWS; Apache-2; heavyweight ops; reduces license risk but doesn't reduce ops weight.
- **Sonic** — small, fast, no facets, no fuzzy search; over-narrow.
- **Typesense** — multi-tenant-friendly; less mature than Meilisearch for the messaging use case.

PRD-messenger NFR matrix mandates:
- p99 ≤ 350ms for 50-result single-tenant search → all candidates above can meet at small-mid scale.
- Per-tenant index sharding → Meilisearch, Elasticsearch, OpenSearch, Typesense all support; Tantivy supports via embedded multi-index pattern.
- 5TB max index per cell → Meilisearch + Elasticsearch + OpenSearch can; Tantivy can per-index but ops gets thin past ~100GB.
- Cedar-policy-evaluated server-side filtering → all support via filter-clause API.
- Encrypted-index posture (per messenger ADR family's E2E commitment for Personal-DM tier) → none provide encrypted-index out of the box; Personal-DM tier searches client-side per `personal-dm-scope.cedar` `forbid` of server-side capability execution; Professional-channel tier search runs server-side on ciphertext-with-tokenized-index per the BC's design.

The decision needs a primary backend for production scale + an embedded fallback for development cells, single-cell tenants (starter tier), and oyatie's own observability self-host deployment.

## Decision

oyatie messenger ships a **two-backend search stack** behind the `MessageSearchIndex` port trait in `oya-messenger-message-stream-kernel`:

1. **Primary backend: Meilisearch 0.10.0 LTS.** Adapter: `oya-messenger-message-stream-adapter-search-meilisearch`. Per-tenant index pattern (`tenant-<tenant_id>-messages`); per-channel facet for ACL-filter prefiltering; Cedar policy evaluation post-filter (Cedar runs on the messenger usecase layer before returning hits). Default for all multi-cell deployments and tenant-tier ≥ pro.

2. **Embedded fallback backend: Tantivy 0.22.x.** Adapter: `oya-messenger-message-stream-adapter-search-tantivy`. Embedded in-process; index files written to tenant-scoped on-disk paths; used for:
   - Single-cell deployments (oyatie's own observability self-host, demo deployments, integration test cells).
   - Starter-tier tenants whose message volume + search QPS comfortably fits embedded (≤ 100GB per tenant, ≤ 10 search QPS sustained).
   - Local development environments where running a Meilisearch sidecar is overkill.

3. **Same port trait `MessageSearchIndex` in `-kernel`**: both adapters implement the trait; selection is a Helm value + tenant config. No business logic shifts between backends.

4. **No Elasticsearch / OpenSearch at M03.** They remain as future adapter options if a regulated tenant requires ELK-stack compatibility for their own reasons, but oyatie does NOT default to them.

5. **Personal-DM tier search**: client-side only (per `personal-dm-scope.cedar` forbid of server-side capability execution on personal DMs). Both backends serve only Professional-channel tier search.

6. **Search index encrypted-at-rest**: index files written under tenant DEK envelope encryption for both backends; the Meilisearch instance and Tantivy index files are encrypted at the volume layer (LUKS / EBS / OCI Block Volume per pack); the search-token index inside the volume is plaintext tokens because token-level FHE search is not feasible at the µservice's latency budget — the residency + volume encryption + Cedar-filter combination is the documented mitigation.

7. **Pinned LTS + quarterly upgrade IPs**: Meilisearch 0.10.0 LTS until 1.x LTS lands; Tantivy 0.22.x pinned; upgrade IPs scheduled per quarter.

## Alternatives Considered

### A. Tantivy embedded only (no separate search server)
- Pros: zero ops surface; one binary; fastest dev experience; cheapest at small scale.
- Cons: ops gets thin past ~100GB per index; faceted search via Tantivy extensions is less mature than Meilisearch's native facets; multi-tenant horizontal scale-out across cells is more complex (have to manage cell-residence of indexes per tenant); past mid-scale the operator surface gets larger than Meilisearch's.
- Rejected as primary; accepted as fallback for the use cases where it actually wins.

### B. Elasticsearch 8.x
- Pros: mature, broad operator pool, vast ecosystem, deep tuning surface.
- Cons: Elastic License + SSPL means it's not OSI-approved free software (legal review per pack); JVM ops weight (heap tuning, GC pauses); operator labor cost ~3-5x Meilisearch; oyatie's data-class annotation lane finds it hard to verify Elasticsearch field-level data-class because Elasticsearch's mapping is shape-not-meaning.
- Rejected: license posture + ops weight + cost vs. benefit unfavourable.

### C. OpenSearch 2.x
- Pros: Apache-2 license clean; AWS-supported; ELK-compatible.
- Cons: same ops weight as Elasticsearch (it's a fork); same JVM cost; same operator complexity; the license clean-up doesn't reduce the operational mismatch with Meilisearch's much lower ops floor.
- Rejected: ops weight unfavourable; license clean-up doesn't justify the operator cost increase.

### D. Sonic (lightweight search server)
- Pros: tiny, fast, minimal ops.
- Cons: no facets; no fuzzy/typo tolerance; no multi-tenant management primitives; over-narrow for messenger's search needs (channel filter + author filter + thread-id filter + reaction filter).
- Rejected: feature-deficient for messenger search FR-08.

### E. Typesense
- Pros: multi-tenant-friendly; modern API; HTTP + JSON.
- Cons: less mature than Meilisearch at the multi-tenant message-search use case; smaller community; documentation thinner.
- Rejected: less mature than Meilisearch for the specific use case.

### F. Build a search engine from scratch on Postgres FTS
- Pros: no additional substrate; uses existing Postgres.
- Cons: Postgres FTS at 5TB/cell + tenant-isolation + facets + typo-tolerance is a road to ruin; documented anti-pattern at multi-tenant SaaS scale.
- Rejected: well-known scaling cliff.

## Consequences

### Positive

- Meilisearch's HTTP + JSON API + single-binary deployment minimises ops surface vs. ELK alternatives; the µservice operator team has one search-substrate to monitor + patch + capacity-plan.
- Tantivy fallback covers small-cell + dev + observability self-host without spinning up Meilisearch when it isn't needed; reduces overhead for the cases where overhead matters.
- Both backends are Rust-native (Meilisearch is a Rust binary; Tantivy is a Rust library) → operator and code-author skill sets align with oyatie's primary Rust posture; no JVM tax.
- Port trait `MessageSearchIndex` keeps both adapters honest — switching tenants between backends is operationally meaningful but architecturally cheap.
- Apache-2 / MIT licenses for both backends → no license posture concerns at pack-level review.

### Negative

- Two backends to maintain (two adapters, two test sets, two upgrade IPs per quarter). Mitigated by sharing the port trait + contract tests in `microservices/messenger/tests/contract/search/`.
- Meilisearch operator pool is smaller than Elasticsearch's; mitigated by Meilisearch's much smaller surface (one binary, one config file, native HTTP). Operator on-call documentation in runbook needed.
- Tenant migration from Tantivy → Meilisearch (when a starter tenant grows past the Tantivy comfort zone) requires re-index, dual-publish window, and cutover; documented as `microservices/messenger/runbooks/search-backend-migration.md` (NEW) — not auto-migratable.
- Personal-DM tier client-side search is bandwidth- and battery-heavy on mobile; mitigated by client-side index caching + WASM-Tantivy on web; this is the tradeoff of preserving E2E.

### Operational

- Cargo workspace adds `oya-messenger-message-stream-adapter-search-meilisearch` + `oya-messenger-message-stream-adapter-search-tantivy`.
- IaC: `microservices/messenger/iac/helm/meilisearch/` chart for multi-cell deployments; embedded Tantivy needs no separate chart, just volume provisioning.
- Per-tenant config: `search_backend: meilisearch | tantivy` (default: meilisearch).
- Dashboards: per-backend search latency panels; backend-cross-comparison panel for ops triage.
- CI: contract tests run against both backends; lane `messenger-search-contract-conformance` BLOCKS PRs that pass on one but fail on the other.
- Per `microservices/messenger/PHASE-01-TEAM-CHANNELS-DM-THREADS.md` IP-005 `search-index-iac` and IP-013 `search-and-cedar-filter` consume this ADR's decision.

### Regulatory

- **Cedar policy evaluation** (orthogonal to backend) — implemented in the messenger usecase layer; both backends are policy-blind and rely on the usecase layer for ACL prefilter + post-filter; Cedar evaluation runs server-side, no client trust.
- **GDPR Art. 5(1)(c)** (data minimisation) — search index excludes redacted PII / PHI per `policy/redaction-phi.md`; both backends honour the redaction layer.
- **HIPAA 45 CFR §164.312** — index volume encryption + tenant DEK envelope encryption satisfies "encryption + decryption controls."
- **KR PIPA Art. 28** (technical security measures) — pack-kr deployments enforce KR-resident search-index volumes.
- **License posture**: Meilisearch (MIT/Apache-2 dual) + Tantivy (MIT/Apache-2) → no license concerns at pack-level legal review.

## References

- Meilisearch project — `https://www.meilisearch.com/docs` (0.10.0 LTS)
- Tantivy project — `https://github.com/quickwit-oss/tantivy` (0.22.x)
- Elasticsearch Elastic License + SSPL — `https://www.elastic.co/licensing/elastic-license`
- OpenSearch project — `https://opensearch.org/`
- Sonic — `https://github.com/valeriansaliou/sonic`
- Typesense — `https://typesense.org/docs/`
- AWS CloudSearch + Lucene precedent (industry comparison)
- Apache Lucene engine internals
- ADR-0131 — Per-microservice flat layout
- ADR-0132 — Product-platform-and-bundle dissolution
- ADR-0133 — Industry best-practice conformance program
- `microservices/messenger/PRD.md` FR-08, AC-07, Open Question 1
- `microservices/messenger/PHASE-01-TEAM-CHANNELS-DM-THREADS.md` IP-005, IP-013
- `microservices/messenger/policy/personal-dm-scope.cedar` (server-side capability forbid for Personal-DM tier)
- `microservices/messenger/policy/redaction-phi.md`
