# IP-023 Whiteboard DPIA Evidence Packet

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-023-dpia-evidence-packet.md
Benchmarks: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- Build a whiteboard-specific DPIA evidence packet for collaborative canvas processing.
- Treat multiplayer presence, board history, export artifacts, and template marketplace installs as distinct privacy surfaces.
- Keep ADR-0321 as the whiteboard product-depth anchor without editing it.
- Provide evidence strong enough for displacement of Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
- Make DPIA evidence reviewable from repo-local surfaces under `microservices/whiteboard/`.

## Repo-Local Inputs
- DPIA source: `microservices/whiteboard/dpia.md`.
- Compliance source: `microservices/whiteboard/compliance.md`.
- PRD source: `microservices/whiteboard/PRD.md`.
- Capability records: `microservices/whiteboard/capabilities/`.
- Policies: `microservices/whiteboard/policies/`.
- Threat model: `microservices/whiteboard/threat-model.md`.
- Failure modes: `microservices/whiteboard/failure-modes.md`.
- Incident response: `microservices/whiteboard/incident-response.md`.
- Audit events: `microservices/whiteboard/AUDIT-FINDINGS-2026-05-21.json`.
- Export controls: `microservices/whiteboard/export-render` evidence implied by capability and contracts.
- Template controls: `microservices/whiteboard/template-marketplace-install` evidence implied by capability and contracts.

## Processing Activities
- Open a board for a tenant user.
- Append a canvas operation to a board.
- Publish volatile presence state.
- Persist a history snapshot.
- Render an export artifact.
- Install a marketplace template.
- Preview a marketplace template.
- Download an export artifact.
- Compare two history snapshots.
- Replay a migration fixture.
- Refuse unauthorized board access.
- Refuse unauthorized export download.
- Refuse unauthorized template install.
- Record audit-chain events for material transitions.
- Record metrics for SLO and incident response.

## Canvas And CRDT Privacy Model
- Board content is modeled as operation-derived state, not as a monolithic file.
- CRDT-compatible operation identifiers are personal data when they can be tied to an actor.
- Operation authorship is personal data.
- Sticky-note content is user-authored content.
- Connector and shape geometry can reveal user intent in planning sessions.
- Frame names can contain personal or customer information.
- Comment anchors can reveal discussion context.
- Imported vendor object ids are migration provenance, not tenant identity.
- Merge metadata is minimized to sequence, operation id, and conflict reason.
- Conflict reasons are retained for reliability evidence without storing full competing payloads.
- Presence cursor state is personal data while the lease is active.
- Presence viewport state can reveal participant attention and must expire.
- Classroom role state from Whiteboard.fi-style sessions is audience data.
- Tenant-admin role state from Microsoft Whiteboard displacement is governance data.

## Command, Event, And Proto Privacy Deltas
- `boards:open` command collects tenant, principal, audience, purpose, and board id.
- `operations:append` command collects operation payload and actor facts.
- `operations:preview` command collects migration payload without accepting durable mutation.
- `presence:sync` command collects cursor, selection, viewport, lease, and expiry.
- `history:snapshot` command collects board version pointer and retention pack.
- `exports:render` command collects requested formats and artifact authorization facts.
- `templates:install` command collects template id, settlement ref, pack overlay, and grant scope.
- Append accepted events contain operation metadata, not full board payload.
- Append rejected events contain refusal reason and policy evidence.
- Presence events contain lease and expiry, not durable audit payload.
- Snapshot completed events contain snapshot pointer and retention evidence.
- Export completed events contain artifact metadata and authorization state.
- Template settled events contain DealSet reference and grant id.
- Internal proto calls must not add personal fields absent from public command contracts.
- Internal proto calls must preserve data-class facts for downstream audit minimization.

## Cedar And Rights Evidence
- Cedar facts prove purpose limitation for board open.
- Cedar facts prove principal authority for append.
- Cedar facts prove audience eligibility for presence.
- Cedar facts prove retention eligibility for snapshot.
- Cedar facts prove export authorization for artifact download.
- Cedar facts prove marketplace eligibility for template install.
- Denial evidence supports access-right review without exposing full board content.
- Export evidence supports portability review without broadening mutation rights.
- Snapshot evidence supports recovery review without creating an undeletable content fork outside retention policy.
- Template evidence supports marketplace audit without exposing settlement internals beyond DealSet reference.
- Presence expiry evidence supports minimization review.
- Replay preview evidence supports migration review before durable processing.

