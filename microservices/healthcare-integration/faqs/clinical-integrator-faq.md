---
doc_class: FAQ
microservice: healthcare-integration
persona: clinical-integrator
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# healthcare-integration — Clinical Integrator FAQ

## Q1: What's the difference between HL7v2.5.1 and HL7v2.7? Which should I use?

HL7v2.5.1 is the most widely deployed in US hospitals (~ 70 % of installations as of 2026). HL7v2.7 added richer demographics (race/ethnicity per CDC OMB-1997), insurance information (IN1/IN2/IN3 segments), and patient-encounter financial detail. Use whichever your source EHR sends — Epic typically defaults to 2.5.1 for ADT, 2.5 for ORM/ORU; Cerner defaults to 2.4 with custom Z-segments; Meditech ranges 2.3-2.5; Allscripts 2.4-2.5.1. The oyatie parser detects MSH-12 and dispatches to the correct version. For new deployments, prefer 2.7 if the source supports it (better demographics + insurance data lossless to FHIR).

## Q2: Why FHIR R5 first-class instead of R4 (which most US payers use)?

Per ONC's Cures Act Final Rule + USCDI versioning, USCDI v3+ requires FHIR R4 minimum, but R5 backward-compatibility shims exist. oyatie ships R5 first-class because (a) R5 has better support for subscription notifications (Subscription resource), (b) better terminology binding (ValueSet expansion), (c) richer questionnaire model (clinical decision support). The R4 backward-compatibility shim auto-down-converts R5 resources for R4-only consumers — your tenant gets the richer authoring model without breaking R4 integrations. If your tenant has a hard R4-only constraint, set the FHIR API server default to R4 via portal → FHIR → "Default version" → R4.

## Q3: A patient says they don't consent to information-sharing with another provider. How do I enforce this?

Use the FHIR Consent resource (paid tenant_class tier). The substrate's consent-cascade engine evaluates every FHIR read against any active Consent for that patient. To deny sharing with `Provider X`:

```http
POST /Consent
{
  "resourceType": "Consent",
  "status": "active",
  "scope": { "coding": [{"system": "http://terminology.hl7.org/CodeSystem/consentscope", "code": "patient-privacy"}] },
  "category": [{"coding": [{"system": "http://terminology.hl7.org/CodeSystem/consentcategorycodes", "code": "ihe-consent"}]}],
  "patient": { "reference": "Patient/<id>" },
  "dateTime": "2026-05-20T14:00:00Z",
  "provision": {
    "type": "deny",
    "actor": [{"role": {"coding": [{"code": "RECIPIENT"}]}, "reference": {"identifier": {"system": "urn:oid:2.16.840.1.113883.4.6", "value": "<NPI-of-Provider-X>"}}}],
    "purpose": [{"code": "TREATMENT"}]
  }
}
```

Any subsequent FHIR read by Provider X (identified by NPI in their OAuth2 bearer token) is denied with HTTP 403 + an audit-chain event. Patient retains the right to revoke the consent at any time (HIPAA §164.508(b)(5) revocation).

## Q4: Our hospital has an in-house EHR that doesn't speak FHIR, only proprietary HL7v2. Can it consume oyatie FHIR data?

Yes via the HL7v2 outbound channel. Portal → Channels → "New channel" → "HL7v2 outbound MLLP". Configure the FHIR-to-HL7v2 mapping (the inverse of what you set up for inbound). Subscribe to FHIR resource change events (via FHIR Subscription) and have the outbound channel emit ADT/ORM/ORU messages to your in-house EHR's MLLP listener.

## Q5: What happens if our HL7 source sends a message with a code from a custom terminology we haven't registered?

The substrate's terminology engine attempts the canonical mappings (SNOMED-CT, LOINC, ICD-10-CM, RxNorm, plus pack-specific overlays). If the code is unrecognised, the message is parsed but the FHIR resource gets a `Coding` element with `system = "urn:oid:<tenant-custom-system>"` + `code = <as-received>` and no `display`. The substrate emits a `terminology::unmapped_code` event to a separate Pulsar topic for triage; your terminology team can then either register the mapping or accept the unmapped state. Critical clinical decision support workflows should NOT operate on unmapped codes — they get filtered out of the CDS engine until mapped.

## Q6: How do I handle deduplication of patients across multiple source EHRs?

Use IHE PIX/PDQ (paid tenant_class tier). The substrate's PIX manager assigns a master patient identifier (Person resource in FHIR) and links source-EHR identifiers (Patient.identifier from each source). PDQ provides query-by-demographics for cross-source matching.

Provisioning:
1. Configure each source EHR's identifier system URL: portal → PIX → "Source systems".
2. Configure matching rules: typically `last_name + first_name + DOB + (SSN-last-4 OR phone)` is the standard.
3. The PIX manager auto-matches on each incoming ADT; manual review queue for ambiguous matches.
4. Query via PDQ: `POST /Patient/$match` with demographics; returns ranked matches with confidence scores.

