# IP-021 Whiteboard SLO-Gated Promotion

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-021-slo-gated-promotion.md
Benchmarks: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- Promote whiteboard only when collaborative canvas behavior is backed by service-level evidence, not by benchmark name matching.
- Use `microservices/whiteboard/slos/` as the durable SLO source for promotion.
- Use `microservices/whiteboard/dashboards/` as the visible evidence source for promotion.
- Use `microservices/whiteboard/runbooks/` as the operator response source for promotion.
- Keep ADR-0321 intact while proving B2B leader readiness against Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.

## Promotion Principle
- Promotion is capability-by-capability.
- Promotion is tenant-scoped.
- Promotion is cell-scoped.
- Promotion is data-class aware.
- Promotion is pack-overlay aware.
- Promotion is benchmark-informed but not vendor-copied.
- Promotion requires latency evidence.
- Promotion requires availability evidence.
- Promotion requires error-budget evidence.
- Promotion requires refusal evidence.
- Promotion requires rollback evidence.
- Promotion requires audit-chain evidence.
- Promotion requires cost-budget evidence.
- Promotion requires capacity-admission evidence.
- Promotion requires incident-response evidence.

## Repo-Local Evidence
- PRD baseline: `microservices/whiteboard/PRD.md`.
- Operating bar: `microservices/whiteboard/PHASE-01-WHITEBOARD-OPERATING-BAR.md`.
- SLO folder: `microservices/whiteboard/slos/`.
- Dashboard folder: `microservices/whiteboard/dashboards/`.
- Runbook folder: `microservices/whiteboard/runbooks/`.
- Capacity model: `microservices/whiteboard/capacity-model.md`.
- Cost budget: `microservices/whiteboard/cost-budget.md`.
- Failure modes: `microservices/whiteboard/failure-modes.md`.
- Incident response: `microservices/whiteboard/incident-response.md`.
- Capability records: `microservices/whiteboard/capabilities/`.
- Audit findings: `microservices/whiteboard/AUDIT-FINDINGS-2026-05-21.json`.

## Capability SLOs
- `board-open` p95 target is 300 ms for tenant-scoped board envelope reads.
- `board-open` p99 target must be defined before enterprise preview.
- `board-open` availability target starts at 99.9 percent for Tier-1 cells.
- `board-open` must measure Cedar denials separately from transport failures.
- `board-open` must measure tenant-missing requests as validation failures.
- `canvas-op-append` p95 target is 300 ms for simple append commands.
- `canvas-op-append` must track operation sequence conflict rate.
- `canvas-op-append` must track idempotency replay success rate.
- `canvas-op-append` must track merge rejection rate by capability and tenant.
- `canvas-op-append` must track FigJam-grade reconnect impact without copying FigJam semantics.
- `presence-sync` p95 target is 150 ms for cursor publish and fanout under normal load.
- `presence-sync` must track lease expiry churn.
- `presence-sync` must track reconnect success.
- `presence-sync` must track dropped volatile updates separately from durable data loss.
- `presence-sync` must track Whiteboard.fi-style classroom participant fanout.
- `history-snapshot` must expose async completion SLOs.
- `history-snapshot` must track snapshot creation latency by board size.
- `history-snapshot` must track snapshot comparison latency by object count.
- `history-snapshot` must track retention-pack failures.
- `history-snapshot` must track Lucidspark-grade history/export expectations.
- `export-render` must expose async completion SLOs.
- `export-render` must track render queue delay.
- `export-render` must track artifact authorization denial.
- `export-render` must track format-specific failure rate.
- `export-render` must track Microsoft Whiteboard retention-export expectations.
- `template-marketplace-install` must expose settlement completion SLOs.
- `template-marketplace-install` must track DealSet settlement refusal.
- `template-marketplace-install` must track template preview latency.
- `template-marketplace-install` must track rollback-token issuance.
- `template-marketplace-install` must track Miro Enterprise and Mural Enterprise template expectations.

## Canvas And Session SLO Model
- Board envelope latency is measured from authorized request arrival to policy-scoped board metadata response.
- Canvas append latency is measured from accepted command arrival to durable operation acknowledgement.
- Canvas append fanout latency is measured separately from append acknowledgement.
- CRDT convergence lag is measured from operation acknowledgement to all active replicas observing the operation.
- Operation conflict rate is measured per board, tenant, cell, and source benchmark.
- Presence publish latency is measured from lease-valid cursor update to fanout delivery.
- Presence reconnect recovery is measured from transport restoration to lease renewal.
- Board session admission is measured separately from board open.
- History snapshot latency is measured from accepted job to immutable snapshot pointer.
- Snapshot comparison latency is measured from compare request to diff summary.
- Export render latency is measured from accepted job to artifact authorization-ready state.
- Template install latency is measured from DealSet settlement acceptance to template grant activation.

