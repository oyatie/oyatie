# IP-030 Data Pipeline CDC freshness watermark governance

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-030-cdc-freshness-watermark-governance.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Govern CDC freshness watermarks for connector runs, replay, transforms, and lineage.
- Prevent stale data from being presented as fresh.
- Prevent replay from moving watermarks without custody.
- Preserve source, tenant, connector, cell, and data-class context on every watermark.
- Bind watermark advancement to Cedar, audit, SLO, and rollback evidence.
- Keep CDC ownership inside Data Pipeline because pipeline runs and replay need a local owner.
- Treat Fivetran and Airbyte Cloud incremental sync behavior as benchmark pressure.
- Treat Hevo and Stitch freshness reporting as lightweight usability pressure.
- Treat Estuary Flow low-latency capture as real-time pressure.
- Treat Matillion, Talend Cloud, and Informatica IICS as governed enterprise pressure.

## Prerequisites
- Read `microservices/data-pipeline/PRD.md`.
- Read `microservices/data-pipeline/ARCHITECTURE.md`.
- Read `microservices/data-pipeline/capabilities/connector-run-start.yaml`.
- Read `microservices/data-pipeline/capabilities/replay-cursor-advance.yaml`.
- Read `microservices/data-pipeline/runbooks/local-ingest-freshness-burn.md`.
- Read `microservices/data-pipeline/runbooks/replay-cursor-rollback.md`.
- Read `microservices/data-pipeline/runbooks/provider-rate-limit.md`.
- Read `microservices/data-pipeline/slos/local-ingest-freshness.openslo.yaml`.
- Read `microservices/data-pipeline/slos/replay-freshness.openslo.yaml`.
- Read `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`.
- Read `microservices/data-pipeline/policies/local-ingest-source-scope.cedar`.
- Read `microservices/data-pipeline/policies/local-transform-run-control.cedar`.

## Domain model
- Aggregate: `cdc_freshness_watermark`.
- Identity: `tenant_id + connector_id + source_object_id + watermark_kind`.
- Watermark kind: source, captured, landed, transformed, lineage_applied, replayed.
- Required value: ordered source cursor or logical timestamp.
- Required status: proposed, held, advanced, rolled_back, stale, superseded.
- Required freshness: observed lag and SLO budget.
- Required policy: ingest source scope and replay cursor approval.
- Required audit: proposed event and advanced event.
- Required rollback: previous watermark and target projection checkpoint.
- Required lineage: reconciliation epoch at advancement.
- Required transform: transform run id when transformed watermark advances.
- Required custody: dead-letter replay custody id when replay moves watermark.

## Watermark rules
- Source watermark records what provider claims is available.
- Captured watermark records what connector has safely captured.
- Landed watermark records what raw storage has durably accepted.
- Transformed watermark records what transform output has durably accepted.
- Lineage-applied watermark records graph visibility.
- Replayed watermark records cursor after custody replay.
- Watermarks only advance monotonically inside a kind.
- Rollback creates a new rollback state instead of deleting history.
- Cross-kind comparison can show lag but cannot imply completion.
- Freshness status must name the slowest required kind.
- Provider freshness does not equal tenant-visible freshness.
- Tenant-visible freshness requires transformed and lineage-applied watermarks.

## Implementation steps
- Add kernel comparison for CDC watermark ordering.
- Add usecase command `watermark.propose`.
- Add usecase command `watermark.advance`.
- Add usecase command `watermark.hold`.
- Add usecase command `watermark.rollback`.
- Evaluate Cedar before propose and advance.
- Require connector run id for source and captured watermarks.
- Require raw landing id for landed watermark.
- Require transform run id for transformed watermark.
- Require lineage reconciliation id for lineage-applied watermark.
- Require replay custody id for replayed watermark.
- Emit `oya.data.pipeline.watermark.proposed`.
- Emit `oya.data.pipeline.watermark.advanced`.
- Emit `oya.data.pipeline.watermark.held`.
- Emit `oya.data.pipeline.watermark.rolled_back`.
- Update local-ingest-freshness SLO projection.
- Update replay-freshness SLO projection.
- Attach benchmark pressure label as metadata only.

## Evidence payload
- `tenant_id` is mandatory.
- `home_cell` is mandatory.
- `connector_id` is mandatory.
- `source_object_id` is mandatory.
- `watermark_kind` is mandatory.
- `watermark_value_before` is mandatory.
- `watermark_value_after` is mandatory for advance.
- `provider_reported_freshness` is mandatory when source reports it.
- `observed_lag_ms` is mandatory.
- `slo_budget_remaining` is mandatory.
- `connector_run_id` is mandatory for captured kinds.
- `transform_run_id` is mandatory for transformed kinds.
- `lineage_epoch` is mandatory for lineage-applied kinds.
- `replay_custody_id` is mandatory for replayed kinds.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.
- `rollback_bundle_id` is mandatory for rollback.
- `staleness_reason` is mandatory for held status.

## Policy gates
- Cedar denies advance without tenant scope.
- Cedar denies advance without source object ownership.
- Cedar denies transformed watermark without transform approval.
- Cedar denies lineage-applied watermark without reconciliation epoch.
- Cedar denies replayed watermark without custody case.
- Cedar denies rollback without rollback bundle.
- Cedar denies regulated source advance outside allowed cell.
- Cedar denies provider freshness claim as tenant-visible freshness.
- Cedar denies watermark mutation while audit-chain is unavailable.
- Cedar denies DealSet-licensed connector advance when license is stale.

