---
id: ADR-WS-0005
title: AI-copilot node-generation scope bounds (T0/T1/T2 capability tiers)
microservice: workflow-studio
status: Accepted
date: 2026-05-17
owner: axis-workflow + council-architecture
deciders: council-architecture, axis-workflow, ops-security, council-legal-compliance, axis-foundry-providers
supersedes: []
superseded_by: []
related: [ADR-0110, ADR-0131, ADR-0140]
related_specs: [/specs/products/workflow-studio.json]
related_artifacts:
  - microservices/workflow-studio/PRD.md (FR-12, AC-05, §"Open Questions" Q4)
  - microservices/workflow-studio/capabilities/T0-suggest.yaml
  - microservices/workflow-studio/capabilities/T1-assist.yaml
  - microservices/workflow-studio/capabilities/T2-auto.yaml
  - microservices/workflow-studio/IP-008-llm-assist-adapter.md
  - microservices/workflow-studio/dashboards/copilot-quality.json
purpose: Resolve the scope of the workflow-studio AI-copilot — which capability tiers (T0/T1/T2) it operates in, and specifically whether the copilot may generate DSL fragments that cross-call other µservices.
doc_status: published
---

# ADR-WS-0005: AI-copilot — T0 + T1 intra-µservice by default; T2 cross-µservice gated by Cedar + ChangeSet review

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

PRD §"Functional Requirements" FR-12 (Should-GA): "tenant developer drafts a workflow via prose; LLM emits candidate spec; I review." AC-05: "LLM-authored spec via API; valid: opens in editor; invalid: precise per-line error." PRD §"Open Questions" Q4 (gates IP-008): "LLM-assist invocation: stream-back-to-browser via WS vs server-side full-draft then send?" — that question is about *transport*; this ADR resolves the broader **scope** question that is implicit in the capability-tier files (`capabilities/T0-suggest.yaml`, `T1-assist.yaml`, `T2-auto.yaml`):

> **Should copilot be allowed to generate DSL fragments that cross-call other µservices, or only intra-workflow-studio?**

The capability tier framework is canonical for oyatie (per `feedback_quality_performance_scalability_bar.md` and the `/specs/products/*` capability tiers):

- **T0 — Suggest**: copilot surfaces authoring suggestions to the tenant; tenant decides. No autonomous action. Lowest-risk tier.
- **T1 — Assist**: copilot drafts a candidate spec / fragment; tenant reviews & accepts/rejects. Reversible; tenant is the actor.
- **T2 — Auto**: copilot acts autonomously (CRDT merge, editor session persistence, retry orchestration). Bounded by ChangeSet contract + Cedar policies.

Risk dimensions:

1. **EU AI Act (Regulation (EU) 2024/1689)** — applies extraterritorially. Arts. 9–15 (high-risk AI systems) imposes risk management, data governance, technical documentation, record-keeping, transparency, human oversight, accuracy/robustness, post-market monitoring obligations. Recital 65 lists "AI systems used in the area of [...] employment, workers management" as high-risk; tenant workflows in HR / finance / healthcare contexts can fall in scope. Arts. 50–55 (transparency) require labeling AI-generated content.
2. **GDPR Art. 22** — prohibits solely-automated decisions producing legal effects, with exceptions (consent, contract, law). T2 autonomous spec generation that affects tenant end-users can engage Art. 22; T1 leaves a human in the loop and side-steps the prohibition.
3. **EU DSA / DMA + sector regulations** — depending on tenant industry, additional bounds apply.
4. **Tenant trust boundary** — a copilot that generates DSL fragments calling foreign µservices effectively widens the tenant's blast radius without explicit consent. The tenant authorized "draft me a workflow"; not "draft me a workflow that integrates Slack + Salesforce + my mail µservice."
5. **Supply-chain risk** — LLM-emitted code-shaped artifacts have a documented prompt-injection attack surface (see OWASP LLM Top-10 LLM01: Prompt Injection, LLM07: System Prompt Leakage). Cross-µservice fragments multiply the attack surface.
6. **Tenant ownership of authored artifacts** — the tenant owns the workflow they authored. Copilot output blurs ownership when scope is broad; bounding scope makes the ownership boundary crisp.
7. **Audit + reversibility** — every copilot action must be reversible via the ChangeSet state machine (ADR-0110). Cross-µservice fragments compose multiple ChangeSets per draft, which complicates reversibility.

