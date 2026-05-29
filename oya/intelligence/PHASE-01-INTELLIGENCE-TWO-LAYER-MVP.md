---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
status: Active
entry_gate: |
  ADR-0255 + ADR-0255 amendment + ADR-0263 + ADR-0296 accepted; oyatie Cargo workspace ready
  to accept the 40+ `oya-intelligence-*` crates under microservices/intelligence/crates/;
  microservices/intelligence/manifest.json updated with substrate-tier + substrate-dependency
  declarations; the cell, cloud-secrets, policy-engine, observability, and audit-chain µservices
  already past dev (their SLO gates green).
exit_gate: |
  All 25 IPs merged; library-first SDK published (Rust + TS + Python + Swift + Kotlin);
  network REST + gRPC surfaces live; refusal-baseline + EU AI Act Annex III gates wired; audit
  tap emits at ≥ 99.99 % to audit-chain; first-token latency p99 < 2.0 s on Anthropic Sonnet 4
  default route; provider-credential BYOK end-to-end proven against Anthropic + OpenAI + Google + Bedrock + vLLM per ADR-0255 §D-4;
  brand-ux-surface SDK renders sparkle + tier badge + streaming text + citation; competitor-parity
  matrix updated; `oya-governance-promotion-readiness` lane green for `intelligence`.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion (observability)
    reason: dispatch + audit-tap SLOs only meaningful once SLO substrate is live
  - milestone: M01-foundation
    phase: cell + cloud-secrets baseline
    reason: tenant-scope envelope + OpenBao sidecar credential-handle path
owner_team: axis-intelligence
related_adrs: [ADR-0255, ADR-0255-amendment-library-first, ADR-0263, ADR-0296]
related_specs: [/specs/intelligence-two-layer-substrate.json]
date: 2026-05-20
doc_status: published
---

# P01-intelligence-two-layer-mvp: Land the AI Substrate end-to-end

## Purpose

This phase ships the full ADR-0255 design — Layer-A AI Substrate (model-routing + providers +
guardrails + eval + attribution + credential-resolver + audit-tap) plus Layer-B Consumer Brand UX
Surface (brand-ux-surface) — as one cohesive phase under M01-foundation. The phase delivers the
library-first dispatch SDK per the ADR-0255 amendment, the network-opt-in REST + gRPC surfaces,
multi-modal day-one transport (text + image + audio + video), caller-side RAG primitives, and
audience-tag-on-every-call discipline.

This phase advances master-plan principles per `CLAUDE.md`:
- Hyperscaler-grade (Stripe + Palantir + Linear + AWS bar) by matching OpenAI / Anthropic / Vertex
  API parity feature-for-feature plus library-first invariant.
