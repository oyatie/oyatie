# IP-001 Data Pipeline tenant-scope kernel

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-001-tenant-scope-kernel.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Define the kernel value objects that make every Data Pipeline command tenant-bound before adapters run.
- Keep tenant scope independent of connector vendor naming.
- Bind connector, pipeline-run, transform, lineage, and replay documents to one tenant envelope.
- Prevent source-system ids from becoming cross-tenant lookup keys.
- Ensure source snapshots, CDC cursors, and replay windows cannot be interpreted without home cell.
- Give Cedar facts a stable kernel shape shared by REST, AsyncAPI, proto, and worker paths.
- Keep DealSet license checks below tenant scope rather than beside it.
- Make Fivetran and Airbyte Cloud parity subordinate to Oyatie tenant proof.
- Make Hevo and Stitch ease-of-use subordinate to tenant and audit proof.
- Make Matillion, Talend Cloud, Informatica IICS, and Estuary Flow enterprise pressure subordinate to tenant isolation.

## Local references
- `microservices/data-pipeline/PRD.md` defines connector, pipeline-run, transform, lineage, and replay scope.
- `microservices/data-pipeline/ARCHITECTURE.md` defines the ADR-0105 layer map.
- `microservices/data-pipeline/contracts/local-openapi-v1.yaml` carries public command DTOs.
- `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml` carries event envelopes.
- `microservices/data-pipeline/contracts/local-operations-v1.proto` carries internal operations.
- `microservices/data-pipeline/policies/local-ingest-source-scope.cedar` consumes tenant facts.
- `microservices/data-pipeline/policy/auditor-scope.cedar` consumes auditor tenant scope.
- `microservices/data-pipeline/capabilities/connector-run-start.yaml` names tenant fields.
- `microservices/data-pipeline/capabilities/replay-cursor-advance.yaml` names replay tenant fields.
- `microservices/data-pipeline/dpia.md` constrains tenant evidence handling.

## Kernel value objects
- `TenantScope` contains `tenant_id`, `home_cell`, `jurisdiction_code`, and `pack_overlay_ids`.
- `PipelinePrincipal` contains `principal_id`, `audience_type`, and delegated actor chain.
- `SourceObjectScope` contains `connector_id`, `source_system_id`, and `source_object_id`.
- `DataPipelinePurpose` contains business purpose, replay purpose, and export purpose.
- `DataPipelineClass` contains source data class, derived data class, and audit data class.
- `ConnectorRunScope` binds tenant, principal, source object, and connector catalog version.
- `TransformRunScope` binds tenant, principal, transform job, transform version, and source object.
- `LineageScope` binds tenant, graph partition, reconciliation epoch, and edge custody id.
- `ReplayScope` binds tenant, cursor range, dead-letter custody id, and rollback bundle id.
- `TenantAuditScope` binds audit event class, trace id, idempotency key, and Cedar decision id.

## Command deltas
- `connector.run.start` must accept `TenantScope`.
- `connector.run.start` must reject bare source-system ids.
- `schema.drift.hold` must carry `SourceObjectScope`.
- `pipeline.run.create` must carry `DataPipelinePurpose`.
- `transform.job.approve` must carry `TransformRunScope`.
- `lineage.edge.record` must carry `LineageScope`.
- `replay.cursor.advance` must carry `ReplayScope`.
- `deadletter.replay.approve` must carry both `ReplayScope` and `TenantAuditScope`.
- `watermark.advance` must carry home cell and data class.
- `dealset.connector.license` must read tenant scope before license scope.

## Event deltas
- `oya.data.pipeline.connector.run.started` includes tenant and home cell.
- `oya.data.pipeline.connector.run.denied` includes Cedar decision id.
- `oya.data.pipeline.schema_drift.quarantined` includes source object scope.
- `oya.data.pipeline.transform.approved` includes transform run scope.
- `oya.data.pipeline.lineage.edge.recorded` includes graph partition.
- `oya.data.pipeline.replay.cursor.advanced` includes replay cursor range.
- `oya.data.pipeline.dead_letter.captured` includes custody tenant envelope.
- `oya.data.pipeline.watermark.advanced` includes watermark tenant envelope.
- `oya.data.pipeline.dealset.connector.checked` includes tenant and license scope.
- `oya.data.pipeline.audit.exported` includes tenant audit scope.

## Proto deltas
- `TenantScope` is a first-class proto message.
- `PipelinePrincipal` is a first-class proto message.
- `SourceObjectScope` is a first-class proto message.
- `ReplayScope` is a first-class proto message.
- `TransformRunScope` is a first-class proto message.
- `LineageScope` is a first-class proto message.
- `TenantAuditScope` is a first-class proto message.
- Every internal request embeds one tenant-bearing scope message.
- Every internal response echoes `tenant_id` and `audit_event_id`.
- Proto validation rejects requests with source ids but no tenant.

## Cedar facts
- `principal_tenant` comes from `TenantScope.tenant_id`.
- `principal_audience` comes from `PipelinePrincipal.audience_type`.
- `resource_tenant` comes from source object catalog binding.
- `resource_cell` comes from `TenantScope.home_cell`.
- `resource_data_class` comes from `DataPipelineClass`.
- `action_purpose` comes from `DataPipelinePurpose`.
- `connector_license_state` comes from DealSet lookup.
- `replay_window_state` comes from `ReplayScope`.
- `lineage_partition` comes from `LineageScope`.
- `audit_chain_target` comes from `TenantAuditScope`.