Existing controls in the µservice:
- Per-tenant LLM choice via foundry-providers (PRD §FR-12; `sdk-plan.md` BYO-LLM).
- PII redactor + prompt-injection scrub before LLM call (`competitor-parity-matrix.md` §"LLM-assist authoring" T-I-05, T-S-05).
- Schema-validated completion (LLM output passes through dsl-loader + Cedar preview before save).
- 90-day archival of prompts + completions for audit (PRD §"Audit + Compliance").

## Decision

Adopt the following capability-tier scope bounds for the workflow-studio AI-copilot:

### T0 — Suggest (always-on by default; per-tenant opt-out)

- Scope: intra-workflow-studio. Copilot surfaces inline authoring suggestions (suggested next node, suggested parameter, suggested connection) drawn from the tenant's own workflow history + the per-pack node library.
- Cross-µservice scope: **forbidden**. Suggestions reference only workflow-studio's own node library and the tenant's own workflows.
- Autonomy: zero. Suggestions are advisory; tenant explicitly accepts each one.
- Cedar gate: presence-check (tenant has `workflow-studio.copilot.t0.suggest` entitlement); no per-action policy.
- ChangeSet: each accepted suggestion is treated identically to a manual user edit; no separate ChangeSet branch.
- EU AI Act posture: not high-risk (purely suggestive, no autonomous output). Transparency obligation met by labelling suggestions as AI-generated in the UI.

### T1 — Assist (opt-in per-tenant; default off)

- Scope: intra-workflow-studio. Copilot drafts a candidate workflow_spec.v1 fragment from tenant prose ("when a sales lead exceeds $50k, route to senior AE"). Fragment may include workflow-studio's own node-library nodes only; may reference Ontology object types for typed-field configuration (read-only, via the ontology-sdk; this is intra-product because Ontology is the canonical info graph).
- Cross-µservice scope: **forbidden by default**. The copilot MUST NOT emit nodes that call foreign µservices (mail, messenger, calendar, foundry-providers, etc.) at the T1 tier. Attempting to emit such a node is a hard error in the LLM-assist adapter; the fragment is rejected and the user is shown a precise diagnostic explaining the bound.
- Autonomy: bounded. Copilot drafts; tenant explicitly reviews and accepts/rejects in the editor.
- Cedar gate: presence-check (`workflow-studio.copilot.t1.assist`) + per-call policy that asserts no cross-µservice node references. Cedar evaluation runs server-side per ADR-WS-0004 hybrid pattern.
- ChangeSet: each accepted T1 draft is one ChangeSet (per ADR-0110); reversible by the tenant via `oya vcs revert`.
- EU AI Act posture: potentially high-risk depending on tenant industry. Transparency (Art. 50) is satisfied by labeling. Human oversight (Art. 14) is satisfied by tenant-accept-before-save. Records (Art. 12) are satisfied by the 90-day prompt/completion archive. Risk management (Art. 9) is delegated to the per-tenant DPIA per `dpia.md`.

### T2 — Auto (opt-in per-tenant + per-capability; default off; cross-µservice gated)

- Scope: T2 has two sub-scopes:
  - **T2-intra**: workflow-studio's existing autonomous capabilities — CRDT op auto-merge, editor session auto-persist, retry orchestration on transient failures. These are the autonomous behaviours already on the canvas; they predate the copilot.
  - **T2-cross**: copilot draft of DSL fragments that include nodes calling other µservices. **This is the gated case**.
- T2-cross gate:
  1. Tenant must have explicitly enabled `workflow-studio.copilot.t2.auto.cross-microservice` for the specific destination µservice (mail, messenger, foundry-providers, etc.). Enablement is per-(tenant, destination-µservice) pair, not blanket.
  2. Cedar policy must permit the cross-call. Cedar entities include `(tenant, source-microservice=workflow-studio, destination-microservice, capability-tier=T2, copilot-origin=true)`. Default-deny per ADR-0140.
  3. The candidate spec MUST be routed through the ChangeSet state machine (ADR-0110) as a proposed-state ChangeSet that requires human-author + reviewer-agent approval before the foreign µservice's spec-binding is committed. This is the "review-required" hatch — T2-cross never auto-saves a cross-µservice fragment.
  4. The 2-person rule for cross-µservice copilot output: the LLM is the first author; a human OR another agent acting under a different signing key is the required reviewer. Same agent signing both author + reviewer roles fails the gate.
  5. The destination µservice's own SDK contracts apply (the workflow-engine SDK, the mail SDK, etc.); copilot output is bounded by what those SDKs already permit external callers to do.
