# ADR-MS-001 — DICOMweb-first substrate

`microservice: imaging`
`status: ACCEPTED`
`date: 2026-05-21`
`wave: 15M-G`
`authority: ADR-0132 + user directive 2026-05-21 (single-concern split)`

## Context

The DICOM PS 3.x standard has two transport surfaces:

1. **DIMSE** — the original 1993 upper-layer protocol over TCP. C-STORE / C-FIND / C-MOVE / C-GET / MWL / MPPS / N-CREATE / N-SET / N-ACTION / N-EVENT-REPORT / N-GET. Used by virtually every imaging modality on the market.
2. **DICOMweb** — added by NEMA in PS 3.18. RESTful surface: WADO-RS / QIDO-RS / STOW-RS / UPS-RS. Used by modern zero-footprint viewers and increasingly by mobile and AI-vendor clients.

Legacy PACS vendors (GE Centricity, Philips IntelliSpace, Sectra, Fujifilm Synapse, Agfa Impax) implement DIMSE as the substrate and bolt DICOMweb on top as a translation layer. Visage 7 demonstrated DICOMweb-first server-side rendering at scale.

Per ADR-0253 (HTTP/3 + QUIC default protocol), Oyatie's protocol default is HTTP/3 + QUIC, which aligns naturally with DICOMweb (HTTP-based) but not DIMSE (TCP-based). Per ADR-0145 (direct gRPC inter-µservice with 3 invariants), the µservice has no Workflow+Ontology adapter to translate between DIMSE and DICOMweb.

## Decision

**The imaging µservice is DICOMweb-first.** DIMSE is supported as a compatibility bridge but is not the canonical substrate.

Implementation:

1. PACS / VNA persist + index records as DICOMweb-native structures (Study/Series/Instance JSON metadata + pixel blob in `cloud-storage`).
2. DICOMweb endpoints (WADO-RS / QIDO-RS / STOW-RS / UPS-RS) are direct first-class HTTP/3 + QUIC surfaces.
3. DIMSE listener pods translate inbound C-STORE / C-FIND / C-MOVE / C-GET / MWL / MPPS / N-* into internal DICOMweb operations.
4. The DICOM Conformance Statement (PS 3.4) is published per release.
5. The 10,250 instances/min per-pod C-STORE throughput claim from `microservices/healthcare-integration/performance-benchmark-numbers-2026-05-20.md` is preserved by tuning the DIMSE-to-DICOMweb bridge to minimal-copy semantics.

## Consequences

### Positive

- Modern HTTP/3 + QUIC alignment with ADR-0253.
- Easier scale-out: HTTP/3 load balancers + edge CDN for thumbnails.
- AI vendor integration is HTTP-native (most AI vendors today consume DICOMweb).
- Patient portal + mobile clients consume DICOMweb directly.
- Visage 7 proof-point for server-side rendering performance.

### Negative

- DIMSE bridge introduces translation cost; mitigated by minimal-copy bridge implementation.
- Vendor-quirk handling lives in the DIMSE-to-DICOMweb bridge rather than at the substrate.
- IHE profiles (XDS-I.b, XCA-I) require both DIMSE and DICOMweb actor roles; double-implementation in some cases.

### Neutral

- DICOM Conformance Statement is unchanged in scope — both DIMSE and DICOMweb conformance must be published.

## Alternatives Considered

- **DIMSE-first** (legacy vendor pattern). Rejected: DIMSE is not HTTP/3-aligned per ADR-0253; harder to scale; harder to integrate with modern AI/mobile/patient-portal clients.
- **Hybrid (peer substrates)** — both DIMSE and DICOMweb as primary. Rejected: doubles persistence cost, makes consistency harder, doesn't fit ADR-0253.
- **DIMSE-only on substrate, DICOMweb as adapter** — modern variant of DIMSE-first. Rejected: same scale and modernization concerns.

## References

- DICOM PS 3.18 (Web Services)
- ADR-0145 (direct gRPC inter-µservice)
- ADR-0253 (HTTP/3 + QUIC default)
- `microservices/healthcare-integration/performance-benchmark-numbers-2026-05-20.md` (10,250 inst/min preserved)
- Visage 7 server-side rendering paper (Bui et al., 2018)
