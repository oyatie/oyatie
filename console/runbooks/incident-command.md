# Ops Control Center Runbook — Incident Command

## Trigger

Use this runbook for SEV1-SEV4 incidents declared through `/ops/v1/incidents` or the matching gRPC command.

## Steps

1. Confirm operator identity, tenant scope, and MFA state.
2. Declare the incident with severity, title, summary, and idempotency key.
3. Check cluster health and tenant isolation posture before approving remediation.
4. Record each remediation decision with rationale and audit seal.
5. Export a signed evidence pack after mitigation.

## Acceptance criteria

- Every decision has actor identity, rationale, and audit seal.
- No raw cross-tenant data is pasted into the incident record.
- Any break-glass use cites a ticket and follow-up owner.
