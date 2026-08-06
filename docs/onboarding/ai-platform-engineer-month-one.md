---
doc_class: OnboardingGuide
role: "AI platform engineer, intelligence microservice"
status: Published
date: 2026-05-20
owner: "axis-intelligence + ops-ml-platform + council-ml"
related_oyatie_adrs:
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
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

# AI Platform Engineer Month-One Onboarding

Audience: AI platform engineer, intelligence microservice.
Industry precedent: Google model-card discipline, NIST AI RMF lifecycle governance, EU AI Act risk-tier obligations, and OpenAI-style model routing guardrail separation.

This guide is written for a programming-capable new joiner with no prior Oyatie architecture knowledge. Every phase names the repo files to read, the artifact to produce, the owner who reviews it, and the stop condition that proves the phase is complete.

Substance rule: do not treat this guide as orientation prose. Treat it as a work plan whose outputs can be inspected, replayed, or rejected.

You join the intelligence microservice to build model routing, retrieval, prompt fences, evals, tenant isolation, and AI Act compliance into the platform substrate rather than bolting them onto product surfaces.
Month one is a controlled path through ADR-0255, model lifecycle ADRs, RAG contracts, prompt guardrail capabilities, and EU AI Act risk tiers. You will ship a small change only after proving tenant isolation and policy evidence.
The intelligence service is two-layer: audience-neutral AI substrate plus consumer brand surface. Your onboarding work stays in the substrate unless a mentor explicitly assigns a brand-surface task.

## Hyperscaler-Grade Reading Contract

- Named precedent: Google model-card discipline, NIST AI RMF lifecycle governance, EU AI Act risk-tier obligations, and OpenAI-style model routing guardrail separation.
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
1. docs/decisions/ADR-0701-monorepo-capability-live-apex.md
2. docs/decisions/ADR-0709-general-live-apex.md
3. docs/decisions/ADR-0709-general-live-apex.md
4. specs/capabilities/eu-ai-act-risk-class-registry.json
5. contracts/openapi/foundry/rag-v1.yaml
6. contracts/openapi/foundry/rag-v1.meta.yaml
7. crates/oya-intelligence-rag-api/src/lib.rs
8. crates/oya-intelligence-rag-api/tests/foundry_rag_retrieve_api.rs
9. crates/oya-intelligence-rag-endpoint-kernel/src/lib.rs
10. microservices/intelligence/catalog/oya-intelligence-guardrails-prompt-classifier-kernel.yaml
11. microservices/intelligence/capabilities/guardrails-classify-prompt.yaml
12. microservices/intelligence/IP-064-guardrails-prompt-classifier-kernel.md
13. docs/runbooks/foundry/prompt-injection-fired.md
14. docs/runbooks/foundry-model-cutover.md
15. docs/runbooks/foundry-model-lora-adapter-rollback.md


### Named ADRs to read

- ADR-0255 intelligence as two-layer AI substrate
- ADR-0308 ML model lifecycle AI Act compliance
- ADR-0144 EU AI Act graduated risk tier model
- ADR-0309 fairness audit and civil rights
- ADR-0243 Cedar universal gate

### Named playgrounds

1. crates/oya-intelligence-rag-api/tests/foundry_rag_retrieve_api.rs
   - Artifact: write a four-sentence note explaining what this playground proves for AI platform engineer, intelligence microservice.
2. contracts/openapi/foundry/rag-v1.yaml
   - Artifact: write a four-sentence note explaining what this playground proves for AI platform engineer, intelligence microservice.
3. microservices/intelligence/capabilities/guardrails-classify-prompt.yaml
   - Artifact: write a four-sentence note explaining what this playground proves for AI platform engineer, intelligence microservice.
4. docs/runbooks/foundry/prompt-injection-fired.md
   - Artifact: write a four-sentence note explaining what this playground proves for AI platform engineer, intelligence microservice.

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

1. AI-STARTER-001 add one tenant-isolation assertion to the RAG retrieve API test
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for AI.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
2. AI-STARTER-002 add one prompt-fence failure case to a guardrails capability record
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for AI.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
3. AI-STARTER-003 map one capability to the EU AI Act graduated risk registry
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for AI.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
4. AI-STARTER-004 improve a model rollback runbook with eval and audit evidence checks
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for AI.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.

