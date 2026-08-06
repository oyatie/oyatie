---
runbook: connector-cascade-failure
microservice: connector
owner_team: axis-integration + ops-sre-reliability
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0145, ADR-0263]
doc_status: published
---

# Runbook — Connector Cascade Failure

## A. Trigger conditions

- `oya_connector_action_error_rate_5m{connector="<X>"} > 0.5` for 5+ minutes
- `oya_connector_circuit_breaker_open_total{connector="<X>"} > 0`
- PagerDuty alert `ConnectorCascadeFailure` fires
- Tenant reports widespread "vendor error" responses

## B. Pre-checks

1. Confirm scope: `kubectl exec -n connector deploy/connector-adapter-worker -- curl localhost:9090/metrics | grep oya_connector_circuit_breaker_open_total` — which connector(s)?
2. Check vendor status page (e.g., status.salesforce.com, status.stripe.com).
3. Check our egress network: `kubectl exec -n connector deploy/connector-adapter-worker -- nc -zv <vendor-host> 443`.
4. Check audit chain for recent ProviderCredentialRotated events (rotation could correlate).

## C. Procedure

1. **Confirm vendor outage vs oyatie issue** (≤2min)
   - Vendor status page: if RED, this is vendor-side; goto step 4.
   - Vendor status page: if GREEN but our error rate elevated, this is oyatie-side; goto step 2.
   - Audit `oya.connector.adapter-worker` for recent deploy correlation.

2. **Roll back recent deploy if correlated** (≤5min)
   ```bash
   kubectl rollout history deployment/connect-adapter-worker -n connect
   kubectl rollout undo deployment/connect-adapter-worker -n connect
   ```
   Verify error rate drops within 60s. Audit: `ConnectorAdapterRolledBack`.

3. **Per-connector kill-switch** (if rollback insufficient; ≤3min)
   ```bash
   kubectl patch configmap connect-adapter-config -n connect --type merge \
     -p '{"data":{"<connector>_circuit_force_open":"true"}}'
   kubectl rollout restart deployment/connect-adapter-worker -n connect
   ```
   Tenant dashboards show `<connector> unavailable`; failed actions go to DLQ.

4. **Vendor outage path** (vendor-side issue)
   - Verify circuit-breaker is open (`oya_connector_circuit_breaker_open_total{connector="<X>"} > 0`).
   - Surface tenant notification via ops-dashboard-control-center incident-declare capability (severity Sev-3 unless impact is wide).
   - Monitor vendor status; circuit auto-closes after half-open success.

5. **DLQ accumulation** (≤5min)
   - Check DLQ depth: `oya_connector_dlq_depth{connector="<X>"}`.
   - Per ADR-0145 §invariant-1, DLQ accepts overflow without blocking new actions.
   - If approaching tenant retention cap, surface via ops-dashboard.

## D. Verification

```bash
# Error rate trending down
kubectl exec -n observability prometheus-0 -- promtool query instant \
  "oya_connector_action_error_rate_5m{connector='<X>'}"

# Circuit-breaker closed
curl http://connector-adapter-worker.connector:9090/metrics | grep oya_connector_circuit_breaker_open

# DLQ stable or draining
curl http://connector-adapter-worker.connector:9090/metrics | grep oya_connector_dlq_depth
```

Expected: error_rate < 0.05; circuit_breaker_open = 0; DLQ depth flat or decreasing.

## E. Rollback (if mitigation actions made it worse)

```bash
# Re-enable connector
kubectl patch configmap connect-adapter-config -n connect --type merge \
  -p '{"data":{"<connector>_circuit_force_open":"false"}}'
kubectl rollout restart deployment/connect-adapter-worker -n connect

# If rollback caused new issue, redeploy original version
kubectl rollout undo deployment/connect-adapter-worker -n connect
```

## F. Post-incident

- Blameless retro within 7d.
- Evidence-pack export via ops-dashboard-control-center per ADR-0263.
- Update vendor SLA tracking in `microservices/connector/catalog/connectors/<connector>.yaml` if vendor-side.
- If oyatie-side regression: add property test to `oya-connector-adapter-domain` covering the regression.

## G. References

- ADR-0145 inter-microservice communication reform §invariant-1
- ADR-0263 audit-event emission contract
- `microservices/connector/runbooks/dlq-overflow.md`
- `microservices/connector/runbooks/connector-rate-limit-saturation.md`
- Vendor status page links: status.salesforce.com, status.stripe.com, etc.
