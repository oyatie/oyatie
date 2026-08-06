---
doc_class: OnboardingGuide
role: "security engineer, platform security and tenant trust"
status: Published
date: 2026-05-20
owner: "council-security + ops-security"
related_oyatie_adrs:
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - microservices/messenger/decisions/ADR-MSG-001-mls-e2ee-key-delivery-architecture.md
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

# Security Engineer Month-One Onboarding

Audience: security engineer, platform security and tenant trust.
Industry precedent: AWS IAM policy lifecycle, Signal MLS key schedule discipline, Cloudflare rollback-first edge security, and Google BeyondCorp least-privilege access.

This guide is written for a programming-capable new joiner with no prior Oyatie architecture knowledge. Every phase names the repo files to read, the artifact to produce, the owner who reviews it, and the stop condition that proves the phase is complete.

Substance rule: do not treat this guide as orientation prose. Treat it as a work plan whose outputs can be inspected, replayed, or rejected.

You join security to make Cedar fragments, MLS key delivery, per-tenant secret custody, and threat modeling concrete enough that product teams cannot ship around them.
Month one is a sequence of security read paths, small policy and runbook contributions, and a threat-model project that proves you can reason across tenant, cell, policy, audit, identity, and regulated data boundaries.
Security work here is not a review stamp. It is a lifecycle owner role: define a gate, prove the gate in CI or runtime telemetry, define rollback, and bind evidence to the audit chain.

## Hyperscaler-Grade Reading Contract

- Named precedent: AWS IAM policy lifecycle, Signal MLS key schedule discipline, Cloudflare rollback-first edge security, and Google BeyondCorp least-privilege access.
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
1. docs/decisions/ADR-0702-identity-authz-live-apex.md
2. specs/cedar-fragment-schema.json
3. registry/cedar-fragments.json
4. docs/runbooks/cedar-fragment-emergency-rollback.md
5. docs/standards/cedar-policy-discipline.md
6. docs/standards/messenger-e2e-encryption-mls.md
7. microservices/messenger/decisions/ADR-MSG-001-mls-e2ee-key-delivery-architecture.md
8. microservices/messenger/runbooks/e2e-encryption-key-rotation.md
9. docs/decisions/ADR-0702-identity-authz-live-apex.md
10. crates/oya-secrets-domain/src/lib.rs
11. crates/oya-secrets-file-adapter/tests/file_secret_store.rs
12. docs/runbooks/per-cell-hsm-rotation.md
13. docs/runbooks/security-incident-response.md
14. docs/runbooks/provider-credential-leak-response.md
15. docs/standards/privacy-review.md


### Named ADRs to read

- ADR-0294 Cedar fragment soak and anomaly rollback
- ADR-0243 Cedar universal gate
- ADR-0043 OpenBao and HSM per cell
- ADR-MSG-001 MLS E2EE key delivery architecture
- ADR-0295 bootstrap CI SPIFFE kill switch
- ADR-0008 data use boundary

### Named playgrounds

1. crates/oya-policy-cedar-api/tests/cedar_policy_publish_api.rs
   - Artifact: write a four-sentence note explaining what this playground proves for security engineer, platform security and tenant trust.
2. crates/oya-secrets-domain/tests/secret_vault.rs
   - Artifact: write a four-sentence note explaining what this playground proves for security engineer, platform security and tenant trust.
3. microservices/messenger/runbooks/e2e-encryption-key-rotation.md
   - Artifact: write a four-sentence note explaining what this playground proves for security engineer, platform security and tenant trust.
4. docs/runbooks/cedar-policy-breach.md
   - Artifact: write a four-sentence note explaining what this playground proves for security engineer, platform security and tenant trust.

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

1. SEC-STARTER-001 add a missing negative Cedar fragment fixture for default-deny coverage
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for SEC.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
2. SEC-STARTER-002 add an OpenBao path redaction check to a secret custody runbook
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for SEC.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
3. SEC-STARTER-003 extend an MLS key rotation runbook with a replay verification step
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for SEC.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
4. SEC-STARTER-004 add one STRIDE row to a microservice threat model with an audit event id
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for SEC.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.

### Mentor pairing protocol

Pair with a policy-engine mentor for Cedar lifecycle, a messenger security mentor for MLS, and an OpenBao owner for secret custody. Each pairing produces one threat, one mitigating control, one observable signal, and one rollback branch.
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

