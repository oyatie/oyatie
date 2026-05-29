---
doc_class: Onboarding
microservice: healthcare-integration
persona: clinical-integrator
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# healthcare-integration — Clinical Integrator First Week

Audience: an HL7/FHIR integration engineer joining a tenant's clinical-systems team. You have hospital/EHR background and Mirth/Cloverleaf experience; you may be new to oyatie.

## Day 1 — orientation + BAA sign-off + access

Morning (3 h):

1. Sign the oyatie BAA (Business Associate Agreement) per HIPAA §164.502(e). Without a signed BAA, no PHI may flow through the substrate. The BAA is provisioned via the `contract-lifecycle-management` µservice; your tenant admin handles the signing.
2. Receive `iam` invite. Cedar role `healthcare-integration::clinical-integrator` binds: `hl7::channel::{read,write,deploy}`, `fhir::resource::{read,write}`, `fhir::profile::{read,publish}`, `audit::healthcare::read`.
3. Log in to the integration portal at `https://healthcare-integration.<tenant>.oyatie.io`.
4. Verify the BAA appears in your tenant's compliance dashboard with a valid signed-date.

Afternoon (4 h):

5. Read the substrate primer: portal → Help → "Healthcare Integration 101" (~ 45 min).
6. Read the HL7v2 channel architecture overview: `microservices/healthcare-integration/ARCHITECTURE.md` § "HL7 channel architecture".
7. Read the FHIR R5 profiles in use: portal → FHIR → Profiles (you'll see USCDI v4 + tenant-specific profiles).
8. List the existing HL7v2 channels (if any) — for each: source system, message types subscribed, throughput, target FHIR mapping.

End of Day 1 deliverable: BAA signed + access verified + channel inventory in `inventory/hl7-channels.md`.

## Day 2 — first HL7v2 channel (ADT-A01 from a hospital EHR)

Morning (4 h):

1. Define the channel: portal → Channels → "New channel" → "HL7v2 inbound MLLP".
2. Configure:
   - Source: MLLP listener on port 6661 (substrate-managed; TLS via mutual-cert auth).
   - Source EHR: configure on the EHR side to send ADT messages to `tcp+mllp://hl7-ingest.<tenant>.oyatie.io:6661` with the provided client certificate.
   - Message types: ADT^A01 (admit), ADT^A02 (transfer), ADT^A03 (discharge), ADT^A08 (update), ADT^A11/A12/A13 (cancel), ADT^A23 (delete).
   - Parsing: HL7v2.5.1 (most common for ADT in the US); auto-detect MSH-12.
   - Validation: strict mode for required fields (MSH-3 sending app, MSH-5 receiving app, MSH-7 timestamp, MSH-10 message control ID, MSH-11 processing ID).
   - On parse failure: send NAK (negative ack) with error code; emit `hl7.parse_failure` to audit-chain.
3. Save the channel as a draft. Configuration is NOT live until you deploy it.

Afternoon (3 h):

4. Configure the FHIR mapping for ADT^A01 → Patient + Encounter:
   - PID-3 (patient identifier) → Patient.identifier with the system URL `http://<source-ehr>.example.com/patient-mrn`.
   - PID-5 (patient name) → Patient.name (family + given).
   - PID-7 (DOB) → Patient.birthDate.
   - PID-8 (sex) → Patient.gender (with terminology binding M→male, F→female, U→unknown, O→other).
   - PID-11 (address) → Patient.address.
   - PV1-2 (patient class) → Encounter.class (I=inpatient, O=outpatient, E=emergency).
   - PV1-3 (assigned location) → Encounter.location.
   - PV1-44 (admit date) → Encounter.period.start.
   - PV1-45 (discharge date) → Encounter.period.end (typically empty for A01; populated for A03).
5. Configure the upsert behaviour: if Patient.identifier already exists, merge; if Encounter for the same Encounter.identifier exists, update; else create.

End of Day 2 deliverable: channel defined + FHIR mapping configured + ready for test deploy.

## Day 3 — channel deploy + smoke test

Morning (3 h):

1. Deploy the channel to dev: portal → Channels → your channel → "Deploy to dev". The substrate provisions the MLLP listener + persists the channel config + restarts the affected Mirth/Camel routes.
2. Verify deployment: portal → Channels → status = "Deployed". Listener is now accepting connections.
3. Test from the EHR side: have the EHR admin send a test ADT^A01 message.
4. Observe the ingest:
   - portal → Channels → your channel → "Live stream". See incoming messages with green checkmarks.
   - audit-chain query: `oya audit-chain query --tenant <tenant-id> --event-class hl7::message::received --since "1 hour ago"`.
5. Query the resulting FHIR resources: `curl https://fhir.<tenant>.oyatie.io/Patient?identifier=<mrn>` (with your bearer token). You should see the Patient + linked Encounter.

Afternoon (4 h):

6. Replay test: have the EHR send 100 sample ADT messages over 10 minutes. Verify:
   - Ingest rate matches send rate (no message lag).
   - Each message produces the correct FHIR resources.
   - Failed messages (deliberate bad data) produce NAKs + audit-chain failure events.
7. Read the dead-letter queue handling: portal → Channels → "Dead letter queue". Failed messages stay here for 7 days; you can manually retry, edit, or discard.

End of Day 3 deliverable: channel live in dev + 100-message smoke test green + DLQ workflow understood.

## Day 4 — FHIR consumer + Smart-on-FHIR

Morning (4 h):

1. Browse the FHIR API: portal → FHIR → "API browser". See the OpenAPI / Swagger spec. Try a GET `/Patient`, POST `/Patient`, GET `/Encounter?patient=<id>`.
2. Author a Smart-on-FHIR app registration: portal → FHIR → "Smart Apps" → "Register new app". Provide: redirect URI, scope (e.g. `patient/Patient.read user/Encounter.read launch/patient`), launch URL.
3. The portal returns a client_id + client_secret. The Smart App can now authenticate via OAuth2 + receive a bearer token bound to the launching user's Cedar permissions.

Afternoon (3 h):

4. Test a Smart-on-FHIR launch:
   - From the portal, click "Launch test app" → enter a test patient MRN.
   - The substrate generates a Smart launch context (patient_id, encounter_id, OAuth state).
   - Your Smart App receives the launch + exchanges code for token + queries FHIR with the patient context.
   - Verify the app only sees the patient's data, not other patients.
5. Read the FHIR audit log: every read by the Smart App emits a FHIR AuditEvent resource (per IHE ATNA) — `curl https://fhir.<tenant>.oyatie.io/AuditEvent?agent.who.identifier=<smart-app-id>`.

End of Day 4 deliverable: FHIR API understood + 1 Smart-on-FHIR test app registered + ATNA audit trail verified.

## Day 5 — DICOM + Direct Project + go-live

Morning (4 h):

1. (If DICOM in scope) Configure DICOM ingest: portal → DICOM → "Add SCP node". Provide AE-Title, port, calling-AE allowlist.
2. Test C-STORE: from a PACS or DICOM workstation, send a study to your AE-Title. Verify:
   - Receipt in audit-chain (`dicom::cstore::received`).
   - The ImagingStudy FHIR resource auto-created.
   - WADO-RS retrieval works (browser-side viewer integration).
3. (If Direct Project in scope) Configure Direct mailbox: portal → Direct → "Provision Direct address". Choose `<role>@direct.<tenant>.oyatie.io` (e.g. `intake@direct.acme-hospital.com`). The substrate provisions the mailbox + DirectTrust-rooted certificate.

Afternoon (4 h):

4. Promote the channel to staging: portal → Channels → your channel → "Promote to staging". Per ADR-0130 the promotion gate requires the SLO evidence + the channel-deploy drill green.
5. Coordinate with the EHR side for staging cutover: production EHR sends to staging endpoint; verify 24 h of clean operation.
6. Document the channel: `microservices/healthcare-integration/channels/<source-ehr>-adt-flow.md` — describe source, mappings, retention, contact info, escalation path.

End of Week 1 deliverable: at least 1 HL7 channel live (dev + staging), FHIR API + Smart-on-FHIR demonstrated, DICOM/Direct Project provisioned (if applicable), documentation committed.

## What you should know by end of week 1

- HL7v2 channel authoring + deployment.
- FHIR R5 resource model + USCDI v4 profiles.
- Smart-on-FHIR app launch + OAuth2 flow.
- DICOM C-STORE + WADO-RS basics.
- Direct Project mailbox + DirectTrust certs.
- Audit-chain ATNA emission for every PHI access.
- Dead-letter queue + retry workflow.

## What you should NOT do in week 1

- Don't disable IHE ATNA audit emission. Every PHI access MUST be audited per HIPAA Security Rule §164.312(b).
- Don't reduce HL7 message retention below 6 y minimum (HIPAA + state pediatric records may require longer; check your tenant's pack overlay).
- Don't deploy a channel directly to production. Always: dev → staging → prod with at least 7 days of clean staging operation.
- Don't store PHI in a FHIR resource extension without registering the extension; substrate validation rejects unknown extensions.
- Don't disable consent-cascade enforcement (paid tenant_class feature). Patient consent governs read access; bypassing it is an immediate HIPAA violation.
- Don't accept HL7v2 messages over plaintext TCP. MLLP-over-TLS with mutual-cert auth is the substrate-enforced minimum.
