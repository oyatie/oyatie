---
doc_class: OnboardingGuide
role: "product manager, capability and migration product lane"
status: Published
date: 2026-05-20
owner: "council-product + axis-product-ops"
related_oyatie_adrs:
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0705-product-protocol-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
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

# Product Manager Month-One Onboarding

Audience: product manager, capability and migration product lane.
Industry precedent: Amazon working-backwards PRFAQ, Stripe capability packaging, Palantir ontology-backed product operations, and Atlassian migration playbook discipline.

This guide is written for a programming-capable new joiner with no prior Oyatie architecture knowledge. Every phase names the repo files to read, the artifact to produce, the owner who reviews it, and the stop condition that proves the phase is complete.

Substance rule: do not treat this guide as orientation prose. Treat it as a work plan whose outputs can be inspected, replayed, or rejected.

You join product to translate personas, journeys, microservices, capability tiers, compliance packs, and migration playbooks into deliverable scope that engineering can build without guessing.
Month one is grounded in the persona x journey x microservice coverage matrix and ADR-0316 capability-tier doctrine. The product role owns activation bundles, not product-fragment sprawl.
Your first deliverables must name the customer problem, user journey, capability tier, Cedar permits, ontology projection, workflow template, UX shell, compliance overlay, migration path, and measurable success criteria.

## Hyperscaler-Grade Reading Contract

- Named precedent: Amazon working-backwards PRFAQ, Stripe capability packaging, Palantir ontology-backed product operations, and Atlassian migration playbook discipline.
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
1. docs/architecture/persona-journey-microservice-cross-coverage-matrix-2026-05-21.md
2. docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
3. docs/decisions/ADR-0709-general-live-apex.md
4. registry/capability-tiers/index.json
5. registry/capability-tiers/gold.json
6. registry/capability-tiers/platinum.json
7. microservices/workflow-studio/capability-tiers/tier-matrix.md
8. microservices/tenancy/README.md
9. docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md
10. microservices/workflow-studio/migration-playbooks/from-n8n.md
11. microservices/messenger/migration-playbooks/from-slack.md
12. microservices/intelligence/migration-playbooks/from-github-merge-queue-and-bors.md
13. docs/standards/prfaq-template.md
14. docs/products/erp-coverage/PRD.md
15. specs/masterplan.json


### Named ADRs to read

- ADR-0316 capability-tier over product fragmentation
- ADR-0245 substrate versus product layering
- ADR-0249 marketplace categories and overlays
- ADR-0257 ontology versioning and deprecation handshake
- ADR-0315 ERP coverage doctrine

### Named playgrounds

1. docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md
   - Artifact: write a four-sentence note explaining what this playground proves for product manager, capability and migration product lane.
2. microservices/workflow-studio/templates/definitions/operations/oya-workflow-studio-template-vendor-onboarding.json
   - Artifact: write a four-sentence note explaining what this playground proves for product manager, capability and migration product lane.
3. registry/capability-tiers/checkpoint.json
   - Artifact: write a four-sentence note explaining what this playground proves for product manager, capability and migration product lane.
4. docs/standards/prfaq-template.md
   - Artifact: write a four-sentence note explaining what this playground proves for product manager, capability and migration product lane.

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

1. PM-STARTER-001 add missing success metric wording to one capability-tier matrix
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for PM.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
2. PM-STARTER-002 map one journey to its microservice, Cedar permit, and migration playbook
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for PM.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
3. PM-STARTER-003 tighten one migration playbook persona and acceptance criterion
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for PM.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
4. PM-STARTER-004 draft a PRFAQ appendix for a capability activation bundle without creating a new microservice
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for PM.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.

### Mentor pairing protocol

Pair with a product council mentor for scope, an engineering lead for implementation path, and a compliance partner for regulated claims. Every pairing ends with a one-page capability brief and explicit out-of-scope boundary.
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

First independent project: Own `PM-PROJ-001`: define one capability activation bundle under ADR-0316, including persona, journey, microservice owner, Cedar permits, ontology projections, workflow templates, migration playbook, and tier success metrics.

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

