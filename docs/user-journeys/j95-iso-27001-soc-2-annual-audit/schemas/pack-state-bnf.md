---
doc_class: BNF-v4.1-Pack-State-Grammar
journey_id: j95-iso-27001-soc-2-annual-audit
status: draft
date: 2026-05-20
related_adrs:
  - ADR-0105-thirteen-layer-canonical-enum
  - ADR-0251-compliance-pack-cell-certification-levels
---

# j95 BNF v4.1 Pack State Grammar

```bnf
<journey-pack-state> ::= <journey-id> ":" <tenant-id> ":" <pack-set> ":" <cell-cert> ":" <cedar-decision> ":" <audit-event>
<journey-id> ::= "j95"
<pack-set> ::= <pack-id> | <pack-id> "," <pack-set>
<pack-id> ::= "ISO-27001-2022" | "ISO-22301-2019" | "SOC-2-TYPE-II"
<cell-cert> ::= "global-enterprise-assurance" | "business-continuity-ready"
<cedar-decision> ::= "permit" | "forbid" | "abstain"
<audit-event> ::= "EVT-J" <number> "-" <service-token> "-" <sequence>
```

## ADR-0105 13-layer mapping

| Layer | Journey binding |
|---|---|
| experience | j95 scope confirmation with pack ISO-27001-2022 |
| edge | j95 evidence collector mapping with pack ISO-22301-2019 |
| api-rest | j95 control owner attestation with pack SOC-2-TYPE-II |
| api-async | j95 business continuity exercise proof with pack ISO-27001-2022 |
| adapter | j95 auditor portal freeze with pack ISO-22301-2019 |
| usecase | j95 findings remediation loop with pack SOC-2-TYPE-II |
| domain | j95 scope confirmation with pack ISO-27001-2022 |
| kernel | j95 evidence collector mapping with pack ISO-22301-2019 |
| policy | j95 control owner attestation with pack SOC-2-TYPE-II |
| eventing | j95 business continuity exercise proof with pack ISO-27001-2022 |
| observability | j95 auditor portal freeze with pack ISO-22301-2019 |
| iac | j95 findings remediation loop with pack SOC-2-TYPE-II |
| evidence | j95 scope confirmation with pack ISO-27001-2022 |

## Exact article terminals

- <article-1> ::= "ISO/IEC 27001:2022 clauses 4 through 10 and Annex A controls A.5, A.6, A.7, A.8"
- <article-2> ::= "ISO/IEC 27002:2022 Annex A implementation guidance for organizational, people, physical, and technological controls"
- <article-3> ::= "ISO 22301:2019 clauses 8.4 incident response, 8.5 business continuity plans, and 8.6 exercise program"
- <article-4> ::= "AICPA SOC 2 Trust Services Criteria CC1 through CC9"
- <article-5> ::= "SOC 2 availability criteria A1.1 through A1.3"
- <article-6> ::= "SOC 2 confidentiality criteria C1.1 through C1.2"
- <article-7> ::= "SOC 2 processing integrity PI1.1 through PI1.5"
- <article-8> ::= "SOC 2 privacy criteria P1.1 through P8.1"
