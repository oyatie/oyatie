---
id: ADR-CIAC-001
title: Terraform Module Versioning with Multi Tenant Input Substitution
status: Proposed
date: 2026-05-20
microservice: cloud-iac
related_oyatie_adrs:
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0041-gitops-trunk-based-and-release-branch-cut-at-tag.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
decision_owner: axis-cloud-iac
---

# ADR-CIAC-001: Terraform Module Versioning with Multi Tenant Input Substitution

## Context

- Cloud-iac owns deterministic render, validate, apply, rollback, registry, drift detection, and GitOps reconciliation for IaC.
- The PRD names Helm, Terraform/OpenTofu, Kustomize, ArgoCD, Flux, module registry, drift detection, and apply ledger responsibilities.
- The repo contains OpenTofu modules under `microservices/cloud-iac/tofu/modules/`.
- The user title names Terraform module versioning; Oyatie implementation posture uses OpenTofu-compatible module semantics.
- Named pressure CIAC-P1: one module version can be consumed by many tenants and packs with different inputs.
- Named pressure CIAC-P2: tenant input substitution must not create hidden module forks.
- Named pressure CIAC-P3: module upgrades need SemVer, pinning, drift preview, and rollback.
- Named pressure CIAC-P4: secrets must remain references, not substituted raw values.
- Named pressure CIAC-P5: plan-preview must be deterministic for a given module version and tenant input set.
- Named pressure CIAC-P6: pack overlays must change inputs without changing module source.
- Named pressure CIAC-P7: provider and module version constraints must prevent accidental newest-version drift.
- Named pressure CIAC-P8: per-tenant state must remain pack-pinned.
- Named pressure CIAC-P9: generated manifests need audit-chain and provenance evidence.
- Named pressure CIAC-P10: module authors need a stable contract for required, optional, sensitive, and computed variables.
- Constraint CIAC-C1: GitOps release branch and tag policy follows ADR-0041.
- Constraint CIAC-C2: secrets and HSM per cell follow ADR-0043.
- Constraint CIAC-C3: per-microservice flat layout follows ADR-0131.
- Constraint CIAC-C4: multi-cluster federation follows ADR-0171.
- Constraint CIAC-C5: vendor lock-in avoidance follows ADR-0173.
- Constraint CIAC-C6: GitOps IaC lifecycle tiers follow ADR-0202.
- Constraint CIAC-C7: tenant scope and Cedar gates follow ADR-0243 and ADR-0244.
- Constraint CIAC-C8: observability follows ADR-0263.
- Constraint CIAC-C9: audit-chain evidence follows ADR-0003.
- Constraint CIAC-C10: cloud-iac does not own cloud-secrets or cloud-k8s runtime operations.
- Existing `tofu/modules/kms`, `vpc`, `dns`, `cloud-account`, `secrets-bootstrap`, and `k8s-namespace-bootstrap` modules need version discipline.
- Existing cloud-iac contracts include OpenAPI, AsyncAPI, and proto surfaces.
- The decision must define module versioning and tenant input substitution together.

## Decision

- Adopt `IacModuleVersionMatrix v1`.
- Use OpenTofu module semantics as the implementation baseline while keeping Terraform-compatible module language where feasible.
- Version every reusable IaC module with SemVer.
- Pin every tenant module use to an exact module version or approved constraint set.
- Store module source separately from tenant input sets.
- Store tenant input sets as typed substitution overlays.
- Store pack input overlays separately from tenant input overlays.
- Resolve inputs in this order: module defaults, pack overlay, tenant overlay, environment overlay, emergency override.
- Require every substituted value to declare data class, source, sensitivity, and validation rule.
- Require secret values to be represented as `SecretReference`, never raw secret bytes.
- Require provider versions to be explicitly constrained.
- Require module versions to be selected through cloud-iac registry, not ad hoc git references.
- Require plan-preview before any module version promotion.
- Require drift detection before and after module version promotion.
- Require rollback target to be previous known-good module version and input digest.
- Bind every plan to `{module_version, input_digest, provider_lock_digest, tenant_id, pack_code}`.
- Bind every apply to a signed provenance event and audit-chain seal.
- Use immutable module release artifacts addressed by digest.
- Use generated lock files for provider and module dependencies.
- Use per-tenant state keys with pack-pinned storage.
- Use Cedar to gate module registration, input substitution, plan, apply, rollback, and drift remediation.
- Use validation policies to reject tenant inputs outside declared variable contracts.
- Use schema migrations for module variable changes that remove or rename variables.
- Permit additive optional variables in minor releases.
- Require major version for variable removals, type changes, resource identity changes, or destructive defaults.
- Require patch version for bug fixes that do not change plan output for unchanged inputs.
- Require release notes to state state migration expectations.
- Name event `cloud_iac.module.version.registered.v1`.
- Name event `cloud_iac.input_set.substituted.v1`.
- Name event `cloud_iac.plan_preview.completed.v1`.
- Name event `cloud_iac.apply.executed.v1`.
- Make this ADR authoritative for Terraform/OpenTofu module versioning and tenant input substitution.

