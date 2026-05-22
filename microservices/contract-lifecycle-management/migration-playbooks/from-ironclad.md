---
doc_class: MigrationPlaybook
microservice: contract-lifecycle-management
source_vendor: Ironclad
dimension_id: X-D4 (audit) + S-001 (substance)
date: 2026-05-21
---

# Migration Playbook: Ironclad → Oyatie CLM

This playbook is the executable, step-by-step procedure to migrate a tenant's contracts from Ironclad to Oyatie CLM. Use in conjunction with `vendor-mapping/ironclad-field-mapping.md` for field-level reference.

## Prerequisites

- Tenant has provisioned an Oyatie CLM µservice in their chosen `deployment_context`.
- Tenant has signed Oyatie's Master Service Agreement.
- Tenant has selected `tenant_class` (typically `paid + billing_components=[per_seat]`).
- Tenant has configured `jurisdiction_packs` per their legal scope.
- Source Ironclad tenant has an API key with read access to all relevant workflows.
- The migration operator has BAA / NDA with both Oyatie and the customer.

## Phase 0: discovery + scoping (1-2 weeks)

1. Inventory Ironclad assets:
   ```bash
   oya migration-discover --vendor ironclad --tenant <tenant_id> \
     --ironclad-api-key <key> --output discovery.json
   ```
   Yields counts of: workflows, records, signed contracts, draft contracts, active obligations, custodians, approval chains, integrations.

2. Map Ironclad workflows → Oyatie contract types per `taxonomies/contract-type-taxonomy.md`. The discovery report flags any unmapped workflow as `needs-manual-mapping`.

3. Identify pack overlays needed for the tenant's contracts:
   - GDPR-relevant contracts → `gdpr` pack.
   - HIPAA BAAs → `hipaa-baa` pack.
   - SOX-relevant contracts → `sox-404` pack.
   - Korean contracts → `kr-pipa` pack.
   - Broker-dealer contracts → `sec-17a-4` pack.

4. Identify integrations to replace:
   - Salesforce sync (if used) → Oyatie crm µservice cross-emit.
   - Slack notifications → Oyatie workflow-engine notification adapter.
   - Webhooks → Oyatie webhook adapter.

5. Schedule a freeze window (typically 24-72 hours) during which no new contracts are created in Ironclad and the cutover happens.

## Phase 1: tenant provisioning (1-2 days)

1. Provision tenant in Oyatie:
   ```bash
   tofu apply -var tenant_id=<tenant> \
     -var deployment_context=<context> \
     -var jurisdiction_packs='["gdpr","esign"]' \
     -var tenant_class=paid \
     -var billing_components='["per_seat"]'
   ```

2. Configure pack activations and Cedar policies.

3. Provision counterparty MDM:
   - Import all Ironclad counterparties via discovery report.
   - Run sanctions screening per `counterparty-mdm/counterparty-mdm.md`.
   - Resolve LEI / EIN / company-registry via GLEIF and per-jurisdiction registries.

4. Configure approval routing matrix per `legal-dimensions/approval-routing-matrix.md`.

5. Import clause library templates from Ironclad Schemas.

## Phase 2: signed-contract migration (1-3 days)

1. Bulk export signed contracts from Ironclad:
   ```bash
   oya migration-export --vendor ironclad --tenant <tenant_id> \
     --status signed --output signed-contracts/
   ```

2. For each signed contract:
   - Materialize the signed PDF artefact.
   - Verify the existing signature on the PDF (PAdES or DocuSign attestation).
   - **Do NOT re-sign**. Preserve the original signature evidence as-is; cross-emit a `migrated_from_ironclad_with_original_signature` annotation.
   - Apply Oyatie audit-chain seal over the migrated artefact (this seal does not replace the original signature; it adds a chain-of-custody record).

3. Migrate metadata fields per `vendor-mapping/ironclad-field-mapping.md`.

4. Migrate activity log → Oyatie audit-chain events as historical-import events.

5. Apply WORM lock per pack overlay (e.g. `sec-17a-4` → SeaweedFS Compliance bucket).

