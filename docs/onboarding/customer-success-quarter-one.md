---
doc_class: OnboardingGuide
role: "customer success manager, tenant onboarding and migration success"
status: Published
date: 2026-05-20
owner: "customer-success + gtm-operations + ops-compliance"
related_oyatie_adrs:
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
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

# Customer Success Quarter-One Onboarding

Audience: customer success manager, tenant onboarding and migration success.
Industry precedent: Salesforce enterprise success-plan discipline, Stripe migration readiness reviews, Atlassian enterprise cloud migration playbooks, and AWS Well-Architected customer review cadence.

This guide is written for a programming-capable new joiner with no prior Oyatie architecture knowledge. Every phase names the repo files to read, the artifact to produce, the owner who reviews it, and the stop condition that proves the phase is complete.

Substance rule: do not treat this guide as orientation prose. Treat it as a work plan whose outputs can be inspected, replayed, or rejected.

You join customer success to make per-tenant onboarding, migration journeys, FAQs, training, support escalation, and evidence-backed adoption concrete for every design partner and regulated tenant.
Quarter one is a customer-facing operational path: learn tenant onboarding, map migration playbooks, run FAQ drills, shadow escalations, and build one tenant success plan that engineering and compliance can execute.
The CSM role does not promise around platform gaps. It translates tenant need into an evidence-bound plan with owner, pack, migration step, rollback path, support channel, and adoption metric.

## Hyperscaler-Grade Reading Contract

- Named precedent: Salesforce enterprise success-plan discipline, Stripe migration readiness reviews, Atlassian enterprise cloud migration playbooks, and AWS Well-Architected customer review cadence.
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
1. docs/runbooks/tenant-onboarding.md
2. templates/checklists/tenant-onboarding.md
3. docs/runbooks/design-partner-onboarding.md
4. docs/runbooks/design-partner-feedback-session.md
5. docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md
6. docs/user-journeys/j96-ksa-uae-mena-tenant-onboarding/README.md
7. microservices/tenancy/runbooks/tenant-onboarding.md
8. microservices/tenancy/IP-journey-j145-cross-tenant-onboarding-overlay.md
9. microservices/workflow-studio/migration-playbooks/from-n8n.md
10. microservices/messenger/migration-playbooks/from-slack.md
11. microservices/mail/decisions/ADR-MAIL-0004-spam-classifier-eu-ai-act-scope.md
12. microservices/connector/migration-playbooks/from-slack-connect-and-teams-external.md
13. microservices/marketplace/migration-playbooks/from-stripe-connect.md
14. docs/runbooks/tenant-escalation-management.md
15. docs/runbooks/regulator-evidence-pack-regen.md


### Named ADRs to read

- ADR-0244 tenant scoping
- ADR-0316 capability tier activation
- ADR-0010 regional pack architecture
- ADR-0250 build-ahead-of-certification claim boundary
- ADR-0273 per-tenant email deliverability

### Named playgrounds

1. docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md
   - Artifact: write a four-sentence note explaining what this playground proves for customer success manager, tenant onboarding and migration success.
2. docs/runbooks/design-partner-feedback-session.md
   - Artifact: write a four-sentence note explaining what this playground proves for customer success manager, tenant onboarding and migration success.
3. microservices/workflow-studio/tutorials/build-customer-onboarding-flow.md
   - Artifact: write a four-sentence note explaining what this playground proves for customer success manager, tenant onboarding and migration success.
4. microservices/workflow-studio/faqs/no-code-builder-faq.md
   - Artifact: write a four-sentence note explaining what this playground proves for customer success manager, tenant onboarding and migration success.

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

1. CSM-STARTER-001 add a tenant-ready expected-output row to an onboarding checklist
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for CSM.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
2. CSM-STARTER-002 update one migration FAQ with rollback and escalation owner
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for CSM.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
3. CSM-STARTER-003 add a design-partner feedback question tied to a capability-tier metric
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for CSM.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
4. CSM-STARTER-004 map one customer objection to a documentation reference and support playbook
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for CSM.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.

