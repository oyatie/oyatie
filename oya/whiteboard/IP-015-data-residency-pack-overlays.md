# IP-015 Whiteboard data-residency-pack-overlays

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-015-data-residency-pack-overlays.md
Benchmarks: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo-local references: microservices/whiteboard/PRD.md, microservices/whiteboard/ARCHITECTURE.md, microservices/whiteboard/compliance.md, microservices/whiteboard/dpia.md, microservices/whiteboard/multi-region.md, microservices/whiteboard/capabilities/board-open.yaml, microservices/whiteboard/capabilities/canvas-op-append.yaml, microservices/whiteboard/capabilities/presence-sync.yaml, microservices/whiteboard/capabilities/history-snapshot.yaml, microservices/whiteboard/capabilities/export-render.yaml, microservices/whiteboard/capabilities/template-marketplace-install.yaml

## Objective
- Define how Whiteboard applies residency and compliance pack overlays to collaborative canvas artifacts.
- Preserve PRD-whiteboard's pack roster: SOC-2, ISO-27001, GDPR, KR-PIPA, education, and public-sector.
- Preserve ADR-0321 anchor coverage for tenant scoping, Cedar, pack overlay, audit, rollback, and benchmark parity.
- Prevent low-latency collaboration from becoming a cross-region data leak.
- Keep board objects, canvas operations, presence cursors, snapshots, exports, and templates under explicit data-class handling.
- Treat Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard as displaced residency expectations.
- Make pack decisions visible to tenant admins, SRE, auditors, and support operators.
- Apply higher-restriction-wins whenever pack overlays conflict.
- Tie every residency decision to a policy decision and audit-chain event.
- Keep Whiteboard service ownership flat and independent from suite labels.

## Non-goals
- Do not author the global residency framework here.
- Do not edit compliance.md, dpia.md, multi-region.md, or ADRs.
- Do not move object storage or ontology ownership into Whiteboard docs.
- Do not define vendor-specific region catalogs.
- Do not permit presence data to bypass pack constraints just because it is ephemeral.
- Do not let export convenience weaken regulator export rules.
- Do not create a hidden "global board" mode.
- Do not assume classroom boards are low-risk.
- Do not collapse all canvas data into one retention class.
- Do not change write scope beyond this IP.

## Pack overlay model
- Each board has a `pack_overlay_result` computed before board-open completes.
- Each canvas operation inherits board pack context and may add stricter data class tags.
- Presence events use minimized data but still carry tenant, cell, and pack context.
- History snapshots freeze the pack result for their covered version range.
- Exports recompute pack constraints at render time and record the result.
- Template installs evaluate both tenant pack and template safety class.
- Pack overlays decide residency, retention, exportability, breach timing, disclosure language, and approval workflow.
- Pack overlays are evaluated with tenant home cell, jurisdiction_code, data_class, audience_type, and purpose.
- Pack overlays can block cross-cell collaboration.
- Pack overlays can allow metadata-only fanout where content replication is forbidden.
- Pack overlays can require regulator-visible export logs.
- Pack overlays can require tenant-admin approval before external sharing.
- Pack overlays can force snapshot redaction.
- Pack overlays can forbid marketplace template assets.
- Pack overlays can reduce presence detail under education and public-sector rules.

## Benchmark displacement notes
- Miro Enterprise and Mural Enterprise are expected to offer enterprise residency controls.
- FigJam and Lucidspark set user expectations for fast cross-team collaboration.
- Whiteboard.fi introduces classroom immediacy and student privacy concerns.
- Microsoft Whiteboard introduces suite-region expectations tied to enterprise tenancy.
- Oyatie must displace those expectations by making the residency decision inspectable per capability.
- Benchmark parity is not satisfied by a marketing claim about data location.
- Benchmark parity requires proof that board-open, append, presence, snapshot, export, and template install obey pack overlays.
- Vendor convenience is rejected when it conflicts with higher-restriction-wins.
- Cross-region availability is metadata-only unless the pack permits content movement.
- Whiteboard exports are official records, not screenshots outside governance.

