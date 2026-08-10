---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix
microservice: tenancy
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-tenancy + council-architecture
deciders: axis-tenancy, council-architecture, gtm-customer-success
related_adrs: [ADR-0018, ADR-0123, ADR-0139]
related_artifacts:
  - tenancy/PRD.md (§Competitive Benchmark)
  - /specs/hyperscaler-gates.json (HG-TEN gate)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (tenancy µservice)

## Purpose

Quantitative + qualitative parity comparison vs the industry-leading multi-tenant identity + isolation products. Drives the `oya-governance-hyperscaler-maturity-claims` gate (per ADR-0123 HG-TEN). Re-validated bi-annually because the competitor landscape moves.

## Competitor Set

| Competitor | Product / surface | Primary differentiator | Source |
|---|---|---|---|
| AWS Cognito | Cognito User Pools + Identity Center | Mature B2C/B2B; AWS-native | `docs.aws.amazon.com/cognito/` |
| Auth0 (Okta) | Auth0 Organizations | Mature B2B-SaaS tenant model; rich rule engine | `auth0.com/docs/manage-users/organizations` |
| WorkOS | Organizations + Directory Sync | Best-in-class B2B SSO + provisioning | `workos.com/docs` |
| Stripe | platform model | Tenant (account) lifecycle; payment-platform precedent | `stripe.com/docs/connect` |
| Microsoft Entra (Azure AD External ID) | Multi-tenant directory | Mature enterprise; Microsoft-native | `learn.microsoft.com/en-us/entra/external-id/` |
| Neon | Serverless Postgres with branching | Per-tenant Postgres schema; instant provisioning | `neon.tech/docs` |
| Citus (Microsoft) | Citus multi-tenant Postgres | Shard-key tenant distribution; we adopt as substrate | `docs.citusdata.com` |

## Feature Parity Matrix

### Tenant lifecycle

| Capability | oyatie | Cognito | Auth0 | WorkOS | Stripe | Entra | Neon |
|---|---|---|---|---|---|---|---|
| Sub-5-min self-serve activation | ✅ (target) | ✅ | ✅ | ✅ | ✅ (Stripe-managed) | ✅ | ✅ |
| Tenant lifecycle FSM (suspend/resume/delete) | ✅ | partial | ✅ | ✅ | ✅ | partial | partial |
| Jurisdiction pinning (immutable at creation) | ✅ | per-region (mutable) | partial | partial | per-region | partial | per-region |
| Multi-pack residency (11 packs) | ✅ | 30+ regions | 15+ regions | partial | varies | 60+ regions | partial |
| Self-serve tenant operator portal | M01+1 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

### Isolation primitives (the differentiator)

| Capability | oyatie | Cognito | Auth0 | WorkOS | Stripe | Entra | Neon |
|---|---|---|---|---|---|---|---|
| Postgres RLS with FORCE ROW LEVEL SECURITY | ✅ | n/a | n/a | n/a | n/a | n/a | partial (per-schema only) |
| JWT + RLS + Cedar defence-in-depth | ✅ | JWT only | JWT only | JWT only | API-key | JWT only | RLS only |
| Per-tenant Postgres role separation | ✅ | n/a | n/a | n/a | n/a | n/a | partial |
| LEAN check `rls-no-superuser-bypass` | ✅ (oyatie unique) | – | – | – | – | – | – |
| LEAN check `rls-force-on-tenant-tables` | ✅ (oyatie unique) | – | – | – | – | – | – |
| LEAN check `jwt-key-fingerprint-advertised` | ✅ (oyatie unique) | – | – | – | – | – | – |
| Continuous DB-state validator (5min cadence) | ✅ (oyatie unique) | – | – | – | – | – | – |

### Right-to-erasure / DSR (the second differentiator)

| Capability | oyatie | Cognito | Auth0 | WorkOS | Stripe | Entra | Neon |
|---|---|---|---|---|---|---|---|
| Cross-µservice DSR cascade | ✅ | per-µservice manual | per-µservice manual | per-µservice manual | per-µservice manual | per-µservice manual | manual |
| Cryptographic proof-of-erasure (Merkle + Ed25519) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Per-µservice erasure-receipt aggregation | ✅ | – | – | – | – | – | – |
| Per-pack legal-SLA timer enforcement | ✅ | partial | partial | partial | partial | partial | partial |
| Regulator-disclosable certificate via API | ✅ | – | – | – | – | – | – |
| Soft-delete with 30d recovery grace | ✅ | partial | partial | partial | partial | partial | partial |

### Operational + integrations

| Capability | oyatie | Cognito | Auth0 | WorkOS | Stripe | Entra | Neon |
|---|---|---|---|---|---|---|---|
| Patroni HA + Citus sharding | ✅ (self-hosted) | n/a (SaaS) | n/a (SaaS) | n/a (SaaS) | n/a (SaaS) | n/a (SaaS) | partial (own HA model) |
| Multi-language SDK | M01: Rust; M01+1: TS; M02: Py/Go; M03: JVM | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Cedar / Rego / OPA policy integration | ✅ Cedar | partial (IAM) | rule engine | partial | partial | partial (Conditional Access) | – |
| Tenant scope enum (trial/production/sandbox/internal) | ✅ | partial | ✅ | ✅ | partial | partial | partial |
| Audit-chain Ed25519 sealed events | ✅ (oyatie unique) | partial (logs) | partial (logs) | partial (logs) | partial (logs) | partial (logs) | – |

