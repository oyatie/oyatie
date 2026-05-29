# IP-016 Data Pipeline backfill replay worker

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-016-backfill-replay-worker.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Define the worker that replays accepted backfill and dead-letter work.
- Keep replay separated from connector run start.
- Keep replay separated from generic workflow retry.
- Bind replay to custody, cursor, transform, lineage, watermark, and cost evidence.
- Prevent replay from advancing cursor before durable target confirmation.
- Treat Fivetran historical resync as benchmark pressure.
- Treat Airbyte Cloud reset/resync as benchmark pressure.
- Treat Hevo and Stitch replay simplicity as usability pressure.
- Treat Matillion and Talend Cloud job rerun as transform pressure.
- Treat Informatica IICS and Estuary Flow as governed replay pressure.

## Local references
- `microservices/data-pipeline/backfill-replay.md` is the direct replay authority.
- `microservices/data-pipeline/capabilities/replay-cursor-advance.yaml` binds cursor movement.
- `microservices/data-pipeline/policies/local-deadletter-replay-approval.cedar` gates replay.
- `microservices/data-pipeline/runbooks/dead-letter-drain.md` handles dead-letter backlog.
- `microservices/data-pipeline/runbooks/replay-cursor-rollback.md` handles cursor rollback.
- `microservices/data-pipeline/runbooks/local-pipeline-replay-window.md` handles replay windows.
- `microservices/data-pipeline/slos/replay-freshness.openslo.yaml` tracks replay freshness.
- `microservices/data-pipeline/slos/local-deadletter-rate.openslo.yaml` tracks failed items.
- `microservices/data-pipeline/contracts/local-operations-v1.proto` carries worker callbacks.
- `microservices/data-pipeline/failure-modes.md` classifies replay failures.

## Worker inputs
- Replay window id is required.
- Tenant scope is required.
- Cursor before is required.
- Cursor target is required.
- Dead-letter custody id is required for failed-item replay.
- Source object id is required.
- Connector catalog version is required.
- Transform version id is required when transform ran.
- Lineage reconciliation epoch is required when graph output is affected.
- Watermark kind is required when CDC freshness changes.
- Cost estimate id is required for expensive replay.
- Cedar decision id is required.

## Worker state
- `queued` means replay is admitted.
- `leased` means worker owns bounded attempt.
- `reading_source` means source or custody read started.
- `writing_target` means target mutation started.
- `verifying_target` means idempotent confirmation started.
- `advancing_cursor` means cursor movement is pending.
- `completed` means cursor and evidence are final.
- `failed_retryable` means retry is allowed.
- `failed_terminal` means operator review is required.
- `rolled_back` means forward attempt was reverted.
- `cancelled_safe` means no cursor moved.
- `cancelled_unsafe` means rollback review is required.

## Command deltas
- `replay.window.open` creates bounded replay window.
- `replay.worker.lease` leases one worker attempt.
- `replay.worker.heartbeat` extends active lease.
- `replay.item.apply` writes target mutation idempotently.
- `replay.item.verify` confirms target mutation.
- `replay.cursor.advance` moves cursor after verification.
- `replay.window.complete` closes window.
- `replay.window.fail` records terminal failure.
- `replay.window.rollback` restores cursor before.
- `replay.window.cancel` cancels with safety class.
- `replay.cost.finalize` records actual replay cost.
- `replay.watermark.update` records CDC effect.

## Event deltas
- `replay.window_opened` records window bounds.
- `replay.worker_leased` records worker lease.
- `replay.item_started` records item attempt.
- `replay.item_applied` records target mutation.
- `replay.item_verified` records idempotent confirmation.
- `replay.cursor_advanced` records cursor movement.
- `replay.window_completed` records closure.
- `replay.window_failed` records failure.
- `replay.window_rolled_back` records rollback.
- `replay.worker_lease_expired` records lost worker.
- Events include custody id.
- Events include cursor before and after.

## Proto deltas
- `ReplayWindowRef` carries window id and cursor bounds.
- `ReplayWorkerLease` carries lease id and expiry.
- `ReplayApplyRequest` carries idempotency key.
- `ReplayVerifyRequest` carries target checkpoint.
- `ReplayCursorAdvanceRequest` carries verified target checkpoint.
- `ReplayRollbackRequest` carries rollback bundle.
- `ReplayCostRef` carries estimate and actual ids.
- `ReplayWatermarkEffect` carries watermark kind and value.
- Proto rejects cursor advance without verify checkpoint.
- Proto rejects replay apply without custody id.
- Proto rejects worker heartbeat after lease expiry.
- Proto rejects rollback without cursor before.

## Cedar facts
- `replay_window_id` is a policy fact.
- `cursor_before` is a policy fact.
- `cursor_target` is a policy fact.
- `custody_state` is a policy fact.
- `worker_lease_state` is a policy fact.
- `schema_drift_state` is a policy fact.
- `lineage_epoch_state` is a policy fact.
- `transform_approval_state` is a policy fact.
- `watermark_state` is a policy fact.
- `cost_budget_state` is a policy fact.
- `dealset_license_state` is a policy fact.
- `pack_overlay_state` is a policy fact.

## Workflow decisions
- Replay opens only after custody approval.
- Replay validates current policy before worker lease.
- Replay locks cursor range before applying items.
- Replay verifies target mutation before cursor advance.
- Replay updates watermark only after cursor advance.
- Replay finalizes cost after worker completion.
- Replay emits audit event at each irreversible boundary.
- Replay stops when schema drift becomes unresolved.
- Replay stops when lineage epoch becomes invalid.
- Replay stops when transform approval becomes stale.
- Replay stops when DealSet license becomes inactive.
- Replay stops when pack overlay becomes stricter.

