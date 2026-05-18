---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
status: Active
entry_gate: |
  ADR-0131 per-microservice flat layout accepted; audit-chain µservice P01 in flight (evidence emission delegates to it); foundry-runtime + foundry-eval + foundry-guardrails + foundry-supervisor µservice scaffolds present (Workflow event sources); observability µservice's SLO substrate available for self-SLO authoring; ADR-0024 (eval-evidence integration) inheritance confirmed.
exit_gate: |
  All 15 IPs merged; oya-foundry-evidence-* crate families landed (kernel/domain/usecase/api/adapter/adapter-postgres/adapter-s3/adapter-audit-chain-bridge/rest/worker/sdk/app) and Cargo workspace builds clean; pack-assembly end-to-end drill exits 0 with mocked foundry-runtime+eval+guardrails sources; audit-chain bridge integration smoke-passes; regulator-export framework profiles (eu-ai-act / hipaa / gdpr / kr-pipa / soc2 / iso-27001) drill-pass; HG-FOUNDRY-EVIDENCE registered in /specs/hyperscaler-gates.json; oya gate validate per-microservice-layout --microservice foundry-evidence exits 0; cargo nextest run --workspace exits 0; cross-pack-replication-forbidden lane green; hyperscaler-maturity-claims lane green; ADR-0133 evidence-claim matrix CI-asserted.
depends_on:
  - milestone: M01-foundation
    phase: P01-audit-chain-substrate
    reason: evidence packs delegate Merkle sealing to audit-chain; without audit-chain the substrate is not available
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion (observability)
    reason: self-SLO authoring requires the SLO engine
owner_team: axis-foundry-evidence
related_adrs: [ADR-0003, ADR-0024, ADR-0028, ADR-0056, ADR-0105, ADR-0106, ADR-0139, ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/foundry-evidence.json, /specs/per-microservice-flat-layout.json, /specs/foundry-agent-runtime.json]
date: 2026-05-17
doc_status: published
---

# P01-foundry-evidence-frontend: Land the Foundry-agent evidence frontend end-to-end

## Purpose

This phase ships the full ADR-0131 Foundry-split design — a Foundry-specific evidence frontend that aggregates per-capability-invocation signals from foundry-runtime, foundry-eval, foundry-guardrails, and foundry-supervisor into structured evidence packs and emits them to the global `audit-chain` substrate for cryptographic sealing.

This phase advances master-plan principles:
- Hyperscaler-grade traceability (EU AI Act Art. 12+26+18 + HIPAA §164.312(b) + GDPR Art. 30 + KR PIPA Art. 29) by construction.
- Bominal-inheritance precedence (ADR-0003 emission contract, ADR-0024 eval-evidence, ADR-0028 audit-chain inherited 1:1 via the `audit-chain` substrate; oyatie overlays only the Foundry-specific aggregation + framework-filtered export).
- Per-microservice flat layout (ADR-0131).
- Honest-claim posture (ADR-0133): no claimed performance/depth without CI-asserted evidence; competitor parity matrix declares gaps.
- Nothing scheduled-for-distinct-tracked-work (no "later we'll add EU AI Act profile" stubs).

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected |
|---|---|---|
| `foundry-evidence` | `capability-invocation-recorder`, `evidence-pack-builder`, `eval-evidence-aggregator`, `evidence-query`, `regulator-export` | All under `microservices/foundry-evidence/` per ADR-0131; ~12 new Rust crates across BCs |

Plus repo-wide artifacts:
- `Cargo.toml` (workspace) — register the 12 new crates.
- `/specs/foundry-evidence.json` — formalised spec (new).
- `/specs/hyperscaler-gates.json` — register HG-FOUNDRY-EVIDENCE gate per ADR-0123.
- `.github/branch-protection.yaml` — add `oya-foundry-evidence-self-verification` lane to required_status_checks on `dev`.

### Out-of-scope

- Merkle sealing + Ed25519 + HSM + WORM blob storage — owned by `audit-chain` µservice (ADR-0131 substrate split). This phase consumes that substrate; it does not reimplement.
- Per-vertical evidence-pack schema extensions (e.g., medical-device EU MDR overlays) — owned by per-pack overlay µservices; this phase ships the canonical-base schema (per `feedback_canonical_base_localization.md`).
- Tenant-portal UI for evidence browsing — owned by `application` µservice's portal frontends; this phase ships the API surface only.
- Migration of pre-existing crates `oya-foundry-evidence-*` (if any) — physical move owned by IP-M01-MIGR-FND-4 per ADR-0131 §"Migration sequencing"; this phase scaffolds natively.

