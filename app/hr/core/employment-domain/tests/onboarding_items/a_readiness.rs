fn valid_evidence(suffix: &str) -> AuditEvidenceRef {
    AuditEvidenceRef {
        value: format!("audit/hr/onboarding/{suffix}"),
    }
}

/// Returns an input where every mandatory item is cleared with evidence.
fn all_cleared_input() -> OnboardingReadinessInput {
    OnboardingReadinessInput {
        employee_id: "emp_001".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        evaluated_at_epoch_seconds: 1_700_000_000,
        checklist: vec![
            OnboardingChecklistItem {
                kind: OnboardingChecklistItemKind::RightToWorkI9,
                is_mandatory: true,
                is_cleared: true,
                evidence_ref: Some(valid_evidence("rtw/001")),
            },
            OnboardingChecklistItem {
                kind: OnboardingChecklistItemKind::BackgroundCheck,
                is_mandatory: true,
                is_cleared: true,
                evidence_ref: Some(valid_evidence("bgcheck/001")),
            },
            OnboardingChecklistItem {
                kind: OnboardingChecklistItemKind::EquipmentProvisioning,
                is_mandatory: false,
                is_cleared: true,
                evidence_ref: Some(valid_evidence("equip/001")),
            },
            OnboardingChecklistItem {
                kind: OnboardingChecklistItemKind::AccessGrant,
                is_mandatory: false,
                is_cleared: true,
                evidence_ref: Some(valid_evidence("access/001")),
            },
            OnboardingChecklistItem {
                kind: OnboardingChecklistItemKind::MandatoryTraining,
                is_mandatory: true,
                is_cleared: true,
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

    assert_eq!(result.decision.value, OnboardingDecision::Ready);
    assert!(
        result.outstanding_items.value.is_empty(),
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
        .checklist
        .retain(|i| i.kind != OnboardingChecklistItemKind::BackgroundCheck);

    let result = evaluate_onboarding_readiness(input)
        .expect("should not error — missing item is a NOT_READY, not an Err");

    assert_eq!(result.decision.value, OnboardingDecision::NotReady);
    assert!(
        result
            .outstanding_items
            .value
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
    // Mark RightToWorkI9 as not cleared and strip its evidence
    for item in &mut input.checklist {
        if item.kind == OnboardingChecklistItemKind::RightToWorkI9 {
            item.is_cleared = false;
            item.evidence_ref = None;
        }
    }

    let result = evaluate_onboarding_readiness(input)
        .expect("uncleared item yields NOT_READY decision, not Err");

    assert_eq!(result.decision.value, OnboardingDecision::NotReady);
    assert!(
        result
            .outstanding_items
            .value
            .contains(&OnboardingChecklistItemKind::RightToWorkI9),
        "RightToWorkI9 must be listed as an outstanding item"
    );
}

// ---------------------------------------------------------------------------
// [st3] Test: cleared mandatory item with no evidence → OnboardingItemNotCleared
// ---------------------------------------------------------------------------

#[test]
fn only_optional_items_all_cleared_returns_ready() {
    let input = OnboardingReadinessInput {
        employee_id: "emp_001".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        evaluated_at_epoch_seconds: 1_700_000_000,
        checklist: vec![
            OnboardingChecklistItem {
                kind: OnboardingChecklistItemKind::EquipmentProvisioning,
                is_mandatory: false,
                is_cleared: true,
                evidence_ref: Some(valid_evidence("equip/001")),
            },
            OnboardingChecklistItem {
                kind: OnboardingChecklistItemKind::AccessGrant,
                is_mandatory: false,
                is_cleared: false,
                evidence_ref: None,
            },
        ],
    };

    let result = evaluate_onboarding_readiness(input)
        .expect("optional-only checklist with no mandatory items should not error");

    assert_eq!(
        result.decision.value,
        OnboardingDecision::Ready,
        "no mandatory items means nothing blocks READY"
    );
    assert!(result.outstanding_items.value.is_empty());
}

// ---------------------------------------------------------------------------
// [st3] Test: invalid evidence ref on cleared mandatory item → error
// ---------------------------------------------------------------------------

#[test]
fn multiple_uncleared_mandatory_items_lists_all_blockers() {
    let mut input = all_cleared_input();
    for item in &mut input.checklist {
        if matches!(
            item.kind,
            OnboardingChecklistItemKind::RightToWorkI9
                | OnboardingChecklistItemKind::BackgroundCheck
        ) {
            item.is_cleared = false;
            item.evidence_ref = None;
        }
    }

    let result = evaluate_onboarding_readiness(input)
        .expect("multiple uncleared items yield NOT_READY, not Err");

    assert_eq!(result.decision.value, OnboardingDecision::NotReady);
    assert!(
        result
            .outstanding_items
            .value
            .contains(&OnboardingChecklistItemKind::RightToWorkI9),
        "RightToWorkI9 must be in blockers"
    );
    assert!(
        result
            .outstanding_items
            .value
            .contains(&OnboardingChecklistItemKind::BackgroundCheck),
        "BackgroundCheck must be in blockers"
    );
    assert_eq!(
        result.outstanding_items.value.len(),
        2,
        "exactly two blockers"
    );
}

// ---------------------------------------------------------------------------
// [st3] Test: zero evaluated_at_epoch_seconds → InvalidEvaluatedAt error
// ---------------------------------------------------------------------------
