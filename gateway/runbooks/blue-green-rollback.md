# Runbook — Blue/Green rollback

**Authority:** ADR-0114 (canary observability rollback) + ADR-0139 (agentic SLO-gated promotion).
**Owner:** axis-network + ops-deployments.
**Trigger SEV:** SEV-1 or SEV-2.
**Last reviewed:** 2026-05-20.

## A — Trigger conditions

- SLO burn-rate `edge-availability` fast-burn-1h > 14× per `microservices/observability/IP-014-automated-rollback-primitive.md`.
- Customer-impacting regression on green; auto-rollback to blue.
- Manual rollback initiated by release captain.

## B — Pre-checks

1. Confirm active routing weight: `oyatie-bg-status --route <route>` → e.g. `green: 100%, blue: 0%`.
2. Confirm blue version still warm (had traffic ≤2h ago): `oyatie-bg-warm-check --version blue`.
3. Confirm blue passing health-checks: `kubectl get pods -n api-gateway -l version=blue --field-selector=status.phase=Running`.
4. Confirm SLO state on blue from pre-rollout snapshot.

## C — Procedure

1. **Initiate rollback** (manual):
   - `oyatie-bg-swap --route <route> --to blue --reason "SEV-1: <description>" --approval-count 2`.
   - Soak ≥30s per `iac/envoy-config-bluegreen.yaml`.
   - Audit: `oya.api_gateway.bluegreen.routed`.
2. **OR Auto-rollback** (triggered by burn-rate):
   - Per IP-014 automated rollback primitive; emits the same swap with `--reason burnrate-auto`.
3. **Verify traffic shift:**
   - `oyatie-bg-status` → `blue: 100%`.
   - `api_gateway_requests_total{version="green"}` rate drops to 0 within 60s.
4. **Monitor recovery:**
   - SLO `edge-availability` should recover within 5min.
   - Customer-facing latency should return to baseline.
5. **Notify release captain + customer comms:**
   - `oyatie-status post --severity sev-1 --message "Rolled back to blue"`.
6. **Drain green:**
   - `kubectl scale deployment api-gateway-green --replicas=0 -n api-gateway`.
   - Keep image + manifests for forensic analysis.

## D — Verification

- 100% traffic on blue.
- SLO `edge-availability` recovers within 5min.
- No 5xx attributable to gateway during rollback window.
- Customer impact ceased.

## E — Re-promote green

After fix:

1. Deploy fixed green version with new image tag.
2. Smoke-test on staging cell.
3. Canary 5% → 25% → 50% → 100% per `runbooks/canary-cohort-weighting.md` (cross-ref observability runbook).
4. Each step gated by SLO burn-rate check per ADR-0139.

## F — Post-incident

1. **Postmortem within 5 business days.**
2. **Catalog regression** in `docs/regressions/`.
3. **CI lane addition** if regression should have been caught earlier.
4. **Customer comms** with attribution + fix-ETA.

## G — References

- ADR-0114, ADR-0139
- `microservices/observability/IP-014-automated-rollback-primitive.md`
- `iac/envoy-config-bluegreen.yaml`
- `microservices/api-gateway/runbooks/circuit-breaker-engaged.md`