## Implementation Plans

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-storage-backend-iac.md`](IP-001-storage-backend-iac.md) | Helm/Kustomize for Postgres (HA evidence index) + evidence-blob-store S3 bucket (proxied through audit-chain WORM) + per-pack overlay | pending | cloud-secrets + axis-foundry-evidence | audit-chain P01 IP-001 |
| [`IP-002-self-slo-manifest.md`](IP-002-self-slo-manifest.md) | OpenSLO manifests at `microservices/foundry-evidence/slos/` for pack_assembly_latency / evidence_query_latency / audit_chain_backlog_depth / regulator_export_latency | pending | axis-foundry-evidence | observability P01 |
| [`IP-003-capability-invocation-recorder-kernel.md`](IP-003-capability-invocation-recorder-kernel.md) | `oya-foundry-evidence-capability-invocation-recorder-kernel` crate: port traits + entity types + errors | pending | axis-foundry-evidence | — |
| [`IP-004-evidence-pack-builder-kernel.md`](IP-004-evidence-pack-builder-kernel.md) | `oya-foundry-evidence-evidence-pack-builder-kernel`: port traits for SignalSource (runtime/eval/guardrails/supervisor) + AuditChainBridge | pending | axis-foundry-evidence | IP-003 |
| [`IP-005-evidence-pack-builder-domain.md`](IP-005-evidence-pack-builder-domain.md) | `oya-foundry-evidence-evidence-pack-builder-domain`: pack-schema construction + invariants + framework-profile builders | pending | axis-foundry-evidence | IP-004 |
| [`IP-006-evidence-pack-builder-usecase-and-adapters.md`](IP-006-evidence-pack-builder-usecase-and-adapters.md) | `oya-foundry-evidence-evidence-pack-builder-{usecase,api,adapter,adapter-postgres,adapter-s3,adapter-audit-chain-bridge,worker,app}` | pending | axis-foundry-evidence | IP-005, IP-001 |
| [`IP-007-capability-invocation-recorder-stack.md`](IP-007-capability-invocation-recorder-stack.md) | `oya-foundry-evidence-capability-invocation-recorder-{domain,usecase,api,adapter,rest,sdk}`: sync REST receipt + WAL + dead-letter | pending | axis-foundry-evidence | IP-003, IP-006 |
| [`IP-008-eval-evidence-aggregator.md`](IP-008-eval-evidence-aggregator.md) | `oya-foundry-evidence-eval-evidence-aggregator-{kernel,domain,usecase,adapter,worker}`: foundry-eval verdict join per ADR-0024 | pending | axis-foundry-evidence + axis-foundry-eval | IP-006 |
| [`IP-009-evidence-query-stack.md`](IP-009-evidence-query-stack.md) | `oya-foundry-evidence-evidence-query-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,sdk}`: tenant-scoped + Cedar-gated query; audit-of-audits emission | pending | axis-foundry-evidence | IP-006 |
| [`IP-010-regulator-export-stack.md`](IP-010-regulator-export-stack.md) | `oya-foundry-evidence-regulator-export-{kernel,domain,usecase,api,adapter,rest,worker}`: framework-filtered signed bundle assembly | pending | axis-foundry-evidence + council-privacy | IP-009 |
| [`IP-011-audit-chain-bridge.md`](IP-011-audit-chain-bridge.md) | `oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge`: SDK consumer of audit-chain emission per Bominal ADR-0003 | pending | axis-foundry-evidence + axis-audit-chain | audit-chain P01 IP-014 |
| [`IP-012-sdk-cross-microservice.md`](IP-012-sdk-cross-microservice.md) | `oya-foundry-evidence-sdk` consumed by foundry-runtime + foundry-guardrails + foundry-supervisor + foundry-eval | pending | axis-foundry-evidence + axis-foundry-runtime + axis-foundry-guardrails | IP-007 |
| [`IP-013-regulator-export-framework-profiles.md`](IP-013-regulator-export-framework-profiles.md) | Six framework profiles (eu-ai-act / hipaa / gdpr / kr-pipa / soc2 / iso-27001) with citation-anchored field selectors | pending | council-privacy + axis-foundry-evidence | IP-010 |
| [`IP-014-evidence-archive-cascade.md`](IP-014-evidence-archive-cascade.md) | Hot→warm→cold archival cascade interlocked with audit-chain retention cascade | pending | axis-foundry-evidence + axis-audit-chain | IP-010 |
| [`IP-015-self-observability-slo-wiring.md`](IP-015-self-observability-slo-wiring.md) | Wire foundry-evidence SLI emission into observability substrate; HG-FOUNDRY-EVIDENCE gate registration; hyperscaler-maturity-claims lane green | pending | axis-foundry-evidence + axis-observability | IP-002, IP-006..IP-010 |

Coverage check vs scope: every BC (capability-invocation-recorder, evidence-pack-builder, eval-evidence-aggregator, evidence-query, regulator-export) has at least one IP. Every concrete file target in `PRD.md`'s NFR table is owned by exactly one IP.

## Acceptance Gates

```bash
cargo check --workspace --all-features
cargo build --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo deny check
cargo doc --workspace --no-deps

