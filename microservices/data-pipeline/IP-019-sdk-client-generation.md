# IP-019 Data Pipeline SDK client generation

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-019-sdk-client-generation.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Generate SDK clients from Data Pipeline REST, AsyncAPI, and proto contracts.
- Keep SDK methods aligned to connector, drift, transform, lineage, replay, watermark, cost, and audit operations.
- Prevent SDK convenience helpers from bypassing tenant scope, idempotency, Cedar receipt, or audit target.
- Expose typed error codes for policy, license, residency, custody, and freshness failures.
- Treat Fivetran and Airbyte Cloud client ergonomics as benchmark pressure.
- Treat Hevo and Stitch setup simplicity as benchmark pressure.
- Treat Matillion and Talend Cloud transform SDKs as workflow pressure.
- Treat Informatica IICS metadata clients as governance pressure.
- Treat Estuary Flow streaming clients as watermark pressure.
- Preserve local contract source of truth.

## Local references
- `microservices/data-pipeline/sdk-plan.md` is the SDK authority.
- `microservices/data-pipeline/contracts/local-openapi-v1.yaml` generates REST clients.
- `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml` generates event clients.
- `microservices/data-pipeline/contracts/local-operations-v1.proto` generates internal clients.
- `microservices/data-pipeline/contracts/openapi-v1.yaml` is companion REST.
- `microservices/data-pipeline/contracts/asyncapi-v1.yaml` is companion event.
- `microservices/data-pipeline/contracts/data-pipeline-v1.proto` is companion proto.
- `microservices/data-pipeline/capabilities/connector-run-start.yaml` defines connector method.
- `microservices/data-pipeline/capabilities/transform-job-approve.yaml` defines transform method.
- `microservices/data-pipeline/capabilities/replay-cursor-advance.yaml` defines replay method.

## SDK modules
- `DataPipelineConnectorClient` starts and reads connector runs.
- `DataPipelineSchemaDriftClient` handles drift cases.
- `DataPipelineTransformClient` handles transform approval and status.
- `DataPipelineLineageClient` handles reconciliation cases.
- `DataPipelineReplayClient` handles dead-letter replay and cursor status.
- `DataPipelineWatermarkClient` handles CDC freshness.
- `DataPipelineCostClient` handles estimates and actuals.
- `DataPipelineDealSetClient` handles connector license checks.
- `DataPipelineAuditClient` handles evidence exports.
- `DataPipelineCapacityClient` handles admission reads.
- `DataPipelineSloClient` handles promotion evidence.
- `DataPipelineIncidentClient` handles runbook-linked incidents.

## Generated request guards
- Tenant id is required by every mutation helper.
- Home cell is required by every mutation helper.
- Principal id is required by actor-driven helpers.
- Data class is required by source, transform, replay, and export helpers.
- Purpose is required by mutation helpers.
- Idempotency key is required by mutation helpers.
- Trace context is required by mutation helpers.
- Cedar decision receipt is required after explicit policy helper.
- Audit target is required by mutation helpers.
- DealSet decision is required by licensed connector helpers.
- Pack overlay is required by regulated helpers.
- Replay custody id is required by replay helpers.

## Typed errors
- `TenantRequiredError` maps tenant missing.
- `PolicyDeniedError` maps Cedar denial.
- `ConnectorLicenseInactiveError` maps DealSet inactive.
- `SchemaDriftUnresolvedError` maps drift blocker.
- `TransformCostRequiredError` maps missing estimate.
- `LineageEpochRequiredError` maps missing reconciliation.
- `ReplayCustodyRequiredError` maps missing custody.
- `WatermarkBackwardError` maps invalid CDC advance.
- `PackOverlayRequiredError` maps residency blocker.
- `CapacityDeniedError` maps admission refusal.
- `AuditUnavailableError` maps audit-chain outage.
- `ProviderRateLimitedError` maps provider throttling.

