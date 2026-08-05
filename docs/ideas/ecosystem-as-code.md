# Ecosystem-as-Code (EaC)

> Status: unratified north-star vision (2026-07-02). Zero authority until founder
> ratification. The eventual apex ADR **amends ADR-0516** (same mechanic ADR-0516
> used on ADR-0515): the Agentic Delivery Fabric becomes *the delivery-plane
> instance of EaC and remains governing for delivery*. Crystallized from deep
> interview `interview_20260702_150957` (final ambiguity 12%, threshold 20%).

## Problem Statement

How might every party in the oyatie ecosystem — oyatie itself, tenant companies,
and their users — express, run, and govern their entire operations (infra,
policy, pipelines, config, monitoring, security, data, approvals, supply-chain
relationships) as versioned code artifacts, while the daily work happens on a
visual, ontology-backed, AI-assisted canvas that non-technical people can
actually use?

## Thesis

Every operational concern of the ecosystem is a versioned, schema'd, declarative
artifact on the **versioned truth substrate** (git today; owned AST-SCM /
corpus-graph destination per ADR-0518 + the corpus governance substrate), and it
**translates to APIs cloud-natively**: reconcilers project truth one-way into
declarative API resources (CRD + reconciler doctrine, ADR-0349), matching the
K8s-native, API-first, CLI-retirement posture. Every human or AI surface —
canvas, console, agent — is a **client** of that truth: the only write path is a
governed commit/PR, drift is detected but never written back, and the canvas
feels live through optimistic local state while nothing is true until committed.

Three planes, one truth: **truth plane** (everything-as-code) · **semantic
plane** (ontology) · **experience plane** (canvas + intelligence).

Authoring is tri-modal and modality-equal: **as-code** (hand-edited artifacts),
**no-code** (canvas), and **agentic** are first-class peers. The agentic
modality is agents wielding the same code and canvas surfaces to assist users —
Zapier/n8n-class automation, but over governed truth instead of a proprietary
model. Any artifact authorable in one modality is authorable in the others, all
converging on the same governed commit as **byte-identical canonical
serializations** after normalization. No canvas-only state, no agent-only
state, no code-only capability — parity is structural: the canvas curates
ergonomic UX for the hot paths and covers every remaining schema field with a
schema-generated form, so nothing expressible as-code is unreachable visually.

## What's genuinely new

One claim carries the document: **tenant-authored operations through a no-code /
AI projection over code truth.** ServiceNow-, Salesforce-Flow-, and Retool-class
products are canvas-owns-model — proprietary truth, unreviewable, unprovable.
Here every canvas gesture materializes as a governed, diffable, versioned,
policy-gated artifact: no-code a compliance officer can trust because fail-closed
policy and provable artifacts sit underneath every click. The no-code surface
must be ergonomic for non-technical users (compliance officers, ops managers) —
not engineers wearing a visual skin.

Everything else composes existing canon by name:

- **Semantic plane** = ADR-0059 Ontology µservice (Object/Link/Action Types) +
  the G004 unified PBAC/ReBAC authz engine + the pack schema registry as an
  Ontology object family. No third ontology.
- **Experience plane** = `oya/workflow-studio` evolution under the
  console-greenfield doctrine (multi-platform, spatial). Not unowned greenfield.
- **Runtime workflows** target the ADR-0035 hybrid state-machine + DAG engine
  (per-tenant versioning, jurisdiction overlays, agent-authored steps) — still
  status:proposed; ratification is a prerequisite for any executable-workflow
  wedge.

## Proving wedge

**Policy-pack editor + simulator over billing statutory rulepacks** — already
founder-refined in [policy-pack-substrate](policy-pack-substrate.md) (authoring
ladder rung 2 demands exactly this editor), and it exercises all three planes
end-to-end: packs are truth-plane artifacts, the pack registry is the semantic
plane, the editor is the experience plane, and SMT-analyzable packs give
intelligence a provable authoring target.

