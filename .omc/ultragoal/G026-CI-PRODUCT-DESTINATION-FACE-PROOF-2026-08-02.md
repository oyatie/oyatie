# G026 CI/product-tail destination-face proof — 2026-08-02

State: **PLANNING_ONLY — CI CAPABILITY OWNERSHIP PROVEN, LEAF DESTINATIONS NOT BORN**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements `G026-OYA-PRODUCT-TAIL-CENSUS-2026-08-02.md`; no move-plan JSON, source move, registry edit, push, or activation occurred.

## Result

The capability registry proves that all three live Oya CI services belong to capability `ci`, but the live `ci/` tree does **not** yet contain their destination leaves. This closes the ownership question without inventing paths.

| Source service | Crates | Registry owner | Exact live destination leaf | Disposition |
|---|---:|---|---|---|
| `oya/ci-controller` | 4 | `ci` | none | **FACE_BIRTH_REQUIRED** |
| `oya/ci-tide` | 3 | `ci` | none | **FACE_BIRTH_REQUIRED** |
| `oya/ci-webhook-gateway` | 5 | `ci` | none | **FACE_BIRTH_REQUIRED** |

`specs/capability-registry.json` capability `ci` explicitly absorbs `ci`, `oya/ci-controller`, `oya/ci-tide`, and `oya/ci-webhook-gateway`. Its charter names the delivery fabric, controller, Tide merge queue, and webhook gateway. That is placement authority, not evidence that a specific leaf path already exists.

## Live destination-face census

Current `ci/` contains **376 tracked files**:

- `ci/ports/path-resolver` — one Cargo/BUCK package
- `ci/adapters/path-resolver` — one Cargo/BUCK package
- `ci/facade/*` — the gate/productized-check population
- no live `ci/core/*` package
- no controller, Tide, or webhook-gateway leaf under `ci/core`, `ci/adapters`, or `ci/facade`

Exact probes on immutable `origin/dev` are absent for:

- `ci/core/{controller,tide,webhook-gateway}`
- `ci/adapters/{controller-github,controller-k8s,tide-github,webhook-gateway-github}`
- `ci/facade/{controller,tide,webhook-gateway}`

Therefore the next operation is not a path-only MOVE plan. A reviewed face-birth design must first map the twelve source packages to the closed `core|ports|adapters|facade` grammar, including the three app packages and the webhook gateway's three distinct adapters.

## Existing CI move plan is disjoint

`specs/reorg/ci-move-plan.json` has **46** `moves[]` rows. Its own comment says it moves the cloud-CI gate keystone into productized `ci/facade` names. Every move is `cloud/cloud-ci/gates/*` → `ci/facade/*`; it contains no occurrence of:

- `oya/ci-controller`
- `oya/ci-tide`
- `oya/ci-webhook-gateway`
- `ci/core`
- `ci/adapters`

The plan's first row is the accounting-registry gate and its last row is topology-manifest-contract; neither class is one of the three runtime CI services. Overlap with the twelve G026 CI crates is **0**.

Consequences:

1. Do not extend the executed gate plan as though it had already named runtime-service leaves.
2. Do not author a second plan until face birth gives every source crate a non-colliding destination.
3. Once leaves are born, one CI-runtime plan may cover the twelve crates because the registry gives them one capability owner; the plan still needs per-crate importer, Cargo/BUCK, service-name, deployment, and affected-set rewrites.

## Product composition tail

The remaining non-intelligence, non-CI product population is **90 crates** across 30 product directories. For 29 of 30 product names, neither `app/<product>` nor an exact top-level capability directory exists on `origin/dev`. The sole lexical top-level match is `oya/docs` → `docs`, but `docs/` is repository governance/documentation, not proven as the product composition destination. Lexical equality is not placement proof.

The capability registry's `app_products_note` explicitly classifies CRM, HR, ITSM, payroll, treasury, office/docs/sheets/slides/notes/sites/whiteboard, and the other product verticals as `app/<product>/` composition members rather than capabilities. Yet `app/` is not live on `origin/dev`. Therefore all 90 crates remain **APP_FACE_BIRTH_REQUIRED**; no per-service executable move plan is truthful yet.

## Non-code Oya shells

An immutable-tree census finds **48 immediate `oya/` directories with no Cargo manifest below them** (excluding the root file `oya/BUCK`). They are not empty: most contain substantial Markdown, YAML, JSON, Cedar, protobuf, Terraform, or other non-code artifacts. Several are already listed in a capability's `absorbs_current_dirs`; several healthcare/social/payment product shells belong to future `app/` composition. Their lack of Cargo manifests does not authorize deletion.

This corrects the ambiguous phrase “empty dirs” to **non-code product/capability shells**. Their disposition belongs to G030 corpus-consumer classification plus the owning capability/app migration lane, not a G026 crate move plan.

## Smallest safe next slice

After the admission chain is healthy and an independent design review is available:

1. author one **CI runtime face-birth design**, not a move plan, for the twelve crates;
2. prove the mapping against closed face grammar and all live importers/deployments;
3. only then author one executable CI runtime move plan;
4. separately birth `app/<product>` before any of the 90 product crates move.

Until then, G026 is honestly blocked at destination birth. No executable G026 move-plan JSON is warranted.

## Follow-on

Face-birth design landed plan-only: `G026-CI-RUNTIME-FACE-BIRTH-DESIGN-2026-08-02.md` (12-row map, 0 collisions, no move-plan JSON).
