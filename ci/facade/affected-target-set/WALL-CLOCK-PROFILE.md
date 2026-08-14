# `gate · affected-set` wall-clock profile (measured 2026-08-08)

Measurement record for bead `oyatie-zng`. Every number below was read off a completed
`oya-ci-required` run via `gh api .../jobs`, or off the job log; none is estimated.
**No code changed to take these measurements** — every number was read off a completed run. The
PR carrying this document is NOT documentation-only: it also (a) makes `resolve()` UNION synthetic
dependency declarations with a non-empty `owner()` instead of consulting them only as a no-owner
fallback (`src/lib.rs`), (b) seeds `**/*.md` to
`root//governance/check/adr-citation-closure:check-adr-citation-closure-gate` and drops `**/*.md`
from `inert_selection_classes` (`affected-set-policy.json`), and (c) re-freezes the
citation-closure census against dev at 885794461 (`adr-citation-closure-policy.json`). (a) and (b)
are selector-authority changes and are the repair for the false green this branch itself hit. The
conclusion below — that FULL-tier work is irreducible inside this package, and that the one lever
which does not shrink coverage lives outside it — is about the measurement, not about the diff.

## 1. The job is trimodal, not a single number

`gate · affected-set` duration over the 7 most recent completed successful runs
(2026-08-07T23:05Z .. 2026-08-08T11:27Z):

| run | event / branch | job | `Binding affected-set build + test` | decision |
|---|---|---|---|---|
| 31226011563 | PR `evidence/completion-packets` | 6.0m | 0s | NO-GRAPH-TARGETS |
| 31255030523 | PR `fix/modeled-crypto-not-callable` | 8.5m | 179s | cone (no escalation) |
| 31241020054 | push `dev` | 26.2m | 1331s | FULL (push tier) |
| 31254973893 | push `dev` | 27.9m | 1424s | FULL (push tier) |
| 31253758836 | PR `gate/adr-citation-closure` | 28.9m | 1392s | FULL (escalated) |
| 31249079674 | PR `gate/workflow-lane-preflight` | 29.1m | 1393s | FULL (escalated) |
| 31248740033 | PR `gate/adr-citation-closure` | 29.1m | 1409s | FULL (escalated) |

FULL runs (n=5): binding step median 1393s, range 1331–1424 (±3.3%). The bead's 29.1/29.4-minute
figure is correct **for a FULL run** and is not steady state across event classes.

## 2. Inside a FULL run, one command is 86% of it

Phase telemetry, two independent FULL runs:

| phase | 31248740033 | 31253758836 |
|---|---|---|
| `buck2 build //... --keep-going --build-report` | 1201s | 1193s |
| `buck2 test //... --keep-going` (test-health ratchet) | 207s | 198s |
| binding step total | 1409s | 1392s |

Both runs report the identical action count for the head build:

```
Cache hits: 0%
Commands: 14362 (cached: 0, remote: 0, local: 14362)
```

`cached: 0, remote: 0` on every buck2 invocation in this job — the derive preflight
(`Commands: 4`, `Commands: 2`, `Commands: 454`), the face materializer (`Commands: 220`),
and the head build (`Commands: 14362`). The job starts from an empty runner-local `buck-out`
by design (ADR-0554 D10, `.github/workflows/oya-ci-required.yml:554-560`).

## 3. Fixed floor, paid by every run including the 6-minute one

`Materialize cloud-ci generated faces (out-of-graph boundary)`: 200, 216, 217, 217, 219, 220,
227s across the same 7 runs (median 217s). It is a `buck2 run` of a single binary that executes
220–454 cold local actions. Plus checkout ~20s, rustup ~9s, merge-base baseline 0–98s. Floor
≈ 250–350s regardless of the decision — which is essentially the whole 6.0m docs-only run.

## 4. Five candidate causes, all refuted by the measurement

