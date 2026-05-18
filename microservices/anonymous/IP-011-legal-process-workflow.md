---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-shared-substrate
phase: P02-anonymous-foundation
impl_plan_id: IP-011-legal-process-workflow
status: pending
execution_unit: ChangeSet
owner: ops-security + general-counsel + axis-anonymous
acceptance_lanes: [cargo-check, cargo-test, oya-governance-dual-control-conformance]
---

# IP-011: Legal-process workflow (workflow-engine cross-link, dual-control, audit-chain)

## Intent

Author the full legal-process-disclosure BC vertical. The disclosure workflow is multi-step (intake → counsel review → dual-control approval → 14-day notice OR court-prohibited gag-order → key-ceremony → disclosure execute → audit-chain seal → transparency-report inclusion) and is implemented via `workflow-engine` orchestration with Cedar-gated steps per `policy/legal-process-disclosure.cedar`.

## Acceptance

- Dual-control approval requires 2 distinct principals from distinct organisational units (Cedar enforced)
- Chain-of-custody hash unbroken across all 7 workflow steps
- Audit-chain seal on every state transition
- Dry-run executes the full flow in dev cluster with throwaway keys
- Transparency-report aggregator populated correctly with court-prohibited flag
- Per `runbooks/legal-process-court-order-receipt.md` paths A-F implementable
