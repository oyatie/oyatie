# Executable PRD + Implementation Spec: Agentic Delivery Fabric

**Date:** 2026-05-10
**Status:** Actionable planning artifact; no implementation started.
**Owner target:** `axis-foundry` + `ops-sre-reliability` + future `agentic-delivery` platform pod.
**Problem:** Git/GitHub Actions are workflow bottlenecks for autonomous agent pods and Rust-first high-velocity delivery.
**Decision direction:** Build an Oyatie-owned Agentic Delivery Fabric: internal Change Graph + Rust-first CI lane engine + autonomous pod runtime + release-captain CD + Git/GitHub compatibility adapters.

---

## 0. One-page execution brief

### Target result

Create a production-grade delivery substrate where autonomous agent pods can ship bounded changes at high velocity with evidence-gated autonomy, while TPM/Orchestrator only coordinates multiproject dependencies, risk, sequencing, and incidents.

### Product thesis

- **Pod = autonomous agent delivery unit.** It owns paths/capabilities, makes changes, runs evidence, and ships inside its authority.
- **TPM/Orchestrator = dependency/risk coordinator.** It does not micromanage implementation or routine approvals.
- **CI/CD = delivery operating system.** It owns execution, evidence, provenance, rollback, replay, policy, and release promotion.
- **VCS = Change Graph first, Git-compatible second.** Git remains import/export/storage compatibility; agent workflow truth is a durable changeset graph that supports stacked and parallel work.

### MVP scope

Build a local-first then cluster-ready system around existing `repoctl`/agent-daemon concepts:

1. Rust CI lane planner/runner with affected-crate graph, `cargo nextest`, `sccache` metrics, and evidence bundles.
2. Change Graph v0 schemas for changesets, patchsets, stacks, evidence, policy verdicts, and Git export refs.
3. Agent Pod manifest with ownership and autonomy policy.
4. Submission gate that merges/exports only when required evidence exists.
5. GitHub Actions compatibility/status mirror, not core execution.

### Non-goals for v0

- Full replacement of Git object storage.
- Immediate migration to Sapling/Mononoke/Gerrit as primary platform.
- Production auto-deploy for high-risk services.
- Generic multi-language CI before Rust-first excellence.
- Hosted SaaS CI dependency as the core substrate.

### Success metrics

| Metric | Target |
|---|---:|
| Fast Rust check lane P95 | <= 3 min |
| Affected Rust verification P95 | <= 10 min |
| Full workspace nightly P95 | <= 60 min initially |
| Trusted compile cache hit rate | >= 80% after warmup |
| Low-risk pod TPM approvals | 0 required |
| Missing evidence submit failures | 100% fail closed |
| Revert/rollback drill | <= 5 min for eligible low-risk artifact |
| Agent workspace collision rate | < 1% of runs |
| Every changeset has trace/evidence | 100% |

---

## 1. Product requirements

### 1.1 Personas

#### Autonomous Agent Pod

A bounded autonomous delivery actor. Needs:

- clear ownership scope;
- reproducible workspace;
- fast Rust feedback;
- durable changeset identity;
- stacked/parallel work support;
- ability to ship low-risk changes after gates pass;
- no need for routine TPM permission.

#### TPM / Orchestrator

Coordinates multiproject delivery. Needs:

- dependency graph across pods/projects;
- risk dashboard;
- blocked stack visibility;
- release train readiness;
- escalation queue only for material risk;
- no routine code-review workload.

#### Platform/SRE

Operates delivery substrate. Needs:

- runner health;
- queue health;
- cache health;
- artifact retention;
- supply-chain provenance;
- rollback/replay;
- incident artifact generation.

#### Security/Compliance

Ensures safe autonomy. Needs:

- least-privilege pod identities;
- no secrets in agent runtime;
- policy-as-code gates;
- audit trail;
- evidence retention;
- provenance and SBOM.

### 1.2 Functional requirements

#### FR-1 Change Graph

The system MUST represent work as logical changesets with immutable patchsets and explicit stack edges.

Acceptance:

- Create a changeset without creating a Git branch.
- Add multiple patchsets to one changeset.
- Represent parent/child changes in a stack.
- Attach evidence and policy verdicts to patchsets.
- Export a changeset to Git branch and/or GitHub PR for compatibility.

#### FR-2 Rust CI lane engine

The system MUST plan and execute Rust lanes using affected-crate analysis.

Acceptance:

- Detect touched crates from diff/path graph.
- Include reverse transitive dependents.
- Emit required lanes by blast radius.
- Run `cargo fmt`, `cargo check`, `cargo clippy`, `cargo nextest`, and security/dependency gates.
- Produce machine-readable lane results and JUnit where applicable.

#### FR-3 Evidence bundle

The system MUST generate a durable evidence bundle for every patchset run.

Acceptance:

- Includes lane statuses, logs, artifacts, cache metrics, trace id, policy verdicts, and reproduction command.
- Submit gate refuses missing/expired evidence.
- Evidence is content-addressed or immutable by ID.

#### FR-4 Agent Pod manifest

The system MUST define pod ownership, autonomy, and escalation rules in a typed manifest.

Acceptance:

- A pod can declare owned paths, crates, capabilities, and autonomy tier.
- A pod can merge/export low-risk owned changes if gates pass.
- Cross-project/high-risk/root-manifest/security changes require TPM/Orchestrator gate.

#### FR-5 Submission gate

The system MUST gate integration/export by evidence and policy, not by prompt compliance.

Acceptance:

- Refuses if required lanes missing/failing/stale.
- Refuses if ownership does not cover touched paths.
- Refuses if blast radius exceeds pod autonomy.
- Refuses if stack parent not integrated or not included in atomic submit group.
- Emits actionable failure reason.

#### FR-6 Git/GitHub compatibility

The system MUST preserve Git/GitHub compatibility during migration.

Acceptance:

- Import existing Git commits/branches into Change Graph.
- Export changeset to Git branch.
- Optionally open/update GitHub PR.
- Mirror statuses from Oyatie lanes to GitHub checks.
- Do not require GitHub Actions for core execution.

#### FR-7 Release-captain CD

The system MUST support progressive release promotion and rollback for eligible artifacts.

Acceptance:

- Immutable artifact generated after green submit.
- SBOM/provenance attached.
- Canary/wave/bake policy configurable.
- Automatic rollback on metric gate failure.
- TPM sees release train state, not manual step checklist.

### 1.3 Non-functional requirements

| Area | Requirement |
|---|---|
| Reliability | Lane results and changeset state must be durable and replayable. |
| Security | Agent runtime cannot hold production deploy secrets. |
| Performance | Rust affected verification P95 <= 10m after warm cache. |
| Scalability | Scheduler supports N parallel pods with path/stack conflict detection. |
| Auditability | Every submit/release has trace/evidence/provenance. |
| Reversibility | Every low-risk submit has graph-level revert or patchset rollback plan. |
| Compatibility | Git export remains available through migration. |

---

## 2. System architecture

### 2.1 Planes

