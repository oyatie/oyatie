# IP-023 Data Pipeline DPIA evidence packet

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-023-dpia-evidence-packet.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Define DPIA evidence for Data Pipeline processing.
- Cover connector capture, schema drift samples, transform outputs, lineage graph, dead-letter custody, replay, watermarks, and audit export.
- Preserve data class, purpose, pack overlay, retention, residency, and deletion evidence.
- Prevent benchmark parity from weakening privacy evidence.
- Treat Fivetran and Airbyte Cloud data movement as DPIA pressure.
- Treat Hevo and Stitch low-friction ELT as DPIA usability pressure.
- Treat Matillion and Talend Cloud transform data handling as DPIA pressure.
- Treat Informatica IICS privacy governance as benchmark pressure.
- Treat Estuary Flow streaming capture as freshness/privacy pressure.
- Keep DPIA evidence tenant-scoped.

## Local references
- `microservices/data-pipeline/dpia.md` is the direct DPIA authority.
- `microservices/data-pipeline/compliance.md` defines pack impact.
- `microservices/data-pipeline/policy/data-residency.md` defines residency.
- `microservices/data-pipeline/policies/local-null-rate-quarantine.cedar` defines quality quarantine.
- `microservices/data-pipeline/policies/local-quality-threshold-enforcement.cedar` defines quality gates.
- `microservices/data-pipeline/runbooks/local-quarantine-release-review.md` handles quarantine release.
- `microservices/data-pipeline/runbooks/tenant-pack-conflict.md` handles pack conflict.
- `microservices/data-pipeline/dashboards/compliance-pack-health.json` observes pack health.
- `microservices/data-pipeline/slos/audit-emission-lag.openslo.yaml` tracks evidence lag.
- `microservices/data-pipeline/contracts/local-openapi-v1.yaml` anchors export commands.

## DPIA processing records
- Connector capture record names source system.
- Source object record names data class.
- Schema drift sample record names sample custody.
- Transform run record names derived data class.
- Lineage edge record names graph visibility class.
- Dead-letter custody record names failed payload class.
- Replay record names original and current processing purpose.
- Watermark record names freshness metadata class.
- Audit export record names recipient and export region.
- DealSet record names licensed connector context.
- Retention record names retention timer.
- Deletion record names erasure behavior.

## Data subject risk fields
- Identifiability risk is recorded.
- Sensitive data risk is recorded.
- Cross-border transfer risk is recorded.
- Automated decision risk is recorded.
- Replay duplication risk is recorded.
- Dead-letter exposure risk is recorded.
- Schema drift misclassification risk is recorded.
- Transform enrichment risk is recorded.
- Lineage inference risk is recorded.
- Watermark freshness misstatement risk is recorded.
- Audit export over-disclosure risk is recorded.
- Vendor benchmark substitution risk is recorded.

## Command deltas
- `dpia.packet.create` creates packet.
- `dpia.packet.attach_connector` attaches connector evidence.
- `dpia.packet.attach_drift` attaches drift evidence.
- `dpia.packet.attach_transform` attaches transform evidence.
- `dpia.packet.attach_lineage` attaches graph evidence.
- `dpia.packet.attach_replay` attaches custody and cursor evidence.
- `dpia.packet.attach_watermark` attaches freshness evidence.
- `dpia.packet.attach_pack_overlay` attaches residency evidence.
- `dpia.packet.attach_retention` attaches retention evidence.
- `dpia.packet.export` exports scoped packet.
- `dpia.packet.revoke_export` revokes export grant.
- `dpia.packet.close` records review completion.

## Event deltas
- `dpia.packet_created` records packet start.
- `dpia.connector_evidence_attached` records connector evidence.
- `dpia.drift_evidence_attached` records sample custody evidence.
- `dpia.transform_evidence_attached` records derived data evidence.
- `dpia.lineage_evidence_attached` records graph evidence.
- `dpia.replay_evidence_attached` records custody evidence.
- `dpia.watermark_evidence_attached` records freshness evidence.
- `dpia.pack_overlay_attached` records residency evidence.
- `dpia.packet_exported` records export.
- `dpia.export_revoked` records revocation.
- Events include packet id.
- Events include data class.

## Proto deltas
- `DpiaPacketRef` includes packet id.
- `DpiaEvidenceRef` includes evidence class.
- `DpiaRiskRef` includes risk rating.
- `DpiaExportRequest` includes recipient scope.
- `DpiaExportRequest` includes export region.
- `DpiaExportResponse` includes evidence bundle id.
- `DpiaRetentionRef` includes retention timer.
- `DpiaDeletionRef` includes deletion timer.
- `DpiaReplayEvidence` includes original and current purpose.
- `DpiaLineageEvidence` includes graph visibility.
- Proto rejects packet export without auditor or privacy scope.
- Proto rejects packet closure with missing required classes.

## Cedar facts
- `dpia_packet_id` is a policy fact.
- `evidence_class` is a policy fact.
- `data_class` is a policy fact.
- `processing_purpose` is a policy fact.
- `export_region` is a policy fact.
- `recipient_scope` is a policy fact.
- `pack_overlay_state` is a policy fact.
- `retention_class` is a policy fact.
- `deletion_class` is a policy fact.
- `privacy_reviewer_scope` is a policy fact.
- `auditor_scope` is a policy fact.
- `risk_rating` is a policy fact.

