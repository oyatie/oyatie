# Runbook — Cell evacuation

**Authority:** ADR-0248 (cellular) + ADR-0252 (HLC time coordination).
**Owner:** axis-network + ops-platform.
**Trigger SEV:** SEV-1 (multi-cell evac) / SEV-2 (single cell).
**Last reviewed:** 2026-05-20.

## A — Trigger conditions

- Cell K8s API outage > 5min.
- Cell network partition.
- Per-cell SLO `edge-availability` < 0.99 for >10min.
- Scheduled DR drill.

## B — Pre-checks

1. Confirm cell affected:
   - `kubectl --context cell-<id> get nodes`.
2. Confirm sister cells healthy:
   - `kubectl --context cell-<sister-id> get nodes`.
3. Confirm sov-cell-compliance: if affected cell is sov-cell-kr/cn, DR target MUST stay within jurisdiction.
4. Confirm DNS NS1 health-check API reachable.

## C — Procedure

1. **Initiate cell de-pool** from Anycast:
   - `oyatie-cell-depool --cell <cell-id> --reason "<incident-id>"`.
   - Effect: NS1 health-check fails for the cell; BGP withdraw within 60s.
   - Audit: `oya.api_gateway.cell.depooled`.
2. **Wait 60s** for in-flight requests to drain (per cell connection-idle timeout).
3. **Verify zero new connections to the cell:**
   - `kubectl --context cell-<id> get svc -n api-gateway` → no new endpoints.
4. **Failover tenants to sister cells:**
   - Per `multi-region.md`:
     - Non-sov tenants: failover cross-region.
     - Sov-cell-kr: failover to ap-seoul-2/3 (within KR).
     - Sov-cell-cn: failover to cn-shanghai-2 (within PRC).
5. **Customer comms:**
   - `oyatie-status post --severity sev-1 --message "Cell <id> evacuated; traffic shifted to sister cells"`.
6. **Repair the affected cell** (operations team).
7. **Re-pool** after smoke-test:
   - `oyatie-cell-pool --cell <cell-id>`.
   - Re-balance per shuffle-shard.

## D — Verification

- Affected cell receives 0 requests for >5min.
- Sister cells absorb load with SLO recovery.
- Customer-facing latency within +20% of baseline.
- No sov-cell-jurisdiction violation (verify Cedar fragment `sov-cloud-overlay.cedar`).

## E — Rollback (re-pool)

After repair:

1. Verify cell healthy: pods running, mTLS handshake test passing.
2. Re-pool with gradual weight: `oyatie-cell-pool --cell <cell-id> --weight 10%`.
3. Increase weight over 30min: 10% → 50% → 100%.
4. Audit: `oya.api_gateway.cell.repooled`.

## F — Post-incident

1. **Postmortem within 5 business days.**
2. **Cross-region DR drill** if not scheduled within last 90d → schedule.
3. **Sov-cell-jurisdiction audit** if any cross-jurisdiction routing detected.

## G — References

- ADR-0248
- `multi-region.md`
- `iac/k8s-deployment.yaml`
- `observability/runbooks/canary-graduation.md`
