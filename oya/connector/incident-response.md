---
microservice: connector
doc_class: IncidentResponse
date: 2026-05-20
owner_team: axis-integration + ops-sre-reliability
status: Accepted
related_adrs: [ADR-0263, ADR-0295, ADR-0297]
companion_docs:
  - microservices/connector/runbooks/connector-cascade-failure.md
  - microservices/connector/runbooks/oauth-token-revocation-cascade.md
  - microservices/connector/runbooks/webhook-replay-attack-detected.md
  - microservices/connector/runbooks/connector-rate-limit-saturation.md
  - microservices/connector/runbooks/signature-verification-cascade-failure.md
  - microservices/connector/runbooks/dlq-overflow.md
doc_status: published
---

# Incident Response — connector

## Severity classification

| Sev | Trigger | Response time |
|---|---|---|
| Sev-1 | Webhook receiver fully down platform-wide; OR signing-secret leak suspected | Page within 2min; war room within 10min |
| Sev-2 | Single pack down OR connector-adapter dispatch availability < 99% for 10min | Page within 5min; war room within 30min |
| Sev-3 | Single connector circuit-open OR DLQ growth rate > 10× baseline | Page within 15min |
| Sev-4 | Schema-drift detected on high-traffic wiring | Notify owner; no page |

## On-call

- Primary: axis-integration on-call rotation (PagerDuty: `axis-integration-oncall`).
- Secondary: ops-sre-reliability (PagerDuty: `ops-sre-reliability-oncall`).
- Escalation chain: primary → secondary → axis-integration tech lead → council-architecture.
- Communication: `#connect-incidents` Slack; `#sev1-warroom` for Sev-1.

## Sev-1 playbook

1. **Acknowledge page within 2min.**
2. **Declare incident** in ops-dashboard-control-center via incident-declare capability.
3. **Open war room** (`#sev1-warroom` + Zoom bridge); page secondary.
4. **Initial triage** (≤10min):
   - Check `oya_connector_*` baseline signals on Grafana dashboard `connector-overview`.
   - Check audit chain for recent privileged actions (`OAuthGrantRevoked`, `ProviderCredentialRotated`).
   - Check Cedar deny rate (`oya_cedar_deny_total{ms="connector"}`) for anomalies.
5. **Mitigation options** (in order):
   - Per-route circuit-breaker (Envoy admin); kill specific BC route.
   - Per-tenant rate-limit (Cedar quota override); throttle specific tenant.
   - Kill-switch (per ADR-0295): `kubectl annotate ns connector oya.kill-switch=true` — last resort.
6. **Rollback** if recent deploy correlated: `kubectl rollout undo deployment/connect-<bc> -n connect`.
7. **Post-incident** within 7d: blameless retrospective; evidence pack export per ADR-0263.

## Secret leak playbook (signing secrets, OAuth refresh tokens)

If any per-tenant webhook signing secret or OAuth refresh token is suspected leaked:
1. Rotate the affected secret via OpenBao admin; OpenBao emits revocation events.
2. Force re-OAuth for all grants using affected client (broker schedules `OAuthGrantRotationForced`).
3. Notify affected tenant within 24h per ADR-0273 + GDPR Art. 33 (if EU tenant; 72h max).
4. KR PIPC notification within 24h per PIPA Art. 34 (if KR tenant).
5. Audit chain: export full evidence pack for forensics.

## Vendor breach playbook (vendor-side compromise affects oyatie integrations)

If a vendor (e.g., Salesforce, Stripe) reports a breach impacting their OAuth infrastructure:
1. Mass-revoke all OAuth grants for affected vendor; broker emits bulk `OAuthGrantRevoked`.
2. Pause all wirings using affected vendor.
3. Coordinate with vendor on re-OAuth timeline.
4. Notify affected tenants.
5. Evidence pack: export per ADR-0263 for tenant + auditor consumption.

## Reference runbooks

See `runbooks/` directory.
