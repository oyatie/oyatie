---
runbook: connector-attestation-revoked
microservice: connector
owner_team: axis-integration + council-security
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0243, ADR-0247, ADR-0249, ADR-0263, ADR-0293, ADR-0294, ADR-0295, ADR-0296]
companion_docs:
  - microservices/connector/threat-model.md
  - microservices/connector/policy/connector-catalog-publishing.cedar
  - microservices/connector/runbooks/connector-cascade-failure.md
  - microservices/connector/runbooks/signature-verification-cascade-failure.md
doc_status: published
---

# Runbook — Connector Attestation Revoked

## A. Trigger conditions

- PagerDuty alert `ConnectorAttestationRevoked` fires
- Audit event `ConnectorAttestationRevoked` in ADR-0263 registry
- Security team revokes MPO (Marketplace Publisher Operator) signing key
- cosign keyless OIDC SBOM signature fails validation against Sigstore TUF root
- `oya_connector_catalog_attestation_verify_fail_total{connector="<X>"} > 0`
- Connector's publisher namespace is suspended (abuse, ToS violation, breach)
- Vendor discloses supply-chain compromise of their connector SDK

## B. Pre-checks

1. **Identify the connector and revocation class** (≤2min)
   ```bash
   kubectl exec -n connect deploy/connect-catalog-api -- \
     catalog-admin attestation-status --connector="${CONNECTOR_NAME}"
   # Returns: attestation_state, revoked_at, revocation_reason, publisher_namespace
   ```
2. **Determine blast radius** (≤5min)
   ```bash
   # How many active tenant wirings use this connector?
   kubectl exec -n connect deploy/connect-catalog-api -- \
     catalog-admin wiring-count --connector="${CONNECTOR_NAME}" --status=active
   # Check active OAuth grants
   kubectl exec -n connect deploy/connect-oauth-broker -- \
     query-grants --connector="${CONNECTOR_NAME}" --status=active | jq length
   ```
3. **Is this a supply-chain compromise?**
   - If YES: escalate immediately to security incident; follow `docs/runbooks/security-incident-response.md`.
   - If NO (key rotation, ToS issue): follow standard revocation below.
4. **Check dependency graph** — does any other connector depend on this SDK?
   ```bash
   kubectl exec -n connect deploy/connect-catalog-api -- \
     catalog-admin sdk-dependency-graph --sdk="${CONNECTOR_SDK_VERSION}"
   ```

## C. Procedure

1. **Disable the connector in catalog** (≤3min)
   ```bash
   kubectl exec -n connect deploy/connect-catalog-api -- \
     catalog-admin disable --connector="${CONNECTOR_NAME}" \
       --reason="attestation-revoked" \
       --revocation-ref="${REVOCATION_EVENT_ID}"
   ```
   Cedar gate `connector-catalog-publishing.cedar` will now deny `catalog:read` for the disabled connector.
   Audit: `ConnectorDisabledPostRevocation` event emitted.

2. **Force-open circuit breaker** (≤3min)
   ```bash
   kubectl patch configmap connect-adapter-config -n connect --type merge \
     -p "{\"data\":{\"${CONNECTOR_NAME}_circuit_force_open\":\"true\",\"${CONNECTOR_NAME}_attestation_revoked\":\"true\"}}"
   kubectl rollout restart deployment/connect-adapter-worker -n connect
   ```
   In-flight actions fail fast; affected actions route to DLQ.

3. **Quarantine DLQ entries** (≤5min)
   ```bash
   kubectl exec -n connect deploy/connect-dlq-worker -- \
     dlq-quarantine --connector="${CONNECTOR_NAME}" \
       --reason="attestation-revoked-$(date +%Y%m%d)"
   ```

4. **Revoke OAuth grants** (≤10min)
   - Only revoke if revocation reason is supply-chain compromise or credential leak.
   - For ToS violation: soft-disable grants (tenant can re-auth when re-attested).
   ```bash
   if [[ "${REVOCATION_REASON}" == "supply-chain-compromise" ]]; then
     kubectl exec -n connect deploy/connect-oauth-broker -- \
       revoke-all-grants --connector="${CONNECTOR_NAME}" --force
   else
     kubectl exec -n connect deploy/connect-oauth-broker -- \
       soft-disable-grants --connector="${CONNECTOR_NAME}"
   fi
   ```
   Audit: `OAuthGrantRevoked` or `OAuthGrantSoftDisabled` per tenant.

