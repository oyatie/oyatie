# G026 product `app/` face-birth design — 2026-08-02

State: **PLANNING_ONLY — PRODUCT-ROOT SKELETON MAP, NOT A MOVE PLAN, NOT ACTIVATED**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements `G026-OYA-PRODUCT-TAIL-CENSUS-2026-08-02.md` and `G026-CI-RUNTIME-FACE-BIRTH-DESIGN-2026-08-02.md`.  
No path move, Cargo/BUCK rewrite, registry edit, push, or activation occurred.

## Purpose

Prove the **minimal honest** product-root birth contract for the **90** non-intelligence, non-CI crates under **30** `oya/` product tops **before** any executable product move-plan JSON is authored.

This is deliberately **not** a 90-row destination table. Most product crates lack a closed face grammar today (`domain` / `usecase` / `api` / `rest` / `grpc` / `service` / `infrastructure` / bare `*-app`). Inventing `app/<product>/{core,ports,adapters,facade}/…` leaves for every crate would be fiction.

## Authority (binding)

| Source | Binding claim |
|---|---|
| ADR-0562 §3 rule #5 | deployable surface composing **2+ capabilities** for a tenant → `app/<product>/` |
| ADR-0562 §6 | single-capability "app" is a **mis-placed facade**; membership lint enforces 2+ |
| ADR-0615 §1 | substrate/product split confirmed; product never homes under a capability `facade/` |
| ADR-0615 §2 Q12 | `console` = web-shell **substrate**; multi-capability ops views → `app/ops-console/<vertical>` |
| ADR-0615 §6 | `app/healthcare` contexts + separate `app/health-diagnostics` + separate `app/social` |
| `specs/capability-registry.json#app_products_note` | SaaS vertical roster is illustrative; **exact app roster is enumerated by migration lanes**, not closed by the registry |
| `specs/capability-registry.json#meta_directories[app/]` | composition ring; `owns_crates: true`; never defines a capability engine |
| Immutable tip probe | `app/` **ABSENT** on `origin/dev` (zero children) |

Closed capability face grammar (`core|ports|adapters|facade`) applies **inside a registered capability**. It is **not** automatically the internal grammar of `app/<product>/`. Product internal layout is a separate migration-lane decision (context dirs and/or clean-arch seams). Do not force `app/office/core/…` without an accepted product-layout ADR/lane.

## Source inventory (90 crates / 30 tops)

From the G026 product-tail census (immutable tip; intelligence 78 and CI 12 excluded):

| Crates | Top |
|---:|---|
| 19 | office |
| 14 | community |
| 8 | application |
| 6 | itsm |
| 5 | hr |
| 5 | payroll |
| 3 | crm |
| 2 each | plant-maintenance, production-planning, quality-management, real-estate, supply-chain-planning, treasury, warehouse |
| 1 each | contract-lifecycle-management, design-collaboration, docs, financial-planning, global-trade, incident-management, learning-management, marketing-automation, notes, performance-management, sheets, sites, slides, translate, whiteboard, workplace-integration |

**Role histogram (path/name heuristic, not face authority):** app-ish 23, adapter 4, kernel/core 3, port-ish 1, **other 59**. The 59 "other" crates are why a per-crate face map is refused here.

## Product-root roster (proposed birth set)

Roots below are **named destinations only**. Birth means creating the empty `app/<product>/` tree (OWNERS + charter stub + optional context placeholders) so later one-PR moves have a non-colliding home. Birth does **not** move crates.

### A. Authority-listed SaaS verticals with live crate mass (primary)

| Product root | Source tops (crates) | Authority | 2+ capability proof status |
|---|---|---|---|
| `app/office` | `oya/office` (19) + office-context tops with cargo: `docs,sheets,slides,notes,sites,whiteboard,design-collaboration,translate` (8×1=8) → **27 crates candidate cone** | `app_products_note` lists `office/docs/sheets/slides/notes/sites/whiteboard/calendar/forms/design-collaboration` and `translate` | **PARTIAL** — office crates mostly internal + `policy`/`data` edges; full multi-capability composition not yet graph-proven for every leaf. Root birth still authorized by registry note; **crate MOVE blocked** until per-cone 2+ proof or owner composition declaration |
| `app/community` | `oya/community` (14) | `app_products_note` names `community` | **WEAK on registered capabilities** — measured edges are mostly `libs/oya-shared-*` + `oya-data-boundary-kernel`; not 2 registered capability facades. Root birth OK as named vertical; MOVE blocked pending composition declaration (likely storage/data + messaging/iam when real PEPs land) |
| `app/crm` | `oya/crm` (3) | named | WEAK (mostly `data` boundary) |
| `app/hr` | `oya/hr` (5) | named | WEAK |
| `app/payroll` | `oya/payroll` (5) | named | WEAK |
| `app/itsm` | `oya/itsm` (6) | named | WEAK (`policy` only detected) |
| `app/incident-management` | `oya/incident-management` (1) | named | UNPROVEN |
| `app/treasury` | `oya/treasury` (2) | named | WEAK |
| `app/supply-chain-planning` | `oya/supply-chain-planning` (2) | named | WEAK |
| `app/production-planning` | `oya/production-planning` (2) | named | WEAK |
| `app/plant-maintenance` | `oya/plant-maintenance` (2) | named | WEAK |
| `app/warehouse` | `oya/warehouse` (2) | named | WEAK |
| `app/real-estate` | `oya/real-estate` (2) | named | WEAK |
| `app/global-trade` | `oya/global-trade` (1) | named | WEAK |
| `app/learning-management` | `oya/learning-management` (1) | named | UNPROVEN |
| `app/performance-management` | `oya/performance-management` (1) | named | UNPROVEN |
| `app/quality-management` | `oya/quality-management` (2) | named | WEAK |
| `app/contract-lifecycle-management` | `oya/contract-lifecycle-management` (1) | named | UNPROVEN |
| `app/financial-planning` | `oya/financial-planning` (1) | named | UNPROVEN |
| `app/marketing-automation` | `oya/marketing-automation` (1) | named | UNPROVEN |

