---
runbook: signature-verification-cascade-failure
microservice: connector
owner_team: axis-integration + ops-security
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0263, ADR-0297]
doc_status: published
---

# Runbook — Signature Verification Cascade Failure

## A. Trigger conditions

- `oya_connector_webhook_signature_verify_fail_total > threshold` (default 100/min)
- Suspected: signing secret leaked, vendor rotated keys, or attack
- Tenant reports vendor webhooks all returning 401

## B. Pre-checks

1. Confirm scope: single tenant, single connector, or platform-wide?
2. Check if vendor recently published a key-rotation announcement.
3. Check audit chain for `SignatureSecretRotationGrace` events.

## C. Procedure

1. **Identify scope** (≤2min)
   ```promql
   topk(10, sum by (tenant_id, connector) (rate(oya_connector_webhook_signature_verify_fail_total[5m])))
   ```

2. **Verify vendor signing-key version** (≤5min)
   - Check vendor docs for current key/secret.
   - If vendor rotated, update per-tenant signing-secret via OpenBao.

3. **Per-tenant secret rotation with grace** (≤5min)
   ```bash
   # Mark current secret as "previous"; accept both for 5min
   curl -X POST http://connector-webhook-receiver-edge.connector:8080/admin/rotate-secret \
     -H "Authorization: Bearer <step-up-token>" \
     -d '{"wiring_id": "<id>", "grace_seconds": 300}'
   ```

4. **Attack scenario** (signing secret theft suspected)
   - Per `runbooks/webhook-replay-attack-detected.md`
   - Force rotation; tenant notified
   - Forensics: review recent `WebhookSignatureVerifyFailed` source IPs

5. **Vendor coordination** (if vendor bug)
   - Open vendor support ticket
   - Tenant notified of vendor-side issue per ADR-0273

## D. Verification

```promql
# Verify-fail rate dropping
rate(oya_connector_webhook_signature_verify_fail_total[5m])

# Verify-success rate restored
rate(oya_connector_webhook_signature_verify_success_total[5m])
```

## E. Rollback

If rotation broke legitimate flow: extend grace window; coordinate with tenant.

## F. Post-incident

- If vendor-side rotation announced: update catalog metadata + tenant notifications proactively next time.
- If attack: review WAF rules + bot-mgmt thresholds.

## G. References

- ADR-0263, ADR-0297
- `microservices/connector/policy/payload-signature-verification.cedar`
- `microservices/connector/runbooks/webhook-replay-attack-detected.md`
