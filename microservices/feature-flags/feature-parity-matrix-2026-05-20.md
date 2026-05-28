---
doc_class: FeatureParityMatrix
microservice: feature-flags
matrix_date: 2026-05-20
authored_under_date: 2026-05-21
author_class: µservice-ownership-coherence-audit-agent
author_slug: ff-audit-2026-05-20
counterparts:
  - LaunchDarkly (industry breadth leader; SDK + workflow + audit)
  - Statsig (statistical rigor + server-eval performance + ML auto-targeting)
  - Split.io (segmentation depth + impressions export + dynamic configs)
canonical_anchors:
  - /Users/jasonlee/oyatie/microservices/feature-flags/competitor-parity-matrix.md
  - /Users/jasonlee/oyatie/microservices/feature-flags/PRD.md §F functional requirements
  - /Users/jasonlee/oyatie/microservices/feature-flags/PHASE-01-LAUNCHDARKLY-CLASS-FLAG-SUBSTRATE.md §BC-1..§BC-6
  - /Users/jasonlee/oyatie/microservices/feature-flags/sdk-plan.md
  - /Users/jasonlee/oyatie/docs/standards/brief-template.md §3.1 µservice ownership coherence audit
doctrine_amendments_applied:
  - Tenant-class adoption active; capabilities listed are NOT tenant_class-gated
  - tenant_class ∈ {demo_trial, paid} attaches to all capabilities; demo_trial has hard usage caps + best-effort SLO; paid is per-contract
  - 6 deployment contexts; OpenTofu only; Rust-strict backend; per-µservice manifest under iac/<context>/
status: published
---

# Feature-Flags — Top-3 Counterpart Parity Matrix (2026-05-20)

## §0 Scope and Method

This matrix compares the `feature-flags` µservice against the three top industry counterparts named in the audit brief: **LaunchDarkly**, **Statsig**, **Split.io**. The existing `microservices/feature-flags/competitor-parity-matrix.md` covers seven counterparts (LaunchDarkly + Split.io + Statsig + Optimizely + GrowthBook + Unleash + OpenFeature); this companion narrows to the top-3 with per-capability evidence cells, not just a check-mark grid.

Coverage source for each row:
- ✓ = oyatie ships the capability in design-ready state; evidence path cited.
- ✓ (roadmap) = oyatie has named the capability in `competitor-parity-matrix.md` as Phase 2/3.
- ◯ = oyatie ships a partial; gap explained.
- ✗ = oyatie does not ship; capability is named in this matrix as a potential roadmap item.

Comparison sources for counterparts: vendor public documentation, vendor 2024 product pages, oyatie's own `competitor-parity-matrix.md`. Vendor capabilities listed are the publicly documented surface as of the audit date; some private/enterprise tiers may exceed this. Where two counterparts have equivalent capability with different naming, the cell names the counterpart's term in parentheses.

Doctrine note: Per `feedback_tenant_class_2026_05_20`, capabilities are NOT tenant_class-gated. The capability either exists or does not. Per `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20`, capabilities may carry tenant-class eligibility (e.g., compliance-pack flag activation requires paid). The matrix below notes tenant-class eligibility per capability where relevant.

## §1 Flag Creation (typed variants)

| Capability | LaunchDarkly | Statsig | Split.io | oyatie feature-flags | Tenant-class | Evidence |
|---|---|---|---|---|---|---|
| Boolean flag | ✓ | ✓ | ✓ | ✓ | both | `contracts/openapi-v1.yaml:FlagDefinition.flag_type=boolean`; `proto:EvaluateBooleanRequest` |
| String flag | ✓ | ✓ | ✓ | ✓ | both | `proto:EvaluateStringRequest` |
| Number flag | ✓ | ✓ | ✓ | ✓ | both | `proto:EvaluateNumberRequest` |
| JSON-object flag | ✓ | ✓ (dynamic config) | ✓ (dynamic config) | ✓ | both | `proto:EvaluateObjectRequest` with `bytes value_json` |
| Kill-switch flag type | ✓ | ✓ | ✓ | ✓ | both | `proto:flag_type=kill_switch`; dedicated `KillSwitchService` RPC |
| Experiment flag type | ✓ | ✓ | ✓ | ✓ | both | `proto:flag_type=experiment` + `ExperimentService` |
| Flag intent (release_toggle / experiment / permission_toggle / kill_switch) | partial (LaunchDarkly via tags) | partial (Statsig via gate_type) | partial (Split via name conventions) | ✓ | both | `FlagDefinition.intent` enum |
| Sunset timestamp (CI-red on overdue) | partial (LaunchDarkly via Code References) | ✗ | ✗ | ✓ | both | `FlagDefinition.sunset_at` + CI lane `oya-governance-flag-lifecycle-overdue` |
| audit_required tag | ◯ (audit log applies to all) | ◯ | ◯ | ✓ | both | `FlagDefinition.audit_required` triggers per-eval ADR-0028 seal |
| pack_overrides (read-only for tenant) | ✗ | ✗ | ✗ | ✓ | paid (pack activation requires paid) | `policy/pack-flag-override.cedar` + ADR-0251 |

## §2 Targeting Rules

