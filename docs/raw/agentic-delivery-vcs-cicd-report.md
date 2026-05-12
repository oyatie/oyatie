# Agentic Delivery Fabric: Rust-first CI/CD and VCS report

**Date:** 2026-05-10
**Status:** Ralplan/autoresearch report; no implementation started.
**Request:** Design the exact delivery direction for an agentic CI/CD and VCS substrate where Git and GitHub Actions are treated as bottlenecks.
**Thesis:** Build an internal Agentic Delivery Fabric. Git/GitHub become compatibility/export surfaces, not the core workflow substrate.

---

## Executive decision

Oyatie should build a **Rust-first Agentic Delivery Fabric** with three core moves:

1. **Replace GitHub Actions as the execution substrate.**
   - GitHub Actions may remain a compatibility/status mirror while the real scheduler, runners, cache, evidence store, and release engine are Oyatie-owned.
   - The core substrate should run on ephemeral, policy-constrained agent pods and Rust-first build/test workers.

2. **Stop treating Git branches/PRs as the primary agent work model.**
   - Keep Git compatibility for interchange, migration, and external tooling.
   - Introduce an internal **Change Graph**: durable changesets, patchsets, stack edges, ownership, evidence, policy verdicts, and replay metadata.
   - Use `jj`/Jujutsu-style local ergonomics first; borrow Gerrit patchset semantics for review/submission; study Sapling/Mononoke/EdenFS as the long-term large-monorepo reference.

3. **Make autonomous agent pods first-class delivery actors.**
   - A pod is an autonomous agent/team responsible for a bounded ownership area.
   - It can plan, implement, test, review, merge, and deploy inside its autonomy tier when evidence gates pass.
   - TPM/Orchestrator handles only multiproject dependencies, sequencing, risk, incidents, capacity, and release-train tradeoffs.

The slogan: **agents do not ask to ship; they prove they are allowed to ship.**

---

## RALPLAN-DR summary

### Principles

1. **Evidence over approval:** a pod ships by producing required evidence, not by waiting for routine human or TPM permission.
2. **Git-compatible, not Git-bound:** preserve Git import/export while moving agent workflow state to a richer changeset/evidence graph.
3. **Rust-first latency:** optimize for Cargo/Rust feedback loops before generic CI abstractions.
4. **Autonomy is scoped:** pods have full authority inside declared ownership/manifests; cross-area risk routes to TPM/Orchestrator.
5. **Hyperscaler union, startup implementation:** adopt Amazon/Google/Meta/Microsoft strengths, but sequence them through small, reversible layers.

### Top decision drivers

1. **Parallel + stacked agent work** without branch/ref/rebase collisions.
2. **Fast, deterministic Rust verification** with cache/reuse, affected-graph selection, and hermetic reproducibility.
3. **Production-grade safety**: policy gates, traceability, supply-chain provenance, progressive rollout, automatic rollback.

### Viable options

#### Option A — GitHub Actions + Git PRs, optimized

- Pros: fastest to maintain; minimal migration; existing ecosystem.
- Cons: still bottlenecked by PR/branch semantics, GitHub runner constraints, weak native stack semantics, and non-agentic evidence model.
- Verdict: useful as current baseline only; not sufficient for requested velocity/autonomy.

#### Option B — Git-compatible internal Delivery Fabric with Change Graph (**recommended**)

- Pros: preserves compatibility while solving agent-native changeset, stack, evidence, and orchestration problems; can ship incrementally.
- Cons: requires new control-plane and schema discipline; risk of rebuilding Gerrit badly if scope is not tight.
- Verdict: best near/mid-term path.

#### Option C — Adopt Gerrit as primary review/change system

- Pros: mature patchsets, Change-Id, submit requirements, topics, serious pre-submit model.
- Cons: cultural/UX migration, weaker GitHub-native flow, still not enough for agent evidence/pod autonomy by itself.
- Verdict: borrow semantics; consider as backend only if Change Graph needs mature review UI quickly.

#### Option D — Sapling/Mononoke/EdenFS-style platform

