---
id: ADR-MS-001
title: Flag evaluation, safety killswitch, and experiment governance contract for feature-flags
status: Proposed
date: 2026-05-20
microservice: feature-flags
related_oyatie_adrs:
  - ADR-0003-audit-chain-and-evidence-emission
  - ADR-0007-cedar-authorization-policy-and-persona-tier
  - ADR-0008-data-use-boundary
  - ADR-0009-cell-architecture-per-tenant-per-region
  - ADR-0037-public-api-stability-tiers-and-deprecation
  - ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback
  - ADR-0128-hyperscaler-architecture-invariants
  - ADR-0131-per-microservice-flat-layout
decision_owner: axis-feature-flags + release-engineering
---

# ADR-MS-001: Flag evaluation, safety killswitch, and experiment governance contract for feature-flags

## Context

- Pressure name: release-control integrity pressure.
- `feature-flags` owns flag evaluation, flag mutation, rollout, targeting, experiment design, safety killswitch, and pack override behavior.
- Progressive delivery depends on fast flag reads and strongly governed writes.
- A weak flag system can bypass rollout gates, leak experiments across tenants, or delay emergency shutdown.
- The feature-flags OpenAPI contract exposes `POST /flags/{flag_key}/evaluate`.
- The feature-flags AsyncAPI contract emits `FlagChanged` and `FlagEvaluated`.
- Local contracts also include `openapi-v1.yaml`, `asyncapi-v1.yaml`, `feature_flags.proto`, and `openfeature-sdk-contract.md`.
- Local policies include `flag-mutation-authorization.cedar`, `safety-killswitch-authorization.cedar`, `experiment-design-authorization.cedar`, `tenant-targeting.cedar`, `pack-flag-override.cedar`, `pack-overlay-authorization.cedar`, and `schema.cedarschema`.
- Local policies also include `abuse-defence.cedar`, `auditor-scope.cedar`, `ci-scope.cedar`, `data-residency.md`, and `emergency-services-bypass.cedar`.
- Local SLOs include `feature-flags.openslo.yaml`, `flag-eval-latency.openslo.yaml`, `flag-state-propagation.openslo.yaml`, `killswitch-fire-latency.openslo.yaml`, and `experiment-result-freshness.openslo.yaml`.
- Local dashboards include `flag-state-overview.json`, `killswitch-history.json`, `experiment-results.json`, and `pack-override-coverage.md`.
- Constraint name: read-fast write-governed.
- Evaluation must be low latency and highly available.
- Mutation must be policy-gated, audited, staged, and reversible.
- Constraint name: tenant targeting pressure.
- A flag can target by tenant, pack, cell tier, cohort, SDK, capability tier, or experiment assignment.
- Any target rule that ignores tenant or pack can leak functionality across boundaries.
- Constraint name: emergency shutdown pressure.
- Killswitches must propagate faster than ordinary rollout changes.
- Killswitch authority must be narrow and evidence-rich.
- Constraint name: experiment ethics pressure.
- Experiments alter user experience and may affect pricing, safety, consent, or regulated workflows.
- Experiment design must carry purpose, metric, holdout, stop rule, and guardrail thresholds.
- Constraint name: SDK conformance pressure.
- OpenFeature-compatible clients must get consistent evaluation semantics across Rust, TypeScript, Python, Java, Go, .NET, and Swift.

## Decision

