---
doc_class: Tutorial
tutorial_id: TUT-OYATIE-ONT-SFDC-009
persona: "Noah Bennett, RevOps analyst migrating Salesforce CRM data"
prerequisite_packs:
  - canonical-base
  - ontology-projection
  - workplace-integration
  - sales-cloud-core
related_oyatie_adrs:
  - ADR-0055
  - ADR-0059
  - ADR-0243
  - ADR-0244
  - ADR-0257
  - ADR-0316
status: Draft
date: 2026-05-20
owner: docs-experience
estimated_completion_time: "110 minutes"
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# Project Salesforce CRM Data into the Oyatie Ontology

## Goal

You will connect a Salesforce sandbox, map Account, Contact, and Opportunity objects into Oyatie ontology types, publish a tenant-scoped projection named `projection:salesforce-revops-pipeline:v1`, run a dry sync, materialize approved records, and verify lineage plus Cedar field visibility.

## Prerequisites

- RevOps account: `noah.bennett@acme.example`.
- Tenant: `tenant-acme-robotics`.
- Workspace: `workspace-revops-migration`.
- External source: `Salesforce sandbox acme-sfdc-dev`.
- External source id: `source-salesforce-acme-dev`.
- Projection id: `projection:salesforce-revops-pipeline:v1`.
- Role projection id: `role-projection:revops-analyst:v1`.
- Lineage policy id: `lineage-policy:salesforce-revops:v1`.
- Capability tier: `sales-cloud-core`.
- Subscribed microservices: `connector`, `ontology`, `policy-engine`, `workflow-engine`, `audit-chain`, `tenancy`, `identity`, `drive`, `intelligence`.
- Required Cedar permit: `connect.source.register`.
- Required Cedar permit: `connect.source.test`.
- Required Cedar permit: `ontology.projection.create`.
- Required Cedar permit: `ontology.projection.preview`.
- Required Cedar permit: `ontology.projection.materialize`.
- Required Cedar permit: `ontology.projection.rollback`.
- Required Cedar permit: `policy.projection.evaluate`.
- Required Cedar permit: `audit.ontology.read`.
- Salesforce object: `Account`.
- Salesforce object: `Contact`.
- Salesforce object: `Opportunity`.
- Sample Account id: `001ACME0000001`.
- Sample Opportunity id: `006ACME0000420`.
- Named saved query: `tutorial.ontology_salesforce_projection_status`.

## Step-by-Step

1. Open the RevOps migration workspace.
   - Sign in as `noah.bennett@acme.example`.
   - Switch to `Acme Robotics`.
   - Open workspace `RevOps Migration`.
   - Confirm tenant context: `tenant-acme-robotics`.
   - Confirm capability tier: `sales-cloud-core`.
   - Open `Ontology -> Projection Studio`.
   - Screenshot checkpoint: capture the Projection Studio landing page.
   - If `sales-cloud-core` is not active, stop and ask the tenant admin to grant it.
   - The projection is tenant-scoped and must not create a CRM microservice fork.
   - Keep `Lineage required` visible in the toolbar.

2. Register the Salesforce source.
   - Click `External sources`.
   - Click `Register source`.
   - Source type: `Salesforce`.
   - Source id: `source-salesforce-acme-dev`.
   - Display name: `Salesforce sandbox acme-sfdc-dev`.
   - Environment: `Sandbox`.
   - Authentication: `OAuth client credential via tenant secret`.
   - Tenant secret path: `openbao://tenant-acme-robotics/connect/salesforce/dev`.
   - Data residency: `US`.
   - Click `Save source`.
   - Expected toast: `External source registered`.

3. Test the source connection.
   - Open the source details.
   - Click `Test connection`.
   - Expected status: `Connected`.
   - Expected API version: `v60.0` or later.
   - Expected objects discovered: `Account`, `Contact`, `Opportunity`.
   - Expected permission mode: `read-only import`.
   - Screenshot checkpoint: capture the connection test.
   - If credentials fail, rotate the tenant secret before continuing.
   - Do not store Salesforce credentials in the projection manifest.
   - Close the source drawer.

