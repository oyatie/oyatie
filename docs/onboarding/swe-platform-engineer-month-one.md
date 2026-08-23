---
doc_class: OnboardingGuide
role: "backend Rust SWE, platform substrate team"
status: Published
date: 2026-05-20
owner: "axis-platform-substrate + council-architecture"
related_oyatie_adrs:
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
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
  - governance-doc-rigor
  - governance-doc-graph-6hops
---

# SWE Platform Engineer Month-One Onboarding

Audience: backend Rust SWE, platform substrate team.
Industry precedent: Palantir Foundry ontology projection plus AWS cell-based substrate ownership and Stripe-style append-only audit evidence.

This guide is written for a programming-capable new joiner with no prior Oyatie architecture knowledge. Every phase names the repo files to read, the artifact to produce, the owner who reviews it, and the stop condition that proves the phase is complete.

Substance rule: do not treat this guide as orientation prose. Treat it as a work plan whose outputs can be inspected, replayed, or rejected.

You join the backend Rust platform substrate team as an owner of the code paths that make every product surface tenant-scoped, policy-gated, audit-emitting, and ontology-projectable.
The first month is not feature-tourism. It is a controlled path from environment bring-up to a small Rust contribution that proves you can move inside the layer enum, Cedar policy gates, audit-chain evidence, and ontology projection without creating a cross-layer shortcut.
The local shorthand in older material says 12-layer enum. ADR-0105 amended that vocabulary to a 13-value enum by adopting `api`; your onboarding task is to know the original 12-layer intent and the ADR-0105 amendment before naming or moving crates.

## Hyperscaler-Grade Reading Contract

- Named precedent: Palantir Foundry ontology projection plus AWS cell-based substrate ownership and Stripe-style append-only audit evidence.
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
1. specs/root-hub-pointers.json
2. specs/masterplan.json
3. docs/standards/layer-enum-adr-0105.md
4. specs/crate-naming-audit.json
5. Cargo.toml
6. crates/policy-cedar-domain/src/lib.rs
7. crates/policy-cedar-api/tests/cedar_policy_publish_api.rs
8. crates/audit-chain-domain/src/lib.rs
9. crates/audit-chain-domain/tests/merkle_chain.rs
10. crates/audit-chain-usecase/tests/audit_event_emit.rs
11. crates/ontology-kernel/src/lib.rs
12. crates/ontology-api/tests/object_graph_entity_upsert_api.rs
13. registry/catalog/policy-cedar-domain.yaml
14. registry/catalog/audit-chain-domain.yaml
15. registry/catalog/ontology-kernel.yaml


### Named ADRs to read

- ADR-0105 layer enum amendment
- ADR-0243 Cedar as universal gate
- ADR-0003 audit chain and evidence emission
- ADR-0006 ontology typed entity layer
- ADR-0257 library-first ontology read path
- ADR-0263 observability emission contract

### Named playgrounds

1. crates/policy-cedar-api/tests/cedar_policy_publish_api.rs
   - Artifact: write a four-sentence note explaining what this playground proves for backend Rust SWE, platform substrate team.
2. crates/audit-chain-domain/tests/merkle_chain.rs
   - Artifact: write a four-sentence note explaining what this playground proves for backend Rust SWE, platform substrate team.
3. crates/ontology-kernel/tests/types.rs
   - Artifact: write a four-sentence note explaining what this playground proves for backend Rust SWE, platform substrate team.
4. registry/catalog/check-ontology-projection-coverage.yaml
   - Artifact: write a four-sentence note explaining what this playground proves for backend Rust SWE, platform substrate team.

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

1. SWE-STARTER-001 add a missing audit-event assertion in `crates/audit-chain-usecase/tests/audit_event_emit.rs`
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for SWE.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
2. SWE-STARTER-002 extend one `registry/catalog/*.yaml` row with an explicit layer enum note
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for SWE.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
3. SWE-STARTER-003 add a Cedar default-deny fixture to `crates/policy-cedar-api/tests/cedar_policy_publish_api.rs`
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for SWE.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.
4. SWE-STARTER-004 document one stale 12-layer reference with the ADR-0105 13-value correction
   - Acceptance evidence: targeted test, screenshot, evidence file, runbook diff, or reviewer-approved trace for SWE.
   - Rollback evidence: name the exact file revert or policy rollback and the check that proves it worked.

