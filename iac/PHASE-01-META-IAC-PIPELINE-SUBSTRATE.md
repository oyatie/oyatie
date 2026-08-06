---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P01-meta-iac-pipeline-substrate
status: Active
entry_gate: |
  ADR-0131 + ADR-0117 accepted; cargo workspace ready to accept the new crates under
  iac/src/crates/; cloud-k8s + cloud-secrets minimal substrate live so the
  bootstrap dependency chain is satisfied.
exit_gate: |
  All 15 IPs merged; HG-CLOUD-IAC gate in /specs/hyperscaler-gates.json registers green; the
  iac-applier consumes a real EligibilityChanged (eligible) event end-to-end and applies a µservice;
  drift-detector runs continuously across pack-kr cluster; rollback drill verified ≤2min; the
  branch-protected `oya-ci-required` fan-in reports green for workspace tests,
  per-microservice-layout, and authority-cohesion gates.
depends_on:
  - milestone: M01-foundation
    phase: prior phases per master-plan-sequencing
    reason: workspace, branch-protection, cloud-k8s minimum-viable cluster, cloud-secrets
            OpenBao deployment, observability SLO gate (downstream signal) all preceding
owner_team: axis-cloud-iac
related_adrs: [ADR-0117, ADR-0120, ADR-0121, ADR-0139, ADR-0131]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/hyperscaler-gates.json]
date: 2026-05-17
doc_status: published
---

# P01-meta-iac-pipeline-substrate: Land the IaC pipeline end-to-end

## Purpose

This phase ships the full cloud-iac design: Layer-A self-hosted IaC OSS stack (ArgoCD + Flux + OpenTofu + Helm-controller + Kustomize-controller + Postgres iac-state-index) and Layer-B oyatie-owned crates (iac-renderer + iac-validator + iac-applier + iac-rollback + iac-registry). Delivered as one phase in M01-foundation because every other oyatie µservice depends on cloud-iac to render + apply its IaC; the SLO gate (ADR-0139) needs ApplyExecuted signals to advance per-component release pointers.

This phase advances master-plan principles:
- Hyperscaler-grade in every practice (one IaC pipeline canonicalised across Helm + Kustomize + OpenTofu; not three diverging tools).
- Nothing scheduled-for-distinct-tracked-work (SLSA L3 + Sigstore + Cedar policy + audit-chain seal default day-1).
- No silent regression (drift detection ≤1h per cluster; rollback signed + audit-emitted).
- Per-microservice flat layout (this phase native author under ADR-0131).

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `cloud-iac` | `iac-renderer`, `iac-validator`, `iac-applier`, `iac-rollback`, `iac-registry` | All under `iac/` per ADR-0131 | `oya-cloud-iac-iac-renderer-{kernel,domain,usecase,api,adapter,adapter-helm,adapter-kustomize,adapter-opentofu,rest,worker,sdk,app}` + `oya-cloud-iac-iac-validator-{kernel,domain,usecase,api,adapter,rest,worker,app}` + `oya-cloud-iac-iac-applier-{kernel,domain,usecase,api,adapter,adapter-argocd,rest,worker,app}` + `oya-cloud-iac-iac-rollback-{kernel,domain,usecase,api,adapter,rest,worker,app}` + `oya-cloud-iac-iac-registry-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk,app}` |

Plus repo-wide cross-cutting artifacts:
- `Cargo.toml` (workspace) — register the new crates under `iac/src/crates/`.
- `/specs/hyperscaler-gates.json` — register HG-CLOUD-IAC gate per ADR-0123.
- `.github/branch-protection.yaml` — add `oya-cloud-iac-iac-smoke` to required_status_checks on `dev`.

### Out-of-scope

- Migration of existing µservices' IaC (observability, tenancy, ontology, …) into the cloud-iac registry — this phase ships the substrate; per-µservice IaC adoption is incremental per-µservice phases that follow.
- Multi-pack global iac-registry federation — scheduled-for-distinct-tracked-work to a subsequent-to-M01-completion ADR.
- Tenant-authored IaC programmability — scheduled-for-distinct-tracked-work; M01 ships oyatie's own IaC orchestration; tenant-side authoring is via git-PR only.
- ArgoCD ApplicationSet templating for tenant-namespaced applications — scheduled-for-distinct-tracked-work to a successor-IP phase (FP-NN) once first tenant onboards.

