# IP-022 Whiteboard Chaos Drill Pack

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-022-chaos-drill-pack.md
Benchmarks: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- Define whiteboard-specific chaos drills before preview promotion.
- Exercise collaborative canvas failure modes that generic document or chat drills do not cover.
- Prove recovery for board access, append sequencing, presence fanout, history snapshots, export rendering, and template settlement.
- Preserve ADR-0321 and the existing ADR set while deepening operational evidence.
- Use Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard as displacement pressure tests.

## Repo-Local Anchors
- Failure modes: `microservices/whiteboard/failure-modes.md`.
- Incident response: `microservices/whiteboard/incident-response.md`.
- Runbooks: `microservices/whiteboard/runbooks/`.
- SLOs: `microservices/whiteboard/slos/`.
- Dashboards: `microservices/whiteboard/dashboards/`.
- Capacity model: `microservices/whiteboard/capacity-model.md`.
- Cost budget: `microservices/whiteboard/cost-budget.md`.
- Backfill replay: `microservices/whiteboard/backfill-replay.md`.
- Capabilities: `microservices/whiteboard/capabilities/`.
- Audit findings: `microservices/whiteboard/AUDIT-FINDINGS-2026-05-21.json`.

## Drill Entry Criteria
- Drill has a named capability owner.
- Drill has tenant and cell scope.
- Drill has declared data class.
- Drill has declared audience type.
- Drill has declared benchmark pressure source.
- Drill has a rollback plan.
- Drill has a dashboard panel.
- Drill has alert expectation.
- Drill has audit-chain expectation.
- Drill has SLO impact expectation.
- Drill has operator runbook link.
- Drill has stop condition.
- Drill has no production-destructive default.
- Drill has pack overlay applicability.
- Drill has ADR-0321 trace.

## Whiteboard Failure Domain Model
- Failure domain `board-envelope` covers metadata reads, access policy, and board session admission.
- Failure domain `operation-log` covers CRDT-compatible append, idempotency, conflict, and replay ordering.
- Failure domain `presence-mesh` covers cursor, selection, viewport, lease renewal, and volatile fanout.
- Failure domain `snapshot-worker` covers history capture, retention check, comparison, and replay pointer integrity.
- Failure domain `render-worker` covers export queue, render formats, artifact hash, and download authorization.
- Failure domain `template-settlement` covers preview, DealSet reference, grant activation, and rollback token.
- Failure domain `audit-chain` covers accepted mutation evidence and refusal evidence.
- Failure domain `policy-eval` covers Cedar allow, deny, error, timeout, and policy snapshot mismatch.
- Failure domain `migration-replay` covers Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard fixtures.

## Command, Event, And Proto Fault Injection
- Inject command fault into `boards:open` by removing tenant fact.
- Inject command fault into `operations:append` by replaying stale sequence.
- Inject command fault into `operations:preview` by passing unmapped vendor permission.
- Inject command fault into `history:snapshot` by forcing retention-pack mismatch.
- Inject command fault into `exports:render` by forcing renderer queue saturation.
- Inject command fault into `templates:install` by withholding DealSet settlement id.
- Inject event fault into append accepted event publication.
- Inject event fault into append rejected event publication.
- Inject event fault into presence lease renewal event publication.
- Inject event fault into snapshot completed event publication.
- Inject event fault into export completed event publication.
- Inject event fault into template settled event publication.
- Inject proto fault into internal append worker after edge allow.
- Inject proto fault into internal render worker after job acceptance.
- Inject proto fault into internal presence fanout after lease renewal.

## Cedar Facts Under Drill
- Drill payload must declare `tenant_id`.
- Drill payload must declare `principal_id`.
- Drill payload must declare `audience_type`.
- Drill payload must declare `purpose`.
- Drill payload must declare `capability`.
- Drill payload must declare `data_class`.
- Drill payload must declare `board_id` when board state is touched.
- Drill payload must declare `operation_id` when append state is touched.
- Drill payload must declare `presence_lease_id` when presence state is touched.
- Drill payload must declare `snapshot_id` when history state is touched.
- Drill payload must declare `artifact_id` when export download is touched.
- Drill payload must declare `dealset_id` when marketplace install is touched.
- Drill evidence must record Cedar allow, deny, timeout, or error.
- Drill evidence must record policy snapshot id.
- Drill evidence must record refusal reason without storing full canvas payload.

