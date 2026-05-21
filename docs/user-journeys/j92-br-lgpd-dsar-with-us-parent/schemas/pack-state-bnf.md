---
doc_class: BNF-v4.1-Pack-State-Grammar
journey_id: j92-br-lgpd-dsar-with-us-parent
status: draft
date: 2026-05-20
related_adrs:
  - ADR-0105-thirteen-layer-canonical-enum
  - ADR-0251-compliance-pack-cell-certification-levels
---

# j92 BNF v4.1 Pack State Grammar

```bnf
<journey-pack-state> ::= <journey-id> ":" <tenant-id> ":" <pack-set> ":" <cell-cert> ":" <cedar-decision> ":" <audit-event>
<journey-id> ::= "j92"
<pack-set> ::= <pack-id> | <pack-id> "," <pack-set>
<pack-id> ::= "BR-LGPD" | "US-CCPA" | "EU-GDPR"
<cell-cert> ::= "br-sovereign" | "us-parent-restricted" | "eu-transfer-reviewed"
<cedar-decision> ::= "permit" | "forbid" | "abstain"
<audit-event> ::= "EVT-J" <number> "-" <service-token> "-" <sequence>
```

## ADR-0105 13-layer mapping

| Layer | Journey binding |
|---|---|
| experience | j92 LGPD request intake with pack BR-LGPD |
| edge | j92 US parent inventory discovery with pack US-CCPA |
| api-rest | j92 higher-restriction floor calculation with pack EU-GDPR |
| api-async | j92 portability bundle build with pack BR-LGPD |
| adapter | j92 ANPD-ready incident audit with pack US-CCPA |
| usecase | j92 Portuguese response delivery with pack EU-GDPR |
| domain | j92 LGPD request intake with pack BR-LGPD |
| kernel | j92 US parent inventory discovery with pack US-CCPA |
| policy | j92 higher-restriction floor calculation with pack EU-GDPR |
| eventing | j92 portability bundle build with pack BR-LGPD |
| observability | j92 ANPD-ready incident audit with pack US-CCPA |
| iac | j92 Portuguese response delivery with pack EU-GDPR |
| evidence | j92 LGPD request intake with pack BR-LGPD |

## Exact article terminals

- <article-1> ::= "LGPD Law 13.709/2018 Article 6 purpose, adequacy, necessity, transparency, security principles"
- <article-2> ::= "LGPD Article 7 lawful bases for personal data processing"
- <article-3> ::= "LGPD Article 11 sensitive personal data processing"
- <article-4> ::= "LGPD Article 18 data-subject rights including access, correction, anonymization, portability, deletion, and revocation"
- <article-5> ::= "LGPD Article 33 international transfer conditions"
- <article-6> ::= "LGPD Article 38 data protection impact report authority"
- <article-7> ::= "LGPD Article 46 security measures"
- <article-8> ::= "LGPD Article 48 security incident communication"
- <article-9> ::= "California Civil Code sections 1798.100, 1798.105, 1798.110, 1798.115, 1798.130 CCPA/CPRA rights"
- <article-10> ::= "GDPR Articles 15, 17, 20, 22, and 33 for overlapping EU subject records"
