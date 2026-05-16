---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P11-IP-006
title: Wave-B integration — gate promotions on canary; webhook-route IP-004/005/006
status: scaffolded
tier: S
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
source_adrs:
  - ../../../../../../docs/decisions/ADR-0112-webhook-driven-foundry-agent-invocation.md
  - ../../../../../../docs/decisions/ADR-0114-canary-observability-rollback.md
depends_on:
  - M-CC-P11-IP-001
  - M-CC-P11-IP-002
  - M-CC-P11-IP-003
  - M-CC-P11-IP-004
  - M-CC-P11-IP-005
purpose: Wire wave-A substrate into the live pipeline. Promotion workflows gate on canary verdict; IP-004/005/006 from M-CC-P10 are retrofitted to receive events via the IP-003 webhook router instead of `workflow_run:` triggers.
---

# M-CC-P11-IP-006 — Wave-B integration

## Scope

Five wiring edits + 2 new subcommands:

1. **Gate `promote-dev-to-staging.yml`** on canary controller
   verdict. Replace the unconditional `gh api -X PATCH refs/heads/staging`
   with: query canary controller endpoint → only advance if
   verdict ∈ {PROMOTE}. Fail-closed on timeout (treat as
   EXTEND_OBSERVATION, refuse to advance).
2. **Gate `promote-staging-to-production.yml`** similarly.
3. **Retrofit `pr-review.yml`** to receive its trigger via the IP-003
   webhook router instead of `workflow_run:`. The `workflow_run:`
   trigger stays during transition; receiver also fires on the
   same event class for redundancy. After 14 days clean, remove
   `workflow_run:` trigger.
4. **Retrofit `ci-failure-fix-loop.yml`** similarly.
5. **Retrofit IP-006 merge-queue** to consume admission-log events
   from the webhook router instead of polling the filesystem
   tick-loop. IP-006 polls remain enabled as fallback during
   transition.
6. **Add `oya canary rollback --changeset <id>`** subcommand — opens
   a canonical revert PR through dev → staging → production.
7. **Add `oya canary force-rewind --target <ref> --to-sha <sha>
   --justification <text>`** subcommand — short-lived branch-protection
   bypass, signed by human, alarmed.

## Dependencies

ALL five wave-A IPs (IP-001..IP-005) must land first. This is the
convergence wave.

## Acceptance

- `promote-dev-to-staging.yml` includes a `canary_gate` step that
  queries `http://canary-controller/verdict?product=...` and
  refuses to advance on non-PROMOTE.
- Same for `promote-staging-to-production.yml`.
- Synthetic PR exercise: a clean PR opens → CI green → IP-004
  APPROVE → IP-006 admits → merges to dev → canary controller
  emits PROMOTE → staging advances → controller emits PROMOTE →
  production advances. End-to-end webhook trace in evidence.
- Synthetic canary-failure exercise: a PR opens → merges to dev
  → canary controller emits ROLLBACK at stage-1 → staging does
  NOT advance → `oya canary rollback` opens revert PR.
- Synthetic emergency exercise: `oya canary force-rewind` rewinds
  staging from SHA-A to SHA-B with human signature; rewind-log
  records the operation; alarm lane fires if frequency exceeds
  2/30 days.
- `workflow_run:` triggers on `pr-review.yml` + `ci-failure-fix-loop.yml`
  removed after 14 days of redundant operation; redundancy
  recorded in evidence.

## Symbols to grit-claim

- `.github/workflows/promote-dev-to-staging.yml::*` (add canary
  gate step)
- `.github/workflows/promote-staging-to-production.yml::*` (same)
- `.github/workflows/pr-review.yml::*` (router trigger added,
  workflow_run kept for 14d)
- `.github/workflows/ci-failure-fix-loop.yml::*` (same)
- `tools/oya-foundry-vcs-merge-queue-fix-loop-app/src/{webhook_consumer,fallback_poll}.rs::*`
- `crates/oya-dev-cli/src/commands/canary.rs::*` (new module,
  hosts `rollback` + `force-rewind` subcommands)
- `crates/oya-dev-cli/src/commands/mod.rs::run` (register
  `canary` subcommand)

## Exit evidence

- `/evidence/agentic-vcs-pipeline/ip-006-end-to-end-promote-trace.json`
- `/evidence/agentic-vcs-pipeline/ip-006-rollback-revert-pr.json`
- `/evidence/agentic-vcs-pipeline/ip-006-force-rewind-with-human-signature.json`
- `/evidence/agentic-vcs-pipeline/ip-006-workflow-run-retirement-redundancy.json`
