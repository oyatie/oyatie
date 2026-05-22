---
doc_class: MigrationPlaybook
microservice: contract-lifecycle-management
source_vendor: DocuSign CLM
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Migration Playbook — DocuSign CLM → oyatie contract-lifecycle-management

Audience: a legal-ops team currently on DocuSign CLM (formerly SpringCM) Enterprise who wants to migrate to oyatie's CLM substrate over 12-16 weeks.

Outcome: all templates migrated, all in-flight contracts continued on oyatie, all signed contracts archived with cryptographic continuity, DocuSign CLM platform decommissioned (you may keep DocuSign eSignature standalone).

## Phase 0 — discovery (week 1)

1. Inventory DocuSign CLM:
   - Contract types (Configuration → Contract Types).
   - Templates per contract type (Templates → list).
   - Active workflows (Workflows → list with version).
   - Active in-flight contracts (Reports → Contracts In-Flight).
   - Signed contracts in the repository (Reports → Signed Contracts; export to CSV).
   - User roles + permissions.
   - Integrations (CRM connectors, BYO DocuSign accounts, API webhooks).
   - Obligation tracker configuration.
2. Inventory commercial exposure:
   - DocuSign CLM contract end date.
   - Per-seat pricing tier + minimum-seat commit.
   - Envelope volume + per-envelope rate (DocuSign eSignature billing is separate).
   - Storage envelope (DocuSign CLM has a storage cap; overage charges).
3. Identify high-priority migration scope:
   - Pack-bound tenants (KR-PIPA, HIPAA, EU GDPR) — migrate first.
   - High-volume contract types (NDAs, MSAs, employment offers) — establish migration patterns.
   - Long-tail (one-off contract types) — migrate last.

Deliverable: `migration-plan.md` enumerating every DocuSign CLM artefact + target oyatie tier.

## Phase 1 — stand up oyatie + integrate DocuSign eSignature (weeks 2-3)

1. Deploy oyatie CLM IaC into the target cell per `iac/clm-canonical-helm.yaml`.
2. Configure DocuSign eSignature as the bundled e-signature provider (you keep your existing DocuSign eSignature account; only the CLM platform changes).
3. Smoke test: create a sample contract type, sample template, sample workflow, end-to-end signature via DocuSign. Verify everything works before migrating any production data.

## Phase 2 — template migration (weeks 4-7)

DocuSign CLM templates are MS Word DOCX with proprietary SpringCM merge fields. oyatie uses standard OOXML with JSPath bindings. The converter is at:

```sh
cargo run -p oya-dev-cli -- clm template-import \
    --source docusign-clm \
    --input templates/dscm-msa-v3.docx \
    --output microservices/contract-lifecycle-management/templates/msa-v3.docx \
    --mapping templates/dscm-field-mapping.json
```

Manual mapping file (`dscm-field-mapping.json`):

```json
{
  "{{Customer_Name}}": "customer.legal_name",
  "{{Customer_Address_Block}}": "customer.registered_address",
  "{{Effective_Date}}": "effective_date",
  "{{Service_Description}}": "services.description",
  "{{Fees_Annual}}": "services.fees_eur",
  "{{Term_Months}}": "term_months",
  "{{Governing_Law}}": "governing_law",
  "{{Counterparty_Signer_Name}}": "counterparty.signer.name",
  "{{Counterparty_Signer_Email}}": "counterparty.signer.email"
}
```

For each template:
1. Run the converter.
2. Open the result in MS Word; verify the formatting + placeholders look right.
3. Bind to the corresponding oyatie contract type.
4. Test-render with 3 sample data sets.
5. Publish v1.0.0.

Plan: 1-2 days per medium-complexity template; 4-5 days per high-complexity (custom field types, conditional sections, etc).

## Phase 3 — workflow migration (weeks 8-9)

DocuSign CLM workflows use the proprietary "Process Flow" designer. oyatie uses a declarative YAML/JSON workflow definition.

For each DocuSign CLM workflow:
1. Document the stages on paper (Draft → Review → Approve → Send → Signed; with branches + approvers).
2. Configure the oyatie workflow via portal → Workflows → "New workflow".
3. Map approver roles: DocuSign CLM uses static role assignments; oyatie uses dynamic Cedar-permit-based assignment (e.g. "approver = any user with `clm::contract::approve` AND in user.org_unit = legal").
4. Test with a synthetic contract; verify each stage transition + approver routing.

## Phase 4 — in-flight contract migration (weeks 10-11)

