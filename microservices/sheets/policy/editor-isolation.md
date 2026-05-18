---
doc_class: PolicySpec
title: Editor Isolation Contract
microservice: sheets
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-sheets + ops-security + council-design-system
deciders: axis-sheets, ops-security, council-architecture, council-design-system
related_adrs: [ADR-0028, ADR-0065, ADR-0103, ADR-0135, ADR-0131, ADR-0140, ADR-SHEETS-0006]
related_artifacts:
  - microservices/sheets/threat-model.md (T-I-01, T-I-04, T-I-06, T-I-07, T-I-08, T-T-07)
  - microservices/sheets/dpia.md (R-02, R-14, R-16, R-22)
  - microservices/sheets/policy/tenant-scope.cedar
review_cadence: quarterly + on every collab / share-ACL / range-ACL substrate change
doc_status: published
---

# Editor Isolation Contract (sheets µservice)

## Purpose

Define the load-bearing isolation contract between tenant workbook sessions, function-library, collab participants, CDN cache, XLSX import sandboxes, AI-formula bridges, and per-range ACL. Sheets is a multi-tenant hero product; per-tenant workbook isolation + per-range ACL granularity (ADR-SHEETS-0006) is the largest confidentiality control surface. Canonical reference for `oya-governance-editor-isolation-conformance` LEAN lane.

## Workbook Session Isolation Model

### Per-tenant workbook session boundary

Every workbook session belongs to exactly one tenant. The session boundary applies to:

1. **Workbook metadata + cell rows** — Postgres + Redis tagged by tenant_id; Arrow/Parquet large-sheet blocks per-(tenant, workbook, sheet) key.
2. **CRDT op stream** — WebSocket gateway lease is keyed (tenant_id, workbook_id); cross-tenant lease denied.
3. **Per-seat license attribution** — tenancy SDK lookup scoped to tenant.
4. **Per-range ACL** — Cedar policy fragments enforce read/edit per range_id within tenant scope.
5. **AI-formula invocations** — foundry-runtime SDK call carries tenant context.
6. **Connected-sheets external-query results** — query result materialised only into the requesting workbook within tenant.
7. **XLSX import/export jobs** — gVisor sandbox per tenant; output S3 path tenant-scoped.
8. **Audit-chain seals** — emitted per (tenant_id, workbook_id, sheet_id, cell_ref, version_sha).

### Enforcement layers (defense-in-depth)

| Layer | Enforcement | Failure mode |
|---|---|---|
| OIDC at REST/WS upgrade | Reject if tenant_id claim missing or mismatched | 401 unauthorized; audit `sheets_oidc_tenant_mismatch` |
| Cedar tenant-scope policy | Default-deny on all actions; permits only when principal.tenant_id == resource.tenant_id | Cedar denial; audit `sheets_cedar_cross_tenant_attempt` |
| Cedar per-range ACL (ADR-SHEETS-0006) | Default-deny for read/write outside `allowed_range_acls` | Cedar denial; audit `sheets_range_acl_violation` |
| Postgres Row-Level Security (RLS) | Predicate on every row: `tenant_id = current_setting('app.current_tenant_id')` | Postgres returns empty result; audit `sheets_postgres_rls_block` |
| Citus partition | Tenant_id is partition key | Cross-shard query denied at coordinator |
| Per-tenant Postgres connection pool | Connection-level session variable carries tenant_id | Pool returns 503; audit `sheets_pool_rebinding_denied` |
| WebSocket gateway lease | Per (tenant_id, workbook_id) lease via Redis | WS upgrade rejected; audit `sheets_ws_lease_cross_tenant` |
| Server-side stamping | Editor REST + WS handler overwrites any client-supplied tenant_id with OIDC claim | Spoofing attempt logged |
| gVisor sandbox per XLSX import job | Sandbox boundary; tenant data inside cannot reach other tenant | Sandbox escape would breach (mitigated by gVisor mature posture) |

All nine layers must fail simultaneously for cross-tenant access. LEAN check `oya-governance-citus-rls-enforced` validates layers 4 + 5 + 6 at every PR. LEAN check `oya-governance-sheets-range-acl-cedar-required` validates layer 3 on every render-path PR.

## Per-Range ACL Granularity (ADR-SHEETS-0006)

### ACL model

Per-range named-ACL is the canonical column/range-level access control granularity for Sheets:

```text
workbook → sheets → named ranges → ACL entries
                                   ├── principal: TenantUser::<oidc_sub>
                                   ├── decision: allow_read | allow_edit | deny
                                   └── audit_seal: Ed25519(tenant_id, workbook_id, range_id, principal, decision, timestamp)
```

