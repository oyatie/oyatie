---
id: ADR-0058
status: Superseded
superseded_by: [ADR-701]
doc_status: published
---

# ADR-0058: Flat microservice catalog — Product Groups retired

> **Status:** Accepted
> **Owner:** `council-architecture`
> **Date:** 2026-05-13
> **Related:** ADR-0001, ADR-0056, ADR-0059, ADR-0060, ADR-0061, ADR-0062

---

## Context

Prior planning artifacts used "Product Groups", "Arms", and "Verticals" as architectural groupings (Healthcare Arm, Corporate SaaS Arm, FinTech Arm, Platform/Ops). Session decisions 2026-05-13 retired these groupings entirely.

User instruction 2026-05-13:
- "oyatie is just working in parallel. same product as bominal."
- "they are individual and modularized products that can be integrated with each other similar to how microservices integrate with each other in clean architecture."
- "In essence everything is 'shared'."

Oyatie is a flat catalog of shared microservices. Any tenant can enable any subset. Healthcare / Enterprise / FinTech / Social are **sales and marketing segmentation only** — not architectural grouping, not directory structure, not code structure.

**Naming justification:** "flat catalog" — the established microservices architecture term for an independently deployable, modular service catalog. "Product Group" / "Arm" / "Vertical" are retired per this ADR.

---

## Decision

We adopt the **flat microservice catalog** as the canonical architecture. No vertical, arm, product group, or platform grouping exists in code, directories, or architecture. Every feature and product is an independent microservice registered in `[workspace.metadata.oya.microservices]` (per ADR-0056 BNF v4.1).

### Canonical flat catalog (complete as of 2026-05-13)

```
Foundry (internal-only, not tenant-facing):
  foundry

Substrate (always-on; underpin every product):
  tenancy, identity, audit-chain, eventing, secrets,
  observability, kms, policy, search, vector,
  data-boundary, finance-library, capability-registry,
  records, application, ads, analytics

Cloud substrate:
  cloud (cloud-tenancy, cloud-iam, cloud-kms, cloud-compute,
         cloud-storage, cloud-network, cloud-billing, cloud-cell,
         cloud-region, cloud-observability)

Ecosystem adapter layer:
  workflow, ontology

Application (B2B shell):
  application

Customer-facing products:
  medical, pharmacy, healthcare-portal, emergency, clinical,
  hr, payroll, accounting, ats, grc, performance,
  manufacturing, logistics, facility-ops, procurement, security,
  payments, insurance, finance, banking,
  connect,
  dining, cellar
```

### Tenant enables à-la-carte

A tenant enables a subset of products from the flat catalog — like enabling services in an AWS console:

- Hospital tenant example: `{medical, pharmacy, hr, payroll, accounting, payments}`
- Corporate tenant example: `{hr, payroll, accounting, manufacturing, procurement, payments}`
- FinTech tenant example: `{payments, banking, finance, hr, payroll, accounting}`

No tenant is locked to a group. Any combination is valid.

### Sales segmentation is GTM only

"Healthcare" / "Enterprise" / "FinTech" / "Social" labels are:
- Permitted in: marketing copy, pricing pages, bundle defaults, go-to-market segmentation
- Forbidden in: code structure, directory layout, crate names, ADR titles, architectural diagrams

### Retired terms

The following terms are retired from all oyatie architectural artifacts:
- Product Group
- Arm (Healthcare Arm, Corporate SaaS Arm, FinTech Arm, Platform/Ops)
- Vertical (as architectural grouping)
- "platform" (as oyatie substrate name; use "shared" or the specific µservice name)

### BNF v4.1 alignment

The BNF v4.1 (ADR-0056) slots encode this directly:
- No `shared|vertical` slot2 enum — retired
- Slot2 = registered microservice name (open kebab, registry-validated)
- `oya-check-architecture` refuses crate names with `oya-platform-*`, `oya-shared-*` (except for actual shared substrate crates)

### Bominal inheritance

Oyatie inherits Bominal's product decisions 1:1 with glossary translation:
- Bominal "Arm" → oyatie "microservice" or flat catalog entry
- Bominal "Platform/Ops" → oyatie "shared substrate"
- Bominal "Modular Product Shell" → oyatie "Application"
- Bominal "Workspace" → oyatie "Connect"

Per `[[feedback-bominal-inheritance-precedence]]` override #4 and #5.

---

## Consequences

### Quality / Performance / Scalability (per ADR-0062)

- Flat catalog eliminates cross-group dependency edges; LEAN-A2 cross-microservice refusal check is cleaner with no arm/group exceptions.
- Each microservice independently scalable; no group-level capacity planning.
- Performance targets declared per-microservice in individual PRDs; no group-level pooling of perf budgets.

**Clean architecture lanes that apply directly to the flat catalog:**

| Lane | What it enforces on the flat catalog |
|---|---|
| `oya-shared-architecture-check-cli -- cross-product-refusal` (LEAN-A2) | No direct cross-microservice imports at any layer; product crates like `oya-medical-*` MUST NOT import `oya-pharmacy-*` |
| `oya-shared-bounded-contexts-check-cli` | Every microservice registered in `[workspace.metadata.oya.microservices]`; unregistered µservice names fail |
| `oya-check-statelessness-cli` | All `application`/`rest`/`grpc`/`graphql`/`worker` layer crates in every catalog microservice have zero module-level mutable state |
| `oya-check-shardability-cli` | All DB-backed microservices declare `tenant_id` partition key + RLS |

Per `[[feedback-clean-architecture-requirements]]` §4 (cross-product rule) and Bominal ADR-0101 (hexagonal microservice standard, inherited).

### Concrete artifacts changed

Directory structure:
```
crates/
  oya-tenancy-kernel/
  oya-identity-kernel/
  oya-workflow-kernel/
  oya-ontology-entity-kernel/
  oya-medical-domain/
  oya-mail-kernel/
  oya-payments-ledger-application/
  ... (all per BNF v4.1; no subdirectories by arm/group/vertical)
```

No `crates/healthcare/`, `crates/enterprise/`, `crates/fintech/` directories. All crates flat under `crates/`.

`[workspace.metadata.oya]` in root `Cargo.toml`:
```toml
[workspace.metadata.oya.microservices]
# full flat list per catalog above
# NO [workspace.metadata.oya.verticals] or [workspace.metadata.oya.arms]
```

### Positive

- Any tenant can enable any product combination; no artificial grouping limits.
- Clean architecture: each microservice independently deployable, testable, scalable.
- No group-level blast radius; one microservice's outage does not cascade to its "arm."

### Negative

- Go-to-market must explicitly distinguish architectural flat catalog from sales bundles.
- New contributors may expect grouping; onboarding docs must explain the flat model.

---

## Related

- ADR-0001 (cohesion thesis — flat catalog is the product)
- ADR-0056 (BNF v4.1 — slot2 = microservice name, no shared|vertical)
- ADR-0059 (Workflow + Ontology = ecosystem adapter layer)
- ADR-0060 (Bominal-inheritance — override #4 and #5)
- ADR-0061 (Application — B2B shell)
- ADR-0062 (Quality/Performance/Scalability bar)
- `[[feedback-flat-product-catalog]]` — canonical session decision 2026-05-13
- `[[feedback-bominal-inheritance-precedence]]` — override list