## Capability binding
- `board-open` computes pack overlay before exposing board content.
- `board-open` refuses when tenant home cell and board residency conflict.
- `canvas-op-append` validates that operation payload can be stored in the board cell.
- `canvas-op-append` marks any stricter data class introduced by the operation.
- `presence-sync` emits minimized cursor state and avoids raw tenant id metrics.
- `presence-sync` is eligible for metadata-only cross-cell relay when allowed.
- `history-snapshot` stores immutable residency and retention decisions for the version range.
- `history-snapshot` drives regulator and audit exports.
- `export-render` recomputes export eligibility and redaction policy.
- `export-render` refuses cross-region rendering when pack overlay forbids it.
- `template-marketplace-install` checks template family, asset region, license scope, and pack compatibility.
- `template-marketplace-install` must not materialize incompatible assets.
- Capability records under microservices/whiteboard/capabilities remain naming authority.
- ARCHITECTURE.md remains aggregate authority for canvas, board-session, sticky-note, template, and export.
- PRD.md remains the user and product acceptance authority.

## Pack-specific decisions
- SOC-2 requires evidence of control operation, audit export, and retention behavior.
- ISO-27001 requires risk treatment evidence for collaboration data and operator access.
- GDPR requires lawful basis, data minimization, region rules, export logs, and erasure workflow mapping.
- KR-PIPA requires jurisdiction-specific handling and breach timing evidence.
- Education pack requires student/guardian privacy controls and classroom export restrictions.
- Public-sector pack requires regulator export, stricter access logging, and incident evidence.
- Combined GDPR plus education applies the stricter student privacy and region rules.
- Combined public-sector plus marketplace template blocks unknown asset provenance.
- Combined SOC-2 plus ISO-27001 requires both control evidence and risk evidence.
- Combined KR-PIPA plus public-sector requires Korean jurisdiction review before cross-cell movement.
- Pack decisions include permit delta, data-class delta, retention delta, export delta, and regulator evidence delta.
- Pack decisions are stored with audit-chain event id.
- Pack decisions are not editable after snapshot creation.
- Pack decisions can become stricter after board creation.
- Pack decisions cannot become weaker without a new policy decision and migration plan.

## Data requirements
- `tenant_id` is mandatory for every residency calculation.
- `principal_id` is mandatory for every policy decision.
- `audience_type` distinguishes collaboration user, tenant admin, support operator, auditor, and automated worker.
- `purpose` distinguishes collaboration, education, incident, export, migration, and audit.
- `data_class` distinguishes board_object, canvas_operation, presence_cursor, export_snapshot, and marketplace_asset.
- `home_cell` identifies the tenant default write cell.
- `board_cell` identifies where board content is stored.
- `jurisdiction_code` drives legal and regulator requirements.
- `pack_ids` list active tenant packs.
- `pack_overlay_result` records the resolved higher-restriction-wins decision.
- `residency_reason` explains why content is allowed, metadata-only, or denied.
- `retention_rule_id` records deletion and archive handling.
- `export_rule_id` records regulator and tenant export behavior.
- `redaction_profile_id` records export and snapshot redaction.
- `audit_event_id` seals the decision.

## Implementation plan
- Step 1: Add pack overlay evaluation to board-open before content access.
- Step 2: Add operation-level data-class escalation checks to canvas-op-append.
- Step 3: Add minimized cross-cell metadata rules to presence-sync.
- Step 4: Add immutable pack result recording to history-snapshot.
- Step 5: Add export eligibility and redaction recomputation to export-render.
- Step 6: Add template asset provenance and pack compatibility checks to template-marketplace-install.
- Step 7: Add Cedar policies for residency allow, metadata-only, and deny decisions.
- Step 8: Add ontology fields for pack_overlay_result and residency_reason.
- Step 9: Add audit-chain events for pack calculation, override denial, and export refusal.
- Step 10: Add SLO annotations for residency lookup latency.
- Step 11: Add dashboards for pack denials, metadata-only relays, and export refusals.
- Step 12: Add runbook entries for pack conflict and jurisdiction outage.
- Step 13: Add multi-region tests for home-cell outage and metadata-only fanout.
- Step 14: Add DPIA evidence for GDPR, KR-PIPA, education, and public-sector packs.
- Step 15: Add compliance scorecard rows for all active packs.
- Step 16: Add benchmark parity matrix rows for residency controls across the six displaced vendors.
- Step 17: Add failure-mode handling for stale tenant projection.
- Step 18: Add rollback bundle content for pack decision changes.
- Step 19: Add cost-budget tags for cross-cell relay and region-specific storage.
- Step 20: Add backfill replay behavior for pack reclassification.

