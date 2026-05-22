---
doc_class: ImplementationPlan
ip_id: IP-031-email-compose
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0245, ADR-0251, ADR-0253-amendment, ADR-0263, ADR-0321, ADR-0328]
bounded_context: email
journey_id: J-MA-31-marketing-email-composition
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-031: Marketing Email Composition

## Context

Marcus Chen launches product-update emails to a 30k-contact segment. The marketing-automation µservice owns the composition surface (subject + content + dynamic tokens + smart content rules + A/B variants + send-time-optimization binding); the mail µservice owns delivery execution. Without the composition surface, the tenant has to compose in HubSpot Marketing Email or Marketo Email Editor 2.0 and copy artifacts into Oyatie — losing token-resolution evidence, smart-content versioning, accessibility audit history, and A/B variant snapshots. This slice creates the bespoke `marketing_email` aggregate with accessibility-checker pass before publish and immutable post-publish content.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_email` | `email_id` | `uuid primary key` | Oyatie email id. |
| `marketing_email` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_email` | `subject_template` | `text not null` | Mustache-style with ontology-trait tokens. |
| `marketing_email` | `content_blocks` | `jsonb not null` | Array of typed blocks (text, image, button, dynamic-content, A/B-variant). |
| `marketing_email` | `from_name` | `text not null` | Immutable post-publish. |
| `marketing_email` | `from_email` | `text not null` | Validated against tenant verified-sender list. |
| `marketing_email` | `reply_to` | `text` | Optional override of from_email. |
| `marketing_email` | `preview_text` | `text not null` | Inbox preview snippet. |
| `marketing_email` | `accessibility_audit_score` | `numeric(4,2)` | 0.00-100.00; publish requires ≥ 85.00. |
| `marketing_email` | `status` | `text not null` | draft / validated / published / retired. |
| `marketing_email` | `published_at_hlc` | `hlc` | Immutable timestamp. |
| `marketing_email_variant` | `variant_id` | `uuid primary key` | A/B variant id. |
| `marketing_email_variant` | `email_id` | `uuid not null` | FK. |
| `marketing_email_variant` | `allocation_bps` | `int not null` | 0-10000; per-variant sums to 10000. |

## API Endpoints

REST `POST /v1/marketing-automation/emails`:

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-ma000031",
  "subject_template": "{{first_name}}, your {{product.name}} update is here",
  "content_blocks": [
    {"type": "text", "value": "Hi {{first_name}}, here is your monthly update."},
    {"type": "dynamic_content", "rule": "{{lifecycle_stage}} == 'Customer'", "blocks": [{"type": "text", "value": "As a Customer, you get..."}]},
    {"type": "button", "label": "View update", "href": "{{utm_link}}"}
  ],
  "from_name": "Acme Product Team",
  "from_email": "product@acme.io",
  "preview_text": "Your product update for May 2026"
}
```

REST `POST /v1/marketing-automation/emails/{email_id}:validate` runs accessibility audit + token resolution against ontology + dynamic-content rule compilation.

REST `POST /v1/marketing-automation/emails/{email_id}:publish` requires `accessibility_audit_score ≥ 85.00`; transitions status to `published` and seals.

gRPC `MarketingEmailService.Compose` / `.Validate` / `.Publish` mirror REST over HTTP/3.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::ComposeEmail` | `MarketingEmail::*` | `tenant_id`, `tenant_class`, `from_email` |
| `User::"marketing.ops"` | `marketingAutomation::PublishEmail` | `MarketingEmail::email_id` | `accessibility_audit_score`, `tenant_class`, `from_email_verified` |
| `Service::"send-scheduler"` | `marketingAutomation::ScheduleEmailSend` | `MarketingEmail::email_id` | `tenant_class`, `monthly_send_count`, `frequency_cap_reservation`, `deliverability_admit` |

Demo-trial gate: `tenant_class == 'demo_trial' && monthly_send_count >= 5000` denies with `429 demo_trial_cap_hit`.

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| HubSpot Marketing Email | `MarketingEmail` | content blocks become typed `content_blocks` jsonb; HubSpot Smart Content rules become `dynamic_content` block predicates. |
| Marketo Email | `MarketingEmail` | Marketo Tokens become Mustache `{{trait}}` tokens against ontology trait registry. |
| Mailchimp Regular Campaign | `MarketingEmail` | Mailchimp Merge Tags become Mustache tokens. |
| Mailchimp Postcard | (delegated to IP-053 Postcard) | Postcard is a distinct primitive. |