- Decision name: deterministic policy-bound flag control.
- `feature-flags` will use deterministic server-side evaluation as the source of truth for governed flags.
- SDKs may cache evaluation results only under signed state version, TTL, tenant, pack, and flag key.
- `POST /flags/{flag_key}/evaluate` is the canonical public evaluation endpoint.
- Evaluations require tenant id, principal or anonymous subject id, flag key, evaluation context, data class, pack overlay, SDK name, SDK version, and trace context.
- Evaluation responses include variation, reason, flag version, rule id, state version, cache TTL, evidence mode, and trace correlation id.
- Evaluation events emit `FlagEvaluated` with bounded labels and no raw targeting attributes.
- Mutations use internal action contracts and emit `FlagChanged` after Cedar approval.
- Flag mutation requires owner team, rollout plan, rollback condition, blast-radius estimate, affected packs, and metric guardrails.
- Tenant targeting rules require explicit tenant, pack, and cell predicates.
- Pack flag overrides require pack owner approval and cannot weaken data residency or safety policy.
- Experiment definitions require hypothesis, primary metric, guardrail metrics, holdout percent, minimum detectable effect, stop rule, and review owner.
- Experiments that touch protected data classes require privacy review evidence before activation.
- Killswitch definitions require owning service, safety condition, max propagation latency, authorized operator roles, and reset procedure.
- Killswitch activation bypasses ordinary rollout cadence but does not bypass tenant, pack, audit, or ownership evidence.
- Killswitch propagation must satisfy service-local latency target.
- Flag state propagation must satisfy service-local propagation target.
- Flag evaluation latency must satisfy service-local evaluation latency target.
- Experiment result freshness must satisfy service-local freshness target.
- Flag reads must remain available during ordinary mutation pipeline outage through signed state snapshots.
- Flag writes must fail closed when Cedar, audit-chain, or state-version signing is unavailable.
- Every flag has one of four classes: `release`, `permission`, `experiment`, or `safety`.
- `safety` flags always outrank `release`, `permission`, and `experiment` flags.
- `pack` overrides outrank ordinary rollout rules but not safety killswitches.
- `tenant` explicit deny outranks broad cohort allow.
- Rule evaluation order is safety, tenant deny, pack override, permission, experiment, release default.
- Metrics may include flag key hash, flag class, pack, SDK, result reason, and state version.
- Metrics must not include raw targeting attributes or personal identifiers.

## Alternatives Considered

### Alternative 1: Use an external flag vendor as authority

- Pros: mature UI, SDKs, experimentation, and rollout controls.
- Pros: lower initial development effort.
- Cons: tenant and pack policy evidence becomes vendor-specific.
- Cons: killswitch authority may not line up with Oyatie audit-chain semantics.
- Cons: on-prem and sovereign cells need provider-neutral behavior.
- Rejected because vendor systems can inform parity but cannot own governed flag state.

### Alternative 2: Client-only flag evaluation

- Pros: lowest server read load.
- Pros: strong offline behavior for SDKs.
- Cons: clients can evaluate stale or tampered state.
- Cons: tenant and pack policy decisions become hard to audit.
- Cons: emergency killswitch propagation depends on every client refresh.
- Rejected because governed flags need server-side authoritative evaluation.

### Alternative 3: One mutable JSON config file per environment

- Pros: simple to inspect.
- Pros: easy for early prototypes.
- Cons: no policy-gated mutation.
- Cons: no per-tenant or pack-aware audit evidence.
- Cons: no SDK conformance or event semantics.
- Rejected because release control cannot be an ad hoc config file.

### Alternative 4: Treat experiments as ordinary release flags

- Pros: simpler flag taxonomy.
- Pros: one rollout path for all changes.
- Cons: experiments require holdouts, metrics, guardrails, and stop rules.
- Cons: ordinary rollout approval does not answer ethics or data-class concerns.
- Cons: experiment results need freshness and attribution evidence.
- Rejected because experiments are governed decisions, not just percentage rollouts.

### Alternative 5: Let emergency operators mutate any flag directly

- Pros: fast intervention.
- Pros: fewer escalation steps during incidents.
- Cons: broad mutation power can create unsafe state.
- Cons: ordinary release flags could be changed without review.
- Cons: post-incident evidence would be incomplete.
- Rejected because killswitch authority must be narrow and typed.

## Consequences

### Positive

- Flag evaluation is deterministic and SDK-conformant.
- Mutation, pack override, experiment design, and killswitch actions are separately governed.
- Emergency shutdown is faster than ordinary rollout while still audited.
- Tenants and packs cannot accidentally receive ineligible flag states.
- OpenFeature clients can use one contract across languages.
- Experiment metrics, holdouts, and guardrails become evidence-bound.
- Progressive delivery can consume flag state with rollback conditions.
- Dashboards can show flag state, killswitch history, experiments, and pack override coverage.

### Negative

- Server-side evaluation requires low-latency infrastructure.
- SDK cache invalidation is more complex than simple static config.
- Experiment design policy requires product, privacy, and data review upkeep.
- Pack overrides can create rule precedence complexity.
- Killswitch governance requires clear operator roles and drills.
- Mutation workflows need signing and state-version management.
- Flag key hash metrics reduce direct human readability.

### Neutral

- SDKs may evaluate locally when using signed snapshots within TTL.
- OpenFeature remains the SDK interoperability target.
- External flag vendors may be migration comparators or temporary adapters.
- Release orchestration remains governed by progressive delivery ADRs.
- Feature flag state is not a substitute for Cedar authorization in domain services.

### Follow-up work

