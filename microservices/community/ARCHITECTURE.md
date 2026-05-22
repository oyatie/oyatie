# community

_This file was created by the Wave-3-C anchor-sweep. Expand all stub sections during content-pass review._

---



## §principals
This anchor is closed for `community` against ADR-0242 §D-1: principal roster and tenant-scoped caller model.

### Service-specific answer
- Platform principal `oyatie.community.runtime` owns normal `moderate-action` execution and never borrows a tenant principal.
- Platform principal `oyatie.community.worker` owns async jobs, retry queues, and backfill replay listed in `microservices/community/backfill-replay.md` when present.
- Platform principal `oyatie.community.auditor` has read-only evidence access through auditor Cedar fragments, not direct database credentials.
- Platform principal `oyatie.community.ci` is limited to synthetic tenants and non-production cells by the CI-scope Cedar fragment.
- Tenant callers are represented as `<tenant>.community.caller` and must provide `tenant_id`, `principal_id`, `audience_type`, and workload SVID context.
- Cross-µservice callers expected by dependency graph: `identity`, `tenancy`, `policy-engine`, `observability`, `audit-chain`, `cloud-secrets`.
- Example: a tenant principal invoking `moderate-action` is evaluated as `<tenant>.community.moderate-action` before any `community` state mutation.
- Forbidden: caller-supplied `oyatie.*` principals; ADR-0242 treats `oyatie` as its own tenant, not a namespace tenants can impersonate.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: AWS IAM service-linked roles is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud service agents is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §cedar-gates
This anchor is closed for `community` against ADR-0243 §D-2: Cedar fragment roster, default-deny and action taxonomy.

### Service-specific answer
- Default-deny is represented by the first matching Cedar fragment in `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- Action namespace uses `community::<bounded_context>::<verb>`; the first protected action is `community::moderate-action::execute`.
- Every evaluation context carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, `bot_score`, and `provider_credential_mode`.
- Mutating actions require an audit event class before the usecase layer runs; read actions require a purpose and data-class declaration.
- Auditor reads are time-boxed and read-only; CI principals are sandbox-only; emergency bypass policies never skip audit emission.
- Concrete fragments in scope: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- Example: `community::moderate-action::execute` denies if `resource.tenant_id != principal.tenant_id` or if the compliance pack adds a stricter overlay.
- Fragment publish observes ADR-0294 soak before activation; rollback reverts the fragment pointer, not the business-state rows.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: AWS Verified Permissions Cedar policy evaluation is the reference pattern for the control shape described here.
- Precedent 2: Google Zanzibar relationship checks is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §tenant-scoping
This anchor is closed for `community` against ADR-0244 §D-3: tenant_id, audience_type, provider_credential_mode and row/event boundaries.

### Service-specific answer
- Audience type for this service is `B2C_CONSUMER + B2B_TENANT` and is copied into audit events plus any public contract response that exposes policy posture.
- Provider credential mode is `tenant-provider-byok where external providers exist; platform-default credentials otherwise`; provider-BYOK and encryption-BYOK stay separate per ADR-0255 §D-4 / ADR-0251 §D-10.
- Required fields on mutable rows/events: `tenant_id`, `principal_id`, `caller_tenant_id` when delegated, `home_cell`, `jurisdiction_code`, `data_class`, and `audit_event_class`.
- State surface `community.community` is documented as tenant-scoped with `tenant_id`, `principal_id`, `audit_event_class`, `home_cell`, and `lifecycle_state` columns/events.
- Planned table/event surface `community.moderate_action_2` inherits the same `tenant_id` and audit fields until a migration file supersedes this placeholder name.
- Planned table/event surface `community.moderate_action_3` inherits the same `tenant_id` and audit fields until a migration file supersedes this placeholder name.
- Planned table/event surface `community.moderate_action_4` inherits the same `tenant_id` and audit fields until a migration file supersedes this placeholder name.
- Planned table/event surface `community.moderate_action_5` inherits the same `tenant_id` and audit fields until a migration file supersedes this placeholder name.
- Cross-tenant reads fail at Cedar before storage adapters see a query; storage row-level policy is defence-in-depth, not the primary guard.
- Example: `moderate-action` reads include `tenant_id` and `home_cell`; a stale `jurisdiction_code` forces most-restrictive-pack handling until tenancy refresh completes.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: Stripe Connect connected-account isolation is the reference pattern for the control shape described here.
- Precedent 2: AWS Organizations account-boundary pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §substrate-product-binding
This anchor is closed for `community` against ADR-0245 §D-1: substrate/product classification and dependency direction.

### Service-specific answer
- Manifest classifies `community` as `product`, so this section treats it as a product consumer.
- Declared substrate/product dependencies: `identity`, `tenancy`, `policy-engine`, `observability`, `audit-chain`, `cloud-secrets`.
- If substrate: products consume `community` only through contracts `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- If product: `community` may call substrate services but must not create product-to-product synchronous dependencies.
- Dependency direction is inward to clean core crates; adapter and framework code never defines domain terms for other µservices.
- Primary bounded contexts bound to this classification: `community`.
- Example: `moderate-action` may depend on `tenancy` for tenant state and `observability` for audit emission, but not on another product UI workflow.
- ADR-0280 substrate-of-substrate ordering is documented here so delivery planning can parallelize product work without creating hidden runtime coupling.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: Palantir Foundry shared ontology substrate is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud shared VPC/service-project split is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §policy-evaluation
This anchor is closed for `community` against ADR-0246 §D-4: library-first policy evaluation mode and fallback limits.

