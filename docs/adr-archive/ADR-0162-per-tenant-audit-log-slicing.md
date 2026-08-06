---
id: ADR-0162
status: Superseded
deciders: council-architecture, axis-audit-chain, axis-tenancy, axis-governance, ops-sre-reliability
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-700]
related: [ADR-0003, ADR-0008, ADR-0009, ADR-0028, ADR-0038, ADR-0049, ADR-0128, ADR-0143, ADR-0158, ADR-0164]
related_specs:
  - /specs/per-tenant-audit-log-slicing-canonical.json
  - /specs/hyperscaler-architecture-invariants.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0162 — Per-tenant Audit-Chain Slicing (partition by tenant_id; sovereign-tenant dedicated shard; per-tenant retrieval API)

## Status

Accepted (2026-05-18). Refines the audit-chain emission contract (ADR-0003) so audit-chain Merkle seals partition by `tenant_id` and a per-tenant retrieval API exposes that tenant's seals (and only that tenant's seals) under Cedar-gated access. Aligns with the AWS CloudTrail-per-account pattern.

## Context

ADR-0003 fixed the audit-chain emission contract — every state-changing event ends in a Merkle-tree-sealed audit chain. ADR-0028 named the cloud-microservice architecture; ADR-0038 named the trust portal. ADR-0049 fixed residency.

ADR-0003 did not pin:

1. The **partition strategy** within the audit chain. Without per-tenant partitioning, a tenant-DPO Discovery request (e.g. "show me every audit seal touching tenant X for the past 30 days") requires a full Merkle traversal — operationally infeasible at scale.
2. The **sovereign-tenant dedicated-shard** contract. A `pack-ksa` sovereign tenant cannot share an audit-chain shard with non-KSA tenants because (a) the shard's storage may not leave KSA cells, (b) the shard's encryption key custody is KSA-only, (c) the shard's deletion-on-DSR cascade must not affect other tenants.
3. The **per-tenant retrieval API contract.** Tenant DPOs need self-service audit retrieval (per ADR-0038 trust portal); the API surface must be Cedar-gated and tenant-scoped by default.

The hyperscaler precedent:

- **AWS CloudTrail** runs per-account by default. Each account has its own CloudTrail log; cross-account aggregation is opt-in. This is the canonical "audit per tenant" pattern.
- **Google Cloud Audit Logs** are per-project (≈ per-tenant). Retrieval is project-scoped.
- **Azure Activity Log** is per-subscription.

The pattern is: audit-log partition === tenant boundary === access boundary.

## Decision

Audit-chain seals partition by `tenant_id`. The audit-chain µservice maintains:

### Sharding scheme

- **Per-pack shared shard.** Multi-tenant cells (e.g. `pack-us-shared`) use a *per-pack* audit-chain Merkle tree; tenant_id partition is a *leaf-level* partition within the shared tree. The Merkle root covers all tenants in the pack; per-tenant retrieval traverses the per-tenant subtree.
- **Per-sovereign-tenant dedicated shard.** A tenant in `pack-ksa` / `pack-uae` / `pack-eu-sovereign` / any pack marked `dedicated_audit_shard: true` gets its OWN audit-chain Merkle tree. Storage, encryption key, sealing schedule, retention are tenant-scoped.
- **Per-cell sharding within shared shards.** A tenant pinned to cell `kr-seoul1` has its audit-chain leaves stored in the kr-seoul1 audit-chain shard; cross-cell DR replicates per ADR-0009.

### Sealing cadence

- **Hot leaves** — append within 100 ms (per ADR-0003 contract).
- **Hourly seal** — every per-tenant subtree root recomputed and signed (Ed25519 per ADR-0003) every hour.
- **Daily root anchor** — per-pack root anchored to an immutable storage tier (oya-s3-cold per ADR-0161) daily.
- **Cross-shard root** — fleet-wide root computed daily; published to the trust portal (ADR-0038) for tenants to verify their own subtree against.