## Implementation Plans

Ordered list. Each IP is an executable ChangeSet under this phase folder. Dependencies inline.

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-layer-a-argocd-flux-iac.md`](IP-001-layer-a-argocd-flux-iac.md) | Helm/Kustomize charts for ArgoCD + Flux + Helm-controller + Kustomize-controller under `iac/iac/helm/` | pending | axis-cloud-iac | — |
| [`IP-002-layer-a-opentofu-iac.md`](IP-002-layer-a-opentofu-iac.md) | Helm chart for OpenTofu self-hosted runner + per-pack state-bucket OpenTofu config | pending | axis-cloud-iac | — |
| [`IP-003-iac-renderer-kernel.md`](IP-003-iac-renderer-kernel.md) | `oya-cloud-iac-iac-renderer-kernel`: port traits + entities + sealed traits | pending | axis-cloud-iac | — |
| [`IP-004-iac-renderer-domain-usecase.md`](IP-004-iac-renderer-domain-usecase.md) | `-domain` + `-usecase`: dependency-ordering + content-digest + render orchestrator | pending | axis-cloud-iac | IP-003 |
| [`IP-005-iac-renderer-adapter-trio.md`](IP-005-iac-renderer-adapter-trio.md) | `-adapter-helm` + `-adapter-kustomize` + `-adapter-opentofu`: backend-qualified renderers | pending | axis-cloud-iac | IP-003 |
| [`IP-006-iac-validator-kernel-domain-usecase.md`](IP-006-iac-validator-kernel-domain-usecase.md) | `oya-cloud-iac-iac-validator-*` core stack (kernel + domain + usecase + adapter) | pending | axis-cloud-iac | IP-003, IP-005 |
| [`IP-007-iac-applier-kernel-domain-usecase.md`](IP-007-iac-applier-kernel-domain-usecase.md) | `oya-cloud-iac-iac-applier-*` core stack + `-adapter-argocd` | pending | axis-cloud-iac | IP-003, IP-006 |
| [`IP-008-iac-registry-postgres.md`](IP-008-iac-registry-postgres.md) | `oya-cloud-iac-iac-registry-*` core stack + `-adapter-postgres`; iac-state-index schema | pending | axis-cloud-iac | IP-003 |
| [`IP-009-iac-rollback-engine.md`](IP-009-iac-rollback-engine.md) | `oya-cloud-iac-iac-rollback-*` core stack; coordinate with SLO gate rollback | pending | axis-cloud-iac | IP-007 |
| [`IP-010-rest-surfaces.md`](IP-010-rest-surfaces.md) | `*-rest` crates for all 5 BCs; OpenAPI conformance | pending | axis-cloud-iac | IP-006, IP-007, IP-008, IP-009 |
| [`IP-011-worker-binaries.md`](IP-011-worker-binaries.md) | `*-worker` crates for all 5 BCs; long-lived loops | pending | axis-cloud-iac | IP-010 |
| [`IP-012-app-composition-roots.md`](IP-012-app-composition-roots.md) | `*-app` composition-root binaries; one per BC | pending | axis-cloud-iac | IP-011 |
| [`IP-013-sdk-and-observability-slo.md`](IP-013-sdk-and-observability-slo.md) | `oya-cloud-iac-iac-renderer-sdk` + `-iac-registry-sdk` + OpenSLO manifests at `iac/slos/` | pending | axis-cloud-iac | IP-011 |
| [`IP-014-per-pack-iac-overlays.md`](IP-014-per-pack-iac-overlays.md) | Per-pack Kustomize overlays; pack-kr live; conditional for pack-eu / pack-us / pack-us-healthcare / pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa | pending | axis-cloud-iac | IP-007, IP-008 |
| [`IP-015-hg-cloud-iac-registration.md`](IP-015-hg-cloud-iac-registration.md) | HG-CLOUD-IAC gate in `/specs/hyperscaler-gates.json`; per ADR-0123 maturity claims gate | pending | axis-cloud-governance + axis-cloud-iac | IP-014 |

Coverage check vs. PRD §"Bounded Contexts": all 40 crates across 5 BCs (12 + 8 + 9 + 8 + 10 = 47 with -app + -sdk + backend-qualified; per ADR-0131 only 40 are net new). The `-sdk` crates ship as part of IP-013; if additional bindings (TS / Py / Go / JVM) become priorities they ship in a successor-IP phase. The bootstrap problem (cloud-iac applies its own IaC) is resolved in IP-015 — minimum-viable substrate first bootstraps via cloud-k8s + cloud-secrets; then cloud-iac applies itself thereafter.

## Acceptance Gates

All gates must pass before `exit_gate` is declared.

### Cargo / CI gates (exit 0 required)

```bash
cargo check --workspace --all-features
cargo build --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo deny check
cargo doc --workspace --no-deps
```

### Fitness lane gates

```text
cloud-ci/oya-ci governance gate `lean-a1` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `lean-a2` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `port-location` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `layer-correctness` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `per-microservice-layout` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `statelessness` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `shardability` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `authority-cohesion` is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `hyperscaler-maturity-claims` is green in the branch-protected `oya-ci-required` context
```

### Substrate gates introduced by this phase

```text
cloud-ci/oya-ci governance gate `cloud-iac-iac-smoke` for --pack pack-kr is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `cloud-iac-render-determinism` is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `cloud-iac-apply-scope-isolation` is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `cloud-iac-drift-detection-coverage` is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `cloud-iac-provenance-slsa-l3` is green in the branch-protected `oya-ci-required` context
```

### End-to-end drill gates

| Scenario | Command | Pass criterion |
|---|---|---|
| Render determinism | `cargo nextest run -p oya-cloud-iac-iac-renderer-usecase --test render_determinism` | identical input → identical content digest |
| Apply scope isolation | `cargo nextest run -p oya-cloud-iac-iac-applier-usecase --test apply_scope_isolation` | apply refused when manifest references resources outside µservice scope |
| Drift detection | scripted e2e: mutate a cluster resource; assert detection within ≤1h | drift report emitted; `DriftDetected` event consumed by observability + audit-chain |
| Rollback drill | scripted e2e: apply v2; auto-revert to v1; assert ≤2min | live cluster reverts; ApplyRolledBack event sealed |
| SLSA L3 verification | `cargo nextest run -p oya-cloud-iac-iac-registry-usecase --test slsa_l3_verify` | apply refused for unsigned chart; accepted for signed |

### Workflow + Ontology integration gates

```text
cloud-ci/oya-ci governance gate `workflow-event-registry` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `ontology-type-registry` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context
```

## Clean Architecture Compliance

Layer assignments and dependency direction:

| Crate (BNF v4.1) | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-cloud-iac-iac-renderer-kernel` | `kernel` | (nothing project-internal) | all other layers |
| `oya-cloud-iac-iac-renderer-domain` | `domain` | `kernel` | `usecase`, `adapter`, `rest`, `worker`, `app` |
| `oya-cloud-iac-iac-renderer-usecase` | `usecase` | `domain`, `kernel` | `adapter`, `rest`, `worker`, `app` |
| `oya-cloud-iac-iac-renderer-adapter` | `adapter` | `usecase`, `domain`, `kernel` | `rest`, `worker`, `app` |
| `oya-cloud-iac-iac-renderer-adapter-helm` | `adapter` | `usecase`, `domain`, `kernel` | same |
| `oya-cloud-iac-iac-renderer-adapter-kustomize` | `adapter` | `usecase`, `domain`, `kernel` | same |
| `oya-cloud-iac-iac-renderer-adapter-opentofu` | `adapter` | `usecase`, `domain`, `kernel` | same |
| `oya-cloud-iac-iac-renderer-rest` | `rest` | `usecase`, `domain`, `kernel`, `api` | `adapter` directly (uses ports) |
| `oya-cloud-iac-iac-renderer-worker` | `worker` | `usecase`, `domain`, `kernel` | `adapter` directly |
| `oya-cloud-iac-iac-renderer-sdk` | `sdk` | `kernel`, `api` | other layers |
| `oya-cloud-iac-iac-renderer-app` | `app` | composition root | none — but only wiring |
| (same enum mapping per BC for iac-validator / iac-applier / iac-rollback / iac-registry) |  |  |  |

