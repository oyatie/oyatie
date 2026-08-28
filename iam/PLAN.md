---
doc_class: Owner-PLAN
owner: iam
status: Active
date: 2026-08-27
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
  - iam/ADR.md
  - iam/PRD.md
  - iam/SPEC.md
---

# IAM remaining work

<baseline>

## Verified baseline

This plan was established against
`8489b29bce609b8ee3a3e5874f1d3013672d20c9`.

Retain and mature the canonical identity domain/API, OIDC/JWKS, SCIM,
workload-identity-consumption, and device-attestation behavior. Policy owns
Cedar/ReBAC evaluation and in-process `Check`; Secrets owns SVID, certificate,
key, and secret issuance.

The tenant-rbac removal inventory is exactly 39 crate directories and four
hand-authored OpenSLO files. Full deletion is NON-DISPATCHABLE until a founder-
accepted ADR-0710 amendment corrects its false concrete live-instance claim
while preserving in-process fail-closed VAP/CEL plus RBAC and PSA. No partial
37-crate state or preparatory salvage is authorized.

The HR owner-law correction at
`4203d2f794bea0abdf3ef2f1032a2729819f6c6f` replaces the former split-first,
HR-only salvage sequence with the same atomic IAM-owner prerequisite. That
commit is review evidence, not authority on this branch: the protected PR that
carries it must be merged into the deletion lane's `origin/dev` base before
dispatch. This IAM plan does not authorize an edit to `app/hr/`.

</baseline>

<delivery_lanes>

## Semantic delivery lanes

| Lane | Class and write set | Entry gate | Success | Failure |
|---|---|---|---|---|
| `publish-iam-owner-law` | Documentation only: `iam/{ADR,PRD,SPEC,PLAN}.md` | Exact reviewed base | Four files agree on ownership, gate, inventory, objectives, evidence, and future-feature separation | Runtime edits, root-law edits, claims that blocked work has landed, or missing five-field law |
| `remove-obsolete-iam-buck-census-loader` | Separate systemic structural cleanup: `iam/BUCK` only, plus no tenant-rbac path or lockfile | D-17 owner/build review | Deleted governance load and census targets are absent; target discovery reaches package BUCK files | Combining this path with tenant-rbac deletion or inventing a replacement census |
| `correct-kubernetes-admission-instance-record` | Founder-owned authority amendment: `docs/decisions/ADR-0710-kubernetes-admission-substrate-is-the-api-server.md` | Founder interview and acceptance | False IAM “live instance” citation is corrected while VAP/CEL, RBAC, PSA, fail-closed settings, and no-remote-PDP invariant remain explicit | Weakening admission, moving it to IAM/Policy RPC, or treating an unaccepted draft as authority |
| `transfer-authorization-engine-to-policy` | Future cross-owner structural migration; exact IAM and Policy paths plus mechanical lock changes are fixed by a later joint owner plan, and no tenant-rbac path may enter | A live `policy/` owner with all four law files; current PDP/Cedar inventory and reverse closure; one serialized lock writer | Cedar/ReBAC evaluation, snapshots, tuples, and `Check` are Policy-owned while IAM emits only principal and compiled-role input | Empty Policy scaffold, retained IAM evaluation, behavior change hidden in a move, tenant-rbac salvage, or unbounded write set |
| `transfer-identity-issuance-to-secrets` | Future cross-owner structural migration; exact IAM and Secrets paths plus mechanical lock changes are fixed by a later joint owner plan, and no tenant-rbac path may enter | Complete Secrets owner law; current SVID/certificate/key-issuance inventory and reverse closure; one serialized lock writer | Secrets owns issuance and key custody while IAM consumes only verification/workload-identity material through ports | IAM retains issuance authority, consumer behavior is deleted, tenant-rbac salvage, mixed semantics and structure, or unbounded write set |
| `mature-retained-identity-contracts` | Behavior-only per-leaf work in existing identity domain/API, OIDC/JWKS, SCIM, workload-consumption, and device-attestation packages; each dispatch names exact `src/` and `tests/` files | One existing leaf package, target-scoped Cargo/Buck closure, named objective and failure injection | Retained identity behavior meets tenant binding, durability, deterministic compilation, and PRD objectives without Policy/Secrets/app ownership | Package/lock mutation, new facade/proto hidden in behavior, vertical-app composition, authorization evaluation, or unsupported maturity claim |
| `remove-tenant-vertical-cone` | One serialized structural lane: exact inventory below plus mechanical `Cargo.lock` only | Founder amendment accepted; corrected HR owner law merged into the base; complete set reverified; separate IAM Buck cleanup landed | Every named path and dependency is absent; surviving IAM targets and protected workspace Cargo pass | Any subset, move, salvage, fake main, wrapper, REST compatibility, manifest split, unrelated lock churn, or failed graph/test evidence |
| `build-kubernetes-caller-admission` | Future k8s-owner feature under owner-approved `k8s/{core,ports,adapters,facade}` paths | Corrected ADR-0710 plus k8s owner law and design | Real API-server VAP/CEL/RBAC+PSA behavior and failure injection exist without a remote PDP | IAM-owned admission, review-only manifest, fail-open settings, or remote relationship lookup |
| `build-iam-connect-identity-facade` | Future IAM new feature; exact proto/facade paths require a later owner decision | Accepted protobuf/Connect generator and runtime contract plus a write set disjoint from retirement | Generated one-wire identity service has a real main/listener and measured objectives | Rename wrapper, handwritten Connect framing, REST/gRPC/transcode, or tenant-rbac dependency |

