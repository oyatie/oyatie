# app/office reorg drain notes (`integ/office`)

## Ownership (rule 3e)

- **Forever home:** `app/office/**` (this rail).
- **Source (read-only):** `oya/office/**` on `origin/dev` until shrink-only delete lands on `integ/oya`.
- **Writes:** only under `app/office/**` on this tip.

## Completed

- Hygiene deepen: retargeted `oya/office/` + `microservices/office/` path cites → `app/office/` in README/manifest (forever-home authority). (this rail)

- Slice 1: invent inventory + forward-declared `manifest.json` (`869198b65`).
- **Wave-1 absorb:** 19-crate forest copied to `app/office/` (63 crate files). BUCK path cites rewritten `//oya/office/` → `//app/office/`.
- Substrate ports (tenant/authz/storage/search kernels + product kernel split) retained in-place; burn into capability homes deferred to follow-on slice.

## Inventory (absorbed forest)

19 crate directories (63 tracked crate files; forest only — no non-crate siblings on source):

| Dir | Face (heuristic) | Files |
|-----|------------------|------:|
| `oya-office-api-app` | app | 4 |
| `oya-office-authz-domain` | domain / substrate-port candidate | 3 |
| `oya-office-collab-domain` | domain | 3 |
| `oya-office-collab-gateway-app` | app | 4 |
| `oya-office-doc-domain` | domain | 3 |
| `oya-office-drive-api` | api lib | 3 |
| `oya-office-drive-api-app` | app | 4 |
| `oya-office-drive-domain` | domain | 3 |
| `oya-office-drive-worker` | worker | 4 |
| `oya-office-format-domain` | domain | 3 |
| `oya-office-format-worker` | worker | 4 |
| `oya-office-kernel` | product kernel | 3 |
| `oya-office-search-kernel` | kernel / substrate-port candidate | 3 |
| `oya-office-sheet-domain` | domain | 3 |
| `oya-office-sheets-api` | api lib | 3 |
| `oya-office-slide-domain` | domain | 3 |
| `oya-office-storage-kernel` | kernel / substrate-port candidate | 3 |
| `oya-office-tenant-domain` | domain / substrate-port candidate | 3 |
| `oya-office-web-app` | app | 4 |

## Substrate ports rewrite plan (follow-on; rewrite ≠ forever git-mv)

Judgment: substrate-shaped contracts burn into capability homes via redesign/rewrite (path + crate rename + consumer retarget). Wave-1 dual-homes them under `app/office/` so product crates stay buildable; capability rails own the burn.

| Crate (`app/office/`) | Intent | Proposed forever home | Notes |
|-----------------------|--------|-----------------------|-------|
| `oya-office-storage-kernel` | Object/metadata storage **port** | `storage/` | Port-trait / kernel layer; drop `oya-office-*` prefix. |
| `oya-office-search-kernel` | Tenant-scoped search/index **port** | search/indexing capability (pending registry) | Keep redaction contracts. |
| `oya-office-tenant-domain` | Tenant/quota/rate-limit/region | `tenancy/` (+ `iam/` if split) | Rewrite deps onto shared tenancy IDs. |
| `oya-office-authz-domain` | Authz / sharing / export | `iam/` authz surface | Extract shared policy; leave Office gates in product domain. |
| `oya-office-kernel` | Shared IDs, request context, audit | Split: generic → tenancy/iam; Office-only stays here | Do not promote whole crate as platform substrate. |

**In-forest consumers (Wave-1 tip census, BUCK deps):**

| Substrate crate | In-forest dependents |
|-----------------|----------------------|
| `oya-office-kernel` | authz, tenant, storage, search, collab, drive-*, format-*, doc/sheet/slide, sheets-api, web-app, workers, apps |
| `oya-office-authz-domain` | `drive-domain`, `drive-api` |
| `oya-office-storage-kernel` | (none in-forest BUCK — elevate external cites) |
| `oya-office-search-kernel` | (none in-forest BUCK — elevate external cites) |
| `oya-office-tenant-domain` | (none in-forest BUCK — elevate external cites) |

**Order (suggested):**

1. Land capability-side port traits (storage/search/tenancy/iam) with Office adapters behind ports.
2. Rewrite Office consumers onto capability ports; delete Office-prefixed substrate crates from `app/office/`.
3. Shrink-only delete `oya/office/**` on `integ/oya` after verify.
4. Hub retargets on tip-free `integ/specs`.

Sibling dual-home note (judgment, out of this rail): docs/sheets/slides may share surfaces — do not annex from `integ/office`.

## Remaining for later slices

1. **Substrate ports rewrite** per table (capability rails + this rail for consumer retarget).
2. **Shrink-only burn** of `oya/office/**` on `integ/oya`.
3. **Hub retargets** (`specs/**`, capability-registry `app_products`) on tip-free `integ/specs`.
4. **Crate rename** `oya-office-*` → destination naming (follow-on with substrate burn).

## Out of envelope

- `oya/office/**` deletes — `integ/oya` shrink-only only.
- `Cargo.lock` / root workspace membership — lock tip only.
- `specs/**` hub edits — `integ/specs` only.
- Sibling products under `oya/*` or `app/*` other than office.
- Capability-home writes (`storage/`, `tenancy/`, `iam/`) — those destination rails.
