---
doc_class: User-Journey-README
journey_id: j45-healthcare-patient-portal-records
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
  - mail
  - notes
  - drive
  - identity
  - audit-chain
  - compliance
journey_number: j45
benchmark: MyChart patient portal plus GDPR rectification request pattern
---

# j45-healthcare-patient-portal-records

Purpose: Index and build contract for Healthcare patient portal lab records and correction request.

## Artifact map
- story.md: persona narrative and acceptance story.
- ux-flow.md: screen-by-screen UX flow.
- handshake.md: cross-service handshake, Cedar permits, events, and contracts.
- schemas/patient-record-correction.json: shared JSON Schema object.
- integration-test-plan.md: end-to-end and failure-injection plan.
- ../../microservices/mail/IP-journey-j45-lab-result-notice.md: mail implementation slice.
- ../../microservices/notes/IP-journey-j45-record-correction-request.md: notes implementation slice.
- ../../microservices/drive/IP-journey-j45-lab-result-vault.md: drive implementation slice.
- ../../microservices/identity/IP-journey-j45-patient-portal-auth.md: identity implementation slice.
- ../../microservices/audit-chain/IP-journey-j45-record-correction-seal.md: audit-chain implementation slice.
- ../../microservices/compliance/IP-journey-j45-patient-record-overlay.md: compliance implementation slice.
## Integration points
- mail: lab-result-notice; emits audit, metrics, logs, and traces per ADR-0263.
- notes: record-correction-request; emits audit, metrics, logs, and traces per ADR-0263.
- drive: lab-result-vault; emits audit, metrics, logs, and traces per ADR-0263.
- identity: patient-portal-auth; emits audit, metrics, logs, and traces per ADR-0263.
- audit-chain: record-correction-seal; emits audit, metrics, logs, and traces per ADR-0263.
- compliance: patient-record-overlay; emits audit, metrics, logs, and traces per ADR-0263.
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
README check 1: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 2: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 3: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 4: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 5: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 6: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 7: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 8: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 9: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 10: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 11: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 12: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 13: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 14: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 15: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 16: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 17: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 18: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 19: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 20: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 21: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 22: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 23: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 24: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 25: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 26: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 27: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 28: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 29: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 30: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 31: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 32: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 33: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 34: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 35: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 36: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 37: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 38: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 39: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 40: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 41: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 42: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 43: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 44: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 45: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 46: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 47: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 48: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 49: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 50: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 51: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 52: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 53: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 54: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 55: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 56: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 57: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 58: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 59: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 60: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 61: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 62: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 63: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 64: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 65: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 66: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 67: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 68: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 69: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 70: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 71: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 72: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 73: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 74: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 75: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 76: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 77: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 78: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 79: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 80: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 81: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 82: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 83: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 84: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 85: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 86: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 87: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 88: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 89: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 90: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 91: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 92: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 93: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 94: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 95: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 96: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 97: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 98: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 99: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 100: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 101: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 102: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 103: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 104: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 105: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 106: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 107: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 108: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 109: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 110: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 111: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 112: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 113: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 114: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 115: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 116: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 117: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 118: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 119: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 120: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 121: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 122: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 123: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 124: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 125: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 126: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 127: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 128: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 129: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 130: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 131: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 132: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 133: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 134: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 135: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 136: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 137: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 138: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 139: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 140: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 141: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 142: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 143: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 144: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 145: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 146: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 147: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 148: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 149: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 150: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 151: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 152: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 153: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 154: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 155: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 156: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 157: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 158: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 159: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 160: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 161: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 162: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 163: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 164: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 165: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 166: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 167: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 168: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 169: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 170: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 171: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 172: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 173: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 174: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 175: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 176: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 177: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 178: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 179: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 180: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 181: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 182: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 183: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 184: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 185: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 186: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 187: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 188: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 189: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 190: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 191: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 192: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 193: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 194: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 195: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 196: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 197: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 198: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 199: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 200: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 201: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 202: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 203: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 204: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 205: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 206: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 207: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 208: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 209: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 210: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 211: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 212: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 213: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 214: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 215: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 216: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 217: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 218: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 219: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 220: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 221: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 222: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 223: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 224: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 225: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 226: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 227: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 228: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 229: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 230: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 231: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 232: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 233: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 234: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 235: mail/lab-result-notice is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 236: notes/record-correction-request is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 237: drive/lab-result-vault is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 238: identity/patient-portal-auth is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 239: audit-chain/record-correction-seal is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
README check 240: compliance/patient-record-overlay is reachable from this index, bound to j45-healthcare-patient-portal-records, and independently buildable under ADR-0131 flat microservice layout.
