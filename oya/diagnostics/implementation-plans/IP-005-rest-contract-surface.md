# IP-005: REST Contract Surface

Status: Reconciled
Date: 2026-05-21

## Goal

Expose lab/pathology REST commands and FHIR read projections.

## Scope

- Lab order and lab result endpoints.
- Pathology case and sign-out endpoints.
- Specimen, critical result, reference range, reflex, and delivery endpoints.
- FHIR R5 `Observation` and lab/pathology `DiagnosticReport` reads.

## Acceptance

- OpenAPI contains no PACS, DICOM, radiology, or FHIR `ImagingStudy` paths.
