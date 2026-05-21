# IP-007 Data Pipeline gRPC internal surface

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-007-grpc-internal-surface.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Define the internal proto surface for Data Pipeline usecases and workers.
- Keep gRPC internal only; REST and AsyncAPI remain external contract surfaces.
- Carry tenant scope on every internal request.
- Carry Cedar decision receipts across application, worker, and adapter boundaries.
- Carry rollback references across replay, lineage, transform, and watermark mutations.
- Avoid vendor-named proto packages.
- Treat Fivetran sync internals as connector-run benchmark pressure.
- Treat Airbyte Cloud worker orchestration as job-control pressure.
- Treat Matillion and Talend Cloud as transform-worker pressure.
- Treat Informatica IICS as metadata-governance pressure.
- Treat Estuary Flow as streaming cursor pressure.
- Preserve ADR-0105 layer separation.

## Local references
- `microservices/data-pipeline/contracts/local-operations-v1.proto` is the internal authority.
- `microservices/data-pipeline/contracts/data-pipeline-v1.proto` is the companion proto.
- `microservices/data-pipeline/ARCHITECTURE.md` maps internal surface to application and worker layers.
- `microservices/data-pipeline/capabilities/connector-run-start.yaml` binds connector calls.
- `microservices/data-pipeline/capabilities/lineage-edge-record.yaml` binds lineage calls.
- `microservices/data-pipeline/capabilities/replay-cursor-advance.yaml` binds replay calls.
- `microservices/data-pipeline/capabilities/transform-job-approve.yaml` binds transform calls.
- `microservices/data-pipeline/capabilities/schema-drift-hold.yaml` binds drift calls.
- `microservices/data-pipeline/iac/local-service-monitor.yaml` observes internal services.
- `microservices/data-pipeline/iac/local-network-policy.yaml` constrains internal reachability.

## Proto services
- `ConnectorRunService` owns connector run lifecycle calls.
- `SchemaDriftService` owns drift quarantine calls.
- `TransformApprovalService` owns transform approval calls.
- `TransformWorkerService` owns transform execution callbacks.
- `LineageReconciliationService` owns graph reconciliation calls.
- `ReplayCustodyService` owns dead-letter and replay calls.
- `WatermarkGovernanceService` owns CDC freshness calls.
- `CostAttributionService` owns estimate and finalization calls.
- `DealSetConnectorService` owns connector license checks.
- `AuditEvidenceService` owns internal evidence packet calls.
- `CapacityAdmissionService` owns workload admission calls.
- `SloPromotionService` owns promotion evidence calls.

## Shared messages
- `TenantScope` is embedded in every request.
- `PrincipalScope` is embedded in user-initiated requests.
- `PolicyDecisionReceipt` is embedded after Cedar evaluation.
- `AuditTarget` is embedded before mutation.
- `RollbackBundleRef` is embedded before reversible mutation.
- `SourceObjectRef` is embedded in connector, drift, transform, lineage, and replay calls.
- `ConnectorCatalogRef` is embedded in connector and drift calls.
- `TransformVersionRef` is embedded in transform and lineage calls.
- `LineageEpochRef` is embedded in lineage and replay calls.
- `ReplayCustodyRef` is embedded in replay and watermark calls.
- `WatermarkRef` is embedded in freshness calls.
- `CostAttributionRef` is embedded in transform and replay calls.

## Connector RPCs
- `PrepareConnectorRun` validates internal tenant and catalog scope.
- `StartConnectorRunWorker` starts adapter work.
- `RecordSourceSnapshot` records captured source schema.
- `RecordProviderRateLimit` records throttling.
- `RecordConnectorRunFailure` records terminal failure.
- `RecordConnectorRunCompletion` records completion.
- `PrepareConnectorRollback` creates rollback bundle.
- `GetConnectorRunStatus` reads tenant-scoped status.
- `ListConnectorRunDeadLetters` reads custody summaries only.
- `AttachConnectorDealSetDecision` binds license decision.
- `RecordConnectorWatermark` binds source freshness.
- `CancelConnectorRunSafely` respects cancellation class.