Common pitfall: identical-name twins. Most PIX managers fail to distinguish; we recommend tenants augment matching with SSN-last-4 or insurance member ID where available.

## Q7: What's the C-STORE throughput limit and how do I size for our PACS migration?

demo_trial: 200 instances/min. paid: 2 000 instances/min. paid: ~ 10 000 instances/min (limited by DICOM cluster + SeaweedFS write throughput). For PACS migrations, calculate study size × studies/day. Example: 5 000 studies/day × 50 instances/study = 250 000 instances/day → ~ 175 instances/min average. Need to handle peaks: typically 3× average. So 525 instances/min peak → paid tier comfortable. If you're migrating 1 M studies historical, the bulk-load uses a separate ingest path (dcm4chee-arc bulk-import) that bypasses the normal C-STORE rate limit; budget ~ 50 GiB/min throughput on the SeaweedFS side.

## Q8: How do I integrate with TEFCA QHIN (Trusted Exchange Framework Common Agreement Qualified Health Information Network)?

paid tier or above. Portal → TEFCA → "Onboard as QHIN sub-participant". The substrate proxies your tenant's TEFCA queries through one of the active QHINs (Epic Nexus, eHealth Exchange, Health Gorilla, etc — pick one). You provide:
- Your tenant's HIPAA covered entity ID + NPI.
- Network of providers (NPIs) participating.
- QHIN of your choice (we provide federation with all 7 designated QHINs as of 2026).
- Use cases: Individual Access Services, Treatment, Payment, Healthcare Operations (FACT-2 categories).

Once onboarded, your tenant can issue TEFCA queries (`/Patient/$find-tefca`) and receive responses from any reachable QHIN participant. Audit-chain events track every cross-network query for FACT-2 reporting.

## Q9: A patient invokes their HIPAA right of access (§164.524) — they want all their records. How do I produce them?

Portal → Patient → search → "Right of Access Export". Filter: patient (matched via PIX), date range (default: lifetime). The substrate produces a structured ZIP:
- All FHIR resources for the patient (Patient, Encounter, Observation, Condition, Procedure, Medication*, AllergyIntolerance, Immunization, ImagingStudy, DiagnosticReport, DocumentReference, etc).
- All DICOM studies with the WADO-RS retrieval URLs.
- All Direct Project messages sent/received.
- Audit log of every access (per HIPAA §164.528 right to accounting).
- A human-readable PDF index.

The export must complete within 30 days per §164.524(b)(2); typically completes within 24 h. The patient receives a secure download URL via the `notifications` µservice + email + optional SMS. Chain-of-custody anchored to audit-chain.

## Q10: We're a clinic in Korea on KR-PIPA-Health. Can our doctors view records from our patients' US providers?

Cross-border PHI sharing is gated by both KR-PIPA Art. 17 (cross-border personal data transfer) + KR Medical Service Act § 21 + HIPAA §164.508 + HHS guidance on international transfers. The technically supported path:
1. Patient provides written consent (Korean + English) per KR-PIPA Art. 17(2). Stored as FHIR Consent resource with `provision.purpose = ['cross-border-transfer']`.
2. US provider is identified by NPI + HIPAA covered entity status.
3. The substrate's NCPeH gateway (paid tier; pack-bound) sends a Patient Summary request to the US side via TEFCA + receives the summary.
4. The summary is rendered for the Korean clinician but flagged "Cross-Border Data" + retains the original transfer-consent reference.

Typical turnaround: 24-48 h depending on the US provider's QHIN. Not all US providers participate; if the patient's provider doesn't, the patient can pursue HIPAA Right of Access (Q9) themselves and upload the records to your Korean clinic manually.

## Q11: Our PACS sends DICOM SR (Structured Reports) and SC (Secondary Captures). Do these become FHIR resources too?

Yes. DICOM SR → DiagnosticReport (FHIR) with the SR content rendered as both a HTML attachment and structured Observations where the SR template is recognised (TID 1500 for measurement reports is supported; specialist templates like TID 1500.1 for breast imaging require registration). DICOM SC → ImagingStudy + Binary FHIR resource for the JPEG/PNG content. The dcm4chee-arc → FHIR converter is in `crates/oya-healthcare-integration-dicom2fhir-kernel`.

## Q12: We need to disable a channel because the source EHR is sending malformed messages flooding our DLQ. How do I stop the bleeding fast?

Portal → Channels → your channel → "Disable" (red button). The MLLP listener stops accepting connections within 5 s. The source EHR's TCP connection is reset; the EHR's outbound queue starts buffering on its side. Resolve the malformed-message issue (work with the source EHR vendor), then re-enable the channel. The substrate buffers messages from the EHR side for up to 1 h before the EHR's side starts dropping; longer outages need coordination with the EHR vendor to pause + resume their outbound queue. For emergency situations, use `oya healthcare-integration channel-quarantine --channel <id> --reason "<reason>"` which auto-pages your team + the substrate oncall.
