---
doc_status: published
---

# Team: Vertical — Industrial (MES / OEE / ISA-95 / OPC UA)

## Mission
This team owns the industrial vertical: Manufacturing Execution Systems (MES), Overall Equipment Effectiveness (OEE), ISA-95 production hierarchy, OPC UA device integration, SCADA historian, and OT/IT boundary safety controls. It exists because industrial tenants operate physical equipment where a software fault can cause safety incidents, making auditability, autonomy ceiling enforcement, and OT-network isolation as critical as in healthcare. It does **not** own cloud compute infrastructure or the SaaS workflow engine.

## Owned axes / surfaces / contracts
- **Axis(es):** Vertical industry cloud — Industrial (Axis 2 sub-axis)
- **Surfaces:**
  - `vertical-industrial-kernel` — `WorkOrder`, `ProductionLot`, `Equipment`, `OeeRecord`, `AlarmEvent`, `Shift`
  - `vertical-industrial-domain-*` — MES execution, OEE calculation, work-order lifecycle, alarm management
  - `vertical-industrial-adapter-opcua` — OPC UA server/client adapter (read-only by default; write requires autonomy ceiling approval)
  - `vertical-industrial-adapter-isa95` — ISA-95 production hierarchy, B2MML mapping
  - Products owned: `products/vertical-industrial/PRD.md`
- **Cross-axis contracts (DESIGN §10):**
  - `Audit-chain event` (emitter — work-order completion, alarm events, OT write-commands)
  - `Autonomy ceiling policy` (consumer — OT write commands require explicit ceiling approval)
- **Catalog records:** `crates/vertical-industrial-*`
- **Runbooks:** `runbooks/industrial-ot-write-emergency-stop.md`, `runbooks/opcua-adapter-disconnect.md`
- **ADRs:** ADR-0033 (industrial OT/IT boundary)

## In-scope work
- MES: work-order scheduling, dispatch, execution tracking, material consumption, quality inspection
- OEE: availability, performance, quality calculation; downtime categorization; shift reporting
- ISA-95 hierarchy: enterprise → site → area → work-center → work-unit mapping
- OPC UA integration: device telemetry read, setpoint write (autonomy-ceiling-gated), alarm subscription
- SCADA historian adapter: time-series ingest, tag namespace, data retention
- OT/IT network isolation: no direct OT-network egress; all writes mediated by approved capabilities
- Alarm management: ISA-18.2 compliant alarm rationalization, acknowledgment, escalation
- Production analytics: yield, scrap, throughput, energy efficiency (analytics plane)
- KR industrial regulatory pack: 산업안전보건법 (Occupational Safety and Health Act) compliance artifacts

## Out-of-scope (anti-scope)
- Hardware PLC programming (Oyatie interfaces with, does not program PLCs)
- Cloud infrastructure (→ `axis-cloud`)
- SaaS workflow engine (→ `axis-saas`)
- OT network management (customer-operated; Oyatie is an IT-side integration)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `platform-audit-evidence` | OT write-command audit chain records | Per OT write |
| `axis-foundry` | Autonomy ceiling enforcement for OT write capabilities | Per OT write |
| `axis-saas` | Workflow engine for work-order lifecycle | Per-release |
| `ops-compliance` | KR 산업안전보건법 regulatory watch | Monthly |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `ops-compliance` | Safety and OT audit evidence packs | Quarterly |
| `gtm-customer-success` | Industrial tenant OEE dashboards | Monthly |

## Success metrics
- **OT write commands without autonomy ceiling approval:** 0 (hard zero)
- **MES work-order cycle time accuracy:** ≥ 99% match to physical floor clock
- **OEE calculation audit completeness:** 100%
- **OPC UA adapter uptime:** ≥ 99.9%
- **Industrial regulatory evidence pack regeneration:** ≤ 4 h (PRD §4.2)

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council for OT/IT boundary contract changes; `ops-security` for OT write incidents
- Compliance: `ops-compliance` for safety regulatory incidents
- Founder: as last resort

## Communication cadence
- Stand-up: daily async
- Weekly: 45-min sync — OPC UA adapter health, OEE metrics, autonomy-ceiling OT write queue
- Cross-team review: quarterly safety review with `ops-security`

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules; OT write PRs require security-reviewer
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch; OT/IT boundary (ADR-0033) amendments are P0

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| OT write command issued without autonomy ceiling approval | Catastrophic | Cedar policy gate; audit chain mandatory; emergency stop runbook |
| OPC UA adapter vulnerability exposes OT network | Catastrophic | Read-only by default; write-path security review; `ops-security` quarterly OT audit |
| MES data inaccuracy causes incorrect production decisions | High | Work-order audit chain; reconciliation against physical floor records |

## Sources scanned
PRD.md §3.1, DESIGN.md §10, ADR-0033, products/vertical-industrial/PRD.md (draft).