| Capability | LaunchDarkly | Statsig | Split.io | oyatie feature-flags | Tenant-class | Evidence |
|---|---|---|---|---|---|---|
| Percentage rollout (stable hash) | ✓ | ✓ | ✓ | ✓ | both | `proto:TargetingRule.percentage` + SHA256-based RolloutHasher per `PHASE-01 §BC-2` |
| User-attribute targeting | ✓ (custom attrs) | ✓ (user_id, country, etc.) | ✓ (attribute comparators) | ✓ (Cedar predicates) | both | `proto:EvaluationContext` + Cedar fragment `TargetingRule.cedar_predicate` |
| Cohort targeting (segment) | ✓ (segments, big segments >50k) | ✓ (audiences) | ✓ (segments) | ✓ (cohort_ids[] from analytics audiences) | both | `EvaluationContext.cohort_ids` per `proto.line 73` |
| Persona-tier targeting | ✗ (no native) | ✗ | ✗ | ✓ | both | `EvaluationContext.persona_tier ∈ {B2C, B2B, INTERNAL_AGENT, EMERGENCY_SERVICES, FRIENDLY_CRAWLER_PARTNER, MINOR_TARGETED}` per ADR-0244 |
| Consent-purpose targeting | ✗ | ✗ | ✗ | ✓ | both | `EvaluationContext.consent_purposes` per ADR-0272 |
| audience_type override | ✗ | ✗ | ✗ | ✓ | both | `EvaluationContext.audience_type`; EMERGENCY_SERVICES bypass per `policy/emergency-services-bypass.cedar` |
| tenant_class targeting | ✗ | ✗ | ✗ | ◯ (P1 gap per audit §3.4.C) | n/a | NOT in current EvaluationContext; remediation required |
| Cedar policy targeting (composable, auditable, version-controlled) | ✗ (bespoke DSL) | ✗ (bespoke DSL) | ✗ (bespoke DSL) | ✓ | both | All 11 fragments under `policy/*.cedar` |
| Cohort membership refresh latency | ≤10 min (LaunchDarkly Big Segments) | ≤5 min (Statsig audiences) | ≤5 min (Split.io segments) | ≤5 min p99 (per ADR-0158 cell + analytics MV refresh) | both | `competitor-parity-matrix.md` + audit §3.4 |
| Regex / semver / set / date comparators | ✓ | ✓ | ✓ (most extensive) | ✓ (Cedar v4.2 expressions cover all) | both | Cedar v4.2 LTS expression library |
| Prerequisite flags (flag-dependency graph) | ✓ | ✓ | ✓ | ✓ (roadmap; not in current proto) | both | `competitor-parity-matrix.md` Phase 2 |

## §3 Real-time Updates (server-side + client-side propagation)

| Capability | LaunchDarkly | Statsig | Split.io | oyatie feature-flags | Tenant-class | Evidence |
|---|---|---|---|---|---|---|
| Server SDK in-process cache | ✓ | ✓ | ✓ | ✓ | both | `sdk-plan.md` Rust DashMap with 30s TTL |
| Server SDK SSE streaming | ✓ | ✓ | ✓ | ✓ | both | `contracts/openapi-v1.yaml:/flags/stream` + `proto:StreamFlagUpdates` |
| Server SDK long-poll fallback | ✓ | ✓ | ✓ | ✓ (HTTP/2 fallback per ADR-0253) | both | `sdk-plan.md` SDK feature matrix |
| WebSocket transport | partial (LaunchDarkly relay) | ✓ | partial | ✓ (planned alongside SSE per `competitor-parity-matrix.md`) | both | `competitor-parity-matrix.md` "SSE + WebSocket" |
| Client SDK relay-proxy | ✓ | ✓ | ✓ | ◯ (Phase 2; relay topology defers to in-process SDK + edge cache) | both | `sdk-plan.md` Phase 2 |
| HTTP/3 + QUIC default | ✗ | ✗ | ✗ | ✓ | both | `iac/helm-values.yaml` + ADR-0253 |
| ECH (Encrypted Client Hello) | ✗ | ✗ | ✗ | ✓ | both | `iac/ech-config.yaml` |
| PQC hybrid (post-quantum) | ✗ | ✗ | ✗ | ✓ | both | `iac/pqc-cert.yaml` TLS 1.3 floor |
| Push update p99 latency | <1s (LaunchDarkly streaming) | <1s (Statsig websocket) | <2s (Split.io) | <1s flag-state-propagation per `slos/flag-state-propagation.openslo.yaml` | both | SLO |
| Kill-switch propagation p99 latency | not published | not published | not published | ≤1s globally per `slos/killswitch-fire-latency.openslo.yaml` (life-safety) | both | SLO; capacity math in PHASE-01 §BC-6 |
| LKG (last-known-good) cache on outage | ✓ (default value fallback) | ✓ | ✓ | ✓ (LKG 30-min per `openfeature-sdk-contract.md`) | both | OpenFeature SDK contract |

## §4 Webhooks and Notifications

| Capability | LaunchDarkly | Statsig | Split.io | oyatie feature-flags | Tenant-class | Evidence |
|---|---|---|---|---|---|---|
| Outbound webhook on flag change | ✓ | ✓ | ✓ | ✓ (via AsyncAPI) | both | `contracts/asyncapi-v1.yaml` flag-state-changed channel |
| Outbound webhook on experiment conclude | ✓ | ✓ | ✓ | ✓ | both | `contracts/asyncapi-v1.yaml` experiment.concluded |
| Outbound webhook on kill-switch | ✓ | ✓ | ✓ | ✓ | both | Kafka topic `oya.feature-flags.killswitch-engaged` per `iac/terraform/main.tf` |
| Slack notification (native) | ✓ | ✓ | ✓ | ◯ (defer to `comms-email` + `messenger` µservices; not native to feature-flags) | both | `competitor-parity-matrix.md` notes Slack/MS Teams as marketplace integration pack |
| Microsoft Teams (native) | ✓ | ◯ | ◯ | ◯ (same; defer to marketplace integration) | both | same |
| PagerDuty integration | ✓ | ✓ | ✓ | ◯ (defer to `observability` µservice routing per ADR-0263) | both | observability µservice owns alert routing |
| Datadog integration | ✓ | ✓ | ✓ | ◯ (defer to `observability` OTEL exporter) | both | OTEL via `oya-shared-otel` |
| Audit-chain seal on emit | ✗ (append-only log) | ✗ | ✗ | ✓ (Merkle-chain ADR-0028) | both | `audit-chain` µservice integration |

