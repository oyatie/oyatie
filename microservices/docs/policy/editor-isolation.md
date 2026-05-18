---
doc_class: PolicySpec
title: Editor Isolation Contract
microservice: docs
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-docs + ops-security + council-design-system
deciders: axis-docs, ops-security, council-architecture, council-design-system
related_adrs: [ADR-0028, ADR-0065, ADR-0131, ADR-0140 (retired per ADR-0145), ADR-DOCS-0001, ADR-DOCS-0004]
related_artifacts:
  - microservices/docs/threat-model.md (T-T-01, T-I-01, T-I-02, T-I-03, T-E-01)
  - microservices/docs/dpia.md (R-01, R-02, R-16)
  - microservices/docs/policy/tenant-scope.cedar
review_cadence: quarterly + on every collab-crdt / block-types / sharing-and-permissions change
doc_status: published
---

# Editor Isolation Contract (docs µservice)

## Purpose

Define the load-bearing isolation contract between tenant editor sessions, block-tree state, CRDT op streams, comment threads, suggestion state machines, share-link tokens, CDN cache, and embed-resolver snapshots. Docs is a multi-tenant hero product; per-tenant editor session isolation is the single biggest confidentiality control surface. Canonical reference for `oya-governance-editor-isolation-conformance` LEAN lane.

## Editor Session Isolation Model

### Per-tenant editor session boundary

Every editor session belongs to exactly one tenant, identified by the OIDC `tenant_id` claim. The session boundary applies to:

1. **Editor session state** (active drafts, cursor, viewport, undo history, presence) — Postgres + Valkey row-tagged by tenant_id.
2. **CRDT op stream** — WebSocket gateway lease is keyed (tenant_id, document_id); cross-tenant lease denied.
3. **Block-tree state** — every block carries tenant_id; queries filtered.
4. **Comment + suggestion threads** — scoped to (document_id, thread_id); tenant_id inherited from doc.
5. **Per-seat license attribution** — tenancy SDK lookup scoped to tenant.
6. **AI writing-assist invocations** — foundry-providers SDK call carries tenant context; cross-tenant prompt leakage denied.
7. **Embed-resolver snapshots** — cached per (embedding_tenant_id, source_ref) tuple; cross-tenant cache key prevented.
8. **Audit-chain seals** — emitted per (tenant_id, document_id, version_sha).

### Enforcement layers (defense-in-depth)

| Layer | Enforcement | Failure mode |
|---|---|---|
| OIDC at REST/WS upgrade | Reject if tenant_id claim missing or mismatched | 401 unauthorized; audit `docs_oidc_tenant_mismatch` |
| Cedar tenant-scope policy | Default-deny on all actions; permits only when principal.tenant_id == resource.tenant_id | Cedar denial; audit `docs_cedar_cross_tenant_attempt` |
| Postgres Row-Level Security (RLS) | Predicate on every row: `tenant_id = current_setting('app.current_tenant_id')` | Postgres returns empty result; audit `docs_postgres_rls_block` |
| Postgres block-level RLS (per ADR-DOCS-0004) | Predicate on `blocks` table: includes per-block ACL check | empty result; audit `docs_postgres_block_acl_block` |
| Per-tenant Postgres connection pool | Connection-level session variable carries tenant_id; rebinding requires re-auth | Pool returns 503; audit `docs_pool_rebinding_denied` |
| S3 per-tenant prefix | Tenant ID is part of the object key prefix; cross-prefix access denied at IAM | 403; audit `docs_s3_cross_tenant_block` |
| WebSocket gateway lease | Per (tenant_id, document_id) lease via Valkey; consistent-hash routing | WS upgrade rejected; audit `docs_ws_lease_cross_tenant` |
| Server-side stamping | Editor REST + WS handler overwrites any client-supplied tenant_id with OIDC claim | Spoofing attempt logged; no behavior change |

All eight layers must fail simultaneously for cross-tenant access. LEAN check `oya-governance-citus-rls-enforced` validates layers 3 + 4 + 5 at every PR.

