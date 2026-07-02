---
doc_class: MasterPlan
shape: compatibility_projection_non_authoritative
length_cap: 800
authority_tier: 4
status: Accepted
date: 2026-05-19
owners:
- council-architecture
canonical_authority: /specs/masterplan.json
live_plan_authority: false
read_contract:
  audience:
    - humans
  read_timing_class: on-demand
  freshness_rule: "Projection only; conflicts resolve to /specs/masterplan.json#masterplan_v2."
companion_docs:
- /specs/root-hub-pointers.json
- /specs/master-plan-sequencing.json
- /specs/planning-closure-contract.json
- /specs/planning-closure-status-closure-ledger.json
- docs/decisions/ADR-0217-vertical-slice-rollout-order.md
authority_chain_declaration: |
  system / developer / user instructions
    > /specs/root-hub-pointers.json
    > docs/AGENTS.md (operating contract until explicit /specs/agent-operating-contract.json PHASE-5 promotion evidence)
    > installed agent-runtime skill and role catalog (for Codex: ~/.codex/skills + ~/.codex/agents; project .codex overlays only when intentionally checked in)
    > /specs/masterplan.json#masterplan_v2 (sole live plan authority and work-item ID namespace)
    > machine-readable specs and registries under /specs, /registry, /evidence, and /templates (supporting evidence/provenance only unless directly cited by masterplan v2)
    > external/upstream skill documentation (informational only; not vendored into this repo)
    > repo-root Redirect-class files (non-authoritative; lane-thin)
    > working drafts (never authoritative)
purpose: "Human compatibility projection for the machine-readable Oyatie master plan."
doc_status: published
---
# Oyatie Master Plan

This file is a human compatibility projection only. It is not a live plan authority, does not mint work-item IDs, and does not carry status claims. The canonical master plan, live work-item ID space, dependency DAG, surface dispositions, and read contracts live in `/specs/masterplan.json#masterplan_v2`.

## Current Authority

- Canonical plan authority: `/specs/masterplan.json`
- Canonical fragment for this consolidation: `/specs/masterplan.json#masterplan_v2`
- Live work-item ID namespace: `MPV2-####`, validated by the cloud-ci cross-artifact agreement masterplan-v2 authority check.
- Former plan surfaces (`/specs/master-plan-sequencing.json`, `/specs/planning-closure-contract.json`, `/specs/planning-closure-status-closure-ledger.json`, `docs/ROADMAP.md`, `docs/decisions/ADR-0015-architectural-flattening-target.md`, `docs/decisions/ADR-0042-observability-stack-otel-and-in-house-ui.md`, `docs/decisions/ADR-0046-vector-store-strategy.md`, `docs/decisions/ADR-0052-inventory-grit-cutover.md`, `docs/decisions/ADR-0053-grit-icm-as-sanctioned-primitives.md`, `docs/decisions/ADR-0057-cutover-mechanics-rename-plan-v4.md`, `docs/decisions/ADR-0097-intelligence-account-adapter-rename-target-slot-last.md`, `docs/decisions/ADR-0101-supervisor-mountpoint-direct-hyper.md`, `docs/decisions/ADR-0102-intelligence-settings-template-canonical-rendering.md`, `docs/decisions/ADR-0103-grit-cutover-inventory.md`, `docs/decisions/ADR-0107-tools-implicit-app-convention.md`, `docs/decisions/ADR-0110-changeset-state-machine.md`, `docs/decisions/ADR-0112-webhook-driven-intelligence-agent-invocation.md`, `docs/decisions/ADR-0113-vcs-orchestrator-end-to-end.md`, `docs/decisions/ADR-0120-rust-first-onprem-tooling-with-paired-uninstall.md`, `docs/decisions/ADR-0121-onprem-k8s-stack-kubeadm-containerd-istio-envoy.md`, `docs/decisions/ADR-0124-own-merge-queue-webhook-driven.md`, `docs/decisions/ADR-0136-intelligence-as-single-microservice.md`, `docs/decisions/ADR-0137-intelligence-bounded-contexts.md`, `docs/decisions/ADR-0138-intelligence-six-path-deprecation.md`, `docs/decisions/ADR-0140-cross-cutting-carriers-adapter-exemption.md`, `docs/decisions/ADR-0141-workflow-ontology-read-path-direct.md`, `docs/decisions/ADR-0143-intelligence-per-bc-release-pointer.md`, `docs/decisions/ADR-0160-progressive-delivery-flagger.md`, `docs/decisions/ADR-0170-developer-portal.md`, `docs/decisions/ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md`, `docs/decisions/ADR-0187-canonical-oidc-idp-zitadel-primary.md`, `docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md`, `docs/decisions/ADR-0359-jenkins-completely-replaces-github-actions.md`, `docs/decisions/ADR-0361-jenkins-native-cicd-revamp-execution.md`, `docs/decisions/ADR-0372-frontend-stack-solidjs-ts-with-rust-wasm-compute-modules.md`, `docs/decisions/ADR-0509-hyperscaler-service-decomposition-pattern.md`, `docs/decisions/ADR-0511-ci-orchestration-argo-workflows-supersede-jenkins.md`, `docs/decisions/ADR-0513-oya-ci-bespoke-rust-prow-cicd-platform.md`, `docs/decisions/ADR-0514-build-ci-cd-pipeline-target-architecture-hyperscaler-remediation.md`, and legacy `.omc/.omx/.gjc/.hermes` artifacts) are absorbed provenance or runtime data, not live plan authorities.

Historical `.omc`/`.omx` planning prompts and local runtime stores may be forensically read only when a gate or masterplan v2 evidence reference asks for them. They never override `/specs/masterplan.json`.

## Projection Contract

This projection intentionally avoids duplicating sequence, scope, status, or dependency detail. Humans use it as a pointer; agents and gates read `/specs/masterplan.json#masterplan_v2` directly.

Any update that adds roadmap content, work-item IDs, readiness status, or sequencing here without a generated-projection freshness gate is stale on arrival and must be rejected.