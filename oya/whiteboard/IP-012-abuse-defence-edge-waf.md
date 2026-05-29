# IP-012 Whiteboard abuse-defence-edge-waf

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-012-abuse-defence-edge-waf.md
Benchmarks displaced: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## 1. Outcome
- Define edge and in-service abuse defence for the whiteboard collaboration surface.
- Protect board open, canvas append, presence sync, history snapshot, export render, and template install without adding friction to clean collaboration.
- Block cursor storms, write floods, invite spraying, classroom session abuse, export scraping, and template package abuse.
- Preserve tenant-scoped Cedar authority as the source of truth for authorization.
- Preserve WAF and edge controls as suspicion and rate controls, not as hidden policy grants.
- Preserve audit-chain evidence for every block, throttle, challenge, or operator escalation.
- Satisfy ADR-0321 by naming abuse cases specific to Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard displacement.
- Keep emergency-services bypass out of scope except for respecting microservices/whiteboard/policy/emergency-services-bypass.cedar when a broader platform rule invokes it.
- Keep clean collaboration fast enough to meet local SLOs.
- Keep all WAF claims tied to local policy, IaC, dashboards, runbooks, and SLO files.

## 2. Local Source Anchors
- microservices/whiteboard/threat-model.md defines threat posture.
- microservices/whiteboard/policy/abuse-defence.cedar defines the policy-side abuse controls.
- microservices/whiteboard/iac/edge-waf.yaml defines edge WAF deployment posture.
- microservices/whiteboard/iac/production-ingress.yaml defines production ingress.
- microservices/whiteboard/iac/network-policy.yaml defines network boundaries.
- microservices/whiteboard/iac/ech-config.yaml defines encrypted client hello posture.
- microservices/whiteboard/iac/pqc-cert.yaml defines PQC certificate posture.
- microservices/whiteboard/dashboards/abuse-defence-outcomes.json is the abuse outcome dashboard.
- microservices/whiteboard/runbooks/cursor-storm-throttle.md is the cursor storm runbook.
- microservices/whiteboard/runbooks/canvas-op-hotspot.md is the canvas operation hotspot runbook.
- microservices/whiteboard/runbooks/local-session-throttle-activation.md is the local throttle runbook.
- microservices/whiteboard/runbooks/moderation-report-escalation.md is the moderation escalation runbook.
- microservices/whiteboard/slos/local-cursor-latency.openslo.yaml defines clean cursor latency expectations.
- microservices/whiteboard/slos/local-stroke-persistence-latency.openslo.yaml defines append latency expectations.
- microservices/whiteboard/slos/local-board-load-time.openslo.yaml defines board load expectations.
- microservices/whiteboard/slos/audit-emission-lag.openslo.yaml defines evidence emission expectations.

## 3. Abuse Domain Model
- `AbuseSignal` is a scored observation, not an authorization decision.
- `AbuseSignal.signal_id` is mandatory.
- `AbuseSignal.tenant_id` is mandatory for tenant-scoped traffic.
- `AbuseSignal.board_id_hash` is used for metrics and dashboards.
- `AbuseSignal.principal_id_hash` is used for metrics and dashboards.
- `AbuseSignal.source_ip_prefix` is stored only at approved granularity.
- `AbuseSignal.user_agent_family` is stored for WAF correlation.
- `AbuseSignal.capability` identifies board-open, canvas-op-append, presence-sync, history-snapshot, export-render, or template-marketplace-install.
- `AbuseSignal.signal_class` identifies cursor-storm, op-flood, invite-spray, export-scrape, template-probe, classroom-spawn-flood, replay-probe, or credential-lease-probe.
- `AbuseDecision` is the enforcement decision derived from one or more signals.
- `AbuseDecision.action` is allow, sample, throttle, challenge, shadow-block, block, quarantine, or escalate.
- `AbuseDecision.policy_decision_id` links to Cedar when policy context is involved.
- `AbuseDecision.audit_event_id` links to signed evidence.
- `AbuseWindow` defines a rate window by tenant, board, principal, connection, capability, and cell.
- `AbuseWindow` does not create cross-tenant aggregation.
- `AbuseCase` ties repeated decisions to a moderation or operator workflow.
- `AbuseCase.owner_team` is axis-whiteboard unless escalated to security.

