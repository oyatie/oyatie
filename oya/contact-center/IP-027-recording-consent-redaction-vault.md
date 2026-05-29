---
doc_class: ImplementationPlan
ip_id: IP-027-recording-consent-redaction-vault
microservice: contact-center
related_adrs: [ADR-0243, ADR-0263, ADR-0272, ADR-0321]
journey_id: J-CC-27-recording-consent-and-redaction
status: proposed
date: 2026-05-20
owner: axis-contact-center
availability: paid
---

# IP-027: Recording Consent Redaction Vault

## Context

This net-new slice covers recording consent, transcript redaction, and export proof. It displaces Genesys recording policies, NICE CXone recording controls, Five9 call recordings, Talkdesk recordings, and AWS Contact Lens recording outputs while preserving consent and pack rules.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `contact_recording_vault` | `recording_id` | `uuid primary key` | Recording metadata row; media stored out of band. |
| `contact_recording_vault` | `tenant_id` | `uuid not null` | Tenant partition. |
| `contact_recording_vault` | `interaction_id` | `uuid not null` | Call/chat interaction. |
| `contact_recording_vault` | `consent_event_id` | `text not null` | Consent proof. |
| `contact_recording_vault` | `redaction_profile` | `text not null` | PCI, HIPAA, PII, labor profile. |
| `contact_recording_vault` | `storage_ref` | `text not null` | Object-store ref. |
| `contact_recording_vault` | `retention_expires_at` | `timestamptz not null` | Pack-specific retention. |

## API Endpoints

REST `POST /v1/contact-center/recordings/{recording_id}:redact`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-cc000001",
  "redaction_profile": "pci-and-pii",
  "export_reason": "quality_review",
  "ticket_id": "GRC-8821"
}
```

gRPC `RecordingVaultService.Redact(RedactRecordingRequest)` returns `redacted_recording_ref`, `redaction_event_id`, and `audit_event_id`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `Service::"media-recorder"` | `contactCenter::PersistRecording` | `ContactRecording::*` | `tenant_id`, `interaction_id`, `consent_event_id` |
| `User::"quality.manager"` | `contactCenter::RedactRecording` | `ContactRecording::*` | `redaction_profile`, `ticket_id`, `export_reason` |
| `User::"auditor"` | `contactCenter::ExportRecording` | `ContactRecording::*` | `pack_id`, `redacted_only=true` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Genesys Recording | `ContactRecording` | recording id maps to `source_ref.id`; consent imported separately. |
| NICE CXone Recording | `ContactRecording` | contact id maps to interaction id. |
| Five9 Call Recording | `ContactRecording` | call id and agent id become refs. |
| Talkdesk Recording | `ContactRecording` | recording url becomes storage import source. |
| AWS Recording | `ContactRecording` | S3 key maps to storage ref after copy. |

## Workflow Steps

1. `VerifyConsent` requires consent event before vault write.
2. `PersistMetadata` writes recording row.
3. `ApplyRedactionProfile` produces redacted object.
4. `AuthorizeExport` checks Cedar for export.
5. `SealRecordingEvidence` emits audit events.

Branches: missing consent blocks storage; PCI profile required when payment signal present; export denied if ticket missing.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-CONTACT-CENTER-RECORDING-STORED` | `tenant_id`, `interaction_id`, `recording_id`, `consent_event_id` |
| `EVT-CONTACT-CENTER-RECORDING-REDACTED` | `recording_id`, `redaction_profile`, `ticket_id` |
| `EVT-DATA-EGRESS` | Emitted for every recording export. |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Store metadata | 35 ms | 140 ms | 300 ms | 2k writes/s/cell | 99.95% |
| Redact 30-min recording | 8 s | 90 s | 180 s | 200 jobs/hour/cell | 99.9% |

## Failure Modes + Recovery

- Consent event missing: quarantine media and refuse playback/export.
- Redaction worker failure: retain original sealed object, mark redaction pending, retry idempotently.
- Retention pack mismatch: apply stricter retention and open compliance task.

## Migration Notes

Vendor recordings frequently arrive with weak consent evidence. Migration must import audio separately from authorization and must not expose playback until consent and redaction metadata are complete.

## Cross-µservice Handoffs

- `consent` stores recording consent proof.
- `storage-object` stores sealed and redacted media.
- `audit-chain` seals store, redact, and export events.
- `privacy` consumes recording metadata for DSR.
- `compliance` evaluates retention overlays.
