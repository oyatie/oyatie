---
id: ADR-0028
status: Superseded
superseded_by: [ADR-700]
doc_status: published
---

# ADR-0028: Cloud microservice — compute substrate with stable product surface across three infrastructure phases

> **Status:** Accepted
> **Owner:** `oya-cloud`
> **Date:** 2026-05-09 (rewritten 2026-05-13 — Cloud is a flat µservice, not an "axis")
> **Related:** ADR-0001, ADR-0002, ADR-0003, ADR-0007, ADR-0011, ADR-0032, ADR-0042, ADR-0043, ADR-0044, ADR-0045, ADR-0049, ADR-0058

---

## Context

Cloud is the compute substrate microservice in the flat catalog. Like every other microservice in the catalog, Cloud is independent, modular, and consumed by other microservices via the shared substrate layer. Cloud owns physical reality: racks, fiber, transformer redundancy, KEPCO substations, refrigerant loops, and per-cell HSM partitions. All other microservices run on Cloud cells.

The cohesion thesis (ADR-0001) requires that Cloud expose the same Tenant + Identity + Audit + Capability + Runtime + Autonomy substrates as every other microservice. Cloud does not re-implement those substrates; it consumes them.

Cloud's trajectory moves through three infrastructure phases while maintaining a phase-invariant product surface. Tenants buy capacity now and re-platform never.

---

## Decision

We adopt a **three-phase compute trajectory** with a **phase-invariant product surface**. Customers consume the same APIs, the same SKUs, the same IAM model, and the same audit shape regardless of whether the underlying capacity is rented, leased in a colo, or owned in a mega-DC.

**Naming justification (BNF v4.1, ADR-0056):**
- Cloud µservice crates: `oya-cloud-<bc>-<layer>` where `cloud` is the registered µservice name
- Examples: `oya-cloud-compute-kernel`, `oya-cloud-storage-adapter`, `oya-cloud-billing-rest`, `oya-cloud-iam-kernel`

### Phases

- **Phase 1 — Public-cloud consumption (initial).** Primary: OCI KR-Seoul region 1; secondary: OCI KR-Chuncheon; fail-open: AWS ap-northeast-2. All capacity rented; per-tenant cells as Kubernetes namespaces + dedicated node pools per data class.
- **Phase 2 — Hybrid Oyatie colo.** Primary KR colo lease (Equinix SL1/SL2 + KT IDC Mokdong + LG U+ Pyeongchon, three-site quorum). 60% steady-state on owned hardware; bursting + DR remain on Phase 1. Per-cell HSM partitions land here per ADR-0043.
- **Phase 3 — Greenfield Oyatie mega-DC.** Single-tenant 30-50MW campus, KR-eastcoast primary. Phase 1 capacity remains as permanent edge PoPs.

### Phase-invariant product surface

```rust
// oya-cloud-surface-kernel
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

- **Compute SKUs** are defined once and survive all three phases. Phase 1 maps `BareMetalLease` to OCI Bare Metal shapes; Phase 2/3 map it to Oyatie-owned racks. Same SKU, same SLA, different fulfillment.
- **Storage:** Object (S3-compatible), Block (NVMe-class IOPS tiers), File (NFSv4.1 / SMB3), Archive (cold), plus database tier per ADR-0045.
- **Network:** VPC (per-tenant per-cell), Load Balancer (L4 + L7 with mTLS), DNS (per-tenant zones + DNSSEC), Interconnect, DDoS protection, Service Mesh (per ADR-0044).
- **IAM:** Cedar-policy gated (ADR-0007); federates SSO (SAML2 + OIDC); issues STS-style short-lived credentials; emits to audit chain (ADR-0003).
- **Regions, AZs, Cells.** Day 1: KR-Seoul1 (3 AZs, each ≥ 30km separation per KR FSC DR guidance). A cell is a tenant-isolation unit within an AZ.
- **Billing:** per-resource per-tenant; KR 세금계산서 format mandatory for KR cells.
- **FinOps:** per-microservice cost attribution, per-cell unit-economics, reservation recommendations, cost-anomaly detection.

### Anti-scope

Cloud does not ship its own tenant model, identity surface, or audit emitter — those are shared substrates from ADR-0002 / ADR-0003. Cloud does not ship custom silicon, custom switching ASICs, or custom optical transceivers.

---

## Consequences

### Concrete crate layout (BNF v4.1)

```
oya-cloud-surface-kernel        — phase-invariant SKU types + ports
oya-cloud-compute-kernel        — compute port traits
oya-cloud-compute-adapter       — OCI/AWS/bare-metal fulfillment impls
oya-cloud-storage-kernel        — storage port traits
oya-cloud-storage-adapter       — S3-compat + block + file impls
oya-cloud-storage-sdk           — public_layers SDK (consumed by other µservices)
oya-cloud-network-kernel        — VPC + LB + DNS port traits
oya-cloud-network-adapter       — network impls
oya-cloud-iam-kernel            — IAM port traits (Cedar-gated)
oya-cloud-billing-kernel        — billing domain types
oya-cloud-billing-rest          — billing API
oya-cloud-finops-domain         — cost attribution logic
oya-cloud-cell-kernel           — cell routing types
oya-cloud-dcops-kernel          — DCIM types for Phase 2/3 (ADR-0032)
```

`cloud` is registered in `[workspace.metadata.oya.microservices]`. The `sdk` layer is declared in `public_layers = ["sdk"]` (per ADR-0056 Cloud dual-role mechanism), allowing other µservices to depend on `oya-cloud-storage-sdk` and `oya-cloud-compute-sdk`.

### Positive

- Customers buy capacity now and re-platform never.
- Phase-shift risk is bounded to Cloud-internal fulfillment plumbing; product teams above Cloud do not block on greenfield.
- KR data residency (ADR-0049) satisfied from day 1 via OCI KR-Seoul1.

### Negative

- Phase-invariant constraint slows Phase 1; hyperscaler-only SKUs are prohibited.
- Phase-2 colo financials exposed to KEPCO industrial tariff volatility.
- DC ops hiring required earlier than a software-only company.

### Operational

- `oya-check-cloud-surface` lane: any PR adding a hyperscaler-only SKU is rejected unless it ships the colo + greenfield path.
- Per-cell HSM partition rotation on quarterly drill calendar (ADR-0043).
- DR drill: full-AZ failure quarterly; full-region failure annually.

---

## Alternatives considered

### Alternative A — Pure reseller (white-label OCI/AWS forever)

- **Rejected because:** no jurisdictional sovereignty; cannot make residency promises that bind on the underlying provider; cohesion thesis undercut.

### Alternative B — Skip Phase 2, jump to greenfield

- **Rejected because:** Phase 2 (colo) is the operational rehearsal. We learn DC ops on someone else's building before building our own.

---

## Related

- ADR-0001 (cohesion thesis — Cloud is a µservice in the flat catalog)
- ADR-0002 (tenant + identity substrates)
- ADR-0003 (audit chain)
- ADR-0032 (DCIM for Phase 2/3)
- ADR-0043 (HSM + KMS)
- ADR-0044 (service mesh)
- ADR-0045 (database tier)
- ADR-0049 (residency)
- ADR-0056 (BNF v4.1 — Cloud dual-role + public_layers)
- ADR-0058 (Flat microservice catalog)
- `[[feedback-flat-product-catalog]]` — Cloud is a shared µservice substrate, not an axis
