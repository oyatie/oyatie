---
doc_class: OnboardingGuide
role: "SPA and web engineer, Workflow Studio and web surfaces"
status: Published
date: 2026-05-20
owner: "axis-frontend + axis-workflow-studio + council-design-system"
related_oyatie_adrs:
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0704-k8s-port-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - microservices/workflow-studio/decisions/ADR-WFS-001-yjs-crdt-for-collaborative-canvas-editing.md
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/DOC-CATALOG.md
  - docs/STANDARDS-AND-TEMPLATES.md
inbound_citations:
  - docs/onboarding/intern-day-one.md
  - docs/onboarding/intern-week-one.md
enforced_by:
  - governance-doc-rigor
  - governance-doc-graph-6hops
---

# Frontend Engineer Week-One Onboarding

Audience: SPA and web engineer, Workflow Studio and web surfaces.
Industry precedent: Figma FigJam collaborative canvas, Google Docs real-time editing, Linear keyboard-first workflows, and Cloudflare dashboard operational clarity.

This guide is written for a programming-capable new joiner with no prior Oyatie architecture knowledge. Every phase names the repo files to read, the artifact to produce, the owner who reviews it, and the stop condition that proves the phase is complete.

Substance rule: do not treat this guide as orientation prose. Treat it as a work plan whose outputs can be inspected, replayed, or rejected.

You join the web frontend lane where Workflow Studio is the first surface that forces canvas performance, CRDT collaboration, accessibility, localization, and policy-aware UX to work at the same time.
Week one is scoped to read the client-stack ADRs, run a local Workflow Studio playground, fix one small UI or test issue, and prove that you can ship a web change without weakening WCAG 2.2 AA, locale overlays, or canvas collaboration semantics.
The repo contains both Loro-first portability doctrine and a workflow-studio Yjs ADR. Treat Yjs as a concrete collaborative-canvas concern and ADR-0142 as the portability guardrail that prevents hard coupling to any single CRDT.

## Hyperscaler-Grade Reading Contract

- Named precedent: Figma FigJam collaborative canvas, Google Docs real-time editing, Linear keyboard-first workflows, and Cloudflare dashboard operational clarity.
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
1. docs/decisions/ADR-0700-ci-admission-live-apex.md
2. docs/decisions/ADR-0700-ci-admission-live-apex.md
3. docs/decisions/ADR-0704-k8s-port-live-apex.md
4. docs/standards/wcag-2-2-aa-checklist.md
5. docs/standards/locale-routing.md
6. docs/standards/i18n-canonical.md
7. microservices/workflow-studio/ARCHITECTURE.md
8. microservices/workflow-studio/PRD.md
9. microservices/workflow-studio/contracts/openapi/workflow-studio.yaml
10. microservices/workflow-studio/clients/web-sveltekit/src/templates/CatalogBrowser.svelte
11. microservices/workflow-studio/clients/web-sveltekit/src/templates/TemplateDetail.svelte
12. microservices/workflow-studio/clients/web-sveltekit/packages/i18n-source/source.ftl
13. microservices/workflow-studio/clients/web-sveltekit/packages/i18n-source/ko-KR.ftl
14. microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml
15. microservices/workflow-studio/runbooks/collaborative-canvas-merge-conflict.md


### Named ADRs to read

- ADR-0185 Workflow Studio client stack
- ADR-0204 canvas library and performance commitments
- ADR-0142 CRDT portability trait
- ADR-0292 minor-user doctrine and accessibility-sensitive UX
- ADR-WFS-001 Yjs CRDT for collaborative canvas editing

### Named playgrounds

1. microservices/workflow-studio/templates/fixtures/workflow-studio-template-new-hire-onboarding.fixture.json
   - Artifact: write a four-sentence note explaining what this playground proves for SPA and web engineer, Workflow Studio and web surfaces.
2. microservices/workflow-studio/templates/explainers/workflow-studio-template-new-hire-onboarding.md
   - Artifact: write a four-sentence note explaining what this playground proves for SPA and web engineer, Workflow Studio and web surfaces.
3. microservices/workflow-studio/clients/web-sveltekit/src/templates/catalog.spec.ts
   - Artifact: write a four-sentence note explaining what this playground proves for SPA and web engineer, Workflow Studio and web surfaces.
4. microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md
   - Artifact: write a four-sentence note explaining what this playground proves for SPA and web engineer, Workflow Studio and web surfaces.

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

