// Retired external CD UI, API, and sync authorization boundary retained for retirement inventory only.
permit(
  principal,
  action in [
    Oyatie::Action::"retired-cd.ui.read",
    Oyatie::Action::"retired-cd.application.read",
    Oyatie::Action::"retired-cd.application.sync"
  ],
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.context == "aws-guest" &&
  principal.roles.contains(Oyatie::Role::"cd-contributor")
};

permit(
  principal,
  action in [
    Oyatie::Action::"retired-cd.project.admin",
    Oyatie::Action::"retired-cd.cluster.register",
    Oyatie::Action::"retired-cd.policy.configure"
  ],
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.context == "aws-guest" &&
  principal.roles.contains(Oyatie::Role::"platform-cd-admin")
};

forbid(
  principal,
  action,
  resource
) when {
  principal.tenant_id != resource.tenant_id || resource.context != "aws-guest"
};
