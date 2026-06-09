---
doc_class: Runbook
doc_id: RUNBOOK-WORKPLACE_INTEGRATION-AUTO_REBALANCE
microservice: workplace-integration
status: wave-15-zf-scaffold
date: 2026-05-21
owner_team: axis-workplace-integration
bounded_context: sharding-automation-operations
implementation_phase: doctrine-propagation-only
rust_code_status: not-authored-in-this-wave
source_adrs:
  - ADR-0346
  - ADR-0347
  - ADR-0348
  - ADR-0349
---

# Runbook: Workplace Integration Auto Rebalance

## Operator Contract
- Runbook id: workplace-integration-auto-rebalance.
- Primary service namespace: `workplace-integration`.
- Scenario: cell load skew requires tenant movement from a hot cell to a cooler eligible cell.
- Automation event class: `autosharding.auto_rebalance`.
- Owning team: axis-workplace-integration.
- Audience: on-call engineer, SRE incident commander, and governance reviewer.
- Required authority: Cedar permit for every tenant movement, shard mutation, or cross-jurisdiction candidate.
- Stop condition: metrics are green for 30 minutes, audit-chain rows are sealed, rollback metadata is preserved, and ADR citation validation remains green.
- Safety invariant: this is doctrine propagation only; implementation remains sequenced after ADR acceptance.
- Safety invariant: never bypass residency, compliance pack, cosign, JCasC, GitOps, or audit-chain controls to speed up mitigation.
- Safety invariant: prefer refusal with evidence over a partially observed automation event.

## Doctrine Anchors
- D-CICD-AUTHORITY purpose wording: branch-protected `oya-ci-required` is the live cloud CI merge authority; local command output is transition evidence only.
- Current CI authority evidence: `D-CICD-AUTHORITY`; `D-CLOUD-NATIVE`; `D-GOVERNANCE-CENTRAL`.
- D-GOVERNANCE-CENTRAL purpose wording: central governance vocabulary supersedes scattered lane naming in active authority surfaces.
- Current governance evidence: `D-GOVERNANCE-CENTRAL` central PaC/CaC/PDP/evidence pipeline acceptance.
- ADR-0348 purpose wording: cellular topology MUST support three control-plane-driven automation modes underneath the cell-level promotion gates already doctrined in ADR-0341.
- ADR-0348 auto-rebalance wording: when cell load skews beyond promotion-gate criteria, the cell-orchestrator automatically migrates tenants from hot cells to cooler cells.
- ADR-0348 dynamic-sharding wording: shard count within a cell adjusts based on load: HOT-SPLIT when shard p99 latency exceeds SLO OR capacity utilization exceeds 80%; COLD-MERGE when adjacent shards both run below 20% utilization for more than 24 hours.
- Current ADR-0348 evidence: central governance acceptance plus branch-protected `oya-ci-required` evidence.
- D-CICD-AUTHORITY cutover wording: owned oya-ci is the same canonical pipeline after cutover, not a parallel authority; delivery substrates remain subordinate to the current SSOT.
- Current delivery/governance evidence: `D-CICD-AUTHORITY`; `D-CLOUD-NATIVE`; `D-GOVERNANCE-CENTRAL`.

## Trigger Conditions
- Trigger 1: cell promotion criteria breach due to load skew.
- Trigger 2: candidate cooler cell has tier, residency, compliance pack, and headroom eligibility.
- Trigger 3: tenant migration backlog threatens the service SLO budget.
- Trigger 4: `oya_sharding_auto_rebalance_candidate_count` crosses the declared threshold for two evaluator windows.
- Trigger 5: `oya_sharding_auto_rebalance_migration_seconds_p99` threatens the service SLO budget or promotion-gate quiet window.
- Trigger 6: governance reports missing sharding automation coverage for this service.
- Trigger 7: branch-protected `oya-ci-required` or owned oya-ci cutover evidence blocks the release train for the sharding automation lane.
- Trigger 8: ArgoCD reports a pending sync tied to this service after a sharding automation manifest change.

