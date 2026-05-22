---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-data-warehouse
microservice: data-warehouse
status: wave-15a-remediated
date: 2026-05-21
authored_by: solo-owner-data-warehouse
remediation_wave: Wave-15A-DATA-WAREHOUSE-FIX
owner_team: axis-data-warehouse + council-product
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0145
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0251
  - ADR-0252
  - ADR-0253
  - ADR-0254
  - ADR-0314
  - ADR-0315
  - ADR-0316
  - ADR-0321
  - ADR-0322
  - ADR-0328
  - ADR-0329
  - ADR-0330
  - ADR-0331
  - ADR-0337
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
companion_docs:
  - microservices/data-warehouse/ARCHITECTURE.md
  - microservices/data-warehouse/compliance.md
  - microservices/data-warehouse/manifest.json
  - microservices/data-warehouse/README.md
  - microservices/data-warehouse/REMEDIATION-NOTES-2026-05-21.md
planned_enforcement_ref: oya-governance-data-warehouse-doc-suite
tenant_class_model:
  classes: [demo_trial, paid]
  paid.billing_components:
    - compute_credits
    - storage_bytes
    - egress_gb
    - share_consumer_events
    - ml_training_units
    - streaming_ingest_events
    - vector_index_serving
    - federated_query_bytes
    - container_udf_seconds
    - time_travel_storage_days
    - fail_safe_storage_days
---

# PRD — Data Warehouse

## A. Problem statement

Oyatie tenants today have to integrate with Snowflake, BigQuery, and
Databricks separately to get a complete analytical posture: Snowflake for
secure data sharing and time-travel, BigQuery for serverless slot economics
and BigQuery-ML, Databricks for the open Lakehouse substrate. Each of those
choices fragments billing, fragments residency posture, fragments lineage,
and forces the tenant to maintain three separate IAM postures and three
separate cost-governance workflows.

Data Warehouse closes that gap by holding the union envelope of the three
counterparts inside a single tenant-scoped microservice under oyatie's
shared substrate. It does **not** absorb the ontology, the workflow runtime,
the model registry, or payments. It holds analytical state and the analytical
query and write paths against that state.

The problem this PRD must solve, concretely:

1. A tenant must be able to land Snowflake-class workloads (virtual warehouses,
   time-travel, zero-copy clone, secure share, Snowpark Container UDF)
   without leaving oyatie.
2. A tenant must be able to land BigQuery-class workloads (serverless slots,
   BI-Engine-class acceleration, BigQuery-ML SQL surface, BigLake federated
   query, geospatial, vector search) without leaving oyatie.
3. A tenant must be able to land Databricks-class workloads (Delta + Iceberg +
   Hudi write, Unity-Catalog-class namespace, Photon-class engine, Auto
   Loader, Delta Live Tables, Change Data Feed) without leaving oyatie.
4. The tenant must be billed on the composable `paid.billing_components`
   model from ADR-0331, not on a retired named capability levels capacity ladder.
5. A `demo_trial` tenant must be able to evaluate the warehouse at zero
   marginal cost with hard caps, no Cedar bypass, no sharing, no vector
   search, no Container UDF.

## B. Target users (six distinct personas, each with distinct acceptance)

### B.1 Marcus Chen — operations owner at a 600-person B2B SaaS

- Role: VP Engineering / head of platform at a 600-person B2B SaaS.
- Tenant class: `paid` with `compute_credits` + `storage_bytes` +
  `share_consumer_events` enabled.
- Wants: predictable monthly burn, ability to share governed datasets with
  three downstream customers without ops overhead, time-travel for
  accidental-delete recovery.
- Distinct acceptance: Marcus can publish a governed share with row-level
  filter, set a 14-day time-travel window, and see the
  `share_consumer_events` burn line item in `cost-budget.md`'s burn-rate
  dashboard within 5 minutes of consumer reads.

### B.2 Yejin Park — side-business owner on `demo_trial`

- Role: solo SaaS founder evaluating oyatie alongside her day job.
- Tenant class: `demo_trial`.
- Wants: to run her analytical queries to validate the product fit without
  signing a contract; to convert to `paid` later without losing data.
