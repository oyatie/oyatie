---
runbook: webhook-replay-attack-detected
microservice: connector
owner_team: axis-integration + ops-security
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0263, ADR-0297]
doc_status: published
---

# Runbook — Webhook Replay Attack Detected

## A. Trigger conditions

- `oya_connector_webhook_replay_blocked_total > N` (configurable; default N=10 in 5min)
- Anomaly detection: same idempotency-key seen ≥3× within 1min
- Manual report from tenant: "I see duplicate workflow runs"

## B. Pre-checks

1. Distinguish replay attack from misconfigured vendor:
   - Replay attack: HMAC-valid payloads + identical timestamps from rotating IPs
   - Vendor bug: HMAC-valid payloads with monotonically increasing timestamps but same idempotency-keys (vendor retry loop)
2. Confirm with vendor support channel if vendor bug suspected.

## C. Procedure

1. **Confirm replay pattern** (≤2min)
   ```bash
   # Recent replay blocks
   kubectl logs -n connect deploy/connect-webhook-receiver-edge --tail=1000 | grep WebhookReplayBlocked

   # Source IPs
   curl 'http://connector-webhook-receiver-edge.connector:9090/metrics' | \
     grep 'oya_connector_webhook_replay_blocked_total'
   ```

2. **Adaptive challenge engaged** (already active per `abuse-defence.cedar`)
   - Bot-mgmt should auto-escalate source IPs to bot_score > 95.
   - Edge WAF blocks subsequent requests.
   - UX-floor: legitimate vendor traffic from known JA4 fingerprints unaffected.

3. **Tenant-side investigation** (≤10min)
   - Identify affected wiring(s) via audit chain `WebhookReplayBlocked` events.
   - Notify wiring owner via in-app + email (per ADR-0273).
   - Suggest signing-secret rotation if vendor-side compromise suspected.

4. **Rotate webhook signing secret** (if compromise suspected)
   ```bash
   # Mark current secret as rotated; broker accepts both old + new for 5min grace
   curl -X POST http://connector-webhook-receiver-edge.connector:8080/admin/rotate-secret \
     -H "Authorization: Bearer <step-up-token>" \
     -d '{"wiring_id": "<id>"}'
   ```
   - Tenant pastes new secret into vendor admin (e.g., Shopify webhook config).
   - After 5min grace, old secret rejected.

5. **Source-IP block** (if attack persistent; ≤2min)
   ```bash
   # Add to edge WAF deny list
   kubectl patch configmap edge-waf-rules -n connect --type merge \
     -p '{"data":{"deny_ips":"<ip1>,<ip2>,..."}}'
   ```

## D. Verification

```bash
# Replay-block rate trending down
curl http://connector-webhook-receiver-edge.connector:9090/metrics | grep oya_connector_webhook_replay_blocked_total

# Tenant traffic unaffected (UX-floor)
curl http://connector-webhook-receiver-edge.connector:9090/metrics | grep 'oya_connector_webhook_receive_p99_seconds{outcome="2xx"}'
# Expected: p99 ≤ 0.1s
```

## E. Rollback

If WAF block caused false positive (legitimate vendor IPs blocked):
- Remove from deny list.
- Apologize to tenant; verify their integration recovers.

## F. Post-incident

- Forensics: full payload digests of replayed events (raw payloads expire per 7d DLQ retention; only digests in audit chain long-term).
- Vendor coordination: if vendor-side breach, coordinate broader rotation.
- Update detection thresholds in `policy/abuse-defence.cedar` if needed (≥60s soak per ADR-0294).

## G. References

- ADR-0263 audit-event emission
- ADR-0297 abuse-defence baseline
- `gateway/connector/policy/abuse-defence.cedar`
- `gateway/connector/policy/webhook-receiver-gating.cedar`
- documentation-rigor.md §3.2.3 UX-floor invariants
