---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
status: Active
entry_gate: |
  PRD-workflow-engine accepted; ADR-0131 unbundle accepted; sibling workflow-studio µservice scaffolded;
  cargo workspace ready to accept the 41 new crates under microservices/workflow-engine/src/crates/;
  Postgres + Citus + Valkey + ClickHouse Layer-A IaC available via cloud-iac µservice.
exit_gate: |
  All 15 IPs merged; engine binary deployed to dev cluster; deterministic-replay CI lane present in
  .github/branch-protection.yaml required_status_checks on dev and staging; release/workflow-engine/{staging,production}
  pattern protection live; durable-execution end-to-end drill passes (engine kill → restart → identical step sequence);
  cargo nextest run --workspace exits 0; oya gate validate per-microservice-layout --microservice workflow-engine exits 0;
  oya gate validate authority-cohesion exits 0; HG-WF-ENGINE gate in /specs/hyperscaler-gates.json registers green.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion
    reason: workflow-engine SLO promotion gate must exist before workflow-engine itself can be advanced past dev
  - milestone: M02b-substrate-ready
    phase: prior phases per master-plan-sequencing
    reason: workspace + branch-protection + Cargo metadata authority must precede engine crate authoring
owner_team: axis-workflow
related_adrs: [ADR-0103, ADR-0139, ADR-0131]
related_specs: [/specs/microservices/workflow.json, /specs/per-microservice-flat-layout.json]
date: 2026-05-17
doc_status: published
---

# P01-durable-execution-substrate: Land the workflow engine end-to-end

## Purpose

This phase ships the full workflow-engine substrate (engine half of the ADR-0131 workflow unbundle) — durable execution at Temporal parity, deterministic replay, sub-second event-to-action latency, per-tenant Citus sharding, audit-sealed run history, replay-debugger-backend. It is delivered as one phase in M02b-substrate-ready because every other oyatie µservice depends on the engine to route cross-product events via the orchestration adapter (per `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md`).

This phase advances master-plan principles:
- Hyperscaler-grade in every practice (Temporal-class durable execution + per-tenant linear sharding).
- Nothing scheduled-for-distinct-tracked-work (every FUTURE-marked stub in any consumer µservice's workflow-event handling is decommissioned by this phase's event-bus SDK).
- No silent regression (deterministic-replay CI lane is BLOCKER day 1).
- Per-microservice flat layout (this phase ships natively under ADR-0131; sibling = workflow-studio).

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `workflow-engine` | `spec-store`, `execution-engine`, `state-machine`, `event-bus`, `replay-debugger-backend` | All under `microservices/workflow-engine/` per ADR-0131 | 41 crates per PRD §"Layer mapping per BC" |

Plus these repo-wide artifacts (cross-cutting per ADR-0131):
- `.github/branch-protection.yaml` — add `oya-governance-deterministic-replay`, `oya-governance-workflow-spec-signature-verification` to required_status_checks on `dev`; add pattern protection for `release/workflow-engine/{staging,production}`.
- `Cargo.toml` (workspace) — register the 41 new crates under `microservices/workflow-engine/src/crates/`.
- `/specs/hyperscaler-gates.json` — register HG-WF-ENGINE gate per ADR-0123.
- `docs/standards/workflow-step-determinism.md` (NEW) — cross-cutting standard for step-body authoring; declares forbidden APIs (system time, non-deterministic RNG, uncached I/O).

Naming justifications for the new crate families are in `microservices/workflow-engine/PRD.md` §"Bounded Contexts".

### Out-of-scope

- The visual editor surface — separate µservice (`microservices/workflow-studio/`) per ADR-0131; runs in parallel phase.
- KR carrier connector library + government API connectors — owned by a successor-IP `integrations` µservice phase; out of scope for engine substrate.
- Plugin marketplace — scheduled-for-distinct-tracked-work to a subsequent-to-M02b-completion plugin-substrate phase.
- Cross-region active-active engine — scheduled-for-distinct-tracked-work to a subsequent-to-M02b-completion ADR per PRD §"Horizontal Scalability".

