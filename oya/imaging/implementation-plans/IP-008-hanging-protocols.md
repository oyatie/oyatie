# IP-008 — Hanging protocols (DICOM PS 3.18 HP IOD + Oyatie extension)

`scope: oya-imaging-hanging-protocol-app + oya-imaging-hanging-protocol-domain`
`wave_target: 18-imaging-rad-workflow`
`adr_binding: ADR-0105`

## Objective

Stand up the hanging-protocol matching + apply engine. Per-modality + per-body-part + per-radiologist preference. Apply p95 < 150ms.

## Scope

1. DICOM PS 3.18 Hanging Protocol IOD parser.
2. Oyatie-extension YAML schema for richer features (per-radiologist viewport-count preference, AI overlay default visibility).
3. Match algorithm:
   1. Exact radiologist + modality + body-part match.
   2. Radiologist + modality wildcard body-part.
   3. Tenant default per (modality, body-part).
   4. Built-in fallback per modality.
4. Apply: persist DICOM Presentation State + push viewer layout config.

## Acceptance criteria

- Apply p95 < 150ms (FR-RAD-004).
- Match correctness test: per-radiologist override always wins over tenant default.
- 100 ACR-published hanging protocols load + apply.

## Dependencies

- IP-007.

## Risks

- DICOM HP IOD complexity; mitigate with progressive rollout (radiology first, enterprise imaging later).

## Estimated effort

- 6–8 person-weeks.