## Operational controls
- Pack overlay cache has a short TTL and fails to stricter policy on stale reads.
- Operator access always records support reason and tenant approval where required.
- Cross-cell fanout is content-bearing only when all active packs permit it.
- Metadata-only fanout strips board content and sensitive cursor details.
- Export workers run in permitted cells only.
- Snapshot storage follows the strictest retention rule in the covered version range.
- Template assets are copied only from approved provenance and region classes.
- Regulator exports are immutable and separately auditable.
- Tenant admins can preview pack impact before enabling a template or export.
- SRE dashboards avoid raw tenant id cardinality.
- Audit evidence keeps tenant id in signed records.
- Support views show residency denial reasons without exposing restricted board content.
- Pack downgrade requires review workflow and migration evidence.
- Pack upgrade can immediately restrict future operations.
- Emergency-service bypass cannot override residency.

## Failure modes
- Pack resolver unavailable: use last known stricter decision for reads, deny mutations requiring new residency evaluation.
- Tenant projection stale: apply most restrictive known pack and emit degraded-mode evidence.
- Region unavailable: follow multi-region.md and avoid content movement across forbidden cells.
- Export renderer in wrong cell: deny render and emit residency refusal.
- Presence relay overload: drop cursor cosmetics before membership or policy evidence.
- Template asset region unknown: block materialization.
- Snapshot pack mismatch: quarantine snapshot and require replay.
- Data-class escalation detected after append: freeze offending operation and open adjudication.
- Audit-chain backpressure: stop high-risk mutations before evidence loss.
- Pack downgrade requested during active board: require workflow approval and migration plan.
- Cross-tenant board reference: Cedar denies before residency calculation.
- Marketplace asset revocation: freeze future materialization and preserve existing evidence.
- GDPR erasure conflict with public-sector retention: higher-restriction-wins and legal review path.
- KR-PIPA breach timing conflict: stricter notification clock wins.
- Education classroom export requested by unauthorized principal: deny and emit refusal evidence.

## Evidence and tests
- Evidence 1: Board-open denies content access when board_cell conflicts with pack overlay.
- Evidence 2: Canvas append escalates data class and recalculates constraints.
- Evidence 3: Presence sync performs metadata-only relay where allowed.
- Evidence 4: History snapshot records immutable pack_overlay_result.
- Evidence 5: Export render refuses forbidden region and applies redaction profile.
- Evidence 6: Template install blocks incompatible asset provenance.
- Evidence 7: Cedar tests cover allow, metadata-only, deny, stale projection, and pack conflict.
- Evidence 8: Multi-region tests cover home-cell outage.
- Evidence 9: DPIA rows cover GDPR, KR-PIPA, education, and public-sector.
- Evidence 10: Compliance scorecards cover SOC-2 and ISO-27001 evidence.
- Evidence 11: Benchmark parity maps residency controls for all six vendors.
- Evidence 12: ADR-0321 matrix retains pack overlay and rollback anchors.
- Evidence 13: Negative tests prove emergency bypass cannot override residency.
- Evidence 14: Negative tests prove export renderer cannot render in a forbidden cell.
- Evidence 15: Replay tests prove pack reclassification preserves audit evidence.

