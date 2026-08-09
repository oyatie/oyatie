# Ops Control Center Runbook — Deployment Approval and Rollback

## Trigger

Use this runbook when a deployment awaits approval, hold, rejection, or rollback execution.

## Steps

1. Review deployment health, error-budget state, and freeze-window reference.
2. Approve, hold, or reject the deployment with rationale.
3. If rollback is needed, record a distinct rollback decision with target revision.
4. Confirm GitOps controller owns downstream mutation; the dashboard records decision state only.
5. Verify audit-chain seal and evidence-pack export receipt.

## Acceptance criteria

- Rollback is not implied by failed deployment approval; it has its own decision.
- Each command is idempotent and tied to an operator identity.
- GitOps remains the mutation path.
