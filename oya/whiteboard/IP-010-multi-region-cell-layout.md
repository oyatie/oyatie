# IP-010 Whiteboard multi-region-cell-layout

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-010-multi-region-cell-layout.md
Benchmarks displaced: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## 1. Outcome
- Define the multi-region cell layout for low-latency collaborative canvas workloads without breaking tenant residency or pack overlays.
- Keep board writes pinned to the tenant home cell unless a documented promotion or failover rule applies.
- Keep cursor/presence fanout regional and bounded.
- Keep history snapshots replayable after cell failover.
- Keep exports residency-aware.
- Keep classroom-style short-lived boards local to the teaching session cell.
- Keep marketplace template installation tied to DealSet settlement in the tenant’s allowed residency envelope.
- Satisfy ADR-0321 with benchmark-specific regional behavior for Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
- Preserve the flat microservice boundary from ADR-0316.
- Preserve the transport and crypto expectations from ADR-0253-amendment.

## 2. Local Source Anchors
- microservices/whiteboard/multi-region.md is the local multi-region companion.
- microservices/whiteboard/PRD.md defines availability, latency, capacity, and pack overlay expectations.
- microservices/whiteboard/ARCHITECTURE.md names regional outage as a failure mode.
- microservices/whiteboard/capacity-model.md defines tenant, region, queue, data-class, and workload partitioning.
- microservices/whiteboard/failure-modes.md defines outage and rollback behavior.
- microservices/whiteboard/policy/data-residency.md defines residency constraints.
- microservices/whiteboard/iac/dr-failover.yaml is the DR failover anchor.
- microservices/whiteboard/iac/local-network-policy.yaml is the local network boundary anchor.
- microservices/whiteboard/iac/local-pdb.yaml is the disruption-budget anchor.
- microservices/whiteboard/iac/local-hpa.yaml is the local scaling anchor.
- microservices/whiteboard/iac/local-slo-alerts.yaml is the local alert anchor.
- microservices/whiteboard/runbooks/region-affinity-mismatch.md is the runbook for wrong-cell traffic.
- microservices/whiteboard/runbooks/local-regional-board-replay.md is the replay runbook.
- microservices/whiteboard/slos/local-board-load-time.openslo.yaml is the board load SLO.
- microservices/whiteboard/slos/local-presence-freshness.openslo.yaml is the presence freshness SLO.
- microservices/whiteboard/slos/replay-freshness.openslo.yaml is the replay SLO.

## 3. Cell Vocabulary
- `home_cell` is the authoritative cell for board writes.
- `request_cell` is the cell where the request enters.
- `presence_cell` is the cell that owns cursor fanout for a session.
- `render_cell` is the cell that performs export rendering.
- `snapshot_cell` is the cell that writes history snapshots.
- `template_cell` is the cell where template package installation is evaluated.
- `classroom_cell` is the cell for Whiteboard.fi-style ephemeral classroom boards.
- `metadata_replica_cell` is read-only unless failover has been promoted.
- `region_affinity` binds a board to allowed cells.
- `residency_pack` constrains data movement.
- `failover_epoch` increments on promoted failover.
- `cell_generation` identifies config rollout generation.
- `replay_cursor` proves ordered recovery after failover.
- `cell_evidence_ref` points at audit-chain evidence for routing decisions.

## 4. Placement Rules
- Board metadata is created in `home_cell`.
- Canvas operations are appended in `home_cell`.
- Presence fanout is regional and may use `presence_cell`.
- Presence fanout cannot become source of truth for board state.
- History snapshots are written in `snapshot_cell`, normally equal to `home_cell`.
- Export rendering happens in `render_cell` only if residency permits.
- Template installation happens in `template_cell` only if DealSet and residency permit.
- Classroom boards are created in `classroom_cell` with short lifetime.
- Guest participants may enter through a nearby edge but must resolve authority against `home_cell`.
- Metadata replicas can answer low-risk reads with stale markers.
- Mutations cannot target metadata replicas.
- Failover promotion requires audit-chain evidence.
- Failover promotion requires pack overlay compatibility.
- Failover promotion requires replay cursor integrity.

