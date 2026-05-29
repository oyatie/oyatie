---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-foundry-eval
microservice: foundry-eval
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M01-foundation
bominal_source: []
related_adrs: [ADR-0024, ADR-0026, ADR-0056, ADR-0105, ADR-0106, ADR-0139, ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/per-microservice-flat-layout.json]
date: 2026-05-17
owner_team: axis-foundry
doc_status: published
---

# PRD-foundry-eval: Foundry Eval Harness Substrate

## Purpose

The `foundry-eval` microservice is oyatie's eval-harness substrate. It runs per-capability eval sets, computes pass/fail verdicts, produces parity reports against provider baselines, replays past production traces with deterministic divergence assertions, and gates capability publish + in-house-model cutover decisions (ADR-0024 + ADR-0026). It is the canonical eval substrate referenced by every capability owner and every model-routing decision in `foundry-runtime` / `foundry-providers`.

Per ADR-0131 (per-microservice flat layout) + ADR-0132 (product-platform dissolution), `foundry-eval` ships as a flat µservice — the former `foundry` product bundle split into `foundry-providers`, `foundry-runtime`, `foundry-supervisor`, `foundry-evidence`, `foundry-guardrails`, and `foundry-eval`. This PRD scaffolds the eval split.

This µservice is **shared substrate**, not a hero product. Tenants do not author eval-sets directly; capability owners do, and tenants consume the resulting publish-gate verdicts and the public parity report. It is consumed by `foundry-runtime` (capability publish gate), `foundry-providers` (A/B routing decision), every capability owner team (nightly eval), and `foundry-evidence` (audit emission of eval evidence).

The µservice has no Bominal equivalent and originates in oyatie.

## Tenant Value

