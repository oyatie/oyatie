// Cedar policy fragments for managed-k8s-control-plane-host (ADR-0376).
//
// Authorization doctrine: ONLY platform / control-plane-operator principals may
// provision or tear down a tenant control plane. Tenants are DEFAULT-DENY on the
// control-plane-host concern — a tenant manages its cluster as a resource via the
// managed-k8s-cluster-lifecycle API, never the control-plane host directly.
// This is the core mitigation for the threat model's central risk: a tenant
// reaching the management cluster or a peer tenant's control plane.
//
// Cedar semantics: default-deny + forbid-overrides-permit (formally verified,
// arXiv 2403.04651). The explicit forbid below is belt-and-suspenders against the
// management-cluster escalation path even if a permit is ever mis-authored.

// PERMIT: platform control-plane operators may provision/observe/teardown.
permit(
  principal in Role::"platform-control-plane-operator",
  action in [
    Action::"ControlPlaneProvision",
    Action::"ControlPlaneStatus",
    Action::"ControlPlaneTeardown"
  ],
  resource is TenantControlPlane
) when {
  resource.microservice == "managed-k8s-control-plane-host" &&
  context.management_cluster_scope == true &&
  context.audit_chain_emit == true
};

// PERMIT (read-only): the sibling cluster-lifecycle + sla-observability services
// may read control-plane status (no mutation).
permit(
  principal in Role::"managed-k8s-platform-service",
  action == Action::"ControlPlaneStatus",
  resource is TenantControlPlane
) when {
  resource.microservice == "managed-k8s-control-plane-host"
};

// FORBID: a tenant principal may NEVER act on the control-plane host concern,
// regardless of any permit. Tenants reach the host only transitively via the
// cluster-lifecycle API under the platform service identity.
forbid(
  principal in Role::"tenant",
  action in [
    Action::"ControlPlaneProvision",
    Action::"ControlPlaneStatus",
    Action::"ControlPlaneTeardown"
  ],
  resource is TenantControlPlane
);

// FORBID: any provision/teardown that is not scoped to the management cluster, or
// that does not emit an audit-chain seal, is denied (fail-closed; ADR-0376
// operational boundary + audit-chain seal_events contract).
forbid(
  principal,
  action in [Action::"ControlPlaneProvision", Action::"ControlPlaneTeardown"],
  resource is TenantControlPlane
) when {
  context.management_cluster_scope != true ||
  context.audit_chain_emit != true
};
