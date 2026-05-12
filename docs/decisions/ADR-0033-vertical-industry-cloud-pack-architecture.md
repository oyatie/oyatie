# ADR-0033: Vertical industry cloud pack architecture — per-vertical bounded contexts with kernel/domain/app/adapter/api/worker/runtime layers

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `axis-vertical`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0002, ADR-0003, ADR-0007, ADR-0011, ADR-0028, ADR-0029, ADR-0034, ADR-0049

---

## Context

Axis 3 (Vertical industry cloud) is the axis with the highest per-tenant differentiation requirement. A healthcare tenant has FHIR resources, encounter workflows, EMR-side compliance constraints; a manufacturing tenant has work-orders, OEE telemetry, MES integrations; a logistics tenant has shipments, lanes, customs filings; a fintech tenant has loans, payments, KYC chains. None of these can be served by a single generic SaaS shell — but if each vertical ships its own everything, the cohesion thesis collapses and we have built seven products instead of one.

The pack-of-19 foundation ADRs named Vertical as an axis but did not pin the per-vertical bounded-context layout, the per-vertical canonical entity model, the per-region extension mechanism, or the cohesion-substrate consumption pattern. This ADR pins them so that adding a new vertical (e.g. construction-tech, agritech, edtech) is a structured operation, not an ad-hoc subtree.

---

## Decision

We adopt a **seven-layer per-vertical bounded context** under `crates/oya-vertical-<name>-*` for every vertical we ship. Each vertical consumes the six cohesion substrates from ADR-0001 plus a per-vertical regulatory pack from the regional-pack architecture; per-region extensions plug into the vertical kernel via trait impls.

### Seven-layer layout

| Layer | Crate naming | Concern |
|---|---|---|
| **kernel** | `crates/oya-vertical-<name>-kernel` | Canonical entity model + domain invariants + per-region extension trait surface |
| **domain** | `crates/oya-vertical-<name>-domain` | Domain logic (use cases / aggregates / domain services) |
| **app** | `crates/oya-vertical-<name>-app` | Application orchestration (transactions / sagas / app services) |
| **adapter** | `crates/oya-vertical-<name>-adapter-*` | External-system adapters (one crate per integration: EMR / MES / TMS / KYC vendor) |
| **api** | `crates/oya-vertical-<name>-api` | Public API (REST + gRPC + GraphQL surfaces; OpenAPI / proto / SDL artifacts) |
| **worker** | `crates/oya-vertical-<name>-worker` | Background jobs + scheduled tasks + event consumers |
| **runtime** | `crates/oya-vertical-<name>-runtime` | Per-vertical capability registrations + per-vertical persona-tier defaults |

### Per-vertical canonical entity model

Each vertical pins its canonical entity model in its kernel crate. The model is the single source of truth; per-region extensions override fields, never replace the entity.

| Vertical | Canonical entities | Standards alignment |
|---|---|---|
| **Healthcare** | `Patient`, `Encounter`, `Observation`, `Condition`, `Procedure`, `MedicationRequest`, `DiagnosticReport`, `Practitioner`, `Organization`, `Location` | FHIR R5 |
| **Industrial / manufacturing** | `WorkOrder`, `Asset`, `MaintenanceTask`, `OeeRecord`, `QualityInspection`, `BillOfMaterials`, `Shift`, `Operator` | ISA-95 + MIMOSA + OEE |
| **Logistics** | `Shipment`, `Lane`, `Carrier`, `Vehicle`, `Driver`, `Hub`, `Stop`, `CustomsFiling`, `BillOfLading` | UN/EDIFACT + EPCIS |
| **Fintech** | `Loan`, `Payment`, `Account`, `Transaction`, `Counterparty`, `KycRecord`, `AmlAlert`, `Statement`, `Disbursement` | ISO 20022 + FAPI |
| **Public-sector** | `Citizen`, `Service`, `Application`, `Decision`, `Disbursement`, `Disclosure` | per-jurisdiction (KR 행정안전부 표준) |
| **Education-K12** | `Student`, `Class`, `Lesson`, `Assignment`, `Grade`, `Teacher`, `School`, `Guardian` | per-region SIS standard |
| **Education-HE** | `Student`, `Course`, `Enrollment`, `Grade`, `Faculty`, `Term`, `Program`, `ResearchProject` | per-region HE standard |
| **Defense** | _(anti-scope: see ADR-0034 / founder ratification gate)_ | _gated_ |
| **Energy/utilities** | `Meter`, `MeterReading`, `Outage`, `Asset`, `WorkOrder`, `RatePlan`, `Bill`, `Disconnect` | IEC 61970/61968 |
| **Retail** | `Product`, `Sku`, `Order`, `Customer`, `Inventory`, `Store`, `Promotion`, `Return` | GS1 + EPCIS |

