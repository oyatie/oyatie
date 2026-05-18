---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-drive-foundation
impl_plan_id: IP-003-file-store-adapters
status: pending
execution_unit: ChangeSet
owner: axis-drive
acceptance_lanes: [cargo-build, cargo-nextest, oya-governance-object-store-backend-conformance, oya-check-rls-coverage]
---

# IP-003: file-store -adapter-postgres + -adapter-s3 + -adapter-garage + -adapter-seaweedfs

## Intent

Author the metadata + object-store adapters: `oya-drive-file-store-adapter-postgres` (per-tenant RLS schema), `oya-drive-file-store-adapter-s3` (abstract S3-compat), `oya-drive-file-store-adapter-garage` (primary backend per ADR-DRIVE-0001), `oya-drive-file-store-adapter-seaweedfs` (archive tier). MinIO uses `-adapter-s3` directly.

## Concrete File Targets

| Path | Action |
|---|---|
| `oya-drive-file-store-adapter-postgres/...` | created — Postgres schema migrations + RLS policies |
| `oya-drive-file-store-adapter-postgres/migrations/0001_initial.sql` | created — file, file_version, immutability_record, legal_hold tables with RLS |
| `oya-drive-file-store-adapter-s3/...` | created — S3 SigV4 client; multipart; range; object-lock |
| `oya-drive-file-store-adapter-garage/...` | created — Garage-specific quirks; cell-aware retry |
| `oya-drive-file-store-adapter-seaweedfs/...` | created — archive tier read; promotion + demotion |

## Acceptance Gates

```bash
cargo build -p oya-drive-file-store-adapter-{postgres,s3,garage,seaweedfs}
cargo nextest run -p oya-drive-file-store-adapter-postgres -- rls_per_tenant
cargo nextest run -p oya-drive-file-store-adapter-garage -- s3_tests_corpus
cargo nextest run -p oya-drive-file-store-adapter-seaweedfs -- s3_tests_corpus
cargo run -p oya-dev-cli -- gate validate object-store-backend-conformance --microservice drive
```

## References

- ADR-0105 Amendment 3 (backend-qualified adapters).
- ADR-DRIVE-0001 — object-storage substrate.
- `s3-tests` Ceph conformance corpus.
