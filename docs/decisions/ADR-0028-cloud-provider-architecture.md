# ADR-0028: Cloud provider axis — three-phase compute trajectory with stable product surface

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `axis-cloud`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0002, ADR-0003, ADR-0007, ADR-0011, ADR-0032, ADR-0042, ADR-0043, ADR-0044, ADR-0045, ADR-0049

---

## Context

Axis 5 (Cloud) is the only axis that owns physical reality. The other six axes are software all the way down; Cloud must reason about racks, fiber, transformer redundancy, KEPCO substations, refrigerant loops, and per-cell HSM partitions. The pack-of-19 foundation ADRs decided that Oyatie ships a true cloud provider rather than a thin reseller, but they did not pin the **trajectory** by which that happens, nor the **product-surface stability invariant** that lets customers buy capacity now without re-platforming when the underlying compute shifts from public-cloud consumption to colocation to greenfield mega-DC. Without that pin, every team building on top of Cloud would either (a) wait for greenfield to ship before building, or (b) hardcode public-cloud assumptions that strand the colocation and greenfield investments.

The cohesion thesis (ADR-0001) requires that Cloud expose the same Tenant + Identity + Audit + Capability + Runtime + Autonomy substrates as every other axis. Cloud is also the substrate consumer for all other axes — Workspace runs on Cloud, Foundry runs on Cloud, Vertical packs run on Cloud — so the Cloud product surface is *the* compatibility constraint for the entire ecosystem. This ADR pins both the trajectory (Phase 1 → 2 → 3) and the surface invariants (compute / storage / network / IAM / regions / billing / observability / FinOps) that hold across phases.

---

## Decision

We adopt a **three-phase compute trajectory** with a **phase-invariant product surface**. Customers consume the same APIs, the same SKUs (with phase-tagged pricing), the same IAM model, and the same audit shape, regardless of whether the underlying capacity is rented from a hyperscaler, leased in an Oyatie colo, or owned in an Oyatie-built mega-DC.

### Phases

- **Phase 1 — Public-cloud consumption (W0..W12 default).** Primary providers: OCI (KR-Seoul region 1, primary; KR-Chuncheon region 2, secondary) and AWS (ap-northeast-2 fail-open, ap-northeast-1 DR). All capacity rented; per-tenant cells implemented as Kubernetes namespaces + dedicated node pools per data class.
- **Phase 2 — Hybrid Oyatie colo (W12..W36).** Primary KR colo lease (Equinix SL1/SL2 + KT IDC Mokdong + LG U+ Pyeongchon, three-site quorum). 60% of steady-state load runs on owned hardware in leased space; bursting and DR remain on Phase-1 capacity. Per-cell HSM partitions land here per ADR-0043.
- **Phase 3 — Greenfield Oyatie mega-DC (W36+).** Single-tenant 30-50MW campus, KR-eastcoast (geothermal/seawater cooling) primary, KR-westcoast secondary. Phase-1 capacity remains a permanent tier for global edge points-of-presence; Phase 2 colos are gradually wound down or kept for compliance-driven jurisdictional residency.

### Phase-invariant product surface (the contract)

```rust
// crates/oya-cloud-surface-kernel
pub struct CloudSurface {
    pub compute: ComputeSurface,
    pub storage: StorageSurface,
    pub network: NetworkSurface,
    pub iam: IamSurface,
    pub regions: RegionsSurface,
    pub billing: BillingSurface,
    pub observability: ObservabilitySurface,
    pub finops: FinOpsSurface,
}

pub enum ComputeSku {
    ManagedKubernetes { tier: KubeTier, node_class: NodeClass },
    Functions { runtime: FunctionRuntime, cold_start_class: ColdStartClass },
    VirtualMachine { shape: VmShape, isolation: IsolationLevel },
    BareMetalLease { rack_class: RackClass, term: LeaseTerm },
    Gpu { accelerator: AcceleratorClass, interconnect: InterconnectClass },
    EdgeCompute { pop_class: PopClass, latency_budget_ms: u16 },
}
```