Port traits live exclusively in `*-kernel` crates; implementations exclusively in `*-adapter*`. Domain calls through ports; domain never imports adapter.

Cross-product integration check: this phase introduces NO direct imports between `cloud-iac` and any other product µservice's crates. All cross-product data flow uses Workflow events (`RenderRequested`, `ApplyStarted/Completed/RolledBack`, `DriftDetected`, `MicroserviceRegistered`, `EligibilityChanged`) and Ontology reads/writes.

CI lanes that must green before phase exit gate (same as §"Fitness lane gates" above).

## ChangeSet Contract per IP

Every IP in this phase emits a ChangeSet per ADR-0110. Minimum ChangeSet payload at `iac/evidence/multispectrum/<change_id>-<unix_ts>.json` on controller-recorded GitOps change-bundle finalization:

```json
{
  "change_id": "ULID",
  "ip_id": "IP-NNN-<slug>",
  "microservice": "cloud-iac",
  "milestone": "M01-foundation",
  "phase": "P01-meta-iac-pipeline-substrate",
  "claim_paths": ["iac/src/crates/<crate>/**", "..."],
  "intent": "<one-line>",
  "spec_refs": ["iac/PRD.md§<section>", "/specs/per-microservice-flat-layout.json§<section>"],
  "acceptance_lanes_green": ["cargo-check", "cargo-build", "cargo-clippy", "cargo-nextest", "cargo-deny", "lean-a1", "lean-a2", "lean-a3", "lean-a4", "per-microservice-layout", "cloud-iac-iac-smoke"],
  "test_count": {"unit": <int>, "integration": <int>, "e2e": <int>},
  "coverage_pct": <float>,
  "multispectrum_review_facets": ["F1..F9", "A1..A7", "M1..M2"],
  "signature": "Ed25519:<sig>",
  "executed_at": "ISO8601"
}
```