## Workflow Steps

1. `ValidateSenderDomain` confirms `from_email` domain is in tenant's verified-sender list (sourced from mail µservice).
2. `CompileContentBlocks` validates each block against block-type schema.
3. `ResolveTokens` validates every `{{token}}` resolves against ontology trait registry.
4. `CompileDynamicContentRules` validates dynamic-content predicates compile.
5. `RunAccessibilityAudit` checks alt text, color contrast, heading structure, link descriptors; produces score.
6. `AuthorizeCompose` calls Cedar.
7. `PersistDraft` writes `marketing_email` row in `draft` status.
8. On `:publish`, `EnsureAuditScoreFloor` requires score ≥ 85.00.
9. `SealPublish` emits `EVT-MARKETING-EMAIL-PUBLISHED` and transitions status.

Decision branches:
- Unverified sender domain → 422 `sender_domain_unverified`.
- Unknown token → 422 `unknown_token` with ontology trait suggestion.
- Audit score < 85.00 → 422 `accessibility_audit_floor` with remediation hints.
- Demo-trial cap exceeded → 429 `demo_trial_cap_hit`.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-EMAIL-CREATED` | `tenant_id`, `email_id`, `tenant_class`, `cedar_decision_id` |
| `EVT-MARKETING-EMAIL-VALIDATED` | `email_id`, `accessibility_audit_score`, `token_count`, `dynamic_block_count` |
| `EVT-MARKETING-EMAIL-PUBLISHED` | `email_id`, `published_at_hlc`, `tenant_class`, `cedar_decision_id` |
| `EVT-MARKETING-EMAIL-VARIANT-ADDED` | `email_id`, `variant_id`, `allocation_bps` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Compose email | 60 ms | 250 ms | 600 ms | 200 rps/cell | 99.95% (paid) |
| Validate (with accessibility audit) | 150 ms | 800 ms | 2 s | 50 rps/cell | 99.9% |
| Publish | 80 ms | 300 ms | 700 ms | 100 rps/cell | 99.95% |

Demo-trial tenants: best-effort (no contractual SLO).

## Failure Modes + Recovery

- Sender domain unverified → 422 with remediation link to mail µservice domain verification flow.
- Ontology trait registry drift → token resolution returns ontology candidates for the unknown trait.
- Accessibility audit unreachable → fail-closed with `502 audit_service_unreachable`; never silently lower threshold.
- Smart-content rule compile failure → 422 with rule-position + reason.
- Publish race condition (two principals publish same draft) → CAS on `version` column; second publish gets `409 version_conflict`.

## Migration Notes

HubSpot Marketing Email export ZIP contains HTML + assets + smart-content rules + token map. Import pipeline:
1. Extract HTML and rewrite asset URLs to Oyatie drive µservice references.
2. Map HubSpot Personalization Tokens to ontology traits (mapping table in `migration-playbooks/from-hubspot-marketing-hub.md` §4.2).
3. Map HubSpot Smart Content rules to `dynamic_content` block predicates.
4. Run accessibility audit; HubSpot does not require ≥ 85.00 floor so some imports will land in `validated` status pending remediation.

Marketo Email Editor 2.0 export uses `MKTOEMAIL` schema; map Marketo Tokens to ontology traits per `migration-playbooks/from-marketo.md` §4.2.

Mailchimp Regular Campaign export uses Merge Tags `*|FNAME|*`; map to Mustache `{{first_name}}` per `migration-playbooks/from-mailchimp.md` §4.2.

## Cross-µservice Handoffs

- `mail` validates verified-sender domain via mail contract; supplies DKIM/SPF/DMARC status to deliverability bounded context.
- `ontology` validates token resolution against trait registry.
- `audit-chain` seals every email lifecycle event.
- `data-boundary` labels content blocks with PII_QUASI when subject-specific tokens are used.
- `drive` hosts large image assets via signed URLs.
- `intelligence` predicts subject-line lift for A/B variants (consumed by `a-b-test` bounded context).
- `finops` receives compose CPU + storage dimensions; receives per_usage `email_sends` meter on send.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-031-email-compose.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-031-email-compose.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-031-email-compose.md` matched [`finops`, `per_usage`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-031-email-compose.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