## Per-Block ACL (per ADR-DOCS-0004)

### ACL model

Each block carries an `acl` field with three roles:

```text
BlockAcl {
  visibility: public | team_visible | private,
  principals: [user_id, ...],     // explicit allow-list when private
  teams: [team_id, ...],          // explicit team allow-list when team_visible
  inherited_from_doc: bool        // default true; false when block has explicit overrides
}
```

### Enforcement chain

| Layer | Enforcement |
|---|---|
| Cedar `per-block-acl.cedar` | refuses block reads when principal not in block's `principals` |
| Postgres RLS | predicate filters block-tree rows by per-block ACL |
| Application | server-side stamping; block ACL never client-mutable |
| LEAN check | `oya-check-per-block-acl` validates that every block-read path applies ACL |

### Per-block ACL drift detection

`oya-governance-share-acl-drift` weekly scan: compare per-block ACL records vs effective access logs; drift triggers Sev-2 + auto-revert.

## Cross-Tenant Collab — FORBIDDEN

Collaborative editing is bounded by tenant + document:

- Two users from same tenant editing same doc: CRDT merge applies.
- Two users from different tenants editing same doc: IMPOSSIBLE — docs are tenant-scoped.
- Two users from different tenants editing different docs: each in own session; no shared state.
- One user with a cross-tenant share grant: receives snapshot-of-doc only (Cedar refuses CRDT op publish from cross-tenant principal).

WebSocket gateway enforces:
```text
on WS message (subscriber_oidc, op):
  if subscriber_oidc.tenant_id != op.tenant_id:
    refuse + audit-emit docs_cross_tenant_collab_attempt
  if subscriber_oidc.document_id != op.document_id:
    refuse + audit-emit docs_cross_document_collab_attempt
  if subscriber.role == "cross_tenant_share_recipient":
    refuse + audit-emit docs_share_recipient_crdt_publish_attempt
  else:
    deliver
```

No exception path. Even council-operators inspecting tenant data must use auditor-scope JIT (read-only; cannot inject CRDT ops).

## CRDT Op Authenticity

Per ADR-DOCS-0001 (Loro CRDT) + threat-model T-T-01:

- Every CRDT op carries OIDC-derived author SPIFFE-identity at the WS gateway.
- Op envelope signed Ed25519 at the gateway before fan-out.
- LEAN check `oya-check-crdt-op-signature` validates every accepted op has a verifiable signature.
- Loro merge engine receives only signed ops; unsigned ops refused at adapter boundary.

## Embed-Resolver Source-Side ACL Passthrough

Per ADR-DOCS-0004 + threat-model T-I-03:

- Embed-resolver re-evaluates the SOURCE-side ACL at every fetch using the embedding doc's principal.
- Source µservice (workflow-studio / sheets / slides) evaluates its own Cedar policy; returns 403 or redacted placeholder if denied.
- Embed snapshot cached per (embedding_tenant_id, source_ref) with short TTL (≤ 5 min).
- Grant revocation propagates via Workflow event; cache invalidated.
- LEAN check `oya-check-embed-resolver-acl-passthrough` validates the resolver does not bypass source-side ACL.

## CDN Cache Isolation

### Cache-key partitioning

All docs static assets cached at CDN edge with key:

```text
cache_key = (asset_path, pack, version)
```

- No `hashed_tenant_id` ever in CDN cache key — docs static assets are tenant-agnostic (block-type schemas, design-system primitives, WASM editor bundle, KaTeX font files).
- Tenant content NEVER cached at CDN edge.

### Cache invalidation

- Per release: CDN purge of all (asset_path, pack, version) keys for the previous version.
- Purge SLI: ≤ 60s p99 propagation across edge nodes.

LEAN check `oya-governance-cdn-cache-key-tenant-isolated` validates CDN cache configuration against this rule.

## Browser-Side Isolation

### Strict CSP

