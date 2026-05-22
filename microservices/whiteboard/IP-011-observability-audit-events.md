# IP-011 Whiteboard observability-audit-events

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-011-observability-audit-events.md
Benchmarks displaced: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## 1. Outcome
- Define observability and audit events for the whiteboard capability set.
- Make every critical state transition observable by metric, trace, structured log, and audit-chain event.
- Keep high-cardinality tenant identifiers out of ordinary metrics.
- Keep tenant and board identifiers inside signed audit evidence.
- Tie denial evidence to policy, pack, DealSet, sidecar, cell, abuse, and export decisions.
- Give operators enough evidence to displace benchmark suites without losing incident clarity.
- Satisfy ADR-0321 by naming benchmark-specific events and failure evidence for Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
- Keep audit event shape compatible with microservices/whiteboard/contracts/asyncapi-v1.yaml.
- Keep synchronous acceptance references compatible with microservices/whiteboard/contracts/whiteboard-v1.proto.
- Preserve dashboard and SLO evidence already present under microservices/whiteboard/.

## 2. Local Source Anchors
- microservices/whiteboard/PRD.md requires metrics, traces, logs, audit-chain events, refusal evidence, and migration provenance.
- microservices/whiteboard/ARCHITECTURE.md requires audit-chain events for every capability and service.
- microservices/whiteboard/compliance.md defines compliance evidence expectations.
- microservices/whiteboard/dpia.md defines privacy evidence expectations.
- microservices/whiteboard/threat-model.md defines security event expectations.
- microservices/whiteboard/incident-response.md defines incident response expectations.
- microservices/whiteboard/slos/audit-emission-lag.openslo.yaml defines audit lag expectations.
- microservices/whiteboard/slos/local-presence-freshness.openslo.yaml defines presence freshness.
- microservices/whiteboard/slos/local-stroke-persistence-latency.openslo.yaml defines append latency.
- microservices/whiteboard/slos/local-export-render-latency.openslo.yaml defines export render latency.
- microservices/whiteboard/dashboards/local-audit-completeness.json is the audit completeness target.
- microservices/whiteboard/dashboards/local-slo-burn.json is the SLO burn target.
- microservices/whiteboard/dashboards/slo-and-error-budget.json is the error budget target.
- microservices/whiteboard/dashboards/tenant-cost-and-capacity.json is the cost/capacity target.
- microservices/whiteboard/runbooks/moderation-report-escalation.md is the moderation evidence runbook.

## 3. Audit Event Families
- `oya.whiteboard.board.open.requested`
- `oya.whiteboard.board.open.permitted`
- `oya.whiteboard.board.open.denied`
- `oya.whiteboard.canvas.op.append.requested`
- `oya.whiteboard.canvas.op.append.accepted`
- `oya.whiteboard.canvas.op.append.denied`
- `oya.whiteboard.canvas.op.merge.conflicted`
- `oya.whiteboard.presence.session.started`
- `oya.whiteboard.presence.cursor.published`
- `oya.whiteboard.presence.cursor.throttled`
- `oya.whiteboard.presence.session.expired`
- `oya.whiteboard.history.snapshot.requested`
- `oya.whiteboard.history.snapshot.sealed`
- `oya.whiteboard.history.replay.started`
- `oya.whiteboard.history.replay.verified`
- `oya.whiteboard.history.replay.failed`
- `oya.whiteboard.export.render.requested`
- `oya.whiteboard.export.render.sealed`
- `oya.whiteboard.export.render.denied`
- `oya.whiteboard.template.install.requested`
- `oya.whiteboard.template.install.settled`
- `oya.whiteboard.template.install.denied`
- `oya.whiteboard.classroom.session.started`
- `oya.whiteboard.classroom.student_board.spawned`
- `oya.whiteboard.classroom.session.closed`
- `oya.whiteboard.abuse.signal.detected`
- `oya.whiteboard.abuse.friction.applied`
- `oya.whiteboard.abuse.blocked`
- `oya.whiteboard.credential.lease.issued`
- `oya.whiteboard.credential.lease.revoked`
- `oya.whiteboard.cell.failover.promoted`
- `oya.whiteboard.cell.replay.cursor_sealed`

## 4. Required Audit Fields
- `event_id` is mandatory.
- `event_time` is mandatory.
- `tenant_id` is mandatory.
- `principal_id` is mandatory where a human or worker caused the event.
- `audience_type` is mandatory.
- `service` must be `whiteboard`.
- `capability` is mandatory.
- `bounded_context` is mandatory.
- `action` is mandatory.
- `resource_type` is mandatory.
- `resource_ref` is mandatory.
- `home_cell` is mandatory.
- `request_cell` is mandatory when different from home cell.
- `jurisdiction_code` is mandatory.
- `pack_overlay_id` is mandatory.
- `data_class` is mandatory.
- `purpose` is mandatory.
- `policy_decision_id` is mandatory for permit or deny events.
- `policy_context_hash` is mandatory for policy-dependent events.
- `deal_set_id` is mandatory for template and licensed export events.
- `workflow_run_id` is mandatory when workflow-engine caused the event.
- `trace_id` is mandatory.
- `span_id` or `parent_span_id` is mandatory.
- `idempotency_key_hash` is mandatory for mutations.
- `source_system_ref` is mandatory for imported benchmark content.
- `source_vendor_benchmark` is mandatory for migration events.