### Mentor pairing protocol

Pair with a substrate mentor for the first Rust diff, a policy-engine reviewer for the Cedar gate, and an audit-chain reviewer for evidence semantics. Every pairing ends with a written note: changed file, invariant tested, rollback path, and remaining uncertainty.
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

First independent project: Own `SWE-PROJ-001`: add one tiny ontology projection coverage check that refuses a mutating action without an ontology object id and emits an audit-chain event on refusal.

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

- Cedar coverage gate fixture hardening
- audit-chain event schema conformance sweep
- ontology projection crate naming cleanup
- layer enum onboarding lint for stale 12-layer language

### Key contacts in other teams

- axis-policy-engine
- axis-audit-chain
- axis-ontology
- axis-tenancy
- ops-sre-reliability
- council-architecture

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
1. 13-value layer enum and why `api` is protocol-neutral
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for 13-value layer enum and why `api` is protocol-neutral.
2. Cedar fragment lifecycle from proposal to signed activation
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for Cedar fragment lifecycle from proposal to signed activation.
3. audit-chain Merkle seal, Ed25519 signature, and replay verification
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for audit-chain Merkle seal, Ed25519 signature, and replay verification.
4. ontology projection for workflow and product surfaces
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for ontology projection for workflow and product surfaces.
5. per-tenant scoping in Rust domain types
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for per-tenant scoping in Rust domain types.
6. library-first read path versus service fan-out
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for library-first read path versus service fan-out.
7. registry catalog row shape and ownership metadata
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for registry catalog row shape and ownership metadata.
8. versioning and deprecation for substrate crates
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for versioning and deprecation for substrate crates.
9. multi-region audit event replication behavior
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for multi-region audit event replication behavior.
10. sovereign-cell constraints for regulated tenants
   - Study artifact: one note that names precedent, failure modes, capacity or timing boundary, observability hook, rollback, multi-region behavior, sovereign-cell behavior, and versioning impact for sovereign-cell constraints for regulated tenants.
Domain expertise stop condition: you can answer what breaks if this topic is implemented incorrectly, who owns the rollback, and what evidence proves the system recovered.
- Verification for Month 2: deep-dive notes cover every listed topic and at least one note is reviewed by a cross-team owner
- Stop condition for Month 2: mentor and owner can point to the artifact without asking you to explain hidden context.

## Quarter 1: Ownership


### Named OKRs

- OKR-SWE-Q1-1: ship one Rust substrate PR with tests and no cross-layer dependency regression
- OKR-SWE-Q1-2: close one audit-chain evidence gap without weakening event semantics
- OKR-SWE-Q1-3: publish one internal walkthrough explaining Cedar plus ontology projection for a real action

### Named on-call rotation entry

Enter `platform-substrate-shadow` rotation after the month-one project merges; quarter-one target is one shadowed incident review and one audit-chain replay exercise.

### Named team-OKR contribution

Contribute to `TEAM-OKR-SUBSTRATE-2026Q2`: no unowned policy, audit, or ontology substrate primitives in promoted changes.

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

1. Treating the layer enum as naming trivia instead of dependency direction.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for backend Rust SWE, platform substrate team.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
2. Calling Cedar from the wrong layer and hiding policy decisions in application code.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for backend Rust SWE, platform substrate team.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
3. Emitting an audit event after mutation rather than making audit emission part of the transaction boundary.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for backend Rust SWE, platform substrate team.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
4. Using ontology ids as display labels instead of stable typed projections.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for backend Rust SWE, platform substrate team.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
5. Referencing a compatibility registry row as if it were the canonical spec.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for backend Rust SWE, platform substrate team.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
6. Adding a Rust dependency before checking existing substrate crates.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for backend Rust SWE, platform substrate team.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
7. Writing tests that only assert success and never assert default-deny behavior.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for backend Rust SWE, platform substrate team.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.
8. Skipping multi-region behavior because the local dev cell is single-region.
   - Detection: ask whether the artifact has owner, evidence, rollback, and cross-team impact for backend Rust SWE, platform substrate team.
   - Recovery: narrow the claim, add the missing evidence, and route the handoff to the owning team before proceeding.

