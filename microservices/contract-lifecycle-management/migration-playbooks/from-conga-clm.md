---
doc_class: MigrationPlaybook
microservice: contract-lifecycle-management
source_vendor: Conga CLM (formerly Apttus)
dimension_id: X-D4 (audit) + S-003 (substance)
date: 2026-05-21
---

# Migration Playbook: Conga CLM → Oyatie CLM

This playbook is the executable procedure to migrate a tenant's contracts from Conga CLM (formerly Apttus Contract Management) to Oyatie CLM. Use in conjunction with `vendor-mapping/conga-clm-field-mapping.md`.

Conga CLM presents two deployment shapes:

- **Salesforce-native** (the dominant historical install base; Apttus's heritage).
- **Conga standalone** (post-2021 split from Salesforce).

This playbook covers both; differences are called out.

## Prerequisites

- Tenant Oyatie CLM µservice provisioned in their `deployment_context`.
- Tenant has signed Oyatie's Master Service Agreement.
- Tenant has selected `tenant_class` (typically `paid + billing_components=[per_seat]`; large enterprise customers may add `per_usage`).
- Tenant has configured `jurisdiction_packs`.
- Source Conga / Salesforce access:
  - Salesforce-native: Apex-enabled API user with read access to Apttus__* objects + Salesforce Account / Contact / Opportunity.
  - Standalone: Conga API key.

## Phase 0: discovery + scoping (1-3 weeks; Conga discovery often takes longer than Ironclad due to Salesforce data-model complexity)

1. Inventory Conga / Salesforce assets:
   ```bash
   oya migration-discover --vendor conga-clm --tenant <tenant_id> \
     --salesforce-instance <url> --salesforce-api-key <key> \
     --output discovery.json
   ```
   Yields counts of: agreements, clauses, MSAs, order forms, schedules, amendments, accounts, contacts, opportunities (with CPQ-CLM bridge), workflows, approval chains.

2. Map Conga agreements → Oyatie contract types per `taxonomies/contract-type-taxonomy.md`.

3. Identify Salesforce custom fields that need migration:
   - Most Apttus__ standard fields map per `vendor-mapping/conga-clm-field-mapping.md`.
   - Custom fields on Agreement (Apttus__Agreement__c) require tenant-specific decisions.
   - Validate Salesforce reports / dashboards used; these may need re-implementation in Oyatie's `dashboards/`.

4. Identify Salesforce automations:
   - Apex triggers on Agreement / Clause objects.
   - Workflow rules / Process Builder / Flow automations.
   - These do NOT migrate; document and re-implement in Oyatie workflow-engine + Cedar.

5. Identify CPQ-CLM bridge (if used):
   - Salesforce CPQ Quote → Apttus__Agreement__c flow.
   - This is Q-001 open issue: Oyatie's CPQ-CLM bridge is in roadmap. For now, manual quote → contract creation, or use Oyatie crm µservice with Quote object.

6. Identify Conga Composer template usage:
   - Composer templates produce documents from Salesforce data.
   - Migrate templates to Oyatie clause library + variable bindings.

7. Schedule freeze window (typically 48-96 hours for Conga; longer than Ironclad due to Salesforce sync complexity).

## Phase 1: tenant provisioning (2-3 days; longer than Ironclad due to Salesforce sync)

1. Provision tenant per the standard Oyatie provisioning flow.

2. Configure crm ↔ CLM cross-emit (since Conga heritage is Salesforce-native, the customer likely keeps Salesforce for CRM and Oyatie for CLM):
   - Salesforce Account → Oyatie counterparty (one-way sync from SFDC).
   - Salesforce Contact → Oyatie counterparty.signatory_authorities[].
   - Salesforce Opportunity → Oyatie crm.opportunity.
   - Configure the Salesforce middleware / iPaaS layer (Workato, MuleSoft, Boomi) to forward updates to Oyatie.

3. Import clause library from Apttus__Clause__c.

4. Configure approval routing matrix.

## Phase 2: agreement migration (1-3 weeks; bulk export from Salesforce)

1. Bulk export Agreement records via Salesforce Bulk API:
   ```bash
   sfdx force:data:bulk:query -q "SELECT Id, Name, Apttus__Status__c, ..." \
     -u <salesforce-instance> -p data/agreements.csv
   ```

2. Bulk export attached documents via Salesforce ContentVersion API.

3. For each agreement:
   - Map per `vendor-mapping/conga-clm-field-mapping.md`.
   - Migrate parent-child relationships (MSA → Order Forms / Schedules / Amendments).
   - **Do NOT re-sign**. Preserve original signature evidence as-is (per the Ironclad playbook approach).
   - Apply Oyatie audit-chain seal as a chain-of-custody record.
   - Apply pack overlays + WORM lock.

4. Validate each migrated agreement.

## Phase 3: clause library + Composer templates (2-5 days)

1. Migrate Apttus__Clause__c records to Oyatie clause_templates.

2. For each Conga Composer template:
   - Parse the Composer template's variable bindings.
   - Translate to Oyatie clause library template syntax.
   - Test rendering with sample data.

## Phase 4: obligation migration (1 day)

Same as Ironclad playbook: re-run IP-027 obligation extraction against migrated agreements.

## Phase 5: cutover (48-96 hours freeze)

1. Announce freeze.
2. Final delta export from Conga.
3. Final delta migration.
4. Switch the Salesforce ↔ CLM integration from Conga to Oyatie:
   - Remove Apttus from Salesforce package (if standalone), OR
   - Disable Apttus triggers + workflow rules.
5. Enable Oyatie ↔ Salesforce cross-emit.
6. Announce Oyatie CLM as canonical.

## Phase 6: post-cutover (ongoing)

Same as Ironclad playbook.

## Salesforce-native specific considerations

If the tenant's Salesforce org runs other Apttus components beyond CLM (e.g. Apttus CPQ, Apttus Billing):

- Apttus CPQ → Oyatie crm CPQ + cloud-billing.
- Apttus Billing → Oyatie cloud-billing.

These migrations are out of scope for this CLM playbook but should be sequenced.

## Conga AI history

Conga AI suggestions are vendor-proprietary and don't migrate. Pre-migration, document that historical Conga AI suggestions become "unattributed historical edits" in Oyatie.

## Common pitfalls

1. **Salesforce field-level security**: Apttus__Agreement__c fields may be restricted by Salesforce FLS. Verify the migration API user has read access to all relevant fields.
2. **Apttus__Master_Agreement__c**: parent-child relationships must be migrated in topological order (MSAs before Order Forms before Amendments).
3. **Currency conversion**: if Salesforce multi-currency is enabled, ensure currency conversion rates at migration time match contract execution dates.
4. **Salesforce sandboxes vs production**: discovery should target the production org; sandboxes typically have stale data.
5. **Conga Composer template parsing**: not all Composer templates translate cleanly to Oyatie clause library syntax. Surface complex templates for manual rework.

## Rollback

Same as Ironclad playbook.

## Estimated effort

- Discovery + scoping: 1-3 weeks (longer due to Salesforce complexity).
- Provisioning: 2-3 days.
- Migration:
  - 1k-10k agreements: 2-3 weeks total.
  - 10k-100k agreements: 6-8 weeks total.
  - >100k agreements: 10-16 weeks total.
- Cutover freeze: 48-96 hours.

## Audit events

- `oya.contract.lifecycle.management.migration.conga.phase_started`
- `oya.contract.lifecycle.management.migration.conga.discovery_completed`
- `oya.contract.lifecycle.management.migration.conga.agreement_migrated`
- `oya.contract.lifecycle.management.migration.conga.composer_template_translated`
- `oya.contract.lifecycle.management.migration.conga.obligation_re_extracted`
- `oya.contract.lifecycle.management.migration.conga.cutover_completed`
- `oya.contract.lifecycle.management.migration.conga.salesforce_integration_switched`
