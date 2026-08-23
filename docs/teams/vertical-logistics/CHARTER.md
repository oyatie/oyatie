---
doc_status: published
---

# Team: Vertical — Logistics (Shipment / Dock / EDI / Route)

## Mission
This team owns the logistics vertical: shipment lifecycle, dock scheduling, EDI transaction processing (214/990/997 and KR customs), route optimization, Hours of Service (HOS) compliance, and cold-chain monitoring. It exists because logistics tenants require real-time supply-chain visibility with regulatory compliance across customs, freight, and carrier regulations — and the audit chain must cover every cross-border data flow. It does **not** own cloud infrastructure, the SaaS workflow engine, or the underlying eventing backbone.

## Owned axes / surfaces / contracts
- **Axis(es):** Vertical industry cloud — Logistics (Axis 2 sub-axis)
- **Surfaces:**
  - `vertical-logistics-kernel` — `Shipment`, `ShipmentLeg`, `DockAppointment`, `Route`, `Carrier`, `HosLog`, `ColdChainRecord`
  - `vertical-logistics-domain-*` — shipment lifecycle, dock scheduling, EDI transaction processing, route optimization
  - `vertical-logistics-adapter-edi` — EDI 214 (shipment status), 990 (response), 997 (functional ack), KR 관세청 customs EDI
  - Products owned: `products/vertical-logistics/PRD.md`
- **Cross-axis contracts (DESIGN §10):**
  - `Audit-chain event` (emitter — cross-border shipments, customs declarations)
  - `DSR / consent withdrawal cascade` (ack required — driver PII in HOS logs)
- **Catalog records:** `crates/vertical-logistics-*`
- **Runbooks:** `runbooks/logistics-edi-failure.md`, `runbooks/cold-chain-breach-alert.md`
- **ADRs:** logistics compliance ADR (to be authored)

## In-scope work
- Shipment lifecycle: order receipt, pick/pack, dispatch, in-transit tracking, delivery confirmation, returns
- Dock scheduling: appointment booking, dock-door assignment, yard management, labor planning
- EDI processing: 214 (shipment status), 990 (response to tender), 997 (functional acknowledgment), 856 (advance ship notice), KR 관세청 customs EDI
- Route optimization: TSP/VRP solver, multi-stop, time-window constraints, carrier selection
- HOS compliance: US FMCSA Hours of Service, driver log, ELD integration
- Cold-chain: temperature telemetry ingest, breach alert, compliance attestation
- Carrier integration: KR parcel carriers (CJ대한통운, 롯데, 한진), global (FedEx, UPS, DHL) via adapter
- KR customs: 수출입 신고, 관세청 EDI, HS code classification

## Out-of-scope (anti-scope)
- Fleet hardware management (customer-operated)
- Cloud infrastructure (→ `axis-cloud`)
- Consumer last-mile apps

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `platform-audit-evidence` | Cross-border shipment and customs audit records | Per shipment |
| `axis-saas` | Workflow engine for shipment lifecycle | Per-release |
| `axis-foundry` | Route optimization capabilities under autonomy ceiling | Wave gate |
| `ops-compliance` | KR customs regulatory watch, US FMCSA HOS changes | Monthly |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `ops-compliance` | Customs and carrier audit evidence | Quarterly |
| `gtm-customer-success` | Logistics tenant SLA dashboards | Monthly |

## Success metrics
- **EDI 997 functional ack turnaround:** < 1 h from receipt
- **On-time shipment tracking event completeness:** ≥ 99.5%
- **Cold-chain breach detection latency:** < 5 min from telemetry to alert
- **KR customs EDI submission accuracy:** 100%
- **Driver PII DSR completion:** 100% within 72 h

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council for logistics EDI schema contract changes
- Compliance: `ops-compliance` for customs regulatory incidents
- Founder: as last resort

## Communication cadence
- Stand-up: daily async
- Weekly: 30-min sync — EDI health, customs queue, cold-chain alerts
- Cross-team review: monthly compliance review with `ops-compliance`

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| KR customs EDI format change breaks submission | High | Monthly regulatory watch; EDI adapter versioning |
| Cold-chain breach not detected in time | High | Telemetry ingest SLO < 5 min; PagerDuty alert |
| Driver HOS log PII leaked | High | HOS logs classified as `internal_only`; DSR cascade covered |

## Sources scanned
PRD.md §3.1, DESIGN.md §10, DOC-CATALOG.md §2.5, products/vertical-logistics/PRD.md (draft).