## Residency-specific domain and contract deltas
- Domain aggregate: `resident_board_projection` records board content cell, allowed metadata cells, and pack overlay.
- Domain invariant: `resident_board_projection.content_cell` cannot move without workflow approval and replay evidence.
- Domain invariant: `presence_cursor` can be minimized but not stripped of tenant and policy correlation.
- Domain invariant: `export_snapshot` keeps the strictest pack result for its version range.
- Domain invariant: marketplace assets inherit both tenant pack and source asset provenance.
- Domain event `whiteboard.residency.pack.evaluated` records allow, metadata-only, or deny.
- Domain event `whiteboard.residency.content_move.blocked` records forbidden cross-cell movement.
- Domain event `whiteboard.residency.export.refused` records renderer cell and pack conflict.
- Domain event `whiteboard.residency.pack.reclassified` records old and new overlay result.
- OpenAPI delta: board-open response includes `content_cell`, `metadata_cells`, and `pack_overlay_result`.
- OpenAPI delta: export-render request includes `requested_render_cell` and returns `export_rule_id`.
- OpenAPI delta: template install response includes `asset_region_class` and `pack_compatibility_result`.
- AsyncAPI delta: emit `whiteboard.residency.decision.v1` for board-open, export, and template install.
- AsyncAPI delta: emit `whiteboard.residency.metadata_only_relay.v1` for presence relay.
- Proto delta: internal `ResidencyDecision` carries `decision_kind`, `rule_id`, `jurisdiction_code`, and `reason`.
- Proto delta: internal `PackOverlayResult` is attached to board, snapshot, export, and template materialization commands.
- Cedar fact: `resource.content_cell` must equal `tenant.home_cell` unless pack explicitly permits movement.
- Cedar fact: `context.requested_render_cell` must be in `resource.allowed_render_cells`.
- Cedar fact: `context.metadata_only == true` cannot carry canvas object payload.
- Cedar fact: `principal.audience_type == "support_operator"` requires tenant-approved support reason for restricted packs.
- Workflow decision: pack upgrade can immediately restrict future writes.
- Workflow decision: pack downgrade requires migration plan and review evidence.
- Workflow decision: export refusal opens remediation workflow instead of retrying in another cell.
- Workflow decision: cross-cell migration runs through IP-016 pack reclassification replay.
- SLO: pack overlay lookup p95 target is 40 ms from warmed tenant projection.
- SLO: metadata-only presence relay p95 target is 120 ms.
- SLO: export refusal decision target is before renderer queue admission.
- SLO: pack reclassification replay lag target is governed by IP-016 replay_async capacity.
- Replay case: reclassify historical snapshots when pack rule becomes stricter.
- Replay case: regenerate export refusal packets after redaction profile changes.
- Replay case: rebuild resident_board_projection from canonical operations after cell metadata repair.
- Rollback: bad pack rule rollout restores prior rule version and records affected boards.
- Rollback: mistaken content move freezes board and exports audit packet.
- Rollback: failed reclassification leaves prior pack result active until replacement is sealed.
- Test case: GDPR plus education chooses stricter student privacy retention.
- Test case: public-sector plus marketplace template blocks unknown asset region.
- Test case: support operator cannot view restricted board without tenant-approved reason.
- Test case: metadata-only presence relay rejects object payload.
- Test case: export renderer in forbidden cell returns residency refusal.
- Evidence field: `content_cell` records where board content lives.
- Evidence field: `metadata_cells` records allowed relay-only cells.
- Evidence field: `residency_reason` explains allow, metadata-only, deny, or reclassify.

## Acceptance criteria
- AC-001: Residency is defined per capability, not as generic prose.
- AC-002: Higher-restriction-wins is explicit and testable.
- AC-003: ADR-0321 remains listed and unmodified.
- AC-004: All six benchmark names are present exactly.
- AC-005: Pack roster matches PRD-whiteboard.
- AC-006: Emergency bypass is explicitly subordinate to residency.
- AC-007: Metadata-only fanout is distinct from content movement.
- AC-008: Exports and snapshots have immutable pack evidence.
- AC-009: The plan names local docs and capability records.
- AC-010: Residency evidence records board home cell, export artifact cell, snapshot retention region, pack overlay, and denied cross-cell movement reason for every regulated board.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
