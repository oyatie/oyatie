# Diagnostics

Diagnostics is the Oyatie lab + pathology microservice.

It owns:

- lab orders and lab results;
- pathology cases and sign-out;
- specimen chain of custody;
- critical-result escalation;
- reference ranges and reflex testing;
- result authorization, interpretation, and delivery;
- turn-around-time and quality-control evidence.

It does not own imaging. Imaging orders, PACS/VNA, DICOM, radiologist workflow, FHIR `ImagingStudy`, and imaging reports are owned by `../imaging/`.

## Canonical Files

- `PRD.md`
- `ARCHITECTURE.md`
- `manifest.json`
- `contracts/openapi.yaml`
- `contracts/asyncapi.yaml`
- `contracts/proto/diagnostics.proto`
- `policies/*.cedar`
- `slos/*.openslo.yaml`

## Reconciliation

Wave 15M-RECONCILE removed the former bundled imaging surface from diagnostics after the dedicated imaging microservice became authoritative. Remaining references to imaging are cross-service handoff references only.