## 4. Edge Command Deltas
- `BoardOpenEdgeCheck` runs before board metadata fetch.
- `BoardOpenEdgeCheck` detects invite spraying.
- `BoardOpenEdgeCheck` detects repeated denied guest opens.
- `BoardOpenEdgeCheck` detects impossible travel only as suspicion.
- `CanvasAppendEdgeCheck` runs before append admission.
- `CanvasAppendEdgeCheck` detects operation flood by board and connection.
- `CanvasAppendEdgeCheck` detects shape deletion bursts.
- `CanvasAppendEdgeCheck` detects sticky-note spam bursts.
- `PresencePublishEdgeCheck` runs before cursor fanout.
- `PresencePublishEdgeCheck` detects cursor storm.
- `PresencePublishEdgeCheck` detects participant spoof attempts.
- `HistoryReplayEdgeCheck` runs before replay or snapshot read.
- `HistoryReplayEdgeCheck` detects replay scraping.
- `ExportRenderEdgeCheck` runs before render queue admission.
- `ExportRenderEdgeCheck` detects artifact scraping and repeated forbidden profiles.
- `TemplateInstallEdgeCheck` runs before package fetch.
- `TemplateInstallEdgeCheck` detects template enumeration and license probing.
- `ClassroomSpawnEdgeCheck` runs before Whiteboard.fi-style student board spawn.
- `CredentialLeaseEdgeCheck` runs before sidecar lease for source import, export, or template flows.

## 5. Event Deltas
- Emit `oya.whiteboard.abuse.signal.detected` for each scored signal above reporting threshold.
- Emit `oya.whiteboard.abuse.signal.sampled` when signal is retained only for trend evidence.
- Emit `oya.whiteboard.abuse.decision.allow` when a suspicious request is explicitly allowed.
- Emit `oya.whiteboard.abuse.decision.throttle` when rate limits slow a request.
- Emit `oya.whiteboard.abuse.decision.challenge` when a clean-room challenge is required.
- Emit `oya.whiteboard.abuse.decision.shadow_block` when suspicious background probes are hidden from attackers.
- Emit `oya.whiteboard.abuse.decision.block` when a request is denied by abuse defence.
- Emit `oya.whiteboard.abuse.case.opened` when repeated signals require operator review.
- Emit `oya.whiteboard.abuse.case.escalated` when moderation or security owns follow-up.
- Emit `oya.whiteboard.abuse.false_positive.recorded` when an operator reverses an abuse decision.
- Emit `oya.whiteboard.edge.waf.rule.matched` for WAF rule matches.
- Emit `oya.whiteboard.edge.waf.rule.updated` for controlled rule rollout.
- Emit `oya.whiteboard.edge.waf.bypass.denied` for unapproved bypass attempts.
- Emit `oya.whiteboard.edge.waf.bypass.applied` only when an approved platform bypass applies.

## 6. Proto and Contract Deltas
- Add abuse check metadata to internal command envelopes, not to domain objects.
- `AbuseCheckContext` carries tenant id.
- `AbuseCheckContext` carries principal id hash for metrics and principal id for audit-protected evidence.
- `AbuseCheckContext` carries board id hash for metrics and board id for audit-protected evidence.
- `AbuseCheckContext` carries capability.
- `AbuseCheckContext` carries request cell.
- `AbuseCheckContext` carries home cell.
- `AbuseCheckContext` carries connection id hash.
- `AbuseCheckContext` carries source ip prefix.
- `AbuseCheckContext` carries user agent family.
- `AbuseCheckContext` carries policy decision id when available.
- `AbuseCheckContext` carries pack overlay id.
- `AbuseCheckContext` carries trace id.
- `AbuseDecisionRef` carries decision id.
- `AbuseDecisionRef` carries action.
- `AbuseDecisionRef` carries throttle tier when applicable.
- `AbuseDecisionRef` carries retry-after when applicable.
- `AbuseDecisionRef` carries audit event id.
- `AbuseDecisionRef` carries runbook ref.
- Public contracts expose only stable error classes, not internal signal scores.

