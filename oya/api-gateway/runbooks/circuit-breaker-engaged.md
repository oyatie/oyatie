# Runbook — Circuit breaker engaged

**Authority:** ADR-0157 + Hystrix-pattern circuit-breaker doctrine.
**Owner:** axis-network.
**Trigger SEV:** SEV-1 (multiple upstreams) / SEV-2 (single upstream).
**Last reviewed:** 2026-05-20.

## A — Trigger conditions

- `oya_api_gateway_upstream_circuit_open_total` > 0 for >2min.
- Per-upstream 5xx rate > 5% sustained 1min → circuit trips (5 consecutive 5xx).
- Customer reporting 502/503 on specific routes.

## B — Pre-checks

1. Identify affected upstream µservice + cell:
   - `oya_api_gateway_upstream_circuit_open_total by (upstream, cell)`.
2. Check upstream µservice health directly:
   - `kubectl get pods -n <upstream> -l app=<upstream> --field-selector=status.phase=Running`.
3. Check upstream SLO from its own observability:
   - `microservices/<upstream>/dashboards/<svc>-overview.json`.

## C — Procedure

1. **Engage upstream on-call** if not already:
   - PagerDuty: `<upstream>-oncall`.
2. **Verify gateway side:**
   - mTLS handshake metric: `oya_api_gateway_upstream_mtls_handshake_failed_total`.
   - Circuit half-open success rate: `oya_api_gateway_upstream_circuit_half_open_success_total`.
3. **If gateway-side issue** (e.g. SPIFFE bundle stale):
   - Restart SPIFFE bundle loader: `kubectl rollout restart deployment/spire-bundle-loader -n spire`.
4. **If upstream-side issue:**
   - Wait for upstream recovery; circuit auto-closes after 3 consecutive 2xx.
   - OR force-close circuit (after upstream confirms fix): `oyatie-circuit --upstream <svc> --close --reason "<incident-id>"`.
5. **Customer comms** if SEV-1.
6. **Monitor recovery:**
   - `oya_api_gateway_upstream_circuit_state{upstream=<svc>}` → `closed`.

## D — Verification

- Circuit state: `closed`.
- 5xx rate < 0.1% on the route.
- Customer-facing SLO `edge-availability` recovered.

## E — Rollback

N/A — circuit-breaker is the rollback mechanism. If circuit is misbehaving (false positives), investigate per-upstream 5xx threshold tuning.

## F — Post-incident

1. Postmortem if SEV-1 or multi-tenant impact.
2. Cross-µservice postmortem with the affected upstream's owner.
3. If circuit-tuning misfired, adjust threshold per `iac/envoy-config.yaml`.

## G — References

- ADR-0157
- `iac/envoy-config.yaml` (circuit-breaker block)
- Hystrix circuit-breaker pattern brief
- Google SRE Workbook ch. 18 (Graceful Degradation)
