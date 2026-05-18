---
doc_class: ContractSpec
title: Backfill + Replay Contract
microservice: sheets
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-sheets + ops-sre-reliability
deciders: axis-sheets, council-architecture, ops-sre-reliability
related_adrs: [ADR-0028, ADR-0110, ADR-0135, ADR-0131]
related_artifacts:
  - microservices/sheets/PRD.md
  - microservices/sheets/capacity-model.md
  - microservices/sheets/threat-model.md §"T-T-01" + §"T-T-02"
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (sheets µservice)

## Purpose

Specify how sheets handles two scenarios:
1. **Backfill** — historical workbook sessions / CRDT op streams / cell-edit logs need reconstruction (e.g., post-incident analysis, audit-chain integrity verification, regulatory disclosure request).
2. **Replay** — an existing workbook version needs re-rendering OR re-saving with different parameters (e.g., bug fix in formula-engine, conditional-formatting rule change, XLSX importer version change, schema migration).

## Backfill

### Contract

When historical workbook state needs reconstruction:

1. **Read source**: Postgres `cell_edit_seals` table (Ed25519-sealed per cell-edit, sampled per PRD threat-model R-NN) + Postgres `workbook_version_pointers` + S3 `workbook_snapshots/<workbook_id>/<version_sha>.bin` + Arrow/Parquet large-sheet blocks + audit-chain seal log (cross-µservice fallback).
2. **Reconstruct cell graph**: from version-history snapshot pointers, hydrate workbook at requested version; replay sampled cell-edit log to current; validate sequence-num monotonic; verify HMAC on every op.
3. **Verify audit-chain integrity**: every reconstructed cell-edit must have a corresponding seal in audit-chain µservice; chain must be reconstructable.
4. **Emit `WorkbookReconstructed` event**: consumed by observability for forensic dashboards; audit-chain seals the reconstruction itself.

### Constraints

- Backfill does NOT mutate the original workbook record; reconstruction emits new events with `kind=reconstructed` tag.
- Retention bound: Postgres `cell_edit_seals` retained 30d hot; cold-tier S3 retains seals 7y per Bominal ADR-0028.
- Cost: backfill is bounded by `O(cell_edit_count × workbook_count)`; workbooks older than 30d require cold-tier fetch (slower; cost-budget bounded per `cost-budget.md`).
- Per-tenant rate-limiting: tenants cannot trigger more than 1 backfill per workbook per hour (anti-abuse).
- 2-person rule + ops-security approval for any backfill that touches cross-tenant data.

### Verification

- Integration test: seed Postgres + Redis + S3 with synthetic workbook; reconstruct via backfill; verify reconstructed cell graph == original cell graph.
- Audit-chain integrity: every backfilled event has seal lineage to original.
- Determinism: re-running backfill emits identical reconstructed state.

## Replay

### Contract

Replay re-emits / re-renders a workbook version with the current formula-engine + recalc-engine + formatting code:

| Trigger | Procedure | Output |
|---|---|---|
| Bug fix in formula-engine (per ADR-SHEETS-0002) | Replay all saved workbooks through new formula-engine; assert formula results unchanged on Excel-reference corpus subset | If non-equal: flag for tenant review; do NOT auto-overwrite |
| Bug fix in recalc-engine | Replay recalc on saved workbooks; assert no new diagnostic errors | Surface errors per-tenant |
| Conditional-formatting rule change | Replay formatting resolution; surface visual diff | Tenant decides accept/reject |
| XLSX importer version bump (calamine major version) | Replay XLSX import on the 100-workbook golden corpus per ADR-SHEETS-0007; assert best-effort tier preserved | If regression: roll back importer |
| Pivot-table aggregator change | Replay pivot evaluation; surface result diff | Tenant decides accept/reject |
| Chart renderer change | Replay chart render; surface visual diff | Tenant decides accept/reject |
| Schema migration (canonical-sheet v1 → v2) | Replay via migration adapter; emit v2 sheet; preserve v1 lineage | Two-version coexistence |

### Procedure

1. Operator invokes: `cargo run -p oya-dev-cli -- sheets replay --workbook <id> --reason "<rfc>"`.
2. CLI requires 2-person rule + ops-security approval (replay touches tenant data).
3. Engine re-runs formula-engine + recalc against current code; compares against stored version_sha.
4. Emits `WorkbookReplayed` event with `prior_version_sha`, `replay_version_sha`, `formula_results_changed`, `differences_summary`, `reason`.
5. Audit-chain seal: replay itself is sealed.

### Constraints

- Replay does NOT mutate the original workbook; new version SHA created with explicit `kind=replayed` label.
- Recalc determinism invariant: replay of a clean workbook MUST produce identical formula-engine outputs. If it doesn't, that's a regression in formula-engine OR recalc-engine — file a bug.
- Replay cannot overwrite production-tier release pointers; new replay version stays in `draft` state pending tenant promotion.

### Verification

- Integration test: replay 100-workbook XLSX golden corpus through current formula-engine; expect best-effort tier preserved per ADR-SHEETS-0007.
- Migration test: replay v1 canonical-sheet through v2 adapter; expect lossless conversion.

## Cost Model

| Operation | Frequency | Estimated cost per call |
|---|---|---|
| Backfill on regulatory request | per-tenant-disclosure | ~$1.50 (1 workbook, 30d history, Postgres + S3 + Arrow/Parquet cold fetch) |
| Replay on formula-engine bug fix | per-formula-engine-release | ~$75 (full replay across all workbooks; 100K active workbooks) |
| Replay on calamine version bump | per-calamine-major-release | ~$10 (corpus replay only) |
| Replay on schema migration (v1→v2) | one-time per migration | ~$1000 (full corpus; bounded by versioning ADR) |

Costs surfaced in `cost-budget.md` — backfill / replay budgeted as part of Sheets's operational envelope.

## Limitations

- Backfill quality is bounded by Postgres + audit-chain retention windows. Workbooks older than 7y are forensically lost (intentional per ADR-0028 retention).
- Replay assumes deterministic formula-engine; non-determinism is a bug (caught by `oya-governance-sheets-recalc-determinism` lane).
- Schema migrations (canonical-sheet v1 → v2) require explicit migration adapters; not auto-replayable without operator sign-off.
- Cross-tenant backfill (for breach forensics) requires legal + privacy approval per pack regulation (GDPR Art. 15 DSR; KR PIPA Art. 35; HIPAA §164.524).
- AI-formula prompts at LLM provider may persist beyond retention (mitigated by zero-retention LLM model selection).
- Connected-sheets external-source results are NOT replayable from sheets state alone (external source must also be available).

## References

- `microservices/sheets/PRD.md`.
- `microservices/sheets/capacity-model.md`.
- `microservices/sheets/cost-budget.md`.
- `microservices/sheets/contracts/asyncapi/sheets-events.yaml`.
- ADR-0028 Audit-chain.
- ADR-0110 ChangeSet state machine.
- ADR-0135 Sheets net-new µservice.
- ADR-SHEETS-0001 Loro CRDT.
- ADR-SHEETS-0002 formula-engine conformance.
- ADR-SHEETS-0004 recalc-engine architecture.
- ADR-SHEETS-0007 XLSX export fidelity.
- Loro CRDT replay semantics — `loro.dev/docs`.
- Google SRE Workbook ch. 9.
