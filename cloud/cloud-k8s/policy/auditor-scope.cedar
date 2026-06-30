// cloud-k8s µservice — auditor-scope policy fragment
// Cedar v4
//
// Purpose: scope external auditors (SOC 2 / ISO 27001 / GDPR DPA / KR PIPC /
//          HIPAA OCR / KR-CSAP) to read-only access on their specific tenant's
//          cluster-related artifacts, time-boxed via JIT token.
// Owner:   ops-security + council-privacy
// References: policy/cluster-isolation.md §"Audit Trail"; threat-model T-S-05, T-L-05
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// PERMIT 1 — auditor reads scoped-tenant cluster state
// =============================================================================
//
// Auditor JIT token issued by OpenBao carries:
//   - tenant_scope: list of tenants in engagement scope (NOT "all tenants")
//   - engagement_window: ISO-8601 interval; out-of-window access auto-rejected
//   - engagement_id: ULID for audit-trail correlation

permit (
  principal in Auditor::?a,
  action in [Action::"get", Action::"list", Action::"watch", Action::"read_logs"],
  resource in KubernetesResource::?r
)
when {
  principal has tenant_scope &&
  principal has engagement_window_start &&
  principal has engagement_window_end &&
  resource has namespace &&
  resource.namespace like "tenant-*" &&
  principal.tenant_scope.contains(substring(resource.namespace, 7, 23)) &&
  context.request_ts >= principal.engagement_window_start &&
  context.request_ts <= principal.engagement_window_end
};

// =============================================================================
// PERMIT 2 — auditor reads cluster-wide non-tenant artifacts (policies, configs)
// =============================================================================
//
// Auditor verifying CIS K8s benchmark conformance reads cluster-wide config.
// These reads are not tenant-specific; permitted within engagement window.

permit (
  principal in Auditor::?a,
  action in [Action::"get", Action::"list"],
  resource in KubernetesResource::?r
)
when {
  principal has engagement_window_start &&
  principal has engagement_window_end &&
  context.request_ts >= principal.engagement_window_start &&
  context.request_ts <= principal.engagement_window_end &&
  resource has kind &&
  (
    resource.kind == "Node" ||
    resource.kind == "Namespace" ||
    resource.kind == "ClusterPolicy" ||
    resource.kind == "PeerAuthentication" ||
    resource.kind == "StorageClass" ||
    resource.kind == "ClusterRole" ||
    resource.kind == "ClusterRoleBinding"
  )
};

// =============================================================================
// PERMIT 3 — auditor reads audit-chain via Mimir (cluster-wide audit log) for
// engagement-scoped tenants
// =============================================================================

permit (
  principal in Auditor::?a,
  action == Action::"read_audit_log",
  resource in AuditRecord::?r
)
when {
  principal has tenant_scope &&
  resource has tenant_id &&
  principal.tenant_scope.contains(resource.tenant_id) &&
  principal has engagement_window_start &&
  principal has engagement_window_end &&
  context.request_ts >= principal.engagement_window_start &&
  context.request_ts <= principal.engagement_window_end
};

// =============================================================================
// FORBID — auditor never has write access
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [Action::"create", Action::"update", Action::"patch", Action::"delete", Action::"exec"],
  resource
);

// =============================================================================
// FORBID — auditor never reads SECRET-class resources
// =============================================================================

forbid (
  principal in Auditor::?a,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};

// =============================================================================
// FORBID — auditor never reads outside engagement window
// =============================================================================

forbid (
  principal in Auditor::?a,
  action,
  resource
)
unless {
  principal has engagement_window_start &&
  principal has engagement_window_end &&
  context.request_ts >= principal.engagement_window_start &&
  context.request_ts <= principal.engagement_window_end
};

// =============================================================================
// FORBID — auditor never pivots beyond engagement-scoped tenants
// =============================================================================

forbid (
  principal in Auditor::?a,
  action,
  resource in KubernetesResource::?r
)
when {
  principal has tenant_scope &&
  resource has namespace &&
  resource.namespace like "tenant-*" &&
  !principal.tenant_scope.contains(substring(resource.namespace, 7, 23))
};
