# IP-020 Whiteboard Catalog Layer Registration

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-020-catalog-layer-registration.md
Benchmarks: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- Register whiteboard as a first-class catalog layer without creating a vendor-suite boundary.
- Bind catalog records to `businessCapability: canvas-collaboration`.
- Bind every record to the six capability YAML files under `microservices/whiteboard/capabilities/`.
- Preserve ADR-0321 by making product tier labels descriptive, not service-boundary drivers.
- Make catalog registration strong enough for B2B leader displacement of Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.

## Catalog Inputs
- PRD input: `microservices/whiteboard/PRD.md`.
- Manifest input: `microservices/whiteboard/manifest.json`.
- Architecture input: `microservices/whiteboard/ARCHITECTURE.md`.
- Capability input: `microservices/whiteboard/capabilities/board-open.yaml`.
- Capability input: `microservices/whiteboard/capabilities/canvas-op-append.yaml`.
- Capability input: `microservices/whiteboard/capabilities/presence-sync.yaml`.
- Capability input: `microservices/whiteboard/capabilities/history-snapshot.yaml`.
- Capability input: `microservices/whiteboard/capabilities/export-render.yaml`.
- Capability input: `microservices/whiteboard/capabilities/template-marketplace-install.yaml`.
- Catalog directory: `microservices/whiteboard/catalog/`.
- Compliance input: `microservices/whiteboard/compliance.md`.
- DPIA input: `microservices/whiteboard/dpia.md`.
- Threat input: `microservices/whiteboard/threat-model.md`.
- Audit input: `microservices/whiteboard/AUDIT-FINDINGS-2026-05-21.json`.

## Layer Model
- The catalog layer records service identity.
- The catalog layer records capability identity.
- The catalog layer records tenant-scope requirements.
- The catalog layer records pack overlay applicability.
- The catalog layer records data-class bindings.
- The catalog layer records policy-evaluation mode.
- The catalog layer records ontology-read mode.
- The catalog layer records marketplace settlement obligations.
- The catalog layer records benchmark displacement coverage.
- The catalog layer records operational owner.
- The catalog layer records promotion stage.
- The catalog layer records rollback evidence location.
- The catalog layer records audit export obligations.
- The catalog layer records SLO gate references.
- The catalog layer records threat-control references.

## Service Registration
- Service id remains `whiteboard`.
- Service category remains `canvas-collaboration`.
- Service does not register as `miro-replacement`.
- Service does not register as `mural-replacement`.
- Service does not register as `figjam-replacement`.
- Service does not register as `lucidspark-replacement`.
- Service does not register as `whiteboard-fi-replacement`.
- Service does not register as `microsoft-whiteboard-replacement`.
- Service owner is whiteboard axis plus council-product as stated by the PRD.
- Service status follows reserved Wave-3 anchor language until promotion gates pass.
- Service cell partitioning must appear in catalog metadata before load promotion.
- Service tenant isolation must appear before any user-facing capability activation.
- Service export semantics must appear separately from board mutation semantics.
- Service history semantics must appear separately from presence semantics.
- Service template marketplace semantics must appear separately from canvas editing semantics.

