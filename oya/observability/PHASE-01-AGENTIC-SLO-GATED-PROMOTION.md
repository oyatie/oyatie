---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P01-agentic-slo-gated-promotion
status: Active
entry_gate: |
  ADR-0139 + ADR-0131 accepted; /specs/agentic-slo-gated-promotion.json + /specs/per-microservice-flat-layout.json published; cargo workspace ready to accept the 14 new crates under microservices/observability/crates/.
exit_gate: |
  All 15 IPs merged; oya-governance-promotion-readiness CI lane present in Jenkins/Forgejo required checks on dev and staging; release/<ms>/{staging,production} required-check rules live; rollback primitive verified via end-to-end drill; cargo nextest run --workspace exits 0; oya gate validate per-microservice-layout --microservice observability exits 0; oya gate validate authority-cohesion exits 0; HG-OBS gate in /specs/hyperscaler-gates.json registers green.
depends_on:
  - milestone: M01-foundation
    phase: prior phases per master-plan-sequencing
    reason: workspace + Jenkins/Forgejo required-check + Cargo metadata authority must precede gate authoring
owner_team: axis-observability
related_adrs: [ADR-0139, ADR-0131]
related_specs: [/specs/agentic-slo-gated-promotion.json, /specs/per-microservice-flat-layout.json]
date: 2026-05-17
doc_status: published
---

# P01-agentic-slo-gated-promotion: Land the SLO gate end-to-end

## Purpose

This phase ships the full ADR-0139 design — Layer-A (self-hosted Grafana OSS observability stack) plus Layer-B (oyatie-owned SLO engine + eligibility ledger + per-microservice release pointers + event-driven promote workflows + automated rollback + canary cohort weighting). It is delivered as one phase in M01-foundation because every other oyatie µservice depends on the SLO gate to advance past `dev` per the bootstrap-order policy in `/specs/agentic-slo-gated-promotion.json`.

This phase advances master-plan principles:
- Hyperscaler-grade in every practice (industry-leading OSS observability stack + canonical Google SRE burn-rate model).
- Nothing scheduled-for-distinct-tracked-work (every FUTURE-marked stub in the existing promote workflows is decommissioned by this phase).
- No silent regression (production-tier breach auto-reverts via the rollback primitive).
- Per-microservice flat layout (this phase is itself the first native author under ADR-0131).

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `observability` | `slo-engine`, `otel-ingest` | All under `microservices/observability/` per ADR-0131 | `oya-observability-slo-engine-{kernel,domain,application,adapter,rest,worker,app}` and `oya-observability-otel-ingest-{kernel,domain,application,adapter,worker,app}` |

Plus these repo-wide artifacts (cross-cutting per ADR-0131):
- Jenkins/Forgejo required-check configuration — add `oya-governance-promotion-readiness` to required checks on `dev` and `staging`; add required-check rules for `release/*/staging` and `release/*/production`.
- Promotion pipeline jobs — switch primary trigger to signed `eligibility-changed` events; retain cron as heartbeat; remove FUTURE-marked stub references.
- Production promotion pipeline jobs — analogous.
- `docs/standards/observability-slo.md` (NEW) — cross-cutting OpenSLO authoring rules; SLI catalog; burn-rate threshold convention.
- `registry/promotion-eligibility.jsonl` (NEW) — append-only ledger.
- `Cargo.toml` (workspace) — register the 13 new crates under `microservices/observability/crates/`.
- `/specs/hyperscaler-gates.json` — register HG-OBS gate per ADR-0123.

Naming justifications for the new crate families are in `microservices/observability/PRD.md` §"Bounded Contexts".

### Out-of-scope