- persona x journey x microservice matrix review
- ADR-0316 capability-tier activation bundle
- migration playbook acceptance criteria pass
- PRFAQ for a product surface without microservice fragmentation

### Key contacts in other teams

- council-product
- axis-workflow-studio
- axis-ontology
- axis-tenancy
- axis-policy-engine
- ops-compliance
- customer-success

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
1. capability tier as activation bundle
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for capability tier as activation bundle.
2. persona x journey x microservice coverage model
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for persona x journey x microservice coverage model.
3. migration playbook shape and rollback expectations
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for migration playbook shape and rollback expectations.
4. Cedar permit set as product entitlement boundary
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for Cedar permit set as product entitlement boundary.
5. ontology projection pinning for product scope
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for ontology projection pinning for product scope.
6. workflow template coverage for first action
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for workflow template coverage for first action.
7. UX shell manifest and localization obligations
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for UX shell manifest and localization obligations.
8. compliance overlay and claim boundary
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for compliance overlay and claim boundary.
9. pricing, packaging, and tenant eligibility
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for pricing, packaging, and tenant eligibility.
10. success metrics tied to observable behavior
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for success metrics tied to observable behavior.
Domain expertise stop condition: you can answer what breaks if this topic is implemented incorrectly, who owns the rollback, and what evidence proves the system recovered.
- Verification for Month 2: deep-dive notes cover every listed topic and at least one note is reviewed by a cross-team owner
- Stop condition for Month 2: mentor and owner can point to the artifact without asking you to explain hidden context.

## Quarter 1: Ownership


### Named OKRs

- OKR-PM-Q1-1: ship one capability bundle brief accepted by engineering, compliance, and customer-success
- OKR-PM-Q1-2: close one journey coverage gap with acceptance criteria and migration path
- OKR-PM-Q1-3: prevent one proposed product fragment from becoming a new microservice by mapping it to ADR-0316 tier doctrine

### Named on-call rotation entry

Enter `product-escalation-shadow` rotation in month two; quarter-one target is one customer migration escalation and one capability-tier rollback review.

### Named team-OKR contribution

Contribute to `TEAM-OKR-PRODUCT-2026Q2`: all first-deliverable product scope is expressed as capability tiers with journey, migration, compliance, and telemetry evidence.

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

1. Inventing a new product microservice when a capability tier is the correct surface.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for product manager, capability and migration product lane.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
2. Writing persona stories that do not map to a journey and microservice owner.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for product manager, capability and migration product lane.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
3. Treating migration as a sales checklist instead of a rollback-capable product flow.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for product manager, capability and migration product lane.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
4. Skipping Cedar permits, ontology projections, or workflow templates in product scope.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for product manager, capability and migration product lane.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
5. Using competitor parity as a feature list without measurable success criteria.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for product manager, capability and migration product lane.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
6. Making compliance claims before evidence and owner are named.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for product manager, capability and migration product lane.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
7. Separating UX shell decisions from locale and accessibility obligations.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for product manager, capability and migration product lane.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
8. Prioritizing roadmap optics over first-action adoption and tenant activation evidence.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for product manager, capability and migration product lane.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.

## Cross-Team Collaboration Playbook

| Team | Handoff ritual | Minimum payload |
| --- |--- |--- |
| axis-ontology | Capability model handoff | Attach object types, projection pins, and deprecation impact. |
| axis-policy-engine | Entitlement handoff | Attach Cedar permit set, deny cases, and tier eligibility. |
| axis-workflow-engine | Journey handoff | Attach workflow template, state machine, and rollback path. |
| customer-success | Migration handoff | Attach tenant cohort, success plan, FAQ, and training artifact. |
| ops-compliance | Claim handoff | Attach compliance pack, evidence id, and forbidden claim list. |
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
| capability tier | Activation bundle made of permits, projections, templates, UX, compliance, and telemetry. |
| persona x journey x microservice | Coverage model connecting user, workflow, and owning service. |
| migration playbook | Stepwise customer move from incumbent system to Oyatie with rollback and validation. |
| PRFAQ | Working-backwards product artifact that clarifies customer value before build. |
| claim boundary | Exact claim supported by current product and evidence state. |

## Escalation Channels

