# IP-026 Data Pipeline connector schema-drift quarantine

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-026-connector-schema-drift-quarantine.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Own schema drift quarantine inside Data Pipeline rather than Connect.
- Detect additive, narrowing, widening, rename, type, nullability, and semantic drift.
- Keep tenant, connector, source object, and capture cursor on every drift decision.
- Block writes that would corrupt warehouse, ontology, lineage, or replay projections.
- Preserve a reviewable sample bundle before any automatic adaptation.
- Make drift quarantine visible to operators before replay or transform approval.
- Treat Fivetran automated schema evolution as a parity benchmark, not an authority model.
- Treat Airbyte Cloud catalog refresh as a pressure test for connector-level review.
- Treat Hevo and Stitch lightweight ELT behavior as a benchmark for fast user feedback.
- Treat Matillion, Talend Cloud, Informatica IICS, and Estuary Flow as enterprise drift-control pressure.

## Prerequisites
- Read `microservices/data-pipeline/PRD.md` for connector, pipeline-run, transform, lineage, and replay contexts.
- Read `microservices/data-pipeline/ARCHITECTURE.md` for ADR-0105 layer ownership.
- Read `microservices/data-pipeline/capabilities/schema-drift-hold.yaml`.
- Read `microservices/data-pipeline/runbooks/schema-drift-quarantine.md`.
- Read `microservices/data-pipeline/runbooks/local-schema-drift-lag.md`.
- Read `microservices/data-pipeline/runbooks/local-quarantine-release-review.md`.
- Read `microservices/data-pipeline/slos/local-schema-drift-latency.openslo.yaml`.
- Read `microservices/data-pipeline/slos/local-ingest-freshness.openslo.yaml`.
- Read `microservices/data-pipeline/policies/local-quality-threshold-enforcement.cedar`.
- Read `microservices/data-pipeline/policies/local-null-rate-quarantine.cedar`.
- Read `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`.
- Read `microservices/data-pipeline/contracts/local-openapi-v1.yaml`.

## Domain model
- Aggregate: `schema_drift_quarantine_case`.
- Identity: `tenant_id + connector_id + source_object_id + drift_fingerprint`.
- Cursor: `source_snapshot_id + cdc_watermark + ingest_attempt_id`.
- Required actor: `principal_id` with `DATA_PIPELINE_OPERATOR` audience.
- Required policy decision: Cedar permit from schema-drift hold capability.
- Required evidence: before schema, after schema, sampled records, inferred risk.
- Required custody: raw source sample remains sealed until review.
- Required disposition: accept, adapt, remap, suppress, split, or reject.
- Required replay note: downstream replay cannot cross unresolved case.
- Required lineage note: lineage edges mark quarantined fields as provisional.
- Required transform note: transform approval requires drift disposition id.
- Required warehouse note: projection writes pause for impacted table path.

## Drift classification
- Additive column with nullable default routes to low-risk review.
- Additive column with sensitive-name heuristic routes to privacy review.
- Type widening routes to transform compatibility review.
- Type narrowing routes to hard quarantine.
- Nullability loosening routes to quality threshold review.
- Nullability tightening routes to backfill readiness review.
- Enum expansion routes to semantic mapping review.
- Enum contraction routes to dead-letter risk review.
- Field rename routes to lineage reconciliation review.
- Field deletion routes to replay custody review.
- Primary key mutation routes to hard quarantine.
- Watermark column mutation routes to CDC governance review.

## Implementation steps
- Add a domain transition `drift_detected -> quarantine_opened`.
- Record the current connector catalog version before opening the case.
- Record source-system response headers when available.
- Hash the before and after schema fragments with tenant-local salt.
- Bind `data_class` to each changed field before preview.
- Evaluate Cedar before sample materialization.
- Store samples in tenant home cell only.
- Emit `oya.data.pipeline.schema_drift.quarantined`.
- Block connector-run-start for the affected source object.
- Keep unrelated source objects runnable in the same connector.
- Produce transform compatibility hints without mutating transforms.
- Produce lineage provisional edge hints without committing graph repair.
- Expose an operator review command in the local OpenAPI surface.
- Expose a case-opened event in the local AsyncAPI surface.
- Require idempotency key on accept, adapt, remap, and reject commands.
- Require DealSet settlement check when drift changes licensed connector scope.
- Require pack overlay resolution before releasing regulated fields.
- Require rollback bundle before any adaptive schema promotion.

## Evidence payload
- `tenant_id` is mandatory.
- `home_cell` is mandatory.
- `connector_id` is mandatory.
- `source_object_id` is mandatory.
- `connector_catalog_version_before` is mandatory.
- `connector_catalog_version_after` is mandatory.
- `drift_fingerprint` is mandatory.
- `drift_class` is mandatory.
- `field_paths_changed` is mandatory.
- `risk_tier` is mandatory.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.
- `lineage_provisional_edge_count` is mandatory.
- `transform_impacted_count` is mandatory.
- `dead_letter_risk_count` is mandatory.
- `replay_cursor_min` is mandatory.
- `replay_cursor_max` is mandatory.
- `operator_disposition` is mandatory after closure.

## Policy gates
- Cedar denies if tenant scope is missing.
- Cedar denies if connector owner differs from caller tenant.
- Cedar denies if source object belongs to another home cell.
- Cedar denies if the drift sample requests raw PII without pack permit.
- Cedar denies if operator attempts release without reviewer separation.
- Cedar denies if DealSet coverage is stale for the connector license.
- Cedar denies if the new field conflicts with data-residency overlay.
- Cedar denies if quarantine closure lacks audit-chain emission.
- Cedar denies if transform approval bypasses drift disposition.
- Cedar denies if lineage repair attempts direct graph mutation before reconciliation.