## Quantitative Performance Parity

(30-day rolling-window evaluations on equivalent workloads.)

| Metric | oyatie target | Auth0 reference | Cognito reference | Notes |
|---|---|---|---|---|
| Tenant validate p99 latency | ≤ 5 ms | ≤ 50 ms typical | ≤ 100 ms | oyatie advantage via Valkey cache + RLS-as-server-enforcement |
| Tenant activation p99 latency | ≤ 5 min | ≤ 5 min | ≤ 5 min | parity |
| DSR completion (oyatie M01) | ≤ 30 days | varies (often manual) | varies (often manual) | oyatie unique end-to-end automation |
| Validate-path availability monthly | 99.99% | 99.9% | 99.9% | oyatie tighter (highest in catalog) |
| JWT signing-key rotation cadence | 30d | 90d typical | varies | oyatie tighter rotation discipline |

## Key Parity Gaps to Close (oyatie → industry leader)

| # | Gap | Owner | Target close |
|---|---|---|---|
| 1 | Multi-language SDK breadth (Py / Go / JVM / .NET) | axis-tenancy | M02–M03 |
| 2 | Self-serve tenant operator portal (web UI) | axis-tenancy + gtm | M01+1 |
| 3 | Mature B2B SSO + directory sync (WorkOS / Auth0 strength) | identity µservice (separate; depends on tenancy) | M02 |
| 4 | Tenant + customer-tenant joint-controllership UX | council-privacy + gtm | M02 |
| 5 | Built-in support for SAML / OIDC tenant federation | identity µservice (separate) | M03 |

## Key oyatie Differentiators (NOT in any competitor)

1. **Cryptographic proof-of-erasure**: Merkle-rooted regulator-disclosable certificate aggregated across every µservice's DSR handler. Auth0/Cognito offer per-product erasure; none offer cross-µservice cryptographic aggregation.
2. **Three-layer defence-in-depth**: RLS (database row) + JWT (request claim) + Cedar (policy evaluator). Three orthogonal failure modes; no single point of compromise.
3. **OpenSLO-shape RLS authoring**: Declarative `microservices/tenancy/policy/rls/<table>.yaml` + PR-reviewable + CI-validated + auto-deployed. Competitors hide RLS in migration files; oyatie surfaces.
4. **Patroni + Citus self-hosted substrate**: full control of HA topology + shard distribution + WAL retention; competitors are SaaS-only.
5. **Multi-pack pinning with explicit cross-pack-replication-forbidden**: 11 packs with concrete per-pack regulatory citations + LEAN-enforced residency.
6. **LEAN-enforced isolation invariants**: rls-no-superuser-bypass + rls-force-on-tenant-tables + jwt-key-fingerprint-advertised lanes refuse PR-time evasion of isolation.

## Claim-Boundary Rules

Sales claims permitted (citation-bounded):
- ✅ "oyatie tenancy includes cryptographic proof-of-erasure aggregated across every µservice; we believe this is unique among production multi-tenant SaaS platforms as of 2026-05-17" (review bi-annually).
- ✅ "oyatie tenancy enforces Postgres RLS + JWT + Cedar as three orthogonal isolation layers" (factually true; cited in `policy/rls-isolation.md`).
- ✅ "oyatie tenancy is multi-pack residency by design with cross-pack movement explicitly forbidden" (cited).

Sales claims FORBIDDEN (per ADR-0123 hyperscaler-maturity-claim-gate):
- ❌ "oyatie is faster than Auth0 on tenant validation" (no published Auth0 benchmark; would be unsourced superiority).
- ❌ "oyatie is HIPAA-compliant out of the box" (conditional on BAA + pack-us-healthcare activation; do not claim universal).
- ❌ "We replace Stripe Connect" (different scope; we don't process payments; do not claim universal substitution).

## Bi-Annual Refresh Process

| Step | Owner |
|---|---|
| 1. Survey competitor docs for changes | gtm-customer-success |
| 2. Update this matrix; cite sources | axis-tenancy |
| 3. Re-run quantitative benchmarks (load tests in staging) | ops-sre-reliability |
| 4. Council-architecture review for claim-boundary updates | council-architecture |
| 5. Publish + notify sales/gtm | gtm-customer-success |

## References

- `tenancy/PRD.md` §Competitive Benchmark.
- `/specs/hyperscaler-gates.json` HG-TEN gate.
- ADR-0123 (hyperscaler-maturity-claim-gate).
- ADR-0018 (tenancy + RLS posture).
- ADR-0139 (agentic SLO-gated promotion).
- Competitor docs cited inline.