### Mentor pairing protocol

Pair with an intelligence substrate mentor for model routing, an ML governance mentor for lifecycle evidence, and a policy-engine mentor for Cedar gating. Every pairing ends with tenant id, model id, data class, risk tier, prompt fence, eval, and rollback note.
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

First independent project: Own `AI-PROJ-001`: add a per-tenant RAG retrieval guardrail fixture that refuses cross-tenant context, emits an audit event, and maps the request to an EU AI Act risk tier.

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

- per-tenant model router smoke evidence
- RAG tenant isolation test fixture
- prompt fence eval set improvement
- EU AI Act risk-tier registry mapping

### Key contacts in other teams

- axis-intelligence
- ops-ml-platform
- axis-foundry
- axis-policy-engine
- axis-audit-chain
- ops-compliance
- council-privacy

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
1. two-layer intelligence substrate and brand surface separation
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for two-layer intelligence substrate and brand surface separation.
2. per-tenant model router and provider-BYOK credential model
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for per-tenant model router and provider-BYOK credential model.
3. RAG retrieval isolation and context boundary
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for RAG retrieval isolation and context boundary.
4. prompt fences and prompt-injection classifier capability
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for prompt fences and prompt-injection classifier capability.
5. model card, lifecycle stage, eval, and rollback schema
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for model card, lifecycle stage, eval, and rollback schema.
6. EU AI Act graduated risk tier and deployment-context mutation
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for EU AI Act graduated risk tier and deployment-context mutation.
7. high-risk AI refusal until conformity path exists
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for high-risk AI refusal until conformity path exists.
8. fairness audit and civil-rights evidence
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for fairness audit and civil-rights evidence.
9. Cedar gating for AI capability execution
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for Cedar gating for AI capability execution.
10. audit-chain attribution for model choice and generated output
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for audit-chain attribution for model choice and generated output.
Domain expertise stop condition: you can answer what breaks if this topic is implemented incorrectly, who owns the rollback, and what evidence proves the system recovered.
- Verification for Month 2: deep-dive notes cover every listed topic and at least one note is reviewed by a cross-team owner
- Stop condition for Month 2: mentor and owner can point to the artifact without asking you to explain hidden context.

## Quarter 1: Ownership


### Named OKRs

- OKR-AI-Q1-1: ship one AI substrate PR with tenant isolation, eval, audit, and rollback evidence
- OKR-AI-Q1-2: map one AI capability to EU AI Act tier with model card and risk evidence
- OKR-AI-Q1-3: add one prompt-fence regression that catches injection or cross-tenant leakage

### Named on-call rotation entry

Enter `ai-platform-shadow` rotation after one model-router or RAG guardrail PR merges; quarter-one target is one model rollback drill and one prompt-injection incident shadow.

### Named team-OKR contribution

Contribute to `TEAM-OKR-INTELLIGENCE-2026Q2`: every AI substrate call has tenant isolation, Cedar gate, prompt fence, eval evidence, risk tier, and audit-chain attribution.

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

1. Routing by provider preference without tenant policy and cost evidence.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for AI platform engineer, intelligence microservice.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
2. Letting prompts carry unscoped tenant context into shared retrieval.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for AI platform engineer, intelligence microservice.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
3. Treating prompt fences as UX copy instead of enforceable guardrails.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for AI platform engineer, intelligence microservice.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
4. Calling a model low-risk without deployment-context risk analysis.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for AI platform engineer, intelligence microservice.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
5. Shipping RAG retrieval tests that never assert cross-tenant refusal.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for AI platform engineer, intelligence microservice.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
6. Skipping model-card and rollback evidence for a model version change.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for AI platform engineer, intelligence microservice.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
7. Putting brand-surface behavior into the substrate layer.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for AI platform engineer, intelligence microservice.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
8. Using AI Act language as a label instead of binding it to obligations and refusal states.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for AI platform engineer, intelligence microservice.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.

## Cross-Team Collaboration Playbook