## Alternatives Considered

### One Module Fork per Tenant

- Pros: each tenant can customize freely.
- Pros: low abstraction pressure.
- Pros: easy emergency patches.
- Cons: destroys reuse and auditability.
- Cons: upgrades become unbounded.
- Cons: drift detection cannot reason over one canonical source.
- Rejected because tenant differences belong in typed input overlays.

### Floating Module Version Constraints

- Pros: tenants can receive fixes automatically.
- Pros: less manual version bump work.
- Pros: common in small Terraform estates.
- Cons: plan output can change without explicit review.
- Cons: rollback target becomes ambiguous.
- Cons: provider and module drift can break deterministic previews.
- Rejected for production; exact versions or approved narrow constraints are required.

### Raw Variable Files per Tenant

- Pros: simple and familiar.
- Pros: works with Terraform and OpenTofu CLIs.
- Pros: easy for operators to edit.
- Cons: weak type and data-class enforcement.
- Cons: secrets can leak into repo or logs.
- Cons: provenance of each value is hard to prove.
- Rejected in favor of typed substitution overlays.

### Fully Generated Modules

- Pros: maximum flexibility from higher-level specs.
- Pros: can hide IaC language details from teams.
- Pros: central generator can enforce standards.
- Cons: generated source diffs are harder to review.
- Cons: module authors lose control of provider-specific behavior.
- Cons: generator bugs affect every tenant at once.
- Rejected for v1; generated inputs and manifests are acceptable, modules stay authored.

### Terraform Cloud or Similar Remote Backend as Authority

- Pros: mature module registry, state, and run workflows.
- Pros: policy checks and UI are available.
- Pros: less internal control-plane work.
- Cons: conflicts with vendor displacement and pack-pinned state.
- Cons: external service becomes apply authority.
- Cons: raw tenant inputs and state residency become harder.
- Rejected for core cloud-iac operation.

## Consequences

- Positive: module source remains canonical and reusable.
- Positive: tenant differences are visible as typed overlays.
- Positive: plan-preview is deterministic by version and input digest.
- Positive: secrets stay as references.
- Positive: SemVer communicates upgrade risk.
- Positive: rollback can target previous module version and input digest.
- Positive: drift detection can compare declared version to applied state.
- Negative: module authors must maintain variable schemas and release notes.
- Negative: input substitution resolver becomes load-bearing.
- Negative: exact version pinning requires explicit rollout workflow.
- Negative: emergency overrides need strong expiry and audit.
- Neutral: Terraform-compatible modules can run through OpenTofu.
- Neutral: pack overlays can evolve independently of tenant overlays.
- Neutral: provider lock files become first-class evidence.
- Neutral: module registry is a cloud-iac bounded context, not a global repo convention.
- Follow-up work CIAC-F1: add module variable schema registry.
- Follow-up work CIAC-F2: add input digest to plan-preview contract.
- Follow-up work CIAC-F3: add migration rule examples for major versions.
- Follow-up work CIAC-F4: add module version dashboard.
- Follow-up work CIAC-F5: add secret reference linter for input overlays.

## Implementation Notes

