---
doc_class: ImplementationPlan
ip_id: IP-033-form
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0245, ADR-0251, ADR-0253-amendment, ADR-0263, ADR-0321, ADR-0328]
bounded_context: form
journey_id: J-MA-33-marketing-form-capture
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
boundary_settled_by: ADR-MS-MA-003
---

# IP-033: Marketing Form

## Context

Sam Okafor (B2C marketing engineer) needs to capture leads from embedded forms, pop-up forms, and JS-SDK-collected forms on third-party sites. HubSpot Forms includes a "collected forms" variant where a JS SDK captures third-party form submissions; this slice replicates that with signed-token submission + GDPR consent block + progressive profiling. The boundary with the forms µservice is settled by ADR-MS-MA-003: marketing-automation owns the marketing-context form (field config + consent block + post-submit cascade); forms substrate hosts the underlying form-rendering primitive.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_form` | `form_id` | `uuid primary key` | Oyatie form id. |
| `marketing_form` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_form` | `form_kind` | `text not null` | regular_embed / popup / collected / chatflow_form. |
| `marketing_form` | `field_config` | `jsonb not null` | Array of typed fields (text, email, dropdown, multi-select, consent_checkbox). |
| `marketing_form` | `validation_rules` | `jsonb not null` | Per-field rules (required, regex, length, custom). |
| `marketing_form` | `conditional_logic_dag` | `jsonb not null` | Show/hide fields based on prior answers. |
| `marketing_form` | `progressive_profiling_rules` | `jsonb` | Skip already-known fields. |
| `marketing_form` | `gdpr_consent_block` | `jsonb not null` | Per pack overlay; resolves at render. |
| `marketing_form` | `captcha_provider` | `text` | reCAPTCHA / hCaptcha / none (per pack). |
| `marketing_form` | `post_submit_redirect_url` | `text` | Validated against tenant allow-list. |
| `marketing_form` | `post_submit_notification_emails` | `text[]` | Operator notification destinations. |
| `marketing_form` | `published_at_hlc` | `hlc` | Immutable timestamp. |
| `marketing_form_submission` | `submission_id` | `uuid primary key` | Submission event id. |
| `marketing_form_submission` | `form_id` | `uuid not null` | FK. |
| `marketing_form_submission` | `subject_hash` | `text not null` | Hashed subject ref. |
| `marketing_form_submission` | `field_values` | `jsonb not null` | Submitted field values (data-boundary labeled). |
| `marketing_form_submission` | `idempotency_key` | `text not null unique` | Defends against double submission. |
| `marketing_form_submission` | `submitted_at_hlc` | `hlc not null` | HLC stamp. |
| `marketing_form_submission` | `signed_token_verified` | `boolean not null` | True for collected variant. |

## API Endpoints

REST `POST /v1/marketing-automation/forms`:

```json
{
  "tenant_id": "...",
  "form_kind": "regular_embed",
  "field_config": [
    {"name": "first_name", "type": "text", "required": true},
    {"name": "email", "type": "email", "required": true},
    {"name": "company_size", "type": "dropdown", "options": ["1-10","11-50","51-200","201+"]},
    {"name": "marketing_consent", "type": "consent_checkbox", "required_for_pack": ["GDPR","CASL"]}
  ],
  "validation_rules": {"email": {"regex": "^[^@]+@[^@]+$"}},
  "conditional_logic_dag": {"company_size": [{"if": "201+", "show": ["use_case"]}]},
  "gdpr_consent_block": {"text": "I agree to receive marketing communications. See our privacy policy.", "version": "2026-05-01"},
  "post_submit_redirect_url": "/thank-you"
}
```

REST `POST /v1/marketing-automation/forms/{form_id}:submit` accepts public submissions:

```json
{
  "field_values": {"first_name": "Alice", "email": "alice@acme.io", "marketing_consent": true},
  "idempotency_key": "form-sub-2026-05-21-abc123",
  "signed_token": "..."
}
```