Validated by `oya-governance-multispectrum-evidence` lane against `/specs/multispectrum-review.json` v2.4.0; PRs without conforming evidence refused.

## Per-IP Test Coverage Threshold

Inherits observability PHASE-01 §"Per-IP Test Coverage Threshold" matrix (kernel 90/80; domain 95/90; usecase 90/80; adapter 85/75 + ≥2 integration against real backend; rest 85/75 + 1 per route; worker 85/75 + 1 e2e; sdk 90/80; app 60/wiring; IaC IPs ≥1 helm-install + helm-test).

Enforced via `cargo nextest run --workspace --all-features` + `cargo llvm-cov --workspace --fail-under-lines <threshold>`.

## GitOps Change-Bundle Locks

Per ADR-0116, this phase uses branch-protected GitOps primitives and controller-managed change bundles.

```text
Claim before beginning each IP: branch-protected GitOps/PR workflow with controller-managed change-bundle evidence scoped to the changed paths and intent.

Verify: branch-protected GitOps/PR workflow with controller-managed change-bundle evidence

Done: branch-protected GitOps/PR workflow with controller-managed change-bundle evidence

Promote — fast-forward release pointer through the SLO gate via branch-protected GitOps/PR workflow with controller-managed change-bundle evidence
```

Multispectrum evidence per docs/AGENTS.md §changeset: each IP emits `iac/evidence/multispectrum/<change_id>-<unix_ts>.json` per `/specs/multispectrum-review.json` v2.4.0.

## References

- ADR-0117: Cloud-native infrastructure.
- ADR-0120: Rust-first on-prem tooling.
- ADR-0121: On-prem k8s stack.
- ADR-0139: Agentic SLO-gated promotion (downstream consumer of ApplyExecuted events).
- ADR-0131: Per-microservice flat layout (location authority).
- ADR-0056: BNF v4.1.
- ADR-0105: 13-layer enum.
- ADR-0123: Hyperscaler maturity claim gate (HG-CLOUD-IAC).
- `iac/PRD.md`.
- Memory: `feedback_clean_architecture_requirements.md`, `feedback_quality_performance_scalability_bar.md`, `feedback_naming_justification.md`, `feedback_oya_vcs_canonical_2026_05_16.md`.
