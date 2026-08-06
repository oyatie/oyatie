---
id: ADR-0335
status: Accepted
amended_by: [ADR-619]
planning_impact: true
date: 2026-05-21
owner_team:
  - council-architecture
  - axis-intelligence
  - axis-foundry
deciders:
  - user-directive-2026-05-21
  - council-architecture
  - axis-intelligence
  - axis-foundry
supersedes:
  - microservices/foundry/PRD.md
  - microservices/foundry/ARCHITECTURE.md
  - microservices/foundry/PHASE-01-FOUNDRY-FOUNDATION.md
  - microservices/foundry/PHASE-02-FOUNDRY-DATA-SUBSTRATE-ADDENDUM.md
amends:
  - ADR-0136
  - ADR-0138
  - ADR-0220
  - ADR-0239
  - ADR-0247
  - ADR-0255
related:
  - ADR-0255-intelligence-as-two-layer-ai-substrate.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0220-consumer-intelligence-substrate.md
  - ADR-0239-amendment-intelligence-internal-scope-clarification-2026-05-18.md
  - ADR-0136-amendment
  - ADR-0138-intelligence-six-path-deprecation.md
  - ADR-0116-retire-external-agent-coordination-tooling.md
  - ADR-0112-webhook-driven-intelligence-agent-invocation.md
  - ADR-0113-vcs-orchestrator-end-to-end.md
  - ADR-0132-product-platform-and-bundle-dissolution.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0333-cell-microservice-retired-pattern-not-service.md
  - ADR-0334-shorts-microservice-merged-into-social.md
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
related_sources:
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_intelligence_two_layer_substrate.md
  - microservices/foundry/PRD.md
  - microservices/foundry/ARCHITECTURE.md
  - microservices/intelligence/manifest.json
  - microservices/intelligence/IP-001-consumer-intelligence-substrate.md
  - docs/decisions/ADR-0255-intelligence-as-two-layer-ai-substrate.md
  - docs/decisions/ADR-0247-self-hosting-self-modification-doctrine.md
  - docs/decisions/ADR-0220-consumer-intelligence-substrate.md
doc_class: Architecture-Decision-Record
purpose: >
  Retire the foundry µservice as a standalone service surface; absorb its AI
  pipeline orchestration, eval, training, RLHF, red-team, and model registry
  responsibilities into the intelligence µservice per ADR-0255 KS#14
  two-layer AI substrate; drop "retired external agent harness" terminology corpus-wide as the
  retired internal pipeline brand name.
---

# ADR-0335: foundry µservice retired; absorbed by intelligence; retired external agent harness terminology dropped

## Status

Accepted — 2026-05-21.

This ADR executes the ADR-0255 KS#14 two-layer intelligence substrate
doctrine. ADR-0255 names intelligence as the canonical AI substrate that
"absorbs Foundry". This ADR records the absorption landing.

The retirement removes a service boundary.

The retirement does not remove any AI capability.

The retirement does not remove the eval substrate.

The retirement does not remove training, RLHF, or red-team capability.

The retirement does not remove the model registry.

The retirement does not remove the agent dispatch surface.

The retirement does not remove guardrails or autonomy ceilings.

The retirement does not remove evidence collection or audit-chain
integration.

The retirement does not weaken ADR-0247 (self-modification doctrine).

The retirement does not weaken ADR-0255 (intelligence two-layer substrate).

This ADR codifies where every retired `foundry` responsibility now lives.

It also retires "retired external agent harness" as a canonical primitive, executing the
retirement queued by ADR-0247 D-10 and confirmed by ADR-0328 D-9.18 +
D-9.22 + D-12.22..D-12.24.

## Context

The prior `foundry` µservice was the consolidation of six earlier
candidates per ADR-0136: runtime, supervisor, eval, evidence, guardrails,
providers. ADR-0138 sequenced the six-path deprecation. ADR-0239
amendment clarified foundry as INTERNAL-only — the retired external agent harness agentic
development pipeline — and split consumer AI into the separate
intelligence µservice per ADR-0220.

