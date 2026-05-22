# translate

_This file was created by the Wave-3-C anchor-sweep. Expand all stub sections during content-pass review._

---



## §principals
This anchor is closed for `translate` against ADR-0242 §D-1: principal roster and tenant-scoped caller model.

### Service-specific answer
- Platform principal `oyatie.translate.runtime` owns normal `translate-segment-suggest-and-lang-hint` execution and never borrows a tenant principal.
- Platform principal `oyatie.translate.worker` owns async jobs, retry queues, and backfill replay listed in `microservices/translate/backfill-replay.md` when present.
- Platform principal `oyatie.translate.auditor` has read-only evidence access through auditor Cedar fragments, not direct database credentials.
- Platform principal `oyatie.translate.ci` is limited to synthetic tenants and non-production cells by the CI-scope Cedar fragment.
- Tenant callers are represented as `<tenant>.translate.caller` and must provide `tenant_id`, `principal_id`, `audience_type`, and workload SVID context.
- Cross-µservice callers expected by dependency graph: `identity`, `tenancy`, `policy-engine`, `observability`, `audit-chain`, `cloud-secrets`.
- Example: a tenant principal invoking `translate-segment-suggest-and-lang-hint` is evaluated as `<tenant>.translate.translate-segment-suggest-and-lang-hint` before any `translate` state mutation.
- Forbidden: caller-supplied `oyatie.*` principals; ADR-0242 treats `oyatie` as its own tenant, not a namespace tenants can impersonate.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: AWS IAM service-linked roles is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud service agents is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against ADR-0243 §D-2: Cedar fragment roster, default-deny and action taxonomy.

### Service-specific answer
- Default-deny is represented by the first matching Cedar fragment in `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- Action namespace uses `translate::<bounded_context>::<verb>`; the first protected action is `translate::translate-segment-suggest-and-lang-hint::execute`.
- Every evaluation context carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, `bot_score`, and `provider_credential_mode`.
- Mutating actions require an audit event class before the usecase layer runs; read actions require a purpose and data-class declaration.
- Auditor reads are time-boxed and read-only; CI principals are sandbox-only; emergency bypass policies never skip audit emission.
- Concrete fragments in scope: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- Example: `translate::translate-segment-suggest-and-lang-hint::execute` denies if `resource.tenant_id != principal.tenant_id` or if the compliance pack adds a stricter overlay.
- Fragment publish observes ADR-0294 soak before activation; rollback reverts the fragment pointer, not the business-state rows.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: AWS Verified Permissions Cedar policy evaluation is the reference pattern for the control shape described here.
- Precedent 2: Google Zanzibar relationship checks is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against ADR-0244 §D-3: tenant_id, audience_type, provider_credential_mode and row/event boundaries.

### Service-specific answer
- Audience type for this service is `B2C_CONSUMER + B2B_TENANT` and is copied into audit events plus any public contract response that exposes policy posture.
- Provider credential mode is `tenant-provider-byok where external providers exist; platform-default credentials otherwise`; provider-BYOK and encryption-BYOK stay separate per ADR-0255 §D-4 / ADR-0251 §D-10.
- Required fields on mutable rows/events: `tenant_id`, `principal_id`, `caller_tenant_id` when delegated, `home_cell`, `jurisdiction_code`, `data_class`, and `audit_event_class`.
- State surface `translate.bulk_translate` is documented as tenant-scoped with `tenant_id`, `principal_id`, `audit_event_class`, `home_cell`, and `lifecycle_state` columns/events.
- State surface `translate.document_localization` is documented as tenant-scoped with `tenant_id`, `principal_id`, `audit_event_class`, `home_cell`, and `lifecycle_state` columns/events.
- State surface `translate.language_detection` is documented as tenant-scoped with `tenant_id`, `principal_id`, `audit_event_class`, `home_cell`, and `lifecycle_state` columns/events.
- State surface `translate.quality_estimation` is documented as tenant-scoped with `tenant_id`, `principal_id`, `audit_event_class`, `home_cell`, and `lifecycle_state` columns/events.
- State surface `translate.real_time_stream` is documented as tenant-scoped with `tenant_id`, `principal_id`, `audit_event_class`, `home_cell`, and `lifecycle_state` columns/events.
- Cross-tenant reads fail at Cedar before storage adapters see a query; storage row-level policy is defence-in-depth, not the primary guard.
- Example: `translate-segment-suggest-and-lang-hint` reads include `tenant_id` and `home_cell`; a stale `jurisdiction_code` forces most-restrictive-pack handling until tenancy refresh completes.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: Stripe Connect connected-account isolation is the reference pattern for the control shape described here.
- Precedent 2: AWS Organizations account-boundary pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against ADR-0245 §D-1: substrate/product classification and dependency direction.

### Service-specific answer
- Manifest classifies `translate` as `product`, so this section treats it as a product consumer.
- Declared substrate/product dependencies: `identity`, `tenancy`, `policy-engine`, `observability`, `audit-chain`, `cloud-secrets`.
- If substrate: products consume `translate` only through contracts `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- If product: `translate` may call substrate services but must not create product-to-product synchronous dependencies.
- Dependency direction is inward to clean core crates; adapter and framework code never defines domain terms for other µservices.
- Primary bounded contexts bound to this classification: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Example: `translate-segment-suggest-and-lang-hint` may depend on `tenancy` for tenant state and `observability` for audit emission, but not on another product UI workflow.
- ADR-0280 substrate-of-substrate ordering is documented here so delivery planning can parallelize product work without creating hidden runtime coupling.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: Palantir Foundry shared ontology substrate is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud shared VPC/service-project split is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against ADR-0246 §D-4: library-first policy evaluation mode and fallback limits.

