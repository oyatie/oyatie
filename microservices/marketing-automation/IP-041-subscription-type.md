---
doc_class: ImplementationPlan
ip_id: IP-041-subscription-type
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0251, ADR-0263, ADR-0321, ADR-0328]
bounded_context: subscription-type
journey_id: J-MA-41-publication-subscription-management
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-041: Subscription Type

## Context

Subscription types are publication-channel categories: newsletter / product-update / event-invite / educational-series. HubSpot Subscription Type + Communication Preference Page is the canonical model. Marketo Communication Limits and Mailchimp Groups are narrower. Subscription types are distinct from suppression: subscription-type tracks positive opt-in per category; consent-audience (IP-027) tracks suppression. The two together honor CAN-SPAM + CASL + GDPR + ePrivacy.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_subscription_type` | `subscription_type_id` | `uuid primary key` | Subscription type id. |
| `marketing_subscription_type` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_subscription_type` | `name` | `text not null` | Display name. |
| `marketing_subscription_type` | `purpose` | `text not null` | Purpose tag. |
| `marketing_subscription_type` | `default_opt_in` | `boolean not null default false` | Per pack: GDPR/CASL require explicit opt-in. |
| `marketing_subscription_type` | `disclosure_block` | `jsonb not null` | Per pack overlay. |
| `marketing_subscription_type` | `unsubscribe_link_required` | `boolean not null default true` | CAN-SPAM/CASL. |
| `marketing_subscription_type` | `active` | `boolean not null default true` | Activate / deactivate. |
| `marketing_subscription_membership` | `membership_id` | `uuid primary key` | Per-subject membership. |
| `marketing_subscription_membership` | `subscription_type_id` | `uuid not null` | FK. |
| `marketing_subscription_membership` | `subject_hash` | `text not null` | Subject ref. |
| `marketing_subscription_membership` | `status` | `text not null` | subscribed / unsubscribed / pending_confirmation. |
| `marketing_subscription_membership` | `opt_in_evidence_id` | `text` | Reference to consent-graph evidence. |
| `marketing_subscription_membership` | `last_change_hlc` | `hlc not null` | HLC. |

## API Endpoints

REST `POST /v1/marketing-automation/subscription-types`:

```json
{
  "tenant_id": "...",
  "name": "Product Updates",
  "purpose": "product_marketing",
  "default_opt_in": false,
  "disclosure_block": {
    "en": "Monthly product updates and feature announcements",
    "fr": "Mises à jour mensuelles..."
  }
}
```

REST `POST /v1/marketing-automation/subscription-types/{subscription_type_id}/subscribe`:

```json
{"subject_hash": "h_abc123", "opt_in_evidence_id": "cg_evidence_uuid"}
```

REST `POST /v1/marketing-automation/subscription-types/{subscription_type_id}/unsubscribe` (subject-initiated):

```json
{"subject_hash": "h_abc123", "source": "subject_link_click"}
```

REST `GET /preferences/{tenant_id}/{subject_token}` renders the Communication Preference Page.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::CreateSubscriptionType` | `MarketingSubscriptionType::*` | `tenant_class`, `subscription_types_count` |
| `User::"subject"` | `marketingAutomation::SubjectUnsubscribe` | `MarketingSubscriptionMembership::*` | `subject_hash`, `signed_token_verified` |

Subject-initiated unsubscribe always allowed per Cedar policy (GDPR Article 21 / CAN-SPAM / CASL).

Demo-trial gate: `tenant_class == 'demo_trial' && subscription_types_count >= 5` denies create.

## Workflow Steps

1. `ValidateDisclosureBlock` ensures required locales per pack overlay.
2. `AuthorizeCreate` calls Cedar.
3. `PersistType` writes row.
4. On subscribe, `ValidateOptInEvidence` resolves opt_in_evidence_id against consent-graph.
5. `PersistMembership` writes membership row.
6. On unsubscribe, `CascadeToConsentAudience` calls IP-027 append-revocation with channel + purpose derived from subscription type.
7. `EmitChange` emits `EVT-MARKETING-SUBSCRIPTION-*`.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-SUBSCRIPTION-TYPE-CREATED` | `subscription_type_id`, `purpose`, `disclosure_block_locales`, `tenant_class` |
| `EVT-MARKETING-SUBSCRIPTION-SUBSCRIBED` | `membership_id`, `subscription_type_id`, `subject_hash`, `opt_in_evidence_id`, `consent_graph_evidence_resolved: true` |
| `EVT-MARKETING-SUBSCRIPTION-UNSUBSCRIBED` | `membership_id`, `subscription_type_id`, `subject_hash`, `source`, `cascaded_to_consent_audience: true` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Subscribe | 40 ms | 150 ms | 400 ms | 2000 rps/cell | 99.95% |
| Unsubscribe (subject-initiated) | 30 ms | 120 ms | 300 ms | 5000 rps/cell | 99.99% |
| Render preference page | 25 ms | 100 ms | 250 ms | 1000 rps/cell | 99.99% |

## Failure Modes + Recovery

- Consent-graph unreachable on subscribe → 502 `consent_graph_unreachable`; fail-closed because positive opt-in evidence cannot be sealed.
- Unsubscribe replay → idempotent (already-unsubscribed returns 200 with `x-already-unsubscribed: true`).
- Disclosure block locale missing for pack overlay → 422 `disclosure_locale_required`.

## Migration Notes

HubSpot Subscription Type export preserves per-type config + per-contact subscription state. Marketo Communication Limits map by topic. Mailchimp Groups map narrower (subject can be in multiple groups).

## Cross-µservice Handoffs

- `consent-graph` provides opt-in evidence custody.
- `consent-audience` (IP-027) cascades on unsubscribe.
- `mail` reads subscription status before send admission.
- `audit-chain` seals every change.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-041-subscription-type.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-041-subscription-type.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].