## Board-Open Drills
- Drill BO-01: deny board open when tenant id is missing.
- Drill BO-02: deny board open when principal is unknown.
- Drill BO-03: deny board open when audience type is incompatible.
- Drill BO-04: return typed Cedar refusal during policy engine denial.
- Drill BO-05: hold board-open promotion when policy engine errors.
- Drill BO-06: degrade read-only board envelope during export subsystem failure.
- Drill BO-07: isolate a hot board to its tenant and cell.
- Drill BO-08: verify Miro Enterprise-sized board envelope latency.
- Drill BO-09: verify Mural Enterprise facilitation board access controls.
- Drill BO-10: verify Whiteboard.fi instructor and participant board access split.
- Drill BO-11: verify Microsoft Whiteboard tenant-admin read posture.
- Drill BO-12: verify audit-chain event for access refusal.
- Drill BO-13: verify no board id infers tenant.
- Drill BO-14: verify rollback removes preview tenant eligibility.

## Canvas-Append Drills
- Drill CA-01: inject duplicate idempotency key.
- Drill CA-02: inject stale operation sequence.
- Drill CA-03: inject out-of-order append batch.
- Drill CA-04: inject transient network timeout after accepted append.
- Drill CA-05: verify retry reuses idempotency key.
- Drill CA-06: verify non-idempotent append does not auto-retry.
- Drill CA-07: verify accepted operation loss pages.
- Drill CA-08: verify conflict is not counted as availability failure.
- Drill CA-09: verify FigJam-style rapid multiplayer append pressure.
- Drill CA-10: verify Miro Enterprise board scale append pressure.
- Drill CA-11: verify Mural Enterprise facilitation burst pressure.
- Drill CA-12: verify Microsoft Whiteboard conflict migration fixture.
- Drill CA-13: verify operation audit event includes tenant and principal.
- Drill CA-14: verify rollback freezes writes without deleting board history.

## Presence-Sync Drills
- Drill PS-01: expire participant lease during active session.
- Drill PS-02: drop stale cursor updates.
- Drill PS-03: interrupt websocket or event-stream transport.
- Drill PS-04: reconnect participant after cell-local disruption.
- Drill PS-05: fan out cursor state to classroom-sized Whiteboard.fi session.
- Drill PS-06: fan out cursor state to FigJam-sized multiplayer session.
- Drill PS-07: throttle noisy presence publisher.
- Drill PS-08: reject presence publish without tenant scope.
- Drill PS-09: reject presence publish without principal scope.
- Drill PS-10: verify volatile drop is not durable data loss.
- Drill PS-11: verify presence metrics by tenant, board, and audience type.
- Drill PS-12: verify audit only captures policy transitions, not every cursor.
- Drill PS-13: verify emergency disable removes presence only.
- Drill PS-14: verify board and append paths remain available during presence outage.

## History-Snapshot Drills
- Drill HS-01: saturate snapshot queue.
- Drill HS-02: fail snapshot worker after job acceptance.
- Drill HS-03: deny snapshot for unauthorized principal.
- Drill HS-04: test retention pack conflict.
- Drill HS-05: test snapshot comparison timeout.
- Drill HS-06: test rollback to previous snapshot pointer.
- Drill HS-07: verify Lucidspark-grade diagram snapshot fixture.
- Drill HS-08: verify Miro Enterprise board-history fixture.
- Drill HS-09: verify Microsoft Whiteboard retention export fixture.
- Drill HS-10: verify audit-chain event for snapshot creation.
- Drill HS-11: verify snapshot data class remains `export_snapshot`.
- Drill HS-12: verify snapshot failure burns accepted-job error budget.
- Drill HS-13: verify denied snapshot does not burn availability budget.
- Drill HS-14: verify snapshot replay does not mutate live board.

## Export-Render Drills
- Drill ER-01: saturate export queue.
- Drill ER-02: fail renderer after job acceptance.
- Drill ER-03: deny artifact download for wrong tenant.
- Drill ER-04: deny artifact download for wrong principal.
- Drill ER-05: corrupt generated artifact hash.
- Drill ER-06: expire artifact before download.
- Drill ER-07: verify PDF export fixture for Mural Enterprise displacement.
- Drill ER-08: verify diagram export fixture for Lucidspark displacement.
- Drill ER-09: verify tenant-governed export fixture for Microsoft Whiteboard displacement.
- Drill ER-10: verify Miro Enterprise board export fixture.
- Drill ER-11: verify render format failure isolates the job.
- Drill ER-12: verify artifact access logs include tenant, principal, and data class.
- Drill ER-13: verify rollback disables download without deleting audit event.
- Drill ER-14: verify export outage does not block board open.

