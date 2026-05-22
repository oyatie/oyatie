---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: L-015
authoritative_source: Regulation (EU) 2024/1689 (EU AI Act)
related_packs: [eidas, gdpr]
date: 2026-05-21
---

# EU AI Act Annex III Classification for CLM AI

Regulation (EU) 2024/1689 (the EU AI Act, entered into force 2024-08-01; phased application through 2027) classifies AI systems by risk: Prohibited (Article 5), High-Risk (Article 6 + Annex III), Limited-Risk (Articles 50-52), Minimal-Risk. CLM uses AI for clause suggestion, redlining, obligation extraction, and renewal-risk scoring. This document declares each AI feature's classification under the Act.

## Article 5 — prohibited practices

CLM AI features do **not** implement any of the eight Article 5 prohibited practices:

- (a) Subliminal techniques beyond consciousness — N/A.
- (b) Exploitation of vulnerabilities — N/A.
- (c) Social scoring by public authorities — N/A.
- (d) Predictive policing based on profiling — N/A.
- (e) Untargeted scraping of facial images — N/A.
- (f) Emotion recognition in workplace/education — N/A (CLM does not analyze counterparty emotions).
- (g) Biometric categorization based on sensitive attributes — N/A.
- (h) Real-time remote biometric identification in publicly accessible spaces for law enforcement — N/A.

## Annex III — high-risk classification

Annex III enumerates eight categories of high-risk AI. CLM AI is evaluated against each:

### 1. Biometric identification, categorization, emotion recognition

**Not Annex III.** CLM does not perform biometric processing.

### 2. Critical infrastructure (safety components)

**Not Annex III.** CLM is not a safety component of critical infrastructure.

### 3. Education and vocational training

**Not Annex III.** CLM is not used to determine access to education or evaluate learning outcomes.

### 4. Employment, workers management, access to self-employment

**Conditional.** CLM AI clause-suggestion and obligation-extraction are NOT Annex III when applied to commercial contracts. They MAY be Annex III when applied to **employment contracts** if:

- The AI is used to evaluate candidates (recruitment) — Annex III 4(a).
- The AI is used to make decisions about promotion, termination, work-allocation — Annex III 4(b).
- The AI is used to monitor or evaluate employees — Annex III 4(b).

CLM's default behaviour for employment contracts: AI suggestions are **advisory only**; final clause acceptance is by a human reviewer. AI output is logged but does not autonomously modify the contract. Under this design, CLM does **not** qualify as Annex III 4 even for employment contracts.

When a tenant configures `ai_redlining.autonomous_acceptance = true` for employment contracts, the system is reclassified as Annex III 4 and the tenant assumes the high-risk operator obligations (registration, risk management, data governance, transparency).

### 5. Access to essential private and public services

**Not Annex III.** CLM does not determine access to credit, social benefits, emergency services, etc.

### 6. Law enforcement

**Not Annex III.** CLM is not used by law enforcement to evaluate persons.

### 7. Migration, asylum, border control

**Not Annex III.** N/A.

### 8. Administration of justice and democratic processes

**Not Annex III.** CLM is not used to assist a judicial authority in researching and interpreting facts and the law (Annex III 8(a)). CLM is used by **commercial parties** in private contract negotiation; it is not used by courts.

(Note: if a court adopted CLM's AI clause-suggestion to draft court orders, that adoption would fall under Annex III 8(a). The µservice does not market to or onboard judicial-authority tenants for this purpose.)

## Conclusion — default classification

CLM AI features (clause suggestion, redlining, obligation extraction, renewal-risk scoring) are classified as **Limited Risk** under the EU AI Act, not High Risk.

**Exception**: CLM AI applied to **employment contracts** with `autonomous_acceptance = true` becomes Annex III 4 high-risk; the tenant must comply with the Article 8-15 high-risk obligations.

## Article 50 — transparency obligations (Limited Risk)

Under Article 50, providers of AI systems that interact with natural persons or generate content must:

- Inform the natural person that they are interacting with an AI system (unless obvious from the circumstances).
- Mark AI-generated content as such (where the content is text intended to inform the public).

CLM's AI-suggested clauses are marked as AI-generated in the UI and in the contract metadata (`clause.ai_generated = true` + `ai_model_attestation`). The signatory always sees that a clause is AI-suggested before sealing the contract.

## Annex III tenant operator overlay

When `autonomous_acceptance = true` for employment contracts is enabled:

```
HighRiskAIOperatorObligations {
  ai_system_id: AISystemId,                       // EU database registration
  risk_management_system: RiskMgmtRef,            // Article 9
  data_governance: DataGovernanceRef,             // Article 10
  technical_documentation: TechDocRef,            // Article 11
  record_keeping: RecordKeepingRef,                // Article 12
  transparency: TransparencyRef,                  // Article 13
  human_oversight: HumanOversightRef,             // Article 14
  accuracy_robustness: AccuracyRobustnessRef,     // Article 15
  ce_marking: CEMarkingRef,                       // Article 48
}
```

The µservice produces these artefacts but the tenant is responsible for registration in the EU AI database (Article 71) and CE marking.

## Foundation model (GPAI) obligations

If a CLM AI feature uses a general-purpose AI model (Llama-3.1-8B/70B, Claude, GPT-4o), the foundation model provider has Article 53-55 obligations. CLM as the deployer does NOT inherit foundation-model obligations; the deployer must:

- Ensure the foundation model used has Article 53 documentation available.
- Not subject the foundation model to use it claims not to support.
- Respect copyright (Article 53(1)(d)) — CLM's AI suggestions cite training-data-source attribution where possible.

## AI model provenance

Per `legal-dimensions/ai-redlining-prompt-template.md` + `IP-027`, each AI suggestion carries:

- `model_id`: the specific model (e.g. "anthropic/claude-3-7-sonnet@20260301").
- `model_version`: pinned version.
- `prompt_hash`: BLAKE3 of the actual prompt used.
- `temperature` + `seed`: for reproducibility.
- `output_hash`: BLAKE3 of the suggestion.
- `confidence`: model self-reported confidence (where available).

## Cedar gate

```cedar
forbid (
  principal,
  action == Action::"AIClauseAutoAccept",
  resource is Contract
) when {
  resource.contract_type in ["employment", "termination_letter", "promotion_decision"] &&
  resource.tenant.eu_ai_act_high_risk_operator_registered == false
};
```

## Audit events

- `oya.contract.lifecycle.management.ai.suggestion_generated`
- `oya.contract.lifecycle.management.ai.suggestion_accepted` (human-in-the-loop)
- `oya.contract.lifecycle.management.ai.autonomous_acceptance_attempted` (Annex III boundary check)
- `oya.contract.lifecycle.management.ai.high_risk_classification_change`

## Standards references

- Regulation (EU) 2024/1689 (EU AI Act).
- Commission AI Office guidance.
- ETSI TR 103 305-5 (AI lifecycle).
- ISO/IEC 42001:2023 (AI Management Systems).
- NIST AI RMF 1.0.
