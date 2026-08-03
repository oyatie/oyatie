# G026 application cone owner-split brief — 2026-08-02

State: **PLANNING_ONLY — OWNER SPLIT PROVEN; NO `app/application`; NO MOVE PLAN**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements `G026-PRODUCT-APP-FACE-BIRTH-DESIGN-2026-08-02.md`.  
No path, package, workspace, registry, generated face, GitOps declaration, or cluster state was changed.

## Decision

`oya/application` is not one bounded context and is not one tenant product. Its eight Rust crates split into four ownership classes:

1. console shell substrate;
2. single-capability facades/engines already owned by marketplace, comms, storage, and workflow;
3. a true many-capability composition root whose product identity is unresolved;
4. a cloud product-surface domain whose owning capability/product boundary is unresolved.

Therefore the reorg must **not** birth `app/application`, and must not move the directory atomically as one product. The next executable artifact cannot be an eight-row move plan. It must wait for the two unresolved owner rulings and independently reviewed destination-face decisions.

## Binding placement rules

ADR-0562 §3 applies in this order:

- deployable surface composing **2+ registered capabilities** for a tenant → `app/<product>/`;
- otherwise exactly one registered capability, selected by what the system is; engine → `core`, traits → `ports`, transient implementation → `adapters`, sold surface/composition root → `facade`.

ADR-0562 §6 makes a single-capability `app/` member a mis-placed facade.

ADR-0615 Q12 is more specific than the registry's generic “web shell” example: the **console shell itself is the `console` substrate capability**; only ops-dashboard leaves composing 2+ capabilities become `app/ops-console/<vertical>`.

The closed capability registry contains no `application` capability and no capability absorbs `oya/application`. Its `app_products_note` enumerates named tenant products; it does not authorize a generic `app/application` bucket.

## Eight-crate split