- Distinct acceptance: Yejin's queries are admitted within the demo_trial
  caps (≤ 100M bytes per query, ≤ 50 GiB total storage, 0 share consumers);
  her conversion to `paid` is a metadata flip that preserves all `dataset`
  rows and column lineage with no re-ingest.

### B.3 Diana Alvarez — agency principal serving multiple sub-tenants

- Role: founder of a 40-person data-engineering agency with 18 client
  workspaces (multi-tenant on her oyatie account).
- Tenant class: `paid` with `compute_credits` + `storage_bytes` +
  `share_consumer_events` + `federated_query_bytes`.
- Wants: clear per-client cost attribution; per-client share isolation;
  federated query against each client's existing data lake.
- Distinct acceptance: Diana sees per-sub-tenant burn line items in the
  cost dashboard; her Cedar policy refuses cross-sub-tenant clone and
  refuses federated query to URLs not on her per-sub-tenant allow-list.

### B.4 Nadia Singh — enterprise administrator, HIPAA + KR-PIPA dual pack

- Role: head of cloud platform at a US-HQ healthcare company with a Seoul
  subsidiary.
- Tenant class: `paid` with `compute_credits` + `storage_bytes` +
  `egress_gb` + `time_travel_storage_days` + `fail_safe_storage_days` +
  `ml_training_units`.
- Pack overlay: `[HIPAA-2024, KR-PIPA, SOC-2]`.
- Wants: provable in-region residency for KR-PIPA data; CMK/BYOK encryption
  binding; 90-day time-travel + 35-day fail-safe; HIPAA audit-chain export
  passing OCR audit.
- Distinct acceptance: KR-PIPA data lives in `ap-northeast-2` and never
  egresses; CMK key references are scoped to Nadia's tenant; HIPAA
  audit-chain export emits within 24 hours of an audit request; PHI
  columns are masked-by-default unless Cedar allows.

### B.5 Omar Watkins — SRE on call for warehouse incidents

- Role: SRE responsible for warehouse availability and incident response.
- Tenant class: principal-level access to operator-class commands across
  tenants.
- Wants: a kill-cost-ledger row for every `job.kill` he issues; a runbook
  per failure mode; an emergency-services-bypass break-glass with
  two-person rule and 24-hour auto-expire.
- Distinct acceptance: every kill emits `cost_attribution.partial` with
  bytes-scanned + slot-seconds-up-to-kill; the bypass logs to the
  audit-chain with both principal IDs; bypass auto-expires.

### B.6 Hana Mori — auditor tracing policy decisions across tenants

- Role: external auditor performing a SOC-2 or HIPAA control audit.
- Tenant class: read-only auditor scope; cannot mutate or share.
- Wants: time-travel access to historical query state; audit-chain export
  signed by oyatie; column-level lineage trace for any disputed row.
- Distinct acceptance: Hana can query the warehouse at a past timestamp
  within the tenant's purchased time-travel window without paying for new
  `compute_credits` (auditor cost is borne by the audited tenant); her
  reads emit auditor-class audit-chain rows that are flagged as
  read-only.

## C. User stories (24 stories, one per persona × major capability, each with distinct acceptance)

US-001 — Marcus, virtual warehouse sizing
- As Marcus, I want to spin a multi-cluster virtual warehouse of size
  `MEDIUM` with auto-suspend at 60 s idle, so my interactive analyst
  pool gets sub-second cold-start without paying for idle compute.
- Acceptance: `POST /v1/warehouses` with `size=MEDIUM`, `auto_suspend_seconds=60`
  returns a warehouse handle within 250 ms; suspend triggers within 60 s of
  idle; resume completes < 800 ms p99 on a warm pool.

US-002 — Marcus, governed share with row-level filter
- As Marcus, I want to publish a governed share of `orders_2026` to a
  customer's reader-account with a row-level filter `region = 'US'`, so
  the customer sees only their slice.
- Acceptance: `POST /v1/shares` succeeds within 500 ms; the consumer
  account reads only US rows; consumer reads accrue to Marcus's
  `share_consumer_events`; a Cedar deny on the filter cannot be bypassed
  by directly addressing the underlying delta path.

US-003 — Marcus, 14-day time-travel
- As Marcus, I want a 14-day time-travel window on his `customer_profile`
  table, so an accidental UPDATE can be reverted by query within two
  weeks.
