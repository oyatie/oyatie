// cloud-k8s µservice — public-read policy fragment
// Cedar v4
//
// Purpose: explicit allow-list for anonymous (no OIDC) reads. Used for liveness /
//          readiness probes and the cluster's public health surface. Everything
//          else is denied by default (per Cedar deny-overrides semantics).
// Owner:   axis-cloud + ops-security
// References: policy/cluster-isolation.md §"Invariant CI-08"
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// PERMIT 1 — anonymous health / readiness probes
// =============================================================================
//
// Kubernetes liveness + readiness probes are anonymous (no token). Cedar
// permits only the exact /health and /ready paths on the api-proxy surface.

permit (
  principal == Anonymous::"public",
  action == Action::"get",
  resource in HttpEndpoint::?e
)
when {
  resource has path &&
  (resource.path == "/health" || resource.path == "/ready" || resource.path == "/metrics-public")
};

// =============================================================================
// PERMIT 2 — anonymous cluster-version + supported-versions read
// =============================================================================
//
// Public surface: the cluster's reported Kubernetes version + the supported-
// version window. Used for marketing + tenant SDK compatibility checks.

permit (
  principal == Anonymous::"public",
  action == Action::"get",
  resource in HttpEndpoint::?e
)
when {
  resource has path &&
  resource.path == "/cluster-version"
};

// =============================================================================
// FORBID — never expose tenant data anonymously
// =============================================================================

forbid (
  principal == Anonymous::"public",
  action,
  resource in KubernetesResource::?r
)
when {
  resource has namespace &&
  resource.namespace like "tenant-*"
};

// =============================================================================
// FORBID — never expose reserved-namespace data anonymously
// =============================================================================

forbid (
  principal == Anonymous::"public",
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
// FORBID — anonymous writes always
// =============================================================================

forbid (
  principal == Anonymous::"public",
  action in [Action::"create", Action::"update", Action::"patch", Action::"delete", Action::"exec"],
  resource
);