- **Tenant Outcome 1 — Capability quality you can audit.** Every capability a tenant invokes carries a Cosign-signed eval-set run with the latest verdict; the verdict is published in the capability catalog UI; tenants can refuse to upgrade until eval evidence is current.
- **Tenant Outcome 2 — Per-vertical / per-locale assurance.** Regulated-vertical tenants (pack-us-healthcare, pack-kr-finance) see per-locale + per-vertical eval cohorts pass before the capability ships to their environment.
- **Tenant Outcome 3 — Replay-grounded regression detection.** Past production invocations of a tenant's capability call become regression-detection assets; a model upgrade that would have regressed the tenant's traffic is detected before it lands.
- **Internal Outcome 4 — Provider switch with evidence.** When an in-house model variant beats a provider per-vertical (ADR-0026), the cutover is gated on a per-capability A/B parity-win in foundry-eval; no leap-of-faith model swaps.
- **Internal Outcome 5 — Cross-capability bar.** Every capability across every product passes the same adversarial + linguistic + parity gate; eliminates per-team divergence in what "production-ready capability" means.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | capability owner | to author a signed eval set at `microservices/intelligence-eval/eval-sets/<capability>/v<n>.evalset.yaml` | my capability can publish | eval-set-registry | Must |
| FR-02 | eval runner | to execute an eval set against a candidate route (provider+model) and emit per-case results | publish-gate / nightly / A/B / replay paths share one execution substrate | eval-runner | Must |
| FR-03 | parity analyzer | to compare two eval-runs (incumbent vs candidate; provider-A vs provider-B; provider vs in-house) per-cohort | A/B routing changes and in-house-cutover decisions are evidence-backed | parity-analyzer | Must |
| FR-04 | replay engine | to replay a sampled production-trace cohort against a candidate route with ≤ 100ms divergence tolerance on deterministic seeds | model upgrades catch regression before stable | replay-engine | Must |
| FR-05 | baseline-output store | to persist Cosign-signed baseline outputs immutable + per-subject-keyed encrypted (DSR-shred per ADR-0024 §"Resolved 1") | regression detection is reproducible; subject erasure shreds without record-deletion | baseline-output-store | Must |
| FR-06 | publish gate | to refuse `oya admin capability publish` when eval-set missing / adversarial cohort failed / linguistic minima unmet / latest run stale | a capability that lacks empirical assurance never reaches tenants | eval-runner + eval-set-registry | Must |
| FR-07 | nightly orchestrator | to run every published capability's eval set against its current route on a 24h cadence and alarm on per-capability pass-rate drop ≥ 2 consecutive runs | drift is detected, not surprised | eval-runner | Must |
| FR-08 | A/B router gate | to refuse `oya admin route preference` changes when the eval-set A/B verdict isn't a per-cohort win | routing changes carry their own provenance | parity-analyzer | Must |
| FR-09 | in-house cutover gate | to emit a `InHouseCutoverEligible(capability, model)` verdict when the in-house variant beats the provider per-cohort on the live eval-set | ADR-0026 cutover is automated and reversible | parity-analyzer | Must |
| FR-10 | tenant operator | to view per-capability latest verdict + per-cohort breakdown via `slo:eval:read:<tenant>` scope | tenants can refuse upgrades on stale or failing eval evidence | eval-runner-sdk | Must |
| FR-11 | DSR cascade | to receive an `EraseSubject(subject_id)` event and shred all per-subject DEKs in the replay store within the 30d SLA | GDPR Art. 17 / KR PIPA Art. 36 erasure satisfied without breaking cross-cohort continuity | replay-engine + baseline-output-store | Must |
| FR-12 | foundry-evidence | to receive per-eval-run audit events (signed; Merkle-sealed) | every publish / nightly / A/B / replay run is auditable | eval-runner | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Eval-case execution latency (one case, GPU-eligible runner) | ≤ 500ms | ≤ 2s | ≤ 5s | excluding model-call latency (which is the model's own SLO) |
| Eval-set total wall time (1000 cases; parallel 32 GPU pods) | ≤ 5 min | ≤ 15 min | ≤ 30 min | bounded by max parallelism + provider rate-limits |
| Replay-determinism divergence (deterministic-seed cases) | ≤ 50 ms | ≤ 100 ms | ≤ 150 ms | end-to-end; per ADR-0024 + bar of "≤ 100ms divergence tolerance" |
| Parity verdict emission (incumbent vs candidate, per-cohort) | ≤ 200ms | ≤ 1s | ≤ 3s | post run completion; ClickHouse cohort-rollup query |
| Publish-gate verdict latency (read latest run + adversarial check) | ≤ 250ms | ≤ 1s | ≤ 2s | hot path for `oya admin capability publish` |
| Nightly orchestrator queue lag | ≤ 60s | ≤ 5min | — | from cron tick to first case dispatched |
| Replay-trace fetch latency (S3) | ≤ 100ms | ≤ 500ms | ≤ 1s | per replay-sample case |
| Baseline-output read latency (S3) | ≤ 50ms | ≤ 200ms | ≤ 500ms | per case (Cosign verify on cold-path) |

### Security

- All eval-set authoring requires Cosign-signed manifests (per ADR-0024 §"Eval kernel"); checked-in YAML + detached sig.
- Baseline outputs encrypted with per-subject-keyed envelopes (DEK-per-subject wrapped by per-tenant KEK per Bominal ADR-0043); KMS-resident KEKs.
- Replay traces inherit the subject-keyed encryption; DSR cascade shreds DEKs, never records.
- GPU runner pool uses gVisor or Kata sandboxes (no shared CUDA context across cases); per-case ephemeral filesystem.
- All eval-run emissions (PromQL / ClickHouse INSERT / Mimir) signed Ed25519 by eval-runner SPIFFE identity.
- Secrets (provider API keys for eval-time invocation; ClickHouse password; OpenBao tokens) follow the local-OpenBao SecretReference pattern; raw secrets never enter the repo, chat, or checkpoints.

### Audit + Compliance

- Every `EvalRunStarted`, `EvalRunCompleted`, `ParityVerdictEmitted`, `ReplayDivergenceDetected`, `InHouseCutoverEligible`, and `EvalSubjectShred` event emits an audit-chain record (Merkle / Ed25519 per Bominal ADR-0028) to `foundry-evidence`.
- ClickHouse `parity_analytics` table is append-only + partition-by-week; per-week partitions immutable post-seal.
- EU AI Act §15 (accuracy) + §17 (logging) compliance: per-capability latest eval-run = the §15 accuracy evidence; nightly run cadence + replay = the §17 logging surface.
- HIPAA when medical capabilities evaluated (pack-us-healthcare): PHI never enters eval-sets directly; synthetic-PHI fixtures only; the gate ensures medical-capability eval-cases declare `data_class=PHI_SYNTHETIC`.

### Availability + SLO

- Availability target: 99.9% monthly for the publish-gate verdict path (capability publish must be available even when nightly orchestrator is paused).
- ClickHouse parity-analytics availability target: 99.5% monthly (analytics, not hot-path).
- Nightly orchestrator: 99% successful run-completion rate (a failed nightly retries within 6h).
- RTO: ≤ 30 min. RPO: ≤ 1 eval-run (re-runnable from eval-set + checkpointed run state).

### Data residency

- Eval sets, baseline outputs, replay traces, parity analytics inherit the per-pack residency per ADR-0117. Pack-us-healthcare eval data stays in HIPAA-eligible US region; pack-kr in KR; pack-eu in EU.
- Cross-pack eval-result aggregation goes through differential-privacy aggregation (per `policy/dp-analysis.md`) before exposure on the public parity dashboard.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename), layers used by this µservice: `kernel`, `domain`, `usecase`, `api`, `adapter`, `rest`, `worker`, `sdk`, `app`. The eval-runner BC includes backend-qualified adapters per ADR-0105 §"Amendment 3" (`*-adapter-<backend>`): `-adapter-s3`, `-adapter-clickhouse`, `-adapter-gpu`.

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `eval-set-registry` | `oya-foundry-eval-eval-set-registry-{kernel,domain,usecase,api,adapter,app}` | Eval-set manifest read; Cosign signature verification; per-capability index; version pinning | `EvalSet`, `EvalCase`, `EvalSetVersion`, `CosignAttestation` |
| `eval-runner` | `oya-foundry-eval-eval-runner-{kernel,domain,usecase,api,adapter,adapter-s3,adapter-gpu,rest,worker,sdk,app}` | Eval-set execution; per-case dispatch; aggregate computation; publish-gate; nightly + A/B + replay orchestration | `EvalRun`, `EvalCaseResult`, `EvalAggregate`, `ProviderRoute` |
| `parity-analyzer` | `oya-foundry-eval-parity-analyzer-{kernel,domain,usecase,api,adapter,adapter-clickhouse,rest,worker,app}` | Two-run delta; per-cohort comparison; in-house-cutover verdict; competitor-parity matrix emission | `ParityReport`, `CohortDelta`, `InHouseCutoverVerdict` |
| `replay-engine` | `oya-foundry-eval-replay-engine-{kernel,domain,usecase,api,adapter,adapter-s3,worker,app}` | Replay-trace sampling; deterministic-seed execution; divergence detection; per-subject-DEK shred | `ReplaySample`, `DivergenceReport`, `SubjectDek` |
| `baseline-output-store` | `oya-foundry-eval-baseline-output-store-{kernel,domain,usecase,api,adapter,adapter-s3,app}` | Cosign-verified baseline read/write; per-subject-keyed envelope; DSR shred surface | `BaselineOutput`, `EnvelopeKey`, `ShredEvent` |

