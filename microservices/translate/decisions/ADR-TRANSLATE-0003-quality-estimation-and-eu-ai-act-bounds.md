---
doc_class: AdrSpec
template_id: TPL-ADR
adr_id: ADR-TRANSLATE-0003
title: Quality Estimation and EU AI Act bounds
status: Accepted
deciders: council-architecture, council-privacy, axis-translate, ops-security, ops-compliance
date: 2026-05-17
microservice: translate
supersedes: []
superseded_by: []
related_adrs: [ADR-0135, ADR-0131, ADR-TRANSLATE-0001, ADR-TRANSLATE-0004]
related_artifacts:
  - microservices/translate/PRD.md
  - microservices/translate/compliance.md
  - microservices/translate/dpia.md
  - microservices/translate/policy/ai-act-overlay.md
  - microservices/translate/IP-007-quality-estimation-stack.md
align_with: workflow-studio/decisions/ADR-WS-0005 (referenced; equivalent EU AI Act treatment)
doc_status: published
---

# ADR-TRANSLATE-0003 — Quality Estimation and EU AI Act bounds

## Context

Quality Estimation (QE) — predicting a quality score for a translation without a reference translation — is a separable function the µservice deploys via foundry-runtime. Use cases:

- Flag translations below threshold for human review (workflow-engine integration).
- Surface confidence to in-editor translation preview.
- Drive auto-acceptance of high-confidence MT in human-in-loop workflows.
- Compound TM with post-edit-distance metrics.

QE crosses an **EU AI Act (Reg. (EU) 2024/1689)** line: it is an AI system whose output may inform downstream actions. The classification under the Act determines compliance posture:

- **Prohibited** (Art. 5) — not applicable (QE does not exhibit any Art. 5 banned property).
- **High-risk** (Art. 6 + Annex III) — applicable IF QE is used in:
  - Employment context (Annex III §4): triaging translated CVs / employment-related communications.
  - Education (§3): triaging educational assessments.
  - Access to essential private + public services (§5): credit + benefits-related translations.
  - Law enforcement (§6): translated evidence.
  - Migration / asylum (§7): translated immigration documents.
  - Administration of justice (§8): translated legal documents.
  - Medical devices (under MDR; via Annex I).
- **Limited risk** (Art. 50) — default for QE-as-confidence-score on general content; Art. 50 transparency obligation applies.
- **Minimal risk** — not applicable.

oyatie must (a) honor the high-risk requirements (Arts. 9–15: risk management, data governance, technical documentation, record-keeping, transparency to deployers, human oversight, accuracy + cybersecurity) when triggered, and (b) emit Art. 50 transparency disclosure on every QE call regardless.

Industry references:

- **COMET** (`unbabel.github.io/COMET/`) — state-of-the-art reference-based + reference-free metrics.
- **COMET-Kiwi** (reference-free) — primary QE model class deployed.
- **TransQuest** (Ranasinghe et al. 2020) — QE alternative.
- **OpenKiwi** (Unbabel, 2019) — open-source reference-free QE.
- **WMT QE shared task** — yearly benchmark; pass-rate target.
- **Workflow Studio ADR-WS-0005** — sibling µservice's equivalent EU AI Act treatment; this ADR aligns.

## Decision

### 1. Default classification: limited-risk per Art. 50

QE deployed by translate µservice is classified **limited risk** by default. Art. 50 transparency obligation honored via:

- `EuAiActDisclosure` audit-chain event emitted **per QE call** (and per MT call too, since translation is also AI-system output).
- Per-call disclosure carries `(engine, model_id, jurisdiction, classification, system_prompt_hash, response_hash, decision_id, occurred_at)`.
- Tenant UIs consuming `qe_score` are required (per `developer-docs/translate/eu-ai-act-disclosure-consumption.md`) to render a transparency notice when displaying QE-informed acceptance decisions to end users.

### 2. High-risk classification: content-class-driven

QE is automatically classified `EuAiActClassification::HighRisk` when the request's `content_class` is one of:

- `ContentClass::Medical`
- `ContentClass::Employment`
- `ContentClass::Credit`
- `ContentClass::Legal`

When high-risk classification triggers:

1. **FRIA-on-file gate** — `policy_repo.assert_fria_on_file(content_class).await?` must succeed. Tenants without a Fundamental Rights Impact Assessment (Art. 27) on file for the relevant deployment context receive `EuAiActError::FriaRequired`.
2. **Risk management system (Art. 9)** — refer to this ADR + threat-model.md + DPIA.md.
3. **Data governance (Art. 10)** — TM provenance + termbase governance per ADR-TRANSLATE-0002 + per-vendor DPA.
4. **Technical documentation (Art. 11)** — this ADR + decisions/README.md + PRD.md + compliance.md.
5. **Record-keeping (Art. 12)** — `EuAiActDisclosure` + `QualityEstimated` + `TranslationCompleted` events stored 10y per audit-chain.
6. **Transparency to deployers (Art. 13)** — disclosure event includes model id + jurisdiction + classification.
7. **Human oversight (Art. 14)** — high-risk QE results NEVER auto-accept; they are routed through workflow-engine for human review.
8. **Accuracy + robustness + cybersecurity (Art. 15)** — QE model versioned + canary-deployed + golden-set evaluated (pass ≥ 0.99) + cybersecurity via mTLS + Cedar.

