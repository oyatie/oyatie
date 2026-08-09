---
id: ADR-SDK-0003
title: "Developer sandboxes are tenancy sandbox-class tenants"
status: Proposed
date: 2026-05-18
microservice: developer-sdk
related_oyatie_adrs:
  - ADR-0131
  - ADR-0173
  - ADR-0213
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0263
decision_owner: axis-ecosystem + council-tenancy
---

# ADR-SDK-0003: Developer sandboxes are tenancy sandbox-class tenants

## Context

- The named pressure is `sandbox-isolation-without-a-shadow-tenant-system`.
- Developer-sdk must give every registered developer a safe environment to test APIs, webhooks, SDK examples, payout callbacks, and marketplace submissions.
- The prior incident class is `sandbox_fork_drift`: a separate sandbox model diverged from the tenancy model and let tests pass in sandbox while production denied the same principal.
- The second prior incident class is `demo_tenant_unscoped`: demonstration resources were not bound to a real tenant id and bypassed quota and Cedar gates.
- The third prior incident class is `sandbox_cleanup_orphaned`: temporary developer resources outlived their TTL and accumulated cost.
- ADR-0242 states that oyatie itself is a tenant; sandbox actors must therefore still be tenant-shaped.
- ADR-0244 states that tenant is the universal scoping primitive; developer sandbox cannot be a parallel scope.
- ADR-0243 states that policy decisions are Cedar decisions; sandbox entitlements must flow through the same policy engine.
- ADR-0263 requires telemetry from sandbox actions so leaked or abusive sandbox usage is visible.
- ADR-0213 makes developer-sdk the public Ecosystem-as-a-Service surface, so sandbox semantics are part of the external contract.
- Developers need production-like API behavior without the right to mutate real tenant data.
- Sandbox APIs must be safe for public docs and tutorials.
- Sandbox tenants must be quota-limited, short-lived by default, and cheap to garbage-collect.
- Sandbox tenants must still support pack overlays for developers building regulated integrations.
- Sandbox tenants must support webhook, event, and payout simulations without contacting external banks or regulators.
- Sandbox tenants must use fake-but-typed identifiers, not arbitrary strings.
- Sandbox tenants must produce audit events marked as sandbox so evidence stores do not confuse them with production.
- Sandbox tenants must be revocable instantly when a developer account is suspended.
- Sandbox tenants must not require a separate database schema per developer when the tenancy service already owns provisioning.
- Sandbox tenants must remain visible in the developer portal with usage, TTL, quota, and cleanup state.

## Decision

- We choose `tenancy sandbox-class tenant` as the only developer sandbox primitive.
- The named pattern is `first-class test tenant`, similar to Stripe test mode, but implemented as real tenancy rows instead of an account-level mode flag.
- Developer-sdk calls tenancy endpoint `POST /v1/tenants/sandboxes` to create sandboxes.
- The resource type is `Tenancy::Tenant` with `tenant_class="sandbox"`.
- The owner principal is `DeveloperSdk::DeveloperAccount`.
- The tenant id uses prefix `tn_sbx_`.
- Sandbox resources use the same tenant_id columns as production resources.
- Sandbox resources set `environment="sandbox"` in Cedar context.
- Sandbox resources set `data_class="synthetic"` unless a fixture explicitly models regulated data.
- Default sandbox TTL is 14 days.
- Maximum sandbox TTL is 90 days.
- Default developer sandbox count is 3 active sandboxes.
- Enterprise marketplace partners may request up to 25 active sandboxes.
- Default API rate limit is 1,000 requests per hour per sandbox.
- Default event emission limit is 10,000 events per day per sandbox.
- Default storage quota is 10 GiB per sandbox.
- Default payout simulation amount cap is 10,000 synthetic currency units per day.
- Sandbox tenants can install compliance pack overlays only in `simulation` mode.
- Sandbox tenant pack overlays never assert production certification readiness.
- Sandbox webhook deliveries use `sandbox-signing-key`, not production webhook signing keys.
- Sandbox payout callbacks never leave developer-sdk and payments simulation adapters.
- Sandbox events emit to the audit-chain with `sandbox=true`.
- Sandbox logs have 30-day retention by default.
- Sandbox audit summary has 1-year retention for abuse investigation.
- Sandbox deletion is a tenancy lifecycle operation, not a developer-sdk local deletion.
- Developer-sdk owns only developer UX, SDK helpers, and bootstrap token issuance.
- Tenancy owns tenant state, lifecycle locks, quota, and cleanup.
- Cedar action `developer-sdk.sandbox.create` gates sandbox creation.
- Cedar action `developer-sdk.sandbox.delete` gates sandbox deletion.
- Cedar action `developer-sdk.sandbox.extend_ttl` gates TTL extension.
- Cedar action `developer-sdk.sandbox.bootstrap_token.issue` gates SDK bootstrap token issuance.
- The p95 sandbox create latency target is 3 seconds.
- The p99 sandbox create latency target is 8 seconds.
- Cleanup lag SLO is p99 <= 30 minutes after TTL expiry.
- Abuse revocation target is <= 60 seconds from developer suspension.