Total crates introduced: **49** (6 eval-set-registry + 11 eval-runner + 9 parity-analyzer + 8 replay-engine + 7 baseline-output-store + shared `app` composition root not double-counted). For the artifact pack we ship 12 of these crates initially (kernel + domain + usecase + api + adapter + adapter-s3 + adapter-clickhouse + adapter-gpu + rest + worker + sdk + app for the eval-runner BC; other BCs incremental).

Naming justification — `eval-runner`:

```
NAME: oya-foundry-eval-eval-runner-<layer>
JUSTIFICATION:
- microservice = foundry-eval (microservices/intelligence-eval/ per ADR-0131)
- bc-tokens = eval-runner: primary BC for eval-set execution; sibling BCs
  (eval-set-registry, parity-analyzer, replay-engine, baseline-output-store)
  justify explicit BC token per ADR-0056 v4.1 BC-optionality rule.
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical enum.
  - kernel: port-trait + sealed-trait + entity types (EvalRun, EvalCaseResult,
    EvalAggregate, ProviderRoute). Zero I/O. data_class annotated.
  - domain: pure aggregate computation (pass-rate math, per-cohort rollup,
    threshold check) — no I/O, no async.
  - usecase: orchestrators reading eval-sets, dispatching cases to runners,
    composing aggregates, emitting verdicts via ports.
  - api: protocol-neutral typed I/O contracts.
  - adapter: protocol-neutral implementations (filesystem eval-set reader, etc.)
  - adapter-s3: backend-qualified adapter for baseline-output + replay-trace I/O
    (per ADR-0105 Amendment 3 *-adapter-<backend> pattern).
  - adapter-gpu: backend-qualified adapter for GPU-pool case execution (CUDA
    or ROCm shim; gVisor / Kata sandbox enforcement). Per ADR-0105 Amendment 3.
  - rest: HTTP handler/route layer; consumes -api types.
  - worker: nightly orchestrator + on-demand run executor (long-lived).
  - sdk: client library for capability owners (Rust + TS bindings) to author
    eval-sets locally + read verdict via API. Closes the OpenAI-Evals /
    Anthropic-evals SDK gap.
  - app: composition root binary.
- exemptions claimed: none. -adapter-{s3,gpu,clickhouse} all use canonical
  Amendment 3 pattern.
```

