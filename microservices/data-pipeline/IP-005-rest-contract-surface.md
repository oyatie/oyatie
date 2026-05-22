# IP-005 Data Pipeline REST contract surface

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-005-rest-contract-surface.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Define the REST contract that starts connector runs.
- Define the REST contract that opens schema drift holds.
- Define the REST contract that approves transform jobs.
- Define the REST contract that records lineage edges.
- Define the REST contract that approves dead-letter replay.
- Define the REST contract that advances replay cursors.
- Define the REST contract that advances CDC watermarks.
- Define the REST contract that exports audit evidence.
- Keep HTTP/3-first behavior compatible with ADR-0253-amendment.
- Keep OpenAPI 3.2.0 as the public shape.
- Keep REST commands tenant-scoped before body-specific validation.
- Keep vendor benchmark fields out of path names.

## Local references
- `microservices/data-pipeline/contracts/local-openapi-v1.yaml` is the immediate REST authority.
- `microservices/data-pipeline/contracts/openapi-v1.yaml` is the non-local companion contract.
- `microservices/data-pipeline/PRD.md` provides command intent.
- `microservices/data-pipeline/ARCHITECTURE.md` maps REST to ADR-0105.
- `microservices/data-pipeline/policies/local-ingest-source-scope.cedar` gates connector run.
- `microservices/data-pipeline/policies/local-transform-run-control.cedar` gates transform approval.
- `microservices/data-pipeline/policies/local-deadletter-replay-approval.cedar` gates replay approval.
- `microservices/data-pipeline/policy/auditor-scope.cedar` gates evidence export.
- `microservices/data-pipeline/slos/read-latency.openslo.yaml` anchors read latency.
- `microservices/data-pipeline/slos/write-latency.openslo.yaml` anchors write latency.

## REST resources
- `/v1/data-pipeline/connectors/{connector_id}/runs` starts connector work.
- `/v1/data-pipeline/connectors/{connector_id}/schema-drift-cases` opens drift review.
- `/v1/data-pipeline/schema-drift-cases/{case_id}/disposition` closes drift review.
- `/v1/data-pipeline/transforms/{transform_job_id}/approvals` approves transform work.
- `/v1/data-pipeline/lineage/reconciliation-cases` opens graph reconciliation.
- `/v1/data-pipeline/lineage/reconciliation-cases/{case_id}/apply` applies graph repair.
- `/v1/data-pipeline/dead-letter/{case_id}/replay-approvals` approves replay.
- `/v1/data-pipeline/replay-windows/{window_id}/cursor` advances replay cursor.
- `/v1/data-pipeline/watermarks/{watermark_id}/advance` advances CDC freshness.
- `/v1/data-pipeline/audit-exports` creates evidence exports.
- `/v1/data-pipeline/cost-attributions/{run_id}` reads transform cost.
- `/v1/data-pipeline/slo-promotions/{promotion_id}` reads promotion evidence.

## Request headers
- `Oya-Tenant-Id` is required.
- `Oya-Home-Cell` is required.
- `Oya-Principal-Id` is required.
- `Oya-Audience-Type` is required.
- `Oya-Data-Class` is required.
- `Oya-Purpose` is required.
- `Oya-Idempotency-Key` is required on mutation.
- `Traceparent` is required.
- `Oya-Pack-Overlay` is required when regulated.
- `Oya-Cedar-Decision-Id` is required after policy evaluation.
- `Oya-Audit-Target` is required on mutation.
- `Oya-DealSet-License-Id` is required when licensed connector applies.

## Command body fields
- Connector run body includes `source_object_id`.
- Connector run body includes `connector_catalog_version`.
- Connector run body includes `requested_watermark`.
- Schema drift body includes `before_schema_hash`.
- Schema drift body includes `after_schema_hash`.
- Schema drift disposition body includes `operator_review_id`.
- Transform approval body includes `transform_version_id`.
- Transform approval body includes `cost_estimate_id`.
- Lineage reconciliation body includes `ontology_snapshot_id`.
- Replay approval body includes `dead_letter_case_id`.
- Replay cursor body includes `cursor_before` and `cursor_after`.
- Watermark body includes `watermark_kind` and `watermark_value_after`.

