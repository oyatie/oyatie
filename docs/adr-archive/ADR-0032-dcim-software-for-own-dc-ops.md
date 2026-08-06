---
id: ADR-0032
status: Rejected
doc_status: published
---

> **Disposition light-edit (2026-08-06):** Keep Rejected: DCIM own-DC ops — optional; not on critical path for cloud-native first

# ADR-0032: DCIM software for Oyatie-owned DC operations — `crates/oya-cloud-dcops-*` with anti-scope on custom silicon

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `cloud`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0003, ADR-0007, ADR-0011, ADR-0028, ADR-0029, ADR-0042, ADR-0049

---

## Context

ADR-0028 commits the cloud microservice to a three-phase trajectory ending in greenfield Oyatie mega-DCs. From Phase 2 onward we operate physical infrastructure: rack-and-stack, power, cooling, network ops, sustainability, regulatory compliance. The industry-standard term for the software that runs a DC is **DCIM** (Data Center Infrastructure Management). Off-the-shelf DCIM (Sunbird, Nlyte, Schneider EcoStruxure, Vertiv Trellis) is built for colocation operators and enterprise IT, not for a cloud provider whose tenants are themselves multi-axis SaaS workloads. None of them consume the cohesion-thesis substrates (Tenant / Identity / Audit / Capability / Runtime / Autonomy); all of them ship their own auth, their own audit log, and their own scripting surface — which is exactly the cohesion violation pattern we exist to prevent.

We must also be explicit about anti-scope: every cloud provider that has tried to design custom silicon (NICs, switches, accelerators, optical transceivers) has burned years and capital. We adopt the discipline that we use commercial silicon only and own nothing below the OEM line. This ADR pins the in-house DCIM bounded contexts, the integration surfaces (BMS / BAS / power monitoring / cooling / physical security / asset lifecycle / capacity planning / dispatch / sustainability / regulatory), and the anti-scope.

---

## Decision

We build `crates/oya-cloud-dcops-*` as the in-house DCIM, consuming the same six cohesion substrates as every other axis. Off-the-shelf DCIM is rejected; per-vendor BMS/BAS adapters are isolated behind explicit ports.

### Bounded contexts

| Crate | Concern |
|---|---|
| `crates/oya-cloud-dcops-inventory` | Per-rack / per-PDU / per-switch / per-server inventory; per-asset lifecycle state |
| `crates/oya-cloud-dcops-capacity` | Capacity model (compute / power / cooling / network / floorspace); reservations |
| `crates/oya-cloud-dcops-power` | PDU / ATS / UPS / generator / fuel monitoring + control surface |
| `crates/oya-cloud-dcops-cooling` | Cooling-loop telemetry (CRAH / CRAC / chillers / heat exchangers); setpoint control |
| `crates/oya-cloud-dcops-sustainability` | PUE / WUE / CUE; per-region carbon accounting |
| `crates/oya-cloud-dcops-bms-bas` | Building Management / Building Automation adapter (HVAC / lighting / fire / water-leak / access control) |
| `crates/oya-cloud-dcops-network-ops` | Cable map / fiber / patch panel / cross-connect / OTDR results |
| `crates/oya-cloud-dcops-physical-security` | Badge / CCTV / mantrap / environmental sensors (vision substrate integration) |
| `crates/oya-cloud-dcops-asset-lifecycle` | Procurement → receive → install → operate → retire → erase → recycle |
| `crates/oya-cloud-dcops-thermal-planning` | Per-rack thermal model + airflow CFD seed inputs |
| `crates/oya-cloud-dcops-workorder` | Work-order ticketing + technician dispatch + parts supply |
| `crates/oya-cloud-dcops-regulatory` | Per-region regulatory pack (Uptime Tier-III/IV, EN 50600, ISMS-DC, CSA STAR-Cloud) |

### Substrate consumption

- **Tenant.** DCIM is a single-tenant (Oyatie operations) system, but it carries per-customer-tenant tagging on inventory (e.g. dedicated racks for healthcare cells).
- **Identity.** DC-ops users authenticate via the Identity kernel (ADR-0002); per-role Cedar policies (ADR-0007).
- **Audit chain.** Every DCIM mutation (rack power on/off, cooling setpoint change, badge access grant) audit-chained per ADR-0003.
- **Capability registry.** DCIM exposes capabilities (`cloud.dcops.power.cycle`, `cloud.dcops.cooling.setpoint`, `cloud.dcops.workorder.assign`) per ADR-0011; Foundry agents can consume them at the appropriate persona-tier.
- **Agent runtime.** DCIM agents (e.g. anomaly detection, capacity-forecasting) run on the Foundry runtime per ADR-0007.
- **Autonomy ceiling.** DCIM-critical actions (e.g. power-cycle a rack carrying production tenants) require persona tier ≥ `proxy` with explicit on-call human approval — autonomy ceiling is hard-coded in the capability record.