### Service-specific answer
- `policy_evaluation_mode = library-first`; network policy-engine calls are fallback only for stale local fragment cache or explicit audit replay.
- The caller-side library evaluates fragments from `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more with the full action/resource/context tuple.
- Partial-context evaluation is forbidden: no action can be evaluated without `tenant_id`, `principal_id`, `audience_type`, `resource_id`, and `data_class`.
- Cache freshness target is ≤5 minutes for normal fragments and immediate invalidation for deny-list, credential, or pack-overlay revocations.
- Fallback network evaluation emits `PolicyEvaluationFallbackUsed` with fragment hash and reason so ADR-0263 can detect degraded posture.
- Example: `community::moderate-action::execute` evaluates locally, then emits an audit event before the usecase writes `community.community`.
- OpenBao credential sidecar is never called before policy allow; policy decides whether the credential lookup is authorized.
- This mirrors OPA sidecar discipline: policy is close to the caller, but the source of truth remains signed and soaked Cedar fragments.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: AWS Verified Permissions local Cedar model is the reference pattern for the control shape described here.
- Precedent 2: Open Policy Agent sidecar evaluation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §cell-eligibility
This anchor is closed for `community` against ADR-0248 §D-1: cell tier, shard width, DR pair and shuffle-shard behavior.

### Service-specific answer
- Cell eligibility declaration: `not declared in manifest; bound here to the conservative platform default`.
- Tier 0/1 control-plane paths run in hardened cells; tenant data planes can shard per tenant, pack, region, and workload class.
- Per-cell shard key is `(tenant_id, home_cell, jurisdiction_code)`; DR pair selection uses `dr_cell` where data-residency permits failover.
- Shuffle-shard width is documented by `multi-region.md` or defaults to three independent cells for Tier-1 control paths.
- Regional outage behavior: keep reads local where pack permits, stop cross-border replication where pack forbids it, and preserve audit emission locally.
- Example: `moderate-action` traffic in a KR pack tenant stays in KR home cell; DR failover requires pack approval and emits a cell-failover audit event.
- Capacity math lives in `capacity-model.md`; this section binds the shard dimensions so the math is not detached from topology.
- Cloud Hypervisor/Kata isolation applies to Tier 0/1 pods; Tier 2/3 paths inherit the same network policy and SPIFFE identity floor.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: AWS cell-based architecture is the reference pattern for the control shape described here.
- Precedent 2: Route 53 shuffle-sharding isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §intelligence-dispatch
This anchor is closed for `community` against ADR-0255 §D-2: Intelligence calls, library-first dispatch and audience tags.

### Service-specific answer
- `community` uses Intelligence only through a declared dispatch boundary; default is library-first and network-opt-in must name a latency budget.
- Audience tag format is `community.moderate-action` and is included in every AI/risk/model call audit event.
- No prompt, model choice, tool call, or generated output may cross tenant boundaries; `tenant_id` and `data_class` travel with the request.
- Provider credential mode follows tenant provider-BYOK where configured; default provider credentials are platform-owned only for platform-permitted consumer paths.
- Heavy network calls route to `intelligence` gRPC only when local dispatch cannot satisfy the model/tool requirement.
- Example: `moderate-action` can request classification or summarization with `audience_tag=community.moderate-action`; the response is logged with model version and policy decision.
- Failure mode: if Intelligence is unavailable, the usecase must choose deterministic fallback, queue for retry, or deny high-risk automation rather than inventing output.
- Cross-links: `intelligence`, `policy-engine`, `observability`, and `cloud-secrets` own the model routing, policy decision, audit, and credential surfaces.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: Palantir AIP tool execution boundary is the reference pattern for the control shape described here.
- Precedent 2: Azure OpenAI tenant-isolated deployment pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §ontology-read-path
This anchor is closed for `community` against ADR-0257 §D-2: Ontology entity reads, projection cache and freshness floor.

### Service-specific answer
- Ontology read mode is library-first projection cache for `community` unless this section documents a network-only exception.
- Entity types read by this µservice: `Tenant`, `Principal`, `PolicyGrant`, `CompliancePack`, plus bounded-context entities named below.
- Bounded-context entity `community` is read/written as an ontology node or edge projection when `community` needs cross-surface discovery.
- Freshness floor: 30 seconds for tenant/compliance/security fields; 5 minutes for catalog/product metadata; stale security fields force most-restrictive handling.
- Example: `moderate-action` resolves `tenant.compliance_packs`, `principal.audience_type`, and `resource.home_cell` before Cedar evaluation.
- Cache invalidation is event-driven from Ontology; network read is reserved for cache miss plus explicit latency budget.
- Cross-links: `ontology` owns type semantics; `tenancy` owns tenant lifecycle; `policy-engine` consumes the projection for Cedar context.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: Palantir Foundry ontology projection pattern is the reference pattern for the control shape described here.
- Precedent 2: Google Knowledge Graph serving-cache pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §transport
This anchor is closed for `community` against ADR-0253 §D-1: HTTP/3 fallback chain, strict TLS, ECH and PQC posture.

### Service-specific answer
- Public/API contracts in scope: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- REST and gRPC advertise HTTP/3 through `Alt-Svc: h3=":443"`; fallback order is HTTP/3, then HTTP/2, then HTTP/1.1 with first acceptable winning.
- HTTP/1.0, TLS <1.3, self-signed production certificates, `insecure_skip_verify`, and MITM bypass headers are forbidden.
- ECH is advertised through HTTPS RR `ech=` config wherever the platform terminates TLS; ECH-disabled clients fall back to ordinary TLS 1.3 without refusal.
- PQC hybrid `X25519MLKEM768` is offered where the client/server pair supports it; classical X25519/P-256 fallback is accepted during migration.
- IaC transport evidence: `microservices/community/iac/helm/community/Chart.yaml`, `microservices/community/iac/helm/community/templates/deployment.yaml`, `microservices/community/iac/helm/community/templates/hpa.yaml`, `microservices/community/iac/helm/community/templates/networkpolicy.yaml`, `microservices/community/iac/helm/community/templates/pdb.yaml`, `microservices/community/iac/helm/community/templates/prometheusrule.yaml`; +6 more.
- Example: `community` `moderate-action` calls use HTTP/3 on normal networks and HTTP/2 when UDP/QUIC is blocked by enterprise firewalls.
- Async/event transport preserves tenant and audit context on every message; webhook ingress verifies HMAC/mTLS before Cedar sees the payload.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: Google QUIC/HTTP3 rollout pattern is the reference pattern for the control shape described here.
- Precedent 2: Cloudflare ECH and post-quantum TLS experiments is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §deployment-shape
This anchor is closed for `community` against ADR-0254 §D-2: Kubernetes, Cloud Hypervisor, Kata, Wasm/container/VM split.

### Service-specific answer
- Runtime components derive from bounded contexts `community` and deploy as separate app/worker/API pods where files exist.
- Default shape is Kubernetes pods with Cloud Hypervisor + Kata isolation for Tier 0/1 paths; lower tiers inherit network policy and SPIFFE identity.
- Wasm is reserved for untrusted plugin or user-authored execution; normal business usecases remain containers; bootstrap/control roots may require VM isolation.
- IaC manifests in scope: `microservices/community/iac/helm/community/Chart.yaml`, `microservices/community/iac/helm/community/templates/deployment.yaml`, `microservices/community/iac/helm/community/templates/hpa.yaml`, `microservices/community/iac/helm/community/templates/networkpolicy.yaml`, `microservices/community/iac/helm/community/templates/pdb.yaml`, `microservices/community/iac/helm/community/templates/prometheusrule.yaml`; +6 more.
- Catalog crates/components: `microservices/community/catalog/oya-community-kb-article-store-adapter-postgres.yaml`, `microservices/community/catalog/oya-community-kb-article-store-adapter-s3.yaml`, `microservices/community/catalog/oya-community-kb-article-store-adapter.yaml`, `microservices/community/catalog/oya-community-kb-article-store-api.yaml`, `microservices/community/catalog/oya-community-kb-article-store-app.yaml`, `microservices/community/catalog/oya-community-kb-article-store-domain.yaml`; +18 more.
- Example: `moderate-action` API runs as a Kata-isolated container; scheduled/background work for `community` uses a separate worker principal.
- OpenBao and SPIFFE mounts are sidecars, not linked into domain/core crates; adapters own framework code.
- Rollback is deployment-level first: pin previous image digest, keep schema backward-compatible, and replay idempotent events after recovery.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: AWS Firecracker microVM isolation is the reference pattern for the control shape described here.
- Precedent 2: GKE Sandbox/Kata isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §observability
This anchor is closed for `community` against ADR-0263 §D-1: audit events, metrics, logs and trace span shape.

### Service-specific answer
- SLO/dashboard sources in scope: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Audit classes use prefix `oya.community.<context>.<outcome>` and must be registered centrally per ADR-0263.
- Core counter metric: `community_moderate_action_total` with dimensions `outcome`, `route_class`, `tenant_id_class`, and `cell_tier`.
- Core latency metric: `community_moderate_action_latency_ms` with bounded cardinality and no raw `tenant_id` label.
- Reference evidence artifact: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`.
- Trace root span is `<service>.<capability>`; child spans are `policy.evaluate`, `ontology.read`, `storage.write/read`, `audit.emit`, and provider/adapter calls.
- Logs are structured JSON, redacted by data class, retained per pack, and include correlation id plus audit event id.
- Example trace: `community.moderate-action` -> `policy.evaluate` -> `storage.moderate_action` -> `audit.emit`.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: Google SRE four primary SRE signals is the reference pattern for the control shape described here.
- Precedent 2: OpenTelemetry semantic conventions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §abuse-defence
This anchor is closed for `community` against ADR-0297 §D-3: anti-bot, anti-spoof, anti-scrape controls plus UX floor.