## Command methods
- `startConnectorRun` wraps connector run start.
- `openSchemaDriftCase` wraps drift hold.
- `recordSchemaDriftDisposition` wraps drift closure.
- `requestTransformApproval` wraps transform approval.
- `openLineageReconciliation` wraps graph case.
- `applyLineageReconciliation` wraps graph apply.
- `approveDeadLetterReplay` wraps replay approval.
- `advanceReplayCursor` wraps cursor movement.
- `advanceWatermark` wraps CDC freshness.
- `checkConnectorDealSet` wraps license check.
- `requestCapacityAdmission` wraps admission check.
- `createAuditExport` wraps evidence packet.

## Event methods
- `onConnectorRunAccepted` subscribes to connector acceptance.
- `onConnectorRunCompleted` subscribes to connector completion.
- `onSchemaDriftQuarantined` subscribes to drift hold.
- `onTransformApproved` subscribes to transform approval.
- `onTransformCostFinalized` subscribes to actual cost.
- `onLineageReconciliationApplied` subscribes to graph apply.
- `onDeadLetterCaptured` subscribes to custody capture.
- `onReplayCursorAdvanced` subscribes to cursor movement.
- `onWatermarkAdvanced` subscribes to freshness movement.
- `onDealSetConnectorChecked` subscribes to license state.
- `onAuditExportCreated` subscribes to evidence export.
- `onRollbackCompleted` subscribes to compensating events.

## Proto clients
- Internal SDK exposes connector run service stubs.
- Internal SDK exposes schema drift service stubs.
- Internal SDK exposes transform worker service stubs.
- Internal SDK exposes lineage reconciliation service stubs.
- Internal SDK exposes replay custody service stubs.
- Internal SDK exposes watermark governance service stubs.
- Internal SDK exposes cost attribution service stubs.
- Internal SDK exposes capacity admission service stubs.
- Internal SDK exposes audit evidence service stubs.
- Internal SDK hides adapter-only methods from public packages.
- Internal SDK requires service mesh identity.
- Internal SDK rejects use outside allowed runtime.

## Cedar facts
- SDK policy helper builds tenant facts.
- SDK policy helper builds source object facts.
- SDK policy helper builds transform facts.
- SDK policy helper builds lineage facts.
- SDK policy helper builds replay facts.
- SDK policy helper builds watermark facts.
- SDK policy helper builds DealSet facts.
- SDK policy helper builds pack overlay facts.
- SDK policy helper builds capacity facts.
- SDK policy helper builds audit export facts.
- SDK does not accept raw permit boolean.
- SDK passes decision receipt object.

## Workflow decisions
- Public SDK starts commands; it does not run workers.
- Public SDK subscribes to events; it does not mutate projections directly.
- Internal SDK is restricted to service mesh.
- SDK generation fails if required headers are missing.
- SDK generation fails if operation ids are missing.
- SDK generation fails if error taxonomy is missing.
- SDK helper cannot synthesize tenant id.
- SDK helper cannot synthesize Cedar receipt.
- SDK helper can create idempotency key only when caller asks.
- SDK helper surfaces runbook refs on actionable errors.
- SDK helper surfaces benchmark metadata only in parity reports.
- SDK helper redacts commercial DealSet details by default.

## Failure cases
- Contract drift blocks SDK generation.
- Missing operation id blocks SDK generation.
- Missing required header blocks SDK generation.
- Missing typed error blocks SDK generation.
- Missing event version blocks event SDK generation.
- Missing proto message version blocks internal SDK generation.
- SDK method without idempotency support is rejected.
- SDK method without tenant scope is rejected.
- SDK replay helper without custody id is rejected.
- SDK lineage helper without epoch is rejected.
- SDK watermark helper with provider-only freshness is rejected.
- SDK audit helper without auditor scope is rejected.

## Replay cases
- Replay SDK requires custody case id.
- Replay SDK requires cursor before and target.
- Replay SDK exposes held replay state.
- Replay SDK exposes replay freshness impact.
- Replay SDK exposes original failure event id.
- Replay SDK exposes current policy decision id.
- Replay SDK exposes DealSet state when licensed.
- Replay SDK exposes pack overlay blocker.
- Replay SDK exposes rollback bundle id.
- Replay SDK does not expose raw dead-letter payload.
- Replay SDK maps duplicate replay to idempotent response.
- Replay SDK maps cursor rollback to typed event.