## Alternatives Considered

### Separate developer-sdk sandbox table

- Pro: developer-sdk can build quickly without tenancy dependency.
- Pro: sandbox cleanup can be local to developer-sdk.
- Pro: developer portal queries are simple.
- Con: tenant scope diverges from production.
- Con: Cedar policies need a second resource universe.
- Con: quota and lifecycle rules duplicate tenancy.
- Con: integration tests may pass in sandbox and fail in production.
- Tradeoff: speed but structural drift.
- Rejected because `sandbox_fork_drift` is the exact failure to prevent.

### Account-level test mode flag

- Pro: familiar Stripe-like mental model.
- Pro: one developer account can flip between live and test.
- Pro: fewer tenant records.
- Con: every downstream service must branch on mode.
- Con: one missing branch can mutate production data.
- Con: tenant_id is no longer the universal partition key.
- Con: audit semantics become ambiguous.
- Tradeoff: familiar UX but weaker isolation.
- Rejected; we can present a test-mode UX over first-class sandbox tenants.

### Ephemeral local-only sandbox

- Pro: no shared infrastructure cost.
- Pro: developer can run offline.
- Pro: good for simple examples.
- Con: does not test Cedar, tenancy, eventing, webhooks, or pack overlays.
- Con: external marketplace integrations need shared callback URLs.
- Con: support cannot inspect remote evidence.
- Tradeoff: convenience but weak integration fidelity.
- Partially accepted as SDK mock mode, not as canonical sandbox.

### Fork production tenant with masked data

- Pro: realistic state.
- Pro: useful for enterprise staging.
- Pro: catches integration bugs against true tenant shape.
- Con: high privacy risk.
- Con: expensive masking and DSR obligations.
- Con: easy to create cross-environment data confusion.
- Tradeoff: realism but excessive risk for general developer sandboxes.
- Rejected for public developer sandboxes; enterprise staging requires a separate controlled ADR.

## Consequences

- Positive: sandbox behavior exercises the same tenant and Cedar primitives as production.
- Positive: developer examples use real tenant ids and real API shapes.
- Positive: cleanup, quota, and lifecycle locks are inherited from tenancy.
- Positive: audit and telemetry clearly distinguish sandbox from production.
- Positive: sandbox pack simulation teaches regulated integration behavior without asserting certification.
- Negative: developer-sdk is now dependent on tenancy availability for sandbox creation.
- Negative: sandbox creation has more latency than inserting a local developer-sdk row.
- Negative: tenancy must support enough sandbox UX metadata for developer portal display.
- Negative: cleanup bugs can become tenancy bugs, not just developer-sdk bugs.
- Neutral: local mock mode remains useful but is explicitly not the canonical sandbox.
- Neutral: test-mode UI can still hide tenant mechanics from developers.
- Follow-up work: implement `SDK-IP-003-sandbox-tenant-bootstrap`.
- Follow-up work: add tenancy cleanup evidence to developer portal.
- Follow-up work: add sandbox pack simulation matrix.
- Follow-up work: add quota dashboard for developer sandboxes.

## Implementation Notes

