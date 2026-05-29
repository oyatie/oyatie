---
doc_class: User-Journey-README
journey_id: j46-healthcare-prescription-renewal-workflow
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
  - workflow-studio
  - workflow-engine
  - mail
  - identity
  - connect
  - compliance
journey_number: j46
benchmark: Epic MyChart refill request plus pharmacy eRx routing pattern
---

# j46-healthcare-prescription-renewal-workflow

Purpose: Index and build contract for Healthcare prescription renewal routed to doctor and pharmacy.

## Artifact map
- story.md: persona narrative and acceptance story.
- ux-flow.md: screen-by-screen UX flow.
- handshake.md: cross-service handshake, Cedar permits, events, and contracts.
- schemas/prescription-renewal-request.json: shared JSON Schema object.
- integration-test-plan.md: end-to-end and failure-injection plan.
- ../../microservices/workflow-studio/IP-journey-j46-rx-renewal-template.md: workflow-studio implementation slice.
- ../../microservices/workflow-engine/IP-journey-j46-prescriber-routing.md: workflow-engine implementation slice.
- ../../microservices/mail/IP-journey-j46-rx-status-messaging.md: mail implementation slice.
- ../../microservices/identity/IP-journey-j46-patient-prescriber-resolution.md: identity implementation slice.
- ../../microservices/connector/IP-journey-j46-pharmacy-adapter.md: connect implementation slice.
- ../../microservices/compliance/IP-journey-j46-rx-overlay.md: compliance implementation slice.
## Integration points
- workflow-studio: rx-renewal-template; emits audit, metrics, logs, and traces per ADR-0263.
- workflow-engine: prescriber-routing; emits audit, metrics, logs, and traces per ADR-0263.
- mail: rx-status-messaging; emits audit, metrics, logs, and traces per ADR-0263.
- identity: patient-prescriber-resolution; emits audit, metrics, logs, and traces per ADR-0263.
- connect: pharmacy-adapter; emits audit, metrics, logs, and traces per ADR-0263.
- compliance: rx-overlay; emits audit, metrics, logs, and traces per ADR-0263.
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
README check 1: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 2: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 3: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 4: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 5: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 6: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 7: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 8: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 9: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 10: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 11: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 12: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 13: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 14: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 15: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 16: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 17: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 18: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 19: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 20: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 21: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 22: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 23: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 24: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 25: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 26: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 27: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 28: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 29: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 30: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 31: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 32: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 33: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 34: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 35: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 36: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 37: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 38: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 39: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 40: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 41: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 42: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 43: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 44: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 45: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 46: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 47: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 48: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 49: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 50: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 51: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 52: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 53: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 54: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 55: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 56: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 57: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 58: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 59: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 60: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 61: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 62: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 63: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 64: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 65: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 66: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 67: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 68: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 69: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 70: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 71: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 72: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 73: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 74: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 75: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 76: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 77: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 78: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 79: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 80: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 81: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 82: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 83: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 84: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 85: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 86: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 87: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 88: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 89: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 90: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 91: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 92: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 93: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 94: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 95: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 96: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 97: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 98: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 99: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 100: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 101: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 102: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 103: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 104: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 105: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 106: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 107: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 108: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 109: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 110: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 111: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 112: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 113: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 114: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 115: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 116: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 117: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 118: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 119: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 120: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 121: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 122: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 123: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 124: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 125: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 126: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 127: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 128: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 129: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 130: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 131: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 132: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 133: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 134: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 135: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 136: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 137: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 138: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 139: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 140: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 141: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 142: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 143: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 144: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 145: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 146: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 147: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 148: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 149: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 150: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 151: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 152: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 153: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 154: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 155: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 156: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 157: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 158: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 159: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 160: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 161: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 162: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 163: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 164: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 165: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 166: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 167: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 168: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 169: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 170: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 171: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 172: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 173: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 174: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 175: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 176: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 177: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 178: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 179: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 180: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 181: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 182: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 183: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 184: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 185: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 186: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 187: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 188: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 189: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 190: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 191: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 192: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 193: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 194: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 195: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 196: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 197: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 198: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 199: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 200: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 201: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 202: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 203: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 204: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 205: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 206: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 207: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 208: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 209: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 210: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 211: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 212: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 213: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 214: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 215: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 216: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 217: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 218: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 219: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 220: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 221: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 222: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 223: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 224: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 225: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 226: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 227: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 228: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 229: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 230: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 231: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 232: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 233: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 234: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 235: workflow-studio/rx-renewal-template is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 236: workflow-engine/prescriber-routing is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 237: mail/rx-status-messaging is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 238: identity/patient-prescriber-resolution is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 239: connect/pharmacy-adapter is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
README check 240: compliance/rx-overlay is reachable from this index, bound to j46-healthcare-prescription-renewal-workflow, and independently buildable under ADR-0131 flat microservice layout.
