#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_hr_employment_domain::{
    HrDomainError, OnboardingChecklistItem, OnboardingChecklistItemKind, OnboardingReadinessInput,
    OnboardingReadinessOutcome, evaluate_onboarding_readiness,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn valid_evidence(suffix: &str) -> String {
    format!("audit/hr/onboarding/{suffix}")
}

/// Returns an input where every mandatory item is cleared with evidence.
fn all_cleared_input() -> OnboardingReadinessInput {
    OnboardingReadinessInput {
        employee_id: "emp_001".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        checklist_items: vec![
            OnboardingChecklistItem {
                kind: OnboardingChecklistItemKind::RightToWork,
                is_mandatory: true,
                cleared: true,
                evidence_ref: Some(valid_evidence("rtw/001")),
            },
            OnboardingChecklistItem {
                kind: OnboardingChecklistItemKind::BackgroundCheck,
                is_mandatory: true,
                cleared: true,
                evidence_ref: Some(valid_evidence("bgcheck/001")),
            },
            OnboardingChecklistItem {
                kind: OnboardingChecklistItemKind::EquipmentProvisioning,
                is_mandatory: false,
                cleared: true,
                evidence_ref: Some(valid_evidence("equip/001")),
            },
            OnboardingChecklistItem {
                kind: OnboardingChecklistItemKind::AccessGrant,
                is_mandatory: false,
                cleared: true,
                evidence_ref: Some(valid_evidence("access/001")),
            },
            OnboardingChecklistItem {
                kind: OnboardingChecklistItemKind::MandatoryTraining,
                is_mandatory: true,
                cleared: true,
                evidence_ref: Some(valid_evidence("training/001")),
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// [st3] Test: all mandatory items cleared → READY decision
// ---------------------------------------------------------------------------

#[test]
fn all_mandatory_items_cleared_returns_ready_decision() {
    let result = evaluate_onboarding_readiness(all_cleared_input())
        .expect("should not error when all items are cleared");

    assert_eq!(result.outcome, OnboardingReadinessOutcome::Ready);
    assert!(
        result.outstanding_mandatory_items.is_empty(),
        "READY decision must have no outstanding items"
    );
}

// ---------------------------------------------------------------------------
// [st3] Test: missing mandatory item → NOT_READY with blocker list
// ---------------------------------------------------------------------------

#[test]
fn missing_mandatory_item_returns_not_ready_with_blocker_list() {
    let mut input = all_cleared_input();
    // Remove the BackgroundCheck item entirely
    input
        .checklist_items
        .retain(|i| i.kind != OnboardingChecklistItemKind::BackgroundCheck);

    let result = evaluate_onboarding_readiness(input)
        .expect("should not error — missing item is a NOT_READY, not an Err");

    assert_eq!(result.outcome, OnboardingReadinessOutcome::NotReady);
    assert!(
        result
            .outstanding_mandatory_items
            .contains(&OnboardingChecklistItemKind::BackgroundCheck),
        "BackgroundCheck must appear in the blocker list"
    );
}

// ---------------------------------------------------------------------------
// [st3] Test: uncleared mandatory item → NOT_READY
// ---------------------------------------------------------------------------

#[test]
fn uncleared_mandatory_item_returns_not_ready() {
    let mut input = all_cleared_input();
    // Mark RightToWork as not cleared and strip its evidence
    for item in &mut input.checklist_items {
        if item.kind == OnboardingChecklistItemKind::RightToWork {
            item.cleared = false;
            item.evidence_ref = None;
        }
    }

    let result = evaluate_onboarding_readiness(input)
        .expect("uncleared item yields NOT_READY decision, not Err");

    assert_eq!(result.outcome, OnboardingReadinessOutcome::NotReady);
    assert!(
        result
            .outstanding_mandatory_items
            .contains(&OnboardingChecklistItemKind::RightToWork),
        "RightToWork must be listed as an outstanding item"
    );
}

// ---------------------------------------------------------------------------
// [st3] Test: mandatory item cleared without evidence → NOT_READY
// ---------------------------------------------------------------------------

#[test]
fn mandatory_item_cleared_without_evidence_returns_not_ready() {
    let mut input = all_cleared_input();
    for item in &mut input.checklist_items {
        if item.kind == OnboardingChecklistItemKind::MandatoryTraining {
            item.cleared = true;
            item.evidence_ref = None; // cleared flag set but no AuditEvidenceRef
        }
    }

    let result = evaluate_onboarding_readiness(input)
        .expect("missing evidence on cleared item yields NOT_READY, not Err");

    assert_eq!(result.outcome, OnboardingReadinessOutcome::NotReady);
    assert!(
        result
            .outstanding_mandatory_items
            .contains(&OnboardingChecklistItemKind::MandatoryTraining),
        "MandatoryTraining without evidence must appear as a blocker"
    );
}

// ---------------------------------------------------------------------------
// [st3] Test: empty checklist → OnboardingItemsRequired error
// ---------------------------------------------------------------------------

#[test]
fn empty_checklist_returns_onboarding_items_required_error() {
    let input = OnboardingReadinessInput {
        employee_id: "emp_001".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        checklist_items: vec![],
    };

    let err = evaluate_onboarding_readiness(input)
        .expect_err("empty checklist must return an error");

    assert_eq!(err, HrDomainError::OnboardingItemsRequired);
}

// ---------------------------------------------------------------------------
// [st3] Test: duplicate checklist item kinds → OnboardingItemNotCleared error
// ---------------------------------------------------------------------------

#[test]
fn duplicate_checklist_item_kinds_returns_error() {
    let mut input = all_cleared_input();
    // Duplicate RightToWork
    input.checklist_items.push(OnboardingChecklistItem {
        kind: OnboardingChecklistItemKind::RightToWork,
        is_mandatory: true,
        cleared: true,
        evidence_ref: Some(valid_evidence("rtw/duplicate")),
    });

    let err = evaluate_onboarding_readiness(input)
        .expect_err("duplicate item kinds must return an error");

    assert_eq!(err, HrDomainError::OnboardingDuplicateItem);
}

// ---------------------------------------------------------------------------
// [st3] Test: invalid employee_id → InvalidEmployeeId error
// ---------------------------------------------------------------------------

#[test]
fn invalid_employee_id_returns_invalid_employee_id_error() {
    let input = OnboardingReadinessInput {
        employee_id: "not-prefixed".to_owned(),
        ..all_cleared_input()
    };

    let err = evaluate_onboarding_readiness(input)
        .expect_err("invalid employee_id must return InvalidEmployeeId");

    assert_eq!(err, HrDomainError::InvalidEmployeeId);
}

// ---------------------------------------------------------------------------
// [st3] Test: invalid tenant_id → InvalidTenantId error
// ---------------------------------------------------------------------------

#[test]
fn invalid_tenant_id_returns_invalid_tenant_id_error() {
    let input = OnboardingReadinessInput {
        tenant_id: "ten_".to_owned(), // prefix-only, empty suffix
        ..all_cleared_input()
    };

    let err = evaluate_onboarding_readiness(input)
        .expect_err("prefix-only tenant_id must return InvalidTenantId");

    assert_eq!(err, HrDomainError::InvalidTenantId);
}

// ---------------------------------------------------------------------------
// [st3] Test: invalid legal_entity_id → InvalidLegalEntityId error
// ---------------------------------------------------------------------------

#[test]
fn invalid_legal_entity_id_returns_invalid_legal_entity_id_error() {
    let input = OnboardingReadinessInput {
        legal_entity_id: "le_../escape".to_owned(),
        ..all_cleared_input()
    };

    let err = evaluate_onboarding_readiness(input)
        .expect_err("path-traversal legal_entity_id must return InvalidLegalEntityId");

    assert_eq!(err, HrDomainError::InvalidLegalEntityId);
}

// ---------------------------------------------------------------------------
// [st3] Test: only optional items present (no mandatory items) → READY
//            Optional items do not block the transition; an input with zero
//            mandatory items but non-empty checklist is allowed to be READY.
// ---------------------------------------------------------------------------

#[test]
fn only_optional_items_all_cleared_returns_ready() {
    let input = OnboardingReadinessInput {
        employee_id: "emp_001".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        checklist_items: vec![
            OnboardingChecklistItem {
                kind: OnboardingChecklistItemKind::EquipmentProvisioning,
                is_mandatory: false,
                cleared: true,
                evidence_ref: Some(valid_evidence("equip/001")),
            },
            OnboardingChecklistItem {
                kind: OnboardingChecklistItemKind::AccessGrant,
                is_mandatory: false,
                cleared: false,
                evidence_ref: None,
            },
        ],
    };

    let result = evaluate_onboarding_readiness(input)
        .expect("optional-only checklist with no mandatory items should not error");

    assert_eq!(
        result.outcome,
        OnboardingReadinessOutcome::Ready,
        "no mandatory items means nothing blocks READY"
    );
    assert!(result.outstanding_mandatory_items.is_empty());
}

// ---------------------------------------------------------------------------
// [st3] Test: invalid evidence ref on cleared mandatory item → error
// ---------------------------------------------------------------------------

#[test]
fn invalid_evidence_ref_on_cleared_mandatory_item_returns_error() {
    let mut input = all_cleared_input();
    for item in &mut input.checklist_items {
        if item.kind == OnboardingChecklistItemKind::RightToWork {
            item.evidence_ref = Some("audit/hr/onboarding/bearer-token".to_owned());
        }
    }

    let err = evaluate_onboarding_readiness(input)
        .expect_err("credential-shaped evidence ref must be rejected");

    assert_eq!(err, HrDomainError::InvalidAuditEvidenceRef);
}

// ---------------------------------------------------------------------------
// [st3] Test: multiple uncleared mandatory items → NOT_READY lists all blockers
// ---------------------------------------------------------------------------

#[test]
fn multiple_uncleared_mandatory_items_lists_all_blockers() {
    let mut input = all_cleared_input();
    for item in &mut input.checklist_items {
        if matches!(
            item.kind,
            OnboardingChecklistItemKind::RightToWork | OnboardingChecklistItemKind::BackgroundCheck
        ) {
            item.cleared = false;
            item.evidence_ref = None;
        }
    }

    let result = evaluate_onboarding_readiness(input)
        .expect("multiple uncleared items yield NOT_READY, not Err");

    assert_eq!(result.outcome, OnboardingReadinessOutcome::NotReady);
    assert!(
        result
            .outstanding_mandatory_items
            .contains(&OnboardingChecklistItemKind::RightToWork),
        "RightToWork must be in blockers"
    );
    assert!(
        result
            .outstanding_mandatory_items
            .contains(&OnboardingChecklistItemKind::BackgroundCheck),
        "BackgroundCheck must be in blockers"
    );
    assert_eq!(
        result.outstanding_mandatory_items.len(),
        2,
        "exactly two blockers"
    );
}