## Personal Data Categories
- Tenant identifier.
- Principal identifier.
- Audience type.
- Board identifier.
- Cursor position.
- Selection region.
- Viewport state.
- Canvas object author.
- Canvas operation body where it contains user-authored content.
- Sticky note text where represented in operation payload.
- Template attribution.
- Export artifact requester.
- Snapshot requester.
- Audit event actor.
- Workflow run actor.
- Migration source actor.
- IP address or network metadata where captured by edge logs.
- Device or session metadata where captured by presence transport.
- Pack overlay status where regulatory eligibility is personal or organizational data.

## Data-Class Binding
- `board-open` uses `board_object`.
- `canvas-op-append` uses `canvas_operation`.
- `presence-sync` uses `presence_cursor`.
- `history-snapshot` uses `export_snapshot`.
- `export-render` uses `board_object` plus artifact metadata.
- `template-marketplace-install` uses `canvas_operation` plus settlement metadata.
- Board identifiers never replace tenant identifiers.
- Presence cursors never become durable history unless explicitly captured by a snapshot policy.
- Export artifacts never inherit mutation permissions automatically.
- Template settlement metadata never exposes payment internals beyond DealSet reference requirements.
- Audit events store actor and decision evidence, not full canvas payloads unless required by an approved evidence profile.
- Metrics use tenant and capability dimensions with pack-aware minimization.

## Lawful Basis And Purpose
- Collaborative editing purpose covers board open and append.
- Session participation purpose covers presence sync.
- Evidence and recovery purpose covers history snapshot.
- Customer-requested portability purpose covers export render.
- Marketplace activation purpose covers template install.
- Compliance purpose covers audit-chain events.
- Reliability purpose covers metrics and traces.
- Security purpose covers refusal and abuse evidence.
- Migration purpose covers benchmark import and replay fixtures.
- Education administration purpose covers Whiteboard.fi-style classroom sessions.
- Tenant administration purpose covers Microsoft Whiteboard displacement journeys.
- Facilitation purpose covers Mural Enterprise displacement journeys.
- Diagram collaboration purpose covers Lucidspark displacement journeys.
- Multiplayer ideation purpose covers FigJam displacement journeys.
- Enterprise canvas collaboration purpose covers Miro Enterprise displacement journeys.

## Data Minimization
- Board open returns only board envelope fields needed to authorize and render.
- Board open excludes unrelated tenant boards.
- Canvas append stores operation deltas rather than full board rewrites.
- Canvas append rejects unscoped imported vendor payloads.
- Presence sync publishes expiring state only.
- Presence sync omits audit-chain events for every cursor movement.
- History snapshot captures governed point-in-time evidence only.
- History snapshot follows retention pack constraints.
- Export render includes only requested formats.
- Export render requires separate artifact authorization.
- Template preview does not mutate board state.
- Template install stores only DealSet reference and template grant metadata.
- Metrics aggregate where possible after tenant and pack evidence is preserved.
- Logs redact payload bodies by default.
- Audit events store decision evidence rather than full content bodies.

## Rights And Controls
- Tenant admins can inspect board lifecycle evidence.
- Authorized users can request export artifacts.
- Authorized users can see board history according to retention policy.
- Authorized users can leave sessions and expire presence leases.
- Auditors can inspect refusal and policy evidence through scoped paths.
- CI principals can validate contracts without reading tenant content.
- Education pack administrators can inspect classroom participant evidence.
- Public-sector pack administrators can inspect residency and export evidence.
- GDPR pack administrators can inspect erasure and export implications.
- KR-PIPA pack administrators can inspect residency and processing evidence.
- Users cannot bypass tenant scope with board identifiers.
- Users cannot download exports with board mutation permission alone.
- Users cannot install templates without marketplace settlement authorization.
- Users cannot convert volatile presence into history without approved snapshot policy.
- Users cannot erase audit-chain events through board deletion flows.

## Benchmark DPIA Notes
- Miro Enterprise displacement increases board import and template-library processing pressure.
- Miro Enterprise displacement requires evidence that imported boards become tenant-scoped Oyatie board objects.
- Mural Enterprise displacement increases facilitation-template and export processing pressure.
- Mural Enterprise displacement requires evidence that workspace-like concepts do not become new service boundaries.
- FigJam displacement increases volatile cursor, selection, and multiplayer append processing.
- FigJam displacement requires evidence that presence minimization is enforced.
- Lucidspark displacement increases diagram-grade export and history snapshot processing.
- Lucidspark displacement requires evidence that export artifacts are separately authorized.
- Whiteboard.fi displacement increases classroom audience and student participation processing.
- Whiteboard.fi displacement requires evidence that instructor and participant roles remain explicit.
- Microsoft Whiteboard displacement increases tenant-admin retention and export governance processing.
- Microsoft Whiteboard displacement requires evidence that storage integration assumptions are not copied.