Granularity tier choice (per ADR-SHEETS-0006):
- **Per-cell ACL**: too fine-grained; cost-prohibitive at 1M-cell workbook scale; rejected.
- **Per-range named-ACL**: ✅ chosen; matches Google Sheets protected-ranges UX with finer Cedar-policy-fragment enforcement.
- **Whole-sheet ACL**: too coarse; rejected.

### Cedar policy fragment (excerpt; see `tenant-scope.cedar` PERMIT 9 and FORBID)

```cedar
permit (
  principal in TenantOperator::?p,
  action in [Action::"read_cell", Action::"read_range"],
  resource in CellOrRange::?cr
) when {
  resource has range_id &&
  principal has allowed_range_acls &&
  resource.range_id in principal.allowed_range_acls
};

forbid (
  principal in TenantOperator::?p,
  action in [Action::"read_cell", Action::"read_range", Action::"write_cell", Action::"write_range", Action::"write_formula"],
  resource in CellOrRange::?cr
) when {
  resource has range_id &&
  principal has allowed_range_acls &&
  !(resource.range_id in principal.allowed_range_acls)
};
```

### ACL change audit

Every share-permission or per-range ACL change emits `share_acl_changed` event with old/new ACL hashes, signed Ed25519. Per ADR-0028.

### ACL drift detection

Quarterly drift audit compares Cedar policy fragments to Postgres-stored ACL rows; mismatch fires Sev-1 alert per failure-mode FM-05.

## Per-Pack Function-Library Scoping

### Function-library model

The formula-engine ≥400-function library (per ADR-SHEETS-0002) ships INSIDE the Sheets binary; there is NO runtime function-library loading. Per-pack jurisdiction overlay determines which functions are exposed in the cell-grid auto-complete UX:

- **Base library (~400 fns)**: math/logical/lookup/statistical/financial/text/date/array (Excel-reference subset per ADR-SHEETS-0002).
- **Per-pack overlays**: small additions per pack (e.g., pack-us-healthcare adds HIPAA-aware redaction-helper functions; pack-eu adds GDPR-aware consent-check functions).

Function-library upgrade requires:
1. Sheets binary release (function-library compiled-in).
2. Signed-commit policy on the function-library crate.
3. CI lane `oya-governance-sheets-formula-engine-correctness` runs the LibreOffice Calc reference corpus per ADR-SHEETS-0002.
4. Rollback runbook `runbooks/formula-engine-rollback.md` ready before merge.

## Cross-Tenant Collab — FORBIDDEN

Collaborative editing is bounded by tenant + workbook:

- Two users from same tenant editing same workbook: CRDT merge applies.
- Two users from different tenants editing same workbook: IMPOSSIBLE — workbooks are tenant-scoped.
- Two users from different tenants editing different workbooks: each in own session; no shared state.

WebSocket gateway enforces:
```text
on WS message (subscriber_oidc, op):
  if subscriber_oidc.tenant_id != op.tenant_id:
    refuse + audit-emit sheets_cross_tenant_collab_attempt
  if subscriber_oidc.workbook_id != op.workbook_id:
    refuse + audit-emit sheets_cross_workbook_collab_attempt
  else:
    deliver
```

No exception path.

## CDN Cache Isolation

### Cache-key partitioning

```text
cache_key = (asset_path, pack, hashed_tenant_id?, version)
```

- `hashed_tenant_id` is OPTIONAL: included only for tenant-specific content.
- Without `hashed_tenant_id`: tenant-agnostic content (WASM bundles, spec schema, design-system primitives, function-library descriptor catalog).

### Cache invalidation

- Per release: CDN purge of all (asset_path, pack, version) keys for the previous version.
- Purge SLI: ≤ 60s p99 propagation across edge nodes.
- Browser-side version pin via `<meta data-sheets-version="...">`.

### Tenant-specific content forbidden at CDN

**No tenant workbook content is EVER cached at CDN edge.** Workbook state lives in Postgres + Redis + S3 + Arrow/Parquet only; CDN serves only tenant-agnostic assets.

LEAN check `oya-governance-cdn-cache-key-tenant-isolated` validates CDN cache configuration against this rule.

## XLSX Import / Export Sandbox Isolation

### gVisor user-mode sandbox

