# ADR-MS-001: Diagnostics Owns Lab + Pathology Only

Status: Amended by Wave 15M-RECONCILE
Date: 2026-05-21

## Context

Diagnostics was initially drafted as a bundled lab + imaging + pathology microservice. The dedicated imaging microservice now exists under `microservices/imaging/` and is authoritative for imaging under ADR-0132 single-concern discipline.

## Decision

Diagnostics owns lab and pathology diagnostic evidence only. Imaging order, imaging result, DICOM study, PACS, VNA, DICOMweb, DIMSE, radiologist workflow, imaging report, and FHIR `ImagingStudy` concerns are removed from diagnostics and are owned by `microservices/imaging/`.

Diagnostics may keep references to imaging only for cross-service handoffs, such as requesting image correlation for a lab/pathology result or storing an imaging report reference returned by imaging.

## Consequences

- `ADR-MS-002-dicom-substrate.md` is removed from diagnostics.
- `IP-007-dicom-substrate-binding.md` is removed from diagnostics.
- Cedar policies, OpenSLOs, OpenAPI, AsyncAPI, proto, manifest, and counterpart matrix must not contain diagnostics-owned imaging contexts.
- Pathology owns case, specimen, narrative, sign-out, addendum, and amendment workflows; imaging owns any image artifact custody needed by pathology workflows.

## Supersession

The canonical imaging authority is:

- `microservices/imaging/PRD.md`
- `microservices/imaging/REMEDIATION-NOTES-2026-05-21.md`