## Capability Registration
- Register `board-open` with `data_class=board_object`.
- Register `board-open` with required fields `tenant_id`, `principal_id`, `audience_type`, `purpose`, and `data_class`.
- Register `board-open` with Miro Enterprise and Mural Enterprise parity notes.
- Register `board-open` with Whiteboard.fi classroom-session notes where board session access matters.
- Register `canvas-op-append` with `data_class=canvas_operation`.
- Register `canvas-op-append` with append sequence and idempotency requirements.
- Register `canvas-op-append` with Miro Enterprise, Mural Enterprise, and FigJam parity notes.
- Register `canvas-op-append` with Microsoft Whiteboard conflict behavior as a displacement risk.
- Register `presence-sync` with `data_class=presence_cursor`.
- Register `presence-sync` with lease expiry and volatile state requirements.
- Register `presence-sync` with FigJam and Microsoft Whiteboard parity notes.
- Register `presence-sync` with Whiteboard.fi instructor/student audience notes.
- Register `history-snapshot` with `data_class=export_snapshot`.
- Register `history-snapshot` with retention and audit-chain requirements.
- Register `history-snapshot` with Lucidspark and Miro Enterprise parity notes.
- Register `history-snapshot` with Microsoft Whiteboard retention-export notes.
- Register `export-render` with `data_class=board_object`.
- Register `export-render` with async job and artifact authorization requirements.
- Register `export-render` with Mural Enterprise, Lucidspark, and Microsoft Whiteboard parity notes.
- Register `template-marketplace-install` with `data_class=canvas_operation`.
- Register `template-marketplace-install` with DealSet settlement under ADR-0314.
- Register `template-marketplace-install` with Miro Enterprise and Mural Enterprise template-library parity.
- Register `template-marketplace-install` with FigJam starter-template migration notes.

## ADR Binding
- ADR-0105 anchors layer naming and service boundaries.
- ADR-0131 anchors tenant-scoped product evidence.
- ADR-0242 anchors implementation plan traceability.
- ADR-0243 anchors capability record governance.
- ADR-0244 anchors policy and authorization posture.
- ADR-0246 anchors data and operational evidence.
- ADR-0253-amendment anchors HTTP/3 h3-alt-svc plus ECH/PQC posture.
- ADR-0257 anchors principal and audience handling.
- ADR-0258 anchors audit-chain handling.
- ADR-0263 anchors pack overlay handling.
- ADR-0294 anchors catalog promotion discipline.
- ADR-0296 anchors operational proof discipline.
- ADR-0297 anchors migration and rollback discipline.
- ADR-0314 anchors marketplace DealSet settlement.
- ADR-0321 anchors whiteboard-specific B2B leader scope.

## Benchmark Registration
- Miro Enterprise appears as a board, canvas operation, template, export, and history benchmark.
- Miro Enterprise is not copied as an API namespace.
- Miro Enterprise parity is measured through tenant-scoped board collaboration.
- Mural Enterprise appears as a facilitation-template, export, and workspace-control benchmark.
- Mural Enterprise is not copied as a workspace service boundary.
- Mural Enterprise parity is measured through pack-aware collaboration and templates.
- FigJam appears as a multiplayer presence and canvas append benchmark.
- FigJam is not copied as a design-file service boundary.
- FigJam parity is measured through reconnect-safe cursor and operation semantics.
- Lucidspark appears as a diagram-grade export and snapshot benchmark.
- Lucidspark is not copied as a diagram-storage service boundary.
- Lucidspark parity is measured through export fidelity and history evidence.
- Whiteboard.fi appears as a classroom-board and audience-control benchmark.
- Whiteboard.fi is not copied as an education-only product split.
- Whiteboard.fi parity is measured through scoped participants and instructor moderation evidence.
- Microsoft Whiteboard appears as a tenant-admin, retention, export, and collaboration benchmark.
- Microsoft Whiteboard is not copied as an Office file-storage dependency.
- Microsoft Whiteboard parity is measured through tenant governance and retention-safe export.

## Registration Fields
- Catalog row must include `service=whiteboard`.
- Catalog row must include `businessCapability=canvas-collaboration`.
- Catalog row must include `tier=product`.
- Catalog row must include `capability`.
- Catalog row must include `bindingAdrs`.
- Catalog row must include `benchmarks`.
- Catalog row must include `tenantScope.requiredFields`.
- Catalog row must include `marketplaceSettlement` where applicable.
- Catalog row must include `policyEvaluationMode`.
- Catalog row must include `ontologyReadMode`.
- Catalog row must include `dataClass`.
- Catalog row must include `packOverlays`.
- Catalog row must include `sloRef`.
- Catalog row must include `runbookRef`.
- Catalog row must include `dashboardRef`.
- Catalog row must include `threatModelRef`.
- Catalog row must include `dpiaRef`.
- Catalog row must include `auditFindingRef`.
- Catalog row must include `rollbackRef`.
- Catalog row must include `ownerTeam`.