```text
+--------------------------------------------------------------------------------+
| TPM / Orchestrator Plane                                                       |
| dependencies, risk register, release train, cross-pod conflicts, incidents     |
+--------------------------------------------------------------------------------+
| Release Plane                                                                  |
| immutable artifacts, SBOM/provenance, canary/waves/bake, rollback              |
+--------------------------------------------------------------------------------+
| Evidence + Policy Plane                                                        |
| lane results, traces, cache metrics, ownership, blast radius, submit gates      |
+--------------------------------------------------------------------------------+
| Rust CI Lane Engine                                                            |
| affected graph, cargo fmt/check/clippy/nextest/deny/machete/audit/bench        |
+--------------------------------------------------------------------------------+
| Change Graph Plane                                                             |
| changesets, patchsets, stacks, conflict state, Git/GitHub/jj adapters          |
+--------------------------------------------------------------------------------+
| Agent Pod Runtime                                                              |
| ephemeral workspaces, sandbox, least privilege, no production secrets           |
+--------------------------------------------------------------------------------+
```

### 2.2 Component map

| Component | Purpose | Initial implementation surface |
|---|---|---|
| `repoctl change` | Change Graph CLI | new `repoctl` command group |
| `repoctl ci` | Rust lane planner/runner | new/extended `repoctl` command group |
| `repoctl pod` | Pod manifest validation/status | new command group |
| `repoctl submit` | evidence/policy-gated integration/export | new command group |
| `deliveryd` | durable scheduler/controller | later service; v0 can be file-backed/local |
| `agent-daemon` adapter | launches pod runs | extend existing daemon plan |
| Evidence store | immutable bundles/log refs | local `.omx` first, object store later |
| Git adapter | import/export refs | Git CLI/libgit2/gitoxide later |
| jj adapter | local stacked workspace ergonomics | shell adapter first |
| GitHub adapter | PR/status mirror | optional compatibility layer |

### 2.3 Data flow

```text
Issue/spec/task
  -> TPM assigns or pod claims if in ownership scope
  -> ChangeSet created
  -> Agent pod gets ephemeral workspace
  -> Agent produces PatchSet
  -> Rust CI lane planner computes required lanes
  -> Lane engine runs affected graph
  -> EvidenceBundle emitted
  -> Policy engine evaluates autonomy/blast/evidence/stack
  -> Submit gate exports/merges OR blocks with reason
  -> Release plane promotes artifact if configured
  -> Trace/evidence retained for replay and audit
```

---

## 3. Data model specification

### 3.1 ChangeSet

```yaml
schema: oya.delivery.change-set.v1
id: chg_01HX...
title: "short user/business intent"
description: "why this exists"
state: draft|ready|blocked|submitted|abandoned|reverted
owner_pod: pod_foundry_repoctl
author_actor:
  kind: agent|human|system
  id: codex_run_...
base:
  vcs: git
  repo: bominal
  commit: <sha>
stack:
  stack_id: stk_01HX...
  parents: [chg_parent]
  children: [chg_child]
blast_radius: docs|local|axis_substrate|cohesion_substrate|root_manifest|infra|security|release
paths_touched: []
crates_touched: []
patchsets: []
evidence_required: []
evidence_latest: []
policy_verdict_latest: null
export_refs:
  git_branch: null
  github_pr: null
  gerrit_change: null
created_at: "..."
updated_at: "..."
```

### 3.2 PatchSet

```yaml
schema: oya.delivery.patch-set.v1
id: ps_01HX...
change_id: chg_01HX...
sequence: 3
base_commit: <sha>
diff_ref: cas://diff/...
workspace_ref: workspace://...
created_by: agent_run_...
summary: "what changed"
lineage:
  previous_patchset: ps_...
  supersedes: []
conflicts:
  status: none|detected|resolved
  files: []
evidence_refs: []
trace_id: trace_...
```

### 3.3 Stack

```yaml
schema: oya.delivery.stack.v1
id: stk_01HX...
title: "multi-change feature or dependency chain"
changes:
  - chg_a
  - chg_b
submit_mode: independent|ordered|atomic
status: open|blocked|submitting|submitted|abandoned
risk_summary: "..."
tpm_required: true|false
```

### 3.4 EvidenceBundle

```yaml
schema: oya.delivery.evidence-bundle.v1
id: evb_01HX...
change_id: chg_...
patchset_id: ps_...
created_at: "..."
valid_until: "..."
trace_id: trace_...
reproduction:
  command: "repoctl ci run --change chg_... --patchset ps_..."
  image: "oci://..."
  toolchain: "rustc 1.xx.x"
lanes:
  - id: cargo-fmt
    status: passed|failed|skipped|timed_out
    duration_ms: 1234
    log_ref: cas://logs/...
    artifact_refs: []
cache:
  sccache_hit_rate: 0.82
  remote_action_cache_hit_rate: 0.76
policy_inputs:
  blast_radius: local
  ownership_scope: pod_foundry_repoctl
  touched_paths_hash: sha256:...
```

### 3.5 PolicyVerdict

```yaml
schema: oya.delivery.policy-verdict.v1
id: pv_01HX...
change_id: chg_...
patchset_id: ps_...
status: allow|deny|needs_tpm|needs_human|needs_security
reasons:
  - code: missing_lane|failed_lane|stale_evidence|ownership_gap|blast_radius_exceeded|stack_parent_blocked
    message: "human actionable explanation"
required_actions: []
created_at: "..."
```

### 3.6 AgentPodManifest

```yaml
schema: oya.delivery.agent-pod.v1
id: pod_foundry_repoctl
name: "Foundry Repoctl Pod"
owners:
  primary: axis-foundry
  tpm: platform-orchestrator
scope:
  paths:
    - tools/repoctl/**
    - services/agent/daemon/**
  crates:
    - repoctl
  capabilities:
    - repoctl.*
autonomy:
  max_tier: T3
  can_create_changes: true
  can_submit_if:
    - blast_radius in [docs, local]
    - all_required_lanes_green
    - ownership_coverage == full
    - no_cross_project_dependency
    - no_open_sev1_or_sev2
  tpm_required_if:
    - blast_radius in [axis_substrate, cohesion_substrate, root_manifest, infra, security, release]
    - touches_public_api
    - touches_root_manifest
    - cross_project_dependency
runtime:
  sandbox: restricted
  network: allowlist
  secrets: none
  max_parallel_runs: 4
```

### 3.7 LaneDefinition

```yaml
schema: oya.delivery.lane.v1
id: cargo-nextest-affected
kind: rust_test
owner: axis-foundry
command_template: "cargo nextest run --workspace --all-features {filter_args}"
timeout_ms: 600000
required_for:
  blast_radius: [local, axis_substrate, cohesion_substrate]
artifacts:
  - junit
  - nextest_archive
cache_policy:
  read: trusted_and_untrusted
  write: trusted_ci_only
retry_policy:
  flaky_result: fail
  retries: 1
```

---

## 4. CLI/API specification

### 4.1 `repoctl change`

```bash
repoctl change create \
  --title "..." \
  --owner-pod pod_foundry_repoctl \
  --base HEAD \
  --format json
```

Output: `oya.delivery.change-set.v1`.

