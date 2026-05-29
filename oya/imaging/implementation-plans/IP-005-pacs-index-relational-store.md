# IP-005 — PACS index relational store

`scope: oya-imaging-adapter-cloud-data + oya-imaging-pacs-usecase`
`wave_target: 17-imaging-pacs`
`adr_binding: ADR-0244 (tenant scoping) + ADR-0105`

## Objective

Stand up the PACS index relational store on `cloud-data` (PostgreSQL-compatible) with tenant-id partition key on every table, cell-local primary + cross-AZ replication.

## Scope

1. Schema migrations: `study`, `series`, `instance`, `mpps`, `mwl_workitem`, `presentation_state`, `structured_report`.
2. Tenant-id partition key per ADR-0244.
3. Compound indexes: (tenant_id, study_instance_uid), (tenant_id, patient_id, study_date), (tenant_id, modality, body_part).
4. QIDO-RS query translation (per-tag filters → SQL).
5. Worklist materialized view.

## Acceptance criteria

- QIDO-RS study-level query p95 < 200ms with 10M-row index per tenant (FR-PACS-002).
- Cross-tenant query leak prevention: integration test asserts SELECT cannot return cross-tenant rows.
- Schema migration runs idempotent.

## Dependencies

- IP-001.
- `cloud-data` µservice.

## Risks

- PostgreSQL partition limits at 1000+ tenants per shard; mitigate with hash partitioning + tenant-shard mapping.
- Tag-filter combinatorial complexity in QIDO-RS; mitigate with prepared-statement cache.

## Estimated effort

- 6–8 person-weeks.