### Mentor pairing protocol

Pair with a senior CSM for the first tenant plan, a PM for capability scope, and compliance for regulated claims. Every pairing ends with tenant state, migration step, adoption metric, support owner, and risk flag.
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

First independent project: Own `CSM-PROJ-001`: produce a per-tenant onboarding success plan for one design partner from kickoff through first action, migration rollback, FAQ handoff, and quarter-one adoption review.

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

- tenant onboarding GTM success plan
- migration journey FAQ library
- design partner feedback session improvements
- first-action adoption metric review

### Key contacts in other teams

- customer-success
- council-product
- axis-tenancy
- axis-workflow-studio
- ops-compliance
- ops-sre-reliability
- gtm-partnerships

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
1. tenant onboarding stages and evidence gates
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for tenant onboarding stages and evidence gates.
2. migration playbook rollback and validation
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for migration playbook rollback and validation.
3. FAQ design for technical and non-technical buyers
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for FAQ design for technical and non-technical buyers.
4. per-tenant regional pack and compliance claim boundary
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for per-tenant regional pack and compliance claim boundary.
5. first-action adoption and time-to-value measurement
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for first-action adoption and time-to-value measurement.
6. support escalation and incident handoff language
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for support escalation and incident handoff language.
7. design partner feedback capture and synthesis
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for design partner feedback capture and synthesis.
8. capability-tier packaging and activation constraints
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for capability-tier packaging and activation constraints.
9. email deliverability and identity readiness
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for email deliverability and identity readiness.
10. renewal risk and expansion signal capture
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for renewal risk and expansion signal capture.
Domain expertise stop condition: you can answer what breaks if this topic is implemented incorrectly, who owns the rollback, and what evidence proves the system recovered.
- Verification for Month 2: deep-dive notes cover every listed topic and at least one note is reviewed by a cross-team owner
- Stop condition for Month 2: mentor and owner can point to the artifact without asking you to explain hidden context.

## Quarter 1: Ownership


### Named OKRs

- OKR-CSM-Q1-1: publish one design-partner success plan accepted by PM, engineering, and compliance
- OKR-CSM-Q1-2: close one migration FAQ gap with evidence, rollback, and escalation owner
- OKR-CSM-Q1-3: run one tenant onboarding review from kickoff to first action with adoption metric evidence

### Named on-call rotation entry

Enter `customer-escalation-shadow` rotation in month two; quarter-one target is one migration escalation shadow and one regulated-tenant evidence review.

### Named team-OKR contribution

Contribute to `TEAM-OKR-CSM-2026Q2`: every design partner has success plan, migration map, FAQ set, first-action metric, and escalation path.

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

1. Promising a feature without mapping it to a capability tier and owner.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for customer success manager, tenant onboarding and migration success.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
2. Treating onboarding as kickoff-only rather than first-action plus adoption evidence.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for customer success manager, tenant onboarding and migration success.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
3. Hiding migration risk instead of naming rollback and validation steps.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for customer success manager, tenant onboarding and migration success.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
4. Answering regulated-tenant questions from memory instead of evidence-bound docs.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for customer success manager, tenant onboarding and migration success.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
5. Letting FAQs drift from current product and compliance state.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for customer success manager, tenant onboarding and migration success.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
6. Escalating to engineering without tenant state, reproduction path, and business impact.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for customer success manager, tenant onboarding and migration success.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
7. Confusing design-partner feedback with committed roadmap scope.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for customer success manager, tenant onboarding and migration success.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
8. Ignoring regional pack, identity, and email deliverability prerequisites.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for customer success manager, tenant onboarding and migration success.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.

## Cross-Team Collaboration Playbook

