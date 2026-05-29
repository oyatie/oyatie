---
doc_class: ImplementationPlan
microservice: treasury
status: Accepted
date: 2026-05-21
owner_team: axis-treasury + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0244, ADR-0253, ADR-0297, ADR-0314, ADR-0315]
planned_enforcement_ref: oya-governance-treasury-doc-set
ip_id: IP-007
---

# IP-007: Usecase layer for cash-position

## A. Intent
Implement the usecase slice for Treasury.cash-position. The slice is single-PR-sized, tenant-scoped, and contract-bound to OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, BNF v4.1, and ADR-0105 layer vocabulary.

## B. Acceptance criteria
- Treasury.cash-position has typed inputs and outputs.
- Cedar default deny is preserved.
- EVT-TREASURY-CASH_POSITION-IP_ACCEPTED is emitted by tests or evidence fixtures.
- Marketplace settlement remains read-only and owned by marketplace per ADR-0314.
- Benchmarks are named: SAP TRM Treasury and Risk Management | Oracle Fusion Cash Management | Workday Financial Management cash and treasury counterpart | NetSuite Cash Management | Microsoft Dynamics 365 Finance Cash and Bank.

## C. Verification
Run unit, contract, policy, worker replay, and integration tests for this slice; attach dashboard and audit evidence to the PR.
- IP detail 001: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 002: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 003: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 004: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 005: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 006: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 007: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 008: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 009: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 010: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 011: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 012: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 013: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 014: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 015: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 016: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 017: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 018: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 019: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 020: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 021: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 022: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 023: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 024: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 025: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 026: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 027: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 028: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 029: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 030: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 031: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 032: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 033: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 034: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 035: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 036: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 037: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 038: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 039: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 040: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 041: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 042: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 043: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 044: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 045: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 046: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 047: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 048: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 049: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 050: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 051: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 052: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 053: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 054: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 055: Treasury.cash-position.usecase verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