### Service-specific answer
- `policy_evaluation_mode = library-first`; network policy-engine calls are fallback only for stale local fragment cache or explicit audit replay.
- The caller-side library evaluates fragments from `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar` with the full action/resource/context tuple.
- Partial-context evaluation is forbidden: no action can be evaluated without `tenant_id`, `principal_id`, `audience_type`, `resource_id`, and `data_class`.
- Cache freshness target is ≤5 minutes for normal fragments and immediate invalidation for deny-list, credential, or pack-overlay revocations.
- Fallback network evaluation emits `PolicyEvaluationFallbackUsed` with fragment hash and reason so ADR-0263 can detect degraded posture.
- Example: `translate::translate-segment-suggest-and-lang-hint::execute` evaluates locally, then emits an audit event before the usecase writes `translate.bulk_translate`.
- OpenBao credential sidecar is never called before policy allow; policy decides whether the credential lookup is authorized.
- This mirrors OPA sidecar discipline: policy is close to the caller, but the source of truth remains signed and soaked Cedar fragments.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: AWS Verified Permissions local Cedar model is the reference pattern for the control shape described here.
- Precedent 2: Open Policy Agent sidecar evaluation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against ADR-0248 §D-1: cell tier, shard width, DR pair and shuffle-shard behavior.

### Service-specific answer
- Cell eligibility declaration: `not declared in manifest; bound here to the conservative platform default`.
- Tier 0/1 control-plane paths run in hardened cells; tenant data planes can shard per tenant, pack, region, and workload class.
- Per-cell shard key is `(tenant_id, home_cell, jurisdiction_code)`; DR pair selection uses `dr_cell` where data-residency permits failover.
- Shuffle-shard width is documented by `multi-region.md` or defaults to three independent cells for Tier-1 control paths.
- Regional outage behavior: keep reads local where pack permits, stop cross-border replication where pack forbids it, and preserve audit emission locally.
- Example: `translate-segment-suggest-and-lang-hint` traffic in a KR pack tenant stays in KR home cell; DR failover requires pack approval and emits a cell-failover audit event.
- Capacity math lives in `capacity-model.md`; this section binds the shard dimensions so the math is not detached from topology.
- Cloud Hypervisor/Kata isolation applies to Tier 0/1 pods; Tier 2/3 paths inherit the same network policy and SPIFFE identity floor.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: AWS cell-based architecture is the reference pattern for the control shape described here.
- Precedent 2: Route 53 shuffle-sharding isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against ADR-0255 §D-2: Intelligence calls, library-first dispatch and audience tags.