## Response fields
- Every mutation response returns `audit_event_id`.
- Every mutation response returns `cedar_decision_id`.
- Every mutation response returns `idempotency_key`.
- Every mutation response returns `tenant_id`.
- Every mutation response returns `home_cell`.
- Connector run response returns `connector_run_id`.
- Drift response returns `schema_drift_case_id`.
- Transform response returns `transform_run_id` or `approval_id`.
- Lineage response returns `reconciliation_epoch`.
- Replay response returns `replay_custody_id`.
- Watermark response returns `watermark_status`.
- Export response returns `evidence_bundle_id`.

## Error model
- `DATA_PIPELINE_TENANT_REQUIRED` maps to 400.
- `DATA_PIPELINE_POLICY_DENIED` maps to 403.
- `DATA_PIPELINE_CONNECTOR_LICENSE_STALE` maps to 409.
- `DATA_PIPELINE_SCHEMA_DRIFT_UNRESOLVED` maps to 409.
- `DATA_PIPELINE_TRANSFORM_COST_REQUIRED` maps to 409.
- `DATA_PIPELINE_LINEAGE_EPOCH_REQUIRED` maps to 409.
- `DATA_PIPELINE_REPLAY_CUSTODY_REQUIRED` maps to 409.
- `DATA_PIPELINE_WATERMARK_BACKWARD` maps to 422.
- `DATA_PIPELINE_PACK_OVERLAY_REQUIRED` maps to 403.
- `DATA_PIPELINE_AUDIT_CHAIN_UNAVAILABLE` maps to 503.
- `DATA_PIPELINE_PROVIDER_RATE_LIMITED` maps to 429.
- `DATA_PIPELINE_CONFLICTING_IDEMPOTENCY` maps to 409.

## Cedar facts
- REST adapter maps headers into tenant facts.
- REST adapter maps path ids into resource facts.
- REST adapter maps body purpose into action facts.
- REST adapter maps pack overlay header into compliance facts.
- REST adapter maps DealSet header into license facts.
- REST adapter maps custody ids into replay facts.
- REST adapter maps lineage epoch into graph facts.
- REST adapter maps cost estimate into transform facts.
- REST adapter maps watermark kind into CDC facts.
- REST adapter maps audit target into evidence facts.
- REST adapter never lets body tenant override header tenant.
- REST adapter never lets query tenant override header tenant.

## Workflow decisions
- REST mutation starts workflow only after request validation.
- REST mutation evaluates policy before workflow start.
- REST mutation creates rollback expectation when operation is reversible.
- REST query reads tenant-scoped projections only.
- REST export requires auditor scope.
- REST replay approval does not directly replay records.
- REST lineage apply does not bypass reconciliation.
- REST transform approval does not bypass cost estimate.
- REST watermark advance does not trust provider freshness alone.
- REST DealSet license check does not expose commercial details cross-tenant.
- REST errors include runbook pointer when operator action exists.
- REST responses avoid raw payload samples.

## Failure and replay cases
- Duplicate connector run request returns prior run response.
- Duplicate transform approval returns prior approval response.
- Duplicate replay approval returns prior approval response.
- Duplicate watermark advance returns current watermark state.
- Connector adapter failure returns accepted workflow with failed step state.
- Policy denial returns refusal evidence.
- Provider rate limit returns retry-after plus runbook link.
- Schema drift conflict returns case id.
- Lineage conflict returns reconciliation case id.
- Replay custody conflict returns dead-letter case id.
- Audit-chain outage blocks mutation.
- OpenAPI validation failure emits no domain event.

## Evidence fields
- `openapi_operation_id` is mandatory.
- `http_method` is mandatory.
- `route_template` is mandatory.
- `tenant_id` is mandatory.
- `home_cell` is mandatory.
- `principal_id` is mandatory.
- `idempotency_key` is mandatory on mutation.
- `trace_id` is mandatory.
- `request_body_hash` is mandatory on mutation.
- `response_body_hash` is mandatory on mutation.
- `cedar_decision_id` is mandatory on mutation.
- `audit_event_id` is mandatory on mutation.
- `error_code` is mandatory on failure.
- `runbook_ref` is mandatory when operator action exists.
- `benchmark_pressure` is mandatory for parity summary.
- `contract_version` is mandatory.

