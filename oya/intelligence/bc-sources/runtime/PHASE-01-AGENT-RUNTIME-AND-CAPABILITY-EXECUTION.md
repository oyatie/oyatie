---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P01-agent-runtime-and-capability-execution
status: Active
entry_gate: |
  ADR-0025 + ADR-0131 accepted; foundry-supervisor capability registry contract published;
  foundry-providers + foundry-guardrails + foundry-evidence siblings have at least their PRD merged;
  cargo workspace ready to accept the 35 new crates under microservices/intelligence-runtime/src/crates/.
exit_gate: |
  All 15 IPs merged; capability dispatch p99 ≤50ms verified by load test; session-state hot read
  p99 ≤10ms verified; autonomy-ceiling refusal end-to-end test green; runtime-pool drain verified;
  cargo nextest run --workspace exits 0; oya gate validate per-microservice-layout
  --microservice foundry-runtime exits 0; oya gate validate authority-cohesion exits 0;
  HG-FR gate in /specs/hyperscaler-gates.json registers green; OpenSLO manifests merged + verdict
  eligible at staging tier per ADR-0139.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion (observability)
    reason: SLO gate authority must precede any new µservice's first promotion to staging
owner_team: axis-foundry-runtime
related_adrs: [ADR-0022, ADR-0024, ADR-0025, ADR-0056, ADR-0105, ADR-0106, ADR-0110, ADR-0123, ADR-0139, ADR-0131]
related_specs: [/specs/agent-operating-contract.json, /specs/per-microservice-flat-layout.json, /specs/hyperscaler-gates.json]
date: 2026-05-17
doc_status: published
---

# P01-agent-runtime-and-capability-execution: Land the runtime end-to-end

## Purpose

This phase ships the full ADR-0025 design — the agent runtime + capability execution plane of the Foundry split per ADR-0131. The runtime hosts capability invocations, manages session state, dispatches through siblings (`foundry-providers`, `foundry-guardrails`, `foundry-evidence`), and emits invocation telemetry. It is delivered as one phase in M01-foundation because every Foundry-class product (Workflow Studio first; future hero products subsequently) depends on a working agent runtime.

This phase advances master-plan principles:
- Hyperscaler-grade in every practice (AWS Bedrock + GCP Vertex + Azure AI Foundry + LangServe + OpenAI Assistants competitive posture).
- Nothing scheduled-for-distinct-tracked-work (every FUTURE-marked seam in the prior monolithic Foundry runtime sketch is decommissioned by this phase).
- No silent regression (autonomy-ceiling violation surfaces auto-blocks dispatch).
- Per-microservice flat layout (this phase is authored natively under ADR-0131 alongside observability).

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `foundry-runtime` | `capability-executor`, `session-state`, `invocation-orchestrator`, `runtime-pool`, `capability-registry-cache` | All under `microservices/intelligence-runtime/` per ADR-0131 | `oya-foundry-runtime-{capability-executor,session-state,invocation-orchestrator,runtime-pool,capability-registry-cache}-{layer}` (35 crates total per PRD) |

Plus these repo-wide artifacts (cross-cutting per ADR-0131):
- `Cargo.toml` (workspace) — register the 35 new crates under `microservices/intelligence-runtime/src/crates/`.
- `/specs/hyperscaler-gates.json` — register HG-FR gate per ADR-0123.
- `.github/branch-protection.yaml` — add the additive lanes `foundry-runtime-iac-smoke` + `foundry-runtime-load-latency` to `dev` and `staging`.

Naming justifications for the new crate families are in `microservices/intelligence-runtime/PRD.md` §"Bounded Contexts".

### Out-of-scope

- Tenant-supplied custom code execution (sandboxed WASM or out-of-process container). Deferred per PRD Open Question 5; M01 supports descriptor-only capabilities (the runtime dispatches through providers + guardrails, no tenant-side code path).
- Cross-pack session migration. Per PRD §"Horizontal Scalability", cross-pack is forbidden; migration tooling is subsequent-to-M01-completion.
- Provider-side credential rotation orchestration. Owned by `foundry-providers` µservice's own phase.
- Eval-harness orchestration. Owned by `foundry-evidence` µservice (consumes invocation events emitted here).
- Capability authoring UX. Owned by Workflow Studio product surface.

## Implementation Plans

