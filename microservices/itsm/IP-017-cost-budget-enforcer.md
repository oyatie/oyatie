---
doc_class: IP
ip_id: IP-017-cost-budget-enforcer
microservice: itsm
status: rewritten-wave-15-ip-substance
date: 2026-05-21
owner_team: axis-itsm + cost-platform
counterparts: [ServiceNow ITSM, Jira Service Management, Freshservice]
source_artifacts:
  - microservices/itsm/manifest.json
  - microservices/itsm/dashboards/tenant-cost-and-capacity.json
  - microservices/itsm/contracts/openapi-v1.yaml
  - microservices/itsm/src/domain/mod.rs
---

# IP-017 ITSM Cost Budget Enforcer

## A. Problem
ITSM features can create unbounded cost: AI deflection attempts, workflow executions, mobile push, CMDB discovery, attachment storage, service-catalog provider calls, and backfill replays. The stamped IP did not distinguish demo_trial caps from paid billing components.

This IP turns manifest tenant-class limits into admission checks and cost evidence.

## B. Approach
Use `manifest.json` as the budget authority:

| Meter | demo_trial cap | Paid billing behavior |
|---|---|---|
| tickets/month | 500 | per usage |
| CMDB CIs | 200 | per stored CI/usage |
| workflow executions/month | 1000 | workflow execution usage |
| AI deflection attempts/month | 200 | AI usage line item |
| attachment storage | 5 GiB | GB-month |
| mobile API calls/month | 5000 | API usage |

Admission happens before expensive work, but after enough request validation to produce useful refusal evidence.

## C. Deliverables
- Budget admission port for ITSM actions that consume metered resources.
- Dashboard panels in `tenant-cost-and-capacity.json`.
- Tests for demo cap refusal and paid metering pass-through.
- Audit events for `budget_admitted`, `budget_denied`, and `budget_near_limit`.
- Cost dimensions that include tenant class, action, bounded context, and cell.

## D. Implementation
1. Extract demo_trial caps and paid meter names from `manifest.json` into a typed budget config.
2. Map ITSM actions to meters: ticket create, CMDB sync, workflow execution, AI deflection, attachment storage, mobile call.
3. Invoke budget admission before AI, workflow, discovery, or storage-heavy actions.
4. Do not budget-gate emergency P0/P1 acknowledgement except to emit post-facto cost evidence.
5. Add dashboard panels for cap usage percent and refusal counts.
6. Add tests that demo_trial ticket 501 refuses with a budget denial.
7. Add paid-path tests that metering records are emitted without cap refusal.
8. Add rollback: disable costly optional features while preserving incident open/change approval.

## E. Acceptance
- Demo trial caps match `manifest.json`.
- Paid tenants emit metering events for configured billing components.
- Budget refusal is not confused with Cedar denial.
- Cost evidence contains no raw tenant id in metrics labels.

## F. Evidence
- `manifest.json` defines ITSM demo and paid meter shape.
- `dashboards/tenant-cost-and-capacity.json` exists.
- `contracts/openapi-v1.yaml` declares actions that can map to usage.
- ADR-0331 governs demo_trial vs paid behavior.

## G. Counterparts
| Counterpart | Gap closed by this IP |
|---|---|
| ServiceNow subscription entitlements | Explicit demo caps and paid meters |
| Jira Service Management user/asset limits | Admission evidence instead of hidden quota failure |
| Freshservice tier limits | Per-action budget behavior tied to tenant class |

## H. Cold-start buildability notes
- Parse caps from manifest before hardcoding limits elsewhere.
- Keep budget denial separate from Cedar denial.
- Add demo_trial cap tests for ticket and AI deflection meters first.
- Emit paid metering even when there is no cap refusal.
- Do not gate emergency acknowledgement on budget.
- Use hashed tenant labels in metrics.
- Keep storage and mobile meters separate.
- Add near-limit warning before hard denial.
- Treat missing billing integration as a follow-up, not a pass claim.
- Keep ADR-0331 tenant-class names exact.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/itsm/IP-017-cost-budget-enforcer.md` matched [`cost`, `metered`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/itsm/IP-017-cost-budget-enforcer.md`, `microservices/itsm/manifest.json`, `microservices/itsm/capacity-model.md`, `microservices/itsm/compliance.md`, `microservices/itsm/ARCHITECTURE.md`].