- Acceptance: `time_travel_storage_days=14` accrues to Marcus's
  billing line item; `SELECT * FROM customer_profile AT(TIMESTAMP =>
  '2026-05-07T12:00:00Z')` returns the table as of that instant; query
  cost equals the at-rest scan cost without any restore fee.

US-004 — Yejin, demo_trial query admission
- As Yejin on `demo_trial`, I want my 80-million-byte query to run; I
  want my 200-million-byte query to be refused with a clear cap message.
- Acceptance: query under 100M bytes runs; query over 100M bytes is
  refused with `tenant_class_cap_exceeded` and a pointer to the upgrade
  flow; no `compute_credits` are charged.

US-005 — Yejin, convert to paid without losing data
- As Yejin, I want to upgrade from `demo_trial` to `paid` and find all
  my datasets, schema, and column lineage preserved.
- Acceptance: a `POST /v1/tenants/{id}/class` transition from
  `demo_trial` to `paid` is metadata-only (no data re-ingest); all
  `dataset_catalog` and `column_lineage_graph` rows survive; old caps
  fall away; new `paid.billing_components` accrual begins at the
  transition timestamp.

US-006 — Diana, per-sub-tenant cost attribution
- As Diana, I want her cost dashboard to show one row per sub-tenant
  with `compute_credits`, `storage_bytes`, `share_consumer_events`, and
  `federated_query_bytes` broken out.
- Acceptance: the cost dashboard returns one row per sub-tenant within
  500 ms; numbers reconcile to the audit-chain `cost_attribution.*`
  events within ± 1 cent per billing period.

US-007 — Diana, federated query allow-list
- As Diana, I want her federated-query Cedar policy to allow scan only
  on a per-sub-tenant URL allow-list; any URL outside the allow-list
  must deny.
- Acceptance: `local-federated-query-target.cedar` refuses
  `ExternalTable::scan` for off-list URLs; the deny event lands in the
  audit chain; on-list scan returns rows.

US-008 — Diana, cross-sub-tenant clone refusal
- As Diana, I want a `database.clone` from sub-tenant A's database to
  sub-tenant B's namespace to be refused.
- Acceptance: the clone is refused with
  `cross_tenant_clone_forbidden`; even if Diana herself owns both
  sub-tenants under her account, the Cedar boundary holds.

US-009 — Nadia, KR-PIPA in-region residency
- As Nadia, I want her KR-tenant data to live only in `ap-northeast-2`
  and to refuse retention-tier transitions that would egress.
- Acceptance: `local-freshness-tier-control.cedar` refuses a transition
  whose destination region is not in `ap-northeast-2` or a sibling;
  the refusal lands in the audit chain; a forced override requires
  Nadia + an SRE + a logged residency-pack waiver.

US-010 — Nadia, CMK/BYOK key binding
- As Nadia, I want her warehouse data at rest encrypted under a KMS
  key her organization owns; key rotation must complete within 24 h of
  her rotation request.
- Acceptance: warehouse storage is encrypted under a `cloud-kms` key
  reference owned by Nadia's tenant; rotation emits a `key_rotation.requested`
  event and a `key_rotation.completed` event within 24 h with a re-encrypted
  pointer; the new pointer covers all `storage_bytes` accrued.

US-011 — Nadia, 90-day time-travel + 35-day fail-safe
- As Nadia, I want a 90-day time-travel window + 35-day fail-safe on her
  PHI tables, so a HIPAA audit can be answered for any moment in the
  last 125 days.
- Acceptance: `time_travel_storage_days=90` and
  `fail_safe_storage_days=35` both accrue to Nadia's billing; a query
  at -125 days runs (within fail-safe via SRE break-glass) or
  -89 days runs (within time-travel, no break-glass).

US-012 — Nadia, SQL-callable ML model train
- As Nadia, I want to run `CREATE MODEL` on her warehouse tables and
  have model artefacts land in the `intelligence` µservice registry,
  with `ml_training_units` accruing to her tenant.
- Acceptance: `CREATE MODEL` returns within the training-time budget;
  the model registers in `intelligence`; `ml_training_units` accrual
  matches the trainer's reported unit count within ± 1 %.

US-013 — Omar, kill-cost ledger
- As Omar, I want every query I kill to emit a `cost_attribution.partial`
  event with bytes-scanned-up-to-kill and slot-seconds-spent.