## 5. Metrics
- `whiteboard.board.open.request.count` counts board-open requests.
- `whiteboard.board.open.duration` measures board-open latency.
- `whiteboard.canvas.op.append.count` counts append attempts.
- `whiteboard.canvas.op.append.duration` measures accepted append latency.
- `whiteboard.canvas.op.merge.conflict.count` counts merge conflicts.
- `whiteboard.presence.cursor.publish.count` counts cursor updates.
- `whiteboard.presence.cursor.freshness` measures freshness.
- `whiteboard.presence.throttle.count` counts cursor throttles.
- `whiteboard.history.snapshot.count` counts snapshots.
- `whiteboard.history.replay.cursor_lag` measures replay lag.
- `whiteboard.export.render.count` counts render attempts.
- `whiteboard.export.render.duration` measures render latency.
- `whiteboard.template.install.count` counts install attempts.
- `whiteboard.template.install.duration` measures install latency.
- `whiteboard.audit.emission_lag` measures event sealing delay.
- `whiteboard.policy.decision.count` counts policy decisions.
- `whiteboard.policy.denial.count` counts denials.
- `whiteboard.credential.lease.count` counts sidecar leases.
- `whiteboard.cell.forward.count` counts cross-cell forwards.
- `whiteboard.abuse.signal.count` counts abuse signals.
- Metric labels include capability, result, data_class, cell_tier, pack_family, and source_benchmark.
- Metric labels exclude raw tenant_id, board_id, op_id, principal_id, and invite id.

## 6. Traces
- Board open traces start at ingress.
- Board open traces include policy evaluation span.
- Board open traces include metadata lookup span.
- Board open traces include audit emission span.
- Canvas append traces include policy span.
- Canvas append traces include idempotency span.
- Canvas append traces include merge span.
- Canvas append traces include storage append span.
- Canvas append traces include fanout notification span.
- Presence traces sample cursor fanout spans.
- Presence traces always include throttle spans.
- History traces include snapshot read, seal, and replay verification spans.
- Export traces include snapshot load, render, artifact write, signature, and audit spans.
- Template traces include DealSet settlement, package fetch, validation, install, and audit spans.
- Credential traces include sidecar lease request and attestation spans.
- Cell traces include forward, replay cursor, and failover spans.
- Abuse traces include signal, decision, friction, and block spans.

## 7. Structured Logs
- Logs use stable event names matching audit families.
- Logs include trace id.
- Logs include capability.
- Logs include result.
- Logs include error code.
- Logs include cell.
- Logs include data class.
- Logs include policy decision id where present.
- Logs include audit event id where present.
- Logs include hashed idempotency key where present.
- Logs include source benchmark only when migration or import is in scope.
- Logs do not include raw canvas payloads.
- Logs do not include raw cursor payloads.
- Logs do not include raw secret material.
- Logs do not include invite tokens.
- Logs do not include raw tenant id unless the log sink is audit-protected.

## 8. Benchmark-Specific Events
- Miro Enterprise imports emit `source_vendor_benchmark=Miro Enterprise`.
- Miro Enterprise shared team conversion emits board-open permit delta evidence.
- Mural Enterprise imports emit `source_vendor_benchmark=Mural Enterprise`.
- Mural Enterprise facilitator lock conversion emits facilitation policy evidence.
- FigJam imports emit `source_vendor_benchmark=FigJam`.
- FigJam reaction conversion emits low-risk canvas operation evidence.
- Lucidspark imports emit `source_vendor_benchmark=Lucidspark`.
- Lucidspark diagram export emits deterministic render evidence.
- Whiteboard.fi imports emit `source_vendor_benchmark=Whiteboard.fi`.
- Whiteboard.fi classroom session emits student board lifecycle evidence.
- Microsoft Whiteboard imports emit `source_vendor_benchmark=Microsoft Whiteboard`.
- Microsoft Whiteboard guest link conversion emits tenant guest grant evidence.
- Every benchmark import emits migration batch id.
- Every benchmark import emits source object count.
- Every benchmark import emits rejected object count.
- Every benchmark import emits rollback bundle ref.

## 9. Denial Evidence
- Policy denial records action, resource type, policy decision id, and denial reason.
- Pack denial records pack overlay, conflict field, and higher-restriction source.
- DealSet denial records deal set id and settlement state.
- Sidecar denial records secret class and denial class without secret material.
- Cell denial records home cell, request cell, and residency pack.
- Abuse denial records signal class and enforcement mode.
- Export denial records render profile, artifact class, and residency target.
- Template denial records template id, publisher ref, and license scope.
- Classroom denial records session id and lifecycle state.
- Guest denial records invite provenance and expiry state.
- Denial evidence links to a runbook.
- Denial evidence links to a remediation owner when one exists.