First independent project: Own `SEC-PROJ-001`: write a threat-model addendum for Cedar fragment hot-reload TOCTOU covering soak, anomaly detection, automatic revocation, audit emission, and per-cell rollback.

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

- Cedar fragment lifecycle threat model
- MLS key delivery and rotation evidence pass
- per-tenant OpenBao custody path review
- bootstrap CI SPIFFE kill-switch tabletop

### Key contacts in other teams

- axis-policy-engine
- axis-identity
- axis-messenger
- axis-secrets
- axis-audit-chain
- ops-sre-reliability
- ops-compliance

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
1. Cedar fragment states: proposed, signed, soaking, activated, revoked, sunset
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for Cedar fragment states: proposed, signed, soaking, activated, revoked, sunset.
2. fragment anomaly detector and revoker separation of duty
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for fragment anomaly detector and revoker separation of duty.
3. MLS RFC 9420 epoch handling and delivery guarantees
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for MLS RFC 9420 epoch handling and delivery guarantees.
4. per-tenant OpenBao paths and per-cell HSM partitions
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for per-tenant OpenBao paths and per-cell HSM partitions.
5. SPIFFE bootstrap identity and kill-switch semantics
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for SPIFFE bootstrap identity and kill-switch semantics.
6. tenant secret custody, rotation, and shredding
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for tenant secret custody, rotation, and shredding.
7. supply-chain signature verification and rollback
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for supply-chain signature verification and rollback.
8. STRIDE threat model for policy, identity, and audit chain
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for STRIDE threat model for policy, identity, and audit chain.
9. break-glass with evidence and dual control
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for break-glass with evidence and dual control.
10. regulated data constraints under KR, EU, CN, FedRAMP, and IL5/6 packs
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for regulated data constraints under KR, EU, CN, FedRAMP, and IL5/6 packs.
Domain expertise stop condition: you can answer what breaks if this topic is implemented incorrectly, who owns the rollback, and what evidence proves the system recovered.
- Verification for Month 2: deep-dive notes cover every listed topic and at least one note is reviewed by a cross-team owner
- Stop condition for Month 2: mentor and owner can point to the artifact without asking you to explain hidden context.

## Quarter 1: Ownership


### Named OKRs

- OKR-SEC-Q1-1: close one Cedar fragment lifecycle finding with tests and rollback evidence
- OKR-SEC-Q1-2: run one MLS key rotation tabletop with audit-chain evidence
- OKR-SEC-Q1-3: publish one threat model that names tenant, cell, secret, identity, and audit boundaries

### Named on-call rotation entry

Enter `security-incident-shadow` rotation after completing one tabletop; quarter-one target is one Cedar rollback drill and one per-cell HSM rotation shadow.

### Named team-OKR contribution

Contribute to `TEAM-OKR-SECURITY-2026Q2`: every policy and secret custody surface has lifecycle, telemetry, and rollback evidence before GA gates harden.

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

1. Approving a Cedar fragment without soak-phase anomaly behavior.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for security engineer, platform security and tenant trust.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
2. Treating OpenBao as a generic vault instead of per-tenant per-cell custody.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for security engineer, platform security and tenant trust.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
3. Rotating keys without an audit-chain event and replay proof.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for security engineer, platform security and tenant trust.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
4. Writing threat models that list assets but no attacker action.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for security engineer, platform security and tenant trust.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
5. Using MLS terminology loosely without epoch, credential, and delivery semantics.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for security engineer, platform security and tenant trust.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
6. Adding a break-glass path that cannot be revoked or reconstructed.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for security engineer, platform security and tenant trust.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
7. Conflating app authorization Cedar with Kubernetes admission policy.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for security engineer, platform security and tenant trust.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
8. Letting a security exception outlive its sunset and owner.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for security engineer, platform security and tenant trust.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.

## Cross-Team Collaboration Playbook

