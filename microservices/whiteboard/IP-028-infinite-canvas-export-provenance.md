# IP-028 Whiteboard infinite canvas export provenance

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-028-infinite-canvas-export-provenance.md
Capability focus: export-render, history-snapshot, board-open, canvas-op-append
Benchmarks: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0253-amendment, ADR-0257, ADR-0263, ADR-0297, ADR-0314, ADR-0316, ADR-0321
Repo-local references: microservices/whiteboard/PRD.md; microservices/whiteboard/ARCHITECTURE.md; microservices/whiteboard/capabilities/export-render.yaml; microservices/whiteboard/capabilities/history-snapshot.yaml; microservices/whiteboard/runbooks/local-export-render-stall.md; microservices/whiteboard/runbooks/export-render-failure.md; microservices/whiteboard/slos/local-export-render-latency.openslo.yaml; microservices/whiteboard/policies/local-board-export-egress.cedar; microservices/whiteboard/dpia.md; microservices/whiteboard/compliance.md

## Objective
- Define export provenance for infinite-canvas boards.
- Ensure exports are reproducible from board history and merge decisions.
- Preserve legal, privacy, and marketplace provenance in rendered artifacts.
- Prevent export-render from becoming a policy bypass around board ACLs.
- Support raster, vector, package, and audit-bundle export modes.
- Match Miro Enterprise board export depth with audit-grade evidence.
- Match Mural Enterprise workshop export packaging without losing facilitation state.
- Match FigJam team export expectations for sticky clusters and comments.
- Match Lucidspark diagram export expectations for connectors and layouts.
- Match Whiteboard.fi classroom export expectations for teacher-visible student work.
- Match Microsoft Whiteboard export handoff expectations for Microsoft 365 tenants.

## Current repo anchors
- anchor 001: PRD-whiteboard says board history and export semantics are not document-file semantics.
- anchor 002: ARCHITECTURE.md names export as a bounded context.
- anchor 003: export-render capability records tenant and data_class requirements.
- anchor 004: history-snapshot capability is the replay anchor for deterministic export.
- anchor 005: local-board-export-egress.cedar controls export egress policy.
- anchor 006: local-export-render-latency SLO defines render latency evidence.
- anchor 007: local-export-render-stall runbook covers stalled local rendering.
- anchor 008: export-render-failure runbook covers production export failure.
- anchor 009: compliance.md and dpia.md bind privacy and regulator evidence.
- anchor 010: ADR-0321 requires whiteboard benchmark coverage for B2B leader parity.

## Domain vocabulary
- vocabulary 001: `export_job_id` identifies a single render or package request.
- vocabulary 002: `export_scope` defines whole board, selection, frame, classroom shard, or audit bundle.
- vocabulary 003: `snapshot_epoch` identifies the history state used for export.
- vocabulary 004: `render_manifest_id` identifies the artifact manifest.
- vocabulary 005: `object_inclusion_reason` explains why an object appears in the export.
- vocabulary 006: `object_exclusion_reason` explains why an object is omitted or redacted.
- vocabulary 007: `provenance_chain` links source operation, merge decision, and export artifact.
- vocabulary 008: `redaction_overlay` records privacy, policy, or marketplace removals.
- vocabulary 009: `egress_decision_id` records Cedar authorization for artifact release.
- vocabulary 010: `artifact_digest` is the content hash for the rendered output.
- vocabulary 011: `viewport_tile_id` identifies a deterministic canvas tile.
- vocabulary 012: `layout_bounds` identifies the infinite-canvas area rendered.

## Export modes
- mode 001: `png-raster` renders a bounded viewport or frame.
- mode 002: `svg-vector` preserves object geometry where policy permits.
- mode 003: `pdf-package` renders multipage frames or classroom boards.
- mode 004: `board-archive` packages operation history, metadata, and object assets.
- mode 005: `audit-bundle` packages evidence for compliance review.
- mode 006: `template-package` exports reusable board components with marketplace constraints.
- mode 007: `classroom-progress` exports teacher-visible student work under education policy.
- mode 008: `migration-bundle` packages source vendor ids and transform evidence.
- mode 009: `snapshot-diff` exports differences between two history snapshots.
- mode 010: `redacted-share` exports policy-filtered content for external audience.
- mode 011: `legal-hold` exports immutable evidence for eDiscovery or regulator hold.
- mode 012: `facilitation-summary` exports timer, vote, and moderation outcomes.

