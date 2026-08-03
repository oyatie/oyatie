# Managed K8s Cluster Lifecycle — Cost / FinOps

## Cost model (ADR-0340 declaration)

| Dimension | Cluster-lifecycle value |
|-----------|-------------------------|
| Admission compute | Request validation plus one quota-decision port call |
| Backend actuation | One control-plane-host port call only after quota allow |
| Persistent storage owned here | None in the current deterministic foundation |
| Scaling dimension | Per cluster lifecycle request |
| Cell placement class | Dogfood/design foundation; no production placement claim |

Cluster-lifecycle does not own quota-service metering, billing emission, or quota
storage. Cost references in this service are admission-budget and dependency
framing only: quota must allow before provisioning, and lifecycle calls must stay
bounded so follow-on FinOps attribution can connect lifecycle operations to the
tenant-quota and control-plane-host services without making billing-readiness
claims here.