## Cross-Team Collaboration Playbook

| Team | Handoff ritual | Minimum payload |
| --- |--- |--- |
| axis-policy-engine | Cedar handoff | Attach permit fragment id, default-deny proof, signature state, and soak expectation. |
| axis-audit-chain | Evidence handoff | Attach event id, payload schema, seal position, replay command, and retention class. |
| axis-ontology | Projection handoff | Attach object type, action id, schema revision, and reverse dependency list. |
| ops-sre-reliability | Runbook handoff | Attach failure tree, rollback command, and dashboard pointer. |
| ops-compliance | Regulated data handoff | Attach data class, residency pack, DSR effect, and audit evidence id. |
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
| 13-value layer enum | ADR-0105 amendment that adds `api` to the prior 12-value clean-architecture enum. |
| Cedar universal gate | The doctrine that policy lives in versioned Cedar fragments and code asks the policy engine. |
| audit-chain seal | Append-only evidence record that can be replayed and verified. |
| ontology projection | Typed projection from domain action to stable object and relationship ids. |
| substrate crate | Shared low-level capability that product surfaces consume without owning. |

## Escalation Channels

| Escalation | Use when | Owner |
| --- |--- |--- |
| mentor checkpoint | you can proceed locally but need review of reasoning or evidence | assigned mentor |
| axis owner | a file or policy belongs to another team | axis-policy-engine |
| council review | claim boundary, doctrine, compliance, or security interpretation changes | axis-platform-substrate + council-architecture |
| SRE on-call | dev-cell, incident, or reliability path blocks verification | ops-sre-reliability |
| security review | credential, tenant isolation, policy, or regulated data risk appears | ops-security |

## Resources & References

- docs/standards/documentation-rigor.md
- docs/standards/layer-enum-adr-0105.md
- docs/standards/cedar-policy-discipline.md
- docs/standards/ontology-projection-substrate.md
- docs/runbooks/audit-chain-integrity-check.md
- specs/cedar-fragment-schema.json
- specs/microservices/ontology.json

Reference-reading protocol: open the resource, identify the authority section, write the one-sentence claim it supports, and record whether the resource is doctrine, spec, implementation, test, runbook, dashboard, or evidence.

## Role-Specific Drill Library

Use this ledger when you need extra practice or when a mentor asks for stronger evidence. Each drill is intentionally small but must end with a verifiable artifact.

