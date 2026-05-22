---
doc_class: User-Journey-README
journey_id: j43-healthcare-nurse-patient-handoff
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
  - notes
  - identity
  - intelligence
  - ontology
  - audit-chain
  - compliance
journey_number: j43
benchmark: Epic handoff report plus Palantir Foundry ontology projection pattern
---

# j43-healthcare-nurse-patient-handoff

Purpose: Index and build contract for Healthcare nurse shift-end handoff for eight patient cases.

## Artifact map
- story.md: persona narrative and acceptance story.
- ux-flow.md: screen-by-screen UX flow.
- handshake.md: cross-service handshake, Cedar permits, events, and contracts.
- schemas/clinical-handoff-bundle.json: shared JSON Schema object.
- integration-test-plan.md: end-to-end and failure-injection plan.
- ../../microservices/notes/IP-journey-j43-shift-handoff-note.md: notes implementation slice.
- ../../microservices/identity/IP-journey-j43-nurse-break-glass-scope.md: identity implementation slice.
- ../../microservices/intelligence/IP-journey-j43-clinical-summary-assist.md: intelligence implementation slice.
- ../../microservices/ontology/IP-journey-j43-patient-read-path.md: ontology implementation slice.
- ../../microservices/audit-chain/IP-journey-j43-hipaa-seal.md: audit-chain implementation slice.
- ../../microservices/compliance/IP-journey-j43-hipaa-cell-overlay.md: compliance implementation slice.
## Integration points
- notes: shift-handoff-note; emits audit, metrics, logs, and traces per ADR-0263.
- identity: nurse-break-glass-scope; emits audit, metrics, logs, and traces per ADR-0263.
- intelligence: clinical-summary-assist; emits audit, metrics, logs, and traces per ADR-0263.
- ontology: patient-read-path; emits audit, metrics, logs, and traces per ADR-0263.
- audit-chain: hipaa-seal; emits audit, metrics, logs, and traces per ADR-0263.
- compliance: hipaa-cell-overlay; emits audit, metrics, logs, and traces per ADR-0263.
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
README check 1: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 2: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 3: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 4: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 5: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 6: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 7: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 8: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 9: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 10: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 11: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 12: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 13: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 14: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 15: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 16: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 17: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 18: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 19: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 20: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 21: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 22: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 23: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 24: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 25: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 26: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 27: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 28: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 29: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 30: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 31: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 32: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 33: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 34: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 35: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 36: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 37: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 38: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 39: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 40: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 41: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 42: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 43: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 44: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 45: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 46: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 47: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 48: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 49: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 50: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 51: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 52: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 53: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 54: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 55: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 56: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 57: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 58: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 59: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 60: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 61: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 62: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 63: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 64: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 65: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 66: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 67: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 68: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 69: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 70: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 71: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 72: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 73: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 74: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 75: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 76: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 77: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 78: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 79: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 80: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 81: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 82: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 83: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 84: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 85: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 86: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 87: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 88: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 89: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 90: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 91: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 92: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 93: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 94: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 95: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 96: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 97: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 98: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 99: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 100: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 101: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 102: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 103: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 104: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 105: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 106: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 107: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 108: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 109: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 110: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 111: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 112: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 113: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 114: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 115: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 116: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 117: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 118: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 119: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 120: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 121: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 122: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 123: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 124: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 125: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 126: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 127: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 128: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 129: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 130: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 131: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 132: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 133: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 134: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 135: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 136: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 137: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 138: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 139: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 140: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 141: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 142: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 143: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 144: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 145: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 146: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 147: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 148: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 149: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 150: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 151: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 152: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 153: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 154: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 155: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 156: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 157: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 158: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 159: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 160: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 161: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 162: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 163: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 164: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 165: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 166: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 167: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 168: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 169: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 170: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 171: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 172: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 173: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 174: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 175: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 176: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 177: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 178: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 179: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 180: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 181: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 182: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 183: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 184: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 185: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 186: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 187: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 188: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 189: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 190: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 191: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 192: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 193: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 194: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 195: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 196: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 197: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 198: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 199: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 200: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 201: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 202: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 203: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 204: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 205: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 206: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 207: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 208: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 209: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 210: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 211: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 212: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 213: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 214: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 215: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 216: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 217: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 218: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 219: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 220: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 221: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 222: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 223: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 224: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 225: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 226: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 227: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 228: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 229: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 230: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 231: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 232: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 233: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 234: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 235: notes/shift-handoff-note is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 236: identity/nurse-break-glass-scope is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 237: intelligence/clinical-summary-assist is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 238: ontology/patient-read-path is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 239: audit-chain/hipaa-seal is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
README check 240: compliance/hipaa-cell-overlay is reachable from this index, bound to j43-healthcare-nurse-patient-handoff, and independently buildable under ADR-0131 flat microservice layout.