| Team | Handoff ritual | Minimum payload |
| --- |--- |--- |
| council-product | Capability scope handoff | Attach tier, target persona, first action, and out-of-scope list. |
| axis-tenancy | Tenant setup handoff | Attach tenant id, region, pack, identity state, and onboarding stage. |
| ops-compliance | Customer claim handoff | Attach evidence doc, allowed wording, and forbidden wording. |
| ops-sre-reliability | Incident escalation handoff | Attach customer impact, timeline, dashboard, and runbook branch. |
| gtm-partnerships | Commercial handoff | Attach contract constraint, success metric, and renewal risk. |
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
| first action | The first tenant workflow that proves activation beyond login. |
| success plan | Tenant-specific adoption, migration, risk, and evidence plan. |
| migration rollback | Documented path to return tenant to prior state without data loss. |
| design partner | Early customer whose feedback informs but does not override platform gates. |
| customer escalation | Structured path from customer impact to support, SRE, product, or compliance owner. |

## Escalation Channels

| Escalation | Use when | Owner |
| --- |--- |--- |
| mentor checkpoint | you can proceed locally but need review of reasoning or evidence | assigned mentor |
| axis owner | a file or policy belongs to another team | customer-success |
| council review | claim boundary, doctrine, compliance, or security interpretation changes | customer-success + gtm-operations + ops-compliance |
| SRE on-call | dev-cell, incident, or reliability path blocks verification | ops-sre-reliability |
| security review | credential, tenant isolation, policy, or regulated data risk appears | ops-security |

## Resources & References

- docs/runbooks/tenant-onboarding.md
- templates/checklists/tenant-onboarding.md
- docs/runbooks/design-partner-onboarding.md
- docs/runbooks/design-partner-feedback-session.md
- docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md
- microservices/workflow-studio/migration-playbooks/from-n8n.md
- docs/runbooks/tenant-escalation-management.md

Reference-reading protocol: open the resource, identify the authority section, write the one-sentence claim it supports, and record whether the resource is doctrine, spec, implementation, test, runbook, dashboard, or evidence.

## Role-Specific Drill Library

Use this ledger when you need extra practice or when a mentor asks for stronger evidence. Each drill is intentionally small but must end with a verifiable artifact.

