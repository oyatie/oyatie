# IP-003 Whiteboard Ontology Projection

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-003-ontology-projection.md
Planning lane: B2B-leader IP substance deepening pass
Primary concern: library-first ontology projection for whiteboard boards, sessions, canvas objects, presence, snapshots, exports, and templates
Local references: microservices/whiteboard/PRD.md; microservices/whiteboard/ARCHITECTURE.md; microservices/whiteboard/manifest.json; microservices/whiteboard/catalog/oya-whiteboard-canvas-collaboration-domain.yaml; microservices/whiteboard/catalog/oya-whiteboard-canvas-collaboration-kernel.yaml; microservices/whiteboard/catalog/oya-whiteboard-canvas-collaboration-usecase.yaml; microservices/whiteboard/contracts/whiteboard-v1.proto; microservices/whiteboard/contracts/local-operations-v1.proto
Capability references: whiteboard-board-open; whiteboard-canvas-op-append; whiteboard-presence-sync; whiteboard-history-snapshot; whiteboard-export-render; whiteboard-template-marketplace-install
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Benchmark displacement set: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard

## Executive Intent
- Ontology projection turns whiteboard collaboration state into governed enterprise objects without letting vendor object models become Oyatie source of truth.
- The projection must support Miro Enterprise boards, Mural Enterprise rooms, FigJam canvases, Lucidspark diagrams, Whiteboard.fi class boards, and Microsoft Whiteboard tenant boards as imports, not as native boundary definitions.
- ADR-0321 requires B2B leader coverage; this IP makes that coverage queryable through canonical ontology records.
- ADR-0316 prevents the projection from creating vendor-shaped services; product names remain benchmark inputs and UX labels.
- ADR-0243 and the capability records require library-first ontology reads, so projection must be deterministic and local-callable.
- ADR-0314 requires marketplace template provenance and DealSet settlement to remain visible in projected template objects.
- ADR-0105 requires projection code to stay in domain/usecase/kernel layers and expose catalog registration without mixing transport concerns.
- Projection is not a copy of raw board payload; it is a governed view containing identifiers, relationships, policy context, provenance, and audit links.
- Projection must allow other microservices to reason about whiteboard artifacts without reading raw strokes or vendor payloads.
- Projection must preserve enough context for migration replay, export audit, and compliance pack evidence.

## Projection Scope
- Project Board as the durable tenant-owned collaboration container.
- Project BoardSession as the live collaboration session bound to Board.
- Project CanvasOperation as the append-only operational mutation record.
- Project CanvasObject as the materialized board element derived from operations.
- Project StickyNote as a CanvasObject specialization with author and text classification.
- Project Shape as a CanvasObject specialization with geometry and semantic labels.
- Project Connector as a CanvasObject specialization linking shape endpoints.
- Project Frame as a CanvasObject specialization for spatial grouping.
- Project CommentThread as a governed annotation object.
- Project PresenceCursor as a short-lived but governed collaboration signal.
- Project HistorySnapshot as a point-in-time board state projection.
- Project ExportRender as a generated artifact with retention and egress controls.
- Project TemplateInstall as a tenant installation of first-party or marketplace template material.
- Project TemplateSource as the provenance record for imported or marketplace templates.
- Project VendorImport as the source-vendor mapping for displaced benchmarks.
- Project DealSetBinding as the commercial settlement link for template material.
- Project PolicyDecisionRef as the link to Cedar decision evidence.
- Project AuditChainRef as the link to immutable audit events.
- Project ResidencyRef as the link to data-residency overlays.
- Project RetentionRef as the link to pack-specific retention requirements.

