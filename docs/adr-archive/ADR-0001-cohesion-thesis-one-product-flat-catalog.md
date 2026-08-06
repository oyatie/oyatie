---
id: ADR-0001
status: Superseded
superseded_by: [ADR-0705]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0001: Adopt the cohesion thesis — one product across a flat microservice catalog joined at six shared substrates

> **Status:** Accepted
> **Owner:** `council-architecture`
> **Date:** 2026-05-09 (rewritten 2026-05-13)
> **Related:** ADR-0002, ADR-0003, ADR-0004, ADR-0005, ADR-0006, ADR-0007, ADR-0008, ADR-0009, ADR-0010, ADR-0011, ADR-0058, ADR-0059

---

## Context

Oyatie's commercial premise is that an integrated **ecosystem-as-a-service** wins over a portfolio of best-of-breed substitutes whenever the integration tax dominates the unit-quality gap between the integrated and unbundled options. Multi-vendor enterprise stacks pay that tax in five places simultaneously: identity, billing, audit, consent, and capability registration. Each of those is a per-vendor adapter whose drift becomes a perpetual integration cost. A single product that spans every layer of an enterprise's compute, data, and intelligence surface — but only if the layers actually share substrate at the contract level — collapses that drift to zero.

Oyatie is the same product as Bominal, built in parallel. Every feature and product in the catalog is independent, modular, and integrate-able like microservices in clean architecture. A hospital tenant may enable `{medical, pharmacy, hr, payroll, accounting, payments}`; a corporate tenant may enable `{hr, payroll, accounting, manufacturing, procurement, payments}`. There is no architectural grouping by industry or arm — those labels are sales and marketing segmentation only.

---

## Decision

We adopt the **cohesion thesis** as the foundational invariant of the Oyatie codebase, product, and roadmap:

> Oyatie is one cohesive product across a flat catalog of shared microservices, joined at exactly six shared substrates: **single tenancy**, **single identity**, **single audit chain**, **single capability registry**, **single agent runtime**, and **single autonomy ceiling**. No microservice ships a surface that re-implements any of those six substrates.

### Flat microservice catalog

All customer-facing products and substrate features live in a flat catalog. Any tenant can enable any subset. There is no vertical, arm, product group, or platform grouping in code or architecture:

```
Application (B2B unified shell): tenants sign in; enable products à-la-carte.

Flat catalog — customer-facing products:
  medical, pharmacy, healthcare-portal, emergency, clinical,
  hr, payroll, accounting, ats, grc, performance,
  manufacturing, logistics, facility-ops, procurement, security,
  payments, insurance, finance, banking,
  connect (dual-context: messenger + mail + community),
  dining, cellar, ...

Cross-product adapter/glue layer:
  workflow (state machines, DAGs, approvals, escalations, SLA, handoffs)

Information layer (Palantir-Ontology equivalent):
  ontology (typed entities + links + actions + audit-chain +
            pillars [org / person] + property tiers + DUB + RLS +
            jurisdiction overlays)

Substrate features (always-on; underpin every other product):
  tenancy, identity, audit-chain, eventing, secrets,
  observability, kms, policy (Cedar), search, vector,
  data-boundary, finance-library, capability-registry,
  records (FHIR-canonical), application (B2B shell),
  ads, analytics

Runtime substrate:
  cloud-tenancy, cloud-iam, cloud-kms, cloud-compute, cloud-storage,
  cloud-network, cloud-billing, cloud-cell, cloud-region,
  cloud-observability

Foundry (internal-only): grit, icm, oya-tooling-agent-read, LEAN check
  binaries, xtask-metadata-augment, Cedar engine,
  Wasmtime/Firecracker, Proof Ladder, fitness lanes,
  9 architecture planes, Wave integration framework.
```

**Personal** (B2C) is a separate entry path via the person-pillar; it does not go through the B2B Application shell.

### The six substrates are codified

Each substrate has exactly one owning bounded context (per BNF v4.1, ADR-0056):

- Single tenancy → `oya-tenancy-kernel` (ADR-0002)
- Single identity → `oya-identity-kernel` (ADR-0002)
- Single audit chain → `oya-audit-chain-kernel` (ADR-0003)
- Single capability registry → `oya-intelligence-capability-kernel` + catalog (ADR-0011)
- Single agent runtime → `oya-intelligence-runtime-*` (ADR-0007)
- Single autonomy ceiling → `oya-intelligence-policy-kernel` (ADR-0007)

### Cohesion invariants

```rust
// oya-governance-cohesion-kernel
pub enum ForbiddenPattern {
    /// A microservice re-implements an entity that already lives in a substrate kernel.
    SubstrateForking { microservice: MicroserviceId, substrate: Substrate, evidence: PathBuf },
    /// A microservice ships a tenant boundary that bypasses the canonical Tenant kernel.
    TenantSidecar { microservice: MicroserviceId, evidence: PathBuf },
    /// A microservice emits regulatory events outside the audit-chain kernel.
    OffChainAudit { microservice: MicroserviceId, evidence: PathBuf },
    /// A microservice exposes an agent surface that bypasses the capability registry.
    UnregisteredCapability { crate_id: CrateId, capability_id: String },
    /// A microservice enforces an autonomy decision locally instead of via the policy kernel.
    LocalAutonomyOverride { crate_id: CrateId, evidence: PathBuf },
}
```

The fitness lane `oya-check-cohesion` walks the catalog + dep graph + audit-emission inventory and hard-fails any forbidden pattern.

### Boundary

- Applies to: every crate under `crates/oya-*`, every catalog record, every capability, every contract.
- Does not apply to: experimental research crates explicitly outside the workspace tree, third-party deps.

---

## Consequences

### Positive

- The cohesion thesis becomes a CI-enforced invariant. Drift is detected at PR time.
- New microservices inherit the substrates automatically.
- Customer-facing positioning ("one tenancy, one audit, one identity, one ceiling") is mechanically true.

### Negative

- Every PR that crosses a microservice boundary pays an explicit review tax.
- Refactoring a substrate is a heavy maneuver because every microservice depends on it.
- Some ergonomically tempting shortcuts (e.g. microservice-local tenant cache) are forbidden.

---

## Alternatives considered

### Alternative A — Microservice-local substrates with cross-service adapters

- **Rejected because:** the integration tax is precisely what we are trying to remove.

### Alternative B — Grouping by sales segment (Healthcare / Enterprise / FinTech)

- **Rejected because:** grouping is sales and marketing segmentation only; putting it in architecture creates dependency edges that violate the cohesion invariant.

---

## Related

- ADR-0002 (Tenant + Identity kernel)
- ADR-0003 (Audit chain)
- ADR-0007 (Cedar policy + autonomy ceiling)
- ADR-0011 (Capability registry)
- ADR-0056 (BNF v4.1)
- ADR-0058 (Flat microservice catalog — Product Groups retired)
- ADR-0059 (Workflow + Ontology = ecosystem adapter layer)
- `[[feedback-flat-product-catalog]]` — canonical session decision 2026-05-13
- `[[feedback-bominal-inheritance-precedence]]` — Bominal-inheritance + overrides
