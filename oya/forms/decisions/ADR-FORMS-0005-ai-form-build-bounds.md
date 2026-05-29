---
id: ADR-FORMS-0005
title: AI-form-build capability tier bounds (T0/T1/T2-intra/T2-cross); EU AI Act Annex III §4 high-risk classification
microservice: forms
status: Accepted
date: 2026-05-17
owner: axis-forms + council-architecture + council-privacy + council-legal-compliance
deciders: council-architecture, axis-forms, ops-security, council-legal-compliance, council-privacy, foundry-providers-team
supersedes: []
superseded_by: []
related: [ADR-0110, ADR-0131, ADR-0140 (retired per ADR-0145), ADR-WS-0005, ADR-FORMS-0001, ADR-FORMS-0004]
related_specs: [/specs/microservices/forms.json]
related_artifacts:
  - microservices/forms/PRD.md FR-22 + AC-25
  - microservices/forms/capabilities/T0-suggest.yaml
  - microservices/forms/capabilities/T1-assist.yaml
  - microservices/forms/capabilities/T2-auto.yaml
  - microservices/forms/dashboards/ai-form-build-quality.json
  - microservices/forms/runbooks/ai-form-build-rollback.md
  - microservices/forms/dpia.md
doc_status: published
---

# ADR-FORMS-0005: AI-form-build — T0 + T1 intra-µservice by default; T2-cross gated by Cedar + ChangeSet review + 2-person rule; Annex III §4 high-risk classification

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

Tenants ask for "build me a form from a sentence" (PRD FR-22). The natural product expression of this is AI-form-build: tenant prose → candidate form definition → tenant reviews + accepts. This capability is structurally similar to workflow-studio's AI-copilot (ADR-WS-0005); we inherit the tier-aware framework and the EU-AI-Act-grounded boundary analysis, and we add forms-specific risk factors:

1. **Annex III §4 (EU AI Act 2024/1689)** — explicitly lists "AI systems intended to be used [...] (a) to recruit or select natural persons, in particular to place targeted job advertisements, to analyse and filter job applications, and to evaluate candidates [...] (b) to make decisions affecting terms of work-related relationships [...]" as **high-risk AI systems**. A form authored by AI that is then used for hiring screening engages Annex III §4 obligations:
   - Risk-management system (Art. 9).
   - Data + data governance (Art. 10).
   - Technical documentation (Art. 11).
   - Record-keeping (Art. 12).
   - Transparency (Art. 13).
   - Human oversight (Art. 14).
   - Accuracy + robustness + cybersecurity (Art. 15).
   - Post-market monitoring (Art. 72).
   - Conformity assessment + CE-marking (Arts. 43, 47).
2. **GDPR Art. 22** — solely-automated decisions producing legal effects (employment, credit, insurance) are restricted; T1 human-in-loop sidesteps this; T2 must preserve human review.
3. **OWASP LLM Top-10**: prompt-injection (LLM01), insecure output (LLM02), training-data poisoning (LLM03), system-prompt leakage (LLM07).
4. **Per-form data-class declaration**: LLM must correctly infer `data_class` for fields it emits; misclassification (e.g., emitting `data_class=NORMAL` for a name field) results in PII not being column-encrypted at write time.
5. **Tenant trust boundary**: tenant authorised "draft me a form"; not "draft me a form that emails on submit using my mail µservice OR exports to my drive µservice OR triggers a workflow that spends my budget" — cross-µservice scope must be explicit + gated.

Tier framework inherited from `feedback_quality_performance_scalability_bar.md` + ADR-WS-0005:
- **T0 — Suggest**: AI surfaces authoring suggestions; tenant decides. No autonomous action. Lowest-risk.
- **T1 — Assist**: AI drafts a candidate; tenant reviews & accepts. Reversible; tenant is the actor.
- **T2 — Auto**: AI acts autonomously (within bounded operations). Default-deny + Cedar gate + ChangeSet review.

## Decision

Adopt the following tenant-class scope bounds for the forms AI-form-build:

### T0 — Suggest (always-on by default; per-tenant opt-out)

- Scope: intra-forms-µservice. AI surfaces inline authoring suggestions (suggested next field, suggested data_class, suggested branching predicate, suggested WCAG fix, suggested DPIA prompt).
- Cross-µservice scope: **forbidden**. Suggestions reference only forms' own field library + ontology Form entity bindings.
- Autonomy: zero. Suggestions advisory; tenant explicitly accepts each.
- Cedar gate: presence-check (`forms.t0.suggest` entitlement); no per-action policy.
- ChangeSet: each accepted suggestion = manual user edit; no separate ChangeSet branch.
- EU AI Act posture: not high-risk (purely suggestive). Transparency met by labeling suggestions as AI-generated in the UI.

