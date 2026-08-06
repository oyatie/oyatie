// ADR-0349 ArgoCD UI, API, and sync authorization boundary.
permit(
  principal,
  action in [
    Oyatie::Action::"argocd.ui.read",
    Oyatie::Action::"argocd.application.read",
    Oyatie::Action::"argocd.application.sync"
  ],
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.context == "colo" &&
  principal.roles.contains(Oyatie::Role::"cd-contributor")
};

permit(
  principal,
  action in [
    Oyatie::Action::"argocd.project.admin",
    Oyatie::Action::"argocd.cluster.register",
    Oyatie::Action::"argocd.policy.configure"
  ],
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.context == "colo" &&
  principal.roles.contains(Oyatie::Role::"platform-cd-admin")
};

forbid(
  principal,
  action,
  resource
) when {
  principal.tenant_id != resource.tenant_id || resource.context != "colo"
};
