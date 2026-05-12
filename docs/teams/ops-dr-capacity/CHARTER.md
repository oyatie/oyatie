# Team: Ops — DR & Capacity

## Mission
This team owns disaster recovery planning, capacity modelling, and region-failover automation across all Oyatie axes. It exists because horizontal scalability and region failover are structural invariants per PRD §3.1 — "optimization built in, not bolted on" — and someone must own the proof that the architecture actually survives a region failure and stays within capacity headroom at every wave. It does **not** own the SLO catalog (→ `ops-sre-reliability`) or the FinOps unit economics (→ `ops-finops`); it owns the physical capacity envelope and the proven-failover evidence.

## Owned axes / surfaces / contracts
- **Axis(es):** Cross-cutting operations
- **Surfaces:**
  - DR drill program: quarterly non-prod failover drills per axis; annual prod-equiv drills
  - Capacity model: per-axis capacity headroom, growth projections, scaling triggers
  - Region-failover automation (#214): multi-AZ failover scripts, runbook automation, chaos tooling
  - Capacity-planning board: per-axis headroom dashboards, resource-scaling recommendations
- **Cross-axis contracts (DESIGN §10):**
  - `Region / AZ / Cell` (consumer) — DR drills exercise cell-failover; capacity modelling reads cell taxonomy
- **Catalog records:** DR tooling and capacity-modelling scripts (no product crates)
- **Runbooks:** `runbooks/region-failover.md` (co-owned with `axis-cloud`), `runbooks/capacity-scaling-emergency.md`, `runbooks/dr-drill-playbook.md`
- **ADRs:** DR automation ADR (to be authored at W-Cloud-Preview)

## In-scope work
- DR drill planning and execution: quarterly non-prod failover for each axis; annual prod-equivalent drill
- Failover automation: IaC-driven multi-AZ failover (#214), tested in non-prod each quarter
- Capacity modelling: per-axis resource projections (compute, storage, network, Kafka partitions, search index shards); growth forecast vs current headroom
- Scaling trigger definitions: CPU/memory/queue-depth thresholds that trigger horizontal scale-out
- Game-day facilitation: chaos engineering experiments (with `ops-sre-reliability`)
- Region-provisioning time tracking: target ≤ 2 weeks per new region post-W-Cloud-Stable (PRD §4.2)
- DR evidence for compliance: provide failover proof records to `ops-compliance` for CSAP/K-ISMS-P DR-control evidence

## Out-of-scope (anti-scope)
- SLO authorship and burn-rate gating (→ `ops-sre-reliability`)
- FinOps unit economics (→ `ops-finops`)
- Cloud infrastructure provisioning (→ `axis-cloud` — DR team designs the drills; cloud team provisions the cells)
- Incident management (→ `ops-sre-reliability`)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `axis-cloud` | Cell taxonomy, region failover IaC (#214), multi-AZ provisioning | Quarterly drill + wave gate |
| `ops-sre-reliability` | SLO targets to validate during DR drill; chaos game-day co-facilitation | Quarterly |
| `ops-finops` | Cost headroom for capacity scaling recommendations | Monthly |
| All axis teams | Participation in quarterly DR drills; capacity growth inputs | Quarterly |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `axis-cloud` | DR drill results to validate multi-AZ automation | Quarterly |
| `ops-compliance` | Failover proof records for CSAP/K-ISMS-P DR control evidence | Quarterly + audit |
| `ops-sre-reliability` | Capacity headroom alerts fed into error-budget context | Monthly |
| `council-architecture` | DR posture summary for wave-gate readiness | Per wave |

## Success metrics
- **DR drill success rate:** 100% quarterly in non-prod (all axes exercised)
- **Annual prod-equivalent drill:** completed with RTO/RPO validated
- **Capacity headroom below trigger threshold incidents:** 0 in production
- **Region-provisioning time:** ≤ 2 weeks post-W-Cloud-Stable (PRD §4.2)
- **Failover automation coverage:** 100% of production axes have automated failover by W-Cloud-Stable
- **DR evidence records delivered to `ops-compliance`:** 100% on drill completion

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council for region/AZ/cell contract changes affecting DR design
- Cloud: `axis-cloud` lead for cell-provisioning blockers
- Founder: as last resort (prod failover event)

## Communication cadence
- Stand-up: daily async
- Weekly: 30-min sync — drill schedule, capacity headroom review
- Cross-team review: quarterly DR drill debrief with all axis leads + `ops-sre-reliability`

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch; DR automation ADR authored at W-Cloud-Preview

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| DR drill not executed quarterly | High | Quarterly drill is a wave-gate requirement; SRE lead escalates if missed |
| Capacity headroom exhausted before scaling triggers fire | High | Headroom dashboard; alert at 70% capacity; emergency scaling runbook |
| Region-provisioning takes > 2 weeks post-W-Cloud-Stable | Medium | IaC profile automation; region-provisioning time tracked monthly |

## Sources scanned
PRD.md §3.1 (horizontal scaling, W-Cloud-Preview gate), §4.2 (region provisioning metric), DESIGN.md §9 (horizontal scalability primitives: region failover drill row), ADR-0040.