### T1 — Assist (opt-in per-tenant; default off)

- Scope: intra-forms-µservice. AI drafts a candidate form.v1 fragment from tenant prose ("build me a customer feedback form with 1-5 scale + comments + email"). Fragment includes forms' own field types + Ontology Form entity binding; may reference workflow-engine workflow-trigger destination ONLY if tenant explicitly enables (see T2-cross).
- Cross-µservice scope: **forbidden by default** at T1. The AI MUST NOT emit fields that bridge to foreign µservices (mail/messenger/drive/payment) at T1. Attempting to emit such a field is a hard error in the dsl-loader.
- Autonomy: bounded. AI drafts; tenant explicitly reviews + accepts in the builder.
- Cedar gate: presence-check (`forms.t1.assist`) + per-call policy that asserts no cross-µservice node references. Cedar evaluation runs server-side.
- ChangeSet: each accepted T1 draft is one ChangeSet (per ADR-0110); reversible by the tenant via `oya vcs revert`.
- EU AI Act posture: **potentially high-risk** depending on tenant industry. Builder asks "is this form used for employment / credit / insurance screening?" at publish time:
  - If `annex_iii_4_screening=true`: T1 still allowed BUT triggers high-risk classification → mandatory DPIA + AI-Act conformity assessment + post-market monitoring SLI tracked.
  - If false: not high-risk; transparency obligation (Art. 50) met by builder banner; human oversight (Art. 14) met by tenant-accept-before-save.

### T2 — Auto (opt-in per-tenant + per-capability; default off; cross-µservice gated)

- Scope: T2 has two sub-scopes (mirroring ADR-WS-0005 structure):
  - **T2-intra**: forms' existing autonomous capabilities — response capture + audit-chain seal + column encryption + workflow-trigger to a configured workflow-engine destination. These pre-date AI-form-build; AI-form-build doesn't change their tier.
  - **T2-cross**: AI-form-build draft of fragments that include cross-µservice destinations (form-on-submit → email via mail; form-on-submit → trigger workflow in workflow-engine; file-upload → drive; etc.). **This is the gated case.**
- **T2-cross gate**:
  1. Tenant explicitly enables `forms.t2.auto.cross-microservice.<destination>` per destination µservice. Enablement is per-(tenant, destination-µservice) pair, not blanket.
  2. Cedar policy permits the cross-call. Cedar entities: `(tenant, source-microservice=forms, destination-microservice, tenant-class=T2, ai-build-origin=true)`. Default-deny per ADR-0140.
  3. Candidate spec routed through ChangeSet state machine (ADR-0110) as proposed-state ChangeSet requiring human-author + reviewer-agent approval before the foreign µservice's binding is committed. T2-cross NEVER auto-saves a cross-µservice fragment.
  4. **2-person rule**: LLM is first author; a human OR another agent acting under a different signing key is the required reviewer. Same agent signing both author + reviewer roles fails the gate.
  5. Destination µservice's own SDK contracts apply (workflow-engine SDK, mail SDK, drive SDK, etc.); AI output is bounded by what those SDKs already permit external callers to do.
- EU AI Act posture: T2-cross is highest-risk; treat as **high-risk under EU AI Act Arts. 9-15 by default**. Tenant DPIA mandatory before enablement. Post-market monitoring (Art. 72) mandatory via `dashboards/ai-form-build-quality.json` SLIs.

### Annex III §4 (employment / credit / insurance screening) high-risk classification

- Tenant attests `annex_iii_4_screening` at form publish time (boolean).
- If true:
  1. High-risk classification recorded in Form entity + audit-chain seal.
  2. DPIA prompt surfaced in builder; publish blocked until DPIA reference attested.
  3. AI-Act conformity assessment reference required (`legal/ai-act-conformity.md`).
  4. CE-marking + notified body engagement at council-legal-compliance discretion.
  5. Quarterly post-market monitoring report includes Annex-III-§4 forms cohort.
- If false: still subject to T0/T1/T2 normal posture; no Annex III §4 obligations beyond regular GDPR.
- **Detection sensitivity**: AI-form-build runs a high-recall classifier on the prompt + emitted form for likely-Annex-III-§4 patterns (employment keywords, scoring/ranking fields, hiring-context language); if classifier flags AND tenant attested false, the builder surfaces a "double-check" warning. Tenant attestation is authoritative (we do not auto-flip the boolean), but the discrepancy is audit-sealed.

### Forbidden under all tiers

- AI output that bypasses Cedar policy (e.g., `cedar_check: skip`). Hard error.
- AI output that emits `data_class=NORMAL` for a field whose label/placeholder LLM-inferred as PII. Hard error; tenant sees diagnostic.
- AI output that references µservices the tenant has no entitlement to. Hard error.
- AI output passing through to save without dsl-loader validation + form.v1 canonicalisation (ADR-FORMS-0001) + Cedar preview. Hard error.
- AI output suggesting a captcha-disabled form on anonymous-submission-allowed mode. Hard error.

