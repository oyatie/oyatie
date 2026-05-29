---
doc_class: ImplementationPlan
ip_id: IP-001-tenant-scope-kernel
microservice: contact-center
related_adrs: [ADR-0242, ADR-0243, ADR-0244, ADR-0257, ADR-0263, ADR-0321]
journey_id: J-CC-01-tenant-contact-center-launch
status: proposed
date: 2026-05-20
owner: axis-contact-center
availability: paid
---

# IP-001: Contact Center Tenant Scope Kernel

## Context

This slice creates the tenant boundary for a contact center launch without copying Genesys division, NICE CXone business unit, Five9 domain, Talkdesk account, or AWS instance semantics into the platform. Nadia Singh is the named persona: she activates a regulated support tenant and needs queues, agents, recordings, and routing state to stay tenant-scoped before any call or chat can enter service.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `contact_center_scope` | `scope_id` | `uuid primary key` | Stable Oyatie contact-center scope. |
| `contact_center_scope` | `tenant_id` | `uuid not null` | Tenant partition key. |
| `contact_center_scope` | `service_region` | `text not null` | Cell/region where routing is allowed. |
| `contact_center_scope` | `regulated_channel_flags` | `text[] not null` | `voice_recorded`, `sms`, `chat`, `emergency_bypass`. |
| `contact_center_scope` | `source_vendor_refs` | `jsonb not null default '{}'` | Genesys org/division, NICE CXone BU, Five9 domain, Talkdesk account, AWS instance. |
| `contact_center_scope` | `recording_residency_pack` | `text not null` | HIPAA, GDPR, PCI-DSS, TCPA overlay. |
| `contact_center_scope` | `created_at` | `timestamptz not null` | HLC timestamp. |

## API Endpoints

REST `POST /v1/contact-center/scopes`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-cc000001",
  "service_region": "us-east-cell-2",
  "regulated_channel_flags": ["voice_recorded", "chat"],
  "recording_residency_pack": "TCPA",
  "source_vendor_refs": {
    "genesys_division_id": "div-44",
    "nice_cxone_business_unit_id": "bu-19",
    "five9_domain_id": "dom-8",
    "talkdesk_account_id": "td-220",
    "aws_connect_instance_arn": "arn:aws:connect:us-east-1:111:instance/abc"
  }
}
```

gRPC `ContactCenterScopeService.CreateScope(CreateContactCenterScopeRequest)` adds `request_context.trace_id` and returns `scope_id`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"contact-center.admin"` | `contactCenter::CreateScope` | `ContactCenterScope::*` | `tenant_id`, `service_region`, `recording_residency_pack` |
| `Service::"telephony-adapter"` | `contactCenter::BindVendorInstance` | `ContactCenterScope::*` | `source_vendor`, `migration_batch_id`, `dry_run` |
| `User::"auditor"` | `contactCenter::ReadScope` | `ContactCenterScope::*` | `read_reason`, `ticket_id`, `pack_id` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Genesys Division | `ContactCenterScope` | `divisionId -> source_vendor_refs.genesys_division_id`; division members map separately to agent assignments. |
| NICE CXone Business Unit | `ContactCenterScope` | `businessUnitId -> source_vendor_refs.nice_cxone_business_unit_id`. |
| Five9 Domain | `ContactCenterScope` | `domainId -> source_vendor_refs.five9_domain_id`; domain timezone advisory only. |
| Talkdesk Account | `ContactCenterScope` | `accountId -> source_vendor_refs.talkdesk_account_id`. |
| AWS Instance | `ContactCenterScope` | `instanceArn -> source_vendor_refs.aws_connect_instance_arn`; instance policies do not become Cedar grants. |

## Workflow Steps

1. `ResolveTenant` verifies support tenant and activated packs.
2. `NormalizeVendorInstanceRefs` validates vendor ids and source namespace.
3. `EvaluateScopePermit` calls Cedar before write.
4. `PersistScope` inserts `contact_center_scope`.
5. `ProjectContactCenterScope` writes ontology node.
6. `SealAudit` emits scope creation evidence.

Branches: duplicate vendor instance returns `409 scope_conflict`; unsupported recording pack returns `403 recording_pack_denied`; missing region returns `422 service_region_required`.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-CONTACT-CENTER-SCOPE-CREATED` | `tenant_id`, `scope_id`, `service_region`, `source_vendor_refs_hash`, `cedar_decision_id` |
| `EVT-DATA-EGRESS` | Only when migration preview exports source routing ids. |
| `EVT-ERROR-CONTACT-CENTER-SCOPE` | `tenant_id`, `source_vendor`, `error_code`, `recovery_branch` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Create scope | 50 ms | 190 ms | 380 ms | 120 rps/cell | 99.95% |
| Read scope | 12 ms | 55 ms | 110 ms | 2,000 rps/cell | 99.99% |

## Failure Modes + Recovery

- Vendor instance collision: preserve existing scope, emit conflict evidence, and require migration-owner approval.
- Cedar deny: persist no row and return denial with policy version.
- Recording pack mismatch: block agent login until pack overlay and scope agree.

## Migration Notes

Genesys, NICE CXone, Five9, Talkdesk, and AWS all treat tenancy and routing boundaries differently. Migration stores vendor ids as references only; Oyatie scope, tenant, and pack fields become the authority.

## Cross-µservice Handoffs

- `tenancy` validates tenant membership.
- `identity` maps agents and supervisors.
- `policy-engine` evaluates Cedar.
- `ontology` stores `ContactCenterScope`.
- `audit-chain` seals contact-center scope events.
