---
doc_class: ImplementationPlan
ip_id: IP-054-mobile-sdk-seam
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0253-amendment, ADR-0263, ADR-0321, ADR-0328]
bounded_context: mobile-sdk-seam
journey_id: J-MA-54-tenant-mobile-app-sdk
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
frontend_languages: [Swift (iOS/macOS), Kotlin (Android)]
---

# IP-054: Mobile SDK Seam

## Context

HubSpot Mobile SDK + Marketo Mobile + Mailchimp Mobile App enable tenants to ingest in-app events and receive push notifications. Per the Rust-strict policy with frontend exceptions, the tenant-facing SDK is authored in Swift (iOS/macOS) and Kotlin (Android). This µservice provides the server-side seam contract — the SDK itself lives under tenant-supplied client apps.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_mobile_device` | `device_id` | `uuid primary key` | Per-device row. |
| `marketing_mobile_device` | `tenant_id` | `uuid not null` | Tenant. |
| `marketing_mobile_device` | `subject_hash` | `text not null` | Subject ref. |
| `marketing_mobile_device` | `device_token` | `text not null` | APNs or FCM token. |
| `marketing_mobile_device` | `platform` | `text not null` | ios / android. |
| `marketing_mobile_device` | `app_id` | `text not null` | Bundle id / package name. |
| `marketing_mobile_device` | `app_version` | `text` | App version. |
| `marketing_mobile_device` | `registered_at_hlc` | `hlc not null` | HLC. |

## API Endpoints (server-side seam contract)

REST `POST /v1/marketing-automation/mobile/devices` (called by SDK at app launch).

REST `POST /v1/marketing-automation/mobile/in-app-events` (called by SDK on user action).

REST `POST /v1/marketing-automation/mobile/{device_id}:request-push-send` (server-driven push).

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `Service::"mobile-sdk"` | `marketingAutomation::RegisterMobileDevice` | `MarketingMobileDevice::*` | `tenant_id`, `signed_api_key_verified` |
| `Service::"mobile-sdk"` | `marketingAutomation::IngestInAppEvent` | `MarketingBehavioralEvent::*` | `device_id`, `subject_hash`, `tenant_class`, `daily_event_count` |

## Workflow Steps

1. SDK obtains tenant API key + per-subject signed token at app login.
2. SDK registers device via `POST /devices`; server validates API key + signed token.
3. SDK posts in-app events via `POST /in-app-events`; server delegates to behavioral-profile (IP-044).
4. Server-driven push: `POST /:request-push-send` enqueues to messenger µservice.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-MOBILE-DEVICE-REGISTERED` | `device_id`, `subject_hash`, `platform`, `app_id` |
| `EVT-MARKETING-MOBILE-EVENT-INGESTED` | `device_id`, `event_id`, `subject_hash` |

## Migration Notes

HubSpot Mobile SDK migration requires replacing tenant client app SDK; events not transferred from vendor. Marketo Mobile and Mailchimp Mobile App migration similar.

## Cross-µservice Handoffs

- `messenger` delivers push notifications.
- `behavioral-profile` ingests in-app events.
- `consent-audience` records opt-in/opt-out for push channel.
- `audit-chain` seals events.
- Frontend SDKs live in tenant-supplied iOS (Swift) and Android (Kotlin) projects per Rust-strict policy frontend exception.