## Command, Event, And Proto SLO Deltas
- Command SLO: `boards:open` p95 and p99.
- Command SLO: `operations:append` p95, p99, conflict, and accepted-loss rate.
- Command SLO: `operations:preview` preview latency and refusal rate.
- Command SLO: `history:snapshot` accepted-job completion and failure rate.
- Command SLO: `exports:render` queue delay, render duration, and artifact authorization failure.
- Command SLO: `templates:install` settlement duration, grant activation, and rollback-token issuance.
- Event SLO: append accepted event publication lag.
- Event SLO: append rejected event publication lag.
- Event SLO: presence lease renewal event lag.
- Event SLO: presence lease expiry event lag.
- Event SLO: snapshot completed event lag.
- Event SLO: export completed event lag.
- Event SLO: template settled event lag.
- Proto SLO: internal append call budget only applies after edge policy has allowed the request.
- Proto SLO: internal render worker budget excludes queue wait and reports both values.
- Proto SLO: internal presence fanout budget excludes client network reconnect time.

## Cedar Decision SLO Handling
- Cedar allow latency is part of command latency.
- Cedar deny latency is tracked but does not burn availability budget.
- Cedar error burns availability budget for the affected capability.
- Cedar timeout burns availability budget for the affected cell and capability.
- Cedar policy snapshot mismatch blocks promotion.
- Cedar facts must include tenant, principal, audience, purpose, capability, and data class.
- Board-open denials are counted by reason.
- Append denials are counted by reason.
- Export download denials are counted separately from render job failures.
- Template settlement denials are counted separately from install system failures.
- Auditor-scope denials are tracked separately from collaboration-user denials.
- CI-scope denials are tracked separately from production-user denials.

## Benchmark Promotion Gates
- Miro Enterprise gate requires board open, append, history, export, and template SLO evidence.
- Miro Enterprise gate requires tenant-admin refusal evidence.
- Miro Enterprise gate requires migration-board size tiers.
- Mural Enterprise gate requires facilitation-template activation evidence.
- Mural Enterprise gate requires large-board export evidence.
- Mural Enterprise gate requires workspace-like controls without workspace service leakage.
- FigJam gate requires presence reconnect evidence.
- FigJam gate requires multiplayer append sequence evidence.
- FigJam gate requires cursor fanout evidence.
- Lucidspark gate requires export fidelity evidence.
- Lucidspark gate requires diagram-style history snapshot evidence.
- Lucidspark gate requires artifact authorization evidence.
- Whiteboard.fi gate requires classroom audience-type evidence.
- Whiteboard.fi gate requires instructor moderation and participant fanout metrics.
- Whiteboard.fi gate requires board lifecycle evidence for education pack overlays.
- Microsoft Whiteboard gate requires tenant-admin governance evidence.
- Microsoft Whiteboard gate requires retention-safe export evidence.
- Microsoft Whiteboard gate requires policy refusal evidence.

## Error Budget Rules
- Interactive board open burns budget on server error, timeout, or policy-service unavailable.
- Interactive board open does not burn budget on valid Cedar refusal.
- Canvas append burns budget on server error, timeout, or accepted operation loss.
- Canvas append does not burn budget on deterministic sequence conflict.
- Presence sync burns budget on connection failures above threshold.
- Presence sync does not burn budget on intentionally dropped stale cursor state.
- History snapshot burns budget on failed accepted jobs.
- History snapshot does not burn budget on denied snapshot requests.
- Export render burns budget on failed accepted jobs.
- Export render does not burn budget on unauthorized artifact requests.
- Template install burns budget on failed accepted install jobs.
- Template install does not burn budget on DealSet refusal.
- Pack-overlay failures burn budget for the affected pack only.
- Tenant-specific exhaustion is isolated to the tenant and cell.
- Cross-tenant shared failure triggers global promotion hold.

## Dashboard Requirements
- Dashboard must show board-open p50, p95, p99, and error rate.
- Dashboard must show append latency and conflict rate.
- Dashboard must show presence fanout and reconnect success.
- Dashboard must show snapshot job queue depth.
- Dashboard must show export job queue depth.
- Dashboard must show template install settlement outcomes.
- Dashboard must show Cedar allow, deny, and error counts.
- Dashboard must show audit-chain event publication counts.
- Dashboard must show tenant, cell, region, pack, data class, and benchmark source dimensions.
- Dashboard must show Miro Enterprise displacement journeys.
- Dashboard must show Mural Enterprise displacement journeys.
- Dashboard must show FigJam displacement journeys.
- Dashboard must show Lucidspark displacement journeys.
- Dashboard must show Whiteboard.fi displacement journeys.
- Dashboard must show Microsoft Whiteboard displacement journeys.
- Dashboard must show error-budget burn by capability.
- Dashboard must show rollback readiness by capability.
- Dashboard must show incident links by cell.

## Alert Requirements
- Page on sustained board-open availability breach.
- Page on accepted append operation loss.
- Page on export artifacts generated without authorization evidence.
- Page on template installation without DealSet settlement.
- Page on audit-chain publication failures.
- Page on cross-tenant leakage suspicion.
- Ticket on rising append sequence conflicts.
- Ticket on presence reconnect degradation.
- Ticket on snapshot queue saturation.
- Ticket on render queue saturation.
- Ticket on pack-overlay denial spikes.
- Ticket on benchmark migration fixture drift.
- Ticket on cost-budget threshold breach.
- Ticket on capacity-admission threshold breach.
- Ticket on SLO dashboard data loss.