Ordered list. Each IP is an executable ChangeSet under this phase folder. Dependencies inline.

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-runtime-cluster-iac.md`](IP-001-runtime-cluster-iac.md) | Helm/Kustomize charts for the runtime-pool, Valkey 8.1 (Redis wire-compat), Postgres 16 LTS under `microservices/intelligence-runtime/iac/helm/` | pending | ops-sre-reliability + axis-foundry-runtime | — |
| [`IP-002-redis-and-postgres-baseline.md`](IP-002-redis-and-postgres-baseline.md) | Valkey cluster + Postgres mirror schema + OpenBao secret references | pending | ops-sre-reliability | IP-001 |
| [`IP-003-capability-executor-kernel.md`](IP-003-capability-executor-kernel.md) | kernel layer crate: port traits + entities + errors | pending | axis-foundry-runtime | IP-002 |
| [`IP-004-capability-executor-domain-and-usecase.md`](IP-004-capability-executor-domain-and-usecase.md) | domain (pure math) + usecase (orchestrator) crates for executor | pending | axis-foundry-runtime | IP-003 |
| [`IP-005-capability-registry-cache-stack.md`](IP-005-capability-registry-cache-stack.md) | kernel + usecase + api + adapter + adapter-postgres + worker + app crates for registry-cache | pending | axis-foundry-runtime | IP-003 |
| [`IP-006-session-state-stack.md`](IP-006-session-state-stack.md) | kernel + domain + usecase + api + adapter + adapter-redis + adapter-postgres + sdk + app for session-state | pending | axis-foundry-runtime | IP-002 |
| [`IP-007-invocation-orchestrator-stack.md`](IP-007-invocation-orchestrator-stack.md) | kernel + domain + usecase + api + adapter + worker + app for orchestrator | pending | axis-foundry-runtime | IP-004, IP-006 |
| [`IP-008-runtime-pool-stack.md`](IP-008-runtime-pool-stack.md) | kernel + usecase + api + adapter + worker + app for runtime-pool | pending | ops-sre-reliability + axis-foundry-runtime | IP-001 |
| [`IP-009-capability-executor-api-and-rest.md`](IP-009-capability-executor-api-and-rest.md) | api + rest crates exposing the OpenAPI contract; mTLS to sibling µservices | pending | axis-foundry-runtime | IP-004, IP-007 |
| [`IP-010-capability-executor-sdk.md`](IP-010-capability-executor-sdk.md) | Rust SDK crate for programmatic capability invocation | pending | axis-foundry-runtime | IP-009 |
| [`IP-011-capability-executor-app.md`](IP-011-capability-executor-app.md) | composition root binary wiring executor + orchestrator + pool + cache + session-state adapters | pending | axis-foundry-runtime | IP-005, IP-006, IP-007, IP-008, IP-009 |
| [`IP-012-autonomy-ceiling-gate.md`](IP-012-autonomy-ceiling-gate.md) | AutonomyGate wiring: tenancy lookup + per-tenant ceiling read + dispatch refusal + audit emission | pending | axis-foundry-runtime + ops-security | IP-004 |
| [`IP-013-dsr-cascade-session-handler.md`](IP-013-dsr-cascade-session-handler.md) | TenantDsrCascade event consumer in session-state worker | pending | axis-foundry-runtime + council-privacy | IP-006 |
| [`IP-014-runtime-self-slo-manifests.md`](IP-014-runtime-self-slo-manifests.md) | OpenSLO manifests for availability/latency/correctness/freshness at `slos/` per ADR-0139 | pending | axis-foundry-runtime + axis-observability | IP-011 |
| [`IP-015-hg-fr-hyperscaler-gate-registration.md`](IP-015-hg-fr-hyperscaler-gate-registration.md) | Register HG-FR in `/specs/hyperscaler-gates.json` per ADR-0123; competitor parity assertions | pending | axis-foundry-runtime + council-architecture | IP-014 |

Coverage check vs. PRD §"Bounded Contexts": all 35 crates land via IP-003 through IP-011 (executor=8, registry-cache=7, session-state=9, orchestrator=7, pool=6, app crates included). IaC + Redis/Postgres seed by IP-001+IP-002. Cross-cutting policy bring-up by IP-012 (autonomy) and IP-013 (DSR). Substrate authority via IP-014 (SLO) and IP-015 (HG).

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
oya gate validate lean-a1 --microservice foundry-runtime
oya gate validate lean-a2 --microservice foundry-runtime
oya gate validate port-location --microservice foundry-runtime
oya gate validate layer-correctness --microservice foundry-runtime
oya gate validate per-microservice-layout --microservice foundry-runtime
oya gate validate statelessness --microservice foundry-runtime
oya gate validate shardability --microservice foundry-runtime
oya gate validate authority-cohesion
oya gate validate hyperscaler-maturity-claims
```

