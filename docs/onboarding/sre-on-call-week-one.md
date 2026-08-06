---
doc_class: OnboardingGuide
role: "SRE and on-call engineer, reliability operations"
status: Published
date: 2026-05-20
owner: "ops-sre-reliability + axis-observability"
related_oyatie_adrs:
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/DOC-CATALOG.md
  - docs/STANDARDS-AND-TEMPLATES.md
inbound_citations:
  - docs/onboarding/intern-day-one.md
  - docs/onboarding/intern-week-one.md
enforced_by:
  - oya-governance-doc-rigor
  - oya-governance-doc-graph-6hops
---

# SRE On-Call Week-One Onboarding

Audience: SRE and on-call engineer, reliability operations.
Industry precedent: Google SRE incident command, AWS Health Dashboard operational state, PagerDuty event orchestration, and Cloudflare rollback-first reliability practice.

This guide is written for a programming-capable new joiner with no prior Oyatie architecture knowledge. Every phase names the repo files to read, the artifact to produce, the owner who reviews it, and the stop condition that proves the phase is complete.

Substance rule: do not treat this guide as orientation prose. Treat it as a work plan whose outputs can be inspected, replayed, or rejected.

You join SRE to keep the platform observable, operable, and recoverable across cells, tenants, policy gates, incident-management flows, and Ops Dashboard Control Center surfaces.
Week one is focused: get dev-cell access, learn incident command, read the runbook library, shadow one on-call handoff, and make a small runbook or dashboard improvement with verification evidence.
SRE here is not a dashboard-watcher role. You own the shortest path from a production symptom to a safe rollback, a sealed incident timeline, and a follow-up that prevents recurrence.

## Hyperscaler-Grade Reading Contract

- Named precedent: Google SRE incident command, AWS Health Dashboard operational state, PagerDuty event orchestration, and Cloudflare rollback-first reliability practice.
- Failure-mode tree: each week includes at least three explicit failure paths to inspect before claiming readiness.
- Capacity math: when a task names latency, node count, audit throughput, tenant count, or escalation timing, write the derivation in the onboarding issue instead of copying a target.
- Observability hooks: every contribution names metrics, traces, logs, audit events, dashboards, or evidence files that prove the behavior.
- Rollback path: every state-changing task names how the change is reverted and how the revert is verified.
- Multi-region awareness: every globally visible task names the behavior when the secondary region or sovereign cell is unreachable.
- Sovereign-cell awareness: regulated data paths must mention KR-CSAP, EU-sovereign, CN-PIPL, IL5/6, and FedRAMP-High impact if the role touches tenant data.
- Versioning and deprecation: every public contract, schema, event, policy fragment, or user-visible capability names its version and sunset expectations.

## Day 0: Laptop Bring-Up

Day 0 exists to remove access uncertainty before day-one work starts. Complete these items before opening the first code or documentation PR.
| Item | Owner | Artifact | Failure branch |
| --- |--- |--- |--- |
| Repo access | engineering-ops | `git remote -v` shows the Oyatie repo and `git status --short` runs without auth failure | If SSO blocks access, file access ticket and stop before cloning private tenant artifacts |
| Identity profile | axis-identity | Dev identity has MFA and passkey enrolled | If MFA fails, do not ask a teammate to share tokens; escalate to identity support |
| Dev tenant | axis-tenancy | Sandbox tenant id is recorded in onboarding issue | If tenant creation fails, attach tenancy error and do not reuse another tenant |
| OpenBao path | axis-secrets | Read-only role path assigned with values redacted | If secret read returns broad wildcard access, report as security issue |
| Calendar holds | mentor | All checkpoint meetings created | If mentor is unavailable, use the backup contact listed in escalation channels |
- Verification for Day 0: onboarding issue contains access receipts, redacted secret path, and sandbox tenant id
- Stop condition for Day 0: mentor and owner can point to the artifact without asking you to explain hidden context.

## Day 1: Environment Setup

Target result: a working local development cell, editor, CLI, credentials, and first evidence bundle.
Paste-runnable command sequence:

```bash
code --install-extension rust-lang.rust-analyzer
code --install-extension vadimcn.vscode-lldb
code --install-extension redhat.vscode-yaml
code --install-extension svelte.svelte-vscode
code --install-extension ms-playwright.playwright
./bin/oya doctor
git status --short --branch
./bin/oya dev cell status --cell dev-cell-a
```

| Tool | Role-specific expectation |
| --- |--- |
| VS Code | Install the repo profile plus rust-analyzer, Even Better TOML, YAML, Svelte for VS Code, Playwright Test, CodeLLDB, Cedar policy syntax, and Markdown All in One. Artifact: screenshot or `code --list-extensions` pasted into the onboarding issue. |
| oya CLI | Run `./bin/oya doctor`, `git status --short --branch`, `./bin/oya verify --help`, and the role-specific smoke command. Artifact: terminal transcript with command, exit code, and expected output note. |
| vault credentials | Request the dev-cell OpenBao path for the role and confirm access to read-only bootstrap credentials only. Artifact: secret path receipt with values redacted. |
| dev cell access | Join `dev-cell-a` with the assigned sandbox tenant and verify the cell health endpoint. Artifact: `dev-cell-access-ok` evidence row in the onboarding issue. |
| mentor pairing | Book the named mentor checkpoint series before writing code. Artifact: calendar holds for day 1, day 3, week 2, week 4, month 2, and quarter 1. |