## Failure cases
- Worker lease expiry leaves cursor unchanged.
- Source read failure leaves cursor unchanged.
- Target write failure leaves cursor unchanged unless verified partial exists.
- Verification failure leaves cursor unchanged.
- Cursor advance failure opens rollback review.
- Watermark update failure holds freshness state.
- Cost finalization failure marks completion incomplete.
- Audit-chain outage blocks cursor advance.
- Cedar outage blocks worker lease.
- DealSet inactive blocks licensed source replay.
- Pack conflict blocks regulated replay.
- Duplicate replay attempt returns existing window state.

## Replay-specific cases
- Historical backfill uses bounded cursor window.
- Dead-letter replay uses custody case id.
- Schema adaptation replay references drift case id.
- Transform correction replay references transform version.
- Lineage repair replay references reconciliation epoch.
- CDC catch-up replay references watermark kind.
- Provider-rate-limit replay uses delayed scheduling.
- Migration-only replay permits export path only.
- Audit-only replay cannot mutate target.
- Over-quota replay follows capacity admission.
- Emergency-frozen replay cannot progress.
- Rollback replay creates new rollback state.

## Evidence fields
- `tenant_id` is mandatory.
- `replay_window_id` is mandatory.
- `cursor_before` is mandatory.
- `cursor_target` is mandatory.
- `cursor_after` is mandatory on completion.
- `custody_case_id` is mandatory.
- `worker_lease_id` is mandatory.
- `source_object_id` is mandatory.
- `connector_catalog_version` is mandatory.
- `target_checkpoint_id` is mandatory.
- `verify_checkpoint_id` is mandatory.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.
- `rollback_bundle_id` is mandatory.
- `cost_attribution_id` is mandatory when costly.
- `benchmark_pressure` is mandatory for parity summary.

## SLOs
- Replay freshness uses cursor completion events.
- Dead-letter rate uses captured and completed events.
- Worker lease expiry rate feeds capacity dashboard.
- Replay target write latency is measured separately.
- Cursor advance latency is measured separately.
- Watermark update lag feeds freshness SLO.
- Cost finalization lag feeds tenant cost dashboard.
- Policy denial rate feeds local policy dashboard.
- Audit emission lag gates completion claim.
- Provider rate-limit replay backlog feeds operator remediation.
- Replay rollback rate feeds failure-mode dashboard.
- Duplicate replay rate feeds idempotency health.

## Test cases
- Replay rejects missing custody id.
- Replay rejects unresolved schema drift.
- Replay rejects invalid lineage epoch.
- Replay rejects stale transform approval.
- Replay rejects inactive DealSet license.
- Replay rejects stricter pack overlay.
- Worker lease expiry leaves cursor unchanged.
- Target verification required before cursor advance.
- Cursor rollback restores cursor before.
- Duplicate replay returns existing state.
- Watermark updates after cursor advance.
- Cost actual records after completion.

## Rollback
- Rollback restores cursor before.
- Rollback marks target checkpoint reverted.
- Rollback emits replay rolled back event.
- Rollback updates watermark with rolled-back state.
- Rollback preserves failed target evidence.
- Rollback preserves worker lease history.
- Rollback preserves DealSet decision history.
- Rollback preserves pack overlay history.
- Rollback recomputes replay freshness.
- Rollback recomputes cost actuals.
- Rollback links replay-cursor-rollback runbook.
- Rollback requires audit event.

## Acceptance criteria
- Replay never advances cursor before target verification.
- Replay always has custody id.
- Replay always has policy receipt.
- Replay always has audit events.
- Replay always has rollback bundle.
- Replay observes schema drift, lineage, transform, DealSet, and pack blockers.
- Replay freshness reflects held windows.
- Replay cost is attributed.
- Every benchmark reference is comparative.
- Backfill replay worker remains Data Pipeline-specific.

## Citation map
- `microservices/data-pipeline/backfill-replay.md`
- `microservices/data-pipeline/capabilities/replay-cursor-advance.yaml`
- `microservices/data-pipeline/policies/local-deadletter-replay-approval.cedar`
- `microservices/data-pipeline/runbooks/dead-letter-drain.md`
- `microservices/data-pipeline/runbooks/replay-cursor-rollback.md`
- `microservices/data-pipeline/runbooks/local-pipeline-replay-window.md`
- `microservices/data-pipeline/slos/replay-freshness.openslo.yaml`
- `microservices/data-pipeline/slos/local-deadletter-rate.openslo.yaml`
- `microservices/data-pipeline/contracts/local-operations-v1.proto`
- `microservices/data-pipeline/failure-modes.md`
- `ADR-0105`
- `ADR-0321`

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/data-pipeline/IP-016-backfill-replay-worker.md:29` - - `microservices/data-pipeline/contracts/local-operations-v1.proto` carries worker callbacks.; `microservices/data-pipeline/IP-016-backfill-replay-worker.md:239` - - `microservices/data-pipeline/contracts/local-operations-v1.proto`.
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-016-backfill-replay-worker.md:176` - ## SLOs; `microservices/data-pipeline/IP-016-backfill-replay-worker.md:182` - - Watermark update lag feeds freshness SLO..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-016-backfill-replay-worker.md:12` - - Bind replay to custody, cursor, transform, lineage, watermark, and cost evidence.; `microservices/data-pipeline/IP-016-backfill-replay-worker.md:71` - - `replay.cost.finalize` records actual replay cost..
