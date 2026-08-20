---
id: ADR-APP-001
title: Entry Point Router with Per Tenant Feature Flag Injection
status: Proposed
date: 2026-05-20
microservice: application
related_oyatie_adrs:
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
decision_owner: axis-application
---

# ADR-APP-001: Entry Point Router with Per Tenant Feature Flag Injection

## Context

- Application is the tenant front door and modular product shell.
- It owns shell routing, tenant context, auth gateway, module loader, and frontend bundle serve.
- The PRD requires one tenant origin, SSO, route authorization, module isolation, SRI, and tenant admin enablement.
- Tenant product access depends on entitlements, feature flags, compliance packs, rollout state, and user roles.
- Named pressure APP-P1: route resolution must happen before product bundles render.
- Named pressure APP-P2: feature flags must be injected by tenant context, not hardcoded by product clients.
- Named pressure APP-P3: product modules need consistent flag evaluation across SSR, hydration, and client navigation.
- Named pressure APP-P4: a tenant must not see disabled or unpurchased modules.
- Named pressure APP-P5: flags can be safety-critical rollout gates, not only UI toggles.
- Named pressure APP-P6: flag evaluation must not leak tenant attributes to third-party providers.
- Named pressure APP-P7: route decisions need Cedar permit evidence and OTel spans.
- Named pressure APP-P8: global search and app switcher need a consistent enabled-module view.
- Named pressure APP-P9: cold-cache TTI must remain under the PRD budget.
- Named pressure APP-P10: internal admin routes and external tenant routes must stay separated.
- Constraint APP-C1: tenant identity comes from ADR-0002 and tenant scope from ADR-0244.
- Constraint APP-C2: Cedar gates route and module access per ADR-0007 and ADR-0243.
- Constraint APP-C3: Application shell doctrine comes from ADR-0061.
- Constraint APP-C4: feature flag substrate comes from ADR-0159.
- Constraint APP-C5: internal/external API separation comes from ADR-0177.
- Constraint APP-C6: step-up authentication classes come from ADR-0189.
- Constraint APP-C7: observability and evidence follow ADR-0263.
- Constraint APP-C8: application remains flat under ADR-0131 and does not own product business logic.
- Constraint APP-C9: product bundle integrity still uses SRI and signed manifests.
- Constraint APP-C10: route and flag APIs must remain additive.
- The current PRD names route resolution, tenant context, auth gateway, and module loader as separate bounded contexts.
- The decision must specify where feature flag injection happens.
- The decision must not turn Application into a product grouping layer.

## Decision

- Adopt an entry-point router with per-tenant feature flag injection.
- Name the pattern `TenantEntryRouter v1`.
- Resolve tenant context before route matching.
- Resolve session and ACR class before module manifest fetch.
- Evaluate Cedar route authorization before loading product bundles.
- Evaluate feature flags after tenant context and before module manifest response.
- Inject flag snapshots into the shell bootstrap payload and module manifest response.
- Use OpenFeature-compatible evaluation context semantics at the application boundary.
- Keep the in-house flag store as the authoritative provider.
- Do not call third-party flag services during route resolution.
- Use flag dimensions: tenant_id, pack_code, environment, module_id, user_role, ACR class, rollout cohort, and entitlement.
- Treat `tenant_id` as a low-level scope input and never as a client-provided trusted value.
- Treat route-level flags as server-side only.
- Treat presentation flags as client-readable only after server evaluation.
- Treat safety flags as default-deny on evaluator failure.
- Treat optional UI flags as default-off on evaluator failure.
- Bind every flag snapshot to a `flag_snapshot_id` and content digest.
- Include `flag_snapshot_id` in all downstream module bootstrap calls.
- Include Cedar permit id and flag snapshot id in route-resolution audit events.
- Cache flag snapshots per tenant and module for short TTLs with explicit invalidation.
- Invalidate snapshots on tenant entitlement change, compliance pack change, flag change, role change, and emergency kill switch.
- Require module owners to declare required flags in signed module manifests.
- Reject module manifest responses when required server-side flags are missing.
- Use app-switcher visible module list from the same route plus flag resolution pipeline.
- Keep product-specific business flags owned by product services, but surfaced through Application for entry routing.
- Keep route registry owned by Application shell-routing.
- Keep module bundle signing owned by module-loader and frontend-bundle-serve.
- Publish `application.route.resolved.v1` with route, permit, module, and snapshot.
- Publish `application.flag.snapshot.injected.v1` with digest and module list.
- Make this ADR authoritative for Application entry routing and flag injection.

## Alternatives Considered

### Client-Side Flag Evaluation Only

- Pros: simple product team integration.
- Pros: fewer server calls during route resolution.
- Pros: flexible UI experimentation.
- Cons: leaks flag inputs and disabled module names.
- Cons: cannot protect safety-critical routes.
- Cons: creates SSR and hydration mismatch risk.
- Rejected because route gating must be server-side and Cedar-bound.