### B. Authority-listed roots with **zero crates today** (scaffold-only when built)

Do **not** birth these empty roots in the first product PR unless a lane needs the parking destination for non-code shell relocation (G030/G026 hygiene). ADR-0615 already fixed names:

| Product root | Source shells / notes | Crates today |
|---|---|---:|
| `app/healthcare` | contexts: emr, pharmacy, patient-monitoring, healthcare-integration, emergency, imaging | 0 |
| `app/health-diagnostics` | `oya/diagnostics` (clinical lab; not healthcare context) | 0 |
| `app/social` | separate product (not healthcare); may relate to community later — **do not merge without owner ruling** | 0 |
| `app/connect` | named in note; engines may stay `comms` | 0 crates under that top |
| `app/payments` | named; non-code shell today | 0 |

### C. Explicitly **not** product-root birth targets

| Source | Why not `app/<that-name>` | Disposition class |
|---|---|---|
| `oya/application` (8 crates) | Not in `app_products_note` roster as a SaaS vertical. Manifest self-describes `product-developer-application-shell`. Shell frontend Cargo description cites **console** (ADR-0393). ADR-0615 Q12 homes shell substrate under **`console`**, multi-cap ops views under `app/ops-console/<vertical>`. | **OWNER_SPLIT_REQUIRED** — see § Application partition |
| `oya/workplace-integration` (1) | Not named as app product; scaffold smell | KEEP_IN_OYA until owner classifies |
| CI tops (`ci-*`) | Capability `ci` — see CI runtime face-birth design | not product |
| `oya/intelligence/**` | G024 remainder | not G026 product |

## Application partition (blocking honesty)

`oya/application` is a **mixed bag**, not one product root:

| Crate | Measured edges (path deps) | Tentative class | Move target class |
|---|---|---|---|
| `oya-application-shell-frontend` | shared OIDC + platform contracts; self-described console portal shell | **console substrate** | future `console/facade/…` (or console leaf per console move lane) — **not** `app/application` |
| `oya-application-app` | **many** registered capabilities: intelligence, audit, cell, iam, data, observability, messaging, secrets, tenancy, network (+ libs) | **true multi-capability composition root** | candidate `app/…` **or** console composition facade — **owner must pick the product name**; do not invent `app/application` |
| `oya-workspace-{chat,drive,forms,meet}-api` | each pairs a substrate facade/domain (comms / storage / workflow) + data-boundary | **thin workspace API adapters** over substrate | likely stay with substrate facade *or* land under whichever product owns the workspace surface after owner ruling; **not** automatic office |
| `oya-saas-plugin-app` | `marketplace/core/plugin-kernel` only | single-capability composition | **marketplace facade** candidate (mis-placed if forced into app/) |
| `oya-cloud-surface-domain` | data-boundary only | domain residual | KEEP until owner |

**Hard rule:** do not birth `app/application`. The string "application" is a historical µservice name (ADR-0106), not a SaaS vertical.

## Collision check

Immutable probes on tip:

- `app` → **ABSENT**
- `app/community`, `app/office`, `app/crm`, … → **ABSENT**
- Top-level `office`, `community`, `healthcare` outside `oya/` → **ABSENT**

First birth PR that creates `app/` must also satisfy membership-lint allowlist expectations (ADR-0562 §6 already names `app` as allowed top-level meta dir). Confirm live membership-lint policy includes `app` before merge (read-only check at activation time; not edited here).

## What "face birth" means for products (narrow)

For products, birth is **root (+ optional context) creation**, not capability-face package creation:

```
app/<product>/
  OWNERS
  README.md          # charter: composes which capabilities; not an engine
  contexts/…         # ONLY when authority names contexts (office, healthcare)
```

