---
doc_status: published
---

# Team: Vertical — Food (Supply-Chain Compliance)

## Mission
This team owns the food vertical: food manufacturing/processing supply-chain compliance, recipe management, allergen control, nutritional labeling compliance (KR 식품표시법, US FDA nutrition label, EU 1169/2011), HACCP plan management, and upstream farm-to-processor traceability linkage. This is a **skeleton team** — scope is defined but staffing and wave timing are deferred to W-Vertical-Fan-Out. It is a close neighbor of `vertical-agriculture` (farm origin records) and shares the traceability chain at the farm → processor handoff.

## Owned axes / surfaces / contracts
- **Axis(es):** Vertical industry cloud — Food (Axis 2 sub-axis)
- **Surfaces:**
  - `vertical-food-kernel` — `FoodProduct`, `Recipe`, `Ingredient`, `AllergenDeclaration`, `NutritionFact`, `HaccpPlan`, `ProcessingLot`
  - `vertical-food-domain-*` — recipe lifecycle, allergen control, nutritional label generation, HACCP management, processing lot traceability
  - Products owned: `products/vertical-food/PRD.md` (skeleton)
- **Cross-axis contracts (DESIGN §10):**
  - `Audit-chain event` (emitter — processing lot events, HACCP corrective actions, recall events)
- **Catalog records:** `crates/vertical-food-*`
- **Runbooks:** TBD at W-Vertical-Fan-Out
- **ADRs:** TBD at activation; KR 식품위생법 + FDA FSMA + EU food-law compliance ADR

## In-scope work
- Recipe management: ingredient bills of materials, allergen declaration, nutritional calculation
- Allergen control: 14 EU major allergens, KR 18 allergens per 식품표시법, US FDA Top-9
- Nutritional label compliance: KR 영양표시, US Nutrition Facts panel, EU 1169/2011 format
- HACCP plan management: CCP identification, critical limits, monitoring, corrective actions, verification records
- Processing lot traceability: raw-material receipt → processing → packaging → distribution lot linkage
- Food recall management: lot identification, downstream notification, regulatory report
- KR 식품위생법 + 식품표시법 compliance; US FDA FSMA Preventive Controls; EU 852/2004 + 1169/2011
- Upstream linkage: farm-origin traceability records from `vertical-agriculture`

## Out-of-scope (anti-scope)
- Consumer recipe apps
- Restaurant POS (→ not a food-service vertical; use `vertical-retail` POS for food retail)
- Cloud infrastructure (→ `axis-cloud`)
- Farm-level field management (→ `vertical-agriculture`)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `axis-saas` | Workflow engine for HACCP and recall workflows | Per-release |
| `platform-audit-evidence` | Processing lot and HACCP audit records | Per event |
| `vertical-agriculture` | Farm-origin traceability records at processor boundary | Per batch |
| `ops-compliance` | KR 식품위생법 / FDA FSMA regulatory watch | Monthly |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `ops-compliance` | HACCP and food-safety certification audit evidence | Quarterly |
| `gtm-customer-success` | Food tenant supply-chain health dashboards | Monthly |

## Success metrics
*(Defined at W-Vertical-Fan-Out)*
- **Processing lot audit completeness:** 100%
- **HACCP corrective action record completeness:** 100%
- **Allergen declaration accuracy:** 100% (critical — recall risk)
- **Recall notification turnaround:** < 4 h from recall trigger to downstream notification

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council; `ops-compliance` for FSMA / 식품위생법 incidents
- Founder: as last resort (recall events involve brand risk)

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
| Allergen declaration error causes recall | Catastrophic | 100% accuracy gate; HACCP CCP monitoring |
| HACCP corrective action not recorded | High | Audit chain gate on HACCP record creation |
| Recall notification delayed | High | Recall workflow SLA < 4 h; PagerDuty alert |

## Sources scanned
PRD.md §3.1, DESIGN.md §1, DOC-CATALOG.md §2.5, products/vertical-food/PRD.md (skeleton).