Day-one artifact checklist:
- VS Code extension list captured in the onboarding issue.
- `./bin/oya doctor` result attached with exit code.
- OpenBao or vault credential path recorded with secret values redacted.
- Dev cell access receipt attached.
- Sandbox tenant id attached.
- First mentor checkpoint scheduled.
- One role-specific smoke path from this guide selected for week-one work.
- Verification for Day 1: environment evidence bundle has commands, output summary, and failure notes
- Stop condition for Day 1: mentor and owner can point to the artifact without asking you to explain hidden context.

## Week 1: Code Walkthrough

Read these files in order. Do not browse randomly; the order teaches authority, doctrine, contract, implementation, test, and operational evidence.
1. docs/INCIDENT-MANAGEMENT.md
2. docs/RUNBOOKS-INDEX.md
3. docs/standards/observability.md
4. docs/standards/observability-slo.md
5. docs/standards/on-call.md
6. microservices/ops-dashboard-control-center/README.md
7. microservices/ops-dashboard-control-center/ARCHITECTURE.md
8. microservices/ops-dashboard-control-center/PRD.md
9. microservices/ops-dashboard-control-center/runbooks/incident-command.md
10. microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md
11. microservices/ops-dashboard-control-center/dashboards/ops-overview.json
12. microservices/incident-management/README.md
13. microservices/incident-management/contracts/openapi-v1.yaml
14. microservices/incident-management/slos/local-page-to-acknowledge.openslo.yaml
15. microservices/observability/manifest.json


### Named ADRs to read

- ADR-0042 observability stack
- ADR-0186 observability backplane layering
- ADR-0114 canary observability rollback
- ADR-0263 observability emission contract
- ADR-0248 cellular architecture

### Named playgrounds

1. microservices/ops-dashboard-control-center/dashboards/ops-overview.json
   - Artifact: write a four-sentence note explaining what this playground proves for SRE and on-call engineer, reliability operations.
2. microservices/incident-management/runbooks/local-page-to-ack-burn.md
   - Artifact: write a four-sentence note explaining what this playground proves for SRE and on-call engineer, reliability operations.
3. docs/runbooks/on-call-handover.md
   - Artifact: write a four-sentence note explaining what this playground proves for SRE and on-call engineer, reliability operations.
4. docs/runbooks/error-budget-exhaustion.md
   - Artifact: write a four-sentence note explaining what this playground proves for SRE and on-call engineer, reliability operations.

### Week-one failure modes to inspect

- A required file path moved or does not exist. Artifact: issue comment with replacement path or broken-link finding.
- A doctrine document and implementation artifact disagree. Artifact: contradiction note with both references.
- A local smoke command passes but does not prove the production claim. Artifact: claim-boundary note.
- A policy, audit, tenant, locale, AI, or compliance branch has no rollback. Artifact: missing-rollback finding.
- A dashboard or runbook lacks expected output. Artifact: runbook improvement candidate.
- Verification for Week 1: walkthrough note names files read, ADRs read, playground outputs, and one concrete starter PR candidate
- Stop condition for Week 1: mentor and owner can point to the artifact without asking you to explain hidden context.

## Week 2: First Contribution Path

Target result: one small merged or review-ready contribution that exercises the role without widening scope.

### Named easy bugs and starter PRs

1. SRE-STARTER-001 add an expected metric label to one Ops Dashboard SLO runbook
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for SRE.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
2. SRE-STARTER-002 add a rollback verification step to `deployment-rollback.md`
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for SRE.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
3. SRE-STARTER-003 add one on-call handoff checklist row tied to a dashboard panel
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for SRE.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
4. SRE-STARTER-004 tighten an incident-management runbook branch for page-to-ack burn
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for SRE.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.

### Mentor pairing protocol

Pair with primary on-call for the first handoff, incident commander for the first tabletop, and observability owner for dashboard evidence. Every pairing ends with symptom, dashboard, runbook, rollback, and post-incident follow-up.
Pairing agenda:
- Read the diff goal aloud in one sentence.
- Name the governing ADR and the doc or test that proves the behavior.
- Name at least one failure branch before editing.
- Make the smallest change that proves the behavior.
- Run the narrowest verification first.
- Record the output in the onboarding issue.
- Ask the mentor to identify any missing rollback or evidence path.
- Open the PR or checkpoint note only after the artifact exists.
- Verification for Week 2: starter PR or checkpoint note includes owner, files, tests, rollback, and open risks
- Stop condition for Week 2: mentor and owner can point to the artifact without asking you to explain hidden context.

## Week 3-4: Independent Work

First independent project: Own `SRE-PROJ-001`: connect one incident-management SLO to an Ops Dashboard panel and add a runbook branch that verifies rollback and audit timeline completeness.

