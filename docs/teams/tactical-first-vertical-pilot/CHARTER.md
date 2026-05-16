---
doc_status: published
---

# Team: Tactical — First Vertical Pilot

## Mission
This team coordinates the W-Vertical-Pilot wave: the first full-stack vertical implementation running end-to-end on the Foundation + Axes preview stack with real design-partner tenants. It is a **time-boxed coordination team**, not a permanent team — it dissolves when the W-Vertical-Pilot wave gate passes and the learnings are embedded into each axis and vertical team's backlog. Its primary job is to unblock cross-team dependencies that slow the pilot, not to own product surfaces.

> **Retirement note (2026-05-09):** The predecessor team `tactical-m3-launch` is **RETIRED** as of 2026-05-09 because the "M3" milestone vocabulary was dropped when the PRD §3.1 sequencing language was replaced with `Foundation → Substrate → Axis-Preview → …`. The word "M3" appears nowhere in the current canonical docs. This team (`tactical-first-vertical-pilot`) replaces it. Any open work items that referenced `tactical-m3-launch` should be re-triaged against the W-Vertical-Pilot scope. See `docs/teams/README.md § Retired teams` for the archived record.

## Owned axes / surfaces / contracts
- **Axis(es):** Cross-cutting coordination (no product surfaces owned)
- **Surfaces:**
  - W-Vertical-Pilot readiness board: per-axis readiness checklist, blocker tracker, dependency matrix
  - Design-partner pilot coordination: tenant onboarding sequencing, pilot feedback cadence with `gtm-customer-success`
  - Cross-team dependency matrix: which deliverable each team needs from which other team to unblock the pilot
- **Cross-axis contracts:** none owned
- **Catalog records:** none
- **Runbooks:** `runbooks/vertical-pilot-wave-gate-readiness.md`
- **ADRs:** none authored; ADR-0050 (wave plan) consulted

## In-scope work
- W-Vertical-Pilot gate readiness: track the gate criteria from PRD §3.1 ("Pilot tenant runs end-to-end on foundation+axes preview stack")
- Vertical selection: facilitate the council-architecture decision on which vertical to pilot first (likely `vertical-corporate` per PRD §3.1 comment; council decides)
- Cross-team dependency matrix: enumerate which deliverables block the pilot (e.g., `platform-tenancy-identity` Move #0, `axis-foundry` SecretProvider, `axis-saas` workflow engine preview, `axis-cloud` cell provisioning, `vertical-corporate` payroll workflows)
- Blocker escalation: unblock cross-team dependencies that no single team can resolve alone; escalate to council-architecture within 24 h if a blocker persists > 3 days
- Pilot tenant onboarding coordination: sequence the first 3 KR Group tenant onboardings with `vertical-corporate` and `gtm-customer-success`
- Feedback synthesis: collect pilot learnings weekly; synthesize into structured input for council-architecture and relevant axis teams
- Wave-gate evidence collection: assemble the evidence pack that council-architecture reviews to approve the W-Vertical-Pilot gate

## Out-of-scope (anti-scope)
- Product surface ownership (→ each axis and vertical team)
- Permanent team responsibilities (this team dissolves after W-Vertical-Pilot gate passes)
- Authoring the vertical pilot product (→ `vertical-corporate` + axis teams)
- Commercial negotiations with pilot tenants (→ `gtm-sales-se`)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `platform-tenancy-identity` | Move #0 tenancy kernel (P0 prereq for the pilot) | Weekly status |
| `axis-foundry` | SecretProvider + KMS (Issue #1315); daemon hardening; smoke lane | Weekly status |
| `axis-saas` | Workflow engine preview; OG; tenant onboarding UX | Weekly status |
| `axis-cloud` | Cell provisioning for pilot tenant | Weekly status |
| `vertical-corporate` | Payroll + HR workflows for first 3 KR Group tenants | Weekly status |
| `platform-audit-evidence` | Audit chain operational for pilot (100% evidence completeness gate) | Weekly status |
| `platform-privacy-dub` | Data Use Boundary ADR Accepted (P0 prereq) | ADR gate |
| `gtm-customer-success` | Design-partner onboarding + feedback cadence | Per pilot tenant |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `council-architecture` | W-Vertical-Pilot wave-gate evidence pack | Wave gate |
| All axis + vertical teams | Unblocked cross-team dependencies; pilot learnings feedback | Weekly |
| `gtm-sales-se` | Pilot status for pipeline conversations | Monthly |

## Success metrics
- **W-Vertical-Pilot gate criteria met:** all criteria from PRD §3.1 satisfied ("Pilot tenant runs end-to-end on foundation+axes preview stack")
- **Pilot tenant onboarding:** ≥ 3 KR Group tenants live end-to-end
- **Blocker resolution time:** < 3 days for cross-team blockers before escalation
- **Pilot feedback synthesis:** weekly synthesis delivered to council-architecture
- **Wave-gate evidence pack delivery:** complete, on time for council review
- **Team dissolution:** occurs at W-Vertical-Pilot gate pass + 2-week handover

## Escalation path
- Internal: pilot lead → council-architecture (direct — this team exists to escalate blockers)
- Founder: if council-architecture cannot resolve a blocker within 5 days
- Commercial: `gtm-sales-se` for any pilot-tenant commercial issues

## Communication cadence
- Stand-up: daily sync (this is an active coordination team during the pilot sprint)
- Weekly: 60-min cross-team sync with all pilot-axis leads
- Ad hoc: blocker escalation within 24 h of detection

## Bandwidth + hiring
- Current FTE: 1-2 dedicated coordinators (seconded from axis teams or hired as program managers)
- Target FTE: dissolves at mission completion
- Open requisitions: none (time-boxed role)

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules (tooling PRs if any)
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: N/A (coordination function; ADRs authored by axis teams)

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| `axis-foundry` SecretProvider (Issue #1315) not resolved → pilot blocked | High | Weekly status check; escalate to council-architecture at 2-week stall |
| Data Use Boundary ADR not Accepted before pilot begins | High | Pilot cannot start without ADR; track weekly with `platform-privacy-dub` |
| Pilot tenant disengages before W-Vertical-Pilot gate | High | `gtm-customer-success` manages relationship; pilot team removes product blockers |
| Move #0 tenancy delayed → no tenant isolation for pilot | Catastrophic | Move #0 is P0 prereq; pilot cannot run without it |

## Sources scanned
PRD.md §3.1 (W-Vertical-Pilot gate criteria), §4.1 (design-partner targets), ADR-0050 (wave plan), DOC-CATALOG.md §2.1 (doc.roadmap former owner = tactical-m3-launch, now rolling).

---

## Retirement record for `tactical-m3-launch`

| Field | Value |
|---|---|
| Former team ID | `tactical-m3-launch` |
| Retirement date | 2026-05-09 |
| Reason | "M3" vocabulary dropped per PRD §3.1 vocabulary update 2026-05-09; milestone numbering (M0/M1/M2/M3/MVP) replaced by wave sequencing language |
| Replacement | `tactical-first-vertical-pilot` (this team) |
| Open work items | Re-triage against W-Vertical-Pilot scope; any item not in scope of W-Vertical-Pilot goes to the owning axis team |
| Doc previously owned | `ROADMAP.md` (rolling owner; now `council-architecture` until reassigned) |
