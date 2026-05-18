---
id: ADR-COMM-0004
status: Accepted
date: 2026-05-17
microservice: community
deciders: axis-community, ops-sre-reliability, council-architecture
owner: axis-community
supersedes: []
superseded_by: []
related:
  - ADR-MSGR-0003
  - ADR-0105
  - ADR-0135
  - ADR-0131
  - ADR-0132
related_artifacts:
  - microservices/community/PRD.md (FR-07, §"Performance" search-query row)
  - microservices/community/PHASE-01-COMMUNITY-SUBSTRATE.md (IP-009 search-index)
  - microservices/community/IP-009-search-index-elasticsearch.md (this ADR supersedes the Elasticsearch reference in its phrasing)
  - microservices/community/catalog/oya-community-search-index-adapter-search.yaml
  - microservices/messenger/decisions/ADR-MSGR-0003-search-backend-selection.md
purpose: Close PRD-community FR-07's open backend-selection question — fix the canonical content search backend (Meilisearch 0.10.0 LTS primary; Tantivy embedded fallback), aligning with the messenger sibling µservice's ADR-MSGR-0003 to share operator skill set + container images + helm idioms.
---

# ADR-COMM-0004: Content search backend — Meilisearch 0.10.0 LTS primary; Tantivy embedded fallback (aligned with ADR-MSGR-0003)

## Status

Accepted — 2026-05-17.

## Context

PRD-community FR-07 commits the µservice to "search across announcements + Q&A + KB articles ... search p99 ≤ 500 ms". §"Performance" pins the search SLO at p99 ≤ 500 ms / p999 ≤ 1.2 s. `IP-009-search-index-elasticsearch.md` was authored with "Elasticsearch" in its filename as a placeholder reference; the choice of backend was *not* fixed by an ADR.

The sibling µservice `messenger` resolved its equivalent search-backend question in `ADR-MSGR-0003-search-backend-selection.md` (Meilisearch 0.10.0 LTS primary, Tantivy embedded fallback). The forces in community's search problem are nearly identical:

- Multi-tenant indexing (one logical index per tenant; ACL filter at query time via Cedar policy).
- Faceted search across BCs (announcement vs. question vs. KB-article vs. discussion vs. tag).
- Typo tolerance + fuzzy matching expected by tenant users.
- p99 ≤ 500 ms (community is more generous than messenger's 350 ms because community ranks heavier docs with longer bodies; this still falls inside Meilisearch's comfort zone).
- License posture (Apache-2 / MIT preferred; Elastic License / SSPL avoided).
- JVM-tax avoidance for operator labor cost (oyatie is a Rust-native shop).
- Pack-kr / pack-us-healthcare / pack-eu data-residency commitments require per-pack search-index residency.

Community's search problem differs from messenger's in three places:

1. **Document size**: community KB articles are 100 KB body + N attachments; messenger documents are 4 KB messages. Meilisearch handles both shapes well but the per-doc cost is higher for KB articles → index storage budget per tenant is higher.
2. **Cross-BC search**: a single query must rank announcements + questions + KB articles + discussions together. Multi-index query is required; Meilisearch supports via "search across multiple indexes" feature; Tantivy supports via embedded multi-index pattern.
3. **Voted-content boost**: post rank is a function of vote tally (ADR-COMM-0002). The search ranker must incorporate vote-as-feature. Both Meilisearch and Tantivy support custom ranking-rule weights; BM25 + vote-weight = production-grade Q&A search.

The candidate set is the same as messenger's:
- Meilisearch (0.10.0 LTS) — Rust binary, HTTP API, MIT/Apache-2 dual-licensed.
- Tantivy (0.22.x) — embedded Rust full-text library.
- Elasticsearch (8.x or 7.17 OSS) — JVM, Elastic License + SSPL (not OSI-approved).
- OpenSearch (2.x) — JVM, Apache-2, AWS-forked.
- Sonic / Typesense — narrower; Sonic lacks facets, Typesense less mature for our use case.
- Postgres FTS — built-in, scales poorly past mid-tenant.

The PRD's NFR matrix mandates:
- p99 ≤ 500 ms cross-space ranked → all viable candidates can meet.
- p999 ≤ 1.2 s → Meilisearch + Tantivy + Elasticsearch all comfortably; Postgres FTS marginal at 5TB-per-cell.
- 5TB max index per cell → Meilisearch + Elasticsearch + OpenSearch can; Tantivy can per-index but ops gets thin past ~100GB.
- Per-tenant index sharding → all candidates support.
- Cedar-policy-evaluated server-side filtering → all support via filter-clause API.

## Decision

The community µservice ships a **two-backend search stack** behind the `ContentSearchIndex` port trait in `oya-community-search-index-kernel`:

