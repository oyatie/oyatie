# performance-management — colo deployment

Single-tenant colocation deployment for tenants whose contracts require physical isolation
in a named colo facility.

## Provisioning

```bash
tofu apply -var="tenant_id=<uuid>" -var="tenant_class=paid" \
           -var="colo_location=equinix-dc11" -var="kubeconfig_path=~/.kube/config"
```
