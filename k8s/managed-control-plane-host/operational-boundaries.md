# Operational boundaries — `managed-k8s-control-plane-host`

**Authority:** ADR-0376.

## Hard invariants

1. **Management-cluster-only.** This service runs in, and connects ONLY to,
   Oyatie's management (control-plane) cluster. It reconciles tenant control
   planes (hosted Kamaji `TenantControlPlane` pods; dedicated Talos spoke
   references) from the management plane. It is enforced at boot: the production
   `[[bin]]` reads the management kubeconfig from `$OYATIE_MGMT_KUBECONFIG` and
   refuses to start if absent (`BootError::MissingMgmtKubeconfig`) — it never
   silently falls back to the in-memory fake.
2. **Never runs tenant workloads.** This is a control-plane management service. It
   does not schedule, host, or proxy any tenant workload. It holds NO
   tenant-cluster kubeconfig and opens NO connection to a tenant cluster's API
   server.
3. **No data-plane path.** The only data it handles is control-plane identity
   (tenant_id, cluster_name, opaque handle, tier, status, endpoint URL). No
   tenant PII, no tenant workload data, no secrets in the typed control flow
   (operator-facing error details are `INTERNAL_ONLY` and never carry kubeconfig
   material).
4. **Tenant-zero is not special.** Oyatie's own clusters (dogfood) are provisioned
   through the same path under an ordinary `ClusterRef.tenant_id`; there is no
   internal-bypass identity (ADR-0376 oyatie-dogfood-tenancy).

## Deployment posture

- Runs HA in the management cluster (ADR-0376 makes management-cluster HA a hard
  prerequisite for hosted-tier density).
- Mesh: Cilium L3/L4 + Istio Ambient ztunnel (ADR-0148); `manifest.json#mesh_layering`.
- Secrets: management kubeconfig + any provider credentials via OpenBao
  (`secrets_substrate`), never plaintext env/file.

## What this service is NOT

- Not the cluster-CRUD API (that is `managed-k8s-cluster-lifecycle`).
- Not the quota/RBAC enforcer (that is `managed-k8s-tenant-quota`).
- Not the SLA/uptime surface (that is `managed-k8s-sla-observability`).

It owns ONLY the control-plane-host concern and the `ControlPlaneProvisioning`
port those services consume.