- EU AI Act posture: T2-cross is the highest-risk tier; treat as **high-risk under EU AI Act Arts. 9–15 by default**. Tenant DPIA is mandatory before enablement. Post-market monitoring (Art. 72) is mandatory via the `copilot-quality.json` dashboard's `copilot_t2_cross_acceptance_rate`, `copilot_t2_cross_revert_rate`, `copilot_t2_cross_safety_signal` SLIs.

### Forbidden under all tiers

- Copilot emitting DSL fragments that bypass Cedar policy (e.g., a node that sets `cedar_check: skip`). Hard error in the adapter.
- Copilot emitting DSL fragments that touch fields marked `data_class = SECRET` without an explicit secrets-handling node + tenant consent. Hard error.
- Copilot emitting workflows that reference µservices that don't exist or that the tenant has no entitlement to. Hard error; precise diagnostic.
- Copilot output passing through to save without dsl-loader validation + canonical-form canonicalization (per ADR-WS-0002) + Cedar preview (per ADR-WS-0004). Hard error.

## Alternatives Considered

### Alternative A — T0 + T1 intra-µservice only; T2 entirely forbidden

Bound the copilot to suggestions and intra-product drafts; no autonomous cross-µservice action at all.

- **Pros**
  - Maximally safe; smallest blast radius.
  - Trivially compliant with EU AI Act Art. 22 (human-in-loop always).
  - Simplest implementation; no Cedar plumbing for cross-µservice copilot calls.
- **Cons**
  - Falls behind competitors that already offer cross-product Copilot (Microsoft Power Automate Copilot draws data from Microsoft Graph; Zapier AI references multiple Zap apps).
  - The "draft me a workflow that emails the senior AE" is the obvious low-friction tenant ask; refusing it limits product differentiation.
  - The capability-tier framework anticipates T2 use (per `capabilities/T2-auto.yaml`); forbidding it entirely would require revoking that capability.
- **Rejected reason**: too restrictive for the hero-product ambition. T2-cross with strict gates is the right balance.

### Alternative B — Unlimited cross-µservice copilot at T1 (no Cedar/ChangeSet gate)

Allow the copilot to freely emit fragments that call any µservice the tenant has access to, at T1.

- **Pros**
  - Maximal copilot capability; "draft me a workflow that does X, Y, Z" works without per-destination configuration.
  - Competitive with the freest interpretation of competitor Copilot offerings.
- **Cons**
  - Widens tenant blast radius unconsciously; tenant authorized "draft a workflow" without thinking through all the destination µservices the LLM might choose.
  - Prompt-injection risk multiplies; a poisoned prompt could craft a fragment that exfiltrates data via cross-product calls (e.g., reads a customer table, sends it to an attacker-controlled webhook).
  - EU AI Act Arts. 9–15 high-risk posture for any tenant in regulated industries; default-on cross-product copilot is hard to defend in a regulatory audit.
  - Reversibility is more complex when a single draft spans multiple µservices' ChangeSets.
- **Rejected reason**: blast radius + supply-chain attack surface + EU AI Act high-risk default. The marginal capability gain over the chosen design is small; the marginal risk is large.

### Alternative C — T2 cross-µservice copilot with Cedar but no ChangeSet review (auto-commit on Cedar pass)

Cedar gates cross-µservice copilot output but, on Cedar pass, the fragment auto-commits without explicit ChangeSet review.

- **Pros**
  - Lower friction than the chosen design.
  - Cedar policies are the trust boundary; "if Cedar said yes, why force human review?"