- Migration of existing µservices (tenancy, ontology, workflow, etc.) into `microservices/<ms>/` — owned by the IP-M01-MIGR-* series per ADR-0131; runs in parallel under separate phases. Until those migrations land, this phase's gate authors OpenSLO manifests only for `observability` itself (self-observability) and proves the gate end-to-end via the synthetic-probe fallback per PRD AC-02.
- Federated cross-region Mimir/Loki/Tempo — scheduled-for-distinct-tracked-work to a subsequent-to-M01-completion ADR per PRD §"Horizontal Scalability".
- Tenant-defined SLO authoring UX — covered by `microservices/observability/PRD.md` Open Question 3; scheduled-for-distinct-tracked-work to a successor-IP ADR; the underlying engine accepts OpenSLO manifests in tenant scope, so no gate-shape change is implied.
- Rollback of pre-existing `staging`/`production` ref schema (the deprecated tree-wide refs retain read-only existence per `/specs/agentic-slo-gated-promotion.json` §"deprecated_refs"; explicit removal is a successor-IP ADR).

## Implementation Plans

Ordered list. Each IP is an executable ChangeSet under this phase folder. Dependencies inline.

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-layer-a-grafana-stack-iac.md`](IP-001-layer-a-grafana-stack-iac.md) | Helm/Kustomize charts for Grafana Alloy + Prometheus + Mimir + Loki + Tempo + Pyroscope + Grafana + Alertmanager + Grafana OnCall under `microservices/observability/iac/helm/` | pending | axis-observability | — |
| [`IP-002-openslo-manifest-convention.md`](IP-002-openslo-manifest-convention.md) | `docs/standards/observability-slo.md` cross-cutting standard; first OpenSLO manifests for `observability` self-SLOs at `microservices/observability/slos/` | pending | axis-observability | — |
| [`IP-003-slo-engine-kernel.md`](IP-003-slo-engine-kernel.md) | `oya-observability-slo-engine-kernel` crate: port traits (SloTargetRepository, PrometheusClient, BurnRateEvaluator, EligibilityLedgerWriter), entities, value objects | pending | axis-observability | IP-002 |
| [`IP-004-slo-engine-domain.md`](IP-004-slo-engine-domain.md) | `oya-observability-slo-engine-domain` crate: burn-rate math, multi-window aggregation, error-budget arithmetic; verified against Google SRE Workbook reference values | pending | axis-observability | IP-003 |
| [`IP-005-slo-engine-application.md`](IP-005-slo-engine-application.md) | `oya-observability-slo-engine-application` crate: orchestrator that reads OpenSLO + Prometheus and writes ledger via ports | pending | axis-observability | IP-004 |
| [`IP-006-slo-engine-adapter.md`](IP-006-slo-engine-adapter.md) | `oya-observability-slo-engine-adapter` crate: OpenSLO YAML reader, Mimir PromQL HTTP client, JSONL ledger writer | pending | axis-observability | IP-003 |
| [`IP-007-slo-engine-rest.md`](IP-007-slo-engine-rest.md) | `oya-observability-slo-engine-rest` crate: OpenAPI-defined REST surface for human/agent ledger + SLO queries (`microservices/observability/contracts/openapi/slo-engine.yaml`) | pending | axis-observability | IP-005, IP-006 |
| [`IP-008-slo-engine-worker.md`](IP-008-slo-engine-worker.md) | `oya-observability-slo-engine-worker` crate: continuous burn-rate evaluator, 60s cadence, emits signed `EligibilityChanged` events | pending | axis-observability | IP-005, IP-006 |
| [`IP-009-slo-engine-app.md`](IP-009-slo-engine-app.md) | `oya-observability-slo-engine-app` composition root binary wiring worker + rest + adapters | pending | axis-observability | IP-007, IP-008 |
| [`IP-010-promotion-eligibility-ledger.md`](IP-010-promotion-eligibility-ledger.md) | `registry/promotion-eligibility.jsonl` schema, union-merge driver assertion, append-only writer crate hook | pending | axis-observability | IP-005 |
| [`IP-011-per-component-release-pointers.md`](IP-011-per-component-release-pointers.md) | `release/<ms>/<env>` ref naming, required-check rules, fast-forward primitive on top of signed Git refs | pending | ops-sre-reliability | IP-010 |
| [`IP-012-governance-promotion-readiness-lane.md`](IP-012-governance-promotion-readiness-lane.md) | New BLOCKER CI lane `oya-governance-promotion-readiness`; reads ledger; refuses release-pointer advancement unless verdicts green | pending | axis-foundry | IP-010, IP-011 |
| [`IP-013-event-driven-promote-workflows.md`](IP-013-event-driven-promote-workflows.md) | Rewrite promotion jobs to consume signed `eligibility-changed` events; decommission FUTURE stubs | pending | ops-sre-reliability | IP-012 |
| [`IP-014-automated-rollback-primitive.md`](IP-014-automated-rollback-primitive.md) | Production-tier fast-burn → ref revert; signed; ledger `rollback` verdict; Grafana OnCall incident | pending | axis-observability | IP-011, IP-013 |
| [`IP-015-canary-cohort-weighting.md`](IP-015-canary-cohort-weighting.md) | Service-mesh traffic-split ramp 1 → 10 → 50 → 100 %; abort-on-burn; otel-ingest BC integration | pending | ops-sre-reliability | IP-001, IP-008 |

Coverage check vs. ADR-0139 §"Concrete file and crate changes": all 7 slo-engine layer crates (IP-003–IP-009), Layer-A IaC (IP-001), OpenSLO manifests + standard (IP-002), ledger (IP-010), pointers (IP-011), gate lane (IP-012), workflows (IP-013), rollback (IP-014), canary (IP-015). The `otel-ingest` BC crates ship as part of the Layer-A IaC + otel collector integration in IP-001 + IP-015; if they require their own IP they will be appended as IP-016+ during execution.

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

```bash
oya gate validate lean-a1 --microservice observability        # layer ordering
oya gate validate lean-a2 --microservice observability        # cross-product refusal
oya gate validate port-location --microservice observability  # ports in kernel
oya gate validate layer-correctness --microservice observability
oya gate validate per-microservice-layout --microservice observability  # ADR-0131
oya gate validate statelessness --microservice observability
oya gate validate shardability --microservice observability
oya gate validate authority-cohesion                          # registers HG-OBS
oya gate validate hyperscaler-maturity-claims                 # ADR-0123
```

### Substrate gates introduced by this phase

```bash
oya gate validate oya-governance-promotion-readiness --sha <head-sha> --env staging
oya gate validate oya-governance-promotion-readiness --sha <head-sha> --env production
oya gate validate aggregation-index-generation                 # ADR-0131 sibling lane
```

### End-to-end drill gates

| Scenario | Command | Pass criterion |
|---|---|---|
| Eligibility happy path | `cargo nextest run -p oya-observability-slo-engine-application --test eligibility_happy_path` | verdict `eligible`; ledger record appended; event emitted |
| Fast-burn hold | `cargo nextest run -p oya-observability-slo-engine-application --test fast_burn_hold` | verdict transitions `eligible → held` within ≤60 s of synthetic burn injection |
| Promotion gating | `cargo nextest run -p oya-dev-cli --test governance_promotion_readiness_refusal` | lane exits non-zero with structured JSON listing held microservices |
| Rollback drill | scripted e2e: inject burn → assert ref reverted | production ref at prior pointer within ≤60 s; `rollback` verdict appended; Grafana OnCall incident raised |
| Canary ramp | scripted e2e: ramp 1 → 10 → 50 → 100 % | each step honours its `min_duration_seconds` and `exit_criterion` |

### Workflow + Ontology integration gates

```bash
oya gate validate workflow-event-registry --microservice observability
oya gate validate ontology-type-registry --microservice observability
```

## Clean Architecture Compliance

Layer assignments and dependency direction:

| Crate (BNF v4.1) | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-observability-slo-engine-kernel` | `kernel` | (nothing project-internal) | all other layers |
| `oya-observability-slo-engine-domain` | `domain` | `kernel` | `application`, `adapter`, `rest`, `worker`, `app` |
| `oya-observability-slo-engine-application` | `application` | `domain`, `kernel` | `adapter`, `rest`, `worker`, `app` |
| `oya-observability-slo-engine-adapter` | `adapter` | `application`, `domain`, `kernel` | `rest`, `worker`, `app` directly |
| `oya-observability-slo-engine-rest` | `rest` | `application`, `domain`, `kernel` | `adapter` directly (uses ports) |
| `oya-observability-slo-engine-worker` | `worker` | `application`, `domain`, `kernel` | `adapter` directly (uses ports) |
| `oya-observability-slo-engine-app` | `app` | (composition-root wiring only) | none — but only wiring |
| `oya-observability-otel-ingest-{kernel,domain,application,adapter,worker,app}` | same enum mapping per BC |  |  |

