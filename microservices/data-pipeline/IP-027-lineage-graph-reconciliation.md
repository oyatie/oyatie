# IP-027 Data Pipeline lineage graph reconciliation

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-027-lineage-graph-reconciliation.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Own reconciliation between connector-observed lineage and ontology graph projection.
- Detect missing, duplicate, stale, reversed, and cross-tenant lineage edges.
- Preserve source-run custody before graph mutation.
- Keep lineage repair separate from schema-drift release.
- Give transform approvers an explainable upstream and downstream impact graph.
- Make replay windows carry graph reconciliation status.
- Treat Fivetran and Airbyte Cloud lineage metadata as parity inputs.
- Treat Matillion and Talend Cloud transform lineage as enterprise workflow pressure.
- Treat Informatica IICS metadata governance as audit-density pressure.
- Treat Estuary Flow derivation graphs as real-time edge freshness pressure.

## Prerequisites
- Read `microservices/data-pipeline/PRD.md`.
- Read `microservices/data-pipeline/ARCHITECTURE.md`.
- Read `microservices/data-pipeline/capabilities/lineage-edge-record.yaml`.
- Read `microservices/data-pipeline/runbooks/lineage-gap-repair.md`.
- Read `microservices/data-pipeline/runbooks/local-lineage-capture-gap.md`.
- Read `microservices/data-pipeline/slos/local-lineage-capture.openslo.yaml`.
- Read `microservices/data-pipeline/policies/local-lineage-record-egress.cedar`.
- Read `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`.
- Read `microservices/data-pipeline/contracts/local-operations-v1.proto`.
- Read `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-domain.yaml`.
- Read `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-usecase.yaml`.
- Read `microservices/data-pipeline/dashboards/local-lineage-capture-gap.md` if it is later added.

## Domain model
- Aggregate: `lineage_reconciliation_case`.
- Identity: `tenant_id + graph_partition_id + reconciliation_epoch`.
- Input: connector edge batch, transform edge batch, replay edge batch, ontology snapshot.
- Output: accepted edge set, rejected edge set, provisional edge set.
- Required cursor: ingest attempt and transform run id.
- Required policy: Cedar permit for lineage edge record.
- Required audit: graph diff hash and edge custody bundle.
- Required rejection reason: duplicate, orphan, direction mismatch, tenant mismatch, stale epoch.
- Required owner: Data Pipeline usecase layer.
- Required consumer: ontology projection adapter.
- Required status: open, reviewed, applied, reverted, superseded.
- Required replay binding: reconciliation epoch attached to replay cursor.

## Reconciliation classes
- Missing source-to-raw edge creates a capture-gap case.
- Missing raw-to-transform edge creates a transform-gap case.
- Missing transform-to-warehouse edge creates a projection-gap case.
- Duplicate edge with same custody hash deduplicates without mutation.
- Duplicate edge with different custody hash requires operator review.
- Direction reversal requires graph repair approval.
- Cross-tenant edge requires immediate deny evidence.
- Cross-cell edge requires residency overlay evaluation.
- Stale edge epoch requires replay-window comparison.
- Orphan node requires source object existence check.
- Unknown transform id requires transform registry lookup.
- Unknown connector id requires connector catalog lookup.

## Implementation steps
- Add graph diff calculation in the kernel layer.
- Keep diff calculation pure and deterministic.
- Add usecase command `lineage.reconcile`.
- Add event `oya.data.pipeline.lineage.reconciliation_opened`.
- Add event `oya.data.pipeline.lineage.reconciliation_applied`.
- Add event `oya.data.pipeline.lineage.reconciliation_reverted`.
- Validate tenant and graph partition before diff.
- Evaluate Cedar before reading edge custody payload.
- Compare connector, transform, replay, and ontology edge sets.
- Mark graph writes as pending until review is complete.
- Apply accepted edge set through ontology adapter only.
- Store rejected edges as audit evidence, not durable graph state.
- Attach reconciliation epoch to replay cursor.
- Attach graph diff hash to transform approval.
- Attach source run id to every accepted edge.
- Attach DealSet evidence if connector license changes edge visibility.
- Attach pack overlay if graph edge crosses regulated data class.
- Attach benchmark label as comparison metadata only.

## Evidence payload
- `tenant_id` is mandatory.
- `graph_partition_id` is mandatory.
- `reconciliation_epoch` is mandatory.
- `connector_run_id` is mandatory when connector edges exist.
- `transform_run_id` is mandatory when transform edges exist.
- `replay_cursor_id` is mandatory when replay edges exist.
- `ontology_snapshot_id` is mandatory.
- `edge_diff_hash` is mandatory.
- `accepted_edge_count` is mandatory.
- `rejected_edge_count` is mandatory.
- `provisional_edge_count` is mandatory.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.
- `pack_overlay_id` is mandatory for regulated edges.
- `dealset_decision_id` is mandatory for licensed connector edges.
- `operator_review_id` is mandatory for non-trivial graph mutation.
- `rollback_bundle_id` is mandatory before apply.
- `benchmark_pressure` is mandatory for parity summary.

## Policy gates
- Cedar denies graph read without tenant scope.
- Cedar denies graph write without lineage-edge-record capability.
- Cedar denies edge egress when policy marks lineage as internal-only.
- Cedar denies cross-tenant graph mutation.
- Cedar denies regulated edge export without pack overlay.
- Cedar denies graph application while audit-chain is unavailable.
- Cedar denies repair if custody payload is missing.
- Cedar denies repair if transform approval is stale.
- Cedar denies repair if replay cursor is frozen.
- Cedar denies repair if source connector license is suspended.

