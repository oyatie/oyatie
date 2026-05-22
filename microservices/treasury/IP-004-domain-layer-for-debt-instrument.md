---
doc_class: ImplementationPlan
microservice: treasury
status: Accepted
date: 2026-05-21
owner_team: axis-treasury + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0244, ADR-0253, ADR-0297, ADR-0314, ADR-0315]
planned_enforcement_ref: oya-governance-treasury-doc-suite
ip_id: IP-004
---

# IP-004: Domain layer for debt-instrument

## A. Intent
Implement the domain slice for Treasury.debt-instrument. The slice is single-PR-sized, tenant-scoped, and contract-bound to OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, BNF v4.1, and ADR-0105 layer vocabulary.

## B. Acceptance criteria
- Treasury.debt-instrument has typed inputs and outputs.
- Cedar default deny is preserved.
- EVT-TREASURY-DEBT_INSTRUMENT-IP_ACCEPTED is emitted by tests or evidence fixtures.
- Marketplace settlement remains read-only and owned by marketplace per ADR-0314.
- Benchmarks are named: SAP TRM Treasury and Risk Management | Oracle Fusion Cash Management | Workday Financial Management cash and treasury counterpart | NetSuite Cash Management | Microsoft Dynamics 365 Finance Cash and Bank.

## C. Verification
Run unit, contract, policy, worker replay, and integration tests for this slice; attach dashboard and audit evidence to the PR.
- IP detail 001: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 002: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 003: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 004: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 005: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 006: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 007: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 008: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 009: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 010: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 011: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 012: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 013: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 014: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 015: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 016: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 017: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 018: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 019: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 020: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 021: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 022: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 023: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 024: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 025: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 026: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 027: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 028: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 029: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 030: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 031: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 032: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 033: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 034: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 035: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 036: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 037: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 038: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 039: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 040: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 041: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 042: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 043: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 044: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 045: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 046: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 047: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 048: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 049: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 050: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 051: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 052: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 053: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 054: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
- IP detail 055: Treasury.debt-instrument.domain verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.