Required checkpoints:
| Checkpoint | Timing | Reviewer | Artifact |
| --- |--- |--- |--- |
| Design skim | start of week 3 | mentor plus owning axis | one-page scope, non-goals, failure modes |
| Midpoint evidence | middle of week 3 | mentor | test or runbook evidence proving the hardest branch |
| Pre-review | end of week 3 | cross-team owner | diff plus rollback proof |
| Final review | week 4 | owning council or axis lead | merged PR or accepted checkpoint with next owner |
| Retrospective | week 4 close | mentor | what changed, what failed, what should be documented next |
Independent work guardrails:
- Do not invent a new abstraction until an existing utility or registry row cannot solve the problem.
- Do not bypass Cedar, audit-chain, tenant, locale, accessibility, compliance, or AI guardrails to make the slice smaller.
- Do not claim ownership of a cross-team surface without a named handoff ritual.
- Do not merge a documentation-only fix if it contradicts implementation evidence.
- Do not accept a green local check when the claim depends on multi-region, sovereign-cell, or tenant isolation behavior.
- Verification for Week 3-4: independent project has pre-review evidence, rollback note, and mentor sign-off
- Stop condition for Week 3-4: mentor and owner can point to the artifact without asking you to explain hidden context.

## Month 1: Acceleration

By the end of month one, you should own repeatable work rather than single isolated tasks.

### Named projects to own

- Ops Dashboard Control Center on-call view hardening
- incident-management SLO evidence pass
- runbook library freshness review
- observability dashboard label cardinality check

### Key contacts in other teams

- ops-sre-reliability
- axis-observability
- axis-incident-management
- axis-ops-dashboard-control-center
- axis-audit-chain
- axis-cell

### Month-one operating rhythm

| Cadence | Action | Artifact |
| --- |--- |--- |
| weekly | review one governing ADR and one implementation artifact | two-paragraph drift or no-drift note |
| weekly | review one runbook, SLO, dashboard, or evidence path | freshness note with expected output |
| biweekly | pair outside your home team | handoff note naming owner and stop condition |
| monthly | present one failure mode and rollback path | recorded note in team meeting log |
| monthly | clean one documentation assumption gap | merged doc or accepted issue |
- Verification for Month 1: one owned project is review-ready and cross-team contacts can explain your current scope
- Stop condition for Month 1: mentor and owner can point to the artifact without asking you to explain hidden context.

## Month 2: Domain Expertise

Month two moves from operating the checklist to explaining why the checklist exists.
1. incident command roles and statuspage sync
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for incident command roles and statuspage sync.
2. Ops Dashboard Control Center operator flows
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for Ops Dashboard Control Center operator flows.
3. runbook freshness and branch completeness
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for runbook freshness and branch completeness.
4. OpenTelemetry traces, metrics, logs, and audit event joins
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for OpenTelemetry traces, metrics, logs, and audit event joins.
5. SLO burn rate and error budget policy
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for SLO burn rate and error budget policy.
6. cell failover and tenant evacuation
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for cell failover and tenant evacuation.
7. canary rollback and feature flag kill switch
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for canary rollback and feature flag kill switch.
8. paging storm throttle and escalation policy corruption
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for paging storm throttle and escalation policy corruption.
9. postmortem sealing and evidence retention
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for postmortem sealing and evidence retention.
10. capacity admission, saturation, and brownout behavior
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for capacity admission, saturation, and brownout behavior.
Domain expertise stop condition: you can answer what breaks if this topic is implemented incorrectly, who owns the rollback, and what evidence proves the system recovered.
- Verification for Month 2: deep-dive notes cover every listed topic and at least one note is reviewed by a cross-team owner
- Stop condition for Month 2: mentor and owner can point to the artifact without asking you to explain hidden context.

## Quarter 1: Ownership


### Named OKRs

- OKR-SRE-Q1-1: shadow two incidents or game days and write one prevention follow-up
- OKR-SRE-Q1-2: improve one runbook so every step has command, expected signal, and rollback branch
- OKR-SRE-Q1-3: connect one dashboard panel to its SLO and incident-management workflow

### Named on-call rotation entry

Enter `primary-oncall-shadow` in week one and `secondary-oncall` only after the runbook improvement merges and a handoff drill is signed off.

### Named team-OKR contribution

Contribute to `TEAM-OKR-SRE-2026Q2`: every Sev-1/2 path has dashboard, runbook, rollback, postmortem, and audit-chain evidence.

### Quarter-one ownership review

| Question | Required answer |
| --- |--- |
| What do you own? | Named project, doc, runbook, test, gate, dashboard, or customer artifact. |
| What can fail? | At least three failure modes with expected behavior. |
| How is it observed? | Metric, trace, log, audit event, dashboard, or evidence file. |
| How is it rolled back? | Named command, file revert, policy revocation, or tenant rollback procedure. |
| Who receives handoff? | Named team and ritual from the collaboration playbook. |
- Verification for Quarter 1: manager and mentor accept OKR evidence, rotation readiness, and team-OKR contribution
- Stop condition for Quarter 1: mentor and owner can point to the artifact without asking you to explain hidden context.

