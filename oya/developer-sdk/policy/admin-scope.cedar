// developer-sdk µservice — platform-admin-scope policy
// Cedar v4 schema
// Purpose: platform admins can review KYC queue, review payout exceptions, revoke developers.

forbid (principal, action, resource);

permit (
  principal in PlatformAdmin::?p,
  action in [
    Action::"list_kyc_queue",
    Action::"get_kyc_record",
    Action::"approve_kyc",
    Action::"reject_kyc",
    Action::"list_payout_exceptions",
    Action::"approve_payout_above_threshold",
    Action::"revoke_developer"
  ],
  resource
)
when {
  principal has admin_role
};

permit (
  principal in TaxComplianceReviewer::?p,
  action in [
    Action::"export_1099_misc_batch",
    Action::"export_vat_moss_batch",
    Action::"export_vat_kr_batch"
  ],
  resource
);
