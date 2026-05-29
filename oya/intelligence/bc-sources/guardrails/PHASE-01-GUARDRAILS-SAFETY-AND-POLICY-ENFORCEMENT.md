---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P01-guardrails-safety-and-policy-enforcement
status: Active
entry_gate: |
  ADR-0022 + ADR-0131 + ADR-0140 (retired per ADR-0145) accepted; foundry-runtime PRD published; observability PHASE-01 substrate live so guardrails' self-SLOs can be evaluated; cargo workspace ready to accept the new crates under microservices/intelligence-guardrails/src/crates/.
exit_gate: |
  All 15 IPs merged; classifier-model-serving Helm chart deployed; Cedar v4 policy bundle validates default-deny; rule-store Postgres schema migrated; pre-invocation classification p99 ≤ 50ms on reference workload; post-output validation p99 ≤ 100ms; Sev-1 jailbreak drill auto-creates post-mortem; foundry-runtime calls guardrails on every dispatch (verified by oya-governance-runtime-guardrails-coupling lane); HG-FGUARD gate green; cargo nextest run --workspace exits 0; oya gate validate per-microservice-layout --microservice foundry-guardrails exits 0; oya gate validate authority-cohesion exits 0.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion (observability)
    reason: guardrails publishes its own SLOs and is gated by them
  - milestone: M01-foundation
    phase: P01-foundry-runtime-runtime-and-sessions (foundry-runtime)
    reason: foundry-runtime is the sole in-cluster caller; coupling lane requires both
owner_team: axis-foundry-guardrails
related_adrs: [ADR-0022, ADR-0056, ADR-0105, ADR-0106, ADR-0110, ADR-0123, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0140]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/hyperscaler-gates.json, /specs/agent-operating-contract.json]
date: 2026-05-17
doc_status: published
---

# P01-guardrails-safety-and-policy-enforcement: Land the foundry-guardrails substrate end-to-end

## Purpose

This phase ships the foundry-guardrails substrate: prompt classifier, output validator, autonomy-ceiling gate (Cedar v4), content-safety rule engine (Postgres-backed), jailbreak detector (heuristic + classifier + LLM-judge ensemble), and AI-slop detector. It is delivered as one phase in M01-foundation because foundry-runtime cannot enter tenant-bearing traffic without this gate (the ADR-0131 Foundry split makes guardrails a precondition for runtime acceptance into production tier).

This phase advances master-plan principles:
- Hyperscaler-grade in every practice (competitor parity vs Bedrock / Constitutional AI / OpenAI Moderation / Azure Content Safety / Perspective / NeMo Guardrails / Llama Guard).
- Nothing scheduled-for-distinct-tracked-work (no FUTURE-marked stub; M01 ships all 6 BCs with at least one canonical implementation each).
- No silent regression (rule rollouts use shadow→enforce per ADR-0114 precedent).
- Per-microservice flat layout (this phase is the first native Foundry-side author under ADR-0131 after foundry-runtime).
- Cedar v4 + default-deny (no permit-by-default anywhere).

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected |
|---|---|---|
| `foundry-guardrails` | `prompt-classifier`, `output-validator`, `autonomy-ceiling-gate`, `content-safety-rule-engine`, `jailbreak-detector`, `ai-slop-detector` | All under `microservices/intelligence-guardrails/` per ADR-0131 |

Plus these repo-wide artifacts (cross-cutting per ADR-0131):
- `.github/branch-protection.yaml` — add `oya-governance-runtime-guardrails-coupling` to required_status_checks on `dev` and `staging`.
- `Cargo.toml` (workspace) — register the new crates under `microservices/intelligence-guardrails/src/crates/`.
- `/specs/hyperscaler-gates.json` — register HG-FGUARD gate per ADR-0123.
- `docs/quality/ai-slop-defense/ai-slop-failure-mode-catalogue.md` — referenced as the source-of-truth catalogue for `ai-slop-detector` test fixtures.

Naming justifications are in `microservices/intelligence-guardrails/PRD.md` §"Bounded Contexts".

### Out-of-scope