### Per-tenant retrieval API

The audit-chain µservice exposes:

```
GET /v1/audit-chain/tenant/{tenant_id}/seals?since={iso8601}&until={iso8601}&event_class={class}
```

Properties:

- **Cedar-gated.** The api-gateway tier (ADR-0157) verifies the JWT and confirms `principal.tenant_id == path.tenant_id` (a tenant can only retrieve its own seals). Service-tier callers (e.g. governance µservice running a compliance attestation) require a Cedar policy explicitly granting cross-tenant read.
- **Pagination** — returns seals in pages of 1000; cursor-based.
- **Inclusion-proof option** — `?proof=true` returns a Merkle inclusion proof per seal so the tenant can verify the seal is in the published root.
- **Per-event-class filter** — `event_class` filter (e.g. `DataSubjectRequestExecuted`, `TenantSettingsChanged`) so a tenant DPO can scope retrieval.
- **DSR-cascade-safe** — retrieving a seal does NOT side-channel personal data; the seal contains hashes + metadata only; personal-data fields are zeroed per ADR-0008 DUBO.

### Sovereign-tenant dedicated-shard contract

For sovereign packs (`pack-ksa`, `pack-eu-sovereign`, `pack-uae`, `pack-ru-if-onboarded`):

- Each tenant in the pack gets a dedicated shard.
- Shard storage is in-cell only (no cross-cell replication beyond intra-region DR).
- Shard encryption key custody is in-region HSM (per ADR-0043; per ADR-0164 air-gap variant).
- Shard sealing private key (Ed25519) is in-region HSM.
- Shard deletion on tenant offboarding leaves no residue in other tenants' shards (ADR-0008 DSR cascade).
- Cross-shard verification (the fleet-wide root) is published in-region only for sovereign-pinned tenants; non-sovereign packs publish to the global trust portal.

### Audit-shard µservice topology

The `audit-chain` µservice runs `active_passive` per cell (ADR-0158 disposition). Each cell hosts its own shards (per-pack shared shard + dedicated sovereign-tenant shards). Cross-cell DR replicates the shards intra-region only (per ADR-0049 sovereign-pin).

## Alternatives considered

### Alternative A — Single global Merkle tree (no per-tenant partition)