| Team | Handoff ritual | Minimum payload |
| --- |--- |--- |
| axis-policy-engine | AI gate handoff | Attach Cedar permit, deny case, risk tier, and prompt fence id. |
| axis-audit-chain | AI attribution handoff | Attach model id, tenant id, retrieval set id, output event, and retention class. |
| ops-ml-platform | Model lifecycle handoff | Attach model card, eval result, drift signal, and rollback plan. |
| ops-compliance | AI Act handoff | Attach risk class, deployment context, transparency duty, and conformity status. |
| council-privacy | RAG data boundary handoff | Attach data class, lawful basis, retention rule, and DSR effect. |
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
| model router | Per-tenant decision layer that selects provider and model under policy, cost, risk, and availability constraints. |
| RAG | Retrieval augmented generation path that must keep tenant context isolated and auditable. |
| prompt fence | Guardrail that constrains prompt, tool, retrieval, and output behavior. |
| model card | Versioned description of model purpose, data, evals, risk, and limitations. |
| EU AI Act tier | Risk class derived from capability and deployment context. |

## Escalation Channels

| Escalation | Use when | Owner |
| --- |--- |--- |
| mentor checkpoint | you can proceed locally but need review of reasoning or evidence | assigned mentor |
| axis owner | a file or policy belongs to another team | axis-intelligence |
| council review | claim boundary, doctrine, compliance, or security interpretation changes | axis-intelligence + ops-ml-platform + council-ml |
| SRE on-call | dev-cell, incident, or reliability path blocks verification | ops-sre-reliability |
| security review | credential, tenant isolation, policy, or regulated data risk appears | ops-security |

## Resources & References

- docs/decisions/ADR-0701-monorepo-capability-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md
- contracts/openapi/foundry/rag-v1.yaml
- docs/runbooks/foundry/prompt-injection-fired.md
- docs/runbooks/foundry-model-cutover.md
- specs/capabilities/eu-ai-act-risk-class-registry.json

Reference-reading protocol: open the resource, identify the authority section, write the one-sentence claim it supports, and record whether the resource is doctrine, spec, implementation, test, runbook, dashboard, or evidence.

## Role-Specific Drill Library

Use this ledger when you need extra practice or when a mentor asks for stronger evidence. Each drill is intentionally small but must end with a verifiable artifact.

