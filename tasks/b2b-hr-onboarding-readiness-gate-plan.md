# Plan: b2b-hr-onboarding-readiness-gate

**Vertical:** b2b  
**Crate:** `oya-hr-employment-domain`  
**Branch:** `feat/task-b2b-hr-onboarding-readiness-gate-2026-05-28`

---

## Subtasks

### [st1] Add onboarding domain model to `crates/oya-hr-employment-domain/src/lib.rs`

Add the following types alongside existing domain types:

- `OnboardingChecklistItemKind` enum: `RightToWorkI9`, `BackgroundCheck`, `EquipmentProvisioning`, `AccessGrant`, `MandatoryTraining`
- `OnboardingChecklistItem` struct: `kind`, `is_mandatory: bool`, `is_cleared: bool`, `evidence_ref: Option<AuditEvidenceRef>`
- `OnboardingReadinessInput` struct: raw string identifiers + `Vec<OnboardingChecklistItem>` + `evaluated_at_epoch_seconds`
- `OnboardingReadinessDecision` struct: Classified fields wrapping validated newtypes + `decision: Classified<OnboardingDecision>` + `outstanding_items: Classified<Vec<OnboardingChecklistItemKind>>` + `schema_version`
- `OnboardingDecision` enum: `Ready`, `NotReady`
- New `HrDomainError` variants: `OnboardingItemsRequired`, `OnboardingItemNotCleared`, `DuplicateOnboardingItem`
- New constant: `ONBOARDING_READINESS_SCHEMA_VERSION: u32 = 1`

**Acceptance:** `cargo check -p oya-hr-employment-domain --all-targets` clean; new types carry `data_class` annotations consistent with existing structs; reuse `EmployeeId`, `TenantId`, `LegalEntityId`, `AuditEvidenceRef` newtypes.

---

### [st2] Implement `evaluate_onboarding_readiness` pure evaluator fn

Pure fn `evaluate_onboarding_readiness(input: OnboardingReadinessInput) -> Result<OnboardingReadinessDecision, HrDomainError>`:

1. Validate `employee_id`, `tenant_id`, `legal_entity_id` using existing `validate_identifier` helpers.
2. Reject empty checklist: return `Err(HrDomainError::OnboardingItemsRequired)`.
3. Reject duplicate item kinds: return `Err(HrDomainError::DuplicateOnboardingItem)`.
4. Reject `evaluated_at_epoch_seconds == 0`: return `Err(HrDomainError::InvalidEvaluatedAt)`.
5. Collect mandatory uncleared items into `outstanding_items`.
6. If any mandatory item is cleared but has no evidence ref: return `Err(HrDomainError::OnboardingItemNotCleared)` (evidence required for cleared mandatory items).
7. If `outstanding_items` is empty: return `Ready` decision with empty `outstanding_items`.
8. Otherwise: return `NotReady` decision enumerating `outstanding_items`.
9. Never panics; returns `Result<_, HrDomainError>`.

**Acceptance:** READY only when every mandatory item is cleared with valid evidence; missing/uncleared mandatory item yields deterministic NOT_READY decision listing blockers; invalid IDs return matching `HrDomainError`.

---

### [st3] Add `tests/onboarding.rs`

Follow the `tests/<topic>.rs` convention (file at `crates/oya-hr-employment-domain/tests/onboarding.rs`).

Cases:
1. All mandatory items cleared with evidence → `OnboardingDecision::Ready`
2. Missing mandatory item (item absent from list) → `OnboardingDecision::NotReady` with blocker in `outstanding_items`
3. Mandatory item present but `is_cleared = false` → `OnboardingDecision::NotReady`
4. Empty checklist → `Err(HrDomainError::OnboardingItemsRequired)`
5. Invalid `employee_id` → `Err(HrDomainError::InvalidEmployeeId)`
6. Invalid `tenant_id` → `Err(HrDomainError::InvalidTenantId)`
7. Invalid `legal_entity_id` → `Err(HrDomainError::InvalidLegalEntityId)`
8. Duplicate item kinds → `Err(HrDomainError::DuplicateOnboardingItem)`
9. Cleared mandatory item missing evidence ref → `Err(HrDomainError::OnboardingItemNotCleared)`
10. Optional (non-mandatory) item uncleared does not block READY

**Acceptance:** `cargo nextest run -p oya-hr-employment-domain` passes including new onboarding tests; pre-existing employment/leave/privacy/compliance tests remain green.

---

## Acceptance Summary

| Subtask | Gate |
|---------|------|
| st1 | `cargo check -p oya-hr-employment-domain --all-targets` clean |
| st2 | Evaluator READY/NOT_READY/Err paths deterministic |
| st3 | `cargo nextest run -p oya-hr-employment-domain` all green |
