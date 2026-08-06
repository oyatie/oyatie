# cloud-iac

_This file was created by the Wave-3-C anchor-sweep. Expand all scaffold sections during content-pass review._

---



## §principals
This anchor is closed for `cloud-iac` against ADR-0242 §D-1: principal roster and tenant-scoped caller model.

### Service-specific answer
- Platform principal `oyatie.cloud-iac.runtime` owns normal `iac-apply` execution and never borrows a tenant principal.
- Platform principal `oyatie.cloud-iac.worker` owns async jobs, retry queues, and backfill replay listed in `iac/backfill-replay.md` when present.
- Platform principal `oyatie.cloud-iac.auditor` has read-only evidence access through auditor Cedar fragments, not direct database credentials.
- Platform principal `oyatie.cloud-iac.ci` is limited to synthetic tenants and non-production cells by the CI-scope Cedar fragment.
- Tenant callers are represented as `<tenant>.cloud-iac.caller` and must provide `tenant_id`, `principal_id`, `audience_type`, and workload SVID context.
- Cross-µservice callers expected by dependency graph: `identity`, `tenancy`, `policy-engine`, `observability`, `audit-chain`, `cloud-secrets`.
- Example: a tenant principal invoking `iac-apply` is evaluated as `<tenant>.cloud-iac.iac-apply` before any `cloud-iac` state mutation.
- Forbidden: caller-supplied `oyatie.*` principals; ADR-0242 treats `oyatie` as its own tenant, not a namespace tenants can impersonate.

### Concrete inventory used
- Service: `cloud-iac`; owner `axis-cloud-iac`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-iac`.
- Capability records cited: `iac/capabilities/iac-apply.yaml`, `iac/capabilities/iac-render.yaml`, `iac/capabilities/iac-rollback.yaml`.
- API surfaces cited: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar/policy artifacts cited: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar binding: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- State/event binding: `cloud_iac.cloud_iac`.
- Capability binding: `iac-apply`, `iac-render`, `iac-rollback`.
- SLO binding: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`.
- Runbook binding: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-iac`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-iac`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-iac` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-iac` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-iac`.

### Hyperscaler precedents
- Precedent 1: AWS IAM service-linked roles is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud service agents is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-iac` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya-ci-required` evidence should include marker absence, section-count policy, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §cell-provisioning

`cloud-iac` owns cell provisioning, lifecycle, registry, and infrastructure
capacity after Wave 15L. The former `microservices/cell/` service boundary is
retired because cells are topology, not an RPC service. OpenTofu state is the
cell registry: if a cell is not declared, applied, and tracked in OpenTofu state,
it does not exist for tenancy placement or api-gateway routing.

Cell registry model:
- `cell_id`: stable topology identifier, never derived from a tenant id.
- `pack`: compliance and residency pack the cell may host.
- `region`: deployment region or sovereign-region label.
- `state`: `requested`, `provisioning`, `ready`, `draining`, `retired`.
- `accepts_new_tenants`: false during warmup, drain, quarantine, and retirement.
- `capacity_envelope`: tenants, pods, Postgres connections, object-storage
  prefix bytes, and control-plane headroom.
- `assignment_epoch_floor`: lowest tenant assignment epoch allowed to route here.
- `tofu_state_ref`: immutable pointer to the applied OpenTofu module state.

OpenTofu ownership:
- A cell is an OpenTofu module instance, not an ad hoc Kubernetes namespace.
- Modules create the namespace, network policy, SPIFFE trust bundle, Postgres
  logical schema or database boundary, object-storage prefix, observability
  labels, and secret-binding references.
- Helm and Kustomize artifacts may render inside the module, but Terraform,
  Pulumi, CloudFormation, and ARM remain non-authoritative for new work.
- `cloud-iac` publishes a read API and event stream for cell topology; the
  registry source of truth remains OpenTofu state, not a separate cell database.

Lifecycle operations:
1. `request-cell`: capacity or pack onboarding asks for a new cell module.
2. `provision-cell`: OpenTofu applies infrastructure and writes state.
3. `mark-ready`: health, policy, and audit prerequisites pass; the cell can
   accept new tenants.
4. `drain-cell`: `accepts_new_tenants=false`, tenancy migration planning begins,
   and api-gateway stops using the cell for new primary assignments.
5. `retire-cell`: after zero active assignments and sealed audit evidence, the
   module destroys mutable resources or moves them to retention.

Capacity ownership:
- `cloud-iac` owns planned capacity and scale-out thresholds from infrastructure
  state.
- `observability` owns live utilization, SLO burn, and isolation evidence.
- New-cell triggers require both planned capacity pressure and live evidence.
- The old cell capacity model migrates here for infrastructure math: per-cell
  tenant envelopes, warm-pool depth, Postgres connection ceilings, and object
  storage prefix ceilings are cloud-iac inputs.

Cross-service contract:
- `tenancy` consumes ready candidate cells and records the selected assignment.
- `observability` consumes lifecycle events and attaches cell labels to SLOs.
- `api-gateway` consumes routeable cells from tenant principal context; it does
  not query OpenTofu on the request hot path.
- `audit-chain` seals lifecycle transitions and assignment-facing registry
  evidence.

Verification:
- Every `ready` cell must have an OpenTofu state ref, observability label set,
  audit-chain signing context, SPIFFE trust binding, and data-residency pack.
- `drain` and `retire` must prove zero active tenant assignments before
  destructive changes.
- ADR-0333 is the retirement decision record for why this ownership sits here
  instead of in a standalone cell service.

## §cedar-gates
This anchor is closed for `cloud-iac` against ADR-0243 §D-2: Cedar fragment roster, default-deny and action taxonomy.

### Service-specific answer
- Default-deny is represented by the first matching Cedar fragment in `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- Action namespace uses `cloud-iac::<bounded_context>::<verb>`; the first protected action is `cloud-iac::iac-apply::execute`.
- Every evaluation context carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, `bot_score`, and `provider_credential_mode`.
- Mutating actions require an audit event class before the usecase layer runs; read actions require a purpose and data-class declaration.
- Auditor reads are time-boxed and read-only; CI principals are sandbox-only; emergency bypass policies never skip audit emission.
- Concrete fragments in scope: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- Example: `cloud-iac::iac-apply::execute` denies if `resource.tenant_id != principal.tenant_id` or if the compliance pack adds a stricter overlay.
- Fragment publish observes ADR-0294 soak before activation; rollback reverts the fragment pointer, not the business-state rows.

