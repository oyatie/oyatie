---
doc_class: Runbook
doc_id: RUNBOOK-LEARNING_MANAGEMENT-HOT_SPLIT
microservice: learning-management
status: wave-15-zf-scaffold
date: 2026-05-21
owner_team: axis-learning-management + council-product
bounded_context: sharding-automation-operations
implementation_phase: doctrine-propagation-only
rust_code_status: not-authored-in-this-wave
source_adrs:
  - ADR-0346
  - ADR-0347
  - ADR-0348
  - ADR-0349
---

# Runbook: Learning Management Hot Split

## Operator Contract
- Runbook id: learning-management-hot-split.
- Primary service namespace: `learning-management`.
- Scenario: a shard is hot and must split within its current cell without changing tenant residency.
- Automation event class: `autosharding.dynamic_sharding.hot_split`.
- Owning team: axis-learning-management + council-product.
- Audience: on-call engineer, SRE incident commander, and governance reviewer.
- Required authority: Cedar permit for every tenant movement, shard mutation, or cross-jurisdiction candidate.
- Stop condition: metrics are green for 30 minutes, audit-chain rows are sealed, rollback metadata is preserved, and ADR citation validation remains green.
- Safety invariant: this is doctrine propagation only; implementation remains sequenced after ADR acceptance.
- Safety invariant: never bypass residency, compliance pack, cosign, JCasC, GitOps, or audit-chain controls to speed up mitigation.
- Safety invariant: prefer refusal with evidence over a partially observed automation event.

## Doctrine Anchors
- ADR-0346 provenance note: legacy local verifier wording is retained only as historical/local-feedback context; ADR-0515 makes the branch-protected `oya-ci-required` context the live CI authority.
- ADR-0346 enforced_by lanes: `oya-governance-oya-verify-ci-mirror-coverage`; `oya-governance-oya-verify-ci-step-exit-semantics`; `oya-governance-oya-verify-skip-flag-allowlist`; `oya-governance-oya-submit-calls-verify`; `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 purpose wording: every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request.
- ADR-0347 enforced_by lanes: `oya-governance-no-foundry-fitness-residue`; `oya-governance-lane-prefix-vocabulary`; `oya-governance-rename-inventory-presence`.
- ADR-0348 purpose wording: cellular topology MUST support three control-plane-driven automation modes underneath the cell-level promotion gates already doctrined in ADR-0341.
- ADR-0348 auto-rebalance wording: when cell load skews beyond promotion-gate criteria, the cell-orchestrator automatically migrates tenants from hot cells to cooler cells.
- ADR-0348 dynamic-sharding wording: shard count within a cell adjusts based on load: HOT-SPLIT when shard p99 latency exceeds SLO OR capacity utilization exceeds 80%; COLD-MERGE when adjacent shards both run below 20% utilization for more than 24 hours.
- ADR-0348 enforced_by lanes: `oya-governance-sharding-automation-coverage`; `oya-governance-autosharding-manual-mode-refusal`; `oya-governance-auto-rebalance-residency-honored`; `oya-governance-dynamic-sharding-threshold-coverage`; `oya-governance-audit-chain-emit-on-automation-events`; `oya-governance-tenant-migration-reversibility`.
- ADR-0349 purpose wording: ADR-0349 Jenkins CI wording is historical/provenance after ADR-0515; ArgoCD remains separately authorized CD evidence where applicable.
- ADR-0349 enforced_by lanes: `oya-governance-jenkins-github-actions-parity`; `oya-governance-argocd-application-cosign-verified`; `oya-governance-argocd-tenant-namespace-isolation`; `oya-governance-jenkins-jcasc-only`; `oya-governance-deploy-audit-chain-emit`.

## Trigger Conditions
- Trigger 1: shard p99 latency exceeds the declared SLO threshold.
- Trigger 2: shard utilization exceeds the declared hot_split_utilization_threshold_percent.
- Trigger 3: the target split preserves tenant-scoped routing and audit-chain continuity.
- Trigger 4: `oya_sharding_hot_split_threshold_breach_total` crosses the declared threshold for two evaluator windows.
- Trigger 5: `oya_sharding_hot_split_duration_seconds_p99` threatens the service SLO budget or promotion-gate quiet window.
- Trigger 6: governance reports missing sharding automation coverage for this service.
- Trigger 7: missing branch-protected `oya-ci-required` evidence blocks the release train for the sharding automation lane
- Trigger 8: ArgoCD reports a pending sync tied to this service after a sharding automation manifest change.

## Preflight Checklist
1. Set incident context: `export INCIDENT_ID=INC-learning-management-hot-split-$(date -u +%Y%m%dT%H%M%SZ); export SERVICE=learning-management; export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Verify service deployment: `kubectl -n learning-management rollout status deploy/learning-management --timeout=60s`.
3. Verify alerts: `oya observability alerts list --service learning-management --runbook hot-split --since 30m`.
4. Verify primary metric: `oya metrics query oya_sharding_hot_split_threshold_breach_total --service learning-management --cell $CELL --window 30m`.
5. Verify secondary metric: `oya metrics query oya_sharding_hot_split_duration_seconds_p99 --service learning-management --cell $CELL --window 30m`.
6. Verify Cedar decision path: `oya cedar eval --principal ops.sre.oncall --action sharding_automation.execute --resource service:$SERVICE --tenant $TENANT`.
7. Verify residency and compliance pack filters before any candidate target is accepted.
8. Verify audit-chain availability: `oya audit-chain health --cell $CELL --tenant $TENANT`.
9. Verify ArgoCD sync health: `argocd app get $SERVICE --refresh`.
10. Verify branch-protected `oya-ci-required` evidence from GitHub Actions/branch protection before declaring the runbook complete.

