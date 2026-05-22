---
doc_class: PolicySpec
title: Tenant Isolation Contract
microservice: connect
status: Accepted
date: 2026-05-20
owner_team: axis-integration + ops-security
related_adrs: [ADR-0244, ADR-0248, ADR-0254, ADR-0295]
related_artifacts:
  - microservices/connect/threat-model.md
  - microservices/connect/policy/connector-authorization.cedar
doc_status: published
---

# Tenant Isolation Contract — connect

## Invariants

| ID | Invariant | Enforcement |
|---|---|---|
| TI-01 | Every PG row carries `tenant_id` | Schema CHECK constraint + migration test |
| TI-02 | Every OpenBao path is tenant-scoped (`secret/<tenant_id>/connect/*`) | OpenBao policy fragment |
| TI-03 | Every connector adapter runs in its own Kata sandbox per tenant per ADR-0254 | K8s admission webhook |
| TI-04 | Every action invocation passes Cedar `connector-authorization.cedar` gate | runtime |
| TI-05 | Every audit event tagged with `tenant_id` per ADR-0263 | sealer enforces |
| TI-06 | No cross-tenant joins in PG queries | sqlx compile-time check via custom lint |
| TI-07 | Reserved tenants (`tenant:oya-ci`, `tenant:oya-staging`, `tenant:oya-aggregate`, `tenant:oya-sandbox`) cannot be impersonated by customer principals | Cedar `ci-scope.cedar` |
| TI-08 | Per-tenant Valkey token-buckets sharded per ADR-0248 (shuffle N=4 of M=64 shards) | Valkey cluster config |
| TI-09 | Per-tenant DLQ encrypted with tenant-scoped key | OpenBao envelope encryption |
| TI-10 | Per-tenant webhook DNS subdomain isolates routing (ADR-0273) | DNS GSLB config |

## Test plan

- Property test: random tenant pairs cannot leak via any read path (10k iterations per CI run).
- Fuzz: malicious tenant_id values rejected at Cedar gate.
- Chaos: kill one tenant's Kata sandbox; verify no impact to other tenants' actions.

## References

- ADR-0244 tenant as universal scoping primitive
- ADR-0254 Cloud Hypervisor + Kata pods
- ADR-0263 audit-event-class emission contract