### Concrete inventory used
- Service: `cloud-iac`; owner `axis-cloud-iac`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-iac`.
- Capability records cited: `iac/capabilities/iac-apply.yaml`, `iac/capabilities/iac-render.yaml`, `iac/capabilities/iac-rollback.yaml`.
- API surfaces cited: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar/policy artifacts cited: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar binding: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- State/event binding: `cloud_iac.cloud_iac`.
- Capability binding: `iac-apply`, `iac-render`, `iac-rollback`.
- SLO binding: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`.
- Runbook binding: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-iac`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-iac`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-iac` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-iac` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-iac`.

### Hyperscaler precedents
- Precedent 1: AWS Verified Permissions Cedar policy evaluation is the reference pattern for the control shape described here.
- Precedent 2: Google Zanzibar relationship checks is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-iac` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya-ci-required` evidence should include marker absence, section-count policy, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §tenant-scoping
This anchor is closed for `cloud-iac` against ADR-0244 §D-3: tenant_id, audience_type, provider_credential_mode and row/event boundaries.

### Service-specific answer
- Audience type for this service is `B2C_CONSUMER + B2B_TENANT` and is copied into audit events plus any public contract response that exposes policy posture.
- Provider credential mode is `tenant-provider-byok where external providers exist; platform-default credentials otherwise`; provider-BYOK and encryption-BYOK stay separate per ADR-0255 §D-4 / ADR-0251 §D-10.
- Required fields on mutable rows/events: `tenant_id`, `principal_id`, `caller_tenant_id` when delegated, `home_cell`, `jurisdiction_code`, `data_class`, and `audit_event_class`.
- State surface `cloud_iac.cloud_iac` is documented as tenant-scoped with `tenant_id`, `principal_id`, `audit_event_class`, `home_cell`, and `lifecycle_state` columns/events.
- Planned table/event surface `cloud_iac.iac_apply_2` inherits the same `tenant_id` and audit fields until a migration file supersedes this scaffold name.
- Planned table/event surface `cloud_iac.iac_apply_3` inherits the same `tenant_id` and audit fields until a migration file supersedes this scaffold name.
- Planned table/event surface `cloud_iac.iac_apply_4` inherits the same `tenant_id` and audit fields until a migration file supersedes this scaffold name.
- Planned table/event surface `cloud_iac.iac_apply_5` inherits the same `tenant_id` and audit fields until a migration file supersedes this scaffold name.
- Cross-tenant reads fail at Cedar before storage adapters see a query; storage row-level policy is defence-in-depth, not the primary guard.
- Example: `iac-apply` reads include `tenant_id` and `home_cell`; a stale `jurisdiction_code` forces most-restrictive-pack handling until tenancy refresh completes.

### Concrete inventory used
- Service: `cloud-iac`; owner `axis-cloud-iac`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-iac`.
- Capability records cited: `iac/capabilities/iac-apply.yaml`, `iac/capabilities/iac-render.yaml`, `iac/capabilities/iac-rollback.yaml`.
- API surfaces cited: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar/policy artifacts cited: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar binding: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- State/event binding: `cloud_iac.cloud_iac`.
- Capability binding: `iac-apply`, `iac-render`, `iac-rollback`.
- SLO binding: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`.
- Runbook binding: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-iac`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-iac`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-iac` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-iac` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-iac`.

