---
doc_class: FoundrySpecCheckpoint
title: "Foundry Pipeline Substrate Documentation Checkpoint"
status: Draft
date: 2026-05-20
owner: "axis-foundry + council-foundry-vcs"
related_oyatie_adrs:
  - ADR-0110
  - ADR-0111
  - ADR-0112
  - ADR-0113
  - ADR-0116
  - ADR-0136
  - ADR-0220
  - ADR-0221
  - ADR-0263
---

# Foundry Pipeline Substrate Documentation Checkpoint

> RETIRED-CONTEXT NOTE 2026-05-21: The `foundry` µservice this checkpoint references is RETIRED per ADR-0335 (Wave 15I). The substantive agentic-pipeline doctrine (changeset state, admission gate, merge queue, completion gate, webhook-driven invocation, VCS orchestrator) lives in ADRs 0110, 0111, 0112, 0113, 0116, 0247, 0255 and is implemented across vcs-orchestrator + intelligence + workflow + audit-chain + observability + identity + tenancy + policy-engine. The "Hermes" name is RETIRED corpus-wide per ADR-0247 D-10 + ADR-0328 D-9.22 + ADR-0335 D-26..D-36. This checkpoint is preserved as historical evidence; for live AI substrate authority cite `microservices/intelligence/manifest.json`.

This checkpoint records the May 20, 2026 documentation slice for the internal agentic-development pipeline (previously branded Hermes, now retired).

Canonical authored specs live under `microservices/intelligence/spec/` because the active Oya VCS claim, verify, done, and promote scope for this slice was `microservices/intelligence`. Authority is HISTORICAL after ADR-0335.

The specs explicitly preserved the ADR-0136/ADR-0220 boundary at authoring time: Foundry was internal development infrastructure, while consumer-facing AI belonged to Intelligence. ADR-0335 collapses that boundary by absorbing Foundry into Intelligence per ADR-0255 KS#14.

Authored files:
- microservices/intelligence/spec/changeset-state-machine.md (912 lines)
- microservices/intelligence/spec/merge-queue-projected-state.md (903 lines)
- microservices/intelligence/spec/webhook-driven-agent-invocation.md (916 lines)
- microservices/intelligence/spec/vcs-orchestrator-end-to-end.md (919 lines)
- microservices/intelligence/spec/agent-pipeline-isolation-worktree.md (898 lines)
- microservices/intelligence/spec/admission-gate-policy-and-evidence.md (898 lines)
- microservices/intelligence/spec/completion-gate-reviewer-and-ci.md (914 lines)
- microservices/intelligence/spec/agent-types-and-roles.md (903 lines)

Protected directories intentionally untouched: `microservices/intelligence/capability-tiers`, `onboarding`, `faqs`, `tutorials`, `benchmarks`, `migration-playbooks`, `reference-implementations`, and `decisions`.

Verification sequence required by the task:
- `./bin/oya vcs verify --agent codex-foundry-spec-w1 --evidence 'specs_authored:8' microservices/intelligence`
- `./bin/oya vcs done --agent codex-foundry-spec-w1 --evidence 'specs_authored:8' microservices/intelligence`
- `./bin/oya vcs promote --agent codex-foundry-spec-w1 --bundle foundry-spec-w1-2026-05-20 --environment dev --evidence 'specs_authored:8' microservices/intelligence`

