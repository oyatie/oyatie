---
doc_class: BNF-v4.1-Pack-State-Grammar
journey_id: j94-sox-404-public-company-controls
status: draft
date: 2026-05-20
related_adrs:
  - ADR-0105-thirteen-layer-canonical-enum
  - ADR-0251-compliance-pack-cell-certification-levels
---

# j94 BNF v4.1 Pack State Grammar

```bnf
<journey-pack-state> ::= <journey-id> ":" <tenant-id> ":" <pack-set> ":" <cell-cert> ":" <cedar-decision> ":" <audit-event>
<journey-id> ::= "j94"
<pack-set> ::= <pack-id> | <pack-id> "," <pack-set>
<pack-id> ::= "SOX-404" | "DODD-FRANK-WHISTLEBLOWER"
<cell-cert> ::= "us-public-company-controls" | "audit-readiness"
<cedar-decision> ::= "permit" | "forbid" | "abstain"
<audit-event> ::= "EVT-J" <number> "-" <service-token> "-" <sequence>
```

## ADR-0105 13-layer mapping

| Layer | Journey binding |
|---|---|
| experience | j94 control inventory import with pack SOX-404 |
| edge | j94 segregation-of-duties graph with pack DODD-FRANK-WHISTLEBLOWER |
| api-rest | j94 quarterly evidence close with pack SOX-404 |
| api-async | j94 management certification packet with pack DODD-FRANK-WHISTLEBLOWER |
| adapter | j94 external auditor read-only portal with pack SOX-404 |
| usecase | j94 whistleblower protected intake with pack DODD-FRANK-WHISTLEBLOWER |
| domain | j94 control inventory import with pack SOX-404 |
| kernel | j94 segregation-of-duties graph with pack DODD-FRANK-WHISTLEBLOWER |
| policy | j94 quarterly evidence close with pack SOX-404 |
| eventing | j94 management certification packet with pack DODD-FRANK-WHISTLEBLOWER |
| observability | j94 external auditor read-only portal with pack SOX-404 |
| iac | j94 whistleblower protected intake with pack DODD-FRANK-WHISTLEBLOWER |
| evidence | j94 control inventory import with pack SOX-404 |

## Exact article terminals

- <article-1> ::= "Sarbanes-Oxley Act section 302 issuer officer certifications"
- <article-2> ::= "Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting"
- <article-3> ::= "15 U.S.C. 7262 SOX 404 management assessment and auditor attestation"
- <article-4> ::= "Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting"
- <article-5> ::= "Sarbanes-Oxley Act section 806 whistleblower anti-retaliation"
- <article-6> ::= "Sarbanes-Oxley Act section 802 records destruction penalties"
- <article-7> ::= "Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection"
- <article-8> ::= "SEC Rule 21F-17 anti-impediment to whistleblower communication"
