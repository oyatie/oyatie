# cloud-k8s

The cloud-k8s substrate owns Kubernetes cluster bootstrap, node lifecycle,
network-policy application, service-mesh control-plane integration, ingress,
CSI integration, and Kubernetes API proxy surfaces.

## Tenant Class Model

cloud-k8s follows ADR-0330. The service no longer models customer capability
levels. Runtime differences are expressed through:

- `tenant_class`: `demo_trial` or `paid`
- `billing_components`: `revenue_share`, `per_seat`, `per_usage`
- `cell_topology`: shared-cloud, dedicated-cloud, hybrid, on-prem connected,
  or on-prem air-gapped placement
- `compliance_pack`: regulatory overlays that require pack-bound custody

`demo_trial` defaults to the OCI Always Free profile with time and usage caps.
`paid` tenants use the same product surface with commercial shape carried by
`billing_components`; resilience and custody requirements belong to
`cell_topology` or `compliance_pack`.

Canonical model: `docs/decisions/ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md`.
