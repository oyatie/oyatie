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
# ADR-0513: oya-ci — bespoke-Rust Prow (Forgejo-native CI/CD platform)

## Status

Accepted — 2026-05-30 (founder-locked). **Phased replacement of ADR-0380's Jenkins+Groovy gate path**
by a bespoke-Rust platform — the formal supersession of ADR-0380 lands at the Phase-1 cutover (when
the Jenkins gate path is deleted); Jenkins remains a hardened BRIDGE meanwhile. Folds ADR-0111 (merge-queue) and ADR-0116
(reviewer-APPROVE) into the platform's `tide`/`plugins` components; retains ADR-0374's webhook
gateway as the `hook`; on the ADR-0363 Forgejo substrate; builds on ADR-0392 (Buck2).

## Date

2026-05-30

## Context

CI-go-live (ADR-0380) stood up an enforced gate: a Forgejo webhook → the bespoke-Rust
`ci-webhook-gateway` → a Jenkins `oya-ci-gate` pipeline (Groovy, loaded via `cpsScm` from `dev`) →
an ephemeral Jenkins agent pod → the buck2 affected-gate script → a Forgejo commit-status. The
gateway and the gate **logic** (`infra/ci/buck2-affected-gate.sh`) proved solid; the
**Jenkins+Groovy+cpsScm+ephemeral-agent layer did not.** Five distinct failure modes were observed
in production this week:

1. **Parse fragility.** A stray `'''` in a *comment* inside an `sh '''…'''` block terminated the
   Groovy string → the pipeline failed to LOAD → no status posted on any PR. Groovy CPS is
   un-lintable locally and fails on load.
2. **Self-deadlock.** A broken gate-config on `dev` cannot be fixed *through* the gate (the required
   `oya-ci-gate` status can't be produced), forcing repeated admin-bypass of branch protection.
3. **Un-introspectable.** Jenkins is in-cluster (ClusterIP, no console access); failures had to be
   reverse-engineered from a one-line status + gateway logs.
4. **Cold-pod / no-completion.** Ephemeral agent pods (image pull, `git unshallow`) hang or die
   without posting → "pending" forever or no status; the only lever is a manual re-trigger.
5. **Too many hops.** Each of webhook→gateway→genericTrigger→cpsScm→ephemeral-pod→buck2→status is a
   failure point.

Validation: the robust design *is* Prow's architecture — Kubernetes' own CI, run across CNCF at
scale — `hook` (webhook→job resource) / `plank` (controller→K8s pod per job) / `crier`
(report commit-status). Trunk-sourced presubmit (build PR code as untrusted data; run the gate
orchestration from the trusted base branch) is GitHub's documented `pull_request` vs
`pull_request_target` security model. Adopting Prow's Go code is a poor fit (GitHub/GCS-coupled,
10× our need, would displace the working Forgejo-native Rust gateway); we **adopt the shape, not
the code**, in pure Rust. The founder scoped the FULL Prow component set (not a minimal runner),
because merge-automation, job-types, UI, and plugins are genuinely needed — and `tide` *is* the
merge-queue (ADR-0111), so one platform unifies several separately-planned systems.

## Decision

Build **`oya-ci`**: a bespoke-Rust, Forgejo-native, K8s-native (kube-rs) reimplementation of Prow's
full component shape, on the Talos substrate. We adopt Prow's decomposition; the gate **logic**
(`buck2-affected-gate.sh`) and the bespoke `ci-webhook-gateway` are retained.

**Feature-parity table (Prow → oya-ci; required by the bespoke-over-OSS doctrine):**

