---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-translate-platform
impl_plan_id: IP-010-bulk-translate-stack
status: pending
execution_unit: ChangeSet
owner: axis-translate
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: Bulk Translate stack (`oya-translate-bulk-*`)

## Intent

Asynchronous bulk translation jobs over XLIFF 2.1 / TMX 1.4 / TBX / large multi-file inputs. Job state in Postgres; payload in S3; concurrency cap per tenant. Per PRD §"Performance": 10 k-segment XLIFF p95 ≤ 60 s.

## ChangeSet boundary

Crates: `oya-translate-bulk-{kernel, domain, usecase, api, adapter-postgres, adapter-s3, adapter-redis, rest, worker, sdk, app}`.

## Job Lifecycle

```text
Submitted → Queued → InProgress → (Completed | Failed | Cancelled)
                          ↓
                    per-chunk fan-out (≤ 16 concurrent per job; ≤ 64 concurrent per tenant)
                          ↓
                    each chunk = (segment-batch ≤ 100) routed via translate-router-usecase
                          ↓
                    chunk result → S3 (per-chunk file) + Postgres job state
                          ↓
                    on all chunks completed → merge → S3 final output + Postgres state=Completed
                          ↓
                    BulkJobCompleted event → audit-chain
```

## Postgres Schema (Excerpt)

```sql
CREATE TABLE bulk_jobs (
  id UUID PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  project_id TEXT,
  state TEXT NOT NULL,                       -- 'Submitted'|'Queued'|'InProgress'|'Completed'|'Failed'|'Cancelled'
  input_format TEXT NOT NULL,                -- 'xliff-2.1'|'tmx-1.4'|'tbx'|'mixed'
  input_s3_key TEXT NOT NULL,
  output_s3_key TEXT,
  source_lang TEXT,
  target_lang TEXT,
  total_segments INTEGER,
  completed_segments INTEGER,
  failed_segments INTEGER,
  started_at TIMESTAMPTZ,
  completed_at TIMESTAMPTZ,
  created_by TEXT NOT NULL,
  evidence_ref TEXT
);
CREATE INDEX idx_bulk_jobs_tenant_state ON bulk_jobs (tenant_id, state);
ALTER TABLE bulk_jobs ENABLE ROW LEVEL SECURITY;
CREATE POLICY bulk_jobs_tenant ON bulk_jobs USING (tenant_id = current_setting('oya.tenant_id'));
```

## XLIFF 2.1 / TMX 1.4 / TBX I/O

- Parse with `quick-xml` + schema validation (XLIFF 2.1 OASIS schema; TMX 1.4 LISA OSCAR DTD; TBX 3.0 ISO 30042).
- Entity / DTD / XSL resolution disabled (F-01..F-03 in threat-model).
- Streaming parse for ≥ 100 MiB inputs.
- Per-segment extraction respects `<unit>` + `<segment>` + `<source>` + `<target>` semantics; `<state>` updated to `translated` on output.

## REST + WS API

```
POST   /bulk/jobs                            — submit (multipart; signed-URL upload alternative)
GET    /bulk/jobs/{job_id}                    — state + progress
DELETE /bulk/jobs/{job_id}                    — cancel
GET    /bulk/jobs/{job_id}/output             — signed-URL to S3
WS     /bulk/jobs/{job_id}/events             — progress stream
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_xliff_2_1_lossless_round_trip` | spec |
| `test_tmx_1_4_lossless_tu_round_trip` | spec |
| `test_tbx_import_round_trip` | spec |
| `test_concurrency_cap_16_per_job` | invariant |
| `test_concurrency_cap_64_per_tenant` | invariant |
| `test_job_cancel_drains_in_flight_chunks` | clean shutdown |
| `test_per_chunk_failure_retries` | retry policy |
| `test_signed_url_for_output_expires` | security |
| `tests/load/bulk_translate_10k_xliff_p95_under_60s.rs` | AC-10 |
| `tests/integration/cross_tenant_bulk_read_denied.rs` | RLS |

## Halt Conditions

- TMX/TBX parse accepts external entity.
- Concurrency cap violated.
- Cross-tenant signed-URL access succeeds.

## Next IP

[`IP-011-real-time-stream-stack.md`](IP-011-real-time-stream-stack.md)
