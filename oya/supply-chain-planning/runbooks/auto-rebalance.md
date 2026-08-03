---
doc_class: Runbook
doc_id: RUNBOOK-SUPPLY_CHAIN_PLANNING-AUTO_REBALANCE
microservice: supply-chain-planning
status: target-provenance-scaffold
date: 2026-05-21
owner_team: axis-supply-chain-planning + axis-erp-parity
bounded_context: sharding-automation-operations
inventory_classification: target-provenance
claim_ceiling: metadata-only-preview-target-no-live-runtime-slo-dr-cloud-or-ga-readiness
visible_repo_path: oya/supply-chain-planning/runbooks/auto-rebalance.md
canonical_prd: specs/microservices/supply-chain-planning.json
operational_claim: none
implementation_phase: doctrine-propagation-only
rust_code_status: not-authored-in-this-wave
source_adrs:
  - ADR-0346
  - ADR-0347
  - ADR-0348
  - ADR-0349
---

# Runbook: Supply Chain Planning Auto Rebalance

## Claim Ceiling
This file is target/provenance inventory for the metadata-only Supply Chain Planning preview. The steps, metric names, command shapes, evidence fields, and closure criteria below describe future operating posture only; they are not live telemetry, runtime automation, incident recovery proof, SLO/DR readiness, cloud deployment evidence, or GA readiness. Treat any legacy local verifier, CI/CD bridge, or Kubernetes/CD wording as provenance unless a later activation card supplies measured evidence.

## Operator Contract
- Runbook id: supply-chain-planning-auto-rebalance.
- Primary service namespace: `supply-chain-planning`.
- Scenario: cell load skew requires tenant movement from a hot cell to a cooler eligible cell.
- Automation event class: `autosharding.auto_rebalance`.
- Owning team: axis-supply-chain-planning + axis-erp-parity.
- Audience: on-call engineer, SRE incident commander, and governance reviewer.
- Required authority: Cedar permit for every tenant movement, shard mutation, or cross-jurisdiction candidate.
- Target stop condition: a future activated service would require measured metrics, sealed audit-chain rows, rollback metadata, and ADR citation validation; this preview runbook supplies none of that evidence.
- Safety invariant: this is doctrine propagation only under the preview PRD claim ceiling; implementation remains sequenced after separate activation evidence.
- Safety invariant: never bypass residency, compliance pack, cosign, JCasC, GitOps, or audit-chain controls to speed up mitigation.
- Safety invariant: prefer refusal with evidence over a partially observed automation event.

## Doctrine Anchors
- ADR-0346 provenance note: legacy local verifier wording is retained only as historical/local-feedback context; ADR-0515 makes the branch-protected `oya-ci-required` context the live CI authority.
- ADR-0346 enforced_by lanes: legacy local-verifier coverage/status lanes retained as provenance only; no merge authority or runtime readiness claim.
- ADR-0347 purpose wording: every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request.
- ADR-0347 enforced_by lanes: `oya-governance-no-foundry-fitness-residue`; `oya-governance-lane-prefix-vocabulary`; `oya-governance-rename-inventory-presence`.
- ADR-0348 purpose wording: cellular topology MUST support three control-plane-driven automation modes underneath the cell-level promotion gates already doctrined in ADR-0341.
- ADR-0348 auto-rebalance wording: when cell load skews beyond promotion-gate criteria, the cell-orchestrator automatically migrates tenants from hot cells to cooler cells.
- ADR-0348 dynamic-sharding wording: shard count within a cell adjusts based on load: HOT-SPLIT when shard p99 latency exceeds SLO OR capacity utilization exceeds 80%; COLD-MERGE when adjacent shards both run below 20% utilization for more than 24 hours.
- ADR-0348 enforced_by lanes: `oya-governance-sharding-automation-coverage`; `oya-governance-autosharding-manual-mode-refusal`; `oya-governance-auto-rebalance-residency-honored`; `oya-governance-dynamic-sharding-threshold-coverage`; `oya-governance-audit-chain-emit-on-automation-events`; `oya-governance-tenant-migration-reversibility`.
- ADR-0349 purpose wording: legacy CI/CD bridge wording is historical/provenance after ADR-0515 and does not establish live deployment readiness for this service.
- ADR-0349 enforced_by lanes: legacy CI/CD bridge parity and CD provenance lanes only; no live service sync, deployment, or audit-chain emission is claimed here.

## Trigger Conditions
- Trigger 1: cell promotion criteria breach due to load skew.
- Trigger 2: candidate cooler cell has tier, residency, compliance pack, and headroom eligibility.
- Trigger 3: tenant migration backlog threatens the service SLO budget.
- Trigger 4: `oya_sharding_auto_rebalance_candidate_count` crosses the declared threshold for two evaluator windows.
- Trigger 5: `oya_sharding_auto_rebalance_migration_seconds_p99` threatens the service SLO budget or promotion-gate quiet window.
- Trigger 6: governance reports missing sharding automation coverage for this service.
- Trigger 7: target branch-protected `oya-ci-required` evidence would block release promotion for a future sharding automation lane.
- Trigger 8: target CD sync evidence would be required after a future sharding automation manifest change.

