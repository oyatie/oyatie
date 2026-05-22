# Dashboard — Rate-Limit Hits

**Owner:** axis-network.
**Source:** `dashboards/rate-limit-hits.json`.

## Purpose

Diagnose 429 storms; per-tenant rate-limit utilisation; Valkey hot key detection.

## Use cases

- SEV-2 rate-limit-saturation incident response.
- Per-tenant FinOps cost ceiling tracking.
- DDoS pattern detection (anomalous 429 ratio).

## SLO bindings

Tied to: `slos/edge-availability.openslo.yaml`.

## Runbook bindings

- `runbooks/rate-limit-saturation.md` — primary runbook.
- `runbooks/ddos-mitigation.md` — escalation if pattern is DDoS.

## References

- `policy/rate-limit.cedar`
- ADR-0244
