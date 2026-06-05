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
Stripe (platform-facilitator) + Salesforce Tenant Management + Slack Enterprise Grid +
Atlassian Cloud Organisation.

## Entry points

- `PRD.md`, `ARCHITECTURE.md`, `threat-model.md`, `dpia.md`, `compliance.md`.
- `runbooks/`: tenant-onboarding, suspension, deletion, RLS recovery, Citus rebalance, JWT
  rotation.

## Bounded contexts

`tenant-lifecycle` / `sub-scope-registry` / `reserved-namespace` / `kyb-kyc` / `dr-pairing` /
`data-residency-enforcement` / `lifecycle-locks` / `citus-distribution` / `per-tenant-quota`.

## Doctrine references

- ADR-0346 (historical local-verifier doctrine): active tenancy evidence is Buck2 target output plus trusted Rust/Prow `oya-ci-required`; do not revive the retired local dev CLI, manual verification bridge, or manual success-status path.
- [ADR-0347](../../docs/decisions/ADR-0347-foundry-fitness-to-governance-bulk-rename.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 (historical self-hostable CI/CD doctrine): active direction is ADR-0513 native Rust/Prow oya-ci plus native release conveyor; GitHub Actions remains temporary/shadow hosted PR compatibility until native cutover.
