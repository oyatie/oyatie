---
doc_class: Tutorial
microservice: healthcare-integration
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Tutorial — Ingest an HL7v2 ORM lab order and publish a FHIR ServiceRequest

Goal: build an end-to-end flow where a hospital EHR sends an HL7v2 ORM^O01 (lab order) message to oyatie, the substrate parses it, creates a FHIR ServiceRequest, validates against USCDI v4, and emits a FHIR Subscription notification to a downstream lab information system.

Prereqs:

- BAA signed.
- `healthcare-integration::clinical-integrator` Cedar role.
- paid tier or higher.
- An EHR system that can send HL7v2.5.1 ORM messages over MLLP-over-TLS.
- ~ 3 hours.

## Step 1 — set up the MLLP listener

Portal → Channels → "New channel" → "HL7v2 inbound MLLP".

Configure:
- Channel name: `ehr-x-orm-inbound`.
- Listener: port 6662 (substrate-managed; mTLS required).
- TLS certificate: download the substrate's listener cert + issue your EHR a client cert via the `kms` µservice (`oya kms issue-cert --purpose=hl7-client --subject CN=ehr-x-prod`).
- Source IP allowlist: your EHR's outbound IP block.
- HL7 version: 2.5.1.
- Message types: subscribe to `ORM^O01`.
- Application acks: AA (accept) / AE (error) / AR (reject). Use AA for successful ingest, AE for transient errors (substrate will retry on the EHR side), AR for validation failures (won't retry).
- Strict validation: require MSH-9 message type, MSH-10 control ID, MSH-11 processing ID, ORC-1 order control (NW=new, CA=cancel, etc), OBR-2 placer order number, OBR-4 universal service ID.

Save the channel as draft.

## Step 2 — author the FHIR mapping

The ORM^O01 segments to map:
- MSH (header) → metadata only (not in FHIR ServiceRequest).
- PID (patient ID) → Patient resource (resolve or create via PIX).
- PV1 (visit) → Encounter resource (resolve or create).
- ORC (order common) → ServiceRequest.identifier (ORC-2 + ORC-3), ServiceRequest.status (ORC-5: A=active, C=completed, CA=cancelled, IP=in-progress, DC=discontinued, ER=erroneous).
- OBR (order request) → ServiceRequest.code (OBR-4: universal service ID → LOINC binding), ServiceRequest.requester (OBR-16: ordering provider → Practitioner resource), ServiceRequest.priority (OBR-27.6 priority code → routine/urgent/asap/stat), ServiceRequest.occurrenceDateTime (OBR-7: observation date/time), ServiceRequest.reasonCode (OBR-31: reason for study).
- NTE (notes) → ServiceRequest.note (text only).

Portal → Channels → your channel → "FHIR Mapping" → "ORM^O01 → ServiceRequest". The visual mapper shows the HL7 message tree on the left + FHIR resource tree on the right; drag fields to map.

Cross-references:
- OBR-4 (universal service ID) → look up in the LOINC ConceptMap; if not found, fall back to the EHR's source code with `system = urn:oid:<source-EHR-OID>`.
- ORC-12 (ordering provider) → look up in the Practitioner registry by NPI (PID-3.4 or via demographics matching); if not found, create with `qualification.system = "http://hl7.org/fhir/sid/us-npi"`.

Validation: USCDI v4 ServiceRequest must have:
- `status` (active / completed / etc)
- `intent` (order)
- `code` (with at least one CodeableConcept binding to LOINC if possible)
- `subject` (reference to Patient)

Save the mapping.

## Step 3 — deploy to dev

Portal → Channels → your channel → "Deploy to dev". The substrate provisions the MLLP listener within 30 s.

Verify: `kubectl -n healthcare-integration get pods | grep hl7-mllp-listener-orm`. You should see a healthy pod.

## Step 4 — send a test message

Use the HL7 message tester:

```sh
cat <<'EOF' > test-orm.hl7
MSH|^~\&|EHR-X|HOSP-A|OYATIE|TENANT-ACME|20260520143000||ORM^O01|MSG-12345|P|2.5.1
PID|||MR-12345^^^EHR-X^MR||DOE^JOHN^A||19800101|M|||123 Main St^^Boston^MA^02101^USA||617-555-1212
PV1||I|2W^201^A|||||||MED||||||||VST-78901
ORC|NW|PLA-001|FILL-001|GROUP-001|A||||20260520143000|||1234567890^SMITH^ALICE^^^^DR||||||||HOSP-A
OBR|1|PLA-001|FILL-001|2951-2^Sodium^LN||20260520143000|20260520143000||||||Fasting since midnight||1234567890^SMITH^ALICE||||||||LAB|||^^^20260520143000^^R
EOF

oya healthcare-integration hl7-send-test \
    --channel ehr-x-orm-inbound \
    --env dev \
    --input test-orm.hl7
```

Expected output:

```
Channel: ehr-x-orm-inbound (dev)
Message sent: 1
ACK received: AA (control_id=MSG-12345)
FHIR resources created:
  - Patient/Patient_001H... (matched existing via PIX on MR-12345)
  - Encounter/Encounter_001H... (created from VST-78901)
  - ServiceRequest/ServiceRequest_001H... (created from PLA-001/FILL-001)
  - Practitioner/Practitioner_001H... (resolved NPI 1234567890)
Validation: USCDI v4 conformant ✔
Audit events emitted: 4 (hl7.message.received, fhir.patient.upserted, fhir.encounter.upserted, fhir.servicerequest.created)
```

## Step 5 — verify in FHIR

```sh
curl -H "Authorization: Bearer $OYA_FHIR_TOKEN" \
    https://fhir.dev.<tenant>.oyatie.io/ServiceRequest/ServiceRequest_001H...
```

Returns the structured FHIR resource:

```json
{
  "resourceType": "ServiceRequest",
  "id": "ServiceRequest_001H...",
  "status": "active",
  "intent": "order",
  "priority": "routine",
  "code": {
    "coding": [
      {
        "system": "http://loinc.org",
        "code": "2951-2",
        "display": "Sodium [Moles/volume] in Serum or Plasma"
      }
    ]
  },
  "subject": { "reference": "Patient/Patient_001H..." },
  "encounter": { "reference": "Encounter/Encounter_001H..." },
  "requester": { "reference": "Practitioner/Practitioner_001H..." },
  "authoredOn": "2026-05-20T14:30:00Z",
  "reasonCode": [{"text": "Fasting since midnight"}],
  "identifier": [
    {"system": "urn:oid:1.2.3.4.5.6.7.8", "value": "PLA-001"},
    {"system": "urn:oid:1.2.3.4.5.6.7.9", "value": "FILL-001"}
  ]
}
```

## Step 6 — subscribe a downstream LIS

Lab Information System wants to be notified when ServiceRequest is created. Create a FHIR Subscription:

```sh
curl -H "Authorization: Bearer $OYA_FHIR_TOKEN" \
     -H "Content-Type: application/fhir+json" \
     -X POST \
     https://fhir.dev.<tenant>.oyatie.io/Subscription \
     -d '{
       "resourceType": "Subscription",
       "status": "active",
       "topic": "http://oyatie.io/fhir/subscription-topic/servicerequest-created",
       "channel": {
         "type": "rest-hook",
         "endpoint": "https://lis.your-tenant.com/fhir-webhook",
         "header": ["Authorization: Bearer <LIS-tokenized-secret>"]
       }
     }'
```

The next ServiceRequest created in this tenant fires a POST to the LIS endpoint with the resource payload. Verify via the LIS-side logs + the audit-chain `fhir::subscription::notification_sent` events.

## Step 7 — test the cancel flow

Send an ORC-1=CA cancel message for the same order:

```sh
cat <<'EOF' > test-orm-cancel.hl7
MSH|^~\&|EHR-X|HOSP-A|OYATIE|TENANT-ACME|20260520150000||ORM^O01|MSG-12346|P|2.5.1
PID|||MR-12345^^^EHR-X^MR||DOE^JOHN^A||19800101|M
ORC|CA|PLA-001|FILL-001|GROUP-001|CA||||20260520150000|||1234567890^SMITH^ALICE^^^^DR||||||||HOSP-A
OBR|1|PLA-001|FILL-001|2951-2^Sodium^LN
EOF

oya healthcare-integration hl7-send-test \
    --channel ehr-x-orm-inbound \
    --env dev \
    --input test-orm-cancel.hl7
```

The substrate detects the existing ServiceRequest by identifier (PLA-001/FILL-001), updates `status` to `revoked` (per FHIR R5 semantics for cancelled orders), and emits another subscription notification.

## Step 8 — promote to staging

Once the dev flow is green for 24 h with at least 100 messages processed cleanly:

```sh
oya healthcare-integration channel-promote \
    --channel ehr-x-orm-inbound \
    --from dev --to staging \
    --evidence "100-msg-clean-24h dev-environment"
```

The substrate's promotion gate verifies:
- ≥ 100 messages processed in dev with ≥ 99 % parse success rate.
- ≥ 24 h continuous operation without channel restart.
- DLQ at < 1 % of total messages.
- All FHIR resources pass USCDI v4 validation.
- IHE ATNA audit-chain events emitted for every message + every FHIR write.

If green, promotion lifts within 5 min. If red, the gate explains what's missing.

## What you've built

A production HL7v2 ORM → FHIR ServiceRequest flow with:
- mTLS-secured MLLP listener.
- Strict HL7v2.5.1 parsing + USCDI v4 FHIR R5 mapping.
- LOINC + NPI terminology binding.
- PIX-based patient identity resolution.
- FHIR Subscription notification to a downstream LIS.
- IHE ATNA audit-trail emission for compliance.
- Cancel-order handling.
- Promotion-gate evidence chain.

## Common pitfalls

| Pitfall | Mitigation |
|---|---|
| OBR-4 universal service ID not in LOINC | Configure source-EHR-specific terminology in portal → Terminology → "Source overlays" before going live |
| Plaintext MLLP (no TLS) | Substrate refuses to provision a non-TLS listener; configure the EHR side properly |
| Patient matching failure for PIX | Tune matching rules; for hospitals with frequent demographic mismatches, lower the auto-match threshold + raise the manual-review queue |
| ORC-12 ordering provider has no NPI | Some hospitals use local provider IDs; configure the substrate to accept local IDs with `system = urn:oid:<local-provider-OID>` and map to NPI lazily |
| Subscription endpoint requires non-standard headers | The Subscription.channel.header array supports arbitrary headers; substrate sends them as-is |
| Cancel comes before create due to network reordering | Substrate buffers cancels for ≤ 5 min waiting for the create; after timeout, emits an `hl7.cancel_without_create` event for triage |