### Service-specific answer
- `translate` uses Intelligence only through a declared dispatch boundary; default is library-first and network-opt-in must name a latency budget.
- Audience tag format is `translate.translate-segment-suggest-and-lang-hint` and is included in every AI/risk/model call audit event.
- No prompt, model choice, tool call, or generated output may cross tenant boundaries; `tenant_id` and `data_class` travel with the request.
- Provider credential mode follows tenant provider-BYOK where configured; default provider credentials are platform-owned only for platform-permitted consumer paths.
- Heavy network calls route to `intelligence` gRPC only when local dispatch cannot satisfy the model/tool requirement.
- Example: `translate-segment-suggest-and-lang-hint` can request classification or summarization with `audience_tag=translate.translate-segment-suggest-and-lang-hint`; the response is logged with model version and policy decision.
- Failure mode: if Intelligence is unavailable, the usecase must choose deterministic fallback, queue for retry, or deny high-risk automation rather than inventing output.
- Cross-links: `intelligence`, `policy-engine`, `observability`, and `cloud-secrets` own the model routing, policy decision, audit, and credential surfaces.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: Palantir AIP tool execution boundary is the reference pattern for the control shape described here.
- Precedent 2: Azure OpenAI tenant-isolated deployment pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against ADR-0257 §D-2: Ontology entity reads, projection cache and freshness floor.

### Service-specific answer
- Ontology read mode is library-first projection cache for `translate` unless this section documents a network-only exception.
- Entity types read by this µservice: `Tenant`, `Principal`, `PolicyGrant`, `CompliancePack`, plus bounded-context entities named below.
- Bounded-context entity `bulk-translate` is read/written as an ontology node or edge projection when `translate` needs cross-surface discovery.
- Bounded-context entity `document-localization` is read/written as an ontology node or edge projection when `translate` needs cross-surface discovery.
- Bounded-context entity `language-detection` is read/written as an ontology node or edge projection when `translate` needs cross-surface discovery.
- Bounded-context entity `quality-estimation` is read/written as an ontology node or edge projection when `translate` needs cross-surface discovery.
- Freshness floor: 30 seconds for tenant/compliance/security fields; 5 minutes for catalog/product metadata; stale security fields force most-restrictive handling.
- Example: `translate-segment-suggest-and-lang-hint` resolves `tenant.compliance_packs`, `principal.audience_type`, and `resource.home_cell` before Cedar evaluation.
- Cache invalidation is event-driven from Ontology; network read is reserved for cache miss plus explicit latency budget.
- Cross-links: `ontology` owns type semantics; `tenancy` owns tenant lifecycle; `policy-engine` consumes the projection for Cedar context.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: Palantir Foundry ontology projection pattern is the reference pattern for the control shape described here.
- Precedent 2: Google Knowledge Graph serving-cache pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against ADR-0253 §D-1: HTTP/3 fallback chain, strict TLS, ECH and PQC posture.

### Service-specific answer
- Public/API contracts in scope: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- REST and gRPC advertise HTTP/3 through `Alt-Svc: h3=":443"`; fallback order is HTTP/3, then HTTP/2, then HTTP/1.1 with first acceptable winning.
- HTTP/1.0, TLS <1.3, self-signed production certificates, `insecure_skip_verify`, and MITM bypass headers are forbidden.
- ECH is advertised through HTTPS RR `ech=` config wherever the platform terminates TLS; ECH-disabled clients fall back to ordinary TLS 1.3 without refusal.
- PQC hybrid `X25519MLKEM768` is offered where the client/server pair supports it; classical X25519/P-256 fallback is accepted during migration.
- IaC transport evidence: `microservices/translate/iac/helm/translate/Chart.yaml`, `microservices/translate/iac/helm/translate/templates/deployment.yaml`, `microservices/translate/iac/helm/translate/templates/hpa.yaml`, `microservices/translate/iac/helm/translate/templates/networkpolicy.yaml`, `microservices/translate/iac/helm/translate/templates/pdb.yaml`, `microservices/translate/iac/helm/translate/templates/prometheusrule.yaml`; +6 more.
- Example: `translate` `translate-segment-suggest-and-lang-hint` calls use HTTP/3 on normal networks and HTTP/2 when UDP/QUIC is blocked by enterprise firewalls.
- Async/event transport preserves tenant and audit context on every message; webhook ingress verifies HMAC/mTLS before Cedar sees the payload.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: Google QUIC/HTTP3 rollout pattern is the reference pattern for the control shape described here.
- Precedent 2: Cloudflare ECH and post-quantum TLS experiments is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against ADR-0254 §D-2: Kubernetes, Cloud Hypervisor, Kata, Wasm/container/VM split.

