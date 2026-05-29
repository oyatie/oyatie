# IP-003: Ontology Projection

Status: Reconciled
Date: 2026-05-21

## Goal

Project lab and pathology facts into ontology without moving domain ownership out of diagnostics.

## Scope

- LOINC, SNOMED-CT, UCUM, FHIR `Observation`, FHIR `DiagnosticReport`, FHIR `Specimen`, and FHIR `ServiceRequest`.
- Pathology case, diagnosis, amendment, addendum, and sign-out terms.
- Opaque external references for imaging correlation only.

## Acceptance

- Ontology receives lab/pathology facts and terminology bindings.
- Diagnostics does not project FHIR `ImagingStudy`.
