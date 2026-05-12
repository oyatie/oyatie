# ADR-0001: Adopt the cohesion thesis — one product across seven axes joined at six shared substrates

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `council-architecture`
> **Date:** 2026-05-09
> **Related:** ADR-0002, ADR-0003, ADR-0004, ADR-0005, ADR-0006, ADR-0007, ADR-0008, ADR-0009, ADR-0010, ADR-0011, ADR-0012

---

## Context

Oyatie's commercial premise is that an integrated **ecosystem-as-a-service** wins over a portfolio of best-of-breed substitutes whenever the integration tax dominates the unit-quality gap between the integrated and unbundled options. Multi-vendor enterprise stacks pay that tax in five places simultaneously: identity, billing, audit, consent, and capability registration. Each of those is a per-vendor adapter whose drift becomes a perpetual integration cost. A single product that spans every layer of an enterprise's compute, data, and intelligence surface — but only if the layers actually share substrate at the contract level — collapses that drift to zero.

The repositioning conversations on 2026-05-08 and 2026-05-09 shifted Oyatie from a vertical-SaaS framing to a seven-axis ecosystem framing (SaaS multi-tenant platform, Workspace / Productivity Suite, Vertical industry cloud, Foundry agent runtime + engineering platform, Cloud provider, Search, Advertising + analytics). Without an explicit foundational ADR that names the *cohesion contract*, every subsequent axis-level ADR is unmoored: it can claim the moat without pointing at the substrates that produce it. This ADR pins the thesis so that every other ADR in this pack inherits the same baseline.

---

## Decision

We adopt the **cohesion thesis** as the foundational invariant of the Oyatie codebase, product, and roadmap. It states:

> Oyatie is one cohesive product across seven axes — SaaS, Workspace, Vertical, Foundry, Cloud, Search, Ads/Analytics — joined at exactly six shared substrates: **single tenancy**, **single identity**, **single audit chain**, **single capability registry**, **single agent runtime**, and **single autonomy ceiling**. No axis ships a surface that re-implements any of those six substrates.

### Concrete commitments

- **Axis enumeration is closed.** The seven axes above are the canonical set. Adding, removing, splitting, or merging an axis is governed by ADR-0012 (Axis Admission Protocol), not by ad-hoc PRs.
- **The six substrates are codified.** Each substrate has exactly one owning bounded context (per ADR-0015 flat-crates target):
  - Single tenancy → `crates/oya-platform-tenant-kernel` (ADR-0002)
  - Single identity → `crates/oya-platform-identity-kernel` (ADR-0002)
  - Single audit chain → `crates/oya-platform-audit-chain-kernel` (ADR-0003)
  - Single capability registry → `crates/oya-foundry-capability-kernel` + catalog (ADR-0011)
  - Single agent runtime → `crates/oya-foundry-runtime-*` (ADR-0007)
  - Single autonomy ceiling → `crates/oya-foundry-policy-kernel` (ADR-0007)
- **Cross-axis contracts are first-class artifacts.** The DESIGN §10 contract surface is the auditable cohesion check. ADR-0011 makes the contract registry the source of truth.
- **Every axis declares its plane** (control / data / analytics) per ADR-0004; every axis declares its data classes touched per ADR-0008.

### Cohesion invariants (the rules every other ADR in this pack must satisfy)

```rust
// crates/oya-foundation-cohesion-kernel
pub struct CohesionInvariants {
    pub axes: [Axis; 7],
    pub substrates: [Substrate; 6],
    pub forbidden_patterns: Vec<ForbiddenPattern>,
}

pub enum ForbiddenPattern {
    /// An axis re-implements an entity that already lives in a substrate kernel.
    SubstrateForking { axis: Axis, substrate: Substrate, evidence: PathBuf },
    /// An axis ships a tenant boundary that bypasses the canonical Tenant kernel.
    TenantSidecar { axis: Axis, evidence: PathBuf },
    /// An axis emits regulatory events outside the audit-chain kernel.
    OffChainAudit { axis: Axis, evidence: PathBuf },
    /// An axis exposes an agent surface that bypasses the capability registry.
    UnregisteredCapability { crate_id: CrateId, capability_id: String },
    /// An axis enforces an autonomy decision locally instead of via the policy kernel.
    LocalAutonomyOverride { crate_id: CrateId, evidence: PathBuf },
}
```

The fitness lane `oya-foundry-fitness-cohesion` walks the catalog + dep graph + audit-emission inventory and hard-fails any forbidden pattern. The lane is gating for every PR labeled `cross-axis`.

### Boundary (where the thesis applies; where it does not)