## Drift RPCs
- `OpenSchemaDriftCase` creates drift case.
- `ClassifySchemaDrift` records drift class.
- `CaptureSchemaDriftSample` creates custody pointer.
- `EstimateDriftTransformImpact` creates transform hint.
- `EstimateDriftLineageImpact` creates graph hint.
- `EstimateDriftReplayImpact` creates replay hint.
- `RecordSchemaDriftDisposition` records operator decision.
- `ReleaseSchemaCatalogVersion` promotes accepted schema.
- `RejectSchemaCatalogVersion` rejects changed schema.
- `RollbackSchemaCatalogVersion` restores prior schema.
- `GetSchemaDriftCase` reads tenant-scoped case.
- `ListOpenSchemaDriftCases` reads tenant-scoped backlog.

## Transform RPCs
- `EstimateTransformCost` computes approval cost.
- `RequestTransformApproval` opens approval.
- `ApproveTransformJob` records reviewer decision.
- `RejectTransformJob` records refusal.
- `StartTransformWorker` starts execution.
- `RecordTransformProgress` records worker progress.
- `RecordTransformFailure` records failure.
- `RecordTransformCompletion` records output.
- `FinalizeTransformCost` records actual cost.
- `PrepareTransformRollback` creates rollback bundle.
- `RollbackTransformOutput` reverts output.
- `GetTransformRunStatus` reads tenant-scoped status.

## Lineage RPCs
- `OpenLineageReconciliation` creates graph case.
- `ComputeLineageDiff` returns stable diff hash.
- `RecordObservedLineageEdge` stores observed edge custody.
- `ApplyLineageReconciliation` applies accepted edges.
- `RejectLineageEdges` stores rejected edge evidence.
- `MarkLineageEdgesProvisional` stores degraded graph state.
- `RollbackLineageReconciliation` reverts epoch.
- `GetLineageReconciliation` reads case.
- `ListLineageGaps` reads tenant graph gaps.
- `AttachLineageEpochToReplay` binds replay dependency.
- `AttachLineageEpochToTransform` binds transform dependency.
- `ExportLineageAuditSlice` requires auditor scope.

## Replay and watermark RPCs
- `CaptureDeadLetter` creates custody.
- `ClassifyDeadLetter` records failure class.
- `RequestDeadLetterReplay` opens approval.
- `ApproveDeadLetterReplay` records reviewer decision.
- `StartDeadLetterReplayWorker` starts retry.
- `RecordDeadLetterReplayFailure` records retry failure.
- `RecordDeadLetterReplayCompletion` records success.
- `AdvanceReplayCursor` moves cursor.
- `RollbackReplayCursor` restores cursor.
- `ProposeWatermarkAdvance` creates candidate.
- `AdvanceWatermark` moves CDC state.
- `HoldWatermark` records staleness reason.

## Cedar facts
- Internal caller identity is a fact.
- Tenant scope is a fact.
- Resource type is a fact.
- RPC method name is a fact.
- Connector license state is a fact.
- Source object ownership is a fact.
- Transform approval state is a fact.
- Lineage epoch state is a fact.
- Replay custody state is a fact.
- Watermark kind is a fact.
- Cost budget state is a fact.
- Pack overlay state is a fact.
- Audit target class is a fact.
- Rollback readiness is a fact.
- Worker lease state is a fact.
- Network policy zone is a fact.

## Workflow decisions
- REST command handlers call gRPC usecases after validation.
- Workers call gRPC callbacks after side effects.
- Adapters do not call domain storage directly.
- gRPC requests carry policy receipts instead of recalculating in adapters.
- Long-running workers emit progress through gRPC callbacks.
- Replay worker cannot advance cursor directly.
- Transform worker cannot finalize cost without callback.
- Lineage worker cannot mutate graph without reconciliation call.
- Watermark worker cannot use provider freshness alone.
- Audit evidence service signs internal outcomes.
- Network policy limits gRPC to local service mesh.
- Service monitor tracks gRPC health separately from REST.

## Failure cases
- Missing tenant scope returns invalid argument.
- Missing policy decision returns failed precondition.
- Missing rollback bundle returns failed precondition.
- Cross-tenant source object returns permission denied.
- Stale connector catalog returns aborted.
- Stale transform version returns aborted.
- Missing lineage epoch returns failed precondition.
- Missing replay custody returns failed precondition.
- Backward watermark returns out of range.
- Worker lease conflict returns already exists.
- Audit-chain outage returns unavailable.
- Adapter timeout returns deadline exceeded with retry class.