### Hyperscaler precedents
- Precedent 1: Stripe connected-account isolation is the reference pattern for the control shape described here.
- Precedent 2: AWS Organizations account-boundary pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-iac` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya-ci-required` evidence should include marker absence, section-count policy, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §substrate-product-binding
This anchor is closed for `cloud-iac` against ADR-0245 §D-1: substrate/product classification and dependency direction.

### Service-specific answer
- Manifest classifies `cloud-iac` as `product`, so this section treats it as a product consumer.
- Declared substrate/product dependencies: `identity`, `tenancy`, `policy-engine`, `observability`, `audit-chain`, `cloud-secrets`.
- If substrate: products consume `cloud-iac` only through contracts `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- If product: `cloud-iac` may call substrate services but must not create product-to-product synchronous dependencies.
- Dependency direction is inward to clean core crates; adapter and framework code never defines domain terms for other µservices.
- Primary bounded contexts bound to this classification: `cloud-iac`.
- Example: `iac-apply` may depend on `tenancy` for tenant state and `observability` for audit emission, but not on another product UI workflow.
- ADR-0280 substrate-of-substrate ordering is documented here so delivery planning can parallelize product work without creating hidden runtime coupling.

### Concrete inventory used
- Service: `cloud-iac`; owner `axis-cloud-iac`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-iac`.
- Capability records cited: `iac/capabilities/iac-apply.yaml`, `iac/capabilities/iac-render.yaml`, `iac/capabilities/iac-rollback.yaml`.
- API surfaces cited: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar/policy artifacts cited: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar binding: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- State/event binding: `cloud_iac.cloud_iac`.
- Capability binding: `iac-apply`, `iac-render`, `iac-rollback`.
- SLO binding: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`.
- Runbook binding: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-iac`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-iac`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-iac` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-iac` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-iac`.

### Hyperscaler precedents
- Precedent 1: Palantir Cloud governance shared ontology substrate is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud shared VPC/service-project split is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-iac` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya-ci-required` evidence should include marker absence, section-count policy, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §policy-evaluation
This anchor is closed for `cloud-iac` against ADR-0246 §D-4: library-first policy evaluation mode and fallback limits.

### Service-specific answer
- `policy_evaluation_mode = library-first`; network policy-engine calls are fallback only for stale local fragment cache or explicit audit replay.
- The caller-side library evaluates fragments from `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar` with the full action/resource/context tuple.
- Partial-context evaluation is forbidden: no action can be evaluated without `tenant_id`, `principal_id`, `audience_type`, `resource_id`, and `data_class`.
- Cache freshness target is ≤5 minutes for normal fragments and immediate invalidation for deny-list, credential, or pack-overlay revocations.
- Fallback network evaluation emits `PolicyEvaluationFallbackUsed` with fragment hash and reason so ADR-0263 can detect degraded posture.
- Example: `cloud-iac::iac-apply::execute` evaluates locally, then emits an audit event before the usecase writes `cloud_iac.cloud_iac`.
- OpenBao credential sidecar is never called before policy allow; policy decides whether the credential lookup is authorized.
- This mirrors OPA sidecar discipline: policy is close to the caller, but the source of truth remains signed and soaked Cedar fragments.

