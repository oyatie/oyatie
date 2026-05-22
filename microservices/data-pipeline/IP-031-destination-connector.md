# IP-031 Data Pipeline destination-connector bounded context

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-031-destination-connector.md
Authored: 2026-05-21
Source audit: microservices/data-pipeline/coherence-audit-2026-05-20.md §3.9.2, §3.9.3
Benchmarks: Fivetran (destination plans for Snowflake/BigQuery/Redshift/Databricks/Synapse/Postgres/Iceberg/Delta), Airbyte (destination connectors), dbt Cloud (warehouse adapters)
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0242, ADR-0243, ADR-0244, ADR-0245, ADR-0247, ADR-0248, ADR-0251, ADR-0252, ADR-0253, ADR-0254, ADR-0255, ADR-0314, ADR-0315, ADR-0316, ADR-0321, ADR-0329, ADR-0330, ADR-0331

## Objective
- Promote `destination-connector` from an implicit responsibility of the existing `connector` bounded context to a first-class named bounded context inside data-pipeline.
- Keep source ingestion (`connector`) and destination loading (`destination-connector`) separately observable so freshness, cost, dead-letter custody, and pack-overlay enforcement attribute cleanly.
- Maintain Fivetran-class destination coverage (warehouse + lakehouse + ontology + analytics + reverse-ETL targets) without forcing every loader through one polymorphic adapter.
- Bind each destination-load action to tenant scope, Cedar permit, idempotency receipt, audit-chain evidence, lineage facets, rollback bundle, and cost dimensions.
- Make destination authority readable so reviewers can answer "which side owns idempotency", "which side owns retry", "which side owns schema evolution", and "which side owns commit visibility" for any pipeline run.

## Prerequisites
- Read `microservices/data-pipeline/PRD.md` §A and §H (tenant-class section after rewrite).
- Read `microservices/data-pipeline/ARCHITECTURE.md` §C (bounded contexts) and §D (integration topology).
- Read `microservices/data-pipeline/decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md` §Decision rows.
- Read `microservices/data-pipeline/coherence-audit-2026-05-20.md` §5.1 (data-warehouse boundary finding).
- Read `microservices/data-pipeline/feature-parity-matrix-2026-05-20.md` §2 (connector inventory).
- Read `microservices/data-pipeline/capabilities/connector-run-start.yaml`.
- Read `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml` and `contracts/local-openapi-v1.yaml`.
- Read `microservices/data-warehouse/manifest.json` to confirm the destination substrate ownership boundary.
- Read `microservices/data-pipeline/IP-030-cdc-freshness-watermark-governance.md` (watermark kinds, particularly `landed`).

## Domain model
- Aggregate: `destination_load_run`.
- Identity: `tenant_id + destination_id + connector_run_id + load_attempt_id`.
- Sub-aggregate: `destination_table_binding` (one row per destination object touched by a load run).
- Required actor: `principal_id` with `DATA_PIPELINE_OPERATOR` or `oyatie.foundry.pipeline_operator` audience.
- Required policy decision: Cedar permit from `local-destination-load-scope.cedar`.
- Required substrate dependency: `data-warehouse` for warehouse classes, `ontology` for entity-projection class, `analytics` for read-optimized projection class, `cloud-storage` for object-lake class.
- Required evidence: source watermark snapshot, destination commit cursor, rows committed, bytes committed, schema fingerprint, lineage facet payload.
- Required custody: dead-letter rows attached to load attempt id, not to source watermark.
- Required disposition: committed, partially-committed, rolled-back, quarantined, abandoned.
- Required transform note: a destination load that materializes a transformed dataset references its transform_run_id.
- Required cost dimensions: bytes loaded, rows loaded, destination commit duration, destination retries, dead-letter row count.

