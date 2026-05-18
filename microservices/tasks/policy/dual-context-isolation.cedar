// Cedar policy: dual-context-isolation
// Microservice: tasks
// Purpose: enforce Personal vs Professional dual-context isolation + EU AI Act
// Annex III §4 employment-context auto-assign refusal until conformity assessment.
// Owner: ops-security + axis-tasks + council-privacy
// Status: Accepted (2026-05-17)
// Related: ADR-0140 (retired per ADR-0145), ADR-TASKS-0006, policy/task-isolation.md
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =====================================================================================
// DUAL-CONTEXT REFUSAL: Personal reader ↛ Professional; Professional reader ↛ Personal.
// =====================================================================================

forbid (
  principal in TasksRole::"professional_reader",
  action == Action::"task_read",
  resource in TaskContext::"Personal"
);

forbid (
  principal in TasksRole::"personal_reader",
  action == Action::"task_read",
  resource in TaskContext::"Professional"
);

forbid (
  principal in TasksRole::"professional_reader",
  action in [
    Action::"task_write",
    Action::"task_assign",
    Action::"task_comment",
    Action::"task_dependency_add"
  ],
  resource in TaskContext::"Personal"
);

forbid (
  principal in TasksRole::"personal_reader",
  action in [
    Action::"task_write",
    Action::"task_assign",
    Action::"task_comment",
    Action::"task_dependency_add"
  ],
  resource in TaskContext::"Professional"
);

// =====================================================================================
// CONTEXT-BRIDGE: permitted only at field-level grant.
// =====================================================================================

permit (
  principal in TasksRole::"context_bridge_reader",
  action == Action::"task_read",
  resource
)
when {
  context.personal_to_professional_grant.field == resource.field_name &&
  context.personal_to_professional_grant.consent_at <= context.now &&
  context.personal_to_professional_grant.revoked_at == null
};

// =====================================================================================
// EU AI ACT ANNEX III §4 — T2 auto-assign in employment-context REFUSED until conformity.
// (ADR-TASKS-0006)
// =====================================================================================

// Refuse T2 auto-assign in EU pack employment context unconditionally
// until ADR-TASKS-0006 conformity-assessment ADR ships per pack.
forbid (
  principal in TasksRole::"foundry_t2_classifier",
  action == Action::"task_t2_auto_assign",
  resource
)
when {
  resource.pack_tag == "pack-eu" &&
  resource.context_kind == "employment"
};

// Refuse T2 auto-assign in US pack employment context until fairness-audit complete.
forbid (
  principal in TasksRole::"foundry_t2_classifier",
  action == Action::"task_t2_auto_assign",
  resource
)
when {
  resource.pack_tag == "pack-us" &&
  resource.context_kind == "employment" &&
  context.fairness_audit_complete == false
};

// Refuse T2 auto-assign in pack-us-NY for AEDT until NY DCWP audit complete.
forbid (
  principal in TasksRole::"foundry_t2_classifier",
  action == Action::"task_t2_auto_assign",
  resource
)
when {
  resource.pack_tag == "pack-us" &&
  resource.state == "NY" &&
  context.ny_local_law_144_aedt_audit_complete == false
};

// Refuse T2 auto-assign in KR employment context until 직장 갑질 protections compliance.
forbid (
  principal in TasksRole::"foundry_t2_classifier",
  action == Action::"task_t2_auto_assign",
  resource
)
when {
  resource.pack_tag == "pack-kr" &&
  resource.context_kind == "employment"
};

// Refuse T2 auto-assign in JP employment context pending Labour Standards Act review.
forbid (
  principal in TasksRole::"foundry_t2_classifier",
  action == Action::"task_t2_auto_assign",
  resource
)
when {
  resource.pack_tag == "pack-jp" &&
  resource.context_kind == "employment"
};

// Refuse T2 auto-assign in AU employment context pending Fair Work Act review.
forbid (
  principal in TasksRole::"foundry_t2_classifier",
  action == Action::"task_t2_auto_assign",
  resource
)
when {
  resource.pack_tag == "pack-au" &&
  resource.context_kind == "employment"
};

// =====================================================================================
// EU AI ACT ART. 14 HUMAN OVERSIGHT — T2 actions require 30s reversibility window
// when permitted.
// =====================================================================================

permit (
  principal in TasksRole::"foundry_t2_classifier",
  action == Action::"task_t2_auto_assign",
  resource
)
when {
  // Outside employment-context: T2 auto-assign permitted (e.g., personal project work).
  resource.context_kind != "employment" &&
  // EU AI Act Art. 14 human oversight: reversibility window 30s.
  context.reversibility_window_seconds == 30 &&
  // EU AI Act Art. 50 transparency: user labelled.
  context.user_label_shown == true &&
  // Per-decision audit-chain seal computed.
  context.audit_chain_pre_emitted == true
};

// =====================================================================================
// EU AI ACT ART. 50 — Every T0/T1/T2 AI surface MUST be user-labelled.
// =====================================================================================

forbid (
  principal in TasksRole::"foundry_classifier",
  action in [
    Action::"task_t0_suggest",
    Action::"task_t1_priority_suggest",
    Action::"task_t1_auto_categorise"
  ],
  resource
)
when {
  context.user_label_shown == false
};

// =====================================================================================
// FOUNDRY-RUNTIME BRIDGE — every AI invocation flows through Cedar admission.
// (ADR-TASKS-0006 + threat-model.md T-E-04 mitigation)
// =====================================================================================

forbid (
  principal in TasksRole::"foundry_classifier",
  action,
  resource
)
when {
  // No AI action permitted on Personal context without explicit user-opt-in.
  resource.context_kind == "Personal" &&
  context.user_ai_opt_in == false
};

// =====================================================================================
// CONTEXT IMMUTABILITY: task context cannot be flipped post-creation.
// =====================================================================================

forbid (
  principal,
  action == Action::"task_context_change",
  resource
);

// =====================================================================================
// LEAN check: `oya-check-context-isolation` validates no usecase queries both contexts
// in a single transaction. `oya-check-foundry-bridge-cedar-gate` validates AI surfaces
// route through this Cedar policy. `oya-check-auto-assign-fairness` (NEW per ADR-TASKS-
// 0006) validates fairness-audit complete before pack-us T2 auto-assign permitted.
// =====================================================================================
