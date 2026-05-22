---
doc_class: User-Journey-README
journey_id: j38-b2b-e-signing-contract
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
  - workplace-integration
  - drive
  - audit-chain
  - mail
  - identity
journey_number: j38
benchmark: DocuSign envelope plus Adobe Acrobat Sign audit-certificate pattern
---

# j38-b2b-e-signing-contract

Purpose: Index and build contract for B2B e-signing contract with regulator audit-chain seal.

## Artifact map
- story.md: persona narrative and acceptance story.
- ux-flow.md: screen-by-screen UX flow.
- handshake.md: cross-service handshake, Cedar permits, events, and contracts.
- schemas/esign-contract-envelope.json: shared JSON Schema object.
- integration-test-plan.md: end-to-end and failure-injection plan.
- ../../microservices/workplace-integration/IP-journey-j38-e-sign-session.md: workplace-integration implementation slice.
- ../../microservices/drive/IP-journey-j38-contract-record-archive.md: drive implementation slice.
- ../../microservices/audit-chain/IP-journey-j38-regulator-seal.md: audit-chain implementation slice.
- ../../microservices/mail/IP-journey-j38-counterparty-envelope.md: mail implementation slice.
- ../../microservices/identity/IP-journey-j38-external-signer-resolution.md: identity implementation slice.
## Integration points
- workplace-integration: e-sign-session; emits audit, metrics, logs, and traces per ADR-0263.
- drive: contract-record-archive; emits audit, metrics, logs, and traces per ADR-0263.
- audit-chain: regulator-seal; emits audit, metrics, logs, and traces per ADR-0263.
- mail: counterparty-envelope; emits audit, metrics, logs, and traces per ADR-0263.
- identity: external-signer-resolution; emits audit, metrics, logs, and traces per ADR-0263.
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
README check 1: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 2: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 3: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 4: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 5: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 6: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 7: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 8: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 9: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 10: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 11: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 12: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 13: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 14: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 15: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 16: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 17: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 18: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 19: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 20: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 21: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 22: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 23: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 24: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 25: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 26: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 27: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 28: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 29: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 30: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 31: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 32: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 33: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 34: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 35: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 36: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 37: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 38: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 39: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 40: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 41: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 42: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 43: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 44: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 45: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 46: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 47: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 48: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 49: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 50: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 51: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 52: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 53: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 54: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 55: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 56: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 57: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 58: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 59: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 60: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 61: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 62: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 63: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 64: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 65: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 66: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 67: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 68: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 69: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 70: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 71: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 72: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 73: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 74: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 75: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 76: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 77: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 78: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 79: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 80: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 81: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 82: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 83: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 84: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 85: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 86: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 87: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 88: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 89: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 90: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 91: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 92: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 93: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 94: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 95: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 96: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 97: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 98: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 99: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 100: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 101: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 102: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 103: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 104: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 105: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 106: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 107: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 108: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 109: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 110: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 111: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 112: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 113: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 114: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 115: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 116: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 117: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 118: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 119: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 120: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 121: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 122: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 123: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 124: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 125: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 126: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 127: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 128: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 129: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 130: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 131: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 132: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 133: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 134: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 135: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 136: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 137: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 138: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 139: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 140: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 141: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 142: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 143: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 144: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 145: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 146: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 147: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 148: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 149: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 150: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 151: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 152: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 153: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 154: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 155: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 156: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 157: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 158: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 159: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 160: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 161: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 162: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 163: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 164: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 165: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 166: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 167: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 168: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 169: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 170: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 171: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 172: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 173: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 174: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 175: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 176: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 177: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 178: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 179: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 180: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 181: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 182: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 183: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 184: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 185: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 186: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 187: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 188: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 189: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 190: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 191: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 192: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 193: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 194: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 195: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 196: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 197: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 198: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 199: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 200: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 201: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 202: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 203: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 204: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 205: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 206: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 207: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 208: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 209: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 210: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 211: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 212: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 213: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 214: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 215: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 216: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 217: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 218: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 219: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 220: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 221: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 222: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 223: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 224: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 225: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 226: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 227: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 228: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 229: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 230: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 231: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 232: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 233: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 234: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 235: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 236: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 237: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 238: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 239: mail/counterparty-envelope is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 240: identity/external-signer-resolution is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 241: workplace-integration/e-sign-session is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 242: drive/contract-record-archive is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
README check 243: audit-chain/regulator-seal is reachable from this index, bound to j38-b2b-e-signing-contract, and independently buildable under ADR-0131 flat microservice layout.