```bash
repoctl change patchset create \
  --change chg_... \
  --workspace /path/to/workspace \
  --summary "..." \
  --format json
```

Output: `oya.delivery.patch-set.v1`.

```bash
repoctl change stack create --changes chg_a,chg_b --submit-mode ordered --format json
repoctl change status --change chg_... --format json
repoctl change export git --change chg_... --remote origin --format json
repoctl change export github-pr --change chg_... --draft --format json
```

### 4.2 `repoctl ci`

```bash
repoctl ci plan --change chg_... --patchset ps_... --format json
```

Output: planned lanes, affected crates, blast radius, estimated time.

```bash
repoctl ci run --plan ci_plan_... --format json
```

Output: `oya.delivery.evidence-bundle.v1`.

```bash
repoctl ci explain --change chg_... --why-lane cargo-nextest-affected
```

### 4.3 `repoctl pod`

```bash
repoctl pod validate --manifest product-control/pods/pod_foundry_repoctl.yaml
repoctl pod claim --pod pod_foundry_repoctl --change chg_... --format json
repoctl pod status --pod pod_foundry_repoctl --format json
```

### 4.4 `repoctl submit`

```bash
repoctl submit check --change chg_... --patchset ps_... --format json
repoctl submit apply --change chg_... --mode git-export --format json
repoctl submit stack --stack stk_... --mode ordered --format json
```

Rules:

- `check` is read-only and always safe.
- `apply` requires policy `allow` or explicit TPM approval token for gated cases.
- `stack` respects `submit_mode` and parent readiness.

### 4.5 `deliveryd` HTTP API v0

Initial API can be local-only or internal service.

```http
POST /delivery/v1/changes
GET  /delivery/v1/changes/{id}
POST /delivery/v1/changes/{id}/patchsets
POST /delivery/v1/ci/plans
POST /delivery/v1/ci/runs
POST /delivery/v1/submit/check
POST /delivery/v1/submit/apply
GET  /delivery/v1/pods/{id}/status
GET  /delivery/v1/stacks/{id}
GET  /delivery/v1/traces/{trace_id}/incident-artifact
```

---

## 5. Rust-first CI/CD technical spec

### 5.1 Affected graph algorithm

Inputs:

- changed file paths from PatchSet diff;
- Cargo workspace metadata;
- crate dependency graph;
- lane rules;
- blast radius rules.

Algorithm:

1. Map changed paths to direct crates.
2. If root manifest/toolchain/build scripts touched, mark `root_manifest` and require broad lanes.
3. Compute reverse transitive dependent crates.
4. Include integration/e2e crates by declared test ownership mapping.
5. Emit lane plan:
   - direct fast checks;
   - dependent tests;
   - full workspace only when root/shared substrate touched.
6. Cache plan by `(base_commit, patchset_hash, cargo_metadata_hash)`.

### 5.2 Required Rust lanes by blast radius

| Blast radius | Required lanes |
|---|---|
| docs | docs lint, link/check, no Rust compile unless examples touched |
| local crate | fmt, check affected, clippy affected, nextest affected, deny/machete if deps touched |
| axis substrate | fmt, workspace check, clippy affected+dependents, nextest affected+dependents, semver if public |
| cohesion substrate | workspace check/clippy/nextest, architecture boundary, policy/data-class lanes |
| root manifest | full workspace, deny, audit, machete, cargo tree diff, one-at-a-time submit |
| unsafe/concurrency | plus Miri/sanitizer/loom where configured |
| hot path | plus benchmark gate |

### 5.3 Cache design

- `sccache` for Rust compiler invocation caching.
- `CARGO_INCREMENTAL=0` in CI unless a measured lane proves otherwise.
- Cache writes allowed only from trusted CI workers.
- Agent pods may read cache but not write trusted cache by default.
- Track:
  - compile requests;
  - cache hits/misses;
  - non-cacheable reasons;
  - bytes uploaded/downloaded;
  - wall-clock saved estimate.

### 5.4 Remote execution path

Start Cargo-native. Move hot lanes to Bazel/Buck2 REAPI only when metrics show Cargo+sccache insufficient.

Decision checkpoint:

- if affected Rust P95 remains > 10m after sccache/nextest/affected graph;
- if full workspace nightly > 60m;
- if cache hit-rate < 60% due to Cargo limitations;
- then pilot Buck2/Bazel for selected crates.

---

## 6. VCS/change-management spec

### 6.1 Required semantics

The VCS layer MUST support:

- parallel independent agent work;
- stacked dependent changes;
- patchset history per logical change;
- safe undo of agent attempts;
- conflict state as explicit data;
- evidence attached to patchsets;
- ordered or atomic stack submission;
- Git-compatible export.

### 6.2 v0 storage

Use file-backed JSON/YAML under `.omx/delivery/` for MVP:

```text
.omx/delivery/
  changes/chg_*.json
  patchsets/ps_*.json
  stacks/stk_*.json
  evidence/evb_*.json
  verdicts/pv_*.json
  traces/*.json
```

Later migrate to embedded DB/service store.

### 6.3 Adapter sequence

1. Git adapter: import/export branch/commit/diff.
2. `jj` adapter: local workspace operations for agents.
3. GitHub adapter: PR/status mirror.
4. Gerrit adapter: optional if patchset review backend desired.
5. Sapling-like virtualization: future only if repo scale demands.

### 6.4 Conflict policy

- Conflict detected at patchset create or stack rebase simulation.
- Conflict blocks submit, not change creation.
- Agent may attempt conflict resolution in its workspace.
- Shared ownership conflicts route to TPM/Orchestrator if ownership scopes overlap materially.

---

## 7. Agent pod autonomy model

### 7.1 Autonomy tiers

| Tier | Authority |
|---|---|
| T0 | read/analyze only |
| T1 | create changesets/patchsets; no submit |
| T2 | submit docs/local non-prod changes with green gates |
| T3 | submit owned code changes with green gates and rollback plan |
| T4 | production deploy/rollback within declared service policy |

### 7.2 TPM/Orchestrator gate triggers

- cross-project dependency;
- root manifest/toolchain/shared CI substrate;
- infra/security/compliance/data-class boundary;
- public API or schema compatibility impact;
- stack spans multiple pods;
- release train conflict;
- incident/sev state;
- policy ambiguity.

### 7.3 Pod runtime security

- ephemeral workspace;
- no production secrets;
- egress allowlist;
- resource quotas;
- sandboxed filesystem;
- separate apply/deploy identity;
- complete trace of model/tool/shell actions;
- kill switch per pod.

---

## 8. Release-captain CD spec

### 8.1 Release object

```yaml
schema: oya.delivery.release.v1
id: rel_...
artifact_ref: oci://...
source_change: chg_...
provenance_ref: cas://...
sbom_ref: cas://...
policy:
  rollout: canary
  stages:
    - one_box
    - cell_5_percent
    - region_wave_1
    - region_wave_2
  bake_times: []
rollback:
  automatic: true
  metric_gates: []
```

### 8.2 Promotion stages