## Alternatives Considered

### Alternative A — T0 + T1 intra-µservice only; T2 entirely forbidden

Bound AI-form-build to suggestions + intra-product drafts; no autonomous cross-µservice action at all.

- **Pros**: maximally safe; smallest blast radius; trivially Art. 22 compliant; simplest implementation.
- **Cons**: falls behind Microsoft Copilot-for-Forms / Typeform AI / Jotform AI Form Builder which all draft cross-product flows. The "build me a form that emails the senior AE when filled" is the obvious tenant ask; refusing it limits product differentiation. Tier framework anticipates T2.
- **Rejected reason**: too restrictive for competitive parity. T2-cross with strict gates is the right balance.

### Alternative B — Unlimited cross-µservice AI-form-build at T1 (no Cedar/ChangeSet gate)

Allow AI to freely emit fragments calling any µservice the tenant has entitlement to, at T1.

- **Pros**: maximal AI capability; "build me a form that does X, Y, Z" works without per-destination configuration.
- **Cons**: widens tenant blast radius unconsciously; tenant authorised "draft a form" without thinking through every destination µservice the LLM might choose; prompt-injection risk multiplies; EU AI Act Arts. 9-15 high-risk by default; reversibility complex when a single draft spans multiple µservices' ChangeSets.
- **Rejected reason**: blast radius + supply-chain attack surface + EU AI Act high-risk default. Marginal capability gain is small; marginal risk is large.

### Alternative C — T2 cross-µservice AI-form-build with Cedar but no ChangeSet review (auto-commit on Cedar pass)

Cedar gates cross-µservice AI output but auto-commits on Cedar pass.