Naming justification — `parity-analyzer`:

```
NAME: oya-foundry-eval-parity-analyzer-<layer>
JUSTIFICATION:
- microservice = foundry-eval.
- bc-tokens = parity-analyzer: sibling BC; cohort-delta + in-house-cutover
  + competitor-parity-matrix emission.
- layer = <layer>: one crate per layer.
  - kernel: ParityReport, CohortDelta, InHouseCutoverVerdict entities.
  - domain: delta math (winning-margin, ε-bounded DP aggregation).
  - usecase: orchestrators reading two runs, computing delta, emitting verdict.
  - api: protocol-neutral typed contracts.
  - adapter: protocol-neutral implementations.
  - adapter-clickhouse: backend-qualified adapter for parity-analytics table
    INSERT + cohort-rollup SELECT (per ADR-0105 Amendment 3).
  - rest: HTTP handlers exposing per-capability parity history.
  - worker: long-lived parity-rollup worker (recomputes cohort deltas hourly).
  - app: composition root.
- exemptions claimed: none.
```

Naming justification — `replay-engine`:

```
NAME: oya-foundry-eval-replay-engine-<layer>
JUSTIFICATION:
- microservice = foundry-eval.
- bc-tokens = replay-engine: replay-trace sampling + deterministic-seed
  execution + divergence detection + per-subject-DEK shred.
- layer = <layer>:
  - kernel: ReplaySample, DivergenceReport, SubjectDek entities.
  - domain: divergence-tolerance arithmetic, deterministic-seed validation.
  - usecase: orchestrators sampling traces, replaying, comparing, emitting.
  - api: typed contracts.
  - adapter: protocol-neutral.
  - adapter-s3: S3-backed replay-trace fetch + per-subject-DEK shred.
  - worker: continuous replay-sample worker.
  - app: composition root.
- exemptions claimed: none.
```

Naming justification — `eval-set-registry`:

```
NAME: oya-foundry-eval-eval-set-registry-<layer>
JUSTIFICATION:
- microservice = foundry-eval.
- bc-tokens = eval-set-registry: per-capability eval-set index + version pin
  + Cosign verify.
- layer = <layer>: kernel (EvalSet, EvalCase, CosignAttestation) + domain
  (version-comparison logic) + usecase (orchestrator) + api (typed) + adapter
  (Postgres-backed metadata + Cosign-verify) + app (composition root).
- bc-tokens repeated form 'eval-eval-set-registry' is non-collapsing because
  µservice token 'foundry-eval' and bc token 'eval-set-registry' are distinct
  identifiers under BNF v4.1; the concatenated form
  oya-foundry-eval-eval-set-registry-<layer> is therefore canonical and the
  apparent redundancy is structural, not a naming smell.
- exemptions claimed: none.
```

Naming justification — `baseline-output-store`:

```
NAME: oya-foundry-eval-baseline-output-store-<layer>
JUSTIFICATION:
- microservice = foundry-eval.
- bc-tokens = baseline-output-store: per-case baseline output + per-subject DEK
  envelope + DSR shred.
- layer = <layer>: kernel (BaselineOutput, EnvelopeKey, ShredEvent) + domain
  (envelope arithmetic, shred audit) + usecase + api + adapter +
  adapter-s3 + app.
- exemptions claimed: none.
```