## Template-Install Drills
- Drill TI-01: refuse template install without DealSet settlement id.
- Drill TI-02: fail settlement submission after preview.
- Drill TI-03: deny template install for incompatible pack overlay.
- Drill TI-04: rollback installed template grant.
- Drill TI-05: test duplicate install idempotency.
- Drill TI-06: test marketplace source unavailable.
- Drill TI-07: verify Miro Enterprise template-library fixture.
- Drill TI-08: verify Mural Enterprise facilitation-template fixture.
- Drill TI-09: verify FigJam starter-template migration fixture.
- Drill TI-10: verify template preview does not mutate board.
- Drill TI-11: verify template grant carries tenant and principal.
- Drill TI-12: verify DealSet refusal does not burn availability budget.
- Drill TI-13: verify settlement errors page only after accepted install.
- Drill TI-14: verify rollback token is audit-chain linked.

## Cross-Capability Drills
- Drill XC-01: board open succeeds while presence is disabled.
- Drill XC-02: append refuses while board remains readable.
- Drill XC-03: export outage leaves append path untouched.
- Drill XC-04: snapshot queue saturation leaves presence fanout untouched.
- Drill XC-05: template settlement outage leaves existing templates usable.
- Drill XC-06: tenant-specific quota blocks one tenant only.
- Drill XC-07: cell-specific outage blocks promotion for that cell only.
- Drill XC-08: pack-specific policy error blocks that pack only.
- Drill XC-09: audit-chain outage blocks mutating promotion.
- Drill XC-10: capacity-admission breach stops new large sessions.
- Drill XC-11: cost-budget breach throttles async exports first.
- Drill XC-12: emergency-services bypass remains explicit and audited.
- Drill XC-13: marketplace failure does not affect non-marketplace board editing.
- Drill XC-14: benchmark migration import failure does not affect native boards.

## Evidence Capture
- Capture drill id.
- Capture capability.
- Capture tenant id.
- Capture cell.
- Capture region.
- Capture principal.
- Capture audience type.
- Capture data class.
- Capture pack overlay.
- Capture benchmark source.
- Capture injected fault.
- Capture expected refusal.
- Capture actual refusal.
- Capture SLO impact.
- Capture error-budget impact.
- Capture dashboard link.
- Capture runbook link.
- Capture audit event id.
- Capture rollback action.
- Capture stop condition.

## Safety Constraints
- Drills default to isolated test tenants.
- Drills cannot delete board history.
- Drills cannot suppress audit-chain publication.
- Drills cannot bypass Cedar default deny.
- Drills cannot fabricate DealSet settlement success.
- Drills cannot infer tenant from board id.
- Drills cannot use vendor-named service folders.
- Drills cannot edit ADR-0321.
- Drills cannot run against production without a separate external approval path.
- Drills cannot promote capabilities by themselves.
- Drills cannot mark audit findings closed.
- Drills cannot mutate catalog records outside their evidence references.

## Workflow Decisions
- Workflow decision: each drill declares setup, injection, expected refusal, recovery, and rollback before execution.
- Workflow decision: drills run capability-by-capability so one failing path does not mask another.
- Workflow decision: export and template drills require DealSet and residency prechecks before fault injection.
- Workflow decision: presence drills may shed volatile cursor state but must preserve join, leave, and lease evidence.
- Workflow decision: replay drills must prove deterministic recovery from sealed history, not from a manually patched snapshot.
- Workflow decision: drill evidence feeds IP-021 promotion and IP-025 audit closeout without directly changing their states.

## Acceptance Criteria
- Chaos drill pack names Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
- Chaos drill pack preserves ADR-0321 and the existing ADR binding set.
- Chaos drill pack defines drills for all six whiteboard capabilities.
- Chaos drill pack separates durable board data from volatile presence data.
- Chaos drill pack separately covers export artifacts, history snapshots, and template settlement.
- Chaos drill pack captures tenant, principal, audience, data class, pack, cell, and benchmark evidence.
- Chaos drill pack defines safe stop conditions and rollback expectations.
- Chaos drill pack links evidence to SLO promotion in IP-021.
- Chaos drill pack supports audit closeout in IP-025.
- Chaos drill pack does not require `oya vcs verify`, `done`, or `promote`.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
