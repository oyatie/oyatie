# IP-016 Whiteboard backfill-replay-worker

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-016-backfill-replay-worker.md
Benchmarks: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo-local references: microservices/whiteboard/PRD.md, microservices/whiteboard/ARCHITECTURE.md, microservices/whiteboard/backfill-replay.md, microservices/whiteboard/failure-modes.md, microservices/whiteboard/capabilities/board-open.yaml, microservices/whiteboard/capabilities/canvas-op-append.yaml, microservices/whiteboard/capabilities/presence-sync.yaml, microservices/whiteboard/capabilities/history-snapshot.yaml, microservices/whiteboard/capabilities/export-render.yaml, microservices/whiteboard/capabilities/template-marketplace-install.yaml, microservices/whiteboard/runbooks, microservices/whiteboard/scorecards

## Objective
- Define a replay worker for board history, vendor migration, template re-materialization, and audit repair.
- Preserve the Whiteboard distinction that board history and export semantics are not document-file semantics.
- Preserve ADR-0321 anchors for replay evidence, rollback, tenant scoping, Cedar, audit, pack overlay, and benchmark parity.
- Rebuild derived projections without mutating immutable canvas operations.
- Support import/migration paths from Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
- Make replay deterministic enough for auditors and SREs to compare before/after state.
- Keep marketplace DealSet evidence intact when replay touches installed templates.
- Keep residency pack decisions intact when replay touches snapshots or exports.
- Keep emergency board evidence intact when replay touches incident ranges.
- Make every replay resumable, bounded, tenant-scoped, and idempotent.

## Non-goals
- Do not create a generic ETL engine.
- Do not reinterpret vendor data outside Whiteboard's ontology projection.
- Do not delete original operations during replay.
- Do not replay across tenants in one transaction.
- Do not bypass Cedar because replay is internal.
- Do not bypass residency because replay is offline.
- Do not invent new capabilities outside the six capability records.
- Do not edit backfill-replay.md in this slice.
- Do not update ADR-0321.
- Do not touch code or queue configuration here.

## Replay categories
- Category `projection_rebuild` rebuilds read models from canonical operations.
- Category `history_snapshot_repair` regenerates snapshots after renderer or pack bug fixes.
- Category `export_regeneration` recreates export artifacts when redaction or watermark policy changes.
- Category `vendor_import_replay` reprocesses imported boards from a displaced vendor source.
- Category `template_rematerialization` reapplies installed template materialization after catalog fixes.
- Category `settlement_evidence_replay` rehydrates marketplace DealSet audit records.
- Category `presence_compaction` rebuilds participant timelines from presence events.
- Category `emergency_after_action` reconstructs incident board ranges for after-action review.
- Category `pack_reclassification` recalculates residency overlays after pack changes.
- Category `capacity_shard_rebalance` rebuilds projections after partition movement.
- Each category declares read source, write target, allowed mutation, rollback bundle, and stop condition.
- Each category has a dry-run mode.
- Each category has tenant and cell bounds.
- Each category has audit-chain evidence.
- Each category has replay id and idempotency key.

## Benchmark displacement notes
- Miro Enterprise import pressures bulk object mapping and sticky-note fidelity.
- Mural Enterprise import pressures facilitation structures and voting artifacts.
- FigJam import pressures multiplayer operation order and lightweight widgets.
- Lucidspark import pressures diagram semantics and connector fidelity.
- Whiteboard.fi import pressures classroom board batches and short-lived student artifacts.
- Microsoft Whiteboard import pressures enterprise-market identity and board ownership.
- Oyatie replay displaces those paths by separating source vendor labels from canonical Whiteboard operations.
- Benchmark parity requires replay evidence, not just initial import success.
- Vendor source ids are provenance fields, not durable aggregate ids.
- Replay must be able to prove why imported content was accepted, transformed, rejected, or quarantined.

## Capability binding
- `board-open` reads rebuilt projections but never triggers replay directly.
- `board-open` exposes replay degraded state when projections are under repair.
- `canvas-op-append` remains the canonical source for operation replay.
- `canvas-op-append` events are immutable and version-monotonic.
- `presence-sync` contributes participant timeline events for compaction.
- `presence-sync` replay cannot synthesize participant actions without source evidence.
- `history-snapshot` is both replay source and replay target depending on category.
- `history-snapshot` repair records previous snapshot hash and new snapshot hash.
- `export-render` can regenerate artifacts from snapshots and operations.
- `export-render` replay must preserve redaction and watermark policy.
- `template-marketplace-install` replay rematerializes only when settlement evidence is valid.
- `template-marketplace-install` replay cannot create new commercial entitlements.
- Capability records remain under microservices/whiteboard/capabilities.
- backfill-replay.md remains the local companion for worker-specific buildout.
- failure-modes.md remains the companion for quarantine and degraded states.