4. Create the projection draft.
   - Click `New projection`.
   - Projection id: `projection:salesforce-revops-pipeline:v1`.
   - Capability tier: `sales-cloud-core`.
   - Tenant scope: `tenant-acme-robotics`.
   - Source id: `source-salesforce-acme-dev`.
   - Lifecycle status: `draft`.
   - ADR refs: `ADR-0257`, `ADR-0316`.
   - Click `Create draft`.
   - Expected canvas title: `Salesforce RevOps Pipeline`.
   - Screenshot checkpoint: capture projection metadata.

5. Map Salesforce Account.
   - Click `Add object mapping`.
   - Source object: `Salesforce Account`.
   - Ontology object ref: `ontology://crm/sales#Account`.
   - Source id field: `Id`.
   - Display field: `Name`.
   - Tenant field: static `tenant-acme-robotics`.
   - Owner field: `OwnerId`.
   - Status field: `IsDeleted`.
   - Field mapping: `Name -> account_name`.
   - Field mapping: `Industry -> industry`.
   - Field mapping: `AnnualRevenue -> annual_revenue`.
   - Click `Save mapping`.

6. Map Salesforce Contact.
   - Click `Add object mapping`.
   - Source object: `Salesforce Contact`.
   - Ontology object ref: `ontology://crm/sales#Contact`.
   - Source id field: `Id`.
   - Display field: `Name`.
   - Parent link: `AccountId -> ontology://crm/sales#Account`.
   - Field mapping: `Email -> work_email`.
   - Field mapping: `Title -> job_title`.
   - Field mapping: `Phone -> business_phone`.
   - Field visibility for `Email`: `sales_team_only`.
   - Field visibility for `Phone`: `sales_team_only`.
   - Screenshot checkpoint: capture Contact field visibility.

7. Map Salesforce Opportunity.
   - Click `Add object mapping`.
   - Source object: `Salesforce Opportunity`.
   - Ontology object ref: `ontology://crm/sales#Opportunity`.
   - Source id field: `Id`.
   - Display field: `Name`.
   - Parent link: `AccountId -> ontology://crm/sales#Account`.
   - Field mapping: `StageName -> stage`.
   - Field mapping: `Amount -> forecast_amount`.
   - Field mapping: `CloseDate -> expected_close_date`.
   - Field mapping: `Probability -> probability_percent`.
   - Click `Save mapping`.
   - Screenshot checkpoint: capture Opportunity mapping.

8. Define relationship mappings.
   - Open `Relations`.
   - Add relation: `ontology://crm/sales#Account->Opportunity`.
   - Source join: `Opportunity.AccountId = Account.Id`.
   - Cardinality: `one Account to many Opportunities`.
   - Add relation: `ontology://crm/sales#Account->Contact`.
   - Source join: `Contact.AccountId = Account.Id`.
   - Cardinality: `one Account to many Contacts`.
   - Click `Save relations`.
   - Expected validation: `Relation cardinality valid`.
   - Screenshot checkpoint: capture relation map.
   - Relations must not cross tenant boundaries.

9. Pin schema revisions.
   - Open `Schema pins`.
   - Account schema revision: `crm.sales.Account@2026-05-20`.
   - Contact schema revision: `crm.sales.Contact@2026-05-20`.
   - Opportunity schema revision: `crm.sales.Opportunity@2026-05-20`.
   - Relation schema revision: `crm.sales.Relations@2026-05-20`.
   - Function revision: `calculateForecastAmount@2026-05-20`.
   - Click `Save pins`.
   - Expected validation: `All schema revisions pinned`.
   - Screenshot checkpoint: capture schema pin table.
   - ADR-0257 requires pins before projection activation.

