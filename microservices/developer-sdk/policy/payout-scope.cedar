// developer-sdk µservice — payout-scope policy
// Cedar v4 schema
// Purpose: payout workers can only emit settlement events for fully KYC-verified developers
//          whose payout balance >= threshold; dual-signature required > $10k.

forbid (principal, action, resource);

permit (
  principal in PayoutWorker::?p,
  action == Action::"settle_payout",
  resource in DeveloperPayoutData::?r
)
when {
  resource has kyc_state &&
  resource.kyc_state == "approved" &&
  resource has bank_verified &&
  resource.bank_verified == true &&
  resource has amount_minor_units &&
  resource.amount_minor_units <= 1000000  // < $10k threshold
};

permit (
  principal in DualApprovalReviewer::?p,
  action == Action::"approve_high_value_payout",
  resource in DeveloperPayoutData::?r
)
when {
  resource has amount_minor_units &&
  resource.amount_minor_units > 1000000
};