- **Pros**: lower friction than chosen design. Cedar is the trust boundary; "if Cedar said yes, why force review?"
- **Cons**: Cedar enforces structural rules (who can call what); does NOT enforce semantic correctness (the LLM's *interpretation* of the tenant ask). Without ChangeSet review, an LLM-misinterpreted prose ask gets auto-deployed; tenant discovers at run-time. EU AI Act Art. 14 human-oversight obligation harder to defend.
- **Rejected reason**: Cedar necessary but not sufficient for T2-cross safety. ChangeSet review is the human-oversight hatch the EU AI Act expects.

### Alternative D — AI-form-build intra-µservice only; cross-µservice ALWAYS via tenant-explicit Workflow + Ontology composition (no AI involvement)

AI stays strictly intra-product; cross-µservice composition is a tenant-manual step via Workflow + Ontology adapter.

- **Pros**: honours `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md` strictly. Architecturally clean.
- **Cons**: tenant UX friction; most cross-product asks come in prose; forcing manual composition adds an authoring step. Tier T2 capability slot unused.
- **Note**: chosen Decision actually preserves the architectural property — AI-emitted cross-µservice fragments STILL flow through the Workflow + Ontology adapter pattern at run-time; the AI drafts the adapter composition, but it's still composed-through-Workflow+Ontology. The chosen design adds the tier gate on top of that adapter pattern.
- **Rejected reason** (as standalone): unnecessarily restrictive; chosen design preserves invariant via gated T2-cross.

### Alternative E — Train a smaller, domain-specific form-build model in-house

Train a fine-tuned model only on form-build patterns.

- **Pros**: smaller blast radius; no third-party prompt leak; potentially cheaper at scale.
- **Cons**: training-data sourcing risk (tenant data cannot be used for training without explicit DPA + opt-in); maintenance cost; lag behind general-purpose LLMs; EU AI Act Art. 10 data governance still applies.
- **Rejected reason**: not a tier question; orthogonal model-sourcing decision. BYO-LLM + pack-resident provider routing is the chosen sourcing posture; in-house fine-tuning is a future Tier-G+ feature.

## Consequences

### Architectural

- The `oya-forms-ai-build-adapter` implements the tier-aware adapter:
  - T0/T1 path: bounded to forms' field library + Ontology reads.
  - T2-cross path: Cedar entity construction + ChangeSet draft creation + reviewer-agent invocation + 2-person rule enforcement.
- The dsl-loader rejects AI-emitted fragments that violate tier scope; structural enforcement (not policy-based) for defence-in-depth.
- The AI-build adapter exposes `FormAiBuildDraft{tenant, draft_id, prompt_hash, completion_hash, tier, cross_microservice_destinations[], annex_iii_4_screening, high_risk_classified, dpia_required}` as Ontology object type; `tier` + `cross_microservice_destinations` + `annex_iii_4_screening` queryable for compliance reporting.

### Downstream µservices

1. **foundry-providers** — LLM-call envelope carries tier + scope; archives scope for cross-product audit.
2. **tenancy** — per-tenant entitlements: `forms.t0`, `forms.t1`, `forms.t2.auto.intra`, `forms.t2.auto.cross-microservice.<destination>` issued + revocable.
3. **All destination µservices** for T2-cross (workflow-engine, mail, messenger, drive, sheets, etc.) — each ships Cedar policy fragment defining whether they accept AI-form-build-originated cross-calls at T2 + which capabilities. Each owning team owns that decision.
4. **observability** — `dashboards/ai-form-build-quality.json` panels: `oya_forms_ai_build_t2_cross_acceptance_rate`, `oya_forms_ai_build_t2_cross_revert_rate`, `oya_forms_ai_build_t2_cross_safety_signal_total`, `oya_forms_ai_build_annex_iii_4_classified_total`.
5. **council-legal-compliance** — tenant-side DPIA template + per-tenant T2-cross enablement workflow per EU AI Act Art. 72.
6. **dpia.md** — R-03 + R-15 risks tracked.

### SLOs and CI lanes affected

- `oya-governance-cedar-preview-required` — exercised for every AI-build output.
- `oya-forms-ai-build-quality` — schema-valid rate ≥ 80%; non-zero adversarial output blocked = informational SLI.
- `oya-forms-ai-build-annex-iii-4-classification-conformance` — high-recall sensitivity ≥ 90%.
- `oya-forms-ai-build-t2-cross-safety-signal-count` — Sev-2 if non-zero in any 24h window.
- `oya-forms-ai-build-forbidden-output-blocked-count` — informational; sudden spike = prompt-injection campaign.
- `oya-forms-ai-build-revert-rate` — quality SLI.

### Compliance + audit

- Every AI-build invocation emits `FormAiBuildRequested` (PRD §"Workflow events produced") with tier + scope + annex_iii_4_screening.
- 90-day archive of prompt + completion + tier + scope + Cedar decision per `policy/data-residency.md`.
- T2-cross drafts emit `FormAiBuildT2CrossDraftReviewed{verdict, reviewer_identity}` events.
- EU AI Act post-market monitoring quarterly report to council-legal-compliance summarising T2-cross usage + safety signals + Annex III §4 cohort + revert rate.
- DPIA updated per release to reflect tier-aware AI-build scope.

### Risk register

- **Risk**: T2-cross gate fatigue — tenant disables ChangeSet review to ship faster. **Mitigation**: per-tenant Cedar entitlement is granular per destination; tenant can grant blanket-consent for low-risk destinations only via explicit allow-list authored under `microservices/forms/specs/`.
- **Risk**: prompt-injection succeeds at emitting an out-of-scope fragment. **Mitigation**: structural rejection in dsl-loader + Cedar preview; SLI on forbidden-output-blocked-count.
- **Risk**: EU AI Act classification evolves; T1 posture becomes high-risk. **Mitigation**: quarterly compliance review (per ADR-0133 axis-4); ADR-FORMS-0005 supersession is the upgrade path.
- **Risk**: tenant LLM (BYO-LLM) emits non-deterministic output. **Mitigation**: schema-valid completion enforced; non-conforming output rejected (AC-25).
- **Risk**: Annex III §4 misuse by tenant (declares false; actually uses for hiring). **Mitigation**: high-recall classifier + audit-sealed discrepancy + quarterly review. Tenant attestation authoritative but discrepancy investigable.

## References

- Regulation (EU) 2024/1689 (EU AI Act) — Arts. 9, 10, 11, 12, 13, 14, 15, 43, 47, 50, 72; Annex III §4. Official Journal of the European Union, 12 July 2024 — `eur-lex.europa.eu/legal-content/EN/TXT/?uri=OJ:L_202401689`.
- Regulation (EU) 2016/679 (GDPR) Art. 22 — automated individual decision-making.
- OWASP Top 10 for Large Language Model Applications — `owasp.org/www-project-top-10-for-large-language-model-applications/`.
- ADR-WS-0005 (workflow-studio AI-copilot bounds) — sibling decision; structural inheritance.
- ADR-0110 ChangeSet state machine.
- ADR-0140 Cedar policy enforcement.
- ADR-FORMS-0001 (form.v1 schema; dsl-loader validation).
- ADR-FORMS-0004 (CEL branching; AI-emitted predicates).
- `microservices/forms/PRD.md` FR-22, AC-25.
- `microservices/forms/dpia.md`.
- `microservices/forms/dashboards/ai-form-build-quality.json`.
- `microservices/forms/runbooks/ai-form-build-rollback.md`.
- `feedback_workflow_objectgraph_adapter_layer.md` — Workflow + Ontology adapter pattern.