- Acceptance: `POST /v1/queries/{id}/kill` returns within 200 ms; the
  audit chain row carries `bytes_scanned_up_to_kill` and
  `slot_seconds_to_kill`; the cost dashboard reflects the partial within
  60 s.

US-014 — Omar, emergency-services-bypass two-person rule
- As Omar, I want the emergency break-glass to require two principals
  and to auto-expire in 24 hours.
- Acceptance: a one-principal break-glass attempt is refused; a
  two-principal break-glass with valid Cedar attestations is granted
  for ≤ 24 h; expiry triggers without manual revoke.

US-015 — Omar, kill admission control
- As Omar, I want to be able to issue a region-wide query kill in case
  of a runaway workload; the kill must complete within 10 s for at
  least the 50 highest-cost queries.
- Acceptance: `POST /v1/queries/kill-all?region=us-east-1` issues
  individual kills; p99 kill-to-acknowledged ≤ 10 s for the top 50
  queries by `slot_seconds_to_now`.

US-016 — Hana, time-travel audit query
- As Hana the auditor, I want to read the warehouse state at a past
  timestamp inside the tenant's purchased window without paying for
  new `compute_credits`.
- Acceptance: auditor-class principal does not accrue
  `compute_credits` to its own tenant; the audited tenant accrues the
  cost; the audit-chain marks the row `auditor_origin=true`.

US-017 — Hana, audit-chain signed export
- As Hana, I want to export the last 12 months of audit-chain rows for
  a given tenant in a single signed bundle.
- Acceptance: `POST /v1/audit-chain/export` returns a signed manifest
  within 30 minutes for ≤ 1 TB of audit rows; signature verifiable via
  oyatie public key.

US-018 — Hana, column-level lineage trace
- As Hana, I want to trace a specific row in a published `paid_invoices`
  dataset back to its source-system fields.
- Acceptance: `GET /v1/lineage/columns?dataset=paid_invoices&column=net_amount`
  returns the per-version source-system chain within 1 s; immutability
  invariant holds across schema evolution events.

US-019 — Marcus, Delta lake write
- As Marcus, I want to write a Delta table to oyatie storage and run
  OPTIMIZE/Z-ORDER.
- Acceptance: `lake-table-write` accrues `storage_bytes` +
  `compute_credits`; `delta-optimize-zorder` accrues
  `compute_credits` only; Z-ORDER metadata appears in `delta_log`.

US-020 — Diana, Iceberg snapshot + cross-cloud federation
- As Diana, I want her Iceberg snapshot pointer to be readable by both
  a BigLake-style external query from AWS and a Photon-class engine in
  GCP without copying data.
- Acceptance: `iceberg-metadata-register` accepts an Iceberg pointer;
  external scans from a different cloud's compute engine succeed
  without re-ingest.

US-021 — Nadia, Auto Loader streaming ingest
- As Nadia, I want her HL7 / FHIR event stream to land in a Delta
  retired-basic table with schema inference and CDF on.
- Acceptance: `auto-loader-stream-ingest` accrues
  `streaming_ingest_events`; schema infers from the first 1000 events;
  CDF emits the per-event delta for downstream subscribers.

US-022 — Marcus, Delta Live Tables declarative pipeline
- As Marcus, I want to author a DLT pipeline that takes his retired-basic
  table to retired-standard to canonical with `expect_or_fail` quality rules.
- Acceptance: `dlt-pipeline-declare` accepts the YAML; expectations
  emit `dlt.expectation.violated` events; failed rows quarantine in a
  side table.

US-023 — Marcus, change-data feed subscribe from a cross-µservice
  consumer
- As Marcus, I want his `payments` µservice to subscribe to CDF for the
  `orders_2026` delta table and react in real time.
- Acceptance: `change-data-feed-subscribe` accrues
  `streaming_ingest_events`; per-event lag SLO ≤ 5 s p99.

US-024 — Nadia, vector search on PHI-masked semantic layer
- As Nadia, I want vector similarity search over a semantic layer that
  masks PHI; the search must return semantic neighbors without
  unmasking PHI.
- Acceptance: `vector-index-serve` accrues `vector_index_serving`;
  Cedar refuses unmasked field projection; the response surfaces
  vector distance + masked text only.

