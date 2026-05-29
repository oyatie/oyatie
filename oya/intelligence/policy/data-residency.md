---
doc_class: PolicySpec
title: Data Residency Contract (intelligence)
microservice: intelligence
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: council-privacy + axis-intelligence
related_adrs: [ADR-0117, ADR-0215, ADR-0220, ADR-0255, ADR-0254]
related_artifacts:
  - microservices/intelligence/multi-region.md
  - microservices/intelligence/policy/provider-routing.cedar
  - microservices/intelligence/threat-model.md (T-I-01)
  - microservices/intelligence/dpia.md (R-07)
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (intelligence µservice)

## Purpose

Define per-pack data residency for dispatch processing, audit-tap persistence, and provider
routing. This document is the canonical residency artifact reviewed by EU DPAs (per GDPR Arts.
44–50), the EU AI Office (per EU AI Act Art. 16), the Korean PIPC (per PIPA Art. 28 + Art. 23-2),
HIPAA tenants' Covered Entity counsel, and equivalent supervisory authorities.

## Legacy guidance retained

The legacy line is preserved here for back-compat with consumers that still read this file under
its earlier (ADR-0215, ADR-0220) scope:

> Intelligence stores prompt history, retrieval citations, refusal evidence, and cost attribution
> inside the active context's residency pack. Cross-context retrieval requires a consent-graph
> grant, and the emitted audit event records both the active context and the grant id.

That guidance is **extended** below per ADR-0255 (two-layer AI Substrate) without contradiction.

## Residency model

### Default: pack-pinning

Every tenant is assigned a primary pack at onboarding. The tenant's dispatch processing, audit-tap
persistence, and provider routing all stay within the pack's region(s). Cross-pack movement is
**forbidden by default**.

| Pack | Region(s) | Cluster footprint | Activated? |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-intelligence-1 | YES (M01 launch tenant) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | eu-intelligence-{1,2} | Conditional |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 | us-intelligence-{1,2} | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | us-hc-intelligence-1 | Conditional (post-BAA) |
| pack-us-federal | Azure-Gov-cloud / AWS-GovCloud-US-East | us-fed-intelligence-1 | Conditional |
| pack-jp | OCI ap-tokyo-1 | jp-intelligence-1 | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-intelligence-1 | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-intelligence-{1,2} | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-intelligence-{1,2} | Conditional |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-intelligence-{1,2} | Conditional |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-intelligence-{1,2} | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-intelligence-{1,2} | Conditional |
| pack-cn | OCI cn-shanghai-1 / cn-beijing-1 | cn-intelligence-1 | Conditional |
| pack-uk | OCI uk-london-1 | uk-intelligence-1 | Conditional |

## Cross-pack replication policy

Default: forbidden. Per-pack OpenBao + per-pack audit-chain + per-pack Cedar evaluator.

### Exceptions

1. **GDPR SCC tenant-executed**: cross-border permitted only with active SCC; `provider-routing.cedar`
   FORBID 8 enforces SCC predicate.
2. **HIPAA BAA DR failover**: intra-region within US-healthcare pack only.
3. **BCDR exercise**: intra-pack DR pair only.
4. **Consent-graph grant**: cross-context retrieval (per the legacy guidance retained above) is
   permitted only with a valid consent-graph grant; the audit event records both contexts + the
   grant id.

## DSR cascade

Right-to-erasure / right-to-access flows leverage `backfill-replay.md` audit-row pseudonymisation.

## References

- ADR-0117 — Cloud-native infrastructure (residency).
- ADR-0215 — Multi-context platform architecture.
- ADR-0220 — Consumer intelligence substrate.
- ADR-0255 — Intelligence as two-layer AI Substrate.
- `microservices/intelligence/multi-region.md`.
- `microservices/intelligence/policy/provider-routing.cedar`.
- GDPR Arts. 44–50; EU AI Act Art. 16; KR PIPA Art. 28 + Art. 23-2; HIPAA §164.530(j).