## Canonical Object Fields
- Board includes tenant_id.
- Board includes board_id.
- Board includes title.
- Board includes tenant_home_cell.
- Board includes created_by_principal_id.
- Board includes created_at.
- Board includes policy_pack_set.
- Board includes data_class=board_object.
- Board includes source_vendor when migrated.
- Board includes source_object_id when migrated.
- Board includes ontology_projection_version.
- Board includes audit_chain_target.
- BoardSession includes session_id.
- BoardSession includes board_id.
- BoardSession includes tenant_id.
- BoardSession includes audience_type.
- BoardSession includes facilitator_principal_ids.
- BoardSession includes participant_principal_ids.
- BoardSession includes started_at and ended_at.
- BoardSession includes session_state.
- BoardSession includes meeting_binding when Meet or Messenger substrate context is present.
- CanvasOperation includes operation_id.
- CanvasOperation includes board_id.
- CanvasOperation includes session_id when live.
- CanvasOperation includes operation_kind.
- CanvasOperation includes idempotency_key.
- CanvasOperation includes principal_id.
- CanvasOperation includes created_at.
- CanvasOperation includes scope_hash.
- CanvasOperation includes policy_decision_ref.
- CanvasObject includes object_id.
- CanvasObject includes board_id.
- CanvasObject includes object_kind.
- CanvasObject includes geometry.
- CanvasObject includes z_order.
- CanvasObject includes author_principal_id.
- CanvasObject includes last_operation_id.
- CanvasObject includes data_class.
- CanvasObject includes source_vendor when imported.
- CanvasObject includes source_object_id when imported.

## Relationship Graph
- Board belongs_to Tenant.
- Board located_in TenantHomeCell.
- Board governed_by PolicyPackSet.
- Board has_many BoardSessions.
- Board has_many CanvasOperations.
- Board has_many CanvasObjects.
- Board has_many HistorySnapshots.
- Board has_many ExportRenders.
- Board may_have VendorImport.
- Board may_have TemplateInstall.
- BoardSession belongs_to Board.
- BoardSession has_many PresenceCursors.
- BoardSession has_many CanvasOperations.
- BoardSession may_bind Meet session.
- BoardSession may_bind Messenger thread.
- CanvasOperation belongs_to Board.
- CanvasOperation may_materialize CanvasObject.
- CanvasOperation authorized_by PolicyDecisionRef.
- CanvasOperation audited_by AuditChainRef.
- CanvasObject belongs_to Board.
- Connector links two CanvasObject endpoints.
- StickyNote may_reference CommentThread.
- HistorySnapshot derived_from Board and CanvasOperation range.
- ExportRender derived_from HistorySnapshot or Board.
- TemplateInstall derived_from TemplateSource.
- TemplateInstall settled_by DealSetBinding.
- VendorImport maps source_vendor to source_object_id.
- ResidencyRef constrains HistorySnapshot and ExportRender.
- RetentionRef constrains HistorySnapshot, ExportRender, and AuditChainRef.

## Benchmark Projection Rules
- Miro Enterprise workspace maps to VendorImport.source_workspace_id, not tenant_id.
- Miro Enterprise board maps to Board.source_object_id and source_vendor=Miro Enterprise.
- Miro Enterprise frame maps to Frame when geometry and hierarchy are present.
- Miro Enterprise sticky note maps to StickyNote with author provenance.
- Miro Enterprise template maps to TemplateSource and requires DealSetBinding when commercial.
- Mural Enterprise room maps to BoardSession or Board grouping metadata, not a service boundary.
- Mural Enterprise mural maps to Board.source_object_id and source_vendor=Mural Enterprise.
- Mural Enterprise facilitator role maps to BoardSession.facilitator_principal_ids.
- Mural Enterprise voting or timer metadata maps to workflow template annotations when present.
- Mural Enterprise template maps to TemplateSource with settlement evidence.
- FigJam file maps to Board.source_object_id and source_vendor=FigJam.
- FigJam section maps to Frame.
- FigJam widget maps to CanvasObject with app_provenance metadata.
- FigJam cursor maps to PresenceCursor with presence_cursor data_class.
- FigJam comment maps to CommentThread with principal or external actor evidence.
- Lucidspark board maps to Board.source_object_id and source_vendor=Lucidspark.
- Lucidspark diagram shape maps to Shape.
- Lucidspark connector maps to Connector with endpoint references.
- Lucidspark export maps to ExportRender with diagram semantics preserved.
- Lucidspark template maps to TemplateSource with library provenance.
- Whiteboard.fi class maps to BoardSession roster metadata under education pack.
- Whiteboard.fi student board maps to Board with education pack and roster references.
- Whiteboard.fi teacher broadcast maps to BoardSession facilitator event metadata.
- Microsoft Whiteboard board maps to Board.source_object_id and source_vendor=Microsoft Whiteboard.
- Microsoft Whiteboard Teams meeting link maps to BoardSession.meeting_binding.
- Microsoft Whiteboard Loop or Office linkage maps to external_object_ref without service ownership transfer.

