---
doc_class: Template
template_id: TPL-RUNBOOK
status: Accepted
date: 2026-05-12
purpose: |
  Diátaxis-aligned operational runbook. Dual-audience: identical text readable by an on-call agent (Foundry runbook-execution capability) and a human on-call engineer. Symptom → diagnostic → mitigation → postmortem link → SLO impact. Every alert resolves to a runbook URL; every runbook is exec-readable.
supersedes: docs/templates/runbook-template.md
header_note: "Supersedes prior docs/templates/runbook-template.md once reviewed."
enforcing_fitness_lane: governance-runbook-index-resolves
owner_team: ops-sre-reliability
related:
  - docs/RUNBOOKS-INDEX.md
  - docs/SLO-CATALOG.md
  - docs/INCIDENT-MANAGEMENT.md
  - docs/MISTAKES-LEDGER.md
  - docs/templates/postmortem-template.md
adrs_cited:
  - ADR-0053  # sanctioned primitives (agent path)
  - ADR-0052  # inventory (audit emission)
doc_status: published
---

<!-- Supersedes prior docs/templates/runbook-template.md once reviewed. -->

```yaml
# Required frontmatter
---
doc_class: Runbook
template_id: TPL-RUNBOOK
runbook_id: RB-<axis>-<slug>
title: "<imperative one-line>"
status: draft | active | deprecated
severities_supported: [Sev-1, Sev-2, Sev-3, Sev-4]
owner_team: <team-id>
last_verified: YYYY-MM-DD
last_drilled: YYYY-MM-DD
slo_topic: <oyatie.<axis>.<slo-name>>
audit_emission_topic: oyatie.ops.runbook.invoked
related_runbooks: [RB-..., RB-...]
related_adrs: [ADR-####, ...]
diataxis_class: how-to
authority_chain_declaration: |
  docs/CONSTITUTION.md > rest of docs/ > catalog records > Redirect-class > working drafts.
---
```

# Runbook RB-<axis>-<slug>: <imperative title>

## Trigger / symptom

The exact alert(s) or symptom(s) that should cause an operator (or agent) to open this runbook. Be specific:
- Alert name(s) (link to alert definition in `infra/alerts/`).
- SLI breach pattern (e.g., latency p99 > 300ms for 5min).
- Customer report shape (e.g., "cannot list capabilities in workspace X").

## SLO impact

- SLO affected: `<oyatie.<axis>.<slo-name>>` (per `docs/SLO-CATALOG.md`).
- Error budget burn rate at trigger: `<1x|3x|14x>` → severity mapping per `docs/RELEASE-MANAGEMENT.md`.

## Pre-checks (5 minutes max)

Binary verifiable conditions before executing the runbook.

- [ ] Pre-check 1: <condition> — verify by `<command|dashboard>` — expected `<output>`.
- [ ] Pre-check 2: <condition>.
- [ ] Pre-check 3: <condition>.

If any pre-check fails, **STOP** and route to a different runbook (cite which) or escalate per `docs/INCIDENT-MANAGEMENT.md §Escalation`.

## Diagnostic steps

<!-- agent-instructions:start -->
**Agent path** (Foundry runbook-execution capability):
- Every diagnostic step **MUST** capture raw stdout/stderr from a sanctioned command and attach it to the audit chain; retired helper wrappers must not be required.
- After step completion, emit `EVT-RUNBOOK-STEP-<n>` with step ID + outcome + timestamp.
- Halt and emit `BLOCKED_ON_HUMAN_ORCHESTRATOR` per `templates/checklists/escalation-checklist.md` if an unexpected outcome appears at any step.
<!-- agent-instructions:end -->

**Human path:** the same commands; paste output to incident bridge.

```
1. Verify the affected surface — `oya ops surface inspect <id>`
   Expected: <output shape>
   If differs: route to RB-<other>; do not proceed.

2. Inspect recent deploys — `<sanctioned deploy-log query>`
   Expected: <output shape>
   If differs: suspect deploy; jump to Mitigation step `M2`.

3. Inspect audit chain — `oya admin audit-chain replay --topic <topic> --last 10m`
   Expected: per-tenant emission cadence steady.
```

## Mitigation steps

Numbered, each reversible if possible.

```
M1. Quarantine — `oya ops quarantine <surface> --reason "<runbook RB-... triggered>"`
    Expected: traffic routes to fallback; audit emits `EVT-QUARANTINE`.

M2. Roll back — `oya ops rollback <surface> --to <prior-deploy-sha>`
    Expected: deploy completes <90s; SLO returns to within budget within 5 min.

M3. Capability tier downshift (Foundry runbooks only) — `oya foundry capability tier-set <id> T<n-1>`
    Expected: Cedar policy update; audit emits `EVT-CAPABILITY-TIER-CHANGED`.
```

## Rollback

What to undo if mitigation made things worse. Always reversible. Name the exact reverse command.

## Verification

- [ ] SLO returns to within budget (paste dashboard link).
- [ ] No new alerts in last 30 min.
- [ ] Tenant-customer notification cleared (if Sev-1).
- [ ] Audit chain emits `EVT-RUNBOOK-COMPLETED` with outcome `resolved`.

## Postmortem trigger

- Sev-1: postmortem required within 30 days (per `docs/templates/postmortem-template.md`).
- Sev-2: postmortem required within 60 days.
- Sev-3/4: postmortem RECOMMENDED if regression-class; required if `MISTAKES-LEDGER` row authored.

## Post-incident updates

- [ ] Postmortem authored per `docs/templates/postmortem-template.md`.
- [ ] `docs/MISTAKES-LEDGER.md` row added if a new mechanical prevention is identified (per `docs/templates/mistakes-ledger-row-template.md`).
- [ ] This runbook updated with new pre-checks / steps / verification if the incident exposed a gap.
- [ ] `docs/RUNBOOKS-INDEX.md` row updated (`last_verified` bumped).

## Audit-chain emission

Every runbook invocation emits to `oyatie.ops.runbook.invoked` per ADR-0003 with: `runbook-id`, `invoker-id`, `timestamp`, `outcome (resolved|escalated|unresolved)`, `affected tenant(s)/surface(s)`, `audit-chain emission ID`.

## Sources

- Per-service playbook (linked).
- Related ADR / issue refs.
- `docs/INCIDENT-MANAGEMENT.md §<section>`.
- Google SRE workbook chapter cited (if applicable).
