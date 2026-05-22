---
doc_class: IP
ip_id: IP-018-capacity-admission-control
microservice: itsm
status: rewritten-wave-15-ip-substance
date: 2026-05-21
owner_team: axis-itsm + sre
counterparts: [ServiceNow ITSM, Jira Service Management, Freshservice]
source_artifacts:
  - microservices/itsm/manifest.json
  - microservices/itsm/dashboards/local-domain-throughput.json
  - microservices/itsm/dashboards/slo-and-error-budget.json
  - microservices/itsm/src/domain/mod.rs
---

# IP-018 ITSM Capacity Admission Control

## A. Problem
ITSM has critical and noncritical workloads in the same µservice. A CMDB discovery surge, KB import, or backfill replay must not starve P1 incident creation, SLA breach detection, or on-call acknowledgement. The stamped IP described capacity generically and did not define priority classes.

This IP defines admission classes for ITSM actions.

## B. Approach
Classify work by operational priority:

| Class | Examples | Admission behavior |
|---|---|---|
| critical | P0/P1 incident open, page ack, SLA breach | reserve capacity, never wait behind bulk |
| interactive | portal ticket create, agent action, status update | normal p95 budget |
| background | backfill replay, CMDB discovery, analytics export | queue and shed first |
| optional | AI deflection, report refresh | degrade before core flow |

Capacity decisions must include tenant class and cell while preserving the retired capacity-tier doctrine from ADR-0331.

## C. Deliverables
- Admission policy mapping ITSM capabilities to capacity class.
- Queue/backpressure behavior for background workers.
- Dashboard panels in `local-domain-throughput.json` and `slo-and-error-budget.json`.
- Tests proving critical actions bypass background saturation.
- Runbook for stuck background queue and admission misclassification.

## D. Implementation
1. Add capacity class metadata beside `Capability::action_slug()`.
2. Reserve capacity for `Priority::is_major()` incident paths and SLA breach recomputation.
3. Apply per-tenant and per-cell concurrency limits for background replay/discovery.
4. Shed optional AI/report work before rejecting interactive ticket operations.
5. Emit admission decisions with class, queue depth, and degraded-mode reason.
6. Add dashboard burn alerts when interactive p95 approaches the SLO.
7. Add tests simulating saturated background queue and successful P1 incident open.
8. Document rollback as lowering background worker concurrency, not disabling ITSM.

## E. Acceptance
- Critical P0/P1 and SLA paths have reserved capacity.
- Background jobs are queued or shed before interactive traffic fails.
- Capacity terms do not revive retired silver/gold/platinum tenant tiers.
- Dashboards show admission decision counts by class.

## F. Evidence
- `manifest.json` names demo_trial and paid tenant classes.
- `src/domain/mod.rs` defines `Priority::is_major()` and `Capability`.
- `dashboards/local-domain-throughput.json` and `slo-and-error-budget.json` exist.
- ADR-0331 controls tenant-class vocabulary.

## G. Counterparts
| Counterpart | Gap closed by this IP |
|---|---|
| ServiceNow operational priority queues | Explicit critical/background admission classes |
| Jira Service Management automation throughput | Bulk work cannot starve incidents |
| Freshservice workflow execution limits | Degradation order is documented and tested |

## H. Cold-start buildability notes
- Add capacity class metadata beside `Capability`.
- Reserve critical capacity before tuning background queues.
- Simulate saturated background jobs in tests.
- Keep P0/P1 priority checks in domain terms.
- Shed optional AI/report work before interactive ticket work.
- Do not revive retired silver/gold/platinum tenant tiers.
- Emit queue depth without raw tenant labels.
- Link admission denial to dashboard burn.
- Roll back by lowering worker concurrency.
- Keep background replay and CMDB discovery in the lowest admission class.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-018-capacity-admission-control.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/itsm/IP-018-capacity-admission-control.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].
