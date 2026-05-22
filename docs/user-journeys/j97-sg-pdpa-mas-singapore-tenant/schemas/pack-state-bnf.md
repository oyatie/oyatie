---
doc_class: BNF-v4.1-Pack-State-Grammar
journey_id: j97-sg-pdpa-mas-singapore-tenant
status: draft
date: 2026-05-20
related_adrs:
  - ADR-0105-thirteen-layer-canonical-enum
  - ADR-0251-compliance-pack-cell-certification-levels
---

# j97 BNF v4.1 Pack State Grammar

```bnf
<journey-pack-state> ::= <journey-id> ":" <tenant-id> ":" <pack-set> ":" <cell-cert> ":" <cedar-decision> ":" <audit-event>
<journey-id> ::= "j97"
<pack-set> ::= <pack-id> | <pack-id> "," <pack-set>
<pack-id> ::= "SG-PDPA" | "SG-MAS-TRM" | "SG-MTCS-L3"
<cell-cert> ::= "sg-mtcs-l3" | "sg-financial-services"
<cedar-decision> ::= "permit" | "forbid" | "abstain"
<audit-event> ::= "EVT-J" <number> "-" <service-token> "-" <sequence>
```

## ADR-0105 13-layer mapping

| Layer | Journey binding |
|---|---|
| experience | j97 fintech tenant activation with pack SG-PDPA |
| edge | j97 PDPA consent catalog with pack SG-MAS-TRM |
| api-rest | j97 MAS critical-system tagging with pack SG-MTCS-L3 |
| api-async | j97 MTCS-L3 cell proof with pack SG-PDPA |
| adapter | j97 cross-border home-jurisdiction review with pack SG-MAS-TRM |
| usecase | j97 incident drill export with pack SG-MTCS-L3 |
| domain | j97 fintech tenant activation with pack SG-PDPA |
| kernel | j97 PDPA consent catalog with pack SG-MAS-TRM |
| policy | j97 MAS critical-system tagging with pack SG-MTCS-L3 |
| eventing | j97 MTCS-L3 cell proof with pack SG-PDPA |
| observability | j97 cross-border home-jurisdiction review with pack SG-MAS-TRM |
| iac | j97 incident drill export with pack SG-MTCS-L3 |
| evidence | j97 fintech tenant activation with pack SG-PDPA |

## Exact article terminals

- <article-1> ::= "Singapore PDPA section 11 accountability"
- <article-2> ::= "Singapore PDPA sections 13 to 17 consent, purpose, and withdrawal duties"
- <article-3> ::= "Singapore PDPA section 20 notification of purposes"
- <article-4> ::= "Singapore PDPA section 21 access and correction"
- <article-5> ::= "Singapore PDPA section 24 protection obligation"
- <article-6> ::= "Singapore PDPA section 25 retention limitation"
- <article-7> ::= "Singapore PDPA section 26 transfer limitation"
- <article-8> ::= "Singapore PDPA section 26A data breach notification"
- <article-9> ::= "MAS Notice on Technology Risk Management incident reporting provisions for relevant incidents"
- <article-10> ::= "MAS Notice 658 cybersecurity overlay as tenant pack citation in this journey brief"