## Implementation Plans

Ordered list. Each IP is an executable ChangeSet under this phase folder. Dependencies inline.

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-layer-a-postgres-citus-valkey-clickhouse-iac.md`](IP-001-layer-a-postgres-citus-valkey-clickhouse-iac.md) | Helm + Kustomize charts for Postgres+Citus, Valkey (Sentinel HA), ClickHouse, workflow-runtime deployment under `microservices/workflow-engine/iac/helm/` | pending | axis-workflow | — |
| [`IP-002-spec-store-kernel-domain.md`](IP-002-spec-store-kernel-domain.md) | `oya-workflow-engine-spec-store-{kernel,domain}` crates: WorkflowSpec, SpecVersion, SpecSignature entities + pure compile/validate domain | pending | axis-workflow | — |
| [`IP-003-state-machine-kernel-domain.md`](IP-003-state-machine-kernel-domain.md) | `oya-workflow-engine-state-machine-{kernel,domain,usecase,api,adapter,adapter-postgres}` — pure transition evaluation + checkpoint persistence | pending | axis-workflow | IP-002 |
| [`IP-004-execution-engine-kernel-domain.md`](IP-004-execution-engine-kernel-domain.md) | `oya-workflow-engine-execution-engine-{kernel,domain}` crates: WorkflowRun, StepExecution, RetryAttempt, SlaTimer entities; pure retry-backoff + SLA-timer arithmetic | pending | axis-workflow | IP-003 |
| [`IP-005-execution-engine-usecase-durable-execution.md`](IP-005-execution-engine-usecase-durable-execution.md) | `oya-workflow-engine-execution-engine-{usecase,api,adapter,adapter-postgres,adapter-valkey}` — durable-execution authoritative store + ephemeral lease state; the engineering heart of this phase | pending | axis-workflow | IP-004 |
| [`IP-006-event-bus-kernel-domain-adapter.md`](IP-006-event-bus-kernel-domain-adapter.md) | `oya-workflow-engine-event-bus-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-valkey}` — typed event publish/subscribe with outbox + replay-from-offset | pending | axis-workflow | IP-005 |
| [`IP-007-event-bus-rest-worker-sdk-app.md`](IP-007-event-bus-rest-worker-sdk-app.md) | `oya-workflow-engine-event-bus-{rest,worker,sdk,app}` — outbox relay worker; SDK consumed by every µservice; REST entry | pending | axis-workflow | IP-006 |
| [`IP-008-spec-store-usecase-api-adapter-rest-sdk-app.md`](IP-008-spec-store-usecase-api-adapter-rest-sdk-app.md) | spec-store remaining layers: usecase + api + adapter + adapter-postgres + rest + sdk + app | pending | axis-workflow | IP-002, IP-005 |
| [`IP-009-execution-engine-rest-worker-sdk-app.md`](IP-009-execution-engine-rest-worker-sdk-app.md) | execution-engine remaining layers: rest + worker + sdk + app; the engine binary composition root | pending | axis-workflow | IP-005, IP-007 |
| [`IP-010-replay-debugger-backend-kernel-domain.md`](IP-010-replay-debugger-backend-kernel-domain.md) | replay-debugger-backend kernel + domain: pure replay logic over event log | pending | axis-workflow | IP-005 |
| [`IP-011-replay-debugger-backend-usecase-adapter.md`](IP-011-replay-debugger-backend-usecase-adapter.md) | replay-debugger-backend usecase + api + adapter + adapter-postgres + adapter-clickhouse | pending | axis-workflow | IP-010 |
| [`IP-012-replay-debugger-backend-rest-sdk-app.md`](IP-012-replay-debugger-backend-rest-sdk-app.md) | replay-debugger-backend rest + sdk + app | pending | axis-workflow | IP-011 |
| [`IP-013-observability-slo-manifests.md`](IP-013-observability-slo-manifests.md) | OpenSLO manifests for workflow-engine self-SLOs under `microservices/workflow-engine/slos/` (consumed by observability µservice's promotion gate) | pending | axis-workflow + axis-observability | IP-009 |
| [`IP-014-branch-protection-and-hyperscaler-gates.md`](IP-014-branch-protection-and-hyperscaler-gates.md) | `.github/branch-protection.yaml` updates; `/specs/hyperscaler-gates.json` HG-WF-ENGINE registration; release pointer creation | pending | axis-workflow + ops-sre-reliability | IP-013 |
| [`IP-015-deterministic-replay-lane.md`](IP-015-deterministic-replay-lane.md) | New BLOCKER CI lane `oya-governance-deterministic-replay`; validates that any new workflow spec under the engine passes a deterministic-replay drill | pending | axis-workflow + axis-foundry | IP-012, IP-014 |

Coverage check vs. PRD §"Bounded Contexts" layer table: all 41 crates accounted for (spec-store 9 + execution-engine 12 + state-machine 6 + event-bus 11 + replay-debugger-backend 11; minus 8 redundant counts because some crates serve multiple BCs at composition-root layer = 41 net).

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
oya gate validate lean-a1 --microservice workflow-engine
oya gate validate lean-a2 --microservice workflow-engine
oya gate validate port-location --microservice workflow-engine
oya gate validate layer-correctness --microservice workflow-engine
oya gate validate per-microservice-layout --microservice workflow-engine
oya gate validate statelessness --microservice workflow-engine
oya gate validate shardability --microservice workflow-engine
oya gate validate authority-cohesion
oya gate validate hyperscaler-maturity-claims
```