## D. Functional requirements

The FR family is differentiated per primitive, per F-D2-03 remediation. Each
FR is bound to one Cedar fragment, one contract route, and at least one
SLO.

### D.1 Warehouse query (tenant-olap)

- FR-Q-001 `warehouse-query-run.submit` must accept tenant scope, principal,
  audience type, purpose, data class, pack overlay, idempotency key, trace
  context, audit-chain target, and an optional `warehouse_size` hint
  (`XS`..`6XL`).
- FR-Q-002 `warehouse-query-run.submit` must refuse if the tenant's
  `compute_credits` balance is zero with `tenant_credit_exhausted`.
- FR-Q-003 `warehouse-query-run.submit` must refuse if `demo_trial` and
  bytes-estimate > 100M with `tenant_class_cap_exceeded`.
- FR-Q-004 `warehouse-query-run.cancel` must emit
  `cost_attribution.partial` within 5 s.
- FR-Q-005 `warehouse-query-run.profile` must return query profile within
  500 ms p99 for queries < 1 hour old.

### D.2 Dataset (lineage immutability)

- FR-D-001 `dataset.publish` must refuse if column-completeness < 99.5 %
  for `paid` or < 95 % for `demo_trial`.
- FR-D-002 `dataset.evolveSchema` must additive-only by default; a
  non-additive change requires two-person approval.
- FR-D-003 `dataset.snapshot` must complete metadata snapshot within 2 s
  regardless of dataset row count (metadata-only).
- FR-D-004 `dataset.clone` (zero-copy) must complete within 1 s and must
  refuse cross-`tenant_id` clones.
- FR-D-005 `dataset.timeTravel` must accept `AT(TIMESTAMP=…|VERSION=…|
  STATEMENT=…)`; resolve within the tenant's purchased window or refuse with
  `time_travel_window_exceeded`.

### D.3 Warehouse job (admission + kill)

- FR-J-001 `job.admit` must check `compute_credits` budget before queue
  entry, never start-then-throttle.
- FR-J-002 `job.kill` must emit `cost_attribution.partial` with
  bytes-scanned-up-to-kill + slot-seconds.
- FR-J-003 `job.resize` must respect tenant's `warehouse_size_max` cap.
- FR-J-004 `job.spillToCold` must respect `pack_overlay` residency before
  spilling.

### D.4 Semantic model (masking + row access)

- FR-M-001 `model.define` must allow per-field masking-policy attachment.
- FR-M-002 `model.bindRowAccess` must accept a Cedar entity reference for
  per-row gating.
- FR-M-003 `model.refresh` must mark old rows as superseded, not delete;
  preserves lineage.

### D.5 Retention tier (residency-pinned)

- FR-T-001 `tier.transition` must refuse cross-`pack_overlay`-boundary moves
  with `residency_pack_violation`.
- FR-T-002 `tier.recall` must restore an object from cold to hot within
  the SLO of the cold-storage tier (typically 1 h p99 for cold,
  12 h p99 for frozen).
- FR-T-003 `tier.snapshotForRetention` must emit a retention-bearing
  snapshot row in the audit chain.

### D.6 Lake table (Delta + Iceberg + Hudi)

- FR-L-001 `lake.write` must enforce ACID via Delta log / Iceberg manifest /
  Hudi commit per table format.
- FR-L-002 `lake.merge` must support MERGE INTO semantics with row-level
  conflict resolution.
- FR-L-003 `lake.optimize` must perform OPTIMIZE/Z-ORDER and accrue
  `compute_credits`; refuse on `demo_trial`.
- FR-L-004 `lake.vacuum` must respect the tenant's `time_travel_storage_days`
  before reclaiming.
- FR-L-005 `lake.cdfSubscribe` must accrue `streaming_ingest_events` per
  delivered event.

### D.7 Governed share (producer + consumer)

- FR-S-001 `share.publish` must require `paid` tenant_class +
  `share_consumer_events` billing component.
- FR-S-002 `share.attachFilter` must store Cedar entity + filter expression;
  filter evaluation is at consumer read time.
- FR-S-003 `share.consumerRegister` must accept reader-account (non-tenant)
  consumers; the consumer pays nothing; the producer pays for events.