Operational lane names remain semantic. ADR identifiers appear only as
authority provenance and never as job, test, error, or lane labels.

</delivery_lanes>

<deletion_inventory>

## Exact post-amendment deletion set

### Core: seven crate directories

```text
iam/core/tenant-rbac-audit-chain-emission
iam/core/tenant-rbac-auth-app
iam/core/tenant-rbac-deployment-manifest
iam/core/tenant-rbac-domain
iam/core/tenant-rbac-tenant-admission-policy
iam/core/tenant-rbac-tenant-workload-manifest
iam/core/tenant-rbac-usecase
```

### Ports: ten crate directories

```text
iam/ports/tenant-rbac-api
iam/ports/tenant-rbac-tenant-autoscaling-contract
iam/ports/tenant-rbac-tenant-availability-contract
iam/ports/tenant-rbac-tenant-cost-allocation-contract
iam/ports/tenant-rbac-tenant-egress-policy-contract
iam/ports/tenant-rbac-tenant-image-provenance-contract
iam/ports/tenant-rbac-tenant-residency-contract
iam/ports/tenant-rbac-tenant-resource-quota-contract
iam/ports/tenant-rbac-tenant-secret-boundary-contract
iam/ports/tenant-rbac-tenant-workload-identity-contract
```

### Adapters: five crate directories

```text
iam/adapters/tenant-rbac-postgres-rls-storage
iam/adapters/tenant-rbac-postgres-rls-transaction-contract
iam/adapters/tenant-rbac-postgres-rls-write-contract
iam/adapters/tenant-rbac-storage-inmemory
iam/adapters/tenant-rbac-workflow-inmemory
```

### Facades and evidence: seventeen crate directories

```text
iam/facade/tenant-rbac-app
iam/facade/tenant-rbac-audit-chain-runtime-evidence
iam/facade/tenant-rbac-deployment-evidence
iam/facade/tenant-rbac-disbursement-evidence
iam/facade/tenant-rbac-erp-parity-map
iam/facade/tenant-rbac-identity-provider-runtime-evidence
iam/facade/tenant-rbac-identity-provider-verification
iam/facade/tenant-rbac-listener-gateway
iam/facade/tenant-rbac-listener-runtime-evidence
iam/facade/tenant-rbac-local-inmemory-harness
iam/facade/tenant-rbac-local-runtime-composition
iam/facade/tenant-rbac-postgres-rls-runtime-evidence
iam/facade/tenant-rbac-readiness-gate
iam/facade/tenant-rbac-slo-evidence
iam/facade/tenant-rbac-statutory-filing-evidence
iam/facade/tenant-rbac-tenant-workload-runtime-evidence
iam/facade/tenant-rbac-workflow-runtime-evidence
```

### Hand-authored OpenSLO files: four files

```text
iam/observability/slos/tenant-rbac/tenant-rbac-audit-emission-lag-p99.openslo.yaml
iam/observability/slos/tenant-rbac/tenant-rbac-availability.openslo.yaml
iam/observability/slos/tenant-rbac/tenant-rbac-latency-p99.openslo.yaml
iam/observability/slos/tenant-rbac/tenant-rbac-readiness-gate-correctness.openslo.yaml
```

`Cargo.lock` may change only as mechanically required by removal of those 39
packages. `iam/BUCK`, root `Cargo.toml`, retained identity packages, ADR-0710,
and every HR/Payroll/Accounting path are outside this lane. Git does not track
the empty `iam/observability/slos/tenant-rbac/` directory.

</deletion_inventory>

<parallelism>

## Disjoint-write scheduling

The owner-law, IAM Buck cleanup, HR authority correction, and founder-amendment
lanes have disjoint files and may be prepared independently. IAM does not write
HR law; the protected merge of the reviewed HR correction is an external
ordering dependency. Observation of either draft or commit is not protected
authority and cannot release deletion.

