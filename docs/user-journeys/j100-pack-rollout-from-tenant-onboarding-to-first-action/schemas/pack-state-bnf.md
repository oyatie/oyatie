---
doc_class: BNF-v4.1-Pack-State-Grammar
journey_id: j100-pack-rollout-from-tenant-onboarding-to-first-action
status: draft
date: 2026-05-20
related_adrs:
  - ADR-0105-thirteen-layer-canonical-enum
  - ADR-0251-compliance-pack-cell-certification-levels
---

# j100 BNF v4.1 Pack State Grammar

```bnf
<journey-pack-state> ::= <journey-id> ":" <tenant-id> ":" <pack-set> ":" <cell-cert> ":" <cedar-decision> ":" <audit-event>
<journey-id> ::= "j100"
<pack-set> ::= <pack-id> | <pack-id> "," <pack-set>
<pack-id> ::= "PACK-AGNOSTIC" | "HIPAA-WORKED-EXAMPLE"
<cell-cert> ::= "general-to-hipaa-certified-migration" | "pack-rollout-safe"
<cedar-decision> ::= "permit" | "forbid" | "abstain"
<audit-event> ::= "EVT-J" <number> "-" <service-token> "-" <sequence>
```

## ADR-0105 13-layer mapping

| Layer | Journey binding |
|---|---|
| experience | j100 mid-flight pack activation with pack PACK-AGNOSTIC |
| edge | j100 pre-migration inventory with pack HIPAA-WORKED-EXAMPLE |
| api-rest | j100 HIPAA cell eligibility check with pack PACK-AGNOSTIC |
| api-async | j100 Cedar fragment refresh with pack HIPAA-WORKED-EXAMPLE |
| adapter | j100 workflow compensation with pack PACK-AGNOSTIC |
| usecase | j100 first protected action proof with pack HIPAA-WORKED-EXAMPLE |
| domain | j100 mid-flight pack activation with pack PACK-AGNOSTIC |
| kernel | j100 pre-migration inventory with pack HIPAA-WORKED-EXAMPLE |
| policy | j100 HIPAA cell eligibility check with pack PACK-AGNOSTIC |
| eventing | j100 Cedar fragment refresh with pack HIPAA-WORKED-EXAMPLE |
| observability | j100 workflow compensation with pack PACK-AGNOSTIC |
| iac | j100 first protected action proof with pack HIPAA-WORKED-EXAMPLE |
| evidence | j100 mid-flight pack activation with pack PACK-AGNOSTIC |

## Exact article terminals

- <article-1> ::= "45 CFR 164.308 administrative safeguards"
- <article-2> ::= "45 CFR 164.310 physical safeguards"
- <article-3> ::= "45 CFR 164.312 technical safeguards"
- <article-4> ::= "45 CFR 164.316 policies, procedures, and documentation requirements"
- <article-5> ::= "45 CFR 164.502 uses and disclosures of protected health information"
- <article-6> ::= "45 CFR 164.514 de-identification and limited data set requirements"
- <article-7> ::= "45 CFR 164.524 access of individuals to protected health information"
- <article-8> ::= "45 CFR 164.530 administrative requirements"
- <article-9> ::= "ADR-0251 pack activation and cell certification levels"
- <article-10> ::= "ADR-0243 Cedar default-deny and signed fragment bundle publication"