## Decision Tree
1. If Cedar denies the operation, stop the automation and attach the decision id to the incident.
2. If residency or compliance pack filters remove every candidate, refuse the automation and page compliance secondary.
3. If audit-chain emit is unhealthy, freeze the operation before state mutation.
4. If only observability is stale, refresh telemetry once and compare against the last sealed audit-chain event.
5. If GitOps sync is pending, pause execution until ArgoCD confirms the service declaration is current.
6. If branch-protected `oya-ci-required` evidence is unavailable, keep the change in report-only state; legacy `oya verify --ci-required` output may be attached only as optional local-feedback/provenance.
7. If all gates pass, continue with the smallest reversible cohort.
8. If the first cohort fails validation, roll back from the audit-chain pointer and do not expand blast radius.

## Procedure
1. Read the four canonical threshold fields before taking action; do not default-fill missing values.
2. Freeze writes for the smallest tenant cohort needed to create the split boundary.
3. Create child shard assignments atomically and keep routing dual-read until validation passes.
4. Seal the hot-split audit-chain row before removing the parent shard from the routing table.
5. Generate the execution plan: `oya cell-rebalancer dynamic-sharding hot-split plan --service $SERVICE --cell $CELL --tenant $TENANT --incident $INCIDENT_ID --dry-run --output json`.
6. Review the plan for tenant count, shard count, source cell, target cell, residency result, compliance result, and rollback pointer.
7. Execute only after two-person incident authorization: `oya cell-rebalancer dynamic-sharding hot-split execute --service $SERVICE --cell $CELL --tenant $TENANT --incident $INCIDENT_ID --confirm`.
8. Watch the first cohort until p99 latency, error budget burn, routing convergence, and audit-chain emit all return green.
9. Keep ArgoCD in sync-only mode; do not run manual `kubectl apply` or manual Helm CLI deploys.
10. Preserve the generated evidence bundle under the incident id and attach it to the governance review.

## Evidence Requirements
- Evidence 1: audit-chain event `autosharding.dynamic_sharding.hot_split.planned` with `service`, `cell`, `tenant`, `incident_id`, `cedar_decision_id`, and `rollback_pointer`.
- Evidence 2: audit-chain event `autosharding.dynamic_sharding.hot_split.executed` with source and target placement or shard epoch identifiers.
- Evidence 3: audit-chain event `autosharding.dynamic_sharding.hot_split.validated` with metric snapshots for `oya_sharding_hot_split_threshold_breach_total` and `oya_sharding_hot_split_duration_seconds_p99`.
- Evidence 4: Cedar permit or denial id for every state-mutating step.
- Evidence 5: residency and compliance pack candidate filter output.
- Evidence 6: ArgoCD Application sync id and cosign verification policy result.
- Evidence 7: GitHub Actions run/status URL proving branch-protected `oya-ci-required` acceptance for this service.
- Evidence 8: optional legacy `oya verify --ci-required` local-feedback result, or N/A with rationale, never merge authority.
- Evidence 9: governance lane names from ADR-0347, ADR-0348, and ADR-0349 included in the incident handoff.
- Evidence 10: rollback rehearsal output proving reversibility from the audit-chain trail.

## Rollback Path
1. Freeze further cohorts: `oya flags set sharding_automation.hot-split.hold=true --service $SERVICE --cell $CELL --reason $INCIDENT_ID`.
2. Restore routing from the last sealed audit-chain rollback pointer.
3. Execute rollback: `oya cell-rebalancer dynamic-sharding hot-split rollback --service $SERVICE --cell $CELL --tenant $TENANT --incident $INCIDENT_ID --from-audit-chain`.
4. Verify source and target epochs match the preflight snapshot.
5. Re-run Cedar evaluation to prove the rollback did not introduce cross-tenant or cross-jurisdiction access.
6. Re-run ArgoCD refresh and confirm no manual drift remains.
7. Keep the incident open until the rollback validation window has held for 30 minutes.

## Validation And Closure
1. Confirm all trigger metrics are back under threshold for 30 minutes.
2. Confirm no audit-chain emit gaps exist for the incident window.
3. Confirm Cedar decisions are sealed and tied to the incident id.
4. Confirm `oya-governance-auto-rebalance-residency-honored` or `oya-governance-dynamic-sharding-threshold-coverage` evidence is attached as applicable.
5. Confirm `oya-governance-audit-chain-emit-on-automation-events` evidence is attached for every automation event.
6. Confirm branch-protected `oya-ci-required` evidence is attached per ADR-0515; Jenkins/Prow wording is provenance only.
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