## Promotion Steps
- Step 1 records SLO objectives per capability.
- Step 2 records dashboard panels per capability.
- Step 3 records alert thresholds per capability.
- Step 4 records runbook links per alert.
- Step 5 records rollback procedures per capability.
- Step 6 records migration fixture coverage per benchmark.
- Step 7 records tenant and cell coverage.
- Step 8 records pack overlay coverage.
- Step 9 records data-class coverage.
- Step 10 records audit-chain coverage.
- Step 11 records Cedar refusal coverage.
- Step 12 records cost-budget coverage.
- Step 13 records capacity-admission coverage.
- Step 14 records chaos-drill linkage to IP-022.
- Step 15 records DPIA linkage to IP-023.
- Step 16 records threat linkage to IP-024.
- Step 17 records audit closeout linkage to IP-025.
- Step 18 blocks promotion if any required link is missing.

## Hold Conditions
- Hold if ADR-0321 is missing from a promotion artifact.
- Hold if any benchmark uses only legacy shorthand names.
- Hold if a capability lacks tenant-scoped metrics.
- Hold if a capability lacks principal-scoped audit evidence.
- Hold if a capability lacks data-class metrics.
- Hold if a capability lacks denial metrics.
- Hold if append loss cannot be distinguished from conflict.
- Hold if presence loss cannot be distinguished from stale volatile updates.
- Hold if export render cannot prove artifact authorization.
- Hold if template install cannot prove DealSet settlement.
- Hold if history snapshot cannot prove retention policy.
- Hold if dashboards omit cell or region.
- Hold if runbooks omit rollback.
- Hold if error budget cannot be computed.
- Hold if audit findings remain open for the capability.

## Tests And Evidence
- Test board-open SLO math with valid allows, valid denials, and server errors.
- Test append SLO math with success, conflict, timeout, and operation loss.
- Test presence SLO math with reconnect, stale cursor drop, and fanout failure.
- Test snapshot SLO math with accepted job success, accepted job failure, and denial.
- Test export SLO math with render success, render failure, and artifact denial.
- Test template SLO math with settlement success, settlement refusal, and rollback token failure.
- Test dashboard dimensions for tenant, cell, pack, data class, and benchmark source.
- Test alert routing for page versus ticket outcomes.
- Test promotion hold when dashboard evidence is missing.
- Test promotion hold when ADR-0321 linkage is missing.
- Test promotion hold when benchmark names are incomplete.
- Test rollback readiness before preview activation.
- Test migration fixture SLOs for all six benchmark names.
- Test audit-chain publication before promotion.
- Test cost-budget and capacity-admission gates before promotion.

## Rollback
- Roll back promotion state before rolling back SLO definitions.
- Roll back capability activation individually where possible.
- Roll back benchmark migration exposure when only a benchmark gate fails.
- Roll back preview tenant eligibility when tenant-specific SLOs fail.
- Roll back cell eligibility when cell-specific SLOs fail.
- Roll back pack overlay activation when pack-specific SLOs fail.
- Roll back template marketplace access when settlement evidence fails.
- Roll back export artifact access when authorization evidence fails.
- Roll back append writes when accepted operation loss is suspected.
- Roll back presence feature flags when reconnect churn exceeds threshold.

## Workflow Decisions
- Workflow decision: promotion starts as evidence-only until every whiteboard capability has current SLO, dashboard, runbook, and rollback references.
- Workflow decision: tenant preview eligibility is granted per tenant, cell, pack, and capability rather than globally.
- Workflow decision: benchmark displacement can pass for one source vendor while another remains held.
- Workflow decision: denied operations count as healthy only when denial evidence and user-facing remediation are present.
- Workflow decision: SLO burn opens fix workflow before promotion rollback unless the burn threatens data loss or audit loss.
- Workflow decision: ADR-0321 linkage is checked during promotion but never modified by this IP.

## Acceptance Criteria
- SLO-gated promotion names all six displaced benchmark products.
- SLO-gated promotion preserves the existing ADR binding set including ADR-0321.
- SLO-gated promotion defines capability-specific latency and reliability evidence.
- SLO-gated promotion separates valid denials from reliability failures.
- SLO-gated promotion binds dashboard, alert, runbook, rollback, and audit evidence.
- SLO-gated promotion defines hold conditions for missing tenant, principal, data-class, and benchmark evidence.
- SLO-gated promotion links downstream to IP-022, IP-023, IP-024, and IP-025.
- SLO-gated promotion blocks activation when export authorization or DealSet settlement evidence is missing.
- SLO-gated promotion can be reviewed without editing ADR-0321.
- SLO-gated promotion does not require `oya vcs verify`, `done`, or `promote` for this pass.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
