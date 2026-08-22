# Runbook — Bot storm

**Authority:** ADR-0297 (in flight) + documentation-rigor.md §3.2.3.
**Owner:** axis-network + ops-security.
**Trigger SEV:** SEV-1 (regional) / SEV-2 (per-tenant).
**Last reviewed:** 2026-05-20.

## A — Trigger conditions

- `api_gateway_bot_score_bucket{le="80"}` ratio < 0.7 for >5min (i.e. >30% of traffic is bot-scored ≥80).
- `api_gateway_captcha_challenge_issued_total` rate > 10k/s sustained.
- `api_gateway_honeypot_hits_total` rate spike.
- Customer report: "I'm being challenged with CAPTCHA on every request."

## B — Pre-checks

1. Distinguish bot-storm from legitimate-crawler spike:
   - Verify friendly-crawler allow-list traffic: `oyatie-bot --crawler-stats`.
   - Reverse-DNS + forward-DNS check on Googlebot/Bingbot.
2. Check bot-score model health:
   - Wasm filter crashes: `kubectl get pods -n api-gateway -l app=envoy --field-selector status.phase=CrashLoopBackOff` (should be empty).
   - Model confidence histogram: `api_gateway_bot_score_confidence_bucket`.
3. Check ASN concentration:
   - `api_gateway_requests_total{bot_score=~"9.."}` by ASN; top-3 ASNs.

## C — Procedure

1. **Identify pattern:**
   - Volumetric (low-cost bot farm) → engage rate-limit + IP-blocklist.
   - Sophisticated (residential-proxy, AI-CAPTCHA-solver) → engage adaptive challenge + device attestation requirement.
   - Targeted (single tenant) → engage tenant-specific override.
2. **Raise bot-score threshold globally** (temporarily):
   - Edit `policy/abuse-defence.cedar`: forbid `bot_score > 85` (down from 95).
   - Push fragment + soak ≥60s per ADR-0294.
   - Effect: aggressive deny on bot-scored traffic.
3. **Enable CAPTCHA-on-suspicion at lower threshold:**
   - `oyatie-captcha-config --threshold 60 --duration 1h`.
4. **Block known-bad ASNs:**
   - `oyatie-asn-block --asn 12345 --duration 24h`.
   - Audit: `oya.api_gateway.asn.blocked`.
5. **Coordinate with Cloudflare:**
   - `curl -X PATCH https://api.cloudflare.com/client/v4/zones/$ZONE/settings/bot_fight_mode -H "Authorization: Bearer $CF_API_TOKEN" -d '{"value":"on"}'`.
6. **Honeypot activation:**
   - Activate honeypot routes for the attacker tenant; canary payload tracker.
   - Audit: `oya.api_gateway.honeypot.activated`.
7. **Monitor false-positive rate:**
   - `api_gateway_bot_score_false_positive_ratio` — if > 0.05, lower threshold and engage human review.
8. **Status page** if SEV-1.
9. **De-escalate** when bot-score distribution returns to baseline + 1σ for ≥30min.

## D — Verification

- Bot-score distribution returns to baseline (≥70% of requests at score ≤80).
- CAPTCHA issue rate < 1k/s.
- Customer reports of false CAPTCHA cease.
- Honeypot canary payloads not appearing in attacker scrapes.

## E — Rollback

If bot-score threshold lowering caused legitimate-user lockout:

1. Revert Cedar fragment: re-push original `abuse-defence.cedar`.
2. Revert CAPTCHA threshold: `oyatie-captcha-config --threshold 90`.
3. Verify SLO `bot-score-false-positive-rate` < 0.01.

## F — Post-incident

1. Postmortem if SEV-1.
2. Update bot-score model with attack-pattern features (per model-card update workflow).
3. Update `policy/abuse-defence.cedar` with new permanent forbid rules if patterns are persistent.
4. Update `iac/edge-waf.yaml` with new WAF signatures.

## G — References

- `policy/abuse-defence.cedar`
- `dashboards/bot-score-distribution.json`
- `microservices/api-gateway/threat-model.md` §C-D
- `microservices/api-gateway/runbooks/ddos-mitigation.md`
- ADR-0297 (in flight)