### Service-specific answer
- Internet-facing `community` routes use edge rate limits by IP, JA4 fingerprint, tenant, route class, and `moderate-action` action class.
- Passive bot scoring is forwarded as `X-Oya-Bot-Score`; Cedar composes the score with quota and tenant tier before any challenge.
- Default path is friction-free: legitimate traffic receives no CAPTCHA, no JS proof-of-work, and ≤2ms p99 edge scoring budget.
- Cedar/IaC controls in scope: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +16 more.
- Canary payloads for `community` use fake `moderate-action` identifiers and honey endpoints under `/.well-known/oya-canary/community`.
- Anti-spoof: HMAC on webhooks, SPIFFE SVID on service calls, SameSite=Strict on cookies, mTLS for machine clients, audit signatures via sidecar.
- Anti-scrape: per-fingerprint pagination caps, breadth-first crawl detection, watermarking for high-value content, and partner allow-list for friendly crawlers.
- Emergency-services and accessibility bypasses are evaluated before bot-score friction and still emit audit events.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: Cloudflare Bot Management and Turnstile is the reference pattern for the control shape described here.
- Precedent 2: Stripe Radar passive risk scoring is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §critical-path-edge-cases
This anchor is closed for `community` against documentation-rigor.md §3.2.5: applicable human-safety and platform edge-case handling.

