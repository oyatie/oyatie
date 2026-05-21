---
doc_class: Reference
shape: Reference
microservice: tenancy
companion_docs:
  - microservices/tenancy/ARCHITECTURE.md
  - microservices/tenancy/PRD.md
related_adrs:
  - ADR-0244
  - ADR-0242
inbound_citations:
  - docs/DOC-CATALOG.md
---

# tenancy

The tenant universal-scoping substrate (ADR-0244). Tenant lifecycle + sub-scope registry +
reserved-namespace enforcement + KYB-KYC + DR-pairing + data-residency enforcement + lifecycle
locks + Citus distribution + per-tenant quotas. Hyperscaler precedents: AWS Organisations +
Stripe Connect (platform-facilitator) + Salesforce Tenant Management + Slack Enterprise Grid +
Atlassian Cloud Organisation.

## Entry points

- `PRD.md`, `ARCHITECTURE.md`, `threat-model.md`, `dpia.md`, `compliance.md`.
- `runbooks/`: tenant-onboarding, suspension, deletion, RLS recovery, Citus rebalance, JWT
  rotation.

## Bounded contexts

`tenant-lifecycle` / `sub-scope-registry` / `reserved-namespace` / `kyb-kyc` / `dr-pairing` /
`data-residency-enforcement` / `lifecycle-locks` / `citus-distribution` / `per-tenant-quota`.
