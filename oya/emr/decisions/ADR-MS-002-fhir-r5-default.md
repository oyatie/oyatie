---
id: ADR-EMR-MS-002
status: Accepted
deciders: axis-emr, council-architecture, council-clinical, ops-security
date: 2026-05-21
microservice: emr
purpose: Adopt FHIR R5 as the canonical default for the EMR REST API, with FHIR R4 as a compatibility surface served via Accept-Version negotiation. Defer FHIR R6 compatibility shim until R6 finalizes.
related:
  - ADR-EMR-MS-001
  - ADR-EMR-MS-003
  - ADR-0131
  - ADR-0145
  - ADR-0251
  - 21st Century Cures Act / ONC HTI-1 Final Rule
  - TEFCA Common Agreement v2 (2024)
---

# ADR-EMR-MS-002: FHIR R5 as default; R4 as compatibility surface

## Status

Accepted — 2026-05-21.

## Context

oyatie EMR must expose a FHIR API surface. The choice of FHIR version is decisive:

| Version | Status (2026-05) | Adoption | Notes |
|---|---|---|---|
| FHIR R3 (STU3) | Retired | <2% | No new tenants. |
| FHIR R4 (4.0.1) | Normative since 2019 | Most US EHRs; ONC Cures certification baseline | TEFCA Phase 2 mandate; USCDI v4 binds to R4. |
| FHIR R4B (4.3.0) | Normative for select resources | Limited | Niche. |
| FHIR R5 (5.0.0) | Normative since October 2023 (with errata) | Growing — Epic R5 readiness 2025, athena R5 2026 | Improved Patient resource, refined Observation, new SubscriptionTopic, normative MedicationRequest. |
| FHIR R6 | Pre-ballot | None yet | Pending HL7 Working Group |

The 21st Century Cures Act ONC Health IT Certification (HTI-1) mandates FHIR R4 with USCDI v4 as the certification baseline. TEFCA QHIN participation also requires R4. However, R5 carries improvements that oyatie wants by default:

- More precise MedicationRequest dosageInstruction (with `additionalInstruction` codeableReference).
- Subscriptions as first-class `SubscriptionTopic` resources (cleaner than R4's WebSocket Subscription).
- Refined Encounter modeling (Encounter.subject + Encounter.serviceProvider explicit).
- Improved patient identifier normalization.
- Better support for clinical-decision-support (CDS Hooks 2.0 aligns with R5).

oyatie's strategic position is: ship R5 as the default to surpass competitor capability; serve R4 for the regulatory + interoperability ecosystem.

## Decision

1. **FHIR R5 is the canonical default REST representation.** Every EMR REST handler emits FHIR R5 JSON unless the request explicitly negotiates a different version.
2. **FHIR R4 is supported via Accept-Version negotiation.** Clients may send `Accept-Version: 4.0.1` (or use the FHIR Capability Statement `fhirVersion` element) and receive R4-shaped resources. Internal conversion via `oya-emr-rest`'s FHIR-version-bridge middleware.
3. **No simultaneous R3, R4B, or pre-R6 default support.** R3 retired; R4B niche; R6 deferred.
4. **R6 compatibility shim deferred** until HL7 Working Group finalizes R6 normative. The decision will be revisited via successor ADR when R6 ballot closes.
5. **Capability Statement** emitted at `/fhir/metadata` declares both `4.0.1` and `5.0.0` as `fhirVersion` entries with the corresponding RestfulCapabilityMode.

## Bridge layer

The R5↔R4 conversion is per-resource-type. A canonical mapping table lives in `oya-emr-rest/src/fhir_bridge/r5_r4_map.rs`:

```text
Resource             | R5 → R4 changes
---------------------|--------------------------------------------------------
Patient              | identifier slicing preserved; communication[] retained
Encounter            | serviceProvider → R4 location[] derived; period preserved
MedicationRequest    | dosageInstruction.additionalInstruction → R4 patientInstruction string
Observation          | valueAttachment dropped (R5-only); valueCodeableConcept preserved
AllergyIntolerance   | type{allergy|intolerance} → R4 same
DocumentReference    | content.attachment.url path-normalize
ServiceRequest       | category[] preserved; intent enum aligns
DiagnosticReport     | conclusionCode[] preserved
CarePlan             | activity.detail.kind enum aligns
CareTeam             | reasonCode[] preserved
```

Round-tripping is **NOT guaranteed** for R5-only fields; the bridge layer emits a `OperationOutcome.issue[].severity=warning` when fields drop during R5→R4 downgrade.

## Rejected alternatives

- **FHIR R4 as default + R5 as opt-in.** Rejected — most competitor EMRs ship R4-default with no R5 path; oyatie's position is to lead with R5 to capture next-cycle interoperability.
- **R4-only support.** Rejected — surrenders the modern Subscriptions + MedicationRequest semantics.
- **R5-only support.** Rejected — breaks USCDI v4 certification + TEFCA Phase 2.
- **Per-tenant version pinning.** Rejected — adds complex per-tenant maintenance burden; Accept-Version negotiation is industry-standard.
- **Pre-emptive R6 shim.** Rejected — R6 not normative; would carry maintenance cost for an unstable surface.

## Consequences

### Positive

- Tenants and integrations can pick R5 OR R4 per call; oyatie ships ahead of competitor norms.
- TEFCA Phase 2 + USCDI v4 certification path remains open via R4.
- HL7 R5-native features (SubscriptionTopic, refined MedicationRequest) deliver to clinicians today rather than waiting for industry catchup.

### Negative

- Maintaining two FHIR versions requires test parity (round-trip tests in `microservices/emr/tests/integration/fhir_r5_r4_roundtrip/`).
- R5-only fields not perfectly round-trippable to R4; warning-emit pattern adds noise for R4-pinned consumers.
- Internal domain types must be R5-shape-native; R4-shape is a downstream serialization.

### Operational

- Bridge layer is in the `oya-emr-rest` crate; lives under `src/fhir_bridge/`.
- Per-resource R5↔R4 mapping unit tests required; integration tests against HL7 reference Capability Statement.
- Capability Statement endpoint `/fhir/metadata` returns both fhirVersion entries; emitted by `oya-emr-rest` at startup.

### Migration

- Tenants migrating in from Epic / Cerner (R4 default) experience EMR via R4 surface; gradual migration to R5 by reading the `/fhir/metadata` Capability Statement and adopting R5 endpoints incrementally.

## Verification

- `oya gate validate fhir-version-coverage` confirms both R5 + R4 endpoints emit valid resources against the HL7 reference validator.
- `cargo test -p oya-emr-rest --test fhir_r5_r4_roundtrip` exits 0.
- `/fhir/metadata` response includes both fhirVersion entries.

## References

- HL7 FHIR R5 (5.0.0) normative content, October 2023.
- HL7 FHIR R4 (4.0.1) normative content.
- ONC HTI-1 Final Rule (2024) certification baseline.
- TEFCA Common Agreement v2 (2024).
- USCDI v4 (2024) + draft v5.
- ADR-EMR-MS-001 BC decomposition.
- ADR-0131 per-microservice flat layout.