## 7. Cedar Facts
- Cedar receives `context.abuse.signal_class` for policy-aware abuse cases.
- Cedar receives `context.abuse.throttle_tier` for rate decisions that alter capability behavior.
- Cedar receives `context.abuse.prior_denials_count_bucket`.
- Cedar receives `context.abuse.connection_reputation`.
- Cedar receives `context.abuse.source_ip_reputation`.
- Cedar receives `context.abuse.classroom_session_spawn_count_bucket`.
- Cedar receives `context.abuse.export_attempt_count_bucket`.
- Cedar receives `context.abuse.template_probe_count_bucket`.
- Cedar receives `context.abuse.cursor_rate_bucket`.
- Cedar receives `context.abuse.operation_rate_bucket`.
- Cedar does not receive raw cursor payloads.
- Cedar does not receive raw canvas operation payloads.
- Cedar does not receive raw invite tokens.
- Cedar does not receive raw secret material.
- Policy denial and abuse block remain separate result classes.
- Abuse suspicion cannot grant access that Cedar denied.
- Cedar permit cannot bypass WAF block when platform safety rules require blocking.
- WAF allow cannot bypass Cedar denial.

## 8. Board Open Defence
- Detect repeated board-open attempts with expired invite references.
- Detect repeated board-open attempts for boards outside tenant scope.
- Detect guest invite enumeration.
- Detect board id guessing.
- Detect high-volume opens from one connection.
- Detect high-volume opens against one board.
- Detect high-volume opens across boards in one tenant.
- Apply sample for low-risk spikes.
- Apply throttle for repeated misses.
- Apply challenge for suspicious guest opens.
- Apply block for invite enumeration.
- Emit audit evidence for denied opens.
- Link open abuse to local-board-load-time SLO when throttle affects clean traffic.
- Microsoft Whiteboard guest-link parity is preserved by translating links to explicit grants.
- Anonymous link abuse is refused, even if a displaced benchmark allowed it.

## 9. Canvas and CRDT Defence
- Detect append rate above board budget.
- Detect append rate above principal budget.
- Detect append rate above connection budget.
- Detect deletion bursts.
- Detect shape movement oscillation loops.
- Detect sticky-note spam.
- Detect malformed operation payload references.
- Detect stale revision replay attempts.
- Detect duplicate idempotency key abuse.
- Detect cross-cell append floods.
- Throttle cursor-like cosmetic operations before semantic operations.
- Throttle low-risk reactions before blocking board edits.
- Block malformed operation references.
- Block stale replay attempts after threshold.
- Preserve accepted operation log for replay.
- Preserve denied operation evidence for incident review.
- Miro Enterprise and FigJam displacement requires fast collaboration, so clean append paths remain low-friction.
- Mural Enterprise facilitator locks can reduce abuse window for workshop boards.

## 10. Presence Defence
- Detect cursor updates above per-connection budget.
- Detect cursor updates above per-board budget.
- Detect participant spoofing.
- Detect connection churn.
- Detect fanout partition hotspot.
- Detect repeated stale cursor epochs.
- Detect impossible participant count changes.
- Throttle cursor publish before session close.
- Close session after sustained spoofing.
- Keep teacher/facilitator visibility into throttled participants.
- Preserve local-cursor-latency SLO for clean users.
- Emit `cursor-storm-throttle` runbook links.
- FigJam parity requires smooth cursors for clean participants.
- Whiteboard.fi parity requires classroom owner visibility during throttle.

