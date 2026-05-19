# Ops Dashboard / Control Center Threat Model

## Assets

- Operator identities and roles.
- Deployment approval and rollback decisions.
- Incident records and remediation decisions.
- Tenant isolation posture and evidence refs.
- Audit-chain seal refs and evidence-pack export tickets.

## Threats and controls

| Threat | Control | Evidence |
|---|---|---|
| Cross-tenant operator data leak | Cedar tenant equality rule; posture responses expose evidence refs only | `policy/cedar/operator-actions.cedar`, `tenant-isolation.md` |
| Unauthorized rollback | T3 capability, MFA, approval rationale, idempotency key, audit seal | `capabilities/rollback-execute.yaml` |
| Break-glass misuse | ticket-required forbid rule and evidence-pack export | `policy/cedar/operator-actions.cedar` |
| Stale cluster health signal | health freshness SLO and observed timestamp | `slos/cluster-health-freshness.openslo.yaml` |
| Evidence tampering | audit-chain seal ref, object ref, cosign/keyless algorithm | `manifest.json#compliance` |

## Acceptance criteria

- Every high-impact operator action is T3 and audit-chain sealed.
- Tenant posture is read-only unless a separate approved workflow exists.
- Runtime implementation must add policy fixtures before command execution is enabled.
