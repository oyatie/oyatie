---
doc_class: Template
template_id: TPL-MILE-README
status: Accepted
date: 2026-05-13
purpose: |
  Canonical shape for .omc/plans/milestones/M0X-slug/README.md.
  Carries milestone intent, entry/exit gates, phase index (table),
  risk register, dependency citations (Bominal ADRs + oyatie ADRs),
  and agent-navigability pointer. An autonomous executor reading this
  README can enter the milestone and locate the first open phase without
  escalation.
enforcing_fitness_lane: governance-plan-hierarchy
owner_team: council-architecture
related:
  - docs/templates/phase-spec-template.md
  - docs/templates/impl-plan-template.md
  - docs/templates/INDEX.md
  - .omc/plans/MASTERPLAN.md
adrs_cited:
  - ADR-0053  # sanctioned primitives
  - ADR-0054  # scaffold-claim pattern
  - ADR-0056  # BNF v4.1
doc_status: published
---

```yaml
# Required frontmatter for every milestone README.md
---
doc_class: MilestoneReadme
template_id: TPL-MILE-README
milestone_id: M0X-<slug>        # e.g. M03-first-paying-tenant
parent: ../../MASTERPLAN.md     # relative path to MASTERPLAN
status: Proposed | Active | Complete
entry_gate: |
  Exact condition from the prior milestone that must be true before M0X starts.
  Name the prior milestone ID + its exit criterion, or "none" for M01.
exit_gate: |
  Exact condition that declares M0X complete. Must name at least:
  - Buck2/cloud-ci command or required context evidence
  - a product-level outcome (e.g., "1 KR paying tenant live")
owner_team: <team-id>
bominal_adrs_inherited:
  - ADR-####  # list Bominal ADRs this milestone inherits 1:1 (translated)
oyatie_adrs_cited:
  - ADR-####  # Oyatie-specific ADRs governing this milestone
---
```

# M0X-<slug>: <one-line milestone title, present tense>

## Milestone Intent

One paragraph. What this milestone delivers and why it is a coherent
commitment. Which product-level outcome it unlocks. Which Master Plan
principles it advances (cite by number from `.omc/plans/MASTERPLAN.md §2`).
Present tense; durable outcome framing.

---

## Entry Gate

Exact condition from prior milestone that must be true before this milestone
begins. Reference the prior milestone's exit gate by ID.

Example: "M02 exit gate complete: Foundry engine + substrate µservices all
ship; 9 architecture planes green; all `--report-only` fitness lanes flipped
to BLOCKER."

If M01: "none — first milestone."

---

## Exit Gate

Exact, measurable conditions that declare this milestone complete:

1. `<Buck2 / cloud-ci command>` exits 0 across all milestone-scope targets.
3. `<product outcome>`: e.g., "1 KR group paying tenant live; 4대보험 EDI green."
4. `<fitness lane>`: all LEAN lanes exit 0 for milestone-scope crates.

---

## Phase Index

| Phase ID | Title | Status | Impl plans | Phase README |
|---|---|---|---|---|
| P01-<slug> | `<title>` | Proposed \| Active \| Complete | `<N>` IPs | [`phases/P01-<slug>/README.md`](phases/P01-<slug>/README.md) |
| P02-<slug> | `<title>` | Proposed \| Active \| Complete | `<N>` IPs | [`phases/P02-<slug>/README.md`](phases/P02-<slug>/README.md) |

Parallelism: phases `P01` and `P02` may run in parallel if entry gates are
independent. Phase `P03` serializes on `P01` exit gate. Declare serialization
root here.

---

## Risk Register

Milestone-scoped risk slice. Full register: `docs/RISK-REGISTER.md`.

| ID | Risk | Likelihood | Impact | Owner | Mitigation | Status |
|---|---|---|---|---|---|---|
| RM-M0X-01 | `<one-line risk>` | High \| Med \| Low | High \| Med \| Low | `<team-id>` | `<mitigation>` | Open \| Mitigated |

---

## Dependencies

### Bominal ADRs inherited (1:1 with oyatie glossary translation)

| Bominal ADR | Title | Translation note |
|---|---|---|
| ADR-#### | `<title>` | `platform → shared`; `Object Graph → Ontology`; `Shell → Application` |

### oyatie ADRs cited

| oyatie ADR | Title | Scope |
|---|---|---|
| ADR-#### | `<title>` | `<which phases it governs>` |

### Upstream milestone dependencies

| Milestone | Exit gate required | Status |
|---|---|---|
| M0X-1 | `<exit gate excerpt>` | Open \| Complete |

---

## Agent-Navigability Pointer

The first file a fresh executor MUST read to enter this milestone:

```
.omc/plans/milestones/M0X-<slug>/README.md   ← you are here
  → phases/P01-<slug>/README.md              ← first open phase
  → phases/P01-<slug>/<impl-plan-name>.md    ← first open IP
```

Before claiming, executor MUST:
2. Read `docs/prds/<µservice>.md` for each µservice in milestone scope.
3. Read `docs/CONSTITUTION.md §Decision principles + §Prohibitions`.
4. Confirm no symbols in the first IP are currently claimed by another agent.

---

## References

- MASTERPLAN: `.omc/plans/MASTERPLAN.md`
- Bominal ADRs (translated): per `feedback_bominal_inheritance_precedence.md`
- oyatie overrides: `feedback_*.md` memory files
- ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim), ADR-0056 (BNF v4.1)
- Memory: `feedback_milestone_phase_hierarchy.md`,
  `feedback_grit_claim_work_done.md`,
  `feedback_quality_performance_scalability_bar.md`