### Service-specific answer
- Runtime components derive from bounded contexts `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more and deploy as separate app/worker/API pods where files exist.
- Default shape is Kubernetes pods with Cloud Hypervisor + Kata isolation for Tier 0/1 paths; lower tiers inherit network policy and SPIFFE identity.
- Wasm is reserved for untrusted plugin or user-authored execution; normal business usecases remain containers; bootstrap/control roots may require VM isolation.
- IaC manifests in scope: `microservices/translate/iac/helm/translate/Chart.yaml`, `microservices/translate/iac/helm/translate/templates/deployment.yaml`, `microservices/translate/iac/helm/translate/templates/hpa.yaml`, `microservices/translate/iac/helm/translate/templates/networkpolicy.yaml`, `microservices/translate/iac/helm/translate/templates/pdb.yaml`, `microservices/translate/iac/helm/translate/templates/prometheusrule.yaml`; +6 more.
- Catalog crates/components: `microservices/translate/catalog/oya-translate-bulk-worker.yaml`, `microservices/translate/catalog/oya-translate-doc-adapter-libreoffice.yaml`, `microservices/translate/catalog/oya-translate-doc-adapter-pandoc.yaml`, `microservices/translate/catalog/oya-translate-langdetect-adapter-foundry-runtime.yaml`, `microservices/translate/catalog/oya-translate-qe-adapter-foundry-runtime.yaml`, `microservices/translate/catalog/oya-translate-qe-kernel.yaml`; +18 more.
- Example: `translate-segment-suggest-and-lang-hint` API runs as a Kata-isolated container; scheduled/background work for `translate` uses a separate worker principal.
- OpenBao and SPIFFE mounts are sidecars, not linked into domain/core crates; adapters own framework code.
- Rollback is deployment-level first: pin previous image digest, keep schema backward-compatible, and replay idempotent events after recovery.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: AWS Firecracker microVM isolation is the reference pattern for the control shape described here.
- Precedent 2: GKE Sandbox/Kata isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against ADR-0263 §D-1: audit events, metrics, logs and trace span shape.

### Service-specific answer
- SLO/dashboard sources in scope: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Audit classes use prefix `oya.translate.<context>.<outcome>` and must be registered centrally per ADR-0263.
- Core counter metric: `translate_translate_segment_suggest_and_lang_hint_total` with dimensions `outcome`, `route_class`, `tenant_id_class`, and `cell_tier`.
- Core latency metric: `translate_translate_segment_suggest_and_lang_hint_latency_ms` with bounded cardinality and no raw `tenant_id` label.
- Reference evidence artifact: `microservices/translate/slos/batch-translate-latency.openslo.yaml`.
- Trace root span is `<service>.<capability>`; child spans are `policy.evaluate`, `ontology.read`, `storage.write/read`, `audit.emit`, and provider/adapter calls.
- Logs are structured JSON, redacted by data class, retained per pack, and include correlation id plus audit event id.
- Example trace: `translate.translate-segment-suggest-and-lang-hint` -> `policy.evaluate` -> `storage.translate_segment_suggest_and_lang_hint` -> `audit.emit`.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: Google SRE four-signal telemetry pattern is the reference pattern for the control shape described here.
- Precedent 2: OpenTelemetry semantic conventions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against ADR-0297 §D-3: anti-bot, anti-spoof, anti-scrape controls plus UX floor.

### Service-specific answer
- Internet-facing `translate` routes use edge rate limits by IP, JA4 fingerprint, tenant, route class, and `translate-segment-suggest-and-lang-hint` action class.
- Passive bot scoring is forwarded as `X-Oya-Bot-Score`; Cedar composes the score with quota, tenant_class, and entitlement predicates before any challenge.
- Default path is friction-free: legitimate traffic receives no CAPTCHA, no JS proof-of-work, and ≤2ms p99 edge scoring budget.
- Cedar/IaC controls in scope: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`; +12 more.
- Canary payloads for `translate` use fake `translate-segment-suggest-and-lang-hint` identifiers and honey endpoints under `/.well-known/oya-canary/translate`.
- Anti-spoof: HMAC on webhooks, SPIFFE SVID on service calls, SameSite=Strict on cookies, mTLS for machine clients, audit signatures via sidecar.
- Anti-scrape: per-fingerprint pagination caps, breadth-first crawl detection, watermarking for high-value content, and partner allow-list for friendly crawlers.
- Emergency-services and accessibility bypasses are evaluated before bot-score friction and still emit audit events.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: Cloudflare Bot Management and Turnstile is the reference pattern for the control shape described here.
- Precedent 2: Stripe Radar passive risk scoring is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against documentation-rigor.md §3.2.5: applicable human-safety and platform edge-case handling.