- **Pros:** simplest implementation; one global root.
- **Cons:** per-tenant retrieval requires full traversal (operationally infeasible); cross-tenant access leak risk (a tenant can theoretically traverse other tenants' subtrees); sovereign-pinned tenant data may co-locate with non-sovereign tenants in the global tree (residency violation).
- **Rejected because:** ADR-0038 + ADR-0049 + ADR-0008 all require per-tenant boundary; global tree violates each.

### Alternative B — Per-tenant Merkle tree (every tenant gets a dedicated shard, regardless of pack)

- **Pros:** maximum isolation.
- **Cons:** at fleet scale (10⁶+ tenants in the multi-tenant SaaS packs), millions of separate Merkle trees with their own sealing keys is operationally prohibitive (key management, root publishing, anchor cost).
- **Rejected because:** the operational cost is unbounded; the AWS / GCP / Azure precedent is "per-account" not "per-row".

### Alternative C — Per-pack shared shard + per-sovereign-tenant dedicated shard (this ADR)

- **Pros:** correct isolation where it matters (sovereign); operationally tractable for multi-tenant SaaS packs (per-pack shard with per-tenant subtree); aligns with the CloudTrail-per-account model; per-tenant retrieval is O(log n) via subtree.
- **Cons:** two sharding models in the codebase (shared vs. dedicated); per-pack overlay declares which.
- **Accepted.**

### Alternative D — Off-load to external SaaS audit service (Splunk / Datadog Audit)

- **Pros:** zero infra build.
- **Cons:** ADR-0049 residency + ADR-0164 air-gap requirements forbid external audit SaaS; audit-chain non-repudiation contract (ADR-0003) requires in-house cryptographic control.
- **Rejected because:** sovereign + non-repudiation requirements forbid.

### Alternative E — Per-event-class shards (one shard per event_class regardless of tenant)

- **Pros:** event-class queries fast.
- **Cons:** tenant-scoped retrieval requires cross-shard join; tenant offboarding requires multi-shard delete; doesn't match the access boundary (tenant) we actually need.
- **Rejected because:** tenant is the access boundary; event_class is a secondary filter.

## Consequences

### Positive

1. **Per-tenant retrieval is O(log n)** — Merkle subtree traversal not full-tree scan.
2. **Sovereign tenant isolation structural** — dedicated shards; encryption key in-region; deletion safe.
3. **CloudTrail-per-account precedent** — well-known pattern; auditors recognize.
4. **Cross-tenant access leakage structurally impossible** — Cedar gate at retrieval; per-tenant subtree.
5. **DSR cascade safe** — tenant offboarding deletes the tenant's subtree (shared shard) or shard (dedicated); no other tenant affected.
6. **Inclusion proofs for tenant self-verification** — tenant DPO can verify any seal is in the published root.
7. **Per-pack overlay choice** — shared vs. dedicated decided in pack overlay; not per-µservice.

### Negative

1. **Two sharding models in the codebase.** Shared and dedicated; per-pack overlay must declare; CI lane enforces.
2. **Sovereign-tenant dedicated shard cost.** Per-tenant Merkle tree + per-tenant sealing key + per-tenant HSM partition is real operational cost. Pricing model for sovereign packs reflects.
3. **Root anchor publishing schedule.** Daily fleet-wide root + hourly subtree root + 100ms hot-leaf append — three sealing tiers; ops monitors each.
4. **Sovereign tenants do NOT contribute to global fleet root.** A tenant in `pack-ksa` cannot prove inclusion in the global root because their shard is in-region only. PRD discloses.

### Operational

1. `audit-chain` µservice PRD updated with per-tenant slicing contract (Companion).
2. CI lane `cloud-ci/Rust gate packet audit-chain-per-tenant-slicing` enforces (a) every retrieval API call is Cedar-gated and tenant-scoped, (b) every sovereign pack overlay declares dedicated shards, (c) every per-tenant subtree's leaves contain only that tenant's events.
3. Per-tenant retrieval surface exposed via the api-gateway tier (ADR-0157) under `/v1/audit-chain/tenant/{tenant_id}/`.
4. Trust portal (ADR-0038) consumes the retrieval API to display tenant-self-service audit views.
5. Per-pack overlay declares `dedicated_audit_shard: true|false` in `microservices/audit-chain/iac/kustomize/components/pack-{name}/values.yaml`.

## References

- AWS CloudTrail per-account model — https://docs.aws.amazon.com/awscloudtrail/latest/userguide/cloudtrail-concepts.html
- Google Cloud Audit Logs per-project — https://cloud.google.com/logging/docs/audit
- Azure Activity Log per-subscription — https://learn.microsoft.com/azure/azure-monitor/essentials/activity-log
- Certificate Transparency Merkle Tree (RFC 9162) — per-log partition pattern.
- ADR-0003 — audit-chain emission contract (this ADR refines the partition).
- ADR-0008 — Data Use Boundary (DSR cascade).
- ADR-0009 — cell architecture (per-cell sharding).
- ADR-0028 — cloud microservice architecture.
- ADR-0038 — trust portal (per-tenant retrieval surface).
- ADR-0043 — HSM + KMS (per-tenant sealing key custody).
- ADR-0049 — residency (sovereign-pin).
- ADR-0128 — hyperscaler architecture invariants.
- ADR-0143 — foundry per-BC release pointer.
- ADR-0157 — api-gateway tier (retrieval surface).
- ADR-0158 — multi-region disposition.
- ADR-0164 — sovereign cloud / air-gapped deployment.
