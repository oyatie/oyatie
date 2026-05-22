---
doc_class: User-Journey-README
journey_id: j47-healthcare-billing-and-insurance
status: Proposed
date: 2026-05-20
authority_tier: 3
persona: Yejin Park
locale: ko-KR
tenant_scope: yejin-personal-health
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
  - payments
  - connect
  - mail
  - tenancy
  - compliance
journey_number: j47
benchmark: Stripe healthcare payments plus X12 837 insurance-claim submission pattern
---

# j47-healthcare-billing-and-insurance

Purpose: Index and build contract for Healthcare billing and insurance auto-submission.

## Artifact map
- story.md: persona narrative and acceptance story.
- ux-flow.md: screen-by-screen UX flow.
- handshake.md: cross-service handshake, Cedar permits, events, and contracts.
- schemas/healthcare-billing-claim.json: shared JSON Schema object.
- integration-test-plan.md: end-to-end and failure-injection plan.
- ../../microservices/payments/IP-journey-j47-hospital-bill-payment.md: payments implementation slice.
- ../../microservices/connect/IP-journey-j47-insurance-claim-submit.md: connect implementation slice.
- ../../microservices/mail/IP-journey-j47-bill-and-eob-thread.md: mail implementation slice.
- ../../microservices/tenancy/IP-journey-j47-provider-patient-scope.md: tenancy implementation slice.
- ../../microservices/compliance/IP-journey-j47-healthcare-billing-overlay.md: compliance implementation slice.
## Integration points
- payments: hospital-bill-payment; emits audit, metrics, logs, and traces per ADR-0263.
- connect: insurance-claim-submit; emits audit, metrics, logs, and traces per ADR-0263.
- mail: bill-and-eob-thread; emits audit, metrics, logs, and traces per ADR-0263.
- tenancy: provider-patient-scope; emits audit, metrics, logs, and traces per ADR-0263.
- compliance: healthcare-billing-overlay; emits audit, metrics, logs, and traces per ADR-0263.
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
README check 1: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 2: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 3: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 4: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 5: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 6: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 7: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 8: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 9: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 10: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 11: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 12: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 13: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 14: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 15: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 16: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 17: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 18: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 19: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 20: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 21: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 22: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 23: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 24: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 25: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 26: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 27: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 28: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 29: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 30: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 31: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 32: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 33: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 34: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 35: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 36: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 37: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 38: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 39: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 40: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 41: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 42: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 43: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 44: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 45: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 46: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 47: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 48: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 49: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 50: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 51: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 52: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 53: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 54: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 55: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 56: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 57: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 58: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 59: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 60: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 61: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 62: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 63: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 64: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 65: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 66: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 67: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 68: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 69: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 70: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 71: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 72: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 73: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 74: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 75: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 76: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 77: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 78: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 79: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 80: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 81: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 82: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 83: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 84: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 85: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 86: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 87: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 88: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 89: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 90: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 91: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 92: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 93: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 94: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 95: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 96: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 97: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 98: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 99: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 100: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 101: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 102: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 103: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 104: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 105: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 106: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 107: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 108: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 109: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 110: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 111: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 112: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 113: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 114: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 115: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 116: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 117: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 118: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 119: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 120: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 121: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 122: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 123: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 124: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 125: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 126: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 127: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 128: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 129: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 130: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 131: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 132: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 133: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 134: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 135: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 136: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 137: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 138: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 139: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 140: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 141: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 142: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 143: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 144: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 145: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 146: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 147: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 148: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 149: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 150: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 151: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 152: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 153: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 154: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 155: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 156: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 157: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 158: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 159: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 160: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 161: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 162: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 163: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 164: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 165: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 166: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 167: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 168: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 169: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 170: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 171: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 172: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 173: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 174: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 175: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 176: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 177: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 178: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 179: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 180: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 181: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 182: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 183: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 184: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 185: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 186: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 187: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 188: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 189: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 190: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 191: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 192: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 193: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 194: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 195: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 196: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 197: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 198: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 199: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 200: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 201: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 202: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 203: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 204: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 205: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 206: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 207: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 208: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 209: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 210: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 211: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 212: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 213: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 214: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 215: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 216: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 217: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 218: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 219: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 220: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 221: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 222: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 223: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 224: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 225: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 226: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 227: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 228: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 229: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 230: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 231: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 232: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 233: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 234: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 235: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 236: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 237: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 238: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 239: tenancy/provider-patient-scope is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 240: compliance/healthcare-billing-overlay is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 241: payments/hospital-bill-payment is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 242: connect/insurance-claim-submit is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
README check 243: mail/bill-and-eob-thread is reachable from this index, bound to j47-healthcare-billing-and-insurance, and independently buildable under ADR-0131 flat microservice layout.
