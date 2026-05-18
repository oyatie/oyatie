---
doc_class: ImplementationPlan
milestone: M03-workspace-tier-foundation
phase: P01-forms-foundation
impl_plan_id: IP-014-export-worker
status: pending
execution_unit: ChangeSet
owner: axis-forms + council-privacy
acceptance_lanes: [cargo-test, oya-forms-export-latency, oya-forms-export-pii-redaction]
---

# IP-014: Export worker (CSV / XLSX / JSON / Sheets-bridge; streaming)

## Intent

Streaming export of responses to CSV / XLSX / JSON / sheets-bridge. PII columns redacted by default; unredacted requires Cedar entitlement + audit-chain seal of export.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/forms/src/worker/export/worker.rs` | create |
| `microservices/forms/src/worker/export/csv.rs` | create — streaming CSV |
| `microservices/forms/src/worker/export/xlsx.rs` | create — streaming XLSX |
| `microservices/forms/src/worker/export/json.rs` | create |
| `microservices/forms/src/worker/export/sheets_bridge.rs` | create — sheets µservice contract |
| `microservices/forms/src/worker/export/pii_redaction.rs` | create |
| `microservices/forms/src/worker/export/signed_manifest.rs` | create — Ed25519 seal |
| `microservices/forms/tests/export_csv_100k.rs` | create |
| `microservices/forms/tests/export_pii_redaction.rs` | create |

## Acceptance Gates

- CSV 100k responses ≤ 5s p95.
- XLSX 100k responses ≤ 10s p95.
- PII redacted unless entitled + audit-sealed.
- Sheets-bridge round-trip verified.

## References

- ADR-FORMS-0003.
- Sheets µservice contracts.

## Next IP

[`IP-015-hg-forms-registration.md`](IP-015-hg-forms-registration.md)
