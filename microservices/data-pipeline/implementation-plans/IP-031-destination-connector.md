# IP-031 Data Pipeline destination connector finalization

Service: data-pipeline
Implementation plan: IP-031
Wave: 15A-DATA-PIPELINE-FINALIZER
Date: 2026-05-21
Scope path: microservices/data-pipeline/implementation-plans/IP-031-destination-connector.md
Audit source: microservices/data-pipeline/coherence-audit-2026-05-20.md
Audit finding: Section 3.9.2 names destination connectors as missing or thin.
Parity source: microservices/data-pipeline/feature-parity-matrix-2026-05-20.md
Primary ADR: microservices/data-pipeline/decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md

## Scope
- Establish `destination-connector` as the explicit bounded context for load targets.
- Keep source connector extraction under the existing `connector` context.
- Own load-attempt state, destination idempotency receipts, commit cursors, rollback bundles, and load dead letters.
- Cover warehouse destinations, lakehouse destinations, object stores, streams, ontology projections, analytics projections, and reverse-ETL targets.
- Treat destination loading as a policy-gated data movement operation, not as a hidden tail step inside extraction.
- Bind every destination load to tenant_id, tenant_class, home_cell, pack overlay, data_class, source watermark, and lineage facet evidence.
- Keep data-warehouse as substrate dependency: data-pipeline owns the load run; data-warehouse owns storage and commit cursor materialization.
- Preserve ADR-MS-001 lineage-before-transform-commit rule: output is not authoritative until destination commit and lineage evidence are sealed.
- Close audit Section 4 table row where destination connectors had no capability, SLO, runbook, or IP.
- No files outside microservices/data-pipeline/ are required for this IP.

## Interfaces
- REST command `POST /data-pipeline/actions/destination-load.open`.
- REST command `POST /data-pipeline/actions/destination-load.commit`.
- REST command `POST /data-pipeline/actions/destination-load.partial-commit`.
- REST command `POST /data-pipeline/actions/destination-load.rollback`.
- REST command `POST /data-pipeline/actions/destination-load.quarantine`.
- gRPC service `DestinationConnectorControl`.
- Event topic `oya.data.pipeline.destination.*`.
- Capability record `capabilities/destination-load-commit.yaml`.
- Policy fragment `policies/local-destination-load-scope.cedar`.
- Contract projection `contracts/destination-binding-v1.yaml`.
- Catalog projection `catalog/oya-data-pipeline-destination-connector-domain.yaml`.
- Dashboard projection `dashboards/local-domain-throughput.json` gains destination_class split.
- Runbook projection `runbooks/destination-load-rollback.md`.
- SLO projection `slos/local-destination-commit-latency.openslo.yaml`.

## Data Flow
- Source connector publishes a captured watermark through IP-030.
- Transform run, if present, seals transform_run_id and output schema fingerprint through IP-029.
- Destination load opens with tenant_id, destination_id, destination_class, connector_run_id or transform_run_id, idempotency_key, and expected schema fingerprint.
- Cedar evaluates tenant ownership, destination class permission, pack overlay, tenant_class capacity, and audit-chain availability.
- Destination adapter prepares schema or confirms existing schema version.
- Destination adapter writes data under a load_attempt_id.
- Destination returns destination_commit_cursor, rows_committed, bytes_committed, retry_count, and receipt hash.
- Data-pipeline emits lineage facet payload before marking the load authoritative.
- Data-pipeline advances the IP-030 landed watermark only after destination commit is acknowledged.
- Data-pipeline attaches dead-letter rows to load_attempt_id when commit is partial or failed.
- Data-pipeline emits audit-chain events for open, commit, rollback, quarantine, and dead-letter attach.
- Downstream IP-034 exposures receive impact events if rollback or quarantine occurs.