ADR-0247 then established the self-modification doctrine: foundry runs
as `oyatie.foundry.*` principals inside the `oyatie` tenant; foundry IS
self-modification, not a separate axis. ADR-0247 D-10 queued the
retirement of the "retired external agent harness" name as inherited-from-external terminology
(an inherited external package) that never became canonical.

ADR-0255 then established the intelligence two-layer substrate (Layer A
AI substrate + Layer B Consumer Brand Surface) and named intelligence
as the canonical AI µservice that absorbs Foundry. The session memory
note `feedback_intelligence_two_layer_substrate` records this as KS#14.

ADR-0328 then sequenced this retirement explicitly: D-9.18 assigns
"Foundry retirement plus retired external agent harness-drop cleanup" to sub-wave 15I; D-9.22
states "15I drops `retired external agent harness` as a canonical primitive"; D-12.22..D-12.24
establish the corpus-wide drop policy.

Waves 15K (network → community), 15L (cell retired as pattern), and
15O (shorts absorbed into social) established the in-session
retirement precedent. Each pruned a service boundary while preserving
the underlying capabilities and absorbing them into the correct owner.

The 2026-05-20 coherence audit found the foundry artifacts substantive
(493 total artifacts per `specs/microservices/foundry.json`). The
issue is not lack of substance.

The issue is ownership shape. Two AI µservices duplicate the same
substrate. Intelligence already owns the consumer surface, the
provider router, the guardrail stack, the eval surface, the
audit-tap, the attribution surface, the brand UX surface, the
credential resolver, the assist-draft, and the context-aware
retrieval. Foundry duplicates the same model router, the same
provider adapters, the same eval runner, the same guardrails kernel,
and the same evidence pipeline.

The user revisited the µservice boundary on 2026-05-21 and confirmed
the absorption.

Therefore foundry retires as a µservice and its responsibilities
absorb into intelligence as the canonical AI substrate.

## Decision

### D-1..D-12. Service boundary

D-1. `microservices/foundry/` is retired as a standalone µservice.

D-2. `microservices/foundry/` keeps only a `RETIRED.md` redirect marker
plus historical-evidence subdirectories explicitly preserved.

D-3. Historical foundry service content is not the live authority
after this ADR.

D-4. `microservices/intelligence/` is the canonical AI substrate
µservice.

D-5. `microservices/intelligence/` is the canonical owner of model
routing.

D-6. `microservices/intelligence/` is the canonical owner of the
guardrail stack.

D-7. `microservices/intelligence/` is the canonical owner of the eval
substrate.

D-8. `microservices/intelligence/` is the canonical owner of the
audit-tap.

D-9. `microservices/intelligence/` is the canonical owner of provider
adapters.

D-10. `microservices/intelligence/` is the canonical owner of credential
resolution.

D-11. `microservices/intelligence/` is the canonical owner of attribution.

D-12. `microservices/intelligence/` is the canonical owner of the
consumer brand UX surface.

### D-13..D-25. Absorbed AI pipeline orchestration

D-13. Agent runtime orchestration absorbs into intelligence dispatch-flow.

D-14. Supervisor control-plane orchestration absorbs into intelligence
operational substrate.

D-15. Capability registry absorbs into intelligence model and adapter
registry.

D-16. Eval runner + parity analyzer + replay engine absorb into the
intelligence eval bounded context.

D-17. Evidence pack builder + regulator export + audit-chain bridge
absorb into the intelligence audit-tap bounded context.

D-18. Guardrails (prompt classifier, output validator, autonomy-tier
gate, content-safety, jailbreak detector, AI-slop detector) absorb
into the intelligence guardrails bounded context.

