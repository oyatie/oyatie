# performance-management — oyatie-iaas deployment

Oyatie as a cloud provider. Runs on the `cloud-*` substrate µservices (which are Oyatie's
own IaaS surface) per ADR-0254. Cloud Hypervisor + Kata pods give VM-grade isolation
inside the Kubernetes cluster.

## Provisioning

```bash
tofu apply -var="tenant_id=<uuid>" -var="tenant_class=paid"
```
