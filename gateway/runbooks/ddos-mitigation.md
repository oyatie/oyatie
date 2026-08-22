# Runbook — DDoS mitigation

**Authority:** ADR-0297 (in flight) + `microservices/api-gateway/threat-model.md`.
**Owner:** axis-network + ops-security.
**Trigger SEV:** SEV-1 (regional) / SEV-0 (global).
**Last reviewed:** 2026-05-20.

## A — Trigger conditions

- `api_gateway_ddos_scrub_dropped_bytes` > 10Gbps sustained 60s.
- `api_gateway_requests_total{code="429"}` rate > 50k/s for >2min.
- `api_gateway_tls_handshake_duration_seconds_bucket{le="2"}` p99 > 2s for >1min.
- Out-of-band: PagerDuty alert from upstream DDoS provider (Magic Transit / Akamai Prolexic).

## B — Pre-checks

1. Confirm legitimate traffic baseline from last hour: `sum(rate(api_gateway_requests_total{tenant_id!=""}[5m])) by (cell)`. Expect ≤ 6M req/s globally.
2. Verify BGP-layer scrub is engaged at provider (Cloudflare Magic Transit / AWS Shield Advanced / on-prem Prolexic): `curl https://api.cloudflare.com/client/v4/accounts/$ACCOUNT/magic/router/health -H "Authorization: Bearer $CF_API_TOKEN"` — expect `"status": "scrubbing"`.
3. Identify source ASNs: `api_gateway_requests_total{code="403"}` by `asn`. ASN concentration > 30% indicates targeted attack from specific ASN.
4. Verify rate-limit Valkey cluster health: `valkey-cli -h <cell-valkey> ping` → PONG.
5. Verify circuit breakers not tripped (would suggest upstream degradation, not edge): `api_gateway_upstream_circuit_open_total` should be flat.

## C — Procedure

1. **Declare incident** in `#api-gateway-warroom`:
   - `gh issue create --repo oyatie/incidents --title "DDoS SEV-1 $(date -u +%FT%T.000Z)" --label sev-1,ddos`.
2. **Engage BGP scrub** (if not already): contact upstream provider's NOC; provide attack signature (volume, source ASNs, target prefixes).
   - Timing budget: ≤5min.
   - Audit: `oya.api_gateway.ddos.scrub.engaged`.
3. **Raise WAF mode to "under attack"** via Cloudflare API:
   - `curl -X PATCH https://api.cloudflare.com/client/v4/zones/$ZONE/settings/security_level -H "Authorization: Bearer $CF_API_TOKEN" -d '{"value":"under_attack"}'`.
   - Timing budget: ≤30s.
   - Effect: aggressive bot-score gating + JavaScript challenge on every anonymous request.
   - Audit: `oya.api_gateway.waf.mode.elevated`.
4. **Lower per-IP rate-limit** via Cedar fragment push:
   - Edit `policy/rate-limit.cedar`: `rate_per_minute_per_ip > 100` (down from 1000).
   - Push to policy-engine ledger; soak ≥60s per ADR-0294.
   - Audit: `oya.api_gateway.cedar.permit.matched` for the new fragment.
5. **Activate honeypot routes** if scrape signature detected:
   - `oyatie-honeypot-enable --tenant <attacker-tenant> --duration 1h`.
   - Audit: `oya.api_gateway.honeypot.activated`.
6. **Coordinate with downstream µservices** to engage circuit-breakers if upstream load also rising:
   - `oyatie-circuit --upstream payments --trip 30s`.
7. **Status page update** within 15min:
   - `oyatie-status post --severity sev-1 --message "DDoS mitigation engaged"`.
8. **Monitor SLO recovery:**
   - Watch `dashboards/edge-overview.json`; p99 latency target ≤200ms.
   - Watch `api_gateway_requests_total{code=~"5.."}` rate target ≤0.01 of total.
9. **If sustained >2h or escalating:** escalate to CTO + customer comms team.
10. **De-escalate** when attack volume returns to baseline + 1σ for ≥10min:
    - Revert WAF mode: `curl -X PATCH ... -d '{"value":"medium"}'`.
    - Revert rate-limit fragment: re-push original `rate-limit.cedar`.
    - Disable honeypot.
    - Audit: `oya.api_gateway.waf.mode.normal`.

## D — Verification

- `api_gateway_requests_total{code="429"}` rate < 5k/s.
- `api_gateway_tls_handshake_duration_seconds` p99 < 200ms.
- Customer-facing latency restored to baseline + 10%.
- Status page updated to "monitoring" then "resolved".

## E — Rollback

If WAF mode "under attack" causes excessive legitimate-user friction (CAPTCHA storm on legitimate traffic):

1. Lower WAF mode one tier: `curl -X PATCH ... -d '{"value":"medium"}'`.
2. Restore default rate-limit.
3. Keep BGP scrub engaged at provider (no rollback at network layer until 24h-clean).
4. Audit: `oya.api_gateway.waf.mode.normal`.

## F — Post-incident

1. **Postmortem within 5 business days** per `incident-response.md §G`.
2. **Lessons from attack signature:**
   - New WAF rules → `iac/edge-waf.yaml`.
   - New Cedar permit/forbid → `policy/abuse-defence.cedar`.
   - New bot-score model features → bot-management subsystem.
3. **Cross-µservice review** if attack exploited handoff weakness.
4. **Customer comms** with attack summary + mitigation timeline.

## G — References

- `microservices/api-gateway/threat-model.md` §C-D
- `microservices/api-gateway/failure-modes.md` §A
- `microservices/api-gateway/runbooks/rate-limit-saturation.md`
- `microservices/api-gateway/runbooks/bot-storm.md`
- ADR-0157, ADR-0297 (in flight).
- Cloudflare DDoS Trends Report 2024 H2.
- Akamai State of the Internet — Security Report 2024.
