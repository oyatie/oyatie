# IP-006 — Per-modality acquisition workers

`scope: oya-imaging-acquisition-<modality>-worker crates`
`wave_target: 17-imaging-pacs`
`adr_binding: ADR-MS-004 (enterprise imaging scope)`

## Objective

Stand up the per-modality acquisition state-machine workers covering CT / MRI / X-ray / Ultrasound / Mammography / Nuclear / PET / Fluoroscopy / Angiography / IR / DEXA / OCT / Echo / Cath / Dermatology / Pathology WSI / Dental / Surgical.

## Scope

1. Per-modality worker crate implementing the universal state machine:
   `SCHEDULED → ARRIVED → MWL_PULLED → ACQUIRING → ACQUIRE_COMPLETE → C_STORED → TECH_QC → FORWARDED_TO_WORKLIST`.
2. Modality-specific deltas:
   - CT: RDSR emission on ACQUIRE_COMPLETE.
   - MRI: per-sequence metadata.
   - Mammography: MQSA breast positioning metadata.
   - Fluoroscopy / IR: cumulative fluoroscopy time + air-kerma area product.
   - Ultrasound: live preview frames.
   - NM / PET: SUV calibration.
   - Pathology WSI: tiled JPEG2000 multi-resolution.
3. MWL / MPPS integration (FR-ACQ-002 + FR-ACQ-003).
4. Vendor-quirk profile selection (IP-002).

## Acceptance criteria

- Each modality state machine has unit tests covering happy path + 5 failure transitions (modality abort, network drop mid-MPPS, MWL stale, MPPS not closing, C-STORE retry).
- Mammography acquisition emits MQSA breast positioning metadata.
- CT acquisition emits RDSR by default per NEMA XR-29.
- Fluoroscopy + IR emit cumulative fluoroscopy time + air-kerma area product (FR-ACQ-006).

## Dependencies

- IP-001, IP-002, IP-005.

## Risks

- Vendor-modality quirks (e.g., GE CT private-tag absence on certain protocols); mitigate with per-vendor profile lookup.
- Pathology WSI very-large file handling.

## Estimated effort

- 16–24 person-weeks (18 modalities × ~1 person-week each).

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/imaging/implementation-plans/IP-006-modality-acquisition-workers.md:16` - - CT: RDSR emission on ACQUIRE_COMPLETE..