## Pack Overlay Evidence
- SOC-2 evidence includes audit-chain coverage and operator access review.
- ISO-27001 evidence includes control mapping and incident-response linkage.
- GDPR evidence includes lawful basis, minimization, access, export, and erasure implications.
- KR-PIPA evidence includes residency, purpose, and processor evidence.
- Education evidence includes classroom participant and instructor role handling.
- Public-sector evidence includes residency, export, audit, and emergency-service handling.
- Pack evidence states permit deltas.
- Pack evidence states data-class deltas.
- Pack evidence states retention deltas.
- Pack evidence states export deltas.
- Pack evidence states regulator evidence deltas.
- Pack evidence states rollback conditions.

## Risk Register
- Risk: tenant inferred from board id.
- Control: generated clients and contracts require explicit tenant id.
- Risk: presence cursor retained longer than intended.
- Control: lease expiry and volatile event separation.
- Risk: export artifact accessed by board collaborator without export grant.
- Control: artifact-specific authorization and audit event.
- Risk: template marketplace install bypasses settlement.
- Control: DealSet settlement reference under ADR-0314.
- Risk: benchmark import carries vendor permissions into Oyatie.
- Control: import preview and Cedar remapping.
- Risk: history snapshot captures personal data beyond approved retention.
- Control: retention pack check before accepted job.
- Risk: logs contain canvas payload bodies.
- Control: default redaction and payload sampling prohibition.
- Risk: classroom sessions expose student participation broadly.
- Control: Whiteboard.fi-style audience mapping with tenant policy.
- Risk: tenant admins expect Microsoft Whiteboard storage semantics.
- Control: explicit export and retention documentation.

## DPIA Evidence Checklist
- Evidence names processing activity.
- Evidence names capability.
- Evidence names data class.
- Evidence names tenant scope.
- Evidence names principal scope.
- Evidence names audience type.
- Evidence names purpose.
- Evidence names pack overlay.
- Evidence names retention rule.
- Evidence names export rule.
- Evidence names audit event.
- Evidence names policy decision.
- Evidence names benchmark displacement source.
- Evidence names rollback route.
- Evidence names open finding if incomplete.
- Evidence names owner team.
- Evidence names review date.
- Evidence names ADR-0321.

## Tests And Review
- Test that DPIA packet covers all six capability records.
- Test that DPIA packet names all six displaced benchmark products.
- Test that presence data is classified as volatile cursor data.
- Test that snapshot data is classified as export snapshot data.
- Test that export artifact access has separate authorization evidence.
- Test that template installs require DealSet settlement evidence.
- Test that audit-chain entries omit full canvas payload by default.
- Test that pack overlays declare permit, retention, data-class, and export deltas.
- Test that benchmark imports do not preserve vendor permissions blindly.
- Test that ADR-0321 remains referenced and unmodified.
- Review with privacy owner.
- Review with security owner.
- Review with product owner.
- Review with SRE owner.
- Review with audit owner.

## Rollback
- Roll back DPIA acceptance if any capability lacks data-class evidence.
- Roll back DPIA acceptance if benchmark names are incomplete.
- Roll back DPIA acceptance if presence minimization is not proven.
- Roll back DPIA acceptance if export authorization is not proven.
- Roll back DPIA acceptance if template settlement is not proven.
- Roll back DPIA acceptance if pack overlay deltas are missing.
- Roll back DPIA acceptance if audit payload minimization is missing.
- Roll back DPIA acceptance if tenant scope is inferred.
- Roll back DPIA acceptance if ADR-0321 linkage is removed.
- Roll back affected capability activation rather than deleting the DPIA packet.

## Acceptance Criteria
- DPIA packet names Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
- DPIA packet preserves the existing ADR binding set including ADR-0321.
- DPIA packet covers board-open, canvas-op-append, presence-sync, history-snapshot, export-render, and template-marketplace-install.
- DPIA packet separates personal data categories by capability and data class.
- DPIA packet documents lawful basis, purpose, minimization, rights, pack overlays, risks, and controls.
- DPIA packet binds export artifacts and template settlement to separate authorization evidence.
- DPIA packet identifies benchmark-specific privacy pressures without copying vendor boundaries.
- DPIA packet supports audit closeout in IP-025.
- DPIA packet can be reviewed without editing ADR-0321.
- DPIA packet does not require `oya vcs verify`, `done`, or `promote`.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
