# app/office reorg drain notes (`integ/office`)

## Ownership (rule 3e)

- **Forever home:** `app/office/**` (this rail).
- **Source (read-only):** `oya/office/**` on `origin/dev` — 19-crate forest, **no** top-level `Cargo.toml` / product manifest.
- **Writes:** only under `app/office/**` on this tip.
- **This slice:** inventory-only + forward-declared `manifest.json`. **No crate code moves.**

## Inventory (`origin/dev:oya/office`, 2026-08-10)

19 crate directories (63 tracked files total; forest only — no non-crate siblings):

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

Source authority for this table: `git ls-tree` of `origin/dev:oya/office` (no top-level manifest on that tip).

## Completed (this rail)

- Slice 1: create `app/office/REORG-DRAIN.md` + forward-declared inventory `app/office/manifest.json`.
- Crate dirs listed; source paths remain under `oya/office/**` until a later absorb slice.

## Substrate ports rewrite plan (follow-on; rewrite ≠ git-mv)

Office currently vendors substrate-shaped contracts inside the product forest. They must **burn into capability homes** via redesign/rewrite (path + crate rename + consumer retarget), not a blind tree move into `app/office/`.

| Source crate (`oya/office/`) | Intent | Proposed forever home (judgment) | Notes |
|------------------------------|--------|----------------------------------|-------|
| `oya-office-storage-kernel` | Provider-neutral object/metadata storage **port** | `storage/` (capability ports face) | Docstring already says port-trait / kernel layer; stop prefixing `oya-office-*`. |
| `oya-office-search-kernel` | Tenant-scoped search/index **port** | search/indexing capability (or owning substrate once registered) | Keep redaction contracts; drop Office-only naming. |
| `oya-office-tenant-domain` | Tenant/quota/rate-limit/region allowlist contracts | `tenancy/` (+ `iam/` if control-plane splits) | Depends only on `oya-office-kernel` today — rewrite deps onto shared tenancy IDs. |
| `oya-office-authz-domain` | Authz / sharing / export / audit-bound access | `iam/` authz surface | G083 tenant-security baseline spans Drive/API/search/collab/storage — extract shared policy, leave Office-specific gates in product domain. |
| `oya-office-kernel` | Shared IDs, request context, audit shape | Split: generic IDs → tenancy/iam kernels; Office-only types stay as `app/office` product kernel | Do **not** promote the whole crate as a platform substrate. |

**Order (suggested):**

1. Inventory consumers of the five crates above (BUCK + Cargo path deps) — still on `oya/office/**` until absorb.
2. Land capability-side port traits (storage/search/tenancy/iam) with Office adapters behind ports.
3. Absorb remaining Office product crates into `app/office/**` (drive/doc/sheet/slide/collab/format + apps/workers).
4. Rewrite Office consumers onto capability ports; delete Office-prefixed substrate crates.
5. Shrink-only delete `oya/office/**` on `integ/oya` after verify.

Sibling dual-home note (judgment, out of this rail): docs/sheets/slides may share surfaces with this forest — do not annex from `integ/office`.

## Remaining for later slices

1. **Absorb product crates** into `app/office/**` (still deferred — not this tip).
2. **Substrate ports rewrite** per table above (capability rails + this rail for consumer retarget).
3. **Shrink-only burn** of `oya/office/**` on `integ/oya`.
4. **Hub retargets** (`specs/**`, capability-registry `app_products`) on tip-free `integ/specs`.

## Out of envelope

- `oya/office/**` deletes — `integ/oya` shrink-only only.
- `Cargo.lock` / root workspace membership — lock tip only.
- `specs/**` hub edits — `integ/specs` only.
- Sibling products under `oya/*` or `app/*` other than office.