- No silent regression (every public dispatch shape protected by `oya-lean-a10` lane day one).
- Per-microservice flat layout (this phase ships native under ADR-0131).
- Build ahead of certification (ADR-0250 KS#9) — EU AI Act Annex III refusal + KR PIPA Art. 23
  refusal floors live day one, not retrofitted.

## Scope

### In-scope

| µservice | Bounded contexts | Files / crates affected |
|---|---|---|
| `intelligence` | `model-routing`, `providers`, `guardrails`, `eval`, `attribution`, `brand-ux-surface`, `credential-resolver`, `audit-tap` | All under `microservices/intelligence/` per ADR-0131 |

BNF v4.1 crate names per layer enum:

```text
oya-intelligence-model-routing-{kernel,domain,usecase,adapter,api,rest,grpc,worker}
oya-intelligence-providers-adapter-{anthropic,openai,google,bedrock,azure-openai,
    cohere,mistral,vllm,sglang,tensorrt-llm,apple-foundation,openrouter,
    together,groq,huggingface-inference,replicate}
oya-intelligence-guardrails-{kernel,domain,usecase,adapter}
oya-intelligence-eval-{kernel,domain,usecase,adapter,worker}
oya-intelligence-attribution-{kernel,domain,usecase,adapter}
oya-intelligence-brand-ux-surface-{sdk-rs,sdk-ts,sdk-swift,sdk-kotlin,adapter}
oya-intelligence-credential-resolver-{kernel,usecase,adapter}
oya-intelligence-audit-tap-{usecase,adapter,worker}
oya-intelligence-app
```

Plus these repo-wide artifacts:
- `.github/branch-protection.yaml` — add `oya-governance-promotion-readiness` for `intelligence`.
- `Cargo.toml` (workspace) — register the new crates.
- `/specs/intelligence-two-layer-substrate.json` (NEW) — canonical spec.
- `microservices/intelligence/manifest.json` — substrate-tier + substrate-dependency declaration.

### Out-of-scope

- `intelligence-embeddings` and `intelligence-fine-tuning` — separate µservices per ADR-0255 §D
  promotion; tracked by their own M01 phases.
- Marketplace publication of dispatch adapter pack — deferred to M02-marketplace per ADR-0249.
- Model-hosted RAG (caller-side RAG is in scope; provider-side hosted RAG is delegated to the
  embeddings µservice).
- Fine-tuning lifecycle (registered as separate µservice).

## Implementation plans

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| IP-001 | Domain layer: DispatchRequest entity + value objects | pending | axis-intelligence | — |
| IP-002 | Domain layer: SecretReference value object (ADR-0255 §D-4) | pending | axis-intelligence | — |
| IP-003 | Domain layer: RefusalDecision entity + reason taxonomy | pending | axis-intelligence | IP-001 |
| IP-004 | Domain layer: RoutingDecision entity + provider catalog | pending | axis-intelligence | IP-001 |
| IP-005 | Domain layer: EvalRecord entity + canonicalen-set schema | pending | axis-intelligence | IP-001 |
| IP-006 | Domain layer: Attribution entity + citation schema | pending | axis-intelligence | IP-001 |
| IP-007 | Kernel: model-router port traits | pending | axis-intelligence | IP-001..IP-004 |
| IP-008 | Kernel: guardrail stack port traits | pending | axis-intelligence | IP-003 |
| IP-009 | Kernel: audit-tap port traits | pending | axis-intelligence | IP-001 |
| IP-010 | Usecase: dispatch flow orchestrator | pending | axis-intelligence | IP-007..IP-009 |
| IP-011 | Adapter: Anthropic API | pending | axis-intelligence | IP-010 |
| IP-012 | Adapter: OpenAI API | pending | axis-intelligence | IP-010 |
| IP-013 | Adapter: Google AI Studio + Vertex | pending | axis-intelligence | IP-010 |
| IP-014 | Adapter: AWS Bedrock | pending | axis-intelligence | IP-010 |
| IP-015 | Adapter: vLLM self-hosted | pending | axis-intelligence | IP-010 |
| IP-016 | REST API surface | pending | axis-intelligence | IP-010..IP-015 |
| IP-017 | gRPC API surface | pending | axis-intelligence | IP-010..IP-015 |
| IP-018 | Worker: audit emission to audit-chain seal stream | pending | axis-intelligence | IP-009 |
| IP-019 | App: composition root binary | pending | axis-intelligence | IP-016..IP-018 |
| IP-020 | Credential resolver sidecar integration (ADR-0296) | pending | ops-security | IP-002 |
| IP-021 | Multi-modal transport (text + image + audio + video) | pending | axis-intelligence | IP-010 |
| IP-022 | Streaming (SSE + WebSocket) transport | pending | axis-intelligence | IP-016 |
| IP-023 | EU AI Act high-risk refusal wiring | pending | council-privacy + axis-intelligence | IP-008 |
| IP-024 | provider-credential BYOK tenant onboarding flow (ADR-0255 §D-4) | pending | gtm-customer-success + ops-security | IP-020 |
| IP-025 | Library-first caller-side PQ eval | pending | axis-intelligence | IP-005 |

Coverage check vs ADR-0255 §"Required surfaces": all 8 BCs covered; library-first SDK shipped
across 5 languages (IP-011/-012/-013/-014/-015 expose adapters; IP-016/-017 expose network
surfaces; brand-ux-surface ships under separate PHASE-02).

## Acceptance gates

```bash
cargo check  --workspace --all-features
cargo build  --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo deny check
cargo doc   --workspace --no-deps

# Repo-wide gates
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice intelligence
cargo run -p oya-dev-cli -- gate validate cedar-fragment-coverage  --microservice intelligence
cargo run -p oya-dev-cli -- gate validate openapi-conformance      --microservice intelligence
cargo run -p oya-dev-cli -- gate validate asyncapi-conformance     --microservice intelligence
cargo run -p oya-dev-cli -- gate validate audit-tap-emit           --microservice intelligence
cargo run -p oya-dev-cli -- gate validate library-first-dispatch   --microservice intelligence
```

### Fitness lane gates

| Lane | Threshold | Block status |
|---|---|---|
| `oya-governance-cedar-fragment-coverage` | 100 % BC × layer coverage | BLOCKER |
| `oya-governance-eu-ai-act-annex-iii-refusal` | All Annex III categories refused | BLOCKER |
| `oya-governance-byok-platform-default-disambiguation` | BYOK label correct per tenant pack | BLOCKER |
| `oya-governance-audit-tap-emit-ratio` | ≥ 99.99 % | BLOCKER |
| `oya-governance-library-first-dispatch-invariant` | In-process SDK is default | BLOCKER |
| `oya-governance-version-pinning-conformance` | All provider SDK + Rust deps pinned | BLOCKER |
| `oya-governance-doc-coverage` | Doc-coverage ≥ ADR-0063 floor | BLOCKER |
| `oya-governance-naming-justification` | Every new BC carries naming-justification block | BLOCKER |

### SLO gate

The dispatch availability + latency + first-token-latency + streaming-throughput +
audit-emission-success OpenSLO manifests must report `eligible` in the eligibility ledger before
this phase's `exit_gate` is declared.

### Multi-spectrum review

Per multispectrum-review v2.4.0 doctrine (`feedback_multispectrum_review_v22.md` + extension):
F1-F9 + M1-M2 + F10-F11-F13 + A1-A7 facets, each with its own subagent lens, evidence at
`evidence/multispectrum/<change_id>-<unix_ts>.json`. Phase cannot exit without all facets returning
APPROVE.

## Risk register

| ID | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R-01 | Provider rate-limit during launch | M | M | Per-provider QPS budget + per-tenant quota; fallback to secondary provider; see `failure-modes.md` FM-01 |
| R-02 | Provider credential leak via misconfigured provider-credential BYOK (ADR-0255 §D-4) | L | H | ADR-0296 sidecar credential-handle path; provider credential never enters intelligence process memory |
| R-03 | Refusal false-positive cascade (over-refusal harms UX) | M | M | A/B + canonicalen-set eval; per-pack refusal floor isolated from per-tenant policy; runbook `refusal-false-positive-cascade.md` |
| R-04 | Refusal false-negative (CSAM / violence / Annex III leak through) | L | H | Multi-stage refusal: pre-call classifier + post-call classifier + provider's own filter; EU AI Act Annex III gate as hard refusal |
| R-05 | Audit-row forgery | L | H | Ed25519 signature by `audit-tap-worker` SPIFFE identity; audit-chain seal verifies signature chain |
| R-06 | Library-first invariant violated (callers go via network instead of SDK) | M | L | `oya-governance-library-first-dispatch-invariant` lane greps for `http://intelligence` in non-shell code; warns + blocks merge |
| R-07 | Multi-modal scope creep | M | M | Video transport behind `oya-feature-flag::intelligence::video` flag; default off; promote per tenant after eval |
| R-08 | Cross-tenant context leak | L | H | Tenant-scope envelope on every call; Cedar refuses cross-tenant; per-tenant credential isolation |

## References

- ADR-0255 — Intelligence as two-layer AI Substrate.
- ADR-0255 amendment — Library-first network-opt-in clarification.
- ADR-0263 — Audit-tap.
- ADR-0296 — Sidecar credential-handle.
- ADR-0250 — Build ahead of certification.
- `microservices/intelligence/PRD.md`.
- `microservices/intelligence/ARCHITECTURE.md`.
- `microservices/intelligence/threat-model.md`.
- `microservices/intelligence/dpia.md`.
- `docs/standards/documentation-rigor.md`.
