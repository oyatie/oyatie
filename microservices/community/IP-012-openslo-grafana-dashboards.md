---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-012
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community + axis-observability
related_adrs: [ADR-0105, ADR-0135, ADR-0139, ADR-0131]
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012 — OpenSLO manifests + Grafana dashboards

## Intent

Author per-BC OpenSLO manifests at `slos/*.openslo.yaml` and Grafana dashboards at `dashboards/*.json`. Wire to observability burn-rate evaluator.

## Scope

- OpenSLO: feed-render-p99, post-create-p99, vote-cast-p99, search-p99, moderation-p99, kb-publish-p99.
- Dashboards: post-throughput, vote-rate, moderation-queue-depth (this IP).
- Burn-rate alerts via Alertmanager + Grafana OnCall.

## Deliverables

- 6 OpenSLO manifests.
- 3 Grafana dashboards JSON.
- Alertmanager routing config.

## Acceptance

- Manifests validate against OpenSLO schema.
- Dashboards render in Grafana.
- Burn-rate evaluator reads community SLOs.
- Alert routing tested end-to-end.

## Owner

axis-community + axis-observability.

## Wave 15 substance conversion

### A. Problem this IP closes

Community cannot claim Reddit/Teamblind/Handshake-grade operation if its SLOs are generic dashboards detached from actual product flows.
The old IP listed OpenSLO and Grafana files but did not bind them to existing service files, promotion gates, alert runbooks, or counterpart performance expectations.
This IP closes observable product readiness for post creation, feed rendering, vote casting, search, moderation, KB publish, and audit seal latency.

### B. Approach

Use existing OpenSLO manifests in `microservices/community/slos/` as the source of truth and Grafana dashboards in `dashboards/` as operator evidence.
Route SLO evaluation through the observability µservice's `slo-engine` contract rather than bespoke community code.
Dashboards must group by tenant class, space mode, region, cell, operation, and bounded context without exposing raw tenant identifiers in shared views.
Alerts link to community runbooks and feed the `oya-vcs` promotion-readiness gate.

### C. Deliverables

- Validate `audit-chain-seal-latency.openslo.yaml`, `feed-render-latency.openslo.yaml`, `kb-article-publish-latency.openslo.yaml`, `moderation-action-latency.openslo.yaml`, `post-create-latency.openslo.yaml`, `search-query-latency.openslo.yaml`, and `vote-cast-latency.openslo.yaml`.
- Update dashboards `post-throughput.json`, `vote-rate.json`, and `moderation-queue-depth.json` with labels matching contract operations.
- Add dashboard panels or documented backlog for feed render, search, KB publish, and audit seal if absent.
- Add alert routing to runbooks for search rebuild, vote anomaly, spam flood, moderation queue, and KB restore.
- Add observability manifest evidence to the community catalog/capability records.

### D. Implementation steps

1. Parse each OpenSLO YAML and confirm service name, SLI query, target, and window.
2. Map `post-create` SLO to OpenAPI `createPost`.
3. Map `vote-cast` SLO to OpenAPI `castVote` and proto `CastVote`.
4. Map `search-query` SLO to search-index query path and rebuild stale-index alerts.
5. Map `moderation-action` SLO to `applyModerationAction` and queue depth dashboard.
6. Map `kb-article-publish` SLO to KB publication state transition.
7. Add labels for `surface_mode` values: reddit, teamblind, handshake, linkedin-subset, public-kb, developer-forum.
8. Ensure dashboards use low-cardinality tenant class labels rather than raw tenant IDs.
9. Wire Alertmanager/Grafana OnCall routes to runbooks.
10. Add evidence that observability `getEligibilityVerdict` can read community SLO verdicts.

### E. Acceptance

- Every existing SLO file validates and maps to a contract operation or named background worker.
- Dashboards render with operation labels and do not leak raw tenant IDs.
- Burn-rate alerts link to concrete runbooks under `microservices/community/runbooks/`.
- Promotion readiness can cite seven consecutive green community SLO cycles.
- Counterpart performance rows in `competitor-parity-matrix.md` have matching SLO evidence or explicit gaps.

### F. Evidence

- `microservices/community/slos/*.openslo.yaml`.
- `microservices/community/dashboards/*.json`.
- `microservices/observability/contracts/openapi/slo-engine.yaml`.
- `microservices/community/competitor-parity-matrix.md` performance parity rows.
- `microservices/community/runbooks/*.md`.

### G. Counterpart closure

| Counterpart | Operational expectation | This IP closure |
|---|---|---|
| Reddit | reliable post/feed/vote latency | post, feed, vote SLOs and dashboards |
| Teamblind | workplace safety queue latency | moderation action SLO and queue dashboard |
| Handshake | search/application community reliability | search and post-create evidence |
| Datadog/Grafana | SLO-backed operational dashboards | OpenSLO plus Grafana dashboard integration |
| GitHub Discussions | operational visibility for discussion quality | post, reply, search, and moderation SLOs cover developer-forum mode |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/community/IP-012-openslo-grafana-dashboards.md` matched `openapi`; contract files `microservices/community/contracts/openapi/community.yaml, microservices/community/contracts/asyncapi/community-events.yaml, microservices/community/contracts/proto/community.proto`; type anchor `microservices/community/manifest.json`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/community/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/community/IP-012-openslo-grafana-dashboards.md` matched `p99, SLO`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.
