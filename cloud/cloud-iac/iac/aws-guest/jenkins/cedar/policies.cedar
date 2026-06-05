// Retired external CI web UI and API authorization boundary retained for retirement inventory only.
permit(
  principal,
  action in [
    Oyatie::Action::"retired-ci.ui.read",
    Oyatie::Action::"retired-ci.job.read",
    Oyatie::Action::"retired-ci.job.build"
  ],
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.context == "aws-guest" &&
  principal.roles.contains(Oyatie::Role::"ci-contributor")
};

permit(
  principal,
  action in [
    Oyatie::Action::"retired-ci.admin.read",
    Oyatie::Action::"retired-ci.admin.write",
    Oyatie::Action::"retired-ci.api.configure"
  ],
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.context == "aws-guest" &&
  principal.roles.contains(Oyatie::Role::"platform-ci-admin")
};

forbid(
  principal,
  action,
  resource
) when {
  principal.tenant_id != resource.tenant_id || resource.context != "aws-guest"
};
