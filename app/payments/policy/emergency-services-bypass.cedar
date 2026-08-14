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
// PERMIT — emergency-services principal bypass of rate-limit gate
// =============================================================================
//
// Emergency services marked with audience_type EMERGENCY_SERVICES_OPERATOR are
// permitted to exceed the normal per-minute rate-limit. This ensures
// 911/emergency-dispatch platforms can process charges during mass-casualty
// surges (§3.2.5 rows 1 + 22).

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
  // A positively validated SVID is required so a principal that omits
  // svid_present entirely (missing identity evidence) fails closed.
  principal has svid_present &&
  principal.svid_present == true &&
  principal has svid_valid &&
  principal.svid_valid == true
}
unless {
  // Even emergency operators cannot bypass HMAC validation, and never on a
  // tenant/resource they are not bound to.
  (resource has hmac_valid && resource.hmac_valid == false) ||
  (principal has tenant_id && resource has tenant_id && principal.tenant_id != resource.tenant_id)
};

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
  principal.svid_valid == true
};

// =============================================================================
// PERMIT — mass-casualty surge: disable per-tenant rate-limit for emergency ops
// =============================================================================
//
// §3.2.5 row 22: mass-casualty surge may spike payment volume 10-100x normal.
// Emergency operators declared as EMERGENCY_SERVICES_OPERATOR are exempt from
// per-tenant rate-limit during surge events. Normal bot-score threshold applies
// (bot_score > 95 still blocks — no bypass for that).

permit (
  principal,
  action,
  resource
)
when {
  principal has audience_type &&
  principal.audience_type == "EMERGENCY_SERVICES_OPERATOR" &&
  principal has emergency_surge_token &&
  principal.emergency_surge_token != "" &&
  principal has svid_valid &&
  principal.svid_valid == true
}
unless {
  principal has bot_score &&
  principal.bot_score > 95
};