## Destination classes
- Warehouse class: Snowflake / BigQuery / Redshift / Databricks SQL / Synapse / Postgres analytic mode / ClickHouse. Commit semantics: transactional batch.
- Lakehouse class: Delta Lake / Iceberg / Hudi / Apache Paimon / Iceberg-on-S3. Commit semantics: manifest commit.
- Object-lake class: S3 / GCS / Azure Blob / R2 / OCI Object Storage / on-prem MinIO / Ceph. Commit semantics: object PUT with idempotency.
- Streaming class: Kafka / Pulsar / Kinesis / EventHubs / NATS JetStream. Commit semantics: offset advance after batch ack.
- Ontology projection class: oyatie `ontology` µservice. Commit semantics: entity-version advancement per ADR-0245 substrate/product layering.
- Analytics projection class: oyatie `analytics` µservice read store. Commit semantics: read-model write barrier.
- Reverse-ETL class: Salesforce / HubSpot / NetSuite / Marketo / Iterable / Klaviyo / Braze / Customer.io / Slack / MS Teams / Notion / Airtable / Monday. Commit semantics: vendor API idempotency receipt.
- Custom destination class: any destination delivered via the CDK authoring workflow (see IP-037).

## Implementation steps
- Add `destination-connector` to `manifest.json` `bounded_contexts` (alongside `connector`, `pipeline-run`, `transform`, `lineage`, `replay`).
- Update `ARCHITECTURE.md` §C to declare `destination_load_run` as the destination aggregate (PRD-bespoke-rewrite IP carries the same update for PRD §C).
- Add `src/domain/destination_connector.rs` with `DestinationLoadRun`, `DestinationTableBinding`, `DestinationClass` enum, and `LoadDisposition` enum.
- Add `src/usecase/load_run.rs` exposing `load_run.open`, `load_run.commit`, `load_run.partial_commit`, `load_run.rollback`, `load_run.quarantine`, `load_run.abandon` commands.
- Add `src/adapter/destination/<class>/mod.rs` per destination class with a stable `DestinationAdapter` trait.
- Add `local-destination-load-scope.cedar` to `policies/` declaring tenant + destination ownership + commit-class permission.
- Add `oya.data.pipeline.destination.load_opened` and `oya.data.pipeline.destination.load_committed` events to AsyncAPI surface.
- Add `oya.data.pipeline.destination.load_rolled_back` event with rollback_bundle_id correlation.
- Add `oya.data.pipeline.destination.dead_letter_attached` event with custody id correlation.
- Wire `landed` watermark advancement (IP-030 §watermark kinds) to a destination_load_run commit, not to a connector pull alone.
- Add a runbook `destination-load-rollback.md` describing rollback-bundle replay for each destination class.
- Add an SLO `local-destination-commit-latency.openslo.yaml` with class-specific p95 budgets (warehouse 60s, lakehouse 30s, object-lake 5s, streaming 1s, ontology 2s, analytics 5s, reverse-ETL 10s).
- Add a capability yaml `capabilities/destination-load-commit.yaml`.
- Add a catalog row `catalog/oya-data-pipeline-destination-connector-domain.yaml`.

## Evidence payload
- `tenant_id` is mandatory.
- `home_cell` is mandatory.
- `destination_id` is mandatory.
- `destination_class` is mandatory.
- `connector_run_id` is mandatory for source-driven loads.
- `transform_run_id` is mandatory for transform-driven loads.
- `load_attempt_id` is mandatory.
- `source_watermark_snapshot` is mandatory.
- `destination_commit_cursor` is mandatory after commit disposition.
- `rows_committed` is mandatory.
- `bytes_committed` is mandatory.
- `schema_fingerprint_before` is mandatory.
- `schema_fingerprint_after` is mandatory.
- `lineage_facet_payload` is mandatory.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.
- `rollback_bundle_id` is mandatory for rollback and quarantine dispositions.
- `dead_letter_custody_ids` is mandatory when dead-letter rows attach.
- `cost_dimensions` is mandatory (bytes_loaded, rows_loaded, commit_duration_ms, retry_count, dead_letter_count).

## Authority allocation (resolves audit §5.1 cross-microservice finding)
- Idempotency: data-pipeline owns the idempotency receipt at the `load_attempt_id` level. data-warehouse may also enforce destination-side idempotency; data-pipeline must accept the destination receipt and never double-commit on retry.
- Retry: data-pipeline owns retry policy (exponential backoff, retry budget, retry-after honoring). data-warehouse never retries on behalf of data-pipeline.
- Schema evolution: negotiated. data-pipeline drives the change (drift quarantine from IP-026); data-warehouse executes the destination DDL once disposition is signed and adapter capability supports it.
- Commit visibility: data-warehouse owns the destination commit cursor. data-pipeline reads it and stores it as `destination_commit_cursor` evidence.
- Lineage emission: data-pipeline owns lineage facet emission (per ADR-MS-001).
- Cost attribution: data-pipeline owns load-job cost (bytes loaded, commit duration); data-warehouse owns storage cost.