For each contract currently in DocuSign CLM but not yet signed:
1. Export the latest draft via DocuSign CLM API: `dscm export-contract --id <id> --include-redlines`.
2. Upload to oyatie via `cargo run -p oya-dev-cli -- clm contract-import-inflight`.
3. The substrate creates a new contract in oyatie with the imported draft as the current revision; previous revisions become history entries.
4. Continue the contract lifecycle in oyatie. The counterparty doesn't notice the platform change unless they're also in DocuSign CLM (rare).

Communications:
- Internal: 7 days advance notice + 24 h reminder to legal-ops team.
- External: only if a counterparty is co-authoring in DocuSign CLM (rare in B2B).

## Phase 5 — signed contract archive (weeks 12-13)

For each signed contract in DocuSign CLM's repository:
1. Export via API in bulk: `dscm bulk-export --signed-after 2018-01-01 --format zip`.
2. Each contract bundle includes: signed PDF, signature audit trail (DocuSign Certificate of Completion), metadata.
3. Import to oyatie's WORM cold tier:

```sh
cargo run -p oya-dev-cli -- clm signed-contract-import \
    --source-format dscm-bulk-zip \
    --input ./dscm-archive/ \
    --target-retention 7y \
    --emit-audit-chain-anchor true \
    --emit-chain-of-custody "imported-from-dscm-2026-05-20-by-legal-ops"
```

The import:
- Writes each PDF to SeaweedFS WORM Compliance.
- Cross-emits a Merkle anchor to `audit-chain`.
- Records chain-of-custody (who, when, source).
- Preserves the original DocuSign signature evidence (the Certificate of Completion is the legal proof of signature; oyatie doesn't re-sign).

Retention re-anchor: oyatie starts a 7-y retention clock from the import date PLUS preserves the original DocuSign signed-date for legal-effect calculations. If a regulator asks "show me a contract signed on 2020-03-15", oyatie returns the contract with its original 2020-03-15 signature evidence intact.

## Phase 6 — obligation re-extraction (week 14)

For high-value signed contracts (typically those with `value_usd > 100_000` or those involving regulated data), re-run obligation extraction in oyatie:

```sh
cargo run -p oya-dev-cli -- clm obligation-extract-bulk \
    --tenant <your-tenant-id> \
    --filter "value_usd > 100000" \
    --output obligation-export.csv
```

Compare against DocuSign CLM's existing obligation tracker. Discrepancies:
- oyatie finds new obligations DocuSign missed → import to oyatie obligation tracker.
- DocuSign tracker has obligations oyatie missed → manual import.
- Both agree → import to oyatie.

After 7 days, oyatie's obligation tracker is canonical.

## Phase 7 — cutover (week 15)

1. Disable user logins on DocuSign CLM (route to "platform migrated to oyatie" landing page).
2. Update CRM connectors to point at oyatie's webhooks.
3. Update API integrations to call oyatie's endpoints.
4. Email all internal users with the cutover announcement.

## Phase 8 — wind-down (week 16)

1. Cancel DocuSign CLM contract (you keep DocuSign eSignature standalone, billed separately).
2. Receive final DocuSign CLM invoice; pay any minimum-commit residual.
3. Update tenant ARCHITECTURE.md § "Contract Lifecycle" to reference oyatie exclusively.

## Common pitfalls

| Pitfall | Mitigation |
|---|---|
| DocuSign CLM "Custom Code" workflows | These run server-side JavaScript inside DocuSign; no direct oyatie equivalent. Reimplement as `http-call` to a tenant API |
| SpringCM legacy "Folder permissions" model | oyatie uses Cedar permit-based access (richer); explicit re-permission required |
| Templates with conditional sections (DocuSign's "include if") | oyatie supports via JSPath conditionals in OOXML; manual review per template |
| Signed contracts with DocuSign-specific merge fields visible in the PDF | The PDF is immutable; merge fields are baked in; no remediation needed unless a regulator complains |
| Integration with Salesforce via the DocuSign Salesforce app | Switch to oyatie's Salesforce connector (portal → Integrations → Salesforce); requires Salesforce admin's API permissions |
| BYO DocuSign account credentials embedded in workflows | Move credentials to `kms` µservice; reference via `${kms.docusign.api_key}` in oyatie workflows |
| Contract IDs as foreign keys in tenant systems | Provide a mapping table during cutover; oyatie generates new IDs but the import preserves the original DocuSign CLM ID in a `legacy_id` field |