1. FE-STARTER-001 add a missing keyboard assertion to a template catalog test
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for FE.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
2. FE-STARTER-002 fix one Fluent locale overlay fallback in `ko-KR.ftl` or `ar-SA.ftl`
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for FE.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
3. FE-STARTER-003 add a canvas benchmark fixture note for 1000-node p99 frame-time review
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for FE.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
4. FE-STARTER-004 improve the collaborative-canvas merge-conflict runbook with a UI verification step
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for FE.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.

### Mentor pairing protocol

Pair with a Workflow Studio frontend mentor for the first UI change, an accessibility reviewer for the WCAG assertion, and an i18n reviewer for locale overlays. Every pairing ends with screenshot, axe result, keyboard path, and locale diff.
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

First independent project: Own `FE-PROJ-001`: make the template catalog path prove keyboard navigation, locale fallback, and CRDT-safe optimistic update behavior in one small testable slice.

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

- Workflow Studio template catalog accessibility pass
- Yjs and Loro portability vocabulary reconciliation
- locale overlay smoke fixtures for ko-KR and ar-SA
- canvas 1000-node performance evidence review

### Key contacts in other teams

- axis-workflow-studio
- council-design-system
- axis-i18n
- axis-policy-engine
- ops-sre-reliability

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
1. SvelteKit web phase and Leptos phase-two trigger
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for SvelteKit web phase and Leptos phase-two trigger.
2. svelte-flow adapter boundary and canvas future
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for svelte-flow adapter boundary and canvas future.
3. Yjs collaborative editing semantics and CRDT conflict resolution
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for Yjs collaborative editing semantics and CRDT conflict resolution.
4. ADR-0142 portability trait and alternate adapter compile gate
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for ADR-0142 portability trait and alternate adapter compile gate.
5. WCAG 2.2 AA keyboard, focus, target-size, and error-state coverage
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for WCAG 2.2 AA keyboard, focus, target-size, and error-state coverage.
6. Fluent locale overlays and tenant regional packs
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for Fluent locale overlays and tenant regional packs.
7. canvas p99 frame-time budget at 1000 nodes
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for canvas p99 frame-time budget at 1000 nodes.
8. presence awareness and selection halo semantics
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for presence awareness and selection halo semantics.
9. Cedar-driven UX affordance hiding versus disabled controls
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for Cedar-driven UX affordance hiding versus disabled controls.
10. minor-user UX restrictions under ADR-0292
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for minor-user UX restrictions under ADR-0292.
Domain expertise stop condition: you can answer what breaks if this topic is implemented incorrectly, who owns the rollback, and what evidence proves the system recovered.
- Verification for Month 2: deep-dive notes cover every listed topic and at least one note is reviewed by a cross-team owner
- Stop condition for Month 2: mentor and owner can point to the artifact without asking you to explain hidden context.

## Quarter 1: Ownership


### Named OKRs

- OKR-FE-Q1-1: ship one Workflow Studio UI PR with keyboard, screen-reader, locale, and visual regression evidence
- OKR-FE-Q1-2: reduce one canvas collaboration failure mode to a tested runbook branch
- OKR-FE-Q1-3: publish one component handoff note that names API contract, accessibility state, and locale overlay

### Named on-call rotation entry

Enter `workflow-studio-web-shadow` rotation after two merged UI PRs; quarter-one target is one canvas performance incident shadow and one locale-pack rollback drill.

### Named team-OKR contribution

Contribute to `TEAM-OKR-WFS-2026Q2`: Workflow Studio web paths meet WCAG 2.2 AA, locale overlay, and collaboration integrity floors before beta.

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

1. Treating canvas drag-and-drop as sufficient without keyboard alternative.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for SPA and web engineer, Workflow Studio and web surfaces.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
2. Hard-coding English strings instead of using Fluent source files.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for SPA and web engineer, Workflow Studio and web surfaces.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
3. Coupling UI state directly to Yjs documents without the portability boundary.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for SPA and web engineer, Workflow Studio and web surfaces.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
4. Using disabled controls to hide Cedar denies without explaining policy state.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for SPA and web engineer, Workflow Studio and web surfaces.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
5. Optimizing for 20-node demos while ignoring 1000-node frame-time budgets.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for SPA and web engineer, Workflow Studio and web surfaces.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
6. Shipping a visual fix without mobile-portrait, tablet, desktop, and wide-desktop checks.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for SPA and web engineer, Workflow Studio and web surfaces.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
7. Assuming WCAG 2.1 AA is enough when the repo standard requires WCAG 2.2 AA.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for SPA and web engineer, Workflow Studio and web surfaces.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
8. Adding a new component without a locale, focus, empty-state, error-state, and loading-state pass.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for SPA and web engineer, Workflow Studio and web surfaces.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.

