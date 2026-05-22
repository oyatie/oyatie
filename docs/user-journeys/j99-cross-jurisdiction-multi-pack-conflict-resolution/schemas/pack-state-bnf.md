---
doc_class: BNF-v4.1-Pack-State-Grammar
journey_id: j99-cross-jurisdiction-multi-pack-conflict-resolution
status: draft
date: 2026-05-20
related_adrs:
  - ADR-0105-thirteen-layer-canonical-enum
  - ADR-0251-compliance-pack-cell-certification-levels
---

# j99 BNF v4.1 Pack State Grammar

```bnf
<journey-pack-state> ::= <journey-id> ":" <tenant-id> ":" <pack-set> ":" <cell-cert> ":" <cedar-decision> ":" <audit-event>
<journey-id> ::= "j99"
<pack-set> ::= <pack-id> | <pack-id> "," <pack-set>
<pack-id> ::= "EU-GDPR" | "US-CCPA" | "KR-PIPA" | "AU-PRIVACY-ACT"
<cell-cert> ::= "eu-sovereign" | "us-ccpa-ready" | "kr-csap" | "au-irap-protected"
<cedar-decision> ::= "permit" | "forbid" | "abstain"
<audit-event> ::= "EVT-J" <number> "-" <service-token> "-" <sequence>
```

## ADR-0105 13-layer mapping

| Layer | Journey binding |
|---|---|
| experience | j99 data lineage discovery with pack EU-GDPR |
| edge | j99 pack conflict graph with pack US-CCPA |
| api-rest | j99 higher-restriction floor selection with pack KR-PIPA |
| api-async | j99 Cedar deny-wins simulation with pack AU-PRIVACY-ACT |
| adapter | j99 transparency report publication with pack EU-GDPR |
| usecase | j99 regulator evidence partitioning with pack US-CCPA |
| domain | j99 data lineage discovery with pack KR-PIPA |
| kernel | j99 pack conflict graph with pack AU-PRIVACY-ACT |
| policy | j99 higher-restriction floor selection with pack EU-GDPR |
| eventing | j99 Cedar deny-wins simulation with pack US-CCPA |
| observability | j99 transparency report publication with pack KR-PIPA |
| iac | j99 regulator evidence partitioning with pack AU-PRIVACY-ACT |
| evidence | j99 data lineage discovery with pack EU-GDPR |

## Exact article terminals

- <article-1> ::= "GDPR Article 5 principles, Article 6 lawful basis, Article 15 access, Article 17 erasure, Article 20 portability, Article 22 automated decision safeguards, Article 33 breach notification"
- <article-2> ::= "California Civil Code sections 1798.100, 1798.105, 1798.110, 1798.115, 1798.120, 1798.130 CCPA/CPRA rights"
- <article-3> ::= "Korean PIPA Articles 15, 17, 21, 22, 23, 29, 34, 35, 36, 37 data processing, transfer, security, breach, access, correction, suspension rights"
- <article-4> ::= "Privacy Act 1988 APP 1, APP 3, APP 5, APP 6, APP 8, APP 11, APP 12, APP 13 and Part IIIC eligible data breach notification"
- <article-5> ::= "ADR-0304 higher-restriction-pack-floor-wins conflict rule"
- <article-6> ::= "ADR-0251 cell certification levels and cross-pack Cedar gate"
- <article-7> ::= "ADR-0263 audit-event class requirements for every cross-pack decision"