### Drill SWE-001: layer enum crate classification
- Read: specs/root-hub-pointers.json
- Connects to: 13-value layer enum and why `api` is protocol-neutral
- Build or inspect: a minimal artifact that proves layer enum crate classification without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for layer enum crate classification.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show layer enum crate classification is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-001 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-002: Cedar publish default-deny
- Read: specs/masterplan.json
- Connects to: Cedar fragment lifecycle from proposal to signed activation
- Build or inspect: a minimal artifact that proves Cedar publish default-deny without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar publish default-deny.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar publish default-deny is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-002 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-003: audit event replay
- Read: docs/standards/layer-enum-adr-0105.md
- Connects to: audit-chain Merkle seal, Ed25519 signature, and replay verification
- Build or inspect: a minimal artifact that proves audit event replay without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit event replay.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit event replay is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-003 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-004: ontology action projection
- Read: specs/crate-naming-audit.json
- Connects to: ontology projection for workflow and product surfaces
- Build or inspect: a minimal artifact that proves ontology action projection without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ontology action projection.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ontology action projection is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-004 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-005: tenant_id type boundary
- Read: Cargo.toml
- Connects to: per-tenant scoping in Rust domain types
- Build or inspect: a minimal artifact that proves tenant_id type boundary without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant_id type boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant_id type boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-005 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-006: OpenAPI api crate boundary
- Read: crates/policy-cedar-domain/src/lib.rs
- Connects to: library-first read path versus service fan-out
- Build or inspect: a minimal artifact that proves OpenAPI api crate boundary without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for OpenAPI api crate boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show OpenAPI api crate boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-architecture with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-006 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-007: catalog row ownership
- Read: crates/policy-cedar-api/tests/cedar_policy_publish_api.rs
- Connects to: registry catalog row shape and ownership metadata
- Build or inspect: a minimal artifact that proves catalog row ownership without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for catalog row ownership.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show catalog row ownership is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-007 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-008: registry reverse dependency lookup
- Read: crates/audit-chain-domain/src/lib.rs
- Connects to: versioning and deprecation for substrate crates
- Build or inspect: a minimal artifact that proves registry reverse dependency lookup without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for registry reverse dependency lookup.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show registry reverse dependency lookup is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-008 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-009: multi-region audit replication
- Read: crates/audit-chain-domain/tests/merkle_chain.rs
- Connects to: multi-region audit event replication behavior
- Build or inspect: a minimal artifact that proves multi-region audit replication without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for multi-region audit replication.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show multi-region audit replication is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-009 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-010: sovereign-cell data class handling
- Read: crates/audit-chain-usecase/tests/audit_event_emit.rs
- Connects to: sovereign-cell constraints for regulated tenants
- Build or inspect: a minimal artifact that proves sovereign-cell data class handling without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for sovereign-cell data class handling.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show sovereign-cell data class handling is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-010 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-011: Rust property test fixture
- Read: crates/ontology-kernel/src/lib.rs
- Connects to: 13-value layer enum and why `api` is protocol-neutral
- Build or inspect: a minimal artifact that proves Rust property test fixture without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Rust property test fixture.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Rust property test fixture is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-011 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-012: Merklized evidence chain
- Read: crates/ontology-api/tests/object_graph_entity_upsert_api.rs
- Connects to: Cedar fragment lifecycle from proposal to signed activation
- Build or inspect: a minimal artifact that proves Merklized evidence chain without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Merklized evidence chain.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Merklized evidence chain is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-architecture with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-012 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-013: Cedar soak rollback
- Read: registry/catalog/policy-cedar-domain.yaml
- Connects to: audit-chain Merkle seal, Ed25519 signature, and replay verification
- Build or inspect: a minimal artifact that proves Cedar soak rollback without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar soak rollback.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar soak rollback is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-013 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-014: ontology version deprecation
- Read: registry/catalog/audit-chain-domain.yaml
- Connects to: ontology projection for workflow and product surfaces
- Build or inspect: a minimal artifact that proves ontology version deprecation without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ontology version deprecation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ontology version deprecation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-014 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-015: policy evaluation latency budget
- Read: registry/catalog/ontology-kernel.yaml
- Connects to: per-tenant scoping in Rust domain types
- Build or inspect: a minimal artifact that proves policy evaluation latency budget without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for policy evaluation latency budget.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show policy evaluation latency budget is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-015 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-016: audit payload redaction
- Read: specs/root-hub-pointers.json
- Connects to: library-first read path versus service fan-out
- Build or inspect: a minimal artifact that proves audit payload redaction without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit payload redaction.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit payload redaction is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-016 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-017: event schema migration
- Read: specs/masterplan.json
- Connects to: registry catalog row shape and ownership metadata
- Build or inspect: a minimal artifact that proves event schema migration without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for event schema migration.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show event schema migration is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-017 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-018: crate naming exception analysis
- Read: docs/standards/layer-enum-adr-0105.md
- Connects to: versioning and deprecation for substrate crates
- Build or inspect: a minimal artifact that proves crate naming exception analysis without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for crate naming exception analysis.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show crate naming exception analysis is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-architecture with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-018 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-019: layer enum crate classification
- Read: specs/crate-naming-audit.json
- Connects to: multi-region audit event replication behavior
- Build or inspect: a minimal artifact that proves layer enum crate classification without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for layer enum crate classification.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show layer enum crate classification is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-019 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-020: Cedar publish default-deny
- Read: Cargo.toml
- Connects to: sovereign-cell constraints for regulated tenants
- Build or inspect: a minimal artifact that proves Cedar publish default-deny without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar publish default-deny.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar publish default-deny is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-020 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-021: audit event replay
- Read: crates/policy-cedar-domain/src/lib.rs
- Connects to: 13-value layer enum and why `api` is protocol-neutral
- Build or inspect: a minimal artifact that proves audit event replay without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit event replay.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit event replay is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-021 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-022: ontology action projection
- Read: crates/policy-cedar-api/tests/cedar_policy_publish_api.rs
- Connects to: Cedar fragment lifecycle from proposal to signed activation
- Build or inspect: a minimal artifact that proves ontology action projection without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ontology action projection.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ontology action projection is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-022 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-023: tenant_id type boundary
- Read: crates/audit-chain-domain/src/lib.rs
- Connects to: audit-chain Merkle seal, Ed25519 signature, and replay verification
- Build or inspect: a minimal artifact that proves tenant_id type boundary without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant_id type boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant_id type boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-023 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-024: OpenAPI api crate boundary
- Read: crates/audit-chain-domain/tests/merkle_chain.rs
- Connects to: ontology projection for workflow and product surfaces
- Build or inspect: a minimal artifact that proves OpenAPI api crate boundary without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for OpenAPI api crate boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show OpenAPI api crate boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-architecture with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-024 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-025: catalog row ownership
- Read: crates/audit-chain-usecase/tests/audit_event_emit.rs
- Connects to: per-tenant scoping in Rust domain types
- Build or inspect: a minimal artifact that proves catalog row ownership without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for catalog row ownership.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show catalog row ownership is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-025 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-026: registry reverse dependency lookup
- Read: crates/ontology-kernel/src/lib.rs
- Connects to: library-first read path versus service fan-out
- Build or inspect: a minimal artifact that proves registry reverse dependency lookup without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for registry reverse dependency lookup.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show registry reverse dependency lookup is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-026 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-027: multi-region audit replication
- Read: crates/ontology-api/tests/object_graph_entity_upsert_api.rs
- Connects to: registry catalog row shape and ownership metadata
- Build or inspect: a minimal artifact that proves multi-region audit replication without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for multi-region audit replication.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show multi-region audit replication is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-027 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-028: sovereign-cell data class handling
- Read: registry/catalog/policy-cedar-domain.yaml
- Connects to: versioning and deprecation for substrate crates
- Build or inspect: a minimal artifact that proves sovereign-cell data class handling without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for sovereign-cell data class handling.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show sovereign-cell data class handling is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-028 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-029: Rust property test fixture
- Read: registry/catalog/audit-chain-domain.yaml
- Connects to: multi-region audit event replication behavior
- Build or inspect: a minimal artifact that proves Rust property test fixture without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Rust property test fixture.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Rust property test fixture is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-029 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-030: Merklized evidence chain
- Read: registry/catalog/ontology-kernel.yaml
- Connects to: sovereign-cell constraints for regulated tenants
- Build or inspect: a minimal artifact that proves Merklized evidence chain without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Merklized evidence chain.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Merklized evidence chain is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-architecture with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-030 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-031: Cedar soak rollback
- Read: specs/root-hub-pointers.json
- Connects to: 13-value layer enum and why `api` is protocol-neutral
- Build or inspect: a minimal artifact that proves Cedar soak rollback without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar soak rollback.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar soak rollback is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-031 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-032: ontology version deprecation
- Read: specs/masterplan.json
- Connects to: Cedar fragment lifecycle from proposal to signed activation
- Build or inspect: a minimal artifact that proves ontology version deprecation without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ontology version deprecation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ontology version deprecation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-032 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-033: policy evaluation latency budget
- Read: docs/standards/layer-enum-adr-0105.md
- Connects to: audit-chain Merkle seal, Ed25519 signature, and replay verification
- Build or inspect: a minimal artifact that proves policy evaluation latency budget without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for policy evaluation latency budget.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show policy evaluation latency budget is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-033 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-034: audit payload redaction
- Read: specs/crate-naming-audit.json
- Connects to: ontology projection for workflow and product surfaces
- Build or inspect: a minimal artifact that proves audit payload redaction without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit payload redaction.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit payload redaction is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-034 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-035: event schema migration
- Read: Cargo.toml
- Connects to: per-tenant scoping in Rust domain types
- Build or inspect: a minimal artifact that proves event schema migration without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for event schema migration.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show event schema migration is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-035 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-036: crate naming exception analysis
- Read: crates/policy-cedar-domain/src/lib.rs
- Connects to: library-first read path versus service fan-out
- Build or inspect: a minimal artifact that proves crate naming exception analysis without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for crate naming exception analysis.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show crate naming exception analysis is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-architecture with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-036 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-037: layer enum crate classification
- Read: crates/policy-cedar-api/tests/cedar_policy_publish_api.rs
- Connects to: registry catalog row shape and ownership metadata
- Build or inspect: a minimal artifact that proves layer enum crate classification without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for layer enum crate classification.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show layer enum crate classification is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-037 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-038: Cedar publish default-deny
- Read: crates/audit-chain-domain/src/lib.rs
- Connects to: versioning and deprecation for substrate crates
- Build or inspect: a minimal artifact that proves Cedar publish default-deny without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar publish default-deny.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar publish default-deny is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-038 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-039: audit event replay
- Read: crates/audit-chain-domain/tests/merkle_chain.rs
- Connects to: multi-region audit event replication behavior
- Build or inspect: a minimal artifact that proves audit event replay without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit event replay.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit event replay is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-039 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-040: ontology action projection
- Read: crates/audit-chain-usecase/tests/audit_event_emit.rs
- Connects to: sovereign-cell constraints for regulated tenants
- Build or inspect: a minimal artifact that proves ontology action projection without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ontology action projection.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ontology action projection is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-040 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-041: tenant_id type boundary
- Read: crates/ontology-kernel/src/lib.rs
- Connects to: 13-value layer enum and why `api` is protocol-neutral
- Build or inspect: a minimal artifact that proves tenant_id type boundary without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant_id type boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant_id type boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-041 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-042: OpenAPI api crate boundary
- Read: crates/ontology-api/tests/object_graph_entity_upsert_api.rs
- Connects to: Cedar fragment lifecycle from proposal to signed activation
- Build or inspect: a minimal artifact that proves OpenAPI api crate boundary without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for OpenAPI api crate boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show OpenAPI api crate boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-architecture with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-042 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-043: catalog row ownership
- Read: registry/catalog/policy-cedar-domain.yaml
- Connects to: audit-chain Merkle seal, Ed25519 signature, and replay verification
- Build or inspect: a minimal artifact that proves catalog row ownership without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for catalog row ownership.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show catalog row ownership is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-043 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-044: registry reverse dependency lookup
- Read: registry/catalog/audit-chain-domain.yaml
- Connects to: ontology projection for workflow and product surfaces
- Build or inspect: a minimal artifact that proves registry reverse dependency lookup without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for registry reverse dependency lookup.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show registry reverse dependency lookup is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-044 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-045: multi-region audit replication
- Read: registry/catalog/ontology-kernel.yaml
- Connects to: per-tenant scoping in Rust domain types
- Build or inspect: a minimal artifact that proves multi-region audit replication without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for multi-region audit replication.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show multi-region audit replication is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-045 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-046: sovereign-cell data class handling
- Read: specs/root-hub-pointers.json
- Connects to: library-first read path versus service fan-out
- Build or inspect: a minimal artifact that proves sovereign-cell data class handling without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for sovereign-cell data class handling.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show sovereign-cell data class handling is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-046 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-047: Rust property test fixture
- Read: specs/masterplan.json
- Connects to: registry catalog row shape and ownership metadata
- Build or inspect: a minimal artifact that proves Rust property test fixture without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Rust property test fixture.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Rust property test fixture is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-047 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-048: Merklized evidence chain
- Read: docs/standards/layer-enum-adr-0105.md
- Connects to: versioning and deprecation for substrate crates
- Build or inspect: a minimal artifact that proves Merklized evidence chain without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Merklized evidence chain.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Merklized evidence chain is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-architecture with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-048 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-049: Cedar soak rollback
- Read: specs/crate-naming-audit.json
- Connects to: multi-region audit event replication behavior
- Build or inspect: a minimal artifact that proves Cedar soak rollback without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar soak rollback.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar soak rollback is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-049 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-050: ontology version deprecation
- Read: Cargo.toml
- Connects to: sovereign-cell constraints for regulated tenants
- Build or inspect: a minimal artifact that proves ontology version deprecation without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ontology version deprecation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ontology version deprecation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-050 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-051: policy evaluation latency budget
- Read: crates/policy-cedar-domain/src/lib.rs
- Connects to: 13-value layer enum and why `api` is protocol-neutral
- Build or inspect: a minimal artifact that proves policy evaluation latency budget without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for policy evaluation latency budget.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show policy evaluation latency budget is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-051 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-052: audit payload redaction
- Read: crates/policy-cedar-api/tests/cedar_policy_publish_api.rs
- Connects to: Cedar fragment lifecycle from proposal to signed activation
- Build or inspect: a minimal artifact that proves audit payload redaction without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit payload redaction.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit payload redaction is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-052 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-053: event schema migration
- Read: crates/audit-chain-domain/src/lib.rs
- Connects to: audit-chain Merkle seal, Ed25519 signature, and replay verification
- Build or inspect: a minimal artifact that proves event schema migration without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for event schema migration.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show event schema migration is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-053 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-054: crate naming exception analysis
- Read: crates/audit-chain-domain/tests/merkle_chain.rs
- Connects to: ontology projection for workflow and product surfaces
- Build or inspect: a minimal artifact that proves crate naming exception analysis without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for crate naming exception analysis.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show crate naming exception analysis is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-architecture with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-054 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-055: layer enum crate classification
- Read: crates/audit-chain-usecase/tests/audit_event_emit.rs
- Connects to: per-tenant scoping in Rust domain types
- Build or inspect: a minimal artifact that proves layer enum crate classification without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for layer enum crate classification.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show layer enum crate classification is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-055 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-056: Cedar publish default-deny
- Read: crates/ontology-kernel/src/lib.rs
- Connects to: library-first read path versus service fan-out
- Build or inspect: a minimal artifact that proves Cedar publish default-deny without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar publish default-deny.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar publish default-deny is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-056 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-057: audit event replay
- Read: crates/ontology-api/tests/object_graph_entity_upsert_api.rs
- Connects to: registry catalog row shape and ownership metadata
- Build or inspect: a minimal artifact that proves audit event replay without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit event replay.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit event replay is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-057 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-058: ontology action projection
- Read: registry/catalog/policy-cedar-domain.yaml
- Connects to: versioning and deprecation for substrate crates
- Build or inspect: a minimal artifact that proves ontology action projection without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ontology action projection.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ontology action projection is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-058 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-059: tenant_id type boundary
- Read: registry/catalog/audit-chain-domain.yaml
- Connects to: multi-region audit event replication behavior
- Build or inspect: a minimal artifact that proves tenant_id type boundary without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for tenant_id type boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show tenant_id type boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-059 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-060: OpenAPI api crate boundary
- Read: registry/catalog/ontology-kernel.yaml
- Connects to: sovereign-cell constraints for regulated tenants
- Build or inspect: a minimal artifact that proves OpenAPI api crate boundary without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for OpenAPI api crate boundary.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show OpenAPI api crate boundary is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-architecture with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-060 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-061: catalog row ownership
- Read: specs/root-hub-pointers.json
- Connects to: 13-value layer enum and why `api` is protocol-neutral
- Build or inspect: a minimal artifact that proves catalog row ownership without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for catalog row ownership.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show catalog row ownership is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-061 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-062: registry reverse dependency lookup
- Read: specs/masterplan.json
- Connects to: Cedar fragment lifecycle from proposal to signed activation
- Build or inspect: a minimal artifact that proves registry reverse dependency lookup without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for registry reverse dependency lookup.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show registry reverse dependency lookup is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-062 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-063: multi-region audit replication
- Read: docs/standards/layer-enum-adr-0105.md
- Connects to: audit-chain Merkle seal, Ed25519 signature, and replay verification
- Build or inspect: a minimal artifact that proves multi-region audit replication without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for multi-region audit replication.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show multi-region audit replication is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-063 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-064: sovereign-cell data class handling
- Read: specs/crate-naming-audit.json
- Connects to: ontology projection for workflow and product surfaces
- Build or inspect: a minimal artifact that proves sovereign-cell data class handling without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for sovereign-cell data class handling.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show sovereign-cell data class handling is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-064 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-065: Rust property test fixture
- Read: Cargo.toml
- Connects to: per-tenant scoping in Rust domain types
- Build or inspect: a minimal artifact that proves Rust property test fixture without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Rust property test fixture.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Rust property test fixture is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-065 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-066: Merklized evidence chain
- Read: crates/policy-cedar-domain/src/lib.rs
- Connects to: library-first read path versus service fan-out
- Build or inspect: a minimal artifact that proves Merklized evidence chain without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Merklized evidence chain.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Merklized evidence chain is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-architecture with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-066 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-067: Cedar soak rollback
- Read: crates/policy-cedar-api/tests/cedar_policy_publish_api.rs
- Connects to: registry catalog row shape and ownership metadata
- Build or inspect: a minimal artifact that proves Cedar soak rollback without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar soak rollback.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar soak rollback is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-067 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-068: ontology version deprecation
- Read: crates/audit-chain-domain/src/lib.rs
- Connects to: versioning and deprecation for substrate crates
- Build or inspect: a minimal artifact that proves ontology version deprecation without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for ontology version deprecation.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show ontology version deprecation is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-068 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-069: policy evaluation latency budget
- Read: crates/audit-chain-domain/tests/merkle_chain.rs
- Connects to: multi-region audit event replication behavior
- Build or inspect: a minimal artifact that proves policy evaluation latency budget without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for policy evaluation latency budget.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show policy evaluation latency budget is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-ontology with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-069 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-070: audit payload redaction
- Read: crates/audit-chain-usecase/tests/audit_event_emit.rs
- Connects to: sovereign-cell constraints for regulated tenants
- Build or inspect: a minimal artifact that proves audit payload redaction without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for audit payload redaction.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show audit payload redaction is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-tenancy with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-070 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-071: event schema migration
- Read: crates/ontology-kernel/src/lib.rs
- Connects to: 13-value layer enum and why `api` is protocol-neutral
- Build or inspect: a minimal artifact that proves event schema migration without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for event schema migration.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show event schema migration is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to ops-sre-reliability with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-071 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-072: crate naming exception analysis
- Read: crates/ontology-api/tests/object_graph_entity_upsert_api.rs
- Connects to: Cedar fragment lifecycle from proposal to signed activation
- Build or inspect: a minimal artifact that proves crate naming exception analysis without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for crate naming exception analysis.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show crate naming exception analysis is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to council-architecture with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-072 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-073: layer enum crate classification
- Read: registry/catalog/policy-cedar-domain.yaml
- Connects to: audit-chain Merkle seal, Ed25519 signature, and replay verification
- Build or inspect: a minimal artifact that proves layer enum crate classification without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for layer enum crate classification.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show layer enum crate classification is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-policy-engine with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-073 contains file path, claim, evidence, rollback, and reviewer.

### Drill SWE-074: Cedar publish default-deny
- Read: registry/catalog/audit-chain-domain.yaml
- Connects to: ontology projection for workflow and product surfaces
- Build or inspect: a minimal artifact that proves Cedar publish default-deny without widening beyond backend Rust SWE, platform substrate team.
- Failure tree: name happy path, tenant or region failure, malicious or mistaken actor, and stale-contract failure for Cedar publish default-deny.
- Observability: name the metric, trace, log, audit event, dashboard, evidence file, or customer artifact that would show Cedar publish default-deny is working.
- Rollback: name the command, file revert, policy revocation, model rollback, locale rollback, runbook branch, or customer comms correction that restores safety.
- Review: route to axis-audit-chain with target result, evidence, and stop condition.
- Done artifact: onboarding issue row SWE-074 contains file path, claim, evidence, rollback, and reviewer.

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