### Substrate gates introduced by this phase

```bash
oya gate validate foundry-runtime-iac-smoke
oya gate validate foundry-runtime-load-latency --target dispatch_p99=50ms --target session_hot_p99=10ms
```

### End-to-end drill gates

| Scenario | Command | Pass criterion |
|---|---|---|
| Dispatch happy path | `cargo nextest run -p oya-foundry-runtime-capability-executor-usecase --test dispatch_happy_path` | verdict `completed`; event emitted; latency ≤50ms |
| Autonomy refusal | `cargo nextest run -p oya-foundry-runtime-capability-executor-usecase --test autonomy_refusal` | tier-T3 dispatch on tenant capped at T2 returns refusal + `AutonomyViolationDetected` |
| Session hot read | `cargo nextest run -p oya-foundry-runtime-session-state-adapter-redis --test redis_hot_read_p99` | p99 ≤10ms under synthetic load |
| Pod drain | scripted e2e: inject in-flight invocations + initiate drain | invocations parked + pod retired ≤60s; zero data loss |
| Registry hot-reload | scripted e2e: PR-merge a capability descriptor change | runtime picks up new version ≤30s |
| Provider-credential isolation | `tests/e2e/provider-credential-isolation.rs` | no provider secret materialised in runtime pod |
| Cross-tenant refusal | `tests/integration/cross-tenant-refusal.rs` | 403 + Cedar audit |
| DSR cascade | `tests/e2e/dsr-cascade-session.rs` | soft-delete completes within synthetic-30-day clock |

### Workflow + Ontology integration gates

```bash
oya gate validate workflow-event-registry --microservice foundry-runtime
oya gate validate ontology-type-registry --microservice foundry-runtime
```

## Clean Architecture Compliance

Layer assignments and dependency direction (one row per BC kernel; downstream crates follow PRD §Bounded Contexts):

