# IP-001: Tenant Scope Kernel

Status: Reconciled
Date: 2026-05-21

## Goal

Implement tenant, cell, patient, specimen, order, case, result, report, and audit identifiers for lab/pathology diagnostics.

## Scope

- Tenant and cell partitioning for lab orders, lab results, pathology cases, specimens, and QC records.
- Correlation and causation IDs for event delivery.
- No image object, PACS, VNA, or DICOM identifier ownership.

## Acceptance

- Every lab/pathology aggregate carries `tenant_id`, `cell_id`, and immutable audit identifiers.
- Cross-service references to imaging are stored as opaque external references only.
