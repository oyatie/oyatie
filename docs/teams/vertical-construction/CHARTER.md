---
doc_status: published
---

# Team: Vertical — Construction (Project Management)

## Mission
This team owns the construction vertical: project management, bid management, subcontractor coordination, RFI/submittal workflows, punch-list, and safety incident tracking. This is a **skeleton team** — scope is defined but staffing and wave timing are deferred to W-Vertical-Fan-Out.

## Owned axes / surfaces / contracts
- **Axis(es):** Vertical industry cloud — Construction (Axis 2 sub-axis)
- **Surfaces:**
  - `vertical-construction-kernel` — `Project`, `SubmittalPackage`, `Rfi`, `PunchItem`, `SafetyIncident`, `SubcontractorBid`
  - `vertical-construction-domain-*` — project lifecycle, bid management, submittal/RFI workflow, punch-list
  - Products owned: `products/vertical-construction/PRD.md` (skeleton)
- **Cross-axis contracts (DESIGN §10):**
  - `Audit-chain event` (emitter — safety incidents, contract changes)
- **Catalog records:** `crates/vertical-construction-*`
- **Runbooks:** TBD at W-Vertical-Fan-Out
- **ADRs:** TBD at activation

## In-scope work
- Project management: WBS, schedule (CPM/PERT), cost management, earned value
- Bid management: ITB, bid submission, bid leveling, award workflow
- Submittal/RFI: document routing, review cycle, approval tracking
- Punch-list: deficiency tracking, closeout, warranty management
- Safety: incident reporting, OSHA compliance (US), KR 산업안전보건법, near-miss tracking
- Document management: drawing version control, BIM integration (IFC file handling)

## Out-of-scope (anti-scope)
- BIM authoring software (Oyatie integrates with, does not replace Autodesk/Revit)
- Cloud infrastructure (→ `axis-cloud`)
- Hardware procurement tracking (materials only; equipment is out of scope)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `axis-saas` | Workflow engine for submittal/RFI/bid workflows | Per-release |
| `platform-audit-evidence` | Safety incident and contract audit records | Per event |
| `ops-compliance` | OSHA / KR 산업안전보건법 regulatory watch | Monthly |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `ops-compliance` | Safety incident audit evidence | Quarterly |

## Success metrics
*(Defined at W-Vertical-Fan-Out)*
- **Safety incident audit completeness:** 100%
- **Submittal review cycle time:** ≤ contract SLA
- **RFI response audit completeness:** 100%

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council; `ops-compliance` for safety regulatory incidents
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
| Safety incident not captured in audit chain | High | Audit chain emission gate on safety incident creation |

## Sources scanned
PRD.md §3.1, DESIGN.md §1, DOC-CATALOG.md §2.5, products/vertical-construction/PRD.md (skeleton).
