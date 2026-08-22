// cloud-k8s µservice — tenant-scope policy fragment
// Cedar v4 schema; conforms to microservices/cloud-k8s/policy/schema.cedarschema (Slice D)
//
// Purpose: enforce per-tenant scope on every Kubernetes API call mediated by
//          `kubernetes-api-proxy`. Tenant operators may only operate on resources
//          in their own namespace; cross-tenant reads / writes / watches are refused.
// Owner:   ops-security + axis-cloud
// Audit:   every deny emits cloud_k8s_tenant_unauthorized_call_total
// References: policy/cluster-isolation.md §"Namespace Tenancy Model" + §"Invariant CI-08"
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// PERMIT 1 — tenant operator reads / writes within own namespace
// =============================================================================
//
// A principal bound to TenantOperator::"<tenant-id>" may act on any
// KubernetesResource whose namespace matches "tenant-<tenant_id>".

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"get",
    Action::"list",
    Action::"watch",
    Action::"create",
    Action::"update",
    Action::"patch",
    Action::"delete"
  ],
  resource in KubernetesResource::?r
)
when {
  principal has tenant_id &&
  resource has namespace &&
  resource.namespace == ("tenant-" + principal.tenant_id)
};

// =============================================================================
// PERMIT 2 — tenant operator reads own tenant's NetworkPolicy / AuthorizationPolicy
// =============================================================================
//
// These are derived from Cedar fragments; tenant can READ to verify but cannot
// edit (writes go through PR review per branch-protection).

permit (
  principal in TenantOperator::?p,
  action in [Action::"get", Action::"list", Action::"watch"],
  resource in KubernetesResource::?r
)
when {
  principal has tenant_id &&
  resource has namespace &&
  resource.namespace == ("tenant-" + principal.tenant_id) &&
  resource has kind &&
  (resource.kind == "NetworkPolicy" || resource.kind == "AuthorizationPolicy")
};

// =============================================================================
// FORBID — explicit cross-namespace deny (defence-in-depth)
// =============================================================================

forbid (
  principal in TenantOperator::?p,
  action,
  resource in KubernetesResource::?r
)
when {
  principal has tenant_id &&
  resource has namespace &&
  resource.namespace != ("tenant-" + principal.tenant_id)
};

// =============================================================================
// FORBID — tenant operators cannot exec into pods on production-tier namespaces
// =============================================================================
//
// Per threat-model T-I-05: kubectl exec captures workload memory; refuse for
// production-tier namespaces. Operators with JIT elevation can override via
// ops-security 2-person rule (handled by separate Cedar fragment).

forbid (
  principal in TenantOperator::?p,
  action == Action::"exec",
  resource in KubernetesResource::?r
)
when {
  resource has tier &&
  resource.tier == "production"
};

// =============================================================================
// FORBID — tenants cannot touch reserved namespaces
// =============================================================================
//
// Reserved namespace prefixes per policy/cluster-isolation.md §"Reserved namespaces"

forbid (
  principal in TenantOperator::?p,
  action,
  resource in KubernetesResource::?r
)
when {
  resource has namespace &&
  (
    resource.namespace like "kube-*" ||
    resource.namespace like "istio-*" ||
    resource.namespace like "cilium-*" ||
    resource.namespace like "cosign-*" ||
    resource.namespace like "cloud-k8s-*" ||
    resource.namespace like "oya-*"
  )
};

// =============================================================================
// FORBID — never allow direct etcd access (only kube-apiserver via api-proxy)
// =============================================================================

forbid (
  principal,
  action,
  resource in KubernetesResource::?r
)
when {
  resource has kind &&
  resource.kind == "EtcdEndpoint"
};

// =============================================================================
// FORBID — never allow kubelet direct access
// =============================================================================

forbid (
  principal in TenantOperator::?p,
  action,
  resource in KubernetesResource::?r
)
when {
  resource has kind &&
  resource.kind == "KubeletEndpoint"
};

// =============================================================================
// FORBID — never expose SECRET-class resources (kubeadm tokens, signing keys)
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};