## Cedar Policy
- Deny destination load open when principal tenant_id differs from destination tenant_id.
- Deny destination load open when tenant_class is not `demo_trial` or `paid`.
- Deny paid-only connector package loads for demo_trial unless package grants demo_trial capability.
- Deny cross-cell destination when pack overlay prohibits payload movement.
- Deny commit without either connector_run_id or transform_run_id.
- Deny commit when schema_fingerprint_after does not match accepted drift disposition from IP-026.
- Deny commit when lineage_facet_payload is missing.
- Deny rollback without rollback_bundle_id.
- Deny reverse-ETL destination when DealSet license from IP-014 is absent or expired.
- Deny object-lake destination when BYOK requirement from compliance pack is unmet.
- Deny commit during audit-chain outage.
- Deny adapter execution when dependency lockfile from IP-036 has drifted.

## Event Shapes
- `oya.data.pipeline.destination.load_opened` carries tenant_id, tenant_class, home_cell, destination_id, destination_class, load_attempt_id, source_ref, policy_decision_id.
- `oya.data.pipeline.destination.schema_prepared` carries schema_fingerprint_before, schema_fingerprint_after, drift_case_id, adapter_version.
- `oya.data.pipeline.destination.load_committed` carries destination_commit_cursor, rows_committed, bytes_committed, commit_duration_ms, lineage_facet_id.
- `oya.data.pipeline.destination.partial_commit_detected` carries partial_rows, failed_rows, dead_letter_batch_id, retry_after_ms.
- `oya.data.pipeline.destination.dead_letter_attached` carries dead_letter_id, custody_case_id, payload_hash, replay_policy_version.
- `oya.data.pipeline.destination.load_rolled_back` carries rollback_bundle_id, prior_commit_cursor, restored_commit_cursor, watermark_repair_id.
- `oya.data.pipeline.destination.quarantined` carries quarantine_id, reason, owner_team, next_review_at.
- Every event includes correlation_id, audit_event_id, cedar_decision_id, and traceparent.

## SLO Targets
- Reuse `availability.openslo.yaml` target 0.999 for destination control-plane availability.
- Reuse `write-latency.openslo.yaml` target 0.999 for command acceptance.
- Reuse `read-latency.openslo.yaml` target 0.999 for load status reads.
- Reuse `policy-decision-latency.openslo.yaml` target 0.999 for Cedar decisions.
- Reuse `audit-emission-lag.openslo.yaml` target 0.999 for load events.
- Reuse `local-ingest-freshness.openslo.yaml` target 0.995 for captured-to-landed freshness.
- Reuse `replay-freshness.openslo.yaml` target 0.999 when rollback requires replay.
- Reuse `local-schema-drift-latency.openslo.yaml` target 0.999 before schema commit.
- Reuse `local-transform-latency.openslo.yaml` target 0.99 when transform output feeds the destination.
- Reuse `local-lineage-capture.openslo.yaml` target 0.999 before authoritative commit.
- Reuse `local-deadletter-rate.openslo.yaml` target 0.995 for failed destination rows.
- Reuse `local-quality-null-rate.openslo.yaml` target 0.999 for destination quality gates.
- Add `local-destination-commit-latency.openslo.yaml`: p95 warehouse 60s, lakehouse 30s, object-lake 5s, streaming 1s, projection 5s, reverse-ETL 10s.

## Failure Modes
- Destination API stall leaves load_attempt_id in opened state and links provider-rate-limit runbook.
- Partial write opens custody case and blocks landed watermark advancement.
- Schema mismatch opens IP-026 drift case and quarantines the load.
- Commit cursor mismatch rolls back and emits destination rollback event.
- Lineage capture failure blocks authoritative publication.
- Audit-chain failure holds commit even if destination wrote successfully.
- Cedar outage fails closed before adapter dispatch.
- Destination cell outage follows multi-region metadata-only fallback.
- Reverse-ETL vendor rate limit pauses without changing source watermark.
- Package signature failure from IP-036 blocks custom destination execution.
- Cost-budget exhaustion from IP-017 denies retry amplification.
- Exposure notification failure from IP-034 sends downstream impact to dead letter.

## Migration
- Migration starts by declaring destination-connector in manifest bounded contexts.
- Existing connector load behavior is wrapped, not deleted.
- Root IP-031 remains historical evidence; this file is the implementation-plans handoff.
- Destination classes are added behind Cedar deny-by-default policy.
- The first migration slice supports warehouse and object-lake adapters only.
- The second slice adds lakehouse and streaming adapters.
- The third slice adds ontology and analytics projection adapters.
- The fourth slice adds reverse-ETL through DealSet-aware packages.
- Historical landed watermarks are backfilled from known destination commit cursors.
- Old connector-run evidence receives load_attempt_id backrefs.
- No customer-facing tier migration exists; tenant_class stays the only customer axis.
- ADR-0248 cellular tier language is retained only as home_cell topology.