- Applies to: every crate under `crates/oya-*`, every catalog record under `registry/catalog/`, every capability under `product-control/capabilities/`, every contract under `contracts/`, every regional pack under `regional-packs/`.
- Does not apply to: experimental research crates explicitly outside the workspace tree, third-party deps, the legacy `modules/` / `services/` / `platform/` tree being migrated under ADR-0015.

---

## Consequences

### Positive

- The cohesion thesis becomes a CI-enforced invariant rather than slide-deck text. Drift is detected at PR time, not in a quarterly audit.
- New axes inherit the substrates automatically — no axis ships its own tenant model, identity surface, or audit emitter.
- The pack-of-19 foundation ADRs becomes self-consistent: each substrate gets a dedicated ADR (0002–0007) and each cross-cutting protocol (0008–0019) builds on this baseline.
- Customer-facing positioning ("one tenancy, one audit, one identity, one ceiling") is mechanically true, not aspirational.

### Negative

- The discipline cost is real. Every PR that crosses an axis pays an explicit review tax (cross-axis label, cross-axis reviewer pair, fitness-lane block on substrate forking).
- Refactoring a substrate is a heavy maneuver because every axis depends on it. The Tenant kernel is by design the most-reviewed file in the repository.
- Some short-term-attractive shortcuts (e.g. axis-local tenant cache, axis-local consent receipt) are forbidden, even when ergonomically tempting.

### Operational

- Council-architecture holds standing review authority over substrate kernels.
- The cohesion fitness lane runs on every PR and emits an evidence record per check.
- Substrate kernel changes require an ADR amendment in this pack (or a downstream ADR that explicitly cites the relevant substrate ADR).
- The PR template's `## Traceability` section must cite at least one ADR from this pack on any PR labeled `cross-axis`.

---

## Alternatives considered

### Alternative A — Axis-local substrates with cross-axis adapters

- **Pros:** familiar SOA pattern; easy to onboard external contributors who think per-service.
- **Cons:** every cross-axis flow becomes a multi-vendor integration internally; drift is guaranteed; the cohesion moat collapses to "we happen to ship from one vendor" rather than "we share substrate."
- **Rejected because:** the integration tax is precisely what we are trying to remove. Adopting it internally re-creates it.

### Alternative B — Two-tier substrate (one per super-axis: platform vs cloud vs intelligence)

- **Pros:** smaller per-substrate review surface; clear sub-domain ownership.
- **Cons:** drift still occurs at the super-axis boundary; the audit chain in particular cannot be partitioned without losing tamper-evident properties.
- **Rejected because:** the math on audit chain immutability requires a single hash chain per tenant; partitioning destroys the property we depend on.

### Alternative C — Loose cohesion declaration (memo, no CI gate)

- **Pros:** zero tooling cost on day one.
- **Cons:** every prior memo-style invariant in the codebase has drifted. Without a fitness lane, the cohesion claim is unverifiable.
- **Rejected because:** failure mode is exactly the failure mode this ADR exists to prevent.

---

## Open questions

1. **Q1.** Should the cohesion fitness lane be `oya-foundry-fitness-cohesion` (Foundry-owned) or its own top-level lane? Default: Foundry-owned, since Foundry already owns the catalog the lane reads. → ADR-0011.
2. **Q2.** Does the Workspace axis (axis 2) need its own substrate (e.g. shared document-format kernel) added to the canonical six? Default: NO; Workspace consumes the existing six. Re-evaluate at W-Workspace-Preview gate. → ADR-0012.
3. **Q3.** What is the minimum viable cohesion test set the lane runs at PR time vs nightly? Default: PR-time runs catalog + dep-graph checks; nightly runs the full evidence-emission audit. → owner: `axis-foundry`.
4. **Q4.** Should an axis sub-axis (e.g. Foundry's robotics-control sub-axis per DESIGN §3.0.2) be allowed to declare an additional substrate? Default: NO, sub-axes inherit only. → ADR-0012.

---

## References

- `docs/PRD.md` §1 (north star), §5 (cohesion thesis), §6 (constraints)
- `docs/DESIGN.md` §1 (cohesion thesis), §10 (cross-axis contract surface), §11 (cross-axis contradiction audit)
- `docs/CONTRADICTION-LEDGER.md` LEDG-007 (cross-axis contracts incomplete) — addressed here
- ADR-0002 (Tenant + Identity kernel), ADR-0003 (Audit chain), ADR-0007 (Cedar policy + persona tier), ADR-0011 (Cross-axis contract registry), ADR-0012 (Axis admission protocol), ADR-0015 (Architectural flattening target)
