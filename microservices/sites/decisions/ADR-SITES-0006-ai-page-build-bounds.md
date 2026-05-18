---
id: ADR-SITES-0006
status: Accepted
date: 2026-05-17
microservice: sites
deciders: axis-sites, council-architecture, council-privacy, foundry-runtime
owner: axis-sites + council-privacy
supersedes: []
superseded_by: []
related:
  - ADR-0140
  - ADR-0133
  - ADR-WS-0006
  - ADR-DOCS-0006
  - ADR-SHEETS-0006
related_artifacts:
  - microservices/sites/PRD.md §FR-22, AC-13
  - microservices/sites/capabilities/T2-auto.yaml
  - microservices/sites/policy/editor-isolation.md Invariant 4
  - microservices/sites/runbooks/ai-page-build-rollback.md
  - microservices/sites/compliance.md §"pack-eu" overlay
purpose: |
  Define the EU AI Act bounds and refusal envelope for T2
  AI-page-build. Align with sibling µservice T2 capabilities
  (workflow-studio, docs, sheets, slides) for cross-µservice
  consistency.
---

# ADR-SITES-0006: AI-page-build T2 EU AI Act bounds — HR/legal/medical contexts REFUSED pending Annex III §3 conformity assessment

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

T2 AI-page-build (per PRD §FR-22) lets editors generate a full page
from a prompt + context overlay. Per `feedback_autonomous_decision_
principles.md`, this is a "consequential action" — it materially
shapes the published content tenants ship to the world. Per the EU AI
Act Regulation (EU) 2024/1689:

- **Art. 50 (transparency)**: applies to AI-generated content;
  visible label required ("AI is suggesting this page").
- **Art. 14 (human oversight)**: applies to high-risk AI systems;
  reversibility window + explicit accept satisfies for limited-risk.
- **Annex III §3 (employment + workers)**: any AI system used "in
  employment, workers management and access to self-employment, in
  particular for…recruitment…performance evaluation…task allocation"
  is HIGH-RISK. Requires conformity assessment, post-market monitoring,
  transparency obligations.
- **Annex III §1 (essential services and benefits)**: any AI used
  "to evaluate the eligibility of natural persons for essential
  private services and essential public services and benefits" is
  HIGH-RISK.

A tenant could legitimately use T2 to generate:
- A marketing landing page (limited-risk).
- A blog post (limited-risk).
- A product page (limited-risk).
- An intranet landing page (limited-risk).

But a tenant could ALSO use T2 to generate:
- A job-posting page where the language affects hiring (Annex III §3 high-risk).
- A medical-services page that describes treatments (medical-device classification adjacent + Annex III §5 health).
- A legal-services page where language affects legal outcomes (Annex III §8 administration of justice).
- A credit-decision page (Annex III §1 essential services).

Without conformity assessment for the HIGH-RISK overlays, the sites
µservice CANNOT ship T2 in those contexts in pack-eu without exposure.

Cross-µservice consistency: workflow-studio + docs + sheets + slides
T2 capabilities face the same question. The decision must align.

## Decision

The sites µservice ships **T2 AI-page-build with LIMITED-RISK default
classification** and **HIGH-RISK contexts REFUSED at the Cedar policy
layer** until a successor-IP ADR-SITES-XXXX completes the Annex III §3
conformity assessment.

Concrete bindings:
- **Allowed context overlays at T2**: `marketing`, `blog`, `product`,
  `landing`, `intranet_general`.
- **Refused context overlays at T2** (until ADR-SITES-XXXX):
  - `hr` (Annex III §3 employment context).
  - `legal` (Annex III §8 justice administration).
  - `medical` (Annex III §5 healthcare).
  - `employment_decision` (Annex III §3).
  - `credit_decision` (Annex III §1 essential services).
- **Cedar refuses HIGH-RISK contexts**: `policy/tenant-scope.cedar`
  has explicit `forbid` rule for these overlays.
- **EU AI Act labels**: per Art. 50, T2-suggested pages bear "AI is
  suggesting this page — review before publish" label in the editor
  UI; the label is structural (not tenant-removable).
- **Reversibility per Art. 14**: 30-second window; explicit accept
  required; cancellation reverts.
- **Audit-chain**: every T2 invocation, accept, refusal, and
  post-hoc-revert sealed with Ed25519 + EU AI Act classification field.
- **Tenant-DEK prompt wrapping**: prompts ciphertext-only to LLM
  provider via foundry-runtime private-inference channel.
- **No cross-tenant training**: foundry-runtime structurally refuses;
  LEAN `oya-check-ai-page-build-tenant-isolation` enforces.
- **Post-publish safety classifier**: T2 output reviewed
  asynchronously; flagged outputs trigger runbook
  `ai-page-build-rollback.md`.
- **Per-pack LLM provider restriction**: pack-eu uses EU-resident LLM
  providers only; pack-us-healthcare uses BAA-on-file providers only.

## Alternatives Considered

### A. Ship T2 with HR/legal/medical contexts enabled at launch

- **Pros**:
  - Maximum tenant capability.
  - Equal-priority with Webflow AI / Notion AI / Squarespace AI.
- **Cons**:
  - EU AI Act Annex III §3 conformity assessment required BEFORE
    market placement (Art. 6); we don't have the assessment.
  - Liability exposure under Art. 99 (penalties up to 7% global
    turnover).
  - Council-privacy + external AI compliance firm: assessment is
    minimum 3-6 months.
