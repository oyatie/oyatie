# Milvus Tenant Quota Runbook

**Authority:** ADR-0155, ADR-0192 §"Multi-tenancy isolation"
**Last reviewed:** 2026-05-18

## Quota exceeded — diagnosis

`milvus_proxy_req_count{result="quota_exceeded"}` alerts when a tenant exceeds upsertRate / searchRate.

1. Identify the tenant: `milvus_proxy_req_count{result="quota_exceeded"}` PromQL by tenant label.
2. Check the tenant's tier in the tenancy µservice.
3. If tier-appropriate burst, document + escalate to capacity for tier upgrade.
4. If misbehaving tenant, contact account team.

## Quota upgrade path

When a tenant moves Tier → Tier:

1. Tenancy µservice emits `tenant.tier_changed` event.
2. Foundry tenant-bootstrap controller (IP-092) re-applies the new tier's QUOTA.
3. Verify: `kubectl exec milvus-proxy-0 -- /milvus/bin/milvus_cli show quota tenant_ten_acme`.

## Quota matrix (per-tier defaults)

| Tier | qps/collection | upsertRate (rows/sec) | replica.num |
|---|---|---|---|
| Trial | 50 | 100 | 1 |
| Starter | 500 | 1,000 | 2 |
| Growth | 5,000 | 10,000 | 2 |
| Enterprise | 50,000 | 100,000 | 3 |

Custom quota outside this matrix requires capacity-team approval + audit-chain event.
