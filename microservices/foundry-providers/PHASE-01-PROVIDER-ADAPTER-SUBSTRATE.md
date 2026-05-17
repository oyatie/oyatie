---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P01-provider-adapter-substrate
status: Active
entry_gate: |
  ADR-0131 + ADR-0025 accepted; ADR-0026 phase-1 in-house-model spec ready; `cloud-secrets` (OpenBao) µservice deployed and reachable for ≥ pack-kr; cargo workspace ready to accept the new crates under microservices/foundry-providers/src/crates/.
exit_gate: |
  All 15 IPs merged; `oya-foundry-providers-credential-isolation` LEAN lane present in .github/branch-protection.yaml required_status_checks on dev and staging; `provider-router` decision p99 ≤ 5 ms verified via load test; OpenBao SecretReference resolution p99 ≤ 10 ms verified; `cargo nextest run --workspace` exits 0; `oya gate validate per-microservice-layout --microservice foundry-providers` exits 0; `oya gate validate authority-cohesion` exits 0; HG-FPRV gate in /specs/hyperscaler-gates.json registers green.
depends_on:
  - milestone: M01-foundation
    phase: observability/P01-agentic-slo-gated-promotion
    reason: SLO ledger must exist so provider-router demote/recover decisions consume canonical burn-rate signal
  - milestone: M01-foundation
    phase: cloud-secrets/P01-openbao-substrate
    reason: OpenBao SecretReference resolver is the canonical credential path; raw-credential leak prevention is non-negotiable
owner_team: axis-foundry + ops-security
related_adrs: [ADR-0025, ADR-0026, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0130, ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/per-microservice-flat-layout.json]
date: 2026-05-17
doc_status: published
---

# P01-provider-adapter-substrate: Land foundry-providers end-to-end

## Purpose

This phase ships the full ADR-0131 + ADR-0025 design for the provider-adapter substrate: a `provider-router` capability-aware router, six per-vendor adapter pairs (Claude / OpenAI / Gemini × API + subscription), an in-house-model adapter aligned to ADR-0026, an OpenBao credential-vault-bridge, and a provider-health-monitor sub-module. It is delivered as one phase in M01-foundation because every workload µservice that calls a foundation model depends on this substrate to advance past `dev` per the per-µservice gate posture.

This phase advances master-plan principles:
- Hyperscaler-grade in every practice (in-process router p99 ≤ 5 ms beats every hosted-router peer; per-provider Ed25519 audit-chain seal exceeds OpenRouter / LiteLLM).
- Nothing deferred within scope (every adapter ships with full credential isolation; no raw-credential code paths anywhere).
- No silent regression (per-tenant adapter version pin + adapter-substitution attack hardening prevent silent vendor behavior change).
- Per-microservice flat layout (this phase authors under `microservices/foundry-providers/` per ADR-0131).

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `foundry-providers` | `provider-router`, `anthropic-adapter`, `openai-adapter`, `gemini-adapter`, `in-house-model-adapter`, `credential-vault-bridge`, `provider-health-monitor` | All under `microservices/foundry-providers/` per ADR-0131 | `oya-foundry-providers-router-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}`, `oya-foundry-providers-adapter-anthropic-api`, `oya-foundry-providers-adapter-anthropic-subscription`, `oya-foundry-providers-adapter-openai-api`, `oya-foundry-providers-adapter-openai-subscription`, `oya-foundry-providers-adapter-gemini-api`, `oya-foundry-providers-adapter-gemini-subscription`, `oya-foundry-providers-adapter-in-house`, `oya-foundry-providers-adapter-openbao` |

Plus these repo-wide artifacts (cross-cutting per ADR-0131):
- `.github/branch-protection.yaml` — add `oya-foundry-providers-credential-isolation` BLOCKER lane to required_status_checks on `dev` and `staging`.
- `docs/standards/provider-adapter.md` (NEW) — cross-cutting adapter-conformance rules; credential isolation requirements; per-vendor disclosure record schema.
- `Cargo.toml` (workspace) — register new crates under `microservices/foundry-providers/src/crates/` and **relocate** existing `crates/oya-foundry-account-adapter-*` to the new path (their module surface is preserved as compat re-exports for one minor version per ADR §"no silent regression").
- `/specs/hyperscaler-gates.json` — register HG-FPRV gate.

### Out-of-scope

- Decommissioning the supervisor lifecycle in `crates/oya-foundry-account-*` — owned by a parallel `foundry-runtime` phase; this phase merely relocates the per-vendor adapter crates and preserves their surface as re-exports.
- New open-source local-model adapters (Llama-3.x / Mixtral / Qwen) — `adapter-in-house` interface is generic enough to absorb them later; concrete adapter ships under ADR-0026 phase 4.
- Aggregator-billing tenants (BYO OpenRouter/LiteLLM key) — out of scope for M01.
- Tenant-facing provider selection UI — Workflow Studio is the consumer; tenant-facing UX ships there.

## Implementation Plans

