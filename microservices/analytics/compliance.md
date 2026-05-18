# Analytics µservice — Compliance Posture

**Authority:** ADR-0038 DSR cascade, ADR-0008 DUBO, ADR-0156 PII registry
**Last reviewed:** 2026-05-18

## Frameworks in scope

| Framework | Scope | Evidence path |
|---|---|---|
| GDPR | EU pack tenants | `microservices/analytics/policy/gdpr.cedar` + audit-chain |
| K-PIPA (Korean PIPA) | KR pack tenants | `microservices/analytics/policy/kpipa.cedar` + audit-chain |
| SOC 2 Type II | All tenants | `microservices/analytics/scorecards/soc2-controls.json` |
| HIPAA (where applicable) | Tenants with `data_class=PHI` | `microservices/analytics/policy/hipaa.cedar` |

## Key controls

### Data-class taxonomy (ADR-0008 DUBO)

Every column in every per-tenant table has a `data_class` tag per ADR-0156. Cedar policies forbid cross-class projection without an explicit DUBO grant.

### DSR (Data Subject Right) cascade (ADR-0038)

- **Right to access.** Tenant runs audit-log-search; results delivered via the regulator-export evidence pack flow (IP-013).
- **Right to erasure.** Tenant offboard → IP-002 controller drops the per-tenant database → proof-of-erasure emitted into audit-chain.
- **Right to portability.** IP-013 regulator-export produces NDJSON in a documented schema; portable to any compatible analytics system.

### Audit chain (ADR-0003)

- Every external query emits an audit event.
- Audit events are themselves stored in ClickHouse (recursive observation defense).
- Cosign-signed quarterly evidence pack.

### Retention (ADR-0184 + per-table TTL)

- Tenant audit-log: 7 years (compliance).
- Tenant business KPIs: 90 days hot + 1 year cold.
- Telemetry rollups: 1 year cold (observability cluster, not this µservice).

### Residency (ADR-0049 + ADR-0010)

- StrictKR tenant data lives only in kr-* cells.
- StrictEU tenant data lives only in eu-* cells.
- KrWithUsFailover: primary KR; failover to us-* under explicit DR scenarios only.

## Audit trail of audit trail

The audit-chain stream itself emits a meta-audit event on every audit-chain emission. Loss of audit events is detectable via the meta-stream's sequence number gap.

## References

- ADR-0003 audit chain, ADR-0008 DUBO, ADR-0038 DSR cascade, ADR-0049 cross-region, ADR-0156 PII registry, ADR-0193.