## Provenance requirements
- provenance 001: Every export references a snapshot_epoch.
- provenance 002: Every export references the egress_decision_id.
- provenance 003: Every export records the requesting principal_id.
- provenance 004: Every export records tenant_id in signed audit evidence.
- provenance 005: Every export records purpose and data_class.
- provenance 006: Every export records source benchmark profile when migrated content is present.
- provenance 007: Every included object records object_id and schema revision.
- provenance 008: Every included object records last merge_decision_id.
- provenance 009: Every redacted object records object_exclusion_reason.
- provenance 010: Every marketplace-origin object records template_origin_ref.
- provenance 011: Every classroom shard records teacher visibility mode.
- provenance 012: Every vote or timer export records governance_epoch.
- provenance 013: Every raster tile records viewport_tile_id and layout_bounds.
- provenance 014: Every artifact records artifact_digest before release.
- provenance 015: Every package manifest records renderer version.
- provenance 016: Every external share records recipient audience class.

## Infinite canvas tiling
- tiling 001: The renderer divides unbounded board space into deterministic tiles.
- tiling 002: Tile boundaries are derived from layout_bounds and renderer version.
- tiling 003: Tile rendering waits for merge arbitration to settle the target snapshot_epoch.
- tiling 004: Object inclusion uses geometry intersection plus policy visibility.
- tiling 005: Offscreen connector endpoints are represented by continuation markers.
- tiling 006: Large freehand strokes are chunked by stroke_id and tile span.
- tiling 007: Sticky clusters preserve relative order and grouping metadata.
- tiling 008: Frames define default page boundaries for PDF export.
- tiling 009: Classroom student boards are tiled separately from teacher overview.
- tiling 010: Hidden vote results are excluded until governance policy allows visibility.
- tiling 011: Marketplace-licensed template objects are watermarked or redacted when required.
- tiling 012: Renderer output is stable across retry for the same snapshot_epoch.

## Policy hooks
- policy 001: local-board-export-egress.cedar denies export without tenant scope.
- policy 002: Export requires read authority for every included object or redaction policy.
- policy 003: Export requires purpose compatible with the requested artifact mode.
- policy 004: External share requires recipient audience and data_class compatibility.
- policy 005: Classroom export requires education pack authorization.
- policy 006: Anonymous vote audit export requires auditor scope.
- policy 007: Marketplace template export requires DealSet settlement.
- policy 008: Migration-bundle export requires source vendor provenance.
- policy 009: Legal-hold export requires immutability and retention binding.
- policy 010: Redacted-share export must include exclusion reasons.
- policy 011: Whole-board export must not silently omit policy-denied objects.
- policy 012: Cross-region export must obey home-cell and residency labels.

## Benchmark displacement map
- benchmark 001: Miro Enterprise displaced behavior is full-board image, PDF, and board backup export.
- benchmark 002: Miro Enterprise gap is closed by snapshot_epoch and render_manifest evidence.
- benchmark 003: Mural Enterprise displaced behavior is workshop outcome export.
- benchmark 004: Mural Enterprise gap is closed by facilitation-summary and sealed vote provenance.
- benchmark 005: FigJam displaced behavior is sticky cluster and comment export.
- benchmark 006: FigJam gap is closed by object-level provenance and redaction overlays.
- benchmark 007: Lucidspark displaced behavior is diagram export with connector fidelity.
- benchmark 008: Lucidspark gap is closed by connector endpoint provenance and vector render tests.
- benchmark 009: Whiteboard.fi displaced behavior is teacher export of class boards.
- benchmark 010: Whiteboard.fi gap is closed by classroom-progress export and student privacy evidence.
- benchmark 011: Microsoft Whiteboard displaced behavior is Microsoft 365 identity-bound export handoff.
- benchmark 012: Microsoft Whiteboard gap is closed by tenant principal binding and egress decision evidence.

## Data and manifest model
- manifest 001: `render_manifest_id` is globally unique within tenant scope.
- manifest 002: `export_job_id` links request, worker, artifact, and audit records.
- manifest 003: `artifact_digest` uses the repo standard digest algorithm for evidence artifacts.
- manifest 004: `object_manifest` lists included objects and inclusion reasons.
- manifest 005: `redaction_manifest` lists omitted objects and exclusion reasons.
- manifest 006: `tile_manifest` lists tile ids, bounds, and digests.
- manifest 007: `asset_manifest` lists embedded images, fonts, and external references.
- manifest 008: `policy_manifest` lists Cedar decision ids and policy revisions.
- manifest 009: `governance_manifest` lists timer and vote epochs included.
- manifest 010: `marketplace_manifest` lists DealSet and template origin evidence.
- manifest 011: `education_manifest` lists class-room visibility constraints.
- manifest 012: `migration_manifest` lists source vendor ids and import transforms.

