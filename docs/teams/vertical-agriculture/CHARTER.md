---
doc_status: published
---

# Team: Vertical — Agriculture (Traceability)

## Mission
This team owns the agriculture vertical: crop/livestock traceability, field management, food-safety compliance (KR 농산물이력제, US FDA FSMA, EU Farm-to-Fork), and precision agriculture integrations (IoT sensor telemetry, weather data). This is a **skeleton team** — scope is defined but staffing and wave timing are deferred to W-Vertical-Fan-Out.

## Owned axes / surfaces / contracts
- **Axis(es):** Vertical industry cloud — Agriculture (Axis 2 sub-axis)
- **Surfaces:**
  - `vertical-agriculture-kernel` — `Farm`, `Field`, `Crop`, `LivestockBatch`, `HarvestRecord`, `TraceabilityLot`, `ChemicalApplication`
  - `vertical-agriculture-domain-*` — crop lifecycle, traceability chain, chemical-application recording, harvest reporting
  - Products owned: `products/vertical-agriculture/PRD.md` (skeleton)
- **Cross-axis contracts (DESIGN §10):**
  - `Audit-chain event` (emitter — traceability lot events, chemical application records, food-safety certification events)
- **Catalog records:** `crates/vertical-agriculture-*`
- **Runbooks:** TBD at W-Vertical-Fan-Out
- **ADRs:** TBD at activation; KR 농산물이력제 + FSMA compliance ADR

## In-scope work
- Crop traceability: field → harvest → pack → ship lot tracking (KR 농산물이력제, FSMA Produce Safety Rule, EU Farm-to-Fork)
- Livestock traceability: animal ID, movement records, veterinary treatment records (KR 가축이력제)
- Field management: planting, fertilization, irrigation, chemical application records (with required waiting periods)
- Precision agriculture: IoT sensor telemetry ingest (soil moisture, temperature, drone imagery), weather data integration
- Food-safety compliance: KR 안전관리인증기준 (HACCP), FDA FSMA, EU 852/2004
- Supply chain: farm → packer → distributor → retailer traceability chain linkage

## Out-of-scope (anti-scope)
- Consumer food-traceability apps (B2B farmers and food producers only)
- Commodity trading platform
- Cloud infrastructure (→ `axis-cloud`)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `axis-saas` | Workflow engine for crop lifecycle and compliance workflows | Per-release |
| `platform-audit-evidence` | Traceability lot and food-safety audit records | Per event |
| `ops-compliance` | KR 농산물이력제 / FDA FSMA regulatory watch | Monthly |
| `vertical-food` | Shared traceability chain linkage at farm → processor boundary | Per onboard |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `vertical-food` | Farm-origin traceability records for food processing compliance | Per batch |
| `ops-compliance` | Food-safety certification audit evidence | Quarterly |

## Success metrics
*(Defined at W-Vertical-Fan-Out)*
- **Traceability lot audit completeness:** 100%
- **HACCP compliance record completeness:** 100%
- **KR 농산물이력제 traceability query response time:** < 2 s

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council; `ops-compliance` for FSMA / HACCP incidents
- Founder: as last resort

## Communication cadence
- Stand-up: async (skeleton phase)
- Weekly: 30-min sync at W-Vertical-Fan-Out activation

## Bandwidth + hiring
- Current FTE: 0 (skeleton)
- Target FTE: TBD per axis-wave
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch once active

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Traceability chain gap causes food recall audit failure | High | 100% lot audit completeness; HACCP record gate |
| Chemical-application waiting period not enforced | High | Waiting-period rule engine in domain; audit trail |

## Sources scanned
PRD.md §3.1, DESIGN.md §1, DOC-CATALOG.md §2.5, products/vertical-agriculture/PRD.md (skeleton).
