---
doc_class: Phase
microservice: feature-flags
phase_id: PHASE-01
status: In Progress
date: 2026-05-20
milestone: M01-foundation
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0139
  - ADR-0159
  - ADR-0160
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0248
  - ADR-0250
  - ADR-0253
companion_docs:
  - microservices/feature-flags/PRD.md
  - microservices/feature-flags/ARCHITECTURE.md
  - microservices/feature-flags/manifest.json
planned_enforcement_ref: oya-governance-microservice-doc-suite
---

# PHASE-01 — LaunchDarkly-class Flag Substrate

## Phase objective

Deliver a production-grade, OpenFeature-compliant feature-flag substrate that matches or exceeds the capability parity of LaunchDarkly, Split.io, Statsig, Optimizely, GrowthBook, Unleash, and the OpenFeature specification. This phase establishes the canonical flag kernel, all six bounded contexts (flag-management / targeting-rules / experiment-design / metric-attribution / rollout-orchestration / safety-killswitch), the Cedar authorization surface, the OpenFeature gRPC + REST provider, and the per-language SDK baseline.

Hyperscaler precedent: LaunchDarkly's relay-proxy + streaming architecture; Statsig's server-side evaluation + Bayesian experimentation; Cloudflare's Workers KV for flag state propagation.

## Why this phase now

Feature flags are the third orthogonal gating tier (alongside ChangeSet gates ADR-0110 and progressive delivery ADR-0160). Without runtime flag substrate, every feature rollout requires a deployment. With it, product teams can decouple deployment from release, run statistically rigorous experiments, and operate emergency kill-switches without rollbacks. LaunchDarkly reported that teams using feature flags ship 50× more frequently with 73× fewer outages (2023 State of Feature Management report).

## Bounded contexts delivered

### BC-1: Flag Management

**Purpose:** CRUD lifecycle for flag definitions.

**Crates:**
- `oya-feature-flags-flag-kernel` — `FlagDefinition`, `FlagKey`, `FlagVariant`, `FlagLifecycleState` value types.
- `oya-feature-flags-flag-domain` — `FlagDomainService`, `FlagRepository` trait, `FlagLifecyclePolicy`.
- `oya-feature-flags-flag-usecase` — `CreateFlagUseCase`, `MutateFlagUseCase`, `ArchiveFlagUseCase`, `UndoFlagMutationUseCase`.
- `oya-feature-flags-flag-adapter-postgres` — Postgres + Citus backed `FlagRepository` implementation.
- `oya-feature-flags-flag-api` — gRPC service definition (via proto3).
- `oya-feature-flags-flag-rest` — Axum-based REST handlers.
- `oya-feature-flags-flag-app` — Composition root; wires all layers.
- `oya-feature-flags-flag-worker` — Background workers: replication propagator, stale-flag scanner, pack-overlay applicator.

**Storage:** `flags` table (tenant_id, flag_key, definition JSONB, lifecycle_state, pack_overrides JSONB, created_at HLC, updated_at HLC). Patroni-replicated Postgres; Citus for shard-by-tenant_id.

**Throughput capacity math:**
- Flag definitions per tenant: ≤10,000 (p99 tenant); median ~200.
- Write rate: ≤60 mutations/min per tenant (rate-limited). Platform-wide: 60 × 100k tenants = 6M mutations/min max (engineering ceiling; typical <<0.01% of tenants active at peak).
- Postgres write throughput: 50k writes/s per Patroni primary; 60× headroom above typical load.
- Cross-region replication lag: ≤5s (Patroni streaming replication + WAL shipping).

### BC-2: Targeting Rules

**Purpose:** Per-tenant Cedar-based targeting predicates; percentage rollout hashing.

**Crates:**
- `oya-feature-flags-targeting-kernel` — `TargetingRule`, `EvaluationContext`, `RolloutBucket`.
- `oya-feature-flags-targeting-domain` — `TargetingRuleEvaluator` (library-first Cedar), `RolloutHasher` (stable HMAC-SHA256).
- `oya-feature-flags-targeting-usecase` — `EvaluateTargetingUseCase`.
- `oya-feature-flags-targeting-adapter` — Cedar fragment cache adapter; Wasm pre-compilation cache.

**Rollout hash algorithm (LaunchDarkly precedent):**
```
bucket = SHA256(tenant_id + "." + flag_key + "." + user_id + "." + salt) % 10000 / 100.0
// Result: float [0.0, 100.0); deterministic per (tenant, flag, user, salt)
// Salt rotated per experiment activation to prevent cross-experiment correlation
```

### BC-3: Experiment Design

**Purpose:** A/B + multivariate experiment design, activation, and statistical conclusion.

**Crates:**
- `oya-feature-flags-experiment-kernel` — `Experiment`, `ExperimentVariant`, `ExperimentMetric`, `StatSigResult`.
- `oya-feature-flags-experiment-domain` — `ExperimentDomainService`, `BayesianScorer`, `FrequentistScorer`, `SequentialTester`.
- `oya-feature-flags-experiment-usecase` — `DesignExperimentUseCase`, `ActivateExperimentUseCase`, `ConcludeExperimentUseCase`.
- `oya-feature-flags-experiment-adapter` — Integration with metric-attribution BC; Statsig-compatible event ingestion.