1. build immutable artifact;
2. sign/provenance/SBOM;
3. deploy one-box or local cell;
4. bake;
5. canary 5%;
6. bake;
7. wave rollout;
8. auto rollback on SLO/security/health failure;
9. mark release complete.

### 8.3 Release acceptance

- rollback drill exists;
- metric gates configured;
- no open Sev1/Sev2 affecting surface;
- evidence bundle linked;
- release trace exists.

---

## 9. Implementation work breakdown

### Milestone M0 — Baseline and scaffolding

**Objective:** establish metrics and artifact locations.

Tasks:

1. Create `.omx/delivery/` MVP store layout.
2. Define JSON schemas for ChangeSet/PatchSet/Stack/EvidenceBundle/PolicyVerdict/AgentPod/Lane.
3. Add schema validation command skeletons.
4. Capture current CI baseline from existing docs/workflows/local runs.
5. Define trace id propagation convention.

Acceptance:

- schema validator passes for sample fixtures;
- baseline report generated;
- no mutation to GitHub or production systems.

### Milestone M1 — Rust CI lane planner

**Objective:** plan required Rust lanes without executing them.

Tasks:

1. Parse Cargo metadata.
2. Map changed files to crates.
3. Compute reverse dependencies.
4. Classify blast radius.
5. Emit `ci-plan.v1`.
6. Add tests for path/crate mapping and root manifest behavior.

Acceptance:

- local crate change produces affected-only plan;
- root manifest change produces broad plan;
- docs-only change avoids Rust lanes unless code examples affected.

### Milestone M2 — Rust CI runner + evidence

**Objective:** execute lane plan and emit evidence.

Tasks:

1. Implement lane runner for fmt/check/clippy/nextest.
2. Add nextest JUnit/archive support.
3. Collect sccache stats where configured.
4. Emit EvidenceBundle.
5. Store logs/artifacts immutably in local MVP store.

Acceptance:

- failing lane blocks evidence status;
- nextest JUnit captured;
- cache metrics present or explicit `not_configured` reason.

### Milestone M3 — Change Graph v0

**Objective:** create and update changesets/patchsets independent of branches.

Tasks:

1. Implement `repoctl change create`.
2. Implement `repoctl change patchset create` from workspace diff.
3. Implement stack create/status.
4. Attach evidence to patchset.
5. Add graph-level status calculation.

Acceptance:

- multiple patchsets on one change;
- stack with parent/child represented;
- abandoned patchset does not destroy change lineage.

### Milestone M4 — Git + jj adapters

**Objective:** preserve compatibility and improve agent local workflow.

Tasks:

1. Git import/export branch adapter.
2. GitHub PR/status mirror dry-run adapter.
3. `jj` workspace experiment adapter.
4. Stack rebase simulation.
5. Conflict state recording.

Acceptance:

- changeset exports to Git branch;
- status mirror payload generated without GitHub mutation by default;
- jj trial can create/update a stacked local workspace;
- conflicts are recorded as state.

### Milestone M5 — Pod manifest + policy gate

**Objective:** enforce scoped autonomy.

Tasks:

1. Define pod manifest schema.
2. Validate ownership coverage.
3. Implement blast-radius/autonomy evaluator.
4. Implement `repoctl submit check`.
5. Implement policy verdict reasons.

Acceptance:

- owned low-risk change allowed when evidence green;
- cross-scope change returns `needs_tpm`;
- missing evidence denies submit.

### Milestone M6 — Submit/export apply path

**Objective:** apply/export through gated service path.

Tasks:

1. Implement `repoctl submit apply --mode git-export`.
2. Add protected direct-write checks.
3. Add stack ordered submit.
4. Add rollback/revert metadata.
5. Add audit event output.

Acceptance:

- no raw direct push from agent workspace;
- apply requires allow verdict;
- stack submit respects parent order;
- revert instructions emitted.

### Milestone M7 — Cluster runner pilot

**Objective:** move execution off local/GitHub Actions core.

Tasks:

1. Define runner pod spec.
2. Add queue abstraction.
3. Add artifact store abstraction.
4. Add OpenTelemetry spans.
5. Run Rust lanes in ephemeral worker.

Acceptance:

- at least one Rust CI plan runs in ephemeral runner;
- evidence stored outside workspace;
- GitHub status can be mirrored from Oyatie result.

### Milestone M8 — Release-captain pilot

**Objective:** progressive release/rollback for one low-risk artifact.

Tasks:

1. Define release object.
2. Attach artifact/provenance/SBOM refs.
3. Configure canary/bake stages.
4. Add synthetic metric gate.
5. Run rollback drill.

Acceptance:

- release trace generated;
- automatic rollback path demonstrated;
- TPM dashboard shows state without manual checklist.

---

## 10. Test plan

### 10.1 Unit tests

- schema validation for all v1 objects;
- path-to-crate mapping;
- reverse dependency graph;
- blast radius classifier;
- autonomy evaluator;
- evidence freshness calculation;
- stack order validation;
- policy verdict reason generation.

### 10.2 Integration tests

- ChangeSet -> PatchSet -> CI plan -> EvidenceBundle -> SubmitCheck.
- docs-only change does not run Rust lanes.
- root manifest change requires full workspace and one-at-a-time submit.
- failed clippy blocks submit.
- stale evidence blocks submit.
- `jj` adapter trial does not corrupt Git export.
- Git branch export round trip.

### 10.3 End-to-end tests

- Low-risk pod-owned Rust change reaches `allow` verdict.
- Cross-pod stack reaches `needs_tpm` verdict.
- Two-change stack submits in order.
- Conflict blocks submit and records conflict files.
- Runner failure emits replayable incident artifact.
- Release canary rollback drill passes.

### 10.4 Security tests

- agent pod cannot access production secret mount;
- untrusted pod cannot write trusted cache;
- egress denied by default;
- high-blast submit requires TPM token;
- GitHub mutation disabled unless explicit adapter credentials present.

### 10.5 Performance tests

- affected plan generation under 5s for current repo;
- fast Rust check P95 <= 3m after warm cache;
- affected nextest P95 <= 10m;
- cache hit-rate surfaced in every run.

---

## 11. Migration plan

### Stage 1 — Shadow mode

- Current Git/GitHub Actions continue.
- Delivery Fabric plans/runs in parallel and compares results.
- No authoritative submit.

### Stage 2 — Advisory mode

- Fabric results appear as required local evidence for selected pods.
- GitHub Actions still blocks final merge.

### Stage 3 — Authoritative CI mode

- Fabric Rust lanes become source of truth.
- GitHub Actions mirrors status and runs compatibility smoke only.

### Stage 4 — Agent submit mode

- Low-risk pod-owned changes can submit/export after policy allow.
- TPM sees dashboard; no routine approval.

### Stage 5 — Release-captain mode

- Eligible artifacts release through automated canary/wave/rollback.
- Manual release captain role retired for those surfaces.

---