## Policy gates
- Cedar denies load_run.open without tenant scope.
- Cedar denies load_run.open if destination ownership differs from caller tenant.
- Cedar denies load_run.open if destination class is restricted by pack overlay (e.g., KR-PIPA may block cross-jurisdiction warehouse destinations).
- Cedar denies load_run.commit without an active connector_run_id or transform_run_id.
- Cedar denies load_run.commit if schema_fingerprint_after diverges from the accepted catalog version (IP-026 disposition required).
- Cedar denies load_run.commit if dead-letter rows exceed the policy threshold without operator review.
- Cedar denies load_run.rollback without a present rollback_bundle_id.
- Cedar denies load_run.quarantine without a custody case in `replay` context.
- Cedar denies destination-class promotion without ADR-0254 deployment shape evidence (K8s + Cloud Hypervisor adapter binding).
- Cedar denies destination commit when audit-chain is unavailable.

## Benchmark displacement
- Fivetran destination parity means warehouse + lakehouse + object-lake load semantics are first-class, not a footnote on the connector concept.
- Airbyte destination parity means each destination class names its commit semantics and dead-letter behavior independent of source pull.
- dbt Cloud parity means transformed datasets land as destination_load_run rows whose source is a transform_run_id rather than a connector_run_id, preserving lineage attribution.
- Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, and Estuary Flow remain reference pressure but do not displace ADR-0245 substrate-vs-product layering.
- Oyatie adds: tenant-scoped destination ownership, Cedar default-deny on class-restricted destinations, ADR-0251 pack-aware destination residency, ADR-0252 HLC-stamped commit cursors, ADR-0253 HTTP/3-default destination dispatch, ADR-0314 marketplace DealSet binding for licensed destination connectors.

## Failure handling
- If destination API stalls, hold the load attempt at `opened` state and link `runbooks/provider-rate-limit.md`.
- If destination commit fails after partial write, open a custody case and route to `runbooks/dead-letter-drain.md`.
- If destination schema diverges mid-load, abort the attempt, freeze the affected `landed` watermark (IP-030), and open an IP-026 drift case.
- If Cedar is unavailable, fail closed: no load_run.commit; emit refusal evidence.
- If audit-chain is unavailable, hold the commit and surface the degraded banner on the dashboard.
- If a class adapter is removed, mark every active load_run for that class as `abandoned` and require manual rollback bundle review.
- If a regional outage isolates the destination home cell, follow `multi-region.md` and emit cell-fence evidence.
- If a destination's idempotency receipt diverges from data-pipeline's load_attempt_id, hard-quarantine the load and require human reconciliation.

## Tests and evidence
- Unit test: load_run state machine rejects commit without rows_committed > 0.
- Unit test: load_run state machine rejects rollback without rollback_bundle_id.
- Contract test: `load_run.open` rejects missing destination_class.
- Contract test: `load_run.commit` rejects mismatched schema fingerprint.
- Policy test: cross-tenant destination access denied.
- Policy test: pack-restricted destination class denied for non-pack tenant.
- Replay test: destination_load_run rollback cleanly restores destination_commit_cursor.
- Lineage test: lineage facet payload carries source connector_run_id and transform_run_id correctly.
- SLO test: `local-destination-commit-latency` burn opens the destination-load-rollback runbook.
- Audit test: load_opened, load_committed, and load_rolled_back share correlation id.
- Cost test: cost_dimensions roll up to tenant + destination_class + cell + pack in capacity-model projections.

## Rollback
- Roll back by creating a `rolled_back` disposition with a rollback_bundle_id.
- Preserve the forward destination_commit_cursor as evidence.
- Recompute `landed` watermark (IP-030) downward to the prior commit cursor.
- Recompute lineage-applied watermark downward to match.
- Mark dead-letter rows in custody with `post_rollback_replay_required = true`.
- Emit `oya.data.pipeline.destination.load_rolled_back`.
- Preserve DealSet licence decisions (commercial obligations stay; only data state rolls back).
- Link rollback to `runbooks/destination-load-rollback.md`.
- Notify dependent `transform` and `analytics` projections so they reschedule their downstream barriers.

