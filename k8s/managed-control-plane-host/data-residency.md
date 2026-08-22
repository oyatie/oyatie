# Data residency — `managed-k8s-control-plane-host`

**Authority:** ADR-0376, ADR-0009 (per-tenant per-region cells).

## Data handled

This service handles ONLY control-plane identity + lifecycle metadata:

| Datum | Class | Notes |
|-------|-------|-------|
| `tenant_id` | TENANT_SCOPED | tenant identity; no PII content |
| `cluster_name` | TENANT_SCOPED | tenant-assigned cluster name |
| `handle` | TENANT_SCOPED | adapter-issued opaque control-plane handle |
| `tier` / `datastore_class` / `status` | INTERNAL_ONLY | lifecycle metadata |
| `endpoint` | TENANT_SCOPED | tenant control-plane API-server URL |
| operator error detail | INTERNAL_ONLY | never carries kubeconfig/secret material |

**No tenant PII. No tenant workload data.** It does not store or process the
contents of any tenant cluster.

## Residency posture

- The service runs in the management cluster of the tenant's assigned cell
  (ADR-0009 per-tenant per-region cell); the control-plane metadata it holds is
  resident in that cell's region.
- **Hosted tier:** the tenant control plane (pods + datastore) is resident in the
  management cluster's region. For residency-constrained tenants, the cell
  placement (ADR-0009) determines the region; cross-jurisdiction placement
  requires the ADR-0243 Cedar migration permit (out of scope for this lane).
- **Dedicated tier:** the Talos spoke is resident wherever the tenant's dedicated
  hardware/cell is — the strongest residency + sovereignty story (air-gap-capable
  per ADR-0375).

## Deferred

The DPIA for HOSTING tenant control planes is explicitly deferred to
`managed-k8s-commercial-ga` (ADR-0376). This lane does not make a DPIA claim;
it records the data classes the live integration must cover.