## Evidence fields
- `sdk_package_name` is mandatory.
- `sdk_version` is mandatory.
- `source_contract` is mandatory.
- `operation_id` is mandatory for command methods.
- `event_type` is mandatory for event methods.
- `proto_service` is mandatory for internal stubs.
- `tenant_scope_required` is mandatory.
- `idempotency_required` is mandatory for mutations.
- `policy_receipt_required` is mandatory when policy-gated.
- `audit_target_required` is mandatory when mutation.
- `typed_error_count` is mandatory.
- `generated_at` is mandatory.
- `contract_hash` is mandatory.
- `test_fixture_id` is mandatory.
- `benchmark_pressure` is mandatory for parity summary.
- `redaction_profile` is mandatory for DealSet and payload fields.

## SLOs
- SDK generation freshness is tracked by contract hash age.
- SDK contract test failures block promotion.
- SDK typed error coverage feeds quality dashboard.
- SDK replay helper tests feed replay safety.
- SDK event subscription tests feed audit completeness.
- SDK internal stub compatibility feeds worker health.
- SDK idempotency helper tests feed duplicate safety.
- SDK redaction tests feed compliance health.
- SDK benchmark parity report is documentation evidence only.
- SDK generated clients do not affect runtime availability.
- SDK stale version count feeds release readiness.
- SDK method coverage feeds catalog registration.

## Test cases
- Generated connector method requires tenant id.
- Generated transform method requires cost estimate id.
- Generated replay method requires custody id.
- Generated lineage method requires reconciliation epoch.
- Generated watermark method rejects provider-only freshness.
- Generated DealSet method redacts commercial details.
- Generated audit export method requires auditor scope.
- Generated events include event version.
- Generated proto stubs include message version.
- Typed errors map REST error codes.
- SDK generation fails on missing operation id.
- SDK idempotency helper returns stable key when provided seed is fixed.

## Rollback
- SDK rollback publishes prior package version.
- Contract rollback regenerates SDK from prior hash.
- Deprecated methods remain until removal window closes.
- Typed errors remain backward compatible.
- Event subscribers keep prior event versions.
- Proto stubs keep prior message versions.
- Replay helpers preserve custody requirement.
- DealSet redaction remains enabled after rollback.
- Audit export helpers preserve auditor scope.
- Rollback emits SDK version event.
- Tests run against prior and active SDK.
- Benchmark report is regenerated after rollback.

## Acceptance criteria
- SDK methods are generated from local contracts.
- SDK mutation helpers require tenant and idempotency.
- SDK policy helpers return decision receipts.
- SDK replay helpers require custody ids.
- SDK lineage helpers require reconciliation epochs.
- SDK transform helpers require cost ids.
- SDK watermark helpers distinguish freshness kinds.
- SDK redacts DealSet and payload-sensitive fields.
- Every benchmark reference is comparative.
- SDK generation remains Data Pipeline-specific.

## Citation map
- `microservices/data-pipeline/sdk-plan.md`
- `microservices/data-pipeline/contracts/local-openapi-v1.yaml`
- `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`
- `microservices/data-pipeline/contracts/local-operations-v1.proto`
- `microservices/data-pipeline/contracts/openapi-v1.yaml`
- `microservices/data-pipeline/contracts/asyncapi-v1.yaml`
- `microservices/data-pipeline/contracts/data-pipeline-v1.proto`
- `microservices/data-pipeline/capabilities/connector-run-start.yaml`
- `microservices/data-pipeline/capabilities/transform-job-approve.yaml`
- `microservices/data-pipeline/capabilities/replay-cursor-advance.yaml`
- `ADR-0105`
- `ADR-0321`

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/data-pipeline/IP-019-sdk-client-generation.md:22` - - `microservices/data-pipeline/contracts/local-openapi-v1.yaml` generates REST clients.; `microservices/data-pipeline/IP-019-sdk-client-generation.md:23` - - `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml` generates event clients..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-019-sdk-client-generation.md:190` - ## SLOs.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-019-sdk-client-generation.md:10` - - Keep SDK methods aligned to connector, drift, transform, lineage, replay, watermark, cost, and audit operations.; `microservices/data-pipeline/IP-019-sdk-client-generation.md:93` - - `onTransformCostFinalized` subscribes to actual cost..
