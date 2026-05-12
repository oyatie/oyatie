# Team: Crew — ADR Promotion

## Mission
This crew exists for one purpose: burn down the backlog of 71 Proposed ADRs to Accepted status, maintain the supersession graph, and ensure no Proposed ADR sits unreviewed for more than 30 days. It is a **time-boxed crew**, not a permanent team — it dissolves when the Proposed backlog reaches ≤ 5 open ADRs and a steady-state ADR review cadence is embedded in the monthly council meeting. It does **not** author new ADRs (that is each axis team's job); it facilitates the review-and-promotion pipeline.

## Owned axes / surfaces / contracts
- **Axis(es):** Cross-cutting (process function)
- **Surfaces:**
  - ADR-INDEX.md (owner — auto-emitted from `decisions/` directory; maintained by this crew)
  - `decisions/` directory: supersession graph, status transitions, deprecation notices
  - ADR promotion pipeline: triage, routing to the correct axis team for review, tracking resolution
  - ADR promotion metrics dashboard (simple — count of Proposed / In-Review / Accepted / Superseded per week)
- **Cross-axis contracts:** none owned
- **Catalog records:** none
- **Runbooks:** `runbooks/adr-promotion-triage.md`, `runbooks/adr-supersession-graph-update.md`
- **ADRs:** the crew facilitates promotion of all ADRs; it does not author them

## In-scope work
- Triage: every Proposed ADR is assigned a reviewing team (the team most affected by the decision) within 5 business days of filing
- Routing: send each ADR to the owning team for review; escalate cross-cutting ADRs to `council-architecture`
- Facilitation: follow up with reviewing teams weekly until decision (Accept / Reject / Supersede)
- Supersession graph: when an ADR is superseded, update the supersession graph in `decisions/`; update ADR-INDEX.md
- ADR-INDEX.md maintenance: auto-emit from `decisions/` directory; human-reviewed diff before merge
- Status tracking: 30-day SLA for every Proposed ADR to reach a decision; escalate overdue ADRs to `council-architecture`
- Deprecation: flag ADRs whose context has changed (wave milestones dropped, vocabulary changes); propose supersession
- Monthly batch: prepare the monthly ADR batch for council-architecture monthly meeting; includes promotion votes, supersession proposals, new-ADR registration

## Out-of-scope (anti-scope)
- Authoring ADR content (→ each axis team authors; crew facilitates)
- Architectural decisions (→ council-architecture; crew prepares the agenda but doesn't vote)
- Code changes (→ crew is a process function; no code owned)
- Permanent ADR governance after backlog is cleared (→ embeds into council-architecture monthly meeting)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| All axis + vertical teams | ADR review participation; decision within 30 days of assignment | Per ADR |
| `council-architecture` | Final vote authority on cross-cutting ADRs; escalation target for overdue ADRs | Monthly batch + on escalation |
| `council-privacy` | Review authority for privacy-architecture ADRs | Per privacy ADR |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `council-architecture` | ADR batch agenda prepared for monthly meeting | Monthly |
| All axis teams | ADR-INDEX.md accurate and current | Per ADR transition |
| `platform-tenancy-identity` | ADR-0044, ADR-0006 promoted to Accepted | ADR batch |
| `axis-foundry` | ADR-0015, ADR-0015, ADR-0050, ADR-0001, ADR-0039 promoted | ADR batch |
| `ops-compliance` | Regulatory-change ADRs promoted promptly | Per regulatory change |

## Success metrics
- **Proposed ADR backlog:** from 71 → ≤ 5 (crew dissolves at ≤ 5)
- **ADR 30-day review SLA:** 100% of Proposed ADRs reach a decision within 30 days
- **ADR-INDEX.md staleness:** 0 ADRs with mismatched status between `decisions/` files and INDEX
- **Supersession graph completeness:** 100% of superseded ADRs have a recorded supersession pointer
- **Monthly batch prepared on schedule:** 100%

## Escalation path
- Internal: crew lead → council-architecture chair
- Overdue ADRs: auto-escalate to council-architecture at 25 days; hard escalation at 30 days
- Founder: only for ADRs that cross the commercial boundary (rare)

## Communication cadence
- Stand-up: twice-weekly async (ADR triage + status check)
- Monthly: ADR batch preparation sync with council-architecture
- Crew dissolution: when backlog ≤ 5 and steady-state is embedded in council-architecture monthly meeting

## Bandwidth + hiring
- Current FTE: 1-2 dedicated crew members (part-time rotation from axis teams)
- Target FTE: dissolves at mission completion
- Open requisitions: none (rotation model)

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules (ADR-INDEX.md and decisions/ PRs)
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: crew receives ADRs; does not initiate them

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| ADR backlog not cleared before W-Foundation wave gate | High | Backlog burndown tracked weekly; escalate to council-architecture at risk |
| Supersession graph becomes inconsistent | Medium | ADR-INDEX.md auto-emit from decisions/ + human-diff review |
| Crew disbanded before backlog cleared | Medium | Dissolution criteria are explicit (≤ 5 open); council-architecture must ratify dissolution |

## Sources scanned
PRD.md §3.1 (W-Foundation gate: all foundation ADRs Accepted), DOC-CATALOG.md §2.1 (doc.adr_index owner = crew-adr-promotion), DESIGN.md §8 (flat crates — ADR-0015 state as of 2026-05-09: 71 Proposed → Accepted backlog referenced in original brief).