Ordered list. Each IP is an executable ChangeSet under this phase folder. Dependencies inline.

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-router-kernel.md`](IP-001-router-kernel.md) | `oya-foundry-providers-router-kernel`: port traits (`ProviderInvoker`, `ProviderRouter`, `CredentialResolver`, `HealthMonitor`), entities, sealed traits | pending | axis-foundry | — |
| [`IP-002-router-domain.md`](IP-002-router-domain.md) | `oya-foundry-providers-router-domain`: routing algebra (capability-fit × cost × residency × health weighting); pure | pending | axis-foundry | IP-001 |
| [`IP-003-router-usecase.md`](IP-003-router-usecase.md) | `oya-foundry-providers-router-usecase`: orchestration that composes router + invoker + credential-resolver + health-monitor | pending | axis-foundry | IP-002 |
| [`IP-004-router-api.md`](IP-004-router-api.md) | `oya-foundry-providers-router-api`: protocol-neutral typed contracts (`ProviderInvokeRequest`, `RouterDecision`, etc.) | pending | axis-foundry | IP-001 |
| [`IP-005-router-adapter.md`](IP-005-router-adapter.md) | `oya-foundry-providers-router-adapter`: Postgres + Redis repository impls (provider-config + token-bucket) | pending | axis-foundry | IP-001 |
| [`IP-006-adapter-anthropic-api.md`](IP-006-adapter-anthropic-api.md) | `oya-foundry-providers-adapter-anthropic-api`: Claude API HTTP transport; BLAKE3 + Ed25519 envelope | pending | axis-foundry | IP-001 |
| [`IP-007-adapter-anthropic-subscription.md`](IP-007-adapter-anthropic-subscription.md) | `oya-foundry-providers-adapter-anthropic-subscription`: Claude Pro/Max subscription transport | pending | axis-foundry | IP-001 |
| [`IP-008-adapter-openai-api.md`](IP-008-adapter-openai-api.md) | `oya-foundry-providers-adapter-openai-api`: OpenAI API HTTP transport | pending | axis-foundry | IP-001 |
| [`IP-009-adapter-openai-subscription.md`](IP-009-adapter-openai-subscription.md) | `oya-foundry-providers-adapter-openai-subscription`: ChatGPT Plus subscription transport | pending | axis-foundry | IP-001 |
| [`IP-010-adapter-gemini-api.md`](IP-010-adapter-gemini-api.md) | `oya-foundry-providers-adapter-gemini-api`: Gemini API HTTP transport | pending | axis-foundry | IP-001 |
| [`IP-011-adapter-gemini-subscription.md`](IP-011-adapter-gemini-subscription.md) | `oya-foundry-providers-adapter-gemini-subscription`: Gemini Advanced subscription transport | pending | axis-foundry | IP-001 |
| [`IP-012-adapter-in-house.md`](IP-012-adapter-in-house.md) | `oya-foundry-providers-adapter-in-house`: vLLM/TGI co-located endpoint transport per ADR-0026 | pending | axis-foundry | IP-001 |
| [`IP-013-adapter-openbao.md`](IP-013-adapter-openbao.md) | `oya-foundry-providers-adapter-openbao`: SecretReference resolver; per-tenant lease cache; rotation hook | pending | axis-foundry + ops-security | IP-001 |
| [`IP-014-router-rest-worker-app.md`](IP-014-router-rest-worker-app.md) | REST surface + worker (health monitor + cost roll-up) + composition-root app binary | pending | axis-foundry | IP-003, IP-005, IP-006..IP-013 |
| [`IP-015-router-sdk.md`](IP-015-router-sdk.md) | `oya-foundry-providers-router-sdk` Rust client + TS scaffold | pending | axis-foundry + gtm | IP-014 |

## Per-IP Test Coverage Threshold

| Layer | Line coverage | Branch coverage | Property tests | Notes |
|---|---|---|---|---|
| kernel | 90 % | 80 % | required | port-sealed; entity-invariant tests |
| domain | 95 % | 90 % | required | routing algebra is the math; canonical worked examples |
| usecase | 85 % | 70 % | optional | orchestration |
| adapter | 80 % | 70 % | as-needed | upstream-mocked; honest integration tests under tests/integration |
| rest / worker / app | 70 % | 60 % | optional | thin composition layers |
| sdk | 80 % | 70 % | optional | client surface |

## Verification

`cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry-providers` and `… authority-cohesion` and `… credential-isolation --microservice foundry-providers` exit 0.

Load test (`tests/load/router_decision.rs`) demonstrates router decision p99 ≤ 5 ms over 100 K decisions on a single core; OpenBao resolution p99 ≤ 10 ms with cache warm.

End-to-end drill: a synthetic workflow request routed across Anthropic API, OpenAI API, Gemini API, in-house adapter; each emits a `ProviderInvoked` audit event with Ed25519 seal; rotation drill demonstrates zero downtime for a 5-minute credential rotation window.

## References

- `microservices/foundry-providers/PRD.md`.
- ADR-0025, ADR-0026, ADR-0028, ADR-0056, ADR-0105, ADR-0106, ADR-0130, ADR-0131, ADR-0132, ADR-0133.
- `/specs/per-microservice-flat-layout.json`.
- `docs/standards/observability-slo.md`.
