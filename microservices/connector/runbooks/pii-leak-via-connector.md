---
runbook: pii-leak-via-connector
microservice: connector
owner_team: axis-integration + council-privacy + ops-sre-reliability
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0244, ADR-0251, ADR-0263, ADR-0276, ADR-0296, ADR-0297]
companion_docs:
  - microservices/connector/threat-model.md
  - microservices/connector/compliance.md
  - microservices/connector/policy/data-residency.md
  - microservices/connector/runbooks/connector-cascade-failure.md
doc_status: published
---

# Runbook — PII Leak via Connector

## A. Trigger conditions

- Detection signal `PIILeakViaConnector` fires from detection substrate (family 8: policy violation)
- Audit event `ConnectorActionInvoked` for a data-class-tagged field shows destination outside tenant's approved data-residency zone
- DLP egress scan alerts: `oya_connector_dlq_pii_egress_blocked_total > 0`
- Tenant-admin reports that personal data appeared in an external system where it should not
- Security researcher responsible-disclosure report references PII exposure in connector payload
- `oya_connector_payload_pii_detected_total{connector="<X>",destination="external"} > 0`

## B. Pre-checks

1. **Identify the connector and payload class** (≤5min)
   ```bash
   # Query audit log for recent ConnectorActionInvoked events with PII-class fields
   kubectl exec -n connect deploy/connect-audit-reader -- \
     query-audit-events \
       --event-class ConnectorActionInvoked \
       --filter 'data_class IN ("PII","PHI","SENSITIVE_FINANCIAL")' \
       --since 1h \
       --format json | jq '.[] | {tenant_id, connector_name, destination_host, data_fields}'
   ```
2. **Determine scope — single tenant vs multi-tenant** (≤5min)
   ```bash
   kubectl exec -n connect deploy/connect-audit-reader -- \
     query-audit-events --event-class ConnectorActionInvoked \
       --filter 'pii_detected=true' --since 24h \
       | jq 'group_by(.tenant_id) | map({tenant: .[0].tenant_id, count: length})'
   ```
3. **Confirm DLP block status** — did the egress scan catch it before transmission?
   ```bash
   curl -s http://connector-adapter-worker.connector:9090/metrics \
     | grep oya_connector_payload_pii_detected_total
   # Check: _blocked vs _transmitted counters
   ```
4. **Assess GDPR/PIPA 72h window** — determine if breach notification is required.
   - If PII was transmitted outside tenant's approved zone: **YES, breach notification likely required**.
   - Engage `council-privacy` immediately.

## C. Procedure

1. **Stop the leaking connector** (≤3min)
   ```bash
   # Force-open circuit breaker for the specific connector
   kubectl patch configmap connect-adapter-config -n connect --type merge \
     -p "{\"data\":{\"${CONNECTOR_NAME}_circuit_force_open\":\"true\"}}"
   kubectl rollout restart deployment/connect-adapter-worker -n connect
   ```
   Audit: `ConnectorCircuitForced` event emitted.

2. **Revoke affected OAuth grants** (≤5min)
   ```bash
   # Revoke all grants for this connector for affected tenants
   for TENANT_ID in "${AFFECTED_TENANTS[@]}"; do
     curl -s -X POST \
       "http://connector-oauth-broker.connector/internal/grants/revoke-all" \
       -H "Authorization: Bearer ${INTERNAL_TOKEN}" \
       -d "{\"tenant_id\":\"${TENANT_ID}\",\"connector\":\"${CONNECTOR_NAME}\"}"
   done
   ```
   Timing budget: ≤5min per batch of 100 tenants.
   Audit: `OAuthGrantRevoked` events emitted per tenant.

3. **Quarantine DLQ entries for the connector** (≤5min)
   ```bash
   kubectl exec -n connect deploy/connect-dlq-worker -- \
     dlq-quarantine --connector="${CONNECTOR_NAME}" \
       --data-class="PII,PHI,SENSITIVE_FINANCIAL" \
       --reason="pii-leak-investigation-$(date +%Y%m%d)"
   ```

