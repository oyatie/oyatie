# Managed K8s SLA Observability — Tenant Isolation

## Isolation Guarantees (ADR-0376 / ADR-0007)

1. **Tenant-cluster keying**: Every snapshot, summary, and evidence handle is
   scoped by `(tenant_id, cluster_name)` before it is loaded or aggregated.

2. **Default-deny read scope**: Tenant summary and evidence reads require an
   authenticated tenant/account/project scope matching the requested tenant and
   cluster. A principal without a matching scope is denied before summary lookup.

3. **No cross-tenant rollup leakage**: Fleet, region, and cell summaries aggregate
   only records already authorized for the same tenant context.

4. **No secret-bearing evidence**: Evidence handles may reference collector runs,
   trace IDs, and OpenSLO records, but must not expose raw kubeconfigs, provider
   credentials, bearer tokens, or other tenant secrets.

5. **Tenant-zero parity**: Dogfood/tenant-zero traffic follows the same read and
   evidence rules as any customer tenant; no internal bypass is authoritative.
