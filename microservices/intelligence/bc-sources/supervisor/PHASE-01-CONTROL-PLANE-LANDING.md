---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P01-control-plane-landing
status: Active
entry_gate: |
  ADR-0131 (foundry split) + PRD-foundry-supervisor + /specs/foundry-supervisor-control-plane.json
  published; Cargo workspace ready to accept the new crates under
  microservices/intelligence-supervisor/src/crates/; foundry-runtime + foundry-providers
  scaffolds present (PRD-stub acceptable; binary build optional).
exit_gate: |
  All 15 IPs merged; oya-foundry-supervisor-{kill-switch-coverage,deployment-rate,
  autonomy-violation-rate} dashboards live; AC-01..AC-10 pass; HG-FND-SUP gate green;
  cargo nextest --workspace exits 0; oya gate validate per-microservice-layout
  --microservice foundry-supervisor exits 0; oya gate validate authority-cohesion
  exits 0; supervisor's own OpenSLO manifest at microservices/intelligence-supervisor/slos/
  authored and observed by observability µservice.
depends_on:
  - milestone: M01-foundation
    phase: prior phases (per master-plan-sequencing)
    reason: workspace + branch-protection + governance lanes precede the control plane authoring
  - microservice: foundry-runtime
    phase: PHASE-01 scaffold
    reason: supervisor manages runtime workers; ports must exist on the runtime side
  - microservice: observability
    phase: PHASE-01 (already landed)
    reason: SLO gate consumes supervision events; supervisor consumes EligibilityChanged
owner_team: axis-foundry-control-plane
related_adrs: [ADR-0024, ADR-0105, ADR-0123, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/foundry-supervisor-control-plane.json, /specs/per-microservice-flat-layout.json]
date: 2026-05-17
doc_status: published
---

# P01-control-plane-landing: Land the Foundry supervisor control plane end-to-end

## Purpose

Ship the full ADR-0131 §"Foundry split" supervisor scope — Layer-A substrate (Postgres, Valkey, Kubernetes Operator pattern), all five BCs (agent-fleet-lifecycle, capability-deployment, autonomy-policy-enforcement, supervision-event-bus, kill-switch-circuit-breaker), supervisor self-SLOs authored, integration with `observability` SLO gate (ADR-0139) and `foundry-evidence` audit chain wired.

Hyperscaler-grade in every practice: the supervisor's own kill-switch engage is itself an SLO observed by `observability`; the µservice ships dashboards on day 1 (deployment-rate, kill-switch-coverage, autonomy-violation-rate) per ADR-0133.

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected |
|---|---|---|
| `foundry-supervisor` | all 5 BCs | All under `microservices/intelligence-supervisor/` per ADR-0131 |

Plus cross-cutting:
- `microservices/intelligence-supervisor/slos/{kill-switch-engage.openslo.yaml, deployment-admit.openslo.yaml, supervision-event-lag.openslo.yaml, autonomy-policy-eval.openslo.yaml}` — supervisor's own SLO manifests
- `.github/branch-protection.yaml` — add HG-FND-SUP-claim, oya-foundry-supervisor-canary-rollout-gated lanes
- `Cargo.toml` (workspace) — register new crates
- `/specs/hyperscaler-gates.json` — register HG-FND-SUP per ADR-0123

### Out-of-scope

- Migration of any legacy `foundry-*` crate sets into this µservice — owned by IP-M01-MIGR-FND-3 per ADR-0131.
- `foundry-runtime` execution-plane authoring — owned by its own µservice's PHASE-01.
- `foundry-providers` vendor SDKs — owned by its own µservice.
- Cross-region (multi-pack) federation of fleet state — scheduled-for-distinct-tracked-work to a subsequent-to-M01-completion ADR; M01 ships per-pack pinning per `policy/data-residency.md`.