10. Configure lineage policy.
    - Open `Lineage`.
    - Lineage policy id: `lineage-policy:salesforce-revops:v1`.
    - Source refs required: enabled.
    - Derived field lineage required: enabled.
    - Source timestamp field: `SystemModstamp`.
    - Source record url template: `https://acme-dev.my.salesforce.com/{Id}`.
    - Derived field: `weighted_pipeline = Amount * Probability`.
    - Derived field input refs: `Opportunity.Amount`, `Opportunity.Probability`.
    - Click `Save lineage`.
    - Screenshot checkpoint: capture lineage rules.
    - Projection fields must not become silent new sources of truth.

11. Configure role projection.
    - Open `Role projections`.
    - Role projection id: `role-projection:revops-analyst:v1`.
    - Role: `RevOps Analyst`.
    - Visible objects: `Account`, `Contact`, `Opportunity`.
    - Hidden fields: `Contact.PersonalEmail`, `Contact.HomePhone`.
    - Allowed actions: `projection.read`, `forecast.calculate`.
    - Denied actions: `source.write`, `contact.export_unrestricted`.
    - Cache partition keys: `tenant_id`, `role_projection_id`.
    - Click `Save role projection`.
    - Screenshot checkpoint: capture the role projection.

12. Validate Cedar policy.
    - Open `Policy`.
    - Principal: `noah.bennett@acme.example`.
    - Action: `ontology.projection.preview`.
    - Resource: `projection:salesforce-revops-pipeline:v1`.
    - Context: `role-projection:revops-analyst:v1`.
    - Click `Evaluate`.
    - Expected decision: `Permit`.
    - Change action to `source.write`.
    - Click `Evaluate`.
    - Expected decision: `Deny`.
    - Screenshot checkpoint: capture both permit and deny results.

13. Run projection validation.
    - Click `Validate projection`.
    - Expected check: `schema pins present`.
    - Expected check: `field visibility complete`.
    - Expected check: `lineage policy complete`.
    - Expected check: `cache partition safe`.
    - Expected check: `Cedar fixtures pass`.
    - Expected check: `no projection writes to source`.
    - If any check fails, open the failed row and repair it.
    - Screenshot checkpoint: capture validation result.
    - Click `Save draft`.
    - Expected toast: `Projection draft saved`.

14. Run a dry sync.
    - Click `Dry sync`.
    - Source: `source-salesforce-acme-dev`.
    - Record limit: `500`.
    - Sample Account id: `001ACME0000001`.
    - Sample Opportunity id: `006ACME0000420`.
    - Mutation mode: `none`.
    - Click `Start dry sync`.
    - Expected state: `Dry sync complete`.
    - Expected generated objects: greater than `0`.
    - Expected errors: `0`.
    - Screenshot checkpoint: capture dry sync summary.

15. Preview projected records.
    - Open `Preview`.
    - Search Account id `001ACME0000001`.
    - Confirm object type: `Account`.
    - Confirm lineage source: `Salesforce Account`.
    - Confirm related Opportunities tab shows expected rows.
    - Search Opportunity id `006ACME0000420`.
    - Confirm `weighted_pipeline` has lineage inputs.
    - Confirm hidden fields are not visible.
    - Screenshot checkpoint: capture Account and Opportunity previews.
    - Export preview as `salesforce-revops-preview.json`.

16. Materialize approved records.
    - Click `Materialize`.
    - Scope: `Dry sync approved sample`.
    - Max records: `500`.
    - Idempotency key: `sf-revops-materialize-2026-05-20`.
    - Audit label: `tutorial-salesforce-projection`.
    - Click `Start materialization`.
    - Expected state: `Materialization complete`.
    - Expected errors: `0`.
    - Expected audit event: `OntologyProjectionMaterialized`.
    - Screenshot checkpoint: capture materialization summary.
    - Do not enable continuous sync yet.