- Add rule-precedence fixture suite for safety, tenant deny, pack override, permission, experiment, and release.
- Add OpenFeature provider conformance tests for all supported SDKs.
- Add killswitch drill runbook for each safety-class flag.
- Add experiment review checklist for protected data classes.
- Add state snapshot signing and verification tests.
- Add pack override coverage dashboard thresholds.

## Implementation Notes

### Data Shapes

- `FlagDefinition` fields: `flag_key`, `flag_class`, `owner_team`, `default_variation`, `state_version`, `rules`, `created_at`, `archived_at`.
- `EvaluationContext` fields: `tenant_id`, `subject_id_hash`, `pack`, `cell_tier`, `sdk_name`, `sdk_version`, `attributes_ref`, `purpose`, `data_class`.
- `FlagEvaluationRequest` fields: `flag_key`, `tenant_id`, `principal_id`, `evaluation_context`, `traceparent`, `stale_ok`, `required_state_version`.
- `FlagEvaluationResponse` fields: `variation`, `reason`, `flag_version`, `rule_id`, `state_version`, `cache_ttl_ms`, `evidence_mode`, `trace_correlation_id`.
- `FlagMutation` fields: `flag_key`, `mutation_type`, `owner_team`, `rollout_plan_id`, `rollback_condition`, `blast_radius`, `affected_packs`, `metric_guardrails`.
- `TargetRule` fields: `rule_id`, `predicate`, `variation`, `priority`, `tenant_predicate`, `pack_predicate`, `cell_predicate`, `expires_at`.
- `ExperimentDefinition` fields: `experiment_id`, `flag_key`, `hypothesis`, `primary_metric`, `guardrail_metrics`, `holdout_percent`, `minimum_detectable_effect`, `stop_rule`, `review_owner`.
- `KillswitchDefinition` fields: `killswitch_id`, `flag_key`, `owning_service`, `safety_condition`, `max_propagation_latency_ms`, `authorized_roles`, `reset_procedure`.
- `FlagChanged` event fields: `flag_key_hash`, `flag_class`, `state_version`, `mutation_type`, `policy_version`, `operator_role`, `evidence_id`.
- `FlagEvaluated` event fields: `flag_key_hash`, `flag_class`, `variation`, `reason`, `state_version`, `sdk_name`, `pack`, `evidence_mode`.

### API Endpoints

- `POST /flags/{flag_key}/evaluate` evaluates a single governed flag.
- `POST /flags/{flag_key}/evaluate` returns deterministic reason codes for rule, default, stale snapshot, deny, or killswitch.
- Internal action `flag.mutate` creates, amends, archives, or rolls back flag definitions.
- Internal action `flag.rollout` changes rollout percentage or cohort assignment.
- Internal action `experiment.design` creates or amends experiment definitions.
- Internal action `safety-killswitch.fire` activates a safety-class override.
- Internal action `pack-flag-override.apply` applies a pack-specific override.
- Internal action `state-snapshot.publish` publishes signed state snapshots for SDK caches.

### Cedar Policies

- `policy/flag-mutation-authorization.cedar` authorizes create, amend, archive, and rollback.
- `policy/safety-killswitch-authorization.cedar` authorizes emergency activation and reset.
- `policy/experiment-design-authorization.cedar` authorizes experiment creation and changes.
- `policy/tenant-targeting.cedar` enforces tenant-scoped targeting predicates.
- `policy/pack-flag-override.cedar` authorizes pack overrides.
- `policy/pack-overlay-authorization.cedar` prevents pack overrides from weakening residency or safety.
- `policy/schema.cedarschema` declares policy entities and actions.
- `policy/auditor-scope.cedar` allows evidence review without exposing raw targeting attributes.
- `policy/ci-scope.cedar` allows SDK conformance and contract tests.
- Policy must deny mutation when owner team or rollback condition is missing.
- Policy must deny experiment activation without guardrail metrics.
- Policy must deny killswitch activation by roles outside authorized operator set.

### SLO Targets

- `feature-flags.openslo.yaml`: overall service objective for flag control.
- `flag-eval-latency.openslo.yaml`: evaluation latency objective for read path.
- `flag-state-propagation.openslo.yaml`: state propagation objective for normal changes.
- `killswitch-fire-latency.openslo.yaml`: emergency propagation objective for safety flags.
- `experiment-result-freshness.openslo.yaml`: experiment result freshness objective.
- Safety-class flags must use the shortest propagation SLO.
- SDK snapshot TTL must be lower than the relevant propagation objective for governed flags.

## Verification

