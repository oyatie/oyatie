# performance-management — guest-on-aws

Customer-owned AWS VPC; Oyatie operates the service. Use when customer requires data
residency in their own AWS account.

## Provisioning

```bash
tofu init
tofu apply -var="tenant_id=<uuid>" -var="tenant_class=paid" \
           -var="vpc_id=vpc-..." -var="eks_cluster_name=customer-eks"
```