17. Test rollback preview.
    - Open `Rollback`.
    - Select materialization batch `sf-revops-materialize-2026-05-20`.
    - Choose `Preview rollback only`.
    - Click `Preview`.
    - Expected affected records: same as materialized count.
    - Expected source writes: `0`.
    - Expected audit event: `OntologyProjectionRollbackPreviewed`.
    - Screenshot checkpoint: capture rollback preview.
    - Do not execute rollback in the happy path.
    - The preview proves reversibility without mutating state.

18. Run final verification query.
    - Open `Ontology -> Saved checks`.
    - Choose `tutorial.ontology_salesforce_projection_status`.
    - Input `tenant_id=tenant-acme-robotics`.
    - Input `projection_id=projection:salesforce-revops-pipeline:v1`.
    - Input `source_id=source-salesforce-acme-dev`.
    - Click `Run`.
    - Expected title: `Salesforce ontology projection complete`.
    - Expected state: `PASS`.
    - Screenshot checkpoint: capture query output.
    - Save the output next to `salesforce-revops-preview.json`.
    - The tutorial is complete when validation, materialization, and lineage pass.

## Verification

- Named query: `tutorial.ontology_salesforce_projection_status`.
- Query location: `Ontology -> Saved checks`.
- Query input `tenant_id`: `tenant-acme-robotics`.
- Query input `projection_id`: `projection:salesforce-revops-pipeline:v1`.
- Query input `source_id`: `source-salesforce-acme-dev`.
- Expected output field: `source_connection`.
- Expected output value: `connected`.
- Expected output field: `mapped_objects`.
- Expected output value: `Account,Contact,Opportunity`.
- Expected output field: `schema_pins_present`.
- Expected output value: `true`.
- Expected output field: `lineage_policy`.
- Expected output value: `lineage-policy:salesforce-revops:v1`.
- Expected output field: `role_projection`.
- Expected output value: `role-projection:revops-analyst:v1`.
- Expected output field: `cedar_preview_permit`.
- Expected output value: `true`.
- Expected output field: `cedar_source_write_deny`.
- Expected output value: `true`.
- Expected output field: `dry_sync_errors`.
- Expected output value: `0`.
- Expected output field: `materialization_errors`.
- Expected output value: `0`.
- Expected output field: `source_writes`.
- Expected output value: `0`.
- Expected output field: `rollback_preview_available`.
- Expected output value: `true`.
- Expected output field: `result_label`.
- Expected output value: `Salesforce ontology projection complete`.
- CLI equivalent:

```bash
oya ontology verify projection \
  --tenant tenant-acme-robotics \
  --projection projection:salesforce-revops-pipeline:v1 \
  --source source-salesforce-acme-dev
```

- CLI expected line: `PASS tutorial.ontology_salesforce_projection_status`.
- CLI expected line: `mapped_objects=Account,Contact,Opportunity`.
- CLI expected line: `schema_pins_present=true`.
- CLI expected line: `source_writes=0`.
- Audit event to inspect: `ExternalSourceRegistered`.
- Audit event to inspect: `OntologyProjectionCreated`.
- Audit event to inspect: `OntologyProjectionValidated`.
- Audit event to inspect: `OntologyProjectionMaterialized`.
- Audit event to inspect: `OntologyProjectionRollbackPreviewed`.
- Evidence artifact: `salesforce-revops-preview.json`.
- Evidence artifact: `tutorial-salesforce-projection-materialization-summary.pdf`.
- Dashboard: `Ontology -> Projection health`.
- Expected tile: `No projection drift`.

## Common Pitfalls + Recovery