5. **Notify affected tenants** (≤15min)
   ```bash
   curl -X POST http://ops-dashboard.internal/v1/incidents \
     -d "{
       \"severity\": \"sev2\",
       \"title\": \"${CONNECTOR_NAME} connector temporarily unavailable\",
       \"reason\": \"security review in progress\",
       \"eta\": \"${REATTESTED_ETA}\"
     }"
   ```
   If supply-chain compromise: severity sev1; engage council-security.

6. **Publisher remediation path** (≤48h SLA)
   - Contact MPO publisher via `marketplace-publisher-support@<platform_owner>`.
   - MPO must: patch the compromised component + rebuild + re-sign + re-submit for security review.
   - Re-attestation runs via `catalog:attest` Cedar action.

7. **Re-enable after re-attestation** (only after security review passes)
   ```bash
   # Verify new attestation
   kubectl exec -n connect deploy/connect-catalog-api -- \
     catalog-admin attestation-status --connector="${CONNECTOR_NAME}"
   # Confirm attestation_state == "attested"

   # Re-enable
   kubectl exec -n connect deploy/connect-catalog-api -- \
     catalog-admin enable --connector="${CONNECTOR_NAME}"
   kubectl patch configmap connect-adapter-config -n connect --type merge \
     -p "{\"data\":{\"${CONNECTOR_NAME}_circuit_force_open\":\"false\",\"${CONNECTOR_NAME}_attestation_revoked\":\"false\"}}"
   kubectl rollout restart deployment/connect-adapter-worker -n connect
   ```

8. **Replay DLQ** (after re-enable)
   ```bash
   kubectl exec -n connect deploy/connect-dlq-worker -- \
     dlq-replay --connector="${CONNECTOR_NAME}" \
       --quarantine-ref="attestation-revoked-$(date +%Y%m%d)"
   ```

## D. Verification

```bash
# Attestation status verified
kubectl exec -n connect deploy/connect-catalog-api -- \
  catalog-admin attestation-status --connector="${CONNECTOR_NAME}"
# Expected: attestation_state=attested, revoked_at=null

# Circuit breaker closed
curl -s http://connector-adapter-worker.connector:9090/metrics \
  | grep "oya_connector_circuit_breaker_open_total{connector=\"${CONNECTOR_NAME}\"}"
# Expected: 0

# No new attestation-failure events
kubectl exec -n connect deploy/connect-audit-reader -- \
  query-audit-events --event-class ConnectorAttestationRevoked \
    --connector="${CONNECTOR_NAME}" --since 5m | jq length
# Expected: 0
```

## E. Rollback

- If re-enable causes attestation-verify failures: force-open circuit again (step 2).
- If DLQ replay causes side-effects: pause replay, quarantine affected entries.
  ```bash
  kubectl exec -n connect deploy/connect-dlq-worker -- dlq-replay-pause --connector="${CONNECTOR_NAME}"
  ```

## F. Post-incident

- Security post-mortem within 5 business days for supply-chain class.
- Update `catalog/connectors/${CONNECTOR_NAME}.yaml` with `attestation_history` entry.
- File advisory in `AUDIT-FINDINGS-${DATE}.json` per ADR-0263 evidence format.
- Review all other connectors from the same publisher for potential exposure.
- Harden: require SBOM + cosign attestation at build time in connector CI lane.

## G. References

- ADR-0243 Cedar universal gate
- ADR-0247 self-modification doctrine (meta-trust-root)
- ADR-0249 multi-category marketplace
- ADR-0293 meta-trust-root attestation path
- ADR-0294 Cedar fragment soak window
- ADR-0295 bootstrap CI SPIFFE + kill-switch
- `microservices/connector/policy/connector-catalog-publishing.cedar`
- `microservices/connector/runbooks/connector-cascade-failure.md`
- `docs/runbooks/security-incident-response.md`