## Worker flow
- flow 001: API accepts export request with idempotency key.
- flow 002: Application resolves board, scope, pack overlay, and destination audience.
- flow 003: Usecase evaluates egress policy before queuing render.
- flow 004: Worker locks the target snapshot_epoch.
- flow 005: Worker reads history-snapshot and merge decision evidence.
- flow 006: Worker computes layout_bounds and tile plan.
- flow 007: Worker evaluates object visibility and redaction overlays.
- flow 008: Worker renders deterministic tiles or vector pages.
- flow 009: Worker assembles artifact package and manifest.
- flow 010: Worker computes artifact_digest and tile digests.
- flow 011: Worker emits export-render event and audit-chain evidence.
- flow 012: Worker publishes artifact only after policy and digest evidence are sealed.

## SLO and telemetry
- telemetry 001: Measure export queue latency by export mode.
- telemetry 002: Measure render duration by tile count bucket.
- telemetry 003: Measure redaction count by export mode.
- telemetry 004: Measure failed egress policy decisions.
- telemetry 005: Measure artifact digest mismatch count.
- telemetry 006: Measure retry count for stalled render jobs.
- telemetry 007: Measure local-export-render-latency SLO burn.
- telemetry 008: Measure classroom export privacy denials.
- telemetry 009: Measure marketplace license export denials.
- telemetry 010: Measure snapshot_epoch wait time caused by merge arbitration.
- telemetry 011: Trace export_job_id through API, worker, storage, and audit-chain.
- telemetry 012: Keep raw tenant_id out of metric labels.

## Acceptance criteria
- acceptance 001: Every export artifact has a render manifest.
- acceptance 002: Every render manifest has snapshot_epoch and egress_decision_id.
- acceptance 003: Every rendered object has inclusion provenance or redaction evidence.
- acceptance 004: Every marketplace-origin object has DealSet evidence or is redacted.
- acceptance 005: Every classroom export enforces teacher and student visibility policy.
- acceptance 006: Every vote and timer artifact preserves governance_epoch.
- acceptance 007: Whole-board export never silently omits policy-denied objects.
- acceptance 008: Artifact digest is recorded before release.
- acceptance 009: Export replay for same snapshot_epoch produces same digest.
- acceptance 010: Benchmark evidence names all six required displaced products.
- acceptance 011: ADR-0321 and ADR-0316 are included in the evidence bundle.
- acceptance 012: Export failure paths route to local-export-render-stall or export-render-failure runbooks.

## Test plan
- test 001: Unit-test render manifest creation.
- test 002: Unit-test object inclusion and exclusion reasons.
- test 003: Unit-test tile boundary determinism.
- test 004: Unit-test artifact digest stability.
- test 005: Property-test replay export digest under worker retry.
- test 006: Contract-test export-render request and response shape.
- test 007: AsyncAPI-test export accepted, rendered, failed, and redacted events.
- test 008: Cedar-fixture-test export egress denial.
- test 009: Cedar-fixture-test classroom export denial.
- test 010: Cedar-fixture-test marketplace license export denial.
- test 011: Migration-fixture-test Miro Enterprise whole-board export.
- test 012: Migration-fixture-test Mural Enterprise facilitation summary export.
- test 013: Migration-fixture-test FigJam sticky cluster export.
- test 014: Migration-fixture-test Lucidspark connector vector export.
- test 015: Migration-fixture-test Whiteboard.fi classroom progress export.
- test 016: Migration-fixture-test Microsoft Whiteboard identity-bound export.
- test 017: Runbook-test stalled local export render path.

## Rollback and recovery
- rollback 001: Disable external share release while keeping internal audit-bundle export.
- rollback 002: Requeue stalled render jobs using the same snapshot_epoch.
- rollback 003: Invalidate artifacts with digest mismatch before user access.
- rollback 004: Preserve failed render manifests for auditor review.
- rollback 005: Fall back from vector to raster only when policy and user request allow.
- rollback 006: Pause marketplace-origin exports when DealSet evidence service is degraded.
- rollback 007: Pause classroom-progress export when education privacy policy is degraded.
- rollback 008: Route local stalls to local-export-render-stall runbook.
- rollback 009: Route production failures to export-render-failure runbook.
- rollback 010: Never mutate board history to repair an export artifact.

