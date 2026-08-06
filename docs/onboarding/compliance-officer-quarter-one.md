---
doc_class: OnboardingGuide
role: "compliance officer, data protection officer, or privacy counsel"
status: Published
date: 2026-05-20
owner: "ops-compliance + council-privacy + council-legal"
related_oyatie_adrs:
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
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

# Compliance Officer Quarter-One Onboarding

Audience: compliance officer, data protection officer, or privacy counsel.
Industry precedent: Microsoft Trust Center evidence discipline, Stripe compliance and financial-controls mapping, Google DSR process rigor, and AWS Artifact-style evidence packaging.

This guide is written for a programming-capable new joiner with no prior Oyatie architecture knowledge. Every phase names the repo files to read, the artifact to produce, the owner who reviews it, and the stop condition that proves the phase is complete.

Substance rule: do not treat this guide as orientation prose. Treat it as a work plan whose outputs can be inspected, replayed, or rejected.

You join the compliance, privacy, and counsel lane to turn tenant promises into evidence-bound obligations across data protection, certification readiness, regional packs, DSRs, AI Act duties, and audit-chain retention.
Quarter one is deliberately longer than engineering onboarding because the role must learn both the doctrine and the evidence machinery: what a product team claims, what a regulator can ask for, what evidence is accepted, and what must be refused until a gate exists.
Your work product is not advisory prose alone. Every compliance decision must map to a pack, an owner, a data class, a jurisdiction, an audit-chain event, and a renewal cadence.

## Hyperscaler-Grade Reading Contract

- Named precedent: Microsoft Trust Center evidence discipline, Stripe compliance and financial-controls mapping, Google DSR process rigor, and AWS Artifact-style evidence packaging.
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
1. docs/PRIVACY-PROGRAM.md
2. docs/security-program/security-program.json
3. docs/RISK-REGISTER.md
4. docs/VENDOR-PARTNER-LEDGER.md
5. docs/standards/privacy-review.md
6. specs/compliance-pack-schema.json
7. specs/capabilities/eu-ai-act-risk-class-registry.json
8. docs/decisions/ADR-0709-general-live-apex.md
9. docs/decisions/ADR-0709-general-live-apex.md
10. docs/decisions/ADR-0709-general-live-apex.md
11. docs/runbooks/regulator-evidence-pack-regen.md
12. docs/runbooks/dsr-cascade-with-evidence.md
13. docs/runbooks/breach-notification-council-escalation.md
14. microservices/compliance/compliance.md
15. microservices/compliance/dashboards/evidence-coverage.json


### Named ADRs to read

- ADR-0008 data use boundary
- ADR-0010 regional pack architecture
- ADR-0250 build ahead of certification
- ADR-0144 EU AI Act graduated risk tier
- ADR-0308 ML model lifecycle AI Act compliance
- ADR-0244 tenant scoping

### Named playgrounds

1. docs/runbooks/regulator-evidence-pack-regen.md
   - Artifact: write a four-sentence note explaining what this playground proves for compliance officer, data protection officer, or privacy counsel.
2. docs/runbooks/dsr-cascade-with-evidence.md
   - Artifact: write a four-sentence note explaining what this playground proves for compliance officer, data protection officer, or privacy counsel.
3. docs/user-journeys/j76-eu-gdpr-dsar-full-cascade/README.md
   - Artifact: write a four-sentence note explaining what this playground proves for compliance officer, data protection officer, or privacy counsel.
4. docs/user-journeys/j77-eu-ai-act-high-risk-credit-decision/README.md
   - Artifact: write a four-sentence note explaining what this playground proves for compliance officer, data protection officer, or privacy counsel.

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

1. COMP-STARTER-001 add a missing evidence cadence row to a compliance evidence pack
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for COMP.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
2. COMP-STARTER-002 map one user journey to its lawful-basis and data-class obligations
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for COMP.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
3. COMP-STARTER-003 improve a DSR runbook branch with regulator-facing evidence wording
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for COMP.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
4. COMP-STARTER-004 add a risk-register cross-reference for an AI Act capability tier
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for COMP.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.

### Mentor pairing protocol

Pair with a privacy counsel mentor for data class and lawful basis, a compliance engineer for evidence pack mechanics, and an audit-chain owner for evidence replay. Every pairing ends with a mapped control, evidence owner, and review cadence.
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

