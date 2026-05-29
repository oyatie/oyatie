// plugin-app-store µservice — platform-admin-scope policy
// Cedar v4 schema
// Purpose: only platform admins can vet, approve, reject, or revoke plugin versions across tenants.

forbid (principal, action, resource);

permit (
  principal in PlatformAdmin::?p,
  action in [
    Action::"list_vetting_queue",
    Action::"get_submission",
    Action::"approve_submission",
    Action::"reject_submission",
    Action::"revoke_plugin",
    Action::"view_vetting_metrics"
  ],
  resource
)
when {
  principal has admin_role &&
  principal.admin_role in [AdminRole::"platform_admin", AdminRole::"vetting_reviewer", AdminRole::"security_council"]
};

permit (
  principal in SecurityCouncil::?p,
  action == Action::"emergency_revoke_plugin",
  resource
)
when {
  principal has council_member &&
  principal.council_member == true
};
