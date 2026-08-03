# G036 exact 48-kernel protected-context gap — 2026-08-02

State: **PLANNING EVIDENCE ONLY — EXACT GAP SET; NO ACTIVATION/BASELINE/POLICY EDIT**  
Authority: `origin/dev` at `0c1014b87f0d881a821faa6a872b309deba0cfbf` (#1529 merged; ARC request declared `22Gi`, live request still `20Gi`).  
Supplements `G036-PROTECTED-GRAPH-CENSUS-2026-08-02.md` and `G036-MULTI-ROOT-SELF-CONFORMANCE-DESIGN-2026-08-02.md`.

## Exact set arithmetic

At immutable tip:

- 56 immediate `governance/check/*` kernel directories;
- 56/56 have a `BUCK` package;
- 56/56 declare at least one `rust_test` target;
- 8 distinct kernels are selected by `ci/facade/affected-target-set/affected-set-policy.json`;
- 48 are the exact set difference.

```text
56 total − 8 policy-selected = 48 protected-context-gap candidates
```

The eight selected kernels are:

```text
active-artifact-contract
codeowners-mirror
data-class
doc-catalog
image-signing-discipline
pr-traceability
raci-coverage
slsa-l3-evidence-grounded
```

## Exact 48-kernel gap set

```text
a11y-discipline
adr-citation
aspirational-enforcement
audit-chain-seal-coverage
authority-cohesion
authz-tier-discipline
benchmark
cedar-fragment-coverage
client-stack-discipline
cohesion
cursor-pagination-coverage
documentation-system
event-schema-versioning
glossary-coverage
glossary-vocabulary
high-risk-auto-decision-refusal
honest-claims
iac-tier-discipline
id-discipline
idempotency-key-coverage
layered-architecture-discipline
metric-cardinality
mobile-native
no-grouping
olap-tier-discipline
ontology-projection-coverage
openapi-rest-route-parity
otel-trace-propagation
perf-budget
placeholder-debt
pre-push
protection-context-match
quality-lane
readme-coverage
release-pack
retired-vocabulary
rpo-rto-coverage
runbook-freshness
runbook-index
shardability
statelessness
supply-chain
tenant-cost-labels-coverage
typescript-workspace
vector-store-discipline
vendor-lockin-discipline
vendor-recency
wasm-runtime-discipline
```

## Measured reachability boundary

Proven:

1. `.github/workflows/oya-ci-required.yml` has no executable `//governance/check/...` label or recursive `//governance/check/...` pattern; its only direct prose reference names retained-local `pr-traceability`.
2. `marketplace/facade/dev-cli/BUCK` names all 56 kernels, but that bridge is retirement-marked local feedback and not merge authority.
3. affected-set policy names the eight selected kernels above.
4. `ci/facade/gate-self-conformance/gate-self-conformance-policy.json` sets a single `gates_root` of `ci/facade`.
5. baseline-ratchet's outside-fleet test uses `fitness-*` catalog facets as a shrink-only placement guard; it does not register or execute the 48 governance tests.
6. all 48 have real Rust tests, so this is an admission-reachability gap, not missing test targets.

Not yet proven:

- Buck2 query reachability from some required constituent target through an indirect dependency into any of the 48 tests. Static source/policy evidence finds no such route, but Buck graph proof remains required before implementation.
- that all 48 should block every change; an affected-set owner may select contract-specific slices rather than a universal fan-in.
- that test existence implies semantic quality or non-vacuity.

Therefore call them **protected-context-gap candidates**, not definitely unexecuted at runtime, until Buck2 reachability evidence is captured on a healthy runner.

## Minimum activation design

Do not add 48 workflow commands and do not baseline findings. The bounded product change is:

1. Extend gate-self-conformance from one `gates_root` to policy-selected multiple roots, preserving `ci/facade` behavior.
2. Give each root a target naming contract rather than assuming `ci-{name}-gate`; governance packages use `check-{name}-unittest` with measured exceptions such as `*-discipline` target suffixes.
3. Add a protected registration meta-test that derives all `rust_test` targets under each selected root and proves fan-in reachability.
4. Keep affected-set selection as the workload-sizing layer; self-conformance checks registration completeness, not “run all 48 on every diff.”
5. RED controls must prove: a synthetic orphan governance test is rejected; a registered target passes; a source-only bridge label does not count; an empty discovered set fails.
6. No known-bad baseline for the 48; activation must first classify failures and repair or owner-retire them.

## Tip recompute (2026-08-02 post-#1529)

Recomputed against immutable tip `0c1014b87` via `git ls-tree` of `governance/check/` children + BUCK/`rust_test` presence + path tokens in `ci/facade/affected-target-set/affected-set-policy.json`:

```text
TOTAL_DIRS=56 HAVE_BUCK=56 NO_RUST_TEST=0 SELECTED=8 GAP=48
EIGHT_MATCH=true GAP_MATCH=true
```

The eight selected kernels and exact 48-gap list above are unchanged from the pre-#1529 packet. Arithmetic is tip-current; activation is not.

## Dependency and admission order

- Design remains blocked on independent review and healthy protected CI.
- G028 #1529 MERGED (`0c1014b87`, declared 22Gi). Live ARS/ERS still 20Gi — GitOps reconciler inert; prior founder ruling fixes class B / KEEP_CURRENT_LAB=true and independent design APPROVE remains pending (`OWNER-DECISION-G028-ABC-LIVE-22GI-2026-08-02.md`, `G028-CLASS-B-PERMANENT-LAB-GITOPS-DESIGN-2026-08-02.md`). No live apply authorized from this packet.
- Promoted-tip CI run `30767156146` FAILED FULL on `//oya:corpus-yaml-facts` with no-exit-code; live 22Gi not observed; 22Gi serialization necessary≠sufficient.
- #1526 corpus repair then #1523 restack remain ahead in the train and blocked until live 22Gi via an admitted reconciler.
- Do not combine G036 with G037 registry-lane repair, G030 cleanup, G023 deletion, or G026 reorg moves.

## Non-actions

- No workflow, affected-set policy, self-conformance policy, BUCK file, baseline, or generated face edited.
- No claim of protected green or independent approval.
- No 48-row activation PR opened.
- No cluster mutation; live ARC remains 20Gi; no helm/CRS/render.sh apply.