Layer mapping per BC (13-layer canonical enum from ADR-0105; `usecase` per ADR-0106):

| BC | kernel | domain | usecase | api | adapter | adapter-s3 | adapter-clickhouse | adapter-gpu | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `eval-set-registry` | y | y | y | y | y | — | — | — | — | — | — | y |
| `eval-runner` | y | y | y | y | y | y | — | y | y | y | y | y |
| `parity-analyzer` | y | y | y | y | y | — | y | — | y | y | — | y |
| `replay-engine` | y | y | y | y | y | y | — | — | — | y | — | y |
| `baseline-output-store` | y | y | y | y | y | y | — | — | — | — | — | y |

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `EvalSetRepository` | `oya-foundry-eval-eval-set-registry-kernel` | `-adapter` (Postgres + filesystem reader) | `INTERNAL_ONLY` (manifest content) |
| `CosignVerifier` | `oya-foundry-eval-eval-set-registry-kernel` | `-adapter` (Sigstore Cosign Rust SDK) | `INTERNAL_ONLY` |
| `EvalRunner` | `oya-foundry-eval-eval-runner-kernel` | `-usecase` (orchestrator) | `BEHAVIORAL_TENANT_PRODUCT` |
| `CaseDispatcher` | `oya-foundry-eval-eval-runner-kernel` | `-adapter-gpu` (Kubernetes Job dispatcher; CUDA / ROCm shim) | `BEHAVIORAL_TENANT_PRODUCT` |
| `EvalRunStore` | `oya-foundry-eval-eval-runner-kernel` | `-adapter-s3` (S3 PUT per run; S3 GET per replay) | `BEHAVIORAL_TENANT_PRODUCT` + `AUDIT` |
| `ParityAnalyzer` | `oya-foundry-eval-parity-analyzer-kernel` | `-usecase` (delta + verdict orchestrator) | `BEHAVIORAL_TENANT_PRODUCT` |
| `ParityAnalyticsStore` | `oya-foundry-eval-parity-analyzer-kernel` | `-adapter-clickhouse` (ClickHouse INSERT + rollup SELECT) | `BEHAVIORAL_TENANT_PRODUCT` |
| `ReplaySampler` | `oya-foundry-eval-replay-engine-kernel` | `-usecase` (sampling orchestrator) | `BEHAVIORAL_TENANT_PRODUCT` + `PII_QUASI_IDENTIFIER` |
| `DivergenceDetector` | `oya-foundry-eval-replay-engine-kernel` | `-domain` (pure divergence arithmetic) | — |
| `SubjectDekStore` | `oya-foundry-eval-replay-engine-kernel` + `-baseline-output-store-kernel` | `-adapter-s3` (KMS-wrapped per-subject DEKs; shred = DEK delete) | `SECRET` + `AUDIT` |
| `BaselineOutputStore` | `oya-foundry-eval-baseline-output-store-kernel` | `-adapter-s3` (Cosign-verified S3 object) | `BEHAVIORAL_TENANT_PRODUCT` + per-case data class |
| `EvalEvidenceEmitter` | `oya-foundry-eval-eval-runner-kernel` | `-adapter` (foundry-evidence client) | `AUDIT` |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `oya-check-data-class` LEAN lane refuses unannotated fields at PR-time.