The deletion lane is serialized because it owns `Cargo.lock`. No other
structural or dependency lane runs against that lockfile concurrently. The
future Kubernetes feature is k8s-owned and starts only from amended authority.
The future IAM Connect feature may proceed independently once its own
proto/runtime decision fixes a write set disjoint from retirement. Neither
future feature is stacked into deletion.

Policy and Secrets transfers are independently gated cross-owner structural
lanes and serialize on their own mechanical `Cargo.lock` updates. Retained IAM
behavior may fan out by existing leaf package only when each write set is
disjoint from those transfers and from retirement; missing packages, proto,
facades, or ports remain separate structural/new-feature work.

One worktree has one writer. A failing prerequisite becomes a new semantic
resolution card; the worker does not widen its file set.

</parallelism>

<deletion_evidence>

## Evidence for `remove-tenant-vertical-cone`

Before mutation:

1. Record the exact `origin/dev` SHA and founder-accepted ADR-0710 amendment.
2. Confirm that the protected HR authority change carrying the correction
   reviewed at `4203d2f794bea0abdf3ef2f1032a2729819f6c6f` is merged into that
   `origin/dev` base; an open PR or reachable commit does not satisfy the gate.
3. Enumerate the 39 directories and four files from the candidate tip and
   compare them exactly with this set. Missing or additional paths refuse
   dispatch.
4. Record locked Cargo reverse dependencies and Buck reverse dependencies.
5. Confirm the separate `iam/BUCK` cleanup has landed so target-scoped Buck
   queries evaluate rather than fail on the deleted governance loader.

After mutation, on the same tip:

```text
git ls-files | rg '^iam/(core|ports|adapters|facade)/tenant-rbac-'
git ls-files iam/observability/slos/tenant-rbac
rg -n 'tenant-rbac|tenant_rbac' iam Cargo.toml Cargo.lock
buck2 targets //iam/...
buck2 build //iam/...
buck2 test //iam/...
cargo metadata --locked --offline --format-version 1
cargo fmt --all --check
cargo nextest run --locked --workspace --profile ci
cargo clippy --workspace --all-targets -- -D warnings
```

The two path scans return empty. The text search has no implementation,
manifest, package, or lock reference; owner-law history/provenance is reviewed
and allowed. Buck build/test prove the surviving IAM target closure. Workspace
Cargo runs only in the protected serialized integration lane and proves that
the mechanical lockfile is complete.

Failure injection removes one entry from a synthetic copy of the expected set
and proves the preflight comparison refuses; retains one SLO and proves the
negative scan refuses; and introduces a forbidden IAM-to-app edge fixture and
proves dependency admission rejects it. No deletion success is inferred from
file counts alone.

</deletion_evidence>

<risks>

## Known risks and controls

| Risk | Control |
|---|---|
| A worker interprets ADR-0719 as immediate deletion authority | Founder-accepted ADR-0710 correction is an explicit hard entry gate |
| Stale HR law dispatches split-first or HR-only IAM salvage | Deletion requires the corrected HR authority to be protected-merged into its base; IAM never edits or accepts dispatch authority from an HR plan |
| The false live-instance citation is fixed by weakening admission | Amendment acceptance requires the abstract VAP/CEL+RBAC+PSA, fail-closed, no-remote-PDP invariant verbatim in substance |
| A 37-crate compromise leaves a smaller review product | Exact-set comparison admits only all 39 crates plus four SLO files |
| Useful identity behavior is copied from the vertical cone | Canonical identity/OIDC/JWKS/SCIM behavior is retained in place; deletion performs no salvage or move |
| Buck cannot evaluate because `iam/BUCK` loads deleted governance code | Separate systemic cleanup lands first and remains outside deletion |
| Mechanical lockfile work conflicts with another lane | Deletion is the sole serialized `Cargo.lock` writer |
| A fake process or REST wrapper is presented as progress | Future Connect facade is a later generated-protobuf feature with real listener evidence |
| Kubernetes admission moves into IAM | New admission implementation is owned and reviewed under `k8s/` |

</risks>

<completion>

This worker handoff ends after the four documents are locally verified and a
signed commit is returned to the coordinator because this dispatch forbids a
push or GitHub mutation. The authority change is not complete or mergeable
evidence until a protected PR against `dev` has green `presubmit`, independent
APPROVE, resolved review threads, satisfied branch protection, and a protected
squash merge. This lane does not dispatch deletion, amend ADR-0710 or ADR-0719,
repair `iam/BUCK`, transfer Policy/Secrets code, implement Kubernetes
admission, mature identity behavior, or start the IAM Connect facade.

</completion>