## 12. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Rebuilding too much platform | File-backed MVP; Git-compatible adapters; measure before replacing storage. |
| Cache poisoning | trusted writes only; provenance; worker identity; CAS verification. |
| Agent over-autonomy | scoped pod manifests; blast radius; evidence gates; TPM triggers. |
| Slow Rust CI remains slow | affected graph, nextest sharding, sccache metrics, then Buck2/Bazel pilot. |
| Stack complexity | start with ordered stacks; defer atomic cross-repo submit. |
| GitHub drift | status mirror adapter and import/export tests. |
| Security leakage | no prod secrets in pods; egress allowlist; separate apply identity. |
| Cultural confusion | TPM role definition and dashboard-first escalation. |

---

## 13. Exact first 10 implementation issues

1. **ADF-001:** Add `oya.delivery.*.v1` schemas and sample fixtures.
2. **ADF-002:** Add `repoctl delivery schema validate` for fixtures and local store.
3. **ADF-003:** Implement Rust affected graph planner from Cargo metadata.
4. **ADF-004:** Implement blast-radius classifier for docs/local/root/infra/security.
5. **ADF-005:** Implement `repoctl ci plan` JSON output.
6. **ADF-006:** Implement `repoctl ci run` for fmt/check/clippy/nextest with evidence output.
7. **ADF-007:** Implement ChangeSet/PatchSet local store and `repoctl change create/patchset create`.
8. **ADF-008:** Implement EvidenceBundle attachment and `repoctl submit check` fail-closed policy.
9. **ADF-009:** Implement AgentPod manifest validation and ownership coverage.
10. **ADF-010:** Implement Git export dry-run and GitHub status mirror payload generation.

Each issue is independently testable and does not require production deployment.

---

## 14. Source anchors

- Existing repo docs: `docs/consolidated/decisions/ADR-0050-automation-first-pipeline.md`, `docs/consolidated/decisions/ADR-0041-gitops-trunk-based-and-release-branch-cut-at-tag.md`, `docs/consolidated/standards/ci-lanes.md`, `docs/consolidated/RELEASE-MANAGEMENT.md`.
- GitHub Actions limits: https://docs.github.com/en/actions/reference/limits
- Jujutsu Git compatibility: https://jj-vcs.github.io/jj/latest/git-compatibility/
- Jujutsu operation log: https://jj-vcs.github.io/jj/latest/operation-log/
- Gerrit changes: https://gerrit-review.googlesource.com/Documentation/concept-changes.html
- Gerrit submit requirements: https://gerrit-review.googlesource.com/Documentation/config-submit-requirements.html
- Sapling intro: https://sapling-scm.com/docs/introduction/
- Sapling scale: https://sapling-scm.com/docs/scale/overview/
- Buck2 remote execution: https://buck2.build/docs/users/remote_execution/
- Bazel remote cache/execution: https://bazel.build/remote/caching, https://bazel.build/docs/remote-execution
- cargo-nextest partitioning/flaky: https://nexte.st/docs/ci-features/partitioning/, https://nexte.st/docs/features/retries/
- sccache Rust docs: https://github.com/mozilla/sccache/blob/main/docs/Rust.md
- Cargo build cache: https://doc.rust-lang.org/stable/cargo/reference/build-cache.html
- Amazon continuous delivery: https://aws.amazon.com/builders-library/going-faster-with-continuous-delivery/
- Amazon safe deployments: https://aws.amazon.com/builders-library/automating-safe-hands-off-deployments/
- Amazon release captain: https://aws.amazon.com/builders-library/cicd-pipeline/
- Microsoft BuildXL: https://github.com/microsoft/BuildXL
- Microsoft Build Accelerator: https://devblogs.microsoft.com/engineering-at-microsoft/large-scale-distributed-builds-with-microsoft-build-accelerator/
---

## 15. Architecture review checklist

Before implementation handoff, the plan MUST pass these review questions:

1. **Is the Change Graph smaller than a VCS rewrite?** v0 must remain a workflow/evidence graph over Git-compatible storage; it must not become a new object database until Phase 6 evidence says Git storage is the bottleneck.
2. **Is Rust latency addressed before orchestration complexity?** M1-M2 must land before cluster runners or release-captain work.
3. **Can a pod ship without hidden privilege?** Agent pods may produce patchsets and evidence; apply/deploy identities remain separate and policy-gated.
4. **Does TPM stay out of routine implementation?** TPM gates are limited to cross-project, high-blast, root, infra, security, public API, release train, or incident conditions.
5. **Is every gate testable?** Each policy denial must name missing evidence, ownership gap, stale lane, failed lane, stack blocker, or autonomy breach.
6. **Can the system be abandoned safely?** Git export and existing workflows remain available until the Fabric is authoritative by evidence.

### Steelman antithesis to this plan

The strongest opposing view is that building a custom delivery substrate before exhausting GitHub/Git optimization risks recreating Gerrit, Buildkite, and parts of Bazel badly. The plan could slow product work if it becomes platform-first rather than bottleneck-first. The mitigation is to keep the first milestones file-backed, Git-compatible, Rust-lane-focused, and measured. No storage replacement, cluster scheduler, or release automation becomes mandatory until baseline metrics prove the preceding layer is useful.

### Critical tradeoff

The central tradeoff is **semantic control vs. platform surface area**. A Change Graph gives agent-native stacked work, evidence, autonomy, and replay. But every new semantic object creates operations burden. Therefore v0 must define schemas and gates first, implement only the minimum CLI surface needed to prove Rust CI and stacked changes, and delay service/database/distributed-runner complexity until after shadow-mode evidence.


---

## 16. Consistency and submit protocol

This section is mandatory before implementation. The Change Graph v0 is a **metadata/control plane over Git-compatible protected history**, not a replacement object database.

### 16.1 Store consistency model

v0 local store MUST be append-only at the event layer.

```yaml
schema: oya.delivery.event.v1
id: evt_01HX...
stream: change/chg_... | stack/stk_... | submit/sub_...
sequence: 42
idempotency_key: "actor:operation:stable-input-hash"
actor: agent_run_... | human_... | system_...
event_type: ChangeCreated|PatchSetCreated|EvidenceAttached|PolicyEvaluated|LeaseAcquired|SubmitStarted|SubmitApplied|SubmitFailed|LeaseReleased
object_ref: chg_... | ps_... | evb_... | pv_...
object_hash: sha256:...
prev_event_hash: sha256:...
created_at: "..."
```

Rules:

1. Materialized `ChangeSet`, `Stack`, and `EvidenceBundle` JSON files are projections. The event log is source of truth.
2. Every append uses compare-and-swap on `(stream, expected_sequence, prev_event_hash)`.
3. Every mutating command requires an idempotency key. Retrying the same command returns the original event/result.
4. Patchset sequence is monotonic per `change_id`; duplicate sequence writes are rejected unless idempotency key matches the original write.
5. Corrupted projection can be rebuilt from events.
6. Corrupted event hash chain blocks submit until repaired by an explicit recovery command.

### 16.2 Lease model

```yaml
schema: oya.delivery.lease.v1
id: lease_...
scope: change/chg_... | stack/stk_... | root-manifest | release/rel_...
holder: pod_... | submit_worker_...
expires_at: "..."
fencing_token: 123
state: active|released|expired|stolen_after_expiry
```

Rules:

- Patchset creation requires a per-change lease.
- Stack submit requires a per-stack lease.
- Root manifest/toolchain submit requires global `root-manifest` lease and one-at-a-time queue.
- Apply/export requires a fresh fencing token; stale token writes are rejected.
- Expired leases do not grant submit authority; recovery must record `LeaseExpired` and acquire a new token.

### 16.3 Submit transaction

Submit is a state machine, not a shell script.

```text
READY
  -> acquire submit lease
  -> freeze patchset IDs and evidence IDs
  -> re-evaluate policy on frozen inputs
  -> rebase/simulate stack on current protected head
  -> run required merge-queue lane if protected head changed
  -> apply/export through protected identity
  -> record exported Git refs / PR / commit SHA
  -> release lease
  -> SUBMITTED
```

Failure states:

- `blocked_policy`
- `blocked_stack_parent`
- `blocked_conflict`
- `blocked_stale_evidence`
- `failed_apply`
- `failed_status_mirror`
- `recovered_idempotent_duplicate`

All failure states MUST emit `PolicyVerdict` or `SubmitFailed` with next action.

### 16.4 Stack submit semantics

| Mode | Semantics | v0 support |
|---|---|---|
| independent | each change can submit alone after parents integrated | yes |
| ordered | submit parent-to-child with revalidation after each apply | yes |
| atomic | all changes commit together or none do | not v0 except single-repo squash group after explicit design |

v0 MUST implement `ordered`; v0 MUST reject `atomic` unless a backend-specific transaction is available and tested.

### 16.5 Protected integration invariant

Until Phase 6 decides otherwise:

- Git protected `main` remains authoritative integration history.
- Change Graph is authoritative workflow/evidence history.
- Git export commits MUST include trailers:

```text
Change-Id: chg_...
Patch-Set: ps_...
Evidence-Bundle: evb_...
Trace-Id: trace_...
Pod: pod_...
```

- Direct Git push from an agent workspace is forbidden.
- Apply identity is separate from agent identity.

---

## 17. VCS adapter invariants

### 17.1 Git mapping

| Change Graph | Git |
|---|---|
| `ChangeSet.id` | `Change-Id` trailer and branch metadata |
| `PatchSet.id` | commit trailer or exported ref metadata |
| `Stack.parents` | parent trailers + submit ordering |
| `EvidenceBundle.id` | commit trailer + status context |
| `PolicyVerdict` | status/check conclusion |

Git export MUST be deterministic for the same frozen patchset input.

### 17.2 jj mapping

| Change Graph | jj |
|---|---|
| `ChangeSet` | jj change lineage/workspace change |
| `PatchSet` | exported immutable snapshot of jj change at time of patchset creation |
| `Stack` | jj change parent chain |
| conflict state | jj conflict materialization plus Change Graph conflict projection |

Rules:

- jj is a workspace ergonomics adapter, not v0 source of truth.
- Interleaved raw Git mutation in a colocated jj workspace MUST be detected by base hash mismatch before patchset creation.
- jj operation log IDs MAY be stored as debug metadata but MUST NOT be required for Git export compatibility.

### 17.3 GitHub mirror mapping

GitHub is a compatibility surface:

- PR title/body mirror ChangeSet summary.
- Checks mirror EvidenceBundle lane results.
- Labels mirror blast radius and pod ownership.
- Comments are informational only; policy truth remains in Change Graph.
- GitHub Actions may run smoke/compatibility lanes but is not authoritative once Fabric CI is authoritative.

### 17.4 Optional Gerrit mapping

If Gerrit is piloted:

- `ChangeSet.id` maps to Gerrit `Change-Id`.
- `PatchSet.sequence` maps to Gerrit patch set number.
- Submit requirements mirror PolicyVerdict requirements.
- Topics map to Stack IDs.

---

## 18. Typed release gates

Release-captain CD MUST import canonical release requirements instead of empty metric placeholders.

```yaml
schema: oya.delivery.release-gates.v1
required:
  - immutable_artifact
  - sbom_spdx_or_cyclonedx
  - cosign_signature
  - provenance_attestation
  - no_open_sev1_sev2
  - release_evidence_bundle
  - rollback_plan
  - rollback_drill_freshness
rollout:
  stages:
    - one_box
    - canary_5_percent
    - canary_25_percent
    - canary_50_percent
    - full_100_percent
  burn_rate_rollback:
    one_hour_slo_burn_threshold: 14.4
  bake_time_policy:
    one_box_minimum: "1h unless service-specific stricter policy"
    first_region_wave_minimum: "12h unless service-specific stricter policy"
```

Rules:

- Missing SBOM/provenance/signature blocks release.
- Open Sev1/Sev2 affecting the surface blocks release.
- Metric gate failure triggers rollback before human paging where automation can act safely.
- Rollback failure escalates to on-call + TPM with incident artifact.

---

## 19. Hard implementation gates added from architect review

These gates upgrade the milestone plan:

1. M0 MUST implement append-only event fixtures and projection rebuild test, not only static JSON schemas.
2. M1 MUST implement precise direct + reverse transitive Rust affected graph before any cluster runner work.
3. M3 MUST implement per-change leases, CAS append, idempotency keys, and monotonic patchset sequence.
4. M4 MUST implement deterministic Git export and base mismatch detection for jj/Git interleaving.
5. M5 MUST implement submit transaction dry-run with frozen inputs and policy re-evaluation.
6. M6 MUST serialize root-manifest submits and reject unsupported atomic stacks.
7. M8 MUST include typed release gates, burn-rate rollback simulation, and rollback alarm failure-injection.


---

## 20. Executor contract patch: schemas, transactional CLI/API, exact first issues

This section supersedes any weaker command examples or issue descriptions above when implementation begins.

### 20.1 Complete mandatory schema set

M0 MUST create fixtures and validators for all of these schemas:

| Schema | Fixture path | Purpose |
|---|---|---|
| `oya.delivery.change-set.v1` | `registry/delivery/fixtures/change-set.valid.json` | logical change |
| `oya.delivery.patch-set.v1` | `registry/delivery/fixtures/patch-set.valid.json` | immutable patchset snapshot |
| `oya.delivery.stack.v1` | `registry/delivery/fixtures/stack.valid.json` | ordered/independent change stack |
| `oya.delivery.evidence-bundle.v1` | `registry/delivery/fixtures/evidence-bundle.valid.json` | CI/test/policy evidence |
| `oya.delivery.policy-verdict.v1` | `registry/delivery/fixtures/policy-verdict.valid.json` | submit decision |
| `oya.delivery.agent-pod.v1` | `registry/delivery/fixtures/agent-pod.valid.json` | autonomous pod scope |
| `oya.delivery.lane.v1` | `registry/delivery/fixtures/lane.valid.json` | lane definition |
| `oya.delivery.ci-plan.v1` | `registry/delivery/fixtures/ci-plan.valid.json` | affected graph lane plan |
| `oya.delivery.event.v1` | `registry/delivery/fixtures/event.valid.json` | append-only event |
| `oya.delivery.lease.v1` | `registry/delivery/fixtures/lease.valid.json` | lease/fencing contract |
| `oya.delivery.release.v1` | `registry/delivery/fixtures/release.valid.json` | release object |
| `oya.delivery.release-gates.v1` | `registry/delivery/fixtures/release-gates.valid.json` | typed rollout gates |