### BMS/BAS integration

```rust
// crates/oya-cloud-dcops-bms-bas/src/adapter.rs
pub trait BmsBasAdapter {
    fn read_telemetry(&self) -> Result<TelemetrySet>;
    fn write_setpoint(&self, setpoint: Setpoint, approver: ApproverIdentity) -> Result<()>;
}

pub struct SiemensDesigoAdapter { /* per-vendor */ }
pub struct HoneywellNiagaraAdapter { /* per-vendor */ }
pub struct JohnsonControlsMetasysAdapter { /* per-vendor */ }
pub struct LgEnsAdapter { /* KR-specific */ }
pub struct LsElectricAdapter { /* KR-specific */ }
```

Per-vendor adapters live behind the trait; the rest of DCIM never sees vendor specifics. KR-specific adapters (LG ENS / LS Electric) ship at Phase 2 since KR colos overwhelmingly use these.

### Power monitoring

- **Per-PDU.** Per-circuit current / voltage / power factor / kWh; per-circuit alarm thresholds.
- **Per-ATS / per-UPS.** Battery state-of-charge; transfer events; predictive replacement.
- **Per-generator.** Fuel level; runtime hours; load test schedule.
- **Per-substation feed.** KEPCO substation feed metadata; per-feed redundancy class (N / N+1 / 2N).

### Cooling control

- **Per-loop telemetry.** Supply / return temperature; flow rate; ΔT.
- **Setpoint control.** Per-zone setpoint; setpoint change requires Cedar policy + audit-chained.
- **Free-cooling economizer mode.** Per-region (KR-eastcoast geothermal / seawater eligible 8 months/year).
- **Liquid cooling readiness.** Direct-to-chip + rear-door heat-exchanger readiness for GPU SKUs (per ADR-0028).

### Network ops

- **Cable map.** Per-cable origin / destination / length / type / install-date.
- **Fiber.** Per-strand health (OTDR results); per-fiber loss budget.
- **Patch panel.** Per-port mapping; auto-discovery via LLDP + per-vendor MIB.
- **Cross-connect.** Per-MMR cross-connect inventory (Phase 2 colo); KR carrier-neutral exchange (KINX) integration.

### Physical security

- **Badge.** Per-badge access policy (Cedar-policied); per-zone access matrix.
- **CCTV.** Per-camera retention class; integration with vision substrate for anomaly detection (e.g. tailgating, after-hours rack access).
- **Mantrap.** Per-mantrap state machine; door interlock enforced.
- **Environmental sensors.** Per-rack temperature / humidity / smoke / water-leak; thresholds alarmed.

### Asset lifecycle

- **Procurement.** PO → receive → asset-tag at receive dock.
- **Install.** Per-asset install record (rack / U position / cable-in / cable-out).
- **Operate.** Per-asset health / utilization / warranty.
- **Retire.** Per-asset retirement reason; data-bearing asset erase (NIST 800-88 Purge or Destroy) + Cosign-signed proof-of-erasure per ADR-0038.
- **Recycle.** Per-region e-waste vendor; per-shipment chain-of-custody.

### Capacity + thermal planning

- **Capacity model.** Per-cell compute / power / cooling / network / floorspace headroom; alerts at 70%.
- **Thermal model.** Per-rack thermal envelope; per-aisle hot/cold separation; CFD seed inputs to per-region engineering.
- **Forecast.** ML model fed by per-microservice growth telemetry; recommends procurement quarters in advance.

### Workorder + technician dispatch

- Integrates with **Workspace Tasks** (ADR-0029) and **Workspace Calendar** (ADR-0029) — no separate ticketing UI.
- Per-workorder priority / SLA / parts requirement / technician skill match.
- Per-technician on-call rotation; per-shift handoff.

### Sustainability + carbon accounting

- **PUE / WUE / CUE per region.** Real-time + 30-day rolling.
- **Per-region carbon intensity** sourced from per-region grid operator (KEPCO for KR; per-region provider elsewhere).
- **Per-tenant carbon attribution.** Optional per-tenant report; experimental at GA.
- **GHG Protocol Scope 1/2/3** alignment; per-region CDP disclosure.

### Regulatory

- **Uptime Institute Tier-III/IV.** Per-DC certification; annual recertification.
- **EN 50600 series.** Per-DC compliance attestation.
- **KR ISMS-DC + KISA 클라우드보안인증 (CSAP).** Per-cell certification; annual audit.
- **CSA STAR-Cloud.** Per-cell certification; per-control evidence chain.

### Anti-scope: chip designer

We do not design custom silicon at any phase. Specifically forbidden:

- Custom CPU / accelerator silicon (commercial: NVIDIA / AMD / Intel / Qualcomm / Samsung).
- Custom NIC / DPU silicon (commercial: NVIDIA BlueField / AMD Pensando / Intel IPU).
- Custom switch ASIC (commercial: Broadcom Tomahawk / NVIDIA Spectrum / Intel Tofino).
- Custom optical transceiver silicon (commercial vendor).

If at Phase 3+ scale a per-component RFP returns no acceptable commercial bid, founder ratification is required to revisit this anti-scope (per ADR-0001 axis-admission protocol equivalent).

---

## Consequences

### Positive

- DCIM as a cohesion-substrate consumer means DC-ops actions are first-class auditable and policy-controlled — the same way every other axis works.
- Per-vendor BMS/BAS adapter isolation lets us swap building automation vendors per colo without rewriting DCIM.
- Anti-scope on custom silicon keeps the org focused on the moat (cohesion, multi-axis product) rather than the trap (multi-year silicon programs).
- Foundry-driven DCIM agents (capacity forecasting, thermal anomaly detection, workorder triage) ship with the same governance every other agent ships with.

### Negative

- Building DCIM in-house is a real engineering cost — 6-12 person-years for the first complete system.
- Per-vendor BMS/BAS adapter coverage at Phase 2 adds latency to colo onboarding (every new colo means an adapter audit).
- Cooling control surface is high-blast-radius; a wrong setpoint write can damage equipment. Cedar policy + autonomy ceiling must be enforced strictly.

### Operational

- Per-DC SLO catalog: PUE target ≤ 1.3 (Phase 2), ≤ 1.2 (Phase 3 KR-eastcoast); WUE target ≤ 0.5 L/kWh; uptime target Tier-III ≥ 99.982% (annual).
- Per-cell HSM partition rotation + per-DC fire-drill quarterly.
- Per-vendor BMS/BAS adapter regression test set runs nightly against vendor sandbox.
- Per-asset retirement audit chain reviewed monthly; proof-of-erasure published to Trust Portal per ADR-0038.
- DCIM lane: `oya-governance-dcim-substrate` enforces no DCIM action escapes the audit chain.

---

## Alternatives considered

### Alternative A — Off-the-shelf DCIM (Sunbird / Nlyte / Schneider EcoStruxure / Vertiv Trellis)

- **Pros:** faster to first-DC.
- **Cons:** does not consume substrates; ships own auth + audit; per-vendor lock-in; cannot expose cohesion-grade capabilities to Foundry agents.
- **Rejected because:** the cohesion violation is structural.

### Alternative B — DCIM as a non-cohesion appliance (DCIM in its own bounded subnet, not consuming substrates)

- **Pros:** isolation simpler.
- **Cons:** DC-ops actions are exempt from the cohesion contract; we would have one set of audit guarantees for tenant workloads and a different set for the substrate they run on.
- **Rejected because:** DC-ops actions have higher blast radius than tenant workloads; they need *more* governance, not less.

### Alternative C — Allow custom silicon at Phase 3+

- **Pros:** unit economics ceiling raised.
- **Cons:** every cloud provider that has tried this has burned 5-10 years and billions of dollars; the unit-economics gain materializes only at scales we will not reach for a decade.
- **Rejected because:** focus on the moat; revisit only at founder ratification.

---

## Open questions

1. **Q1.** Phase-2 colo BMS/BAS — start with Siemens Desigo or Honeywell Niagara as primary adapter? Default: Siemens (KR colo penetration). → owner: `cloud`.
2. **Q2.** Vision substrate (CCTV anomaly detection) — Foundry capability or DCIM-internal? Default: Foundry capability per ADR-0007 governance. → ADR-0011.
3. **Q3.** Per-tenant carbon attribution at GA or W+24? Default: experimental at GA, GA at W+24. → owner: `cloud`.
4. **Q4.** Liquid-cooling at Phase 2 (some colos) or Phase 3 only? Default: Phase 2 for GPU SKUs in colos that support rear-door HX; full direct-to-chip Phase 3. → ADR-0028.
5. **Q5.** Anti-scope revisit cadence? Default: annual founder review at end of Phase 2. → owner: `cloud`.

---

## References

- `docs/PRD.md` §7 (cloud axis), §10 (sustainability)
- `docs/DESIGN.md` §4 (cloud), §10 (cross-microservice contracts)
- Uptime Institute Tier-III/IV; EN 50600 series; ASHRAE TC9.9 thermal guidelines
- KR ISMS-DC; KISA 클라우드보안인증 (CSAP); CSA STAR-Cloud
- NIST 800-88 Rev 1 (media sanitization); GHG Protocol Scope 1/2/3
- ADR-0001 (cohesion), ADR-0003 (audit), ADR-0007 (Cedar + persona tier), ADR-0011 (capability registry), ADR-0028 (cloud trajectory), ADR-0029 (workspace tasks/calendar integration), ADR-0042 (observability), ADR-0049 (residency)
