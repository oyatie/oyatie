---
doc_class: DashboardReference
microservice: connector
dashboard_uid: oya-connector-oauth-token-health
date: 2026-05-20
related_adrs: [ADR-0255, ADR-0263, ADR-0296]
companion_docs:
  - microservices/connector/slos/oauth-token-health.openslo.yaml
  - microservices/connector/runbooks/oauth-token-revocation-cascade.md
  - microservices/connector/policy/oauth-broker-authorization.cedar
doc_status: published
---

# Dashboard Reference — OAuth Token Health

## Purpose

Monitors the health of per-tenant OAuth token lifecycle: grant issuance, refresh, rotation, revocation, and expiry. The token health state directly gates connector availability — a stale or revoked token means the connector-adapter-worker cannot invoke the vendor API.

Hyperscaler precedent: Okta System Log + Auth0 Token Usage Analytics.

## Dashboard location

Grafana UID: `oya-connector-oauth-token-health`
Path: `dashboards/oauth-token-health.json` (Grafana provisioning via `iac/helm/connect/values.yaml`)

## Key panels

| Panel | Metric | Alert threshold |
|---|---|---|
| Token fetch latency P99 | `oya_connector_oauth_token_fetch_duration_seconds` | >500ms → SLO breach |
| Active grants by connector | `oya_connector_oauth_grants_active{connector="..."}` | Drop to 0 for active wiring → alert |
| Expired grants (un-rotated) | `oya_connector_oauth_grants_expired` | >0 for any connector → warn |
| Revocation events (rate) | `oya_connector_oauth_revocation_total` | Spike → `oauth-token-revocation-cascade` runbook |
| Token rotation lag | `oya_connector_oauth_rotation_lag_seconds` | >300s → warn |
| Provider error rate on token endpoint | `oya_connector_oauth_provider_error_total` | >1% → investigate |
| provider-credential BYOK mode breakdown (ADR-0255 §D-4) | `oya_connector_credential_mode{mode="byok|oyatie_shared"}` | oyatie_shared count → deprecation tracker |

## SLO linkage

- `connect-oauth-token-health` SLO (target 99.5%): token fetch succeeds within 500ms.
- Error-budget burn shown on main SLO overview dashboard.

## Failure modes covered

1. **Provider token endpoint outage** — vendor's `/token` endpoint is down. Metric: `oya_connector_oauth_provider_error_total` spikes. Action: circuit-breaker opens; DLQ accumulates.
2. **Refresh token expiry (non-rotating vendor)** — vendor does not support refresh-token-rotation; token silently expired. Metric: `oya_connector_oauth_grants_expired`. Action: tenant prompted to re-auth.
3. **OpenBao latency spike** — sidecar can't fetch refresh token from OpenBao within 60s TTL budget. Metric: `oya_connector_openbao_fetch_latency_p99`. Action: follow `ops-dashboard-control-center` runbook for OpenBao.
4. **provider-credential BYOK rotation gap (ADR-0255 §D-4)** — tenant rotated their OAuth client secret but didn't update SecretReference. Metric: `oya_connector_oauth_credential_mismatch_total`. Action: surface prompt in workflow-studio.

## Capacity math (Little's Law)

At 10,000 active tenants × avg 5 connectors × 1 refresh per 50min:
- Refresh rate = 10,000 × 5 / 3000s = 16.7 refreshes/s
- P99 at 500ms budget → queue depth ≤ 16.7 × 0.5 = 8.3 in-flight at steady state
- 10× headroom: 83 in-flight at peak; provisioned at 200 concurrent refresh goroutines per broker pod

## Cross-references

- `runbooks/oauth-token-revocation-cascade.md` — when revocation events spike
- `policy/oauth-broker-authorization.cedar` — Cedar gate for token operations
- `slos/oauth-token-health.openslo.yaml` — SLO definition
- ADR-0255 §D-4 provider-BYOK credential mode
- ADR-0296 library-first credential sidecar