## Implementation Plans

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-postgres-layer-a-iac.md`](IP-001-postgres-layer-a-iac.md) | Helm chart for HA Postgres (master + replica per pack region); OpenBao-managed credentials | pending | ops-sre-reliability | — |
| [`IP-002-redis-layer-a-iac.md`](IP-002-redis-layer-a-iac.md) | Helm chart for Valkey Cluster (3 shards × 2 replicas per pack region); kill-switch state + supervision-event-bus stream | pending | ops-sre-reliability | — |
| [`IP-003-k8s-operator-iac.md`](IP-003-k8s-operator-iac.md) | Kubernetes Operator deployment (kube-rs based controller-runtime); CRDs (`Agent`, `AgentDeployment`, `AutonomyPolicy`, `KillSwitch`); RBAC | pending | axis-foundry-control-plane | IP-001, IP-002 |
| [`IP-004-agent-fleet-lifecycle-kernel.md`](IP-004-agent-fleet-lifecycle-kernel.md) | Kernel crate: port traits + entities | pending | axis-foundry-control-plane | — |
| [`IP-005-autonomy-policy-enforcement.md`](IP-005-autonomy-policy-enforcement.md) | Cedar evaluator + tenant-entitlement store; default-deny; per-invocation precondition | pending | ops-security + axis-foundry-control-plane | IP-004 |
| [`IP-006-capability-deployment.md`](IP-006-capability-deployment.md) | Admit + canary rollout + roll-forward + roll-back; integrates `EligibilityChanged` gate | pending | axis-foundry-control-plane | IP-003, IP-005 |
| [`IP-007-supervision-event-bus.md`](IP-007-supervision-event-bus.md) | AMQP + Valkey Streams (Redis wire-compat) substrate; per-event Ed25519 signature; subscriber registration | pending | axis-foundry-control-plane | IP-002, IP-004 |
| [`IP-008-kill-switch-engage-state.md`](IP-008-kill-switch-engage-state.md) | Kill-switch state model in Valkey; CRD watch fan-out; engage/disengage with audit-chain | pending | axis-foundry-control-plane | IP-002, IP-003, IP-007 |
| [`IP-009-kill-switch-propagation.md`](IP-009-kill-switch-propagation.md) | Sub-second propagation to in-flight workers via runtime-side handshake; integration test ≤ 1s p99 | pending | axis-foundry-control-plane | IP-008 |
| [`IP-010-fleet-state-postgres-adapter.md`](IP-010-fleet-state-postgres-adapter.md) | Postgres-backed fleet-state repository + tenant-shard routing | pending | axis-foundry-control-plane | IP-001, IP-004 |
| [`IP-011-rest-api.md`](IP-011-rest-api.md) | REST surface per `contracts/openapi/foundry-supervisor.yaml`; OIDC + Cedar gating | pending | axis-foundry-control-plane | IP-005, IP-006, IP-008, IP-010 |
| [`IP-012-supervisor-self-slos.md`](IP-012-supervisor-self-slos.md) | Author OpenSLO manifests at `microservices/intelligence-supervisor/slos/`; observability gate observes | pending | axis-foundry-control-plane + axis-observability | IP-011 |
| [`IP-013-sdk-rust-and-ts.md`](IP-013-sdk-rust-and-ts.md) | Rust SDK first-party + TypeScript generated; published to internal registry | pending | axis-foundry-control-plane | IP-011 |
| [`IP-014-app-composition-root.md`](IP-014-app-composition-root.md) | App binary wiring all BCs; lease-leadership election; mTLS + SPIFFE; OpenBao integration | pending | axis-foundry-control-plane | IP-001..IP-013 |
| [`IP-015-e2e-drills-and-dashboards.md`](IP-015-e2e-drills-and-dashboards.md) | End-to-end drills (AC-02 kill-switch latency, AC-03 canary, AC-04 autonomy refusal, AC-06 drain, AC-07/08 failover) + Grafana dashboards | pending | axis-foundry-control-plane + ops-sre-reliability | IP-014 |

## Acceptance Gates

### Cargo / CI gates

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
oya gate validate lean-a1                  --microservice foundry-supervisor
oya gate validate lean-a2                  --microservice foundry-supervisor
oya gate validate port-location            --microservice foundry-supervisor
oya gate validate layer-correctness        --microservice foundry-supervisor
oya gate validate per-microservice-layout  --microservice foundry-supervisor
oya gate validate statelessness            --microservice foundry-supervisor
oya gate validate shardability             --microservice foundry-supervisor
oya gate validate authority-cohesion
oya gate validate cedar-fragment-coverage  --microservice foundry-supervisor
oya gate validate hyperscaler-maturity-claims
```

### End-to-end drill gates

| Scenario | Command | Pass criterion |
|---|---|---|
| Kill-switch latency | `cargo nextest run -p oya-foundry-supervisor-kill-switch-circuit-breaker-worker --test kill_switch_latency` | p99 ≤ 1 s; p999 ≤ 2 s; sample size ≥ 100k engage cycles |
| Canary rollout gated by observability | `cargo nextest run -p oya-foundry-supervisor-capability-deployment-worker --test canary_rollout_gated` | rollout pauses at held verdict; rolls back on rollback verdict |
| Autonomy tier refusal | `cargo nextest run -p oya-foundry-supervisor-autonomy-policy-enforcement-usecase --test tier_escalation_refused` | Cedar denies; audit-chain emits `AutonomyViolated` |
| Drain with zero loss | `cargo nextest run -p oya-foundry-supervisor-agent-fleet-lifecycle-worker --test drain_zero_loss` | drain completes; in-flight reach `success`; no `AgentEvicted{reason=lost}` |
| Postgres failover | scripted chaos: kill master pod | control-plane available ≤ 30 s |
| Valkey failover | scripted chaos: kill one replica | kill-switch engage p99 stays ≤ 1 s |