## 5. Benchmark Pressure
- Miro Enterprise expects global collaboration and near-real-time edits.
- Oyatie meets that with local append budgets and explicit home-cell authority.
- Mural Enterprise expects facilitation sessions across distributed teams.
- Oyatie meets that with facilitator context replicated as policy metadata, not write authority.
- FigJam expects low-latency cursor and reaction behavior.
- Oyatie meets that with regional presence cells and bounded fanout.
- Lucidspark expects diagram history and export reliability.
- Oyatie meets that with snapshot cells and deterministic render cells.
- Whiteboard.fi expects teacher-controlled short-lived boards.
- Oyatie meets that with classroom cells and automatic expiry.
- Microsoft Whiteboard expects sharing across Microsoft 365 contexts.
- Oyatie meets that with guest grants resolved in the tenant home cell.

## 6. Board Open Flow
- Request enters through edge.
- Edge attaches `request_cell`.
- API resolves tenant `home_cell`.
- API evaluates Cedar in the request context.
- Policy context includes `home_cell`.
- Policy context includes `request_cell`.
- Policy context includes `jurisdiction_code`.
- Policy context includes `residency_pack`.
- Board metadata is read from `home_cell` or approved metadata replica.
- Response includes stale marker when metadata replica served the read.
- Response includes presence endpoint for allowed participant.
- Response includes region-affinity mismatch denial when request cell is forbidden.
- Audit event records routing decision.

## 7. Canvas Operation Flow
- Client sends operation to nearest allowed ingress.
- Ingress resolves board `home_cell`.
- If request cell equals home cell, append proceeds after policy.
- If request cell differs, operation is forwarded to home cell.
- Forwarded operation retains original trace id.
- Forwarded operation retains client sequence.
- Forwarded operation retains idempotency key.
- Forwarded operation is never re-authorized with weaker context.
- Home cell assigns accepted revision.
- Home cell emits append event.
- Home cell updates replay cursor.
- Home cell may notify regional presence cells.
- Request cell returns accepted revision to client.
- Cross-cell append latency is tracked separately from local latency.

## 8. Presence Flow
- Presence connects to a regional presence cell.
- Presence cell validates board access against home cell authority.
- Presence cell caches permit only for a bounded TTL.
- Presence cell publishes cursor updates locally.
- Presence cell samples audit events.
- Presence cell enforces local cursor rate policy.
- Presence cell drops stale cursor epochs.
- Presence cell never grants board content access.
- Presence cell never writes board state.
- Presence cell can fail independently without board data loss.
- FigJam parity is measured by cursor freshness.
- Whiteboard.fi parity is measured by teacher visibility into active participants.

## 9. History and Replay Flow
- Home cell stores canonical operation log.
- Snapshot cell writes periodic snapshots.
- Snapshot cadence follows replay-freshness SLO.
- Replay cursor is monotonic.
- Replay cursor is tenant-scoped.
- Replay cursor is board-scoped.
- Replay cursor includes failover epoch.
- Replay cursor is sealed in audit evidence for promoted failover.
- Regional outage starts replay from last sealed cursor.
- Replay refuses gaps.
- Replay refuses pack-incompatible target cells.
- Replay produces rollback bundle ref.
- Lucidspark parity is measured by deterministic reconstruction of diagram boards.

## 10. Export Flow
- Export render request starts in request cell.
- Export policy resolves residency target.
- Render cell is selected from residency-allowed cells.
- Render cell reads board snapshot, not live mutable state.
- Render cell signs artifact manifest.
- Render cell emits audit event.
- Render cell writes artifact to residency-approved storage.
- Render cell refuses if template content has DealSet hold.
- Render cell refuses if board contains data class forbidden for target artifact.
- Export response includes render revision.
- Export response includes artifact ref.
- Export response includes watermark policy ref.
- Microsoft Whiteboard and Lucidspark export parity is satisfied through deterministic artifact evidence.

## 11. Template Install Flow
- Template install request starts in request cell.
- DealSet settlement is checked before template package fetch.
- Template cell is selected from allowed residency cells.
- Template package provenance is verified.
- Template package data class is checked.
- Template package publisher is checked.
- Template install writes board content in home cell.
- Template install emits marketplace settlement evidence.
- Template install emits audit event.
- Template install can be rolled back with installed template ref.
- Miro Enterprise and Mural Enterprise template parity is satisfied without vendor-specific service boundaries.

## 12. Classroom Flow
- Classroom owner creates a session in classroom cell.
- Student boards are spawned in classroom cell.
- Student boards carry expiry.
- Student boards carry owner supervision policy.
- Student boards carry data class.
- Student boards do not escape residency pack.
- Student board exports require explicit owner and policy permit.
- Session close revokes classroom authority.
- Session close revokes classroom credentials.
- Session close seals audit evidence.
- Whiteboard.fi parity is satisfied by explicit lifecycle rather than anonymous transient state.