### Product Service Evaluates Flags After Bundle Load

- Pros: product owners keep flag logic near business behavior.
- Pros: Application remains thinner.
- Pros: product services can use domain-specific inputs.
- Cons: disabled modules can be loaded before denial.
- Cons: app-switcher and global route list become inconsistent.
- Cons: TTI suffers from product round trips.
- Rejected for entry routing; product services may still evaluate deeper business flags.

### Third-Party Feature Flag SaaS at Route Time

- Pros: mature targeting UI.
- Pros: experimentation tools exist.
- Pros: SDKs are common.
- Cons: leaks tenant attributes and route decisions.
- Cons: creates external dependency for every app entry.
- Cons: conflicts with in-house and residency posture.
- Rejected for route-time flags.

### Static Tenant Module List Only

- Pros: deterministic and cacheable.
- Pros: easy to audit.
- Pros: no runtime flag evaluator dependency.
- Cons: cannot support progressive delivery or emergency kill switches.
- Cons: cannot express pack and role-specific module visibility.
- Cons: slow tenant rollout iteration.
- Rejected because ADR-0159 requires a feature flag substrate.

### API Gateway Owns Entry Routing

- Pros: all ingress routing centralized.
- Pros: good north-south enforcement posture.
- Pros: gateway already sees requests first.
- Cons: Application owns product shell route semantics.
- Cons: gateway should not know module manifests and app-switcher state.
- Cons: product bootstrap flags are not gateway concerns.
- Rejected because api-gateway routes traffic while Application routes user product surfaces.

## Consequences

- Positive: disabled modules are not loaded.
- Positive: route, Cedar, and flag evidence share one bootstrap trace.
- Positive: SSR, hydration, and client navigation receive consistent snapshots.
- Positive: app-switcher and route authorization use the same source of truth.
- Positive: emergency kill switches can remove a module at entry.
- Positive: third-party flag providers are not in the critical path.
- Negative: Application route resolution becomes more responsibility-heavy.
- Negative: flag snapshot invalidation must be correct.
- Negative: product teams must declare entry flags in manifests.
- Negative: cache mistakes can produce stale module visibility.
- Neutral: product services can still own deeper domain flags.
- Neutral: optional UI flags can be client-readable.
- Neutral: safety flags remain server-only.
- Neutral: internal admin routes stay separate.
- Follow-up work APP-F1: add `flag_snapshot_id` to module manifest schema.
- Follow-up work APP-F2: add route resolver trace examples.
- Follow-up work APP-F3: add emergency kill switch runbook.
- Follow-up work APP-F4: add app-switcher snapshot contract tests.
- Follow-up work APP-F5: add tenant entitlement invalidation event.

## Implementation Notes

- Data shape `TenantRouteRequest`: `{host, path, method, session_id, requested_module, request_id}`.
- Data shape `TenantRouteContext`: `{tenant_id, pack_code, user_id, roles, acr_class, entitlement_set_hash, environment}`.
- Data shape `FeatureFlagSnapshot`: `{snapshot_id, tenant_id, module_id, flag_values, digest, expires_at, source_version}`.
- Data shape `ModuleEntryManifest`: `{module_id, route_prefixes, bundle_ref, sri_hash, required_flags, min_acr_class}`.
- Data shape `RouteResolution`: `{tenant_id, route_id, module_id, permit_id, snapshot_id, decision, redirect}`.
- Data shape `FlagInvalidation`: `{tenant_id, module_id, reason, changed_by, created_at, audit_event_id}`.
- Data shape `AppSwitcherProjection`: `{tenant_id, user_id, modules, snapshot_id, generated_at}`.
- Postgres table `application_route_registry` stores route to module mappings.
- Postgres table `application_module_entry_manifest` stores signed manifest metadata.
- Postgres table `application_flag_snapshot` stores short-lived snapshots and digest.
- Postgres table `application_flag_invalidation` stores invalidation reasons.
- Valkey key `app:flag_snapshot:{tenant_id}:{module_id}:{user_hash}` stores hot snapshots.
- REST endpoint `POST /v1/application/routes/resolve` resolves entry route.
- REST endpoint `GET /v1/application/modules/{module_id}/entry-manifest` returns signed manifest plus snapshot.
- REST endpoint `GET /v1/application/app-switcher` returns enabled modules for current user.
- REST endpoint `POST /v1/application/flags/invalidate` invalidates snapshots.
- REST endpoint `POST /v1/application/modules/{module_id}/kill-switch` disables module entry.
- AsyncAPI channel `application.route.resolved.v1` emits route decision.
- AsyncAPI channel `application.flag.snapshot.injected.v1` emits snapshot injection.
- AsyncAPI channel `application.flag.snapshot.invalidated.v1` emits invalidation.
- AsyncAPI channel `application.module.kill_switch.engaged.v1` emits kill switch.
- Cedar action `application::route::resolve` requires authenticated session and tenant match.
- Cedar action `application::module::load` requires entitlement, role, pack, and ACR class.
- Cedar action `application::flag::snapshot_read` requires module owner or route resolver.
- Cedar action `application::flag::invalidate` requires tenant admin or rollout operator.
- Cedar action `application::module::kill_switch` requires incident commander or service owner.
- SLO target `application_route_resolve_p95_ms` is <=100.
- SLO target `application_flag_snapshot_injection_p95_ms` is <=25 after context load.
- SLO target `application_disabled_module_load_total` is 0.
- SLO target `application_tti_warm_p99_ms` is <=2000.
- SLO target `application_flag_snapshot_staleness_p95_seconds` is <=60.