Port traits live exclusively in `*-kernel` crates; implementations exclusively in `*-adapter`. Domain calls through ports; domain never imports adapter.

Cross-product integration check: this phase introduces NO direct imports between `observability` and any other product µservice's crates. All cross-product data flow uses Workflow events (`EligibilityChanged`, `PromotionExecuted`, `RollbackExecuted`, `MicroserviceRegistered`, `OpenSLOManifestUpdated`) and Ontology reads/writes (`SLOTarget`, `EligibilityVerdict`, `ReleasePointer`, `Microservice`).

CI lanes that must green before phase exit gate (same as §"Fitness lane gates" above).

## ChangeSet Contract per IP

Every IP in this phase emits a ChangeSet per ADR-0110 (claimable + verifiable + bundleable + promotable). The minimum ChangeSet payload per IP is written at `microservices/observability/evidence/multispectrum/<change_id>-<unix_ts>.json` before opening the pull request against `dev`:

```json
{
  "change_id": "ULID",
  "ip_id": "IP-NNN-<slug>",
  "microservice": "observability",
  "milestone": "M01-foundation",
  "phase": "P01-agentic-slo-gated-promotion",
  "claim_paths": ["microservices/observability/src/crates/<crate>/**", "..."],
  "intent": "<one-line>",
  "spec_refs": ["microservices/observability/PRD.md§<section>", "/specs/agentic-slo-gated-promotion.json§<section>"],
  "acceptance_lanes_green": ["cargo-check", "cargo-build", "cargo-clippy", "cargo-nextest", "cargo-deny", "lean-a1", "lean-a2", "lean-a3", "lean-a4", "per-microservice-layout", "openslo-conformance"],
  "test_count": {"unit": <int>, "integration": <int>, "e2e": <int>},
  "coverage_pct": <float>,
  "multispectrum_review_facets": ["F1..F9", "A1..A7", "M1..M2"],
  "signature": "Ed25519:<sig>",
  "executed_at": "ISO8601"
}
```