### Substrate gates introduced by this phase

```bash
oya gate validate deterministic-replay --microservice workflow-engine --spec <path>
oya gate validate workflow-spec-signature-verification --microservice workflow-engine
oya gate validate workflow-event-registry --microservice workflow-engine
oya gate validate ontology-type-registry --microservice workflow-engine
```

### End-to-end drill gates

| Scenario | Command | Pass criterion |
|---|---|---|
| Durable execution restart | scripted e2e: kill engine pod mid-run; restart; verify identical completion | run completes with no duplicated step effects; audit chain intact |
| Deterministic replay | `cargo nextest run -p oya-workflow-engine-replay-debugger-backend-domain --test deterministic_replay` | 100% replay produces identical step sequence |
| 10k concurrent runs | `k6 run tests/load/engine-10k-runs.js` | p99 step execution ≤ 200ms; p99 event-to-action ≤ 500ms |
| Outbox crash recovery | scripted e2e: kill outbox worker mid-flush; verify no event loss | resumed worker emits from last persisted offset |
| Tenant subscription isolation | `cargo nextest run -p oya-workflow-engine-event-bus-domain --test tenant_subscription_isolation` | tenant A cannot read tenant B events |
| Spec signature tampering detected | `cargo nextest run -p oya-workflow-engine-spec-store-domain --test spec_signature_tampering_detected` | tampered spec refused at read time |
| Replay throughput | `cargo nextest run -p oya-workflow-engine-replay-debugger-backend-worker --test replay_throughput` | ≥ 1000 steps/s/worker on a single CPU |

### Workflow + Ontology integration gates

```bash
oya gate validate workflow-event-registry --microservice workflow-engine
oya gate validate ontology-type-registry --microservice workflow-engine
```

## Clean Architecture Compliance

Layer assignments and dependency direction (one representative BC; same shape for the other four BCs):