### Concrete inventory used
- Service: `cloud-iac`; owner `axis-cloud-iac`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-iac`.
- Capability records cited: `iac/capabilities/iac-apply.yaml`, `iac/capabilities/iac-render.yaml`, `iac/capabilities/iac-rollback.yaml`.
- API surfaces cited: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar/policy artifacts cited: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar binding: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- State/event binding: `cloud_iac.cloud_iac`.
- Capability binding: `iac-apply`, `iac-render`, `iac-rollback`.
- SLO binding: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`.
- Runbook binding: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-iac`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-iac`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-iac` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-iac` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-iac`.

### Hyperscaler precedents
- Precedent 1: AWS Verified Permissions local Cedar model is the reference pattern for the control shape described here.
- Precedent 2: Open Policy Agent sidecar evaluation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-iac` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya-ci-required` evidence should include marker absence, section-count policy, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §cell-eligibility
This anchor is closed for `cloud-iac` against ADR-0248 §D-1: cell tier, shard width, DR pair and shuffle-shard behavior.

### Service-specific answer
- Cell eligibility declaration: `not declared in manifest; bound here to the conservative platform default`.
- Tier 0/1 control-plane paths run in hardened cells; tenant data planes can shard per tenant, pack, region, and workload class.
- Per-cell shard key is `(tenant_id, home_cell, jurisdiction_code)`; DR pair selection uses `dr_cell` where data-residency permits failover.
- Shuffle-shard width is documented by `multi-region.md` or defaults to three independent cells for Tier-1 control paths.
- Regional outage behavior: keep reads local where pack permits, stop cross-border replication where pack forbids it, and preserve audit emission locally.
- Example: `iac-apply` traffic in a KR pack tenant stays in KR home cell; DR failover requires pack approval and emits a cell-failover audit event.
- Capacity math lives in `capacity-model.md`; this section binds the shard dimensions so the math is not detached from topology.
- Cloud Hypervisor/Kata isolation applies to Tier 0/1 pods; Tier 2/3 paths inherit the same network policy and SPIFFE identity floor.

### Concrete inventory used
- Service: `cloud-iac`; owner `axis-cloud-iac`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-iac`.
- Capability records cited: `iac/capabilities/iac-apply.yaml`, `iac/capabilities/iac-render.yaml`, `iac/capabilities/iac-rollback.yaml`.
- API surfaces cited: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar/policy artifacts cited: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar binding: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- State/event binding: `cloud_iac.cloud_iac`.
- Capability binding: `iac-apply`, `iac-render`, `iac-rollback`.
- SLO binding: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`.
- Runbook binding: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-iac`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-iac`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-iac` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-iac` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-iac`.

### Hyperscaler precedents
- Precedent 1: AWS cell-based architecture is the reference pattern for the control shape described here.
- Precedent 2: Route 53 shuffle-sharding isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-iac` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya-ci-required` evidence should include marker absence, section-count policy, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §transport
This anchor is closed for `cloud-iac` against ADR-0253 §D-1: HTTP/3 fallback chain, strict TLS, ECH and PQC posture.

### Service-specific answer
- Public/API contracts in scope: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- REST and gRPC advertise HTTP/3 through `Alt-Svc: h3=":443"`; fallback order is HTTP/3, then HTTP/2, then HTTP/1.1 with first acceptable winning.
- HTTP/1.0, TLS <1.3, self-signed production certificates, `insecure_skip_verify`, and MITM bypass headers are forbidden.
- ECH is advertised through HTTPS RR `ech=` config wherever the platform terminates TLS; ECH-disabled clients fall back to ordinary TLS 1.3 without refusal.
- PQC hybrid `X25519MLKEM768` is offered where the client/server pair supports it; classical X25519/P-256 fallback is accepted during migration.
- IaC transport evidence: `iac/iac/helm/argocd/Chart.yaml`, `iac/iac/helm/argocd/templates/.gitkeep`, `iac/iac/helm/argocd/values.yaml`, `iac/iac/helm/flagger/Chart.yaml`, `iac/iac/helm/helm-controller/Chart.yaml`, `iac/iac/helm/helm-controller/templates/deployment.yaml`; +6 more.
- Example: `cloud-iac` `iac-apply` calls use HTTP/3 on normal networks and HTTP/2 when UDP/QUIC is blocked by enterprise firewalls.
- Async/event transport preserves tenant and audit context on every message; webhook ingress verifies HMAC/mTLS before Cedar sees the payload.

