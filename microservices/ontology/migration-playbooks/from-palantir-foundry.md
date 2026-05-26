---
doc_class: MigrationPlaybook
microservice: ontology
vendor: Palantir Foundry Ontology
date: 2026-05-20
doc_status: published
---

# Migration playbook — Palantir Foundry Ontology → oyatie ontology

Audience: a team using Palantir Foundry's Ontology surface (the Ontology Object Type Manager + Pipeline Builder + Workshop ecosystem) who wants to move to oyatie's `ontology` µservice. Drivers: cost reduction (Foundry pricing scales aggressively), per-tenant SaaS-shape isolation, audit-chain non-repudiation, integration with the rest of oyatie's substrate.

## Why this migration matters

Palantir Foundry is excellent at:

- The end-to-end product surface (Ontology + Pipeline Builder + Workshop + Actions).
- Government + defence-grade compliance posture.
- ML + analytical workloads at scale (Foundry's Compute Modules).

oyatie ontology offers:

- Lower per-entity cost at our envelope.
- Per-tenant Cedar gates (Foundry has its own security model; not Cedar-compatible).
- Audit-chain non-repudiation (Foundry audit is separate; not cryptographic).
- Multi-tenant SaaS shape (Foundry is per-organization).

Caveat: oyatie does NOT replicate Foundry's full surface — Pipeline Builder is `workflow-engine`+`workflow-studio` (separate); Workshop is the tenant's own application layer; Actions is workflow-engine actions. The migration is multi-µservice.

## Step 1 — Inventory the Foundry estate (≤ 2-3 weeks)

```bash
# From Foundry admin (via Foundry API):
foundry-cli ontology object-types list --output ./foundry-object-types.json
foundry-cli ontology link-types list --output ./foundry-link-types.json
foundry-cli ontology actions list --output ./foundry-actions.json
foundry-cli ontology object-set list --output ./foundry-object-sets.json
foundry-cli pipeline list --output ./governance-pipelines.json
foundry-cli workshop list --output ./foundry-workshops.json
```

Document:

- Object types (= oyatie entity types). Property counts + types.
- Link types (= oyatie relationship types). Cardinality.
- Object set definitions (= oyatie ad-hoc queries).
- Actions (= oyatie workflow-engine workflows).
- Pipeline definitions (= oyatie workflow-engine or analytics MVs).
- Workshop applications (= tenant-built UI; require tenant-side rewrite).

Typical mid-size Foundry install: 40-200 object types, 100-500 link types, 50-200 actions, 20-100 pipelines, 10-30 workshops.

## Step 2 — Schema translation (≤ 4-8 weeks)

```sh
oya ontology migrate convert-foundry \
    --input ./foundry-object-types.json,./foundry-link-types.json \
    --output ./oyatie-schema.yaml \
    --tenant acme-corp
```

The translator:

- Maps Foundry object types → oyatie entity types.
- Maps Foundry link types → oyatie relationship types.
- Maps Foundry property types (string, integer, date, decimal, boolean, array, struct) → oyatie property types.
- Maps Foundry's "primary key" → oyatie's `primary: true`.
- Maps Foundry's "title key" → an oyatie computed display field.

Manual review required for:

- Foundry struct property types (nested objects) → oyatie typically flattens or models as a child entity.
- Foundry's "geo" property type → oyatie uses PostGIS-backed `geo` for paid tenant_class.
- Foundry's "media reference" property type → oyatie typically uses an S3-uri property.
- Foundry's link-type-property semantics differ slightly from oyatie's relationship-properties.

## Step 3 — Data migration (≤ 2-6 weeks per 100M entities)

```sh
oya ontology migrate import-foundry-data \
    --tenant acme-corp \
    --foundry-export-uri "foundry://datasets/ontology-snapshot-2026-05-20" \
    --schema ./oyatie-schema.yaml \
    --throttle-rate 50000-entities-per-sec
```

The import:

- Reads Foundry's exported Parquet from Foundry Datasets.
- Validates each entity against the registered schema.
- Bulk-inserts to PostgreSQL with Citus distribution.
- Creates the relationship edges.

Throttle keeps the import within the per-tenant quota; 50k entities/sec on paid tenant_class is ~ 4 hours per 100M entities.

Verify counts post-import:

```sql
SELECT entity_type, count(*)
FROM tenant_acme_corp.entity
GROUP BY entity_type
ORDER BY entity_type;
```

Cross-check against Foundry's per-object-type counts. Acceptable drift: 0 % (entity-level integrity must match).

## Step 4 — Migrate Actions → workflow-engine workflows (≤ 4-12 weeks)

Foundry Actions are workflows that mutate the ontology + dispatch side-effects. They become oyatie `workflow-engine` workflows.

Example Foundry Action ("Approve Order"):

```typescript
// Foundry Action TypeScript
action ApproveOrder {
  parameter orderId: ObjectReference<Order>;
  effects {
    modifyObject(orderId, { status: 'approved' });
    sendEmail(orderId.customerId, 'Order approved');
  }
}
```

oyatie workflow equivalent:

```yaml
workflow_id: approve-order
version: 1
inputs:
  order_id: { type: string, required: true }
steps:
  - id: update_order_status
    handler: ontology.entity.update
    inputs:
      entity_type: Order
      entity_id: "{{order_id}}"
      properties: {status: "approved"}
  - id: send_email
    handler: mail.send_template
    inputs:
      template_id: order_approved
      recipient_resolver: "{{update_order_status.entity.customer_id}}"
```

Translation tooling:

```sh
oya workflow-engine migrate convert-foundry-actions \
    --input ./foundry-actions.json \
    --output-dir ./oyatie-workflows/ \
    --tenant acme-corp
```

~ 70 % auto-translate; remainder needs manual review (Foundry's effect predicates are richer than oyatie's step handlers).

## Step 5 — Workshop → tenant-application rewrite (≤ 8-26 weeks)

Foundry Workshops are React-based applications built within Foundry. They consume Ontology objects + dispatch Actions.

oyatie doesn't provide a Workshop equivalent. The tenant must:

1. Build their own application UI (React / Vue / Angular).
2. Consume oyatie ontology via REST / gRPC / GraphQL.
3. Dispatch oyatie workflow-engine workflows for actions.

For tenants who want a low-code application builder, recommend `workflow-studio` for the workflow layer + a tenant-chosen front-end framework (or low-code tool like Retool, Tooljet, Appsmith) for the UI.

This is the largest migration effort. Plan 8-26 weeks per Workshop application depending on complexity.

## Step 6 — Pipeline Builder → workflow-engine or analytics MVs (≤ 4-12 weeks)

Foundry Pipelines are dataflow graphs (Spark-backed). Two paths:

1. **Workflow-engine pattern**: for event-driven pipelines (e.g., "when an Order is updated, recompute the Customer LTV"). Translate to a workflow with a Pulsar-trigger.
2. **Analytics MV pattern**: for batch-rollup pipelines (e.g., "daily revenue rollup by region"). Translate to a ClickHouse Materialized View in the `analytics` µservice.

## Step 7 — Cutover (≤ 1 d post-readiness)

After all data + Actions + UIs + Pipelines have migrated:

```sh
# Flip the tenant's ontology provider
oya governance set-config \
    --tenant acme-corp \
    --key default_ontology_provider \
    --value oyatie

# Audit-emit
oya audit emit \
    --tenant acme-corp \
    --event-class governance.ontology_substrate.cut_over \
    --payload '{"from":"palantir-foundry","to":"oyatie","cutover_at":"2026-05-20T14:00:00Z"}'
```

Existing Foundry Workshops stop being the source of truth; oyatie becomes canonical.

## Step 8 — Foundry decommission (≤ 90-180 d post-cutover)

Foundry stays read-only for historical access. After ≥ 90 d:

- Export final Foundry state for archival.
- Decommission the Foundry tenant per Palantir contract notice.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Foundry's full surface (Workshop, Pipeline Builder) doesn't 1:1 migrate | Critical | Scope the migration: ontology + actions only; rebuild Workshop UIs separately |
| Foundry struct + media + geo property types need re-modeling | High | Pre-audit; plan per-type re-modeling |
| Object set queries don't translate 1:1 (Foundry's query DSL) | Medium | Rewrite ad-hoc queries in Cypher / GraphQL |
| Foundry's row-level security model doesn't map to Cedar 1:1 | High | Re-author Cedar policies; pre-validate sample queries |
| Workshop application rewrites take longer than estimated | Critical | Plan 8-26 weeks per Workshop; sequence highest-value first |
| Pipeline Builder dataflow semantics differ from oyatie | High | Translate per pipeline; some require redesign |
| Per-tenant Foundry licence requires negotiation | Medium | Engage Palantir account manager for contract amendment |
| Auditors familiar with Foundry need re-training on oyatie | Medium | Provide oyatie audit-chain training; produce parity-table |
| Foundry's Government / Defence customers expect parity | Critical | Validate sovereign-pack feature parity before migration |
| Data export from Foundry takes longer than expected | Medium | Schedule export over weekends; budget 3× expected time |
| Foundry-specific UI tooling (Quiver, Slate) has no oyatie equivalent | High | Tenant-built UI; recommend Retool / Appsmith for low-code |
| OAuth + SSO integration differs | Low | Re-configure via tenant IdP; standard SAML / OIDC |