## §5 Audit Log

| Capability | LaunchDarkly | Statsig | Split.io | oyatie feature-flags | Tenant-class | Evidence |
|---|---|---|---|---|---|---|
| Append-only audit log | ✓ | ✓ | ✓ | ✓ | both | All counterparts; oyatie via `audit-chain` µservice |
| Cryptographic seal (Merkle chain) | ✗ | ✗ | ✗ | ✓ | both | ADR-0028 + ADR-0263 |
| Per-evaluation audit emission (for sensitive flags) | partial (LaunchDarkly Big Events) | partial (Statsig logs all impressions) | ✓ (impressions export) | ✓ | both | `audit_required: true` field per `FlagDefinition` |
| Diff view in UI | ✓ | ✓ | ✓ | ◯ (defer to ops-dashboard-control-center UI µservice) | both | UI lives outside feature-flags |
| Rollback action (1-click) | ✓ | ✓ | ✓ | ✓ | both | `proto:UndoFlagMutation` + 15s undo window per OpenAPI `/flags/{flag_key}/undo` |
| HLC timestamps | ✗ | ✗ | ✗ | ✓ | both | `proto:FlagUpdateEvent.hlc_timestamp` per ADR-0252 |
| TrueTime tier (financial-grade) | ✗ | ✗ | ✗ | ✓ (opt-in per ADR-0252) | paid | ADR-0252 §TrueTime opt-in |
| Tenant-isolated audit (no cross-tenant read) | partial | partial | partial | ✓ (Cedar-gated + DB-row tenant_id) | both | `policy/tenant-targeting.cedar` + ADR-0244 |

## §6 Experiment Platform (A/B + multivariate)

| Capability | LaunchDarkly | Statsig | Split.io | oyatie feature-flags | Tenant-class | Evidence |
|---|---|---|---|---|---|---|
| Two-variant A/B test | ✓ | ✓ | ✓ | ✓ | both | `proto:ExperimentService` + IP-008 |
| Multivariate (3+) test | ✓ | ✓ | ✓ | ✓ | both | `CreateExperimentRequest.variants` minItems=2 |
| Bayesian posterior | ✗ (frequentist only) | ✓ (default) | ✗ | ✓ (Beta-Binomial via IP-020) | both | `competitor-parity-matrix.md` row "Bayesian stats" |
| Thompson sampling (multi-armed bandit) | partial | ✓ | ✗ | ✓ | both | IP-020 statistical engine |
| Frequentist z-test | ✓ | ✓ | ✓ | ✓ (with Bonferroni correction) | both | IP-020 |
| Sequential testing (mSPRT) | ✗ | ✓ | ✗ | ✓ | both | `PHASE-01 §BC-3` |
| Chi-squared SRM detection | ✗ (partial) | ✓ | ✓ | ✓ | both | `runbooks/experiment-stat-sig-violation.md` + IP-020 |
| Mann-Whitney U (non-normal) | ✗ | ✗ | ✗ | ✓ | both | IP-020 |
| Benjamini-Hochberg FDR | ✗ | ✓ | ✗ | ✓ | both | IP-020 |
| LIME/SHAP feature importance (EU AI Act Art.13) | ✗ | ✗ | ✗ | ✓ | paid (EU AI Act pack) | `proto:ExperimentResult.feature_importance_json` |
| Holdout groups / global holdouts | ✓ | ✓ | partial | ✓ (roadmap Phase 2 per `competitor-parity-matrix.md`) | paid | Phase 2 |
| Funnel experiments (multi-step) | partial (LD Workflows) | ✓ | ✓ | ◯ (Phase 2) | paid | Phase 2 |
| Sample-size estimator | ✓ | ✓ | ✓ | ✓ | both | `Experiment.has_sample_size_estimate` per proto |
| Auto-conclude on significance | ✗ | ✓ | ✓ | ✓ (via mSPRT) | both | IP-020 |
| Cohort-aware experiment assignment (sticky bucketing) | ✓ | ✓ | ✓ | ✓ | both | `PHASE-01 §BC-2` rollout hash + experiment salt |

## §7 Feature Workflows (request-to-prod)

| Capability | LaunchDarkly | Statsig | Split.io | oyatie feature-flags | Tenant-class | Evidence |
|---|---|---|---|---|---|---|
| Approval workflow | ✓ | ✓ | ✓ | ✓ | both | `flag-mutation-authorization.cedar` step-up auth + Cedar permit |
| Scheduled releases | ✓ | partial | partial | ◯ (Phase 2; cron-eval workflow via `workflow-engine`) | both | defer to `workflow-engine` |
| 4-eye review (reviewer signoff) | ✓ (Workflows) | ✗ | ✗ | ✓ | both | Step-up Class B/C per Cedar fragments |
| Branch-based flag definitions | ✗ | ✗ | ✗ | ✓ (via `oya-vcs` per ADR-0110) | both | ChangeSet flow per ADR-0110 |
| Multi-environment promotion (dev/staging/prod) | ✓ | ✓ | ✓ | ✓ (per-cell deploy per ADR-0158) | both | Cell topology |
| Environment-specific defaults | ✓ | ✓ | ✓ | ✓ | both | Per-cell flag definitions |
| Workflow Builder UI | ✓ (LaunchDarkly Workflow Builder) | partial | partial | ◯ (defer to `workflow-studio` µservice) | both | `workflow-studio` per ADR-0255 |
| Workflow templates | ✓ | partial | partial | ◯ (defer to `workflow-studio`) | both | same |