- **Cons**
  - Cedar policies enforce structural rules (who can call what); they do not enforce semantic correctness (the LLM's *interpretation* of the tenant ask).
  - Without ChangeSet review, an LLM-misinterpreted prose ask gets auto-deployed; the tenant discovers it at run-time.
  - EU AI Act Art. 14 human oversight obligation is harder to defend without a documented human-review step.
- **Rejected reason**: Cedar is necessary but not sufficient for T2-cross safety. The ChangeSet review is the human-oversight hatch the EU AI Act expects.

### Alternative D — Copilot intra-µservice only; cross-µservice ALWAYS gated via Workflow + Ontology adapter explicitly composed by tenant

The copilot stays strictly intra-product; if tenant wants cross-µservice composition, they must compose it via the canonical Workflow + Ontology adapter pattern (per `feedback_workflow_objectgraph_adapter_layer.md`) — composition is a separate manual step.

- **Pros**
  - Honours `feedback_workflow_objectgraph_adapter_layer.md` strictly — all inter-product flows go through Workflow + Ontology.
  - Architecturally clean.
- **Cons**
  - Tenant UX friction: most cross-product asks come naturally in prose ("when a customer signs up, send them an email"); forcing manual adapter composition adds an authoring step.
  - The tier framework's T2 capability slot is left mostly unused.
- **Note**: The chosen Decision actually preserves this architectural property — copilot-emitted cross-µservice fragments STILL flow through the Workflow + Ontology adapter pattern at run-time; the copilot drafts the adapter composition, but it's still composed-through-Workflow+Ontology. The chosen design adds the tier gate on top of that adapter pattern, not in place of it.
- **Rejected reason** (as a standalone alternative): unnecessarily restrictive; the chosen design preserves the architectural invariant via gated T2-cross rather than forbidding the use-case outright.

## Consequences

### Architectural

- The `oya-workflow-studio-visual-canvas-adapter` extension for LLM-assist (per PHASE-01 IP-008) implements the tier-aware adapter:
  - T0/T1 path: bounded to workflow-studio's node library + Ontology reads.
  - T2-cross path: Cedar entity construction + ChangeSet draft creation + reviewer-agent invocation + 2-person rule enforcement.
- The dsl-loader rejects LLM-emitted fragments that violate tier scope; this is a structural enforcement (not policy-based) for defence-in-depth.
- The LLM-assist adapter exposes `LlmAssistDraft{tenant, draft_id, prompt_hash, completion_hash, accepted_at, tier, cross_microservice_destinations[]}` as the Ontology object type (per PRD §"Ontology writes"); the `tier` and `cross_microservice_destinations` fields are queryable for compliance reporting.

### Downstream impact on other µservices and IPs

1. **IP-008 (LLM-assist adapter)** — implements the tier scope bounds; integration tests for each forbidden-output case (cross-µservice at T1, secrets-touch without consent, Cedar bypass, no dsl-loader validation).
2. **foundry-providers µservice** — LLM-call envelope carries the tier + scope; foundry-providers does not enforce the tier itself (that's workflow-studio's job) but archives the scope for cross-product audit.
3. **tenancy µservice** — per-tenant entitlements for `copilot.t0`, `copilot.t1`, `copilot.t2.auto.intra`, `copilot.t2.auto.cross-microservice.<destination>` are issued and revocable.
4. **workflow-engine µservice** — engine spec-store accepts T1-accepted drafts as ordinary spec submissions; T2-cross drafts arrive via the ChangeSet state machine, not directly.
5. **All destination µservices** for T2-cross (mail, messenger, calendar, social, shorts, network, anonymous, foundry-providers, etc.) — each ships a Cedar policy fragment defining whether they accept copilot-originated cross-calls at T2 + which capabilities. Each destination µservice's owning team owns that decision.
6. **observability µservice** — `copilot-quality.json` dashboard gains `copilot_t1_drafts_per_tenant_per_hour`, `copilot_t2_cross_drafts_per_tenant_per_hour`, `copilot_t2_cross_acceptance_rate`, `copilot_t2_cross_revert_rate`, `copilot_t2_cross_safety_signal` (any high-risk classification trigger), `copilot_eu_ai_act_high_risk_invocation_count`.
7. **council-legal-compliance** — tenant-side DPIA template + per-tenant T2-cross enablement workflow per EU AI Act post-market monitoring (Art. 72) authored.

### SLOs and CI lanes affected

- `oya-governance-cedar-preview-required` — exercised for every copilot output (T0/T1/T2).
- `oya-governance-editor-execution-forbidden` — copilot output never executes; only emits.
- `workflow-studio.copilot_t2_cross_safety_signal_count` — Sev-1 if non-zero in any 24h window (any signal of high-risk classification triggers manual review).
- `workflow-studio.copilot_forbidden_output_blocked_count` — informational SLI; non-zero is normal (the adapter is doing its job), but a sudden spike correlates to prompt-injection attempts.
- `workflow-studio.copilot_t2_cross_acceptance_rate` — quality SLI; low rate signals copilot quality issues.
- `workflow-studio.copilot_t2_cross_revert_rate` — quality SLI; high rate signals the human-review hatch is catching too many bad drafts.

### Compliance + audit

- Every copilot invocation emits `LlmAssistDraftRequested` (PRD §"Workflow events produced") with tier + scope.
- 90-day archival of prompt + completion + tier + scope + Cedar decision per PRD §"Audit + Compliance".
- T2-cross drafts emit additional `LlmAssistT2CrossDraftReviewerAssigned` and `LlmAssistT2CrossDraftReviewed{verdict, reviewer_identity}` events.
- EU AI Act post-market monitoring report quarterly to council-legal-compliance summarising T2-cross usage + safety signals + revert rate.
- DPIA (per `microservices/workflow-studio/dpia.md`) updated to reflect the tier-aware copilot scope.

### Risk register

- **Risk**: T2-cross gate fatigue — tenant disables ChangeSet review to speed things up. **Mitigation**: per-tenant Cedar entitlement is granular per destination; tenant can grant blanket consent for low-risk destinations only via an explicit allowlist authored under `microservices/workflow-studio/specs/`.
- **Risk**: prompt-injection succeeds at emitting an out-of-scope fragment. **Mitigation**: structural rejection in dsl-loader + Cedar preview; SLO on forbidden-output-blocked-count.
- **Risk**: EU AI Act classification evolves; current T1 posture becomes high-risk. **Mitigation**: quarterly compliance review (per ADR-0133 axis-4); ADR-WS-0005 supersession is the upgrade path.
- **Risk**: tenant LLM (BYO-LLM) emits non-deterministic output; copilot drafts vary turn-to-turn. **Mitigation**: schema-valid completion enforced; non-conforming output is rejected (AC-05); telemetry on rejection rate per LLM.

## References

- PRD `microservices/workflow-studio/PRD.md` FR-12, AC-05, §"Open Questions" Q4, §"Audit + Compliance", §"Workflow events produced", §"Ontology writes".
- `microservices/workflow-studio/capabilities/T0-suggest.yaml`, `T1-assist.yaml`, `T2-auto.yaml`.
- `microservices/workflow-studio/IP-008-llm-assist-adapter.md`.
- `microservices/workflow-studio/dpia.md`.
- `microservices/workflow-studio/compliance.md`.
- `microservices/workflow-studio/dashboards/copilot-quality.json`.
- `microservices/workflow-studio/competitor-parity-matrix.md` §"LLM-assist authoring", §"Forbidden claims".
- Regulation (EU) 2024/1689 of the European Parliament and of the Council of 13 June 2024 (EU AI Act) — Arts. 9, 10, 11, 12, 13, 14, 15, 50, 72; Recital 65. Official Journal of the European Union, 12 July 2024. `eur-lex.europa.eu/legal-content/EN/TXT/?uri=OJ:L_202401689`.
- Regulation (EU) 2016/679 (GDPR) Art. 22 — automated individual decision-making.
- OWASP Top 10 for Large Language Model Applications — `owasp.org/www-project-top-10-for-large-language-model-applications/`.
- ADR-0140 — Cedar policy enforcement.
- ADR-0110 — ChangeSet state machine (review + reversibility contract).
- ADR-WS-0002 — DSL canonical form (LLM output canonicalization).
- ADR-WS-0004 — Jurisdiction overlay renderer (Cedar preview contract).
- `feedback_workflow_objectgraph_adapter_layer.md` — Workflow + Ontology adapter pattern.
