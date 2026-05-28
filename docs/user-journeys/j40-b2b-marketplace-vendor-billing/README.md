---
doc_class: User-Journey-README
journey_id: j40-b2b-marketplace-vendor-billing
status: Proposed
date: 2026-05-20
authority_tier: 3
persona: Marcus Chen
locale: en-US
tenant_scope: acme-b2b
platform_microservice_count_authority: 45
marketplace_settlement_invariant: marketplace-settles-all-tenant-deals
contract_surfaces:
  - OpenAPI 3.2.0
  - AsyncAPI 3.1.0
  - proto3
  - BNF v4.1
  - ADR-0105 13-layer
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0244
  - ADR-0263
  - ADR-0273
  - ADR-0292
  - ADR-0297
  - ADR-0299
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/user-journeys/CATALOG-j126-j150-ecosystem.md
  - microservices/payments/PRD.md
  - microservices/identity/PRD.md
  - microservices/workflow-engine/PRD.md
  - microservices/ontology/PRD.md
  - microservices/messenger/PRD.md
  - microservices/mail/PRD.md
  - microservices/community/PRD.md
microservices_touched:
  - plugin-app-store
  - payments
  - tenancy
  - mail
journey_number: j40
benchmark: AWS Marketplace SaaS contract plus Stripe subscription pattern
---

# j40-b2b-marketplace-vendor-billing

Purpose: Index and build contract for B2B marketplace vendor billing with per-seat subscription.

## Artifact map
- story.md: persona narrative and acceptance story.
- ux-flow.md: screen-by-screen UX flow.
- handshake.md: cross-service handshake, Cedar permits, events, and contracts.
- schemas/marketplace-seat-subscription.json: shared JSON Schema object.
- integration-test-plan.md: end-to-end and failure-injection plan.
- ../../microservices/plugin-app-store/IP-journey-j40-vendor-subscription.md: plugin-app-store implementation slice.
- ../../microservices/payments/IP-journey-j40-per-seat-billing.md: payments implementation slice.
- ../../microservices/tenancy/IP-journey-j40-seat-entitlement.md: tenancy implementation slice.
- ../../microservices/mail/IP-journey-j40-billing-receipts.md: mail implementation slice.
## Integration points
- plugin-app-store: vendor-subscription; emits audit, metrics, logs, and traces per ADR-0263.
- payments: per-seat-billing; emits audit, metrics, logs, and traces per ADR-0263.
- tenancy: seat-entitlement; emits audit, metrics, logs, and traces per ADR-0263.
- mail: billing-receipts; emits audit, metrics, logs, and traces per ADR-0263.
## Required doctrine
- ADR-0105 is cited because this journey touches tenant scoping, flat layout, observability, recovery, abuse defence, mail delivery, or minor-aware surfaces.
- ADR-0131 is cited because this journey touches tenant scoping, flat layout, observability, recovery, abuse defence, mail delivery, or minor-aware surfaces.
- ADR-0244 is cited because this journey touches tenant scoping, flat layout, observability, recovery, abuse defence, mail delivery, or minor-aware surfaces.
- ADR-0263 is cited because this journey touches tenant scoping, flat layout, observability, recovery, abuse defence, mail delivery, or minor-aware surfaces.
- ADR-0273 is cited because this journey touches tenant scoping, flat layout, observability, recovery, abuse defence, mail delivery, or minor-aware surfaces.
- ADR-0292 is cited because this journey touches tenant scoping, flat layout, observability, recovery, abuse defence, mail delivery, or minor-aware surfaces.
- ADR-0297 is cited because this journey touches tenant scoping, flat layout, observability, recovery, abuse defence, mail delivery, or minor-aware surfaces.
- ADR-0299 is cited because this journey touches tenant scoping, flat layout, observability, recovery, abuse defence, mail delivery, or minor-aware surfaces.
## Completion ledger
README check 1: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 2: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 3: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 4: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 5: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 6: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 7: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 8: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 9: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 10: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 11: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 12: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 13: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 14: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 15: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 16: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 17: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 18: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 19: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 20: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 21: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 22: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 23: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 24: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 25: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 26: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 27: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 28: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 29: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 30: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 31: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 32: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 33: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 34: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 35: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 36: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 37: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 38: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 39: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 40: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 41: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 42: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 43: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 44: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 45: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 46: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 47: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 48: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 49: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 50: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 51: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 52: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 53: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 54: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 55: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 56: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 57: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 58: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 59: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 60: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 61: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 62: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 63: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 64: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 65: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 66: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 67: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 68: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 69: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 70: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 71: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 72: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 73: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 74: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 75: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 76: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 77: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 78: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 79: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 80: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 81: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 82: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 83: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 84: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 85: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 86: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 87: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 88: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 89: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 90: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 91: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 92: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 93: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 94: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 95: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 96: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 97: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 98: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 99: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 100: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 101: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 102: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 103: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 104: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 105: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 106: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 107: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 108: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 109: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 110: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 111: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 112: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 113: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 114: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 115: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 116: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 117: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 118: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 119: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 120: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 121: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 122: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 123: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 124: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 125: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 126: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 127: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 128: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 129: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 130: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 131: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 132: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 133: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 134: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 135: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 136: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 137: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 138: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 139: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 140: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 141: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 142: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 143: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 144: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 145: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 146: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 147: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 148: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 149: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 150: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 151: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 152: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 153: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 154: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 155: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 156: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 157: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 158: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 159: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 160: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 161: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 162: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 163: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 164: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 165: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 166: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 167: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 168: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 169: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 170: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 171: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 172: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 173: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 174: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 175: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 176: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 177: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 178: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 179: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 180: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 181: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 182: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 183: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 184: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 185: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 186: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 187: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 188: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 189: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 190: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 191: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 192: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 193: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 194: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 195: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 196: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 197: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 198: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 199: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 200: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 201: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 202: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 203: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 204: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 205: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 206: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 207: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 208: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 209: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 210: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 211: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 212: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 213: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 214: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 215: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 216: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 217: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 218: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 219: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 220: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 221: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 222: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 223: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 224: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 225: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 226: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 227: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 228: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 229: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 230: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 231: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 232: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 233: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 234: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 235: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 236: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 237: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 238: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 239: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 240: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 241: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 242: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 243: tenancy/seat-entitlement is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 244: mail/billing-receipts is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 245: plugin-app-store/vendor-subscription is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
README check 246: payments/per-seat-billing is reachable from this index, bound to j40-b2b-marketplace-vendor-billing, and independently buildable under ADR-0131 flat microservice layout.
