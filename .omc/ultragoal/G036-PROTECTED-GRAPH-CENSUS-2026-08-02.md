# G036 governance-check protected-graph census — 2026-08-02

State: **PLANNING EVIDENCE ONLY — NOT ACTIVATED, NOT REVIEWED, NOT ADMITTED**

Source anchor: freshly fetched `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.

## Non-vacuous census

| Class | Count | Evidence |
|---|---:|---|
| Immediate `governance/check/*` kernel directories | 56 | `git ls-tree -d --name-only origin/dev:governance/check` |
| Kernels with a `BUCK` package | 56 | `git cat-file -e origin/dev:governance/check/<kernel>/BUCK` for every row |
| Explicitly selected by protected affected-set policy | 8 | `ci/facade/affected-target-set/affected-set-policy.json` synthetic dependencies |
| Exposed by retired `marketplace/facade/dev-cli` bridge | 56 | distinct `//governance/check/<kernel>:` labels in `marketplace/facade/dev-cli/BUCK` |
| Bridge-only remainder | 48 | set difference: bridge labels minus protected-policy selections |
| No-BUCK/dark directory | 0 | every immediate kernel has `BUCK` |

The eight policy-selected kernels are:

1. `active-artifact-contract`
2. `codeowners-mirror`
3. `data-class`
4. `doc-catalog`
5. `image-signing-discipline`
6. `pr-traceability`
7. `raci-coverage`
8. `slsa-l3-evidence-grounded`

All 56 kernels are named by the retired bridge, so the remaining 48 are not source-dark. They are **protected-context-dark unless another mechanically proven route exists**. This census did not find one: the required workflow contains no explicit `//governance/check/...` label and no recursive `//governance/check/...` pattern. Its direct workflow prose reference is `governance/check/pr-traceability`, explicitly described as retained locally while PR-body admission is retired.

## `gates_root` limitation

`ci/facade/gate-self-conformance/gate-self-conformance-policy.json` sets:

```json
{"scan":{"gates_root":"ci/facade"}}
```

The self-conformance implementation enumerates only the configured root and considers workflow registration relative to that root. Therefore it cannot prove registration for `governance/check/*`. The required workflow does execute recursive `//ci/facade/...` coverage, but not recursive `//governance/check/...` coverage.

Changing only `gates_root` from `ci/facade` to `governance/check` would merely move the blind spot. G036 needs either:

- a multi-root self-conformance contract that proves every retained gate reaches the single protected context, with born-blocking fixtures; or
- retirement of kernels that have no required-context consumer after individual semantic/authority review.

## Safety ruling

Do not infer that all 48 bridge-only kernels should be deleted. The bridge proves that implementations and consumers once existed; each kernel still requires semantic authority and non-bridge consumer review. Equally, do not classify them as binding merely because they build or because the bridge aggregates them. Build reachability is not protected-context reachability.

## Independent-review status

The read-only architecture review lane terminated on transport/decryption failure before a verdict. No approval is inferred.

## Smallest safe next slice

Implement a **multi-root, born-blocking self-conformance proof** before wiring or retiring individual kernels. The fixture must demonstrate a failing retained `governance/check` kernel that has BUCK targets and bridge exposure but no protected-workflow route, then turn green only when a real protected route is present. Do not exempt or relabel the 48 rows to make the finding disappear.
