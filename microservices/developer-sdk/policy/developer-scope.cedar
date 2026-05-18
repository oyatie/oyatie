// developer-sdk µservice — developer-scope policy
// Cedar v4 schema
// Purpose: developer can only act on their own developer record + their own plugins + their own sandbox.

forbid (principal, action, resource);

permit (
  principal in Developer::?d,
  action in [
    Action::"get_developer",
    Action::"submit_id_document",
    Action::"submit_liveness_proof",
    Action::"add_bank_account",
    Action::"issue_signing_key",
    Action::"revoke_signing_key",
    Action::"list_signing_keys",
    Action::"provision_sandbox",
    Action::"reset_sandbox",
    Action::"download_sdk",
    Action::"get_payout_balance",
    Action::"stream_payout_ledger",
    Action::"download_tax_form",
    Action::"submit_plugin_version",
    Action::"stream_submission_status"
  ],
  resource in DeveloperData::?r
)
when {
  principal has developer_id &&
  resource has developer_id &&
  principal.developer_id == resource.developer_id
};