### Concrete inventory used
- Service: `cloud-iac`; owner `axis-cloud-iac`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-iac`.
- Capability records cited: `iac/capabilities/iac-apply.yaml`, `iac/capabilities/iac-render.yaml`, `iac/capabilities/iac-rollback.yaml`.
- API surfaces cited: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar/policy artifacts cited: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar binding: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- State/event binding: `cloud_iac.cloud_iac`.
- Capability binding: `iac-apply`, `iac-render`, `iac-rollback`.
- SLO binding: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`.
- Runbook binding: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-iac`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-iac`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-iac` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-iac` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-iac`.

### Hyperscaler precedents
- Precedent 1: Google QUIC/HTTP3 rollout pattern is the reference pattern for the control shape described here.
- Precedent 2: Cloudflare ECH and post-quantum TLS experiments is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-iac` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya-ci-required` evidence should include marker absence, section-count policy, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §deployment-shape
This anchor is closed for `cloud-iac` against ADR-0254 §D-2: Kubernetes, Cloud Hypervisor, Kata, Wasm/container/VM split.

### Service-specific answer
- Runtime components derive from bounded contexts `cloud-iac` and deploy as separate app/worker/API pods where files exist.
- Default shape is Kubernetes pods with Cloud Hypervisor + Kata isolation for Tier 0/1 paths; lower tiers inherit network policy and SPIFFE identity.
- Wasm is reserved for untrusted plugin or user-authored execution; normal business usecases remain containers; bootstrap/control roots may require VM isolation.
- IaC manifests in scope: `iac/iac/helm/argocd/Chart.yaml`, `iac/iac/helm/argocd/templates/.gitkeep`, `iac/iac/helm/argocd/values.yaml`, `iac/iac/helm/flagger/Chart.yaml`, `iac/iac/helm/helm-controller/Chart.yaml`, `iac/iac/helm/helm-controller/templates/deployment.yaml`; +6 more.
- Catalog crates/components: `iac/catalog/oya-cloud-iac-iac-applier-adapter-argocd.yaml`, `iac/catalog/oya-cloud-iac-iac-applier-adapter.yaml`, `iac/catalog/oya-cloud-iac-iac-applier-api.yaml`, `iac/catalog/oya-cloud-iac-iac-applier-app.yaml`, `iac/catalog/oya-cloud-iac-iac-applier-domain.yaml`, `iac/catalog/oya-cloud-iac-iac-applier-kernel.yaml`; +18 more.
- Example: `iac-apply` API runs as a Kata-isolated container; scheduled/background work for `cloud-iac` uses a separate worker principal.
- OpenBao and SPIFFE mounts are sidecars, not linked into domain/core crates; adapters own framework code.
- Rollback is deployment-level first: pin previous image digest, keep schema backward-compatible, and replay idempotent events after recovery.

### Concrete inventory used
- Service: `cloud-iac`; owner `axis-cloud-iac`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-iac`.
- Capability records cited: `iac/capabilities/iac-apply.yaml`, `iac/capabilities/iac-render.yaml`, `iac/capabilities/iac-rollback.yaml`.
- API surfaces cited: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar/policy artifacts cited: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar binding: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- State/event binding: `cloud_iac.cloud_iac`.
- Capability binding: `iac-apply`, `iac-render`, `iac-rollback`.
- SLO binding: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`.
- Runbook binding: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-iac`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-iac`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-iac` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-iac` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-iac`.

### Hyperscaler precedents
- Precedent 1: AWS Firecracker microVM isolation is the reference pattern for the control shape described here.
- Precedent 2: GKE Sandbox/Kata isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-iac` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya-ci-required` evidence should include marker absence, section-count policy, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §observability
This anchor is closed for `cloud-iac` against ADR-0263 §D-1: audit events, metrics, logs and trace span shape.

