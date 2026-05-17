---
doc_class: PolicySpec
title: Editor Isolation Contract
microservice: workflow-studio
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-workflow + ops-security + council-design-system
deciders: axis-workflow, ops-security, council-architecture, council-design-system
related_adrs: [ADR-0028, ADR-0065, ADR-0103, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/workflow-studio/threat-model.md (T-I-01, T-I-04, T-I-07, T-I-08)
  - microservices/workflow-studio/dpia.md (R-02, R-16)
  - microservices/workflow-studio/policy/tenant-scope.cedar
review_cadence: quarterly + on every collab/node-library substrate change
doc_status: published
---

# Editor Isolation Contract (workflow-studio µservice)

## Purpose

Define the load-bearing isolation contract between tenant editor sessions, node libraries, collab participants, CDN cache, and replay-debugger streams. Studio is a multi-tenant hero product; per-tenant editor session isolation is the single biggest confidentiality control surface. Canonical reference for `oya-governance-editor-isolation-conformance` LEAN lane.

## Editor Session Isolation Model

### Per-tenant editor session boundary

Every editor session belongs to exactly one tenant, identified by the OIDC `tenant_id` claim. The session boundary applies to:

1. **Editor session state** (active drafts, cursor, viewport, undo history) — Postgres + Redis row-tagged by tenant_id.
2. **CRDT op stream** — WebSocket gateway lease is keyed (tenant_id, definition_id); cross-tenant lease denied.
3. **Per-seat license attribution** — tenancy SDK lookup scoped to tenant.
4. **LLM-assist invocations** — foundry-providers SDK call carries tenant context; cross-tenant prompt leakage denied.
5. **Debugger streams** — engine SDK stream filters by tenant.
6. **Audit-chain seals** — emitted per (tenant_id, definition_id, version_sha).

### Enforcement layers (defense-in-depth)

| Layer | Enforcement | Failure mode |
|---|---|---|
| OIDC at REST/WS upgrade | Reject if tenant_id claim missing or mismatched | 401 unauthorized; audit `studio_oidc_tenant_mismatch` |
| Cedar tenant-scope policy | Default-deny on all actions; permits only when principal.tenant_id == resource.tenant_id | Cedar denial; audit `studio_cedar_cross_tenant_attempt` |
| Postgres Row-Level Security (RLS) | Predicate on every row: `tenant_id = current_setting('app.current_tenant_id')` | Postgres returns empty result; audit `studio_postgres_rls_block` |
| Citus partition | Tenant_id is partition key; queries route to single shard | Cross-shard query denied at coordinator |
| Per-tenant Postgres connection pool | Connection-level session variable carries tenant_id; rebinding requires re-auth | Pool returns 503; audit `studio_pool_rebinding_denied` |
| WebSocket gateway lease | Per (tenant_id, definition_id) lease via Redis; consistent-hash routing | WS upgrade rejected; audit `studio_ws_lease_cross_tenant` |
| Server-side stamping | Editor REST + WS handler overwrites any client-supplied tenant_id with OIDC claim | Spoofing attempt logged; no behavior change |

All seven layers must fail simultaneously for cross-tenant access. LEAN check `oya-governance-citus-rls-enforced` validates layers 3 + 4 + 5 at every PR.

## Per-Pack Node-Library Scoping

### Library catalog model

Node libraries are organized per-pack:

```text
node-library-registry
├── pack-agentic (6 domain libraries)
│   ├── agentic-orchestration  (LLM call nodes, agent loops)
│   └── ...
├── pack-business
│   ├── crm-integrations  (Salesforce, HubSpot, Pipedrive)
│   ├── erp-integrations
│   └── ...
├── pack-healthcare
│   ├── ehr-integrations  (FHIR R4, HL7 v2)
│   └── ...
├── pack-supply-chain
├── pack-delivery
└── pack-dev  (developer tooling nodes; build/deploy/test)
```

Each library:
- Signed (Ed25519) by a per-pack signing key in OpenBao.
- Versioned via SHA-256 of the canonicalized descriptor bundle.
- Distributed via CDN edge (per-pack cache key).
- Loaded by Studio only when tenant's pack tag matches.

### Cross-pack node access (forbidden)

Tenants in pack-kr cannot load pack-eu libraries unless they have multi-pack entitlement (rare; per-tenant DPA exception).

Cedar enforcement:
```cedar
forbid (
  principal in TenantOperator::?p,
  action == Action::"load_node_library",
  resource in NodeLibrary::?l
) when {
  resource has pack &&
  principal has packs &&
  !(resource.pack in principal.packs)
};
```

### Library signature verification

Every node library load path verifies:
1. Per-pack public key in OpenBao validates the descriptor bundle's Ed25519 signature.
2. SHA-256 of descriptor bundle matches the published `version_sha`.
3. Signing key NOT in OpenBao revocation list (CRL).

Failure: refuse load; render "Node library unavailable — contact administrator" banner; audit-emit `studio_node_library_signature_invalid`.

## Cross-Tenant Collab — FORBIDDEN

Collaborative editing is bounded by tenant + definition:

- Two users from same tenant editing same definition: CRDT merge applies.
- Two users from different tenants editing same definition: IMPOSSIBLE — definitions are tenant-scoped.
- Two users from different tenants editing different definitions: each in own session; no shared state.

WebSocket gateway enforces:
```text
on WS message (subscriber_oidc, op):
  if subscriber_oidc.tenant_id != op.tenant_id:
    refuse + audit-emit studio_cross_tenant_collab_attempt
  if subscriber_oidc.definition_id != op.definition_id:
    refuse + audit-emit studio_cross_definition_collab_attempt
  else:
    deliver
```

No exception path. Even council-operators inspecting tenant data must use auditor-scope JIT (read-only; cannot inject CRDT ops).

## CDN Cache Isolation

### Cache-key partitioning

All Studio static assets cached at CDN edge with key:

```text
cache_key = (asset_path, pack, hashed_tenant_id?, version)
```

- `hashed_tenant_id` is OPTIONAL: included only for tenant-specific content (none in M03 launch; reserved for post-GA per-tenant branding which is iframed).
- Without `hashed_tenant_id`: tenant-agnostic content (WASM bundles, spec schema, node library descriptors).

### Cache invalidation

- Per release: CDN purge of all (asset_path, pack, version) keys for the previous version.
- Purge SLI: ≤ 60s p99 propagation across edge nodes.
- Browser-side version pin: HTML `<meta data-studio-version="...">`; mismatch triggers hard reload.

### Tenant-specific content forbidden at CDN

**No tenant draft content is EVER cached at CDN edge.** Editor session state lives in Postgres + Redis only; CDN serves only:
- WASM bundles (per-release; tenant-agnostic).
- Spec schema (tenant-agnostic).
- Node library descriptors (per-pack; tenant-agnostic).
- Design-system primitives (tenant-agnostic).

LEAN check `oya-governance-cdn-cache-key-tenant-isolated` validates CDN cache configuration against this rule.

## Browser-Side Isolation

### Strict CSP

```text
Content-Security-Policy:
  default-src 'self' https://cdn-<pack>.oyatie.dev;
  script-src 'self' 'wasm-unsafe-eval' 'nonce-<random>' https://cdn-<pack>.oyatie.dev;
  style-src 'self' 'unsafe-inline' https://cdn-<pack>.oyatie.dev;
  img-src 'self' data: https://cdn-<pack>.oyatie.dev;
  connect-src 'self' wss://studio-<pack>.oyatie.dev https://workflow-engine-<pack>.oyatie.dev;
  font-src 'self' https://cdn-<pack>.oyatie.dev;
  object-src 'none';
  frame-ancestors 'none';
  base-uri 'self';
  form-action 'self';
  upgrade-insecure-requests;
  require-trusted-types-for 'script';
  trusted-types studio-default;
```

### Trusted Types

Studio uses Trusted Types via the `studio-default` policy; all DOM sink usage (e.g., `innerHTML`-equivalent) goes through this policy. Leptos virtual-DOM is the default surface; raw DOM access through Trusted Types-validated paths only.

### SRI on WASM chunks

Every WASM chunk in HTML:
```html
<script src="/v1.2.3/canvas.wasm.js" integrity="sha384-<hash>" crossorigin="anonymous"></script>
```

Mismatch → browser refuses load → Studio falls back to "Editor unavailable — please reload" banner + audit-emit `studio_wasm_sri_mismatch`.

## Operator Access (Trusted-but-Audited)

### Read-only with 2-person rule

Council operators may read tenant draft contents only via:
1. JIT elevation through OpenBao (TTL ≤ 4h).
2. 2-person rule: 2 council operators must co-sign the elevation.
3. Read pattern monitoring: `> 5 cross-tenant reads/min` triggers anomaly Sev-2 alert.
4. Every read audit-chain-sealed with operator identity.

### Write/elevate-permission FORBIDDEN

Operators cannot:
- Inject CRDT ops on tenant's behalf.
- Submit specs as tenant.
- Modify tenant's editor session state.
- Enable LLM-assist features on tenant's behalf.

All write paths require tenant-owned OIDC; operator OIDC refused at write Cedar policy.

## Verification

- `oya gate validate editor-isolation-conformance` — exit 0.
- `oya gate validate citus-rls-enforced` — exit 0.
- `oya gate validate cdn-cache-key-tenant-isolated` — exit 0.
- `oya gate validate node-library-signature-verification` — exit 0.
- Quarterly chaos drill: induce cross-tenant editor session access attempt; verify rejection at every defense-in-depth layer.
- Annual external pen-test against editor surface.

## References

- ADR-0028 (audit-chain).
- ADR-0065 (Leptos for browser UI).
- ADR-0131 (per-microservice flat layout).
- ADR-0140 (Cedar policy enforcement).
- `microservices/workflow-studio/threat-model.md` T-I-01, T-I-04, T-I-07, T-I-08.
- `microservices/workflow-studio/dpia.md` R-02, R-16.
- `microservices/workflow-studio/policy/tenant-scope.cedar`.
- `microservices/workflow-studio/policy/data-residency.md`.
- OWASP ASVS v4.0 V12 (Web Service) + V14 (Configuration).
- Cedar v4 policy language — `cedarpolicy.com`.
- Strict CSP best practices — `web.dev/strict-csp`.
- Trusted Types W3C draft.