## Workflow decisions
- Workflow templates must start with tenant validation.
- Connector discovery cannot start before tenant validation.
- Schema drift review cannot materialize samples before Cedar permit.
- Transform approval cannot run without tenant budget context.
- Lineage repair cannot read graph partition without tenant scope.
- Replay approval cannot inspect dead-letter payload without tenant scope.
- Watermark advancement cannot use provider freshness without tenant scope.
- DealSet license workflow cannot execute before tenant ownership check.
- Audit export workflow must keep tenant id in signed evidence, not metrics labels.
- Emergency bypass remains out of scope for normal Data Pipeline tenant kernel commands.

## Failure cases
- Missing tenant id fails before connector adapter call.
- Unknown home cell fails before storage adapter call.
- Jurisdiction mismatch blocks source object reads.
- Principal tenant mismatch emits deny evidence.
- Source object tenant mismatch emits security incident evidence.
- Replay custody tenant mismatch blocks cursor movement.
- Lineage graph tenant mismatch blocks edge write.
- Transform job tenant mismatch blocks approval.
- DealSet tenant mismatch blocks connector license use.
- Audit-chain outage blocks high-risk mutation.

## Replay cases
- Replay cannot reuse tenant scope from the original failed item without revalidation.
- Replay must compare current tenant pack overlays with failure-time overlays.
- Replay must compare current Cedar decision with failure-time decision.
- Replay must preserve original source object scope.
- Replay must preserve original connector catalog version.
- Replay must preserve original transform version.
- Replay must preserve original lineage epoch.
- Replay must preserve original watermark value.
- Replay must preserve original data class.
- Replay must emit a new audit event.

## Evidence fields
- `tenant_id` is mandatory.
- `home_cell` is mandatory.
- `jurisdiction_code` is mandatory.
- `principal_id` is mandatory.
- `audience_type` is mandatory.
- `connector_id` is mandatory for connector work.
- `source_object_id` is mandatory for source work.
- `data_class` is mandatory for all work.
- `purpose` is mandatory for all mutation work.
- `idempotency_key` is mandatory for all mutation work.
- `trace_id` is mandatory.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.
- `pack_overlay_ids` is mandatory.
- `dealset_decision_id` is mandatory when connector license applies.
- `rollback_bundle_id` is mandatory for replay or destructive correction.

## SLOs
- Tenant-scope validation p95 target is lower than connector adapter p95.
- Tenant-scope validation failures count against policy decision latency.
- Missing tenant failures do not count as provider failures.
- Cross-tenant denials count against abuse-defence dashboards.
- Audit emission lag tracks tenant mutation completion.
- Replay freshness excludes records blocked by tenant-scope denial.
- Ingest freshness excludes records blocked by tenant-scope denial.
- Lineage capture excludes edges blocked by tenant-scope denial.
- Cost attribution records tenant-scope validation time separately.
- Runbooks must distinguish tenant denial from source outage.

## Test cases
- Missing `tenant_id` rejects every command.
- Mismatched `home_cell` rejects connector run.
- Mismatched principal tenant rejects replay approval.
- Missing `data_class` rejects transform approval.
- Missing `purpose` rejects schema drift release.
- Missing `cedar_decision_id` rejects audit export.
- Cross-tenant source object rejects lineage edge record.
- Cross-tenant dead-letter custody rejects replay cursor advance.
- DealSet lookup cannot run before tenant scope.
- Idempotency replay returns same tenant-scoped result.

## Rollback
- Tenant-scope kernel rollback restores prior value-object definitions.
- In-flight connector runs keep old scope version until completion.
- New connector runs use the rolled-back scope version.
- Replay windows opened under new scope remain frozen until reviewed.
- Schema drift cases opened under new scope remain evidence-only.
- Lineage graph writes under new scope are revertible by audit event.
- Transform approvals under new scope require reapproval after rollback.
- DealSet decisions remain immutable even if scope version rolls back.
- Audit events are never deleted.
- Rollback evidence references ADR-0105 and ADR-0321.

## Acceptance criteria
- Every Data Pipeline command has tenant scope before adapter work.
- Every Data Pipeline event echoes tenant scope.
- Every proto request has a tenant-bearing scope message.
- Every Cedar decision can be traced to kernel facts.
- Every replay checks current and original tenant scope.
- Every benchmark reference is comparative, not authoritative.
- Every failure path emits refusal evidence.
- Every rollback path preserves audit history.
- Existing local contracts can add these fields without suite boundary changes.
- Data Pipeline remains the owner of its tenant kernel.

## Citation map
- `microservices/data-pipeline/PRD.md`
- `microservices/data-pipeline/ARCHITECTURE.md`
- `microservices/data-pipeline/contracts/local-openapi-v1.yaml`
- `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`
- `microservices/data-pipeline/contracts/local-operations-v1.proto`
- `microservices/data-pipeline/policies/local-ingest-source-scope.cedar`
- `microservices/data-pipeline/policy/auditor-scope.cedar`
- `microservices/data-pipeline/capabilities/connector-run-start.yaml`
- `microservices/data-pipeline/capabilities/replay-cursor-advance.yaml`
- `microservices/data-pipeline/dpia.md`
- `ADR-0105`
- `ADR-0314`
- `ADR-0321`

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/data-pipeline/IP-001-tenant-scope-kernel.md:23` - - `microservices/data-pipeline/contracts/local-openapi-v1.yaml` carries public command DTOs.; `microservices/data-pipeline/IP-001-tenant-scope-kernel.md:24` - - `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml` carries event envelopes..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-001-tenant-scope-kernel.md:146` - ## SLOs.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-001-tenant-scope-kernel.md:151` - - Audit emission lag tracks tenant mutation completion.; `microservices/data-pipeline/IP-001-tenant-scope-kernel.md:155` - - Cost attribution records tenant-scope validation time separately..