REST `GET /forms/embed/{form_id}.js` serves the embeddable JS for `form_kind == 'regular_embed'` or `'collected'`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::PublishForm` | `MarketingForm::form_id` | `tenant_class`, `active_forms_count` |
| `Service::"form-receiver"` | `marketingAutomation::RecordFormSubmission` | `MarketingForm::form_id` | `signed_token_verified`, `captcha_ok`, `idempotency_key_new`, `pack_overlay` |

Demo-trial gate: `tenant_class == 'demo_trial' && active_forms_count >= 5` denies publish.

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| HubSpot Form | `MarketingForm.form_kind == 'regular_embed'` | HubSpot field types map 1:1 to Oyatie field types. |
| HubSpot Pop-up Form | `MarketingForm.form_kind == 'popup'` | Trigger rules become `triggers` jsonb. |
| HubSpot Collected Form | `MarketingForm.form_kind == 'collected'` | JS SDK signed-token submission preserved. |
| HubSpot Chatflow as Form | `MarketingForm.form_kind == 'chatflow_form'` | Chatflow step bindings become field collectors. |
| Marketo Form | `MarketingForm.form_kind == 'regular_embed'` | Marketo Form Filling Out trigger preserved as `triggers`. |
| Mailchimp Signup Form | `MarketingForm.form_kind == 'regular_embed'` | Mailchimp Merge Fields become form fields. |

## Workflow Steps

1. `ValidateFieldConfig` checks per-field type + per-pack consent_checkbox presence.
2. `CompileValidationRules` validates regex + length syntax.
3. `CompileConditionalLogicDag` ensures DAG (no cycles).
4. `ValidateRedirectAllowlist` checks `post_submit_redirect_url` against tenant allow-list.
5. `AuthorizePublish` calls Cedar.
6. `Publish` transitions status; provisions per-form signed-token secret in OpenBao.
7. On submit, `VerifySignedToken` for collected variant; `VerifyCaptcha` per pack; `EnforceIdempotency`.
8. `RecordSubmission` writes row; emits `EVT-MARKETING-FORM-SUBMITTED`.
9. Post-submit cascade: trigger journey via workflow-canvas; create/update subject via crm contract; send notification email.

Decision branches:
- Required field missing → 422 `field_required`.
- Captcha fail → 403 `captcha_failed`.
- Idempotency replay → return prior result with `x-idempotent-replay: true`.
- Signed-token invalid (collected variant) → 401 `signed_token_invalid`.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-FORM-CREATED` | `tenant_id`, `form_id`, `form_kind`, `tenant_class` |
| `EVT-MARKETING-FORM-PUBLISHED` | `form_id`, `published_at_hlc`, `tenant_class` |
| `EVT-MARKETING-FORM-SUBMITTED` | `form_id`, `submission_id`, `subject_hash`, `idempotency_key`, `pack_overlay`, `consent_block_version` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Publish form | 80 ms | 300 ms | 700 ms | 100 rps/cell | 99.95% |
| Submit form | 50 ms | 200 ms | 500 ms | 3000 rps/cell | 99.99% |
| Render form (JS embed) | 20 ms | 100 ms | 250 ms | 10000 rps/cell | 99.99% |

## Failure Modes + Recovery

- Captcha provider outage → fall back to alternative captcha provider per pack overlay; never silently skip.
- Idempotency key conflict → return original submission result (idempotent replay).
- Signed-token expiry on collected variant → 401 with renewal hint.
- GDPR consent block version mismatch with rendered form → 409 `consent_block_version_stale`; require re-fetch.

## Migration Notes

HubSpot Form export uses HubSpot Forms API; field types are well-documented (text, email, single-line, multi-line, dropdown, radio, checkbox, file, captcha, consent). Import preserves field order + validation rules + conditional logic. Collected-form JS SDK migration requires JavaScript snippet replacement on the tenant's third-party sites — operator-driven.

Marketo Form export uses `MKTOFORMS2`; Marketo Smart List filters that trigger on form submission become `triggers` jsonb on the workflow-canvas side.

Mailchimp Signup Form is simpler; mapping preserves field-by-field.

## Cross-µservice Handoffs

- `forms` substrate hosts rendering + collection per ADR-MS-MA-003.
- `consent-graph` records consent block acceptance evidence.
- `openbao` stores per-form signed-token secret.
- `crm` receives subject create/update on submit.
- `workflow-canvas` triggers journey on submit per workflow-canvas trigger registry.
- `audit-chain` seals every lifecycle event.
- `data-boundary` labels submitted field values per data-class.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-033-form.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-033-form.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].