Cross-product rule: `foundry-eval` MUST NOT import any other product µservice crate at any layer. All cross-product flows go through Workflow (events) or Ontology (entity reads/writes). LEAN-A2 CI lane enforces. `foundry-eval` consumes `foundry-runtime` (to invoke capabilities under eval) and emits to `foundry-evidence` exclusively via the Workflow event topology declared below.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice foundry-eval` — dependency-direction
- `oya gate validate lean-a2 --microservice foundry-eval` — cross-product-refusal
- `oya gate validate port-location --microservice foundry-eval` — ports in kernel
- `oya gate validate layer-correctness --microservice foundry-eval` — layer enum match
- `oya gate validate per-microservice-layout --microservice foundry-eval` — ADR-0131 conformance
- `oya gate validate statelessness --microservice foundry-eval`
- `oya gate validate shardability --microservice foundry-eval`
- `oya gate validate foundry-eval-coverage` — refuses capability publish without eval-set
- `oya gate validate foundry-eval-adversarial-coverage` — refuses publish without 4 adversarial sub-cohorts
- `oya gate validate foundry-eval-linguistic-coverage` — refuses publish without KR + JP + EN minima
- `oya gate validate foundry-eval-replay-determinism` — refuses model upgrade when ≥ 100ms divergence breach

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine / DAG |
|---|---|---|---|
| `EvalRunStarted` | eval-runner dispatches first case of a run | `foundry-evidence`, `axis-foundry` dashboards | eval-run state machine (started → running → completed / failed) |
| `EvalRunCompleted` | eval-runner finalises aggregate | `foundry-runtime` (publish-gate), `foundry-providers` (router), `foundry-evidence` | as above |
| `ParityVerdictEmitted` | parity-analyzer emits delta verdict | `foundry-providers` (router-preference gate), `foundry-evidence` | parity-state-machine |
| `ReplayDivergenceDetected` | replay-engine finds ≥ 100ms divergence on deterministic-seed case | `foundry-runtime` (model-upgrade gate), `axis-foundry`, `foundry-evidence` | divergence-investigation-state |
| `InHouseCutoverEligible` | parity-analyzer determines in-house variant beats provider per-cohort | `foundry-providers` (router preference), `axis-foundry`, ADR-0026 cutover automation | in-house-cutover-state |
| `EvalSubjectShred` | DSR cascade fires per-subject-DEK delete | `foundry-evidence`, `tenancy` (DSR ledger), audit chain | erasure-state (per Bominal ADR-0043) |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `CapabilityPublishRequested` | `foundry-runtime` | `eval-runner` | dispatch publish-gate run; emit `EvalRunCompleted` |
| `RoutingPreferenceChangeRequested` | `foundry-providers` | `parity-analyzer` | dispatch A/B verdict; emit `ParityVerdictEmitted` |
| `ModelUpgradeProposed` | `foundry-runtime` | `replay-engine` | sample + replay; emit `ReplayDivergenceDetected` if breach |
| `EraseSubjectRequested` | `tenancy` (DSR cascade) | `replay-engine` + `baseline-output-store` | shred per-subject DEKs; emit `EvalSubjectShred` |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `EvalRun{capability, version, route, started_at, aggregate, passed}` | `evaluates→Capability` | `eval-runner` | Ed25519 via `foundry-evidence` |
| `ParityReport{capability, route_a, route_b, cohort_deltas, verdict}` | `compares→Capability` | `parity-analyzer` | Ed25519 |
| `ReplayDivergence{capability, sample_id, divergence_ms, seed, signature}` | `regresses→Capability` | `replay-engine` | Ed25519 |
| `BaselineOutput{case_id, content_hash, envelope_key_id}` | `baseline_for→EvalCase` | `baseline-output-store` | Ed25519 |

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `Capability` (from `foundry-runtime`) | `eval-set-registry` | `filter(active=true)` to enumerate capabilities requiring eval-set coverage |
| `Provider` (from `foundry-providers`) | `eval-runner` | route resolution for case dispatch |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| OpenAI | OpenAI Evals (github.com/openai/evals) | per-task eval registry; pluggable graders; GitHub-checked-in eval YAML | `github.com/openai/evals` |
| Anthropic | Internal evals harness + responsible-scaling evals | adversarial cohort patterns (prompt-injection / data-class / autonomy bypass) | Anthropic responsible-scaling policy; Apollo Research evaluations |
| LangSmith | LangSmith Evals (langchain/langsmith) | trace-based eval + replay; LLM-as-judge; per-experiment dashboards | `docs.langchain.com/langsmith/evaluation` |
| Patronus AI | Patronus eval API | adversarial + safety cohorts; hosted eval-as-a-service | `patronus.ai/docs` |
| Braintrust | Braintrust eval platform | per-experiment scoring; LLM-as-judge; CI integration | `braintrust.dev/docs` |
| Inspect AI (UK AISI) | Inspect AI framework | adversarial + autonomy evaluations; reproducible runs | `github.com/UKGovernmentBEIS/inspect_ai` |

Key parity gaps to close (ordered by priority):

1. **Self-hosted, no vendor coupling, signed eval-sets**: OpenAI Evals is GitHub-only (no signing); LangSmith/Patronus/Braintrust are SaaS. Target: Cosign-signed eval-sets + self-hosted runner + per-subject-keyed replay store.
2. **Capability-publish gate**: none of the competitors gate capability publish on the eval verdict at the runtime layer; LangSmith/Braintrust gate at CI only. Target: ledger-backed, signed, per-capability gate that `foundry-runtime` enforces at admission.
3. **In-house-model cutover decision substrate**: no competitor explicitly couples eval substrate to provider-vs-in-house cutover. Target: per-cohort parity-win → cutover-eligibility verdict consumed by `foundry-providers`.
4. **Replay-determinism ≤ 100ms divergence tolerance**: LangSmith does trace-replay but does not assert deterministic-seed divergence bounds. Target: per ADR-0024 + per `policy/replay-determinism.md`.
5. **EU AI Act §15+§17 evidence-grade out-of-box**: no competitor emits EU AI Act §15 (accuracy) + §17 (logging) compliant artefacts by construction. Target: every eval-run carries the §15 + §17 evidence schema.

## Performance Targets

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Eval-set total wall time (1000 cases) | ≤ 5 min | ≤ 15 min | ≤ 30 min | GPU pool autoscaled |
| Replay-determinism divergence | ≤ 50 ms | ≤ 100 ms | ≤ 150 ms | per ADR-0024 |
| Publish-gate latency | ≤ 250 ms | ≤ 1 s | ≤ 2 s | hot path |
| Parity verdict latency | ≤ 200 ms | ≤ 1 s | ≤ 3 s | post run-complete |
| GPU runner pool throughput | — | 32 parallel cases | — | per cluster; HPA bounded by GPU node availability |
| ClickHouse cohort-rollup query | ≤ 50 ms | ≤ 200 ms | ≤ 500 ms | ClickHouse MergeTree tuned |
| Postgres eval-set metadata read | ≤ 5 ms | ≤ 20 ms | ≤ 50 ms | per primary key |

Error budget:
- Monthly error budget for publish-gate path: 0.1 % (≈ 44 min/month).
- Burn-rate alarm on publish-gate: 14.4× burn over 1h triggers page.
- Error budget policy: `microservices/intelligence-eval/runbooks/error-budget-policy.md`.

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed`. Rationale: eval-runner workers are stateless (re-derivable from eval-set + baseline-output store); Postgres for eval-set metadata; S3 for baseline-outputs and replay traces; ClickHouse for parity-analytics; GPU pool ephemeral.