## Projection Boundaries
- Projection must not expose raw stroke payloads to services that only need metadata.
- Projection must not store vendor access control as canonical authorization.
- Projection must not infer tenant from source workspace, room, file, class, or meeting.
- Projection must not treat public links as tenant membership.
- Projection must not erase source_vendor after import.
- Projection must not erase source_object_id after import.
- Projection must not collapse BoardSession and Board into one object.
- Projection must not collapse PresenceCursor into CanvasOperation.
- Projection must not classify cursor data as ungoverned telemetry.
- Projection must not map marketplace template material without DealSetBinding.
- Projection must not expose export URLs without ExportRender residency and retention references.
- Projection must not let ontology reads bypass Cedar authorization.
- Projection must not let ontology projection create cross-tenant joins by default.
- Projection must not add vendor-specific object names to service boundaries.
- Projection must not change ADR-0321.
- Projection must not change manifest dependencies.
- Projection must not change capability record required fields.
- Projection must not bypass audit_chain_target.
- Projection must not create a new database shared with adjacent microservices.
- Projection must not make source-vendor data the canonical board state.

## Implementation Steps
- Define projection version whiteboard-ontology-v1.
- Define Board projection builder from BoardScope and board store metadata.
- Define BoardSession projection builder from session membership state.
- Define CanvasOperation projection builder from command envelopes.
- Define CanvasObject projection builder from materialized operation results.
- Define PresenceCursor projection builder from presence-sync events.
- Define HistorySnapshot projection builder from snapshot worker output.
- Define ExportRender projection builder from export worker output.
- Define TemplateInstall projection builder from template-marketplace-install usecase.
- Define VendorImport projection builder for benchmark migration inputs.
- Define DealSetBinding projection builder from marketplace settlement status.
- Define PolicyDecisionRef projection builder from IP-002 policy evidence.
- Define AuditChainRef projection builder from audit event emission.
- Define ResidencyRef projection builder from data-residency policy outputs.
- Define RetentionRef projection builder from compliance pack overlays.
- Register domain catalog metadata in oya-whiteboard-canvas-collaboration-domain.yaml.
- Register kernel catalog metadata in oya-whiteboard-canvas-collaboration-kernel.yaml.
- Register usecase catalog metadata in oya-whiteboard-canvas-collaboration-usecase.yaml.
- Align proto messages in whiteboard-v1.proto with projection ids.
- Align local operations proto with replay-safe projection ids.
- Expose projection read APIs only after policy evaluation.

## Data Quality Rules
- Projection ids are deterministic within tenant_id and board_id.
- Projection ids do not include vendor display names.
- Geometry fields use canonical units and coordinate origin.
- Text-bearing fields carry data_class and redaction eligibility.
- Author fields use principal_id or external_actor_ref.
- Unknown imported authors map to external_actor_ref with source_vendor and source_actor_id.
- Imported timestamps retain source timestamp and import timestamp.
- Source timestamps never override service audit timestamps.
- Board title is display data and cannot authorize access.
- Object labels are display data and cannot authorize access.
- Connector endpoints must reference existing projected CanvasObjects.
- Frame membership must reference existing projected CanvasObjects.
- HistorySnapshot ranges must reference operation ids.
- ExportRender must reference snapshot id or board id.
- TemplateInstall must reference TemplateSource and DealSetBinding when applicable.
- PresenceCursor must expire according to presence freshness rules.
- Projection version must be stored on every projected record.
- Projection errors must include object kind and source_vendor when safe.
- Projection errors must not include raw payloads.
- Projection errors must be replayable from preserved source ids.

