---
doc_class: PhaseSpec
template_id: TPL-PHASE
milestone: M01-foundation
phase: P01-eval-harness-substrate
microservice: foundry-eval
status: Accepted
date: 2026-05-17
owner_team: axis-foundry
deciders: council-architecture, axis-foundry, ops-sre-reliability, ops-security
related_adrs: [ADR-0024, ADR-0026, ADR-0056, ADR-0105, ADR-0106, ADR-0130, ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/per-microservice-flat-layout.json]
doc_status: published
---

# Phase-01: Eval Harness Substrate (foundry-eval M01)

## Intent

Stand up the foundry-eval µservice end-to-end as a Layer-A + Layer-B substrate that gates capability publish, runs nightly eval, A/B-routes provider preference, replays past production traces with deterministic divergence assertions, and emits cutover-eligibility verdicts to `foundry-providers`. The phase delivers the eval-runner BC's full layer stack and adapter set plus the bootstrap of eval-set-registry, parity-analyzer, replay-engine, and golden-output-store. Other BCs receive scaffolding now and full layer-stacks in M02.

## Scope

In-scope (M01-P01):

- Eval-set authoring path: `microservices/foundry-eval/eval-sets/<capability>/v<n>.evalset.yaml` + Cosign signature.
- Eval-runner full layer stack (kernel + domain + usecase + api + adapter + adapter-s3 + adapter-gpu + rest + worker + sdk + app).
- Bootstrap of eval-set-registry, parity-analyzer, replay-engine, golden-output-store (kernel + domain + usecase + adapter + app each; full layer stack in M02).
- Layer-A IaC: GPU runner pool Helm chart, Postgres Helm chart, ClickHouse Helm chart, golden-output-store (MinIO + KMS) Helm chart.
- Layer-B CI lanes: `foundry-eval-coverage`, `foundry-eval-adversarial-coverage`, `foundry-eval-linguistic-coverage`, `foundry-eval-nightly`, `foundry-eval-route-ab`, `foundry-eval-replay`.
- 3 capabilities (eval-run, parity-compare, replay-execute) + 3 dashboards + 6 runbooks + 6 policies + 3 contracts.
- DSR cascade integration: per-subject DEK shred surface tested end-to-end.
- EU AI Act §15 + §17 evidence-schema emission on every eval-run.

Out-of-scope (M01-P01; scheduled-for-distinct-tracked-work to M02-P02):

- Multi-arm A/B (> 2 routes) parity analysis.
- LLM-as-judge grader pluggability (M01 ships one judge per capability; rotation policy in M02).
- Cross-region eval-set replication (M01 is single-region per pack).
- Tenant-authored eval-sets (capabilities only in M01; tenant authoring in M02).

## Per-IP Test Coverage Threshold

Per-class minima (kernel / domain / usecase / api / adapter / rest / worker / sdk / app):

| Class | Coverage | Tests required |
|---|---|---|
| kernel | 90% line / 80% branch | port-trait sealedness; entity serde; data_class annotations present |
| domain | 95% line / 90% branch | pure-function regression on aggregate math; divergence-tolerance arithmetic |
| usecase | 90% line / 80% branch | orchestrator scenarios under mocked ports |
| api | 90% line | typed-contract roundtrip |
| adapter | 85% line | mocked I/O round-trip + integration against test substrate |
| rest | 90% line | handler-route happy + error paths |
| worker | 85% line | cron-tick + queue-depth scenarios |
| sdk | 90% line | client-side roundtrip against rest |
| app | 80% line | composition-root smoke |

## Implementation Plan DAG

```text
IP-001  Layer-A GPU runner pool Helm chart
IP-002  Layer-A Postgres + ClickHouse + golden-store Helm charts (bundle)
IP-003  eval-runner kernel
IP-004  eval-runner domain
IP-005  eval-runner usecase
IP-006  eval-runner api
IP-007  eval-runner adapter (filesystem eval-set reader; provider-route resolver)
IP-008  eval-runner adapter-s3 (golden-output read; eval-run store)
IP-009  eval-runner adapter-gpu (Kubernetes Job dispatcher; CUDA shim)
IP-010  eval-runner rest
IP-011  eval-runner worker (nightly orchestrator; on-demand runs)
IP-012  eval-runner sdk (Rust client; TS bindings via wasm-bindgen)
IP-013  eval-runner app (composition root)
IP-014  parity-analyzer kernel + domain + adapter-clickhouse + usecase
IP-015  replay-engine kernel + domain + adapter-s3 + usecase + worker
```

Dependency order:
- IP-003 (kernel) ⇒ IP-004 (domain) ⇒ IP-005 (usecase) ⇒ IP-006 (api) ⇒ {IP-007, IP-008, IP-009} ⇒ IP-010 (rest) ⇒ IP-011 (worker) ⇒ IP-012 (sdk) ⇒ IP-013 (app).
- IP-001, IP-002 (Layer-A IaC) parallel to IP-003..IP-013.
- IP-014, IP-015 parallel after IP-005 (need usecase port-traits).

## Acceptance Gates

Each IP must green:

```bash
cargo check --workspace
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo nextest run --workspace
cargo deny check
cargo doc --workspace --no-deps
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry-eval
cargo run -p oya-dev-cli -- gate validate lean-a1 --microservice foundry-eval
cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice foundry-eval
cargo run -p oya-dev-cli -- gate validate port-location --microservice foundry-eval
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice foundry-eval
cargo run -p oya-dev-cli -- gate validate data-class --microservice foundry-eval
cargo run -p oya-dev-cli -- gate validate statelessness --microservice foundry-eval
cargo run -p oya-dev-cli -- gate validate shardability --microservice foundry-eval
```

Phase-complete additionally requires:

```bash
cargo run -p oya-dev-cli -- gate validate foundry-eval-coverage
cargo run -p oya-dev-cli -- gate validate foundry-eval-adversarial-coverage
cargo run -p oya-dev-cli -- gate validate foundry-eval-linguistic-coverage
cargo run -p oya-dev-cli -- gate validate foundry-eval-replay-determinism
cargo run -p oya-dev-cli -- gate validate authority-cohesion  # HG-FE registered
```

Per ADR-0130 SLO gate: `release/foundry-eval/staging` advances only when SLI burn-rate is green; `release/foundry-eval/production` advances only after staging burn-rate clean for 24h.

## Halt Conditions

- Replay-determinism p99 divergence exceeds 100ms on the deterministic-seed cohort.
- Publish-gate latency p99 exceeds 1s.
- Any cross-product import detected (LEAN-A2 fail).
- Any port outside kernel.
- Any I/O reachable from kernel.

## References

- ADR-0024 (eval harness + replay design)
- ADR-0026 (in-house model substrate roadmap; cutover gate)
- ADR-0130 (agentic SLO-gated promotion; inherited)
- ADR-0131 (per-microservice flat layout)
- ADR-0133 (industry best-practice conformance program; HG-FE)
- microservices/foundry-eval/PRD.md
- /specs/per-microservice-flat-layout.json
