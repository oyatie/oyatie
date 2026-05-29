# IP-028 Data Pipeline dead-letter replay custody

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-028-dead-letter-replay-custody.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Own custody for dead-letter records before replay, discard, or export.
- Prevent failed pipeline data from becoming invisible queue residue.
- Preserve tenant, source, schema, transform, lineage, and cursor evidence.
- Make replay decisions reviewable before any write-side retry.
- Keep dead-letter replay separate from generic workflow retry.
- Bind every replay to the original failure cause and current policy decision.
- Treat Fivetran and Airbyte Cloud retry flows as usability pressure.
- Treat Hevo and Stitch dead-letter simplicity as fast recovery pressure.
- Treat Matillion and Talend Cloud job-failure workflows as transform pressure.
- Treat Informatica IICS and Estuary Flow as governed replay custody pressure.

## Prerequisites
- Read `microservices/data-pipeline/PRD.md`.
- Read `microservices/data-pipeline/ARCHITECTURE.md`.
- Read `microservices/data-pipeline/backfill-replay.md`.
- Read `microservices/data-pipeline/failure-modes.md`.
- Read `microservices/data-pipeline/capabilities/replay-cursor-advance.yaml`.
- Read `microservices/data-pipeline/runbooks/dead-letter-drain.md`.
- Read `microservices/data-pipeline/runbooks/replay-cursor-rollback.md`.
- Read `microservices/data-pipeline/runbooks/local-pipeline-replay-window.md`.
- Read `microservices/data-pipeline/slos/local-deadletter-rate.openslo.yaml`.
- Read `microservices/data-pipeline/slos/replay-freshness.openslo.yaml`.
- Read `microservices/data-pipeline/policies/local-deadletter-replay-approval.cedar`.
- Read `microservices/data-pipeline/policy/lineage-replay-authorization.cedar`.

## Domain model
- Aggregate: `dead_letter_replay_case`.
- Identity: `tenant_id + dead_letter_partition + failure_fingerprint`.
- Custody unit: failed item, source payload hash, transform hash, policy decision, cursor.
- Required status: captured, classified, approved, replaying, completed, discarded, exported.
- Required owner: data-pipeline replay usecase.
- Required worker: backfill replay worker.
- Required policy: local dead-letter replay approval.
- Required audit: failure event and replay event.
- Required lock: replay window lock by cursor range.
- Required rollback: previous cursor and target projection state.
- Required lineage: edge impact before replay.
- Required cost: CPU, memory, storage, and egress estimate before replay.

## Custody classes
- Source API timeout routes to retryable custody.
- Source rate limit routes to delayed retry custody.
- Schema mismatch routes to schema-drift quarantine custody.
- Transform exception routes to transform approval custody.
- Policy deny routes to non-replayable custody until permit changes.
- Data quality breach routes to quarantine custody.
- Lineage gap routes to graph reconciliation custody.
- Warehouse projection failure routes to idempotent replay custody.
- Duplicate idempotency key routes to suppress custody.
- Cross-tenant payload routes to security custody.
- Pack residency violation routes to compliance custody.
- Unknown failure routes to manual review custody.

## Implementation steps
- Capture failed item before dropping queue visibility.
- Hash payload with tenant-local salt.
- Record source connector and object ids.
- Record transform job and version ids.
- Record lineage reconciliation epoch.
- Record CDC watermark and replay cursor.
- Record Cedar decision at failure time.
- Re-evaluate Cedar before replay.
- Lock replay window before retry.
- Estimate replay cost before approval.
- Emit `oya.data.pipeline.dead_letter.captured`.
- Emit `oya.data.pipeline.dead_letter.replay_approved`.
- Emit `oya.data.pipeline.dead_letter.replay_completed`.
- Emit `oya.data.pipeline.dead_letter.discarded`.
- Require reviewer separation for discard.
- Require pack overlay for regulated payload export.
- Require DealSet status for licensed source replay.
- Require rollback bundle before cursor advancement.

## Evidence payload
- `tenant_id` is mandatory.
- `dead_letter_partition` is mandatory.
- `failure_fingerprint` is mandatory.
- `payload_hash` is mandatory.
- `source_connector_id` is mandatory.
- `source_object_id` is mandatory.
- `transform_job_id` is mandatory when transform ran.
- `lineage_epoch` is mandatory when lineage exists.
- `cdc_watermark` is mandatory when source is CDC.
- `replay_cursor_before` is mandatory.
- `replay_cursor_after` is mandatory after completion.
- `cedar_decision_id_at_failure` is mandatory.
- `cedar_decision_id_at_replay` is mandatory.
- `audit_event_id` is mandatory.
- `cost_estimate_id` is mandatory.
- `approval_id` is mandatory for replay.
- `discard_reason` is mandatory for discard.
- `rollback_bundle_id` is mandatory.

## Policy gates
- Cedar denies replay without tenant scope.
- Cedar denies replay without custody case id.
- Cedar denies replay when source connector license is suspended.
- Cedar denies replay when pack overlay prohibits target cell.
- Cedar denies replay when schema drift case is unresolved.
- Cedar denies replay when lineage reconciliation is unresolved.
- Cedar denies replay when transform approval is stale.
- Cedar denies discard without reviewer separation.
- Cedar denies export without regulated evidence pack.
- Cedar denies cursor advance without replay completion audit.