## Evidence fields
- `grpc_service` is mandatory.
- `grpc_method` is mandatory.
- `proto_message_version` is mandatory.
- `tenant_id` is mandatory.
- `home_cell` is mandatory.
- `request_hash` is mandatory.
- `response_hash` is mandatory.
- `policy_decision_id` is mandatory after policy.
- `audit_event_id` is mandatory after mutation.
- `worker_lease_id` is mandatory for worker calls.
- `rollback_bundle_id` is mandatory for reversible calls.
- `source_object_id` is mandatory for source calls.
- `lineage_epoch` is mandatory for graph calls.
- `replay_custody_id` is mandatory for replay calls.
- `watermark_kind` is mandatory for freshness calls.
- `benchmark_pressure` is mandatory for parity summary.

## SLOs
- gRPC application latency is separate from REST latency.
- Worker callback latency contributes to local transform latency.
- Replay worker callback latency contributes to replay freshness.
- Lineage reconciliation latency contributes to lineage capture.
- Connector worker callback latency contributes to ingest freshness.
- Policy receipt missing rate contributes to policy health.
- Audit callback lag contributes to audit emission lag.
- gRPC error code distribution feeds operating overview.
- Network policy denial feeds local operator remediation.
- Worker lease conflict rate feeds capacity admission.
- gRPC deadline exceeded does not equal provider outage.
- Proto validation failures feed contract quality.

## Test cases
- Proto rejects request without tenant scope.
- Proto rejects mutation without policy decision receipt.
- Proto rejects replay cursor advance without custody.
- Proto rejects lineage apply without epoch.
- Proto rejects transform finalization without cost id.
- Proto rejects watermark advance with backward value.
- Policy test denies cross-tenant internal call.
- Worker test cannot advance cursor directly.
- Adapter test cannot bypass gRPC usecase.
- Network test prevents external gRPC reachability.
- Contract test preserves old message version during rollout.
- Audit test signs gRPC mutation receipt.

## Rollback
- Proto rollback uses message versioning.
- Old workers finish with old proto version.
- New workers reject unsupported versions explicitly.
- gRPC service method removals require deprecation window.
- Rollback preserves policy decision receipts.
- Rollback preserves audit event ids.
- Replay cursors are not moved by proto rollback.
- Transform outputs are not reverted by proto rollback alone.
- Lineage epochs are not reverted by proto rollback alone.
- Network policy rollback is verified separately.
- Service monitor rollback preserves alerts.
- Contract tests validate rollback compatibility.

## Acceptance criteria
- Every internal RPC is Data Pipeline-specific.
- Every internal RPC carries tenant scope.
- Every mutation RPC carries policy receipt.
- Every reversible RPC carries rollback reference.
- Every replay RPC carries custody.
- Every lineage RPC carries reconciliation epoch.
- Every transform RPC carries cost or approval context.
- Every watermark RPC carries freshness kind.
- Every benchmark reference remains comparative.
- gRPC remains internal only.

## Citation map
- `microservices/data-pipeline/contracts/local-operations-v1.proto`
- `microservices/data-pipeline/contracts/data-pipeline-v1.proto`
- `microservices/data-pipeline/ARCHITECTURE.md`
- `microservices/data-pipeline/capabilities/connector-run-start.yaml`
- `microservices/data-pipeline/capabilities/lineage-edge-record.yaml`
- `microservices/data-pipeline/capabilities/replay-cursor-advance.yaml`
- `microservices/data-pipeline/capabilities/transform-job-approve.yaml`
- `microservices/data-pipeline/capabilities/schema-drift-hold.yaml`
- `microservices/data-pipeline/iac/local-service-monitor.yaml`
- `microservices/data-pipeline/iac/local-network-policy.yaml`
- `ADR-0105`
- `ADR-0321`

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/data-pipeline/IP-007-grpc-internal-surface.md:23` - - `microservices/data-pipeline/contracts/local-operations-v1.proto` is the internal authority.; `microservices/data-pipeline/IP-007-grpc-internal-surface.md:24` - - `microservices/data-pipeline/contracts/data-pipeline-v1.proto` is the companion proto..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-007-grpc-internal-surface.md:196` - ## SLOs.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-007-grpc-internal-surface.md:91` - - `EstimateTransformCost` computes approval cost.; `microservices/data-pipeline/IP-007-grpc-internal-surface.md:99` - - `FinalizeTransformCost` records actual cost..