## SLOs
- REST write p95 is measured by write-latency SLO.
- REST read p95 is measured by read-latency SLO.
- REST policy latency is measured separately.
- REST audit emission lag is measured separately.
- REST connector accepted latency excludes provider work.
- REST transform approval latency excludes worker run time.
- REST lineage apply latency includes ontology adapter time.
- REST replay approval latency excludes replay worker time.
- REST watermark advance latency includes projection update time.
- REST error rates are grouped by Data Pipeline error code.
- REST idempotency hit rate is tracked for replay safety.
- REST payload size is monitored for dead-letter export risk.

## Test cases
- OpenAPI validates connector run required headers.
- OpenAPI validates schema drift disposition body.
- OpenAPI validates transform approval cost estimate id.
- OpenAPI validates lineage reconciliation epoch.
- OpenAPI validates replay custody id.
- OpenAPI validates watermark kind.
- OpenAPI validates audit export auditor scope.
- Contract test rejects body tenant mismatch.
- Contract test rejects missing idempotency key.
- Contract test returns stable idempotent response.
- Error test maps policy deny to 403.
- Error test maps backward watermark to 422.

## Rollback
- REST contract rollback uses versioned operation ids.
- Removed fields remain accepted until deprecation window closes.
- New required fields roll back to optional with denial warning.
- Idempotency records remain valid across rollback.
- Audit evidence stores contract version.
- Workflow runs started by old contract finish under old version.
- OpenAPI rollback emits contract-retired evidence.
- SDK generation consumes rollback version.
- Operator docs point to restored route shapes.
- Benchmark metadata is not removed during rollback.
- Rollback never deletes prior audit events.
- Rollback is verified with contract tests.

## Acceptance criteria
- Every REST mutation has tenant headers.
- Every REST mutation has idempotency.
- Every REST mutation has Cedar evidence.
- Every REST mutation has audit evidence.
- Every replay route uses custody ids.
- Every lineage route uses reconciliation ids.
- Every transform route uses cost ids.
- Every watermark route uses CDC semantics.
- Every benchmark reference remains comparative.
- REST contract stays Data Pipeline-specific.

## Citation map
- `microservices/data-pipeline/contracts/local-openapi-v1.yaml`
- `microservices/data-pipeline/contracts/openapi-v1.yaml`
- `microservices/data-pipeline/PRD.md`
- `microservices/data-pipeline/ARCHITECTURE.md`
- `microservices/data-pipeline/policies/local-ingest-source-scope.cedar`
- `microservices/data-pipeline/policies/local-transform-run-control.cedar`
- `microservices/data-pipeline/policies/local-deadletter-replay-approval.cedar`
- `microservices/data-pipeline/policy/auditor-scope.cedar`
- `microservices/data-pipeline/slos/read-latency.openslo.yaml`
- `microservices/data-pipeline/slos/write-latency.openslo.yaml`
- `ADR-0105`
- `ADR-0321`

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/data-pipeline/IP-005-rest-contract-surface.md:23` - - `microservices/data-pipeline/contracts/local-openapi-v1.yaml` is the immediate REST authority.; `microservices/data-pipeline/IP-005-rest-contract-surface.md:24` - - `microservices/data-pipeline/contracts/openapi-v1.yaml` is the non-local companion contract..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-005-rest-contract-surface.md:164` - ## SLOs; `microservices/data-pipeline/IP-005-rest-contract-surface.md:165` - - REST write p95 is measured by write-latency SLO..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-005-rest-contract-surface.md:45` - - `/v1/data-pipeline/cost-attributions/{run_id}` reads transform cost.; `microservices/data-pipeline/IP-005-rest-contract-surface.md:70` - - Transform approval body includes `cost_estimate_id`..
