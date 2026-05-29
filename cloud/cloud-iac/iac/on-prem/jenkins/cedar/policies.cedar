// ADR-0349 Jenkins web UI and API authorization boundary.
permit(
  principal,
  action in [
    Oyatie::Action::"jenkins.ui.read",
    Oyatie::Action::"jenkins.job.read",
    Oyatie::Action::"jenkins.job.build"
  ],
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.context == "on-prem" &&
  principal.roles.contains(Oyatie::Role::"ci-contributor")
};

permit(
  principal,
  action in [
    Oyatie::Action::"jenkins.admin.read",
    Oyatie::Action::"jenkins.admin.write",
    Oyatie::Action::"jenkins.api.configure"
  ],
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.context == "on-prem" &&
  principal.roles.contains(Oyatie::Role::"platform-ci-admin")
};

forbid(
  principal,
  action,
  resource
) when {
  principal.tenant_id != resource.tenant_id || resource.context != "on-prem"
};