The schema is validated by the `oya-governance-multispectrum-evidence` lane against `/specs/multispectrum-review.json` v2.4.0; PRs without a conforming evidence file are refused.

## Per-IP Test Coverage Threshold

| IP class | Minimum unit-test count | Minimum integration-test count | Minimum e2e-test count | Coverage threshold |
|---|---|---|---|---|
| kernel crate (`*-kernel`) | 1 per public type + 1 per port trait | 0 (kernel is pure; no I/O) | 0 | 90% line; 80% branch |
| domain crate (`*-domain`) | 1 per public function + property tests for math | 0 | 0 | 95% line; 90% branch |
| usecase crate (`*-usecase`) | 1 per use case (happy + 2 sad paths) | ≥ 3 against mocked ports | 0 | 90% line; 80% branch |
| adapter crate (`*-adapter*`) | 1 per port-impl method | ≥ 2 against real backend (Mimir test container) | 0 | 85% line; 75% branch |
| rest crate (`*-rest`) | 1 per route (happy + auth-fail + tenant-mismatch) | ≥ 2 cross-route flows | 1 per route via REST integration test | 85% line; 75% branch |
| worker crate (`*-worker`) | 1 per orchestration arm | ≥ 1 long-lived loop integration test | 1 e2e (60s evaluator cycle injecting synthetic SLI) | 85% line; 75% branch |
| sdk crate (`*-sdk`) | 1 per public client method (happy + retry + auth-fail) | ≥ 2 against rest crate | 0 | 90% line; 80% branch |
| app crate (`*-app`) | composition-root smoke tests | 0 (delegates to worker/rest tests) | 1 startup-and-shutdown smoke | 60% line (mostly wiring) |
| IaC IPs (Helm / OpenTofu) | n/a | ≥ 1 helm-install + helm-test smoke per chart | 1 against kind/k3d cluster | n/a |

