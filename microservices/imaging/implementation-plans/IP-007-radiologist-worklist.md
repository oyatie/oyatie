# IP-007 — Radiologist worklist + dynamic prioritization

`scope: oya-imaging-worklist-app + oya-imaging-worklist-api + oya-imaging-worklist-rest`
`wave_target: 18-imaging-rad-workflow`
`adr_binding: ADR-0105 + ADR-0244`

## Objective

Provide a dynamic prioritized worklist for radiologists with sort/filter/route capabilities matching Sectra reading-room ergonomic standards.

## Scope

1. Worklist materialized view query.
2. Sort by priority (STAT > preliminary > routine > addendum) with stable secondary sort by acquisition time.
3. Filter by modality, body-part, sub-specialty, on-call group.
4. Sub-specialty routing rules per radiologist credential.
5. Worklist load p95 < 800ms for ≤5000 items (FR-RAD-003).

## Acceptance criteria

- Worklist load p95 < 800ms with 5000-item fixture.
- Sub-specialty routing test: mammography reads only surface to MQSA-certified radiologists.
- On-call rotation respected.

## Dependencies

- IP-001, IP-005.

## Risks

- Materialized-view refresh lag; mitigate with incremental refresh + per-tenant fast-path.

## Estimated effort

- 6–8 person-weeks.
