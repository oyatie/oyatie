---
doc_class: MigrationPlaybook
microservice: sheets
vendor: Google Sheets + Airtable + Coda (parallel migration)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Google Sheets / Airtable / Coda → oyatie sheets

Audience: an oyatie tenant migrating workbooks + bases + docs-with-grids from Google Sheets, Airtable, or Coda to oyatie's `sheets` µservice.

## Why this migration is non-trivial

Each source has different shape conventions:

- **Google Sheets** is grid-of-cells (close to Excel/oyatie shape). Formulas, named ranges, charts, pivot tables all map. Apps Script does NOT port.
- **Airtable** is database-of-rows (closer to Notion-database than Excel). Field types map; "Linked record" fields map to oyatie's cross-sheet reference; "Formula" fields map; "Automations" do NOT port directly (re-build in workflow-engine).
- **Coda** is doc-with-grids (closer to Notion than Excel). Tables port; "Formulas" port mostly; "Buttons" + "Automations" require workflow-engine.

The 80/20: most workbooks port cleanly via the auto-converter; the 20 % needing manual review is on integrations (Apps Script, Airtable Automations, Coda Buttons).

## Step 1 — Export source workbooks (≤ 1-3 days per 100 workbooks)

Google Sheets:

```sh
oya sheets migrate inventory \
    --source google-sheets \
    --google-workspace-id "$WORKSPACE_ID" \
    --service-account-json ./service-account.json \
    --window 2020-01-01..2026-05-20 \
    --out inventory/google-sheets-workbooks.yaml
```

The Google Drive API enumerates all Sheets the tenant owns. For each, fetch the export URLs (XLSX for the export + Apps-Script-source for any scripts).

Airtable:

```sh
oya sheets migrate inventory \
    --source airtable \
    --airtable-api-key "$AIRTABLE_API_KEY" \
    --out inventory/airtable-bases.yaml
```

For each Airtable base: schema + records + linked-records + views + automations + buttons.

Coda:

```sh
oya sheets migrate inventory \
    --source coda \
    --coda-api-token "$CODA_API_TOKEN" \
    --out inventory/coda-docs.yaml
```

## Step 2 — Audit script / automation portability (≤ 1 week)

```sh
oya sheets migrate script-portability-audit \
    --inventory inventory/google-sheets-workbooks.yaml \
    --source-platform google-sheets \
    --out audit/google-apps-script-portability.yaml
```

The audit:

1. For each workbook, identify any Apps Scripts.
2. Classify the script: `purely-utility` (function library; portable as oyatie functions), `external-integration` (calls Gmail / Calendar / Drive — port via workflow-engine), `webhook-triggered` (port via workflow-engine trigger), `complex-state-management` (manual port; complex).
3. Emit a per-script-recommended-action.

Similarly for Airtable Automations + Coda Buttons.

## Step 3 — Convert + upload workbooks (≤ 2-4 weeks)

For Google Sheets (the simplest path; XLSX export + re-import):

```sh
oya sheets migrate convert-google-sheets \
    --inventory inventory/google-sheets-workbooks.yaml \
    --output-dir ./migration-staging/google-sheets/ \
    --concurrency 4
```

For each Sheet:

1. Export as XLSX from Google Drive API.
2. Convert via the standard XLSX import pipeline.
3. Preserve metadata (sheet name, sharing-list, comments).
4. Note any conversion warnings (formulas using LAMBDA in old Google Sheets; some array-formula configurations).

For Airtable:

```sh
oya sheets migrate convert-airtable \
    --inventory inventory/airtable-bases.yaml \
    --output-dir ./migration-staging/airtable/ \
    --mapping airtable-to-oyatie-mapping.yaml
```

Airtable's field types map per a custom mapping:

| Airtable field type | oyatie cell type | Notes |
|---|---|---|
| Single-line text | Text | Direct |
| Long text | Text + cell-format = wrap | Direct |
| Attachment | URL + foundry-bridge to attached blob | Attachments become URL-references; blobs move to drive µservice |
| Checkbox | Boolean | Direct |
| Single-select | Text + data-validation list | List rules |
| Multi-select | Array<Text> + data-validation list | Each row's value is JSON-encoded array |
| Date | Date | Direct |
| Date+Time | DateTime | Direct |
| Phone number | Text + format validation | Direct |
| Email | Text + format validation | Direct |
| URL | URL | Direct |
| Number | Number | Direct |
| Currency | Number + format = currency | Direct |
| Percentage | Number + format = % | Direct |
| Rating | Number | Range 1-5 etc. |
| Linked record | Cross-sheet reference + named-range | Linked records become a named-range cross-sheet pattern |
| Formula | Formula | Most port; some Airtable-specific functions require manual port |
| Rollup | Formula = SUMIF over linked records | Manual port; check the rollup definition |
| Lookup | Formula = VLOOKUP / INDEX-MATCH | Manual port |
| Count | Formula = COUNTIF | Direct |
| Auto-number | Formula = ROW() | Direct |
| Created time | Cell metadata (audit-chain `cell_created_at`) | Direct |
| Last modified time | Cell metadata (audit-chain `cell_modified_at`) | Direct |

## Step 4 — Re-build automations in workflow-engine (≤ 2-8 weeks per tenant)

For each Apps Script / Airtable Automation / Coda Button identified in Step 2, re-build as a workflow-engine workflow:

```sh
oya workflow-engine import \
    --source google-apps-script \
    --script-file ./scripts/my-monthly-report.gs \
    --target-tenant drill-acme \
    --output workflow-definitions/my-monthly-report.yaml
```

The migration is per-script; complex scripts may need 1-3 days of engineer time each. Plan accordingly.

## Step 5 — Test + cutover (≤ 4-12 weeks)

For each tenant:

- Day 0-14: import all workbooks into oyatie; users test in oyatie alongside source.
- Day 14-28: users test workflow-engine automations alongside source automations.
- Day 28-42: cut over editing to oyatie; source becomes read-only.
- Day 42+: per source contract, downgrade or cancel.

Monitor:

```sh
oya sheets migrate cutover-status --tenant drill-acme --source google-sheets
```

Tracks: workbooks-open-on-oyatie-vs-source, formula-evaluation-drift, user-feedback-flags.

## Step 6 — Decommission source (≤ 1 month)

```sh
oya sheets migrate decommission \
    --tenant drill-acme \
    --source google-sheets \
    --evidence-out evidence/migrations/google-sheets-to-oyatie-drill-acme.json
```

The evidence: workbook count, conversion log, script-portability decisions, workflow-engine workflows authored, cutover-period metrics, source decommission date.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Apps Script does not port automatically | High | Audit per Step 2; budget engineer time per complex script |
| Airtable Automations do not port | High | Same; re-build in workflow-engine |
| Linked-record semantics in Airtable differ from oyatie cross-sheet | Medium | Per-table review; the converter flags |
| Formula edge cases (LAMBDA, LET) | Medium | Per-workbook fidelity check; manual fix if needed |
| Cross-source-workbook references break during migration | High | Migrate in dependency order; do not delete source while a workbook is referenced |
| User UX divergence | High | Pilot with 5-10 power users for 30 d before broad rollout |
| Tenant relies on source's "Importrange" cross-document references | High | Convert to oyatie's connected-sheets pattern |
