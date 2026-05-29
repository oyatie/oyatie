---
ip_id: IP-024
microservice: compliance
bounded_context: dpia-orchestration
layer: adapter
status: planned
related_adrs: [ADR-0244, ADR-0276, ADR-0251]
---

# IP-024 — DPIA orchestration Postgres adapter

## A. Problem

DPIA records are long-lived regulatory evidence and must survive restarts, audits, and pack-activation decisions. The usecase in IP-018 needs tenant-scoped persistence, but storing DPIAs as untyped blobs would break RLS, backup portability, and control mapping. The prior IP named tables but not the tenant, status, and migration discipline.

## B. Approach

Implement a Postgres adapter for DPIA records with row-level security and Citus distribution by `tenant_id`. The adapter owns schema, migrations, query ports, and backup-portability serialization. It stores data flows, risk findings, mitigations, DPO signatures, and audit seal references as normalized tables.

## C. Deliverables

| Artifact | Change |
|---|---|
| `microservices/compliance/iac/terraform-module.tf` | database provisioning hook if this adapter needs dedicated DB resources |
| `microservices/compliance/contracts/openapi.yaml` | status/read schemas consumed by future REST |
| `microservices/compliance/slos/evidence-emission-lag.openslo.yaml` | finalization-to-evidence freshness |
| `microservices/compliance/runbooks/dsar-backlog-overflow.md` | distinguish DPIA backlog from DSAR backlog |

## D. Implementation

1. Create migrations for `dpia_records`, `dpia_data_flows`, `dpia_risks`, `dpia_mitigations`, `dpia_signatures`, and `dpia_audit_refs`.
2. Every table carries `tenant_id NOT NULL`, `home_cell`, `pack_id`, and `created_at`.
3. Enable RLS with policies equivalent to `principal.tenant_id == row.tenant_id`.
4. Use status enum: `open`, `inventory_collected`, `risk_assessed`, `mitigation_pending`, `dpo_review`, `finalized`, `blocked`.
5. Store DPO signature subject, timestamp, key id, and seal reference separately from free-text rationale.
6. Add backup-portability export shape for ADR-0276 with checksums per table.
7. Add tests for RLS rejection, same-tenant read, stale migration checksum, finalized read, and backup export/import.

## E. Acceptance

- Cross-tenant DPIA reads return no rows and are tested at adapter level.
- Migrations are reversible or explicitly carry a rollback playbook.
- Finalized DPIA records expose audit seal references for evidence export.
- Backup-portability export can restore a tenant's DPIA set into a clean database.

## F. Evidence

- `microservices/compliance/PRD.md` makes cross-tenant isolation a Sev-1 invariant.
- `microservices/compliance/manifest.json` names `tenant-cell and regional-failover execution domain`.
- `microservices/compliance/competitor-parity-matrix.md` lists OneTrust/ServiceNow GRC as privacy workflow counterparts.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| OneTrust | Provides persistent DPIA workflow records while keeping data in Oyatie's Postgres/RLS boundary. |
| ServiceNow GRC | Narrows workflow-record persistence parity without centralizing all risk data outside the tenant cell. |
| Vanta / Drata | Exceeds checklist-style privacy evidence by preserving typed risk and mitigation state. |

## H. Non-goals and handoff boundaries

- Do not put DPIA orchestration rules in SQL triggers; IP-018 usecase owns workflow logic.
- Do not store tenant-mixed DPIA tables without RLS.
- Do not store DPO signatures as unstructured text only; key id and seal refs are required.
- Do not make backup export the only recovery mechanism; normal migrations still need checksums.
- Do not persist raw subject data when data-flow refs and classified summaries are enough.

## I. Fixture set

- `tenant_a_dpia_read_success.sqltest` proves same-tenant read.
- `tenant_b_dpia_read_empty.sqltest` proves RLS isolation.
- `finalized_dpia_has_audit_refs.sqltest` proves evidence linkage.
- `backup_export_roundtrip.sqltest` proves portability.
- `rollback_checksum_mismatch.sqltest` proves migration safety.

## J. Launch blockers

- Any DPIA table lacks `tenant_id NOT NULL`.
- RLS is disabled in migrations or test setup.
- DPO signature records lack key id or seal refs.
- Backup export cannot restore into a clean database.
- Migration rollback plan is absent for schema changes.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/compliance/IP-024-dpia-orchestration-adapter-postgres.md` matched `openapi`; contract files `microservices/compliance/contracts/openapi.yaml, microservices/compliance/contracts/asyncapi.yaml, microservices/compliance/contracts/compliance.proto`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/compliance/IP-024-dpia-orchestration-adapter-postgres.md` matched `emission`; anchors `microservices/compliance/manifest.json, crates/oya-shared-compliance-evidence-kernel/src/lib.rs`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.