## Observability
- Emit whiteboard_ontology_projection_total by object_kind, result, capability, and source_vendor.
- Emit whiteboard_ontology_projection_seconds by object_kind and result.
- Emit whiteboard_ontology_projection_error_total by reason and source_vendor.
- Emit whiteboard_ontology_vendor_import_total by benchmark name.
- Emit whiteboard_ontology_relationship_total by relationship kind.
- Emit trace span whiteboard.ontology.project_board.
- Emit trace span whiteboard.ontology.project_session.
- Emit trace span whiteboard.ontology.project_operation.
- Emit trace span whiteboard.ontology.project_object.
- Emit trace span whiteboard.ontology.project_vendor_import.
- Emit audit event whiteboard.ontology.projected for accepted projections.
- Emit audit event whiteboard.ontology.projection_failed for failed projections.
- Add dashboard dimension ontology_projection_version.
- Add dashboard dimension object_kind.
- Add dashboard dimension source_vendor.
- Add dashboard dimension capability_record_name.
- Add dashboard dimension policy_pack_set.
- Route projection failures to local-crdt-merge-conflict when operation materialization fails.
- Route projection failures to board-history-corruption when snapshot ranges fail.
- Route projection failures to template-import-rollback when template source projection fails.
- Route projection failures to region-affinity-mismatch when tenant_home_cell conflicts with projection target.

## Test Plan
- Unit test Board projection requires tenant_id.
- Unit test Board projection requires board_id.
- Unit test Board projection preserves tenant_home_cell.
- Unit test BoardSession projection requires active session_id.
- Unit test CanvasOperation projection requires operation_id.
- Unit test CanvasOperation projection preserves idempotency_key.
- Unit test PresenceCursor projection carries presence_cursor data_class.
- Unit test HistorySnapshot projection preserves operation range.
- Unit test ExportRender projection preserves residency reference.
- Unit test TemplateInstall projection requires TemplateSource.
- Unit test TemplateInstall projection requires DealSetBinding for marketplace material.
- Unit test VendorImport accepts Miro Enterprise.
- Unit test VendorImport accepts Mural Enterprise.
- Unit test VendorImport accepts FigJam.
- Unit test VendorImport accepts Lucidspark.
- Unit test VendorImport accepts Whiteboard.fi.
- Unit test VendorImport accepts Microsoft Whiteboard.
- Unit test VendorImport rejects generic Miro.
- Unit test VendorImport rejects generic Lucid when Lucidspark is required.
- Property test projection ids are stable for same tenant and source ids.
- Property test projection ids differ across tenants.
- Relationship test Connector endpoints resolve to projected Shape ids.
- Relationship test Frame membership resolves to projected object ids.
- Replay test projection rebuilds from stored operation ids.
- Replay test projection preserves source_vendor across replays.
- Contract test proto carries projection ids.
- Contract test REST projection reads require policy allow.
- Audit test projection success emits whiteboard.ontology.projected.
- Audit test projection failure emits whiteboard.ontology.projection_failed.
- Dashboard test projection metrics include object_kind and source_vendor.

## Acceptance Criteria
- Every whiteboard capability can emit or read ontology projections without raw vendor payload dependency.
- Every projected record includes tenant_id, projection version, and audit linkage.
- Every imported benchmark record preserves source_vendor and source_object_id.
- Every marketplace template projection preserves DealSetBinding.
- Every projection read is guarded by policy.
- Every projection write is traceable to a capability, command, worker, or replay.
- Every graph relationship is tenant-bounded.
- Every live-session object is distinguishable from durable board state.
- Every export and snapshot projection includes residency and retention references.
- Every projection error is observable without leaking raw canvas payload.
- Every projection can be rebuilt from command, outbox, or replay evidence.
- No projection turns a vendor object model into an Oyatie service boundary.
- No projection bypasses ADR-0321, ADR-0314, or ADR-0316 constraints.
- No projection requires editing ADR-0321.

