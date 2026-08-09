# Ops Control Center Runbook — KR Localization Escalation Hook

## Trigger

Use this runbook when a Korea localization pack workflow needs an operational escalation, evidence-pack export, or region-specific incident handoff.

## Steps

1. Keep canonical-base behavior unchanged; use pack-specific evidence refs.
2. Confirm KR corpus lock and pack evidence refs before escalation.
3. Route operator communications through the incident command workflow.
4. Export signed evidence scoped to the authorized tenant, window, and pack.

## Acceptance criteria

- KR escalation remains an operational hook, not canonical-base logic.
- Evidence paths are pack-specific and audit-chain sealed.
- Localization handoff does not bypass tenant isolation or Cedar policy.