**Statistical methods:**
1. **Bayesian posterior**: Beta-Binomial model for conversion rates; Thompson sampling for multi-armed bandits.
2. **Frequentist two-proportion z-test**: for sample-size-sufficient experiments.
3. **Sequential testing (mSPRT)**: mixture Sequential Probability Ratio Test; allows early stopping without inflating Type I error.
4. **Chi-squared goodness-of-fit**: for variant assignment balance verification (detect SRM — Sample Ratio Mismatch).
5. **Mann-Whitney U test**: for non-normal continuous metrics.

Hyperscaler precedent: Statsig's Bayesian + frequentist hybrid; Optimizely's sequential testing (Stats Accelerator); Netflix's interleaving experiments.

### BC-4: Metric Attribution

**Purpose:** Attribute metric events (conversions, errors, latency) to experiment variants.

**Crates:**
- `oya-feature-flags-metric-kernel` — `MetricEvent`, `AttributionWindow`, `ExperimentAssignmentId`.
- `oya-feature-flags-metric-domain` — `MetricAttributionService`, `ConversionFunnelAnalyzer`.
- `oya-feature-flags-metric-usecase` — `AttributeMetricUseCase`, `ComputeExperimentResultsUseCase`.
- `oya-feature-flags-metric-adapter` — AsyncAPI consumer for metric events; ClickHouse writer for analytics.

**Attribution window:** configurable per experiment (default: 14 days); controlled by `attribution_window_days` field. Events outside window excluded from analysis. Multi-touch attribution model: last-touch by default; configurable to first-touch or data-driven.

### BC-5: Rollout Orchestration

**Purpose:** Progressive rollout scheduling, canary cohort management, automated rollback triggers.

**Crates:**
- `oya-feature-flags-rollout-kernel` — `RolloutPlan`, `RolloutStage`, `RolloutTrigger`, `RolloutGate`.
- `oya-feature-flags-rollout-domain` — `RolloutOrchestrator`, `CanaryCohortManager`, `RolloutGateEvaluator`.
- `oya-feature-flags-rollout-usecase` — `ScheduleRolloutUseCase`, `AdvanceRolloutStageUseCase`, `RollbackRolloutUseCase`.
- `oya-feature-flags-rollout-adapter` — Integration with observability SLO substrate; reads SLO burn-rate for auto-rollback.

**Rollout stages (Flagger-compatible, ADR-0160):**
1. Stage 0: 0% (flag off; canary cohort only in shadow).
2. Stage 1: 1% (canary cohort; early signal).
3. Stage 2: 10% (wider canary; SLO gate).
4. Stage 3: 50% (pre-GA; SLO gate + manual review).
5. Stage 4: 100% (GA).

Auto-rollback trigger: if `oya_error_budget_burn_rate_1h > 5` during rollout stage ≥2, rollout halts and rolls back to previous stage.

### BC-6: Safety Kill-Switch

**Purpose:** Emergency flag disable with audit-sealed activation; life-safety bypass for emergency-services principals.

**Crates:**
- `oya-feature-flags-killswitch-kernel` — `KillSwitch`, `KillSwitchReason`, `KillSwitchActivation`.
- `oya-feature-flags-killswitch-domain` — `KillSwitchDomainService`, `EmergencyBypassEvaluator`.
- `oya-feature-flags-killswitch-usecase` — `EngageKillSwitchUseCase`, `DisengageKillSwitchUseCase`.
- `oya-feature-flags-killswitch-adapter` — Broadcast kill-switch activation to all cells within ≤1s (Kafka fanout).

**Kill-switch latency target:**
- Engagement: ≤100ms to first cell; ≤1s to all cells globally.
- Capacity math: kill-switch activation is a single Kafka message broadcast; Kafka fanout to N cells in O(1). At 50 cells: 50 × 20ms Kafka round-trip = 1s worst-case fan-out.

## Acceptance criteria

- [ ] `oya-feature-flags-flag-kernel` compiles with `deny(warnings)`.
- [ ] All 6 BCs have kernel + domain + usecase + adapter + rest crates scaffolded.
- [ ] Flag evaluation p99 ≤1ms (cell-local, no cache miss).
- [ ] Kill-switch engagement ≤1s to all cells globally.
- [ ] OpenFeature provider conformance tests pass.
- [ ] Cedar targeting-rule evaluation passes cross-tenant isolation test.
- [ ] Pack overlays applied correctly for HIPAA + PCI packs.
- [ ] Audit events emitted for all flag lifecycle transitions.
- [ ] `oya-governance-adr-adherence-matrix` green for rows 1-28.

## IPs in this phase

- IP-002 through IP-017 (see `manifest.json:ips`).

## Dependencies

- `microservices/tenancy/` — GA (tenant resolution).
- `microservices/policy-engine/` — GA (Cedar evaluation substrate).
- `microservices/observability/` — GA (SLO burn-rate for rollout gating).
- `microservices/cloud-secrets/` — GA (OpenBao credential sidecar).

## Failure modes in this phase

1. **Cedar fragment soak delay blocks rollout**: targeting-rule Cedar fragment requires ≥60s soak (ADR-0294); engineer expects instant activation. Mitigation: UI shows soak countdown; shadow-mode evaluation during soak.
2. **Cross-region replication lag during kill-switch**: kill-switch activated but DR-pair cell still evaluating old state for ≤5s. Mitigation: kill-switch uses Kafka broadcast path (≤1s) not Postgres replication path.
3. **Percentage-rollout hash collision**: two flags with same key pattern produce correlated rollout buckets. Mitigation: salt per experiment activation; SRM (Chi-squared) check at 1% stage.