D-19. Provider router + 8 adapters absorb into the intelligence
providers bounded context (which already enumerates 18 first-class
providers, superseding foundry's 8).

D-20. Training and fine-tuning workflows absorb into intelligence as
provider-side or in-house adapter operations.

D-21. RLHF (reinforcement learning from human feedback) workflows
absorb into intelligence eval and provider-adapter surfaces.

D-22. Red-team workflows absorb into intelligence guardrails +
intelligence eval (regression refusal correctness).

D-23. Model registry absorbs into intelligence provider-adapter
registry plus tenant-class entitlement gating per ADR-0330.

D-24. Webhook-driven agent invocation per ADR-0112 absorbs into
intelligence dispatch-flow, retargeting webhook receivers from a
retired foundry namespace to the intelligence event namespace.

D-25. End-to-end VCS orchestration per ADR-0113 remains a Foundry
self-modification concern under the `oyatie.foundry.*` Cedar principal
namespace; the principal namespace persists, the µservice does not.

### D-26..D-36. retired external agent harness terminology drop

D-26. "retired external agent harness" is retired as a canonical primitive corpus-wide,
executing ADR-0247 D-10 + ADR-0328 D-9.22 + ADR-0328 D-12.22.

D-27. "retired external agent harness pipeline" is replaced by "intelligence pipeline" or
"AI substrate pipeline" depending on context.

D-28. "retired external agent harness agentic development pipeline" is replaced by
"oyatie.foundry workflow library" (per ADR-0247 D-10 mapping) when
the context is self-modification; replaced by "intelligence pipeline"
when the context is AI workflow execution.

D-29. "retired external agent harness agent" is replaced by "oyatie.foundry.<workflow-id>
instance" (per ADR-0247 D-10).

D-30. ADRs documenting retired external agent harness history (ADR-0136-amendment, ADR-0220,
ADR-0239, ADR-0242, ADR-0245, ADR-0247, ADR-0211, ADR-0253-amendment,
ADR-0328) retain retired external agent harness references in their historical sections; new
content does not introduce retired external agent harness.

D-31. Onboarding and FAQ documents previously framed around retired external agent harness
are rewritten to drop the retired external agent harness brand. The substantive content
moves to intelligence as appropriate, or retires as retired external agent harness-specific
internal onboarding that does not carry forward.

D-32. Canonical primitives (`tools/hooks/_canonical-primitives.md`)
drop the foundry / retired external agent harness line and route AI substrate references to
intelligence only.

D-33. CLAUDE.md and other agent-rules surfaces drop active "retired external agent harness"
references; historical pointers remain only inside ADRs marked as
history.

D-34. Memory files may retain historical retired external agent harness references; new
memory entries do not introduce retired external agent harness terminology.

D-35. ADR-0619 replaces the proposed source-derived gate names with one neutral strict-zero
residue rule that scans every tracked path and raw blob without carve-outs.

D-36. The retired external agent harness drop is structural; no replacement term is needed
because the underlying capability is now "intelligence" (consumer) or
"oyatie.foundry workflow library inside dev-tools-cell-N"
(self-modification).

### D-37..D-50. Crate transition debt

D-37. `oya-foundry-*` crates that exist in the workspace at landing
time are transition debt under ADR-0138 strangler discipline.

D-38. The transition-debt treatment follows the precedent established
by Wave 15L (ADR-0333 D-59): existing crates are not renamed in this
ADR to avoid a 122-crate rename cascade across 43 dependent crates,
which would risk breaking `cargo check --workspace` and stall the
substantive corpus-wide work this wave authorizes.

D-39. New code must not generate `oya-foundry-*` crates after this
ADR.

D-40. New code targeting AI substrate capabilities must generate
`oya-intelligence-*` crates per ADR-0255 + intelligence manifest.

D-41. Existing `oya-foundry-*` crate references in dependent crates
remain compilable; the crate names are namespaces, not service
boundaries.

D-42. The `oyatie.foundry.*` Cedar principal namespace remains active
per ADR-0247; the principal namespace is independent of µservice
shape.

D-43. Future renaming of `oya-foundry-*` crates to `oya-intelligence-*`
crates is a separate cleanup wave, sequenced after this ADR lands.

D-44. The current cargo workspace remains green at landing time.

D-45. The foundry workspace members retain workspace membership as
historical-evidence crates.

D-46. Workspace member ordering is not changed by this ADR.

D-47. No `Cargo.toml` workspace members are deleted by this ADR.

D-48. Future workspace cleanup may delete unused `oya-foundry-*`
members per a separate retirement queue.

D-49. The `oya-check-eu-ai-act-annex-iii-refusal` crate is already
deleted per session state; this ADR does not reinstate it.

D-50. The intelligence manifest's existing AI surface enumeration
(`bounded_contexts`, `capabilities`, `slos`, `ips`) already covers
the substantive foundry scope. This ADR records the absorption
without duplicating the enumeration.

### D-51..D-65. Specs and structural updates

D-51. `specs/microservices/foundry.json` is rewritten as a retirement
marker pointer to this ADR + intelligence.

D-52. `specs/microservices/manifests-index.json` updates the foundry
entry to mark it retired with the intelligence absorption pointer.

D-53. `specs/master-plan-sequencing.json` marks foundry as
retired-by-wave-15I in the µservice roster and removes it from active
µservice lists where it conflicts with the retirement.

D-54. `specs/root-hub-pointers.json` updates `prd_foundry` and other
foundry pointer keys to mark them retired-by-wave-15I with the
absorption pointer.

D-55. `microservices/intelligence/manifest.json` appends an
`absorbed_microservices` field listing `foundry` with this ADR as
authority.

D-56. `tools/hooks/_canonical-primitives.md` updates the AI Substrate
section: foundry retired; intelligence is the single AI surface.

D-57. The canonical primitives section drops the line claiming
`microservices/foundry/` is "internal retired external agent harness dev pipeline".

D-58. The canonical primitives section retains the existing
intelligence pointer and adds an explicit "foundry retired per
ADR-0335" note.

D-59. Catalog and registry entries pointing at foundry update to
either retired pointers or intelligence absorption pointers.

D-60. `microservices/foundry/PRD.md` is no longer live authority.

D-61. `microservices/foundry/ARCHITECTURE.md` is no longer live
authority.

D-62. `microservices/foundry/PHASE-01-FOUNDRY-FOUNDATION.md` is no
longer live authority.

D-63. `microservices/foundry/PHASE-02-FOUNDRY-DATA-SUBSTRATE-ADDENDUM.md`
is no longer live authority.

D-64. `microservices/foundry/manifest.json` is not the active
µservice manifest after this ADR.

D-65. The foundry IP namespace (IP-001..IP-097 + IP-journey-* +
IP-WASMTIME-*) is historical; future IP authoring lands in the
intelligence IP namespace with renumbering as needed.

### D-66..D-75. Self-modification preserved

D-66. ADR-0247 doctrine remains in force.

D-67. Self-modification under the `oyatie.foundry.*` Cedar principal
namespace remains the operating model for code-modifying workflows.

D-68. The dev-tools cell topology referenced in ADR-0247 remains
valid; cells are infrastructure topology per ADR-0333, not a service.

D-69. Workflow libraries that previously lived under "Foundry"
framing live under "intelligence pipeline" framing when execution is
AI-substrate-bound, or under "oyatie.foundry workflow library"
framing when execution is self-modification-bound.

D-70. The webhook-driven agent invocation pattern of ADR-0112
remains valid; the receiver lives inside intelligence (or workflow,
depending on the surface) instead of a retired foundry µservice.

D-71. The VCS-orchestrator end-to-end pattern of ADR-0113 remains
valid; the orchestrator lives across vcs-orchestrator + intelligence
+ workflow, not inside a retired foundry µservice.

D-72. The webhook-receiver kernel (RETIRED per ADR-0363)
was transition debt per D-37; its substance is preserved for future
relocation to intelligence.

D-73. The changeset-state-machine, admission-gate, merge-queue, and
completion-gate substrates described under `microservices/foundry/spec/`
remain doctrine for the agentic pipeline; the doctrine lives in ADRs
0110, 0111, 0112, 0113, 0116, 0247, and 0255.

D-74. The agentic pipeline doctrine is independent of the retired
µservice boundary.

D-75. The agentic pipeline doctrine continues to be implemented by
the cross-cutting substrate µservices (vcs-orchestrator, intelligence,
workflow, audit-chain, observability, identity, tenancy,
policy-engine).

### D-76..D-85. Strangler discipline

D-76. ADR-0138 strangler discipline applies.

D-77. Because the µservice retires before launch, the retirement
uses the zero-current-consumer variant per ADR-0333 D-70.

D-78. The zero-current-consumer variant keeps a redirect marker and
removes live authority.

D-79. The redirect marker is enough because no production caller is
being migrated.

D-80. Cross-reference sweeps must route old paths to absorption
targets.

D-81. Historical forensic mentions may survive only when clearly
marked historical.

D-82. Machine-readable specs must not list foundry as an active
µservice after this ADR.

D-83. Counts that included foundry as an active µservice must be
corrected when touched.

D-84. New ADRs must cite intelligence (not foundry) for AI substrate
ownership.

D-85. New IPs targeting AI substrate capabilities must land under
`microservices/intelligence/IP-*`.

## Absorption Map

| Retired responsibility | Successor owner | Successor authority |
|---|---|---|
| Agent runtime (capability executor, session state, invocation orchestrator, runtime pod pool, capability registry cache) | intelligence | `microservices/intelligence/manifest.json#bounded_contexts.model-routing` + `dispatch-flow` usecase |
| Supervisor (fleet lifecycle, capability deployment, kill-switch + circuit-breaker, autonomy policy enforcement, supervision event bus) | intelligence | `microservices/intelligence/manifest.json#bounded_contexts.guardrails` + observability + audit-tap |
| Eval (eval runner, parity analyzer, replay engine, golden-output store) | intelligence | `microservices/intelligence/manifest.json#bounded_contexts.eval` |
| Evidence (capability-invocation recorder, evidence pack builder, regulator export, audit-chain bridge) | intelligence | `microservices/intelligence/manifest.json#bounded_contexts.audit-tap` |
| Guardrails (prompt classifier, output validator, autonomy-tier gate, content-safety rule engine, jailbreak detector, AI-slop detector) | intelligence | `microservices/intelligence/manifest.json#bounded_contexts.guardrails` |
| Providers (LLM provider router + 8 adapters + OpenBao credential isolation) | intelligence | `microservices/intelligence/manifest.json#bounded_contexts.providers` (already enumerates 18 first-class providers) |
| Credential isolation | intelligence | `microservices/intelligence/manifest.json#bounded_contexts.credential-resolver` (already wired to OpenBao per ADR-0296) |
| Attribution | intelligence | `microservices/intelligence/manifest.json#bounded_contexts.attribution` |
| Brand UX surface | intelligence | `microservices/intelligence/manifest.json#bounded_contexts.brand-ux-surface` |
| Webhook-driven invocation (ADR-0112) | intelligence + workflow | dispatch-flow + workflow event bus |
| VCS orchestrator end-to-end (ADR-0113) | vcs-orchestrator + intelligence + workflow | distributed across substrate owners |
| Changeset state machine (ADR-0110) | vcs-orchestrator | `microservices/vcs-orchestrator/` |
| Merge queue projected state (ADR-0111) | vcs-orchestrator | `microservices/vcs-orchestrator/` |
| Admission gate policy + evidence | vcs-orchestrator + policy-engine + audit-chain | distributed across substrate owners |
| Completion gate (reviewer + CI) | vcs-orchestrator + observability | distributed across substrate owners |
| Self-modification principal namespace `oyatie.foundry.*` | identity + policy-engine | Cedar principal authority per ADR-0247 |
| HG-FOUNDRY hyperscaler-grade conformance gate | intelligence | folds into HG-INTELLIGENCE per ADR-0255 |

## Successor Contract

C-1. Intelligence is the AI substrate writer.

C-2. Intelligence is the dispatch surface for AI workflows.

C-3. Intelligence is the eval owner for AI substrate regression.

C-4. Intelligence is the guardrails owner for AI refusal correctness.

C-5. Intelligence is the audit-tap owner for AI evidence sealing.

C-6. Intelligence is the provider-adapter owner.

C-7. Intelligence is the credential-resolver owner.

C-8. Intelligence is the attribution owner.

C-9. Intelligence is the brand-UX-surface owner.

C-10. Identity carries the signed principal context including
`oyatie.foundry.*` self-modification principals.

C-11. Policy-engine carries the Cedar corpus for AI substrate
decisions and self-modification authorization.

C-12. Tenancy persists tenant-class and pack pinning for AI workloads.

C-13. Cloud-iac provisions the intelligence workload and its model
storage.

C-14. Observability owns AI substrate SLO burn under the intelligence
labels.

C-15. Audit-chain seals AI dispatch, refusal, BYOK rotation, and
substrate evidence per the intelligence audit-tap.

C-16. Api-gateway routes AI traffic to the intelligence cell-aware
routes.

C-17. Workload µservices consume AI events from the intelligence event
namespace, not a retired foundry namespace.

C-18. No workload µservice calls a retired foundry endpoint.

C-19. No workload µservice infers AI ownership from a stale foundry
crate.

C-20. The only approved AI substrate kernel surface is under the
intelligence workspace.

C-21. The only approved AI substrate contract surface is intelligence
OpenAPI, AsyncAPI, and proto.

## Consequences

Retiring `microservices/foundry/` and absorbing it into `microservices/intelligence/` means the AI substrate, model ownership, and related responsibilities move to intelligence as the canonical owner rather than a standalone foundry service; the data-model, operational, and migration consequences are enumerated in the sections below.

## Data Model Consequences

M-1. `DispatchRequest` is the canonical AI substrate envelope per
intelligence IP-001..IP-010.

M-2. `RefusalDecision`, `RoutingDecision`, `EvalRecord`, and
`Attribution` are the canonical domain types.

M-3. `oyatie.foundry.*` is a Cedar principal namespace, not a service.

M-4. Audit evidence may carry `intelligence_session_id` plus
`oyatie_foundry_workflow_id` when self-modification execution is in
scope.

M-5. Metrics aggregate by intelligence labels; no `foundry` label is
emitted by new code.

M-6. Dashboards include AI dispatch health, refusal-rate burn,
first-token latency, streaming throughput, provider latency, BYOK
rotation, eval correctness, and audit-emission success — all under
intelligence labels.

M-7. Public APIs expose AI capabilities through intelligence
endpoints with the v1 contract surface.

M-8. OpenTofu modules for the intelligence workload subsume the
modules previously planned under foundry.

M-9. Cedar context includes `tenant_class`, `pack_set`, `audience`,
`actor`, and `intelligence_capability` (no foundry-specific context).

## Operational Consequences

O-1. AI dispatch incidents are intelligence runbook concerns.

O-2. AI refusal-rate burn is observability + intelligence concern.

O-3. AI provider circuit-breaker is intelligence concern.

O-4. AI BYOK key rotation is intelligence concern.

O-5. AI eval regression is intelligence concern.

O-6. AI substrate audit-emission failure is audit-chain + intelligence
concern.

O-7. Self-modification incidents follow ADR-0247 dev-tools-cell-N
runbook ownership.

O-8. The retired foundry runbook set is preserved as historical
evidence; new runbooks land under intelligence.

O-9. The retired foundry dashboard set is preserved as historical
evidence; new dashboards land under intelligence.

O-10. The retired foundry SLO set is preserved as historical
evidence; new SLOs land under intelligence.

## ADR Preservation

P-1. ADR-0255 remains active and authoritative for the intelligence
two-layer substrate.

P-2. ADR-0247 remains active and authoritative for self-modification.

P-3. ADR-0220 remains active; its scope statement that foundry is
internal-only is now historical context, because the µservice is
retired.

P-4. ADR-0239 amendment remains active as historical context; its
operative split now resolves to "intelligence owns AI; self-modification
runs as `oyatie.foundry.*` principals inside the `oyatie` tenant".

P-5. ADR-0136 + ADR-0136-amendment remain active as historical
context; the 6→1 consolidation precedent stands.

P-6. ADR-0138 strangler discipline remains active and is the
operating retirement pattern.

P-7. ADR-0112 + ADR-0113 + ADR-0116 remain active for the agentic
pipeline doctrine.

P-8. ADR-0132 no-grouping policy remains active; intelligence is
single-concern with a two-layer internal shape that does not violate
the policy.

P-9. ADR-0245 substrate-vs-product layering remains active.

P-10. ADR-0211 in-house tech stack policy remains active; intelligence
is Class C in-house mandatory.

## Rejected Alternatives

R-1. Keep `foundry` standalone.

R-2. Rejected because the bounded contexts overlap with intelligence.

R-3. Rejected because two AI µservices duplicate the same substrate.

R-4. Rejected because ADR-0255 explicitly absorbs foundry into
intelligence.

R-5. Rename `foundry` to `intelligence-foundry` as a sub-µservice.

R-6. Rejected because ADR-0132 forbids sub-µservice composition; the
intelligence µservice carries its own two-layer internal shape.

R-7. Move foundry into a hypothetical `agent-platform` µservice.

R-8. Rejected because the AI substrate is intelligence, and
self-modification is a Cedar principal namespace, not a separate
service.

R-9. Keep `foundry/PRD.md` as a live reference.

R-10. Rejected because live old docs preserve an incorrect service
boundary.

R-11. Delete foundry evidence without a redirect.

R-12. Rejected because future agents need a deterministic retirement
pointer.

R-13. Rename all 122 `oya-foundry-*` crates to `oya-intelligence-foundry-*`
in this ADR.

R-14. Rejected because the rename cascade across 43 dependent crates
risks breaking `cargo check --workspace`; the rename is sequenced as
a separate cleanup wave per D-43.

R-15. Keep the "retired external agent harness" brand as a sub-namespace under intelligence.

R-16. Rejected because ADR-0247 D-10 + ADR-0328 D-9.22 explicitly
retire retired external agent harness as a canonical primitive.

R-17. Replace "retired external agent harness" with a new internal brand name.

R-18. Rejected because no replacement is needed; "intelligence pipeline"
or "oyatie.foundry workflow library" already covers every use case.

R-19. Move agentic pipeline doctrine ADRs (0110, 0111, 0112, 0113)
into the intelligence µservice tree.

R-20. Rejected because doctrine ADRs live in `docs/decisions/` and
are owned by council-architecture, not a specific µservice.

## Migration Plan

S-1. Author this ADR.

S-2. Replace active `microservices/foundry/` content authority with
`RETIRED.md`.

S-3. Rewrite `microservices/foundry/onboarding/pipeline-engineer-first-week.md`
to drop "retired external agent harness" or retire the file in place.

S-4. Rewrite `microservices/foundry/faqs/pipeline-engineer-faq.md`
to drop "retired external agent harness" or retire the file in place.

S-5. Rewrite `microservices/foundry/spec/*.md` files where they are
operative agentic-pipeline doctrine that should remain accessible;
otherwise retire in place with a pointer to the relevant ADR.

S-6. Update `microservices/intelligence/manifest.json` to declare
`absorbed_microservices: ["foundry"]` with this ADR as authority.

S-7. Update `specs/master-plan-sequencing.json` to mark foundry
retired-by-wave-15I.

S-8. Update `specs/microservices/foundry.json` as a retirement
marker.

S-9. Update `specs/microservices/manifests-index.json` to update the
foundry pointer.

S-10. Update `specs/root-hub-pointers.json` to mark `prd_foundry`
retired with absorption pointer.

S-11. Update `tools/hooks/_canonical-primitives.md` AI Substrate
section: foundry retired; intelligence is the single AI surface; drop
the retired external agent harness line.

S-12. Sweep corpus-wide for "retired external agent harness" references and rewrite per
D-26..D-36.

S-13. Update memory note
`feedback_intelligence_two_layer_substrate.md` with the Wave 15I
retirement addendum.

S-14. Verify `cargo check --workspace` remains green.

S-15. Verify no active docs/specs references still route readers to
`microservices/foundry/` as a live AI surface.

S-16. Report any remaining historical references or validation gaps.

## Verification

V-1. `microservices/foundry/RETIRED.md` exists.

V-2. `microservices/foundry/RETIRED.md` cites this ADR.

V-3. `microservices/intelligence/manifest.json` declares the absorbed
scope.

V-4. `specs/microservices/foundry.json` is a retirement marker.

V-5. `specs/microservices/manifests-index.json` does not list foundry
as an active manifest pointer (or marks it retired with absorption
pointer).

V-6. `specs/master-plan-sequencing.json` does not list foundry in the
active µservice phase roster, and Wave 15I records the retirement.

V-7. `specs/root-hub-pointers.json` `prd_foundry` is marked retired
with absorption pointer.

V-8. `tools/hooks/_canonical-primitives.md` AI Substrate section
names intelligence only; drops the retired external agent harness line.

V-9. Active docs and specs cross-reference sweep points to successor
owner intelligence.

V-10. Historical retired external agent harness mentions remain only in ADRs documenting the
retirement (this ADR, ADR-0136-amendment, ADR-0220, ADR-0239,
ADR-0247, ADR-0211, ADR-0245, ADR-0253-amendment, ADR-0328) plus
memory files.

V-11. `cargo check --workspace` exits 0.

V-12. ADR-0255 doctrine remains in force.

V-13. ADR-0247 doctrine remains in force.

V-14. ADR-0132 doctrine remains in force.

V-15. ADR-0138 strangler discipline remains in force.

V-16. No commit is created by this wave.

## Completion Report

The completion report is embedded as an HTML comment so automated
readers can parse the ADR without changing the visible decision text.

<!--
wave: 15I
status: completed-locally
decision: foundry µservice retired; AI substrate absorbed into intelligence; retired external agent harness terminology dropped corpus-wide
absorbing_microservice: microservices/intelligence/
retired_marker: microservices/foundry/RETIRED.md
absorption_map_owner: microservices/intelligence/manifest.json
prd_owner: microservices/intelligence/PRD.md
manifest_owner: microservices/intelligence/manifest.json
precedent_waves: Wave 15K network→community; Wave 15L cell retire; Wave 15O shorts→social
authority_adrs: ADR-0255 KS#14 intelligence two-layer; ADR-0247 self-modification; ADR-0220 consumer intelligence; ADR-0239 foundry internal-only amendment; ADR-0132 no-grouping; ADR-0138 strangler; ADR-0328 D-9.18/D-9.22 wave-15I sequencing
retired_brand_drop_authority: ADR-0247 D-10; ADR-0328 D-9.22 + D-12.22..D-12.24; ADR-0619
crate_transition_debt_policy: ADR-0333 D-59 precedent; existing oya-foundry-* crates retained as transition debt; future rename in separate cleanup wave
commits: none
-->