Optional later, per product lane (not this design):

- clean-arch package folds inside a context, **or**
- keep current `domain/usecase/api/app` package names under `app/<product>/crates/…` until a product-layout ADR lands.

**Forbidden in birth PRs:**

- 90-row move-plan JSON
- inventing empty `app/<product>/core|ports|adapters|facade` leaves "for symmetry" with capabilities
- deleting `oya/<product>/` trees
- folding `social` into `community` or `health-diagnostics` into `healthcare`
- moving `oya/application` wholesale

## Smallest honest birth slice (when unblocked)

Activation only after: independent design APPROVE + preferred #1526 observed green + #1523 promoted green (same admission chain as CI runtime birth).

**Slice P0 — meta root only (1 PR, zero crates):**

1. Create `app/` with OWNERS + charter pointing at ADR-0562 §3#5 / ADR-0615 §1 and `app_products_note`.
2. No crate moves. No registry absorb edits that orphan `oya/*`.
3. Prove membership lint accepts `app/` top-level.
4. Anti-vacuity: `git ls-tree` shows `app/` present; focus-family and crate counts unchanged.

**Slice P1 — first named product roots without moves (1 PR or serial tiny PRs):**

Birth empty roots for the **largest authority-named cones that already have crate mass**:

1. `app/office` (+ context placeholders listed in the note that have live crates: docs, sheets, slides, notes, sites, whiteboard, design-collaboration; translate as sibling or office context per owner)
2. `app/community`
3. `app/crm`, `app/hr`, `app/payroll` (ERP-ish cluster; still separate roots)

Still **zero crate moves**. Purpose: parking destinations + OWNERS so later move PRs are one-home.

**Slice P2 — first crate MOVE (separate plan JSON, later):**

Only after a chosen cone has:

1. non-colliding destination root live on tip,
2. **documented 2+ registered-capability composition** (Cargo path deps to ≥2 capability packages **or** explicit owner composition declaration recorded in the product README/ADR),
3. ArtifactMove + debrand rules,
4. no dual-home dark wiring,
5. independent APPROVE on that move plan.

Candidate order by mass × authority clarity: office cone → community → hr/payroll/crm. **Not** application shell.

## Office cone grouping rule

`app_products_note` writes `office/docs/sheets/…` as **one product with contexts**, not eight capabilities.

| Live top | Crates | Birth home |
|---|---:|---|
| `oya/office` | 19 | `app/office` (default context / shared kernels+apps) |
| `oya/docs` | 1 | `app/office` context `docs` (product docs — **not** repo `docs/` governance tree) |
| `oya/sheets` | 1 | `app/office` context `sheets` |
| `oya/slides` | 1 | `app/office` context `slides` |
| `oya/notes` | 1 | `app/office` context `notes` |
| `oya/sites` | 1 | `app/office` context `sites` |
| `oya/whiteboard` | 1 | `app/office` context `whiteboard` |
| `oya/design-collaboration` | 1 | `app/office` context `design-collaboration` |
| `oya/translate` | 1 | **owner pick**: office context vs standalone `app/translate` (note lists translate after office list; do not auto-fold) |

Calendar/forms: named in office list but **no cargo tops** today; forms engines absorbed by `workflow` (registry). Do not invent `app/office/forms` crates from substrate forms domains (ADR-0615 substrate/product split).

## Non-code shells (48) — still not this design

Healthcare shells, payments, social, connect, etc. remain G030 + owning-lane disposition. Product-root birth may later park shells under `app/<product>/` docs/contracts, but **absence of Cargo is not DELETE** (census binding).

## Sequencing relative to other G026 tracks

| Track | State | Blocks product moves? |
|---|---|---|
| CI runtime face-birth (12 crates) | design done; no move JSON | no — parallel capability track |
| Tools leaves | 17/17 missing | no tools/product coupling |
| Product root birth (this file) | design only | yes — roots must exist before moves |
| Application split | owner-blocked | yes for those 8 crates |
| G028 / #1526 / #1523 | admission chain RED/local | preferred before any accounting-facing or large move train |

## Explicit non-goals

- No `specs/reorg/*-move-plan.json` authored here.
- No claim that WEAK/UNPROVEN cones already satisfy ADR-0562 §6 2+ test for MOVE.
- No `app/application` root.
- No merge of social↔community or diagnostics↔healthcare.
- No tools or intelligence moves.
- No live cluster deploy.
- No independent APPROVE available (Agent decrypt 400; Codex usage limit; ouroboros 503 class remain fused).

## Independent review

Not obtained. Coordinator design evidence only — **not APPROVE**.

## Non-claims

- Not permission to push or merge birth/move PRs.
- Not a closed app roster (registry explicitly leaves roster to migration lanes).
- Not proof of final internal package layout under each product.
- Not a deletion plan for any `oya/*` tree.