```text
Content-Security-Policy:
  default-src 'self' https://cdn-<pack>.oyatie.dev;
  script-src 'self' 'wasm-unsafe-eval' 'nonce-<random>' https://cdn-<pack>.oyatie.dev;
  style-src 'self' 'unsafe-inline' https://cdn-<pack>.oyatie.dev;
  img-src 'self' data: https://cdn-<pack>.oyatie.dev https://attachments-<pack>.oyatie.dev;
  connect-src 'self' wss://docs-<pack>.oyatie.dev https://docs-<pack>.oyatie.dev;
  font-src 'self' https://cdn-<pack>.oyatie.dev;
  object-src 'none';
  frame-ancestors 'none';
  base-uri 'self';
  form-action 'self';
  upgrade-insecure-requests;
  require-trusted-types-for 'script';
  trusted-types docs-default;
```

### Trusted Types

Docs uses Trusted Types via the `docs-default` policy; all DOM sink usage (e.g., `innerHTML`-equivalent rendering of block content) goes through this policy.

### SRI on WASM chunks

Every WASM chunk in HTML carries an integrity attribute; mismatch refuses load + audit-emit `docs_wasm_sri_mismatch`.

## Operator Access (Trusted-but-Audited)

### Read-only with 2-person rule

Council operators may read tenant doc contents only via:
1. JIT elevation through OpenBao (TTL ≤ 4h).
2. 2-person rule: 2 council operators must co-sign the elevation.
3. Read pattern monitoring: `> 5 cross-tenant reads/min` triggers anomaly Sev-2 alert.
4. Every read audit-chain-sealed with operator identity.

### Write/elevate-permission FORBIDDEN

Operators cannot:
- Inject CRDT ops on tenant's behalf.
- Submit edits as tenant.
- Modify tenant's editor session state.
- Enable AI-assist features on tenant's behalf.
- Grant or revoke shares on behalf of tenant.

All write paths require tenant-owned OIDC; operator OIDC refused at write Cedar policy.

## Export Pipeline Isolation (per ADR-DOCS-0003)

- Per-job gVisor sandbox; tmpfs only; no network egress.
- Per-job tenant_id stamped on the export request; sandbox cannot access other tenants.
- Output validated against output-type schema (PDF/A, DOCX, etc.) before emission.
- Per-tenant export quota; cumulative export-second budget.
- LEAN check `oya-governance-export-sandbox-conformance` validates sandbox config.

## Verification

- `oya gate validate editor-isolation-conformance` — exit 0.
- `oya gate validate citus-rls-enforced` — exit 0.
- `oya gate validate cdn-cache-key-tenant-isolated` — exit 0.
- `oya gate validate crdt-op-signature` — exit 0.
- `oya gate validate per-block-acl` — exit 0.
- `oya gate validate embed-resolver-acl-passthrough` — exit 0.
- `oya gate validate export-sandbox-conformance` — exit 0.
- Quarterly chaos drill: induce cross-tenant editor session access attempt; verify rejection at every defense-in-depth layer.
- Annual external pen-test against editor surface.

## References

- ADR-0028 (audit-chain).
- ADR-0065 (Leptos for browser UI).
- ADR-0131 (per-microservice flat layout).
- ADR-0140 (Cedar policy enforcement).
- ADR-DOCS-0001 (CRDT — Loro).
- ADR-DOCS-0004 (per-block ACL).
- `microservices/docs/threat-model.md` T-T-01, T-I-01, T-I-02, T-I-03, T-E-01.
- `microservices/docs/dpia.md` R-01, R-02, R-16.
- `microservices/docs/policy/tenant-scope.cedar`.
- `microservices/docs/policy/data-residency.md`.
- OWASP ASVS v4.0 V12 (Web Service) + V14 (Configuration).
- Cedar v4 policy language — `cedarpolicy.com`.
- Strict CSP best practices — `web.dev/strict-csp`.
- Trusted Types W3C draft.
- `microservices/workflow-studio/policy/editor-isolation.md` — sibling reference.
