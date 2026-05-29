---
doc_class: IP
ip_id: IP-011-observability-audit-events
microservice: itsm
status: rewritten-wave-15-ip-substance
date: 2026-05-21
owner_team: axis-itsm + observability
counterparts: [ServiceNow ITSM, Jira Service Management, Freshservice]
source_artifacts:
  - microservices/itsm/src/domain/mod.rs
  - microservices/itsm/src/usecase/mod.rs
  - microservices/itsm/dashboards/local-audit-completeness.json
  - microservices/itsm/dashboards/slo-and-error-budget.json
---

# IP-011 ITSM Observability and Audit Events

## A. Problem
ITSM cannot claim ServiceNow/Jira/Freshservice parity if an operator can open a P1 incident, approve a change, or publish a status update without a replayable audit trail. The stamped IP did not name events, metrics, traces, dashboards, or the Rust enum that already exists.

The specific gap is to bind ITSM usecase receipts to ADR-0263 event classes and dashboard evidence.

## B. Approach
Use `AuditEventKind` as the code-level event taxonomy and keep audit events separate from operational metrics. Metrics can hash tenant labels; audit-chain evidence keeps signed tenant details.

| Audit kind | Required dimensions | Dashboard |
|---|---|---|
| `IncidentOpened` | tenant, ticket, priority, cell | `local-audit-completeness.json` |
| `SlaBreached` | tenant, ticket, elapsed, policy | `slo-and-error-budget.json` |
| `ProblemLinked` | ticket, problem, relation type | `operating-bar-overview.json` |
| `ChangeApproved` | ticket, change, approver, freeze result | `local-policy-decisions.json` |
| `CmdbRelationUpdated` | CI ids, relation, source | `local-domain-throughput.json` |
| `MajorIncidentBridgeOpened` | incident, MLS group, responder set | `local-operator-remediation.json` |

## C. Deliverables
- A stable ADR-0263 event-class mapping for every `AuditEventKind`.
- Usecase receipts in `src/usecase/mod.rs` carrying enough subject metadata for audit publication.
- Dashboard definitions that can separate missing audit events from normal zero traffic.
- Integration tests that assert audit publication occurs after successful state change.
- Denial event taxonomy for Cedar failures from IP-008.

## D. Implementation
1. Add a table or machine-readable mapping from `AuditEventKind` to `oya.itsm.*` event classes.
2. Extend `AuditPublisher::publish_audit` implementations with dimensions for priority, capability, and cell when available.
3. Update `OpenIncident`, `RecomputeSla`, and `ApproveChange` tests to assert audit events are stored in `InMemoryItsmPorts`.
4. Add dashboards for audit completeness ratio and policy denial count.
5. Ensure metrics do not use raw `tenant_id`; keep tenant in signed audit evidence only.
6. Add tracing spans around adapter, usecase, policy, repository, and audit publish boundaries.
7. Define degraded behavior when audit-chain is unavailable: high-risk mutations pause; safe reads continue with degraded banner.
8. Link SLO burn evidence to `SlaBreached` events so SLA drift cannot be hidden.

## E. Acceptance
- Every successful mutating usecase emits exactly one success audit event.
- Cedar denials emit denial evidence and no success event.
- Dashboard JSON files include audit completeness and SLO burn evidence.
- Tests prove event ordering: authorize, mutate, publish audit.

## F. Evidence
- `src/domain/mod.rs` defines `AuditEventKind`.
- `src/usecase/mod.rs` defines `AuditPublisher` and publishes audit on current usecases.
- `dashboards/local-audit-completeness.json`, `dashboards/local-policy-decisions.json`, and `dashboards/slo-and-error-budget.json` exist.
- ADR-0263 governs audit event emission.

## G. Counterparts
| Counterpart | Gap closed by this IP |
|---|---|
| ServiceNow audit/history sets | Signed audit-chain events mapped to code-level enum |
| Jira Service Management automation logs | Usecase receipts prove mutation/event ordering |
| Freshservice audit logs | Denial and degraded-mode evidence are first-class |

## H. Cold-start buildability notes
- Start by asserting `InMemoryItsmPorts` captures audit calls.
- Add one event-class constant per `AuditEventKind`.
- Keep success and denial events separate.
- Do not log raw ticket descriptions in metrics or traces.
- Add span names around adapter and usecase boundaries before repository internals.
- Keep audit-chain outage behavior aligned with IP-013.
- Verify dashboard JSON references the final event names.
- Include capability and bounded context in evidence where available.
- Make missing audit event tests fail loudly.
- Preserve existing integration tests as baseline evidence.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-011-observability-audit-events.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/itsm/IP-011-observability-audit-events.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/itsm/IP-011-observability-audit-events.md` matched [`emission`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/itsm/IP-011-observability-audit-events.md`, `microservices/itsm/manifest.json`, `microservices/itsm/capacity-model.md`, `microservices/itsm/compliance.md`, `microservices/itsm/ARCHITECTURE.md`].
