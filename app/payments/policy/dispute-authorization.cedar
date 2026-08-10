// payments µservice — dispute-authorization Cedar v4.2 LTS fragment
//
// Purpose: gate all dispute operations: receive, acknowledge, submit evidence,
//          accept, escalate. Default-deny; permit only verifiable principals
//          acting within their mandate.
// Owner:   ops-fraud + axis-payments
// References: ARCHITECTURE.md §cedar-gates; compliance.md §1; IP-009; IP-010
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar
// @soak-window-seconds: 60   per ADR-0294
// @publisher-svid: spiffe://oyatie/payments/foundry-agent

// =============================================================================
// Default deny (baseline — this fragment adds permits on top)
// =============================================================================

// =============================================================================
// PERMIT — dispute webhook receiver: ingest inbound PSP dispute webhook
// =============================================================================

permit (
  principal in PaymentsWebhookReceiver::?r,
  action == Action::"dispute.receive",
  resource in DisputeWebhookPayload::?w
)
when {
  principal has svid_present && principal.svid_present == true &&
  resource has hmac_valid && resource.hmac_valid == true &&
  resource has webhook_age_seconds && resource.webhook_age_seconds <= 300
};

// =============================================================================
// PERMIT — tenant dispute handler: read dispute, submit evidence
// =============================================================================

permit (
  principal,
  action in [
    Action::"dispute.read",
    Action::"dispute.submit-evidence",
    Action::"dispute.accept"
  ],
  resource in Dispute::?d
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has kyb_verified && principal.kyb_verified == true &&
  principal has svid_valid && principal.svid_valid == true
};

// =============================================================================
// PERMIT — ops-fraud: escalate dispute, read all disputes
// =============================================================================

permit (
  principal in OyaFraudOps::?o,
  action in [
    Action::"dispute.read",
    Action::"dispute.escalate",
    Action::"dispute.override-elder-abuse-flag"
  ],
  resource in Dispute::?d
)
when {
  principal has svid_present && principal.svid_present == true &&
  principal has role && principal.role == "ops-fraud"
};

// =============================================================================
// FORBID — block dispute evidence submission after evidence_due_at deadline
// =============================================================================

forbid (
  principal,
  action == Action::"dispute.submit-evidence",
  resource in Dispute::?d
)
when {
  resource has evidence_window_expired &&
  resource.evidence_window_expired == true
};

// =============================================================================
// FORBID — block dispute acceptance for elder-abuse-flagged disputes
//          without ops-fraud approval (§3.2.5 row 4)
// =============================================================================

forbid (
  principal,
  action == Action::"dispute.accept",
  resource in Dispute::?d
)
when {
  resource has elder_abuse_flag &&
  resource.elder_abuse_flag == true &&
  !(principal has role && principal.role == "ops-fraud")
};

// =============================================================================
// FORBID — block dispute ops for COPPA principals (<13)
// =============================================================================

forbid (
  principal,
  action in [Action::"dispute.submit-evidence", Action::"dispute.accept"],
  resource
)
when {
  principal has age_class &&
  principal.age_class == "under_13"
};
