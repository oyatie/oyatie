# Analytics µservice — Threat Model

**Authority:** ADR-0007 Cedar, ADR-0008 DUBO, ADR-0193, ADR-0156 PII registry
**Last reviewed:** 2026-05-18

## Trust boundaries

1. **Tenant ↔ API gateway.** Tenant authenticates per ADR-0157; gateway forwards SPIFFE-bound identity downstream.
2. **API gateway ↔ analytics REST/gRPC handlers.** Cedar-authorized; per-tenant scoped.
3. **Analytics handlers ↔ ClickHouse.** Adapter binds to the per-tenant database via the `tenant_{tid}_reader` / `tenant_{tid}_writer` role.
4. **ClickHouse ↔ S3-compat cold tier.** OpenBao-resolved credentials; per-cell scoped.
5. **Outbox ↔ ClickHouse Kafka engine.** Pulsar KoP endpoint; mutual TLS.

## STRIDE per surface

### Spoofing

- **Threat:** Caller fakes `tenant_id` in API request.
- **Mitigation:** Cedar verifies `principal.tenant_id == resource.tenant_id`. Adapter additionally calls `assert_same_tenant(caller, qualified_table)` at the kernel layer before SQL dispatch.

### Tampering

- **Threat:** Attacker injects SQL via the typed query DSL.
- **Mitigation:** Typed query DSL never accepts raw SQL from external callers; the adapter renders parameterized SQL with `{name:Type}` placeholders. The `TableName` type at construction rejects strings containing `;` or `--`. SQL injection is foreclosed at the type layer.

### Repudiation

- **Threat:** Tenant denies running a query they actually ran.
- **Mitigation:** Audit-chain emission per ADR-0003 — every external query is logged with `(tenant_id, principal, query_hash, ts, result_count)`.

### Information disclosure

- **Threat:** Cross-tenant data leak via misconfigured row-level policy.
- **Mitigation:** Defense in depth — per-tenant database (primary isolation) + row-level policy (fallback) + adapter-layer `assert_same_tenant`. Penetration test as part of IP-014 acceptance.
- **Threat:** PII surfacing in audit-log-search results.
- **Mitigation:** Cedar policy filters PII-tagged columns per ADR-0156 PII registry. Per-axis policy at `microservices/analytics/policy/audit-log-pii.cedar`.

### Denial of service

- **Threat:** Tenant runs runaway query.
- **Mitigation:** Per-tenant QUOTA per ADR-0155 + ClickHouse `max_execution_time` per query setting.
- **Threat:** Tenant ingest burst exceeds capacity.
- **Mitigation:** Pulsar consumer offset backpressure; per-collection upsertRate quota.

### Elevation of privilege

- **Threat:** Tenant-reader role escalates to writer.
- **Mitigation:** ClickHouse RBAC + Cedar Action::"Insert" vs Action::"Read" separation.
- **Threat:** Admin password leak.
- **Mitigation:** Admin password resolved via OpenBao SecretReference; 90-day rotation; never in source.

## High-impact incident scenarios

1. **Tenant data leak.** Detection: audit-chain query showing cross-tenant pattern. Response: immediate Cedar policy lockdown; forensic ClickHouse query-log audit; tenant notification per GDPR 72h rule.
2. **Backup compromise.** Detection: cosign signature verification fails on backup pull. Response: invalidate the backup; fall back to the prior valid backup; investigate the signing chain.
3. **Ingest pipeline lateral attack.** Detection: anomalous tenant_id values in the Kafka stream. Response: reject malformed events at the consumer; quarantine; investigate source µservice.

## References

- ADR-0007, ADR-0008, ADR-0038, ADR-0156, ADR-0193.