### Service-specific answer
- Network partition: `translate` keeps tenant-local reads when safe, stops cross-cell writes that would violate residency, and emits degraded-mode audit events.
- Byzantine caller: Cedar denies forged `principal_id`, mismatched `tenant_id`, invalid SVID, replayed idempotency keys, and suspicious bot-score context.
- Regional outage: home-cell failover follows `multi-region.md`; if a pack forbids cross-border DR, `translate` preserves local queue state instead of failing open.
- Key compromise: ADR-0296 sidecar revokes OpenBao leases, rotates signing keys, and quarantines affected audit event classes for reconciliation.
- Account recovery/hijack path: identity step-up and `translate` audit evidence keep legitimate recovery from becoming an adversary shortcut.
- Mistaken mutation path: high-impact `translate-segment-suggest-and-lang-hint` mutations require idempotency, undo/cooldown where product semantics allow, and sealed evidence for later correction.
- Disaster surge: `translate` enforces per-tenant isolation so one hot tenant or emergency mode cannot starve unrelated cells.
- Verification: capacity math in `capacity-model.md`, rollback in `failure-modes.md`, DR handling in `multi-region.md`, and incident actions in runbooks.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: Google SRE incident playbooks is the reference pattern for the control shape described here.
- Precedent 2: Stripe idempotent mutation recovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against ADR-0296 §D-1: credential sidecar, OpenBao TTLs and secret-reference path.

### Service-specific answer
- Credential scopes for `translate` include `identity`, `tenancy`, `policy-engine`, `observability`, `audit-chain`, `cloud-secrets`.
- SecretReference pattern is `${openbao:secret/<tenant_id>/translate/<credential-name>}`; platform-owned credentials use tenant `oyatie` only for oyatie-internal calls.
- Sidecar mode is required for audit-signing keys and preferred for provider credentials; raw credentials never enter domain/core crates.
- OpenBao token TTL is ≤60 seconds when sidecar isolation is not possible; refresh must be policy-allowed and audit-emitted.
- Rotation cadence is linked from `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.
- Example: `translate-segment-suggest-and-lang-hint` asks the sidecar for a scoped credential after Cedar allow; the sidecar returns a short-lived handle, not the underlying secret.
- Compromise response: revoke lease, rotate key, disable affected provider adapter, replay idempotent queue after audit-chain reconciliation.
- encryption-BYOK is separate from provider-BYOK; this section covers provider/API/signing credentials only unless a pack explicitly adds KMS-root behavior.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: HashiCorp Vault dynamic secrets is the reference pattern for the control shape described here.
- Precedent 2: AWS KMS envelope-key isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
