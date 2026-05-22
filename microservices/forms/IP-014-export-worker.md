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

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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
- PRD FR-13, FR-14 and AC-16 / AC-17.
- `microservices/forms/contracts/openapi/forms.openapi.yaml`.
- `microservices/forms/slos/export-csv-latency.openslo.yaml`.
- `microservices/forms/runbooks/export-pipeline-failure.md`.
- `microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md`.

## Foundation A-G Substance

- A. Product scope: export is the governed egress path for response data, analytics handoff, and sheets bridge.
- B. Domain model: `ExportRequest`, `ColumnProjection`, `RedactionPolicy`, `SignedExportManifest`, and `ExportCheckpoint` are explicit.
- C. Contracts: REST starts and observes export jobs; sheets bridge receives versioned, ordered, redacted columns.
- D. Policy: unredacted PII requires Cedar entitlement, audit-chain seal, purpose, and pack-resident destination.
- E. Operations: streaming checkpoints allow retry without duplicate rows; failed exports emit signed manifest and rollback object writes.
- F. Observability: emit export p95, row throughput, redaction overrides, sheets bridge failures, and manifest verification failures.
- G. Promotion: 100k CSV/XLSX benchmarks, PII redaction adversarial tests, sheets bridge round-trip, and runbook drill gate done.

## Counterpart Benchmark

- Counterpart: Google Forms to Sheets export, HubSpot Forms CSV export, Notion Forms/Databases export, and Salesforce Web-to-Lead report export.
- Defensible parity claim: Oyatie must stream large exports within SLO while keeping PII redacted by default.
- Differentiator: every unredacted export is entitlement-checked and audit-sealed.
- Grep counterpart names: HubSpot Forms; Notion Forms/Databases; Salesforce Web-to-Lead.

## Remediation Notes

- Expanded export with PRD, OpenAPI, SLO, benchmark, and runbook grounding.
- Added A-G substance for egress domain, contracts, policy, operations, observability, and promotion.
- Added counterpart names for grep-recognized parity review.

## Verification Evidence Required

- 100k CSV and XLSX benchmarks prove streaming latency budgets without full materialization.
- Redaction adversarial corpus proves PII remains hidden unless entitlement and audit seal are present.
- Sheets bridge round-trip proves column order, version binding, and redaction survive handoff.
- Export failure runbook drill proves checkpoint retry and object rollback behavior.
- Signed manifest verification proves export evidence can be audited later.

## Next IP

[`IP-015-hg-forms-registration.md`](IP-015-hg-forms-registration.md)