- Pros: Meta-scale monorepo, stacked workflows, virtualized filesystem, huge-repo ergonomics.
- Cons: public stack is not turnkey; high operational complexity.
- Verdict: long-term reference architecture, not v0 substrate.

#### Option E — Fully custom non-Git VCS immediately

- Pros: maximum semantic fit.
- Cons: huge risk; ecosystem loss; likely delays product velocity.
- Verdict: reject for v0. Build Change Graph first; only replace storage after measured need.

---

## Benchmark: union of hyperscaler strengths

| Source model | Strength to copy | Delivery Fabric translation |
|---|---|---|
| **Amazon** | Pipelines as release captain; hands-off safe deployments; waves, bake time, automated rollback, deployment windows, immutable artifacts. Sources: [Going faster with continuous delivery](https://aws.amazon.com/builders-library/going-faster-with-continuous-delivery/), [Automating safe, hands-off deployments](https://aws.amazon.com/builders-library/automating-safe-hands-off-deployments/), [My CI/CD pipeline is my release captain](https://aws.amazon.com/builders-library/cicd-pipeline/). | Release plane owns canary/waves/bake/rollback. Humans/TPM do not manually shepherd normal releases. |
| **Google** | Trunk-based development, presubmit culture, hermetic build/test, scaled static analysis, ownership, monorepo discipline. Sources: [Software Engineering at Google — CI](https://abseil.io/resources/swe-book/html/ch24.html), [Static Analysis](https://abseil.io/resources/swe-book/html/ch20.html). | Every changeset gets presubmit lanes, ownership checks, affected graph tests, static analysis, and submit requirements before integration. |
| **Meta** | Sapling/Mononoke/EdenFS for giant monorepos; stacked diffs; Buck/Buck2 remote execution; high-concurrency developer workflows. Sources: [Sapling introduction](https://sapling-scm.com/docs/introduction/), [Sapling scale overview](https://sapling-scm.com/docs/scale/overview/), [Buck2 remote execution](https://buck2.build/docs/users/remote_execution/). | Internal Change Graph supports stacks/patchsets; long-term repo virtualization and remote execution borrowed from Meta architecture. |
| **Microsoft** | One Engineering System; Scalar/VFS-for-Git lessons; BuildXL/CloudBuild distributed caching and trustworthy incrementality. Sources: [BuildXL](https://github.com/microsoft/BuildXL), [Microsoft Build Accelerator](https://devblogs.microsoft.com/engineering-at-microsoft/large-scale-distributed-builds-with-microsoft-build-accelerator/). | Build graph must be deterministic, cache-aware, distributed, and observable; repo virtualization matters if monorepo size grows. |

**Union:** Amazon release safety + Google presubmit/trunk rigor + Meta stacked/monorepo ergonomics + Microsoft distributed deterministic builds.

---

## Why Git is the bottleneck for autonomous agents

Git is excellent as a distributed content store and interchange format. It is weak as the **primary coordination model for many autonomous agents** because:

- Branch names are mutable global-ish coordination objects.
- Rebase/force-push semantics are hazardous for unattended workers.
- PRs flatten attempts, evidence, policy, and patchset lineage into comments/statuses.
- Stacked work is bolted on through conventions, not native graph semantics.
- Cross-repo or multiproject submission is non-transactional.
- Merge queues serialize too much when the system cannot reason about true affected graphs.
- Git has no native concept of agent authority, blast radius, eval evidence, or release readiness.

Therefore the target is not “no Git anywhere.” The target is: **Git is storage/export; Change Graph is workflow truth.**

---

## Target architecture

### 1. Change Graph plane

A durable internal graph of work units.

Core objects:

```yaml
ChangeSet:
  id: chg_...
  stack_id: stk_...
  parent_changes: []
  base_snapshot: snap_...
  ownership_area: axis/foundry/repoctl
  author_actor: agent|human|pod
  patchsets:
    - patchset_id: ps_...
      diff_ref: cas://...
      created_at: ...
      attempt_id: run_...
  evidence_refs: []
  policy_verdicts: []
  submit_requirements: []
  export_refs:
    git_branch: optional
    github_pr: optional
    gerrit_change: optional
```

Required properties:

- immutable patchsets;
- mutable logical change;
- explicit stack edges;
- conflict state as data;
- ownership and autonomy tier;
- trace/evidence pointers;
- Git export/import adapter;
- `jj` adapter for local agent workspaces;
- future Gerrit/Sapling adapters if needed.

### 2. Agent Pod plane

A pod is an autonomous delivery actor.

Pod manifest:

```yaml
AgentPod:
  id: pod_foundry_repoctl
  owns:
    paths: ["tools/repoctl/**", "services/agent/daemon/**"]
    capabilities: ["repoctl.*", "agent.daemon.*"]
  autonomy:
    max_tier: T3
    can_merge_if: ["all_required_lanes_green", "blast_radius <= local", "no_cross_project_blocker"]
    can_deploy_if: ["release_plane_approves", "canary_policy_green"]
  escalation:
    tpm_required_for: ["cross_project_dependency", "root_manifest", "infra", "public_api_break", "security_boundary"]
```

Autonomy rule:

- Inside owned area + low/moderate blast radius + gates green: pod can ship.
- Cross-project, high-blast, infra, public API, security, compliance, or production incident: TPM/Orchestrator coordinates.

### 3. Rust-first CI lane engine

Rust lanes are first-class, not generic shell jobs.

Baseline lane families:

- format: `cargo fmt --all -- --check`
- fast compile: `cargo check --workspace --all-targets --all-features`
- lint: `cargo clippy --workspace --all-features --all-targets -- -D warnings`
- test: `cargo nextest run --workspace --all-features`
- affected tests: direct crates + transitive dependents
- dependency hygiene: `cargo deny`, `cargo machete`, `cargo audit` / RustSec
- API compatibility: `cargo semver-checks` for public crates
- mutation/fuzz/property lanes for tagged high-risk crates
- benchmarks for hot-path surfaces
- Miri/sanitizer/loom lanes for unsafe/concurrency surfaces

Rust acceleration stack:

- `sccache` as Rust compiler wrapper; Rust support is official but has caveats, so cache hit-rate must be measured ([sccache Rust docs](https://github.com/mozilla/sccache/blob/main/docs/Rust.md), [Cargo build cache](https://doc.rust-lang.org/stable/cargo/reference/build-cache.html)).
- `cargo-nextest` for partitioning/sharding, retries, flaky detection, and JUnit output ([partitioning](https://nexte.st/docs/ci-features/partitioning/), [retries/flaky tests](https://nexte.st/docs/features/retries/)).
- remote cache/execution via Bazel/Buck2-compatible REAPI when Cargo-native acceleration plateaus ([Bazel remote caching](https://bazel.build/remote/caching), [Bazel remote execution](https://bazel.build/docs/remote-execution), [Buck2 remote execution](https://buck2.build/docs/users/remote_execution/)).
- trusted-cache-writes only from controlled CI workers to avoid poisoning.

### 4. Execution substrate plane

Own the execution substrate.

Recommended split:

- **Temporal-style durable orchestration** for agent workflows, retries, human gates, long-running state, and replay ([Temporal workflows](https://docs.temporal.io/workflows)).
- **Kubernetes ephemeral pods** for agent runs and build workers.
- **Remote execution / CAS** for build/test scale.
- **Object store** for artifacts/evidence/logs.
- **OpenTelemetry-compatible traces** for model calls, tool calls, build steps, policy, approvals, cache metrics, and deployment events.

Tekton/Argo/Buildkite are useful references or bootstrap options, but the target product is Oyatie-owned control plane, not GitHub Actions or a hosted CI UI.

### 5. Evidence and policy plane

Every changeset must carry machine-checkable evidence:

```yaml
EvidenceBundle:
  change_id: chg_...
  patchset_id: ps_...
  lanes:
    - lane: cargo-nextest-affected
      status: passed
      duration_ms: 412000
      logs: cas://...
      junit: cas://...
  cache:
    sccache_hit_rate: 0.83
    remote_action_cache_hit_rate: 0.76
  policy:
    blast_radius: local
    ownership: passed
    data_class: passed
    supply_chain: passed
  trace_id: otel://...
```

Submit is blocked unless required evidence exists and policy verdicts pass.

### 6. Release plane

Release should behave like Amazon’s release-captain pipeline:

- immutable artifacts;
- signed SBOM/provenance;
- environment promotion;
- one-box/cell/canary stages;
- metric-gated bake time;
- automated rollback before human paging when possible;
- deployment windows;
- release-train dashboard for TPM.

### 7. TPM/Orchestrator plane

TPM does not review routine implementation. TPM owns:

- multiproject dependency graph;
- release train sequencing;
- capacity/rate limits;
- risk register;
- cross-pod conflicts;
- high-blast-radius approvals;
- incident response and rollback coordination;
- roadmap tradeoffs.

TPM output is orchestration state, not code diff commentary.

---

## Exact delivery plan

### Phase 0 — Baseline and proof harness

**Goal:** prove current bottlenecks and create measurement baseline.

Deliverables:

- `delivery.metrics.baseline.v1`: current lead time, CI duration, queue time, flake rate, cache hit rate, rework rate.
- `delivery.trace.v1`: trace schema for every run.
- Shadow run of current GitHub Actions lanes through local/repoctl runner where possible.

Exit criteria:

- Baseline report exists.
- At least 10 recent changes replayed or simulated.
- Bottleneck decomposition: VCS wait, test wait, review wait, deploy wait.

### Phase 1 — Rust lane engine, still Git-compatible

**Goal:** make Rust CI fast and deterministic before changing VCS semantics.

Deliverables:

- `repoctl ci plan`: computes affected Rust crates and required lanes.
- `repoctl ci run`: executes lane graph locally/worker-side.
- nextest partitioning/JUnit/flaky-result support.
- sccache metrics ingestion.
- lane budgets and owner metadata.

Exit criteria:

- affected-graph lane P95 ≤ 10 minutes;
- fast check lane P95 ≤ 3 minutes for local-scope changes;
- cache hit-rate reported for every run;
- GitHub Actions can be marked compatibility-only for Rust lanes.

### Phase 2 — Change Graph v0

**Goal:** agents stop coordinating primarily through branches.

Deliverables:

- `ChangeSet`, `PatchSet`, `Stack`, `EvidenceBundle` schemas.
- Git import/export adapter.
- `jj` workspace adapter for local agent operations.
- conflict detection and stack rebase simulation.
- evidence attached to patchset, not PR comment.

Exit criteria:

- 20 trial changes represented in Change Graph;
- at least 5 stacked changes handled without branch-name coordination;
- GitHub PR export remains possible;
- reverting an agent attempt is graph-level, not manual git surgery.

### Phase 3 — Agent Pod autonomy pilot

**Goal:** one low-risk pod can ship without TPM/human micromanagement.

Deliverables:

- pod manifest schema;
- ownership/path/capability mapping;
- autonomy-tier submit requirements;
- policy-controlled merge/apply job;
- audit trail per pod action.

Exit criteria:

- one pod ships low-risk doc/tooling/Rust-internal changes through evidence gates;
- no raw direct push from agent runtime;
- rollback path tested;
- TPM receives dashboard only, not approval queue, for low-risk changes.

### Phase 4 — Own CI/CD substrate

**Goal:** GitHub Actions is no longer core CI/CD.

Deliverables:

- durable workflow orchestrator;
- Kubernetes runner pools;
- artifact/CAS/evidence store;
- lane scheduler with resource quotas;
- GitHub status mirror adapter;
- OpenTelemetry trace export.

Exit criteria:

- >80% per-PR lanes execute on Oyatie substrate;
- GitHub Actions only mirrors statuses / compatibility checks;
- failed run can be replayed from trace + artifact refs;
- all CI writes happen through gated service identity, not agent pod credentials.

### Phase 5 — Release-captain automation

**Goal:** release pipeline handles normal promotion/rollback.

Deliverables:

- release artifact immutability;
- signed provenance/SBOM;
- canary/wave/bake policy;
- automatic rollback gate;
- release train dashboard.

Exit criteria:

- one non-critical service/component releases via automated promotion;
- rollback drill passes;
- on-call alert happens after automatic rollback attempt for eligible failures.

### Phase 6 — Scale and VCS evolution decision

**Goal:** decide whether to remain Git-compatible or move storage further.

Evaluate:

- Change Graph + Git export enough?
- Need Gerrit backend for patchset review?
- Need Sapling-like repo virtualization?
- Need Buck2/Bazel full migration for Rust monorepo scale?

Exit criteria:

- decision ADR with measured evidence;
- no speculative VCS rewrite without bottleneck proof.

---

## PRD outline

### Product

Agentic Delivery Fabric: autonomous pod-oriented CI/CD + changeset/VCS substrate.

### Users

- autonomous agent pods;
- human engineers;
- TPM/Orchestrator;
- SRE/release owners;
- security/compliance reviewers.

### Jobs to be done

- pod can take a bounded task from issue/spec to shipped artifact;
- TPM can see and resolve cross-project dependency/risk;
- CI can prove readiness with minimal latency;
- release can promote/rollback safely without manual shepherding;
- VCS can represent parallel and stacked work natively.

### Non-goals

- immediate full replacement of Git storage;
- immediate migration away from GitHub as remote mirror;
- generic language CI before Rust-first excellence;
- broad production auto-deploy without canary/rollback proof.

### Success metrics

- small Rust change lead time: ≤ 2 hours P50, ≤ 1 day P95;
- affected CI P95: ≤ 10 minutes;
- fast check P95: ≤ 3 minutes;
- cache hit-rate: ≥ 80% trusted CI writes;
- flaky test rate: < 0.5% blocking lanes;
- false-green critical incidents: 0 tolerated;
- rollback drill: ≤ 5 minutes to safe state for eligible services;
- agent work collision rate: decreasing month-over-month;
- TPM intervention rate: only high-blast/cross-project branches.

---

## Test specification outline

### Unit/schema tests

- ChangeSet/PatchSet/Stack schema validation.
- Pod manifest ownership and autonomy rules.
- Lane budget and required-evidence resolution.
- Affected-graph crate selection.

### Integration tests

- Git import/export round trip.
- `jj` workspace create/update/rebase/export trial.
- stacked changes with parent dependency.
- Rust lane execution with nextest JUnit + flaky handling.
- sccache metrics ingestion.
- policy fail-closed for missing evidence.

### End-to-end tests

- agent pod creates low-risk changeset, runs lanes, passes policy, exports PR/status.
- stacked two-change flow with downstream revalidation.
- failed lane produces replayable evidence bundle.
- release candidate promotes through canary and auto-rolls back on synthetic SLO burn.

### Observability tests

- every run emits trace id;
- traces include model/tool/build/policy/cache/deploy spans;
- incident artifact can be generated from trace id.

### Security tests

- agent runtime has no production secret;
- egress blocked except allowlist;
- cache writes denied from untrusted pod;
- high-blast change requires TPM/Orchestrator gate;
- plugin/tool metadata treated as untrusted input.

---

## ADR decision draft

### Decision

Adopt an Oyatie-owned Agentic Delivery Fabric: Rust-first CI lane engine, internal Change Graph, autonomous pod manifests, durable workflow orchestration, and release-captain automation. Git/GitHub remain compatibility/mirror surfaces until measured evidence justifies replacing storage or review backend.

### Drivers

- Git branch/PR semantics do not represent high-concurrency autonomous agent delivery well.
- GitHub Actions is not the right core substrate for hyperscaler-grade Rust CI/CD velocity.
- Autonomous pods need evidence-gated shipping authority.
- TPM/Orchestrator should coordinate dependencies and risks, not routine implementation.

### Alternatives rejected

- Keep GitHub Actions as core: insufficient for autonomy, evidence graph, and large-scale queue control.
- Replace Git immediately: too risky without measured bottleneck proof.
- Adopt Gerrit wholesale immediately: strong semantics, but cultural/UX migration cost; borrow semantics first.
- Adopt Sapling stack immediately: excellent reference, not turnkey enough for v0.

### Consequences

Positive:

- higher agent throughput;
- more reliable stacked/parallel work;
- better Rust CI latency;
- better auditability and replay;
- clearer TPM role.

Negative:

- new control plane to build and operate;
- possible overengineering if metrics are ignored;
- adapter complexity with GitHub/Git;
- strict schema/process discipline required.

---

## Guardrails

1. Agent pods do not hold production secrets.
2. Direct writes to trunk/prod happen only through gated service identities.
3. Cache writes only from trusted CI workers.
4. Every changeset has evidence, policy verdict, and trace id.
5. High-blast changes require TPM/Orchestrator gate.
6. Release rollout is progressive and rollback-capable.
7. No custom VCS storage migration without measured bottleneck evidence.
8. GitHub Actions is retired gradually; status compatibility remains until consumers are migrated.

---

## Recommended follow-up staffing

### `$ralph`

Use for single-owner follow-up to turn this report into canonical ADR/PRD/test-spec docs.

### `$team`

Use when implementation begins. Suggested lanes:

- **VCS/Change Graph executor:** schemas, Git adapter, `jj` adapter.
- **Rust CI executor:** affected graph, nextest, sccache metrics, lane runner.
- **Control-plane executor:** durable workflow runner, evidence store, trace model.
- **Security/release executor:** pod sandbox policy, release-captain pipeline, rollback drills.
- **Verifier:** end-to-end shadow runs and acceptance evidence.

### `$ultragoal`

Use after report approval to track durable staged delivery across phases.

### `$autoresearch-goal`

Use only for deeper benchmark research: Gerrit vs jj vs Sapling, Buck2 vs Bazel for Rust, Temporal vs Tekton/Argo for control plane.

---

## Source index

- GitHub Actions limits: https://docs.github.com/en/actions/reference/limits
- Jujutsu Git compatibility: https://jj-vcs.github.io/jj/latest/git-compatibility/
- Jujutsu operation log: https://jj-vcs.github.io/jj/latest/operation-log/
- Gerrit changes / Change-Id: https://gerrit-review.googlesource.com/Documentation/concept-changes.html
- Gerrit submit requirements: https://gerrit-review.googlesource.com/Documentation/config-submit-requirements.html
- Gerrit cross-repo changes: https://gerrit-review.googlesource.com/Documentation/cross-repository-changes.html
- Sapling introduction: https://sapling-scm.com/docs/introduction/
- Sapling scale: https://sapling-scm.com/docs/scale/overview/
- Buck2 remote execution: https://buck2.build/docs/users/remote_execution/
- Bazel remote caching: https://bazel.build/remote/caching
- Bazel remote execution: https://bazel.build/docs/remote-execution
- Temporal workflows: https://docs.temporal.io/workflows
- cargo-nextest partitioning: https://nexte.st/docs/ci-features/partitioning/
- cargo-nextest retries/flaky tests: https://nexte.st/docs/features/retries/
- sccache Rust docs: https://github.com/mozilla/sccache/blob/main/docs/Rust.md
- Cargo build cache: https://doc.rust-lang.org/stable/cargo/reference/build-cache.html
- Amazon continuous delivery: https://aws.amazon.com/builders-library/going-faster-with-continuous-delivery/
- Amazon hands-off deployments: https://aws.amazon.com/builders-library/automating-safe-hands-off-deployments/
- Amazon release captain: https://aws.amazon.com/builders-library/cicd-pipeline/
- Microsoft BuildXL: https://github.com/microsoft/BuildXL
- Microsoft Build Accelerator: https://devblogs.microsoft.com/engineering-at-microsoft/large-scale-distributed-builds-with-microsoft-build-accelerator/
