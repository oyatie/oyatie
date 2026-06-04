---
id: ADR-0513
status: Accepted
planning_impact: true
date: 2026-05-30
owners:
  - council-architecture
  - founder
supersedes:
  - ADR-0511
  - ADR-0380
  - ADR-0111
amends:
  - ADR-0349
superseded_by: []
relates:
  - ADR-0380
  - ADR-0111
  - ADR-0116
  - ADR-0374
  - ADR-0363
  - ADR-0392
---
# ADR-0513: oya-ci — bespoke-Rust Prow-shaped Kubernetes-native CI/CD platform

## Status

Accepted — 2026-05-30 (founder-locked). Amended 2026-06-04: this is now the active CI authority direction for dev-lane unlock. The required context is `oya-ci-required`, produced by a Rust, Prow-shaped, Kubernetes-native controller from trusted SCM/controller state. GitHub is a PR/publication adapter and GitHub Actions is shadow compatibility only. The native SCM is Sapling-inspired and Rust-biased, but it is a service/control-plane seam, not a revived `oya vcs` CLI. CD is release-conveyor-like: a native progressive-delivery/audit seam fed by signed CI evidence, not a CLI revival.

Historical 2026-05-30 context below described a phased replacement of ADR-0380's retired external
CI gate path by a bespoke-Rust platform. The 2026-06-04 amendment supersedes any
bridge-to-retired-substrate wording: no retired external SCM/CI/CD substrate is interim authority.
ADR-0111 (merge-queue) and ADR-0116 (reviewer-APPROVE) fold into the platform's
`tide`/`plugins` components. ADR-0374's webhook-gateway shape is retained as the `hook`; ADR-0363
remains retired-agentic-VCS provenance; ADR-0392 keeps Buck2 as build/test/check authority.

## 2026-06-04 amendment — fit with native SCM and conveyor-like CD

- **SCM fit:** the Sapling-inspired bespoke SCM owns worktree leases, stacked changes, semantic conflict metadata, virtual merge heads, and trusted candidate/base snapshots. It exposes Git/GitHub publication adapters while the native service matures. Do **not** revive an `oya vcs` CLI; current agent lanes use plain `git`/`gh`, and durable native control goes through service APIs, Rust libraries, Buck2 targets, and Prow jobs.
- **CI fit:** `oya-ci` consumes trusted SCM/controller state, not candidate-authored gate definitions. It creates ProwJob-style Kubernetes workloads that run Buck2 build/test/check targets, then posts `oya-ci-required`.
- **CD fit:** the CD destination is release-conveyor-like. It consumes signed build/evidence/release-ledger outputs after `oya-ci` and handles progressive delivery, rollback, policy, and audit. GitHub Actions CD artifacts remain dry-run/shadow compatibility.
- **Governance fit:** `oya gate` and `oya verify` CLI invocations are retired as CI/merge authority. Preserve useful checks only as Rust kernels, Buck2 targets, and Prow/Kubernetes-native jobs.
- **Stateless compute / cached cloud I/O fit:** `oya-ci` jobs follow `trigger -> fetch remote state -> compute/validate -> export artifacts -> destroy ephemeral workspace`. The job pod is disposable compute; durable state belongs in trusted SCM/controller refs, Buck2 remote execution/CAS/cache, immutable object artifacts, status, and audit ledgers. Cache is allowed and required for hyperscaler performance, but it is not correctness authority: it must be content-addressed, regional/cell-local for hot-path I/O, trust-domain separated, quarantined for untrusted PR writes, promoted by trusted postsubmit/periodic jobs, and covered by cold-cache probes.

## Date

2026-05-30

## Context

CI-go-live (ADR-0380) stood up an enforced gate using a third-party forge webhook, a bespoke-Rust
gateway, a retired external CI pipeline layer, an ephemeral agent pod, the Buck2 affected-gate
logic, and a forge commit-status. The gateway and the gate **logic** proved useful as design inputs;
the multi-hop retired external CI layer did not. Five distinct failure modes were observed:

1. **Parse fragility.** A stray `'''` in a *comment* inside an `sh '''…'''` block terminated the
   pipeline string → the pipeline failed to LOAD → no status posted on any PR. The DSL was
   hard to lint locally and failed on load.
