---
doc_class: ImplementationPlan
ip_id: IP-030-compensation-readiness-handoff
microservice: performance-management
related_adrs: [ADR-0243, ADR-0263, ADR-0321]
journey_id: J-PM-30-compensation-readiness-handoff
status: proposed
date: 2026-05-20
owner: axis-performance-management
capability_tier: T3
---

# IP-030: Compensation Readiness Handoff

## Context

This net-new slice hands approved performance outcomes to compensation planning without letting performance-management own payroll or pay decisions. It displaces Lattice compensation exports, Workday Talent performance-to-compensation feeds, 15Five review exports, and Culture Amp engagement overlays with an auditable readiness packet.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `performance_comp_readiness_packet` | `packet_id` | `uuid primary key` | Immutable handoff packet. |
| `performance_comp_readiness_packet` | `tenant_id` | `uuid not null` | Tenant partition. |
| `performance_comp_readiness_packet` | `cycle_id` | `uuid not null` | Review cycle. |
| `performance_comp_readiness_packet` | `worker_ref` | `text not null` | Hashed worker ref. |
| `performance_comp_readiness_packet` | `outcome_summary_ref` | `text not null` | Review/calibration summary ref. |
| `performance_comp_readiness_packet` | `fairness_check_ref` | `text not null` | Calibration fairness evidence. |
| `performance_comp_readiness_packet` | `handoff_state` | `text not null` | `draft`, `approved`, `sent`, `revoked`. |

## API Endpoints

REST `POST /v1/performance-management/compensation-readiness-packets`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-pm000001",
  "cycle_id": "cycle_2026_h1",
  "worker_ref": "hris:worker:778",
  "outcome_summary_ref": "review-summary:778:2026h1",
  "fairness_check_ref": "fairness:check:2026h1:44"
}
```

gRPC `CompReadinessService.ApprovePacket(ApproveCompReadinessPacketRequest)` returns `packet_id`, `handoff_state`, and audit id.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"hrbp"` | `performanceManagement::ApproveCompReadinessPacket` | `CompReadinessPacket::*` | `cycle_id`, `worker_ref`, `fairness_check_ref` |
| `Service::"compensation-planning"` | `performanceManagement::ReceiveReadinessPacket` | `CompReadinessPacket::*` | `handoff_state=approved`, `packet_id` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Lattice Compensation Export | `CompReadinessPacket` | review outcome maps to outcome summary ref. |
| Workday Talent Compensation Input | `CompReadinessPacket` | talent review outcome maps to packet. |
| 15Five Review Export | `CompReadinessPacket` | review summary maps to outcome summary ref. |
| Culture Amp Engagement Overlay | `CompReadinessSignal` | aggregate engagement signal remains advisory. |

## Workflow Steps

1. `LoadReviewOutcome` verifies review cycle closed.
2. `VerifyFairnessCheck` requires green or approved exception.
3. `EvaluateApprovalPermit` checks HRBP and labor-pack policy.
4. `CreateReadinessPacket` writes packet.
5. `SendHandoff` emits event to compensation-planning.

Branches: fairness check red blocks packet; cycle open returns `409`; compensation service receives only approved packets.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-PERFORMANCE-COMP-READINESS-CREATED` | `tenant_id`, `packet_id`, `cycle_id`, `worker_ref_hash` |
| `EVT-PERFORMANCE-COMP-READINESS-SENT` | `packet_id`, `handoff_state`, `compensation_request_id` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Create packet | 70 ms | 300 ms | 650 ms | 500 rps/cell | 99.95% |
| Send handoff | 100 ms | 800 ms | 1.8 s | 20k packets/hour/cell | 99.9% |

## Failure Modes + Recovery

- Compensation-planning unavailable: keep packet approved, retry handoff idempotently.
- Fairness evidence missing: block packet and return required check.
- Packet sent then revoked: emit revocation and compensation must ignore prior packet.

## Migration Notes

Vendor exports often include ratings plus salary recommendations. Oyatie imports only performance readiness evidence; compensation amounts and payroll decisions remain outside this microservice.

## Cross-µservice Handoffs

- `compensation-planning` receives approved packets.
- `hris` supplies worker and cycle refs.
- `policy-engine` gates approval.
- `audit-chain` seals creation and handoff events.
- `finance` consumes only downstream compensation outputs, not raw review evidence.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/performance-management/IP-030-compensation-readiness-handoff.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/performance-management/IP-030-compensation-readiness-handoff.md`, `microservices/performance-management/manifest.json`, `microservices/performance-management/ARCHITECTURE.md`, `microservices/performance-management/PRD.md`, `microservices/performance-management/multi-region.md`, `microservices/performance-management/capacity-model.md`].