## Replay state machine
- State `requested` records tenant, category, requested range, requester, and reason.
- State `admitted` records capacity and policy admission.
- State `dry_run_started` records source cursor and target projection.
- State `dry_run_completed` records candidate changes, rejects, and cost estimate.
- State `approved` records operator or workflow approval where required.
- State `running` records active cursor, batch id, and worker lease.
- State `paused` records bounded pause reason and resume cursor.
- State `quarantined` records mismatch reason and affected object ids.
- State `completed` records final hashes and audit event id.
- State `rolled_back` records rollback bundle and restored projection hash.
- State `failed` records terminal reason and remediation owner.
- No state can skip dry-run for vendor import, pack reclassification, or settlement evidence replay.
- No state can mark completed without audit-chain seal.
- No state can process a batch without tenant and cell bounds.
- Worker lease prevents concurrent writes to the same replay target.

## Data requirements
- `replay_id` is globally unique.
- `tenant_id` is mandatory.
- `home_cell` and `target_cell` are mandatory.
- `category` selects replay rules.
- `source_cursor` is opaque and resumable.
- `target_projection` names the rebuilt surface.
- `source_vendor` is optional evidence for import replay.
- `source_object_id` is provenance, not primary identity.
- `operation_range_start` and `operation_range_end` bound canonical operation replay.
- `dry_run_hash` records candidate output.
- `completed_hash` records final output.
- `policy_decision_id` records Cedar admission.
- `pack_overlay_result` records residency constraints.
- `dealset_evidence_id` records commercial proof when needed.
- `audit_event_id` seals replay.

## Implementation plan
- Step 1: Define replay categories and state transition contract.
- Step 2: Add Cedar policy for replay requester authority and category constraints.
- Step 3: Add dry-run projection comparison for canonical canvas operations.
- Step 4: Add vendor import replay transform reports for each displaced benchmark source.
- Step 5: Add snapshot repair with previous and new hash evidence.
- Step 6: Add export regeneration with redaction and watermark preservation.
- Step 7: Add template rematerialization guarded by DealSet evidence.
- Step 8: Add presence compaction that preserves membership evidence.
- Step 9: Add pack reclassification that applies IP-015 higher-restriction-wins.
- Step 10: Add emergency after-action replay that preserves IP-013 bypass evidence.
- Step 11: Add capacity admission so replay cannot starve live collaboration.
- Step 12: Add cost-budget emission per replay batch.
- Step 13: Add worker lease and idempotency behavior.
- Step 14: Add quarantine queues for transform mismatch and hash drift.
- Step 15: Add runbook entries for stuck replay, rollback, and quarantine review.
- Step 16: Add dashboards for lag, batch duration, reject rate, and replay cost.
- Step 17: Add scorecards for dry-run coverage and post-replay hash verification.
- Step 18: Add benchmark parity evidence for replay across six vendors.
- Step 19: Add audit export shape for replay decisions.
- Step 20: Add rollback bundle generation for each category.

## Operational controls
- Replay runs with worker authority, not user authority, but still requires Cedar admission.
- Replay batches are bounded by tenant, category, range, and cell.
- Live board-open and canvas-op-append receive priority over non-urgent replay.
- Emergency after-action replay can receive priority after live emergency traffic.
- Replay reads immutable operation logs and writes derived projections.
- Replay cannot modify canonical operation history.
- Replay stores dry-run reports before writes.
- Replay writes are idempotent by replay_id and batch cursor.
- Replay emits cost tags for tenant, category, source vendor, cell, and target projection.
- Replay degraded state is visible in board-open.
- Replay rollback restores prior projection hashes, not deleted user history.
- Quarantined replay requires human or workflow adjudication.
- Settlement evidence replay requires ADR-0314 proof.
- Pack reclassification replay requires IP-015 pack decision evidence.
- Worker credentials use short-lived sidecar-bound secrets.

## Failure modes
- Worker lease lost: stop batch, preserve cursor, allow safe resume.
- Dry-run hash mismatch: quarantine before writes.
- Completed hash mismatch: roll back projection and quarantine.
- Cedar unavailable: deny new replay and continue only already admitted safe pause.
- Audit-chain outage: pause before committing replay writes.
- Pack resolver stale: apply stricter policy or pause.
- DealSet evidence missing: block settlement replay and template rematerialization.
- Source vendor transform unknown: reject source object with reason.
- Capacity pressure: pause non-urgent replay.
- Emergency board active: deprioritize unrelated replay in the same cell.
- Export renderer unavailable: pause export regeneration only.
- Snapshot storage unavailable: pause snapshot repair and preserve cursor.
- Duplicate replay request: return prior replay_id and state.
- Cross-tenant source reference: deny and emit refusal evidence.
- Replay rollback fails: freeze target projection and escalate to runbook.

## Evidence and tests
- Evidence 1: Dry-run report exists before write for required categories.
- Evidence 2: Replay cannot mutate canonical operations.
- Evidence 3: Worker lease prevents concurrent writes to same target projection.
- Evidence 4: Vendor import replay records accepted, transformed, rejected, and quarantined objects.
- Evidence 5: Snapshot repair records previous and new hashes.
- Evidence 6: Export regeneration preserves redaction and watermark policy.
- Evidence 7: Template rematerialization requires DealSet evidence.
- Evidence 8: Pack reclassification applies higher-restriction-wins.
- Evidence 9: Emergency after-action replay preserves bypass_controls evidence.
- Evidence 10: Capacity admission prevents replay from starving live board traffic.
- Evidence 11: Cost-budget events include replay category and source vendor.
- Evidence 12: Benchmark parity maps replay paths for all six displaced vendors.
- Evidence 13: ADR-0321 matrix covers replay, rollback, tenant, Cedar, audit, pack, and benchmark anchors.
- Evidence 14: Quarantine tests cover hash drift and unknown transform.
- Evidence 15: Runbook drill covers stuck replay and rollback.

