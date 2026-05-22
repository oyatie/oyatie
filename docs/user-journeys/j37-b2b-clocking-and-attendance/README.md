---
doc_class: User-Journey-README
journey_id: j37-b2b-clocking-and-attendance
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
  - connect
  - payments
  - identity
  - observability
journey_number: j37
benchmark: Workday Time Tracking plus ADP Workforce Now export pattern
---

# j37-b2b-clocking-and-attendance

Purpose: Index and build contract for B2B clocking and attendance with geofence and ADP export.

## Artifact map
- story.md: persona narrative and acceptance story.
- ux-flow.md: screen-by-screen UX flow.
- handshake.md: cross-service handshake, Cedar permits, events, and contracts.
- schemas/attendance-clock-event.json: shared JSON Schema object.
- integration-test-plan.md: end-to-end and failure-injection plan.
- ../../microservices/workplace-integration/IP-journey-j37-clock-in-geofence.md: workplace-integration implementation slice.
- ../../microservices/connect/IP-journey-j37-adp-payroll-export.md: connect implementation slice.
- ../../microservices/payments/IP-journey-j37-payroll-ledger-hold.md: payments implementation slice.
- ../../microservices/identity/IP-journey-j37-worker-shift-principal.md: identity implementation slice.
- ../../microservices/observability/IP-journey-j37-attendance-slo-traces.md: observability implementation slice.
## Integration points
- workplace-integration: clock-in-geofence; emits audit, metrics, logs, and traces per ADR-0263.
- connect: adp-payroll-export; emits audit, metrics, logs, and traces per ADR-0263.
- payments: payroll-ledger-hold; emits audit, metrics, logs, and traces per ADR-0263.
- identity: worker-shift-principal; emits audit, metrics, logs, and traces per ADR-0263.
- observability: attendance-slo-traces; emits audit, metrics, logs, and traces per ADR-0263.
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
README check 1: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 2: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 3: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 4: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 5: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 6: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 7: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 8: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 9: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 10: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 11: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 12: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 13: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 14: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 15: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 16: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 17: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 18: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 19: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 20: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 21: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 22: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 23: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 24: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 25: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 26: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 27: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 28: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 29: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 30: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 31: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 32: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 33: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 34: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 35: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 36: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 37: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 38: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 39: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 40: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 41: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 42: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 43: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 44: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 45: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 46: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 47: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 48: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 49: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 50: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 51: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 52: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 53: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 54: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 55: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 56: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 57: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 58: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 59: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 60: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 61: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 62: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 63: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 64: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 65: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 66: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 67: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 68: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 69: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 70: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 71: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 72: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 73: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 74: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 75: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 76: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 77: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 78: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 79: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 80: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 81: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 82: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 83: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 84: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 85: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 86: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 87: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 88: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 89: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 90: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 91: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 92: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 93: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 94: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 95: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 96: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 97: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 98: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 99: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 100: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 101: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 102: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 103: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 104: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 105: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 106: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 107: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 108: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 109: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 110: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 111: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 112: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 113: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 114: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 115: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 116: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 117: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 118: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 119: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 120: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 121: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 122: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 123: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 124: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 125: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 126: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 127: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 128: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 129: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 130: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 131: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 132: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 133: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 134: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 135: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 136: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 137: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 138: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 139: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 140: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 141: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 142: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 143: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 144: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 145: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 146: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 147: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 148: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 149: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 150: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 151: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 152: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 153: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 154: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 155: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 156: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 157: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 158: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 159: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 160: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 161: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 162: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 163: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 164: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 165: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 166: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 167: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 168: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 169: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 170: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 171: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 172: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 173: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 174: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 175: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 176: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 177: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 178: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 179: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 180: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 181: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 182: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 183: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 184: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 185: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 186: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 187: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 188: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 189: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 190: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 191: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 192: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 193: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 194: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 195: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 196: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 197: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 198: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 199: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 200: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 201: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 202: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 203: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 204: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 205: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 206: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 207: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 208: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 209: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 210: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 211: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 212: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 213: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 214: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 215: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 216: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 217: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 218: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 219: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 220: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 221: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 222: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 223: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 224: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 225: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 226: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 227: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 228: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 229: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 230: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 231: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 232: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 233: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 234: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 235: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 236: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 237: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 238: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 239: identity/worker-shift-principal is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 240: observability/attendance-slo-traces is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 241: workplace-integration/clock-in-geofence is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 242: connect/adp-payroll-export is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
README check 243: payments/payroll-ledger-hold is reachable from this index, bound to j37-b2b-clocking-and-attendance, and independently buildable under ADR-0131 flat microservice layout.