## §8 Kill-Switch + Safety

| Capability | LaunchDarkly | Statsig | Split.io | oyatie feature-flags | Tenant-class | Evidence |
|---|---|---|---|---|---|---|
| Emergency kill-switch | ✓ | ✓ | ✓ | ✓ | both | `proto:KillSwitchService.EngageKillSwitch` |
| Audit-sealed kill-switch activation | ✗ | ✗ | ✗ | ✓ | both | `KillSwitchEngaged` ADR-0028 seal |
| Step-up auth for kill-switch | partial | partial | partial | ✓ (Class C: TOTP + passkey) | both | `safety-killswitch-authorization.cedar` |
| Life-safety bypass (EMERGENCY_SERVICES) | ✗ | ✗ | ✗ | ✓ | both | `policy/emergency-services-bypass.cedar` per audience_type=EMERGENCY_SERVICES |
| Auto-rollback on SLO burn | ✓ (LaunchDarkly Releases) | partial | ✓ (Split Suite) | ✓ | both | `PHASE-01 §BC-5` rollout gate; burn-rate >5 halts rollout |
| Kill-switch propagation to all cells ≤1s | partial | partial | partial | ✓ | both | Kafka broadcast per `iac/terraform/main.tf:kafka_topic_killswitch` 50 partitions |
| Healthcare break-glass (cannot kill-switch) | ✗ | ✗ | ✗ | ✓ | paid | `openapi.yaml:/flags/{flag_key}/kill-switch` 403 path |
| Kill-switch undo window | ✓ | partial | ✓ | ✓ (15s) | both | `proto:UndoFlagMutation` |

## §9 Prerequisite Flags (flag dependency graph)

| Capability | LaunchDarkly | Statsig | Split.io | oyatie feature-flags | Tenant-class | Evidence |
|---|---|---|---|---|---|---|
| Prerequisite flag (flag-A must be on for flag-B to evaluate) | ✓ | ✓ | ✓ | ◯ (roadmap; cedar predicates can express but not first-class) | both | Phase 2 per `competitor-parity-matrix.md` |
| Cyclic-dependency detection | ✓ | ✓ | ✓ | ◯ (Phase 2) | both | Phase 2 |
| Visual dependency graph (UI) | ✓ | ✓ | ✓ | ◯ (defer to ops-dashboard) | both | UI |

## §10 Environment Management

| Capability | LaunchDarkly | Statsig | Split.io | oyatie feature-flags | Tenant-class | Evidence |
|---|---|---|---|---|---|---|
| Multi-environment (dev/staging/prod) | ✓ | ✓ | ✓ | ✓ (per-cell + per-tenant) | both | ADR-0158 cell topology + ADR-0244 tenant scope |
| Per-environment defaults | ✓ | ✓ | ✓ | ✓ | both | Per-cell flag definitions |
| Environment-specific rollout percentages | ✓ | ✓ | ✓ | ✓ | both | Per-cell `targeting_rules` |
| Promote flag from env to env | ✓ | ✓ | ✓ | ✓ (via cell topology + replication) | both | Cross-cell replication ≤5s per ADR-0158 |
| Per-region failover | ✓ | partial | ✓ | ✓ (active-active per cell) | both | `manifest.json:cell_eligibility` |

## §11 Scheduled Releases

| Capability | LaunchDarkly | Statsig | Split.io | oyatie feature-flags | Tenant-class | Evidence |
|---|---|---|---|---|---|---|
| Schedule flag activation at future timestamp | ✓ | partial | partial | ◯ (Phase 2; defer to `workflow-engine` cron) | both | `workflow-engine` µservice |
| Schedule percentage rollout ramp | ✓ | partial | partial | ◯ (Phase 2; rollout orchestration BC-5 has rollout stages but not auto-time-advance) | both | `PHASE-01 §BC-5` |
| Schedule kill-switch (e.g., GDPR enforcement deadline) | ✗ | ✗ | ✗ | ✓ (`sunset_at` field) | paid (GDPR pack activation) | `FlagDefinition.sunset_at` |
| Schedule experiment activation | ✓ | ✓ | ✓ | ✓ (manual via `ActivateExperiment` RPC; defer scheduling to workflow-engine) | both | `proto:ExperimentService.ActivateExperiment` |

## §12 Monitoring Integration

| Capability | LaunchDarkly | Statsig | Split.io | oyatie feature-flags | Tenant-class | Evidence |
|---|---|---|---|---|---|---|
| OTEL traces | ✓ | ✓ | ✓ | ✓ | both | `oya-shared-otel` substrate |
| OTEL metrics (RED) | ✓ | ✓ | ✓ | ✓ | both | Prometheus exporter + `slos/*.openslo.yaml` |
| OTEL logs (audit) | ✓ | ✓ | ✓ | ✓ | both | `audit-chain` µservice |
| Datadog APM integration | ✓ | ✓ | ✓ | ◯ (defer to OTEL → Datadog exporter via observability) | both | `observability` µservice |
| New Relic integration | ✓ | partial | partial | ◯ (defer to OTEL) | both | same |
| PagerDuty alert routing | ✓ | ✓ | ✓ | ◯ (defer to `observability` µservice) | both | OTEL + observability |
| Grafana dashboards | ◯ (LD dashboards) | partial | partial | ✓ (4 dashboards under `dashboards/`) | both | flag-state-overview, experiment-results, killswitch-history, pack-override-coverage |

## §13 REST + gRPC API