## Target Preflight Checklist (not live run instructions)
1. Set incident context: `export INCIDENT_ID=INC-supply-chain-planning-auto-rebalance-$(date -u +%Y%m%dT%H%M%SZ); export SERVICE=supply-chain-planning; export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Record service activation evidence only if a separate activation card proves a deployed service; this preview inventory does not authorize live cluster checks.
3. Verify alerts: `oya observability alerts list --service supply-chain-planning --runbook auto-rebalance --since 30m`.
4. Verify primary metric: `oya metrics query oya_sharding_auto_rebalance_candidate_count --service supply-chain-planning --cell $CELL --window 30m`.
5. Verify secondary metric: `oya metrics query oya_sharding_auto_rebalance_migration_seconds_p99 --service supply-chain-planning --cell $CELL --window 30m`.
6. Verify Cedar decision path: `oya cedar eval --principal ops.sre.oncall --action sharding_automation.execute --resource service:$SERVICE --tenant $TENANT`.
7. Verify residency and compliance pack filters before any candidate target is accepted.
8. Verify audit-chain availability: `oya audit-chain health --cell $CELL --tenant $TENANT`.
9. Record CD sync health only if a separate activation card proves an imported application; this preview inventory does not claim CD readiness.
10. Verify branch-protected `oya-ci-required` evidence from GitHub Actions/branch protection before declaring the runbook complete.

## Decision Tree
1. If Cedar denies the operation, stop the automation and attach the decision id to the incident.
2. If residency or compliance pack filters remove every candidate, refuse the automation and page compliance secondary.
3. If audit-chain emit is unhealthy, freeze the operation before state mutation.
4. If only observability is stale, refresh telemetry once and compare against the last sealed audit-chain event.
5. If target CD sync evidence is unavailable, keep the future operation in report-only posture.
6. If branch-protected `oya-ci-required` evidence is unavailable, keep the future change in report-only state; legacy local-verifier output may be attached only as optional local-feedback/provenance.
7. If all future gates pass with measured evidence, continue with the smallest reversible cohort.
8. If the first future cohort fails validation, roll back from the audit-chain pointer and do not expand blast radius.

## Procedure
1. Build the candidate set from capacity_model, ResidencyClass, compliance_pack, cell_placement_class, and shuffle-sharding constraints.
2. Reject any target cell that would cross jurisdiction without an explicit Cedar permit.
3. Stage the migration in dry-run mode and compare source/target tenant assignment epochs before execution.
4. Execute only one bounded tenant cohort at a time and seal the audit-chain row before moving to the next cohort.
5. Generate the execution plan: `oya cell-rebalancer auto-rebalance plan --service $SERVICE --cell $CELL --tenant $TENANT --incident $INCIDENT_ID --dry-run --output json`.
6. Review the plan for tenant count, shard count, source cell, target cell, residency result, compliance result, and rollback pointer.
7. Record the future execution command shape only after two-person incident authorization evidence exists; this file does not authorize execution.
8. Future validation would watch the first cohort until p99 latency, error budget burn, routing convergence, and audit-chain emit all return green.
9. Keep future CD posture sync-only; do not rely on manual cluster or Helm changes as readiness evidence.
10. Preserve any future generated evidence bundle under the incident id and attach it to the governance review.

## Future Evidence Requirements
- Evidence 1: audit-chain event `autosharding.auto_rebalance.planned` with `service`, `cell`, `tenant`, `incident_id`, `cedar_decision_id`, and `rollback_pointer`.
- Evidence 2: audit-chain event `autosharding.auto_rebalance.executed` with source and target placement or shard epoch identifiers.
- Evidence 3: audit-chain event `autosharding.auto_rebalance.validated` with metric snapshots for `oya_sharding_auto_rebalance_candidate_count` and `oya_sharding_auto_rebalance_migration_seconds_p99`.
- Evidence 4: Cedar permit or denial id for every state-mutating step.
- Evidence 5: residency and compliance pack candidate filter output.
- Evidence 6: CD application sync id and cosign verification policy result, only after separate activation evidence exists.
- Evidence 7: GitHub Actions run/status URL proving branch-protected `oya-ci-required` acceptance for this service.
- Evidence 8: optional legacy local-verifier feedback result, or N/A with rationale, never merge authority.
- Evidence 9: governance lane names from ADR-0347, ADR-0348, and ADR-0349 included in the incident handoff.
- Evidence 10: future rollback rehearsal output proving reversibility from the audit-chain trail.

## Target Rollback Path
1. Freeze further cohorts: `oya flags set sharding_automation.auto-rebalance.hold=true --service $SERVICE --cell $CELL --reason $INCIDENT_ID`.
2. Restore routing from the last sealed audit-chain rollback pointer.
3. Record the rollback command shape only after measured activation evidence exists; this preview file does not authorize rollback execution.
4. Verify source and target epochs match the preflight snapshot.
5. Re-run Cedar evaluation to prove the rollback did not introduce cross-tenant or cross-jurisdiction access.
6. Re-run future CD refresh evidence and confirm no manual drift remains.
7. Keep the incident open until the rollback validation window has held for 30 minutes.

## Validation And Closure
1. Future closure would confirm all trigger metrics are back under threshold for 30 minutes.
2. Future closure would confirm no audit-chain emit gaps exist for the incident window.
3. Confirm Cedar decisions are sealed and tied to the incident id.
4. Confirm `oya-governance-auto-rebalance-residency-honored` or `oya-governance-dynamic-sharding-threshold-coverage` evidence is attached as applicable.
5. Confirm `oya-governance-audit-chain-emit-on-automation-events` evidence is attached for every automation event.
6. Confirm branch-protected `oya-ci-required` evidence is attached per ADR-0515; legacy bridge/Prow wording is provenance only.
7. Confirm future CD evidence did not sync unsigned images or cross tenant namespaces.
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
