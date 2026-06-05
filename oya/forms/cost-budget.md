---
doc_class: CostBudget
microservice: forms
status: Accepted
date: 2026-05-17
owner_team: axis-forms + ops-finops + council-product
review_cadence: monthly
doc_status: published
---

# Forms — Cost Budget

## Per-Tenant Cost Drivers

| Driver | Unit | Cost |
|---|---|---|
| Form authoring (builder session) | per active builder-hour | $0.02 |
| Form publish | per publish | $0.001 |
| Form render | per render (CDN-cached) | $0.00001 |
| Submission | per response | $0.0005 (incl. audit-chain seal, Postgres write, validation) |
| File upload + scan | per MB | $0.0002 |
| AI-form-build (T2) | per LLM round-trip | $0.04 (BYO-LLM-dependent; pack-resident provider) |
| Bulk-distribute | per recipient | $0.0003 (mail/messenger fan-out) |
| Webhook delivery | per delivery | $0.0001 |
| Export | per 1k responses CSV | $0.005 |
| E-signature (QES) | per signed envelope | $0.20 (qualified-cert cost-pass-through) |

## Per-Pack Monthly Operating Cost (GA forecast)

| Pack | Compute | Storage | LLM | CDN+WAF | Total |
|---|---|---|---|---|---|
| pack-kr | $4,500 | $1,200 | $800 | $900 | $7,400 |
| pack-eu | $5,800 | $2,000 | $1,400 | $1,100 | $10,300 |
| pack-us | $6,400 | $2,300 | $1,600 | $1,200 | $11,500 |
| pack-us-healthcare | $4,200 | $1,800 | $1,000 | $900 | $7,900 |
| pack-jp | $3,800 | $900 | $700 | $800 | $6,200 |
| pack-sg | $3,200 | $800 | $600 | $700 | $5,300 |
| pack-au | $4,400 | $1,300 | $900 | $1,000 | $7,600 |
| pack-in | $3,600 | $1,100 | $700 | $800 | $6,200 |
| pack-br | $3,800 | $1,200 | $700 | $900 | $6,600 |
| pack-ae | $3,500 | $1,000 | $600 | $800 | $5,900 |
| pack-ksa | $3,500 | $1,000 | $600 | $800 | $5,900 |
| **Total** | $46,700 | $14,600 | $9,600 | $9,900 | **$80,800 / mo** |

## Gross Margin Target

At GA tier-2 pricing ($25/user/mo with included forms), forms-µservice gross-margin target ≥ 80%. Current model projects 82% at Q2 GA.

## Per-Capability Cost Caps (alerts)

| Capability | Per-tenant cap (monthly) | Alert at |
|---|---|---|
| AI-form-build invocations | 1000 | 800 |
| Bulk-distribute recipients | 100k | 80k |
| Export operations | 1000 | 800 |
| File-upload bytes | 100GB | 80GB |
| Webhook deliveries | 1M | 800k |

Tenant exceeding cap requires explicit Tier-G upsell; soft-cap auto-throttles.

## Cost Tags

Per-cost-tag emitted to FinOps:
- `microservice=forms`
- `pack=<pack>`
- `tenant_id=<hash>`
- `workload_class=F1..F9`
- `tenant_class=T0|T1|T2`

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=forms-cost-tag-conformance` exit 0.
- Monthly FinOps review.

## References

- `capacity-model.md`.
- ADR-0131 per-microservice flat layout (cost accounting per µservice).
- FinOps Foundation framework.