| candidate | verdict | evidence |
|---|---|---|
| full-corpus walk where incremental would do | refuted | `phase=derive-affected-set-tier ... elapsed_seconds=0` |
| recomputing a graph that could be cached | refuted | derive is 0s; the cost is executing actions, not deriving them |
| O(n·m) join over the target set | refuted | same; the gate binary's own runtime is sub-second |
| a second full test pass inside one job | refuted | `buck2 test //...` is 198–207s because it reuses the build already in `buck-out` |
| cold merge-base rebuild (11m12s–17m48s) | refuted | `build-health: trusted merge-base baseline pair REUSED from run 31246463235 ... the cold merge-base rebuild is skipped` — 86–98s on PRs, 0s on pushes |
| `[]` affected set doing FULL-tier work for the wrong reason | refuted | the fast runs resolve `decision=NO-GRAPH-TARGETS` naming all 7 changed files, licensed by `inert_selection_classes`; the anti-vacuity predicate is live (`affected-set-policy.json:252`) |
| **cold buck2 with no warm cache** | **confirmed** | `Cache hits: 0%`, `remote: 0` on 14362 actions |

## 5. Why FULL fires so often, and why that must not be narrowed

Run 31248740033 escalated for two reasons, both correct:

```
- buildfile `governance/check/adr-citation-closure/BUCK` changed (blast radius exceeds its own package)
- unowned path `Cargo.lock` has no buck2 owner and no synthetic-dependency declaration (derivation uncertainty)
```

Any PR that adds a crate produces both. The buildfile rule (`src/lib.rs:589-601`) is sound: a
new or edited BUCK file adds and removes targets that dependents resolve, so a head-only
`rdeps()` cone cannot bound it. Narrowing either trigger buys wall clock by checking less.
**Not doing it.** The tier decision is not the defect; the cold cache is.

## 6. The lever, and the gap nobody has closed

Filesystem snapshotting of `buck-out` is closed by decision — a 6.37 GB `actions/cache` archive
crossed the owned node's ephemeral-storage eviction threshold on 2026-08-01
(`oya-ci-required.yml:554-560`). The sanctioned route is a Buck2-aware remote action cache + CAS,
and most of it already exists:

- `infra/ci/buckconfig/warm-cache-ro.buckconfig` / `warm-cache-rw.buckconfig` carry endpoint and
  instance materialization tokens plus `tls = true`, `remote_cache_enabled = true`, and the cache
  execution platform. The controller validates `specs/cache-endpoints.json` and fills those tokens
  only in its private per-child config. Shipped dark.
- `specs/cache-warm-license.json` ships `warm_reads_licensed: false`; ADR-0556 D2 is an IFF —
  warm-eligible class AND the most recent scheduled cold integrity-canary GREEN. The resolver
  refuses every warm mode until then. **This is a blocking precondition, not a footnote.**
- **The gap:** `oya-cloud-ci-cache-wiring-bin` has 12 call sites — 9 in
  `.github/workflows/cache-integrity-canary.yml`, 3 in `oya-ci-required.yml` (:587, :604, :606),
  all three inside the `buck2` job (:502-626, `CACHE_BUILD_CLASS: untrusted-author-presubmit`).
  The `gate-affected-target-set` job (:650-1026) **never calls the resolver.** Licensing warm
  reads tomorrow would not move this job by one second.

Ordered work, none of it in this package, in **two phases**. Step 0 is an authorization gate, not
engineering — but it gates *activation*, not the bootstrap that produces the evidence activation
itself demands:

