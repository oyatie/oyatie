---
contract: metric-naming-convention
microservice: connector
authored: 2026-05-20
canonical_authority: ADR-0064
related_adrs: [ADR-0064, ADR-0128, ADR-0133, ADR-0263]
status: canonical-base
---

# Metric Naming Convention — connector

Per ADR-0064 canonical-base-and-localization-packs.

## Required metric surface

Every connector component emits via OTel using the prefix `oya_connector_*`. Cardinality budgets per documentation-rigor §1.2 Observability.

### Action invocation

| Metric | Type | Labels | Cardinality |
|---|---|---|---|
| `oya_connector_action_total` | counter | connector, action, status | 500 × 50 × 5 = 125k |
| `oya_connector_action_duration_seconds_bucket` | histogram | connector, action, status, le | 125k × 12 buckets |
| `oya_connector_action_rate_limit_total` | counter | connector, action, outcome | 500 × 50 × 3 = 75k |
| `oya_connector_circuit_breaker_open_total` | gauge | connector | 500 |

### OAuth

| Metric | Type | Labels |
|---|---|---|
| `oya_connector_oauth_grant_total` | counter | connector, outcome |
| `oya_connector_oauth_grant_active` | gauge | connector |
| `oya_connector_oauth_token_refresh_total` | counter | connector, outcome |

### Webhook

| Metric | Type | Labels |
|---|---|---|
| `oya_connector_webhook_receive_total` | counter | connector, verify_outcome |
| `oya_connector_webhook_replay_blocked_total` | counter | connector |
| `oya_connector_webhook_signature_verify_fail_total` | counter | connector, alg |
| `oya_connector_webhook_receive_duration_seconds_bucket` | histogram | connector, le |

### DLQ

| Metric | Type | Labels |
|---|---|---|
| `oya_connector_dlq_depth` | gauge | tenant_id, wiring_id |
| `oya_connector_dlq_replay_total` | counter | wiring_id, outcome |

### Abuse defence

| Metric | Type | Labels |
|---|---|---|
| `oya_connector_abuse_defence_challenge_total` | counter | challenge_class, outcome |
| `oya_connector_abuse_defence_blocked_total` | counter | reason |

## Cardinality discipline

`tenant_id` label only on `oya_connector_dlq_depth` (tenant-scoped operations) — elsewhere routed via OTel resource attribute to avoid metric-cardinality explosion.

## References

- ADR-0064 canonical-base-and-localization
- ADR-0263 audit-event emission contract
- `microservices/observability/contracts/metric-naming-convention.md`
