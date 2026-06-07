// CN-PIPL-2021 Cedar Fragment: dsar-response
// fragment_id: pack-cn-pipl-2021-dsar-response
// version: 1.0.0
// binding_adr: ADR-0251
// regulation: PIPL Article 45 — data subjects have the right to access, copy, and port their
//             personal information. Controllers must respond within 15 business days.
//             PIPL Article 47 — erasure rights.
//             PIPL Article 44 — right to explanation of automated decision-making logic.
//
// This fragment gates DSAR-related actions and enforces the 15-business-day SLA.
//
// Hyperscaler precedent:
//   Tencent Cloud's personal information management portal (腾讯云个人信息管理中心) implements
//   a 15-business-day DSAR response SLA with automated subject verification (WeChat/EID) and
//   audit trail export. This is the canonical reference implementation for PIPL Article 45.

namespace pack::cn_pipl_2021;

// -----------------------------------------------------------------------
// Rule 1: Permit authenticated data subjects to request access to their
//         own PI_CN_PIPL and PI_CN_SENSITIVE data.
// -----------------------------------------------------------------------
permit(
  principal is oyatie::DataSubject,
  action in [
    oyatie::Action::"dsar::access",
    oyatie::Action::"dsar::copy",
    oyatie::Action::"dsar::portability"
  ],
  resource
)
when {
  resource.tenant_id == principal.tenant_id &&
  resource.subject_id == principal.id &&
  resource.data_class in ["PI_CN_PIPL", "PI_CN_SENSITIVE"] &&
  context.subject_identity_verified == true
}
advice {
  "audit_event_class": "CnPiplDsarAccessPermit",
  "sla_deadline_business_days": 15,
  "regulation_reference": "PIPL Article 45 (right to access and copy personal information; 15-business-day response deadline)"
};

// -----------------------------------------------------------------------
// Rule 2: Forbid DSAR response delay beyond 15 business days.
//
// The DSAR workflow engine injects context.dsar_business_days_elapsed at
// each evaluation step. This rule fires if the deadline is exceeded and
// no extension has been granted.
// -----------------------------------------------------------------------
forbid(
  principal is oyatie::Operator,
  action == oyatie::Action::"dsar::defer",
  resource
)
when {
  resource.data_class in ["PI_CN_PIPL", "PI_CN_SENSITIVE"] &&
  context.dsar_business_days_elapsed >= 15 &&
  context.dsar_extension_granted != true
}
advice {
  "reason": "DSAR response deadline of 15 business days (PIPL Article 45) has been exceeded. Deferral is no longer permitted. Immediate response or escalation required.",
  "audit_event_class": "CnPiplDsarDeadlineExceededDeny",
  "regulation_reference": "PIPL Article 45 (15-business-day response deadline)"
};

// -----------------------------------------------------------------------
// Rule 3: Permit erasure requests from authenticated data subjects.
// -----------------------------------------------------------------------
permit(
  principal is oyatie::DataSubject,
  action == oyatie::Action::"dsar::erasure",
  resource
)
when {
  resource.subject_id == principal.id &&
  resource.data_class in ["PI_CN_PIPL", "PI_CN_SENSITIVE"] &&
  context.subject_identity_verified == true &&
  context.legal_hold_active != true
}
advice {
  "audit_event_class": "CnPiplDsarErasurePermit",
  "regulation_reference": "PIPL Article 47 (right to erasure; controller must delete within reasonable time)"
};

// -----------------------------------------------------------------------
// Rule 4: Permit automated-decision explanation requests.
// -----------------------------------------------------------------------
permit(
  principal is oyatie::DataSubject,
  action == oyatie::Action::"dsar::automated-decision-explanation",
  resource
)
when {
  resource.subject_id == principal.id &&
  context.subject_identity_verified == true
}
advice {
  "audit_event_class": "CnPiplDsarAutomatedDecisionExplanationPermit",
  "regulation_reference": "PIPL Article 44 (right to explanation of automated decision-making rules and logic)"
};