0. **Go-gate, and its scope.** `docs/decisions/ADR-0700-ci-admission-live-apex.md` live hard norm 4
   keeps warm CAS / RE activation **fail-closed** until an explicit go-gate: credentials (#1541),
   cache-only proof, and an **Accepted** activation ADR. That norm also says outright that apex
   gists mentioning `remote_enabled=true` are historical design, not activation authority — which
   is exactly what the dark `warm-cache-*.buckconfig` wiring quoted above is. What norm 4 gates is
   **activation**: licensing warm reads, mounting a cache identity into the required lane, and
   `remote_enabled=true`. It does **not** gate deploying a cache-only CAS or running the
   pre-license canary. It cannot: the gate's own criterion is "cache-only proof", and that proof is
   produced by *operating* the cache-only tier. Reading step 0 as blocking the bootstrap makes the
   gate demand an artifact only the steps it blocks can produce.

### Phase A — non-activating bootstrap (no go-gate)

`specs/cache-warm-license.json` stays `warm_reads_licensed: false` and the required lane stays cold
throughout. This phase **produces** the cache-only proof the gate demands.

- **A0. KMS/TLS bootstrap — EXTERNAL DEPENDENCY, not owned by this rollout.** A3's identity exchange
  presumes an OpenBao that speaks authenticated TLS on `:8202`, and the RUNBOOK ordering that governs
  A1 puts the CAS deploy after this block's readback. **Declared state provides neither.**
  `infra/gitops/values.yaml:95` registers the `openbao` Application with `include:
  openbao.k8s.yaml` only, and that manifest declares `listener "tcp" { address = "0.0.0.0:8200";
  tls_disable = true }` with a Service exposing exactly `{http 8200, cluster 8201}` — no 8202, no
  TLS. `openbao-tls-migration.k8s.yaml` and `openbao-ci-identity.k8s.yaml` exist on disk but appear
  in **no** `include:` (the only mention of the former is the comment at `values.yaml:93-94` saying
  promotion is a reviewed source switch). This block is listed so the dependency is **recorded, not
  implied**; it is sequenced by `infra/external-secrets/RUNBOOK.md:25-60`, whose order is
  authoritative over the wording of any single comment.
  - *(i) TLS bootstrap* (RUNBOOK step 1). Secret `oya-kms/openbao-server-tls` (`tls.crt`/`tls.key`)
    plus the **populated** `openbao-offline-root-ca` ConfigMap in both `external-secrets` and
    `arc-runners`, with the certificate covering `openbao.oya-kms.svc`. Do not apply the empty
    public-CA scaffold directly.
  - *(ii) Identity roles* (RUNBOOK step 2). Apply `infra/kms/openbao-ci-identity.k8s.yaml` and
    bootstrap the JWT + PKI roles **over the existing plaintext 8200 listener** — this is the origin
    of `github-cas-writer-dev-push` / `github-cas-reader-integrity-canary` and the two `pki_cas_*`
    roles that A3 needs. Note the ordering consequence: role *creation* happens on 8200 before TLS
    exists; what requires `:8202` is the canary controller's **runtime** OIDC->PKI exchange.
  - *(iii) TLS promotion* (RUNBOOK step 3). A **reviewed PR** switching `infra/gitops/values.yaml:95`
    from `openbao.k8s.yaml` to `openbao-tls-migration.k8s.yaml` **and** adding an exact-path
    Application for `infra/kms/openbao-ci-identity.k8s.yaml` with `cascadeDelete: true`. Raw
    `kubectl apply` of the migration manifest is **forbidden**: Argo owns the same Deployment,
    Service and NetworkPolicy with prune + self-heal, so a direct apply is reverted or fights the
    controller.
  - *(iv) Readback* (RUNBOOK step 4). Service ports `8200..8203`, authenticated TLS health on
    `8202`, plaintext `8200` still answering during the dual-listener phase, and the Namespace +
    `openbao-data` PVC UIDs identical before and after reconciliation.
- **A1.** Deploy the cache-only tier — `storage/adapters/nativelink/nativelink-cas.k8s.yaml` (CAS +
  Action Cache only; the scheduler and worker tiers are deliberately not deployed until the RE
  phase). Deploying it does **not** make it reachable from the canary; that is A2, and it is work,
  not a property.
  - *Precondition and ordering, against declared state rather than against the manifest.* The
    manifest existing is not the manifest being deployed: `grep -n nativelink infra/gitops/values.yaml`
    returns **nothing**, so nothing reconciles this file today. Per `infra/external-secrets/RUNBOOK.md`
    step 5, it is added as an **exact-path GitOps Application with `cascadeDelete: true`**, in a
    reviewed PR, and **only after** the A0(iv) KMS/TLS readback — never by raw `kubectl
    apply`, which would leave Argo owning the same objects with prune + self-heal. A1 is therefore
    ordered after A0, not first.
- **A2.** **Move the canary onto an in-cluster runner.** Without this step A4 cannot produce the
  cache-only proof at all, and the omission is invisible because it reads as a property of A1
  rather than a task. The CAS is cluster-internal: both Services are ClusterIP (no `spec.type`,
  `nativelink-cas.k8s.yaml:357-383`), there is no Ingress, LoadBalancer, NodePort or Gateway
  anywhere under `storage/adapters/nativelink/` or `infra/ci/`, and the endpoint DATA materialized
  into the private buckconfig dials
  `grpc://nativelink-cas-{writer,reader}.oya-ci.svc.cluster.local:{50051,50052}` — cluster DNS,
  which a GitHub-hosted runner cannot resolve or route. The canary is pinned to hosted amd64
  (`cache-integrity-canary.yml:98`, `runs-on: ubuntu-latest`) and is consumed via `workflow_call`,
  which cannot override `runs-on` — so **both** lanes are affected, not just A4: A3's writer seed
  runs the same reusable workflow (`oya-ci-required.yml:627-635`). Left as-is, A4's verdict stays
  `INACTIVE_NO_ENDPOINT` (nonzero) permanently. The move itself is a one-line `runs-on` change to
  the `oya-arm64` scale set; the CAS client labels it needs are **already provisioned and not
  outstanding work** — `infra/arc/runner-scale-set-arm64-values.yaml:51-62` already sets
  `oya.io/nativelink-cas-{reader,writer}: "true"` on the runner pod template, and that file is
  registered in GitOps at `infra/gitops/values.yaml:131`, so the labels come with the fleet for
  free. Do not re-provision them.
  - *Why not the alternative.* Exposing the CAS externally would also require replacing the
    `nativelink-cas-ingress` NetworkPolicy, which admits :50051/:50052 only `from` a
    `podSelector` — and a podSelector can only ever match a pod **in** the cluster, so no hosted
    runner can satisfy it regardless of routing. That is a larger, security-relevant change than
    moving the canary, and it is not chosen here.
  - *Consequence this step must carry.* `oya-arm64` is an **arm64** fleet (arch-pinned by
    nodeSelector) while the required lane is amd64-hosted, and buck2 action keys include `cpu:`
    and `os:` — the same file's comment at line 66 says so. An arm64 canary therefore proves
    byte-equality in a **different cache namespace** from the one the required amd64 lane would
    later consume. State that in the activation record, or a future lane will read an arm64 GREEN
    as a licence for the amd64 lane. Covering the required lane needs either an amd64 in-cluster
    fleet or an explicit, separately argued cross-arch claim.
- **A3.** Issue the fixed reader/writer OIDC identities the canary controller exchanges for
  (`cache-integrity-canary.yml:182-189`). **This is not a standalone action.** The identities are
  the `github-cas-writer-dev-push` / `github-cas-reader-integrity-canary` JWT roles and the two
  `pki_cas_*` PKI roles created in **A0(ii)**, and the canary performs the exchange against
  `OYA_OPENBAO_ADDR: https://openbao.oya-kms.svc:8202` (`cache-integrity-canary.yml:75`, asserted —
  not incidental — by `infra/arc/tests/ci_workspace_capacity.rs:1353`). That address **does not
  resolve until A0(iii) lands**: declared state today terminates plaintext 8200 only. Until then
  this step has no endpoint to exchange against.
- **A4.** Set repo var `OYA_CAS_IDENTITY_PROOF_ENABLED=true` so the **already-wired**
  `cache-writer-identity` job (`oya-ci-required.yml:627-635`, trusted dev push only) seeds the
  fresh CAS.
- **A5.** Run the `workflow_dispatch` reader proof with `prelicense_probe=true` and `writer_run_id`
  set to the same-SHA writer run (`cache-integrity-canary.yml:44-66`, `:167-174`, `:205`). Its
  byte-equality canary verdict **is** the cache-only proof.

Three live artifacts confirm the repo already treats this bootstrap as non-activating and expects it
*ahead* of any gate: the canary workflow's own header ("THIS JOB THEREFORE FAILS DAILY UNTIL CAS
EXISTS, AND THAT IS THE CORRECT READING", verdict `INACTIVE_NO_ENDPOINT` exiting nonzero — CAS
bring-up is *awaited*, not gated); the `cache-writer-identity` job already shipped behind a repo
var; and the canary's `prelicense_probe` input, whose verdict condition runs while
`steps.license.outputs.warm_licensed != 'true'` — a proof path designed to execute *before*
licensing.