## Common Anti-Patterns to Avoid

1. Acknowledging pages without linking the incident room and evidence timeline.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for SRE and on-call engineer, reliability operations.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
2. Relying on tribal knowledge instead of the runbook library.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for SRE and on-call engineer, reliability operations.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
3. Changing alert thresholds to stop noise without proving user impact.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for SRE and on-call engineer, reliability operations.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
4. Rollback that restores service but loses audit or compliance evidence.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for SRE and on-call engineer, reliability operations.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
5. Treating dev-cell success as proof of multi-region readiness.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for SRE and on-call engineer, reliability operations.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
6. Ignoring cardinality budgets in new metrics.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for SRE and on-call engineer, reliability operations.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
7. Writing postmortems that stop at human error.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for SRE and on-call engineer, reliability operations.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
8. Forgetting tenant blast-radius and sovereign-cell constraints during incidents.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for SRE and on-call engineer, reliability operations.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.

## Cross-Team Collaboration Playbook

| Team | Handoff ritual | Minimum payload |
| --- |--- |--- |
| axis-observability | Telemetry handoff | Attach metric, trace, log, audit event, cardinality budget, and dashboard panel. |
| axis-incident-management | Incident handoff | Attach incident id, role map, timeline, statuspage state, and postmortem owner. |
| axis-cell | Cell operation handoff | Attach region, tenant cohort, evacuation path, and rollback state. |
| axis-audit-chain | Evidence handoff | Attach event id, seal state, replay command, and retention class. |
| ops-compliance | Regulated incident handoff | Attach notification clock, regulator packet owner, and trust portal state. |
Handoff rules:
- Start with the target result, not the history of the work.
- Name the file, contract, policy, runbook, or evidence object that changed.
- State what is not changing.
- Attach verification evidence and rollback path.
- Name the next owner and stop condition.
- Record the handoff in the onboarding issue or project tracker so it survives team memory loss.

## Glossary

| Term | Role-specific meaning |
| --- |--- |
| incident commander | Owner of incident process, timeline, roles, and stop condition. |
| SLO burn | Rate at which service consumes allowed error budget. |
| Ops Dashboard Control Center | Operator surface for cluster, incident, evidence, tenant, and rollback workflows. |
| runbook freshness | Evidence that procedure still matches the current system and tooling. |
| rollback proof | Verification that state, telemetry, and audit evidence returned to expected shape. |

## Escalation Channels

| Escalation | Use when | Owner |
| --- |--- |--- |
| mentor checkpoint | you can proceed locally but need review of reasoning or evidence | assigned mentor |
| axis owner | a file or policy belongs to another team | ops-sre-reliability |
| council review | claim boundary, doctrine, compliance, or security interpretation changes | ops-sre-reliability + axis-observability |
| SRE on-call | dev-cell, incident, or reliability path blocks verification | ops-sre-reliability |
| security review | credential, tenant isolation, policy, or regulated data risk appears | ops-security |

## Resources & References

- docs/INCIDENT-MANAGEMENT.md
- docs/RUNBOOKS-INDEX.md
- docs/standards/on-call.md
- docs/standards/observability.md
- docs/runbooks/on-call-handover.md
- docs/runbooks/error-budget-exhaustion.md
- microservices/ops-dashboard-control-center/README.md

Reference-reading protocol: open the resource, identify the authority section, write the one-sentence claim it supports, and record whether the resource is doctrine, spec, implementation, test, runbook, dashboard, or evidence.

## Role-Specific Drill Library

Use this ledger when you need extra practice or when a mentor asks for stronger evidence. Each drill is intentionally small but must end with a verifiable artifact.