Recommended schema paths:

```text
registry/delivery/schemas/change-set.v1.schema.json
registry/delivery/schemas/patch-set.v1.schema.json
registry/delivery/schemas/stack.v1.schema.json
registry/delivery/schemas/evidence-bundle.v1.schema.json
registry/delivery/schemas/policy-verdict.v1.schema.json
registry/delivery/schemas/agent-pod.v1.schema.json
registry/delivery/schemas/lane.v1.schema.json
registry/delivery/schemas/ci-plan.v1.schema.json
registry/delivery/schemas/event.v1.schema.json
registry/delivery/schemas/lease.v1.schema.json
registry/delivery/schemas/release.v1.schema.json
registry/delivery/schemas/release-gates.v1.schema.json
```

### 20.2 Transactional CLI contract

All mutating commands MUST accept `--idempotency-key`. Commands that append to an event stream MUST also accept `--expected-sequence` and `--prev-event-hash` unless they first acquire a lease that returns those values.

#### `repoctl change create`

```bash
repoctl change create \
  --title "..." \
  --owner-pod pod_foundry_repoctl \
  --base HEAD \
  --idempotency-key "$ACTOR:create:$TASK_HASH" \
  --expected-sequence 0 \
  --prev-event-hash sha256:genesis \
  --format json
```

Failure payload:

```json
{
  "schema": "oya.delivery.command-error.v1",
  "status": "failed",
  "code": "cas_mismatch|duplicate_idempotency_key|invalid_schema|ownership_unknown",
  "message": "human actionable explanation",
  "retryable": true,
  "current_sequence": 4,
  "current_prev_event_hash": "sha256:..."
}
```

#### `repoctl change lease acquire`

```bash
repoctl change lease acquire \
  --scope change/chg_... \
  --holder pod_foundry_repoctl \
  --ttl 15m \
  --idempotency-key "$RUN:lease:chg_..." \
  --format json
```

Output includes `lease_id`, `fencing_token`, `expected_sequence`, and `prev_event_hash`.

#### `repoctl change patchset create`

```bash
repoctl change patchset create \
  --change chg_... \
  --workspace /path/to/workspace \
  --summary "..." \
  --lease-id lease_... \
  --fencing-token 123 \
  --idempotency-key "$RUN:patchset:$WORKSPACE_DIFF_HASH" \
  --expected-sequence 7 \
  --prev-event-hash sha256:... \
  --format json
```

#### `repoctl ci plan`

```bash
repoctl ci plan \
  --change chg_... \
  --patchset ps_... \
  --base-commit <sha> \
  --patchset-hash sha256:... \
  --format json
```

Output schema: `oya.delivery.ci-plan.v1`.

#### `repoctl submit check`

```bash
repoctl submit check \
  --change chg_... \
  --patchset ps_... \
  --evidence evb_... \
  --freeze-inputs \
  --format json
```

Output schema: `oya.delivery.policy-verdict.v1`.

#### `repoctl submit apply`

```bash
repoctl submit apply \
  --change chg_... \
  --patchset ps_... \
  --frozen-evidence evb_... \
  --policy-verdict pv_... \
  --lease-id lease_... \
  --fencing-token 456 \
  --idempotency-key "$RUN:submit:chg_...:ps_...:evb_..." \
  --expected-sequence 12 \
  --prev-event-hash sha256:... \
  --mode git-export \
  --format json
```

Failure payload codes MUST include:

- `stale_fencing_token`
- `lease_expired`
- `stale_evidence`
- `policy_not_allow`
- `protected_head_moved`
- `stack_parent_blocked`
- `unsupported_atomic_stack`
- `git_export_failed`
- `status_mirror_failed`
- `root_manifest_lease_required`

### 20.3 Transactional HTTP API contract

Mutating `deliveryd` endpoints MUST accept:

```json
{
  "idempotency_key": "stable key",
  "expected_sequence": 12,
  "prev_event_hash": "sha256:...",
  "lease_id": "lease_...",
  "fencing_token": 456,
  "actor": { "kind": "agent", "id": "run_..." }
}
```

Responses MUST include:

```json
{
  "status": "ok|failed",
  "event_id": "evt_...",
  "sequence": 13,
  "event_hash": "sha256:...",
  "projection_ref": "chg_...|ps_...|evb_...",
  "trace_id": "trace_..."
}
```

### 20.4 Exact first 10 implementation issues, v2

These replace the earlier ADF-001–ADF-010 list.

#### ADF-001 — Delivery schema registry and fixtures

- **Depends on:** none.
- **Target paths:** `registry/delivery/schemas/*.schema.json`, `registry/delivery/fixtures/*.valid.json`, `registry/delivery/fixtures/*.invalid.json`.
- **CLI:** none yet.
- **Acceptance tests:** Group A.
- **Exit:** all 12 mandatory schemas have valid and invalid fixtures.

#### ADF-002 — Repoctl schema validator command

- **Depends on:** ADF-001.
- **Target paths:** `tools/repoctl/src/delivery.rs` or equivalent command module, `tools/repoctl/src/main.rs` command wiring, `tools/repoctl/tests/delivery_cli.rs`.
- **CLI:** `repoctl delivery validate --schema registry/delivery/schemas --input <path> --format json`.
- **Acceptance tests:** Group A.
- **Exit:** valid fixtures pass, invalid fixtures fail with schema path and reason.

#### ADF-003 — Append-only event store MVP

- **Depends on:** ADF-001, ADF-002.
- **Target paths:** `tools/repoctl/src/delivery_store.rs`, `.omx/delivery/events/` fixture docs, tests.
- **CLI:** `repoctl delivery store append --stream ... --idempotency-key ... --expected-sequence ... --prev-event-hash ... --payload ...`.
- **Acceptance tests:** Group J.
- **Exit:** CAS mismatch, duplicate idempotency, projection rebuild, and hash-chain corruption tests pass.

#### ADF-004 — Lease and fencing contract

- **Depends on:** ADF-003.
- **Target paths:** `tools/repoctl/src/delivery_lease.rs`, tests.
- **CLI:** `repoctl change lease acquire|release --scope ... --holder ... --ttl ... --format json`.
- **Acceptance tests:** Group J.
- **Exit:** expired lease and stale fencing token tests fail closed.

#### ADF-005 — Rust affected graph planner

- **Depends on:** ADF-001, ADF-002.
- **Target paths:** `tools/repoctl/src/ci/affected.rs`, `tools/repoctl/src/ci.rs`, tests.
- **CLI:** `repoctl ci plan --change ... --patchset ... --base-commit ... --patchset-hash ... --format json`.
- **Acceptance tests:** Group B, Group I.
- **Exit:** direct crates, reverse dependents, docs-only, root manifest, public API, unsafe/concurrency cases pass.

#### ADF-006 — Rust lane runner and EvidenceBundle writer