## Dependencies
- IP-001 tenant scope kernel supplies TenantScope.
- IP-002 Cedar default deny supplies policy posture.
- IP-003 ontology projection receives destination projection load events.
- IP-004 workflow templates can invoke destination load commands.
- IP-005 REST contract surface publishes destination endpoints.
- IP-006 async event surface publishes destination events.
- IP-007 gRPC internal surface publishes DestinationConnectorControl.
- IP-008 policy eval library binding evaluates destination Cedar.
- IP-009 credential sidecar binding supplies destination credentials.
- IP-010 multi-region cell layout constrains destination home_cell.
- IP-011 observability audit events emits audit evidence.
- IP-012 abuse-defence edge WAF protects public load control paths.
- IP-013 emergency services bypass does not bypass residency.
- IP-014 marketplace DealSet settlement licenses paid destination connectors.
- IP-015 data-residency pack overlays constrain cross-cell destination.
- IP-016 backfill replay worker repairs failed destination rows.
- IP-017 cost budget enforcer guards load retries.
- IP-018 capacity admission control gates load concurrency.
- IP-019 SDK generation exposes load commands to clients.
- IP-020 catalog layer registration registers destination adapter catalog rows.
- IP-021 SLO-gated promotion blocks promotion on load SLO burn.
- IP-022 chaos drill pack tests destination partial commit.
- IP-023 DPIA evidence packet records payload movement.
- IP-024 threat model control map maps destination abuse cases.
- IP-025 audit findings closeout proves destination finding closure.
- IP-026 schema drift quarantine holds unsafe destination DDL.
- IP-027 lineage graph reconciliation seals load lineage edges.
- IP-028 dead-letter replay custody owns load dead-letter replay.
- IP-029 transform cost attribution feeds destination load cost.
- IP-030 CDC freshness watermark governance advances landed watermark.

## ADR-MS-001 Binding
- ADR-MS-001 requires lineage before authoritative transform commit.
- Destination load commit is the authoritative output boundary for loaded datasets.
- The lineage operation must cite destination_commit_cursor.
- Replay must re-evaluate Cedar before repeating destination side effects.
- Metrics must not include raw tenant identifiers; tenant details stay in signed audit evidence.
- Append-only run records are preferred over mutable job status.

## Acceptance Gates
- Gate 1: manifest declares destination-connector as a bounded context.
- Gate 2: OpenAPI, gRPC, and AsyncAPI all expose destination load operations.
- Gate 3: Cedar tests deny cross-tenant and cross-cell loads.
- Gate 4: destination_load_run cannot commit without lineage facet payload.
- Gate 5: landed watermark advances only after load_committed.
- Gate 6: rollback restores prior destination_commit_cursor and emits audit evidence.
- Gate 7: every destination class has at least one adapter test fixture.
- Gate 8: DealSet-required reverse-ETL load is denied without license.
- Gate 9: all 12 existing OpenSLOs are referenced by the promotion checklist.
- Gate 10: local-destination-commit-latency SLO is filed before implementation starts.
- Gate 11: IP-001 through IP-030 references remain intact in this plan.
- Gate 12: remediation notes mark audit destination connector gap closed by this IP.


## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/implementation-plans/IP-031-destination-connector.md:22` - - Close audit Section 4 table row where destination connectors had no capability, SLO, runbook, or IP.; `microservices/data-pipeline/implementation-plans/IP-031-destination-connector.md:39` - - SLO projection `slos/local-destination-commit-latency.openslo.yaml`..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/implementation-plans/IP-031-destination-connector.md:84` - - Reuse `audit-emission-lag.openslo.yaml` target 0.999 for load events.; `microservices/data-pipeline/implementation-plans/IP-031-destination-connector.md:139` - - IP-017 cost budget enforcer guards load retries..