- Unit test `evaluation_request_requires_tenant_pack_sdk_and_data_class`.
- Unit test `evaluation_response_includes_reason_state_version_and_ttl`.
- Unit test `rule_order_safety_before_tenant_deny_before_pack_override`.
- Unit test `experiment_definition_requires_holdout_guardrails_and_stop_rule`.
- Unit test `killswitch_definition_requires_authorized_roles_and_reset`.
- Unit test `sdk_cache_key_includes_tenant_pack_flag_and_state_version`.
- Property test `deterministic_evaluation_same_context_same_state`.
- Property test `tenant_deny_always_overrides_broad_cohort_allow`.
- Property test `safety_killswitch_always_overrides_experiment`.
- Cedar test `flag_mutation_denies_missing_rollback_condition`.
- Cedar test `experiment_design_denies_missing_guardrail_metrics`.
- Cedar test `tenant_targeting_denies_missing_tenant_predicate`.
- Cedar test `pack_override_denies_residency_weakening`.
- Cedar test `killswitch_denies_unauthorized_operator_role`.
- Contract test `feature-flags.openapi.yaml_contains_evaluate_path`.
- Contract test `feature-flags.asyncapi.yaml_contains_changed_and_evaluated_events`.
- Contract test `openfeature-sdk-contract.md_matches_provider_behavior`.
- Integration test `evaluate_flag_emits_flag_evaluated_without_raw_attributes`.
- Integration test `mutate_flag_emits_flag_changed_after_policy`.
- Integration test `killswitch_fire_propagates_before_normal_rollout`.
- Integration test `pack_override_applies_only_to_eligible_pack`.
- Integration test `experiment_activation_requires_privacy_review_for_protected_data`.
- Integration test `signed_snapshot_rejected_when_state_version_mismatch`.
- Load test `flag_eval_latency_meets_slo_under_sdk_mix`.
- Load test `flag_state_propagation_meets_slo`.
- Load test `killswitch_fire_latency_meets_slo`.
- Load test `experiment_result_freshness_meets_slo`.
- Chaos test `audit_chain_unavailable_blocks_mutation`.
- Chaos test `cedar_unavailable_blocks_writes_but_allows_valid_signed_read_snapshot`.
- Chaos test `state_snapshot_signing_failure_blocks_publish`.
- Metric `oya_feature_flags_eval_latency_good_total`.
- Metric `oya_feature_flags_state_propagation_good_total`.
- Metric `oya_feature_flags_killswitch_fire_latency_good_total`.
- Metric `oya_feature_flags_experiment_result_freshness_good_total`.
- Metric `oya_feature_flags_mutation_denied_total`.
- Dashboard `dashboards/flag-state-overview.json`.
- Dashboard `dashboards/killswitch-history.json`.
- Dashboard `dashboards/experiment-results.json`.
- Dashboard `dashboards/pack-override-coverage.md`.
- Runbook check `runbooks/killswitch-fire-drill.md` covers activation and reset.
- Runbook check `runbooks/experiment-guardrail-breach.md` covers stop rule.
- Promotion gate blocks if safety-class flag lacks killswitch drill evidence.
- Promotion gate blocks if OpenFeature conformance fails for any supported SDK.

## References

- Oyatie ADR-0003: Audit chain and evidence emission.
- Oyatie ADR-0007: Cedar authorization policy and persona tier.
- Oyatie ADR-0008: Data use boundary.
- Oyatie ADR-0009: Cell architecture per tenant per region.
- Oyatie ADR-0037: Public API stability tiers and deprecation.
- Oyatie ADR-0040: Progressive delivery canary blue-green metric-gated rollback.
- Oyatie ADR-0128: Hyperscaler architecture invariants.
- Oyatie ADR-0131: Per-microservice flat layout.
- OpenFeature specification and provider API documentation.
- LaunchDarkly documentation: flag evaluation, targeting, experiments, and kill switches.
- Unleash documentation: strategies, variants, constraints, and gradual rollout.
- RFC 9110: HTTP Semantics.
- W3C Trace Context Recommendation.
- Kohavi, Tang, and Xu: Trustworthy Online Controlled Experiments.
- Deng, Xu, Kohavi, and Walker: Improving the Sensitivity of Online Controlled Experiments by Utilizing Pre-Experiment Data.
- Google SRE Workbook: SLOs and alerting.
- Cedar policy language documentation.
- NIST SP 800-63B: Operator authentication lifecycle.
- CNCF TAG App Delivery: Progressive delivery concepts.
