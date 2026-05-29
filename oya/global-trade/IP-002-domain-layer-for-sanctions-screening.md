---
doc_class: ImplementationPlan
microservice: global-trade
status: Accepted
date: 2026-05-21
owner_team: axis-global-trade + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0244, ADR-0253, ADR-0297, ADR-0314, ADR-0315]
planned_enforcement_ref: oya-governance-global-trade-doc-set
ip_id: IP-002
---

# IP-002: Domain layer for sanctions-screening

## A. Intent
Implement the domain slice for Global Trade.sanctions-screening. The slice is single-PR-sized, tenant-scoped, and contract-bound to OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, BNF v4.1, and ADR-0105 layer vocabulary.

## B. Acceptance criteria
- Global Trade.sanctions-screening has typed inputs and outputs.
- Cedar default deny is preserved.
- EVT-GLOBAL_TRADE-SANCTIONS_SCREENING-IP_ACCEPTED is emitted by tests or evidence fixtures.
- Marketplace settlement remains read-only and owned by marketplace per ADR-0314.
- Benchmarks are named: SAP GTS Global Trade Services | Oracle Global Trade Management | Workday supplier-compliance workflow counterpart | NetSuite international tax and trade counterpart | Microsoft Dynamics 365 global trade and export-control counterpart.

## C. Verification
Run unit, contract, policy, worker replay, and integration tests for this slice; attach dashboard and audit evidence to the PR.
- IP detail 001: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 002: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 003: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 004: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 005: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 006: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 007: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 008: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 009: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 010: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 011: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 012: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 013: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 014: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 015: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 016: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 017: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 018: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 019: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 020: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 021: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 022: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 023: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 024: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 025: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 026: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 027: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 028: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 029: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 030: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 031: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 032: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 033: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 034: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 035: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 036: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 037: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 038: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 039: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 040: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 041: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 042: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 043: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 044: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 045: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 046: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 047: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 048: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 049: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 050: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 051: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 052: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 053: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 054: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 055: Global Trade.sanctions-screening.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
