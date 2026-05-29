---
doc_class: MigrationPlaybook
microservice: consent-graph
vendor: OneTrust + TrustArc (privacy / consent management platforms)
date: 2026-05-20
doc_status: published
---

# Migration playbook — OneTrust + TrustArc → oyatie consent-graph

Audience: an oyatie tenant migrating consent + agreement + data-subject-rights workflows from OneTrust DataGuide or TrustArc Privacy Management Platform to oyatie's `consent-graph` µservice.

## Why this migration is non-trivial

OneTrust and TrustArc are PRIVACY MANAGEMENT PLATFORMS — they're workflow-tool oriented. oyatie consent-graph is a REAL-TIME ENFORCEMENT substrate. The mismatch:

- OneTrust/TrustArc store consent records + DSAR workflows + cookie-banner configurations.
- consent-graph stores DataSharingAgreement records + the bilateral chain + the cross-tenant projection enforcement.

The migration approach:

1. Import OneTrust/TrustArc CONSENT RECORDS (data-subject-level consent) into consent-graph's per-tenant consent registry.
2. Re-author OneTrust/TrustArc B2B partner AGREEMENTS as consent-graph DataSharingAgreements.
3. Re-build OneTrust/TrustArc DSAR WORKFLOWS in workflow-engine.
4. Keep OneTrust/TrustArc for the workflow-tool features (cookie-banner UI, in-app consent prompts) IF the tenant doesn't move to a hosted equivalent yet.

The cleanest migration path is parallel: tenants run BOTH for a transition period; consent-graph becomes the enforcement source-of-truth; OneTrust/TrustArc remains the workflow-tool until the tenant builds equivalent oyatie workflows.

## Step 1 — Export consent records from source (≤ 1-3 days)

OneTrust:

```sh
oya consent-graph migrate inventory \
    --source onetrust \
    --onetrust-tenant-id "$ONETRUST_TENANT_ID" \
    --onetrust-api-key "$ONETRUST_API_KEY" \
    --export-classes consent-records,b2b-agreements,dsar-workflows,cookie-banner-configs \
    --window 2020-01-01..2026-05-20 \
    --out inventory/onetrust.yaml
```

TrustArc:

```sh
oya consent-graph migrate inventory \
    --source trustarc \
    --trustarc-org "$TRUSTARC_ORG" \
    --trustarc-api-token "$TRUSTARC_API_TOKEN" \
    --export-classes consent-records,b2b-agreements,dsar-workflows \
    --out inventory/trustarc.yaml
```

## Step 2 — Re-author B2B agreements as DataSharingAgreements (≤ 2-8 weeks)

For each B2B partner relationship in the OneTrust/TrustArc export, draft an oyatie agreement.

```sh
oya consent-graph migrate b2b-agreement \
    --source-agreement-id ot-bvp-acme-partner-x \
    --source onetrust \
    --target-grantor drill-acme \
    --target-grantee drill-partner-x \
    --convert-scope-spec true \
    --convert-predicate-narrowing true \
    --output draft-converted-agreements/ot-bvp-acme-partner-x.json
```

The converter maps:

| OneTrust / TrustArc field | oyatie DataSharingAgreement field | Notes |
|---|---|---|
| "Vendor / Partner Name" | `grantee` | Direct |
| "Data Classification" | scope spec entity-types | Map to Ontology entity-types |
| "Data Fields" | field_narrowing.include | Direct |
| "Processing Purposes" | purpose_specification | Direct |
| "Retention Period" | retention_at_grantee | Direct |
| "International Transfer Allowed" | geographic_constraint | If allowed → `none`; else → `same-region` / `eu-eea-only` etc. |
| "Sub-processor Allowed" | sub_processor_chain_allowed | Direct |
| "Right to Audit Specified" | audit_clause_present | Boolean |

Manual review required for:

- Complex multi-purpose agreements (one B2B partner uses data for 5 purposes; oyatie agreements typically one purpose per agreement; split into multiple).
- Sub-processor chains (oyatie tracks first-hop; sub-processors require separate agreements with each sub-processor).
- Agreements expired but kept for audit (`archived` state; don't re-activate).

## Step 3 — Re-build DSAR workflows in workflow-engine (≤ 1-4 weeks)

OneTrust/TrustArc DSAR workflows have specific shapes (intake, ID verification, search, redact, fulfill). Each becomes an oyatie workflow-engine workflow.

```sh
oya workflow-engine import \
    --source onetrust \
    --source-workflow-id ot-dsar-art-15-access \
    --target-tenant drill-acme \
    --output workflow-definitions/dsar-art-15-access.yaml
```

The workflow engine handles the actual fulfillment; consent-graph handles the cross-tenant data-share aspect (notifying grantees when a DSAR-bound data subject's data must be deleted on the grantee side).

## Step 4 — Cookie-banner integration (optional; if tenant doesn't have alternative)

OneTrust + TrustArc both ship cookie-banner UI. If the tenant doesn't have an alternative (e.g., a homegrown banner), KEEP OneTrust/TrustArc for the banner; have it write consent records to a webhook that lands in oyatie consent-graph.

oyatie does NOT (as of 2026-05) ship a tenant-facing cookie-banner; consent-graph is enforcement-only. If/when oyatie ships a banner, migrate the banner config separately.

## Step 5 — Cutover (≤ 4-8 weeks)

- Day 0-14: oyatie consent-graph runs in shadow; receives the same consent events that OneTrust receives.
- Day 14-28: cross-tenant data sharing migrates to oyatie agreement-based path; OneTrust agreements become read-only.
- Day 28-42: DSAR workflows cut over to workflow-engine; OneTrust DSAR workflows are decommissioned.
- Day 42+: cookie-banner stays at OneTrust (until oyatie ships its own).

Monitor:

```sh
oya consent-graph migrate cutover-status \
    --tenant drill-acme \
    --source onetrust
```

## Step 6 — Decommission

```sh
oya consent-graph migrate decommission \
    --tenant drill-acme \
    --source onetrust \
    --keep-active-for cookie-banner \
    --evidence-out evidence/migrations/onetrust-to-oyatie-drill-acme.json
```

The `--keep-active-for cookie-banner` keeps OneTrust active just for the banner; everything else is on oyatie.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| OneTrust DSAR workflows are complex; re-build takes weeks | High | Budget 1-2 weeks per critical DSAR workflow; start with Art. 15 + Art. 17 |
| Cookie-banner is regulator-visible (DPC inspections) | Critical | Don't disrupt the banner during migration; switch later when oyatie ships equivalent |
| B2B agreement-to-DataSharingAgreement mapping is lossy | Medium | Manual review per agreement; do not bulk-convert |
| Sub-processor chains don't map 1:1 | High | Identify sub-processors per agreement; create separate consent-graph agreements with each |
| Consent records from before 2018 (pre-GDPR) may not have lawful basis | Critical | Audit per Step 1; flag pre-GDPR records for legal review |
| Cookie-consent vs B2B-consent confusion | Medium | Separate the two during inventory; cookie is per-data-subject; B2B is per-partner |
| Tenant loses access to their consent records during cutover | High | Maintain both systems for 60+ days; do not delete OneTrust records until oyatie evidence verified |