- Data shape `IacModule`: `{module_id, name, source_path, owner_service, current_version, registry_digest}`.
- Data shape `IacModuleVersion`: `{module_id, version, semver_class, source_digest, variable_schema_digest, provider_constraints, release_notes_ref}`.
- Data shape `IacVariableSchema`: `{module_id, version, variables, sensitive_variables, defaults, validation_rules, removed_variables}`.
- Data shape `TenantInputSet`: `{tenant_id, input_set_id, module_id, module_version, pack_code, environment, values_digest}`.
- Data shape `InputValue`: `{name, value_ref, data_class, sensitivity, source_layer, validation_status}`.
- Data shape `SecretReference`: `{provider, path, key, version, cell, rotation_policy}`.
- Data shape `PlanPreview`: `{tenant_id, module_id, module_version, input_digest, provider_lock_digest, diff_summary, risk_class}`.
- Data shape `ApplyExecution`: `{tenant_id, apply_id, plan_id, module_version, input_digest, state_key, audit_event_id}`.
- Data shape `ModuleRollbackTarget`: `{tenant_id, module_id, from_version, to_version, previous_input_digest, previous_state_ref}`.
- Postgres table `cloud_iac_module` stores module identity.
- Postgres table `cloud_iac_module_version` stores immutable versions.
- Postgres table `cloud_iac_variable_schema` stores variable contracts.
- Postgres table `cloud_iac_tenant_input_set` stores typed overlays.
- Postgres table `cloud_iac_plan_preview` stores deterministic plan metadata.
- Postgres table `cloud_iac_apply_execution` stores apply evidence.
- Object path `iac/modules/{module_id}/{version}/source.tar.zst` stores immutable module artifact.
- Object path `iac/inputs/{tenant_id}/{module_id}/{input_digest}.json` stores typed input overlay.
- Object path `iac/plans/{tenant_id}/{plan_id}.tfplan` stores plan artifact.
- State key `state/{pack_code}/{tenant_id}/{module_id}/{environment}.tfstate` is pack-pinned.
- REST endpoint `POST /v1/cloud-iac/modules` registers module.
- REST endpoint `POST /v1/cloud-iac/modules/{module_id}/versions` registers module version.
- REST endpoint `POST /v1/cloud-iac/modules/{module_id}/input-sets` creates tenant input set.
- REST endpoint `POST /v1/cloud-iac/plan-previews` creates plan preview.
- REST endpoint `POST /v1/cloud-iac/applies` applies approved plan.
- REST endpoint `POST /v1/cloud-iac/rollbacks` rolls back to previous version and input digest.
- REST endpoint `GET /v1/cloud-iac/modules/{module_id}/versions/{version}/schema` returns variable schema.
- AsyncAPI channel `cloud_iac.module.version.registered.v1` publishes version digest.
- AsyncAPI channel `cloud_iac.input_set.substituted.v1` publishes input digest.
- AsyncAPI channel `cloud_iac.plan_preview.completed.v1` publishes diff and risk.
- AsyncAPI channel `cloud_iac.apply.executed.v1` publishes apply evidence.
- AsyncAPI channel `cloud_iac.rollback.executed.v1` publishes rollback target.
- Cedar action `cloud_iac::module::register_version` requires module owner.
- Cedar action `cloud_iac::input_set::substitute` requires tenant operator and variable contract compliance.
- Cedar action `cloud_iac::plan_preview::create` requires read access to module and tenant input set.
- Cedar action `cloud_iac::apply::execute` requires approved plan and environment authority.
- Cedar action `cloud_iac::rollback::execute` requires incident or rollout authority.
- SLO target `cloud_iac_plan_preview_p99_seconds` is <=30.
- SLO target `cloud_iac_input_substitution_determinism_ratio` is 1.0.
- SLO target `cloud_iac_secret_reference_leak_total` is 0.
- SLO target `cloud_iac_apply_state_pin_correctness_ratio` is 1.0.
- SLO target `cloud_iac_module_rollback_p99_minutes` is <=5.

## Verification

