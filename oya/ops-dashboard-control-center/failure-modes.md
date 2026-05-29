# Ops Dashboard / Control Center Failure Modes

| Failure mode | Detection | Safe response |
|---|---|---|
| Command API cannot seal audit event | operator-action-audit-completeness SLO drops below 1.0 | fail closed; do not record success |
| Idempotency store conflict | 409 response and conflict metric | return previous accepted result only when body hash matches |
| Cluster health signal stale | cluster-health-freshness SLO breach | mark health unknown; block approval automation |
| Tenant posture evidence missing | posture response lacks evidence refs | return warn/fail posture; require follow-up evidence export |
| Evidence export unavailable | evidence-pack-freshness SLO breach | return export ticket failure; keep incident open |
| Cedar policy unavailable | authorization error rate spike | fail closed except ticketed break-glass path |

## Acceptance criteria

- Failure modes never silently convert into pass states.
- Operator UI must show unknown separately from green.
- Every failure branch emits audit or diagnostic evidence.
