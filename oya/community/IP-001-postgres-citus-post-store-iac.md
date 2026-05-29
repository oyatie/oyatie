---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-001
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community + ops-iac
related_adrs: [ADR-0056, ADR-0105, ADR-0135, ADR-0131]
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001 — Postgres + Citus post-store IaC

## Intent

Provision the Citus-on-Postgres cluster that backs `post-store`, `voting-engine`, `moderation-queue`, and `kb-article-store`. Per-tenant distribution column `tenant_id`; RLS forced; Patroni for HA.

## Scope

- 1 coordinator + N workers per region.
- Replication factor 2; sync standby; WAL retention 7 d (14 d L/XL).
- pgvector + pg_partman extensions enabled.
- CIS-Postgres benchmark applied.
- Per-tenant schema `community`; tables: `posts`, `revisions`, `threads`, `votes`, `moderation_actions`, `flags`, `kb_articles`, `kb_article_revisions`, `kb_attachments`, `subscriptions`, `mentions`.

## Deliverables

- `iac/helm/postgres/Chart.yaml` + `values.yaml` (this IP)
- `iac/kustomize/base/kustomization.yaml` overlay reference
- Citus schema migration in `src/migrations/0001_initial.sql`
- RLS policy per table
- Backup CronJob (WAL-G to S3)

## Acceptance

- Cluster up; coordinator + 2 workers minimum.
- RLS enforced on every table.
- WAL backup runs hourly; restore drill green.
- Citus distribution column = `tenant_id` on every sharded table.
- Patroni leader election green.

## Risks

- Citus rebalancer storm during onboarding spike → throttle.
- WAL-G S3 backpressure → multi-region S3 endpoint.

## Owner

axis-community + ops-iac.

## Wave 15 substance conversion

### A. Problem this IP closes

The community product cannot offer Reddit-style threads, Teamblind-style anonymous workplace rooms, or Handshake-style job discussion without a tenant-sharded relational store that is explicitly designed for posts, replies, votes, moderation records, KB articles, and immutable revisions.
The prior shell named Postgres and Citus, but it did not bind the storage plan to the real community contracts or to the Wave 15K merge of the retired `network` professional surface into `community`.
This IP closes the substrate gap under `microservices/community/contracts/openapi/community.yaml` for `createPost`, `listPosts`, `postReply`, `castVote`, `applyModerationAction`, and `createKbArticle`.
It also closes the evidence gap between `microservices/community/manifest.json` bounded contexts and the actual persistence artifacts needed by `post-store`, `thread-tree`, `voting-engine`, `moderation-queue`, and `kb-article-store`.

### B. Approach

Use one tenant-scoped Citus/Postgres persistence cluster with `tenant_id` as the distribution column for mutable community rows and with strict RLS as a storage-level backstop behind Cedar.
The logical schema must mirror the contract entities in `community.proto`: `Post`, `ThreadNode`, `VoteTally`, `Flag`, `ModerationAction`, `KbArticle`, and `Attachment`.
The IaC target is not the existing drifted Terraform reference `microservices/community/iac/terraform/grafana-rbac.tf`; this IP must create canonical OpenTofu context modules or explicitly record the ADR-0328 gap before implementation.
Use `microservices/community/runbooks/post-mass-deletion.md`, `runbooks/vote-anomaly.md`, `runbooks/kb-attachment-restore.md`, and `runbooks/moderation-queue-clear.md` as restore and failure-mode evidence inputs.

### C. Deliverables

