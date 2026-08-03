# G026 application unresolved-owner packet — 2026-08-02

State: **PLANNING_ONLY — TWO OWNER RULINGS BLOCK MOVE JSON**  
Authority: `origin/dev` at `0c1014b87f0d881a821faa6a872b309deba0cfbf` (#1529 merged; ARC request declared `22Gi`, live request still `20Gi`).  
Supplements `G026-APPLICATION-CONE-OWNER-SPLIT-2026-08-02.md`. No destination or move-plan row is proposed.

## Exact unresolved set

The `oya/application` Rust cone contains eight crates. Six have bounded capability classes pending collision/importer review. Exactly two still lack the identity needed to name a destination:

| Source crate | What tip proves | What tip does not prove |
|---|---|---|
| `oya/application/crates/oya-application-app` | Rust package exists; composes many capabilities; structurally qualifies for `app/<product>` only if it is one deployable tenant product | product identity, deployable boundary, whether it should split, exact `app/<product>` leaf |
| `oya/application/crates/oya-cloud-surface-domain` | Rust package exists; models cloud product-surface/SKU/fulfilment invariants | single registered-capability ownership vs named multi-capability tenant product vs decomposition/retirement |

Tip also proves:

- `app/` is absent;
- registered capabilities include `console`, `marketplace`, `comms`, `storage`, and `workflow`;
- no registered capability named `application` exists;
- `app/application` and `app/workspace` are absent and unauthorized;
- all eight source packages still exist under `oya/application`.

## Owner question A — `oya-application-app`

Accountable product owner must choose exactly one:

1. **NAME_PRODUCT** — identify the existing tenant product this composition deploys; provide product authority, deployment boundary, composed capability list, and non-colliding `app/<product>` root.
2. **SPLIT** — if this is a test/foundation aggregation rather than one product, partition it by actual product/use-case; no generic shared-app root.
3. **KEEP_PENDING** — if deployable/product identity is not yet established; source remains until evidence exists.

Forbidden answers: `app/application`, `app/foundation`, or any name inferred only from crate suffix/dependency fan-in.

## Owner question B — `oya-cloud-surface-domain`

Accountable cloud-product/capability owner must choose exactly one:

1. **SINGLE_CAPABILITY** — name the closed-registry capability and prove whether this is `core` or sold `facade` from runtime/API consumers.
2. **NAMED_PRODUCT** — prove it is a deployable composition of 2+ capabilities and name the existing tenant product under `app/<product>`.
3. **DECOMPOSE_OR_RETIRE** — if it is legacy doctrine/test material rather than a cohesive runtime domain, name split/delete authority and successor contracts.
4. **KEEP_PENDING** — if none is proven.

Forbidden answer: place by the word `cloud`, by one current helper dependency, or by guessing a new capability.

## Required owner response format

```text
source_crate | decision | accountable_owner | authority | deployable_or_capability_proof | destination_class | exact_leaf_or_pending | acceptance_check
```

`exact_leaf_or_pending` must remain `pending` unless fresh immutable-tip collision and importer proofs succeed after owner identity is settled.

## Six already-bounded siblings (not part of this owner packet)

- shell frontend → `console` capability class;
- SaaS plugin app → `marketplace` capability class, core-vs-facade pending;
- chat/meet APIs → `comms/facade` class;
- drive API → `storage/facade` class;
- forms API → `workflow/facade` class.

These may become separate serial moves only after independent review and fresh collision/importer proof. They must not wait for one eight-crate mega-plan, and they must not be grouped under `app/workspace`.

## Admission order

1. Independent design APPROVE on owner split and destination-face designs.
2. Owner answers for these two crates.
3. Fresh `git ls-tree <immutable-ref> -- <destination>` collision probes.
4. Importer/dependency proof and non-code ownership accounting.
5. Separate capability/product move plans, codemod-executed one serial move at a time.
6. Buck2-authoritative graph verification and protected PR admission.

The live PR train remains ahead: G028 APPROVE/admit/GitOps observe, then #1526 cold FULL, then #1523 restack.

## Non-actions

- No move-plan JSON.
- No `app/` birth.
- No `app/application`, `app/foundation`, or `app/workspace`.
- No exact destination leaf invented.
- No source move/delete or non-code shell deletion.
- No generated JSON edit.
- No independent APPROVE inferred from failed transport.
- No cluster or canonical dirty-checkout mutation.