### Service-specific answer
- SLO/dashboard sources in scope: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`; +3 more.
- Audit classes use prefix `oya.cloud.iac.<context>.<outcome>` and must be registered centrally per ADR-0263.
- Core counter metric: `cloud_iac_iac_apply_total` with dimensions `outcome`, `route_class`, `tenant_id_class`, and `cell_tier`.
- Core latency metric: `cloud_iac_iac_apply_latency_ms` with bounded cardinality and no raw `tenant_id` label.
- Reference evidence artifact: `iac/slos/helm-chart-lint-correctness.openslo.yaml`.
- Trace root span is `<service>.<capability>`; child spans are `policy.evaluate`, `ontology.read`, `storage.write/read`, `audit.emit`, and provider/adapter calls.
- Logs are structured JSON, redacted by data class, retained per pack, and include correlation id plus audit event id.
- Example trace: `cloud-iac.iac-apply` -> `policy.evaluate` -> `storage.iac_apply` -> `audit.emit`.

### Concrete inventory used
- Service: `cloud-iac`; owner `axis-cloud-iac`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-iac`.
- Capability records cited: `iac/capabilities/iac-apply.yaml`, `iac/capabilities/iac-render.yaml`, `iac/capabilities/iac-rollback.yaml`.
- API surfaces cited: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar/policy artifacts cited: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar binding: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- State/event binding: `cloud_iac.cloud_iac`.
- Capability binding: `iac-apply`, `iac-render`, `iac-rollback`.
- SLO binding: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`.
- Runbook binding: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-iac`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-iac`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-iac` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-iac` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-iac`.

### Hyperscaler precedents
- Precedent 1: Google SRE four primary service signals is the reference pattern for the control shape described here.
- Precedent 2: OpenTelemetry semantic conventions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-iac` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya-ci-required` evidence should include marker absence, section-count policy, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §abuse-defence
This anchor is closed for `cloud-iac` against ADR-0297 §D-3: anti-bot, anti-spoof, anti-scrape controls plus UX floor.

### Service-specific answer
- Internet-facing `cloud-iac` routes use edge rate limits by IP, JA4 fingerprint, tenant, route class, and `iac-apply` action class.
- Passive bot scoring is forwarded as `X-Oya-Bot-Score`; Cedar composes the score with quota and tenant_class before any challenge.
- Default path is friction-free: legitimate traffic receives no CAPTCHA, no JS proof-of-work, and ≤2ms p99 edge scoring budget.
- Cedar/IaC controls in scope: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`; +12 more.
- Canary payloads for `cloud-iac` use fake `iac-apply` identifiers and honey endpoints under `/.well-known/oya-canary/cloud-iac`.
- Anti-spoof: HMAC on webhooks, SPIFFE SVID on service calls, SameSite=Strict on cookies, mTLS for machine clients, audit signatures via sidecar.
- Anti-scrape: per-fingerprint pagination caps, breadth-first crawl detection, watermarking for high-value content, and partner allow-list for friendly crawlers.
- Emergency-services and accessibility bypasses are evaluated before bot-score friction and still emit audit events.

### Concrete inventory used
- Service: `cloud-iac`; owner `axis-cloud-iac`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-iac`.
- Capability records cited: `iac/capabilities/iac-apply.yaml`, `iac/capabilities/iac-render.yaml`, `iac/capabilities/iac-rollback.yaml`.
- API surfaces cited: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar/policy artifacts cited: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar binding: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- State/event binding: `cloud_iac.cloud_iac`.
- Capability binding: `iac-apply`, `iac-render`, `iac-rollback`.
- SLO binding: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`.
- Runbook binding: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-iac`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-iac`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-iac` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-iac` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-iac`.

### Hyperscaler precedents
- Precedent 1: Cloudflare Bot Management and Turnstile is the reference pattern for the control shape described here.
- Precedent 2: Stripe Radar passive risk scoring is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-iac` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya-ci-required` evidence should include marker absence, section-count policy, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §critical-path-edge-cases
This anchor is closed for `cloud-iac` against documentation-rigor.md §3.2.5: applicable human-safety and platform edge-case handling.

### Service-specific answer
- Network partition: `cloud-iac` keeps tenant-local reads when safe, stops cross-cell writes that would violate residency, and emits degraded-mode audit events.
- Byzantine caller: Cedar denies forged `principal_id`, mismatched `tenant_id`, invalid SVID, replayed idempotency keys, and suspicious bot-score context.
- Regional outage: home-cell failover follows `multi-region.md`; if a pack forbids cross-border DR, `cloud-iac` preserves local queue state instead of failing open.
- Key compromise: ADR-0296 sidecar revokes OpenBao leases, rotates signing keys, and quarantines affected audit event classes for reconciliation.
- Account recovery/hijack path: identity step-up and `cloud-iac` audit evidence keep legitimate recovery from becoming an adversary shortcut.
- Mistaken mutation path: high-impact `iac-apply` mutations require idempotency, undo/cooldown where product semantics allow, and sealed evidence for later correction.
- Disaster surge: `cloud-iac` enforces per-tenant isolation so one hot tenant or emergency mode cannot starve unrelated cells.
- Verification: capacity math in `capacity-model.md`, rollback in `failure-modes.md`, DR handling in `multi-region.md`, and incident actions in runbooks.