## Cross-Team Collaboration Playbook

| Team | Handoff ritual | Minimum payload |
| --- |--- |--- |
| axis-workflow-studio | Canvas handoff | Attach node count, frame-time evidence, CRDT room state, and replay fixture. |
| council-design-system | Component handoff | Attach token usage, responsive screenshots, focus order, and state matrix. |
| axis-i18n | Locale handoff | Attach source.ftl key, overlay diff, fallback path, and RTL note. |
| axis-policy-engine | Policy UX handoff | Attach Cedar deny reason, user-facing state, and audit event id. |
| ops-sre-reliability | Frontend incident handoff | Attach dashboard, runbook branch, and rollback commit. |
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
| Workflow Studio canvas | The visual workflow graph editor with nodes, edges, presence, zoom, pan, and validation. |
| Yjs CRDT | Collaborative editing data model used by the Workflow Studio Yjs ADR and guarded by portability doctrine. |
| WCAG 2.2 AA | Accessibility floor for keyboard, focus, target size, contrast, error handling, and semantics. |
| locale overlay | Regional or tenant translation layer applied over canonical source strings. |
| presence awareness | Shared cursor, selection, and collaborator state that must not corrupt graph state. |

## Escalation Channels

| Escalation | Use when | Owner |
| --- |--- |--- |
| mentor checkpoint | you can proceed locally but need review of reasoning or evidence | assigned mentor |
| axis owner | a file or policy belongs to another team | axis-workflow-studio |
| council review | claim boundary, doctrine, compliance, or security interpretation changes | axis-frontend + axis-workflow-studio + council-design-system |
| SRE on-call | dev-cell, incident, or reliability path blocks verification | ops-sre-reliability |
| security review | credential, tenant isolation, policy, or regulated data risk appears | ops-security |

## Resources & References

- docs/standards/wcag-2-2-aa-checklist.md
- docs/standards/locale-routing.md
- docs/standards/ux-best-practices.md
- microservices/workflow-studio/ARCHITECTURE.md
- microservices/workflow-studio/PHASE-01-VISUAL-AUTHORING-SUBSTRATE.md
- microservices/workflow-studio/benchmarks/n8n-zapier-make-workato-vs-oyatie.md
- microservices/workflow-studio/runbooks/canvas-perf-regression.md

Reference-reading protocol: open the resource, identify the authority section, write the one-sentence claim it supports, and record whether the resource is doctrine, spec, implementation, test, runbook, dashboard, or evidence.

## Role-Specific Drill Library

Use this ledger when you need extra practice or when a mentor asks for stronger evidence. Each drill is intentionally small but must end with a verifiable artifact.