- FR-S-004 `share.revoke` must take effect within 5 s globally.

### D.8 Federated / external table

- FR-F-001 `external-table.scan` must refuse off-allow-list URLs
  (`local-federated-query-target.cedar`).
- FR-F-002 `external-table.scan` must accrue `federated_query_bytes`.

### D.9 Vector + ML

- FR-V-001 `vector-index.query` must refuse if `vector_index_serving`
  billing component is disabled.
- FR-ML-001 `sql-ml.train` must register output model in `intelligence`
  µservice via direct gRPC (ADR-0145).
- FR-ML-002 `sql-ml.predict` must call `intelligence` µservice for
  inference; warehouse never holds model weights.

### D.10 Container UDF

- FR-C-001 `container-udf.execute` must accrue `container_udf_seconds` per
  CPU-second.
- FR-C-002 `container-udf.execute` must run on a Cloud Hypervisor pod
  (ADR-0254) with tenant-isolated network policy.

## E. Non-functional requirements

### E.1 Performance (binds to `performance-benchmark-numbers-2026-05-20.md`)

- Cold-start warehouse resume: p99 ≤ 800 ms.
- Zero-copy clone: p99 ≤ 1 s (metadata only).
- Time-travel resolve: p99 ≤ 1 s for queries within the purchased window.
- CDF event delivery lag: p99 ≤ 5 s.
- Delta write commit: p99 ≤ 2 s for ≤ 100 MiB batches.
- Iceberg snapshot commit: p99 ≤ 3 s.
- Vector search (top-10, 1M vectors): p99 ≤ 50 ms.
- Federated query overhead: p99 ≤ 20 % over equivalent native scan.

### E.2 Scalability

- 10 000 concurrent queries per tenant on the largest virtual warehouse.
- 1 000 lake-table writers per tenant.
- 100 000 vector queries per second per index.
- 1 M streaming-ingest events per second per Auto-Loader stream.

### E.3 Availability

- 99.9 % monthly availability for the REST surface
  (`slos/availability.openslo.yaml`).
- 99.95 % monthly availability for governed-share read path
  (`slos/governed-share-consumer-lag.openslo.yaml`).

### E.4 Compliance packs supported

`SOC-2`, `ISO-27001`, `GDPR`, `HIPAA-2024`, `PCI-DSS-L1-v4`, `KR-PIPA`,
`EU-sovereign`, `JP-APPI`, `CA-PIPEDA`, `BR-LGPD`, `IN-DPDP`.

### E.5 DR posture (ADR-0343)

- Service target: RTO p99 ≤ 3600s and RPO p99 ≤ 300s for warehouse metadata, Iceberg snapshots, governed-share state, and query-admission ledgers until a stricter D-2 `manifest.json#dr` block lands.
- Compliance floors considered: HIPAA-2024 RTO 3600s/RPO 300s/multi-region true, PCI-DSS-L1-v4 RTO 86400s/RPO 3600s, SOC2-T2 RTO 14400s/RPO 900s, ISO27001-2022 RTO 14400s/RPO 3600s, and KR-PIPA resident-registration-number RTO 3600s/RPO 300s/multi-region true. HIPAA/KR protected data drive the effective 3600s/300s and multi-region requirement.
- Failover runbook reference: `runbooks/cross-region-replica-lag.md`, `runbooks/dataset-export-failure.md`, `runbooks/workload-pool-starvation.md`, and `runbooks/schema-evolution-break.md`.
- Multi-region posture: active-active read/control plane by home cell; object-table payload replication is pack-aware and metadata-only across cells unless the tenant pack permits analytical data replication.
- Tenant-visible behavior: tenants keep governed-share reads and time-travel recovery semantics during cell loss; stale analytical freshness is preferable to ungoverned cross-region reads.

### E.6 Capacity model (ADR-0340)