## Benchmark displacement
- Fivetran parity means failed sync rows are explainable to operators.
- Airbyte Cloud parity means job logs connect to retriable records.
- Hevo parity means common transient failures recover quickly.
- Stitch parity means lightweight replay still carries audit custody.
- Matillion parity means transformation job failures are tied to replay.
- Talend Cloud parity means governed remediation is explicit.
- Informatica IICS parity means stewardship and compliance review are first-class.
- Estuary Flow parity means real-time collection errors preserve watermarks.
- Oyatie adds Cedar, DealSet, pack, lineage, and rollback evidence to every replay.
- Vendor retry semantics never bypass Data Pipeline custody.

## Failure handling
- If custody write fails, keep queue item invisible only for bounded lease.
- If audit-chain fails, stop replay approval.
- If Cedar fails, fail closed for replay and discard.
- If cost estimate fails, require manual approval with degraded marker.
- If replay worker fails, keep cursor unchanged.
- If cursor lock fails, do not replay.
- If payload hash mismatch occurs, classify as custody breach.
- If source object is deleted, require lineage and schema review.
- If target projection already changed, run rollback before retry.
- If replay repeats the same failure, escalate to incident response.

## Tests and evidence
- Contract test: replay approval rejects missing custody id.
- Policy test: discard requires reviewer separation.
- Policy test: cross-tenant payload replay is denied.
- Replay test: cursor does not advance on failed replay.
- Replay test: successful replay emits before and after cursor.
- Audit test: capture and completion events share fingerprint.
- Lineage test: unresolved graph case blocks replay.
- Schema test: unresolved drift case blocks replay.
- Cost test: replay approval includes estimate id.
- SLO test: local-deadletter-rate burn links runbook.

## Rollback
- Roll back by restoring `replay_cursor_before`.
- Mark replayed payloads as reverted, not deleted.
- Emit `oya.data.pipeline.dead_letter.replay_reverted`.
- Reopen custody case with reverted status.
- Preserve failed replay evidence.
- Release replay window lock after revert event.
- Recompute cost impact after revert.
- Recompute lineage impact after revert.
- Recompute schema drift links after revert.
- Link operator response to `runbooks/replay-cursor-rollback.md`.

## Acceptance criteria
- No dead-letter item is replayed without custody evidence.
- No dead-letter item is discarded without reviewer separation.
- No replay advances cursor without completion audit.
- No replay bypasses Cedar.
- No replay bypasses pack overlay.
- No replay bypasses lineage reconciliation.
- No replay bypasses schema drift disposition.
- No replay bypasses cost attribution.
- Every benchmark reference stays comparative.
- Data Pipeline owns dead-letter replay custody end to end.

## Citation map
- `microservices/data-pipeline/backfill-replay.md` anchors replay behavior.
- `microservices/data-pipeline/failure-modes.md` anchors failure classes.
- `microservices/data-pipeline/capabilities/replay-cursor-advance.yaml` anchors capability.
- `microservices/data-pipeline/runbooks/dead-letter-drain.md` anchors dead-letter response.
- `microservices/data-pipeline/runbooks/replay-cursor-rollback.md` anchors rollback.
- `microservices/data-pipeline/slos/local-deadletter-rate.openslo.yaml` anchors SLO.
- `microservices/data-pipeline/slos/replay-freshness.openslo.yaml` anchors freshness.
- `microservices/data-pipeline/policies/local-deadletter-replay-approval.cedar` anchors policy.
- `microservices/data-pipeline/policy/lineage-replay-authorization.cedar` anchors authorization.
- `ADR-0105` anchors layer map.
- `ADR-0314` anchors DealSet.
- `ADR-0321` anchors documentation rigor.

## Operator review prompts
- Reviewer asks whether the item is replayable or permanently invalid.
- Reviewer asks whether the failure came from source, transform, policy, or projection.
- Reviewer asks whether the payload hash matches captured custody.
- Reviewer asks whether replay would cross a frozen cursor window.
- Reviewer asks whether schema drift is still unresolved.
- Reviewer asks whether lineage reconciliation is still unresolved.
- Reviewer asks whether transform approval is still current.
- Reviewer asks whether replay cost exceeds tenant threshold.
- Reviewer asks whether pack overlay permits the target cell.
- Reviewer asks whether DealSet license covers the source connector.
- Reviewer asks whether discard has separated reviewer approval.
- Reviewer asks whether export exposes regulated payload.
- Reviewer asks whether retry count indicates incident escalation.
- Reviewer records the answer set before replay.
- Reviewer signs the case with the same audit correlation id.

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `iceberg_snapshot`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-028-dead-letter-replay-custody.md:148` - - SLO test: local-deadletter-rate burn links runbook.; `microservices/data-pipeline/IP-028-dead-letter-replay-custody.md:180` - - `microservices/data-pipeline/slos/local-deadletter-rate.openslo.yaml` anchors SLO..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-028-dead-letter-replay-custody.md:46` - - Required cost: CPU, memory, storage, and egress estimate before replay.; `microservices/data-pipeline/IP-028-dead-letter-replay-custody.md:72` - - Estimate replay cost before approval..