### Drill CSM-001: tenant kickoff readiness
- Read: docs/runbooks/tenant-onboarding.md
- Connects to: tenant onboarding stages and evidence gates
- Build or inspect: a minimal artifact that proves tenant kickoff readiness without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant kickoff readiness.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant kickoff readiness is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-001 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-002: first-action journey
- Read: templates/checklists/tenant-onboarding.md
- Connects to: migration playbook rollback and validation
- Build or inspect: a minimal artifact that proves first-action journey without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for first-action journey.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show first-action journey is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-002 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-003: migration rollback FAQ
- Read: docs/runbooks/design-partner-onboarding.md
- Connects to: FAQ design for technical and non-technical buyers
- Build or inspect: a minimal artifact that proves migration rollback FAQ without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for migration rollback FAQ.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show migration rollback FAQ is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-003 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-004: design partner feedback
- Read: docs/runbooks/design-partner-feedback-session.md
- Connects to: per-tenant regional pack and compliance claim boundary
- Build or inspect: a minimal artifact that proves design partner feedback without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for design partner feedback.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show design partner feedback is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-004 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-005: regulated customer question
- Read: docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md
- Connects to: first-action adoption and time-to-value measurement
- Build or inspect: a minimal artifact that proves regulated customer question without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regulated customer question.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regulated customer question is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-005 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-006: email deliverability prerequisite
- Read: docs/user-journeys/j96-ksa-uae-mena-tenant-onboarding/README.md
- Connects to: support escalation and incident handoff language
- Build or inspect: a minimal artifact that proves email deliverability prerequisite without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for email deliverability prerequisite.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show email deliverability prerequisite is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-006 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-007: identity provider setup
- Read: microservices/tenancy/runbooks/tenant-onboarding.md
- Connects to: design partner feedback capture and synthesis
- Build or inspect: a minimal artifact that proves identity provider setup without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for identity provider setup.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show identity provider setup is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to gtm-partnerships with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-007 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-008: regional pack onboarding
- Read: microservices/tenancy/IP-journey-j145-cross-tenant-onboarding-overlay.md
- Connects to: capability-tier packaging and activation constraints
- Build or inspect: a minimal artifact that proves regional pack onboarding without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regional pack onboarding.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regional pack onboarding is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-008 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-009: support escalation handoff
- Read: microservices/workflow-studio/migration-playbooks/from-n8n.md
- Connects to: email deliverability and identity readiness
- Build or inspect: a minimal artifact that proves support escalation handoff without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for support escalation handoff.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show support escalation handoff is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-009 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-010: renewal risk signal
- Read: microservices/messenger/migration-playbooks/from-slack.md
- Connects to: renewal risk and expansion signal capture
- Build or inspect: a minimal artifact that proves renewal risk signal without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for renewal risk signal.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show renewal risk signal is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-010 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-011: capability tier explanation
- Read: microservices/mail/decisions/ADR-MAIL-0004-spam-classifier-eu-ai-act-scope.md
- Connects to: tenant onboarding stages and evidence gates
- Build or inspect: a minimal artifact that proves capability tier explanation without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for capability tier explanation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show capability tier explanation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-011 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-012: customer objection mapping
- Read: microservices/connector/migration-playbooks/from-slack-connect-and-teams-external.md
- Connects to: migration playbook rollback and validation
- Build or inspect: a minimal artifact that proves customer objection mapping without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for customer objection mapping.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show customer objection mapping is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-012 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-013: tenant success metric
- Read: microservices/marketplace/migration-playbooks/from-stripe-connect.md
- Connects to: FAQ design for technical and non-technical buyers
- Build or inspect: a minimal artifact that proves tenant success metric without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant success metric.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant success metric is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-013 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-014: migration validation evidence
- Read: docs/runbooks/tenant-escalation-management.md
- Connects to: per-tenant regional pack and compliance claim boundary
- Build or inspect: a minimal artifact that proves migration validation evidence without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for migration validation evidence.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show migration validation evidence is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to gtm-partnerships with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-014 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-015: trust portal evidence request
- Read: docs/runbooks/regulator-evidence-pack-regen.md
- Connects to: first-action adoption and time-to-value measurement
- Build or inspect: a minimal artifact that proves trust portal evidence request without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for trust portal evidence request.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show trust portal evidence request is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-015 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-016: product scope boundary
- Read: docs/runbooks/tenant-onboarding.md
- Connects to: support escalation and incident handoff language
- Build or inspect: a minimal artifact that proves product scope boundary without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for product scope boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show product scope boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-016 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-017: onboarding checklist gap
- Read: templates/checklists/tenant-onboarding.md
- Connects to: design partner feedback capture and synthesis
- Build or inspect: a minimal artifact that proves onboarding checklist gap without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for onboarding checklist gap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show onboarding checklist gap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-017 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-018: FAQ drift review
- Read: docs/runbooks/design-partner-onboarding.md
- Connects to: capability-tier packaging and activation constraints
- Build or inspect: a minimal artifact that proves FAQ drift review without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for FAQ drift review.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show FAQ drift review is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-018 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-019: tenant kickoff readiness
- Read: docs/runbooks/design-partner-feedback-session.md
- Connects to: email deliverability and identity readiness
- Build or inspect: a minimal artifact that proves tenant kickoff readiness without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant kickoff readiness.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant kickoff readiness is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-019 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-020: first-action journey
- Read: docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md
- Connects to: renewal risk and expansion signal capture
- Build or inspect: a minimal artifact that proves first-action journey without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for first-action journey.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show first-action journey is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-020 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-021: migration rollback FAQ
- Read: docs/user-journeys/j96-ksa-uae-mena-tenant-onboarding/README.md
- Connects to: tenant onboarding stages and evidence gates
- Build or inspect: a minimal artifact that proves migration rollback FAQ without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for migration rollback FAQ.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show migration rollback FAQ is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to gtm-partnerships with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-021 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-022: design partner feedback
- Read: microservices/tenancy/runbooks/tenant-onboarding.md
- Connects to: migration playbook rollback and validation
- Build or inspect: a minimal artifact that proves design partner feedback without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for design partner feedback.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show design partner feedback is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-022 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-023: regulated customer question
- Read: microservices/tenancy/IP-journey-j145-cross-tenant-onboarding-overlay.md
- Connects to: FAQ design for technical and non-technical buyers
- Build or inspect: a minimal artifact that proves regulated customer question without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regulated customer question.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regulated customer question is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-023 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-024: email deliverability prerequisite
- Read: microservices/workflow-studio/migration-playbooks/from-n8n.md
- Connects to: per-tenant regional pack and compliance claim boundary
- Build or inspect: a minimal artifact that proves email deliverability prerequisite without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for email deliverability prerequisite.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show email deliverability prerequisite is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-024 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-025: identity provider setup
- Read: microservices/messenger/migration-playbooks/from-slack.md
- Connects to: first-action adoption and time-to-value measurement
- Build or inspect: a minimal artifact that proves identity provider setup without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for identity provider setup.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show identity provider setup is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-025 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-026: regional pack onboarding
- Read: microservices/mail/decisions/ADR-MAIL-0004-spam-classifier-eu-ai-act-scope.md
- Connects to: support escalation and incident handoff language
- Build or inspect: a minimal artifact that proves regional pack onboarding without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regional pack onboarding.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regional pack onboarding is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-026 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-027: support escalation handoff
- Read: microservices/connector/migration-playbooks/from-slack-connect-and-teams-external.md
- Connects to: design partner feedback capture and synthesis
- Build or inspect: a minimal artifact that proves support escalation handoff without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for support escalation handoff.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show support escalation handoff is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-027 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-028: renewal risk signal
- Read: microservices/marketplace/migration-playbooks/from-stripe-connect.md
- Connects to: capability-tier packaging and activation constraints
- Build or inspect: a minimal artifact that proves renewal risk signal without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for renewal risk signal.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show renewal risk signal is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to gtm-partnerships with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-028 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-029: capability tier explanation
- Read: docs/runbooks/tenant-escalation-management.md
- Connects to: email deliverability and identity readiness
- Build or inspect: a minimal artifact that proves capability tier explanation without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for capability tier explanation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show capability tier explanation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-029 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-030: customer objection mapping
- Read: docs/runbooks/regulator-evidence-pack-regen.md
- Connects to: renewal risk and expansion signal capture
- Build or inspect: a minimal artifact that proves customer objection mapping without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for customer objection mapping.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show customer objection mapping is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-030 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-031: tenant success metric
- Read: docs/runbooks/tenant-onboarding.md
- Connects to: tenant onboarding stages and evidence gates
- Build or inspect: a minimal artifact that proves tenant success metric without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant success metric.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant success metric is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-031 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-032: migration validation evidence
- Read: templates/checklists/tenant-onboarding.md
- Connects to: migration playbook rollback and validation
- Build or inspect: a minimal artifact that proves migration validation evidence without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for migration validation evidence.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show migration validation evidence is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-032 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-033: trust portal evidence request
- Read: docs/runbooks/design-partner-onboarding.md
- Connects to: FAQ design for technical and non-technical buyers
- Build or inspect: a minimal artifact that proves trust portal evidence request without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for trust portal evidence request.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show trust portal evidence request is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-033 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-034: product scope boundary
- Read: docs/runbooks/design-partner-feedback-session.md
- Connects to: per-tenant regional pack and compliance claim boundary
- Build or inspect: a minimal artifact that proves product scope boundary without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for product scope boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show product scope boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-034 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-035: onboarding checklist gap
- Read: docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md
- Connects to: first-action adoption and time-to-value measurement
- Build or inspect: a minimal artifact that proves onboarding checklist gap without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for onboarding checklist gap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show onboarding checklist gap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to gtm-partnerships with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-035 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-036: FAQ drift review
- Read: docs/user-journeys/j96-ksa-uae-mena-tenant-onboarding/README.md
- Connects to: support escalation and incident handoff language
- Build or inspect: a minimal artifact that proves FAQ drift review without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for FAQ drift review.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show FAQ drift review is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-036 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-037: tenant kickoff readiness
- Read: microservices/tenancy/runbooks/tenant-onboarding.md
- Connects to: design partner feedback capture and synthesis
- Build or inspect: a minimal artifact that proves tenant kickoff readiness without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant kickoff readiness.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant kickoff readiness is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-037 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-038: first-action journey
- Read: microservices/tenancy/IP-journey-j145-cross-tenant-onboarding-overlay.md
- Connects to: capability-tier packaging and activation constraints
- Build or inspect: a minimal artifact that proves first-action journey without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for first-action journey.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show first-action journey is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-038 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-039: migration rollback FAQ
- Read: microservices/workflow-studio/migration-playbooks/from-n8n.md
- Connects to: email deliverability and identity readiness
- Build or inspect: a minimal artifact that proves migration rollback FAQ without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for migration rollback FAQ.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show migration rollback FAQ is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-039 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-040: design partner feedback
- Read: microservices/messenger/migration-playbooks/from-slack.md
- Connects to: renewal risk and expansion signal capture
- Build or inspect: a minimal artifact that proves design partner feedback without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for design partner feedback.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show design partner feedback is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-040 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-041: regulated customer question
- Read: microservices/mail/decisions/ADR-MAIL-0004-spam-classifier-eu-ai-act-scope.md
- Connects to: tenant onboarding stages and evidence gates
- Build or inspect: a minimal artifact that proves regulated customer question without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regulated customer question.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regulated customer question is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-041 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-042: email deliverability prerequisite
- Read: microservices/connector/migration-playbooks/from-slack-connect-and-teams-external.md
- Connects to: migration playbook rollback and validation
- Build or inspect: a minimal artifact that proves email deliverability prerequisite without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for email deliverability prerequisite.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show email deliverability prerequisite is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to gtm-partnerships with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-042 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-043: identity provider setup
- Read: microservices/marketplace/migration-playbooks/from-stripe-connect.md
- Connects to: FAQ design for technical and non-technical buyers
- Build or inspect: a minimal artifact that proves identity provider setup without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for identity provider setup.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show identity provider setup is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-043 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-044: regional pack onboarding
- Read: docs/runbooks/tenant-escalation-management.md
- Connects to: per-tenant regional pack and compliance claim boundary
- Build or inspect: a minimal artifact that proves regional pack onboarding without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regional pack onboarding.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regional pack onboarding is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-044 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-045: support escalation handoff
- Read: docs/runbooks/regulator-evidence-pack-regen.md
- Connects to: first-action adoption and time-to-value measurement
- Build or inspect: a minimal artifact that proves support escalation handoff without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for support escalation handoff.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show support escalation handoff is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-045 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-046: renewal risk signal
- Read: docs/runbooks/tenant-onboarding.md
- Connects to: support escalation and incident handoff language
- Build or inspect: a minimal artifact that proves renewal risk signal without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for renewal risk signal.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show renewal risk signal is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-046 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-047: capability tier explanation
- Read: templates/checklists/tenant-onboarding.md
- Connects to: design partner feedback capture and synthesis
- Build or inspect: a minimal artifact that proves capability tier explanation without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for capability tier explanation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show capability tier explanation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-047 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-048: customer objection mapping
- Read: docs/runbooks/design-partner-onboarding.md
- Connects to: capability-tier packaging and activation constraints
- Build or inspect: a minimal artifact that proves customer objection mapping without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for customer objection mapping.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show customer objection mapping is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-048 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-049: tenant success metric
- Read: docs/runbooks/design-partner-feedback-session.md
- Connects to: email deliverability and identity readiness
- Build or inspect: a minimal artifact that proves tenant success metric without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant success metric.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant success metric is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to gtm-partnerships with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-049 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-050: migration validation evidence
- Read: docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md
- Connects to: renewal risk and expansion signal capture
- Build or inspect: a minimal artifact that proves migration validation evidence without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for migration validation evidence.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show migration validation evidence is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-050 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-051: trust portal evidence request
- Read: docs/user-journeys/j96-ksa-uae-mena-tenant-onboarding/README.md
- Connects to: tenant onboarding stages and evidence gates
- Build or inspect: a minimal artifact that proves trust portal evidence request without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for trust portal evidence request.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show trust portal evidence request is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-051 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-052: product scope boundary
- Read: microservices/tenancy/runbooks/tenant-onboarding.md
- Connects to: migration playbook rollback and validation
- Build or inspect: a minimal artifact that proves product scope boundary without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for product scope boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show product scope boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-052 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-053: onboarding checklist gap
- Read: microservices/tenancy/IP-journey-j145-cross-tenant-onboarding-overlay.md
- Connects to: FAQ design for technical and non-technical buyers
- Build or inspect: a minimal artifact that proves onboarding checklist gap without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for onboarding checklist gap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show onboarding checklist gap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-053 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-054: FAQ drift review
- Read: microservices/workflow-studio/migration-playbooks/from-n8n.md
- Connects to: per-tenant regional pack and compliance claim boundary
- Build or inspect: a minimal artifact that proves FAQ drift review without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for FAQ drift review.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show FAQ drift review is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-054 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-055: tenant kickoff readiness
- Read: microservices/messenger/migration-playbooks/from-slack.md
- Connects to: first-action adoption and time-to-value measurement
- Build or inspect: a minimal artifact that proves tenant kickoff readiness without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant kickoff readiness.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant kickoff readiness is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-055 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-056: first-action journey
- Read: microservices/mail/decisions/ADR-MAIL-0004-spam-classifier-eu-ai-act-scope.md
- Connects to: support escalation and incident handoff language
- Build or inspect: a minimal artifact that proves first-action journey without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for first-action journey.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show first-action journey is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to gtm-partnerships with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-056 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-057: migration rollback FAQ
- Read: microservices/connector/migration-playbooks/from-slack-connect-and-teams-external.md
- Connects to: design partner feedback capture and synthesis
- Build or inspect: a minimal artifact that proves migration rollback FAQ without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for migration rollback FAQ.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show migration rollback FAQ is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-057 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-058: design partner feedback
- Read: microservices/marketplace/migration-playbooks/from-stripe-connect.md
- Connects to: capability-tier packaging and activation constraints
- Build or inspect: a minimal artifact that proves design partner feedback without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for design partner feedback.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show design partner feedback is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-058 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-059: regulated customer question
- Read: docs/runbooks/tenant-escalation-management.md
- Connects to: email deliverability and identity readiness
- Build or inspect: a minimal artifact that proves regulated customer question without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regulated customer question.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regulated customer question is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-059 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-060: email deliverability prerequisite
- Read: docs/runbooks/regulator-evidence-pack-regen.md
- Connects to: renewal risk and expansion signal capture
- Build or inspect: a minimal artifact that proves email deliverability prerequisite without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for email deliverability prerequisite.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show email deliverability prerequisite is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-060 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-061: identity provider setup
- Read: docs/runbooks/tenant-onboarding.md
- Connects to: tenant onboarding stages and evidence gates
- Build or inspect: a minimal artifact that proves identity provider setup without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for identity provider setup.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show identity provider setup is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-061 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-062: regional pack onboarding
- Read: templates/checklists/tenant-onboarding.md
- Connects to: migration playbook rollback and validation
- Build or inspect: a minimal artifact that proves regional pack onboarding without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regional pack onboarding.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regional pack onboarding is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-062 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-063: support escalation handoff
- Read: docs/runbooks/design-partner-onboarding.md
- Connects to: FAQ design for technical and non-technical buyers
- Build or inspect: a minimal artifact that proves support escalation handoff without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for support escalation handoff.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show support escalation handoff is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to gtm-partnerships with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-063 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-064: renewal risk signal
- Read: docs/runbooks/design-partner-feedback-session.md
- Connects to: per-tenant regional pack and compliance claim boundary
- Build or inspect: a minimal artifact that proves renewal risk signal without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for renewal risk signal.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show renewal risk signal is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-064 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-065: capability tier explanation
- Read: docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md
- Connects to: first-action adoption and time-to-value measurement
- Build or inspect: a minimal artifact that proves capability tier explanation without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for capability tier explanation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show capability tier explanation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-065 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-066: customer objection mapping
- Read: docs/user-journeys/j96-ksa-uae-mena-tenant-onboarding/README.md
- Connects to: support escalation and incident handoff language
- Build or inspect: a minimal artifact that proves customer objection mapping without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for customer objection mapping.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show customer objection mapping is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-066 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-067: tenant success metric
- Read: microservices/tenancy/runbooks/tenant-onboarding.md
- Connects to: design partner feedback capture and synthesis
- Build or inspect: a minimal artifact that proves tenant success metric without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant success metric.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant success metric is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-067 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-068: migration validation evidence
- Read: microservices/tenancy/IP-journey-j145-cross-tenant-onboarding-overlay.md
- Connects to: capability-tier packaging and activation constraints
- Build or inspect: a minimal artifact that proves migration validation evidence without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for migration validation evidence.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show migration validation evidence is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-068 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-069: trust portal evidence request
- Read: microservices/workflow-studio/migration-playbooks/from-n8n.md
- Connects to: email deliverability and identity readiness
- Build or inspect: a minimal artifact that proves trust portal evidence request without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for trust portal evidence request.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show trust portal evidence request is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-069 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-070: product scope boundary
- Read: microservices/messenger/migration-playbooks/from-slack.md
- Connects to: renewal risk and expansion signal capture
- Build or inspect: a minimal artifact that proves product scope boundary without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for product scope boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show product scope boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to gtm-partnerships with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-070 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-071: onboarding checklist gap
- Read: microservices/mail/decisions/ADR-MAIL-0004-spam-classifier-eu-ai-act-scope.md
- Connects to: tenant onboarding stages and evidence gates
- Build or inspect: a minimal artifact that proves onboarding checklist gap without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for onboarding checklist gap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show onboarding checklist gap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-071 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-072: FAQ drift review
- Read: microservices/connector/migration-playbooks/from-slack-connect-and-teams-external.md
- Connects to: migration playbook rollback and validation
- Build or inspect: a minimal artifact that proves FAQ drift review without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for FAQ drift review.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show FAQ drift review is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-072 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-073: tenant kickoff readiness
- Read: microservices/marketplace/migration-playbooks/from-stripe-connect.md
- Connects to: FAQ design for technical and non-technical buyers
- Build or inspect: a minimal artifact that proves tenant kickoff readiness without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant kickoff readiness.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant kickoff readiness is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-073 contains file path, claim, evidence, rollback, and reviewer.

### Drill CSM-074: first-action journey
- Read: docs/runbooks/tenant-escalation-management.md
- Connects to: per-tenant regional pack and compliance claim boundary
- Build or inspect: a minimal artifact that proves first-action journey without widening beyond customer success manager, tenant onboarding and migration success.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for first-action journey.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show first-action journey is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row CSM-074 contains file path, claim, evidence, rollback, and reviewer.

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
