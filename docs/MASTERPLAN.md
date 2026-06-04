---
doc_class: MasterPlan
shape: compatibility_projection
length_cap: 800
authority_tier: 0
status: Accepted
date: 2026-05-19
owners:
- council-architecture
canonical_authority: /specs/masterplan.json
companion_docs:
- /specs/root-hub-pointers.json
- /specs/master-plan-sequencing.json
- /specs/planning-closure-contract.json
- /specs/planning-closure-status-closure-ledger.json
- docs/decisions/ADR-0217-vertical-slice-rollout-order.md
authority_chain_declaration: |
  system / developer / user instructions
    > /specs/root-hub-pointers.json
    > docs/AGENTS.md (until /specs/agent-operating-contract.json PHASE-5 promotion)
    > tools/agent-skills/AGENTS.md (inherited base from addyosmani/agent-skills MIT — universal intent→skill mapping, anti-rationalization, persona/skill/command orchestration; oyatie overlays via this file and wins on conflict)
    > machine-readable specs and registries under .omc/
    > docs/ authority files during markdown-retirement compatibility
    > tools/agent-skills/CLAUDE.md (informational; describes vendored subtree, not oyatie)
    > repo-root Redirect-class files (non-authoritative; lane-thin)
    > working drafts (never authoritative)
purpose: "Human compatibility projection for the machine-readable Oyatie master plan."
doc_status: published
---
# Oyatie Master Plan

This file is a compatibility projection for humans. It is not the implementation authority. Agents and automation must resolve master-plan truth through `/specs/root-hub-pointers.json`, `/specs/masterplan.json`, `/specs/master-plan-sequencing.json`, `/specs/planning-closure-contract.json`, `/specs/planning-closure-status-closure-ledger.json`, and the current gate output.

## Current Authority

The canonical master plan is `/specs/masterplan.json`.

Planning-closure truth is machine-readable and Buck2-guarded through the current
authority/hygiene checks; the former `oya-dev-cli ... gate validate
planning-closure` command is retired as merge/CI authority and must not be
revived.

```bash
buck2 build //:repo-hygiene-automation-check //:buck2-authority-policy-check
```

The active long-running implementation goal is `/evidence/goals/fd001-planning-closure-implementation-goal-2026-05-19.json`.

Historical planning prompts and old `.omc/plans/` root drafts are archived under `.omc/archive/stale-documents/2026-05-19-planning-closure/` and must not be used as sequencing, scope, open-question, or implementation-start authority.


## P00 GitHub transition and hygiene unlock

The imminent top priority is P00: use GitHub/GitHub Actions as a temporary lane-unlocker while native SCM/CI/CD/cloud workspace seams mature. Buck2 remains build/test/check authority, and the native destination remains cloud native, Kubernetes-native, and hyperscaler native.

P00 also includes repo hygiene automation: `specs/repo-hygiene-automation.json` governs git/worktree, branch/merge, repository publication, disk/workspace, Kubernetes workload, and documentation-sprawl hygiene. Run:

```bash
buck2 build //:repo-hygiene-automation-check
buck2 build //tools/oya-doc-staleness-inventory-app:doc-staleness-inventory-unit-tests
buck2 build //tools/oya-doc-staleness-inventory-app:doc-staleness-inventory-json
```

The native Sapling-inspired SCM must expose GitHub public/private publication and status adapters, but GitHub is not the durable source of truth. Shared docs, indexes, registries, and workflows stay pointer-thin and route detail to lane-owned shards. Retired external SCM/CI/CD substrate names are tombstoned in `specs/retired-external-substrate-registry.json`.

The P00 backlog now treats shared-surface reduction and security hardening as
first-class prep work: vertical-owned reusable CI shards or generated
consolidation should replace repeated edits to canonical shared workflows, new
fanout automation is Rust/Buck2-first, and
`specs/repo-hygiene-automation.json#security_hardening_backlog` records the
valid zero-trust, ABAC, microsegmentation, workload-identity, pod/runtime
isolation, CI-secret isolation, and multi-account guardrail items for follow-up.

