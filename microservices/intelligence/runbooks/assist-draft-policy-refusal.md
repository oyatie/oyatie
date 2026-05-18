# Assist Draft Policy Refusal Runbook

## Trigger

Assist-draft or retrieval requests begin failing policy checks above the design threshold.

## Checks

1. Confirm consent grant, tenant budget, and context scope are present.
2. Compare refusal events against `oya.intelligence.policy.refused`.
3. Verify retrieval citations remain tenant-scoped.
4. Route product-impact review to axis-intelligence without changing policy in incident mode.