## Catalog Domain Objects
- Register `Board` as the tenant-scoped collaboration container.
- Register `BoardSession` as the active participant window for open boards.
- Register `CanvasOperation` as the append-only mutation unit.
- Register `PresenceLease` as volatile participant state.
- Register `HistorySnapshot` as immutable recovery and evidence state.
- Register `ExportRenderJob` as async artifact production state.
- Register `ExportArtifact` as separately authorized output state.
- Register `TemplateGrant` as a tenant and principal scoped template activation.
- Register `DealSetSettlementRef` as marketplace settlement evidence.
- Register `MigrationReplay` as preview-to-accept vendor displacement state.
- Register `PolicyDecisionRef` as Cedar evidence attached to material transitions.
- Register `AuditChainRef` as durable evidence attached to accepted mutations and refusals.

## Command And Event Catalog Deltas
- Catalog `board-open` as an OpenAPI command plus audit event source.
- Catalog `canvas-op-append` as an OpenAPI command, AsyncAPI event source, and optional internal proto call.
- Catalog `presence-sync` as an AsyncAPI channel family with lease commands.
- Catalog `history-snapshot` as an OpenAPI async job command and snapshot-completed event source.
- Catalog `export-render` as an OpenAPI async job command and export-completed event source.
- Catalog `template-marketplace-install` as an OpenAPI command and settlement-completed event source.
- Catalog append rejected events separately from append accepted events.
- Catalog presence expired events separately from participant-left user intent.
- Catalog export artifact download as a command separate from export render.
- Catalog snapshot comparison as a read command separate from snapshot creation.
- Catalog template preview as non-mutating command separate from template install.
- Catalog migration replay preview as non-mutating command separate from accepted replay.

## Cedar Fact Catalog
- Fact catalog includes `tenant_id`.
- Fact catalog includes `principal_id`.
- Fact catalog includes `audience_type`.
- Fact catalog includes `purpose`.
- Fact catalog includes `data_class`.
- Fact catalog includes `capability`.
- Fact catalog includes `board_id`.
- Fact catalog includes `board_session_id`.
- Fact catalog includes `operation_id`.
- Fact catalog includes `presence_lease_id`.
- Fact catalog includes `snapshot_id`.
- Fact catalog includes `export_job_id`.
- Fact catalog includes `artifact_id`.
- Fact catalog includes `template_id`.
- Fact catalog includes `dealset_id`.
- Fact catalog includes `pack_overlay`.
- Fact catalog includes `source_benchmark`.
- Fact catalog includes `migration_replay_id`.

## Workflow Registration Decisions
- Board open registers as an interactive workflow with low latency SLOs.
- Canvas append registers as an interactive workflow with CRDT and idempotency evidence.
- Presence sync registers as a volatile session workflow with fail-soft behavior.
- History snapshot registers as an async evidence workflow with retention gates.
- Export render registers as an async portability workflow with artifact gates.
- Template install registers as a marketplace workflow with DealSet settlement.
- Migration import registers as a replay workflow with preview and acceptance states.
- Audit refusal registers as a compliance workflow separate from successful mutation.
- Promotion registers as a gated workflow dependent on IP-021 SLO evidence.
- Chaos readiness registers as a gated workflow dependent on IP-022 drill evidence.
- DPIA readiness registers as a gated workflow dependent on IP-023 evidence.
- Threat readiness registers as a gated workflow dependent on IP-024 controls.
- Audit closeout registers as a gated workflow dependent on IP-025 finding evidence.

## Admission Rules
- Admission fails if any capability record omits ADR-0321.
- Admission fails if any capability record omits ADR-0314 where template marketplace settlement is in scope.
- Admission fails if any benchmark list uses only shorthand displaced names.
- Admission fails if any record infers tenant from board id.
- Admission fails if any record omits principal.
- Admission fails if any record omits audience type.
- Admission fails if any record omits purpose.
- Admission fails if any record omits data class.
- Admission fails if any record mixes presence data with durable board history.
- Admission fails if any record treats export artifacts as board mutations.
- Admission fails if any record lets templates install without settlement metadata.
- Admission fails if any record creates a vendor-named folder.
- Admission fails if any record creates a suite boundary.
- Admission fails if any record lacks rollback evidence.
- Admission fails if any record lacks audit evidence.