## Benchmark displacement
- Fivetran parity means schema change visibility and downstream impact preview.
- Airbyte Cloud parity means catalog diff review and connector-specific handling.
- Hevo parity means low-friction operator prompts for simple additive drift.
- Stitch parity means lightweight extraction must still emit Oyatie custody evidence.
- Matillion parity means transformation impact is first-class, not a footnote.
- Talend Cloud parity means governed mapping and steward review are required.
- Informatica IICS parity means enterprise metadata governance is preserved.
- Estuary Flow parity means real-time capture drift respects watermarks and derivations.
- None of these vendors displace ADR-0321 tenant, Cedar, audit, or rollback anchors.
- Vendor names stay in evidence as benchmarks, never as domain object names.

## Failure handling
- If catalog refresh fails, retain the previous accepted catalog.
- If sample capture fails, open a case with `sample_unavailable`.
- If Cedar is unavailable, fail closed for mutation and leave reads degraded.
- If audit-chain is unavailable, hold release commands.
- If lineage service is unavailable, keep provisional edge payload locally.
- If transform registry is unavailable, block adaptive release.
- If warehouse projection already wrote partial data, open replay custody case.
- If CDC cursor moved past quarantined data, freeze affected replay window.
- If operator closes the wrong case, require reversal with fresh approval.
- If vendor API changes names without types, force lineage reconciliation.

## Tests and evidence
- Contract test: schema drift case create request rejects missing tenant.
- Contract test: release command rejects missing disposition.
- Policy test: cross-tenant sample access is denied.
- Policy test: regulated field release requires pack overlay permit.
- Replay test: unresolved case blocks replay cursor advance.
- Lineage test: provisional edges do not enter durable graph.
- Transform test: impacted transform cannot be approved without disposition.
- SLO test: local-schema-drift-latency burn opens runbook link.
- Audit test: quarantine opened and closed events share correlation id.
- Regression test: additive safe field does not block unrelated connector objects.

## Rollback
- Roll back by restoring the last accepted connector catalog version.
- Preserve quarantine case history after rollback.
- Mark replay windows touched by the rejected schema as blocked.
- Emit `oya.data.pipeline.schema_drift.rollback_completed`.
- Revoke temporary sample access grants.
- Recompute lineage provisional edges after rollback.
- Recompute transform impact hints after rollback.
- Keep DealSet license decisions immutable in the audit chain.
- Attach rollback evidence to `runbooks/schema-drift-quarantine.md`.
- Do not delete dead-letter items created during the failed schema window.

## Acceptance criteria
- Operators can see exactly which fields changed and why the case opened.
- Data writes for impacted source object pause until disposition.
- Unimpacted source objects continue under the same tenant and connector.
- Every closure path emits audit-chain evidence.
- Every closure path references Cedar decision id.
- Every closure path references local contract surface.
- Every closure path references local runbook surface.
- Every closure path references at least one benchmark pressure.
- Every closure path respects ADR-0321.
- The IP remains specific to Data Pipeline and does not reassign work to Connect.

## Citation map
- `microservices/data-pipeline/PRD.md` anchors product scope.
- `microservices/data-pipeline/ARCHITECTURE.md` anchors layer ownership.
- `microservices/data-pipeline/capabilities/schema-drift-hold.yaml` anchors capability.
- `microservices/data-pipeline/runbooks/schema-drift-quarantine.md` anchors operator response.
- `microservices/data-pipeline/slos/local-schema-drift-latency.openslo.yaml` anchors SLO.
- `microservices/data-pipeline/contracts/local-openapi-v1.yaml` anchors command shape.
- `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml` anchors event shape.
- `microservices/data-pipeline/policies/local-null-rate-quarantine.cedar` anchors policy.
- `microservices/data-pipeline/policies/local-quality-threshold-enforcement.cedar` anchors quality gate.
- `ADR-0105` anchors layer map.
- `ADR-0314` anchors DealSet settlement.
- `ADR-0321` anchors documentation-rigor answer scope.

## Operator review prompts
- Reviewer asks whether the drift changes destination table shape.
- Reviewer asks whether the drift changes source object identity.
- Reviewer asks whether the drift changes CDC watermark interpretation.
- Reviewer asks whether the drift changes regulated data classification.
- Reviewer asks whether the drift changes connector license scope.
- Reviewer asks whether the drift changes transform approval requirements.
- Reviewer asks whether the drift changes lineage reconciliation requirements.
- Reviewer asks whether the drift changes replay custody requirements.
- Reviewer asks whether the drift changes data-residency routing.
- Reviewer asks whether the drift changes SLO alert thresholds.
- Reviewer asks whether the drift changes dashboard disclosure.
- Reviewer asks whether the drift changes operator runbook path.
- Reviewer asks whether the drift can be accepted without backfill.
- Reviewer records the answer set before closure.
- Reviewer signs the case with the same audit correlation id.

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/data-pipeline/IP-026-connector-schema-drift-quarantine.md:31` - - Read `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`.; `microservices/data-pipeline/IP-026-connector-schema-drift-quarantine.md:32` - - Read `microservices/data-pipeline/contracts/local-openapi-v1.yaml`..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `iceberg_snapshot`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-026-connector-schema-drift-quarantine.md:146` - - SLO test: local-schema-drift-latency burn opens runbook link.; `microservices/data-pipeline/IP-026-connector-schema-drift-quarantine.md:179` - - `microservices/data-pipeline/slos/local-schema-drift-latency.openslo.yaml` anchors SLO..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-026-connector-schema-drift-quarantine.md:110` - - Cedar denies if quarantine closure lacks audit-chain emission..