| Crate (BNF v4.1) | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-foundry-runtime-capability-executor-kernel` | `kernel` | (nothing project-internal) | all other layers |
| `oya-foundry-runtime-capability-executor-domain` | `domain` | `kernel` | `usecase`, `adapter`, `rest`, `sdk`, `app` |
| `oya-foundry-runtime-capability-executor-usecase` | `usecase` | `domain`, `kernel` | `adapter`, `rest`, `sdk`, `app` |
| `oya-foundry-runtime-capability-executor-api` | `api` | `kernel` | downstream layers |
| `oya-foundry-runtime-capability-executor-adapter` | `adapter` | `usecase`, `domain`, `kernel`, `api` | `rest`, `sdk`, `app` directly |
| `oya-foundry-runtime-capability-executor-rest` | `rest` | `usecase`, `domain`, `kernel`, `api` | `adapter` directly (uses ports) |
| `oya-foundry-runtime-capability-executor-sdk` | `sdk` | `api`, `kernel` | downstream layers |
| `oya-foundry-runtime-capability-executor-app` | `app` | (composition-root wiring only) | none — but only wiring |
| `oya-foundry-runtime-session-state-{kernel,...,adapter-redis,adapter-postgres,...}` | per ADR-0105 enum | analogous | analogous |
| `oya-foundry-runtime-invocation-orchestrator-{kernel,...}` | analogous | analogous | analogous |
| `oya-foundry-runtime-runtime-pool-{kernel,...}` | analogous | analogous | analogous |
| `oya-foundry-runtime-capability-registry-cache-{kernel,...,adapter-postgres}` | analogous | analogous | analogous |

Port traits live exclusively in `*-kernel` crates; implementations exclusively in `*-adapter*`. Domain calls through ports; domain never imports adapter.

Cross-product integration check: this phase introduces NO direct crate imports between `foundry-runtime` and any other product µservice. Sibling µservice traffic (`foundry-providers`, `foundry-guardrails`, `foundry-evidence`, `foundry-supervisor`) is over mTLS-protected REST/gRPC contracts.

CI lanes that must green before phase exit gate: same as §"Fitness lane gates" above.

## ChangeSet Contract per IP

Every IP in this phase emits a ChangeSet per ADR-0110 (claimable + verifiable + bundleable + promotable). The minimum ChangeSet payload per IP, written at `microservices/intelligence-runtime/evidence/multispectrum/<change_id>-<unix_ts>.json` on `oya vcs done`:

```json
{
  "change_id": "ULID",
  "ip_id": "IP-NNN-<slug>",
  "microservice": "foundry-runtime",
  "milestone": "M01-foundation",
  "phase": "P01-agent-runtime-and-capability-execution",
  "claim_paths": ["microservices/intelligence-runtime/src/crates/<crate>/**", "..."],
  "intent": "<one-line>",
  "spec_refs": ["microservices/intelligence-runtime/PRD.md§<section>", "/specs/agent-operating-contract.json§<section>"],
  "acceptance_lanes_green": ["cargo-check", "cargo-build", "cargo-clippy", "cargo-nextest", "cargo-deny", "lean-a1", "lean-a2", "lean-a3", "lean-a4", "per-microservice-layout"],
  "test_count": {"unit": <int>, "integration": <int>, "e2e": <int>},
  "coverage_pct": <float>,
  "multispectrum_review_facets": ["F1..F9", "A1..A7", "M1..M2"],
  "signature": "Ed25519:<sig>",
  "executed_at": "ISO8601"
}
```

## Per-IP Test Coverage Threshold

| IP class | Minimum unit-test count | Minimum integration-test count | Minimum e2e-test count | Coverage threshold |
|---|---|---|---|---|
| kernel crate | 1 per public type + 1 per port trait | 0 | 0 | 90% line; 80% branch |
| domain crate | 1 per public function + property tests | 0 | 0 | 95% line; 90% branch |
| usecase crate | 1 per use case (happy + 2 sad paths) | ≥3 against mocked ports | 0 | 90% line; 80% branch |
| adapter crate | 1 per port-impl method | ≥2 against real backend (test container) | 0 | 85% line; 75% branch |
| adapter-redis crate | 1 per method | ≥2 against testcontainers Valkey 8.1 (Redis wire-compat) | 1 hot-read p99 load test | 85% line |
| adapter-postgres crate | 1 per method | ≥2 against testcontainers Postgres 16 | 1 cold-restore latency test | 85% line |
| rest crate | 1 per route (happy + auth-fail + tenant-mismatch) | ≥2 cross-route flows | 1 per route via REST integration test | 85% line |
| worker crate | 1 per orchestration arm | ≥1 long-lived loop integration test | 1 e2e (drain or hot-reload) | 85% line |
| sdk crate | 1 per public method (happy + retry + auth-fail) | ≥2 against rest crate | 0 | 90% line |
| app crate | composition-root smoke tests | 0 | 1 startup-and-shutdown smoke | 60% line |
| IaC IPs | n/a | ≥1 helm-install smoke | 1 against kind | n/a |

Enforced by `cargo nextest run --workspace` + `cargo llvm-cov --workspace --fail-under-lines <threshold>`.

## Oya VCS Symbol Locks

Per ADR-0116 (read-then-reverted by `feedback_oya_vcs_canonical_2026_05_16`), this phase uses `oya vcs` primitives exclusively via `cargo run -p oya-dev-cli -- vcs ...`. Grit / ICM are explicitly NOT used.

```bash
cargo run -p oya-dev-cli -- vcs claim \
  --agent <agent-id> \
  --intent "<IP-NNN-slug>: <one-line intent>" \
  --paths "microservices/intelligence-runtime/src/crates/<crate>/**"
cargo run -p oya-dev-cli -- vcs verify --agent <agent-id> --changeset <id>
cargo run -p oya-dev-cli -- vcs done --agent <agent-id> --changeset <id>
cargo run -p oya-dev-cli -- vcs promote --changeset <id>
```

Multispectrum evidence per docs/AGENTS.md §changeset: each IP emits `microservices/intelligence-runtime/evidence/multispectrum/<change_id>-<unix_ts>.json` per `/specs/multispectrum-review.json` v2.4.0.

## References

- ADR-0022; ADR-0024; ADR-0025; ADR-0056; ADR-0105; ADR-0106; ADR-0110; ADR-0123; ADR-0139; ADR-0131; ADR-0132; ADR-0133.
- `microservices/intelligence-runtime/PRD.md`.
- `microservices/observability/PHASE-01-AGENTIC-SLO-GATED-PROMOTION.md` (precedent shape for ADR-0131 phase artifact).
- Memory: `feedback_milestone_phase_hierarchy.md`, `feedback_naming_justification.md`, `feedback_oya_vcs_canonical_2026_05_16.md`, `feedback_clean_architecture_requirements.md`, `feedback_quality_performance_scalability_bar.md`.
