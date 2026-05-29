# IP-014 — Enterprise imaging beyond radiology

`scope: oya-imaging-acquisition-{echo,cath,oct,derm,path-wsi,dental,surgical}-worker + per-discipline SOP class support`
`wave_target: 20-imaging-enterprise`
`adr_binding: ADR-MS-004`

## Objective

Extend the imaging substrate beyond radiology to cardiology (echo, cath, EP), ophthalmology (OCT, fundus, visual field), dermatology, pathology WSI, dental, surgical video.

## Scope

1. Echo acquisition worker + DICOM SR-TID-5200 adult-echo + adult-echo-pediatric.
2. Cath / EP worker + DICOM coronary angio + ECG waveform SR.
3. OCT worker + DICOM Ophthalmic Tomography IOD.
4. Fundus + Visual Field worker + DICOM Ophthalmic Photography + Ophthalmic Visual Field IODs.
5. Dermatology worker + DICOM Visible Light Photography IOD.
6. Pathology WSI worker + tiled JPEG2000 pyramidal multi-resolution storage.
7. Dental worker + DICOM Dental Pano IOD.
8. Surgical video worker + DICOM VL Endoscopic IOD.
9. Per-discipline hanging protocols + structured reports.

## Acceptance criteria

- Per-discipline acquisition state-machine tests pass.
- Pathology WSI 100GB-slide round-trip storage + retrieve test.
- Cross-discipline image correlation via FHIR ImagingSelection.

## Dependencies

- IP-001, IP-005, IP-006, IP-008, IP-009.

## Risks

- Pathology WSI economics (storage scale).
- Per-discipline workflow complexity.

## Estimated effort

- 18–24 person-weeks (6 disciplines × ~3 person-weeks each).