### Per-region extensions plug into vertical kernel

```rust
// crates/oya-vertical-healthcare-kernel
pub trait RegionExtension {
    fn region_id(&self) -> RegionId;
    fn patient_identifier_format(&self) -> IdentifierFormat;
    fn mandatory_classifications(&self) -> Vec<ClassificationCode>;
    fn billing_code_system(&self) -> CodeSystem;
}

// regional-packs/kr/healthcare/src/extension.rs
pub struct KrHealthcareExtension; // 주민등록번호 + KCD-8 + 건강보험심사평가원 EDI

impl RegionExtension for KrHealthcareExtension { /* ... */ }
```

The vertical kernel never knows which region's extension is active; it asks the trait.

### Cohesion: every vertical consumes the same six substrates plus the same axis surfaces

Every vertical:

- Consumes the **Tenant + Identity** kernels (ADR-0002).
- Consumes the **Audit chain** (ADR-0003) for every domain mutation.
- Registers its capabilities in the **Capability registry** (ADR-0011).
- Runs Foundry agents on the **Agent runtime** (ADR-0007) at the **Autonomy ceiling** (ADR-0007).
- Runs on **Cloud** (ADR-0028) cells.
- Stores state in **Database tier** (ADR-0045) per the data-tier matrix.
- Reaches users via **Workspace** apps (ADR-0029) when appropriate (e.g. healthcare scheduling reaches users via Workspace Calendar).
- Sources ads via the **Ads gate** (ADR-0031) when applicable, respecting **per-vertical hard-deny** (ADR-0034).
- Routes search via **Search bridge** (ADR-0030) for tenant-private enterprise search.

### Vertical anti-scope

A vertical does not ship:

- Its own tenant model, identity surface, audit chain, capability registry, agent runtime, autonomy ceiling.
- Its own observability stack (uses ADR-0042).
- Its own database (uses ADR-0045).
- Its own ad surface (uses ADR-0031, gated by ADR-0034 hard-deny).
- A workflow engine (uses ADR-0035).

### Vertical admission protocol

Adding a new vertical requires:

1. An ADR in this pack (or downstream pack) naming the vertical, its canonical entity model, its day-1 region pack, and its data-class overrides (per ADR-0034).
2. A capability inventory at the kernel layer (per ADR-0011).
3. A per-vertical regulatory pack binding (per regional-pack architecture).
4. A persona-tier default policy (per ADR-0007).
5. Cohesion fitness lane confirmation that the vertical does not re-implement any substrate.

### Vertical lifecycle states

- **Proposed.** ADR drafted; no code yet.
- **Preview.** Kernel + domain + minimum app surface; one tenant; data-class overrides enforced.
- **Stable.** Per-region adapter coverage complete; SLO catalog published; trust portal entry.
- **GA.** Multi-region; multi-tenant; per-deprecation telemetry per ADR-0037.
- **Deprecated.** Per ADR-0038 sunset cascade.

---

## Consequences

### Positive

- A new vertical onboards via a structured 7-layer template; no ad-hoc subtrees.
- Per-region extensions are plug-in (trait impls), not forks of the vertical.
- Canonical entity model alignment with industry standards (FHIR / ISA-95 / EPCIS / ISO 20022) makes integration with existing tenant systems achievable.
- Cohesion is mechanically enforced: a vertical that re-implements (e.g.) its own audit chain fails the fitness lane.