- **Rejected**.

### B. Disable T2 entirely until conformity assessment lands

- **Pros**:
  - Zero regulatory risk.
- **Cons**:
  - Marketing/blog/product/landing tenants want T2; deferring
    indefinitely is a competitive loss vs Webflow / Notion / etc.
  - Cross-µservice consistency: workflow-studio + docs already
    decided to ship T2 in limited-risk contexts; sites should align.
- **Rejected** in favor of context-overlay gating.

### C. Ship T2 with HR/legal/medical contexts but block at the user-prompt content (post-hoc classification)

- **Pros**:
  - Maximum capability with attempt at safety.
- **Cons**:
  - Classification false-negatives = HIGH-RISK content slips through;
    not Annex III conformant.
  - "Best-effort moderation" doesn't satisfy Annex III conformity
    obligation.
- **Rejected** — moderation is a defence-in-depth, not the primary
  gate.

### D. Context-overlay gating at Cedar  ← **CHOSEN**

- **Pros**:
  - Refusal is structural (Cedar), not "best-effort".
  - Tenant explicitly declares context_overlay; legal/HR/medical
    are refused by name.
  - Conformity-assessment path stays clear: when ADR-SITES-XXXX
    lands, Cedar policy is amended.
  - Cross-µservice consistent (sibling ADRs ADR-WS-0006, ADR-DOCS-
    0006, ADR-SHEETS-0006).
- **Cons**:
  - Tenants in HR/legal/medical context can't use T2 until
    conformity ships; sites loses parity in those segments.
- **Accepted**.

## Consequences

### Positive

- **EU AI Act-compliant by construction.** Annex III §3 contexts are
  refused at Cedar; conformity assessment can ship incrementally.
- **Cross-µservice consistent.** workflow-studio + docs + sheets +
  slides + sites all refuse HIGH-RISK contexts; tenants encounter the
  same boundary across the suite.
- **Art. 50 transparency labels** structurally enforced.
- **Art. 14 reversibility** built into the 30s window.
- **No cross-tenant training** structurally forbidden via foundry-
  runtime channel.

### Negative

- **Tenants in HR/legal/medical can't use T2 yet.** Competitive
  weakness in those segments. Mitigation: ADR-SITES-XXXX conformity
  assessment as scheduled work.
- **Per-pack LLM provider restriction** (EU-only / BAA-only) adds
  ops complexity. Mitigation: foundry-runtime maintains the routing.
- **Post-publish safety classifier** adds review latency; flagged
  outputs may surprise editors days after publish. Mitigation:
  classifier review SLO ≤ 24h; flagged content tagged but not
  auto-reverted.

### Operational

- **Cedar policy** explicit `forbid` for HIGH-RISK overlays.
- **Audit-chain** every T2 event sealed with `eu_ai_act_classification`
  field (`limited_risk`, `minimal_risk`, `high_risk_refused`).
- **Runbook `ai-page-build-rollback.md`** covers post-hoc reverts.
- **Quarterly review** by council-privacy of T2 invocation patterns;
  refusal-pattern audit.

### Regulatory

- **EU AI Act Art. 50**: transparency labels in editor UI.
- **EU AI Act Art. 14**: reversibility window.
- **EU AI Act Annex III §3, §5, §8, §1**: REFUSED pending conformity.
- **EU AI Act Art. 99**: penalty exposure minimized.
- **GDPR Art. 22**: T2 is creative-assistance, NOT a decision affecting
  legal/significant rights of a data subject at default; refusal of
  HIGH-RISK contexts ensures we don't slip into Art. 22 territory.
- **HIPAA**: pack-us-healthcare T2 uses BAA-on-file LLM providers only.

## Verification

- [ ] **HIGH-RISK context refusal at Cedar** —
  `cargo nextest run -p oya-sites-page-usecase -- ai_page_build_refusal_hr`.
- [ ] **Cross-tenant prompt isolation** —
  `cargo nextest run -p oya-sites-page-usecase -- ai_page_build_tenant_isolation`.
- [ ] **Audit-chain EU AI Act field** —
  `cargo nextest run -p oya-sites-page-app -- audit_chain_eu_ai_act_field`.
- [ ] **Reversibility 30s window** —
  `cargo nextest run -p oya-sites-page-usecase -- ai_page_build_reversibility_30s`.

## References

- EU AI Act Regulation (EU) 2024/1689 — Arts. 6, 14, 50, 99;
  Annex III §1, §3, §5, §8.
- GDPR Regulation (EU) 2016/679 — Art. 22 (automated decisions).
- HIPAA 45 CFR §164.504(e) (BAA).
- ADR-0140 (Cedar policy).
- ADR-0133 (industry best-practice).
- Sibling: ADR-WS-0006 (workflow-studio T2); ADR-DOCS-0006 (docs T2);
  ADR-SHEETS-0006 (sheets T2); ADR-SLIDES-0006 (slides T2).
- `microservices/sites/PRD.md` §FR-22, AC-13.
- `microservices/sites/capabilities/T2-auto.yaml`.
- `microservices/sites/policy/editor-isolation.md` Invariant 4.
- `microservices/sites/runbooks/ai-page-build-rollback.md`.
- `microservices/sites/compliance.md` §"pack-eu" overlay.