First independent project: Own `COMP-PROJ-001`: build a quarter-one evidence map for one tenant onboarding journey that spans privacy notice, DSR path, data residency, AI Act tier if applicable, and audit-chain retention.

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

- DSR cascade evidence map
- regional pack obligations inventory
- AI Act graduated-tier review
- Trust portal evidence refresh checklist

### Key contacts in other teams

- council-privacy
- council-legal
- ops-compliance
- axis-tenancy
- axis-audit-chain
- axis-regional-pack
- ops-security

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
1. data-use boundary and purpose limitation
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for data-use boundary and purpose limitation.
2. regional pack composition and regulator anchor mapping
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for regional pack composition and regulator anchor mapping.
3. DSR intake, cascade, proof of erasure, and appeal
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for DSR intake, cascade, proof of erasure, and appeal.
4. breach notification severity and timing
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for breach notification severity and timing.
5. EU AI Act risk tier mutation under deployment context
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for EU AI Act risk tier mutation under deployment context.
6. ML model lifecycle evidence and post-market monitoring
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for ML model lifecycle evidence and post-market monitoring.
7. build-ahead-of-certification claim boundary
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for build-ahead-of-certification claim boundary.
8. vendor and partner risk evidence cadence
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for vendor and partner risk evidence cadence.
9. tenant onboarding privacy packet
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for tenant onboarding privacy packet.
10. audit-chain retention, legal hold, and regulator export
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for audit-chain retention, legal hold, and regulator export.
Domain expertise stop condition: you can answer what breaks if this topic is implemented incorrectly, who owns the rollback, and what evidence proves the system recovered.
- Verification for Month 2: deep-dive notes cover every listed topic and at least one note is reviewed by a cross-team owner
- Stop condition for Month 2: mentor and owner can point to the artifact without asking you to explain hidden context.

## Quarter 1: Ownership


### Named OKRs

- OKR-COMP-Q1-1: publish one evidence map accepted by privacy, security, and audit-chain owners
- OKR-COMP-Q1-2: close one stale compliance reference by binding it to a real pack, control, and owner
- OKR-COMP-Q1-3: run one DSR or regulator evidence tabletop with replayable audit evidence

### Named on-call rotation entry

Enter `regulatory-response-shadow` rotation in month two; quarter-one target is one DSR cascade tabletop and one breach-notification evidence drill.

### Named team-OKR contribution

Contribute to `TEAM-OKR-COMPLIANCE-2026Q2`: every regulated capability has pack mapping, evidence owner, renewal cadence, and claim boundary.

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

1. Writing compliance obligations without naming evidence owner and refresh cadence.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for compliance officer, data protection officer, or privacy counsel.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
2. Treating GDPR, HIPAA, PCI, KR PIPA, DPDPA, and AI Act as generic checklists.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for compliance officer, data protection officer, or privacy counsel.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
3. Accepting a product claim that lacks a pack, control, or audit event.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for compliance officer, data protection officer, or privacy counsel.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
4. Using screenshots as evidence when machine-readable audit-chain evidence exists.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for compliance officer, data protection officer, or privacy counsel.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
5. Approving a DSR flow without erasure proof and exception handling.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for compliance officer, data protection officer, or privacy counsel.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
6. Ignoring tenant-specific overrides and data residency constraints.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for compliance officer, data protection officer, or privacy counsel.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
7. Letting legal wording drift from the actual Cedar and audit behavior.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for compliance officer, data protection officer, or privacy counsel.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
8. Treating high-risk AI as blocked forever instead of blocked until conformity path exists.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for compliance officer, data protection officer, or privacy counsel.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.

## Cross-Team Collaboration Playbook

| Team | Handoff ritual | Minimum payload |
| --- |--- |--- |
| axis-tenancy | Tenant obligation handoff | Attach tenant type, region, residency, sub-scope, and DSR SLA. |
| axis-audit-chain | Evidence handoff | Attach audit event, retention class, export format, and replay path. |
| axis-regional-pack | Regional pack handoff | Attach regulation anchors, pack version, overlay behavior, and sunset cadence. |
| ops-security | Control evidence handoff | Attach control id, test evidence, exception state, and owner. |
| council-product | Claim boundary handoff | Attach allowed marketing claim, forbidden claim, and evidence link. |
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
| DPIA | Data protection impact assessment for regulated processing. |
| DSR | Data subject request path with intake, cascade, proof, and exception handling. |
| compliance pack | Versioned regulatory overlay with controls, evidence, and enforcement semantics. |
| claim boundary | The exact public or internal claim that evidence supports. |
| lawful basis | Privacy justification for processing personal data in a specific context. |