- **Depends on:** ADF-005.
- **Target paths:** `tools/repoctl/src/ci/runner.rs`, `tools/repoctl/src/ci/evidence.rs`, tests.
- **CLI:** `repoctl ci run --plan <ci-plan> --format json`.
- **Acceptance tests:** Group C, Group I.
- **Exit:** fmt/check/clippy/nextest lanes emit EvidenceBundle with logs, JUnit/archive refs, cache metrics or `not_configured` reason.

#### ADF-007 — ChangeSet/PatchSet commands on event store

- **Depends on:** ADF-003, ADF-004.
- **Target paths:** `tools/repoctl/src/change.rs`, `tools/repoctl/tests/change_cli.rs`.
- **CLI:** `repoctl change create`, `repoctl change patchset create`, `repoctl change status` with transactional flags.
- **Acceptance tests:** Group D, Group J.
- **Exit:** multiple patchsets, monotonic sequence, lineage, conflict projection, and lease requirements pass.

#### ADF-008 — Submit check policy engine

- **Depends on:** ADF-006, ADF-007.
- **Target paths:** `tools/repoctl/src/submit.rs`, `tools/repoctl/src/policy.rs`, tests.
- **CLI:** `repoctl submit check --freeze-inputs ...`.
- **Acceptance tests:** Group F, Group K.
- **Exit:** missing/failed/stale evidence, ownership gap, blast radius, and stack parent blockers produce structured PolicyVerdict.

#### ADF-009 — Deterministic Git export and GitHub mirror dry-run

- **Depends on:** ADF-008.
- **Target paths:** `tools/repoctl/src/vcs/git_export.rs`, `tools/repoctl/src/vcs/github_mirror.rs`, tests.
- **CLI:** `repoctl change export git`, `repoctl change export github-pr --dry-run`, `repoctl submit apply --mode git-export --dry-run`.
- **Acceptance tests:** Group E, Group K, Group L.
- **Exit:** deterministic trailers, replay-safe export, status payload generation, and protected-head-moved detection pass.

#### ADF-010 — Pod manifest autonomy gate

- **Depends on:** ADF-008.
- **Target paths:** `product-control/pods/*.yaml`, `tools/repoctl/src/pod.rs`, tests.
- **CLI:** `repoctl pod validate`, `repoctl pod claim`, `repoctl pod status`.
- **Acceptance tests:** Group F, Group G.
- **Exit:** owned low-risk change can reach allow verdict; cross-project/high-blast changes return `needs_tpm`; runtime security policy is represented in pod status.

### 20.5 Dependency order

ADF-010 depends on ADF-008. Pod autonomy gates consume the submit policy engine; they do not feed it. The authoritative dependency order is:

```text
ADF-001
  -> ADF-002
    -> ADF-003 -> ADF-004 -> ADF-007 -----------\
    -> ADF-005 -> ADF-006 -----------------------+-> ADF-008 -> ADF-009
                                                   \
                                                    -> ADF-010
```

Expanded order:

1. ADF-001 creates schemas/fixtures.
2. ADF-002 validates schemas.
3. ADF-003 creates append-only store.
4. ADF-004 adds leases/fencing.
5. ADF-005 creates Rust affected graph planner.
6. ADF-006 creates Rust lane runner/evidence writer.
7. ADF-007 creates ChangeSet/PatchSet commands on the event store.
8. ADF-008 creates submit check/policy engine from ADF-006 evidence + ADF-007 graph state.
9. ADF-009 creates deterministic Git export/GitHub mirror after policy exists.
10. ADF-010 creates pod manifest autonomy gates after policy exists.

Implementation MUST NOT start cluster runners, production release automation, or VCS storage replacement until ADF-001 through ADF-010 pass.


---

## 21. Final critic patch: HTTP submit/apply and dependency DAG

### 21.1 Explicit HTTP submit/apply contract

`POST /delivery/v1/submit/apply` MUST use the generic transactional envelope from §20.3 plus explicit frozen submit inputs.

Request:

```json
{
  "idempotency_key": "run_123:submit:chg_1:ps_3:evb_7",
  "expected_sequence": 12,
  "prev_event_hash": "sha256:...",
  "lease_id": "lease_submit_...",
  "fencing_token": 456,
  "actor": { "kind": "agent", "id": "run_123" },
  "change_id": "chg_...",
  "patchset_id": "ps_...",
  "frozen_patchset_hash": "sha256:...",
  "frozen_evidence_id": "evb_...",
  "frozen_evidence_hash": "sha256:...",
  "policy_verdict_id": "pv_...",
  "policy_verdict_hash": "sha256:...",
  "mode": "git-export",
  "protected_head": "<git-sha-observed-during-check>",
  "stack_id": null
}
```

Success response:

```json
{
  "schema": "oya.delivery.submit-apply-response.v1",
  "status": "ok",
  "event_id": "evt_...",
  "sequence": 13,
  "event_hash": "sha256:...",
  "projection_ref": "chg_...",
  "trace_id": "trace_...",
  "export_refs": {
    "git_commit": "<sha>",
    "git_branch": "adf/chg_...",
    "github_pr": null
  }
}
```

Failure response:

```json
{
  "schema": "oya.delivery.command-error.v1",
  "status": "failed",
  "code": "stale_fencing_token|lease_expired|stale_evidence|policy_not_allow|protected_head_moved|stack_parent_blocked|unsupported_atomic_stack|git_export_failed|status_mirror_failed|root_manifest_lease_required|cas_mismatch",
  "message": "human actionable explanation",
  "retryable": true,
  "current_sequence": 13,
  "current_prev_event_hash": "sha256:...",
  "trace_id": "trace_...",
  "required_actions": []
}
```

Rules:

- `frozen_evidence_id` and `policy_verdict_id` are required for every submit/apply request.
- `frozen_patchset_hash`, `frozen_evidence_hash`, and `policy_verdict_hash` are rechecked before export/apply.
- `protected_head` movement returns `protected_head_moved` unless the request includes an explicit revalidation event from the same frozen inputs.
- HTTP and CLI submit/apply MUST share the same failure code enum.

### 21.2 Correct ADF dependency DAG

ADF-010 depends on ADF-008 because pod autonomy evaluation consumes the submit policy engine. The dependency graph is therefore:

```text
ADF-001
  -> ADF-002
    -> ADF-003 -> ADF-004 -> ADF-007 -----------\
    -> ADF-005 -> ADF-006 -----------------------+-> ADF-008 -> ADF-009
                                                   \
                                                    -> ADF-010
```

Expanded order:

1. ADF-001 creates schemas/fixtures.
2. ADF-002 validates schemas.
3. ADF-003 creates append-only store.
4. ADF-004 adds leases/fencing.
5. ADF-005 creates Rust affected graph planner.
6. ADF-006 creates Rust lane runner/evidence writer.
7. ADF-007 creates ChangeSet/PatchSet commands on the event store.
8. ADF-008 creates submit check/policy engine from ADF-006 evidence + ADF-007 graph state.
9. ADF-009 creates deterministic Git export/GitHub mirror after policy exists.
10. ADF-010 creates pod manifest autonomy gates after policy exists.

No issue depends on ADF-010 before ADF-008. No cycle exists.
