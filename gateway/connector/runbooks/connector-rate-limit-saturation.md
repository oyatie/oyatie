---
runbook: connector-rate-limit-saturation
microservice: connector
owner_team: axis-integration + ops-sre-reliability
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0145, ADR-0248, ADR-0263]
doc_status: published
---

# Runbook — Connector Rate-Limit Saturation

## A. Trigger conditions

- `oya_connector_action_rate_limit_total{outcome="hit"} > threshold` for 10+ minutes
- Single tenant consuming >50% of vendor's daily quota
- Cascading 429s causing DLQ growth

## B. Pre-checks

1. Identify scope: per-tenant, per-vendor, or platform-wide?
2. Check shuffle-shard distribution: is one Valkey shard hot?
3. Check vendor's published rate-limit (vary by tenant plan).

## C. Procedure

1. **Identify hotspot** (≤2min)
   ```promql
   topk(10, sum by (tenant_id, connector) (rate(oya_connector_action_rate_limit_total{outcome="hit"}[5m])))
   ```

2. **Per-tenant throttle** (≤5min)
   - Cedar policy: temporary `request_rate_per_minute` override
   - Surface in tenant dashboard: "Your usage of <connector> is approaching limit"

3. **Vendor coordination** (≤30min)
   - For sustained high usage: contact vendor about quota uplift
   - Some vendors (Salesforce, Slack) offer tier-based quota; uplift via tenant's vendor account

4. **DLQ growth response** (≤10min)
   - Per `runbooks/dlq-overflow.md`
   - Mass-retry once vendor quota window resets (typically hourly)

5. **Shuffle-shard rebalance** (if Valkey shard hot)
   ```bash
   # Identify hot shard
   kubectl exec -n valkey valkey-cluster-0 -- valkey-cli --cluster info <node>
   # Migrate slot to less-loaded shard
   kubectl exec -n valkey valkey-cluster-0 -- valkey-cli --cluster reshard <node>
   ```

## D. Verification

```promql
# Rate-limit hit rate trending down
sum(rate(oya_connector_action_rate_limit_total{outcome="hit"}[5m]))

# Shard distribution balanced
sum by (shard) (oya_valkey_keys_total{db="connector"})
```

## E. Rollback

Throttle override is auto-expiring (default 1h); no manual rollback needed. If broader quota was uplifted via vendor and quota was overprovisioned, monitor cost via FinOps.

## F. Post-incident

- Update tenant's recommended-tier in catalog if their usage suggests upgrade.
- If platform-wide pattern: capacity-model.md review + connector-adapter scale-out plan.

## G. References

- ADR-0145 inter-microservice communication §invariant-1 (circuit-breaker)
- ADR-0248 cellular architecture (shuffle-sharding)
- `gateway/connector/runbooks/dlq-overflow.md`
- `microservices/connector/capacity-model.md`