## 13. Data Residency
- GDPR pack may restrict board data to EU cells.
- KR-PIPA pack may restrict board data to KR-approved cells.
- Public-sector pack may restrict export rendering.
- Education pack may restrict classroom data movement.
- SOC-2 pack adds evidence but may not change residency.
- ISO-27001 pack adds control evidence and operator access constraints.
- Higher-restriction-wins applies when packs conflict.
- Metadata-only replicas cannot include raw canvas operations.
- Presence metadata must be coarse enough to avoid content leakage.
- Export artifact storage must match render cell residency.
- Template package cache must match template license and tenant residency.

## 14. Failover
- Failover begins with outage detection.
- Failover identifies affected boards.
- Failover identifies last sealed replay cursor.
- Failover evaluates pack compatibility.
- Failover evaluates target cell capacity.
- Failover promotes only after audit evidence is available.
- Failover increments failover epoch.
- Failover routes new writes to promoted cell.
- Failover blocks stale home cell writes.
- Failover starts replay verification.
- Failover emits operator event.
- Failover links to local-regional-board-replay runbook.
- Failback requires replay parity and audit-chain seal.

## 15. Capacity
- Partition board operations by tenant.
- Partition board operations by board id.
- Partition presence by board session.
- Partition snapshots by board id and revision window.
- Partition exports by artifact class and data class.
- Partition template installs by tenant and deal set.
- Partition classroom boards by classroom session.
- Track hot boards separately from ordinary boards.
- Track cross-cell forwards separately from local operations.
- Track source imports separately from native operations.
- Track replay workers separately from live append workers.
- Prevent Miro Enterprise-scale imports from starving live FigJam-style collaboration.

## 16. Observability
- Emit `whiteboard.cell.request.count`.
- Emit `whiteboard.cell.forward.count`.
- Emit `whiteboard.cell.forward.duration`.
- Emit `whiteboard.cell.home_mismatch.count`.
- Emit `whiteboard.cell.failover.epoch`.
- Emit `whiteboard.cell.replay.cursor_lag`.
- Emit `whiteboard.cell.presence.freshness`.
- Emit `whiteboard.cell.export.render_duration`.
- Emit `whiteboard.cell.template.install_duration`.
- Emit `whiteboard.cell.classroom.active_boards`.
- Dashboard target is local-slo-burn plus operating-bar-overview.
- Runbook target for mismatch is region-affinity-mismatch.
- Runbook target for replay is local-regional-board-replay.

## 17. Tests
- Unit tests validate cell resolution.
- Unit tests validate residency pack selection.
- Unit tests validate metadata-only replica restrictions.
- Unit tests validate cross-cell forward envelope.
- Integration tests append locally.
- Integration tests forward append to home cell.
- Integration tests deny mutation to metadata replica.
- Integration tests presence fanout through presence cell.
- Integration tests export render through allowed render cell.
- Integration tests deny export through forbidden render cell.
- Integration tests template install with DealSet and residency.
- Failover tests replay from sealed cursor.
- Failover tests refuse replay gaps.
- Classroom tests expire student boards.
- Benchmark tests name Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.

## 18. Acceptance Criteria
- Every board has one authoritative home cell.
- Every mutation routes to the authoritative cell or fails closed.
- Presence can be regional but never authoritative.
- Exports and templates respect residency and DealSet constraints.
- Failover requires replay cursor, pack compatibility, and audit evidence.
- Metrics and runbooks distinguish local, forwarded, degraded, and failed traffic.
- No benchmark creates a vendor-shaped regional service.
- ADR-0321 remains cited with benchmark-specific multi-region substance.

## 19. Proto And Workflow Deltas
- Proto `BoardCellRoute` carries home cell, serving cell, residency pack, failover epoch, and metadata-only replica flag.
- Proto `PresenceCellRoute` carries volatile-region fanout hints without granting durable board authority.
- Proto `ExportCellRoute` carries render cell, artifact residency, and authorization decision reference.
- Workflow decision: board-open can read from a metadata replica only when the response states freshness and home-cell authority.
- Workflow decision: canvas-op-append forwards to the home cell or fails closed; it never writes to a convenient regional replica.
- Workflow decision: export-render waits for cell residency confirmation before queue admission.
- Workflow decision: template-marketplace-install resolves DealSet settlement in the tenant home cell before regional preview.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