2. **Self-deadlock.** A broken gate-config on `dev` cannot be fixed *through* the gate (the required
   required status can't be produced), forcing repeated admin-bypass of branch protection.
3. **Un-introspectable.** The CI control plane was opaque from the normal developer path; failures
   had to be reverse-engineered from a one-line status plus gateway logs.
4. **Cold-pod / no-completion.** Ephemeral agent pods (image pull, `git unshallow`) hang or die
   without posting → "pending" forever or no status; the only lever is a manual re-trigger.
5. **Too many hops.** Each of webhook→gateway→trigger→branch-loaded config→ephemeral-pod→buck2→status is a
   failure point.

Validation: the robust design *is* Prow's architecture — Kubernetes' own CI, run across CNCF at
scale — `hook` (webhook→job resource) / `plank` (controller→K8s pod per job) / `crier`
(report commit-status). Trunk-sourced presubmit (build PR code as untrusted data; run the gate
   orchestration from the trusted base branch) is GitHub's documented `pull_request` vs
`pull_request_target` security model. Adopting Prow's Go code is a poor fit (provider/storage
couplings, 10× our need, would displace the Rust-first gateway direction); we **adopt the shape, not
the code**, in pure Rust. The founder scoped the FULL Prow component set (not a minimal runner),
because merge-automation, job-types, UI, and plugins are genuinely needed — and `tide` *is* the
merge-queue (ADR-0111), so one platform unifies several separately-planned systems.

## Decision

Build **`oya-ci`**: a bespoke-Rust, provider-adapter-facing, K8s-native (kube-rs)
reimplementation of Prow's full component shape, on the Talos substrate. We adopt Prow's
decomposition; Buck2 gate semantics and the bespoke webhook-gateway shape are retained as Rust/Buck2
controller inputs, not as candidate-owned shell or retired external CI authority.

**Feature-parity table (Prow → oya-ci; required by the bespoke-over-OSS doctrine):**

| Prow component | oya-ci (Rust) | Subsumes / notes |
|---|---|---|
| hook (webhook ingest, event+command routing) | `oya-ci-webhook-gateway` (exists) | native SCM/GitHub adapter ingress; extend for plugin/command dispatch |
| plank (job controller: K8s Job per job) | `oya-ci-controller` (kube-rs) | The reliable gate executor; Job-per-PR |
| crier (report status/comments) | reporter (reuse gateway status client) | Terminal-status-ALWAYS + failure summary |
| ProwJob + config (presubmit/postsubmit/periodic/batch) | labeled K8s Job + config | the buck2 gate = one presubmit job type |
| tide (merge automation) | `oya-ci-merge` | **= the merge-queue (ADR-0111)** + auto-merge + required-context/approval (ADR-0116) |
| deck (web UI) | `oya-ci-deck` (SolidJS) | CI visibility for founder + agents |
| sinker (GC) | K8s `ttlSecondsAfterFinished` + GC loop | K8s-native |
| plugins (ChatOps + governance) | `oya-ci-plugins` on the gateway | governance pipeline; reviewer-agent (ADR-0116) |
| pod-utils → GCS artifacts | `kubectl logs` + SeaweedFS-S3 | no GCS coupling (self-host lens) |

**Cloud I/O and performance doctrine.** Stateless does not mean cacheless. The recommended
cloud-native/hyperscaler pattern is ephemeral compute with externalized, immutable, measured state:
SCM refs for source truth, Buck2 remote execution/CAS/cache for build acceleration, object storage
for logs/coverage/SBOM/build artifacts, and a status/audit ledger for merge truth. Hot-path cache and
artifact reads should stay regional/cell-local to avoid egress and latency; cross-region replication is
asynchronous. Untrusted PR jobs may read approved cache namespaces and write only quarantined cache
entries; trusted postsubmit/periodic lanes promote warmed entries. Every lane must remain correct on a
cold cache, and CI pods must declare ephemeral-storage requests/limits plus cache-hit, CAS byte,
artifact-upload-latency, and eviction metrics.

**Security (trunk-sourcing).** The controller runs a K8s Job that executes the **trunk (`dev`)** gate
script + affected-detection against the PR ref; the PR's code is built as untrusted data. A PR cannot
weaken its own gate by editing the script or Job spec. Because the controller is a **deployed service**
(not config on the branch it gates), a bad PR cannot break it — eliminating the self-deadlock (mode 2).

