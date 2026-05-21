---
doc_class: BNF-v4.1-Pack-State-Grammar
journey_id: j91-us-state-money-transmitter-licensing
status: draft
date: 2026-05-20
related_adrs:
  - ADR-0105-thirteen-layer-canonical-enum
  - ADR-0251-compliance-pack-cell-certification-levels
---

# j91 BNF v4.1 Pack State Grammar

```bnf
<journey-pack-state> ::= <journey-id> ":" <tenant-id> ":" <pack-set> ":" <cell-cert> ":" <cedar-decision> ":" <audit-event>
<journey-id> ::= "j91"
<pack-set> ::= <pack-id> | <pack-id> "," <pack-set>
<pack-id> ::= "US-MSB" | "US-CA-MTL" | "US-NY-MTL" | "US-TX-MTL" | "US-FL-MTL" | "US-WA-MTL"
<cell-cert> ::= "us-general" | "us-financial-services" | "state-mtl-evidence-ready"
<cedar-decision> ::= "permit" | "forbid" | "abstain"
<audit-event> ::= "EVT-J" <number> "-" <service-token> "-" <sequence>
```

## ADR-0105 13-layer mapping

| Layer | Journey binding |
|---|---|
| experience | j91 threshold detection with pack US-MSB |
| edge | j91 state license gap analysis with pack US-CA-MTL |
| api-rest | j91 surety bond packet with pack US-NY-MTL |
| api-async | j91 NMLS evidence upload with pack US-TX-MTL |
| adapter | j91 Cedar-gated payment throttling with pack US-FL-MTL |
| usecase | j91 regulator renewal calendar with pack US-WA-MTL |
| domain | j91 threshold detection with pack US-MSB |
| kernel | j91 state license gap analysis with pack US-CA-MTL |
| policy | j91 surety bond packet with pack US-NY-MTL |
| eventing | j91 NMLS evidence upload with pack US-TX-MTL |
| observability | j91 Cedar-gated payment throttling with pack US-FL-MTL |
| iac | j91 regulator renewal calendar with pack US-WA-MTL |
| evidence | j91 threshold detection with pack US-MSB |

## Exact article terminals

- <article-1> ::= "31 CFR 1010.100(ff) money transmitter definition"
- <article-2> ::= "31 CFR 1022.210 money services business anti-money-laundering program"
- <article-3> ::= "31 CFR 1022.320 suspicious activity reporting for money services businesses"
- <article-4> ::= "California Financial Code section 2030 license requirement and section 2037 surety/securities obligation"
- <article-5> ::= "New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding"
- <article-6> ::= "Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security"
- <article-7> ::= "Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security"
- <article-8> ::= "Washington RCW 19.230.030 license required and 19.230.050 surety bond"