## Title-Specific Command, Event, And Proto Deltas
- BoardProjected event is emitted when board metadata becomes ontology-readable.
- BoardSessionProjected event is emitted when live session facts become ontology-readable.
- CanvasOperationProjected event is emitted when an accepted operation receives graph identity.
- CanvasObjectProjected event is emitted when CRDT merge materializes an object.
- PresenceProjected event is emitted only as scoped, expiring session evidence.
- HistorySnapshotProjected event is emitted after snapshot materialization.
- ExportRenderProjected event is emitted after export artifact metadata is governed.
- TemplateInstallProjected event is emitted after template objects and DealSetBinding are linked.
- VendorImportProjected event is emitted after source_vendor and source_object_id are mapped.
- whiteboard-v1.proto must include projection_ref fields on board, session, operation, snapshot, export, template, and import messages.
- local-operations-v1.proto must include projection_version in merge, replay, and import worker commands.
- Proto projection refs must be opaque ids, not serialized graph records.
- REST status responses must include ontology_projection_ref when projection exists.
- Async events must include ontology_projection_ref when projection updates downstream state.
- Replay commands must accept projection_version and fail on unsupported versions.
- Projection deltas must be additive across schema versions.
- Projection deltas must preserve old refs for audit lookups.
- Projection delete is represented as superseded or inactive, not physical deletion.
- Projection repair events must include prior_projection_ref and new_projection_ref.
- SDK clients must treat projection refs as evidence links, not authorization tokens.

## Title-Specific Canvas, CRDT, And Session Facts
- CRDT materialized_view_version maps to CanvasObject.projection_version.
- CRDT conflict_count maps to projection quality evidence, not user-visible board content.
- CanvasOperation.operation_kind maps to object lifecycle relationships.
- CanvasOperation.previous_operation_id maps to causality edges.
- CanvasObject.geometry maps to canonical spatial facts.
- Connector.endpoint ids map to relationship edges that Lucidspark displacement requires.
- Frame membership maps to containment edges that Miro Enterprise displacement requires.
- StickyNote author maps to principal or external actor facts that FigJam displacement requires.
- BoardSession facilitator ids map to role edges that Mural Enterprise displacement requires.
- BoardSession roster binding maps to education pack edges that Whiteboard.fi displacement requires.
- BoardSession meeting binding maps to external meeting edges that Microsoft Whiteboard displacement requires.
- PresenceCursor freshness maps to expiring session facts, not durable board history.
- HistorySnapshot operation range maps to immutable reconstruction facts.
- ExportRender artifact metadata maps to retention and residency facts.
- TemplateInstall maps template source, DealSetBinding, and materialized object ids.

## Title-Specific Cedar, SLO, And Evidence Gates
- Projection reads require Cedar allow for the projected object family.
- Projection writes require prior command policy allow or replay authorization.
- Projection repair requires operator or system-worker policy allow.
- Projection access evidence includes policy_decision_ref, ontology_projection_ref, and audit_event_ref.
- Projection latency for board-open contributes to local-board-load-time when board metadata is read.
- Projection latency for operation materialization contributes to local-crdt-merge-success.
- Projection latency for presence contributes to local-presence-freshness only for fanout metadata.
- Projection latency for export contributes to local-export-render-latency.
- Projection rebuild freshness contributes to replay-freshness.
- Projection audit events contribute to audit-emission-lag.
- Evidence fields must include object_kind, relationship_kind, projection_version, source_vendor, source_object_id, and policy_pack_set.
- Evidence fields must include crdt_clock when projection follows CRDT merge.
- Evidence fields must include session_state when projection follows session open or close.
- Evidence fields must include retention_class and residency_zone for snapshot or export projection.
- Evidence fields must include marketplace_dealset_id for template projection.

## Rollback
- Roll back projection readers before projection writers.
- Keep source_vendor and source_object_id fields once any migration has run.
- Keep projection_version once records exist.
- Preserve projection error events for audit continuity.
- Disable benchmark import mapping per source only through feature flags or route gating.
- Do not delete projection records during rollback; mark them inactive or superseded.
- Route projection rollback issues to template-import-rollback, board-history-corruption, local-crdt-merge-conflict, or region-affinity-mismatch runbooks.
- Treat deletion of DealSetBinding from marketplace template projections as a commercial governance incident.
- Treat cross-tenant projection joins as a security incident.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