Enforced by:
- `cargo nextest run --workspace --all-features` exits 0 (overall test runner).
- `cargo llvm-cov --workspace --fail-under-lines <threshold>` exits 0 (coverage).
- Per-IP CI workflow specifies the per-IP thresholds in its `[acceptance_lanes]` frontmatter.

## Required-checks diff preview

IP-013 (event-driven promote workflows) updates the Jenkins/Forgejo required-check configuration with the diff below. Surfaced here so reviewers can preview the change at phase-start, not at IP-merge time.

```yaml
branches:
  dev:
    required_status_checks:
      # existing checks:
      - cargo-fmt
      - cargo-check
      - cargo-clippy
      - cargo-nextest
      - oya-governance-admission
      - oya-governance-provider-execution
      - oya-governance-supply-chain
      - oya-governance-cohesion
      - oya-governance-api-semver
      - oya-governance-protection-context-match
      - oya-pr-review
      # ADDED by this phase (IP-012 + IP-013):
      - oya-governance-promotion-readiness            # NEW; reads Mimir eligibility verdicts
      - oya-governance-openslo-conformance   # NEW; from docs/standards/observability-slo.md
      - oya-governance-mimir-tenancy-enforced # NEW; from /specs/agentic-slo-gated-promotion.json §mimir_multi_tenancy
      - oya-governance-per-microservice-layout   # NEW; from ADR-0131
      - oya-governance-aggregation-index-generation   # NEW; from ADR-0131
      - oya-governance-no-grouping    # NEW; from ADR-0132
      - oya-governance-multispectrum-evidence  # NEW; ChangeSet payload validator
      - oya-governance-version-pinning-conformance   # NEW; from docs/standards/observability-slo.md §"Version Pinning"

  staging:
    required_status_checks:
      # ADDED by this phase:
      - oya-governance-promotion-readiness

  # ADDED — pattern-based protection for per-component release pointers
  ? release/*/staging
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    require_signed_tags: true
    required_status_checks:
      - oya-governance-promotion-readiness

  ? release/*/production
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    require_signed_tags: true
    required_status_checks:
      - oya-governance-promotion-readiness
```

IP-011 (per-component release pointers) creates the initial set of `release/<microservice>/{staging,production}` refs (one per active µservice); the pattern protection covers all current + future refs.

## Git/Jenkins governance handoff

Per ADR-0363 and the current repo AGENTS contract, this phase uses one isolated branch/worktree per implementation lane, a pull request against `dev`, Jenkins CI, `oya gate`, and `oya verify`. Grit, ICM, and retired `oya vcs` primitives are explicitly NOT used.

```bash
# Create an isolated branch/worktree before beginning each IP.
git worktree add ../oyatie-observability-<ip-id> -b feature/observability-<ip-id> dev

# Verify after each IP's acceptance gates pass.
buck2 build //:repo-hygiene-automation-check --ci-required
cargo run -p oya-dev-cli -- verify --ci-required

# Open a PR against dev; Jenkins CI and reviewer APPROVE provide merge readiness.
git push -u origin feature/observability-<ip-id>

# Promotion is Jenkins/ArgoCD-governed after merge, using signed image digests.
```

Multispectrum evidence per docs/AGENTS.md §changeset: each IP emits `microservices/observability/evidence/multispectrum/<change_id>-<unix_ts>.json` per `/specs/multispectrum-review.json` v2.4.0.

## References

- ADR-0139: Agentic SLO-gated promotion (this phase's design authority).
- ADR-0131: Per-microservice flat layout (this phase's location authority).
- ADR-0056: BNF v4.1.
- ADR-0105: 13-layer enum.
- ADR-0110: ChangeSet state machine.
- ADR-0114: Canary observability rollback.
- ADR-0116: Retire external agent-coordination tooling.
- ADR-0123: Hyperscaler maturity claim gate (HG-OBS).
- `/specs/agentic-slo-gated-promotion.json`.
- `/specs/per-microservice-flat-layout.json`.
- `microservices/observability/PRD.md`.
- Memory: `feedback_milestone_phase_hierarchy.md`, `feedback_naming_justification.md`, `feedback_oya_vcs_canonical_2026_05_16.md`, `feedback_clean_architecture_requirements.md`, `feedback_quality_performance_scalability_bar.md`.