| Crate (BNF v4.1) | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-workflow-engine-execution-engine-kernel` | `kernel` | (nothing project-internal) | all other layers |
| `oya-workflow-engine-execution-engine-domain` | `domain` | `kernel` | `usecase`, `adapter`, `rest`, `worker`, `app` |
| `oya-workflow-engine-execution-engine-usecase` | `usecase` | `domain`, `kernel` | `adapter`, `rest`, `worker`, `app` |
| `oya-workflow-engine-execution-engine-api` | `api` | `kernel` | other layers |
| `oya-workflow-engine-execution-engine-adapter` | `adapter` | `usecase`, `domain`, `kernel` | `rest`, `worker`, `app` directly |
| `oya-workflow-engine-execution-engine-adapter-postgres` | `adapter-postgres` | `adapter`, `usecase`, `domain`, `kernel` | `rest`, `worker`, `app` directly |
| `oya-workflow-engine-execution-engine-adapter-valkey` | `adapter-valkey` | `adapter`, `usecase`, `domain`, `kernel` | `rest`, `worker`, `app` directly |
| `oya-workflow-engine-execution-engine-rest` | `rest` | `usecase`, `api`, `domain`, `kernel` | `adapter*` directly (uses ports) |
| `oya-workflow-engine-execution-engine-worker` | `worker` | `usecase`, `api`, `domain`, `kernel` | `adapter*` directly (uses ports) |
| `oya-workflow-engine-execution-engine-sdk` | `sdk` | `api`, `kernel` | adapter/rest/worker/app |
| `oya-workflow-engine-execution-engine-app` | `app` | (composition-root wiring only) | none — but only wiring |

Port traits live exclusively in `*-kernel` crates; implementations exclusively in `*-adapter*` crates. Domain calls through ports; domain never imports adapter.

Cross-product integration check: this phase introduces NO direct imports between `workflow-engine` and any other product µservice's crates. All cross-product data flow uses the event-bus (this µservice IS the orchestration adapter) and Ontology reads/writes.

## ChangeSet Contract per IP

Every IP in this phase emits a ChangeSet per ADR-0110 (claimable + verifiable + bundleable + promotable). The minimum ChangeSet payload per IP is written at `microservices/workflow-engine/evidence/multispectrum/<change_id>-<unix_ts>.json` before opening the pull request against `dev`:

```json
{
  "change_id": "ULID",
  "ip_id": "IP-NNN-<slug>",
  "microservice": "workflow-engine",
  "milestone": "M02b-substrate-ready",
  "phase": "P01-durable-execution-substrate",
  "claim_paths": ["microservices/workflow-engine/src/crates/<crate>/**", "..."],
  "intent": "<one-line>",
  "spec_refs": ["microservices/workflow-engine/PRD.md§<section>", "/specs/microservices/workflow.json§<section>"],
  "acceptance_lanes_green": ["cargo-check", "cargo-build", "cargo-clippy", "cargo-nextest", "cargo-deny", "lean-a1", "lean-a2", "per-microservice-layout", "deterministic-replay"],
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
| kernel crate (`*-kernel`) | 1 per public type + 1 per port trait | 0 (pure) | 0 | 90% line; 80% branch |
| domain crate (`*-domain`) | 1 per public function + property tests for math + deterministic-replay invariant | 0 | 0 | 95% line; 90% branch |
| usecase crate (`*-usecase`) | 1 per use case (happy + 2 sad paths) | ≥ 3 against mocked ports | 0 | 90% line; 80% branch |
| adapter crate (`*-adapter*`) | 1 per port-impl method | ≥ 2 against real backend (Postgres / Valkey / ClickHouse test container) | 0 | 85% line; 75% branch |
| rest crate (`*-rest`) | 1 per route (happy + auth-fail + tenant-mismatch) | ≥ 2 cross-route flows | 1 per route via REST integration test | 85% line; 75% branch |
| worker crate (`*-worker`) | 1 per orchestration arm | ≥ 1 long-lived loop integration test | 1 e2e (durable-execution restart drill) | 85% line; 75% branch |
| sdk crate (`*-sdk`) | 1 per public client method (happy + retry + auth-fail) | ≥ 2 against rest crate | 0 | 90% line; 80% branch |
| app crate (`*-app`) | composition-root smoke tests | 0 (delegates to worker/rest tests) | 1 startup-and-shutdown smoke | 60% line (mostly wiring) |
| IaC IPs (Helm / OpenTofu) | n/a | ≥ 1 helm-install + helm-test smoke per chart | 1 against kind/k3d cluster | n/a |

Enforced by:
- `cargo nextest run --workspace --all-features` exits 0.
- `cargo llvm-cov --workspace --fail-under-lines <threshold>` exits 0.
- Per-IP CI workflow specifies the per-IP thresholds in its `[acceptance_lanes]` frontmatter.

## branch-protection.yaml diff preview

IP-014 (branch-protection + hyperscaler-gates) updates `.github/branch-protection.yaml` with:

```yaml
branches:
  dev:
    required_status_checks:
      # existing checks (from observability phase) plus:
      - oya-governance-deterministic-replay              # NEW; from this phase's IP-015
      - oya-governance-workflow-spec-signature-verification  # NEW; from this phase's IP-002 + IP-008
      - oya-governance-workflow-event-registry           # NEW; ensures every emitted event is registered

  staging:
    required_status_checks:
      # ADDED by this phase:
      - oya-governance-deterministic-replay
      - oya-governance-promotion-readiness                       # already added by observability phase; engine respects it

  # ADDED — pattern-based protection for workflow-engine release pointers
  ? release/workflow-engine/staging
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    require_signed_tags: true
    required_status_checks:
      - oya-governance-promotion-readiness

  ? release/workflow-engine/production
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    require_signed_tags: true
    required_status_checks:
      - oya-governance-promotion-readiness
```

## Git/Jenkins governance handoff

Per ADR-0363 and the current repo AGENTS contract, this phase uses one isolated branch/worktree per implementation lane, a pull request against `dev`, Jenkins CI, `oya gate`, and `oya verify`. Grit, ICM, and retired `oya vcs` primitives are explicitly NOT used.

```bash
# Create an isolated branch/worktree before beginning each IP.
git worktree add ../oyatie-workflow-engine-<ip-id> -b feature/workflow-engine-<ip-id> dev

# Verify after each IP's acceptance gates pass.
cargo run -p oya-dev-cli -- gate run-all --ci-required
cargo run -p oya-dev-cli -- verify --ci-required

# Open a PR against dev; Jenkins CI and reviewer APPROVE provide merge readiness.
git push -u origin feature/workflow-engine-<ip-id>

# Promotion is Jenkins/ArgoCD-governed after merge, using signed image digests.
```

Multispectrum evidence per docs/AGENTS.md §changeset: each IP emits `microservices/workflow-engine/evidence/multispectrum/<change_id>-<unix_ts>.json` per `/specs/multispectrum-review.json` v2.4.0.

## References

- ADR-0035 (Bominal): Workflow engine (hybrid state machine + DAG); inherited.
- ADR-0103 (Bominal): Workflow hexagonal migration; inherited.
- ADR-0148 (Bominal): Workflow engine (extended); inherited.
- ADR-0028 (Bominal): Audit chain; inherited.
- ADR-0056: BNF v4.1.
- ADR-0105: 13-layer enum.
- ADR-0110: ChangeSet state machine.
- ADR-0116: Retire external agent-coordination tooling.
- ADR-0123: Hyperscaler maturity claim gate.
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout + workflow unbundle.
- `/specs/microservices/workflow.json`.
- `/specs/per-microservice-flat-layout.json`.
- `microservices/workflow-engine/PRD.md`.
- Memory: `feedback_workflow_is_shared.md`, `feedback_workflow_studio_scope.md`, `feedback_workflow_objectgraph_adapter_layer.md`, `feedback_milestone_phase_hierarchy.md`, `feedback_naming_justification.md`, `feedback_clean_architecture_requirements.md`, `feedback_quality_performance_scalability_bar.md`.