- Add or ratchet `microservices/community/iac/oyatie-public-cloud/postgres-citus/` with `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, and `README.md` in OpenTofu syntax.
- Add matching `guest-on-aws`, `oci-guest`, `on-prem`, `colo`, and `oyatie-iaas` context modules or an explicit N/A manifest for any unsupported context.
- Add the first SQL migration under the eventual community storage adapter crate path for `posts`, `post_revisions`, `thread_nodes`, `votes`, `moderation_actions`, `flags`, `kb_articles`, `kb_article_revisions`, `kb_attachments`, `subscriptions`, and `mentions`.
- Bind every table to `tenant_id`, `space_id`, `home_cell`, `jurisdiction_code`, `audit_event_class`, `created_at`, and `updated_at` where the row mutates.
- Publish catalog evidence by updating the relevant `microservices/community/catalog/oya-community-*-adapter-postgres.yaml` entries.
- Update `microservices/community/capabilities/post-create.yaml`, `vote-cast.yaml`, and `moderate-action.yaml` with storage dependency evidence.
- Add restore drill evidence to the remediation or evidence folder referenced by `runbooks/post-mass-deletion.md`.

### D. Implementation steps

1. Parse `microservices/community/contracts/proto/community.proto` and list every message field that must persist.
2. Map OpenAPI routes in `contracts/openapi/community.yaml` to tables and indexes; document route-to-table coverage in the module README.
3. Create the tenant distribution plan: `posts`, `thread_nodes`, `votes`, `flags`, `moderation_actions`, and KB tables distributed by `tenant_id`.
4. Add RLS policies that require `current_setting('oyatie.tenant_id') = tenant_id` and deny unset tenant context.
5. Add append-only guards for `post_revisions`, `kb_article_revisions`, and `moderation_actions`.
6. Add LTREE/GIN/BTREE index choices for reply traversal, tag lookup, vote tally reads, queue state reads, and KB article lookup.
7. Add WAL-G restore configuration with tenant-scoped restore rehearsal instructions.
8. Wire OpenTofu outputs for coordinator endpoint, reader endpoint, backup bucket reference, RLS policy checksum, and migration bundle SHA.
9. Add a validation command in the README that proves `tenant_id` is the Citus distribution key for every sharded table.
10. Record any missing OpenTofu context as a blocker instead of pretending the Terraform path satisfies ADR-0328.

### E. Acceptance

- `microservices/community/contracts/openapi/community.yaml` operations listed above each map to at least one migration-owned table.
- `microservices/community/contracts/proto/community.proto` messages `Post`, `ThreadNode`, `VoteTally`, `Flag`, `ModerationAction`, `KbArticle`, and `Attachment` have documented persistence coverage.
- RLS negative test proves a mismatched tenant cannot read, vote, moderate, or restore another tenant's records.
- Restore drill proves a tenant-scoped post deletion can recover without crossing a residency boundary.
- The ADR-0328 OpenTofu context check has concrete module paths or a named blocker, not a Terraform-only claim.

### F. Evidence

- `microservices/community/PRD.md` four-pillar purpose: Reddit, Teamblind, Handshake, LinkedIn jobs/profile/recruiter subset.
- `microservices/community/manifest.json` bounded context crate roster and top counterparts.
- `microservices/community/contracts/openapi/community.yaml` route surface.
- `microservices/community/contracts/proto/community.proto` persistence entities.
- `microservices/community/competitor-parity-matrix.md` product-pillar parity.
- `microservices/community/coherence-audit-2026-05-20.md` ADR-0328 OpenTofu and OS-support gaps.

### G. Counterpart closure

| Counterpart | Storage expectation | This IP closure |
|---|---|---|
| Reddit | durable subreddit/post/comment/vote persistence | tenant-sharded `posts`, `thread_nodes`, `votes`, and vote tally indexes |
| Teamblind | workplace-scoped anonymous posts with moderation evidence | tenant/workplace RLS plus append-only moderation and identity-blindable author references |
| Handshake | job/community discussion records tied to employer/candidate flows | space-scoped posts and KB articles with audit and residency labels |
| LinkedIn Jobs/Profile/Recruiter | professional discussions and profile-adjacent posts without engagement-feed ownership | storage supports jobs/profile/recruiter surfaces while preserving the forbidden feed boundary |
| GitHub Discussions | repository-scoped discussions with durable threads | same post/thread schema supports developer-forum mode without separate storage |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/community/IP-001-postgres-citus-post-store-iac.md` matched `openapi, .proto`; contract files `microservices/community/contracts/openapi/community.yaml, microservices/community/contracts/asyncapi/community-events.yaml, microservices/community/contracts/proto/community.proto`; type anchor `microservices/community/manifest.json`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/community/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/community/IP-001-postgres-citus-post-store-iac.md` matched `multi-region`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.