| Team | Handoff ritual | Minimum payload |
| --- |--- |--- |
| axis-policy-engine | Fragment lifecycle handoff | Attach fragment id, signature, soak window, anomaly thresholds, and rollback event. |
| axis-messenger | MLS handoff | Attach epoch, credential mode, delivery proof, and failed-recipient branch. |
| axis-secrets | Secret custody handoff | Attach OpenBao path, HSM partition, rotation cadence, and redaction proof. |
| axis-audit-chain | Security evidence handoff | Attach event name, retention class, seal id, and replay command. |
| ops-compliance | Regulatory security handoff | Attach control mapping, pack id, evidence owner, and review cadence. |
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
| soak window | Shadow-evaluation period before a Cedar fragment enforces. |
| anomaly revoker | Automation that revokes a fragment after detector evidence crosses policy thresholds. |
| MLS epoch | Messaging group key state version under RFC 9420 semantics. |
| OpenBao path | Per-tenant per-cell secret location with strict access and redaction expectations. |
| break-glass | Emergency privilege path that must emit evidence and have a revocation plan. |

## Escalation Channels

| Escalation | Use when | Owner |
| --- |--- |--- |
| mentor checkpoint | you can proceed locally but need review of reasoning or evidence | assigned mentor |
| axis owner | a file or policy belongs to another team | axis-policy-engine |
| council review | claim boundary, doctrine, compliance, or security interpretation changes | council-security + ops-security |
| SRE on-call | dev-cell, incident, or reliability path blocks verification | ops-sre-reliability |
| security review | credential, tenant isolation, policy, or regulated data risk appears | ops-security |

## Resources & References

- docs/standards/cedar-policy-discipline.md
- docs/standards/messenger-e2e-encryption-mls.md
- docs/runbooks/cedar-fragment-emergency-rollback.md
- docs/runbooks/security-incident-response.md
- docs/runbooks/per-cell-hsm-rotation.md
- docs/runbooks/provider-credential-leak-response.md
- docs/standards/privacy-review.md

Reference-reading protocol: open the resource, identify the authority section, write the one-sentence claim it supports, and record whether the resource is doctrine, spec, implementation, test, runbook, dashboard, or evidence.

## Role-Specific Drill Library

Use this ledger when you need extra practice or when a mentor asks for stronger evidence. Each drill is intentionally small but must end with a verifiable artifact.