- Per-tenant baseline: demo tenants receive 50GiB storage and two concurrent query slots; paid tenants start with one warehouse token (4 vCPU/16GiB), 100GiB hot metadata/object cache, 10 connection slots, and purchased `paid.billing_components` for compute, storage, egress, and fail-safe days.
- Scaling dimension: `query_slot`, `lake_table_writer`, `streaming_ingest_event`, `vector_query`, `federated_query_byte`, and `container_udf_second` scale independently.
- Cell placement class: Tier-1/Tier-2/Tier-3 eligible as declared by `manifest.json` `cell_eligibility`; regulated warehouse state lands in Tier-3 home cells, while high-volume non-regulated scans can run in Tier-1/Tier-2 capacity cells.
- Autoscaling boundaries: minimum zero warm compute for idle demo tenants and one metadata/control replica per home cell; maximum 10,000 concurrent queries and 1,000 lake-table writers per tenant as already declared in §E.2, with query admission rejecting beyond purchased capacity.
- Tenant load profile served: interactive BI, secure share reads, vector lookups, lake-table writes, and batch UDF work are admitted by the resource that actually burns spend instead of one retired capacity ladder.

### E.7 Sustainability + cost attribution (ADR-0344)

- Every dataset registration, query admission, Iceberg snapshot commit, governed-share read, export, vector lookup, SQL-ML call, and Container UDF audit row emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours`.
- Carbon-aware provider routing: yes for compaction, backfill, export, SQL-ML training orchestration, and non-interactive maintenance; no for interactive governed-share reads, incident restore, HIPAA-EM, PCI-realtime-fraud, or high-risk regulated query paths.
- Tenant cost transparency surface: the warehouse burn-rate dashboard, `cost-budget.md`, and FinOps portal expose compute_credits, storage_bytes, egress_gb, share_consumer_events, vector_index_serving, and container_udf_seconds by tenant/cell/provider.
- Regulatory driver: CSRD, SB-253, and SEC climate-disclosure reports require spend and emissions to align with the tenant's analytical operation, not only the infrastructure invoice.

### E.8 API versioning posture (ADR-0342)

- Public API version model: SQL REST, metadata REST, governed-share, export, and proto contracts use the YYYY-MM-DD carrier triplet: `Oyatie-API-Version: <date>`, `/api/data-warehouse/<date>/...`, and proto3 `api_version` fields.
- SDK semver model: generated SDKs and warehouse client libraries publish `major.minor.patch`; semver major is reserved for breaking changes to supported date-versioned contracts.
- Support window: last N=3 public contract dates are supported for at least 180 days.
- Per-tenant pinning: yes for BI tools, governed-share consumers, migration clients, and embedded SQL clients.
- Internal-mesh exemption: yes; direct gRPC to intelligence for SQL-ML and to substrate services preserves ADR-0145 while public carriers stay date-versioned.

## F. Out of scope (re-stated for clarity)

- Tenant identity issuance (owner: `tenant` µservice).
- Cedar engine internals (owner: `compliance` µservice).
- Workflow DAG runtime (owner: `workflow-engine`).
- Model artefacts and inference (owner: `intelligence`).
- Payment rail settlement (owner: `billing-rails`).
- Notebook UI (owner: `studio-shell`).

## G. Rollout plan

- Wave-15A (this PRD): land the bespoke rewrite, 14 IP slices for the
  Lakehouse + Snowflake-distinctive primitives, 14 OpenSLO files, 6 new
  Cedar fragments, 14 new capability YAMLs.
- Wave-15B: numeric SLO binding to dashboards; per-pack data-flow tables
  in `multi-region.md`.
- Wave-15C: integration with `intelligence` µservice for `sql-ml.train`
  + vector serving; integration with `cloud-kms` for CMK rotation.
- Wave-15D: migration playbooks dry-run on real Snowflake / BigQuery /
  Databricks exports.

## H. Acceptance gate for the µservice as a whole

The µservice is considered substance-bar-passing when:

- Every persona in §B has at least one concrete acceptance row passing in
  the integration test suite.
- The audit defects F-D2-01..F-D8-02 are all closed (see
  `REMEDIATION-NOTES-2026-05-21.md`).
- The 60-primitive union coverage from §2.4 of the audit reaches ≥ 85 %
  PASS (currently ~ 89 % after Wave-15A; see
  `feature-parity-matrix-2026-05-20.md`).
- All Cedar default-deny fragments lint clean (`cedar validate` against
  `policies/`).
- All OpenSLO files lint clean (`openslo lint slos/`).

End of PRD.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `data-warehouse` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `data-warehouse` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 3 module pin(s) across 1 context(s).
- Scaling input: `per_query` with cell placement `Tier-2` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
