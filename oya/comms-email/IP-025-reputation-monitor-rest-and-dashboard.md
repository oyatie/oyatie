---
ip_id: IP-025
microservice: comms-email
bounded_context: reputation-monitoring
layer: rest
related_adrs: [ADR-0253, ADR-0263, ADR-0201]
---

# IP-025 — reputation monitor REST and dashboard

## Goal

Expose tenant-safe reputation score, source breakdown, anomalies, and circuit-breaker state
to tenant admins and operators. This IP binds the read API to the dashboard and policy
already present in the service.

## Service anchors

- Dashboard: `microservices/comms-email/dashboards/reputation-monitoring.json`.
- Worker counterpart: `microservices/comms-email/IP-020-reputation-monitor-worker.md`.
- Policy: `microservices/comms-email/policy/abuse-defence.cedar` marketing circuit breaker
  when reputation score drops below 70.
- Authorization: `microservices/comms-email/policy/action-authorization.cedar` permits
  `Action::"view_deliverability"` for tenant admins and attested service principals.
- Runbook: `microservices/comms-email/runbooks/reputation-drop-circuit-breaker-engaged.md`.

## Contract delta

Extend `microservices/comms-email/contracts/openapi.yaml` with read-only endpoints for:

- Current tenant reputation score.
- Source contribution summary for Gmail Postmaster, SNDS, Sender Score, Talos, bounce rate,
  complaint rate, and authentication failures.
- Recent anomalies and current circuit-breaker state.

All endpoints are tenant-scoped and must not expose another tenant's reputation data.

## Counterpart refs

- IP-020 computes score and breaker state.
- IP-021 supplies bounce and complaint classification.
- IP-018 receives list-hygiene remediation feedback.
- IP-013 may consider reputation when selecting provider pool.

## Acceptance

- Dashboard panel metric names stay aligned with the API response fields or documented
  transforms.
- `Action::"view_deliverability"` is the authorization action for tenant-admin reads.
- The API clearly separates transactional send status from marketing circuit-breaker state.
- Current OpenAPI absence is resolved before implementation is claimed.
