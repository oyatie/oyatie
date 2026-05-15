---
purpose: "Take Cloud axis to public GA — marketplace open, ISV onboarding, multi-AZ failover automation, FinOps surfaces, 99.99% SLA commitment."
---

---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M05-P01
title: Cloud-Stable GA (Marketplace + ISV + Multi-AZ + FinOps)
status: stub
purpose: Take Cloud axis to public GA — marketplace open, ISV onboarding, multi-AZ failover automation, FinOps surfaces, 99.99% SLA commitment.
---

# M05-P01 — Cloud-Stable GA

## Purpose
Per [`../../../../../docs/ROADMAP.md`](../../../../../docs/ROADMAP.md) §2.8 W-Cloud-Stable.

## Acceptance
- Public Cloud SLA committed at 99.99%.
- Marketplace catalog public; ≥ 10 ISV listings.
- Multi-AZ failover automation drilled quarterly.
- FinOps surface exposes per-tenant per-axis cost allocation with anomaly detection.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Public-GA marketplace + ISV onboarding | stub | [`IP-001-public-marketplace-isv.md`](IP-001-public-marketplace-isv.md) |
| IP-002 | Multi-AZ failover automation + quarterly drill | stub | [`IP-002-multi-az-failover.md`](IP-002-multi-az-failover.md) |
| IP-003 | FinOps surface public-facing | stub | [`IP-003-finops-public.md`](IP-003-finops-public.md) |

## Estimated parallelism
3 agents.

## Symbols-touched
`crates/oya-cloud-marketplace-{api,app}-*`, `crates/oya-cloud-multi-az-failover-*`, `crates/oya-cloud-finops-{api,app}-*`.

## Agent-handoff
```
icm store -t context-oyatie -c "M05-P01 complete: Cloud-Stable GA with 99.99% SLA + marketplace + multi-AZ failover" -i critical -k "M05,P01,cloud-stable,ga,complete"
```
