---
doc_class: MigrationPlaybook
microservice: forms
vendor: Google Forms + Typeform + Jotform + SurveyMonkey + Formstack + Tally + Microsoft Forms
date: 2026-05-20
doc_status: published
---

# Migration playbook — Google Forms / Typeform / Jotform / SurveyMonkey / Formstack / Tally / Microsoft Forms → oyatie forms

Audience: an oyatie tenant migrating their form-collection substrate from Google Forms, Typeform, Jotform, SurveyMonkey, Formstack, Tally, or Microsoft Forms to oyatie's `forms` µservice.

## Why this migration is non-trivial

- **Form definitions** port cleanly via API/export.
- **Submission archives** can be large (millions of rows); migration is bandwidth-bound.
- **Integrations** (webhooks, Zapier, payment gateways) need re-author.
- **Embedded forms** on tenant websites need URL updates.

The 80/20: form definitions + recent submissions port cleanly; the 20 % needing care is webhook integrations + historical-submission migration + warehouse-export reconfiguration.

## Step 1 — Inventory the source (≤ 1-2 weeks per provider)

For Google Forms:

```sh
oya forms migrate inventory \
    --source google-forms \
    --google-workspace-id "$WORKSPACE_ID" \
    --service-account-json ./service-account.json \
    --out inventory/google-forms.yaml
```

Captures: forms, questions, responses, response-spreadsheet links, owners, sharing.

For Typeform:

```sh
oya forms migrate inventory \
    --source typeform \
    --typeform-pat "$TYPEFORM_PAT" \
    --out inventory/typeform.yaml
```

Captures: forms, questions, logic-jumps, themes, responses, webhooks, integrations.

For Jotform:

```sh
oya forms migrate inventory \
    --source jotform \
    --jotform-api-key "$JOTFORM_API_KEY" \
    --out inventory/jotform.yaml
```

For SurveyMonkey:

```sh
oya forms migrate inventory \
    --source surveymonkey \
    --surveymonkey-access-token "$SURVEYMONKEY_TOKEN" \
    --out inventory/surveymonkey.yaml
```

For Formstack:

```sh
oya forms migrate inventory \
    --source formstack \
    --formstack-pat "$FORMSTACK_PAT" \
    --out inventory/formstack.yaml
```

For Tally:

```sh
oya forms migrate inventory \
    --source tally \
    --tally-pat "$TALLY_PAT" \
    --out inventory/tally.yaml
```

For Microsoft Forms:

```sh
oya forms migrate inventory \
    --source microsoft-forms \
    --tenant-id "$M365_TENANT_ID" \
    --graph-token "$GRAPH_TOKEN" \
    --out inventory/microsoft-forms.yaml
```

## Step 2 — Audit mapping (≤ 1 week)

```sh
oya forms migrate audit \
    --inventory inventory/typeform.yaml \
    --source-platform typeform \
    --out audit/typeform-mapping.yaml
```

| Source concept | oyatie equivalent | Risk |
|---|---|---|
| Form | Form | Direct |
| Question (text, multiple-choice, rating, etc.) | Field | Direct (per type mapping) |
| Logic Jump | Logic-jump | Direct (Typeform is closest in semantics) |
| Theme / branding | Form theme | Direct (best-match) |
| Response | Submission | Direct |
| Webhook | Webhook | Direct |
| Hidden field (URL params) | Hidden field | Direct |
| Calculator | Calculation field | Direct (paid) |
| Payment field | Payment field (Stripe / Adyen / PayPal) | Direct |
| File upload | File upload | Direct |
| Welcome screen | First page | Direct |
| End screen | Last page | Direct |
| Notifications (email, Slack) | Notifications | Direct |
| Integrations (Zapier, Make, Salesforce) | Re-author per oyatie | Manual |
| Analytics | Form analytics | Direct (different surface) |
| Custom CSS / theme | Custom theme | Direct (best-match) |
| Embed code | Embed code | Direct |

For Google Forms:

| Google Forms concept | oyatie equivalent |
|---|---|
| Form | Form |
| Question | Field |
| Section break | Page break |
| Required toggle | Required toggle |
| Response | Submission |
| Linked Google Sheets | Warehouse-export to sheets µservice |
| Quiz mode (with answer key) | Quiz feature (paid) |
| File upload (with Google Drive integration) | File upload (to oyatie drive) |
| Theme customization | Theme |

For SurveyMonkey / Formstack / Jotform / Tally / Microsoft Forms: similar mapping.

## Step 3 — Convert form definitions (≤ 2-4 weeks)

```sh
oya forms migrate convert \
    --source typeform \
    --inventory inventory/typeform.yaml \
    --output-dir ./migration-staging/typeform/ \
    --target-tenant drill-acme \
    --concurrency 4
```

For each form:

1. Parse fields + logic-jumps.
2. Map field types per the audit table.
3. Apply tenant theme.
4. Set up warehouse-export per tenant config.
5. Configure captcha.
6. Map webhook endpoints.

Re-author Zapier / Make integrations: tenant identifies critical integrations + re-creates with oyatie webhook + workflow-engine.

## Step 4 — Migrate historical submissions (≤ 1-2 weeks)

```sh
oya forms migrate submissions \
    --source typeform \
    --tenant drill-acme \
    --form-id form-789xyz \
    --target-oyatie-form drill-acme-2026-feedback \
    --window 2023-01-01..2026-05-20 \
    --concurrency 4
```

For each submission:

1. Fetch from source.
2. Map field-IDs to oyatie field-names.
3. Insert into submission-store.
4. Apply timestamp + creator from source.
5. Skip duplicates (idempotent).

Throughput: ~ 100 submissions/sec per worker; for 1M submissions: ~ 3 hours sequential or ~ 45 min at concurrency=4.

Warehouse-export catches up after submission import.

## Step 5 — URL + embed-code updates (≤ 1-2 weeks)

Tenant updates:

- Website embeds: change iframe src to new oyatie URL.
- Email links: update for new form URL.
- QR codes: regenerate.
- Cross-references (Slack, Notion, Discord): update.

We provide a redirect:

```sh
oya forms migrate redirect-create \
    --tenant drill-acme \
    --source typeform \
    --source-form-id form-789xyz \
    --target oyatie-form-id drill-acme-2026-feedback \
    --validity 90d
```

Old typeform URL redirects to oyatie URL for 90 days; gives tenant time to update embeds.

## Step 6 — Test + cutover (≤ 2-4 weeks)

Wave-based:

- Week 1: 5 % of forms migrated; tenant tests.
- Week 2: 25 %.
- Week 3: 50 %.
- Week 4: 100 %.

For each form:

- Submit test entries; verify warehouse-export.
- Test payment if applicable.
- Test webhook delivery.
- Verify Cedar + consent fields.

## Step 7 — Decommission source (≤ 1 month)

```sh
oya forms migrate decommission \
    --tenant drill-acme \
    --source typeform \
    --evidence-out evidence/migrations/typeform-to-oyatie-drill-acme.json
```

After 90-day redirect validity, source forms can be cancelled.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Logic-jump semantics differ between platforms | Medium | Per-form review; test each branching path |
| Custom theme / CSS not portable 1:1 | Medium | Apply best-match; tenant adjusts |
| Embedded forms on third-party sites | High | Redirect for 90 days; tenant updates embeds |
| Zapier / Make integrations | High | Re-author per oyatie webhook + workflow |
| Payment provider account switching | High | Tenant re-configures Stripe / Adyen; test thoroughly |
| Historical submissions too large for one-time migration | High | Multi-batch migration; warehouse-export catches up |
| Multi-tenant Google Workspace (form ownership shared) | Medium | Per-form decide ownership; consolidate during migration |
| Typeform Pro features (Logic + Calculator) | Medium | Direct mappable; tested per form |
| SurveyMonkey survey-paths (their proprietary flow) | High | Re-author as oyatie logic-jumps; test each path |
| Microsoft Forms with embedded Power Automate | High | Re-author per oyatie webhook + workflow-engine |
| Form analytics + dashboards reset | Low | Documented; tenant configures new dashboards |
| HIPAA forms (Patient intake) | Critical | Validate pack-bound at paid tenant_class with compliance_pack before migration |