### Service-specific answer
- Network partition: `community` keeps tenant-local reads when safe, stops cross-cell writes that would violate residency, and emits degraded-mode audit events.
- Byzantine caller: Cedar denies forged `principal_id`, mismatched `tenant_id`, invalid SVID, replayed idempotency keys, and suspicious bot-score context.
- Regional outage: home-cell failover follows `multi-region.md`; if a pack forbids cross-border DR, `community` preserves local queue state instead of failing open.
- Key compromise: ADR-0296 sidecar revokes OpenBao leases, rotates signing keys, and quarantines affected audit event classes for reconciliation.
- Account recovery/hijack path: identity step-up and `community` audit evidence keep legitimate recovery from becoming an adversary shortcut.
- Mistaken mutation path: high-impact `moderate-action` mutations require idempotency, undo/cooldown where product semantics allow, and sealed evidence for later correction.
- Disaster surge: `community` enforces per-tenant isolation so one hot tenant or emergency mode cannot starve unrelated cells.
- Verification: capacity math in `capacity-model.md`, rollback in `failure-modes.md`, DR handling in `multi-region.md`, and incident actions in runbooks.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: Google SRE incident playbooks is the reference pattern for the control shape described here.
- Precedent 2: Stripe idempotent mutation recovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §credential-isolation
This anchor is closed for `community` against ADR-0296 §D-1: credential sidecar, OpenBao TTLs and secret-reference path.