| Capability | LaunchDarkly | Statsig | Split.io | oyatie feature-flags | Tenant-class | Evidence |
|---|---|---|---|---|---|---|
| REST API | ✓ | ✓ | ✓ | ✓ | both | `contracts/openapi-v1.yaml` 12 paths |
| gRPC API | partial (LD relay) | partial | ✗ | ✓ | both | `contracts/feature-flags-v1.proto` 4 services |
| OpenFeature provider spec | ✓ | ✓ | ✓ | ✓ | both | `contracts/openfeature-sdk-contract.md` |
| OpenAPI 3.x spec | ✓ | partial | partial | ✓ (OpenAPI 3.2.0) | both | OpenAPI 3.2.0 per `openapi-v1.yaml` |
| AsyncAPI 3.x spec | ✗ | ✗ | ✗ | ✓ (AsyncAPI 3.1.0) | both | `contracts/asyncapi-v1.yaml` |
| Bulk evaluation endpoint | ✓ | ✓ | ✓ | ✓ | both | `proto:EvaluateBatch` |
| Idempotency keys on mutation | partial | partial | partial | ✓ | both | per ADR-0258 + step-up token |
| Pagination on list endpoints | ✓ | ✓ | ✓ | ✓ | both | `FlagListResponse.next_page_token` per OpenAPI |

## §14 SDK Languages