### Phase B — behind the ADR-0700 norm 4 go-gate

Credentials (#1541), the Phase-A cache-only proof, and an **Accepted** activation ADR.

- **B1.** Flip `specs/cache-warm-license.json` `warm_reads_licensed`, recording the licensing run in
  `licensed_by_canary_run`.
- **B2.** **The PR build class is not one line of data.** `specs/cache-warmth-policy.json` pins
  `untrusted-author-presubmit` to `warmth: cold, cache_read: false, cache_write: false`, and its
  reason text calls a read-only relaxation "a reviewed two-way policy edit". The same file's
  `product_contract.door_asymmetry` overrides that reason: the cold-required floor is **one-way**.
  `untrusted-author-presubmit` is one of the four `COLD_REQUIRED_FLOOR` classes in
  `ci/facade/build-cache-policy/tests/cache_conformance.rs:34-39`, and
  `cold_required_floor_holds_even_under_a_licensed_fixture` (`:696-714`) binds cold /
  `cache_read: false` / `cache_write: false` / `Bypass` **even under a licensed fixture** — so
  editing the JSON alone turns that live test RED, and leaving it alone leaves the step inert. The
  move is an ADR **plus** a `COLD_REQUIRED_FLOOR` amendment **plus** the JSON edit, in that order,
  and the door now runs through ADR-0703 (live apex for `cas_cache`), which superseded ADR-0556 —
  not through ADR-0556, which is itself already Superseded.

  **The consequence this profile must not bury:** while `untrusted-author-presubmit` stays on the
  cold floor, warm reuse cannot reach the PR lane at all. The CAS then buys the dev-push FULL runs
  (26.2m / 27.9m — rows 31241020054 and 31254973893 in section 1) and **not** the escalated PR FULL
  runs (28.9–29.1m) this bead is actually about. The cheaper alternative, needing no ADR and no
  floor amendment: run the dev-push producer under a trusted warm-eligible class and leave PRs
  cold. That speeds the dev-push tier and leaves every PR-lane number in section 1 unchanged.
- **B3.** **Issue and mount a cache identity for this lane.** `controlled_child` resolves its
  overlay through `effective_buckconfig`, which **rejects every warm mode** when
  `OYA_CACHE_TLS_CLIENT_CERT` is unset, empty or non-absolute
  (`ci/facade/build-cache-policy/src/lib.rs:230-245`), and `gate-affected-target-set` grants only
  `contents: read` / `actions: read` and mounts no secret (`oya-ci-required.yml:652-654`). Wiring
  the resolver in without this step either fails the required job or leaves it in bypass — no
  reuse either way. Fork PRs are handed no secret at all, so they must resolve to a declared
  cold/bypass class rather than to a broken warm one. Mounting a secret into the **required** job is
  activation, which is why this sits after the gate and not in Phase A.
- **B4.** Wire `gate-affected-target-set`'s build + test through `cache-wiring-bin` with its own
  build class (trusted-push for the dev-push producer, read-only untrusted-author for PRs, cold for
  forks). This is a `.github/workflows/oya-ci-required.yml` edit and is the piece that has never
  been written.

  **PRECONDITION, not covered by B3 — reachability.** `gate-affected-target-set` is
  `runs-on: ubuntu-latest` (`oya-ci-required.yml:657`) while `cache-wiring-bin` dials
  `nativelink-cas-{writer,reader}.oya-ci.svc.cluster.local:{50051,50052}` from the validated
  `specs/cache-endpoints.json` profile. A hosted runner cannot resolve or route those,
  and per A2 external exposure is ruled out by the `nativelink-cas-ingress` `podSelector`, which no
  hosted runner can ever satisfy. B3 supplies a **certificate, not reachability** — so with B3 alone
  this lane resolves to bypass or fails outright. **No reuse either way.** This step therefore also
  requires moving the job onto an in-cluster runner.

  **And that is NOT the one-line change A2 was.** This is the **binding linux-amd64** lane, and the
  only in-cluster fleets declared in `infra/gitops/values.yaml` are `oya-arm64` (`:125`) and
  `oya-live-postgres-arm64` (`:137`) — **both arm64**. The comment at `:122-124` says an amd64 box
  "adds a sibling entry + values file", i.e. **none exists today**. Covering this lane therefore
  needs either an amd64 scale set (a new `infra/arc/runner-scale-set-amd64-values.yaml` plus a
  sibling Application entry) or the explicitly argued cross-arch claim the A2 consequence note
  demands — an arm64 GREEN is not a licence for the amd64 lane, because buck2 action keys include
  `cpu:` and `os:`. Recording that dependency is part of this step; do not leave it implied.
- **B5.** Re-measure. Do not quote a speedup before then.

What is measured about reuse, locally, on this package's own graph:

```
cold: Commands: 690 (cached: 0, remote: 0, local: 690)   Network: Down: 29MiB   Pass 7 Fail 0 Skip 0
warm: (no Commands line — zero actions executed)          Network: Down: 0B      Pass 7 Fail 0 Skip 0
```

That establishes the actions are stably keyed and re-usable across invocations — the
precondition for remote reuse. It does **not** establish the hit rate a remote CAS would reach
across runners, and no number for that is asserted here.