### Drill FE-001: keyboard-only node creation
- Read: docs/decisions/ADR-0700-ci-admission-live-apex.md
- Connects to: SvelteKit web phase and Leptos phase-two trigger
- Build or inspect: a minimal artifact that proves keyboard-only node creation without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for keyboard-only node creation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show keyboard-only node creation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-001 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-002: Yjs merge replay
- Read: docs/decisions/ADR-0700-ci-admission-live-apex.md
- Connects to: svelte-flow adapter boundary and canvas future
- Build or inspect: a minimal artifact that proves Yjs merge replay without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Yjs merge replay.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Yjs merge replay is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-design-system with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-002 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-003: Fluent key fallback
- Read: docs/decisions/ADR-0704-k8s-port-live-apex.md
- Connects to: Yjs collaborative editing semantics and CRDT conflict resolution
- Build or inspect: a minimal artifact that proves Fluent key fallback without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Fluent key fallback.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Fluent key fallback is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-i18n with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-003 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-004: ko-KR locale overlay
- Read: docs/standards/wcag-2-2-aa-checklist.md
- Connects to: ADR-0142 portability trait and alternate adapter compile gate
- Build or inspect: a minimal artifact that proves ko-KR locale overlay without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ko-KR locale overlay.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ko-KR locale overlay is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-004 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-005: ar-SA RTL check
- Read: docs/standards/locale-routing.md
- Connects to: WCAG 2.2 AA keyboard, focus, target-size, and error-state coverage
- Build or inspect: a minimal artifact that proves ar-SA RTL check without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ar-SA RTL check.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ar-SA RTL check is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-005 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-006: canvas 1000-node pan budget
- Read: docs/standards/i18n-canonical.md
- Connects to: Fluent locale overlays and tenant regional packs
- Build or inspect: a minimal artifact that proves canvas 1000-node pan budget without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for canvas 1000-node pan budget.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show canvas 1000-node pan budget is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-006 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-007: presence disconnect recovery
- Read: microservices/workflow-studio/ARCHITECTURE.md
- Connects to: canvas p99 frame-time budget at 1000 nodes
- Build or inspect: a minimal artifact that proves presence disconnect recovery without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for presence disconnect recovery.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show presence disconnect recovery is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-design-system with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-007 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-008: Cedar-denied UI state
- Read: microservices/workflow-studio/PRD.md
- Connects to: presence awareness and selection halo semantics
- Build or inspect: a minimal artifact that proves Cedar-denied UI state without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar-denied UI state.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar-denied UI state is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-i18n with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-008 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-009: minor-user restricted template
- Read: microservices/workflow-studio/contracts/openapi/workflow-studio.yaml
- Connects to: Cedar-driven UX affordance hiding versus disabled controls
- Build or inspect: a minimal artifact that proves minor-user restricted template without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for minor-user restricted template.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show minor-user restricted template is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-009 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-010: focus ring visible state
- Read: microservices/workflow-studio/clients/web-sveltekit/src/templates/CatalogBrowser.svelte
- Connects to: minor-user UX restrictions under ADR-0292
- Build or inspect: a minimal artifact that proves focus ring visible state without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for focus ring visible state.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show focus ring visible state is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-010 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-011: screen reader node label
- Read: microservices/workflow-studio/clients/web-sveltekit/src/templates/TemplateDetail.svelte
- Connects to: SvelteKit web phase and Leptos phase-two trigger
- Build or inspect: a minimal artifact that proves screen reader node label without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for screen reader node label.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show screen reader node label is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-011 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-012: template catalog load state
- Read: microservices/workflow-studio/clients/web-sveltekit/packages/i18n-source/source.ftl
- Connects to: svelte-flow adapter boundary and canvas future
- Build or inspect: a minimal artifact that proves template catalog load state without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for template catalog load state.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show template catalog load state is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-design-system with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-012 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-013: collab conflict runbook
- Read: microservices/workflow-studio/clients/web-sveltekit/packages/i18n-source/ko-KR.ftl
- Connects to: Yjs collaborative editing semantics and CRDT conflict resolution
- Build or inspect: a minimal artifact that proves collab conflict runbook without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for collab conflict runbook.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show collab conflict runbook is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-i18n with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-013 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-014: svelte-flow adapter boundary
- Read: microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml
- Connects to: ADR-0142 portability trait and alternate adapter compile gate
- Build or inspect: a minimal artifact that proves svelte-flow adapter boundary without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for svelte-flow adapter boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show svelte-flow adapter boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-014 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-015: OpenAPI contract DTO mismatch
- Read: microservices/workflow-studio/runbooks/collaborative-canvas-merge-conflict.md
- Connects to: WCAG 2.2 AA keyboard, focus, target-size, and error-state coverage
- Build or inspect: a minimal artifact that proves OpenAPI contract DTO mismatch without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for OpenAPI contract DTO mismatch.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show OpenAPI contract DTO mismatch is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-015 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-016: visual regression screenshot
- Read: docs/decisions/ADR-0700-ci-admission-live-apex.md
- Connects to: Fluent locale overlays and tenant regional packs
- Build or inspect: a minimal artifact that proves visual regression screenshot without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for visual regression screenshot.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show visual regression screenshot is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-016 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-017: mobile-portrait toolbar wrap
- Read: docs/decisions/ADR-0700-ci-admission-live-apex.md
- Connects to: canvas p99 frame-time budget at 1000 nodes
- Build or inspect: a minimal artifact that proves mobile-portrait toolbar wrap without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for mobile-portrait toolbar wrap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show mobile-portrait toolbar wrap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-design-system with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-017 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-018: wide-desktop minimap layout
- Read: docs/decisions/ADR-0704-k8s-port-live-apex.md
- Connects to: presence awareness and selection halo semantics
- Build or inspect: a minimal artifact that proves wide-desktop minimap layout without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for wide-desktop minimap layout.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show wide-desktop minimap layout is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-i18n with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-018 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-019: keyboard-only node creation
- Read: docs/standards/wcag-2-2-aa-checklist.md
- Connects to: Cedar-driven UX affordance hiding versus disabled controls
- Build or inspect: a minimal artifact that proves keyboard-only node creation without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for keyboard-only node creation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show keyboard-only node creation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-019 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-020: Yjs merge replay
- Read: docs/standards/locale-routing.md
- Connects to: minor-user UX restrictions under ADR-0292
- Build or inspect: a minimal artifact that proves Yjs merge replay without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Yjs merge replay.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Yjs merge replay is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-020 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-021: Fluent key fallback
- Read: docs/standards/i18n-canonical.md
- Connects to: SvelteKit web phase and Leptos phase-two trigger
- Build or inspect: a minimal artifact that proves Fluent key fallback without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Fluent key fallback.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Fluent key fallback is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-021 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-022: ko-KR locale overlay
- Read: microservices/workflow-studio/ARCHITECTURE.md
- Connects to: svelte-flow adapter boundary and canvas future
- Build or inspect: a minimal artifact that proves ko-KR locale overlay without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ko-KR locale overlay.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ko-KR locale overlay is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-design-system with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-022 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-023: ar-SA RTL check
- Read: microservices/workflow-studio/PRD.md
- Connects to: Yjs collaborative editing semantics and CRDT conflict resolution
- Build or inspect: a minimal artifact that proves ar-SA RTL check without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ar-SA RTL check.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ar-SA RTL check is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-i18n with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-023 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-024: canvas 1000-node pan budget
- Read: microservices/workflow-studio/contracts/openapi/workflow-studio.yaml
- Connects to: ADR-0142 portability trait and alternate adapter compile gate
- Build or inspect: a minimal artifact that proves canvas 1000-node pan budget without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for canvas 1000-node pan budget.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show canvas 1000-node pan budget is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-024 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-025: presence disconnect recovery
- Read: microservices/workflow-studio/clients/web-sveltekit/src/templates/CatalogBrowser.svelte
- Connects to: WCAG 2.2 AA keyboard, focus, target-size, and error-state coverage
- Build or inspect: a minimal artifact that proves presence disconnect recovery without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for presence disconnect recovery.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show presence disconnect recovery is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-025 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-026: Cedar-denied UI state
- Read: microservices/workflow-studio/clients/web-sveltekit/src/templates/TemplateDetail.svelte
- Connects to: Fluent locale overlays and tenant regional packs
- Build or inspect: a minimal artifact that proves Cedar-denied UI state without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar-denied UI state.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar-denied UI state is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-026 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-027: minor-user restricted template
- Read: microservices/workflow-studio/clients/web-sveltekit/packages/i18n-source/source.ftl
- Connects to: canvas p99 frame-time budget at 1000 nodes
- Build or inspect: a minimal artifact that proves minor-user restricted template without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for minor-user restricted template.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show minor-user restricted template is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-design-system with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-027 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-028: focus ring visible state
- Read: microservices/workflow-studio/clients/web-sveltekit/packages/i18n-source/ko-KR.ftl
- Connects to: presence awareness and selection halo semantics
- Build or inspect: a minimal artifact that proves focus ring visible state without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for focus ring visible state.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show focus ring visible state is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-i18n with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-028 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-029: screen reader node label
- Read: microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml
- Connects to: Cedar-driven UX affordance hiding versus disabled controls
- Build or inspect: a minimal artifact that proves screen reader node label without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for screen reader node label.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show screen reader node label is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-029 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-030: template catalog load state
- Read: microservices/workflow-studio/runbooks/collaborative-canvas-merge-conflict.md
- Connects to: minor-user UX restrictions under ADR-0292
- Build or inspect: a minimal artifact that proves template catalog load state without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for template catalog load state.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show template catalog load state is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-030 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-031: collab conflict runbook
- Read: docs/decisions/ADR-0700-ci-admission-live-apex.md
- Connects to: SvelteKit web phase and Leptos phase-two trigger
- Build or inspect: a minimal artifact that proves collab conflict runbook without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for collab conflict runbook.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show collab conflict runbook is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-031 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-032: svelte-flow adapter boundary
- Read: docs/decisions/ADR-0700-ci-admission-live-apex.md
- Connects to: svelte-flow adapter boundary and canvas future
- Build or inspect: a minimal artifact that proves svelte-flow adapter boundary without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for svelte-flow adapter boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show svelte-flow adapter boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-design-system with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-032 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-033: OpenAPI contract DTO mismatch
- Read: docs/decisions/ADR-0704-k8s-port-live-apex.md
- Connects to: Yjs collaborative editing semantics and CRDT conflict resolution
- Build or inspect: a minimal artifact that proves OpenAPI contract DTO mismatch without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for OpenAPI contract DTO mismatch.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show OpenAPI contract DTO mismatch is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-i18n with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-033 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-034: visual regression screenshot
- Read: docs/standards/wcag-2-2-aa-checklist.md
- Connects to: ADR-0142 portability trait and alternate adapter compile gate
- Build or inspect: a minimal artifact that proves visual regression screenshot without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for visual regression screenshot.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show visual regression screenshot is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-034 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-035: mobile-portrait toolbar wrap
- Read: docs/standards/locale-routing.md
- Connects to: WCAG 2.2 AA keyboard, focus, target-size, and error-state coverage
- Build or inspect: a minimal artifact that proves mobile-portrait toolbar wrap without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for mobile-portrait toolbar wrap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show mobile-portrait toolbar wrap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-035 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-036: wide-desktop minimap layout
- Read: docs/standards/i18n-canonical.md
- Connects to: Fluent locale overlays and tenant regional packs
- Build or inspect: a minimal artifact that proves wide-desktop minimap layout without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for wide-desktop minimap layout.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show wide-desktop minimap layout is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-036 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-037: keyboard-only node creation
- Read: microservices/workflow-studio/ARCHITECTURE.md
- Connects to: canvas p99 frame-time budget at 1000 nodes
- Build or inspect: a minimal artifact that proves keyboard-only node creation without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for keyboard-only node creation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show keyboard-only node creation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-design-system with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-037 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-038: Yjs merge replay
- Read: microservices/workflow-studio/PRD.md
- Connects to: presence awareness and selection halo semantics
- Build or inspect: a minimal artifact that proves Yjs merge replay without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Yjs merge replay.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Yjs merge replay is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-i18n with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-038 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-039: Fluent key fallback
- Read: microservices/workflow-studio/contracts/openapi/workflow-studio.yaml
- Connects to: Cedar-driven UX affordance hiding versus disabled controls
- Build or inspect: a minimal artifact that proves Fluent key fallback without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Fluent key fallback.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Fluent key fallback is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-039 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-040: ko-KR locale overlay
- Read: microservices/workflow-studio/clients/web-sveltekit/src/templates/CatalogBrowser.svelte
- Connects to: minor-user UX restrictions under ADR-0292
- Build or inspect: a minimal artifact that proves ko-KR locale overlay without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ko-KR locale overlay.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ko-KR locale overlay is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-040 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-041: ar-SA RTL check
- Read: microservices/workflow-studio/clients/web-sveltekit/src/templates/TemplateDetail.svelte
- Connects to: SvelteKit web phase and Leptos phase-two trigger
- Build or inspect: a minimal artifact that proves ar-SA RTL check without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ar-SA RTL check.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ar-SA RTL check is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-041 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-042: canvas 1000-node pan budget
- Read: microservices/workflow-studio/clients/web-sveltekit/packages/i18n-source/source.ftl
- Connects to: svelte-flow adapter boundary and canvas future
- Build or inspect: a minimal artifact that proves canvas 1000-node pan budget without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for canvas 1000-node pan budget.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show canvas 1000-node pan budget is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-design-system with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-042 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-043: presence disconnect recovery
- Read: microservices/workflow-studio/clients/web-sveltekit/packages/i18n-source/ko-KR.ftl
- Connects to: Yjs collaborative editing semantics and CRDT conflict resolution
- Build or inspect: a minimal artifact that proves presence disconnect recovery without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for presence disconnect recovery.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show presence disconnect recovery is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-i18n with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-043 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-044: Cedar-denied UI state
- Read: microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml
- Connects to: ADR-0142 portability trait and alternate adapter compile gate
- Build or inspect: a minimal artifact that proves Cedar-denied UI state without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar-denied UI state.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar-denied UI state is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-044 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-045: minor-user restricted template
- Read: microservices/workflow-studio/runbooks/collaborative-canvas-merge-conflict.md
- Connects to: WCAG 2.2 AA keyboard, focus, target-size, and error-state coverage
- Build or inspect: a minimal artifact that proves minor-user restricted template without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for minor-user restricted template.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show minor-user restricted template is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-045 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-046: focus ring visible state
- Read: docs/decisions/ADR-0700-ci-admission-live-apex.md
- Connects to: Fluent locale overlays and tenant regional packs
- Build or inspect: a minimal artifact that proves focus ring visible state without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for focus ring visible state.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show focus ring visible state is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-046 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-047: screen reader node label
- Read: docs/decisions/ADR-0700-ci-admission-live-apex.md
- Connects to: canvas p99 frame-time budget at 1000 nodes
- Build or inspect: a minimal artifact that proves screen reader node label without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for screen reader node label.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show screen reader node label is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-design-system with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-047 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-048: template catalog load state
- Read: docs/decisions/ADR-0704-k8s-port-live-apex.md
- Connects to: presence awareness and selection halo semantics
- Build or inspect: a minimal artifact that proves template catalog load state without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for template catalog load state.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show template catalog load state is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-i18n with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-048 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-049: collab conflict runbook
- Read: docs/standards/wcag-2-2-aa-checklist.md
- Connects to: Cedar-driven UX affordance hiding versus disabled controls
- Build or inspect: a minimal artifact that proves collab conflict runbook without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for collab conflict runbook.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show collab conflict runbook is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-049 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-050: svelte-flow adapter boundary
- Read: docs/standards/locale-routing.md
- Connects to: minor-user UX restrictions under ADR-0292
- Build or inspect: a minimal artifact that proves svelte-flow adapter boundary without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for svelte-flow adapter boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show svelte-flow adapter boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-050 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-051: OpenAPI contract DTO mismatch
- Read: docs/standards/i18n-canonical.md
- Connects to: SvelteKit web phase and Leptos phase-two trigger
- Build or inspect: a minimal artifact that proves OpenAPI contract DTO mismatch without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for OpenAPI contract DTO mismatch.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show OpenAPI contract DTO mismatch is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-051 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-052: visual regression screenshot
- Read: microservices/workflow-studio/ARCHITECTURE.md
- Connects to: svelte-flow adapter boundary and canvas future
- Build or inspect: a minimal artifact that proves visual regression screenshot without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for visual regression screenshot.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show visual regression screenshot is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-design-system with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-052 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-053: mobile-portrait toolbar wrap
- Read: microservices/workflow-studio/PRD.md
- Connects to: Yjs collaborative editing semantics and CRDT conflict resolution
- Build or inspect: a minimal artifact that proves mobile-portrait toolbar wrap without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for mobile-portrait toolbar wrap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show mobile-portrait toolbar wrap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-i18n with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-053 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-054: wide-desktop minimap layout
- Read: microservices/workflow-studio/contracts/openapi/workflow-studio.yaml
- Connects to: ADR-0142 portability trait and alternate adapter compile gate
- Build or inspect: a minimal artifact that proves wide-desktop minimap layout without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for wide-desktop minimap layout.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show wide-desktop minimap layout is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-054 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-055: keyboard-only node creation
- Read: microservices/workflow-studio/clients/web-sveltekit/src/templates/CatalogBrowser.svelte
- Connects to: WCAG 2.2 AA keyboard, focus, target-size, and error-state coverage
- Build or inspect: a minimal artifact that proves keyboard-only node creation without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for keyboard-only node creation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show keyboard-only node creation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-055 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-056: Yjs merge replay
- Read: microservices/workflow-studio/clients/web-sveltekit/src/templates/TemplateDetail.svelte
- Connects to: Fluent locale overlays and tenant regional packs
- Build or inspect: a minimal artifact that proves Yjs merge replay without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Yjs merge replay.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Yjs merge replay is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-056 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-057: Fluent key fallback
- Read: microservices/workflow-studio/clients/web-sveltekit/packages/i18n-source/source.ftl
- Connects to: canvas p99 frame-time budget at 1000 nodes
- Build or inspect: a minimal artifact that proves Fluent key fallback without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Fluent key fallback.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Fluent key fallback is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-design-system with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-057 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-058: ko-KR locale overlay
- Read: microservices/workflow-studio/clients/web-sveltekit/packages/i18n-source/ko-KR.ftl
- Connects to: presence awareness and selection halo semantics
- Build or inspect: a minimal artifact that proves ko-KR locale overlay without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ko-KR locale overlay.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ko-KR locale overlay is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-i18n with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-058 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-059: ar-SA RTL check
- Read: microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml
- Connects to: Cedar-driven UX affordance hiding versus disabled controls
- Build or inspect: a minimal artifact that proves ar-SA RTL check without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ar-SA RTL check.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ar-SA RTL check is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-059 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-060: canvas 1000-node pan budget
- Read: microservices/workflow-studio/runbooks/collaborative-canvas-merge-conflict.md
- Connects to: minor-user UX restrictions under ADR-0292
- Build or inspect: a minimal artifact that proves canvas 1000-node pan budget without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for canvas 1000-node pan budget.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show canvas 1000-node pan budget is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-060 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-061: presence disconnect recovery
- Read: docs/decisions/ADR-0700-ci-admission-live-apex.md
- Connects to: SvelteKit web phase and Leptos phase-two trigger
- Build or inspect: a minimal artifact that proves presence disconnect recovery without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for presence disconnect recovery.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show presence disconnect recovery is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-061 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-062: Cedar-denied UI state
- Read: docs/decisions/ADR-0700-ci-admission-live-apex.md
- Connects to: svelte-flow adapter boundary and canvas future
- Build or inspect: a minimal artifact that proves Cedar-denied UI state without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar-denied UI state.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar-denied UI state is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-design-system with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-062 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-063: minor-user restricted template
- Read: docs/decisions/ADR-0704-k8s-port-live-apex.md
- Connects to: Yjs collaborative editing semantics and CRDT conflict resolution
- Build or inspect: a minimal artifact that proves minor-user restricted template without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for minor-user restricted template.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show minor-user restricted template is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-i18n with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-063 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-064: focus ring visible state
- Read: docs/standards/wcag-2-2-aa-checklist.md
- Connects to: ADR-0142 portability trait and alternate adapter compile gate
- Build or inspect: a minimal artifact that proves focus ring visible state without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for focus ring visible state.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show focus ring visible state is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-064 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-065: screen reader node label
- Read: docs/standards/locale-routing.md
- Connects to: WCAG 2.2 AA keyboard, focus, target-size, and error-state coverage
- Build or inspect: a minimal artifact that proves screen reader node label without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for screen reader node label.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show screen reader node label is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-065 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-066: template catalog load state
- Read: docs/standards/i18n-canonical.md
- Connects to: Fluent locale overlays and tenant regional packs
- Build or inspect: a minimal artifact that proves template catalog load state without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for template catalog load state.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show template catalog load state is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-066 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-067: collab conflict runbook
- Read: microservices/workflow-studio/ARCHITECTURE.md
- Connects to: canvas p99 frame-time budget at 1000 nodes
- Build or inspect: a minimal artifact that proves collab conflict runbook without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for collab conflict runbook.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show collab conflict runbook is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-design-system with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-067 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-068: svelte-flow adapter boundary
- Read: microservices/workflow-studio/PRD.md
- Connects to: presence awareness and selection halo semantics
- Build or inspect: a minimal artifact that proves svelte-flow adapter boundary without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for svelte-flow adapter boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show svelte-flow adapter boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-i18n with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-068 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-069: OpenAPI contract DTO mismatch
- Read: microservices/workflow-studio/contracts/openapi/workflow-studio.yaml
- Connects to: Cedar-driven UX affordance hiding versus disabled controls
- Build or inspect: a minimal artifact that proves OpenAPI contract DTO mismatch without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for OpenAPI contract DTO mismatch.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show OpenAPI contract DTO mismatch is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-069 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-070: visual regression screenshot
- Read: microservices/workflow-studio/clients/web-sveltekit/src/templates/CatalogBrowser.svelte
- Connects to: minor-user UX restrictions under ADR-0292
- Build or inspect: a minimal artifact that proves visual regression screenshot without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for visual regression screenshot.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show visual regression screenshot is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-070 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-071: mobile-portrait toolbar wrap
- Read: microservices/workflow-studio/clients/web-sveltekit/src/templates/TemplateDetail.svelte
- Connects to: SvelteKit web phase and Leptos phase-two trigger
- Build or inspect: a minimal artifact that proves mobile-portrait toolbar wrap without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for mobile-portrait toolbar wrap.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show mobile-portrait toolbar wrap is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-workflow-studio with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-071 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-072: wide-desktop minimap layout
- Read: microservices/workflow-studio/clients/web-sveltekit/packages/i18n-source/source.ftl
- Connects to: svelte-flow adapter boundary and canvas future
- Build or inspect: a minimal artifact that proves wide-desktop minimap layout without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for wide-desktop minimap layout.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show wide-desktop minimap layout is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-design-system with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-072 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-073: keyboard-only node creation
- Read: microservices/workflow-studio/clients/web-sveltekit/packages/i18n-source/ko-KR.ftl
- Connects to: Yjs collaborative editing semantics and CRDT conflict resolution
- Build or inspect: a minimal artifact that proves keyboard-only node creation without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for keyboard-only node creation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show keyboard-only node creation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-i18n with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-073 contains file path, claim, evidence, rollback, and reviewer.

### Drill FE-074: Yjs merge replay
- Read: microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml
- Connects to: ADR-0142 portability trait and alternate adapter compile gate
- Build or inspect: a minimal artifact that proves Yjs merge replay without widening beyond SPA and web engineer, Workflow Studio and web surfaces.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Yjs merge replay.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Yjs merge replay is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row FE-074 contains file path, claim, evidence, rollback, and reviewer.

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
