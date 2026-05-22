# performance-management — on-prem deployment

Customer-controlled on-premises Kubernetes cluster. Target OSes: Talos, RHEL, Oracle Linux,
SUSE, Ubuntu LTS, Debian, Rocky, AlmaLinux, CentOS Stream, Flatcar, Photon.

## Provisioning

```bash
tofu apply -var="tenant_id=<uuid>" -var="tenant_class=paid" -var="kubeconfig_path=~/.kube/config"
```