- **Compute SKUs** (six categories above) are defined once and survive all three phases. Phase 1 maps `BareMetalLease` to OCI Bare Metal shapes; Phase 2 maps it to actual Oyatie-owned racks; Phase 3 maps it to the same. Same SKU, same SLA contract surface, different fulfillment.
- **Storage** offers four canonical surfaces — Object (S3-compatible), Block (NVMe-class IOPS tiers), File (NFSv4.1 / SMB3), Archive (cold) — plus the database tier per ADR-0045. Per-cell key material via ADR-0043.
- **Network** offers VPC (per-tenant per-cell), Load Balancer (L4 + L7 with mTLS termination), DNS (per-tenant zones + DNSSEC), Interconnect (Direct Connect equivalent — phase 1 via OCI FastConnect / AWS DX; phase 2+ via owned cross-connect), DDoS protection (line-rate scrubbing per region), Service Mesh (per ADR-0044).
- **IAM** is Cedar-policy gated (per ADR-0007), federates SSO (SAML2 + OIDC), issues STS-style short-lived credentials, requires MFA for privileged actions, and emits to the audit chain (ADR-0003) for every authorization decision.
- **Regions, AZs, Cells.** Day 1: KR-Seoul1 (3 AZs, each ≥ 30km separation per Korean Financial Services Commission DR guidance). Per-pack region admission via the regional-pack architecture. A *cell* is a tenant-isolation unit within an AZ; per-vertical packs may require dedicated cells (e.g. healthcare PHI cells separate from generic SaaS cells).
- **Billing** is per-resource per-tenant with a per-pack tax-invoice format (KR 세금계산서 format mandatory for KR cells; per-region equivalents elsewhere). A single `BillingEvent` schema covers usage, metered overage, reservations, commitments, and credits.
- **Observability** ships per-tenant SLO dashboards (latency / availability / error budget) plus the audit-chain mirror (per ADR-0042 stack). Tenants own their own data in their own observability namespace; cross-tenant aggregation requires an explicit admin grant.
- **FinOps** ships per-axis cost attribution, per-cell unit-economics dashboards, reservation/commitment recommendations, and a cost-anomaly detector. The FinOps surface is the same in all three phases; in Phase 1 it speaks public-cloud cost APIs, in Phase 2/3 it speaks DCIM (per ADR-0032).

### Anti-scope

Cloud does not ship its own tenant model, identity surface, or audit emitter — those are the substrates from ADR-0002 / ADR-0003. Cloud also does not ship custom silicon, custom switching ASICs, or custom optical transceivers; commercial silicon only (per ADR-0032 anti-scope).

---

## Consequences

### Positive

- Customers buy capacity now and re-platform never; the SKU surface they sign a 3-year reservation against survives the trajectory.
- Phase-shift risk is bounded to Cloud-internal fulfillment plumbing; product teams above Cloud do not block on greenfield.
- The phase-invariant surface forces every SKU we ship to be implementable on rented, leased, and owned infra — ruling out shortcuts that would only work on one tier.
- KR data residency (per ADR-0049) is satisfied from day 1 via OCI KR-Seoul1, then strengthened with owned KR colo, then absolute via owned KR mega-DC — without changing the customer-facing residency contract.

### Negative

- The phase-invariant constraint slows down Phase 1; we cannot lean on hyperscaler-only SKUs (e.g. proprietary serverless databases) that would not survive Phase 3.
- Phase-2 colo financials are exposed to KEPCO industrial tariff volatility and KT/LG U+ cross-connect contracts; the FinOps surface must model both.
- Building a real cloud is a >10-year arc; the org must commit to operating-experience hiring (DC ops, network engineering, BMS/BAS) earlier than a software-only company would.

### Operational

