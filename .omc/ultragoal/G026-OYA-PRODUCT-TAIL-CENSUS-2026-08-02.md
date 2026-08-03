# G026 oya product/CI tail census — 2026-08-02

State: **PLANNING_ONLY_NOT_ACTIVATED**
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.
No code move. No shared-registry edit. No executable move-plan JSON in this file.

## Cardinality

| Class | Count | Notes |
|---|---:|---|
| `oya/` immediate children | **82** | dirs only |
| `oya/**/Cargo.toml` crate roots | **180** | |
| `oya/intelligence/**` crates | **78** | **G024 remainder scope — out of G026 move authorship** |
| non-intelligence `oya/**` crates | **102** | G026 product/CI tail + legacy product services |
| `oya/` dirs with **zero** Cargo.toml | **48** | non-code shells (docs/specs/contracts); not MOVE/DELETE targets from absence of Cargo alone |
| CI product services with crates | **3 dirs / 12 crates** | `ci-controller` 4, `ci-tide` 3, `ci-webhook-gateway` 5 |

`cloud/cloud-kernel` is **LIVE** on current `origin/dev` (**170** tracked files). Earlier ABSENT reading was a false negative; see `G023-CLOUD-KERNEL-LIVE-TREE-CENSUS-2026-08-02.md`. G023 remains the approved deletion after #1523 promotion.

## Partition (binding)

| Partition | Crates | Owning goal | G026 action |
|---|---:|---|---|
| `oya/intelligence/*` | 78 | **G024** | exclude from G026 move plans |
| `oya/ci-{controller,tide,webhook-gateway}/*` | 12 | G026 CI tail | disposition toward existing `ci/` / delivery-fabric faces; check `specs/reorg/ci-move-plan.json` before authoring a second plan |
| Product services with crates (office, community, …) | 90 | G026 product tail | APP_FACE_BIRTH_REQUIRED; `app/` absent on origin/dev |
| Non-code product/capability shells | 0 crates / 48 dirs | G026 hygiene + G030 | consumer/capability/app disposition; not crate moves |

## Non-intelligence crate buckets (exact)

| Bucket | Crates |
|---|---:|
| office | 19 |
| community | 14 |
| application | 8 |
| itsm | 6 |
| ci-webhook-gateway | 5 |
| hr | 5 |
| payroll | 5 |
| ci-controller | 4 |
| ci-tide | 3 |
| crm | 3 |
| plant-maintenance | 2 |
| production-planning | 2 |
| quality-management | 2 |
| real-estate | 2 |
| supply-chain-planning | 2 |
| treasury | 2 |
| warehouse | 2 |
| contract-lifecycle-management | 1 |
| design-collaboration | 1 |
| docs | 1 |
| financial-planning | 1 |
| global-trade | 1 |
| incident-management | 1 |
| learning-management | 1 |
| marketing-automation | 1 |
| notes | 1 |
| performance-management | 1 |
| sheets | 1 |
| sites | 1 |
| slides | 1 |
| translate | 1 |
| whiteboard | 1 |
| workplace-integration | 1 |
| **non-intel total** | **102** |

## Non-code `oya/` shells (no Cargo.toml) — 48

accounting, analytics, api-gateway, audit-chain, calendar, comms-email, compliance, connector, consent-graph, contact-center, data-pipeline, data-warehouse, detection, developer-sdk, diagnostics, drive, emergency, emr, feature-flags, finops-portal, forms, governance, healthcare-integration, identity, imaging, mail, marketplace, meet, messenger, observability, ontology, ops-dashboard-control-center, oya-authn-device-firmware, oya-billing, oya-cost, oya-flags, oya-identity, oya-meter, patient-monitoring, payments, pharmacy, plugin-app-store, recordings, social, tasks, tenant-rbac, workflow-engine, workflow-studio

