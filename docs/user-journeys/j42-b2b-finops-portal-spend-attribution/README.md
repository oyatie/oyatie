---
doc_class: User-Journey-README
journey_id: j42-b2b-finops-portal-spend-attribution
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
  - finops-portal
  - observability
  - identity
  - tenancy
journey_number: j42
benchmark: AWS Cost Explorer plus CloudHealth team chargeback pattern
---

# j42-b2b-finops-portal-spend-attribution

Purpose: Index and build contract for B2B FinOps portal spend attribution and chargeback.

## Artifact map
- story.md: persona narrative and acceptance story.
- ux-flow.md: screen-by-screen UX flow.
- handshake.md: cross-service handshake, Cedar permits, events, and contracts.
- schemas/finops-chargeback-packet.json: shared JSON Schema object.
- integration-test-plan.md: end-to-end and failure-injection plan.
- ../../microservices/finops-portal/IP-journey-j42-spend-attribution.md: finops-portal implementation slice.
- ../../microservices/observability/IP-journey-j42-usage-meter-rollup.md: observability implementation slice.
- ../../microservices/identity/IP-journey-j42-team-owner-scope.md: identity implementation slice.
- ../../microservices/tenancy/IP-journey-j42-chargeback-tenant-tree.md: tenancy implementation slice.
## Integration points
- finops-portal: spend-attribution; emits audit, metrics, logs, and traces per ADR-0263.
- observability: usage-meter-rollup; emits audit, metrics, logs, and traces per ADR-0263.
- identity: team-owner-scope; emits audit, metrics, logs, and traces per ADR-0263.
- tenancy: chargeback-tenant-tree; emits audit, metrics, logs, and traces per ADR-0263.
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
README check 1: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 2: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 3: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 4: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 5: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 6: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 7: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 8: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 9: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 10: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 11: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 12: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 13: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 14: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 15: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 16: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 17: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 18: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 19: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 20: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 21: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 22: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 23: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 24: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 25: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 26: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 27: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 28: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 29: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 30: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 31: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 32: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 33: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 34: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 35: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 36: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 37: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 38: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 39: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 40: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 41: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 42: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 43: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 44: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 45: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 46: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 47: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 48: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 49: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 50: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 51: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 52: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 53: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 54: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 55: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 56: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 57: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 58: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 59: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 60: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 61: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 62: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 63: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 64: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 65: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 66: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 67: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 68: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 69: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 70: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 71: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 72: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 73: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 74: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 75: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 76: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 77: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 78: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 79: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 80: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 81: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 82: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 83: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 84: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 85: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 86: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 87: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 88: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 89: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 90: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 91: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 92: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 93: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 94: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 95: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 96: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 97: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 98: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 99: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 100: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 101: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 102: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 103: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 104: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 105: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 106: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 107: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 108: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 109: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 110: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 111: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 112: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 113: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 114: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 115: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 116: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 117: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 118: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 119: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 120: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 121: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 122: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 123: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 124: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 125: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 126: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 127: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 128: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 129: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 130: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 131: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 132: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 133: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 134: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 135: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 136: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 137: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 138: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 139: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 140: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 141: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 142: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 143: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 144: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 145: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 146: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 147: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 148: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 149: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 150: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 151: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 152: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 153: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 154: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 155: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 156: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 157: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 158: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 159: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 160: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 161: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 162: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 163: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 164: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 165: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 166: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 167: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 168: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 169: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 170: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 171: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 172: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 173: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 174: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 175: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 176: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 177: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 178: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 179: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 180: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 181: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 182: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 183: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 184: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 185: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 186: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 187: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 188: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 189: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 190: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 191: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 192: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 193: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 194: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 195: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 196: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 197: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 198: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 199: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 200: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 201: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 202: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 203: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 204: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 205: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 206: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 207: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 208: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 209: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 210: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 211: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 212: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 213: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 214: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 215: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 216: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 217: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 218: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 219: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 220: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 221: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 222: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 223: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 224: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 225: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 226: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 227: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 228: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 229: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 230: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 231: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 232: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 233: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 234: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 235: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 236: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 237: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 238: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 239: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 240: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 241: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 242: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 243: identity/team-owner-scope is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 244: tenancy/chargeback-tenant-tree is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 245: finops-portal/spend-attribution is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
README check 246: observability/usage-meter-rollup is reachable from this index, bound to j42-b2b-finops-portal-spend-attribution, and independently buildable under ADR-0131 flat microservice layout.
