---
doc_class: BNF-v4.1-Pack-State-Grammar
journey_id: j96-ksa-uae-mena-tenant-onboarding
status: draft
date: 2026-05-20
related_adrs:
  - ADR-0105-thirteen-layer-canonical-enum
  - ADR-0251-compliance-pack-cell-certification-levels
---

# j96 BNF v4.1 Pack State Grammar

```bnf
<journey-pack-state> ::= <journey-id> ":" <tenant-id> ":" <pack-set> ":" <cell-cert> ":" <cedar-decision> ":" <audit-event>
<journey-id> ::= "j96"
<pack-set> ::= <pack-id> | <pack-id> "," <pack-set>
<pack-id> ::= "KSA-NDMO" | "KSA-PDPL" | "UAE-PDPL"
<cell-cert> ::= "ksa-sovereign" | "uae-controlled-transfer"
<cedar-decision> ::= "permit" | "forbid" | "abstain"
<audit-event> ::= "EVT-J" <number> "-" <service-token> "-" <sequence>
```

## ADR-0105 13-layer mapping

| Layer | Journey binding |
|---|---|
| experience | j96 Arabic tenant signup with pack KSA-NDMO |
| edge | j96 KSA sovereign cell placement with pack KSA-PDPL |
| api-rest | j96 NDMO classification mapping with pack UAE-PDPL |
| api-async | j96 UAE branch transfer review with pack KSA-NDMO |
| adapter | j96 SDAIA-ready evidence packet with pack KSA-PDPL |
| usecase | j96 right-to-access bilingual response with pack UAE-PDPL |
| domain | j96 Arabic tenant signup with pack KSA-NDMO |
| kernel | j96 KSA sovereign cell placement with pack KSA-PDPL |
| policy | j96 NDMO classification mapping with pack UAE-PDPL |
| eventing | j96 UAE branch transfer review with pack KSA-NDMO |
| observability | j96 SDAIA-ready evidence packet with pack KSA-PDPL |
| iac | j96 right-to-access bilingual response with pack UAE-PDPL |
| evidence | j96 Arabic tenant signup with pack KSA-NDMO |

## Exact article terminals

- <article-1> ::= "KSA PDPL Royal Decree M/19 Article 5 lawful basis and consent principles"
- <article-2> ::= "KSA PDPL Article 6 processing without consent exceptions"
- <article-3> ::= "KSA PDPL Article 18 data subject rights and controller response duties"
- <article-4> ::= "KSA PDPL Article 20 personal data breach notification to the competent authority"
- <article-5> ::= "KSA PDPL Article 29 transfer or disclosure of personal data outside the Kingdom"
- <article-6> ::= "SDAIA Regulation on Personal Data Transfer Outside the Kingdom implementing PDPL Article 29"
- <article-7> ::= "NDMO National Data Governance Interim Regulations data classification and data sharing controls"
- <article-8> ::= "UAE Federal Decree-Law No. 45 of 2021 Article 4 data subject rights"
- <article-9> ::= "UAE PDPL Articles 22 and 23 cross-border transfer controls"
- <article-10> ::= "UAE PDPL Article 24 personal data security and breach notification obligations"