6. Validate: for each migrated contract, run `oya migration-validate --contract-id <id>` to verify field completeness, signature evidence preservation, retention policy applied.

## Phase 3: draft + in-flight migration (1-2 days)

1. Export draft and in-flight contracts from Ironclad.

2. For each draft:
   - Migrate via `vendor-mapping/ironclad-field-mapping.md`.
   - State: Draft / Review / Approved (based on Ironclad status).
   - Migrate redline history → Oyatie redline events.
   - Migrate approval history → Oyatie approval evidence.

3. For in-flight contracts (sent for signature but not yet signed):
   - Coordinate with the signatories.
   - Option A: complete signing in Ironclad first; then migrate as signed (preferred).
   - Option B: rescind the Ironclad envelope; re-create in Oyatie; re-send for signature.

## Phase 4: obligation migration (1 day)

1. For each signed contract, re-run IP-027 obligation extraction against the Oyatie-imported version.

2. Compare extracted obligations with any obligations Ironclad had identified.

3. Reconcile differences (likely the Oyatie extraction will find additional obligations).

4. Surface low-confidence extractions for human review.

## Phase 5: cutover (24-72 hours freeze)

1. Announce freeze to all contract authors.

2. Final delta export from Ironclad.

3. Final delta migration to Oyatie.

4. DNS / domain switch (if applicable): legal-ops.tenant.com → oyatie-clm endpoint.

5. Disable Ironclad write access; preserve Ironclad as read-only for 90 days as a fallback.

6. Announce Oyatie CLM as the canonical CLM to all contract authors.

## Phase 6: post-cutover (ongoing)

1. Monitor Oyatie CLM usage via SLO dashboards.

2. Maintain Ironclad read-only access for 90 days.

3. After 90 days, formally decommission Ironclad tenant (with the customer's written confirmation).

4. Recompute compliance attestations against the new Oyatie evidence base.

## Demo_trial conversion path

A common pattern: customer signs up for an Oyatie CLM `tenant_class=demo_trial` to evaluate the migration. They migrate a representative subset (5-10 contracts) into the demo_trial. After validation, they convert to `tenant_class=paid` and run the full migration.

Demo_trial limits: max 5 active contracts, max 100KB document size, AES-only e-signature, 30-day retention. Full migration requires conversion to paid.

## Common pitfalls

1. **Counterparty deduplication**: Ironclad may have the same counterparty under multiple spellings ("Acme Inc.", "Acme, Inc.", "ACME INC"). Oyatie's counterparty MDM dedups; verify the dedup is correct.
2. **Custom field translation**: Ironclad workflows have arbitrary custom fields. Some map cleanly to Oyatie metadata; others may need a tenant-specific extension. Decide upfront which fields are first-class and which are bag-of-tags.
3. **Approval chain ordering**: Ironclad supports sequential + parallel approvers. Oyatie's matrix supports both; verify the ordering is preserved.
4. **AI suggestion provenance**: Ironclad Jurist suggestions are not provenance-tracked the same way Oyatie tracks them. Pre-migration, accept that historical Ironclad AI suggestions become "unattributed historical edits" in Oyatie.

## Rollback

If cutover fails:

1. Re-enable Ironclad write access.
2. Disable Oyatie write access (read-only).
3. Document the failure mode + root cause.
4. Re-run cutover after fix.

## Estimated effort

- Discovery + scoping: 1-2 weeks.
- Provisioning: 1-2 days.
- Migration (depending on volume):
  - 1k-10k contracts: 1-2 weeks total.
  - 10k-100k contracts: 4-6 weeks total.
  - >100k contracts: 8-12 weeks total.
- Cutover freeze: 24-72 hours.

## Audit events

- `oya.contract.lifecycle.management.migration.ironclad.phase_started`
- `oya.contract.lifecycle.management.migration.ironclad.discovery_completed`
- `oya.contract.lifecycle.management.migration.ironclad.contract_migrated`
- `oya.contract.lifecycle.management.migration.ironclad.obligation_re_extracted`
- `oya.contract.lifecycle.management.migration.ironclad.cutover_completed`
- `oya.contract.lifecycle.management.migration.ironclad.tenant_decommissioned`
