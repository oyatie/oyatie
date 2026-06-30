// cloud-k8s µservice — CI-scope policy fragment
// Cedar v4
//
// Purpose: scope the oya-ci principal's Kubernetes API access to cluster-readonly
//          observations + writes confined to the `oya-ci` namespace.
// Owner:   axis-platform + axis-cloud + ops-security
// References: policy/cluster-isolation.md §"Reserved namespaces"; ADR-0123 HG-CLOUD-K8S
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// PERMIT 1 — CI principal reads cluster state (read-only, cluster-wide)
// =============================================================================
//
// CI lanes (oya-cloud-k8s-iac-smoke, oya-check-cis-k8s-benchmark, etc.) read
// cluster state to verify conformance.

permit (
  principal == Service::"oya-ci",
  action in [Action::"get", Action::"list", Action::"watch"],
  resource in KubernetesResource::?r
)
when {
  resource has kind &&
  (
    resource.kind == "Pod" ||
    resource.kind == "Node" ||
    resource.kind == "Namespace" ||
    resource.kind == "Service" ||
    resource.kind == "NetworkPolicy" ||
    resource.kind == "AuthorizationPolicy" ||
    resource.kind == "PeerAuthentication" ||
    resource.kind == "ClusterPolicy" ||
    resource.kind == "StorageClass" ||
    resource.kind == "PersistentVolume" ||
    resource.kind == "PersistentVolumeClaim" ||
    resource.kind == "Deployment" ||
    resource.kind == "StatefulSet" ||
    resource.kind == "DaemonSet" ||
    resource.kind == "ConfigMap" ||
    resource.kind == "ServiceAccount"
  )
};

// =============================================================================
// PERMIT 2 — CI principal writes in oya-ci namespace only
// =============================================================================

permit (
  principal == Service::"oya-ci",
  action in [Action::"create", Action::"update", Action::"patch", Action::"delete"],
  resource in KubernetesResource::?r
)
when {
  resource has namespace &&
  resource.namespace == "oya-ci"
};

// =============================================================================
// PERMIT 3 — CI principal validates admission webhooks via dry-run
// =============================================================================

permit (
  principal == Service::"oya-ci",
  action == Action::"create",
  resource in KubernetesResource::?r
)
when {
  resource has dry_run &&
  resource.dry_run == true
};

// =============================================================================
// FORBID — CI principal cannot exec into pods
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action in [Action::"exec", Action::"attach", Action::"portforward"],
  resource
);

// =============================================================================
// FORBID — CI principal cannot touch SECRET-class resources
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};

// =============================================================================
// FORBID — CI principal cannot read tenant namespaces' Pod logs
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action == Action::"read_logs",
  resource in KubernetesResource::?r
)
when {
  resource has namespace &&
  resource.namespace like "tenant-*"
};

// =============================================================================
// FORBID — CI principal cannot write to reserved namespaces (except oya-ci)
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action in [Action::"create", Action::"update", Action::"patch", Action::"delete"],
  resource in KubernetesResource::?r
)
when {
  resource has namespace &&
  resource.namespace != "oya-ci" &&
  (
    resource.namespace like "kube-*" ||
    resource.namespace like "istio-*" ||
    resource.namespace like "cilium-*" ||
    resource.namespace like "cosign-*" ||
    resource.namespace like "cloud-k8s-*" ||
    resource.namespace like "oya-*" ||
    resource.namespace like "tenant-*"
  )
};