oya gate validate lean-a1 --microservice foundry-evidence
oya gate validate lean-a2 --microservice foundry-evidence
oya gate validate port-location --microservice foundry-evidence
oya gate validate layer-correctness --microservice foundry-evidence
oya gate validate per-microservice-layout --microservice foundry-evidence
oya gate validate statelessness --microservice foundry-evidence
oya gate validate shardability --microservice foundry-evidence
oya gate validate authority-cohesion
oya gate validate hyperscaler-maturity-claims
oya gate validate cross-pack-replication-forbidden --microservice foundry-evidence
oya gate validate audit-chain-bridge-only --microservice foundry-evidence
```

### End-to-end drill gates

| Scenario | Command | Pass criterion |
|---|---|---|
| record_invocation happy path | `cargo nextest run -p oya-foundry-evidence-capability-invocation-recorder-usecase --test record_happy_path` | receipt returned ≤ 500 ms; WAL durable |
| Full pack assembly end-to-end | scripted e2e with mocked runtime/eval/guardrails sources | pack visible in evidence_query ≤ 2 s; audit-chain event_id present |
| eval-evidence join correctness | `cargo nextest run -p oya-foundry-evidence-eval-evidence-aggregator --test eval_join_correctness` | eval verdict matches foundry-eval state at invocation timestamp |
| Regulator-export EU AI Act profile | `cargo nextest run -p oya-foundry-evidence-regulator-export-usecase --test eu_ai_act_profile` | bundle contains exactly Art. 12 + Art. 18 + Art. 26 fields |
| Regulator-export HIPAA profile | `cargo nextest run -p oya-foundry-evidence-regulator-export-usecase --test hipaa_profile` | bundle contains §164.312(b) + §164.308(a)(1)(ii)(D) fields |
| Cross-pack-replication refusal | `cargo nextest run -p oya-foundry-evidence-evidence-query-rest --test cross_pack_refusal` | foreign-pack query rejected with structured error |
| audit-of-audits emission | property-based across 1k reads | every read produced an `audit-chain` event with `event_class=foundry.evidence.read.v1` |

## Clean Architecture Compliance

Layer assignments per ADR-0105 13-layer enum (canonical):

| Crate (BNF v4.1) | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-foundry-evidence-*-kernel` | `kernel` | (nothing project-internal) | all other layers |
| `oya-foundry-evidence-*-domain` | `domain` | own-BC `kernel` | adapter, rest, worker, sdk, app |
| `oya-foundry-evidence-*-usecase` | `usecase` | own-BC `domain`, `kernel` | adapter, rest, worker, sdk, app |
| `oya-foundry-evidence-*-api` | `api` | own-BC `usecase`, `domain`, `kernel` | adapter, rest, worker, sdk, app |
| `oya-foundry-evidence-*-adapter*` | `adapter` | own-BC `api`, `usecase`, `domain`, `kernel`; `oya-audit-chain-emission-sdk` (only for the audit-chain-bridge adapter) | rest, worker, sdk, app |
| `oya-foundry-evidence-*-rest` | `rest` | own-BC `api`, `adapter` | worker, sdk, app |
| `oya-foundry-evidence-*-worker` | `worker` | own-BC `api`, `adapter` | rest, sdk, app |
| `oya-foundry-evidence-*-sdk` | `sdk` | own-BC `api` (read-only types) | adapter, rest, worker, app |
| `oya-foundry-evidence-*-app` | `app` | own-BC `rest`, `worker`, `adapter` | — |

Cross-BC imports forbidden except through own-BC `api` re-exports. Cross-µservice imports forbidden except via SDK + Workflow + Ontology adapter layer per `feedback_workflow_objectgraph_adapter_layer.md`.

## Hyperscaler-grade gates registered

- `HG-FOUNDRY-EVIDENCE` registered in `/specs/hyperscaler-gates.json` per ADR-0123:
  - p99 `record_invocation` ≤ 500 ms (CI lane: load-drill).
  - p99 `evidence_query` ≤ 100 ms (CI lane: load-drill).
  - Pack assembly success rate ≥ 99.99 % (CI lane: chaos-drill).
  - Audit-chain bridge availability ≥ 99.99 % (CI lane: bridge-availability-drill).
  - Six framework profiles produce regulator-loadable bundles (CI lane: regulator-profile-drill).

## ADR-0133 honesty contract

Per ADR-0133, claims in `competitor-parity-matrix.md` are CI-asserted via `hyperscaler-maturity-claims` lane. Where parity is not yet achieved, the matrix declares a **honest gap** with a roadmap pointer; the gate refuses commit-claims for non-implemented depth.

## Next phase

P02-foundry-evidence-vertical-overlays (subsequent-to-M01-completion): pack-specific evidence-pack schema extensions (EU MDR for medical-device tenants on pack-eu; FDA 21 CFR Part 11 for pack-us-healthcare; KR-FSS sector overlay for pack-kr financial-services tenants).
