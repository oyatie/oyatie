---
ip_id: IP-EMR-001
title: Scaffold per-BC kernel + domain crates for 15 bounded contexts
microservice: emr
status: planned
date: 2026-05-21
sequence: 1
depends_on: []
unblocks: [IP-EMR-002, IP-EMR-003]
estimated_effort_hours: 80
owner: axis-emr
---

# IP-EMR-001: Scaffold per-BC kernel + domain crates

## Goal

Materialize the 15-BC × 2-layer (kernel + domain) crate scaffold per ADR-EMR-MS-001 + ADR-0105 13-layer enum + ADR-0131 flat layout.

## Deliverables

Create 30 Rust crates under `microservices/emr/src/crates/`:

```
oya-emr-patient-kernel              oya-emr-patient-domain
oya-emr-encounter-kernel            oya-emr-encounter-domain
oya-emr-problem-kernel              oya-emr-problem-domain
oya-emr-medication-kernel           oya-emr-medication-domain
oya-emr-allergy-kernel              oya-emr-allergy-domain
oya-emr-vital-kernel                oya-emr-vital-domain
oya-emr-note-kernel                 oya-emr-note-domain
oya-emr-order-kernel                oya-emr-order-domain
oya-emr-result-kernel               oya-emr-result-domain
oya-emr-care-team-kernel            oya-emr-care-team-domain
oya-emr-order-set-kernel            oya-emr-order-set-domain
oya-emr-documentation-kernel        oya-emr-documentation-domain
oya-emr-billing-code-kernel         oya-emr-billing-code-domain
oya-emr-patient-education-kernel    oya-emr-patient-education-domain
oya-emr-portal-session-kernel       oya-emr-portal-session-domain
```

Each kernel crate carries:

- Newtype value-objects (e.g., `PatientId`, `EncounterId`, `Mrn`, `Dob`).
- Port trait definitions (`PatientRepository`, etc.).
- Invariant validators (e.g., `Dob::new` refuses future dates).

Each domain crate carries:

- Aggregates + entities (`Patient`, `Encounter`, `Problem`, `Medication`, …).
- Value-objects (composite types per BC).
- Domain services (e.g., `MedicationReconciliation::reconcile`).

## Acceptance criteria

- 30 crates exist with valid Cargo.toml.
- All 30 crates `cargo check -p` exit 0.
- All 30 crates have at least one `#[cfg(test)]` unit test.
- `[workspace.members]` in root `Cargo.toml` references the new paths.
- `oya gate validate per-microservice-layout --microservice emr` exits 0.

## Out of scope

- Use-case orchestration (IP-EMR-002).
- Adapters (IP-EMR-003).
- REST routing (IP-EMR-004).