### Concrete inventory used
- Service: `cloud-iac`; owner `axis-cloud-iac`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-iac`.
- Capability records cited: `iac/capabilities/iac-apply.yaml`, `iac/capabilities/iac-render.yaml`, `iac/capabilities/iac-rollback.yaml`.
- API surfaces cited: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar/policy artifacts cited: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar binding: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- State/event binding: `cloud_iac.cloud_iac`.
- Capability binding: `iac-apply`, `iac-render`, `iac-rollback`.
- SLO binding: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`.
- Runbook binding: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-iac`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-iac`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-iac` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-iac` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-iac`.

### Hyperscaler precedents
- Precedent 1: Google SRE incident playbooks is the reference pattern for the control shape described here.
- Precedent 2: Stripe idempotent mutation recovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-iac` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya-ci-required` evidence should include marker absence, section-count policy, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §credential-isolation
This anchor is closed for `cloud-iac` against ADR-0296 §D-1: credential sidecar, OpenBao TTLs and secret-reference path.

### Service-specific answer
- Credential scopes for `cloud-iac` include `identity`, `tenancy`, `policy-engine`, `observability`, `audit-chain`, `cloud-secrets`.
- SecretReference pattern is `${openbao:secret/<tenant_id>/cloud-iac/<credential-name>}`; platform-owned credentials use tenant `oyatie` only for oyatie-internal calls.
- Sidecar mode is required for audit-signing keys and preferred for provider credentials; raw credentials never enter domain/core crates.
- OpenBao token TTL is ≤60 seconds when sidecar isolation is not possible; refresh must be policy-allowed and audit-emitted.
- Rotation cadence is linked from `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +2 more.
- Example: `iac-apply` asks the sidecar for a scoped credential after Cedar allow; the sidecar returns a short-lived handle, not the underlying secret.
- Compromise response: revoke lease, rotate key, disable affected provider adapter, replay idempotent queue after audit-chain reconciliation.
- encryption-BYOK is separate from provider-BYOK; this section covers provider/API/signing credentials only unless a pack explicitly adds KMS-root behavior.

### Concrete inventory used
- Service: `cloud-iac`; owner `axis-cloud-iac`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-iac`.
- Capability records cited: `iac/capabilities/iac-apply.yaml`, `iac/capabilities/iac-render.yaml`, `iac/capabilities/iac-rollback.yaml`.
- API surfaces cited: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar/policy artifacts cited: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/proto/cloud-iac.proto`.
- Cedar binding: `iac/policy/auditor-scope.cedar`, `iac/policy/ci-scope.cedar`, `iac/policy/data-residency.md`, `iac/policy/iac-isolation.md`, `iac/policy/public-read.cedar`, `iac/policy/tenant-scope.cedar`.
- State/event binding: `cloud_iac.cloud_iac`.
- Capability binding: `iac-apply`, `iac-render`, `iac-rollback`.
- SLO binding: `iac/slos/helm-chart-lint-correctness.openslo.yaml`, `iac/slos/iac-apply-latency.openslo.yaml`, `iac/slos/iac-render-latency.openslo.yaml`, `iac/slos/iac-rollback-latency.openslo.yaml`, `iac/slos/iac-validator-availability.openslo.yaml`, `iac/slos/slsa-provenance-completeness.openslo.yaml`.
- Runbook binding: `iac/runbooks/drift-remediation.md`, `iac/runbooks/gitops-reconciler-restart.md`, `iac/runbooks/registry-restore.md`, `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/rollback-orchestration.md`, `iac/runbooks/seaweedfs-volume-failover.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-iac`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-iac`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-iac` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-iac` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-iac`.

### Hyperscaler precedents
- Precedent 1: HashiCorp Vault dynamic secrets is the reference pattern for the control shape described here.
- Precedent 2: AWS KMS envelope-key isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-iac` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya-ci-required` evidence should include marker absence, section-count policy, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.
