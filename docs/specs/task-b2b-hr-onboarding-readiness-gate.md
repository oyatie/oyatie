# Spec: b2b-hr-onboarding-readiness-gate

**Vertical:** b2b  
**Lane:** b2b-hr-onboarding-readiness-gate  
**Crate (sole owner):** `hr-employment-domain`  
**ADR authority:** ADR-0509 (flat single-crate per service, mod-based subsystems)  
**Stage authored:** SPEC (2026-05-29)

---

## Objective

Extend the `hr-employment-domain` pure-domain crate with a pre-hire onboarding readiness slice. The slice models a checklist of mandatory onboarding items (right-to-work/I-9 verification, background-check clearance, equipment and access provisioning, mandatory training) and a day-one gate that evaluates whether all mandatory items are cleared with audit evidence before an employee may transition to EMPLOYED. No storage, REST, gRPC, workflow dispatch, or new workspace members are introduced.

---

## Vertical & Boundary

- **Vertical:** b2b HR — employment lifecycle sub-domain
- **Domain boundary:** Pure invariant evaluation; no I/O, no async, no persistence
- **Reuses:** `EmployeeId`, `TenantId`, `LegalEntityId`, `AuditEvidenceRef` newtypes; `Classified<T>` wrappers; `HrDomainError`; `validate_identifier` / `validate_evidence_ref` helpers
- **Does NOT touch:** REST adapters, gRPC protos, payroll, leave, root `Cargo.toml`, any other crate

---

## Contracts

### Domain model (Rust types — no REST/proto surface in this slice)

This is a domain-only slice. No OpenAPI or proto3 contract is published at this stage. The evaluator function signature is the contract surface:

```rust
pub fn evaluate_onboarding_readiness(
    input: OnboardingReadinessInput,
) -> Result<OnboardingReadinessDecision, HrDomainError>
```

Input fields (all validated, no raw strings escape as newtypes):
- `employee_id: String` — validated via `emp_` prefix
- `tenant_id: String` — validated via `ten_` prefix
- `legal_entity_id: String` — validated via `le_` prefix
- `checklist: Vec<OnboardingChecklistItem>` — non-empty, no duplicate kinds
- `evaluated_at_epoch_seconds: u64` — non-zero

Output (`OnboardingReadinessDecision`):
- `decision: Classified<OnboardingDecision>` — `Ready` | `NotReady`
- `outstanding_items: Classified<Vec<OnboardingChecklistItemKind>>` — empty on READY, blockers on NOT_READY
- All identifier fields wrapped in `Classified<T>` with appropriate `data_class`
- `schema_version: Classified<u32>` tagged PUBLIC

Error variants added to `HrDomainError`:
- `OnboardingItemsRequired` — empty checklist supplied
- `DuplicateOnboardingItem` — same kind appears more than once
- `OnboardingItemNotCleared` — mandatory item marked cleared but no evidence ref

---

## Module Layout (flat clean-arch, ADR-0509)

All new code lives inside `crates/hr-employment-domain/src/lib.rs` following the codebase pattern of a single flat file with domain types + evaluator fns. No new modules or files are introduced in `src/`.

```
crates/hr-employment-domain/
  src/
    lib.rs          ← new types + evaluate_onboarding_readiness fn appended here
  tests/
    employment.rs   (existing)
    kr_council.rs   (existing)
    kr_rules.rs     (existing)
    leave_balance.rs (existing)
    leave.rs        (existing)
    privacy.rs      (existing)
    rulepack_manifest.rs (existing)
    onboarding.rs   ← NEW: integration tests for this slice
```

---

## Data Class Annotations

| Field | data_class |
|-------|-----------|
| `employee_id` | `INTERNAL_ONLY` |
| `tenant_id` | `INTERNAL_ONLY` |
| `legal_entity_id` | `INTERNAL_ONLY` |
| `checklist` / item kinds | `INTERNAL_ONLY` |
| `evidence_ref` within items | `INTERNAL_ONLY` |
| `decision` | `INTERNAL_ONLY` |
| `outstanding_items` | `INTERNAL_ONLY` |
| `evaluated_at_epoch_seconds` | `INTERNAL_ONLY` |
| `schema_version` | `PUBLIC` |

`OnboardingChecklistItem.evidence_ref` is `Option<AuditEvidenceRef>` — for cleared mandatory items a non-`None` validated ref is required. Optional items may omit evidence.

---

## Testing Strategy

File: `crates/hr-employment-domain/tests/onboarding.rs`

| Case | Expected outcome |
|------|-----------------|
| All mandatory items cleared + valid evidence | `Ok(decision = Ready, outstanding_items = [])` |
| One mandatory item absent | `Ok(decision = NotReady, outstanding_items = [<missing kind>])` |
| Mandatory item present, `is_cleared = false` | `Ok(decision = NotReady, outstanding_items = [<kind>])` |
| Optional item uncleared | `Ok(decision = Ready, ...)` — does not block |
| Empty checklist | `Err(OnboardingItemsRequired)` |
| Duplicate item kind in checklist | `Err(DuplicateOnboardingItem)` |
| Invalid `employee_id` | `Err(InvalidEmployeeId)` |
| Invalid `tenant_id` | `Err(InvalidTenantId)` |
| Invalid `legal_entity_id` | `Err(InvalidLegalEntityId)` |
| Cleared mandatory item, `evidence_ref = None` | `Err(OnboardingItemNotCleared)` |

Gate command: `cargo nextest run -p hr-employment-domain`

---

## Boundaries / Out of Scope

- No EMPLOYED-status transition write (domain evaluation only; no state machine mutation)
- No persistence or event emission
- No REST or gRPC layer
- No new crate or workspace member
- No root `Cargo.toml` edit
- No changes to any other crate