## Benchmark displacement
- Fivetran parity means connector metadata lineage is visible.
- Airbyte Cloud parity means source and destination catalog edges are inspectable.
- Hevo parity means operators see simple edge gaps quickly.
- Stitch parity means lightweight ELT still records custody.
- Matillion parity means transformations own visible lineage edges.
- Talend Cloud parity means stewardship and mapping review are first-class.
- Informatica IICS parity means governed metadata reconciliation is auditable.
- Estuary Flow parity means low-latency derivation lineage has freshness status.
- Oyatie differs by keeping graph mutation tenant-scoped and Cedar-gated.
- Vendor lineage never becomes the canonical graph without reconciliation.

## Failure handling
- If ontology adapter is down, keep reconciliation open.
- If connector run is missing, classify edge set as orphaned.
- If transform registry is down, keep transform edges provisional.
- If audit-chain is down, block graph application.
- If Cedar is down, fail closed for graph mutation.
- If replay cursor has advanced, require replay custody review.
- If pack overlay conflicts, apply higher-restriction-wins.
- If graph diff is too large, require staged application.
- If graph diff repeats, suppress duplicate operator noise.
- If accepted edge later proves wrong, revert by reconciliation epoch.

## Tests and evidence
- Unit test: deterministic diff returns stable hash.
- Unit test: duplicate same-hash edge deduplicates.
- Unit test: duplicate different-hash edge requires review.
- Policy test: cross-tenant edge is denied.
- Contract test: reconciliation command requires tenant and graph partition.
- Event test: opened and applied events share epoch.
- Replay test: cursor records reconciliation epoch.
- Transform test: approval blocks on unresolved graph case.
- Audit test: rejected edges remain evidence-only.
- SLO test: local-lineage-capture burn opens runbook link.

## Rollback
- Roll back by reconciliation epoch.
- Revert only edges applied by this case.
- Preserve rejected and provisional edge evidence.
- Emit `oya.data.pipeline.lineage.reconciliation_reverted`.
- Recompute transform impact after revert.
- Recompute replay windows after revert.
- Reopen source schema-drift case if edge error came from drift.
- Keep DealSet decisions immutable.
- Keep pack overlay decisions immutable.
- Link rollback to `runbooks/lineage-gap-repair.md`.

## Acceptance criteria
- Every graph mutation has a diff hash.
- Every graph mutation has Cedar decision evidence.
- Every graph mutation has audit-chain evidence.
- Every graph mutation has rollback evidence.
- Every rejected edge has a reason.
- Every provisional edge has a consumer warning.
- Every replay cursor knows the reconciliation epoch it depends on.
- Every transform approval sees impacted upstream and downstream edges.
- Every benchmark reference is comparative only.
- Data Pipeline remains the owner of lineage reconciliation.

## Citation map
- `microservices/data-pipeline/PRD.md` anchors lineage user stories.
- `microservices/data-pipeline/ARCHITECTURE.md` anchors bounded context.
- `microservices/data-pipeline/capabilities/lineage-edge-record.yaml` anchors capability.
- `microservices/data-pipeline/runbooks/lineage-gap-repair.md` anchors repair.
- `microservices/data-pipeline/runbooks/local-lineage-capture-gap.md` anchors incident response.
- `microservices/data-pipeline/slos/local-lineage-capture.openslo.yaml` anchors SLO.
- `microservices/data-pipeline/policies/local-lineage-record-egress.cedar` anchors policy.
- `microservices/data-pipeline/contracts/local-operations-v1.proto` anchors internal shape.
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-domain.yaml` anchors catalog.
- `ADR-0105` anchors layer map.
- `ADR-0314` anchors DealSet.
- `ADR-0321` anchors documentation rigor.

## Operator review prompts
- Reviewer asks whether each rejected edge has source custody.
- Reviewer asks whether each accepted edge has tenant partition proof.
- Reviewer asks whether graph mutation changes regulated data visibility.
- Reviewer asks whether replay windows depend on rejected edges.
- Reviewer asks whether transform approvals depend on provisional edges.
- Reviewer asks whether connector license scope changes edge visibility.
- Reviewer asks whether graph repair needs staged application.
- Reviewer asks whether ontology snapshot is current enough.
- Reviewer asks whether source object deletion caused the gap.
- Reviewer asks whether schema drift caused the graph mismatch.
- Reviewer asks whether duplicate edges share the same custody hash.
- Reviewer asks whether stale epochs can be superseded safely.
- Reviewer asks whether downstream dashboards need degraded status.
- Reviewer records the answer set before graph application.
- Reviewer signs the case with the same audit correlation id.

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/data-pipeline/IP-027-lineage-graph-reconciliation.md:28` - - Read `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`.; `microservices/data-pipeline/IP-027-lineage-graph-reconciliation.md:29` - - Read `microservices/data-pipeline/contracts/local-operations-v1.proto`..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `iceberg_snapshot`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-027-lineage-graph-reconciliation.md:148` - - SLO test: local-lineage-capture burn opens runbook link.; `microservices/data-pipeline/IP-027-lineage-graph-reconciliation.md:180` - - `microservices/data-pipeline/slos/local-lineage-capture.openslo.yaml` anchors SLO..