## Verification

- Unit test `route_resolve_requires_tenant_context_first` proves ordering.
- Unit test `disabled_module_never_gets_manifest` proves no bundle load.
- Unit test `server_side_safety_flag_defaults_deny` proves safe evaluator failure.
- Unit test `presentation_flag_defaults_off` proves optional UI fallback.
- Unit test `acr_class_blocks_high_risk_admin_route` proves step-up integration.
- Unit test `flag_snapshot_digest_changes_on_entitlement_change` proves invalidation input.
- Contract test `module_entry_manifest_includes_snapshot_id` proves loader contract.
- Contract test `app_switcher_uses_same_snapshot_as_route_resolver` proves consistency.
- Property test `route_resolution_same_context_same_digest` proves deterministic snapshots.
- Replay test `route_resolved_events_rebuild_module_visibility` proves evidence projection.
- Integration test `tenant_without_module_entitlement_cannot_load_bundle` proves entitlement gate.
- Integration test `kill_switch_removes_module_from_app_switcher_and_route` proves emergency path.
- Integration test `ssr_and_hydration_use_same_flag_snapshot` proves UI consistency.
- Failure test `flag_store_unavailable_denies_safety_route` proves safe fallback.
- Failure test `cache_stale_after_invalidation_fails_test` proves invalidation.
- Security test `client_supplied_tenant_id_ignored` proves host/session authority.
- Security test `third_party_flag_provider_not_called_on_route_resolve` proves residency posture.
- Metric `application_route_resolve_duration_ms` tracks route latency.
- Metric `application_flag_snapshot_injection_duration_ms` tracks flag injection.
- Metric `application_disabled_module_load_total` tracks blocked disabled module attempts.
- Metric `application_flag_snapshot_staleness_seconds` tracks stale cache.
- Metric `application_module_kill_switch_total` tracks emergency disables.
- Dashboard `application-entry-router` shows route decisions, latency, and denies.
- Dashboard `application-feature-flag-injection` shows snapshot latency, staleness, and invalidations.
- Dashboard `application-module-visibility` shows app-switcher modules by tenant and role.
- Dashboard `application-kill-switches` shows active module kill switches and recovery.
- Alert `ApplicationDisabledModuleLoadAttempt` fires on any disabled module load attempt.
- Alert `ApplicationRouteResolveLatencyBurn` fires when p95 exceeds 100 ms.
- Alert `ApplicationFlagSnapshotStale` fires when p95 staleness exceeds 60 seconds.
- Alert `ApplicationTtiWarmRegression` fires when warm p99 exceeds 2 seconds.

## References

- Internal: microservices/application/PRD.md
- Internal: microservices/application/policy/route-isolation.md
- Internal: docs/decisions/ADR-0709-general-live-apex.md
- Internal: docs/decisions/ADR-0701-monorepo-capability-live-apex.md
- Internal: docs/decisions/ADR-0701-monorepo-capability-live-apex.md
- Internal: docs/decisions/ADR-0709-general-live-apex.md
- OpenFeature evaluation context specification: https://openfeature.dev/specification/sections/evaluation-context/
- OpenFeature flag evaluation specification: https://openfeature.dev/specification/sections/flag-evaluation/
- OpenAPI Specification: https://spec.openapis.org/oas/
- W3C Subresource Integrity: https://www.w3.org/TR/SRI/
- W3C Content Security Policy Level 3: https://www.w3.org/TR/CSP3/
- W3C Trace Context: https://www.w3.org/TR/trace-context/
- OpenTelemetry semantic conventions: https://opentelemetry.io/docs/concepts/semantic-conventions/
- Cedar policy language syntax: https://docs.cedarpolicy.com/policies/syntax-policy.html
- CloudEvents Specification: https://cloudevents.io/
- OWASP ASVS: https://owasp.org/www-project-application-security-verification-standard/