## Preflight Checklist
1. Set incident context: `export INCIDENT_ID=INC-workplace-integration-auto-rebalance-$(date -u +%Y%m%dT%H%M%SZ); export SERVICE=workplace-integration; export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Verify service deployment readiness through Kubernetes Deployment status surfaced by the GitOps/observability control plane; manual cluster commands are diagnostic only.
3. Verify alerts through the observability API for service `workplace-integration`, runbook `auto-rebalance`, and the last 30 minutes.
4. Verify primary metric: `oya metrics query oya_sharding_auto_rebalance_candidate_count --service workplace-integration --cell $CELL --window 30m`.
5. Verify secondary metric: `oya metrics query oya_sharding_auto_rebalance_migration_seconds_p99 --service workplace-integration --cell $CELL --window 30m`.
6. Verify the Cedar decision path through the policy-evaluation API for principal `ops.sre.oncall`, action `sharding_automation.execute`, service `workplace-integration`, and the selected tenant cohort.
7. Verify residency and compliance pack filters before any candidate target is accepted.
8. Verify audit-chain availability: `oya audit-chain health --cell $CELL --tenant $TENANT`.
9. Verify ArgoCD sync health: `argocd app get $SERVICE --refresh`.
10. Verify branch-protected `oya-ci-required` acceptance evidence exists for this service before declaring the runbook complete.

## Decision Tree
1. If Cedar denies the operation, stop the automation and attach the decision id to the incident.
2. If residency or compliance pack filters remove every candidate, refuse the automation and page compliance secondary.
3. If audit-chain emit is unhealthy, freeze the operation before state mutation.
4. If only observability is stale, refresh telemetry once and compare against the last sealed audit-chain event.
5. If GitOps sync is pending, pause execution until ArgoCD confirms the service declaration is current.
6. If `oya-ci-required` acceptance evidence is unknown, keep the change in report-only state and do not treat local command output as authority.
7. If all gates pass, continue with the smallest reversible cohort.
8. If the first cohort fails validation, roll back from the audit-chain pointer and do not expand blast radius.

## Procedure
1. Build the candidate set from capacity_model, ResidencyClass, compliance_pack, cell_placement_class, and shuffle-sharding constraints.
2. Reject any target cell that would cross jurisdiction without an explicit Cedar permit.
3. Stage the migration in dry-run mode and compare source/target tenant assignment epochs before execution.
4. Execute only one bounded tenant cohort at a time and seal the audit-chain row before moving to the next cohort.
5. Generate the execution plan: `oya cell-rebalancer auto-rebalance plan --service $SERVICE --cell $CELL --tenant $TENANT --incident $INCIDENT_ID --dry-run --output json`.
6. Review the plan for tenant count, shard count, source cell, target cell, residency result, compliance result, and rollback pointer.
7. Execute only after two-person incident authorization: `oya cell-rebalancer auto-rebalance execute --service $SERVICE --cell $CELL --tenant $TENANT --incident $INCIDENT_ID --confirm`.
8. Watch the first cohort until p99 latency, error budget burn, routing convergence, and audit-chain emit all return green.
9. Keep ArgoCD in sync-only mode; do not run manual `kubectl apply` or manual Helm CLI deploys.
10. Preserve the generated evidence bundle under the incident id and attach it to the governance review.

## Evidence Requirements
- Evidence 1: audit-chain event `autosharding.auto_rebalance.planned` with `service`, `cell`, `tenant`, `incident_id`, `cedar_decision_id`, and `rollback_pointer`.
- Evidence 2: audit-chain event `autosharding.auto_rebalance.executed` with source and target placement or shard epoch identifiers.
- Evidence 3: audit-chain event `autosharding.auto_rebalance.validated` with metric snapshots for `oya_sharding_auto_rebalance_candidate_count` and `oya_sharding_auto_rebalance_migration_seconds_p99`.
- Evidence 4: Cedar permit or denial id for every state-mutating step.
- Evidence 5: residency and compliance pack candidate filter output.
- Evidence 6: ArgoCD Application sync id and cosign verification policy result.
- Evidence 7: branch-protected `oya-ci-required` run id proving the live required context accepted this service surface.
- Evidence 8: local command output, if collected, marked as transition evidence only and not as merge authority.
- Evidence 9: governance lane names from ADR-0347, ADR-0348, and ADR-0349 included in the incident handoff.
- Evidence 10: rollback rehearsal output proving reversibility from the audit-chain trail.

## Rollback Path
1. Freeze further cohorts: `oya flags set sharding_automation.auto-rebalance.hold=true --service $SERVICE --cell $CELL --reason $INCIDENT_ID`.
2. Restore routing from the last sealed audit-chain rollback pointer.
3. Execute rollback: `oya cell-rebalancer auto-rebalance rollback --service $SERVICE --cell $CELL --tenant $TENANT --incident $INCIDENT_ID --from-audit-chain`.
4. Verify source and target epochs match the preflight snapshot.
5. Re-run Cedar evaluation to prove the rollback did not introduce cross-tenant or cross-jurisdiction access.
6. Re-run ArgoCD refresh and confirm no manual drift remains.
7. Keep the incident open until the rollback validation window has held for 30 minutes.

## Validation And Closure
1. Confirm all trigger metrics are back under threshold for 30 minutes.
2. Confirm no audit-chain emit gaps exist for the incident window.
3. Confirm Cedar decisions are sealed and tied to the incident id.
4. Confirm central governance evidence is attached as applicable.
5. Confirm central governance evidence is attached for every automation event.
6. Confirm branch-protected `oya-ci-required` acceptance evidence is attached per D-CICD-AUTHORITY.
7. Confirm ArgoCD did not sync unsigned images and did not cross tenant namespaces.
8. Confirm the post-incident note cites ADR-0346, ADR-0347, ADR-0348, and ADR-0349 by exact ID.
9. Close only after the incident commander records the stop condition and evidence bundle hash.
10. Leave implementation gaps to Wave 15-ZA/ZB/ZD/ZE; do not add code from this runbook lane.

## References
- ADR-0346
- ADR-0347
- ADR-0348
- ADR-0349
- ADR-0263
- ADR-0243
- ADR-0181
- ADR-0254