Every XLSX import + export job runs inside its own gVisor user-mode sandbox:
- Restricted syscall surface.
- Egress denied (no network from inside sandbox).
- Filesystem: read-only tempdir per job.
- RAM + CPU + wall-clock budget enforced per job.
- One sandbox instance per tenant per job; sandbox terminated on completion.

### AV-scan defense-in-depth

Before XLSX enters the sandbox, ClamAV + OPSWAT MetaDefender scan the file:
- Both must pass.
- Positive scan: file refused; quarantine bucket; tenant notified; audit-emit `sheets_xlsx_upload_av_positive`.
- File-size cap (default 200 MB); decompression-bomb detection (> 100× expansion ratio refused); formula-bomb detection (> 10M cell formulas refused).

LEAN check `oya-governance-sheets-import-sandboxed-and-avscan-required` validates import pipeline at PR-time.

## Browser-Side Isolation

### Strict CSP

```text
Content-Security-Policy:
  default-src 'self' https://cdn-<pack>.oyatie.dev;
  script-src 'self' 'wasm-unsafe-eval' 'nonce-<random>' https://cdn-<pack>.oyatie.dev;
  style-src 'self' 'unsafe-inline' https://cdn-<pack>.oyatie.dev;
  img-src 'self' data: https://cdn-<pack>.oyatie.dev;
  connect-src 'self' wss://sheets-<pack>.oyatie.dev https://sheets-<pack>.oyatie.dev;
  font-src 'self' https://cdn-<pack>.oyatie.dev;
  object-src 'none';
  frame-ancestors 'none';
  base-uri 'self';
  form-action 'self';
  upgrade-insecure-requests;
  require-trusted-types-for 'script';
  trusted-types sheets-default;
```

### Trusted Types

Sheets uses Trusted Types via the `sheets-default` policy.

### SRI on WASM chunks

Every WASM chunk in HTML:
```html
<script src="/v1.2.3/cell-grid.wasm.js" integrity="sha384-<hash>" crossorigin="anonymous"></script>
```

Mismatch → browser refuses load → Sheets falls back to "Sheets unavailable — please reload" banner + audit-emit `sheets_wasm_sri_mismatch`.

## Operator Access (Trusted-but-Audited)

### Read-only with 2-person rule

Council operators may read tenant workbook contents only via:
1. JIT elevation through OpenBao (TTL ≤ 4h).
2. 2-person rule: 2 council operators must co-sign the elevation.
3. Read pattern monitoring: `> 5 cross-tenant reads/min` triggers anomaly Sev-2 alert.
4. Every read audit-chain-sealed with operator identity.

### Write/elevate-permission FORBIDDEN

Operators cannot:
- Inject CRDT ops on tenant's behalf.
- Write cells as tenant.
- Modify tenant's per-range ACL.
- Enable AI-formula features on tenant's behalf.

All write paths require tenant-owned OIDC; operator OIDC refused at write Cedar policy.

## Verification

- `oya gate validate editor-isolation-conformance --microservice sheets` — exit 0.
- `oya gate validate citus-rls-enforced --microservice sheets` — exit 0.
- `oya gate validate sheets-range-acl-cedar-required --microservice sheets` — exit 0.
- `oya gate validate cdn-cache-key-tenant-isolated --microservice sheets` — exit 0.
- `oya gate validate sheets-import-sandboxed-and-avscan-required --microservice sheets` — exit 0.
- Quarterly chaos drill: induce cross-tenant workbook access attempt; verify rejection at every defense-in-depth layer.
- Annual external pen-test against editor surface.

## References

- ADR-0028 (audit-chain).
- ADR-0065 (Leptos for browser UI).
- ADR-0135 (sheets net-new µservice).
- ADR-0131 (per-microservice flat layout).
- ADR-0140 (Cedar policy enforcement).
- ADR-SHEETS-0006 (per-range ACL granularity).
- `microservices/sheets/threat-model.md` T-I-01, T-I-04, T-I-06, T-I-07, T-I-08, T-T-07.
- `microservices/sheets/dpia.md` R-02, R-14, R-16, R-22.
- `microservices/sheets/policy/tenant-scope.cedar`.
- `microservices/sheets/policy/data-residency.md`.
- OWASP ASVS v4.0 V12 (Web Service) + V14 (Configuration).
- Cedar v4.2 LTS policy language — `cedarpolicy.com`.
- Strict CSP best practices — `web.dev/strict-csp`.
- Trusted Types W3C draft.
- gVisor — `gvisor.dev`.
- ClamAV — `clamav.net`.
- OPSWAT MetaDefender — `opswat.com/products/metadefender`.