### Service-specific answer
- Credential scopes for `community` include `identity`, `tenancy`, `policy-engine`, `observability`, `audit-chain`, `cloud-secrets`.
- SecretReference pattern is `${openbao:secret/<tenant_id>/community/<credential-name>}`; platform-owned credentials use tenant `oyatie` only for oyatie-internal calls.
- Sidecar mode is required for audit-signing keys and preferred for provider credentials; raw credentials never enter domain/core crates.
- OpenBao token TTL is ≤60 seconds when sidecar isolation is not possible; refresh must be policy-allowed and audit-emitted.
- Rotation cadence is linked from `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.
- Example: `moderate-action` asks the sidecar for a scoped credential after Cedar allow; the sidecar returns a short-lived handle, not the underlying secret.
- Compromise response: revoke lease, rotate key, disable affected provider adapter, replay idempotent queue after audit-chain reconciliation.
- encryption-BYOK is separate from provider-BYOK; this section covers provider/API/signing credentials only unless a pack explicitly adds KMS-root behavior.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: HashiCorp Vault dynamic secrets is the reference pattern for the control shape described here.
- Precedent 2: AWS KMS envelope-key isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §community-as-4-pillar-product

**Status:** Accepted 2026-05-21 — supersedes the mis-scaffolded `microservices/anonymous/` µservice (deleted 2026-05-21 per user clarification; folded here) and absorbs the retired professional-network service per Wave 15K.

### Overview

`community` is the umbrella µservice for four product surfaces sharing one tenant-scoped substrate. The four surfaces are analogues of Reddit, Teamblind, Handshake, and the jobs/profile/recruiter subset of LinkedIn — offered as a 4-pillar product under a single Cedar gate + ontology read-path + audit-chain emission substrate. Tenant surface configuration selects which surfaces are active.

Wave 15K retires the former professional-network path because the name read like
networking infrastructure while the content was a LinkedIn-class professional
product. The professional content migrates here; infrastructure networking
remains a `cloud-network` concern, not a community concern.

The decision to fold anonymity into `community` (rather than keep it as `microservices/anonymous/`) is grounded in:
1. **Architectural coherence:** anonymous posting is a posting-mode, not a data-plane domain. Post storage, vote engine, feed render, search, and moderation are shared concerns across all four surfaces.
2. **Substrate deduplication:** ADR-0245 (substrate vs product) forbids per-surface duplication of the Cedar gate + ontology read-path. A standalone anonymous µservice would have duplicated community's core storage and moderation substrate.
3. **Cedar gate unification:** all four posting modes are gated by Cedar fragments under `community/policy/`. Splitting them across µservices would have required cross-µservice Cedar trust delegation — a pattern ADR-0243 explicitly discourages.

### Four product surfaces

| Surface | Analogue | Posting mode | Identity model |
|---|---|---|---|
| **Reddit-style** | Reddit | pseudonymous | User-chosen stable handle; no employer verification |
| **Teamblind-style** | Teamblind | persona-anchored | Verified employer + role; anonymous handle; blinded credential |
| **Handshake-style** | Handshake | identity-anchored | Verified student / candidate / employer; job-search and recruiting visible |
| **LinkedIn jobs/profile subset** | LinkedIn Jobs / Profile / Recruiter | identity-anchored | Verified professional identity; resume, skills, connections, InMail, endorsements, recommendations, recruiter search |

**Excluded from the LinkedIn subset:** engagement-optimized text feed, status
broadcasting, follower-acquisition mechanics, sponsored post promotion, and
algorithmic For-You-style attention ranking. Community-native posts use thread,
vote, moderation, and relevance signals instead.

Plus three **fully-anonymous** sub-surfaces per ADR-0300:

| Sub-surface | ADR-0300 invariant set | Capability record |
|---|---|---|
| Whistleblower submission | §W-* | `capabilities/whistleblower-submission.yaml` |
| SecureDrop press-source | §P-* | `capabilities/securedrop-press-source.yaml` |
| Bug-bounty submission | §B-* + §3.2.5 row 27 | `capabilities/bug-bounty-submission.yaml` |

### Posting-mode taxonomy

Anonymity and professional identity are posting modes within community, not separate µservices. The four posting modes form a spectrum:

```
identity-anchored          persona-anchored        pseudonymous        fully-anonymous
(LinkedIn/Handshake)       (Teamblind)             (Reddit)            (whistleblower /
                                                                        press-source /
                                                                        bug-bounty)