## Benchmark displacement
- Fivetran parity means incremental sync freshness is visible.
- Airbyte Cloud parity means job-level sync cursors are inspectable.
- Hevo parity means operators get understandable freshness status.
- Stitch parity means simple ELT still has durable watermark history.
- Matillion parity means transformed freshness is separate from landed freshness.
- Talend Cloud parity means governed CDC movement has approval evidence.
- Informatica IICS parity means enterprise metadata controls watermarks.
- Estuary Flow parity means streaming freshness is low-latency and derivation-aware.
- Oyatie adds tenant, Cedar, lineage, replay, pack, and rollback evidence.
- Vendor cursor names never become canonical Data Pipeline object names.

## Failure handling
- If provider API stalls, hold source watermark and link provider-rate-limit runbook.
- If connector capture fails, do not advance captured watermark.
- If raw landing fails, do not advance landed watermark.
- If transform fails, keep transformed watermark stale.
- If lineage reconciliation fails, keep lineage-applied watermark stale.
- If replay fails, keep replayed watermark unchanged.
- If Cedar fails, fail closed for watermark mutation.
- If audit-chain fails, hold all advances.
- If SLO burn triggers, link local-ingest-freshness runbook.
- If rollback occurs, create rollback state with previous value.

## Tests and evidence
- Unit test: watermark ordering rejects backward advance.
- Unit test: rollback creates new state.
- Contract test: advance command rejects missing kind.
- Policy test: replayed advance requires custody id.
- Policy test: transformed advance requires transform run id.
- Policy test: lineage-applied advance requires reconciliation epoch.
- SLO test: local-ingest-freshness burn opens runbook.
- Replay test: replay-freshness projection updates after custody completion.
- Audit test: propose and advance share correlation id.
- Dashboard test: stale reason is visible without raw tenant-id metric cardinality.

## Rollback
- Roll back by creating `rolled_back` status.
- Preserve the forward watermark value as evidence.
- Restore target projection checkpoint.
- Emit `oya.data.pipeline.watermark.rolled_back`.
- Recompute freshness projections after rollback.
- Recompute replay windows after rollback.
- Recompute lineage-visible freshness after rollback.
- Recompute transform-visible freshness after rollback.
- Preserve DealSet and pack overlay decisions.
- Link rollback to `runbooks/replay-cursor-rollback.md`.

## Acceptance criteria
- Tenant-visible freshness never relies on provider claim alone.
- Watermarks do not move backward except through explicit rollback state.
- Replay cannot move watermark without custody.
- Transform cannot move watermark without transform run evidence.
- Lineage cannot move watermark without reconciliation epoch.
- Every advance has Cedar evidence.
- Every advance has audit-chain evidence.
- Every hold names a staleness reason.
- Every benchmark reference is comparative.
- Data Pipeline owns CDC freshness watermark governance.

## Citation map
- `microservices/data-pipeline/PRD.md` anchors connector and replay requirements.
- `microservices/data-pipeline/ARCHITECTURE.md` anchors bounded context.
- `microservices/data-pipeline/capabilities/connector-run-start.yaml` anchors connector capability.
- `microservices/data-pipeline/capabilities/replay-cursor-advance.yaml` anchors replay capability.
- `microservices/data-pipeline/runbooks/local-ingest-freshness-burn.md` anchors incident response.
- `microservices/data-pipeline/slos/local-ingest-freshness.openslo.yaml` anchors ingest SLO.
- `microservices/data-pipeline/slos/replay-freshness.openslo.yaml` anchors replay SLO.
- `microservices/data-pipeline/policies/local-ingest-source-scope.cedar` anchors source policy.
- `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml` anchors event shape.
- `ADR-0105` anchors layer map.
- `ADR-0314` anchors DealSet.
- `ADR-0321` anchors documentation rigor.

## Operator review prompts
- Reviewer asks whether source freshness differs from tenant-visible freshness.
- Reviewer asks whether captured watermark is behind source watermark.
- Reviewer asks whether landed watermark is behind captured watermark.
- Reviewer asks whether transformed watermark is behind landed watermark.
- Reviewer asks whether lineage-applied watermark is behind transformed watermark.
- Reviewer asks whether replayed watermark moved through custody.
- Reviewer asks whether provider rate limits explain staleness.
- Reviewer asks whether schema drift holds the watermark.
- Reviewer asks whether graph reconciliation holds the watermark.
- Reviewer asks whether pack overlay blocks cell movement.
- Reviewer asks whether DealSet license blocks connector advance.
- Reviewer asks whether SLO burn requires incident response.
- Reviewer asks whether rollback should restore projection checkpoint.
- Reviewer records the answer set before advance.
- Reviewer signs the case with the same audit correlation id.

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/data-pipeline/IP-030-cdc-freshness-watermark-governance.md:30` - - Read `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`.; `microservices/data-pipeline/IP-030-cdc-freshness-watermark-governance.md:183` - - `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml` anchors event shape..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-030-cdc-freshness-watermark-governance.md:13` - - Bind watermark advancement to Cedar, audit, SLO, and rollback evidence.; `microservices/data-pipeline/IP-030-cdc-freshness-watermark-governance.md:40` - - Required freshness: observed lag and SLO budget..
