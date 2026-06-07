---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-013
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community + governance
related_adrs: [ADR-0105, ADR-0135, ADR-0139, ADR-0131]
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013 — oya-governance promotion-readiness wiring

## Intent

Wire the community µservice's release pointers (`release/community/<region>/{dev,staging,production}`) to the oya-governance promotion-readiness lane. Community-specific gate criteria reference observability's eligibility verdict.

## Scope

- Per-region release pointers.
- Promotion gate criteria: SLO green; Cedar coverage green; cargo test green; chaos drill green.
- Auto-promote workflows on cadence (matching ADR-0139 cadence).
- Rollback wiring (force-fast-forward to prior pointer).

## Deliverables

- `release/community/<region>/{dev,staging,production}` ref creation.
- GitHub Actions workflow per region.
- Branch protection per pointer.

## Acceptance

- Promotion-readiness lane returns GREEN for community after 7 consecutive observability cycles.
- Rollback fires automatically on production-tier fast-burn.

## Owner

axis-community + governance.

## Wave 15 substance conversion

### A. Problem this IP closes

Community promotion cannot be a generic release-pointer fast-forward. It has product-specific blockers: post correctness, anonymity safety, moderation queue health, search freshness, KB restore readiness, SLO burn, Cedar coverage, and Wave 15K network-successor scope.
The old IP listed release pointers and seven SLO cycles but did not define the community-specific gate payload or its relationship to observability's `EligibilityVerdict`.
This IP closes the promotion-readiness contract for community.

### B. Approach

Use governance-managed release pointers and observability `slo-engine` eligibility verdicts as the promotion gate source.
Community adds service-local gates for Cedar coverage, contract compatibility, search freshness, moderation queue backlog, anonymity policy tests, KB attachment restore, and chaos drill freshness.
Promotion produces signed evidence, and rollback reverts release pointers plus queues reindex/restore follow-up work where needed.

### C. Deliverables

- Define community gate inputs for `release/community/<region>/{dev,staging,production}` or the current Jenkins/GitHub release-pointer equivalent.
- Bind observability API `GET /microservices/{microservice}/eligibility/{environment}/{sha}` from `microservices/observability/contracts/openapi/slo-engine.yaml`.
- Add checks for SLO green, Cedar compile/coverage, OpenAPI/proto compatibility, search freshness, moderation backlog, KB restore drill, and contract tests.
- Add evidence schema fields for tenant-class, region, cell, source SHA, target env, and Wave 15K network-successor scope.
- Link rollback to `microservices/community/runbooks/post-mass-deletion.md`, `search-rebuild.md`, and `moderation-queue-clear.md`.

### D. Implementation steps

1. Read observability `EligibilityVerdict` schema and use its `verdict`, `burn_rate_snapshot`, and `openslo_manifest_sha` fields.
2. Define community `PromotionReadinessInput` with source SHA, target env, region, cell, tenant class, and changed bounded contexts.
3. Query or consume seven consecutive community SLO eligibility cycles before staging-to-production.
4. Run Cedar compile and negative-test coverage for community policy fragments.
5. Validate `contracts/openapi/community.yaml`, `contracts/proto/community.proto`, and `contracts/asyncapi/community-events.yaml`.
6. Check moderation queue backlog and unresolved high-risk flags against thresholds.
7. Check search reindex lag and KB attachment restore drill freshness.
8. Check anonymity-mode tests for Teamblind and whistleblower/responsible-disclosure paths.
9. Emit signed promotion evidence and audit-chain event.
10. On fast-burn or gate regression, rollback the pointer and open follow-up evidence for stale indexes or moderation backlog.

### E. Acceptance

- Community cannot promote to production on SLO held/rejected/rollback verdict.
- Community cannot promote if Cedar coverage misses a mutating OpenAPI/proto operation.
- Community cannot promote if Teamblind anonymity negative tests fail.
- Community cannot promote if moderation queue backlog or search freshness exceeds threshold.
- Rollback path cites concrete runbooks and preserves audit evidence.

### F. Evidence

- `microservices/observability/contracts/openapi/slo-engine.yaml` `EligibilityVerdict`.
- `microservices/community/slos/*.openslo.yaml`.
- `microservices/community/policy/*.cedar`.
- `microservices/community/contracts/*`.
- `microservices/community/runbooks/moderation-queue-clear.md`, `search-rebuild.md`, and `post-mass-deletion.md`.
- `microservices/community/manifest.json` Wave 15K successor notes for `network`.

### G. Counterpart closure

| Counterpart | Release-safety expectation | This IP closure |
|---|---|---|
| Reddit | avoid global moderation/search regressions | moderation backlog and search freshness gates |
| Teamblind | avoid anonymity regressions | anonymity negative tests as promotion blockers |
| Handshake | protect employment-sensitive flows | tenant-class and employment-sensitive gate inputs |
| GitHub/GitLab | CI release gating | Governance release pointer and signed evidence |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/community/IP-013-oya-governance-promotion-readiness.md` matched `openapi, asyncapi, .proto`; contract files `microservices/community/contracts/openapi/community.yaml, microservices/community/contracts/asyncapi/community-events.yaml, microservices/community/contracts/proto/community.proto`; type anchor `microservices/community/manifest.json`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/community/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/community/IP-013-oya-governance-promotion-readiness.md` matched `SLO`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.