### 3. QE deployment lifecycle

- Model versioned: `translate-qe-comet-kiwi-vN`.
- Golden eval set: `microservices/translate/capabilities/eval/qe-golden.jsonl`; pass threshold 0.99; refreshed per release per ADR-0139.
- Rollback: per `runbooks/quality-estimation-rollback.md` ≤ 30 min RTO.
- Per-call `QualityScore.eu_ai_act_classification` field always populated.

### 4. EU AI Act disclosure consumption (downstream)

Per `policy/ai-act-overlay.md`, every µservice that consumes translate's QE score must:

- Surface the disclosure to end users when QE informs an automated acceptance/rejection.
- Provide a human-review path for high-risk classifications.
- Honor Art. 13 deployer transparency obligations toward their own end users (translate is processor; tenant is controller-deployer).

## Alternatives Considered

### Alternative A — Classify QE as minimal risk (default; no disclosure)

- **Pros**: simpler; no per-call event emission overhead.
- **Cons**: misclassifies under EU AI Act when used in legal/medical/employment context; regulatory exposure on first high-risk audit.
- **Verdict**: rejected. Default limited-risk + content-class auto-promote-to-high-risk is the safest posture.

### Alternative B — Refuse to deploy QE in EU pack

- **Pros**: no AI Act compliance burden.
- **Cons**: QE is the feature; refusing it means losing the value-add competitor parity (Phrase + Smartling all ship QE); commercially uncompetitive in EU.
- **Verdict**: rejected.

### Alternative C — Classify ALL QE as high-risk regardless of content class

- **Pros**: maximally conservative; uniform posture; no per-call branching.
- **Cons**: forces FRIA-on-file gate for all tenants for all content; commercially burdensome for UI-string + general-text tenants; over-compliance.
- **Verdict**: rejected.

### Alternative D — Outsource QE to vendor (e.g., Unbabel COMET API hosted)

- **Pros**: model maintenance burden gone.
- **Cons**: tenant data leaves oyatie; residency invariant broken; cost; vendor lock-in.
- **Verdict**: rejected.

### Alternative E — Skip QE; rely on human review only

- **Pros**: zero AI Act exposure on QE side.
- **Cons**: cost of human review unbounded; commercially uncompetitive; the entire industry deploys QE as a scaling lever.
- **Verdict**: rejected.

## Consequences

### positive

1. **EU AI Act compliance posture documented + enforced** — Art. 50 disclosure baked in; high-risk auto-classification by content-class; FRIA-on-file gate; alignment with Workflow Studio ADR-WS-0005.
2. **Tenant trust** — published ADR + per-call audit event + reviewable evidence makes auditor walkthroughs straightforward.
3. **Lever for human-in-loop workflow** — QE informs which translations land for human review, compounding cost optimization.
4. **Forward-compatible** — when EU AI Act enforcement details settle, the disclosure event schema accommodates additional fields without breaking changes.

### negative

1. **FRIA-on-file gate adds tenant onboarding friction** for high-risk content tenants — mitigated by tenant-onboarding documentation + per-pack templates.
2. **Per-call `EuAiActDisclosure` audit emission** — non-zero cost (NATS publish + audit-chain ingest); folded into cost-budget.md.
3. **Misclassification by tenant** (tenant labels content class wrong) — tenants are controller; oyatie best-effort; documented in DPA.

### neutral

1. **Quarterly golden eval** is the same cadence as other QE deployments in industry; no new burden.
2. **QE deployment via foundry-runtime** is the same pattern as MT + LangDetect; no new runtime concern.
3. **Per-call disclosure event** is sealed with Ed25519 like every other translate event; uniform posture.

## Validation

- `tests/integration/eu_ai_act_disclosure_emitted_per_qe_call.rs`.
- `tests/integration/high_risk_content_class_requires_fria.rs`.
- `tests/integration/golden_eval_pass_rate_99.rs`.
- Annual external EU AI Act audit by privacy counsel.
- Workflow Studio ADR-WS-0005 cross-validation.

## References

- EU AI Act (Reg. (EU) 2024/1689) — Arts. 5 + 6 + 9–15 + 27 + 50 + Annex III.
- EU AI Act Recital 53 + 54 (transparency obligations).
- EU AI Office implementation guidance (per-publication).
- COMET / COMET-Kiwi (`unbabel.github.io/COMET/`).
- TransQuest (Ranasinghe et al. 2020).
- OpenKiwi (Unbabel 2019).
- WMT QE shared-task results.
- Workflow Studio ADR-WS-0005 (sibling alignment).
- ADR-0135 — parent ADR.
- ADR-0131 — flat layout.
- ADR-TRANSLATE-0001 — engine routing (QE sampling).
- ADR-TRANSLATE-0004 — residency-bound inference (QE always in-house).
- GDPR Art. 22 (automated decision-making) — translate does NOT make automated decisions with legal effect; QE is informational only.
- `policy/ai-act-overlay.md` (sibling artifact in this µservice).
