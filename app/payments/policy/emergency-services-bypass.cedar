// payments µservice — emergency-services-bypass Cedar v4.2 LTS fragment
//
// Purpose: §3.2.5 row 1 — ensure payment flows critical to emergency services
//          are never blocked by rate-limits, fraud-score gates, or step-up auth.
//          Applies to: emergency-dispatch platforms, 911-call-centre billing,
//          hospital emergency-department payment corridors.
// Owner:   ops-security + ops-compliance + axis-payments
// References: documentation-rigor.md §3.2.5 row 1; threat-model.md T-E-01
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar
// @soak-window-seconds: 60   per ADR-0294
// @publisher-svid: spiffe://oyatie/payments/foundry-agent

// =============================================================================
// Rate-limit exemption — modeled in abuse-defence.cedar, not here
// =============================================================================
//
// The emergency rate-limit exemption for §3.2.5 rows 1 + 22 lives as an
// `unless` exception on the ANTI-BOT rate-limit forbid in abuse-defence.cedar
// (validated SVID + emergency_surge_token + tenant binding). Cedar gives
// forbids precedence over permits, so a competing permit here could never
// override that forbid; the exception must be expressed in the forbid itself.

// =============================================================================
// PERMIT — emergency-services bypass of fraud-score gate
// =============================================================================
//
// Fraud-score gate (score > 80 = deny) is bypassed for verified emergency
// operator principals. The SVID + audience_type double check ensures only
// legitimate emergency operators get this bypass — not spoofed requests.

permit (
  principal,
  action == Action::"charge.create",
  resource
)
when {
  principal has audience_type &&
  principal.audience_type == "EMERGENCY_SERVICES_OPERATOR" &&
  principal has svid_present &&
  principal.svid_present == true &&
  principal has svid_valid &&
  principal.svid_valid == true
}
unless {
  // Block if fraud-score is astronomically high (> 99) — absolute floor
  principal has fraud_score &&
  principal.fraud_score > 99
};

// =============================================================================
// PERMIT — emergency-services bypass of step-up auth for large payouts
// =============================================================================
//
// Hospital billing systems and emergency-dispatch platforms may need to process
// large invoices (> $10k) without interactive step-up. The step-up auth
// requirement in abuse-defence.cedar is overridden here for this audience type.

permit (
  principal,
  action == Action::"payout.execute",
  resource in Payout::?o
)
when {
  principal has audience_type &&
  principal.audience_type == "EMERGENCY_SERVICES_OPERATOR" &&
  principal has svid_present &&
  principal.svid_present == true &&
  principal has svid_valid &&
  principal.svid_valid == true &&
  // Retain the same positive screening evidence the ordinary payout-worker
  // permit requires: the emergency exception lifts step-up, not the bank
  // verification / cooling-period / sanctions / KYC controls.
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has bank_account_state &&
  resource.bank_account_state == "verified" &&
  resource has cooling_period_elapsed &&
  resource.cooling_period_elapsed == true &&
  resource has recipient_sanctions_match &&
  resource.recipient_sanctions_match == false &&
  resource has kyc_revoked &&
  resource.kyc_revoked == false
};

// =============================================================================
// PERMIT — mass-casualty surge: emergency payment actions for emergency ops
// =============================================================================
//
// §3.2.5 row 22: mass-casualty surge may spike payment volume 10-100x normal.
// The permit enumerates the exact emergency PAYMENT actions (charge create/
// capture and payout execute — the money-movement flows that must not stall)
// and binds the principal to the resource's tenant when both carry a tenant,
// so an unrelated read or administrative action cannot ride the surge token.
// Normal bot-score threshold still applies (bot_score > 95 blocks).

permit (
  principal,
  action in [
    Action::"charge.create",
    Action::"charge.capture",
    Action::"payout.execute"
  ],
  resource
)
when {
  principal has audience_type &&
  principal.audience_type == "EMERGENCY_SERVICES_OPERATOR" &&
  principal has emergency_surge_token &&
  principal.emergency_surge_token != "" &&
  principal has svid_present &&
  principal.svid_present == true &&
  principal has svid_valid &&
  principal.svid_valid == true &&
  (!(principal has tenant_id && resource has tenant_id) ||
    principal.tenant_id == resource.tenant_id)
}
unless {
  principal has bot_score &&
  principal.bot_score > 95
};