## Escalation Channels

| Escalation | Use when | Owner |
| --- |--- |--- |
| mentor checkpoint | you can proceed locally but need review of reasoning or evidence | assigned mentor |
| axis owner | a file or policy belongs to another team | council-privacy |
| council review | claim boundary, doctrine, compliance, or security interpretation changes | ops-compliance + council-privacy + council-legal |
| SRE on-call | dev-cell, incident, or reliability path blocks verification | ops-sre-reliability |
| security review | credential, tenant isolation, policy, or regulated data risk appears | ops-security |

## Resources & References

- docs/PRIVACY-PROGRAM.md
- docs/security-program/security-program.json
- docs/RISK-REGISTER.md
- docs/standards/privacy-review.md
- docs/runbooks/dsr-cascade-with-evidence.md
- docs/runbooks/regulator-evidence-pack-regen.md
- specs/compliance-pack-schema.json

Reference-reading protocol: open the resource, identify the authority section, write the one-sentence claim it supports, and record whether the resource is doctrine, spec, implementation, test, runbook, dashboard, or evidence.

## Role-Specific Drill Library

Use this ledger when you need extra practice or when a mentor asks for stronger evidence. Each drill is intentionally small but must end with a verifiable artifact.

### Drill COMP-001: DSR erasure proof
- Read: docs/PRIVACY-PROGRAM.md
- Connects to: data-use boundary and purpose limitation
- Build or inspect: a minimal artifact that proves DSR erasure proof without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for DSR erasure proof.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show DSR erasure proof is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-001 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-002: lawful basis mapping
- Read: docs/security-program/security-program.json
- Connects to: regional pack composition and regulator anchor mapping
- Build or inspect: a minimal artifact that proves lawful basis mapping without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for lawful basis mapping.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show lawful basis mapping is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-legal with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-002 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-003: regional pack overlay
- Read: docs/RISK-REGISTER.md
- Connects to: DSR intake, cascade, proof of erasure, and appeal
- Build or inspect: a minimal artifact that proves regional pack overlay without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regional pack overlay.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regional pack overlay is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-003 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-004: AI Act high-risk review
- Read: docs/VENDOR-PARTNER-LEDGER.md
- Connects to: breach notification severity and timing
- Build or inspect: a minimal artifact that proves AI Act high-risk review without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for AI Act high-risk review.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show AI Act high-risk review is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-004 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-005: breach notification clock
- Read: docs/standards/privacy-review.md
- Connects to: EU AI Act risk tier mutation under deployment context
- Build or inspect: a minimal artifact that proves breach notification clock without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for breach notification clock.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show breach notification clock is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-005 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-006: vendor evidence refresh
- Read: specs/compliance-pack-schema.json
- Connects to: ML model lifecycle evidence and post-market monitoring
- Build or inspect: a minimal artifact that proves vendor evidence refresh without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for vendor evidence refresh.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show vendor evidence refresh is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-regional-pack with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-006 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-007: Trust portal claim boundary
- Read: specs/capabilities/eu-ai-act-risk-class-registry.json
- Connects to: build-ahead-of-certification claim boundary
- Build or inspect: a minimal artifact that proves Trust portal claim boundary without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Trust portal claim boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Trust portal claim boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-security with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-007 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-008: data residency exception
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: vendor and partner risk evidence cadence
- Build or inspect: a minimal artifact that proves data residency exception without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for data residency exception.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show data residency exception is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-008 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-009: minor-user privacy flow
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: tenant onboarding privacy packet
- Build or inspect: a minimal artifact that proves minor-user privacy flow without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for minor-user privacy flow.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show minor-user privacy flow is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-legal with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-009 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-010: DPIA residual risk
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: audit-chain retention, legal hold, and regulator export
- Build or inspect: a minimal artifact that proves DPIA residual risk without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for DPIA residual risk.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show DPIA residual risk is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-010 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-011: audit export request
- Read: docs/runbooks/regulator-evidence-pack-regen.md
- Connects to: data-use boundary and purpose limitation
- Build or inspect: a minimal artifact that proves audit export request without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit export request.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit export request is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-011 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-012: retention legal hold
- Read: docs/runbooks/dsr-cascade-with-evidence.md
- Connects to: regional pack composition and regulator anchor mapping
- Build or inspect: a minimal artifact that proves retention legal hold without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for retention legal hold.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show retention legal hold is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-012 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-013: cross-border transfer review
- Read: docs/runbooks/breach-notification-council-escalation.md
- Connects to: DSR intake, cascade, proof of erasure, and appeal
- Build or inspect: a minimal artifact that proves cross-border transfer review without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cross-border transfer review.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cross-border transfer review is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-regional-pack with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-013 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-014: evidence cadence miss
- Read: microservices/compliance/compliance.md
- Connects to: breach notification severity and timing
- Build or inspect: a minimal artifact that proves evidence cadence miss without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for evidence cadence miss.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show evidence cadence miss is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-security with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-014 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-015: regulator packet regeneration
- Read: microservices/compliance/dashboards/evidence-coverage.json
- Connects to: EU AI Act risk tier mutation under deployment context
- Build or inspect: a minimal artifact that proves regulator packet regeneration without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regulator packet regeneration.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regulator packet regeneration is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-015 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-016: privacy notice drift
- Read: docs/PRIVACY-PROGRAM.md
- Connects to: ML model lifecycle evidence and post-market monitoring
- Build or inspect: a minimal artifact that proves privacy notice drift without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for privacy notice drift.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show privacy notice drift is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-legal with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-016 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-017: tenant onboarding packet
- Read: docs/security-program/security-program.json
- Connects to: build-ahead-of-certification claim boundary
- Build or inspect: a minimal artifact that proves tenant onboarding packet without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant onboarding packet.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant onboarding packet is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-017 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-018: compliance dashboard evidence gap
- Read: docs/RISK-REGISTER.md
- Connects to: vendor and partner risk evidence cadence
- Build or inspect: a minimal artifact that proves compliance dashboard evidence gap without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for compliance dashboard evidence gap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show compliance dashboard evidence gap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-018 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-019: DSR erasure proof
- Read: docs/VENDOR-PARTNER-LEDGER.md
- Connects to: tenant onboarding privacy packet
- Build or inspect: a minimal artifact that proves DSR erasure proof without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for DSR erasure proof.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show DSR erasure proof is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-019 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-020: lawful basis mapping
- Read: docs/standards/privacy-review.md
- Connects to: audit-chain retention, legal hold, and regulator export
- Build or inspect: a minimal artifact that proves lawful basis mapping without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for lawful basis mapping.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show lawful basis mapping is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-regional-pack with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-020 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-021: regional pack overlay
- Read: specs/compliance-pack-schema.json
- Connects to: data-use boundary and purpose limitation
- Build or inspect: a minimal artifact that proves regional pack overlay without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regional pack overlay.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regional pack overlay is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-security with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-021 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-022: AI Act high-risk review
- Read: specs/capabilities/eu-ai-act-risk-class-registry.json
- Connects to: regional pack composition and regulator anchor mapping
- Build or inspect: a minimal artifact that proves AI Act high-risk review without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for AI Act high-risk review.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show AI Act high-risk review is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-022 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-023: breach notification clock
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: DSR intake, cascade, proof of erasure, and appeal
- Build or inspect: a minimal artifact that proves breach notification clock without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for breach notification clock.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show breach notification clock is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-legal with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-023 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-024: vendor evidence refresh
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: breach notification severity and timing
- Build or inspect: a minimal artifact that proves vendor evidence refresh without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for vendor evidence refresh.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show vendor evidence refresh is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-024 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-025: Trust portal claim boundary
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: EU AI Act risk tier mutation under deployment context
- Build or inspect: a minimal artifact that proves Trust portal claim boundary without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Trust portal claim boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Trust portal claim boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-025 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-026: data residency exception
- Read: docs/runbooks/regulator-evidence-pack-regen.md
- Connects to: ML model lifecycle evidence and post-market monitoring
- Build or inspect: a minimal artifact that proves data residency exception without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for data residency exception.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show data residency exception is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-026 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-027: minor-user privacy flow
- Read: docs/runbooks/dsr-cascade-with-evidence.md
- Connects to: build-ahead-of-certification claim boundary
- Build or inspect: a minimal artifact that proves minor-user privacy flow without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for minor-user privacy flow.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show minor-user privacy flow is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-regional-pack with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-027 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-028: DPIA residual risk
- Read: docs/runbooks/breach-notification-council-escalation.md
- Connects to: vendor and partner risk evidence cadence
- Build or inspect: a minimal artifact that proves DPIA residual risk without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for DPIA residual risk.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show DPIA residual risk is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-security with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-028 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-029: audit export request
- Read: microservices/compliance/compliance.md
- Connects to: tenant onboarding privacy packet
- Build or inspect: a minimal artifact that proves audit export request without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit export request.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit export request is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-029 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-030: retention legal hold
- Read: microservices/compliance/dashboards/evidence-coverage.json
- Connects to: audit-chain retention, legal hold, and regulator export
- Build or inspect: a minimal artifact that proves retention legal hold without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for retention legal hold.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show retention legal hold is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-legal with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-030 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-031: cross-border transfer review
- Read: docs/PRIVACY-PROGRAM.md
- Connects to: data-use boundary and purpose limitation
- Build or inspect: a minimal artifact that proves cross-border transfer review without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cross-border transfer review.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cross-border transfer review is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-031 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-032: evidence cadence miss
- Read: docs/security-program/security-program.json
- Connects to: regional pack composition and regulator anchor mapping
- Build or inspect: a minimal artifact that proves evidence cadence miss without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for evidence cadence miss.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show evidence cadence miss is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-032 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-033: regulator packet regeneration
- Read: docs/RISK-REGISTER.md
- Connects to: DSR intake, cascade, proof of erasure, and appeal
- Build or inspect: a minimal artifact that proves regulator packet regeneration without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regulator packet regeneration.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regulator packet regeneration is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-033 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-034: privacy notice drift
- Read: docs/VENDOR-PARTNER-LEDGER.md
- Connects to: breach notification severity and timing
- Build or inspect: a minimal artifact that proves privacy notice drift without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for privacy notice drift.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show privacy notice drift is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-regional-pack with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-034 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-035: tenant onboarding packet
- Read: docs/standards/privacy-review.md
- Connects to: EU AI Act risk tier mutation under deployment context
- Build or inspect: a minimal artifact that proves tenant onboarding packet without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant onboarding packet.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant onboarding packet is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-security with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-035 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-036: compliance dashboard evidence gap
- Read: specs/compliance-pack-schema.json
- Connects to: ML model lifecycle evidence and post-market monitoring
- Build or inspect: a minimal artifact that proves compliance dashboard evidence gap without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for compliance dashboard evidence gap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show compliance dashboard evidence gap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-036 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-037: DSR erasure proof
- Read: specs/capabilities/eu-ai-act-risk-class-registry.json
- Connects to: build-ahead-of-certification claim boundary
- Build or inspect: a minimal artifact that proves DSR erasure proof without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for DSR erasure proof.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show DSR erasure proof is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-legal with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-037 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-038: lawful basis mapping
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: vendor and partner risk evidence cadence
- Build or inspect: a minimal artifact that proves lawful basis mapping without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for lawful basis mapping.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show lawful basis mapping is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-038 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-039: regional pack overlay
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: tenant onboarding privacy packet
- Build or inspect: a minimal artifact that proves regional pack overlay without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regional pack overlay.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regional pack overlay is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-039 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-040: AI Act high-risk review
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: audit-chain retention, legal hold, and regulator export
- Build or inspect: a minimal artifact that proves AI Act high-risk review without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for AI Act high-risk review.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show AI Act high-risk review is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-040 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-041: breach notification clock
- Read: docs/runbooks/regulator-evidence-pack-regen.md
- Connects to: data-use boundary and purpose limitation
- Build or inspect: a minimal artifact that proves breach notification clock without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for breach notification clock.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show breach notification clock is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-regional-pack with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-041 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-042: vendor evidence refresh
- Read: docs/runbooks/dsr-cascade-with-evidence.md
- Connects to: regional pack composition and regulator anchor mapping
- Build or inspect: a minimal artifact that proves vendor evidence refresh without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for vendor evidence refresh.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show vendor evidence refresh is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-security with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-042 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-043: Trust portal claim boundary
- Read: docs/runbooks/breach-notification-council-escalation.md
- Connects to: DSR intake, cascade, proof of erasure, and appeal
- Build or inspect: a minimal artifact that proves Trust portal claim boundary without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Trust portal claim boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Trust portal claim boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-043 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-044: data residency exception
- Read: microservices/compliance/compliance.md
- Connects to: breach notification severity and timing
- Build or inspect: a minimal artifact that proves data residency exception without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for data residency exception.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show data residency exception is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-legal with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-044 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-045: minor-user privacy flow
- Read: microservices/compliance/dashboards/evidence-coverage.json
- Connects to: EU AI Act risk tier mutation under deployment context
- Build or inspect: a minimal artifact that proves minor-user privacy flow without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for minor-user privacy flow.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show minor-user privacy flow is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-045 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-046: DPIA residual risk
- Read: docs/PRIVACY-PROGRAM.md
- Connects to: ML model lifecycle evidence and post-market monitoring
- Build or inspect: a minimal artifact that proves DPIA residual risk without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for DPIA residual risk.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show DPIA residual risk is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-046 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-047: audit export request
- Read: docs/security-program/security-program.json
- Connects to: build-ahead-of-certification claim boundary
- Build or inspect: a minimal artifact that proves audit export request without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit export request.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit export request is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-047 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-048: retention legal hold
- Read: docs/RISK-REGISTER.md
- Connects to: vendor and partner risk evidence cadence
- Build or inspect: a minimal artifact that proves retention legal hold without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for retention legal hold.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show retention legal hold is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-regional-pack with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-048 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-049: cross-border transfer review
- Read: docs/VENDOR-PARTNER-LEDGER.md
- Connects to: tenant onboarding privacy packet
- Build or inspect: a minimal artifact that proves cross-border transfer review without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cross-border transfer review.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cross-border transfer review is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-security with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-049 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-050: evidence cadence miss
- Read: docs/standards/privacy-review.md
- Connects to: audit-chain retention, legal hold, and regulator export
- Build or inspect: a minimal artifact that proves evidence cadence miss without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for evidence cadence miss.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show evidence cadence miss is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-050 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-051: regulator packet regeneration
- Read: specs/compliance-pack-schema.json
- Connects to: data-use boundary and purpose limitation
- Build or inspect: a minimal artifact that proves regulator packet regeneration without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regulator packet regeneration.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regulator packet regeneration is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-legal with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-051 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-052: privacy notice drift
- Read: specs/capabilities/eu-ai-act-risk-class-registry.json
- Connects to: regional pack composition and regulator anchor mapping
- Build or inspect: a minimal artifact that proves privacy notice drift without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for privacy notice drift.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show privacy notice drift is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-052 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-053: tenant onboarding packet
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: DSR intake, cascade, proof of erasure, and appeal
- Build or inspect: a minimal artifact that proves tenant onboarding packet without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant onboarding packet.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant onboarding packet is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-053 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-054: compliance dashboard evidence gap
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: breach notification severity and timing
- Build or inspect: a minimal artifact that proves compliance dashboard evidence gap without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for compliance dashboard evidence gap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show compliance dashboard evidence gap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-054 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-055: DSR erasure proof
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: EU AI Act risk tier mutation under deployment context
- Build or inspect: a minimal artifact that proves DSR erasure proof without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for DSR erasure proof.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show DSR erasure proof is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-regional-pack with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-055 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-056: lawful basis mapping
- Read: docs/runbooks/regulator-evidence-pack-regen.md
- Connects to: ML model lifecycle evidence and post-market monitoring
- Build or inspect: a minimal artifact that proves lawful basis mapping without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for lawful basis mapping.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show lawful basis mapping is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-security with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-056 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-057: regional pack overlay
- Read: docs/runbooks/dsr-cascade-with-evidence.md
- Connects to: build-ahead-of-certification claim boundary
- Build or inspect: a minimal artifact that proves regional pack overlay without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regional pack overlay.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regional pack overlay is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-057 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-058: AI Act high-risk review
- Read: docs/runbooks/breach-notification-council-escalation.md
- Connects to: vendor and partner risk evidence cadence
- Build or inspect: a minimal artifact that proves AI Act high-risk review without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for AI Act high-risk review.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show AI Act high-risk review is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-legal with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-058 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-059: breach notification clock
- Read: microservices/compliance/compliance.md
- Connects to: tenant onboarding privacy packet
- Build or inspect: a minimal artifact that proves breach notification clock without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for breach notification clock.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show breach notification clock is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-059 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-060: vendor evidence refresh
- Read: microservices/compliance/dashboards/evidence-coverage.json
- Connects to: audit-chain retention, legal hold, and regulator export
- Build or inspect: a minimal artifact that proves vendor evidence refresh without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for vendor evidence refresh.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show vendor evidence refresh is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-060 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-061: Trust portal claim boundary
- Read: docs/PRIVACY-PROGRAM.md
- Connects to: data-use boundary and purpose limitation
- Build or inspect: a minimal artifact that proves Trust portal claim boundary without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Trust portal claim boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Trust portal claim boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-061 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-062: data residency exception
- Read: docs/security-program/security-program.json
- Connects to: regional pack composition and regulator anchor mapping
- Build or inspect: a minimal artifact that proves data residency exception without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for data residency exception.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show data residency exception is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-regional-pack with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-062 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-063: minor-user privacy flow
- Read: docs/RISK-REGISTER.md
- Connects to: DSR intake, cascade, proof of erasure, and appeal
- Build or inspect: a minimal artifact that proves minor-user privacy flow without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for minor-user privacy flow.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show minor-user privacy flow is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-security with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-063 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-064: DPIA residual risk
- Read: docs/VENDOR-PARTNER-LEDGER.md
- Connects to: breach notification severity and timing
- Build or inspect: a minimal artifact that proves DPIA residual risk without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for DPIA residual risk.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show DPIA residual risk is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-064 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-065: audit export request
- Read: docs/standards/privacy-review.md
- Connects to: EU AI Act risk tier mutation under deployment context
- Build or inspect: a minimal artifact that proves audit export request without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit export request.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit export request is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-legal with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-065 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-066: retention legal hold
- Read: specs/compliance-pack-schema.json
- Connects to: ML model lifecycle evidence and post-market monitoring
- Build or inspect: a minimal artifact that proves retention legal hold without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for retention legal hold.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show retention legal hold is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-066 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-067: cross-border transfer review
- Read: specs/capabilities/eu-ai-act-risk-class-registry.json
- Connects to: build-ahead-of-certification claim boundary
- Build or inspect: a minimal artifact that proves cross-border transfer review without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cross-border transfer review.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cross-border transfer review is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-067 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-068: evidence cadence miss
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: vendor and partner risk evidence cadence
- Build or inspect: a minimal artifact that proves evidence cadence miss without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for evidence cadence miss.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show evidence cadence miss is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-068 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-069: regulator packet regeneration
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: tenant onboarding privacy packet
- Build or inspect: a minimal artifact that proves regulator packet regeneration without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for regulator packet regeneration.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show regulator packet regeneration is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-regional-pack with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-069 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-070: privacy notice drift
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: audit-chain retention, legal hold, and regulator export
- Build or inspect: a minimal artifact that proves privacy notice drift without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for privacy notice drift.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show privacy notice drift is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-security with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-070 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-071: tenant onboarding packet
- Read: docs/runbooks/regulator-evidence-pack-regen.md
- Connects to: data-use boundary and purpose limitation
- Build or inspect: a minimal artifact that proves tenant onboarding packet without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant onboarding packet.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant onboarding packet is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-071 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-072: compliance dashboard evidence gap
- Read: docs/runbooks/dsr-cascade-with-evidence.md
- Connects to: regional pack composition and regulator anchor mapping
- Build or inspect: a minimal artifact that proves compliance dashboard evidence gap without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for compliance dashboard evidence gap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show compliance dashboard evidence gap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-legal with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-072 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-073: DSR erasure proof
- Read: docs/runbooks/breach-notification-council-escalation.md
- Connects to: DSR intake, cascade, proof of erasure, and appeal
- Build or inspect: a minimal artifact that proves DSR erasure proof without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for DSR erasure proof.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show DSR erasure proof is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-073 contains file path, claim, evidence, rollback, and reviewer.

### Drill COMP-074: lawful basis mapping
- Read: microservices/compliance/compliance.md
- Connects to: breach notification severity and timing
- Build or inspect: a minimal artifact that proves lawful basis mapping without widening beyond compliance officer, data protection officer, or privacy counsel.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for lawful basis mapping.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show lawful basis mapping is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row COMP-074 contains file path, claim, evidence, rollback, and reviewer.

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