### Negative

- Seven-layer layout is more crates per vertical than a flat layout; the flat-crates target (ADR-0015 equivalent) absorbs this cost intentionally.
- Industry-standard alignment is a real cost (FHIR is large; ISA-95 is large; ISO 20022 is huge); per-vertical kernel teams must own the mapping.
- Per-region extensions multiply the test surface (vertical × region matrix).

### Operational

- Per-vertical SLO catalog includes domain-relevant metrics (e.g. healthcare: claim-submission latency; manufacturing: OEE recompute latency).
- Per-vertical regulatory pack audit (annual; per-region).
- Per-vertical capability inventory reviewed at each release per ADR-0037 stability tiers.
- Per-vertical migration tooling from incumbent vendors (e.g. healthcare: Epic / Cerner / 의료법인 EMR; fintech: per-bank legacy core).

---

## Alternatives considered

### Alternative A — Single generic vertical shell with per-vertical config files

- **Pros:** less per-vertical code.
- **Cons:** verticals are not configurations of a generic; they are different domains. Forcing them into a generic shell either over-generalizes or produces a shell so leaky each vertical is a fork in practice.
- **Rejected because:** domain modeling is the moat for vertical packs; over-generalization destroys it.

### Alternative B — Per-vertical full-stack (own substrate, own runtime, own everything)

- **Pros:** per-vertical team independence.
- **Cons:** cohesion thesis violated immediately; we would have N-vertical products.
- **Rejected because:** the cohesion moat is the whole point.

### Alternative C — Per-region-then-per-vertical (region first, vertical second)

- **Pros:** matches some regulatory authorities' jurisdictional thinking.
- **Cons:** verticals are durable, regions are extensions; flipping the hierarchy makes the canonical entity model fragmented across regions.
- **Rejected because:** the entity model is the durable axis.

---

## Open questions

1. **Q1.** Day-1 verticals at GA: healthcare + industrial + logistics + fintech, or also K12-education? Default: HC + IND + LOG + FIN at GA; K12 in W+12. → ADR-0034.
2. **Q2.** Defense vertical anti-scope — keep at founder ratification only, or define a "civilian-defense" subset (e.g. coast guard logistics) at lower gate? Default: founder ratification only at GA. → ADR-0034.
3. **Q3.** Per-vertical UI shell — share Workspace UI components or per-vertical custom UI? Default: share Workspace components for generic surfaces; per-vertical custom for domain-heavy surfaces. → owner: `axis-vertical`.
4. **Q4.** Per-vertical SDK in Python / TS / Go for tenant developers? Default: YES; auto-generated from per-vertical API contract per ADR-0037. → ADR-0037.
5. **Q5.** Per-vertical kernel testing requires per-region fixture corpus; per-region fixture authoring is owned by which team? Default: per-region pack team owns fixture; vertical kernel team owns trait surface. → owner: `axis-vertical` + regional-pack owners.

---

## References

- `docs/PRD.md` §7 (vertical axis), §11 (per-vertical residency)
- `docs/DESIGN.md` §4 (vertical architecture), §10 (cross-axis contracts)
- HL7 FHIR R5; ISA-95 / MIMOSA; UN/EDIFACT + EPCIS; ISO 20022; IEC 61970/61968; GS1
- KR 「의료법」, 「자본시장법」, 「전자금융거래법」, 「화학물질관리법」, 「약사법」, 「유아교육법」, 「초·중등교육법」
- ADR-0001 (cohesion), ADR-0002 (tenant + identity), ADR-0003 (audit), ADR-0007 (Cedar + persona tier), ADR-0011 (capability registry), ADR-0028 (cloud), ADR-0029 (workspace), ADR-0034 (per-vertical data class overrides), ADR-0035 (workflow engine), ADR-0037 (API stability tiers), ADR-0038 (DSR cascade), ADR-0042 (observability), ADR-0045 (database tier), ADR-0049 (residency)