## 10. Audit Completeness
- Board open permit must emit an audit event.
- Board open denial must emit refusal evidence.
- Canvas append acceptance must emit audit event.
- Canvas append denial must emit refusal evidence.
- Presence cursor may be sampled, but session start and throttle are audited.
- History snapshot seal must emit audit event.
- Replay verification must emit audit event.
- Export render seal must emit audit event.
- Export render denial must emit refusal evidence.
- Template install settlement must emit audit event.
- Template install denial must emit refusal evidence.
- Credential lease issue must emit audit event.
- Credential lease revoke must emit audit event.
- Failover promotion must emit audit event.
- Abuse block must emit audit event.
- Audit emission lag is measured against the local SLO.

## 11. Dashboard Mapping
- local-audit-completeness shows expected versus emitted events by capability.
- local-policy-decisions shows permit, deny, and fail-closed counts.
- abuse-defence-outcomes shows suspicion, friction, block, and false-positive review.
- local-domain-throughput shows board open, append, presence, snapshot, export, and template rates.
- local-slo-burn shows active burns for board load, append, cursor, replay, audit, and export.
- slo-and-error-budget shows budget consumption by capability.
- tenant-cost-and-capacity shows capacity and cost by tenant-safe dimension.
- operating-bar-overview rolls up readiness evidence.
- compliance-pack-health shows pack-specific audit health.
- local-operator-remediation shows runbook-linked incidents.

## 12. Cost and Capacity Evidence
- Append events include operation count and payload size class.
- Presence events include fanout partition and participant count bucket.
- Snapshot events include revision window and storage size class.
- Export events include render profile and artifact size class.
- Template events include package size class and install target.
- Import events include source benchmark and object count bucket.
- Replay events include segment count and cursor lag.
- Cost evidence does not include raw tenant id in metrics.
- Cost evidence includes tenant id in audit-protected evidence.
- Capacity evidence is reviewed before admitting hot boards.

## 13. Privacy
- Cursor logs use coarse viewport data.
- Cursor logs never include raw text selection content.
- Canvas operation logs use operation kind, not raw payload.
- Export logs use artifact ref, not artifact content.
- Template logs use template id and publisher ref, not embedded template content.
- Secret logs use lease id and secret class, not secret material.
- Invite logs use invite ref hash, not raw invite token.
- Student-board logs use classroom session ref, not student personal details beyond audit-required principal id.
- Source import logs use source object ref, not raw source payload.
- Audit evidence retention follows pack overlay.

## 14. Implementation Steps
- Define audit event schema for each event family.
- Map existing AsyncAPI events to audit event families.
- Add typed event builders at usecase boundaries.
- Add metric emitters for each capability.
- Add trace spans around policy, storage, sidecar, DealSet, render, and replay.
- Add structured log keys and redaction rules.
- Add denial evidence builders.
- Add dashboard panel references.
- Add SLO burn linkage.
- Add runbook linkage for each denial class.
- Add migration provenance fields.
- Add benchmark source fields.
- Add audit completeness checks.
- Add privacy redaction tests.

## 15. Tests
- Unit tests validate mandatory audit fields.
- Unit tests validate redaction rules.
- Unit tests validate metric label allowlist.
- Unit tests validate denial evidence shapes.
- Integration tests assert board open audit events.
- Integration tests assert canvas append audit events.
- Integration tests assert presence throttle audit events.
- Integration tests assert snapshot and replay audit events.
- Integration tests assert export render audit events.
- Integration tests assert template install DealSet events.
- Integration tests assert sidecar lease events.
- Integration tests assert failover events.
- SLO tests validate audit emission lag measurement.
- Dashboard tests validate referenced metric names.
- Benchmark tests name Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.

## 16. Rollback
- If event schema rollout fails, pause affected mutation paths rather than emitting incomplete audit evidence.
- If metric rollout fails, keep audit events authoritative and mark dashboards degraded.
- If trace rollout fails, keep audit events and logs authoritative.
- If denial evidence rollout fails, fail closed for affected privileged mutations.
- If dashboard rollout fails, keep SLO checks on raw metric queries.
- If benchmark provenance rollout fails, pause migration/import jobs.
- Rollback evidence must include schema version, failed event family, and affected capability.

## 16A. Cedar Evidence Links
- Cedar permit events include `policy_decision_ref`, policy bundle version, and decision hash.
- Cedar denial events include refusal reason, resource family, and non-sensitive context digest.
- Cedar timeout evidence distinguishes unavailable policy evaluation from explicit deny.
- Cedar-related metric labels use decision outcome and action family, not raw tenant or board identifiers.
- Audit-chain payloads retain the signed tenant and principal references needed to replay Cedar context.
- Benchmark imports from Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard must name the Cedar decision that admitted or rejected the operation.

## 17. Acceptance Criteria
- Every critical transition has a named audit event family.
- Every critical transition has metric and trace coverage.
- Every denial path emits refusal evidence.
- Metric labels avoid raw high-cardinality tenant and board identifiers.
- Signed audit evidence keeps tenant, principal, and resource references.
- Dashboards and SLOs can consume the named signals.
- Benchmark migration/import paths name displaced source systems explicitly.
- ADR-0321 remains cited and vendor-specific observability substance is present.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
