# Dashboard — Bot Score Distribution

**Owner:** axis-network + ops-security.
**Source:** `dashboards/bot-score-distribution.json`.

## Purpose

Bot-management visibility. Diagnose bot-storms. Track CAPTCHA pass rate.

## SLO bindings

- Custom advisory metric: `oya_api_gateway_bot_score_false_positive_ratio` < 0.01.

## Runbooks

- `runbooks/bot-storm.md`
- `runbooks/ddos-mitigation.md`

## References

- `policy/abuse-defence.cedar`
- ADR-0297 (in flight)
- documentation-rigor.md §3.2.3