## Workflow decisions
- DPIA packet starts when regulated processing changes.
- Connector evidence attaches before transform evidence.
- Drift sample evidence attaches before release.
- Transform evidence records derived data class.
- Lineage evidence records graph inference risk.
- Replay evidence records duplicate processing risk.
- Watermark evidence records freshness misstatement risk.
- Pack overlay evidence records residency decision.
- Export requires auditor or privacy scope.
- Export redacts raw payload.
- Closure requires all mandatory evidence classes.
- Packet rollback records superseding packet.

## Failure cases
- Missing data class blocks packet closure.
- Missing processing purpose blocks packet closure.
- Missing pack overlay blocks regulated export.
- Missing retention class blocks packet closure.
- Missing deletion class blocks packet closure.
- Drift sample without custody blocks packet closure.
- Replay evidence without custody blocks packet closure.
- Lineage evidence without visibility class blocks packet closure.
- Watermark evidence without freshness class blocks packet closure.
- Export region mismatch blocks export.
- Raw payload in packet export is incident.
- Benchmark-only DPIA claim is rejected.

## Replay cases
- Replay attaches original processing purpose.
- Replay attaches current processing purpose.
- Replay attaches dead-letter custody id.
- Replay attaches cursor before and after.
- Replay attaches duplicate processing risk.
- Replay attaches pack overlay at failure.
- Replay attaches pack overlay at retry.
- Replay attaches deletion effect.
- Replay attaches retention effect.
- Replay export redacts raw failed payload.
- Replay rollback creates correction evidence.
- Replay cannot close DPIA packet without custody.

## Evidence fields
- `dpia_packet_id` is mandatory.
- `tenant_id` is mandatory.
- `data_class` is mandatory.
- `processing_purpose` is mandatory.
- `source_system_id` is mandatory.
- `source_object_id` is mandatory.
- `pack_overlay_ids` is mandatory.
- `residency_decision_id` is mandatory.
- `retention_class` is mandatory.
- `deletion_class` is mandatory.
- `risk_rating` is mandatory.
- `reviewer_id` is mandatory.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.
- `export_region` is mandatory when exported.
- `benchmark_pressure` is mandatory for parity summary.

## SLOs
- DPIA packet assembly lag feeds compliance pack health.
- Audit emission lag tracks DPIA export events.
- Missing evidence age feeds operator remediation.
- Pack conflict age feeds compliance dashboard.
- Drift sample evidence age feeds schema drift risk.
- Replay custody evidence age feeds replay freshness risk.
- Lineage evidence age feeds lineage capture risk.
- Export revocation latency feeds privacy health.
- Raw payload redaction failures feed incident dashboard.
- Packet closure age feeds readiness dashboard.
- Privacy review age feeds review SLA.
- DPIA rollback age feeds audit completeness.

## Test cases
- Packet closure rejects missing data class.
- Packet closure rejects missing purpose.
- Packet export rejects missing auditor scope.
- Packet export rejects disallowed region.
- Drift evidence requires custody id.
- Replay evidence requires custody id.
- Lineage evidence requires visibility class.
- Watermark evidence requires freshness class.
- Raw payload export is rejected.
- Benchmark-only packet is rejected.
- Rollback creates superseding packet.
- Revocation emits audit event.

## Rollback
- DPIA rollback supersedes packet.
- DPIA rollback preserves original evidence.
- Export revocation records recipient and region.
- Replay correction appends evidence.
- Drift correction appends evidence.
- Lineage correction appends evidence.
- Watermark correction appends evidence.
- Retention correction appends evidence.
- Deletion correction appends evidence.
- Rollback emits DPIA superseded event.
- Compliance dashboard recomputes packet status.
- Audit export includes original and superseding packets.

## Acceptance criteria
- DPIA packet covers connector, drift, transform, lineage, replay, watermark, and export.
- DPIA packet has data class and purpose.
- DPIA packet has pack overlay and residency decision.
- DPIA export is scoped and redacted.
- Replay evidence includes custody.
- Drift evidence includes sample custody.
- Lineage evidence includes visibility class.
- Every benchmark reference is comparative.
- Packet closure blocks on missing evidence.
- DPIA evidence remains Data Pipeline-specific.

## Citation map
- `microservices/data-pipeline/dpia.md`
- `microservices/data-pipeline/compliance.md`
- `microservices/data-pipeline/policy/data-residency.md`
- `microservices/data-pipeline/policies/local-null-rate-quarantine.cedar`
- `microservices/data-pipeline/policies/local-quality-threshold-enforcement.cedar`
- `microservices/data-pipeline/runbooks/local-quarantine-release-review.md`
- `microservices/data-pipeline/runbooks/tenant-pack-conflict.md`
- `microservices/data-pipeline/dashboards/compliance-pack-health.json`
- `microservices/data-pipeline/slos/audit-emission-lag.openslo.yaml`
- `microservices/data-pipeline/contracts/local-openapi-v1.yaml`
- `ADR-0105`
- `ADR-0321`

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/data-pipeline/IP-023-dpia-evidence-packet.md:30` - - `microservices/data-pipeline/contracts/local-openapi-v1.yaml` anchors export commands.; `microservices/data-pipeline/IP-023-dpia-evidence-packet.md:240` - - `microservices/data-pipeline/contracts/local-openapi-v1.yaml`.
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-023-dpia-evidence-packet.md:176` - ## SLOs.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-023-dpia-evidence-packet.md:29` - - `microservices/data-pipeline/slos/audit-emission-lag.openslo.yaml` tracks evidence lag.; `microservices/data-pipeline/IP-023-dpia-evidence-packet.md:178` - - Audit emission lag tracks DPIA export events..
