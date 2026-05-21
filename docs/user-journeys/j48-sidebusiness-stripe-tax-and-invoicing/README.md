---
doc_class: User-Journey-README
journey_id: j48-sidebusiness-stripe-tax-and-invoicing
status: Proposed
date: 2026-05-20
authority_tier: 3
persona: Yejin Park
locale: ko-KR
tenant_scope: yejin-vintage-business
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
  - finops-portal
  - mail
  - compliance
  - connect
journey_number: j48
benchmark: Stripe Tax plus Toss Payments KR-FSS reporting pattern
---

# j48-sidebusiness-stripe-tax-and-invoicing

Purpose: Index and build contract for Side-business Stripe tax and invoicing at KR-FSS threshold.

## Artifact map
- story.md: persona narrative and acceptance story.
- ux-flow.md: screen-by-screen UX flow.
- handshake.md: cross-service handshake, Cedar permits, events, and contracts.
- schemas/kr-fss-tax-filing-packet.json: shared JSON Schema object.
- integration-test-plan.md: end-to-end and failure-injection plan.
- ../../microservices/payments/IP-journey-j48-kr-fss-threshold-ledger.md: payments implementation slice.
- ../../microservices/finops-portal/IP-journey-j48-tax-filing-console.md: finops-portal implementation slice.
- ../../microservices/mail/IP-journey-j48-tax-notice-delivery.md: mail implementation slice.
- ../../microservices/compliance/IP-journey-j48-kr-fss-overlay.md: compliance implementation slice.
- ../../microservices/connect/IP-journey-j48-adp-kr-export.md: connect implementation slice.
## Integration points
- payments: kr-fss-threshold-ledger; emits audit, metrics, logs, and traces per ADR-0263.
- finops-portal: tax-filing-console; emits audit, metrics, logs, and traces per ADR-0263.
- mail: tax-notice-delivery; emits audit, metrics, logs, and traces per ADR-0263.
- compliance: kr-fss-overlay; emits audit, metrics, logs, and traces per ADR-0263.
- connect: adp-kr-export; emits audit, metrics, logs, and traces per ADR-0263.
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
README check 1: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 2: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 3: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 4: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 5: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 6: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 7: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 8: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 9: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 10: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 11: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 12: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 13: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 14: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 15: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 16: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 17: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 18: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 19: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 20: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 21: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 22: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 23: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 24: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 25: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 26: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 27: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 28: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 29: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 30: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 31: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 32: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 33: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 34: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 35: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 36: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 37: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 38: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 39: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 40: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 41: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 42: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 43: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 44: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 45: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 46: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 47: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 48: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 49: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 50: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 51: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 52: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 53: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 54: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 55: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 56: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 57: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 58: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 59: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 60: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 61: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 62: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 63: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 64: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 65: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 66: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 67: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 68: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 69: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 70: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 71: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 72: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 73: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 74: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 75: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 76: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 77: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 78: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 79: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 80: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 81: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 82: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 83: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 84: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 85: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 86: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 87: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 88: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 89: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 90: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 91: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 92: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 93: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 94: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 95: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 96: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 97: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 98: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 99: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 100: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 101: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 102: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 103: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 104: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 105: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 106: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 107: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 108: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 109: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 110: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 111: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 112: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 113: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 114: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 115: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 116: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 117: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 118: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 119: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 120: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 121: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 122: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 123: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 124: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 125: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 126: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 127: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 128: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 129: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 130: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 131: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 132: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 133: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 134: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 135: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 136: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 137: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 138: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 139: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 140: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 141: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 142: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 143: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 144: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 145: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 146: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 147: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 148: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 149: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 150: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 151: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 152: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 153: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 154: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 155: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 156: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 157: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 158: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 159: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 160: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 161: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 162: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 163: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 164: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 165: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 166: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 167: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 168: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 169: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 170: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 171: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 172: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 173: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 174: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 175: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 176: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 177: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 178: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 179: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 180: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 181: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 182: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 183: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 184: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 185: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 186: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 187: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 188: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 189: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 190: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 191: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 192: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 193: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 194: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 195: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 196: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 197: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 198: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 199: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 200: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 201: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 202: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 203: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 204: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 205: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 206: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 207: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 208: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 209: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 210: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 211: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 212: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 213: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 214: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 215: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 216: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 217: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 218: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 219: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 220: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 221: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 222: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 223: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 224: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 225: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 226: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 227: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 228: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 229: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 230: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 231: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 232: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 233: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 234: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 235: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 236: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 237: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 238: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 239: compliance/kr-fss-overlay is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 240: connect/adp-kr-export is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 241: payments/kr-fss-threshold-ledger is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 242: finops-portal/tax-filing-console is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
README check 243: mail/tax-notice-delivery is reachable from this index, bound to j48-sidebusiness-stripe-tax-and-invoicing, and independently buildable under ADR-0131 flat microservice layout.
