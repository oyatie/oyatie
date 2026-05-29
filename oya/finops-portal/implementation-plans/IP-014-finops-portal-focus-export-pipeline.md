---
ip_id: IP-014
ip_status: ready
slice_owner: ops-finops
authored: 2026-05-18
slice: finops-portal/focus-export/pipeline
related_adrs: [ADR-0131, ADR-0174, ADR-0186, ADR-0197, ADR-0199]
depends_on: [IP-001, IP-002, IP-004]
target_lines: 150
---

# IP-014 — FOCUS 1.3 export pipeline

## Why this slice

ADR-0199 §D-4 names FOCUS 1.3 as the canonical export schema for
per-tenant cost data. Tenants need to download their cost data in
FOCUS 1.3 so they can plug it into their own finance pipeline (e.g.
their BI / CFO dashboards).

This slice authors the end-to-end FOCUS export pipeline:

1. A query against OpenCost + Mimir for the requested tenant +
   period.
2. A translator from the internal `TenantInvoice` + line shape into
   the FOCUS 1.3 column schema.
3. A streaming serializer (Parquet + CSV) writing to SeaweedFS via
   the s3 adapter.
4. A signed-URL issuer that returns a 1-hour-TTL download URL.

The export is **deterministic** and **byte-stable**: running the
same export twice produces the same bytes (modulo the audit-chain
emit timestamp).

## Acceptance criteria

1. Two new crates:
   - `crates/oya-finops-portal-focus-export-kernel/` — FOCUS 1.3
     schema types + translator (pure).
   - `crates/oya-finops-portal-focus-export-adapter-seaweedfs/` —
     streaming writer + signed-URL issuer.
2. Kernel public types:
   - `FocusRow` — every required FOCUS 1.3 column field
     (`BillingAccountId`, `ServiceName`, `ChargeCategory`,
     `BilledCost`, `EffectiveCost`, `Tags`, etc., per spec).
   - `FocusManifest` — schema_version "1.3", tenant_id, period,
     row_count, byte_count_estimate.
3. Kernel public function:
   ```rust
   pub fn translate(
       invoice: &TenantInvoice,
       lines: &[InvoiceLine],
   ) -> Result<(FocusManifest, Vec<FocusRow>), TranslateError>;
   ```
4. Adapter public function:
   ```rust
   pub async fn write_parquet(
       &self, manifest: &FocusManifest, rows: &[FocusRow],
       bucket: &str, key: &str,
   ) -> Result<WrittenObject, WriteError>;
   pub async fn issue_download_url(
       &self, key: &str, ttl: Duration,
   ) -> Result<SignedUrl, IssueError>;
   ```
5. The translator passes FOCUS 1.3 conformance: every required
   column populated; column types match the spec; missing optional
   columns emit `null` rather than omit (FOCUS spec requirement).
6. Streaming write keeps memory bounded ≤ 64 MB for tenants with
   ≤ 10 million line items.
7. Each export emits a `FocusExportDownloaded` audit-chain event
   when the signed URL is **used** (not when issued).
8. ≥ 8 tests:
   - happy translate round-trip.
   - missing required field returns `TranslateError::MissingColumn`.
   - byte-stable output (running translate twice produces equal
     `Vec<FocusRow>` bytes).
   - signed-URL TTL respected.
   - 10M-row streaming write keeps memory bounded (load test).
   - audit-chain emit fires on first use only (idempotency).
   - Parquet schema matches FOCUS 1.3 spec.
   - CSV alternative output also conforms.

## File-level work plan

1. Kernel `Cargo.toml`, `src/lib.rs`, `src/schema.rs`,
   `src/translate.rs`, `src/manifest.rs`, `src/error.rs`.
2. Adapter `Cargo.toml`, `src/lib.rs`, `src/writer.rs`,
   `src/url.rs`, `src/error.rs`.
3. `contracts/focus-export-internal.openapi.yaml` —
   internal-only API surface (between api and adapter).

## FOCUS 1.3 spec mapping (subset)

| FOCUS column           | Source in `InvoiceLine`               |
|------------------------|---------------------------------------|
| `BillingAccountId`     | `tenant_id` (cast to UUID-string)     |
| `ServiceName`          | `cost_center` mapped via lookup       |
| `ServiceCategory`      | `cost_center.service_category()`      |
| `ChargeCategory`       | `Usage` (default; `Credit` for credit lines) |
| `BilledCost`           | `line.amount_cents / 100.0`           |
| `EffectiveCost`        | `BilledCost - applied_credit_for_this_line` |
| `ChargePeriodStart`    | `period.start`                        |
| `ChargePeriodEnd`      | `period.end`                          |
| `Tags`                 | from `line.tags` map                  |

Mappings that **fail** (e.g. unknown cost-center) emit
`TranslateError::UnknownCostCenter { cost_center }` so the export
fails fast rather than silently mismatch.

## Streaming + bounded memory

The Parquet writer uses the `parquet` crate with row-group size 256
and column dictionary encoding to keep memory low. For a 10M-row
tenant, peak memory should sit < 64 MB; load-tested in
`tests/load_focus_export.rs` (ignored by default; runs in
nightly CI).

## Signed-URL issuance + audit-chain

- `issue_download_url` returns a presigned URL valid for `ttl`
  (default 1 h, max 24 h per regulator-policy).
- On **first download** (detected via SeaweedFS access log),
  the pipeline emits `FocusExportDownloaded` to audit-chain
  with `{ tenant_id, period, object_key, downloaded_at, ip_hash }`.
- Re-downloading the same key within TTL re-emits the event
  (per-download audit).

## Risk + mitigation

- **Risk**: FOCUS 1.3 spec evolves; export drifts. **Mitigation**:
  the `schema_version` is `1.3` (pinned in manifest); a future
  bump is an ADR + a new translator with version-aware dispatch.
- **Risk**: tenant-id leakage cross-tenant via the bucket layout.
  **Mitigation**: bucket key prefix is `t/{tenant_id_hash}/...`
  and the bucket has per-tenant IAM scope via Cedar in IP-007.

## Out-of-scope

- The bucket itself — provisioned by cloud-iac.
- The download UI — finops-portal app crate.

## References

- ADR-0199 — FinOps + FOCUS 1.3 canonical.
- ADR-0197 — backup substrate.
- FOCUS 1.3 spec — https://focus.finops.org/focus-specification/.

## Verification

- `cargo test -p oya-finops-portal-focus-export-kernel`.
- `cargo test -p oya-finops-portal-focus-export-adapter-seaweedfs`.
- `oya gate focus-spec-conformance --version 1.3`.