## Clean Architecture Compliance

| Crate | Layer | Imports | Forbidden |
|---|---|---|---|
| `*-kernel` | `kernel` | (none project-internal) | all other layers |
| `*-domain` | `domain` | `kernel` | `usecase`, `adapter*`, `rest`, `worker`, `app` |
| `*-usecase` | `usecase` | `domain`, `kernel` | `adapter*`, `rest`, `worker`, `app` |
| `*-api` | `api` | `kernel` | `domain`, `usecase`, `adapter*`, `rest`, `worker`, `app` |
| `*-adapter` | `adapter` | `usecase`, `domain`, `kernel`, `api` | `rest`, `worker`, `app` |
| `*-adapter-postgres` | `adapter` (backend-qualified) | same | same |
| `*-adapter-k8s-operator` | `adapter` (backend-qualified) | same | same |
| `*-rest` | `rest` | `usecase`, `domain`, `kernel`, `api` | `adapter*` directly (uses ports) |
| `*-worker` | `worker` | `usecase`, `domain`, `kernel`, `api` | `adapter*` directly (uses ports) |
| `*-sdk` | `sdk` | `api`, `kernel` | `adapter*`, `usecase`, `domain` |
| `*-app` | `app` | composition-root wiring only | none — but only wiring |

Cross-product imports: zero. All cross-µservice flow goes through Workflow events + Ontology reads/writes.

## ChangeSet Contract per IP

Each IP emits a ChangeSet per ADR-0110 at `microservices/intelligence-supervisor/evidence/multispectrum/<change_id>-<unix_ts>.json` validated by `oya-governance-multispectrum-evidence` lane.

## Per-IP Test Coverage Threshold

| IP class | Min unit | Min integration | Min e2e | Coverage threshold |
|---|---|---|---|---|
| kernel | 1 per public type + 1 per port | 0 | 0 | 90 % line / 80 % branch |
| domain | 1 per public fn + property tests | 0 | 0 | 95 % line / 90 % branch |
| usecase | 1 per use-case (happy + 2 sad) | ≥ 3 against mocks | 0 | 90 % line / 80 % branch |
| adapter / adapter-postgres / adapter-k8s-operator | 1 per port-impl method | ≥ 2 against real backend (testcontainers Postgres + Valkey + kind k8s) | 0 | 85 % line / 75 % branch |
| rest | 1 per route (happy + auth-fail + cedar-deny) | ≥ 2 cross-route flows | 1 per route | 85 % line / 75 % branch |
| worker | 1 per orchestration arm | ≥ 1 long-lived loop test | 1 e2e | 85 % line / 75 % branch |
| sdk | 1 per public method | ≥ 2 against rest | 0 | 90 % line / 80 % branch |
| app | composition smoke | 0 | 1 startup+shutdown | 60 % line |
| IaC IPs | n/a | helm-install + helm-test smoke per chart | 1 against kind | n/a |

## Oya VCS Symbol Locks

```bash
cargo run -p oya-dev-cli -- vcs claim \
  --agent <agent-id> \
  --intent "<IP-NNN-slug>: <one-line intent>" \
  --paths "microservices/intelligence-supervisor/src/crates/<crate>/**"

cargo run -p oya-dev-cli -- vcs verify --agent <agent-id> --changeset <id>
cargo run -p oya-dev-cli -- vcs done   --agent <agent-id> --changeset <id>
cargo run -p oya-dev-cli -- vcs promote --changeset <id>
```

## References

- ADR-0024 (eval harness contract; supervisor consumes EvalRegression).
- ADR-0105 (13-layer enum); ADR-0106 (application→usecase).
- ADR-0123 (hyperscaler maturity claim gate; HG-FND-SUP).
- ADR-0139 (SLO gate; supervisor consumes EligibilityChanged).
- ADR-0131 §"Foundry split" (this µservice's scope).
- ADR-0132 (product-suite-and-bundle dissolution).
- ADR-0133 (industry-best-practice conformance).
- ADR-0140 (Cedar policy enforcement).
- `microservices/intelligence-supervisor/PRD.md`.
- `microservices/observability/PRD.md` (consumer of supervisor events).
- Kubernetes Operator pattern — `kubernetes.io/docs/concepts/extend-kubernetes/operator/`.
- kube-rs — `kube.rs`.
- Cedar v4 — `cedarpolicy.com`.
