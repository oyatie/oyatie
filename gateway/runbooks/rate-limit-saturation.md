# Runbook — Rate-limit saturation

**Authority:** ADR-0157 + ADR-0248 (cellular).
**Owner:** axis-network.
**Trigger SEV:** SEV-2.
**Last reviewed:** 2026-05-20.

## A — Trigger conditions

- `oya_api_gateway_requests_total{code="429"}` rate per cell > 5k/s sustained 5min.
- Per-tenant 429 spike on dashboard `dashboards/rate-limit-hits.json`.
- Customer ticket reporting unexpected 429s.

## B — Pre-checks

1. Confirm tenant tier + expected rate-limit: `kubectl get tenantconfig -n tenancy -o json | jq '.spec.rate_limit'`.
2. Verify Valkey cluster health per cell:
   - `valkey-cli -h <cell-valkey> cluster info` → `cluster_state:ok`.
3. Check for Valkey hot key (single tenant DDoS):
   - `valkey-cli -h <cell-valkey> --hotkeys` — top hot key.
4. Check upstream DDoS dashboard for inbound surge.

## C — Procedure

1. **Identify tenant + route class:**
   - `oyatie-rl-inspect --tenant <id> --window 15m` → bucket history.
2. **Determine cause:**
   - Legitimate traffic spike → consider tenant tier upgrade.
   - DDoS / bot → engage `runbooks/ddos-mitigation.md`.
   - Misconfigured client (retry storm) → contact tenant ops.
   - Saturated Valkey node → re-shuffle-shard.
3. **For legitimate spike:**
   - Temporarily raise tenant cap: `oyatie-rl-override --tenant <id> --rate 2x --duration 1h`.
   - Audit: `oya.api_gateway.rate-limit.override.applied`.
4. **For DDoS / bot:**
   - Engage `runbooks/ddos-mitigation.md`.
5. **For Valkey hot key:**
   - Re-shuffle-shard cell: `oyatie-cell-resharddistribute --cell <cell-id>`.
   - Timing: ≤2min.
   - Audit: `oya.api_gateway.cell.resharddistribute.applied`.
6. **For misconfigured client:**
   - Contact tenant ops; document in ticket; suggest exponential backoff per RFC 9457.

## D — Verification

- `oya_api_gateway_requests_total{code="429", tenant_id=<id>}` rate < 100/s.
- Per-tenant SLO `edge-availability` recovers to ≥0.9995.
- Customer ticket resolved.

## E — Rollback

If rate-limit override was applied and caused upstream µservice degradation:

1. Revoke override: `oyatie-rl-override --tenant <id> --revoke`.
2. Audit: `oya.api_gateway.rate-limit.override.revoked`.

## F — Post-incident

Document in `docs/postmortems/` if SEV-2 or higher. Cross-reference Cedar fragment changes to `policy/rate-limit.cedar`.

## G — References

- `policy/rate-limit.cedar`
- `dashboards/rate-limit-hits.json`
- ADR-0157, ADR-0248