- `foundry-runtime` invocation orchestrator changes (already covered by foundry-runtime's own PHASE). This phase only adds the `coupling` lane that asserts foundry-runtime calls guardrails on every dispatch.
- `foundry-providers` LLM-judge low-cost endpoint (Open Question 1; resolved in successor-IP ADR).
- Tenant-facing Workflow Studio policy-author UX (scheduled-for-distinct-tracked-work to Workflow Studio's own roadmap; this phase ships the engine + Cedar fragment authoring via git PR).
- Cross-pack rule replication (forbidden by default per `policy/data-residency.md`).

## Implementation Plans

Ordered list. Each IP is an executable ChangeSet under this phase folder. Dependencies inline.

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-cedar-policy-engine-iac.md`](IP-001-cedar-policy-engine-iac.md) | Helm chart for in-cluster Cedar engine serving (sidecar pattern + standalone evaluator pool); Cedar v4 policy bundle authoring + validation pipeline | pending | axis-foundry-guardrails | — |
| [`IP-002-classifier-model-serving-iac.md`](IP-002-classifier-model-serving-iac.md) | Helm chart for ONNX-runtime classifier-serving (BERT-class PII/PHI + Llama-Guard-class content-safety + jailbreak classifier); Cosign-signed model artifacts; per-pack S3 model registry | pending | axis-foundry-guardrails + ops-sre-reliability | — |
| [`IP-003-rule-store-postgres-iac.md`](IP-003-rule-store-postgres-iac.md) | Helm chart for per-pack Postgres HA (rule store + Cedar fragment registry + audit-mutation log); migrations under `iac/postgres/migrations/` | pending | axis-foundry-guardrails + ops-sre-reliability | — |
| [`IP-004-prompt-classifier-kernel.md`](IP-004-prompt-classifier-kernel.md) | `oya-foundry-guardrails-prompt-classifier-kernel`: port traits + entities + value objects | pending | axis-foundry-guardrails | — |
| [`IP-005-output-validator-kernel.md`](IP-005-output-validator-kernel.md) | `oya-foundry-guardrails-output-validator-kernel`: port traits + entities | pending | axis-foundry-guardrails | — |
| [`IP-006-autonomy-ceiling-gate-kernel-and-cedar-adapter.md`](IP-006-autonomy-ceiling-gate-kernel-and-cedar-adapter.md) | `oya-foundry-guardrails-autonomy-ceiling-gate-kernel` + `-adapter-cedar` (Cedar v4 client + policy-bundle loader) | pending | axis-foundry-guardrails | IP-001 |
| [`IP-007-content-safety-rule-engine-kernel-and-postgres-adapter.md`](IP-007-content-safety-rule-engine-kernel-and-postgres-adapter.md) | `oya-foundry-guardrails-content-safety-rule-engine-kernel` + `-adapter-postgres` | pending | axis-foundry-guardrails | IP-003 |
| [`IP-008-jailbreak-detector-ensemble.md`](IP-008-jailbreak-detector-ensemble.md) | `oya-foundry-guardrails-jailbreak-detector-{kernel,domain,usecase,adapter,adapter-classifier-model}`: heuristic + classifier + LLM-judge ensemble | pending | axis-foundry-guardrails | IP-002, IP-004 |
| [`IP-009-ai-slop-detector.md`](IP-009-ai-slop-detector.md) | `oya-foundry-guardrails-ai-slop-detector-{kernel,domain,usecase,adapter}`: catalogue-driven pattern detection | pending | axis-foundry-guardrails | IP-005 |
| [`IP-010-classifier-model-adapter-onnx.md`](IP-010-classifier-model-adapter-onnx.md) | `-adapter-classifier-model` shared between prompt-classifier + jailbreak-detector kernels; ONNX-runtime client; per-model version pinning | pending | axis-foundry-guardrails | IP-002 |
| [`IP-011-rest-and-grpc-surface.md`](IP-011-rest-and-grpc-surface.md) | `-rest` crates for all 6 BCs; OpenAPI 3.2 + gRPC contracts; per-route Cedar policy bind | pending | axis-foundry-guardrails | IP-006, IP-007, IP-008, IP-009 |
| [`IP-012-worker-and-app-composition.md`](IP-012-worker-and-app-composition.md) | `-worker` + `-app` crates (composition roots); rule-cache hot-reload; shadow-mode runner | pending | axis-foundry-guardrails | IP-011 |
| [`IP-013-runtime-guardrails-coupling-lane.md`](IP-013-runtime-guardrails-coupling-lane.md) | New BLOCKER CI lane `oya-governance-runtime-guardrails-coupling`: asserts every foundry-runtime dispatch path goes through foundry-guardrails before foundry-providers | pending | axis-foundry + axis-foundry-guardrails | IP-011 |
| [`IP-014-shadow-mode-rollout-and-false-positive-budget.md`](IP-014-shadow-mode-rollout-and-false-positive-budget.md) | Shadow→enforce rule rollout per ADR-0114 precedent; per-tenant false-positive escalation budget; rule-author dashboard | pending | axis-foundry-guardrails | IP-007 |
| [`IP-015-sdk-rust-and-typescript.md`](IP-015-sdk-rust-and-typescript.md) | `oya-foundry-guardrails-prompt-classifier-sdk` (Rust first-party) + TS SDK via OpenAPI generator | pending | axis-foundry-guardrails + gtm | IP-011 |

Coverage check vs PRD: every BC has at least one IP touching kernel + adapter + rest + worker + app; HG-FGUARD gate land lane in IP-013; SDK in IP-015. Total: 15 IPs (matches deliverable spec).

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
oya gate validate lean-a1 --microservice foundry-guardrails
oya gate validate lean-a2 --microservice foundry-guardrails
oya gate validate port-location --microservice foundry-guardrails
oya gate validate layer-correctness --microservice foundry-guardrails
oya gate validate per-microservice-layout --microservice foundry-guardrails
oya gate validate statelessness --microservice foundry-guardrails
oya gate validate shardability --microservice foundry-guardrails
oya gate validate data-class --microservice foundry-guardrails
oya gate validate cedar-fragment-coverage --microservice foundry-guardrails
oya gate validate authority-cohesion
oya gate validate hyperscaler-maturity-claims
```

### Substrate gates introduced by this phase

```bash
oya gate validate runtime-guardrails-coupling --sha <head-sha>
oya gate validate classifier-model-cosign-signed
oya gate validate cedar-default-deny-enforced
oya gate validate rule-store-migrations-up-to-date
```

### End-to-end drill gates

| Scenario | Command | Pass criterion |
|---|---|---|
| Allow happy path | `cargo nextest run -p oya-foundry-guardrails-prompt-classifier-usecase --test classify_allow_happy` | verdict allow; signed; ledger record appended |
| Block: jailbreak | `cargo nextest run --test jailbreak_block` | verdict block; reason `jailbreak_injection`; Sev-1 incident issued |
| Block: PHI in non-HC pack | `cargo nextest run --test phi_block_non_hipaa` | verdict block; reason `phi_outside_baa` |
| Tier excess refusal | `cargo nextest run --test tier_excess_refusal` | refused at autonomy-ceiling-gate |
| Shadow→enforce promotion | scripted e2e: deploy rule shadow; verify decisions; promote enforce | enforce-decision matches shadow-decision within tolerance |
| FP escalation budget | scripted e2e: tenant marks N+1 block as FP | budget exceeded message; rule-author queue receives |
| Classifier-model rollback | scripted e2e: deploy bad model; verify rollback runbook | prior model restored within RTO |

### Workflow + Ontology integration gates

```bash
oya gate validate workflow-event-registry --microservice foundry-guardrails
oya gate validate ontology-type-registry --microservice foundry-guardrails
```

## Clean Architecture Compliance

Layer assignments and dependency direction:

| Crate (BNF v4.1) | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-foundry-guardrails-*-kernel` | `kernel` | (nothing project-internal) | all other layers |
| `oya-foundry-guardrails-*-domain` | `domain` | `kernel` | `usecase`, `adapter*`, `rest`, `worker`, `app` |
| `oya-foundry-guardrails-*-usecase` | `usecase` | `domain`, `kernel` | `adapter*`, `rest`, `worker`, `app` |
| `oya-foundry-guardrails-*-api` | `api` | `kernel` | adapter, rest |
| `oya-foundry-guardrails-*-adapter` | `adapter` | `usecase`, `domain`, `kernel` | `rest`, `worker`, `app` directly |
| `oya-foundry-guardrails-*-adapter-cedar` | `adapter` (backend-qualified) | same | same |
| `oya-foundry-guardrails-*-adapter-postgres` | `adapter` (backend-qualified) | same | same |
| `oya-foundry-guardrails-*-adapter-classifier-model` | `adapter` (backend-qualified) | same | same |
| `oya-foundry-guardrails-*-rest` | `rest` | `usecase`, `domain`, `kernel`, `api` | `adapter*` directly (uses ports) |
| `oya-foundry-guardrails-*-worker` | `worker` | `usecase`, `domain`, `kernel`, `api` | `adapter*` directly (uses ports) |
| `oya-foundry-guardrails-*-app` | `app` | composition root only | none — wiring |
| `oya-foundry-guardrails-*-sdk` | `sdk` | `api`, `kernel` | none |

Port traits live exclusively in `*-kernel` crates; implementations exclusively in `*-adapter*`. Domain/usecase calls through ports; never imports adapter.

Cross-product integration: this phase introduces NO direct imports between `foundry-guardrails` and any other product µservice. Cross-product flow via Workflow events (`GuardrailDecisionEmitted`, `JailbreakDetected`, etc.) + Ontology reads.

## ChangeSet Contract per IP

Every IP in this phase emits a ChangeSet per ADR-0110 (claimable + verifiable + bundleable + promotable). The minimum ChangeSet payload per IP, written at `microservices/intelligence-guardrails/evidence/multispectrum/<change_id>-<unix_ts>.json`:

```json
{
  "change_id": "ULID",
  "ip_id": "IP-NNN-<slug>",
  "microservice": "foundry-guardrails",
  "milestone": "M01-foundation",
  "phase": "P01-guardrails-safety-and-policy-enforcement",
  "claim_paths": ["microservices/intelligence-guardrails/src/crates/<crate>/**", "..."],
  "intent": "<one-line>",
  "spec_refs": ["microservices/intelligence-guardrails/PRD.md§<section>", "ADR-0140§Cedar"],
  "acceptance_lanes_green": ["cargo-check", "cargo-build", "cargo-clippy", "cargo-nextest", "cargo-deny", "lean-a1", "lean-a2", "per-microservice-layout", "cedar-fragment-coverage", "data-class"],
  "test_count": {"unit": 0, "integration": 0, "e2e": 0},
  "coverage_pct": 0.0,
  "multispectrum_review_facets": ["F1..F9", "A1..A7", "M1..M2", "F13_safety"],
  "signature": "Ed25519:<sig>",
  "executed_at": "ISO8601"
}
```

Validated by the `oya-governance-multispectrum-evidence` lane against `/specs/multispectrum-review.json` v2.4.0.

## Per-IP Test Coverage Threshold

| IP class | Minimum unit | Minimum integration | Minimum e2e | Coverage threshold |
|---|---|---|---|---|
| kernel | 1 per public type + 1 per port trait | 0 | 0 | 90% line; 80% branch |
| domain | 1 per public function + property tests | 0 | 0 | 95% line; 90% branch |
| usecase | 1 per use case (happy + 2 sad paths) | ≥3 against mocked ports | 0 | 90% line; 80% branch |
| adapter (incl. backend-qualified) | 1 per port-impl method | ≥2 against real backend (Cedar / Postgres / ONNX testcontainer) | 0 | 85% line; 75% branch |
| rest | 1 per route (happy + auth-fail + tenant-mismatch) | ≥2 cross-route | 1 per route | 85% line; 75% branch |
| worker | 1 per arm | ≥1 long-lived loop | 1 e2e | 85% line; 75% branch |
| sdk | 1 per client method (happy + retry + auth-fail) | ≥2 against rest | 0 | 90% line; 80% branch |
| app | composition-root smoke | 0 | 1 startup-and-shutdown smoke | 60% line |
| IaC | n/a | ≥1 helm-install + helm-test smoke per chart | 1 against kind | n/a |

Enforced by:
- `cargo nextest run --workspace --all-features` exits 0.
- `cargo llvm-cov --workspace --fail-under-lines <threshold>` exits 0.

## branch-protection.yaml diff preview

IP-013 updates `.github/branch-protection.yaml` with:

```yaml
branches:
  dev:
    required_status_checks:
      # ADDED by this phase (IP-013):
      - oya-governance-runtime-guardrails-coupling
      - oya-governance-cedar-default-deny-enforced
      - oya-governance-classifier-model-cosign-signed

  staging:
    required_status_checks:
      - oya-governance-runtime-guardrails-coupling
```

## Oya VCS Symbol Locks

Per ADR-0116 + the 2026-05-16 reversal (`oya vcs canonical`), this phase uses `oya vcs` primitives exclusively.

```bash
cargo run -p oya-dev-cli -- vcs claim \
  --agent <agent-id> \
  --intent "<IP-NNN-slug>: <one-line intent>" \
  --paths "microservices/intelligence-guardrails/src/crates/<crate>/**"

cargo run -p oya-dev-cli -- vcs verify --agent <agent-id> --changeset <id>
cargo run -p oya-dev-cli -- vcs done --agent <agent-id> --changeset <id>
cargo run -p oya-dev-cli -- vcs promote --changeset <id>
```

Multispectrum evidence per docs/AGENTS.md §changeset: each IP emits `microservices/intelligence-guardrails/evidence/multispectrum/<change_id>-<unix_ts>.json` per `/specs/multispectrum-review.json` v2.4.0; F13_safety facet added for this µservice's safety-bearing posture.

## References

- ADR-0022: Autonomy ceiling enforcement.
- ADR-0056: BNF v4.1.
- ADR-0105: 13-layer enum.
- ADR-0106: usecase rename.
- ADR-0110: ChangeSet state machine.
- ADR-0123: Hyperscaler maturity claim gate (HG-FGUARD).
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout (Foundry split).
- ADR-0132: Product-platform + bundle dissolution.
- ADR-0133: Industry-best-practice conformance program.
- ADR-0140: Cedar policy substrate.
- `/specs/per-microservice-flat-layout.json`.
- `/specs/hyperscaler-gates.json`.
- `/specs/agent-operating-contract.json`.
- `docs/quality/ai-slop-defense/ai-slop-failure-mode-catalogue.md`.
- `microservices/intelligence-guardrails/PRD.md`.
- Memory: `feedback_clean_architecture_requirements.md`, `feedback_quality_performance_scalability_bar.md`, `feedback_oya_vcs_canonical_2026_05_16.md`, `feedback_naming_justification.md`.
