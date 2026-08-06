---
id: ADR-0513
status: Superseded
superseded_by: [ADR-0515]
planning_impact: true
date: 2026-05-30
owners:
  - council-architecture
  - founder
relates:
  - ADR-0380
  - ADR-0111
  - ADR-0116
  - ADR-0374
  - ADR-0363
  - ADR-0392
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.

# ADR-0513: oya-ci — bespoke-Rust Prow (cloud-scm-native CI/CD platform) (superseded by ADR-0515)

## Status

Accepted — 2026-05-30 (founder-locked). **Phased replacement of ADR-0380's cloud-ci+Groovy gate path**
by a bespoke-Rust platform — the formal supersession of ADR-0380 lands at the Phase-1 cutover (when
the cloud-ci gate path is deleted); cloud-ci remains a hardened BRIDGE meanwhile. Folds ADR-0111 (merge-queue) and ADR-0116
(reviewer-APPROVE) into the platform's `tide`/`plugins` components; retains ADR-0374's webhook
gateway as the `hook`; on the ADR-0363 cloud-scm substrate; builds on ADR-0392 (Buck2).

**Amendment — 2026-06-02:** Tide/merge-queue is not an optional deferred adoption. It is owned here, in cloud-ci/oya-ci,
with Phase 0 placing the admission contract and Phase 1 scaling projected-state/batch automation. `oya` CLI invocations
are not CI authority; their semantics must be ported into Rust cloud-ci gate crates/adapters.

**Superseded — 2026-06-06:** Reshaped + ratified by ADR-0515 — 2026-06-06: the bespoke-Rust instinct is the seed; the 'clone Prow's 8 components' framing is superseded by the two-nouns/four-faces model. Authority moves to ADR-0515. cloud-scm-native wording replaces forgejo-native throughout.

## Date

2026-05-30

## Context

CI-go-live (ADR-0380) stood up an enforced gate: a cloud-scm webhook → the bespoke-Rust
`ci-webhook-gateway` → a cloud-ci `oya-ci-gate` pipeline (Groovy, loaded via `cpsScm` from `dev`) →
an ephemeral cloud-ci agent pod → the buck2 affected-gate script → a cloud-scm commit-status. The
gateway and the gate **logic** (`infra/ci/buck2-affected-gate.sh`) proved solid; the
**cloud-ci+Groovy+cpsScm+ephemeral-agent layer did not.** Five distinct failure modes were observed
in production this week:

1. **Parse fragility.** A stray `'''` in a *comment* inside an `sh '''…'''` block terminated the
   Groovy string → the pipeline failed to LOAD → no status posted on any PR. Groovy CPS is
   un-lintable locally and fails on load.
2. **Self-deadlock.** A broken gate-config on `dev` cannot be fixed *through* the gate (the required
   `oya-ci-gate` status can't be produced), forcing repeated admin-bypass of branch protection.
3. **Un-introspectable.** cloud-ci is in-cluster (ClusterIP, no console access); failures had to be
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
10× our need, would displace the working cloud-scm-native Rust gateway); we **adopt the shape, not
the code**, in pure Rust. The founder scoped the FULL Prow component set (not a minimal runner),
because merge-automation, job-types, UI, and plugins are genuinely needed — and `tide` *is* the
merge-queue (ADR-0111), so one platform unifies several separately-planned systems.

## Decision

Build **`oya-ci`**: a bespoke-Rust, cloud-scm-native, K8s-native (kube-rs) reimplementation of Prow's
full component shape, on the Talos substrate. We adopt Prow's decomposition; the gate **logic**
(`buck2-affected-gate.sh`) and the bespoke `ci-webhook-gateway` are retained.

**Feature-parity table (Prow → oya-ci; required by the bespoke-over-OSS doctrine):**

| Prow component | oya-ci (Rust) | Subsumes / notes |
|---|---|---|
| hook (webhook ingest, event+command routing) | `oya-ci-webhook-gateway` (exists) | cloud-scm-native; extend for plugin/command dispatch |
| plank (job controller: K8s Job per job) | `oya-ci-controller` (kube-rs) | The reliable gate executor; Job-per-PR |
| crier (report status/comments) | reporter (reuse gateway's cloud-scm client) | Terminal-status-ALWAYS + failure summary |
| ProwJob + config (presubmit/postsubmit/periodic/batch) | labeled K8s Job + config | the buck2 gate = one presubmit job type |
| tide (merge automation) | `oya-ci-tide` | **= the merge-queue (ADR-0111)** + auto-merge + required-context/approval (ADR-0116); placement is immediate, scale features phase in |
| deck (web UI) | `oya-ci-deck` (Leptos/Rust-WASM) | CI visibility for founder + agents; follows ADR-0393 frontend decision |
| sinker (GC) | K8s `ttlSecondsAfterFinished` + GC loop | K8s-native |
| plugins (ChatOps + governance) | `oya-ci-plugins` on the gateway | governance pipeline; reviewer-agent (ADR-0116) |
| pod-utils → GCS artifacts | `kubectl logs` + SeaweedFS-S3 | no GCS coupling (self-host lens) |

**Security (trunk-sourcing).** The controller runs a K8s Job that executes the **trunk (`dev`)** gate
script + affected-detection against the PR ref; the PR's code is built as untrusted data. A PR cannot
weaken its own gate by editing the script or Job spec. Because the controller is a **deployed service**
(not config on the branch it gates), a bad PR cannot break it — eliminating the self-deadlock (mode 2).

**Phasing.**
- **Phase 0 — Bridge + Tide placement:** harden the current bridge only as a safety stopgap, define the
  Prow/cloud-ci required context, prove trunk/controller-sourced producer security, and land the `oya-ci-tide`
  admission contract/fixtures. cloud-ci may transport status during the bridge, but `oya` CLI is not CI authority.
- **Phase 1 — Core (plank+crier+tide-minimum):** `oya-ci-controller` spawns a Job per PR, watches it, posts a
  terminal status, and enforces the minimal Tide admission gate (required contexts + reviewer/multispectrum +
  merge-tree/projected-state guard); cut over and **delete the cloud-ci gate path**. (kube-rs is already a blessed
  workspace dependency — no new dependency-seam.)
- **Phase 2 — Tide scale:** pool + batch + speculative-retest + auto-rebase/auto-merge on green; scales ADR-0111 and
  the Sweep migration engine's auto-merge after ownership/contract are already live.
- **Phase 3 — job-types + deck.** **Phase 4 — plugins / ChatOps / reviewer-agent (ADR-0116).**

## Consequences

**Positive:** kills all five failure modes (no Groovy/CPS parse fragility; deadlock-proof by being a
deployed service; `kubectl logs`-introspectable; terminal-status-always; fewer hops); one pure-Rust,
cloud-scm-native platform replaces cloud-ci + a separate merge-queue + bespoke auto-merge glue; aligns
with the bespoke-over-OSS doctrine and the `kubers`/`source` Rust-K8s ambition. **Negative/cost:** we
own a platform (reinvention/edge-case risk, mitigated by lifting Prow's proven plank state-machine and
adversarial review); a multi-phase build. **Neutral:** the gate *logic* and branch-protection are
unchanged; the gateway is retained.

**Process rule:** the Phase-1 cutover (replacing the live gate) is a deliberate, founder-gated step,
verified by a parallel-run (both gates green on the same PRs) before deleting the cloud-ci path. The same cutover proves no required context is produced by an `oya` CLI invocation.
