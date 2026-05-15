---
doc_class: PhasePlan
parent: ./INDEX.md
id: M-CC-P00-plan
title: Oya VCS approved ralplan v5 fold-in
status: approved-folded
source_plan: ../../../../ralplan-gitops-vcs-replacement-20260514.md
source_spec: ../../../../../specs/gitops-vcs-replacement.json
purpose: Move the grit VCS replacement earlier in the master plan and upgrade it into an agent-native, provider-neutral GitOps control plane.
---
# Oya VCS phase charter

## Mission

Move the VCS replacement for grit earlier in the master plan and upgrade it from local merge serialization into an agent-native, provider-neutral VCS/GitOps control plane.

## Core thesis

Queue semantic agent-authored ChangeSets, not provider PRs. Use AST/parser-backed symbols and knowledge-graph dependency closure to avoid merge conflicts, avoid wasted work, keep high-throughput agents moving, and rebuild only the crates/packages/deployables whose dependency closure changed.

## Universal decomposition rule

Milestones own outcomes. Phases own delivery gates. ImplementationPlans are the ChangeSet-sized units that agents claim, implement, verify, bundle, and promote. A plan that cannot be claimed, verified, and promoted independently must be split before execution.

## Deliberate pre-mortem

| Scenario | Automated fix | Escalation |
|---|---|---|
| Grit authority drift | Disable projection writer; reconcile from grit watch/status; quarantine affected bundles. | Any repo transition bypasses grit. |
| Lock overreach | Split ChangeSet; emit rescheduling FixupTask; downgrade lock scope. | Lock scope exceeds graph-proven affected closure. |
| Stale lock/dead agent | Mark stale, request grit-authoritative recovery, requeue dependents. | Stale recovery lacks grit evidence. |
| Wrong cold-build closure | Invalidate closure cache; rebuild expanded closure; emit evidence gap. | Closure freshness cannot be verified. |
| Provider backpressure | Degrade to cached issue state; retry adapter; keep native state moving. | Issue freshness exceeds promotion SLA. |
| Bundle/artifact mismatch | Quarantine bundle; regenerate from controller state; rerun package/deploy evidence. | Coverage and artifact evidence mismatch persists. |
| ops surface lies | Mark degraded; publish blocker/incident state; freeze promotion lane. | Missing/stale health evidence. |
| Provider lock-in | Move leaked field behind port; add native adapter fixture. | Adapter-specific core type reaches kernel. |
