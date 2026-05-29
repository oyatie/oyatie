---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-004
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community
related_adrs: [ADR-0105, ADR-0106, ADR-0135, ADR-0131]
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004 — post-store adapter-postgres + rest + worker + sdk + app crates

## Intent

Land the remaining post-store crate set: `adapter`, `adapter-postgres`, `rest`, `worker`, `sdk`, `app`.

## Scope

- `adapter` — port traits referenced by usecase.
- `adapter-postgres` — Postgres-backed adapter; RLS-aware connection acquisition; uses sqlx; pg_partman partitioning by month.
- `rest` — HTTP gateway implementing `contracts/openapi/community.yaml` for post-store paths.
- `worker` — NATS consumer for search-reindex + audit-bridge tasks.
- `sdk` — Rust client SDK (TypeScript / Python generated separately).
- `app` — composition root; wires kernel → domain → usecase → adapter-postgres → rest + worker.

## Deliverables

- Full crate set landed.
- Integration tests under `tests/`.

## Acceptance

- `cargo test -p oya-community-post-store-* --all-features` green.
- Integration test: `post_create_then_read` against a real Postgres instance.
- Load drill: 1 k creates / s, p99 ≤ 250 ms.
- SLO authored at `slos/post-create.openslo.yaml`.

## Owner

axis-community.

## Wave 15 substance conversion

### A. Problem this IP closes

The post-store crate family cannot stop at domain/usecase. The community surface needs a real Postgres adapter, REST gateway, worker, SDK, and composition root so the contract routes in `community.yaml` and the events in `community-events.yaml` become executable.
The previous shell collapsed five layers into one short paragraph and did not define the boundaries that keep adapters from reintroducing tenant leaks or framework types into core logic.
This IP closes the executable path from `createPost`/`editPost`/`deletePost` through Citus/Postgres persistence, NATS reindex events, audit emission, and Rust SDK consumption.

### B. Approach

Keep the adapter set explicit: `adapter` owns traits and shared adapter errors, `adapter-postgres` owns SQL and RLS session setup, `rest` owns HTTP extraction/serialization, `worker` owns async reindex/audit repair tasks, `sdk` owns Rust client calls, and `app` wires all dependencies.
The REST layer must implement the real operation IDs in `microservices/community/contracts/openapi/community.yaml` and must not invent routes beyond that file.
The worker must publish or consume the AsyncAPI subjects `community.search.reindex.requested`, `community.post.created`, `community.post.edited`, and `community.post.deleted`.

### C. Deliverables

- Add crates `oya-community-post-store-adapter`, `adapter-postgres`, `rest`, `worker`, `sdk`, and `app`.
- Add SQL integration tests for tenant RLS, edit revision append, tombstone, tag lookup, and body hash event generation.
- Add REST tests for `POST /spaces/{space_id}/posts`, `GET /spaces/{space_id}/posts`, `GET/PATCH/DELETE /posts/{post_id}`.
- Add worker tests for search reindex event publishing and audit repair idempotency.
- Update catalog files already listed in `microservices/community/catalog/`.
- Add a Rust SDK example to `microservices/community/reference-implementations/post-comment-vote-rust-sdk.md`.

### D. Implementation steps

1. Implement `PostRepository` in `adapter-postgres` using the schema from IP-001 and tenant-scoped connection setup.
2. Set `oyatie.tenant_id`, `oyatie.principal_id`, and `oyatie.home_cell` at transaction start before any query.
3. Implement REST extraction for bearer identity, `tenant_id`, `space_id`, and idempotency key without changing OpenAPI path shapes.
4. Translate usecase errors to the documented OpenAPI responses, including 401, 403, 404, 409, 422, and 429 where applicable.
5. Emit `PostCreated` and `PostEdited` events with `body_sha256` rather than raw content.
6. Consume retryable worker jobs for search reindex and audit repair from NATS JetStream with bounded retry and dead-letter subjects.
7. Add SDK methods matching the OpenAPI operation IDs and gRPC request names.
8. Wire the `app` crate with OpenBao secret references, not literal database or NATS credentials.
9. Add smoke tests with a real Postgres/Citus-compatible local container when the integration profile is enabled.
10. Add runbook references for mass deletion and spam flood paths.

### E. Acceptance

- `cargo test -p oya-community-post-store-adapter-postgres --features integration --locked` passes once implemented.
- REST tests prove routes match `contracts/openapi/community.yaml`.
- Worker tests prove raw body text never appears in emitted events.
- SDK example compiles against the generated API types.
- App crate depends inward on usecase/API and outward on adapter/rest/worker only.

### F. Evidence

- `microservices/community/contracts/openapi/community.yaml` post routes.
- `microservices/community/contracts/asyncapi/community-events.yaml` post and reindex channels.
- `microservices/community/reference-implementations/post-comment-vote-rust-sdk.md`.
- `microservices/community/runbooks/post-mass-deletion.md` and `runbooks/search-rebuild.md`.
- `microservices/community/manifest.json` layer list and crate roster.

### G. Counterpart closure

| Counterpart | Executable expectation | This IP closure |
|---|---|---|
| Reddit | API-backed posts and comments with durable moderation state | REST + Postgres + worker path for post lifecycle |
| Teamblind | anonymous workplace posts without identity leakage | adapter transactions bind tenant/principal while events hash body content |
| Handshake | recruiter/employer community Q&A accessible by SDK/API | Rust SDK and stable REST operation IDs |
| Discourse | migration-friendly API and background jobs | worker + SDK path aligned to Discourse migration playbook |
| GitHub Discussions | API-backed repository discussions | REST operation IDs and Rust SDK support developer-forum clients |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/community/IP-004-post-store-adapter-postgres-rest-worker-sdk-app.md` matched `openapi, asyncapi`; contract files `microservices/community/contracts/openapi/community.yaml, microservices/community/contracts/asyncapi/community-events.yaml, microservices/community/contracts/proto/community.proto`; type anchor `microservices/community/manifest.json`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/community/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/community/IP-004-post-store-adapter-postgres-rest-worker-sdk-app.md` matched `p99, SLO`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/community/IP-004-post-store-adapter-postgres-rest-worker-sdk-app.md` matched `emission`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.