| Escalation | Use when | Owner |
| --- |--- |--- |
| mentor checkpoint | you can proceed locally but need review of reasoning or evidence | assigned mentor |
| axis owner | a file or policy belongs to another team | council-product |
| council review | claim boundary, doctrine, compliance, or security interpretation changes | council-product + axis-product-ops |
| SRE on-call | dev-cell, incident, or reliability path blocks verification | ops-sre-reliability |
| security review | credential, tenant isolation, policy, or regulated data risk appears | ops-security |

## Resources & References

- docs/standards/prfaq-template.md
- docs/architecture/persona-journey-microservice-cross-coverage-matrix-2026-05-21.md
- docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
- registry/capability-tiers/index.json
- docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md
- microservices/workflow-studio/migration-playbooks/from-n8n.md
- docs/decisions/ADR-0709-general-live-apex.md

Reference-reading protocol: open the resource, identify the authority section, write the one-sentence claim it supports, and record whether the resource is doctrine, spec, implementation, test, runbook, dashboard, or evidence.

## Role-Specific Drill Library

Use this ledger when you need extra practice or when a mentor asks for stronger evidence. Each drill is intentionally small but must end with a verifiable artifact.

### Drill PM-001: persona journey mapping
- Read: docs/architecture/persona-journey-microservice-cross-coverage-matrix-2026-05-21.md
- Connects to: capability tier as activation bundle
- Build or inspect: a minimal artifact that proves persona journey mapping without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for persona journey mapping.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show persona journey mapping is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-001 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-002: capability tier brief
- Read: docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
- Connects to: persona x journey x microservice coverage model
- Build or inspect: a minimal artifact that proves capability tier brief without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for capability tier brief.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show capability tier brief is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-002 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-003: migration acceptance criteria
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: migration playbook shape and rollback expectations
- Build or inspect: a minimal artifact that proves migration acceptance criteria without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for migration acceptance criteria.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show migration acceptance criteria is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-003 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-004: Cedar entitlement matrix
- Read: registry/capability-tiers/index.json
- Connects to: Cedar permit set as product entitlement boundary
- Build or inspect: a minimal artifact that proves Cedar entitlement matrix without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar entitlement matrix.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar entitlement matrix is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-004 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-005: ontology projection pin
- Read: registry/capability-tiers/gold.json
- Connects to: ontology projection pinning for product scope
- Build or inspect: a minimal artifact that proves ontology projection pin without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ontology projection pin.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ontology projection pin is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-005 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-006: workflow template coverage
- Read: registry/capability-tiers/platinum.json
- Connects to: workflow template coverage for first action
- Build or inspect: a minimal artifact that proves workflow template coverage without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for workflow template coverage.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show workflow template coverage is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-006 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-007: UX shell localization
- Read: microservices/workflow-studio/capability-tiers/tier-matrix.md
- Connects to: UX shell manifest and localization obligations
- Build or inspect: a minimal artifact that proves UX shell localization without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for UX shell localization.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show UX shell localization is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-007 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-008: competitor parity metric
- Read: microservices/tenancy/README.md
- Connects to: compliance overlay and claim boundary
- Build or inspect: a minimal artifact that proves competitor parity metric without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for competitor parity metric.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show competitor parity metric is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-008 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-009: first-action activation
- Read: docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md
- Connects to: pricing, packaging, and tenant eligibility
- Build or inspect: a minimal artifact that proves first-action activation without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for first-action activation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show first-action activation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-009 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-010: tenant cohort eligibility
- Read: microservices/workflow-studio/migration-playbooks/from-n8n.md
- Connects to: success metrics tied to observable behavior
- Build or inspect: a minimal artifact that proves tenant cohort eligibility without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant cohort eligibility.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant cohort eligibility is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-010 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-011: compliance claim boundary
- Read: microservices/messenger/migration-playbooks/from-slack.md
- Connects to: capability tier as activation bundle
- Build or inspect: a minimal artifact that proves compliance claim boundary without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for compliance claim boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show compliance claim boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-011 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-012: pricing package guardrail
- Read: microservices/intelligence/migration-playbooks/from-github-merge-queue-and-bors.md
- Connects to: persona x journey x microservice coverage model
- Build or inspect: a minimal artifact that proves pricing package guardrail without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for pricing package guardrail.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show pricing package guardrail is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-012 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-013: migration rollback branch
- Read: docs/standards/prfaq-template.md
- Connects to: migration playbook shape and rollback expectations
- Build or inspect: a minimal artifact that proves migration rollback branch without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for migration rollback branch.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show migration rollback branch is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-013 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-014: product fragment refusal
- Read: docs/products/erp-coverage/PRD.md
- Connects to: Cedar permit set as product entitlement boundary
- Build or inspect: a minimal artifact that proves product fragment refusal without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for product fragment refusal.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show product fragment refusal is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-014 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-015: journey success metric
- Read: specs/masterplan.json
- Connects to: ontology projection pinning for product scope
- Build or inspect: a minimal artifact that proves journey success metric without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for journey success metric.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show journey success metric is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-015 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-016: PRFAQ FAQ answer
- Read: docs/architecture/persona-journey-microservice-cross-coverage-matrix-2026-05-21.md
- Connects to: workflow template coverage for first action
- Build or inspect: a minimal artifact that proves PRFAQ FAQ answer without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for PRFAQ FAQ answer.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show PRFAQ FAQ answer is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-016 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-017: customer escalation script
- Read: docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
- Connects to: UX shell manifest and localization obligations
- Build or inspect: a minimal artifact that proves customer escalation script without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for customer escalation script.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show customer escalation script is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-017 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-018: roadmap dependency ordering
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: compliance overlay and claim boundary
- Build or inspect: a minimal artifact that proves roadmap dependency ordering without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for roadmap dependency ordering.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show roadmap dependency ordering is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-018 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-019: persona journey mapping
- Read: registry/capability-tiers/index.json
- Connects to: pricing, packaging, and tenant eligibility
- Build or inspect: a minimal artifact that proves persona journey mapping without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for persona journey mapping.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show persona journey mapping is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-019 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-020: capability tier brief
- Read: registry/capability-tiers/gold.json
- Connects to: success metrics tied to observable behavior
- Build or inspect: a minimal artifact that proves capability tier brief without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for capability tier brief.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show capability tier brief is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-020 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-021: migration acceptance criteria
- Read: registry/capability-tiers/platinum.json
- Connects to: capability tier as activation bundle
- Build or inspect: a minimal artifact that proves migration acceptance criteria without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for migration acceptance criteria.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show migration acceptance criteria is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-021 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-022: Cedar entitlement matrix
- Read: microservices/workflow-studio/capability-tiers/tier-matrix.md
- Connects to: persona x journey x microservice coverage model
- Build or inspect: a minimal artifact that proves Cedar entitlement matrix without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar entitlement matrix.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar entitlement matrix is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-022 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-023: ontology projection pin
- Read: microservices/tenancy/README.md
- Connects to: migration playbook shape and rollback expectations
- Build or inspect: a minimal artifact that proves ontology projection pin without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ontology projection pin.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ontology projection pin is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-023 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-024: workflow template coverage
- Read: docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md
- Connects to: Cedar permit set as product entitlement boundary
- Build or inspect: a minimal artifact that proves workflow template coverage without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for workflow template coverage.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show workflow template coverage is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-024 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-025: UX shell localization
- Read: microservices/workflow-studio/migration-playbooks/from-n8n.md
- Connects to: ontology projection pinning for product scope
- Build or inspect: a minimal artifact that proves UX shell localization without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for UX shell localization.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show UX shell localization is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-025 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-026: competitor parity metric
- Read: microservices/messenger/migration-playbooks/from-slack.md
- Connects to: workflow template coverage for first action
- Build or inspect: a minimal artifact that proves competitor parity metric without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for competitor parity metric.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show competitor parity metric is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-026 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-027: first-action activation
- Read: microservices/intelligence/migration-playbooks/from-github-merge-queue-and-bors.md
- Connects to: UX shell manifest and localization obligations
- Build or inspect: a minimal artifact that proves first-action activation without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for first-action activation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show first-action activation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-027 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-028: tenant cohort eligibility
- Read: docs/standards/prfaq-template.md
- Connects to: compliance overlay and claim boundary
- Build or inspect: a minimal artifact that proves tenant cohort eligibility without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant cohort eligibility.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant cohort eligibility is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-028 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-029: compliance claim boundary
- Read: docs/products/erp-coverage/PRD.md
- Connects to: pricing, packaging, and tenant eligibility
- Build or inspect: a minimal artifact that proves compliance claim boundary without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for compliance claim boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show compliance claim boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-029 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-030: pricing package guardrail
- Read: specs/masterplan.json
- Connects to: success metrics tied to observable behavior
- Build or inspect: a minimal artifact that proves pricing package guardrail without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for pricing package guardrail.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show pricing package guardrail is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-030 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-031: migration rollback branch
- Read: docs/architecture/persona-journey-microservice-cross-coverage-matrix-2026-05-21.md
- Connects to: capability tier as activation bundle
- Build or inspect: a minimal artifact that proves migration rollback branch without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for migration rollback branch.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show migration rollback branch is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-031 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-032: product fragment refusal
- Read: docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
- Connects to: persona x journey x microservice coverage model
- Build or inspect: a minimal artifact that proves product fragment refusal without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for product fragment refusal.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show product fragment refusal is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-032 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-033: journey success metric
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: migration playbook shape and rollback expectations
- Build or inspect: a minimal artifact that proves journey success metric without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for journey success metric.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show journey success metric is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-033 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-034: PRFAQ FAQ answer
- Read: registry/capability-tiers/index.json
- Connects to: Cedar permit set as product entitlement boundary
- Build or inspect: a minimal artifact that proves PRFAQ FAQ answer without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for PRFAQ FAQ answer.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show PRFAQ FAQ answer is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-034 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-035: customer escalation script
- Read: registry/capability-tiers/gold.json
- Connects to: ontology projection pinning for product scope
- Build or inspect: a minimal artifact that proves customer escalation script without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for customer escalation script.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show customer escalation script is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-035 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-036: roadmap dependency ordering
- Read: registry/capability-tiers/platinum.json
- Connects to: workflow template coverage for first action
- Build or inspect: a minimal artifact that proves roadmap dependency ordering without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for roadmap dependency ordering.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show roadmap dependency ordering is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-036 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-037: persona journey mapping
- Read: microservices/workflow-studio/capability-tiers/tier-matrix.md
- Connects to: UX shell manifest and localization obligations
- Build or inspect: a minimal artifact that proves persona journey mapping without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for persona journey mapping.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show persona journey mapping is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-037 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-038: capability tier brief
- Read: microservices/tenancy/README.md
- Connects to: compliance overlay and claim boundary
- Build or inspect: a minimal artifact that proves capability tier brief without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for capability tier brief.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show capability tier brief is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-038 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-039: migration acceptance criteria
- Read: docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md
- Connects to: pricing, packaging, and tenant eligibility
- Build or inspect: a minimal artifact that proves migration acceptance criteria without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for migration acceptance criteria.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show migration acceptance criteria is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-039 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-040: Cedar entitlement matrix
- Read: microservices/workflow-studio/migration-playbooks/from-n8n.md
- Connects to: success metrics tied to observable behavior
- Build or inspect: a minimal artifact that proves Cedar entitlement matrix without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar entitlement matrix.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar entitlement matrix is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-040 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-041: ontology projection pin
- Read: microservices/messenger/migration-playbooks/from-slack.md
- Connects to: capability tier as activation bundle
- Build or inspect: a minimal artifact that proves ontology projection pin without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ontology projection pin.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ontology projection pin is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-041 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-042: workflow template coverage
- Read: microservices/intelligence/migration-playbooks/from-github-merge-queue-and-bors.md
- Connects to: persona x journey x microservice coverage model
- Build or inspect: a minimal artifact that proves workflow template coverage without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for workflow template coverage.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show workflow template coverage is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-042 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-043: UX shell localization
- Read: docs/standards/prfaq-template.md
- Connects to: migration playbook shape and rollback expectations
- Build or inspect: a minimal artifact that proves UX shell localization without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for UX shell localization.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show UX shell localization is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-043 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-044: competitor parity metric
- Read: docs/products/erp-coverage/PRD.md
- Connects to: Cedar permit set as product entitlement boundary
- Build or inspect: a minimal artifact that proves competitor parity metric without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for competitor parity metric.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show competitor parity metric is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-044 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-045: first-action activation
- Read: specs/masterplan.json
- Connects to: ontology projection pinning for product scope
- Build or inspect: a minimal artifact that proves first-action activation without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for first-action activation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show first-action activation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-045 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-046: tenant cohort eligibility
- Read: docs/architecture/persona-journey-microservice-cross-coverage-matrix-2026-05-21.md
- Connects to: workflow template coverage for first action
- Build or inspect: a minimal artifact that proves tenant cohort eligibility without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant cohort eligibility.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant cohort eligibility is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-046 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-047: compliance claim boundary
- Read: docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
- Connects to: UX shell manifest and localization obligations
- Build or inspect: a minimal artifact that proves compliance claim boundary without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for compliance claim boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show compliance claim boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-047 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-048: pricing package guardrail
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: compliance overlay and claim boundary
- Build or inspect: a minimal artifact that proves pricing package guardrail without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for pricing package guardrail.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show pricing package guardrail is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-048 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-049: migration rollback branch
- Read: registry/capability-tiers/index.json
- Connects to: pricing, packaging, and tenant eligibility
- Build or inspect: a minimal artifact that proves migration rollback branch without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for migration rollback branch.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show migration rollback branch is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-049 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-050: product fragment refusal
- Read: registry/capability-tiers/gold.json
- Connects to: success metrics tied to observable behavior
- Build or inspect: a minimal artifact that proves product fragment refusal without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for product fragment refusal.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show product fragment refusal is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-050 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-051: journey success metric
- Read: registry/capability-tiers/platinum.json
- Connects to: capability tier as activation bundle
- Build or inspect: a minimal artifact that proves journey success metric without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for journey success metric.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show journey success metric is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-051 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-052: PRFAQ FAQ answer
- Read: microservices/workflow-studio/capability-tiers/tier-matrix.md
- Connects to: persona x journey x microservice coverage model
- Build or inspect: a minimal artifact that proves PRFAQ FAQ answer without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for PRFAQ FAQ answer.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show PRFAQ FAQ answer is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-052 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-053: customer escalation script
- Read: microservices/tenancy/README.md
- Connects to: migration playbook shape and rollback expectations
- Build or inspect: a minimal artifact that proves customer escalation script without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for customer escalation script.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show customer escalation script is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-053 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-054: roadmap dependency ordering
- Read: docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md
- Connects to: Cedar permit set as product entitlement boundary
- Build or inspect: a minimal artifact that proves roadmap dependency ordering without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for roadmap dependency ordering.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show roadmap dependency ordering is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-054 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-055: persona journey mapping
- Read: microservices/workflow-studio/migration-playbooks/from-n8n.md
- Connects to: ontology projection pinning for product scope
- Build or inspect: a minimal artifact that proves persona journey mapping without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for persona journey mapping.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show persona journey mapping is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-055 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-056: capability tier brief
- Read: microservices/messenger/migration-playbooks/from-slack.md
- Connects to: workflow template coverage for first action
- Build or inspect: a minimal artifact that proves capability tier brief without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for capability tier brief.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show capability tier brief is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-056 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-057: migration acceptance criteria
- Read: microservices/intelligence/migration-playbooks/from-github-merge-queue-and-bors.md
- Connects to: UX shell manifest and localization obligations
- Build or inspect: a minimal artifact that proves migration acceptance criteria without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for migration acceptance criteria.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show migration acceptance criteria is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-057 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-058: Cedar entitlement matrix
- Read: docs/standards/prfaq-template.md
- Connects to: compliance overlay and claim boundary
- Build or inspect: a minimal artifact that proves Cedar entitlement matrix without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar entitlement matrix.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar entitlement matrix is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-058 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-059: ontology projection pin
- Read: docs/products/erp-coverage/PRD.md
- Connects to: pricing, packaging, and tenant eligibility
- Build or inspect: a minimal artifact that proves ontology projection pin without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ontology projection pin.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ontology projection pin is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-059 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-060: workflow template coverage
- Read: specs/masterplan.json
- Connects to: success metrics tied to observable behavior
- Build or inspect: a minimal artifact that proves workflow template coverage without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for workflow template coverage.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show workflow template coverage is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-060 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-061: UX shell localization
- Read: docs/architecture/persona-journey-microservice-cross-coverage-matrix-2026-05-21.md
- Connects to: capability tier as activation bundle
- Build or inspect: a minimal artifact that proves UX shell localization without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for UX shell localization.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show UX shell localization is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-061 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-062: competitor parity metric
- Read: docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
- Connects to: persona x journey x microservice coverage model
- Build or inspect: a minimal artifact that proves competitor parity metric without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for competitor parity metric.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show competitor parity metric is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-062 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-063: first-action activation
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: migration playbook shape and rollback expectations
- Build or inspect: a minimal artifact that proves first-action activation without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for first-action activation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show first-action activation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-063 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-064: tenant cohort eligibility
- Read: registry/capability-tiers/index.json
- Connects to: Cedar permit set as product entitlement boundary
- Build or inspect: a minimal artifact that proves tenant cohort eligibility without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant cohort eligibility.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant cohort eligibility is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-064 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-065: compliance claim boundary
- Read: registry/capability-tiers/gold.json
- Connects to: ontology projection pinning for product scope
- Build or inspect: a minimal artifact that proves compliance claim boundary without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for compliance claim boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show compliance claim boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-065 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-066: pricing package guardrail
- Read: registry/capability-tiers/platinum.json
- Connects to: workflow template coverage for first action
- Build or inspect: a minimal artifact that proves pricing package guardrail without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for pricing package guardrail.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show pricing package guardrail is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-066 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-067: migration rollback branch
- Read: microservices/workflow-studio/capability-tiers/tier-matrix.md
- Connects to: UX shell manifest and localization obligations
- Build or inspect: a minimal artifact that proves migration rollback branch without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for migration rollback branch.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show migration rollback branch is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-067 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-068: product fragment refusal
- Read: microservices/tenancy/README.md
- Connects to: compliance overlay and claim boundary
- Build or inspect: a minimal artifact that proves product fragment refusal without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for product fragment refusal.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show product fragment refusal is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-068 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-069: journey success metric
- Read: docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md
- Connects to: pricing, packaging, and tenant eligibility
- Build or inspect: a minimal artifact that proves journey success metric without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for journey success metric.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show journey success metric is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-069 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-070: PRFAQ FAQ answer
- Read: microservices/workflow-studio/migration-playbooks/from-n8n.md
- Connects to: success metrics tied to observable behavior
- Build or inspect: a minimal artifact that proves PRFAQ FAQ answer without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for PRFAQ FAQ answer.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show PRFAQ FAQ answer is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to customer-success with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-070 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-071: customer escalation script
- Read: microservices/messenger/migration-playbooks/from-slack.md
- Connects to: capability tier as activation bundle
- Build or inspect: a minimal artifact that proves customer escalation script without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for customer escalation script.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show customer escalation script is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-product with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-071 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-072: roadmap dependency ordering
- Read: microservices/intelligence/migration-playbooks/from-github-merge-queue-and-bors.md
- Connects to: persona x journey x microservice coverage model
- Build or inspect: a minimal artifact that proves roadmap dependency ordering without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for roadmap dependency ordering.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show roadmap dependency ordering is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-072 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-073: persona journey mapping
- Read: docs/standards/prfaq-template.md
- Connects to: migration playbook shape and rollback expectations
- Build or inspect: a minimal artifact that proves persona journey mapping without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for persona journey mapping.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show persona journey mapping is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-073 contains file path, claim, evidence, rollback, and reviewer.

### Drill PM-074: capability tier brief
- Read: docs/products/erp-coverage/PRD.md
- Connects to: Cedar permit set as product entitlement boundary
- Build or inspect: a minimal artifact that proves capability tier brief without widening beyond product manager, capability and migration product lane.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for capability tier brief.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show capability tier brief is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row PM-074 contains file path, claim, evidence, rollback, and reviewer.

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