- Unit test `input_resolution_order_is_stable` proves defaults, pack, tenant, environment, emergency order.
- Unit test `secret_reference_never_resolves_to_raw_value_in_plan_metadata` proves secret safety.
- Unit test `major_version_required_for_variable_removal` proves SemVer gate.
- Unit test `patch_version_cannot_change_plan_for_same_inputs` proves patch discipline.
- Unit test `provider_constraint_required_for_module_version` proves dependency pinning.
- Unit test `tenant_state_key_is_pack_pinned` proves residency.
- Contract test `plan_preview_contains_module_version_and_input_digest` proves deterministic key.
- Contract test `module_schema_endpoint_marks_sensitive_variables` proves safe UI.
- Property test `same_module_version_same_inputs_same_digest` proves substitution determinism.
- Replay test `apply_events_rebuild_current_module_version_matrix` proves audit projection.
- Integration test `tenant_overlay_cannot_set_unknown_variable` proves schema validation.
- Integration test `raw_secret_in_input_overlay_is_rejected` proves secret posture.
- Integration test `module_upgrade_requires_plan_preview_before_apply` proves workflow gate.
- Integration test `rollback_uses_previous_input_digest` proves safe rollback.
- Failure test `provider_lock_drift_blocks_apply` proves dependency reproducibility.
- Failure test `plan_preview_failure_does_not_mutate_state` proves preview safety.
- Security test `tenant_operator_cannot_apply_other_tenant_input_set` proves Cedar scope.
- Security test `emergency_override_expires_and_emits_audit` proves override discipline.
- Metric `cloud_iac_module_version_total` tracks versions by module and SemVer class.
- Metric `cloud_iac_input_substitution_duration_ms` tracks resolver performance.
- Metric `cloud_iac_plan_preview_duration_seconds` tracks preview latency.
- Metric `cloud_iac_secret_reference_reject_total` tracks unsafe input attempts.
- Metric `cloud_iac_apply_by_module_version_total` tracks rollout adoption.
- Metric `cloud_iac_rollback_total` tracks rollback by module and reason.
- Dashboard `cloud-iac-module-version-matrix` shows module versions, tenants, packs, and adoption.
- Dashboard `cloud-iac-input-substitution` shows overlay resolution, failures, and digest churn.
- Dashboard `cloud-iac-plan-preview` shows duration, risk class, and drift blockers.
- Dashboard `cloud-iac-rollback-readiness` shows last-green targets and rollback SLO.
- Alert `CloudIacRawSecretRejected` fires on every raw secret attempt.
- Alert `CloudIacPlanPreviewLatencyBurn` fires when p99 exceeds 30 seconds.
- Alert `CloudIacProviderLockDrift` fires when lock digest changes outside version registration.
- Alert `CloudIacStatePinViolation` fires on any cross-pack state key.

## References

- Internal: microservices/cloud-iac/PRD.md
- Internal: microservices/cloud-iac/tofu/modules/kms/README.md
- Internal: microservices/cloud-iac/tofu/modules/vpc/README.md
- Internal: docs/decisions/ADR-0709-general-live-apex.md
- Internal: docs/decisions/ADR-0702-identity-authz-live-apex.md
- OpenTofu module block syntax: https://opentofu.org/docs/language/modules/syntax/
- OpenTofu version constraints: https://opentofu.org/docs/language/expressions/version-constraints/
- OpenTofu provider requirements: https://opentofu.org/docs/language/providers/requirements/
- OpenTofu module registry protocol: https://opentofu.org/docs/v1.8/internals/module-registry-protocol/
- Terraform module sources documentation: https://developer.hashicorp.com/terraform/language/modules/sources
- Terraform module version constraints documentation: https://developer.hashicorp.com/terraform/language/modules/syntax
- Semantic Versioning 2.0.0: https://semver.org/
- Argo CD documentation: https://argo-cd.readthedocs.io/
- Flux documentation: https://fluxcd.io/flux/
- SLSA specification: https://slsa.dev/spec/
- Sigstore Cosign documentation: https://docs.sigstore.dev/cosign/
- Cedar policy language syntax: https://docs.cedarpolicy.com/policies/syntax-policy.html
- OpenTelemetry semantic conventions: https://opentelemetry.io/docs/concepts/semantic-conventions/
