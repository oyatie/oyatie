---
doc_class: BNF-v4.1-Pack-State-Grammar
journey_id: j98-au-privacy-apra-cps-234-tenant
status: draft
date: 2026-05-20
related_adrs:
  - ADR-0105-thirteen-layer-canonical-enum
  - ADR-0251-compliance-pack-cell-certification-levels
---

# j98 BNF v4.1 Pack State Grammar

```bnf
<journey-pack-state> ::= <journey-id> ":" <tenant-id> ":" <pack-set> ":" <cell-cert> ":" <cedar-decision> ":" <audit-event>
<journey-id> ::= "j98"
<pack-set> ::= <pack-id> | <pack-id> "," <pack-set>
<pack-id> ::= "AU-PRIVACY-ACT" | "APRA-CPS-234" | "AU-IRAP-PROTECTED"
<cell-cert> ::= "au-irap-protected" | "au-financial-services"
<cedar-decision> ::= "permit" | "forbid" | "abstain"
<audit-event> ::= "EVT-J" <number> "-" <service-token> "-" <sequence>
```

## ADR-0105 13-layer mapping

| Layer | Journey binding |
|---|---|
| experience | j98 AU tenant eligibility with pack AU-PRIVACY-ACT |
| edge | j98 APP notice and consent bind with pack APRA-CPS-234 |
| api-rest | j98 IRAP PROTECTED cell placement with pack AU-IRAP-PROTECTED |
| api-async | j98 CPS 234 asset classification with pack AU-PRIVACY-ACT |
| adapter | j98 APRA notification drill with pack APRA-CPS-234 |
| usecase | j98 OAIC breach packet rehearsal with pack AU-IRAP-PROTECTED |
| domain | j98 AU tenant eligibility with pack AU-PRIVACY-ACT |
| kernel | j98 APP notice and consent bind with pack APRA-CPS-234 |
| policy | j98 IRAP PROTECTED cell placement with pack AU-IRAP-PROTECTED |
| eventing | j98 CPS 234 asset classification with pack AU-PRIVACY-ACT |
| observability | j98 APRA notification drill with pack APRA-CPS-234 |
| iac | j98 OAIC breach packet rehearsal with pack AU-IRAP-PROTECTED |
| evidence | j98 AU tenant eligibility with pack AU-PRIVACY-ACT |

## Exact article terminals

- <article-1> ::= "Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information"
- <article-2> ::= "APP 3 collection of solicited personal information"
- <article-3> ::= "APP 5 notification of collection"
- <article-4> ::= "APP 6 use or disclosure"
- <article-5> ::= "APP 8 cross-border disclosure"
- <article-6> ::= "APP 11 security of personal information"
- <article-7> ::= "APP 12 access and APP 13 correction"
- <article-8> ::= "Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification"
- <article-9> ::= "APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls"
- <article-10> ::= "APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification"