- Pitfall: the Salesforce credential is pasted into the projection.
- Recovery: remove it and use `openbao://tenant-acme-robotics/connect/salesforce/dev`.
- Pitfall: Account, Contact, and Opportunity map to custom duplicate object types.
- Recovery: map to `ontology://crm/sales#Account`, `Contact`, and `Opportunity`.
- Pitfall: schema pins are left blank.
- Recovery: pin all object, relation, and function revisions before dry sync.
- Pitfall: lineage is disabled for derived fields.
- Recovery: require lineage inputs for `weighted_pipeline`.
- Pitfall: role projection exposes personal contact fields.
- Recovery: hide `Contact.PersonalEmail` and `Contact.HomePhone`.
- Pitfall: Cedar permits `source.write`.
- Recovery: fix policy so projection is read/materialize only and does not write Salesforce.
- Pitfall: continuous sync is enabled too early.
- Recovery: disable continuous sync; this tutorial only materializes the approved sample.
- Pitfall: dry sync returns deleted Salesforce rows.
- Recovery: filter `IsDeleted=false`.
- Pitfall: the tenant id comes from Salesforce.
- Recovery: set tenant field static to `tenant-acme-robotics`.
- Pitfall: cross-tenant relations appear.
- Recovery: block materialization and inspect relation joins.
- Pitfall: materialization lacks idempotency key.
- Recovery: rerun with `sf-revops-materialize-2026-05-20`.
- Pitfall: rollback preview would write to Salesforce.
- Recovery: stop; rollback should affect Oyatie materialized projection state only.
- Pitfall: preview shows hidden phone fields.
- Recovery: repair field visibility and rerun projection validation.
- Pitfall: source connection test passes but dry sync fails.
- Recovery: inspect object-level Salesforce permissions for Account, Contact, and Opportunity.
- Pitfall: projection creates a product service fork.
- Recovery: keep `sales-cloud-core` as a capability tier over ontology, connect, policy, and workflow.
- Pitfall: audit events lack source id.
- Recovery: rerun materialization after fixing audit profile.
- Pitfall: the query reports `source_writes=1`.
- Recovery: disable the projection and investigate immediately.
- Pitfall: sample data contains real sensitive personal data.
- Recovery: use a sandbox fixture or redact before projection.

## Projection Acceptance Checklist

Accept the Salesforce projection only when these values are visible.

- Source connection id should be `source-salesforce-acme-dev`.
- Projection id should be `projection:salesforce-revops-pipeline:v1`.
- Tenant id should be fixed as `tenant-acme-robotics`.
- Source object `Account` should map to ontology object `Organization`.
- Source object `Contact` should map to ontology object `Person`.
- Source object `Opportunity` should map to ontology object `DealSet`.
- Materialization key should be `sf-revops-materialize-2026-05-20`.
- Dry run should show `source_writes=0`.
- Dry run should show `cross_tenant_relations=0`.
- Dry run should show `hidden_fields_projected=0`.
- Materialized sample count should match the approved fixture count.
- Rollback preview should affect Oyatie projection state only.

The projection is not a data migration victory unless rollback is safe.

The projection is not a CRM replacement.

The projection should make Salesforce data usable inside Oyatie ontology, workflow, and policy surfaces.

Keep continuous sync disabled until a separate production runbook approves it.

## Next Tutorials

- [Use intelligence to summarize a 200-page contract](ai-assisted-document-summarization.md).
- [List, sell, buy, and settle a marketplace deal](marketplace-list-sell-buy.md).
- [Activate HIPAA, SOC 2, GDPR, and KR-PIPA](multi-pack-tenant-activation.md).
- [Build an employee-onboarding workflow](workflow-studio-build-employee-onboarding.md).

## References

- [Ontology Projection Substrate Standard](../standards/ontology-projection-substrate.md).
- [Ontology projection schema](../../specs/ontology-projection-schema.json).
- [Ontology microservice spec](../../specs/microservices/ontology.json).
- [Ontology object type versioning ADR](../decisions/ADR-0257-ontology-object-type-versioning-deprecation-handshake.md).
- [Workflow ontology ecosystem adapter ADR](../decisions/ADR-0059-workflow-ontology-ecosystem-adapter-layer.md).
- [Capability Tier Over Product Fragmentation ADR](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md).
- [Tenant-to-tenant data sharing through ontology projection journey](../user-journeys/j118-tenant-to-tenant-data-sharing-via-ontology-projection/README.md).
- [Documentation Rigor](../standards/documentation-rigor.md).
- [Doc Style](../standards/doc-style.md).