## 11. History and Replay Defence
- Detect repeated snapshot reads across revision windows.
- Detect replay cursor probing.
- Detect requests for forbidden revision ranges.
- Detect cross-cell replay requests outside residency pack.
- Detect high-volume history reads by one principal.
- Detect high-volume history reads by one source ip prefix.
- Throttle replay read probes.
- Block forbidden revision windows.
- Block residency-violating replay requests.
- Seal replay denial evidence.
- Keep replay cursor monotonic.
- Keep replay cursor tenant-scoped.
- Link failed replay to local-regional-board-replay runbook when cell movement is involved.
- Lucidspark parity requires history depth, but replay scraping is still blocked.

## 12. Export Defence
- Detect repeated export render attempts.
- Detect artifact class probing.
- Detect render profile probing.
- Detect export attempts after policy denial.
- Detect export attempts after DealSet hold.
- Detect export attempts outside residency target.
- Detect high-volume download of sealed artifacts.
- Throttle repeated safe exports.
- Challenge suspicious artifact download.
- Block forbidden profile probing.
- Block residency violations.
- Block export attempts during audit-chain pause.
- Emit export-render-failure runbook links.
- Microsoft Whiteboard and Lucidspark export parity requires trustworthy export evidence, not broad download shortcuts.

## 13. Template and Marketplace Defence
- Detect template id enumeration.
- Detect publisher ref probing.
- Detect license scope probing.
- Detect install attempts during DealSet hold.
- Detect repeated install rollback loops.
- Detect package hash mismatch.
- Detect package provenance mismatch.
- Detect cross-tenant template install attempts.
- Throttle template enumeration.
- Block package hash mismatch.
- Block provenance mismatch.
- Block DealSet hold bypass.
- Emit template rollback runbook links.
- Miro Enterprise and Mural Enterprise template parity requires marketplace safety, not vendor trust inheritance.

## 14. Classroom Defence
- Detect student board spawn floods.
- Detect classroom session reuse after expiry.
- Detect teacher authority spoofing.
- Detect student identity churn.
- Detect mass export from student boards.
- Detect cross-classroom board opens.
- Throttle spawn when classroom budget is exceeded.
- Close expired sessions.
- Block teacher spoof attempts.
- Block cross-classroom opens.
- Preserve classroom owner audit evidence.
- Preserve student-board lifecycle evidence.
- Whiteboard.fi parity is satisfied by managed ephemeral boards with explicit expiry.

## 15. WAF Rule Groups
- `whiteboard-board-open-rules` protects board open endpoints.
- `whiteboard-canvas-op-rules` protects operation append endpoints.
- `whiteboard-presence-rules` protects presence endpoints.
- `whiteboard-history-rules` protects snapshot and replay endpoints.
- `whiteboard-export-rules` protects render and download endpoints.
- `whiteboard-template-rules` protects marketplace template endpoints.
- `whiteboard-classroom-rules` protects classroom session endpoints.
- `whiteboard-source-import-rules` protects migration import callbacks.
- `whiteboard-credential-lease-rules` protects sidecar-facing lease endpoints.
- Rules roll out in observe mode first.
- Rules graduate to throttle.
- Rules graduate to block only after false-positive review.
- Rules include rollback ids.
- Rules include owner team.
- Rules include dashboard panel references.

## 16. SLO Interaction
- Abuse defence must not burn local-board-load-time for clean users.
- Abuse defence must not burn local-stroke-persistence-latency for clean users.
- Abuse defence must not burn local-cursor-latency for clean users.
- Abuse defence may intentionally slow suspicious traffic.
- Suspicious traffic is measured separately from clean SLO traffic.
- Throttled traffic emits separate SLO exclusion evidence.
- Blocked traffic emits refusal evidence.
- Audit emission lag still applies to abuse decisions.
- WAF rollout has its own false-positive budget.
- False-positive budget is reviewed before block-mode promotion.
- Cursor storm mitigation can trade suspect cursor freshness for board stability.
- Canvas hotspot mitigation can trade suspect write throughput for replay integrity.