## First Deliverable

FD-001 is Tenant RBAC view plus Tenant RBAC view at full production depth. This is not a preview scope and not a reduced launch. The first deliverable exits only when the canonical base and Korea localization pack are both ready with evidence.

Required FD-001 surfaces:

- core
- messenger
- mail
- community
- infra
- ops dashboard and control center
- intelligence
- workflow
- ontology
- canonical base
- Korea localization pack

Later sector-specific verticals remain downstream of FD-001. They do not dilute the first-deliverable exit bar.

## Architecture Bar

FD-001 uses flat microservices, clean architecture, API-first contracts, independent horizontal scaling, tenant isolation, observability, auditability, policy compliance, performance budgets, explicit rollback, and evidence-backed gates.

Every microservice must have:

- PRD, phase, implementation-plan, and ChangeSet coverage
- inward dependency direction across kernel, domain, application, API, adapter, and runtime layers
- OpenAPI, AsyncAPI, or protobuf contracts before handlers
- service-owned data boundaries and no direct cross-service calls
- Workflow and Ontology integration for cross-service orchestration and information flow
- tenant quota, backpressure, and horizontal scaling strategy
- golden-signal telemetry, SLOs, runbooks, and incident evidence
- audit-chain events, retention, replay, and provenance evidence
- threat model, policy controls, and compliance evidence
- capacity model, load evidence, and cost attribution
- import, export, migration, rollback, and vendor-exit paths

The ops dashboard and control center are part of FD-001, not an afterthought. They must cover deployment state, tenant health, incident response, policy posture, audit evidence, rollback, restore, and operator-safe remediation.

## Deployment Bar

Deployment must be reproducible through cloud-native Kubernetes, OCI artifacts, GitOps, OpenTofu, SBOM and provenance attestations, and conformance evidence.

Supported execution environments include Talos, Ubuntu LTS, Debian, Fedora Server, Oracle Linux, RHEL-compatible distributions, CentOS Stream, Rocky Linux, AlmaLinux, SUSE Linux Enterprise, and macOS Apple Silicon local parity.

Production images default to distroless or scratch. Any exception requires evidence, a vulnerability budget, and a replacement path.

Bootstrap must support one-command or one-click setup. Talos and other cluster hosts must be able to join remotely through a secure configuration-driven flow that fails closed when prerequisites are missing.

## Development Order

Execution follows vertical delivery ordering:

1. Lock shared contracts, schemas, architecture rules, policy, bootstrap, deployment, and evidence gates.
2. Build FD-001 through product-vertical slices that include customer UX, domain logic, APIs, data, policy, telemetry, tests, operations, deployment, and evidence.
3. Parallelize only after shared contracts are locked. Safe parallel lanes include messenger, mail, community, ops dashboard/control center, intelligence, workflow, ontology, infra, and Korea localization pack.
4. Serialize shared data model ownership, root workspace manifest changes, public API compatibility changes, branch protection, and promotion policy changes.
5. Promote interim dev work only through an isolated plain-git branch, PR against `dev`, automated GitHub compatibility/shadow evidence, Buck2 build/test/check evidence, the trusted Prow/Kubernetes-native `oya-ci-required` target context as it cuts over, and reviewer/governance approval. Native promotion returns to the cloud native/Kubernetes-native/hyperscaler-native SCM/CI/CD substrate after cutover evidence; retired `oya gate` / `oya verify` CLI output is not merge authority.

## Claim Rule

Planning closure means the implementation contract is complete enough to start execution without inventing strategy mid-flight. It does not mean product implementation is already complete.

Production-grade, hyperscaler-grade, or industry-leading claims are valid only after implementation evidence exists for the relevant ChangeSets and the required gates pass with current output.