## Acceptance criteria
- `destination-connector` is declared in manifest.json `bounded_contexts`.
- ARCHITECTURE.md §C lists `destination_load_run` as the aggregate.
- PRD §C and §D enumerate destination commands (open, commit, partial_commit, rollback, quarantine, abandon).
- Cedar policies `local-destination-load-scope.cedar` and `local-destination-class-restriction.cedar` exist.
- Six destination class adapters (warehouse, lakehouse, object-lake, streaming, ontology, analytics, reverse-ETL) exist as trait implementations with at least one concrete sub-class implemented per class.
- IP-030 `landed` watermark advances through `load_run.commit` and never through a connector pull alone.
- IP-026 drift case can hold a `load_run.open` request.
- Authority allocation (§Authority allocation) is referenced in `contracts/data-pipeline-v1.proto` doc comments on load operations.
- Cross-microservice contract finding from coherence-audit §5.1 is resolved: data-pipeline + data-warehouse boundary is published in `contracts/destination-binding-v1.yaml`.

## Citation map
- `microservices/data-pipeline/coherence-audit-2026-05-20.md` §3.9 anchors the gap.
- `microservices/data-pipeline/feature-parity-matrix-2026-05-20.md` §2 anchors the connector inventory.
- `microservices/data-pipeline/IP-030-cdc-freshness-watermark-governance.md` anchors the `landed` watermark.
- `microservices/data-pipeline/IP-026-connector-schema-drift-quarantine.md` anchors drift hold.
- `microservices/data-pipeline/decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md` anchors lineage facet emission.
- `ADR-0245` anchors substrate-vs-product layering (destination substrate is data-warehouse; data-pipeline owns the load run).
- `ADR-0248` anchors cellular shape for destination home cell.
- `ADR-0251` anchors pack-overlay restrictions for destination residency.
- `ADR-0253` anchors HTTP/3 default for destination dispatch.
- `ADR-0314` anchors marketplace DealSet for licensed destination connectors.
- `ADR-0321` anchors documentation-rigor answer scope.
- `ADR-0329`, `ADR-0330`, `ADR-0331` anchor wave-15A remediation discipline.

## Operator review prompts
- Reviewer asks whether destination_class is correctly named.
- Reviewer asks whether source watermark snapshot matches commit cursor expectations.
- Reviewer asks whether pack overlay permits destination residency.
- Reviewer asks whether DealSet license covers the destination class.
- Reviewer asks whether dead-letter rows fall under custodial replay or operator quarantine.
- Reviewer asks whether transform_run_id is the correct upstream (vs connector_run_id).
- Reviewer asks whether destination commit cursor was advanced atomically.
- Reviewer asks whether rollback bundle restores both schema and data.
- Reviewer asks whether downstream `analytics` and `ontology` projections were notified.
- Reviewer signs the load run case with the same audit correlation id.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/data-pipeline/IP-031-destination-connector.md:24` - - Read `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml` and `contracts/local-openapi-v1.yaml`.; `microservices/data-pipeline/IP-031-destination-connector.md:157` - - Authority allocation (§Authority allocation) is referenced in `contracts/data-pipeline-v1.proto` doc comments on load operations..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `14400s` RTO p99 and `900s` RPO p99.
- Applicable compliance pack floor: `KR-PIPA-2023-amendment` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=14400`, `rpo_p99_seconds=900`, `multi_region_required=false`, `drill_cadence_required=semi-annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-031-destination-connector.md:63` - - Add an SLO `local-destination-commit-latency.openslo.yaml` with class-specific p95 budgets (warehouse 60s, lakehouse 30s, object-lake 5s, streaming 1s, ontology 2s,...; `microservices/data-pipeline/IP-031-destination-connector.md:122` - - If a regional outage isolates the destination home cell, follow `multi-region.md` and emit cell-fence evidence..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-031-destination-connector.md:12` - - Keep source ingestion (`connector`) and destination loading (`destination-connector`) separately observable so freshness, cost, dead-letter custody, and pack-overlay...; `microservices/data-pipeline/IP-031-destination-connector.md:14` - - Bind each destination-load action to tenant scope, Cedar permit, idempotency receipt, audit-chain evidence, lineage facets, rollback bundle, and cost dimensions..
