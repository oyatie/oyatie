# IP-002 — DIMSE bridge (C-STORE / C-FIND / C-MOVE / C-GET / MWL / MPPS / N-*)

`scope: oya-imaging-dimse-api + oya-imaging-modality-vendor-quirks`
`wave_target: 16-imaging-substrate`
`adr_binding: ADR-MS-001 (DICOMweb-first, DIMSE-bridged) + ADR-0105`

## Objective

Provide DIMSE compatibility bridge to the DICOMweb-first substrate. Preserve the 10,250 inst/min throughput claim. Handle all major vendor private-tag patterns.

## Scope

1. `oya-imaging-dimse-api` — DIMSE upper-layer protocol over TCP listener.
2. Association negotiation (presentation context, transfer syntax, SOP class).
3. C-STORE-SCP / C-STORE-SCU implementations.
4. C-FIND-SCP for patient/study/series/instance levels + MWL.
5. C-MOVE-SCP with destination AE-Title routing.
6. C-GET-SCP.
7. MPPS N-CREATE / N-SET / N-ACTION / N-EVENT-REPORT / N-GET.
8. Structured Report (DICOM SR-TID-1500 + ACR templates) C-STORE.
9. `oya-imaging-modality-vendor-quirks` — vendor private-tag library for GE / Siemens / Philips / Canon / Hologic / Hitachi / Mindray.
10. DICOM Conformance Statement generator (PS 3.4).

## Acceptance criteria

- C-STORE throughput sustains 10,250 inst/min/pod under sustained load (preserves healthcare-integration claim).
- DICOM PS 3.4 Conformance Statement passes IHE TF conformance test suite for IRWF.b + SWF.b.
- Per-vendor regression suite covers ≥10 known-quirk patterns per vendor.
- MWL response p95 < 500ms (FR-ACQ-002).
- C-FIND patient-level query p95 < 200ms with 10M-row index.

## Dependencies

- IP-001 (DICOMweb substrate kernel).
- Cedar policy `technologist-can-acquire.cedar` (Cedar gating on AE-title pairing).

## Risks

- TLS-over-DIMSE per IHE ATNA; cert lifecycle alignment with `cloud-secrets` rotation.
- Vendor private tag fingerprinting in zero-copy parser.
- Performance regressions vs. healthcare-integration baseline; mitigate with continuous benchmark lane.

## Out-of-scope (deferred)

- AI marketplace (IP-013).
- Hanging protocols (IP-010).

## Testing strategy

- DICOM PS 3.4 conformance suite.
- IHE Connectathon participation (NA + Europe).
- Vendor-quirk regression corpus.
- Performance benchmark lane.

## Estimated effort

- 10–14 person-weeks for Wave 16-imaging-substrate.
