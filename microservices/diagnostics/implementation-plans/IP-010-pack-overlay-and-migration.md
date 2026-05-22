# IP-010: Pack Overlay and Migration

Status: Reconciled
Date: 2026-05-21

## Goal

Bind diagnostics compliance packs and remove the former imaging bundle.

## Scope

- HIPAA, CLIA, CAP, ISO 15189, GxP, KR IVD, and EU IVDR overlays for lab/pathology.
- Migration of lab/pathology order/result/case material from healthcare-integration where needed.
- Removal of diagnostics-local imaging, PACS, VNA, and DICOM artifacts.

## Acceptance

- Pack overlays do not include ACR, DICOM conformance, IHE Radiology, mammography, or PACS/VNA evidence under diagnostics.
- Reconciliation notes point to `microservices/imaging/` for imaging authority.