full identity visible  →   employer verified,   →  handle only,    →  no identity token
                           user_id hidden           no employer         attached to
                                                    verification        submission
```

Each mode maps to exactly one Cedar policy fragment:

| Mode | Cedar fragment |
|---|---|
| identity-anchored | `policy/anonymity-mode-identity-anchored.cedar` |
| persona-anchored | `policy/anonymity-mode-persona-anchored.cedar` |
| pseudonymous | `policy/anonymity-mode-pseudonymous.cedar` |
| fully-anonymous | `policy/anonymity-mode-fully-anonymous.cedar` |

A session is bound to exactly one posting mode at initiation. Switching modes requires a new session + re-authentication. Cedar enforces this: the persona-anchored fragment forbids `issue_blinded_credential` for identity-anchored principals; the identity-anchored fragment forbids `publish_post_anonymous` for verified professionals.

### Migrated professional-network content

| Retired `network` responsibility | Community owner |
|---|---|
| Resume / Profile aggregates, profile export, profile verification | `professional-profile` BC |
| Connections graph, connection requests, mutual connections, blocks/restricts | `professional-graph` + `connection-request` BCs |
| InMail-equivalent outreach | `inmail-bridge` BC through `messenger` |
| Endorsements + recommendations | `endorsement-engine` BC |
| Jobs, applications, recruiter-stub, ATS handoff | `jobs-recruiter` BC |
| Skill assessments | `skill-assessments` BC |
| Pages / Events | `pages-events` BC; forum-native events stay on existing `events` surface |

### Shared substrate (all four surfaces consume)

All four product surfaces and three fully-anonymous sub-surfaces share:

- **Cedar gate substrate** — ADR-0243; per-mode fragments layer on top of `policy/default-deny.cedar`
- **Ontology read-path** — ADR-0257; `ontology_read_mode = library`; all reads pass `tenant_id`
- **Audit-chain emission** — ADR-0263; every action emits a Merkle-sealed audit event
- **Post + thread storage** — Postgres/Citus post-store BC (community IP-002..IP-005)
- **Vote engine** — Wilson-score ranking (community IP-006)
- **Moderation queue** — human + auto-moderation pipeline (community IP-007)
- **Search index** — Meilisearch (community IP-009)
- **Intelligence dispatch** — ADR-0255; capability-switch policy, with employment-ranking paths treated as high-risk and audit-gated

### Surface differentiation (tenant surface config)

A tenant's community surface configuration selects active surfaces:

```json
{
  "surface_flags": [
    "teamblind-mode",
    "reddit-mode",
    "professional-profile-mode",
    "handshake-mode",
    "whistleblower-submission",
    "securedrop-press-source",
    "bug-bounty-submission"
  ]
}
```

A tenant that omits `teamblind-mode` gets no persona-anchored surface. Cedar gates enforce this: the persona-anchored fragment checks the tenant surface flag before allowing anonymous workplace posting. No direct API call can bypass the flag because the Cedar default-deny fragment runs first.

### Migration note: professional-network path retired 2026-05-21

The former professional-network directory is retired by Wave 15K. Its source
artifacts remain provenance for this merge, but the only live service path is
`microservices/community/`. Professional profile, connection, InMail,
endorsement, recommendation, jobs, recruiter, skill-assessment, pages, and
events docs must be authored under community going forward. Cloud-networking
counterparts such as AWS VPC Lattice, Google Cross-Cloud Network, and Azure
Virtual WAN belong to `cloud-network`, not to this product surface.

### Migration note: microservices/anonymous/ deleted 2026-05-21

The `microservices/anonymous/` directory was mis-scaffolded on 2026-05-20 by a Wave-3 agent that misread §3.2.5 critical-path matrix rows 6/7/27 as a standalone µservice. Per user clarification 2026-05-21, all 106 artifacts have been extracted into `community/` and the directory deleted. The focused extraction IP is `community/IP-N-anonymous-fold-extraction.md`. Every ADR previously scoped to `anonymous` (ADR-ANON-0001..0007) is now cross-referenced under community's decisions registry.

**Relevant references:**
- `community/IP-N-anonymous-fold-extraction.md` — extraction IP (pending → complete tracking)
- `docs/architecture/transition-anonymous-to-community-2026-05-21.json` — classification artifact
- `docs/decisions/superseded/` — any anonymous-scoped ADRs that are superseded