### Drill SRE-001: page-to-ack burn
- Read: docs/INCIDENT-MANAGEMENT.md
- Connects to: incident command roles and statuspage sync
- Build or inspect: a minimal artifact that proves page-to-ack burn without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for page-to-ack burn.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show page-to-ack burn is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-001 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-002: incident room not created
- Read: docs/RUNBOOKS-INDEX.md
- Connects to: Ops Dashboard Control Center operator flows
- Build or inspect: a minimal artifact that proves incident room not created without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for incident room not created.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show incident room not created is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-observability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-002 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-003: dashboard perf degradation
- Read: docs/standards/observability.md
- Connects to: runbook freshness and branch completeness
- Build or inspect: a minimal artifact that proves dashboard perf degradation without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for dashboard perf degradation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show dashboard perf degradation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-incident-management with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-003 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-004: deployment rollback
- Read: docs/standards/observability-slo.md
- Connects to: OpenTelemetry traces, metrics, logs, and audit event joins
- Build or inspect: a minimal artifact that proves deployment rollback without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for deployment rollback.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show deployment rollback is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ops-dashboard-control-center with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-004 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-005: on-call handoff failure
- Read: docs/standards/on-call.md
- Connects to: SLO burn rate and error budget policy
- Build or inspect: a minimal artifact that proves on-call handoff failure without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for on-call handoff failure.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show on-call handoff failure is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-005 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-006: statuspage sync gap
- Read: microservices/ops-dashboard-control-center/README.md
- Connects to: cell failover and tenant evacuation
- Build or inspect: a minimal artifact that proves statuspage sync gap without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for statuspage sync gap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show statuspage sync gap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-cell with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-006 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-007: SLO error budget exhaustion
- Read: microservices/ops-dashboard-control-center/ARCHITECTURE.md
- Connects to: canary rollback and feature flag kill switch
- Build or inspect: a minimal artifact that proves SLO error budget exhaustion without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for SLO error budget exhaustion.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show SLO error budget exhaustion is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-007 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-008: metric cardinality spike
- Read: microservices/ops-dashboard-control-center/PRD.md
- Connects to: paging storm throttle and escalation policy corruption
- Build or inspect: a minimal artifact that proves metric cardinality spike without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for metric cardinality spike.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show metric cardinality spike is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-observability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-008 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-009: audit timeline gap
- Read: microservices/ops-dashboard-control-center/runbooks/incident-command.md
- Connects to: postmortem sealing and evidence retention
- Build or inspect: a minimal artifact that proves audit timeline gap without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit timeline gap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit timeline gap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-incident-management with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-009 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-010: cell failover
- Read: microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md
- Connects to: capacity admission, saturation, and brownout behavior
- Build or inspect: a minimal artifact that proves cell failover without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cell failover.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cell failover is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ops-dashboard-control-center with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-010 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-011: tenant evacuation
- Read: microservices/ops-dashboard-control-center/dashboards/ops-overview.json
- Connects to: incident command roles and statuspage sync
- Build or inspect: a minimal artifact that proves tenant evacuation without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant evacuation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant evacuation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-011 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-012: paging storm throttle
- Read: microservices/incident-management/README.md
- Connects to: Ops Dashboard Control Center operator flows
- Build or inspect: a minimal artifact that proves paging storm throttle without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for paging storm throttle.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show paging storm throttle is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-cell with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-012 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-013: postmortem seal failure
- Read: microservices/incident-management/contracts/openapi-v1.yaml
- Connects to: runbook freshness and branch completeness
- Build or inspect: a minimal artifact that proves postmortem seal failure without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for postmortem seal failure.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show postmortem seal failure is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-013 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-014: observability ingest lag
- Read: microservices/incident-management/slos/local-page-to-acknowledge.openslo.yaml
- Connects to: OpenTelemetry traces, metrics, logs, and audit event joins
- Build or inspect: a minimal artifact that proves observability ingest lag without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for observability ingest lag.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show observability ingest lag is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-observability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-014 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-015: cluster health stale
- Read: microservices/observability/manifest.json
- Connects to: SLO burn rate and error budget policy
- Build or inspect: a minimal artifact that proves cluster health stale without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cluster health stale.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cluster health stale is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-incident-management with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-015 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-016: brownout activation
- Read: docs/INCIDENT-MANAGEMENT.md
- Connects to: cell failover and tenant evacuation
- Build or inspect: a minimal artifact that proves brownout activation without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for brownout activation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show brownout activation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ops-dashboard-control-center with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-016 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-017: runbook freshness check
- Read: docs/RUNBOOKS-INDEX.md
- Connects to: canary rollback and feature flag kill switch
- Build or inspect: a minimal artifact that proves runbook freshness check without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for runbook freshness check.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show runbook freshness check is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-017 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-018: regulated incident notification
- Read: docs/standards/observability.md
- Connects to: paging storm throttle and escalation policy corruption
- Build or inspect: a minimal artifact that proves regulated incident notification without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regulated incident notification.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regulated incident notification is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-cell with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-018 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-019: page-to-ack burn
- Read: docs/standards/observability-slo.md
- Connects to: postmortem sealing and evidence retention
- Build or inspect: a minimal artifact that proves page-to-ack burn without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for page-to-ack burn.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show page-to-ack burn is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-019 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-020: incident room not created
- Read: docs/standards/on-call.md
- Connects to: capacity admission, saturation, and brownout behavior
- Build or inspect: a minimal artifact that proves incident room not created without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for incident room not created.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show incident room not created is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-observability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-020 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-021: dashboard perf degradation
- Read: microservices/ops-dashboard-control-center/README.md
- Connects to: incident command roles and statuspage sync
- Build or inspect: a minimal artifact that proves dashboard perf degradation without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for dashboard perf degradation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show dashboard perf degradation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-incident-management with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-021 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-022: deployment rollback
- Read: microservices/ops-dashboard-control-center/ARCHITECTURE.md
- Connects to: Ops Dashboard Control Center operator flows
- Build or inspect: a minimal artifact that proves deployment rollback without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for deployment rollback.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show deployment rollback is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ops-dashboard-control-center with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-022 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-023: on-call handoff failure
- Read: microservices/ops-dashboard-control-center/PRD.md
- Connects to: runbook freshness and branch completeness
- Build or inspect: a minimal artifact that proves on-call handoff failure without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for on-call handoff failure.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show on-call handoff failure is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-023 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-024: statuspage sync gap
- Read: microservices/ops-dashboard-control-center/runbooks/incident-command.md
- Connects to: OpenTelemetry traces, metrics, logs, and audit event joins
- Build or inspect: a minimal artifact that proves statuspage sync gap without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for statuspage sync gap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show statuspage sync gap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-cell with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-024 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-025: SLO error budget exhaustion
- Read: microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md
- Connects to: SLO burn rate and error budget policy
- Build or inspect: a minimal artifact that proves SLO error budget exhaustion without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for SLO error budget exhaustion.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show SLO error budget exhaustion is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-025 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-026: metric cardinality spike
- Read: microservices/ops-dashboard-control-center/dashboards/ops-overview.json
- Connects to: cell failover and tenant evacuation
- Build or inspect: a minimal artifact that proves metric cardinality spike without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for metric cardinality spike.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show metric cardinality spike is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-observability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-026 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-027: audit timeline gap
- Read: microservices/incident-management/README.md
- Connects to: canary rollback and feature flag kill switch
- Build or inspect: a minimal artifact that proves audit timeline gap without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit timeline gap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit timeline gap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-incident-management with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-027 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-028: cell failover
- Read: microservices/incident-management/contracts/openapi-v1.yaml
- Connects to: paging storm throttle and escalation policy corruption
- Build or inspect: a minimal artifact that proves cell failover without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cell failover.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cell failover is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ops-dashboard-control-center with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-028 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-029: tenant evacuation
- Read: microservices/incident-management/slos/local-page-to-acknowledge.openslo.yaml
- Connects to: postmortem sealing and evidence retention
- Build or inspect: a minimal artifact that proves tenant evacuation without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant evacuation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant evacuation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-029 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-030: paging storm throttle
- Read: microservices/observability/manifest.json
- Connects to: capacity admission, saturation, and brownout behavior
- Build or inspect: a minimal artifact that proves paging storm throttle without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for paging storm throttle.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show paging storm throttle is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-cell with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-030 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-031: postmortem seal failure
- Read: docs/INCIDENT-MANAGEMENT.md
- Connects to: incident command roles and statuspage sync
- Build or inspect: a minimal artifact that proves postmortem seal failure without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for postmortem seal failure.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show postmortem seal failure is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-031 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-032: observability ingest lag
- Read: docs/RUNBOOKS-INDEX.md
- Connects to: Ops Dashboard Control Center operator flows
- Build or inspect: a minimal artifact that proves observability ingest lag without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for observability ingest lag.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show observability ingest lag is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-observability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-032 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-033: cluster health stale
- Read: docs/standards/observability.md
- Connects to: runbook freshness and branch completeness
- Build or inspect: a minimal artifact that proves cluster health stale without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cluster health stale.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cluster health stale is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-incident-management with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-033 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-034: brownout activation
- Read: docs/standards/observability-slo.md
- Connects to: OpenTelemetry traces, metrics, logs, and audit event joins
- Build or inspect: a minimal artifact that proves brownout activation without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for brownout activation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show brownout activation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ops-dashboard-control-center with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-034 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-035: runbook freshness check
- Read: docs/standards/on-call.md
- Connects to: SLO burn rate and error budget policy
- Build or inspect: a minimal artifact that proves runbook freshness check without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for runbook freshness check.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show runbook freshness check is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-035 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-036: regulated incident notification
- Read: microservices/ops-dashboard-control-center/README.md
- Connects to: cell failover and tenant evacuation
- Build or inspect: a minimal artifact that proves regulated incident notification without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regulated incident notification.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regulated incident notification is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-cell with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-036 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-037: page-to-ack burn
- Read: microservices/ops-dashboard-control-center/ARCHITECTURE.md
- Connects to: canary rollback and feature flag kill switch
- Build or inspect: a minimal artifact that proves page-to-ack burn without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for page-to-ack burn.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show page-to-ack burn is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-037 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-038: incident room not created
- Read: microservices/ops-dashboard-control-center/PRD.md
- Connects to: paging storm throttle and escalation policy corruption
- Build or inspect: a minimal artifact that proves incident room not created without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for incident room not created.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show incident room not created is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-observability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-038 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-039: dashboard perf degradation
- Read: microservices/ops-dashboard-control-center/runbooks/incident-command.md
- Connects to: postmortem sealing and evidence retention
- Build or inspect: a minimal artifact that proves dashboard perf degradation without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for dashboard perf degradation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show dashboard perf degradation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-incident-management with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-039 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-040: deployment rollback
- Read: microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md
- Connects to: capacity admission, saturation, and brownout behavior
- Build or inspect: a minimal artifact that proves deployment rollback without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for deployment rollback.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show deployment rollback is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ops-dashboard-control-center with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-040 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-041: on-call handoff failure
- Read: microservices/ops-dashboard-control-center/dashboards/ops-overview.json
- Connects to: incident command roles and statuspage sync
- Build or inspect: a minimal artifact that proves on-call handoff failure without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for on-call handoff failure.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show on-call handoff failure is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-041 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-042: statuspage sync gap
- Read: microservices/incident-management/README.md
- Connects to: Ops Dashboard Control Center operator flows
- Build or inspect: a minimal artifact that proves statuspage sync gap without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for statuspage sync gap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show statuspage sync gap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-cell with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-042 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-043: SLO error budget exhaustion
- Read: microservices/incident-management/contracts/openapi-v1.yaml
- Connects to: runbook freshness and branch completeness
- Build or inspect: a minimal artifact that proves SLO error budget exhaustion without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for SLO error budget exhaustion.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show SLO error budget exhaustion is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-043 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-044: metric cardinality spike
- Read: microservices/incident-management/slos/local-page-to-acknowledge.openslo.yaml
- Connects to: OpenTelemetry traces, metrics, logs, and audit event joins
- Build or inspect: a minimal artifact that proves metric cardinality spike without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for metric cardinality spike.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show metric cardinality spike is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-observability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-044 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-045: audit timeline gap
- Read: microservices/observability/manifest.json
- Connects to: SLO burn rate and error budget policy
- Build or inspect: a minimal artifact that proves audit timeline gap without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit timeline gap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit timeline gap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-incident-management with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-045 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-046: cell failover
- Read: docs/INCIDENT-MANAGEMENT.md
- Connects to: cell failover and tenant evacuation
- Build or inspect: a minimal artifact that proves cell failover without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cell failover.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cell failover is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ops-dashboard-control-center with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-046 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-047: tenant evacuation
- Read: docs/RUNBOOKS-INDEX.md
- Connects to: canary rollback and feature flag kill switch
- Build or inspect: a minimal artifact that proves tenant evacuation without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant evacuation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant evacuation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-047 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-048: paging storm throttle
- Read: docs/standards/observability.md
- Connects to: paging storm throttle and escalation policy corruption
- Build or inspect: a minimal artifact that proves paging storm throttle without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for paging storm throttle.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show paging storm throttle is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-cell with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-048 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-049: postmortem seal failure
- Read: docs/standards/observability-slo.md
- Connects to: postmortem sealing and evidence retention
- Build or inspect: a minimal artifact that proves postmortem seal failure without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for postmortem seal failure.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show postmortem seal failure is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-049 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-050: observability ingest lag
- Read: docs/standards/on-call.md
- Connects to: capacity admission, saturation, and brownout behavior
- Build or inspect: a minimal artifact that proves observability ingest lag without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for observability ingest lag.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show observability ingest lag is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-observability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-050 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-051: cluster health stale
- Read: microservices/ops-dashboard-control-center/README.md
- Connects to: incident command roles and statuspage sync
- Build or inspect: a minimal artifact that proves cluster health stale without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cluster health stale.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cluster health stale is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-incident-management with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-051 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-052: brownout activation
- Read: microservices/ops-dashboard-control-center/ARCHITECTURE.md
- Connects to: Ops Dashboard Control Center operator flows
- Build or inspect: a minimal artifact that proves brownout activation without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for brownout activation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show brownout activation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ops-dashboard-control-center with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-052 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-053: runbook freshness check
- Read: microservices/ops-dashboard-control-center/PRD.md
- Connects to: runbook freshness and branch completeness
- Build or inspect: a minimal artifact that proves runbook freshness check without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for runbook freshness check.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show runbook freshness check is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-053 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-054: regulated incident notification
- Read: microservices/ops-dashboard-control-center/runbooks/incident-command.md
- Connects to: OpenTelemetry traces, metrics, logs, and audit event joins
- Build or inspect: a minimal artifact that proves regulated incident notification without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regulated incident notification.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regulated incident notification is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-cell with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-054 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-055: page-to-ack burn
- Read: microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md
- Connects to: SLO burn rate and error budget policy
- Build or inspect: a minimal artifact that proves page-to-ack burn without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for page-to-ack burn.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show page-to-ack burn is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-055 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-056: incident room not created
- Read: microservices/ops-dashboard-control-center/dashboards/ops-overview.json
- Connects to: cell failover and tenant evacuation
- Build or inspect: a minimal artifact that proves incident room not created without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for incident room not created.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show incident room not created is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-observability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-056 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-057: dashboard perf degradation
- Read: microservices/incident-management/README.md
- Connects to: canary rollback and feature flag kill switch
- Build or inspect: a minimal artifact that proves dashboard perf degradation without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for dashboard perf degradation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show dashboard perf degradation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-incident-management with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-057 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-058: deployment rollback
- Read: microservices/incident-management/contracts/openapi-v1.yaml
- Connects to: paging storm throttle and escalation policy corruption
- Build or inspect: a minimal artifact that proves deployment rollback without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for deployment rollback.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show deployment rollback is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ops-dashboard-control-center with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-058 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-059: on-call handoff failure
- Read: microservices/incident-management/slos/local-page-to-acknowledge.openslo.yaml
- Connects to: postmortem sealing and evidence retention
- Build or inspect: a minimal artifact that proves on-call handoff failure without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for on-call handoff failure.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show on-call handoff failure is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-059 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-060: statuspage sync gap
- Read: microservices/observability/manifest.json
- Connects to: capacity admission, saturation, and brownout behavior
- Build or inspect: a minimal artifact that proves statuspage sync gap without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for statuspage sync gap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show statuspage sync gap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-cell with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-060 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-061: SLO error budget exhaustion
- Read: docs/INCIDENT-MANAGEMENT.md
- Connects to: incident command roles and statuspage sync
- Build or inspect: a minimal artifact that proves SLO error budget exhaustion without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for SLO error budget exhaustion.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show SLO error budget exhaustion is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-061 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-062: metric cardinality spike
- Read: docs/RUNBOOKS-INDEX.md
- Connects to: Ops Dashboard Control Center operator flows
- Build or inspect: a minimal artifact that proves metric cardinality spike without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for metric cardinality spike.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show metric cardinality spike is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-observability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-062 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-063: audit timeline gap
- Read: docs/standards/observability.md
- Connects to: runbook freshness and branch completeness
- Build or inspect: a minimal artifact that proves audit timeline gap without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit timeline gap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit timeline gap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-incident-management with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-063 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-064: cell failover
- Read: docs/standards/observability-slo.md
- Connects to: OpenTelemetry traces, metrics, logs, and audit event joins
- Build or inspect: a minimal artifact that proves cell failover without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cell failover.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cell failover is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ops-dashboard-control-center with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-064 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-065: tenant evacuation
- Read: docs/standards/on-call.md
- Connects to: SLO burn rate and error budget policy
- Build or inspect: a minimal artifact that proves tenant evacuation without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant evacuation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant evacuation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-065 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-066: paging storm throttle
- Read: microservices/ops-dashboard-control-center/README.md
- Connects to: cell failover and tenant evacuation
- Build or inspect: a minimal artifact that proves paging storm throttle without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for paging storm throttle.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show paging storm throttle is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-cell with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-066 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-067: postmortem seal failure
- Read: microservices/ops-dashboard-control-center/ARCHITECTURE.md
- Connects to: canary rollback and feature flag kill switch
- Build or inspect: a minimal artifact that proves postmortem seal failure without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for postmortem seal failure.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show postmortem seal failure is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-067 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-068: observability ingest lag
- Read: microservices/ops-dashboard-control-center/PRD.md
- Connects to: paging storm throttle and escalation policy corruption
- Build or inspect: a minimal artifact that proves observability ingest lag without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for observability ingest lag.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show observability ingest lag is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-observability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-068 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-069: cluster health stale
- Read: microservices/ops-dashboard-control-center/runbooks/incident-command.md
- Connects to: postmortem sealing and evidence retention
- Build or inspect: a minimal artifact that proves cluster health stale without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cluster health stale.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cluster health stale is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-incident-management with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-069 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-070: brownout activation
- Read: microservices/ops-dashboard-control-center/runbooks/deployment-rollback.md
- Connects to: capacity admission, saturation, and brownout behavior
- Build or inspect: a minimal artifact that proves brownout activation without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for brownout activation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show brownout activation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ops-dashboard-control-center with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-070 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-071: runbook freshness check
- Read: microservices/ops-dashboard-control-center/dashboards/ops-overview.json
- Connects to: incident command roles and statuspage sync
- Build or inspect: a minimal artifact that proves runbook freshness check without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for runbook freshness check.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show runbook freshness check is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-071 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-072: regulated incident notification
- Read: microservices/incident-management/README.md
- Connects to: Ops Dashboard Control Center operator flows
- Build or inspect: a minimal artifact that proves regulated incident notification without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regulated incident notification.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regulated incident notification is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-cell with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-072 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-073: page-to-ack burn
- Read: microservices/incident-management/contracts/openapi-v1.yaml
- Connects to: runbook freshness and branch completeness
- Build or inspect: a minimal artifact that proves page-to-ack burn without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for page-to-ack burn.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show page-to-ack burn is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-073 contains file path, claim, evidence, rollback, and reviewer.

### Drill SRE-074: incident room not created
- Read: microservices/incident-management/slos/local-page-to-acknowledge.openslo.yaml
- Connects to: OpenTelemetry traces, metrics, logs, and audit event joins
- Build or inspect: a minimal artifact that proves incident room not created without widening beyond SRE and on-call engineer, reliability operations.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for incident room not created.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show incident room not created is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-observability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SRE-074 contains file path, claim, evidence, rollback, and reviewer.

## Checkpoint Ledger

| Phase | Evidence | Reviewer |
| --- |--- |--- |
| Day 0 | access receipts and sandbox tenant id | mentor |
| Day 1 | environment setup command outputs | mentor |
| Week 1 | read-path and playground notes | mentor plus role owner |
| Week 2 | starter PR or checkpoint artifact | owning axis reviewer |
| Week 3-4 | independent project evidence | cross-team owner |
| Month 1 | owned project and contact map | manager |
| Month 2 | deep-dive notes | mentor plus council reviewer |
| Quarter 1 | OKR, rotation, and team contribution evidence | manager plus rotation owner |

Clean halt rule: when the required artifact exists, evidence is attached, rollback is named, and the reviewer has a concrete next action or no action, stop. Do not keep expanding scope to make the onboarding artifact look larger.
