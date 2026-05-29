# ADR-MS-004 — Enterprise imaging scope (beyond radiology)

`microservice: imaging`
`status: ACCEPTED`
`date: 2026-05-21`
`wave: 15M-G`
`authority: ADR-0132 + user directive 2026-05-21`

## Context

Historically, PACS systems were radiology-only. "Enterprise imaging" extends scope to non-radiology images:

- **Cardiology** — echo, cath, EP (12-lead ECG via DICOM ECG SOP).
- **Ophthalmology** — OCT, fundus, visual field.
- **Dermatology** — clinical photography.
- **Pathology** — whole-slide imaging (DICOM Pathology / VL Whole Slide Microscopy IOD; slide sizes 10–100+ GB).
- **Dental** — panoramic + CBCT.
- **Surgical** — intraoperative video (Visible Light + Endoscopic).

Vendors take different positions:

- **Agfa Enterprise Imaging** — early enterprise-imaging unifier; positioned as a Europe-strong leader.
- **Philips Enterprise Imaging** — added on top of IntelliSpace PACS via Carestream Vue acquisition.
- **GE Centricity Universal Viewer** — has cardiology / ophthalmology modules but separately licensed.
- **Sectra** — has cardiology and pathology modules; tightly integrated.
- **Visage 7** — radiology-focused; less enterprise-imaging.

Pathology WSI is the outlier — slide sizes routinely 10–100+ GB; pyramidal tile architecture; bespoke workflow (case-list rather than worklist; biopsy block / slide hierarchy rather than DICOM study/series/instance).

## Decision

**The imaging µservice's enterprise-imaging scope includes cardiology, ophthalmology, dermatology, pathology WSI, dental, and surgical video from day one.** Pathology WSI is in scope with the explicit caveat that it may split later into its own `pathology` µservice if slide-size economics + bespoke workflow grow beyond imaging's substrate.

Implementation:

1. Each non-radiology modality has its own per-modality acquisition worker per ARCHITECTURE.md §1.8.
2. DICOM SOP classes are extended beyond radiology: Ophthalmic Tomography IOD, Ophthalmic Photography IOD, Visible Light Photography IOD, Dental Pano IOD, VL Endoscopic IOD, VL Whole Slide Microscopy IOD.
3. Cardiology echo gets dedicated DICOM SR-TID-5200 adult-echo + adult-echo-pediatric reports.
4. Cath / EP gets DICOM coronary angio + ECG waveform SR.
5. Pathology WSI uses tiled JPEG2000 multi-resolution pyramidal storage for very-large slides.
6. Per-specialty hanging protocols (echo lab layouts, mammography hanging, OCT bilateral compare) extend the radiology hanging-protocol engine.

## Consequences

### Positive

- Single enterprise-imaging substrate vs. GE / Philips / Sectra fragmentation across separately licensed modules.
- Cross-discipline image correlation (e.g., dermatology photo → radiology breast study) supported via FHIR ImagingSelection.
- New disciplines (intraoperative video, surgical imaging) can be added via adapter crates.

### Negative

- Pathology WSI imposes very-large-blob handling; tiered storage cost.
- Per-discipline workflow nuances increase product surface area.
- Some disciplines (path, cardiology) may eventually justify their own µservice; split path is open.

### Neutral

- DICOM SOP class coverage is broader than radiology-only PACS; conformance statements expand.

## Alternatives Considered

- **Radiology-only scope** (legacy PACS pattern). Rejected: misses enterprise imaging market.
- **Radiology + cardiology only** (Agfa / Philips early enterprise-imaging pattern). Rejected: still misses dermatology / pathology / dental.
- **Separate µservices per discipline**. Considered for Wave 16+; deferred — single µservice keeps cross-discipline correlation simple. Split is open per ADR-0132 if substrate concerns diverge.

## Open Questions / Future Splits

- **Pathology WSI** may split into its own µservice when slide-size + workflow economics justify.
- **Cardiology imaging** may split if structural-heart / TAVR planning workloads grow.
- **POCUS (point-of-care ultrasound)** may split into an edge µservice if tablet/phone acquisition becomes dominant.

## References

- DICOM PS 3.3 IODs:
  - Ophthalmic Tomography Image (C.8.17)
  - Ophthalmic Photography 8-Bit Image (C.8.13)
  - Visible Light Photographic Image (C.8.12)
  - VL Endoscopic Image (C.8.12)
  - VL Whole Slide Microscopy Image (C.8.12)
  - Dental Pano Image
- ASE adult-echo SR
- Sectra Enterprise Imaging white paper
- Agfa Enterprise Imaging white paper
- ADR-0132 (no-grouping policy; split-when-warranted)