- Data shape `DeveloperSandboxTenantV1` wraps a tenancy tenant reference.
- Field `sandbox_id` is a ULID prefixed by `sbx_`.
- Field `tenant_id` is a tenancy id prefixed by `tn_sbx_`.
- Field `developer_account_id` links to `DeveloperSdk::DeveloperAccount`.
- Field `tenant_class` is always `sandbox`.
- Field `environment` is always `sandbox`.
- Field `created_at` is RFC 3339.
- Field `expires_at` is RFC 3339 and required.
- Field `max_ttl_days` defaults to 14 and caps at 90.
- Field `quota_profile` defaults to `developer_sandbox_default`.
- Field `pack_simulations` is an array of pack ids in simulation mode.
- Field `bootstrap_token_ref` points to an OpenBao transit-signed token.
- Field `cleanup_state` is one of `active`, `expiring`, `delete_requested`, `deleted`, or `cleanup_failed`.
- Field `audit_sandbox_marker` is always true.
- API endpoint `POST /v1/developer/sandboxes` creates a sandbox through tenancy.
- API endpoint `GET /v1/developer/sandboxes` lists current developer sandboxes.
- API endpoint `GET /v1/developer/sandboxes/{sandbox_id}` returns tenant id, TTL, quota, and pack simulation state.
- API endpoint `POST /v1/developer/sandboxes/{sandbox_id}/bootstrap-token` issues an SDK bootstrap token.
- API endpoint `POST /v1/developer/sandboxes/{sandbox_id}/extend` requests TTL extension.
- API endpoint `DELETE /v1/developer/sandboxes/{sandbox_id}` requests cleanup.
- Tenancy endpoint `POST /v1/tenants/sandboxes` remains the source of truth.
- Cedar principal is `DeveloperSdk::DeveloperAccount::"<developer_account_id>"`.
- Cedar action `developer-sdk.sandbox.create` applies to resource `Tenancy::TenantClass::"sandbox"`.
- Cedar action `developer-sdk.sandbox.delete` applies to resource `DeveloperSdk::SandboxTenant`.
- Cedar action `developer-sdk.sandbox.extend_ttl` applies to resource `DeveloperSdk::SandboxTenant`.
- Cedar action `developer-sdk.sandbox.bootstrap_token.issue` applies to resource `DeveloperSdk::SandboxTenant`.
- Cedar context field `active_sandbox_count` must be <= the developer's quota.
- Cedar context field `requested_ttl_days` must be <= 90.
- Cedar context field `pack_mode` must equal `simulation`.
- Cedar context field `developer_status` must equal `active`.
- Example permit: principal `DeveloperSdk::DeveloperAccount::"dev_01HY"`, action `developer-sdk.sandbox.create`, resource `Tenancy::TenantClass::"sandbox"`, context `{active_sandbox_count:1, requested_ttl_days:14, developer_status:"active"}`.
- Example forbid: same principal and action with context `{active_sandbox_count:4, quota:3}`.
- Example forbid: same principal and action with context `{pack_mode:"production"}`.
- Bootstrap token audience is `developer-sdk.sandbox`.
- Bootstrap token TTL is 24 hours.
- Bootstrap token includes `sandbox_id`, `tenant_id`, `developer_account_id`, `cell_id`, and `pack_simulations`.
- Sandbox webhook signature key path is `transit/keys/{cell_id}/{pack_id}/developer-sdk/sandbox-webhook-ed25519-v1`.
- Sandbox cleanup worker listens for tenancy event `TenantSandboxExpired`.
- Sandbox cleanup worker calls deletion through tenancy, not direct table mutation.
- OpenTelemetry span `developer_sdk.sandbox.create` wraps developer-sdk and tenancy calls.
- Metric `oya_developer_sdk_sandbox_create_latency_ms` tracks create latency.
- Metric `oya_developer_sdk_sandbox_active_total` tracks active sandboxes by cell and quota profile.
- Metric `oya_developer_sdk_sandbox_cleanup_lag_seconds` tracks cleanup lag.
- Metric `oya_developer_sdk_sandbox_quota_denied_total` tracks denied creation attempts.
- Dashboard `developer-sdk-sandbox-tenants.json` shows active count, create latency, cleanup lag, quota denials, and pack simulation mix.
- SLO `developer-sdk-sandbox-create-latency.openslo.yaml` sets p95 <= 3 seconds and p99 <= 8 seconds.
- SLO `developer-sdk-sandbox-cleanup-lag.openslo.yaml` sets p99 <= 30 minutes after expiry.
- Failure mode `tenancy_unavailable` returns 503 and prevents sandbox creation.
- Failure mode `quota_exceeded` returns 429 with current quota state.
- Failure mode `cleanup_failed` opens a Sev-3 ticket after 30 minutes and a Sev-2 after 2 hours.
- Failure mode `developer_suspended` triggers all sandbox bootstrap tokens revoked within 60 seconds.
- Failure mode `pack_simulation_mismatch` returns 403 and emits `DeveloperSandboxPackSimulationDenied`.