**Active-active compatibility**: `stateless-compatible` for runners + worker; Postgres uses logical-replication-friendly schemas (per `policy/sharding.md`); ClickHouse is horizontally shardable via `Distributed` engine; S3 is multi-region replicable.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Eval-runner workers | 4 replicas | 50 replicas | queue depth > 60s of cadence |
| GPU runner pool (eval cases) | 8 GPUs | 64 GPUs | pending-case backlog > 5 min |
| Postgres eval-set metadata QPS | 1 k | 10 k | replica CPU > 70% |
| ClickHouse parity-analytics QPS | 100 | 1 k | ClickHouse CPU > 70% |
| S3 baseline-output throughput | 1 GB/s | 10 GB/s | object-storage SLO breach |
| Replay traces ingest | 10⁶/day | 10⁸/day | S3 PUT rate-limit warning |

Scale-out policy:
- Kubernetes HPA: eval-runner workers scale on CPU `>70%`; min 2 max 50.
- GPU pool: Karpenter-managed (per ADR-0198) via GPU-class NodePool; min 2 max 64 GPUs per cell; spot-eligible for non-critical (nightly) workloads.
- Postgres: streaming-replication primary + read-replicas; HA promotion via Patroni.
- ClickHouse: 3-replica ZooKeeper-coordinated cluster; shard-by-week partitioning.
- Pre-warmed pool: 2 GPU pods + 2 worker pods; cold-start budget ≤ 60s for GPU, ≤ 500ms for worker.