## Cross-Document Trace
- PRD user stories map to catalog capabilities.
- PRD functional requirements map to command and event contract families.
- PRD non-functional requirements map to SLO and dashboard records.
- PRD compliance impact maps to DPIA and pack overlay records.
- PRD migration questions map to import and replay catalog notes.
- Capability records map to generated SDK methods in IP-019.
- Capability records map to SLO gates in IP-021.
- Capability records map to chaos drills in IP-022.
- Capability records map to DPIA evidence in IP-023.
- Capability records map to threat controls in IP-024.
- Capability records map to audit closeout in IP-025.
- Competitor matrix maps benchmarks to displacement obligations.
- SDK plan maps catalog operations to generated client surfaces.
- Operating bar maps catalog rows to phase promotion.
- Audit findings map catalog misses to closeout evidence.

## Review Checklist
- Reviewer confirms all six capabilities are present.
- Reviewer confirms all six displaced benchmark names are present.
- Reviewer confirms ADR-0321 is preserved.
- Reviewer confirms no ADR file was modified.
- Reviewer confirms no new service boundary was introduced.
- Reviewer confirms no vendor namespace was introduced.
- Reviewer confirms tenant scope is explicit.
- Reviewer confirms principal scope is explicit.
- Reviewer confirms audience type is explicit.
- Reviewer confirms data class is explicit.
- Reviewer confirms policy evaluation mode is explicit.
- Reviewer confirms ontology read mode is explicit.
- Reviewer confirms DealSet settlement is explicit for templates.
- Reviewer confirms export artifacts are separately governed.
- Reviewer confirms presence state is separately governed.
- Reviewer confirms history snapshots are separately governed.

## Rollback
- Roll back catalog activation before removing capability records.
- Roll back affected capability rows one at a time.
- Preserve prior catalog evidence for audit comparison.
- Revert benchmark display names only if downstream consumers cannot parse them.
- Revert promotion state if any SLO gate in IP-021 fails.
- Revert chaos-drill eligibility if any drill in IP-022 lacks evidence.
- Revert DPIA activation if IP-023 evidence is incomplete.
- Revert threat-control activation if IP-024 controls are incomplete.
- Revert audit closeout if IP-025 leaves open findings.
- Never roll back by deleting ADR-0321 references.

## Workflow Decisions
- Workflow decision: catalog admission starts in draft until every capability record has tenant, principal, audience, data class, and owner evidence.
- Workflow decision: benchmark displacement labels are display metadata and cannot become service namespaces.
- Workflow decision: promotion consumers read catalog state only after IP-021 SLO gates are linked.
- Workflow decision: chaos consumers read catalog state only after IP-022 drill evidence names the target capability.
- Workflow decision: DPIA and threat-model consumers read catalog state only after IP-023 and IP-024 references resolve.
- Workflow decision: audit closeout in IP-025 can close catalog findings only when the registered row includes rollback evidence.

## Acceptance Criteria
- Catalog registration names Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
- Catalog registration preserves the existing ADR binding set including ADR-0321.
- Catalog registration references every whiteboard capability record.
- Catalog registration defines fields required for tenant, principal, audience, purpose, and data class.
- Catalog registration distinguishes board, operation, presence, snapshot, export, and template concerns.
- Catalog registration binds template marketplace work to DealSet settlement.
- Catalog registration blocks vendor-named namespaces and suite boundaries.
- Catalog registration links to IP-019 through IP-025 evidence surfaces.
- Catalog registration defines concrete admission failures and rollback steps.
- Catalog registration can be reviewed without running `oya vcs verify`, `done`, or `promote`.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