1. **Primary backend: Meilisearch 0.10.0 LTS.** Adapter: `oya-community-search-index-adapter-search-meilisearch`. Per-tenant index pattern `tenant-<tenant_id>-community-{announcements,questions,kb,discussions}`; cross-BC ranked query via Meilisearch multi-search; Cedar policy post-filter at the usecase layer. Default for all multi-cell deployments and tenant-tier ≥ pro.

2. **Embedded fallback backend: Tantivy 0.22.x.** Adapter: `oya-community-search-index-adapter-search-tantivy`. Embedded in-process under tenant-scoped on-disk paths; used for:
   - Single-cell deployments (oyatie's own observability self-host, demo, integration test cells).
   - Starter-tier tenants whose total community-doc volume + search QPS fits embedded (≤ 100 GB per tenant, ≤ 10 QPS sustained).
   - Local development cells where running Meilisearch as a sidecar is overkill.

3. **Same port trait `ContentSearchIndex` in `-kernel`**; both adapters implement the trait. Selection is a Helm value + tenant config (`search_backend: meilisearch | tantivy`, default `meilisearch`).

4. **Voted-content boost** is implemented as a custom ranking rule weight on the `wilson_score` or `hot_score` field of each indexed document (ADR-COMM-0002 ranks are computed at the usecase layer and pushed into the index as scalar fields). Both backends support custom-rule rank weighting.

5. **Cross-BC search** is exposed as a single query endpoint that fans out to per-BC indexes and merges results by a combined `(bm25_score, vote_score)` lexicographic tuple. Cedar policy evaluation runs post-merge to drop docs the principal cannot read.

6. **No Elasticsearch / OpenSearch at M02.** They remain as future adapter options if a regulated tenant requires ELK-stack compatibility for their own audit reasons; oyatie does NOT default to them.

7. **Pack-kr / pack-us-healthcare / pack-eu**: per-pack Meilisearch deployments live inside the pack's residency boundary; cross-pack search is forbidden by `policy/tenant-scope.cedar`; the `IP-009` adapter implementation honours per-pack hostnames via the IaC values file (`microservices/community/iac/helm/community/values.yaml`).

8. **Search index encrypted-at-rest**: index files reside on tenant DEK-envelope-encrypted volumes (LUKS / EBS / OCI Block Volume per pack); token-level encryption inside the index is not feasible at the latency budget — volume encryption + residency + Cedar-filter combination is the documented mitigation, identical to the messenger sibling.

9. **Pinned LTS + quarterly upgrade IPs**: Meilisearch 0.10.0 LTS until 1.x LTS lands; Tantivy 0.22.x pinned; upgrade IPs scheduled per quarter, sequenced jointly with messenger's upgrade cadence to share operator on-call experience.

## Alternatives Considered

### A. Adopt a different backend than messenger (e.g., Elasticsearch for community, Meilisearch for messenger)
- Pros: per-µservice optimisation in principle.
- Cons: two backends in production = two operator skill sets, two upgrade cadences, two chart families, two failure modes; the µservices are siblings under the same umbrella ADR-0135 and should share operator burden by default; the community workload is not so different from messenger's that a different backend would meaningfully win.
- Rejected: operator-skill-set divergence cost outweighs marginal per-workload optimisation.

### B. Tantivy embedded only (no Meilisearch)
- Pros: zero ops surface; one Rust crate.
- Cons: 5 TB / cell ceiling is uncomfortable in Tantivy; faceted multi-index search is less mature than Meilisearch's; multi-cell scale-out for large tenants is harder; oyatie's promise of Stack-Overflow-Teams-class search needs Meilisearch's maturity.
- Rejected as primary; accepted as fallback.

### C. Elasticsearch 8.x
- Pros: mature, broad operator pool.
- Cons: Elastic License + SSPL non-OSI-approved (legal review per pack required); JVM operator cost ~3-5× Meilisearch; oyatie's data-class annotation lane has trouble verifying Elasticsearch field-level data-class because the mapping is shape-not-meaning; misaligned with Rust-native ops posture.
- Rejected: license + ops cost.

### D. OpenSearch 2.x
- Pros: Apache-2 clean.
- Cons: same JVM ops weight as Elasticsearch.
- Rejected: license cleanup doesn't justify ops cost increase.

### E. Postgres FTS (use the existing Citus Postgres cluster)
- Pros: no additional substrate; existing operator skill.
- Cons: FTS at 5 TB / cell with tenant isolation + facets + typo tolerance + voted-content boost is a documented anti-pattern; latency exceeds budget past ~50 GB / tenant.
- Rejected: scaling cliff documented in industry.

### F. Hybrid (Postgres FTS for small tenants, Meilisearch for large; no Tantivy fallback)
- Pros: keeps small-tenant cost low without spinning up Meilisearch.
- Cons: two backends with different ranking algorithms + different facet semantics → cross-tier ranking inconsistency UX cliff when a tenant grows out of FTS; Tantivy is closer to Meilisearch's ranking shape than Postgres FTS, so the cliff is shallower.
- Rejected: Tantivy fallback is a better small-tenant fallback than Postgres FTS.

## Consequences

### Positive

- Operator skill set, helm idioms, on-call runbooks, upgrade procedures are *shared* with messenger via ADR-MSGR-0003; one team can operate both µservices' search.
- Apache-2 / MIT license posture; no Elastic License legal review per pack.
- Rust-native primary + Rust-embedded fallback; aligns with oyatie's primary Rust posture.
- Two-backend port trait keeps both adapters honest; CI contract tests run against both backends (lane `community-search-contract-conformance`).
- Voted-content boost is a clean ranking-rule weight in both backends; no ML black box.

### Negative

- Two adapters to maintain; two test suites; two quarterly upgrade IPs. Mitigated by sharing the port trait + shared upgrade cadence with messenger.
- Cross-BC merge logic lives in usecase layer; complexity is moderate but real. Mitigated by a single `MultiSearchQuery` type + a single merger implementation under property-based tests.
- Tenant migration from Tantivy → Meilisearch (when a starter tenant grows past the comfort zone) requires re-index + dual-publish + cutover. Documented as `runbooks/search-backend-migration.md` (NEW, to be authored when first tenant requires it).
- Meilisearch operator pool is smaller than Elasticsearch's. Mitigated by Meilisearch's small surface; operator on-call docs in `runbooks/search-rebuild.md` (already exists) cover the day-2 ops scenarios.

### Operational

- Cargo workspace adds `oya-community-search-index-adapter-search-meilisearch` + `oya-community-search-index-adapter-search-tantivy`. The existing catalog entry `oya-community-search-index-adapter-search.yaml` documents the port-side; per-backend adapter catalog entries follow in IP-009 implementation.
- IaC: `microservices/community/iac/helm/community/` chart references the shared `meilisearch` Helm dep (same chart version as messenger to share image cache); embedded Tantivy needs no separate chart, just volume provisioning.
- Per-tenant config: `search_backend: meilisearch | tantivy` (default: meilisearch).
- Dashboards: `dashboards/search-latency.json` (NEW, to be authored under IP-012); per-backend search-latency panels for ops triage.
- CI: contract tests run against both backends; lane `community-search-contract-conformance` BLOCKS PRs that pass on one but fail on the other.
- IP-009 (currently named with "-elasticsearch" suffix) phrasing inside the file is amended in a successor-IP commit; the IP-009 *filename* is preserved to avoid breaking phase-sequencing references; the file's body is updated to reflect this ADR.

### Regulatory

- **Cedar policy evaluation** is orthogonal to backend; both backends are policy-blind; Cedar evaluates at the usecase layer (ACL prefilter + post-filter).
- **GDPR Art. 5(1)(c)** (data minimisation): search index excludes redacted PII per `policy/redaction-pii.md` (paired with the messenger sibling's `redaction-phi.md`); both backends honour the redaction layer at index-build time.
- **HIPAA 45 CFR §164.312** (pack-us-healthcare): index-volume encryption + tenant DEK envelope satisfies "encryption + decryption controls."
- **KR PIPA Art. 28**: pack-kr deployments enforce KR-resident search-index volumes; cross-pack search forbidden by Cedar.
- **License posture**: Meilisearch (MIT/Apache-2 dual) + Tantivy (MIT/Apache-2) → no license concerns at pack-level legal review.

## References

- Meilisearch project — `https://www.meilisearch.com/docs` (0.10.0 LTS)
- Tantivy project — `https://github.com/quickwit-oss/tantivy` (0.22.x)
- Apache Lucene engine internals — `https://lucene.apache.org/core/`
- Elasticsearch Elastic License + SSPL — `https://www.elastic.co/licensing/elastic-license`
- OpenSearch project — `https://opensearch.org/`
- Sonic — `https://github.com/valeriansaliou/sonic`
- Typesense — `https://typesense.org/docs/`
- BM25 ranking — Robertson & Zaragoza, "The Probabilistic Relevance Framework: BM25 and Beyond" — `https://doi.org/10.1561/1500000019`
- ADR-MSGR-0003 — sibling µservice's identical backend selection
- ADR-0135 — Connect-unbundle
- ADR-0131 — Per-microservice flat layout
- ADR-0132 — Product-suite-and-bundle dissolution
- `microservices/community/PRD.md` FR-07
- `microservices/community/IP-009-search-index-elasticsearch.md`
- `microservices/community/catalog/oya-community-search-index-adapter-search.yaml`
