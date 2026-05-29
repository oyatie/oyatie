---
doc_class: ImplementationPlan
ip_id: IP-002-cedar-default-deny
microservice: marketing-automation
related_adrs: [ADR-0243, ADR-0244, ADR-0246, ADR-0263, ADR-0294, ADR-0321]
journey_id: J-MA-02-permitted-audience-build
status: proposed
date: 2026-05-20
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-002: Marketing Automation Cedar Default Deny

## Context

This slice replaces Marketo role permissions, HubSpot marketing permissions, Mailchimp Premium seat roles, Iterable API-key scopes, and Braze team roles with one default-deny Cedar gate. Nadia Singh is the named persona: she activates campaign permissions per tenant and needs every deny to explain which purpose, audience, and channel blocked execution.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_policy_binding` | `binding_id` | `uuid primary key` | Immutable policy binding row. |
| `marketing_policy_binding` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_policy_binding` | `scope_id` | `uuid not null` | FK to `marketing_tenant_scope`. |
| `marketing_policy_binding` | `action_name` | `text not null` | Cedar action, e.g. `LaunchCampaign`. |
| `marketing_policy_binding` | `purpose_allowlist` | `text[] not null` | Consent purposes allowed for action. |
| `marketing_policy_binding` | `channel_allowlist` | `text[] not null` | `email`, `sms`, `push`, `in_app`, `webhook`. |
| `marketing_policy_binding` | `policy_version` | `bigint not null` | Monotonic fragment version after soak. |
| `marketing_policy_binding` | `soak_started_at` | `timestamptz not null` | ADR-0294 soak boundary. |

## API Endpoints

REST `POST /v1/marketing-automation/policy/evaluate`

```json
{
  "principal": "User::marketing.manager.42",
  "action": "marketingAutomation::LaunchCampaign",
  "resource": "Campaign::cmp_2026q2_upgrade",
  "context": {
    "tenant_id": "018f8ad2-0e0f-7ad2-92d2-ma000001",
    "scope_id": "01HXMA_SCOPE",
    "purpose": "product_marketing",
    "channel": "email",
    "residency_pack_id": "GDPR"
  }
}
```

gRPC `MarketingPolicyService.Evaluate(EvaluateMarketingPolicyRequest)` returns `decision`, `policy_version`, `deny_reason`, and `audit_event_id`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.manager"` | `marketingAutomation::LaunchCampaign` | `Campaign::*` | `tenant_id`, `scope_id`, `purpose`, `channel`, `frequency_window` |
| `User::"agency.operator"` | `marketingAutomation::BuildSegment` | `Segment::*` | `tenant_id`, `client_tenant_id`, `trait_classes` |
| `Service::"journey-runner"` | `marketingAutomation::SendStep` | `JourneyStep::*` | `tenant_id`, `consent_snapshot_id`, `suppression_revision` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Marketo Role | `MarketingPermitBinding` | `roleName -> action_name[]`; workspace scope maps through IP-001. |
| HubSpot Permission Set | `MarketingPermitBinding` | `permissions[] -> action_name[]`; seat type ignored unless mapped to persona. |
| Mailchimp Premium Role | `MarketingPermitBinding` | `role -> action_name`; audience access becomes resource scope. |
| Iterable API Key Scope | `MarketingPermitBinding` | `permissions -> service principal actions`; API key never becomes user principal. |
| Braze Team Role | `MarketingPermitBinding` | `team_role -> purpose_allowlist + channel_allowlist`. |

## Workflow Steps

1. `LoadBinding` reads active binding for tenant and scope.
2. `CompileContext` pulls consent, channel, pack, and frequency signals.
3. `EvaluateDefaultDeny` asks policy-engine library first.
4. `ExplainDeny` expands Cedar diagnostics into field-specific refusal.
5. `SealDecision` writes audit event and policy version.

Branches: missing binding denies; stale policy version denies mutating actions; auditor read can proceed only with `read_reason`.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-POLICY-DECISION` | `tenant_id`, `principal`, `action`, `resource`, `decision`, `policy_version` |
| `EVT-MARKETING-POLICY-DENIED` | `deny_reason`, `purpose`, `channel`, `residency_pack_id`, `cedar_decision_id` |
| `EVT-CAPABILITY-INVOKED` | Emitted before policy-bound campaign launch capability executes. |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Policy evaluate | 8 ms | 35 ms | 75 ms | 8,000 eval/s/cell | 99.99% |
| Policy publish after soak | 90 ms | 500 ms | 900 ms | 20 publishes/min/cell | 99.95% |

## Failure Modes + Recovery

- Policy-engine unavailable: fail closed for mutation, allow cached read-only auditor checks for 60 seconds.
- Fragment soak not complete: reject publish and emit `EVT-MARKETING-POLICY-DENIED`.
- Vendor role import ambiguous: keep the imported role in `migration_shadow_policy` until a human maps it to Cedar actions.

## Migration Notes

Marketo, HubSpot, Mailchimp Premium, Iterable, and Braze all mix UI roles with API scopes. Migration must produce a reviewable mapping table; it must not auto-grant `LaunchCampaign`, `ExportAudience`, or `OverrideSuppression` from any vendor-admin label.

## Cross-µservice Handoffs

- `policy-engine` owns Cedar evaluation and fragment soak.
- `identity` resolves user and service principals.
- `consent` supplies purpose snapshots.
- `audit-chain` seals decisions and denies.
- `workflow-engine` consumes policy decisions before journey nodes execute.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-002-cedar-default-deny.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-002-cedar-default-deny.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].