## Replay-specific domain and contract deltas
- Domain aggregate: `whiteboard_replay_job` owns replay category, source cursor, target projection, and final hash.
- Domain invariant: `whiteboard_replay_job.tenant_id` is fixed for the entire job.
- Domain invariant: `whiteboard_replay_job.source_cursor` advances only after batch audit seal.
- Domain invariant: `whiteboard_replay_job.target_projection` cannot change after dry-run.
- Domain invariant: canonical canvas operations are read-only replay source.
- Domain event `whiteboard.replay.dry_run_completed` records candidate hash and reject count.
- Domain event `whiteboard.replay.batch_committed` records cursor movement and target hash.
- Domain event `whiteboard.replay.quarantined` records mismatch and affected object ids.
- Domain event `whiteboard.replay.rollback_completed` records restored projection hash.
- OpenAPI delta: replay request includes `category`, `operation_range`, `source_vendor`, `dry_run_only`, and `target_projection`.
- OpenAPI delta: replay response includes `replay_id`, `dry_run_hash`, `estimated_cost_units`, and `approval_required`.
- AsyncAPI delta: emit `whiteboard.replay.batch.committed.v1` per committed batch.
- AsyncAPI delta: emit `whiteboard.replay.quarantined.v1` with reason code and remediation owner.
- Proto delta: internal `ReplayBatch` carries `source_cursor`, `worker_lease_id`, and `idempotency_key`.
- Proto delta: internal `ReplayTransformReport` carries accepted, transformed, rejected, and quarantined counts.
- Cedar fact: `principal.role` must allow replay category and target projection.
- Cedar fact: `context.replay_category == "settlement_evidence_replay"` requires DealSet evidence scope.
- Cedar fact: `context.replay_category == "pack_reclassification"` requires active pack policy version.
- Cedar fact: `context.source_vendor` must be one of admitted migration sources when vendor import replay is requested.
- Workflow decision: dry-run is mandatory before writes for vendor import, pack reclassification, and settlement evidence.
- Workflow decision: approval is mandatory when replay changes visible board projections.
- Workflow decision: quarantine blocks downstream completion and opens review.
- Workflow decision: rollback restores derived projection hash, not canonical operations.
- SLO: replay worker lease acquisition p95 target is 500 ms under normal cell load.
- SLO: replay batch commit p95 target is category-specific and excludes approval wait.
- SLO: replay quarantine alert target is 60 seconds.
- SLO: replay cannot consume more than its admitted replay_async capacity lane.
- Replay case: Miro Enterprise bulk sticky import maps source ids to canonical canvas operations.
- Replay case: Mural Enterprise facilitation artifacts map to template and canvas operation projections.
- Replay case: FigJam multiplayer order maps to operation sequence validation.
- Replay case: Lucidspark connectors map to diagram object projection and export renderer.
- Replay case: Whiteboard.fi classroom batches map to education pack retention.
- Replay case: Microsoft Whiteboard suite ownership maps to tenant board ownership evidence.
- Rollback: failed batch restores previous target projection hash and cursor.
- Rollback: bad transform rule marks source objects rejected and preserves original payload evidence.
- Rollback: settlement replay rollback leaves DealSet evidence unchanged and removes derived projection only.
- Test case: replay cannot write without worker lease.
- Test case: replay cannot cross tenant boundary.
- Test case: dry-run hash mismatch prevents write.
- Test case: duplicate replay request returns prior job state.
- Test case: vendor source id collision quarantines the object.
- Evidence field: `source_cursor` proves resumability.
- Evidence field: `target_projection_hash_before` and `target_projection_hash_after` prove derived write scope.
- Evidence field: `transform_report_id` links accepted, transformed, rejected, and quarantined objects.

## Acceptance criteria
- AC-001: Replay categories are explicit and bounded.
- AC-002: Dry-run, approval, run, quarantine, completion, and rollback states are defined.
- AC-003: ADR-0321 remains listed and unmodified.
- AC-004: All six benchmark names are present exactly.
- AC-005: Replay cannot mutate canonical operation history.
- AC-006: Settlement, residency, emergency, capacity, and cost cross-links are concrete.
- AC-007: Failure modes distinguish worker, policy, audit, pack, settlement, capacity, and transform failures.
- AC-008: Evidence includes hash and cursor requirements.
- AC-009: Repo-local references include backfill-replay.md and failure-modes.md.
- AC-010: Replay evidence records source cursor, transformed operation count, rejected operation count, sealed checkpoint id, and deterministic rollback range for each backfill batch.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