- The Cloud axis runs an `oya-foundry-fitness-cloud-surface` lane that diff-checks every PR against the phase-invariant SKU surface — any PR that adds a hyperscaler-only SKU is rejected unless it also ships the colo + greenfield path.
- Per-cell HSM partition rotation is on the quarterly drill calendar (ADR-0043).
- DR drill: every region exercises full-AZ failure quarterly; full-region failure annually with a warm secondary.
- FinOps publishes per-axis monthly cost-attribution to the Trust Portal (ADR-0038) so tenants see what they are paying for.
- Cell promotion (a cell graduating from "preview" to "stable") requires a documented capacity headroom of ≥ 30% across compute / storage / network / power.

---

## Alternatives considered

### Alternative A — Pure reseller (white-label OCI / AWS forever)

- **Pros:** zero capex; fastest time-to-market.
- **Cons:** structurally subservient margin; no jurisdictional sovereignty; we cannot make residency promises that bind on the underlying provider.
- **Rejected because:** the cohesion thesis is undercut — we would not own the substrate that all other axes run on.

### Alternative B — Skip Phase 2, jump from public cloud to greenfield

- **Pros:** simpler fulfillment plumbing; one cutover instead of two.
- **Cons:** capex exposure with no operational rehearsal; we would learn DC ops on the production mega-DC.
- **Rejected because:** Phase 2 (colo) is the rehearsal phase. We learn power, cooling, network ops, vendor management, and BMS integration on someone else's building before we build our own.

### Alternative C — Phase-variant SKUs (different APIs per phase)

- **Pros:** simpler per-phase implementation.
- **Cons:** customers re-platform at every phase shift; reservation contracts become unenforceable; the FinOps surface fragments.
- **Rejected because:** phase-shift cost is the moat we are building. Letting it leak to customers destroys the moat.

### Alternative D — Single-region only until Phase 3

- **Pros:** simpler operational footprint.
- **Cons:** cannot satisfy the per-pack residency invariants (ADR-0049); cannot offer DR within KR.
- **Rejected because:** KR financial regulation requires intra-KR ≥ 30km AZ separation from day 1.

---

## Open questions

1. **Q1.** Is the Phase-2 colo footprint two sites or three? Default: three for KR (Mokdong + Pyeongchon + Chuncheon) per Financial Services Commission BCP guidance. → ADR pending in regional-pack architecture.
2. **Q2.** Does GPU SKU at Phase 1 include H200/B200-class accelerators, or restrict to H100/L40S to avoid hyperscaler waitlist risk? Default: H100/L40S only at Phase 1; H200+ enabled at Phase 2 when we control allocation. → owner: `axis-cloud`.
3. **Q3.** Edge PoP roadmap — start with KR-only (5 PoPs) or include JP/SG/US-west at Phase 1? Default: KR-only for Phase 1; JP/SG added at Phase 2 with tenant demand gate. → ADR-0049.
4. **Q4.** Does Phase 3 include a second mega-DC site simultaneously, or sequential build? Default: sequential — KR-eastcoast first, KR-westcoast 18 months later. → owner: `axis-cloud`.
5. **Q5.** Does Cloud expose a "raw colo" SKU (customer-supplied hardware) at Phase 2? Default: NO; complicates audit chain integrity and FinOps attribution. Re-evaluate at Phase 3. → ADR-0032.

---

## References

- `docs/PRD.md` §7 (cloud axis), §11 (residency)
- `docs/DESIGN.md` §4 (cloud architecture), §10 (cross-axis contracts)
- `docs/regional-packs/kr-pack.md` (KR Financial Services Commission BCP, KCMVP, ISMS-DC)
- KR 「전자금융감독규정」 §15 (data residency for financial services)
- Uptime Institute Tier-III/IV; EN 50600 series; CSA STAR-Cloud
- ADR-0001 (cohesion thesis), ADR-0002 (tenant + identity), ADR-0003 (audit chain), ADR-0007 (Cedar policy), ADR-0011 (capability registry), ADR-0032 (DCIM), ADR-0042 (observability), ADR-0043 (secrets + HSM), ADR-0044 (service mesh), ADR-0045 (database tier), ADR-0049 (residency)