| Prow component | oya-ci (Rust) | Subsumes / notes |
|---|---|---|
| hook (webhook ingest, event+command routing) | `oya-ci-webhook-gateway` (exists) | Forgejo-native; extend for plugin/command dispatch |
| plank (job controller: K8s Job per job) | `oya-ci-controller` (kube-rs) | The reliable gate executor; Job-per-PR |
| crier (report status/comments) | reporter (reuse gateway's Forgejo client) | Terminal-status-ALWAYS + failure summary |
| ProwJob + config (presubmit/postsubmit/periodic/batch) | labeled K8s Job + config | the buck2 gate = one presubmit job type |
| tide (merge automation) | `oya-ci-merge` | **= the merge-queue (ADR-0111)** + auto-merge + required-context/approval (ADR-0116) |
| deck (web UI) | `oya-ci-deck` (SolidJS) | CI visibility for founder + agents |
| sinker (GC) | K8s `ttlSecondsAfterFinished` + GC loop | K8s-native |
| plugins (ChatOps + governance) | `oya-ci-plugins` on the gateway | governance pipeline; reviewer-agent (ADR-0116) |
| pod-utils → GCS artifacts | `kubectl logs` + SeaweedFS-S3 | no GCS coupling (self-host lens) |

**Security (trunk-sourcing).** The controller runs a K8s Job that executes the **trunk (`dev`)** gate
script + affected-detection against the PR ref; the PR's code is built as untrusted data. A PR cannot
weaken its own gate by editing the script or Job spec. Because the controller is a **deployed service**
(not config on the branch it gates), a bad PR cannot break it — eliminating the self-deadlock (mode 2).

**Phasing.** The 2026-06-02 amendment below narrows the historical phasing: Tide
admission ownership, `oya-ci-required`, PR-head pinning, mergeability/conflict checks,
and automatic merge after CI are active Phase-0 contracts now; later Tide phases scale
projected-state batching, retest, auto-rebase, deck, and plugins.
- **Phase 0 — Bridge (in progress):** harden Jenkins (`post{aborted}` terminal status — landed; +
  presubmit Jenkinsfile-parse validation, warm image) to stop the bleeding while Phase 1 is built.
- **Phase 1 — Core (plank+crier):** `oya-ci-controller` spawns a Job per PR, watches it, posts a
  terminal status; cut over and **delete the Jenkins gate path**. (kube-rs is already a blessed
  workspace dependency — no new dependency-seam.)
- **Phase 2 — tide / merge-queue:** pool + batch + speculative-retest + auto-merge on green; subsumes
  ADR-0111 and the Sweep migration engine's auto-merge.
- **Phase 3 — job-types + deck.** **Phase 4 — plugins / ChatOps / reviewer-agent (ADR-0116).**

## Consequences

**Positive:** kills all five failure modes (no Groovy/CPS parse fragility; deadlock-proof by being a
deployed service; `kubectl logs`-introspectable; terminal-status-always; fewer hops); one pure-Rust,
Forgejo-native platform replaces Jenkins + a separate merge-queue + bespoke auto-merge glue; aligns
with the bespoke-over-OSS doctrine and the `kubers`/`source` Rust-K8s ambition. **Negative/cost:** we
own a platform (reinvention/edge-case risk, mitigated by lifting Prow's proven plank state-machine and
adversarial review); a multi-phase build. **Neutral:** the gate *logic* and branch-protection are
unchanged; the gateway is retained.

**Process rule:** the Phase-1 cutover (replacing the live gate) is a deliberate, founder-gated step,
verified by a parallel-run (both gates green on the same PRs) before deleting the Jenkins path.

## Supersession

This ADR formally supersedes three prior decisions and amends one:

- **Supersedes ADR-0511** ("CI orchestration = Argo Workflows"): ADR-0511 was Proposed but never
  Accepted. The bespoke-Rust oya-ci controller is the chosen CI-orchestration direction; Argo
  Workflows (a CNCF OSS adoption) is superseded in favour of the bespoke-over-OSS doctrine that
  governs this codebase. The correct half of ADR-0511 — self-hosted, k8s-native CI orchestration,
  no GitHub Actions SPOF — is retained in spirit; only the chosen implementation changes.

- **Supersedes ADR-0380** ("CI-loop closure on Talos: Jenkins farm re-establishment + Forgejo
  gating"): ADR-0380 established the Jenkins-farm gate path (generic-webhook-trigger + Groovy
  pipeline + ephemeral agent pods). That path is being retired as described in Phase 1 of this ADR.
  The five failure modes documented in this ADR's Context section are precisely the ADR-0380 Jenkins
  path. On Phase-1 cutover (deletion of the Jenkins gate path), ADR-0380's gate design is fully
  retired; ADR-0513's oya-ci-controller is the replacement.

- **Supersedes (folds) ADR-0111** ("Merge queue: projected-merge-state + fix-at-any-stage"): the
  merge-queue algorithm defined in ADR-0111 is subsumed by the `tide` phase of oya-ci (Phase 2).
  The projected-merge-state invariants, fix-at-any-stage re-validation, and fairness rules from
  ADR-0111 are the specification input for `oya-ci-merge`; they are not separately implemented.

- **Amends ADR-0349** ("Jenkins (LTS) + ArgoCD canonical CI/CD substrate"): this ADR retires ONLY
  the Jenkins-CI half of ADR-0349. ArgoCD-CD remains the canonical GitOps CD substrate per
  ADR-0349 and ADR-0375 and is NOT affected by this supersession. ADR-0349's ArgoCD decisions,
  OpenTofu module homes, cosign-verify policy, and audit-chain emitter integration are unchanged.

## 2026-06-02 Phase-0 Tide/admission amendment

Founder directive on 2026-06-02 makes Tide ownership an active cloud-ci/oya-ci
Phase-0 contract, not a deferrable local-process concern. Phase 0 places and
tests the admission surface: required `oya-ci-required` context, PR-head pinning,
mergeability/conflict checks, and automatic merge after CI for both Forgejo and
the GitHub bootstrap mirror. Later phases may still scale batching,
projected-state retesting, auto-rebase, deck, and plugins, but the ownership and
auto-merge-after-CI contract are decided now.

This amendment preserves the non-claim boundary: checked-in scripts/specs/tests
are local/static or bridge enforcement until the trusted cloud-ci/oya-ci
producer posts `oya-ci-required` and the live forge requires it on the candidate
SHA.