| Source crate | Measured role / deps | Ownership class | Destination ruling now |
|---|---|---|---|
| `oya-application-shell-frontend` | Production Leptos portal shell; shell registry, token broker, design system, nav/control-center surface | console shell substrate | **`console` capability confirmed**; exact face/leaf requires collision + importer review, likely facade composition-root shape; **not app/** |
| `oya-saas-plugin-app` | plugin invocation/runtime contract; only capability dep is `marketplace-plugin-kernel` | single-capability marketplace system | **`marketplace` capability confirmed**; core-vs-facade face unresolved by name alone; **not app/** |
| `oya-workspace-chat-api` | REST/API boundary around `comms-messenger-domain`; data-boundary helper | single-capability sold comms surface | **`comms/facade` class**; exact non-colliding leaf pending |
| `oya-workspace-meet-api` | REST/API boundary around `comms-meet-domain`; data-boundary helper | single-capability sold comms surface | **`comms/facade` class**; exact non-colliding leaf pending |
| `oya-workspace-drive-api` | HTTP boundary around `storage-drive-domain`; data-boundary helper | single-capability sold storage surface | **`storage/facade` class**; exact non-colliding leaf pending |
| `oya-workspace-forms-api` | REST boundary around `workflow-forms-domain`; data-boundary helper | single-capability sold workflow surface | **`workflow/facade` class**; exact non-colliding leaf pending |
| `oya-application-app` | “Foundation application slice”; directly composes intelligence, audit, cell, IAM, network, data, observability, messaging, secrets, tenancy, governance/cost, and boundary kernels | real 2+ capability composition | **`app/<owner-named-product>` required**, but product name unresolved; do not invent `app/foundation` or `app/application` |
| `oya-cloud-surface-domain` | cloud product-surface invariants and SKU/fulfilment phase model; only current path dep is data-boundary helper | boundary unresolved | **OWNER RULING REQUIRED**: prove a single registered capability (then core/facade), or prove a named 2+ capability tenant product (then app); dependency count alone is insufficient |

## Why the four workspace APIs are not an `app/workspace` product

Each API currently wraps one domain capability:

- Chat and Meet → `comms`;
- Drive → `storage`;
- Forms → `workflow`.

Their common dependency on `oya-data-boundary-kernel` supplies classification parsing and boundary primitives; it does not by itself establish a second product capability. The registry's boundary notes explicitly preserve single-capability Drive/Recordings and comms sold APIs as capability facades. A future end-user collaboration suite wiring comms + storage + IAM + billing could be `app/<product>`, but that composition is not what these four crates implement today.

Grouping the four under `app/workspace` now would violate the 2+ test and create the exact suite-wrapper shape ADR-0562 rejects.

## Why shell is not the many-capability product root

The shell renders module visibility and brokers tokens, but ADR-0615 Q12 classified the shell, design system, nav, and token broker as the `console` substrate. It is not reclassified by the generic `app/` meta-directory note. Product views composed inside the shell may belong to named `app/ops-console/<vertical>` members; the shell implementation remains `console`.

## Why `oya-application-app` cannot be named mechanically

The crate clearly passes the structural 2+ test, but its dependency fan-in is not a product name. “Foundation” describes a program/slice and “application” is a generic layer noun. The app roster is owner-enumerated by migration lanes, and `app/<product>` must state what tenant product ships.

Required owner ruling:

1. identify the one tenant product this composition delivers; or
2. split the crate by product/use-case if it is a test aggregation or shared foundation rather than one deployable product; or
3. identify an existing named product root that owns it.

Until then, KEEP at source. Do not create `app/foundation`, `app/application`, or a generic shared-app root.

## Why `oya-cloud-surface-domain` remains unresolved

Its source says it owns a cloud customer product-surface contract spanning fulfillment phases and compute SKU types. That semantic role could be:

- a single capability's domain/facade;
- a named cloud product composition spanning multiple capabilities; or
- a legacy doctrine/test kernel requiring decomposition.

The current Cargo edge only to the data-boundary helper proves none of those. Placement must follow WHAT IT IS plus owner/deployable evidence, not the `cloud` name or a guessed dependency graph.

## Safe sequencing

### A. Independently review this split before any move JSON

Review must confirm:

- the Q12 shell ruling;
- the single-capability nature of the four workspace APIs;
- marketplace ownership and core-vs-facade face for the plugin runtime;
- owner/product ruling for `oya-application-app`;
- owner/capability ruling for `oya-cloud-surface-domain`.

Transport failure is not APPROVE.

### B. Capability-owned leaves may become separate serial moves

Only after approval and fresh immutable-tip collision probes:

1. workspace APIs can move by owning capability, not as an application batch;
2. shell can move with the console lane;
3. plugin runtime can move with the marketplace lane after face classification.

Each move needs codemod execution, Buck2-authoritative graph verification, path/package/importer rewrites, catalog/membership producer regeneration, and same-commit non-code/deploy ownership. No hand move and no hand-edited `*.generated.json`.

### C. Composition roots wait for named ownership

`oya-application-app` and `oya-cloud-surface-domain` remain KEEP_PENDING_OWNER. They cannot block clean single-capability moves, but no source directory is deleted while either remains.

## Non-code shell

`oya/application` also contains contracts, Cedar policy, capability YAML, catalog rows, dashboards, IaC, SLOs, runbooks, scorecards, a manifest, and historical docs. These are not automatically owned by whichever crate moves first. Each artifact must co-move with its semantic owner or remain until the last owner split closes. Deleting the top directory is a final accounting result, not a first step.

## Next smallest slices

1. Obtain owner ruling for the two unresolved crates; do not ask for one ruling on all eight.
2. Run exact destination collision/importer proofs for the four workspace API facade candidates.
3. Run console importer/collision proof for the shell under the already-landed console face structure.
4. Classify marketplace plugin runtime core-vs-facade from deployable/API consumers, not its `-app` suffix.
5. Only then author separate capability-lane move plans; never an `application` mega-plan.

## Non-actions and non-claims

- No `app/application` or `app/foundation` birth.
- No claim that a common data-boundary helper creates a multi-capability app.
- No exact destination leaf names asserted before collision/importer proof.
- No move-plan JSON.
- No deletion of non-code content.
- No independent APPROVE; the latest G028 review transport also failed and remains non-approval.
