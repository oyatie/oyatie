---
ip_id: IP-EMR-002
title: Build per-BC use-case + application composition layers
microservice: emr
status: planned
date: 2026-05-21
sequence: 2
depends_on: [IP-EMR-001]
unblocks: [IP-EMR-004, IP-EMR-005, IP-EMR-006]
estimated_effort_hours: 100
owner: axis-emr
---

# IP-EMR-002: Use-case + application composition layers

## Goal

Implement the use-case orchestration layer and the application composition root for each BC. Use cases are port-bound; application crates wire concrete adapters.

## Deliverables

30 crates:

```
oya-emr-<bc>-usecase            (15 crates)
oya-emr-<bc>-application        (15 crates)
```

Use-case examples (per BC):

- `patient`: CreatePatient, ReadPatient, SearchPatient, MergePatient, UnmergePatient, DeidentifyPatient.
- `encounter`: StartEncounter, TransferEncounter, DischargeEncounter, ReopenEncounter, ReadEncounter.
- `problem`: AddProblem, AmendProblem, ResolveProblem, SearchProblem.
- `medication`: PrescribeMedication, ReconcileMedication, DiscontinueMedication, RefillMedication, CheckMedicationInteraction.
- `allergy`: RecordAllergy, RefuteAllergy, SearchAllergy.
- `vital`: RecordVital, StreamVital, TrendVital.
- `note`: DraftNote, AutosaveNote, SignNote, AmendNote, CoSignNote.
- `order`: EnterOrder, EnterOrderSet, VerifyOrder, CancelOrder.
- `result`: ReceiveResult, ReviewResult, AcknowledgeResult.
- `care-team`: AssignCareTeam, DischargeCareTeam, ReadCareTeam.
- `order-set`: AuthorOrderSet, PublishOrderSet, DeprecateOrderSet, RetireOrderSet.
- `documentation`: AuthorTemplate, AuthorSmartPhrase, ExpandDotPhrase.
- `billing-code`: CaptureBillingCodes, PhysicianAttest, CoderFinalize.
- `patient-education`: AssignPatientEducation, AcknowledgePatientEducation.
- `portal-session`: PortalLogin, InitiateProxyGrant, FhirRead, BulkExport.

## Acceptance criteria

- 30 crates compile.
- Each use case has at least one unit test against a mock port.
- `cargo check --workspace` exits 0.
- Cedar policy stubs in `microservices/emr/policies/` referenced by use cases via the policy-engine port.

## Out of scope

- Persistent adapters (IP-EMR-003).
- REST/gRPC/AsyncAPI external surfaces (IP-EMR-004 / 005 / 006).