### Drill SEC-001: Cedar hot-reload TOCTOU
- Read: docs/decisions/ADR-0702-identity-authz-live-apex.md
- Connects to: Cedar fragment states: proposed, signed, soaking, activated, revoked, sunset
- Build or inspect: a minimal artifact that proves Cedar hot-reload TOCTOU without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar hot-reload TOCTOU.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar hot-reload TOCTOU is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-001 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-002: fragment soak anomaly
- Read: specs/cedar-fragment-schema.json
- Connects to: fragment anomaly detector and revoker separation of duty
- Build or inspect: a minimal artifact that proves fragment soak anomaly without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for fragment soak anomaly.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show fragment soak anomaly is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-identity with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-002 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-003: default-deny escape attempt
- Read: registry/cedar-fragments.json
- Connects to: MLS RFC 9420 epoch handling and delivery guarantees
- Build or inspect: a minimal artifact that proves default-deny escape attempt without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for default-deny escape attempt.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show default-deny escape attempt is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-messenger with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-003 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-004: MLS epoch mismatch
- Read: docs/runbooks/cedar-fragment-emergency-rollback.md
- Connects to: per-tenant OpenBao paths and per-cell HSM partitions
- Build or inspect: a minimal artifact that proves MLS epoch mismatch without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for MLS epoch mismatch.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show MLS epoch mismatch is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-secrets with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-004 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-005: MLS recipient compromise
- Read: docs/standards/cedar-policy-discipline.md
- Connects to: SPIFFE bootstrap identity and kill-switch semantics
- Build or inspect: a minimal artifact that proves MLS recipient compromise without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for MLS recipient compromise.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show MLS recipient compromise is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-005 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-006: OpenBao path leak
- Read: docs/standards/messenger-e2e-encryption-mls.md
- Connects to: tenant secret custody, rotation, and shredding
- Build or inspect: a minimal artifact that proves OpenBao path leak without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for OpenBao path leak.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show OpenBao path leak is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-006 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-007: per-cell HSM rotation
- Read: microservices/messenger/decisions/ADR-MSG-001-mls-e2ee-key-delivery-architecture.md
- Connects to: supply-chain signature verification and rollback
- Build or inspect: a minimal artifact that proves per-cell HSM rotation without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for per-cell HSM rotation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show per-cell HSM rotation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-007 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-008: SPIFFE kill switch
- Read: microservices/messenger/runbooks/e2e-encryption-key-rotation.md
- Connects to: STRIDE threat model for policy, identity, and audit chain
- Build or inspect: a minimal artifact that proves SPIFFE kill switch without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for SPIFFE kill switch.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show SPIFFE kill switch is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-008 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-009: bootstrap CI compromise
- Read: docs/decisions/ADR-0702-identity-authz-live-apex.md
- Connects to: break-glass with evidence and dual control
- Build or inspect: a minimal artifact that proves bootstrap CI compromise without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for bootstrap CI compromise.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show bootstrap CI compromise is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-identity with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-009 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-010: provider credential leak
- Read: crates/oya-secrets-domain/src/lib.rs
- Connects to: regulated data constraints under KR, EU, CN, FedRAMP, and IL5/6 packs
- Build or inspect: a minimal artifact that proves provider credential leak without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for provider credential leak.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show provider credential leak is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-messenger with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-010 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-011: break-glass evidence
- Read: crates/oya-secrets-file-adapter/tests/file_secret_store.rs
- Connects to: Cedar fragment states: proposed, signed, soaking, activated, revoked, sunset
- Build or inspect: a minimal artifact that proves break-glass evidence without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for break-glass evidence.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show break-glass evidence is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-secrets with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-011 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-012: policy revocation race
- Read: docs/runbooks/per-cell-hsm-rotation.md
- Connects to: fragment anomaly detector and revoker separation of duty
- Build or inspect: a minimal artifact that proves policy revocation race without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for policy revocation race.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show policy revocation race is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-012 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-013: tenant secret custody audit
- Read: docs/runbooks/security-incident-response.md
- Connects to: MLS RFC 9420 epoch handling and delivery guarantees
- Build or inspect: a minimal artifact that proves tenant secret custody audit without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant secret custody audit.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant secret custody audit is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-013 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-014: supply-chain signature failure
- Read: docs/runbooks/provider-credential-leak-response.md
- Connects to: per-tenant OpenBao paths and per-cell HSM partitions
- Build or inspect: a minimal artifact that proves supply-chain signature failure without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for supply-chain signature failure.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show supply-chain signature failure is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-014 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-015: redaction boundary test
- Read: docs/standards/privacy-review.md
- Connects to: SPIFFE bootstrap identity and kill-switch semantics
- Build or inspect: a minimal artifact that proves redaction boundary test without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for redaction boundary test.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show redaction boundary test is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-015 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-016: security incident tabletop
- Read: docs/decisions/ADR-0702-identity-authz-live-apex.md
- Connects to: tenant secret custody, rotation, and shredding
- Build or inspect: a minimal artifact that proves security incident tabletop without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for security incident tabletop.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show security incident tabletop is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-identity with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-016 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-017: threat model STRIDE row
- Read: specs/cedar-fragment-schema.json
- Connects to: supply-chain signature verification and rollback
- Build or inspect: a minimal artifact that proves threat model STRIDE row without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for threat model STRIDE row.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show threat model STRIDE row is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-messenger with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-017 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-018: sovereign pack key constraint
- Read: registry/cedar-fragments.json
- Connects to: STRIDE threat model for policy, identity, and audit chain
- Build or inspect: a minimal artifact that proves sovereign pack key constraint without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for sovereign pack key constraint.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show sovereign pack key constraint is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-secrets with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-018 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-019: Cedar hot-reload TOCTOU
- Read: docs/runbooks/cedar-fragment-emergency-rollback.md
- Connects to: break-glass with evidence and dual control
- Build or inspect: a minimal artifact that proves Cedar hot-reload TOCTOU without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar hot-reload TOCTOU.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar hot-reload TOCTOU is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-019 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-020: fragment soak anomaly
- Read: docs/standards/cedar-policy-discipline.md
- Connects to: regulated data constraints under KR, EU, CN, FedRAMP, and IL5/6 packs
- Build or inspect: a minimal artifact that proves fragment soak anomaly without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for fragment soak anomaly.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show fragment soak anomaly is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-020 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-021: default-deny escape attempt
- Read: docs/standards/messenger-e2e-encryption-mls.md
- Connects to: Cedar fragment states: proposed, signed, soaking, activated, revoked, sunset
- Build or inspect: a minimal artifact that proves default-deny escape attempt without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for default-deny escape attempt.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show default-deny escape attempt is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-021 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-022: MLS epoch mismatch
- Read: microservices/messenger/decisions/ADR-MSG-001-mls-e2ee-key-delivery-architecture.md
- Connects to: fragment anomaly detector and revoker separation of duty
- Build or inspect: a minimal artifact that proves MLS epoch mismatch without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for MLS epoch mismatch.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show MLS epoch mismatch is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-022 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-023: MLS recipient compromise
- Read: microservices/messenger/runbooks/e2e-encryption-key-rotation.md
- Connects to: MLS RFC 9420 epoch handling and delivery guarantees
- Build or inspect: a minimal artifact that proves MLS recipient compromise without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for MLS recipient compromise.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show MLS recipient compromise is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-identity with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-023 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-024: OpenBao path leak
- Read: docs/decisions/ADR-0702-identity-authz-live-apex.md
- Connects to: per-tenant OpenBao paths and per-cell HSM partitions
- Build or inspect: a minimal artifact that proves OpenBao path leak without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for OpenBao path leak.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show OpenBao path leak is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-messenger with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-024 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-025: per-cell HSM rotation
- Read: crates/oya-secrets-domain/src/lib.rs
- Connects to: SPIFFE bootstrap identity and kill-switch semantics
- Build or inspect: a minimal artifact that proves per-cell HSM rotation without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for per-cell HSM rotation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show per-cell HSM rotation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-secrets with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-025 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-026: SPIFFE kill switch
- Read: crates/oya-secrets-file-adapter/tests/file_secret_store.rs
- Connects to: tenant secret custody, rotation, and shredding
- Build or inspect: a minimal artifact that proves SPIFFE kill switch without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for SPIFFE kill switch.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show SPIFFE kill switch is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-026 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-027: bootstrap CI compromise
- Read: docs/runbooks/per-cell-hsm-rotation.md
- Connects to: supply-chain signature verification and rollback
- Build or inspect: a minimal artifact that proves bootstrap CI compromise without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for bootstrap CI compromise.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show bootstrap CI compromise is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-027 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-028: provider credential leak
- Read: docs/runbooks/security-incident-response.md
- Connects to: STRIDE threat model for policy, identity, and audit chain
- Build or inspect: a minimal artifact that proves provider credential leak without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for provider credential leak.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show provider credential leak is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-028 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-029: break-glass evidence
- Read: docs/runbooks/provider-credential-leak-response.md
- Connects to: break-glass with evidence and dual control
- Build or inspect: a minimal artifact that proves break-glass evidence without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for break-glass evidence.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show break-glass evidence is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-029 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-030: policy revocation race
- Read: docs/standards/privacy-review.md
- Connects to: regulated data constraints under KR, EU, CN, FedRAMP, and IL5/6 packs
- Build or inspect: a minimal artifact that proves policy revocation race without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for policy revocation race.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show policy revocation race is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-identity with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-030 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-031: tenant secret custody audit
- Read: docs/decisions/ADR-0702-identity-authz-live-apex.md
- Connects to: Cedar fragment states: proposed, signed, soaking, activated, revoked, sunset
- Build or inspect: a minimal artifact that proves tenant secret custody audit without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant secret custody audit.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant secret custody audit is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-messenger with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-031 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-032: supply-chain signature failure
- Read: specs/cedar-fragment-schema.json
- Connects to: fragment anomaly detector and revoker separation of duty
- Build or inspect: a minimal artifact that proves supply-chain signature failure without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for supply-chain signature failure.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show supply-chain signature failure is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-secrets with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-032 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-033: redaction boundary test
- Read: registry/cedar-fragments.json
- Connects to: MLS RFC 9420 epoch handling and delivery guarantees
- Build or inspect: a minimal artifact that proves redaction boundary test without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for redaction boundary test.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show redaction boundary test is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-033 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-034: security incident tabletop
- Read: docs/runbooks/cedar-fragment-emergency-rollback.md
- Connects to: per-tenant OpenBao paths and per-cell HSM partitions
- Build or inspect: a minimal artifact that proves security incident tabletop without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for security incident tabletop.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show security incident tabletop is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-034 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-035: threat model STRIDE row
- Read: docs/standards/cedar-policy-discipline.md
- Connects to: SPIFFE bootstrap identity and kill-switch semantics
- Build or inspect: a minimal artifact that proves threat model STRIDE row without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for threat model STRIDE row.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show threat model STRIDE row is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-035 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-036: sovereign pack key constraint
- Read: docs/standards/messenger-e2e-encryption-mls.md
- Connects to: tenant secret custody, rotation, and shredding
- Build or inspect: a minimal artifact that proves sovereign pack key constraint without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for sovereign pack key constraint.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show sovereign pack key constraint is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-036 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-037: Cedar hot-reload TOCTOU
- Read: microservices/messenger/decisions/ADR-MSG-001-mls-e2ee-key-delivery-architecture.md
- Connects to: supply-chain signature verification and rollback
- Build or inspect: a minimal artifact that proves Cedar hot-reload TOCTOU without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar hot-reload TOCTOU.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar hot-reload TOCTOU is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-identity with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-037 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-038: fragment soak anomaly
- Read: microservices/messenger/runbooks/e2e-encryption-key-rotation.md
- Connects to: STRIDE threat model for policy, identity, and audit chain
- Build or inspect: a minimal artifact that proves fragment soak anomaly without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for fragment soak anomaly.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show fragment soak anomaly is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-messenger with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-038 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-039: default-deny escape attempt
- Read: docs/decisions/ADR-0702-identity-authz-live-apex.md
- Connects to: break-glass with evidence and dual control
- Build or inspect: a minimal artifact that proves default-deny escape attempt without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for default-deny escape attempt.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show default-deny escape attempt is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-secrets with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-039 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-040: MLS epoch mismatch
- Read: crates/oya-secrets-domain/src/lib.rs
- Connects to: regulated data constraints under KR, EU, CN, FedRAMP, and IL5/6 packs
- Build or inspect: a minimal artifact that proves MLS epoch mismatch without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for MLS epoch mismatch.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show MLS epoch mismatch is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-040 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-041: MLS recipient compromise
- Read: crates/oya-secrets-file-adapter/tests/file_secret_store.rs
- Connects to: Cedar fragment states: proposed, signed, soaking, activated, revoked, sunset
- Build or inspect: a minimal artifact that proves MLS recipient compromise without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for MLS recipient compromise.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show MLS recipient compromise is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-041 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-042: OpenBao path leak
- Read: docs/runbooks/per-cell-hsm-rotation.md
- Connects to: fragment anomaly detector and revoker separation of duty
- Build or inspect: a minimal artifact that proves OpenBao path leak without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for OpenBao path leak.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show OpenBao path leak is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-042 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-043: per-cell HSM rotation
- Read: docs/runbooks/security-incident-response.md
- Connects to: MLS RFC 9420 epoch handling and delivery guarantees
- Build or inspect: a minimal artifact that proves per-cell HSM rotation without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for per-cell HSM rotation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show per-cell HSM rotation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-043 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-044: SPIFFE kill switch
- Read: docs/runbooks/provider-credential-leak-response.md
- Connects to: per-tenant OpenBao paths and per-cell HSM partitions
- Build or inspect: a minimal artifact that proves SPIFFE kill switch without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for SPIFFE kill switch.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show SPIFFE kill switch is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-identity with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-044 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-045: bootstrap CI compromise
- Read: docs/standards/privacy-review.md
- Connects to: SPIFFE bootstrap identity and kill-switch semantics
- Build or inspect: a minimal artifact that proves bootstrap CI compromise without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for bootstrap CI compromise.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show bootstrap CI compromise is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-messenger with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-045 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-046: provider credential leak
- Read: docs/decisions/ADR-0702-identity-authz-live-apex.md
- Connects to: tenant secret custody, rotation, and shredding
- Build or inspect: a minimal artifact that proves provider credential leak without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for provider credential leak.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show provider credential leak is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-secrets with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-046 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-047: break-glass evidence
- Read: specs/cedar-fragment-schema.json
- Connects to: supply-chain signature verification and rollback
- Build or inspect: a minimal artifact that proves break-glass evidence without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for break-glass evidence.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show break-glass evidence is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-047 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-048: policy revocation race
- Read: registry/cedar-fragments.json
- Connects to: STRIDE threat model for policy, identity, and audit chain
- Build or inspect: a minimal artifact that proves policy revocation race without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for policy revocation race.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show policy revocation race is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-048 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-049: tenant secret custody audit
- Read: docs/runbooks/cedar-fragment-emergency-rollback.md
- Connects to: break-glass with evidence and dual control
- Build or inspect: a minimal artifact that proves tenant secret custody audit without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant secret custody audit.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant secret custody audit is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-049 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-050: supply-chain signature failure
- Read: docs/standards/cedar-policy-discipline.md
- Connects to: regulated data constraints under KR, EU, CN, FedRAMP, and IL5/6 packs
- Build or inspect: a minimal artifact that proves supply-chain signature failure without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for supply-chain signature failure.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show supply-chain signature failure is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-050 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-051: redaction boundary test
- Read: docs/standards/messenger-e2e-encryption-mls.md
- Connects to: Cedar fragment states: proposed, signed, soaking, activated, revoked, sunset
- Build or inspect: a minimal artifact that proves redaction boundary test without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for redaction boundary test.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show redaction boundary test is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-identity with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-051 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-052: security incident tabletop
- Read: microservices/messenger/decisions/ADR-MSG-001-mls-e2ee-key-delivery-architecture.md
- Connects to: fragment anomaly detector and revoker separation of duty
- Build or inspect: a minimal artifact that proves security incident tabletop without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for security incident tabletop.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show security incident tabletop is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-messenger with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-052 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-053: threat model STRIDE row
- Read: microservices/messenger/runbooks/e2e-encryption-key-rotation.md
- Connects to: MLS RFC 9420 epoch handling and delivery guarantees
- Build or inspect: a minimal artifact that proves threat model STRIDE row without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for threat model STRIDE row.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show threat model STRIDE row is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-secrets with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-053 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-054: sovereign pack key constraint
- Read: docs/decisions/ADR-0702-identity-authz-live-apex.md
- Connects to: per-tenant OpenBao paths and per-cell HSM partitions
- Build or inspect: a minimal artifact that proves sovereign pack key constraint without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for sovereign pack key constraint.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show sovereign pack key constraint is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-054 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-055: Cedar hot-reload TOCTOU
- Read: crates/oya-secrets-domain/src/lib.rs
- Connects to: SPIFFE bootstrap identity and kill-switch semantics
- Build or inspect: a minimal artifact that proves Cedar hot-reload TOCTOU without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar hot-reload TOCTOU.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar hot-reload TOCTOU is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-055 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-056: fragment soak anomaly
- Read: crates/oya-secrets-file-adapter/tests/file_secret_store.rs
- Connects to: tenant secret custody, rotation, and shredding
- Build or inspect: a minimal artifact that proves fragment soak anomaly without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for fragment soak anomaly.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show fragment soak anomaly is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-056 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-057: default-deny escape attempt
- Read: docs/runbooks/per-cell-hsm-rotation.md
- Connects to: supply-chain signature verification and rollback
- Build or inspect: a minimal artifact that proves default-deny escape attempt without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for default-deny escape attempt.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show default-deny escape attempt is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-057 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-058: MLS epoch mismatch
- Read: docs/runbooks/security-incident-response.md
- Connects to: STRIDE threat model for policy, identity, and audit chain
- Build or inspect: a minimal artifact that proves MLS epoch mismatch without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for MLS epoch mismatch.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show MLS epoch mismatch is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-identity with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-058 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-059: MLS recipient compromise
- Read: docs/runbooks/provider-credential-leak-response.md
- Connects to: break-glass with evidence and dual control
- Build or inspect: a minimal artifact that proves MLS recipient compromise without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for MLS recipient compromise.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show MLS recipient compromise is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-messenger with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-059 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-060: OpenBao path leak
- Read: docs/standards/privacy-review.md
- Connects to: regulated data constraints under KR, EU, CN, FedRAMP, and IL5/6 packs
- Build or inspect: a minimal artifact that proves OpenBao path leak without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for OpenBao path leak.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show OpenBao path leak is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-secrets with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-060 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-061: per-cell HSM rotation
- Read: docs/decisions/ADR-0702-identity-authz-live-apex.md
- Connects to: Cedar fragment states: proposed, signed, soaking, activated, revoked, sunset
- Build or inspect: a minimal artifact that proves per-cell HSM rotation without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for per-cell HSM rotation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show per-cell HSM rotation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-061 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-062: SPIFFE kill switch
- Read: specs/cedar-fragment-schema.json
- Connects to: fragment anomaly detector and revoker separation of duty
- Build or inspect: a minimal artifact that proves SPIFFE kill switch without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for SPIFFE kill switch.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show SPIFFE kill switch is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-062 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-063: bootstrap CI compromise
- Read: registry/cedar-fragments.json
- Connects to: MLS RFC 9420 epoch handling and delivery guarantees
- Build or inspect: a minimal artifact that proves bootstrap CI compromise without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for bootstrap CI compromise.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show bootstrap CI compromise is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-063 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-064: provider credential leak
- Read: docs/runbooks/cedar-fragment-emergency-rollback.md
- Connects to: per-tenant OpenBao paths and per-cell HSM partitions
- Build or inspect: a minimal artifact that proves provider credential leak without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for provider credential leak.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show provider credential leak is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-064 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-065: break-glass evidence
- Read: docs/standards/cedar-policy-discipline.md
- Connects to: SPIFFE bootstrap identity and kill-switch semantics
- Build or inspect: a minimal artifact that proves break-glass evidence without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for break-glass evidence.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show break-glass evidence is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-identity with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-065 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-066: policy revocation race
- Read: docs/standards/messenger-e2e-encryption-mls.md
- Connects to: tenant secret custody, rotation, and shredding
- Build or inspect: a minimal artifact that proves policy revocation race without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for policy revocation race.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show policy revocation race is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-messenger with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-066 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-067: tenant secret custody audit
- Read: microservices/messenger/decisions/ADR-MSG-001-mls-e2ee-key-delivery-architecture.md
- Connects to: supply-chain signature verification and rollback
- Build or inspect: a minimal artifact that proves tenant secret custody audit without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant secret custody audit.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant secret custody audit is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-secrets with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-067 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-068: supply-chain signature failure
- Read: microservices/messenger/runbooks/e2e-encryption-key-rotation.md
- Connects to: STRIDE threat model for policy, identity, and audit chain
- Build or inspect: a minimal artifact that proves supply-chain signature failure without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for supply-chain signature failure.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show supply-chain signature failure is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-068 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-069: redaction boundary test
- Read: docs/decisions/ADR-0702-identity-authz-live-apex.md
- Connects to: break-glass with evidence and dual control
- Build or inspect: a minimal artifact that proves redaction boundary test without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for redaction boundary test.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show redaction boundary test is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-069 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-070: security incident tabletop
- Read: crates/oya-secrets-domain/src/lib.rs
- Connects to: regulated data constraints under KR, EU, CN, FedRAMP, and IL5/6 packs
- Build or inspect: a minimal artifact that proves security incident tabletop without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for security incident tabletop.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show security incident tabletop is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-compliance with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-070 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-071: threat model STRIDE row
- Read: crates/oya-secrets-file-adapter/tests/file_secret_store.rs
- Connects to: Cedar fragment states: proposed, signed, soaking, activated, revoked, sunset
- Build or inspect: a minimal artifact that proves threat model STRIDE row without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for threat model STRIDE row.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show threat model STRIDE row is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-071 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-072: sovereign pack key constraint
- Read: docs/runbooks/per-cell-hsm-rotation.md
- Connects to: fragment anomaly detector and revoker separation of duty
- Build or inspect: a minimal artifact that proves sovereign pack key constraint without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for sovereign pack key constraint.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show sovereign pack key constraint is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-identity with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-072 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-073: Cedar hot-reload TOCTOU
- Read: docs/runbooks/security-incident-response.md
- Connects to: MLS RFC 9420 epoch handling and delivery guarantees
- Build or inspect: a minimal artifact that proves Cedar hot-reload TOCTOU without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar hot-reload TOCTOU.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar hot-reload TOCTOU is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-messenger with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-073 contains file path, claim, evidence, rollback, and reviewer.

### Drill SEC-074: fragment soak anomaly
- Read: docs/runbooks/provider-credential-leak-response.md
- Connects to: per-tenant OpenBao paths and per-cell HSM partitions
- Build or inspect: a minimal artifact that proves fragment soak anomaly without widening beyond security engineer, platform security and tenant trust.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for fragment soak anomaly.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show fragment soak anomaly is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-secrets with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SEC-074 contains file path, claim, evidence, rollback, and reviewer.

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