These are **not empty** and **not** automatic DELETE. Immutable-tree census shows substantial non-code content (Markdown/YAML/JSON/Cedar/protobuf/Terraform). Many are already listed in capability `absorbs_current_dirs`; healthcare/social/payments shells belong to future `app/` composition. Disposition belongs to G030 consumer classification plus the owning capability/app lane, not a G026 crate move plan.

## CI product tail (12 crates) — disposition class (updated)

Capability ownership is proven; exact leaf destinations are not. See `G026-CI-PRODUCT-DESTINATION-FACE-PROOF-2026-08-02.md`.

| Service | Crates | Class | Notes |
|---|---:|---|---|
| `oya/ci-controller` | 4 (app, github-adapter, k8s-adapter, kernel) | **FACE_BIRTH_REQUIRED under capability `ci`** | Registry absorbs into `ci`. Live `ci/core|adapters|facade` leaves for controller are absent. |
| `oya/ci-tide` | 3 (app, github-adapter, kernel) | **FACE_BIRTH_REQUIRED under capability `ci`** | Same owner; no Tide leaf under live `ci/`. |
| `oya/ci-webhook-gateway` | 5 (app, authz-cedar, ed25519, github, kernel) | **FACE_BIRTH_REQUIRED under capability `ci`** | Same owner; no webhook-gateway leaf under live `ci/`. |

`specs/reorg/ci-move-plan.json` remains **0-overlap** (46 gate moves only). Do not extend that plan for runtime services. Do not author a CI runtime move plan until face birth maps all twelve packages.

## Product services with crates (90) — disposition policy (not per-row plan)

Default for every non-CI product bucket:

1. **APP_FACE_BIRTH_REQUIRED** — `app/` is not live on `origin/dev`; 29/30 product names have neither `app/<product>` nor a top-level capability directory. Lexical `docs/` is repository governance, not a proven product destination for `oya/docs`.
2. **KEEP_IN_OYA** until an `app/<product>/` composition face is born and independently reviewed.
3. **MOVE** only when the app face exists (or is born in the same plan), catalog ArtifactMove + debrand rules hold, and dual-home dark wiring is impossible.
4. **DELETE_CANDIDATE** only with zero live importers + registry absence + explicit supersession — never from a sibling shell's lack of Cargo manifests.
5. Office (19) and community (14) remain the largest bodies; they need app-face birth first. Do not fold them into a tools plan.

Per-row executable plans remain **out of scope**. Next product-tail slice = one app-face birth design, not a 90-row mega-plan.

## Existing reorg plans that constrain G026

Present under `specs/reorg/` on origin/dev (from prior census; re-list before authoring):

- `ci-move-plan.json` — **must be read before any oya/ci-* plan**
- `intelligence-*.json` — G024
- `governance-check-move-plan.json` — may absorb tools governance *kernels* (libs), not tools apps until faces exist
- `kernel-move-plan.BLOCKED.json` — materializer blocked

## Sequencing (G026 only)

1. Tools: blocked on **born leaf faces** (see destination-existence audit). No tools move-plan JSON.
2. oya CI tail: capability owner proven as `ci`; leaf destinations absent → **face-birth design first**, then one non-overlapping runtime plan. Do not extend the gate-only `ci-move-plan.json`.
3. oya product tail: **app-face birth first**; no crate move plan while `app/` is absent.
4. Non-code shells: G030 + owning-capability/app lanes; not crate moves and not automatic DELETE.
5. Keep `tools/oya-reorg-codemod-app` stationary until tools tail empties.
6. No activation until independent review APPROVE + protected CI + #1526 corpus repair observed green (tools/product moves still behind reorg executor health).

## Non-claims

- Not a move plan. No JSON under `specs/reorg/` authored here.
- 102 non-intel crates are **not** 102 MOVE rows.
- 48 non-code shells are **not** 48 DELETE rows.
- G024 owns intelligence 78; G026 must not double-plan them.

## Follow-on

Product root birth design (no moves): `G026-PRODUCT-APP-FACE-BIRTH-DESIGN-2026-08-02.md`.