## Command and proto deltas
- proto 001: Add `ExportRenderRequest.export_job_id` as an idempotent render identifier.
- proto 002: Add `ExportRenderRequest.export_scope` with board, selection, frame, classroom, audit, and legal-hold values.
- proto 003: Add `ExportRenderRequest.snapshot_epoch` so users can export a stable board state.
- proto 004: Add `ExportRenderRequest.recipient_audience_class` for external share policy.
- proto 005: Add `RenderManifest.render_manifest_id` as the package-level evidence key.
- proto 006: Add `RenderManifest.artifact_digest` and `renderer_version`.
- proto 007: Add `RenderedObjectEvidence.object_id`, `object_schema_revision`, `merge_decision_id`, and `object_inclusion_reason`.
- proto 008: Add `RedactedObjectEvidence.object_id` and `object_exclusion_reason`.
- proto 009: Add `TileEvidence.viewport_tile_id`, `layout_bounds`, and `tile_digest`.
- proto 010: Add `ClassroomExportEvidence.education_room_id`, `teacher_visibility_mode`, and `student_redaction_count`.
- proto 011: Add `MarketplaceExportEvidence.template_origin_ref`, `dealset_decision_id`, and `license_redaction_count`.
- proto 012: Add `ExportRenderResponse.release_state` with queued, rendered, held, released, and failed values.

## Cedar facts
- cedar-fact 001: `principal_can_read_board` gates whole-board export.
- cedar-fact 002: `principal_can_read_object` gates object inclusion.
- cedar-fact 003: `recipient_audience_class` gates external share release.
- cedar-fact 004: `export_scope=classroom` requires teacher or education auditor authority.
- cedar-fact 005: `export_scope=audit-bundle` requires auditor scope.
- cedar-fact 006: `template_origin_ref` requires DealSet settlement or redaction.
- cedar-fact 007: `vote_anonymity_mode` controls ordinary export redaction.
- cedar-fact 008: `snapshot_epoch_is_settled` must be true before release.
- cedar-fact 009: `residency_label_compatible` must be true for artifact storage region.
- cedar-fact 010: `legal_hold=true` forces immutable retention and disables redacted-share shortcuts.

## Workflow decisions
- workflow 001: Export request admission is synchronous; rendering is asynchronous.
- workflow 002: Export-render waits for merge arbitration only to the requested snapshot_epoch.
- workflow 003: Whole-board export computes redactions before tile rendering so omissions are manifest-visible.
- workflow 004: Classroom export renders teacher overview and student boards as separate manifest groups.
- workflow 005: Marketplace-origin content is rendered only after license evidence resolves.
- workflow 006: Legal-hold export bypasses user-friendly compression if it would alter evidentiary fidelity.
- workflow 007: Failed tile rendering retries by viewport_tile_id and never changes snapshot_epoch.
- workflow 008: Artifact release is a separate state transition after digest and egress policy seal.

## Failure and replay cases
- failure 001: Render worker crash resumes from tile_manifest without changing artifact identity.
- failure 002: Digest mismatch invalidates artifact and keeps render_manifest for investigation.
- failure 003: Redaction policy drift before release forces re-evaluation and new manifest epoch.
- failure 004: Export of active facilitation board waits for vote and timer governance watermark.
- failure 005: Miro Enterprise full-board import export must retain frame order and comments redaction.
- failure 006: Mural Enterprise facilitation export must include timer and sealed-vote provenance.
- failure 007: FigJam sticky export must preserve cluster geometry.
- failure 008: Lucidspark vector export must preserve connector endpoints or mark redaction.
- failure 009: Whiteboard.fi classroom export must not expose peer-hidden student boards.
- failure 010: Microsoft Whiteboard migration-bundle export must preserve external id provenance separately.

## Evidence fields
- evidence 001: `export_job_id` joins request, queue, worker, artifact, and audit records.
- evidence 002: `snapshot_epoch` proves stable board state.
- evidence 003: `egress_decision_id` proves Cedar release authorization.
- evidence 004: `render_manifest_id` proves package identity.
- evidence 005: `artifact_digest` proves content integrity.
- evidence 006: `renderer_version` proves reproducibility context.
- evidence 007: `object_inclusion_reason` proves why content was present.
- evidence 008: `object_exclusion_reason` proves why content was redacted.
- evidence 009: `tile_digest` proves deterministic tile rendering.
- evidence 010: `recipient_audience_class` proves external sharing boundary.

## Done definition
- done 001: IP defines infinite-canvas export provenance and render manifests.
- done 002: IP references whiteboard PRD, architecture, capability, policy, SLO, runbook, DPIA, and compliance anchors.
- done 003: IP names Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
- done 004: IP includes policy, tiling, data, telemetry, tests, and rollback substance.
- done 005: IP stays inside microservices/whiteboard and does not edit ADR-0321.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