## 17. Evidence Fields
- `abuse_signal_id`
- `abuse_decision_id`
- `signal_class`
- `signal_score_bucket`
- `enforcement_action`
- `throttle_tier`
- `retry_after_ms`
- `false_positive_review_id`
- `runbook_ref`
- `waf_rule_id`
- `waf_rule_version`
- `policy_decision_id`
- `audit_event_id`
- `trace_id`
- `capability`
- `home_cell`
- `request_cell`
- `source_benchmark`
- `migration_batch_id`
- `deal_set_id`
- `pack_overlay_id`
- `data_class`
- `classroom_session_id`

## 18. Workflow Decisions
- Abuse case creation opens workflow-engine remediation.
- Cursor storm opens a local operator workflow.
- Canvas hotspot opens a capacity review workflow.
- Export scraping opens a security review workflow.
- Template probing opens a marketplace integrity workflow.
- Classroom spawn flood opens an education-pack review workflow.
- False-positive reversal opens a WAF rule review workflow.
- Block-mode promotion requires reviewer approval.
- Emergency bypass applicability is recorded as platform policy evidence.
- Repeated abuse from a source import opens migration quarantine workflow.
- Repeated credential lease probes open sidecar hardening workflow.

## 19. Failure and Replay Cases
- False positive throttle must be replayable from audit evidence.
- False positive block must be reversible with operator approval.
- Missing abuse evidence fails closed for block promotion.
- Missing WAF rule version fails rollback.
- Edge outage falls back to in-service checks only when policy allows.
- In-service abuse engine outage allows clean low-risk reads but blocks high-risk mutations if no decision can be made.
- Audit-chain outage blocks new block-mode promotion.
- Policy outage keeps Cedar fail-closed for mutations.
- Dashboard outage does not disable enforcement.
- Replay re-evaluates historical signals with original rule version.
- Replay compares old and new rule decisions before promotion.
- Replay protects against silently changing false-positive history.

## 20. Tests
- Unit tests validate AbuseSignal mandatory fields.
- Unit tests validate AbuseDecision allowed actions.
- Unit tests validate WAF rule version presence.
- Unit tests validate metric label redaction.
- Cedar tests validate abuse context facts.
- Board-open tests detect invite spraying.
- Canvas tests detect operation flood.
- Canvas tests detect stale revision replay abuse.
- Presence tests detect cursor storm.
- History tests detect replay probing.
- Export tests detect artifact scraping.
- Template tests detect template enumeration.
- Classroom tests detect spawn flood.
- Sidecar tests detect credential lease probing.
- Workflow tests create abuse cases for repeated signals.
- Replay tests compare old and new rule versions.
- Rollback tests revert block-mode rule to observe mode.
- SLO tests separate clean and suspect traffic.
- Benchmark tests name Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.

## 21. Rollback
- Roll back a WAF rule by rule id and version.
- Roll back from block mode to throttle mode first.
- Roll back from throttle mode to observe mode if false positives continue.
- Preserve all decision evidence during rollback.
- Reopen blocked board opens when operator approves false-positive reversal.
- Replay throttled cursor sessions to prove no state was lost.
- Replay denied canvas operations only as dry-run, never as automatic writes.
- Requeue export renders only after policy, residency, and DealSet checks pass.
- Requeue template installs only after package provenance and settlement pass.
- Close classroom sessions only through lifecycle workflow.
- Revoke suspicious credential leases when abuse rollback touches sidecar paths.
- Attach rollback evidence to abuse case.

## 22. Acceptance Criteria
- Abuse defence covers all six whiteboard capabilities.
- WAF rules are capability-specific and versioned.
- Cedar receives bounded abuse facts without raw content leakage.
- Clean traffic remains measured separately from suspect traffic.
- Every throttle, challenge, block, and escalation emits audit evidence.
- Every WAF block has a rollback path.
- Every false positive has a review and reversal path.
- Benchmark displacement is explicit for Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
- ADR-0321 remains cited with vendor-specific abuse and failure cases.
- No vendor-specific service boundary is introduced.


## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