Cross-region story:
- M01 launch: per-pack region (pack-kr → KR; pack-eu → EU; pack-us → US; pack-us-healthcare → US HIPAA-eligible).
- Cross-region replication of eval-sets (read-only): via S3 cross-region replication + Cosign signature carries across.
- Cross-region replication of baseline-outputs / replay traces: forbidden by default (residency); allowed only with explicit tenant SCC.

Sharding:
- Eval-runs partition by `(capability_id, week)`; ClickHouse `parity_analytics` partitions by week.
- Replay traces partition by `(capability_id, day, tenant_id)`.
- Per-subject DEKs partition by `subject_id` (per-tenant-keyed).
- `oya-check-shardability-cli` CI lane verifies partition-key presence.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | Eval-set manifest at `eval-sets/<capability>/v<n>.evalset.yaml` validates against schema + carries Cosign signature | `cargo run -p oya-foundry-eval-eval-set-registry-rest -- validate <path>` exit 0 |
| AC-02 | Publish-gate refuses publish when eval-set missing | e2e under `tests/e2e/publish-gate-no-eval-set.rs` |
| AC-03 | Publish-gate refuses publish when adversarial cohort fails | e2e |
| AC-04 | Nightly run cadence triggers within 60s of cron tick | timed e2e |
| AC-05 | Replay determinism divergence ≤ 100ms p99 on deterministic-seed cases | tests/load/replay-determinism.rs |
| AC-06 | Parity verdict emission per-cohort within 1s p99 of run completion | timed e2e |
| AC-07 | DSR cascade shreds per-subject DEKs within 30d SLA; replay attempts on shredded subject return `EVT-REPLAY-SUBJECT-SHRED` | e2e under `tests/e2e/dsr-shred-cascade.rs` |
| AC-08 | InHouseCutoverEligible verdict emits only when per-cohort parity-win is achieved (no partial wins) | parity-analyzer unit + e2e |
| AC-09 | All Layer-A Helm charts deploy clean against a kind cluster | CI lane `foundry-eval-iac-smoke` |
| AC-10 | `oya gate validate per-microservice-layout --microservice foundry-eval` exit 0 | ADR-0131 lane |
| AC-11 | `oya gate validate authority-cohesion` exit 0 — HG-FE registered | ADR-0123 lane |
| AC-12 | EU AI Act §15 evidence schema: every EvalRun emission carries `eu_ai_act_section_15_accuracy_metric` + `eu_ai_act_section_17_logging_payload` fields | schema regression test |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | LLM-as-judge graders: which model is the canonical judge for HumanJudged eval cases? Default: rotate quarterly across two top providers + one in-house variant; per-quarter consistency check (κ ≥ 0.7) | axis-foundry | resolved in IP-006 |
| 2 | GPU pool sharing policy: dedicated foundry-eval pool vs shared with foundry-runtime? | ops-sre-reliability + axis-foundry | dedicated for M01 (eval cadence + reproducibility); ADR-#### if pressure changes |
| 3 | Replay-trace retention upper bound (currently 24 months per ADR-0024); does pack-us-healthcare extend to 6y per HIPAA? | council-privacy | pack-us-healthcare overlay extends to 6y; cost-budget reflects |
| 4 | Multi-arm A/B testing (> 2 routes) vs strictly pairwise: which does parity-analyzer model first? | axis-foundry | strictly pairwise for M01; multi-arm in M02 |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0024 | Foundry eval harness and replay | the design this PRD scaffolds |
| ADR-0026 | In-house AI model substrate roadmap | foundry-eval is the cutover gate |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0106 | application → usecase rename | new crates use `usecase` |
| ADR-0139 | Agentic SLO-gated promotion | foundry-eval inherits the SLO gate model |
| ADR-0131 | Per-microservice flat layout | this PRD authored under it |
| ADR-0132 | Product-platform-and-bundle dissolution | foundry-eval split from foundry product bundle |
| ADR-0133 | Industry best-practice conformance program | HG-FE bar |
| ADR-0123 | Hyperscaler maturity claim gate | HG-FE registers here |
| ADR-0116 | Retire external agent-coordination tooling | oya vcs primitives used throughout |
