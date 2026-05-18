// anonymous µservice — legal-process disclosure policy fragment
// Cedar v4.2 LTS; default-deny.
//
// Purpose: gate the legal-process disclosure workflow (court-order receipt,
//          dual-control approval, 14-day notice, audit-chain seal,
//          transparency-report inclusion). This is the ONLY policy fragment
//          that can authorise platform-side correlation of user_id ↔ post_id.
//          Every other Cedar policy in this µservice FORBIDS that action.
//
// Owner:   ops-security + general-counsel + council-privacy
// Audit:   every action emits an audit-chain seal with chain-of-custody hash
// References:
//   - ADR-ANON-0003 (legal-process workflow)
//   - PRD-anonymous I7 invariant
//   - ECPA / SCA §§2510-2523, 2701-2712 (US)
//   - UK Investigatory Powers Act 2016 §§70-71
//   - KR 통신비밀보호법 Art. 9 (and Art. 9-2 gag-order doctrine)
//   - EU DSA Art. 27 (transparency-report inclusion)

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — court-order intake (recording only; no disclosure yet)
// =============================================================================
//
// A `legal_process_intake` principal (typically a member of the legal-process
// intake team) records the court order. Recording does NOT itself authorise
// disclosure; it is a workflow precondition.

permit (
  principal in LegalProcessIntake::?p,
  action == Action::"record_court_order",
  resource in LegalProcessOrder::?r
)
when {
  principal has legal_process_intake_entitlement &&
  principal.legal_process_intake_entitlement == true &&
  resource has court_order_doctype &&
  resource.court_order_doctype in [
    "subpoena",
    "warrant",
    "court_order",
    "preservation_request",
    "ncmec_cybertipline_request",
    "ipa_2016_targeted_interception_warrant",
    "ipa_2016_targeted_equipment_interference_warrant",
    "kr_telecom_secrecy_act_warrant",
    "jp_telecom_secrecy_court_order"
  ]
};

// =============================================================================
// PERMIT 2 — dual-control approval (TWO DISTINCT principals required)
// =============================================================================
//
// The disclosure workflow requires two distinct principals, each holding the
// `legal_process_approver` entitlement, and each from a distinct organisational
// unit (per the two-person-rule industry pattern + ADR-ANON-0003 §"Dual-Control").

permit (
  principal in LegalProcessApprover::?p,
  action == Action::"sign_dual_control_approval",
  resource in LegalProcessOrder::?r
)
when {
  principal has legal_process_approver_entitlement &&
  principal.legal_process_approver_entitlement == true &&
  principal has organizational_unit &&
  resource has approvals &&
  resource.approvals has count &&
  resource.approvals.count < 2 &&
  // Defence-in-depth: principal MUST NOT be the same as any existing approver
  // (Cedar at the policy layer; the workflow engine also enforces uniqueness)
  principal has approver_id &&
  !(principal.approver_id in resource.approvals)
};

// =============================================================================
// PERMIT 3 — disclosure execution (ONLY after dual-control + 14d notice OR
//            court-prohibited gag-order)
// =============================================================================
//
// The actual disclosure (correlate user_id ↔ post_id, package, deliver to
// law enforcement) is permitted ONLY when:
//
// (a) dual-control approval is recorded (2 distinct approvers); AND
// (b) EITHER the 14-day end-user notice has been served (ECPA §2705(a))
//     OR a court-issued gag-order is on record (ECPA §2705(b) / IPA 2016 §57 /
//     KR 통신비밀보호법 Art. 9-2 / JP Telecom Secrecy Act).

permit (
  principal in LegalProcessExecutor::?p,
  action == Action::"execute_disclosure",
  resource in LegalProcessOrder::?r
)
when {
  principal has legal_process_executor_entitlement &&
  principal.legal_process_executor_entitlement == true &&
  resource has approvals &&
  resource.approvals has count &&
  resource.approvals.count >= 2 &&
  resource has approvals_distinct_org_units &&
  resource.approvals_distinct_org_units == true &&
  resource has user_notice_status &&
  (
    resource.user_notice_status == "served_14d_elapsed" ||
    resource.user_notice_status == "court_prohibited_gag_order"
  ) &&
  resource has chain_of_custody_initialized &&
  resource.chain_of_custody_initialized == true
};

// =============================================================================
// PERMIT 4 — disclosure-package sealing (audit-chain seal)
// =============================================================================

permit (
  principal in LegalProcessExecutor::?p,
  action == Action::"seal_disclosure_package",
  resource in DisclosurePackage::?r
)
when {
  principal has legal_process_executor_entitlement &&
  resource has order_ref &&
  resource has chain_of_custody_hash
};

// =============================================================================
// PERMIT 5 — transparency-report aggregator reads aggregate counts
// =============================================================================

permit (
  principal == TransparencyReportAggregator::"system",
  action == Action::"read_aggregate_disclosure_count",
  resource in LegalProcessOrder::?r
);

// =============================================================================
// FORBID — single-approver disclosure (dual-control bypass attempt)
// =============================================================================

forbid (
  principal,
  action == Action::"execute_disclosure",
  resource in LegalProcessOrder::?r
)
unless {
  resource has approvals &&
  resource.approvals has count &&
  resource.approvals.count >= 2 &&
  resource.approvals_distinct_org_units == true
};

// =============================================================================
// FORBID — disclosure without chain-of-custody init
// =============================================================================

forbid (
  principal,
  action == Action::"execute_disclosure",
  resource in LegalProcessOrder::?r
)
unless {
  resource has chain_of_custody_initialized &&
  resource.chain_of_custody_initialized == true
};

// =============================================================================
// FORBID — same-principal acts as both approvers (two-person rule)
// =============================================================================
//
// Even if a principal has two distinct entitlement claims, they cannot sign
// both approvals on the same order. The workflow engine enforces uniqueness
// at runtime; Cedar enforces it here at the policy layer.

forbid (
  principal,
  action == Action::"sign_dual_control_approval",
  resource in LegalProcessOrder::?r
)
when {
  resource has approvals &&
  principal has approver_id &&
  principal.approver_id in resource.approvals
};