Next candidates, selected on wedge-1 exit evidence (one line, not pinned):
tenant approval workflow drawn on canvas executing on the ADR-0035 engine;
cross-company supply-chain data intake, fail-closed through the policy engine.

## Authoring ladder

Internal dogfood (ADR-0242: oyatie-is-a-tenant) → tenant admin →
plugin-app-store distribution (ADR-0213 / ADR-0534; "marketplace" is reserved
brand-layer vocabulary) → **cross-company federation** — the genuinely novel
rung: operations, policies, and data flows spanning trust domains, stacked over
the ReBAC relationship graph, always fail-closed through the policy engine.

## Not doing (and why)

- **True live write-back without a commit boundary** — every prior art
  (Terraform drift, ArgoCD manual edits) retreated from it; it silently mints a
  second truth plane.
- **A new apex umbrella above ADR-0516** — supersession in costume over a
  one-way door; amendment is the honest mechanic.
- **A third ontology / rival truth substrate** — anti-sprawl; compose ADR-0059
  and the corpus graph.
- **Roadmap re-sequencing now** — doc-only; current W1 security/reorg work
  already builds the substrate (packs, ReBAC, corpus graph). The wedge rides
  the already-planned pack authoring ladder.

## Key assumptions to validate

- [ ] A non-technical compliance author can complete a real statutory pack
      change through the editor + simulator without touching YAML — validate in
      wedge 1 with one real KR statutory change.
- [ ] One-way reconciler projection + optimistic canvas state delivers a
      "live"-feeling canvas at acceptable latency — validate in the wedge-1
      editor before generalizing the experience plane.
- [ ] The pack schema registry can express the ontology objects the canvas
      needs (entities, relationships, actions) without forking ADR-0059 —
      validate by modeling the wedge-1 editor's node types as Ontology objects.
- [ ] Modality parity holds in practice: the same wedge-1 pack change can be
      authored as-code, on the canvas, and by an agent (with pre-promotion
      proof), landing as byte-identical canonical serializations — validate
      all three paths in wedge 1, including the schema-generated fallback form
      for fields outside the curated UX.

## Open questions

1. **Multi-tenant truth hosting (top, load-bearing):** tenants do not have git.
   Isolation, authz-on-truth-objects, and federation for *tenant* truth have no
   accepted ADR — this substrate underlies all three planes.
2. **Owned Policy IR vs Cedar dialect** (carried from policy-pack-substrate).
3. **Pack-registry / ontology home** (mirrors the open authz/ vs iam/ question).

## Prior art & supersession verdicts

| Artifact | Verdict |
|---|---|
| ADR-0516 agentic delivery fabric (apex, one-way) | **amends** — becomes EaC's delivery-plane instance, remains governing for delivery |
| ADR-0059 workflow + ontology adapter layer (Accepted) | **composes** — IS the semantic plane |
| G004 unified PBAC/ReBAC authz northstar | **composes** — relationship half of the ontology; PDP gates every flow |
| policy-pack-substrate.md (founder-refined 2026-07-02) | **composes** — wedge 1 is that doc's authoring-ladder rung 2 |
| ADR-0035 hybrid SM+DAG workflow engine (Proposed) | **defers** — ratification prerequisite for executable-workflow candidates |
| ADR-0242 oyatie-is-a-tenant | **composes** — dogfood rung |
| ADR-0213 / ADR-0249 / ADR-0534 plugin-app-store, naming, pack marketplace | **composes** — distribution rung + reserved vocabulary |
| ADR-0518 bespoke AST-SCM + corpus governance substrate | **composes** — truth-substrate destination; git is transitional |
| ADR-0349 GitOps CRD + reconciler doctrine | **composes** — the EaC→API translation mechanic |
| unified-ecosystem-thesis-2026-05-21 | **amends** — EaC is its sharpened successor framing |
