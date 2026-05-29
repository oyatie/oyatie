---
doc_class: ImplementationPlan
ip_id: IP-027-consent-suppression-ledger
microservice: marketing-automation
related_adrs: [ADR-0243, ADR-0272, ADR-0263, ADR-0321]
journey_id: J-MA-27-global-suppression-before-send
status: proposed
date: 2026-05-20
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-027: Consent Suppression Ledger

## Context

This net-new slice gives Hana Mori an auditable suppression ledger that proves a contact was excluded before send. It subsumes Marketo Unsubscribed, HubSpot subscription types, Mailchimp Premium cleaned/unsubscribed state, Iterable channel subscription state, and Braze subscription groups.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_suppression_ledger` | `suppression_id` | `uuid primary key` | Immutable suppression entry. |
| `marketing_suppression_ledger` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_suppression_ledger` | `subject_ref` | `text not null` | Hashed profile/contact ref. |
| `marketing_suppression_ledger` | `purpose` | `text not null` | Consent purpose blocked. |
| `marketing_suppression_ledger` | `channel` | `text not null` | `email`, `sms`, `push`, `in_app`. |
| `marketing_suppression_ledger` | `source_vendor` | `text` | Vendor origin if migrated. |
| `marketing_suppression_ledger` | `effective_at` | `timestamptz not null` | HLC effective time. |
| `marketing_suppression_ledger` | `revoked_by_event_id` | `text` | Audit event proving reversal. |

## API Endpoints

REST `POST /v1/marketing-automation/suppressions:check`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-ma000001",
  "subject_ref": "profile_hash_98f1",
  "purpose": "product_marketing",
  "channel": "email",
  "send_context": {"campaign_id": "cmp_q2_upgrade", "journey_step_id": "mail_1"}
}
```

gRPC `MarketingSuppressionService.Check(CheckSuppressionRequest)` returns `allowed`, `suppression_id`, and `proof_event_id`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `Service::"journey-runner"` | `marketingAutomation::CheckSuppression` | `MarketingSuppressionLedger::*` | `tenant_id`, `subject_ref`, `purpose`, `channel` |
| `User::"privacy.admin"` | `marketingAutomation::AppendSuppression` | `MarketingSuppressionLedger::*` | `lawful_basis`, `source_vendor`, `effective_at` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Marketo Unsubscribed | `MarketingSuppression` | boolean becomes purpose/channel-specific entry. |
| HubSpot Subscription Type | `MarketingSuppression` | subscription id maps to `purpose`. |
| Mailchimp Premium Cleaned/Unsubscribed | `MarketingSuppression` | status maps to channel email block reason. |
| Iterable Channel Subscription | `MarketingSuppression` | channel state maps to `channel`. |
| Braze Subscription Group | `MarketingSuppression` | group id maps to purpose/channel pair. |

## Workflow Steps

1. `ResolveSubject` hashes contact/profile identifier through data-boundary.
2. `LoadSuppressionEntries` queries active entries by purpose and channel.
3. `EvaluateOverridePermit` allows transactional exceptions only through Cedar.
4. `ReturnProof` attaches suppression evidence to send decision.
5. `SealCheck` emits audit event for every denied send.

Branches: transactional notice can bypass marketing suppression only with `transactional_notice` purpose; missing subject denies; revoked suppression requires active revocation event.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-SUPPRESSION-APPENDED` | `tenant_id`, `suppression_id`, `purpose`, `channel`, `source_vendor` |
| `EVT-MARKETING-SEND-SUPPRESSED` | `tenant_id`, `campaign_id`, `subject_ref_hash`, `suppression_id` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Suppression check | 6 ms | 25 ms | 60 ms | 50k checks/s/cell | 99.99% |
| Append suppression | 35 ms | 160 ms | 350 ms | 1k writes/s/cell | 99.95% |

## Failure Modes + Recovery

- Ledger unavailable: fail closed for marketing sends and emit incident metric.
- Conflicting vendor states: most restrictive state wins; create reconciliation task.
- Subject hash mismatch: quarantine migration row and require re-hash with same data-boundary salt.

## Migration Notes

Marketo and Mailchimp often store global unsubscribe while HubSpot, Iterable, and Braze allow topic or subscription-group granularity. Migration expands broad unsubscribes to purpose/channel rows and never narrows consent without explicit evidence.

## Cross-µservice Handoffs

- `consent` owns lawful-basis records.
- `data-boundary` hashes subject refs.
- `mail` and `notification` consume allow/deny decisions.
- `audit-chain` seals suppression proof.
- `privacy` consumes ledger rows for DSR exports.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-027-consent-suppression-ledger.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/marketing-automation/IP-027-consent-suppression-ledger.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].
