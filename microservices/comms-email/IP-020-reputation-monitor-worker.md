---
ip_id: IP-020
microservice: comms-email
bounded_context: reputation-monitoring
layer: worker
related_adrs: [ADR-0201, ADR-0255, ADR-0263]
---

# IP-020 — reputation monitor worker

## Goal

Compute tenant reputation from delivery, bounce, complaint, authentication, and external
postmaster signals, then enforce the service's marketing circuit breaker. This worker is
the control loop behind the policy and dashboard already present in comms-email.

## Service anchors

- Dashboard: `microservices/comms-email/dashboards/reputation-monitoring.json` contains
  panels for tenants below 70, Gmail Postmaster score, SNDS score, and circuit breaker state.
- Policy: `microservices/comms-email/policy/abuse-defence.cedar` forbids
  `Action::"send_marketing"` when `resource.tenant.reputation_score < 70`.
- Runbook: `microservices/comms-email/runbooks/reputation-drop-circuit-breaker-engaged.md`
  is the operator path after the worker engages the breaker.
- Deliverability dashboard counterpart:
  `microservices/comms-email/dashboards/deliverability.json`.

## Worker behavior

1. Pull internal delivery/bounce/complaint rates from IP-008/IP-021 event outputs.
2. Pull authentication failures from IP-016/IP-017 aggregate inbound and outbound signals.
3. Ingest external signals for Gmail Postmaster, Microsoft SNDS, Sender Score, and Talos
   where tenant credentials exist.
4. Normalize to a 0..100 tenant reputation score.
5. Set breaker state when score drops below 70 and emit `oya.comms-email.reputation-drop`.

## Counterpart refs

- IP-021 supplies bounce classification and storm data.
- IP-018 receives list hygiene feedback when poor list quality drives reputation drop.
- IP-025 exposes read-only reputation REST/dashboard access.
- IP-013 can use reputation state as an input to provider routing decisions.

## Acceptance

- Score below 70 causes the existing Cedar marketing-send forbid to evaluate true.
- Dashboard metric names remain aligned with `dashboards/reputation-monitoring.json`.
- Runbook trigger and alert name remain consistent with the dashboard.
- Transactional send is not blocked by this worker unless another policy gate denies it.