### Drill AI-001: tenant-isolated RAG retrieve
- Read: docs/decisions/ADR-0701-monorepo-capability-live-apex.md
- Connects to: two-layer intelligence substrate and brand surface separation
- Build or inspect: a minimal artifact that proves tenant-isolated RAG retrieve without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant-isolated RAG retrieve.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant-isolated RAG retrieve is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-intelligence with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-001 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-002: model router policy deny
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: per-tenant model router and provider-BYOK credential model
- Build or inspect: a minimal artifact that proves model router policy deny without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for model router policy deny.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show model router policy deny is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-ml-platform with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-002 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-003: provider-BYOK credential selection
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: RAG retrieval isolation and context boundary
- Build or inspect: a minimal artifact that proves provider-BYOK credential selection without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for provider-BYOK credential selection.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show provider-BYOK credential selection is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-foundry with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-003 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-004: prompt injection fired
- Read: specs/capabilities/eu-ai-act-risk-class-registry.json
- Connects to: prompt fences and prompt-injection classifier capability
- Build or inspect: a minimal artifact that proves prompt injection fired without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for prompt injection fired.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show prompt injection fired is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-004 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-005: prompt fence eval regression
- Read: contracts/openapi/foundry/rag-v1.yaml
- Connects to: model card, lifecycle stage, eval, and rollback schema
- Build or inspect: a minimal artifact that proves prompt fence eval regression without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for prompt fence eval regression.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show prompt fence eval regression is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-005 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-006: EU AI Act tier mutation
- Read: contracts/openapi/foundry/rag-v1.meta.yaml
- Connects to: EU AI Act graduated risk tier and deployment-context mutation
- Build or inspect: a minimal artifact that proves EU AI Act tier mutation without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for EU AI Act tier mutation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show EU AI Act tier mutation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-006 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-007: high-risk refusal
- Read: crates/oya-intelligence-rag-api/src/lib.rs
- Connects to: high-risk AI refusal until conformity path exists
- Build or inspect: a minimal artifact that proves high-risk refusal without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for high-risk refusal.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show high-risk refusal is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-007 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-008: model card completeness
- Read: crates/oya-intelligence-rag-api/tests/foundry_rag_retrieve_api.rs
- Connects to: fairness audit and civil-rights evidence
- Build or inspect: a minimal artifact that proves model card completeness without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for model card completeness.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show model card completeness is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-intelligence with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-008 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-009: model rollback drill
- Read: crates/oya-intelligence-rag-endpoint-kernel/src/lib.rs
- Connects to: Cedar gating for AI capability execution
- Build or inspect: a minimal artifact that proves model rollback drill without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for model rollback drill.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show model rollback drill is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-ml-platform with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-009 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-010: fairness audit evidence
- Read: microservices/intelligence/catalog/oya-intelligence-guardrails-prompt-classifier-kernel.yaml
- Connects to: audit-chain attribution for model choice and generated output
- Build or inspect: a minimal artifact that proves fairness audit evidence without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for fairness audit evidence.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show fairness audit evidence is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-foundry with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-010 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-011: retrieval source attribution
- Read: microservices/intelligence/capabilities/guardrails-classify-prompt.yaml
- Connects to: two-layer intelligence substrate and brand surface separation
- Build or inspect: a minimal artifact that proves retrieval source attribution without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for retrieval source attribution.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show retrieval source attribution is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-011 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-012: cross-tenant context leak
- Read: microservices/intelligence/IP-064-guardrails-prompt-classifier-kernel.md
- Connects to: per-tenant model router and provider-BYOK credential model
- Build or inspect: a minimal artifact that proves cross-tenant context leak without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cross-tenant context leak.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cross-tenant context leak is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-012 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-013: Cedar gated tool call
- Read: docs/runbooks/foundry/prompt-injection-fired.md
- Connects to: RAG retrieval isolation and context boundary
- Build or inspect: a minimal artifact that proves Cedar gated tool call without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar gated tool call.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar gated tool call is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-013 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-014: cost-aware model route
- Read: docs/runbooks/foundry-model-cutover.md
- Connects to: prompt fences and prompt-injection classifier capability
- Build or inspect: a minimal artifact that proves cost-aware model route without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cost-aware model route.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cost-aware model route is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-014 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-015: drift signal threshold
- Read: docs/runbooks/foundry-model-lora-adapter-rollback.md
- Connects to: model card, lifecycle stage, eval, and rollback schema
- Build or inspect: a minimal artifact that proves drift signal threshold without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for drift signal threshold.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show drift signal threshold is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-intelligence with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-015 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-016: human appeal path
- Read: docs/decisions/ADR-0701-monorepo-capability-live-apex.md
- Connects to: EU AI Act graduated risk tier and deployment-context mutation
- Build or inspect: a minimal artifact that proves human appeal path without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for human appeal path.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show human appeal path is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-ml-platform with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-016 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-017: GPAI transparency notice
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: high-risk AI refusal until conformity path exists
- Build or inspect: a minimal artifact that proves GPAI transparency notice without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for GPAI transparency notice.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show GPAI transparency notice is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-foundry with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-017 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-018: audit-chain output attribution
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: fairness audit and civil-rights evidence
- Build or inspect: a minimal artifact that proves audit-chain output attribution without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit-chain output attribution.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit-chain output attribution is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-018 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-019: tenant-isolated RAG retrieve
- Read: specs/capabilities/eu-ai-act-risk-class-registry.json
- Connects to: Cedar gating for AI capability execution
- Build or inspect: a minimal artifact that proves tenant-isolated RAG retrieve without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant-isolated RAG retrieve.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant-isolated RAG retrieve is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-019 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-020: model router policy deny
- Read: contracts/openapi/foundry/rag-v1.yaml
- Connects to: audit-chain attribution for model choice and generated output
- Build or inspect: a minimal artifact that proves model router policy deny without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for model router policy deny.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show model router policy deny is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-020 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-021: provider-BYOK credential selection
- Read: contracts/openapi/foundry/rag-v1.meta.yaml
- Connects to: two-layer intelligence substrate and brand surface separation
- Build or inspect: a minimal artifact that proves provider-BYOK credential selection without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for provider-BYOK credential selection.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show provider-BYOK credential selection is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-021 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-022: prompt injection fired
- Read: crates/oya-intelligence-rag-api/src/lib.rs
- Connects to: per-tenant model router and provider-BYOK credential model
- Build or inspect: a minimal artifact that proves prompt injection fired without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for prompt injection fired.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show prompt injection fired is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-intelligence with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-022 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-023: prompt fence eval regression
- Read: crates/oya-intelligence-rag-api/tests/foundry_rag_retrieve_api.rs
- Connects to: RAG retrieval isolation and context boundary
- Build or inspect: a minimal artifact that proves prompt fence eval regression without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for prompt fence eval regression.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show prompt fence eval regression is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-ml-platform with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-023 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-024: EU AI Act tier mutation
- Read: crates/oya-intelligence-rag-endpoint-kernel/src/lib.rs
- Connects to: prompt fences and prompt-injection classifier capability
- Build or inspect: a minimal artifact that proves EU AI Act tier mutation without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for EU AI Act tier mutation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show EU AI Act tier mutation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-foundry with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-024 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-025: high-risk refusal
- Read: microservices/intelligence/catalog/oya-intelligence-guardrails-prompt-classifier-kernel.yaml
- Connects to: model card, lifecycle stage, eval, and rollback schema
- Build or inspect: a minimal artifact that proves high-risk refusal without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for high-risk refusal.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show high-risk refusal is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-025 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-026: model card completeness
- Read: microservices/intelligence/capabilities/guardrails-classify-prompt.yaml
- Connects to: EU AI Act graduated risk tier and deployment-context mutation
- Build or inspect: a minimal artifact that proves model card completeness without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for model card completeness.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show model card completeness is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-026 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-027: model rollback drill
- Read: microservices/intelligence/IP-064-guardrails-prompt-classifier-kernel.md
- Connects to: high-risk AI refusal until conformity path exists
- Build or inspect: a minimal artifact that proves model rollback drill without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for model rollback drill.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show model rollback drill is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-027 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-028: fairness audit evidence
- Read: docs/runbooks/foundry/prompt-injection-fired.md
- Connects to: fairness audit and civil-rights evidence
- Build or inspect: a minimal artifact that proves fairness audit evidence without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for fairness audit evidence.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show fairness audit evidence is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-028 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-029: retrieval source attribution
- Read: docs/runbooks/foundry-model-cutover.md
- Connects to: Cedar gating for AI capability execution
- Build or inspect: a minimal artifact that proves retrieval source attribution without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for retrieval source attribution.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show retrieval source attribution is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-intelligence with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-029 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-030: cross-tenant context leak
- Read: docs/runbooks/foundry-model-lora-adapter-rollback.md
- Connects to: audit-chain attribution for model choice and generated output
- Build or inspect: a minimal artifact that proves cross-tenant context leak without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cross-tenant context leak.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cross-tenant context leak is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-ml-platform with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-030 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-031: Cedar gated tool call
- Read: docs/decisions/ADR-0701-monorepo-capability-live-apex.md
- Connects to: two-layer intelligence substrate and brand surface separation
- Build or inspect: a minimal artifact that proves Cedar gated tool call without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar gated tool call.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar gated tool call is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-foundry with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-031 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-032: cost-aware model route
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: per-tenant model router and provider-BYOK credential model
- Build or inspect: a minimal artifact that proves cost-aware model route without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cost-aware model route.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cost-aware model route is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-032 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-033: drift signal threshold
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: RAG retrieval isolation and context boundary
- Build or inspect: a minimal artifact that proves drift signal threshold without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for drift signal threshold.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show drift signal threshold is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-033 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-034: human appeal path
- Read: specs/capabilities/eu-ai-act-risk-class-registry.json
- Connects to: prompt fences and prompt-injection classifier capability
- Build or inspect: a minimal artifact that proves human appeal path without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for human appeal path.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show human appeal path is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-034 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-035: GPAI transparency notice
- Read: contracts/openapi/foundry/rag-v1.yaml
- Connects to: model card, lifecycle stage, eval, and rollback schema
- Build or inspect: a minimal artifact that proves GPAI transparency notice without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for GPAI transparency notice.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show GPAI transparency notice is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-035 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-036: audit-chain output attribution
- Read: contracts/openapi/foundry/rag-v1.meta.yaml
- Connects to: EU AI Act graduated risk tier and deployment-context mutation
- Build or inspect: a minimal artifact that proves audit-chain output attribution without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit-chain output attribution.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit-chain output attribution is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-intelligence with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-036 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-037: tenant-isolated RAG retrieve
- Read: crates/oya-intelligence-rag-api/src/lib.rs
- Connects to: high-risk AI refusal until conformity path exists
- Build or inspect: a minimal artifact that proves tenant-isolated RAG retrieve without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant-isolated RAG retrieve.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant-isolated RAG retrieve is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-ml-platform with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-037 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-038: model router policy deny
- Read: crates/oya-intelligence-rag-api/tests/foundry_rag_retrieve_api.rs
- Connects to: fairness audit and civil-rights evidence
- Build or inspect: a minimal artifact that proves model router policy deny without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for model router policy deny.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show model router policy deny is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-foundry with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-038 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-039: provider-BYOK credential selection
- Read: crates/oya-intelligence-rag-endpoint-kernel/src/lib.rs
- Connects to: Cedar gating for AI capability execution
- Build or inspect: a minimal artifact that proves provider-BYOK credential selection without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for provider-BYOK credential selection.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show provider-BYOK credential selection is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-039 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-040: prompt injection fired
- Read: microservices/intelligence/catalog/oya-intelligence-guardrails-prompt-classifier-kernel.yaml
- Connects to: audit-chain attribution for model choice and generated output
- Build or inspect: a minimal artifact that proves prompt injection fired without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for prompt injection fired.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show prompt injection fired is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-040 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-041: prompt fence eval regression
- Read: microservices/intelligence/capabilities/guardrails-classify-prompt.yaml
- Connects to: two-layer intelligence substrate and brand surface separation
- Build or inspect: a minimal artifact that proves prompt fence eval regression without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for prompt fence eval regression.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show prompt fence eval regression is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-041 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-042: EU AI Act tier mutation
- Read: microservices/intelligence/IP-064-guardrails-prompt-classifier-kernel.md
- Connects to: per-tenant model router and provider-BYOK credential model
- Build or inspect: a minimal artifact that proves EU AI Act tier mutation without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for EU AI Act tier mutation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show EU AI Act tier mutation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-042 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-043: high-risk refusal
- Read: docs/runbooks/foundry/prompt-injection-fired.md
- Connects to: RAG retrieval isolation and context boundary
- Build or inspect: a minimal artifact that proves high-risk refusal without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for high-risk refusal.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show high-risk refusal is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-intelligence with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-043 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-044: model card completeness
- Read: docs/runbooks/foundry-model-cutover.md
- Connects to: prompt fences and prompt-injection classifier capability
- Build or inspect: a minimal artifact that proves model card completeness without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for model card completeness.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show model card completeness is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-ml-platform with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-044 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-045: model rollback drill
- Read: docs/runbooks/foundry-model-lora-adapter-rollback.md
- Connects to: model card, lifecycle stage, eval, and rollback schema
- Build or inspect: a minimal artifact that proves model rollback drill without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for model rollback drill.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show model rollback drill is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-foundry with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-045 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-046: fairness audit evidence
- Read: docs/decisions/ADR-0701-monorepo-capability-live-apex.md
- Connects to: EU AI Act graduated risk tier and deployment-context mutation
- Build or inspect: a minimal artifact that proves fairness audit evidence without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for fairness audit evidence.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show fairness audit evidence is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-046 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-047: retrieval source attribution
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: high-risk AI refusal until conformity path exists
- Build or inspect: a minimal artifact that proves retrieval source attribution without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for retrieval source attribution.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show retrieval source attribution is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-047 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-048: cross-tenant context leak
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: fairness audit and civil-rights evidence
- Build or inspect: a minimal artifact that proves cross-tenant context leak without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cross-tenant context leak.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cross-tenant context leak is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-048 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-049: Cedar gated tool call
- Read: specs/capabilities/eu-ai-act-risk-class-registry.json
- Connects to: Cedar gating for AI capability execution
- Build or inspect: a minimal artifact that proves Cedar gated tool call without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar gated tool call.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar gated tool call is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-049 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-050: cost-aware model route
- Read: contracts/openapi/foundry/rag-v1.yaml
- Connects to: audit-chain attribution for model choice and generated output
- Build or inspect: a minimal artifact that proves cost-aware model route without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cost-aware model route.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cost-aware model route is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-intelligence with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-050 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-051: drift signal threshold
- Read: contracts/openapi/foundry/rag-v1.meta.yaml
- Connects to: two-layer intelligence substrate and brand surface separation
- Build or inspect: a minimal artifact that proves drift signal threshold without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for drift signal threshold.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show drift signal threshold is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-ml-platform with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-051 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-052: human appeal path
- Read: crates/oya-intelligence-rag-api/src/lib.rs
- Connects to: per-tenant model router and provider-BYOK credential model
- Build or inspect: a minimal artifact that proves human appeal path without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for human appeal path.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show human appeal path is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-foundry with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-052 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-053: GPAI transparency notice
- Read: crates/oya-intelligence-rag-api/tests/foundry_rag_retrieve_api.rs
- Connects to: RAG retrieval isolation and context boundary
- Build or inspect: a minimal artifact that proves GPAI transparency notice without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for GPAI transparency notice.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show GPAI transparency notice is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-053 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-054: audit-chain output attribution
- Read: crates/oya-intelligence-rag-endpoint-kernel/src/lib.rs
- Connects to: prompt fences and prompt-injection classifier capability
- Build or inspect: a minimal artifact that proves audit-chain output attribution without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit-chain output attribution.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit-chain output attribution is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-054 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-055: tenant-isolated RAG retrieve
- Read: microservices/intelligence/catalog/oya-intelligence-guardrails-prompt-classifier-kernel.yaml
- Connects to: model card, lifecycle stage, eval, and rollback schema
- Build or inspect: a minimal artifact that proves tenant-isolated RAG retrieve without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant-isolated RAG retrieve.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant-isolated RAG retrieve is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-055 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-056: model router policy deny
- Read: microservices/intelligence/capabilities/guardrails-classify-prompt.yaml
- Connects to: EU AI Act graduated risk tier and deployment-context mutation
- Build or inspect: a minimal artifact that proves model router policy deny without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for model router policy deny.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show model router policy deny is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-056 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-057: provider-BYOK credential selection
- Read: microservices/intelligence/IP-064-guardrails-prompt-classifier-kernel.md
- Connects to: high-risk AI refusal until conformity path exists
- Build or inspect: a minimal artifact that proves provider-BYOK credential selection without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for provider-BYOK credential selection.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show provider-BYOK credential selection is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-intelligence with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-057 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-058: prompt injection fired
- Read: docs/runbooks/foundry/prompt-injection-fired.md
- Connects to: fairness audit and civil-rights evidence
- Build or inspect: a minimal artifact that proves prompt injection fired without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for prompt injection fired.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show prompt injection fired is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-ml-platform with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-058 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-059: prompt fence eval regression
- Read: docs/runbooks/foundry-model-cutover.md
- Connects to: Cedar gating for AI capability execution
- Build or inspect: a minimal artifact that proves prompt fence eval regression without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for prompt fence eval regression.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show prompt fence eval regression is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-foundry with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-059 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-060: EU AI Act tier mutation
- Read: docs/runbooks/foundry-model-lora-adapter-rollback.md
- Connects to: audit-chain attribution for model choice and generated output
- Build or inspect: a minimal artifact that proves EU AI Act tier mutation without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for EU AI Act tier mutation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show EU AI Act tier mutation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-060 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-061: high-risk refusal
- Read: docs/decisions/ADR-0701-monorepo-capability-live-apex.md
- Connects to: two-layer intelligence substrate and brand surface separation
- Build or inspect: a minimal artifact that proves high-risk refusal without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for high-risk refusal.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show high-risk refusal is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-061 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-062: model card completeness
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: per-tenant model router and provider-BYOK credential model
- Build or inspect: a minimal artifact that proves model card completeness without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for model card completeness.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show model card completeness is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-062 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-063: model rollback drill
- Read: docs/decisions/ADR-0709-general-live-apex.md
- Connects to: RAG retrieval isolation and context boundary
- Build or inspect: a minimal artifact that proves model rollback drill without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for model rollback drill.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show model rollback drill is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-063 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-064: fairness audit evidence
- Read: specs/capabilities/eu-ai-act-risk-class-registry.json
- Connects to: prompt fences and prompt-injection classifier capability
- Build or inspect: a minimal artifact that proves fairness audit evidence without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for fairness audit evidence.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show fairness audit evidence is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-intelligence with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-064 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-065: retrieval source attribution
- Read: contracts/openapi/foundry/rag-v1.yaml
- Connects to: model card, lifecycle stage, eval, and rollback schema
- Build or inspect: a minimal artifact that proves retrieval source attribution without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for retrieval source attribution.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show retrieval source attribution is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-ml-platform with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-065 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-066: cross-tenant context leak
- Read: contracts/openapi/foundry/rag-v1.meta.yaml
- Connects to: EU AI Act graduated risk tier and deployment-context mutation
- Build or inspect: a minimal artifact that proves cross-tenant context leak without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cross-tenant context leak.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cross-tenant context leak is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-foundry with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-066 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-067: Cedar gated tool call
- Read: crates/oya-intelligence-rag-api/src/lib.rs
- Connects to: high-risk AI refusal until conformity path exists
- Build or inspect: a minimal artifact that proves Cedar gated tool call without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar gated tool call.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar gated tool call is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-067 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-068: cost-aware model route
- Read: crates/oya-intelligence-rag-api/tests/foundry_rag_retrieve_api.rs
- Connects to: fairness audit and civil-rights evidence
- Build or inspect: a minimal artifact that proves cost-aware model route without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for cost-aware model route.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show cost-aware model route is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-068 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-069: drift signal threshold
- Read: crates/oya-intelligence-rag-endpoint-kernel/src/lib.rs
- Connects to: Cedar gating for AI capability execution
- Build or inspect: a minimal artifact that proves drift signal threshold without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for drift signal threshold.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show drift signal threshold is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-069 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-070: human appeal path
- Read: microservices/intelligence/catalog/oya-intelligence-guardrails-prompt-classifier-kernel.yaml
- Connects to: audit-chain attribution for model choice and generated output
- Build or inspect: a minimal artifact that proves human appeal path without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for human appeal path.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show human appeal path is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-privacy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-070 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-071: GPAI transparency notice
- Read: microservices/intelligence/capabilities/guardrails-classify-prompt.yaml
- Connects to: two-layer intelligence substrate and brand surface separation
- Build or inspect: a minimal artifact that proves GPAI transparency notice without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for GPAI transparency notice.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show GPAI transparency notice is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-intelligence with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-071 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-072: audit-chain output attribution
- Read: microservices/intelligence/IP-064-guardrails-prompt-classifier-kernel.md
- Connects to: per-tenant model router and provider-BYOK credential model
- Build or inspect: a minimal artifact that proves audit-chain output attribution without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit-chain output attribution.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit-chain output attribution is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-ml-platform with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-072 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-073: tenant-isolated RAG retrieve
- Read: docs/runbooks/foundry/prompt-injection-fired.md
- Connects to: RAG retrieval isolation and context boundary
- Build or inspect: a minimal artifact that proves tenant-isolated RAG retrieve without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant-isolated RAG retrieve.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant-isolated RAG retrieve is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-foundry with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-073 contains file path, claim, evidence, rollback, and reviewer.

### Drill AI-074: model router policy deny
- Read: docs/runbooks/foundry-model-cutover.md
- Connects to: prompt fences and prompt-injection classifier capability
- Build or inspect: a minimal artifact that proves model router policy deny without widening beyond AI platform engineer, intelligence microservice.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for model router policy deny.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show model router policy deny is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row AI-074 contains file path, claim, evidence, rollback, and reviewer.

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