## Verification

- Test `sandbox_creation_uses_tenancy_endpoint` verifies developer-sdk never inserts tenant rows directly.
- Test `sandbox_tenant_has_tenant_class_sandbox` verifies created tenants carry `tenant_class=sandbox`.
- Test `sandbox_resources_carry_tenant_id` verifies downstream events include `tenant_id`.
- Test `sandbox_creation_quota_enforced` verifies Cedar denies over-quota developers.
- Test `sandbox_ttl_extension_cap_enforced` verifies TTL cannot exceed 90 days.
- Test `sandbox_bootstrap_token_audience` verifies token audience is `developer-sdk.sandbox`.
- Test `sandbox_webhook_uses_sandbox_key` verifies production webhook key cannot sign sandbox callbacks.
- Test `sandbox_pack_production_forbidden` verifies pack mode cannot be production.
- Test `developer_suspension_revokes_sandboxes` verifies revocation target <= 60 seconds.
- Test `sandbox_cleanup_after_ttl` verifies cleanup event deletes resources through tenancy.
- Metric `oya_developer_sdk_sandbox_create_latency_ms` must meet p95 <= 3 seconds in reference cell.
- Metric `oya_developer_sdk_sandbox_cleanup_lag_seconds` must meet p99 <= 30 minutes.
- Metric `oya_developer_sdk_sandbox_quota_denied_total` must be visible by tenant_class.
- Dashboard `developer-sdk-sandbox-tenants.json` must include cleanup failures and active quota usage.
- Dashboard `tenancy-sandbox-class.json` must include developer-sdk-created sandboxes.
- CI check `developer-sdk-no-shadow-tenant-table` rejects local sandbox tenant tables.
- CI check `oya-governance-tenant-scope --microservice developer-sdk` verifies tenant id propagation.
- CI check `oya-governance-cedar-action-coverage --microservice developer-sdk` verifies sandbox endpoints map to Cedar.
- CI check `sandbox-token-production-key-boundary` verifies sandbox tokens cannot use release signing keys.
- CI check `oya-governance-observability-emission --microservice developer-sdk` validates audit, metric, trace coverage.
- Load test creates 1,000 sandboxes in one cell and requires p99 <= 8 seconds.
- Chaos test makes tenancy unavailable and verifies fail-closed create behavior.
- Chaos test delays cleanup event and verifies cleanup lag alert.
- Security test attempts production pack activation from sandbox and expects 403.
- Audit query verifies every sandbox create has `DeveloperSandboxCreated` event with `sandbox=true`.

## References

- ADR-0131: Per-microservice flat layout.
- ADR-0173: Vendor lock-in avoidance and stack ownership.
- ADR-0213: Ecosystem-as-a-Service architecture.
- ADR-0242: Oyatie is a tenant doctrine.
- ADR-0243: Cedar as Universal Gate.
- ADR-0244: Tenant as universal scoping primitive.
- ADR-0263: Observability emission contract.
- Stripe API test mode documentation as industry precedent.
- GitHub Codespaces lifecycle and quota documentation as sandbox precedent.
- AWS IAM sandbox/account-vending patterns.
- NIST SP 800-190: Application Container Security Guide.
- SOC 2 CC6.6 access removal expectations.
- ISO/IEC 27001:2022 A.5.18 access rights.
- OpenBao transit documentation for sandbox token signing.
- OpenTelemetry semantic conventions.
- Oya tenancy microservice manifest and sandbox-class registry.
- Oya VCS admission evidence model for sandbox lifecycle changes.
- Kubernetes namespace lifecycle controller documentation as cleanup precedent.
- Google Cloud project factory pattern as account-vending precedent.
