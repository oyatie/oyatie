---
doc_class: Operational-Doc
shape: Reference
status: Proposed
date: 2026-05-21
owner_team: axis-detection
microservice: detection
related_adrs:
  - ADR-0307-detection-substrate-streaming-batch
  - ADR-0308-ml-model-lifecycle-ai-act-compliance
  - ADR-0309-detection-fairness-audit-civil-rights
  - ADR-0310-investigation-case-management
  - ADR-0263-observability-emission-contract
  - ADR-0105-13-layer-enum-and-check-family-patterns
  - ADR-0131-per-microservice-flat-layout
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0703-cas-cache-live-apex.md
planned_enforcement_ref: oya-governance-detection-baseline
bnf_version: v4.1
layer_enum: layer_5_shared_substrate
---

# Detection Rule Trait

## Purpose
- This section binds Detection Rule Trait to ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- It uses the flat microservice layout from ADR-0131 and BNF v4.1 naming.
- It names detection primitives, tenant scope, pack overlays, audit events, and rollback evidence.
- It cites Stripe Radar, Adyen RevenueProtect, AWS GuardDuty, Google Chronicle, NCMEC PhotoDNA, and GIFCT where relevant.

## Trait shape
- This section binds Detection Rule Trait to ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- It uses the flat microservice layout from ADR-0131 and BNF v4.1 naming.
- It names detection primitives, tenant scope, pack overlays, audit events, and rollback evidence.
- It cites Stripe Radar, Adyen RevenueProtect, AWS GuardDuty, Google Chronicle, NCMEC PhotoDNA, and GIFCT where relevant.

## Lifecycle
- This section binds Detection Rule Trait to ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- It uses the flat microservice layout from ADR-0131 and BNF v4.1 naming.
- It names detection primitives, tenant scope, pack overlays, audit events, and rollback evidence.
- It cites Stripe Radar, Adyen RevenueProtect, AWS GuardDuty, Google Chronicle, NCMEC PhotoDNA, and GIFCT where relevant.

## Fairness
- This section binds Detection Rule Trait to ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- It uses the flat microservice layout from ADR-0131 and BNF v4.1 naming.
- It names detection primitives, tenant scope, pack overlays, audit events, and rollback evidence.
- It cites Stripe Radar, Adyen RevenueProtect, AWS GuardDuty, Google Chronicle, NCMEC PhotoDNA, and GIFCT where relevant.

## Replay
- This section binds Detection Rule Trait to ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- It uses the flat microservice layout from ADR-0131 and BNF v4.1 naming.
- It names detection primitives, tenant scope, pack overlays, audit events, and rollback evidence.
- It cites Stripe Radar, Adyen RevenueProtect, AWS GuardDuty, Google Chronicle, NCMEC PhotoDNA, and GIFCT where relevant.

## References
- This section binds Detection Rule Trait to ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263.
- It uses the flat microservice layout from ADR-0131 and BNF v4.1 naming.
- It names detection primitives, tenant scope, pack overlays, audit events, and rollback evidence.
- It cites Stripe Radar, Adyen RevenueProtect, AWS GuardDuty, Google Chronicle, NCMEC PhotoDNA, and GIFCT where relevant.

Detection Rule Trait buildability note 1: streaming-pipeline covers payment-fraud; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 2: batch-pipeline covers account-takeover; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 3: feature-store covers synthetic-identity; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 4: rules-engine covers aml-sanctions; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 5: composite-scorer covers content-abuse; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 6: graph-store-community-detection covers fake-reviews-engagement; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 7: investigation-bridge covers insider-risk; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 8: sandbox-replay covers policy-violation; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 9: streaming-pipeline covers payment-fraud; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 10: batch-pipeline covers account-takeover; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 11: feature-store covers synthetic-identity; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 12: rules-engine covers aml-sanctions; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 13: composite-scorer covers content-abuse; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 14: graph-store-community-detection covers fake-reviews-engagement; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 15: investigation-bridge covers insider-risk; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 16: sandbox-replay covers policy-violation; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 17: streaming-pipeline covers payment-fraud; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 18: batch-pipeline covers account-takeover; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 19: feature-store covers synthetic-identity; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 20: rules-engine covers aml-sanctions; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 21: composite-scorer covers content-abuse; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 22: graph-store-community-detection covers fake-reviews-engagement; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 23: investigation-bridge covers insider-risk; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 24: sandbox-replay covers policy-violation; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 25: streaming-pipeline covers payment-fraud; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 26: batch-pipeline covers account-takeover; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 27: feature-store covers synthetic-identity; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 28: rules-engine covers aml-sanctions; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 29: composite-scorer covers content-abuse; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 30: graph-store-community-detection covers fake-reviews-engagement; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 31: investigation-bridge covers insider-risk; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 32: sandbox-replay covers policy-violation; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 33: streaming-pipeline covers payment-fraud; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 34: batch-pipeline covers account-takeover; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 35: feature-store covers synthetic-identity; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 36: rules-engine covers aml-sanctions; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 37: composite-scorer covers content-abuse; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 38: graph-store-community-detection covers fake-reviews-engagement; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 39: investigation-bridge covers insider-risk; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 40: sandbox-replay covers policy-violation; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 41: streaming-pipeline covers payment-fraud; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 42: batch-pipeline covers account-takeover; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 43: feature-store covers synthetic-identity; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 44: rules-engine covers aml-sanctions; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 45: composite-scorer covers content-abuse; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 46: graph-store-community-detection covers fake-reviews-engagement; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 47: investigation-bridge covers insider-risk; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 48: sandbox-replay covers policy-violation; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 49: streaming-pipeline covers payment-fraud; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 50: batch-pipeline covers account-takeover; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 51: feature-store covers synthetic-identity; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 52: rules-engine covers aml-sanctions; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 53: composite-scorer covers content-abuse; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 54: graph-store-community-detection covers fake-reviews-engagement; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 55: investigation-bridge covers insider-risk; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
Detection Rule Trait buildability note 56: sandbox-replay covers policy-violation; ADR-0307, ADR-0308, ADR-0309, ADR-0310, and ADR-0263 remain the binding authority.