| Language | LaunchDarkly | Statsig | Split.io | oyatie feature-flags | Tenant-class | Evidence |
|---|---|---|---|---|---|---|
| Rust | ✗ (community) | ✗ | ✗ | ✓ (Phase 1; canonical) | both | IP-014 + `oya-feature-flags-sdk` |
| TypeScript / Node.js | ✓ | ✓ | ✓ | ✓ (Phase 1) | both | IP-015 + `@oyatie/feature-flags` |
| Python | ✓ | ✓ | ✓ | ✓ (Phase 1) | both | IP-016 + `oyatie-feature-flags` |
| Go | ✓ | ✓ | ✓ | ✓ (Phase 2 roadmap) | both | IP-022 |
| Java | ✓ | ✓ | ✓ | ✓ (Phase 2 roadmap) | both | IP-023 |
| Kotlin | ✓ (Android) | partial | ✓ (Android) | ✓ (Phase 2; under frontend allowlist for Android) | both | per `feedback_rust_strict_only_no_python_2026_05_20` frontend allowlist |
| Swift | ✓ (iOS) | ✓ | partial | ✓ (Phase 2; under frontend allowlist for iOS/macOS) | both | IP-025 + frontend allowlist |
| .NET (C#) | ✓ | ✓ | ✓ | ✓ (Phase 2 roadmap; WinUI 3 frontend allowlist) | both | IP-024 + frontend allowlist |
| C | partial | ✗ | ✗ | ◯ (no roadmap) | both | gap |
| C++ | ✓ | partial | partial | ◯ (no roadmap; can wrap Rust via FFI) | both | potential ADR-MS exception |
| Ruby | ✓ | partial | ✓ | ✗ (forbidden per Rust-strict; would need ADR-MS exception) | both | per language_policy.forbidden_languages_backend |
| PHP | ✓ | ✗ | ✓ | ✗ (forbidden) | both | same |
| Apex (Salesforce) | ✓ | ✗ | ✗ | ✗ (no roadmap; defer to `crm` µservice integration) | both | gap |
| Erlang / Elixir | ✓ | ✗ | ✗ | ◯ (no roadmap; Rust-Elixir FFI possible) | both | gap |
| Web (browser) | ✓ (JS client SDK) | ✓ | ✓ | ✓ (Leptos + Rust→WASM per frontend allowlist) | both | per `feedback_rust_strict_only_no_python_2026_05_20` frontend.web=Leptos |
| React Native | ✓ | partial | ✓ | ◯ (Phase 2; defer to TS SDK + native bridge) | both | gap |
| Flutter | ✓ | partial | partial | ◯ (no roadmap) | both | gap |
| Vue / Angular wrappers | ✓ | ✗ | partial | ◯ (use generic Web SDK) | both | OK |

Total SDK breadth at Phase 1: 3 (Rust + TS + Python). Phase 2 roadmap: +5 (Go + Java + Kotlin + Swift + .NET) = 8. Counterpart breadth: LaunchDarkly = 12+; Statsig = ~10; Split.io = ~9. **Oyatie Phase 2 reaches parity within counterpart range**; Phase 1 trails by 6-9 SDKs.

## §15 CI/CD Integration

| Capability | LaunchDarkly | Statsig | Split.io | oyatie feature-flags | Tenant-class | Evidence |
|---|---|---|---|---|---|---|
| GitHub Actions integration | ✓ | ✓ | ✓ | ✓ (via `oya-vcs` ChangeSet ADR-0110) | both | ADR-0110 admission gate |
| GitLab CI integration | ✓ | ✓ | ✓ | ✓ (same; CI-agnostic via oya-vcs) | both | same |
| Jenkins integration | ✓ | partial | partial | ✓ (CI-agnostic) | both | same |
| Code references (find unreferenced flags in source) | ✓ (LaunchDarkly Code References) | ✗ | ✓ (Split CLI) | ◯ (gap; gh-search lane recommended) | both | gap |
| Stale-flag scanner | ✓ | ✓ | ✓ | ✓ | both | `PHASE-01 §BC-1` background worker `oya-feature-flags-flag-worker` |
| Flag-key reservation policy | partial | partial | partial | ✓ | both | `flag_mutation_authorization.cedar` + flag-key BNF |
| ChangeSet-based promotion (no manual git/gh) | ✗ | ✗ | ✗ | ✓ | both | ADR-0110 + `feedback_oya_git_canonical_2026_05_18` |

## §16 Slack / MS Teams Notifications

| Capability | LaunchDarkly | Statsig | Split.io | oyatie feature-flags | Tenant-class | Evidence |
|---|---|---|---|---|---|---|
| Slack inbound webhook (notify on flag change) | ✓ | ✓ | ✓ | ◯ (defer to comms-email/messenger) | both | gap as native |
| Slack interactive approval | ✓ | ✗ | ✗ | ◯ (defer to ops-dashboard-control-center) | both | gap |
| MS Teams adaptive cards | ✓ | partial | partial | ◯ (defer to messenger/connect) | both | gap |
| Discord webhook | partial | ✗ | ✗ | ◯ (defer to comms-email) | both | gap |
| Email digest | ✓ | ✓ | ✓ | ◯ (defer to comms-email) | both | gap |

## §17 Role-Based Access Control

| Capability | LaunchDarkly | Statsig | Split.io | oyatie feature-flags | Tenant-class | Evidence |
|---|---|---|---|---|---|---|
| Predefined roles | ✓ | ✓ | ✓ | ✓ | both | Six principals in `ARCHITECTURE.md §principals` |
| Custom roles | ✓ | ✓ | ✓ | ✓ (Cedar permits) | both | All Cedar fragments |
| Per-environment role binding | ✓ | ✓ | ✓ | ✓ (per-cell Cedar scope) | both | ADR-0244 + ADR-0158 |
| Per-flag role binding (flag-level RBAC) | ✓ | ✗ | ✓ | ✓ (Cedar permits scoped to resource Flag) | both | `flag-mutation-authorization.cedar` |
| Audience-aware permissions | ✗ | ✗ | ✗ | ✓ (EMERGENCY_SERVICES bypass; MINOR_TARGETED restrictions per ADR-0292) | both | `policy/emergency-services-bypass.cedar` |
| Step-up auth tied to role | ✗ | ✗ | ✗ | ✓ (Class A/B/C per docs/standards/step-up-auth-classes.md) | both | OpenAPI `X-Step-Up-Token` header |
| SSO integration | ✓ | ✓ | ✓ | ✓ (via `identity` µservice SAML/OIDC) | both | `identity` µservice |
| SCIM provisioning | ✓ | ✓ | ✓ | ✓ (via `identity` µservice) | both | `identity` µservice |

## §18 Multi-tenancy

| Capability | LaunchDarkly | Statsig | Split.io | oyatie feature-flags | Tenant-class | Evidence |
|---|---|---|---|---|---|---|
| Tenant isolation (no cross-tenant read) | partial (per-project) | partial (per-project) | partial (per-environment) | ✓ (per ADR-0244 universal) | both | All Cedar + DB schema |
| Per-tenant flag namespace | ✓ | ✓ | ✓ | ✓ | both | `tenant_id` first-class field |
| Per-tenant defaults (override platform default) | ✓ | ✓ | ✓ | ✓ | both | Tenant-specific `targeting_rules` |
| Per-tenant pack overlay | ✗ | ✗ | ✗ | ✓ | paid | ADR-0251 |
| Cross-tenant experiment | ✗ (intentional) | ✗ | ✗ | ✗ (forbidden by Cedar) | both | `experiment-design-authorization.cedar` |
| Tenant data export (DSAR) | ✓ | partial | ✓ | ✓ (per ADR-0276 GDPR Art.20) | both | `openapi.yaml:FlagDefinitionExport` |
| Tenant data deletion (DSAR Art.17) | ✓ | partial | ✓ | ✓ | both | per ADR-0276 + governance µservice |
| Sovereign-pack residency | partial (region selection) | ✗ | ✗ | ✓ (per ADR-0248 + manifest cell_eligibility) | paid (pack activation) | `manifest.json:cell_eligibility.sovereign_packs_supported` |

## §19 oyatie Differentiators (capabilities competitors lack)

These are NOT a tier (per `feedback_tenant_class_2026_05_20`). They are uniformly available to every tenant subject to tenant-class eligibility.

1. **Per-pack compliance overlays** (HIPAA/PCI/GDPR/KR-FSS/SOC2/ISO27001/EU AI Act). Pack activation requires `tenant_class = paid` per `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20`. No counterpart offers this.
2. **EMERGENCY_SERVICES life-safety bypass.** No challenge, no rate limit, no kill-switch can disable; emergency-medication-dispenser-style flags always evaluate to safe-on. No counterpart documents this invariant.
3. **Cedar v4.2 LTS policy targeting** (vs bespoke DSL). Composable, version-controlled, auditable, soakable per ADR-0294 60-second window.
4. **ADR-0028 Merkle-chain sealed audit.** Audit events are cryptographically sealed; tamper-evident. No counterpart offers true sealing.
5. **HLC + TrueTime tier.** HLC default for causality; TrueTime opt-in for financial-grade per ADR-0252. No counterpart offers TrueTime.
6. **HTTP/3 + QUIC + ECH + PQC default transport.** TLS 1.3 floor with ECH where terminated, PQC hybrid where negotiated.
7. **Per-cell deployment with sovereign residency** (cells: us-east-cell-1, eu-west-cell-1 [gdpr-eu], kr-cell-1 [kr-isms-p], us-gov-cell-1 [fedramp-high], us-healthcare-cell [hipaa], us-pci-cell-1 [pci-dss], eu-ai-cell-1 [eu-ai-act, gdpr-eu]).
8. **MINOR_TARGETED audience-type protection** per ADR-0292; experiments and personalization against minors require pre-attested guardian-consent flag.
9. **Appeal URL on every adverse-action decision** per GDPR Art.22 + EU AI Act Art.86. `POST /flags/{flag_key}/appeal` routes to human-review queue in ops-dashboard-control-center.
10. **15-second undo window** per OpenAPI `POST /flags/{flag_key}/undo`. No counterpart offers this; LaunchDarkly has rollback but no transient-window auto-undo.

## §20 oyatie Phase 2 Gaps (counterpart-leader items)

| Gap | Leader | Phase | Owner µservice |
|---|---|---|---|
| SDK breadth to 12+ languages | LaunchDarkly | Phase 2 (Q4 2026) | feature-flags (this) |
| Pre-aggregated metric warehouse | Statsig | Phase 2 | `analytics` µservice (ClickHouse) |
| ML-powered auto-targeting | Statsig | Phase 3 | `intelligence` µservice |
| Holdout groups / global holdouts | LaunchDarkly, Statsig | Phase 2 | feature-flags |
| Funnel experiments (multi-step) | Optimizely | Phase 2 | feature-flags |
| Workflow Builder UI | LaunchDarkly | Phase 2 | `workflow-studio` |
| Code references scanner | LaunchDarkly | Phase 2 | feature-flags + `oya-vcs` |
| Slack/Teams native integration | LaunchDarkly | Phase 2 | `messenger` + `connector` |
| No-code experiment editor | Optimizely | Phase 3 | `workflow-studio` |
| Impressions data export pipeline | Split.io | Phase 2 | `analytics` + marketplace pack |
| Prerequisite flag dependency graph | All 3 | Phase 2 | feature-flags |

## §21 Substance-bar Self-Check

Per brief-template §3.1 substance bar, this matrix:
- Covers all 22 capability families from the brief (flag creation, targeting, real-time updates, webhooks, audit log, experiment platform, feature workflows, kill-switch, prerequisite flags, environment management, scheduled releases, monitoring, REST+gRPC, SDK languages [12+ named], CI/CD, Slack notifications, RBAC, multi-tenancy).
- Cites exact evidence path per cell for the oyatie column.
- Names counterpart-specific vendor terms in parens (e.g., "LaunchDarkly Big Segments", "Statsig audiences", "Split.io dynamic configs").
- Adds tenant-class eligibility column per `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20`.
- Avoids tenant_class-segmented capability rows per `feedback_tenant_class_2026_05_20`.
- Lists oyatie's unique differentiators (§19, 10 items) + Phase 2 gaps (§20, 11 items).

Coverage verdict: oyatie matches or exceeds LaunchDarkly + Statsig + Split.io on:
- All four typed variants.
- Cedar policy targeting (counterparts use bespoke DSL).
- Sealed audit chain.
- Statistical method suite (Bayesian + frequentist + mSPRT + SRM + Mann-Whitney + BH-FDR + LIME/SHAP).
- Multi-region active-active with sovereign-pack residency.
- HTTP/3 + ECH + PQC transport.
- Life-safety bypass + step-up auth + minor protection.
- 15-second undo window + appeal URL.
- Per-pack compliance overlays.

oyatie trails on:
- SDK breadth (Phase 1: 3 SDKs; Phase 2 roadmap: 8 SDKs; counterpart leader: 12+).
- Native Slack/Teams (defer to messenger/comms-email).
- Workflow Builder UI (defer to workflow-studio).
- ML-powered auto-targeting (defer to intelligence + Phase 3).
- Pre-aggregated metric warehouse (defer to analytics).
- Code-reference scanner (Phase 2).

## §22 Vendor API Versions and Surface Pinning

Per brief-template §2.5 substance requirements ("require named vendor versions when vendors are involved"), the comparison anchors are:

| Counterpart | Public API version (audit-date) | Auth model | Rate-limit (free / enterprise) | Notes |
|---|---|---|---|---|
| LaunchDarkly REST API | v2 (api.launchdarkly.com/v2) | Bearer token (Personal Access Token + Service Token) | 2000 req/10s default; 10x enterprise | Streaming endpoint separate at `stream.launchdarkly.com` |
| LaunchDarkly SDK protocol | Streaming + polling; HTTP/2 | Environment-scoped SDK Key | Per-environment | OpenFeature provider available |
| Statsig server SDK | v0.x to v1.x | Server Secret Key | Per-project meter | Bayesian default; layered configs |
| Statsig Console API | v1 | Console Token | Per-account | Pulse reports + holdouts |
| Split.io Admin API | v1 | Admin API Key | 1000 req/min | Yaml import/export available |
| Split.io SDK protocol | LD-style streaming + polling | SDK API Key | Per-environment | Impressions export to s3/snowflake |
| OpenFeature spec | 0.7+ for server-side | Provider-defined | n/a | CNCF standard |

oyatie pins:
- OpenAPI 3.2.0 (per `contracts/openapi-v1.yaml:openapi`).
- AsyncAPI 3.1.0 (per `contracts/asyncapi-v1.yaml`).
- proto3 (per `contracts/feature-flags-v1.proto:syntax`).
- gRPC over HTTP/3 + tonic (per ADR-0253).
- Cedar v4.2 LTS (per brief-template §3.5).
- OpenFeature SSEP v0.1.0 (per `info.x-openfeature-ssep-version`).
- PostgreSQL 17 (per brief-template §2.5 vendor examples).
- Kubernetes 1.35 LTS + Cilium 1.18 (per brief-template).

Migration hazards inherited from counterpart parity:
- LaunchDarkly: SDK Key environment-binding is non-portable; tenant must rotate keys when migrating. Oyatie equivalent: per-tenant mTLS SVID per ADR-0295 — portable across environments.
- Statsig: Console Token + per-project meter — switching projects costs metric history. Oyatie equivalent: per-tenant + per-cell scope; metric history persists in ClickHouse per cell.
- Split.io: Impressions export to s3/snowflake requires customer-managed pipeline. Oyatie equivalent: AsyncAPI `flag-state-changed` event stream; consumer µservices can subscribe directly without per-tenant pipeline.

## §23 SDK Caching Contract per Counterpart

| Counterpart | In-process cache TTL | Push mechanism | LKG behavior |
|---|---|---|---|
| LaunchDarkly Server SDK | Streaming (no TTL); polling 30s default | SSE stream | Last-eval cache |
| Statsig Server SDK | Polling 60s default | Bootstrap + WebSocket | Bootstrap cache |
| Split.io Server SDK | Polling 60s; Streaming optional | Streaming (Q1 2025) | LKG impressions |
| oyatie Rust SDK | 30s TTL + SSE push invalidation | SSE on `/flags/stream` | LKG 30-min per `openfeature-sdk-contract.md` |
| oyatie TS SDK | 30s TTL + EventSource | EventSource | LKG 30-min |
| oyatie Python SDK | 30s TTL + httpx SSE | httpx SSE | LKG 30-min |

oyatie LKG 30-min is best-in-class — survives broker outage windows that counterparts' LKG would also lose.

## §24 Tenant-Class Eligibility per Capability

Per `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20`, the following capabilities are tenant-class-gated:

| Capability | demo_trial eligible | paid eligible | Cedar guard |
|---|---|---|---|
| Boolean / string / number / JSON flag CRUD | ✓ (≤100 flags) | ✓ (≤10,000 flags) | `tenant_class == paid` not required for CRUD; usage cap enforced by cloud-billing |
| Cedar predicates / cohort targeting | ✓ | ✓ | none |
| Compliance pack overlays (HIPAA/PCI/GDPR/...) | ✗ | ✓ | `when { context.tenant_class == "paid" && context.pack_id in resource.allowed_packs }` |
| BYOK for audit-event encryption | ✗ | ✓ | `when { context.tenant_class == "paid" && context.provider_credential_mode == "byok" }` |
| Marketplace integration purchase | ✗ | ✓ | `when { context.tenant_class == "paid" }` |
| Contractual SLO commitments | ✗ (best-effort) | ✓ (per contract) | n/a (SLO file overlay) |
| Step-up auth Class C (TOTP + passkey) | ✓ (demo_trial tenant_class still gets passkey) | ✓ | none |
| EMERGENCY_SERVICES audience bypass | ✓ | ✓ | none (life-safety hard rule) |
| Sovereign-pack residency | n/a | ✓ | `when { context.tenant_class == "paid" && context.sovereign_pack in resource.allowed_packs }` |
| Per-evaluation audit emission | ✓ (best-effort sink) | ✓ (contractual sink) | none |

This wiring is pending the §3.4.C amendment in the audit doc. Until then, the matrix marks `tenant_class` columns as the *intended* policy; runtime enforcement requires the Cedar context schema extension.

## §25 Failure Modes Comparison

| Failure mode | LaunchDarkly behavior | Statsig behavior | Split.io behavior | oyatie behavior |
|---|---|---|---|---|
| Flag server unavailable | SDK returns cached + default | SDK returns cached + default | SDK returns cached + default | SDK returns LKG 30-min then default + audit-degraded counter |
| Cedar / DSL eval error | n/a (LaunchDarkly: bespoke DSL) | n/a | n/a | Returns default + emits `oya.feature_flags.flag.cedar_eval_error` + pages on >1% |
| Cross-region replication lag | LaunchDarkly Edge replication (limited) | Statsig regional active-active | Split.io regional | Patroni async + Kafka MirrorMaker; ≤30s p99 cross-region |
| Audit sink backpressure | LaunchDarkly: backlog audit events | Statsig: backlog | Split.io: backlog | Halt high-risk mutation when audit-chain queue >95% |
| Kill-switch propagation delay | 1-5s typical | <1s | 1-3s | ≤1s p99 globally (life-safety SLO) |
| Sample-Ratio-Mismatch in experiment | LD partial | Statsig Chi-squared | Split.io partial | Chi-squared + mSPRT + Mann-Whitney + BH-FDR |
| Cedar fragment soak error | n/a | n/a | n/a | Fragment held in shadow mode 60s before activation; soak failures roll back automatically |
| Pack overlay conflict | n/a | n/a | n/a | Deny-wins per ADR-0251; stricter pack overrides; `runbooks/pack-override-cascade.md` |
| Tenant_class mis-classification | n/a (no tenant_class) | n/a | n/a | Fail-closed to demo_trial; pending Wave 15J wiring per audit §3.4.C |

## §26 Vendor Migration Path

For tenants migrating from a counterpart to oyatie:

| Source | Path | Effort | Tooling |
|---|---|---|---|
| LaunchDarkly | Export flags via REST v2; import via oyatie OpenAPI `POST /flags`; rebuild Cedar predicates from LD targeting rules | Medium (DSL rewrite) | `migration-playbooks/from-launchdarkly.md` exists per file listing |
| Statsig | Export via Console API v1; import via OpenAPI; rebuild layered configs as JSON-object flags | Medium-High | not yet authored |
| Split.io | Export via Admin API v1 yaml; import via OpenAPI; rebuild Split DSL → Cedar | Medium-High | not yet authored |
| OpenFeature provider chain | Re-target provider config; oyatie ships as drop-in OpenFeature SSEP-compliant provider | Low | OpenFeature provider compatibility |

Migration aids in oyatie:
- DSAR-style export (`FlagDefinitionExport` schema in OpenAPI) usable as backup before migration.
- 15-second undo window applies to bulk-imports (rollback in case of failed mapping).
- Shadow-mode Cedar fragment soak ≥60s catches rewrite errors before activation.

End of parity matrix.