**Phasing.** The 2026-06-02 amendment below narrows the historical phasing: Tide
admission ownership, `oya-ci-required`, PR-head pinning, mergeability/conflict checks,
and automatic merge after CI are active Phase-0 contracts now; later Tide phases scale
projected-state batching, retest, auto-rebase, deck, and plugins.
- **Phase 0 — Native required-context baseline (in progress):** target dev branch protection at
  `oya-ci-required` from Prow/Kubernetes-native controller state; GitHub Actions may emit shadow
  evidence only and retired external SCM/CI/CD substrates are not interim authorities.
- **Phase 1 — Core (plank+crier):** `oya-ci-controller` spawns a Job per PR, watches it, posts a
  terminal status; cut over any remaining shadow/adapter evidence after parity proof. (kube-rs is
  already a blessed workspace dependency — no new dependency-seam.)
- **Phase 2 — tide / merge-queue:** pool + batch + speculative-retest + auto-merge on green; subsumes
  ADR-0111 and the Sweep migration engine's auto-merge.
- **Phase 3 — job-types + deck.** **Phase 4 — plugins / ChatOps / reviewer-agent (ADR-0116).**

## Consequences

**Positive:** kills all five failure modes (no retired CI DSL parse fragility; deadlock-proof by
being a deployed service; `kubectl logs`-introspectable; terminal-status-always; fewer hops); one
pure-Rust, provider-adapter-facing platform replaces retired external CI plus separate merge-queue
and bespoke auto-merge glue; aligns with the bespoke-over-OSS doctrine and the `kubers`/`source`
Rust-K8s ambition. **Negative/cost:** we
own a platform (reinvention/edge-case risk, mitigated by lifting Prow's proven plank state-machine and
adversarial review); a multi-phase build. **Neutral:** the gate *logic* and branch-protection are
unchanged; the gateway is retained.

**Process rule:** the native authority cutover is a deliberate, founder-gated step, verified by
source-bound `oya-ci-required` status on candidate SHAs plus Buck2/Prow parity evidence before any
shadow adapter is removed.

## Supersession

This ADR formally supersedes three prior decisions and amends one:

- **Supersedes ADR-0511** (external workflow-controller CI orchestration): ADR-0511 was Proposed
  but never Accepted. The bespoke-Rust oya-ci controller is the chosen CI-orchestration direction.
  The correct half of ADR-0511 — self-hosted, K8s-native CI orchestration, no GitHub Actions SPOF —
  is retained in spirit; only the chosen implementation changes.

- **Supersedes ADR-0380** (legacy CI-loop closure on Talos): ADR-0380 established the retired
  external CI gate path. That path is not interim authority. The five failure modes documented in
  this ADR's Context section are precisely that legacy path; ADR-0513's oya-ci-controller is the
  replacement.

- **Supersedes (folds) ADR-0111** ("Merge queue: projected-merge-state + fix-at-any-stage"): the
  merge-queue algorithm defined in ADR-0111 is subsumed by the `tide` phase of oya-ci (Phase 2).
  The projected-merge-state invariants, fix-at-any-stage re-validation, and fairness rules from
  ADR-0111 are the specification input for `oya-ci-merge`; they are not separately implemented.

- **Amends ADR-0349** (retired external CI/CD substrate): ADR-0349 is historical provenance, not
  interim authority. Active CI authority is ADR-0513 `oya-ci-required`; active CD direction is the
  release-conveyor-like native progressive-delivery seam fed by signed oya-ci evidence.

## 2026-06-02 Phase-0 Tide/admission amendment

Founder directive on 2026-06-02 makes Tide ownership an active cloud-ci/oya-ci
Phase-0 contract, not a deferrable local-process concern. Phase 0 places and
tests the admission surface: required `oya-ci-required` context, PR-head pinning,
mergeability/conflict checks, and automatic merge after CI for both the native SCM adapter and
the GitHub publication mirror. Later phases may still scale batching,
projected-state retesting, auto-rebase, deck, and plugins, but the ownership and
auto-merge-after-CI contract are decided now.

This amendment preserves the non-claim boundary: checked-in scripts/specs/tests
are local/static or bridge enforcement until the trusted cloud-ci/oya-ci
producer posts `oya-ci-required` and the live forge requires it on the candidate
SHA.