4. **Notify affected tenants** (≤15min from confirmation)
   - Via ops-dashboard-control-center incident-declare API:
   ```bash
   curl -X POST http://ops-dashboard.internal/v1/incidents \
     -H "Authorization: Bearer ${OPS_TOKEN}" \
     -d "{
       \"severity\": \"sev1\",
       \"title\": \"PII Leak via ${CONNECTOR_NAME} connector\",
       \"affected_tenants\": ${AFFECTED_TENANTS_JSON},
       \"description\": \"Personal data may have been transmitted to an unauthorized destination. Investigation in progress.\",
       \"remediation_eta_minutes\": 60
     }"
   ```
   Timing: GDPR Art. 33 requires 72h regulator notification; affected user notification per Art. 34 if high-risk.

5. **Engage council-privacy for breach assessment** (≤30min)
   - Share: connector name, data classes, tenant list, volume estimate, destination host, time window.
   - Privacy team determines: was data actually transmitted (not just flagged)?
   - If transmitted: initiate breach-notification workflow per pack (GDPR 72h, PIPA 72h, HIPAA 60d).
   - Pack-specific breach-notification: `kubectl exec -n connect deploy/connect-compliance-runner -- trigger-breach-notification --tenant-id="${TENANT_ID}" --pack="${ACTIVE_PACK}"`

6. **Root-cause analysis** (≤2h)
   - Inspect connector adapter source: was a PII field inadvertently mapped?
   - Check data-mapping configuration: which field mappings include PII-class data?
   - Review connector YAML in `catalog/connectors/${CONNECTOR_NAME}.yaml` for data-class annotations
   - Check DLP egress-scan policy: was the data-class tag missing, causing DLP bypass?
   - Check if the connector vendor's schema changed (schema drift → PII field exposure)

7. **Patch and re-attest** (≤4h)
   - Fix: update connector adapter to exclude PII fields from payload, or add DLP tag
   - Re-run security review (`catalog:attest` action in Cedar policy)
   - Update `catalog/connectors/${CONNECTOR_NAME}.yaml` with `pii_fields_excluded: true`

8. **Re-enable connector** (only after patch + re-attest)
   ```bash
   kubectl patch configmap connect-adapter-config -n connect --type merge \
     -p "{\"data\":{\"${CONNECTOR_NAME}_circuit_force_open\":\"false\"}}"
   kubectl rollout restart deployment/connect-adapter-worker -n connect
   ```

## D. Verification

```bash
# Confirm no more PII egress
curl -s http://connector-adapter-worker.connector:9090/metrics \
  | grep "oya_connector_payload_pii_transmitted_total{connector=\"${CONNECTOR_NAME}\"}"
# Expected: counter should not be incrementing post-patch

# Confirm DLP block active
kubectl exec -n connector deploy/connector-adapter-worker -- \
  curl localhost:9090/metrics | grep oya_connector_dlq_pii_egress_blocked_total

# Confirm OAuth grants revoked
kubectl exec -n connect deploy/connect-oauth-broker -- \
  query-grants --connector="${CONNECTOR_NAME}" --status=active
# Expected: 0 active grants for affected tenants
```

## E. Rollback

- If connector re-enable triggers recurrence: force-open circuit again (step 1).
- If OAuth revocation caused collateral impact: selectively re-grant for unaffected tenants.
- Rollback patch deployment:
  ```bash
  kubectl rollout undo deployment/connect-adapter-worker -n connect
  ```

## F. Post-incident

- Blameless retro within 48h (PII leak is Sev-1; retro cadence accelerated).
- Evidence pack export per ADR-0263 for regulatory evidence.
- Update connector's `data_class_audit` field in `catalog/connectors/${CONNECTOR_NAME}.yaml`.
- Add property-based fuzz test: generate PII-containing payloads → verify DLP blocks all egress.
- Review DLP egress-scan coverage for all connectors with data-mapping enabled.
- If GDPR breach: file regulator notification; update DPIA at `dpia.md`.
- Audit DLQ quarantine entries weekly until all are inspected and cleared.

## G. References

- ADR-0244 §D-2 tenant scoping + data-class taxonomy
- ADR-0251 §pack-overlay compliance-pack breach-notification workflow
- ADR-0263 audit-event emission contract
- ADR-0276 backup portability GDPR Art. 20
- ADR-0296 library-first credential sidecar
- `microservices/connector/policy/data-residency.md`
- `microservices/connector/runbooks/connector-cascade-failure.md`
- `microservices/connector/compliance.md §pack-overlay-roster`
- GDPR Art. 33/34 breach notification obligations
