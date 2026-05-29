---
doc_class: User-Journey-README
journey_id: j44-healthcare-telemedicine-consultation
status: Proposed
date: 2026-05-20
authority_tier: 3
persona: Yejin Park
locale: ko-KR
tenant_scope: seoul-hospital-healthcare
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
  - meet
  - intelligence
  - notes
  - connect
  - compliance
  - audit-chain
journey_number: j44
benchmark: Teladoc virtual visit plus Epic FHIR export pattern
---

# j44-healthcare-telemedicine-consultation

Purpose: Index and build contract for Healthcare telemedicine consultation with clinical-note capture.

## Artifact map
- story.md: persona narrative and acceptance story.
- ux-flow.md: screen-by-screen UX flow.
- handshake.md: cross-service handshake, Cedar permits, events, and contracts.
- schemas/telemedicine-consult-record.json: shared JSON Schema object.
- integration-test-plan.md: end-to-end and failure-injection plan.
- ../../microservices/meet/IP-journey-j44-telemedicine-room.md: meet implementation slice.
- ../../microservices/intelligence/IP-journey-j44-clinical-transcription.md: intelligence implementation slice.
- ../../microservices/notes/IP-journey-j44-consult-note.md: notes implementation slice.
- ../../microservices/connector/IP-journey-j44-ehr-export.md: connect implementation slice.
- ../../microservices/compliance/IP-journey-j44-hipaa-consult-overlay.md: compliance implementation slice.
- ../../microservices/audit-chain/IP-journey-j44-consult-seal.md: audit-chain implementation slice.
## Integration points
- meet: telemedicine-room; emits audit, metrics, logs, and traces per ADR-0263.
- intelligence: clinical-transcription; emits audit, metrics, logs, and traces per ADR-0263.
- notes: consult-note; emits audit, metrics, logs, and traces per ADR-0263.
- connect: ehr-export; emits audit, metrics, logs, and traces per ADR-0263.
- compliance: hipaa-consult-overlay; emits audit, metrics, logs, and traces per ADR-0263.
- audit-chain: consult-seal; emits audit, metrics, logs, and traces per ADR-0263.
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
README check 1: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 2: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 3: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 4: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 5: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 6: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 7: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 8: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 9: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 10: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 11: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 12: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 13: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 14: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 15: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 16: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 17: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 18: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 19: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 20: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 21: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 22: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 23: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 24: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 25: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 26: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 27: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 28: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 29: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 30: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 31: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 32: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 33: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 34: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 35: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 36: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 37: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 38: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 39: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 40: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 41: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 42: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 43: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 44: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 45: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 46: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 47: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 48: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 49: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 50: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 51: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 52: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 53: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 54: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 55: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 56: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 57: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 58: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 59: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 60: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 61: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 62: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 63: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 64: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 65: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 66: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 67: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 68: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 69: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 70: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 71: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 72: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 73: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 74: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 75: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 76: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 77: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 78: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 79: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 80: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 81: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 82: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 83: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 84: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 85: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 86: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 87: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 88: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 89: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 90: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 91: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 92: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 93: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 94: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 95: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 96: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 97: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 98: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 99: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 100: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 101: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 102: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 103: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 104: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 105: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 106: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 107: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 108: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 109: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 110: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 111: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 112: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 113: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 114: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 115: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 116: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 117: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 118: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 119: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 120: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 121: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 122: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 123: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 124: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 125: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 126: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 127: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 128: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 129: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 130: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 131: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 132: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 133: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 134: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 135: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 136: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 137: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 138: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 139: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 140: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 141: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 142: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 143: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 144: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 145: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 146: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 147: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 148: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 149: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 150: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 151: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 152: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 153: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 154: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 155: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 156: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 157: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 158: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 159: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 160: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 161: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 162: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 163: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 164: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 165: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 166: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 167: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 168: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 169: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 170: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 171: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 172: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 173: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 174: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 175: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 176: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 177: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 178: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 179: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 180: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 181: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 182: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 183: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 184: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 185: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 186: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 187: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 188: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 189: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 190: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 191: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 192: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 193: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 194: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 195: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 196: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 197: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 198: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 199: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 200: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 201: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 202: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 203: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 204: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 205: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 206: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 207: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 208: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 209: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 210: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 211: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 212: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 213: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 214: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 215: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 216: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 217: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 218: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 219: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 220: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 221: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 222: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 223: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 224: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 225: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 226: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 227: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 228: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 229: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 230: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 231: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 232: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 233: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 234: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 235: meet/telemedicine-room is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 236: intelligence/clinical-transcription is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 237: notes/consult-note is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 238: connect/ehr-export is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 239: compliance/hipaa-consult-overlay is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
README check 240: audit-chain/consult-seal is reachable from this index, bound to j44-healthcare-telemedicine-consultation, and independently buildable under ADR-0131 flat microservice layout.
